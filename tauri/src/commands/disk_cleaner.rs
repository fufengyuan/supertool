use jwalk::WalkDir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

// ── Constants ──────────────────────────────────────────────
const MAX_DEPTH: usize = 10;
const MAX_ENTRIES: u64 = 500_000;
const CACHE_ESTIMATE_DEPTH: usize = 3;
const DUPLICATE_MAX_FILES: u64 = 100_000;
const CATEGORY_MAX_FILES: u64 = 200_000;

// ── Types ──────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirEntry {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub file_type: String,
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

#[derive(Debug, Deserialize)]
pub struct _DeleteParams {
    pub paths: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DeleteResult {
    pub success: Vec<String>,
    pub failed: Vec<(String, String)>,
    pub total_freed: u64,
}

#[derive(Debug, Serialize)]
pub struct DiskInfo {
    pub mount_point: String,
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Serialize)]
pub struct DuplicateGroup {
    pub key: String,
    pub files: Vec<DirEntry>,
    pub total_size: u64,
    pub wasted_space: u64,
}

// ── Commands ───────────────────────────────────────────────

#[tauri::command(rename_all = "camelCase")]
pub fn get_home_dir() -> Option<String> {
    dirs::home_dir().map(|p| p.to_string_lossy().to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub fn get_disk_info() -> Vec<DiskInfo> {
    #[cfg(target_os = "macos")]
    { get_disk_info_unix("/") }
    #[cfg(target_os = "linux")]
    { get_disk_info_unix("/") }
    #[cfg(target_os = "windows")]
    { get_disk_info_windows() }
}

/// Scan a directory and return its children sorted by size (descending)
#[tauri::command(rename_all = "camelCase")]
pub fn scan_directory(path: String) -> Result<Vec<DirEntry>, String> {
    let dir = PathBuf::from(&path);
    if !dir.exists() {
        return Err(format!("路径不存在: {}", path));
    }
    if !dir.is_dir() {
        return Err(format!("不是目录: {}", path));
    }

    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(err) => return Err(format!("无法读取目录: {}", err)),
    };

    let mut results: Vec<DirEntry> = Vec::new();
    let mut handles = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let entry_path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        let metadata = match entry_path.symlink_metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let file_type = if metadata.is_dir() { "directory" } else { "file" };
        let modified = metadata.modified().ok().map(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        });

        if metadata.is_dir() {
            // Spawn parallel task to compute size+count in one pass
            let p = entry_path.clone();
            let handle = std::thread::spawn(move || {
                let (size, count) = get_dir_info(&p, MAX_DEPTH);
                (entry_path, name, size, modified, count)
            });
            handles.push(handle);
        } else {
            results.push(DirEntry {
                path: entry_path.to_string_lossy().to_string(),
                name,
                size: metadata.len(),
                file_type: file_type.to_string(),
                modified,
                children_count: None,
            });
        }
    }

    // Collect parallel results
    for handle in handles {
        if let Ok((entry_path, name, size, modified, children_count)) = handle.join() {
            results.push(DirEntry {
                path: entry_path.to_string_lossy().to_string(),
                name,
                size,
                file_type: "directory".to_string(),
                modified,
                children_count: Some(children_count),
            });
        }
    }

    results.sort_by(|a, b| b.size.cmp(&a.size));
    Ok(results)
}

