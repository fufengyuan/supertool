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
            (format!("{}:", &listen[..idx]), listen[idx+1..].to_string())
        }
    } else {
        ("".to_string(), listen.to_string())
    };

    // Split by comma and expand ranges
    for part in port_part.split(',') {
        let part = part.trim();
        if part.is_empty() { continue; }
        if let Some(idx) = part.find('-') {
            let start: u16 = part[..idx].parse().unwrap_or(80);
            let end: u16 = part[idx+1..].parse().unwrap_or(start);
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
pub fn generate_nginx_config_decomposed(conn: &Connection, preset_id: &str) -> Result<NginxConfigResult, String> {
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

fn append_basic_settings(conn: &Connection, preset_id: &str, out: &mut String) -> Result<(), String> {
    let settings = get_basic_settings_by_preset(conn, preset_id).map_err(|e| e.to_string())?;
    for s in &settings {
        if !s.name.is_empty() {
            out.push_str(&format!("{} {};\n", s.name, s.value));
        }
    }
    out.push_str("\n");
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

/// Decomposed version: extract upstreams and server blocks into separate sub-files
fn append_http_block_decomposed(
    conn: &Connection,
    preset_id: &str,
    out: &mut String,
    sub_files: &mut Vec<NginxSubFile>,
) -> Result<(), String> {
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

    // Upstreams (decomposed into separate files)
    let upstreams = get_upstreams_by_preset(conn, preset_id).map_err(|e| e.to_string())?;
    for u in &upstreams {
        if u.proxy_type != 0 { continue; }
        let mut sub = String::new();
        append_upstream(conn, u, &mut sub)?;
        let filename = sanitize_filename(&format!("http-upstream-{}.conf", u.name));
        add_sub_file(sub_files, &filename, &sub);
        out.push_str(&format!("    include conf.d/{};\n", filename));
    }

    // Servers (decomposed into separate files)
    let servers = get_servers_by_preset(conn, preset_id).map_err(|e| e.to_string())?;
    for s in &servers {
        if !s.enabled { continue; }
        if s.proxy_type != 0 { continue; }
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
        if !srv.enabled { continue; }
        out.push_str(&format!("        server {}:{}", srv.address, srv.port));
        if srv.weight > 1 { out.push_str(&format!(" weight={}", srv.weight)); }
        if srv.max_fails != 3 { out.push_str(&format!(" max_fails={}", srv.max_fails)); }
        if !srv.fail_timeout.is_empty() && srv.fail_timeout != "10s" {
            out.push_str(&format!(" fail_timeout={}", srv.fail_timeout));
        }
        if srv.max_conns > 0 { out.push_str(&format!(" max_conns={}", srv.max_conns)); }
        if srv.backup { out.push_str(" backup"); }
        if srv.down { out.push_str(" down"); }
        if !srv.param.is_empty() { out.push_str(&format!(" {}", srv.param)); }
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
            if s.def { listen_val += " default_server"; }
            if s.proxy_protocol { listen_val += " proxy_protocol"; }
            if s.ssl {
                listen_val += " ssl";
                if s.http2 == 1 { listen_val += " http2"; } // old-style http2
            }
            out.push_str(&format!("        {};\n", listen_val));
        }
        if s.ipv6 {
            for port in &ports {
                let mut listen_ipv6 = format!("listen [::]:{}", port);
                if s.def { listen_ipv6 += " default_server"; }
                if s.proxy_protocol { listen_ipv6 += " proxy_protocol"; }
                if s.ssl { listen_ipv6 += " ssl"; }
                out.push_str(&format!("        {};\n", listen_ipv6));
            }
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
                out.push_str(&format!("        ssl_protocols       {};\n", s.protocols.replace(",", " ")));
            }
        }

        // Custom params - prepend mode
        append_param_json_prepend(s, out);

        // IP blacklist/whitelist
        if s.deny_allow > 0 {
            if !s.deny_id.is_empty() || s.deny_allow == 2 || s.deny_allow == 3 {
                if let Ok(Some(da)) = get_deny_allow_by_id(conn, if s.deny_allow == 2 { &s.allow_id } else { &s.deny_id }) {
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
            if s.deny_allow == 1 { out.push_str("        allow all;\n"); }
            if s.deny_allow == 2 { out.push_str("        deny all;\n"); }
        }

        // Locations
        let locations = crate::db::nginx::get_locations_by_server(conn, &s.id)
            .map_err(|e| e.to_string())?;

        for loc in &locations {
            if !loc.enabled { continue; }
            append_location_block(conn, loc, s, out)?;
        }

        // Custom params - append mode from paramJson
        if !s.param_json.is_empty() {
            if let Ok(extras) = serde_json::from_str::<Vec<serde_json::Value>>(&s.param_json) {
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
        if s.proxy_protocol { listen_val += " proxy_protocol"; }
        if s.proxy_type == 2 { listen_val += " udp"; }
        if s.ssl { listen_val += " ssl"; }
        out.push_str(&format!("        {};\n", listen_val));
        if s.ipv6 {
            let mut listen_ipv6 = format!("listen [::]:{}", s.listen);
            if s.proxy_protocol { listen_ipv6 += " proxy_protocol"; }
            if s.ssl { listen_ipv6 += " ssl"; }
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

fn append_location_block(conn: &Connection, loc: &NginxLocation, server: &NginxServer, out: &mut String) -> Result<(), String> {
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
    append_location_param_json_prepend(loc, out);

    match loc.loc_type {
        0 | 2 => {
            // proxy_pass (type 0) or upstream (type 2)

            // proxy_pass directive
            if loc.loc_type == 0 && !loc.value.is_empty() {
                out.push_str(&format!("            proxy_pass {};\n", loc.value));
            } else if loc.loc_type == 2 || (!loc.upstream_id.is_empty()) {
                let upstream_type = if loc.upstream_type == 1 { "https" } else { "http" };
                let upstream_name = get_upstream_name(conn, &loc.upstream_id);
                let path = if loc.upstream_path.is_empty() { "" } else { &loc.upstream_path };
                out.push_str(&format!("            proxy_pass {}://{}{};\n", upstream_type, upstream_name, path));
            } else if loc.upstream_type == 1 && !loc.upstream_id.is_empty() {
                // Manual upstream reference
                out.push_str(&format!("            proxy_pass http://{};\n", loc.upstream_id));
            }

            // Websocket support
            if loc.websocket {
                out.push_str("            proxy_http_version 1.1;\n");
                out.push_str("            proxy_set_header Upgrade $http_upgrade;\n");
                out.push_str("            proxy_set_header Connection \"upgrade\";\n");
            }

            // Header settings with configurable headerHost
            if loc.header {
                out.push_str(&format!("            proxy_set_header Host {};\n",
                    if loc.header_host.is_empty() { "$host" } else { &loc.header_host }));
                out.push_str("            proxy_set_header X-Real-IP $remote_addr;\n");
                out.push_str("            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;\n");
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
            let root_type = if loc.root_type == "alias" { "alias" } else { "root" };
            if loc.root_path.contains('$') {
                // Dynamic path — use as-is
                out.push_str(&format!("            {} {};\n", root_type, loc.root_path));
            } else {
                let path = loc.root_path.trim_end_matches('/');
                out.push_str(&format!("            {} {};\n", root_type, if root_type == "alias" { format!("{}/", path) } else { path.to_string() }));
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
            out.push_str(&format!("            return {} {};\n",
                if loc.value.is_empty() { "302" } else { &loc.value },
                ret_url
            ));
        }
        _ => {}
    }

    // Custom params - append mode
    append_location_param_json_append(loc, out);

    out.push_str("        }\n\n");
    Ok(())
}

fn append_param_json_prepend(s: &NginxServer, out: &mut String) {
    if !s.param_json.is_empty() {
        if let Ok(extras) = serde_json::from_str::<Vec<serde_json::Value>>(&s.param_json) {
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
}

fn append_location_param_json_prepend(loc: &NginxLocation, out: &mut String) {
    if !loc.param_json.is_empty() {
        if let Ok(extras) = serde_json::from_str::<Vec<serde_json::Value>>(&loc.param_json) {
            for extra in &extras {
                let pos = extra.get("position").and_then(|v| v.as_i64()).unwrap_or(0);
                if pos == 1 {
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

fn append_location_param_json_append(loc: &NginxLocation, out: &mut String) {
    if !loc.param_json.is_empty() {
        if let Ok(extras) = serde_json::from_str::<Vec<serde_json::Value>>(&loc.param_json) {
            for extra in &extras {
                let pos = extra.get("position").and_then(|v| v.as_i64()).unwrap_or(0);
                // Default position 0 = append
                if pos == 0 || pos == 2 {
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

/// Decomposed version: extract stream upstreams and servers into sub-files
fn append_stream_block_decomposed(
    conn: &Connection,
    preset_id: &str,
    out: &mut String,
    sub_files: &mut Vec<NginxSubFile>,
) -> Result<(), String> {
    let streams = crate::db::nginx::get_streams_by_preset(conn, preset_id)
        .map_err(|e| e.to_string())?;
    if streams.is_empty() { return Ok(()); }

    // Decompose stream upstreams into sub-files
    let upstreams = get_upstreams_by_preset(conn, preset_id).map_err(|e| e.to_string())?;
    for u in &upstreams {
        if u.proxy_type != 1 { continue; }
        let mut sub = String::new();
        append_upstream(conn, u, &mut sub)?;
        let filename = sanitize_filename(&format!("stream-upstream-{}.conf", u.name));
        add_sub_file(sub_files, &filename, &sub);
        out.push_str(&format!("    include conf.d/{};\n", filename));
    }

    out.push_str("stream {\n");
    for s in &streams {
        if !s.enabled { continue; }

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
