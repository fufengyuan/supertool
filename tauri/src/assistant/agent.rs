//! AI 助手 —— 多轮工具调用循环
//!
//! 事件投递纪律（沿用 6.50.6 部署卡死的教训）：macOS 上每次 `app.emit` 都要回主线程
//! 做一次 webview eval，所以文本增量在后端攒批（≥120 字或 ≥80ms 才发一次），
//! 而不是每个 token 发一条事件。

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

use serde_json::{Value, json};
use supertool_core::logic::CoreService;
use tauri::{AppHandle, Emitter};

use super::{context, llm, safety, tools};

/// 单轮对话最多允许模型连续调用几次工具（防止无限打转）
pub const MAX_TOOL_ROUNDS: usize = 8;
const DELTA_FLUSH_CHARS: usize = 120;
const DELTA_FLUSH_MS: u128 = 80;

static RUNNING_TURNS: LazyLock<Mutex<HashMap<String, (tokio::task::JoinHandle<()>, Arc<AtomicBool>)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// 中止一次回答（前端「停止」按钮）
pub fn abort_turn(turn_id: &str) -> bool {
    let taken = RUNNING_TURNS.lock().map(|mut m| m.remove(turn_id)).ok().flatten();
    match taken {
        Some((handle, cancel)) => {
            cancel.store(true, Ordering::SeqCst);
            handle.abort();
            true
        }
        None => false,
    }
}

pub fn active_turn_count() -> usize {
    RUNNING_TURNS.lock().map(|m| m.len()).unwrap_or(0)
}

fn emit_event(app: &AppHandle, turn_id: &str, payload: Value) {
    let mut obj = match payload {
        Value::Object(map) => map,
        other => {
            let mut map = serde_json::Map::new();
            map.insert("detail".into(), other);
            map
        }
    };
    obj.insert("turnId".into(), json!(turn_id));
    let _ = app.emit("assistant-event", Value::Object(obj));
}

/// 文本增量攒批器（见文件头注释）
#[derive(Default)]
struct DeltaBatcher {
    buf: String,
    last: Option<std::time::Instant>,
}

impl DeltaBatcher {
    fn push(&mut self, delta: &str) -> Option<String> {
        self.buf.push_str(delta);
        let due = self.buf.chars().count() >= DELTA_FLUSH_CHARS
            || self
                .last
                .map(|t| t.elapsed().as_millis() >= DELTA_FLUSH_MS)
                .unwrap_or(true);
        if due {
            self.last = Some(std::time::Instant::now());
            if self.buf.is_empty() {
                return None;
            }
            return Some(std::mem::take(&mut self.buf));
        }
        None
    }

    fn take(&mut self) -> Option<String> {
        if self.buf.is_empty() {
            None
        } else {
            Some(std::mem::take(&mut self.buf))
        }
    }
}

