use serde_json::{json, Value};

impl super::CoreService {
    pub async fn get_all_lan_users(&self) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let users = crate::db::lan::get_all_users(db.conn()).map_err(|e| e.to_string())?;
            Ok::<_, String>(json!(users))
        });
        result
    }
    pub async fn get_all_lan_messages(&self) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let msgs = crate::db::lan::get_all_messages(db.conn()).map_err(|e| e.to_string())?;
            let chat_msgs = crate::db::lan::get_all_chat_messages(db.conn()).map_err(|e| e.to_string())?;
            let transfers = crate::db::lan::get_all_file_transfers(db.conn()).map_err(|e| e.to_string())?;
            Ok::<_, String>(json!({
                "messages": msgs,
                "chatMessages": chat_msgs,
                "fileTransfers": transfers,
            }))
        });
        result
    }
    #[allow(dead_code)]
    pub async fn insert_lan_user(&self, user: crate::db::lan::LanUser) -> Result<(), String> {
        self.with_db(|db| {
            crate::db::lan::insert_user(db.conn(), &user).map_err(|e| e.to_string())
        })
    }
    #[allow(dead_code)]
    pub async fn insert_lan_message(&self, msg: crate::db::lan::LanMessage) -> Result<(), String> {
        self.with_db(|db| {
            crate::db::lan::insert_message(db.conn(), &msg).map_err(|e| e.to_string())
        })
    }
    #[allow(dead_code)]
    pub async fn insert_chat_message(&self, msg: crate::db::lan::ChatMessage) -> Result<(), String> {
        self.with_db(|db| {
            crate::db::lan::insert_chat_message(db.conn(), &msg).map_err(|e| e.to_string())
        })
    }
    #[allow(dead_code)]
    pub async fn insert_file_transfer(&self, ft: crate::db::lan::FileTransfer) -> Result<(), String> {
        self.with_db(|db| {
            crate::db::lan::insert_file_transfer(db.conn(), &ft).map_err(|e| e.to_string())
        })
    }
    // ============ Backup ============
}