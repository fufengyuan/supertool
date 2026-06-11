---
name: stool-cli
category: devops
description: SuperTool `stool` CLI v4.73.5 — AI Agent 专属运维工具。直连 `supertool-core` 共享库，零 UDS/HTTP 依赖。支持 Hermes/Claw AI 对话、服务器管理、CI/CD、数据库、Git、日志、MFA、笔记、记账、周报、Nginx、备份。
trigger: 使用 stool 命令、排查 CLI 失败、添加新 CLI 命令、AI 运维操作
---

# SuperTool `stool` CLI v4.73.5

> AI Agent 专属运维工具 —— 直连 `supertool-core` 共享库，零 UDS/HTTP 依赖，完全独立运行。

## 架构

```
┌──────────────┐     ┌──────────────────────┐     ┌──────────────────┐
│  stool CLI   │────▶│   supertool-core     │◀────│  Tauri GUI       │
│  (clap)      │     │  (db/ssh/redis/cicd) │     │  (Vue3 + IPC)    │
│  ~6MB        │     │  ── SQLite ──▶       │     │                  │
└──────────────┘     └──────────────────────┘     └──────────────────┘
```

- **CLI**: 直连 `supertool-core` 共享库，零网络/UDS/HTTP，完全独立
- **Tauri GUI**: 同样直连 core，负责密钥加密存储和 UI
- **共享库 `core/`**: 单一事实来源，包含所有业务逻辑

## 工作区结构

```
Cargo.toml       # workspace members = ["core", "cli", "tauri",
                   #   "claw-*", "hermes-*"]
core/            # supertool-core — 共享库
cli/             # stool CLI — 直连 core，独立二进制
tauri/           # Tauri GUI — 直连 core + Vue 3 前端
```

## 全命令速查

### 基础
```bash
stool version                         # 显示版本
stool guide                           # 使用指南
```

### 📋 任务管理
```bash
stool todo add "文本" [-p high|medium|low] [-d 日期] [-t 标签] [--description 描述]
stool todo list [-c true|false] [-t 标签] [-l 50] [-j]
stool todo complete <id>
stool todo uncomplete <id>
stool todo delete <id>
stool todo show <id> [-j]
stool todo edit <id> [-t "文本"] [-p high] [--due 日期] [--tag 标签]
stool todo search "关键词" [-j]
stool todo stats [-j]
stool todo clear

stool subtask list <todo_id> [-j]
stool subtask add <todo_id> "文本"
stool subtask complete <sub_id>
stool subtask delete <sub_id>

stool project list [-j]
stool project add "项目名" [-d "描述"]
stool project show <id> [-j]
stool project update <id> [-n "新名"] [--description 描述]
stool project delete <id>
stool project stats <id> [-j]
stool project todos <id> [-j]
```

### 🖥️ 服务器管理
```bash
stool server list [-j]
stool server add "名称" <host> [端口] [用户]
stool server test <id>
stool server exec <id> "命令" [--timeout 60]
stool server health <id> [-j]
stool server diagnose <id> [-j]
stool server delete <id>

# 文件操作
stool server read <id> <路径>
stool server ls <id> [--path /目录] [-j]
stool server download <id> <远程路径> [--output 本地路径]
stool server mkdir <id> <路径>

# Java 进程
stool server java-ps <id> [-j]
```

> ⚠️ `server rm` / `server exec-batch` / `server java-restart` 已禁用 CLI。

### 🚀 CI/CD
```bash
stool cicd list [-j]
stool cicd status <project_id> [-j]
stool cicd deploy <config_id> [--stream] [--watch]
stool cicd history <config_id> [-l 20] [--status ...] [-j]
stool cicd step-logs <deploy_log_id> [-j]
stool cicd rollback <config_id> <deploy_log_id>
stool cicd cancel <config_id>
stool cicd modules <config_id> [-j]
stool cicd logs <project_id> [-l 20]
```

### 🗄️ 数据库
```bash
stool db list [-j]
stool db query -d <db_id> "SELECT ..." [-j]
stool db tables -d <db_id> [--db 库名] [-j]
stool db databases -d <db_id> [-j]
stool db disconnect <id>

# Redis
stool db redis -d <id> keys "pattern"
stool db redis -d <id> get <key>
stool db redis -d <id> type <key>
stool db redis -d <id> ttl <key>
stool db redis -d <id> h-get <key> <field>
stool db redis -d <id> h-get-all <key>
stool db redis -d <id> h-len <key>
stool db redis -d <id> l-range <key> [start] [stop]
stool db redis -d <id> l-len <key>
stool db redis -d <id> s-members <key>
stool db redis -d <id> s-card <key>
stool db redis -d <id> set <key> <value>
stool db redis -d <id> delete <key>
```