/// Scan for large files by category
#[tauri::command(rename_all = "camelCase")]
pub fn scan_by_category(path: String, limit: u32) -> Result<Vec<FileCategory>, String> {
    let dir = PathBuf::from(&path);
    if !dir.exists() {
        return Err(format!("路径不存在: {}", path));
    }

    let limit = limit.max(10); // at least 10
    let max_files = CATEGORY_MAX_FILES;

    let mut file_map: std::collections::HashMap<String, (String, String, u32, u64, Vec<DirEntry>)> =
        std::collections::HashMap::new();

    let count = Arc::new(AtomicU64::new(0));
    let start = Instant::now();

    // Parallel walk with jwalk
    let entries: Vec<_> = WalkDir::new(&dir)
        .max_depth(MAX_DEPTH)
        .skip_hidden(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .take(max_files as usize)
        .collect();

    for entry in entries {
        let current = count.fetch_add(1, Ordering::Relaxed);
        if current >= max_files {
            break;
        }

        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Check timeout: if more than 10 seconds, stop collecting
        if start.elapsed().as_secs() > 10 {
            break;
        }

        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_else(|| "无扩展名".to_string());

        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        let (icon, label) = get_category_info(&ext);
        let modified = entry.metadata().ok().and_then(|m| {
            m.modified()
                .ok()
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64)
        });

        let entry_info = DirEntry {
            path: path.to_string_lossy().to_string(),
            name,
            size,
            file_type: "file".to_string(),
            modified,
            children_count: None,
        };

        let cat = file_map.entry(ext.clone()).or_insert_with(|| (icon.clone(), label.clone(), 0, 0, Vec::new()));
        cat.2 += 1;
        cat.3 += size;
        cat.4.push(entry_info);
    }

    // Sort and limit files within each category
    for (_, (_, _, _, _, files)) in file_map.iter_mut() {
        files.sort_by(|a, b| b.size.cmp(&a.size));
        files.truncate(limit as usize);
    }

    let mut categories: Vec<FileCategory> = file_map
        .into_iter()
        .map(|(ext, (icon, label, count, total_size, files))| FileCategory {
            extension: ext,
            label,
            icon,
            count,
            total_size,
            files,
        })
        .collect();

    categories.sort_by(|a, b| b.total_size.cmp(&a.total_size));
    Ok(categories)
}

/// Get known cache paths for the current OS
#[tauri::command(rename_all = "camelCase")]
pub fn get_cache_paths() -> Vec<CachePath> {
    let mut caches = Vec::new();
    let now = Instant::now();

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            let cache_paths = [
                ("~/Library/Caches", "系统缓存", "macOS 应用程序缓存", true),
                ("~/Library/Caches/com.apple.Safari", "Safari 缓存", "浏览器缓存数据", true),
                ("~/Library/Caches/com.google.Chrome", "Chrome 缓存", "浏览器缓存数据", true),
                ("~/Library/Caches/CloudKit", "iCloud 缓存", "iCloud 同步缓存", true),
                ("~/Library/Developer/Xcode/DerivedData", "Xcode 构建缓存", "Xcode 编译产物", true),
                ("~/Library/Developer/Xcode/iOS DeviceSupport", "Xcode 设备支持", "iOS 设备符号表", true),
                ("~/Library/Caches/Homebrew", "Homebrew 缓存", "包管理器下载缓存", true),
                ("~/Library/Caches/pip", "pip 缓存", "Python 包缓存", true),
                ("~/Library/Caches/CocoaPods", "CocoaPods 缓存", "iOS 依赖缓存", true),
                ("~/Library/Logs", "系统日志", "应用日志文件", true),
                ("~/Library/Caches/com.microsoft.VSCode", "VS Code 缓存", "编辑器缓存", true),
                ("~/Library/Caches/WebKit", "WebKit 缓存", "Web 渲染引擎缓存", true),
                ("~/.npm", "npm 缓存", "Node.js 包缓存", true),
                ("~/.cache", "通用缓存", "跨平台缓存目录", true),
                ("/private/var/log/asl", "系统日志(ASL)", "Apple 系统日志", false),
                ("/System/Volumes/Data/private/var/vm", "虚拟内存交换文件", "swap 文件，重启后自动清理", false),
            ];

            for (rel_path, name, desc, safe) in &cache_paths {
                if now.elapsed().as_secs() > 5 { break; } // overall timeout 5s
                let full_path = rel_path.replace("~/", &format!("{}/", home.to_string_lossy()));
                let p = PathBuf::from(&full_path);
                if !p.exists() { continue; }

                // Quick estimate: only go 3 levels deep for cache dirs
                let size = estimate_dir_size(&p, CACHE_ESTIMATE_DEPTH);
                if size > 0 {
                    caches.push(CachePath {
                        path: full_path,
                        name: name.to_string(),
                        description: desc.to_string(),
                        size,
                        safe_to_clean: *safe,
                    });
                }
            }
        }

        // System-wide caches
        let sys_caches = [
            ("/private/var/folders", "系统临时文件", "macOS 临时文件目录", false),
        ];
        for (path, name, desc, safe) in &sys_caches {
            if now.elapsed().as_secs() > 5 { break; }
            let p = PathBuf::from(path);
            if !p.exists() { continue; }
            let size = estimate_dir_size(&p, CACHE_ESTIMATE_DEPTH);
            if size > 0 {
                caches.push(CachePath {
                    path: path.to_string(),
                    name: name.to_string(),
                    description: desc.to_string(),
                    size,
                    safe_to_clean: *safe,
                });
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(home) = dirs::home_dir() {
            let cache_paths = [
                ("~/.cache", "用户缓存", "XDG 缓存目录", true),
                ("~/.cache/thumbnails", "缩略图缓存", "文件管理器缩略图", true),
                ("~/.npm", "npm 缓存", "Node.js 包缓存", true),
                ("~/.cache/pip", "pip 缓存", "Python 包缓存", true),
                ("~/.local/share/Trash", "回收站", "已删除文件", true),
                ("~/.mozilla/firefox", "Firefox 缓存", "浏览器缓存", true),
                ("~/.config/google-chrome", "Chrome 缓存", "浏览器缓存", true),
                ("~/.cache/v8-compile-cache", "V8 编译缓存", "Node.js/V8 缓存", true),
            ];

            for (rel_path, name, desc, safe) in &cache_paths {
                if now.elapsed().as_secs() > 5 { break; }
                let full_path = rel_path.replace("~/", &format!("{}/", home.to_string_lossy()));
                let p = PathBuf::from(&full_path);
                if !p.exists() { continue; }
                let size = estimate_dir_size(&p, CACHE_ESTIMATE_DEPTH);
                if size > 0 {
                    caches.push(CachePath {
                        path: full_path,
                        name: name.to_string(),
                        description: desc.to_string(),
                        size,
                        safe_to_clean: *safe,
                    });
                }
            }
        }

        // System caches
        let sys_caches = [
            ("/var/cache/apt", "APT 缓存", "Debian/Ubuntu 包缓存", true),
            ("/var/cache/yum", "YUM 缓存", "CentOS/RHEL 包缓存", true),
            ("/var/log", "系统日志", "日志文件", false),
            ("/tmp", "临时文件", "系统临时目录", false),
        ];
        for (path, name, desc, safe) in &sys_caches {
            if now.elapsed().as_secs() > 5 { break; }
            let p = PathBuf::from(path);
            if !p.exists() { continue; }
            let size = estimate_dir_size(&p, CACHE_ESTIMATE_DEPTH);
            if size > 0 {
                caches.push(CachePath {
                    path: path.to_string(),
                    name: name.to_string(),
                    description: desc.to_string(),
                    size,
                    safe_to_clean: *safe,
                });
            }
        }
    }

    caches.sort_by(|a, b| b.size.cmp(&a.size));
    caches
}

