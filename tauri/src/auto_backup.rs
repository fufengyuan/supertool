//! 自动备份调度：读取设置项（auto_backup_enabled / auto_backup_frequency /
//! auto_backup_time / auto_backup_path），到点静默执行完整备份并轮转。
//! 每 10 分钟 tick 一次；同一天内只成功执行一次（state.last_run_date 去重）。

use std::sync::Arc;
use supertool_core::logic::CoreService;
use tokio::sync::Mutex;

#[derive(Default, Clone)]
pub struct AutoBackupState {
    last_run_date: Arc<Mutex<String>>,
}

pub async fn auto_backup_loop(core: CoreService, state: AutoBackupState) {
    log::info!("[AutoBackup] 调度线程启动（每 10 分钟检查）");
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(600)).await;
        if let Err(e) = tick(&core, &state).await {
            log::warn!("[AutoBackup] 本次检查失败: {}", e);
        }
    }
}

async fn tick(core: &CoreService, state: &AutoBackupState) -> Result<(), String> {
    // 读设置（独立 key，与 DataBackup.vue onMounted 读取的一致）
    let enabled = get_setting_str(core, "auto_backup_enabled").await?;
    if enabled != "true" && enabled != "1" {
        return Ok(());
    }
    let frequency = get_setting_str(core, "auto_backup_frequency").await?; // daily | weekly
    let time = get_setting_str(core, "auto_backup_time").await?; // HH:MM
    let path = get_setting_str(core, "auto_backup_path").await?;

    // 到点判断：当前本地时间 HH:MM >= 设定 HH:MM（错过点也在当天内补跑一次）
    let now_local = chrono::Local::now();
    let today = now_local.format("%Y-%m-%d").to_string();
    let now_hm = now_local.format("%H:%M").to_string();
    let target = if time.trim().is_empty() { "02:00".to_string() } else { time.trim().to_string() };
    if now_hm.as_str() < target.as_str() {
        return Ok(());
    }

    // 周期去重：daily 看今天是否已跑；weekly 看今天是否周一（ISO 周一）且本周未跑
    let is_monday = now_local.format("%u").to_string() == "1"; // %u: 1=Monday
    if frequency == "weekly" && !is_monday {
        return Ok(());
    }

    {
        let mut last = state.last_run_date.lock().await;
        if *last == today {
            return Ok(());
        }
        *last = today.clone();
    }

    log::info!("[AutoBackup] 开始自动备份（{} {}）", frequency, target);
    match core.run_auto_backup(&path, 14).await {
        Ok(p) => {
            log::info!("[AutoBackup] 自动备份完成: {}", p);
            // 记录最近一次成功时间供前端展示
            let _ = set_setting(
                core,
                "auto_backup_last_success",
                &now_local.to_rfc3339(),
            )
            .await;
            let _ = p;
            Ok(())
        }
        Err(e) => {
            log::error!("[AutoBackup] 自动备份失败: {}", e);
            // 失败后清掉今天标记，下个 tick 重试
            let mut last = state.last_run_date.lock().await;
            *last = String::new();
            Err(e)
        }
    }
}

/// 读设置并取字符串（core.get_setting 返回 json!(String)，无值时为空串）
async fn get_setting_str(core: &CoreService, key: &str) -> Result<String, String> {
    let v = core.get_setting(key).await?;
    Ok(v.as_str().unwrap_or("").to_string())
}

async fn set_setting(core: &CoreService, key: &str, value: &str) -> Result<(), String> {
    core.set_setting(key, value).await.map(|_| ())
}
