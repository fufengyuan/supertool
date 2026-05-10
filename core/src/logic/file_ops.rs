/// File Ops module — extracted from mod.rs
use super::CoreService;
use crate::db::Database;
use serde_json::{json, Value};
use std::path::PathBuf;
use rusqlite::params;
use std::fs;
use std::io::Read;

impl super::CoreService {
    pub async fn get_app_path(&self) -> Result<Value, String> {
        Ok(json!(self.app_dir.to_string_lossy()))
    }
    pub async fn read_file_content(&self, path: &str) -> Result<Value, String> {
        match std::fs::read_to_string(path) {
            Ok(content) => Ok(json!({"success": true, "content": content})),
            Err(e) => Ok(json!({"success": false, "error": e.to_string()})),
        }
    }
    pub async fn read_directory(&self, path: &str) -> Result<Value, String> {
        match std::fs::read_dir(path) {
            Ok(entries) => {
                let mut result = Vec::new();
                for entry in entries {
                    if let Ok(entry) = entry {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                        result.push(json!({"name": name, "isDirectory": is_dir}));
                    }
                }
                Ok(json!({"success": true, "entries": result}))
            }
            Err(e) => Ok(json!({"success": false, "error": e.to_string()})),
        }
    }
}
