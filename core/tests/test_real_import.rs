use rusqlite::Connection;
/// Real import path test — calls the actual DB CRUD functions used by import_nginx_config.
/// This catches SQL column/value mismatches that the test helper (insert_parsed_to_db)
/// wouldn't catch because it writes its own SQL.
use std::collections::HashMap;

fn setup_db() -> (Connection, String) {
    let conn = Connection::open_in_memory().unwrap();
    supertool_core::db::init_db(&conn).unwrap();
    let preset_id = "real-import-test";
    let now = "2025-06-01T00:00:00Z";
    conn.execute(
        "INSERT INTO nginx_presets (id, name, serverId, configPath, description, groupName, isActive, createdAt, updatedAt) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        rusqlite::params![preset_id, "import-test", "", "/etc/nginx/nginx.conf", "", "default", 0, now, now],
    ).unwrap();
    (conn, preset_id.to_string())
}

#[test]
fn test_real_import_funcs_basic_settings() {
    let (conn, pid) = setup_db();
    let now = "2025-06-01T00:00:00Z";

    // Insert via the REAL add_nginx_basic_setting function
    supertool_core::db::nginx::add_nginx_basic_setting(
        &conn,
        &supertool_core::db::nginx::NginxBasicSetting {
            id: "bs-1".into(),
            preset_id: pid.clone(),
            name: "worker_processes".into(),
            value: "auto".into(),
            sort: 0,
            created_at: now.into(),
        },
    )
    .unwrap();
    supertool_core::db::nginx::add_nginx_basic_setting(
        &conn,
        &supertool_core::db::nginx::NginxBasicSetting {
            id: "bs-2".into(),
            preset_id: pid.clone(),
            name: "events".into(),
            value: "{ worker_connections 1024; }".into(),
            sort: 1,
            created_at: now.into(),
        },
    )
    .unwrap();

    let settings = supertool_core::db::nginx::get_basic_settings_by_preset(&conn, &pid).unwrap();
    assert_eq!(
        settings.len(),
        2,
        "should insert 2 basic settings via real function"
    );
}

#[test]
fn test_real_import_funcs_server_with_locations() {
    let (conn, pid) = setup_db();
    let now = "2025-06-01T00:00:00Z";

    // Insert cert (needed for SSL server)
    supertool_core::db::nginx::add_nginx_cert(
        &conn,
        &supertool_core::db::nginx::NginxCert {
            id: "cert-1".into(),
            preset_id: pid.clone(),
            name: "test".into(),
            pem: "/etc/ssl/cert.pem".into(),
            key: "/etc/ssl/key.pem".into(),
            domain: "example.com".into(),
            created_at: now.into(),
        },
    )
    .unwrap();

    // Insert server via REAL add_nginx_server
    supertool_core::db::nginx::add_nginx_server(
        &conn,
        &supertool_core::db::nginx::NginxServer {
            id: "srv-1".into(),
            preset_id: pid.clone(),
            proxy_type: 0,
            listen: "443".into(),
            ip: "".into(),
            def: true,
            ipv6: true,
            proxy_protocol: false,
            server_name: "example.com".into(),
            ssl: true,
            cert_id: "cert-1".into(),
            rewrite: false,
            rewrite_listen: "".into(),
            http2: 1,
            protocols: "TLSv1.2 TLSv1.3".into(),
            password_id: "".into(),
            deny_allow: 0,
            deny_id: "".into(),
            allow_id: "".into(),
            proxy_upstream_id: "".into(),
            descr: "Test server".into(),
            key: "".to_string(),
            pem: "".to_string(),
            enabled: true,
            sort: 0,
            param_json: "".into(),
            created_at: now.into(),
            updated_at: now.into(),
        },
    )
    .unwrap();

    // Insert location via REAL add_nginx_location (the one that was broken!)
    supertool_core::db::nginx::add_nginx_location(
        &conn,
        &supertool_core::db::nginx::NginxLocation {
            id: "loc-1".into(),
            server_id: "srv-1".into(),
            enabled: true,
            path: "/".into(),
            loc_type: 0,
            value: "".into(),
            upstream_type: 0,
            upstream_id: "".into(),
            upstream_path: "".into(),
            root_path: "/var/www".into(),
            root_page: "".into(),
            root_type: "".into(),
            header: true,
            websocket: false,
            cros: false,
            header_host: "".into(),
            return_url: "".into(),
            return_path: false,
            param_json: "".into(),
            sort: 0,
            descr: "root".into(),
            created_at: now.into(),
        },
    )
    .unwrap();

    // Insert another location
    supertool_core::db::nginx::add_nginx_location(
        &conn,
        &supertool_core::db::nginx::NginxLocation {
            id: "loc-2".into(),
            server_id: "srv-1".into(),
            enabled: true,
            path: "/api".into(),
            loc_type: 0,
            value: "".into(),
            upstream_type: 1,
            upstream_id: "backend".into(),
            upstream_path: "/".into(),
            root_path: "".into(),
            root_page: "".into(),
            root_type: "".into(),
            header: true,
            websocket: true,
            cros: true,
            header_host: "$host".into(),
            return_url: "".into(),
            return_path: false,
            param_json: "".into(),
            sort: 1,
            descr: "api proxy".into(),
            created_at: now.into(),
        },
    )
    .unwrap();

    // Verify
    let locations = supertool_core::db::nginx::get_locations_by_server(&conn, "srv-1").unwrap();
    assert_eq!(
        locations.len(),
        2,
        "should have 2 locations via real function"
    );
    assert_eq!(locations[0].path, "/");
}

