# 审计报告：Config 读取 / System Prompt / Session 持久化 / Hook 权限系统

**审计日期**: 2026-06-05  
**审计范围**: `tauri/src/commands/claw_chat.rs`, `tauri/src/commands/claw_config.rs` vs 上游 `claw-tools/src/lib.rs`, `claw-runtime/src/session.rs`, `claw-runtime/src/conversation.rs`, `claw-runtime/src/permissions.rs`

---

## 1. Config 读取

### 1.1 上游 Config 体系

上游有**两级 config**，结构清晰：

| 作用域 | 路径 | 格式 | 说明 |
|--------|------|------|------|
| Global | `~/.claw/settings.json` | JSON object | 全局配置（通过 `CLAW_CONFIG_HOME` 可覆盖） |
| Settings (per-project) | `{cwd}/.claw/settings.local.json` | JSON object | 项目级覆盖配置 |

上游 `config_home_dir()` (lib.rs:6381) 优先读取 `CLAW_CONFIG_HOME` 环境变量，fallback 到 `$HOME/.claw`。  
上游 config 读写使用 `read_json_object` / `write_json_object` (lib.rs:6396)，通用 JSON object 操作，支持空文件和文件不存在两种情况。

### 1.2 我们的 Config 体系

我们的 `claw_config.rs` 使用**单一配置文件** `~/.claw/config.json`：

```
config_path() → ~/.claw/config.json
```

结构体 `ClawConfig` 包含：`api_key`, `base_url`, `model`, `provider`, `max_iterations`, `skill_bytes_cap`, `max_retries`, `reasoning_effort`, `tool_output_truncation`, `auto_compaction`。

### 1.3 差异与问题

| 项目 | 上游 | 我们 | 风险 |
|------|------|------|------|
| 配置文件路径 | `~/.claw/settings.json` (Global) + `.{cwd}/.claw/settings.local.json` | `~/.claw/config.json` | ⚠️ **路径不一致**。上游使用 `settings.json`，我们使用 `config.json`。如果用户同时使用上游 CLI 和我们的 Tauri GUI，两者的 config 互不可见。 |
| CLAW_CONFIG_HOME 支持 | ✅ 支持 | ❌ 不支持 | ⚠️ 上游尊重 `CLAW_CONFIG_HOME` 环境变量，我们硬编码 `dirs::home_dir()`。 |
| 项目级配置 | ✅ `.claw/settings.local.json` | ❌ 不支持 | ⚠️ 上游支持 per-project 配置覆盖，我们没有。 |
| Config 字段集合 | 上游 config 是通用 JSON object，支持任意字段（如 `trustedRoots`, `hooks`, `permissions`, `mcpServers` 等） | 固定结构体，只支持我们定义的字段 | ⚠️ 上游 config 中的 hooks、permissions、MCP 配置等字段被我们的实现完全忽略。 |
| 安全性 | 空文件和不存在都正确处理 | 不存在返回 default，空文件会报 JSON 解析错误 | ⚡ 低风险：`serde_json::from_str("")` 会失败。 |

### 1.4 `setup_env_from_claw_config` 环境变量设置

```rust
// claw_chat.rs:517-557
pub(crate) fn setup_env_from_claw_config() -> Result<(), String> {
    let config = read_claw_config()?;
    if config.api_key.is_empty() { return Ok(()); }

    if has_base_url {
        // OpenAI-compatible 模式
        OPENAI_API_KEY, OPENAI_BASE_URL, OPENAI_MODEL
    } else {
        // 根据 model 名前缀推断 provider
        if starts_with("claude") → ANTHROPIC_API_KEY, ANTHROPIC_MODEL
        if starts_with("openai") → OPENAI_API_KEY, OPENAI_MODEL
        if starts_with("grok")   → XAI_API_KEY, XAI_MODEL
        else                      → ANTHROPIC_API_KEY, ANTHROPIC_MODEL (default)
    }
}
```

**发现：**

