use std::path::PathBuf;
use std::sync::Arc;
use supertool_omp::{OmpManager, ProcessEvent};
use tauri::{Emitter, Manager};
use uuid::Uuid;

/// omp 二进制路径
const OMP_BIN: &str = "omp";

/// OMP command 模块 — Tauri 事件桥接层
///
/// 将 `supertool-omp` crate 的 ProcessEvent 回调转为 Tauri 事件：
/// - `omp:stdout` — 单行 stdout
/// - `omp:stderr` — 单行 stderr
/// - `omp:exit`   — 进程退出

/// 启动一个 omp session
#[tauri::command(rename_all = "camelCase")]
pub async fn omp_start(
    app: tauri::AppHandle,
    manager: tauri::State<'_, OmpManager>,
    session_id: String,
    args: Vec<String>,
    cwd: Option<String>,
) -> Result<String, String> {
    // 如果 session_id 为空，自动生成
    let sid = if session_id.is_empty() {
        Uuid::new_v4().to_string()
    } else {
        session_id
    };

    let cwd_path = cwd.map(PathBuf::from);
    let app_clone = app.clone();
    let sid_clone = sid.clone();

    let handler = Arc::new(move |event: ProcessEvent| {
        match event {
            ProcessEvent::Stdout(line) => {
                let _ = app_clone.emit("omp:stdout", serde_json::json!({
                    "sessionId": sid_clone,
                    "data": line,
                }));
            }
            ProcessEvent::Stderr(line) => {
                let _ = app_clone.emit("omp:stderr", serde_json::json!({
                    "sessionId": sid_clone,
                    "data": line,
                }));
            }
            ProcessEvent::Exit(code) => {
                let _ = app_clone.emit("omp:exit", serde_json::json!({
                    "sessionId": sid_clone,
                    "exitCode": code,
                }));
                // 自动清理 session
                let app = app_clone.clone();
                let sid = sid_clone.clone();
                tokio::spawn(async move {
                    let mgr = app.state::<OmpManager>();
                    let _ = mgr.stop(&sid).await;
                });
            }
        }
    });

    // args: 默认 ["launch"]，用传入的 args
    let cmd_args: Vec<String> = if args.is_empty() {
        vec!["launch".to_string()]
    } else {
        args
    };

    manager
        .start(&sid, &cmd_args, cwd_path, handler)
        .await
        .map_err(|e| e.to_string())?;

    Ok(sid)
}

/// 向 omp session 写入数据
#[tauri::command(rename_all = "camelCase")]
pub async fn omp_write(
    manager: tauri::State<'_, OmpManager>,
    session_id: String,
    data: String,
) -> Result<(), String> {
    manager
        .write(&session_id, &data)
        .await
        .map_err(|e| e.to_string())
}

/// 终止 omp session
#[tauri::command(rename_all = "camelCase")]
pub async fn omp_stop(
    manager: tauri::State<'_, OmpManager>,
    session_id: String,
) -> Result<(), String> {
    manager
        .stop(&session_id)
        .await
        .map_err(|e| e.to_string())
}

/// 检查 session 是否存活
#[tauri::command(rename_all = "camelCase")]
pub async fn omp_is_running(
    manager: tauri::State<'_, OmpManager>,
    session_id: String,
) -> Result<bool, String> {
    Ok(manager.is_running(&session_id).await)
}
