use std::collections::HashMap;
use std::sync::Mutex;
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex as TokioMutex;

/// 本地进程管理器
pub struct LocalProcessManager {
    processes: Mutex<HashMap<String, ProcessHandle>>,
}

struct ProcessHandle {
    stdin: TokioMutex<tokio::process::ChildStdin>,
    child: TokioMutex<Option<Child>>,
}

impl LocalProcessManager {
    pub fn new() -> Self {
        Self {
            processes: Mutex::new(HashMap::new()),
        }
    }
}

/// 启动本地进程，stdout/stderr → Tauri 事件流
#[tauri::command(rename_all = "camelCase")]
pub async fn start_local_process(
    app: tauri::AppHandle,
    manager: tauri::State<'_, LocalProcessManager>,
    process_id: String,
    command: String,
    args: Vec<String>,
    cwd: Option<String>,
) -> Result<(), String> {
    let mut cmd = Command::new(&command);
    cmd.args(&args);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    cmd.stdin(std::process::Stdio::piped());
    if let Some(dir) = &cwd {
        cmd.current_dir(dir);
    }

    let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn: {e}"))?;

    let stdout = child.stdout.take().ok_or("No stdout")?;
    let stderr = child.stderr.take().ok_or("No stderr")?;
    let child_stdin = child.stdin.take().ok_or("No stdin")?;

    let app_exit = app.clone();
    let pid_exit = process_id.clone();

    // 单个后台任务：同时监控 stdout + stderr EOF，然后 wait
    tokio::spawn(async move {
        // 读 stdout
        let app_so = app_exit.clone();
        let pid_so = pid_exit.clone();
        let out_handle = tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = app_so.emit("local-process-data", serde_json::json!({
                    "processId": pid_so,
                    "data": line,
                    "stream": "stdout",
                }));
            }
        });

        // 读 stderr
        let app_se = app_exit.clone();
        let pid_se = pid_exit.clone();
        let err_handle = tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = app_se.emit("local-process-data", serde_json::json!({
                    "processId": pid_se,
                    "data": line,
                    "stream": "stderr",
                }));
            }
        });

        // 等待输出流读完
        let _ = tokio::join!(out_handle, err_handle);
        // 此时 stdin 仍被 ProcessHandle 持有，但子进程可能已退出
        // 等一小会儿让子进程自然退出
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        // child 已通过 ProcessHandle 管理，此处不再 wait
        // 发出退出事件（无 exitCode）
        let _ = app_exit.emit("local-process-exit", serde_json::json!({
            "processId": pid_exit,
            "exitCode": null,
        }));
    });

    let handle = ProcessHandle {
        stdin: TokioMutex::new(child_stdin),
        child: TokioMutex::new(Some(child)),
    };

    {
        let mut map = manager.processes.lock().map_err(|e| format!("Lock: {e}"))?;
        map.insert(process_id, handle);
    }

    Ok(())
}

/// 写入 stdin
#[tauri::command(rename_all = "camelCase")]
pub async fn write_to_local_process(
    manager: tauri::State<'_, LocalProcessManager>,
    process_id: String,
    data: String,
) -> Result<(), String> {
    let mut handle = {
        let mut map = manager.processes.lock().map_err(|e| format!("Lock: {e}"))?;
        map.remove(&process_id)
    }
    .ok_or("Process not found")?;

    {
        let mut stdin = handle.stdin.lock().await;
        stdin
            .write_all(data.as_bytes())
            .await
            .map_err(|e| format!("Write: {e}"))?;
        stdin.flush().await.map_err(|e| format!("Flush: {e}"))?;
    }

    {
        let mut map = manager.processes.lock().map_err(|e| format!("Lock: {e}"))?;
        map.insert(process_id, handle);
    }

    Ok(())
}

/// 终止进程
#[tauri::command(rename_all = "camelCase")]
pub async fn kill_local_process(
    manager: tauri::State<'_, LocalProcessManager>,
    process_id: String,
) -> Result<(), String> {
    let handle = {
        let mut map = manager.processes.lock().map_err(|e| format!("Lock: {e}"))?;
        map.remove(&process_id)
    };

    if let Some(handle) = handle {
        {
            let mut stdin = handle.stdin.lock().await;
            let _ = stdin.shutdown().await;
        }
        let mut child_lock = handle.child.lock().await;
        if let Some(child) = child_lock.as_mut() {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
    }

    Ok(())
}
