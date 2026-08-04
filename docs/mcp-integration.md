# stool MCP Server 接入指南

`stool mcp serve` 将 SuperTool CLI 的核心能力包装成 **MCP（Model Context Protocol）工具**，让 Claude Code / Cursor / Trae 等 AI 编码工具原生调用：服务器管理、CI/CD、数据库、日志、Git、MFA、任务、审计。

## 快速开始

```bash
stool mcp serve            # 启动 stdio MCP server（阻塞运行）
stool mcp list-tools       # 查看工具清单（调试用）
```

MCP 走 stdio 协议（每行一条 JSON-RPC 消息），由 AI 客户端作为子进程拉起，**不需要手动运行**——只需在客户端配置里指向该命令。

## Claude Code 接入

编辑 `~/.claude.json`（或项目级 `.mcp.json`）的 `mcpServers`：

```json
{
  "mcpServers": {
    "stool": {
      "command": "/usr/local/bin/stool",
      "args": ["mcp", "serve"]
    }
  }
}
```

重启 Claude Code 后，通过 `@stool` 前缀即可调用工具（如 `@stool server_list`）。

## Cursor 接入

Settings → MCP → Add server：

```
名称: stool
类型: command
命令: /usr/local/bin/stool mcp serve
```

## 工具清单（22 个）

| 分组 | 工具 | 说明 |
|------|------|------|
| 服务器 | `server_list` | 列出服务器（含分组/审批状态） |
| | `server_exec` | 执行 shell 命令（⚠️ 高危命令与需审批服务器自动拦截） |
| CI/CD | `cicd_list` | 部署配置列表 |
| | `cicd_deploy` | 触发部署（⚠️ 需审批配置自动拦截，提示用户 GUI 确认） |
| | `cicd_history` | 部署历史 |
| 数据库 | `db_list` / `db_query` / `db_tables` | SQL 查询与表结构 |
| Redis | `redis_keys` / `redis_get` | key 浏览与取值 |
| 日志 | `log_list` / `log_search` / `log_tail` / `log_context` | 搜索与上下文定位 |
| Git | `git_status` / `git_log` / `git_branches` | 仓库状态与历史 |
| MFA | `mfa_code` | 生成 TOTP 验证码 |
| 任务 | `todo_list` / `todo_add` / `todo_complete` | 任务管理 |
| 审计 | `audit_list` | 操作审计查询 |

资源（resources）：`stool://servers`、`stool://log-presets`、`stool://cicd-configs`（只读，AI 可直接查询）。

## 安全模型

- **高危命令拦截**：`server_exec` 复用 CLI 的 `is_dangerous_command`（rm -rf、kill -9、shutdown、curl|sh 等），被拦时返回 isError
- **审批拦截**：`server_exec` / `cicd_deploy` 检查目标的 `requiresApproval`，开启审核时返回 isError 并提示"请用户在 GUI 确认"（审批闭环接入后自动流转，见 Phase 4）
- **只读优先**：初始工具集以查询为主，写操作（exec/deploy）均带拦截
- **审计**：所有写操作经 CLI 分发层落 `audit_logs`（参数脱敏），`audit_list` 可查

## 使用示例（Claude Code 对话）

> 用户：看看生产服务器 3 的磁盘情况
> AI：`@stool server_exec`（id=1781071313377, command=`df -h`）→ 返回磁盘使用
> AI：发现 `/` 使用率 95%，建议清理日志 → `@stool log_search`（preset=9, keyword=ERROR）定位问题

> 用户：帮我部署 prepay 服务到 dev
> AI：`@stool cicd_deploy`（config_id=..., branch=dev）
> AI：若被审批拦截 → 提示"该配置需要人工审批，请在 SuperTool GUI 的部署页面确认"
