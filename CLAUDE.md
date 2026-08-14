# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**SuperTool** — Cross-platform desktop operations management tool. Three-crate Rust workspace (`core`, `cli`, `tauri`) with Vue 3 frontend. CLI directly links to `supertool-core` shared library (zero UDS/HTTP).

## Development Commands

### Frontend / Tauri

```bash
pnpm dev                    # Vue dev server (port 1420)
pnpm build                  # Build Vue frontend (vue-tsc + vite)
pnpm tauri dev              # Tauri dev environment (frontend + backend)
pnpm lint                   # Run oxlint
pnpm lint:fix               # oxlint with auto-fix
pnpm format                 # Prettier
vue-tsc --noEmit            # TypeScript type checking

# 打包构建（使用 build.sh）
pnpm build:app              # CLI + Tauri 构建（不打包，native arch）
pnpm build:app:arm64        # CLI + Tauri 构建（arm64）
pnpm build:app:x64          # CLI + Tauri 构建（x64）
pnpm build:app:universal    # CLI + Tauri 构建（macOS universal）
pnpm build:pkg              # CLI + Tauri + 平台打包（自动检测 OS 生成 dmg/pkg/deb/exe）
pnpm build:pkg:arm64        # 同上，arm64
pnpm build:pkg:x64          # 同上，x64
pnpm build:pkg:universal    # 同上，macOS universal
```

### Rust / CLI

```bash
cargo check --workspace     # Full workspace compilation check (zero errors required)
cargo build --release       # Build all workspace crates
cd cli && cargo build --release       # CLI standalone (~12MB)
cd tauri && cargo build --release     # Tauri binary
```

### Testing

- No test framework is currently configured.

## Architecture

### Workspace Structure

```
Cargo.toml          # [workspace] members = ["core", "cli", "tauri"]
core/               # supertool-core — shared library (single source of truth)
cli/                # stool CLI — directly links to core, standalone
tauri/              # Tauri GUI — directly links to core, Vue 3 frontend
src/                # Vue 3 frontend source (tauri window content)
```

### Data Flow

```
stool CLI ──┐
            ├──▶ supertool-core (CoreService) ──▶ SQLite / SSH / MySQL / Redis / ...
Tauri GUI ──┘
```

Both CLI and GUI directly link to `supertool-core`. **No UDS, no HTTP middleware.**

### Core Library (`core/`)

```
core/src/
├── lib.rs              # Re-exports: db, encryption, logic
├── service.rs          # CoreService — main entry point for all operations
├── db/                 # SQLite data access (database, servers, cicd, projects, openvpn, wireguard, lan)
├── encryption/         # AES-256-GCM encryption/decryption
└── logic/              # Business logic
    ├── data_dir.rs         # Data directory resolution (~/.supertool or ~/.supertool_dir)
    ├── ssh.rs              # SSH connection management
    ├── cicd_deploy.rs      # Deployment engine
    ├── git.rs              # Git operations
    ├── openvpn.rs          # OpenVPN tunnel
    ├── wireguard.rs        # WireGuard tunnel
    ├── nginx.rs            # Nginx config
    └── log_sanitizer.rs    # Log desensitization
```

### CLI (`cli/`)

```
cli/src/
├── main.rs             # clap entry point + command dispatch
├── commands/           # Subcommands: todo, server, cicd, db, log, git
├── runtime.rs          # CliRuntime — manages DB connection and CoreService
├── types.rs            # clap CLI type definitions
├── output.rs           # Output formatting (print_json, print_error, print_success)
├── utils.rs            # shell_quote, is_dangerous_command, format_size
└── guide.rs            # Usage guide output
```

### Tauri GUI (`tauri/`)

```
tauri/
├── src/
│   ├── main.rs             # Tauri app entry
│   ├── lib.rs              # Tauri command library
│   └── commands/           # IPC command handlers (all use CoreService)
├── tauri.conf.json         # Tauri configuration
└── Cargo.toml              # version = "4.0.0", edition = "2024"
```

### Frontend (`src/`)

```
src/
├── main.ts             # Vue app entry (Pinia, i18n, router)
├── App.vue
├── router/             # Vue Router config
├── views/              # Page components
├── components/         # Reusable components
├── layouts/            # Layout components
├── stores/             # Pinia state management
├── utils/              # Utilities (i18n, settings, theme)
├── locales/            # zh-CN, en-US translations
└── assets/             # Static resources (CSS)
```

## Key Configuration

