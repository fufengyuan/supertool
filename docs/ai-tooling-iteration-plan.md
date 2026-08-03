# SuperTool AI 工具增强迭代计划

> 定位：SuperTool = **AI Agent 的运维工具箱（stool CLI）**，GUI 只做辅助（审批 / 审计 / 可视化）。
> 目标：让 AI 能通过 `stool` CLI 原生、安全、可追溯地完成「巡检 → 诊断 → 部署 → 验证 → 回滚」全闭环。
> 版本基线：6.8.14（2026-08）。计划执行顺序 = 阶段序号，每阶段独立可交付。

---

## 一、现状差距（2026-08 调查结论）

| 差距 | 现状 | 影响 |
|------|------|------|
| 输出不结构化 | `db redis` 13 个子命令、`log search/tail/context`、`server exec/read/download/mkdir/rm/java-restart`、`mfa code/parse-uri`、`cicd deploy/rollback`、`nginx fetch/test/deploy`、Git 写操作、全部写操作（add/update/delete）—— 均为裸文本/ANSI，错误也是 `✗ 文本` 到 stderr | AI 无法可靠解析，只能人读 |
| 无 MCP / 工具协议 | 全仓库 0 个 MCP / function-calling / schema 代码，AI 只能靠 skill 文档手拼命令 | AI 不能原生工具化调用 |
| core 能力大量闲置 | Git 60+ 方法只暴露 8 个（diff/stash/cherry-pick/revert/compare/blame/conflict 全在 core 未接 CLI）；OpenVPN/WireGuard/LAN/log_sanitizer 零 CLI | AI 能力面窄 |
| 审批硬拦截 | `cicd deploy` 遇 requiresApproval 直接 `bail!("请在 GUI 中手动确认部署")`，CLI 无审批交互 | AI 无法完成部署闭环 |
| 零审计 | CLI 操作不留痕，无审计表，GUI 无展示 | AI 做了什么不可追溯 |

## 二、迭代阶段

### Phase 0：CLI 输出结构化（地基，所有后续阶段的前提）

**0.1 全局 JSON 输出规范**
- 统一成功/失败 envelope：成功 `{"ok": true, "data": <结果>}`；失败 `{"ok": false, "error": {"message": "...", "code": <exit_code>}}`
- `-j/--json` 提升为**全局 flag**（clap global），不再逐命令定义；写操作默认输出 `{"ok": true, "data": {"id": "...", ...}}` 而非 `✓ 文本`
- 涉及：`cli/src/types.rs`（全局 flag）、`cli/src/output.rs`（envelope + 紧凑模式）、`cli/src/main.rs`（分发层统一包装错误）、13 个 `commands/*.rs`

**0.2 缺口命令补 `-j`（按 AI 使用频率排序）**
- `db redis` 全部 13 个子命令（keys/get/set/delete/type/ttl/h-get/h-get-all/h-len/l-range/l-len/s-members/s-card）
- `log search / tail / context`（结构化：serverId/lineNum/isMatch/content）
- `server exec / exec-batch / read / download / mkdir / rm / java-restart`（exec 返回 exit_code + stdout + stderr + duration）
- `cicd deploy / rollback / cancel / logs`（deploy 流式模式下 `--json` 输出逐行事件对象）
- `mfa code / parse-uri`、`nginx fetch/test/deploy`、Git 写操作、`backup export/import`
- 写操作统一：成功返回新对象 id + 摘要

**0.3 错误统一 + exit code 规范化**
- stderr 在 JSON 模式下输出 `{"ok": false, "error": {...}}`，非 JSON 模式保持人读
- exit code 规范：0=成功，1=业务错误，2=参数错误（clap 默认），3=未授权/需审批，4=连接失败
- 高危命令拦截错误码区分（`is_dangerous_command`、路径拦截）→ 5

