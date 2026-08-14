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

/// 采样 JS：先展开折叠的代码块（ne-doc/CodeMirror 懒渲染）并交替滚动触发 IntersectionObserver 懒加载，
/// 再返回候选容器与代码容器文本的最大长度。
/// 支付宝文档页面用 IntersectionObserver 懒加载（元素进入视口才渲染代码块），
/// 隐藏 WebView 不滚动则代码块永不在视口 → 必须模拟滚动（700ms 轮询间隔天然分帧）
const SAMPLE_JS: &str = r#"(function(){
    // ne-doc 代码块默认可能折叠（ne-codeblock-collapsed-button），点击后 CodeMirror 才渲染代码
    var btns = document.querySelectorAll('.ne-codeblock-collapsed-button, .codeblock-collapsed-button, [class*="collapsed-button"]');
    for (var b = 0; b < btns.length; b++) { try { btns[b].click(); } catch (e) {} }
    // 交替滚动到底/顶：触发 IntersectionObserver 懒加载（每次采样间隔 700ms，天然分帧）
    window.__spaScrollToggle = !window.__spaScrollToggle;
    var docEl = document.documentElement;
    var scrollH = Math.max(docEl.scrollHeight || 0, document.body ? document.body.scrollHeight : 0);
    window.scrollTo(0, window.__spaScrollToggle ? scrollH : 0);
    var sels = ['main','article','[role="main"]','.markdown-body','.markdown','#content','#article-content','.article-content','.doc-content','.doc-content-wrapper','.markdown-content','[class*="detail-content"]','[class*="doc-detail"]'];
    var best = 0;
    for (var i = 0; i < sels.length; i++) {
        var el = document.querySelector(sels[i]);
        if (el) { var l = el.innerText ? el.innerText.trim().length : 0; if (l > best) best = l; }
    }
    // 代码块（ne-code/ne-codeblock/CodeMirror）文本并入渲染判定（代码块可能在容器外/刚展开/刚懒加载）
    var codeBest = 0;
    var blocks = document.querySelectorAll('.ne-code, .ne-codeblock, .CodeMirror');
    for (var k = 0; k < blocks.length; k++) {
        var t = blocks[k].innerText ? blocks[k].innerText.trim().length : 0;
        if (t > codeBest) codeBest = t;
    }
    return Math.max(best, codeBest);
})()"#;

