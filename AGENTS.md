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
- **部署进度事件必须在后端攒批**（`tauri/src/commands/cicd.rs::DeployProgressBatcher`）：构建期 stdout 逐行 `app.emit` 实测峰值 700~10000 事件/秒，macOS 上每次 emit 都要回主线程做一次 webview eval，事件风暴会**卡死整个窗口**（点击无响应，部署结束才恢复）。高频行（status=`building`/`installing`）按 200ms 合并成一个 `stage:"batch"` 事件（单批 200 行，超出丢最旧行并标注省略数，日志文件仍全量）；状态事件与报错行立即发（先冲缓冲保序）。**勿把批量挪回前端** —— 前端 50ms 批量只压 Vue 重渲染，压不住事件投递（曾因此修过一次仍然卡）。详见 [docs/cicd-deploy-ui-freeze.md](docs/cicd-deploy-ui-freeze.md)
- 增量上传：远端 `.deploy_manifest.json` 记录文件 SHA-256；**回滚恢复备份后必须删除 manifest**，否则下次增量部署会误判"未变更"跳过上传（已修复过一次，勿回退）
- 健康检查失败自动回滚：依赖远端 `.deploy_backup.tar.gz`（tar -P 绝对路径打包），回滚后重跑重启脚本

**向导新建（CicdConfigWizard.vue）**：选仓库后 `scanProject(gitRepo.path)` 识别构建工具/多模块；**代码可能不在仓库根目录**（如 `src/xxx` 子模块），此时需用户「选择目录」`pickLocalDir()` 定位实际代码目录（即 `draft.localPath`）再扫描，localPath 随配置一并保存。
**编辑页与新建页一致**：点击已有配置也走同一 `CicdConfigWizard`（`:initial` prefill，`openEditWizard` 统一入口），完成回调共享 `applyWizardPayload`（带 id 即更新）；编辑 prefill 会触发 `gitRepoId` watcher→`scanProject`，需守卫避免扫描覆盖已回填的模块列表。向导已内嵌「高级设置」（多环境/部署保障/工具路径）覆盖旧分组表单全部字段。**坑**：主区显示向导 or 旧分组表单用 `showWizard` **计算属性**（`isNewConfig` → 向导；`!selectedConfigId` → 空态；否则取 `!advancedModeFromWizard`），不要用「boolean+watcher」，否则首屏自动选中时 watcher 时序竞态会把 `wizardMode` 留在 false 导致误渲染旧分组表单。
**单体部署主模块**：`parentBuildPath` 是**主模块目录**（产物 jar 所在目录，常在子目录如商城 `SRC/mall/seller-api`），后端 `single_deploy_root()` 按 `parentBuildPath→buildPath→根目录` 解析「构建+收集」路径，保证在哪构建就在哪收集；填根目录会拿不到 jar。
**单体部署路径铁律**（2026-08-24 修复三层叠加缺陷）：① `DeployConfig.local_path` 优先取 `cicd_config.local_path`（向导「选择目录」的代码实际目录），空才回退 `gitRepo.path`——否则 localPath 指向子目录时部署引擎仍在仓库根构建；**DeployConfig 有两个构造点必须同改**：core/mod.rs 的 execute_deploy 包装（CLI 路径）与 tauri/commands/cicd.rs 的 build_deploy_config（GUI 部署路径）——只改一处会导致 GUI 部署仍用仓库根（f3bd014d 补修）；② npm 单体模式下**旧模块表不参与构建/收集**（`do_build` 逐模块分支有 `!parent_build_mode` 门禁）——复制配置会把源配置的模块行一并复制进 `deploy_modules`，单体配置带着它会被劫持成逐模块构建；③ 前端 `applyWizardPayload` 的 parentBuildPath 兜底填充仅限 maven 场景（npm 留空=localPath 本身；填绝对仓库根会被 `PathBuf::join` 整体替换导致打包原路径）；④ npm 单体无 target 时走 `find_dist_dir`（dist/dist\/build\/h5/build\/dist/unpackage...候选 + package.json outDir）+ `emit_collect_dist` zip 打包兜底——**zip -r 对已存在档案是追加，压缩前必须先删旧包**。⑤ run_npm_build 执行前预检构建目录 package.json：脚本不存在直接报「可用 build* 候选 + 去配置改」，起始日志带实际构建目录。单测在 `cicd_deploy.rs::single_deploy_tests`。**⑥ localPath 改子目录后存量模块行旧路径（含仓库前缀如 SRC/mall/base-api）会 join 双重前缀**——`resolve_module_dir` 统一解析：join 不存在时回退取末段再 join，build_single_module 与 collect_artifacts 多模块分支都必须走它。**⑦ 前端单体「构建目录」与「产物目录」是两个字段（2026-08-26）**：构建目录=parentBuildPath（留空即代码目录，跑 npm 需有 package.json）；产物目录=cicd_configs.outputPath（相对代码目录，如 build/h5），collect_artifacts 单产物 npm 分支优先 outputPath（`root.join(outputPath) 存在`）否则 find_dist_dir——uni-app 产物在 build/h5 时默认会误收 build 目录，必须显式填 outputPath；新字段涉及 cicd_configs 三处同改（cicd_tables 迁移+CREATE、cicd.rs struct+row+INSERT/UPDATE）+ DeployConfig 两构造点。**多模块前端同理**：模块行 buildPath/modulePath=构建目录、outputPath=产物子目录（本来已分离）；模块行未填 outputPath 且属于前端模块（build_tool 为 npm/pnpm/yarn 或 artifact_type=dist）时，collect_artifacts 自动回退 `find_dist_dir(模块目录)`，避免"构建成功但静默无产物"。**⑧ maven 父统一构建「构建目录」与「产物目录」必须分离（2026-08-26 修复）**：CI-Friendly revision 项目（mall 系）聚合根定义 `<revision>` 且兄弟模块依赖 reactor——**parentBuildPath 绝不能指向子模块**（会触发单模块构建，effective model 里 `${revision}` 不展开、兄弟依赖解析失败），必须留空（构建根=聚合根 localPath）；产物从配置级 outputPath 收集（如 `mall-server/target`）。collect_artifacts 单产物分支按工具分流：cargo→outputPath|target/release、maven→outputPath|target 收 jar（`collect_from_dir`）、npm→outputPath(is_dir)|find_dist_dir 收 zip。向导 maven 单体同样显示「产物目录」。**⑨ applyWizardPayload 的 parentBuildPath maven 兜底必须用 localPath 而非 repo.path（2026-08-26）**：localPath 可被向导选到代码子目录（聚合根 SRC/mall），repo.path（仓库根）可能无 pom.xml——兜底填它会导致 `no POM in this directory` 直接失败（PathBuf::join 绝对路径整体替换再回绕才碰巧等值）；`run_maven_build` 的 -P profile 必须过滤空串（用户清空 profile 后 Some("") 会拼出 `-P ""`）。**⑩ maven 稳定构建必须覆盖项目级并行与 build cache（2026-08-26）**：mall 等项目的 `.mvn/maven.config` 带 `-T 1C` + Maven Build Cache 扩展（maven-build-cache-config.xml），CICD 子进程在聚合根跑会触发 `Could not acquire lock(s)`（并行 + cache 扩展写本地仓库锁竞争，IDE 手动构建不踩）。`run_maven_build` 末尾追加 `-T 1` + `-Dmaven.build.cache.enabled=false`（CLI 参数优先级高于 maven.config），求稳不求快。**⑪ lib 分离过滤规则单体与多模块都支持（2026-08-26）**：多模块用模块行 `libFilterRules`；单体（单产物）用配置级 `cicd_configs.libFilterRules`（新列，三处同改 + DeployConfig 两构造点），`collect_from_dir` 单产物 maven 分支把 `config.lib_filter_rules` 传入——lib.zip 内仅打包 `find -name` 匹配的文件（白名单）。向导单体面板（maven）新增「Jar/Lib 分离」开关 + 「lib 过滤规则」输入；finish payload 的 `libSeparate` 不再按 isMultiModule fallback（后端仅 maven 生效，非 maven 传 true 无害）。单测 `collect_maven_single_lib_respects_filter_rules` 校验 zip 内容。**⑫ npm 脚本的模块行回退仅限逐模块构建（2026-08-26）**：`run_npm_build` 开头 `!config.parent_build_mode` 才读首启模块的 buildCommand 推导脚本名——单体/父统一模式下模块行可能是复制残留（前端配置带 `mvn clean package` 的 mall 行会把 npm 脚本解析成 "mvn" 报错）；单体构建命令必须走配置级 npmScript/npmCustomScript。**⑬ 前端「产物目录」配置失效必须显式警告（2026-08-26）**：collect_artifacts 前端分支当 outputPath 对应目录不存在时会**静默回退 find_dist_dir**（uni-app 产物实际在 `dist/build/h5`，用户填 `build/h5` 不存在 → 命中 `dist` 父目录打包成 dist.zip 上传，用户以为传了 build 目录）。现在：outputPath 目录不存在 → `collect` stage warning「配置的产物目录 X 不存在，已回退自动扫描」+ info 打出实际收集目录；`collect_artifacts` 增加 `emit` 参数（与 do_build 等一致传 `&emit`）。uni-app h5 产物目录=`dist/build/h5`，勿填 `build/h5`。
**CICD 字段权威来源约定**（2026-08-24 字段清理重构）：① npm 构建脚本权威字段是配置级 `npmScript/npmCustomScript`（向导下拉动态列出 package.json build* 脚本）；模块行 `buildCommand` 仅作向后兼容回退（`run_npm_build` 剥前缀取脚本名），多模块逐模块路径仍原生执行模块行命令；② **deploy_history 表已废弃**——无任何写入路径，历史查询统一走 `deploy_logs`（Dashboard 用新增的 `get_all_deploy_logs` 带 configName JOIN，CLI/MCP 的 `get_deploy_history_by_config` 委托读 deploy_logs 返回 DeployLog）；③ 回滚结果不改原记录 status（保留 failed 等终态供过滤），以 `rolled-back:success|partial at <时间>` 追加到 errorMessage；④ cicd_configs 表的 sshHost/sshPort/sshUser/sshKeyPath/sshPassword 已在 init 时 drop_column_if_exists 清除，勿再引用。⑤ **部署模式（parentBuildMode）只能被显式操作修改**：`scanLocalProject` 由 localPath watcher 隐式触发，其多模块检测仅允许在新建配置（无 id）时设置模式——否则用户每次打开编辑都被悄悄改回单体、parentBuildPath 被污染成绝对路径；手动点「扫描模块」（autoDetectParentBuild）属显式操作可覆盖。⑥ **Maven 模块有效性判据是 SpringBoot 启动类**：`cicd_tools::has_spring_boot_main`（扫 src/main/java 找 @SpringBootApplication）；无启动类的子模块 type='maven-dep'（纯依赖），不进 scanProject 的 moduleNames、前端模块树加「依赖」徽标且不可添加为部署单元。⑦ **向导模块行配置全覆盖（2026-08-26）**：模块行展开区已补全 buildPath/outputPath/buildCommand/artifactName/artifactType（自动|jar|jar-plus-lib|dist）/buildTool/libFilterRules 输入（lib 过滤仅 maven 且启用 libSeparate 时显示）；scan prefill、finish modPayload（新建多模块不丢字段）、applyWizardPayload 新建分支（src 无 id 时从 payload 兜底读取）三处必须同步。存量配置构建目录从 buildPath 迁移：编辑 prefill 时 `parentBuildPath || buildPath` 回填展示。**npm/pnpm/yarn Home 无 UI，由 nodeHome 推导，勿另加输入框**。

