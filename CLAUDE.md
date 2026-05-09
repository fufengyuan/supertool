# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**SuperTool** — Cross-platform desktop operations management tool. Three-crate Rust workspace (`core`, `cli`, `tauri`) with Vue 3 frontend. CLI directly links to `supertool-core` shared library (zero UDS/HTTP).

## Development Commands

### Frontend / Tauri

```bash
pnpm dev                    # Vue dev server (port 1420)
pnpm tauri dev              # Tauri dev environment (frontend + backend)
pnpm tauri build            # Production build (.deb/.AppImage/.dmg)
pnpm build                  # Build Vue frontend for production
pnpm lint                   # Run oxlint
pnpm lint:fix               # oxlint with auto-fix
pnpm format                 # Prettier
vue-tsc --noEmit            # TypeScript type checking
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

- **`cargo fmt` may produce false positives** with edition 2024 — use `cargo check --workspace` zero errors as the standard
- **CLI is fully standalone** — no GUI dependency required, ~12MB binary
- **No UDS/HTTP communication** — everything goes through direct `supertool-core` function calls
- **Sensitive data** (SSH passwords, DB passwords) encrypted with AES-256-GCM, never exposed in CLI output
- **RequiresApproval** — production environment CLI operations require GUI approval; test/dev can bypass
- **Git workflow**: commit immediately after code changes, don't stack uncommitted modifications
