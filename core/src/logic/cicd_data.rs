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

    pub async fn import_cicd_data(
        &self,
        data: &Value,
        mode: &str,
    ) -> Result<(usize, usize), String> {
        let mut imported = 0;
        let mut skipped = 0;

        let _ = self.with_db(|db| {
            let conn = db.conn();
            if mode == "replace" {
                conn.execute_batch("DELETE FROM deploy_step_logs; DELETE FROM deploy_logs; DELETE FROM deploy_modules; DELETE FROM cicd_configs; DELETE FROM deploy_history;")
                    .map_err(|e| e.to_string())?;
            }

            // cicd_configs — 30 columns
            if let Some(configs) = data.get("cicdConfigs").and_then(|v| v.as_array()) {
                for c in configs {
                    let id = c.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    if mode == "merge" {
                        let exists: i64 = conn.prepare("SELECT COUNT(*) FROM cicd_configs WHERE id = ?")
                            .ok().and_then(|mut s| s.query_row([id], |r| r.get(0)).ok()).unwrap_or(0);
                        if exists > 0 { skipped += 1; continue; }
                    }
                    let servers_val: Option<String> = c.get("servers").and_then(|v| v.as_str()).map(|s| s.to_string())
                        .or_else(|| c.get("servers").map(|v| serde_json::to_string(v).unwrap_or_default()));
                    let _ = conn.execute(
                        "INSERT OR REPLACE INTO cicd_configs (id, name, deployBranch, mavenSettings, mavenProfile, deployPath, libSeparate, restartScript, healthCheckUrl, healthCheckTimeout, createdAt, updatedAt, groupName, parentBuildMode, parentBuildPath, requiresApproval, buildTool, buildCommand, buildPath, repoUrl, localPath, npmScript, npmCustomScript, mavenHome, npmHome, javaHome, nodeHome, servers, lastDeployedAt, gitRepoId, pnpmHome, yarnHome, buildMode)
                         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26,?27,?28,?29,?30,?31,?32,?33)",
                        rusqlite::params![
                            id,
                            c.get("name").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("deployBranch").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("mavenSettings").and_then(|v|v.as_str()),
                            c.get("mavenProfile").and_then(|v|v.as_str()).unwrap_or("prod"),
                            c.get("deployPath").and_then(|v|v.as_str()).unwrap_or("/"),
                            if c.get("libSeparate").and_then(|v|v.as_bool()).unwrap_or(false) { 1 } else { 0 },
                            c.get("restartScript").and_then(|v|v.as_str()).unwrap_or("./restart.sh"),
                            c.get("healthCheckUrl").and_then(|v|v.as_str()),
                            c.get("healthCheckTimeout").and_then(|v|v.as_i64()).unwrap_or(30),
                            c.get("createdAt").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("updatedAt").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("groupName").and_then(|v|v.as_str()).unwrap_or("未分组"),
                            if c.get("parentBuildMode").and_then(|v|v.as_bool()).unwrap_or(false) { 1 } else { 0 },
                            c.get("parentBuildPath").and_then(|v|v.as_str()).unwrap_or(""),
                            if c.get("requiresApproval").and_then(|v|v.as_bool()).unwrap_or(false) { 1 } else { 0 },
                            c.get("buildTool").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("buildCommand").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("buildPath").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("repoUrl").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("localPath").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("npmScript").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("npmCustomScript").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("mavenHome").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("npmHome").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("javaHome").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("nodeHome").and_then(|v|v.as_str()).unwrap_or(""),
                            servers_val.unwrap_or_default(),
                            c.get("lastDeployedAt").and_then(|v|v.as_str()),
                            c.get("gitRepoId").and_then(|v|v.as_str()),
                            c.get("pnpmHome").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("yarnHome").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("buildMode").and_then(|v|v.as_str()).unwrap_or("local"),
                        ]
                    );
                    imported += 1;
                }
            }

            // deploy_modules — 16 columns
            if let Some(modules) = data.get("deployModules").and_then(|v| v.as_array()) {
                for m in modules {
                    let _ = conn.execute(
                        "INSERT OR REPLACE INTO deploy_modules (id, configId, moduleName, modulePath, artifactName, deployOrder, deployPath, enabled, createdAt, updatedAt, libFilterRules, buildCommand, buildPath, outputPath, buildTool, artifactType)
                         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
                        rusqlite::params![
                            m.get("id").and_then(|v|v.as_str()).unwrap_or(""),
                            m.get("configId").and_then(|v|v.as_str()).unwrap_or(""),
                            m.get("moduleName").and_then(|v|v.as_str()).unwrap_or(""),
                            m.get("modulePath").and_then(|v|v.as_str()).unwrap_or(""),
                            m.get("artifactName").and_then(|v|v.as_str()).unwrap_or(""),
                            m.get("deployOrder").and_then(|v|v.as_i64()).unwrap_or(0),
                            m.get("deployPath").and_then(|v|v.as_str()).unwrap_or(""),
                            if m.get("enabled").and_then(|v|v.as_bool()).unwrap_or(true) { 1 } else { 0 },
                            m.get("createdAt").and_then(|v|v.as_str()).unwrap_or(""),
                            m.get("updatedAt").and_then(|v|v.as_str()).unwrap_or(""),
                            m.get("libFilterRules").and_then(|v|v.as_str()).unwrap_or(""),
                            m.get("buildCommand").and_then(|v|v.as_str()).unwrap_or(""),
                            m.get("buildPath").and_then(|v|v.as_str()).unwrap_or(""),
                            m.get("outputPath").and_then(|v|v.as_str()).unwrap_or(""),
                            m.get("buildTool").and_then(|v|v.as_str()).unwrap_or(""),
                            m.get("artifactType").and_then(|v|v.as_str()).unwrap_or(""),
                        ]
                    );
                    imported += 1;
                }
            }

            // deploy_logs
            if let Some(logs) = data.get("deployLogs").and_then(|v| v.as_array()) {
                for l in logs {
                    let _ = conn.execute(
                        "INSERT OR REPLACE INTO deploy_logs (id, configId, status, startTime, endTime, errorMessage, progress, triggeredBy, createdAt, logFilePath, artifactPaths)
                         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                        rusqlite::params![
                            l.get("id").and_then(|v|v.as_str()).unwrap_or(""),
                            l.get("configId").and_then(|v|v.as_str()).unwrap_or(""),
                            l.get("status").and_then(|v|v.as_str()).unwrap_or(""),
                            l.get("startTime").and_then(|v|v.as_str()).unwrap_or(""),
                            l.get("endTime").and_then(|v|v.as_str()),
                            l.get("errorMessage").and_then(|v|v.as_str()),
                            l.get("progress").and_then(|v|v.as_i64()).unwrap_or(0),
                            l.get("triggeredBy").and_then(|v|v.as_str()),
                            l.get("createdAt").and_then(|v|v.as_str()).unwrap_or(""),
                            l.get("logFilePath").and_then(|v|v.as_str()),
                            l.get("artifactPaths").and_then(|v|v.as_str()),
                        ]
                    );
                    imported += 1;
                }
            }

            // deploy_history
            if let Some(history) = data.get("deployHistory").and_then(|v| v.as_array()) {
                for h in history {
                    let _ = conn.execute(
                        "INSERT OR REPLACE INTO deploy_history (id, configId, status, deployedAt, rolledBack, rolledBackAt)
                         VALUES (?1,?2,?3,?4,?5,?6)",
                        rusqlite::params![
                            h.get("id").and_then(|v|v.as_str()).unwrap_or(""),
                            h.get("configId").and_then(|v|v.as_str()).unwrap_or(""),
                            h.get("status").and_then(|v|v.as_str()).unwrap_or(""),
                            h.get("deployedAt").and_then(|v|v.as_str()).unwrap_or(""),
                            if h.get("rolledBack").and_then(|v|v.as_bool()).unwrap_or(false) { 1 } else { 0 },
                            h.get("rolledBackAt").and_then(|v|v.as_str()),
                        ]
                    );
                    imported += 1;
                }
            }

            // deploy_step_logs
            if let Some(steps) = data.get("deployStepLogs").and_then(|v| v.as_array()) {
                for s in steps {
                    let _ = conn.execute(
                        "INSERT OR REPLACE INTO deploy_step_logs (id, deployLogId, stage, status, message, timestamp)
                         VALUES (?1,?2,?3,?4,?5,?6)",
                        rusqlite::params![
                            s.get("id").and_then(|v|v.as_str()).unwrap_or(""),
                            s.get("deployLogId").and_then(|v|v.as_str()).unwrap_or(""),
                            s.get("stage").and_then(|v|v.as_str()).unwrap_or(""),
                            s.get("status").and_then(|v|v.as_str()).unwrap_or(""),
                            s.get("message").and_then(|v|v.as_str()),
                            s.get("timestamp").and_then(|v|v.as_str()).unwrap_or(""),
                        ]
                    );
                    imported += 1;
                }
            }

            Ok::<(), String>(())
        });

        Ok((imported, skipped))
    }

    // ============ Log Presets ============
}