详见 [docs/cicd-multi-env-deploy.md](docs/cicd-multi-env-deploy.md)

### AI 配置助手

- 后端在 `tauri/src/assistant/`（llm 双协议 / tools 注册表 / agent 循环 / safety 红线 / knowledge 内置知识库 / context 窗口裁剪 / floating 悬浮窗），模型配置存取在 `core/src/logic/ai_provider.rs`（settings 的 `ai_providers` + `ai_active_model`，apiKey AES 加密、对外只回掩码，`__clear__` 哨兵清除）
- **助手没有任何写库工具**：改动只能由 `propose_config_change` 产出提案 → 前端 `ProposalCard` 逐条确认 → 由 `src/composables/useAssistantChat.ts::applyProposal` 调用各功能页一直在用的既有命令写入。新增工具时不得破坏这条，且必须同步 `tools::registry_exposes_no_dangerous_capabilities` 的精确名 + 关键字双重断言
- **凭据绝不允许进入模型上下文**：所有工具返回值必须过 `safety::deep_redact`（既按字段名抹，也逐个字符串抹形态——`environments`/`servers`/`restartScript` 这类整段 JSON/脚本文本里塞的密钥，键名匹配拦不住）；`read_db_connections` 刻意不 SELECT password 列；需要凭据时由用户在提案卡片里亲手输入
- 助手没有文件能力，唯一本地读取入口是 `safety::read_text_file_in`（只允许部署日志目录、canonicalize 后前缀校验）——**不要**把已有的 `commands::cicd::read_log_file`（无路径校验）包成工具
- 助手事件（`assistant-event` 的文本/思考增量）与部署进度事件一样必须**后端攒批**（≥120 字或 ≥80ms），逐 token emit 会重演窗口卡死
- 上下文窗口按模型配置（`AiModel.contextWindow`），裁剪在 `context::trim_to_budget`（CJK 1 字≈1 token 的保守估算）；Anthropic 侧要求角色严格交替且首条为 user，`llm::anthropic_messages` 已处理，改消息结构时要同步
- 教学与报错特征都在 `knowledge.rs` 内置（内容源自本文件与 docs 的结论），不读仓库文件；新增踩坑结论要同时补进 knowledge
- 详见 [docs/ai-config-assistant.md](docs/ai-config-assistant.md)

