# 审计报告：claw_chat_send 工具循环 vs 上游 ConversationRuntime::run_turn

**日期**：2026-06-05
**审查范围**：上游 `claw-runtime/src/conversation.rs` `run_turn` 方法 (line 318-524)
vs 我们 `tauri/src/commands/claw_chat.rs` `claw_chat_send` 函数 (line 814-1147)

---

## 总结

我们的 `claw_chat_send` 在工具循环的整体结构上与上游保持一致（loop → API 调用 → 工具执行 → 推回 session），但**缺失了上游的多个关键子系统**，包括权限策略、hook 结果处理、session 健康探针、usage 追踪等。这些差异按严重度分类如下：

| 严重度 | 数量 |
|--------|------|
| CRITICAL | 2 |
| MAJOR | 6 |
| MINOR | 7 |

---

## 差异详情

### 🔴 CRITICAL — 权限策略完全缺失

**上游行为** (conversation.rs:421-456)：

```rust
let permission_outcome = if pre_hook_result.is_cancelled() {
    PermissionOutcome::Deny { ... }
} else if pre_hook_result.is_failed() {
    PermissionOutcome::Deny { ... }
} else if pre_hook_result.is_denied() {
    PermissionOutcome::Deny { ... }
} else if let Some(prompt) = prompter.as_mut() {
    self.permission_policy.authorize_with_context(
        &tool_name, &effective_input, &permission_context, Some(*prompt),
    )
} else {
    self.permission_policy.authorize_with_context(
        &tool_name, &effective_input, &permission_context, None,
    )
};
```

上游的 `permission_policy.authorize_with_context()` 实现了完整的权限层级：
- `denied_tools` 白名单拒绝
- `deny_rules` / `ask_rules` / `allow_rules` 正则规则匹配
- 全局 `active_mode`（Allow / ReadOnly / WorkspaceWrite / Prompt / DangerFullAccess）
- hook 覆盖的 `PermissionOverride`（Allow / Ask / Deny）
- 需要时通过 `PermissionPrompter` 弹出用户确认

**我们的实现** (claw_chat.rs:1038-1052)：

```rust
let hook_result = runtime::HookRunner::new(runtime::RuntimeHookConfig::default())
    .run_pre_tool_use(tool_name, &tool_input.to_string());
if hook_result.is_denied() {
    // 仅拒绝，跳过工具执行
    continue;
}
// 直接执行工具，无权限策略检查
```

**影响**：我们的 Tauri GUI 会在**没有任何权限检查**的情况下执行任何工具。没有 denied_tools、没有 deny_rules、没有全局 permission mode、没有用户确认机制。虽然 Tauri GUI 场景下用户直接交互可能降低风险，但这是一个**安全架构缺口**。

**建议**：引入 `PermissionPolicy`，至少配置 `Allow` 或 `WorkspaceWrite` 模式，并保留 deny_rules 能力。

---

### 🔴 CRITICAL — Pre-tool hook 结果处理严重不完整

**上游行为** (conversation.rs:411-456)：

上游处理 pre-hook 结果的多个维度：

1. **三态拒绝**：`is_cancelled()`、`is_failed()`、`is_denied()` 三种情况分别处理，都生成包含具体原因的 Deny
2. **input 修改**：`pre_hook_result.updated_input()` 可以替换工具的输入参数（如安全清洗 bash 命令）
3. **权限覆盖**：`pre_hook_result.permission_override()` 和 `permission_reason()` 传递给权限策略
4. **反馈合并**：拒绝时通过 `merge_hook_feedback(pre_hook_result.messages(), reason, true)` 将 hook 消息合并到 tool result 中

**我们的实现** (claw_chat.rs:1038-1052)：

1. 仅检查 `is_denied()`，**忽略了 `is_cancelled()` 和 `is_failed()`**
2. **不读取** `updated_input()`，直接使用原始 tool_input
3. **不传递** `permission_override` / `permission_reason`
4. **不合并** hook 消息到 tool result 中

**影响**：
- hook 可以通过 `cancelled`/`failed` 状态拒绝工具，但我们不会捕获这些状态
- hook 修改的 input（如安全清理）会被忽略，可能执行了危险操作
- hook 的反馈信息对 LLM 不可见，模型无法理解被拒绝的原因

