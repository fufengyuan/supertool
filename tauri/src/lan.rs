/// LAN 协作服务 — UDP 广播发现 + TCP 可靠文件传输 + SQLite 持久化
///
/// - UDP: std::net::UdpSocket 广播发现/心跳/消息
/// - 文件传输: std::net::Tcp 可靠传输
/// - 消息/文件记录: SQLite (db/lan.rs)
use supertool_core::db::lan::{self, ChatMessage, FileTransfer as DbFileTransfer};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream, UdpSocket};
use if_addrs::get_if_addrs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
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
        Self {
            user_id: user_id.clone(),
            user_name,
            nick_name: Mutex::new(String::new()),
            avatar: Mutex::new("😀".to_string()),
            my_status: Mutex::new("online".to_string()),
            version: "2.0".to_string(),
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
        eprintln!("[LAN] Starting LAN service...");

        // Detect local IP
        let local_ip = Self::detect_local_ip().unwrap_or_else(|| "unknown".to_string());
        *self.local_ip.lock().unwrap() = local_ip.clone();
        eprintln!("[LAN] Detected local IP: {}", local_ip);
        self.add_log("info", &format!("Local IP: {}", local_ip));
        
        // Debug: log broadcast addresses
        let broadcast_addrs = Self::get_broadcast_addrs();
        eprintln!("[LAN] Broadcast addresses: {:?}", broadcast_addrs);
        self.add_log("info", &format!("Broadcast addresses: {:?}", broadcast_addrs));

        // Setup receive path
        let receive_path = supertool_core::logic::data_dir::received_files_dir()
            .to_string_lossy()
            .to_string();
        *self.receive_path.lock().unwrap() = receive_path.clone();
        fs::create_dir_all(&receive_path).map_err(|e| format!("创建接收目录失败: {}", e))?;

        // Start UDP discovery socket with SO_REUSEADDR + SO_REUSEPORT
        // Electron uses dgram.createSocket({ reuseAddr: true }) — this is the Rust equivalent
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), DISCOVERY_PORT);
        let sock = socket2::Socket::new(
            socket2::Domain::IPV4,
            socket2::Type::DGRAM,
            Some(socket2::Protocol::UDP),
        ).map_err(|e| format!("创建 UDP socket 失败: {}", e))?;
        sock.set_reuse_address(true).map_err(|e| format!("set_reuse_address 失败: {}", e))?;
        sock.set_broadcast(true).map_err(|e| format!("set_broadcast 失败: {}", e))?;
        sock.bind(&addr.into()).map_err(|e| format!("UDP 绑定失败: {}", e))?;
        sock.set_nonblocking(true).map_err(|e| format!("set_nonblocking 失败: {}", e))?;
        let udp: UdpSocket = sock.into();
        let udp = Arc::new(udp);
        *self.udp_socket.lock().unwrap() = Some(Arc::clone(&udp));
        self.add_log("info", &format!("UDP discovery on port {}", DISCOVERY_PORT));

        let tcp_port = *self.tcp_port.lock().unwrap();
        self.add_log("info", &format!("TCP file transfer on port {}", tcp_port));

        self.is_running.store(true, Ordering::SeqCst);
        self.stop_flag.store(false, Ordering::SeqCst);

        // ===== UDP receive thread =====
        let peers = Arc::clone(&self.peers);
        let log = Arc::clone(&self.log_buffer);
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
        thread::spawn(move || {
            let mut buf = [0u8; 65536];
            let mut recv_count = 0u64;
            while !stop.load(Ordering::SeqCst) {
                match recv_udp.recv_from(&mut buf) {
                    Ok((len, addr)) => {
                        recv_count += 1;
                        if recv_count <= 5 || recv_count % 50 == 0 {
                            Self::add_log_static(&log, "info", &format!(
                                "[UDP RECV] #{} from {}:{} len={} preview={}",
                                recv_count,
                                addr.ip(),
                                addr.port(),
                                len,
                                String::from_utf8_lossy(&buf[..len.min(120)]).chars().take(80).collect::<String>()
                            ));
                        }
                        if addr.ip().is_loopback() {
                            continue;
                        }
                        if let Ok(text) = std::str::from_utf8(&buf[..len]) {
                            if let Ok(data) = serde_json::from_str::<serde_json::Value>(text) {
                                Self::handle_udp_message(
                                    &data, &addr, &user_id, &nick_name, &avatar, &my_status,
                                    &version, &peers, &log, &msg_history, &file_transfers,
                                    &receive_path, tcp_p, &recv_app_handle, &db_conn,
                                );
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(100));
                    }
                    Err(_) => thread::sleep(Duration::from_millis(100)),
                }
            }
        });

        // ===== Heartbeat thread =====
        {
            let heartbeat_stop = Arc::clone(&self.stop_flag);
            let heartbeat_udp = Arc::clone(&udp);
            let heartbeat_peers = Arc::clone(&self.peers);
            let heartbeat_log = Arc::clone(&self.log_buffer);
            let heartbeat_user_id = self.user_id.clone();
            let heartbeat_nick = self.nick_name.lock().unwrap().clone();
            let heartbeat_avatar = self.avatar.lock().unwrap().clone();
            let heartbeat_status = self.my_status.lock().unwrap().clone();
            let heartbeat_version = self.version.clone();
            let heartbeat_app = self.app_handle.lock().unwrap().clone();

            thread::spawn(move || {
                while !heartbeat_stop.load(Ordering::SeqCst) {
                    thread::sleep(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));
                    let presence = serde_json::json!({
                        "type": "heartbeat",
                        "userId": heartbeat_user_id,
                        "name": if heartbeat_nick.is_empty() { &heartbeat_user_id } else { &heartbeat_nick },
                        "userName": if heartbeat_nick.is_empty() { &heartbeat_user_id } else { &heartbeat_nick },
                        "avatar": heartbeat_avatar,
                        "status": heartbeat_status,
                        "version": heartbeat_version,
                        "messagePort": DISCOVERY_PORT,
                        "timestamp": chrono::Utc::now().timestamp_millis(),
                    });
                    if let Ok(msg) = serde_json::to_string(&presence) {
                        let broadcast_addrs = Self::get_broadcast_addrs();
                        for addr in &broadcast_addrs {
                            let _ = heartbeat_udp.send_to(msg.as_bytes(), format!("{}:{}", addr, DISCOVERY_PORT));
                        }
                    }
                    Self::check_offline_peers(&heartbeat_peers, &heartbeat_log, &heartbeat_app);
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
                // Bind TCP listener inside the thread so we can accept in a loop
                let listener = match std::net::TcpListener::bind(format!("0.0.0.0:{}", tcp_port_val)) {
                    Ok(l) => l,
                    Err(e) => {
                        Self::add_log_static(&log, "error", &format!("TCP listen failed: {}", e));
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
                            Self::handle_tcp_connection(stream, &rp, &tf, &lg, &uid, &nick, &dc, &ah);
                        });
                    }
                }
            });
        }

        // Initial discovery burst
        self.broadcast_presence();
        thread::sleep(Duration::from_millis(500));
        self.broadcast_presence();
        thread::sleep(Duration::from_millis(1000));
        self.broadcast_presence();

        self.add_log("info", "LAN service started");
        Ok(())
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        self.stop_flag.store(true, Ordering::SeqCst);
        // Close UDP socket to unblock recv thread
        if let Some(udp) = self.udp_socket.lock().unwrap().take() {
            // Connect to localhost to break recv_from
            let _ = udp.connect("127.0.0.1:1");
            drop(udp);
        }
        self.peers.lock().unwrap().clear();
        self.add_log("info", "LAN service stopped");
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
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

        match msg_type {
            "heartbeat" | "discovery" => {
                let peer_id = data["userId"].as_str().unwrap_or("");
                if peer_id.is_empty() || peer_id == my_user_id {
                    return;
                }

                let peer_name = data["userName"].as_str()
                    .or_else(|| data["name"].as_str())
                    .unwrap_or(peer_id).to_string();
                let peer_avatar = data["avatar"].as_str().map(|s| s.to_string());
                let peer_version = data["version"].as_str().map(|s| s.to_string());
                let peer_status = data["status"].as_str().map(|s| s.to_string());
                // Use messagePort from packet if available, otherwise use sender's port
                let message_port = data["messagePort"].as_u64()
                    .map(|p| p as u16)
                    .unwrap_or(addr.port());

                // Version compatibility check
                if let Some(ref v) = peer_version {
                    if let Some(major) = v.split('.').next() {
                        if let Some(my_major) = my_version.split('.').next() {
                            if major != my_major {
                                return;
                            }
                        }
                    }
                }

                let now = chrono::Utc::now().timestamp_millis();
                let mut peers_map = peers.lock().unwrap();
                let is_new = !peers_map.contains_key(peer_id);

                let peer = Peer {
                    id: peer_id.to_string(),
                    name: peer_name.clone(),
                    avatar: peer_avatar,
                    address: addr.ip().to_string(),
                    message_port,
                    version: peer_version,
                    last_seen: now,
                    online: true,
                    status: peer_status,
                };
                peers_map.insert(peer_id.to_string(), peer);

                if is_new {
                    Self::add_log_static(log, "info", &format!("Peer discovered: {} ({})", peer_id, addr.ip()));
                    if let Some(app) = app_handle {
                        let payload = serde_json::json!({
                            "peerId": peer_id,
                            "address": addr.ip().to_string(),
                            "name": peer_name,
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
                            let _ = reply_sock.send_to(msg.as_bytes(), format!("{}:{}", addr.ip(), addr.port()));
                        }
                    }
                }
            }
            "message" => {
                if let Ok(msg) = serde_json::from_value::<LanMessage>(data.clone()) {
                    msg_history.lock().unwrap().push(msg.clone());

                    // Persist to chat_messages table
                    let my_nick_str = if my_nick.is_empty() { my_user_id.to_string() } else { my_nick.to_string() };
                    let chat_msg = ChatMessage {
                        id: msg.message_id.clone().unwrap_or_else(|| format!("msg-{}", chrono::Utc::now().timestamp_millis())),
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

                    Self::add_log_static(log, "info", &format!("Message from {}: {}",
                        msg.from.as_deref().unwrap_or("unknown"),
                        msg.content.as_deref().unwrap_or("")));
                    if let Some(app) = app_handle {
                        let _ = app.emit("lan-message-received", data.clone());
                    }
                }
            }
            "file_start" => {
                let file_id = data["id"].as_str()
                    .or_else(|| data["fileId"].as_str());
                let file_name = data["fileName"].as_str();
                let file_size = data["fileSize"].as_u64();
                let from_id = data["fromUserId"].as_str()
                    .or_else(|| data["from"].as_str());
                let from_name = data["fromUserName"].as_str()
                    .or_else(|| data["fromName"].as_str());
                if let (Some(file_id), Some(file_name), Some(file_size), Some(from_id), Some(from_name)) =
                    (file_id, file_name, file_size, from_id, from_name) {
                    let to_name = if my_nick.is_empty() { my_user_id.to_string() } else { my_nick.to_string() };
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
                    file_transfers.lock().unwrap().insert(file_id.to_string(), transfer);

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
                        }).to_string();
                        let chat_msg = ChatMessage {
                            id: file_id.to_string(),
                            from_user_id: from_id.to_string(),
                            from_user_name: from_name.to_string(),
                            to_user_id: my_user_id.to_string(),
                            to_user_name: if my_nick.is_empty() { my_user_id.to_string() } else { my_nick.to_string() },
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

                    Self::add_log_static(log, "info", &format!("File transfer started: {} ({} bytes)", file_name, file_size));
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
                Self::add_log_static(log, "info", &format!("file_start_ack received for file: {}", file_id));
            }
            // Collaboration message types — forward to frontend
            // Map msg_type to hyphen-format event names matching frontend listeners
            "assign_task" | "task_update" | "task_status_change" | "task_comment"
            | "collaboration_started" | "collaboration_ended" => {
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
        mut stream: TcpStream,
        receive_path: &str,
        transfers: &Arc<Mutex<HashMap<String, FileTransfer>>>,
        log: &Arc<Mutex<Vec<LanLogEntry>>>,
        my_user_id: &str,
        my_nick: &str,
        db_conn: &Arc<Mutex<Connection>>,
        app_handle: &Option<tauri::AppHandle>,
    ) {
        // Read file metadata header: FILE <name> <size> <id>\n
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut header = String::new();
        if reader.read_line(&mut header).is_err() {
            return;
        }

        let parts: Vec<&str> = header.trim().splitn(4, ' ').collect();
        if parts.len() < 4 || parts[0] != "FILE" {
            Self::add_log_static(log, "error", &format!("Invalid TCP header: {}", header));
            return;
        }

        const MAX_FILE_SIZE: u64 = 500 * 1024 * 1024;
        let raw_file_name = parts[1];
        let file_size: u64 = parts[2].parse().unwrap_or(0);
        let file_id = parts[3];
        // Get from_user info from transfers map if available
        let (from_id, from_name) = {
            let tf = transfers.lock().unwrap();
            if let Some(t) = tf.get(&file_id.to_string()) {
                (t.from_user_id.clone(), t.from_user_name.clone())
            } else {
                ("unknown".to_string(), "unknown".to_string())
            }
        };
        let my_nick_str = if my_nick.is_empty() { my_user_id.to_string() } else { my_nick.to_string() };
        if file_size > MAX_FILE_SIZE {
            Self::add_log_static(log, "error", &format!("File too large: {} bytes", file_size));
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
                let candidate = PathBuf::from(receive_path)
                    .join(format!("{}{}{}", stem, counter, ext));
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
            match stream.read(&mut buf) {
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
                                let _ = lan::update_file_transfer(&conn, file_id, "receiving", progress_pct, None, None);
                            }
                            if let Some(app) = app_handle {
                                let _ = app.emit("lan-file-transfer-progress", serde_json::json!({
                                    "fileId": file_id,
                                    "progress": progress_pct,
                                    "received": received,
                                    "total": file_size,
                                }));
                            }
                        }
                    }
                }
                Err(_) => break,
            }
        }

        let save_path_str = save_path.to_string_lossy().to_string();

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
            let _ = lan::update_file_transfer(&conn, file_id, "completed", 100, Some(&save_path_str), Some(&completed_at));
            // Also update the chat_message for this file — update content JSON with status+filePath
            let content_json = serde_json::json!({
                "fileName": file_name,
                "fileSize": file_size,
                "filePath": save_path_str,
                "status": "completed",
                "isImage": Self::is_image_file(&file_name),
            }).to_string();
            let _ = conn.execute(
                "UPDATE chat_messages SET content = ?1 WHERE id = ?2",
                rusqlite::params![content_json, file_id],
            );
        }

        Self::add_log_static(log, "info", &format!("File received: {} ({} bytes)", file_name, received));

        if let Some(app) = app_handle {
            let _ = app.emit("lan-file-transfer-completed", serde_json::json!({
                "fileId": file_id,
                "fileName": file_name,
                "fileSize": file_size,
                "received": received,
                "filePath": save_path_str,
            }));
            // Also emit lan-file-received for frontend compatibility
            let _ = app.emit("lan-file-received", serde_json::json!({
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
            }));
        }
    }

    fn check_offline_peers(
        peers: &Arc<Mutex<HashMap<String, Peer>>>,
        log: &Arc<Mutex<Vec<LanLogEntry>>>,
        app_handle: &Option<tauri::AppHandle>,
    ) {
        let now = chrono::Utc::now().timestamp_millis();
        let mut peers_map = peers.lock().unwrap();
        for peer in peers_map.values_mut() {
            if peer.online && (now - peer.last_seen) > (PEER_TIMEOUT_SECS as i64 * 1000) {
                peer.online = false;
                Self::add_log_static(log, "info", &format!("Peer offline: {}", peer.id));
                // Emit lan-peer-lost event for frontend
                if let Some(app) = app_handle {
                    let payload = serde_json::json!({
                        "id": peer.id,
                        "name": peer.name,
                        "address": peer.address,
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
        matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "svg")
    }

    fn add_log(&self, level: &str, message: &str) {
        Self::add_log_static(&self.log_buffer, level, message);
    }

    /// Calculate subnet broadcast addresses from all non-loopback, non-virtual IPv4 interfaces
    /// Uses if-addrs crate to get real netmask, matching Electron's os.networkInterfaces() behavior
    fn get_broadcast_addrs() -> Vec<String> {
        let mut addrs = Vec::new();
        // Always include global broadcast
        addrs.push("255.255.255.255".to_string());

        // Use if-addrs to enumerate interfaces with real netmask (matches Electron's os.networkInterfaces)
        if let Ok(interfaces) = get_if_addrs() {
            for iface in interfaces {
                // Skip loopback
                if iface.is_loopback() {
                    continue;
                }
                // Skip virtual/VPN interfaces: tun, tap, utun, docker, bridge, veth, virbr
                let name = iface.name.to_lowercase();
                if name.starts_with("tun") || name.starts_with("tap")
                    || name.starts_with("utun") || name.starts_with("docker")
                    || name.starts_with("br-") || name.starts_with("veth")
                    || name.starts_with("virbr")
                {
                    continue;
                }
                if let if_addrs::IfAddr::V4(v4) = iface.addr {
                    let ip = v4.ip.octets();
                    let mask = v4.netmask.octets();
                    // Calculate broadcast: ip | (~netmask)
                    let broadcast = format!(
                        "{}.{}.{}.{}",
                        ip[0] | (!mask[0] & 0xff),
                        ip[1] | (!mask[1] & 0xff),
                        ip[2] | (!mask[2] & 0xff),
                        ip[3] | (!mask[3] & 0xff),
                    );
                    addrs.push(broadcast);
                }
            }
        }

        // Deduplicate
        addrs.sort();
        addrs.dedup();
        addrs
    }

    fn detect_local_ip() -> Option<String> {
        // Use if-addrs to enumerate interfaces, filtering out virtual/VPN interfaces
        // This avoids picking up the VPN IP (e.g., tun0) when the default route goes through it
        if let Ok(interfaces) = get_if_addrs() {
            for iface in interfaces {
                if iface.is_loopback() {
                    continue;
                }
                let name = iface.name.to_lowercase();
                if name.starts_with("tun") || name.starts_with("tap")
                    || name.starts_with("utun") || name.starts_with("docker")
                    || name.starts_with("br-") || name.starts_with("veth")
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

    pub fn broadcast_presence(&self) {
        if !self.is_running.load(Ordering::SeqCst) {
            return;
        }
        let nick = self.nick_name.lock().unwrap().clone();
        let avatar = self.avatar.lock().unwrap().clone();
        let status = self.my_status.lock().unwrap().clone();

        let presence = serde_json::json!({
            "type": "discovery",
            "userId": self.user_id,
            "name": if nick.is_empty() { &self.user_id } else { &nick },
            "userName": if nick.is_empty() { &self.user_id } else { &nick },
            "avatar": avatar,
            "status": status,
            "version": self.version,
            "messagePort": DISCOVERY_PORT,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        });

        if let Ok(msg) = serde_json::to_string(&presence) {
            if let Some(udp) = self.udp_socket.lock().unwrap().as_ref() {
                for addr in Self::get_broadcast_addrs() {
                    let _ = udp.send_to(msg.as_bytes(), format!("{}:{}", addr, DISCOVERY_PORT));
                }
            }
        }
    }

    /// Send a text message to a peer. Also persists to DB.
    pub fn send_message(&self, peer_id: &str, content: &str) -> Result<bool, String> {
        if !self.is_running.load(Ordering::SeqCst) {
            return Err("LAN service not running".to_string());
        }

        let peers = self.peers.lock().unwrap();
        let peer = peers.get(peer_id).ok_or("Peer not found")?;

        let nick = self.nick_name.lock().unwrap().clone();
        let msg_id = format!("msg-{}", chrono::Utc::now().timestamp_millis());
        let msg = serde_json::json!({
            "type": "message",
            "from": self.user_id,
            "fromName": if nick.is_empty() { &self.user_id } else { &nick },
            "to": peer_id,
            "toName": peer.name.clone(),
            "content": content,
            "timestamp": chrono::Utc::now().timestamp_millis(),
            "messageId": &msg_id,
        });

        // Persist to chat_messages DB
        let to_name = peer.name.clone();
        let chat_msg = ChatMessage {
            id: msg_id.clone(),
            from_user_id: self.user_id.clone(),
            from_user_name: if nick.is_empty() { self.user_id.clone() } else { nick },
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

        if let Ok(data) = serde_json::to_string(&msg) {
            if let Some(udp) = self.udp_socket.lock().unwrap().as_ref() {
                let _ = udp.send_to(data.as_bytes(), format!("{}:{}", peer.address, peer.message_port));
                return Ok(true);
            }
        }
        Err("发送失败".to_string())
    }

    pub fn get_online_peers(&self) -> Vec<Peer> {
        self.peers.lock().unwrap()
            .values()
            .filter(|p| p.online)
            .cloned()
            .collect()
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
        *self.nick_name.lock().unwrap() = name;
        if self.is_running.load(Ordering::SeqCst) {
            self.broadcast_presence();
        }
    }

    pub fn set_avatar(&self, emoji: String) {
        *self.avatar.lock().unwrap() = emoji;
        if self.is_running.load(Ordering::SeqCst) {
            self.broadcast_presence();
        }
    }

    pub fn set_status(&self, status: String) {
        *self.my_status.lock().unwrap() = status;
        if self.is_running.load(Ordering::SeqCst) {
            self.broadcast_presence();
        }
    }

    pub fn get_status(&self) -> String {
        self.my_status.lock().unwrap().clone()
    }

    pub fn refresh_discovery(&self) {
        if !self.is_running.load(Ordering::SeqCst) {
            return;
        }
        self.add_log("info", "Manual discovery refresh");
        self.broadcast_presence();
        thread::sleep(Duration::from_millis(500));
        self.broadcast_presence();
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
        let file_id = file_id.unwrap_or_else(|| format!("file-{}", uuid::Uuid::new_v4().to_string()));

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
        self.file_transfers.lock().unwrap().insert(file_id.clone(), transfer);

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
        }).to_string();
        let chat_msg = ChatMessage {
            id: file_id.clone(),  // Must match file_transfers.id for LEFT JOIN
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
        self.emit_event("lan-file-transfer-started", &serde_json::json!({
            "fileId": file_id,
            "fileName": file_name,
            "fileSize": file_size,
            "to": peer_id,
            "toName": peer.name,
            "status": "sending",
        }));

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
                let _ = udp.send_to(msg.as_bytes(), format!("{}:{}", peer.address, peer.message_port));
            }
        }

        // Start TCP file transfer in background thread
        let peer_addr = peer.address.clone();
        let peer_tcp_port = peer.message_port; // Use same port as discovery for simplicity, or FILE_TRANSFER_PORT
        let fp = file_path.to_string();
        let fn_ = file_name.to_string();
        let fid = file_id.clone();
        let ro = resume_offset;
        let transfers = Arc::clone(&self.file_transfers);
        let log = Arc::clone(&self.log_buffer);
        let send_app_handle = self.app_handle.lock().unwrap().clone();
        let db_conn = Arc::clone(&self.db_conn);

        thread::spawn(move || {
            if let Err(e) = Self::do_send_file(&peer_addr, peer_tcp_port, &fp, &fn_, &fid, ro, &transfers, &log, &send_app_handle, &db_conn) {
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
                    let _ = app.emit("lan-file-transfer-error", serde_json::json!({
                        "fileId": fid,
                        "error": e,
                    }));
                }
            }
        });

        Ok(file_id)
    }

    /// Actually send a file over TCP to the peer
    fn do_send_file(
        peer_addr: &str,
        _peer_tcp_port: u16,
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
        Self::add_log_static(log, "info", &format!("Connecting to {}:{} for file transfer", peer_addr, FILE_TRANSFER_PORT));
        let mut stream = TcpStream::connect_timeout(
            &format!("{}:{}", peer_addr, FILE_TRANSFER_PORT).parse().map_err(|e| format!("解析地址失败: {}", e))?,
            Duration::from_secs(10),
        ).map_err(|e| format!("TCP 连接超时: {}", e))?;

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
            let n = file.read(&mut buf).map_err(|e| format!("读取失败: {}", e))?;
            if n == 0 { break; }
            stream.write_all(&buf[..n]).map_err(|e| format!("写入TCP失败: {}", e))?;
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
                    let _ = lan::update_file_transfer(&conn, file_id, "sending", progress_pct, None, None);
                }
                if let Some(app) = app_handle {
                    let _ = app.emit("lan-file-transfer-progress", serde_json::json!({
                        "fileId": file_id,
                        "progress": progress_pct,
                        "sent": sent,
                        "total": file_size,
                    }));
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
            let _ = lan::update_file_transfer(&conn, file_id, "completed", 100, Some(file_path), Some(&completed_at));
            // Update chat_message for this file — update content JSON with status
            let content_json = serde_json::json!({
                "fileName": file_name,
                "fileSize": file_size,
                "filePath": file_path,
                "status": "completed",
                "isImage": Self::is_image_file(file_name),
            }).to_string();
            let _ = conn.execute(
                "UPDATE chat_messages SET content = ?1, status = 'completed', progress = 100 WHERE id = ?2",
                rusqlite::params![content_json, file_id],
            );
        }

        Self::add_log_static(log, "info", &format!("File sent: {} ({} bytes)", file_name, sent));

        // Get from/to user info from transfers map
        let (from_user_id, from_user_name, to_user_id, to_user_name) = {
            let tf = transfers.lock().unwrap();
            if let Some(t) = tf.get(file_id) {
                (t.from_user_id.clone(), t.from_user_name.clone(), t.to_user_id.clone(), t.to_user_name.clone())
            } else {
                ("unknown".to_string(), "unknown".to_string(), "unknown".to_string(), "unknown".to_string())
            }
        };

        if let Some(app) = app_handle {
            let _ = app.emit("lan-file-transfer-completed", serde_json::json!({
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
            }));
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
            ("screencapture", &["-x", screenshot_path]),           // macOS
            ("gnome-screenshot", &["-f", screenshot_path]),        // GNOME
            ("scrot", &[screenshot_path]),                          // scrot
            ("import", &["-window", "root", screenshot_path]),     // ImageMagick
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
        fs::create_dir_all(&temp_dir)
            .map_err(|e| format!("创建临时目录失败: {}", e))?;

        let file_path = temp_dir.join(file_name);
        let decoded = BASE64.decode(base64_data)
            .map_err(|e| format!("Base64 解码失败: {}", e))?;
        fs::write(&file_path, decoded)
            .map_err(|e| format!("写入文件失败: {}", e))?;

        Ok(file_path.to_string_lossy().to_string())
    }

    /// Read file and encode as base64
    pub fn load_file_as_base64(&self, file_path: &str) -> Result<String, String> {
        let data = fs::read(file_path)
            .map_err(|e| format!("读取文件失败: {}", e))?;
        Ok(BASE64.encode(&data))
    }

    // ========== Collaboration broadcast methods ==========

    /// Send task assignment to a specific peer
    pub fn assign_task(&self, peer_id: &str, task_data: &serde_json::Value) -> Result<bool, String> {
        self.send_collab_message(peer_id, "assign_task", task_data)
    }

    /// Broadcast task update to all online peers
    pub fn broadcast_task_update(&self, task_data: &serde_json::Value) -> Result<usize, String> {
        self.broadcast_collab_message("task_update", task_data)
    }

    /// Broadcast task status change to all online peers
    pub fn broadcast_task_status_change(&self, task_data: &serde_json::Value) -> Result<usize, String> {
        self.broadcast_collab_message("task_status_change", task_data)
    }

    /// Send task comment to a specific peer
    pub fn broadcast_task_comment(&self, peer_id: &str, comment_data: &serde_json::Value) -> Result<bool, String> {
        self.send_collab_message(peer_id, "task_comment", comment_data)
    }

    /// Broadcast collaboration started to all online peers
    pub fn broadcast_collaboration_started(&self, collab_data: &serde_json::Value) -> Result<usize, String> {
        self.broadcast_collab_message("collaboration_started", collab_data)
    }

    /// Broadcast collaboration ended to all online peers
    pub fn broadcast_collaboration_ended(&self, collab_data: &serde_json::Value) -> Result<usize, String> {
        self.broadcast_collab_message("collaboration_ended", collab_data)
    }

    /// Send collaboration message to a specific peer
    fn send_collab_message(&self, peer_id: &str, msg_type: &str, data: &serde_json::Value) -> Result<bool, String> {
        if !self.is_running.load(Ordering::SeqCst) {
            return Err("LAN service not running".to_string());
        }
        let peers = self.peers.lock().unwrap();
        let peer = peers.get(peer_id).ok_or("Peer not found")?;

        let msg = serde_json::json!({
            "type": msg_type,
            "from": self.user_id,
            "fromName": self.nick_name.lock().unwrap().clone(),
            "data": data,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        });

        if let Ok(data_str) = serde_json::to_string(&msg) {
            if let Some(udp) = self.udp_socket.lock().unwrap().as_ref() {
                let _ = udp.send_to(data_str.as_bytes(), format!("{}:{}", peer.address, peer.message_port));
                return Ok(true);
            }
        }
        Err("发送失败".to_string())
    }

    /// Broadcast collaboration message to all online peers, returns sent count
    fn broadcast_collab_message(&self, msg_type: &str, data: &serde_json::Value) -> Result<usize, String> {
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

        let data_str = serde_json::to_string(&msg)
            .map_err(|e| format!("序列化失败: {}", e))?;

        let peers = self.peers.lock().unwrap();
        let mut sent_count = 0;
        if let Some(udp) = self.udp_socket.lock().unwrap().as_ref() {
            for peer in peers.values() {
                if peer.online {
                    let _ = udp.send_to(data_str.as_bytes(), format!("{}:{}", peer.address, peer.message_port));
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

        let data_str = serde_json::to_string(&msg)
            .map_err(|e| format!("序列化失败: {}", e))?;

        let peers = self.peers.lock().unwrap();
        let mut sent_count = 0;
        if let Some(udp) = self.udp_socket.lock().unwrap().as_ref() {
            for peer in peers.values() {
                if peer.online {
                    let _ = udp.send_to(data_str.as_bytes(), format!("{}:{}", peer.address, peer.message_port));
                    sent_count += 1;
                }
            }
        }
        Ok(sent_count)
    }
}
