---
name: stool-cli
category: devops
description: SuperTool `stool` CLI v6.5.0 — AI Agent 专属运维工具。直连 `supertool-core` 共享库，零 UDS/HTTP 依赖。支持服务器管理、CI/CD、数据库、Git、日志、MFA、笔记、记账、周报、Nginx、备份。
trigger: 使用 stool 命令、排查 CLI 失败、添加新 CLI 命令、AI 运维操作
---

# SuperTool `stool` CLI v6.5.0

> AI Agent 专属运维工具 —— 直连 `supertool-core` 共享库，零 UDS/HTTP 依赖，完全独立运行。

## 架构

```
┌──────────────┐     ┌──────────────────────┐     ┌──────────────────┐
│  stool CLI   │────▶│   supertool-core     │◀────│  Tauri GUI       │
│  (clap)      │     │  (db/ssh/redis/cicd) │     │  (Vue3 + IPC)    │
│  ~12MB       │     │  ── SQLite ──▶       │     │                  │
└──────────────┘     └──────────────────────┘     └──────────────────┘
```

- **CLI**: 直连 `supertool-core` 共享库，零网络/UDS/HTTP，完全独立
- **Tauri GUI**: 同样直连 core，负责密钥加密存储和 UI
- **共享库 `core/`**: 单一事实来源，包含所有业务逻辑

## 工作区结构

```
Cargo.toml       # workspace members = ["core", "cli", "tauri"]
core/            # supertool-core — 共享库
cli/             # stool CLI — 直连 core，独立二进制
tauri/           # Tauri GUI — 直连 core + Vue 3 前端
```

## JSON 输出规范（AI 解析约定）

所有命令的 `-j` / 全局 `--json` 输出统一为 **envelope**：

```json
{"ok": true,  "data": <结果对象/数组>}
{"ok": false, "error": {"code": 1, "message": "错误信息"}}
```

- 错误 envelope 输出到 **stderr**，成功 envelope 输出到 **stdout**（分离，AI 可分别读取）
- **全局 `--json`**：`stool --json <任意命令>`，与各命令级 `-j` 等价，可放在任何位置
- **exit code 规范**：`0`=成功，`1`=业务错误，`2`=参数错误(clap)，`3`=需审批/未授权，`4`=连接失败，`5`=高危命令拦截
- 示例：`stool server list -j` → `{"ok": true, "data": [...]}`；`stool cicd deploy <id> --json`（需审批）→ stderr `{"ok": false, "error": {"code": 3, ...}}` 且 exit 3

## 全命令速查

### 基础
```bash
stool version                         # 显示版本
stool guide                           # 使用指南
```

### 📋 任务管理
```bash
stool todo add "文本" [-p high|medium|low] [-d 日期] [-t 标签] [--description 描述] [--project-id 项目ID]
stool todo list [-c true|false] [-t 标签] [-l 50] [-j]
stool todo complete <id>
stool todo uncomplete <id>
stool todo delete <id>
stool todo show <id> [-j]
stool todo edit <id> [-t "文本"] [-p high] [--due 日期] [--tag 标签] [--description 描述]
stool todo search "关键词" [-j]
stool todo stats [-j]
stool todo clear

stool subtask list <todo_id> [-j]
stool subtask add <todo_id> "文本" [--description 描述]
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
stool server exec-batch <id> --script "cmd1\ncmd2" [--timeout 120]
stool server health <id> [-j]
stool server diagnose <id> [-j]
stool server delete <id>

# 文件操作
stool server read <id> <路径>
stool server ls <id> [--path /目录] [-j]
stool server download <id> <远程路径> [--output 本地路径]
stool server mkdir <id> <路径>
stool server rm <id> <路径>                        # 删除文件（高危路径拦截）

# Java 进程
stool server java-ps <id> [-j]
stool server java-restart <id> <jar名称> [--timeout 60]   # kill → 等待 → SIGKILL
```

**`exec-batch`**: 按 `\n` 分割脚本逐行执行，跳过空行和 `#` 注释；首条失败即中止。
**`rm`**: 拦截系统目录（`/`、`/etc`、`/usr`、`/bin`、`/boot`、`/sys`、`/proc`）。
**`java-restart`**: 按 jar 名匹配 Java 进程 → `kill` → 等待 `--timeout` 秒 → 仍未退出则 `kill -9`。**不自动重启**，需配合部署脚本。