**建议**：至少补充三态拒绝检查和 updated_input 应用。

---

### 🟠 MAJOR — Post-tool hook 结果被完全忽略

**上游行为** (conversation.rs:468-494)：

```rust
let post_hook_result = if is_error {
    self.run_post_tool_use_failure_hook(&tool_name, &effective_input, &output)
} else {
    self.run_post_tool_use_hook(&tool_name, &effective_input, &output, false)
};
if post_hook_result.is_denied()
    || post_hook_result.is_failed()
    || post_hook_result.is_cancelled()
{
    is_error = true;  // ← hook 可以将成功的工具结果标记为错误
}
output = merge_hook_feedback(
    post_hook_result.messages(), output,
    post_hook_result.is_denied() || post_hook_result.is_failed() || post_hook_result.is_cancelled(),
);
```

上游的 post-hook 可以：
- 将成功执行的结果**翻转为错误**（`is_error = true`）
- 将反馈消息**合并到输出**中

**我们的实现** (claw_chat.rs:1082-1089)：

```rust
if is_error {
    let _ = runtime::HookRunner::new(runtime::RuntimeHookConfig::default())
        .run_post_tool_use_failure(tool_name, &tool_input.to_string(), &output);
} else {
    let _ = runtime::HookRunner::new(runtime::RuntimeHookConfig::default())
        .run_post_tool_use(tool_name, &tool_input.to_string(), &output, is_error);
}
// return value is discarded — `let _ =`
```

**影响**：post-hook 的任何反馈、拒绝、错误标记能力完全失效。即使 hook 检测到危险输出，LLM 也不会收到任何提示。

---

### 🟠 MAJOR — HookRunner 每次调用都重新创建

**上游行为**：`self.hook_runner` 在 `ConversationRuntime` 构造时初始化一次，使用 `HookRunner::from_feature_config(feature_config)` 从 FeatureConfig 加载 hook 配置（包括用户自定义的 hook 脚本）。

**我们的实现** (claw_chat.rs:1038, 1084, 1087)：

```rust
runtime::HookRunner::new(runtime::RuntimeHookConfig::default())  // 每次创建新实例
```

每个 hook 调用都创建一个新的 `HookRunner`，使用默认配置。

**影响**：
- 用户自定义 hook（如 pre-tool / post-tool 脚本）不会被加载
- 每次 hook 调用的配置都是 `RuntimeHookConfig::default()`，不是用户配置
- 性能开销（重复构造）

**建议**：在 `ClawChatState` 中缓存一个 `HookRunner` 实例，初始化时从用户配置加载。

---

### 🟠 MAJOR — Auto-compaction 时机和机制差异

**上游行为** (conversation.rs:401-405, 564-587)：

```rust
// 在推送 assistant message 之后、检查 pending_tool_uses 之前
if let Some(compaction) = self.maybe_auto_compact() {
    auto_compaction = Some(compaction);
}
```

上游的 auto-compaction：
1. 在**每个循环迭代**都检查（包括无工具调用的终止迭代）
2. 基于 **token 计数**（`usage_tracker.cumulative_usage().input_tokens` > 阈值）
3. 使用 `CompactionConfig { max_estimated_tokens: 0, ..default() }`（激进压缩）
4. 压缩后更新 `self.session`，但**不立即持久化**（持久化由外部控制）

**我们的实现** (claw_chat.rs:878-896)：

```rust
// 在循环迭代开头、API 调用之前
if auto_compaction {
    if runtime::should_compact(sess, CompactionConfig::default()) {
        // 基于消息数量 + token 估计
        let result = runtime::compact_session(sess, CompactionConfig::default());
        // ...
    }
}
```

我们的差异：
1. 仅在**循环开头**检查（API 调用前），不在终端迭代后检查
2. 基于 **消息数 + token 估计**（`should_compact` 检查消息数 > preserve_recent_messages 且 token >= max_estimated_tokens）
3. 使用 `CompactionConfig::default()`（preserve_recent_messages=4, max_estimated_tokens=10000）
4. 压缩后**立即持久化**到磁盘

**影响**：压缩时机不同可能导致在 terminal iteration 后 session 膨胀，而我们的默认阈值（10000 tokens）可能与上游的 token-based 阈值不一致。

---

### 🟠 MAJOR — Session 健康探针缺失

