use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::{LazyLock, Mutex};
use supertool_core::db::cicd::*;
use supertool_core::logic::CoreService;
use supertool_core::logic::cicd_deploy::{
    self, DeployConfig, DeployModuleConfig, DeployServerConfig,
};
use tauri::{Emitter, State};

// 部署取消状态管理：cancel_deploy 将 deploy_id 加入此集合，deploy 任务检查后提前退出
static CANCELLED_DEPLOYS: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

fn is_deploy_cancelled(deploy_id: &str) -> bool {
    CANCELLED_DEPLOYS
        .lock()
        .map(|set| set.contains(deploy_id))
        .unwrap_or(false)
}

// =================== Types ===================

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
    #[serde(rename = "npmHome")]
    pub npm_home: String,
    #[serde(rename = "pnpmHome")]
    pub pnpm_home: String,
    #[serde(rename = "yarnHome")]
    pub yarn_home: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectScanResult {
    #[serde(rename = "buildTool")]
    pub build_tool: Option<String>,
    #[serde(rename = "currentBranch")]
    pub current_branch: Option<String>,
    #[serde(rename = "gitRemoteUrl")]
    pub git_remote_url: Option<String>,
    #[serde(rename = "npmScripts")]
    pub npm_scripts: Option<Vec<String>>,
    #[serde(rename = "recommendedScript")]
    pub recommended_script: Option<String>,
    #[serde(rename = "projectName")]
    pub project_name: Option<String>,
    #[serde(rename = "packageManager")]
    pub package_manager: Option<String>,
    #[serde(rename = "mavenProfiles")]
    pub maven_profiles: Option<Vec<String>>,
    #[serde(rename = "recommendedProfile")]
    pub recommended_profile: Option<String>,
    #[serde(rename = "isMultiModule")]
    pub is_multi_module: Option<bool>,
    #[serde(rename = "moduleNames")]
    pub module_names: Option<Vec<String>>,
    #[serde(rename = "hasParent")]
    pub has_parent: Option<bool>,
    #[serde(rename = "suggestedDeployPath")]
    pub suggested_deploy_path: Option<String>,
}

// =================== Tool Detection Commands ===================

#[tauri::command(rename_all = "camelCase")]
pub fn detect_tool_paths() -> ToolPaths {
    log::info!("[Tauri CMD] detect_tool_paths() called");
    detect_tool_paths_impl()
}

#[tauri::command(rename_all = "camelCase")]
pub fn detect_build_tools() -> HashMap<String, ToolDetectionResult> {
    log::info!("[Tauri CMD] detect_build_tools() called");
    detect_tools_impl()
}

#[tauri::command(rename_all = "camelCase")]
pub fn scan_project(local_path: String) -> ProjectScanResult {
    log::info!("[Tauri CMD] scan_project() called");
    scan_project_impl(&local_path)
}

// =================== SDK Version Check Commands ===================

