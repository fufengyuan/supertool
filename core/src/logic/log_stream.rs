//! 实时日志流 - 移植自 Tauri `tauri/src/commands/logs.rs`
//! 每台服务器一个 OS 线程 + SSH exec `tail -n N -f`，通过 mpsc channel 输出事件
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex, mpsc};

/// 日志流事件
#[derive(Clone, Debug)]
pub enum LogStreamEvent {
    Line {
        stream_id: String,
        server_id: String,
        server_name: String,
        line: String,
    },
    Error {
        stream_id: String,
        server_id: String,
        error: String,
    },
    ServerEnd {
        stream_id: String,
        server_id: String,
    },
}

/// 活跃流：streamId -> cancel_flags per server
static ACTIVE_STREAMS: LazyLock<Mutex<HashMap<String, Vec<Arc<Mutex<bool>>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// 启动日志流
pub fn start_stream(
    core: &crate::logic::CoreService,
    stream_id: &str,
    server_ids: &[String],
    command: &str,
    tx: mpsc::Sender<LogStreamEvent>,
) -> Result<(), String> {
    // Stop existing stream with same ID
    stop_stream(stream_id);

    let mut cancel_flags: Vec<Arc<Mutex<bool>>> = Vec::new();

    for server_id in server_ids {
        let cancel_flag = Arc::new(Mutex::new(false));
        cancel_flags.push(cancel_flag.clone());

        let core_ref = core.clone();
        let sid = server_id.clone();
        let cmd = command.to_string();
        let sid_clone = stream_id.to_string();
        let tx_clone = tx.clone();
        let cf = cancel_flag.clone();

        std::thread::spawn(move || {
            stream_server_logs(&core_ref, &sid_clone, &sid, &cmd, cf, tx_clone);
        });
    }

    ACTIVE_STREAMS
        .lock()
        .unwrap()
        .insert(stream_id.to_string(), cancel_flags);

    Ok(())
}

/// 停止日志流
pub fn stop_stream(stream_id: &str) {
    if let Some(flags) = ACTIVE_STREAMS.lock().unwrap().remove(stream_id) {
        for flag in &flags {
            *flag.lock().unwrap() = true;
        }
    }
}

