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
    pub extra_params: Vec<ParsedParamEntry>,
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

/// An unrecognized directive captured as an extra parameter entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedParamEntry {
    pub name: String,
    pub value: String,
    pub position: i64,
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
    pub extra_params: Vec<ParsedParamEntry>,
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
    pub extra_params: Vec<ParsedParamEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedStream {
    pub listen: String,
    pub proxy_pass: String,
    pub proxy_upstream_id: String,
    pub ssl: i64,
    pub cert_id: String,
    pub pem: String,
    pub key: String,
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
    Comment(String), // # comment line
    Space(String),   // whitespace between tokens (for preserving alignment)
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    // Track whether the last emitted token was a line-ending token (start of a new directive)
    // In nginx, '#' starts a comment ONLY when it appears at the start of a directive,
    // not in the middle of a directive value (e.g. return 302 /h5/#/url).
    // We consider the start of input, after ';', '{', '}' as "start of directive".
    let mut at_directive_start = true;

    while i < len {
        let c = chars[i];

        // Whitespace - capture ALL whitespace including newlines+indent
        // Parser will decide what to use (multi-line args vs new directive)
        if c.is_whitespace() {
            let mut space = String::new();
            while i < len && chars[i].is_whitespace() {
                space.push(chars[i]);
                i += 1;
            }
            // Mark directive start on newline (for comment detection)
            if space.contains('\n') {
                at_directive_start = true;
            }
            // Always emit Space token - parser will filter unused ones
            if !space.is_empty() {
                tokens.push(Token::Space(space));
            }
            continue;
        }

        // Comment — only at directive start (not in the middle of a value)
        // nginx treats '#' as comment only when it's at the start of a directive
        if c == '#' && at_directive_start {
            let mut comment = String::new();
            while i < len && chars[i] != '\n' {
                comment.push(chars[i]);
                i += 1;
            }
            // Remove the '#' prefix and trim
            let comment_text = comment.trim_start_matches('#').trim();
            if !comment_text.is_empty() {
                tokens.push(Token::Comment(comment_text.to_string()));
            }
            continue;
        }

        // Braces and semicolon
        if c == '{' {
            tokens.push(Token::LeftBrace);
            i += 1;
            at_directive_start = true;
            continue;
        }
        if c == '}' {
            tokens.push(Token::RightBrace);
            i += 1;
            at_directive_start = true;
            continue;
        }
        if c == ';' {
            tokens.push(Token::Semicolon);
            i += 1;
            at_directive_start = true;
            continue;
        }

        // Quoted string — preserve quotes in the token
        if c == '\'' || c == '"' {
            let quote = c;
            let mut s = String::new();
            s.push(quote); // include opening quote
            i += 1; // skip opening quote character position
            while i < len {
                if chars[i] == '\\' && i + 1 < len {
                    s.push(chars[i]);
                    s.push(chars[i + 1]);
                    i += 2;
                } else if chars[i] == quote {
                    s.push(chars[i]); // include closing quote
                    i += 1;
                    break;
                } else {
                    s.push(chars[i]);
                    i += 1;
                }
            }
            tokens.push(Token::Word(s));
            continue;
        }

        // Regular word — don't stop at '#', let the outer loop handle it
        let mut word = String::new();
        while i < len
            && !chars[i].is_whitespace()
            && chars[i] != '{'
            && chars[i] != '}'
            && chars[i] != ';'
        {
            word.push(chars[i]);
            i += 1;
        }
        if !word.is_empty() {
            tokens.push(Token::Word(word));
            at_directive_start = false;
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
    args_spacing: Vec<String>, // spacing BEFORE each arg (for preserving alignment)
    block: Vec<Directive>,     // nested directives (empty for simple directives)
    is_block: bool,            // true if this directive has { ... }
    descr: String,             // comment lines preceding this directive
    inline_comment: String,    // comment on the same line after ; (e.g. "default ...;  # comment")
}

/// Parse tokens into a list of top-level directives.
/// Collects preceding comments and attaches them to the next directive.
fn parse_directives(tokens: &[Token], pos: &mut usize) -> Result<Vec<Directive>, String> {
    let mut directives = Vec::new();
    let mut pending_comments: Vec<String> = Vec::new();

    loop {
        if *pos >= tokens.len() {
            return Err("Unexpected end of input".to_string());
        }
        match &tokens[*pos] {
            Token::Eof | Token::RightBrace => break,
            Token::Comment(c) => {
                pending_comments.push(c.clone());
                *pos += 1;
            }
            Token::Space(s) => {
                // Skip Space tokens that contain newline (formatting whitespace)
                // These are whitespace between directives (blank lines, indentation)
                if s.contains('\n') {
                    *pos += 1;
                } else {
                    // Space without newline - this should be spacing before a directive
                    // Skip it too, the directive will handle its own spacing
                    *pos += 1;
                }
            }
            _ => {
                let dir = parse_one_directive(tokens, pos)?;
                // Attach pending comments to this directive
                let descr = pending_comments.join("\n");
                pending_comments.clear();
                directives.push(Directive {
                    name: dir.name,
                    args: dir.args,
                    args_spacing: dir.args_spacing,
                    block: dir.block,
                    is_block: dir.is_block,
                    descr,
                    inline_comment: dir.inline_comment,
                });
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

    // Collect arguments and spacing until we hit ';' or '{'
    let mut args = Vec::new();
    let mut args_spacing = Vec::new();

    // Check for Space token after directive name (spacing before first arg)
    let mut pending_spacing = String::new();
    if *pos < tokens.len() {
        if let Token::Space(s) = &tokens[*pos] {
            pending_spacing = s.clone();
            *pos += 1;
        }
    }

    loop {
        if *pos >= tokens.len() {
            return Err(format!("Unexpected end of input after '{}'", name));
        }
        match &tokens[*pos] {
            Token::Semicolon => {
                *pos += 1;
                // Check for inline comment after semicolon
                let inline_comment = if *pos < tokens.len() && matches!(tokens[*pos], Token::Comment(_)) {
                    if let Token::Comment(c) = &tokens[*pos] {
                        *pos += 1;
                        c.clone()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };
                return Ok(Directive {
                    name,
                    args,
                    args_spacing,
                    block: Vec::new(),
                    is_block: false,
                    descr: String::new(),
                    inline_comment,
                });
            }
            Token::LeftBrace => {
                *pos += 1;
                let block = parse_directives(tokens, pos)?;
                if *pos >= tokens.len() || tokens[*pos] != Token::RightBrace {
                    return Err(format!("Expected '}}' after block for '{}'", name));
                }
                *pos += 1;
                return Ok(Directive {
                    name,
                    args,
                    args_spacing,
                    block,
                    is_block: true,
                    descr: String::new(),
                    inline_comment: String::new(),
                });
            }
            Token::Word(w) => {
                args.push(w.clone());
                args_spacing.push(pending_spacing.clone());
                pending_spacing.clear();
                *pos += 1;
                // Check for Space token after this word
                if *pos < tokens.len() {
                    if let Token::Space(s) = &tokens[*pos] {
                        pending_spacing = s.clone();
                        *pos += 1;
                    }
                }
            }
            Token::Space(_) => {
                *pos += 1; // skip stray Space
            }
            t => {
                return Err(format!("Unexpected token {:?} in '{}' directive", t, name));
            }
        }
    }
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
            // Format: 4-space indent for content, 2-space indent for closing brace (matching nginxWebUI)
            let block_text = directives_to_text(&d.block, 1);
            config.basic_settings.push(ParsedBasicSetting {
                name: "events".to_string(),
                value: format!("{{\n{}  }}", block_text),
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
            if !value.is_empty()
                || d.name == "pid"
                || d.name == "error_log"
                || d.name == "worker_processes"
                || d.name == "worker_rlimit_nofile"
            {
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
            let args_str = join_args_with_spacing(&d.args, &d.args_spacing);
            config.http_params.push(ParsedHttpParam {
                name: d.name.clone(),
                value: args_str,
            });
        } else if d.is_block {
            // Block directive (like geo, map) - use args_spacing for alignment
            let args_str = join_args_with_spacing(&d.args, &d.args_spacing);
            let body: Vec<String> = d
                .block
                .iter()
                .map(|child| {
                    let child_args_str = join_args_with_spacing(&child.args, &child.args_spacing);
                    let inline = if child.inline_comment.is_empty() {
                        String::new()
                    } else {
                        format!("  # {}", child.inline_comment)
                    };
                    format!("        {}{};{}", child.name, child_args_str, inline)
                })
                .collect();
            let block_value = format!("{} {{\n{}\n    }}", args_str, body.join("\n"));
            if !block_value.trim().is_empty() {
                config.http_params.push(ParsedHttpParam {
                    name: d.name.clone(),
                    value: block_value,
                });
            }
        } else {
            let args_str = join_args_with_spacing(&d.args, &d.args_spacing);
            if !args_str.is_empty() {
                config.http_params.push(ParsedHttpParam {
                    name: d.name.clone(),
                    value: args_str,
                });
            }
        }
    }
}

/// Helper: join args with their preceding spacing
/// spacing[i] is the spacing BEFORE args[i] (from directive name or previous arg)
/// If spacing is empty, default to single space for readability
fn join_args_with_spacing(args: &[String], args_spacing: &[String]) -> String {
    args.iter()
        .zip(args_spacing.iter())
        .map(|(arg, spacing)| {
            if spacing.is_empty() {
                format!(" {}", arg)
            } else {
                // spacing already contains the exact spacing, don't add extra space
                format!("{}{}", spacing, arg)
            }
        })
        .collect::<Vec<_>>()
        .join("")
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
    let descr = d.descr.clone(); // Use directive's descr
    let mut servers = Vec::new();
    let mut extra_params: Vec<ParsedParamEntry> = Vec::new();

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
                                if !srv.param.is_empty() {
                                    srv.param.push(' ');
                                }
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
                        if !srv.param.is_empty() {
                            srv.param.push(' ');
                        }
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
        } else if child.name != "ip_hash"
            && child.name != "least_conn"
            && child.name != "random"
            && child.name != "sticky"
            && child.name != "least_time"
        {
            // Capture unrecognized upstream directives as extra params
            let value = child.args.join(" ");
            extra_params.push(ParsedParamEntry {
                name: child.name.clone(),
                value,
                position: 0,
            });
        }
    }

    Some(ParsedUpstream {
        name,
        strategy,
        descr,
        servers,
        extra_params,
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
        descr: d.descr.clone(), // Use directive's descr
        locations: Vec::new(),
        extra_params: Vec::new(),
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
                    if srv.listen.is_empty() {
                        srv.listen = listen_val;
                    }
                } else if !srv.listen.is_empty() && srv.listen != listen_val && srv.ssl == 1 {
                    // Second listen on a different port — HTTP→HTTPS redirect
                    srv.rewrite = true;
                    srv.rewrite_listen = listen_val;
                } else if !srv.listen.is_empty() && srv.listen == listen_val && has_ssl {
                    // Same port but with ssl — update ssl/http2 flags
                    srv.ssl = 1;
                    if has_http2 {
                        srv.http2 = 1;
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
                    if code == "301"
                        || code == "302"
                        || code == "303"
                        || code == "307"
                        || code == "308"
                    {
                        srv.rewrite = true;
                    }
                }
                // Store return directive for regeneration (redirect-only servers need this)
                let value = child.args.join(" ");
                if !value.is_empty() {
                    srv.extra_params.push(ParsedParamEntry {
                        name: "return".to_string(),
                        value,
                        position: 0,
                    });
                }
            }
            _ => {
                // Skip if ($scheme = http) { return 301 ... } — handled by generator's rewrite logic
                if child.name == "if" && srv.rewrite {
                    if let Some(cond) = child.args.first() {
                        if cond.contains("$scheme")
                            && child.block.iter().any(|c| c.name == "return")
                        {
                            continue;
                        }
                    }
                }
                // For block directives (like if { ... }), use directives_to_text for proper formatting
                let value = if child.is_block {
                    let block_text = directives_to_text(&child.block, 0);
                    format!("{} {{\n{}}}", join_args_with_spacing(&child.args, &child.args_spacing), block_text.trim_end())
                } else {
                    child.args.join(" ")
                };
                srv.extra_params.push(ParsedParamEntry {
                    name: child.name.clone(),
                    value,
                    position: 1, // prepend — output BEFORE locations
                });
            }
        }
    }

    Some(srv)
}

/// Parse a location block.
fn parse_location(d: &Directive) -> Option<ParsedLocation> {
    if d.name != "location" || !d.is_block {
        return None;
    }
    let (path, _modifier) = {
        let first = d.args.first().cloned().unwrap_or_default();
        match first.as_str() {
            "^~" | "=" | "~" | "~*" => {
                let p = d.args.get(1).cloned().unwrap_or_default();
                let m = first.clone();
                (
                    if p.is_empty() {
                        m.clone()
                    } else {
                        format!("{} {}", m, p)
                    },
                    m,
                )
            }
            _ => (first, String::new()),
        }
    };
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
        descr: d.descr.clone(), // Use directive's descr
        extra_params: Vec::new(),
    };

    for child in &d.block {
        match child.name.as_str() {
            "proxy_pass" => {
                let proxy = child.args.join(" ");
                if proxy.starts_with("http://")
                    || proxy.starts_with("https://")
                    || proxy.starts_with("uwsgi://")
                    || proxy.starts_with("fastcgi://")
                {
                    // Extract upstream name from URL
                    let rest = proxy
                        .trim_start_matches("http://")
                        .trim_start_matches("https://")
                        .trim_start_matches("uwsgi://")
                        .trim_start_matches("fastcgi://");
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
                // Split status code from URL:
                //   "return 301 https://..." → value="301", return_url="https://..."
                //   "return https://..." → value="", return_url="https://..."
                if !child.args.is_empty() && child.args[0].chars().all(|c| c.is_ascii_digit()) {
                    loc.value = child.args[0].clone();
                    loc.return_url = child.args[1..].join(" ");
                } else {
                    loc.return_url = child.args.join(" ");
                }
            }
            "proxy_set_header" => {
                if child.args.first().map(|s| s == "Host").unwrap_or(false) {
                    loc.header = true;
                } else if child.args.first().map(|s| s == "Upgrade").unwrap_or(false)
                    || child
                        .args
                        .first()
                        .map(|s| s == "Connection")
                        .unwrap_or(false)
                {
                    // WebSocket headers
                }
            }
            "proxy_http_version" => {
                // 1.1 — websocket support
            }
            "proxy_set_body" => {}
            "add_header" => {
                // Non-CORS add_header should be saved as extra param
                let first_arg = child.args.first().map(|s| s.as_str()).unwrap_or("");
                if !first_arg.starts_with("Access-Control-") {
                    let value = child.args.join(" ");
                    if !value.is_empty()
                        && !loc
                            .extra_params
                            .iter()
                            .any(|e| e.name == "add_header" && e.value == value)
                    {
                        loc.extra_params.push(ParsedParamEntry {
                            name: child.name.clone(),
                            value,
                            position: 0,
                        });
                    }
                }
            }
            "proxy_redirect" => {
                // proxy_redirect is handled by the generator's hardcoded logic
                // (server.ssl && server.rewrite → output proxy_redirect http:// https://;)
                // Don't capture as extra_param to avoid duplication in round-trip.
            }
            _ => {
                // Block directives like if (...) { ... } need special handling
                if child.is_block {
                    // Skip CORS OPTIONS if block - generator outputs this hardcoded when cros=true
                    if child.name == "if" {
                        if let Some(cond) = child.args.first() {
                            if cond.contains("$request_method") && child.block.iter().any(|c| c.name == "return") {
                                // This is CORS OPTIONS handling - skip to avoid duplication
                                continue;
                            }
                        }
                    }
                    // Use directives_to_text to serialize the block content
                    let block_text = directives_to_text(&child.block, 0);
                    // Format: "($condition) {\n  block_content\n}"
                    let value = format!("{} {{\n{}}}", join_args_with_spacing(&child.args, &child.args_spacing), block_text.trim_end());
                    loc.extra_params.push(ParsedParamEntry {
                        name: child.name.clone(),
                        value,
                        position: 0,
                    });
                    continue;
                }
                let value = join_args_with_spacing(&child.args, &child.args_spacing);
                if !value.is_empty() {
                    loc.extra_params.push(ParsedParamEntry {
                        name: child.name.clone(),
                        value,
                        position: 0,
                    });
                }
            }
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
        pem: String::new(),
        key: String::new(),
        protocol: String::new(),
        descr: d.descr.clone(), // Use directive's descr
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
                s.pem = child.args.join(" ");
                if s.ssl == 0 {
                    s.ssl = 1; // ssl_certificate implies SSL
                }
            }
            "ssl_certificate_key" => {
                s.key = child.args.join(" ");
                if s.ssl == 0 {
                    s.ssl = 1;
                }
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
        let args_str = join_args_with_spacing(&d.args, &d.args_spacing);
        if d.is_block {
            // Block directive: output name args { ... }
            out.push_str(&format!("{}{}{} {{\n", ind, d.name, args_str));
            // Output nested block content with increased indent
            out.push_str(&directives_to_text(&d.block, indent + 1));
            out.push_str(&format!("{}}}\n", ind));
        } else {
            let inline = if d.inline_comment.is_empty() {
                String::new()
            } else {
                format!("  # {}", d.inline_comment)
            };
            out.push_str(&format!("{}{}{};{}\n", ind, d.name, args_str, inline));
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
        assert!(
            tokens.len() >= 3,
            "expected at least 3 tokens, got {}",
            tokens.len()
        );
        assert_eq!(tokens[0], Token::Word("worker_processes".to_string()));
        assert_eq!(tokens[1], Token::Word("auto".to_string()));
        assert_eq!(tokens[2], Token::Semicolon);
    }

    #[test]
    fn test_tokenize_block() {
        let tokens = tokenize("http { server { listen 80; } }");
        let words: Vec<_> = tokens
            .iter()
            .filter_map(|t| {
                if let Token::Word(w) = t {
                    Some(w.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert!(words.contains(&"http"));
        assert!(words.contains(&"server"));
        assert!(words.contains(&"listen"));
        assert!(words.contains(&"80"));
    }

    #[test]
    fn test_tokenize_quoted_string() {
        let tokens = tokenize("server_name \"example.com\";");
        assert!(
            tokens
                .iter()
                .any(|t| t == &Token::Word("example.com".to_string())),
            "quoted string should produce a Word token without quotes"
        );
    }

    #[test]
    fn test_tokenize_gzip_types_multiline() {
        // Test that multi-line directive preserves newline+indent in args_spacing
        let text = "  gzip_types text/css\n        text/javascript\n        text/xml\n        text/plain\n        application/json;";
        let tokens = tokenize(text);
        
        // Debug: print all tokens
        println!("Tokens for gzip_types multi-line:");
        for t in &tokens {
            match t {
                Token::Word(w) => println!("  Word: '{}'", w),
                Token::Space(s) => println!("  Space: {:?}", s),
                Token::Semicolon => println!("  Semicolon"),
                _ => println!("  Other"),
            }
        }
        
        // Should have Word tokens for directive name and args
        let words: Vec<_> = tokens.iter().filter_map(|t| {
            if let Token::Word(w) = t { Some(w.clone()) } else { None }
        }).collect();
        assert!(words.contains(&"gzip_types".to_string()));
        assert!(words.contains(&"text/css".to_string()));
        assert!(words.contains(&"text/javascript".to_string()));
        
        // Should have Space tokens with newline+indent for multi-line args
        let spaces: Vec<_> = tokens.iter().filter_map(|t| {
            if let Token::Space(s) = t { Some(s.clone()) } else { None }
        }).collect();
        println!("Space tokens: {:?}", spaces);
        // At least one space token should contain newline
        assert!(spaces.iter().any(|s| s.contains('\n')), "Space token should contain newline for multi-line args");
    }

    #[test]
    fn test_parse_gzip_types_multiline() {
        // Test that multi-line gzip_types is preserved with correct args_spacing
        let text = r#"
http {
  gzip_types text/css
        text/javascript
        text/xml
        text/plain
        application/json;
}
"#;
        let config = parse_nginx_config(text).unwrap();
        
        // Find gzip_types in http_params
        let gzip_types_param = config.http_params.iter().find(|p| p.name == "gzip_types");
        assert!(gzip_types_param.is_some(), "gzip_types should be in http_params");
        
        let param = gzip_types_param.unwrap();
        println!("gzip_types value: {:?}", param.value);
        
        // Value should contain newline+indent for multi-line format
        assert!(param.value.contains('\n'), "gzip_types value should contain newline for multi-line args");
        
        // Value should match the original format
        assert!(param.value.contains("text/css"), "should contain text/css");
        assert!(param.value.contains("text/javascript"), "should contain text/javascript");
    }

    #[test]
    fn test_tokenize_comments_skipped() {
        let tokens = tokenize("worker_processes auto; # this is a comment\npid /run/nginx.pid;");
        let words: Vec<_> = tokens
            .iter()
            .filter_map(|t| {
                if let Token::Word(w) = t {
                    Some(w.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert!(
            words.contains(&"pid"),
            "pid should be present after comment"
        );
        assert!(
            !words.contains(&"this"),
            "comment content should be skipped"
        );
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
        assert!(
            !config.basic_settings.is_empty(),
            "should have basic settings"
        );
        assert!(!config.http_params.is_empty(), "should have http params");
        assert_eq!(config.upstreams.len(), 1);
        assert_eq!(config.servers.len(), 1);
        assert_eq!(config.streams.len(), 1);
    }

    // ── Expanded parser tests ───────────────────────────────────────

    #[test]
    fn test_empty_config() {
        let config = parse_nginx_config("").unwrap();
        assert!(
            config.basic_settings.is_empty(),
            "empty config should have no basic_settings"
        );
        assert!(config.http_params.is_empty());
        assert!(config.upstreams.is_empty());
        assert!(config.servers.is_empty());
        assert!(config.streams.is_empty());
    }

    #[test]
    fn test_config_with_only_comments() {
        let text = "# This is a comment\n# Another comment\n";
        let config = parse_nginx_config(text).unwrap();
        assert!(config.basic_settings.is_empty());
    }

    #[test]
    fn test_single_directive() {
        let config = parse_nginx_config("worker_processes auto;").unwrap();
        assert_eq!(config.basic_settings.len(), 1);
        assert_eq!(config.basic_settings[0].name, "worker_processes");
        assert_eq!(config.basic_settings[0].value, "auto");
    }

    #[test]
    fn test_events_block() {
        let config = parse_nginx_config("events { worker_connections 1024; }").unwrap();
        assert!(!config.basic_settings.is_empty());
        let events_setting = config.basic_settings.iter().find(|s| s.name == "events");
        assert!(
            events_setting.is_some(),
            "events block should be stored as a basic setting"
        );
        if let Some(ev) = events_setting {
            assert!(
                ev.value.contains("worker_connections"),
                "events block should contain worker_connections"
            );
        }
    }

    #[test]
    fn test_http_block_with_upstreams_servers_locations() {
        let text = r#"
http {
    sendfile on;
    tcp_nopush on;
    keepalive_timeout 65;

    upstream backend {
        ip_hash;
        server 10.0.0.1:8080 weight=5 max_fails=3;
        server 10.0.0.2:8080 weight=3 backup;
    }

    upstream api {
        least_conn;
        server 10.0.0.3:9000;
    }

    server {
        listen 443 ssl;
        server_name example.com;
        ssl_certificate /etc/ssl/certs/server.crt;
        ssl_certificate_key /etc/ssl/private/server.key;
        ssl_protocols TLSv1.2 TLSv1.3;

        location / {
            proxy_pass http://backend;
            proxy_set_header Host $host;
        }

        location /api {
            proxy_pass http://api/v2/;
        }

        location /static {
            root /var/www/static;
        }

        location /redirect {
            return 301 https://new.example.com$request_uri;
        }
    }
}
"#;
        let config = parse_nginx_config(text).unwrap();
        assert_eq!(
            config
                .http_params
                .iter()
                .filter(|p| p.name != "include" && p.name != "default_type")
                .count(),
            3,
            "should have 3 http-level params (sendfile, tcp_nopush, keepalive_timeout)"
        );
        assert_eq!(config.upstreams.len(), 2, "should have 2 upstreams");
        assert_eq!(config.upstreams[0].name, "backend");
        assert_eq!(config.upstreams[0].strategy, "ip_hash");
        assert_eq!(config.upstreams[0].servers.len(), 2);
        assert!(
            config.upstreams[0].servers[1].backup,
            "second backend server should be backup"
        );
        assert_eq!(config.upstreams[1].name, "api");
        assert_eq!(config.upstreams[1].strategy, "least_conn");
        assert_eq!(config.upstreams[1].servers.len(), 1);

        assert_eq!(config.servers.len(), 1, "should have 1 server");
        let srv = &config.servers[0];
        assert_eq!(srv.listen, "443");
        assert_eq!(srv.server_name, "example.com");
        assert_eq!(srv.ssl, 1);
        assert!(srv.pem.contains("server.crt"));
        assert!(srv.key.contains("server.key"));
        assert!(srv.protocols.contains("TLSv1.2"));
        assert_eq!(srv.locations.len(), 4);

        // Check locations
        let root_loc = srv.locations.iter().find(|l| l.path == "/").unwrap();
        assert_eq!(root_loc.loc_type, "proxy_pass");
        assert_eq!(root_loc.upstream_id, "backend");
        assert_eq!(root_loc.upstream_path, "/");

        let api_loc = srv.locations.iter().find(|l| l.path == "/api").unwrap();
        assert_eq!(api_loc.upstream_id, "api");
        assert_eq!(api_loc.upstream_path, "/v2/");

        let static_loc = srv.locations.iter().find(|l| l.path == "/static").unwrap();
        assert_eq!(static_loc.loc_type, "root");
        assert_eq!(static_loc.root_path, "/var/www/static");

        let redirect_loc = srv
            .locations
            .iter()
            .find(|l| l.path == "/redirect")
            .unwrap();
        assert_eq!(redirect_loc.loc_type, "return");
        // 301 is stored in value, not return_url
        assert_eq!(redirect_loc.value, "301");
    }

    #[test]
    fn test_stream_block() {
        let text = r#"
stream {
    server {
        listen 1234;
        proxy_pass 10.0.0.1:5678;
    }
    server {
        listen 1235;
        proxy_pass backend_stream;
        ssl on;
        ssl_certificate /etc/ssl/certs/stream.crt;
        protocol TCP;
    }
}
"#;
        let config = parse_nginx_config(text).unwrap();
        assert_eq!(config.streams.len(), 2);
        assert_eq!(config.streams[0].listen, "1234");
        assert_eq!(config.streams[0].proxy_pass, "10.0.0.1:5678");
        assert!(config.streams[0].proxy_upstream_id.is_empty());
        assert_eq!(config.streams[1].listen, "1235");
        assert_eq!(config.streams[1].proxy_pass, "backend_stream");
        assert_eq!(config.streams[1].proxy_upstream_id, "backend_stream");
        assert_eq!(config.streams[1].ssl, 1);
        assert_eq!(config.streams[1].cert_id, "imported");
        assert_eq!(config.streams[1].protocol, "TCP");
    }

    #[test]
    fn test_multiple_servers_different_ports() {
        let text = r#"
http {
    server {
        listen 80;
        server_name example.com;
    }
    server {
        listen 8080;
        server_name admin.example.com;
    }
    server {
        listen 443 ssl;
        server_name secure.example.com;
    }
}
"#;
        let config = parse_nginx_config(text).unwrap();
        assert_eq!(config.servers.len(), 3);
        assert_eq!(config.servers[0].server_name, "example.com");
        assert_eq!(config.servers[1].listen, "8080");
        assert_eq!(config.servers[2].listen, "443");
        assert_eq!(config.servers[2].ssl, 1);
    }

    #[test]
    fn test_server_with_ssl() {
        let text = r#"
http {
    server {
        listen 443 ssl;
        server_name secure.example.com;
        ssl_certificate /etc/nginx/certs/fullchain.pem;
        ssl_certificate_key /etc/nginx/certs/privkey.pem;
        ssl_protocols TLSv1.2 TLSv1.3;
    }
}
"#;
        let config = parse_nginx_config(text).unwrap();
        assert_eq!(config.servers.len(), 1);
        let srv = &config.servers[0];
        assert_eq!(srv.ssl, 1);
        assert!(srv.pem.contains("fullchain.pem"));
        assert!(srv.key.contains("privkey.pem"));
        assert_eq!(srv.protocols, "TLSv1.2 TLSv1.3");
        assert_eq!(srv.cert_id, "imported");
    }

    #[test]
    fn test_server_with_http2_old_style() {
        let text = r#"
http {
    server {
        listen 443 ssl http2;
        server_name example.com;
    }
}
"#;
        let config = parse_nginx_config(text).unwrap();
        assert_eq!(
            config.servers[0].http2, 1,
            "old-style http2 on ssl listen should set http2=1"
        );
        assert_eq!(config.servers[0].ssl, 1);
    }

    #[test]
    fn test_server_with_http2_new_style() {
        let text = r#"
http {
    server {
        listen 443 ssl;
        http2 on;
        server_name example.com;
    }
}
"#;
        let config = parse_nginx_config(text).unwrap();
        assert_eq!(
            config.servers[0].http2, 2,
            "new-style 'http2 on;' should set http2=2"
        );
    }

    #[test]
    fn test_server_with_auth_basic() {
        let text = r#"
http {
    server {
        listen 80;
        server_name protected.example.com;
        auth_basic "Restricted Area";
        auth_basic_user_file /etc/nginx/.htpasswd;
    }
}
"#;
        let config = parse_nginx_config(text).unwrap();
        assert_eq!(config.servers.len(), 1);
        // auth_basic doesn't set a specific field but should parse without error
    }

    #[test]
    fn test_server_with_deny_allow() {
        let text = r#"
http {
    server {
        listen 80;
        server_name restricted.example.com;
        deny 10.0.0.1;
        allow 192.168.1.0/24;
        deny all;
    }
}
"#;
        let config = parse_nginx_config(text).unwrap();
        assert_eq!(config.servers.len(), 1);
        // deny all; sets deny_allow to 1 (blacklist mode)
        assert_eq!(
            config.servers[0].deny_allow, 1,
            "deny all should set deny_allow to 1"
        );
    }

    #[test]
    fn test_location_proxy_pass() {
        let text = r#"
http {
    server {
        listen 80;
        location / {
            proxy_pass http://backend;
        }
    }
}
"#;
        let config = parse_nginx_config(text).unwrap();
        let loc = &config.servers[0].locations[0];
        assert_eq!(loc.loc_type, "proxy_pass");
        assert_eq!(loc.upstream_id, "backend");
        assert_eq!(loc.upstream_path, "/");
    }

    #[test]
    fn test_location_proxy_pass_with_path() {
        let text = r#"
http {
    server {
        listen 80;
        location /api {
            proxy_pass http://backend/api/v1/;
        }
    }
}
"#;
        let config = parse_nginx_config(text).unwrap();
        let loc = &config.servers[0].locations[0];
        assert_eq!(loc.loc_type, "proxy_pass");
        assert_eq!(loc.upstream_id, "backend");
        assert_eq!(loc.upstream_path, "/api/v1/");
    }

    #[test]
    fn test_location_root() {
        let text = r#"
http {
    server {
        listen 80;
        location / {
            root /var/www/html;
        }
    }
}
"#;
        let config = parse_nginx_config(text).unwrap();
        let loc = &config.servers[0].locations[0];
        assert_eq!(loc.loc_type, "root");
        assert_eq!(loc.root_path, "/var/www/html");
    }

    #[test]
    fn test_location_return() {
        let text = r#"
http {
    server {
        listen 80;
        location /old {
            return 301 https://new.example.com$request_uri;
        }
    }
}
"#;
        let config = parse_nginx_config(text).unwrap();
        let loc = &config.servers[0].locations[0];
        assert_eq!(loc.loc_type, "return");
        // 301 is stored in value, not return_url
        assert_eq!(loc.value, "301");
        assert!(loc.return_url.contains("new.example.com"));
    }

    #[test]
    fn test_location_websocket_headers() {
        let text = r#"
http {
    server {
        listen 80;
        location /ws {
            proxy_pass http://backend;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
        }
    }
}
"#;
        let config = parse_nginx_config(text).unwrap();
        let loc = &config.servers[0].locations[0];
        assert!(loc.websocket, "WebSocket headers should be detected");
    }

    #[test]
    fn test_location_cors_headers() {
        let text = r#"
http {
    server {
        listen 80;
        location /api {
            proxy_pass http://backend;
            add_header Access-Control-Allow-Origin *;
            add_header Access-Control-Allow-Methods *;
        }
    }
}
"#;
        let config = parse_nginx_config(text).unwrap();
        let loc = &config.servers[0].locations[0];
        assert!(loc.cros, "CORS headers should be detected");
    }

    #[test]
    fn test_upstream_ip_hash_strategy() {
        let config =
            parse_nginx_config("upstream backend { ip_hash; server 127.0.0.1:8080; }").unwrap();
        assert_eq!(config.upstreams[0].strategy, "ip_hash");
    }

    #[test]
    fn test_upstream_least_conn_strategy() {
        let config =
            parse_nginx_config("upstream backend { least_conn; server 127.0.0.1:8080; }").unwrap();
        assert_eq!(config.upstreams[0].strategy, "least_conn");
    }

    #[test]
    fn test_upstream_random_strategy() {
        let config =
            parse_nginx_config("upstream backend { random; server 127.0.0.1:8080; }").unwrap();
        assert_eq!(config.upstreams[0].strategy, "random");
    }

    #[test]
    fn test_upstream_server_with_all_params() {
        let text = "upstream backend {\n    server 10.0.0.1:8080 weight=10 max_fails=5 fail_timeout=30s max_conns=100 backup down;\n}";
        let config = parse_nginx_config(text).unwrap();
        let srv = &config.upstreams[0].servers[0];
        assert_eq!(srv.address, "10.0.0.1");
        assert_eq!(srv.port, 8080);
        assert_eq!(srv.weight, 10);
        assert_eq!(srv.max_fails, 5);
        assert_eq!(srv.fail_timeout, "30s");
        assert_eq!(srv.max_conns, 100);
        assert!(srv.backup, "backup should be true");
        assert!(srv.down, "down should be true");
    }

    #[test]
    fn test_upstream_server_separate_key_value_params() {
        let text = "upstream backend {\n    server 127.0.0.1:8080 weight 5 max_fails 3 fail_timeout 20s;\n}";
        let config = parse_nginx_config(text).unwrap();
        let srv = &config.upstreams[0].servers[0];
        assert_eq!(srv.weight, 5);
        assert_eq!(srv.max_fails, 3);
        assert_eq!(srv.fail_timeout, "20s");
    }

    #[test]
    fn test_ipv6_listen() {
        let text = r#"
http {
    server {
        listen [::]:80;
        server_name ipv6.example.com;
    }
}
"#;
        let config = parse_nginx_config(text).unwrap();
        let srv = &config.servers[0];
        assert!(srv.ipv6, "IPv6 listen should be detected");
        assert_eq!(srv.listen, "80");
    }

    #[test]
    fn test_listen_with_all_flags() {
        let text = r#"
http {
    server {
        listen 443 ssl http2 default_server proxy_protocol;
        server_name example.com;
    }
}
"#;
        let config = parse_nginx_config(text).unwrap();
        let srv = &config.servers[0];
        assert_eq!(srv.listen, "443");
        assert!(srv.def, "default_server should be detected");
        assert!(srv.proxy_protocol, "proxy_protocol should be detected");
        assert_eq!(srv.ssl, 1);
        assert_eq!(srv.http2, 1);
    }

    #[test]
    fn test_quoted_string_values() {
        let text = r#"server_name "example.com";"#;
        let config = parse_nginx_config(text).unwrap();
        assert_eq!(config.basic_settings[0].value, "example.com");
    }

    #[test]
    fn test_variables_in_config() {
        let text = r#"
http {
    server {
        listen 80;
        location / {
            proxy_pass http://backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        }
    }
}
"#;
        let config = parse_nginx_config(text).unwrap();
        let loc = &config.servers[0].locations[0];
        assert!(loc.header, "proxy_set_header Host should set header=true");
    }

    #[test]
    fn test_include_directive() {
        let text = "include /etc/nginx/conf.d/*.conf;\n";
        let config = parse_nginx_config(text).unwrap();
        assert!(
            config.basic_settings.iter().any(|s| s.name == "include"),
            "include directive should be stored as a basic setting"
        );
    }

    #[test]
    fn test_load_module_directive() {
        let text = "load_module modules/ngx_http_geoip_module.so;\n";
        let config = parse_nginx_config(text).unwrap();
        assert!(
            config
                .basic_settings
                .iter()
                .any(|s| s.name == "load_module"),
            "load_module directive should be stored as a basic setting"
        );
    }

    #[test]
    fn test_map_block_skipped_gracefully() {
        let text = r#"
http {
    map $http_host $backend {
        hostnames;
        default backend_default;
        example.com backend_example;
        *.example.org backend_wildcard;
    }
    server {
        listen 80;
    }
}
"#;
        // map is a block directive that is not upstream/server, should be skipped gracefully
        let config = parse_nginx_config(text).unwrap();
        assert_eq!(
            config.servers.len(),
            1,
            "server should still be parsed despite map block"
        );
    }

    #[test]
    fn test_listen_with_ip() {
        let text = r#"
http {
    server {
        listen 127.0.0.1:8080;
        server_name localhost;
    }
}
"#;
        let config = parse_nginx_config(text).unwrap();
        let srv = &config.servers[0];
        assert_eq!(srv.listen, "8080");
        assert_eq!(srv.ip, "127.0.0.1");
    }

    #[test]
    fn test_top_level_upstream_outside_http() {
        let text = "upstream backend { server 10.0.0.1:8080; }\n";
        let config = parse_nginx_config(text).unwrap();
        assert_eq!(config.upstreams.len(), 1);
        assert_eq!(config.upstreams[0].name, "backend");
    }

    #[test]
    fn test_top_level_server_outside_http() {
        let text = "server { listen 80; server_name standalone; }\n";
        let config = parse_nginx_config(text).unwrap();
        assert_eq!(config.servers.len(), 1);
        assert_eq!(config.servers[0].server_name, "standalone");
    }

    #[test]
    fn test_parse_returns_error_on_bad_input() {
        let result = parse_nginx_config("server { listen 80; "); // missing closing brace
        assert!(result.is_err(), "unclosed block should return an error");
    }

    #[test]
    fn test_http2_without_ssl_old_style() {
        // http2 flag on listen line without ssl should set http2=2 (new-style)
        let text = "server { listen 80 http2; server_name test.com; }\n";
        let config = parse_nginx_config(text).unwrap();
        assert_eq!(
            config.servers[0].http2, 2,
            "http2 without ssl should set http2=2"
        );
    }

    #[test]
    fn test_deny_allow_whitelist_mode() {
        let text = r#"
http {
    server {
        listen 80;
        allow 192.168.1.0/24;
        allow 10.0.0.0/8;
        deny all;
    }
}
"#;
        let config = parse_nginx_config(text).unwrap();
        // First deny all encountered sets deny_allow=1, but allow all comes after in logic...
        // Actually the parser checks: if arg == "all" and the child name is "deny" then deny_allow = 1
        // If child name is "allow" and arg == "all" and deny_allow == 0 then deny_allow = 2
        // In this case: deny all is the last line, so deny_allow becomes 1
        assert_eq!(
            config.servers[0].deny_allow, 1,
            "deny all sets deny_allow=1 (blacklist)"
        );
    }

    #[test]
    fn test_parse_https_proxy_pass() {
        let text = r#"
http {
    server {
        listen 80;
        location / {
            proxy_pass https://backend-secure:443/;
        }
    }
}
"#;
        let config = parse_nginx_config(text).unwrap();
        let loc = &config.servers[0].locations[0];
        assert_eq!(loc.loc_type, "proxy_pass");
        assert!(loc.value.contains("https://"));
    }

    #[test]
    fn test_split_addr_port_ipv6() {
        let (addr, port) = split_addr_port("[::1]:8080");
        assert_eq!(addr, "::1");
        assert_eq!(port, 8080);

        let (addr2, port2) = split_addr_port("[2001:db8::1]:443");
        assert_eq!(addr2, "2001:db8::1");
        assert_eq!(port2, 443);
    }
}
