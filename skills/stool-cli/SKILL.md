---
name: stool-cli
category: devops
description: SuperTool `stool` CLI v3.0.0 — AI Agent 专属运维工具。纯 JSON over UDS 通信，零网络/Express 依赖，管理服务器、CI/CD 部署、数据库、日志和 Git 仓库。
trigger: 使用 stool 命令、排查 CLI 失败、添加新 CLI 命令、AI 运维操作
---

# SuperTool `stool` CLI v3.0.0

> AI Agent 专属运维工具 —— 人类用 GUI 配置密钥/参数，AI 用 CLI 执行部署/排查/运维操作。

## 核心概念

| 概念 | 说明 |
|------|------|
| **通信方式** | 纯 JSON over UDS (`~/.supertool/supertool.sock`)，零网络层/Express 依赖。`\n` 分隔的 JSON 对象 `{handler, params}`，代理/VPN 无法劫持 |
| **环境变量** | `SUPERTOOL_SOCKET`（可选，自定义 socket 路径） |
| **前置条件** | SuperTool GUI 必须运行（提供 UDS 监听）。除 `version`/`guide` 外所有命令都需要 socket 可用 |
| **输出格式** | 默认人类可读文本；加 `-j` 返回 JSON（所有 list/status 命令都支持） |
| **ID 完整性** | 所有 list 命令输出完整 36 位 UUID，可复制用于后续命令 |
| **敏感信息** | SSH 密码/DB 密码等敏感信息不暴露给 AI，GUI 中 AES-256-GCM 加密存储 |

## 安全管控矩阵

```
CLI 能做的                          CLI 不能做的（需 GUI 操作）
────────────────────────────────    ─────────────────────────────────
✅ 查看/列出所有资源                ❌ 服务器 requiresApproval=on 时：
✅ 普通命令执行 (exec)                不能 exec/health/diagnose
✅ 部署/回滚（无审核配置）           ❌ CICD requiresApproval=on 时：
✅ 数据库查询（无审核连接）            不能 deploy/rollback
✅ Redis 只读操作                    ❌ DB requiresApproval=on 时：
✅ 文件读取/目录列出/下载              不能 SQL 查询 & Redis 写操作
✅ Git 操作（status/log/pull/push）  ❌ 高危命令被正则拦截：
✅ 日志搜索/实时 tail                  rm -rf, kill -9, shutdown, mkfs,
                                      dd, iptables -F, chmod 777 /,
                                      curl|sh, wget|bash 等
                                     ❌ server exec-batch / rm / java-restart
                                      （已从 CLI 彻底移除，需 GUI 操作）
```

## 命令速查

### 📋 任务管理 (Todo)

```bash
stool todo add "任务文本" [-p high|medium|low] [-d 2024-12-31] [-t 标签] [--description 描述]
stool todo list [-c true|false] [-t 标签] [-l 50] [-j]
stool todo complete <id>              # 标记完成
stool todo uncomplete <id>            # 撤销完成
stool todo delete <id>
stool todo show <id> [-j]             # 详情
stool todo edit <id> [-t "新文本"] [-p high] [--due 日期] [-t 标签]
stool todo search "关键词" [-j]
stool todo stats [-j]                 # 统计（总数/完成/待办/高优）
stool todo clear                      # 清空已完成

# 子任务
stool subtask list <todo_id> [-j]
stool subtask add <todo_id> "文本"
stool subtask complete <sub_id>
stool subtask delete <sub_id>

# 项目
stool project list [-j]
stool project add "项目名" [-d "描述"]
stool project show <id> [-j]
stool project update <id> [-n "新名"] [--description 描述]
stool project delete <id>
stool project stats <id> [-j]    # 项目进度统计
stool project todos <id> [-j]    # 项目关联任务
```

### 🖥️ 服务器管理

```bash
stool server list [-j]           # 列出所有服务器（完整 UUID）
stool server add "名称" <host> [端口] [用户]
stool server test <id>           # 测试 SSH 连接
stool server exec <id> "命令" [--timeout 60]    # 执行远程命令
stool server health <id> [-j]    # 健康检查（磁盘/内存/CPU/Docker/进程）
stool server diagnose <id> [-j]  # 智能诊断（系统信息+Docker+错误日志）
stool server delete <id>

# 文件操作
stool server read <id> <路径>              # 读取远程文件内容
stool server ls <id> [--path /目录] [-j]   # 列出远程目录
stool server download <id> <远程路径> [--output 本地路径]  # 下载文件
stool server mkdir <id> <路径>             # 创建远程目录

# Java 进程管理（Spring Boot）
stool server java-ps <id> [-j]   # 查看 Java 进程（PID/端口/堆内存/运行时间）
```

