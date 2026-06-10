//! Integration test: verify ~/.claw/settings.json format is compatible with upstream ConfigLoader.
//!
//! The critical rule: `provider` must be an **object** (with kind/apiKey/baseUrl),
//! NOT a plain string. The upstream ConfigLoader deserializes provider as a struct,
//! and a bare string causes: "field provider must be an object, got a string"
//!
//! Run: cd /path/to/supertool && cargo test -p supertool --test claw_config_test

use std::path::Path;

#[test]
fn test_claw_settings_provider_is_object() {
    let home = std::env::var("HOME").expect("HOME must be set");
    let settings_path = Path::new(&home).join(".claw").join("settings.json");

    assert!(
        settings_path.exists(),
        "settings.json not found at {}",
        settings_path.display()
    );

    let content = std::fs::read_to_string(&settings_path)
        .expect("Failed to read settings.json");

    let value: serde_json::Value = serde_json::from_str(&content)
        .expect("settings.json is not valid JSON");

    let provider = value.get("provider")
        .expect("settings.json must have 'provider' field");

    assert!(
        provider.is_object(),
        "❌ CRITICAL: 'provider' is a string ('{}'), must be an object!\n\
         ConfigLoader will fail with: 'field \"provider\" must be an object, got a string'\n\
         File: {}",
        provider.as_str().unwrap_or(""),
        settings_path.display()
    );

    let obj = provider.as_object().unwrap();
    assert!(obj.contains_key("kind"), "provider object missing 'kind'");
    assert!(obj.contains_key("apiKey"), "provider object missing 'apiKey'");
    assert!(!obj.get("kind").and_then(|v| v.as_str()).unwrap_or("").is_empty(), "provider.kind is empty");

    println!("✅ ~/.claw/settings.json format is correct!");
    println!("   File: {}", settings_path.display());
    println!("   provider.kind = {}", obj.get("kind").and_then(|v| v.as_str()).unwrap_or(""));
    println!("   provider.apiKey present: {}", obj.contains_key("apiKey"));
    println!("   provider.baseUrl present: {}", obj.contains_key("baseUrl"));
    println!("   model = {}", value.get("model").and_then(|v| v.as_str()).unwrap_or(""));
}
