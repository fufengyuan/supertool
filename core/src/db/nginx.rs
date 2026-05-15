use crate::db::{ApiResponse, Database};
use rusqlite::params;

// ============ Existing Types (kept) ============

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
    #[serde(rename = "isActive", default)]
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
    #[serde(rename = "isCurrent", default)]
    pub is_current: bool,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

// ============ New Structured Types ============

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct NginxServer {
    pub id: String,
    #[serde(rename = "presetId")]
    pub preset_id: String,
    #[serde(rename = "proxyType")]
    pub proxy_type: i64,
    pub listen: String,
    pub ip: String,
    pub def: bool,
    pub ipv6: bool,
    #[serde(rename = "proxyProtocol")]
    pub proxy_protocol: bool,
    #[serde(rename = "serverName")]
    pub server_name: String,
    pub ssl: bool,
    #[serde(rename = "certId")]
    pub cert_id: String,
    pub rewrite: bool,
    #[serde(rename = "rewriteListen")]
    pub rewrite_listen: String,
    pub http2: i64,
    pub protocols: String,
    #[serde(rename = "passwordId")]
    pub password_id: String,
    #[serde(rename = "denyAllow")]
    pub deny_allow: i64,
    #[serde(rename = "denyId")]
    pub deny_id: String,
    #[serde(rename = "allowId")]
    pub allow_id: String,
    #[serde(rename = "proxyUpstreamId")]
    pub proxy_upstream_id: String,
    pub descr: String,
    pub enabled: bool,
    pub sort: i64,
    #[serde(rename = "paramJson")]
    pub param_json: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct NginxLocation {
    pub id: String,
    #[serde(rename = "serverId")]
    pub server_id: String,
    pub enabled: bool,
    pub path: String,
    #[serde(rename = "locType")]
    pub loc_type: i64,
    pub value: String,
    #[serde(rename = "upstreamType", default)]
    pub upstream_type: i64,
    #[serde(rename = "upstreamId")]
    pub upstream_id: String,
    #[serde(rename = "upstreamPath")]
    pub upstream_path: String,
    #[serde(rename = "rootPath")]
    pub root_path: String,
    #[serde(rename = "rootPage")]
    pub root_page: String,
    #[serde(rename = "rootType")]
    pub root_type: String,
    pub header: bool,
    pub websocket: bool,
    pub cros: bool,
    #[serde(rename = "headerHost")]
    pub header_host: String,
    #[serde(rename = "returnUrl")]
    pub return_url: String,
    #[serde(rename = "returnPath")]
    pub return_path: bool,
    #[serde(rename = "paramJson")]
    pub param_json: String,
    pub sort: i64,
    pub descr: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct NginxUpstream {
    pub id: String,
    #[serde(rename = "presetId")]
    pub preset_id: String,
    pub name: String,
    #[serde(rename = "proxyType")]
    pub proxy_type: i64,
    pub strategy: String,
    pub descr: String,
    #[serde(rename = "paramJson")]
    pub param_json: String,
    pub sort: i64,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct NginxUpstreamServer {
    pub id: String,
    #[serde(rename = "upstreamId")]
    pub upstream_id: String,
    pub address: String,
    pub port: i64,
    pub weight: i64,
    #[serde(rename = "maxFails")]
    pub max_fails: i64,
    #[serde(rename = "failTimeout")]
    pub fail_timeout: String,
    #[serde(rename = "maxConns")]
    pub max_conns: i64,
    pub backup: bool,
    pub down: bool,
    pub sort: i64,
    pub enabled: bool,
    pub param: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct NginxHttpParam {
    pub id: String,
    #[serde(rename = "presetId")]
    pub preset_id: String,
    pub name: String,
    pub value: String,
    pub enabled: bool,
    pub sort: i64,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct NginxStream {
    pub id: String,
    #[serde(rename = "presetId")]
    pub preset_id: String,
    pub listen: String,
    #[serde(rename = "proxyUpstreamId")]
    pub proxy_upstream_id: String,
    #[serde(rename = "proxyPass")]
    pub proxy_pass: String,
    pub ssl: bool,
    #[serde(rename = "certId")]
    pub cert_id: String,
    pub protocol: String,
    pub descr: String,
    pub enabled: bool,
    #[serde(rename = "paramJson")]
    pub param_json: String,
    pub sort: i64,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct NginxCert {
    pub id: String,
    #[serde(rename = "presetId")]
    pub preset_id: String,
    pub name: String,
    pub pem: String,
    pub key: String,
    pub domain: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct NginxTemplate {
    pub id: String,
    #[serde(rename = "presetId")]
    pub preset_id: String,
    pub name: String,
    pub content: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct NginxBasicSetting {
    pub id: String,
    #[serde(rename = "presetId")]
    pub preset_id: String,
    pub name: String,
    pub value: String,
    pub sort: i64,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct NginxParam {
    pub id: String,
    #[serde(rename = "presetId")]
    pub preset_id: String,
    #[serde(rename = "serverId")]
    pub server_id: String,
    #[serde(rename = "locationId")]
    pub location_id: String,
    #[serde(rename = "upstreamId")]
    pub upstream_id: String,
    pub name: String,
    pub value: String,
    pub position: i64,
    #[serde(rename = "templateValue")]
    pub template_value: String,
    pub sort: i64,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct NginxDenyAllow {
    pub id: String,
    #[serde(rename = "presetId")]
    pub preset_id: String,
    pub name: String,
    pub ip: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct NginxPassword {
    pub id: String,
    #[serde(rename = "presetId")]
    pub preset_id: String,
    pub name: String,
    pub pass: String,
    pub descr: String,
    pub path: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

// ============ Row Mappers (Existing) ============

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

// ============ Row Mappers (New) ============

pub fn row_to_nginx_server(row: &rusqlite::Row<'_>) -> rusqlite::Result<NginxServer> {
    Ok(NginxServer {
        id: row.get("id")?,
        preset_id: row.get("presetId")?,
        proxy_type: row.get("proxyType")?,
        listen: row.get("listen")?,
        ip: row.get("ip")?,
        def: row.get::<_, i64>("def")? == 1,
        ipv6: row.get::<_, i64>("ipv6")? == 1,
        proxy_protocol: row.get::<_, i64>("proxyProtocol")? == 1,
        server_name: row.get("serverName")?,
        ssl: row.get::<_, i64>("ssl")? == 1,
        cert_id: row.get("certId")?,
        rewrite: row.get::<_, i64>("rewrite")? == 1,
        rewrite_listen: row.get("rewriteListen")?,
        http2: row.get("http2")?,
        protocols: row.get("protocols")?,
        password_id: row.get("passwordId")?,
        deny_allow: row.get("denyAllow")?,
        deny_id: row.get("denyId")?,
        allow_id: row.get("allowId")?,
        proxy_upstream_id: row.get("proxyUpstreamId")?,
        descr: row.get("descr")?,
        enabled: row.get::<_, i64>("enabled")? == 1,
        sort: row.get("sort")?,
        param_json: row.get("paramJson")?,
        created_at: row.get("createdAt")?,
        updated_at: row.get("updatedAt")?,
    })
}

pub fn row_to_nginx_location(row: &rusqlite::Row<'_>) -> rusqlite::Result<NginxLocation> {
    Ok(NginxLocation {
        id: row.get("id")?,
        server_id: row.get("serverId")?,
        enabled: row.get::<_, i64>("enabled")? == 1,
        path: row.get("path")?,
        loc_type: row.get("locType")?,
        value: row.get("value")?,
        upstream_type: row.get("upstreamType")?,
        upstream_id: row.get("upstreamId")?,
        upstream_path: row.get("upstreamPath")?,
        root_path: row.get("rootPath")?,
        root_page: row.get("rootPage")?,
        root_type: row.get("rootType")?,
        header: row.get::<_, i64>("header")? == 1,
        websocket: row.get::<_, i64>("websocket")? == 1,
        cros: row.get::<_, i64>("cros")? == 1,
        header_host: row.get("headerHost")?,
        return_url: row.get("returnUrl")?,
        return_path: row.get::<_, i64>("returnPath")? == 1,
        param_json: row.get("paramJson")?,
        sort: row.get("sort")?,
        descr: row.get("descr")?,
        created_at: row.get("createdAt")?,
    })
}

pub fn row_to_nginx_upstream(row: &rusqlite::Row<'_>) -> rusqlite::Result<NginxUpstream> {
    Ok(NginxUpstream {
        id: row.get("id")?,
        preset_id: row.get("presetId")?,
        name: row.get("name")?,
        proxy_type: row.get("proxyType")?,
        strategy: row.get("strategy")?,
        descr: row.get("descr")?,
        param_json: row.get("paramJson")?,
        sort: row.get("sort")?,
        created_at: row.get("createdAt")?,
        updated_at: row.get("updatedAt")?,
    })
}

pub fn row_to_nginx_upstream_server(row: &rusqlite::Row<'_>) -> rusqlite::Result<NginxUpstreamServer> {
    Ok(NginxUpstreamServer {
        id: row.get("id")?,
        upstream_id: row.get("upstreamId")?,
        address: row.get("address")?,
        port: row.get("port")?,
        weight: row.get("weight")?,
        max_fails: row.get("maxFails")?,
        fail_timeout: row.get("failTimeout")?,
        max_conns: row.get("maxConns")?,
        backup: row.get::<_, i64>("backup")? == 1,
        down: row.get::<_, i64>("down")? == 1,
        sort: row.get("sort")?,
        enabled: row.get::<_, i64>("enabled")? == 1,
        param: row.get("param")?,
    })
}

pub fn row_to_nginx_http_param(row: &rusqlite::Row<'_>) -> rusqlite::Result<NginxHttpParam> {
    Ok(NginxHttpParam {
        id: row.get("id")?,
        preset_id: row.get("presetId")?,
        name: row.get("name")?,
        value: row.get("value")?,
        enabled: row.get::<_, i64>("enabled")? == 1,
        sort: row.get("sort")?,
        created_at: row.get("createdAt")?,
    })
}

pub fn row_to_nginx_stream(row: &rusqlite::Row<'_>) -> rusqlite::Result<NginxStream> {
    Ok(NginxStream {
        id: row.get("id")?,
        preset_id: row.get("presetId")?,
        listen: row.get("listen")?,
        proxy_upstream_id: row.get("proxyUpstreamId")?,
        proxy_pass: row.get("proxyPass")?,
        ssl: row.get::<_, i64>("ssl")? == 1,
        cert_id: row.get("certId")?,
        protocol: row.get("protocol")?,
        descr: row.get("descr")?,
        enabled: row.get::<_, i64>("enabled")? == 1,
        sort: row.get("sort")?,
        param_json: row.get("paramJson")?,
        created_at: row.get("createdAt")?,
        updated_at: row.get("updatedAt")?,
    })
}

pub fn row_to_nginx_cert(row: &rusqlite::Row<'_>) -> rusqlite::Result<NginxCert> {
    Ok(NginxCert {
        id: row.get("id")?,
        preset_id: row.get("presetId")?,
        name: row.get("name")?,
        pem: row.get("pem")?,
        key: row.get("key")?,
        domain: row.get("domain")?,
        created_at: row.get("createdAt")?,
    })
}

pub fn row_to_nginx_template(row: &rusqlite::Row<'_>) -> rusqlite::Result<NginxTemplate> {
    Ok(NginxTemplate {
        id: row.get("id")?,
        preset_id: row.get("presetId")?,
        name: row.get("name")?,
        content: row.get("content")?,
        created_at: row.get("createdAt")?,
    })
}

pub fn row_to_nginx_basic_setting(row: &rusqlite::Row<'_>) -> rusqlite::Result<NginxBasicSetting> {
    Ok(NginxBasicSetting {
        id: row.get("id")?,
        preset_id: row.get("presetId")?,
        name: row.get("name")?,
        value: row.get("value")?,
        sort: row.get("sort")?,
        created_at: row.get("createdAt")?,
    })
}

pub fn row_to_nginx_param(row: &rusqlite::Row<'_>) -> rusqlite::Result<NginxParam> {
    Ok(NginxParam {
        id: row.get("id")?,
        preset_id: row.get("presetId")?,
        server_id: row.get("serverId")?,
        location_id: row.get("locationId")?,
        upstream_id: row.get("upstreamId")?,
        name: row.get("name")?,
        value: row.get("value")?,
        position: row.get("position")?,
        template_value: row.get("templateValue")?,
        sort: row.get("sort")?,
        created_at: row.get("createdAt")?,
    })
}

pub fn row_to_nginx_deny_allow(row: &rusqlite::Row<'_>) -> rusqlite::Result<NginxDenyAllow> {
    Ok(NginxDenyAllow {
        id: row.get("id")?,
        preset_id: row.get("presetId")?,
        name: row.get("name")?,
        ip: row.get("ip")?,
        created_at: row.get("createdAt")?,
    })
}

pub fn row_to_nginx_password(row: &rusqlite::Row<'_>) -> rusqlite::Result<NginxPassword> {
    Ok(NginxPassword {
        id: row.get("id")?,
        preset_id: row.get("presetId")?,
        name: row.get("name")?,
        pass: row.get("pass")?,
        descr: row.get("descr")?,
        path: row.get("path")?,
        created_at: row.get("createdAt")?,
    })
}

// ============ Generic query helpers to reduce repetition ============

fn query_all<T, F>(conn: &rusqlite::Connection, sql: &str, params: &[&dyn rusqlite::types::ToSql], mapper: F) -> rusqlite::Result<Vec<T>>
where F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params, mapper)?;
    rows.collect()
}

fn query_one<T, F>(conn: &rusqlite::Connection, sql: &str, params: &[&dyn rusqlite::types::ToSql], mapper: F) -> rusqlite::Result<Option<T>>
where F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T> {
    let mut stmt = conn.prepare(sql)?;
    match stmt.query_row(params, mapper) {
        Ok(val) => Ok(Some(val)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

fn get_all_by_preset<T, F>(conn: &rusqlite::Connection, table: &str, preset_id: &str, mapper: F) -> rusqlite::Result<Vec<T>>
where F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T> {
    let sql = format!("SELECT * FROM {} WHERE presetId = ?1 ORDER BY sort ASC, createdAt DESC", table);
    query_all(conn, &sql, rusqlite::params![preset_id], mapper)
}

fn get_all_by_fk<T, F>(conn: &rusqlite::Connection, table: &str, column: &str, fk_id: &str, mapper: F) -> rusqlite::Result<Vec<T>>
where F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T> {
    let sql = format!("SELECT * FROM {} WHERE {} = ?1 ORDER BY sort ASC", table, column);
    query_all(conn, &sql, rusqlite::params![fk_id], mapper)
}

fn get_by_id_internal<T, F>(conn: &rusqlite::Connection, table: &str, id: &str, mapper: F) -> rusqlite::Result<Option<T>>
where F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T> {
    let sql = format!("SELECT * FROM {} WHERE id = ?1", table);
    query_one(conn, &sql, rusqlite::params![id], mapper)
}

fn add_row(conn: &rusqlite::Connection, table: &str, id: &str, preset_or_fk: &str, fk_column: &str) -> rusqlite::Result<()> {
    // Dummy — each model handles its own INSERT because columns differ
    // This helper is just for standardized query patterns
    let _ = (conn, table, id, preset_or_fk, fk_column);
    Ok(())
}

fn delete_by_id(conn: &rusqlite::Connection, table: &str, id: &str) -> rusqlite::Result<()> {
    let sql = format!("DELETE FROM {} WHERE id = ?1", table);
    conn.execute(&sql, params![id])?;
    Ok(())
}

fn delete_by_fk(conn: &rusqlite::Connection, table: &str, column: &str, fk_id: &str) -> rusqlite::Result<()> {
    let sql = format!("DELETE FROM {} WHERE {} = ?1", table, column);
    conn.execute(&sql, params![fk_id])?;
    Ok(())
}

fn bool_int(b: bool) -> i64 { if b { 1 } else { 0 } }

// ============ Existing CRUD ============

pub fn get_all_nginx_presets(db: &mut Database) -> ApiResponse<Vec<NginxPreset>> {
    match db.conn().prepare("SELECT * FROM nginx_presets ORDER BY createdAt DESC") {
        Ok(mut stmt) => match stmt.query_map([], row_to_nginx_preset) {
            Ok(rows) => {
                let presets: rusqlite::Result<Vec<NginxPreset>> = rows.collect();
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
        params![preset.id, preset.name, preset.server_id, preset.config_path,
                preset.description, preset.group_name,
                if preset.is_active { 1 } else { 0 },
                preset.created_at, preset.updated_at],
    );
    match result {
        Ok(_) => ApiResponse::ok(preset),
        Err(e) => ApiResponse::err(format!("Insert failed: {}", e)),
    }
}

pub fn update_nginx_preset(db: &mut Database, preset: NginxPreset) -> ApiResponse<()> {
    let result = db.conn_mut().execute(
        "UPDATE nginx_presets SET name=?2, serverId=?3, configPath=?4, description=?5, groupName=?6, isActive=?7, updatedAt=?8 WHERE id=?1",
        params![preset.id, preset.name, preset.server_id, preset.config_path,
                preset.description, preset.group_name,
                if preset.is_active { 1 } else { 0 }, preset.updated_at],
    );
    match result {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(format!("Update failed: {}", e)),
    }
}

pub fn delete_nginx_preset(db: &mut Database, id: &str) -> ApiResponse<()> {
    let conn = db.conn_mut();
    let tx = match conn.transaction() {
        Ok(tx) => tx,
        Err(e) => return ApiResponse::err(format!("Transaction failed: {}", e)),
    };
    // Delete all related records
    for table in &["nginx_config_versions", "nginx_basic_settings", "nginx_servers",
                    "nginx_upstreams", "nginx_http_params", "nginx_certs",
                    "nginx_templates", "nginx_streams", "nginx_params",
                    "nginx_deny_allows", "nginx_passwords"] {
        let sql = format!("DELETE FROM {} WHERE presetId = ?1", table);
        if let Err(e) = tx.execute(&sql, params![id]) {
            let _ = tx.rollback();
            return ApiResponse::err(format!("Delete {} failed: {}", table, e));
        }
    }
    match tx.execute("DELETE FROM nginx_presets WHERE id = ?1", params![id]) {
        Ok(_) => {
            if let Err(e) = tx.commit() {
                return ApiResponse::err(format!("Commit failed: {}", e));
            }
            ApiResponse::ok(())
        }
        Err(e) => {
            let _ = tx.rollback();
            ApiResponse::err(format!("Delete failed: {}", e))
        }
    }
}

pub fn get_config_versions(db: &mut Database, preset_id: &str) -> ApiResponse<Vec<NginxConfigVersion>> {
    match db.conn().prepare("SELECT * FROM nginx_config_versions WHERE presetId = ?1 ORDER BY createdAt DESC") {
        Ok(mut stmt) => match stmt.query_map(params![preset_id], row_to_nginx_config_version) {
            Ok(rows) => {
                let versions: rusqlite::Result<Vec<NginxConfigVersion>> = rows.collect();
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
        params![version.id, version.preset_id, version.content, version.checksum,
                version.comment, if version.is_current { 1 } else { 0 }, version.created_at],
    );
    match result {
        Ok(_) => ApiResponse::ok(version),
        Err(e) => ApiResponse::err(format!("Insert failed: {}", e)),
    }
}

pub fn set_current_version(db: &mut Database, preset_id: &str, version_id: &str) -> ApiResponse<()> {
    let conn = db.conn_mut();
    let tx = match conn.transaction() {
        Ok(tx) => tx,
        Err(e) => return ApiResponse::err(format!("Transaction failed: {}", e)),
    };
    if let Err(e) = tx.execute("UPDATE nginx_config_versions SET isCurrent = 0 WHERE presetId = ?1", params![preset_id]) {
        let _ = tx.rollback();
        return ApiResponse::err(format!("Unset current failed: {}", e));
    }
    match tx.execute("UPDATE nginx_config_versions SET isCurrent = 1 WHERE id = ?1 AND presetId = ?2", params![version_id, preset_id]) {
        Ok(_) => {
            if let Err(e) = tx.commit() { return ApiResponse::err(format!("Commit failed: {}", e)); }
            ApiResponse::ok(())
        }
        Err(e) => {
            let _ = tx.rollback();
            ApiResponse::err(format!("Set current version failed: {}", e))
        }
    }
}

// ============ NginxServer CRUD ============

pub fn get_servers_by_preset(conn: &rusqlite::Connection, preset_id: &str) -> rusqlite::Result<Vec<NginxServer>> {
    get_all_by_preset(conn, "nginx_servers", preset_id, row_to_nginx_server)
}

pub fn get_server_by_id(conn: &rusqlite::Connection, id: &str) -> rusqlite::Result<Option<NginxServer>> {
    get_by_id_internal(conn, "nginx_servers", id, row_to_nginx_server)
}

pub fn add_nginx_server(conn: &rusqlite::Connection, s: &NginxServer) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO nginx_servers (id, presetId, proxyType, listen, ip, def, ipv6, proxyProtocol,
         serverName, ssl, certId, rewrite, rewriteListen, http2, protocols,
         passwordId, denyAllow, denyId, allowId, proxyUpstreamId,
         descr, enabled, sort, paramJson, createdAt, updatedAt)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,
                 ?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26)",
        params![s.id, s.preset_id, s.proxy_type, s.listen, s.ip,
                bool_int(s.def), bool_int(s.ipv6), bool_int(s.proxy_protocol),
                s.server_name, bool_int(s.ssl), s.cert_id, bool_int(s.rewrite),
                s.rewrite_listen, s.http2, s.protocols,
                s.password_id, s.deny_allow, s.deny_id, s.allow_id, s.proxy_upstream_id,
                s.descr, bool_int(s.enabled), s.sort, s.param_json,
                s.created_at, s.updated_at],
    )?;
    Ok(())
}

pub fn update_nginx_server(conn: &rusqlite::Connection, s: &NginxServer) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE nginx_servers SET proxyType=?2, listen=?3, ip=?4, def=?5, ipv6=?6, proxyProtocol=?7,
         serverName=?8, ssl=?9, certId=?10, rewrite=?11, rewriteListen=?12, http2=?13, protocols=?14,
         passwordId=?15, denyAllow=?16, denyId=?17, allowId=?18, proxyUpstreamId=?19,
         descr=?20, enabled=?21, sort=?22, paramJson=?23, updatedAt=?24
         WHERE id=?1",
        params![s.id, s.proxy_type, s.listen, s.ip,
                bool_int(s.def), bool_int(s.ipv6), bool_int(s.proxy_protocol),
                s.server_name, bool_int(s.ssl), s.cert_id, bool_int(s.rewrite),
                s.rewrite_listen, s.http2, s.protocols,
                s.password_id, s.deny_allow, s.deny_id, s.allow_id, s.proxy_upstream_id,
                s.descr, bool_int(s.enabled), s.sort, s.param_json, s.updated_at],
    )?;
    Ok(())
}

pub fn delete_nginx_server(conn: &rusqlite::Connection, id: &str) -> rusqlite::Result<()> {
    delete_by_fk(conn, "nginx_locations", "serverId", id)?;
    delete_by_id(conn, "nginx_servers", id)
}

// ============ NginxLocation CRUD ============

pub fn get_locations_by_server(conn: &rusqlite::Connection, server_id: &str) -> rusqlite::Result<Vec<NginxLocation>> {
    get_all_by_fk(conn, "nginx_locations", "serverId", server_id, row_to_nginx_location)
}

pub fn add_nginx_location(conn: &rusqlite::Connection, loc: &NginxLocation) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO nginx_locations (id, serverId, enabled, path, locType, value,
         upstreamType, upstreamId, upstreamPath, rootPath, rootPage, rootType,
         header, websocket, cros, headerHost, returnUrl, returnPath, paramJson, sort, descr, createdAt)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22)",
        params![loc.id, loc.server_id, bool_int(loc.enabled), loc.path, loc.loc_type, loc.value,
                loc.upstream_type, loc.upstream_id, loc.upstream_path,
                loc.root_path, loc.root_page, loc.root_type,
                bool_int(loc.header), bool_int(loc.websocket), bool_int(loc.cros),
                loc.header_host, loc.return_url, bool_int(loc.return_path),
                loc.param_json, loc.sort, loc.descr, loc.created_at],
    )?;
    Ok(())
}

pub fn update_nginx_location(conn: &rusqlite::Connection, loc: &NginxLocation) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE nginx_locations SET enabled=?2, path=?3, locType=?4, value=?5,
          upstreamType=?6, upstreamId=?7, upstreamPath=?8, rootPath=?9, rootPage=?10, rootType=?11,
          header=?12, websocket=?13, cros=?14, headerHost=?15, returnUrl=?16, returnPath=?17, paramJson=?18, sort=?19, descr=?20
         WHERE id=?1",
        params![loc.id, bool_int(loc.enabled), loc.path, loc.loc_type, loc.value,
                loc.upstream_type, loc.upstream_id, loc.upstream_path,
                loc.root_path, loc.root_page, loc.root_type,
                bool_int(loc.header), bool_int(loc.websocket), bool_int(loc.cros),
                loc.header_host, loc.return_url, bool_int(loc.return_path),
                loc.param_json, loc.sort, loc.descr],
    )?;
    Ok(())
}

pub fn delete_nginx_location(conn: &rusqlite::Connection, id: &str) -> rusqlite::Result<()> {
    delete_by_id(conn, "nginx_locations", id)
}

// ============ NginxUpstream CRUD ============

pub fn get_upstreams_by_preset(conn: &rusqlite::Connection, preset_id: &str) -> rusqlite::Result<Vec<NginxUpstream>> {
    get_all_by_preset(conn, "nginx_upstreams", preset_id, row_to_nginx_upstream)
}

pub fn get_upstream_by_id(conn: &rusqlite::Connection, id: &str) -> rusqlite::Result<Option<NginxUpstream>> {
    get_by_id_internal(conn, "nginx_upstreams", id, row_to_nginx_upstream)
}

pub fn add_nginx_upstream(conn: &rusqlite::Connection, u: &NginxUpstream) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO nginx_upstreams (id, presetId, name, proxyType, strategy, descr, paramJson, sort, createdAt, updatedAt)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        params![u.id, u.preset_id, u.name, u.proxy_type, u.strategy, u.descr, u.param_json, u.sort, u.created_at, u.updated_at],
    )?;
    Ok(())
}

pub fn update_nginx_upstream(conn: &rusqlite::Connection, u: &NginxUpstream) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE nginx_upstreams SET name=?2, proxyType=?3, strategy=?4, descr=?5, paramJson=?6, sort=?7, updatedAt=?8 WHERE id=?1",
        params![u.id, u.name, u.proxy_type, u.strategy, u.descr, u.param_json, u.sort, u.updated_at],
    )?;
    Ok(())
}

pub fn delete_nginx_upstream(conn: &rusqlite::Connection, id: &str) -> rusqlite::Result<()> {
    delete_by_fk(conn, "nginx_upstream_servers", "upstreamId", id)?;
    delete_by_id(conn, "nginx_upstreams", id)
}

// ============ NginxUpstreamServer CRUD ============

pub fn get_upstream_servers(conn: &rusqlite::Connection, upstream_id: &str) -> rusqlite::Result<Vec<NginxUpstreamServer>> {
    get_all_by_fk(conn, "nginx_upstream_servers", "upstreamId", upstream_id, row_to_nginx_upstream_server)
}

pub fn add_nginx_upstream_server(conn: &rusqlite::Connection, s: &NginxUpstreamServer) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO nginx_upstream_servers (id, upstreamId, address, port, weight, maxFails, failTimeout, maxConns, backup, down, sort, enabled, param)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
        params![s.id, s.upstream_id, s.address, s.port, s.weight, s.max_fails, s.fail_timeout,
                s.max_conns, bool_int(s.backup), bool_int(s.down), s.sort, bool_int(s.enabled), s.param],
    )?;
    Ok(())
}

pub fn update_nginx_upstream_server(conn: &rusqlite::Connection, s: &NginxUpstreamServer) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE nginx_upstream_servers SET address=?2, port=?3, weight=?4, maxFails=?5, failTimeout=?6, maxConns=?7, backup=?8, down=?9, sort=?10, enabled=?11, param=?12 WHERE id=?1",
        params![s.id, s.address, s.port, s.weight, s.max_fails, s.fail_timeout, s.max_conns, bool_int(s.backup), bool_int(s.down), s.sort, bool_int(s.enabled), s.param],
    )?;
    Ok(())
}

