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
fn installed_skills_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".hermes")
        .join("skills")
}

/// Resolve path to the bundled hermes-agent skills
fn bundled_skills_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".hermes")
        .join("hermes-agent")
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
        format!("{}...", &s[..117])
    } else {
        s.to_string()
    }
}

/// Scan a skills directory for installed/bundled skills
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

/// Get the hermes CLI path (reuses pattern from profile.rs)
fn hermes_cli_path() -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── trim_to_120 ──────────────────────────────────────────────────────────

    #[test]
    fn test_trim_to_120_short_string() {
        assert_eq!(trim_to_120("hello"), "hello");
    }

    #[test]
    fn test_trim_to_120_exact_boundary() {
        let s = "a".repeat(120);
        assert_eq!(trim_to_120(&s), s);
    }

    #[test]
    fn test_trim_to_120_overflow() {
        let s = "a".repeat(130);
        let result = trim_to_120(&s);
        assert_eq!(result.len(), 120);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_trim_to_120_empty() {
        assert_eq!(trim_to_120(""), "");
    }

    #[test]
    fn test_trim_to_120_multi_byte_chars() {
        // Use 2-byte unicode chars, fewer than the 120-byte boundary
        let s = "é".repeat(50); // 50 chars × 2 bytes = 100 bytes → no truncation
        let result = trim_to_120(&s);
        assert_eq!(result.len(), 100);
        assert_eq!(result, s);
    }

    // ── parse_skill_frontmatter ──────────────────────────────────────────────

    #[test]
    fn test_parse_frontmatter_with_valid_yaml() {
        let content = r#"---
name: "test-skill"
description: "A test skill"
---
# Content"#;
        let (name, desc) = parse_skill_frontmatter(content);
        assert_eq!(name, "test-skill");
        assert_eq!(desc, "A test skill");
    }

    #[test]
    fn test_parse_frontmatter_with_single_quotes() {
        let content = r#"---
name: 'another-skill'
description: 'Another description'
---"#;
        let (name, desc) = parse_skill_frontmatter(content);
        assert_eq!(name, "another-skill");
        assert_eq!(desc, "Another description");
    }

    #[test]
    fn test_parse_frontmatter_without_quotes() {
        let content = r#"---
name: bare-name
description: bare description
---"#;
        let (name, desc) = parse_skill_frontmatter(content);
        assert_eq!(name, "bare-name");
        assert_eq!(desc, "bare description");
    }

    #[test]
    fn test_parse_frontmatter_no_yaml_delimiters() {
        let content = "# My Skill\n\nThis is a skill description.";
        let (name, desc) = parse_skill_frontmatter(content);
        assert_eq!(name, "My Skill");
        assert_eq!(desc, "This is a skill description.");
    }

    #[test]
    fn test_parse_frontmatter_empty() {
        let (name, desc) = parse_skill_frontmatter("");
        assert_eq!(name, "");
        assert_eq!(desc, "");
    }

    #[test]
    fn test_parse_frontmatter_only_yaml_no_content() {
        let content = "---\nname: lonely\n---";
        let (name, desc) = parse_skill_frontmatter(content);
        assert_eq!(name, "lonely");
        assert_eq!(desc, "");
    }

    #[test]
    fn test_parse_frontmatter_missing_fields() {
        let content = r#"---
other: value
---"#;
        let (name, desc) = parse_skill_frontmatter(content);
        assert_eq!(name, "");
        assert_eq!(desc, "");
    }

    #[test]
    fn test_parse_frontmatter_long_description_is_truncated() {
        let long_desc = "a".repeat(200);
        let content = format!(
            r#"---
name: "skill"
description: "{}"
---"#,
            long_desc
        );
        let (name, desc) = parse_skill_frontmatter(&content);
        assert_eq!(name, "skill");
        assert_eq!(desc.len(), 120);
        assert!(desc.ends_with("..."));
    }

    #[test]
    fn test_parse_frontmatter_fallback_no_heading() {
        let content = "Just a plain text without markdown heading.";
        let (name, desc) = parse_skill_frontmatter(content);
        assert_eq!(name, "");
        assert_eq!(desc, "Just a plain text without markdown heading.");
    }

    #[test]
    fn test_parse_frontmatter_fallback_empty_lines_first() {
        let content = "\n\n\n# Heading\n\nSome description here.";
        let (name, desc) = parse_skill_frontmatter(content);
        assert_eq!(name, "Heading");
        assert_eq!(desc, "Some description here.");
    }

    // ── SkillInfo struct ─────────────────────────────────────────────────────

    #[test]
    fn test_skill_info_serialization() {
        let skill = SkillInfo {
            name: "test".into(),
            category: "devops".into(),
            description: "A test skill".into(),
            path: "/tmp/test".into(),
            source: "installed".into(),
        };
        let json = serde_json::to_string(&skill).unwrap();
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"source\":\"installed\""));
    }

    #[test]
    fn test_skill_info_deserialization() {
        let json = r#"{"name":"test","category":"devops","description":"desc","path":"/p","source":"bundled"}"#;
        let skill: SkillInfo = serde_json::from_str(json).unwrap();
        assert_eq!(skill.name, "test");
        assert_eq!(skill.source, "bundled");
    }

    // ── SkillCliResult struct ────────────────────────────────────────────────

    #[test]
    fn test_skill_cli_result_success() {
        let result = SkillCliResult {
            success: true,
            error: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"success\":true"));
        // error: None serializes as null by default (no skip_serializing_if)
        assert!(json.contains("\"error\":null"));
    }

    #[test]
    fn test_skill_cli_result_failure() {
        let result = SkillCliResult {
            success: false,
            error: Some("command not found".into()),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("\"error\":\"command not found\""));
    }

    // ── Directory resolution ─────────────────────────────────────────────────

    #[test]
    fn test_installed_skills_dir_ends_correctly() {
        let path = installed_skills_dir();
        let path_str = path.to_string_lossy();
        assert!(path_str.ends_with(".hermes/skills"));
    }

    #[test]
    fn test_bundled_skills_dir_ends_correctly() {
        let path = bundled_skills_dir();
        let path_str = path.to_string_lossy();
        assert!(path_str.ends_with(".hermes/hermes-agent/skills"));
    }

    // ── scan_skills_dir with temp dirs ───────────────────────────────────────

    #[test]
    fn test_scan_skills_dir_non_existent() {
        let bogus = PathBuf::from("/tmp/__nonexistent_skills_test__");
        let skills = scan_skills_dir(&bogus, "installed");
        assert!(skills.is_empty());
    }

    #[test]
    fn test_scan_skills_dir_empty_dir() {
        let dir = std::env::temp_dir().join(format!("skills_test_empty_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let skills = scan_skills_dir(&dir, "installed");
        assert!(skills.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_skills_dir_with_valid_skills() {
        let base = std::env::temp_dir().join(format!("skills_test_valid_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);

        // Create: devops/git-skill/SKILL.md
        let devops_dir = base.join("devops/git-skill");
        std::fs::create_dir_all(&devops_dir).unwrap();
        std::fs::write(
            devops_dir.join("SKILL.md"),
            "---\nname: \"Git Helper\"\ndescription: \"Git operations\"\n---\n# Content",
        )
        .unwrap();

        // Create: mlops/train-skill/SKILL.md
        let mlops_dir = base.join("mlops/train-skill");
        std::fs::create_dir_all(&mlops_dir).unwrap();
        std::fs::write(
            mlops_dir.join("SKILL.md"),
            "---\nname: 'Trainer'\ndescription: 'ML training'\n---",
        )
        .unwrap();

        // Create: empty/ dir with no SKILL.md (should be skipped)
        let empty_cat = base.join("empty");
        std::fs::create_dir_all(empty_cat.join("no-skill")).unwrap();

        let skills = scan_skills_dir(&base, "installed");
        assert_eq!(skills.len(), 2);

        // Verify sorting: devops < mlops
        assert_eq!(skills[0].category, "devops");
        assert_eq!(skills[0].name, "Git Helper");
        assert_eq!(skills[0].source, "installed");
        assert_eq!(skills[1].category, "mlops");
        assert_eq!(skills[1].name, "Trainer");

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn test_scan_skills_dir_skips_non_directories() {
        let base = std::env::temp_dir().join(format!("skills_test_files_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        // Place a file directly in the base (should be skipped, not a dir)
        std::fs::write(base.join("somefile.md"), "content").unwrap();
        let skills = scan_skills_dir(&base, "installed");
        assert!(skills.is_empty());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn test_scan_skills_dir_fallback_when_no_frontmatter() {
        let base = std::env::temp_dir().join(format!("skills_test_nofm_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let cat_dir = base.join("tools/my-tool");
        std::fs::create_dir_all(&cat_dir).unwrap();
        std::fs::write(cat_dir.join("SKILL.md"), "# My Tool\n\nUseful description.").unwrap();
        let skills = scan_skills_dir(&base, "bundled");
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "My Tool");
        assert_eq!(skills[0].description, "Useful description.");
        assert_eq!(skills[0].source, "bundled");
        let _ = std::fs::remove_dir_all(&base);
    }

    // ── hermes_cli_path ──────────────────────────────────────────────────────

    #[test]
    fn test_hermes_cli_path_returns_string() {
        let path = hermes_cli_path();
        assert!(!path.is_empty());
    }
}
