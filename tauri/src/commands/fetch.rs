use reqwest;
use tauri::{AppHandle, Url, WebviewUrl, WebviewWindowBuilder};
use tokio::sync::oneshot;

/// 校验抓取 URL：仅允许公网 http/https（reqwest 主路径与 WebView 次路径共用，
/// 防止任一入口被用于探测内网/回环/云元数据服务）
fn validate_fetch_url(url: &str) -> Result<(), String> {
    let parsed = Url::parse(url).map_err(|e| format!("无效的 URL: {}", e))?;
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err("仅支持 http/https 协议".into());
    }
    if let Some(host) = parsed.host() {
        if is_blocked_host(&host) {
            return Err(format!("不支持抓取内网/回环地址（{}）", host));
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn fetch_page_content(url: String) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        // 禁用自动重定向：手动逐跳跟随并重新校验（防 302 跳转绕过 SSRF 拦截到内网/元数据端点）
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let mut current = url;
    for _ in 0..5 {
        validate_fetch_url(&current)?;
        let response = client
            .get(&current)
            .send()
            .await
            .map_err(|e| format!("请求失败: {}", e))?;

        // 302/301 重定向：仅跳转状态码才解析 Location 进入下一跳（每跳重新校验目标地址）
        if response.status().is_redirection() {
            if let Some(loc) = response.headers().get(reqwest::header::LOCATION) {
                let loc_str = loc.to_str().map_err(|e| format!("无效的重定向地址: {}", e))?;
                if !loc_str.trim().is_empty() {
                    current = response
                        .url()
                        .join(loc_str)
                        .map_err(|e| format!("解析重定向地址失败: {}", e))?
                        .to_string();
                    continue;
                }
            }
        }

        let text = response.text().await.map_err(|e| format!("读取响应失败: {}", e))?;
        return Ok(text);
    }
    Err("重定向次数过多（超过 5 次）".into())
}

/// IPv4 私网/回环/链路本地/CGNAT 段判定（IPv4 直连与 IPv4-mapped IPv6 共用）
fn is_blocked_v4(o: [u8; 4]) -> bool {
    if o[0] == 127 || o[0] == 0 {
        return true; // 回环 + 未指定
    }
    // 私网：10/8、172.16/12、192.168/16；链路本地：169.254/16；
    // CGNAT 共享地址段 100.64/10（阿里云元数据端点 100.100.100.200 在此）
    o[0] == 10
        || (o[0] == 172 && (16..=31).contains(&o[1]))
        || (o[0] == 192 && o[1] == 168)
        || (o[0] == 169 && o[1] == 254)
        || (o[0] == 100 && (64..=127).contains(&o[1]))
}

/// 拒绝回环/私网/链路本地/CGNAT 地址（防 SSRF 类：恶意 URL 触发本地服务或云元数据端点）。
/// 接收 `Url::host()` 的 `Host<&str>` 枚举（注意：host_str() 对 IPv6 返回带方括号的
/// 字符串 "[::1]"，直接 parse<IpAddr> 会失败导致绕过——必须用 Host 枚举匹配）。
/// 注：仅拦截字面 IP/localhost；域名 DNS 重绑定到内网无法静态识别，属已知局限。
fn is_blocked_host(host: &url::Host<&str>) -> bool {
    match host {
        url::Host::Domain(domain) => {
            let h = domain.to_lowercase();
            h == "localhost" || h.ends_with(".localhost")
        }
        url::Host::Ipv4(v4) => is_blocked_v4(v4.octets()),
        url::Host::Ipv6(v6) => {
            // IPv4-mapped IPv6（::ffff:10.0.0.1 等）按 V4 判定，防绕过
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_blocked_v4(v4.octets());
            }
            // IPv4-compatible IPv6（::127.0.0.1 等，前 6 段全 0）同样按 V4 判定
            let seg = v6.segments();
            if seg[0] == 0 && seg[1] == 0 && seg[2] == 0 && seg[3] == 0 && seg[4] == 0 && seg[5] == 0 {
                let v4 = [
                    (seg[6] >> 8) as u8,
                    (seg[6] & 0xff) as u8,
                    (seg[7] >> 8) as u8,
                    (seg[7] & 0xff) as u8,
                ];
                return is_blocked_v4(v4);
            }
            if v6.is_loopback() || v6.is_unspecified() {
                return true;
            }
            // ULA fc00::/7；链路本地 fe80::/10；site-local fec0::/10（已废弃，顺带拦截）
            (0xfc00..=0xfdff).contains(&seg[0])
                || (0xfe80..=0xfebf).contains(&seg[0])
                || (0xfec0..=0xfeff).contains(&seg[0])
        }
    }
}

