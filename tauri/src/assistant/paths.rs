//! AI 助手 —— 受限的本地路径检索（只为填 CICD 路径服务）
//!
//! 与「文件操作工具」的区别，这条线必须守住：
//! - **只给元信息**：存在性、类型、大小、修改时间、构建标志文件、单层子目录名。
//!   本模块没有任何读文件内容的入口；内容读取仍然只有部署日志白名单那一条路
//!   （`safety::read_text_file_in`）。
//! - **枚举有边界**：目录遍历只允许发生在「搜索根」之内（用户主目录 + 设置里的
//!   Git 扫描目录 + 应用数据目录），且有深度/访问数/结果数三重上限，不跟随软链接。
//! - **凭据目录绝对排除**：`.ssh`/`.gnupg`/`.aws`/钥匙串/浏览器配置等目录既不可枚举、
//!   也不可 stat、也不会作为搜索结果出现——否则助手就成了一个凭据探测器。

use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// 绝对禁止触碰的相对/绝对位置（大小写不敏感按名字匹配）
const DENIED_DIR_NAMES: &[&str] = &[
    ".ssh",
    ".gnupg",
    ".aws",
    ".azure",
    ".gcd",
    ".config",
    ".docker",
    ".netrc",
    "keychains",
    "cookies",
    ".pwd",
];
const DENIED_PATH_PARTS: &[&str] = &[
    "library/keychains",
    "library/application support/google/chrome",
    "library/application support/firefox",
    "library/containers",
    ".supertool/",
];

/// 遍历时直接剪掉的噪音目录（不是安全问题，是结果质量问题）
const PRUNED_DIR_NAMES: &[&str] = &[
    "node_modules",
    ".git",
    ".svn",
    ".hg",
    "target",
    "dist",
    "build",
    ".next",
    ".nuxt",
    ".turbo",
    ".gradle",
    ".cache",
    ".trash",
    ".npm",
    ".pnpm",
    ".yarn",
    ".venv",
    "__pycache__",
    ".idea",
    ".vscode",
    "library",
    "system",
    "windows",
    "program files",
    "applications",
];

/// CICD 配路径真正关心的标志文件：有这些才知道能不能在这构建、产物大概在哪
const BUILD_MARKERS: &[&str] = &[
    "pom.xml",
    "package.json",
    "build.gradle",
    "build.gradle.kts",
    "settings.gradle",
    "Cargo.toml",
    "go.mod",
    "composer.json",
    "requirements.txt",
    "pyproject.toml",
    "Makefile",
    "mvnw",
    "gradlew",
];
/// 常见产物/源码目录，用来判断「产物目录该怎么填」
const LAYOUT_HINTS: &[&str] = &["src", "target", "dist", "build", "web", "h5", "app"];

pub struct WalkLimits {
    pub max_depth: usize,
    pub max_dirs_visited: usize,
    pub max_results: usize,
}

impl Default for WalkLimits {
    fn default() -> Self {
        Self {
            max_depth: 4,
            max_dirs_visited: 4_000,
            max_results: 25,
        }
    }
}

fn dir_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

fn lower_path(path: &Path) -> String {
    path.to_string_lossy().to_lowercase()
}

/// 凭据/系统位置判定：拒绝枚举、也拒绝 stat
pub fn is_denied(path: &Path) -> bool {
    let lp = lower_path(path);
    if DENIED_PATH_PARTS.iter().any(|p| lp.contains(p)) {
        return true;
    }
    path.ancestors().any(|a| {
        let name = dir_name(a);
        !name.is_empty() && DENIED_DIR_NAMES.contains(&name.as_str())
    })
}

fn is_pruned(name: &str) -> bool {
    let n = name.to_lowercase();
    PRUNED_DIR_NAMES.iter().any(|p| *p == n)
}

fn is_hidden(path: &Path) -> bool {
    dir_name(path).starts_with('.')
}

pub fn expand_home(raw: &str) -> PathBuf {
    let trimmed = raw.trim();
    if let Some(rest) = trimmed.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
            return Path::new(&home).join(rest);
        }
    }
    PathBuf::from(trimmed)
}

/// 目录条目摘要（不含任何文件内容）
fn entry_brief(path: &Path) -> Option<Value> {
    let meta = std::fs::symlink_metadata(path).ok()?;
    Some(json!({
        "path": path.to_string_lossy(),
        "name": path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
        "isDir": meta.is_dir(),
        "isFile": meta.is_file(),
        "isSymlink": meta.file_type().is_symlink(),
        "sizeBytes": meta.len(),
        "modified": meta.modified().ok().map(|t| {
            chrono::DateTime::<chrono::Local>::from(t)
                .format("%Y-%m-%d %H:%M")
                .to_string()
        }),
    }))
}

/// 某个目录里有哪些构建标志文件与布局线索（只看存在性）
pub fn dir_signals(path: &Path) -> Value {
    let present: Vec<&str> = BUILD_MARKERS
        .iter()
        .filter(|f| path.join(f).exists())
        .copied()
        .collect();
    let layout: Vec<&str> = LAYOUT_HINTS
        .iter()
        .filter(|d| path.join(d).is_dir())
        .copied()
        .collect();
    json!({
        "buildMarkers": present,
        "layoutDirs": layout,
        "isGitRepo": path.join(".git").exists(),
    })
}

