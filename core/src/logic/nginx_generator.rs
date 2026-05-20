/// Nginx Config Generator
///
/// Reads structured data from DB and generates a complete nginx configuration text.
use crate::db::nginx::*;
use rusqlite::Connection;

/// Parse a listen string into a list of port numbers.
/// Supports: "80", "80,443", "8080-8090", "127.0.0.1:80", "127.0.0.1:8080-8090"
fn parse_ports(listen: &str) -> Vec<String> {
    let mut result = Vec::new();

    // Extract host prefix if present (e.g., "127.0.0.1:80" -> host="127.0.0.1:", port="80")
    let (host_prefix, port_part) = if let Some(idx) = listen.rfind(':') {
        // Check for IPv6
        if listen.contains('[') {
            ("".to_string(), listen.to_string())
        } else {
            (
                format!("{}:", &listen[..idx]),
                listen[idx + 1..].to_string(),
            )
        }
    } else {
        ("".to_string(), listen.to_string())
    };

    // Split by comma and expand ranges
    for part in port_part.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some(idx) = part.find('-') {
            let start: u16 = part[..idx].parse().unwrap_or(80);
            let end: u16 = part[idx + 1..].parse().unwrap_or(start);
            for p in start..=end {
                if p <= end {
                    result.push(format!("{}{}", host_prefix, p));
                }
            }
        } else {
            result.push(format!("{}{}", host_prefix, part));
        }
    }

    if result.is_empty() {
        result.push(listen.to_string());
    }
    result
}
pub fn generate_nginx_config(conn: &Connection, preset_id: &str) -> Result<String, String> {
    let mut output = String::new();

    // 1. Basic settings
    append_basic_settings(conn, preset_id, &mut output)?;

    // 2. HTTP block
    append_http_block(conn, preset_id, &mut output)?;

    // 3. Stream block
    append_stream_block(conn, preset_id, &mut output)?;

    Ok(output)
}

/// Result of a decomposed config generation (multi-file)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NginxConfigResult {
    pub main_config: String,
    pub sub_files: Vec<NginxSubFile>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NginxSubFile {
    pub filename: String,
    pub content: String,
}

/// Generate nginx config in decomposed mode (separate files for upstreams and server blocks)
pub fn generate_nginx_config_decomposed(
    conn: &Connection,
    preset_id: &str,
) -> Result<NginxConfigResult, String> {
    let mut sub_files: Vec<NginxSubFile> = Vec::new();
    let mut main = String::new();

    // 1. Basic settings (always in main)
    append_basic_settings(conn, preset_id, &mut main)?;

    // 2. HTTP block — decomposed
    append_http_block_decomposed(conn, preset_id, &mut main, &mut sub_files)?;

    // 3. Stream block — decomposed
    append_stream_block_decomposed(conn, preset_id, &mut main, &mut sub_files)?;

    Ok(NginxConfigResult {
        main_config: main,
        sub_files,
    })
}

fn append_basic_settings(
    conn: &Connection,
    preset_id: &str,
    out: &mut String,
) -> Result<(), String> {
    let settings = get_basic_settings_by_preset(conn, preset_id).map_err(|e| e.to_string())?;
    for s in &settings {
        if !s.name.is_empty() {
            let trimmed = s.value.trim();
            if trimmed.starts_with('{') {
                // Block-style basic setting (e.g. events { ... })
                out.push_str(&format!("{} {}\n", s.name, trimmed));
            } else {
                out.push_str(&format!("{} {};\n", s.name, s.value));
            }
        }
    }
    out.push_str("\n");
    Ok(())
}

fn append_http_block(conn: &Connection, preset_id: &str, out: &mut String) -> Result<(), String> {
    out.push_str("http {\n");

    // HTTP-level params
    let params = get_http_params_by_preset(conn, preset_id).map_err(|e| e.to_string())?;

    // Check if params already contains include/default_type (from imported config)
    let has_include = params.iter().any(|p| p.enabled && p.name == "include");
    let has_default_type = params.iter().any(|p| p.enabled && p.name == "default_type");

    // Only add default directives if not already present in params
    if !has_include {
        out.push_str("    include       mime.types;\n");
    }
    if !has_default_type {
        out.push_str("    default_type  application/octet-stream;\n");
    }
    out.push_str("\n");

    // Output HTTP-level params (skip include/default_type if we just added defaults)
    for p in &params {
        if p.enabled {
            // Skip include/default_type if we already added them as defaults
            if (!has_include && p.name == "include")
                || (!has_default_type && p.name == "default_type")
            {
                continue;
            }
            if p.value.contains("{\n") || p.value.contains("{ ") {
                // Block-style param (like geo/map) — no trailing semicolon
                out.push_str(&format!("    {}\n", format!("{} {}", p.name, p.value)));
            } else {
                out.push_str(&format!("    {} {};\n", p.name, p.value));
            }
        }
    }
    if !params.is_empty() {
        out.push_str("\n");
    }

    // Global HTTP-level IP blacklist/whitelist
    append_global_deny_allow(conn, preset_id, "http", out)?;

    // Upstreams
    let upstreams = get_upstreams_by_preset(conn, preset_id).map_err(|e| e.to_string())?;
    for u in &upstreams {
        if u.proxy_type != 0 {
            continue;
        } // only HTTP upstreams in http block
        append_upstream(conn, u, out)?;
    }

    // Servers
    let servers = get_servers_by_preset(conn, preset_id).map_err(|e| e.to_string())?;
    for s in &servers {
        if !s.enabled {
            continue;
        }
        if s.proxy_type != 0 {
            continue;
        } // only HTTP servers in http block
        append_server_block(conn, s, out)?;
    }

    out.push_str("}\n\n");
    Ok(())
}

/// Add global deny/allow directives for http/stream/server blocks
/// Uses well-known basic settings: denyAllow{Type}, denyId{Type}, allowId{Type}
fn append_global_deny_allow(
    conn: &Connection,
    preset_id: &str,
    block_type: &str,
    out: &mut String,
) -> Result<(), String> {
    let suffix = match block_type {
        "http" => "Http",
        "stream" => "Stream",
        _ => return Ok(()),
    };

    let settings = get_basic_settings_by_preset(conn, preset_id).map_err(|e| e.to_string())?;

    // Look up denyAllow{suffix} value
    let deny_allow_key = format!("denyAllow{}", suffix);
    let deny_id_key = format!("denyId{}", suffix);
    let allow_id_key = format!("allowId{}", suffix);

    let mut deny_allow_val: i64 = 0;
    let mut deny_id = String::new();
    let mut allow_id = String::new();

    for s in &settings {
        if s.name == deny_allow_key {
            deny_allow_val = s.value.parse::<i64>().unwrap_or(0);
        } else if s.name == deny_id_key {
            deny_id = s.value.clone();
        } else if s.name == allow_id_key {
            allow_id = s.value.clone();
        }
    }

    if deny_allow_val == 0 {
        return Ok(());
    }

    let indentation = "    ";

    if deny_allow_val == 1 {
        // Blacklist only
        if !deny_id.is_empty() {
            if let Ok(Some(da)) = get_deny_allow_by_id(conn, &deny_id) {
                for ip in da.ip.lines() {
                    let ip = ip.trim();
                    if !ip.is_empty() {
                        out.push_str(&format!("{}deny {};\n", indentation, ip));
                    }
                }
            }
        }
        out.push_str(&format!("{}allow all;\n", indentation));
    } else if deny_allow_val == 2 {
        // Whitelist only
        if !allow_id.is_empty() {
            if let Ok(Some(da)) = get_deny_allow_by_id(conn, &allow_id) {
                for ip in da.ip.lines() {
                    let ip = ip.trim();
                    if !ip.is_empty() {
                        out.push_str(&format!("{}allow {};\n", indentation, ip));
                    }
                }
            }
        }
        out.push_str(&format!("{}deny all;\n", indentation));
    } else if deny_allow_val == 3 {
        // Both blacklist and whitelist (allow first, then deny)
        if !allow_id.is_empty() {
            if let Ok(Some(da)) = get_deny_allow_by_id(conn, &allow_id) {
                for ip in da.ip.lines() {
                    let ip = ip.trim();
                    if !ip.is_empty() {
                        out.push_str(&format!("{}allow {};\n", indentation, ip));
                    }
                }
            }
        }
        if !deny_id.is_empty() {
            if let Ok(Some(da)) = get_deny_allow_by_id(conn, &deny_id) {
                for ip in da.ip.lines() {
                    let ip = ip.trim();
                    if !ip.is_empty() {
                        out.push_str(&format!("{}deny {};\n", indentation, ip));
                    }
                }
            }
        }
    }

    if !out.ends_with('\n') {
        out.push_str("\n");
    }
    Ok(())
}