**0.4 紧凑输出（AI token 效率）**
- `print_json` 增加紧凑模式（`--compact` 或 JSON 模式下默认紧凑 + `--pretty` 可选），解决 pretty JSON token 开销大问题

**验收**：全命令 `stool <cmd> ... -j` 输出合法 JSON envelope；错误场景 `echo $?` 命中规范 code；`stool guide` 更新。

### Phase 1：审计基础设施（越早埋点成本越低）

- core 新增 `audit_logs` 表：`id, actor_type(ai|cli|gui|user), actor_name, command, args_json(脱敏后), target(server/db/cicd 等), result(success|failed|blocked), duration_ms, created_at`
- CLI 写操作统一埋点（exec/rm/mkdir/download/deploy/rollback/db 写/redis 写/git 写/mfa add/note/accounting 写）——在命令分发层做（main.rs 拦截），不逐命令改
- 脱敏：参数经 `log_sanitizer::sanitize_params_for_log`（密码/token 打码）后入库
- `stool audit list [-j] [--actor ai] [--target server]` 新命令
- GUI 新增「操作审计」页（src/views/audit/）：列表 + 筛选 + 详情（只读展示）

**验收**：任意写操作后 `stool audit list -j` 可见记录；GUI 审计页可筛选；密码不出现在 args_json。

### Phase 2：MCP Server（AI 原生接入，核心交付）

- `stool mcp serve`：内置 MCP **stdio** server（Rust，JSON-RPC 2.0 实现，无第三方依赖或轻量依赖）
  - `tools/list` / `tools/call`：把 core 高频命令包装成 MCP tools，每个 tool 带名称、描述、JSON schema 参数、结构化输出
  - `resources/list` / `resources/read`：只读资源（server list、log presets、cicd configs）供 AI 直接查询
  - 初始工具集（~25 个）：server.list/health/diagnose/exec(需审批标志)、cicd.list/deploy/status/history/rollback、db.query/tables/structure/data、db.redis.*、log.list/search/tail/context、git.status/log/diff、mfa.code、todo.*、note.*、audit.list
  - 安全：危险 tool（exec/rm/deploy/db 写）在 schema 标注 + MCP 层 requiresApproval 检查（Phase 4 接入审批流）
- 接入配置文档：`docs/mcp-integration.md`（Claude Code mcp.json / Cursor / Trae 各一段配置 + 使用示例）
- 版本：`stool mcp` 独立子命令组

**验收**：`stool mcp serve` 启动后 Claude Code / Cursor 能发现工具并完成一次真实调用（如 `stool server list` 等价 tool）。

### Phase 3：能力扩展（暴露 core 闲置能力）

**3.1 Git 高级命令（AI 代码评审/冲突解决高频）**，core 方法已存在，CLI 补接线：
- `git diff <repo> [--stat|--name-only] [--staged]`、`git diff-file <repo> <path>`
- `git stash list/save/apply/pop/drop/show`、`git cherry-pick <repo> <commit>`、`git revert <repo> <commit>`
- `git compare <repo> <branchA> <branchB> [-j]`（compare_branches）、`git blame <repo> <file> [-j]`
- `git conflicts <repo> [-j]`（conflict_files）、`git reset-to <repo> <commit>`、`git file-log <repo> <path>`（file_history）
- `git remote list/add/remove/set-url`

**3.2 server 诊断扩展**：
- `server df <id> [-j]`（磁盘使用）、`server mem <id> [-j]`、`server ps <id> [--filter java] [-j]`、`server ports <id> [-j]`、`server info <id> [-j]`（uname/uptime/load）

**3.3 log_sanitizer 接入**：CLI/MCP 输出加 `--sanitize` 开关，AI 读生产日志默认脱敏（手机号/身份证/密码模式打码）

**3.4 只读巡检命令**：`openvpn status [-j]`、`wireguard status [-j]`（CLI 补只读查询，控制操作留给 GUI）