### 🚀 CI/CD
```bash
stool cicd list [-j]
stool cicd status <project_id> [-j]
stool cicd deploy <config_id> [--stream] [--watch] [-b <分支>]    # -b/--branch 覆盖配置中的 deployBranch
stool cicd history <config_id> [-l 20] [--status success|failed] [-j]
stool cicd step-logs <deploy_log_id> [-j]
stool cicd rollback <config_id> <deploy_log_id>
stool cicd cancel <config_id>
stool cicd modules <config_id> [-j]
stool cicd logs <config_id> [-l 20]
stool cicd tools [--scan-path /项目路径] [-j]          # 检测构建工具 + SDK 版本 + 项目模块
```

**`cicd tools`** 输出：
- **tools**: java/maven/node/npm/pnpm/yarn/gradle 的可用性 + 版本 + 路径
- **toolPaths**: JAVA_HOME / MAVEN_HOME / NODE_HOME 等
- **sdkVersions**: SDKMAN / NVM 已安装版本
- **projectScan**（需 `--scan-path`）: pom.xml / build.gradle / package.json 检测 + 模块树

### 🗄️ 数据库
```bash
stool db list [-j]
stool db query -d <db_id> "SELECT ..." [-j]                     # 执行 SQL
stool db databases -d <db_id> [-j]                              # 列出数据库
stool db tables -d <db_id> [--db 库名] [-j]                     # 列出表
stool db structure -d <db_id> [--db 库名] <表名> [-j]           # 表结构
stool db data -d <db_id> [--db 库名] <表名> [-l 100] [--offset 0] [-j]  # 表数据
stool db disconnect <id>                                        # 无状态，无操作
```

**CLI 无状态**：每条命令连接 → 执行 → 断开，无连接池。

### 🗄️ Redis
```bash
stool db redis -d <id> keys [pattern]                # 列出 key（默认 *）
stool db redis -d <id> get <key>                     # 获取值（显示类型 + 值）
stool db redis -d <id> set <key> <value>             # 设置 string
stool db redis -d <id> delete <key>                  # 删除 key
stool db redis -d <id> type <key>                    # key 类型
stool db redis -d <id> ttl <key>                     # TTL（-1=永不过期, -2=不存在）
stool db redis -d <id> h-get <key> <field>           # Hash 字段获取
stool db redis -d <id> h-get-all <key>               # Hash 全部获取
stool db redis -d <id> h-len <key>                   # Hash 长度
stool db redis -d <id> l-range <key> [start] [stop]  # List 范围（默认 0 -1）
stool db redis -d <id> l-len <key>                   # List 长度
stool db redis -d <id> s-members <key>               # Set 成员
stool db redis -d <id> s-card <key>                  # Set 基数
```

`db_index` 从连接配置的 `dbIndex` 字段读取（默认 0）。

### 📝 日志
```bash
stool log list [-j]
stool log search <preset> "关键词" [-l 50]
stool log tail <preset> [-l 100]                            # 静态 tail（非流式）
stool log context <preset> <server_id> <行号> [-c 20]       # 查看上下文（目标行 ▶ 标记）
stool log add "名称" --server-ids "id1,id2" --log-path /var/log/app.log [--log-type tail]
stool log delete <id>
```

**`preset` 智能解析**: 可用序号（1-based），CLI 自动转 UUID。 **`log context`**: 显示 `行号` 周边上下文，目标行用 `▶` 标记。 **`-l` 限制**: `log search` 默认只搜最近 `maxLines` 行（预设配置，如 500），历史日志可能搜不到，需增大 `-l` 或用 `log tail` 翻更多。

#### 日志排查实战经验

**场景：通过追踪号(traceId)还原完整调用链路**

Java 日志通常带 `[traceId]` 前缀（如 `[11091235193656539418624]`），一次请求内所有日志共享同一 traceId。

```bash
# 1. 先用业务关键词（订单号/交易号）搜到一条日志，拿到 traceId
stool log search 9 "2026073000152601000000000005" -l 80

# 2. 用 traceId 搜完整链路（-l 加大确保覆盖全量）
stool log search 9 "11091235193656539418624" -l 200

# 3. 过滤关键行（排除 SQL DEBUG 噪音），快速定位问题
stool log search 9 "11091235193656539418624" -l 200 | grep -v "Preparing\|Parameters\|<==" | grep -i "ERROR\|核销\|status=\|结果"
```

**关键词搜索支持 `\|` 多选**（grep 风格）：

```bash
stool log search 9 "预付卡核销失败\|status=5\|支付中\|doPrePay" -l 50
```

**排查顺序**：