pub fn delete_nginx_upstream_server(conn: &rusqlite::Connection, id: &str) -> rusqlite::Result<()> {
    delete_by_id(conn, "nginx_upstream_servers", id)
}

// ============ NginxHttpParam CRUD ============

pub fn get_http_params_by_preset(conn: &rusqlite::Connection, preset_id: &str) -> rusqlite::Result<Vec<NginxHttpParam>> {
    get_all_by_preset(conn, "nginx_http_params", preset_id, row_to_nginx_http_param)
}

pub fn add_nginx_http_param(conn: &rusqlite::Connection, p: &NginxHttpParam) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO nginx_http_params (id, presetId, name, value, enabled, sort, createdAt)
         VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![p.id, p.preset_id, p.name, p.value, bool_int(p.enabled), p.sort, p.created_at],
    )?;
    Ok(())
}

pub fn update_nginx_http_param(conn: &rusqlite::Connection, p: &NginxHttpParam) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE nginx_http_params SET name=?2, value=?3, enabled=?4, sort=?5 WHERE id=?1",
        params![p.id, p.name, p.value, bool_int(p.enabled), p.sort],
    )?;
    Ok(())
}

pub fn delete_nginx_http_param(conn: &rusqlite::Connection, id: &str) -> rusqlite::Result<()> {
    delete_by_id(conn, "nginx_http_params", id)
}

