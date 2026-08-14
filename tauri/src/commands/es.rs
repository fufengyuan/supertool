// Elasticsearch 管理：HTTP REST 直连（可选 Basic Auth）
// 连接复用 database.rs 的 CONNECTION_POOL（DbConnection::Elasticsearch variant）
use serde_json::{json, Value};
use std::time::Duration;

use super::database::{DbConnection, CONNECTION_POOL};

/// Elasticsearch REST 客户端
#[derive(Clone)]
pub struct EsClient {
    client: reqwest::Client,
    base_url: String, // e.g. http://host:9200
    auth: Option<(String, String)>,
}

fn url_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

impl EsClient {
    async fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<Value>,
    ) -> Result<Value, String> {
        let url = format!("{}{}", self.base_url, path);
        let m = reqwest::Method::from_bytes(method.as_bytes())
            .map_err(|e| format!("Invalid HTTP method: {}", e))?;
        let mut req = self.client.request(m, &url);
        if let Some((u, p)) = &self.auth {
            req = req.basic_auth(u.clone(), Some(p.clone()));
        }
        if let Some(b) = body {
            req = req.json(&b);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| format!("ES 请求失败: {}", e))?;
        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| format!("ES 读取响应失败: {}", e))?;
        if status.is_success() {
            if text.trim().is_empty() {
                return Ok(json!({ "ok": true }));
            }
            return Ok(serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text })));
        }
        // 提取 ES error 信息（尽量简洁）
        let reason = serde_json::from_str::<Value>(&text)
            .ok()
            .and_then(|v| v.get("error").cloned())
            .unwrap_or_else(|| json!(text));
        let snippet = reason.to_string();
        let snippet = if snippet.len() > 500 {
            format!("{}...", &snippet[..500])
        } else {
            snippet
        };
        Err(format!(
            "ES {} {} 失败 ({}): {}",
            method, path, status, snippet
        ))
    }
}

/// 建立 ES 连接并校验连通性（5s 连接超时）
pub async fn connect_es(config: &super::database::DbConnectionConfig) -> Result<EsClient, String> {
    let base_url = format!("http://{}:{}", config.host, config.port);
    let auth = if config.username.is_empty() {
        None
    } else {
        let pw = config
            .password
            .as_deref()
            .map(|p| supertool_core::encryption::try_decrypt_password(p))
            .unwrap_or_default();
        Some((config.username.clone(), pw))
    };
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .connect_timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| format!("ES client 创建失败: {}", e))?;
    let es = EsClient {
        client,
        base_url: base_url.clone(),
        auth,
    };
    let v = es
        .request("GET", "/", None)
        .await
        .map_err(|e| format!("ES 连接失败 ({}): {}", base_url, e))?;
    if v.get("version").is_none() {
        return Err(format!("ES 端点 {} 响应异常: {}", base_url, v));
    }
    Ok(es)
}

async fn get_es(id: &str) -> Result<EsClient, String> {
    let pool = CONNECTION_POOL.lock().await;
    match pool.get(id) {
        Some(DbConnection::Elasticsearch(c)) => Ok(c.clone()),
        Some(_) => Err("当前连接不是 Elasticsearch".to_string()),
        None => Err("Connection not found. Call db:connect first.".to_string()),
    }
}

// ============ 集群/节点 ============

