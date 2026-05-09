# SuperTool

[中文文档](README_CN.md) | **English**

Cross-platform desktop operations management tool — built on Tauri v2 + Vue 3 + Rust shared library architecture, unifying server management, CI/CD deployment, databases, logs, and Git repositories.

## Features

- **Unified Architecture**: `core` / `cli` / `tauri` three-crate workspace, all business logic consolidated into `supertool-core`
- **Standalone CLI**: `stool` CLI directly links to the core shared library (~12MB), no GUI required, deployable on servers independently
- **Modern Frontend**: Vue 3 + TypeScript + Tailwind CSS v4 + daisyUI
- **Multi-Database Support**: MySQL, PostgreSQL, Redis direct query and management
- **SSH Management**: Remote server connections, command execution, file operations, health checks
- **CI/CD Deployment**: Automated build/deploy, rollback, deployment history tracking
- **Log Aggregation**: Multi-server log search and real-time tail
- **Git Management**: Repository status, commit history, branch management
- **OpenVPN / WireGuard**: Intranet穿透 and virtual networking
- **Internationalization**: Chinese / English
- **Encrypted Storage**: AES-256-GCM encryption for sensitive data (SSH passwords, DB passwords)

## Architecture

```
supertool/
├── core/          # supertool-core shared library (single source of truth)
│   └── src/
│       ├── db/         # SQLite data access layer
│       ├── encryption/ # AES-256-GCM encryption
│       └── logic/      # Business logic: ssh, cicd, git, openvpn, wireguard, nginx...
├── cli/           # stool CLI (direct core access, standalone)
│   └── src/
│       ├── main.rs       # clap CLI entry point
│       ├── commands/     # All subcommands (todo, server, cicd, db, log, git)
│       ├── runtime.rs    # CliRuntime lifecycle management
│       └── types.rs      # CLI type definitions
├── tauri/         # Tauri GUI (also direct core access)
│   ├── src/            # Rust IPC command layer
│   ├── tauri.conf.json # Tauri config
│   └── ...
└── src/           # Vue 3 frontend
    ├── views/          # Page components
    ├── components/     # Reusable components
    └── ...
```

### Data Flow

```
stool CLI ──┐
            ├──▶ supertool-core (CoreService) ──▶ SQLite / SSH / MySQL / Redis / ...
Tauri GUI ──┘
```

Both CLI and GUI directly link to `supertool-core`, **zero UDS/HTTP middleware**.

