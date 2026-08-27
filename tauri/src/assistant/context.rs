//! 上下文窗口预算与历史裁剪
//!
//! 上下文窗口按模型配置（不是全局常量），裁剪策略：
//! 系统提示词永远保留 → 最近若干轮对话优先保留 → 工具结果按体量截断 →
//! 超预算时从**最旧**的非系统消息开始丢，绝不丢最后一条用户输入。

/// 粗略 token 估算：中日韩字符按 1 字 ≈ 1 token，其余按 4 字符 ≈ 1 token。
/// 不引分词器——估算只需保守，宁可提前裁掉一点。
pub fn estimate_tokens(text: &str) -> usize {
    let mut cjk = 0usize;
    let mut other = 0usize;
    for ch in text.chars() {
        if is_cjk(ch) {
            cjk += 1;
        } else {
            other += 1;
        }
    }
    cjk + other / 4 + 1
}

fn is_cjk(ch: char) -> bool {
    matches!(u32::from(ch),
        0x3040..=0x30FF | 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF | 0xFF00..=0xFF60)
}

/// 单条工具结果允许占用的最大字符数：构建日志动辄几万行，全塞进上下文会立刻顶满窗口
pub const MAX_TOOL_RESULT_CHARS: usize = 6_000;

pub fn clip_for_context(text: &str, max_chars: usize) -> String {
    let count = text.chars().count();
    if count <= max_chars {
        return text.to_string();
    }
    // 保留头尾：报错通常在结尾，前半段常有关键上下文
    let keep = max_chars.saturating_sub(40);
    let head = keep / 3;
    let tail = keep - head;
    let chars: Vec<char> = text.chars().collect();
    let mut out: String = chars[..head].iter().collect();
    out.push_str(&format!("\n…（省略中间 {} 字，完整内容见日志）…\n", count - keep));
    out.push_str(&chars[chars.len() - tail..].iter().collect::<String>());
    out
}

/// 按窗口预算裁剪消息序列。
/// `messages[0]` 约定为系统提示词；`reserve_for_reply` 给模型回复留出空间。
pub fn trim_to_budget(
    messages: &[super::llm::ChatMessage],
    context_window: u32,
    reserve_for_reply: u32,
) -> Vec<super::llm::ChatMessage> {
    use super::llm::ChatMessage;

    let budget = (context_window as usize).saturating_sub(reserve_for_reply as usize).max(512);
    let (system, rest) = match messages.first() {
        Some(m) if m.role == "system" => (Some(m.clone()), &messages[1..]),
        _ => (None, &messages[..]),
    };

    let system_cost = system.as_ref().map(|m| estimate_tokens(&m.content)).unwrap_or(0);
    let mut limit = budget.saturating_sub(system_cost);

    // 从最新往回收集，遇到装不下的就停（保持轮次连续，不会出现半截对话）
    let mut kept: Vec<ChatMessage> = Vec::new();
    for msg in rest.iter().rev() {
        let cost = estimate_tokens(&msg.content)
            + msg
                .tool_calls
                .iter()
                .map(|tc| estimate_tokens(&tc.arguments) + estimate_tokens(&tc.name))
                .sum::<usize>();
        if cost > limit {
            break;
        }
        limit -= cost;
        kept.push(msg.clone());
    }
    kept.reverse();

    let mut out = Vec::with_capacity(kept.len() + 1);
    if let Some(s) = system {
        out.push(s);
    }
    out.extend(kept);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use super::super::llm::ChatMessage;

    #[test]
    fn cjk_costs_more_per_char_than_ascii() {
        assert!(estimate_tokens("配置部署助手") > estimate_tokens("deploy config"));
        assert!(estimate_tokens(&"a".repeat(400)) < 200);
    }

    #[test]
    fn clip_keeps_head_and_tail_and_marks_omission() {
        let text = format!("{}{}{}", "H".repeat(5000), "m".repeat(1000), "T".repeat(500));
        let clipped = clip_for_context(&text, 1000);
        assert!(clipped.chars().count() < 1100);
        assert!(clipped.starts_with('H'));
        assert!(clipped.ends_with('T'), "报错通常在结尾，尾部必须留住");
        assert!(clipped.contains("省略中间"));
        assert_eq!(clip_for_context("short", 1000), "short");
    }

    #[test]
    fn keeps_system_and_latest_turns_when_over_budget() {
        let mut msgs = vec![ChatMessage::system("你是配置助手")];
        for i in 0..200 {
            msgs.push(ChatMessage::user(&format!(
                "第 {} 轮：{}",
                i,
                "很长的中文输入".repeat(20)
            )));
        }
        let trimmed = trim_to_budget(&msgs, 2_000, 500);
        assert_eq!(trimmed[0].role, "system", "系统提示词不能被裁掉");
        assert_eq!(
            trimmed.last().unwrap().content,
            msgs.last().unwrap().content,
            "最后一条用户输入必须保留"
        );
        assert!(trimmed.len() < msgs.len(), "应当确实发生裁剪");
        // 预算充足时原样保留
        assert_eq!(trim_to_budget(&msgs, 200_000, 500).len(), msgs.len());
    }

    #[test]
    fn tool_calls_count_toward_budget() {
        let mut with_call = ChatMessage::text("assistant", "");
        with_call.tool_calls = vec![super::super::llm::ToolCall {
            id: "c1".into(),
            name: "get_cicd_config".into(),
            arguments: json!({"blob": "x".repeat(4000)}).to_string(),
        }];
        let msgs = vec![
            ChatMessage::system("s"),
            with_call,
            ChatMessage::user("继续"),
        ];
        let trimmed = trim_to_budget(&msgs, 1_200, 400);
        assert!(
            !trimmed.iter().any(|m| !m.tool_calls.is_empty()),
            "塞不下的历史轮应整轮丢弃"
        );
        assert_eq!(trimmed.last().unwrap().content, "继续");
    }
}