#[tauri::command(rename_all = "camelCase")]
pub async fn es_cluster_health(id: String) -> Result<Value, String> {
    log::info!("[Tauri CMD] es_cluster_health() called");
    let es = get_es(&id).await?;
    es.request("GET", "/_cluster/health", None).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn es_nodes(id: String) -> Result<Value, String> {
    log::info!("[Tauri CMD] es_nodes() called");
    let es = get_es(&id).await?;
    es.request(
        "GET",
        "/_cat/nodes?format=json&h=name,ip,heap.percent,ram.percent,cpu,load_1m,disk.used_percent,master,version",
        None,
    )
    .await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn es_cluster_stats(id: String) -> Result<Value, String> {
    log::info!("[Tauri CMD] es_cluster_stats() called");
    let es = get_es(&id).await?;
    es.request("GET", "/_cluster/stats", None).await
}

// ============ 索引 ============

#[tauri::command(rename_all = "camelCase")]
pub async fn es_list_indices(id: String) -> Result<Value, String> {
    log::info!("[Tauri CMD] es_list_indices() called");
    let es = get_es(&id).await?;
    es.request("GET", "/_cat/indices?format=json&bytes=b", None)
        .await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn es_index_info(id: String, index: String) -> Result<Value, String> {
    log::info!("[Tauri CMD] es_index_info() called, index={}", index);
    let es = get_es(&id).await?;
    let enc = url_encode(&index);
    let info = es.request("GET", &format!("/{}?flat_settings=false", enc), None).await?;
    let stats = es
        .request("GET", &format!("/{}/_stats", enc), None)
        .await?;
    Ok(json!({ "info": info, "stats": stats }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn es_index_mapping(id: String, index: String) -> Result<Value, String> {
    log::info!("[Tauri CMD] es_index_mapping() called, index={}", index);
    let es = get_es(&id).await?;
    let enc = url_encode(&index);
    es.request("GET", &format!("/{}/_mapping", enc), None).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn es_create_index(
    id: String,
    index: String,
    body: Option<Value>,
) -> Result<Value, String> {
    log::info!("[Tauri CMD] es_create_index() called, index={}", index);
    let es = get_es(&id).await?;
    let enc = url_encode(&index);
    es.request("PUT", &format!("/{}", enc), body).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn es_update_index_settings(
    id: String,
    index: String,
    settings: Value,
) -> Result<Value, String> {
    log::info!("[Tauri CMD] es_update_index_settings() called, index={}", index);
    let es = get_es(&id).await?;
    let enc = url_encode(&index);
    es.request("PUT", &format!("/{}/_settings", enc), Some(settings))
        .await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn es_delete_index(id: String, indices: String) -> Result<Value, String> {
    log::info!("[Tauri CMD] es_delete_index() called, indices={}", indices);
    let es = get_es(&id).await?;
    // 支持逗号分隔多索引
    let encoded: Vec<String> = indices
        .split(',')
        .map(|s| url_encode(s.trim()))
        .filter(|s| !s.is_empty())
        .collect();
    if encoded.is_empty() {
        return Err("索引名不能为空".to_string());
    }
    es.request("DELETE", &format!("/{}", encoded.join(",")), None)
        .await
}

// ============ 别名 / reindex ============

#[tauri::command(rename_all = "camelCase")]
pub async fn es_aliases(id: String) -> Result<Value, String> {
    log::info!("[Tauri CMD] es_aliases() called");
    let es = get_es(&id).await?;
    es.request("GET", "/_cat/aliases?format=json", None).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn es_update_aliases(id: String, actions: Value) -> Result<Value, String> {
    log::info!("[Tauri CMD] es_update_aliases() called");
    let es = get_es(&id).await?;
    es.request("POST", "/_aliases", Some(json!({ "actions": actions })))
        .await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn es_reindex(id: String, source: String, dest: String) -> Result<Value, String> {
    log::info!("[Tauri CMD] es_reindex() called, {} -> {}", source, dest);
    let es = get_es(&id).await?;
    es.request(
        "POST",
        "/_reindex",
        Some(json!({
            "source": { "index": source },
            "dest": { "index": dest }
        })),
    )
    .await
}

// ============ 搜索 / 文档 ============

#[tauri::command(rename_all = "camelCase")]
pub async fn es_search(
    id: String,
    index: String,
    body: Value,
    from: Option<u64>,
    size: Option<u64>,
) -> Result<Value, String> {
    log::info!("[Tauri CMD] es_search() called, index={}", index);
    let es = get_es(&id).await?;
    let enc = url_encode(&index);
    let mut search = body;
    if let Some(f) = from {
        search["from"] = json!(f);
    }
    if let Some(s) = size {
        search["size"] = json!(s);
    }
    es.request("POST", &format!("/{}/_search", enc), Some(search))
        .await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn es_get_document(
    id: String,
    index: String,
    doc_id: String,
) -> Result<Value, String> {
    log::info!("[Tauri CMD] es_get_document() called, index={}", index);
    let es = get_es(&id).await?;
    let enc = url_encode(&index);
    es.request("GET", &format!("/{}/_doc/{}", enc, url_encode(&doc_id)), None)
        .await
}

/// 新增/覆写文档：doc_id 为空则 POST 自动生成 id
#[tauri::command(rename_all = "camelCase")]
pub async fn es_index_document(
    id: String,
    index: String,
    doc_id: Option<String>,
    body: Value,
) -> Result<Value, String> {
    log::info!("[Tauri CMD] es_index_document() called, index={}", index);
    let es = get_es(&id).await?;
    let enc = url_encode(&index);
    match doc_id {
        Some(did) if !did.trim().is_empty() => {
            es.request("PUT", &format!("/{}/_doc/{}", enc, url_encode(&did)), Some(body))
                .await
        }
        _ => es.request("POST", &format!("/{}/_doc", enc), Some(body)).await,
    }
}

/// 局部更新文档（_update 合并 doc）
#[tauri::command(rename_all = "camelCase")]
pub async fn es_update_document(
    id: String,
    index: String,
    doc_id: String,
    body: Value,
) -> Result<Value, String> {
    log::info!("[Tauri CMD] es_update_document() called, index={}", index);
    let es = get_es(&id).await?;
    let enc = url_encode(&index);
    es.request(
        "POST",
        &format!("/{}/_update/{}", enc, url_encode(&doc_id)),
        Some(json!({ "doc": body })),
    )
    .await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn es_delete_document(
    id: String,
    index: String,
    doc_id: String,
) -> Result<Value, String> {
    log::info!("[Tauri CMD] es_delete_document() called, index={}", index);
    let es = get_es(&id).await?;
    let enc = url_encode(&index);
    es.request(
        "DELETE",
        &format!("/{}/_doc/{}", enc, url_encode(&doc_id)),
        None,
    )
    .await
}
