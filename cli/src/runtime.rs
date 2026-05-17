use std::path::PathBuf;
use supertool_core::{CoreService, Database, logic::data_dir};

/// Initialize the CLI runtime: data directory, database, and CoreService
pub struct CliRuntime {
    pub core: CoreService,
    #[allow(dead_code)]
    pub data_dir: PathBuf,
}

impl CliRuntime {
    pub fn init() -> Result<Self, anyhow::Error> {
        let data_dir = data_dir::resolve_data_dir();
        std::fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("supertool.db");
        let database = Database::new(&db_path)?;
        let core = CoreService::new(database, data_dir.clone());

        Ok(Self { core, data_dir })
    }
}