- **Versions**: All crates unified to `version = "4.0.0"`, `edition = "2024"`
- **Data Directory**: `~/.supertool/` (resolved via `data_dir::resolve_data_dir()`, supports `~/.supertool_dir` override)
- **Database**: SQLite at `~/.supertool/supertool.db`
- **Package Manager**: pnpm
- **Vite**: Fixed port 1420 for Tauri dev

## Important Notes

- **开发工具模块（`src/views/devtools/`）设计约定**：2026-08 完成 UI 重构——`DevTools.vue` 为网格卡片首页（分类分区 + 大图标卡片 + 顶部搜索，无侧栏/收藏/最近/折叠菜单）；所有工具页必须套用 `components/ToolPage.vue` 统一外壳（返回按钮 + 图标 + 标题 + 描述，`emit('back')` 返回列表）；应用型工具（如 ApiDebugger）用 `no-scroll` prop 自管理滚动。工具页统一布局模式：`bg-base-100 border rounded-xl` 卡片分区、`btn-primary/btn-outline/btn-ghost` 按钮分级、输入/输出双栏 `textarea font-mono bg-base-200/60`。新增工具需在 `DevToolRegistry.ts` 注册（含 id/icon/category/offline/keywords）并在 `DevTools.vue` 的 `toolComponents` 挂载。`useDevTools.ts`（收藏/最近/折叠）已弃用但被 `ToolCommandPalette.vue` 引用，勿删。历史上部分工具使用过失效 CSS 类（`tool-input`/`result-card`/`hash-result-item` 等，无定义），重构时已全部替换为 tailwind/daisyUI 类。

- **日志聚合（LogAggregator）搜索模式必须全量渲染**：日志行 `whitespace-pre-wrap break-all` 长行会换行，固定行高（24px）虚拟滚动会导致滚动时 `scrollHeight` 波动、浏览器钳制 `scrollTop` 造成回弹（"无法滑到底部"）。历史上两次尝试动态行高测量均失败回滚（`1536e7ce`、`35a263cf`）。搜索模式用 `renderedLines` 全量渲染 + 真实 DOM 定位跳转（`data-log-idx`），流式模式保留虚拟滚动。
- **日志虚拟滚动行高策略**：行高固定的虚拟滚动（spacer = 行数 × 固定高度）在 `whitespace-pre-wrap` 长行下必然回弹。离线日志弹窗（fullLog，大文件无法全量渲染）用"逐行真实行高测量 + 前缀和二分反推"（`fullLogHeightPrefix`/`fullLogRowAtScrollTop`/`fullLogPrefixAt`），行高数组挂在 session（`rowHeights`）上，滚动渐进收敛。
- **实时/连接日志智能吸底约定**：DeployPanel 实时日志、OpenVPNManager/VPNManager 连接日志都实现了"用户上翻暂停自动跟随 + 回到底部按钮"（`xxxUserScrolledUp` + `@scroll` 判断）。注意：程序化赋值 `scrollTop` 不触发 scroll 事件，吸底/回底部后必须手动重置标志，否则按钮不消失、自动吸底被永久跳过（曾因漏重置被 review 拦截）。

- **`cargo fmt` may produce false positives** with edition 2024 — use `cargo check --workspace` zero errors as the standard
- **CLI is fully standalone** — no GUI dependency required, ~12MB binary
- **No UDS/HTTP communication** — everything goes through direct `supertool-core` function calls
- **Sensitive data** (SSH passwords, DB passwords) encrypted with AES-256-GCM, never exposed in CLI output
- **RequiresApproval** — production environment CLI operations require GUI approval; test/dev can bypass
- **Git workflow**: commit immediately after code changes, don't stack uncommitted modifications

## Iteration Plans

- [AI 工具增强迭代计划](docs/ai-tooling-iteration-plan.md) — 定位 AI Agent 运维工具箱：CLI 输出结构化 → MCP server → core 能力扩展 → 审批闭环 + 审计，GUI 只做辅助

## Git Commit 规范 (自动版本号)

**遵循 Conventional Commits 规范，版本号自动更新**：

| Commit 类型 | 版本号变化 | 说明 |
|-------------|------------|------|
| `feat:` | minor (+0.1.0) | 新功能 |
| `fix:` | patch (+0.0.1) | Bug 修复 |
| `feat!:` 或 `BREAKING CHANGE` | major (+1.0.0) | 破坏性变更 |
| `chore:`, `docs:`, `style:` | 不更新 | 维护性提交 |

**示例**：
```bash
git commit -m "feat: add user authentication"    # 4.3.0 → 4.4.0
git commit -m "fix: resolve login timeout"       # 4.4.0 → 4.4.1
git commit -m "feat!: redesign API structure"    # 4.4.1 → 5.0.0
git commit -m "chore: update dependencies"       # 版本号不变
```