// ============ NginxStream CRUD ============

pub fn get_streams_by_preset(conn: &rusqlite::Connection, preset_id: &str) -> rusqlite::Result<Vec<NginxStream>> {
    get_all_by_preset(conn, "nginx_streams", preset_id, row_to_nginx_stream)
}

pub fn add_nginx_stream(conn: &rusqlite::Connection, s: &NginxStream) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO nginx_streams (id, presetId, listen, proxyUpstreamId, proxyPass,
         ssl, certId, protocol, descr, enabled, sort, paramJson, createdAt, updatedAt)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
        params![s.id, s.preset_id, s.listen, s.proxy_upstream_id, s.proxy_pass,
                bool_int(s.ssl), s.cert_id, s.protocol, s.descr,
                bool_int(s.enabled), s.sort, s.param_json, s.created_at, s.updated_at],
    )?;
    Ok(())
}

pub fn update_nginx_stream(conn: &rusqlite::Connection, s: &NginxStream) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE nginx_streams SET listen=?2, proxyUpstreamId=?3, proxyPass=?4,
         ssl=?5, certId=?6, protocol=?7, descr=?8, enabled=?9, sort=?10, paramJson=?11, updatedAt=?12
         WHERE id=?1",
        params![s.id, s.listen, s.proxy_upstream_id, s.proxy_pass,
                bool_int(s.ssl), s.cert_id, s.protocol, s.descr,
                bool_int(s.enabled), s.sort, s.param_json, s.updated_at],
    )?;
    Ok(())
}

