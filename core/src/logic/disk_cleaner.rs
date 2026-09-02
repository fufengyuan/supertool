//! Disk Cleaner module — ported from Tauri commands
//! Scans directories, finds duplicates, deletes items, caches paths, disk info.
use std::path::Path;
use std::time::Instant;
use serde::Serialize;

// ── Constants ──
const MAX_DEPTH: usize = 10;
const MAX_ENTRIES: u64 = 500_000;
const CACHE_ESTIMATE_DEPTH: usize = 3;
const DUPLICATE_MAX_FILES: u64 = 100_000;
const CATEGORY_MAX_FILES: u64 = 200_000;

// ── Public Types ──

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirEntry {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub file_type: String, // "file" | "directory" | "symlink"
    pub modified: Option<u64>,
    pub children_count: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileCategory {
    pub extension: String,
    pub label: String,
    pub icon: String,
    pub count: u32,
    pub total_size: u64,
    pub files: Vec<DirEntry>,
}

#[derive(Debug, Serialize)]
pub struct CachePath {
    pub path: String,
    pub name: String,
    pub description: String,
    pub size: u64,
    pub safe_to_clean: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteResult {
    pub success: Vec<String>,
    pub failed: Vec<(String, String)>,
    pub total_freed: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    pub mount_point: String,
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateGroup {
    pub key: String,
    pub files: Vec<DirEntry>,
    pub total_size: u64,
    pub wasted_space: u64,
}

// ── Public Functions ──

pub fn get_home_dir() -> Option<String> {
    dirs::home_dir().map(|p| p.to_string_lossy().to_string())
}

pub fn get_disk_info() -> Vec<DiskInfo> {
    get_disk_info_impl()
}

pub fn scan_directory(path: String) -> Result<Vec<DirEntry>, String> {
    scan_directory_impl(&path)
}

pub fn scan_by_category(path: String, limit: u32) -> Result<Vec<FileCategory>, String> {
    scan_by_category_impl(&path, limit)
}

pub fn get_cache_paths() -> Vec<CachePath> {
    get_cache_paths_impl()
}

pub fn delete_items(paths: Vec<String>) -> DeleteResult {
    delete_items_impl(&paths)
}

pub fn find_duplicates(path: String, min_size: u64) -> Result<Vec<DuplicateGroup>, String> {
    find_duplicates_impl(&path, min_size)
}

// ── Implementations ──

fn get_disk_info_impl() -> Vec<DiskInfo> {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        let output = std::process::Command::new("df")
            .args(["-k", "/"])
            .output().ok();
        let stdout = match output { Some(o) => String::from_utf8_lossy(&o.stdout).to_string(), None => return vec![] };
        let lines: Vec<&str> = stdout.lines().collect();
        if lines.len() < 2 { return vec![]; }
        let parts: Vec<&str> = lines[1].split_whitespace().collect();
        if parts.len() < 4 { return vec![]; }
        let total = parts[1].parse::<u64>().unwrap_or(0) * 1024;
        let used = parts[2].parse::<u64>().unwrap_or(0) * 1024;
        let free = parts[3].parse::<u64>().unwrap_or(0) * 1024;
        let usage_percent = if total > 0 { (used as f64 / total as f64) * 100.0 } else { 0.0 };
        vec![DiskInfo { mount_point: "/".to_string(), total, used, free, usage_percent }]
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    { vec![] }
}

fn scan_directory_impl(path: &str) -> Result<Vec<DirEntry>, String> {
    let dir = Path::new(path);
    if !dir.exists() { return Err("路径不存在".to_string()); }
    if !dir.is_dir() { return Err("路径不是目录".to_string()); }

    let mut entries = Vec::new();
    let mut rd = std::fs::read_dir(dir).map_err(|e| format!("读取目录失败: {}", e))?;
    while let Some(Ok(entry)) = rd.next() {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path().to_string_lossy().to_string();
        let ft = entry.file_type().map_err(|e| format!("读取类型失败: {}", e))?;
        let file_type = if ft.is_dir() { "directory" } else if ft.is_symlink() { "symlink" } else { "file" };
        let modified = entry.metadata().ok().and_then(|m| m.modified().ok()).map(|t| t.duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_secs()).unwrap_or(0));
        let (size, children_count) = if ft.is_dir() {
            let (s, c) = get_dir_info(&entry.path(), MAX_DEPTH);
            (s, Some(c))
        } else {
            let size = entry.metadata().ok().map(|m| m.len()).unwrap_or(0);
            (size, None)
        };
        entries.push(DirEntry { path, name, size, file_type: file_type.to_string(), modified, children_count });
    }
    Ok(entries)
}

fn get_dir_info(dir: &Path, max_depth: usize) -> (u64, u32) {
    let start = Instant::now();
    let mut total_size = 0u64;
    let mut total_files = 0u32;

    for entry in jwalk::WalkDir::new(dir).max_depth(max_depth).into_iter().filter_map(|r| r.ok()) {
        if total_files >= MAX_ENTRIES as u32 || start.elapsed().as_secs() > 30 {
            break;
        }
        let meta = &entry.metadata();
        if let Ok(meta) = meta {
            total_size += meta.len();
            total_files += 1;
        }
    }
    (total_size, total_files)
}

fn estimate_dir_size(dir: &Path, max_depth: usize) -> u64 {
    let mut total = 0u64;
    let mut count = 0u64;
    let iter = jwalk::WalkDir::new(dir).max_depth(max_depth);
    for entry in iter.into_iter().filter_map(|r| r.ok()) {
            if count >= 10_000 { break; }
            if let Ok(meta) = &entry.metadata() {
                total += meta.len();
                count += 1;
            }
    }
    total
}

fn get_category_info(ext: &str) -> (String, String) {
    match ext.to_lowercase().as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" | "ico" | "bmp" => ("🖼".to_string(), "图片".to_string()),
        "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" => ("🎬".to_string(), "视频".to_string()),
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" => ("🎵".to_string(), "音频".to_string()),
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" => ("📦".to_string(), "压缩包".to_string()),
        "pdf" => ("📄".to_string(), "PDF".to_string()),
        "doc" | "docx" => ("📝".to_string(), "文档".to_string()),
        "xls" | "xlsx" | "csv" => ("📊".to_string(), "电子表格".to_string()),
        "ppt" | "pptx" => ("📽".to_string(), "演示文稿".to_string()),
        "exe" | "dmg" | "pkg" | "msi" | "AppImage" | "deb" | "rpm" => ("⚙".to_string(), "可执行文件".to_string()),
        "iso" | "img" => ("💿".to_string(), "镜像文件".to_string()),
        "log" => ("📋".to_string(), "日志".to_string()),
        "tmp" | "temp" | "swp" | "bak" => ("🗑".to_string(), "临时文件".to_string()),
        "git" | "svn" | "hg" => ("🔧".to_string(), "版本控制".to_string()),
        "node_modules" | "npm" | "yarn" | "pnpm" | "bower_components" => ("📦".to_string(), "前端依赖".to_string()),
        "target" | "build" | "dist" | ".next" | ".nuxt" | "out" => ("🔨".to_string(), "构建产物".to_string()),
        "DS_Store" | "Thumbs.db" | ".directory" => ("👻".to_string(), "系统文件".to_string()),
        "dll" | "so" | "dylib" => ("🔗".to_string(), "动态库".to_string()),
        "ttf" | "otf" | "woff" | "woff2" => ("🔤".to_string(), "字体".to_string()),
        _ => ("📄".to_string(), "其他".to_string()),
    }
}

fn scan_by_category_impl(path: &str, limit: u32) -> Result<Vec<FileCategory>, String> {
    let dir = Path::new(path);
    if !dir.exists() { return Err("路径不存在".to_string()); }
    let mut categories: std::collections::HashMap<String, Vec<DirEntry>> = std::collections::HashMap::new();
    let mut count = 0u64;

    let iter = jwalk::WalkDir::new(dir).max_depth(MAX_DEPTH);
    for entry in iter.into_iter().filter_map(|r| r.ok()) {
            if count >= CATEGORY_MAX_FILES { break; }
            let meta = entry.metadata().ok();
            if let Some(meta) = meta {
                if meta.is_dir() { count += 1; continue; }
                let path_str = entry.path().to_string_lossy().to_string();
                let name = entry.file_name().to_string_lossy().to_string();
                let ext = entry.path().extension().and_then(|e| e.to_str()).unwrap_or("").to_string();
                let size = meta.len();
                let modified = meta.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_secs())).unwrap_or(0);
                let de = DirEntry {
                    path: path_str, name,
                    size, file_type: "file".to_string(),
                    modified: Some(modified), children_count: None,
                };
                categories.entry(ext).or_default().push(de);
                count += 1;
            }
        }

    let limit = if limit > 0 { limit as usize } else { 20 };
    let mut result: Vec<FileCategory> = categories.into_iter()
        .filter(|(_, files)| !files.is_empty())
        .map(|(ext, mut files)| {
            let (icon, label) = get_category_info(&ext);
            let total_size: u64 = files.iter().map(|f| f.size).sum();
            // Sort files by size descending, take top 100
            files.sort_by(|a, b| b.size.cmp(&a.size));
            files.truncate(100);
            FileCategory { extension: ext, label, icon, count: files.len() as u32, total_size, files }
        })
        .collect();
    result.sort_by(|a, b| b.total_size.cmp(&a.total_size));
    result.truncate(limit);
    Ok(result)
}

