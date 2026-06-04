//! Hermes Skills management — browse installed skills, list bundled, install/uninstall
//!
//! Structure:
//!   ~/.hermes/skills/<category>/<skill-name>/SKILL.md   (installed skills)
//!   ~/.hermes/hermes-agent/skills/<category>/<skill-name>/SKILL.md  (bundled skills)
//!
//! Install/Uninstall delegate to `hermes skills install/uninstall` CLI.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

/// A skill entry presented to the frontend
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillInfo {
    pub name: String,
    pub category: String,
    pub description: String,
    pub path: String,
    /// "installed" | "bundled"
    pub source: String,
}

/// Result of install/uninstall operations
#[derive(Debug, Serialize, Deserialize)]
pub struct SkillCliResult {
    pub success: bool,
    pub error: Option<String>,
}

// ── Helpers ────────────────────────────────────────────────────────────────

/// Resolve path to the user's hermes skills directory
pub(crate) fn installed_skills_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".hermes")
        .join("skills")
}

/// Resolve path to the bundled hermes-agent skills
pub(crate) fn bundled_skills_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".hermes")
        .join("hermes-agent")
        .join("skills")
}

/// Parse YAML frontmatter from SKILL.md content (between --- markers)
pub(crate) fn parse_skill_frontmatter(content: &str) -> (String, String) {
    let trimmed = content.trim();
    if !trimmed.starts_with("---") {
        // Fallback: use first heading as name, first paragraph as description
        let name = trim_to_120(
            trimmed
                .lines()
                .find(|l| l.starts_with("# "))
                .map(|l| l[2..].trim())
                .unwrap_or(""),
        );
        let desc = trim_to_120(
            trimmed
                .lines()
                .find(|l| !l.starts_with('#') && !l.starts_with("---") && !l.trim().is_empty())
                .unwrap_or(""),
        );
        return (name, desc);
    }

    let end = trimmed[3..].find("---").map(|i| i + 3);
    let frontmatter = match end {
        Some(i) => &trimmed[3..i],
        None => return (String::new(), String::new()),
    };

    let name = frontmatter
        .lines()
        .find(|l| l.trim().starts_with("name:"))
        .and_then(|l| {
            let val = l.splitn(2, ':').nth(1)?.trim();
            Some(trim_to_120(val.trim_matches('"').trim_matches('\'')))
        })
        .unwrap_or_default();

    let desc = frontmatter
        .lines()
        .find(|l| l.trim().starts_with("description:"))
        .and_then(|l| {
            let val = l.splitn(2, ':').nth(1)?.trim();
            Some(trim_to_120(val.trim_matches('"').trim_matches('\'')))
        })
        .unwrap_or_default();

    (name, desc)
}

pub(crate) fn trim_to_120(s: &str) -> String {
    if s.len() > 120 {
        let truncated: String = s.chars().take(117).collect();
        format!("{}...", truncated)
    } else {
        s.to_string()
    }
}

/// Scan a skills directory for installed/bundled skills
pub(crate) fn scan_skills_dir(base: &PathBuf, source: &str) -> Vec<SkillInfo> {
    let mut skills = Vec::new();
    if !base.exists() {
        return skills;
    }

    let dir = match std::fs::read_dir(base) {
        Ok(d) => d,
        Err(_) => return skills,
    };

    for category_entry in dir.flatten() {
        let cat_path = category_entry.path();
        if !cat_path.is_dir() {
            continue;
        }
        let category = cat_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        for skill_entry in std::fs::read_dir(&cat_path).into_iter().flatten() {
            let skill_entry = match skill_entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let skill_path = skill_entry.path();
            if !skill_path.is_dir() {
                continue;
            }

            let skill_file = skill_path.join("SKILL.md");
            if !skill_file.exists() {
                continue;
            }

            let entry_name = skill_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let (name, description) = match std::fs::read_to_string(&skill_file) {
                Ok(content) => parse_skill_frontmatter(&content),
                Err(_) => (entry_name.clone(), String::new()),
            };

            skills.push(SkillInfo {
                name: if name.is_empty() { entry_name } else { name },
                category: category.clone(),
                description,
                path: skill_path.to_string_lossy().to_string(),
                source: source.to_string(),
            });
        }
    }

    skills.sort_by(|a, b| a.category.cmp(&b.category).then(a.name.cmp(&b.name)));
    skills
}

/// Get the hermes CLI path (reuses pattern from profile.rs)
pub(crate) fn hermes_cli_path() -> String {
    // Prefer ~/.local/bin/hermes
    if let Some(home) = dirs::home_dir() {
        let local = home.join(".local/bin/hermes");
        if local.exists() {
            return local.to_string_lossy().to_string();
        }
    }
    "hermes".to_string()
}

/// Run a command directly (no shell) to prevent command injection
fn run_command_direct(cmd: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Command failed: {}", stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// ── Tauri Commands ─────────────────────────────────────────────────────────

#[tauri::command(rename_all = "camelCase")]
pub fn list_installed_skills() -> Vec<SkillInfo> {
    scan_skills_dir(&installed_skills_dir(), "installed")
}

#[tauri::command(rename_all = "camelCase")]
pub fn list_bundled_skills() -> Vec<SkillInfo> {
    let bundled = scan_skills_dir(&bundled_skills_dir(), "bundled");

    let installed = list_installed_skills();
    let installed_keys: std::collections::HashSet<String> = installed
        .iter()
        .map(|s| format!("{}/{}", s.category, s.name))
        .collect();
    let bundled_keys: std::collections::HashSet<String> = bundled
        .iter()
        .map(|s| format!("{}/{}", s.category, s.name))
        .collect();

    // Merge: bundled first (marked as installed if already present),
    // then any installed skills not in bundled
    let mut result: Vec<SkillInfo> = bundled
        .into_iter()
        .map(|mut s| {
            let key = format!("{}/{}", s.category, s.name);
            if installed_keys.contains(&key) {
                s.source = "installed".to_string();
            }
            s
        })
        .collect();

    // Add installed skills that don't appear in bundled
    for skill in installed {
        let key = format!("{}/{}", skill.category, skill.name);
        if !bundled_keys.contains(&key) {
            if !result.iter().any(|r| r.path == skill.path) {
                let mut s = skill;
                s.source = "installed".to_string();
                result.push(s);
            }
        }
    }

    result.sort_by(|a, b| {
        a.source
            .cmp(&b.source)
            .then(a.category.cmp(&b.category))
            .then(a.name.cmp(&b.name))
    });
    result
}

#[tauri::command(rename_all = "camelCase")]
pub fn get_skill_content(path: String) -> String {
    let skill_file = PathBuf::from(&path).join("SKILL.md");
    if !skill_file.exists() {
        return String::new();
    }
    std::fs::read_to_string(&skill_file).unwrap_or_default()
}

#[tauri::command(rename_all = "camelCase")]
pub fn install_skill(identifier: String) -> SkillCliResult {
    let hermes = hermes_cli_path();
    match run_command_direct(&hermes, &["skills", "install", &identifier, "--yes"]) {
        Ok(_) => SkillCliResult {
            success: true,
            error: None,
        },
        Err(e) => SkillCliResult {
            success: false,
            error: Some(e),
        },
    }
}

#[tauri::command(rename_all = "camelCase")]
pub fn uninstall_skill(identifier: String) -> SkillCliResult {
    let hermes = hermes_cli_path();
    match run_command_direct(&hermes, &["skills", "uninstall", &identifier]) {
        Ok(_) => SkillCliResult {
            success: true,
            error: None,
        },
        Err(e) => SkillCliResult {
            success: false,
            error: Some(e),
        },
    }
}
