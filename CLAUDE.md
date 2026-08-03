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

- **日志聚合（LogAggregator）搜索模式必须全量渲染**：日志行 `whitespace-pre-wrap break-all` 长行会换行，固定行高（24px）虚拟滚动会导致滚动时 `scrollHeight` 波动、浏览器钳制 `scrollTop` 造成回弹（"无法滑到底部"）。历史上两次尝试动态行高测量均失败回滚（`1536e7ce`、`35a263cf`）。搜索模式用 `renderedLines` 全量渲染 + 真实 DOM 定位跳转（`data-log-idx`），流式模式保留虚拟滚动。

- **`cargo fmt` may produce false positives** with edition 2024 — use `cargo check --workspace` zero errors as the standard
- **CLI is fully standalone** — no GUI dependency required, ~12MB binary
- **No UDS/HTTP communication** — everything goes through direct `supertool-core` function calls
- **Sensitive data** (SSH passwords, DB passwords) encrypted with AES-256-GCM, never exposed in CLI output
- **RequiresApproval** — production environment CLI operations require GUI approval; test/dev can bypass
- **Git workflow**: commit immediately after code changes, don't stack uncommitted modifications

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
