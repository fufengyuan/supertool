use anyhow::{bail, Context, Result};
use std::time::Duration;
use tokio::process::Command;
use tokio::time::sleep;

// ─── helpers ──────────────────────────────────────────────

/// 调用 `hermes chat -q <message>`（可选的 session 恢复）。
/// 返回 (stdout 文本, session_id)。
async fn hermes_chat(message: &str, session_id: Option<&str>) -> Result<(String, String)> {
    let mut cmd = Command::new("hermes");
    cmd.arg("chat");

    if let Some(sid) = session_id {
        cmd.arg("-r").arg(sid);
    }

    cmd.arg("-q").arg(message);
    cmd.arg("--quiet");

    let output = cmd
        .output()
        .await
        .context("Failed to spawn hermes CLI. Is `hermes` installed and in PATH?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let err = stderr.trim();
        if err.is_empty() {
            bail!("hermes chat failed (exit code {})", output.status);
        } else {
            bail!("hermes chat failed: {err}");
        }
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // session_id 在 stderr 中：格式 "session_id: <id>"
    let new_session_id = stderr
        .lines()
        .find_map(|l| l.strip_prefix("session_id: "))
        .unwrap_or_default()
        .to_string();

    // 清理 stdout：去掉 "Warning:" 行和空行
    let body: String = stdout
        .lines()
        .filter(|l| {
            let trimmed = l.trim();
            !trimmed.is_empty() && !trimmed.starts_with("Warning:")
        })
        .collect::<Vec<_>>()
        .join("\n");

    Ok((body, new_session_id))
}

/// 打印一条带颜色的分隔信息行。
fn info_line(label: &str, detail: &str) {
    // 使用 ANSI 颜色，简单可靠
    println!("\x1b[36m═══ {label} {detail}\x1b[0m");
}

// ─── hermes ───────────────────────────────────────────────

/// `stool hermes <message>` — 直接通过 hermes CLI 对话（含工具执行）
pub async fn cmd_hermes_chat(message: String) -> Result<()> {
    info_line("Hermes", "⚡");
    let (response, _sid) = hermes_chat(&message, None).await?;
    println!("{response}");
    Ok(())
}

// ─── claw ─────────────────────────────────────────────────

/// `stool claw chat <message>` — 通过 hermes CLI 对话（Claw 模式提示）
pub async fn cmd_claw_chat(message: String) -> Result<()> {
    info_line("Claw", "💬");
    // Claw 模式下加一条系统提示引导
    let enhanced = format!(
        "You are Claw, a focused AI coding assistant. Respond concisely.\n\nUser message: {message}"
    );
    let (response, _sid) = hermes_chat(&enhanced, None).await?;
    println!("{response}");
    Ok(())
}

/// `stool claw goal <text>` — Goal 模式：持续工作直到目标达成
pub async fn cmd_claw_goal(text: String, max_turns: u32) -> Result<()> {
    info_line("Claw Goal", format!("🎯 \"{text}\"").as_str());

    let mut session_id: Option<String> = None;
    let mut prev_response = String::new();

    for turn in 1..=max_turns {
        let message = if turn == 1 {
            info_line("Round", format!("{turn}/{max_turns} 开始").as_str());
            format!(
                r#"Your goal: {text}

Work toward this goal. After each action, I will check whether the goal is complete.
If you believe the goal is fully achieved, end your response with [GOAL_COMPLETE]."#
            )
        } else {
            info_line("Round", format!("{turn}/{max_turns} 继续").as_str());
            format!(
                r#"Continue working toward the goal: {text}

Previous work: {prev_response}

If you believe the goal is now fully achieved, end your response with [GOAL_COMPLETE]."#
            )
        };

        let (response, new_sid) = hermes_chat(&message, session_id.as_deref()).await?;

        if session_id.is_none() && !new_sid.is_empty() {
            session_id = Some(new_sid);
        }

        println!("\n{response}");

        // 检查 LLM 是否自行声明完成
        if response.contains("[GOAL_COMPLETE]") {
            info_line("Goal", "✅ 目标达成！");
            // 去掉标记再显示
            println!("{}", response.replace("[GOAL_COMPLETE]", ""));
            return Ok(());
        }

        // Judge：用简洁一问检查目标是否完成
        let judge_msg = format!(
            r#"Based ONLY on our conversation so far, has the following goal been fully achieved?

GOAL: {text}

Answer with exactly one line:
- YES if the goal is fully achieved
- NO if there is still work to do

Do not explain. Just say YES or NO."#
        );

        let (judge, _) = hermes_chat(&judge_msg, session_id.as_deref()).await?;
        let judge_upper = judge.to_uppercase();

        if judge_upper.contains("YES") && !judge_upper.contains("NO") {
            info_line("Goal", "✅ 目标达成！");
            println!("{response}");
            return Ok(());
        }

        if turn < max_turns {
            let delay = if turn == 1 { 0 } else { 1 };
            if delay > 0 {
                sleep(Duration::from_secs(delay)).await;
            }
        }

        prev_response = response;
    }

    info_line("Goal", "⚠️ 已达最大轮次，目标未完成");
    bail!("Goal not completed within {max_turns} rounds");
}

/// `stool claw loop <message>` — Loop 模式：自动重发循环
pub async fn cmd_claw_loop(
    message: String,
    count: Option<u32>,
    duration: Option<String>,
) -> Result<()> {
    // 解析限制
    let max_iters: u32 = if let Some(c) = count {
        c
    } else if let Some(d) = &duration {
        // 解析时长，如 "5m"、"30s"
        if let Some(secs_str) = d.strip_suffix('s') {
            if let Ok(secs) = secs_str.parse::<u32>() {
                // 每轮约 5-10 秒，估算迭代数
                (secs / 5).max(1)
            } else {
                bail!("Invalid duration format: {d} (use e.g. 30s, 5m)");
            }
        } else if let Some(mins_str) = d.strip_suffix('m') {
            if let Ok(mins) = mins_str.parse::<u32>() {
                (mins * 60 / 5).max(1)
            } else {
                bail!("Invalid duration format: {d} (use e.g. 30s, 5m)");
            }
        } else {
            bail!("Invalid duration format: {d} (use e.g. 30s, 5m)");
        }
    } else {
        u32::MAX // 无限制
    };

    let display_limit = if max_iters == u32::MAX {
        "∞".to_string()
    } else {
        max_iters.to_string()
    };

    info_line(
        "Claw Loop",
        format!("🔄 \"{message}\" (max: {display_limit})").as_str(),
    );

    let mut session_id: Option<String> = None;
    let mut prev_response = String::new();

    for turn in 1..=max_iters {
        let msg = if turn == 1 {
            message.clone()
        } else {
            format!(
                "Continue working on this task: {message}\n\nPrevious iteration result:\n{prev_response}"
            )
        };

        let (response, new_sid) = hermes_chat(&msg, session_id.as_deref()).await?;

        if session_id.is_none() && !new_sid.is_empty() {
            session_id = Some(new_sid);
        }

        info_line("Loop", format!("#{turn} / {display_limit}").as_str());
        println!("{response}");

        prev_response = response;

        if turn < max_iters {
            // 800ms 延迟（仿 oh-my-pi）
            sleep(Duration::from_millis(800)).await;
        }
    }

    info_line("Loop", "✓ 循环结束");
    Ok(())
}
