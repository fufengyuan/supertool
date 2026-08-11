/// WireGuard tunnel — 进程内异步转发。
///
/// 设计：macOS 上创建 TUN 设备需要 root，但隧道转发不需要。
/// 因此只用一个「瞬时提权辅助进程」创建 TUN 并通过 SCM_RIGHTS 把 fd 传回
/// 主进程；主进程在自身进程内（tokio task）异步运行 boringtun 转发循环，
/// 状态直接在内存共享。辅助进程创建 fd 后即退出，无常驻子进程。
use boringtun::noise::{Tunn, TunnResult};
use boringtun::x25519::{PublicKey, StaticSecret};
use serde::{Deserialize, Serialize};
use std::os::unix::io::{AsRawFd, RawFd};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tun2::Configuration;

/// Tunnel config passed from parent process via JSON file
#[derive(Debug, Deserialize)]
pub struct TunnelConfig {
    #[serde(rename = "configId")]
    pub config_id: String,
    #[serde(rename = "configName")]
    pub config_name: String,
    #[serde(rename = "privateKey")]
    pub private_key: String,
    #[serde(rename = "peerPublicKey")]
    pub peer_public_key: String,
    #[serde(rename = "peerEndpoint")]
    pub peer_endpoint: String,
    #[serde(rename = "presharedKey")]
    pub preshared_key: Option<String>,
    pub address: Option<String>,
    pub mtu: Option<i64>,
}

impl TunnelConfig {
    pub fn load(conf_path: &str) -> Result<Self, String> {
        let conf_text = std::fs::read_to_string(conf_path)
            .map_err(|e| format!("读取配置文件失败: {}", e))?;
        serde_json::from_str(&conf_text).map_err(|e| format!("解析配置失败: {}", e))
    }

    /// 解析密钥并构造 boringtun Tunn + UDP socket
    pub async fn build_tunnel(&self) -> Result<(Tunn, tokio::net::UdpSocket), String> {
        use base64::Engine;
        let private_bytes = base64::engine::general_purpose::STANDARD
            .decode(&self.private_key)
            .map_err(|e| format!("私钥解码失败: {}", e))?;
        if private_bytes.len() != 32 {
            return Err("私钥必须是 32 字节".to_string());
        }
        let mut sk = [0u8; 32];
        sk.copy_from_slice(&private_bytes);
        let static_private = StaticSecret::from(sk);

        let peer_bytes = base64::engine::general_purpose::STANDARD
            .decode(&self.peer_public_key)
            .map_err(|e| format!("对端公钥解码失败: {}", e))?;
        if peer_bytes.len() != 32 {
            return Err("对端公钥必须是 32 字节".to_string());
        }
        let mut pk = [0u8; 32];
        pk.copy_from_slice(&peer_bytes);
        let peer_public = PublicKey::from(pk);

        let psk: Option<[u8; 32]> = match self.preshared_key.as_deref() {
            Some(k) if !k.is_empty() => {
                let psk_bytes = base64::engine::general_purpose::STANDARD
                    .decode(k)
                    .map_err(|e| format!("PSK 解码失败: {}", e))?;
                if psk_bytes.len() != 32 {
                    return Err("PSK 必须是 32 字节".to_string());
                }
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&psk_bytes);
                Some(arr)
            }
            _ => None,
        };

        let socket = tokio::net::UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| format!("绑定 UDP socket 失败: {}", e))?;
        socket
            .connect(&self.peer_endpoint)
            .await
            .map_err(|e| format!("连接对端失败: {}", e))?;

        let tunn = Tunn::new(static_private, peer_public, psk, Some(25), 1, None);
        Ok((tunn, socket))
    }

    pub fn tun_config(&self) -> (String, String, u16) {
        let tun_addr = self.address.as_deref().unwrap_or("10.0.0.2/32");
        let tun_ip = tun_addr.split('/').next().unwrap_or("10.0.0.2");
        let tun_mtu = self.mtu.unwrap_or(1420) as u16;
        (tun_ip.to_string(), "255.255.255.0".to_string(), tun_mtu)
    }
}

