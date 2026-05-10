/// OpenVPN 管理器 — 进程管理 + 状态监控
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenVPNStatus {
    pub connected: bool,
    #[serde(rename = "configId")]
    pub config_id: Option<String>,
    #[serde(rename = "configName")]
    pub config_name: Option<String>,
    pub state: String,
    pub log: Vec<String>,
    #[serde(rename = "connectedSince")]
    pub connected_since: Option<String>,
    pub remote: Option<String>,
    #[serde(rename = "bytesSent")]
    pub bytes_sent: Option<u64>,
    #[serde(rename = "bytesReceived")]
    pub bytes_received: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrafficStats {
    #[serde(rename = "bytesSent")]
    pub bytes_sent: u64,
    #[serde(rename = "bytesReceived")]
    pub bytes_received: u64,
    #[serde(rename = "bytesSentHuman")]
    pub bytes_sent_human: String,
    #[serde(rename = "bytesReceivedHuman")]
    pub bytes_received_human: String,
}

struct PendingConnect {
    config_id: String,
    config_name: String,
    content: String,
}

pub struct OpenVPNManager {
    process: Mutex<Option<Child>>,
    status: Arc<Mutex<OpenVPNStatus>>,
    temp_config_path: Mutex<Option<PathBuf>>,
    log_buffer: Arc<Mutex<Vec<String>>>,
    pending_connect: Mutex<Option<PendingConnect>>,
    stop_flag: Arc<AtomicBool>,
    max_log_lines: usize,
}

impl OpenVPNManager {
    pub fn new() -> Self {
        Self {
            process: Mutex::new(None),
            status: Arc::new(Mutex::new(OpenVPNStatus {
                connected: false,
                config_id: None,
                config_name: None,
                state: "disconnected".to_string(),
                log: vec![],
                connected_since: None,
                remote: None,
                bytes_sent: Some(0),
                bytes_received: Some(0),
            })),
            temp_config_path: Mutex::new(None),
            log_buffer: Arc::new(Mutex::new(vec![])),
            pending_connect: Mutex::new(None),
            stop_flag: Arc::new(AtomicBool::new(false)),
            max_log_lines: 500,
        }
    }

    #[allow(dead_code)]
    pub fn check_available(&self) -> Result<String, String> {
        let openvpn_path = self.find_openvpn()?;
        let output = Command::new(&openvpn_path)
            .arg("--version")
            .output()
            .map_err(|e| format!("OpenVPN 不可用: {}", e))?;
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            Ok(version.lines().next().unwrap_or("OpenVPN").to_string())
        } else {
            Err("OpenVPN 版本检测失败".to_string())
        }
    }

    pub fn validate_config(&self, content: &str) -> Result<(), String> {
        if content.trim().is_empty() {
            return Err("配置文件内容为空".to_string());
        }
        let has_client = content.contains("client") || content.contains("pull");
        let has_dev = content.contains("dev ") || content.contains("dev-type");
        let has_proto = content.contains("proto ");
        let has_remote = content.contains("remote ");
        if !has_client && !has_dev && !has_proto && !has_remote {
            return Err("配置文件格式无效".to_string());
        }
        if !has_remote {
            return Err("缺少 remote 指令（服务器地址）".to_string());
        }
        Ok(())
    }

    pub fn get_status(&self) -> OpenVPNStatus {
        let mut status = self.status.lock().unwrap().clone();
        status.log = self.log_buffer.lock().unwrap().clone();
        status
    }

    pub fn get_traffic_stats(&self) -> Option<TrafficStats> {
        let status = self.status.lock().unwrap();
        if !status.connected || status.bytes_sent.is_none() {
            return None;
        }
        Some(TrafficStats {
            bytes_sent: status.bytes_sent.unwrap_or(0),
            bytes_received: status.bytes_received.unwrap_or(0),
            bytes_sent_human: Self::human_bytes(status.bytes_sent.unwrap_or(0)),
            bytes_received_human: Self::human_bytes(status.bytes_received.unwrap_or(0)),
        })
    }

    fn human_bytes(bytes: u64) -> String {
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    fn find_openvpn(&self) -> Result<String, String> {
        // 1. Check bundled OpenVPN binary (from Tauri resources)
        if let Ok(exe) = std::env::current_exe() {
            let platform = if cfg!(target_os = "macos") {
                if cfg!(target_arch = "aarch64") { "macos-arm64" } else { "macos-x64" }
            } else {
                "linux-x64"
            };
            // Try multiple parent levels for different bundle layouts
            let mut bases: Vec<std::path::PathBuf> = Vec::new();
            if let Some(p) = exe.parent() {
                bases.push(p.to_path_buf());                        // same dir
                if let Some(pp) = p.parent() {
                    bases.push(pp.to_path_buf());                   // 1 level up (e.g. /opt/App/bin -> /opt/App)
                    if let Some(ppp) = pp.parent() {
                        bases.push(ppp.to_path_buf());              // 2 levels up
                    }
                }
            }
            for base in &bases {
                let bundled = base.join("resources").join("openvpn").join(platform).join("openvpn");
                if bundled.exists() {
                    let output = Command::new(&bundled).arg("--version").output();
                    if let Ok(o) = output {
                        if o.status.success() {
                            return Ok(bundled.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
        // 2. Check system OpenVPN
        let candidates = [
            "openvpn",
            "/usr/sbin/openvpn",
            "/usr/bin/openvpn",
            "/usr/local/sbin/openvpn",
            "/opt/homebrew/opt/openvpn/sbin/openvpn",
        ];
        for path in &candidates {
            let output = Command::new(path).arg("--version").output();
            if let Ok(o) = output {
                if o.status.success() {
                    return Ok(path.to_string());
                }
            }
        }
        Err("OpenVPN 不可用，请安装 openvpn".to_string())
    }

    fn add_log(&self, line: &str) {
        let mut buf = self.log_buffer.lock().unwrap();
        buf.push(line.to_string());
        if buf.len() > self.max_log_lines {
            let drain_to = buf.len() - self.max_log_lines;
            buf.drain(0..drain_to);
        }
    }

    pub fn connect(
        &self,
        config_id: String,
        config_name: String,
        content: String,
        sudo_password: Option<String>,
    ) -> Result<bool, String> {
        self.validate_config(&content)?;

        {
            let status = self.status.lock().unwrap();
            if status.connected || self.process.lock().unwrap().is_some() {
                drop(status);
                let _ = self.disconnect();
                std::thread::sleep(Duration::from_millis(500));
            }
        }

        {
            let mut status = self.status.lock().unwrap();
            status.connected = false;
            status.config_id = Some(config_id.clone());
            status.config_name = Some(config_name.clone());
            status.state = "connecting".to_string();
            status.log.clear();
            status.bytes_sent = Some(0);
            status.bytes_received = Some(0);
            status.connected_since = None;
            status.remote = None;
        }
        self.log_buffer.lock().unwrap().clear();

        self.add_log(&format!("正在连接 {}...", config_name));

        let tmp_dir = crate::logic::data_dir::tmp_dir();
        fs::create_dir_all(&tmp_dir).map_err(|e| format!("创建目录失败: {}", e))?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let tmp_file = tmp_dir.join(format!("ovpn_{}.conf", timestamp));
        fs::write(&tmp_file, &content).map_err(|e| format!("写入配置失败: {}", e))?;
        *self.temp_config_path.lock().unwrap() = Some(tmp_file.clone());

        self.add_log(&format!("配置文件已写入: {} bytes", content.len()));

        let openvpn_bin = self.find_openvpn()?;
        self.add_log(&format!("OpenVPN: {}", openvpn_bin));

        let mgmt_sock = tmp_dir.join(format!("ovpn_mgmt_{}.sock", timestamp));
        let args = [
            "--config",
            &tmp_file.to_string_lossy(),
            "--management",
            &mgmt_sock.to_string_lossy(),
            "unix",
            "--script-security",
            "2",
        ];

        let needs_sudo = cfg!(target_os = "linux") || cfg!(target_os = "macos");

        let (cmd, spawn_args): (String, Vec<String>) = if needs_sudo {
            let sudo_ok = Command::new("sudo")
                .arg("-n")
                .arg("true")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if sudo_ok {
                let mut a = vec!["sudo".to_string(), "-n".to_string(), openvpn_bin.clone()];
                a.extend(args.iter().map(|s| s.to_string()));
                ("sudo".to_string(), a)
            } else if let Some(ref _pw) = sudo_password {
                let mut a = vec!["sudo".to_string(), "-S".to_string(), openvpn_bin.clone()];
                a.extend(args.iter().map(|s| s.to_string()));
                ("sudo".to_string(), a)
            } else {
                *self.pending_connect.lock().unwrap() = Some(PendingConnect {
                    config_id,
                    config_name,
                    content,
                });
                self.status.lock().unwrap().state = "password_required".to_string();
                self.add_log("⚠️ 需要 sudo 密码");
                return Err("NEEDS_PASSWORD".to_string());
            }
        } else {
            let mut a = vec![openvpn_bin.clone()];
            a.extend(args.iter().map(|s| s.to_string()));
            (openvpn_bin, a)
        };

        let actual_cmd = if needs_sudo {
            if sudo_password.is_some() {
                "sudo".to_string()
            } else {
                let sudo_ok = Command::new("sudo")
                    .arg("-n")
                    .arg("true")
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);
                if sudo_ok {
                    "sudo".to_string()
                } else {
                    cmd
                }
            }
        } else {
            cmd
        };

        self.add_log(&format!("启动: {} {}", actual_cmd, spawn_args.join(" ")));
        self.spawn_openvpn(&actual_cmd, &spawn_args, sudo_password)
    }

    pub fn retry_with_password(&self, password: String) -> Result<bool, String> {
        let pending = self.pending_connect.lock().unwrap().take();
        match pending {
            Some(p) => {
                self.cleanup();
                self.connect(p.config_id, p.config_name, p.content, Some(password))
            }
            None => Err("没有待重试的连接".to_string()),
        }
    }

    fn spawn_openvpn(
        &self,
        cmd: &str,
        args: &[String],
        sudo_password: Option<String>,
    ) -> Result<bool, String> {
        self.stop_flag.store(false, Ordering::SeqCst);

        let mut child = Command::new(cmd)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("启动 OpenVPN 失败: {}", e))?;

        let pid = child.id();
        self.add_log(&format!("进程 PID: {}", pid));

        if let Some(ref pw) = sudo_password {
            if let Some(ref mut stdin) = child.stdin {
                let _ = writeln!(stdin, "{}", pw);
                self.add_log("已发送 sudo 密码");
            }
        }

        let stdout = child.stdout.take();
        *self.process.lock().unwrap() = Some(child);

        let status_arc = Arc::clone(&self.status);
        let log_arc = Arc::clone(&self.log_buffer);
        let stop = Arc::clone(&self.stop_flag);
        let max_log = self.max_log_lines;

        if let Some(stdout) = stdout {
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().flatten() {
                    if stop.load(Ordering::SeqCst) {
                        break;
                    }
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    {
                        let mut buf = log_arc.lock().unwrap();
                        buf.push(trimmed.to_string());
                        if buf.len() > max_log {
                            let drain_to = buf.len() - max_log;
                            buf.drain(0..drain_to);
                        }
                    }
                    if trimmed.contains("Initialization Sequence Completed") {
                        let mut status = status_arc.lock().unwrap();
                        status.state = "connected".to_string();
                        status.connected = true;
                        status.connected_since = Some(chrono::Utc::now().to_rfc3339());
                    }
                    if trimmed.contains("AUTH_FAILED") || trimmed.contains("TLS Error") {
                        let mut status = status_arc.lock().unwrap();
                        status.state = "error".to_string();
                    }
                    if let Some(remote) = extract_remote(trimmed) {
                        let mut status = status_arc.lock().unwrap();
                        if status.remote.is_none() {
                            status.remote = Some(remote);
                        }
                    }
                }
            });
        }

        Ok(true)
    }

    pub fn disconnect(&self) -> Result<(), String> {
        self.stop_flag.store(true, Ordering::SeqCst);

        {
            let status = self.status.lock().unwrap();
            if !status.connected && status.state == "disconnected" {
                return Ok(());
            }
        }

        self.status.lock().unwrap().state = "disconnecting".to_string();
        self.add_log("正在断开...");

        if let Some(mut child) = self.process.lock().unwrap().take() {
            #[cfg(unix)]
            {
                let id = child.id();
                unsafe { libc::kill(id as i32, libc::SIGTERM) };
            }
            #[cfg(not(unix))]
            {
                let _ = child.kill();
            }

            let start = std::time::Instant::now();
            while start.elapsed() < Duration::from_secs(3) {
                if let Ok(Some(_)) = child.try_wait() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }

            if child.try_wait().ok().flatten().is_none() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }

        {
            let mut status = self.status.lock().unwrap();
            status.state = "disconnected".to_string();
            status.connected = false;
            status.config_id = None;
            status.config_name = None;
            status.bytes_sent = Some(0);
            status.bytes_received = Some(0);
        }
        self.log_buffer.lock().unwrap().clear();
        *self.pending_connect.lock().unwrap() = None;

        self.add_log("✅ 已断开");
        self.cleanup();
        Ok(())
    }

    fn cleanup(&self) {
        if let Some(path) = self.temp_config_path.lock().unwrap().take() {
            let _ = fs::remove_file(path);
        }
        let tmp_dir = crate::logic::data_dir::tmp_dir();
        if tmp_dir.exists() {
            if let Ok(entries) = fs::read_dir(&tmp_dir) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if (name.starts_with("ovpn_") || name.starts_with("supertool_"))
                            && name.ends_with(".sock")
                        {
                            let _ = fs::remove_file(entry.path());
                        }
                    }
                }
            }
        }
    }
}

fn extract_remote(line: &str) -> Option<String> {
    if line.contains("link local") || line.contains("link remote") {
        let parts: Vec<&str> = line.split(">").collect();
        if parts.len() > 1 {
            return Some(parts[1].trim().to_string());
        }
    }
    None
}
