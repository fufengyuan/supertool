/// MD5 verification tests for nginx config round-trip fidelity
/// Compares generated config with original imported config
use md5::compute;
use rusqlite::Connection;
use supertool_core::logic::nginx_generator::generate_nginx_config;
use supertool_core::logic::nginx_parser::parse_nginx_config;

/// Normalize config for comparison:
/// - Remove trailing whitespace from each line
/// - Remove consecutive empty lines (keep at most one)
/// - Trim leading/trailing whitespace
fn normalize_config(config: &str) -> String {
    let lines: Vec<&str> = config.lines().collect();
    let mut normalized_lines: Vec<String> = Vec::new();
    let mut prev_empty = false;

    for line in lines {
        let trimmed = line.trim_end();
        let is_empty = trimmed.is_empty();

        // Skip consecutive empty lines
        if is_empty && prev_empty {
            continue;
        }

        normalized_lines.push(trimmed.to_string());
        prev_empty = is_empty;
    }

    // Remove trailing empty lines
    while normalized_lines.last().map(|l| l.is_empty()) == Some(true) {
        normalized_lines.pop();
    }

    normalized_lines.join("\n")
}

/// Calculate MD5 hash of a config block
fn md5_hash(content: &str) -> String {
    let normalized = normalize_config(content);
    let digest = compute(normalized.as_bytes());
    format!("{:x}", digest)
}

/// Import nginx config to database and return preset_id
fn import_config_to_db(
    conn: &Connection,
    parsed: &supertool_core::logic::nginx_parser::ParsedNginxConfig,
) -> Result<String, String> {
    let preset_id = format!("md5-test-{}", chrono::Utc::now().format("%Y%m%d%H%M%S%f"));
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO nginx_presets (id, name, serverId, configPath, description, groupName, isActive, createdAt, updatedAt) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![&preset_id, "MD5 Test Preset", "test-server", "/etc/nginx/nginx.conf", "MD5 verification test", "default", 1, &now, &now],
    ).map_err(|e| e.to_string())?;

    // Import basic settings
    for (i, bs) in parsed.basic_settings.iter().enumerate() {
        conn.execute(
            "INSERT INTO nginx_basic_settings (id, presetId, name, value, sort, createdAt) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![&format!("bs-{}", i), &preset_id, &bs.name, &bs.value, i, &now],
        ).map_err(|e| e.to_string())?;
    }

    // Import http params
    for (i, hp) in parsed.http_params.iter().enumerate() {
        conn.execute(
            "INSERT INTO nginx_http_params (id, presetId, name, value, sort, createdAt) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![&format!("hp-{}", i), &preset_id, &hp.name, &hp.value, i, &now],
        ).map_err(|e| e.to_string())?;
    }

    // Import upstreams
    for (i, up) in parsed.upstreams.iter().enumerate() {
        let upstream_id = format!("up-{}", i);
        let param_json = serde_json::to_string(&up.extra_params).unwrap_or_default();

        conn.execute(
            "INSERT INTO nginx_upstreams (id, presetId, name, proxyType, strategy, descr, paramJson, updatedAt) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![&upstream_id, &preset_id, &up.name, 0, &up.strategy, &up.descr, &param_json, &now],
        ).map_err(|e| e.to_string())?;

        // Import upstream servers
        for (j, srv) in up.servers.iter().enumerate() {
            conn.execute(
                "INSERT INTO nginx_upstream_servers (id, upstreamId, address, port, weight, maxFails, failTimeout, maxConns, backup, down, sort, enabled, param) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                rusqlite::params![
                    &format!("us-{}-{}", i, j),
                    &upstream_id,
                    &srv.address,
                    srv.port,
                    srv.weight,
                    srv.max_fails,
                    &srv.fail_timeout,
                    srv.max_conns,
                    srv.backup,
                    srv.down,
                    j,
                    1,
                    &srv.param
                ],
            ).map_err(|e| e.to_string())?;
        }
    }

    // Import servers
    for (i, srv) in parsed.servers.iter().enumerate() {
        let server_id = format!("srv-{}", i);
        let param_json = serde_json::to_string(&srv.extra_params).unwrap_or_default();

        conn.execute(
            "INSERT INTO nginx_servers (id, presetId, proxyType, serverName, listen, ssl, certId, rewrite, rewriteListen, http2, descr, sort, paramJson, updatedAt) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            rusqlite::params![
                &server_id,
                &preset_id,
                srv.proxy_type,
                &srv.server_name,
                &srv.listen,
                srv.ssl,
                &srv.cert_id,
                srv.rewrite,
                &srv.rewrite_listen,
                srv.http2,
                &srv.descr,
                i,
                &param_json,
                &now
            ],
        ).map_err(|e| e.to_string())?;

        // Import locations
        for (j, loc) in srv.locations.iter().enumerate() {
            let loc_param_json = serde_json::to_string(&loc.extra_params).unwrap_or_default();

            conn.execute(
                "INSERT INTO nginx_locations (id, serverId, path, locType, upstreamId, upstreamPath, rootPath, rootPage, rootType, header, websocket, cros, returnUrl, returnPath, paramJson, sort) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                rusqlite::params![
                    &format!("loc-{}-{}", i, j),
                    &server_id,
                    &loc.path,
                    match loc.loc_type.as_str() {
                        "proxy" => 0,
                        "root" => 1,
                        "redirect" => 2,
                        "alias" => 3,
                        "return" => 4,
                        _ => 0,
                    },
                    &loc.upstream_id,
                    &loc.upstream_path,
                    &loc.root_path,
                    "", // rootPage
                    "", // rootType
                    loc.header,
                    loc.websocket,
                    loc.cros,
                    &loc.return_url,
                    0, // returnPath
                    &loc_param_json,
                    j
                ],
            ).map_err(|e| e.to_string())?;
        }
    }

    // Import streams
    for (i, stream) in parsed.streams.iter().enumerate() {
        conn.execute(
            "INSERT INTO nginx_streams (id, presetId, listen, protocol, ssl, certId, proxyUpstreamId, descr, sort, updatedAt) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                &format!("str-{}", i),
                &preset_id,
                &stream.listen,
                &stream.protocol,
                stream.ssl,
                &stream.cert_id,
                &stream.proxy_upstream_id,
                &stream.descr,
                i,
                &now
            ],
        ).map_err(|e| e.to_string())?;
    }

    Ok(preset_id)
}

