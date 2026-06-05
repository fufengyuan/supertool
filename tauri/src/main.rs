#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod lan;
mod system_logger;
mod tray_notification;
#[cfg(test)]
mod tests;
use supertool_core::logic::openvpn;
use supertool_core::logic::wireguard;
use commands::claw_chat::ClawChatState;

use std::sync::OnceLock;
use supertool_core::db::Database;
use supertool_core::logic::CoreService;
use tauri::Emitter;
use tauri::Manager;
use tauri::Wry;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};

static APP_HANDLE: OnceLock<tauri::AppHandle<Wry>> = OnceLock::new();

#[tauri::command]
fn update_frequent_menu(items: Vec<String>) -> Result<(), String> {
    let handle = APP_HANDLE.get().ok_or("App handle not initialized")?;

    let nav_item = |id: &str, title: &str| -> Result<MenuItem<Wry>, String> {
        MenuItem::with_id(handle, id, title, true, None::<&str>).map_err(|e| e.to_string())
    };

    let mut menu_items: Vec<MenuItem<Wry>> = Vec::new();
    for item in &items {
        let parts: Vec<&str> = item.splitn(2, '|').collect();
        if parts.len() == 2 {
            menu_items.push(nav_item(parts[0], parts[1])?);
        }
    }

    // Build the frequent submenu items manually
    let frequent_submenu =
        Submenu::with_id(handle, "frequent", "常用功能", true).map_err(|e| e.to_string())?;
    for item in menu_items {
        frequent_submenu.append(&item).map_err(|e| e.to_string())?;
    }

    // Rebuild all menus
    let edit_menu = Submenu::with_items(
        handle,
        "编辑",
        true,
        &[
            &MenuItem::with_id(handle, "search", "全局搜索", true, Some("CmdOrCtrl+K"))
                .map_err(|e| e.to_string())?,
            &PredefinedMenuItem::separator(handle).map_err(|e| e.to_string())?,
            &PredefinedMenuItem::undo(handle, Some("撤销")).map_err(|e| e.to_string())?,
            &PredefinedMenuItem::redo(handle, Some("重做")).map_err(|e| e.to_string())?,
            &PredefinedMenuItem::separator(handle).map_err(|e| e.to_string())?,
            &PredefinedMenuItem::cut(handle, Some("剪切")).map_err(|e| e.to_string())?,
            &PredefinedMenuItem::copy(handle, Some("复制")).map_err(|e| e.to_string())?,
            &PredefinedMenuItem::paste(handle, Some("粘贴")).map_err(|e| e.to_string())?,
            &PredefinedMenuItem::separator(handle).map_err(|e| e.to_string())?,
            &PredefinedMenuItem::select_all(handle, Some("全选")).map_err(|e| e.to_string())?,
        ],
    )
    .map_err(|e| e.to_string())?;

    let business_menu = Submenu::with_items(
        handle,
        "业务",
        true,
        &[
            &nav_item("nav_todo", "任务列表")?,
            &nav_item("nav_weekly", "周报")?,
            &nav_item("nav_projects", "项目")?,
            &nav_item("nav_accounting", "记账本")?,
        ],
    )
    .map_err(|e| e.to_string())?;

    let ops_menu = Submenu::with_items(
        handle,
        "运维",
        true,
        &[
            &nav_item("nav_servers", "服务器管理")?,
            &nav_item("nav_cicd", "CI/CD 部署")?,
            &nav_item("nav_logs", "日志聚合")?,
        ],
    )
    .map_err(|e| e.to_string())?;

    let dev_menu = Submenu::with_items(
        handle,
        "开发",
        true,
        &[
            &nav_item("nav_db", "数据库管理")?,
            &nav_item("nav_devtools", "开发工具")?,
            &nav_item("nav_notes", "笔记")?,
            &nav_item("nav_git", "Git 仓库")?,
        ],
    )
    .map_err(|e| e.to_string())?;

    let security_menu = Submenu::with_items(
        handle,
        "安全",
        true,
        &[
            &nav_item("nav_mfa", "MFA 验证码")?,
            &nav_item("nav_vpn", "VPN")?,
        ],
    )
    .map_err(|e| e.to_string())?;

    let system_menu = Submenu::with_items(
        handle,
        "系统",
        true,
        &[
            &MenuItem::with_id(handle, "about", "关于", true, None::<&str>)
                .map_err(|e| e.to_string())?,
            &PredefinedMenuItem::separator(handle).map_err(|e| e.to_string())?,
            &nav_item("nav_backup", "数据备份")?,
            &PredefinedMenuItem::separator(handle).map_err(|e| e.to_string())?,
            &MenuItem::with_id(handle, "toggle_locale", "切换语言", true, None::<&str>)
                .map_err(|e| e.to_string())?,
            &MenuItem::with_id(handle, "toggle_theme", "切换主题", true, None::<&str>)
                .map_err(|e| e.to_string())?,
            &PredefinedMenuItem::separator(handle).map_err(|e| e.to_string())?,
            &PredefinedMenuItem::quit(handle, Some("退出")).map_err(|e| e.to_string())?,
        ],
    )
    .map_err(|e| e.to_string())?;

    let menu = Menu::with_items(
        handle,
        &[
            &edit_menu,
            &frequent_submenu,
            &business_menu,
            &ops_menu,
            &dev_menu,
            &security_menu,
            &system_menu,
        ],
    )
    .map_err(|e| e.to_string())?;

    handle.set_menu(menu).map_err(|e| e.to_string())?;

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Resolve data directory: check ~/.supertool_dir for custom path, fallback to ~/.supertool
            let home_dir = dirs::home_dir().expect("Failed to resolve home directory");
            let config_file = home_dir.join(".supertool_dir");
            let supertool_dir = if config_file.exists() {
                match std::fs::read_to_string(&config_file) {
                    Ok(content) => {
                        let custom_path = content.trim();
                        if !custom_path.is_empty() {
                            let path = std::path::PathBuf::from(custom_path);
                            log::info!("[Main] Using custom data directory: {}", path.display());
                            path
                        } else {
                            home_dir.join(".supertool")
                        }
                    }
                    Err(e) => {
                        log::warn!(
                            "[Main] Failed to read ~/.supertool_dir: {}, using default",
                            e
                        );
                        home_dir.join(".supertool")
                    }
                }
            } else {
                home_dir.join(".supertool")
            };
            std::fs::create_dir_all(&supertool_dir).expect("Failed to create data directory");

            // Initialize file logger (also under ~/.supertool/logs/)
            crate::system_logger::SystemLogger::init(&supertool_dir);

            log::info!("[Main] ================================");
            log::info!("[Main] SuperTool Tauri v{} 启动", env!("CARGO_PKG_VERSION"));
            log::info!("[Main] ================================");

            // Initialize SQLite database
            let db_path = supertool_dir.join("supertool.db");
            let database =
                Database::new(&db_path).expect("[Database] Failed to initialize database");
            log::info!("[Database] 初始化完成: {}", db_path.display());

            // CoreService
            let core = CoreService::new(database, supertool_dir.clone());
            app.manage(core.clone());
            log::info!("[CoreService] 初始化完成");

            // OpenVPN
            app.manage(openvpn::OpenVPNManager::new());
            log::info!("[OpenVPN] 管理器初始化完成");

            // WireGuard
            app.manage(wireguard::WireGuardManager::new());
            log::info!("[WireGuard] 管理器初始化完成");

            // LAN
            let db_path_str = db_path.to_string_lossy().to_string();
            crate::commands::lan::init_lan_service_with_db(&db_path_str);
            app.manage(ClawChatState::new());
            // Auto-start LAN service for team collaboration
            crate::commands::lan::auto_start_lan(app.handle());

            // Build custom application menu (mirrors Electron version)
            // Note: accelerators removed — on Linux GTK they produce
            // Gtk-WARNING about accel group registration. Keyboard shortcuts
            // are handled by the frontend instead.
            let handle = app.handle();
            let nav_item = |id: &str, title: &str| -> Result<MenuItem<Wry>, tauri::Error> {
                MenuItem::with_id(handle, id, title, true, None::<&str>)
            };
            let action_item = |id: &str, title: &str| -> Result<MenuItem<Wry>, tauri::Error> {
                MenuItem::with_id(handle, id, title, true, None::<&str>)
            };

            let edit_menu = Submenu::with_items(
                handle,
                "编辑",
                true,
                &[
                    &action_item("search", "全局搜索")?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::undo(handle, Some("撤销"))?,
                    &PredefinedMenuItem::redo(handle, Some("重做"))?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::cut(handle, Some("剪切"))?,
                    &PredefinedMenuItem::copy(handle, Some("复制"))?,
                    &PredefinedMenuItem::paste(handle, Some("粘贴"))?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::select_all(handle, Some("全选"))?,
                ],
            )?;

            let business_menu = Submenu::with_items(
                handle,
                "业务",
                true,
                &[
                    &nav_item("nav_todo", "任务列表")?,
                    &nav_item("nav_weekly", "周报")?,
                    &nav_item("nav_projects", "项目")?,
                    &nav_item("nav_accounting", "记账本")?,
                ],
            )?;

            let ops_menu = Submenu::with_items(
                handle,
                "运维",
                true,
                &[
                    &nav_item("nav_servers", "服务器管理")?,
                    &nav_item("nav_cicd", "CI/CD 部署")?,
                    &nav_item("nav_logs", "日志聚合")?,
                ],
            )?;

            let dev_menu = Submenu::with_items(
                handle,
                "开发",
                true,
                &[
                    &nav_item("nav_db", "数据库管理")?,
                    &nav_item("nav_devtools", "开发工具")?,
                    &nav_item("nav_notes", "笔记")?,
                    &nav_item("nav_git", "Git 仓库")?,
                ],
            )?;

            let security_menu = Submenu::with_items(
                handle,
                "安全",
                true,
                &[
                    &nav_item("nav_mfa", "MFA 验证码")?,
                    &nav_item("nav_vpn", "VPN")?,
                ],
            )?;

            let system_menu = Submenu::with_items(
                handle,
                "系统",
                true,
                &[
                    &action_item("about", "关于")?,
                    &PredefinedMenuItem::separator(handle)?,
                    &nav_item("nav_backup", "数据备份")?,
                    &PredefinedMenuItem::separator(handle)?,
                    &action_item("toggle_locale", "切换语言")?,
                    &action_item("toggle_theme", "切换主题")?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::quit(handle, Some("退出"))?,
                ],
            )?;

            let menu = Menu::with_items(
                handle,
                &[
                    &edit_menu,
                    &business_menu,
                    &ops_menu,
                    &dev_menu,
                    &security_menu,
                    &system_menu,
                ],
            )?;
            app.set_menu(menu)?;
            let _ = APP_HANDLE.set(app.handle().clone());

            // Handle menu clicks
            let handle_clone = handle.clone();
            app.on_menu_event(move |_app, event| {
                let id = event.id.0.as_str();
                log::info!("[MENU] Clicked menu item ID: {}", id);
                if id.starts_with("nav_") {
                    // Map backend menu IDs to frontend view names
                    let view = match id {
                        "nav_todo" => "todo",
                        "nav_weekly" => "weekly-report",
                        "nav_projects" => "projects",
                        "nav_accounting" => "accounting",
                        "nav_servers" => "servers",
                        "nav_cicd" => "cicd",
                        "nav_logs" => "log-aggregator",
                        "nav_db" => "database",
                        "nav_devtools" => "devtools",
                        "nav_notes" => "notes",
                        "nav_git" => "git",
                        "nav_mfa" => "mfa",
                        "nav_vpn" => "vpn",
                        "nav_backup" => "data-backup",
                        _ => {
                            log::warn!("[MENU] Unknown nav item: {}", id);
                            return;
                        }
                    };
                    log::info!("[MENU] Emitting 'menu:nav' with view='{}'", view);
                    let _ = handle_clone.emit("menu:nav", view);
                } else {
                    let action = match id {
                        "new_task" => "new-task",
                        "search_tasks" => "search-tasks",
                        "about" => "about",
                        "toggle_locale" => "toggle-locale",
                        "toggle_theme" => "toggle-theme",
                        _ => id,
                    };
                    log::info!("[MENU] Emitting 'menu:{}'", action);
                    let _ = handle_clone.emit(&format!("menu:{}", action), ());
                }
            });

            log::info!("[Main] === SuperTool Tauri ready ===");

            // 启动后台通知检查定时器（每 5 分钟检查到期任务）
            crate::tray_notification::start_notification_timer(app.handle().clone());

            // 启动后台告警检查定时器（每分钟检查服务端口和资源到期）
            crate::commands::alert::start_alert_scheduler(app.handle().clone());

            // 启动时清理临时目录中超过 24 小时的旧文件
            {
                let temp_dir = supertool_core::logic::data_dir::tmp_dir();
                if temp_dir.exists() {
                    let now = std::time::SystemTime::now();
                    let mut deleted = 0u64;
                    if let Ok(entries) = std::fs::read_dir(&temp_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_file() {
                                if let Ok(metadata) = path.metadata() {
                                    if let Ok(modified) = metadata.modified() {
                                        if let Ok(duration) = now.duration_since(modified) {
                                            if duration.as_secs() > 24 * 3600 {
                                                let _ = std::fs::remove_file(&path);
                                                deleted += 1;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if deleted > 0 {
                        log::info!("[Main] 启动时清理临时目录，删除了 {} 个旧文件", deleted);
                    }
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Project commands
            commands::projects::get_projects,
            commands::projects::add_project,
            commands::projects::update_project,
            commands::projects::delete_project,
            commands::projects::get_project_stats,
            commands::projects::get_project_todos,
            // Server commands
            commands::servers::get_all_servers,
            commands::servers::get_server_by_id,
            commands::servers::add_server,
            commands::servers::update_server,
            commands::servers::delete_server,
            commands::servers::get_all_server_groups,
            commands::servers::add_server_group,
            commands::servers::update_server_group,
            commands::servers::delete_server_group,
            commands::servers::test_connection,
            // SFTP commands
            commands::servers::sftp_upload_file,
            commands::servers::sftp_download_file,
            commands::servers::sftp_upload_folder,
            commands::servers::sftp_get_downloads_dir,
            commands::servers::list_sftp_dir,
            commands::servers::open_sftp_file_editor,
            commands::servers::delete_sftp_file,
            commands::servers::sftp_create_dir,
            // Database commands (external DB connections)
            commands::database::db_connect,
            commands::database::db_disconnect,
            commands::database::db_query,
            commands::database::db_get_tables,
            commands::database::db_get_databases,
            // PTY Terminal commands
            commands::terminal::create_terminal,
            commands::terminal::read_terminal,
            commands::terminal::write_to_terminal,
            commands::terminal::resize_terminal,
            commands::terminal::close_terminal,
            commands::terminal::is_terminal_active,
            // Claw Chat commands
            commands::claw_chat::claw_chat_init,
            commands::claw_chat::claw_chat_send,
            commands::claw_chat::claw_chat_close,
            commands::claw_chat::claw_chat_abort,
            commands::claw_chat::claw_chat_list_sessions,
            commands::claw_chat::claw_chat_info,
            commands::claw_chat::claw_read_models_config,
            commands::claw_chat::claw_read_stats,
            commands::claw_compact::claw_chat_compact,
            commands::claw_session::claw_chat_set_model,
            commands::claw_session::claw_chat_fork,
            commands::claw_config::claw_config_get,
            commands::claw_config::claw_config_set,
            commands::claw_config::claw_get_permission_mode,
            commands::claw_config::claw_set_permission_mode,
            // Claw Skills commands
            commands::claw_skills::claw_list_skills,
            commands::claw_skills::claw_get_skill_content,
            commands::claw_agents::claw_list_agents,
            commands::claw_tools::claw_list_mcp_servers,
            commands::claw_tools::claw_list_plugins,
            commands::claw_tools::claw_mcp_health,
            commands::claw_cron::claw_list_cron_jobs,
            commands::claw_cron::claw_create_cron_job,
            commands::claw_cron::claw_delete_cron_job,
            commands::claw_cron::claw_toggle_cron_job,
            commands::claw_profiles::claw_get_profile,
            // SSH commands
            commands::ssh::connect_server,
            commands::ssh::disconnect_server,
            commands::ssh::is_server_connected,
            commands::ssh::get_server_monitor,
            // Todo commands
            commands::todos::get_todos,
            commands::todos::add_todo,
            commands::todos::update_todo,
            commands::todos::delete_todo,
            commands::todos::delete_many,
            commands::todos::update_order,
            commands::todos::create_repeat_instance,
            commands::todos::add_tag,
            commands::todos::get_all_tags,
            commands::todos::delete_tag,
            commands::todos::add_subtask,
            commands::todos::update_subtask,
            commands::todos::delete_subtask,
            commands::todos::get_subtasks_for_todo,
            // Settings commands
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::get_notification_settings,
            commands::settings::set_notification_settings,
            commands::settings::get_app_version,
            commands::settings::notification_test,
            tray_notification::play_sound,
            // Notes commands
            commands::notes::get_all_notes,
            commands::notes::add_note,
            commands::notes::update_note,
            commands::notes::delete_note,
            commands::notes::get_all_note_groups,
            commands::notes::add_note_group,
            commands::notes::update_note_group,
            commands::notes::delete_note_group,
            // Weekly report commands
            commands::weekly::get_weekly_reports,
            commands::weekly::get_weekly_report,
            commands::weekly::save_weekly_report,
            // MFA commands
            commands::mfa::get_all_mfa_secrets,
            commands::mfa::add_mfa_secret,
            commands::mfa::update_mfa_secret,
            commands::mfa::delete_mfa_secret,
            commands::mfa::generate_totp,
            commands::mfa::mfa_parse_uri,
            // Accounting commands
            commands::accounting::get_accounting_records,
            commands::accounting::add_accounting_record,
            commands::accounting::update_accounting_record,
            commands::accounting::delete_accounting_record,
            commands::accounting::get_accounting_categories,
            commands::accounting::add_accounting_category,
            commands::accounting::update_accounting_category,
            commands::accounting::delete_accounting_category,
            commands::accounting::get_accounting_stats,
            commands::accounting::get_accounting_trend,
            commands::accounting::check_budget_alerts,
            commands::accounting::export_accounting_csv,
            commands::accounting::get_budgets,
            commands::accounting::add_budget,
            commands::accounting::update_budget,
            commands::accounting::delete_budget,
            commands::accounting::get_templates,
            commands::accounting::add_template,
            commands::accounting::update_template,
            commands::accounting::delete_template,
            commands::accounting::use_template,
            commands::accounting::upload_accounting_receipt,
            commands::accounting::get_accounting_receipt_file,
            // Log commands
            commands::logs::get_log_presets,
            commands::logs::add_log_preset,
            commands::logs::update_log_preset,
            commands::logs::delete_log_preset,
            commands::logs::log_search,
            commands::logs::log_tail,
            commands::logs::logs_start_stream,
            commands::logs::logs_stop_stream,
            commands::logs::write_system_log,
            // Data backup commands
            commands::data_backup::export_all_data,
            commands::data_backup::export_data,
            commands::data_backup::import_all_data,
            commands::data_backup::import_json,
            commands::data_backup::export_csv,
            // OpenVPN commands
            commands::openvpn::openvpn_get_all,
            commands::openvpn::openvpn_add,
            commands::openvpn::openvpn_delete,
            commands::openvpn::openvpn_connect,
            commands::openvpn::openvpn_retry_with_password,
            commands::openvpn::openvpn_disconnect,
            commands::openvpn::openvpn_get_status,
            commands::openvpn::openvpn_get_logs,
            commands::openvpn::openvpn_check_available,
            commands::openvpn::openvpn_validate_config,
            commands::openvpn::openvpn_get_traffic_stats,
            // WireGuard commands
            commands::wireguard::wireguard_get_all,
            commands::wireguard::wireguard_get_by_id,
            commands::wireguard::wireguard_add,
            commands::wireguard::wireguard_update,
            commands::wireguard::wireguard_delete,
            commands::wireguard::wireguard_connect,
            commands::wireguard::wireguard_disconnect,
            commands::wireguard::wireguard_get_status,
            commands::wireguard::wireguard_generate_keypair,
            commands::wireguard::wireguard_derive_public_key,
            commands::data_backup::get_app_path,
            // CICD commands
            commands::cicd::detect_tool_paths,
            commands::cicd::detect_build_tools,
            commands::cicd::detect_sdk_versions,
            commands::cicd::check_java,
            commands::cicd::check_maven,
            commands::cicd::check_node,
            commands::cicd::scan_project,
            commands::cicd::get_cicd_configs,
            commands::cicd::get_cicd_config_by_id,
            commands::cicd::get_cicd_groups,
            commands::cicd::save_cicd_config,
            commands::cicd::delete_cicd_config,
            commands::cicd::deploy,
            commands::cicd::cancel_deploy,
            commands::cicd::rollback,
            commands::cicd::get_deploy_logs,
            commands::cicd::read_log_file,
            commands::cicd::get_rollback_history,
            commands::cicd::get_deploy_modules,
            commands::cicd::get_deploy_step_logs,
            commands::cicd::get_deploy_history,
            commands::cicd::get_all_deploy_history,
            commands::cicd::scan_project_modules,
            commands::cicd::save_deploy_module,
            commands::cicd::update_deploy_module,
            commands::cicd::delete_deploy_module,
            // Database additional commands
            commands::database::db_get_table_structure,
            commands::database::db_get_table_primary_keys,
            commands::database::db_get_views,
            commands::database::db_get_create_sql,
            commands::database::db_get_table_data,
            commands::database::db_compare_structures,
            commands::database::db_execute_structure_sync,
            commands::database::db_compare_data,
            commands::database::db_execute_data_sync,
            commands::database::db_backup_create,
            commands::database::db_backup_list,
            commands::database::db_backup_restore,
            commands::database::db_backup_delete,
            commands::database::db_redis_databases,
            commands::database::db_redis_keys,
            commands::database::db_redis_keys_tree,
            commands::database::db_redis_keys_by_type,
            commands::database::db_redis_key_info,
            commands::database::db_redis_key_value,
            commands::database::db_redis_set_key,
            commands::database::db_redis_add_key,
            commands::database::db_redis_delete_key,
            commands::database::db_redis_exec,
            commands::database::db_redis_scan_keys,
            commands::database::db_redis_streams,
            commands::database::db_redis_stream_info,
            commands::database::db_redis_stream_messages,
            commands::database::db_redis_stream_add,
            commands::database::db_redis_stream_del,
            commands::database::db_redis_stream_delete,
            commands::database::db_redis_stream_group_create,
            commands::database::db_redis_stream_group_destroy,
            commands::database::db_redis_stream_consumers,
            commands::database::db_redis_stream_pending,
            commands::database::db_redis_stream_claim,
            commands::database::db_redis_stream_ack,
            commands::database::db_redis_stream_retry,
            commands::database::db_redis_stream_trim,
            commands::database::db_redis_zset_range,
            commands::database::db_redis_zset_remove,
            // DDL/DML commands
            commands::database::db_insert_table_row,
            commands::database::db_update_table_row,
            commands::database::db_delete_table_row,
            commands::database::db_get_table_data_filtered,
            commands::database::db_test,
            // LAN commands
            commands::lan::lan_start,
            commands::lan::lan_set_status,
            commands::lan::lan_refresh_discovery,
            commands::lan::lan_get_user_info,
            commands::lan::lan_get_all_unread_counts,
            commands::lan::lan_get_status,
            commands::lan::lan_get_network_info,
            commands::lan::lan_get_receive_path,
            commands::lan::lan_get_peers,
            commands::lan::lan_set_nick_name,
            commands::lan::lan_set_avatar,
            commands::lan::lan_upload_avatar,
            commands::lan::lan_get_avatar_path,
            commands::lan::lan_set_receive_path,
            commands::lan::lan_show_open_dialog_for_dirs,
            commands::lan::lan_send_message,
            commands::lan::lan_send_file,
            commands::lan::lan_get_message_history,
            commands::lan::lan_get_file_transfer_history,
            commands::lan::lan_get_logs,
            commands::lan::lan_mark_messages_read,
            commands::lan::lan_get_unread_count,
            commands::lan::lan_stop,
            commands::lan::lan_get_messages_between,
            commands::lan::lan_assign_task,
            commands::lan::lan_sync_task_status,
            commands::lan::lan_broadcast_message,
            commands::lan::lan_broadcast_task_update,
            commands::lan::lan_broadcast_task_status_change,
            commands::lan::lan_broadcast_task_comment,
            commands::lan::lan_broadcast_collaboration_started,
            commands::lan::lan_broadcast_collaboration_ended,
            commands::lan::lan_screenshot,
            commands::lan::lan_save_temp_file,
            commands::lan::lan_load_local_file_as_base64,
            commands::lan::lan_open_file,
            commands::lan::lan_open_file_folder,
            commands::lan::lan_read_image_file,
            commands::lan::lan_check_network_permission,
            commands::lan::lan_get_permission_status,
            // Settings additional commands
            commands::settings::check_network_permission,
            commands::settings::get_menu_icon,
            commands::settings::get_data_dir,
            commands::settings::set_data_dir,
            // Git commands
            commands::git::get_git_branches,
            commands::git::get_git_commits,
            commands::git::scan_local_repos,
            commands::git::validate_repo_path,
            commands::git::open_in_file_manager,
            commands::git::get_git_commit_detail,
            // Git 状态操作
            commands::git::git_status,
            commands::git::git_current_branch,
            commands::git::git_branches,
            commands::git::git_log,
            commands::git::git_diff,
            // Git 写操作
            commands::git::git_add,
            commands::git::git_reset,
            commands::git::git_commit,
            commands::git::git_checkout,
            commands::git::git_create_branch,
            commands::git::git_delete_branch,
            commands::git::git_merge,
            // Git 远程操作
            commands::git::git_pull,
            commands::git::git_push,
            commands::git::git_force_push,
            commands::git::git_fetch,
            commands::git::git_remotes,
            commands::git::git_discard_changes,
            // Git Stash 操作
            commands::git::git_stash_save,
            commands::git::git_stash_list,
            commands::git::git_stash_apply,
            commands::git::git_stash_pop,
            commands::git::git_stash_drop,
            // Git Tag 操作
            commands::git::git_list_tags,
            commands::git::git_create_tag,
            commands::git::git_delete_tag,
            // Git Rebase 操作
            commands::git::git_rebase,
            commands::git::git_rebase_abort,
            commands::git::git_rebase_continue,
            commands::git::git_rebase_interactive,
            commands::git::git_rebase_todo_list,
            // Git 高级操作
            commands::git::git_file_history,
            commands::git::git_unpushed_commits,
            commands::git::git_cherry_pick,
            commands::git::git_revert,
            commands::git::git_amend_commit,
            commands::git::git_reset_to_commit,
            commands::git::git_file_blame,
            commands::git::git_submodule_list,
            commands::git::git_submodule_init,
            // Git 远程仓库管理
            commands::git::git_add_remote,
            commands::git::git_delete_remote,
            commands::git::git_rename_branch,
            commands::git::git_diff_branches,
            commands::git::git_push_tags,
            commands::git::git_clean,
            commands::git::git_delete_remote_branch,
            commands::git::git_checkout_remote_branch,
            commands::git::git_get_file_at_revision,
            // Git 新增高级命令
            commands::git::git_submodule_update,
            commands::git::git_submodule_update_all,
            commands::git::git_compare_commits,
            commands::git::git_create_patch,
            commands::git::git_apply_patch,
            commands::git::git_raw_command,
            commands::git::get_file_tree,
            commands::git::read_file_content,
            commands::git::save_file_content,
            // Git sync commands
            commands::git_sync::git_sync_status,
            commands::git_sync::git_sync_configure,
            commands::git_sync::git_sync_init,
            commands::git_sync::git_sync_pull,
            commands::git_sync::git_sync_push,
            // Nginx commands
            commands::nginx::get_all_nginx_presets,
            commands::nginx::add_nginx_preset,
            commands::nginx::update_nginx_preset,
            commands::nginx::delete_nginx_preset,
            commands::nginx::fetch_nginx_config,
            commands::nginx::test_nginx_config,
            commands::nginx::test_nginx_config_content,
            commands::nginx::deploy_nginx_config,
            commands::nginx::deploy_nginx_config_decomposed,
            commands::nginx::get_nginx_config_versions,
            commands::nginx::save_nginx_config_version,
            commands::nginx::set_active_nginx_version,
            // NginxServer commands
            commands::nginx::get_servers_by_preset,
            commands::nginx::add_nginx_server,
            commands::nginx::update_nginx_server,
            commands::nginx::delete_nginx_server,
            // NginxLocation commands
            commands::nginx::get_locations_by_server,
            commands::nginx::add_nginx_location,
            commands::nginx::update_nginx_location,
            commands::nginx::delete_nginx_location,
            // NginxUpstream commands
            commands::nginx::get_upstreams_by_preset,
            commands::nginx::add_nginx_upstream,
            commands::nginx::update_nginx_upstream,
            commands::nginx::delete_nginx_upstream,
            // NginxUpstreamServer commands
            commands::nginx::get_upstream_servers,
            commands::nginx::add_nginx_upstream_server,
            commands::nginx::update_nginx_upstream_server,
            commands::nginx::delete_nginx_upstream_server,
            // NginxHttpParam commands
            commands::nginx::get_http_params_by_preset,
            commands::nginx::add_nginx_http_param,
            commands::nginx::update_nginx_http_param,
            commands::nginx::delete_nginx_http_param,
            // NginxStream commands
            commands::nginx::get_streams_by_preset,
            commands::nginx::add_nginx_stream,
            commands::nginx::update_nginx_stream,
            commands::nginx::delete_nginx_stream,
            // NginxCert commands
            commands::nginx::get_certs_by_preset,
            commands::nginx::add_nginx_cert,
            commands::nginx::update_nginx_cert,
            commands::nginx::delete_nginx_cert,
            // NginxTemplate commands
            commands::nginx::get_templates_by_preset,
            commands::nginx::add_nginx_template,
            commands::nginx::update_nginx_template,
            commands::nginx::delete_nginx_template,
            // NginxPreview commands
            commands::nginx::preview_nginx_server,
            // NginxBasicSetting commands
            commands::nginx::get_basic_settings,
            commands::nginx::save_basic_settings,
            // NginxParam commands
            commands::nginx::get_params_by_preset,
            commands::nginx::add_nginx_param,
            commands::nginx::update_nginx_param,
            commands::nginx::delete_nginx_param,
            // NginxDenyAllow commands
            commands::nginx::get_deny_allows_by_preset,
            commands::nginx::add_nginx_deny_allow,
            commands::nginx::update_nginx_deny_allow,
            commands::nginx::delete_nginx_deny_allow,
            // NginxPassword commands
            commands::nginx::get_passwords_by_preset,
            commands::nginx::add_nginx_password,
            commands::nginx::update_nginx_password,
            commands::nginx::delete_nginx_password,
            // Config Generation
            commands::nginx::generate_nginx_config,
            commands::nginx::generate_nginx_config_decomposed,
            commands::nginx::import_nginx_config,
            commands::nginx::get_nginx_preset_stats,
            // Disk cleaner commands
            commands::disk_cleaner::get_home_dir,
            commands::disk_cleaner::get_disk_info,
            commands::disk_cleaner::scan_directory,
            commands::disk_cleaner::scan_by_category,
            commands::disk_cleaner::get_cache_paths,
            commands::disk_cleaner::delete_items,
            commands::disk_cleaner::find_duplicates,
            // CLI installer commands
            commands::cli_installer::install_cli_and_skills,
            commands::cli_installer::check_cli_installed,
            // Alert commands
            commands::alert::get_email_config,
            commands::alert::test_email_config,
            commands::alert::save_email_config,
            commands::alert::get_alert_services,
            commands::alert::add_alert_service,
            commands::alert::update_alert_service,
            commands::alert::delete_alert_service,
            commands::alert::get_alert_resources,
            commands::alert::add_alert_resource,
            commands::alert::update_alert_resource,
            commands::alert::delete_alert_resource,
            commands::alert::get_alert_history,
            commands::alert::trigger_alert_check,
            commands::git_repo::get_git_repos,
            commands::git_repo::add_git_repo,
            commands::git_repo::update_git_repo,
            commands::git_repo::delete_git_repo,
            update_frequent_menu,
            // HTTP Fetch (HTML to Markdown tool)
            commands::fetch::fetch_page_content,
            commands::fetch::convert_html_to_md,
            // Agent commands (direct DB access)
            commands::agent::agent_installed,
            commands::agent::agent_list_messages,
            commands::agent::agent_get_stats,
            commands::agent::agent_list_sessions,
            commands::agent::agent_delete_session,
            commands::agent::agent_rename_session,
            commands::agent::agent_get_session,
            commands::agent::agent_search_sessions,
            commands::agent::agent_get_compression_tip,
            commands::agent::save_temp_file,
            commands::agent::clean_temp_dir,
            // Agent Chat Bridge commands (chat/abort still use HTTP bridge)
            commands::hermes_chat::agent_chat,
            commands::hermes_chat::agent_abort_chat,
            commands::hermes_chat::agent_clear_cache,
            // Hermes gateway management
            commands::hermes_gateway::gateway_status,
            commands::hermes_gateway::gateway_start,
            commands::hermes_gateway::gateway_stop,
            commands::hermes_gateway::gateway_restart,
            // Hermes config management (pure Rust, no Python bridge)
            commands::hermes_chat::agent_check_available,
            commands::hermes_chat::agent_get_models,
            commands::hermes_chat::agent_add_model,
            commands::hermes_chat::agent_remove_model,
            commands::hermes_chat::agent_set_model,
            // Hermes API server configuration
            commands::hermes_config::agent_api_server_status,
            commands::hermes_config::agent_configure_api_server,
            // Hermes Toolset management (platform_toolsets.cli + mcp_servers)
            commands::hermes_config::list_toolsets,
            commands::hermes_config::set_toolset_enabled,
            commands::hermes_config::list_mcp_servers,
            // Hermes Config export/import
            commands::hermes_config::get_hermes_config_info,
            commands::hermes_config::export_hermes_config,
            commands::hermes_config::import_hermes_config,
            commands::hermes_config::hermes_set_config,
            // Hermes Sessions management
            commands::hermes_sessions::sessions_export,
            commands::hermes_sessions::sessions_prune,
            // Hermes Insights
            commands::hermes_insights::get_insights,
            // Hermes Memory management
            commands::hermes_memory::read_memory,
            commands::hermes_memory::add_memory_entry,
            commands::hermes_memory::update_memory_entry,
            commands::hermes_memory::remove_memory_entry,
            commands::hermes_memory::write_user_profile,
            commands::hermes_memory::list_memory_providers,
            commands::hermes_memory::set_memory_provider,
            commands::hermes_memory::read_env_vars,
            commands::hermes_memory::save_env_var,
            // Hermes Skills browser
            commands::hermes_skills::list_installed_skills,
            commands::hermes_skills::list_bundled_skills,
            commands::hermes_skills::get_skill_content,
            commands::hermes_skills::install_skill,
            commands::hermes_skills::uninstall_skill,
            // Hermes Cron Jobs
            commands::hermes_cron::list_cron_jobs,
            commands::hermes_cron::create_cron_job,
            commands::hermes_cron::remove_cron_job,
            commands::hermes_cron::pause_cron_job,
            commands::hermes_cron::resume_cron_job,
            commands::hermes_cron::trigger_cron_job,
            // Hermes Kanban board
            commands::kanban::kanban_list_boards,
            commands::kanban::kanban_get_current_board,
            commands::kanban::kanban_list_tasks,
            commands::kanban::kanban_show_task,
            commands::kanban::kanban_create_task,
            commands::kanban::kanban_assign_task,
            commands::kanban::kanban_reclaim_task,
            commands::kanban::kanban_complete_task,
            commands::kanban::kanban_block_task,
            commands::kanban::kanban_unblock_task,
            commands::kanban::kanban_archive_task,
            commands::kanban::kanban_add_comment,
            commands::kanban::kanban_list_assignees,
            commands::kanban::kanban_get_stats,
            commands::kanban::kanban_create_board,
            commands::kanban::kanban_switch_board,
            commands::kanban::kanban_get_task_log,
            // Hermes Profile management (multi-agent)
            commands::profile::profile_list,
            commands::profile::profile_show,
            commands::profile::profile_create,
            commands::profile::profile_delete,
            commands::profile::profile_use,
            commands::profile::profile_describe,
            commands::profile::profile_get_description,
            commands::profile::profile_set_model,
            commands::profile::profile_install,
            commands::profile::profile_update,
            commands::profile::kanban_dispatch,
            commands::profile::kanban_dispatcher_status,
            commands::profile::kanban_workload,
            // Provider credential management
            commands::provider::list_providers,
            commands::provider::save_provider_credential,
            commands::provider::remove_provider_credential,
            commands::provider::start_oauth_flow,
            commands::provider::poll_oauth_result,
            // Image commands
            commands::image::image_compress,
            commands::image::image_resize,
            commands::image::image_convert,
            commands::image::image_crop,
            commands::image::image_remove_bg,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
