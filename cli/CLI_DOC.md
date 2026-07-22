---
name: stool-cli
category: dev
description: Use the SuperTool `stool` CLI to manage todos, servers, databases, CI/CD, logs, Git, MFA, notes, accounting, weekly reports, Nginx, and backups — for autonomous agent 运维 workflows.
trigger: Using stool CLI commands, checking CLI command syntax, debugging stool failures, adding new CLI commands
---

# SuperTool `stool` CLI

**Source**: `cli/src/main.rs`
**Architecture**: Directly links to `supertool-core` shared library — zero UDS/HTTP dependency, runs fully standalone.
**Build**: `cd cli && cargo build --release`

**Golden rule**: Use `-j` for programmatic output. All list commands support `-j` short alias.
**Design principle**: All list commands output only `ID + name`. Operation commands only need `ID + business params`.

## Quick Start

```bash
stool version
stool guide               # Full reference
stool todo list -j        # All tasks as JSON
```

---

## Task Management (Todo)

```bash
stool todo add "Task text" [-p priority] [-d due] [-t tag] [--description DESC] [--project-id PID]
stool todo list [-c true|false] [-t tag] [-l 50] [-j]
stool todo stats [-j]
stool todo search "keyword" [-j]
stool todo complete <id>
stool todo uncomplete <id>
stool todo delete <id>
stool todo show <id> [-j]
stool todo edit <id> [-t text] [-p priority] [--due DUE] [-g tag] [--description DESC]
stool todo clear
```

## Subtasks

```bash
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
stool server exec-batch <id> --script "cmd1\ncmd2" [--timeout 120]  # Multi-line batch execution
stool server health <id> [-j]
stool server diagnose <id> [-j]
stool server delete <id>

# SFTP file management
stool server read <id> <path>                     # Read remote file
stool server ls <id> [--path /dir] [-j]           # List remote directory
stool server download <id> <remote> [--output f]  # Download file
stool server mkdir <id> <path>                    # Create remote directory
stool server rm <id> <path>                       # Delete remote file (high-risk paths blocked)

# Java process management
stool server java-ps <id> [-j]                    # Java process details (PID, port, heap, uptime)
stool server java-restart <id> <name> [--timeout 60]  # Stop Java by jar name (kill → wait → SIGKILL)
```

**Notes**:
- `exec-batch`: Splits `--script` by newlines, executes each line sequentially. Empty lines and `#` comments are skipped. Stops on first failure.
- `rm`: Blocks dangerous paths (`/`, `/etc`, `/usr`, `/bin`, `/boot`, `/sys`, `/proc`).
- `java-restart`: Finds Java processes matching the jar `name`, sends `kill`, waits up to `--timeout` seconds, then `kill -9` if still running. Does NOT auto-restart — use deploy or startup script.

---

## CI/CD

```bash
stool cicd list [-j]
stool cicd status <project_id> [-j]
stool cicd deploy <config_id> [--stream] [--watch]  # --watch: poll until done
stool cicd logs <config_id> [-l 20]
stool cicd step-logs <deploy_log_id> [-j]
stool cicd rollback <config_id> <deploy_log_id>
stool cicd cancel <config_id>
stool cicd modules <config_id> [-j]
stool cicd history <config_id> [-l 20] [--status success|failed] [-j]
stool cicd tools [--scan-path /project/dir] [-j]    # Detect build tools, SDK versions, project modules
```

**`cicd tools`** output includes:
- **tools**: Detected tool availability (java/maven/node/npm/pnpm/yarn/gradle) with version and path
- **toolPaths**: JAVA_HOME, MAVEN_HOME, NODE_HOME, etc.
- **sdkVersions**: SDKMAN/NVM installed versions
- **projectScan** (with `--scan-path`): pom.xml/build.gradle/package.json detection + module tree

---

## Log Aggregator

```bash
stool log list [-j]
stool log add "Name" --server-ids "id1,id2" --log-path /var/log/app.log [--log-type tail]
stool log delete <id>
stool log search <preset_id> "keyword" [-l 50]
stool log tail <preset_id> [-l 100]                # Static tail (not streaming)
stool log context <preset_id> <server_id> <line_num> [-c 20]  # View context lines around line_num
```

**Notes**:
- `preset_id` can be a numeric index (1-based) — CLI auto-resolves to actual ID.
- `log context`: Shows `context_lines` lines centered around `line_num` (half before, half after). Target line marked with `▶`.

---

## Database Management

```bash
stool db list [-j]                                              # List saved connections
stool db disconnect <id>                                        # No-op (CLI is stateless)
stool db query -d <db_id> "SELECT ..." [-j]                     # Execute SQL query
stool db databases -d <db_id> [-j]                              # List databases
stool db tables -d <db_id> [--db database] [-j]                 # List tables
stool db structure -d <db_id> [--db database] <table> [-j]     # Show table structure
stool db data -d <db_id> [--db database] <table> [-l 100] [--offset 0] [-j]  # Browse table data
```

### Redis

