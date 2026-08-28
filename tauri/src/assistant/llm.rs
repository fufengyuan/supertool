//! AI 助手 —— LLM 流式客户端（OpenAI / Anthropic 双协议）
//!
//! 设计要点：
//! - 请求体构造与 SSE 解析都是**纯函数**，可用录制下来的响应报文单测，无需联网；
//! - 上下文窗口从模型配置读入（见 `core::logic::ai_provider`），用于裁剪历史与限制输出；
//! - 只允许访问用户自己配置的提供商地址，助手侧不开放任意 URL 抓取能力。
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::safety::redact_secrets;
use supertool_core::logic::ai_provider::{AiProtocol, AiRoute};

/// 一张图片：媒体类型 + base64 数据（不含 data: 前缀）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBlock {
    /// 如 image/png、image/jpeg、image/webp
    pub media_type: String,
    /// base64 编码的图片字节
    pub data_base64: String,
}

/// 一次会话消息（内部统一用 OpenAI 形态表达，发往 Anthropic 时再做结构转换）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// system | user | assistant | tool
    pub role: String,
    pub content: String,
    /// 用户消息携带的图片（仅当前轮生效，不进历史裁剪预算）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub images: Vec<ImageBlock>,
    /// assistant 发起的工具调用
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
    /// role=tool 时对应的调用 id
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl ChatMessage {
    pub fn text(role: &str, content: impl Into<String>) -> Self {
        Self {
            role: role.to_string(),
            content: content.into(),
            images: Vec::new(),
            tool_calls: Vec::new(),
            tool_call_id: None,
            name: None,
        }
    }
    /// 带图片的用户消息（vision 模型用）
    pub fn user_with_images(content: impl Into<String>, images: Vec<ImageBlock>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
            images,
            tool_calls: Vec::new(),
            tool_call_id: None,
            name: None,
        }
    }
    pub fn system(content: impl Into<String>) -> Self {
        Self::text("system", content)
    }
    pub fn user(content: impl Into<String>) -> Self {
        Self::text("user", content)
    }
    pub fn tool(call_id: &str, name: &str, result: Value) -> Self {
        Self {
            role: "tool".to_string(),
            content: redact_secrets(&result).to_string(),
            images: Vec::new(),
            tool_calls: Vec::new(),
            tool_call_id: Some(call_id.to_string()),
            name: Some(name.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    /// 未解析的参数 JSON 字符串（模型可能分片下发）
    pub arguments: String,
}

#[derive(Debug, Clone)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    /// JSON Schema
    pub parameters: Value,
}

#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub tools: Vec<ToolSpec>,
    pub max_output_tokens: u32,
    pub temperature: Option<f64>,
}

/// 流式过程中向上抛出的事件
#[derive(Debug, Clone, PartialEq)]
pub enum LlmEvent {
    TextDelta(String),
    ThinkingDelta(String),
    /// 一个完整的工具调用（参数已拼装并校验为合法 JSON 才会抛出）
    ToolCall(ToolCall),
    Usage { input_tokens: u64, output_tokens: u64 },
}

/// 一轮模型回复的累计结果
#[derive(Debug, Default, Clone)]
pub struct AssistantTurn {
    pub text: String,
    pub thinking: String,
    pub tool_calls: Vec<ToolCall>,
    pub usage: Option<(u64, u64)>,
}

/// 给提示词用的路由摘要（刻意不含 apiKey，密钥没有任何进入上下文的路径）
#[derive(Debug, Clone)]
pub struct RouteInfo {
    pub provider_name: String,
    pub protocol: String,
    pub model_id: String,
    pub context_window: u32,
    /// 当前模型是否支持识图
    pub vision: bool,
}

// =================== 请求体构造（纯函数） ===================

fn openai_tools(tools: &[ToolSpec]) -> Vec<Value> {
    tools
        .iter()
        .map(|t| {
            json!({
                "type": "function",
                "function": {
                    "name": t.name,
                    "description": t.description,
                    "parameters": t.parameters,
                }
            })
        })
        .collect()
}

fn anthropic_tools(tools: &[ToolSpec]) -> Value {
    Value::Array(
        tools
            .iter()
            .map(|t| {
                json!({
                    "name": t.name,
                    "description": t.description,
                    "input_schema": t.parameters,
                })
            })
            .collect(),
    )
}

/// 相邻同角色合并（Anthropic 要求 user/assistant 严格交替）
fn push_turn(turns: &mut Vec<(String, Vec<Value>)>, role: &str, blocks: Vec<Value>) {
    if blocks.is_empty() {
        return;
    }
    match turns.last_mut() {
        Some((r, b)) if r == role => b.extend(blocks),
        _ => turns.push((role.to_string(), blocks)),
    }
}

