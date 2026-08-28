//! AI 助手 —— 模型提供商配置
//!
//! 存储在 `settings` 表（key = `ai_providers`，JSON 数组），与 `db_connections` 同一套模式：
//! apiKey 用 `crate::encryption`（AES-256-GCM）加密后落盘。
//!
//! 安全边界（AI 助手相关约定，勿回退）：
//! - 对前端/工具层一律走 `list_ai_providers()`，apiKey 只返回掩码，永不出明文；
//! - 只有 LLM 调用层通过 `resolve_ai_route()` 单独取明文 key，且该函数结果不得回传 UI/工具；
//! - 保存时 apiKey 传空或传掩码值表示「沿用已存密钥」，避免掩码被当成新 key 写回。
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const AI_PROVIDERS_KEY: &str = "ai_providers";
/// 显式清除已存密钥的哨兵值（留空 = 沿用旧值，所以清掉需要显式表态）
pub const CLEAR_KEY: &str = "__clear__";
pub const AI_ACTIVE_MODEL_KEY: &str = "ai_active_model";

/// 未填时的默认上下文窗口 / 最大输出（tokens）
pub const DEFAULT_CONTEXT_WINDOW: u32 = 8_192;
pub const DEFAULT_MAX_OUTPUT_TOKENS: u32 = 2_048;
pub const MIN_CONTEXT_WINDOW: u32 = 512;
pub const MAX_CONTEXT_WINDOW: u32 = 4_000_000;

/// 接口协议：两套都支持的模型都能接，用户按提供商实况选择
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum AiProtocol {
    #[serde(alias = "openai-compatible", alias = "openai_compatible")]
    #[default]
    OpenAi,
    #[serde(alias = "claude")]
    Anthropic,
}

impl AiProtocol {
    pub fn as_str(self) -> &'static str {
        match self {
            AiProtocol::OpenAi => "openai",
            AiProtocol::Anthropic => "anthropic",
        }
    }
}

/// 单个模型：模型 ID 完全自由填写（不同网关的 ID 命名各异），上下文窗口按模型配置
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct AiModel {
    /// 请求体里实际发送的模型 ID
    pub id: String,
    /// 界面展示名，留空则用 id
    #[serde(default)]
    pub label: String,
    /// 上下文窗口（tokens）：会话裁剪预算依据
    pub context_window: u32,
    /// 单次回复上限（tokens）
    pub max_output_tokens: u32,
    /// 是否支持识图（视觉输入）：开了才允许给助手发图片/截图
    #[serde(default)]
    pub vision: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct AiProvider {
    pub id: String,
    pub name: String,
    pub protocol: AiProtocol,
    pub base_url: String,
    /// 落盘为密文；入参为空/掩码表示沿用旧值
    pub api_key: String,
    pub models: Vec<AiModel>,
    pub enabled: bool,
}

impl Default for AiProvider {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            protocol: AiProtocol::OpenAi,
            base_url: String::new(),
            api_key: String::new(),
            models: Vec::new(),
            enabled: true,
        }
    }
}

/// LLM 调用层解析出的完整路由（含明文 key，禁止外传）
#[derive(Clone, Debug)]
pub struct AiRoute {
    pub provider_name: String,
    pub protocol: AiProtocol,
    pub base_url: String,
    pub api_key: String,
    pub model_id: String,
    pub context_window: u32,
    pub max_output_tokens: u32,
    /// 当前模型是否支持识图（决定能否带图请求）
    pub vision: bool,
}

/// 密钥掩码：只保留末 4 位用于界面确认「配的是哪个」
pub fn mask_secret(secret: &str) -> String {
    let chars: Vec<char> = secret.chars().collect();
    if chars.is_empty() {
        return String::new();
    }
    if chars.len() <= 8 {
        return "*".repeat(8);
    }
    let tail: String = chars[chars.len() - 4..].iter().collect();
    format!("****{}", tail)
}

/// 看起来像掩码（而非用户新填的明文 key）
fn looks_masked(secret: &str) -> bool {
    secret.starts_with("****") || secret.chars().all(|c| c == '*')
}