pub fn delete_nginx_stream(conn: &rusqlite::Connection, id: &str) -> rusqlite::Result<()> {
    delete_by_id(conn, "nginx_streams", id)
}

// ============ NginxCert CRUD ============

pub fn get_certs_by_preset(conn: &rusqlite::Connection, preset_id: &str) -> rusqlite::Result<Vec<NginxCert>> {
    get_all_by_preset(conn, "nginx_certs", preset_id, row_to_nginx_cert)
}

pub fn add_nginx_cert(conn: &rusqlite::Connection, c: &NginxCert) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO nginx_certs (id, presetId, name, pem, key, domain, createdAt)
         VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![c.id, c.preset_id, c.name, c.pem, c.key, c.domain, c.created_at],
    )?;
    Ok(())
}

pub fn update_nginx_cert(conn: &rusqlite::Connection, c: &NginxCert) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE nginx_certs SET name=?2, pem=?3, key=?4, domain=?5 WHERE id=?1",
        params![c.id, c.name, c.pem, c.key, c.domain],
    )?;
    Ok(())
}

pub fn delete_nginx_cert(conn: &rusqlite::Connection, id: &str) -> rusqlite::Result<()> {
    delete_by_id(conn, "nginx_certs", id)
}

// ============ NginxTemplate CRUD ============

