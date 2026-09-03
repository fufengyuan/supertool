//! CLI 安装器：App 启动时检测 /usr/local/bin/stool 与内置 CLI 的版本差异，
//! 不一致时通过 osascript 提权自动安装（覆盖 dmg 安装用户——pkg 的 postinstall 只覆盖默认安装场景）。
//! 同时负责把内置 skills 同步到用户各技能目录（无需提权）。

use serde_json::{json, Value};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// app bundle 内的 CLI 路径（与 build.sh postinstall 期望一致）
fn bundled_cli_path(app: &AppHandle) -> Result<PathBuf, String> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("解析资源目录失败: {}", e))?;
    let p = resource_dir.join("_up_/target/release/stool");
    if !p.exists() {
        return Err(format!("未找到内置 CLI: {}", p.display()));
    }
    Ok(p)
}

/// 读取某路径 stool 的版本字符串（不存在/失败返回空）
fn read_cli_version(path: &PathBuf) -> String {
    if !path.exists() {
        return String::new();
    }
    let out = std::process::Command::new(path)
        .arg("version")
        .output()
        .ok()
        .and_then(|o| (o.status.success()).then_some(o.stdout))
        .unwrap_or_default();
    let s = String::from_utf8_lossy(&out);
    // "SuperTool CLI v6.8.29" → "6.8.29"
    s.split('v')
        .nth(1)
        .map(|v| v.trim().to_string())
        .unwrap_or_default()
}

/// 检测 CLI 版本差异（前端启动时调用）
#[tauri::command(rename_all = "camelCase")]
pub fn check_cli_version(app: tauri::AppHandle) -> Result<Value, String> {
    log::info!("[Tauri CMD] check_cli_version() called");
    let bundled = bundled_cli_path(&app)?;
    let bundled_ver = read_cli_version(&bundled);
    let installed_path = PathBuf::from("/usr/local/bin/stool");
    let installed_ver = read_cli_version(&installed_path);
    Ok(json!({
        "installed": installed_ver,
        "bundled": bundled_ver,
        "needUpdate": !bundled_ver.is_empty() && installed_ver != bundled_ver,
        "bundledPath": bundled.to_string_lossy().to_string(),
    }))
}

/// 提权安装 CLI 到 /usr/local/bin/stool（osascript 管理员权限，会弹系统密码框）
#[tauri::command(rename_all = "camelCase")]
pub async fn install_cli(app: tauri::AppHandle) -> Result<Value, String> {
    log::info!("[Tauri CMD] install_cli() called");
    let cli_src = bundled_cli_path(&app)?;
    // 用 AppleScript `quoted form of` 做 shell 安全引用（路径含空格/引号/反引号/$ 均安全），
    // 避免手工拼接注入。AppleScript 字符串层只需转义 `"` 与 `\`。
    let esc = cli_src
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    let script = format!(
        "set cliPath to \"{}\"\n\
         do shell script \"mkdir -p /usr/local/bin && cp -f \" & quoted form of cliPath & \" /usr/local/bin/stool && chmod 755 /usr/local/bin/stool\" with administrator privileges",
        esc
    );
    let out = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("osascript 启动失败: {}", e))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(format!("安装失败（可能已取消授权）: {}", err));
    }
    let new_ver = read_cli_version(&PathBuf::from("/usr/local/bin/stool"));
    Ok(json!({ "ok": true, "installed": new_ver }))
}

/// 同步内置 skills 到用户各技能目录（App 以用户身份运行，无需提权）
/// 目录：~/.supertool、~/.hermes、~/.claw、~/.trae-cn、~/.trae、~/.workbuddy、
/// ~/.reasonix、~/.claude、~/.codex、~/.agents（zcode 等共享工具）
#[tauri::command(rename_all = "camelCase")]
pub fn sync_user_skills(app: tauri::AppHandle) -> Result<Value, String> {
    log::info!("[Tauri CMD] sync_user_skills() called");
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("解析资源目录失败: {}", e))?;
    let skills_src = resource_dir.join("_up_/skills");
    if !skills_src.is_dir() {
        return Ok(json!({ "ok": true, "copied": 0, "reason": "内置无 skills 目录" }));
    }

    let home = dirs::home_dir().ok_or("解析用户目录失败")?;
    // 技能目录 → 目标根（保留 skill 子目录结构）
    // 覆盖主流编程工具/IDE 的用户级 skill 加载目录：
    // supertool 门户 / Hermes / Claw / Trae(国内+国际) / WorkBuddy / Reasonix / Claude Code / Codex / zcode 等（~/.agents）
    let targets = [
        home.join(".supertool/skills"),
        home.join(".hermes/skills"),
        home.join(".claw/skills"),
        home.join(".trae-cn/skills"),
        home.join(".trae/skills"),
        home.join(".workbuddy/skills"),
        home.join(".reasonix/skills"),
        home.join(".claude/skills"),
        home.join(".codex/skills"),
        home.join(".agents/skills"),
    ];
    let mut copied = 0usize;
    for target_root in &targets {
        if let Ok(entries) = std::fs::read_dir(&skills_src) {
            for entry in entries.flatten() {
                let skill_dir = entry.path();
                if !skill_dir.is_dir() {
                    continue;
                }
                let skill_name = entry.file_name();
                let src_md = skill_dir.join("SKILL.md");
                if !src_md.is_file() {
                    continue;
                }
                let target_dir = target_root.join(&skill_name);
                if std::fs::create_dir_all(&target_dir).is_ok() {
                    if std::fs::copy(&src_md, target_dir.join("SKILL.md")).is_ok() {
                        copied += 1;
                    }
                }
            }
        }
    }
    Ok(json!({ "ok": true, "copied": copied }))
}
