use async_trait::async_trait;
use indexmap::IndexMap;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

use hermes_core::{tool_schema, JsonSchema, ToolError, ToolHandler, ToolSchema};

pub struct CredentialFilesHandler;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialFileMount {
    pub host_path: PathBuf,
    pub container_path: String,
}

fn entry_path(entry: &Value) -> Option<&str> {
    match entry {
        Value::String(path) => Some(path.as_str()),
        Value::Object(map) => map
            .get("path")
            .and_then(Value::as_str)
            .or_else(|| map.get("name").and_then(Value::as_str)),
        _ => None,
    }
}

fn normalize_container_base(container_base: &str) -> &str {
    container_base.trim_end_matches('/')
}

pub fn credential_file_mount_for_entry(
    hermes_home: &Path,
    container_base: &str,
    entry: &Value,
) -> Result<Option<CredentialFileMount>, String> {
    let Some(raw_path) = entry_path(entry).map(str::trim).filter(|p| !p.is_empty()) else {
        return Ok(None);
    };

    let rel = Path::new(raw_path);
    if rel.is_absolute() {
        return Err("credential file path must be relative to HERMES_HOME".to_string());
    }
    if rel.components().any(|component| {
        matches!(
            component,
            std::path::Component::ParentDir
                | std::path::Component::RootDir
                | std::path::Component::Prefix(_)
        )
    }) {
        return Err("credential file path escapes HERMES_HOME".to_string());
    }

    let home = hermes_home
        .canonicalize()
        .map_err(|e| format!("failed to resolve HERMES_HOME: {e}"))?;
    let host_path = home.join(rel);
    if !host_path.exists() {
        return Ok(None);
    }
    let resolved = host_path
        .canonicalize()
        .map_err(|e| format!("failed to resolve credential file: {e}"))?;
    if resolved.strip_prefix(&home).is_err() {
        return Err("credential file symlink escapes HERMES_HOME".to_string());
    }

    let rel_str = rel.to_string_lossy().replace('\\', "/");
    Ok(Some(CredentialFileMount {
        host_path: resolved,
        container_path: format!(
            "{}/{}",
            normalize_container_base(container_base),
            rel_str.trim_start_matches('/')
        ),
    }))
}

pub fn credential_file_mounts(
    hermes_home: &Path,
    container_base: &str,
    entries: &[Value],
) -> (Vec<CredentialFileMount>, Vec<String>, Vec<String>) {
    let mut mounts = Vec::new();
    let mut missing = Vec::new();
    let mut rejected = Vec::new();

    for entry in entries {
        let label = entry_path(entry).unwrap_or("").to_string();
        match credential_file_mount_for_entry(hermes_home, container_base, entry) {
            Ok(Some(mount)) => mounts.push(mount),
            Ok(None) => {
                if !label.is_empty() {
                    missing.push(label);
                }
            }
            Err(_) => {
                if !label.is_empty() {
                    rejected.push(label);
                }
            }
        }
    }

    (mounts, missing, rejected)
}

pub fn path_contains_filtered_hidden_component(path: &Path) -> bool {
    let native_match = path.components().any(|component| {
        matches!(
            component,
            std::path::Component::Normal(part)
                if matches!(part.to_str(), Some(".git" | ".github" | ".hub"))
        )
    });
    native_match
        || path
            .to_string_lossy()
            .split(['/', '\\'])
            .any(|part| matches!(part, ".git" | ".github" | ".hub"))
}

pub fn resolved_path_escapes_root(resolved: &Path, root: &Path) -> bool {
    resolved.strip_prefix(root).is_err()
}

#[async_trait]
impl ToolHandler for CredentialFilesHandler {
    async fn execute(&self, params: Value) -> Result<String, ToolError> {
        let path = params.get("path").and_then(|v| v.as_str()).unwrap_or("");
        if path.is_empty() {
            return Err(ToolError::InvalidParams("Missing 'path'".into()));
        }
        let exists = tokio::fs::metadata(path).await.is_ok();
        Ok(json!({"path":path,"exists":exists}).to_string())
    }