/// JS 渲染页面（SPA）抓取：开隐藏 WebView 加载 URL，等脚本执行完渲染后提取正文 HTML。
/// 普通 reqwest 抓取拿不到 Vue/React 打包页面的正文，必须让 JS 在浏览器内核中运行。
/// 注意：隐藏 WebView 中远程页面无 Tauri IPC 权限（capabilities 仅 local），无本地数据面。
#[tauri::command]
pub async fn fetch_page_content_js(app: AppHandle, url: String) -> Result<String, String> {
    validate_fetch_url(&url)?;
    let parsed = Url::parse(&url).map_err(|e| format!("无效的 URL: {}", e))?;

    // 唯一窗口 label（原子计数器，避免时间戳碰撞与 close 失败后同 label 重建冲突）
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let label = format!(
        "spa_fetch_{}",
        COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    );

    // 导航锁源：只允许与初始 URL 同 host:port 的顶层导航，禁止 file: 等本地 scheme
    // （防页面自我导航到任意地址/同主机其他端口，配合无 IPC 权限的远程页面；
    //   用 port_or_known_default 比较，避免 https 默认 443 被误判为不同端口）
    let initial_host = parsed.host_str().unwrap_or("").to_string();
    let initial_port = parsed.port_or_known_default();

    let webview = WebviewWindowBuilder::new(&app, &label, WebviewUrl::External(parsed))
        .visible(false)
        .on_navigation(move |nav_url| {
            if nav_url.scheme() == "file" {
                return false;
            }
            match nav_url.host_str() {
                Some(h) => h == initial_host && nav_url.port_or_known_default() == initial_port,
                None => false,
            }
        })
        .build()
        .map_err(|e| format!("创建抓取窗口失败: {}", e))?;

    let result = fetch_rendered_html(&webview).await;

    // 无论成败都销毁隐藏窗口；close 失败打日志（否则窗口与 label 残留无法复用）
    if let Err(e) = webview.close() {
        log::error!("[fetch_page_content_js] 关闭抓取窗口失败: {e}");
    }

    result
}

/// 正文容器候选选择器（SAMPLE_JS / EXTRACT_JS 内嵌同一列表）：
/// SPA 文档站结构差异大，固定顺序取第一个可能命中侧边栏/目录等窄容器，
/// 故按「文本最多」选取。

/// 采样 JS：先展开折叠的代码块（CodeMirror 懒渲染），再返回候选容器与 CodeMirror 文本的最大长度
const SAMPLE_JS: &str = r#"(function(){
    // 支付宝文档代码块默认可能折叠（ne-codeblock-collapsed-button），点击后 CodeMirror 才渲染代码
    var btns = document.querySelectorAll('.ne-codeblock-collapsed-button');
    for (var b = 0; b < btns.length; b++) { try { btns[b].click(); } catch (e) {} }
    var sels = ['main','article','[role="main"]','.markdown-body','.markdown','#content','#article-content','.article-content','.doc-content','.doc-content-wrapper','.markdown-content','[class*="detail-content"]','[class*="doc-detail"]'];
    var best = 0;
    for (var i = 0; i < sels.length; i++) {
        var el = document.querySelector(sels[i]);
        if (el) { var l = el.innerText ? el.innerText.trim().length : 0; if (l > best) best = l; }
    }
    // 代码块（CodeMirror）文本并入渲染判定（代码块可能在容器外/刚展开）
    var cmBest = 0;
    var cms = document.querySelectorAll('.CodeMirror');
    for (var k = 0; k < cms.length; k++) {
        var t = cms[k].innerText ? cms[k].innerText.trim().length : 0;
        if (t > cmBest) cmBest = t;
    }
    return Math.max(best, cmBest);
})()"#;

