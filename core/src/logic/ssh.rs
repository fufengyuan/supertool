/// SSH Service — 连接池 + 命令执行 + SFTP + PTY 终端
///
/// 替代 Electron 的 ssh2 npm 包，使用 Rust 的 ssh2 crate（libssh2 绑定）。
/// 支持密码认证和密钥文件认证。
///
/// ## 架构设计
/// - `connections` 用 `Mutex<HashMap<String, Arc<Session>>>` — 只负责存/取/删
/// - **所有阻塞 I/O（channel_session, exec, read）都在 Mutex 之外执行**
/// - `Session` 是线程安全的（所有方法 `&self`），`Arc<Session>` 允许多线程共享
/// - 一个连接卡住（如网络断开）**不会**影响其他服务器或全局功能
/// - TCP 读写超时 + SSH keepalive + 输出大小限制，三重防卡死
use serde::{Deserialize, Serialize};
use ssh2::{Channel, Session, Sftp};
use std::collections::HashMap;
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// 命令执行结果
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecResult {
    pub success: bool,
    pub output: String,
    pub error_output: String,
    pub exit_code: Option<u32>,
}

/// SFTP 文件信息
#[derive(Debug, Serialize)]
pub struct SftpFile {
    pub name: String,
    #[serde(rename = "type")]
    pub file_type: String,
    pub size: u64,
    #[serde(rename = "modifyTime")]
    pub modify_time: String,
    pub permissions: String,
}

/// 服务器连接配置
#[derive(Debug, Clone)]
pub struct SshServerConfig {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u32,
    pub username: String,
    pub password: Option<String>,
    pub ssh_key_path: Option<String>,
}

/// 统一的 SSH 认证入口（connect / test_connection / 一次性会话共用）
///
/// ## 为什么必须有这个函数
/// 数据库里 `sshKeyPath` 对「未配置密钥」的服务器存的是**空字符串 `''`**，不是 NULL。
/// 若照 `if let Some(path) = &config.ssh_key_path` 直接判断，`Some("")` 会被当成有效路径，
/// ssh2 于是去打开空路径，报 `Unable to open private key file`；
/// 因为密钥分支在密码分支之前，配了密码的服务器**永远轮不到密码认证**。
/// 这与 GUI 侧（`tauri/src/commands/logs.rs` 密码优先）行为不一致，
/// 表现为「GUI 能连、CLI 报私钥打不开」。
///
/// ## 策略
/// 1. 仅当密钥路径非空（去空白后）才尝试密钥认证
/// 2. 密钥未配置或认证失败时，回退到密码认证
/// 3. 两者都不可用才报错，并带上密钥失败的原因
/// 返回应当用于认证的密钥路径，未配置则返回 `None`。
///
/// 关键：数据库里「未配置密钥」存的是**空字符串 `''`** 而不是 NULL，
/// 所以空串（或纯空白）必须视为未配置 —— 否则 ssh2 会去打开空路径并报
/// `Unable to open private key file`。
fn usable_key_path(config: &SshServerConfig) -> Option<&str> {
    config
        .ssh_key_path
        .as_deref()
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
}

fn authenticate_session(session: &Session, config: &SshServerConfig) -> Result<(), String> {
    let key_path = usable_key_path(config);

    let mut key_error: Option<String> = None;
    if let Some(path) = key_path {
        match session.userauth_pubkey_file(&config.username, None, Path::new(path), None) {
            Ok(()) => return Ok(()),
            Err(e) => {
                let msg = format!("密钥认证失败({}): {}", path, e);
                log::warn!("[SSH] {} 将尝试回退密码认证", msg);
                key_error = Some(msg);
            }
        }
    }

    if let Some(ref password) = config.password {
        return session
            .userauth_password(&config.username, password)
            .map_err(|e| {
                match key_error {
                    // 密钥也试过了，把两条信息都带上，便于排查
                    Some(k) => format!("{}；密码认证也失败: {}", k, e),
                    None => format!("密码认证失败: {}", e),
                }
            });
    }

    Err(key_error.unwrap_or_else(|| "没有可用的认证方式".to_string()))
}

/// PTY 终端封装
pub struct PtyTerminal {
    pub channel: Channel,
    pub rows: u32,
    pub cols: u32,
    pub server_id: String,
}

