# Nginx 管理功能实施计划

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** 在 SuperTool 中增加 Nginx 配置管理功能，支持远程服务器 nginx 配置的获取、图形化编辑、预检测试、一键发布、版本回滚和灰度切换。

**Architecture:** 
- 后端: Rust 通过 SSH 连接远程服务器，cat/nginx -t 操作 nginx 配置文件
- 前端: Vue 3 + daisyUI，侧边栏预设列表 + 主区域配置编辑器
- 存储: SQLite 缓存配置快照，支持版本回滚
- 集成: 预留 API 供 CI/CD 模块调用实现灰度切换

**Tech Stack:** Rust (ssh2, rusqlite, serde_json), Vue 3, TypeScript, daisyUI

---

## 数据库设计

### nginx_presets 表
```sql
CREATE TABLE IF NOT EXISTS nginx_presets (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,              -- 预设名称，如"生产环境-主站"
  server_id TEXT NOT NULL,         -- 关联 servers 表
  config_path TEXT NOT NULL,       -- nginx 配置文件路径，如 /etc/nginx/nginx.conf
  description TEXT,                -- 备注
  group_name TEXT DEFAULT '未分组', -- 分组
  is_active INTEGER DEFAULT 0,    -- 是否为当前激活的灰度配置
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (server_id) REFERENCES servers(id)
);
```

### nginx_config_versions 表
```sql
CREATE TABLE IF NOT EXISTS nginx_config_versions (
  id TEXT PRIMARY KEY,
  preset_id TEXT NOT NULL,         -- 关联 nginx_presets
  content TEXT NOT NULL,           -- 完整 nginx 配置内容
  checksum TEXT NOT NULL,          -- md5 校验
  comment TEXT,                    -- 版本备注
  is_current INTEGER DEFAULT 0,   -- 是否为当前生效版本
  created_at TEXT NOT NULL,
  FOREIGN KEY (preset_id) REFERENCES nginx_presets(id)
);
```

---

## Task 1: Rust 后端 — 数据库层

**Objective:** 创建 nginx 相关的 SQLite 表和 CRUD 函数

**Files:**
- Create: `tauri/src/db/nginx.rs`
- Modify: `tauri/src/db/mod.rs` (init_db 添加表创建)
- Modify: `tauri/src/db/mod.rs` (添加 `pub mod nginx;`)

**实现:**

1. 在 `db/mod.rs` 的 `init_db()` 中添加建表 SQL
2. 创建 `db/nginx.rs` 实现 CRUD:
   - `get_all_nginx_presets(db) -> ApiResponse<Vec<NginxPreset>>`
   - `add_nginx_preset(db, preset) -> ApiResponse<NginxPreset>`
   - `update_nginx_preset(db, preset) -> ApiResponse<()>`
   - `delete_nginx_preset(db, id) -> ApiResponse<()>`
   - `get_config_versions(db, preset_id) -> ApiResponse<Vec<NginxConfigVersion>>`
   - `add_config_version(db, version) -> ApiResponse<NginxConfigVersion>`
   - `set_current_version(db, preset_id, version_id) -> ApiResponse<()>`
   - `delete_config_version(db, id) -> ApiResponse<()>`

---

## Task 2: Rust 后端 — SSH 操作 nginx 配置

**Objective:** 通过 SSH 读取/写入/测试远程 nginx 配置

**Files:**
- Create: `tauri/src/core/nginx.rs`
- Modify: `tauri/src/core/mod.rs` (添加 nginx 方法)

**核心方法:**

```rust
// 读取远程 nginx 配置
async fn fetch_nginx_config(&self, server_id: &str, config_path: &str) -> Result<String, String>
// 执行: cat {config_path}

// 测试 nginx 配置
async fn test_nginx_config(&self, server_id: &str) -> Result<NginxTestResult, String>
// 执行: nginx -t 2>&1

// 备份远程 nginx 配置
async fn backup_nginx_config(&self, server_id: &str, config_path: &str) -> Result<String, String>
// 执行: cp {config_path} {config_path}.bak.{timestamp}

// 写入并重载 nginx
async fn deploy_nginx_config(&self, server_id: &str, config_path: &str, content: &str) -> Result<NginxDeployResult, String>
// 1. echo '{content}' > {config_path}.tmp
// 2. nginx -t -c {config_path}.tmp
// 3. mv {config_path}.tmp {config_path}
// 4. nginx -s reload

// 回滚 nginx 配置
async fn rollback_nginx_config(&self, server_id: &str, config_path: &str) -> Result<String, String>
// 执行: cp {config_path}.bak {config_path} && nginx -s reload
```

