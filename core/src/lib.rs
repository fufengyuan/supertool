/// SuperTool Core Library
/// Shared business logic, database operations, and models used by both
/// the Tauri GUI and the CLI (stool).
pub mod db;
pub mod db_ops;
pub mod db_pool;
pub mod encryption;
pub mod lan_emitter;
pub mod lan_service;
pub mod logic;

// Re-export the main entry point
pub use db::Database;
pub use logic::CoreService;