```bash
stool db redis -d <db_id> keys [pattern]                # List keys (default pattern: *)
stool db redis -d <db_id> get <key>                     # Get value (shows type + value)
stool db redis -d <db_id> set <key> <value>             # Set string key
stool db redis -d <db_id> delete <key>                  # Delete key
stool db redis -d <db_id> type <key>                    # Get key type
stool db redis -d <db_id> ttl <key>                     # Get TTL (-1=no expiry, -2=not exists)
stool db redis -d <db_id> h-get <key> <field>           # Hash field get
stool db redis -d <db_id> h-get-all <key>               # Hash get all
stool db redis -d <db_id> h-len <key>                   # Hash length
stool db redis -d <db_id> l-range <key> [start] [stop]  # List range (default 0 -1)
stool db redis -d <db_id> l-len <key>                   # List length
stool db redis -d <db_id> s-members <key>               # Set members
stool db redis -d <db_id> s-card <key>                  # Set cardinality
```

**Notes**:
- CLI is **stateless**: each command connects → executes → disconnects. No connection pool.
- `db query -j`: Returns `{"success": true, "rows": [{...}, ...]}` with column names as keys.
- Redis `db_index` is read from the connection config's `dbIndex` field (default 0).
- Redis hash/list/set operations use raw command execution internally.

---

## Git Repository Management

```bash
stool git list [-j]
stool git status --path <repo_path> [-j]
stool git log --path <repo_path> [-l 20] [-j]
stool git branches --path <repo_path> [-j]
stool git pull --path <repo_path>
stool git push --path <repo_path>
stool git commit --path <repo_path> -m "msg" [--files f1 f2]
stool git checkout --path <repo_path> --branch <branch>
```

---

## MFA / 2FA

```bash
stool mfa list [-j]
stool mfa add "Name" <secret> [--issuer ISSUER] [--digits 6] [--period 30] [--algorithm SHA1]
stool mfa delete <id>
stool mfa code <identifier>           # Generate TOTP (by ID or 1-based index)
stool mfa parse-uri "otpauth://..."
```

---

## Notes

```bash
stool note list [--query Q] [--group-id GID] [-j]
stool note add "Title" [--content TEXT] [--group-id GID] [--tags "tag1,tag2"]
stool note update <id> [--title T] [--content TEXT] [--group-id GID] [--tags T]
stool note delete <id>
stool note groups [-j]
stool note add-group "Name" [--color #hex]
stool note update-group <id> [--name N] [--color #hex]
stool note delete-group <id>
```

---

## Accounting

```bash
stool accounting list [--category C] [--type income|expense] [--year Y] [--month M] [-j]
stool accounting add <amount> --category C --type income|expense [--note N] [--date D]
stool accounting update <id> [--amount A] [--category C] [--type T] [--note N]
stool accounting delete <id>
stool accounting categories [-j]
stool accounting add-category "Name" [--icon I] [--color C]
stool accounting delete-category <id>
stool accounting budgets [-j]
stool accounting add-budget <category> <amount> [--month YYYY-MM]
stool accounting delete-budget <id>
stool accounting stats [--year Y] [-j]
stool accounting trend [--months 12] [-j]
```

---

## Weekly Reports

```bash
stool weekly list [-l 10] [-j]
stool weekly show <id> [-j]
stool weekly save "Title" --content "TEXT" [--start-date D] [--end-date D]
```

---

## Nginx Configuration

```bash
stool nginx list [-j]
stool nginx add "Name" [--server-id SID] [--config-path P] [--content TEXT]
stool nginx update <id> [--name N] [--server-id SID] [--config-path P]
stool nginx delete <id>
stool nginx fetch <server_id> <config_path>
stool nginx test <server_id> <config_path>
stool nginx deploy <server_id> <config_path> <content>
stool nginx versions <preset_id> [-j]
```

---

## Backup / Restore

```bash
stool backup export [--output /path/to/file]
stool backup import <file> [--mode merge|replace]
stool backup export-csv    # Export todo data as CSV
```

---

## Pitfalls

1. **`-j` structured output**: All list commands accept `-j` for JSON output.
2. **Build**: `cd cli && cargo build --release` — output binary is `stool`.
3. **No socket required**: CLI links directly to `supertool-core`, reads the same SQLite database as the GUI. No UDS/HTTP dependency.
4. **`db query` without `-j`**: Prints a formatted table with column headers. With `-j`, returns `{"success": true, "rows": [...]}`.
5. **`server exec` high-risk blocking**: Dangerous commands (rm -rf /, shutdown, etc.) are intercepted by the CLI.
6. **`server exec-batch`**: Script is split by newlines. `#`-prefixed lines are treated as comments. Execution stops on first failure.
7. **`server rm`**: Blocks system directories (`/`, `/etc`, `/usr`, `/bin`, `/boot`, `/sys`, `/proc`).
8. **`cicd deploy --watch`**: Polls every 5s until deploy completes/fails (max 10 min).
9. **`cicd history --status`**: Filter by `success`, `failed`, `rolled_back`, `cancelled`.
10. **`log context`**: Target line is marked with `▶`. Context lines show line numbers.
11. **Redis `db_index`**: Read from connection config's `dbIndex` field. Default is 0.
12. **Servers with `requiresApproval`**: CLI blocks command execution on servers that require approval. Use GUI for these.