| 问题 | 严重度 | 说明 |
|------|--------|------|
| `unsafe { std::env::set_var }` | ⚠️ 中 | 在多线程程序中修改进程环境变量是 unsafe 行为（Rust edition 2024）。Tauri 是多线程的，如果有其他线程同时读取环境变量，可能产生数据竞争。但目前实际使用中通常在初始化阶段调用，风险可控。 |
| provider 推断不够全面 | ⚡ 低 | 不支持 `dashscope`/`qwen`/`deepseek` 等模型前缀的自动推断。如果 model 名为 `qwen-max`，会 fallback 到 Anthropic，但用户实际可能要走 OpenAI-compatible 路径。 |
| base_url 存在时无条件设为 OpenAI | ⚡ 低 | 有 base_url 时一律设 OPENAI_*，即使 model 是 `claude-sonnet-4-6`。这在使用 Claude 中转站时是正确的（OpenAI-compatible API），但如果用户同时配置了 base_url + Anthropic 模型名且期望走 Anthropic native API，会出问题。当前这不算问题，因为有 base_url 时走 OpenAI-compatible 是标准做法。 |

---

## 2. System Prompt

### 2.1 上游 System Prompt 构建

上游 `build_agent_system_prompt` (lib.rs:4199):

```rust
fn build_agent_system_prompt(subagent_type: &str, model: &str) -> Result<Vec<String>, String> {
    let cwd = std::env::current_dir()?;
    let mut prompt = load_system_prompt(
        cwd,
        DEFAULT_AGENT_SYSTEM_DATE.to_string(),  // 上游用固定日期 "unknown"！
        std::env::consts::OS,
        "unknown",                               // os_version 也是 "unknown"
        model_family_identity_for(model),
    )?;
    prompt.push(format!("You are a background sub-agent of type `{subagent_type}`..."));
    Ok(prompt)
}
```

上游 `load_system_prompt` (prompt.rs:621) → `load_system_prompt_with_context` (prompt.rs:635):
1. 通过 `ConfigLoader::default_for(&cwd).load()` 加载运行时配置
2. 通过 `discover_with_git_and_rules_import()` 发现项目上下文（CLAUDE.md 等）
3. 使用 `SystemPromptBuilder` 组装完整 prompt（包含 intro、system、tools、context、config 等 sections）

### 2.2 我们的 System Prompt 构建

```rust
// claw_chat.rs:460-487
pub(crate) fn claw_agent_system_prompt(skill_bytes_cap: usize) -> String {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
    let model = env!("ANTHROPIC_MODEL") or env!("OPENAI_MODEL") or "claude-sonnet-4-6";

    let base_prompt = match runtime::load_system_prompt(
        &cwd,
        chrono::Utc::now().format("%Y-%m-%d").to_string(),  // 我们用真实日期 ✅
        std::env::consts::OS.to_string(),
        "26.5".to_string(),  // macOS 版本号，上游用 "unknown"
        api::model_family_identity_for(&model),
    ) { ... };

    let skills_section = load_hermes_skills(skill_bytes_cap);
    // append skills
}
```

**差异分析：**

| 项目 | 上游 | 我们 | 影响 |
|------|------|------|------|
| 日期 | `DEFAULT_AGENT_SYSTEM_DATE`（固定值 "unknown"） | `chrono::Utc::now()` 实时日期 | ✅ **我们更好**，日期对 prompt 中的日期相关指令有意义。 |
| os_version | `"unknown"` | `"26.5"` (macOS) | ✅ **我们更好**，精确的 OS 版本有助于工具兼容性判断。 |
| load_system_prompt 来源 | `claw-tools/src/lib.rs` re-export | `claw-runtime::load_system_prompt` | ⚠️ 两者是**同一个函数**，上游 lib.rs 只是 re-export runtime 的。兼容性 OK。 |
| subagent_type 后缀 | 上游追加 "You are a background sub-agent..." | 我们**不追加** | ✅ 合理差异：我们是主对话而非 sub-agent。 |
| Hermes Skills 注入 | ❌ 无 | ✅ 从 `~/.hermes/skills/` 加载 | ✅ 这是我们独有的增值功能。 |

### 2.3 `load_hermes_skills` 兼容性

```rust
// claw_chat.rs:348-457
pub(crate) fn load_hermes_skills(skill_bytes_cap: usize) -> String {
    // 1. 从 ~/.hermes/skills/ 构建 DESCRIPTION.md 索引
    // 2. 对 github, coding-ultimate-rules, dev, devops, software-development 
    //    类别加载完整 SKILL.md
    // 3. 有 byte cap (默认 200KB)
}
```

**发现：**

