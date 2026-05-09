# Code Duplication Audit: tauri/src/ vs core/src/

**Date:** 2026-05-09
**Scope:** Full comparison between `tauri/src/` and `core/src/`

---

## 1. Exact Duplicates

### ✅ `tauri/src/db/` ↔ `core/src/db/` — IDENTICAL (10 files)

All 10 files are **byte-for-byte identical** (`diff -rq` returned zero differences):

| File | Status |
|------|--------|
| `cicd.rs` | ✅ Identical |
| `cicd_tables.rs` | ✅ Identical |
| `database.rs` | ✅ Identical |
| `lan.rs` | ✅ Identical |
| `mod.rs` | ✅ Identical |
| `nginx.rs` | ✅ Identical |
| `openvpn.rs` | ✅ Identical |
| `projects.rs` | ✅ Identical |
| `servers.rs` | ✅ Identical |
| `wireguard.rs` | ✅ Identical |

---

## 2. Diverged Files

### 🟡 `tauri/src/encryption.rs` vs `core/src/encryption.rs`
- **Diff:** 1 line (line 12)
- **Tauri:** `crate::core::data_dir::encryption_key_path()`
- **Core:** `crate::logic::data_dir::encryption_key_path()`
- **Cause:** Path resolution differs because they're in different module trees

### 🟡 `tauri/src/core/mod.rs` (2,938 lines) vs `core/src/logic/mod.rs` (3,381 lines)
**Core has ~443 MORE lines** — core is ahead with new features:

1. **Module declarations differ:**
   - Tauri has: `pub mod lan;`, `pub mod system_logger;`, `pub mod tray_notification;`
   - Core has comment: `// 注: lan, system_logger, tray_notification 属于 GUI 专属，已从 core 移除`

2. **Core added ~370 lines** at end of mod.rs: new CI/CD deployment logic with approval workflow, server resolution, and module config building

3. **Doc comment differs:** "UDS handlers" (Tauri) vs "Tauri commands and CLI" (Core)

### 🟡 `tauri/src/core/cicd_deploy.rs` vs `core/src/logic/cicd_deploy.rs`
**Tauri has ~15 extra lines** — Tauri is ahead with GUI-specific patches:

1. **9 lines** of `crate::core::` vs `crate::logic::` path differences
2. **JAVA_HOME logging:** Tauri emits `"JAVA_HOME: {}"` and `"未配置 JAVA_HOME"` warnings
3. **Maven path resolution:** Tauri has smarter `JAVA_HOME/bin` detection (checks if file vs directory)
4. **Shell environment:** Tauri calls `get_user_shell_env()` to get user's shell PATH (supports sdkman/NVM); Core uses hardcoded `/usr/local/bin:/usr/bin:/bin`
5. **Build command:** Tauri: `.filter(|s| !s.is_empty())`; Core: just the raw Option

### 🟡 `tauri/src/core/nginx.rs` vs `core/src/logic/nginx.rs`
- **Diff:** 1 line — `crate::core::CoreService` vs `crate::logic::CoreService`

### 🟡 `tauri/src/core/openvpn.rs` vs `core/src/logic/openvpn.rs`
- **Diff:** 2 lines — `crate::core::data_dir::` vs `crate::logic::data_dir::`

### 🟡 `tauri/src/core/data_dir.rs` vs `core/src/logic/data_dir.rs`
- **Identical** ✅

### 🔴 Tauri-ONLY files (no core equivalent):
| File | Lines | Purpose |
|------|-------|---------|
| `tauri/src/core/lan.rs` | 1,509 | LAN file transfer, chat, networking (GUI-only) |
| `tauri/src/core/system_logger.rs` | 112 | System logging (GUI-only) |
| `tauri/src/core/tray_notification.rs` | 284 | Desktop tray notifications (GUI-only) |

---

## 3. What `tauri/src/core/` Contains

This is Tauri's **own copy** of the business logic layer. It mirrors `core/src/logic/` but with:

- **14 files total:** `mod.rs`, `data_dir.rs`, `git.rs`, `ssh.rs`, `cicd_deploy.rs`, `openvpn.rs`, `wireguard.rs`, `lan.rs`, `nginx.rs`, `system_logger.rs`, `tray_notification.rs`, `log_sanitizer.rs`
- **Role:** Central `CoreService` struct that holds DB connection, SSH service, and app directory. All Tauri commands and UDS handlers route through this.
- **Status:** This is a **stale fork** of `core/src/logic/` — it started as a copy but has diverged.

---

## 4. Is Tauri Using `core/` or Its Own Copies?

### ❌ Tauri is NOT using the `supertool-core` crate

Despite `tauri/Cargo.toml` declaring:
```toml
supertool-core = { path = "../core" }
```

**Zero imports** of `supertool_core` exist anywhere in `tauri/src/`.

### Tauri's Import Pattern:

