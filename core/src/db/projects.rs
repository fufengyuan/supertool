use crate::db::{ApiResponse, Database, Project, ProjectStats};
use rusqlite::params;

pub fn row_to_project(row: &rusqlite::Row<'_>) -> rusqlite::Result<Project> {
    let archived: i64 = row.get("archived")?;
    Ok(Project {
        id: row.get("id")?,
        name: row.get("name")?,
        description: row.get("description")?,
        color: row.get("color")?,
        repo_path: row.get("repoPath").ok(),
        branch: row.get("branch").ok(),
        repo_path2: row.get("repoPath2").ok(),
        branch2: row.get("branch2").ok(),
        git_url1: row.get("gitUrl1").ok(),
        git_url2: row.get("gitUrl2").ok(),
        git_repo_id: row.get("gitRepoId").ok(),
        git_repo_id2: row.get("gitRepoId2").ok(),
        category: row.get("category").ok(),
        created_at: row.get("createdAt")?,
        updated_at: row.get("updatedAt")?,
        archived: archived == 1,
    })
}

pub fn get_projects(db: &mut Database, only_active: bool) -> ApiResponse<Vec<Project>> {
    let mut query = "SELECT * FROM projects".to_string();
    if only_active {
        query.push_str(" WHERE archived = 0");
    }
    query.push_str(" ORDER BY createdAt DESC");

    match db.conn().prepare(&query) {
        Ok(mut stmt) => match stmt.query_map([], row_to_project) {
            Ok(rows) => {
                let projects: Result<Vec<Project>, rusqlite::Error> = rows.collect();
                match projects {
                    Ok(list) => ApiResponse::ok(list),
                    Err(e) => ApiResponse::err(format!("Query failed: {}", e)),
                }
            }
            Err(e) => ApiResponse::err(format!("Query failed: {}", e)),
        },
        Err(e) => ApiResponse::err(format!("Prepare failed: {}", e)),
    }
}

pub fn add_project(db: &mut Database, project: Project) -> ApiResponse<Project> {
    let result = db.conn_mut().execute(
        "INSERT INTO projects (id, name, description, color, repoPath, branch, repoPath2, branch2, gitUrl1, gitUrl2, gitRepoId, gitRepoId2, category, createdAt, updatedAt, archived)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
        params![
            project.id,
            project.name,
            project.description,
            project.color,
            project.repo_path,
            project.branch,
            project.repo_path2,
            project.branch2,
            project.git_url1,
            project.git_url2,
            project.git_repo_id,
            project.git_repo_id2,
            project.category,
            project.created_at,
            project.updated_at,
            if project.archived { 1 } else { 0 }
        ],
    );
    match result {
        Ok(_) => ApiResponse::ok(project),
        Err(e) => ApiResponse::err(format!("Insert failed: {}", e)),
    }
}

pub fn update_project(db: &mut Database, project: Project) -> ApiResponse<Project> {
    let result = db.conn_mut().execute(
        "UPDATE projects SET name=?2, description=?3, color=?4, repoPath=?5, branch=?6, repoPath2=?7, branch2=?8, gitUrl1=?9, gitUrl2=?10, gitRepoId=?11, gitRepoId2=?12, category=?13, updatedAt=?14, archived=?15 WHERE id=?1",
        params![
            project.id,
            project.name,
            project.description,
            project.color,
            project.repo_path,
            project.branch,
            project.repo_path2,
            project.branch2,
            project.git_url1,
            project.git_url2,
            project.git_repo_id,
            project.git_repo_id2,
            project.category,
            project.updated_at,
            if project.archived { 1 } else { 0 }
        ],
    );
    match result {
        Ok(0) => ApiResponse::err(format!("Project not found: {}", project.id)),
        Ok(_) => ApiResponse::ok(project),
        Err(e) => ApiResponse::err(format!("Update failed: {}", e)),
    }
}

pub fn delete_project(db: &mut Database, id: String) -> ApiResponse<String> {
    // 级联删除关联的 todo
    if let Err(e) = db.conn_mut().execute("DELETE FROM todos WHERE projectId = ?1", params![id]) {
        return ApiResponse::err(format!("Delete todos failed: {}", e));
    }
    let result = db
        .conn_mut()
        .execute("DELETE FROM projects WHERE id = ?1", params![id]);
    match result {
        Ok(0) => ApiResponse::err(format!("Project not found: {}", id)),
        Ok(_) => ApiResponse::ok(id),
        Err(e) => ApiResponse::err(format!("Delete failed: {}", e)),
    }
}

pub fn get_project_stats(db: &mut Database, project_id: String) -> ApiResponse<ProjectStats> {
    let mut stmt = match db.conn().prepare(
        "SELECT COUNT(*) as total, SUM(CASE WHEN completed = 1 THEN 1 ELSE 0 END) as completed FROM todos WHERE projectId = ?1"
    ) {
        Ok(s) => s,
        Err(e) => return ApiResponse::err(format!("Prepare failed: {}", e)),
    };

    match stmt.query_row(params![project_id], |row| {
        let total: i64 = row.get(0)?;
        let completed: Option<i64> = row.get(1)?;
        Ok((total, completed.unwrap_or(0)))
    }) {
        Ok((total, completed)) => {
            let progress = if total > 0 {
                (completed as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            ApiResponse::ok(ProjectStats {
                total,
                completed,
                progress,
            })
        }
        Err(e) => ApiResponse::err(format!("Query failed: {}", e)),
    }
}
