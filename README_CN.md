# SuperTool

**中文** | [English](README.md)

跨平台桌面运维管理工具 —— 基于 Tauri v2 + Vue 3 + Rust 共享库架构，统一管理服务器、CI/CD 部署、数据库、日志、Git 仓库，并**集成 AI Agent**。

## 特性

### 核心运维

- **待办事项**: 个人待办清单，支持优先级、截止日期、子任务、周报生成（首页）
- **项目管理**: 多项目追踪，Git 集成，部署历史
- **服务器管理**: SSH 连接、健康检查、命令执行、文件操作
- **数据库管理**: 直连查询/编辑 MySQL、PostgreSQL、Redis，连接预设
- **CI/CD 部署**: 自动化构建部署流水线、回滚、历史追踪
- **日志聚合**: 多服务器日志搜索、实时 tail、预设配置
- **Git 管理**: 仓库状态、提交历史、分支管理
- **Nginx 管理**: 配置预设、拉取/测试/部署到多服务器
- **VPN 管理**: OpenVPN/WireGuard 隧道管理、内网穿透

### 个人工具

- **笔记**: 笔记管理，支持分组和搜索
- **记账本**: 收支记录、分类、预算、统计
- **MFA**: TOTP 验证器，生成验证码
- **周报**: 从待办完成和项目提交自动生成周报
- **磁盘清理**: 查找并清理大文件、旧日志、缓存目录
- **数据备份**: 导出/导入所有数据（SQLite + 配置）

### AI Agent

- **Hermes 对话**: 内嵌 AI 助手，流式响应、工具调用可视化
- **实时任务面板**: 右侧边栏显示 AI 执行过程中的 todo 进度
- **文件附件**: 选择文件/文件夹/Git 仓库附加到对话
- **会话管理**: 多会话、标题编辑、导出、搜索

### 基础设施

- **告警管理**: 配置和查看系统告警
- **局域网发现**: 自动发现局域网内服务器
- **开发工具**: 调试工具、API 测试、串口工具
- **设置**: 应用配置、主题切换、语言选择

### 架构亮点

- **统一架构**: `core` / `cli` / `tauri` 三 crate workspace，业务逻辑全部在 `supertool-core`
- **独立 CLI**: `stool` CLI（~12MB）直连核心库，可独立部署在服务器
- **现代前端**: Vue 3 + TypeScript + Tailwind CSS v4 + daisyUI
- **国际化**: 中文 / 英文
- **加密存储**: AES-256-GCM 加密 SSH 密码、数据库密码

## 架构

```
supertool/
├── core/          # supertool-core 共享库（单一事实来源）
│   └── src/
│       ├── db/         # SQLite 数据访问层
│       ├── encryption/ # AES-256-GCM 加密
│       └── logic/      # 业务逻辑：ssh, cicd, git, openvpn, wireguard, nginx...
├── cli/           # stool CLI（直连 core，可独立运行）
│   └── src/
│       ├── main.rs       # clap CLI 入口
│       ├── commands/     # 所有子命令（todo, server, cicd, db, log, git）
│       ├── runtime.rs    # CliRuntime 生命周期管理
│       └── types.rs      # CLI 类型定义
├── tauri/         # Tauri GUI（同样直连 core）
│   ├── src/            # Rust IPC 命令层
│   ├── tauri.conf.json # Tauri 配置
│   └── ...
└── src/           # Vue 3 前端
    ├── views/          # 页面组件
    ├── components/     # 可复用组件
    └── ...
```

### 数据流向

```
stool CLI ──┐
            ├──▶ supertool-core (CoreService) ──▶ SQLite / SSH / MySQL / Redis / ...
Tauri GUI ──┘
```

CLI 和 GUI 都直连 `supertool-core`，**零 UDS/HTTP 中间层**。

## 快速开始

### 前置要求