/// 把内部消息序列转成 Anthropic 形态：
/// system 提到顶层、tool 结果转成 user 的 tool_result 块、assistant 工具调用转 tool_use 块，
/// 并保证角色严格交替、首条为 user（工具循环会产生连续 assistant / 连续 tool_result）。
fn anthropic_messages(messages: &[ChatMessage]) -> (Option<String>, Vec<Value>) {
    let mut system: Vec<String> = Vec::new();
    let mut turns: Vec<(String, Vec<Value>)> = Vec::new();

    for m in messages {
        match m.role.as_str() {
            "system" => system.push(m.content.clone()),
            "tool" => push_turn(
                &mut turns,
                "user",
                vec![json!({
                    "type": "tool_result",
                    "tool_use_id": m.tool_call_id.clone().unwrap_or_default(),
                    "content": m.content,
                })],
            ),
            "assistant" => {
                let mut blocks: Vec<Value> = Vec::new();
                if !m.content.is_empty() {
                    blocks.push(json!({ "type": "text", "text": m.content }));
                }
                for tc in &m.tool_calls {
                    let input: Value =
                        serde_json::from_str(&tc.arguments).unwrap_or_else(|_| json!({}));
                    blocks.push(json!({
                        "type": "tool_use", "id": tc.id, "name": tc.name, "input": input,
                    }));
                }
                push_turn(&mut turns, "assistant", blocks);
            }
            _ => {
                let mut blocks: Vec<Value> = Vec::new();
                if !m.content.is_empty() {
                    blocks.push(json!({ "type": "text", "text": m.content }));
                }
                for img in &m.images {
                    blocks.push(json!({
                        "type": "image",
                        "source": {
                            "type": "base64",
                            "media_type": img.media_type,
                            "data": img.data_base64,
                        },
                    }));
                }
                push_turn(&mut turns, "user", blocks);
            }
        }
    }

    // 首条必须是 user：多轮工具循环里可能出现「以 assistant 开头」的历史切片
    if turns.first().map(|(r, _)| r != "user").unwrap_or(true) {
        turns.insert(
            0,
            (
                "user".to_string(),
                vec![json!({ "type": "text", "text": "（接上文）" })],
            ),
        );
    }

    let out: Vec<Value> = turns
        .into_iter()
        .map(|(role, blocks)| json!({ "role": role, "content": blocks }))
        .collect();
    let sys = if system.is_empty() {
        None
    } else {
        Some(system.join("\n\n"))
    };
    (sys, out)
}

pub fn build_body(protocol: AiProtocol, req: &ChatRequest, stream: bool) -> Value {
    match protocol {
        AiProtocol::OpenAi => {
            let mut messages = Vec::new();
            for m in &req.messages {
                let mut obj = serde_json::Map::new();
                obj.insert("role".into(), json!(m.role));
                match m.role.as_str() {
                    "assistant" if !m.tool_calls.is_empty() => {
                        obj.insert("content".into(), json!(if m.content.is_empty() { Value::Null } else { json!(m.content) }));
                        obj.insert(
                            "tool_calls".into(),
                            Value::Array(
                                m.tool_calls
                                    .iter()
                                    .map(|tc| {
                                        json!({
                                            "id": tc.id,
                                            "type": "function",
                                            "function": { "name": tc.name, "arguments": tc.arguments },
                                        })
                                    })
                                    .collect(),
                            ),
                        );
                    }
                    "tool" => {
                        obj.insert("content".into(), json!(m.content));
                        obj.insert(
                            "tool_call_id".into(),
                            json!(m.tool_call_id.clone().unwrap_or_default()),
                        );
                    }
                    _ => {
                        // 用户消息带图时，content 转为内容块数组（OpenAI 多模态规范）
                        if m.role == "user" && !m.images.is_empty() {
                            let mut blocks: Vec<Value> = Vec::new();
                            if !m.content.is_empty() {
                                blocks.push(json!({ "type": "text", "text": m.content }));
                            }
                            for img in &m.images {
                                blocks.push(json!({
                                    "type": "image_url",
                                    "image_url": {
                                        "url": format!(
                                            "data:{};base64,{}",
                                            img.media_type, img.data_base64
                                        ),
                                    },
                                }));
                            }
                            obj.insert("content".into(), Value::Array(blocks));
                        } else {
                            obj.insert("content".into(), json!(m.content));
                        }
                    }
                }
                messages.push(Value::Object(obj));
            }
            let mut body = json!({
                "model": req.model,
                "messages": messages,
                "stream": stream,
            });
            // 部分网关不接受 max_tokens（=0 视为不限制时直接省略）
            if req.max_output_tokens > 0 {
                body["max_tokens"] = json!(req.max_output_tokens);
            }
            if let Some(t) = req.temperature {
                body["temperature"] = json!(t);
            }
            if !req.tools.is_empty() {
                body["tools"] = json!(openai_tools(&req.tools));
            }
            body
        }
        AiProtocol::Anthropic => {
            let (system, messages) = anthropic_messages(&req.messages);
            let mut body = json!({
                "model": req.model,
                "messages": messages,
                // Anthropic 把 max_tokens 定为必填
                "max_tokens": req.max_output_tokens.max(1),
                "stream": stream,
            });
            if let Some(s) = system {
                body["system"] = json!(s);
            }
            if let Some(t) = req.temperature {
                body["temperature"] = json!(t);
            }
            if !req.tools.is_empty() {
                body["tools"] = anthropic_tools(&req.tools);
            }
            body
        }
    }
}

