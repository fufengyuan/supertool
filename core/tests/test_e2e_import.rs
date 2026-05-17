/// Full end-to-end test of the REAL import_nginx_config path (async CoreService method).
/// This catches any issues in the async wrapper, db_write locking, or SQL that the
/// direct DB function tests might miss.
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::runtime::Runtime;

static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

fn setup_core_service() -> (supertool_core::logic::CoreService, String) {
    let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("supertool_import_test_{}", counter));
    let _ = std::fs::create_dir_all(&dir);
    let db_path = dir.join("test.db");
    let _ = std::fs::remove_file(&db_path);

    let db = supertool_core::db::Database::new(&db_path).expect("Should create test database");

    // Insert a preset manually
    let preset_id = "e2e-import-test";
    let now = "2025-06-01T00:00:00Z";
    db.conn().execute(
        "INSERT INTO nginx_presets (id, name, serverId, configPath, description, groupName, isActive, createdAt, updatedAt) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        rusqlite::params![preset_id, "e2e-test", "", "/etc/nginx/nginx.conf", "", "default", 0, now, now],
    ).unwrap();

    let core = supertool_core::logic::CoreService::new(db, dir);
    (core, preset_id.to_string())
}

#[test]
fn test_e2e_import_nginx_config_small() {
    let (core, preset_id) = setup_core_service();

    let config_text = r#"worker_processes auto;
events {
    worker_connections 1024;
}
http {
    include mime.types;
    default_type application/octet-stream;
    upstream backend {
        server 10.0.0.1:8080 weight=5;
        server 10.0.0.2:8080 backup;
    }
    server {
        listen 80;
        server_name example.com;
        location / {
            proxy_pass http://backend;
            proxy_set_header Host $host;
        }
    }
    server {
        listen 443 ssl;
        server_name secure.example.com;
        ssl_certificate /etc/ssl/cert.pem;
        ssl_certificate_key /etc/ssl/key.pem;
        ssl_protocols TLSv1.2 TLSv1.3;
        location / {
            root /var/www;
            try_files $uri $uri/ /index.html;
        }
    }
}
stream {
    server {
        listen 3306;
        proxy_pass 10.0.0.1:3306;
    }
}"#;

    let result = Runtime::new()
        .unwrap()
        .block_on(core.import_nginx_config(&preset_id, config_text))
        .expect("import_nginx_config should succeed");

    eprintln!("Import result: {:?}", result.data);

    // Now generate and verify — use db_read to access the connection
    let generated = core
        .db_read(|conn| {
            supertool_core::logic::nginx_generator::generate_nginx_config(conn, &preset_id)
                .map_err(|e| e.to_string())
        })
        .expect("Should generate config after import")
        .unwrap();

    eprintln!("=== GENERATED CONFIG ===");
    for line in generated.lines().take(20) {
        eprintln!("{}", line);
    }

    assert!(
        generated.contains("worker_processes auto;"),
        "basic settings"
    );
    assert!(generated.contains("upstream backend {"), "upstream");
    assert!(generated.contains("server {"), "server");
    assert!(generated.contains("ssl_certificate"), "ssl cert");
    assert!(generated.contains("listen 3306;"), "stream");

    // Parse generated config and verify structural integrity
    let parsed = supertool_core::logic::nginx_parser::parse_nginx_config(&generated)
        .expect("Generated config should be parseable");

    assert_eq!(parsed.upstreams.len(), 1, "1 upstream");
    assert_eq!(parsed.servers.len(), 2, "2 servers");
    assert_eq!(parsed.streams.len(), 1, "1 stream");

    eprintln!(
        "\n✅ E2E import test passed: {} upstreams, {} servers, {} streams",
        parsed.upstreams.len(),
        parsed.servers.len(),
        parsed.streams.len()
    );
}