/// Decomposed version: extract upstreams and server blocks into separate sub-files
fn append_http_block_decomposed(
    conn: &Connection,
    preset_id: &str,
    out: &mut String,
    sub_files: &mut Vec<NginxSubFile>,
) -> Result<(), String> {
    out.push_str("http {\n");

    // HTTP-level params
    let params = get_http_params_by_preset(conn, preset_id).map_err(|e| e.to_string())?;

    // Check if params already contains include/default_type (from imported config)
    let has_include = params.iter().any(|p| p.enabled && p.name == "include");
    let has_default_type = params.iter().any(|p| p.enabled && p.name == "default_type");

    // Only add default directives if not already present in params
    if !has_include {
        out.push_str("    include       mime.types;\n");
    }
    if !has_default_type {
        out.push_str("    default_type  application/octet-stream;\n");
    }
    out.push_str("\n");

    // Output HTTP-level params (skip include/default_type if we just added defaults)
    for p in &params {
        if p.enabled {
            // Skip include/default_type if we already added them as defaults
            if (!has_include && p.name == "include")
                || (!has_default_type && p.name == "default_type")
            {
                continue;
            }
            if p.value.contains("{\n") || p.value.contains("{ ") {
                // Block-style param (like geo/map) — no trailing semicolon
                out.push_str(&format!("    {}\n", format!("{} {}", p.name, p.value)));
            } else {
                out.push_str(&format!("    {} {};\n", p.name, p.value));
            }
        }
    }
    if !params.is_empty() {
        out.push_str("\n");
    }

    // Global HTTP-level IP blacklist/whitelist
    append_global_deny_allow(conn, preset_id, "http", out)?;

    // Upstreams (decomposed into separate files)
    let upstreams = get_upstreams_by_preset(conn, preset_id).map_err(|e| e.to_string())?;
    for u in &upstreams {
        if u.proxy_type != 0 {
            continue;
        }
        let mut sub = String::new();
        append_upstream(conn, u, &mut sub)?;
        let filename = sanitize_filename(&format!("http-upstream-{}.conf", u.name));
        add_sub_file(sub_files, &filename, &sub);
        out.push_str(&format!("    include conf.d/{};\n", filename));
    }

    // Servers (decomposed into separate files)
    let servers = get_servers_by_preset(conn, preset_id).map_err(|e| e.to_string())?;
    for s in &servers {
        if !s.enabled {
            continue;
        }
        if s.proxy_type != 0 {
            continue;
        }
        let mut sub = String::new();
        append_server_block(conn, s, &mut sub)?;
        let name = if !s.server_name.is_empty() {
            s.server_name.clone()
        } else {
            format!("http-{}", s.listen)
        };
        let filename = sanitize_filename(&format!("{}.conf", name));
        add_sub_file(sub_files, &filename, &sub);
        out.push_str(&format!("    include conf.d/{};\n", filename));
    }

    out.push_str("}\n\n");
    Ok(())
}

/// Helper: add or merge sub-file entry
fn add_sub_file(sub_files: &mut Vec<NginxSubFile>, filename: &str, content: &str) {
    for sf in sub_files.iter_mut() {
        if sf.filename == filename {
            sf.content.push_str("\n");
            sf.content.push_str(content);
            return;
        }
    }
    sub_files.push(NginxSubFile {
        filename: filename.to_string(),
        content: content.to_string(),
    });
}

/// Sanitize a string for use as a filename: replace invalid chars with _
fn sanitize_filename(name: &str) -> String {
    name.replace(' ', "_")
        .replace('/', "_")
        .replace('\\', "_")
        .replace(':', "_")
        .replace('*', "_")
        .replace('?', "_")
        .replace('"', "_")
        .replace('<', "_")
        .replace('>', "_")
        .replace('|', "_")
}

fn append_upstream(conn: &Connection, u: &NginxUpstream, out: &mut String) -> Result<(), String> {
    out.push_str(&format!("    upstream {} {{\n", u.name));

    // Description as comments
    if !u.descr.is_empty() {
        for line in u.descr.lines() {
            if !line.trim().is_empty() {
                out.push_str(&format!("        # {}\n", line.trim()));
            }
        }
    }

    // Strategy
    if u.strategy == "ip_hash" {
        out.push_str("        ip_hash;\n");
    } else if u.strategy == "least_conn" {
        out.push_str("        least_conn;\n");
    } else if u.strategy == "random" {
        out.push_str("        random;\n");
    } else if u.strategy == "sticky" {
        out.push_str("        sticky;\n");
    } else if u.strategy == "least_time" {
        out.push_str("        least_time;\n");
    }

    // Custom params - prepend mode (position=1)
    if !u.param_json.is_empty() {
        if let Ok(extras) = serde_json::from_str::<Vec<serde_json::Value>>(&u.param_json) {
            for extra in &extras {
                let pos = extra.get("position").and_then(|v| v.as_i64()).unwrap_or(0);
                if pos == 1 {
                    let name = extra.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let value = extra.get("value").and_then(|v| v.as_str()).unwrap_or("");
                    if !name.is_empty() {
                        out.push_str(&format!("        {} {};\n", name, value));
                    }
                }
            }
        }
    }

    // Upstream servers
    // Upstream servers
    let servers = crate::db::nginx::get_upstream_servers(conn, &u.id).map_err(|e| e.to_string())?;
    for srv in &servers {
        if !srv.enabled {
            continue;
        }
        out.push_str(&format!("        server {}:{}", srv.address, srv.port));
        if srv.weight > 1 {
            out.push_str(&format!(" weight={}", srv.weight));
        }
        if srv.max_fails != 3 {
            out.push_str(&format!(" max_fails={}", srv.max_fails));
        }
        if !srv.fail_timeout.is_empty() && srv.fail_timeout != "10s" {
            out.push_str(&format!(" fail_timeout={}", srv.fail_timeout));
        }
        if srv.max_conns > 0 {
            out.push_str(&format!(" max_conns={}", srv.max_conns));
        }
        if srv.backup {
            out.push_str(" backup");
        }
        if srv.down {
            out.push_str(" down");
        }
        if !srv.param.is_empty() {
            out.push_str(&format!(" {}", srv.param));
        }
        out.push_str(";\n");
    }

    // Custom params - append mode (position=0 or null)
    if !u.param_json.is_empty() {
        if let Ok(extras) = serde_json::from_str::<Vec<serde_json::Value>>(&u.param_json) {
            for extra in &extras {
                let pos = extra.get("position").and_then(|v| v.as_i64()).unwrap_or(0);
                if pos == 0 {
                    let name = extra.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let value = extra.get("value").and_then(|v| v.as_str()).unwrap_or("");
                    if !name.is_empty() {
                        out.push_str(&format!("        {} {};\n", name, value));
                    }
                }
            }
        }
    }

    out.push_str("    }\n\n");
    Ok(())
}