/// Live tunnel status — 主进程内存共享
#[derive(Debug, Clone, Serialize)]
pub struct TunnelStatus {
    pub connected: bool,
    #[serde(rename = "bytesSent")]
    pub bytes_sent: u64,
    #[serde(rename = "bytesReceived")]
    pub bytes_received: u64,
    #[serde(rename = "latestHandshake")]
    pub latest_handshake: Option<String>,
    #[serde(rename = "connectedSince")]
    pub connected_since: Option<String>,
    pub log: Vec<String>,
}

impl TunnelStatus {
    pub fn new() -> Self {
        Self {
            connected: false,
            bytes_sent: 0,
            bytes_received: 0,
            latest_handshake: None,
            connected_since: None,
            log: Vec::new(),
        }
    }
}

impl Default for TunnelStatus {
    fn default() -> Self {
        Self::new()
    }
}

const TUN_MTU: usize = 1500;
const WG_BUF: usize = 1580;
const MAX_LOG_LINES: usize = 500;

// ────────────────────────────────────────────────────────────────────────────
// SCM_RIGHTS：跨进程传递 TUN fd（辅助进程 root 创建 → 主进程）
// ────────────────────────────────────────────────────────────────────────────

/// 通过已连接的 unix socket 把 fd 传给对方（SCM_RIGHTS，一条空数据 + 附属 fd）
pub fn send_fd(sock: RawFd, fd_to_send: RawFd) -> Result<(), String> {
    unsafe {
        let mut iov = libc::iovec {
            iov_base: std::ptr::null_mut(),
            iov_len: 0,
        };
        let mut msg: libc::msghdr = std::mem::zeroed();
        msg.msg_iov = &mut iov;
        msg.msg_iovlen = 1;

        let cmsg_space = libc::CMSG_SPACE(std::mem::size_of::<libc::c_int>() as u32) as usize;
        let mut cmsg_buf = vec![0u8; cmsg_space];
        msg.msg_control = cmsg_buf.as_mut_ptr() as *mut libc::c_void;
        msg.msg_controllen = cmsg_space as _;

        let cmsg = libc::CMSG_FIRSTHDR(&msg);
        (*cmsg).cmsg_level = libc::SOL_SOCKET;
        (*cmsg).cmsg_type = libc::SCM_RIGHTS;
        (*cmsg).cmsg_len = libc::CMSG_LEN(std::mem::size_of::<libc::c_int>() as u32) as _;
        std::ptr::copy_nonoverlapping(
            &fd_to_send as *const libc::c_int as *const u8,
            libc::CMSG_DATA(cmsg),
            std::mem::size_of::<libc::c_int>(),
        );

        if libc::sendmsg(sock, &msg, 0) < 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
    }
    Ok(())
}