/// SSH 服务 — 管理所有 SSH 连接和终端
///
/// ## 锁设计（关键）
/// `connections: Mutex<HashMap<String, Arc<Session>>>` 只做**索引查询**：
/// 1. 拿 `Arc<Session>` 的 clone（原子操作，O(1)）
/// 2. 释放 Mutex 锁
/// 3. 在锁外用 Session 做所有阻塞 I/O
///
/// 一个连接卡死在 `channel_session()` 中，**不会**锁住其他服务器的操作。
pub struct SshService {
    pub connections: Mutex<HashMap<String, Arc<Session>>>,
    terminals: Mutex<HashMap<String, Arc<Mutex<PtyTerminal>>>>,
    sftp_sessions: Mutex<HashMap<String, Arc<Mutex<Sftp>>>>,
}

impl SshService {
    pub fn new() -> Self {
        Self {
            connections: Mutex::new(HashMap::new()),
            terminals: Mutex::new(HashMap::new()),
            sftp_sessions: Mutex::new(HashMap::new()),
        }
    }

    /// 获取某个服务器的 Session（克隆 Arc，立刻释放全局锁）
    fn get_session(&self, server_id: &str) -> Result<Arc<Session>, String> {
        let conns = self.connections.lock().map_err(|e| e.to_string())?;
        conns
            .get(server_id)
            .ok_or_else(|| "服务器未连接".to_string())
            .map(|s| s.clone())
    }

    /// 创建 PTY 终端（交互式 shell）
    ///
    /// 与 Electron `conn.shell({ term: 'xterm-256color', rows, cols })` 对应。
    /// Rust ssh2: session.channel_session().request_pty("xterm", ...) + channel.shell()
    pub fn create_terminal(
        &self,
        server_id: &str,
        terminal_id: &str,
        rows: u32,
        cols: u32,
    ) -> Result<bool, String> {
        let session = self.get_session(server_id)?;

        let mut channel = session
            .channel_session()
            .map_err(|e| format!("创建通道失败: {}", e))?;

        // 请求 PTY（与 Electron 的 xterm-256color 对应）
        channel
            .request_pty("xterm", None, Some((cols, rows, 0, 0)))
            .map_err(|e| format!("请求 PTY 失败: {}", e))?;

        // 启动交互式 shell
        channel
            .shell()
            .map_err(|e| format!("启动 shell 失败: {}", e))?;

        let terminal = PtyTerminal {
            channel,
            rows,
            cols,
            server_id: server_id.to_string(),
        };
        let mut terminals = self.terminals.lock().map_err(|e| e.to_string())?;
        terminals.insert(terminal_id.to_string(), Arc::new(Mutex::new(terminal)));

        log::info!(
            "[SSH] PTY terminal created: terminal_id={} server_id={} pty={}x{}",
            terminal_id,
            server_id,
            cols,
            rows
        );
        Ok(true)
    }

