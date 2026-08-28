//! AI 助手 —— 受限的源码查阅（只读本项目仓库，供定位项目问题/迭代用）
//!
//! 安全边界（与 paths.rs 同一条线，只多开「读本项目源码内容」这一个口）：
//! - **只读**：没有任何写入口；返回内容仍会被 agent 层 `safety::deep_redact` 脱敏。
//! - **只限本项目根**：所有路径必须 canonicalize 后落在项目根（`CARGO_MANIFEST_DIR` 的父目录）内，
//!   相对路径参数拒绝 `..` 逃逸；检索只遍历白名单子目录（tauri/src、src、core/src、cli/src、docs）。
//! - **有上限**：单文件读取上限、扫描文件数上限、结果数上限；二进制/超大文件跳过。
//! - **凭据排除**：复用 paths::is_denied，`.ssh` 等位置不可枚举也不可读。

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::paths::is_denied;

/// 单文件读取内容上限（字节）：源码片段级查看，避免一次拉进整文件
const MAX_READ_BYTES: u64 = 64 * 1024;
/// 扫描时单文件超过此大小跳过（不做全文匹配）
const MAX_SCAN_BYTES: u64 = 1024 * 1024;
/// 一次扫描最多碰的文件数（防止遍历爆炸）
const MAX_SCANNED_FILES: usize = 3_000;
/// 一次检索最多返回的命中行
const MAX_MATCHES: usize = 40;
/// 单文件内最多返回的命中行
const MAX_MATCHES_PER_FILE: usize = 6;

/// 项目根：编译期锚定 `tauri/` 的父目录（supertool/）。
/// 仅当目录存在时返回（发布到其他机器找不到源码时，工具返回不可用提示而非报错）。
pub fn project_root() -> Option<PathBuf> {
    let tauri_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = tauri_dir.parent()?;
    if root.join("AGENTS.md").exists() && root.join("src").is_dir() {
        Some(root.to_path_buf())
    } else {
        None
    }
}

/// 允许检索的子目录（相对项目根）
const SEARCH_DIRS: &[&str] = &["tauri/src", "src", "core/src", "cli/src", "docs"];
/// 根目录下允许直接检索的散文件
const SEARCH_ROOT_FILES: &[&str] = &["AGENTS.md", "Cargo.toml", "package.json"];

/// 噪音目录（检索时剪枝，只影响结果质量，非安全问题）
fn is_pruned_dir(name: &str) -> bool {
    let n = name.to_lowercase();
    n.starts_with('.') || matches!(n.as_str(), "node_modules" | "dist" | "target" | "build")
}

/// 文本文件判定：按扩展名白名单；无扩展名只放行根级明确列出的配置文件
fn is_text_file(path: &Path) -> bool {
    const TEXT_EXTS: &[&str] = &[
        "rs", "ts", "tsx", "vue", "js", "mjs", "cjs", "json", "toml", "md", "html", "css",
        "scss", "yml", "yaml", "sql", "sh", "svg", "xml", "lock", "conf", "cfg", "ini",
    ];
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => TEXT_EXTS.contains(&ext.to_lowercase().as_str()),
        None => {
            let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            SEARCH_ROOT_FILES.contains(&name.as_str())
        }
    }
}

/// 校验相对路径安全（拒绝绝对路径、`..`、`.`、空段），并返回项目根内的绝对路径。
/// 文件存在时 canonicalize 后双重确认前缀在项目根内（拦截软链接逃逸）。
pub fn resolve_within_root(rel: &str, root: &Path) -> Result<PathBuf, String> {
    if rel.is_empty() || rel.contains('\0') {
        return Err("路径不能为空或含空字符".to_string());
    }
    if rel.starts_with('/') {
        return Err("路径必须是以项目根为基准的相对路径".to_string());
    }
    let segments_ok = rel
        .split('/')
        .all(|seg| !seg.is_empty() && seg != "." && seg != "..");
    if !segments_ok {
        return Err("路径不能包含 .. 等跳转段".to_string());
    }
    let candidate = root.join(rel);
    if candidate.exists() {
        let canon = candidate
            .canonicalize()
            .map_err(|e| format!("无法解析路径: {e}"))?;
        let root_canon = root
            .canonicalize()
            .map_err(|e| format!("无法解析项目根: {e}"))?;
        if !canon.starts_with(&root_canon) {
            return Err("路径越过了项目根目录".to_string());
        }
        return Ok(canon);
    }
    Ok(candidate)
}