| 项目 | 评估 |
|------|------|
| 路径 | `~/.hermes/skills/` — 与上游 `~/.claude/commands/` 路径完全不同，无冲突。 |
| 硬编码类别名 | `["github", "coding-ultimate-rules", "dev", "devops", "software-development"]` — 如果用户添加新的 coding 相关 skill 类别名，不会被自动加载。 |
| byte cap | 默认 200KB，来自 `ClawConfig.skill_bytes_cap`，用户可配置（10KB-2MB），设计合理。 |
| Unicode 安全 | `&brief[..200]` 在 UTF-8 字符串上是字节截断，可能 panic（如果第 200 字节在多字节字符中间）。应使用 `.chars().take(200).collect::<String>()`。 |
| `break` 语义 | `break` 只跳出内层 `for sub in walker`，不跳出外层 `for category`。所以 cap 达到时只跳过当前类别的剩余 skill，不跳过后续类别。这是个 bug。 |

---

## 3. Session 持久化

### 3.1 上游 Session 存储

- **格式**: JSONL (JSON Lines) + 支持旧版 JSON 格式读取
- **路径**: `~/.local/share/opencode/` (全局共享)
- **文件名**: `session-{timestamp}-{counter}.json`
- **元数据记录** (第1行): `{"type":"session_meta","version":N,"session_id":"...","created_at_ms":...,"updated_at_ms":...,...}`
- **消息记录**: `{"type":"message","message":{...}}`
- **Compaction 记录**: `{"type":"compaction_meta",...}`
- **原子写入**: `rotate_session_file_if_needed` + `write_atomic`

### 3.2 我们的 Session 存储

- **格式**: 直接使用 `Session::save_to_path` / `Session::load_from_path`（即 runtime 的 JSONL 实现）
- **路径**: `~/.claw/sessions/`
- **文件名**: `{uuid}.json`（我们用 UUID 而非 `session-{timestamp}-{counter}`）

### 3.3 JSONL 格式兼容性

我们直接调用 `Session::save_to_path()` 和 `Session::load_from_path()`，这是 runtime 提供的 API，JSONL 格式完全一致。**格式兼容**。

`load_from_path` 同时支持 JSON 和 JSONL 两种格式读取（先尝试 JSON，再尝试 JSONL），这是正确的。

### 3.4 `list_sessions_info` 返回字段

```rust
// claw_chat.rs:62-144
sessions.push(serde_json::json!({
    "sessionId": file_stem,          // ✅ 用文件名 stem
    "createdAt": format_ts(created_at_ms),
    "messageCount": message_count,
    "title": title,
}));
```

**缺失字段：**

| 字段 | 是否返回 | 说明 |
|------|----------|------|
| `sessionId` | ✅ | 使用文件名 stem（正确） |
| `createdAt` | ✅ | RFC 3339 格式 |
| `messageCount` | ✅ | 从 JSONL 行数计算 |
| `title` | ✅ | 从第二行（第一条消息）提取 |
| `updatedAt` | ❌ **缺失** | 上游 meta 记录中有 `updated_at_ms`，但我们的 `list_sessions_info` 读取了却丢弃了（`let _updated_at_ms = ...`）。 |
| `model` | ❌ **缺失** | 上游 meta 中可能有 `model` 字段。 |
| `session_id` (内部) | ❌ **缺失** | 文件名 stem 可能与内部 `session_id` 不同（上游支持 fork，子 session 有不同 ID）。 |

### 3.5 Session ID 生成差异

| 项目 | 上游 | 我们 |
|------|------|------|
| 格式 | `session-{timestamp_ms}-{counter}` | UUID v4 |
| 信息含量 | 从 ID 可解析创建时间 | 无时间信息 |

我们的 UUID 格式与上游完全不同，导致两个问题：
1. **无法从 ID 推断创建时间**（上游 `parse_created_at_ms_from_session_id` 依赖 `session-` 前缀）
2. **如果文件名与内部 session_id 不一致**（如 fork 场景），`list_sessions_info` 用文件名 stem 作为 ID 是正确的，但 `load_session` 也是用这个 ID 拼路径，所以能匹配。

### 3.6 `session_messages_to_json` 角色映射

```rust
MessageRole::User => "user",
MessageRole::Assistant => "agent",   // ← 注意：不是 "assistant"
MessageRole::System => "system",
MessageRole::Tool => "tool",
```

上游使用 `"assistant"` 作为角色名。我们用 `"agent"`。如果前端直接使用这个角色名，可能产生兼容性问题。但这只是前端显示用，不影响核心逻辑。

