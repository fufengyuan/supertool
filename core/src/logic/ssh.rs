/// SSH Service — 连接池 + 命令执行 + SFTP + PTY 终端
///
/// 替代 Electron 的 ssh2 npm 包，使用 Rust 的 ssh2 crate（libssh2 绑定）。
/// 支持密码认证和密钥文件认证。
///
/// 与 Electron 的区别：
/// - Electron ssh2 (npm): 回调式 API
/// - Rust ssh2: 同步 API，需要在 tokio::task::spawn_blocking 中调用
/// - PTY 终端：conn.shell() → Rust: session.channel_session().shell()
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

/// SSH 连接封装
pub struct SshConnection {
    pub session: Session,
    pub stream: TcpStream,
}

/// PTY 终端封装
pub struct PtyTerminal {
    pub channel: Channel,
    pub rows: u32,
    pub cols: u32,
    pub server_id: String,
}

/// SSH 服务 — 管理所有 SSH 连接和终端
pub struct SshService {
    pub connections: Mutex<HashMap<String, SshConnection>>,
    terminals: Mutex<HashMap<String, Arc<Mutex<PtyTerminal>>>>,
}

impl SshService {
    pub fn new() -> Self {
        Self {
            connections: Mutex::new(HashMap::new()),
            terminals: Mutex::new(HashMap::new()),
        }
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
        let channel = {
            let conns = self.connections.lock().map_err(|e| e.to_string())?;
            let conn = conns
                .get(server_id)
                .ok_or_else(|| "服务器未连接".to_string())?;

            let mut channel = conn
                .session
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

            channel
        };

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
            terminal.lock().map_err(|e| e.to_string())?.server_id.clone()
        };

