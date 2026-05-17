/// Detailed test that verifies extra params are preserved through import → generate round-trip.
use tokio::runtime::Runtime;

fn setup_core_service() -> (supertool_core::logic::CoreService, String) {
    let dir = std::env::temp_dir().join("supertool_extra_param_test");
    let _ = std::fs::create_dir_all(&dir);
    let db_path = dir.join("test.db");
    let _ = std::fs::remove_file(&db_path);
    let db = supertool_core::db::Database::new(&db_path).unwrap();
    let preset_id = "extra-param-test";
    let now = "2025-06-01T00:00:00Z";
    db.conn().execute(
        "INSERT INTO nginx_presets (id, name, serverId, configPath, description, groupName, isActive, createdAt, updatedAt) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        rusqlite::params![preset_id, "extra-param-test", "", "/etc/nginx/nginx.conf", "", "default", 0, now, now],
    ).unwrap();
    let core = supertool_core::logic::CoreService::new(db, dir);
    (core, preset_id.to_string())
}

#[test]
fn test_parser_captures_extra_params() {
    let test_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("testdata");
    let config_text = std::fs::read_to_string(test_dir.join("nginx_production.conf")).unwrap();
    let parsed = supertool_core::logic::nginx_parser::parse_nginx_config(&config_text).unwrap();

    // Server-level extra params: client_max_body_size
    let prepay = parsed
        .servers
        .iter()
        .find(|s| s.server_name.contains("prepay-.*"))
        .unwrap();
    assert!(
        !prepay.extra_params.is_empty(),
        "prepay server should have extra_params"
    );
    assert_eq!(
        prepay.extra_params[0].name, "client_max_body_size",
        "first extra param should be client_max_body_size"
    );
    assert!(
        prepay.extra_params[0].value.contains("20M"),
        "client_max_body_size should be 20M"
    );

    let api_shop = parsed
        .servers
        .iter()
        .find(|s| s.server_name.contains("api-shop"))
        .unwrap();
    assert!(
        !api_shop.extra_params.is_empty(),
        "api-shop server should have extra_params"
    );
    assert_eq!(
        api_shop.extra_params[0].name, "client_max_body_size",
        "first extra param should be client_max_body_size"
    );
    assert!(
        api_shop.extra_params[0].value.contains("200M"),
        "client_max_body_size should be 200M"
    );

    // Location-level extra params: proxy_set_header* in /api is now handled
    // by header flag, not extra_params. The only extra would have been proxy_redirect
    // which is also handled by generator's hardcoded logic.
    let unionmch = parsed
        .servers
        .iter()
        .find(|s| s.server_name.contains("unionmch"))
        .unwrap();
    let api_loc = unionmch
        .locations
        .iter()
        .find(|l| l.path == "/api")
        .unwrap();
    // proxy_set_header, proxy_redirect are all handled by header flag / generator logic
    // So extra_params for /api should be 0
    assert_eq!(
        api_loc.extra_params.len(),
        0,
        "/api should have 0 extra_params (all handled by header/proxy_redirect)"
    );
    assert!(api_loc.header, "/api should have header=true");

    // gzip/add_header in root locations
    let root_loc = unionmch.locations.iter().find(|l| l.path == "/").unwrap();
    assert!(
        root_loc.extra_params.len() >= 5,
        "root location should have >=5 extra params"
    );
    assert!(
        root_loc
            .extra_params
            .iter()
            .any(|e| e.name == "gzip_static" && e.value.contains("on")),
        "should have gzip_static on"
    );
    assert!(
        root_loc
            .extra_params
            .iter()
            .any(|e| e.name == "add_header" && e.value.contains("Cache-Control")),
        "should have add_header Cache-Control"
    );
    assert!(
        root_loc.extra_params.iter().any(|e| e.name == "gzip_types"),
        "should have gzip_types"
    );

    // CORS headers in prepay /admin-api/ — handled by cros flag, not extra_params
    let prepay_srv = parsed
        .servers
        .iter()
        .find(|s| s.server_name.contains("prepay-.*"))
        .unwrap();
    let admin_api_loc = prepay_srv
        .locations
        .iter()
        .find(|l| l.path == "/admin-api/")
        .unwrap();
    assert!(
        admin_api_loc.cros,
        "should have cros=true for CORS location"
    );
    // WebSocket headers in /infra — handled by websocket flag, not extra_params
    let prepay_srv = parsed
        .servers
        .iter()
        .find(|s| s.server_name.contains("prepay-.*"))
        .unwrap();
    let infra_loc = prepay_srv
        .locations
        .iter()
        .find(|l| l.path == "/infra")
        .unwrap();
    assert!(
        infra_loc.websocket,
        "should have websocket=true for WebSocket location"
    );

    eprintln!("✅ Parser: all extra params correctly captured");
}