**上游行为** (conversation.rs:326-334)：

```rust
if self.session.compaction.is_some() {
    if let Err(error) = self.run_session_health_probe() {
        return Err(RuntimeError::new(format!(
            "Session health probe failed after compaction: {error}. \
             The session may be in an inconsistent state. \
             Consider starting a fresh session with /session new."
        )));
    }
}
```

上游在 turn 开始时，如果 session 已经被压缩过，会执行一个**健康探针**（用 `glob_search` 工具做一个非破坏性探测），确保 runtime 仍然正常工作。

**我们的实现**：完全没有健康探针逻辑。

**影响**：压缩后的 session 如果处于不一致状态，我们会继续执行，可能导致不可预测的行为。

---

### 🟠 MAJOR — 工具输出截断是我们的独有逻辑，上游无此机制

**上游行为**：run_turn 中没有工具输出截断。依赖 auto-compaction 来管理 context 大小。

**我们的实现** (claw_chat.rs:1092-1100)：

```rust
let truncated_output = if output.len() > tool_output_truncation {
    format!("{}...\n\n[Output truncated — was {} chars total]",
        &output[..tool_output_truncation], output.len())
} else {
    output
};
```

**影响**：这是一个**有意的设计决策**，在 GUI 场景下合理（防止巨大输出冲爆 context）。但需要注意到：
1. 上游的 tool output 中可能包含关键的错误诊断信息，截断后 LLM 可能无法正确理解错误
2. 截断发生在 `tool_output_truncation` 字符处，可能导致 UTF-8 字符截断（对中文等多字节字符尤其危险）
3. 没有考虑代码块边界——可能在代码中间截断

**建议**：至少按行边界截断，避免 UTF-8 截断。

---

### 🟠 MAJOR — Usage 追踪完全缺失

**上游行为** (conversation.rs:376-378, 518)：

```rust
if let Some(usage) = usage {
    self.usage_tracker.record(usage);
}
// ...
TurnSummary { usage: self.usage_tracker.cumulative_usage(), ... }
```

上游追踪每个 API 调用的 token 使用量，构建累积 usage，并通过 TurnSummary 返回。

**我们的实现**：没有 usage 追踪。不记录 token 消耗，不返回 usage 信息。

**影响**：GUI 无法向用户展示 token 使用统计。对于计费敏感的模型（如 Claude Opus），用户无法了解消耗情况。

---

### 🟡 MINOR — 上游的 record_* 追踪/事件上报缺失

上游在关键节点调用：
- `record_turn_started(user_input)` — turn 开始
- `record_assistant_iteration(iterations, message, tool_count)` — 每次 LLM 响应
- `record_tool_started(iterations, tool_name)` — 工具执行开始
- `record_tool_finished(iterations, result_message)` — 工具执行完成
- `record_turn_completed(summary)` — turn 成功完成
- `record_turn_failed(iterations, error)` — turn 失败

这些通过 `SessionTracer` 实现 OpenTelemetry 风格的追踪。我们的实现用 `log::info!` 替代，结构化程度较低。

---

### 🟡 MINOR — Hook Abort Signal 缺失

**上游行为**：通过 `HookAbortSignal`（基于 `AtomicBool`）允许在取消对话时中止正在执行的 hook。

**我们的实现**：没有 abort signal 机制。如果用户在 hook 执行期间取消操作，hook 会继续运行直到完成。

---

### 🟡 MINOR — Hook Progress Reporter 缺失

**上游行为**：通过 `HookProgressReporter` trait 报告 hook 执行进度（如 "正在执行 pre-tool hook..."）。

**我们的实现**：没有进度报告机制。长时间运行的 hook 会让用户感到界面无响应。

---

### 🟡 MINOR — Prompt Cache Events 缺失

上游收集 `prompt_cache_events`（缓存命中/未命中事件），用于监控和优化 prompt 缓存。我们的实现不追踪这些事件。

---

### 🟡 MINOR — TurnSummary 返回值缺失

上游返回完整的 `TurnSummary`（含 assistant_messages、tool_results、prompt_cache_events、iterations、usage、auto_compaction），调用方可以据此做进一步处理。我们的 `claw_chat_send` 返回 `Result<(), String>`，没有任何 turn 摘要。

---