## Quick Start

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+) + [pnpm](https://pnpm.io/)
- [Rust](https://rustup.rs/) toolchain (edition 2024)

### Installation

```bash
git clone git@git.code.tencent.com:fufengyuan/supertool.git
cd supertool
pnpm install
```

### Development

```bash
# Start Tauri development environment (frontend + backend)
pnpm tauri dev

# Build CLI standalone (output cli/target/release/stool, ~12MB)
cd cli && cargo build --release

# Production build
pnpm tauri build
```

## Development Commands

### Frontend

```bash
pnpm dev          # Vue dev server (port 1420)
pnpm build        # Production build
pnpm preview      # Preview production build
```

### Tauri

```bash
pnpm tauri dev    # Development environment
pnpm tauri build  # Production build
pnpm tauri        # Tauri CLI
```

### Rust / CLI

```bash
# Full workspace compilation check
cargo check --workspace

# CLI release build
cd cli && cargo build --release

# Tauri release build
cd tauri && cargo build --release
```

### Code Quality

```bash
pnpm lint         # oxlint
pnpm lint:fix     # oxlint --fix
pnpm format       # Prettier
vue-tsc --noEmit  # TypeScript type checking
```

## Project Structure

```
supertool/
├── Cargo.toml              # Workspace config (core, cli, tauri)
├── core/                   # supertool-core shared library
│   ├── Cargo.toml          # version = "4.0.0", edition = "2024"
│   └── src/
│       ├── lib.rs              # Library entry
│       ├── service.rs          # CoreService main entry
│       ├── db/                 # SQLite data access
│       │   ├── database.rs         # Main database operations
│       │   ├── servers.rs          # Server CRUD
│       │   ├── cicd.rs             # CI/CD config
│       │   ├── projects.rs         # Project management
│       │   ├── openvpn.rs          # OpenVPN config
│       │   ├── wireguard.rs        # WireGuard config
│       │   └── lan.rs              # LAN discovery
│       ├── encryption/         # AES-256-GCM encryption
│       └── logic/              # Business logic
│           ├── data_dir.rs         # Data directory resolution
│           ├── ssh.rs              # SSH connection management
│           ├── cicd_deploy.rs      # Deployment engine
│           ├── git.rs              # Git operations
│           ├── openvpn.rs          # OpenVPN tunnel
│           ├── wireguard.rs        # WireGuard tunnel
│           ├── nginx.rs            # Nginx config
│           └── log_sanitizer.rs    # Log sanitization
├── cli/                    # stool CLI
│   ├── Cargo.toml          # version = "4.0.0"
│   └── src/
│       ├── main.rs             # Entry + command registration
│       ├── commands/           # todo, server, cicd, db, log, git
│       ├── runtime.rs          # CliRuntime
│       ├── types.rs            # clap type definitions
│       ├── output.rs           # Output formatting
│       ├── utils.rs            # Utility functions
│       └── guide.rs            # Usage guide
├── tauri/                  # Tauri GUI
│   ├── Cargo.toml          # version = "4.0.0"
│   ├── tauri.conf.json     # Tauri config
│   └── src/
│       ├── main.rs             # Tauri entry
│       ├── lib.rs              # Tauri command library
│       └── commands/           # IPC commands
└── src/                    # Vue 3 frontend
    ├── main.ts             # Vue entry
    ├── App.vue
    ├── router/             # Router config
    ├── views/              # Pages
    ├── components/         # Components
    ├── layouts/            # Layouts
    ├── stores/             # Pinia stores
    ├── utils/              # Utilities
    ├── locales/            # i18n (zh-CN, en-US)
    └── assets/             # Static assets
```

## Technology Stack

### Rust Backend

- **Rust** edition 2024
- **Tauri v2** — Cross-platform desktop framework
- **rusqlite** — SQLite embedded database
- **mysql_async** / **tokio-postgres** — External database connections
- **redis** — Redis client
- **ssh2** — SSH connection management
- **aes-gcm** — AES-256-GCM encryption
- **tokio** — Async runtime

### Frontend

- **Vue 3** — Composition API + `<script setup>`
- **TypeScript** — Strict mode
- **Vite** — Build tool
- **Tailwind CSS v4** + **daisyUI** — Styling
- **Pinia** — State management
- **Vue i18n** — Internationalization
- **Vue Router** — Routing
- **pnpm** — Package manager

### Development Tools

- **oxlint** — Fast linting
- **Prettier** — Code formatting
- **vue-tsc** — TS type checking

## Data Directory

All runtime data stored in `~/.supertool/`:

```
~/.supertool/
├── supertool.db        # SQLite main database
├── logs/               # Application logs
├── tmp/                # Temporary files
├── backups/            # Backup files
├── cicd/               # CI/CD deployment artifacts
└── cli/                # CLI binary (auto-installed to /usr/local/bin/)
```

Customizable via `~/.supertool_dir` file.

## CLI Usage

The CLI is an AI Agent operations tool supporting server management, CI/CD deployment, database queries, log search, and more. See [skills/stool-cli/SKILL.md](skills/stool-cli/SKILL.md) for details.

```bash
# Check version
stool version

# List servers
stool server list -j

# Deploy
stool cicd deploy <config_id> --stream

# Database query
stool db query -d <db_id> "SELECT * FROM users LIMIT 10"

# Log search
stool log search <preset_id> "ERROR" -l 30
```

## Contributing

1. Fork the repository
2. Create a branch: `git checkout -b feature/xxx`
3. Commit changes: `git commit -m 'feat: xxx'`
4. Push: `git push origin feature/xxx`
5. Open a Pull Request

## License

MIT License — see [LICENSE](LICENSE)
