use crate::db::{ApiResponse, Database, Server, ServerGroup};
use rusqlite::params;

pub fn row_to_server(row: &rusqlite::Row<'_>) -> rusqlite::Result<Server> {
    let tags_str: String = row.get("tags")?;
    let tags: Vec<String> = if tags_str.is_empty() {
        Vec::new()
    } else {
        tags_str.split(',').map(|s| s.to_string()).collect()
    };
    let requires_approval: i64 = row.get("requiresApproval")?;

    Ok(Server {
        id: row.get("id")?,
        name: row.get("name")?,
        host: row.get("host")?,
        port: row.get("port")?,
        username: row.get("username")?,
        ssh_key_path: row.get("sshKeyPath").ok(),
        password: row.get("password").ok(),
        description: row.get("description")?,
        tags,
        group_id: row.get("groupId").ok(),
        requires_approval: requires_approval == 1,
        created_at: row.get("createdAt")?,
        updated_at: row.get("updatedAt")?,
    })
}

pub fn row_to_server_group(row: &rusqlite::Row<'_>) -> rusqlite::Result<ServerGroup> {
    Ok(ServerGroup {
        id: row.get("id")?,
        name: row.get("name")?,
        description: row.get("description")?,
        parent_id: row.get("parentId").ok(),
        color: row.get("color")?,
        created_at: row.get("createdAt")?,
        updated_at: row.get("updatedAt")?,
    })
}

fn collect_servers(stmt: &mut rusqlite::Statement<'_>) -> ApiResponse<Vec<Server>> {
    match stmt.query_map([], row_to_server) {
        Ok(rows) => {
            let servers: Result<Vec<Server>, rusqlite::Error> = rows.collect();
            match servers {
                Ok(list) => ApiResponse::ok(list),
                Err(e) => ApiResponse::err(format!("Query failed: {}", e)),
            }
        }
        Err(e) => ApiResponse::err(format!("Query failed: {}", e)),
    }
}

fn collect_groups(stmt: &mut rusqlite::Statement<'_>) -> ApiResponse<Vec<ServerGroup>> {
    match stmt.query_map([], row_to_server_group) {
        Ok(rows) => {
            let groups: Result<Vec<ServerGroup>, rusqlite::Error> = rows.collect();
            match groups {
                Ok(list) => ApiResponse::ok(list),
                Err(e) => ApiResponse::err(format!("Query failed: {}", e)),
            }
        }
        Err(e) => ApiResponse::err(format!("Query failed: {}", e)),
    }
}

// ============ Server CRUD ============

pub fn get_all_servers(db: &mut Database) -> ApiResponse<Vec<Server>> {
    match db
        .conn()
        .prepare("SELECT * FROM servers ORDER BY createdAt DESC")
    {
        Ok(mut stmt) => collect_servers(&mut stmt),
        Err(e) => ApiResponse::err(format!("Prepare failed: {}", e)),
    }
}

pub fn get_server_by_id(db: &mut Database, server_id: String) -> ApiResponse<Option<Server>> {
    let mut stmt = match db.conn().prepare("SELECT * FROM servers WHERE id = ?1") {
        Ok(s) => s,
        Err(e) => return ApiResponse::err(format!("Prepare failed: {}", e)),
    };

    match stmt.query_row(params![server_id], row_to_server) {
        Ok(server) => ApiResponse::ok(Some(server)),
        Err(rusqlite::Error::QueryReturnedNoRows) => ApiResponse::ok(None),
        Err(e) => ApiResponse::err(format!("Query failed: {}", e)),
    }
}

pub fn add_server(db: &mut Database, server: Server) -> ApiResponse<Server> {
    // 密码已在 core/mod.rs 中加密，这里直接存储
    let encrypted_pw = server.password.clone();

    let result = db.conn_mut().execute(
        "INSERT INTO servers (id, name, host, port, username, sshKeyPath, password, description, tags, groupId, requiresApproval, createdAt, updatedAt)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            server.id,
            server.name,
            server.host,
            server.port,
            server.username,
            server.ssh_key_path,
            encrypted_pw,
            server.description,
            server.tags.join(","),
            server.group_id,
            if server.requires_approval { 1 } else { 0 },
            server.created_at,
            server.updated_at,
        ],
    );
    match result {
        Ok(_) => ApiResponse::ok(server),
        Err(e) => ApiResponse::err(format!("Insert failed: {}", e)),
    }
}

