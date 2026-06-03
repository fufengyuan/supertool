//! Claw Skills management — list installed skills
//!
//! Structure:
//!   ~/.claw/skills/<category>/<skill-name>/SKILL.md   (user skills)
//!
//! Future: workspace/.claw/skills/<category>/<skill-name>/SKILL.md (workspace skills)

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
    /// "user" | "workspace"
    pub source: String,
}

// ── Helpers ────────────────────────────────────────────────────────────────

/// Resolve path to the user's Claw skills directory (~/.claw/skills/)
fn user_skills_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".claw")
        .join("skills")
}

/// Parse YAML frontmatter from SKILL.md content (between --- markers)
fn parse_skill_frontmatter(content: &str) -> (String, String) {
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

fn trim_to_120(s: &str) -> String {
    if s.len() > 120 {
        let truncated: String = s.chars().take(117).collect();
        format!("{}...", truncated)
    } else {
        s.to_string()
    }
}

/// Scan a skills directory (base/<category>/<skill-name>/SKILL.md)
fn scan_skills_dir(base: &PathBuf, source: &str) -> Vec<SkillInfo> {
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

// ── Tauri Commands ─────────────────────────────────────────────────────────

#[tauri::command(rename_all = "camelCase")]
pub fn claw_list_skills() -> Vec<SkillInfo> {
    scan_skills_dir(&user_skills_dir(), "user")
}

#[tauri::command(rename_all = "camelCase")]
pub fn claw_get_skill_content(path: String) -> String {
    let skill_file = PathBuf::from(&path).join("SKILL.md");
    if !skill_file.exists() {
        return String::new();
    }
    std::fs::read_to_string(&skill_file).unwrap_or_default()
}