/// 在给定根内按关键词检索源码（核心逻辑，root 可注入以便测试）
fn search_in_root(root: &Path, needle: &str, cap: usize) -> (Vec<Value>, usize) {
    let mut hits: Vec<Value> = Vec::new();
    let mut scanned = 0usize;

    for dir_rel in SEARCH_DIRS {
        let dir = root.join(dir_rel);
        if !dir.is_dir() {
            continue;
        }
        let mut stack = vec![dir];
        while let Some(cur) = stack.pop() {
            if scanned >= MAX_SCANNED_FILES || hits.len() >= cap {
                return (hits, scanned);
            }
            let Ok(read) = std::fs::read_dir(&cur) else { continue };
            for entry in read.flatten() {
                if scanned >= MAX_SCANNED_FILES || hits.len() >= cap {
                    return (hits, scanned);
                }
                let path = entry.path();
                if is_denied(&path) {
                    continue;
                }
                let Ok(meta) = std::fs::symlink_metadata(&path) else { continue };
                if meta.is_dir() {
                    if !is_pruned_dir(&entry.file_name().to_string_lossy()) {
                        stack.push(path);
                    }
                    continue;
                }
                if !meta.is_file() || meta.len() > MAX_SCAN_BYTES || !is_text_file(&path) {
                    continue;
                }
                scanned += 1;
                let Ok(content) = std::fs::read_to_string(&path) else { continue };
                let rel = path.strip_prefix(root).unwrap_or(&path).to_string_lossy().to_string();
                let mut per_file = 0usize;
                for (idx, line) in content.lines().enumerate() {
                    if !line.to_lowercase().contains(needle) {
                        continue;
                    }
                    hits.push(json!({ "path": rel, "line": idx + 1, "text": trim_line(line) }));
                    per_file += 1;
                    if per_file >= MAX_MATCHES_PER_FILE || hits.len() >= cap {
                        break;
                    }
                }
            }
        }
    }

    // 根级散文件
    for name in SEARCH_ROOT_FILES {
        if hits.len() >= cap {
            break;
        }
        let path = root.join(name);
        if !path.is_file() {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else { continue };
        for (idx, line) in content.lines().enumerate() {
            if !line.to_lowercase().contains(needle) {
                continue;
            }
            hits.push(json!({ "path": name.to_string(), "line": idx + 1, "text": trim_line(line) }));
            if hits.len() >= cap {
                break;
            }
        }
    }
    (hits, scanned)
}

/// 在项目根内按关键词检索源码（返回 文件:行号 + 片段）
pub fn search_project_source(query: &str, limit: usize) -> Result<Value, String> {
    let Some(root) = project_root() else {
        return Ok(json!({
            "error": "找不到项目源码目录（本项目仓库不存在于当前构建路径），只能查阅内嵌文档",
        }));
    };
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return Err("缺少检索关键词".to_string());
    }
    let cap = limit.clamp(1, MAX_MATCHES);
    let (hits, scanned) = search_in_root(&root, &needle, cap);
    Ok(json!({
        "hits": hits,
        "truncated": hits.len() >= cap,
        "scannedFiles": scanned,
        "note": if hits.is_empty() {
            "项目源码里没有直接命中。可换关键词（标识符/函数名/字段名/报错字符串）再查，或先 search_project_guides 查文档定位。"
        } else {
            "命中按文件顺序给出；需要看完整上下文用 read_project_source 取文件。"
        },
    }))
}

/// 读取项目根内的单个文件（相对路径，限大小；核心逻辑，root 可注入以便测试）
fn read_in_root(root: &Path, rel: &str) -> Result<Value, String> {
    let path = resolve_within_root(rel, root)?;
    if is_denied(&path) {
        return Err("该路径属于凭据/系统敏感位置，不允许读取".to_string());
    }
    let meta = std::fs::symlink_metadata(&path)
        .map_err(|e| format!("无法访问 {rel}: {e}"))?;
    if !meta.is_file() {
        return Err(format!("{rel} 不是普通文件"));
    }
    if meta.len() > MAX_READ_BYTES {
        return Err(format!(
            "{rel} 超过单次读取上限（{}KB），可用 search_project_source 定位到具体行",
            MAX_READ_BYTES / 1024
        ));
    }
    if !is_text_file(&path) {
        return Err(format!("{rel} 不是文本源码文件"));
    }
    let content = std::fs::read_to_string(&path).map_err(|e| format!("读取失败: {e}"))?;
    let rel_final = path.strip_prefix(root).unwrap_or(&path).to_string_lossy().to_string();
    Ok(json!({
        "path": rel_final,
        "sizeBytes": meta.len(),
        "lines": content.lines().count(),
        "content": content,
    }))
}

/// 读取项目根内单个文件（对外入口）
pub fn read_project_source(rel: &str) -> Result<Value, String> {
    let Some(root) = project_root() else {
        return Ok(json!({ "error": "找不到项目源码目录，无法读取文件" }));
    };
    read_in_root(&root, rel)
}

