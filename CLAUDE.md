# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**SuperTool** — Cross-platform desktop operations management tool. Three-crate Rust workspace (`core`, `cli`, `tauri`) with Vue 3 frontend. CLI directly links to `supertool-core` shared library (zero UDS/HTTP).

## Development Commands

### Frontend / Tauri

```bash
pnpm dev                    # Vue dev server (port 1420)
pnpm build                  # Build Vue frontend (vue-tsc + vite)
pnpm tauri dev              # Tauri dev environment (frontend + backend)
pnpm lint                   # Run oxlint
pnpm lint:fix               # oxlint with auto-fix
pnpm format                 # Prettier
vue-tsc --noEmit            # TypeScript type checking

# 打包构建（使用 build.sh）
pnpm build:app              # CLI + Tauri 构建（不打包，native arch）
pnpm build:app:arm64        # CLI + Tauri 构建（arm64）
pnpm build:app:x64          # CLI + Tauri 构建（x64）
pnpm build:app:universal    # CLI + Tauri 构建（macOS universal）
pnpm build:pkg              # CLI + Tauri + 平台打包（自动检测 OS 生成 dmg/pkg/deb/exe）
pnpm build:pkg:arm64        # 同上，arm64
pnpm build:pkg:x64          # 同上，x64
pnpm build:pkg:universal    # 同上，macOS universal
```

### Rust / CLI

```bash
cargo check --workspace     # Full workspace compilation check (zero errors required)
cargo build --release       # Build all workspace crates
cd cli && cargo build --release       # CLI standalone (~12MB)
cd tauri && cargo build --release     # Tauri binary
```

### Testing

- No test framework is currently configured.

## Architecture

### Workspace Structure

```
Cargo.toml          # [workspace] members = ["core", "cli", "tauri"]
core/               # supertool-core — shared library (single source of truth)
cli/                # stool CLI — directly links to core, standalone
tauri/              # Tauri GUI — directly links to core, Vue 3 frontend
src/                # Vue 3 frontend source (tauri window content)
```

### Data Flow

```
stool CLI ──┐
            ├──▶ supertool-core (CoreService) ──▶ SQLite / SSH / MySQL / Redis / ...
Tauri GUI ──┘
```

Both CLI and GUI directly link to `supertool-core`. **No UDS, no HTTP middleware.**

### Core Library (`core/`)

```
core/src/
├── lib.rs              # Re-exports: db, encryption, logic
├── service.rs          # CoreService — main entry point for all operations
├── db/                 # SQLite data access (database, servers, cicd, projects, openvpn, wireguard, lan)
├── encryption/         # AES-256-GCM encryption/decryption
└── logic/              # Business logic
    ├── data_dir.rs         # Data directory resolution (~/.supertool or ~/.supertool_dir)
    ├── ssh.rs              # SSH connection management
    ├── cicd_deploy.rs      # Deployment engine
    ├── git.rs              # Git operations
    ├── openvpn.rs          # OpenVPN tunnel
    ├── wireguard.rs        # WireGuard tunnel
    ├── nginx.rs            # Nginx config
    └── log_sanitizer.rs    # Log desensitization
```

### CLI (`cli/`)

```
cli/src/
├── main.rs             # clap entry point + command dispatch
├── commands/           # Subcommands: todo, server, cicd, db, log, git
├── runtime.rs          # CliRuntime — manages DB connection and CoreService
├── types.rs            # clap CLI type definitions
├── output.rs           # Output formatting (print_json, print_error, print_success)
├── utils.rs            # shell_quote, is_dangerous_command, format_size
└── guide.rs            # Usage guide output
```

### Tauri GUI (`tauri/`)

```
tauri/
├── src/
│   ├── main.rs             # Tauri app entry
│   ├── lib.rs              # Tauri command library
│   └── commands/           # IPC command handlers (all use CoreService)
├── tauri.conf.json         # Tauri configuration
└── Cargo.toml              # version = "4.0.0", edition = "2024"
```

### Frontend (`src/`)