### 📝 日志
```bash
stool log list [-j]
stool log search <preset> "关键词" [-l 50]
stool log tail <preset> [-l 100]
stool log add "名称" --server-ids "id1,id2" --log-path /var/log/app.log [--log-type tail]
stool log delete <id>
```

### 🔧 Git
```bash
stool git list [-j]
stool git status --path <路径> [-j]
stool git log --path <路径> [-l 20] [-j]
stool git branches --path <路径> [-j]
stool git pull --path <路径>
stool git push --path <路径>
stool git commit --path <路径> -m "消息" [--files f1 f2]
stool git checkout --path <路径> --branch <分支>
```

### 🤖 AI Agent 对话（v4.73+）
```bash
stool hermes <消息>                       # Hermes 对话（全工具支持）
stool claw chat <消息>                    # Claw 对话（全工具支持）
stool claw goal <目标> [--max-turns 20]   # Goal 模式 — 多轮直到目标达成
stool claw loop <消息> [--count 5]         # Loop — 自动重发
stool claw loop <消息> [--duration 30s]   # Loop — 按时间限制
```

**实现**（`cli/src/commands/agent.rs`）：调用官方 Python `hermes chat -q <message> --quiet` 子进程。

| 模式 | 差异 |
|------|------|
| `hermes` | 直通参数 |
| `claw chat` | 注入 Claw 系统提示 `[System: You are Claw, a focused AI coding assistant.]` |
| `claw goal` | 多轮 + session 恢复 (`-r`) + `[GOAL_COMPLETE]` 检测 + Judge 确认轮。1s 间隔 |
| `claw loop` | 每轮 800ms 重发。支持 `--count N` 或 `--duration 30s` |

**session_id 从 stderr 提取**（格式 `session_id: <hex>`），传给后续 `-r` 恢复对话。

### 🔐 MFA 管理
```bash
stool mfa list [-j]        # 列出所有 MFA 密钥
stool mfa code <服务名称>   # 生成 TOTP 验证码
```

### 📝 笔记管理
```bash
stool note list [-j]
stool note add "标题" [-b 分组] [--content "内容"]
stool note show <id> [-j]
stool note delete <id>
```

### 💰 记账管理
```bash
stool accounting add [-t income|expense] -a 金额 -c 分类 [-d 描述] [-n 备注]
stool accounting list [-p YYYY-MM] [-c 分类] [-t income|expense] [-j]
stool accounting stats [-p YYYY-MM] [-j]
stool accounting budget set -c 分类 -a 金额
stool accounting budget check [-j]
stool accounting export [--csv 路径]
```

### 📋 周报管理
```bash
stool weekly generate [--from YYYY-MM-DD] [--to YYYY-MM-DD]
stool weekly list [-j]
stool weekly show <id> [-j]
stool weekly delete <id>
```

### 🌐 Nginx 配置
```bash
stool nginx preset list [-j]
stool nginx preset load <id> <服务器ID>
stool nginx push <服务器ID> [配置路径]
stool nginx pull <服务器ID> [配置路径]
stool nginx test <服务器ID>
stool nginx deploy <服务器ID>
```

### 💾 备份
```bash
stool backup export [--output 路径]
stool backup import [--input 路径]
```

## 版本管理

4 处统一：`package.json` + `cli/Cargo.toml` + `core/Cargo.toml` + `tauri/Cargo.toml`。

| Commit 类型 | 版本变化 |
|-------------|----------|
| `fix:` | patch (+0.0.1) |
| `feat:` | minor (+0.1.0) |
| `feat!:` / `BREAKING CHANGE` | major (+1.0.0) |
| `chore:` / `docs:` / `style:` | 不变 |

Git hook 自动 bump（`scripts/hooks/`），`pnpm install` 时自动配置。

## AI Agent 对话实现细节

### 调用方式（子进程，非 Rust 函数调用）

```rust
Command::new("hermes")
    .arg("chat")
    .arg("-q").arg(message)
    .arg("--quiet")
    .output()
```

**为什么不直接调 `hermes_cli::handle_cli_chat`？**
1. ~~**Model remediation** — 已移除（commit 52a366f4）~~ ✅
2. **Cron schema 不兼容** — `handle_cli_chat` 自动加载 `~/.hermes/cron/jobs.json`，缺少 `id` 字段时整条路径崩溃
3. **Custom provider** — `model.provider: custom` 在 Rust 端报 `NoBackendProvider`