fn get_cache_paths_impl() -> Vec<CachePath> {
    let mut paths = Vec::new();
    let home = dirs::home_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
    if home.is_empty() { return paths; }

    // macOS specific
    #[cfg(target_os = "macos")]
    {
        paths.push(CachePath { path: format!("{}/Library/Caches", home), name: "系统缓存".into(), description: "macOS 应用程序缓存".into(), size: 0, safe_to_clean: true });
        paths.push(CachePath { path: format!("{}/Library/Logs", home), name: "系统日志".into(), description: "应用日志文件".into(), size: 0, safe_to_clean: true });
        paths.push(CachePath { path: format!("{}/Library/Developer/Xcode/DerivedData", home), name: "Xcode 构建缓存".into(), description: "Xcode 编译产物".into(), size: 0, safe_to_clean: true });
        paths.push(CachePath { path: format!("{}/Library/Caches/Google/Chrome", home), name: "Chrome 缓存".into(), description: "浏览器缓存数据".into(), size: 0, safe_to_clean: true });
        paths.push(CachePath { path: format!("{}/Library/Application Support/Code/CachedData", home), name: "VS Code 缓存".into(), description: "编辑器缓存数据".into(), size: 0, safe_to_clean: true });
        paths.push(CachePath { path: format!("{}/Library/Caches/com.spotify.client", home), name: "Spotify 缓存".into(), description: "音乐缓存".into(), size: 0, safe_to_clean: true });
        paths.push(CachePath { path: format!("{}/Library/Caches/Safari", home), name: "Safari 缓存".into(), description: "浏览器缓存数据".into(), size: 0, safe_to_clean: true });
        paths.push(CachePath { path: format!("{}/Library/Caches/com.apple.AppStore", home), name: "App Store 缓存".into(), description: "应用商店缓存".into(), size: 0, safe_to_clean: true });
        paths.push(CachePath { path: format!("{}/Library/Caches/Homebrew", home), name: "Homebrew 缓存".into(), description: "包管理器缓存".into(), size: 0, safe_to_clean: true });
        paths.push(CachePath { path: format!("{}/Library/Caches/com.apple.Safari", home), name: "Safari 缓存".into(), description: "浏览器缓存数据".into(), size: 0, safe_to_clean: true });
        paths.push(CachePath { path: format!("{}/Library/Developer/Xcode/iOS DeviceSupport", home), name: "Xcode 设备支持".into(), description: "iOS 设备符号表".into(), size: 0, safe_to_clean: true });
        paths.push(CachePath { path: format!("{}/Library/Caches/pip", home), name: "pip 缓存".into(), description: "Python 包缓存".into(), size: 0, safe_to_clean: true });
        paths.push(CachePath { path: format!("{}/Library/Caches/CocoaPods", home), name: "CocoaPods 缓存".into(), description: "iOS 依赖缓存".into(), size: 0, safe_to_clean: true });
        paths.push(CachePath { path: format!("{}/Library/Caches/com.microsoft.VSCode", home), name: "VS Code 缓存".into(), description: "编辑器缓存".into(), size: 0, safe_to_clean: true });
        paths.push(CachePath { path: format!("{}/Library/Caches/WebKit", home), name: "WebKit 缓存".into(), description: "Web 渲染引擎缓存".into(), size: 0, safe_to_clean: true });
    }
    // Linux specific
    #[cfg(target_os = "linux")]
    {
        paths.push(CachePath { path: format!("{}/.cache", home), name: "用户缓存".into(), description: "XDG 缓存目录".into(), size: 0, safe_to_clean: true });
        paths.push(CachePath { path: format!("{}/.cache/thumbnails", home), name: "缩略图缓存".into(), description: "文件管理器缩略图".into(), size: 0, safe_to_clean: true });
        paths.push(CachePath { path: format!("{}/.cache/pip", home), name: "pip 缓存".into(), description: "Python 包缓存".into(), size: 0, safe_to_clean: true });
        paths.push(CachePath { path: format!("{}/.local/share/Trash", home), name: "回收站".into(), description: "已删除文件".into(), size: 0, safe_to_clean: true });
        paths.push(CachePath { path: format!("{}/.mozilla/firefox", home), name: "Firefox 缓存".into(), description: "浏览器缓存".into(), size: 0, safe_to_clean: true });
        paths.push(CachePath { path: format!("{}/.config/google-chrome", home), name: "Chrome 缓存".into(), description: "浏览器缓存".into(), size: 0, safe_to_clean: true });
        paths.push(CachePath { path: format!("{}/.cache/v8-compile-cache", home), name: "V8 编译缓存".into(), description: "Node.js/V8 缓存".into(), size: 0, safe_to_clean: true });
        paths.push(CachePath { path: format!("{}/.npm", home), name: "npm 缓存".into(), description: "Node.js 包缓存".into(), size: 0, safe_to_clean: true });
        paths.push(CachePath { path: format!("{}/.cache/chromium", home), name: "Chromium 缓存".into(), description: "Chromium 缓存".into(), size: 0, safe_to_clean: true });
        paths.push(CachePath { path: "/var/cache/apt".into(), name: "APT 缓存".into(), description: "Debian/Ubuntu 包缓存".into(), size: 0, safe_to_clean: true });
        paths.push(CachePath { path: "/var/cache/yum".into(), name: "YUM 缓存".into(), description: "CentOS/RHEL 包缓存".into(), size: 0, safe_to_clean: true });
        paths.push(CachePath { path: "/tmp".into(), name: "临时文件".into(), description: "系统临时目录".into(), size: 0, safe_to_clean: false });
    }
    // Common
    paths.push(CachePath { path: format!("{}/.cargo/registry/cache", home), name: "Cargo 缓存".into(), description: "Rust 包缓存".into(), size: 0, safe_to_clean: true });
    paths.push(CachePath { path: format!("{}/.npm/_cacache", home), name: "npm 缓存".into(), description: "npm 包缓存".into(), size: 0, safe_to_clean: true });
    paths.push(CachePath { path: format!("{}/.yarn/berry/cache", home), name: "Yarn 缓存".into(), description: "Yarn 包缓存".into(), size: 0, safe_to_clean: true });
    paths.push(CachePath { path: format!("{}/.nvm/versions", home), name: "NVM 版本".into(), description: "Node 版本".into(), size: 0, safe_to_clean: false });
    paths.push(CachePath { path: format!("{}/.sdkman", home), name: "SDKMAN".into(), description: "SDK 管理器".into(), size: 0, safe_to_clean: false });
    paths.push(CachePath { path: format!("{}/Downloads", home), name: "下载文件夹".into(), description: "下载文件夹".into(), size: 0, safe_to_clean: false });

    // Estimate sizes
    for p in &mut paths {
        let dir = Path::new(&p.path);
        if dir.exists() {
            p.size = estimate_dir_size(dir, CACHE_ESTIMATE_DEPTH);
        }
    }
    paths.sort_by(|a, b| b.size.cmp(&a.size));
    paths
}

