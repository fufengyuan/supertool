/// Nginx Config Generator
///
/// Reads structured data from DB and generates a complete nginx configuration text.
use crate::db::nginx::*;
use rusqlite::Connection;

/// Generate the full nginx config for a preset
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

fn append_basic_settings(conn: &Connection, preset_id: &str, out: &mut String) -> Result<(), String> {
    let basic = get_basic_setting(conn, preset_id).map_err(|e| e.to_string())?;
    if let Some(b) = basic {
        out.push_str(&format!("worker_processes  {};\n", b.worker_processes));
        out.push_str(&format!("error_log  {} {};\n", b.error_log, b.error_log_level));
        out.push_str(&format!("pid        {};\n", b.pid));
        out.push_str("events {\n");
        if !b.events.is_empty() {
            for line in b.events.lines() {
                out.push_str(&format!("    {}\n", line));
            }
        } else {
            out.push_str(&format!("    worker_connections  {};\n", b.worker_connections));
        }
        out.push_str("}\n\n");
    } else {
        // Defaults
        out.push_str("worker_processes  auto;\n");
        out.push_str("error_log  /var/log/nginx/error.log warn;\n");
        out.push_str("pid        /var/run/nginx.pid;\n");
        out.push_str("events {\n    worker_connections  1024;\n}\n\n");
    }
    Ok(())
}

fn append_http_block(conn: &Connection, preset_id: &str, out: &mut String) -> Result<(), String> {
    out.push_str("http {\n");
    out.push_str("    include       /etc/nginx/mime.types;\n");
    out.push_str("    default_type  application/octet-stream;\n\n");

    // HTTP-level params
    let params = get_http_params_by_preset(conn, preset_id).map_err(|e| e.to_string())?;
    for p in &params {
        if p.enabled {
            out.push_str(&format!("    {} {};\n", p.name, p.value));
        }
    }
    if !params.is_empty() {
        out.push_str("\n");
    }

    // Upstreams
    let upstreams = get_upstreams_by_preset(conn, preset_id).map_err(|e| e.to_string())?;
    for u in &upstreams {
        if u.proxy_type != 0 { continue; } // only HTTP upstreams in http block
        append_upstream(conn, u, out)?;
    }

    // Servers
    let servers = get_servers_by_preset(conn, preset_id).map_err(|e| e.to_string())?;
    for s in &servers {
        if !s.enabled { continue; }
        if s.proxy_type != 0 { continue; } // only HTTP servers in http block
        append_server_block(conn, s, out)?;
    }

    out.push_str("}\n\n");
    Ok(())
}

fn append_upstream(conn: &Connection, u: &NginxUpstream, out: &mut String) -> Result<(), String> {
    out.push_str(&format!("    upstream {} {{\n", u.name));
    if u.strategy == "ip_hash" {
        out.push_str("        ip_hash;\n");
    } else if u.strategy == "least_conn" {
        out.push_str("        least_conn;\n");
    } else if u.strategy == "random" {
        out.push_str("        random;\n");
    }

    let servers = crate::db::nginx::get_upstream_servers(conn, &u.id)
        .map_err(|e| e.to_string())?;
    for srv in &servers {
        out.push_str(&format!("        server {}:{}", srv.address, srv.port));
        if srv.weight > 1 { out.push_str(&format!(" weight={}", srv.weight)); }
        if srv.max_fails != 3 { out.push_str(&format!(" max_fails={}", srv.max_fails)); }
        if !srv.fail_timeout.is_empty() && srv.fail_timeout != "10s" {
            out.push_str(&format!(" fail_timeout={}", srv.fail_timeout));
        }
        if srv.max_conns > 0 { out.push_str(&format!(" max_conns={}", srv.max_conns)); }
        if srv.backup { out.push_str(" backup"); }
        if srv.down { out.push_str(" down"); }
        out.push_str(";\n");
    }

    // Extra params
    if !u.param_json.is_empty() {
        if let Ok(extras) = serde_json::from_str::<Vec<serde_json::Value>>(&u.param_json) {
            for extra in &extras {
                let name = extra.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let value = extra.get("value").and_then(|v| v.as_str()).unwrap_or("");
                if !name.is_empty() {
                    out.push_str(&format!("        {} {};\n", name, value));
                }
            }
        }
    }

    out.push_str("    }\n\n");
    Ok(())
}