**关键: 复用现有 SSH 连接池**
- 使用 `CoreService` 中已有的 `SshService`
- 通过 `self.ssh_service.execute_command(server_id, cmd)` 执行远程命令

---

## Task 3: Rust 后端 — Tauri 命令

**Objective:** 注册 nginx 相关的 Tauri IPC 命令

**Files:**
- Create: `tauri/src/commands/nginx.rs`
- Modify: `tauri/src/commands/mod.rs` (添加 `pub mod nginx;`)
- Modify: `tauri/src/main.rs` (注册命令到 generate_handler!)

**命令列表:**
```rust
#[tauri::command(rename_all = "camelCase")]
async fn get_all_nginx_presets(state) -> Result<ApiResponse, String>

#[tauri::command(rename_all = "camelCase")]  
async fn add_nginx_preset(state, preset) -> Result<ApiResponse, String>

#[tauri::command(rename_all = "camelCase")]
async fn update_nginx_preset(state, preset) -> Result<ApiResponse, String>

#[tauri::command(rename_all = "camelCase")]
async fn delete_nginx_preset(state, id) -> Result<ApiResponse, String>

#[tauri::command(rename_all = "camelCase")]
async fn fetch_nginx_config(state, server_id, config_path) -> Result<ApiResponse, String>

#[tauri::command(rename_all = "camelCase")]
async fn test_nginx_config(state, server_id) -> Result<ApiResponse, String>

#[tauri::command(rename_all = "camelCase")]
async fn deploy_nginx_config(state, server_id, config_path, content, comment) -> Result<ApiResponse, String>

#[tauri::command(rename_all = "camelCase")]
async fn get_nginx_config_versions(state, preset_id) -> Result<ApiResponse, String>

#[tauri::command(rename_all = "camelCase")]
async fn rollback_nginx_version(state, preset_id, version_id) -> Result<ApiResponse, String>

#[tauri::command(rename_all = "camelCase")]
async fn set_active_nginx_version(state, preset_id, version_id) -> Result<ApiResponse, String>
```

---

## Task 4: 前端 — tauri-api.ts 添加 nginx API

**Objective:** 在前端 IPC 层添加 nginx 相关方法

**Files:**
- Modify: `src/utils/tauri-api.ts`

**在 `getTauriAPI()` 中添加 nginx 子 API:**
```typescript
// Nginx
getNginxPresets: () => tauriCall('get_all_nginx_presets'),
addNginxPreset: (preset) => tauriCall('add_nginx_preset', { preset }),
updateNginxPreset: (preset) => tauriCall('update_nginx_preset', { preset }),
deleteNginxPreset: (id) => tauriCall('delete_nginx_preset', { id }),
fetchNginxConfig: (serverId, configPath) => tauriCall('fetch_nginx_config', { serverId, configPath }),
testNginxConfig: (serverId) => tauriCall('test_nginx_config', { serverId }),
deployNginxConfig: (serverId, configPath, content, comment) => tauriCall('deploy_nginx_config', { serverId, configPath, content, comment }),
getNginxConfigVersions: (presetId) => tauriCall('get_nginx_config_versions', { presetId }),
rollbackNginxVersion: (presetId, versionId) => tauriCall('rollback_nginx_version', { presetId, versionId }),
setActiveNginxVersion: (presetId, versionId) => tauriCall('set_active_nginx_version', { presetId, versionId }),
```

---

## Task 5: 前端 — useNginxConfig composable

**Objective:** 封装 nginx 管理的业务逻辑

**Files:**
- Create: `src/composables/useNginxConfig.ts`