/// 提取 JS：取文本最多的候选容器（回退 body），剥离导航/脚本/交互元素，
/// 展开折叠代码块、把 CodeMirror 结构转成标准 pre/code（CodeMirror 每行是一个 pre.CodeMirror-line，
/// turndown 无法识别成完整代码块）、保留文本节点换行（white-space:pre-line 页面的换行只在文本 \n 里），
/// 并附加容器外的代码块（正文容器可能不含 pre，代码块在独立容器）
const EXTRACT_JS: &str = r#"(function(){
    // 先展开折叠的代码块（CodeMirror 懒渲染，折叠时 DOM 无代码内容）
    var btns = document.querySelectorAll('.ne-codeblock-collapsed-button');
    for (var b = 0; b < btns.length; b++) { try { btns[b].click(); } catch (e) {} }
    var sels = ['main','article','[role="main"]','.markdown-body','.markdown','#content','#article-content','.article-content','.doc-content','.doc-content-wrapper','.markdown-content','[class*="detail-content"]','[class*="doc-detail"]'];
    var best = null, bestLen = 0;
    for (var i = 0; i < sels.length; i++) {
        var el = document.querySelector(sels[i]);
        if (!el) continue;
        var l = el.innerText ? el.innerText.trim().length : 0;
        if (l > bestLen) { bestLen = l; best = el; }
    }
    var root = best || document.body;
    var clone = root.cloneNode(true);
    clone.querySelectorAll('script,style,nav,header,footer,aside,iframe,form,button,input,select,textarea,svg,.ad,.ads,.advertisement').forEach(function(el){el.remove()});

    // CodeMirror 结构 → 标准 <pre><code>（按行拼接，行号 gutter 不在 .CodeMirror-code .CodeMirror-line 内）
    function convertCodeMirror(scope) {
        var cms = scope.querySelectorAll ? scope.querySelectorAll('.CodeMirror') : [];
        for (var m = 0; m < cms.length; m++) {
            var cm = cms[m];
            var lines = cm.querySelectorAll('.CodeMirror-code .CodeMirror-line');
            var parts = [];
            for (var k = 0; k < lines.length; k++) {
                parts.push(lines[k].innerText || lines[k].textContent || '');
            }
            if (!parts.length) continue;
            var pre = document.createElement('pre');
            var codeEl = document.createElement('code');
            codeEl.textContent = parts.join('\n');
            pre.appendChild(codeEl);
            if (cm.parentNode) cm.parentNode.replaceChild(pre, cm);
        }
    }

    // 是否在 pre 内（pre/code 的换行必须保留原样，不转 br）
    function insidePre(node) {
        var p = node.parentNode;
        while (p) { if (p.nodeName === 'PRE') return true; p = p.parentNode; }
        return false;
    }
    // 文本节点内的 \n → <br>（浏览器 pre-line 渲染的换行在 HTML 层只是文本换行符）
    (function preserveNewlines(el){
        if (!el.childNodes) return;
        for (var i = 0; i < el.childNodes.length; i++) {
            var node = el.childNodes[i];
            if (node.nodeType === 3) {
                if (insidePre(node) || !node.nodeValue || node.nodeValue.indexOf('\n') === -1) continue;
                var frag = document.createDocumentFragment();
                node.nodeValue.split('\n').forEach(function(part, j){
                    if (j > 0) frag.appendChild(document.createElement('br'));
                    if (part) frag.appendChild(document.createTextNode(part));
                });
                node.parentNode.replaceChild(frag, node);
            } else if (node.nodeType === 1 && node.nodeName !== 'PRE') {
                preserveNewlines(node);
            }
        }
    })(clone);

    // 转换容器内 CodeMirror（在换行处理之后执行，pre 内换行天然保留）
    convertCodeMirror(clone);
    // 转换 document 级 CodeMirror，再收集容器外的 pre（代码块在独立容器时防丢失）
    convertCodeMirror(document.body);
    var outside = [];
    document.querySelectorAll('pre').forEach(function(p){
        if (!best || !best.contains(p)) outside.push(p.cloneNode(true));
    });
    var html = clone.innerHTML;
    if (outside.length) {
        html += '\n' + outside.map(function(p){ return '<div class="extracted-code">' + p.outerHTML + '</div>' }).join('');
    }
    return html;
})()"#;

