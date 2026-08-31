use serde_json::{Value, json};

/// Cicd Data module — extracted from mod.rs
///

impl super::CoreService {
    pub async fn get_all_cicd_data(&self) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let conn = db.conn();
            // cicd_configs
            let mut stmt = conn
                .prepare("SELECT * FROM cicd_configs")
                .map_err(|e| e.to_string())?;
            let configs: Vec<Value> = stmt
                .query_map([], |row| {
                    Ok(json!({
                        "id": row.get::<_, String>("id")?,
                        "name": row.get::<_, String>("name")?,
                        "deployBranch": row.get::<_, String>("deployBranch")?,
                        "mavenSettings": row.get::<_, Option<String>>("mavenSettings")?,
                        "mavenProfile": row.get::<_, String>("mavenProfile")?,
                        "deployPath": row.get::<_, String>("deployPath")?,
                        "libSeparate": row.get::<_, i64>("libSeparate")? == 1,
                        "restartScript": row.get::<_, String>("restartScript")?,
                        "healthCheckUrl": row.get::<_, Option<String>>("healthCheckUrl")?,
                        "healthCheckTimeout": row.get::<_, i64>("healthCheckTimeout")?,
                        "createdAt": row.get::<_, String>("createdAt")?,
                        "updatedAt": row.get::<_, String>("updatedAt")?,
                        "buildTool": row.get::<_, Option<String>>("buildTool")?,
                        "buildCommand": row.get::<_, Option<String>>("buildCommand")?,
                        "buildPath": row.get::<_, Option<String>>("buildPath")?,
                        "repoUrl": row.get::<_, Option<String>>("repoUrl")?,
                        "localPath": row.get::<_, Option<String>>("localPath")?,
                        "npmScript": row.get::<_, Option<String>>("npmScript")?,
                        "npmCustomScript": row.get::<_, Option<String>>("npmCustomScript")?,
                        "mavenHome": row.get::<_, Option<String>>("mavenHome")?,
                        "npmHome": row.get::<_, Option<String>>("npmHome")?,
                        "javaHome": row.get::<_, Option<String>>("javaHome")?,
                        "nodeHome": row.get::<_, Option<String>>("nodeHome")?,
                        "servers": row.get::<_, Option<String>>("servers")?,
                        "groupName": row.get::<_, String>("groupName")?,
                        "lastDeployedAt": row.get::<_, Option<String>>("lastDeployedAt")?,
                        "parentBuildMode": row.get::<_, i64>("parentBuildMode")? == 1,
                        "parentBuildPath": row.get::<_, String>("parentBuildPath")?,
                        "requiresApproval": row.get::<_, Option<i64>>("requiresApproval")?.unwrap_or(0) == 1,
                        "pnpmHome": row.get::<_, Option<String>>("pnpmHome")?,
                        "yarnHome": row.get::<_, Option<String>>("yarnHome")?,
                        "buildMode": row.get::<_, String>("buildMode")?,
                        "gitRepoId": row.get::<_, Option<String>>("gitRepoId")?,
                    }))
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            // deploy_modules
            let mut stmt2 = conn
                .prepare("SELECT * FROM deploy_modules")
                .map_err(|e| e.to_string())?;
            let modules: Vec<Value> = stmt2
                .query_map([], |row| {
                    Ok(json!({
                        "id": row.get::<_, String>("id")?,
                        "configId": row.get::<_, String>("configId")?,
                        "moduleName": row.get::<_, String>("moduleName")?,
                        "modulePath": row.get::<_, String>("modulePath")?,
                        "buildPath": row.get::<_, Option<String>>("buildPath")?,
                        "buildCommand": row.get::<_, Option<String>>("buildCommand")?,
                        "buildTool": row.get::<_, Option<String>>("buildTool")?,
                        "outputPath": row.get::<_, Option<String>>("outputPath")?,
                        "artifactName": row.get::<_, Option<String>>("artifactName")?.unwrap_or_default(),
                        "artifactType": row.get::<_, Option<String>>("artifactType")?,
                        "libFilterRules": row.get::<_, Option<String>>("libFilterRules")?,
                        "deployOrder": row.get::<_, i64>("deployOrder")?,
                        "deployPath": row.get::<_, Option<String>>("deployPath")?,
                        "enabled": row.get::<_, i64>("enabled")? == 1,
                        "createdAt": row.get::<_, String>("createdAt")?,
                        "updatedAt": row.get::<_, String>("updatedAt")?,
                    }))
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            // deploy_logs
            let mut stmt3 = conn
                .prepare("SELECT * FROM deploy_logs ORDER BY id DESC LIMIT 500")
                .map_err(|e| e.to_string())?;
            let logs: Vec<Value> = stmt3
                .query_map([], |row| {
                    Ok(json!({
                        "id": row.get::<_, String>("id")?,
                        "configId": row.get::<_, String>("configId")?,
                        "status": row.get::<_, String>("status")?,
                        "startTime": row.get::<_, String>("startTime")?,
                        "endTime": row.get::<_, Option<String>>("endTime")?,
                        "errorMessage": row.get::<_, Option<String>>("errorMessage")?,
                        "progress": row.get::<_, i64>("progress")?,
                        "triggeredBy": row.get::<_, Option<String>>("triggeredBy")?,
                        "createdAt": row.get::<_, String>("createdAt")?,
                        "logFilePath": row.get::<_, Option<String>>("logFilePath")?,
                        "artifactPaths": row.get::<_, Option<String>>("artifactPaths")?,
                    }))
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            // deploy_history
            let mut stmt4 = conn
                .prepare("SELECT * FROM deploy_history ORDER BY id DESC LIMIT 200")
                .map_err(|e| e.to_string())?;
            let history: Vec<Value> = stmt4
                .query_map([], |row| {
                    Ok(json!({
                        "id": row.get::<_, String>("id")?,
                        "configId": row.get::<_, String>("configId")?,
                        "status": row.get::<_, String>("status")?,
                        "deployedAt": row.get::<_, String>("deployedAt")?,
                        "rolledBack": row.get::<_, i64>("rolledBack")? == 1,
                        "rolledBackAt": row.get::<_, Option<String>>("rolledBackAt")?,
                    }))
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            // deploy_step_logs
            let mut stmt5 = conn
                .prepare("SELECT * FROM deploy_step_logs ORDER BY id DESC LIMIT 1000")
                .map_err(|e| e.to_string())?;
            let steps: Vec<Value> = stmt5
                .query_map([], |row| {
                    Ok(json!({
                        "id": row.get::<_, String>("id")?,
                        "deployLogId": row.get::<_, String>("deployLogId")?,
                        "stage": row.get::<_, String>("stage")?,
                        "status": row.get::<_, String>("status")?,
                        "message": row.get::<_, Option<String>>("message")?,
                        "timestamp": row.get::<_, String>("timestamp")?,
                    }))
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            Ok(json!({
                "cicdConfigs": configs,
                "deployModules": modules,
                "deployLogs": logs,
                "deployHistory": history,
                "deployStepLogs": steps,
            }))
        });
        result
    }

    /// 获取所有部署历史（含配置名），对应 Tauri 版 get_all_deploy_history。
    pub async fn get_all_deploy_history(&self, limit: i64) -> Result<Value, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let conn = db.conn();
        let mut stmt = conn.prepare(
            "SELECT h.id, h.configId, c.name as configName, h.status, h.deployedAt, h.rolledBack, h.rolledBackAt \
             FROM deploy_history h \
             LEFT JOIN cicd_configs c ON h.configId = c.id \
             ORDER BY h.deployedAt DESC LIMIT ?1"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map(rusqlite::params![limit], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>("id")?,
                "configId": row.get::<_, String>("configId")?,
                "configName": row.get::<_, Option<String>>("configName")?,
                "status": row.get::<_, String>("status")?,
                "deployedAt": row.get::<_, String>("deployedAt")?,
                "rolledBack": row.get::<_, i64>("rolledBack")? != 0,
                "rolledBackAt": row.get::<_, Option<String>>("rolledBackAt")?,
            }))
        }).map_err(|e| e.to_string())?;
        let items: Vec<Value> = rows.filter_map(|r| r.ok()).collect();
        serde_json::to_value(&items).map_err(|e| e.to_string())
    }

    // ============ Log Presets ============
}
