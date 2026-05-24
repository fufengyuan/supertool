use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Get hermes CLI path (supports pipx install and direct install)
fn get_hermes_path() -> String {
    // Try common locations in order
    let candidates = [
        "/usr/local/bin/hermes",
        "~/.local/bin/hermes",
        "~/.hermes/hermes-agent/.venv/bin/hermes",
    ];

    for candidate in candidates {
        let path = if candidate.starts_with('~') {
            dirs::home_dir()
                .map(|h| h.join(candidate.replace('~', "")))
                .unwrap_or_else(|| std::path::PathBuf::from(candidate))
        } else {
            std::path::PathBuf::from(candidate)
        };

        if path.exists() {
            return path.to_string_lossy().to_string();
        }
    }

    // Fallback to just "hermes" (will use PATH)
    "hermes".to_string()
}

/// Software development team profile configuration
#[derive(Debug, Deserialize)]
pub struct ProfileConfig {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub skills: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct InstallResult {
    pub cli_installed: bool,
    pub cli_path: String,
    pub skills_installed: bool,
    pub skills_path: String,
    pub profiles_installed: bool,
    pub profiles_count: u32,
}

/// Get the bundled stool binary path from Tauri resources
fn get_bundled_stool_path() -> Option<PathBuf> {
    // In Tauri, resources are at <app>/Contents/Resources/ (macOS) or same dir as binary (Linux/Windows)
    let exe_path = std::env::current_exe().ok()?;
    let exe_dir = exe_path.parent()?.parent()?;
    let stool = exe_dir.join("stool");
    if stool.exists() {
        Some(stool)
    } else {
        // Try direct resources path
        let resources = exe_dir.join("resources").join("stool");
        if resources.exists() {
            Some(resources)
        } else {
            None
        }
    }
}

/// Get the bundled skills directory path from Tauri resources
fn get_bundled_skills_path() -> Option<PathBuf> {
    let exe_path = std::env::current_exe().ok()?;
    let exe_dir = exe_path.parent()?.parent()?;
    let skills = exe_dir.join("skills");
    if skills.exists() {
        Some(skills)
    } else {
        let resources = exe_dir.join("resources").join("skills");
        if resources.exists() {
            Some(resources)
        } else {
            None
        }
    }
}

/// Install stool CLI binary to /usr/local/bin/stool (or fallback to ~/.local/bin/)
pub fn install_stool_cli() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    return Err("CLI installation not supported on Windows".to_string());

    let source = get_bundled_stool_path()
        .ok_or_else(|| "stool binary not found in resources".to_string())?;

    let target = PathBuf::from("/usr/local/bin/stool");

    // Check if source is newer than target (mtime comparison)
    if target.exists() {
        if let (Ok(src_meta), Ok(tgt_meta)) = (fs::metadata(&source), fs::metadata(&target)) {
            if let (Ok(src_mtime), Ok(tgt_mtime)) = (src_meta.modified(), tgt_meta.modified()) {
                if src_mtime <= tgt_mtime {
                    return Ok("/usr/local/bin/stool".to_string());
                }
            }
        }
    }

    // Try sudo install first
    let safe_src = source.to_string_lossy().replace("'", "'\\''");
    let safe_dst = target.to_string_lossy().replace("'", "'\\''");
    let cmd = format!(
        "sudo cp '{}' '{}' && chmod +x '{}'",
        safe_src, safe_dst, safe_dst
    );
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .output()
        .map_err(|e| format!("sudo install failed: {}", e))?;

    if output.status.success() {
        return Ok("/usr/local/bin/stool".to_string());
    }

    // Fallback to ~/.local/bin/
    if let Some(home) = dirs::home_dir() {
        let fallback = home.join(".local").join("bin").join("stool");
        fs::create_dir_all(fallback.parent().unwrap())
            .map_err(|e| format!("Failed to create dir: {}", e))?;
        fs::copy(&source, &fallback).map_err(|e| format!("Failed to copy: {}", e))?;
        // Try chmod, ignore errors
        let _ = std::process::Command::new("chmod")
            .arg("+x")
            .arg(&fallback)
            .output();
        return Ok(fallback.to_string_lossy().to_string());
    }

    Err("Failed to install stool CLI".to_string())
}

/// Get the bundled profiles directory path from Tauri resources
fn get_bundled_profiles_path() -> Option<PathBuf> {
    let exe_path = std::env::current_exe().ok()?;
    let exe_dir = exe_path.parent()?.parent()?;
    let profiles = exe_dir.join("profiles");
    if profiles.exists() {
        Some(profiles)
    } else {
        let resources = exe_dir.join("resources").join("profiles");
        if resources.exists() {
            Some(resources)
        } else {
            None
        }
    }
}

