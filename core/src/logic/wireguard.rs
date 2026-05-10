/// WireGuard 管理器 — boringtun 0.7 + tun2 TUN device for real IP packet forwarding
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use boringtun::noise::Tunn;
use boringtun::noise::TunnResult;
use boringtun::x25519::{PublicKey, StaticSecret};

use tun2::Configuration;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WireGuardStatus {
    pub connected: bool,
    #[serde(rename = "configId")]
    pub config_id: Option<String>,
    #[serde(rename = "configName")]
    pub config_name: Option<String>,
    pub state: String,
    pub log: Vec<String>,
    #[serde(rename = "connectedSince")]
    pub connected_since: Option<String>,
    #[serde(rename = "bytesSent")]
    pub bytes_sent: u64,
    #[serde(rename = "bytesReceived")]
    pub bytes_received: u64,
    #[serde(rename = "latestHandshake")]
    pub latest_handshake: Option<String>,
}

pub struct WireGuardManager {
    status: Arc<Mutex<WireGuardStatus>>,
    tunnel_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
    stop_tx: Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
    max_log_lines: usize,
}

impl WireGuardManager {
    pub fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new(WireGuardStatus {
                connected: false,
                config_id: None,
                config_name: None,
                state: "disconnected".to_string(),
                log: vec![],
                connected_since: None,
                bytes_sent: 0,
                bytes_received: 0,
                latest_handshake: None,
            })),
            tunnel_handle: Mutex::new(None),
            stop_tx: Mutex::new(None),
            max_log_lines: 500,
        }
    }

    /// Generate a new WireGuard key pair using boringtun's x25519
    pub fn generate_keypair(&self) -> Result<(String, String), String> {
        use base64::Engine;
        use rand::rngs::OsRng;
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);
        let private_b64 = base64::engine::general_purpose::STANDARD.encode(secret.as_bytes());
        let public_b64 = base64::engine::general_purpose::STANDARD.encode(public.as_bytes());
        Ok((private_b64, public_b64))
    }

    /// Derive public key from base64 private key
    pub fn public_key_from_private(private_key_b64: &str) -> Result<String, String> {
        use base64::Engine;
        let key_bytes = base64::engine::general_purpose::STANDARD
            .decode(private_key_b64)
            .map_err(|e| format!("私钥解码失败: {}", e))?;
        if key_bytes.len() != 32 {
            return Err(format!("私钥必须是 32 字节，实际 {} 字节", key_bytes.len()));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&key_bytes);
        let secret = StaticSecret::from(arr);
        let public = PublicKey::from(&secret);
        Ok(base64::engine::general_purpose::STANDARD.encode(public.as_bytes()))
    }

    pub fn get_status(&self) -> WireGuardStatus {
        self.status.lock().unwrap().clone()
    }

    fn add_log(&self, line: &str) {
        let mut status = self.status.lock().unwrap();
        status.log.push(line.to_string());
        if status.log.len() > self.max_log_lines {
            let drain_to = status.log.len() - self.max_log_lines;
            status.log.drain(0..drain_to);
        }
    }

    /// Connect to a WireGuard peer with real TUN + UDP forwarding
    pub async fn connect(
        &self,
        config_id: &str,
        config_name: &str,
        private_key_b64: &str,
        peer_public_key_b64: &str,
        peer_endpoint: &str,
        preshared_key_b64: Option<&str>,
        address: Option<&str>,
        mtu: Option<i64>,
    ) -> Result<bool, String> {
        // Disconnect if already connected
        let needs_disconnect = {
            let status = self.status.lock().unwrap();
            status.connected
        };
        if needs_disconnect {
            let _ = self.disconnect().await;
            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        }

        use base64::Engine;

        // Decode private key
        let private_bytes = base64::engine::general_purpose::STANDARD
            .decode(private_key_b64)
            .map_err(|e| format!("私钥解码失败: {}", e))?;
        if private_bytes.len() != 32 {
            return Err("私钥必须是 32 字节".to_string());
        }
        let mut sk = [0u8; 32];
        sk.copy_from_slice(&private_bytes);
        let static_private = StaticSecret::from(sk);

        // Decode peer public key
        let peer_bytes = base64::engine::general_purpose::STANDARD
            .decode(peer_public_key_b64)
            .map_err(|e| format!("对端公钥解码失败: {}", e))?;
        if peer_bytes.len() != 32 {
            return Err("对端公钥必须是 32 字节".to_string());
        }
        let mut pk = [0u8; 32];
        pk.copy_from_slice(&peer_bytes);
        let peer_public = PublicKey::from(pk);

        // Preshared key
        let psk: Option<[u8; 32]> = match preshared_key_b64 {
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

        // Update status to connecting
        {
            let mut status = self.status.lock().unwrap();
            status.connected = false;
            status.config_id = Some(config_id.to_string());
            status.config_name = Some(config_name.to_string());
            status.state = "connecting".to_string();
            status.log.clear();
            status.bytes_sent = 0;
            status.bytes_received = 0;
            status.connected_since = None;
            status.latest_handshake = None;
        }

        self.add_log(&format!(
            "正在连接 WireGuard: {} -> {}",
            config_name, peer_endpoint
        ));

        // --- Create TUN device (utun on macOS) ---
        let tun_addr = address.unwrap_or("10.0.0.2/32");
        let tun_ip = tun_addr.split('/').next().unwrap_or("10.0.0.2");
        let tun_mtu = mtu.unwrap_or(1420) as u16;
        let mut tun_config = Configuration::default();
        tun_config
            .address(tun_ip)
            .netmask("255.255.255.0")
            .mtu(tun_mtu)
            .up();

        let tun_device = tun2::create_as_async(&tun_config)
            .map_err(|e| format!("创建 TUN 设备失败: {}", e))?;

        self.add_log("TUN 设备已创建 (utun)");

        // --- Create UDP socket ---
        let socket = tokio::net::UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| format!("绑定 UDP socket 失败: {}", e))?;
        socket
            .connect(peer_endpoint)
            .await
            .map_err(|e| format!("连接对端失败: {}", e))?;

        self.add_log(&format!("UDP socket 已连接到: {}", peer_endpoint));

        // --- Create boringtun Tunn ---
        let mut tunn = Tunn::new(
            static_private,
            peer_public,
            psk,
            Some(25), // persistent keepalive every 25 seconds
            1,        // our tunnel index
            None,     // no rate limiter
        );

        self.add_log("boringtun 隧道已创建");

        // --- Send initial handshake initiation ---
        let mut handshake_buf = [0u8; 148];
        match tunn.format_handshake_initiation(&mut handshake_buf, false) {
            TunnResult::WriteToNetwork(packet) => {
                socket
                    .send(packet)
                    .await
                    .map_err(|e| format!("发送握手请求失败: {}", e))?;
                self.add_log(&format!("握手请求已发送 ({} bytes)", packet.len()));
            }
            _ => {
                self.add_log("握手请求已生成 (未发送)");
            }
        }

        // --- Setup stop signal channel ---
        let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();

        // --- Clone Arcs for the background task ---
        let status_arc = self.status.clone();
        let max_log_lines = self.max_log_lines;

        // --- Spawn background forwarding task ---
        let handle = tokio::spawn(async move {
            run_forwarding_loop(tunn, tun_device, socket, status_arc, max_log_lines, stop_rx).await;
        });

        // Store handle and stop signal for later disconnect
        {
            let mut th = self.tunnel_handle.lock().unwrap();
            *th = Some(handle);
        }
        {
            let mut st = self.stop_tx.lock().unwrap();
            *st = Some(stop_tx);
        }

        // Mark as connected
        {
            let mut status = self.status.lock().unwrap();
            status.state = "connected".to_string();
            status.connected = true;
            status.connected_since = Some(chrono::Utc::now().to_rfc3339());
            status.latest_handshake = Some(chrono::Utc::now().to_rfc3339());
        }
        self.add_log("✅ WireGuard 隧道已建立");

        Ok(true)
    }

    pub async fn disconnect(&self) -> Result<(), String> {
        // Send stop signal to the forwarding task
        if let Some(tx) = self.stop_tx.lock().unwrap().take() {
            let _ = tx.send(());
        }

        // Wait for the forwarding task to exit (with timeout)
        let handle = { self.tunnel_handle.lock().unwrap().take() };
        if let Some(handle) = handle {
            let _ = tokio::time::timeout(tokio::time::Duration::from_secs(5), handle).await;
        }

        {
            let status = self.status.lock().unwrap();
            if !status.connected && status.state == "disconnected" {
                return Ok(());
            }
        }

        self.add_log("正在断开 WireGuard...");
        {
            let mut status = self.status.lock().unwrap();
            status.state = "disconnected".to_string();
            status.connected = false;
            status.config_id = None;
            status.config_name = None;
            status.bytes_sent = 0;
            status.bytes_received = 0;
            status.log.clear();
            status.latest_handshake = None;
            status.connected_since = None;
        }
        self.add_log("✅ 已断开");
        Ok(())
    }
}

