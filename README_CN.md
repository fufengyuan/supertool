# SuperTool

**中文** | [English](README.md)

跨平台桌面运维管理工具 —— 基于 Tauri v2 + Vue 3 + Rust 共享库架构，统一管理服务器、CI/CD 部署、数据库、日志和 Git 仓库。

## 特性

- **统一架构**: `core` / `cli` / `tauri` 三 crate workspace，业务逻辑全部收敛至 `supertool-core`
- **CLI 独立运行**: `stool` CLI 直连 core 共享库（~12MB），无需 GUI，可独立部署在服务器
- **现代前端**: Vue 3 + TypeScript + Tailwind CSS v4 + daisyUI
- **多数据库支持**: MySQL、PostgreSQL、Redis 直连查询与管理
- **SSH 管理**: 远程服务器连接、命令执行、文件操作、健康检查
- **CI/CD 部署**: 自动化构建部署、回滚、部署历史追踪
- **日志聚合**: 多服务器日志搜索与实时 tail
- **Git 管理**: 仓库状态、提交历史、分支管理
- **OpenVPN / WireGuard**: 内网穿透与虚拟组网
- **国际化**: 中文 / 英文
- **加密存储**: AES-256-GCM 加密敏感信息（SSH 密码、DB 密码）

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

## 项目结构

```
supertool/
├── Cargo.toml              # Workspace 配置 (core, cli, tauri)
├── core/                   # supertool-core 共享库
│   ├── Cargo.toml          # version = "4.0.0", edition = "2024"
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
│   ├── Cargo.toml          # version = "4.0.0"
│   └── src/
│       ├── main.rs             # 入口 + 命令注册
│       ├── commands/           # todo, server, cicd, db, log, git
│       ├── runtime.rs          # CliRuntime
│       ├── types.rs            # clap 类型定义
│       ├── output.rs           # 输出格式化
│       ├── utils.rs            # 工具函数
│       └── guide.rs            # 使用指南
├── tauri/                  # Tauri GUI
│   ├── Cargo.toml          # version = "4.0.0"
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

CLI 是 AI Agent 专用运维工具，支持服务器管理、CI/CD 部署、数据库查询、日志搜索等。详见 [skills/stool-cli/SKILL.md](skills/stool-cli/SKILL.md)。

```bash
# 查看版本
stool version

# 服务器列表
stool server list -j

# 部署
stool cicd deploy <config_id> --stream

# 数据库查询
stool db query -d <db_id> "SELECT * FROM users LIMIT 10"

# 日志搜索
stool log search <preset_id> "ERROR" -l 30
```

## 贡献

1. Fork 仓库
2. 创建分支：`git checkout -b feature/xxx`
3. 提交更改：`git commit -m 'feat: xxx'`
4. 推送：`git push origin feature/xxx`
5. 提交 Pull Request

## 许可证

MIT License — 详见 [LICENSE](LICENSE)
