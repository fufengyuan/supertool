/// WireGuard tunnel subprocess — runs as root, owns the TUN device,
/// runs boringtun forwarding loop, and exposes a UDS control socket.
///
/// Designed to be spawned via `osascript` from the unprivileged Tauri main process:
///   `sudo /path/to/stool wg-tunnel --conf /tmp/wg.json --uds /tmp/wg.sock`
///
/// The parent process talks to this subprocess via UDS JSON-lines protocol:
///   → {"cmd":"status"}     ← {"ok":true,"status":{...}}
///   → {"cmd":"stop"}        ← {"ok":true}
use boringtun::noise::{Tunn, TunnResult};
use boringtun::x25519::{PublicKey, StaticSecret};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
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

/// Live tunnel status — exposed via UDS to the parent process
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
    fn new() -> Self {
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

const TUN_MTU: usize = 1500;
const WG_BUF: usize = 1580;
const MAX_LOG_LINES: usize = 500;

/// Main entry point for the `stool wg-tunnel` subcommand.
/// Reads conf from JSON file, sets up TUN + UDP + boringtun, listens on UDS.
/// Returns once the tunnel exits (either via UDS stop command or fatal error).
pub async fn run_tunnel(conf_path: &str, uds_path: &str) -> Result<(), String> {
    // Write startup marker so the log is easy to correlate
    eprintln!("[wg-tunnel] starting: conf={} uds={}", conf_path, uds_path);

    // 1. Load conf
    let conf_text = std::fs::read_to_string(conf_path)
        .map_err(|e| { let msg = format!("读取配置文件失败: {}", e); eprintln!("[wg-tunnel] {}", &msg); msg })?;
    let conf: TunnelConfig =
        serde_json::from_str(&conf_text).map_err(|e| format!("解析配置失败: {}", e))?;

    // 2. Decode keys
    use base64::Engine;
    let private_bytes = base64::engine::general_purpose::STANDARD
        .decode(&conf.private_key)
        .map_err(|e| format!("私钥解码失败: {}", e))?;
    if private_bytes.len() != 32 {
        return Err("私钥必须是 32 字节".to_string());
    }
    let mut sk = [0u8; 32];
    sk.copy_from_slice(&private_bytes);
    let static_private = StaticSecret::from(sk);

    let peer_bytes = base64::engine::general_purpose::STANDARD
        .decode(&conf.peer_public_key)
        .map_err(|e| format!("对端公钥解码失败: {}", e))?;
    if peer_bytes.len() != 32 {
        return Err("对端公钥必须是 32 字节".to_string());
    }
    let mut pk = [0u8; 32];
    pk.copy_from_slice(&peer_bytes);
    let peer_public = PublicKey::from(pk);

    let psk: Option<[u8; 32]> = match conf.preshared_key.as_deref() {
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

    // 3. Create TUN device (requires root)
    let tun_addr = conf.address.as_deref().unwrap_or("10.0.0.2/32");
    let tun_ip = tun_addr.split('/').next().unwrap_or("10.0.0.2");
    let tun_mtu = conf.mtu.unwrap_or(1420) as u16;
    let mut tun_config = Configuration::default();
    tun_config
        .address(tun_ip)
        .netmask("255.255.255.0")
        .mtu(tun_mtu)
        .up();

    let tun_device = tun2::create_as_async(&tun_config)
        .map_err(|e| format!("创建 TUN 设备失败: {}", e))?;

    // 4. Create UDP socket and connect to peer
    let socket = tokio::net::UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(|e| format!("绑定 UDP socket 失败: {}", e))?;
    socket
        .connect(&conf.peer_endpoint)
        .await
        .map_err(|e| format!("连接对端失败: {}", e))?;

    // 5. Build boringtun Tunn
    let mut tunn = Tunn::new(static_private, peer_public, psk, Some(25), 1, None);

    // 6. Send initial handshake
    let mut hs_buf = [0u8; 148];
    if let TunnResult::WriteToNetwork(packet) = tunn.format_handshake_initiation(&mut hs_buf, false)
    {
        let _ = socket.send(packet).await;
    }

    // 7. Shared status + stop signal
    let status = Arc::new(Mutex::new(TunnelStatus::new()));
    {
        let mut s = status.lock().unwrap();
        s.connected = true;
        s.connected_since = Some(chrono::Utc::now().to_rfc3339());
        s.log.push(format!(
            "正在连接 WireGuard: {} -> {}",
            conf.config_name, conf.peer_endpoint
        ));
    }
    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();

    // 8. Set up UDS listener BEFORE forwarding (parent waits for it)
    // Remove any stale socket file from previous run
    let _ = std::fs::remove_file(uds_path);
    let listener = UnixListener::bind(uds_path).map_err(|e| format!("UDS bind 失败: {}", e))?;
    // Make socket writable by owner only (default is rwx for owner). Fine since
    // both parent (user) and this child (sudo'd from same user) can read it.
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(uds_path, std::fs::Permissions::from_mode(0o600));

    // 9. Spawn forwarding loop and UDS server concurrently
    let status_for_fwd = status.clone();
    let fwd_handle = tokio::spawn(async move {
        forwarding_loop(tunn, tun_device, socket, status_for_fwd, stop_rx).await;
    });

    let status_for_uds = status.clone();
    let stop_tx_arc = Arc::new(tokio::sync::Mutex::new(Some(stop_tx)));
    let uds_handle = tokio::spawn(async move {
        run_uds_server(listener, status_for_uds, stop_tx_arc).await;
    });

    // Wait for forwarding loop to exit (triggered by UDS stop or fatal error)
    let _ = fwd_handle.await;
    uds_handle.abort();

    // Clean up UDS socket file
    let _ = std::fs::remove_file(uds_path);

    Ok(())
}

#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
enum UdsCommand {
    Status,
    Stop,
}

#[derive(Serialize)]
struct UdsResponse {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<TunnelStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

async fn run_uds_server(
    listener: UnixListener,
    status: Arc<Mutex<TunnelStatus>>,
    stop_tx: Arc<tokio::sync::Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
) {
    loop {
        let (stream, _) = match listener.accept().await {
            Ok(x) => x,
            Err(_) => continue,
        };
        let status = status.clone();
        let stop_tx = stop_tx.clone();
        tokio::spawn(async move {
            let (read_half, mut write_half) = stream.into_split();
            let mut reader = BufReader::new(read_half);
            let mut line = String::new();
            while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
                let trimmed = line.trim().to_string();
                line.clear();
                let resp = match serde_json::from_str::<UdsCommand>(&trimmed) {
                    Ok(UdsCommand::Status) => UdsResponse {
                        ok: true,
                        status: Some(status.lock().unwrap().clone()),
                        error: None,
                    },
                    Ok(UdsCommand::Stop) => {
                        if let Some(tx) = stop_tx.lock().await.take() {
                            let _ = tx.send(());
                        }
                        UdsResponse { ok: true, status: None, error: None }
                    }
                    Err(e) => UdsResponse {
                        ok: false,
                        status: None,
                        error: Some(format!("invalid command: {}", e)),
                    },
                };
                let resp_json = serde_json::to_string(&resp).unwrap_or_default();
                let _ = write_half.write_all(resp_json.as_bytes()).await;
                let _ = write_half.write_all(b"\n").await;
            }
        });
    }
}

async fn forwarding_loop(
    mut tunn: Tunn,
    mut tun_device: tun2::AsyncDevice,
    socket: tokio::net::UdpSocket,
    status: Arc<Mutex<TunnelStatus>>,
    mut stop_rx: tokio::sync::oneshot::Receiver<()>,
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

            _ = &mut stop_rx => {
                add_log(&status, "收到停止信号, 关闭隧道...".to_string());
                break;
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
