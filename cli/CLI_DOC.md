---
name: stool-cli
category: dev
description: Use the SuperTool `stool` CLI v3.0.0 to manage todos, servers, databases, CI/CD, Git, and logs — for autonomous agent 运维 workflows.
trigger: Using stool CLI commands, checking CLI command syntax, debugging stool failures, adding new CLI commands
---

# SuperTool `stool` CLI v3.0.0

**Source**: `~/WebstormProjects/todo-list-electron/cli/src/main.rs`
**Version**: v3.0.0
**Transport**: Pure Unix Domain Socket (`~/.supertool/supertool.sock`), env: `SUPERTOOL_SOCKET`

**Golden rule**: Use `-j` for programmatic output. All list commands support `-j` short alias.
**Design principle**: All list commands output only `ID + name`. Operation commands only need `ID + business params`.

## Quick Start

```bash
stool version             # v2.14.0
stool guide               # Full reference
stool todo list -j        # All tasks as JSON
```

---

## Task Management (Todo)

```bash
stool todo add "Task text" [-p priority] [-d due] [-t tag] [--description DESC]
stool todo list [-c true|false] [-t tag] [-l 100] [-j]
stool todo stats [-j]
stool todo search "keyword" [-j]
stool todo complete <id>
stool todo delete <id>
stool todo show <id> [-j]
stool todo edit <id> [-t text] [-p priority] [--due DUE] [-t tag] [--description DESC]
stool todo uncomplete <id>
stool todo clear

# Subtasks
stool subtask list <todo_id> [-j]
stool subtask add <todo_id> "text" [--description DESC]
stool subtask complete <sub_id>
stool subtask delete <sub_id>
```

---

## Projects

```bash
stool project list [-j]
stool project add "Name" [-d "Description"]
stool project show <id> [-j]
stool project update <id> [-n "Name"] [--description DESC]
stool project delete <id>
stool project stats <id> [-j]
stool project todos <id> [-j]
```

---

## Server Management

```bash
stool server list [-j]
stool server add "Name" <host> [port] [user]
stool server test <id>
stool server exec <id> "command" [--timeout 60]
stool server exec-batch "command" [--tag prod]   # 🆕 v2.13.0: batch across all servers
stool server health <id> [-j]
stool server diagnose <id> [-j]
stool server delete <id>

# 🆕 v2.13.0 SFTP file management
stool server ls <id> [--path /dir] [-j]           # List remote directory
stool server download <id> <remote> [--output f]  # Download file (base64)
stool server mkdir <id> <path>                    # Create remote directory
stool server rm <id> <path>                       # Delete remote file

# 🆕 v2.13.0 Java process management (Spring Boot)
stool server read <id> <path>                     # Read remote file
stool server java-ps <id> [-j]                    # Java process details
stool server java-restart <id> --port 8080        # Emergency restart
```

---

## CI/CD

```bash
stool cicd list [-j]
stool cicd status <project_id> [-j]
stool cicd deploy <config_id> [--stream] [--watch]  # --watch: poll until done
stool cicd logs <project_id> [-l 20]
stool cicd step-logs <deploy_log_id> [-j]           # 🆕 v2.13.0
stool cicd rollback <config_id> <deploy_log_id>
stool cicd cancel <config_id>
stool cicd modules <config_id> [-j]
stool cicd history <config_id> [-l 20] [--status success|failed] [-j]
```

---

## Log Aggregator

```bash
stool log list [-j]
stool log add "Name" --server-ids "id1,id2" --log-path /var/log/app.log [--log-type tail]
stool log delete <id>
stool log search <preset_id> "keyword" [-l 50]
stool log tail <preset_id> [-l 100]    # SSE streaming (real-time)
```

Note: preset_id can be a numeric index (1-based) — CLI auto-resolves to actual ID.

---

## Database Management

```bash
stool db connections [-j]
stool db connect --id <id> --name <n> --host <h> [--port 3306] [--type mysql]
stool db disconnect <id>
stool db query -d <db_id> "SELECT ..." [-j]   # -j returns structured JSON with column names
stool db tables -d <db_id> [--db database] [-j]
stool db databases -d <db_id> [-j]

# Redis (enhanced in v2.13.0)
stool db redis -d <db_id> keys "pattern"
stool db redis -d <db_id> get <key>
stool db redis -d <db_id> set <key> <value>
stool db redis -d <db_id> delete <key>
stool db redis -d <db_id> type <key>          # 🆕
stool db redis -d <db_id> ttl <key>           # 🆕
stool db redis -d <db_id> h-get <key> <field> # 🆕
stool db redis -d <db_id> h-get-all <key>     # 🆕
stool db redis -d <db_id> h-len <key>         # 🆕
stool db redis -d <db_id> l-range <key> [start] [stop]  # 🆕
stool db redis -d <db_id> l-len <key>         # 🆕
stool db redis -d <db_id> s-members <key>     # 🆕
stool db redis -d <db_id> s-card <key>        # 🆕
```

---

## Git Repository Management 🆕 v2.13.0

```bash
stool git list [-j]                          # List registered repos
stool git status --path <repo_path> [-j]     # Repository status
stool git log --path <repo_path> [-l 20] [-j]  # Commit history
stool git branches --path <repo_path> [-j]   # Branch list
stool git pull --path <repo_path>            # Pull remote
stool git push --path <repo_path>            # Push to remote
stool git commit --path <repo_path> -m "msg" [--files f1 f2]
stool git checkout --path <repo_path> --branch <branch>
```

---

## Removed Modules (GUI only, not in CLI)

The following modules were removed from CLI v2.12.0+ — they remain available in the GUI:
- ~~MFA/2FA~~ · ~~Accounting/记账~~ · ~~Notes/笔记~~ · ~~Weekly Reports/周报~~
- ~~OpenVPN~~ · ~~Notifications~~ · ~~Backup~~ · ~~Tag management~~ · ~~Git (v2.12.x)~~

---

## Pitfalls

1. **`-j` is now supported everywhere**: All list commands accept `-j` short alias.
2. **Binary version**: Code is v3.0.0. Rebuild: `cd ~/WebstormProjects/todo-list-electron/cli && cargo build --release`.
3. **UDS socket required**: All commands (except `version` and `guide`) require the Electron app running with UDS socket at `~/.supertool/supertool.sock`.
4. **Pure UDS transport**: CLI no longer uses HTTP/TCP. Socket file must exist for connection.
5. **`db query -j`**: Returns array of objects with column names as keys.
6. **`server exec` timeout**: Default 60s. Use `--timeout N` for longer operations.
7. **`cicd history --status`**: Filter by `success`, `failed`, `rolled_back`, `cancelled`.
8. **`cicd deploy --watch`**: Polls every 5s until deploy completes/fails (max 10 min).
9. **Redis commands use string format**: Commands are sent as Redis CLI strings (e.g., `"TYPE mykey"`, `"HGET mykey field"`).
10. **`server read` fallback**: If `/api/servers/:id/read-file` endpoint not available, falls back to `exec "cat <path>"`.
13. **`server download`**: Uses base64 encoding for binary-safe transfer over HTTP.
