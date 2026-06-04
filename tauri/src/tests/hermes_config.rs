//! IPC-style tests for hermes_config commands.

use crate::commands::hermes_config::*;
use std::sync::Mutex;
use tauri::{
    ipc::{CallbackFn, InvokeBody},
    test::{get_ipc_response, mock_builder, mock_context, noop_assets},
    webview::InvokeRequest,
};
use serde_json::json;
    /// Global lock to prevent concurrent temp config writes
static CONFIG_LOCK: Mutex<()> = Mutex::new(());
    /// Helper: run a closure with a temporary ~/.hermes/config.yaml.
    /// Saves the original file if it exists and restores it afterward.
    /// Recover from mutex poisoning so a single test panic doesn't break all subsequent tests.
fn with_temp_config<F>(yaml_content: &str, f: F)
where
    F: FnOnce()
{
    let _lock = CONFIG_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let config_path = dirs::home_dir().unwrap().join(".hermes").join("config.yaml");
    let hermes_dir = config_path.parent().unwrap();
        // Ensure .hermes directory exists
    std::fs::create_dir_all(hermes_dir).ok();
        // Save original if exists
    let original = if config_path.exists() {
        Some(std::fs::read_to_string(&config_path).unwrap_or_default())
    } else {
        None
    };
        // Write temp config
    std::fs::write(&config_path, yaml_content).expect("Failed to write temp config");
        // Run the test
    f();
        // Restore original
    if let Some(orig) = original {
        std::fs::write(&config_path, &orig).expect("Failed to restore config");
    } else {
        std::fs::remove_file(&config_path).ok();
    }
}
    // ══════════════════════════════════════════════════════════════════════════
    // IPC test helpers — tauri::test mock app + invoke_ipc wrapper
    // ══════════════════════════════════════════════════════════════════════════
fn build_test_app() -> (
    tauri::App<tauri::test::MockRuntime>,
    tauri::WebviewWindow<tauri::test::MockRuntime>,
) {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            crate::commands::hermes_config::agent_api_server_status,
            crate::commands::hermes_config::agent_configure_api_server,
            crate::commands::hermes_config::list_toolsets,
            crate::commands::hermes_config::set_toolset_enabled,
            crate::commands::hermes_config::list_mcp_servers,
            crate::commands::hermes_config::get_hermes_config_info,
            crate::commands::hermes_config::export_hermes_config,
            crate::commands::hermes_config::import_hermes_config,
            crate::commands::hermes_config::hermes_set_config,
        ])
        .build(mock_context(noop_assets()))
        .expect("mock app should build");
    let ww = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("webview window should build");
    (app, ww)
}
    /// Invoke an IPC command and deserialize the result.
    /// Returns `Err` if the IPC call itself failed (command returned `Err`, or
    /// deserialization failed).