**更新位置（4处统一）**：
- `package.json`
- `cli/Cargo.toml`
- `core/Cargo.toml`
- `tauri/Cargo.toml`

**机制**：
- Git hooks 位于 `scripts/hooks/`
- `pnpm install` 时自动配置 hooks（postinstall）
- 新环境首次克隆后运行 `./scripts/init-hooks.sh` 或 `pnpm install`

## Conventions（2026-08 补充）

- **core todo 更新是 PATCH 语义**：`core/src/logic/todo.rs` 的 `update_todo` 只更新请求中提供的字段（含 `orderNum`），不会误清空未传字段；前端 `updateTodo(todo)` 传部分字段即可。
- **@ts-nocheck 状态**：全项目已清零（2026-08 完成，含 tauri-api.ts 157 个历史类型错误）。改 tauri-api 时**必须同步实现与接口声明**（TauriAPI interface），否则其他文件 vue-tsc 会报错。
- **清理 @ts-nocheck 踩过的坑**（后续改这些模块注意）：① tauri-api 存在同名方法两组实现（如 dbRedisStreams 754 行组与 2609 行组），**后者覆盖前者**，签名不一致导致参数失效（pattern/group 被忽略）——改接口声明时必须对照 `getTauriAPI` 对象里最后出现的实现；② 接口声明返回 `() => void` 但实现 `return listen(...)` 返回 Promise<UnlistenFn>（onTerminalClose/onLogs*），调用方把 Promise 当函数调用会 TypeError——接口声明应与实现一致用 `Promise<UnlistenFn>`；③ 表单对象直接展开传给后端（`{...serverForm.value}`）会带 tagsInput 等多余字段——用白名单重建；④ 日志行/服务器组等类型用本地内联接口时与 types.ts 不同步（缺 html/sortKey/parentId 字段）——优先用 types.ts 类型。
- **LAN 事件约定**：发送事件用 `tauri-api` 的 `lanBroadcast*` 系列方法（走 `lan_broadcast_*` 后端命令，参数为 JSON 字符串）；`on*` 系列只用于监听（`listen` 事件）。误用 `onXxx(todoId, editor)` 发送会导致运行时 TypeError。
- **lint 配置**（.oxlintrc.json）：`eqeqeq` 允许 `!= null` 惯用法（`null: ignore`）、`no-unused-vars` 豁免 `_` 前缀与 catch 参数；全项目 lint 当前 0 错误。