pub fn get_templates_by_preset(conn: &rusqlite::Connection, preset_id: &str) -> rusqlite::Result<Vec<NginxTemplate>> {
    get_all_by_preset(conn, "nginx_templates", preset_id, row_to_nginx_template)
}

pub fn add_nginx_template(conn: &rusqlite::Connection, t: &NginxTemplate) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO nginx_templates (id, presetId, name, content, createdAt)
         VALUES (?1,?2,?3,?4,?5)",
        params![t.id, t.preset_id, t.name, t.content, t.created_at],
    )?;
    Ok(())
}

pub fn update_nginx_template(conn: &rusqlite::Connection, t: &NginxTemplate) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE nginx_templates SET name=?2, content=?3 WHERE id=?1",
        params![t.id, t.name, t.content],
    )?;
    Ok(())
}

pub fn get_nginx_template_by_id(conn: &rusqlite::Connection, id: &str) -> rusqlite::Result<Option<NginxTemplate>> {
    get_by_id_internal(conn, "nginx_templates", id, row_to_nginx_template)
}

pub fn delete_nginx_template(conn: &rusqlite::Connection, id: &str) -> rusqlite::Result<()> {
    delete_by_id(conn, "nginx_templates", id)
}

// ============ NginxParam CRUD ============

pub fn get_params_by_preset(conn: &rusqlite::Connection, preset_id: &str) -> rusqlite::Result<Vec<NginxParam>> {
    get_all_by_preset(conn, "nginx_params", preset_id, row_to_nginx_param)
}