/// 归一化 baseUrl：必须 http(s)，去掉尾部斜杠；拒绝 URL 内嵌凭据（避免密钥进日志）
fn normalize_base_url(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err("接口地址不能为空".to_string());
    }
    let parsed = url::Url::parse(trimmed).map_err(|e| format!("接口地址非法: {}", e))?;
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err("接口地址仅支持 http/https".to_string());
    }
    if parsed.host_str().unwrap_or("").is_empty() {
        return Err("接口地址缺少主机名".to_string());
    }
    // 允许内网/本机端点（Ollama、LM Studio、公司内网网关），但 key 必须走 apiKey 字段
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("接口地址不能内嵌用户名/密码，请把密钥填在 apiKey".to_string());
    }
    Ok(parsed.to_string().trim_end_matches('/').to_string())
}

fn clamp_window(value: u32, fallback: u32) -> u32 {
    match value {
        0 => fallback,
        v => v.clamp(MIN_CONTEXT_WINDOW, MAX_CONTEXT_WINDOW),
    }
}

/// 校验并就地补默认值；返回可读错误供界面直接展示
fn normalize_provider(p: &mut AiProvider) -> Result<(), String> {
    p.name = p.name.trim().to_string();
    if p.name.is_empty() {
        return Err("提供商名称不能为空".to_string());
    }
    p.base_url = normalize_base_url(&p.base_url)?;
    if p.id.trim().is_empty() {
        p.id = uuid::Uuid::new_v4().to_string();
    }
    let mut seen: Vec<String> = Vec::new();
    for m in p.models.iter_mut() {
        m.id = m.id.trim().to_string();
        if m.id.is_empty() {
            return Err("模型 ID 不能为空".to_string());
        }
        if seen.contains(&m.id) {
            return Err(format!("模型 ID 重复: {}", m.id));
        }
        seen.push(m.id.clone());
        if m.label.trim().is_empty() {
            m.label = m.id.clone();
        } else {
            m.label = m.label.trim().to_string();
        }
        m.context_window = clamp_window(m.context_window, DEFAULT_CONTEXT_WINDOW);
        let max_out = match m.max_output_tokens {
            0 => DEFAULT_MAX_OUTPUT_TOKENS,
            v => v,
        };
        // 输出上限不得超过窗口；至少留 1 token
        m.max_output_tokens = max_out.min(m.context_window.saturating_sub(1)).max(1);
    }
    Ok(())
}

impl AiProvider {
    /// 转成可安全外发给前端/工具层的形态（apiKey 掩码化）
    pub fn to_public(&self) -> Value {
        json!({
            "id": self.id,
            "name": self.name,
            "protocol": self.protocol.as_str(),
            "baseUrl": self.base_url,
            "apiKeyMasked": mask_secret(&self.api_key),
            "hasKey": !self.api_key.is_empty(),
            "models": self.models.iter().map(|m| json!({
                "id": m.id,
                "label": m.label,
                "contextWindow": m.context_window,
                "maxOutputTokens": m.max_output_tokens,
                "vision": m.vision,
            })).collect::<Vec<_>>(),
            "enabled": self.enabled,
        })
    }
}