/// 从已连接的 unix socket 接收一个 fd（SCM_RIGHTS）
pub fn recv_fd(sock: RawFd) -> Result<RawFd, String> {
    unsafe {
        let mut iov = libc::iovec {
            iov_base: std::ptr::null_mut(),
            iov_len: 0,
        };
        let mut msg: libc::msghdr = std::mem::zeroed();
        msg.msg_iov = &mut iov;
        msg.msg_iovlen = 1;

        let cmsg_space = libc::CMSG_SPACE(std::mem::size_of::<libc::c_int>() as u32) as usize;
        let mut cmsg_buf = vec![0u8; cmsg_space];
        msg.msg_control = cmsg_buf.as_mut_ptr() as *mut libc::c_void;
        msg.msg_controllen = cmsg_space as _;

        let n = libc::recvmsg(sock, &mut msg, 0);
        if n < 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        let cmsg = libc::CMSG_FIRSTHDR(&msg);
        if cmsg.is_null() || (*cmsg).cmsg_level != libc::SOL_SOCKET || (*cmsg).cmsg_type != libc::SCM_RIGHTS
        {
            return Err("未收到 fd（SCM_RIGHTS）".to_string());
        }
        let mut fd: libc::c_int = -1;
        std::ptr::copy_nonoverlapping(
            libc::CMSG_DATA(cmsg),
            &mut fd as *mut libc::c_int as *mut u8,
            std::mem::size_of::<libc::c_int>(),
        );
        Ok(fd)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 辅助进程入口：创建 TUN（root）→ 传 fd → 退出
// ────────────────────────────────────────────────────────────────────────────

/// `wg-tunnel` 辅助进程（由 sudo/osascript 提权启动，一次性）：
/// 读取配置 → 创建 TUN 设备（需要 root）→ 连接主进程的 unix socket →
/// 通过 SCM_RIGHTS 把 TUN fd 传给主进程 → 退出。
pub async fn run_tunnel(conf_path: &str, socket_path: &str) -> Result<(), String> {
    eprintln!("[wg-tunnel] 辅助进程: conf={} socket={}", conf_path, socket_path);

    let conf = TunnelConfig::load(conf_path)?;
    let (tun_ip, netmask, tun_mtu) = conf.tun_config();
    let mut tun_config = Configuration::default();
    tun_config.address(&tun_ip).netmask(&netmask).mtu(tun_mtu).up();
    let tun_device = tun2::create_as_async(&tun_config)
        .map_err(|e| format!("创建 TUN 设备失败: {}", e))?;

    let raw_fd = tun_device.as_raw_fd();
    // 需要释放 AsyncDevice 的 fd 所有权以便传走（避免关闭）。用 into_raw_fd 语义：
    // AsyncDevice 无 into_raw_fd，这里用 ManuallyDrop 避免 drop 时关闭 fd。
    let fd_to_send = std::mem::ManuallyDrop::new(tun_device);
    let _ = &fd_to_send;

    // 连接主进程 socket
    let stream = tokio::net::UnixStream::connect(socket_path)
        .await
        .map_err(|e| format!("连接主进程 socket 失败: {}", e))?;
    send_fd(stream.as_raw_fd(), raw_fd).map_err(|e| format!("传递 TUN fd 失败: {}", e))?;

    eprintln!("[wg-tunnel] TUN fd 已传递，辅助进程退出");
    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// 主进程入口：在自身进程内异步运行隧道转发
// ────────────────────────────────────────────────────────────────────────────

/// 主进程内运行隧道：接收辅助进程创建的 TUN fd，构造 AsyncDevice，
/// 在 tokio task 中运行 boringtun 转发循环。状态通过内存 Arc 共享。
pub async fn run_tunnel_in_process(
    conf_path: &str,
    tun_fd: RawFd,
    status: Arc<Mutex<TunnelStatus>>,
) -> Result<(), String> {
    let conf = TunnelConfig::load(conf_path)?;
    let (tun_ip, netmask, tun_mtu) = conf.tun_config();
    let mut tun_config = Configuration::default();
    tun_config
        .address(&tun_ip)
        .netmask(&netmask)
        .mtu(tun_mtu)
        .up();
    // 用辅助进程传回的 fd 构造设备（不重新打开 /dev/tun，无需 root）
    tun_config.raw_fd(tun_fd);

    let tun_device = tun2::create_as_async(&tun_config)
        .map_err(|e| format!("从 fd 构造 TUN 设备失败: {}", e))?;

    let (mut tunn, socket) = conf.build_tunnel().await?;

    // 初始握手
    let mut hs_buf = [0u8; 148];
    if let TunnResult::WriteToNetwork(packet) = tunn.format_handshake_initiation(&mut hs_buf, false)
    {
        let _ = socket.send(packet).await;
    }

    {
        let mut s = status.lock().unwrap();
        s.connected = true;
        s.connected_since = Some(chrono::Utc::now().to_rfc3339());
        s.log.push(format!(
            "正在连接 WireGuard: {} -> {}",
            conf.config_name, conf.peer_endpoint
        ));
    }

    forwarding_loop(tunn, tun_device, socket, status).await;
    Ok(())
}

async fn forwarding_loop(
    mut tunn: Tunn,
    mut tun_device: tun2::AsyncDevice,
    socket: tokio::net::UdpSocket,
    status: Arc<Mutex<TunnelStatus>>,
) {
    let mut tun_read_buf = [0u8; TUN_MTU];
    let mut wg_encap_buf = [0u8; WG_BUF];
    let mut udp_recv_buf = [0u8; WG_BUF];

    let mut keepalive_timer = tokio::time::interval(tokio::time::Duration::from_secs(25));
    keepalive_timer.tick().await; // skip immediate first tick

    let mut bytes_sent: u64 = 0;
    let mut bytes_received: u64 = 0;

    let add_log = |status: &Arc<Mutex<TunnelStatus>>, msg: String| {
        if let Ok(mut s) = status.lock() {
            s.log.push(msg);
            if s.log.len() > MAX_LOG_LINES {
                let drain_to = s.log.len() - MAX_LOG_LINES;
                s.log.drain(0..drain_to);
            }
        }
    };

    loop {
        tokio::select! {
            read_result = tun_device.read(&mut tun_read_buf) => {
                match read_result {
                    Ok(n) if n >= 20 => {
                        let ip_packet = &tun_read_buf[..n];
                        match tunn.encapsulate(ip_packet, &mut wg_encap_buf) {
                            TunnResult::WriteToNetwork(encrypted) => {
                                if let Err(e) = socket.send(encrypted).await {
                                    add_log(&status, format!("发送加密包失败: {}", e));
                                } else {
                                    bytes_sent += encrypted.len() as u64;
                                }
                            }
                            TunnResult::Err(e) => add_log(&status, format!("encapsulate 错误: {:?}", e)),
                            _ => {}
                        }
                    }
                    Err(e) => {
                        add_log(&status, format!("TUN 读取错误, 退出: {}", e));
                        break;
                    }
                    _ => {}
                }
            }

            recv_result = socket.recv_from(&mut udp_recv_buf) => {
                match recv_result {
                    Ok((n, _)) if n > 0 => {
                        let datagram = &udp_recv_buf[..n];
                        bytes_received += n as u64;
                        match tunn.decapsulate(None, datagram, &mut tun_read_buf) {
                            TunnResult::WriteToNetwork(response_pkt) => {
                                if let Err(e) = socket.send(response_pkt).await {
                                    add_log(&status, format!("发送解密响应失败: {}", e));
                                } else {
                                    bytes_sent += response_pkt.len() as u64;
                                }
                                drain_decapsulate(&mut tunn, &socket, &mut tun_device, &mut tun_read_buf, &status, &mut bytes_sent).await;
                            }
                            TunnResult::WriteToTunnelV4(pkt, _) => {
                                let _ = tun_device.write(pkt).await;
                            }
                            TunnResult::WriteToTunnelV6(pkt, _) => {
                                let _ = tun_device.write(pkt).await;
                            }
                            TunnResult::Err(e) => add_log(&status, format!("decapsulate 错误: {:?}", e)),
                            TunnResult::Done => {}
                        }
                    }
                    Err(e) => add_log(&status, format!("UDP 接收错误: {}", e)),
                    _ => {}
                }
            }

            _ = keepalive_timer.tick() => {
                let mut buf = [0u8; 148];
                if let TunnResult::WriteToNetwork(packet) = tunn.format_handshake_initiation(&mut buf, false) {
                    let _ = socket.send(packet).await;
                }
                let mut timer_buf = [0u8; 148];
                if let TunnResult::WriteToNetwork(packet) = tunn.update_timers(&mut timer_buf) {
                    let _ = socket.send(packet).await;
                }
                if let Ok(mut s) = status.lock() {
                    s.latest_handshake = Some(chrono::Utc::now().to_rfc3339());
                }
            }
        }

        if let Ok(mut s) = status.lock() {
            s.bytes_sent = bytes_sent;
            s.bytes_received = bytes_received;
        }
    }

    if let Ok(mut s) = status.lock() {
        s.bytes_sent = bytes_sent;
        s.bytes_received = bytes_received;
        s.connected = false;
    }
}

async fn drain_decapsulate(
    tunn: &mut Tunn,
    socket: &tokio::net::UdpSocket,
    tun_device: &mut tun2::AsyncDevice,
    buf: &mut [u8],
    status: &Arc<Mutex<TunnelStatus>>,
    bytes_sent: &mut u64,
) {
    loop {
        match tunn.decapsulate(None, &[], buf) {
            TunnResult::WriteToNetwork(pkt) => {
                if let Err(e) = socket.send(pkt).await {
                    if let Ok(mut s) = status.lock() {
                        s.log.push(format!("drain 发送失败: {}", e));
                    }
                } else {
                    *bytes_sent += pkt.len() as u64;
                }
            }
            TunnResult::WriteToTunnelV4(pkt, _) => {
                let _ = tun_device.write(pkt).await;
            }
            TunnResult::WriteToTunnelV6(pkt, _) => {
                let _ = tun_device.write(pkt).await;
            }
            TunnResult::Done => break,
            _ => break,
        }
    }
}
