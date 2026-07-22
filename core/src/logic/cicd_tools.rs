//! CI/CD tool detection utilities — Maven, Java, Node.js, SDK versions.
//! Extracted from tauri commands for sharing between Tauri and GPUI apps.

use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolDetectionResult {
    pub available: bool,
    pub version: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolPaths {
    #[serde(rename = "mavenHome")]
    pub maven_home: String,
    #[serde(rename = "javaHome")]
    pub java_home: String,
    #[serde(rename = "nodeHome")]
    pub node_home: String,
    #[serde(rename = "npmHome", default)]
    pub npm_home: String,
    #[serde(rename = "pnpmHome", default)]
    pub pnpm_home: String,
    #[serde(rename = "yarnHome", default)]
    pub yarn_home: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectScanResult {
    #[serde(rename = "hasPomXml")]
    pub has_pom_xml: bool,
    #[serde(rename = "hasBuildGradle")]
    pub has_build_gradle: bool,
    #[serde(rename = "hasPackageJson")]
    pub has_package_json: bool,
    #[serde(rename = "hasPomXmlAdmin")]
    pub has_pom_xml_admin: bool,
    #[serde(rename = "hasPomXmlApp")]
    pub has_pom_xml_app: bool,
    #[serde(rename = "hasDockerCompose")]
    pub has_docker_compose: bool,
    pub language: String,
    #[serde(rename = "buildTool")]
    pub build_tool: String,
    #[serde(rename = "appCount")]
    pub app_count: usize,
    #[serde(rename = "appModules")]
    pub app_modules: Vec<String>,
    #[serde(rename = "hasRobustConfig")]
    pub has_robust_config: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

pub fn run_command(cmd: &str, cwd: Option<&str>) -> CommandOutput {
    let mut parts = cmd.split_whitespace();
    let program = parts.next().unwrap_or("");
    let args: Vec<&str> = parts.collect();
    let mut command = std::process::Command::new("sh");
    let full_cmd = if args.is_empty() { program.to_string() } else { format!("{} {}", program, args.join(" ")) };
    command.arg("-c").arg(&full_cmd);
    let shell_env = crate::logic::cicd_deploy::get_shell_env_for_command();
    for (key, value) in &shell_env { command.env(key, value); }
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());
    if let Some(dir) = cwd { command.current_dir(dir); }
    match command.output() {
        Ok(output) => CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
        },
        Err(_) => CommandOutput { stdout: String::new(), stderr: String::new(), success: false },
    }
}

pub fn find_maven_home() -> Option<String> {
    let mvn_path = run_command("which mvn", None).stdout.trim().to_string();
    if mvn_path.is_empty() { return None; }
    let resolved = if let Ok(path) = std::fs::canonicalize(&mvn_path) {
        path.to_string_lossy().to_string()
    } else { mvn_path };
    if let Some(pos) = resolved.rfind("/bin/mvn") {
        if pos > 0 { return Some(resolved[..pos].to_string()); }
    }
    Some(resolved)
}

pub fn check_java(java_home: Option<String>) -> ToolDetectionResult {
    let java = if let Some(ref home) = java_home { format!("{}/bin/java", home) } else { "java".to_string() };
    let out = run_command(&format!("{} -version 2>&1", java), None);
    if out.success {
        let version = out.stderr.lines().next().map(|l| l.to_string()).or_else(|| out.stdout.lines().next().map(|l| l.to_string()));
        ToolDetectionResult { available: true, version, path: Some(java) }
    } else {
        ToolDetectionResult { available: false, version: None, path: None }
    }
}

pub fn check_maven(maven_home: Option<String>) -> ToolDetectionResult {
    let mvn = if let Some(ref home) = maven_home { format!("{}/bin/mvn", home) } else { "mvn".to_string() };
    let out = run_command(&format!("{} --version 2>&1 | head -1", mvn), None);
    if out.success {
        let version = out.stdout.lines().next().map(|l| l.trim().to_string());
        ToolDetectionResult { available: true, version, path: Some(mvn) }
    } else {
        ToolDetectionResult { available: false, version: None, path: None }
    }
}

pub fn check_node(node_home: Option<String>) -> ToolDetectionResult {
    let node = if let Some(ref home) = node_home { format!("{}/bin/node", home) } else { "node".to_string() };
    let out = run_command(&format!("{} --version 2>&1", node), None);
    if out.success {
        let version = out.stdout.trim().to_string();
        ToolDetectionResult { available: true, version: Some(version), path: Some(node) }
    } else {
        ToolDetectionResult { available: false, version: None, path: None }
    }
}

pub fn detect_tools_impl() -> HashMap<String, ToolDetectionResult> {
    use std::thread;
    // 7 个工具并行检测（彼此无依赖），替代串行 11 次 shell 调用
    let (java_t, maven_t, node_t, npm_t, pnpm_t, yarn_t, gradle_t) = thread::scope(|s| {
        let j = s.spawn(|| check_java(None));
        let m = s.spawn(|| check_maven(None));
        let n = s.spawn(|| check_node(None));
        let np = s.spawn(|| check_simple_tool("npm"));
        let pn = s.spawn(|| check_simple_tool("pnpm"));
        let ya = s.spawn(|| check_simple_tool("yarn"));
        let gr = s.spawn(|| check_simple_tool("gradle"));
        (j.join().unwrap(), m.join().unwrap(), n.join().unwrap(),
         np.join().unwrap(), pn.join().unwrap(), ya.join().unwrap(), gr.join().unwrap())
    });
    let mut tools = HashMap::new();
    tools.insert("java".to_string(), java_t);
    tools.insert("maven".to_string(), maven_t);
    tools.insert("node".to_string(), node_t);
    tools.insert("npm".to_string(), npm_t);
    tools.insert("pnpm".to_string(), pnpm_t);
    tools.insert("yarn".to_string(), yarn_t);
    tools.insert("gradle".to_string(), gradle_t);
    tools
}

/// 检测单个工具的版本和路径（用于 npm/pnpm/yarn/gradle）
fn check_simple_tool(cmd: &str) -> ToolDetectionResult {
    let out = run_command(&format!("{} --version 2>&1", cmd), None);
    let path_out = run_command(&format!("which {}", cmd), None).stdout.trim().to_string();
    if out.success {
        let version = out.stdout.lines().next().map(|s| s.trim().to_string());
        ToolDetectionResult {
            available: true,
            version,
            path: if path_out.is_empty() { None } else { Some(path_out) },
        }
    } else {
        ToolDetectionResult { available: false, version: None, path: None }
    }
}

pub fn detect_tool_paths_impl() -> ToolPaths {
    use std::thread;
    // 6 个 which 命令并行执行，替代串行
    let (mvn, java, node, npm, pnpm, yarn) = thread::scope(|s| {
        let m = s.spawn(|| strip_bin(&find_path("mvn")));
        let j = s.spawn(|| strip_bin(&find_path("java")));
        let n = s.spawn(|| strip_bin(&find_path("node")));
        let np = s.spawn(|| find_path("npm"));
        let pn = s.spawn(|| find_path("pnpm"));
        let ya = s.spawn(|| find_path("yarn"));
        (m.join().unwrap(), j.join().unwrap(), n.join().unwrap(),
         np.join().unwrap(), pn.join().unwrap(), ya.join().unwrap())
    });
    ToolPaths {
        maven_home: mvn,
        java_home: java,
        node_home: node,
        npm_home: npm,
        pnpm_home: pnpm,
        yarn_home: yarn,
    }
}

fn find_path(cmd: &str) -> String {
    run_command(&format!("which {} 2>/dev/null", cmd), None).stdout.trim().to_string()
}

fn strip_bin(bin_path: &str) -> String {
    if bin_path.is_empty() { return String::new(); }
    if let Some(pos) = bin_path.rfind("/bin/") {
        bin_path[..pos].to_string()
    } else { bin_path.to_string() }
}

pub fn scan_project_impl(local_path: &str) -> ProjectScanResult {
    let path = Path::new(local_path);
    let has_pom_xml = path.join("pom.xml").exists();
    let has_build_gradle = path.join("build.gradle").exists() || path.join("build.gradle.kts").exists();
    let has_package_json = path.join("package.json").exists();
    let has_pom_xml_admin = has_pom_xml && path.join("admin").exists();
    let has_pom_xml_app = has_pom_xml && !has_pom_xml_admin;
    let has_docker_compose = path.join("docker-compose.yml").exists();
    let language = if has_pom_xml || has_build_gradle { "java".to_string() } else if has_package_json { "node".to_string() } else { "unknown".to_string() };
    let build_tool = if has_pom_xml { "maven".to_string() } else if has_build_gradle { "gradle".to_string() } else if has_package_json { "npm".to_string() } else { "none".to_string() };

    let mut app_modules = Vec::new();
    if has_pom_xml { if let Ok(entries) = std::fs::read_dir(path) { for e in entries.flatten() { if e.path().join("pom.xml").exists() { app_modules.push(e.file_name().to_string_lossy().to_string()); } } } }

    ProjectScanResult {
        has_pom_xml, has_build_gradle, has_package_json, has_pom_xml_admin, has_pom_xml_app, has_docker_compose,
        language, build_tool, app_count: app_modules.len(), app_modules,
        has_robust_config: false,
    }
}

// =================== Project Module Tree Scanner ===================
//
// 从 tailwind-migration 分支的 tauri/src/commands/cicd.rs 下沉到 core，供 CLI/GPUI 共用。
// 用途：给定项目路径，返回 Maven <modules>/子目录 pom.xml/NPM package.json 的模块树。

/// 模块扫描时跳过的目录名（Maven/NPM 共用，保持一致）。
const SCAN_SKIP_DIRS: &[&str] = &[
    "node_modules", "target", "dist", "build", ".git", ".idea", ".vscode",
    "doc", "docs", "coverage", "test", "tests", "__pycache__", ".next", ".nuxt",
];

/// Maven `<modules>` 嵌套扫描最大深度。
const MAX_MAVEN_MODULE_DEPTH: u8 = 3;

/// `scan_subdirs_for_maven` 向下搜索 pom.xml 的最大层数。
const MAX_SUBDIR_SCAN_DEPTH: u8 = 2;

/// 扫描项目模块树。返回 `{"success": bool, "modules": [...], "error"?: "..."}`。
///
/// 优先级：
/// 1. 根目录 pom.xml 的 `<modules>` 声明（最多 3 层嵌套）
/// 2. 若根目录没有 pom.xml，向下扫描 2 层子目录里第一个 pom.xml
/// 3. 若仍无 Maven 模块，回退到子目录里的 package.json
/// 4. 再退一步：根目录 package.json 作为单模块
pub fn scan_project_modules(project_path: &str) -> serde_json::Value {
    use std::fs;
    let path = Path::new(project_path);
    if !path.exists() {
        return serde_json::json!({ "success": false, "modules": [], "error": "路径不存在" });
    }

    let mut modules = scan_maven_modules_recursive(path, path, 0);

    if modules.is_empty() {
        modules = scan_subdirs_for_maven(path, path, MAX_SUBDIR_SCAN_DEPTH);
    }

    let modules = if modules.is_empty() {
        scan_npm_modules(path)
    } else {
        modules
    };

    let modules = if modules.is_empty() && path.join("package.json").exists() {
        if let Ok(content) = fs::read_to_string(path.join("package.json")) {
            if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(name) = pkg.get("name").and_then(|n| n.as_str()) {
                    vec![serde_json::json!({
                        "name": name,
                        "path": ".",
                        "type": "npm",
                        "children": []
                    })]
                } else { vec![] }
            } else { vec![] }
        } else { vec![] }
    } else {
        modules
    };

    serde_json::json!({ "success": true, "modules": modules })
}