impl super::CoreService {
    /// 读取提供商列表（原始，含密文 apiKey）
    fn read_providers_raw(&self) -> Result<Vec<AiProvider>, String> {
        let raw: String = self.db_read(|conn| {
            conn.query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![AI_PROVIDERS_KEY],
                |row| row.get::<_, String>(0),
            )
            .unwrap_or_default()
        })?;
        if raw.trim().is_empty() {
            return Ok(Vec::new());
        }
        serde_json::from_str::<Vec<AiProvider>>(&raw).map_err(|e| format!("解析 ai_providers 失败: {}", e))
    }

    fn write_providers_raw(&self, providers: &[AiProvider]) -> Result<(), String> {
        let value = serde_json::to_string(providers).map_err(|e| e.to_string())?;
        self.db_write(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
                params![AI_PROVIDERS_KEY, value],
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
        })?
    }

    /// 列表（apiKey 已解密，仅供 LLM 调用层内部使用，禁止外发）
    pub fn ai_providers_decrypted(&self) -> Result<Vec<AiProvider>, String> {
        let mut providers = self.read_providers_raw()?;
        for p in providers.iter_mut() {
            if !p.api_key.is_empty() {
                p.api_key = crate::encryption::try_decrypt_password(&p.api_key);
            }
        }
        Ok(providers)
    }

    /// 对前端/工具层安全：只返回掩码
    pub async fn list_ai_providers(&self) -> Result<Value, String> {
        let providers = self.read_providers_raw()?;
        Ok(json!(providers
            .iter()
            .map(|p| {
                let mut pub_p = p.clone();
                pub_p.api_key = crate::encryption::try_decrypt_password(&p.api_key);
                pub_p.to_public()
            })
            .collect::<Vec<_>>()))
    }

    /// 新增或更新（按 id 匹配）。apiKey 传空/掩码 → 沿用旧密钥。
    pub async fn save_ai_provider(&self, input: Value) -> Result<Value, String> {
        let mut provider: AiProvider =
            serde_json::from_value(input).map_err(|e| format!("提供商配置格式错误: {}", e))?;
        normalize_provider(&mut provider)?;

        let mut providers = self.read_providers_raw()?;
        let existing = providers.iter().find(|p| p.id == provider.id).cloned();

        let incoming_key = provider.api_key.trim().to_string();
        provider.api_key = if incoming_key == CLEAR_KEY {
            String::new()
        } else if incoming_key.is_empty() || looks_masked(&incoming_key) {
            // 沿用旧密文（未配置过则为空）
            existing.as_ref().map(|e| e.api_key.clone()).unwrap_or_default()
        } else {
            crate::encryption::encrypt_password(&incoming_key)
                .map_err(|e| format!("apiKey 加密失败: {}", e))?
        };

        match providers.iter_mut().find(|p| p.id == provider.id) {
            Some(slot) => *slot = provider.clone(),
            None => providers.push(provider.clone()),
        }
        self.write_providers_raw(&providers)?;

        // 首个提供商自动设为当前使用模型，省去用户再点一次
        let has_active: bool = self
            .db_read(|conn| {
                conn.query_row(
                    "SELECT value FROM settings WHERE key = ?1",
                    params![AI_ACTIVE_MODEL_KEY],
                    |r| r.get::<_, String>(0),
                )
                .ok()
            })?
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false);
        if !has_active {
            if let Some(model) = providers
                .iter()
                .find(|p| p.id == provider.id)
                .and_then(|p| p.models.first())
            {
                self.set_active_ai_model(&provider.id, &model.id).await?;
            }
        }
        Ok(self
            .list_ai_providers()
            .await?
            .as_array()
            .and_then(|arr| arr.iter().find(|p| p["id"] == provider.id).cloned())
            .ok_or_else(|| "保存后未能读回提供商".to_string())?)
    }

    pub async fn delete_ai_provider(&self, id: &str) -> Result<Value, String> {
        let mut providers = self.read_providers_raw()?;
        let before = providers.len();
        providers.retain(|p| p.id != id);
        if providers.len() == before {
            return Err(format!("提供商不存在: {}", id));
        }
        self.write_providers_raw(&providers)?;
        Ok(json!({ "success": true, "removed": before - providers.len() }))
    }

    /// 当前使用的模型（{"providerId","modelId"}），未设置为 Null
    pub async fn get_active_ai_model(&self) -> Result<Value, String> {
        let raw: String = self.db_read(|conn| {
            conn.query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![AI_ACTIVE_MODEL_KEY],
                |r| r.get::<_, String>(0),
            )
            .unwrap_or_default()
        })?;
        if raw.trim().is_empty() {
            return Ok(Value::Null);
        }
        Ok(serde_json::from_str(&raw).unwrap_or(Value::Null))
    }

    pub async fn set_active_ai_model(&self, provider_id: &str, model_id: &str) -> Result<Value, String> {
        let providers = self.read_providers_raw()?;
        let provider = providers
            .iter()
            .find(|p| p.id == provider_id)
            .ok_or_else(|| "提供商不存在".to_string())?;
        if !provider.models.iter().any(|m| m.id == model_id) {
            return Err(format!("模型不存在: {}", model_id));
        }
        if !provider.enabled {
            return Err("该提供商已停用，请先启用".to_string());
        }
        let value = json!({ "providerId": provider_id, "modelId": model_id });
        self.db_write(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
                params![AI_ACTIVE_MODEL_KEY, value.to_string()],
            )
            .map_err(|e| e.to_string())
        })??;
        Ok(value)
    }

    /// LLM 调用层入口：把「当前选择」解析成一条完整可执行的路由（含明文 key）
    pub fn resolve_ai_route(&self) -> Result<AiRoute, String> {
        let providers = self.ai_providers_decrypted()?;
        if providers.is_empty() {
            return Err("尚未配置 AI 模型，请到 设置 → AI 模型 添加提供商".to_string());
        }
        let active: Option<Value> = self
            .db_read(|conn| {
                conn.query_row(
                    "SELECT value FROM settings WHERE key = ?1",
                    params![AI_ACTIVE_MODEL_KEY],
                    |r| r.get::<_, String>(0),
                )
                .ok()
            })?
            .and_then(|v| if v.trim().is_empty() { None } else { serde_json::from_str(&v).ok() });

        // 指定优先；否则取第一个启用提供商的第一个模型
        match active {
            Some(a) => self.ai_route_for(
                a["providerId"].as_str().unwrap_or_default(),
                a["modelId"].as_str().unwrap_or_default(),
            ),
            None => {
                let provider = providers
                    .iter()
                    .find(|p| p.enabled && !p.models.is_empty())
                    .ok_or_else(|| "没有启用中的提供商/模型".to_string())?;
                Ok(build_route(provider, &provider.models[0]))
            }
        }
    }

    /// 指定提供商 + 模型解析路由（供「测试连接」按行测试，不必先切换当前模型）
    pub fn ai_route_for(&self, provider_id: &str, model_id: &str) -> Result<AiRoute, String> {
        let providers = self.ai_providers_decrypted()?;
        let provider = providers
            .iter()
            .find(|p| p.id == provider_id)
            .ok_or_else(|| "提供商不存在".to_string())?;
        if !provider.enabled {
            return Err("该提供商已停用".to_string());
        }
        let model = provider
            .models
            .iter()
            .find(|m| m.id == model_id)
            .ok_or_else(|| "模型不存在".to_string())?;
        Ok(build_route(provider, model))
    }
}

