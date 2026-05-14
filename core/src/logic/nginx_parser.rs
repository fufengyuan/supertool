/// Nginx Config Parser
///
/// Reverse of `nginx_generator.rs`. Parses raw nginx configuration text
/// back into structured data that can be saved to the database.
///
/// Uses a two-phase approach:
/// 1. Tokenize the input into tokens (words, strings, braces, semicolons)
/// 2. Recursive-descent parse into a directive tree
/// 3. Analyze the tree to extract Settings / Upstreams / Servers / Locations / Streams
use serde::{Deserialize, Serialize};

// ── Parsed Data Types ──────────────────────────────────────────────

/// The complete result of parsing an nginx configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedNginxConfig {
    pub basic_settings: Vec<ParsedBasicSetting>,
    pub http_params: Vec<ParsedHttpParam>,
    pub upstreams: Vec<ParsedUpstream>,
    pub servers: Vec<ParsedServer>,
    pub streams: Vec<ParsedStream>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedBasicSetting {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedHttpParam {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedUpstream {
    pub name: String,
    pub strategy: String,
    pub descr: String,
    pub servers: Vec<ParsedUpstreamServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedUpstreamServer {
    pub address: String,
    pub port: i64,
    pub weight: i64,
    pub max_fails: i64,
    pub fail_timeout: String,
    pub max_conns: i64,
    pub backup: bool,
    pub down: bool,
    pub param: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedServer {
    pub proxy_type: i64,
    pub listen: String,
    pub ip: String,
    pub def: bool,
    pub ipv6: bool,
    pub proxy_protocol: bool,
    pub server_name: String,
    pub ssl: i64,
    pub cert_id: String,
    pub pem: String,
    pub key: String,
    pub rewrite: bool,
    pub rewrite_listen: String,
    pub http2: i64,
    pub protocols: String,
    pub password_id: String,
    pub deny_allow: i64,
    pub deny_id: String,
    pub allow_id: String,
    pub proxy_upstream_id: String,
    pub descr: String,
    pub locations: Vec<ParsedLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedLocation {
    pub path: String,
    pub loc_type: String,
    pub value: String,
    pub root_path: String,
    pub upstream_id: String,
    pub upstream_path: String,
    pub header: bool,
    pub websocket: bool,
    pub cros: bool,
    pub return_url: String,
    pub descr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedStream {
    pub listen: String,
    pub proxy_pass: String,
    pub proxy_upstream_id: String,
    pub ssl: i64,
    pub cert_id: String,
    pub protocol: String,
    pub descr: String,
}

// ── Tokenizer ──────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Word(String),
    Semicolon,
    LeftBrace,
    RightBrace,
    Eof,
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let c = chars[i];

        // Whitespace
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // Comment
        if c == '#' {
            while i < len && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // Braces and semicolon
        if c == '{' {
            tokens.push(Token::LeftBrace);
            i += 1;
            continue;
        }
        if c == '}' {
            tokens.push(Token::RightBrace);
            i += 1;
            continue;
        }
        if c == ';' {
            tokens.push(Token::Semicolon);
            i += 1;
            continue;
        }

        // Quoted string
        if c == '\'' || c == '"' {
            let quote = c;
            let mut s = String::new();
            i += 1; // skip opening quote
            while i < len {
                if chars[i] == '\\' && i + 1 < len {
                    s.push(chars[i + 1]);
                    i += 2;
                } else if chars[i] == quote {
                    i += 1; // skip closing quote
                    break;
                } else {
                    s.push(chars[i]);
                    i += 1;
                }
            }
            tokens.push(Token::Word(s));
            continue;
        }

        // Regular word
        let mut word = String::new();
        while i < len && !chars[i].is_whitespace() && chars[i] != '{' && chars[i] != '}' && chars[i] != ';' && chars[i] != '#' {
            word.push(chars[i]);
            i += 1;
        }
        if !word.is_empty() {
            tokens.push(Token::Word(word));
        }
    }

    tokens.push(Token::Eof);
    tokens
}

// ── Directive Tree ────────────────────────────────────────────────

/// A parsed nginx directive: either a simple statement or a block.
#[derive(Debug, Clone)]
struct Directive {
    name: String,
    args: Vec<String>,         // arguments before ; or {
    block: Vec<Directive>,     // nested directives (empty for simple directives)
    is_block: bool,            // true if this directive has { ... }
}

/// Parse tokens into a list of top-level directives.
fn parse_directives(tokens: &[Token], pos: &mut usize) -> Result<Vec<Directive>, String> {
    let mut directives = Vec::new();
    loop {
        if *pos >= tokens.len() {
            return Err("Unexpected end of input".to_string());
        }
        match &tokens[*pos] {
            Token::Eof | Token::RightBrace => break,
            _ => {
                let dir = parse_one_directive(tokens, pos)?;
                directives.push(dir);
            }
        }
    }
    Ok(directives)
}

/// Parse a single directive: WORD+ (';' | '{' directives '}')
fn parse_one_directive(tokens: &[Token], pos: &mut usize) -> Result<Directive, String> {
    // First token must be a Word (directive name)
    let name = match &tokens[*pos] {
        Token::Word(w) => {
            *pos += 1;
            w.clone()
        }
        t => {
            return Err(format!(
                "Expected directive name at token {} but found {:?}",
                *pos, t
            ));
        }
    };

    // Collect arguments until we hit ';' or '{'
    let mut args = Vec::new();
    let mut is_block = false;

    loop {
        if *pos >= tokens.len() {
            return Err(format!("Unexpected end of input after '{}'", name));
        }
        match &tokens[*pos] {
            Token::Semicolon => {
                *pos += 1;
                break;
            }
            Token::LeftBrace => {
                *pos += 1;
                is_block = true;
                let block = parse_directives(tokens, pos)?;
                // Expect RightBrace
                if *pos >= tokens.len() || tokens[*pos] != Token::RightBrace {
                    return Err(format!("Expected '}}' after block for '{}'", name));
                }
                *pos += 1;
                return Ok(Directive {
                    name,
                    args,
                    block,
                    is_block: true,
                });
            }
            Token::Word(w) => {
                args.push(w.clone());
                *pos += 1;
            }
            t => {
                return Err(format!(
                    "Unexpected token {:?} in '{}' directive",
                    t, name
                ));
            }
        }
    }

    Ok(Directive {
        name,
        args,
        block: if is_block {
            Vec::new() // populated above
        } else {
            Vec::new()
        },
        is_block,
    })
}

// ── Analyzer ──────────────────────────────────────────────────────

/// Analyze a directive tree and extract structured nginx data.
fn analyze_config(directives: &[Directive]) -> ParsedNginxConfig {
    let mut config = ParsedNginxConfig {
        basic_settings: Vec::new(),
        http_params: Vec::new(),
        upstreams: Vec::new(),
        servers: Vec::new(),
        streams: Vec::new(),
    };

    let mut i = 0;
    while i < directives.len() {
        let d = &directives[i];

        if d.name == "http" && d.is_block {
            analyze_http_block(&d.block, &mut config);
        } else if d.name == "stream" && d.is_block {
            analyze_stream_block(&d.block, &mut config);
        } else if d.name == "events" && d.is_block {
            // Treat events block as a basic setting: "events { ... }"
            let block_text = directives_to_text(&d.block, 0);
            config.basic_settings.push(ParsedBasicSetting {
                name: "events".to_string(),
                value: format!("{{\n{}    }}", block_text),
            });
        } else if d.name == "upstream" && d.is_block {
            // Top-level upstream
            if let Some(up) = parse_upstream(d) {
                config.upstreams.push(up);
            }
        } else if d.name == "server" && d.is_block {
            // Top-level server (rare but possible)
            if let Some(srv) = parse_server_block(d) {
                config.servers.push(srv);
            }
        } else if d.name == "include" || d.name == "load_module" {
            config.basic_settings.push(ParsedBasicSetting {
                name: d.name.clone(),
                value: d.args.join(" "),
            });
        } else if !d.is_block {
            // Simple directive — basic setting
            let value = d.args.join(" ");
            if !value.is_empty() || d.name == "pid" || d.name == "error_log" || d.name == "worker_processes" || d.name == "worker_rlimit_nofile" {
                config.basic_settings.push(ParsedBasicSetting {
                    name: d.name.clone(),
                    value,
                });
            }
        }

        i += 1;
    }

    config
}

/// Analyze http block directives.
fn analyze_http_block(dirs: &[Directive], config: &mut ParsedNginxConfig) {
    for d in dirs {
        if d.name == "upstream" && d.is_block {
            if let Some(up) = parse_upstream(d) {
                config.upstreams.push(up);
            }
        } else if d.name == "server" && d.is_block {
            if let Some(srv) = parse_server_block(d) {
                config.servers.push(srv);
            }
        } else if d.name == "include" || d.name == "default_type" {
            config.http_params.push(ParsedHttpParam {
                name: d.name.clone(),
                value: d.args.join(" "),
            });
        } else if !d.is_block {
            let value = d.args.join(" ");
            if !value.is_empty() {
                config.http_params.push(ParsedHttpParam {
                    name: d.name.clone(),
                    value,
                });
            }
        }
        // Block directives that aren't upstream/server (like geo, map) — skip
    }
}

/// Analyze stream block directives.
fn analyze_stream_block(dirs: &[Directive], config: &mut ParsedNginxConfig) {
    for d in dirs {
        if d.name == "server" && d.is_block {
            if let Some(s) = parse_stream_server(d) {
                config.streams.push(s);
            }
        }
    }
}

/// Parse an upstream block.
fn parse_upstream(d: &Directive) -> Option<ParsedUpstream> {
    if d.name != "upstream" || !d.is_block {
        return None;
    }
    let name = d.args.first()?.clone();
    let mut strategy = String::new();
    let mut descr = String::new();
    let mut servers = Vec::new();

    for child in &d.block {
        if child.name == "server" && !child.is_block {
            // upstream server directive: server address:port params;
            let params = &child.args;
            if let Some(addr_port) = params.first() {
                let (address, port) = split_addr_port(addr_port);
                let mut srv = ParsedUpstreamServer {
                    address,
                    port,
                    weight: 1,
                    max_fails: 3,
                    fail_timeout: String::new(),
                    max_conns: 0,
                    backup: false,
                    down: false,
                    param: String::new(),
                };
                // Parse extra params (both "key=value" and "key value" formats)
                let mut i = 1;
                while i < params.len() {
                    let p = &params[i];
                    // Try key=value format first
                    if let Some(eq_idx) = p.find('=') {
                        let key = &p[..eq_idx];
                        let val = &p[eq_idx + 1..];
                        match key {
                            "weight" => srv.weight = val.parse().unwrap_or(1),
                            "max_fails" => srv.max_fails = val.parse().unwrap_or(3),
                            "fail_timeout" => srv.fail_timeout = val.to_string(),
                            "max_conns" => srv.max_conns = val.parse().unwrap_or(0),
                            _ => {
                                if !srv.param.is_empty() { srv.param.push(' '); }
                                srv.param.push_str(p);
                            }
                        }
                    } else if *p == "backup" {
                        srv.backup = true;
                    } else if *p == "down" {
                        srv.down = true;
                    } else if *p == "weight" && i + 1 < params.len() {
                        srv.weight = params[i + 1].parse().unwrap_or(1);
                        i += 1;
                    } else if *p == "max_fails" && i + 1 < params.len() {
                        srv.max_fails = params[i + 1].parse().unwrap_or(3);
                        i += 1;
                    } else if *p == "fail_timeout" && i + 1 < params.len() {
                        srv.fail_timeout = params[i + 1].clone();
                        i += 1;
                    } else if *p == "max_conns" && i + 1 < params.len() {
                        srv.max_conns = params[i + 1].parse().unwrap_or(0);
                        i += 1;
                    } else {
                        if !srv.param.is_empty() { srv.param.push(' '); }
                        srv.param.push_str(p);
                    }
                    i += 1;
                }
                servers.push(srv);
            }
        } else if child.name == "ip_hash" {
            strategy = "ip_hash".to_string();
        } else if child.name == "least_conn" {
            strategy = "least_conn".to_string();
        } else if child.name == "random" {
            strategy = "random".to_string();
        } else if child.name == "sticky" {
            strategy = "sticky".to_string();
        } else if child.name == "least_time" {
            strategy = "least_time".to_string();
        } else if child.name.starts_with('#') {
            // Comment-like directives (not standard, but we can store descr)
        }
    }

    Some(ParsedUpstream {
        name,
        strategy,
        descr,
        servers,
    })
}

/// Parse a server block.
fn parse_server_block(d: &Directive) -> Option<ParsedServer> {
    if d.name != "server" || !d.is_block {
        return None;
    }
    let mut srv = ParsedServer {
        proxy_type: 0,
        listen: String::new(),
        ip: String::new(),
        def: false,
        ipv6: false,
        proxy_protocol: false,
        server_name: String::new(),
        ssl: 0,
        cert_id: String::new(),
        pem: String::new(),
        key: String::new(),
        rewrite: false,
        rewrite_listen: String::new(),
        http2: 0,
        protocols: String::new(),
        password_id: String::new(),
        deny_allow: 0,
        deny_id: String::new(),
        allow_id: String::new(),
        proxy_upstream_id: String::new(),
        descr: String::new(),
        locations: Vec::new(),
    };

    for child in &d.block {
        match child.name.as_str() {
            "listen" => {
                // listen 80 ssl http2 default_server proxy_protocol;
                // listen [::]:80;
                let mut listen_val = String::new();
                let mut is_ipv6 = false;
                let mut has_ssl = false;
                let mut has_http2 = false;

                for (idx, arg) in child.args.iter().enumerate() {
                    if idx == 0 {
                        if arg.starts_with("[::]") {
                            is_ipv6 = true;
                            listen_val = arg.trim_start_matches("[::]:").to_string();
                        } else if arg.contains(':') {
                            // ip:port
                            if let Some(colon_idx) = arg.rfind(':') {
                                srv.ip = arg[..colon_idx].to_string();
                                listen_val = arg[colon_idx + 1..].to_string();
                            } else {
                                listen_val = arg.clone();
                            }
                        } else {
                            listen_val = arg.clone();
                        }
                    } else {
                        match arg.as_str() {
                            "default_server" => srv.def = true,
                            "proxy_protocol" => srv.proxy_protocol = true,
                            "ssl" => has_ssl = true,
                            "http2" => has_http2 = true,
                            _ => {}
                        }
                    }
                }

                if is_ipv6 {
                    srv.ipv6 = true;
                    // IPv6 listen uses the same port
                    if srv.listen.is_empty() {
                        srv.listen = listen_val;
                    }
                } else {
                    srv.listen = listen_val;
                }

                if has_ssl {
                    srv.ssl = 1;
                }
                if has_http2 {
                    if srv.ssl == 1 {
                        srv.http2 = 1; // old-style http2 on ssl listen
                    } else {
                        srv.http2 = 2; // new-style http2 on;
                    }
                }
            }
            "server_name" => {
                srv.server_name = child.args.join(" ");
            }
            "ssl_certificate" => {
                srv.pem = child.args.join(" ");
                if srv.cert_id.is_empty() {
                    srv.cert_id = "imported".to_string();
                }
            }
            "ssl_certificate_key" => {
                srv.key = child.args.join(" ");
                if srv.cert_id.is_empty() {
                    srv.cert_id = "imported".to_string();
                }
            }
            "ssl_protocols" => {
                srv.protocols = child.args.join(" ");
            }
            "http2" => {
                if child.args.first().map(|s| s == "on").unwrap_or(false) {
                    srv.http2 = 2;
                }
            }
            "auth_basic" => {
                // auth_basic "description"; or auth_basic off;
                if !child.args.is_empty() && child.args[0] != "off" {
                    // Store descr from auth_basic value
                    // We won't try to match password_id here
                }
            }
            "auth_basic_user_file" => {
                // Could try to locate password by path, but for import we just note it
            }
            "deny" => {
                if child.args.first().map(|s| s == "all").unwrap_or(false) {
                    // deny all;
                    if srv.deny_allow != 2 {
                        srv.deny_allow = 1; // blacklist mode
                    }
                }
            }
            "allow" => {
                if child.args.first().map(|s| s == "all").unwrap_or(false) {
                    // allow all;
                    if srv.deny_allow == 0 {
                        srv.deny_allow = 2; // whitelist mode
                    }
                }
            }
            "location" => {
                if child.is_block {
                    if let Some(loc) = parse_location(child) {
                        srv.locations.push(loc);
                    }
                }
            }
            "return" => {
                // return 301 https://$host$request_uri;
                // This could be a rewrite redirect
                if let Some(code) = child.args.first() {
                    if code == "301" || code == "302" || code == "303" || code == "307" || code == "308" {
                        srv.rewrite = true;
                        if child.args.len() > 1 {
                            // The rewrite URL — we can't easily extract the port
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Some(srv)
}

/// Parse a location block.
fn parse_location(d: &Directive) -> Option<ParsedLocation> {
    if d.name != "location" || !d.is_block {
        return None;
    }
    let path = d.args.first().cloned().unwrap_or_default();
    let mut loc = ParsedLocation {
        path,
        loc_type: String::new(),
        value: String::new(),
        root_path: String::new(),
        upstream_id: String::new(),
        upstream_path: String::new(),
        header: false,
        websocket: false,
        cros: false,
        return_url: String::new(),
        descr: String::new(),
    };

    for child in &d.block {
        match child.name.as_str() {
            "proxy_pass" => {
                let proxy = child.args.join(" ");
                if proxy.starts_with("http://") || proxy.starts_with("https://") || proxy.starts_with("uwsgi://") || proxy.starts_with("fastcgi://") {
                    // Extract upstream name from URL
                    let rest = proxy.trim_start_matches("http://").trim_start_matches("https://").trim_start_matches("uwsgi://").trim_start_matches("fastcgi://");
                    if let Some(slash_idx) = rest.find('/') {
                        loc.upstream_id = rest[..slash_idx].to_string();
                        loc.upstream_path = rest[slash_idx..].to_string();
                    } else {
                        loc.upstream_id = rest.to_string();
                        loc.upstream_path = "/".to_string();
                    }
                    loc.loc_type = "proxy_pass".to_string();
                    loc.value = proxy;
                } else {
                    loc.loc_type = "proxy_pass".to_string();
                    loc.value = proxy;
                }
            }
            "root" => {
                loc.loc_type = "root".to_string();
                loc.root_path = child.args.join(" ");
            }
            "return" => {
                loc.loc_type = "return".to_string();
                loc.return_url = child.args.join(" ");
            }
            "proxy_set_header" => {
                if child.args.first().map(|s| s == "Host").unwrap_or(false) {
                    loc.header = true;
                } else if child.args.first().map(|s| s == "Upgrade").unwrap_or(false)
                    || child.args.first().map(|s| s == "Connection").unwrap_or(false)
                {
                    // WebSocket headers
                }
            }
            "proxy_http_version" => {
                // 1.1 — websocket support
            }
            "proxy_set_body" => {}
            "add_header" => {}
            _ => {}
        }
    }

    // Detect websocket: proxy_set_header Upgrade $http_upgrade;
    for child in &d.block {
        if child.name == "proxy_set_header" {
            if let Some(first) = child.args.first() {
                if first == "Upgrade" {
                    loc.websocket = true;
                }
            }
        }
    }

    // Detect CORS headers
    for child in &d.block {
        if child.name == "add_header" {
            if let Some(first) = child.args.first() {
                if first.contains("Access-Control-") {
                    loc.cros = true;
                }
            }
        }
    }

    Some(loc)
}

/// Parse a stream server block.
fn parse_stream_server(d: &Directive) -> Option<ParsedStream> {
    if d.name != "server" || !d.is_block {
        return None;
    }
    let mut s = ParsedStream {
        listen: String::new(),
        proxy_pass: String::new(),
        proxy_upstream_id: String::new(),
        ssl: 0,
        cert_id: String::new(),
        protocol: String::new(),
        descr: String::new(),
    };

    for child in &d.block {
        match child.name.as_str() {
            "listen" => {
                s.listen = child.args.join(" ");
            }
            "proxy_pass" => {
                let pass = child.args.join(" ");
                s.proxy_pass = pass.clone();
                // Check if it references an upstream (not ip:port)
                if !pass.contains(':') {
                    s.proxy_upstream_id = pass;
                }
            }
            "ssl" => {
                if child.args.first().map(|a| a == "on").unwrap_or(false) {
                    s.ssl = 1;
                }
            }
            "ssl_certificate" => {
                s.cert_id = "imported".to_string();
            }
            "protocol" => {
                s.protocol = child.args.join(" ");
            }
            _ => {}
        }
    }

    Some(s)
}

/// Split "address:port" or "address" into (address, port).
fn split_addr_port(s: &str) -> (String, i64) {
    // Handle IPv6: [::1]:8080
    if s.starts_with('[') {
        if let Some(close) = s.find(']') {
            let addr = s[1..close].to_string();
            if close + 1 < s.len() && s.as_bytes()[close + 1] == b':' {
                let port: i64 = s[close + 2..].parse().unwrap_or(80);
                return (addr, port);
            }
            return (addr, 80);
        }
    }

    if let Some(colon_idx) = s.rfind(':') {
        let addr = s[..colon_idx].to_string();
        let port: i64 = s[colon_idx + 1..].parse().unwrap_or(80);
        (addr, port)
    } else {
        (s.to_string(), 80)
    }
}

/// Convert a list of directives back to text (with given indent level).
fn directives_to_text(dirs: &[Directive], indent: usize) -> String {
    let mut out = String::new();
    let ind = "    ".repeat(indent);
    for d in dirs {
        if d.is_block {
            out.push_str(&format!("{}{} {};\n", ind, d.name, d.args.join(" ")));
        } else {
            out.push_str(&format!("{}{} {};\n", ind, d.name, d.args.join(" ")));
        }
    }
    out
}

// ── Public API ─────────────────────────────────────────────────────

/// Parse a complete nginx configuration text and return structured data.
pub fn parse_nginx_config(config_text: &str) -> Result<ParsedNginxConfig, String> {
    let tokens = tokenize(config_text);
    let mut pos = 0;
    let directives = parse_directives(&tokens, &mut pos)?;
    Ok(analyze_config(&directives))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let tokens = tokenize("worker_processes auto;");
        assert!(tokens.len() >= 3);
        assert_eq!(tokens[0], Token::Word("worker_processes".to_string()));
        assert_eq!(tokens[1], Token::Word("auto".to_string()));
        assert_eq!(tokens[2], Token::Semicolon);
    }

    #[test]
    fn test_tokenize_block() {
        let tokens = tokenize("http { server { listen 80; } }");
        let words: Vec<_> = tokens.iter().filter_map(|t| {
            if let Token::Word(w) = t { Some(w.as_str()) } else { None }
        }).collect();
        assert!(words.contains(&"http"));
        assert!(words.contains(&"server"));
        assert!(words.contains(&"listen"));
        assert!(words.contains(&"80"));
    }

    #[test]
    fn test_parse_upstream() {
        let text = "upstream backend {\n    server 10.0.0.1:8080 weight=5 max_fails=3;\n}";
        let config = parse_nginx_config(text).unwrap();
        assert_eq!(config.upstreams.len(), 1);
        assert_eq!(config.upstreams[0].name, "backend");
        assert_eq!(config.upstreams[0].servers.len(), 1);
        assert_eq!(config.upstreams[0].servers[0].address, "10.0.0.1");
        assert_eq!(config.upstreams[0].servers[0].port, 8080);
        assert_eq!(config.upstreams[0].servers[0].weight, 5);
    }

    #[test]
    fn test_parse_server() {
        let text = "server {\n    listen 80 default_server;\n    server_name example.com;\n    location /api {\n        proxy_pass http://backend/api/;\n    }\n}";
        let config = parse_nginx_config(text).unwrap();
        assert_eq!(config.servers.len(), 1);
        assert_eq!(config.servers[0].listen, "80");
        assert!(config.servers[0].def);
        assert_eq!(config.servers[0].server_name, "example.com");
        assert_eq!(config.servers[0].locations.len(), 1);
        assert_eq!(config.servers[0].locations[0].path, "/api");
    }

    #[test]
    fn test_parse_full_config() {
        let text = r#"
worker_processes auto;
events {
    worker_connections 1024;
}
http {
    include mime.types;
    default_type application/octet-stream;
    upstream backend {
        server 127.0.0.1:8080 weight=5;
    }
    server {
        listen 80;
        server_name example.com;
        location / {
            proxy_pass http://backend;
        }
    }
}
stream {
    server {
        listen 1234;
        proxy_pass 10.0.0.1:5678;
    }
}
"#;
        let config = parse_nginx_config(text).unwrap();
        assert!(!config.basic_settings.is_empty(), "should have basic settings");
        assert!(!config.http_params.is_empty(), "should have http params");
        assert_eq!(config.upstreams.len(), 1);
        assert_eq!(config.servers.len(), 1);
        assert_eq!(config.streams.len(), 1);
    }
}