> ⚠️ `server rm` / `server exec-batch` / `server java-restart` 已禁用 CLI，必须用 GUI 操作。

### 🚀 CI/CD 部署

```bash
stool cicd list [-j]                     # 列出所有部署配置
stool cicd status <project_id> [-j]      # 查看配置详情
stool cicd deploy <config_id> [--stream] [--watch]
#   --stream: SSE 实时流式输出（推荐，部署完自动退出）
#   --watch:  每 5 秒轮询状态直到完成（最长 10 分钟）
#   注：--stream 和 --watch 互斥，同时指定时 --stream 优先
stool cicd history <config_id> [-l 20] [--status success|failed|rolled_back|cancelled] [-j]
stool cicd step-logs <deploy_log_id> [-j]  # 某次部署的详细步骤日志
stool cicd rollback <config_id> <deploy_log_id>  # 回滚到指定版本
stool cicd cancel <config_id>            # 取消正在进行的部署
stool cicd modules <config_id> [-j]      # 查看部署模块列表
stool cicd logs <project_id> [-l 20]     # 部署日志（同 history，按项目查询）
```

### 🗄️ 数据库管理

```bash
stool db list [-j]                            # 列出已保存的连接
stool db disconnect <id>                              # 断开连接

# SQL 查询
stool db query -d <db_id> "SELECT ..." [-j]     # -j 返回 JSON 数组（带列名）
stool db tables -d <db_id> [--db 数据库名] [-j]  # 列出表
stool db databases -d <db_id> [-j]              # 列出数据库

# Redis 操作
stool db redis -d <db_id> keys "pattern"        # 搜索键（默认 *）
stool db redis -d <db_id> get <key>             # 获取值
stool db redis -d <db_id> type <key>            # 数据类型
stool db redis -d <db_id> ttl <key>             # 过期时间
stool db redis -d <db_id> h-get <key> <field>   # Hash 字段
stool db redis -d <db_id> h-get-all <key>       # 全部 Hash 字段
stool db redis -d <db_id> h-len <key>           # Hash 字段数
stool db redis -d <db_id> l-range <key> [start] [stop]  # List 范围
stool db redis -d <db_id> l-len <key>           # List 长度
stool db redis -d <db_id> s-members <key>       # Set 成员
stool db redis -d <db_id> s-card <key>          # Set 成员数
stool db redis -d <db_id> set <key> <value>     # 写操作（审核连接会被拒绝）
stool db redis -d <db_id> delete <key>          # 删除操作（审核连接会被拒绝）
```

### 📝 日志聚合器

```bash
stool log list [-j]                          # 列出日志预设
stool log search <preset_id> "关键词" [-l 50]  # 搜索日志（preset_id 可用序号 1,2,3...）
stool log tail <preset_id> [-l 100]          # 实时 tail（SSE 流式，Ctrl+C 退出）
stool log add "名称" --server-ids "id1,id2" --log-path /var/log/app.log [--log-type tail]
stool log delete <id>
```

### 🔧 Git 仓库管理

```bash
stool git list [-j]                          # 列出已注册仓库
stool git status --path <仓库路径> [-j]       # 仓库状态（分支/干净/领先/落后/变更文件）
stool git log --path <仓库路径> [-l 20] [-j]  # 提交历史
stool git branches --path <仓库路径> [-j]     # 分支列表
stool git pull --path <仓库路径>             # 拉取远程更新
stool git push --path <仓库路径>             # 推送到远程
stool git commit --path <仓库路径> -m "消息" [--files f1 f2]
stool git checkout --path <仓库路径> --branch <分支>
```

### 通用

```bash
stool version                                # 显示版本
stool guide                                  # 显示使用指南
```

## 🔄 AI Agent 标准工作流

### 部署 → 验证 → 回滚

