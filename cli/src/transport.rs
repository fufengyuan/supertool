use serde::Deserialize;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

/// Unix Domain Socket 传输层 — 纯 JSON over UDS，无 HTTP 协议
pub struct Transport {
    socket_path: String,
}

impl Transport {
    pub fn new() -> Self {
        Self {
            socket_path: Self::default_socket_path(),
        }
    }

    pub fn default_socket_path() -> String {
        std::env::var("SUPERTOOL_SOCKET")
            .ok()
            .or_else(|| {
                dirs::home_dir()
                    .map(|d| {
                        d.join(".supertool")
                            .join("supertool.sock")
                            .to_string_lossy()
                            .to_string()
                    })
            })
            .unwrap_or_else(|| "/tmp/supertool.sock".to_string())
    }

    /// 发送 JSON 请求并接收 JSON 响应
    pub fn request(
        &self,
        handler: &str,
        params: Option<&serde_json::Value>,
    ) -> anyhow::Result<serde_json::Value> {
        let req = serde_json::json!({
            "handler": handler,
            "params": params,
        });
        let req_str = format!("{}\n", serde_json::to_string(&req)?);

        let mut stream = UnixStream::connect(&self.socket_path)?;
        stream.set_read_timeout(Some(std::time::Duration::from_secs(30)))?;
        stream.set_write_timeout(Some(std::time::Duration::from_secs(30)))?;
        stream.write_all(req_str.as_bytes())?;

        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_line(&mut response)?;

        let resp: serde_json::Value = serde_json::from_str(&response)?;
        if resp.get("success").and_then(|v| v.as_bool()) == Some(true) {
            Ok(resp.get("data").cloned().unwrap_or(serde_json::Value::Null))
        } else {
            let err = resp
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            anyhow::bail!("{}", err)
        }
    }

    /// 流式请求，返回 BufReader 用于逐行读取
    pub fn stream_request(
        &self,
        handler: &str,
        params: Option<&serde_json::Value>,
    ) -> anyhow::Result<BufReader<UnixStream>> {
        let req = serde_json::json!({
            "handler": handler,
            "params": params,
            "stream": true,
        });
        let req_str = format!("{}\n", serde_json::to_string(&req)?);

        let mut stream = UnixStream::connect(&self.socket_path)?;
        stream.set_write_timeout(Some(std::time::Duration::from_secs(30)))?;
        stream.write_all(req_str.as_bytes())?;

        Ok(BufReader::new(stream))
    }

    pub fn health_check(&self) -> bool {
        self.request("update:get-version", None).is_ok()
    }
}

pub struct ApiClient {
    pub transport: Transport,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            transport: Transport::new(),
        }
    }

    /// 泛型请求方法 — 自动将 data 字段反序列化为 T
    pub fn request<T: for<'de> Deserialize<'de>>(
        &self,
        handler: &str,
        params: Option<&serde_json::Value>,
    ) -> anyhow::Result<T> {
        let resp = self.transport.request(handler, params)?;
        Ok(serde_json::from_value(resp)?)
    }

    /// 流式请求
    pub fn stream_request(
        &self,
        handler: &str,
        params: Option<&serde_json::Value>,
    ) -> anyhow::Result<BufReader<UnixStream>> {
        self.transport.stream_request(handler, params)
    }

    pub fn health_check(&self) -> bool {
        self.transport.health_check()
    }

    /// SSE 兼容的流式 tail（用于日志实时跟踪）
    pub fn sse_tail(&self, preset_id: &str, lines: usize, follow: bool) -> anyhow::Result<()> {
        let params = serde_json::json!({
            "presetId": preset_id,
            "lines": lines,
            "follow": follow,
        });
        let mut reader = self.transport.stream_request("log:tail", Some(&params))?;
        let mut line_buf = String::new();
        loop {
            line_buf.clear();
            let n = reader.read_line(&mut line_buf)?;
            if n == 0 {
                break;
            }
            let line = line_buf.trim();
            if line.is_empty() {
                continue;
            }
            // Try parsing as JSON stream event
            if let Ok(event) = serde_json::from_str::<serde_json::Value>(line) {
                if event.get("stream").and_then(|v| v.as_bool()) == Some(true) {
                    let event_type = event.get("event").and_then(|v| v.as_str()).unwrap_or("");
                    match event_type {
                        "start" => {
                            let name = event
                                .get("presetName")
                                .and_then(|v| v.as_str())
                                .unwrap_or(preset_id);
                            println!(
                                "\n  📋 实时日志: {} (Ctrl+C 退出)",
                                name
                            );
                            println!("  {}", "─".repeat(60));
                        }
                        "data" | "line" => {
                            if let Some(l) = event.get("line").and_then(|v| v.as_str()) {
                                println!("  {}", l);
                            } else if let Some(d) = event.get("data").and_then(|v| v.as_str()) {
                                println!("  {}", d);
                            }
                        }
                        "error" => {
                            let s = event
                                .get("serverName")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            let e = event.get("error").and_then(|v| v.as_str()).unwrap_or("");
                            eprintln!("  ❌ {} ({})", e, s);
                        }
                        "complete" => {
                            if !follow {
                                break;
                            }
                        }
                        _ => {}
                    }
                } else {
                    // Non-stream JSON response — could be error
                    if let Some(err) = event.get("error").and_then(|v| v.as_str()) {
                        eprintln!("  ❌ {}", err);
                    }
                }
            }
            std::io::stdout().flush()?;
        }
        Ok(())
    }
}