// ─── Background packet forwarding loop ───────────────────────────────────────

/// Buffer sizes:
///   TUN_MTU: 1500 bytes (standard IP MTU)
///   WG_OVERHEAD: ~80 bytes (WireGuard header + authentication tag)
///   WG_BUF: TUN_MTU + WG_OVERHEAD = 1580 bytes
const TUN_MTU: usize = 1500;
const WG_BUF: usize = 1580;

async fn run_forwarding_loop(
    mut tunn: Tunn,
    mut tun_device: tun2::AsyncDevice,
    socket: tokio::net::UdpSocket,
    status: Arc<Mutex<WireGuardStatus>>,
    max_log_lines: usize,
    mut stop_rx: tokio::sync::oneshot::Receiver<()>,
) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut tun_read_buf = [0u8; TUN_MTU];   // IP packets from TUN
    let mut wg_encap_buf = [0u8; WG_BUF];    // Encrypted WireGuard packets
    let mut udp_recv_buf = [0u8; WG_BUF];    // Received UDP datagrams

    // Keepalive: send handshake every 25 seconds
    let mut keepalive_timer = tokio::time::interval(tokio::time::Duration::from_secs(25));
    // First tick fires immediately — skip it so we don't re-handshake right away
    keepalive_timer.tick().await;

    let mut bytes_sent: u64 = 0;
    let mut bytes_received: u64 = 0;

    // Helper closure to add a log line to the shared status
    let add_log = |status: &Arc<Mutex<WireGuardStatus>>, msg: &str| {
        if let Ok(mut s) = status.lock() {
            s.log.push(msg.to_string());
            if s.log.len() > max_log_lines {
                let drain_to = s.log.len() - max_log_lines;
                s.log.drain(0..drain_to);
            }
        }
    };

    loop {
        tokio::select! {
            // ── TUN → encapsulate → UDP ──
            read_result = tun_device.read(&mut tun_read_buf) => {
                match read_result {
                    Ok(n) if n >= 20 => {
                        // Minimum IPv4 header is 20 bytes
                        let ip_packet = &tun_read_buf[..n];
                        match tunn.encapsulate(ip_packet, &mut wg_encap_buf) {
                            TunnResult::WriteToNetwork(encrypted) => {
                                if let Err(e) = socket.send(encrypted).await {
                                    add_log(&status, &format!("发送加密包失败: {}", e));
                                } else {
                                    bytes_sent += encrypted.len() as u64;
                                }
                            }
                            TunnResult::Err(e) => {
                                add_log(&status, &format!("encapsulate 错误: {:?}", e));
                            }
                            // Done: nothing to send (e.g., handshake not complete yet)
                            _ => {}
                        }
                    }
                    Ok(n) if n > 0 => {
                        add_log(&status, &format!("忽略过短的 TUN 包 ({} bytes)", n));
                    }
                    Err(e) => {
                        add_log(&status, &format!("TUN 读取错误, 退出: {}", e));
                        break;
                    }
                    _ => {}
                }
            }

            // ── UDP → decapsulate → TUN ──
            recv_result = socket.recv_from(&mut udp_recv_buf) => {
                match recv_result {
                    Ok((n, _src_addr)) if n > 0 => {
                        let datagram = &udp_recv_buf[..n];
                        bytes_received += n as u64;

                        // Process the incoming datagram.
                        // If a handshake response is generated (WriteToNetwork),
                        // we must call decapsulate again with empty data until Done.
                        match tunn.decapsulate(None, datagram, &mut tun_read_buf) {
                            TunnResult::WriteToNetwork(response_pkt) => {
                                // Send the response (e.g., handshake reply)
                                if let Err(e) = socket.send(response_pkt).await {
                                    add_log(&status, &format!("发送解密响应失败: {}", e));
                                } else {
                                    bytes_sent += response_pkt.len() as u64;
                                }
                                // Drain any further cascaded messages
                                drain_decapsulate(
                                    &mut tunn, &socket, &mut tun_device,
                                    &mut tun_read_buf, &status,
                                    &mut bytes_sent,
                                ).await;
                            }
                            TunnResult::WriteToTunnelV4(packet, _addr) => {
                                if let Err(e) = tun_device.write(packet).await {
                                    add_log(&status, &format!("TUN 写入 IPv4 错误: {}", e));
                                }
                            }
                            TunnResult::WriteToTunnelV6(packet, _addr) => {
                                if let Err(e) = tun_device.write(packet).await {
                                    add_log(&status, &format!("TUN 写入 IPv6 错误: {}", e));
                                }
                            }
                            TunnResult::Err(e) => {
                                add_log(&status, &format!("decapsulate 错误: {:?}", e));
                            }
                            TunnResult::Done => {}
                        }
                    }
                    Err(e) => {
                        // Transient UDP errors (e.g., ICMP unreachable) shouldn't kill the tunnel
                        add_log(&status, &format!("UDP 接收错误: {}", e));
                    }
                    _ => {}
                }
            }

            // ── Keepalive / rekey timer ──
            _ = keepalive_timer.tick() => {
                // Send keepalive / handshake if needed
                let mut buf = [0u8; 148];
                match tunn.format_handshake_initiation(&mut buf, false) {
                    TunnResult::WriteToNetwork(packet) => {
                        let _ = socket.send(packet).await;
                    }
                    _ => {}
                }
                // Check timers for rekeying
                let mut timer_buf = [0u8; 148];
                match tunn.update_timers(&mut timer_buf) {
                    TunnResult::WriteToNetwork(packet) => {
                        let _ = socket.send(packet).await;
                    }
                    _ => {}
                }
                // Update handshake timestamp in status
                if let Ok(mut s) = status.lock() {
                    s.latest_handshake = Some(chrono::Utc::now().to_rfc3339());
                }
            }

            // ── Stop signal ──
            _ = &mut stop_rx => {
                add_log(&status, "收到停止信号, 关闭隧道...");
                break;
            }
        }

        // Flush traffic stats periodically into shared status (every ~1s worth of events)
        // We rely on the get_status() caller for fresh reads; update after each loop
        if let Ok(mut s) = status.lock() {
            s.bytes_sent = bytes_sent;
            s.bytes_received = bytes_received;
        }
    }

    // Final stats update
    if let Ok(mut s) = status.lock() {
        s.bytes_sent = bytes_sent;
        s.bytes_received = bytes_received;
        s.state = "disconnected".to_string();
        s.connected = false;
    }
}

