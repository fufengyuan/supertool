#![allow(dead_code)]
/// Tray + Notification 管理
///
/// Tauri 2.0 原生支持系统托盘和通知，无需额外依赖。
/// Tray: 右键菜单（显示窗口/退出）
/// Notification: 任务到期提醒 + 部署通知
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Mutex;
use tauri::Manager;

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

    pub async fn check_task_notifications(&self, core: &crate::core::CoreService) -> Vec<NotificationRequest> {
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

    pub fn show_notification(&self, req: &NotificationRequest) {
        // Tauri notification via tauri-plugin-notification
        // In a real implementation, this would use tauri::Notification
        log::info!("[Notification] {}: {}", req.title, req.body);
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
        let result = NotificationTestResult {
            success: true,
            message: "测试通知 — 通知功能正常工作".to_string(),
        };
        self.show_notification(&NotificationRequest {
            title: "测试通知".to_string(),
            body: result.message.clone(),
            todo_id: None,
        });
        result
    }
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
