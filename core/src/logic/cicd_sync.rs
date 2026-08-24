/// Cicd Sync module — extracted from mod.rs
///

impl super::CoreService {
    pub fn get_cicd_configs(&self) -> Result<Vec<crate::db::cicd::CicdConfig>, String> {
        self.db_read(|conn| crate::db::cicd::get_all_cicd_configs(conn).expect("get cicd configs"))
    }

    pub fn get_cicd_groups(&self) -> Result<Vec<String>, String> {
        self.db_read(|conn| crate::db::cicd::get_cicd_groups(conn).expect("get cicd groups"))
    }

    pub fn get_deploy_modules(
        &self,
        config_id: &str,
    ) -> Result<Vec<crate::db::cicd::DeployModule>, String> {
        self.db_read(|conn| {
            crate::db::cicd::get_deploy_modules(conn, config_id).expect("get deploy modules")
        })
    }

    pub fn get_deploy_step_logs(
        &self,
        deploy_log_id: &str,
    ) -> Result<Vec<crate::db::cicd::DeployStepLog>, String> {
        self.db_read(|conn| {
            crate::db::cicd::get_deploy_step_logs(conn, deploy_log_id).expect("get step logs")
        })
    }

    /// 部署历史（deploy_history 表已废弃，2026-08 起改读 deploy_logs）。
    /// 返回 DeployLog：id/status 与原 DeployHistory 兼容，
    /// 时间字段用 start_time（部署开始时间）
    pub fn get_deploy_history_by_config(
        &self,
        config_id: &str,
        limit: i64,
    ) -> Result<Vec<crate::db::cicd::DeployLog>, String> {
        self.get_deploy_logs_by_config(config_id, limit)
    }

    pub fn get_deploy_logs_by_config(
        &self,
        config_id: &str,
        limit: i64,
    ) -> Result<Vec<crate::db::cicd::DeployLog>, String> {
        self.db_read(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT * FROM deploy_logs WHERE configId = ? ORDER BY createdAt DESC LIMIT ?",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(
                    rusqlite::params![config_id, limit],
                    crate::db::cicd::row_to_deploy_log,
                )
                .map_err(|e| e.to_string())?;
            Ok(rows.filter_map(|r| r.ok()).collect())
        })?
    }

    // ============ Misc ============
}