pub fn system_prompt(route: &llm::RouteInfo, tool_names: &[String]) -> String {
    format!(
        "你是 SuperTool（一款本地运维桌面工具）里的「配置助手」。你的职责是：帮用户把各功能模块的参数配好，\
         并教会他们怎么用。\n\n\
        你能用的工具（{count} 个）：{tools}。\n\n\
        【交互录入 —— 优先于一切口头追问】\n\
        用户要录入结构化信息时，只允许用下面两个工具收集，禁止在正文里罗列字段/选项让用户打字回复：\n\
        · 需要收集 2 个及以上字段（新增服务器、新建部署配置、补数据库连接等）→ 调用 request_form 一次性弹出表单。\n\
        · 只需要用户做一个选择或回答一句短话 → 调用 ask：single/multiple 必须给 options 候选让用户勾选，text 让用户自由输入。\n\
        卡片弹出后，正文只写一句引导（如「请填写上方表单」），不要重复字段、不要重复选项、不要替用户回答。\n\
        敏感字段（密码/密钥）type 用 password，name 必须是标准凭据名（password/sshKeyPath/apiKey/token/secret/privateKey），\
        值只保存在本地并自动带入确认卡片，永远不要写进对话。\n\n\
        硬性规则，任何情况下都不得违反：\n\
        1) 你没有写配置的能力，也没有执行命令、写 SQL、访问网络的能力。要改配置只能调用 \
           propose_config_change 生成「变更提案」，由用户在确认卡片上点确认后才由界面写入。\n           你可以用 find_local_path / inspect_local_path / detect_local_project 查本机路径与目录结构\
           （只有路径、类型、大小、有没有 pom.xml/package.json 这类元信息，拿不到文件内容），\
           填 localPath、构建目录、产物目录之前先这样确认真实路径，不要凭猜。\n\
        2) 永远不要索要、猜测或转述密码、SSH 私钥、apiKey、token 等凭据。工具返回里这类字段是 [已隐藏]，\
           提案里出现这类字段会被直接拒绝——遇到需要凭据的场合，告诉用户自己在表单里填哪一格。\n\
        3) 不给猜测性的结论。涉及具体某条配置时先用读类工具看真实值（list_cicd_configs / get_cicd_config /\
           validate_cicd_config / analyze_deploy_error），基于返回值回答。\n\
        4) 解释字段含义或使用步骤前，先用 search_usage_guides 查内置知识库；查不到就明说不确定，\
           不要编造工具里没有的规则。引用知识条目时给出条目标题，方便用户回看。\n\
        5) 用户问「怎么操作」时，可以在给出步骤的同时调用 open_config_page 把他们带过去。\n\
        6) 提案里的 fields 必须是可直接使用的完整值（不要占位符、不要 JSON 字符串化），\
           一次提案只改一个目标，改动多就分多条提案让用户逐条确认。\n\n\
        【本项目的问题排查 —— 先查指南，再翻源码】\n\
        用户问的是「本工具（SuperTool 项目自身）」的实现、bug、某个功能怎么做的、怎么改时：\n\
        · 先 search_project_guides 查内嵌的项目指南（AGENTS.md 约定 + docs/ 文档快照），那里有踩坑结论与架构说明。\n\
        · 指南不够就用 search_project_source 在源码里检索关键词定位（返回 文件:行号 + 片段），\
          再 read_project_source 读那个文件看完整上下文。\n\
        · 只读本项目根；读不到就是不在范围，不要编造代码内容，也不要声称能改源码。\n\n\
        风格：中文、简洁、面向操作。先给结论或下一步动作，再给原因。不要长篇复述配置内容。\n\
        当前接入模型：{provider}（协议 {protocol}，模型 {model}，上下文窗口 {window} tokens）。",
        count = tool_names.len(),
        tools = tool_names.join("、"),
        provider = route.provider_name,
        protocol = route.protocol,
        model = route.model_id,
        window = route.context_window,
    )
}

/// 在后台任务里跑一轮完整对话（含多轮工具调用）。
/// 同步登记 + 异步执行：命令立即返回，过程靠事件推送，避免长任务占住 IPC 调用。
pub fn run_turn(
    app: AppHandle,
    core: CoreService,
    turn_id: String,
    user_message: String,
    history: Vec<llm::ChatMessage>,
) {
    let cancel = Arc::new(AtomicBool::new(false));
    let handle = tokio::spawn({
        let app = app.clone();
        let cancel = cancel.clone();
        let turn_id = turn_id.clone();
        async move {
            run_inner(app, core, &turn_id, user_message, history, cancel).await;
        }
    });
    // 任务可能在登记前就结束了（Guard 先移除、这里后插入会留残骸），登记后补一次清理
    if let Ok(mut m) = RUNNING_TURNS.lock() {
        if handle.is_finished() {
            drop(handle);
        } else {
            m.insert(turn_id, (handle, cancel));
        }
    }
}