#[test]
fn test_real_import_funcs_upstream_with_servers() {
    let (conn, pid) = setup_db();
    let now = "2025-06-01T00:00:00Z";

    // Insert upstream via REAL add_nginx_upstream
    supertool_core::db::nginx::add_nginx_upstream(
        &conn,
        &supertool_core::db::nginx::NginxUpstream {
            id: "up-1".into(),
            preset_id: pid.clone(),
            name: "backend".into(),
            proxy_type: 0,
            strategy: "ip_hash".into(),
            descr: "".into(),
            param_json: "".into(),
            sort: 0,
            created_at: now.into(),
            updated_at: now.into(),
        },
    )
    .unwrap();

    // Insert upstream servers via REAL add_nginx_upstream_server
    supertool_core::db::nginx::add_nginx_upstream_server(
        &conn,
        &supertool_core::db::nginx::NginxUpstreamServer {
            id: "us-1".into(),
            upstream_id: "up-1".into(),
            address: "10.0.0.1".into(),
            port: 8080,
            weight: 5,
            max_fails: 3,
            fail_timeout: "10s".into(),
            max_conns: 100,
            backup: false,
            down: false,
            sort: 0,
            enabled: true,
            param: "".into(),
        },
    )
    .unwrap();
    supertool_core::db::nginx::add_nginx_upstream_server(
        &conn,
        &supertool_core::db::nginx::NginxUpstreamServer {
            id: "us-2".into(),
            upstream_id: "up-1".into(),
            address: "10.0.0.2".into(),
            port: 8080,
            weight: 3,
            max_fails: 5,
            fail_timeout: "30s".into(),
            max_conns: 0,
            backup: true,
            down: false,
            sort: 1,
            enabled: true,
            param: "".into(),
        },
    )
    .unwrap();

    // Verify
    let servers = supertool_core::db::nginx::get_upstream_servers(&conn, "up-1").unwrap();
    assert_eq!(servers.len(), 2);
    assert!(servers[1].backup);
}

#[test]
fn test_real_import_funcs_http_param_and_stream() {
    let (conn, pid) = setup_db();
    let now = "2025-06-01T00:00:00Z";

    supertool_core::db::nginx::add_nginx_http_param(
        &conn,
        &supertool_core::db::nginx::NginxHttpParam {
            id: "hp-1".into(),
            preset_id: pid.clone(),
            name: "sendfile".into(),
            value: "on".into(),
            enabled: true,
            sort: 0,
            created_at: now.into(),
        },
    )
    .unwrap();

    supertool_core::db::nginx::add_nginx_stream(
        &conn,
        &supertool_core::db::nginx::NginxStream {
            id: "st-1".into(),
            preset_id: pid.clone(),
            listen: "3306".into(),
            proxy_upstream_id: "".into(),
            proxy_pass: "10.0.0.1:3306".into(),
            ssl: false,
            cert_id: "".into(),
            protocol: "TCP".into(),
            descr: "MySQL".into(),
            enabled: true,
            sort: 0,
            param_json: "".into(),
            created_at: now.into(),
            updated_at: now.into(),
        },
    )
    .unwrap();

    let params = supertool_core::db::nginx::get_http_params_by_preset(&conn, &pid).unwrap();
    assert_eq!(params.len(), 1);
    let streams = supertool_core::db::nginx::get_streams_by_preset(&conn, &pid).unwrap();
    assert_eq!(streams.len(), 1);
}