```
src/
├── main.ts             # Vue app entry (Pinia, i18n, router)
├── App.vue
├── router/             # Vue Router config
├── views/              # Page components
├── components/         # Reusable components
├── layouts/            # Layout components
├── stores/             # Pinia state management
├── utils/              # Utilities (i18n, settings, theme)
├── locales/            # zh-CN, en-US translations
└── assets/             # Static resources (CSS)
```

## Key Configuration

- **Versions**: All crates unified to `version = "4.0.0"`, `edition = "2024"`
- **Data Directory**: `~/.supertool/` (resolved via `data_dir::resolve_data_dir()`, supports `~/.supertool_dir` override)
- **Database**: SQLite at `~/.supertool/supertool.db`
- **Package Manager**: pnpm
- **Vite**: Fixed port 1420 for Tauri dev

## Important Notes

- **开发工具模块（`src/views/devtools/`）设计约定**：2026-08 完成 UI 重构——`DevTools.vue` 为网格卡片首页（分类分区 + 大图标卡片 + 顶部搜索，无侧栏/收藏/最近/折叠菜单）；所有工具页必须套用 `components/ToolPage.vue` 统一外壳（返回按钮 + 图标 + 标题 + 描述，`emit('back')` 返回列表）；应用型工具（如 ApiDebugger）用 `no-scroll` prop 自管理滚动。工具页统一布局模式：`bg-base-100 border rounded-xl` 卡片分区、`btn-primary/btn-outline/btn-ghost` 按钮分级、输入/输出双栏 `textarea font-mono bg-base-200/60`。新增工具需在 `DevToolRegistry.ts` 注册（含 id/icon/category/offline/keywords）并在 `DevTools.vue` 的 `toolComponents` 挂载。`useDevTools.ts`（收藏/最近/折叠）已弃用但被 `ToolCommandPalette.vue` 引用，勿删。历史上部分工具使用过失效 CSS 类（`tool-input`/`result-card`/`hash-result-item` 等，无定义），重构时已全部替换为 tailwind/daisyUI 类。

- **日志聚合（LogAggregator）搜索模式必须全量渲染**：日志行 `whitespace-pre-wrap break-all` 长行会换行，固定行高（24px）虚拟滚动会导致滚动时 `scrollHeight` 波动、浏览器钳制 `scrollTop` 造成回弹（"无法滑到底部"）。历史上两次尝试动态行高测量均失败回滚（`1536e7ce`、`35a263cf`）。搜索模式用 `renderedLines` 全量渲染 + 真实 DOM 定位跳转（`data-log-idx`），流式模式保留虚拟滚动。
- **日志虚拟滚动行高策略**：行高固定的虚拟滚动（spacer = 行数 × 固定高度）在 `whitespace-pre-wrap` 长行下必然回弹。离线日志弹窗（fullLog，大文件无法全量渲染）用"逐行真实行高测量 + 前缀和二分反推"（`fullLogHeightPrefix`/`fullLogRowAtScrollTop`/`fullLogPrefixAt`），行高数组挂在 session（`rowHeights`）上，滚动渐进收敛。
- **实时/连接日志智能吸底约定**：DeployPanel 实时日志、OpenVPNManager/VPNManager 连接日志都实现了"用户上翻暂停自动跟随 + 回到底部按钮"（`xxxUserScrolledUp` + `@scroll` 判断）。注意：程序化赋值 `scrollTop` 不触发 scroll 事件，吸底/回底部后必须手动重置标志，否则按钮不消失、自动吸底被永久跳过（曾因漏重置被 review 拦截）。

- **`cargo fmt` may produce false positives** with edition 2024 — use `cargo check --workspace` zero errors as the standard
- **CLI is fully standalone** — no GUI dependency required, ~12MB binary
- **No UDS/HTTP communication** — everything goes through direct `supertool-core` function calls
- **Sensitive data** (SSH passwords, DB passwords) encrypted with AES-256-GCM, never exposed in CLI output
- **RequiresApproval** — production environment CLI operations require GUI approval; test/dev can bypass
- **Git workflow**: commit immediately after code changes, don't stack uncommitted modifications