async fn run_inner(
    app: AppHandle,
    core: CoreService,
    turn_id: &str,
    user_message: String,
    history: Vec<llm::ChatMessage>,
    cancel: Arc<AtomicBool>,
) {
    let _guard = FinishGuard(turn_id.to_string());

    let route = match core.resolve_ai_route() {
        Ok(r) => r,
        Err(e) => {
            emit_event(
                &app,
                turn_id,
                json!({"type": "error", "message": e, "needConfig": true}),
            );
            return;
        }
    };
    let specs = tools::tool_specs();
    let names: Vec<String> = specs.iter().map(|s| s.name.clone()).collect();

    let mut messages = vec![llm::ChatMessage::system(system_prompt(
        &llm::RouteInfo {
            provider_name: route.provider_name.clone(),
            protocol: route.protocol.as_str().to_string(),
            model_id: route.model_id.clone(),
            context_window: route.context_window,
        },
        &names,
    ))];
    messages.extend(history);
    messages.push(llm::ChatMessage::user(user_message));

    let context_window = route.context_window;
    let reserve = route.max_output_tokens;

    for round in 0..MAX_TOOL_ROUNDS {
        if cancel.load(Ordering::SeqCst) {
            break;
        }
        let trimmed = context::trim_to_budget(&messages, context_window, reserve);
        if round > 0 && trimmed.len() < messages.len() {
            emit_event(
                &app,
                turn_id,
                json!({"type": "notice", "message": "上下文接近窗口上限，已省略较早的对话"}),
            );
        }
        messages = trimmed;

        let request = llm::ChatRequest {
            model: route.model_id.clone(),
            messages: messages.clone(),
            tools: specs.clone(),
            max_output_tokens: reserve,
            temperature: Some(0.3),
        };

        emit_event(&app, turn_id, json!({"type": "start", "round": round}));
        // 正文与思考分别攒批：混在一个缓冲里会把思考内容当成正文渲染
        let mut batcher = DeltaBatcher::default();
        let mut thinking_batcher = DeltaBatcher::default();
        let mut thinking_open = false;
        let outcome = llm::stream_completion(&route, &request, &mut |event| {
            match event {
                llm::LlmEvent::TextDelta(delta) => {
                    if let Some(chunk) = batcher.push(&delta) {
                        emit_event(&app, turn_id, json!({"type": "delta", "text": chunk}));
                    }
                }
                llm::LlmEvent::ThinkingDelta(delta) => {
                    if !thinking_open {
                        thinking_open = true;
                        emit_event(&app, turn_id, json!({"type": "thinking-start"}));
                    }
                    if let Some(chunk) = thinking_batcher.push(&delta) {
                        emit_event(&app, turn_id, json!({"type": "thinking", "text": chunk}));
                    }
                }
                llm::LlmEvent::ToolCall(call) => {
                    // 工具调用前先把已攒的增量冲出去，保证界面顺序
                    if let Some(rest) = batcher.take() {
                        emit_event(&app, turn_id, json!({"type": "delta", "text": rest}));
                    }
                    if let Some(rest) = thinking_batcher.take() {
                        emit_event(&app, turn_id, json!({"type": "thinking", "text": rest}));
                    }
                    emit_event(
                        &app,
                        turn_id,
                        json!({
                            "type": "tool-start",
                            "callId": call.id,
                            "name": call.name,
                            "arguments": call.arguments,
                        }),
                    );
                }
                llm::LlmEvent::Usage { input_tokens, output_tokens } => {
                    emit_event(
                        &app,
                        turn_id,
                        json!({
                            "type": "usage", "inputTokens": input_tokens,
                            "outputTokens": output_tokens, "contextWindow": context_window,
                        }),
                    );
                }
            }
        })
        .await;

        if let Some(rest) = batcher.take() {
            emit_event(&app, turn_id, json!({"type": "delta", "text": rest}));
        }
        if let Some(rest) = thinking_batcher.take() {
            emit_event(&app, turn_id, json!({"type": "thinking", "text": rest}));
        }

        let turn = match outcome {
            Ok(t) => t,
            Err(e) => {
                emit_event(&app, turn_id, json!({"type": "error", "message": e}));
                return;
            }
        };

        if turn.tool_calls.is_empty() {
            emit_event(
                &app,
                turn_id,
                json!({
                    "type": "done", "text": turn.text, "round": round,
                    "usage": turn.usage.map(|(input, output)| json!({"input": input, "output": output})),
                }),
            );
            return;
        }

        // 执行工具（只读 + 提案），把结果回填给模型继续推理
        messages.push(llm::ChatMessage {
            role: "assistant".to_string(),
            content: turn.text.clone(),
            tool_calls: turn.tool_calls.clone(),
            tool_call_id: None,
            name: None,
        });

        for call in &turn.tool_calls {
            if cancel.load(Ordering::SeqCst) {
                break;
            }
            let args: Value = match serde_json::from_str(&call.arguments) {
                Ok(v) => v,
                Err(e) => json!({"__parseError": e.to_string()}),
            };
            if args.get("__parseError").is_some() {
                let msg = "参数不是合法 JSON，请重新给出".to_string();
                emit_event(
                    &app,
                    turn_id,
                    json!({"type": "tool-result", "callId": call.id, "name": call.name, "error": msg}),
                );
                messages.push(llm::ChatMessage::tool(&call.id, &call.name, json!({"error": msg})));
                continue;
            }
            emit_event(
                &app,
                turn_id,
                json!({"type": "tool-running", "callId": call.id, "name": call.name}),
            );
            let exec = tools::execute(&core, &call.name, &args).await;
            // 深度脱敏：既按字段名抹密钥，也逐个字符串抹形态（内嵌 JSON / 环境变量）
            let cleaned = safety::deep_redact(&exec.payload);
            let shrunk = tools::shrink(cleaned);

            // 交互卡片：request_form / ask 把净化后的 schema 发给前端渲染表单/问题，
            // 用户填完提交后再作为新消息回给模型继续处理（敏感字段值只留在前端本地）。
            if shrunk.get("error").is_none() {
                match call.name.as_str() {
                    "request_form" => {
                        if let Ok(spec) = tools::sanitize_form_schema(&args) {
                            emit_event(
                                &app,
                                turn_id,
                                json!({"type": "form", "callId": call.id, "form": spec}),
                            );
                        }
                    }
                    "ask" => {
                        if let Ok(spec) = tools::sanitize_ask_schema(&args) {
                            emit_event(
                                &app,
                                turn_id,
                                json!({"type": "question", "callId": call.id, "question": spec}),
                            );
                        }
                    }
                    _ => {}
                }
            }

            if !exec.proposals.is_empty() {
                for p in &exec.proposals {
                    emit_event(
                        &app,
                        turn_id,
                        json!({"type": "proposal", "callId": call.id, "proposal": p}),
                    );
                }
            }
            for action in &exec.actions {
                emit_event(
                    &app,
                    turn_id,
                    json!({"type": "action", "action": action}),
                );
            }
            emit_event(
                &app,
                turn_id,
                json!({
                    "type": "tool-result", "callId": call.id, "name": call.name,
                    "result": shrunk.clone(),
                    "isError": shrunk.get("error").is_some(),
                }),
            );
            messages.push(llm::ChatMessage::tool(&call.id, &call.name, shrunk));
        }
    }

    emit_event(
        &app,
        turn_id,
        json!({
            "type": "done",
            "text": "（已达到本轮最多工具调用次数，请换个说法或分步提问）",
            "stopped": cancel.load(Ordering::SeqCst),
        }),
    );
}