fn append_server_block(conn: &Connection, s: &NginxServer, out: &mut String) -> Result<(), String> {
    let locations =
        crate::db::nginx::get_locations_by_server(conn, &s.id).map_err(|e| e.to_string())?;
    append_server_block_inner(conn, s, &locations, out)
}

/// Shared server block generation logic used by both DB-backed and preview paths
fn append_server_block_inner(
    conn: &Connection,
    s: &NginxServer,
    locations: &[NginxLocation],
    out: &mut String,
) -> Result<(), String> {
    // Description as comments
    if !s.descr.is_empty() {
        for line in s.descr.lines() {
            if !line.trim().is_empty() {
                out.push_str(&format!("    # {}\n", line.trim()));
            }
        }
    }

    out.push_str("    server {\n");

    if s.proxy_type == 0 {
        // HTTP proxy

        // server_name
        if !s.server_name.is_empty() {
            out.push_str(&format!("        server_name  {};\n", s.server_name));
        }

        // listen directive (with port range support)
        let ports = parse_ports(&s.listen);
        for port in &ports {
            let mut listen_val = format!("listen {}", port);
            if s.def {
                listen_val += " default_server";
            }
            if s.proxy_protocol {
                listen_val += " proxy_protocol";
            }
            if s.ssl {
                listen_val += " ssl";
                if s.http2 == 1 {
                    listen_val += " http2";
                } // old-style http2
            }
            out.push_str(&format!("        {};\n", listen_val));
        }
        if s.ipv6 {
            for port in &ports {
                let mut listen_ipv6 = format!("listen [::]:{}", port);
                if s.def {
                    listen_ipv6 += " default_server";
                }
                if s.proxy_protocol {
                    listen_ipv6 += " proxy_protocol";
                }
                if s.ssl {
                    listen_ipv6 += " ssl";
                }
                out.push_str(&format!("        {};\n", listen_ipv6));
            }
        }

        // Rewrite listen (HTTP→HTTPS redirect second port)
        if s.rewrite && !s.rewrite_listen.is_empty() && s.rewrite_listen != s.listen {
            out.push_str(&format!("        listen {};\n", s.rewrite_listen));
        }

        // HTTP2 new-style (http2 on;)
        if s.ssl && s.http2 == 2 {
            out.push_str("        http2 on;\n");
        }

        // Password auth
        if !s.password_id.is_empty() {
            // Look up Password by ID
            if let Ok(Some(pw)) = get_password_by_id(conn, &s.password_id) {
                if !pw.descr.is_empty() {
                    out.push_str(&format!("        auth_basic           \"{}\";\n", pw.descr));
                }
                if !pw.path.is_empty() {
                    out.push_str(&format!("        auth_basic_user_file {};\n", pw.path));
                }
            }
        }

        // SSL certs
        if s.ssl && !s.cert_id.is_empty() {
            if let Ok(Some(cert)) = get_cert_by_id(conn, &s.cert_id) {
                out.push_str(&format!("        ssl_certificate      {};\n", cert.pem));
                out.push_str(&format!("        ssl_certificate_key  {};\n", cert.key));
            }
            if !s.protocols.is_empty() {
                out.push_str(&format!(
                    "        ssl_protocols       {};\n",
                    s.protocols.replace(",", " ")
                ));
            }
        }

        // Custom params - prepend mode
        append_param_json_prepend(conn, s, out);

        // IP blacklist/whitelist
        if s.deny_allow > 0 {
            if !s.deny_id.is_empty() || s.deny_allow == 2 || s.deny_allow == 3 {
                if let Ok(Some(da)) = get_deny_allow_by_id(
                    conn,
                    if s.deny_allow == 2 {
                        &s.allow_id
                    } else {
                        &s.deny_id
                    },
                ) {
                    for ip in da.ip.lines() {
                        let ip = ip.trim();
                        if !ip.is_empty() {
                            if s.deny_allow == 1 || s.deny_allow == 3 {
                                out.push_str(&format!("        deny {};\n", ip));
                            }
                            if s.deny_allow == 2 || s.deny_allow == 3 {
                                out.push_str(&format!("        allow {};\n", ip));
                            }
                        }
                    }
                }
            }
            if s.deny_allow == 1 {
                out.push_str("        allow all;\n");
            }
            if s.deny_allow == 2 {
                out.push_str("        deny all;\n");
            }
        }

        // Locations (passed in for preview support)
        for loc in locations {
            if !loc.enabled {
                continue;
            }
            append_location_block(conn, loc, s, out)?;
        }

        // Custom params - append mode from paramJson
        if !s.param_json.is_empty() {
            if let Ok(extras) = serde_json::from_str::<Vec<serde_json::Value>>(&s.param_json) {
                for extra in &extras {
                    let pos = extra.get("position").and_then(|v| v.as_i64()).unwrap_or(0);
                    if pos == 0 {
                        // Check if this entry references a template
                        if let Some(tid) = extra.get("templateId").and_then(|v| v.as_str()) {
                            if !tid.is_empty() {
                                if let Ok(Some(tpl)) = get_nginx_template_by_id(conn, tid) {
                                    for line in tpl.content.lines() {
                                        out.push_str(&format!("        {}\n", line));
                                    }
                                    continue;
                                }
                            }
                        }
                        let name = extra.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let value = extra.get("value").and_then(|v| v.as_str()).unwrap_or("");
                        if !name.is_empty() {
                            out.push_str(&format!("        {} {};\n", name, value));
                        }
                    }
                }
            }
        }

        // HTTP→HTTPS redirect (inside the same server block, like nginxWebUI)
        if s.ssl && s.rewrite {
            let port = s.listen.rsplit(':').next().unwrap_or(&s.listen).to_string();
            out.push_str(&format!(
                "        if ($scheme = http) {{\n            return 301 https://$host:{};\n        }}\n",
                port
            ));
        }
    } else {
        // TCP/UDP proxy (proxyType 1 or 2)

        let mut listen_val = format!("listen {}", s.listen);
        if s.proxy_protocol {
            listen_val += " proxy_protocol";
        }
        if s.proxy_type == 2 {
            listen_val += " udp";
        }
        if s.ssl {
            listen_val += " ssl";
        }
        out.push_str(&format!("        {};\n", listen_val));
        if s.ipv6 {
            let mut listen_ipv6 = format!("listen [::]:{}", s.listen);
            if s.proxy_protocol {
                listen_ipv6 += " proxy_protocol";
            }
            if s.ssl {
                listen_ipv6 += " ssl";
            }
            out.push_str(&format!("        {};\n", listen_ipv6));
        }

        // SSL certs
        if s.ssl && !s.cert_id.is_empty() {
            if let Ok(Some(cert)) = get_cert_by_id(conn, &s.cert_id) {
                out.push_str(&format!("        ssl_certificate      {};\n", cert.pem));
                out.push_str(&format!("        ssl_certificate_key  {};\n", cert.key));
            }
        }

        // Proxy pass
        if !s.proxy_upstream_id.is_empty() {
            let upstream_name = get_upstream_name(conn, &s.proxy_upstream_id);
            out.push_str(&format!("        proxy_pass {};\n", upstream_name));
        }
    }

    out.push_str("    }\n\n");
    Ok(())
}

/// Generate a single server block for preview (without saving to DB).
/// All DB lookups for reference data (certs, templates, etc.) still work
/// since those entities are already saved.
pub fn generate_server_block_preview(
    conn: &Connection,
    server: &NginxServer,
    locations: &[NginxLocation],
) -> Result<String, String> {
    let mut out = String::new();
    append_server_block_inner(conn, server, locations, &mut out)?;
    Ok(out)
}

