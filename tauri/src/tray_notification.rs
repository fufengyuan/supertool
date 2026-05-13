#![allow(dead_code)]
/// Tray + Notification 管理
///
/// Notification: 任务到期提醒 + 部署通知
/// 使用 notify-rust 显示系统原生通知，使用 afplay/paplay 播放提示音
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Mutex;
use tauri::{Emitter, Manager};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotificationRequest {
    pub title: String,
    pub body: String,
    #[serde(rename = "todoId")]
    pub todo_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotificationTestResult {
    pub success: bool,
    pub message: String,
}

pub struct NotificationManager {
    notified_todo_ids: Mutex<HashSet<String>>,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            notified_todo_ids: Mutex::new(HashSet::new()),
        }
    }

    pub async fn check_task_notifications(&self, core: &crate::CoreService) -> Vec<NotificationRequest> {
        let mut notifications = vec![];
        let now = chrono::Utc::now();

        // Get reminder time from settings (default 15 minutes)
        let reminder_minutes = core.get_setting("reminder_time").await
            .ok()
            .and_then(|v| v.as_str().map(|s: &str| s.to_string()))
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(15);

        // Get all todos
        if let Ok(todos_result) = core.get_all_todos().await {
            if let Some(todos) = todos_result.as_array() {
                for todo in todos {
                    let id = todo.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    let completed = todo.get("completed").and_then(|v| v.as_bool()).unwrap_or(false);
                    let due_date = todo.get("dueDate").and_then(|v| v.as_str());

                    if completed || due_date.is_none() {
                        continue;
                    }

                    let due_date = due_date.unwrap();
                    if let Ok(due) = chrono::DateTime::parse_from_rfc3339(due_date) {
                        let time_diff = due.signed_duration_since(now);
                        let minutes_left = time_diff.num_minutes();

                        if minutes_left > 0 && minutes_left <= reminder_minutes {
                            // Check if already notified
                            if self.notified_todo_ids.lock().unwrap().contains(id) {
                                continue;
                            }

                            let title = format!("任务提醒: {}", todo.get("text").and_then(|v| v.as_str()).unwrap_or(""));
                            let body = format!(
                                "任务将在{}分钟后到期\n优先级: {}\n标签: {}",
                                minutes_left,
                                todo.get("priority").and_then(|v| v.as_str()).unwrap_or("medium"),
                                todo.get("tag").and_then(|v| v.as_str()).unwrap_or("未分类")
                            );

                            notifications.push(NotificationRequest {
                                title,
                                body,
                                todo_id: Some(id.to_string()),
                            });

                            self.notified_todo_ids.lock().unwrap().insert(id.to_string());
                        }
                    }
                }
            }
        }

        notifications
    }

    /// 显示系统原生通知 + 播放提示音
    pub fn show_notification(&self, req: &NotificationRequest) {
        play_notification_sound();
        if let Err(e) = notify_rust::Notification::new()
            .summary(&req.title)
            .body(&req.body)
            .show()
        {
            log::warn!("[Notification] Failed to show system notification: {}", e);
            // 降级：尝试 Web Notification (前端已实现)
            log::info!("[Notification] {}: {}", req.title, req.body);
        }
    }

    pub fn dismiss_notification(&self, todo_id: Option<&str>) {
        let mut ids = self.notified_todo_ids.lock().unwrap();
        if let Some(id) = todo_id {
            ids.remove(id);
        } else {
            ids.clear();
        }
    }

    pub fn midnight_cleanup(&self) {
        self.notified_todo_ids.lock().unwrap().clear();
        log::info!("[Notification] Cleared notifiedTodoIds at midnight");
    }

    pub fn test_notification(&self) -> NotificationTestResult {
        play_notification_sound();
        match notify_rust::Notification::new()
            .summary("SuperTool 测试通知")
            .body("如果你看到这条消息，说明通知功能正常工作！")
            .show()
        {
            Ok(_) => NotificationTestResult {
                success: true,
                message: "测试通知已发送！".to_string(),
            },
            Err(e) => {
                log::warn!("[Notification] Test notification failed: {}", e);
                NotificationTestResult {
                    success: false,
                    message: format!("通知发送失败: {}", e),
                }
            }
        }
    }
}

