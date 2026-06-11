use anyhow::{anyhow, bail, Context, Result};
use std::time::Duration;
use tokio::process::Command;
use tokio::time::sleep;

// ─── 调用官方 hermes CLI ─────────────────────────────

/// 调用 `hermes chat -q <message> --quiet`（可选 session 恢复）。
/// 返回 (输出文本, session_id)。
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
        .context("Failed to spawn `hermes` CLI. Is it installed and in PATH?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("hermes CLI failed: {}", stderr.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // session_id 在 stderr 中
    let new_sid = stderr
        .lines()
        .find_map(|l| l.strip_prefix("session_id: "))
        .unwrap_or_default()
        .to_string();

    // 清理 stdout：去掉 Warning: 行和空行
    let body: String = stdout
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim().starts_with("Warning:"))
        .collect::<Vec<_>>()
        .join("\n");

    Ok((body, new_sid))
}

fn info_line(label: &str, detail: &str) {
    println!("\x1b[36m═══ {label} {detail}\x1b[0m");
}

// ─── hermes chat ────────────────────────────────────────

pub async fn cmd_hermes_chat(message: String) -> Result<()> {
    info_line("Hermes", "⚡");
    let (resp, _sid) = hermes_chat(&message, None).await?;
    println!("{resp}");
    Ok(())
}

// ─── claw chat ──────────────────────────────────────────

pub async fn cmd_claw_chat(message: String) -> Result<()> {
    info_line("Claw", "💬");
    let enhanced = format!(
        "[System: You are Claw, a focused AI coding assistant.]\n\n{}",
        message
    );
    let (resp, _sid) = hermes_chat(&enhanced, None).await?;
    println!("{resp}");
    Ok(())
}

// ─── claw goal ──────────────────────────────────────────

pub async fn cmd_claw_goal(text: String, max_turns: u32) -> Result<()> {
    info_line("Claw Goal", &format!("🎯 \"{text}\""));

    let mut session_id: Option<String> = None;
    let mut prev_resp = String::new();

    for turn in 1..=max_turns {
        let msg = if turn == 1 {
            info_line("Round", &format!("{turn}/{max_turns} begin"));
            format!(
                "Goal: {text}\n\nWork toward this goal. When done, end with [GOAL_COMPLETE]."
            )
        } else {
            info_line("Round", &format!("{turn}/{max_turns} continue"));
            format!(
                "Continue working toward the goal: {text}\n\nPrevious work:\n{prev_resp}"
            )
        };

        let (resp, new_sid) = hermes_chat(&msg, session_id.as_deref()).await?;
        if session_id.is_none() && !new_sid.is_empty() {
            session_id = Some(new_sid);
        }

        println!("\n{resp}");

        if resp.contains("[GOAL_COMPLETE]") {
            info_line("Goal", "✅ 达成！");
            println!("{}", resp.replace("[GOAL_COMPLETE]", ""));
            return Ok(());
        }

        // Judge
        if let Some(ref sid) = session_id {
            let judge_msg = format!(
                "Has this goal been fully achieved?\nGoal: {text}\nAnswer YES or NO only."
            );
            let (judge, _) = hermes_chat(&judge_msg, Some(sid)).await?;
            if judge.to_uppercase().contains("YES") && !judge.to_uppercase().contains("NO") {
                info_line("Goal", "✅ 达成！(judge 确认)");
                return Ok(());
            }
        }

        prev_resp = resp;

        if turn < max_turns {
            sleep(Duration::from_secs(1)).await;
        }
    }

    info_line("Goal", "⚠️ 已达最大轮次");
    Ok(())
}

// ─── claw loop ──────────────────────────────────────────

pub async fn cmd_claw_loop(
    message: String,
    count: Option<u32>,
    duration: Option<String>,
) -> Result<()> {
    let max_iters = parse_limit(count, duration)?;
    let display = if max_iters == u32::MAX {
        "∞".into()
    } else {
        max_iters.to_string()
    };

    info_line("Claw Loop", &format!("🔄 \"{message}\" (max: {display})"));

    for turn in 1..=max_iters {
        let msg = if turn == 1 {
            message.clone()
        } else {
            format!("Continue: {message}")
        };

        info_line("Loop", &format!("#{turn} / {display}"));

        match hermes_chat(&msg, None).await {
            Ok((resp, _sid)) => println!("{resp}"),
            Err(e) => eprintln!("Loop error at turn {turn}: {e}"),
        }

        if turn < max_iters {
            sleep(Duration::from_millis(800)).await;
        }
    }

    info_line("Loop", "✓ 循环结束");
    Ok(())
}

fn parse_limit(count: Option<u32>, duration: Option<String>) -> Result<u32> {
    if let Some(c) = count {
        return Ok(c);
    }
    if let Some(d) = duration {
        if let Some(s) = d.strip_suffix('s') {
            let secs: u32 = s.parse().map_err(|_| anyhow::anyhow!("bad duration: {d}"))?;
            return Ok((secs / 5).max(1));
        }
        if let Some(m) = d.strip_suffix('m') {
            let mins: u32 = m.parse().map_err(|_| anyhow::anyhow!("bad duration: {d}"))?;
            return Ok((mins * 60 / 5).max(1));
        }
        bail!("invalid duration: {d} (use e.g. 30s, 5m)");
    }
    Ok(u32::MAX)
}