#[test]
fn test_real_import_round_trip_production_conf() {
    // Test the full import path using real DB functions,
    // then generate and verify the output
    let (conn, pid) = setup_db();
    let now = "2025-06-01T00:00:00Z";

    let test_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("testdata");
    let text = std::fs::read_to_string(test_dir.join("nginx_production.conf"))
        .expect("Cannot read nginx_production.conf");

    let parsed = supertool_core::logic::nginx_parser::parse_nginx_config(&text)
        .expect("Should parse production config");

    // === Call real DB functions (same as import_nginx_config) ===

    // Basic settings
    for (i, bs) in parsed.basic_settings.iter().enumerate() {
        supertool_core::db::nginx::add_nginx_basic_setting(
            &conn,
            &supertool_core::db::nginx::NginxBasicSetting {
                id: format!("bs_{}", i),
                preset_id: pid.clone(),
                name: bs.name.clone(),
                value: bs.value.clone(),
                sort: i as i64,
                created_at: now.into(),
            },
        )
        .unwrap();
    }

    // HTTP params
    for (i, hp) in parsed.http_params.iter().enumerate() {
        supertool_core::db::nginx::add_nginx_http_param(
            &conn,
            &supertool_core::db::nginx::NginxHttpParam {
                id: format!("hp_{}", i),
                preset_id: pid.clone(),
                name: hp.name.clone(),
                value: hp.value.clone(),
                enabled: true,
                sort: i as i64,
                created_at: now.into(),
            },
        )
        .unwrap();
    }

    // Upstreams
    for (ui, up) in parsed.upstreams.iter().enumerate() {
        let up_id = format!("up_{}", ui);
        supertool_core::db::nginx::add_nginx_upstream(
            &conn,
            &supertool_core::db::nginx::NginxUpstream {
                id: up_id.clone(),
                preset_id: pid.clone(),
                name: up.name.clone(),
                proxy_type: 0,
                strategy: up.strategy.clone(),
                descr: up.descr.clone(),
                param_json: String::new(),
                sort: ui as i64,
                created_at: now.into(),
                updated_at: now.into(),
            },
        )
        .unwrap();
        for (si, srv) in up.servers.iter().enumerate() {
            supertool_core::db::nginx::add_nginx_upstream_server(
                &conn,
                &supertool_core::db::nginx::NginxUpstreamServer {
                    id: format!("us_{}_{}", ui, si),
                    upstream_id: up_id.clone(),
                    address: srv.address.clone(),
                    port: srv.port,
                    weight: srv.weight,
                    max_fails: srv.max_fails,
                    fail_timeout: if srv.fail_timeout.is_empty() {
                        "10s".to_string()
                    } else {
                        srv.fail_timeout.clone()
                    },
                    max_conns: srv.max_conns,
                    backup: srv.backup,
                    down: srv.down,
                    sort: si as i64,
                    enabled: true,
                    param: srv.param.clone(),
                },
            )
            .unwrap();
        }
    }

    // Certs (needed for SSL servers — this is what the real import now does)
    let mut cert_lookup: HashMap<String, String> = HashMap::new();
    for (ci, srv) in parsed.servers.iter().enumerate() {
        if srv.ssl != 0 && !srv.pem.is_empty() && !srv.key.is_empty() {
            let key = format!("{}|{}", srv.pem, srv.key);
            if !cert_lookup.contains_key(&key) {
                let cert_id = format!("icert_{}", ci);
                supertool_core::db::nginx::add_nginx_cert(
                    &conn,
                    &supertool_core::db::nginx::NginxCert {
                        id: cert_id.clone(),
                        preset_id: pid.clone(),
                        name: format!("imported_{}", ci),
                        pem: srv.pem.clone(),
                        key: srv.key.clone(),
                        domain: "imported".into(),
                        created_at: now.into(),
                    },
                )
                .unwrap();
                cert_lookup.insert(key, cert_id);
            }
        }
    }

    // Servers with locations — using REAL add_nginx_server and add_nginx_location
    for (si, srv) in parsed.servers.iter().enumerate() {
        let srv_id = format!("srv_{}", si);
        let resolved_cert_id = if srv.ssl != 0 && !srv.pem.is_empty() && !srv.key.is_empty() {
            let key = format!("{}|{}", srv.pem, srv.key);
            cert_lookup.get(&key).cloned().unwrap_or_default()
        } else {
            String::new()
        };

        supertool_core::db::nginx::add_nginx_server(
            &conn,
            &supertool_core::db::nginx::NginxServer {
                id: srv_id.clone(),
                preset_id: pid.clone(),
                proxy_type: 0,
                listen: srv.listen.clone(),
                ip: srv.ip.clone(),
                def: srv.def,
                ipv6: srv.ipv6,
                proxy_protocol: srv.proxy_protocol,
                server_name: srv.server_name.clone(),
                ssl: srv.ssl != 0,
                cert_id: resolved_cert_id,
                rewrite: srv.rewrite,
                rewrite_listen: srv.rewrite_listen.clone(),
                http2: srv.http2,
                protocols: srv.protocols.clone(),
                password_id: srv.password_id.clone(),
                deny_allow: srv.deny_allow,
                deny_id: srv.deny_id.clone(),
                allow_id: srv.allow_id.clone(),
                proxy_upstream_id: srv.proxy_upstream_id.clone(),
                descr: srv.descr.clone(),
                key: srv.key.clone(),
                pem: srv.pem.clone(),
                enabled: true,
                sort: si as i64,
                param_json: String::new(),
                created_at: now.into(),
                updated_at: now.into(),
            },
        )
        .unwrap_or_else(|e| panic!("add_nginx_server failed for {}: {}", srv.server_name, e));

        for (li, loc) in srv.locations.iter().enumerate() {
            let loc_type: i64 = match loc.loc_type.as_str() {
                "proxy_pass" => 0,
                "root" => 1,
                "upstream" => 2,
                "blank" => 3,
                "return" => 4,
                _ => 0,
            };
            supertool_core::db::nginx::add_nginx_location(
                &conn,
                &supertool_core::db::nginx::NginxLocation {
                    id: format!("loc_{}_{}", si, li),
                    server_id: srv_id.clone(),
                    enabled: true,
                    path: loc.path.clone(),
                    loc_type,
                    value: loc.value.clone(),
                    upstream_type: 0,
                    upstream_id: loc.upstream_id.clone(),
                    upstream_path: loc.upstream_path.clone(),
                    root_path: loc.root_path.clone(),
                    root_page: String::new(),
                    root_type: String::new(),
                    header: loc.header,
                    websocket: loc.websocket,
                    cros: loc.cros,
                    header_host: String::new(),
                    return_url: loc.return_url.clone(),
                    return_path: false,
                    param_json: String::new(),
                    sort: li as i64,
                    descr: loc.descr.clone(),
                    created_at: now.into(),
                },
            )
            .unwrap_or_else(|e| panic!("add_nginx_location failed for {}: {}", loc.path, e));
        }
    }

    // Streams
    for (sti, st) in parsed.streams.iter().enumerate() {
        supertool_core::db::nginx::add_nginx_stream(
            &conn,
            &supertool_core::db::nginx::NginxStream {
                id: format!("st_{}", sti),
                preset_id: pid.clone(),
                listen: st.listen.clone(),
                proxy_upstream_id: st.proxy_upstream_id.clone(),
                proxy_pass: st.proxy_pass.clone(),
                ssl: st.ssl != 0,
                cert_id: st.cert_id.clone(),
                protocol: st.protocol.clone(),
                descr: st.descr.clone(),
                enabled: true,
                sort: sti as i64,
                param_json: String::new(),
                created_at: now.into(),
                updated_at: now.into(),
            },
        )
        .unwrap();
    }

    // === Generate and verify ===
    let generated = supertool_core::logic::nginx_generator::generate_nginx_config(&conn, &pid)
        .expect("Should generate after real import");

    assert!(
        generated.contains("upstream"),
        "generated should have upstreams"
    );
    assert!(
        generated.contains("server {"),
        "generated should have servers"
    );
    assert!(
        generated.contains("ssl_certificate"),
        "generated should have ssl_certs"
    );

    // Parse the generated config and verify structural equality
    let gen_parsed = supertool_core::logic::nginx_parser::parse_nginx_config(&generated)
        .expect("generated config should be parseable");

    assert_eq!(
        gen_parsed.upstreams.len(),
        parsed.upstreams.len(),
        "same upstream count: {} vs {}",
        gen_parsed.upstreams.len(),
        parsed.upstreams.len()
    );
    assert_eq!(
        gen_parsed.servers.len(),
        parsed.servers.len(),
        "same server count: {} vs {}",
        gen_parsed.servers.len(),
        parsed.servers.len()
    );

    eprintln!(
        "✅ Real import round-trip: {} upstreams, {} servers, {} ssl_certs in generated",
        gen_parsed.upstreams.len(),
        gen_parsed.servers.len(),
        generated.matches("ssl_certificate").count()
    );
}