#[test]
fn test_extra_params_round_trip() {
    let test_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("testdata");
    let config_text = std::fs::read_to_string(test_dir.join("nginx_production.conf")).unwrap();
    let parsed = supertool_core::logic::nginx_parser::parse_nginx_config(&config_text).unwrap();

    let (core, preset_id) = setup_core_service();

    Runtime::new()
        .unwrap()
        .block_on(core.import_nginx_config(&preset_id, &config_text))
        .expect("import should succeed");

    let generated = core
        .db_read(|conn| {
            supertool_core::logic::nginx_generator::generate_nginx_config(conn, &preset_id)
                .map_err(|e| e.to_string())
        })
        .unwrap()
        .unwrap();

    let gen_parsed = supertool_core::logic::nginx_parser::parse_nginx_config(&generated).unwrap();

    // Compare extra_param counts between original and generated
    for (srv_idx, srv) in parsed.servers.iter().enumerate() {
        let gs = &gen_parsed.servers[srv_idx];
        assert_eq!(
            srv.extra_params.len(),
            gs.extra_params.len(),
            "Server {} extra_param count mismatch: orig={} gen={}",
            srv.server_name,
            srv.extra_params.len(),
            gs.extra_params.len()
        );
        for (loc_idx, loc) in srv.locations.iter().enumerate() {
            let gl = &gs.locations[loc_idx];
            assert_eq!(
                loc.extra_params.len(),
                gl.extra_params.len(),
                "Server {} Location '{}' extra_param count mismatch: orig={} gen={}",
                srv.server_name,
                loc.path,
                loc.extra_params.len(),
                gl.extra_params.len()
            );
        }
    }

    // Also check upstream extra_params
    for (ui, up) in parsed.upstreams.iter().enumerate() {
        let gu = &gen_parsed.upstreams[ui];
        assert_eq!(
            up.extra_params.len(),
            gu.extra_params.len(),
            "Upstream {} extra_param count mismatch: orig={} gen={}",
            up.name,
            up.extra_params.len(),
            gu.extra_params.len()
        );
    }

    // Check raw text for key directives
    let text_checks = [
        "proxy_set_header Host $host;",
        "proxy_set_header X-Real-IP $remote_addr;",
        "proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;",
        "proxy_redirect http:// https://;",
        "gzip_static on;",
        "add_header Cache-Control",
        "gzip on;",
        "gzip_proxied any;",
        "client_max_body_size 20M;",
        "client_max_body_size 200M;",
        "expires 30d;",
        "add_header Access-Control-Allow-Origin *;",
        "proxy_http_version 1.1;",
        "proxy_set_header Upgrade $http_upgrade;",
    ];
    for check in &text_checks {
        assert!(
            generated.contains(check),
            "Generated config missing: {}",
            check
        );
    }

    eprintln!("✅ Round-trip: ALL extra params preserved in generated config");
    eprintln!(
        "   Generated {} bytes, {} servers, {} upstreams, {} locations total",
        generated.len(),
        gen_parsed.servers.len(),
        gen_parsed.upstreams.len(),
        gen_parsed
            .servers
            .iter()
            .map(|s| s.locations.len())
            .sum::<usize>()
    );
}
