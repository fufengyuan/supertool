use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct DirEntry {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub file_type: String, // "file" or "directory"
    pub modified: Option<u64>, // unix timestamp millis
    pub children_count: Option<u32>, // for directories
}

#[derive(Debug, Serialize)]
pub struct FileCategory {
    pub extension: String,
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
pub struct DeleteParams {
    pub paths: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DeleteResult {
    pub success: Vec<String>,
    pub failed: Vec<(String, String)>, // path, error
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

/// Get the user's home directory
#[tauri::command(rename_all = "camelCase")]
pub fn get_home_dir() -> Option<String> {
    dirs::home_dir().map(|p| p.to_string_lossy().to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub fn get_disk_info() -> Vec<DiskInfo> {
    #[cfg(target_os = "macos")]
    {
        get_disk_info_unix("/")
    }
    #[cfg(target_os = "linux")]
    {
        get_disk_info_unix("/")
    }
    #[cfg(target_os = "windows")]
    {
        get_disk_info_windows()
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn get_disk_info_unix(mount: &str) -> Vec<DiskInfo> {
    use std::process::Command;
    let output = Command::new("df")
        .arg("-k")
        .arg(mount)
        .output()
        .ok();

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
    // Use GetDiskFreeSpaceEx via winapi or fallback to empty
    Vec::new()
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

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let entry_path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip permission-denied entries silently
        let metadata = match entry_path.symlink_metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let file_type = if metadata.is_dir() {
            "directory"
        } else {
            "file"
        };

        let size = if metadata.is_dir() {
            match calculate_dir_size(&entry_path) {
                Ok(s) => s,
                Err(_) => 0,
            }
        } else {
            metadata.len()
        };

        let modified = metadata.modified().ok().map(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        });

        let children_count = if metadata.is_dir() {
            match count_children(&entry_path) {
                Ok(c) => Some(c),
                Err(_) => None,
            }
        } else {
            None
        };

        results.push(DirEntry {
            path: entry_path.to_string_lossy().to_string(),
            name,
            size,
            file_type: file_type.to_string(),
            modified,
            children_count,
        });
    }

    // Sort by size descending
    results.sort_by(|a, b| b.size.cmp(&a.size));

    Ok(results)
}

/// Scan for large files by category
#[tauri::command(rename_all = "camelCase")]
pub fn scan_by_category(
    path: String,
    limit: u32,
) -> Result<Vec<FileCategory>, String> {
    let dir = PathBuf::from(&path);
    if !dir.exists() {
        return Err(format!("路径不存在: {}", path));
    }

    let mut file_map: std::collections::HashMap<String, (String, u32, u64, Vec<DirEntry>)> =
        std::collections::HashMap::new();

    walk_for_categories(&dir, &mut file_map, limit);

    let mut categories: Vec<FileCategory> = file_map
        .into_iter()
        .map(|(ext, (icon, count, total_size, files))| FileCategory {
            extension: ext,
            icon: icon.to_string(),
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
                let full_path = rel_path.replace("~/", &format!("{}/", home.to_string_lossy()));
                let size = calculate_dir_size(PathBuf::from(&full_path).as_path()).unwrap_or(0);
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
            let size = calculate_dir_size(PathBuf::from(path).as_path()).unwrap_or(0);
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
                let full_path = rel_path.replace("~/", &format!("{}/", home.to_string_lossy()));
                let size = calculate_dir_size(PathBuf::from(&full_path).as_path()).unwrap_or(0);
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
            let size = calculate_dir_size(PathBuf::from(path).as_path()).unwrap_or(0);
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

    #[cfg(target_os = "windows")]
    {
        if let Some(home) = dirs::home_dir() {
            let cache_paths = [
                ("~\\AppData\\Local\\Temp", "临时文件", "Windows 临时文件", true),
                ("~\\AppData\\Local\\Microsoft\\Windows\\INetCache", "IE 缓存", "Internet Explorer 缓存", true),
                ("~\\AppData\\Local\\Google\\Chrome\\User Data\\Default\\Cache", "Chrome 缓存", "浏览器缓存", true),
                ("~\\AppData\\Roaming\\Code\\Cache", "VS Code 缓存", "编辑器缓存", true),
                ("~\\.nuget\\packages", "NuGet 缓存", ".NET 包缓存", true),
                ("~\\.cargo\\registry", "Cargo 缓存", "Rust 包缓存", true),
                ("~\\AppData\\Local\\npm-cache", "npm 缓存", "Node.js 包缓存", true),
                ("~\\AppData\\Local\\pip\\Cache", "pip 缓存", "Python 包缓存", true),
                ("C:\\Windows\\Temp", "系统临时文件", "Windows 系统临时目录", false),
                ("C:\\Windows\\Prefetch", "预取缓存", "程序启动加速缓存", true),
            ];

            for (rel_path, name, desc, safe) in &cache_paths {
                let full_path = rel_path.replace("~\\", &format!("{}\\", home.to_string_lossy()));
                let size = calculate_dir_size(PathBuf::from(&full_path).as_path()).unwrap_or(0);
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
    }

    // Sort by size descending
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
            calculate_dir_size(&path).unwrap_or(0)
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

    DeleteResult {
        success,
        failed,
        total_freed,
    }
}

/// Analyze duplicates (find files with same name+size)
#[tauri::command(rename_all = "camelCase")]
pub fn find_duplicates(path: String, min_size: u64) -> Result<Vec<DuplicateGroup>, String> {
    let dir = PathBuf::from(&path);
    if !dir.exists() {
        return Err(format!("路径不存在: {}", path));
    }

    let mut file_map: std::collections::HashMap<String, Vec<DirEntry>> =
        std::collections::HashMap::new();

    walk_for_duplicates(&dir, &mut file_map, min_size);

    let groups: Vec<DuplicateGroup> = file_map
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(key, files)| {
            let total_size: u64 = files.iter().map(|f| f.size).sum();
            let wasted = total_size.saturating_sub(files[0].size);
            DuplicateGroup {
                key,
                files,
                total_size,
                wasted_space: wasted,
            }
        })
        .collect();

    // Sort by wasted space descending
    let mut groups = groups;
    groups.sort_by(|a, b| b.wasted_space.cmp(&a.wasted_space));

    Ok(groups)
}

#[derive(Debug, Serialize)]
pub struct DuplicateGroup {
    pub key: String,
    pub files: Vec<DirEntry>,
    pub total_size: u64,
    pub wasted_space: u64,
}

// --- Helper functions ---

fn calculate_dir_size(dir: &Path) -> io::Result<u64> {
    let mut total = 0;
    // Use a simple walker, skip permission errors
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    if let Ok(s) = calculate_dir_size(&path) {
                        total += s;
                    }
                } else if let Ok(meta) = entry.metadata() {
                    total += meta.len();
                }
            }
        }
    }
    Ok(total)
}

fn count_children(dir: &Path) -> io::Result<u32> {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                count += 1;
            }
        }
    }
    Ok(count)
}

fn walk_for_categories(
    dir: &Path,
    map: &mut std::collections::HashMap<String, (String, u32, u64, Vec<DirEntry>)>,
    limit: u32,
) {
    let mut count: u32 = 0;
    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&current) {
            for entry in entries {
                if count >= limit * 100 {
                    return; // safety limit
                }
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_dir() {
                            stack.push(path);
                        } else {
                            count += 1;
                            let ext = path
                                .extension()
                                .map(|e| e.to_string_lossy().to_lowercase())
                                .unwrap_or_else(|| "无扩展名".to_string());

                            let (icon, _) = get_category_info(&ext);

                            let entry_info = DirEntry {
                                path: path.to_string_lossy().to_string(),
                                name: entry.file_name().to_string_lossy().to_string(),
                                size: metadata.len(),
                                file_type: "file".to_string(),
                                modified: metadata.modified().ok().map(|t| {
                                    t.duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_millis() as u64
                                }),
                                children_count: None,
                            };

                            let entry_map = map
                                .entry(ext.clone())
                                .or_insert_with(|| (icon.clone(), 0, 0, Vec::new()));
                            entry_map.1 += 1;
                            entry_map.2 += metadata.len();
                            entry_map.3.push(entry_info);
                        }
                    }
                }
            }
        }
    }

    // Sort files within each category and limit
    for (_, (_, _, _, files)) in map.iter_mut() {
        files.sort_by(|a, b| b.size.cmp(&a.size));
        files.truncate(limit as usize);
    }
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

fn walk_for_duplicates(
    dir: &Path,
    map: &mut std::collections::HashMap<String, Vec<DirEntry>>,
    min_size: u64,
) {
    let mut stack = vec![dir.to_path_buf()];
    let mut processed = 0;

    while let Some(current) = stack.pop() {
        if processed >= 10000 {
            return; // safety limit
        }
        if let Ok(entries) = fs::read_dir(&current) {
            for entry in entries {
                if let Ok(entry) = entry {
                    processed += 1;
                    let path = entry.path();
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_dir() {
                            stack.push(path);
                        } else if metadata.len() >= min_size {
                            let key = format!(
                                "{}_{}",
                                entry.file_name().to_string_lossy(),
                                metadata.len()
                            );
                            let entry_info = DirEntry {
                                path: path.to_string_lossy().to_string(),
                                name: entry.file_name().to_string_lossy().to_string(),
                                size: metadata.len(),
                                file_type: "file".to_string(),
                                modified: metadata.modified().ok().map(|t| {
                                    t.duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_millis() as u64
                                }),
                                children_count: None,
                            };
                            map.entry(key).or_default().push(entry_info);
                        }
                    }
                }
            }
        }
    }
}