障碍 2+3 未解决前，子进程是唯一可靠路径。

### 配置隔离

- `stool hermes` — 通过 `hermes` CLI 读取 `~/.hermes/config.yaml`
- `stool claw` — 注入 Claw 系统提示，同样走 `hermes` CLI
- **绝对不能互 fallback**。claw=claw, hermes=hermes

## 构建

```bash
# 完整构建（推荐）
pnpm build:app                # CLI + Tauri（native arch）
pnpm build:app:arm64          # CLI + Tauri（arm64）
pnpm build:app:x64            # CLI + Tauri（x64）
pnpm build:app:universal      # CLI + Tauri（universal）

# 打包（含 DMG/pkg/deb）
pnpm build:pkg                # 自动检测 OS
pnpm build:pkg:arm64
pnpm build:pkg:universal

# 仅 CLI
cargo build --release -p stool   # 产物 target/release/stool

# 仅 Tauri
pnpm tauri build              # 产物在 target/release/bundle/
```

## 工作流

### 部署 → 验证 → 回滚
```bash
stool cicd deploy my-config-id --stream
stool server health prod-server-id
stool log search prod-logs "ERROR" -l 30
stool server diagnose prod-server-id
stool cicd rollback my-config-id last-good-id
```

### 日常巡检
```bash
stool server list -j
stool todo list -p high -j
stool cicd list -j
```

### 故障排查
```bash
stool server health server-id -j
stool server java-ps server-id -j
stool db redis -d redis-id keys "session:*"
stool db query -d db-id "SELECT COUNT(*) FROM orders WHERE status='pending'"
stool log tail app-logs -l 200
```

## ⚠️ 关键陷阱

1. **`-j` 是 JSON 别名** — 所有 list/status 命令都支持
2. **UUID 不可截断** — 所有 list 输出完整 36 位 UUID
3. **高危命令拦截** — `server exec` 拦截 `rm -rf`、`kill -9`、`shutdown`、`curl|sh` 等
4. **requiresApproval 三重拦截** — 服务器/数据库/CICD 各自独立
5. **preset_id 智能解析** — `log search 1 "关键词"` 序号自动转 UUID
6. **部署超时** — `--watch` 最长 10 分钟；`--stream` 无硬超时
7. **server download** — base64 传输，自动保存本地
8. **CLI 不传冗余参数** — 连接已在 GUI 配好，通过 `-d <db_id>` 引用
9. **session_id 在 stderr** — `hermes chat -q` 的 session_id 输出到 stderr 非 stdout
10. **API key 读写用 Python** — `write_file` 脱敏 API key 为 `***`，用 Python `open()` 直接 I/O
11. **目录名 ≠ package name** — `claw-cli/` 目录的包名是 `supertool-claw`，加 dep 用 package name
12. **Model remediation 已移除** — commit 52a366f4，模型严格按配置，不再自动纠正
13. **配置完全隔离** — Hermes/Claw 绝不互 fallback
14. **edition 2024 set_var 须 unsafe** — `std::env::set_var` 用 `unsafe { ... }` 包裹

## 源码结构

```
cli/src/
├── main.rs                # clap 入口 + 命令分发
├── types.rs               # clap 类型定义
├── runtime.rs             # CliRuntime — DB 连接 + CoreService
├── output.rs              # 格式化输出
├── utils.rs               # shell_quote, is_dangerous_command, format_size
├── guide.rs               # 使用指南
└── commands/
    ├── agent.rs           # hermes/claw AI 对话（子进程调用）
    ├── todo.rs            # 任务管理
    ├── subtask.rs         # 子任务
    ├── project.rs         # 项目管理
    ├── server.rs          # 服务器管理
    ├── cicd.rs            # CI/CD 部署
    ├── db.rs              # 数据库管理
    ├── log.rs             # 日志聚合器
    ├── git.rs             # Git 仓库
    ├── mfa.rs             # MFA 管理
    ├── note.rs            # 笔记管理
    ├── accounting.rs      # 记账管理
    ├── weekly.rs          # 周报管理
    ├── nginx.rs           # Nginx 配置
    └── backup.rs          # 数据备份
```

## CLI 自动分发

App 启动时 `cli_installer.rs` 检测版本差异，通过 AppleScript 提权自动安装 `/usr/local/bin/stool`。同时该项目下的 `skills/stool-cli/SKILL.md` 会自动复制到 `~/.hermes/skills/stool-cli/SKILL.md`。