    /// 读取终端输出
    pub fn read_terminal(&self, terminal_id: &str) -> Result<String, String> {
        let mut output = String::new();

        // 先获取 server_id（短暂持有 terminal 锁）
        let server_id = {
            let terminals = self.terminals.lock().map_err(|e| e.to_string())?;
            let terminal = terminals
                .get(terminal_id)
                .ok_or_else(|| "终端不存在".to_string())?;
            terminal
                .lock()
                .map_err(|e| e.to_string())?
                .server_id
                .clone()
        };

        // 切换 session 为非阻塞模式（短暂持有 connections 锁）
        if let Ok(session) = self.get_session(&server_id) {
            session.set_blocking(false);
        }

        // 读取输出（此时没有锁）
        {
            let terminals = self.terminals.lock().map_err(|e| e.to_string())?;
            let terminal = terminals
                .get(terminal_id)
                .ok_or_else(|| "终端不存在".to_string())?;
            let mut terminal = terminal.lock().map_err(|e| e.to_string())?;
            let mut buf = [0u8; 4096];
            loop {
                match std::io::Read::read(&mut terminal.channel, &mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        output.push_str(&String::from_utf8_lossy(&buf[..n]));
                        if n < buf.len() {
                            break;
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(_) => break,
                }
            }
        }

        // 恢复阻塞模式
        if let Ok(session) = self.get_session(&server_id) {
            session.set_blocking(true);
        }

        Ok(output)
    }

    /// 调整终端 PTY 尺寸（发送 SIGWINCH 信号）
    ///
    /// 与 Electron `stream.setWindow(rows, cols)` 对应。
    pub fn resize_terminal(&self, terminal_id: &str, rows: u32, cols: u32) -> Result<bool, String> {
        let terminals = self.terminals.lock().map_err(|e| e.to_string())?;
        let terminal = terminals
            .get(terminal_id)
            .ok_or_else(|| "终端不存在".to_string())?;

        let mut terminal = terminal.lock().map_err(|e| e.to_string())?;
        terminal
            .channel
            .request_pty_size(cols, rows, None, None)
            .map_err(|e| format!("调整终端尺寸失败: {}", e))?;

        terminal.rows = rows;
        terminal.cols = cols;

        log::info!(
            "[SSH] Terminal resized: terminal_id={} pty={}x{}",
            terminal_id,
            cols,
            rows
        );
        Ok(true)
    }

    /// 向终端写入数据
    ///
    /// 与 Electron `stream.write(data)` 对应。
    pub fn write_to_terminal(&self, terminal_id: &str, data: &str) -> Result<bool, String> {
        let terminals = self.terminals.lock().map_err(|e| e.to_string())?;
        let terminal = terminals
            .get(terminal_id)
            .ok_or_else(|| "终端不存在".to_string())?;

        let mut terminal = terminal.lock().map_err(|e| e.to_string())?;

        // 检查通道是否还活跃
        if terminal.channel.eof() {
            return Err("终端连接已断开".to_string());
        }

        terminal
            .channel
            .write_all(data.as_bytes())
            .map_err(|e| format!("写入终端失败: {}", e))?;

        Ok(true)
    }

    /// 关闭终端
    ///
    /// 与 Electron `stream.end()` 对应。
    pub fn close_terminal(&self, terminal_id: &str) -> Result<bool, String> {
        let mut terminals = self.terminals.lock().map_err(|e| e.to_string())?;
        if let Some(terminal) = terminals.remove(terminal_id) {
            let mut terminal = terminal.lock().map_err(|e| e.to_string())?;
            let _ = terminal.channel.close();
            let _ = terminal.channel.wait_close();
            log::info!("[SSH] Terminal closed: terminal_id={}", terminal_id);
        }
        Ok(true)
    }

    /// 检查终端是否还存活
    pub fn is_terminal_active(&self, terminal_id: &str) -> bool {
        let terminals = self.terminals.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(terminal) = terminals.get(terminal_id) {
            if let Ok(t) = terminal.lock() {
                return !t.channel.eof();
            }
        }
        false
    }

    /// 连接服务器
    pub fn connect(&self, config: &SshServerConfig) -> Result<bool, String> {
        // 先检查是否已连（短锁）
        {
            let conns = self.connections.lock().map_err(|e| e.to_string())?;
            if conns.contains_key(&config.id) {
                return Ok(true);
            }
        }

        log::info!(
            "[SSH] Connecting to {} {}:{} as {}",
            config.id,
            config.host,
            config.port,
            config.username
        );

        let tcp = TcpStream::connect(format!("{}:{}", config.host, config.port))
            .map_err(|e| format!("TCP 连接失败: {}", e))?;

        // 设置 TCP 读写超时，防止阻塞读卡死
        let _ = tcp.set_read_timeout(Some(std::time::Duration::from_secs(30)));
        let _ = tcp.set_write_timeout(Some(std::time::Duration::from_secs(15)));

        let mut session = Session::new().map_err(|e| format!("创建 SSH 会话失败: {}", e))?;
        session.set_tcp_stream(tcp.try_clone().map_err(|e| e.to_string())?);
        session.set_timeout(30_000); // SSH 层超时：握手+认证+操作
        session
            .handshake()
            .map_err(|e| format!("SSH 握手失败: {}", e))?;

        // 认证（空密钥路径会回退到密码，详见 authenticate_session 注释）
        authenticate_session(&session, config)?;

        if !session.authenticated() {
            return Err("认证失败".to_string());
        }

        // 启用 SSH keepalive（每30秒发一次心跳，防止防火墙断开连接）
        let _ = session.set_keepalive(true, 30);

        // 存入连接池（短锁）
        {
            let mut conns = self.connections.lock().map_err(|e| e.to_string())?;
            // 二次检查：认证期间另一个线程可能已经连上了
            if conns.contains_key(&config.id) {
                log::info!(
                    "[SSH] Already connected to {} (concurrent connect)",
                    config.id
                );
                return Ok(true);
            }
            conns.insert(config.id.clone(), Arc::new(session));
        }

        log::info!("[SSH] Connected to {} ({})", config.id, config.name);
        Ok(true)
    }

    /// 测试连接（不保存连接）
    pub fn test_connection(&self, config: &SshServerConfig) -> Result<bool, String> {
        let tcp = TcpStream::connect(format!("{}:{}", config.host, config.port))
            .map_err(|e| format!("TCP 连接失败: {}", e))?;

        let _ = tcp.set_read_timeout(Some(std::time::Duration::from_secs(15)));
        let _ = tcp.set_write_timeout(Some(std::time::Duration::from_secs(10)));

        let mut session = Session::new().map_err(|e| format!("创建 SSH 会话失败: {}", e))?;
        session.set_tcp_stream(tcp.try_clone().map_err(|e| e.to_string())?);
        session.set_timeout(15_000);
        session
            .handshake()
            .map_err(|e| format!("SSH 握手失败: {}", e))?;

        // 认证（空密钥路径会回退到密码，详见 authenticate_session 注释）
        authenticate_session(&session, config)?;

        if !session.authenticated() {
            return Err("认证失败".to_string());
        }

        Ok(true)
    }

    /// 断开连接
    pub fn disconnect(&self, server_id: &str) {
        if let Ok(mut conns) = self.connections.lock() {
            if conns.remove(server_id).is_some() {
                log::info!("[SSH] Disconnected from {}", server_id);
            }
        }
        // 清理缓存的 SFTP 会话
        if let Ok(mut cache) = self.sftp_sessions.lock() {
            cache.remove(server_id);
        }
    }

    /// 检查是否已连接
    /// 注：仅检查连接池中是否存在，不做实际探活（探活会阻塞锁）
    pub fn is_connected(&self, server_id: &str) -> bool {
        let conns = match self.connections.lock() {
            Ok(c) => c,
            Err(_) => return false,
        };
        conns.contains_key(server_id)
    }

    /// 执行命令（锁外执行所有阻塞 I/O）
    ///
    /// 流程：
    /// 1. 短锁取 Arc<Session> clone → 释放锁
    /// 2. 锁外调用 channel_session()（可能阻塞，但不影响其他连接）
    /// 3. 分块读取输出，TCP 超时 + 1MB 上限防卡死
    pub fn exec_command(&self, server_id: &str, command: &str) -> Result<ExecResult, String> {
        let session = self.get_session(server_id)?;

        // === 以下所有阻塞 I/O 均在 Mutex 之外执行 ===

        let mut channel = match session.channel_session() {
            Ok(ch) => {
                let mut ch = ch;
                if let Err(e) = ch.exec(command) {
                    let _ = self.disconnect(server_id);
                    return Err(format!("执行命令失败（连接可能已断开）: {}", e));
                }
                ch
            }
            Err(e) => {
                let _ = self.disconnect(server_id);
                return Err(format!("打开通道失败（连接可能已断开）: {}", e));
            }
        };

        // 分块读取 + 大小限制 + TCP 超时保护（30s read_timeout 已设）
        let mut output = String::new();
        let mut buf = [0u8; 8192];
        const MAX_OUTPUT: usize = 1_000_000; // 1MB 上限

        loop {
            if output.len() >= MAX_OUTPUT {
                output.push_str("\n--- [输出超过 1MB, 截断] ---");
                break;
            }
            match channel.read(&mut buf) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    output.push_str(&String::from_utf8_lossy(&buf[..n]));
                }
                Err(e) => {
                    // TCP 超时或断开 — 有部分输出也比卡死好
                    if output.is_empty() {
                        return Err(format!("读取命令输出失败: {}", e));
                    }
                    log::warn!(
                        "[SSH] exec_command read error (returning partial output): {}",
                        e
                    );
                    break;
                }
            }
        }