#[test]
fn test_e2e_import_production_config() {
    let (core, preset_id) = setup_core_service();

    let test_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("testdata");
    let config_text = std::fs::read_to_string(test_dir.join("nginx_production.conf"))
        .expect("Cannot read nginx_production.conf");

    eprintln!(
        "Importing production config ({} bytes)...",
        config_text.len()
    );

    let result = Runtime::new()
        .unwrap()
        .block_on(core.import_nginx_config(&preset_id, &config_text))
        .expect("import_nginx_config should succeed");

    let summary = result.data;
    eprintln!("Import result: {:?}", summary);

    // Generate — use db_read to access connection
    let generated = core
        .db_read(|conn| {
            supertool_core::logic::nginx_generator::generate_nginx_config(conn, &preset_id)
                .map_err(|e| e.to_string())
        })
        .expect("Should generate")
        .unwrap();

    eprintln!("\nGenerated: {} bytes", generated.len());

    // Verify structure matches original
    let parsed = supertool_core::logic::nginx_parser::parse_nginx_config(&generated)
        .expect("Generated config should be parseable");

    let original_parsed = supertool_core::logic::nginx_parser::parse_nginx_config(&config_text)
        .expect("Original config should be parseable");

    assert_eq!(
        parsed.upstreams.len(),
        original_parsed.upstreams.len(),
        "upstream count: {} vs {}",
        parsed.upstreams.len(),
        original_parsed.upstreams.len()
    );
    assert_eq!(
        parsed.servers.len(),
        original_parsed.servers.len(),
        "server count: {} vs {}",
        parsed.servers.len(),
        original_parsed.servers.len()
    );
    assert!(
        generated.contains("ssl_certificate"),
        "generated should have ssl_certificate (got {} occurrences)",
        generated.matches("ssl_certificate").count()
    );

    eprintln!("\n✅ E2E production import test passed:");
    eprintln!("   Upstreams: {}", parsed.upstreams.len());
    eprintln!("   Servers: {}", parsed.servers.len());
    eprintln!(
        "   SSL certs in generated: {}",
        generated.matches("ssl_certificate").count()
    );
}

#[test]
fn test_e2e_import_prod2_config() {
    let (core, preset_id) = setup_core_service();

    let test_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("testdata");
    let config_text = std::fs::read_to_string(test_dir.join("nginx_prod2.conf"))
        .expect("Cannot read nginx_prod2.conf");

    eprintln!(
        "Importing prod2 config ({} bytes, {} lines)...",
        config_text.len(),
        config_text.lines().count()
    );

    let result = Runtime::new()
        .unwrap()
        .block_on(core.import_nginx_config(&preset_id, &config_text))
        .expect("import_nginx_config should succeed");

    let summary = result.data;
    eprintln!("Import result: {:?}", summary);

    // Generate — use db_read
    let generated = core
        .db_read(|conn| {
            supertool_core::logic::nginx_generator::generate_nginx_config(conn, &preset_id)
                .map_err(|e| e.to_string())
        })
        .expect("Should generate")
        .unwrap();

    eprintln!("Generated: {} bytes", generated.len());

    // Verify
    let parsed = supertool_core::logic::nginx_parser::parse_nginx_config(&generated)
        .expect("Generated config should be parseable");

    let original_parsed = supertool_core::logic::nginx_parser::parse_nginx_config(&config_text)
        .expect("Original config should be parseable");

    assert_eq!(
        parsed.upstreams.len(),
        original_parsed.upstreams.len(),
        "upstream count: {} vs {}",
        parsed.upstreams.len(),
        original_parsed.upstreams.len()
    );
    assert_eq!(
        parsed.servers.len(),
        original_parsed.servers.len(),
        "server count: {} vs {}",
        parsed.servers.len(),
        original_parsed.servers.len()
    );
    let ssl_count = generated.matches("ssl_certificate").count();
    assert!(ssl_count > 0, "generated should have ssl_certificate");

    eprintln!("\n✅ E2E prod2 import test passed:");
    eprintln!("   Upstreams: {}", parsed.upstreams.len());
    eprintln!("   Servers: {}", parsed.servers.len());
    eprintln!("   SSL certs in generated: {}", ssl_count);

    // Check for any errors by parsing back
    let gen_parsed = supertool_core::logic::nginx_parser::parse_nginx_config(&generated)
        .expect("Generated config must be parseable");
    for srv in &gen_parsed.servers {
        let name = if srv.server_name.is_empty() {
            &srv.listen
        } else {
            &srv.server_name
        };
        eprintln!("   Server: {} ({} locations)", name, srv.locations.len());
    }
}