fn append_location_block(
    conn: &Connection,
    loc: &NginxLocation,
    server: &NginxServer,
    out: &mut String,
) -> Result<(), String> {
    // Description as comments
    if !loc.descr.is_empty() {
        for line in loc.descr.lines() {
            if !line.trim().is_empty() {
                out.push_str(&format!("        # {}\n", line.trim()));
            }
        }
    }

    out.push_str(&format!("        location {} {{\n", loc.path));

    // Custom params - prepend mode
    append_location_param_json_prepend(conn, loc, out);

    match loc.upstream_type {
        0 | 2 => {
            // proxy_pass (type 0) or upstream (type 2)

            // proxy_pass directive
            if loc.upstream_type == 0 && !loc.value.is_empty() {
                out.push_str(&format!("            proxy_pass {};\n", loc.value));
            } else if loc.upstream_type == 2 || (!loc.upstream_id.is_empty()) {
                let upstream_type = if loc.upstream_type == 1 {
                    "https"
                } else {
                    "http"
                };
                let upstream_name = get_upstream_name(conn, &loc.upstream_id);
                let path = if loc.upstream_path.is_empty() {
                    ""
                } else {
                    &loc.upstream_path
                };
                out.push_str(&format!(
                    "            proxy_pass {}://{}{};\n",
                    upstream_type, upstream_name, path
                ));
            } else if loc.upstream_type == 1 && !loc.upstream_id.is_empty() {
                // Manual upstream reference
                out.push_str(&format!(
                    "            proxy_pass http://{};\n",
                    loc.upstream_id
                ));
            }

            // Websocket support
            if loc.websocket {
                out.push_str("            proxy_http_version 1.1;\n");
                out.push_str("            proxy_set_header Upgrade $http_upgrade;\n");
                out.push_str("            proxy_set_header Connection \"upgrade\";\n");
            }

            // Header settings with configurable headerHost
            if loc.header {
                out.push_str(&format!(
                    "            proxy_set_header Host {};\n",
                    if loc.header_host.is_empty() {
                        "$host"
                    } else {
                        &loc.header_host
                    }
                ));
                out.push_str("            proxy_set_header X-Real-IP $remote_addr;\n");
                out.push_str(
                    "            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;\n",
                );
                out.push_str("            proxy_set_header X-Forwarded-Proto $scheme;\n");
                out.push_str("            proxy_set_header X-Forwarded-Host $http_host;\n");
                out.push_str("            proxy_set_header X-Forwarded-Port $server_port;\n");
            }

            // CORS support
            if loc.cros {
                out.push_str("            add_header Access-Control-Allow-Origin *;\n");
                out.push_str("            add_header Access-Control-Allow-Methods *;\n");
                out.push_str("            add_header Access-Control-Allow-Headers *;\n");
                out.push_str("            add_header Access-Control-Allow-Credentials true;\n");
                out.push_str("            if ($request_method = 'OPTIONS') {\n");
                out.push_str("                return 204;\n");
                out.push_str("            }\n");
            }

            // proxy_redirect for SSL
            if server.ssl && server.rewrite {
                out.push_str("            proxy_redirect http:// https://;\n");
            }
        }
        1 => {
            // Root / static
            let root_type = if loc.root_type == "alias" {
                "alias"
            } else {
                "root"
            };
            if loc.root_path.contains('$') {
                // Dynamic path — use as-is
                out.push_str(&format!("            {} {};\n", root_type, loc.root_path));
            } else {
                let path = loc.root_path.trim_end_matches('/');
                out.push_str(&format!(
                    "            {} {};\n",
                    root_type,
                    if root_type == "alias" {
                        format!("{}/", path)
                    } else {
                        path.to_string()
                    }
                ));
                if !loc.root_page.is_empty() {
                    out.push_str(&format!("            index {};\n", loc.root_page));
                }
            }
        }
        3 => {} // blank — placeholder
        4 => {
            // Return/redirect
            let ret_url = if loc.return_path {
                format!("{}$request_uri", loc.return_url)
            } else {
                loc.return_url.clone()
            };
            out.push_str(&format!(
                "            return {} {};\n",
                if loc.value.is_empty() {
                    "302"
                } else {
                    &loc.value
                },
                ret_url
            ));
        }
        _ => {}
    }

    // Custom params - append mode
    append_location_param_json_append(conn, loc, out);

    out.push_str("        }\n\n");
    Ok(())
}

fn append_param_json_prepend(conn: &Connection, s: &NginxServer, out: &mut String) {
    if !s.param_json.is_empty() {
        if let Ok(extras) = serde_json::from_str::<Vec<serde_json::Value>>(&s.param_json) {
            for extra in &extras {
                let pos = extra.get("position").and_then(|v| v.as_i64()).unwrap_or(0);
                if pos == 1 {
                    // Check if this entry references a template
                    if let Some(tid) = extra.get("templateId").and_then(|v| v.as_str()) {
                        if !tid.is_empty() {
                            if let Ok(Some(tpl)) = get_nginx_template_by_id(conn, tid) {
                                for line in tpl.content.lines() {
                                    out.push_str(&format!("        {}\n", line));
                                }
                                continue;
                            }
                        }
                    }
                    let name = extra.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let value = extra.get("value").and_then(|v| v.as_str()).unwrap_or("");
                    if !name.is_empty() {
                        out.push_str(&format!("        {} {};\n", name, value));
                    }
                }
            }
        }
    }
}

fn append_stream_param_json_prepend(conn: &Connection, s: &NginxStream, out: &mut String) {
    if !s.param_json.is_empty() {
        if let Ok(extras) = serde_json::from_str::<Vec<serde_json::Value>>(&s.param_json) {
            for extra in &extras {
                if let Some(tid) = extra.get("templateId").and_then(|v| v.as_str()) {
                    if !tid.is_empty() {
                        if let Ok(Some(tpl)) = get_nginx_template_by_id(conn, tid) {
                            for line in tpl.content.lines() {
                                out.push_str(&format!("        {}\n", line));
                            }
                            continue;
                        }
                    }
                }
                let name = extra.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let value = extra.get("value").and_then(|v| v.as_str()).unwrap_or("");
                if !name.is_empty() {
                    out.push_str(&format!("        {} {};\n", name, value));
                }
            }
        }
    }
}

fn append_location_param_json_prepend(conn: &Connection, loc: &NginxLocation, out: &mut String) {
    if !loc.param_json.is_empty() {
        if let Ok(extras) = serde_json::from_str::<Vec<serde_json::Value>>(&loc.param_json) {
            for extra in &extras {
                let pos = extra.get("position").and_then(|v| v.as_i64()).unwrap_or(0);
                if pos == 1 {
                    // Check if this entry references a template
                    if let Some(tid) = extra.get("templateId").and_then(|v| v.as_str()) {
                        if !tid.is_empty() {
                            if let Ok(Some(tpl)) = get_nginx_template_by_id(conn, tid) {
                                for line in tpl.content.lines() {
                                    out.push_str(&format!("            {}\n", line));
                                }
                                continue;
                            }
                        }
                    }
                    let name = extra.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let value = extra.get("value").and_then(|v| v.as_str()).unwrap_or("");
                    if !name.is_empty() {
                        out.push_str(&format!("            {} {};\n", name, value));
                    }
                }
            }
        }
    }
}

