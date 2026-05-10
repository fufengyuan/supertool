/// Cicd Sync module — extracted from mod.rs
use super::CoreService;
use crate::db::Database;
use serde_json::{json, Value};
use std::path::PathBuf;

/// Cicd Sync module — extracted from mod.rs
///

impl super::CoreService {
    pub fn get_cicd_configs(&self) -> Result<Vec<crate::db::cicd::CicdConfig>, String> {
        self.db_read(|conn| crate::db::cicd::get_all_cicd_configs(conn).expect("get cicd configs"))
    }

    pub fn get_cicd_groups(&self) -> Result<Vec<String>, String> {
        self.db_read(|conn| crate::db::cicd::get_cicd_groups(conn).expect("get cicd groups"))
    }

    pub fn get_deploy_logs(&self, project_id: &str, limit: i64) -> Result<Vec<crate::db::cicd::DeployLog>, String> {
        self.db_read(|conn| crate::db::cicd::get_deploy_logs(conn, project_id, limit).expect("get deploy logs"))
    }

    pub fn get_deploy_modules(&self, config_id: &str) -> Result<Vec<crate::db::cicd::DeployModule>, String> {
        self.db_read(|conn| crate::db::cicd::get_deploy_modules(conn, config_id).expect("get deploy modules"))
    }

    pub fn get_deploy_history(&self, project_id: &str, limit: i64) -> Result<Vec<crate::db::cicd::DeployHistory>, String> {
        self.db_read(|conn| crate::db::cicd::get_deploy_history(conn, project_id, limit).expect("get deploy history"))
    }

    pub fn get_deploy_step_logs(&self, deploy_log_id: &str) -> Result<Vec<crate::db::cicd::DeployStepLog>, String> {
        self.db_read(|conn| crate::db::cicd::get_deploy_step_logs(conn, deploy_log_id).expect("get step logs"))
    }

    // ============ Misc ============


}