fn scan_subdirs_for_maven(root_path: &Path, current_path: &Path, max_depth: u8) -> Vec<serde_json::Value> {
    use std::fs;
    if max_depth == 0 { return vec![]; }
    let mut modules = Vec::new();
    if let Ok(entries) = fs::read_dir(current_path) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if !ft.is_dir() { continue; }
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') || SCAN_SKIP_DIRS.contains(&name.as_str()) { continue; }
                let sub_path = entry.path();
                if sub_path.join("pom.xml").exists() {
                    let sub_modules = scan_maven_modules_recursive(root_path, &sub_path, 0);
                    if !sub_modules.is_empty() {
                        let rel_prefix = sub_path
                            .strip_prefix(root_path)
                            .map(|p| format!("./{}", p.to_string_lossy()))
                            .unwrap_or_else(|_| format!("./{}", name));
                        let prefixed: Vec<serde_json::Value> = sub_modules.into_iter().map(|mut m| {
                            if let Some(p) = m.get("path").and_then(|v| v.as_str()) {
                                let full_path = if p == "." {
                                    rel_prefix.clone()
                                } else {
                                    format!("{}/{}", rel_prefix, p.trim_start_matches("./"))
                                };
                                m["path"] = serde_json::json!(full_path);
                            }
                            m
                        }).collect();
                        modules.extend(prefixed);
                    }
                } else {
                    let deeper = scan_subdirs_for_maven(root_path, &sub_path, max_depth - 1);
                    modules.extend(deeper);
                }
            }
        }
    }
    modules
}