fn append_location_param_json_append(conn: &Connection, loc: &NginxLocation, out: &mut String) {
    if !loc.param_json.is_empty() {
        if let Ok(extras) = serde_json::from_str::<Vec<serde_json::Value>>(&loc.param_json) {
            for extra in &extras {
                let pos = extra.get("position").and_then(|v| v.as_i64()).unwrap_or(0);
                // Default position 0 = append
                if pos == 0 || pos == 2 {
                    // Check if this entry references a template
                    if let Some(tid) = extra.get("templateId").and_then(|v| v.as_str()) {
                        if !tid.is_empty() {
                            if let Ok(Some(tpl)) = get_nginx_template_by_id(conn, tid) {
                                for line in tpl.content.lines() {
                                    out.push_str(&format!("            {}\n", line));
                                }
                                continue;
                            }
                        }
                    }
                    let name = extra.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let value = extra.get("value").and_then(|v| v.as_str()).unwrap_or("");
                    if !name.is_empty() {
                        out.push_str(&format!("            {} {};\n", name, value));
                    }
                }
            }
        }
    }
}

fn get_upstream_name(conn: &Connection, upstream_id: &str) -> String {
    if let Ok(Some(u)) = crate::db::nginx::get_upstream_by_id(conn, upstream_id) {
        u.name
    } else {
        upstream_id.to_string()
    }
}

fn get_cert_path(conn: &Connection, cert_id: &str) -> (String, String) {
    if let Ok(Some(cert)) = get_cert_by_id(conn, cert_id) {
        (cert.pem, cert.key)
    } else {
        (String::new(), String::new())
    }
}

fn append_stream_block(conn: &Connection, preset_id: &str, out: &mut String) -> Result<(), String> {
    let streams =
        crate::db::nginx::get_streams_by_preset(conn, preset_id).map_err(|e| e.to_string())?;
    if streams.is_empty() {
        return Ok(());
    }

    out.push_str("stream {\n");
    // Global stream-level IP blacklist/whitelist
    append_global_deny_allow(conn, preset_id, "stream", out)?;
    for s in &streams {
        if !s.enabled {
            continue;
        }
        out.push_str(&format!("    server {{\n"));
        out.push_str(&format!("        listen {};\n", s.listen));
        if s.ssl {
            if !s.cert_id.is_empty() {
                let (pem, key) = get_cert_path(conn, &s.cert_id);
                if !pem.is_empty() {
                    out.push_str(&format!("        ssl_certificate {};\n", pem));
                }
                if !key.is_empty() {
                    out.push_str(&format!("        ssl_certificate_key {};\n", key));
                }
            }
            out.push_str("        ssl_protocols TLSv1.2 TLSv1.3;\n");
        }
        if !s.proxy_upstream_id.is_empty() {
            let upstream_name = get_upstream_name(conn, &s.proxy_upstream_id);
            out.push_str(&format!("        proxy_pass {};\n", upstream_name));
        } else if !s.proxy_pass.is_empty() {
            out.push_str(&format!("        proxy_pass {};\n", s.proxy_pass));
        }
        // Custom params - prepend mode
        append_stream_param_json_prepend(conn, s, out);
        out.push_str("    }\n\n");
    }
    out.push_str("}\n\n");
    Ok(())
}

/// Decomposed version: extract stream upstreams and servers into sub-files
fn append_stream_block_decomposed(
    conn: &Connection,
    preset_id: &str,
    out: &mut String,
    sub_files: &mut Vec<NginxSubFile>,
) -> Result<(), String> {
    let streams =
        crate::db::nginx::get_streams_by_preset(conn, preset_id).map_err(|e| e.to_string())?;
    if streams.is_empty() {
        return Ok(());
    }

    // Decompose stream upstreams into sub-files
    let upstreams = get_upstreams_by_preset(conn, preset_id).map_err(|e| e.to_string())?;
    for u in &upstreams {
        if u.proxy_type != 1 {
            continue;
        }
        let mut sub = String::new();
        append_upstream(conn, u, &mut sub)?;
        let filename = sanitize_filename(&format!("stream-upstream-{}.conf", u.name));
        add_sub_file(sub_files, &filename, &sub);
        out.push_str(&format!("    include conf.d/{};\n", filename));
    }

    out.push_str("stream {\n");
    // Global stream-level IP blacklist/whitelist
    append_global_deny_allow(conn, preset_id, "stream", out)?;
    for s in &streams {
        if !s.enabled {
            continue;
        }

        // Generate stream server block into sub-file
        let mut sub = String::new();
        sub.push_str(&format!("    server {{\n"));
        sub.push_str(&format!("        listen {};\n", s.listen));
        if s.ssl {
            if !s.cert_id.is_empty() {
                let (pem, key) = get_cert_path(conn, &s.cert_id);
                if !pem.is_empty() {
                    sub.push_str(&format!("        ssl_certificate {};\n", pem));
                }
                if !key.is_empty() {
                    sub.push_str(&format!("        ssl_certificate_key {};\n", key));
                }
            }
            sub.push_str("        ssl_protocols TLSv1.2 TLSv1.3;\n");
        }
        if !s.proxy_upstream_id.is_empty() {
            let upstream_name = get_upstream_name(conn, &s.proxy_upstream_id);
            sub.push_str(&format!("        proxy_pass {};\n", upstream_name));
        } else if !s.proxy_pass.is_empty() {
            sub.push_str(&format!("        proxy_pass {};\n", s.proxy_pass));
        }
        // Custom params - prepend mode
        append_stream_param_json_prepend(conn, s, &mut sub);
        sub.push_str("    }\n\n");

        let name = format!("stream-{}", s.listen);
        let filename = sanitize_filename(&format!("{}.conf", name));
        add_sub_file(sub_files, &filename, &sub);
        out.push_str(&format!("    include conf.d/{};\n", filename));
    }
    out.push_str("}\n\n");
    Ok(())
}

// ============ Extra helpers needed for the generator ============

