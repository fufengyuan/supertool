//! Disk Cleaner — thin Tauri wrapper over `supertool_core::logic::disk_cleaner`.
//!
//! 架构约定：磁盘清理的扫描/分类/缓存探测/删除/去重等**共同逻辑**统一在
//! core（`core/src/logic/disk_cleaner.rs`）。本文件只保留 Tauri 特有的命令注册与
//! 参数透传，不再重复实现任何扫描/删除逻辑。
//!
//! 曾有一份 730 行的完整实现直接内联在 tauri 侧（scan_directory/scan_by_category/
//! get_cache_paths/delete_items/find_duplicates 及其全部辅助函数），与 core 逐行重复。
//! 已删除，改为转发 core——保证 GUI / 未来 CLI 行为一致。

use supertool_core::logic::disk_cleaner as core_disk;

// ── Types ───────────────────────────────────────────────────
// 直接复用 core 的类型，避免两套结构体漂移（曾各定义一份 DirEntry/DiskInfo 等）。

pub use supertool_core::logic::disk_cleaner::{
    CachePath, DeleteResult, DirEntry, DiskInfo, DuplicateGroup, FileCategory,
};

// ── Commands ────────────────────────────────────────────────

#[tauri::command(rename_all = "camelCase")]
pub fn get_home_dir() -> Option<String> {
    core_disk::get_home_dir()
}

#[tauri::command(rename_all = "camelCase")]
pub fn get_disk_info() -> Vec<DiskInfo> {
    core_disk::get_disk_info()
}

#[tauri::command(rename_all = "camelCase")]
pub fn scan_directory(path: String) -> Result<Vec<DirEntry>, String> {
    core_disk::scan_directory(path)
}

#[tauri::command(rename_all = "camelCase")]
pub fn scan_by_category(path: String, limit: u32) -> Result<Vec<FileCategory>, String> {
    core_disk::scan_by_category(path, limit)
}

#[tauri::command(rename_all = "camelCase")]
pub fn get_cache_paths() -> Vec<CachePath> {
    core_disk::get_cache_paths()
}

#[tauri::command(rename_all = "camelCase")]
pub fn delete_items(paths: Vec<String>) -> DeleteResult {
    core_disk::delete_items(paths)
}

#[tauri::command(rename_all = "camelCase")]
pub fn find_duplicates(path: String, min_size: u64) -> Result<Vec<DuplicateGroup>, String> {
    core_disk::find_duplicates(path, min_size)
}