fn scan_maven_modules_recursive(root_path: &Path, base_path: &Path, depth: u8) -> Vec<serde_json::Value> {
    use std::fs;
    if depth > MAX_MAVEN_MODULE_DEPTH { return vec![]; }
    let pom_path = base_path.join("pom.xml");
    if !pom_path.exists() { return vec![]; }
    let pom = match fs::read_to_string(&pom_path) {
        Ok(content) => content,
        Err(_) => return vec![],
    };

    let mut modules: Vec<serde_json::Value> = Vec::new();
    if let Ok(re) = regex::Regex::new(r"<modules>\s*([\s\S]*?)</modules>") {
        if let Some(cap) = re.captures(&pom) {
            let module_re = regex::Regex::new(r"<module>\s*([^<]+?)\s*</module>").unwrap();
            let child_names: Vec<String> = module_re
                .captures_iter(&cap[1])
                .map(|c| c[1].trim().to_string())
                .filter(|m| !m.is_empty())
                .collect();

            for mod_name in &child_names {
                let mod_rel_path = if mod_name.starts_with("./") || mod_name.starts_with("../") {
                    mod_name.clone()
                } else {
                    format!("./{}", mod_name)
                };
                let mod_abs_path = base_path.join(mod_name);
                let artifact_id = if mod_abs_path.join("pom.xml").exists() {
                    if let Ok(child_pom) = fs::read_to_string(mod_abs_path.join("pom.xml")) {
                        regex::Regex::new(r"<artifactId>\s*([^<]+?)\s*</artifactId>")
                            .ok()
                            .and_then(|re| re.captures(&child_pom))
                            .map(|c| c[1].trim().to_string())
                    } else { None }
                } else { None };

                let children = scan_maven_modules_recursive(root_path, &mod_abs_path, depth + 1);
                modules.push(serde_json::json!({
                    "name": artifact_id.as_ref().unwrap_or(mod_name),
                    "path": mod_rel_path,
                    "type": "maven",
                    "artifactId": artifact_id,
                    "children": children
                }));
            }
        }
    }

    // 单模块工程：depth == 0 时兜底
    if modules.is_empty() && depth == 0 {
        if let Some(cap) = regex::Regex::new(r"<artifactId>\s*([^<]+?)\s*</artifactId>")
            .ok()
            .and_then(|re| re.captures(&pom))
        {
            modules.push(serde_json::json!({
                "name": &cap[1],
                "path": ".",
                "type": "maven",
                "artifactId": &cap[1],
                "children": []
            }));
        }
    }
    modules
}