### 🟡 MINOR — 工具执行路径差异

**上游**：通过 `self.tool_executor.execute(&tool_name, &effective_input)` → `StaticToolExecutor` → HashMap handler dispatch

**我们的实现**：通过 `tools::execute_tool(&tn, &ti)` → `execute_tool_with_enforcer(None, name, input)` → 大 match 分发

虽然最终效果相同（都调用相同的底层工具函数），但上游使用 trait-based 的抽象，允许测试时 mock 工具执行器。我们的实现直接耦合到具体实现。

注意：`execute_tool` 内部虽然有 `enforcer` 参数，但我们传 `None`，所以内部的 `maybe_enforce_permission_check_with_mode` 会直接返回 `Ok(())`。这与上游一致——上游的权限检查在 conversation runtime 层面完成，而不是在工具执行器内部。

---

### 🟡 MINOR — LLM 超时和 context 溢出恢复是我们独有逻辑

**我们的实现**：有 120 秒 LLM 超时保护 + 4 轮渐进式 compaction（preserve 4→2→1→0）。

**上游**：在 run_turn 层面没有这些逻辑。可能在更上层（CLI 的 main loop 或 API client）处理。

**影响**：这是合理的增强，但属于我们独有的容错逻辑，上游没有对应的对比基准。

---

## 架构对比图

```
上游 run_turn 流程：
┌─────────────────────────────────────────────────┐
│ 1. session health probe (if compacted)          │
│ 2. record_turn_started                          │
│ 3. push_user_text                               │
│ 4. loop:                                        │
│    a. iterations++ / max check                  │
│    b. API call → assistant message              │
│    c. record_assistant_iteration                │
│    d. push assistant_message                    │
│    e. maybe_auto_compact (ALL iterations)       │
│    f. if no tool_uses → break                   │
│    g. for each tool_use:                        │
│       i.   run_pre_tool_use_hook                │
│       ii.  permission_policy.authorize          │  ← 我们缺失
│       iii. apply updated_input                  │  ← 我们缺失
│       iv.  tool_executor.execute                │
│       v.   merge_hook_feedback (pre)            │  ← 我们缺失
│       vi.  run_post_tool_use_hook               │
│       vii. check post-hook deny → is_error      │  ← 我们缺失
│       viii.merge_hook_feedback (post)           │  ← 我们缺失
│       ix.  push tool_result                     │
│       x.   record_tool_finished                 │
│ 5. record_turn_completed                        │
│ 6. return TurnSummary                           │
└─────────────────────────────────────────────────┘

我们的 claw_chat_send 流程：
┌─────────────────────────────────────────────────┐
│ 1. push_user_text + persist                     │
│ 2. for iteration in 0..max:                     │
│    a. auto_compact (beginning only)             │  ← 时机不同
│    b. API call → turn_result                    │
│    c. push assistant_msg                        │
│    d. if no tool_calls → break                  │
│    e. for each tool_call:                       │
│       i.   set workspace dir                    │  ← 我们独有
│       ii.  run_pre_tool_use                     │
│       iii. check is_denied ONLY                 │  ← 不完整
│       iv.  (no permission policy)               │  ← 缺失
│       v.   spawn_blocking → execute_tool        │
│       vi.  run_post_hook (result ignored!)      │  ← 结果丢弃
│       vii. truncate output                      │  ← 我们独有
│       viii.push tool_result                     │
│ 3. emit agent-done                              │  ← 我们独有
│ 4. persist final session                        │
└─────────────────────────────────────────────────┘
```

---

## 修复优先级建议

| 优先级 | 差异项 | 工作量 |
|--------|--------|--------|
| P0 | 补充 pre-hook 三态拒绝 + updated_input | 小 |
| P0 | 处理 post-hook 返回结果（is_error 翻转 + feedback 合并） | 小 |
| P1 | 引入基础 PermissionPolicy | 中 |
| P1 | 缓存 HookRunner 实例（从用户配置初始化） | 小 |
| P2 | 补充 session 健康探针 | 小 |
| P2 | 增加 usage 追踪 | 中 |
| P3 | 修复 UTF-8 截断风险 | 小 |
| P3 | 统一 auto-compaction 时机 | 小 |
| P3 | 补充 hook abort signal / progress reporter | 中 |

---

*审计人：Hermes Agent*