/// Delete selected files/folders
#[tauri::command(rename_all = "camelCase")]
pub fn delete_items(paths: Vec<String>) -> DeleteResult {
    let mut success = Vec::new();
    let mut failed: Vec<(String, String)> = Vec::new();
    let mut total_freed: u64 = 0;

    for path_str in &paths {
        let path = PathBuf::from(path_str);
        if !path.exists() {
            failed.push((path_str.clone(), "路径不存在".to_string()));
            continue;
        }

        // Get size before deleting
        let size = if path.is_dir() {
            get_dir_info(&path, MAX_DEPTH).0
        } else {
            path.metadata().map(|m| m.len()).unwrap_or(0)
        };

        let result = if path.is_dir() {
            fs::remove_dir_all(&path)
        } else {
            fs::remove_file(&path)
        };

        match result {
            Ok(_) => {
                total_freed += size;
                success.push(path_str.clone());
            }
            Err(e) => {
                failed.push((path_str.clone(), e.to_string()));
            }
        }
    }

    DeleteResult { success, failed, total_freed }
}

/// Analyze duplicates (find files with same name+size)
#[tauri::command(rename_all = "camelCase")]
pub fn find_duplicates(path: String, min_size: u64) -> Result<Vec<DuplicateGroup>, String> {
    let dir = PathBuf::from(&path);
    if !dir.exists() {
        return Err(format!("路径不存在: {}", path));
    }

    let min_size = min_size.max(1);
    let start = Instant::now();

    let mut file_map: std::collections::HashMap<String, Vec<DirEntry>> =
        std::collections::HashMap::new();

    // Parallel walk with jwalk
    let entries: Vec<_> = WalkDir::new(&dir)
        .max_depth(MAX_DEPTH)
        .skip_hidden(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .take(DUPLICATE_MAX_FILES as usize)
        .collect();

    for entry in entries {
        if start.elapsed().as_secs() > 30 { break; } // 30s timeout

        let path = entry.path();
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let size = meta.len();
        if size < min_size { continue; }

        let name = entry.file_name().to_string_lossy().to_string();
        let key = format!("{}_{}", name, size);

        let modified = meta.modified().ok().map(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        });

        let entry_info = DirEntry {
            path: path.to_string_lossy().to_string(),
            name,
            size,
            file_type: "file".to_string(),
            modified,
            children_count: None,
        };

        file_map.entry(key).or_default().push(entry_info);
    }

    let groups: Vec<DuplicateGroup> = file_map
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(key, files)| {
            let total_size: u64 = files.iter().map(|f| f.size).sum();
            let wasted = total_size.saturating_sub(files[0].size);
            DuplicateGroup { key, files, total_size, wasted_space: wasted }
        })
        .collect();

    let mut groups = groups;
    groups.sort_by(|a, b| b.wasted_space.cmp(&a.wasted_space));
    Ok(groups)
}