pub fn get_params_by_server(conn: &rusqlite::Connection, server_id: &str) -> rusqlite::Result<Vec<NginxParam>> {
    get_all_by_fk(conn, "nginx_params", "serverId", server_id, row_to_nginx_param)
}

pub fn get_params_by_location(conn: &rusqlite::Connection, location_id: &str) -> rusqlite::Result<Vec<NginxParam>> {
    get_all_by_fk(conn, "nginx_params", "locationId", location_id, row_to_nginx_param)
}

pub fn get_params_by_upstream(conn: &rusqlite::Connection, upstream_id: &str) -> rusqlite::Result<Vec<NginxParam>> {
    get_all_by_fk(conn, "nginx_params", "upstreamId", upstream_id, row_to_nginx_param)
}

pub fn add_nginx_param(conn: &rusqlite::Connection, p: &NginxParam) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO nginx_params (id, presetId, serverId, locationId, upstreamId, name, value, position, templateValue, sort, createdAt)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        params![p.id, p.preset_id, p.server_id, p.location_id, p.upstream_id,
                p.name, p.value, p.position, p.template_value, p.sort, p.created_at],
    )?;
    Ok(())
}

pub fn update_nginx_param(conn: &rusqlite::Connection, p: &NginxParam) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE nginx_params SET name=?2, value=?3, position=?4, templateValue=?5, sort=?6 WHERE id=?1",
        params![p.id, p.name, p.value, p.position, p.template_value, p.sort],
    )?;
    Ok(())
}