/// 拼请求 URL：容忍用户把 baseUrl 填成 `/v1`、`/v1/` 或完整端点
pub fn build_url(protocol: AiProtocol, base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');
    match protocol {
        AiProtocol::OpenAi => {
            if base.ends_with("/chat/completions") {
                base.to_string()
            } else {
                format!("{}/chat/completions", base)
            }
        }
        AiProtocol::Anthropic => {
            if base.ends_with("/messages") {
                base.to_string()
            } else if base.ends_with("/v1") {
                format!("{}/messages", base)
            } else {
                format!("{}/v1/messages", base)
            }
        }
    }
}

/// 鉴权头：两套协议完全不同，别混
pub fn build_headers(protocol: AiProtocol, api_key: &str) -> Vec<(String, String)> {
    let mut headers = vec![("content-type".to_string(), "application/json".to_string())];
    if api_key.is_empty() {
        return headers; // 本机推理服务（Ollama 等）常无需 key
    }
    match protocol {
        AiProtocol::OpenAi => {
            headers.push(("authorization".to_string(), format!("Bearer {}", api_key)))
        }
        AiProtocol::Anthropic => {
            headers.push(("x-api-key".to_string(), api_key.to_string()));
            headers.push(("anthropic-version".to_string(), "2023-06-01".to_string()));
        }
    }
    headers
}

// =================== SSE 解析（纯函数 + 累加器） ===================

/// 一个流式响应的解析累加器：两种协议的工具调用参数都是分片下发的，
/// 必须按 index/id 归并，等块结束才产出一个完整 ToolCall。
#[derive(Debug, Default)]
pub struct SseAccumulator {
    protocol: AiProtocol,
    turn: AssistantTurn,
    /// OpenAI 按 index 归并；Anthropic 按 content block index
    partial_tools: Vec<(String, String, String)>, // (id, name, arguments)
    usage: Option<(u64, u64)>,
    finished: bool,
    /// usage 只在收尾时发一次（finish_reason 与 [DONE] 会先后到）
    usage_emitted: bool,
}

impl SseAccumulator {
    pub fn new(protocol: AiProtocol) -> Self {
        Self {
            protocol,
            ..Default::default()
        }
    }