        let exit_code = channel.exit_status().ok();
        let success = exit_code.map(|c| c == 0).unwrap_or(false);

        let _ = channel.close();
        let _ = channel.wait_close();

        Ok(ExecResult {
            success,
            output,
            error_output: String::new(),
            exit_code: exit_code.map(|c| c as u32),
        })
    }

    /// 在独立连接上批量执行命令（不共享连接池，不影响终端 shell 会话）
    ///
    /// 创建临时 SSH 连接 → 逐条执行命令 → 断开连接。
    /// 用于监控面板等场景，避免与终端的 shell 通道争用同一 session。
    pub fn exec_commands_independent(
        &self,
        config: &SshServerConfig,
        commands: &[String],
    ) -> Result<HashMap<String, ExecResult>, String> {
        // 创建临时 TCP 连接
        let tcp = TcpStream::connect(format!("{}:{}", config.host, config.port))
            .map_err(|e| format!("TCP 连接失败: {}", e))?;
        let _ = tcp.set_read_timeout(Some(std::time::Duration::from_secs(15)));
        let _ = tcp.set_write_timeout(Some(std::time::Duration::from_secs(10)));

        let mut session = Session::new().map_err(|e| format!("创建 SSH 会话失败: {}", e))?;
        session.set_tcp_stream(tcp.try_clone().map_err(|e| e.to_string())?);
        session.set_timeout(20_000);
        session
            .handshake()
            .map_err(|e| format!("SSH 握手失败: {}", e))?;

        // 认证（空密钥路径会回退到密码，详见 authenticate_session 注释）
        authenticate_session(&session, config)?;

        if !session.authenticated() {
            return Err("认证失败".to_string());
        }

        // 逐条执行命令
        let mut results = HashMap::new();
        for cmd in commands {
            let mut channel = session
                .channel_session()
                .map_err(|e| format!("打开通道失败: {}", e))?;
            if let Err(e) = channel.exec(cmd) {
                results.insert(
                    cmd.clone(),
                    ExecResult {
                        success: false,
                        output: String::new(),
                        error_output: format!("执行失败: {}", e),
                        exit_code: None,
                    },
                );
                continue;
            }

            let mut output = String::new();
            let mut buf = [0u8; 8192];
            const MAX_OUTPUT: usize = 1_000_000;
            loop {
                if output.len() >= MAX_OUTPUT {
                    output.push_str("\n--- [输出超过 1MB, 截断] ---");
                    break;
                }
                match channel.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => output.push_str(&String::from_utf8_lossy(&buf[..n])),
                    Err(e) => {
                        if output.is_empty() {
                            output = format!("读取输出失败: {}", e);
                        }
                        break;
                    }
                }
            }

            let exit_code = channel.exit_status().ok();
            let success = exit_code.map(|c| c == 0).unwrap_or(false);
            let _ = channel.close();
            let _ = channel.wait_close();

            results.insert(
                cmd.clone(),
                ExecResult {
                    success,
                    output,
                    error_output: String::new(),
                    exit_code: exit_code.map(|c| c as u32),
                },
            );
        }

        // 断开临时连接（drop session + tcp）
        drop(session);
        Ok(results)
    }

    /// 创建 SFTP 会话并列出目录
    pub fn list_remote_dir(
        &self,
        server_id: &str,
        remote_path: &str,
    ) -> Result<Vec<SftpFile>, String> {
        // SFTP readdir 偶发 WouldBlock（libssh2 通道状态残留），
        // 检测后丢弃缓存重建 SFTP 通道重试一次
        let result = self.list_remote_dir_inner(server_id, remote_path);
        if let Err(ref e) = result {
            if e.contains("Would block") {
                log::warn!(
                    "[SFTP] WouldBlock on {} dir {}, clearing cache and retrying",
                    server_id,
                    remote_path
                );
                if let Ok(mut cache) = self.sftp_sessions.lock() {
                    cache.remove(server_id);
                }
                return self.list_remote_dir_inner(server_id, remote_path);
            }
        }
        result
    }

    fn list_remote_dir_inner(
        &self,
        server_id: &str,
        remote_path: &str,
    ) -> Result<Vec<SftpFile>, String> {
        let sftp_lock = self.get_sftp(server_id)?;
        let sftp = sftp_lock.lock().map_err(|e| e.to_string())?;
        let expanded_path = Self::expand_remote_path(&sftp, remote_path)?;

        let readdir = sftp
            .readdir(Path::new(&expanded_path))
            .map_err(|e| format!("读取目录失败: {}", e))?;

        let files: Vec<SftpFile> = readdir
            .iter()
            .map(|(path, stat)| {
                let file_type = if stat.is_dir() { "directory" } else { "file" };
                let size = stat.size.unwrap_or(0);
                let mtime = stat
                    .mtime
                    .map(|t| {
                        chrono::DateTime::<chrono::Utc>::from_timestamp(t as i64, 0)
                            .map(|dt| dt.to_rfc3339())
                            .unwrap_or_default()
                    })
                    .unwrap_or_default();
                let perms = format!("{:o}", stat.perm.unwrap_or(0));
                SftpFile {
                    name: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    file_type: file_type.to_string(),
                    size,
                    modify_time: mtime,
                    permissions: perms,
                }
            })
            .collect();

        Ok(files)
    }

    /// 下载文件内容为 Base64
    pub fn download_file_base64(
        &self,
        server_id: &str,
        remote_path: &str,
    ) -> Result<String, String> {
        let sftp_lock = self.get_sftp(server_id)?;
        let sftp = sftp_lock.lock().map_err(|e| e.to_string())?;
        let expanded_path = Self::expand_remote_path(&sftp, remote_path)?;

        let mut file = sftp
            .open(Path::new(&expanded_path))
            .map_err(|e| format!("打开文件失败: {}", e))?;

        let mut contents = Vec::new();
        file.read_to_end(&mut contents)
            .map_err(|e| format!("读取文件失败: {}", e))?;

        Ok(base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            &contents,
        ))
    }

    /// 创建远程目录
    pub fn create_remote_dir(&self, server_id: &str, remote_path: &str) -> Result<bool, String> {
        let sftp_lock = self.get_sftp(server_id)?;
        let sftp = sftp_lock.lock().map_err(|e| e.to_string())?;
        let expanded_path = Self::expand_remote_path(&sftp, remote_path)?;

        sftp.mkdir(Path::new(&expanded_path), 0o755)
            .map(|_| true)
            .map_err(|e| format!("创建目录失败: {}", e))
    }

    /// 删除远程文件
    pub fn delete_remote_file(&self, server_id: &str, remote_path: &str) -> Result<bool, String> {
        let sftp_lock = self.get_sftp(server_id)?;
        let sftp = sftp_lock.lock().map_err(|e| e.to_string())?;
        let expanded_path = Self::expand_remote_path(&sftp, remote_path)?;

        sftp.unlink(Path::new(&expanded_path))
            .map(|_| true)
            .map_err(|e| format!("删除文件失败: {}", e))
    }

    /// 上传文件到远程服务器（无进度回调）
    pub fn upload_file(
        &self,
        server_id: &str,
        local_path: &str,
        remote_path: &str,
    ) -> Result<u64, String> {
        self.upload_file_with_progress(server_id, local_path, remote_path, None)
    }

    /// 上传文件到远程服务器，支持进度回调
    /// progress_callback: 参数为 (已上传字节, 文件总字节)，每隔 ~100ms 调用一次
    pub fn upload_file_with_progress(
        &self,
        server_id: &str,
        local_path: &str,
        remote_path: &str,
        progress_callback: Option<&dyn Fn(u64, u64)>,
    ) -> Result<u64, String> {
        let sftp_lock = self.get_sftp(server_id)?;
        let sftp = sftp_lock.lock().map_err(|e| e.to_string())?;
        let local = Path::new(local_path);
        let expanded_path = Self::expand_remote_path(&sftp, remote_path)?;

        // 获取本地文件大小（用于进度计算）
        let file_size = std::fs::metadata(local)
            .map(|m| m.len())
            .map_err(|e| format!("读取文件信息失败: {}", e))?;

        // 打开本地文件（分块读取，避免一次性加载到内存）
        let mut local_file = std::fs::File::open(local)
            .map_err(|e| format!("打开本地文件失败: {}", e))?;

        // 创建远程文件
        let mut remote_file = sftp
            .create(Path::new(&expanded_path))
            .map_err(|e| format!("创建远程文件失败: {}", e))?;

        // 分块读取写入，支持进度回调
        let chunk_size: usize = 64 * 1024; // 64KB chunks
        let mut buffer = vec![0u8; chunk_size];
        let mut uploaded: u64 = 0;
        let mut last_report = std::time::Instant::now();

        loop {
            let n = local_file
                .read(&mut buffer)
                .map_err(|e| format!("读取本地文件失败: {}", e))?;
            if n == 0 {
                break;
            }
            remote_file
                .write_all(&buffer[..n])
                .map_err(|e| format!("写入远程文件失败: {}", e))?;
            uploaded += n as u64;

            // 进度回调：每 100ms 上报一次，避免过于频繁
            if let Some(cb) = progress_callback {
                let now = std::time::Instant::now();
                if now.duration_since(last_report).as_millis() >= 100 || uploaded == file_size {
                    cb(uploaded, file_size);
                    last_report = now;
                }
            }
        }

        remote_file.flush().ok();

        log::info!(
            "[SFTP] Uploaded {} ({}) to {}:{}",
            local_path,
            uploaded,
            server_id,
            expanded_path
        );
        Ok(uploaded)
    }

    /// 下载文件到本地
    pub fn download_file(
        &self,
        server_id: &str,
        remote_path: &str,
        local_path: &str,
    ) -> Result<u64, String> {
        // 无进度回调版本：直接调用带回调版本，传入 noop
        self.download_file_with_progress(server_id, remote_path, local_path, None)
    }

    /// SFTP 下载文件到本地，支持进度回调
    /// progress_callback: 参数为 (已下载字节, 文件总字节)，每隔 ~100ms 调用一次
    pub fn download_file_with_progress(
        &self,
        server_id: &str,
        remote_path: &str,
        local_path: &str,
        progress_callback: Option<&dyn Fn(u64, u64)>,
    ) -> Result<u64, String> {
        let sftp_lock = self.get_sftp(server_id)?;
        let sftp = sftp_lock.lock().map_err(|e| e.to_string())?;
        let expanded_path = Self::expand_remote_path(&sftp, remote_path)?;

        // 打开远程文件
        let mut remote_file = sftp
            .open(Path::new(&expanded_path))
            .map_err(|e| format!("打开远程文件失败: {}", e))?;

        // 获取文件大小（用于进度计算）
        let file_size = sftp
            .stat(Path::new(&expanded_path))
            .map(|attr| attr.size.unwrap_or(0))
            .unwrap_or(0);

        // 确保本地目录存在
        if let Some(parent) = Path::new(local_path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建本地目录失败: {}", e))?;
        }

        // 创建本地文件
        let mut local_file = std::fs::File::create(local_path)
            .map_err(|e| format!("创建本地文件失败: {}", e))?;

        // 分块读取写入，支持进度回调
        let chunk_size: usize = 64 * 1024;  // 64KB chunks
        let mut buffer = vec![0u8; chunk_size];
        let mut downloaded: u64 = 0;
        let mut last_report = std::time::Instant::now();

        loop {
            let n = remote_file
                .read(&mut buffer)
                .map_err(|e| format!("读取远程文件失败: {}", e))?;
            if n == 0 {
                break;
            }
            local_file
                .write_all(&buffer[..n])
                .map_err(|e| format!("写入本地文件失败: {}", e))?;
            downloaded += n as u64;

            // 进度回调：每 100ms 上报一次，避免过于频繁
            if let Some(cb) = progress_callback {
                let now = std::time::Instant::now();
                if now.duration_since(last_report).as_millis() >= 100 || downloaded == file_size {
                    cb(downloaded, file_size);
                    last_report = now;
                }
            }
        }

        local_file.flush().ok();

        log::info!(
            "[SFTP] Downloaded {}:{} to {} ({})",
            server_id,
            remote_path,
            local_path,
            downloaded
        );
        Ok(downloaded)
    }

    /// 递归上传目录
    pub fn upload_dir_recursive(
        &self,
        server_id: &str,
        local_path: &str,
        remote_path: &str,
    ) -> Result<u64, String> {
        let local = Path::new(local_path);
        if !local.is_dir() {
            return Err("本地路径不是目录".to_string());
        }

        // 创建远程目录（短暂持锁）
        let expanded_path = {
            let sftp_lock = self.get_sftp(server_id)?;
            let sftp = sftp_lock.lock().map_err(|e| e.to_string())?;
            let path = Self::expand_remote_path(&sftp, remote_path)?;
            let _ = sftp.mkdir(Path::new(&path), 0o755);
            path
        };

        let mut total_size: u64 = 0;
        for entry in std::fs::read_dir(local).map_err(|e| format!("读取本地目录失败: {}", e))?
        {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let file_name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let new_remote = format!("{}/{}", expanded_path, file_name);

            if path.is_dir() {
                let size =
                    self.upload_dir_recursive(server_id, &path.to_string_lossy(), &new_remote)?;
                total_size += size;
            } else {
                let size = self.upload_file(server_id, &path.to_string_lossy(), &new_remote)?;
                total_size += size;
            }
        }

        log::info!(
            "[SFTP] Uploaded directory {} to {}:{} ({} bytes total)",
            local_path,
            server_id,
            expanded_path,
            total_size
        );
        Ok(total_size)
    }

    // ============ 内部辅助方法 ============

    /// 获取 SFTP 会话（锁外执行所有阻塞 I/O），复用缓存避免频繁创建通道
    fn get_sftp(&self, server_id: &str) -> Result<Arc<Mutex<Sftp>>, String> {
        // 先查缓存
        {
            let cache = self.sftp_sessions.lock().map_err(|e| e.to_string())?;
            if let Some(sftp) = cache.get(server_id) {
                return Ok(sftp.clone());
            }
        }

        let session = self.get_session(server_id)?;

        // 检查连接是否还活着
        if !session.authenticated() {
            self.disconnect(server_id);
            return Err("SSH 连接已断开".to_string());
        }

        // 确保 session 处于阻塞模式
        session.set_blocking(true);

        let sftp = session.sftp().map_err(|e| {
            log::warn!("[SSH] SFTP creation failed for {}: {}", server_id, e);
            let _ = self.disconnect(server_id);
            format!("创建 SFTP 会话失败（连接可能已断开）: {}", e)
        })?;

        let sftp = Arc::new(Mutex::new(sftp));
        let mut cache = self.sftp_sessions.lock().map_err(|e| e.to_string())?;
        cache.insert(server_id.to_string(), sftp.clone());
        Ok(sftp)
    }

    /// 展开远程路径中的 ~ 为实际路径（使用 SFTP realpath）
    /// openssh sftp-server 支持 ~ 语法；某些嵌入式/受限 sftp-server 不支持，
    /// 此时分两步：realpath(".") 拿到 home 目录，再手工替换 ~ 前缀。
    fn expand_remote_path(sftp: &Sftp, remote_path: &str) -> Result<String, String> {
        if !remote_path.starts_with('~') {
            return Ok(remote_path.to_string());
        }

        // 先尝试直接 realpath（openssh 支持 ~ 语法）
        match sftp.realpath(Path::new(remote_path)) {
            Ok(p) => Ok(p.to_string_lossy().to_string()),
            Err(e) => {
                // 降级：realpath(".") 通常返回用户的 home 目录
                let home = sftp
                    .realpath(Path::new("."))
                    .map_err(|e2| {
                        format!(
                            "展开路径 '~' 失败: {}（且无法获取 home 目录: {}）",
                            e, e2
                        )
                    })?;
                let home_str = home.to_string_lossy();

                // 仅处理 ~ 和 ~/xxx 两种形式；~user/xxx 不展开，让后续操作抛原始错误
                let expanded = if remote_path == "~" {
                    home_str.to_string()
                } else if let Some(rest) = remote_path.strip_prefix("~/") {
                    format!("{}/{}", home_str.trim_end_matches('/'), rest)
                } else {
                    // ~user/xxx 不支持，保留原路径让 sftp.open 自己抛错
                    return Ok(remote_path.to_string());
                };
                Ok(expanded)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_with_key(key: Option<&str>) -> SshServerConfig {
        SshServerConfig {
            id: "s1".to_string(),
            name: "test".to_string(),
            host: "127.0.0.1".to_string(),
            port: 22,
            username: "root".to_string(),
            password: Some("pw".to_string()),
            ssh_key_path: key.map(|s| s.to_string()),
        }
    }

    /// 回归用例：数据库里「未配置密钥」存的是空字符串 `''` 而非 NULL。
    /// 空串绝不能被当成有效密钥路径，否则 ssh2 会去打开空路径而报
    /// `Unable to open private key file`，且因为密钥认证先于密码认证，
    /// 配了密码的服务器会永远走不到密码分支。
    #[test]
    fn empty_ssh_key_path_is_treated_as_unset() {
        assert_eq!(usable_key_path(&config_with_key(None)), None);
        assert_eq!(usable_key_path(&config_with_key(Some(""))), None);
        assert_eq!(usable_key_path(&config_with_key(Some("   "))), None);
        assert_eq!(usable_key_path(&config_with_key(Some("\t\n"))), None);
    }

    #[test]
    fn real_ssh_key_path_is_preserved() {
        assert_eq!(
            usable_key_path(&config_with_key(Some("/Users/me/.ssh/id_ed25519_github"))),
            Some("/Users/me/.ssh/id_ed25519_github")
        );
        // 首尾空白应被去掉，避免把 " /path " 当成一个不存在的相对路径
        assert_eq!(
            usable_key_path(&config_with_key(Some("  /Users/me/.ssh/id_rsa  "))),
            Some("/Users/me/.ssh/id_rsa")
        );
    }
}