/// 提取 JS：适配 ne-doc（飞书/Lark 文档引擎）与 CodeMirror 代码块。
/// ne-doc 用自定义标签（ne-p/ne-text/ne-uli/ne-table/ne-card）渲染正文，turndown 无法识别，
/// 需先规范化为标准 HTML；代码块（.ne-code/.ne-codeblock/.CodeMirror）统一转标准 pre/code。
const EXTRACT_JS: &str = r#"(function(){
    // 1. 展开折叠的代码块（ne-doc 的代码块可能默认折叠，CodeMirror 懒渲染，折叠时 DOM 无代码）
    var btns = document.querySelectorAll('.ne-codeblock-collapsed-button, .codeblock-collapsed-button, [class*="collapsed-button"]');
    for (var b = 0; b < btns.length; b++) { try { btns[b].click(); } catch (e) {} }
    // 滚动触发 IntersectionObserver 懒加载（提取前滚到底再回顶，保证最后一批代码块已渲染）
    window.scrollTo(0, document.documentElement.scrollHeight || document.body.scrollHeight);
    window.scrollTo(0, 0);

    // 2. 选正文容器（文本最多者）
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

    // 3. ne-doc 标签规范化：ne-text → 文本/strong；ne-p → p；ne-uli → li（相邻包 ul）；
    //    ne-card → img；ne-viewer-b-filler 删除；ne-table → 标准 table（首行 td → th）。
    //    用 renameTag（移动子节点）而非 innerHTML 复制，避免自定义标签序列化丢失内容
    function renameTag(node, tagName) {
        var nn = document.createElement(tagName);
        while (node.firstChild) { nn.appendChild(node.firstChild); }
        // 仅复制非事件属性（跳过 on*，防远程页面 onerror 等属性进入回传 HTML；下游有 DOMPurify 兜底）
        for (var a = 0; a < node.attributes.length; a++) {
            var attrName = node.attributes[a].name;
            if (attrName.toLowerCase().indexOf('on') === 0) { continue; }
            nn.setAttribute(attrName, node.attributes[a].value);
        }
        node.parentNode.replaceChild(nn, node);
        return nn;
    }
    function normalizeNeTags(el) {
        el.querySelectorAll('.ne-viewer-b-filler').forEach(function(f){ f.remove(); });
        el.querySelectorAll('ne-text').forEach(function(t){
            var bold = t.getAttribute('ne-bold') === 'true';
            if (bold) { var s = document.createElement('strong'); s.textContent = t.textContent || ''; t.parentNode.replaceChild(s, t); }
            else { var tn = document.createTextNode(t.textContent || ''); t.parentNode.replaceChild(tn, t); }
        });
        el.querySelectorAll('ne-strong').forEach(function(s){ renameTag(s, 'strong'); });
        el.querySelectorAll('ne-em').forEach(function(s){ renameTag(s, 'em'); });
        el.querySelectorAll('ne-p').forEach(function(p){ renameTag(p, 'p'); });
        // 列表：删符号列 ne-uli-i，内容列 ne-uli-c → span，ne-uli → li
        el.querySelectorAll('ne-uli-i').forEach(function(i){ i.remove(); });
        el.querySelectorAll('ne-uli-c').forEach(function(c){ renameTag(c, 'span'); });
        el.querySelectorAll('ne-uli').forEach(function(uli){ renameTag(uli, 'li'); });
        el.querySelectorAll('ne-card').forEach(function(card){
            var img = card.querySelector('img');
            if (img) {
                // 只保留 src/alt，丢弃 onerror 等事件属性
                var ni = document.createElement('img');
                if (img.getAttribute('src')) { ni.setAttribute('src', img.getAttribute('src')); }
                if (img.getAttribute('alt')) { ni.setAttribute('alt', img.getAttribute('alt')); }
                card.parentNode.replaceChild(ni, card);
            }
            else { card.remove(); }
        });
        // 表格：先 td/tr，最后 table（首行 td → th）
        el.querySelectorAll('ne-td').forEach(function(td){ renameTag(td, 'td'); });
        el.querySelectorAll('ne-tr').forEach(function(tr){ renameTag(tr, 'tr'); });
        el.querySelectorAll('ne-table').forEach(function(tbl){
            var table = renameTag(tbl, 'table');
            var firstRow = table.querySelector('tr');
            if (firstRow) {
                firstRow.querySelectorAll('td').forEach(function(td){ renameTag(td, 'th'); });
            }
        });
    }
    // 相邻 li 包成 ul（ne-doc 的 ne-uli 平铺，无外层 ul；br/文本不断组，仅块级元素分隔）
    function wrapAdjacentLists(el) {
        Array.prototype.slice.call(el.children).forEach(function(child){ wrapAdjacentLists(child); });
        var group = null;
        Array.prototype.slice.call(el.childNodes).forEach(function(node){
            if (node.nodeType === 1 && node.nodeName === 'LI') {
                if (!group) { group = document.createElement('ul'); el.insertBefore(group, node); }
                group.appendChild(node);
            } else if (node.nodeType === 1 && node.nodeName !== 'BR') {
                group = null;
            }
        });
    }

    // 4. 是否在 pre 内（pre/code 的换行必须保留原样，不转 br）
    function insidePre(node) {
        var p = node.parentNode;
        while (p) { if (p.nodeName === 'PRE') return true; p = p.parentNode; }
        return false;
    }
    // 5. ne-doc 标签规范化（ne-p/ne-text/ne-uli/ne-table/ne-card → 标准标签）
    normalizeNeTags(clone);

    // 6. 代码块统一转标准 pre/code（.ne-code/.ne-codeblock/.CodeMirror 用 innerText 取代码；
    //    行号 gutter 不在 .CodeMirror-code .CodeMirror-line 内；ne-code 内容含行间 \n。
    //    必须在 preserveNewlines 之前执行——否则代码内的 \n 会被转成 <br>，innerText 提取丢换行）
    function convertCodeBlocks(scope) {
        var blocks = scope.querySelectorAll('.ne-code, .ne-codeblock, .CodeMirror');
        for (var i = 0; i < blocks.length; i++) {
            var blk = blocks[i];
            // 跳过含子代码块的容器（从最外层容器取 innerText 已包含全部代码行）
            if (blk.querySelector('.ne-code, .ne-codeblock, .CodeMirror')) continue;
            // 排除 header/工具栏（语言标签、按钮等），只保留代码内容
            var tmp = blk.cloneNode(true);
            tmp.querySelectorAll('.ne-codeblock-header, .codeblock-header, [class*="codeblock-header"], [class*="toolbar"], button, select').forEach(function(h){ h.remove(); });
            var text = tmp.innerText || tmp.textContent || '';
            if (!text.trim()) continue;
            var pre = document.createElement('pre');
            var codeEl = document.createElement('code');
            codeEl.textContent = text.trim();
            pre.appendChild(codeEl);
            blk.parentNode.replaceChild(pre, blk);
        }
    }
    convertCodeBlocks(clone);

    // 7. 文本节点内的 \n → <br>（pre-line 渲染的换行在 HTML 层只是文本换行符；pre 内自动跳过）
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

    // 8. 相邻 li 包成 ul
    wrapAdjacentLists(clone);

    // 9. 转换 document 级代码块，再收集容器外的 pre（代码块在独立容器时防丢失）
    convertCodeBlocks(document.body);
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