    pub fn turn(&self) -> &AssistantTurn {
        &self.turn
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// 输入 SSE 里的 data 载荷（已去掉 `data:` 前缀）。返回本次产生的事件。
    pub fn feed(&mut self, payload: &str) -> Vec<LlmEvent> {
        let payload = payload.trim();
        if payload.is_empty() || payload == "[DONE]" {
            if payload == "[DONE]" {
                self.finished = true;
            }
            return self.flush_if_finished();
        }
        let Ok(chunk) = serde_json::from_str::<Value>(payload) else {
            return Vec::new(); // 非 JSON（ping/注释/网关自定义心跳）直接忽略
        };
        match self.protocol {
            AiProtocol::OpenAi => self.feed_openai(&chunk),
            AiProtocol::Anthropic => self.feed_anthropic(&chunk),
        }
    }

    fn flush_if_finished(&mut self) -> Vec<LlmEvent> {
        let mut events = Vec::new();
        if self.finished {
            events.extend(self.emit_pending_tools());
            events.extend(self.emit_usage());
        }
        events
    }

    fn emit_usage(&mut self) -> Vec<LlmEvent> {
        if self.usage_emitted {
            return Vec::new();
        }
        match self.usage {
            Some((i, o)) => {
                self.usage_emitted = true;
                vec![LlmEvent::Usage {
                    input_tokens: i,
                    output_tokens: o,
                }]
            }
            None => Vec::new(),
        }
    }

    fn emit_pending_tools(&mut self) -> Vec<LlmEvent> {
        let pending = std::mem::take(&mut self.partial_tools);
        pending
            .into_iter()
            .filter_map(|(id, name, arguments)| {
                // 按 index 预分配的占位槽（无名无参）不是真调用
                if name.is_empty() {
                    return None;
                }
                let args = arguments.trim().to_string();
                // 空参数补成 {}，非法 JSON 直接丢弃该调用（避免把半个片段喂给工具层）
                let normalized = if args.is_empty() {
                    "{}".to_string()
                } else {
                    args
                };
                if serde_json::from_str::<Value>(&normalized).is_err() {
                    log::warn!("[assistant] 工具 {} 参数 JSON 不完整，已丢弃", name);
                    return None;
                }
                let call = ToolCall {
                    id: if id.is_empty() {
                        uuid::Uuid::new_v4().to_string()
                    } else {
                        id
                    },
                    name,
                    arguments: normalized,
                };
                self.turn.tool_calls.push(call.clone());
                Some(LlmEvent::ToolCall(call))
            })
            .collect()
    }

    fn feed_openai(&mut self, chunk: &Value) -> Vec<LlmEvent> {
        let mut events = Vec::new();
        if let Some(usage) = chunk.get("usage").filter(|u| !u.is_null()) {
            self.usage = Some((
                usage["prompt_tokens"].as_u64().unwrap_or(0),
                usage["completion_tokens"].as_u64().unwrap_or(0),
            ));
        }
        let Some(choice) = chunk.get("choices").and_then(|c| c.get(0)) else {
            // 有些网关会把 usage 单独放在没有 choices 的最后一个 chunk 里
            if self.finished {
                events.extend(self.flush_if_finished());
            }
            return events;
        };
        let delta = &choice["delta"];
        if let Some(text) = delta["content"].as_str() {
            if !text.is_empty() {
                self.turn.text.push_str(text);
                events.push(LlmEvent::TextDelta(text.to_string()));
            }
        }
        // 兼容部分推理网关的 reasoning/reasoning_content 字段
        for key in ["reasoning_content", "reasoning"] {
            if let Some(t) = delta[key].as_str() {
                if !t.is_empty() {
                    self.turn.thinking.push_str(t);
                    events.push(LlmEvent::ThinkingDelta(t.to_string()));
                }
            }
        }
        if let Some(calls) = delta["tool_calls"].as_array() {
            for call in calls {
                let index = call["index"].as_u64().unwrap_or(0) as usize;
                while self.partial_tools.len() <= index {
                    self.partial_tools
                        .push((String::new(), String::new(), String::new()));
                }
                let slot = &mut self.partial_tools[index];
                if let Some(id) = call["id"].as_str() {
                    if !id.is_empty() {
                        slot.0 = id.to_string();
                    }
                }
                if let Some(name) = call["function"]["name"].as_str() {
                    if !name.is_empty() {
                        slot.1 = name.to_string();
                    }
                }
                if let Some(args) = call["function"]["arguments"].as_str() {
                    slot.2.push_str(args);
                }
            }
        }
        if choice["finish_reason"].as_str().is_some() {
            self.finished = true;
            events.extend(self.emit_pending_tools());
            events.extend(self.emit_usage());
        }
        events
    }

    fn feed_anthropic(&mut self, chunk: &Value) -> Vec<LlmEvent> {
        let mut events = Vec::new();
        let kind = chunk["type"].as_str().unwrap_or_default();
        match kind {
            "message_start" => {
                if let Some(u) = chunk["message"]["usage"].as_object() {
                    let get = |k: &str| u.get(k).and_then(|v| v.as_u64()).unwrap_or(0);
                    self.usage = Some((get("input_tokens"), get("output_tokens")));
                }
            }
            "content_block_start" => {
                let index = chunk["index"].as_u64().unwrap_or(0) as usize;
                let block = &chunk["content_block"];
                if block["type"].as_str() == Some("tool_use") {
                    while self.partial_tools.len() <= index {
                        self.partial_tools
                            .push((String::new(), String::new(), String::new()));
                    }
                    self.partial_tools[index] = (
                        block["id"].as_str().unwrap_or_default().to_string(),
                        block["name"].as_str().unwrap_or_default().to_string(),
                        String::new(),
                    );
                }
            }
            "content_block_delta" => {
                let delta = &chunk["delta"];
                match delta["type"].as_str().unwrap_or_default() {
                    "text_delta" => {
                        if let Some(t) = delta["text"].as_str() {
                            self.turn.text.push_str(t);
                            events.push(LlmEvent::TextDelta(t.to_string()));
                        }
                    }
                    "thinking_delta" => {
                        if let Some(t) = delta["thinking"].as_str() {
                            self.turn.thinking.push_str(t);
                            events.push(LlmEvent::ThinkingDelta(t.to_string()));
                        }
                    }
                    "input_json_delta" => {
                        let index = chunk["index"].as_u64().unwrap_or(0) as usize;
                        while self.partial_tools.len() <= index {
                            self.partial_tools
                                .push((String::new(), String::new(), String::new()));
                        }
                        if let Some(partial) = delta["partial_json"].as_str() {
                            self.partial_tools[index].2.push_str(partial);
                        }
                    }
                    _ => {}
                }
            }
            "message_delta" => {
                if let Some(u) = chunk["usage"]["output_tokens"].as_u64() {
                    let input = self.usage.map(|x| x.0).unwrap_or(0);
                    self.usage = Some((input, u));
                }
            }
            "message_stop" => {
                self.finished = true;
                events.extend(self.emit_pending_tools());
                events.extend(self.emit_usage());
            }
            "error" => {
                self.finished = true;
                log::error!(
                    "[assistant] 模型流返回错误: {}",
                    chunk["error"]["message"].as_str().unwrap_or("未知错误")
                );
            }
            _ => {} // ping / content_block_stop / 未知事件
        }
        events
    }
}

/// 从一行 SSE 文本取出 data 载荷（兼容 `data:{...}` 与 `data: {...}`）
pub fn sse_data_payload(line: &str) -> Option<&str> {
    let line = line.trim_end_matches('\r');
    let rest = line.strip_prefix("data:")?;
    Some(rest.trim_start())
}

/// 发起一次流式补全，事件通过 on_event 同步回调（调用方负责节流后再 emit）
pub async fn stream_completion(
    route: &AiRoute,
    req: &ChatRequest,
    on_event: &mut (dyn FnMut(LlmEvent) + Send),
) -> Result<AssistantTurn, String> {
    use futures::StreamExt;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let body = build_body(route.protocol, req, true);
    let url = build_url(route.protocol, &route.base_url);
    let mut request = client.post(&url);
    for (k, v) in build_headers(route.protocol, &route.api_key) {
        request = request.header(k, v);
    }
    let response = request
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("连接 {} 失败: {}", route.provider_name, e))?;

