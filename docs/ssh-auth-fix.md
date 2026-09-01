# SSH 认证修复：空密钥路径 + 密钥/密码二选一

日期：2026-09-01
涉及提交：`17939ca8`（fix 认证）、`4a0b96b6`（feat 服务器）、`64fa815f`（fix 测试）

## 问题现象

`stool log search` 对某台商城服务器报 `Unable to open private key file`，但 GUI 查日志正常。

## 根因（两层叠加）

1. **数据层**：DB 里「未配置密钥」的服务器 `sshKeyPath` 存的是**空字符串 `''`** 而非 `NULL`。
   - 前端表单初始值 `sshKeyPath: ''`，保存时 `|| ''` 原样写入 → 库里落 `''`。
   - `''` 被下游当成有效路径 `Some("")`，ssh2 去打开空路径报 `Unable to open private key file`。

2. **认证顺序不一致**：
   - `core/src/logic/ssh.rs` 是**私钥优先**：`if let Some(key) { 试密钥 } else if let Some(pw) { 试密码 }`。
     因为 `Some("")` 命中密钥分支，配了密码也永远轮不到密码。
   - `tauri/src/commands/logs.rs` 是**密码优先**：`if let Some(pw)` 先试密码。
     所以 GUI 正常、CLI 报错。

## 修复（三层防御）

### 1. 统一认证入口 `authenticate_session`
`core/src/logic/ssh.rs` 抽出 `usable_key_path()` + `authenticate_session()`，三处重复认证（connect / test_connection / 独立会话）统一走它：
- 空串/纯空白视为未配置密钥
- 密钥路径非空才尝试密钥；密钥失败且有密码时回退密码
- 二者都不可用才报错（错误信息带上密钥失败原因）

### 2. 所有读取 `sshKeyPath` 处补空串过滤
`db/servers.rs::row_to_server`（最根本，SQLite 读出来的 `''` 就变 None）、
`log_presets.rs`（4 处）、`log_stream.rs`、`logic/mod.rs`、`logic/server.rs`。

### 3. 存量数据幂等清理
`core/src/db/mod.rs::init_db()` 追加：
`UPDATE servers SET sshKeyPath = NULL WHERE sshKeyPath IS NOT NULL AND TRIM(sshKeyPath) = ''`

## 密钥/密码二选一配置改造（feat）

- 前端 `ServerForm.vue`：认证方式区加「密码 / SSH 密钥」切换，只显示选中项，切换即清空另一项。
- 新增 `authType` 字段（**不落库**，读取时由 `sshKeyPath` 是否有值推导），加进 `types.ts` 的 `Server` 接口。
- 后端 `core/src/logic/server.rs::normalize_server_auth()`（add_server / update_server 前调用）：
  - 密钥模式：写入 trim 后路径，密码置 `""`（db 层据此清 NULL）
  - 密码模式：`sshKeyPath` 写 NULL
  - **选了密钥却没填路径 → 回退密码模式**（避免把密码也清掉）
  - 未传 `authType`（CLI/MCP/旧客户端）：有密钥路径即视为密钥认证
- `db/servers.rs::update_server` 密码三态语义：`Some("")`=显式清空写 NULL、`Some(pwd)`=新密码、`None`=保留库中旧值（编辑时不改密码场景）。
- `test_connection` 命令补 `authType` 参数；`ssh_ops.rs` 的 `ssh_connect` / `ssh_test_connection` 均支持 `authType="password"` 时忽略密钥路径。

## 测试

新增 6 个回归单测：
- `ssh.rs`: `empty_ssh_key_path_is_treated_as_unset` / `real_ssh_key_path_is_preserved`
- `server.rs`: `password_auth_clears_ssh_key_path` / `key_auth_clears_password` / `key_auth_without_path_falls_back_to_password` / `auth_type_is_inferred_when_absent`

## 既有测试修复

修复了 6 个 baseline 就失败的测试（非本次改动引入）：
- `nginx_generator`: http/stream 参数 name 与 value 间缺空格 → 输出 `sendfileon;`，改为恰一个空格（`trim_start` 后补一个空格，兼容 parser 导入带前导空格 / DB 裸值两种情况）。
- `nginx_parser`: tokenize 现产出 `Space` token 且保留引号，3 个过时断言更新。
- `settings::get_decrypts_legacy_electron_ciphertext`: 原依赖宿主 `~/.supertool/.encryption_secret` 存在才能过（环境依赖脆弱测试）。改为注入可控密钥 + 新增对称的 `encrypt_password_electron` 做 round-trip。`ELECTRON_SECRET` 从 `LazyLock<Option>` 改为 `LazyLock<Mutex<Option>>`，`set_electron_secret_for_test` 仅 `#[cfg(test)]`。

## 验证

- `cargo test -p supertool-core --lib`: **135 passed, 0 failed**（原 129 passed 6 failed）
- `cargo check --workspace`: 通过
- `npx vue-tsc --noEmit`、`vite build`: 通过

## 遗留注意

- `encrypt_password_electron` 未标 `#[cfg(test)]`（settings.rs 测试用），会进入生产二进制但为 pub dead-code，功能无害。若后续要精简可加 `#[cfg(test)]`（需同步 settings.rs 引用，因不同 crate 会编译失败，实际同 crate 可用）。
