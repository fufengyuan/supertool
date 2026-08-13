use reqwest;
use tauri::{AppHandle, Url, WebviewUrl, WebviewWindowBuilder};
use tokio::sync::oneshot;

#[tauri::command]
pub async fn fetch_page_content(url: String) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let text = response
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    Ok(text)
}

/// JS 渲染页面（SPA）抓取：开隐藏 WebView 加载 URL，等脚本执行完渲染后提取正文 HTML。
/// 普通 reqwest 抓取拿不到 Vue/React 打包页面的正文，必须让 JS 在浏览器内核中运行。
/// 注意：隐藏 WebView 中远程页面无 Tauri IPC 权限（capabilities 仅 local），无本地数据面。
#[tauri::command]
pub async fn fetch_page_content_js(app: AppHandle, url: String) -> Result<String, String> {
    let parsed = Url::parse(&url).map_err(|e| format!("无效的 URL: {}", e))?;
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err("仅支持 http/https 协议".into());
    }

    // 唯一窗口 label（原子计数器，避免时间戳碰撞与 close 失败后同 label 重建冲突）
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let label = format!(
        "spa_fetch_{}",
        COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    );

    let webview = WebviewWindowBuilder::new(&app, &label, WebviewUrl::External(parsed))
        .visible(false)
        .build()
        .map_err(|e| format!("创建抓取窗口失败: {}", e))?;

    let result = fetch_rendered_html(&webview).await;

    // 无论成败都销毁隐藏窗口；close 失败打日志（否则窗口与 label 残留无法复用）
    if let Err(e) = webview.close() {
        log::error!("[fetch_page_content_js] 关闭抓取窗口失败: {e}");
    }

    result
}

/// 等页面 JS 渲染完成后提取正文容器 HTML
async fn fetch_rendered_html(webview: &tauri::WebviewWindow) -> Result<String, String> {
    // 轮询判定渲染完成（最多 15s）：
    // 判定条件=正文容器存在且文本>200，或页面整体文本>1000（避免 404/验证页提前判定）
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    let mut rendered = false;
    while std::time::Instant::now() < deadline {
        let js = r#"(function(){
            var root = document.querySelector('main, article, [role="main"], .markdown-body, #content, .article-content, .doc-content, .markdown') || document.body;
            return JSON.stringify({ has: !!document.querySelector('article, main, [role="main"]'), len: root.innerText.trim().length });
        })()"#;
        match eval_js_string(webview, js).await {
            Ok(s) => {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                    let has = v.get("has").and_then(|x| x.as_bool()).unwrap_or(false);
                    let len = v.get("len").and_then(|x| x.as_i64()).unwrap_or(0);
                    if len > 200 && (has || len > 1000) {
                        rendered = true;
                        break;
                    }
                }
            }
            Err(_) => { /* 页面尚未就绪，继续等待 */ }
        }
        tokio::time::sleep(std::time::Duration::from_millis(700)).await;
    }
    if !rendered {
        return Err("等待页面渲染超时（15 秒），页面可能加载缓慢、需要登录或验证".into());
    }

    // 提取正文容器（优先正文选择器，回退 body），并剥离导航/脚本/交互元素。
    // 注：剥离 header/footer 可能误伤正文整体包在其内的罕见页面，属已知取舍
    let js = r#"(function(){
        var root = document.querySelector('main, article, [role="main"], .markdown-body, #content, .article-content, .doc-content, .markdown') || document.body;
        var clone = root.cloneNode(true);
        clone.querySelectorAll('script,style,nav,header,footer,aside,iframe,form,button,input,select,textarea,svg,.ad,.ads,.advertisement').forEach(function(el){el.remove()});
        return clone.innerHTML;
    })()"#;
    eval_js_string(webview, js).await
}

/// 在 webview 中执行 JS 并取回结果（eval_with_callback 的结果是 JSON 序列化字符串）
async fn eval_js_string(webview: &tauri::WebviewWindow, js: &str) -> Result<String, String> {
    let (tx, rx) = oneshot::channel::<String>();
    // 回调类型是 Fn（可能被调用多次），oneshot Sender 是 move 语义 → Mutex 包住，只 send 一次
    let tx = std::sync::Arc::new(std::sync::Mutex::new(Some(tx)));
    webview
        .eval_with_callback(js, move |result| {
            if let Some(sender) = tx.lock().unwrap().take() {
                let _ = sender.send(result);
            }
        })
        .map_err(|e| format!("执行脚本失败: {}", e))?;

    let json = tokio::time::timeout(std::time::Duration::from_secs(2), rx)
        .await
        .map_err(|_| "脚本执行超时".to_string())?
        .map_err(|_| "脚本执行失败（无回调）".to_string())?;

    // JS 返回字符串时回调里是带引号的 JSON；返回数字时是裸数字——统一反序列化
    Ok(serde_json::from_str::<String>(&json).unwrap_or_else(|_| json))
}
