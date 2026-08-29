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
use crate::assistant::llm::clip;

/// 单轮对话最多允许模型连续调用几次工具（防止无限打转）
pub const MAX_TOOL_ROUNDS: usize = 8;
const DELTA_FLUSH_CHARS: usize = 120;
const DELTA_FLUSH_MS: u128 = 80;
/// 一轮输出少于该字符数且无工具调用 → 判定网关抽风（间歇性短回复），静默重试
const MIN_ANSWER_CHARS: usize = 15;
/// 网关抽风/请求失败时的自动重试上限（指数退避）
const MAX_STREAM_RETRIES: usize = 9;
/// 指数退避基础间隔与单次上限（毫秒）。
/// 实测：网关对「1 分钟内 9 次密集请求」会触发限流降级（返回 1 chunk 短文本），
/// 退避必须足够宽，前几次间隔拉大，避免重试反而加剧降级。
const RETRY_BASE_MS: u64 = 2_000;
const RETRY_MAX_MS: u64 = 30_000;

/// 第 n 次重试（1 起）前的等待：2s * 2^(n-1)，封顶 30s
fn retry_delay_ms(n: usize) -> u64 {
    (RETRY_BASE_MS << (n.saturating_sub(1).min(12))).min(RETRY_MAX_MS)
}

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
        【能力边界】\n\
        你能做的只有三件事：\n\
        · 读配置与状态（服务器/Git 仓库/部署配置/数据库连接/日志预设/部署日志/模型配置），基于真实数据回答；\n\
        · 查知识：search_usage_guides 内置教学、search_project_guides 本项目文档、\
          search_project_source / read_project_source 只读本项目源码；\n\
        · 调用 propose_config_change 产出变更提案，用户确认后才由界面写入。\n\
        你没有写库、执行命令、写 SQL、访问网络的能力。文件内容只有两个来源：部署日志（白名单）\
         与本项目源码（限定项目根）；**其他任何文件内容一律读不到**。\n\
        填 localPath/构建目录/产物目录之前，用 find_local_path / inspect_local_path / detect_local_project \
         确认真实路径（只返回路径/类型/大小/构建标志，读不到内容），不要凭猜。\n\
        以下规则任何情况下都不得违反。\n\n\
        【交互录入 —— 优先于一切口头追问】\n\
        用户要录入结构化信息时，只允许用下面两个工具收集，禁止在正文里罗列字段/选项让用户打字回复：\n\
        · 需要收集 2 个及以上字段（新增服务器、新建部署配置、补数据库连接等）→ 调用 request_form 一次性弹出表单。\n\
        · 只需要用户做一个选择或回答一句短话 → 调用 ask：single/multiple 必须给 options 候选让用户勾选，text 让用户自由输入。\n\
        卡片弹出后，正文只写一句引导（如「请填写上方表单」），不要重复字段、不要重复选项、不要替用户回答。\n\
        敏感字段（密码/密钥）type 用 password，name 必须是标准凭据名（password/sshKeyPath/apiKey/token/secret/privateKey）。\n\n\
        【凭据 —— 把「值」和「字段名」分开对待】\n\
        · 值：凭据值永远不写进对话、也不进提案 fields——fields 里放凭据值会被后端直接拒绝；\
          工具返回值里这类字段是 [已隐藏]。你永远看不到用户填的凭据值。\n\
        · 字段名：propose_config_change 的 needUserInput **必须列出**该目标需要的凭据字段名\
          （新建服务器→password/sshKeyPath、数据库连接→password、AI 提供商→apiKey；\
          日志预设与 Git 仓库没有凭据字段，不要虚构），\
          **即使你判断用户已在表单里填过也要列**——确认卡片靠它渲染凭据槽位，并自动带入用户在表单里填的值；\
          漏列会导致槽位不出现、已填密码带不进去，最终写入空密码。\n\
        · 需要凭据时，引导用户到表单/确认卡片的对应位置自己填，不要索要、猜测或转述。\n\n\
        【其他规则】\n\
        1) 不给猜测性结论：涉及具体某条配置先用读类工具看真实值（list_cicd_configs / get_cicd_config /\
           validate_cicd_config / analyze_deploy_error），基于返回值回答。\n\
        2) 解释字段含义或使用步骤前，先用 search_usage_guides 查内置知识库；查不到就明说不确定，\
           不要编造工具里没有的规则。引用知识条目时给出条目标题，方便用户回看。\n\
        3) 用户问「怎么操作」时，可以在给出步骤的同时调用 open_config_page 把他们带过去。\n\
        4) 提案里的 fields 必须是可直接使用的完整值（不要占位符、不要 JSON 字符串化），\
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
    images: Vec<llm::ImageBlock>,
) {
    let cancel = Arc::new(AtomicBool::new(false));
    let handle = tokio::spawn({
        let app = app.clone();
        let cancel = cancel.clone();
        let turn_id = turn_id.clone();
        async move {
            run_inner(app, core, &turn_id, user_message, history, images, cancel).await;
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
    images: Vec<llm::ImageBlock>,
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
    if images.is_empty() {
        messages.push(llm::ChatMessage::user(user_message));
    } else {
        messages.push(llm::ChatMessage::user_with_images(user_message, images));
    }

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
        // 网关间歇性短回复防护：输出过短（<15 字）且无工具调用时静默重试，直到拿到有效回复。
        // 重试成功后的完整文本由 round-text 事件下发，前端按轮补全，覆盖掉之前流式的短开场。
        let mut retried = 0;
        let turn = loop {
            // 首次（retried==0）流式预览发事件；网关抽风重试时静默收集，避免前端文本重复
            let emit_stream = retried == 0;
            let mut batcher = DeltaBatcher::default();
            let mut thinking_batcher = DeltaBatcher::default();
            let mut thinking_open = false;
            let outcome = llm::stream_completion(&route, &request, &mut |event| {
                if !emit_stream {
                    return;
                }
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

            if emit_stream {
                if let Some(rest) = batcher.take() {
                    emit_event(&app, turn_id, json!({"type": "delta", "text": rest}));
                }
                if let Some(rest) = thinking_batcher.take() {
                    emit_event(&app, turn_id, json!({"type": "thinking", "text": rest}));
                }
            }

            let t = match outcome {
                Err(e) if retried < MAX_STREAM_RETRIES => {
                    retried += 1;
                    let wait = retry_delay_ms(retried);
                    log::warn!(
                        "[assistant] 第 {round} 轮请求失败（{e}），{wait}ms 后自动重试 {retried}/{MAX_STREAM_RETRIES}"
                    );
                    emit_event(
                        &app,
                        turn_id,
                        json!({"type": "notice", "message": format!("请求失败，{}ms 后自动重试（{retried}/{MAX_STREAM_RETRIES}）…", wait)}),
                    );
                    if cancel.load(Ordering::SeqCst) {
                        emit_event(&app, turn_id, json!({"type": "done", "text": "（已停止）", "round": round}));
                        return;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(wait)).await;
                    continue; // 回到外层 loop 重新请求
                }
                Err(e) => {
                    log::warn!("[assistant] 第 {round} 轮失败: {e}");
                    emit_event(&app, turn_id, json!({"type": "error", "message": e}));
                    return;
                }
                Ok(t) if t.text.chars().count() < MIN_ANSWER_CHARS
                    && t.tool_calls.is_empty()
                    && retried < MAX_STREAM_RETRIES =>
                {
                    retried += 1;
                    let wait = retry_delay_ms(retried);
                    log::warn!(
                        "[assistant] 第 {round} 轮输出过短（{}字）且无工具调用，疑似网关抽风，{wait}ms 后静默重试 {retried}/{MAX_STREAM_RETRIES}",
                        t.text.chars().count()
                    );
                    emit_event(
                        &app,
                        turn_id,
                        json!({"type": "notice", "message": format!("本轮回答异常中断，{}ms 后自动重试（{retried}/{MAX_STREAM_RETRIES}）…", wait)}),
                    );
                    if cancel.load(Ordering::SeqCst) {
                        emit_event(&app, turn_id, json!({"type": "done", "text": "（已停止）", "round": round}));
                        return;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(wait)).await;
                    continue; // 回到外层 loop 重新请求
                }
                Ok(t) => t,
            };
            break t;
        };
        log::info!(
            "[assistant] 第 {round} 轮完成: text_len={} tool_calls={}",
            turn.text.chars().count(),
            turn.tool_calls.len()
        );
        for call in &turn.tool_calls {
            log::info!("[assistant] 工具调用: {} args={}", call.name, clip(&call.arguments, 200));
        }
        // 本轮完整文本快照：delta 只是流式预览，macOS 事件积压时可能只发到开头（如「我先」），
        // 前端按 round 用这份权威文本补全本轮缺口，避免「说半句就停」。
        emit_event(&app, turn_id, json!({"type": "round-text", "round": round, "text": turn.text}));

        if turn.tool_calls.is_empty() {
            log::info!("[assistant] 完成（无工具调用），输出 {text_len} 字", text_len = turn.text.chars().count());
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
            images: Vec::new(),
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
            log::info!(
                "[assistant] 执行工具 {}: proposals={} actions={}",
                call.name,
                exec.proposals.len(),
                exec.actions.len()
            );
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
        assert!(prompt.contains("needUserInput **必须列出**"), "要强制凭据字段进 needUserInput");
        assert!(prompt.contains("写入空密码"), "要说明漏列的后果");
        // 能力边界要消除矛盾：明确「文件内容只有部署日志+项目源码两个来源」
        assert!(prompt.contains("能力边界"), "提示词要有能力边界段");
        assert!(prompt.contains("其他任何文件内容一律读不到"), "要明确一般文件内容读不到");
        // 凭据要区分「值」与「字段名」，消除「提案里出现字段会被拒绝」的歧义
        assert!(prompt.contains("把「值」和「字段名」分开对待"), "凭据要按值/字段名分开讲");
        assert!(prompt.contains("fields 里放凭据值会被后端直接拒绝"), "值不能进 fields");
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