---

## 4. Hook / Permission 系统

### 4.1 上游 Hook 系统

上游 `conversation.rs` 中的 hook 调用非常精细：

```rust
// conversation.rs:228-297
fn run_pre_tool_use_hook(&mut self, tool_name: &str, input: &str) -> HookRunResult {
    // 使用 self.hook_abort_signal（可取消长时间运行的 hook）
    // 使用 self.hook_progress_reporter（向 UI 报告 hook 进度）
    // 从 HookRunner::from_feature_config(feature_config) 初始化
}
```

上游 `HookRunner` 初始化来源：
- `HookRunner::from_feature_config(feature_config)` — 从 RuntimeFeatureConfig 读取配置
- 配置来源是上游的 config 系统（`~/.claw/settings.json` 中的 hooks 字段）

上游 hook 流程（conversation.rs:411-491）：
1. `run_pre_tool_use_hook` → 检查 `is_cancelled`, `is_failed`, `is_denied`
2. 如果 hook 拒绝 → 返回 `PermissionOutcome::Deny`
3. 如果 hook 提供 `permission_override` → 传入 `PermissionContext`
4. 执行工具 → `run_post_tool_use_hook` / `run_post_tool_use_failure_hook`
5. 如果 post-hook 拒绝 → `is_error = true`（标记工具输出为错误）

### 4.2 我们的 Hook 调用

```rust
// claw_chat.rs:1037-1089
// Pre-tool hook
let hook_result = runtime::HookRunner::new(runtime::RuntimeHookConfig::default())
    .run_pre_tool_use(tool_name, &tool_input.to_string());
if hook_result.is_denied() { ... continue; }

// 工具执行...

// Post-tool hook
if is_error {
    runtime::HookRunner::new(runtime::RuntimeHookConfig::default())
        .run_post_tool_use_failure(tool_name, &tool_input.to_string(), &output);
} else {
    runtime::HookRunner::new(runtime::RuntimeHookConfig::default())
        .run_post_tool_use(tool_name, &tool_input.to_string(), &output, is_error);
}
```

### 4.3 Hook 差异

| 项目 | 上游 | 我们 | 风险 |
|------|------|------|------|
| HookRunner 来源 | `from_feature_config` — 从配置文件读取 hooks 规则 | `RuntimeHookConfig::default()` — **空配置** | 🔴 **严重：我们的 hook 配置永远为空**。`RuntimeHookConfig::default()` 返回的 pre_tool_use / post_tool_use 列表都是空的 `Vec`。这意味着 **用户的 hook 配置完全不生效**。 |
| HookRunner 实例化 | 上游复用同一个实例 | 每次工具调用创建新实例 | ⚠️ 性能开销（每次创建新的 HookRunner），但功能上不影响（因为配置是空的）。 |
| abort_signal | ✅ 传入 `HookAbortSignal`，支持取消长时间 hook | ❌ 不传入，使用 `None` | ⚠️ 如果用户配置了长时间运行的 hook，无法被取消（但目前 hook 配置为空，所以暂时无影响）。 |
| progress_reporter | ✅ 传入 reporter 向 UI 报告 hook 进度 | ❌ 不传入 | ⚡ 低风险：GUI 模式下可接受。 |
| pre-hook 取消/失败处理 | ✅ 处理 `is_cancelled`, `is_failed`, `is_denied` 三种状态 | ⚠️ 只处理 `is_denied` | ⚠️ 上游的 `is_cancelled` 和 `is_failed` 场景我们没有覆盖。 |
| post-hook 拒绝处理 | ✅ post-hook 拒绝会将 output 标记为 error | ❌ 我们调用了 post-hook 但**忽略了返回值** | ⚠️ post-hook 的拒绝/取消信号被丢弃。 |
| permission_override 传递 | ✅ hook 可以提供 `PermissionOverride`（Allow/Deny/Ask） | ❌ 完全不使用 | 🔴 Hook 无法影响权限决策。 |

### 4.4 Permission 系统

#### 上游

```rust
// lib.rs:4304-4309
fn agent_permission_policy() -> PermissionPolicy {
    mvp_tool_specs().into_iter().fold(
        PermissionPolicy::new(PermissionMode::DangerFullAccess),
        |policy, spec| policy.with_tool_requirement(spec.name, spec.required_permission),
    )
}
```

