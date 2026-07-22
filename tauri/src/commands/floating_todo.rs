use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};

/// Create the floating todo window if it doesn't exist yet.
/// Shared between the `open_floating_todo` command and startup auto-show.
pub fn ensure_floating_todo<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    if app.get_webview_window("floating-todo").is_some() {
        return Ok(());
    }
    let window = WebviewWindowBuilder::new(
        app,
        "floating-todo",
        WebviewUrl::App("index.html".into()),
    )
    .title("待办悬浮窗")
    .inner_size(340.0, 500.0)
    .min_inner_size(56.0, 56.0)
    .always_on_top(true)
    .decorations(false)
    .transparent(true)
    .accept_first_mouse(true)
    .skip_taskbar(true)
    .resizable(true)
    .build()?;

    // Position at bottom-right corner (logical coords, handles Retina/Dock correctly)
    let monitor_opt = app.primary_monitor().ok().flatten()
        .or_else(|| window.current_monitor().ok().flatten())
        .or_else(|| window.available_monitors().ok().and_then(|m| m.into_iter().next()));

    if let Some(monitor) = monitor_opt {
        let scale = monitor.scale_factor();
        let mon_pos = monitor.position();
        let mon_size = monitor.size();

        let mon_x = mon_pos.x as f64 / scale;
        let mon_y = mon_pos.y as f64 / scale;
        let mon_w = mon_size.width as f64 / scale;
        let mon_h = mon_size.height as f64 / scale;

        let ww = 340.0_f64;
        let wh = 500.0_f64;
        let gap_x = 20.0_f64;
        #[cfg(target_os = "macos")]
        let gap_y = 80.0_f64; // Dock height
        #[cfg(not(target_os = "macos"))]
        let gap_y = 40.0_f64;

        let x = mon_x + (mon_w - ww - gap_x).max(0.0);
        let y = mon_y + (mon_h - wh - gap_y).max(0.0);

        let _ = window.set_position(tauri::LogicalPosition::new(x, y));
    }

    // macOS: make it visible on all workspaces (spaces)
    #[cfg(target_os = "macos")]
    let _ = window.set_visible_on_all_workspaces(true);

    Ok(())
}

/// Open (or focus) the floating todo window.
/// Creates it if it doesn't exist yet.
#[tauri::command(rename_all = "camelCase")]
pub async fn open_floating_todo(app: AppHandle) -> Result<serde_json::Value, String> {
    if let Some(window) = app.get_webview_window("floating-todo") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    } else {
        ensure_floating_todo(&app).map_err(|e| e.to_string())?;
    }
    Ok(serde_json::json!({ "success": true }))
}

/// Close the floating todo window.
#[tauri::command(rename_all = "camelCase")]
pub async fn close_floating_todo(app: AppHandle) -> Result<serde_json::Value, String> {
    if let Some(window) = app.get_webview_window("floating-todo") {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(serde_json::json!({ "success": true }))
}

/// Toggle open/close the floating todo window.
#[tauri::command(rename_all = "camelCase")]
pub async fn toggle_floating_todo(app: AppHandle) -> Result<serde_json::Value, String> {
    if let Some(window) = app.get_webview_window("floating-todo") {
        if window.is_visible().unwrap_or(false) {
            window.hide().map_err(|e| e.to_string())?;
        } else {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
        }
    } else {
        ensure_floating_todo(&app).map_err(|e| e.to_string())?;
    }
    Ok(serde_json::json!({ "success": true }))
}

/// Set the pinned (always-on-top) state of the floating window.
#[tauri::command(rename_all = "camelCase")]
pub async fn set_floating_todo_pinned(app: AppHandle, pinned: bool) -> Result<serde_json::Value, String> {
    if let Some(window) = app.get_webview_window("floating-todo") {
        window.set_always_on_top(pinned).map_err(|e| e.to_string())?;
    }
    Ok(serde_json::json!({ "success": true }))
}