fn append_server_block(conn: &Connection, s: &NginxServer, out: &mut String) -> Result<(), String> {
    // Build listen directive
    let mut listen_parts: Vec<String> = Vec::new();
    if s.ssl {
        listen_parts.push(format!("{} ssl", s.listen));
        if s.http2 == 1 || s.http2 == 2 { listen_parts[0].push_str(" http2"); }
        if s.proxy_protocol { listen_parts[0].push_str(" proxy_protocol"); }
    } else {
        listen_parts.push(s.listen.clone());
        if s.proxy_protocol { listen_parts[0].push_str(" proxy_protocol"); }
    }
    if s.def { listen_parts[0].push_str(" default_server"); }
    if s.ipv6 { listen_parts.push(format!("[::]:{}", s.listen)); }

    for lp in &listen_parts {
        out.push_str(&format!("    server {{\n        listen {};\n", lp));
    }
    if !s.ip.is_empty() {
        // Replace the listen if ip is set
        // Already handled below
    }

    out.push_str(&format!("        server_name  {};\n", if s.server_name.is_empty() { "_" } else { &s.server_name }));

    // SSL certs
    if s.ssl {
        // Look up cert
        if !s.cert_id.is_empty() {
            if let Ok(Some(cert)) = get_cert_by_id(conn, &s.cert_id) {
                out.push_str(&format!("        ssl_certificate      {};\n", cert.pem));
                out.push_str(&format!("        ssl_certificate_key  {};\n", cert.key));
            }
        }
        if !s.protocols.is_empty() {
            out.push_str(&format!("        ssl_protocols       {};\n", s.protocols.replace(",", " ")));
        }
        if s.http2 == 2 {
            // New style — already handled in listen
        }
    }

    // HTTP->HTTPS redirect
    if s.rewrite && s.ssl {
        out.push_str("        return 301 https://$server_name$request_uri;\n");
        out.push_str("    }\n\n");
        // Add a second server block for the redirect listener
        out.push_str(&format!("    server {{\n        listen {};\n", if s.rewrite_listen.is_empty() { "80" } else { &s.rewrite_listen }));
        out.push_str(&format!("        server_name  {};\n", if s.server_name.is_empty() { "_" } else { &s.server_name }));
        out.push_str("        return 301 https://$server_name$request_uri;\n");
        out.push_str("    }\n\n");
        return Ok(());
    }

    // Root / Index from server level
    // Locations
    let locations = crate::db::nginx::get_locations_by_server(conn, &s.id)
        .map_err(|e| e.to_string())?;
    let mut has_root = false;

    for loc in &locations {
        if !loc.enabled { continue; }
        match loc.loc_type {
            0 => { // proxy_pass
                out.push_str(&format!("        location {} {{\n", loc.path));
                if loc.websocket {
                    out.push_str("            proxy_http_version 1.1;\n");
                    out.push_str("            proxy_set_header Upgrade $http_upgrade;\n");
                    out.push_str("            proxy_set_header Connection \"upgrade\";\n");
                }
                if loc.header {
                    out.push_str("            proxy_set_header Host $host;\n");
                    out.push_str("            proxy_set_header X-Real-IP $remote_addr;\n");
                    out.push_str("            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;\n");
                    out.push_str("            proxy_set_header X-Forwarded-Proto $scheme;\n");
                }
                if loc.cros {
                    out.push_str("            add_header Access-Control-Allow-Origin *;\n");
                    out.push_str("            add_header Access-Control-Allow-Methods \"GET, POST, OPTIONS\";\n");
                    out.push_str("            add_header Access-Control-Allow-Headers \"DNT,User-Agent,X-Requested-With,If-Modified-Since,Cache-Control,Content-Type,Range,Authorization\";\n");
                }
                if loc.upstream_type == 1 && !loc.upstream_id.is_empty() {
                    // Manual upstream reference
                    out.push_str(&format!("            proxy_pass http://{};\n", loc.upstream_id));
                } else if !loc.value.is_empty() {
                    out.push_str(&format!("            proxy_pass {};\n", loc.value));
                } else if !loc.upstream_id.is_empty() {
                    let upstream_name = get_upstream_name(conn, &loc.upstream_id);
                    out.push_str(&format!("            proxy_pass http://{};\n", upstream_name));
                }
                // Extra params
                append_param_json(loc, out);
                out.push_str("        }\n\n");
            }
            1 => { // root
                out.push_str(&format!("        location {} {{\n", loc.path));
                out.push_str(&format!("            root {}/{};\n", loc.root_path, loc.root_page));
                if !loc.value.is_empty() {
                    out.push_str(&format!("            index {};\n", loc.value));
                }
                append_param_json(loc, out);
                out.push_str("        }\n\n");
                has_root = true;
            }
            2 => { // upstream (selectable)
                out.push_str(&format!("        location {} {{\n", loc.path));
                if loc.websocket {
                    out.push_str("            proxy_http_version 1.1;\n");
                    out.push_str("            proxy_set_header Upgrade $http_upgrade;\n");
                    out.push_str("            proxy_set_header Connection \"upgrade\";\n");
                }
                if loc.header {
                    out.push_str("            proxy_set_header Host $host;\n");
                    out.push_str("            proxy_set_header X-Real-IP $remote_addr;\n");
                    out.push_str("            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;\n");
                }
                let upstream_name = get_upstream_name(conn, &loc.upstream_id);
                if !loc.upstream_path.is_empty() {
                    out.push_str(&format!("            proxy_pass http://{}{};\n", upstream_name, loc.upstream_path));
                } else {
                    out.push_str(&format!("            proxy_pass http://{};\n", upstream_name));
                }
                append_param_json(loc, out);
                out.push_str("        }\n\n");
            }
            3 => {} // blank — just a location placeholder
            4 => { // return
                let ret_url = if loc.return_path {
                    format!("{}$request_uri", loc.return_url)
                } else {
                    loc.return_url.clone()
                };
                out.push_str(&format!("        location {} {{\n", loc.path));
                out.push_str(&format!("            return {} {};\n", if loc.value.is_empty() { "302" } else { &loc.value }, ret_url));
                out.push_str("        }\n\n");
            }
            _ => {}
        }
    }

    // Server-level root/index if no location has root
    // Check server-level root from server-level directives
    // (not stored in our model — use param_json for server-level extra directives)

    out.push_str("    }\n\n");
    Ok(())
}

fn append_param_json(loc: &NginxLocation, out: &mut String) {
    if !loc.param_json.is_empty() {
        if let Ok(extras) = serde_json::from_str::<Vec<serde_json::Value>>(&loc.param_json) {
            for extra in &extras {
                let name = extra.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let value = extra.get("value").and_then(|v| v.as_str()).unwrap_or("");
                if !name.is_empty() {
                    out.push_str(&format!("            {} {};\n", name, value));
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
    let streams = crate::db::nginx::get_streams_by_preset(conn, preset_id)
        .map_err(|e| e.to_string())?;
    if streams.is_empty() { return Ok(()); }

    out.push_str("stream {\n");
    for s in &streams {
        if !s.enabled { continue; }
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
        out.push_str("    }\n\n");
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