pub fn update_server(db: &mut Database, server: Server) -> ApiResponse<Server> {
    // 密码已在 core/mod.rs 中加密，有传就用新的，没传就保留旧的
    let encrypted_pw = if server.password.as_ref().map_or(false, |p| !p.is_empty()) {
        server.password.clone()
    } else {
        let mut stmt = db
            .conn()
            .prepare("SELECT password FROM servers WHERE id = ?1")
            .ok();
        if let Some(ref mut s) = stmt {
            if let Ok(existing) = s.query_row(params![server.id], |r| r.get::<_, Option<String>>(0))
            {
                existing
            } else {
                None
            }
        } else {
            None
        }
    };

    let result = db.conn_mut().execute(
        "UPDATE servers SET name=?2, host=?3, port=?4, username=?5, sshKeyPath=?6, password=?7, description=?8, tags=?9, groupId=?10, requiresApproval=?11, updatedAt=?12 WHERE id=?1",
        params![
            server.id,
            server.name,
            server.host,
            server.port,
            server.username,
            server.ssh_key_path,
            encrypted_pw,
            server.description,
            server.tags.join(","),
            server.group_id,
            if server.requires_approval { 1 } else { 0 },
            server.updated_at,
        ],
    );
    match result {
        Ok(_) => ApiResponse::ok(server),
        Err(e) => ApiResponse::err(format!("Update failed: {}", e)),
    }
}

pub fn delete_server(db: &mut Database, server_id: String) -> ApiResponse<String> {
    let result = db
        .conn_mut()
        .execute("DELETE FROM servers WHERE id = ?1", params![server_id]);
    match result {
        Ok(_) => ApiResponse::ok(server_id),
        Err(e) => ApiResponse::err(format!("Delete failed: {}", e)),
    }
}

// ============ Server Group CRUD ============

pub fn get_all_server_groups(db: &mut Database) -> ApiResponse<Vec<ServerGroup>> {
    match db
        .conn()
        .prepare("SELECT * FROM server_groups ORDER BY createdAt ASC")
    {
        Ok(mut stmt) => collect_groups(&mut stmt),
        Err(e) => ApiResponse::err(format!("Prepare failed: {}", e)),
    }
}

pub fn add_server_group(db: &mut Database, group: ServerGroup) -> ApiResponse<ServerGroup> {
    let result = db.conn_mut().execute(
        "INSERT INTO server_groups (id, name, description, parentId, color, createdAt, updatedAt)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            group.id,
            group.name,
            group.description,
            group.parent_id,
            group.color,
            group.created_at,
            group.updated_at,
        ],
    );
    match result {
        Ok(_) => ApiResponse::ok(group),
        Err(e) => ApiResponse::err(format!("Insert failed: {}", e)),
    }
}

pub fn update_server_group(
    db: &mut Database,
    group_id: String,
    name: String,
    description: String,
    parent_id: Option<String>,
    color: String,
) -> ApiResponse<ServerGroup> {
    let now = chrono::Utc::now().to_rfc3339();
    let result = db.conn_mut().execute(
        "UPDATE server_groups SET name=?2, description=?3, parentId=?4, color=?5, updatedAt=?6 WHERE id=?1",
        params![group_id, name, description, parent_id, color, now],
    );
    match result {
        Ok(_) => {
            let mut stmt = db
                .conn()
                .prepare("SELECT * FROM server_groups WHERE id = ?1")
                .ok();
            if let Some(ref mut s) = stmt {
                if let Ok(group) = s.query_row(params![group_id], row_to_server_group) {
                    return ApiResponse::ok(group);
                }
            }
            ApiResponse::err("Group not found after update".to_string())
        }
        Err(e) => ApiResponse::err(format!("Update failed: {}", e)),
    }
}

pub fn delete_server_group(db: &mut Database, group_id: String) -> ApiResponse<String> {
    // Recursively delete child groups
    let child_ids: Vec<String> = db
        .conn()
        .prepare("SELECT id FROM server_groups WHERE parentId = ?1")
        .and_then(|mut stmt| {
            stmt.query_map(params![group_id], |row| row.get::<_, String>(0))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default();

    for child_id in child_ids {
        let _ = delete_server_group(db, child_id);
    }

    let _ = db.conn_mut().execute(
        "UPDATE servers SET groupId = NULL WHERE groupId = ?1",
        params![group_id],
    );

    match db
        .conn_mut()
        .execute("DELETE FROM server_groups WHERE id = ?1", params![group_id])
    {
        Ok(_) => ApiResponse::ok(group_id),
        Err(e) => ApiResponse::err(format!("Delete failed: {}", e)),
    }
}

// ============ Connection Test ============

pub fn test_connection(
    host: String,
    port: i64,
    username: String,
    _password: Option<String>,
    ssh_key_path: Option<String>,
) -> ApiResponse<bool> {
    use std::process::Command;

    let mut cmd = Command::new("ssh");
    cmd.args([
        "-o",
        "BatchMode=yes",
        "-o",
        "ConnectTimeout=5",
        "-o",
        "StrictHostKeyChecking=no",
        "-p",
        &port.to_string(),
    ]);

    if let Some(ref key_path) = ssh_key_path {
        cmd.arg("-i").arg(key_path);
    }

    cmd.arg(format!("{}@{}", username, host));
    cmd.arg("echo ok");

    match cmd.output() {
        Ok(output) => ApiResponse::ok(output.status.success()),
        Err(e) => ApiResponse::err(format!("SSH test failed: {}", e)),
    }
}
