/// UDS Server — Unix Domain Socket 监听器
///
/// 替代 Electron 的 net.createServer()，使用 tokio::net::UnixListener。
/// 每个连接一个独立 task，支持并发多个 CLI 实例。
use crate::core::CoreService;
use crate::uds::protocol::{LineBuffer, UdsRequest, UdsResponse};
use crate::uds::router::JsonRouter;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;

/// UDS Server 实例
pub struct UdsServer {
    pub socket_path: PathBuf,
    router: Arc<JsonRouter>,
    core: Arc<CoreService>,
    shutdown: Arc<Mutex<bool>>,
}

impl UdsServer {
    pub fn new(socket_path: PathBuf, core: Arc<CoreService>) -> Self {
        Self {
            socket_path,
            router: Arc::new(JsonRouter::new()),
            core,
            shutdown: Arc::new(Mutex::new(false)),
        }
    }

    /// 启动 UDS Server（阻塞直到 shutdown）
    pub async fn start(&self) -> std::io::Result<()> {
        // 清理旧的 socket 文件
        if self.socket_path.exists() {
            let _ = std::fs::remove_file(&self.socket_path);
        }

        // 确保父目录存在
        if let Some(parent) = self.socket_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let listener = UnixListener::bind(&self.socket_path)?;

        // 设置 socket 权限（允许本地所有用户访问）
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ =
                std::fs::set_permissions(&self.socket_path, std::fs::Permissions::from_mode(0o766));
        }

        let handler_count = self.router.handler_count();
        log::info!(
            "[UDS] Listening on unix://{} ({} handlers)",
            self.socket_path.display(),
            handler_count
        );

        loop {
            let (stream, _addr) = match listener.accept().await {
                Ok(v) => v,
                Err(e) => {
                    log::error!("[UDS] Accept error: {}", e);
                    continue;
                }
            };

            let router = self.router.clone();
            let core = self.core.clone();
            let shutdown = self.shutdown.clone();

            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(stream, router, core, shutdown).await {
                    log::error!("[UDS] Connection handler error: {}", e);
                }
            });
        }
    }

    /// 处理单个 UDS 连接
    async fn handle_connection(
        stream: UnixStream,
        router: Arc<JsonRouter>,
        core: Arc<CoreService>,
        _shutdown: Arc<Mutex<bool>>,
    ) -> std::io::Result<()> {
        let (mut reader, mut writer) = stream.into_split();
        let mut buf = [0u8; 4096];
        let mut line_buf = LineBuffer::new();

        loop {
            let n = match reader.read(&mut buf).await {
                Ok(0) => break, // EOF — CLI 断开连接
                Ok(n) => n,
                Err(e) => {
                    log::warn!("[UDS] Read error: {}", e);
                    break;
                }
            };

            let lines = line_buf.push(&buf[..n]);
            for line in lines {
                // 解析 JSON 请求
                let req: UdsRequest = match serde_json::from_str(&line) {
                    Ok(r) => r,
                    Err(e) => {
                        let resp = UdsResponse::err(format!("Invalid JSON: {}", e));
                        let _ = writer.write_all(resp.to_line().as_bytes()).await;
                        continue;
                    }
                };

                // 流式请求（log:tail, cicd:deploy-stream）— 特殊处理
                if req.stream == Some(true) {
                    match req.handler.as_str() {
                        "log:tail" => {
                            if let Err(e) = Self::handle_log_stream(&mut writer, &core, &req).await
                            {
                                let resp = UdsResponse::err(e);
                                let _ = writer.write_all(resp.to_line().as_bytes()).await;
                            }
                            continue;
                        }
                        "cicd:deploy-stream" => {
                            if let Err(e) =
                                Self::handle_cicd_deploy_stream(&mut writer, &core, &req).await
                            {
                                let resp = UdsResponse::err(e);
                                let _ = writer.write_all(resp.to_line().as_bytes()).await;
                            }
                            continue;
                        }
                        _ => {}
                    }
                }

                // 非流式请求：正常 dispatch（脱敏后记录日志）
                log::info!("[UDS] → {} params={}", req.handler, crate::core::log_sanitizer::sanitize_params_for_log(req.params.as_ref().unwrap_or(&serde_json::Value::Null), 200));
                let t0 = std::time::Instant::now();
                let resp = router.dispatch(&req, core.clone()).await;
                let elapsed = t0.elapsed().as_millis();
                let success = resp.success;
                log::info!("[UDS] ← {} {} {}ms", req.handler, if success { "✅" } else { "❌" }, elapsed);
                if let Err(e) = writer.write_all(resp.to_line().as_bytes()).await {
                    log::warn!("[UDS] Write error: {}", e);
                    break;
                }
            }
        }

        log::debug!("[UDS] Connection closed");
        Ok(())
    }

    /// 流式日志 tail — 保持连接开放，持续写入 JSON 行
    async fn handle_log_stream(
        writer: &mut tokio::net::unix::OwnedWriteHalf,
        core: &CoreService,
        req: &UdsRequest,
    ) -> Result<(), String> {
        let params = req.params.as_ref().ok_or("Missing params")?;
        let preset_id = params["presetId"].as_str().ok_or("Missing presetId")?;
        let lines = params["lines"].as_u64().unwrap_or(100) as usize;

        // 发送开始事件
        let start_event = serde_json::json!({
            "stream": true,
            "event": "start",
            "presetName": preset_id
        });
        let _ = writer
            .write_all(format!("{}\n", start_event).as_bytes())
            .await;

        // 非流式 tail 结果（后续实现 SSH 流式后替换）
        let result = core.log_tail(preset_id, lines).await?;
        let data_event = serde_json::json!({
            "stream": true,
            "event": "data",
            "data": result
        });
        let _ = writer
            .write_all(format!("{}\n", data_event).as_bytes())
            .await;

        let complete_event = serde_json::json!({
            "stream": true,
            "event": "complete"
        });
        let _ = writer
            .write_all(format!("{}\n", complete_event).as_bytes())
            .await;

        Ok(())
    }

    /// 流式 CI/CD 部署
    async fn handle_cicd_deploy_stream(
        writer: &mut tokio::net::unix::OwnedWriteHalf,
        core: &CoreService,
        req: &UdsRequest,
    ) -> Result<(), String> {
        let params = req.params.as_ref().ok_or("Missing params")?;
        let config_id = params["configId"].as_str().ok_or("Missing configId")?;

        let start_event = serde_json::json!({
            "stream": true,
            "event": "start",
            "configName": config_id
        });
        let _ = writer
            .write_all(format!("{}\n", start_event).as_bytes())
            .await;

        // 执行部署（后续实现 SSH 流式后替换）
        let result = core.cicd_deploy(config_id).await?;
        let complete_event = serde_json::json!({
            "stream": true,
            "event": "complete",
            "success": true,
            "deployLogId": result
        });
        let _ = writer
            .write_all(format!("{}\n", complete_event).as_bytes())
            .await;

        Ok(())
    }

    /// 请求停止
    pub async fn stop(&self) {
        let mut flag = self.shutdown.lock().await;
        *flag = true;
    }
}

/// 创建 UDS socket 路径
pub fn default_socket_path() -> PathBuf {
    crate::core::data_dir::uds_socket_path()
}

/// 从环境变量或默认路径获取 socket 路径
pub fn resolve_socket_path() -> PathBuf {
    std::env::var("SUPERTOOL_SOCKET")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(default_socket_path)
}
