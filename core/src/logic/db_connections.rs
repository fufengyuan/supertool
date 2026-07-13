use super::*;
use crate::db::db_connections::DbConnectionConfig;

impl CoreService {
    pub async fn get_all_db_connections(&self) -> Result<Value, String> {
        self.with_db(|db| {
            let configs = crate::db::db_connections::get_all_db_connections(db)?;
            serde_json::to_value(&configs).map_err(|e| e.to_string())
        })
    }

    pub async fn add_db_connection(&self, config: DbConnectionConfig) -> Result<(), String> {
        self.with_db(move |db| crate::db::db_connections::add_db_connection(db, &config))
    }

    pub async fn update_db_connection(&self, config: DbConnectionConfig) -> Result<(), String> {
        self.with_db(move |db| crate::db::db_connections::update_db_connection(db, &config))
    }

    pub async fn delete_db_connection(&self, id: String) -> Result<(), String> {
        self.with_db(move |db| crate::db::db_connections::delete_db_connection(db, &id))
    }
}