/// Install Hermes profiles from bundled configuration
/// Uses hermes CLI to create profiles with descriptions
pub fn install_hermes_profiles() -> Result<u32, String> {
    let profiles_dir = get_bundled_profiles_path()
        .ok_or_else(|| "profiles directory not found in resources".to_string())?;

    let config_file = profiles_dir.join("software-dev-team.json");
    if !config_file.exists() {
        return Ok(0);
    }

    let content = fs::read_to_string(&config_file)
        .map_err(|e| format!("Failed to read profiles config: {}", e))?;

    let profiles: Vec<ProfileConfig> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse profiles config: {}", e))?;

    let mut installed = 0u32;

    for profile in profiles {
        // Check if profile already exists
        let check_output = Command::new(&get_hermes_path())
            .args(["profile", "list"])
            .output()
            .map_err(|e| format!("Failed to check profiles: {}", e))?;

        let list_output = String::from_utf8_lossy(&check_output.stdout);
        if list_output.contains(&profile.name) {
            // Profile exists, just update description
            let _ = Command::new(&get_hermes_path())
                .args(["profile", "describe", &profile.name, "--set", &profile.description])
                .output();
            installed += 1;
            continue;
        }

        // Create new profile
        let create_result = Command::new(&get_hermes_path())
            .args(["profile", "create", &profile.name])
            .output()
            .map_err(|e| format!("Failed to create profile {}: {}", profile.name, e))?;

        if !create_result.status.success() {
            log::warn!("Failed to create profile {}: {}", profile.name, String::from_utf8_lossy(&create_result.stderr));
            continue;
        }

        // Set description
        let _ = Command::new(&get_hermes_path())
            .args(["profile", "describe", &profile.name, "--set", &profile.description])
            .output();

        // Set model if specified
        if let Some(model) = &profile.model {
            let _ = Command::new(&get_hermes_path())
                .args(["profile", "model", &profile.name, "--set", model])
                .output();
        }

        installed += 1;
    }

    Ok(installed)
}

/// Install Hermes skills from bundled resources to ~/.hermes/skills/
pub fn install_hermes_skills() -> Result<String, String> {
    let source_dir = get_bundled_skills_path()
        .ok_or_else(|| "skills directory not found in resources".to_string())?;

    let skills_root = dirs::home_dir()
        .map(|h| h.join(".hermes").join("skills"))
        .ok_or_else(|| "Cannot determine home directory".to_string())?;

    let mut installed = Vec::new();

    for entry in
        fs::read_dir(&source_dir).map_err(|e| format!("Failed to read skills dir: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let skill_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if skill_name.is_empty() {
            continue;
        }

        let skill_src = path.join("SKILL.md");
        if !skill_src.exists() {
            continue;
        }

        let target_dir = skills_root.join(&skill_name);
        let target_file = target_dir.join("SKILL.md");

        // mtime comparison - only update if source is newer
        if target_file.exists() {
            if let (Ok(src_meta), Ok(tgt_meta)) =
                (fs::metadata(&skill_src), fs::metadata(&target_file))
            {
                if let (Ok(src_mtime), Ok(tgt_mtime)) = (src_meta.modified(), tgt_meta.modified()) {
                    if src_mtime <= tgt_mtime {
                        continue; // Already up-to-date
                    }
                }
            }
        }

        fs::create_dir_all(&target_dir)
            .map_err(|e| format!("Failed to create skill dir: {}", e))?;

        // fs::copy preserves metadata on some platforms, but we explicitly preserve mtime
        // Simple approach: just copy, next launch will compare
        let content = fs::read_to_string(&skill_src)
            .map_err(|e| format!("Failed to read skill file: {}", e))?;
        fs::write(&target_file, content)
            .map_err(|e| format!("Failed to write skill file: {}", e))?;

        installed.push(skill_name);
    }

    Ok(format!(
        "Installed {} skills: {}",
        installed.len(),
        installed.join(", ")
    ))
}

#[tauri::command(rename_all = "camelCase")]
pub fn install_cli_and_skills() -> InstallResult {
    let cli_result = install_stool_cli();
    let skills_result = install_hermes_skills();
    let profiles_result = install_hermes_profiles();

    InstallResult {
        cli_installed: cli_result.is_ok(),
        cli_path: cli_result.unwrap_or_default(),
        skills_installed: skills_result.is_ok(),
        skills_path: skills_result.unwrap_or_default(),
        profiles_installed: profiles_result.is_ok(),
        profiles_count: profiles_result.unwrap_or(0),
    }
}

#[tauri::command(rename_all = "camelCase")]
pub fn check_cli_installed() -> InstallResult {
    let cli_path = "/usr/local/bin/stool";
    let cli_installed = std::path::Path::new(cli_path).exists()
        || dirs::home_dir()
            .map(|h| h.join(".local").join("bin").join("stool").exists())
            .unwrap_or(false);

    let home = dirs::home_dir();
    let skills_path = home
        .as_ref()
        .map(|h| {
            h.join(".hermes")
                .join("skills")
                .join("stool-cli")
                .join("SKILL.md")
        })
        .unwrap_or_default();
    let skills_installed = skills_path.exists();

    let cli_path_str = if std::path::Path::new(cli_path).exists() {
        cli_path.to_string()
    } else {
        home.map(|h| {
            h.join(".local")
                .join("bin")
                .join("stool")
                .to_string_lossy()
                .to_string()
        })
        .unwrap_or_default()
    };

    // Check profiles installed by counting software dev team profiles
    let profiles_installed = Command::new(&get_hermes_path())
        .args(["profile", "list"])
        .output()
        .map(|o| {
            let output = String::from_utf8_lossy(&o.stdout);
            let dev_profiles = ["coder", "reviewer", "tester", "researcher", "writer", "devops", "debugger"];
            dev_profiles.iter().filter(|p| output.contains(*p)).count() as u32
        })
        .unwrap_or(0);

    InstallResult {
        cli_installed,
        cli_path: cli_path_str,
        skills_installed,
        skills_path: skills_path.to_string_lossy().to_string(),
        profiles_installed: profiles_installed > 0,
        profiles_count: profiles_installed,
    }
}