**新手引导**（前端）：核心功能页首次进入弹「功能介绍/使用方法/前置条件」，注册表在 `src/features/featureIntro.ts`（新增功能页在此登记三要素，prereqs 可带回跳路由）；MainLayout 监听 route.path 首次弹一次（**sessionStorage `feature_intro_seen_v1` 会话级，重启后继续弹**），页面右下角「?」可随时重看。前置资源选择处空态提供「去添加」跳转（服务器选择器 GroupedServerSelector、CICD 向导 Git 仓库选择已内置）。

**开发工具**：工具卡片注册表 `src/views/devtools/DevToolRegistry.ts`（DEV_TOOL_REGISTRY 条目：id/name/icon/category/description/offline/keywords）+ `DevTools.vue` 的 `toolComponents` 映射（defineAsyncComponent 懒加载 `./tools/*.vue`），新增工具必须两处都改；页面壳用 `views/devtools/components/ToolPage.vue`（icon/name/description + @back）。Navicat 密码加解密（`tools/NavicatTool.vue`）：Navicat 12+ 为 AES-128-CBC，key=`libcckeylibcckey`、iv=`libcciv libcciv `（Latin1，iv 尾带空格共 16 字节）；加密=明文→CBC→ciphertext-Latin1→hex 大写；解密=hex→Latin1→base64→AES 解密→Latin1。纯前端 crypto-js，本地计算；验证向量 enc("123456")=`833E4ABBC56C89041A9070F043641E3B`。

