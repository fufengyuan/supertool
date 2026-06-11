use anyhow::{anyhow, bail, Context, Result};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

use api::{InputContentBlock, InputMessage, MessageRequest, OutputContentBlock};

// ─── config ─────────────────────────────────────────────

struct LlmConfig {
    #[allow(dead_code)]
    model: String,
    api_key: String,
    base_url: String,
}

fn hermes_config_path() -> PathBuf {
    dirs::home_dir().unwrap_or_default().join(".hermes/config.yaml")
}

fn claw_config_path() -> PathBuf {
    dirs::home_dir().unwrap_or_default().join(".claw/settings.json")
}

fn load_hermes_config() -> Result<LlmConfig> {
    let content = std::fs::read_to_string(&hermes_config_path())
        .with_context(|| format!("Cannot read {}", hermes_config_path().display()))?;
    let v: serde_yaml::Value = serde_yaml::from_str(&content)
        .context("Failed to parse Hermes config (YAML)")?;
    let m = &v["model"];
    Ok(LlmConfig {
        model: m["default"].as_str().unwrap_or("claude-sonnet-4-6").to_string(),
        api_key: m["api_key"].as_str().context("hermes: no model.api_key in config")?.to_string(),
        base_url: m["base_url"].as_str().context("hermes: no model.base_url in config")?.to_string(),
    })
}

fn load_claw_config() -> Result<LlmConfig> {
    let content = std::fs::read_to_string(&claw_config_path())
        .with_context(|| format!("Cannot read {}", claw_config_path().display()))?;
    let v: serde_json::Value = serde_json::from_str(&content)
        .context("Failed to parse Claw config (JSON)")?;
    let active = v["activeModel"].as_str().unwrap_or("Claude Sonnet");
    let models = v["models"].as_array().context("claw: no 'models' array")?;
    let entry = models.iter()
        .find(|m| m["name"].as_str() == Some(active))
        .or_else(|| models.first())
        .context("claw: no models configured")?;
    let key = entry["apiKey"].as_str().unwrap_or("");
    if key.is_empty() || key == "***" {
        bail!("claw: no valid API key in ~/.claw/settings.json.\n       Configure credentials in the SuperTool GUI or edit the file directly.");
    }
    Ok(LlmConfig {
        model: entry["model"].as_str().unwrap_or("claude-sonnet-4-6").to_string(),
        api_key: key.to_string(),
        base_url: entry["baseUrl"].as_str().context("claw: no baseUrl")?.to_string(),
    })
}

// ─── LLM call（使用 workspace api 库）───────────────────