/// 单层子目录 + 标志文件（不递归、不读内容）
pub fn shallow_listing(path: &Path, include_hidden: bool, cap: usize) -> (Vec<String>, usize) {
    let Ok(read) = std::fs::read_dir(path) else {
        return (Vec::new(), 0);
    };
    let mut names = Vec::new();
    let mut total = 0usize;
    for entry in read.flatten() {
        let p = entry.path();
        let Ok(meta) = std::fs::symlink_metadata(&p) else {
            continue;
        };
        if !include_hidden && is_hidden(&p) {
            continue;
        }
        total += 1;
        if meta.is_dir() {
            names.push(format!("{}/", entry.file_name().to_string_lossy()));
        } else {
            names.push(entry.file_name().to_string_lossy().to_string());
        }
        if names.len() >= cap {
            break;
        }
    }
    names.sort();
    (names, total)
}

/// 在给定根目录内按名称模糊查找路径（深度/访问量/结果数三重上限，不跟随软链接）
pub fn find_paths(
    roots: &[PathBuf],
    query: &str,
    dirs_only: bool,
    include_hidden: bool,
    limits: &WalkLimits,
) -> (Vec<Value>, bool) {
    let needle = query.trim().to_lowercase();
    let mut out = Vec::new();
    let mut visited = 0usize;
    let mut truncated = false;

    let mut stack: Vec<(PathBuf, usize)> = roots
        .iter()
        .filter(|r| !is_denied(r))
        .map(|r| (r.clone(), 0usize))
        .collect();

    while let Some((dir, depth)) = stack.pop() {
        if out.len() >= limits.max_results {
            truncated = true;
            break;
        }
        if depth > limits.max_depth || visited >= limits.max_dirs_visited {
            truncated = true;
            continue;
        }
        let Ok(read) = std::fs::read_dir(&dir) else {
            continue;
        };
        visited += 1;
        for entry in read.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let Ok(meta) = std::fs::symlink_metadata(&path) else {
                continue;
            };
            if !include_hidden && name.starts_with('.') {
                continue;
            }
            if is_denied(&path) {
                continue;
            }
            if meta.is_dir() {
                if !is_pruned(&name) && depth < limits.max_depth {
                    stack.push((path.clone(), depth + 1));
                }
            } else if !meta.is_file() {
                // 软链接/设备文件一律不跟随
                continue;
            }
            if name.to_lowercase().contains(&needle)
                && (!dirs_only || meta.is_dir())
                && !(dirs_only && !meta.is_dir())
            {
                if let Some(brief) = entry_brief(&path) {
                    out.push(brief);
                    if out.len() >= limits.max_results {
                        truncated = true;
                        break;
                    }
                }
            }
        }
    }
    (out, truncated)
}

