# AI 配置助手 —— 设计与安全边界

目标：界面配置项太多、上手难，让一个内置助手替用户**读配置、查原因、给建议、出提案**，并把「这个字段填什么」讲清楚。
入口：侧边栏「AI 助手」页 + 悬浮球唤起（模型未配置时首屏直接引导去设置）。

## 分层

| 位置 | 职责 |
|---|---|
| `core/src/logic/ai_provider.rs` | 提供商/模型配置存取（settings 表），apiKey 加密、对外掩码、路由解析。与 CLI 同源 |
| `tauri/src/assistant/llm.rs` | OpenAI / Anthropic 双协议流式客户端。请求体构造与 SSE 解析都是纯函数 |
| `tauri/src/assistant/safety.rs` | 三条红线：脱敏、文件读取白名单、提案密钥字段黑名单 |
| `tauri/src/assistant/tools.rs` | 工具注册表 + 15 个工具 + CICD 规则校验 + 提案白名单校验 |
| `tauri/src/assistant/agent.rs` | 多轮工具调用循环、上下文预算裁剪、事件节流、turn 注册与中止 |
| `tauri/src/assistant/context.rs` | token 估算与历史裁剪（按模型的上下文窗口，不是全局常量） |
| `tauri/src/assistant/knowledge.rs` | 内置知识库：字段释义/坑位说明 + 报错特征→原因→处理办法 |

## 模型接入（用户可自定义）

`settings.ai_providers` = JSON 数组，每项：

- `name` 提供商名称
- `protocol` `openai`（兼容 `/chat/completions`，含各类网关/Ollama/LM Studio）或 `anthropic`（`/v1/messages`）
- `baseUrl` 接口地址，允许内网/本机；拒绝非 http(s) 与 URL 内嵌凭据
- `apiKey` **AES-256-GCM 加密落盘**；读取接口只回 `apiKeyMasked`；保存时空值/掩码值 = 沿用旧密钥
- `models[]` 每项含 `id`（模型 ID，自由填写）、`label`、`contextWindow`（上下文窗口）、`maxOutputTokens`

`settings.ai_active_model` = 当前选用的 `{providerId, modelId}`，首个提供商保存时自动设为当前。
上下文窗口用于两处：历史裁剪预算、单次回复输出上限（输出上限会被收敛到窗口内）。

## 工具集（15 个，四类能力）

读配置：`get_app_snapshot`、`list_servers`、`list_server_groups`、`list_db_connections`、
`list_cicd_configs`、`get_cicd_config`、`get_deploy_history`
诊断：`validate_cicd_config`、`analyze_deploy_error`、`test_server_connection`
路径：`find_local_path`、`inspect_local_path`、`detect_local_project`（填 CICD 的 localPath/构建目录/产物目录用，
复用部署向导同一套 `scan_project_impl`；**只返回路径与元信息，读不到文件内容**）
教学：`search_usage_guides`、`get_usage_guide`、`open_config_page`
变更：`propose_config_change`（**唯一**能导致写库的工具，且必须用户确认）

## 安全红线（都有对应单测）

1. **助手没有写库能力**：`propose_config_change` 只产出提案事件，前端渲染成确认卡片，
   用户点确认后由界面调用既有命令（`add_server` / `save_cicd_config` / `save_ai_provider` …）写入。
   工具注册表里没有 save/update/delete/deploy/exec 类工具，测试
   `registry_exposes_no_dangerous_capabilities` 同时按精确名与关键字双重断言。
2. **文件内容只有白名单那一条路**：唯一的内容读取入口是 `read_text_file_in`，
   只允许 `deploy-logs` 目录（canonicalize 后前缀校验 + 8MB 上限），路径来自数据库而非用户输入，仍做二次校验。
   注意主界面既有的 `read_log_file(file_path)` 无路径校验，**不要**把它包成助手工具。
   读类工具的数据源固定为 core 既有查询；`db_connections` 查询**不 SELECT password 列**。
   路径检索（`assistant/paths.rs`）是刻意开的例外，三条约束：遍历只发生在搜索根（主目录 + `git_scan_directories`
   + 应用数据目录）内且有深度/访问数/结果数上限、不跟随软链接；`.ssh`/`.gnupg`/`.aws`/`.config`/钥匙串/
   `.supertool` 等凭据位置既不可枚举也不可 stat；返回值只有路径/类型/大小/修改时间/构建标志文件，
   `gitRemoteUrl` 还会先抹掉可能内嵌的口令。
3. **不能让助手知道密码类信息**：
   - 工具返回值统一过 `safety::deep_redact`（进上下文前在 agent 层调用；覆盖 `password/passwd/pwd/passphrase/apiKey/secret/token/privateKey/presharedKey/sshKeyPath/keyPath/keyFile/credential/authorization/sessionKey/certPassword/totpSecret`，
     键名归一后判断，`sshKeyPath` 这类 camelCase 也不会漏）；
   - 日志正文过 `redact_text`（PEM 私钥、`Bearer xxx`、`sk-xxx`、`key: value`、`scheme://user:pass@host`）；
   - 提案走 `assert_no_secret_fields` + 分目标字段白名单，白名单外字段一律拒绝并回错误说明；
   - 连通性测试用**已存凭据**在服务端完成，返回体只有 `{ok, reason, hints}`，凭据不进任何响应、不进上下文；
   - 提示词构造只接受不含 apiKey 的 `RouteInfo` 摘要。
   - 因此「新建服务器」流程是：用户给 IP/端口/用户名/分组等 → 助手出提案 → 用户在表单里自己填密码/密钥。
4. **事件节流**（沿用 6.50.6 部署卡死的教训）：文本增量在后端按 ≥120 字 / ≥80ms 攒批再 emit，
   不做逐 token 发送；工具结果按 `MAX_TOOL_RESULT_CHARS` 截断（保留头尾，报错通常在尾部）。
5. **不承诺已生效**：系统提示词明确「你没有任何写入能力」，提案后返回 `queued`，模型只能说「等你确认」。

## 已知边界

- 会话历史由界面持有（后端不落库），重启后不保留；跨设备同步不在范围内。
- token 估算是保守近似（CJK 1 字 ≈ 1 token，其余 4 字符 ≈ 1 token），不引分词器。
- 知识库是内置静态条目（内容来自 AGENTS.md 与 docs 的结论），不读仓库文件；新增规则要改 `knowledge.rs`。
- `validate_cicd_config` 是字段级规则检查，不等于跑一次部署；真正的构建校验仍在部署引擎里。
