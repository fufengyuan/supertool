/// Data directory resolution utility
///
/// Reads ~/.supertool_dir for custom path, falls back to ~/.supertool
/// This module exists to avoid hardcoding the data directory in multiple places.
use std::path::PathBuf;

/// Resolve the data directory path.
/// Checks ~/.supertool_dir for a custom path first, then falls back to ~/.supertool
pub fn resolve_data_dir() -> PathBuf {
    let home_dir = match dirs::home_dir() {
        Some(h) => h,
        None => return PathBuf::from(".supertool"),
    };

    let config_file = home_dir.join(".supertool_dir");
    if config_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&config_file) {
            let custom_path = content.trim();
            if !custom_path.is_empty() {
                return PathBuf::from(custom_path);
            }
        }
    }

    home_dir.join(".supertool")
}

/// Get the path to the encryption key file
pub fn encryption_key_path() -> PathBuf {
    resolve_data_dir().join(".encryption_key")
}

/// Get the path to the UDS socket file
pub fn uds_socket_path() -> PathBuf {
    resolve_data_dir().join("supertool.sock")
}

/// Get the tmp directory path
pub fn tmp_dir() -> PathBuf {
    resolve_data_dir().join("tmp")
}

/// Get the received files directory (LAN)
pub fn received_files_dir() -> PathBuf {
    resolve_data_dir().join("received_files")
}

/// Get the LAN temp directory
pub fn lan_temp_dir() -> PathBuf {
    resolve_data_dir().join("lan_temp")
}

/// Get the CI/CD workspace directory
pub fn cicd_workspace_dir() -> PathBuf {
    resolve_data_dir().join("cicd-workspace")
}
