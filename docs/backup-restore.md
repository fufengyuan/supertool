# 备份 / 恢复（2026-08-31 重写）

## 背景与问题

用户跨机器导出的备份导入后"没有任何效果"——配置没还原、密码丢失、路径全是源机器路径。根因：**旧实现用手写列清单 getter 逐表拼接 SQL**，表结构演化后列清单漏列/列名不匹配，导致数据导入失败或静默丢弃；且服务器密码以明文/不可解密形式导出，恢复后无法读取。

## 设计决策（grilling 确认）

- 导入默认策略 = **覆盖导入（replace）**，本地被备份完整覆盖
- 服务器密码以 **密文随备份导出**，恢复后无需重新录入
- 跨机器导入自动做 **路径改写**（`/Users/<源机器用户>/...` → 本机 home）
- 引入 **自动备份调度**（后端定时任务，保留最近 14 份）
- 密钥方案（用户指定）：**直接使用现有加密密钥，支持在设置页查看/修改**，不额外引入加密密码

## 实现要点

### 元数据驱动导出/导入（core/src/logic/backup.rs）

- 导出：`export_table_rows` 全走 `SELECT *` 整表导出，不再手写列清单
- 导入：按 `PRAGMA table_info` 动态列映射，覆盖/合并两种模式，`GENERIC_TABLES` 清单枚举参与表
- **列名一律加双引号**：`log_presets` 有 `group` 保留字列名，裸写 INSERT 报 `near group: syntax error`，加双引号解决
- CICD 五表也走通用引擎，删除独立的 `import_cicd_data`

### 密钥轮换安全顺序（core/src/logic/settings.rs + encryption.rs）

自定义密钥存 `.encryption_key`（32 字节 base64），Electron 旧口令存 `.encryption_secret`（分离，避免自定义密钥 base64 被当 scrypt 口令导致 Electron 旧密文解不开）。

轮换铁律（**先写回后切换**）：

1. `prepare`：用旧密钥解密全部 TARGETS 内容（临时持有明文）
2. `commit(new_key)`：**单事务**内用 new_key 显式重加密写回所有密文列，再 `set_custom_key` 切换 active key
3. 若 commit 失败：active key 未变、旧密文仍可解 → 重试安全；失败路径 `clear_pending_rotation` 清理临时明文

`TARGETS` 覆盖：`servers.password` / `alert_email_config.smtp_password` / `nginx_passwords.pass` + settings JSON。新增任何用 `encrypt_password` 入库的列必须同步加进 `TARGETS`。

### 自动备份（tauri/src/auto_backup.rs）

- 后端 tokio 定时任务，按 `auto_backup_*` 设置项到点执行 `run_auto_backup`
- 保留最近 14 份轮转（`rotate_backups`）
- 读设置用 `get_setting`（前端 `set_setting` 只存键值、无调度）

### git_repos 表结构矛盾修复（core/src/db/mod.rs）

备份导入 git_repos 时发现 schema 冲突：部分迁移用 `lastCommit`、部分用 `lastOpened`。统一为 `lastOpened`，迁移把 `lastCommit` 改列。CLI `cli/src/commands/git.rs` 同步改。

### 验证

- 用真实备份文件端到端导入：1043 条、0 错误、路径改写 8 处
- 全新 roundtrip：导出→导入 762 条、0 错误，含 CICD 32 条 + 密码密文 21 条
- 跨库/跨版本导入：`stool backup import <file> --mode replace` 到全新 HOME 下验证
- `api_requests` 旧数据 id 为 TEXT 与 INTEGER 主键冲突属历史数据问题，非引擎缺陷
- cargo check / vue-tsc / npm run build 全绿，单测通过（nginx/electron 5 个为基线失败）

## 历史遗留

- 旧实现基于手写列 getter，表结构演化后静默漏列丢配置 → 已废弃，勿回退
- 服务器密码旧备份若为明文/不可解密，需重新录入