    fn schema(&self) -> ToolSchema {
        let mut props = IndexMap::new();
        props.insert("path".into(), json!({"type":"string"}));
        tool_schema(
            "credential_files",
            "Check credential file existence/metadata.",
            JsonSchema::object(props, vec!["path".into()]),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_mount_accepts_path_name_and_string_entries() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let home = tmp.path().join(".hermes");
        std::fs::create_dir_all(&home).expect("home");
        std::fs::write(home.join("token.json"), "{}").expect("token");
        std::fs::write(home.join("google_token.json"), "{}").expect("google");
        std::fs::write(home.join("secret.key"), "key").expect("key");

        let entries = vec![
            json!({"path": "token.json"}),
            json!({"name": "google_token.json", "description": "OAuth token"}),
            json!("secret.key"),
        ];
        let (mounts, missing, rejected) = credential_file_mounts(&home, "/root/.hermes", &entries);

        assert!(missing.is_empty());
        assert!(rejected.is_empty());
        let container_paths: Vec<&str> = mounts.iter().map(|m| m.container_path.as_str()).collect();
        assert_eq!(
            container_paths,
            vec![
                "/root/.hermes/token.json",
                "/root/.hermes/google_token.json",
                "/root/.hermes/secret.key"
            ]
        );
    }

    #[test]
    fn credential_mount_reports_missing_and_prefers_path_over_name() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let home = tmp.path().join(".hermes");
        std::fs::create_dir_all(&home).expect("home");
        std::fs::write(home.join("real.json"), "{}").expect("real");

        let entries = vec![
            json!({"name": "does_not_exist.json"}),
            json!({"path": "real.json", "name": "wrong.json"}),
        ];
        let (mounts, missing, rejected) = credential_file_mounts(&home, "/root/.hermes", &entries);

        assert_eq!(missing, vec!["does_not_exist.json"]);
        assert!(rejected.is_empty());
        assert_eq!(mounts.len(), 1);
        assert_eq!(mounts[0].container_path, "/root/.hermes/real.json");
    }

    #[test]
    fn credential_mount_rejects_traversal_absolute_and_symlink_escape() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let home = tmp.path().join(".hermes");
        std::fs::create_dir_all(&home).expect("home");
        let outside = tmp.path().join("sensitive.json");
        std::fs::write(&outside, "{}").expect("outside");

        for entry in [
            json!("../sensitive.json"),
            json!("../../.ssh/id_rsa"),
            json!(outside.to_string_lossy().to_string()),
        ] {
            assert!(
                credential_file_mount_for_entry(&home, "/root/.hermes", &entry).is_err(),
                "{entry}"
            );
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(&outside, home.join("evil_link.json")).expect("symlink");
            assert!(credential_file_mount_for_entry(
                &home,
                "/root/.hermes",
                &json!("evil_link.json")
            )
            .is_err());
        }
    }

    #[test]
    fn hidden_component_filter_is_component_based() {
        assert!(path_contains_filtered_hidden_component(Path::new(
            "/home/user/.hermes/skills/.hub/quarantine/evil/SKILL.md"
        )));
        assert!(path_contains_filtered_hidden_component(Path::new(
            "/home/user/.hermes/skills/.git/hooks/SKILL.md"
        )));
        assert!(path_contains_filtered_hidden_component(Path::new(
            r"C:\Users\me\.hermes\skills\.hub\quarantine\evil-skill\SKILL.md"
        )));
        assert!(path_contains_filtered_hidden_component(Path::new(
            "/home/user/.hermes/skills/.github/workflows/SKILL.md"
        )));
        assert!(!path_contains_filtered_hidden_component(Path::new(
            "/home/user/.hermes/skills/.my-hidden-skill/SKILL.md"
        )));
        assert!(!path_contains_filtered_hidden_component(Path::new(
            "/home/user/.hermes/skills/my-hub-skill/SKILL.md"
        )));
    }

    #[test]
    fn resolved_path_escape_check_respects_directory_boundaries() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let skills = tmp.path().join("skills");
        let skill_dir = skills.join("axolotl");
        let sibling = skills.join("axolotl-backdoor");
        std::fs::create_dir_all(&skill_dir).expect("skill");
        std::fs::create_dir_all(&sibling).expect("sibling");

        assert!(!resolved_path_escapes_root(
            &skill_dir.join("utils/helper.py"),
            &skill_dir
        ));
        assert!(resolved_path_escapes_root(
            &sibling.join("evil.py"),
            &skill_dir
        ));
        assert!(!resolved_path_escapes_root(&skill_dir, &skill_dir));
    }
}