/// stat 一个具体路径：存在性 + 构建标志 + 单层内容清单（不读内容）
pub fn inspect_path(raw: &str, include_hidden: bool) -> Value {
    let path = expand_home(raw);
    if is_denied(&path) {
        return json!({
            "error": "该位置属于凭据/系统敏感目录，助手不访问",
            "path": raw,
        });
    }
    match std::fs::symlink_metadata(&path) {
        Err(e) => json!({
            "path": path.to_string_lossy(),
            "exists": false,
            "reason": e.to_string(),
        }),
        Ok(meta) => {
            let mut out = match entry_brief(&path) {
                Some(v) => v,
                None => json!({"path": path.to_string_lossy()}),
            };
            out["exists"] = json!(true);
            if meta.is_dir() {
                let mut signals = dir_signals(&path);
                let (children, total) = shallow_listing(&path, include_hidden, 40);
                if let Some(obj) = signals.as_object_mut() {
                    obj.insert("children".to_string(), json!(children));
                    obj.insert("childCount".to_string(), json!(total));
                }
                out["signals"] = signals;
            }
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sandbox(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "st_paths_{}_{}_{}",
            std::process::id(),
            tag,
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// 造一棵接近真实仓库的目录树
    fn fixture(tag: &str) -> PathBuf {
        let root = sandbox(tag);
        let repo = root.join("repo/SRC/mall/seller-api");
        std::fs::create_dir_all(repo.join("src/main/java")).unwrap();
        std::fs::write(repo.join("pom.xml"), b"<project/>").unwrap();
        std::fs::create_dir_all(repo.join("target")).unwrap();
        let front = root.join("repo/SRC/front/mall-h5");
        std::fs::create_dir_all(front.join("dist/build/h5")).unwrap();
        std::fs::write(front.join("package.json"), b"{\"scripts\":{\"build:h5\":\"x\"}}").unwrap();
        std::fs::create_dir_all(front.join("node_modules/left-pad")).unwrap();
        std::fs::create_dir_all(root.join("repo/.git/objects")).unwrap();
        std::fs::create_dir_all(root.join(".ssh")).unwrap();
        std::fs::write(root.join(".ssh/id_rsa"), b"PRIVKEY-DO-NOT-READ").unwrap();
        std::fs::write(root.join(".secret"), b"topsecret").unwrap();
        root
    }

    #[test]
    fn finds_project_directories_by_name() {
        let root = fixture("find");
        let (hits, truncated) = find_paths(
            &[root.clone()],
            "seller",
            true,
            false,
            &WalkLimits::default(),
        );
        assert!(!hits.is_empty(), "应找到 seller-api 目录");
        let first = hits[0].as_object().unwrap();
        assert_eq!(first["isDir"], json!(true));
        assert!(first["path"]
            .as_str()
            .unwrap()
            .ends_with("SRC/mall/seller-api"));
        assert!(!truncated);
        std::fs::remove_dir_all(&root).ok();
    }

    /// 噪音目录必须剪掉，否则结果全被 node_modules/.git 淹没
    #[test]
    fn prunes_noisy_directories() {
        let root = fixture("prune");
        let (hits, _) = find_paths(&[root.clone()], "left", true, false, &WalkLimits::default());
        assert!(hits.is_empty(), "node_modules 内的条目不应出现在结果里");
        let (gits, _) = find_paths(&[root.clone()], "objects", true, false, &WalkLimits::default());
        assert!(gits.is_empty(), ".git 内不应被枚举");
        std::fs::remove_dir_all(&root).ok();
    }

    /// 红线：凭据目录既不能枚举也不能 stat，且不跟随软链接出去
    #[test]
    fn credential_locations_are_off_limits() {
        let root = fixture("cred");
        let (hits, _) = find_paths(
            &[root.clone()],
            "id_rsa",
            false,
            true,
            &WalkLimits {
                max_depth: 6,
                ..Default::default()
            },
        );
        assert!(hits.is_empty(), "私密钥匙不得被找到: {hits:?}");

        let probe = inspect_path(root.join(".ssh/id_rsa").to_str().unwrap(), false);
        assert!(probe.get("error").is_some(), "应拒绝 stat 凭据路径");
        assert!(!probe.to_string().contains("PRIVKEY"));

        let out = inspect_path(root.join(".secret").to_str().unwrap(), true);
        assert_eq!(out["exists"], json!(true), "普通文件允许 stat");
        assert!(
            !out.to_string().contains("topsecret"),
            "绝不能带出文件内容: {out}"
        );
        assert_eq!(out["isFile"], json!(true));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn inspect_reports_build_markers_and_children_not_contents() {
        let root = fixture("inspect");
        let backend = inspect_path(root.join("repo/SRC/mall/seller-api").to_str().unwrap(), false);
        assert_eq!(backend["exists"], json!(true));
        assert!(backend["signals"]["buildMarkers"]
            .as_array()
            .unwrap()
            .contains(&json!("pom.xml")));
        assert!(backend["signals"]["layoutDirs"]
            .as_array()
            .unwrap()
            .contains(&json!("target")));
        assert!(backend["signals"]["children"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c.as_str().unwrap() == "src/"));
        assert!(!backend.to_string().contains("<project/>"), "不得读文件内容");

        let front = inspect_path(root.join("repo/SRC/front/mall-h5").to_str().unwrap(), false);
        assert!(front["signals"]["buildMarkers"]
            .as_array()
            .unwrap()
            .contains(&json!("package.json")));

        let missing = inspect_path(root.join("repo/nope").to_str().unwrap(), false);
        assert_eq!(missing["exists"], json!(false));
        std::fs::remove_dir_all(&root).ok();
    }

    /// 结果数与访问量必须真生效，否则一次搜索能把 CPU 打满
    #[test]
    fn caps_are_enforced() {
        let root = sandbox("caps");
        for i in 0..60 {
            std::fs::create_dir_all(root.join(format!("pkg-{i}"))).unwrap();
        }
        let (hits, truncated) = find_paths(
            &[root.clone()],
            "pkg-",
            true,
            false,
            &WalkLimits {
                max_results: 10,
                ..Default::default()
            },
        );
        assert_eq!(hits.len(), 10);
        assert!(truncated, "被截断时必须告知调用方");

        let (deep, truncated) = find_paths(
            &[root.clone()],
            "pkg-",
            true,
            false,
            &WalkLimits {
                max_depth: 0,
                max_dirs_visited: 1,
                max_results: 5,
            },
        );
        assert!(truncated || deep.is_empty());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn home_expansion_and_denied_prefixes() {
        assert!(expand_home("~/x").is_absolute());
        assert_eq!(expand_home("/abs/path"), PathBuf::from("/abs/path"));
        assert!(is_denied(Path::new("/Users/x/.ssh")));
        assert!(is_denied(Path::new("/Users/x/.ssh/config")));
        assert!(is_denied(Path::new("/Users/x/Library/Keychains/login.keychain-db")));
        assert!(is_denied(Path::new("/Users/x/.supertool/supertool.db")));
        assert!(!is_denied(Path::new("/Users/x/IdeaProjects/app")));
    }
}
