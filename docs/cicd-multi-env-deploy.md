# CICD 多环境部署与界面重设计

> 2026-08-20 交付。涉及配置界面重构（向导 + 分组表单）与部署逻辑增强（多环境 / 队列 / 增量上传 / 健康检查回滚）。

## 需求背景

用户反馈 CICD 配置界面不友好：配置项多而乱、智能化不足、创建和编辑体验差。经确认采用方案：向导 + 表单双模式、健康检查 + 失败自动回滚、部署队列防并发、增量上传、配置内多环境。

## 数据库变更（cicd_configs 表）

新增列（均含 migration，`core/src/db/cicd_tables.rs`）：

| 列 | 类型 | 说明 |
|---|---|---|
| `gitRepoId` | TEXT | 关联 git 仓库 |
| `environments` | TEXT (JSON) | 多环境配置数组 |
| `incrementalUpload` | INTEGER | 增量上传开关（默认 1） |
| `healthCheckRetries` | INTEGER | 健康检查重试次数（默认 3） |

`environments` JSON 结构（camelCase，见 `tauri/src/commands/cicd.rs::EnvEntry`）：

```json
[{
  "name": "生产",
  "deployPath": "/opt/app",
  "servers": [{ "serverId": "srv_x", "deployDir": "" }],
  "envVars": { "NODE_ENV": "production" },
  "healthCheckUrl": "https://...",
  "healthCheckTimeout": 30,
  "healthCheckRetries": 3
}]
```

## 部署逻辑（core/src/logic/cicd_deploy.rs）

### 多环境覆盖（tauri/src/commands/cicd.rs::deploy）

`deploy` 命令新增 `environment` 参数。指定环境后按优先级覆盖：环境专属服务器（含 fallback 目录）> 环境部署路径 > 配置级服务器；`envVars` 注入构建进程（`apply_env_vars`，覆盖 maven/npm/cargo）；环境级健康检查三项配置覆盖配置级。

### 部署队列

`DEPLOY_QUEUES: HashMap<config_id, Arc<tokio::sync::Mutex<()>>>`。同配置并发部署 `try_lock` 失败后排队，发 `stage=queue, status=waiting/acquired` 事件，前端显示"排队中"徽标（amber 色）。

### 增量上传

- 部署前读取远端 `{deployDir}/.deploy_manifest.json`（文件路径 → SHA-256）
- hash 一致的产物跳过上传；全部上传成功后写回新 manifest
- 读清单失败 → 降级全量上传；上传中途失败不写回 manifest（宁可多传不漏传）
- 每台服务器独立 manifest（deploy_to_server 按服务器调用）

### 健康检查 + 自动回滚

- 配置了 `healthCheckUrl` 才启用：curl 探测 HTTP 状态码（2xx/304 通过），重试间隔 3s
- 失败后：恢复远端 `.deploy_backup.tar.gz`（部署前 tar -czf -P 打包即将被覆盖的已存在文件）→ **删除 manifest** → 重跑重启脚本（前端项目跳过）
- 备份/恢复失败均降级为 warning 不中断，健康检查失败才判失败

### 关键坑（已修复，勿回退）

1. **回滚必须删 manifest**：manifest 记录的是新版本 hash，回滚后文件是旧版本，不删会导致下次增量部署误判"未变更"跳过上传 → 服务器停留在旧版本但报告成功
2. **CicdConfig 新字段三处同步**：struct + row_to_cicd_config + INSERT/UPDATE 语句 + CREATE TABLE（全新库），漏一处轻则编译错、重则保存静默丢字段
3. **环境名唯一性**：saveConfig 校验非空 + 去重，重名会导致部署目标错乱

## 前端变更

### 新建向导（src/views/cicd/CicdConfigWizard.vue，新增）

步骤 1 选 git 仓库后自动扫描项目（`scanProject`）：
- 识别构建工具、推荐脚本、部署分支、部署路径
- 多模块项目（`isMultiModule && moduleNames`）自动生成模块勾选列表（默认全选），在步骤 2 构建配置下方展示
- 步骤 4 确认页摘要含「部署模块」行

**代码目录定位（重点）**：`scanProject` 传入的是 `gitRepos[].path`（仓库根目录）。但**代码常不在仓库根目录**（如商城项目在 `src/xxx` 子模块），此时根目录扫描结果为空 → 模块区不显示。向导在步骤 1 提供「选择目录」按钮（`pickLocalDir` → `showOpenDialogForDirs`），将实际代码目录存入 `draft.localPath` 后重新扫描；createConfigFromWizard 会把 localPath 一并写入 `config.localPath`，保证后续构建正确。

完成回调 `CiCdConfig.vue::createConfigFromWizard`：写入 config 基础字段、deployServers、`modules` 数组（自动开启 `parentBuildMode`，`parentBuildPath` 取 git 仓库本地路径），随后统一走 `saveConfig` 落库。

### 编辑表单（src/views/cicd/CiCdConfig.vue，重构）

分组折叠 5 区：基本信息 / 构建配置 / 部署目标 / 多环境部署（tab 切换环境，每环境独立路径、服务器、环境变量、健康检查）/ 部署安全（增量上传开关、健康检查 URL、超时、重试次数）。

### 部署面板（src/views/cicd/DeployPanel.vue）

环境选择器（下拉选择本次部署环境）、排队状态徽标、健康检查/回滚阶段日志着色、历史记录环境徽标。

## 验证

- `cargo check --workspace` ✅
- `npx vue-tsc --noEmit` ✅
- `npm run build` ✅（3.55s）