#[tauri::command(rename_all = "camelCase")]
pub fn detect_sdk_versions() -> serde_json::Value {
    log::info!("[Tauri CMD] detect_sdk_versions() called");
    let tools = detect_tools_impl();

    // Scan SDKMAN candidates
    let home = std::env::var("HOME").unwrap_or_default();
    let sdkman_base = std::path::Path::new(&home)
        .join(".sdkman")
        .join("candidates");

    let mut sdkman: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    for candidate in &["java", "maven", "gradle"] {
        let dir = sdkman_base.join(candidate);
        if !dir.exists() {
            continue;
        }

        let mut versions: Vec<serde_json::Value> = Vec::new();
        let current_link = dir.join("current");
        let current_target = if current_link.exists() && current_link.is_symlink() {
            std::fs::read_link(&current_link)
                .ok()
                .map(|p| p.to_string_lossy().to_string())
        } else {
            None
        };

        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                let ver_path = entry.path();
                let is_current = current_target
                    .as_ref()
                    .map_or(false, |ct| ct.ends_with(&name));
                versions.push(serde_json::json!({
                    "name": name,
                    "path": ver_path.to_string_lossy(),
                    "isCurrent": is_current,
                }));
            }
        }
        versions.sort_by(|a, b| {
            let a_cur = a
                .get("isCurrent")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let b_cur = b
                .get("isCurrent")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if a_cur != b_cur {
                return if a_cur {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                };
            }
            b.get("name")
                .and_then(|v| v.as_str())
                .cmp(&a.get("name").and_then(|v| v.as_str()))
        });
        sdkman.insert(candidate.to_string(), serde_json::Value::Array(versions));
    }

    // Scan NVM versions
    let mut nvm_versions: Vec<serde_json::Value> = Vec::new();
    let nvm_paths = [
        std::env::var("NVM_DIR")
            .ok()
            .map(|p| std::path::PathBuf::from(p).join("versions").join("node")),
        Some(
            std::path::PathBuf::from(&home)
                .join(".nvm")
                .join("versions")
                .join("node"),
        ),
        Some(std::path::PathBuf::from(
            "/opt/homebrew/opt/nvm/versions/node",
        )),
        Some(std::path::PathBuf::from("/usr/local/opt/nvm/versions/node")),
    ];

    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for nvm_base in nvm_paths.iter().flatten() {
        if !nvm_base.exists() {
            continue;
        }
        let current_link = nvm_base.join("current");
        let current_target = if current_link.exists() && current_link.is_symlink() {
            std::fs::read_link(&current_link)
                .ok()
                .map(|p| p.to_string_lossy().to_string())
        } else {
            None
        };

        if let Ok(entries) = std::fs::read_dir(nvm_base) {
            for entry in entries.flatten() {
                if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                if name == "current" || seen.contains(&name) {
                    continue;
                }
                let ver_path = entry.path();
                if !ver_path.join("bin").join("node").exists() {
                    continue;
                }
                seen.insert(name.clone());

                let bin_dir = ver_path.join("bin");
                let is_current = current_target
                    .as_ref()
                    .map_or(false, |ct| ct.ends_with(&name));
                nvm_versions.push(serde_json::json!({
                    "name": name,
                    "path": ver_path.to_string_lossy(),
                    "isCurrent": is_current,
                    "npm": bin_dir.join("npm").exists().then(|| bin_dir.join("npm").to_string_lossy().to_string()),
                    "pnpm": bin_dir.join("pnpm").exists().then(|| bin_dir.join("pnpm").to_string_lossy().to_string()),
                    "yarn": bin_dir.join("yarn").exists().then(|| bin_dir.join("yarn").to_string_lossy().to_string()),
                }));
            }
        }
    }
    nvm_versions.sort_by(|a, b| {
        let a_cur = a
            .get("isCurrent")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let b_cur = b
            .get("isCurrent")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if a_cur != b_cur {
            return if a_cur {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        b.get("name")
            .and_then(|v| v.as_str())
            .cmp(&a.get("name").and_then(|v| v.as_str()))
    });

    serde_json::json!({
        "sdkman": sdkman,
        "nvm": { "node": nvm_versions },
        "current": {
            "maven": tools.get("maven").and_then(|t| t.version.clone()).unwrap_or_default(),
            "java": tools.get("java").and_then(|t| t.version.clone()).unwrap_or_default(),
            "node": tools.get("node").and_then(|t| t.version.clone()).unwrap_or_default(),
            "npm": tools.get("npm").and_then(|t| t.version.clone()).unwrap_or_default(),
        }
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn check_java(java_home: Option<String>) -> ToolDetectionResult {
    log::info!("[Tauri CMD] check_java() called, javaHome={:?}", java_home);
    let cmd = if let Some(ref home) = java_home {
        let java_path = if Path::new(home).is_file() || home.ends_with("/java") {
            home.clone()
        } else {
            format!("{}/bin/java", home)
        };
        format!("{} -version", java_path)
    } else {
        "java -version".to_string()
    };
    let result = run_command(&cmd, None);
    let mut version = None;
    let mut available = false;
    if result.success || !result.stderr.is_empty() {
        // java -version outputs to stderr
        let output = if !result.stderr.is_empty() {
            &result.stderr
        } else {
            &result.stdout
        };
        if let Some(line) = output.lines().next() {
            let ver_str = line.trim().to_string();
            if !ver_str.is_empty() {
                version = Some(ver_str);
                available = true;
            }
        }
    }
    let path = run_command("which java", None).stdout.trim().to_string();
    ToolDetectionResult {
        available,
        version,
        path: if path.is_empty() { None } else { Some(path) },
    }
}

#[tauri::command(rename_all = "camelCase")]
pub fn check_maven(maven_home: Option<String>) -> ToolDetectionResult {
    log::info!(
        "[Tauri CMD] check_maven() called, mavenHome={:?}",
        maven_home
    );
    let cmd = if let Some(ref home) = maven_home {
        // 智能检测: 如果路径以 mvn 结尾(是二进制)则直接使用, 否则当作 MAVEN_HOME 拼接 /bin/mvn
        let mvn_path = if Path::new(home).is_file() || home.ends_with("/mvn") {
            home.clone()
        } else {
            format!("{}/bin/mvn", home)
        };
        format!("{} -version", mvn_path)
    } else {
        "mvn -version".to_string()
    };
    let result = run_command(&cmd, None);
    let mut version = None;
    let available = result.success;
    if available {
        version = result.stdout.lines().next().map(|s| s.trim().to_string());
    }
    let path = run_command("which mvn", None).stdout.trim().to_string();
    ToolDetectionResult {
        available,
        version,
        path: if path.is_empty() { None } else { Some(path) },
    }
}

#[tauri::command(rename_all = "camelCase")]
pub fn check_node(node_home: Option<String>) -> ToolDetectionResult {
    log::info!("[Tauri CMD] check_node() called, nodeHome={:?}", node_home);
    let cmd = if let Some(ref home) = node_home {
        let node_path = if Path::new(home).is_file() || home.ends_with("/node") {
            home.clone()
        } else {
            format!("{}/bin/node", home)
        };
        format!("{} -v", node_path)
    } else {
        "node -v".to_string()
    };
    let result = run_command(&cmd, None);
    let mut version = None;
    let available = result.success;
    if available {
        version = result.stdout.lines().next().map(|s| s.trim().to_string());
    }
    let path = run_command("which node", None).stdout.trim().to_string();
    ToolDetectionResult {
        available,
        version,
        path: if path.is_empty() { None } else { Some(path) },
    }
}

// =================== Implementation Functions ===================

pub fn detect_tools_impl() -> HashMap<String, ToolDetectionResult> {
    let mut tools = HashMap::new();
    let tool_list = [
        ("mvn", "maven"),
        ("npm", "npm"),
        ("node", "node"),
        ("java", "java"),
        ("gradle", "gradle"),
        ("pnpm", "pnpm"),
        ("yarn", "yarn"),
    ];
    for (cmd, name) in &tool_list {
        let result = run_command(&format!("{} --version", cmd), None);
        if result.success {
            let version = result.stdout.lines().next().map(|s| s.trim().to_string());
            let path = run_command(&format!("which {}", cmd), None)
                .stdout
                .trim()
                .to_string();
            tools.insert(
                name.to_string(),
                ToolDetectionResult {
                    available: true,
                    version,
                    path: if path.is_empty() { None } else { Some(path) },
                },
            );
        } else {
            tools.insert(
                name.to_string(),
                ToolDetectionResult {
                    available: false,
                    version: None,
                    path: None,
                },
            );
        }
    }
    if let Some(maven) = tools.get_mut("maven") {
        if maven.available {
            maven.path = find_maven_home();
        }
    }
    tools
}

pub fn detect_tool_paths_impl() -> ToolPaths {
    fn find_path(cmd: &str) -> String {
        let which = run_command(&format!("which {}", cmd), None);
        if !which.stdout.trim().is_empty() {
            return which.stdout.trim().to_string();
        }
        let whereis = run_command(&format!("whereis {}", cmd), None);
        let parts: Vec<&str> = whereis.stdout.trim().split_whitespace().collect();
        if parts.len() > 1 {
            return parts[1].to_string();
        }
        String::new()
    }
    fn strip_bin(bin_path: &str) -> String {
        if let Some(pos) = bin_path.rfind("/bin/") {
            if pos > 0 {
                return bin_path[..pos].to_string();
            }
        }
        bin_path.to_string()
    }
    let mvn_bin = find_path("mvn");
    let java_bin = find_path("java");
    let node_bin = find_path("node");
    let npm_bin = find_path("npm");
    let pnpm_bin = find_path("pnpm");
    let yarn_bin = find_path("yarn");
    ToolPaths {
        maven_home: mvn_bin,
        java_home: if java_bin.is_empty() {
            String::new()
        } else {
            strip_bin(&java_bin)
        },
        node_home: if node_bin.is_empty() {
            String::new()
        } else {
            strip_bin(&node_bin)
        },
        npm_home: npm_bin,
        pnpm_home: pnpm_bin,
        yarn_home: yarn_bin,
    }
}

pub fn scan_project_impl(local_path: &str) -> ProjectScanResult {
    let mut result = ProjectScanResult {
        build_tool: None,
        current_branch: None,
        git_remote_url: None,
        npm_scripts: None,
        recommended_script: None,
        project_name: None,
        package_manager: None,
        maven_profiles: None,
        recommended_profile: None,
        is_multi_module: None,
        module_names: None,
        has_parent: None,
        suggested_deploy_path: None,
    };
    if !Path::new(local_path).exists() {
        return result;
    }
    let has_pom = Path::new(local_path).join("pom.xml").exists();
    let has_package = Path::new(local_path).join("package.json").exists();
    if has_pom {
        result.build_tool = Some("maven".to_string());
    } else if has_package {
        result.build_tool = Some("npm".to_string());
    }
    // Git branch
    let branch_result = run_command("git rev-parse --abbrev-ref HEAD", Some(local_path));
    if branch_result.success {
        let branch = branch_result.stdout.trim();
        if !branch.is_empty() && branch != "HEAD" {
            result.current_branch = Some(branch.to_string());
        }
    }
    // Git remote URL
    let remote_result = run_command("git remote get-url origin", Some(local_path));
    if remote_result.success {
        let remote = remote_result.stdout.trim();
        if !remote.is_empty() {
            result.git_remote_url = Some(remote.to_string());
        }
    }
    // Parse package.json
    if has_package {
        if let Ok(content) = fs::read_to_string(Path::new(local_path).join("package.json")) {
            if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(scripts) = pkg.get("scripts").and_then(|s| s.as_object()) {
                    let script_names: Vec<String> = scripts.keys().cloned().collect();
                    result.npm_scripts = Some(script_names.clone());
                    if scripts.contains_key("build") {
                        result.recommended_script = Some("build".to_string());
                    } else if scripts.contains_key("prod") {
                        result.recommended_script = Some("prod".to_string());
                    } else if scripts.contains_key("start") {
                        result.recommended_script = Some("start".to_string());
                    }
                }
                if let Some(name) = pkg.get("name").and_then(|n| n.as_str()) {
                    result.project_name = Some(name.to_string());
                }
                if Path::new(local_path).join("pnpm-lock.yaml").exists() {
                    result.package_manager = Some("pnpm".to_string());
                } else if Path::new(local_path).join("yarn.lock").exists() {
                    result.package_manager = Some("yarn".to_string());
                } else if Path::new(local_path).join("package-lock.json").exists() {
                    result.package_manager = Some("npm".to_string());
                }
            }
        }
    }
    // Parse pom.xml
    if has_pom {
        if let Ok(pom) = fs::read_to_string(Path::new(local_path).join("pom.xml")) {
            if let Some(cap) = regex::Regex::new(r"<artifactId>([^<]+)</artifactId>")
                .ok()
                .and_then(|re| re.captures(&pom))
            {
                result.project_name = Some(cap[1].to_string());
            }
            if let Ok(re) = regex::Regex::new(r"<profile>\s*<id>([^<]+)</id>") {
                let profiles: Vec<String> = re
                    .captures_iter(&pom)
                    .map(|cap| cap[1].to_string())
                    .collect();
                if !profiles.is_empty() {
                    let recommended = if profiles.contains(&"prod".to_string()) {
                        "prod".to_string()
                    } else if profiles.contains(&"production".to_string()) {
                        "production".to_string()
                    } else {
                        profiles[0].clone()
                    };
                    result.maven_profiles = Some(profiles);
                    result.recommended_profile = Some(recommended);
                }
            }
            if let Ok(re) = regex::Regex::new(r"<modules>\s*([\s\S]*?)</modules>") {
                if let Some(cap) = re.captures(&pom) {
                    let module_re = regex::Regex::new(r"<module>\s*([^<]+?)\s*</module>").unwrap();
                    let modules: Vec<String> = module_re
                        .captures_iter(&cap[1])
                        .map(|c| c[1].trim().to_string())
                        .filter(|m| !m.is_empty())
                        .collect();
                    if modules.len() > 1 {
                        result.is_multi_module = Some(true);
                        result.module_names = Some(modules);
                    }
                }
            }
            if regex::Regex::new(r"<parent>[\s\S]*?</parent>")
                .ok()
                .map(|re| re.is_match(&pom))
                .unwrap_or(false)
            {
                result.has_parent = Some(true);
            }
        }
    }
    if result.build_tool.as_deref() == Some("maven") {
        result.suggested_deploy_path = Some("~/apphome".to_string());
    } else if result.build_tool.as_deref() == Some("npm") {
        result.suggested_deploy_path = Some("/home/nginxWebUI/ui".to_string());
    }
    result
}

struct CommandOutput {
    stdout: String,
    stderr: String,
    success: bool,
}

fn run_command(cmd: &str, cwd: Option<&str>) -> CommandOutput {
    let mut parts = cmd.split_whitespace();
    let program = parts.next().unwrap_or("");
    let args: Vec<&str> = parts.collect();
    // 使用 user_shell_cmd 获取用户完整 PATH（含 Homebrew/sdkman/nvm 等）
    let mut command = std::process::Command::new("sh");
    let full_cmd = if args.is_empty() {
        program.to_string()
    } else {
        format!("{} {}", program, args.join(" "))
    };
    command.arg("-c").arg(&full_cmd);
    // 注入用户 shell 环境变量
    let shell_env = supertool_core::logic::cicd_deploy::get_shell_env_for_command();
    for (key, value) in &shell_env {
        command.env(key, value);
    }
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());
    if let Some(dir) = cwd {
        command.current_dir(dir);
    }
    match command.output() {
        Ok(output) => CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
        },
        Err(_) => CommandOutput {
            stdout: String::new(),
            stderr: String::new(),
            success: false,
        },
    }
}

fn find_maven_home() -> Option<String> {
    let mvn_path = run_command("which mvn", None).stdout.trim().to_string();
    if mvn_path.is_empty() {
        return None;
    }
    let resolved = if let Ok(path) = fs::canonicalize(&mvn_path) {
        path.to_string_lossy().to_string()
    } else {
        mvn_path
    };
    if let Some(pos) = resolved.rfind("/bin/mvn") {
        if pos > 0 {
            return Some(resolved[..pos].to_string());
        }
    }
    Some(resolved)
}

// =================== CICD Config CRUD Commands ===================

#[tauri::command(rename_all = "camelCase")]
pub async fn get_cicd_configs(core: State<'_, CoreService>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_cicd_configs() called");
    let configs = core.db_read(|conn| cicd_get_all_configs(conn).expect("db error"))?;
    serde_json::to_value(&configs).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_cicd_config_by_id(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_cicd_config_by_id() called");
    let config = core.db_read(|conn| cicd_get_config_by_id(conn, &id).expect("db error"))?;
    match config {
        Some(c) => serde_json::to_value(&c).map_err(|e| e.to_string()),
        None => Ok(serde_json::Value::Null),
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_cicd_groups(core: State<'_, CoreService>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_cicd_groups() called");
    let groups = core.db_read(|conn| cicd_get_groups(conn).map_err(|e| e.to_string()))??;
    serde_json::to_value(&groups).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn save_cicd_config(
    core: State<'_, CoreService>,
    config: serde_json::Value,
    modules: Option<Vec<serde_json::Value>>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] save_cicd_config() called");
    let now = chrono::Utc::now().to_rfc3339();
    let mut cicd_config: CicdConfig =
        serde_json::from_value(config.clone()).map_err(|e| format!("解析配置失败: {}", e))?;
    let result = core.db_write(|conn| {
        let existing = match cicd_get_config_by_id(conn, &cicd_config.id) {
            Ok(v) => v,
            Err(e) => panic!(
                "{}",
                supertool_core::logic::log_sanitizer::sanitize_string(&e.to_string())
            ),
        };
        if existing.is_some() {
            cicd_config.updated_at = now.clone();
            if let Err(e) = cicd_update_config(conn, &cicd_config) {
                panic!(
                    "{}",
                    supertool_core::logic::log_sanitizer::sanitize_string(&e.to_string())
                );
            }
        } else {
            cicd_config.created_at = now.clone();
            cicd_config.updated_at = now.clone();
            if let Err(e) = cicd_add_config(conn, &cicd_config) {
                panic!(
                    "{}",
                    supertool_core::logic::log_sanitizer::sanitize_string(&e.to_string())
                );
            }
        }
        // Handle modules
        if let Some(mods) = modules {
            if let Err(e) = conn.execute(
                "DELETE FROM deploy_modules WHERE configId = ?",
                [&cicd_config.id],
            ) {
                panic!(
                    "{}",
                    supertool_core::logic::log_sanitizer::sanitize_string(&e.to_string())
                );
            }
            for m in &mods {
                let mut module: DeployModule =
                    serde_json::from_value(m.clone()).expect("parse module error");
                module.config_id = cicd_config.id.clone();
                module.created_at = now.clone();
                module.updated_at = now.clone();
                if let Err(e) = cicd_add_module(conn, &module) {
                    panic!(
                        "{}",
                        supertool_core::logic::log_sanitizer::sanitize_string(&e.to_string())
                    );
                }
            }
        }
        match cicd_get_config_by_id(conn, &cicd_config.id) {
            Ok(v) => v,
            Err(e) => panic!(
                "{}",
                supertool_core::logic::log_sanitizer::sanitize_string(&e.to_string())
            ),
        }
    });
    match result {
        Ok(Some(c)) => serde_json::to_value(&c).map_err(|e| e.to_string()),
        Ok(None) => Err("保存配置失败".to_string()),
        Err(e) => Err(e),
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_cicd_config(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_cicd_config() called");
    core.db_write(|conn| {
        conn.execute("DELETE FROM deploy_modules WHERE configId = ?", [&id])
            .expect("db delete modules error");
        cicd_delete_config(conn, &id).expect("db delete config error");
        serde_json::json!({ "id": id })
    })
}

// =================== Deploy Commands ===================

#[tauri::command(rename_all = "camelCase")]
pub async fn deploy(
    app: tauri::AppHandle,
    core: State<'_, CoreService>,
    config_id: String,
    confirmed: Option<bool>,
    branch: Option<String>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] deploy() called");
    // Get config from DB
    let cicd_config = core
        .db_read(|conn| cicd_get_config_by_id(conn, &config_id).map_err(|e| e.to_string()))??
        .ok_or("CI/CD 配置不存在")?;

    // Check approval requirement
    if cicd_config.requires_approval && confirmed != Some(true) {
        return Ok(serde_json::json!({
            "success": false,
            "requiresApproval": true,
            "message": "此配置需要审核确认，请确认后再次部署",
            "configName": cicd_config.name
        }));
    }

    let modules =
        core.db_read(|conn| cicd_get_modules(conn, &config_id).map_err(|e| e.to_string()))??;

    // Build DeployConfig
    let mut deploy_config = build_deploy_config(&core, &cicd_config, &modules)?;

    // Override branch if provided (per-deploy branch selection)
    if let Some(ref b) = branch {
        if !b.is_empty() {
            log::info!("[deploy] overriding branch: {} -> {}", deploy_config.branch, b);
            deploy_config.branch = b.clone();
        }
    }

    // Create deploy log
    let deploy_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let deploy_log = DeployLog {
        id: deploy_id.clone(),
        config_id: config_id.clone(),
        status: "running".to_string(),
        start_time: now.clone(),
        end_time: None,
        error_message: None,
        progress: 0,
        triggered_by: "user".to_string(),
        created_at: now.clone(),
        log_file_path: None,
        artifact_paths: None,
    };

    // Save deploy log
    core.db_write(|conn| -> Result<(), String> {
        cicd_add_deploy_log(conn, &deploy_log).map_err(|e| e.to_string())?;
        cicd_touch_deploy(conn, &config_id).map_err(|e| e.to_string())?;
        Ok(())
    })??;

    // Get app dir for deploy logs
    let app_dir = core.db_read(|conn| {
        conn.query_row(
            "SELECT value FROM settings WHERE key = 'appDataDir'",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .unwrap_or_else(|| "/tmp".to_string())
    })?;

    // Emit deploy-log-id-created event immediately
    let _ = app.emit(
        "deploy-log-id-created",
        serde_json::json!({ "deployLogId": deploy_id }),
    );

    let deploy_id_arc = std::sync::Arc::new(deploy_id.clone());
    let config_id_arc = std::sync::Arc::new(config_id.clone());
    let app_arc = std::sync::Arc::new(app.clone());
    let core_clone = core.inner().clone();

    let did_for_closure = deploy_id_arc.clone();
    let cid_for_closure = config_id_arc.clone();
    let app_for_closure = app_arc.clone();

    // Spawn background task for deploy
    tokio::spawn(async move {
        let did_for_cancel = deploy_id_arc.clone();

        let deploy_result = cicd_deploy::execute_deploy(
            &deploy_config,
            &app_dir,
            &deploy_id_arc,
            move |event| {
                let payload = serde_json::json!({
                    "deployLogId": *did_for_closure,
                    "configId": *cid_for_closure,
                    "stage": event.stage,
                    "status": event.status,
                    "message": event.message,
                    "progress": event.progress,
                });
                let _ = app_for_closure.emit("deploy-progress", &payload);
            },
            move || is_deploy_cancelled(&did_for_cancel),
        )
        .await;

        // Update deploy log with result
        let final_status: String;
        let final_error: Option<String>;
        let final_log_path: Option<String>;
        let final_artifact_paths: Option<String>;
        let final_progress: i64;

        match &deploy_result {
            Ok(result) => {
                final_status = if result.cancelled == Some(true) {
                    "cancelled".to_string()
                } else if result.success {
                    "success".to_string()
                } else {
                    "failed".to_string()
                };
                final_error = result.error.clone();
                final_progress = if result.success { 100 } else { 0 };
                final_log_path = Some(result.log_file_path.clone());
                final_artifact_paths =
                    Some(serde_json::to_string(&result.artifact_paths).unwrap_or_default());
            }
            Err(e) => {
                final_status = "failed".to_string();
                final_error = Some(e.clone());
                final_progress = 0;
                final_log_path = None;
                final_artifact_paths = None;
            }
        };

        let new_log = DeployLog {
            id: (*deploy_id_arc).clone(),
            config_id: (*config_id_arc).clone(),
            status: final_status,
            start_time: now.clone(),
            end_time: Some(chrono::Utc::now().to_rfc3339()),
            error_message: final_error,
            progress: final_progress,
            triggered_by: "user".to_string(),
            created_at: now.clone(),
            log_file_path: final_log_path,
            artifact_paths: final_artifact_paths,
        };
        let _ = core_clone.db_write(|conn| cicd_update_deploy_log(conn, &new_log));

        // 写入 deploy_history 记录（供前端部署历史展示使用，成功和失败都记录）
        let history_status = match &deploy_result {
            Ok(result) => {
                if result.cancelled == Some(true) {
                    "cancelled"
                } else if result.success {
                    "success"
                } else {
                    "failed"
                }
            }
            Err(_) => "failed",
        };
        let history = crate::commands::cicd::DeployHistory {
            id: (*deploy_id_arc).clone(),
            config_id: (*config_id_arc).clone(),
            status: history_status.to_string(),
            deployed_at: chrono::Utc::now().to_rfc3339(),
            rolled_back: false,
            rolled_back_at: None,
        };
        let _ = core_clone
            .db_write(|conn| crate::commands::cicd::cicd_add_deploy_history(conn, &history));

        // 清理取消标记
        if let Ok(mut set) = CANCELLED_DEPLOYS.lock() {
            set.remove(&*deploy_id_arc);
        }

        // Emit final notification (native system notification + event to frontend)
        let is_cancelled = match &deploy_result {
            Ok(result) => result.cancelled == Some(true),
            Err(_) => false,
        };
        match &deploy_result {
            Ok(result) => {
                crate::tray_notification::show_deploy_notification(
                    result.success && !is_cancelled,
                    &cicd_config.name,
                    result.error.as_deref(),
                );
                let _ = app_arc.emit(
                    "deploy-notification",
                    serde_json::json!({
                        "success": result.success,
                        "cancelled": is_cancelled,
                        "configId": *config_id_arc,
                        "deployLogId": *deploy_id_arc,
                        "error": result.error,
                    }),
                );
            }
            Err(e) => {
                crate::tray_notification::show_deploy_notification(
                    false,
                    &cicd_config.name,
                    Some(e),
                );
                let _ = app_arc.emit(
                    "deploy-notification",
                    serde_json::json!({
                        "success": false,
                        "configId": *config_id_arc,
                        "deployLogId": *deploy_id_arc,
                        "error": e,
                    }),
                );
            }
        }
    });

    // Return deploy ID immediately so frontend can track progress
    Ok(serde_json::json!({
        "deployId": deploy_id,
        "success": true,
        "message": "部署已启动",
        "status": "running",
    }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn cancel_deploy(
    core: State<'_, CoreService>,
    deploy_log_id: String,
) -> Result<serde_json::Value, String> {
    log::info!(
        "[Tauri CMD] cancel_deploy() called, deploy_log_id={}",
        deploy_log_id
    );

    // 标记为已取消，deploy 任务会在下一个检查点退出
    if let Ok(mut set) = CANCELLED_DEPLOYS.lock() {
        set.insert(deploy_log_id.clone());
    }

    let result = core.db_write(|conn| {
        let log = cicd_get_deploy_log_by_id(conn, &deploy_log_id);
        match log {
            Some(mut log) if log.status == "running" || log.status == "pending" => {
                log.status = "cancelled".to_string();
                log.end_time = Some(chrono::Utc::now().to_rfc3339());
                log.error_message = Some("用户取消部署".to_string());
                cicd_update_deploy_log(conn, &log).expect("db error");
                Ok(serde_json::json!({ "success": true, "status": "cancelled" }))
            }
            Some(log) => Ok(serde_json::json!({
                "success": false,
                "error": format!("部署状态为 {}，无法取消", log.status)
            })),
            None => Ok(serde_json::json!({
                "success": false,
                "error": "部署记录不存在"
            })),
        }
    });
    match result {
        Ok(v) => v,
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn rollback(
    core: State<'_, CoreService>,
    config_id: String,
    log_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] rollback() called");
    let _deploy_log = core
        .db_read(|conn| cicd_get_deploy_log_by_id(conn, &log_id))?
        .ok_or("部署记录不存在")?;

    // Get CICD config to read server info
    let cicd_config = core
        .db_read(|conn| cicd_get_config_by_id(conn, &config_id).expect("db error"))?
        .ok_or("CI/CD 配置不存在")?;

    let rollback_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Parse servers from config JSON
    let mut rollback_errors: Vec<String> = Vec::new();
    if let Some(ref servers_str) = cicd_config.servers {
        if let Ok(servers) = serde_json::from_str::<Vec<serde_json::Value>>(servers_str) {
            for server_val in &servers {
                let host_or_id = server_val
                    .get("host")
                    .or_else(|| server_val.get("serverId"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if host_or_id.is_empty() {
                    continue;
                }

                // Query full server config from servers table
                let server = match core.db_read(|conn| {
                    conn.query_row(
                        "SELECT * FROM servers WHERE id = ? OR host = ?",
                        rusqlite::params![host_or_id, host_or_id],
                        supertool_core::db::servers::row_to_server,
                    )
                }) {
                    Ok(Ok(s)) => s,
                    _ => {
                        rollback_errors.push(format!("{}: 服务器不存在", host_or_id));
                        continue;
                    }
                };

                let host = server.host.clone();
                let port = server.port as u16;
                let username = server.username.clone();
                let password = server
                    .password
                    .clone()
                    .map(|pw| supertool_core::encryption::try_decrypt_password(&pw));
                let ssh_key = server.ssh_key_path.clone();

                // Execute restart script via SSH
                match execute_remote_restart(
                    host.clone(),
                    port,
                    username,
                    password,
                    ssh_key,
                    cicd_config.restart_script.clone(),
                )
                .await
                {
                    Ok(_) => log::info!("[rollback] {}:{} restart successful", host, port),
                    Err(e) => {
                        log::error!("[rollback] {}:{} restart failed: {}", host, port, e);
                        rollback_errors.push(format!("{}:{} → {}", host, port, e));
                    }
                }
            }
        } else {
            rollback_errors.push("服务器配置解析失败".to_string());
        }
    } else {
        rollback_errors.push("未配置部署服务器".to_string());
    }

    // Record rollback in deploy history
    let history = DeployHistory {
        id: rollback_id,
        config_id: config_id.clone(),
        status: if rollback_errors.is_empty() {
            "rollback-success".to_string()
        } else {
            "rollback-partial".to_string()
        },
        deployed_at: now.clone(),
        rolled_back: true,
        rolled_back_at: Some(now.clone()),
    };
    let _ = core.db_write(|conn| cicd_add_deploy_history(conn, &history))?;

    Ok(serde_json::json!({
        "success": rollback_errors.is_empty(),
        "rollbackId": history.id,
        "message": if rollback_errors.is_empty() {
            "回滚成功：已在所有服务器执行重启".to_string()
        } else {
            format!("部分成功: {}", rollback_errors.join("; "))
        },
        "errors": rollback_errors,
    }))
}

/// Execute a restart command on a remote server via SSH
/// 使用 spawn_blocking 避免阻塞 tokio async 运行时
async fn execute_remote_restart(
    host: String,
    port: u16,
    username: String,
    password: Option<String>,
    private_key: Option<String>,
    restart_script: String,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        use ssh2::Session;
        use std::net::TcpStream;

        let addr = format!("{}:{}", host, port);
        let tcp = TcpStream::connect(&addr).map_err(|e| format!("连接 {} 失败: {}", addr, e))?;

        let mut sess = Session::new().map_err(|e| format!("创建 SSH session 失败: {}", e))?;
        sess.set_tcp_stream(tcp);
        sess.set_timeout(30_000);
        sess.handshake()
            .map_err(|e| format!("SSH 握手失败: {}", e))?;

        if let Some(key_path) = private_key {
            sess.userauth_pubkey_file(
                &username,
                None,
                std::path::Path::new(&key_path),
                password.as_deref(),
            )
            .map_err(|e| format!("SSH 密钥认证失败: {}", e))?;
        } else if let Some(ref pw) = password {
            sess.userauth_password(&username, pw)
                .map_err(|e| format!("SSH 密码认证失败: {}", e))?;
        } else {
            return Err("缺少认证信息".to_string());
        }

        if !sess.authenticated() {
            return Err("SSH 认证失败".to_string());
        }

        // 对齐 core/src/logic/cicd_deploy.rs execute_restart 的修复
        // 使用 bash -l -c 加载用户环境变量（JAVA_HOME 等）
        let exec_cmd = if restart_script.starts_with('/') {
            format!(
                "chmod +x {} && bash -l -c '{}' 2>&1",
                restart_script, restart_script
            )
        } else {
            // 相对路径需要 cd，但 rollback 没有 deployDir，默认用 ~/apphome
            format!(
                "cd ~/apphome && chmod +x {} && bash -l -c '{}' 2>&1",
                restart_script, restart_script
            )
        };
        let mut channel = sess
            .channel_session()
            .map_err(|e| format!("创建 SSH channel 失败: {}", e))?;
        channel
            .exec(&exec_cmd)
            .map_err(|e| format!("执行重启命令失败: {}", e))?;

        // 收集输出
        let mut output = String::new();
        use std::io::Read;
        channel.read_to_string(&mut output).ok();
        channel.wait_close().ok();

        let exit_status = channel.exit_status().unwrap_or(-1);
        if exit_status != 0 {
            log::error!(
                "[rollback] restart failed (exit {}): {}",
                exit_status,
                output.trim()
            );
            return Err(format!(
                "重启脚本退出码 {}: {}",
                exit_status,
                output.trim().chars().take(200).collect::<String>()
            ));
        } else {
            log::info!("[rollback] restart success: {}", output.trim());
        }
        sess.disconnect(None, "", None).ok();

        Ok(())
    })
    .await
    .map_err(|e| format!("spawn_blocking 失败: {}", e))?
}

// =================== Log Commands ===================

#[tauri::command(rename_all = "camelCase")]
pub async fn get_deploy_logs(
    core: State<'_, CoreService>,
    config_id: String,
    limit: Option<i64>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_deploy_logs() called");
    let lim = limit.unwrap_or(50);
    core.db_read(|conn| {
        let mut stmt = conn
            .prepare("SELECT * FROM deploy_logs WHERE configId = ? ORDER BY createdAt DESC LIMIT ?")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(rusqlite::params![config_id, lim], row_to_deploy_log)
            .map_err(|e| e.to_string())?;
        let logs: Vec<DeployLog> = rows.filter_map(|r| r.ok()).collect();
        serde_json::to_value(&logs).map_err(|e| e.to_string())
    })?
}

#[tauri::command(rename_all = "camelCase")]
pub async fn read_log_file(file_path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] read_log_file() called");
    let content =
        std::fs::read_to_string(&file_path).map_err(|e| format!("读取日志文件失败: {}", e))?;
    Ok(serde_json::json!({
        "success": true,
        "content": content,
        "filePath": file_path,
    }))
}

/// HTML 转义：仅处理 &, <, > 三个最常见字符，避免日志内容触发 HTML 解析
fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

/// 检测日志级别并返回对应的 Tailwind 类名（仅扫前 200 字符，性能优先）
fn detect_level_class(line: &str) -> &'static str {
    if line.is_empty() {
        return "";
    }
    // 仅扫前 200 个字符（不是字节），避免在 UTF-8 多字节字符中间切片 panic
    let head: String = line.chars().take(200).collect();
    // 大小写不敏感：把 ASCII 字符转大写后再子串匹配，比 regex 快
    let head_upper = head.to_ascii_uppercase();
    if head_upper.contains("ERROR")
        || head_upper.contains("FATAL")
        || head_upper.contains("CRITICAL")
        || head_upper.contains("EXCEPTION")
    {
        return "text-red-400";
    }
    if head_upper.contains("WARN") || head_upper.contains("WARNING") {
        return "text-yellow-300";
    }
    if head_upper.contains("DEBUG") {
        return "text-white/50";
    }
    ""
}

/// 分页读取日志文件：后端预计算每行的 HTML（转义 + 级别色包装 + 关键字高亮），
/// 前端直接 v-html 渲染，避免在前端处理字符串性能瓶颈。
///
/// 参数：
/// - filePath: 本地文件绝对路径
/// - start: 起始行号（0-based，inclusive）
/// - count: 读取行数（建议 50-200）
/// - keyword: 可选关键字。非空时对返回的行做高亮（<mark> 包裹），不做过滤。
///   匹配行号列表由 find_log_matches 单独获取。
#[tauri::command(rename_all = "camelCase")]
pub fn read_log_file_lines(
    file_path: String,
    start: usize,
    count: usize,
    keyword: Option<String>,
) -> Result<serde_json::Value, String> {
    log::info!(
        "[Tauri CMD] read_log_file_lines() path={} start={} count={} keyword={:?}",
        file_path,
        start,
        count,
        keyword
    );

    if count == 0 {
        return Ok(serde_json::json!({
            "success": true,
            "data": {
                "totalLines": 0usize,
                "lines": Vec::<serde_json::Value>::new(),
                "start": 0usize,
                "end": 0usize,
            }
        }));
    }

    let file = std::fs::File::open(&file_path)
        .map_err(|e| format!("打开日志文件失败: {}", e))?;
    use std::io::BufRead;
    let reader = std::io::BufReader::new(file);

    // 准备关键字正则（仅用于高亮，不过滤）
    let kw = keyword
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());
    let highlight_re = if let Some(kw_str) = kw {
        let escaped = regex::escape(kw_str);
        regex::Regex::new(&format!("({})", escaped)).ok()
    } else {
        None
    };

    // 计算单行 HTML 的闭包
    let build_html = |line: &str, re: Option<&regex::Regex>| -> String {
        let escaped = html_escape(line);
        let highlighted = if let Some(re) = re {
            re.replace_all(&escaped, "<mark>$1</mark>").to_string()
        } else {
            escaped
        };
        let cls = detect_level_class(line);
        if cls.is_empty() {
            highlighted
        } else {
            format!("<span class=\"{}\">{}</span>", cls, highlighted)
        }
    };

    // 统一模式：按 [start, start+count) 区间读取，keyword 仅做高亮
    let mut total_lines: usize = 0;
    let end_exclusive = start.saturating_add(count);
    let mut cached_lines: Vec<String> = Vec::with_capacity(count);

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => {
                total_lines = total_lines.saturating_add(1);
                continue;
            }
        };
        let idx = total_lines;
        total_lines = total_lines.saturating_add(1);

        if idx >= start && idx < end_exclusive {
            cached_lines.push(line);
        }
    }

    let lines_json: Vec<serde_json::Value> = cached_lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            serde_json::json!({
                "lineNo": start + i,
                "html": build_html(line, highlight_re.as_ref()),
            })
        })
        .collect();

    Ok(serde_json::json!({
        "success": true,
        "data": {
            "totalLines": total_lines,
            "lines": lines_json,
            "start": start,
            "end": end_exclusive.min(total_lines),
        }
    }))
}

/// 查找日志文件中所有匹配关键字的行号（0-based）。
/// 用于 vim 式搜索：显示完整日志 + 高亮 + 上下跳转。
#[tauri::command(rename_all = "camelCase")]
pub fn find_log_matches(file_path: String, keyword: String) -> Result<serde_json::Value, String> {
    log::info!(
        "[Tauri CMD] find_log_matches() path={} keyword={:?}",
        file_path,
        keyword
    );

    let kw = keyword.trim();
    if kw.is_empty() {
        return Ok(serde_json::json!({
            "success": true,
            "data": { "matchLineNos": Vec::<usize>::new() }
        }));
    }

    let escaped = regex::escape(kw);
    let re = regex::Regex::new(&format!("({})", escaped))
        .map_err(|e| format!("正则编译失败: {}", e))?;

    let file = std::fs::File::open(&file_path)
        .map_err(|e| format!("打开日志文件失败: {}", e))?;
    use std::io::BufRead;
    let reader = std::io::BufReader::new(file);

    let mut match_line_nos: Vec<usize> = Vec::new();
    let mut line_no: usize = 0;
    for line_result in reader.lines() {
        match line_result {
            Ok(l) => {
                if re.is_match(&l) {
                    match_line_nos.push(line_no);
                }
            }
            Err(_) => {}
        }
        line_no = line_no.saturating_add(1);
    }

    Ok(serde_json::json!({
        "success": true,
        "data": { "matchLineNos": match_line_nos }
    }))
}

/// 解压本地 .gz 文件到同目录下的 .decompressed 文件
/// 用于离线日志查看：下载 .gz 后解压，再交给 read_log_file_lines 读取
/// 返回解压后的文件路径
#[tauri::command(rename_all = "camelCase")]
pub fn gunzip_local_file(gz_path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] gunzip_local_file() path={}", gz_path);

    let gz_file = std::fs::File::open(&gz_path)
        .map_err(|e| format!("打开 .gz 文件失败: {}", e))?;
    let decoder = flate2::read::GzDecoder::new(gz_file);
    use std::io::Read;

    // 解压到同目录下，文件名去掉 .gz 后缀，加 .decompressed 防止冲突
    let src_path = std::path::Path::new(&gz_path);
    let file_stem = src_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "decompressed".to_string());
    let parent = src_path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let decompressed_path = parent.join(format!("{}.decompressed", file_stem));

    let mut out_file = std::fs::File::create(&decompressed_path)
        .map_err(|e| format!("创建解压文件失败: {}", e))?;
    let mut buf = vec![0u8; 64 * 1024];
    let mut decoder = decoder;
    let mut total: u64 = 0;
    loop {
        let n = decoder
            .read(&mut buf)
            .map_err(|e| format!("解压读取失败: {}", e))?;
        if n == 0 {
            break;
        }
        std::io::Write::write_all(&mut out_file, &buf[..n])
            .map_err(|e| format!("解压写入失败: {}", e))?;
        total += n as u64;
    }

    log::info!(
        "[Tauri CMD] gunzip_local_file() decompressed {} -> {} ({} bytes)",
        gz_path,
        decompressed_path.display(),
        total
    );

    Ok(serde_json::json!({
        "success": true,
        "data": {
            "decompressedPath": decompressed_path.to_string_lossy().to_string(),
            "bytesWritten": total,
        }
    }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_rollback_history(
    core: State<'_, CoreService>,
    config_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_rollback_history() called");
    core.db_read(|conn| {
        let mut stmt = conn
            .prepare(
                "SELECT * FROM deploy_history WHERE configId = ? AND rolledBack = 1 ORDER BY deployedAt DESC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([config_id], row_to_deploy_history)
            .map_err(|e| e.to_string())?;
        let history: Vec<DeployHistory> = rows.filter_map(|r| r.ok()).collect();
        serde_json::to_value(&history).map_err(|e| e.to_string())
    })?
}

// =================== DB function aliases (avoid name collision with commands) ===================

fn cicd_get_all_configs(conn: &rusqlite::Connection) -> Result<Vec<CicdConfig>, String> {
    supertool_core::db::cicd::get_all_cicd_configs(conn).map_err(|e| e.to_string())
}
fn cicd_get_config_by_id(
    conn: &rusqlite::Connection,
    id: &str,
) -> Result<Option<CicdConfig>, String> {
    supertool_core::db::cicd::get_cicd_config_by_config_id(conn, id).map_err(|e| e.to_string())
}
fn cicd_get_groups(conn: &rusqlite::Connection) -> Result<Vec<String>, String> {
    supertool_core::db::cicd::get_cicd_groups(conn).map_err(|e| e.to_string())
}
fn cicd_add_config(conn: &rusqlite::Connection, c: &CicdConfig) -> Result<CicdConfig, String> {
    supertool_core::db::cicd::add_cicd_config(conn, c).map_err(|e| e.to_string())
}
fn cicd_update_config(
    conn: &rusqlite::Connection,
    c: &CicdConfig,
) -> Result<Option<CicdConfig>, String> {
    supertool_core::db::cicd::update_cicd_config(conn, c).map_err(|e| e.to_string())
}
fn cicd_delete_config(conn: &rusqlite::Connection, id: &str) -> Result<(), String> {
    supertool_core::db::cicd::delete_cicd_config(conn, id).map_err(|e| e.to_string())
}
fn cicd_get_modules(
    conn: &rusqlite::Connection,
    config_id: &str,
) -> Result<Vec<DeployModule>, String> {
    supertool_core::db::cicd::get_deploy_modules(conn, config_id).map_err(|e| e.to_string())
}
fn cicd_add_module(conn: &rusqlite::Connection, m: &DeployModule) -> Result<DeployModule, String> {
    supertool_core::db::cicd::add_deploy_module(conn, m).map_err(|e| e.to_string())
}
fn cicd_add_deploy_log(conn: &rusqlite::Connection, log: &DeployLog) -> Result<DeployLog, String> {
    supertool_core::db::cicd::add_deploy_log(conn, log).map_err(|e| e.to_string())
}
fn cicd_update_deploy_log(
    conn: &rusqlite::Connection,
    log: &DeployLog,
) -> Result<Option<DeployLog>, String> {
    supertool_core::db::cicd::update_deploy_log(conn, log).map_err(|e| e.to_string())
}
fn cicd_get_deploy_log_by_id(conn: &rusqlite::Connection, id: &str) -> Option<DeployLog> {
    supertool_core::db::cicd::get_deploy_log_by_id(conn, id)
        .map_err(|e| e.to_string())
        .ok()
        .flatten()
}
fn cicd_touch_deploy(conn: &rusqlite::Connection, id: &str) -> Result<(), String> {
    supertool_core::db::cicd::touch_cicd_config_deploy(conn, id).map_err(|e| e.to_string())
}
fn cicd_add_deploy_history(
    conn: &rusqlite::Connection,
    h: &DeployHistory,
) -> Result<DeployHistory, String> {
    supertool_core::db::cicd::add_deploy_history(conn, h).map_err(|e| e.to_string())
}

// =================== Helper Functions ===================

fn build_deploy_config(
    core: &supertool_core::logic::CoreService,
    cicd_config: &CicdConfig,
    modules: &[DeployModule],
) -> Result<DeployConfig, String> {
    // 解析服务器引用（DB 存的是 [{serverId, deployDir}]，需查 servers 表补全）
    let servers: Vec<DeployServerConfig> = if let Some(ref servers_str) = cicd_config.servers {
        #[derive(Deserialize)]
        struct ServerRef {
            #[serde(rename = "serverId")]
            server_id: String,
            #[serde(rename = "deployDir")]
            deploy_dir: String,
        }
        let refs: Vec<ServerRef> =
            serde_json::from_str(servers_str).map_err(|e| format!("解析服务器引用失败: {}", e))?;

        refs.into_iter()
            .map(|r| {
                // 直接查 servers 表 + 解密密码
                let server = core.db_read(|conn| {
                    conn.query_row(
                        "SELECT * FROM servers WHERE id = ?1",
                        rusqlite::params![r.server_id],
                        supertool_core::db::servers::row_to_server,
                    )
                    .map_err(|e| e.to_string())
                })??;
                // 密码已在 row_to_server 中解密 (servers.rs 的 get_server_by_id 调用 decrypt_password)
                // 但 row_to_server 不解密，需要手动解密
                let password = server
                    .password
                    .map(|pw| supertool_core::encryption::try_decrypt_password(&pw));
                let base_deploy_dir = if r.deploy_dir.is_empty() {
                    cicd_config.deploy_path.clone()
                } else {
                    r.deploy_dir
                };
                Ok(DeployServerConfig {
                    host: server.host,
                    port: server.port as u16,
                    username: server.username,
                    password,
                    private_key: server.ssh_key_path,
                    deploy_dir: base_deploy_dir.clone(),
                    lib_dir: if cicd_config.lib_separate {
                        Some(format!("{}/lib", base_deploy_dir))
                    } else {
                        None
                    },
                    label: Some(server.name),
                })
            })
            .collect::<Result<Vec<_>, String>>()?
    } else {
        vec![]
    };
    let module_configs: Vec<DeployModuleConfig> = modules
        .iter()
        .map(|m| DeployModuleConfig {
            name: Some(m.module_name.clone()),
            path: Some(m.module_path.clone()),
            build_path: m.build_path.clone(),
            build_command: m.build_command.clone(),
            build_tool: m.build_tool.clone(),
            output_path: m.output_path.clone(),
            artifact_name: Some(m.artifact_name.clone()),
            artifact_type: m.artifact_type.clone(),
            lib_filter_rules: m.lib_filter_rules.clone(),
            deploy_order: m.deploy_order,
            deploy_path: m.deploy_path.clone(),
            enabled: m.enabled,
        })
        .collect();
    Ok(DeployConfig {
        repo_url: cicd_config
            .git_repo_id
            .as_ref()
            .and_then(|id| {
                core.db_read(|conn| {
                    supertool_core::db::git_repo::get_by_id(conn, id)
                        .ok()
                        .flatten()
                })
                .ok()
                .flatten()
            })
            .and_then(|r| r.remote.or(Some(r.path)))
            .unwrap_or_default(),
        branch: cicd_config.deploy_branch.clone(),
        local_path: cicd_config
            .git_repo_id
            .as_ref()
            .and_then(|id| {
                core.db_read(|conn| {
                    supertool_core::db::git_repo::get_by_id(conn, id)
                        .ok()
                        .flatten()
                })
                .ok()
                .flatten()
            })
            .map(|r| r.path),
        build_tool: cicd_config.build_tool.clone(),
        build_command: cicd_config.build_command.clone(),
        build_path: cicd_config.build_path.clone(),
        npm_script: cicd_config.npm_script.clone(),
        npm_custom_script: cicd_config.npm_custom_script.clone(),
        maven_home: cicd_config.maven_home.clone(),
        java_home: cicd_config.java_home.clone(),
        npm_home: cicd_config.npm_home.clone(),
        node_home: cicd_config.node_home.clone(),
        maven_profile: Some(cicd_config.maven_profile.clone()),
        maven_settings: cicd_config.maven_settings.clone(),
        modules: module_configs,
        skip_tests: true,
        parent_build_mode: cicd_config.parent_build_mode,
        parent_build_path: if cicd_config.parent_build_path.is_empty() {
            None
        } else {
            Some(cicd_config.parent_build_path.clone())
        },
        servers,
        deploy_dir: cicd_config.deploy_path.clone(),
        lib_dir: if cicd_config.lib_separate && cicd_config.build_tool.as_deref() == Some("maven") {
            Some(format!("{}/lib", cicd_config.deploy_path))
        } else {
            None
        },
        restart_script: if cicd_config.restart_script.is_empty() {
            None
        } else {
            Some(cicd_config.restart_script.clone())
        },
        lib_separate: cicd_config.lib_separate
            && cicd_config.build_tool.as_deref() == Some("maven"),
        build_mode: cicd_config.build_mode.clone(),
    })
}

// =================== Module CRUD Commands ===================

#[tauri::command(rename_all = "camelCase")]
pub async fn save_deploy_module(
    core: State<'_, CoreService>,
    module: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] save_deploy_module() called");
    let now = chrono::Utc::now().to_rfc3339();
    let mut dm: DeployModule =
        serde_json::from_value(module.clone()).map_err(|e| format!("解析模块失败: {}", e))?;
    dm.created_at = now.clone();
    dm.updated_at = now.clone();
    let result = core.db_write(|conn| cicd_add_module(conn, &dm).expect("db error"))?;
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_deploy_module(
    core: State<'_, CoreService>,
    module: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_deploy_module() called");
    let now = chrono::Utc::now().to_rfc3339();
    let mut dm: DeployModule =
        serde_json::from_value(module.clone()).map_err(|e| format!("解析模块失败: {}", e))?;
    dm.updated_at = now.clone();
    let result = core.db_write(|conn| {
        supertool_core::db::cicd::update_deploy_module(conn, &dm).map_err(|e| e.to_string())
    })?;
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_deploy_module(
    core: State<'_, CoreService>,
    module_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_deploy_module() called");
    let _ = core.db_write(|conn| {
        supertool_core::db::cicd::delete_deploy_module(conn, &module_id).map_err(|e| e.to_string())
    })?;
    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn scan_project_modules(project_path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] scan_project_modules() called");
    let path = Path::new(&project_path);
    if !path.exists() {
        return Ok(serde_json::json!({ "success": false, "modules": [], "error": "路径不存在" }));
    }

    let mut modules = scan_maven_modules_recursive(path, path, 0);

    // If no Maven modules at root, scan up to 2 levels deeper for pom.xml
    // (e.g., pre-pay-service/SRC/yudao/pom.xml is 2 levels deep)
    if modules.is_empty() {
        log::info!("[scan_project_modules] No pom.xml at root, scanning subdirectories...");
        modules = scan_subdirs_for_maven(path, path, 2);
        log::info!(
            "[scan_project_modules] Found {} modules in subdirectories",
            modules.len()
        );
    }

    // If no Maven modules found, scan Node.js packages
    let modules = if modules.is_empty() {
        scan_npm_modules(path)
    } else {
        modules
    };

    // If still empty, try top-level package.json
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
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    } else {
        modules
    };

    Ok(serde_json::json!({ "success": true, "modules": modules }))
}

/// Scan subdirectories for pom.xml files (up to `max_depth` levels)
/// Returns modules with correct relative paths from the original root
fn scan_subdirs_for_maven(
    root_path: &Path,
    current_path: &Path,
    max_depth: u8,
) -> Vec<serde_json::Value> {
    if max_depth == 0 {
        return vec![];
    }
    let mut modules = Vec::new();
    let skip_dirs = [
        "node_modules",
        "target",
        "dist",
        ".git",
        ".idea",
        "doc",
        "docs",
    ];
    if let Ok(entries) = fs::read_dir(current_path) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if !ft.is_dir() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') || skip_dirs.contains(&name.as_str()) {
                    continue;
                }
                let sub_path = entry.path();
                if sub_path.join("pom.xml").exists() {
                    log::info!("[scan_subdirs] Found pom.xml at {}", sub_path.display());
                    let sub_modules = scan_maven_modules_recursive(root_path, &sub_path, 0);
                    if !sub_modules.is_empty() {
                        // Compute relative prefix from root to this subdirectory
                        let rel_prefix = sub_path
                            .strip_prefix(root_path)
                            .map(|p| format!("./{}", p.to_string_lossy()))
                            .unwrap_or_else(|_| format!("./{}", name));
                        let prefixed: Vec<serde_json::Value> = sub_modules
                            .into_iter()
                            .map(|mut m| {
                                if let Some(p) = m.get("path").and_then(|v| v.as_str()) {
                                    let full_path = if p == "." {
                                        rel_prefix.clone()
                                    } else {
                                        format!("{}/{}", rel_prefix, p.trim_start_matches("./"))
                                    };
                                    m["path"] = serde_json::json!(full_path);
                                }
                                m
                            })
                            .collect();
                        modules.extend(prefixed);
                    }
                } else {
                    // Go deeper
                    let deeper = scan_subdirs_for_maven(root_path, &sub_path, max_depth - 1);
                    modules.extend(deeper);
                }
            }
        }
    }
    modules
}

/// Recursively scan Maven modules from a pom.xml, up to 3 levels deep
fn scan_maven_modules_recursive(
    root_path: &Path,
    base_path: &Path,
    depth: u8,
) -> Vec<serde_json::Value> {
    if depth > 3 {
        return vec![];
    }

    let pom_path = base_path.join("pom.xml");
    if !pom_path.exists() {
        return vec![];
    }

    let pom = match fs::read_to_string(&pom_path) {
        Ok(content) => content,
        Err(_) => return vec![],
    };

    let mut modules: Vec<serde_json::Value> = Vec::new();

    // Extract <modules> section
    if let Ok(re) = regex::Regex::new(r"<modules>\s*([\s\S]*?)</modules>") {
        if let Some(cap) = re.captures(&pom) {
            let module_re = regex::Regex::new(r"<module>\s*([^<]+?)\s*</module>").unwrap();
            let child_names: Vec<String> = module_re
                .captures_iter(&cap[1])
                .map(|c| c[1].trim().to_string())
                .filter(|m| !m.is_empty())
                .collect();

            for mod_name in &child_names {
                // Resolve module path relative to base_path
                let mod_rel_path = if mod_name.starts_with("./") || mod_name.starts_with("../") {
                    mod_name.clone()
                } else {
                    format!("./{}", mod_name)
                };

                // Compute absolute path for recursive scanning
                let mod_abs_path = base_path.join(mod_name);

                // Extract artifactId from child pom.xml
                let artifact_id = if mod_abs_path.join("pom.xml").exists() {
                    if let Ok(child_pom) = fs::read_to_string(mod_abs_path.join("pom.xml")) {
                        regex::Regex::new(r"<artifactId>\s*([^<]+?)\s*</artifactId>")
                            .ok()
                            .and_then(|re| re.captures(&child_pom))
                            .map(|c| c[1].trim().to_string())
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Recursively scan for nested sub-modules
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

    // If no <modules> section, treat this pom as a single module (only at depth 0)
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

/// Scan sub-directories for Node.js packages with package.json
fn scan_npm_modules(base_path: &Path) -> Vec<serde_json::Value> {
    let mut modules: Vec<serde_json::Value> = Vec::new();
    let skip_dirs = ["node_modules", "target", "dist", ".git", "coverage"];

    if let Ok(entries) = fs::read_dir(base_path) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() {
                    let name = match entry.file_name().into_string() {
                        Ok(n) => n,
                        Err(_) => continue,
                    };
                    if skip_dirs.contains(&name.as_str()) || name.starts_with('.') {
                        continue;
                    }
                    let pkg_json = entry.path().join("package.json");
                    if pkg_json.exists() {
                        let mod_path = format!("./{}", name);
                        if !modules
                            .iter()
                            .any(|m| m.get("path").and_then(|p| p.as_str()) == Some(&mod_path))
                        {
                            if let Ok(content) = fs::read_to_string(&pkg_json) {
                                if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content)
                                {
                                    let pkg_name =
                                        pkg.get("name").and_then(|n| n.as_str()).unwrap_or(&name);
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
            }
        }
    }
    modules
}

// =================== Missing Tauri Commands ===================

#[tauri::command(rename_all = "camelCase")]
pub async fn get_deploy_modules(
    core: State<'_, CoreService>,
    config_id: String,
) -> Result<Vec<DeployModule>, String> {
    log::info!("[Tauri CMD] get_deploy_modules() called");
    core.db_read(|conn| cicd_get_modules(conn, &config_id).map_err(|e| e.to_string()))?
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_deploy_step_logs(
    core: State<'_, CoreService>,
    deploy_log_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_deploy_step_logs() called");
    let logs = core.db_read(|conn| {
        supertool_core::db::cicd::get_deploy_step_logs(conn, &deploy_log_id)
            .map_err(|e| e.to_string())
    })?;
    serde_json::to_value(&logs).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_deploy_history(
    core: State<'_, CoreService>,
    config_id: String,
    limit: Option<i64>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_deploy_history() called");
    let lim = limit.unwrap_or(50);
    let history = core.db_read(|conn| {
        let mut stmt = conn
            .prepare(
                "SELECT * FROM deploy_history WHERE configId = ? ORDER BY deployedAt DESC LIMIT ?",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(rusqlite::params![config_id, lim], row_to_deploy_history)
            .map_err(|e| e.to_string())?;
        let items: Vec<DeployHistory> = rows.filter_map(|r| r.ok()).collect();
        serde_json::to_value(&items).map_err(|e| e.to_string())
    })??;
    Ok(history)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_all_deploy_history(
    core: State<'_, CoreService>,
    limit: Option<i64>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_all_deploy_history() called");
    let lim = limit.unwrap_or(50);
    let history = core.db_read(|conn| {
        // JOIN cicd_configs to get config name
        let mut stmt = conn
            .prepare(
                "SELECT h.id, h.configId, c.name as configName, h.status, h.deployedAt, h.rolledBack, h.rolledBackAt \
                 FROM deploy_history h \
                 LEFT JOIN cicd_configs c ON h.configId = c.id \
                 ORDER BY h.deployedAt DESC LIMIT ?"
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(rusqlite::params![lim], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>("id")?,
                    "configId": row.get::<_, String>("configId")?,
                    "configName": row.get::<_, Option<String>>("configName")?,
                    "status": row.get::<_, String>("status")?,
                    "deployedAt": row.get::<_, String>("deployedAt")?,
                    "rolledBack": row.get::<_, i64>("rolledBack")? != 0,
                    "rolledBackAt": row.get::<_, Option<String>>("rolledBackAt")?,
                }))
            })
            .map_err(|e| e.to_string())?;
        let items: Vec<serde_json::Value> = rows.filter_map(|r| r.ok()).collect();
        serde_json::to_value(&items).map_err(|e| e.to_string())
    })??;
    Ok(history)
}