**验收**：Git diff/stash/compare 等 10+ 新命令可用且输出结构化；server df/mem/ps 输出 `-j`；`--sanitize` 对样例日志脱敏生效。

### Phase 4：审批闭环（AI 操作生产的安全闸门）

- core 新增 `approval_requests` 表：`id, command, args_json(脱敏), target, requester(ai|cli), status(pending|approved|rejected|expired), created_at, decided_by, decided_at`
- **CLI 侧**：`server exec`/`cicd deploy`/`db 写` 遇 requiresApproval → 创建审批请求 → `--wait-approval` 轮询（间隔 2s，超时 5min）→ GUI 确认后继续执行 / 拒绝则 `exit 3`
- **GUI 侧**：新增「审批中心」页（src/views/approval/）：pending 列表 + 详情（命令 + 脱敏参数 + 发起者）+ 批准/拒绝；CICD 既有审批弹窗迁移到统一机制
- MCP 层：审批型 tool 调用自动走同一流程（AI 请求 → GUI 人工批准 → AI 继续）
- CLI 无 GUI 时：`--yes`（预授权，仅 test/dev 环境）或拒绝

**验收**：AI 通过 MCP 调 `cicd.deploy` 触发 GUI 审批弹窗；批准后部署继续；拒绝后 AI 收到明确错误；全程入审计。

### Phase 5：GUI 辅助化收尾

- 「审批中心」+「操作审计」+「AI 操作记录」（命令历史时间线，CLI/MCP 操作可视化）
- GUI 移除对业务逻辑的依赖倾向，只读展示 + 审批交互（符合"GUI 辅助"定位）
- `settings` 增加「AI 工具」配置项：MCP 开关、审批超时、审计保留策略

**验收**：GUI 能展示 AI 通过 CLI/MCP 做的全部操作与审批；GUI 本身不再新增业务能力。

---

## 三、优先级与依赖

```
Phase 0 输出结构化 ──▶ Phase 2 MCP（MCP 依赖结构化）
      │
      ├──▶ Phase 1 审计（早埋点，写操作路径不变）
      │         │
      │         └──▶ Phase 4 审批（复用审计的命令/参数链路）
      │
      └──▶ Phase 3 能力扩展（新命令直接按 Phase 0 规范输出）
                          │
                          └──▶ Phase 5 GUI 收尾
```

## 四、风险与对策

| 风险 | 对策 |
|------|------|
| MCP 库引入 Rust 依赖复杂度 | 优先手写 JSON-RPC 2.0（协议简单，~200 行），不引重库 |
| 全局 -j 改造破坏现有脚本/技能文档 | Phase 0 保持向后兼容：非 JSON 模式默认输出不变，`-j` 才切 JSON；技能文档同步更新 |
| 审批流轮询阻塞 AI 会话 | 轮询可中断（--timeout）、支持后台发起（不等待直接返回 request_id） |
| 审计落库影响 CLI 性能 | 写操作才审计（读操作不记），异步写 + 失败不阻塞主流程 |
| 脱敏遗漏导致敏感信息入库 | 复用 log_sanitizer 既有模式（sanitize_params_for_log），审计字段强制走该函数 |

## 五、各阶段工作量估算（相对）

| 阶段 | 相对工作量 | 主要文件 |
|------|-----------|---------|
| Phase 0 结构化 | ★★★ | cli/src/output.rs、types.rs、main.rs、13 个 commands/*.rs |
| Phase 1 审计 | ★★ | core 新表 + service、cli/src/main.rs、tauri commands、src/views/audit/ |
| Phase 2 MCP | ★★★★ | cli/src/commands/mcp.rs（新）、docs/mcp-integration.md |
| Phase 3 能力扩展 | ★★★ | cli/src/commands/git.rs、server.rs、log.rs |
| Phase 4 审批 | ★★★ | core 新表 + service、cli（审批流）、src/views/approval/ |
| Phase 5 GUI 收尾 | ★★ | src/views/ 若干 + settings |