- **Tauri 中原生 `confirm()`/`prompt()` 不弹窗**（静默返回 false/null）：删除/覆盖确认必须用 `@tauri-apps/plugin-dialog` 的 `confirm()` 或组件内自定义弹层（参照 KanbanBoard 的 pendingConfirm/pendingBlock）；项目已有 plugin-dialog ^2.7.0 依赖。
- **后端 Tauri 命令参数名必须与前端调用一致**（serde 反序列化不映射别名）：周报模块曾因前端传 `params`/`startDate` 而后端要 `limit`/`weekStart` 导致整条链路失效（历史为空/字段空白/详情打不开）。新增命令前核对 `tauri/src/main.rs` 注册签名与 `tauri-api.ts` 调用参数。
- **Tauri 命令失败返回 `{success:false,error}` 不抛异常**：调用方必须显式检查 `success === false`（DBManager 曾把语法错误记为成功）。批量/写操作前确认返回结构。
- **UI 状态与后端字段语义对齐**：`types.ts` 的接口是准绳（Notes 的 pinned、WeeklyReport 的 weekStart/weekEnd 曾与本地重复接口冲突导致类型错误与字段丢失），改动前端类型先查 `core/src/logic/*.rs` 实际字段。
- **DevTools 加解密/编码工具格式约定**（2026-08）：`EncryptTool.vue` 密钥/IV/明文支持 hex/base64/utf8 格式选择、密文支持 base64/hex（字节流统一走 `parseBytes`/`bytesToString`/`bytesToWordArray`/`wordArrayToBytes` 工具函数）。易错点：① gm-crypto `SM4.encrypt/decrypt` 的 key/iv 只接受 32 位 hex 字符串（`bytesToHex` 转换）；② crypto-js bundle 含 `lib-typedarrays`，`WordArray.create(Uint8Array)` 正确按字节处理，无需手工转 words；③ `CryptoJS.Rabbit` 是 passphrase 模式（随机 salt，OpenSSL 格式），密文必须带 `Salted__` 前缀（8B salt + ciphertext），解密端先 `bytesToBase64` 还原 OpenSSL 字符串再 decrypt；④ 随机字节**不可**直接 `bytesToUtf8`（几乎必然非法 UTF-8 抛异常），UTF-8 格式的随机 IV 用 16 字符可读串生成；⑤ 非对称（SM2/RSA）库内部按 UTF-8 处理，加密输入仅支持文本。`HexTool.vue` String↔Hex 走 UTF-8 字节级转换（`TextEncoder`/`TextDecoder(fatal)`），Latin-1 为可选旧行为。
- **DevTools 编码/进制工具约定**（2026-08）：`toolUtils.ts` 的 `BASE64_CHARS` 是 `'A-Za-z+/'`（i<10 用数字），**勿改回带 `0123456789` 的旧表**——base 62/63 会错位成 `0`/`1`；base>36 的字符大小写敏感（`customParseBigInt` 转小写兜底）。`UnicodeTool.vue` 逆向解析用三个正则常量（`UNICODE_RE`/`HTML_ENTITY_RE`/`CSS_ENTITY_RE`）与 `fromCodePointSafe`（超 0x10FFFF → U+FFFD），改解析逻辑时优先复用，勿用裸 `String.fromCodePoint`（超范围会抛 RangeError 中断渲染）。`CryptoTool.vue` 摘要输出 hex/base64 由 `formatDigest` 统一（sm3 只出 hex，base64 需 `hexToBase64` 手动转）。`AsciiTool.vue` 用 `codePointAt` 而非 `charCodeAt`（emoji/非 BMP 代理单元问题）。
- **DevTools 编码/时间工具约定**（2026-08）：`JwtTool.vue` 的 base64url 解码必须走 `TextDecoder('utf-8')`（`atob` 直接返回 Latin-1，中文 payload 必乱码）。`UrlTool.vue` 编码策略三选：component（`encodeURIComponent`）/ uri（`encodeURI`）/ form（空格→`+`），**解码端 form 模式会把 `+` 还原为空格**（其他模式不动）。`TimeTool.vue` 的 `Date→Timestamp` 用 `zonedTimeToUtcMs`（Intl 时区偏移两次迭代，支持 DST），`new Date('YYYY-MM-DD HH:mm:ss')` 空格格式在 Safari/WKWebView 下 Invalid Date，必须走 `parseFlexibleDateTime`（替换为 `T` 分隔）。
- **DevTools HTML→Markdown 约定**（2026-08）：`HtmlToMdTool.vue` 用前端 `turndown` + `turndown-plugin-gfm`（headingStyle atx / fenced 代码块 / GFM 表格），后端 `convert_html_to_md`（html2md）已移除。易错点：① turndown 默认**不剥离** script/style/head/form 等，必须 `addRule`（用 `nodeName.toUpperCase()` 判断，`'svg'` 不在 `HTMLElementTagNameMap` 类型里不能用标签名数组）；② **不要剥 `form` 容器**（部分站点正文在 form 内，只剥 INPUT/BUTTON/SELECT/OPTION/TEXTAREA/LABEL 控件）；③ GFM 表格要求表头行，无 `<th>` 的表格 turndown 保留 HTML 原样（规范行为，非 bug）；④ `turndown-plugin-gfm` 无类型声明，需在 `vendor.d.ts` 加 `declare module`。
- **Markdown 渲染安全约定**（2026-08）：`composables/useMarkdownRenderer.ts` 的 `renderMarkdown` 是全局唯一 Markdown 渲染入口（marked + DOMPurify + hljs，首次被 HtmlToMdTool 预览使用）。安全铁律：① DOMPurify `ADD_ATTR` **禁止加 `on*` 事件属性**（曾因 `onclick` 白名单导致 XSS，Tauri `csp: null` 无拦截）；② code renderer 的 `language` 必须经 `escapeHtml` 转义，code 一律走 hljs 转义输出，**禁止原样拼 HTML**（"已高亮"跳过分支已删除）；③ 复制按钮用 `data-copy-target` + `setupCopyCode()` 事件委托（返回 handler 绑容器 click），**禁用内联 onclick**；④ 预览容器样式用 scoped `:deep()` 自包含（`.markdown-body` 不要依赖 NoteManager 的全局样式是否加载）。
- **DevTools 网页抓取限制**（2026-08）：`HtmlToMdTool` 的 `fetch_page_content` 用 reqwest 直接 GET，**无法抓取 JS 动态渲染（SPA）页面**（如支付宝文档 opendocs.alipay.com——HTML 仅 18KB 空壳、正文 15 字符，内容靠 webpack 异步加载）。前端用 `isSpaShell`（HTML>1500 且去 script/style 后可见文本<200）检测空壳并提示用户浏览器打开复制粘贴；检测阈值勿放宽到可见文本≥200（会误伤 SSR 慢渲染页的提示路径），打开浏览器用独立 `fetchedUrl` 变量 + `https?://` 协议校验（勿读当前输入框，用户可能已改）。
- **DevTools SPA 渲染抓取**（2026-08）：`fetch_page_content_js` 命令用隐藏 WebView（`WebviewWindowBuilder.visible(false)`）执行 JS 后提取正文，前端 `HtmlToMdTool` 检测 SPA 空壳（`isSpaShell`）自动降级调用，失败则提示浏览器打开复制。关键约定：① 拿 eval 返回值用 `eval_with_callback`（Tauri≥2.11.5，回调 `Fn(String)` 收 JSON 序列化结果；oneshot Sender 需 `Mutex<Option<Sender>>` 包住——Fn 闭包不能 move）；② 窗口 label 用原子计数器（时间戳会碰撞且 close 失败后同 label 重建永久失败），close 错误打 `log::error!`；③ 渲染完成判定=正文容器存在且文本>200 或整体文本>1000（防 404/验证页提前判定），eval 超时 2s、总超时 15s；④ 安全前提：capabilities 仅 local 域，远程页面无 Tauri IPC 权限，隐藏 WebView 加载任意 URL 无本地数据面；⑤ 提取 JS 剥离 nav/header/footer/iframe/form 等，正文选择器列表 `main, article, [role="main"], .markdown-body, #content, .article-content, .doc-content, .markdown`。
- **DevTools SPA 抓取提取与 SSRF 加固**（2026-08）：① 正文提取等待策略：轮询候选容器「文本最多者」长度，**连续两次采样相同**才判定渲染完成（防 SPA 懒加载正文时提前提取只抓到目录），稳定后再等 1.2s、提取后校验 HTML>800 字符，总超时 20s——曾因「整体文本>1000 即判定」导致支付宝文档页只抓到侧边目录；② SSRF 防护铁律：`is_blocked_host` 必须用 `Url::host()` 的 `Host<&str>` 枚举匹配——**`host_str()` 对 IPv6 返回带方括号 "[::1]" 导致 parse<IpAddr> 恒失败绕过**；reqwest 禁自动重定向（`Policy::none()`）手动逐跳每跳重校验；`on_navigation` 用 `port_or_known_default()` 锁同 host:port；拦 CGNAT 100.64/10（阿里云元数据 100.100.100.200）、mapped/compatible/site-local IPv6；③ fetch.rs 的 `#[cfg(test)]` 单测（fetch:: 模块 3 个）是 SSRF 回归防线，改校验逻辑必须同步更新。
- **DevTools SPA 提取换行与代码块**（2026-08）：① **turndown 会把文本节点内的 `\n` 折叠成空格**（HTML 规范），`white-space: pre-line` 渲染的页面（支付宝文档等）换行只在文本节点里，必须提取时把正文容器文本节点的 `\n` 转成 `<br>`（递归遍历 childNodes，**跳过 pre 内节点**——代码换行必须原样，insidePre 向上查祖先）；② 代码块可能在正文容器**外**（独立 code-section），提取时需 `document.querySelectorAll('pre')` 收集容器外 pre 附加到结果（`extracted-code` 包装）；③ 验证方式：happy-dom（vitest 自带 20.9.0）模拟 DOM 跑 EXTRACT_JS + turndown 完整链路。
- **DevTools SPA 代码块提取（CodeMirror）**（2026-08）：支付宝等文档站的代码块用 **CodeMirror** 渲染（页面加载 `CodeMirror.js`），不是标准 `<pre><code>`：① 结构是 `.CodeMirror > .CodeMirror-code > pre.CodeMirror-line`（每行一个 pre），必须提取时按行拼接转成标准 `<pre><code>`（行号在 `.CodeMirror-gutter .CodeMirror-linenumber`，选择 `.CodeMirror-code .CodeMirror-line` 可排除）；② 代码块可能**默认折叠**（`ne-codeblock-collapsed-button` 按钮，CodeMirror 懒渲染折叠时 DOM 无代码）——提取前要 `click()` 展开，采样判定（SAMPLE_JS）须把 `.CodeMirror` 的 innerText 计入，否则折叠代码块文本不触发渲染完成判定；③ 判定方式：从页面 HTML 找 `CodeMirror.js`/chunk 里 `className:"ne-code"`、`collapsed` 状态确认；验证用 happy-dom 构造 `.CodeMirror` 结构跑 EXTRACT_JS。
