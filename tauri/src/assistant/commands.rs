//! AI 助手 —— 模型提供商配置命令
//!
//! 前端只通过这些命令读写提供商，返回结构里**没有明文 apiKey**（只有掩码）。
use serde_json::{Value, json};
use supertool_core::logic::CoreService;
use tauri::State;

/// 列表（apiKey 掩码）
#[tauri::command(rename_all = "camelCase")]
pub async fn list_ai_providers(core: State<'_, CoreService>) -> Result<Value, String> {
    core.list_ai_providers().await
}

/// 新增/更新提供商：apiKey 留空或回传掩码表示沿用已存密钥
#[tauri::command(rename_all = "camelCase")]
pub async fn save_ai_provider(
    core: State<'_, CoreService>,
    provider: Value,
) -> Result<Value, String> {
    core.save_ai_provider(provider).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_ai_provider(
    core: State<'_, CoreService>,
    id: String,
) -> Result<Value, String> {
    core.delete_ai_provider(&id).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_active_ai_model(core: State<'_, CoreService>) -> Result<Value, String> {
    core.get_active_ai_model().await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn set_active_ai_model(
    core: State<'_, CoreService>,
    provider_id: String,
    model_id: String,
) -> Result<Value, String> {
    core.set_active_ai_model(&provider_id, &model_id).await
}

/// 连通性测试：发一条最小请求，验证协议/地址/密钥/模型 ID 是否真的能用。
/// 失败信息以 {ok:false,error} 返回（不抛错），方便界面就地提示怎么改。
#[tauri::command(rename_all = "camelCase")]
pub async fn test_ai_model(
    core: State<'_, CoreService>,
    provider_id: String,
    model_id: String,
) -> Result<Value, String> {
    let route = core.ai_route_for(&provider_id, &model_id)?;
    let request = super::llm::ChatRequest {
        model: route.model_id.clone(),
        messages: vec![super::llm::ChatMessage::user("只回复两个字：可用")],
        tools: Vec::new(),
        // 探测用，不需要长输出
        max_output_tokens: 32.min(route.context_window.saturating_sub(1).max(1)),
        temperature: Some(0.0),
    };
    let started = std::time::Instant::now();
    let mut collector = String::new();
    let outcome = super::llm::stream_completion(&route, &request, &mut |event| {
        if let super::llm::LlmEvent::TextDelta(delta) = event {
            collector.push_str(&delta);
        }
    })
    .await;

    match outcome {
        Ok(turn) => Ok(json!({
            "ok": true,
            "latencyMs": started.elapsed().as_millis() as u64,
            "reply": super::llm::clip(&turn.text, 200),
            "protocol": route.protocol.as_str(),
            "providerName": route.provider_name,
            "modelId": route.model_id,
            "contextWindow": route.context_window,
            "maxOutputTokens": route.max_output_tokens,
        })),
        Err(e) => Ok(json!({
            "ok": false,
            "error": e,
            "contextWindow": route.context_window,
        })),
    }
}