1. `stool log list -j` 找到目标环境的日志预设（序号即可）
2. 用业务唯一标识（tradeNo/orderNo/userId）搜到入口日志，提取 traceId
3. 用 traceId 搜全链路，`-l` 设大（200+）避免遗漏
4. `grep -v "Preparing\|Parameters\|<=="` 过滤 MyBatis SQL 噪音
5. `grep -i` 聚焦 ERROR/INFO 关键行，确认执行顺序和状态流转

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

### 🔐 MFA 管理
```bash
stool mfa list [-j]
stool mfa add "名称" <密钥> [--issuer 发行方] [--digits 6] [--period 30] [--algorithm SHA1]
stool mfa delete <id>
stool mfa code <标识>                    # 生成 TOTP（按 ID 或序号）
stool mfa parse-uri "otpauth://..."
```

### 📝 笔记管理
```bash
stool note list [--query 关键词] [--group-id 分组ID] [-j]
stool note add "标题" [--content 内容] [--group-id 分组ID] [--tags "t1,t2"]
stool note update <id> [--title 标题] [--content 内容] [--group-id 分组ID] [--tags 标签]
stool note delete <id>
stool note groups [-j]
stool note add-group "名称" [--color #hex]
stool note update-group <id> [--name 名称] [--color #hex]
stool note delete-group <id>
```

### 💰 记账管理
```bash
stool accounting list [--category 分类] [--type income|expense] [--year 年] [--month 月] [-j]
stool accounting add <金额> --category 分类 --type income|expense [--note 备注] [--date 日期]
stool accounting update <id> [--amount 金额] [--category 分类] [--type 类型] [--note 备注]
stool accounting delete <id>
stool accounting categories [-j]
stool accounting add-category "名称" [--icon 图标] [--color 颜色]
stool accounting delete-category <id>
stool accounting budgets [-j]
stool accounting add-budget <分类> <金额> [--month YYYY-MM]
stool accounting delete-budget <id>
stool accounting stats [--year 年] [-j]
stool accounting trend [--months 12] [-j]
```

### 📋 周报管理
```bash
stool weekly list [-l 10] [-j]
stool weekly show <id> [-j]
stool weekly save "标题" --content "内容" [--start-date 起] [--end-date 止]
```

### 🌐 Nginx 配置
```bash
stool nginx list [-j]
stool nginx add "名称" [--server-id 服务器ID] [--config-path 路径] [--content 内容]
stool nginx update <id> [--name 名称] [--server-id 服务器ID] [--config-path 路径]
stool nginx delete <id>
stool nginx fetch <server_id> <config_path>
stool nginx test <server_id> <config_path>
stool nginx deploy <server_id> <config_path> <content>
stool nginx versions <preset_id> [-j]
```

### 💾 备份
```bash
stool backup export [--output 路径]
stool backup import <文件> [--mode merge|replace]
stool backup export-csv                    # 导出 todo 为 CSV
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
stool cicd deploy my-config-id --stream                    # 用配置中的默认分支
stool cicd deploy my-config-id --stream -b dev             # 临时用 dev 分支部署
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
stool log context app-logs server-id 1234 -c 30     # 查看第 1234 行周边上下文
```

### Java 服务重启
```bash
stool server java-ps server-id -j                   # 查看当前 Java 进程
stool server java-restart server-id my-app.jar      # 停止 my-app.jar 进程
stool cicd deploy my-app-config --stream            # 重新部署
```

### 批量运维
```bash
stool server exec-batch server-id --script "cd /app
git pull
mvn clean package -DskipTests
systemctl restart myapp"
```

## ⚠️ 关键陷阱

1. **`-j` 是 JSON 别名** — 所有 list/status 命令都支持
2. **UUID 不可截断** — 所有 list 输出完整 36 位 UUID
3. **高危命令拦截** — `server exec` / `exec-batch` 拦截 `rm -rf`、`kill -9`、`shutdown`、`curl|sh` 等
4. **`server rm` 路径拦截** — 系统目录（`/`、`/etc`、`/usr`、`/bin`、`/boot`、`/sys`、`/proc`）拒绝删除
5. **requiresApproval 拦截（生产环境护栏）** — 审批开关只存在于 **服务器 / 数据库 / CICD** 三个模块（GUI 有对应审批配置）；只读操作放行，**写/变更操作拦截**（exit 3）：
   - `server`：写拦截 exec / exec-batch / mkdir / rm；只读放行 read / ls / download / java-ps / health / diagnose / test
   - `cicd`：写拦截 deploy / rollback / cancel（list/status/history/logs 读操作放行）
   - `db`：写拦截非只读 SQL（INSERT/UPDATE/DELETE/DROP/WITH 携带写语句等）与 redis set/delete；只读放行 SELECT/WITH 查询类白名单（SELECT/SHOW/EXPLAIN/DESC/PRAGMA 查询）、tables/structure/data、redis keys/get/type/ttl/h-get 等
   - `log`：无审批开关，全部放行（不拦）
   - `nginx`：**模块级审批开关**（GUI Nginx 页面「部署需审批」，settings `nginx_requires_approval`）——开启后 `nginx deploy` 拦截（exit 3）；`fetch` / `test` 只读放行；开关默认关闭
   - MCP 同步：server_exec / cicd_deploy 拦截；db_query 独立只读白名单（所有连接只允许只读 SQL）；redis_keys / redis_get / log_* 放行
   - AI 遇到 exit 3 应提示用户到 GUI 操作，不要绕过