pub fn get_cert_by_id(conn: &Connection, id: &str) -> rusqlite::Result<Option<NginxCert>> {
    let mut stmt = conn.prepare("SELECT * FROM nginx_certs WHERE id = ?1")?;
    match stmt.query_row(rusqlite::params![id], row_to_nginx_cert) {
        Ok(cert) => Ok(Some(cert)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: set up an in-memory DB with all tables and a test preset.
    fn setup_test_db() -> (Connection, String) {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::init_db(&conn).unwrap();
        let preset_id = "test-preset-001";
        let now = "2025-01-01T00:00:00Z";
        conn.execute(
            "INSERT INTO nginx_presets (id, name, serverId, configPath, description, groupName, isActive, createdAt, updatedAt)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![preset_id, "test-preset", "server-1", "/etc/nginx/nginx.conf",
             "Test preset", "default", 1, now, now],
        ).unwrap();
        (conn, preset_id.to_string())
    }

    #[test]
    fn test_generate_empty_db() {
        // DB with just a preset, no basic settings or servers
        let (conn, preset_id) = setup_test_db();
        let result = generate_nginx_config(&conn, &preset_id).unwrap();
        // Should have basic settings section and http block
        assert!(result.contains("http {"), "should generate http block");
        assert!(result.contains("}"), "should close blocks");
    }

    #[test]
    fn test_generate_basic_settings() {
        let (conn, preset_id) = setup_test_db();
        let now = "2025-01-01T00:00:00Z";
        // Add basic settings
        crate::db::nginx::add_nginx_basic_setting(
            &conn,
            &NginxBasicSetting {
                id: "bs-1".into(),
                preset_id: preset_id.clone(),
                name: "worker_processes".into(),
                value: "auto".into(),
                sort: 0,
                created_at: now.into(),
            },
        )
        .unwrap();
        crate::db::nginx::add_nginx_basic_setting(
            &conn,
            &NginxBasicSetting {
                id: "bs-2".into(),
                preset_id: preset_id.clone(),
                name: "worker_rlimit_nofile".into(),
                value: "65535".into(),
                sort: 1,
                created_at: now.into(),
            },
        )
        .unwrap();

        let result = generate_nginx_config(&conn, &preset_id).unwrap();
        assert!(
            result.contains("worker_processes auto;"),
            "should output worker_processes"
        );
        assert!(
            result.contains("worker_rlimit_nofile 65535;"),
            "should output worker_rlimit_nofile"
        );
    }

    #[test]
    fn test_generate_http_params() {
        let (conn, preset_id) = setup_test_db();
        let now = "2025-01-01T00:00:00Z";

        crate::db::nginx::add_nginx_http_param(
            &conn,
            &NginxHttpParam {
                id: "hp-1".into(),
                preset_id: preset_id.clone(),
                name: "sendfile".into(),
                value: "on".into(),
                enabled: true,
                sort: 0,
                created_at: now.into(),
            },
        )
        .unwrap();
        crate::db::nginx::add_nginx_http_param(
            &conn,
            &NginxHttpParam {
                id: "hp-2".into(),
                preset_id: preset_id.clone(),
                name: "keepalive_timeout".into(),
                value: "65".into(),
                enabled: true,
                sort: 1,
                created_at: now.into(),
            },
        )
        .unwrap();

        let result = generate_nginx_config(&conn, &preset_id).unwrap();
        assert!(result.contains("sendfile on;"), "should output sendfile");
        assert!(
            result.contains("keepalive_timeout 65;"),
            "should output keepalive_timeout"
        );
    }

    #[test]
    fn test_generate_upstream() {
        let (conn, preset_id) = setup_test_db();
        let now = "2025-01-01T00:00:00Z";

        let upstream_id = "up-1";
        crate::db::nginx::add_nginx_upstream(
            &conn,
            &NginxUpstream {
                id: upstream_id.into(),
                preset_id: preset_id.clone(),
                name: "backend".into(),
                proxy_type: 0,
                strategy: "ip_hash".into(),
                descr: "Main backend pool".into(),
                param_json: "".into(),
                sort: 0,
                created_at: now.into(),
                updated_at: now.into(),
            },
        )
        .unwrap();

        crate::db::nginx::add_nginx_upstream_server(
            &conn,
            &NginxUpstreamServer {
                id: "us-1".into(),
                upstream_id: upstream_id.into(),
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
        crate::db::nginx::add_nginx_upstream_server(
            &conn,
            &NginxUpstreamServer {
                id: "us-2".into(),
                upstream_id: upstream_id.into(),
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

        let result = generate_nginx_config(&conn, &preset_id).unwrap();
        assert!(
            result.contains("upstream backend {"),
            "should have upstream block"
        );
        assert!(result.contains("ip_hash;"), "should have strategy");
        assert!(result.contains("10.0.0.1:8080"), "should have first server");
        assert!(
            result.contains("10.0.0.2:8080"),
            "should have second server"
        );
        assert!(result.contains("weight=5"), "weight 5");
        assert!(result.contains("weight=3"), "weight 3");
        assert!(result.contains("backup"), "backup flag");
        assert!(result.contains("max_conns=100"), "max_conns 100");
    }

    #[test]
    fn test_generate_server_basic() {
        let (conn, preset_id) = setup_test_db();
        let now = "2025-01-01T00:00:00Z";

        let server_id = "srv-1";
        crate::db::nginx::add_nginx_server(
            &conn,
            &NginxServer {
                id: server_id.into(),
                preset_id: preset_id.clone(),
                proxy_type: 0,
                listen: "80".into(),
                ip: "".into(),
                def: true,
                ipv6: false,
                proxy_protocol: false,
                server_name: "example.com".into(),
                ssl: false,
                cert_id: "".into(),
                rewrite: false,
                rewrite_listen: "".into(),
                http2: 0,
                protocols: "".into(),
                password_id: "".into(),
                deny_allow: 0,
                deny_id: "".into(),
                allow_id: "".into(),
                proxy_upstream_id: "".into(),
                descr: "Main server".into(),
                enabled: true,
                sort: 0,
                param_json: "".into(),
                created_at: now.into(),
                updated_at: now.into(),
            },
        )
        .unwrap();

        let result = generate_nginx_config(&conn, &preset_id).unwrap();
        assert!(result.contains("server {"), "should have server block");
        assert!(
            result.contains("listen 80 default_server"),
            "should have listen with default_server"
        );
        assert!(
            result.contains("server_name  example.com"),
            "should have server_name"
        );
    }

    #[test]
    fn test_generate_server_ssl() {
        let (conn, preset_id) = setup_test_db();
        let now = "2025-01-01T00:00:00Z";

        let cert_id = "cert-1";
        crate::db::nginx::add_nginx_cert(
            &conn,
            &NginxCert {
                id: cert_id.into(),
                preset_id: preset_id.clone(),
                name: "test-cert".into(),
                pem: "/etc/nginx/certs/fullchain.pem".into(),
                key: "/etc/nginx/certs/privkey.pem".into(),
                domain: "example.com".into(),
                created_at: now.into(),
            },
        )
        .unwrap();

        let server_id = "srv-ssl";
        crate::db::nginx::add_nginx_server(
            &conn,
            &NginxServer {
                id: server_id.into(),
                preset_id: preset_id.clone(),
                proxy_type: 0,
                listen: "443".into(),
                ip: "".into(),
                def: false,
                ipv6: false,
                proxy_protocol: false,
                server_name: "secure.example.com".into(),
                ssl: true,
                cert_id: cert_id.into(),
                rewrite: false,
                rewrite_listen: "".into(),
                http2: 0,
                protocols: "TLSv1.2 TLSv1.3".into(),
                password_id: "".into(),
                deny_allow: 0,
                deny_id: "".into(),
                allow_id: "".into(),
                proxy_upstream_id: "".into(),
                descr: "SSL server".into(),
                enabled: true,
                sort: 0,
                param_json: "".into(),
                created_at: now.into(),
                updated_at: now.into(),
            },
        )
        .unwrap();

        let result = generate_nginx_config(&conn, &preset_id).unwrap();
        assert!(
            result.contains("listen 443 ssl"),
            "should have ssl on listen"
        );
        assert!(
            result.contains("ssl_certificate"),
            "should have ssl_certificate"
        );
        assert!(
            result.contains("fullchain.pem"),
            "should reference pem file"
        );
        assert!(result.contains("privkey.pem"), "should reference key file");
        assert!(
            result.contains("ssl_protocols"),
            "should have ssl_protocols"
        );
        assert!(result.contains("TLSv1.2"), "should have TLSv1.2");
    }

    #[test]
    fn test_generate_server_with_locations() {
        let (conn, preset_id) = setup_test_db();
        let now = "2025-01-01T00:00:00Z";

        let upstream_id = "up-api";
        crate::db::nginx::add_nginx_upstream(
            &conn,
            &NginxUpstream {
                id: upstream_id.into(),
                preset_id: preset_id.clone(),
                name: "api".into(),
                proxy_type: 0,
                strategy: "".into(),
                descr: "".into(),
                param_json: "".into(),
                sort: 0,
                created_at: now.into(),
                updated_at: now.into(),
            },
        )
        .unwrap();

        let server_id = "srv-loc";
        crate::db::nginx::add_nginx_server(
            &conn,
            &NginxServer {
                id: server_id.into(),
                preset_id: preset_id.clone(),
                proxy_type: 0,
                listen: "80".into(),
                ip: "".into(),
                def: false,
                ipv6: false,
                proxy_protocol: false,
                server_name: "loc-test.example.com".into(),
                ssl: false,
                cert_id: "".into(),
                rewrite: false,
                rewrite_listen: "".into(),
                http2: 0,
                protocols: "".into(),
                password_id: "".into(),
                deny_allow: 0,
                deny_id: "".into(),
                allow_id: "".into(),
                proxy_upstream_id: "".into(),
                descr: "".into(),
                enabled: true,
                sort: 0,
                param_json: "".into(),
                created_at: now.into(),
                updated_at: now.into(),
            },
        )
        .unwrap();

        // Location: proxy_pass
        // Use raw SQL because add_nginx_location has a column/parameter mismatch
        conn.execute(
            "INSERT INTO nginx_locations (id, serverId, enabled, path, locType, value,
             upstreamType, upstreamId, upstreamPath, rootPath, rootPage, rootType,
             header, websocket, cros, headerHost, returnUrl, returnPath, paramJson, sort, descr, createdAt)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22)",
            rusqlite::params![
                "loc-1", server_id, 1, "/", 0, "",
                0, upstream_id, "/", "", "", "",
                1, 1, 0, "", "", 0, "", 0, "root location", now
            ],
        ).unwrap();

        // Location: root
        conn.execute(
            "INSERT INTO nginx_locations (id, serverId, enabled, path, locType, value,
             upstreamType, upstreamId, upstreamPath, rootPath, rootPage, rootType,
             header, websocket, cros, headerHost, returnUrl, returnPath, paramJson, sort, descr, createdAt)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22)",
            rusqlite::params![
                "loc-2", server_id, 1, "/static", 0, "",
                1, "", "", "/var/www/static", "index.html", "",
                0, 0, 0, "", "", 0, "", 1, "static files", now
            ],
        ).unwrap();

        // Location: return
        conn.execute(
            "INSERT INTO nginx_locations (id, serverId, enabled, path, locType, value,
             upstreamType, upstreamId, upstreamPath, rootPath, rootPage, rootType,
             header, websocket, cros, headerHost, returnUrl, returnPath, paramJson, sort, descr, createdAt)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22)",
            rusqlite::params![
                "loc-3", server_id, 1, "/old", 0, "301",
                4, "", "", "", "", "",
                0, 0, 0, "", "https://new.example.com", 1, "", 2, "redirect", now
            ],
        ).unwrap();

        let result = generate_nginx_config(&conn, &preset_id).unwrap();
        // Check the server block exists
        assert!(result.contains("server {"), "should have server block");
        assert!(result.contains("location / {"), "should have root location");
        assert!(result.contains("proxy_pass"), "should have proxy_pass");
        assert!(
            result.contains("proxy_http_version 1.1"),
            "websocket support"
        );
        assert!(
            result.contains("proxy_set_header Upgrade"),
            "websocket upgrade"
        );
        assert!(
            result.contains("location /static {"),
            "should have static location"
        );
        assert!(
            result.contains("root /var/www/static"),
            "should have root directive"
        );
        assert!(
            result.contains("location /old {"),
            "should have redirect location"
        );
        assert!(
            result.contains("return 301"),
            "should have return directive"
        );
    }

    #[test]
    fn test_generate_stream() {
        let (conn, preset_id) = setup_test_db();
        let now = "2025-01-01T00:00:00Z";

        crate::db::nginx::add_nginx_stream(
            &conn,
            &NginxStream {
                id: "stream-1".into(),
                preset_id: preset_id.clone(),
                listen: "1234".into(),
                proxy_upstream_id: "".into(),
                proxy_pass: "10.0.0.1:5678".into(),
                ssl: false,
                cert_id: "".into(),
                protocol: "TCP".into(),
                descr: "MySQL proxy".into(),
                enabled: true,
                sort: 0,
                param_json: "".into(),
                created_at: now.into(),
                updated_at: now.into(),
            },
        )
        .unwrap();

        let result = generate_nginx_config(&conn, &preset_id).unwrap();
        assert!(result.contains("stream {"), "should have stream block");
        assert!(
            result.contains("listen 1234;"),
            "should have listen directive"
        );
        assert!(
            result.contains("proxy_pass 10.0.0.1:5678;"),
            "should have proxy_pass"
        );
    }

    #[test]
    fn test_generate_decomposed() {
        let (conn, preset_id) = setup_test_db();
        let now = "2025-01-01T00:00:00Z";

        // Add a server
        let server_id = "srv-dec";
        crate::db::nginx::add_nginx_server(
            &conn,
            &NginxServer {
                id: server_id.into(),
                preset_id: preset_id.clone(),
                proxy_type: 0,
                listen: "80".into(),
                ip: "".into(),
                def: false,
                ipv6: false,
                proxy_protocol: false,
                server_name: "decomposed.example.com".into(),
                ssl: false,
                cert_id: "".into(),
                rewrite: false,
                rewrite_listen: "".into(),
                http2: 0,
                protocols: "".into(),
                password_id: "".into(),
                deny_allow: 0,
                deny_id: "".into(),
                allow_id: "".into(),
                proxy_upstream_id: "".into(),
                descr: "".into(),
                enabled: true,
                sort: 0,
                param_json: "".into(),
                created_at: now.into(),
                updated_at: now.into(),
            },
        )
        .unwrap();

        let result = generate_nginx_config_decomposed(&conn, &preset_id).unwrap();
        assert!(
            result.main_config.contains("http {"),
            "main config should have http block"
        );
        assert!(
            result.main_config.contains("include conf.d/"),
            "main config should include subfiles"
        );
        assert!(
            !result.sub_files.is_empty(),
            "should have at least one subfile"
        );
        let server_file = result
            .sub_files
            .iter()
            .find(|f| f.filename.contains("decomposed.example.com"));
        assert!(
            server_file.is_some(),
            "should have a subfile for the server"
        );
        if let Some(sf) = server_file {
            assert!(
                sf.content.contains("server {"),
                "subfile should contain server block"
            );
        }
    }

    #[test]
    fn test_generate_server_with_ipv6() {
        let (conn, preset_id) = setup_test_db();
        let now = "2025-01-01T00:00:00Z";

        let server_id = "srv-ipv6";
        crate::db::nginx::add_nginx_server(
            &conn,
            &NginxServer {
                id: server_id.into(),
                preset_id: preset_id.clone(),
                proxy_type: 0,
                listen: "80".into(),
                ip: "".into(),
                def: false,
                ipv6: true,
                proxy_protocol: false,
                server_name: "ipv6.example.com".into(),
                ssl: false,
                cert_id: "".into(),
                rewrite: false,
                rewrite_listen: "".into(),
                http2: 0,
                protocols: "".into(),
                password_id: "".into(),
                deny_allow: 0,
                deny_id: "".into(),
                allow_id: "".into(),
                proxy_upstream_id: "".into(),
                descr: "".into(),
                enabled: true,
                sort: 0,
                param_json: "".into(),
                created_at: now.into(),
                updated_at: now.into(),
            },
        )
        .unwrap();

        let result = generate_nginx_config(&conn, &preset_id).unwrap();
        assert!(result.contains("listen [::]:80"), "should have IPv6 listen");
    }

    #[test]
    fn test_generate_server_with_http2_new_style() {
        let (conn, preset_id) = setup_test_db();
        let now = "2025-01-01T00:00:00Z";

        let cert_id = "cert-h2";
        crate::db::nginx::add_nginx_cert(
            &conn,
            &NginxCert {
                id: cert_id.into(),
                preset_id: preset_id.clone(),
                name: "h2-cert".into(),
                pem: "/etc/ssl/certs/h2.pem".into(),
                key: "/etc/ssl/certs/h2.key".into(),
                domain: "h2.example.com".into(),
                created_at: now.into(),
            },
        )
        .unwrap();

        let server_id = "srv-h2";
        crate::db::nginx::add_nginx_server(
            &conn,
            &NginxServer {
                id: server_id.into(),
                preset_id: preset_id.clone(),
                proxy_type: 0,
                listen: "443".into(),
                ip: "".into(),
                def: false,
                ipv6: false,
                proxy_protocol: false,
                server_name: "h2.example.com".into(),
                ssl: true,
                cert_id: cert_id.into(),
                rewrite: false,
                rewrite_listen: "".into(),
                http2: 2,
                protocols: "".into(),
                password_id: "".into(),
                deny_allow: 0,
                deny_id: "".into(),
                allow_id: "".into(),
                proxy_upstream_id: "".into(),
                descr: "".into(),
                enabled: true,
                sort: 0,
                param_json: "".into(),
                created_at: now.into(),
                updated_at: now.into(),
            },
        )
        .unwrap();

        let result = generate_nginx_config(&conn, &preset_id).unwrap();
        assert!(
            result.contains("http2 on;"),
            "new-style http2 should output 'http2 on;'"
        );
    }

    #[test]
    fn test_generate_server_with_auth_basic() {
        let (conn, preset_id) = setup_test_db();
        let now = "2025-01-01T00:00:00Z";

        let pw_id = "pw-1";
        crate::db::nginx::add_nginx_password(
            &conn,
            &NginxPassword {
                id: pw_id.into(),
                preset_id: preset_id.clone(),
                name: "test-pw".into(),
                pass: "".into(),
                descr: "Restricted Area".into(),
                path: "/etc/nginx/.htpasswd".into(),
                created_at: now.into(),
            },
        )
        .unwrap();

        let server_id = "srv-auth";
        crate::db::nginx::add_nginx_server(
            &conn,
            &NginxServer {
                id: server_id.into(),
                preset_id: preset_id.clone(),
                proxy_type: 0,
                listen: "80".into(),
                ip: "".into(),
                def: false,
                ipv6: false,
                proxy_protocol: false,
                server_name: "auth.example.com".into(),
                ssl: false,
                cert_id: "".into(),
                rewrite: false,
                rewrite_listen: "".into(),
                http2: 0,
                protocols: "".into(),
                password_id: pw_id.into(),
                deny_allow: 0,
                deny_id: "".into(),
                allow_id: "".into(),
                proxy_upstream_id: "".into(),
                descr: "".into(),
                enabled: true,
                sort: 0,
                param_json: "".into(),
                created_at: now.into(),
                updated_at: now.into(),
            },
        )
        .unwrap();

        let result = generate_nginx_config(&conn, &preset_id).unwrap();
        assert!(
            result.contains("auth_basic"),
            "should have auth_basic directive"
        );
        assert!(
            result.contains("Restricted Area"),
            "should contain realm description"
        );
        assert!(
            result.contains("auth_basic_user_file"),
            "should have auth_basic_user_file"
        );
        assert!(
            result.contains(".htpasswd"),
            "should reference htpasswd file"
        );
    }

    #[test]
    fn test_generate_server_with_post_param_override() {
        // Server with param_json containing custom proxy params
        let (conn, preset_id) = setup_test_db();
        let now = "2025-01-01T00:00:00Z";

        let server_id = "srv-param";
        crate::db::nginx::add_nginx_server(
            &conn,
            &NginxServer {
                id: server_id.into(),
                preset_id: preset_id.clone(),
                proxy_type: 0,
                listen: "80".into(),
                ip: "".into(),
                def: false,
                ipv6: false,
                proxy_protocol: false,
                server_name: "param.example.com".into(),
                ssl: false,
                cert_id: "".into(),
                rewrite: false,
                rewrite_listen: "".into(),
                http2: 0,
                protocols: "".into(),
                password_id: "".into(),
                deny_allow: 0,
                deny_id: "".into(),
                allow_id: "".into(),
                proxy_upstream_id: "".into(),
                descr: "".into(),
                enabled: true,
                sort: 0,
                param_json: r#"[{"name":"client_max_body_size","value":"100M","position":1}]"#
                    .into(),
                created_at: now.into(),
                updated_at: now.into(),
            },
        )
        .unwrap();

        let result = generate_nginx_config(&conn, &preset_id).unwrap();
        assert!(
            result.contains("client_max_body_size 100M;"),
            "should include custom param from param_json"
        );
    }

    #[test]
    fn test_parse_ports_basic() {
        let ports = parse_ports("80");
        assert_eq!(ports, vec!["80"]);

        let ports2 = parse_ports("443");
        assert_eq!(ports2, vec!["443"]);
    }

    #[test]
    fn test_parse_ports_with_host() {
        let ports = parse_ports("127.0.0.1:8080");
        assert_eq!(ports, vec!["127.0.0.1:8080"]);
    }

    #[test]
    fn test_parse_ports_range() {
        let ports = parse_ports("8080-8082");
        assert_eq!(ports, vec!["8080", "8081", "8082"]);
    }

    #[test]
    fn test_parse_ports_comma_separated() {
        let ports = parse_ports("80,443");
        assert_eq!(ports, vec!["80", "443"]);
    }

    #[test]
    fn test_parse_ports_empty() {
        let ports = parse_ports("");
        // Should return the original string when no splitting can be done
        // but the logic pushes the original listen if result is empty
        // Actually: empty → splits to nothing → pushes original ""
        assert_eq!(ports, vec![""]);
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("hello world"), "hello_world");
        assert_eq!(sanitize_filename("foo/bar:baz"), "foo_bar_baz");
        assert_eq!(sanitize_filename("simple"), "simple");
    }

    #[test]
    fn test_generate_disabled_server_skipped() {
        let (conn, preset_id) = setup_test_db();
        let now = "2025-01-01T00:00:00Z";

        let server_id = "srv-disabled";
        crate::db::nginx::add_nginx_server(
            &conn,
            &NginxServer {
                id: server_id.into(),
                preset_id: preset_id.clone(),
                proxy_type: 0,
                listen: "80".into(),
                ip: "".into(),
                def: false,
                ipv6: false,
                proxy_protocol: false,
                server_name: "disabled.example.com".into(),
                ssl: false,
                cert_id: "".into(),
                rewrite: false,
                rewrite_listen: "".into(),
                http2: 0,
                protocols: "".into(),
                password_id: "".into(),
                deny_allow: 0,
                deny_id: "".into(),
                allow_id: "".into(),
                proxy_upstream_id: "".into(),
                descr: "".into(),
                enabled: false,
                sort: 0,
                param_json: "".into(),
                created_at: now.into(),
                updated_at: now.into(),
            },
        )
        .unwrap();

        let result = generate_nginx_config(&conn, &preset_id).unwrap();
        assert!(
            !result.contains("disabled.example.com"),
            "disabled server should not appear in output"
        );
    }

    #[test]
    fn test_generate_tcp_udp_proxy_not_in_http_block() {
        let (conn, preset_id) = setup_test_db();
        let now = "2025-01-01T00:00:00Z";

        let server_id = "srv-tcp";
        crate::db::nginx::add_nginx_server(
            &conn,
            &NginxServer {
                id: server_id.into(),
                preset_id: preset_id.clone(),
                proxy_type: 1,
                listen: "3306".into(),
                ip: "".into(),
                def: false,
                ipv6: false,
                proxy_protocol: false,
                server_name: "".into(),
                ssl: false,
                cert_id: "".into(),
                rewrite: false,
                rewrite_listen: "".into(),
                http2: 0,
                protocols: "".into(),
                password_id: "".into(),
                deny_allow: 0,
                deny_id: "".into(),
                allow_id: "".into(),
                proxy_upstream_id: "db-upstream".into(),
                descr: "MySQL proxy".into(),
                enabled: true,
                sort: 0,
                param_json: "".into(),
                created_at: now.into(),
                updated_at: now.into(),
            },
        )
        .unwrap();

        // TCP/UDP proxy servers (proxy_type != 0) are filtered out of the http block
        // by `if s.proxy_type != 0 { continue; }` in append_http_block
        let result = generate_nginx_config(&conn, &preset_id).unwrap();
        assert!(
            !result.contains("listen 3306"),
            "TCP proxy servers should NOT appear in http block output"
        );
    }
}