/// 每台服务器的日志流线程——完全照搬 Tauri 的 stream_server_logs
fn stream_server_logs(
    core: &crate::logic::CoreService,
    stream_id: &str,
    server_id: &str,
    command: &str,
    cancel_flag: Arc<Mutex<bool>>,
    tx: mpsc::Sender<LogStreamEvent>,
) {
    use std::io::Read;

    // 1. 获取服务器信息
    let server = match core.db_read(|conn| -> Result<serde_json::Value, String> {
        conn.query_row(
            "SELECT * FROM servers WHERE id = ?1",
            rusqlite::params![server_id],
            |row| {
                let raw_pw: Option<String> = row.get("password")?;
                let decrypted_pw =
                    raw_pw.map(|pw| crate::encryption::try_decrypt_password(&pw));
                Ok(serde_json::json!({
                    "id": row.get::<_, String>("id")?,
                    "name": row.get::<_, String>("name")?,
                    "host": row.get::<_, String>("host")?,
                    "port": row.get::<_, i64>("port")?,
                    "username": row.get::<_, String>("username")?,
                    "password": decrypted_pw,
                    "sshKeyPath": row.get::<_, Option<String>>("sshKeyPath")?,
                }))
            },
        )
        .map_err(|e| e.to_string())
    }) {
        Ok(Ok(s)) => s,
        Ok(Err(e)) | Err(e) => {
            let _ = tx.send(LogStreamEvent::Error {
                stream_id: stream_id.to_string(),
                server_id: server_id.to_string(),
                error: format!("数据库查询失败: {}", e),
            });
            return;
        }
    };

    let host = server["host"].as_str().unwrap_or("").to_string();
    let port = server["port"].as_u64().unwrap_or(22) as u32;
    let username = server["username"].as_str().unwrap_or("").to_string();
    let password = server.get("password").and_then(|v| v.as_str()).map(|s| s.to_string());
    let ssh_key_path = server.get("sshKeyPath").and_then(|v| v.as_str()).map(|s| s.to_string());
    let server_name = server["name"].as_str().unwrap_or("unknown").to_string();

    // 2. TCP 连接
    let tcp = match std::net::TcpStream::connect((host.as_str(), port as u16)) {
        Ok(t) => t,
        Err(e) => {
            let _ = tx.send(LogStreamEvent::Error {
                stream_id: stream_id.to_string(),
                server_id: server_id.to_string(),
                error: format!("TCP连接失败: {}", e),
            });
            return;
        }
    };

    // 3. SSH 会话
    let mut sess = match ssh2::Session::new() {
        Ok(s) => s,
        Err(e) => {
            let _ = tx.send(LogStreamEvent::Error {
                stream_id: stream_id.to_string(),
                server_id: server_id.to_string(),
                error: format!("创建SSH会话失败: {}", e),
            });
            return;
        }
    };
    sess.set_tcp_stream(tcp);
    if let Err(e) = sess.handshake() {
        let _ = tx.send(LogStreamEvent::Error {
            stream_id: stream_id.to_string(),
            server_id: server_id.to_string(),
            error: format!("SSH握手失败: {}", e),
        });
        return;
    }

    // 4. 认证
    let auth_result = if let Some(ref pw) = password {
        sess.userauth_password(&username, pw)
    } else if let Some(ref key) = ssh_key_path {
        sess.userauth_pubkey_file(&username, None, std::path::Path::new(key), None)
    } else {
        sess.userauth_agent(&username)
    };
    if let Err(e) = auth_result {
        let _ = tx.send(LogStreamEvent::Error {
            stream_id: stream_id.to_string(),
            server_id: server_id.to_string(),
            error: format!("SSH认证失败: {}", e),
        });
        return;
    }

    // 5. 执行命令
    let mut channel = match sess.channel_session() {
        Ok(c) => c,
        Err(e) => {
            let _ = tx.send(LogStreamEvent::Error {
                stream_id: stream_id.to_string(),
                server_id: server_id.to_string(),
                error: format!("创建SSH channel失败: {}", e),
            });
            return;
        }
    };
    if let Err(e) = channel.exec(command) {
        let _ = tx.send(LogStreamEvent::Error {
            stream_id: stream_id.to_string(),
            server_id: server_id.to_string(),
            error: format!("执行命令失败: {}", e),
        });
        return;
    }

    // 6. 读取输出
    let mut buf = [0u8; 4096];
    let mut line_buf = String::new();
    loop {
        if *cancel_flag.lock().unwrap() {
            break;
        }

        // 非阻塞读
        match channel.read(&mut buf) {
            Ok(0) => break, // EOF
            Ok(n) => {
                line_buf.push_str(&String::from_utf8_lossy(&buf[..n]));
                // 按行分割发送
                while let Some(pos) = line_buf.find('\n') {
                    let line = line_buf[..pos].to_string();
                    if !line.is_empty() {
                        let _ = tx.send(LogStreamEvent::Line {
                            stream_id: stream_id.to_string(),
                            server_id: server_id.to_string(),
                            server_name: server_name.clone(),
                            line,
                        });
                    }
                    line_buf.drain(..=pos);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // 非阻塞超时，重试
                std::thread::sleep(std::time::Duration::from_millis(50));
                continue;
            }
            Err(e) => {
                let _ = tx.send(LogStreamEvent::Error {
                    stream_id: stream_id.to_string(),
                    server_id: server_id.to_string(),
                    error: format!("读取失败: {}", e),
                });
                break;
            }
        }
    }

    // 发送剩余行
    if !line_buf.is_empty() {
        let _ = tx.send(LogStreamEvent::Line {
            stream_id: stream_id.to_string(),
            server_id: server_id.to_string(),
            server_name,
            line: line_buf,
        });
    }

    let _ = tx.send(LogStreamEvent::ServerEnd {
        stream_id: stream_id.to_string(),
        server_id: server_id.to_string(),
    });
}

/// 生成长连接命令（带 -f / -F），完全照搬 Vue log-tail 计算的 buildCommand
pub fn build_stream_command(preset: &serde_json::Value) -> String {
    let log_type = preset["logType"].as_str().unwrap_or("file");
    let log_path = preset["logPath"].as_str().unwrap_or("");
    let max_lines = preset["maxLines"].as_i64().unwrap_or(200);

    let paths: Vec<String> = log_path
        .split('\n')
        .filter(|p| !p.trim().is_empty())
        .map(|p| p.trim().to_string())
        .collect();

    let quote = |p: &str| {
        if let Some(rest) = p.strip_prefix('~') {
            if rest.is_empty() { "$HOME".to_string() }
            else { format!("$HOME'{}'", rest.replace('\'', "'\\''")) }
        } else {
            format!("'{}'", p.replace('\'', "'\\''"))
        }
    };

    match log_type {
        "journalctl" => format!(
            "journalctl {} -n {} -f --no-pager",
            paths.iter().map(|u| format!("-u {}", quote(u))).collect::<Vec<_>>().join(" "),
            max_lines
        ),
        "docker" => paths.iter()
            .map(|c| format!("(echo \"=== {} ===\" && docker logs --tail {} -f {} 2>&1)",
                quote(c), max_lines, quote(c)))
            .collect::<Vec<_>>().join(" & "),
        "custom" => log_path.to_string(),
        _ => format!("tail -n {} -f {}", max_lines,
            paths.iter().map(|p| quote(p)).collect::<Vec<_>>().join(" ")),
    }
}