6. **preset_id 智能解析** — `log search 1 "关键词"` 序号自动转 UUID
7. **部署超时** — `--watch` 最长 10 分钟（每 5s 轮询）；`--stream` 阻塞直到完成
8. **server download** — base64 传输，自动保存本地
9. **CLI 不传冗余参数** — 连接已在 GUI 配好，通过 `-d <db_id>` 引用
10. **db query 无 `-j`** — 输出格式化表格（列头 + 边框）；有 `-j` 返回 `{"success": true, "rows": [...]}`
11. **db 结构/数据** — `--db` 可选，缺省时用连接配置的 `dbName`
12. **Redis db_index** — 从连接配置 `dbIndex` 字段读取，默认 0
13. **log context** — 目标行用 `▶` 标记，上下文行数 `-c` 默认 20（前后各半）
14. **cicd tools** — 不带 `--scan-path` 只检测本机工具；带路径才扫描项目模块
15. **edition 2024 set_var 须 unsafe** — `std::env::set_var` 用 `unsafe { ... }` 包裹

## 源码结构

```
cli/src/
├── main.rs                # clap 入口 + 命令分发
├── types.rs               # clap 类型定义（所有命令变体）
├── runtime.rs             # CliRuntime — DB 连接 + CoreService
├── output.rs              # 格式化输出（print_json, print_error, print_success）
├── utils.rs               # shell_quote, is_dangerous_command, format_size
├── guide.rs               # 使用指南
└── commands/
    ├── todo.rs            # 任务管理
    ├── subtask.rs         # 子任务
    ├── project.rs         # 项目管理
    ├── server.rs          # 服务器管理（含 exec-batch/rm/java-restart）
    ├── cicd.rs            # CI/CD 部署（含 tools 工具检测）
    ├── database.rs        # 数据库 + Redis（无状态，直连 core）
    ├── log.rs             # 日志聚合器（含 context 上下文查看）
    ├── git.rs             # Git 仓库
    ├── mfa.rs             # MFA 管理
    ├── note.rs            # 笔记管理
    ├── accounting.rs      # 记账管理
    ├── weekly.rs          # 周报管理
    ├── nginx.rs           # Nginx 配置
    └── backup.rs          # 数据备份
```

## CLI 自动分发

## CLI 自动分发（双通道）

**通道 1 — pkg 安装 postinstall（默认安装）**：`build.sh` 构建 pkg 时把 CLI 与 skills 打进 app bundle 的 `Contents/Resources/_up_/`，安装时 postinstall 脚本：
- 复制 `_up_/target/release/stool` → `/usr/local/bin/stool`（分发失败会报错退出，不再静默）
- 复制 `_up_/skills/*` → `/usr/local/share/supertool/skills`
- 复制到用户目录：`~/.hermes/skills/`、`~/.claw/skills/`、`~/.trae-cn/skills/`（Trae IDE）、`~/.supertool/skills` → 符号链接
- 安装位置从 `$3`（pkg 安装目标）推导，兜底 `/Applications`（支持自定义安装卷）

**通道 2 — App 启动检测（dmg 安装/升级场景）**：App 启动时对比 `/usr/local/bin/stool` 与内置 CLI（`_up_/target/release/stool`）版本：
- 不一致 → 顶部横幅提示"一键更新"，osascript 管理员权限安装（`check_cli_version` / `install_cli` Tauri 命令）
- 同时静默同步内置 skills 到用户技能目录（`sync_user_skills`）

> 历史遗留：旧版本曾完全依赖 postinstall，dmg 用户 CLI 永远停在首装版本（如 4.2）。双通道已覆盖。