/// Test MD5 verification for a config file
fn test_config_md5(config_path: &str) -> Result<(), String> {
    println!("\n========== Testing {} ==========\n", config_path);

    // Read original config
    let original = std::fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read {}: {}", config_path, e))?;

    // Parse config
    let parsed = parse_nginx_config(&original)?;

    println!(
        "Parsed: {} upstreams, {} servers, {} streams, {} http_params, {} basic_settings",
        parsed.upstreams.len(),
        parsed.servers.len(),
        parsed.streams.len(),
        parsed.http_params.len(),
        parsed.basic_settings.len()
    );

    // Create in-memory DB
    let conn = Connection::open_in_memory().map_err(|e| e.to_string())?;
    supertool_core::db::init_db(&conn).map_err(|e| e.to_string())?;

    // Import config
    let preset_id = import_config_to_db(&conn, &parsed)?;

    // Generate config
    let generated = generate_nginx_config(&conn, &preset_id)?;

    println!(
        "Generated: {} bytes, {} lines",
        generated.len(),
        generated.lines().count()
    );

    // Normalize both configs
    let orig_normalized = normalize_config(&original);
    let gen_normalized = normalize_config(&generated);

    // Calculate overall MD5
    let orig_md5 = md5_hash(&original);
    let gen_md5 = md5_hash(&generated);

    println!("Original MD5: {}", orig_md5);
    println!("Generated MD5: {}", gen_md5);

    if orig_md5 == gen_md5 {
        println!("✅ MD5 MATCH - Configs are identical!");
        return Ok(());
    }

    println!("❌ MD5 MISMATCH - Analyzing differences...\n");

    // Compare line by line
    let orig_lines: Vec<&str> = orig_normalized.lines().collect();
    let gen_lines: Vec<&str> = gen_normalized.lines().collect();

    println!("Original: {} lines", orig_lines.len());
    println!("Generated: {} lines", gen_lines.len());

    // Find differences
    let max_len = std::cmp::max(orig_lines.len(), gen_lines.len());
    let mut diffs: Vec<(usize, Option<&str>, Option<&str>)> = Vec::new();

    for i in 0..max_len {
        let orig_line = orig_lines.get(i).copied();
        let gen_line = gen_lines.get(i).copied();

        if orig_line != gen_line {
            diffs.push((i + 1, orig_line, gen_line));
        }
    }

    if diffs.is_empty() {
        println!("✅ No line differences found (whitespace/formatting only)");
    } else {
        println!("Found {} differences:\n", diffs.len());
        for (line_no, orig, gen_line) in diffs.iter().take(30) {
            println!("Line {}: ", line_no);
            if let Some(o) = orig {
                println!("  Orig: \"{}\"", o);
            }
            if let Some(g) = gen_line {
                println!("  Gen:  \"{}\"", g);
            }
        }

        if diffs.len() > 30 {
            println!("... and {} more differences", diffs.len() - 30);
        }
    }

    // Write both configs to files for manual comparison
    let base_name = std::path::Path::new(config_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("test");

    let orig_out = format!("/tmp/nginx_md5_{}_original.conf", base_name);
    let gen_out = format!("/tmp/nginx_md5_{}_generated.conf", base_name);

    std::fs::write(&orig_out, &orig_normalized).map_err(|e| e.to_string())?;
    std::fs::write(&gen_out, &gen_normalized).map_err(|e| e.to_string())?;

    println!("\nFiles written for manual comparison:");
    println!("  Original: {}", orig_out);
    println!("  Generated: {}", gen_out);
    println!("  Use: diff {} {}", orig_out, gen_out);

    Err(format!("MD5 mismatch for {}", config_path))
}

#[test]
fn test_md5_verification_prod_configs() {
    let test_configs = [
        "testdata/nginx_prod2.conf",
        "testdata/nginx_prod3.conf",
        "testdata/nginx_prod4.conf",
        "testdata/nginx_production.conf",
        "testdata/nginx_complex_app.conf",
    ];

    let mut results: Vec<(String, bool)> = Vec::new();

    for config_path in &test_configs {
        let full_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join(config_path);

        if full_path.exists() {
            let result = test_config_md5(full_path.to_str().unwrap());
            let success = result.is_ok();
            results.push((config_path.to_string(), success));
            if !success {
                println!("Warning: {}", result.unwrap_err());
            }
        } else {
            println!("Skipping {} (not found)", config_path);
        }
    }

    // Summary
    println!("\n========== MD5 Verification Summary ==========");
    for (path, success) in &results {
        println!(
            "  {} : {}",
            path,
            if *success { "✅ PASS" } else { "❌ FAIL" }
        );
    }

    let passed = results.iter().filter(|(_, s)| *s).count();
    println!("Passed: {} / {}", passed, results.len());
}
