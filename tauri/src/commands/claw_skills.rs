//! Claw Skills management — delegates to `claw-commands` crate
//!
//! Uses `handle_skills_slash_command_json(None, cwd)` from the upstream
//! commands crate for skill discovery, category derivation from path,
//! and source mapping (user → installed, project → bundled).

use commands::handle_skills_slash_command_json;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A skill entry presented to the frontend
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillInfo {
    pub name: String,
    pub category: String,
    pub description: String,
    pub path: String,
    /// "installed" | "bundled"
    pub source: String,
}

// ── Tauri Commands ─────────────────────────────────────────────────────────

#[tauri::command(rename_all = "camelCase")]
pub fn claw_list_skills() -> Result<Vec<SkillInfo>, String> {
    let cwd = std::env::current_dir().map_err(|e| format!("failed to get cwd: {e}"))?;
    let result =
        handle_skills_slash_command_json(None, &cwd).map_err(|e| format!("skills lookup: {e}"))?;

    let skills = result
        .get("skills")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "unexpected skills JSON structure: missing 'skills' array".to_string())?;

    let mut output: Vec<SkillInfo> = Vec::with_capacity(skills.len());
    for skill in skills {
        let name = skill
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let description = skill
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let path = skill
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Derive category from directory structure:
        //   <category>/<skill-name>/SKILL.md → parent dir of parent = category
        let category = if path.contains('/') {
            let p = PathBuf::from(&path);
            p.parent()
                .and_then(std::path::Path::parent)
                .and_then(|gp| gp.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };

        // Map source: user_* → "installed", project_* / other → "bundled"
        let source = skill
            .get("source")
            .and_then(|v| v.get("id"))
            .and_then(|v| v.as_str())
            .map(|id| {
                if id.starts_with("user_") {
                    "installed"
                } else {
                    "bundled"
                }
            })
            .unwrap_or("bundled");

        output.push(SkillInfo {
            name,
            category,
            description,
            path,
            source: source.to_string(),
        });
    }

    output.sort_by(|a, b| a.category.cmp(&b.category).then(a.name.cmp(&b.name)));
    Ok(output)
}

#[tauri::command(rename_all = "camelCase")]
pub fn claw_get_skill_content(path: String) -> String {
    let skill_file = PathBuf::from(&path).join("SKILL.md");
    if !skill_file.exists() {
        return String::new();
    }
    std::fs::read_to_string(&skill_file).unwrap_or_default()
}