/// Drain cascaded decapsulate results after a WriteToNetwork is returned.
/// Per boringtun docs: when WriteToNetwork is returned, you must call
/// decapsulate again with an empty datagram until Done is returned.
async fn drain_decapsulate(
    tunn: &mut Tunn,
    socket: &tokio::net::UdpSocket,
    tun_device: &mut tun2::AsyncDevice,
    buf: &mut [u8],
    status: &Arc<Mutex<WireGuardStatus>>,
    bytes_sent: &mut u64,
) {
    use tokio::io::AsyncWriteExt;

    let add_log = |status: &Arc<Mutex<WireGuardStatus>>, msg: &str| {
        if let Ok(mut s) = status.lock() {
            s.log.push(msg.to_string());
        }
    };

    loop {
        match tunn.decapsulate(None, &[], buf) {
            TunnResult::WriteToNetwork(pkt) => {
                if let Err(e) = socket.send(pkt).await {
                    add_log(status, &format!("drain 发送失败: {}", e));
                } else {
                    *bytes_sent += pkt.len() as u64;
                }
            }
            TunnResult::WriteToTunnelV4(pkt, _addr) => {
                let _ = tun_device.write(pkt).await;
            }
            TunnResult::WriteToTunnelV6(pkt, _addr) => {
                let _ = tun_device.write(pkt).await;
            }
            TunnResult::Done => break,
            TunnResult::Err(e) => {
                add_log(status, &format!("drain decapsulate 错误: {:?}", e));
                break;
            }
        }
    }
}