fn scan_npm_modules(base_path: &Path) -> Vec<serde_json::Value> {
    use std::fs;
    let mut modules: Vec<serde_json::Value> = Vec::new();
    if let Ok(entries) = fs::read_dir(base_path) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if !ft.is_dir() { continue; }
                let name = match entry.file_name().into_string() {
                    Ok(n) => n,
                    Err(_) => continue,
                };
                if SCAN_SKIP_DIRS.contains(&name.as_str()) || name.starts_with('.') { continue; }
                let pkg_json = entry.path().join("package.json");
                if !pkg_json.exists() { continue; }
                let mod_path = format!("./{}", name);
                if modules.iter().any(|m| m.get("path").and_then(|p| p.as_str()) == Some(&mod_path)) { continue; }
                if let Ok(content) = fs::read_to_string(&pkg_json) {
                    if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                        let pkg_name = pkg.get("name").and_then(|n| n.as_str()).unwrap_or(&name);
                        modules.push(serde_json::json!({
                            "name": pkg_name,
                            "path": mod_path,
                            "type": "npm",
                            "children": []
                        }));
                    }
                }
            }
        }
    }
    modules
}

// =================== SDK Version Detection ===================
//
// 从 tailwind-migration 分支迁过来：扫描 SDKMAN 和 NVM 目录，返回可用的
// Java/Maven/Gradle/Node 版本列表。用于 CICD 编辑器的 SDK 版本选择下拉框。

/// 扫描本机 SDKMAN（`~/.sdkman/candidates/<name>/`）和 NVM（`$NVM_DIR` / `~/.nvm/versions/node`）
/// 目录，返回结构：
/// ```json
/// {
///   "sdkman": { "java": [...], "maven": [...], "gradle": [...] },
///   "nvm":    { "node": [...] },
///   "current": { "java","maven","node","npm" }
/// }
/// ```
pub fn detect_sdk_versions_impl() -> serde_json::Value {
    detect_sdk_versions_with_tools(None)
}