fn invoke_ipc<R: serde::de::DeserializeOwned>(
    webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
    cmd: &str,
    body: serde_json::Value,
) -> Result<R, String> {
    let res = get_ipc_response(
        webview,
        InvokeRequest {
            cmd: cmd.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: InvokeBody::Json(body),
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.to_string(),
        },
    );
    match res {
        Ok(response) => response
            .deserialize::<R>()
            .map_err(|e| format!("deserialize error: {e:?}")),
        Err(e) => Err(format!("IPC error: {e:?}")),
    }
}
#[test]
fn test_list_toolsets_all_enabled_by_default() {
        // When config has no platform_toolsets section, all 16 toolsets should be enabled
    with_temp_config("model:\n  default: gpt-4\n", || {
        let result = list_toolsets();
        assert!(result.is_ok());
        let json = result.unwrap();
        let toolsets = json["toolsets"].as_array().unwrap();
        assert_eq!(toolsets.len(), 16);
        for ts in toolsets {
            assert!(
                ts["enabled"].as_bool().unwrap(),
                "{} should be enabled by default when no platform_toolsets.cli exists",
                ts["key"].as_str().unwrap()
            );
        }
    });
}
#[test]
fn test_list_toolsets_platform_no_cli_all_enabled() {
        // When platform_toolsets exists but has no cli key, all toolsets should be enabled
    with_temp_config("platform_toolsets:\n  gui:\n    - web\n", || {
        let result = list_toolsets();
        assert!(result.is_ok());
        let json = result.unwrap();
        let toolsets = json["toolsets"].as_array().unwrap();
        assert_eq!(toolsets.len(), 16);
        for ts in toolsets {
            assert!(
                ts["enabled"].as_bool().unwrap(),
                "{} should be enabled when platform_toolsets has no cli key",
                ts["key"].as_str().unwrap()
            );
        }
    });
}
#[test]
fn test_list_toolsets_empty_cli_all_enabled() {
        // When platform_toolsets.cli is an empty list, all toolsets should be enabled
    with_temp_config("platform_toolsets:\n  cli:\n", || {
        let result = list_toolsets();
        assert!(result.is_ok());
        let json = result.unwrap();
        let toolsets = json["toolsets"].as_array().unwrap();
        assert_eq!(toolsets.len(), 16);
        for ts in toolsets {
            assert!(
                ts["enabled"].as_bool().unwrap(),
                "{} should be enabled when cli list is empty",
                ts["key"].as_str().unwrap()
            );
        }
    });
}
#[test]
fn test_list_toolsets_with_cli_list() {
        // When platform_toolsets.cli has specific keys, only those should be enabled
    with_temp_config(
        "platform_toolsets:\n  cli:\n    - web\n    - terminal\n    - file\n",
        || {
            let result = list_toolsets();
            assert!(result.is_ok());
            let json = result.unwrap();
            let toolsets = json["toolsets"].as_array().unwrap();
            assert_eq!(toolsets.len(), 16);
            for ts in toolsets {
                let key = ts["key"].as_str().unwrap();
                let enabled = ts["enabled"].as_bool().unwrap();
                match key {
                    "web" | "terminal" | "file" => assert!(enabled, "{} should be enabled", key),
                    _ => assert!(!enabled, "{} should be disabled", key),
                }
            }
        },
    );
}
#[test]
fn test_set_toolset_enabled_adds_key() {
    with_temp_config("platform_toolsets:\n  cli:\n    - web\n", || {
        let result = set_toolset_enabled("terminal".to_string(), true);
        assert!(result.is_ok());
            // Verify the config was updated
        let json = list_toolsets().unwrap();
        let toolsets = json["toolsets"].as_array().unwrap();
            // terminal should now be enabled
        let terminal = toolsets.iter().find(|t| t["key"].as_str() == Some("terminal")).unwrap();
        assert!(terminal["enabled"].as_bool().unwrap());
            // web should still be enabled
        let web = toolsets.iter().find(|t| t["key"].as_str() == Some("web")).unwrap();
        assert!(web["enabled"].as_bool().unwrap());
    });
}
#[test]
fn test_set_toolset_enabled_removes_key() {
    with_temp_config("platform_toolsets:\n  cli:\n    - web\n    - terminal\n", || {
        let result = set_toolset_enabled("terminal".to_string(), false);
        assert!(result.is_ok());
            // Verify the config was updated
        let json = list_toolsets().unwrap();
        let toolsets = json["toolsets"].as_array().unwrap();
            // terminal should now be disabled
        let terminal = toolsets.iter().find(|t| t["key"].as_str() == Some("terminal")).unwrap();
        assert!(!terminal["enabled"].as_bool().unwrap());
            // web should still be enabled
        let web = toolsets.iter().find(|t| t["key"].as_str() == Some("web")).unwrap();
        assert!(web["enabled"].as_bool().unwrap());
    });
}
#[test]
fn test_set_toolset_enabled_creates_cli_list_when_missing() {
    with_temp_config("model:\n  default: gpt-4\n", || {
            // Initially all enabled
        let json = list_toolsets().unwrap();
        for ts in json["toolsets"].as_array().unwrap() {
            assert!(ts["enabled"].as_bool().unwrap());
        }
            // Disable one toolset - this should create the cli list
        let result = set_toolset_enabled("web".to_string(), false);
        assert!(result.is_ok());
            // Now only web should be disabled
        let json2 = list_toolsets().unwrap();
        for ts in json2["toolsets"].as_array().unwrap() {
            let key = ts["key"].as_str().unwrap();
            let enabled = ts["enabled"].as_bool().unwrap();
            if key == "web" {
                assert!(!enabled, "web should be disabled");
            } else {
                assert!(enabled, "{} should still be enabled", key);
            }
        }
    });
}
#[test]
fn test_set_toolset_enabled_idempotent_toggle() {
    with_temp_config("platform_toolsets:\n  cli:\n    - web\n", || {
            // Enable already-enabled toolset - should be no-op
        let r1 = set_toolset_enabled("web".to_string(), true);
        assert!(r1.is_ok());
        let json = list_toolsets().unwrap();
        let web = json["toolsets"].as_array().unwrap().iter()
            .find(|t| t["key"].as_str() == Some("web")).unwrap();
        assert!(web["enabled"].as_bool().unwrap());
            // Disable already-disabled toolset - should be no-op
        let r2 = set_toolset_enabled("browser".to_string(), false);
        assert!(r2.is_ok());
        let json2 = list_toolsets().unwrap();
        let browser = json2["toolsets"].as_array().unwrap().iter()
            .find(|t| t["key"].as_str() == Some("browser")).unwrap();
        assert!(!browser["enabled"].as_bool().unwrap());
    });
}
#[test]
fn test_list_mcp_servers_empty() {
    with_temp_config("model:\n  default: gpt-4\n", || {
        let result = list_mcp_servers();
        assert!(result.is_ok());
        let json = result.unwrap();
        let servers = json["mcp_servers"].as_array().unwrap();
        assert!(servers.is_empty(), "No MCP servers should be listed when not configured");
    });
}
#[test]
fn test_list_mcp_servers_with_stdio_server() {
    with_temp_config(
        "mcp_servers:\n  time:\n    command: uvx\n    args:\n      - mcp-server-time\n",
        || {
            let result = list_mcp_servers();
            assert!(result.is_ok());
            let json = result.unwrap();
            let servers = json["mcp_servers"].as_array().unwrap();
            assert_eq!(servers.len(), 1);
            let server = &servers[0];
            assert_eq!(server["name"], "time");
            assert_eq!(server["type"], "stdio");
            assert!(server["detail"].as_str().unwrap().contains("uvx"));
            assert!(server["detail"].as_str().unwrap().contains("mcp-server-time"));
        },
    );
}
#[test]
fn test_list_mcp_servers_with_http_server() {
    with_temp_config(
        "mcp_servers:\n  my-api:\n    url: http://localhost:8080/api\n",
        || {
            let result = list_mcp_servers();
            assert!(result.is_ok());
            let json = result.unwrap();
            let servers = json["mcp_servers"].as_array().unwrap();
            assert_eq!(servers.len(), 1);
            let server = &servers[0];
            assert_eq!(server["name"], "my-api");
            assert_eq!(server["type"], "http");
            assert_eq!(server["detail"], "http://localhost:8080/api");
        },
    );
}
#[test]
fn test_list_mcp_servers_multiple_types() {
    with_temp_config(
        "mcp_servers:\n  time:\n    command: uvx\n    args:\n      - mcp-server-time\n  my-api:\n    url: https://api.example.com/mcp\n",
        || {
            let result = list_mcp_servers();
            assert!(result.is_ok());
            let json = result.unwrap();
            let servers = json["mcp_servers"].as_array().unwrap();
            assert_eq!(servers.len(), 2);
            let time_server = servers.iter().find(|s| s["name"] == "time").unwrap();
            assert_eq!(time_server["type"], "stdio");
            let api_server = servers.iter().find(|s| s["name"] == "my-api").unwrap();
            assert_eq!(api_server["type"], "http");
        },
    );
}
#[test]
fn test_read_enabled_toolsets_from_value_some() {
    let yaml: serde_yaml::Value = serde_yaml::from_str(
        "platform_toolsets:\n  cli:\n    - web\n    - terminal\n"
    ).unwrap();
    let result = read_enabled_toolsets_from_value(&yaml);
    assert!(result.is_some());
    let keys = result.unwrap();
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&"web".to_string()));
    assert!(keys.contains(&"terminal".to_string()));
}
#[test]
fn test_read_enabled_toolsets_from_value_no_platform() {
    let yaml: serde_yaml::Value = serde_yaml::from_str(
        "model:\n  default: gpt-4\n"
    ).unwrap();
    let result = read_enabled_toolsets_from_value(&yaml);
    assert!(result.is_none());
}
#[test]
fn test_read_enabled_toolsets_from_value_no_cli() {
    let yaml: serde_yaml::Value = serde_yaml::from_str(
        "platform_toolsets:\n  gui:\n    - web\n"
    ).unwrap();
    let result = read_enabled_toolsets_from_value(&yaml);
    assert!(result.is_none());
}
#[test]
fn test_read_enabled_toolsets_from_value_empty_cli() {
    let yaml: serde_yaml::Value = serde_yaml::from_str(
        "platform_toolsets:\n  cli:\n"
    ).unwrap();
    let result = read_enabled_toolsets_from_value(&yaml);
    assert!(result.is_none());
}
#[test]
fn test_toolset_keys_and_labels() {
        // Verify all 16 toolsets have correct keys and labels
    with_temp_config("model:\n  default: gpt-4\n", || {
        let result = list_toolsets().unwrap();
        let toolsets = result["toolsets"].as_array().unwrap();
        assert_eq!(toolsets.len(), 16);
        let expected: Vec<(&str, &str)> = vec![
            ("web", "Web"),
            ("browser", "Browser"),
            ("terminal", "Terminal"),
            ("file", "File"),
            ("code_execution", "Code Execution"),
            ("vision", "Vision"),
            ("image_gen", "Image Gen"),
            ("tts", "TTS"),
            ("skills", "Skills"),
            ("memory", "Memory"),
            ("session_search", "Session Search"),
            ("clarify", "Clarify"),
            ("delegation", "Delegation"),
            ("cronjob", "Cron Job"),
            ("moa", "MOA"),
            ("todo", "Todo"),
        ];
        for (i, ts) in toolsets.iter().enumerate() {
            assert_eq!(ts["key"].as_str().unwrap(), expected[i].0);
            assert_eq!(ts["label"].as_str().unwrap(), expected[i].1);
        }
    });
}
#[test]
fn test_list_toolsets_all_toolsets_have_descriptions() {
    with_temp_config("model:\n  default: gpt-4\n", || {
        let result = list_toolsets().unwrap();
        let toolsets = result["toolsets"].as_array().unwrap();
        for ts in toolsets {
            let desc = ts["description"].as_str().unwrap();
            assert!(!desc.is_empty(), "Toolset {} has empty description", ts["key"].as_str().unwrap());
        }
    });
}
#[test]
fn test_list_toolsets_no_config_file() {
        // Test that list_toolsets handles the case when config file doesn't exist.
        // Uses with_temp_config to ensure proper locking so it doesn't race with other tests.
        // We write then remove because with_temp_config needs a file to clean up after.
    with_temp_config("", || {
        let real_path = config_path();
            // Remove the empty file that with_temp_config just wrote
        std::fs::remove_file(&real_path).ok();
        let result = list_toolsets();
        assert!(result.is_ok());
        let json = result.unwrap();
        let toolsets = json["toolsets"].as_array().unwrap();
        assert_eq!(toolsets.len(), 16);
        for ts in toolsets {
            assert!(
                ts["enabled"].as_bool().unwrap(),
                "{} should be enabled when config.yaml doesn't exist",
                ts["key"].as_str().unwrap()
            );
        }
    });
}
#[test]
fn test_set_toolset_enabled_creates_config_from_empty() {
        // When config is empty, toggling should create the structure correctly.
        // We test via with_temp_config("") which simulates empty file.
    with_temp_config("", || {
            // Enable "web" on an empty config
        let result = set_toolset_enabled("web".to_string(), true);
        assert!(result.is_ok());
            // Verify the config file was updated with "web" behind the lock
        let json = list_toolsets().unwrap();
        let web = json["toolsets"].as_array().unwrap().iter()
            .find(|t| t["key"].as_str() == Some("web"))
            .unwrap();
        assert!(web["enabled"].as_bool().unwrap(), "web should be enabled");
            // Verify the raw YAML contains "web"
        let path = config_path();
        let content = std::fs::read_to_string(&path).expect("config should exist after toggle");
        assert!(content.contains("web"), "Config file should contain the web toolset key");
    });
}
#[test]
fn test_set_toolset_enabled_unknown_key() {
        // Unknown toolset key should still be added/removed without error
    with_temp_config("platform_toolsets:\n  cli:\n    - web\n    - terminal\n", || {
            // Enable an unknown key
        let result = set_toolset_enabled("unknown_tool".to_string(), true);
        assert!(result.is_ok());
            // Verify it appears in the list
        let json = list_toolsets().unwrap();
        let toolsets = json["toolsets"].as_array().unwrap();
            // The unknown key won't appear in toolsets output (only 16 known keys),
            // but it should be in the YAML file. Verify the file.
        let path = config_path();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("unknown_tool"), "unknown key should be persisted");
            // Disable the unknown key
        let result2 = set_toolset_enabled("unknown_tool".to_string(), false);
        assert!(result2.is_ok());
    });
}
#[test]
fn test_set_toolset_enabled_preserves_other_config_sections() {
        // Toggling a toolset should not touch unrelated sections like `model` or `mcp_servers`
    with_temp_config(
        "model:\n  default: gpt-4\nmcp_servers:\n  time:\n    command: uvx\nplatform_toolsets:\n  cli:\n    - web\n",
        || {
            let result = set_toolset_enabled("terminal".to_string(), true);
            assert!(result.is_ok());
                // Re-read config and verify model section is intact
            let path = config_path();
            let content = std::fs::read_to_string(&path).unwrap();
            assert!(content.contains("gpt-4"), "model.default should be preserved");
            assert!(content.contains("uvx"), "mcp_servers.time.command should be preserved");
        },
    );
}
#[test]
fn test_set_toolset_enabled_disables_all_but_one() {
        // Boundary: disable 15 toolsets, keeping only 1 enabled.
        // When the cli list is fully emptied, all default to enabled per design.
    with_temp_config("model:\n  default: gpt-4\n", || {
            // Disable 15 toolsets, leaving 'web' as the last enabled
        let all_except_web: Vec<&str> = ALL_TOOLSETS.iter()
            .filter(|(k, _, _)| *k != "web")
            .map(|(k, _, _)| *k)
            .collect();
        for key in &all_except_web {
            let result = set_toolset_enabled(key.to_string(), false);
            assert!(result.is_ok(), "Failed to disable toolset {}", key);
        }
            // Verify only 'web' is enabled
        let json = list_toolsets().unwrap();
        for ts in json["toolsets"].as_array().unwrap() {
            let key = ts["key"].as_str().unwrap();
            let enabled = ts["enabled"].as_bool().unwrap();
            if key == "web" {
                assert!(enabled, "web should be enabled");
            } else {
                assert!(!enabled, "{} should be disabled", key);
            }
        }
    });
}
#[test]
fn test_list_mcp_servers_both_url_and_command() {
        // When an MCP server has both url and command, url takes priority → http type
    with_temp_config(
        "mcp_servers:\n  hybrid:\n    url: http://localhost:8080\n    command: python\n    args:\n      - server.py\n",
        || {
            let result = list_mcp_servers();
            assert!(result.is_ok());
            let json = result.unwrap();
            let servers = json["mcp_servers"].as_array().unwrap();
            assert_eq!(servers.len(), 1);
            let server = &servers[0];
            assert_eq!(server["name"], "hybrid");
            assert_eq!(server["type"], "http", "url should take priority over command");
            assert_eq!(server["detail"], "http://localhost:8080");
        },
    );
}
#[test]
fn test_list_mcp_servers_without_url_or_command() {
        // MCP server entry with neither url nor command → silently skipped
    with_temp_config(
        "mcp_servers:\n  broken:\n    config_file: /path/to/config.json\n",
        || {
            let result = list_mcp_servers();
            assert!(result.is_ok());
            let json = result.unwrap();
            let servers = json["mcp_servers"].as_array().unwrap();
            assert!(servers.is_empty(), "Broken MCP entry should be skipped");
        },
    );
}
#[test]
fn test_list_mcp_servers_server_with_command_no_args() {
        // MCP server with command but no args should render detail as just the command
    with_temp_config(
        "mcp_servers:\n  simple:\n    command: uvx\n",
        || {
            let result = list_mcp_servers();
            assert!(result.is_ok());
            let json = result.unwrap();
            let servers = json["mcp_servers"].as_array().unwrap();
            assert_eq!(servers.len(), 1);
            let server = &servers[0];
            assert_eq!(server["name"], "simple");
            assert_eq!(server["type"], "stdio");
            assert_eq!(server["detail"], "uvx");
        },
    );
}
#[test]
fn test_set_toolset_enabled_empty_config_file() {
        // Toggling in an empty config file should create the structure
    with_temp_config("", || {
        let result = set_toolset_enabled("terminal".to_string(), false);
        assert!(result.is_ok());
            // Verify the structure was created and terminal is disabled
        let json = list_toolsets().unwrap();
        let terminal = json["toolsets"].as_array().unwrap().iter()
            .find(|t| t["key"].as_str() == Some("terminal"))
            .unwrap();
        assert!(!terminal["enabled"].as_bool().unwrap(),
            "terminal should be disabled after toggle on empty config");
    });
}
#[test]
fn test_list_mcp_servers_empty_object() {
        // When mcp_servers is an empty mapping {}, should return empty array
    with_temp_config("mcp_servers: {}", || {
        let result = list_mcp_servers();
        assert!(result.is_ok());
        let json = result.unwrap();
        let servers = json["mcp_servers"].as_array().unwrap();
        assert!(
            servers.is_empty(),
            "Empty mcp_servers mapping should return no servers"
        );
    });
}
#[test]
fn test_set_toolset_enabled_empty_key() {
        // Empty string key should not crash
    with_temp_config("platform_toolsets:\n  cli:\n    - web\n", || {
        let result = set_toolset_enabled("".to_string(), true);
        assert!(result.is_ok());
            // Verify config is still valid after trying empty key
        let json = list_toolsets().unwrap();
        let toolsets = json["toolsets"].as_array().unwrap();
        assert_eq!(toolsets.len(), 16);
    });
}
#[test]
fn test_list_mcp_servers_mcp_servers_is_null() {
        // When mcp_servers is explicitly null, should return empty array gracefully
    with_temp_config("mcp_servers:\nmodel:\n  default: gpt-4\n", || {
        let result = list_mcp_servers();
        assert!(result.is_ok());
        let json = result.unwrap();
        let servers = json["mcp_servers"].as_array().unwrap();
        assert!(
            servers.is_empty(),
            "null mcp_servers should return empty array"
        );
    });
}
#[test]
fn test_list_toolsets_platform_is_not_mapping() {
        // When platform_toolsets is a string instead of a mapping, all toolsets should be enabled
    with_temp_config("platform_toolsets: invalid_string\n", || {
        let result = list_toolsets();
        assert!(result.is_ok());
        let json = result.unwrap();
        let toolsets = json["toolsets"].as_array().unwrap();
        assert_eq!(toolsets.len(), 16);
        for ts in toolsets {
            assert!(
                ts["enabled"].as_bool().unwrap(),
                "{} should be enabled when platform_toolsets is not a mapping",
                ts["key"].as_str().unwrap()
            );
        }
    });
}
#[test]
fn test_list_toolsets_cli_with_non_string_values() {
        // When cli list contains non-string values (numbers, booleans), those should be filtered
    with_temp_config(
        "platform_toolsets:\n  cli:\n    - web\n    - 42\n    - true\n    - terminal\n",
        || {
            let result = list_toolsets();
            assert!(result.is_ok());
            let json = result.unwrap();
            let toolsets = json["toolsets"].as_array().unwrap();
            assert_eq!(toolsets.len(), 16);
                // web and terminal should be enabled
            for ts in toolsets {
                let key = ts["key"].as_str().unwrap();
                let enabled = ts["enabled"].as_bool().unwrap();
                match key {
                    "web" | "terminal" => assert!(enabled, "{} should be enabled", key),
                    _ => assert!(!enabled, "{} should be disabled", key),
                }
            }
        },
    );
}
#[test]
fn test_set_toolset_enabled_disables_all_16() {
        // Boundary: disabling all 16 toolsets should result in an empty cli list,
        // which means all toolsets are enabled by default (backward compatible behavior)
    with_temp_config("model:\n  default: gpt-4\n", || {
        for (key, _, _) in ALL_TOOLSETS {
            let result = set_toolset_enabled(key.to_string(), false);
            assert!(result.is_ok(), "Failed to disable toolset {}", key);
        }
            // When all are disabled, cli list becomes empty,
            // and empty list means all default to enabled
        let json = list_toolsets().unwrap();
        for ts in json["toolsets"].as_array().unwrap() {
            assert!(
                ts["enabled"].as_bool().unwrap(),
                "{} should be enabled when cli list is empty",
                ts["key"].as_str().unwrap()
            );
        }
    });
}
#[test]
fn test_set_toolset_enabled_reenable_after_disabling_all() {
        // Disable all 16 toolsets, then re-enable one — should work correctly
    with_temp_config("model:\n  default: gpt-4\n", || {
            // Disable all
        for (key, _, _) in ALL_TOOLSETS {
            set_toolset_enabled(key.to_string(), false).ok();
        }
            // Re-enable one
        let result = set_toolset_enabled("web".to_string(), true);
        assert!(result.is_ok());
            // Verify only web is enabled
        let json = list_toolsets().unwrap();
        for ts in json["toolsets"].as_array().unwrap() {
            let key = ts["key"].as_str().unwrap();
            let enabled = ts["enabled"].as_bool().unwrap();
            if key == "web" {
                assert!(enabled, "web should be enabled after re-enable");
            } else {
                assert!(!enabled, "{} should still be disabled", key);
            }
        }
    });
}
#[test]
fn test_list_mcp_servers_with_empty_url() {
        // MCP server with an empty url string should still be classified as http
    with_temp_config(
        "mcp_servers:\n  empty-url:\n    url: ''\n",
        || {
            let result = list_mcp_servers();
            assert!(result.is_ok());
            let json = result.unwrap();
            let servers = json["mcp_servers"].as_array().unwrap();
            assert_eq!(servers.len(), 1);
            let server = &servers[0];
            assert_eq!(server["name"], "empty-url");
            assert_eq!(server["type"], "http");
            assert_eq!(server["detail"], "");
        },
    );
}
#[test]
fn test_list_mcp_servers_with_empty_command() {
        // MCP server with an empty command string should be classified as stdio
    with_temp_config(
        "mcp_servers:\n  empty-cmd:\n    command: ''\n",
        || {
            let result = list_mcp_servers();
            assert!(result.is_ok());
            let json = result.unwrap();
            let servers = json["mcp_servers"].as_array().unwrap();
            assert_eq!(servers.len(), 1);
            let server = &servers[0];
            assert_eq!(server["name"], "empty-cmd");
            assert_eq!(server["type"], "stdio");
            assert_eq!(server["detail"], "");
        },
    );
}
#[test]
fn test_list_toolsets_cli_with_special_chars() {
        // cli list with toolset keys containing special characters
    with_temp_config(
        "platform_toolsets:\n  cli:\n    - web\n    - some-tool_with.special/chars\n",
        || {
            let result = list_toolsets();
            assert!(result.is_ok());
            let json = result.unwrap();
            let toolsets = json["toolsets"].as_array().unwrap();
            assert_eq!(toolsets.len(), 16);
                // Only web should be enabled (special key doesn't match any known toolset)
            let web = toolsets.iter().find(|t| t["key"].as_str() == Some("web")).unwrap();
            assert!(web["enabled"].as_bool().unwrap(), "web should be enabled");
        },
    );
}
#[test]
fn test_list_mcp_servers_mixed_valid_and_invalid_entries() {
        // Mixed entries: valid http, valid stdio, broken (no url/command), empty url
    with_temp_config(
        "mcp_servers:\n  valid-http:\n    url: http://localhost:3000\n  valid-stdio:\n    command: python\n    args:\n      - server.py\n  broken:\n    config_file: path/to/config\n  empty-url:\n    url: ''\n",
        || {
            let result = list_mcp_servers();
            assert!(result.is_ok());
            let json = result.unwrap();
            let servers = json["mcp_servers"].as_array().unwrap();
            assert_eq!(servers.len(), 3, "should include valid-http, valid-stdio, empty-url");
            assert!(servers.iter().any(|s| s["name"] == "valid-http"), "valid-http should be present");
            assert!(servers.iter().any(|s| s["name"] == "valid-stdio"), "valid-stdio should be present");
            assert!(servers.iter().any(|s| s["name"] == "empty-url"), "empty-url should be present");
            assert!(!servers.iter().any(|s| s["name"] == "broken"), "broken entry should be filtered out");
        },
    );
}
    // ══════════════════════════════════════════════════════════════════════════
    // IPC-style tests via tauri::test::get_ipc_response
    // ══════════════════════════════════════════════════════════════════════════
