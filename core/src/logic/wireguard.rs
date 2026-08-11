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
    /// PID of the elevated `stool wg-tunnel` subprocess (when using subprocess mode)
    subprocess_pid: Mutex<Option<u32>>,
    /// UDS path for talking to the subprocess
    subprocess_status_path: Mutex<Option<String>>,
    /// Conf file path (cleaned up after subprocess starts)
    subprocess_conf: Mutex<Option<String>>,
    /// Background poller handle that mirrors subprocess status into self.status
    poller_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
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
            subprocess_pid: Mutex::new(None),
            subprocess_status_path: Mutex::new(None),
            subprocess_conf: Mutex::new(None),
            poller_handle: Mutex::new(None),
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

    /// Connect to a WireGuard peer by spawning an elevated `stool wg-tunnel`
    /// subprocess that owns the TUN device and runs boringtun. The GUI process
    /// stays unprivileged and talks to the subprocess over UDS.
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
            status.connected || status.state == "connecting"
        };
        if needs_disconnect {
            let _ = self.disconnect().await;
            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        }

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

        // Find the tunnel host binary: 直接使用当前可执行文件（tauri 二进制自带 wg-tunnel 模式），
        // 不再依赖外部 stool 二进制与路径探测（同属 supertool-core，逻辑单一来源）。
        let tunnel_exe = tunnel_binary_path()?;
        self.add_log(&format!("使用自身隧道模式启动 (wg-tunnel, exe: {})", tunnel_exe));

        // Write tunnel config to temp file
        let ts = chrono::Utc::now().timestamp_millis();
        let conf_path = format!("/tmp/supertool-wg-{}.json", ts);
        let status_path = format!("/tmp/supertool-wg-{}.status.json", ts);

        let conf_json = serde_json::json!({
            "configId": config_id,
            "configName": config_name,
            "privateKey": private_key_b64,
            "peerPublicKey": peer_public_key_b64,
            "peerEndpoint": peer_endpoint,
            "presharedKey": preshared_key_b64,
            "address": address,
            "mtu": mtu,
        });
        std::fs::write(&conf_path, conf_json.to_string())
            .map_err(|e| format!("写入配置文件失败: {}", e))?;

        // On macOS, use osascript to elevate privileges. This pops a native
        // system password dialog ("SuperTool wants to make changes").
        // 免密已配置时走 sudo -n，无需弹密码框（避免误导用户以为要输密码）
        #[cfg(target_os = "macos")]
        if is_passwordless_installed() {
            self.add_log("使用已配置的免密授权启动隧道...");
        } else {
            self.add_log("请求 macOS 系统授权（请在密码框中输入密码）...");
        }
        #[cfg(not(target_os = "macos"))]
        self.add_log("请求 macOS 系统授权（请在密码框中输入密码）...");
        let pid = spawn_tunnel_subprocess(&conf_path, &status_path).await?;
        self.add_log(&format!("tunnel 子进程已启动 (PID: {})", pid));

        // Store subprocess metadata
        {
            *self.subprocess_pid.lock().unwrap() = Some(pid);
            *self.subprocess_status_path.lock().unwrap() = Some(status_path.clone());
            *self.subprocess_conf.lock().unwrap() = Some(conf_path.clone());
        }

        // Wait for the status file to appear (up to 30s)
        let mut status_ready = false;
        for _ in 0..150 {
            if std::path::Path::new(&status_path).exists() {
                status_ready = true;
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }
        if !status_ready {
            // Try to read the subprocess log to give a useful error
            let wg_log_path = "/tmp/supertool-wg.log";
            let log_tail = std::fs::read_to_string(wg_log_path)
                .unwrap_or_default()
                .lines()
                .rev()
                .take(10)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n");
            let detail = if log_tail.trim().is_empty() {
                String::new()
            } else {
                format!("\n子进程日志:\n{}", log_tail)
            };
            return Err(format!("tunnel 子进程启动超时（30s 未就绪）{}", detail));
        }
        self.add_log("隧道进程已就绪（状态文件就位）");

        // Mark as connected
        {
            let mut status = self.status.lock().unwrap();
            status.state = "connected".to_string();
            status.connected = true;
            status.connected_since = Some(chrono::Utc::now().to_rfc3339());
        }
        self.add_log("✅ WireGuard 隧道已建立");

        // Spawn poller that mirrors subprocess status into self.status every 2s
        let status_arc = self.status.clone();
        let status_path_for_poll = status_path.clone();
        let poller = tokio::spawn(async move {
            poll_subprocess_status(status_path_for_poll, status_arc).await;
        });
        *self.poller_handle.lock().unwrap() = Some(poller);

        // Clean up the conf file (the subprocess has already loaded it)
        let _ = std::fs::remove_file(&conf_path);
        *self.subprocess_conf.lock().unwrap() = None;

        Ok(true)
    }

    /// Reset state from "connecting" back to "disconnected" if connection failed mid-way
    pub fn reset_state_if_connecting(&self) {
        let mut status = self.status.lock().unwrap();
        if status.state == "connecting" {
            status.state = "disconnected".to_string();
            status.connected = false;
        }
    }

    pub async fn disconnect(&self) -> Result<(), String> {
        // Stop the status poller first
        if let Some(handle) = self.poller_handle.lock().unwrap().take() {
            handle.abort();
        }

        // Stop the tunnel subprocess via SIGTERM（sudo kill 提权——子进程以 root 运行）
        let status_path = self.subprocess_status_path.lock().unwrap().clone();
        if let Some(pid) = self.subprocess_pid.lock().unwrap().take() {
            // sudo -n kill：若免密已配置直接生效；否则回退 kill（同 uid 时可用）
            let killed = std::process::Command::new("sudo")
                .arg("-n")
                .arg("kill")
                .arg(pid.to_string())
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if !killed {
                let _ = std::process::Command::new("kill")
                    .arg(pid.to_string())
                    .output();
            }
        }
        if let Some(path) = status_path {
            // 等状态文件消失（子进程退出会清理），最多 3s
            for _ in 0..30 {
                if !std::path::Path::new(&path).exists() {
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            let _ = std::fs::remove_file(&path);
        }

        *self.subprocess_status_path.lock().unwrap() = None;

        // Clean up legacy in-process handles (in case we ever fall back)
        if let Some(tx) = self.stop_tx.lock().unwrap().take() {
            let _ = tx.send(());
        }
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

/// Parsed .conf result for the import flow
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedWgConfig {
    pub name: String,
    pub private_key: String,
    pub public_key: String,
    pub address: String,
    pub dns: Option<String>,
    pub mtu: Option<i64>,
    pub peer_public_key: String,
    pub peer_endpoint: String,
    pub peer_allowed_ips: String,
    pub peer_persistent_keepalive: Option<i64>,
    pub preshared_key: Option<String>,
}

impl WireGuardManager {
    /// Parse a WireGuard .conf file content into a ParsedWgConfig.
    /// Returns the first [Peer] section only.
    pub fn parse_conf(content: &str, name: &str) -> Result<ParsedWgConfig, String> {
        let mut interface = std::collections::HashMap::new();
        let mut peers: Vec<std::collections::HashMap<String, String>> = Vec::new();
        let mut current_section: Option<String> = None;

        for line in content.lines() {
            let trimmed = line.trim();
            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
                continue;
            }
            // Section headers: [Interface] or [Peer]
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let section_name = trimmed[1..trimmed.len()-1].trim().to_lowercase();
                current_section = Some(section_name);
                if current_section.as_deref() == Some("peer") {
                    peers.push(std::collections::HashMap::new());
                }
                continue;
            }
            // Key = Value
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim().to_lowercase();
                let value = trimmed[eq_pos + 1..].trim().to_string();
                match current_section.as_deref() {
                    Some("interface") => { interface.insert(key, value); }
                    Some("peer") => {
                        if let Some(peer) = peers.last_mut() {
                            peer.insert(key, value);
                        }
                    }
                    _ => {}
                }
            }
        }

        // Validate required interface fields
        let private_key = interface.get("privatekey")
            .ok_or_else(|| "配置缺少 [Interface] 中的 PrivateKey".to_string())?;
        let address = interface.get("address")
            .ok_or_else(|| "配置缺少 [Interface] 中的 Address".to_string())?;
        let dns = interface.get("dns").cloned();
        let mtu = interface.get("mtu")
            .and_then(|v| v.parse::<i64>().ok());

        // Derive public key from private key
        let public_key = Self::public_key_from_private(private_key)?;

        // Get first peer
        let peer = peers.first()
            .ok_or_else(|| "配置缺少 [Peer] 段".to_string())?;
        if peers.len() > 1 {
            log::warn!("[WireGuard] .conf contains {} peers, only the first will be used", peers.len());
        }
        let peer_public_key = peer.get("publickey")
            .ok_or_else(|| "配置缺少 [Peer] 中的 PublicKey".to_string())?;
        let peer_endpoint = peer.get("endpoint")
            .ok_or_else(|| "配置缺少 [Peer] 中的 Endpoint".to_string())?;
        let peer_allowed_ips = peer.get("allowedips")
            .ok_or_else(|| "配置缺少 [Peer] 中的 AllowedIPs".to_string())?;
        let peer_persistent_keepalive = peer.get("persistentkeepalive")
            .and_then(|v| v.parse::<i64>().ok());
        let preshared_key = peer.get("presharedkey").cloned();

        Ok(ParsedWgConfig {
            name: name.to_string(),
            private_key: private_key.clone(),
            public_key,
            address: address.clone(),
            dns,
            mtu,
            peer_public_key: peer_public_key.clone(),
            peer_endpoint: peer_endpoint.clone(),
            peer_allowed_ips: peer_allowed_ips.clone(),
            peer_persistent_keepalive,
            preshared_key,
        })
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

    let mut tun_read_buf = [0u8; TUN_MTU]; // IP packets from TUN
    let mut wg_encap_buf = [0u8; WG_BUF]; // Encrypted WireGuard packets
    let mut udp_recv_buf = [0u8; WG_BUF]; // Received UDP datagrams

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

// ─── Subprocess (privileged tunnel) helpers ──────────────────────────────────

const SUDOERS_PATH: &str = "/etc/sudoers.d/supertool-wg";

/// Get current logged-in username (USER env or `whoami`)
fn current_username() -> Result<String, String> {
    if let Ok(u) = std::env::var("USER") {
        if !u.is_empty() {
            return Ok(u);
        }
    }
    let out = std::process::Command::new("whoami")
        .output()
        .map_err(|e| format!("whoami 失败: {}", e))?;
    let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if name.is_empty() {
        return Err("无法确定当前用户名".to_string());
    }
    Ok(name)
}

/// Check whether passwordless sudo is installed for the current user + stool binary.
pub fn is_passwordless_installed() -> bool {
    std::path::Path::new(SUDOERS_PATH).exists()
}

/// Install passwordless sudoers rule. Pops up the macOS auth dialog ONCE
/// (osascript) to write a sudoers file allowing the current user to invoke
/// `<stool> wg-tunnel ...` without a password. After this, all WireGuard
/// connections work with no further prompts.
#[cfg(target_os = "macos")]
pub async fn install_passwordless() -> Result<(), String> {
    let user = current_username()?;
    let stool = tunnel_binary_path()?;
    // Resolve to canonical path so the sudoers wildcard match is precise.
    let stool_canonical = std::fs::canonicalize(&stool)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or(stool);

    let rule = format!(
        "# SuperTool WireGuard tunnel — auto-generated, safe to remove\n\
         {} ALL=(root) NOPASSWD: {} wg-tunnel --conf /tmp/supertool-wg-* --status /tmp/supertool-wg-*\n",
        user, stool_canonical
    );

    // Write to a temp file first so we don't end up with a half-written sudoers.
    let ts = chrono::Utc::now().timestamp_millis();
    let tmp = format!("/tmp/supertool-sudoers-{}", ts);
    std::fs::write(&tmp, &rule).map_err(|e| format!("写入临时 sudoers 失败: {}", e))?;

    // Validate the rule with visudo before installing — visudo -c -f <tmp>
    let visudo_check = std::process::Command::new("visudo")
        .args(["-c", "-f", &tmp])
        .output();
    if let Ok(out) = visudo_check {
        if !out.status.success() {
            let _ = std::fs::remove_file(&tmp);
            return Err(format!(
                "sudoers 规则校验失败: {}",
                String::from_utf8_lossy(&out.stderr)
            ));
        }
    }

    // Move into place + chown root:wheel + chmod 0440 (sudoers requirement)
    let q = |s: &str| s.replace('\'', "'\\''");
    let inner = format!(
        "install -o root -g wheel -m 0440 '{}' '{}' && rm -f '{}'",
        q(&tmp),
        SUDOERS_PATH,
        q(&tmp),
    );
    let escaped = inner.replace('\\', "\\\\").replace('"', "\\\"");
    let osa = format!(
        r#"do shell script "{}" with administrator privileges with prompt "SuperTool 需要管理员权限以启用 WireGuard 免密连接""#,
        escaped
    );

    let output = tokio::process::Command::new("osascript")
        .arg("-e")
        .arg(&osa)
        .output()
        .await
        .map_err(|e| format!("启动 osascript 失败: {}", e))?;

    let _ = std::fs::remove_file(&tmp);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("-128") || stderr.contains("canceled") {
            return Err("用户取消了授权".to_string());
        }
        return Err(format!("安装 sudoers 失败: {}", stderr.trim()));
    }
    Ok(())
}

/// Uninstall the passwordless sudoers rule (pops up the auth dialog once).
#[cfg(target_os = "macos")]
pub async fn uninstall_passwordless() -> Result<(), String> {
    if !is_passwordless_installed() {
        return Ok(());
    }
    let q = |s: &str| s.replace('\'', "'\\''");
    let inner = format!("rm -f '{}'", q(SUDOERS_PATH));
    let escaped = inner.replace('\\', "\\\\").replace('"', "\\\"");
    let osa = format!(
        r#"do shell script "{}" with administrator privileges with prompt "SuperTool 需要管理员权限以禁用 WireGuard 免密连接""#,
        escaped
    );

    let output = tokio::process::Command::new("osascript")
        .arg("-e")
        .arg(&osa)
        .output()
        .await
        .map_err(|e| format!("启动 osascript 失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("-128") || stderr.contains("canceled") {
            return Err("用户取消了授权".to_string());
        }
        return Err(format!("移除 sudoers 失败: {}", stderr.trim()));
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub async fn install_passwordless() -> Result<(), String> {
    Err("当前平台暂不支持免密配置".to_string())
}

#[cfg(not(target_os = "macos"))]
pub async fn uninstall_passwordless() -> Result<(), String> {
    Err("当前平台暂不支持免密配置".to_string())
}

/// Locate the tunnel host binary: 当前可执行文件自带 wg-tunnel 模式（tauri 主进程），
/// 无外部 stool 依赖。dev 模式下 cargo run 的 target 目录亦可直接使用。
fn tunnel_binary_path() -> Result<String, String> {
    if let Ok(current_exe) = std::env::current_exe() {
        let p = current_exe.to_string_lossy().to_string();
        if !p.is_empty() {
            return Ok(p);
        }
    }
    Err("无法定位当前可执行文件".to_string())
}

/// Spawn `sudo <current_exe> wg-tunnel` via macOS `osascript` (native password dialog).
/// Returns the spawned PID. The subprocess is detached and lives until stopped.
async fn spawn_tunnel_subprocess(
    conf_path: &str,
    status_path: &str,
) -> Result<u32, String> {
    #[cfg(target_os = "macos")]
    {
        let q = |s: &str| s.replace('\'', "'\\''");
        let tunnel_exe = tunnel_binary_path()?;

        // ── Fast path: passwordless sudo is configured ──
        // Use `sudo -n` directly. No password prompt, no osascript dialog.
        if is_passwordless_installed() {
            let inner = format!(
                "sudo -n '{}' wg-tunnel --conf '{}' --status '{}' </dev/null >/tmp/supertool-wg.log 2>&1 & echo $!",
                q(&tunnel_exe),
                q(conf_path),
                q(status_path),
            );
            let output = tokio::process::Command::new("sh")
                .args(["-c", &inner])
                .output()
                .await
                .map_err(|e| format!("启动 sudo 失败: {}", e))?;

            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Ok(pid) = stdout.trim().parse::<u32>() {
                    return Ok(pid);
                }
            }
            // Fall through to osascript path if -n failed (sudoers got removed)
        }

        // ── Slow path: pop up the macOS auth dialog ──
        let inner = format!(
            "'{}' wg-tunnel --conf '{}' --status '{}' </dev/null >/tmp/supertool-wg.log 2>&1 & echo $!",
            q(&tunnel_exe),
            q(conf_path),
            q(status_path),
        );
        let escaped = inner.replace('\\', "\\\\").replace('"', "\\\"");
        let osa = format!(
            r#"do shell script "{}" with administrator privileges with prompt "SuperTool 需要管理员权限以启动 WireGuard 隧道""#,
            escaped
        );

        let output = tokio::process::Command::new("osascript")
            .arg("-e")
            .arg(&osa)
            .output()
            .await
            .map_err(|e| format!("启动 osascript 失败: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("-128") || stderr.contains("canceled") {
                return Err("用户取消了授权".to_string());
            }
            return Err(format!("osascript 失败: {}", stderr.trim()));
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let pid: u32 = stdout
            .trim()
            .parse()
            .map_err(|_| format!("无法解析子进程 PID: {}", stdout))?;
        Ok(pid)
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err("当前平台暂不支持特权 WireGuard 隧道（仅 macOS）".to_string())
    }
}

/// Send a single JSON command to the tunnel subprocess via UDS and read the response.
/// Periodically read the tunnel status file and mirror it into self.status.
async fn poll_subprocess_status(status_path: String, status: Arc<Mutex<WireGuardStatus>>) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(2));
    loop {
        interval.tick().await;
        let content = std::fs::read_to_string(&status_path);
        match content {
            Ok(json) => {
                let Ok(sub_status) = serde_json::from_str::<serde_json::Value>(&json) else {
                    continue;
                };
                if let Ok(mut s) = status.lock() {
                    if let Some(bs) = sub_status.get("bytesSent").and_then(|v| v.as_u64()) {
                        s.bytes_sent = bs;
                    }
                    if let Some(br) = sub_status.get("bytesReceived").and_then(|v| v.as_u64()) {
                        s.bytes_received = br;
                    }
                    if let Some(lh) = sub_status.get("latestHandshake").and_then(|v| v.as_str()) {
                        s.latest_handshake = Some(lh.to_string());
                    }
                    if let Some(log) = sub_status.get("log").and_then(|v| v.as_array()) {
                        let lines: Vec<String> = log
                            .iter()
                            .filter_map(|l| l.as_str().map(|s| s.to_string()))
                            .collect();
                        if !lines.is_empty() {
                            s.log = lines;
                        }
                    }
                    if let Some(connected) = sub_status.get("connected").and_then(|v| v.as_bool()) {
                        if !connected {
                            s.connected = false;
                            s.state = "disconnected".to_string();
                            break;
                        }
                    }
                }
            }
            Err(_) => {
                // Status file gone — subprocess likely exited (SIGTERM or fatal error)
                if let Ok(mut s) = status.lock() {
                    s.connected = false;
                    s.state = "disconnected".to_string();
                }
                break;
            }
        }
    }
}