/// 带 tools 参数的 SDK 版本检测，避免与 detect_tools_impl 重复执行 shell 命令
pub fn detect_sdk_versions_with_tools(tools: Option<HashMap<String, ToolDetectionResult>>) -> serde_json::Value {
    let tools = tools.unwrap_or_else(detect_tools_impl);

    // ── SDKMAN ──
    let home = std::env::var("HOME").unwrap_or_default();
    let sdkman_base = std::path::Path::new(&home).join(".sdkman").join("candidates");
    let mut sdkman: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    for candidate in &["java", "maven", "gradle"] {
        let dir = sdkman_base.join(candidate);
        if !dir.exists() { continue; }
        let mut versions: Vec<serde_json::Value> = Vec::new();
        let current_link = dir.join("current");
        let current_target = if current_link.exists() && current_link.is_symlink() {
            std::fs::read_link(&current_link).ok().map(|p| p.to_string_lossy().to_string())
        } else { None };
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) { continue; }
                let name = entry.file_name().to_string_lossy().to_string();
                if name == "current" { continue; }
                let ver_path = entry.path();
                let is_current = current_target.as_ref().map_or(false, |ct| ct.ends_with(&name));
                versions.push(serde_json::json!({
                    "name": name,
                    "path": ver_path.to_string_lossy(),
                    "isCurrent": is_current,
                }));
            }
        }
        versions.sort_by(|a, b| {
            let a_cur = a.get("isCurrent").and_then(|v| v.as_bool()).unwrap_or(false);
            let b_cur = b.get("isCurrent").and_then(|v| v.as_bool()).unwrap_or(false);
            if a_cur != b_cur {
                return if a_cur { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater };
            }
            b.get("name").and_then(|v| v.as_str()).cmp(&a.get("name").and_then(|v| v.as_str()))
        });
        sdkman.insert(candidate.to_string(), serde_json::Value::Array(versions));
    }

    // ── NVM node 版本 ──
    let mut nvm_versions: Vec<serde_json::Value> = Vec::new();
    let nvm_paths = [
        std::env::var("NVM_DIR").ok().map(|p| std::path::PathBuf::from(p).join("versions").join("node")),
        Some(std::path::PathBuf::from(&home).join(".nvm").join("versions").join("node")),
        Some(std::path::PathBuf::from("/opt/homebrew/opt/nvm/versions/node")),
        Some(std::path::PathBuf::from("/usr/local/opt/nvm/versions/node")),
    ];
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for nvm_base in nvm_paths.iter().flatten() {
        if !nvm_base.exists() { continue; }
        let current_link = nvm_base.join("current");
        let current_target = if current_link.exists() && current_link.is_symlink() {
            std::fs::read_link(&current_link).ok().map(|p| p.to_string_lossy().to_string())
        } else { None };
        if let Ok(entries) = std::fs::read_dir(nvm_base) {
            for entry in entries.flatten() {
                if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) { continue; }
                let name = entry.file_name().to_string_lossy().to_string();
                if name == "current" || seen.contains(&name) { continue; }
                let ver_path = entry.path();
                if !ver_path.join("bin").join("node").exists() { continue; }
                seen.insert(name.clone());
                let bin_dir = ver_path.join("bin");
                let is_current = current_target.as_ref().map_or(false, |ct| ct.ends_with(&name));
                nvm_versions.push(serde_json::json!({
                    "name": name,
                    "path": ver_path.to_string_lossy(),
                    "isCurrent": is_current,
                    "npm":  bin_dir.join("npm").exists().then(||  bin_dir.join("npm").to_string_lossy().to_string()),
                    "pnpm": bin_dir.join("pnpm").exists().then(|| bin_dir.join("pnpm").to_string_lossy().to_string()),
                    "yarn": bin_dir.join("yarn").exists().then(|| bin_dir.join("yarn").to_string_lossy().to_string()),
                }));
            }
        }
    }
    nvm_versions.sort_by(|a, b| {
        let a_cur = a.get("isCurrent").and_then(|v| v.as_bool()).unwrap_or(false);
        let b_cur = b.get("isCurrent").and_then(|v| v.as_bool()).unwrap_or(false);
        if a_cur != b_cur {
            return if a_cur { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater };
        }
        b.get("name").and_then(|v| v.as_str()).cmp(&a.get("name").and_then(|v| v.as_str()))
    });

    serde_json::json!({
        "sdkman": sdkman,
        "nvm": { "node": nvm_versions },
        "current": {
            "maven": tools.get("maven").and_then(|t| t.version.clone()).unwrap_or_default(),
            "java":  tools.get("java").and_then(|t| t.version.clone()).unwrap_or_default(),
            "node":  tools.get("node").and_then(|t| t.version.clone()).unwrap_or_default(),
            "npm":   tools.get("npm").and_then(|t| t.version.clone()).unwrap_or_default(),
        }
    })
}
