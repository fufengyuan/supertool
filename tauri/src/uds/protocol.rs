/// JSON-over-UDS 协议解析
///
/// 与 Electron 端 uds-api.ts 1:1 兼容：
///   请求: {"handler": "todos:add", "params": {...}}\n
///   响应: {"success": true, "data": {...}}\n
///   错误: {"success": false, "error": "..."}\n
///   流式: {"stream": true, "event": "data", "line": "..."}\n
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// UDS 请求 — 从 CLI (stool) 发来
#[derive(Debug, Deserialize)]
pub struct UdsRequest {
    pub handler: String,
    #[serde(default)]
    pub params: Option<Value>,
    #[serde(default)]
    pub stream: Option<bool>,
}

/// UDS 响应 — 发回给 CLI
#[derive(Debug, Serialize)]
pub struct UdsResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl UdsResponse {
    pub fn ok(data: Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }

    /// 序列化为 JSON 行（带 \n 终止符）
    pub fn to_line(&self) -> String {
        format!(
            "{}\n",
            serde_json::to_string(self)
                .unwrap_or_else(|_| r#"{"success":false,"error":"serialize error"}"#.to_string())
        )
    }
}

/// 流式事件 — 用于 log:tail / cicd:deploy-stream
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct StreamEvent {
    pub stream: bool,
    pub event: String,
    #[serde(flatten)]
    pub payload: Value,
}

impl StreamEvent {
    #[allow(dead_code)]
    pub fn new(event: &str, payload: Value) -> Self {
        Self {
            stream: true,
            event: event.to_string(),
            payload,
        }
    }

    #[allow(dead_code)]
    pub fn to_line(&self) -> String {
        format!("{}\n", serde_json::to_string(self).unwrap_or_default())
    }
}

/// 按行缓冲解析器 — 处理不完整 JSON 行
pub struct LineBuffer {
    buffer: String,
}

impl LineBuffer {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    /// 追加数据块，返回完整的 JSON 行列表
    pub fn push(&mut self, chunk: &[u8]) -> Vec<String> {
        // UTF-8 可能失败（概率极低），忽略无效字节
        let text = String::from_utf8_lossy(chunk);
        self.buffer.push_str(&text);

        let mut lines = Vec::new();
        let split: Vec<&str> = self.buffer.split('\n').collect();
        // 最后一个元素是不完整的行（或空），保留在 buffer 中
        for line in &split[..split.len() - 1] {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                lines.push(trimmed.to_string());
            }
        }
        self.buffer = split.last().unwrap_or(&"").to_string();
        lines
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_request() {
        let json = r#"{"handler":"todos:add","params":{"text":"test"}}"#;
        let req: UdsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.handler, "todos:add");
        assert!(req.params.is_some());
    }

    #[test]
    fn test_response_ok() {
        let resp = UdsResponse::ok(serde_json::json!({"id": "123"}));
        assert!(resp.success);
        assert_eq!(resp.data.unwrap()["id"], "123");
    }

    #[test]
    fn test_response_err() {
        let resp = UdsResponse::err("handler not found".to_string());
        assert!(!resp.success);
        assert_eq!(resp.error.unwrap(), "handler not found");
    }

    #[test]
    fn test_line_buffer() {
        let mut buf = LineBuffer::new();
        // 模拟分批接收数据
        let lines = buf.push(b"{\"handler\":\"test\"}\n");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], r#"{"handler":"test"}"#);

        // 不完整行
        let lines = buf.push(b"{\"handler\":\"partial\"");
        assert_eq!(lines.len(), 0);

        // 补全 + 新行
        let lines = buf.push(b"}\n{\"handler\":\"next\"}\n");
        assert_eq!(lines.len(), 2);
    }
}
