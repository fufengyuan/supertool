//! Hermes Skills management — using hermes-config paths for resolution.
//!
//! Scans SKILL.md files in the installed/bundled skills directories.
//! Install/uninstall with direct file operations (no CLI).

use hermes_config::paths;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ── Types ─────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillInfo {
    pub name: String,
    pub category: String,
    pub description: String,
    pub path: String,
    pub source: String, // "installed" | "bundled"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillCliResult {
    pub success: bool,
    pub error: Option<String>,
}

// ── Paths (ultra crate powered) ───────────────────────────────

fn installed_skills_dir() -> PathBuf {
    paths::skills_dir()
}

fn bundled_skills_dir() -> PathBuf {
    paths::hermes_home().join("hermes-agent").join("skills")
}

// ── YAML frontmatter parser ───────────────────────────────────

fn parse_skill_frontmatter(content: &str) -> (String, String) {
    let trimmed = content.trim();
    if !trimmed.starts_with("---") {
        let name = trimmed
            .lines()
            .find(|l| l.starts_with("# "))
            .map(|l| l[2..].trim())
            .unwrap_or("")
            .to_string();
        let desc = trimmed
            .lines()
            .find(|l| !l.starts_with('#') && !l.starts_with("---") && !l.trim().is_empty())
            .unwrap_or("")
            .to_string();
        return (trim_to_120(&name), trim_to_120(&desc));
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

// ── Directory scanner ─────────────────────────────────────────

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
            let Ok(entry) = skill_entry else { continue };
            let skill_path = entry.path();
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

// ── Install / Uninstall (direct file operations) ──────────────

fn skill_category_name_from_identifier(identifier: &str) -> (String, String) {
    if let Some(slash_pos) = identifier.find('/') {
        let cat = &identifier[..slash_pos];
        let name = &identifier[slash_pos + 1..];
        (cat.to_string(), name.to_string())
    } else {
        ("agent".to_string(), identifier.to_string())
    }
}

#[tauri::command(rename_all = "camelCase")]
pub fn install_skill(identifier: String) -> SkillCliResult {
    let (category, skill_name) = skill_category_name_from_identifier(&identifier);
    let skill_dir = installed_skills_dir().join(&category).join(&skill_name);
    let skill_file = skill_dir.join("SKILL.md");

    if skill_file.exists() {
        return SkillCliResult {
            success: false,
            error: Some(format!("Skill '{}' already installed at {}", identifier, skill_dir.display())),
        };
    }

    // URL install: download from URL
    if identifier.starts_with("http://") || identifier.starts_with("https://") {
        let client = match reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
        {
            Ok(c) => c,
            Err(e) => return SkillCliResult {
                success: false,
                error: Some(format!("Failed to create HTTP client: {e}")),
            },
        };
        let resp = match client.get(&identifier).send() {
            Ok(r) => r,
            Err(e) => return SkillCliResult {
                success: false,
                error: Some(format!("HTTP request failed: {e}")),
            },
        };
        if !resp.status().is_success() {
            return SkillCliResult {
                success: false,
                error: Some(format!("HTTP {}: {}", resp.status(), resp.status().canonical_reason().unwrap_or("unknown"))),
            };
        }
        let content = match resp.text() {
            Ok(c) => c,
            Err(e) => return SkillCliResult {
                success: false,
                error: Some(format!("Failed to read response: {e}")),
            },
        };
        if let Err(e) = std::fs::create_dir_all(&skill_dir) {
            return SkillCliResult {
                success: false,
                error: Some(format!("Failed to create skill dir: {e}")),
            };
        }
        match std::fs::write(&skill_file, &content) {
            Ok(_) => SkillCliResult { success: true, error: None },
            Err(e) => SkillCliResult {
                success: false,
                error: Some(format!("Failed to write SKILL.md: {e}")),
            },
        }
    } else {
        // Local install: copy from bundled skills
        let bundled_skill = bundled_skills_dir()
            .join(&category)
            .join(&skill_name)
            .join("SKILL.md");

        if !bundled_skill.exists() {
            return SkillCliResult {
                success: false,
                error: Some(format!(
                    "Skill '{}' not found. Use a URL, or install via 'hermes skills install {}'",
                    identifier, identifier
                )),
            };
        }

        let content = match std::fs::read_to_string(&bundled_skill) {
            Ok(c) => c,
            Err(e) => return SkillCliResult {
                success: false,
                error: Some(format!("Failed to read bundled SKILL.md: {e}")),
            },
        };
        if let Err(e) = std::fs::create_dir_all(&skill_dir) {
            return SkillCliResult {
                success: false,
                error: Some(format!("Failed to create skill dir: {e}")),
            };
        }
        match std::fs::write(&skill_file, &content) {
            Ok(_) => SkillCliResult { success: true, error: None },
            Err(e) => SkillCliResult {
                success: false,
                error: Some(format!("Failed to write SKILL.md: {e}")),
            },
        }
    }
}

#[tauri::command(rename_all = "camelCase")]
pub fn uninstall_skill(identifier: String) -> SkillCliResult {
    let (category, skill_name) = skill_category_name_from_identifier(&identifier);
    let skill_dir = installed_skills_dir().join(&category).join(&skill_name);

    if !skill_dir.exists() {
        return SkillCliResult {
            success: false,
            error: Some(format!("Skill '{}' not found at {}", identifier, skill_dir.display())),
        };
    }

    match std::fs::remove_dir_all(&skill_dir) {
        Ok(_) => SkillCliResult { success: true, error: None },
        Err(e) => SkillCliResult {
            success: false,
            error: Some(format!("Failed to remove skill directory: {e}")),
        },
    }
}

// ── Tauri Commands ────────────────────────────────────────────

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

    for skill in installed {
        let key = format!("{}/{}", skill.category, skill.name);
        if !bundled_keys.contains(&key) && !result.iter().any(|r| r.path == skill.path) {
            let mut s = skill;
            s.source = "installed".to_string();
            result.push(s);
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