    let status = response.status();
    if !status.is_success() {
        // 不合规的网关可能把请求头/请求体回显在错误里，抹掉凭据形态再抛给上层
        let text = response.text().await.unwrap_or_default();
        return Err(format!(
            "模型返回 {}：{}",
            status.as_u16(),
            clip(&super::safety::redact_text(&text), 600)
        ));
    }

    let mut acc = SseAccumulator::new(route.protocol);
    let mut stream = response.bytes_stream();
    let mut buf = String::new();

    while let Some(item) = stream.next().await {
        let bytes = item.map_err(|e| format!("读取响应流失败: {}", e))?;
        buf.push_str(&String::from_utf8_lossy(&bytes));
        // SSE 以空行分隔事件，这里按行取 data 即可（多行 data 极少见）
        while let Some(pos) = buf.find('\n') {
            let line = buf[..pos].to_string();
            buf.drain(..=pos);
            if let Some(payload) = sse_data_payload(&line) {
                for event in acc.feed(payload) {
                    on_event(event);
                }
            }
        }
        if acc.is_finished() {
            break;
        }
    }
    // 收尾：处理未被换行切断的最后一行
    if let Some(payload) = sse_data_payload(&buf) {
        for event in acc.feed(payload) {
            on_event(event);
        }
    }
    if !acc.is_finished() {
        acc.finished = true;
        for event in acc.flush_if_finished() {
            on_event(event);
        }
    }
    Ok(acc.turn().clone())
}