        // 切换为非阻塞模式（短暂持有 connections 锁）
        {
            let conns = self.connections.lock().map_err(|e| e.to_string())?;
            if let Some(conn) = conns.get(&server_id) {
                conn.session.set_blocking(false);
            }
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

        // 恢复阻塞模式（短暂持有 connections 锁）
        {
            let conns = self.connections.lock().map_err(|e| e.to_string())?;
            if let Some(conn) = conns.get(&server_id) {
                conn.session.set_blocking(true);
            }
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

        let mut session = Session::new().map_err(|e| format!("创建 SSH 会话失败: {}", e))?;
        session.set_tcp_stream(tcp.try_clone().map_err(|e| e.to_string())?);
        session.set_timeout(30_000); // 30秒超时，防止阻塞读卡死
        session
            .handshake()
            .map_err(|e| format!("SSH 握手失败: {}", e))?;

        // 认证
        if let Some(ref key_path) = config.ssh_key_path {
            session
                .userauth_pubkey_file(&config.username, None, Path::new(key_path), None)
                .map_err(|e| format!("密钥认证失败: {}", e))?;
        } else if let Some(ref password) = config.password {
            session
                .userauth_password(&config.username, password)
                .map_err(|e| format!("密码认证失败: {}", e))?;
        } else {
            return Err("没有可用的认证方式".to_string());
        }

        if !session.authenticated() {
            return Err("认证失败".to_string());
        }

        {
            let mut conns = self.connections.lock().map_err(|e| e.to_string())?;
            // 二次检查：认证期间另一个线程可能已经连上了
            if conns.contains_key(&config.id) {
                log::info!("[SSH] Already connected to {} (concurrent connect)", config.id);
                return Ok(true);
            }
            conns.insert(
                config.id.clone(),
                SshConnection {
                    session,
                    stream: tcp,
                },
            );
        }

        log::info!("[SSH] Connected to {} ({})", config.id, config.name);
        Ok(true)
    }

    /// 测试连接（不保存连接）
    pub fn test_connection(&self, config: &SshServerConfig) -> Result<bool, String> {
        let tcp = TcpStream::connect(format!("{}:{}", config.host, config.port))
            .map_err(|e| format!("TCP 连接失败: {}", e))?;

        let mut session = Session::new().map_err(|e| format!("创建 SSH 会话失败: {}", e))?;
        session.set_tcp_stream(tcp.try_clone().map_err(|e| e.to_string())?);
        session
            .handshake()
            .map_err(|e| format!("SSH 握手失败: {}", e))?;

        if let Some(ref key_path) = config.ssh_key_path {
            session
                .userauth_pubkey_file(&config.username, None, Path::new(key_path), None)
                .map_err(|e| format!("密钥认证失败: {}", e))?;
        } else if let Some(ref password) = config.password {
            session
                .userauth_password(&config.username, password)
                .map_err(|e| format!("密码认证失败: {}", e))?;
        } else {
            return Err("没有可用的认证方式".to_string());
        }

        if !session.authenticated() {
            return Err("认证失败".to_string());
        }

        Ok(true)
    }

    /// 断开连接
    pub fn disconnect(&self, server_id: &str) {
        if let Ok(mut conns) = self.connections.lock() {
            if let Some(conn) = conns.remove(server_id) {
                drop(conn.session);
                drop(conn.stream);
                log::info!("[SSH] Disconnected from {}", server_id);
            }
        }
    }

    /// 检查是否已连接（且连接真的活着）
    pub fn is_connected(&self, server_id: &str) -> bool {
        let conns = match self.connections.lock() {
            Ok(c) => c,
            Err(_) => return false,
        };
        if let Some(conn) = conns.get(server_id) {
            // 检查 session 是否还认证着
            conn.session.authenticated()
        } else {
            false
        }
    }

    /// 执行命令
    pub fn exec_command(&self, server_id: &str, command: &str) -> Result<ExecResult, String> {
        let mut channel = {
            let conns = self.connections.lock().map_err(|e| e.to_string())?;
            let conn = conns
                .get(server_id)
                .ok_or_else(|| "服务器未连接".to_string())?;
            match conn.session.channel_session() {
                Ok(ch) => {
                    let mut ch = ch;
                    if let Err(e) = ch.exec(command) {
                        // exec 失败可能是连接已断开，清理掉
                        let _ = self.disconnect(server_id);
                        return Err(format!("执行命令失败（连接可能已断开）: {}", e));
                    }
                    ch
                }
                Err(e) => {
                    // channel_session 失败说明连接已死，清理
                    let _ = self.disconnect(server_id);
                    return Err(format!("打开通道失败（连接可能已断开）: {}", e));
                }
            }
        };

        let mut output = String::new();
        let _ = channel.read_to_string(&mut output);

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

    /// 创建 SFTP 会话并列出目录
    pub fn list_remote_dir(
        &self,
        server_id: &str,
        remote_path: &str,
    ) -> Result<Vec<SftpFile>, String> {
        let sftp = self.get_sftp(server_id)?;

        let readdir = sftp
            .readdir(Path::new(remote_path))
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
        let sftp = self.get_sftp(server_id)?;

        let mut file = sftp
            .open(Path::new(remote_path))
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
        let sftp = self.get_sftp(server_id)?;

        sftp.mkdir(Path::new(remote_path), 0o755)
            .map(|_| true)
            .map_err(|e| format!("创建目录失败: {}", e))
    }

    /// 删除远程文件
    pub fn delete_remote_file(&self, server_id: &str, remote_path: &str) -> Result<bool, String> {
        let sftp = self.get_sftp(server_id)?;

        sftp.unlink(Path::new(remote_path))
            .map(|_| true)
            .map_err(|e| format!("删除文件失败: {}", e))
    }

    /// 上传文件到远程服务器
    pub fn upload_file(
        &self,
        server_id: &str,
        local_path: &str,
        remote_path: &str,
    ) -> Result<u64, String> {
        let sftp = self.get_sftp(server_id)?;
        let local = Path::new(local_path);

        // 读取本地文件
        let local_data = std::fs::read(local).map_err(|e| format!("读取本地文件失败: {}", e))?;
        let file_size = local_data.len() as u64;

        // 创建远程文件并写入
        let mut remote_file = sftp
            .create(Path::new(remote_path))
            .map_err(|e| format!("创建远程文件失败: {}", e))?;

        remote_file
            .write_all(&local_data)
            .map_err(|e| format!("写入远程文件失败: {}", e))?;

        log::info!(
            "[SFTP] Uploaded {} ({}) to {}:{remote_path}",
            local_path,
            file_size,
            server_id
        );
        Ok(file_size)
    }

    /// 下载文件到本地
    pub fn download_file(
        &self,
        server_id: &str,
        remote_path: &str,
        local_path: &str,
    ) -> Result<u64, String> {
        let sftp = self.get_sftp(server_id)?;

        // 打开远程文件
        let mut remote_file = sftp
            .open(Path::new(remote_path))
            .map_err(|e| format!("打开远程文件失败: {}", e))?;

        // 读取远程文件内容
        let mut contents = Vec::new();
        remote_file
            .read_to_end(&mut contents)
            .map_err(|e| format!("读取远程文件失败: {}", e))?;

        let file_size = contents.len() as u64;

        // 确保本地目录存在
        if let Some(parent) = Path::new(local_path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建本地目录失败: {}", e))?;
        }

        // 写入本地文件
        std::fs::write(local_path, &contents).map_err(|e| format!("写入本地文件失败: {}", e))?;

        log::info!(
            "[SFTP] Downloaded {}:{} to {} ({})",
            server_id,
            remote_path,
            local_path,
            file_size
        );
        Ok(file_size)
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

        let sftp = self.get_sftp(server_id)?;

        // 创建远程目录
        let _ = sftp.mkdir(Path::new(remote_path), 0o755);

        let mut total_size: u64 = 0;
        for entry in std::fs::read_dir(local).map_err(|e| format!("读取本地目录失败: {}", e))? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let file_name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let new_remote = format!("{}/{}", remote_path, file_name);

            if path.is_dir() {
                let size = self.upload_dir_recursive(server_id, &path.to_string_lossy(), &new_remote)?;
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
            remote_path,
            total_size
        );
        Ok(total_size)
    }

    // ============ 内部辅助方法 ============

    fn get_sftp(&self, server_id: &str) -> Result<Sftp, String> {
        let conns = self.connections.lock().map_err(|e| e.to_string())?;
        let conn = conns
            .get(server_id)
            .ok_or_else(|| "服务器未连接".to_string())?;

        // 确保 session 处于阻塞模式（防止 read_terminal 切换导致的竞态）
        conn.session.set_blocking(true);

        conn.session
            .sftp()
            .map_err(|e| format!("创建 SFTP 会话失败: {}", e))
    }
}