## Iteration Plans

- [AI 工具增强迭代计划](docs/ai-tooling-iteration-plan.md) — 定位 AI Agent 运维工具箱：CLI 输出结构化 → MCP server → core 能力扩展 → 审批闭环 + 审计，GUI 只做辅助

## Git Commit 规范 (自动版本号)

**遵循 Conventional Commits 规范，版本号自动更新**：

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
- 新环境首次克隆后运行 `./scripts/init-hooks.sh` 或 `pnpm install`

## Conventions（2026-08 补充）

- **core todo 更新是 PATCH 语义**：`core/src/logic/todo.rs` 的 `update_todo` 只更新请求中提供的字段（含 `orderNum`），不会误清空未传字段；前端 `updateTodo(todo)` 传部分字段即可。
- **@ts-nocheck 状态**：全项目已清零（2026-08 批量清理完成），仅 `src/utils/tauri-api.ts` 保留（136 个历史类型错误，暂不清理）；其余文件已全部移除。改 tauri-api 时**必须同步实现与接口声明**（TauriAPI interface），否则其他文件 vue-tsc 会报错。
- **清理 @ts-nocheck 踩过的坑**（后续改这些模块注意）：① tauri-api 存在同名方法两组实现（如 dbRedisStreams 754 行组与 2609 行组），**后者覆盖前者**，签名不一致导致参数失效（pattern/group 被忽略）——改接口声明时必须对照 `getTauriAPI` 对象里最后出现的实现；② 接口声明返回 `() => void` 但实现 `return listen(...)` 返回 Promise<UnlistenFn>（onTerminalClose/onLogs*），调用方把 Promise 当函数调用会 TypeError——接口声明应与实现一致用 `Promise<UnlistenFn>`；③ 表单对象直接展开传给后端（`{...serverForm.value}`）会带 tagsInput 等多余字段——用白名单重建；④ 日志行/服务器组等类型用本地内联接口时与 types.ts 不同步（缺 html/sortKey/parentId 字段）——优先用 types.ts 类型。
- **LAN 事件约定**：发送事件用 `tauri-api` 的 `lanBroadcast*` 系列方法（走 `lan_broadcast_*` 后端命令，参数为 JSON 字符串）；`on*` 系列只用于监听（`listen` 事件）。误用 `onXxx(todoId, editor)` 发送会导致运行时 TypeError。
- **lint 配置**（.oxlintrc.json）：`eqeqeq` 允许 `!= null` 惯用法（`null: ignore`）、`no-unused-vars` 豁免 `_` 前缀与 catch 参数；全项目 lint 当前 0 错误。

- **Tauri 中原生 `confirm()`/`prompt()` 不弹窗**（静默返回 false/null）：删除/覆盖确认必须用 `@tauri-apps/plugin-dialog` 的 `confirm()` 或组件内自定义弹层（参照 KanbanBoard 的 pendingConfirm/pendingBlock）；项目已有 plugin-dialog ^2.7.0 依赖。
- **后端 Tauri 命令参数名必须与前端调用一致**（serde 反序列化不映射别名）：周报模块曾因前端传 `params`/`startDate` 而后端要 `limit`/`weekStart` 导致整条链路失效（历史为空/字段空白/详情打不开）。新增命令前核对 `tauri/src/main.rs` 注册签名与 `tauri-api.ts` 调用参数。
- **Tauri 命令失败返回 `{success:false,error}` 不抛异常**：调用方必须显式检查 `success === false`（DBManager 曾把语法错误记为成功）。批量/写操作前确认返回结构。
- **UI 状态与后端字段语义对齐**：`types.ts` 的接口是准绳（Notes 的 pinned、WeeklyReport 的 weekStart/weekEnd 曾与本地重复接口冲突导致类型错误与字段丢失），改动前端类型先查 `core/src/logic/*.rs` 实际字段。
