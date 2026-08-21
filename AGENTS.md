# SuperTool — Agent 工作约定

## 项目概览

Tauri 2 桌面运维工具（Rust + Vue 3 + TS）。

- `core/` — 共享业务逻辑层（Rust lib），Tauri commands 与 CLI 共用，保证 GUI/CLI 行为一致
- `tauri/` — Tauri shell 层，commands 定义在 `tauri/src/commands/`
- `src/` — Vue 3 前端（vite + tailwind + daisyUI）
- 构建验证三件套：`cargo check --workspace`、`npx vue-tsc --noEmit`、`npm run build`

## 模块约定

### CICD 部署

- 配置存储在 `cicd_configs` 表；多环境以 JSON 数组存于 `environments` 列（结构见 `core/src/db/cicd_tables.rs`）
- 新增配置字段必须同时改三处：`CREATE TABLE` / migration 列表（`cicd_tables.rs`）、`CicdConfig` struct + `row_to_cicd_config`（`core/src/db/cicd.rs`）、`add_cicd_config` / `update_cicd_config` 的 INSERT/UPDATE 语句 —— 漏任何一处会导致保存静默丢字段
- 部署核心流程：`core/src/logic/cicd_deploy.rs::execute_deploy`（git 同步 → 构建 → 收集产物 → SFTP 上传 → 重启 → 健康检查）
- 部署队列：同一 `config_id` 并发部署通过 `DEPLOY_QUEUES`（tokio Mutex）排队，事件 stage=`queue`（waiting/acquired）
- 增量上传：远端 `.deploy_manifest.json` 记录文件 SHA-256；**回滚恢复备份后必须删除 manifest**，否则下次增量部署会误判"未变更"跳过上传（已修复过一次，勿回退）
- 健康检查失败自动回滚：依赖远端 `.deploy_backup.tar.gz`（tar -P 绝对路径打包），回滚后重跑重启脚本

**向导新建（CicdConfigWizard.vue）**：选仓库后 `scanProject(gitRepo.path)` 识别构建工具/多模块；**代码可能不在仓库根目录**（如 `src/xxx` 子模块），此时需用户「选择目录」`pickLocalDir()` 定位实际代码目录（即 `draft.localPath`）再扫描，localPath 随配置一并保存。
**编辑页与新建页一致**：点击已有配置也走同一 `CicdConfigWizard`（`:initial` prefill，`openEditWizard` 统一入口），完成回调共享 `applyWizardPayload`（带 id 即更新）；编辑 prefill 会触发 `gitRepoId` watcher→`scanProject`，需守卫避免扫描覆盖已回填的模块列表。向导已内嵌「高级设置」（多环境/部署保障/工具路径）覆盖旧分组表单全部字段。**坑**：主区显示向导 or 旧分组表单用 `showWizard` **计算属性**（`isNewConfig` → 向导；`!selectedConfigId` → 空态；否则取 `!advancedModeFromWizard`），不要用「boolean+watcher」，否则首屏自动选中时 watcher 时序竞态会把 `wizardMode` 留在 false 导致误渲染旧分组表单。
**单体部署主模块**：`parentBuildPath` 是**主模块目录**（产物 jar 所在目录，常在子目录如商城 `SRC/mall/seller-api`），后端 `single_deploy_root()` 按 `parentBuildPath→buildPath→根目录` 解析「构建+收集」路径，保证在哪构建就在哪收集；填根目录会拿不到 jar。

详见 [docs/cicd-multi-env-deploy.md](docs/cicd-multi-env-deploy.md)

## 提交规范

- 格式 `type(scope): subject`，scope 用中文业务模块名（如 `持续部署`），subject 中文 ≤30 字
- 一次 commit 只覆盖一个模块，跨模块拆多个 commit