**核心逻辑:**
```typescript
export function useNginxConfig() {
  // State
  const presets = ref<NginxPreset[]>([])
  const currentPreset = ref<NginxPreset | null>(null)
  const configContent = ref('')           // 原始 nginx 配置文本
  const parsedConfig = ref<ParsedNginxConfig | null>(null) // 解析后的结构化数据
  const versions = ref<NginxConfigVersion[]>([])
  const loading = ref(false)
  const testResult = ref<NginxTestResult | null>(null)

  // CRUD
  const loadPresets = async () => { ... }
  const savePreset = async (preset) => { ... }
  const deletePreset = async (id) => { ... }

  // Config operations
  const fetchConfig = async (preset) => { ... }  // 从远程获取并解析
  const testConfig = async () => { ... }          // nginx -t 预检
  const deployConfig = async (comment) => { ... } // 发布到远程
  const rollbackToVersion = async (versionId) => { ... }

  // Nginx config parser
  const parseNginxConfig = (raw: string): ParsedNginxConfig => { ... }
  const stringifyNginxConfig = (parsed: ParsedNginxConfig): string => { ... }

  return {
    presets, currentPreset, configContent, parsedConfig, versions,
    loading, testResult,
    loadPresets, savePreset, deletePreset,
    fetchConfig, testConfig, deployConfig, rollbackToVersion,
    parseNginxConfig, stringifyNginxConfig
  }
}
```

---

## Task 6: 前端 — NginxManager.vue 主页面

**Objective:** 创建 nginx 管理的完整 UI 页面

**Files:**
- Create: `src/views/nginx/NginxManager.vue`

**页面布局:**
```
┌──────────────────────────────────────────────────────┐
│ 🔧 Nginx 配置管理                    [+ 新建预设]    │
├──────────────┬───────────────────────────────────────┤
│ 预设列表      │ 配置编辑区                             │
│              │                                       │
│ ▸ 生产-主站   │ [获取配置] [预检测试] [发布] [回滚]      │
│   server1    │                                       │
│ ▸ 生产-API   │ ┌─ 基本设置 ─────────────────────────┐  │
│   server2    │ │ worker_processes auto;             │  │
│ ▸ 测试环境   │ │ events { worker_connections 1024; }│  │
│   server3    │ │ http { ... }                       │  │
│              │ └────────────────────────────────────┘  │
│              │                                       │
│              │ ┌─ Server Blocks ────────────────────┐  │
│              │ │ ▸ server: example.com              │  │
│              │ │   listen: 443 ssl                  │  │
│              │ │   location / { proxy_pass: ... }   │  │
│              │ │ ▸ server: api.example.com          │  │
│              │ └────────────────────────────────────┘  │
│              │                                       │
│              │ ┌─ 版本历史 ─────────────────────────┐  │
│              │ │ v3 - 2026-05-08 10:30 [当前]       │  │
│              │ │ v2 - 2026-05-07 15:20 [回滚]       │  │
│              │ │ v1 - 2026-05-06 09:00 [回滚]       │  │
│              │ └────────────────────────────────────┘  │
├──────────────┴───────────────────────────────────────┤
│ 状态栏: ✅ nginx -t 测试通过 | 📦 已缓存 3 个版本     │
└──────────────────────────────────────────────────────┘
```

**关键交互:**
1. 选择预设 → 自动获取远程配置 → 解析展示
2. 编辑配置 → 实时显示原始文本预览
3. 点击"预检测试" → 远程 nginx -t → 显示结果
4. 点击"发布" → 确认弹窗 → 部署+保存版本
5. 版本列表 → 点击"回滚" → 确认 → 远程回滚

---

## Task 7: 前端 — 路由 + 侧边栏注册

**Objective:** 将 nginx 管理页面注册到路由和侧边栏

**Files:**
- Modify: `src/router/index.ts` (添加路由)
- Modify: `src/App.vue` 或侧边栏组件 (添加菜单项)

**路由:**
```typescript
{
  path: 'nginx',
  name: 'Nginx',
  component: () => import('../views/nginx/NginxManager.vue'),
},
```

---

## 实施顺序

1. ✅ 写计划 (当前)
2. Rust: DB 层 (tables + CRUD)
3. Rust: Core 层 (SSH nginx 操作)
4. Rust: Commands 层 (Tauri IPC)
5. 前端: tauri-api.ts (IPC 封装)
6. 前端: useNginxConfig.ts (业务逻辑)
7. 前端: NginxManager.vue (UI)
8. 前端: 路由 + 侧边栏
9. 编译测试