#[test]
fn test_ipc_list_toolsets() {
        // IPC call to list_toolsets should route correctly and return 16 toolsets
    with_temp_config("model:\n  default: gpt-4\n", || {
        let (_app, ww) = build_test_app();
        let result: serde_json::Value =
            invoke_ipc(&ww, "list_toolsets", json!({})).expect("list_toolsets IPC should succeed");
        let toolsets = result["toolsets"].as_array().unwrap();
        assert_eq!(toolsets.len(), 16);
        for ts in toolsets {
            assert!(
                ts["enabled"].as_bool().unwrap(),
                "{} should be enabled",
                ts["key"].as_str().unwrap()
            );
        }
    });
}
#[test]
fn test_ipc_list_mcp_servers() {
        // IPC call to list_mcp_servers should return correctly typed servers
    with_temp_config(
        "mcp_servers:\n  time:\n    command: uvx\n    args:\n      - mcp-server-time\n  my-api:\n    url: http://localhost:8080\n",
        || {
            let (_app, ww) = build_test_app();
            let result: serde_json::Value =
                invoke_ipc(&ww, "list_mcp_servers", json!({})).expect("list_mcp_servers IPC should succeed");
            let servers = result["mcp_servers"].as_array().unwrap();
            assert_eq!(servers.len(), 2);
            let time = servers.iter().find(|s| s["name"] == "time").unwrap();
            assert_eq!(time["type"], "stdio");
            let api = servers.iter().find(|s| s["name"] == "my-api").unwrap();
            assert_eq!(api["type"], "http");
        },
    );
}
#[test]
fn test_ipc_set_toolset_enabled() {
        // IPC call to set_toolset_enabled should route params correctly
    with_temp_config("platform_toolsets:\n  cli:\n    - web\n", || {
        let (_app, ww) = build_test_app();
        let result: serde_json::Value = invoke_ipc(
            &ww,
            "set_toolset_enabled",
            json!({"key": "terminal", "enabled": true}),
        )
        .expect("set_toolset_enabled IPC should succeed");
        assert_eq!(result["success"], true);
    });
}
#[test]
fn test_ipc_get_hermes_config_info() {
        // IPC call to get_hermes_config_info should return expected shape
    let (_app, ww) = build_test_app();
    let result: serde_json::Value =
        invoke_ipc(&ww, "get_hermes_config_info", json!({}))
            .expect("get_hermes_config_info IPC should succeed");
    assert!(result["hermesHome"].is_string());
    assert!(result["configExists"].is_boolean());
    assert!(result["installed"].is_boolean());
}
#[test]
fn test_ipc_export_hermes_config() {
        // IPC call to export_hermes_config should return content
    with_temp_config("model:\n  default: gpt-4\n", || {
        let (_app, ww) = build_test_app();
        let result: serde_json::Value =
            invoke_ipc(&ww, "export_hermes_config", json!({}))
                .expect("export_hermes_config IPC should succeed");
        assert_eq!(result["success"], true);
        assert!(result["content"].is_string());
        assert!(result["content"].as_str().unwrap().contains("gpt-4"));
    });
}
#[test]
fn test_ipc_import_hermes_config() {
        // IPC call to import_hermes_config with valid YAML
    with_temp_config("", || {
        let (_app, ww) = build_test_app();
        let result: serde_json::Value = invoke_ipc(
            &ww,
            "import_hermes_config",
            json!({"content": "model:\n  default: claude-3\n"}),
        )
        .expect("import_hermes_config IPC should succeed");
        assert_eq!(result["success"], true);
            // Verify the config was actually written
        let path = config_path();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("claude-3"));
    });
}
#[test]
fn test_ipc_hermes_set_config() {
        // IPC call to hermes_set_config with dot-notation key
    with_temp_config("model:\n  default: gpt-4\n", || {
        let (_app, ww) = build_test_app();
        let result: serde_json::Value = invoke_ipc(
            &ww,
            "hermes_set_config",
            json!({"key": "agent.service_tier", "value": "fast"}),
        )
        .expect("hermes_set_config IPC should succeed");
        assert_eq!(result["success"], true);
        assert_eq!(result["key"], "agent.service_tier");
    });
}
#[test]
fn test_ipc_agent_api_server_status() {
        // IPC call to agent_api_server_status should return status shape
    let (_app, ww) = build_test_app();
    let result: serde_json::Value =
        invoke_ipc(&ww, "agent_api_server_status", json!({}))
            .expect("agent_api_server_status IPC should succeed");
    assert!(result["installed"].is_boolean());
    assert!(result["configured"].is_boolean());
    assert!(result["running"].is_boolean());
    assert!(result["needsRestart"].is_boolean());
}
