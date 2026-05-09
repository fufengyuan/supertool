/// SuperTool Core Library
/// Shared business logic, database operations, and models used by both
/// the Tauri GUI and the CLI (stool).
pub mod db;
pub mod encryption;
pub mod logic;

// Re-export the main entry point
pub use logic::CoreService;
pub use db::Database;