**局域网自动重连**（2026-08-27）：架构=UDP 广播心跳（49152，5s 一发：255.255.255.255 + 组播 239.255.0.1 + 定向 x.y.z.255）+ 每 5s `check_offline_peers` 超时 30s 判离线（emit `lan-peer-lost`）+ UDP 消息 + TCP 文件（49154）。断线自愈：① 前端 LanUsers `ensureLanRunningAndProbe()`：调 `lan_start_if_stopped`（服务未初始化重新走 auto_start、已初始化停止则 `ensure_running` 重新拉起；`auto_start_lan` 失败后 spawn 每 10s 重试线程直至成功）→ 存在离线节点时静默重播广播；setInterval 12s 周期探测 + window online/offline 事件即时响应 + peerLost 后延迟 1.5s 重扫；头部「自动重连中」徽标（有离线节点时显示）。② 后端 heartbeat 线程连续 5 周期（25s）全路发送失败打 warning「进入自动重连等待」，恢复后打「自动重连成功」（UDP socket 本身可自愈无需重建）。命令 `lan_start_if_stopped`（commands/lan.rs）已注册 main.rs invoke_handler。

**悬浮待办窗**（floating-todo）：后端命令在 `tauri/src/commands/floating_todo.rs`（open/close/toggle/set_pinned，启动时 ensure_floating_todo 自动创建）；**默认小球形态**（FloatingTodoPanel.vue `collapsed` 初始 true，onMounted 用 `applyWindowSize()` 按形态统一窗口尺寸 球56×56/展开340×500，勿在 onMounted 硬编码展开尺寸否则启动闪大窗）；**关闭入口**在球左上角 × 与展开标题栏 ×（调 closeFloatingTodo 销毁窗口），主窗口侧边栏底部「悬浮待办」toggle 可随时重新打开。

**开源 GitHub 仓库**：remote `github` = `git@github.com:fufengyuan/supertool.git`（SSH 用 `~/.ssh/id_ed25519_github`）；`main` 为开源分支，承载本地 tauri 分支历史（推送 `git push github tauri:main`）。已用 filter-branch 重写历史移除 106MB 沙箱文件 `tauri/.sandbox-home`（本地 tauri 的 commit hash 与腾讯工蜂 origin/tauri 不一致，再推腾讯需 force）；master 分支历史仍含该大文件，勿推 GitHub。

## 提交规范

- 格式 `type(scope): subject`，scope 用中文业务模块名（如 `持续部署`），subject 中文 ≤30 字
- 一次 commit 只覆盖一个模块，跨模块拆多个 commit