fn build_route(provider: &AiProvider, model: &AiModel) -> AiRoute {
    let context_window = clamp_window(model.context_window, DEFAULT_CONTEXT_WINDOW);
    AiRoute {
        provider_name: provider.name.clone(),
        protocol: provider.protocol,
        base_url: provider.base_url.clone(),
        api_key: provider.api_key.clone(),
        model_id: model.id.clone(),
        context_window,
        max_output_tokens: model
            .max_output_tokens
            .max(1)
            .min(context_window.saturating_sub(1).max(1)),
        vision: model.vision,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::logic::CoreService;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static DB_SEQ: AtomicUsize = AtomicUsize::new(0);

    fn temp_core() -> CoreService {
        let seq = DB_SEQ.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "supertool_ai_provider_test_{}_{}",
            std::process::id(),
            seq
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.db");
        let _ = std::fs::remove_file(&path);
        CoreService::new(Database::new(&path).unwrap(), dir)
    }

    fn sample(key: &str) -> Value {
        json!({
            "name": "内网网关",
            "protocol": "openai",
            "baseUrl": "https://gateway.example.com/v1/",
            "apiKey": key,
            "models": [{"id": "qwen-max", "contextWindow": 32000, "maxOutputTokens": 4000}]
        })
    }

    fn stored_raw(core: &CoreService) -> String {
        core.db_read(|c| {
            c.query_row(
                "SELECT value FROM settings WHERE key='ai_providers'",
                [],
                |r| r.get::<_, String>(0),
            )
            .unwrap_or_default()
        })
        .unwrap()
    }

    /// 红线：apiKey 不得明文落盘，列表接口不得回传明文
    #[tokio::test]
    async fn api_key_is_encrypted_at_rest_and_masked_on_read() {
        let core = temp_core();
        let saved = core.save_ai_provider(sample("sk-super-secret-123456")).await.unwrap();
        assert_eq!(saved["apiKeyMasked"], "****3456");
        assert!(
            saved.get("apiKey").is_none(),
            "对外结构不应带 apiKey 字段"
        );

        let raw = stored_raw(&core);
        assert!(!raw.contains("sk-super-secret-123456"), "明文 key 不应落盘: {}", raw);

        // 调用层仍能取回明文
        let route = core.resolve_ai_route().unwrap();
        assert_eq!(route.api_key, "sk-super-secret-123456");
        assert_eq!(route.base_url, "https://gateway.example.com/v1");
        assert_eq!(route.protocol, AiProtocol::OpenAi);
        assert_eq!(route.context_window, 32000);
    }

    /// 编辑时不回传明文：空值/掩码都必须沿用旧 key，否则一编辑就丢密钥
    #[tokio::test]
    async fn saving_with_empty_or_masked_key_keeps_existing_secret() {
        let core = temp_core();
        let saved = core.save_ai_provider(sample("sk-keepme-aaaa")).await.unwrap();
        let id = saved["id"].as_str().unwrap().to_string();

        for keep in ["", "****aaaa"] {
            let mut again = sample(keep);
            again["id"] = json!(id);
            again["name"] = json!("改名后");
            core.save_ai_provider(again).await.unwrap();
            assert_eq!(
                core.resolve_ai_route().unwrap().api_key,
                "sk-keepme-aaaa",
                "apiKey={} 时不应覆盖掉已存密钥",
                keep
            );
        }
        assert_eq!(core.resolve_ai_route().unwrap().provider_name, "改名后");
    }

    /// 清空密钥要显式表态（留空表示沿用，避免一编辑就把 key 抹了）
    #[tokio::test]
    async fn clear_sentinel_removes_stored_key_but_empty_keeps_it() {
        let core = temp_core();
        let saved = core.save_ai_provider(sample("sk-to-clear")).await.unwrap();
        let id = saved["id"].as_str().unwrap().to_string();

        let mut keep = sample("");
        keep["id"] = json!(id);
        core.save_ai_provider(keep).await.unwrap();
        assert_eq!(core.resolve_ai_route().unwrap().api_key, "sk-to-clear");

        let mut clear = sample(CLEAR_KEY);
        clear["id"] = json!(id);
        let after = core.save_ai_provider(clear).await.unwrap();
        assert_eq!(after["hasKey"], false);
        assert_eq!(core.resolve_ai_route().unwrap().api_key, "");
    }

    #[tokio::test]
    async fn validates_input_and_fills_window_defaults() {
        let core = temp_core();
        // 非法协议地址
        let bad = json!({"name":"x","protocol":"openai","baseUrl":"ftp://a.com","apiKey":"k",
                         "models":[{"id":"m"}]});
        assert!(core.save_ai_provider(bad).await.is_err());
        // URL 内嵌凭据
        let bad2 = json!({"name":"x","protocol":"openai","baseUrl":"https://u:p@a.com","apiKey":"k",
                          "models":[{"id":"m"}]});
        assert!(core.save_ai_provider(bad2).await.is_err());
        // 模型 ID 为空
        let bad3 = json!({"name":"x","protocol":"openai","baseUrl":"https://a.com","apiKey":"k",
                          "models":[{"id":"  "}]});
        assert!(core.save_ai_provider(bad3).await.is_err());

        // 未填窗口 → 用默认值；输出上限超窗口 → 收敛到窗口内
        let mut ok = json!({"name":"本地","protocol":"anthropic","baseUrl":"http://127.0.0.1:11434/v1",
                            "apiKey":"", "models":[{"id":"claude-3-5-sonnet"}]});
        ok["models"][0]["maxOutputTokens"] = json!(999_999);
        let saved = core.save_ai_provider(ok).await.unwrap();
        let model = &saved["models"].as_array().unwrap()[0];
        assert_eq!(model["contextWindow"], DEFAULT_CONTEXT_WINDOW);
        assert_eq!(model["maxOutputTokens"], DEFAULT_CONTEXT_WINDOW - 1);
        assert_eq!(model["label"], "claude-3-5-sonnet");
        assert_eq!(saved["protocol"], "anthropic");
        assert_eq!(saved["hasKey"], false);
    }

    /// 首个提供商自动成为当前模型；删除后路由应报错而不是乱指向
    #[tokio::test]
    async fn first_provider_becomes_active_and_delete_clears_route() {
        let core = temp_core();
        assert!(core.resolve_ai_route().is_err());

        let saved = core.save_ai_provider(sample("sk-1")).await.unwrap();
        let id = saved["id"].as_str().unwrap().to_string();
        let active = core.get_active_ai_model().await.unwrap();
        assert_eq!(active["providerId"], saved["id"]);
        assert_eq!(active["modelId"], "qwen-max");

        core.delete_ai_provider(&id).await.unwrap();
        assert!(core.resolve_ai_route().is_err());
        assert_eq!(core.list_ai_providers().await.unwrap().as_array().unwrap().len(), 0);
    }

    #[test]
    fn protocol_accepts_both_spellings_and_mask_keeps_tail() {
        assert_eq!(
            serde_json::from_str::<AiProtocol>("\"anthropic\"").unwrap(),
            AiProtocol::Anthropic
        );
        assert_eq!(
            serde_json::from_str::<AiProtocol>("\"openai-compatible\"").unwrap(),
            AiProtocol::OpenAi
        );
        assert_eq!(mask_secret(""), "");
        assert_eq!(mask_secret("short"), "********");
        assert_eq!(mask_secret("1234567890abcd"), "****abcd");
    }
}
