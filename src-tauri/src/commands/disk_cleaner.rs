use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Clone)]
pub struct DirEntry {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub file_type: String,
    pub modified: Option<u64>,
    pub children_count: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct FileCategory {
    pub extension: String,
    pub icon: String,
    pub label: String,
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

// ─── Home Dir ───

#[tauri::command(rename_all = "camelCase")]
pub fn get_home_dir() -> String {
    dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

// ─── Disk Info ───

#[tauri::command(rename_all = "camelCase")]
pub fn get_disk_info() -> Vec<DiskInfo> {
    #[cfg(target_os = "macos")]
    return get_disk_info_unix("/");
    #[cfg(target_os = "linux")]
    return get_disk_info_unix("/");
    #[cfg(target_os = "windows")]
    return Vec::new();
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn get_disk_info_unix(mount: &str) -> Vec<DiskInfo> {
    use std::process::Command;
    let output = match Command::new("df").arg("-k").arg(mount).output() {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    if lines.len() < 2 {
        return Vec::new();
    }
    let parts: Vec<&str> = lines[1].split_whitespace().collect();
    if parts.len() < 5 {
        return Vec::new();
    }
    let total: u64 = parts[1].parse().unwrap_or(0) * 1024;
    let used: u64 = parts[2].parse().unwrap_or(0) * 1024;
    let free: u64 = parts[3].parse().unwrap_or(0) * 1024;
    let usage_str = parts[4].trim_end_matches('%');
    let usage_percent: f64 = usage_str.parse().unwrap_or(0.0);
    vec![DiskInfo {
        mount_point: mount.to_string(),
        total,
        used,
        free,
        usage_percent,
    }]
}

// ─── Scan Directory ───

#[tauri::command(rename_all = "camelCase")]
pub fn scan_directory(path: String) -> Result<Vec<DirEntry>, String> {
    let dir = PathBuf::from(&path);
    if !dir.exists() {
        return Err(format!("路径不存在: {}", path));
    }
    if !dir.is_dir() {
        return Err(format!("不是目录: {}", path));
    }

    let entries = fs::read_dir(&dir).map_err(|e| format!("无法读取目录: {}", e))?;
    let mut results = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let entry_path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        // Skip symlinks to avoid infinite loops
        if entry_path.is_symlink() {
            continue;
        }
        let metadata = match entry_path.symlink_metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let file_type = if metadata.is_dir() { "directory" } else { "file" };
        let size = if metadata.is_dir() {
            calculate_dir_size(&entry_path).unwrap_or(0)
        } else {
            metadata.len()
        };
        let modified = metadata.modified().ok().map(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        });
        let children_count = if metadata.is_dir() {
            count_children(&entry_path).ok()
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

    results.sort_by(|a, b| b.size.cmp(&a.size));
    Ok(results)
}

// ─── Scan by Category ───

#[tauri::command(rename_all = "camelCase")]
pub fn scan_by_category(path: String, limit: u32) -> Result<Vec<FileCategory>, String> {
    let dir = PathBuf::from(&path);
    if !dir.exists() {
        return Err(format!("路径不存在: {}", path));
    }

    let mut file_map: HashMap<String, (String, String, u32, u64, Vec<DirEntry>)> = HashMap::new();
    walk_for_categories(&dir, &mut file_map, limit);

    let categories: Vec<FileCategory> = file_map
        .into_iter()
        .map(|(ext, (icon, label, count, total_size, files))| FileCategory {
            extension: ext,
            icon,
            label,
            count,
            total_size,
            files,
        })
        .collect();

    let mut categories = categories;
    categories.sort_by(|a, b| b.total_size.cmp(&a.total_size));
    Ok(categories)
}

fn walk_for_categories(
    dir: &Path,
    map: &mut HashMap<String, (String, String, u32, u64, Vec<DirEntry>)>,
    limit: u32,
) {
    let mut count: u32 = 0;
    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        if count >= limit * 100 {
            return;
        }
        if let Ok(entries) = fs::read_dir(&current) {
            for entry in entries {
                if count >= limit * 100 {
                    return;
                }
                if let Ok(entry) = entry {
                    let path = entry.path();
                    // Skip symlinks
                    if path.is_symlink() {
                        continue;
                    }
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_dir() {
                            stack.push(path);
                        } else {
                            count += 1;
                            let ext = path
                                .extension()
                                .map(|e| e.to_string_lossy().to_lowercase())
                                .unwrap_or_else(|| "unknown".to_string());

                            let (icon, label) = get_category_info(&ext);
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

                            let entry = map.entry(ext.clone()).or_insert_with(|| {
                                (icon.to_string(), label.to_string(), 0, 0, Vec::new())
                            });
                            entry.0 = icon.to_string();
                            entry.1 = label.to_string();
                            entry.2 += 1;
                            entry.3 += metadata.len();
                            entry.4.push(entry_info);
                        }
                    }
                }
            }
        }
    }

    for (_, (_, _, _, _, files)) in map.iter_mut() {
        files.sort_by(|a, b| b.size.cmp(&a.size));
        files.truncate(limit as usize);
    }
}

fn get_category_info(ext: &str) -> (&'static str, String) {
    match ext {
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" => ("📦", "压缩包".to_string()),
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" => ("🎬", "视频".to_string()),
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" => ("🎵", "音频".to_string()),
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" | "ico" => ("🖼️", "图片".to_string()),
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" => ("📄", "文档".to_string()),
        "dmg" | "iso" | "img" => ("💿", "磁盘镜像".to_string()),
        "apk" | "ipa" => ("📱", "安装包".to_string()),
        "exe" | "msi" | "app" => ("⚙️", "可执行文件".to_string()),
        "js" | "ts" | "py" | "go" | "rs" | "java" | "cpp" | "c" | "h" => ("💻", "源代码".to_string()),
        "log" => ("📋", "日志".to_string()),
        "tmp" | "temp" | "cache" => ("🗑️", "临时文件".to_string()),
        "woff" | "woff2" | "ttf" | "otf" | "eot" => ("🔤", "字体".to_string()),
        _ => ("📁", format!(".{}", ext)),
    }
}

// ─── Cache Paths ───

#[tauri::command(rename_all = "camelCase")]
pub fn get_cache_paths() -> Vec<CachePath> {
    let mut caches = Vec::new();

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            let cache_paths: &[(PathBuf, &str, &str, bool)] = &[
                (home.join("Library/Caches"), "系统缓存", "macOS 应用程序缓存", true),
                (home.join("Library/Caches/com.apple.Safari"), "Safari 缓存", "浏览器缓存数据", true),
                (home.join("Library/Caches/com.google.Chrome"), "Chrome 缓存", "浏览器缓存数据", true),
                (home.join("Library/Caches/CloudKit"), "iCloud 缓存", "iCloud 同步缓存", true),
                (home.join("Library/Developer/Xcode/DerivedData"), "Xcode 构建缓存", "Xcode 编译产物", true),
                (home.join("Library/Developer/Xcode/iOS DeviceSupport"), "Xcode 设备支持", "iOS 设备符号表", true),
                (home.join("Library/Caches/Homebrew"), "Homebrew 缓存", "包管理器下载缓存", true),
                (home.join("Library/Caches/pip"), "pip 缓存", "Python 包缓存", true),
                (home.join("Library/Caches/CocoaPods"), "CocoaPods 缓存", "iOS 依赖缓存", true),
                (home.join("Library/Logs"), "系统日志", "应用日志文件", true),
                (home.join("Library/Caches/com.microsoft.VSCode"), "VS Code 缓存", "编辑器缓存", true),
                (home.join("Library/Caches/WebKit"), "WebKit 缓存", "Web 渲染引擎缓存", true),
                (home.join(".npm"), "npm 缓存", "Node.js 包缓存", true),
                (home.join(".cache"), "通用缓存", "跨平台缓存目录", true),
            ];

            for (rel_path, name, desc, safe) in cache_paths {
                let size = calculate_dir_size(cache_path).unwrap_or(0);
                if size > 0 {
                    caches.push(CachePath {
                        path: path_str,
                        name: name.to_string(),
                        description: desc.to_string(),
                        size,
                        safe_to_clean: *safe,
                    });
                }
            }
        }
        // System caches
        let sys_caches: &[(&str, &str, &str, bool)] = &[
            ("/private/var/folders", "系统临时文件", "macOS 临时文件目录", false),
        ];
        for (path, name, desc, safe) in sys_caches {
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
            let cache_paths: &[(PathBuf, &str, &str, bool)] = &[
                (home.join(".cache"), "用户缓存", "XDG 缓存目录", true),
                (home.join(".cache/thumbnails"), "缩略图缓存", "文件管理器缩略图", true),
                (home.join(".npm"), "npm 缓存", "Node.js 包缓存", true),
                (home.join(".cache/pip"), "pip 缓存", "Python 包缓存", true),
                (home.join(".local/share/Trash"), "回收站", "已删除文件", true),
                (home.join(".mozilla/firefox"), "Firefox 缓存", "浏览器缓存", true),
                (home.join(".config/google-chrome"), "Chrome 缓存", "浏览器缓存", true),
            ];

            for (cache_path, name, desc, safe) in cache_paths {
                let path_str = cache_path.to_string_lossy().to_string();
                let size = calculate_dir_size(cache_path).unwrap_or(0);
                if size > 0 {
                    caches.push(CachePath {
                        path: path_str,
                        name: name.to_string(),
                        description: desc.to_string(),
                        size,
                        safe_to_clean: *safe,
                    });
                }
            }
        }

        let sys_caches: &[(&str, &str, &str, bool)] = &[
            ("/var/cache/apt", "APT 缓存", "Debian/Ubuntu 包缓存", true),
            ("/var/log", "系统日志", "日志文件", false),
            ("/tmp", "临时文件", "系统临时目录", false),
        ];
        for (path, name, desc, safe) in sys_caches {
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
            let cache_paths: &[(PathBuf, &str, &str, bool)] = &[
                (home.join("AppData\\Local\\Temp"), "临时文件", "Windows 临时文件", true),
                (home.join("AppData\\Local\\Microsoft\\Windows\\INetCache"), "IE 缓存", "Internet Explorer 缓存", true),
                (home.join("AppData\\Local\\Google\\Chrome\\User Data\\Default\\Cache"), "Chrome 缓存", "浏览器缓存", true),
                (home.join("AppData\\Roaming\\Code\\Cache"), "VS Code 缓存", "编辑器缓存", true),
                (home.join(".nuget\\packages"), "NuGet 缓存", ".NET 包缓存", true),
                (home.join(".cargo\\registry"), "Cargo 缓存", "Rust 包缓存", true),
                (home.join("AppData\\Local\\npm-cache"), "npm 缓存", "Node.js 包缓存", true),
                (PathBuf::from("C:\\Windows\\Temp"), "系统临时文件", "Windows 系统临时目录", false),
                (PathBuf::from("C:\\Windows\\Prefetch"), "预取缓存", "程序启动加速缓存", true),
            ];

            for (cache_path, name, desc, safe) in cache_paths {
                let path_str = cache_path.to_string_lossy().to_string();
                let size = calculate_dir_size(cache_path).unwrap_or(0);
                if size > 0 {
                    caches.push(CachePath {
                        path: path_str,
                        name: name.to_string(),
                        description: desc.to_string(),
                        size,
                        safe_to_clean: *safe,
                    });
                }
            }
        }
    }

    caches.sort_by(|a, b| b.size.cmp(&a.size));
    caches
}

// ─── Delete Items ───

/// Protected system paths that cannot be deleted
fn is_protected_path(path: &Path) -> bool {
    // Canonicalize to resolve relative paths like ../../etc
    let resolved = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let path_str = resolved.to_string_lossy();

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        // Protect root-level system directories, but NOT user-writable subdirs
        let protected = [
            "/usr/bin", "/usr/sbin", "/usr/lib", "/usr/libexec",
            "/etc", "/bin", "/sbin", "/lib",
            "/System", "/Library", "/Applications",
            "/var/log", "/var/db",
        ];
        for p in &protected {
            if path_str == *p || path_str.starts_with(&format!("{}/", p)) {
                return true;
            }
        }
        // Protect root / only if it's exactly /
        if path_str == "/" {
            return true;
        }
    }