/// 播放系统提示音
pub fn play_notification_sound() {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("afplay")
            .arg("/System/Library/Sounds/Glass.aiff")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .ok();
    }
    #[cfg(target_os = "linux")]
    {
        // Try paplay first, fallback to canberra-gtk-play
        if std::process::Command::new("paplay")
            .arg("/usr/share/sounds/freedesktop/stereo/bell.oga")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .is_err()
        {
            std::process::Command::new("canberra-gtk-play")
                .arg("-i")
                .arg("message")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .ok();
        }
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("powershell")
            .args(["-c", "[console]::beep(800,300)"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .ok();
    }
}

/// 显示部署结果的系统通知
pub fn show_deploy_notification(success: bool, project_name: &str, error: Option<&str>) {
    play_notification_sound();
    let (title, body) = if success {
        (
            "🚀 部署成功",
            format!("{} 已成功部署", project_name),
        )
    } else {
        (
            "❌ 部署失败",
            format!("{} 部署失败: {}", project_name, error.unwrap_or("未知错误")),
        )
    };
    if let Err(e) = notify_rust::Notification::new()
        .summary(title)
        .body(&body)
        .show()
    {
        log::warn!("[Notification] Failed to show deploy notification: {}", e);
    }
}

/// 发送 LAN 消息系统通知
pub fn show_lan_message_notification(from_name: &str, content: &str) {
    play_notification_sound();
    let title = "💬 局域网消息";
    let body = format!("{}: {}", from_name, content);
    if let Err(e) = notify_rust::Notification::new()
        .summary(title)
        .body(&body)
        .show()
    {
        log::warn!("[Notification] Failed to show LAN message notification: {}", e);
    }
}

/// 启动后台通知检查定时器（每 5 分钟检查一次到期任务）
pub fn start_notification_timer(app_handle: tauri::AppHandle) {
    use std::sync::Arc;
    let manager = Arc::new(NotificationManager::new());

    // 启动时立即检查一次
    {
        let manager = manager.clone();
        let handle = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            if let Some(core) = handle.try_state::<crate::CoreService>() {
                let notifications = manager.check_task_notifications(&core).await;
                for req in notifications {
                    manager.show_notification(&req);
                    // 同时通知前端
                    let _ = handle.emit("todo-notification", serde_json::json!({
                        "todoId": req.todo_id,
                        "title": req.title,
                        "body": req.body,
                    }));
                }
            }
        });
    }

    // 每 5 分钟检查一次
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(5 * 60));
        loop {
            interval.tick().await;
            if let Some(core) = app_handle.try_state::<crate::CoreService>() {
                let notifications = manager.check_task_notifications(&core).await;
                for req in notifications {
                    manager.show_notification(&req);
                    let _ = app_handle.emit("todo-notification", serde_json::json!({
                        "todoId": req.todo_id,
                        "title": req.title,
                        "body": req.body,
                    }));
                }
            }
        }
    });

    log::info!("[Notification] Background notification timer started (every 5 minutes)");
}

/// Tray manager — Tauri 2.0 原生托盘 (TrayIconEvent)
pub struct TrayManager;

impl TrayManager {
    pub fn handle_event(
        &self,
        app: &tauri::AppHandle,
        event: tauri::tray::TrayIconEvent,
    ) {
        match event {
            // Left-click on tray icon: toggle window visibility
            tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                ..
            } => {
                if let Some(window) = app.get_webview_window("main") {
                    if let Ok(visible) = window.is_visible() {
                        if visible {
                            let _ = window.minimize();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