pub fn delete_nginx_param(conn: &rusqlite::Connection, id: &str) -> rusqlite::Result<()> {
    delete_by_id(conn, "nginx_params", id)
}

// ============ NginxDenyAllow CRUD ============

pub fn get_deny_allows_by_preset(conn: &rusqlite::Connection, preset_id: &str) -> rusqlite::Result<Vec<NginxDenyAllow>> {
    let sql = "SELECT * FROM nginx_deny_allows WHERE presetId = ?1 ORDER BY createdAt";
    query_all(conn, sql, rusqlite::params![preset_id], row_to_nginx_deny_allow)
}

pub fn get_deny_allow_by_id(conn: &rusqlite::Connection, id: &str) -> rusqlite::Result<Option<NginxDenyAllow>> {
    get_by_id_internal(conn, "nginx_deny_allows", id, row_to_nginx_deny_allow)
}

pub fn add_nginx_deny_allow(conn: &rusqlite::Connection, d: &NginxDenyAllow) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO nginx_deny_allows (id, presetId, name, ip, createdAt)
         VALUES (?1,?2,?3,?4,?5)",
        params![d.id, d.preset_id, d.name, d.ip, d.created_at],
    )?;
    Ok(())
}

pub fn update_nginx_deny_allow(conn: &rusqlite::Connection, d: &NginxDenyAllow) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE nginx_deny_allows SET name=?2, ip=?3 WHERE id=?1",
        params![d.id, d.name, d.ip],
    )?;
    Ok(())
}