上游 `agent_permission_policy` 使用 `DangerFullAccess` 模式，加上每个工具的 `required_permission`：
- `bash` → `DangerFullAccess`
- `read_file` → `ReadOnly`
- `write_file` → `WorkspaceWrite`
- `edit_file` → `WorkspaceWrite`
- `glob_search` → `ReadOnly`
- 等等...

`PermissionMode` 枚举（permissions.rs:8-15）：
```
ReadOnly < WorkspaceWrite < DangerFullAccess < Prompt < Allow
```

`authorize_with_context` 逻辑（permissions.rs:181-298）：
1. 检查 `denied_tools` 列表 → 直接拒绝
2. 检查 deny_rules → 直接拒绝
3. 检查 hook `PermissionOverride` → Deny/Ask/Allow
4. 检查 ask_rules → 需要用户确认
5. 检查 allow_rules 或 `current_mode >= required_mode` → 允许
6. 如果 `current_mode == Prompt` 或 `WorkspaceWrite` + 需要 `DangerFullAccess` → prompt 用户
7. 否则 → 拒绝

**关键点**：`Allow` 模式是最高级别权限（`Allow` >= `Prompt` >= `DangerFullAccess` >= ...），它跳过所有权限检查，**直接允许所有工具调用**。

#### 我们

```rust
// claw_chat.rs:271-273
let enforcer = runtime::permission_enforcer::PermissionEnforcer::new(
    runtime::PermissionPolicy::new(runtime::PermissionMode::Allow),
);
registry.set_enforcer(enforcer);
```

我们使用 `PermissionMode::Allow` 作为全局权限模式。

### 4.5 Permission 差异

| 项目 | 上游 | 我们 | 影响 |
|------|------|------|------|
| 全局模式 | `DangerFullAccess` | `Allow` | ⚠️ 见下方分析 |
| Tool requirements | ✅ 每个工具设置不同的 required_permission | ❌ 不设置 tool_requirements | 🔴 我们没有任何 per-tool 权限要求 |
| deny_rules | ✅ 从配置读取 | ❌ 无 | ⚠️ 用户无法拒绝特定工具调用 |
| allow_rules | ✅ 从配置读取 | ❌ 无 | — |
| ask_rules | ✅ 从配置读取 | ❌ 无 | — |
| denied_tools | ✅ 无条件拒绝列表 | ❌ 无 | — |
| 用户交互确认 | ✅ 无 Prompt 模式可弹出权限确认 | ❌ 永远自动允许 | ⚠️ GUI 应用中缺少权限确认机制 |

#### `DangerFullAccess` vs `Allow` 深度分析

两者在行为上的**关键区别**：

**`DangerFullAccess`**：满足 `current_mode >= required_mode` 的条件（因为 `DangerFullAccess >= ReadOnly/WorkspaceWrite/DangerFullAccess`），对大部分工具自动允许。但如果某些工具的 `required_permission` 被配置为 `Prompt`（比 `DangerFullAccess` 更高），仍会触发提示。

**`Allow`**：在 `authorize_with_context` 中，`current_mode == PermissionMode::Allow` 会在第 250 行和第 275 行被检查，无条件返回 `PermissionOutcome::Allow`。**它跳过所有权限检查**，甚至包括 `denied_tools` 之外的 deny_rules（不对，deny_rules 在 Allow 检查之前就已经被评估了）。

实际上仔细看代码：
1. `denied_tools` 检查（191行） → 无条件拒绝，**即使 Allow 模式也会被拒绝** ✅
2. `deny_rules` 检查（197行） → 无条件拒绝，**即使 Allow 模式也会被拒绝** ✅
3. Hook `PermissionOverride` → 在 Allow 模式下，`Allow` override 会走到 `current_mode == Allow` → 允许
4. ask_rules 检查 → 在 Allow 模式下，走到 `current_mode == Allow` 分支 → 允许
5. `current_mode >= required_mode` → `Allow` >= 任何 → 允许

所以 `Allow` 模式的效果是：**除了 `denied_tools` 和 `deny_rules` 之外，所有工具调用都会被自动允许**。

这比上游的 `DangerFullAccess` 更宽松。上游的 `DangerFullAccess` + tool requirements 意味着：
- 如果用户在 config 中配置了 `ask_rules`，bash 等工具仍会弹出确认
- 如果 hook 返回 `PermissionOverride::Ask`，也会弹出确认

