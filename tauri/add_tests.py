#!/usr/bin/env python3
"""Append new Rust integration tests before the closing '}' of the test module."""
import re

rs_path = 'tauri/src/commands/hermes_config.rs'

with open(rs_path, 'r') as f:
    content = f.read()

new_tests = '''
    #[test]
    fn test_list_toolsets_all_enabled_by_default() {
        // When config has no platform_toolsets section, all 16 toolsets should be enabled
        with_temp_config("model:\\n  default: gpt-4\\n", || {
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
        with_temp_config("platform_toolsets:\\n  gui:\\n    - web\\n", || {
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
        // When platform_toolsets.cli is an empty list, read_enabled_toolsets_from_value
        // returns None, so all toolsets should be enabled
        with_temp_config("platform_toolsets:\\n  cli:\\n", || {
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
'''

# Find the last closing brace of the test module
# The file ends with the test module closing '}'
# Replace the last '}' (module closing) with new tests + closing
last_brace = content.rstrip().rfind('}')
if last_brace != -1:
    # Remove trailing whitespace after last brace
    content = content[:last_brace].rstrip() + '\n'
    content += new_tests
    content += '\n'

with open(rs_path, 'w') as f:
    f.write(content)

print("Tests added successfully")
