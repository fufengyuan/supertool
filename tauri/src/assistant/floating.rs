//! AI 助手悬浮窗
//!
//! 与悬浮待办窗同一套形态：默认小球（56×56），点开成 400×560 的对话面板。
//! 不随应用启动自动创建（避免打扰），由侧栏/助手页的「悬浮唤起」显式打开。
use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};

pub const WINDOW_LABEL: &str = "floating-assistant";
const PANEL_WIDTH: f64 = 400.0;
const PANEL_HEIGHT: f64 = 560.0;
const BALL_SIZE: f64 = 56.0;

/// 创建悬浮助手窗（已存在则跳过）
pub fn ensure_floating_assistant<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    if app.get_webview_window(WINDOW_LABEL).is_some() {
        return Ok(());
    }
    let window = WebviewWindowBuilder::new(app, WINDOW_LABEL, WebviewUrl::App("index.html".into()))
        .title("AI 配置助手")
        .inner_size(PANEL_WIDTH, PANEL_HEIGHT)
        .min_inner_size(BALL_SIZE, BALL_SIZE)
        .always_on_top(true)
        .decorations(false)
        .transparent(true)
        .accept_first_mouse(true)
        .skip_taskbar(true)
        .resizable(true)
        .build()?;

    // 定位到屏幕右下角（逻辑坐标，兼容多屏/Retina/Dock）
    let monitor_opt = app
        .primary_monitor()
        .ok()
        .flatten()
        .or_else(|| window.current_monitor().ok().flatten());
    if let Some(monitor) = monitor_opt {
        let scale = monitor.scale_factor();
        let mon_x = monitor.position().x as f64 / scale;
        let mon_y = monitor.position().y as f64 / scale;
        let mon_w = monitor.size().width as f64 / scale;
        let mon_h = monitor.size().height as f64 / scale;
        #[cfg(target_os = "macos")]
        let gap_y = 80.0_f64; // Dock 高度
        #[cfg(not(target_os = "macos"))]
        let gap_y = 40.0_f64;
        let x = mon_x + (mon_w - PANEL_WIDTH - 20.0).max(0.0);
        let y = mon_y + (mon_h - PANEL_HEIGHT - gap_y).max(0.0);
        let _ = window.set_position(tauri::LogicalPosition::new(x, y));
    }

    #[cfg(target_os = "macos")]
    let _ = window.set_visible_on_all_workspaces(true);

    Ok(())
}

/// 打开（或聚焦）悬浮助手
#[tauri::command(rename_all = "camelCase")]
pub async fn open_floating_assistant(app: AppHandle) -> Result<serde_json::Value, String> {
    match app.get_webview_window(WINDOW_LABEL) {
        Some(window) => {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
        }
        None => {
            ensure_floating_assistant(&app).map_err(|e| e.to_string())?;
        }
    }
    Ok(serde_json::json!({ "success": true }))
}

/// 关闭（销毁）悬浮助手窗
#[tauri::command(rename_all = "camelCase")]
pub async fn close_floating_assistant(app: AppHandle) -> Result<serde_json::Value, String> {
    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(serde_json::json!({ "success": true }))
}

/// 显示/隐藏切换
#[tauri::command(rename_all = "camelCase")]
pub async fn toggle_floating_assistant(app: AppHandle) -> Result<serde_json::Value, String> {
    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        if window.is_visible().unwrap_or(false) {
            window.hide().map_err(|e| e.to_string())?;
        } else {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
        }
    } else {
        ensure_floating_assistant(&app).map_err(|e| e.to_string())?;
    }
    Ok(serde_json::json!({ "success": true }))
}

/// 置顶开关
#[tauri::command(rename_all = "camelCase")]
pub async fn set_floating_assistant_pinned(
    app: AppHandle,
    pinned: bool,
) -> Result<serde_json::Value, String> {
    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        window.set_always_on_top(pinned).map_err(|e| e.to_string())?;
    }
    Ok(serde_json::json!({ "success": true }))
}