fn delete_items_impl(paths: &[String]) -> DeleteResult {
    let mut success = Vec::new();
    let mut failed = Vec::new();
    let mut total_freed = 0u64;

    for p in paths {
        let path = Path::new(p);
        if !path.exists() {
            failed.push((p.clone(), "路径不存在".to_string()));
            continue;
        }
        // Get size before deletion
        if path.is_dir() {
            total_freed += estimate_dir_size(path, MAX_DEPTH);
        } else {
            total_freed += path.metadata().map(|m| m.len()).unwrap_or(0);
        }
        // Delete
        let result = if path.is_dir() {
            std::fs::remove_dir_all(path)
        } else {
            std::fs::remove_file(path)
        };
        match result {
            Ok(_) => success.push(p.clone()),
            Err(e) => {
                total_freed -= estimate_dir_size(path, MAX_DEPTH);
                failed.push((p.clone(), format!("{}", e)));
            }
        }
    }
    DeleteResult { success, failed, total_freed }
}

fn find_duplicates_impl(path: &str, min_size: u64) -> Result<Vec<DuplicateGroup>, String> {
    let dir = Path::new(path);
    if !dir.exists() { return Err("路径不存在".to_string()); }

    let mut file_map: std::collections::HashMap<String, Vec<DirEntry>> = std::collections::HashMap::new();
    let mut count = 0u64;

    let iter = jwalk::WalkDir::new(dir).max_depth(MAX_DEPTH);
    for entry in iter.into_iter().filter_map(|r| r.ok()) {
            if count >= DUPLICATE_MAX_FILES { break; }
            let meta = entry.metadata().ok();
            if let Some(meta) = meta {
                if meta.is_dir() { count += 1; continue; }
                if meta.len() < min_size { count += 1; continue; }
                let name = entry.file_name().to_string_lossy().to_string();
                let path_str = entry.path().to_string_lossy().to_string();
                let size = meta.len();
                let key = format!("{}:{}", name, size);
                let de = DirEntry {
                    path: path_str, name, size,
                    file_type: "file".to_string(),
                    modified: meta.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_secs())),
                    children_count: None,
                };
                file_map.entry(key).or_default().push(de);
                count += 1;
            }
        }

    let mut groups: Vec<DuplicateGroup> = file_map.into_iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(key, files)| {
            let total_size: u64 = files.iter().map(|f| f.size).sum();
            let wasted_space = total_size - files[0].size;
            DuplicateGroup { key, files, total_size, wasted_space }
        })
        .collect();
    groups.sort_by(|a, b| b.wasted_space.cmp(&a.wasted_space));
    Ok(groups)
}
