use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream, UdpSocket};
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
/// LAN 协作服务 — UDP 广播发现 + UDP 消息 + TCP 可靠文件传输 + SQLite 持久化
///
/// - 发现: UDP 广播心跳 (端口 49152)
/// - 消息: std::net::UdpSocket (端口 49152)
/// - 文件传输: std::net::TcpListener (端口 49154)
/// - 消息/文件记录: SQLite (db/lan.rs)
use supertool_core::db::lan::{self, ChatMessage, FileTransfer as DbFileTransfer};
use tauri::Emitter;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Peer {
    pub id: String,
    pub name: String,
    pub avatar: Option<String>,
    pub address: String,
    #[serde(rename = "messagePort")]
    pub message_port: u16,
    pub version: Option<String>,
    #[serde(rename = "lastSeen")]
    pub last_seen: i64,
    pub online: bool,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LanMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub from: Option<String>,
    #[serde(rename = "fromName")]
    pub from_name: Option<String>,
    pub to: Option<String>,
    pub content: Option<String>,
    pub timestamp: Option<i64>,
    #[serde(rename = "messageId")]
    pub message_id: Option<String>,
    // File transfer fields
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
    #[serde(rename = "fileSize")]
    pub file_size: Option<u64>,
    #[serde(rename = "fileId")]
    pub file_id: Option<String>,
    #[serde(rename = "resumeOffset")]
    pub resume_offset: Option<u64>,
    #[serde(rename = "tcpPort")]
    pub tcp_port: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileTransfer {
    pub id: String,
    #[serde(rename = "fromUserId")]
    pub from_user_id: String,
    #[serde(rename = "fromUserName")]
    pub from_user_name: String,
    #[serde(rename = "toUserId")]
    pub to_user_id: String,
    #[serde(rename = "toUserName")]
    pub to_user_name: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "fileSize")]
    pub file_size: u64,
    #[serde(rename = "filePath")]
    pub file_path: Option<String>,
    pub status: String,
    pub progress: i64,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "completedAt")]
    pub completed_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LanLogEntry {
    pub time: String,
    pub level: String,
    pub message: String,
}

const DISCOVERY_PORT: u16 = 49152;
const FILE_TRANSFER_PORT: u16 = 49154;
const HEARTBEAT_INTERVAL_SECS: u64 = 5;
const PEER_TIMEOUT_SECS: u64 = 30;
const MAX_LOG_ENTRIES: usize = 500;

pub struct LanService {
    user_id: String,
    user_name: String,
    nick_name: Mutex<String>,
    avatar: Mutex<String>,
    my_status: Mutex<String>,
    version: String,

    udp_socket: Mutex<Option<Arc<UdpSocket>>>,
    tcp_port: Mutex<u16>,

    peers: Arc<Mutex<HashMap<String, Peer>>>,
    log_buffer: Arc<Mutex<Vec<LanLogEntry>>>,

    receive_path: Mutex<String>,
    file_transfers: Arc<Mutex<HashMap<String, FileTransfer>>>,
    message_history: Arc<Mutex<Vec<LanMessage>>>,

    is_running: AtomicBool,
    stop_flag: Arc<AtomicBool>,

    local_ip: Mutex<String>,
    app_handle: Mutex<Option<tauri::AppHandle>>,

    /// SQLite connection for message/transfer persistence
    db_conn: Arc<Mutex<Connection>>,
}

impl LanService {
    pub fn new(user_id: String, user_name: String, db_conn: Arc<Mutex<Connection>>) -> Self {
        // Load persisted nickname and avatar from DB
        let nick_name = if let Ok(conn) = db_conn.lock() {
            lan::get_lan_setting(&conn, &format!("nick_name:{}", user_id))
                .ok()
                .flatten()
                .unwrap_or_default()
        } else {
            String::new()
        };
        let avatar = if let Ok(conn) = db_conn.lock() {
            lan::get_lan_setting(&conn, &format!("avatar:{}", user_id))
                .ok()
                .flatten()
                .unwrap_or_else(|| "😀".to_string())
        } else {
            "😀".to_string()
        };

        Self {
            user_id: user_id.clone(),
            user_name,
            nick_name: Mutex::new(nick_name),
            avatar: Mutex::new(avatar),
            my_status: Mutex::new("online".to_string()),
            version: env!("CARGO_PKG_VERSION").to_string(),
            udp_socket: Mutex::new(None),
            tcp_port: Mutex::new(FILE_TRANSFER_PORT),
            peers: Arc::new(Mutex::new(HashMap::new())),
            log_buffer: Arc::new(Mutex::new(vec![])),
            receive_path: Mutex::new(String::new()),
            file_transfers: Arc::new(Mutex::new(HashMap::new())),
            message_history: Arc::new(Mutex::new(vec![])),
            is_running: AtomicBool::new(false),
            stop_flag: Arc::new(AtomicBool::new(false)),
            local_ip: Mutex::new(String::new()),
            app_handle: Mutex::new(None),
            db_conn,
        }
    }