// ── Helper functions ──────────────────────────────────────

/// Get (total_size, entry_count) in one pass with depth limit.
/// Uses jwalk for parallel I/O for large directories.
fn get_dir_info(dir: &Path, max_depth: usize) -> (u64, u32) {
    let total = Arc::new(AtomicU64::new(0));
    let count = Arc::new(AtomicU64::new(0));

    let walker = WalkDir::new(dir)
        .max_depth(max_depth)
        .skip_hidden(false);

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if meta.is_file() {
            total.fetch_add(meta.len(), Ordering::Relaxed);
            count.fetch_add(1, Ordering::Relaxed);
        }
        if count.load(Ordering::Relaxed) >= MAX_ENTRIES {
            break;
        }
    }

    (total.load(Ordering::Relaxed), count.load(Ordering::Relaxed) as u32)
}

/// Quick size estimation for cache directories (shallow depth, fast)
fn estimate_dir_size(dir: &Path, max_depth: usize) -> u64 {
    let mut total: u64 = 0;
    let mut count: u64 = 0;

    let walker = WalkDir::new(dir).max_depth(max_depth).skip_hidden(false);

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        if count >= 10_000 { break; } // limit per directory
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if meta.is_file() {
            total += meta.len();
            count += 1;
        }
    }

    total
}

fn get_category_info(ext: &str) -> (String, String) {
    match ext {
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" => ("📦".into(), "压缩包".into()),
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" => ("🎬".into(), "视频".into()),
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" => ("🎵".into(), "音频".into()),
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" | "ico" => ("🖼️".into(), "图片".into()),
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" => ("📄".into(), "文档".into()),
        "dmg" | "iso" | "img" => ("💿".into(), "磁盘镜像".into()),
        "apk" | "ipa" => ("📱".into(), "安装包".into()),
        "exe" | "msi" | "app" => ("⚙️".into(), "可执行文件".into()),
        "js" | "ts" | "py" | "go" | "rs" | "java" | "cpp" | "c" | "h" => ("💻".into(), "源代码".into()),
        "log" => ("📋".into(), "日志".into()),
        "tmp" | "temp" | "cache" => ("🗑️".into(), "临时文件".into()),
        "woff" | "woff2" | "ttf" | "otf" | "eot" => ("🔤".into(), "字体".into()),
        _ => ("📁".into(), ext.to_string()),
    }
}

// ── OS-specific helpers ────────────────────────────────────

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn get_disk_info_unix(mount: &str) -> Vec<DiskInfo> {
    use std::process::Command;
    let output = Command::new("df").arg("-k").arg(mount).output().ok();

    if let Some(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        if lines.len() >= 2 {
            let parts: Vec<&str> = lines[1].split_whitespace().collect();
            if parts.len() >= 5 {
                let total: u64 = parts[1].parse().unwrap_or(0) * 1024;
                let used: u64 = parts[2].parse().unwrap_or(0) * 1024;
                let free: u64 = parts[3].parse().unwrap_or(0) * 1024;
                let usage_str = parts[4].trim_end_matches('%');
                let usage_percent: f64 = usage_str.parse().unwrap_or(0.0);
                return vec![DiskInfo {
                    mount_point: mount.to_string(),
                    total,
                    used,
                    free,
                    usage_percent,
                }];
            }
        }
    }
    Vec::new()
}

#[cfg(target_os = "windows")]
fn get_disk_info_windows() -> Vec<DiskInfo> {
    Vec::new()
}