pub fn clip(s: &str, max_chars: usize) -> String {
    match s.chars().count() > max_chars {
        true => {
            let mut out: String = s.chars().take(max_chars).collect();
            out.push_str("…(已截断)");
            out
        }
        false => s.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tools() -> Vec<ToolSpec> {
        vec![ToolSpec {
            name: "list_servers".to_string(),
            description: "列出服务器".to_string(),
            parameters: json!({"type": "object", "properties": {}}),
        }]
    }

    fn req(messages: Vec<ChatMessage>) -> ChatRequest {
        ChatRequest {
            model: "m-1".to_string(),
            messages,
            tools: tools(),
            max_output_tokens: 2048,
            temperature: Some(0.2),
        }
    }

    #[test]
    fn openai_body_keeps_system_in_messages_and_tools_shape() {
        let body = build_body(
            AiProtocol::OpenAi,
            &req(vec![
                ChatMessage::system("你是配置助手"),
                ChatMessage::user("建个服务器"),
            ]),
            true,
        );
        assert_eq!(body["model"], "m-1");
        assert_eq!(body["messages"].as_array().unwrap().len(), 2);
        assert_eq!(body["messages"][0]["role"], "system");
        assert_eq!(body["max_tokens"], 2048);
        assert_eq!(body["temperature"], 0.2);
        assert_eq!(body["tools"][0]["function"]["name"], "list_servers");
        assert_eq!(
            body["tools"][0]["function"]["parameters"]["type"],
            "object"
        );
    }

    #[test]
    fn anthropic_body_lifts_system_and_renames_input_schema() {
        let body = build_body(
            AiProtocol::Anthropic,
            &req(vec![
                ChatMessage::system("A"),
                ChatMessage::system("B"),
                ChatMessage::user("hi"),
            ]),
            true,
        );
        assert_eq!(body["system"], "A\n\nB");
        assert_eq!(body["messages"].as_array().unwrap().len(), 1);
        assert_eq!(body["messages"][0]["role"], "user");
        assert_eq!(body["tools"][0]["name"], "list_servers");
        assert!(body["tools"][0].get("input_schema").is_some());
        assert!(body["tools"][0].get("function").is_none());
    }

    #[test]
    fn anthropic_converts_tool_calls_and_results() {
        let assistant = ChatMessage {
            role: "assistant".to_string(),
            content: String::new(),
            images: Vec::new(),
            tool_calls: vec![ToolCall {
                id: "tu_1".to_string(),
                name: "list_servers".to_string(),
                arguments: "{\"a\":1}".to_string(),
            }],
            tool_call_id: None,
            name: None,
        };
        let body = build_body(
            AiProtocol::Anthropic,
            &req(vec![assistant.clone(), ChatMessage::tool("tu_1", "list_servers", json!({"ok":true}))]),
            true,
        );
        let msgs = body["messages"].as_array().unwrap();
        // 历史以 assistant 开头时会自动补一条 user 桥接，这里按角色取而不是按下标
        let assistant_msg = msgs.iter().find(|m| m["role"] == "assistant").unwrap();
        assert_eq!(assistant_msg["content"][0]["type"], "tool_use");
        assert_eq!(assistant_msg["content"][0]["input"]["a"], 1);
        let tool_msg = msgs
            .iter()
            .find(|m| m["content"][0]["type"] == "tool_result")
            .expect("应有 tool_result");
        assert_eq!(tool_msg["role"], "user");
        assert_eq!(tool_msg["content"][0]["tool_use_id"], "tu_1");

        // OpenAI 侧：assistant 带 tool_calls，结果用 role=tool
        let oai = build_body(
            AiProtocol::OpenAi,
            &req(vec![assistant, ChatMessage::tool("tu_1", "list_servers", json!({"ok":true}))]),
            true,
        );
        let om = oai["messages"].as_array().unwrap();
        assert_eq!(om[0]["tool_calls"][0]["function"]["name"], "list_servers");
        assert_eq!(om[0]["content"], Value::Null);
        assert_eq!(om[1]["role"], "tool");
        assert_eq!(om[1]["tool_call_id"], "tu_1");
    }

    /// 带图用户消息：OpenAI 用 image_url(data URI)，Anthropic 用 image(base64 source)
    #[test]
    fn vision_images_are_encoded_per_protocol() {
        let img = ImageBlock {
            media_type: "image/png".to_string(),
            data_base64: "QUJDRA==".to_string(),
        };
        let user = ChatMessage::user_with_images("看这张图", vec![img]);

        let oai = build_body(AiProtocol::OpenAi, &req(vec![user.clone()]), true);
        let content = oai["messages"][0]["content"].as_array().unwrap();
        assert_eq!(content[0]["type"], "text");
        assert_eq!(content[0]["text"], "看这张图");
        assert_eq!(content[1]["type"], "image_url");
        assert_eq!(
            content[1]["image_url"]["url"],
            "data:image/png;base64,QUJDRA=="
        );

        let anth = build_body(AiProtocol::Anthropic, &req(vec![user]), true);
        let blocks = anth["messages"][0]["content"].as_array().unwrap();
        assert_eq!(blocks[0]["type"], "text");
        assert_eq!(blocks[1]["type"], "image");
        assert_eq!(blocks[1]["source"]["type"], "base64");
        assert_eq!(blocks[1]["source"]["media_type"], "image/png");
        assert_eq!(blocks[1]["source"]["data"], "QUJDRA==");
    }

    /// 纯文本用户消息不带图时，content 仍是字符串（不破坏现有请求形态）
    #[test]
    fn text_only_user_keeps_plain_content() {
        let oai = build_body(AiProtocol::OpenAi, &req(vec![ChatMessage::user("hi")]), true);
        assert_eq!(oai["messages"][0]["content"], "hi");
    }

    /// 工具循环会产生连续 assistant / 连续 tool_result，Anthropic 要求严格交替且首条为 user
    #[test]
    fn anthropic_forces_role_alternation_and_user_first() {
        let call = |id: &str| ToolCall {
            id: id.to_string(),
            name: "list_servers".to_string(),
            arguments: "{}".to_string(),
        };
        let msgs = vec![
            ChatMessage::system("s"),
            // 连续两条 assistant（模型说完话又发起调用）
            ChatMessage::text("assistant", "我先看看"),
            ChatMessage {
                role: "assistant".into(),
                content: String::new(),
                images: Vec::new(),
                tool_calls: vec![call("t1"), call("t2")],
                tool_call_id: None,
                name: None,
            },
            // 连续两个工具结果
            ChatMessage::tool("t1", "list_servers", json!({"a": 1})),
            ChatMessage::tool("t2", "list_servers", json!({"b": 2})),
            ChatMessage::user("然后呢"),
        ];
        let body = build_body(AiProtocol::Anthropic, &req(msgs), true);
        let list = body["messages"].as_array().unwrap();
        let roles: Vec<&str> = list.iter().map(|m| m["role"].as_str().unwrap()).collect();
        assert_eq!(roles.first().copied(), Some("user"), "首条必须是 user");
        for w in roles.windows(2) {
            assert_ne!(w[0], w[1], "角色必须交替，实际 {roles:?}");
        }
        // 合并后：连续的两次工具调用结果应在同一条 user 消息里
        let with_results = list
            .iter()
            .find(|m| m["content"].as_array().map(|b| b.iter().any(|x| x["type"] == "tool_result")).unwrap_or(false))
            .expect("应有 tool_result 消息");
        let blocks = with_results["content"].as_array().unwrap();
        assert_eq!(blocks.iter().filter(|b| b["type"] == "tool_result").count(), 2);
        // 连续 assistant 合并成一条，文本与 tool_use 都在里面
        let assistant = list.iter().find(|m| m["role"] == "assistant").unwrap();
        let kinds: Vec<&str> = assistant["content"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| b["type"].as_str().unwrap())
            .collect();
        assert_eq!(kinds, vec!["text", "tool_use", "tool_use"]);
    }

    #[test]
    fn urls_and_headers_per_protocol() {
        assert_eq!(
            build_url(AiProtocol::OpenAi, "https://x.com/v1/"),
            "https://x.com/v1/chat/completions"
        );
        assert_eq!(
            build_url(AiProtocol::OpenAi, "https://x.com/v1/chat/completions"),
            "https://x.com/v1/chat/completions"
        );
        assert_eq!(
            build_url(AiProtocol::Anthropic, "https://x.com"),
            "https://x.com/v1/messages"
        );
        assert_eq!(
            build_url(AiProtocol::Anthropic, "https://x.com/v1"),
            "https://x.com/v1/messages"
        );

        let h = build_headers(AiProtocol::Anthropic, "sk-1");
        assert!(h.iter().any(|(k, v)| k == "x-api-key" && v == "sk-1"));
        assert!(h.iter().any(|(k, _)| k == "anthropic-version"));
        assert!(!h.iter().any(|(k, _)| k == "authorization"));
        let h2 = build_headers(AiProtocol::OpenAi, "sk-1");
        assert!(h2.iter().any(|(k, v)| k == "authorization" && v == "Bearer sk-1"));
        // 本机无 key 的服务不应发出空鉴权头
        assert!(!build_headers(AiProtocol::OpenAi, "")
            .iter()
            .any(|(k, _)| k == "authorization"));
    }

    /// 录制的 OpenAI 流：文本增量 + 分片工具调用参数
    #[test]
    fn accumulates_openai_stream_including_fragmented_tool_args() {
        let frames: Vec<Value> = vec![
            json!({"choices":[{"delta":{"role":"assistant","content":"好的"}}]}),
            json!({"choices":[{"delta":{"content":"，我看下"}}]}),
            json!({"choices":[{"delta":{"tool_calls":[
                {"index":0,"id":"call_9","function":{"name":"list_servers","arguments":""}}]}}]}),
            json!({"choices":[{"delta":{"tool_calls":[
                {"index":0,"function":{"arguments":"{\"group"}}]}}]}),
            json!({"choices":[{"delta":{"tool_calls":[
                {"index":0,"function":{"arguments":"Id\":\"g1\"}"}}]}}]}),
            json!({"choices":[{"delta":{},"finish_reason":"tool_calls"}],
                   "usage":{"prompt_tokens":11,"completion_tokens":5}}),
        ];
        let mut acc = SseAccumulator::new(AiProtocol::OpenAi);
        let mut events = Vec::new();
        for f in &frames {
            events.extend(acc.feed(&f.to_string()));
        }
        assert!(acc.is_finished());
        assert_eq!(acc.turn().text, "好的，我看下");
        assert_eq!(acc.turn().tool_calls.len(), 1);
        let call = &acc.turn().tool_calls[0];
        assert_eq!(call.id, "call_9");
        assert_eq!(call.name, "list_servers");
        assert_eq!(call.arguments, r#"{"groupId":"g1"}"#);
        assert_eq!(
            serde_json::from_str::<Value>(&call.arguments).unwrap()["groupId"],
            "g1"
        );
        assert!(events.contains(&LlmEvent::TextDelta("好的".to_string())));
        assert!(events.contains(&LlmEvent::Usage {
            input_tokens: 11,
            output_tokens: 5
        }));
    }

    /// 截断的工具参数不能当合法调用发出（否则会把半个 JSON 塞进工具层）
    #[test]
    fn drops_incomplete_openai_tool_arguments() {
        let mut acc = SseAccumulator::new(AiProtocol::OpenAi);
        acc.feed(
            &json!({"choices":[{"delta":{"tool_calls":[
                {"index":0,"id":"c1","function":{"name":"t","arguments":"{\"a\":"}}]}}]})
                .to_string(),
        );
        let events = acc.feed(
            &json!({"choices":[{"delta":{},"finish_reason":"tool_calls"}]}).to_string(),
        );
        assert!(events.iter().all(|e| !matches!(e, LlmEvent::ToolCall(_))));
        assert!(acc.turn().tool_calls.is_empty());
    }

    /// 只给 index 不给名字的占位分片不应被当成调用
    #[test]
    fn ignores_padded_tool_slots_without_name() {
        let mut acc = SseAccumulator::new(AiProtocol::OpenAi);
        acc.feed(&json!({"choices":[{"delta":{"tool_calls":[{"index":3,"function":{"arguments":"{}"}}]}}]}).to_string());
        let events = acc.feed(&json!({"choices":[{"delta":{},"finish_reason":"tool_calls"}]}).to_string());
        assert!(events.iter().all(|e| !matches!(e, LlmEvent::ToolCall(_))));
    }

    /// 录制的 Anthropic 流：text/thinking/input_json 三种增量 + message_stop
    #[test]
    fn accumulates_anthropic_stream() {
        let frames: Vec<Value> = vec![
            json!({"type":"message_start","message":{"usage":{"input_tokens":7,"output_tokens":1}}}),
            json!({"type":"content_block_start","index":0,"content_block":{"type":"thinking"}}),
            json!({"type":"content_block_delta","index":0,"delta":{"type":"thinking_delta","thinking":"先看看"}}),
            json!({"type":"content_block_start","index":1,"content_block":{"type":"text"}}),
            json!({"type":"content_block_delta","index":1,"delta":{"type":"text_delta","text":"我来列一下"}}),
            json!({"type":"content_block_start","index":2,"content_block":{
                "type":"tool_use","id":"tu_5","name":"list_servers"}}),
            json!({"type":"content_block_delta","index":2,"delta":{"type":"input_json_delta","partial_json":"{\"g"}}),
            json!({"type":"content_block_delta","index":2,"delta":{
                "type":"input_json_delta","partial_json":"roupId\":\"a\"}"}}),
            json!({"type":"message_delta","usage":{"output_tokens":42}}),
            json!({"type":"message_stop"}),
        ];
        let mut acc = SseAccumulator::new(AiProtocol::Anthropic);
        for f in &frames {
            acc.feed(&f.to_string());
        }
        assert!(acc.is_finished());
        assert_eq!(acc.turn().text, "我来列一下");
        assert_eq!(acc.turn().thinking, "先看看");
        assert_eq!(acc.turn().tool_calls.len(), 1);
        assert_eq!(acc.turn().tool_calls[0].id, "tu_5");
        assert_eq!(acc.turn().tool_calls[0].arguments, r#"{"groupId":"a"}"#);
        assert_eq!(acc.usage, Some((7, 42)));
    }

    #[test]
    fn ignores_heartbeat_and_unknown_frames() {
        let mut acc = SseAccumulator::new(AiProtocol::OpenAi);
        assert!(acc.feed(": keep-alive").is_empty());
        assert!(acc.feed("not json").is_empty());
        assert!(acc.feed("{}").is_empty());
        assert!(!acc.is_finished());
        assert_eq!(sse_data_payload("data:{\"a\":1}"), Some(r#"{"a":1}"#));
        assert_eq!(sse_data_payload("event: foo"), None);
    }
}