async fn call_llm(cfg: &LlmConfig, messages: Vec<InputMessage>, system: Option<&str>) -> Result<String> {
    unsafe { std::env::set_var("OPENAI_API_KEY", &cfg.api_key) };
    unsafe { std::env::set_var("OPENAI_BASE_URL", &cfg.base_url) };

    let client = api::ProviderClient::from_model(&format!("openai/{}", cfg.model))
        .map_err(|e| anyhow!("Cannot create LLM client: {e}"))?;

    let request = MessageRequest {
        model: cfg.model.clone(),
        max_tokens: 8192,
        messages,
        system: system.map(|s| s.to_string()),
        tools: None,
        tool_choice: None,
        stream: false,
        temperature: None,
        top_p: None,
        frequency_penalty: None,
        presence_penalty: None,
        stop: None,
        reasoning_effort: None,
        extra_body: BTreeMap::new(),
    };

    let resp = client.send_message(&request).await
        .map_err(|e| anyhow!("LLM API error: {e}"))?;

    let text: String = resp.content.iter()
        .filter_map(|block| match block {
            OutputContentBlock::Text { text } => Some(text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");

    Ok(text)
}

fn info_line(label: &str, detail: &str) {
    println!("\x1b[36m═══ {label} {detail}\x1b[0m");
}

fn user_msg(text: &str) -> InputMessage {
    InputMessage::user_text(text)
}

fn assistant_msg(text: &str) -> InputMessage {
    InputMessage {
        role: "assistant".to_string(),
        content: vec![InputContentBlock::Text { text: text.to_string() }],
    }
}

// ─── hermes chat（使用 workspace api 库 + Hermes 配置）───

pub async fn cmd_hermes_chat(message: String) -> Result<()> {
    info_line("Hermes", "⚡");
    let cfg = load_hermes_config()?;
    let resp = call_llm(&cfg, vec![user_msg(&message)], None).await?;
    println!("{resp}");
    Ok(())
}

// ─── claw chat（使用 supertool_claw::LlmClient） ────────

pub async fn cmd_claw_chat(message: String) -> Result<()> {
    info_line("Claw", "💬");
    let cfg = load_claw_config()?;
    let resp = call_llm(
        &cfg,
        vec![user_msg(&message)],
        Some("You are Claw, a focused AI coding assistant. Respond concisely."),
    )
    .await?;
    println!("{resp}");
    Ok(())
}

// ─── claw goal ──────────────────────────────────────────

pub async fn cmd_claw_goal(text: String, max_turns: u32) -> Result<()> {
    info_line("Claw Goal", &format!("🎯 \"{text}\""));
    let cfg = load_claw_config()?;

    let goal_system = "You are Claw. Work toward the goal below. When done, end with [GOAL_COMPLETE].";
    let judge_system = "You are a goal judge. Answer YES or NO only.";
    let mut messages: Vec<InputMessage> = Vec::new();

    for turn in 1..=max_turns {
        let msg = if turn == 1 {
            info_line("Round", &format!("{turn}/{max_turns} begin"));
            format!("Goal: {text}\n\nWork toward this goal. End with [GOAL_COMPLETE] when done.")
        } else {
            info_line("Round", &format!("{turn}/{max_turns} continue"));
            format!("Continue: {text}")
        };

        messages.push(user_msg(&msg));
        let resp = call_llm(&cfg, messages.clone(), Some(goal_system)).await?;
        messages.push(assistant_msg(&resp));
        println!("\n{resp}");

        if resp.contains("[GOAL_COMPLETE]") {
            info_line("Goal", "✅ 达成！");
            println!("{}", resp.replace("[GOAL_COMPLETE]", ""));
            return Ok(());
        }

        let judge = call_llm(
            &cfg,
            vec![user_msg(&format!(
                "Goal: {text}\n\nLatest work:\n{resp}\n\nIs this goal fully achieved? Answer YES or NO."
            ))],
            Some(judge_system),
        )
        .await?;
        if judge.to_uppercase().contains("YES") && !judge.to_uppercase().contains("NO") {
            info_line("Goal", "✅ 达成！(judge 确认)");
            return Ok(());
        }

        if turn < max_turns { sleep(Duration::from_secs(1)).await; }
    }

    info_line("Goal", "⚠️ 已达最大轮次");
    bail!("Goal not completed within {max_turns} rounds");
}

pub async fn cmd_claw_loop(message: String, count: Option<u32>, duration: Option<String>) -> Result<()> {
    let max_iters = parse_limit(count, duration)?;
    let display = if max_iters == u32::MAX { "∞".into() } else { max_iters.to_string() };

    info_line("Claw Loop", &format!("🔄 \"{message}\" (max: {display})"));
    let cfg = load_claw_config()?;

    let mut messages: Vec<InputMessage> = Vec::new();
    for turn in 1..=max_iters {
        let msg = if turn == 1 { message.clone() } else { format!("Continue: {message}") };
        messages.push(user_msg(&msg));
        let resp = call_llm(&cfg, messages.clone(), Some("You are Claw. Respond concisely.")).await?;
        messages.push(assistant_msg(&resp));
        info_line("Loop", &format!("#{turn} / {display}"));
        println!("{resp}");
        if turn < max_iters { sleep(Duration::from_millis(800)).await; }
    }
    info_line("Loop", "✓ 循环结束");
    Ok(())
}

fn parse_limit(count: Option<u32>, duration: Option<String>) -> Result<u32> {
    if let Some(c) = count { return Ok(c); }
    if let Some(d) = duration {
        if let Some(s) = d.strip_suffix('s') {
            return Ok((s.parse::<u32>().map_err(|_| anyhow!("bad duration: {d}"))? / 5).max(1));
        }
        if let Some(m) = d.strip_suffix('m') {
            return Ok((m.parse::<u32>().map_err(|_| anyhow!("bad duration: {d}"))? * 60 / 5).max(1));
        }
        bail!("invalid duration format: {d} (use e.g. 30s, 5m)");
    }
    Ok(u32::MAX)
}