    pub fn start(&self) -> Result<(), String> {
        if self.is_running.load(Ordering::SeqCst) {
            eprintln!("[LAN] Service already running, skipping start");
            return Ok(());
        }
        log::info!("[LAN] Starting LAN service...");

        // Setup receive path
        let receive_path = supertool_core::logic::data_dir::received_files_dir()
            .to_string_lossy()
            .to_string();
        *self.receive_path.lock().unwrap() = receive_path.clone();
        fs::create_dir_all(&receive_path).map_err(|e| format!("创建接收目录失败: {}", e))?;
        log::info!("[LAN] Receive path: {}", receive_path);

        // Start UDP socket for message transport (kept for peer-to-peer messaging)
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), DISCOVERY_PORT);
        let sock = socket2::Socket::new(
            socket2::Domain::IPV4,
            socket2::Type::DGRAM,
            Some(socket2::Protocol::UDP),
        )
        .map_err(|e| format!("创建 UDP socket 失败: {}", e))?;
        sock.set_reuse_address(true)
            .map_err(|e| format!("set_reuse_address 失败: {}", e))?;
        sock.set_broadcast(true)
            .map_err(|e| format!("set_broadcast 失败: {}", e))?;
        // SO_REUSEPORT on macOS for UDP broadcast/multicast receiving
        #[cfg(target_os = "macos")]
        {
            let raw_fd = sock.as_raw_fd();
            let optval: libc::c_int = 1;
            if unsafe {
                libc::setsockopt(
                    raw_fd,
                    libc::SOL_SOCKET,
                    libc::SO_REUSEPORT,
                    &optval as *const _ as *const libc::c_void,
                    std::mem::size_of_val(&optval) as libc::socklen_t,
                )
            } != 0
            {
                log::warn!("[LAN] setsockopt SO_REUSEPORT 失败");
            }
        }
        sock.bind(&addr.into())
            .map_err(|e| format!("UDP 绑定失败: {}", e))?;
        sock.set_nonblocking(true)
            .map_err(|e| format!("set_nonblocking 失败: {}", e))?;
        let udp: UdpSocket = sock.into();
        // Join multicast group for LAN discovery
        let mc_addr = Ipv4Addr::new(239, 255, 0, 1);
        if let Err(e) = udp.join_multicast_v4(&mc_addr, &Ipv4Addr::UNSPECIFIED) {
            log::warn!("[LAN] join_multicast_v4 失败: {}", e);
        } else {
            log::info!("[LAN] Joined multicast group {}", mc_addr);
        }
        let udp = Arc::new(udp);
        *self.udp_socket.lock().unwrap() = Some(Arc::clone(&udp));
        log::info!("[LAN] UDP socket bound on 0.0.0.0:{}", DISCOVERY_PORT);

        let tcp_port = *self.tcp_port.lock().unwrap();
        log::info!("[LAN] TCP file transfer on port {}", tcp_port);

        self.is_running.store(true, Ordering::SeqCst);
        self.stop_flag.store(false, Ordering::SeqCst);

        // Detect and set local IP
        let local_ip = Self::detect_local_ip().unwrap_or_else(|| "127.0.0.1".to_string());
        *self.local_ip.lock().unwrap() = local_ip.clone();
        log::info!("[LAN] Detected local IP: {}", local_ip);
        // Log all usable network interfaces for debugging
        if let Ok(interfaces) = if_addrs::get_if_addrs() {
            for iface in interfaces {
                if let if_addrs::IfAddr::V4(v4) = iface.addr {
                    log::info!(
                        "[LAN] Network interface: {} ip={} netmask={} broadcast={:?}",
                        iface.name, v4.ip, v4.netmask, v4.broadcast
                    );
                }
            }
        }

        // ===== UDP receive thread =====
        let peers = Arc::clone(&self.peers);
        let log = Arc::clone(&self.log_buffer);
        // 心跳线程独立保存一份日志缓冲引用（必须在 recv 线程闭包 move 之前 clone）
        let hb_log = Arc::clone(&self.log_buffer);
        let stop = Arc::clone(&self.stop_flag);
        let user_id = self.user_id.clone();
        let nick_name = self.nick_name.lock().unwrap().clone();
        let avatar = self.avatar.lock().unwrap().clone();
        let my_status = self.my_status.lock().unwrap().clone();
        let version = self.version.clone();
        let msg_history = Arc::clone(&self.message_history);
        let file_transfers = Arc::clone(&self.file_transfers);
        let receive_path = self.receive_path.lock().unwrap().clone();
        let recv_app_handle = self.app_handle.lock().unwrap().clone();
        let db_conn = Arc::clone(&self.db_conn);
        let tcp_p = tcp_port;

        let recv_udp = Arc::clone(&udp);
        let recv_local_ip = local_ip.clone();
        thread::spawn(move || {
            let mut buf = [0u8; 65536];
            let mut recv_count = 0u64;
            let mut external_recv_count = 0u64;
            let mut last_external_recv_time = std::time::Instant::now();
            while !stop.load(Ordering::SeqCst) {
                match recv_udp.recv_from(&mut buf) {
                    Ok((len, addr)) => {
                        recv_count += 1;
                        let is_local = addr.ip().to_string() == recv_local_ip
                            || addr.ip().is_loopback();
                        if !is_local {
                            external_recv_count += 1;
                            last_external_recv_time = std::time::Instant::now();
                            log::info!(
                                "[UDP RECV] *** EXTERNAL packet #{} from {}:{} len={} ***",
                                external_recv_count, addr.ip(), addr.port(), len
                            );
                        }
                        if recv_count <= 5 || recv_count % 50 == 0 {
                            Self::add_log_static(
                                &log,
                                "info",
                                &format!(
                                    "[UDP RECV] #{} from {}:{} len={} {}",
                                    recv_count,
                                    addr.ip(),
                                    addr.port(),
                                    len,
                                    if is_local { "(local)" } else { "*** EXTERNAL ***" }
                                ),
                            );
                        }
                        if let Ok(text) = std::str::from_utf8(&buf[..len]) {
                            if let Ok(data) = serde_json::from_str::<serde_json::Value>(text) {
                                Self::handle_udp_message(
                                    &data,
                                    &addr,
                                    &user_id,
                                    &nick_name,
                                    &avatar,
                                    &my_status,
                                    &version,
                                    &peers,
                                    &log,
                                    &msg_history,
                                    &file_transfers,
                                    &receive_path,
                                    tcp_p,
                                    &recv_app_handle,
                                    &db_conn,
                                );
                            } else {
                                log::warn!(
                                    "[UDP RECV] JSON parse failed from {} (len={}): {}",
                                    addr.ip(), len,
                                    &text[..len.min(100)]
                                );
                            }
                        } else {
                            log::warn!("[UDP RECV] Non-UTF8 data from {} (len={})", addr.ip(), len);
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        let ext_idle = last_external_recv_time.elapsed();
                        if external_recv_count == 0 && ext_idle.as_secs() >= 30 && ext_idle.as_secs() % 30 == 0 {
                            log::info!(
                                "[UDP RECV] idle {}s, no EXTERNAL packets (own packets received={})",
                                ext_idle.as_secs(), recv_count
                            );
                        }
                        thread::sleep(Duration::from_millis(100));
                    }
                    Err(e) => {
                        log::warn!("[UDP RECV] recv_from error: {} (kind={:?})", e, e.kind());
                        thread::sleep(Duration::from_millis(100));
                    }
                }
            }
        });

        // ===== UDP broadcast heartbeat thread =====
        {
            let hb_udp = Arc::clone(&udp);
            let hb_stop = Arc::clone(&self.stop_flag);
            let hb_user_id = self.user_id.clone();
            let hb_user_name = self.user_name.clone();
            let hb_avatar = self.avatar.lock().unwrap().clone();
            let hb_status = self.my_status.lock().unwrap().clone();
            let hb_version = self.version.clone();
            let hb_local_ip = self.local_ip.lock().unwrap().clone();
            log::info!("[LAN] My version for heartbeat: {}", hb_version);

            thread::spawn(move || {
                let broadcast_addr =
                    SocketAddr::new(IpAddr::V4(Ipv4Addr::BROADCAST), DISCOVERY_PORT);
                let multicast_addr =
                    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(239, 255, 0, 1)), DISCOVERY_PORT);
                // Compute directed broadcast from detected local IP
                let directed_broadcast = if let Some(octets) = Self::ip_to_octets(&hb_local_ip) {
                    // Assume /24 subnet: x.y.z.255
                    Some(SocketAddr::new(
                        IpAddr::V4(Ipv4Addr::new(octets[0], octets[1], octets[2], 255)),
                        DISCOVERY_PORT,
                    ))
                } else {
                    None
                };
                let mut hb_count = 0u64;
                let mut consecutive_failures = 0u64;
                const FAILURE_ALERT_THRESHOLD: u64 = 5;
                loop {
                    if hb_stop.load(Ordering::SeqCst) {
                        break;
                    }
                    thread::sleep(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));
                    hb_count += 1;

                    let hb = serde_json::json!({
                        "type": "heartbeat",
                        "userId": hb_user_id,
                        "userName": hb_user_name,
                        "avatar": hb_avatar,
                        "status": hb_status,
                        "version": hb_version,
                        "messagePort": DISCOVERY_PORT,
                    });
                    if let Ok(msg) = serde_json::to_string(&hb) {
                        // 一次心跳周期内至少一路发成功即视为网络可用
                        let mut any_success = false;
                        if let Err(e) = hb_udp.send_to(msg.as_bytes(), broadcast_addr) {
                            log::warn!("[LAN] HB#{} broadcast FAILED: {}", hb_count, e);
                        } else {
                            any_success = true;
                        }
                        // Also send to multicast group (239.255.0.1)
                        if let Err(e) = hb_udp.send_to(msg.as_bytes(), multicast_addr) {
                            log::warn!("[LAN] HB#{} multicast FAILED: {}", hb_count, e);
                        } else {
                            any_success = true;
                        }
                        // Also send to directed broadcast (192.168.x.255) if available
                        if let Some(ref dbc) = directed_broadcast {
                            if let Err(e) = hb_udp.send_to(msg.as_bytes(), dbc) {
                                log::warn!("[LAN] HB#{} directed_broadcast {} FAILED: {}", hb_count, dbc, e);
                            } else {
                                any_success = true;
                            }
                        }
                        // 网络断开自检：连续多周期全路失败才告警，恢复后提示（UDP socket 本身可自愈，无需重建）
                        if !any_success {
                            consecutive_failures += 1;
                            if consecutive_failures == FAILURE_ALERT_THRESHOLD {
                                Self::add_log_static(
                                    &hb_log,
                                    "warning",
                                    "连续多次广播失败，网络可能断开，进入自动重连等待（恢复后自动继续广播）",
                                );
                            }
                        } else {
                            if consecutive_failures >= FAILURE_ALERT_THRESHOLD {
                                Self::add_log_static(
                                    &hb_log,
                                    "info",
                                    "广播已恢复，自动重连成功",
                                );
                            }
                            consecutive_failures = 0;
                        }
                        if hb_count <= 5 || hb_count % 5 == 0 {
                            log::info!(
                                "[LAN] HB#{} sent to {}, {}, {:?}",
                                hb_count,
                                broadcast_addr,
                                multicast_addr,
                                directed_broadcast,
                            );
                        }
                    }
                }
            });
            log::info!(
                "[LAN] UDP heartbeat broadcast started (every {}s)",
                HEARTBEAT_INTERVAL_SECS
            );
        }

        // ===== Heartbeat thread (peer timeout check only, no UDP broadcast) =====
        {
            let heartbeat_stop = Arc::clone(&self.stop_flag);
            let heartbeat_peers = Arc::clone(&self.peers);
            let heartbeat_log = Arc::clone(&self.log_buffer);
            let heartbeat_app = self.app_handle.lock().unwrap().clone();
            let heartbeat_db = Arc::clone(&self.db_conn);

            thread::spawn(move || {
                while !heartbeat_stop.load(Ordering::SeqCst) {
                    thread::sleep(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));
                    Self::check_offline_peers(&heartbeat_peers, &heartbeat_log, &heartbeat_app, &heartbeat_db);
                }
            });
        }

        // ===== TCP accept thread =====
        {
            let tcp_stop = Arc::clone(&self.stop_flag);
            let tcp_port_val = tcp_port;
            let receive_path = self.receive_path.lock().unwrap().clone();
            let transfers = Arc::clone(&self.file_transfers);
            let log = Arc::clone(&self.log_buffer);
            let my_user_id = self.user_id.clone();
            let my_nick = self.nick_name.lock().unwrap().clone();
            let db_conn = Arc::clone(&self.db_conn);
            let tcp_app_handle = self.app_handle.lock().unwrap().clone();

            thread::spawn(move || {
                let listener =
                    match std::net::TcpListener::bind(format!("0.0.0.0:{}", tcp_port_val)) {
                        Ok(l) => l,
                        Err(e) => {
                            Self::add_log_static(
                                &log,
                                "error",
                                &format!("TCP listen failed: {}", e),
                            );
                            return;
                        }
                    };
                for stream in listener.incoming() {
                    if tcp_stop.load(Ordering::SeqCst) {
                        break;
                    }
                    if let Ok(stream) = stream {
                        let rp = receive_path.clone();
                        let tf = Arc::clone(&transfers);
                        let lg = Arc::clone(&log);
                        let uid = my_user_id.clone();
                        let nick = my_nick.clone();
                        let dc = Arc::clone(&db_conn);
                        let ah = tcp_app_handle.clone();
                        thread::spawn(move || {
                            Self::handle_tcp_connection(
                                stream, &rp, &tf, &lg, &uid, &nick, &dc, &ah,
                            );
                        });
                    }
                }
            });
        }

        self.add_log("info", "LAN service started");
        Ok(())
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        self.stop_flag.store(true, Ordering::SeqCst);

        // Close UDP socket to unblock recv thread
        if let Some(udp) = self.udp_socket.lock().unwrap().take() {
            let _ = udp.connect("127.0.0.1:1");
            drop(udp);
        }
        self.peers.lock().unwrap().clear();
        self.add_log("info", "LAN service stopped");
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    /// 确保服务在运行：未启动或启动后异常停止时重新拉起（重连保障）。
    /// start() 内部已有 is_running 守卫（已在运行直接跳过），可安全反复调用。
    pub fn ensure_running(&self) -> Result<(), String> {
        if self.is_running.load(Ordering::SeqCst) {
            return Ok(());
        }
        self.start()
    }

    pub fn set_app_handle(&self, app: tauri::AppHandle) {
        *self.app_handle.lock().unwrap() = Some(app);
    }

    fn emit_event(&self, event: &str, payload: &serde_json::Value) {
        if let Some(app) = self.app_handle.lock().unwrap().as_ref() {
            let _ = app.emit(event, payload);
        }
    }

    /// Handle incoming UDP messages
    fn handle_udp_message(
        data: &serde_json::Value,
        addr: &SocketAddr,
        my_user_id: &str,
        my_nick: &str,
        my_avatar: &str,
        my_status: &str,
        my_version: &str,
        peers: &Arc<Mutex<HashMap<String, Peer>>>,
        log: &Arc<Mutex<Vec<LanLogEntry>>>,
        msg_history: &Arc<Mutex<Vec<LanMessage>>>,
        file_transfers: &Arc<Mutex<HashMap<String, FileTransfer>>>,
        _receive_path: &str,
        _my_tcp_port: u16,
        app_handle: &Option<tauri::AppHandle>,
        db_conn: &Arc<Mutex<Connection>>,
    ) {
        let msg_type = data["type"].as_str().unwrap_or("");
        log::debug!("[LAN] handle_udp_message type={:?} from {}", msg_type, addr.ip());

        match msg_type {
            "heartbeat" | "discovery" => {
                let peer_id = data["userId"].as_str().unwrap_or("");
                if peer_id.is_empty() {
                    log::warn!("[LAN] Dropping heartbeat: empty userId from {}", addr.ip());
                    return;
                }
                if peer_id == my_user_id {
                    log::debug!("[LAN] Dropping own heartbeat from {} (my_user_id={})", addr.ip(), my_user_id);
                    return;
                }

                let peer_name = data["userName"]
                    .as_str()
                    .or_else(|| data["name"].as_str())
                    .unwrap_or(peer_id)
                    .to_string();
                let peer_avatar = data["avatar"].as_str().map(|s| s.to_string());
                let peer_version = data["version"].as_str().map(|s| s.to_string());
                let peer_status = data["status"].as_str().map(|s| s.to_string());
                // Use messagePort from packet if available, otherwise use sender's port
                let message_port = data["messagePort"]
                    .as_u64()
                    .map(|p| p as u16)
                    .unwrap_or(addr.port());

                // Log version info without blocking — allow cross-version communication
                if let Some(ref v) = peer_version {
                    if let Some(peer_major) = v.split('.').next() {
                        if let Some(my_major) = my_version.split('.').next() {
                            if peer_major != my_major {
                                Self::add_log_static(
                                    log,
                                    "info",
                                    &format!(
                                        "Peer {} version (v{}) differs from local (v{}) — cross-version communication allowed",
                                        peer_id, v, my_version
                                    ),
                                );
                            }
                        }
                    }
                }

                let now = chrono::Utc::now().timestamp_millis();
                let mut peers_map = peers.lock().unwrap();
                let is_new = !peers_map.contains_key(peer_id);

                // 如果 peer 已存在且有本地保存的头像（avatar:peer_ 开头），保留本地头像
                let preserved_avatar = if !is_new {
                    peers_map
                        .get(peer_id)
                        .and_then(|p| p.avatar.as_ref())
                        .filter(|a| a.starts_with("avatar:peer_"))
                        .cloned()
                } else {
                    None
                };

                let peer = Peer {
                    id: peer_id.to_string(),
                    name: peer_name.clone(),
                    avatar: preserved_avatar.or(peer_avatar.clone()),
                    address: addr.ip().to_string(),
                    message_port,
                    version: peer_version,
                    last_seen: now,
                    online: true,
                    status: peer_status,
                };
                peers_map.insert(peer_id.to_string(), peer);

                // Save peer to DB for persistence across restarts
                if let Ok(conn) = db_conn.lock() {
                    let _ = lan::insert_user(
                        &conn,
                        &supertool_core::db::lan::LanUser {
                            id: peer_id.to_string(),
                            name: peer_name.clone(),
                            ip: addr.ip().to_string(),
                            port: message_port as i64,
                            last_seen: chrono::Utc::now()
                                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                                .to_string(),
                            is_online: true,
                        },
                    );
                }

                // Get peer reference for further use
                let peer_ref = peers_map.get(peer_id).unwrap();

                // Request avatar image if peer has image avatar and we don't have it locally
                if let Some(ref peer_avatar_ref) = peer_ref.avatar {
                    if peer_avatar_ref.starts_with("avatar:")
                        && !peer_avatar_ref.starts_with("avatar:peer_")
                    {
                        // Peer has image avatar (not emoji or already-saved peer avatar)
                        // Check if we have the local copy
                        let data_dir = supertool_core::logic::data_dir::resolve_data_dir();
                        let filename = peer_avatar_ref.strip_prefix("avatar:").unwrap_or("");
                        let local_path = data_dir.join("avatars").join(filename);

                        if !local_path.exists() {
                            // We don't have the image, request it from peer
                            Self::add_log_static(
                                log,
                                "info",
                                &format!(
                                    "Requesting avatar from {} (we don't have {} locally)",
                                    peer_id, peer_avatar_ref
                                ),
                            );

                            let request = serde_json::json!({
                                "type": "avatar_request",
                                "from": my_user_id,
                                "fromName": if my_nick.is_empty() { my_user_id } else { my_nick },
                                "targetAvatar": peer_avatar_ref,
                                "timestamp": chrono::Utc::now().timestamp_millis(),
                            });

                            if let Ok(msg) = serde_json::to_string(&request) {
                                // Send request directly using a temporary UDP socket
                                if let Ok(request_sock) = UdpSocket::bind("0.0.0.0:0") {
                                    let _ = request_sock.send_to(
                                        msg.as_bytes(),
                                        format!("{}:{}", addr.ip(), message_port),
                                    );
                                    Self::add_log_static(
                                        log,
                                        "info",
                                        &format!(
                                            "Sent avatar_request to {} for {}",
                                            peer_id, peer_avatar_ref
                                        ),
                                    );
                                }
                            }
                        }
                    }
                }

                if is_new {
                    Self::add_log_static(
                        log,
                        "info",
                        &format!("Peer discovered: {} ({})", peer_id, addr.ip()),
                    );
                    if let Some(app) = app_handle {
                        let payload = serde_json::json!({
                            "id": peer_id,
                            "userId": peer_id,  // Explicit userId for frontend matching
                            "address": addr.ip().to_string(),
                            "name": peer_name,
                            "avatar": peer_ref.avatar,
                            "version": peer_ref.version,
                            "status": peer_ref.status,
                            "messagePort": message_port,
                        });
                        let _ = app.emit("lan-peer-discovered", payload);
                    }
                }

                // Reply to discovery — send back to the sender's address
                if msg_type == "discovery" {
                    let reply = serde_json::json!({
                        "type": "discovery",
                        "userId": my_user_id,
                        "name": if my_nick.is_empty() { my_user_id } else { my_nick },
                        "userName": if my_nick.is_empty() { my_user_id } else { my_nick },
                        "avatar": my_avatar,
                        "status": my_status,
                        "version": my_version,
                        "messagePort": DISCOVERY_PORT,
                        "timestamp": now,
                    });
                    if let Ok(msg) = serde_json::to_string(&reply) {
                        // Reply to sender's actual port (not DISCOVERY_PORT)
                        if let Ok(reply_sock) = UdpSocket::bind("0.0.0.0:0") {
                            let _ = reply_sock
                                .send_to(msg.as_bytes(), format!("{}:{}", addr.ip(), addr.port()));
                        }
                    }
                }
            }
            "message" => {
                if let Ok(msg) = serde_json::from_value::<LanMessage>(data.clone()) {
                    msg_history.lock().unwrap().push(msg.clone());

                    // Persist to chat_messages table
                    let my_nick_str = if my_nick.is_empty() {
                        my_user_id.to_string()
                    } else {
                        my_nick.to_string()
                    };
                    let chat_msg = ChatMessage {
                        id: msg.message_id.clone().unwrap_or_else(|| {
                            format!("msg-{}", chrono::Utc::now().timestamp_millis())
                        }),
                        from_user_id: msg.from.clone().unwrap_or_default(),
                        from_user_name: msg.from_name.clone().unwrap_or_default(),
                        to_user_id: my_user_id.to_string(),
                        to_user_name: my_nick_str,
                        content: msg.content.clone(),
                        msg_type: "text".to_string(),
                        file_name: None,
                        file_size: None,
                        file_path: None,
                        status: "received".to_string(),
                        progress: 0,
                        created_at: chrono::Utc::now().to_rfc3339(),
                        read: false,
                    };
                    if let Ok(conn) = db_conn.lock() {
                        let _ = lan::insert_chat_message(&conn, &chat_msg);
                    }

                    Self::add_log_static(
                        log,
                        "info",
                        &format!(
                            "Message from {}: {}",
                            msg.from.as_deref().unwrap_or("unknown"),
                            msg.content.as_deref().unwrap_or("")
                        ),
                    );

                    // 发送系统通知
                    let from_name = msg
                        .from_name
                        .as_deref()
                        .or(msg.from.as_deref())
                        .unwrap_or("unknown");
                    let content_preview = msg
                        .content
                        .as_deref()
                        .map(|c| if c.len() > 100 { &c[..100] } else { c })
                        .unwrap_or("");
                    crate::tray_notification::show_lan_message_notification(
                        from_name,
                        content_preview,
                    );

                    if let Some(app) = app_handle {
                        let _ = app.emit("lan-message-received", data.clone());
                    }
                }
            }
            "avatar_update" => {
                // 处理其他用户的头像更新
                let from_id = data["from"].as_str().unwrap_or("");
                if from_id.is_empty() || from_id == my_user_id {
                    return;
                }

                let from_name = data["fromName"].as_str().unwrap_or(from_id);
                let avatar_ref = data["avatar"].as_str();
                let avatar_data = data["avatarData"].as_str();
                let avatar_ext = data["avatarExt"].as_str().unwrap_or("png");

                if let (Some(_avatar_ref), Some(avatar_data)) = (avatar_ref, avatar_data) {
                    // 保存头像图片到本地 avatars 目录，使用发送者 user_id 标识
                    let data_dir = supertool_core::logic::data_dir::resolve_data_dir();
                    let avatars_dir = data_dir.join("avatars");

                    if !avatars_dir.exists() {
                        let _ = fs::create_dir_all(&avatars_dir);
                    }

                    // 使用 peer_id 作为文件名，避免冲突
                    let local_filename = format!("peer_{}.{}", from_id, avatar_ext);
                    let local_path = avatars_dir.join(&local_filename);

                    // 解码 base64 并保存
                    if let Ok(decoded) = BASE64.decode(avatar_data) {
                        if let Ok(_) = fs::write(&local_path, decoded) {
                            // 更新 peer 的 avatar 字段
                            let local_avatar_ref = format!("avatar:{}", local_filename);
                            if let Ok(mut peers_map) = peers.lock() {
                                if let Some(peer) = peers_map.get_mut(from_id) {
                                    peer.avatar = Some(local_avatar_ref.clone());
                                }
                            }

                            Self::add_log_static(
                                log,
                                "info",
                                &format!(
                                    "Avatar update from {}: saved to {}",
                                    from_name,
                                    local_path.display()
                                ),
                            );

                            // 发送前端事件通知更新
                            if let Some(app) = app_handle {
                                let _ = app.emit(
                                    "lan-peer-avatar-updated",
                                    serde_json::json!({
                                        "userId": from_id,
                                        "name": from_name,
                                        "avatar": local_avatar_ref,
                                        "avatarPath": local_path.to_string_lossy().to_string(),
                                    }),
                                );
                            }
                        }
                    }
                }
            }
            "avatar_request" => {
                // 收到头像请求，需要发送自己的头像图片给请求者
                let from_id = data["from"].as_str().unwrap_or("");
                if from_id.is_empty() || from_id == my_user_id {
                    return; // 忽略自己发出的请求
                }

                let from_name = data["fromName"].as_str().unwrap_or(from_id);
                let target_avatar = data["targetAvatar"].as_str().unwrap_or("");

                Self::add_log_static(
                    log,
                    "info",
                    &format!(
                        "Received avatar_request from {} for {}",
                        from_name, target_avatar
                    ),
                );

                // Check if the requested avatar matches my current avatar
                if target_avatar == my_avatar && my_avatar.starts_with("avatar:") {
                    // Read my avatar file and send to requester
                    let filename = my_avatar.strip_prefix("avatar:").unwrap_or("");
                    let data_dir = supertool_core::logic::data_dir::resolve_data_dir();
                    let avatar_path = data_dir.join("avatars").join(filename);

                    if avatar_path.exists() {
                        if let Ok(image_data) = fs::read(&avatar_path) {
                            let base64_data = BASE64.encode(&image_data);
                            let ext = avatar_path
                                .extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or("png");

                            let reply = serde_json::json!({
                                "type": "avatar_update",
                                "from": my_user_id,
                                "fromName": if my_nick.is_empty() { my_user_id } else { my_nick },
                                "avatar": my_avatar,
                                "avatarData": base64_data,
                                "avatarExt": ext,
                                "timestamp": chrono::Utc::now().timestamp_millis(),
                            });

                            if let Ok(msg) = serde_json::to_string(&reply) {
                                // Reply directly to sender's address
                                if let Ok(reply_sock) = UdpSocket::bind("0.0.0.0:0") {
                                    let _ = reply_sock.send_to(
                                        msg.as_bytes(),
                                        format!("{}:{}", addr.ip(), addr.port()),
                                    );
                                    Self::add_log_static(
                                        log,
                                        "info",
                                        &format!("Sent avatar {} to {}", my_avatar, from_name),
                                    );
                                }
                            }
                        }
                    }
                }
            }
            "file_start" => {
                let file_id = data["id"].as_str().or_else(|| data["fileId"].as_str());
                let file_name = data["fileName"].as_str();
                let file_size = data["fileSize"].as_u64();
                let from_id = data["fromUserId"]
                    .as_str()
                    .or_else(|| data["from"].as_str());
                let from_name = data["fromUserName"]
                    .as_str()
                    .or_else(|| data["fromName"].as_str());
                if let (
                    Some(file_id),
                    Some(file_name),
                    Some(file_size),
                    Some(from_id),
                    Some(from_name),
                ) = (file_id, file_name, file_size, from_id, from_name)
                {
                    let to_name = if my_nick.is_empty() {
                        my_user_id.to_string()
                    } else {
                        my_nick.to_string()
                    };
                    let transfer = FileTransfer {
                        id: file_id.to_string(),
                        from_user_id: from_id.to_string(),
                        from_user_name: from_name.to_string(),
                        to_user_id: my_user_id.to_string(),
                        to_user_name: to_name.clone(),
                        file_name: file_name.to_string(),
                        file_size,
                        file_path: None,
                        status: "receiving".to_string(),
                        progress: 0,
                        created_at: chrono::Utc::now().to_rfc3339(),
                        completed_at: None,
                    };
                    file_transfers
                        .lock()
                        .unwrap()
                        .insert(file_id.to_string(), transfer);

                    // Persist to file_transfers table
                    if let Ok(conn) = db_conn.lock() {
                        let db_ft = DbFileTransfer {
                            id: file_id.to_string(),
                            from_user_id: from_id.to_string(),
                            from_user_name: from_name.to_string(),
                            to_user_id: my_user_id.to_string(),
                            to_user_name: to_name.clone(),
                            file_name: file_name.to_string(),
                            file_size: file_size as i64,
                            file_path: None,
                            status: "receiving".to_string(),
                            progress: 0,
                            created_at: chrono::Utc::now().to_rfc3339(),
                            completed_at: None,
                            local_user_id: Some(my_user_id.to_string()),
                        };
                        let _ = lan::insert_file_transfer(&conn, &db_ft);
                    }

                    // Also write a chat_message entry for the file — content stores JSON metadata
                    if let Ok(conn) = db_conn.lock() {
                        let content_json = serde_json::json!({
                            "fileName": file_name,
                            "fileSize": file_size,
                            "filePath": "",
                            "isImage": Self::is_image_file(file_name),
                        })
                        .to_string();
                        let chat_msg = ChatMessage {
                            id: file_id.to_string(),
                            from_user_id: from_id.to_string(),
                            from_user_name: from_name.to_string(),
                            to_user_id: my_user_id.to_string(),
                            to_user_name: if my_nick.is_empty() {
                                my_user_id.to_string()
                            } else {
                                my_nick.to_string()
                            },
                            content: Some(content_json),
                            msg_type: "file".to_string(),
                            file_name: Some(file_name.to_string()),
                            file_size: Some(file_size as i64),
                            file_path: None,
                            status: "receiving".to_string(),
                            progress: 0,
                            created_at: chrono::Utc::now().to_rfc3339(),
                            read: false,
                        };
                        let _ = lan::insert_chat_message(&conn, &chat_msg);
                    }

                    Self::add_log_static(
                        log,
                        "info",
                        &format!("File transfer started: {} ({} bytes)", file_name, file_size),
                    );
                    if let Some(app) = app_handle {
                        let payload = serde_json::json!({
                            "fileId": file_id,
                            "fileName": file_name,
                            "fileSize": file_size,
                            "from": from_id,
                            "fromName": from_name,
                            "status": "receiving",
                        });
                        let _ = app.emit("lan-file-transfer-started", payload);
                    }
                }
            }
            "file_start_ack" => {
                // Electron sends file_start_ack when it receives file_start.
                // Tauri uses TCP for file transfer, so this ACK is informational only.
                // Log it for debugging.
                let file_id = data["id"].as_str().unwrap_or("");
                Self::add_log_static(
                    log,
                    "info",
                    &format!("file_start_ack received for file: {}", file_id),
                );
            }
            // Collaboration message types — forward to frontend
            // Map msg_type to hyphen-format event names matching frontend listeners
            "assign_task"
            | "task_update"
            | "task_status_change"
            | "task_comment"
            | "collaboration_started"
            | "collaboration_ended" => {
                // Persist assign_task to chat_messages DB for history loading
                if msg_type == "assign_task" {
                    let msg_id = data["messageId"]
                        .as_str()
                        .or_else(|| data["id"].as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| {
                            format!("msg-{}", chrono::Utc::now().timestamp_millis())
                        });
                    let from_id = data["from"].as_str().unwrap_or("unknown");
                    let from_name = data["fromName"].as_str().unwrap_or("unknown");
                    let to_id = data["to"].as_str().unwrap_or(my_user_id);
                    let to_name = data["toName"].as_str().unwrap_or(my_nick);
                    let task_data = data["task"].clone();
                    let content_str = serde_json::to_string(&task_data).unwrap_or_default();
                    // Log first before content_str is moved
                    Self::add_log_static(
                        log,
                        "info",
                        &format!("Task assigned from {}: {}", from_id, &content_str),
                    );
                    let chat_msg = ChatMessage {
                        id: msg_id.to_string(),
                        from_user_id: from_id.to_string(),
                        from_user_name: from_name.to_string(),
                        to_user_id: to_id.to_string(),
                        to_user_name: to_name.to_string(),
                        content: Some(content_str),
                        msg_type: "task_assigned".to_string(),
                        file_name: None,
                        file_size: None,
                        file_path: None,
                        status: "received".to_string(),
                        progress: 0,
                        created_at: chrono::Utc::now().to_rfc3339(),
                        read: false,
                    };
                    if let Ok(conn) = db_conn.lock() {
                        let _ = lan::insert_chat_message(&conn, &chat_msg);
                    }
                }
                if let Some(app) = app_handle {
                    let event_name = match msg_type {
                        "assign_task" => "lan-task-assigned",
                        "task_update" => "lan-task-updated",
                        "task_status_change" => "lan-task-status-changed",
                        "task_comment" => "lan-task-comment-added",
                        "collaboration_started" => "lan-collaboration-started",
                        "collaboration_ended" => "lan-collaboration-ended",
                        _ => msg_type,
                    };
                    let _ = app.emit(event_name, data.clone());
                }
            }
            _ => {}
        }
    }

    /// Handle incoming TCP connections for file transfers
    fn handle_tcp_connection(
        stream: TcpStream,
        receive_path: &str,
        transfers: &Arc<Mutex<HashMap<String, FileTransfer>>>,
        log: &Arc<Mutex<Vec<LanLogEntry>>>,
        my_user_id: &str,
        my_nick: &str,
        db_conn: &Arc<Mutex<Connection>>,
        app_handle: &Option<tauri::AppHandle>,
    ) {
        let mut reader = BufReader::new(stream.try_clone().unwrap());

        // Read sender identity handshake: LAN-SEND <user_id>\n
        let mut identity_line = String::new();
        let sender_id = match reader.read_line(&mut identity_line) {
            Ok(_) if identity_line.starts_with("LAN-SEND ") => identity_line
                .trim()
                .strip_prefix("LAN-SEND ")
                .unwrap_or("unknown")
                .to_string(),
            _ => {
                // Fallback for old clients that don't send handshake
                "unknown".to_string()
            }
        };

        // Read header: FILE <name> <size> <id> or MESSAGE <msg_id> <size>
        let mut header = String::new();
        if reader.read_line(&mut header).is_err() {
            return;
        }

        let parts: Vec<&str> = header.trim().split_whitespace().collect();
        if parts.is_empty() {
            Self::add_log_static(log, "error", &format!("Empty TCP header"));
            return;
        }

        let header_type = parts[0];

        if sender_id != "unknown" {
            Self::add_log_static(
                log,
                "info",
                &format!("TCP handshake verified: sender={}, type={}", sender_id, header_type),
            );
        }

        // Handle MESSAGE type (large text messages via TCP)
        if header_type == "MESSAGE" {
            if parts.len() < 3 {
                Self::add_log_static(log, "error", &format!("Invalid MESSAGE header: {}", header));
                return;
            }
            let msg_id = parts[1];
            let content_size: usize = parts[2].parse().unwrap_or(0);
            if content_size > 4096 {
                Self::add_log_static(log, "error", &format!("MESSAGE too large: {} bytes", content_size));
                return;
            }

            // Read message content
            let mut content_buf = vec![0u8; content_size];
            if let Err(e) = reader.read_exact(&mut content_buf) {
                Self::add_log_static(log, "error", &format!("MESSAGE read failed: {}", e));
                return;
            }
            let content = String::from_utf8_lossy(&content_buf).to_string();

            Self::add_log_static(log, "info", &format!("TCP MESSAGE received: {} bytes from {}", content_size, sender_id));
            log::info!("[LAN] TCP MESSAGE received: {} bytes from {}", content_size, sender_id);

            // Persist to DB
            let chat_msg = ChatMessage {
                id: msg_id.to_string(),
                from_user_id: sender_id.to_string(),
                from_user_name: sender_id.to_string(),
                to_user_id: my_user_id.to_string(),
                to_user_name: my_nick.to_string(),
                content: Some(content.clone()),
                msg_type: "text".to_string(),
                file_name: None,
                file_size: None,
                file_path: None,
                status: "received".to_string(),
                progress: 0,
                created_at: chrono::Utc::now().to_rfc3339(),
                read: false,
            };
            if let Ok(conn) = db_conn.lock() {
                let _ = lan::insert_chat_message(&conn, &chat_msg);
            }

            // Emit event to frontend (same format as UDP)
            if let Some(handle) = app_handle {
                let _ = handle.emit("lan-message-received", serde_json::json!({
                    "type": "message",
                    "from": sender_id,
                    "fromName": sender_id,
                    "to": my_user_id,
                    "toName": my_nick,
                    "content": content,
                    "timestamp": chrono::Utc::now().timestamp_millis(),
                    "messageId": msg_id,
                }));
            }

            // Play notification sound (same as UDP)
            let content_preview: String = content.chars().take(100).collect();
            crate::tray_notification::show_lan_message_notification(&sender_id, &content_preview);
            return;
        }

        // Handle FILE type
        if parts.len() < 4 || header_type != "FILE" {
            Self::add_log_static(log, "error", &format!("Invalid TCP header: {}", header));
            return;
        }

        const MAX_FILE_SIZE: u64 = 500 * 1024 * 1024;
        let raw_file_name = parts[1];
        let file_size: u64 = parts[2].parse().unwrap_or(0);
        let file_id = parts[3];
        
        log::info!(
            "[LAN TCP RECV] FILE header parsed: name={}, size={}, id={}, sender={}",
            raw_file_name, file_size, file_id, sender_id
        );
        Self::add_log_static(
            log,
            "info",
            &format!("TCP FILE received: {} ({} bytes) from {}", raw_file_name, file_size, sender_id),
        );
        // Get from_user info from transfers map if available
        let (from_id, from_name) = {
            let tf = transfers.lock().unwrap();
            if let Some(t) = tf.get(&file_id.to_string()) {
                (t.from_user_id.clone(), t.from_user_name.clone())
            } else {
                ("unknown".to_string(), "unknown".to_string())
            }
        };
        let my_nick_str = if my_nick.is_empty() {
            my_user_id.to_string()
        } else {
            my_nick.to_string()
        };
        if file_size > MAX_FILE_SIZE {
            Self::add_log_static(
                log,
                "error",
                &format!("File too large: {} bytes", file_size),
            );
            return;
        }
        let file_name = PathBuf::from(raw_file_name)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unnamed_file".to_string());

        // Avoid overwriting existing files — add numbered suffix
        let save_path = PathBuf::from(receive_path).join(&file_name);
        let save_path = if save_path.exists() {
            let stem = PathBuf::from(&file_name)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let ext = PathBuf::from(&file_name)
                .extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default();
            let mut counter = 1;
            loop {
                let candidate =
                    PathBuf::from(receive_path).join(format!("{}{}{}", stem, counter, ext));
                if !candidate.exists() {
                    break candidate;
                }
                counter += 1;
            }
        } else {
            save_path
        };

        let mut file = match fs::File::create(&save_path) {
            Ok(f) => f,
            Err(e) => {
                Self::add_log_static(log, "error", &format!("Failed to create file: {}", e));
                return;
            }
        };

        let mut received = 0u64;
        let mut buf = [0u8; 64 * 1024];
        let mut last_emit_pct = 0i64;
        loop {
            // Use reader (BufReader) to read file data, not raw stream
            // BufReader may have buffered data from header reading
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if file.write_all(&buf[..n]).is_err() {
                        break;
                    }
                    received += n as u64;

                    // Emit progress
                    if file_size > 0 {
                        let progress_pct = ((received as f64 / file_size as f64) * 100.0) as i64;
                        if progress_pct >= last_emit_pct + 10 {
                            last_emit_pct = (progress_pct / 10) * 10;
                            {
                                let mut tf = transfers.lock().unwrap();
                                if let Some(t) = tf.get_mut(file_id) {
                                    t.progress = progress_pct;
                                }
                            }
                            // Update DB
                            if let Ok(conn) = db_conn.lock() {
                                let _ = lan::update_file_transfer(
                                    &conn,
                                    file_id,
                                    "receiving",
                                    progress_pct,
                                    None,
                                    None,
                                );
                            }
                            if let Some(app) = app_handle {
                                let _ = app.emit(
                                    "lan-file-transfer-progress",
                                    serde_json::json!({
                                        "fileId": file_id,
                                        "status": "receiving",
                                        "progress": progress_pct,
                                        "received": received,
                                        "total": file_size,
                                    }),
                                );
                            }
                        }
                    }
                }
                Err(_) => break,
            }
        }

        let save_path_str = save_path.to_string_lossy().to_string();

        // Log received bytes vs expected
        log::info!(
            "[LAN TCP RECV] File received: {} bytes (expected {}), saved to {}",
            received, file_size, save_path_str
        );
        Self::add_log_static(
            log,
            "info",
            &format!("File saved: {} ({} bytes)", save_path_str, received),
        );

        // Update in-memory transfer status
        {
            let mut tf = transfers.lock().unwrap();
            if let Some(t) = tf.get_mut(file_id) {
                t.status = "completed".to_string();
                t.progress = 100;
                t.file_path = Some(save_path_str.clone());
                t.completed_at = Some(chrono::Utc::now().to_rfc3339());
            }
        }

        // Update DB
        if let Ok(conn) = db_conn.lock() {
            let completed_at = chrono::Utc::now().to_rfc3339();
            let _ = lan::update_file_transfer(
                &conn,
                file_id,
                "completed",
                100,
                Some(&save_path_str),
                Some(&completed_at),
            );
            // Also update the chat_message for this file — update content JSON with status+filePath
            let content_json = serde_json::json!({
                "fileName": file_name,
                "fileSize": file_size,
                "filePath": save_path_str,
                "status": "completed",
                "isImage": Self::is_image_file(&file_name),
            })
            .to_string();
            let _ = conn.execute(
                "UPDATE chat_messages SET content = ?1 WHERE id = ?2",
                rusqlite::params![content_json, file_id],
            );
        }

        Self::add_log_static(
            log,
            "info",
            &format!("File received: {} ({} bytes)", file_name, received),
        );

        if let Some(app) = app_handle {
            let completed_at = chrono::Utc::now().to_rfc3339();
            let _ = app.emit(
                "lan-file-transfer-completed",
                serde_json::json!({
                    "fileId": file_id,
                    "fileName": file_name,
                    "fileSize": file_size,
                    "received": received,
                    "filePath": save_path_str,
                    "status": "completed",
                    "progress": 100,
                    "completedAt": completed_at,
                    "isImage": Self::is_image_file(&file_name),
                }),
            );
            // Also emit lan-file-received for frontend compatibility
            let _ = app.emit(
                "lan-file-received",
                serde_json::json!({
                    "fileId": file_id,
                    "fromUserId": from_id,
                    "fromUserName": from_name,
                    "toUserId": my_user_id,
                    "toUserName": my_nick_str,
                    "fileName": file_name,
                    "fileSize": file_size,
                    "status": "completed",
                    "progress": 100,
                    "filePath": save_path_str,
                    "isImage": Self::is_image_file(&file_name),
                }),
            );

            // Play notification sound + show system notification for file (same as text message)
            let content_preview = format!("[文件] {}", file_name);
            crate::tray_notification::show_lan_message_notification(&from_name, &content_preview);
        }
    }

    fn check_offline_peers(
        peers: &Arc<Mutex<HashMap<String, Peer>>>,
        log: &Arc<Mutex<Vec<LanLogEntry>>>,
        app_handle: &Option<tauri::AppHandle>,
        db_conn: &Arc<Mutex<Connection>>,
    ) {
        let now = chrono::Utc::now().timestamp_millis();
        let mut peers_map = peers.lock().unwrap();
        for peer in peers_map.values_mut() {
            if peer.online && (now - peer.last_seen) > (PEER_TIMEOUT_SECS as i64 * 1000) {
                peer.online = false;
                Self::add_log_static(log, "info", &format!("Peer offline: {0}", peer.id));
                // Update DB
                if let Ok(conn) = db_conn.lock() {
                    let _ = lan::update_user_online_status(&conn, &peer.id, false);
                }
                // Emit lan-peer-lost event for frontend (with online=false so frontend keeps it)
                if let Some(app) = app_handle {
                    let payload = serde_json::json!({
                        "id": peer.id,
                        "name": peer.name,
                        "address": peer.address,
                        "online": false,
                    });
                    let _ = app.emit("lan-peer-lost", payload);
                }
            }
        }
    }

    fn add_log_static(log: &Arc<Mutex<Vec<LanLogEntry>>>, level: &str, message: &str) {
        let entry = LanLogEntry {
            time: chrono::Local::now().format("%H:%M:%S").to_string(),
            level: level.to_string(),
            message: message.to_string(),
        };
        let mut buf = log.lock().unwrap();
        buf.push(entry);
        let len = buf.len();
        if len > MAX_LOG_ENTRIES {
            buf.drain(0..len - MAX_LOG_ENTRIES);
        }
    }

    fn is_image_file(file_name: &str) -> bool {
        let ext = file_name.split('.').last().unwrap_or("").to_lowercase();
        matches!(
            ext.as_str(),
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "svg"
        )
    }

    fn add_log(&self, level: &str, message: &str) {
        Self::add_log_static(&self.log_buffer, level, message);
    }

    /// Parse an IPv4 string like "192.168.1.69" into [192, 168, 1, 69]
    fn ip_to_octets(ip: &str) -> Option<[u8; 4]> {
        let parts: Vec<&str> = ip.split('.').collect();
        if parts.len() != 4 {
            return None;
        }
        let mut octets = [0u8; 4];
        for (i, part) in parts.iter().enumerate() {
            octets[i] = part.parse().ok()?;
        }
        Some(octets)
    }

    /// Detect local IP by enumerating interfaces,
    /// filtering out loopback and virtual/VPN interfaces.
    fn detect_local_ip() -> Option<String> {
        if let Ok(interfaces) = if_addrs::get_if_addrs() {
            for iface in interfaces {
                if iface.is_loopback() {
                    continue;
                }
                let name = iface.name.to_lowercase();
                if name.starts_with("tun")
                    || name.starts_with("tap")
                    || name.starts_with("utun")
                    || name.starts_with("docker")
                    || name.starts_with("br-")
                    || name.starts_with("veth")
                    || name.starts_with("virbr")
                {
                    continue;
                }
                if let if_addrs::IfAddr::V4(v4) = iface.addr {
                    return Some(v4.ip.to_string());
                }
            }
        }
        None
    }

    /// Send a text message to a peer. Also persists to DB.
    pub fn send_message(&self, peer_id: &str, content: &str) -> Result<bool, String> {
        if !self.is_running.load(Ordering::SeqCst) {
            return Err("LAN service not running".to_string());
        }

        // UDP safe payload threshold — max safe UDP payload ≈ 1400 bytes (IPv4 MTU)
        // JSON overhead ~400 bytes, so safe content limit = 800 bytes
        const UDP_SAFE_THRESHOLD: usize = 800;
        // Maximum total message size (including JSON overhead) — 4KB
        const MAX_MESSAGE_BYTES: usize = 4096;

        if content.len() > MAX_MESSAGE_BYTES {
            return Err(format!(
                "消息过长（{} bytes），上限 {} bytes",
                content.len(),
                MAX_MESSAGE_BYTES
            ));
        }

        let peers = self.peers.lock().unwrap();
        let peer = peers.get(peer_id).ok_or("Peer not found")?;

        let nick = self.nick_name.lock().unwrap().clone();
        // Use UUID for unique message ID to avoid timestamp collision
        let msg_id = format!("msg-{}", uuid::Uuid::new_v4());
        let to_name = peer.name.clone();
        let from_name = if nick.is_empty() {
            self.user_id.clone()
        } else {
            nick.clone()
        };

        // Persist to chat_messages DB first (regardless of send method)
        let chat_msg = ChatMessage {
            id: msg_id.clone(),
            from_user_id: self.user_id.clone(),
            from_user_name: from_name.clone(),
            to_user_id: peer_id.to_string(),
            to_user_name: to_name.clone(),
            content: Some(content.to_string()),
            msg_type: "text".to_string(),
            file_name: None,
            file_size: None,
            file_path: None,
            status: "sent".to_string(),
            progress: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
            read: false,
        };
        if let Ok(conn) = self.db_conn.lock() {
            let _ = lan::insert_chat_message(&conn, &chat_msg);
        }

        // Choose transport based on message size
        if content.len() > UDP_SAFE_THRESHOLD {
            // Use TCP for large messages (reliable delivery)
            log::info!(
                "[LAN] send_message: {} bytes > UDP threshold, using TCP -> {}:{}",
                content.len(),
                peer.address,
                FILE_TRANSFER_PORT
            );
            self.send_message_tcp(&peer, &msg_id, content, &to_name)
        } else {
            // Use UDP for small messages (fast)
            let msg = serde_json::json!({
                "type": "message",
                "from": self.user_id,
                "fromName": &from_name,
                "to": peer_id,
                "toName": peer.name.clone(),
                "content": content,
                "timestamp": chrono::Utc::now().timestamp_millis(),
                "messageId": &msg_id,
            });

            if let Ok(data) = serde_json::to_string(&msg) {
                if let Some(udp) = self.udp_socket.lock().unwrap().as_ref() {
                    let target = format!("{}:{}", peer.address, peer.message_port);
                    let data_len = data.as_bytes().len();
                    log::info!("[LAN] send_message UDP: {} bytes -> {}", data_len, target);
                    match udp.send_to(data.as_bytes(), &target) {
                        Ok(sent_bytes) => {
                            log::info!("[LAN] UDP sent {} bytes (payload {})", sent_bytes, data_len);
                            return Ok(true);
                        }
                        Err(e) => {
                            log::error!("[LAN] UDP send failed: {} (payload {} bytes)", e, data_len);
                            return Err(format!("UDP 发送失败: {}", e));
                        }
                    }
                }
            }
            Err("发送失败".to_string())
        }
    }

    /// Send message via TCP (for large messages that exceed UDP MTU)
    fn send_message_tcp(
        &self,
        peer: &Peer,
        msg_id: &str,
        content: &str,
        _to_name: &str,
    ) -> Result<bool, String> {
        let nick = self.nick_name.lock().unwrap().clone();
        let _from_name = if nick.is_empty() { &self.user_id } else { &nick };

        let addr = format!("{}:{}", peer.address, FILE_TRANSFER_PORT);
        let mut stream = match TcpStream::connect(&addr) {
            Ok(s) => s,
            Err(e) => {
                log::error!("[LAN] TCP connect failed: {}", e);
                return Err(format!("TCP 连接失败: {}", e));
            }
        };

        // Handshake: LAN-SEND <user_id>\n
        let handshake = format!("LAN-SEND {}\n", self.user_id);
        if let Err(e) = stream.write_all(handshake.as_bytes()) {
            log::error!("[LAN] TCP handshake failed: {}", e);
            return Err(format!("TCP 握手失败: {}", e));
        }

        // Send as MESSAGE type: MESSAGE <msg_id> <size>\n<content>
        let header = format!("MESSAGE {} {}\n", msg_id, content.len());
        if let Err(e) = stream.write_all(header.as_bytes()) {
            log::error!("[LAN] TCP header failed: {}", e);
            return Err(format!("TCP 发送失败: {}", e));
        }

        if let Err(e) = stream.write_all(content.as_bytes()) {
            log::error!("[LAN] TCP content failed: {}", e);
            return Err(format!("TCP 发送失败: {}", e));
        }

        log::info!("[LAN] TCP message sent: {} bytes to {}", content.len(), addr);
        Ok(true)
    }

    pub fn get_all_peers(&self) -> Vec<Peer> {
        let mut result: Vec<Peer> = self.peers
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect();

        // Include peers from DB that aren't in the in-memory HashMap
        if let Ok(conn) = self.db_conn.lock() {
            if let Ok(db_users) = lan::get_all_users(&conn) {
                let in_memory_ids: std::collections::HashSet<String> =
                    result.iter().map(|p| p.id.clone()).collect();
                for user in db_users {
                    if !in_memory_ids.contains(&user.id) {
                        result.push(Peer {
                            id: user.id,
                            name: user.name,
                            avatar: None,
                            address: user.ip,
                            message_port: user.port as u16,
                            version: None,
                            last_seen: 0,
                            online: false,
                            status: None,
                        });
                    }
                }
            }
        }

        result
    }

    pub fn get_logs(&self, limit: usize) -> Vec<LanLogEntry> {
        let buf = self.log_buffer.lock().unwrap();
        buf.iter().rev().take(limit).cloned().collect()
    }

    pub fn get_local_ip(&self) -> String {
        self.local_ip.lock().unwrap().clone()
    }

    pub fn get_user_id(&self) -> String {
        self.user_id.clone()
    }

    pub fn get_db_conn(&self) -> &Arc<Mutex<Connection>> {
        &self.db_conn
    }

    pub fn get_user_info(&self) -> serde_json::Value {
        let nick = self.nick_name.lock().unwrap().clone();
        let avatar = self.avatar.lock().unwrap().clone();
        serde_json::json!({
            "id": self.user_id,
            "name": if nick.is_empty() { &self.user_name } else { &nick },
            "userName": if nick.is_empty() { &self.user_name } else { &nick },
            "avatar": avatar,
        })
    }

    pub fn set_nickname(&self, name: String) {
        *self.nick_name.lock().unwrap() = name.clone();
        if let Ok(conn) = self.db_conn.lock() {
            let _ = lan::save_lan_setting(&conn, &format!("nick_name:{}", self.user_id), &name);
        }
    }

    pub fn set_avatar(&self, avatar: String) {
        *self.avatar.lock().unwrap() = avatar.clone();
        if let Ok(conn) = self.db_conn.lock() {
            let _ = lan::save_lan_setting(&conn, &format!("avatar:{}", self.user_id), &avatar);
        }

        // 如果是图片头像，广播给其他用户
        if avatar.starts_with("avatar:") {
            if let Err(e) = self.broadcast_avatar_update(&avatar) {
                log::warn!("[LAN] Failed to broadcast avatar update: {}", e);
            }
        }
    }

    /// 广播头像更新到所有在线 peer（包含 base64 图片数据）
    pub fn broadcast_avatar_update(&self, avatar_ref: &str) -> Result<usize, String> {
        if !self.is_running.load(Ordering::SeqCst) {
            return Err("LAN service not running".to_string());
        }

        // 解析 avatar:filename 格式，读取图片文件
        let filename = avatar_ref
            .strip_prefix("avatar:")
            .ok_or("Invalid avatar format")?;
        let data_dir = supertool_core::logic::data_dir::resolve_data_dir();
        let avatar_path = data_dir.join("avatars").join(filename);

        // 读取图片文件并转为 base64
        let image_data = fs::read(&avatar_path).map_err(|e| format!("读取头像图片失败: {}", e))?;
        let base64_data = BASE64.encode(&image_data);

        // 获取图片扩展名用于接收方保存
        let ext = avatar_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png");

        let nick = self.nick_name.lock().unwrap().clone();
        let msg = serde_json::json!({
            "type": "avatar_update",
            "from": self.user_id,
            "fromName": if nick.is_empty() { &self.user_id } else { &nick },
            "avatar": avatar_ref,
            "avatarData": base64_data,
            "avatarExt": ext,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        });

        let data_str = serde_json::to_string(&msg).map_err(|e| format!("序列化失败: {}", e))?;

        // 广播给所有在线 peer
        let peers = self.peers.lock().unwrap();
        let mut sent_count = 0;
        if let Some(udp) = self.udp_socket.lock().unwrap().as_ref() {
            for peer in peers.values() {
                if peer.online {
                    let _ = udp.send_to(
                        data_str.as_bytes(),
                        format!("{}:{}", peer.address, peer.message_port),
                    );
                    sent_count += 1;
                }
            }
        }

        log::info!("[LAN] Broadcasted avatar update to {} peers", sent_count);
        Ok(sent_count)
    }

    pub fn set_status(&self, status: String) {
        *self.my_status.lock().unwrap() = status;
    }

    pub fn get_status(&self) -> String {
        self.my_status.lock().unwrap().clone()
    }

    pub fn refresh_discovery(&self) {
        if !self.is_running.load(Ordering::SeqCst) {
            return;
        }
        self.add_log("info", "Refreshing LAN discovery...");

        // Clear offline peers and send a broadcast to re-discover
        if let Some(udp) = self.udp_socket.lock().unwrap().as_ref() {
            let hb = serde_json::json!({
                "type": "heartbeat",
                "userId": self.user_id,
                "userName": self.user_name,
                "avatar": self.avatar.lock().unwrap().clone(),
                "status": self.my_status.lock().unwrap().clone(),
                "version": self.version,
                "messagePort": DISCOVERY_PORT,
            });
            if let Ok(msg) = serde_json::to_string(&hb) {
                let broadcast_addr =
                    SocketAddr::new(IpAddr::V4(Ipv4Addr::BROADCAST), DISCOVERY_PORT);
                let mc_addr =
                    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(239, 255, 0, 1)), DISCOVERY_PORT);
                match udp.send_to(msg.as_bytes(), broadcast_addr) {
                    Ok(n) => log::info!("[LAN] refresh_discovery sent {} bytes -> {}", n, broadcast_addr),
                    Err(e) => log::warn!("[LAN] refresh_discovery broadcast FAILED: {}", e),
                }
                let _ = udp.send_to(msg.as_bytes(), mc_addr);
            }
        }
    }

    pub fn get_receive_path(&self) -> String {
        self.receive_path.lock().unwrap().clone()
    }

    pub fn set_receive_path(&self, path: String) {
        let _ = fs::create_dir_all(&path);
        *self.receive_path.lock().unwrap() = path;
    }

    /// Send a file to a peer via TCP.
    /// 1. Notify peer via UDP (file_start)
    /// 2. Connect to peer's TCP port and send file data
    /// 3. Persist to DB
    pub fn send_file(
        &self,
        peer_id: &str,
        file_path: &str,
        file_name: &str,
        resume_offset: u64,
        file_id: Option<String>,
    ) -> Result<String, String> {
        if !self.is_running.load(Ordering::SeqCst) {
            return Err("LAN service not running".to_string());
        }

        let peers = self.peers.lock().unwrap();
        let peer = peers.get(peer_id).ok_or("Peer not found")?;

        let file_size = fs::metadata(file_path)
            .map(|m| m.len())
            .map_err(|e| format!("读取文件失败: {}", e))?;

        // Use UUID for unique file_id to avoid timestamp collision
        let file_id =
            file_id.unwrap_or_else(|| format!("file-{}", uuid::Uuid::new_v4().to_string()));

        let nick = self.nick_name.lock().unwrap().clone();
        let transfer = FileTransfer {
            id: file_id.clone(),
            from_user_id: self.user_id.clone(),
            from_user_name: nick.clone(),
            to_user_id: peer_id.to_string(),
            to_user_name: peer.name.clone(),
            file_name: file_name.to_string(),
            file_size,
            file_path: Some(file_path.to_string()),
            status: "sending".to_string(),
            progress: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
        };
        self.file_transfers
            .lock()
            .unwrap()
            .insert(file_id.clone(), transfer);

        // Persist to file_transfers DB
        let db_ft = DbFileTransfer {
            id: file_id.clone(),
            from_user_id: self.user_id.clone(),
            from_user_name: nick.clone(),
            to_user_id: peer_id.to_string(),
            to_user_name: peer.name.clone(),
            file_name: file_name.to_string(),
            file_size: file_size as i64,
            file_path: Some(file_path.to_string()),
            status: "sending".to_string(),
            progress: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
            local_user_id: Some(self.user_id.clone()),
        };
        if let Ok(conn) = self.db_conn.lock() {
            let _ = lan::insert_file_transfer(&conn, &db_ft);
        }

        // Also write a chat_message for this file — content stores JSON metadata like Electron
        let content_json = serde_json::json!({
            "fileName": file_name,
            "fileSize": file_size,
            "filePath": file_path,
            "isImage": Self::is_image_file(file_name),
        })
        .to_string();
        let chat_msg = ChatMessage {
            id: file_id.clone(), // Must match file_transfers.id for LEFT JOIN
            from_user_id: self.user_id.clone(),
            from_user_name: nick.clone(),
            to_user_id: peer_id.to_string(),
            to_user_name: peer.name.clone(),
            content: Some(content_json),
            msg_type: "file".to_string(),
            file_name: Some(file_name.to_string()),
            file_size: Some(file_size as i64),
            file_path: Some(file_path.to_string()),
            status: "sending".to_string(),
            progress: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
            read: false,
        };
        if let Ok(conn) = self.db_conn.lock() {
            let _ = lan::insert_chat_message(&conn, &chat_msg);
        }

        // Emit file transfer start event
        self.emit_event(
            "lan-file-transfer-started",
            &serde_json::json!({
                "fileId": file_id,
                "fileName": file_name,
                "fileSize": file_size,
                "fromUserId": self.user_id,
                "fromUserName": nick,
                "toUserId": peer_id,
                "toUserName": peer.name,
                "status": "sending",
                "progress": 0,
            }),
        );

        // Notify peer via UDP about incoming file
        let file_start = serde_json::json!({
            "type": "file_start",
            "id": file_id,
            "fromUserId": self.user_id,
            "fromUserName": nick,
            "fileId": file_id,
            "from": self.user_id,
            "fromName": nick,
            "fileName": file_name,
            "fileSize": file_size,
            "isImage": Self::is_image_file(file_name),
            "status": "sending",
            "progress": 0,
            "udxPort": *self.tcp_port.lock().unwrap(),
            "udxStreamId": 0,
            "tcpPort": *self.tcp_port.lock().unwrap(),
        });

        if let Ok(msg) = serde_json::to_string(&file_start) {
            if let Some(udp) = self.udp_socket.lock().unwrap().as_ref() {
                let _ = udp.send_to(
                    msg.as_bytes(),
                    format!("{}:{}", peer.address, peer.message_port),
                );
            }
        }

        // Start TCP file transfer in background thread
        let peer_addr = peer.address.clone();
        let peer_tcp_port = peer.message_port;
        let my_id = self.user_id.clone();
        let fp = file_path.to_string();
        let fn_ = file_name.to_string();
        let fid = file_id.clone();
        let ro = resume_offset;
        let transfers = Arc::clone(&self.file_transfers);
        let log = Arc::clone(&self.log_buffer);
        let send_app_handle = self.app_handle.lock().unwrap().clone();
        let db_conn = Arc::clone(&self.db_conn);

        thread::spawn(move || {
            log::info!("[LAN TCP] TCP send thread started: {} -> {}:{}", fn_, peer_addr, FILE_TRANSFER_PORT);
            if let Err(e) = Self::do_send_file(
                &peer_addr,
                peer_tcp_port,
                &my_id,
                &fp,
                &fn_,
                &fid,
                ro,
                &transfers,
                &log,
                &send_app_handle,
                &db_conn,
            ) {
                log::error!("[LAN TCP] File send failed: {}", e);
                Self::add_log_static(&log, "error", &format!("File send failed: {}", e));
                {
                    let mut tf = transfers.lock().unwrap();
                    if let Some(t) = tf.get_mut(&fid) {
                        t.status = "error".to_string();
                    }
                }
                if let Ok(conn) = db_conn.lock() {
                    let _ = lan::update_file_transfer(&conn, &fid, "error", 0, None, None);
                }
                if let Some(app) = send_app_handle {
                    let _ = app.emit(
                        "lan-file-transfer-error",
                        serde_json::json!({
                            "fileId": fid,
                            "error": e,
                        }),
                    );
                }
            }
        });

        Ok(file_id)
    }

    /// Actually send a file over TCP to the peer
    fn do_send_file(
        peer_addr: &str,
        _peer_tcp_port: u16,
        sender_id: &str,
        file_path: &str,
        file_name: &str,
        file_id: &str,
        resume_offset: u64,
        transfers: &Arc<Mutex<HashMap<String, FileTransfer>>>,
        log: &Arc<Mutex<Vec<LanLogEntry>>>,
        app_handle: &Option<tauri::AppHandle>,
        db_conn: &Arc<Mutex<Connection>>,
    ) -> Result<(), String> {
        let file_size = fs::metadata(file_path)
            .map(|m| m.len())
            .map_err(|e| format!("获取文件大小失败: {}", e))?;

        // Wait briefly for the peer to receive the file_start notification
        thread::sleep(Duration::from_millis(300));

        // Connect to peer's TCP file transfer port with 10-second timeout
        log::info!(
            "[LAN TCP] Connecting to {}:{} for file transfer (file_size={})",
            peer_addr, FILE_TRANSFER_PORT, file_size
        );
        let mut stream = TcpStream::connect_timeout(
            &format!("{}:{}", peer_addr, FILE_TRANSFER_PORT)
                .parse()
                .map_err(|e| format!("解析地址失败: {}", e))?,
            Duration::from_secs(10),
        )
        .map_err(|e| format!("TCP 连接超时: {}", e))?;

        // Send sender identity handshake: LAN-SEND <user_id>\n
        stream
            .write_all(format!("LAN-SEND {}\n", sender_id).as_bytes())
            .map_err(|e| format!("发送身份握手失败: {}", e))?;

        // Send header: FILE <name> <size> <id>\n
        let header = format!("FILE {} {} {}\n", file_name, file_size, file_id);
        stream
            .write_all(header.as_bytes())
            .map_err(|e| format!("发送文件头失败: {}", e))?;

        // Open file and seek to resume_offset for resume support
        let mut file = fs::File::open(file_path).map_err(|e| format!("打开文件失败: {}", e))?;
        if resume_offset > 0 {
            use std::io::Seek;
            file.seek(std::io::SeekFrom::Start(resume_offset))
                .map_err(|e| format!("seek 失败: {}", e))?;
        }

        let mut buf = [0u8; 64 * 1024];
        let mut sent = resume_offset;
        let mut last_emit_pct = 0i64;
        loop {
            let n = file
                .read(&mut buf)
                .map_err(|e| format!("读取失败: {}", e))?;
            if n == 0 {
                break;
            }
            stream
                .write_all(&buf[..n])
                .map_err(|e| format!("写入TCP失败: {}", e))?;
            sent += n as u64;

            let progress_pct = if file_size > 0 {
                ((sent as f64 / file_size as f64) * 100.0) as i64
            } else {
                100
            };
            if progress_pct >= last_emit_pct + 10 {
                last_emit_pct = (progress_pct / 10) * 10;
                {
                    let mut tf = transfers.lock().unwrap();
                    if let Some(t) = tf.get_mut(file_id) {
                        t.progress = progress_pct;
                    }
                }
                if let Ok(conn) = db_conn.lock() {
                    let _ = lan::update_file_transfer(
                        &conn,
                        file_id,
                        "sending",
                        progress_pct,
                        None,
                        None,
                    );
                }
                if let Some(app) = app_handle {
                    let _ = app.emit(
                        "lan-file-transfer-progress",
                        serde_json::json!({
                            "fileId": file_id,
                            "status": "sending",
                            "progress": progress_pct,
                            "sent": sent,
                            "total": file_size,
                        }),
                    );
                }
            }
        }

        // Flush and shutdown write side
        stream.flush().ok();
        stream.shutdown(std::net::Shutdown::Write).ok();

        if let Some(t) = transfers.lock().unwrap().get_mut(file_id) {
            t.status = "completed".to_string();
            t.progress = 100;
            t.completed_at = Some(chrono::Utc::now().to_rfc3339());
        }

        if let Ok(conn) = db_conn.lock() {
            let completed_at = chrono::Utc::now().to_rfc3339();
            let _ = lan::update_file_transfer(
                &conn,
                file_id,
                "completed",
                100,
                Some(file_path),
                Some(&completed_at),
            );
            // Update chat_message for this file — update content JSON with status
            let content_json = serde_json::json!({
                "fileName": file_name,
                "fileSize": file_size,
                "filePath": file_path,
                "status": "completed",
                "isImage": Self::is_image_file(file_name),
            })
            .to_string();
            let _ = conn.execute(
                "UPDATE chat_messages SET content = ?1, status = 'completed', progress = 100 WHERE id = ?2",
                rusqlite::params![content_json, file_id],
            );
        }

        Self::add_log_static(
            log,
            "info",
            &format!("File sent: {} ({} bytes)", file_name, sent),
        );
        log::info!("[LAN TCP] File sent successfully: {} ({} bytes)", file_name, sent);

        // Get from/to user info from transfers map
        let (from_user_id, from_user_name, to_user_id, to_user_name) = {
            let tf = transfers.lock().unwrap();
            if let Some(t) = tf.get(file_id) {
                (
                    t.from_user_id.clone(),
                    t.from_user_name.clone(),
                    t.to_user_id.clone(),
                    t.to_user_name.clone(),
                )
            } else {
                (
                    "unknown".to_string(),
                    "unknown".to_string(),
                    "unknown".to_string(),
                    "unknown".to_string(),
                )
            }
        };

        if let Some(app) = app_handle {
            log::info!(
                "[LAN TCP] Emitting lan-file-transfer-completed: fileId={}, status={}",
                file_id, "completed"
            );
            let _ = app.emit(
                "lan-file-transfer-completed",
                serde_json::json!({
                    "fileId": file_id,
                    "fromUserId": from_user_id,
                    "fromUserName": from_user_name,
                    "toUserId": to_user_id,
                    "toUserName": to_user_name,
                    "fileName": file_name,
                    "fileSize": file_size,
                    "filePath": file_path,
                    "status": "completed",
                    "progress": 100,
                    "sent": sent,
                    "completedAt": chrono::Utc::now().to_rfc3339(),
                    "isImage": Self::is_image_file(file_name),
                }),
            );
        }

        Ok(())
    }

    pub fn get_file_transfer_history(&self, limit: usize, _offset: usize) -> Vec<FileTransfer> {
        let transfers = self.file_transfers.lock().unwrap();
        let mut list: Vec<_> = transfers.values().cloned().collect();
        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        list.into_iter().take(limit).collect()
    }

    pub fn get_message_history(&self, limit: usize) -> Vec<LanMessage> {
        let history = self.message_history.lock().unwrap();
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Screenshot — macOS: screencapture, Linux: scrot or gnome-screenshot
    pub fn screenshot(&self) -> Result<String, String> {
        let screenshot_path = "/tmp/lan_screenshot.png";
        // Try different commands
        let cmds: &[(&str, &[&str])] = &[
            ("screencapture", &["-x", screenshot_path]),    // macOS
            ("gnome-screenshot", &["-f", screenshot_path]), // GNOME
            ("scrot", &[screenshot_path]),                  // scrot
            ("import", &["-window", "root", screenshot_path]), // ImageMagick
        ];

        for (cmd, args) in cmds {
            match std::process::Command::new(cmd).args(*args).output() {
                Ok(out) if out.status.success() => {
                    if std::path::Path::new(screenshot_path).exists() {
                        return Ok(screenshot_path.to_string());
                    }
                }
                _ => continue,
            }
        }

        Err("截图失败：未找到可用的截图工具".to_string())
    }

    /// Decode base64 data and write to ~/.supertool/lan_temp/
    pub fn save_temp_file(&self, base64_data: &str, file_name: &str) -> Result<String, String> {
        let temp_dir = supertool_core::logic::data_dir::lan_temp_dir();
        fs::create_dir_all(&temp_dir).map_err(|e| format!("创建临时目录失败: {}", e))?;

        let file_path = temp_dir.join(file_name);
        let decoded = BASE64
            .decode(base64_data)
            .map_err(|e| format!("Base64 解码失败: {}", e))?;
        fs::write(&file_path, decoded).map_err(|e| format!("写入文件失败: {}", e))?;

        Ok(file_path.to_string_lossy().to_string())
    }

    /// Read file and encode as base64
    pub fn load_file_as_base64(&self, file_path: &str) -> Result<String, String> {
        let data = fs::read(file_path).map_err(|e| format!("读取文件失败: {}", e))?;
        Ok(BASE64.encode(&data))
    }

    // ========== Collaboration broadcast methods ==========

    /// Send task assignment to a specific peer
    pub fn assign_task(
        &self,
        peer_id: &str,
        task_data: &serde_json::Value,
    ) -> Result<bool, String> {
        self.send_collab_message(peer_id, "assign_task", task_data)
    }

    /// Broadcast task update to all online peers
    pub fn broadcast_task_update(&self, task_data: &serde_json::Value) -> Result<usize, String> {
        self.broadcast_collab_message("task_update", task_data)
    }

    /// Broadcast task status change to all online peers
    pub fn broadcast_task_status_change(
        &self,
        task_data: &serde_json::Value,
    ) -> Result<usize, String> {
        self.broadcast_collab_message("task_status_change", task_data)
    }

    /// Send task comment to a specific peer
    pub fn broadcast_task_comment(
        &self,
        peer_id: &str,
        comment_data: &serde_json::Value,
    ) -> Result<bool, String> {
        self.send_collab_message(peer_id, "task_comment", comment_data)
    }

    /// Broadcast collaboration started to all online peers
    pub fn broadcast_collaboration_started(
        &self,
        collab_data: &serde_json::Value,
    ) -> Result<usize, String> {
        self.broadcast_collab_message("collaboration_started", collab_data)
    }

    /// Broadcast collaboration ended to all online peers
    pub fn broadcast_collaboration_ended(
        &self,
        collab_data: &serde_json::Value,
    ) -> Result<usize, String> {
        self.broadcast_collab_message("collaboration_ended", collab_data)
    }

    /// Send collaboration message to a specific peer
    fn send_collab_message(
        &self,
        peer_id: &str,
        msg_type: &str,
        data: &serde_json::Value,
    ) -> Result<bool, String> {
        if !self.is_running.load(Ordering::SeqCst) {
            return Err("LAN service not running".to_string());
        }
        let peers = self.peers.lock().unwrap();
        let peer = peers.get(peer_id).ok_or("Peer not found")?;

        // Generate unique message ID
        let msg_id = format!("msg-{}", uuid::Uuid::new_v4());
        let nick = self.nick_name.lock().unwrap().clone();

        // Payload structure matches frontend expectation:
        // { messageId, from, to, fromName, toName, task (for assign_task), timestamp }
        let msg = if msg_type == "assign_task" {
            serde_json::json!({
                "type": msg_type,
                "messageId": msg_id,
                "id": msg_id,
                "from": self.user_id,
                "to": peer_id,
                "fromName": nick,
                "toName": peer.name.clone(),
                "task": data,
                "timestamp": chrono::Utc::now().timestamp_millis(),
            })
        } else {
            serde_json::json!({
                "type": msg_type,
                "messageId": msg_id,
                "id": msg_id,
                "from": self.user_id,
                "to": peer_id,
                "fromName": nick,
                "toName": peer.name.clone(),
                "data": data,
                "timestamp": chrono::Utc::now().timestamp_millis(),
            })
        };

        if let Ok(data_str) = serde_json::to_string(&msg) {
            if let Some(udp) = self.udp_socket.lock().unwrap().as_ref() {
                let _ = udp.send_to(
                    data_str.as_bytes(),
                    format!("{}:{}", peer.address, peer.message_port),
                );
                return Ok(true);
            }
        }
        Err("发送失败".to_string())
    }

    /// Broadcast collaboration message to all online peers, returns sent count
    fn broadcast_collab_message(
        &self,
        msg_type: &str,
        data: &serde_json::Value,
    ) -> Result<usize, String> {
        if !self.is_running.load(Ordering::SeqCst) {
            return Err("LAN service not running".to_string());
        }

        let msg = serde_json::json!({
            "type": msg_type,
            "from": self.user_id,
            "fromName": self.nick_name.lock().unwrap().clone(),
            "data": data,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        });

        let data_str = serde_json::to_string(&msg).map_err(|e| format!("序列化失败: {}", e))?;

        let peers = self.peers.lock().unwrap();
        let mut sent_count = 0;
        if let Some(udp) = self.udp_socket.lock().unwrap().as_ref() {
            for peer in peers.values() {
                if peer.online {
                    let _ = udp.send_to(
                        data_str.as_bytes(),
                        format!("{}:{}", peer.address, peer.message_port),
                    );
                    sent_count += 1;
                }
            }
        }
        Ok(sent_count)
    }

    /// Broadcast a text message to all online peers
    pub fn broadcast_message(&self, content: &str) -> Result<usize, String> {
        if !self.is_running.load(Ordering::SeqCst) {
            return Err("LAN service not running".to_string());
        }

        let nick = self.nick_name.lock().unwrap().clone();
        let msg = serde_json::json!({
            "type": "message",
            "from": self.user_id,
            "fromName": if nick.is_empty() { &self.user_id } else { &nick },
            "content": content,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        });

        let data_str = serde_json::to_string(&msg).map_err(|e| format!("序列化失败: {}", e))?;

        let peers = self.peers.lock().unwrap();
        let mut sent_count = 0;
        if let Some(udp) = self.udp_socket.lock().unwrap().as_ref() {
            for peer in peers.values() {
                if peer.online {
                    let _ = udp.send_to(
                        data_str.as_bytes(),
                        format!("{}:{}", peer.address, peer.message_port),
                    );
                    sent_count += 1;
                }
            }
        }
        Ok(sent_count)
    }
}
