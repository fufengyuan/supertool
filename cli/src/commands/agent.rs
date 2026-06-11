use anyhow::{anyhow, bail, Context, Result};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

use api::{InputContentBlock, InputMessage, MessageRequest, OutputContentBlock};

// ─── config ─────────────────────────────────────────────

struct LlmConfig {
    model: String,
    api_key: String,
    base_url: String,
}

fn hermes_config_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_default();
    home.join(".hermes/config.yaml")
}

fn claw_config_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_default();
    home.join(".claw/settings.json")
}

fn read_file(path: &PathBuf) -> Result<String> {
    std::fs::read_to_string(path)
        .with_context(|| format!("Cannot read {}", path.display()))
}

fn load_hermes_config() -> Result<LlmConfig> {
    let content = read_file(&hermes_config_path())?;
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
    let path = claw_config_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
            let active = v["activeModel"].as_str().unwrap_or("Claude Sonnet");
            if let Some(models) = v["models"].as_array() {
                let entry = models.iter()
                    .find(|m| m["name"].as_str() == Some(active))
                    .or_else(|| models.first());
                if let Some(entry) = entry {
                    let key = entry["apiKey"].as_str().unwrap_or("");
                    if !key.is_empty() && key != "***" {
                        return Ok(LlmConfig {
                            model: entry["model"].as_str().unwrap_or("claude-sonnet-4-6").to_string(),
                            api_key: key.to_string(),
                            base_url: entry["baseUrl"].as_str().unwrap_or("").to_string(),
                        });
                    }
                }
            }
        }
    }
    // Fallback: use Hermes config
    load_hermes_config()
}

// ─── LLM call ───────────────────────────────────────────

async fn call_llm(
    cfg: &LlmConfig,
    messages: Vec<InputMessage>,
    system: Option<&str>,
) -> Result<String> {
    // Set env vars so ProviderClient::from_model can discover them
    // Safety: set_var is unsafe in edition 2024; this is a CLI tool with no threading concerns.
    unsafe { std::env::set_var("OPENAI_API_KEY", &cfg.api_key) };
    unsafe { std::env::set_var("OPENAI_BASE_URL", &cfg.base_url) };

    // Force OpenAI-compatible client via "openai/" prefix
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

fn user_msg(text: &str) -> InputMessage {
    InputMessage {
        role: "user".to_string(),
        content: vec![InputContentBlock::Text { text: text.to_string() }],
    }
}

fn assistant_msg(text: &str) -> InputMessage {
    InputMessage {
        role: "assistant".to_string(),
        content: vec![InputContentBlock::Text { text: text.to_string() }],
    }
}

fn info_line(label: &str, detail: &str) {
    println!("\x1b[36m═══ {label} {detail}\x1b[0m");
}

// ─── public commands ────────────────────────────────────

pub async fn cmd_hermes_chat(message: String) -> Result<()> {
    info_line("Hermes", "⚡");
    let cfg = load_hermes_config()?;
    let resp = call_llm(&cfg, vec![user_msg(&message)], None).await?;
    println!("{resp}");
    Ok(())
}

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

pub async fn cmd_claw_goal(text: String, max_turns: u32) -> Result<()> {
    info_line("Claw Goal", &format!("🎯 \"{text}\""));
    let cfg = load_claw_config()?;

    let goal_system =
        "You are Claw. Work toward the goal below. When you believe it is fully achieved, end your response with [GOAL_COMPLETE].";
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

        // 1) Self-declared complete via marker
        if resp.contains("[GOAL_COMPLETE]") {
            info_line("Goal", "✅ 达成！");
            println!("{}", resp.replace("[GOAL_COMPLETE]", ""));
            return Ok(());
        }

        // 2) Judge: independent call (not added to conversation history)
        let judge_prompt = format!(
            "Goal: {text}\n\nLatest work:\n{resp}\n\nIs this goal fully achieved? Answer YES or NO."
        );
        let judge = call_llm(&cfg, vec![user_msg(&judge_prompt)], Some(judge_system)).await?;
        if judge.to_uppercase().contains("YES") && !judge.to_uppercase().contains("NO") {
            info_line("Goal", "✅ 达成！(judge 确认)");
            return Ok(());
        }

        if turn < max_turns {
            sleep(Duration::from_secs(1)).await;
        }
    }

    info_line("Goal", "⚠️ 已达最大轮次");
    bail!("Goal not completed within {max_turns} rounds");
}

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
    let cfg = load_claw_config()?;

    let mut messages: Vec<InputMessage> = Vec::new();

    for turn in 1..=max_iters {
        let msg = if turn == 1 {
            message.clone()
        } else {
            format!("Continue: {message}")
        };

        messages.push(user_msg(&msg));
        let resp = call_llm(&cfg, messages.clone(), Some("You are Claw. Respond concisely.")).await?;
        messages.push(assistant_msg(&resp));

        info_line("Loop", &format!("#{turn} / {display}"));
        println!("{resp}");

        if turn < max_iters {
            sleep(Duration::from_millis(800)).await;
        }
    }

    info_line("Loop", "✓ 循环结束");
    Ok(())
}

// ─── helpers ────────────────────────────────────────────

fn parse_limit(count: Option<u32>, duration: Option<String>) -> Result<u32> {
    if let Some(c) = count {
        return Ok(c);
    }
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
