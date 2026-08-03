use std::path::PathBuf;
use supertool_core::{CoreService, Database, logic::data_dir};

/// Initialize the CLI runtime: data directory, database, and CoreService
pub struct CliRuntime {
    pub core: CoreService,
    #[allow(dead_code)]
    pub data_dir: PathBuf,
    /// 全局 `--json` 模式（命令级 `-j` 由各命令 action 自行处理，两处任一开启即 JSON 输出）
    pub json_mode: bool,
}

impl CliRuntime {
    pub fn init() -> Result<Self, anyhow::Error> {
        let data_dir = data_dir::resolve_data_dir();
        std::fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("supertool.db");
        let database = Database::new(&db_path)?;
        let core = CoreService::new(database, data_dir.clone());

        Ok(Self {
            core,
            data_dir,
            json_mode: false,
        })
    }

    /// 开启 JSON 模式（命令级 -j 或全局 --json 任一即开启），同步全局输出标志，
    /// 让 print_success/print_error 也输出 envelope（避免文本/JSON 混流）
    pub fn set_json(&mut self, on: bool) {
        self.json_mode = self.json_mode || on;
        crate::output::set_json_mode(self.json_mode);
    }
}