```
tauri/src/main.rs:
  mod db;              // ← LOCAL COPY of db/
  mod core;            // ← LOCAL COPY of logic/ (renamed to "core")
  mod encryption;      // ← LOCAL COPY of encryption.rs
  mod commands;        // ← Tauri-specific Tauri command handlers
  mod uds;             // ← Tauri-specific Unix Domain Socket server
```

### Dependency graph:
```
tauri/src/commands/*.rs → crate::core::CoreService (LOCAL COPY)
tauri/src/uds/*.rs      → crate::core::CoreService (LOCAL COPY)
tauri/src/core/*.rs     → crate::db::* (LOCAL COPY)
tauri/src/encryption.rs → crate::core::data_dir::* (LOCAL COPY)
```

### Some commands bypass CoreService and hit db directly:
- `commands/cicd.rs` — 19 direct `crate::db::cicd::*` calls
- `commands/lan.rs` — uses `crate::db::lan`
- `commands/openvpn.rs` — uses `crate::db::openvpn`
- `commands/wireguard.rs` — uses `crate::db::wireguard`

---

## 5. Cleanup Plan

### Phase 1: Remove `tauri/src/db/` — Use core's db ✅ SAFE

The db/ directories are byte-identical. This is a **no-risk** cleanup.

**Steps:**
1. Delete `tauri/src/db/`
2. In `tauri/src/main.rs`, remove `mod db;`
3. In `tauri/Cargo.toml`, ensure `supertool-core` dependency is present (already is)
4. Replace all `crate::db::` with `supertool_core::db::` across:
   - `tauri/src/core/` (all files)
   - `tauri/src/commands/` (all files)
   - `tauri/src/uds/` (if any)

### Phase 2: Merge Divergences Between core/ and Tauri copies

**2a. Push Tauri-only improvements INTO core:**
- Shell PATH resolution (`get_user_shell_env()`) in `cicd_deploy.rs`
- JAVA_HOME smart detection in `cicd_deploy.rs`
- JAVA_HOME logging in `cicd_deploy.rs`
- Build command `.filter()` in `cicd_deploy.rs`
- Core's new CI/CD approval workflow → merge into Tauri

**2b. After merging, delete `tauri/src/core/` and use core's logic:**
- Delete `tauri/src/core/`
- In `tauri/src/main.rs`, remove `mod core;`
- Replace all `crate::core::` with `supertool_core::logic::`
- For `crate::core::CoreService` → `supertool_core::CoreService` (re-exported)

### Phase 3: Handle Tauri-Exclusive Files

These files have no core equivalent because they're GUI-specific:

| File | Action |
|------|--------|
| `core/lan.rs` (1,509 lines) | **Keep** in Tauri, but move to `tauri/src/lan/` or keep as Tauri-only module |
| `core/system_logger.rs` (112 lines) | **Keep** in Tauri-only module |
| `core/tray_notification.rs` (284 lines) | **Keep** in Tauri-only module |

These should NOT be in `core/` (which is shared with CLI). The core version already removed them (comment confirms this).

### Phase 4: Remove `tauri/src/encryption.rs`

**Steps:**
1. Delete `tauri/src/encryption.rs`
2. Replace `crate::encryption::` with `supertool_core::encryption::`

### Phase 5: Verify UDS Layer Stays Tauri-Specific

The UDS layer (`tauri/src/uds/`) is correctly Tauri-specific:
- `server.rs` — Unix domain socket listener (Tauri only)
- `router.rs` — JSON routing for CLI communication (Tauri only)
- `protocol.rs` — Line-buffered protocol (Tauri only)

These should **stay** in Tauri but must call `supertool_core::CoreService` instead of local copies.

### Phase 6: Commands Layer

`tauri/src/commands/` is correctly Tauri-specific (these are `#[tauri::command]` handlers). They should:
- Stay in Tauri
- Call `supertool_core::CoreService` methods (not bypass to db directly)
- Some commands (cicd.rs especially) bypass CoreService and call db directly — these should be refactored to go through CoreService

### Summary Priority:

| Priority | Action | Risk | Lines Affected |
|----------|--------|------|----------------|
| **P0** | Remove `tauri/src/db/`, use `supertool_core::db::` | Low | ~2,000 LOC deleted |
| **P1** | Merge Tauri→core improvements in cicd_deploy.rs | Medium | ~15 lines |
| **P2** | Merge core→Tauri improvements in mod.rs | Medium | ~370 lines |
| **P3** | Remove `tauri/src/encryption.rs`, use core | Low | ~30 lines |
| **P4** | Remove `tauri/src/core/` (non-Tauri-only files), use `supertool_core::logic::` | High | ~2,900 LOC replaced |
| **P5** | Refactor commands/cicd.rs to use CoreService instead of direct db | Medium | 19 calls |
| **P6** | Move Tauri-only files (lan, system_logger, tray_notification) out of core/ dir | Low | ~1,900 LOC moved |

### Total duplication to eliminate: **~5,000+ lines** across db/ (2,000), core/ (2,900), encryption.rs (30), plus all the path-rewiring.
