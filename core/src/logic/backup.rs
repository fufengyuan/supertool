use serde_json::{json, Value};

impl super::CoreService {
    pub async fn export_all_data(&self) -> Result<Value, String> {
        let todos = self.get_all_todos().await?;
        let projects = self.get_all_projects(true).await?;
        let servers = self.get_all_servers().await?;
        Ok(json!({
            "todos": todos,
            "projects": projects,
            "servers": servers,
        }))
    }
    pub async fn import_all_data(&self, data: Value, mode: &str) -> Result<Value, String> {
        let mut counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        let data_clone = data.clone();
        let mode_owned = mode.to_string();
        if let Some(obj) = data.as_object() {
            // Import todos
            if let Some(items) = obj.get("todos").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    if self.add_todo(item.clone()).await.is_ok() { c += 1; }
                }
                counts.insert("todos".into(), c);
            }
            // Import projects
            if let Some(items) = obj.get("projects").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    if self.add_project(item.clone()).await.is_ok() { c += 1; }
                }
                counts.insert("projects".into(), c);
            }
            // Import servers
            if let Some(items) = obj.get("servers").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    if self.add_server(item.clone()).await.is_ok() { c += 1; }
                }
                counts.insert("servers".into(), c);
            }
            // Import tags
            if let Some(items) = obj.get("tags").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    if !name.is_empty() {
                        if self.add_tag(name).await.is_ok() { c += 1; }
                    }
                }
                counts.insert("tags".into(), c);
            }
            // Import subtasks
            if let Some(items) = obj.get("subtasks").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    if self.add_subtask(item.clone()).await.is_ok() { c += 1; }
                }
                counts.insert("subtasks".into(), c);
            }
            // Import serverGroups
            if let Some(items) = obj.get("serverGroups").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    if self.add_server_group(item.clone()).await.is_ok() { c += 1; }
                }
                counts.insert("serverGroups".into(), c);
            }
            // Import mfaSecrets
            if let Some(items) = obj.get("mfaSecrets").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    if self.add_mfa_secret(item.clone()).await.is_ok() { c += 1; }
                }
                counts.insert("mfaSecrets".into(), c);
            }
            // Import notes
            if let Some(items) = obj.get("notes").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    if self.add_note(item.clone()).await.is_ok() { c += 1; }
                }
                counts.insert("notes".into(), c);
            }
            // Import noteGroups
            if let Some(items) = obj.get("noteGroups").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    if self.add_note_group(item.clone()).await.is_ok() { c += 1; }
                }
                counts.insert("noteGroups".into(), c);
            }
        }
        // CICD data handled separately
        let (cicd_c, cicd_s) = self.import_cicd_data(&data_clone, &mode_owned).await.unwrap_or((0, 0));
        counts.insert("cicdConfigs".into(), cicd_c as u32 + cicd_s as u32);
        Ok(json!(counts))
    }
    // ============ CICD Backup Helpers ============
}