/// 等页面 JS 渲染完成后提取正文容器 HTML。
/// 等待策略（解决 SPA 懒加载正文导致"判定过早只抓到目录"的问题）：
/// 1) 轮询正文容器文本长度，需连续两次采样相同（内容稳定）才认为渲染完成；
/// 2) 稳定后再等 1.2s 让异步注入收尾，提取后校验 HTML 长度 >800，否则重置继续等待重试。
async fn fetch_rendered_html(webview: &tauri::WebviewWindow) -> Result<String, String> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    let mut prev_len: i64 = -1;
    loop {
        if std::time::Instant::now() >= deadline {
            return Err("等待页面渲染超时（20 秒），页面可能加载缓慢、需要登录或验证".into());
        }
        let len: i64 = match eval_js_string(webview, SAMPLE_JS).await {
            Ok(s) => serde_json::from_str::<serde_json::Value>(&s)
                .ok()
                .and_then(|v| v.as_i64())
                .unwrap_or(-1),
            Err(_) => -1,
        };
        if len > 200 && len == prev_len {
            // 内容稳定：等异步注入收尾，再提取并校验
            tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
            match eval_js_string(webview, EXTRACT_JS).await {
                Ok(html) if html.chars().count() > 800 => return Ok(html),
                _ => {
                    // 提取过短或失败：重置稳定标记，继续等待重试
                    prev_len = -1;
                }
            }
        } else {
            prev_len = len;
        }
        tokio::time::sleep(std::time::Duration::from_millis(700)).await;
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_internal_hosts() {
        // IPv4 私网/回环/链路本地/CGNAT
        for u in [
            "http://127.0.0.1:8080/",
            "http://localhost/",
            "http://10.0.0.1/",
            "http://172.16.0.1/",
            "http://192.168.1.1/",
            "http://169.254.169.254/latest/meta-data/",
            "http://100.100.100.200/",   // 阿里云元数据
            "http://100.64.0.1/",        // CGNAT 段起点
            "http://100.127.255.255/",   // CGNAT 段终点
            // IPv6 回环/私网/mapped/compatible
            "http://[::1]/",
            "http://[::ffff:127.0.0.1]/",
            "http://[::ffff:10.0.0.1]/",
            "http://[::127.0.0.1]/",
            "http://[fd00::1]/",
            "http://[fe80::1]/",
            "http://[fec0::1]/",  // site-local（已废弃）
        ] {
            assert!(validate_fetch_url(u).is_err(), "应拦截: {}", u);
        }
    }

    #[test]
    fn validate_allows_public_hosts() {
        for u in [
            "https://example.com/",
            "http://8.8.8.8/",
            "http://114.114.114.114/",
            "https://opendocs.alipay.com/pre-open/07wrzc",
            "http://172.32.0.1/",   // 私网段外
            "http://100.128.0.1/",  // CGNAT 段外
            "http://169.255.0.1/",  // 链路本地段外
            // 公网 IPv6（防新分支过度拦截回归）
            "http://[2606:4700::1111]/",
            "http://[::ffff:8.8.8.8]/",
        ] {
            assert!(validate_fetch_url(u).is_ok(), "应放行: {}", u);
        }
    }

    #[test]
    fn validate_rejects_bad_scheme() {
        assert!(validate_fetch_url("file:///etc/passwd").is_err());
        assert!(validate_fetch_url("ftp://example.com/").is_err());
        assert!(validate_fetch_url("javascript:alert(1)").is_err());
    }
}
