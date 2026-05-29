# Agent 压缩会话消息误标为子Agent卡片 — 排查任务

## 问题现象

在 SuperTool 的 Agent 对话框（`HermesChat.vue`）中，查看经过 Hermes 压缩/重置（`end_reason IN ('session_reset', 'compression')`）的会话历史时，来自 continuation child 会话的消息被错误地渲染成「子 Agent」折叠卡片（`ChildSessionGroup` 组件），而不是作为普通消息内联显示。

## 排查路径

### 1. 确认数据模型

Hermes state.db（`~/.hermes/state.db`）的 session 层级关系：

```
sessions:
  id TEXT PRIMARY KEY
  parent_session_id TEXT   → 指向父会话
  end_reason TEXT          → 'session_reset' | 'compression' | NULL
  started_at REAL          → unix timestamp
  ended_at REAL            → unix timestamp
```

两种父子关系：
- **continuation child**：Hermes 压缩/重置后创建的新会话，继续原对话。父会话的 `end_reason` 为 `session_reset` 或 `compression`。
- **subagent child**：`delegate_task` 工具创建的子 Agent 会话。父会话的 `end_reason` 为 NULL（仍在活跃状态）。

### 2. 检查 `get_compression_tip`（`core/src/db/agent.rs:357`）

作用：从任意 session ID 沿着压缩链走到最新的 continuation child。

```sql
-- 当前 SQL
SELECT s2.id FROM sessions s2
JOIN sessions s1 ON s2.parent_session_id = s1.id
WHERE s1.id = ?
  AND s1.end_reason IN ('session_reset', 'compression')
ORDER BY s2.started_at DESC LIMIT 1
```

**待验证**：用一个已知的 session_reset 根会话做测试，比如：

```sql
-- 找一个有 continuation child 的根会话
SELECT s1.id, s1.end_reason, 
       datetime(s1.ended_at, 'unixepoch', 'localtime') as parent_ended,
       s2.id as child_id, 
       datetime(s2.started_at, 'unixepoch', 'localtime') as child_started
FROM sessions s1
JOIN sessions s2 ON s2.parent_session_id = s1.id
WHERE s1.parent_session_id IS NULL 
  AND s1.end_reason IS NOT NULL
  AND s1.message_count > 50
ORDER BY s1.started_at DESC;
```

确认返回的 `child_id` 是期望的 continuation child。

### 3. 检查 `list_hermes_messages` 的 `is_child` SQL（`core/src/db/agent.rs:612`）

当前 SQL：

```sql
CASE WHEN m.session_id != ?
  AND sessions.parent_session_id = ?
  AND NOT EXISTS (
    SELECT 1 FROM sessions p
    WHERE p.id = ?
      AND p.end_reason IN ('session_reset', 'compression')
  )
  THEN 1 ELSE 0 END as is_child
```

三个 `?` 参数绑定：`[&effective_session_id; 3]`（注意：用的是压缩 tip 的 session ID，不是原始参数）

**待验证**：对一个有 continuation child 的根会话，模拟 SQL 查询 messages 的 `is_child` 值：

```sql
-- 用实际 session ID 替换 TIP_ID、ROOT_ID、CHILD_ID
WITH tip AS (SELECT 'TIP_ID' AS tid)
SELECT 
  m.session_id,
  s.parent_session_id,
  CASE 
    WHEN m.session_id != tip.tid 
         AND s.parent_session_id = tip.tid
         AND NOT EXISTS (
           SELECT 1 FROM sessions p 
           WHERE p.id = tip.tid 
             AND p.end_reason IN ('session_reset', 'compression')
         )
    THEN 1 ELSE 0 
  END as is_child,
  m.role,
  COUNT(*) as cnt
FROM messages m
LEFT JOIN sessions s ON s.id = m.session_id
CROSS JOIN tip
WHERE m.session_id IN ('TIP_ID', 'ROOT_ID', 'CHILD_ID')
GROUP BY m.session_id, is_child, m.role
ORDER BY m.session_id;
```

期望结果：
- root 消息: `is_child = 0`
- tip 自身消息: `is_child = 0`
- continuation child 消息: `is_child = 0`
- 真正的 subagent child 消息: `is_child = 1`

### 4. 检查前端 `displayItems` 计算属性（`HermesChat.vue:611`）

确认前端分组逻辑只依赖 `msg.isChild && msg.sessionId`：

```typescript
// 分组条件
if (msg.isChild && msg.sessionId) {
  // → 加入 ChildSessionGroup
}

// 跳过条件
if (msg.isChild) {continue;}
// → 不单独显示
```

如果后端的 `is_child` 正确为 0，前端不应该渲染成子Agent卡片。

### 5. 检查流式事件处理（`useStreamingHandler.ts`）

在 `handleDelta`、`handleToolStart`、`handleToolComplete`、`handleError` 中：

```typescript
if (eventSid !== currentSessionId.value) {
  // 非当前会话 → 标记 isChild: true
  const syncedMsgs = sessionMsgs.map(m => ({
    ...m,
    isChild: true,
    sessionId: eventSid,
  }));
}
```

**待验证**：Hermes 在压缩/重置后，是否会在同一次 SSE 流中改变 `session_id`。如果会，则这些事件会在前端被标记为 child 消息。查看 `hermes_chat.rs` 中 `captured_session_id` 的取值逻辑：

```rust
// captured_session_id = request参数的session_id ?? 响应头X-Hermes-Session-Id
let captured_session_id = session_id.or(response_session_id);
```

如果 Hermes 响应头返回了新的 session_id 但请求参数有旧值，`captured_session_id` 会取旧值。整个 SSE 流中 session_id 不变。但**跨轮次**（下一次对话）时，前端 `HermesChat.vue:748-753` 会更新 `currentSessionId`。

### 6. 检查 `selectSession` 的压缩 tip 解析（`useSessionManager.ts:89-108`）

```typescript
let effectiveSessionId = session.id;
try {
  const tipResult = await invoke('agent_get_compression_tip', { sessionId: session.id });
  if (tipResult.success && tipResult.tipSessionId !== session.id) {
    effectiveSessionId = tipResult.tipSessionId;
    session = { ...session, id: effectiveSessionId };
  }
} catch (e) {
  // Silently ignore
}
```

如果 `get_compression_tip` 正确返回了 tip，`agent_list_messages` 会用 tip 的 session ID 查询，前述 `is_child` SQL 会正确判断。

## 已发现的问题与修复

| # | 问题 | 位置 | 修复 |
|---|------|------|------|
| 1 | `end_reason` 只检查 `'compression'`，漏了 `'session_reset'` | `agent.rs:380,272` | 改为 `IN ('session_reset', 'compression')` |
| 2 | `s2.started_at >= s1.ended_at` 时间检查排除了 session_reset 的 continuation child（其 started_at 早于 parent 的 ended_at） | `agent.rs:381,273` | 移除该条件 |
| 3 | `is_child` SQL 标记了所有 continuation child 为 `is_child=1` | `agent.rs:612` | 添加 `NOT EXISTS` 子查询排除 |
| 4 | SQL 参数绑定了 `session_id`（原始参数）而非 `effective_session_id` | `agent.rs:626` | 改为 `effective_session_id` |

## 验证清单

- [ ] `get_compression_tip` 对 session_reset 根会话返回正确的 tip
- [ ] `list_hermes_messages` 查询中，continuation child 的消息 `is_child = 0`，subagent 消息 `is_child = 1`
- [ ] APP 重新打包后效果正常（Rust 代码变更需要 `pnpm build:app` 或 `pnpm tauri build` 编译）