/// 任务结束时把 turn 从注册表摘掉（含 panic 回卷）
struct FinishGuard(String);

impl Drop for FinishGuard {
    fn drop(&mut self) {
        if let Ok(mut m) = RUNNING_TURNS.lock() {
            m.remove(&self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deltas_are_coalesced_not_per_token() {
        let mut b = DeltaBatcher::default();
        // 刚过 last_flush 判定时序：第一条会立即冲刷（保证响应感），随后应开始攒
        assert_eq!(b.push("a").as_deref(), Some("a"));
        let mut flushed = 0;
        for i in 0..50 {
            if b.push(&format!("x{i}")).is_some() {
                flushed += 1;
            }
        }
        // 50 次增量（每次 ~3 字符）在窗口内应远少于逐条发送
        assert!(flushed <= 2, "攒批失效：{flushed} 次冲刷 / 50 次增量");
        assert!(!b.take().unwrap_or_default().is_empty() || flushed > 0);
    }

    /// 首字立即出（响应感），后续攒批，收尾必须把残余冲干净（不丢字）
    #[test]
    fn pending_delta_is_always_flushed_at_end() {
        let mut b = DeltaBatcher::default();
        assert_eq!(b.push("第一段").as_deref(), Some("第一段"));
        assert_eq!(b.push("第二段"), None, "窗口内应攒住");
        let rest = b.take().unwrap();
        assert_eq!(rest, "第二段");
        assert!(b.take().is_none(), "取完应为空");
    }

    #[test]
    fn system_prompt_states_the_hard_rules() {
        let route = llm::RouteInfo {
            provider_name: "内网网关".into(),
            protocol: "openai".into(),
            model_id: "qwen-max".into(),
            context_window: 32000,
        };
        let prompt = system_prompt(&route, &["list_servers".to_string(), "search_usage_guides".to_string()]);
        assert!(prompt.contains("propose_config_change"));
        assert!(prompt.contains("[已隐藏]"), "要告诉模型凭据永远是隐藏的");
        assert!(prompt.contains("不得违反"));
        assert!(prompt.contains("qwen-max"));
        assert!(prompt.contains("2 个"));
        // 交互录入必须作为独立强提示存在，且明确禁止正文罗列字段让用户逐条回复
        assert!(prompt.contains("交互录入"), "提示词要引导模型优先用交互卡片");
        assert!(prompt.contains("request_form"), "提示词必须点名 request_form");
        assert!(prompt.contains("ask"), "提示词必须点名 ask");
        assert!(prompt.contains("禁止在正文里罗列字段"), "要明确禁止口头逐条追问");
        // 项目问题排查：必须先查指南、再翻源码，且只读本项目根
        assert!(prompt.contains("本项目的问题排查"), "提示词要有项目排查引导");
        assert!(prompt.contains("search_project_guides"), "提示词必须点名项目指南");
        assert!(prompt.contains("search_project_source"), "提示词必须点名源码检索");
        assert!(prompt.contains("read_project_source"), "提示词必须点名源码读取");
        assert!(prompt.contains("只读本项目根"), "要明确只读本项目根");
        // 不得出现「可以直接保存」这类暗示
        assert!(!prompt.contains("帮你保存"));
    }

    #[tokio::test]
    async fn finish_guard_removes_the_turn() {
        RUNNING_TURNS.lock().unwrap().insert(
            "t-guard".to_string(),
            (
                tokio::spawn(async {
                    futures::future::pending::<()>().await;
                }),
                Arc::new(AtomicBool::new(false)),
            ),
        );
        assert_eq!(active_turn_count(), 1);
        assert!(abort_turn("t-guard"));
        assert_eq!(active_turn_count(), 0);
        assert!(!abort_turn("t-not-exist"));
    }
}
