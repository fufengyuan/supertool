use crate::db::{ApiResponse, Database};
use rusqlite::params;

// Types
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct NginxPreset {
    pub id: String,
    pub name: String,
    #[serde(rename = "serverId")]
    pub server_id: String,
    #[serde(rename = "configPath")]
    pub config_path: String,
    pub description: String,
    #[serde(rename = "groupName")]
    pub group_name: String,
    #[serde(rename = "isActive")]
    pub is_active: bool,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct NginxConfigVersion {
    pub id: String,
    #[serde(rename = "presetId")]
    pub preset_id: String,
    pub content: String,
    pub checksum: String,
    pub comment: String,
    #[serde(rename = "isCurrent")]
    pub is_current: bool,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

// ============ Row Mappers ============

pub fn row_to_nginx_preset(row: &rusqlite::Row<'_>) -> rusqlite::Result<NginxPreset> {
    let is_active: i64 = row.get("isActive")?;
    Ok(NginxPreset {
        id: row.get("id")?,
        name: row.get("name")?,
        server_id: row.get("serverId")?,
        config_path: row.get("configPath")?,
        description: row.get("description")?,
        group_name: row.get("groupName")?,
        is_active: is_active == 1,
        created_at: row.get("createdAt")?,
        updated_at: row.get("updatedAt")?,
    })
}

pub fn row_to_nginx_config_version(row: &rusqlite::Row<'_>) -> rusqlite::Result<NginxConfigVersion> {
    let is_current: i64 = row.get("isCurrent")?;
    Ok(NginxConfigVersion {
        id: row.get("id")?,
        preset_id: row.get("presetId")?,
        content: row.get("content")?,
        checksum: row.get("checksum")?,
        comment: row.get("comment")?,
        is_current: is_current == 1,
        created_at: row.get("createdAt")?,
    })
}

// ============ Nginx Preset CRUD ============

pub fn get_all_nginx_presets(db: &mut Database) -> ApiResponse<Vec<NginxPreset>> {
    match db
        .conn()
        .prepare("SELECT * FROM nginx_presets ORDER BY createdAt DESC")
    {
        Ok(mut stmt) => match stmt.query_map([], row_to_nginx_preset) {
            Ok(rows) => {
                let presets: Result<Vec<NginxPreset>, rusqlite::Error> = rows.collect();
                match presets {
                    Ok(list) => ApiResponse::ok(list),
                    Err(e) => ApiResponse::err(format!("Query failed: {}", e)),
                }
            }
            Err(e) => ApiResponse::err(format!("Query failed: {}", e)),
        },
        Err(e) => ApiResponse::err(format!("Prepare failed: {}", e)),
    }
}

pub fn add_nginx_preset(db: &mut Database, preset: NginxPreset) -> ApiResponse<NginxPreset> {
    let result = db.conn_mut().execute(
        "INSERT INTO nginx_presets (id, name, serverId, configPath, description, groupName, isActive, createdAt, updatedAt)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            preset.id,
            preset.name,
            preset.server_id,
            preset.config_path,
            preset.description,
            preset.group_name,
            if preset.is_active { 1 } else { 0 },
            preset.created_at,
            preset.updated_at,
        ],
    );
    match result {
        Ok(_) => ApiResponse::ok(preset),
        Err(e) => ApiResponse::err(format!("Insert failed: {}", e)),
    }
}

pub fn update_nginx_preset(db: &mut Database, preset: NginxPreset) -> ApiResponse<()> {
    let result = db.conn_mut().execute(
        "UPDATE nginx_presets SET name=?2, serverId=?3, configPath=?4, description=?5, groupName=?6, isActive=?7, updatedAt=?8 WHERE id=?1",
        params![
            preset.id,
            preset.name,
            preset.server_id,
            preset.config_path,
            preset.description,
            preset.group_name,
            if preset.is_active { 1 } else { 0 },
            preset.updated_at,
        ],
    );
    match result {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(format!("Update failed: {}", e)),
    }
}

pub fn delete_nginx_preset(db: &mut Database, id: &str) -> ApiResponse<()> {
    // Also delete associated config versions
    let _ = db
        .conn_mut()
        .execute("DELETE FROM nginx_config_versions WHERE presetId = ?1", params![id]);
    match db
        .conn_mut()
        .execute("DELETE FROM nginx_presets WHERE id = ?1", params![id])
    {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(format!("Delete failed: {}", e)),
    }
}

// ============ Nginx Config Version CRUD ============

pub fn get_config_versions(db: &mut Database, preset_id: &str) -> ApiResponse<Vec<NginxConfigVersion>> {
    match db
        .conn()
        .prepare("SELECT * FROM nginx_config_versions WHERE presetId = ?1 ORDER BY createdAt DESC")
    {
        Ok(mut stmt) => match stmt.query_map(params![preset_id], row_to_nginx_config_version) {
            Ok(rows) => {
                let versions: Result<Vec<NginxConfigVersion>, rusqlite::Error> = rows.collect();
                match versions {
                    Ok(list) => ApiResponse::ok(list),
                    Err(e) => ApiResponse::err(format!("Query failed: {}", e)),
                }
            }
            Err(e) => ApiResponse::err(format!("Query failed: {}", e)),
        },
        Err(e) => ApiResponse::err(format!("Prepare failed: {}", e)),
    }
}

pub fn add_config_version(db: &mut Database, version: NginxConfigVersion) -> ApiResponse<NginxConfigVersion> {
    let result = db.conn_mut().execute(
        "INSERT INTO nginx_config_versions (id, presetId, content, checksum, comment, isCurrent, createdAt)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            version.id,
            version.preset_id,
            version.content,
            version.checksum,
            version.comment,
            if version.is_current { 1 } else { 0 },
            version.created_at,
        ],
    );
    match result {
        Ok(_) => ApiResponse::ok(version),
        Err(e) => ApiResponse::err(format!("Insert failed: {}", e)),
    }
}

pub fn set_current_version(db: &mut Database, preset_id: &str, version_id: &str) -> ApiResponse<()> {
    // First, unset all current versions for this preset
    let _ = db.conn_mut().execute(
        "UPDATE nginx_config_versions SET isCurrent = 0 WHERE presetId = ?1",
        params![preset_id],
    );
    // Then set the specified version as current
    match db.conn_mut().execute(
        "UPDATE nginx_config_versions SET isCurrent = 1 WHERE id = ?1 AND presetId = ?2",
        params![version_id, preset_id],
    ) {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(format!("Set current version failed: {}", e)),
    }
}

pub fn get_current_version(db: &mut Database, preset_id: &str) -> ApiResponse<Option<NginxConfigVersion>> {
    match db
        .conn()
        .prepare("SELECT * FROM nginx_config_versions WHERE presetId = ?1 AND isCurrent = 1 LIMIT 1")
    {
        Ok(mut stmt) => match stmt.query_row(params![preset_id], row_to_nginx_config_version) {
            Ok(version) => ApiResponse::ok(Some(version)),
            Err(rusqlite::Error::QueryReturnedNoRows) => ApiResponse::ok(None),
            Err(e) => ApiResponse::err(format!("Query failed: {}", e)),
        },
        Err(e) => ApiResponse::err(format!("Prepare failed: {}", e)),
    }
}