    #[cfg(target_os = "windows")]
    {
        let protected = [
            "C:\\Windows", "C:\\Program Files", "C:\\Program Files (x86)",
            "C:\\ProgramData", "C:\\Users\\Default",
        ];
        for p in &protected {
            let lower = path_str.to_lowercase();
            if lower == p.to_lowercase() || lower.starts_with(&format!("{}\\", p.to_lowercase())) {
                return true;
            }
        }
    }

    false
}

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
        // Safety: prevent deletion of system paths
        if is_protected_path(&path) {
            failed.push((path_str.clone(), "系统保护路径，禁止删除".to_string()));
            continue;
        }

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

// ─── Find Duplicates ───

#[tauri::command(rename_all = "camelCase")]
pub fn find_duplicates(path: String, min_size: u64) -> Result<Vec<DuplicateGroup>, String> {
    let dir = PathBuf::from(&path);
    if !dir.exists() {
        return Err(format!("路径不存在: {}", path));
    }

    let mut file_map: HashMap<String, Vec<DirEntry>> = HashMap::new();
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

    let mut groups = groups;
    groups.sort_by(|a, b| b.wasted_space.cmp(&a.wasted_space));
    Ok(groups)
}

// ─── Helpers ───

fn calculate_dir_size(dir: &Path) -> io::Result<u64> {
    let mut total = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                // Skip symlinks to avoid infinite loops
                if path.is_symlink() {
                    continue;
                }
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
            if let Ok(_) = entry {
                count += 1;
            }
        }
    }
    Ok(count)
}

fn walk_for_duplicates(
    dir: &Path,
    map: &mut HashMap<String, Vec<DirEntry>>,
    min_size: u64,
) {
    let mut stack = vec![dir.to_path_buf()];
    let mut processed = 0;

    while let Some(current) = stack.pop() {
        if processed >= 50000 {
            return;
        }
        if let Ok(entries) = fs::read_dir(&current) {
            for entry in entries {
                if let Ok(entry) = entry {
                    processed += 1;
                    let path = entry.path();
                    // Skip symlinks
                    if path.is_symlink() {
                        continue;
                    }
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
