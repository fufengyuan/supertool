use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use supertool_core::logic::CoreService;
use tauri::{AppHandle, Emitter, State};

use std::sync::LazyLock;

// Active log streams: streamId -> cancellation flags per server
static ACTIVE_STREAMS: LazyLock<Mutex<HashMap<String, Vec<Arc<Mutex<bool>>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[tauri::command(rename_all = "camelCase")]
pub async fn get_log_presets(core: State<'_, CoreService>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_log_presets() called");
    let result = core.get_log_presets().await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_log_preset(
    core: State<'_, CoreService>,
    preset: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_log_preset() called");
    let result = core.add_log_preset(preset).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_log_preset(
    core: State<'_, CoreService>,
    id: String,
    updates: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_log_preset() called");
    let result = core.update_log_preset(&id, updates).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_log_preset(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_log_preset() called");
    let result = core.delete_log_preset(&id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn log_search(
    core: State<'_, CoreService>,
    preset_id: String,
    keyword: String,
    lines: usize,
    date: Option<String>,
    days: Option<u64>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] log_search() called");
    let result = core.log_search(&preset_id, &keyword, lines, date.as_deref(), days).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn log_tail(
    core: State<'_, CoreService>,
    preset_id: String,
    lines: usize,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] log_tail() called");
    let result = core.log_tail(&preset_id, lines).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

/// Load more historical log lines for an active stream.
/// Queries each server in the preset for older lines beyond `currentCount`
/// and returns them as a JSON result for the frontend to prepend.
#[tauri::command(rename_all = "camelCase")]
pub async fn logs_load_more(
    core: State<'_, CoreService>,
    stream_id: String,
    preset_id: String,
    current_count: usize,
    batch_size: Option<usize>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] logs_load_more() stream={}, preset={}, currentCount={}", stream_id, preset_id, current_count);
    let batch = batch_size.unwrap_or(500);
    let result = core.load_more_logs(&preset_id, current_count, batch).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

/// Load context lines around a specific line number from a specific server.
#[tauri::command(rename_all = "camelCase")]
pub async fn log_context(
    core: State<'_, CoreService>,
    preset_id: String,
    server_id: String,
    line_num: usize,
    context_lines: Option<usize>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] log_context() preset={}, server={}, line={}", preset_id, server_id, line_num);
    let ctx = context_lines.unwrap_or(100);
    let result = core.log_context(&preset_id, &server_id, line_num, ctx).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

// =================== Log Streaming ===================

#[tauri::command(rename_all = "camelCase")]
pub async fn logs_start_stream(
    app: AppHandle,
    core: State<'_, CoreService>,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] logs_start_stream() called");

    let stream_id = params["streamId"].as_str().unwrap_or("").to_string();
    let server_ids: Vec<String> = params["serverIds"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let command = params["command"].as_str().unwrap_or("").to_string();

    if stream_id.is_empty() || server_ids.is_empty() || command.is_empty() {
        return Err("streamId, serverIds, command are required".to_string());
    }

    // Stop any existing stream with same ID
    stop_stream_by_id(&stream_id);

    let mut cancel_flags: Vec<Arc<Mutex<bool>>> = Vec::new();

    for server_id in &server_ids {
        let cancel_flag = Arc::new(Mutex::new(false));
        cancel_flags.push(cancel_flag.clone());

        let app_clone = app.clone();
        let stream_id_clone = stream_id.clone();
        let server_id_clone = server_id.clone();
        let command_clone = command.clone();
        let core_ptr = core.inner().clone();

        // Spawn a thread for each server to avoid blocking
        std::thread::spawn(move || {
            stream_server_logs(
                &core_ptr,
                &app_clone,
                &stream_id_clone,
                &server_id_clone,
                &command_clone,
                cancel_flag,
            );
        });
    }

    // Register active stream
    ACTIVE_STREAMS
        .lock()
        .unwrap()
        .insert(stream_id.clone(), cancel_flags);

    Ok(serde_json::json!({ "success": true, "streamId": stream_id }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn logs_stop_stream(stream_id: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] logs_stop_stream() called: {}", stream_id);
    stop_stream_by_id(&stream_id);
    Ok(serde_json::json!({ "success": true }))
}

fn stop_stream_by_id(stream_id: &str) {
    if let Some(flags) = ACTIVE_STREAMS.lock().unwrap().remove(stream_id) {
        for flag in &flags {
            *flag.lock().unwrap() = true;
        }
    }
}

fn stream_server_logs(
    core: &CoreService,
    app: &AppHandle,
    stream_id: &str,
    server_id: &str,
    command: &str,
    cancel_flag: Arc<Mutex<bool>>,
) {
    use std::io::Read;

    // Get server info (with password decryption)
    let server = match core.db_read(|conn| {
        conn.query_row(
            "SELECT * FROM servers WHERE id = ?1",
            rusqlite::params![server_id],
            |row| {
                let raw_pw: Option<String> = row.get("password")?;
                let decrypted_pw =
                    raw_pw.map(|pw| supertool_core::encryption::try_decrypt_password(&pw));
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
            let _ = app.emit(
                "logs:error",
                serde_json::json!({
                    "streamId": stream_id, "serverId": server_id, "error": e
                }),
            );
            return;
        }
    };

    let host = server["host"].as_str().unwrap_or("");
    let port = server["port"].as_u64().unwrap_or(22) as u32;
    let username = server["username"].as_str().unwrap_or("");
    let password = server.get("password").and_then(|v| v.as_str());
    let ssh_key_path = server.get("sshKeyPath").and_then(|v| v.as_str());

    let config = supertool_core::logic::ssh::SshServerConfig {
        id: server_id.to_string(),
        name: server["name"].as_str().unwrap_or("").to_string(),
        host: host.to_string(),
        port,
        username: username.to_string(),
        password: password.map(|s| s.to_string()),
        ssh_key_path: ssh_key_path.map(|s| s.to_string()),
    };

    // 独立SSH连接，避免污染全局连接池
    let tcp = match std::net::TcpStream::connect((host, port as u16)) {
        Ok(t) => t,
        Err(e) => {
            let _ = app.emit("logs:error", serde_json::json!({
                "streamId": stream_id, "serverId": server_id, "error": format!("TCP连接失败: {}", e)
            }));
            return;
        }
    };
    let mut sess = match ssh2::Session::new() {
        Ok(s) => s,
        Err(e) => {
            let _ = app.emit("logs:error", serde_json::json!({
                "streamId": stream_id, "serverId": server_id, "error": format!("创建SSH会话失败: {}", e)
            }));
            return;
        }
    };
    sess.set_tcp_stream(tcp);
    // handshake 和 auth 必须阻塞模式，否则直接报 operation would block
    if let Err(e) = sess.handshake() {
        let _ = app.emit(
            "logs:error",
            serde_json::json!({
                "streamId": stream_id, "serverId": server_id, "error": format!("SSH握手失败: {}", e)
            }),
        );
        return;
    }

    if let Some(pw) = &config.password {
        if let Err(e) = sess.userauth_password(&config.username, pw) {
            let _ = app.emit("logs:error", serde_json::json!({
                "streamId": stream_id, "serverId": server_id, "error": format!("SSH认证失败: {}", e)
            }));
            return;
        }
    } else if let Some(key_path) = &config.ssh_key_path {
        if let Err(e) =
            sess.userauth_pubkey_file(&config.username, None, std::path::Path::new(key_path), None)
        {
            let _ = app.emit("logs:error", serde_json::json!({
                "streamId": stream_id, "serverId": server_id, "error": format!("SSH密钥认证失败: {}", e)
            }));
            return;
        }
    } else {
        let _ = app.emit(
            "logs:error",
            serde_json::json!({
                "streamId": stream_id, "serverId": server_id, "error": "没有密码或密钥"
            }),
        );
        return;
    }

    let mut channel = match sess.channel_session() {
        Ok(c) => c,
        Err(e) => {
            let _ = app.emit("logs:error", serde_json::json!({
                "streamId": stream_id, "serverId": server_id, "error": format!("创建通道失败: {}", e)
            }));
            return;
        }
    };

    if let Err(e) = channel.exec(command) {
        let _ = app.emit("logs:error", serde_json::json!({
            "streamId": stream_id, "serverId": server_id, "error": format!("执行命令失败: {}", e)
        }));
        return;
    }

    // 命令执行后切换到非阻塞模式，配合cancel_flag轮询
    sess.set_blocking(false);

    // 读取日志（非阻塞轮询）
    let mut buf = [0u8; 4096];
    let mut leftover = String::new();
    let server_name = server["name"].as_str().unwrap_or("");

    loop {
        if *cancel_flag.lock().unwrap() {
            break;
        }
        match channel.read(&mut buf) {
            Ok(0) => {
                // 0字节：可能是EOF也可能是非阻塞下暂时无数据
                if channel.eof() {
                    break;
                }
                // 非阻塞下暂时无数据，短暂休眠避免CPU空转
                std::thread::sleep(std::time::Duration::from_millis(50));
                continue;
            }
            Ok(n) => {
                leftover.push_str(&String::from_utf8_lossy(&buf[..n]));
                while let Some(pos) = leftover.find('\n') {
                    let line = leftover[..pos].trim_end_matches('\r').to_string();
                    leftover = leftover[pos + 1..].to_string();
                    let _ = app.emit(
                        "logs:line",
                        serde_json::json!({
                            "streamId": stream_id,
                            "serverId": server_id,
                            "serverName": server_name,
                            "line": line,
                        }),
                    );
                }
            }
            Err(e) => {
                // 非阻塞模式下会返回 WouldBlock，这是正常的
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    continue;
                }
                // 其他错误：退出
                break;
            }
        }
    }

    if !leftover.trim().is_empty() {
        let _ = app.emit(
            "logs:line",
            serde_json::json!({
                "streamId": stream_id,
                "serverId": server_id,
                "serverName": server_name,
                "line": leftover.trim().to_string(),
            }),
        );
    }

    let _ = channel.close();
    let _ = channel.wait_close();

    let _ = app.emit(
        "logs:server-end",
        serde_json::json!({
            "streamId": stream_id,
            "serverId": server_id,
        }),
    );
}

// =================== System Logger (frontend → file) ===================

/// 前端拦截 console.log 后调用，将日志写入文件
#[tauri::command(rename_all = "camelCase")]
pub fn write_system_log(level: String, prefix: String, message: String) {    crate::system_logger::SystemLogger::write_frontend_log(&level, &prefix, &message);
}

/// 读取已下载的离线日志本地文件（"复制全部"用，原样输出不做行号加工）。
/// 仅限 ~/.supertool 下载缓存目录下的文件，防止任意路径读取。
#[tauri::command(rename_all = "camelCase")]
pub async fn read_log_cache_file(path: String) -> Result<String, String> {
    log::info!("[Tauri CMD] read_log_cache_file() called");
    let home = std::env::var("HOME").unwrap_or_default();
    let allowed = format!("{}/.supertool", home);
    let expanded = shellexpand::tilde(&path).to_string();
    if !expanded.starts_with(&allowed) {
        return Err("仅允许读取 ~/.supertool 缓存目录下的日志文件".to_string());
    }
    let content = tokio::fs::read(&expanded)
        .await
        .map_err(|e| format!("读取日志文件失败: {}", e))?;
    if content.len() > 256 * 1024 * 1024 {
        return Err(format!(
            "日志文件过大（{}MB），请直接查看文件",
            content.len() / 1024 / 1024
        ));
    }
    Ok(String::from_utf8_lossy(&content).to_string())
}