- [Node.js](https://nodejs.org/) (v18+) + [pnpm](https://pnpm.io/)
- [Rust](https://rustup.rs/) 工具链（edition 2024）

### 安装

```bash
git clone git@git.code.tencent.com:fufengyuan/supertool.git
cd supertool
pnpm install
```

### 开发

```bash
# 启动 Tauri 开发环境（前端 + 后端）
pnpm tauri dev

# 单独构建 CLI（输出 cli/target/release/stool，~12MB）
cd cli && cargo build --release

# 构建生产版本
pnpm tauri build
```

## 开发命令

### 前端

```bash
pnpm dev          # Vue 开发服务器 (端口 1420)
pnpm build        # 生产构建
pnpm preview      # 预览生产版本
```

### Tauri

```bash
pnpm tauri dev    # 开发环境
pnpm tauri build  # 生产构建
pnpm tauri        # Tauri CLI
```

### Rust / CLI

```bash
# 全 workspace 编译检查
cargo check --workspace

# CLI 发布构建
cd cli && cargo build --release

# Tauri 发布构建
cd tauri && cargo build --release
```

### 代码质量

```bash
pnpm lint         # oxlint
pnpm lint:fix     # oxlint --fix
pnpm format       # Prettier
vue-tsc --noEmit  # TypeScript 类型检查
```

## Git 提交规范（自动版本号）

遵循 **Conventional Commits** 规范，版本号自动更新：

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
- 新环境克隆后运行 `./scripts/init-hooks.sh` 或 `pnpm install`

## 项目结构

```
supertool/
├── Cargo.toml              # Workspace 配置 (core, cli, tauri)
├── core/                   # supertool-core 共享库
│   ├── Cargo.toml          # version = "4.2.0", edition = "2024"
│   └── src/
│       ├── lib.rs              # 库入口
│       ├── service.rs          # CoreService 主入口
│       ├── db/                 # SQLite 数据访问
│       │   ├── database.rs         # 主数据库操作
│       │   ├── servers.rs          # 服务器 CRUD
│       │   ├── cicd.rs             # CI/CD 配置
│       │   ├── projects.rs         # 项目管理
│       │   ├── openvpn.rs          # OpenVPN 配置
│       │   ├── wireguard.rs        # WireGuard 配置
│       │   └── lan.rs              # 局域网发现
│       ├── encryption/         # AES-256-GCM 加密
│       └── logic/              # 业务逻辑
│           ├── data_dir.rs         # 数据目录解析
│           ├── ssh.rs              # SSH 连接管理
│           ├── cicd_deploy.rs      # 部署引擎
│           ├── git.rs              # Git 操作
│           ├── openvpn.rs          # OpenVPN 隧道
│           ├── wireguard.rs        # WireGuard 隧道
│           ├── nginx.rs            # Nginx 配置
│           └── log_sanitizer.rs    # 日志脱敏
├── cli/                    # stool CLI
│   ├── Cargo.toml          # version = "4.2.0"
│   └── src/
│       ├── main.rs             # 入口 + 命令注册
│       ├── commands/           # todo, server, cicd, db, log, git
│       ├── runtime.rs          # CliRuntime
│       ├── types.rs            # clap 类型定义
│       ├── output.rs           # 输出格式化
│       ├── utils.rs            # 工具函数
│       └── guide.rs            # 使用指南
├── tauri/                  # Tauri GUI
│   ├── Cargo.toml          # version = "4.2.0"
│   ├── tauri.conf.json     # Tauri 配置
│   └── src/
│       ├── main.rs             # Tauri 入口
│       ├── lib.rs              # Tauri 命令库
│       └── commands/           # IPC 命令
└── src/                    # Vue 3 前端
    ├── main.ts             # Vue 入口
    ├── App.vue
    ├── router/             # 路由配置
    ├── views/              # 页面
    ├── components/         # 组件
    ├── layouts/            # 布局
    ├── stores/             # Pinia 状态
    ├── utils/              # 工具函数
    ├── locales/            # i18n (zh-CN, en-US)
    └── assets/             # 静态资源
```

## 技术栈

### Rust 后端

- **Rust** edition 2024
- **Tauri v2** — 跨平台桌面框架
- **rusqlite** — SQLite 嵌入式数据库
- **mysql_async** / **tokio-postgres** — 外部数据库连接
- **redis** — Redis 客户端
- **ssh2** — SSH 连接管理
- **aes-gcm** — AES-256-GCM 加密
- **tokio** — 异步运行时

### 前端

- **Vue 3** — Composition API + `<script setup>`
- **TypeScript** — 严格模式
- **Vite** — 构建工具
- **Tailwind CSS v4** + **daisyUI** — 样式
- **Pinia** — 状态管理
- **Vue i18n** — 国际化
- **Vue Router** — 路由
- **pnpm** — 包管理器

### 开发工具

- **oxlint** — 快速 lint
- **Prettier** — 代码格式化
- **vue-tsc** — TS 类型检查

## 数据目录

所有运行时数据存储在 `~/.supertool/`：

```
~/.supertool/
├── supertool.db        # SQLite 主数据库
├── logs/               # 应用日志
├── tmp/                # 临时文件
├── backups/            # 备份文件
├── cicd/               # CI/CD 部署产物
└── cli/                # CLI 二进制（自动安装到 /usr/local/bin/）
```

可通过 `~/.supertool_dir` 文件自定义路径。

## CLI 使用

CLI 是 AI Agent 专用运维工具，15+ 命令覆盖全部功能。详见 [skills/stool-cli/SKILL.md](skills/stool-cli/SKILL.md)。

```bash
stool version                 # 查看版本

# 待办 & 项目
stool todo list               # 待办列表
stool todo add "任务名称"     # 添加待办
stool subtask list <todo_id>  # 子任务列表
stool project list            # 项目列表

# 服务器 & 数据库
stool server list -j          # 服务器列表 (JSON)
stool server health <id>      # 健康检查
stool db query -d <id> "SQL"  # 数据库查询
stool db list                 # 数据库连接列表

# CI/CD & Git
stool cicd deploy <id> --stream  # 流式部署
stool cicd rollback <id>         # 回滚
stool git status <repo_id>       # Git 状态

# 日志 & Nginx
stool log search <preset> "ERROR" -l 30  # 日志搜索
stool nginx pull <preset>               # 拉取 nginx 配置

# 个人工具
stool note list               # 笔记列表
stool accounting list         # 记账记录
stool mfa list                # MFA 密钥
stool mfa code <id>           # 生成 TOTP 验证码
stool weekly generate         # 生成周报
stool backup export           # 导出所有数据
```

## 贡献

1. Fork 仓库
2. 创建分支：`git checkout -b feature/xxx`
3. 提交更改：`git commit -m 'feat: xxx'`
4. 推送：`git push origin feature/xxx`
5. 提交 Pull Request

## 许可证

MIT License — 详见 [LICENSE](LICENSE)