/// 单行片段裁剪：控制上下文占用
fn trim_line(line: &str) -> String {
    let text = line.trim();
    if text.chars().count() > 160 {
        let mut s: String = text.chars().take(160).collect();
        s.push('…');
        s
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sandbox(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "st_src_{}_{}_{}",
            std::process::id(),
            tag,
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn make_fixture(root: &Path) {
        std::fs::create_dir_all(root.join("tauri/src/assistant")).unwrap();
        std::fs::write(
            root.join("tauri/src/assistant/demo.rs"),
            "pub fn handle_deploy() {}\npub fn settle() {}\n// TODO: fix deploy\n",
        )
        .unwrap();
        std::fs::create_dir_all(root.join("src/views")).unwrap();
        std::fs::write(
            root.join("src/views/App.vue"),
            "<template><div>deploy button</div></template>\n",
        )
        .unwrap();
        // 噪音目录
        std::fs::create_dir_all(root.join("tauri/src/assistant/node_modules/pkg")).unwrap();
        std::fs::write(root.join("tauri/src/assistant/node_modules/pkg/index.js"), "deploy").unwrap();
        std::fs::create_dir_all(root.join("tauri/src/assistant/target")).unwrap();
        std::fs::write(root.join("tauri/src/assistant/target/x.rs"), "deploy").unwrap();
        // 凭据目录
        std::fs::create_dir_all(root.join(".ssh")).unwrap();
        std::fs::write(root.join(".ssh/id_rsa"), "deploy PRIVATEKEY").unwrap();
    }

    #[test]
    fn resolve_rejects_escape_and_allows_inside() {
        let root = sandbox("resolve");
        std::fs::create_dir_all(root.join("sub")).unwrap();
        std::fs::write(root.join("sub/a.rs"), b"fn main() {}").unwrap();

        assert!(resolve_within_root("../secret", &root).is_err(), ".. 逃逸应被拒");
        assert!(resolve_within_root("/etc/passwd", &root).is_err(), "绝对路径应被拒");
        assert!(resolve_within_root("./a", &root).is_err(), ". 段应被拒");
        assert!(resolve_within_root("sub/a.rs", &root).is_ok(), "根内文件应放行");
        std::fs::remove_dir_all(&root).ok();
    }

    #[cfg(unix)]
    #[test]
    fn resolve_canonical_check_blocks_symlink_escape() {
        let root = sandbox("symlink");
        std::fs::create_dir_all(root.join("in")).unwrap();
        let outside = sandbox("outside");
        std::fs::write(outside.join("secret.txt"), b"TOP-SECRET").unwrap();
        std::os::unix::fs::symlink(outside.join("secret.txt"), root.join("in/link.txt")).unwrap();

        let r = resolve_within_root("in/link.txt", &root);
        // 软链接指向根外 → canonicalize 前缀校验必须拦下
        assert!(r.is_err(), "软链接逃逸应被拒: {r:?}");
        std::fs::remove_dir_all(&root).ok();
        std::fs::remove_dir_all(&outside).ok();
    }

    #[test]
    fn search_finds_lines_but_skips_noise_and_credentials() {
        let root = sandbox("search");
        make_fixture(&root);

        let (hits, _) = search_in_root(&root, "deploy", 20);
        let paths: Vec<&str> = hits.iter().map(|h| h["path"].as_str().unwrap()).collect();
        assert!(
            paths.iter().any(|p| p.ends_with("demo.rs")),
            "应命中 tauri/src 内源码: {paths:?}"
        );
        assert!(paths.iter().any(|p| p.ends_with("App.vue")), "应命中 src/ 内源码: {paths:?}");
        assert!(
            !paths.iter().any(|p| p.contains("node_modules")),
            "node_modules 不得被检索: {paths:?}"
        );
        assert!(
            !paths.iter().any(|p| p.contains("/target/")),
            "target 不得被检索: {paths:?}"
        );
        assert!(
            !paths.iter().any(|p| p.contains(".ssh")),
            "凭据目录不得被检索: {paths:?}"
        );
        // 片段裁剪生效
        assert!(hits[0]["text"].as_str().is_some());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn read_in_root_respects_size_and_denied() {
        let root = sandbox("read");
        make_fixture(&root);

        let ok = read_in_root(&root, "tauri/src/assistant/demo.rs").unwrap();
        assert!(ok["content"].as_str().unwrap().contains("handle_deploy"));

        let denied = read_in_root(&root, ".ssh/id_rsa");
        assert!(denied.is_err(), "凭据文件应被拒");

        let escaped = read_in_root(&root, "../secret.txt");
        assert!(escaped.is_err(), "逃逸路径应被拒");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn caps_limit_results() {
        let root = sandbox("caps");
        std::fs::create_dir_all(root.join("src")).unwrap();
        let mut big = String::new();
        for i in 0..100 {
            big.push_str(&format!("line {i} keyword\n"));
        }
        std::fs::write(root.join("src/a.ts"), &big).unwrap();

        // 全局 cap 生效
        let (hits, _) = search_in_root(&root, "keyword", 3);
        assert_eq!(hits.len(), 3, "全局 cap 应生效");
        // 单文件上限生效：100 命中行也只取 MAX_MATCHES_PER_FILE 个
        let (hits, _) = search_in_root(&root, "keyword", 40);
        assert_eq!(hits.len(), MAX_MATCHES_PER_FILE, "单文件命中应有上限");
        std::fs::remove_dir_all(&root).ok();
    }
}