pub fn delete_nginx_deny_allow(conn: &rusqlite::Connection, id: &str) -> rusqlite::Result<()> {
    delete_by_id(conn, "nginx_deny_allows", id)
}

// ============ NginxPassword CRUD ============

pub fn get_passwords_by_preset(conn: &rusqlite::Connection, preset_id: &str) -> rusqlite::Result<Vec<NginxPassword>> {
    let sql = "SELECT * FROM nginx_passwords WHERE presetId = ?1 ORDER BY createdAt";
    query_all(conn, sql, rusqlite::params![preset_id], row_to_nginx_password)
}

pub fn get_password_by_id(conn: &rusqlite::Connection, id: &str) -> rusqlite::Result<Option<NginxPassword>> {
    get_by_id_internal(conn, "nginx_passwords", id, row_to_nginx_password)
}

pub fn add_nginx_password(conn: &rusqlite::Connection, pw: &NginxPassword) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO nginx_passwords (id, presetId, name, pass, descr, path, createdAt)
         VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![pw.id, pw.preset_id, pw.name, pw.pass, pw.descr, pw.path, pw.created_at],
    )?;
    Ok(())
}

pub fn update_nginx_password(conn: &rusqlite::Connection, pw: &NginxPassword) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE nginx_passwords SET name=?2, pass=?3, descr=?4, path=?5 WHERE id=?1",
        params![pw.id, pw.name, pw.pass, pw.descr, pw.path],
    )?;
    Ok(())
}

pub fn delete_nginx_password(conn: &rusqlite::Connection, id: &str) -> rusqlite::Result<()> {
    delete_by_id(conn, "nginx_passwords", id)
}

// ============ NginxBasicSetting CRUD (key-value) ============

pub fn get_basic_settings_by_preset(conn: &rusqlite::Connection, preset_id: &str) -> rusqlite::Result<Vec<NginxBasicSetting>> {
    get_all_by_preset(conn, "nginx_basic_settings", preset_id, row_to_nginx_basic_setting)
}

pub fn add_nginx_basic_setting(conn: &rusqlite::Connection, s: &NginxBasicSetting) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO nginx_basic_settings (id, presetId, name, value, sort, createdAt)
         VALUES (?1,?2,?3,?4,?5,?6)",
        params![s.id, s.preset_id, s.name, s.value, s.sort, s.created_at],
    )?;
    Ok(())
}

pub fn update_nginx_basic_setting(conn: &rusqlite::Connection, s: &NginxBasicSetting) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE nginx_basic_settings SET name=?2, value=?3, sort=?4 WHERE id=?1",
        params![s.id, s.name, s.value, s.sort],
    )?;
    Ok(())
}

pub fn delete_nginx_basic_setting(conn: &rusqlite::Connection, id: &str) -> rusqlite::Result<()> {
    delete_by_id(conn, "nginx_basic_settings", id)
}

pub fn delete_basic_settings_by_preset(conn: &rusqlite::Connection, preset_id: &str) -> rusqlite::Result<()> {
    delete_by_fk(conn, "nginx_basic_settings", "presetId", preset_id)
}