```bash
# 1. 部署
stool cicd deploy my-config-id --stream

# 2. 验证
stool cicd history my-config-id -l 1 -j    # 检查部署结果
stool server health prod-server-id          # 检查服务器健康
stool log search prod-logs "ERROR" -l 30    # 搜索错误日志

# 3. 诊断（如有问题）
stool server diagnose prod-server-id        # 全面诊断
stool server exec prod-server-id "docker logs --tail 50 my-app"  # 容器日志

# 4. 回滚（如需要）
stool cicd rollback my-config-id last-good-deploy-id
stool cicd history my-config-id -l 3 -j     # 验证回滚结果
```

### 日常巡检

```bash
# 服务器状态
stool server list -j

# 高优先级任务
stool todo list -p high -j

# CI/CD 状态
stool cicd list -j
```

### 故障排查

```bash
# 1. 先看服务器健康
stool server health server-id -j

# 2. 看 Java 进程
stool server java-ps server-id -j

# 3. 查 Redis 缓存
stool db redis -d redis-id keys "session:*"
stool db redis -d redis-id get "my:key"

# 4. 查数据库
stool db query -d db-id "SELECT COUNT(*) FROM orders WHERE status='pending'"

# 5. 实时看日志
stool log tail app-logs -l 200
```

## ⚠️ 关键陷阱

1. **纯 UDS 通信** — CLI 仅通过 `~/.supertool/supertool.sock` 连接，GUI 必须运行。支持 `SUPERTOOL_SOCKET` 环境变量自定义路径
2. **Handler 命名规范** — CLI 子命令必须与 Electron `registerHandler` 键名 1:1 对应（`stool db list` → `db:list`）。两侧必须同步改名，且 `search_files` 确认旧名清零。命名规则：全小写 + `-` 分隔模块子功能（如 `log-presets:get-all`、`db:redis:get`），禁驼峰。`db:redis-*` 格式已废弃，统一为 `db:redis:*` 三层冒号分隔
3. **`-j` 是短别名**：`-j` 等价于 `--json`，所有 list/status 命令都支持
4. **高危命令拦截**：`server exec` 会拦截 `rm -rf`、`kill -9`、`shutdown`、`curl|sh` 等，触发后必须用 GUI 执行
5. **requiresApproval 三重拦截**：服务器/数据库/CICD 各自独立，开启后 CLI 对应操作被拒绝，错误信息会说明原因
6. **preset_id 智能解析**：`log search 1 "关键词"` 中的 `1` 会被自动解析为第 1 个预设的真实 UUID
7. **部署超时**：`--watch` 最长等待 10 分钟（5s × 120 次），超时后自动退出；`--stream` 无硬超时，等待服务端发 complete 事件。restart 脚本无超时限制，多服务并行启动也能正常等待完成
8. **server download**：通过 base64 编码传输二进制文件，自动保存到本地
9. **ID 不可截断**：所有 UUID 必须完整输出，截断后无法匹配后续命令。所有 list 命令（包括 `subtask list`、`project todos`、`git list`）必须输出 ID，否则命令链无法闭环
10. **CLI 不传冗余参数**：连接已在 GUI 配好，CLI 通过 `-d <db_id>` 引用即可。CLI 不应要求用户输入 host/port/password 等已保存的信息
11. **macOS DMG 打包**：交叉编译产物需 `cp` 到 `target/release/`，否则 `electron-builder` 打包旧缓存二进制

## 源码与构建

- **源码**: `~/WebstormProjects/todo-list-electron/cli/src/main.rs`（纯 UDS，零网络依赖）
- **传输层**: `cli/src/transport.rs`（`std::UnixStream` 手写 JSON 序列化/反序列化，无 HTTP 客户端）
- **服务端**: `electron/uds-api.ts`（`net.Server` + JSON Router，零 Express 依赖）
- **版本定义**: `cli/Cargo.toml` 中的 `version`，代码通过 `env!("CARGO_PKG_VERSION")` 动态读取
- **构建**: `cd cli && cargo build --release`
- **macOS 交叉编译**: `cargo build --target aarch64-apple-darwin --release`
- **CLI 自动安装**: App 启动时 `cli-installer.ts` 检测版本差异 → AppleScript 密码弹窗 → `sudo -S` 安装到 `/usr/local/bin/stool`
- **Skills 自动分发**: App 启动时 `installHermesSkills()` 比较源文件与 `~/.hermes/skills/stool-cli/SKILL.md` 的 mtime，源文件更新后自动覆盖（无需授权）