而我们的 `Allow` 模式**忽略了 tool requirements**，因为 `Allow` 在权限检查链中更早被匹配（第 275 行的 `current_mode == PermissionMode::Allow`）。

**实际影响**：由于我们没有设置 tool_requirements，也没有 deny/ask rules，这个差异在当前实现中不影响行为。但如果未来添加权限规则，必须切换到 `DangerFullAccess`。

---

## 5. 发现汇总

### 🔴 严重问题 (需要修复)

| # | 问题 | 位置 | 影响 |
|---|------|------|------|
| S1 | **Hook 配置始终为空** — 每次创建 `RuntimeHookConfig::default()`，不从 config 读取 | `claw_chat.rs:1038,1084,1087` | 用户在 `~/.claw/settings.json` 中配置的 hooks 完全不生效 |
| S2 | **pre-hook 只处理 `is_denied`** — 忽略了 `is_cancelled` 和 `is_failed` | `claw_chat.rs:1040` | 上游的 hook 取消和失败信号被丢弃 |
| S3 | **post-hook 返回值被忽略** — `run_post_tool_use` 的拒绝信号不生效 | `claw_chat.rs:1083-1089` | 上游 post-hook 可以将工具输出标记为错误，我们无法做到 |

### ⚠️ 中等问题 (建议修复)

| # | 问题 | 位置 | 影响 |
|---|------|------|------|
| M1 | **Config 路径不一致** — 我们用 `config.json`，上游用 `settings.json` | `claw_config.rs:83-87` | 与上游 CLI 不兼容，无法共享配置 |
| M2 | **不支持 CLAW_CONFIG_HOME** — 上游支持环境变量覆盖 config 目录 | `claw_config.rs:83-87` | 在特殊环境下无法自定义配置路径 |
| M3 | **PermissionMode::Allow 过于宽松** — 缺少 per-tool 权限检查 | `claw_chat.rs:271-273` | 安全审计：所有工具调用都无条件允许（当前无 deny_rules 所以行为一致，但架构上不够健壮） |
| M4 | **list_sessions_info 缺少 updatedAt 和 model** | `claw_chat.rs:62-144` | 前端无法显示会话最后更新时间和使用的模型 |
| M5 | **load_hermes_skills Unicode 截断可能 panic** | `claw_chat.rs:381-385` | `&brief[..200]` 字节截断可能在多字节字符处 panic |
| M6 | **load_hermes_skills 的 break 只跳出内层循环** | `claw_chat.rs:427-430` | byte cap 达到时只跳过当前类别剩余 skill，不跳过后续类别 |

### ⚡ 低风险 / 设计差异 (可接受)

| # | 问题 | 说明 |
|---|------|------|
| L1 | Session ID 用 UUID 而非 `session-{ts}-{counter}` | 功能兼容，但无法从 ID 推断时间 |
| L2 | `session_messages_to_json` 用 `"agent"` 而非 `"assistant"` | 仅影响前端显示 |
| L3 | 不支持项目级配置 `.claw/settings.local.json` | 简化了配置层级 |
| L4 | `unsafe { std::env::set_var }` 在多线程环境下 | 实际只在初始化阶段调用，风险可控 |
| L5 | System prompt 使用真实日期（上游用固定值） | 实际上是改进 |

---

## 6. 修复建议优先级

### P0 (必须修复)

1. **Hook 配置集成**：`HookRunner` 应从 config 读取 hooks 配置（`RuntimeHookConfig::from_feature_config` 或手动解析 `~/.claw/settings.json` 中的 hooks 字段）
2. **pre-hook 完整处理**：同时处理 `is_cancelled`, `is_failed`, `is_denied` 三种状态
3. **post-hook 返回值处理**：将 post-hook 的拒绝信号传递给后续逻辑（标记 output 为 error）

### P1 (建议修复)

4. 考虑将 `PermissionMode::Allow` 改为 `PermissionMode::DangerFullAccess` + tool requirements，为未来的权限控制做准备
5. `list_sessions_info` 补充 `updatedAt` 和 `model` 字段
6. 修复 `load_hermes_skills` 中的 Unicode 截断和 break 语义

### P2 (长期改进)

7. Config 路径与上游对齐（或至少支持 `CLAW_CONFIG_HOME`）
8. 考虑支持项目级配置
9. HookRunner 复用（避免每次工具调用创建新实例）
