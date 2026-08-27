# CICD 部署期间整个窗口卡死（2026-08-27 修复）

## 现象

部署大项目期间，SuperTool **整个窗口**点什么都没反应（其他 app 正常），部署结束才恢复；有时能点但极卡。

## 排查与证据

1. 每个 `app.emit` 在 macOS 上最终都要回到**主线程 runloop** 做一次 webview eval
   （`sample` 抓主线程栈可见 `WryWebViewInner::load_html` → `WebKitWebPage evaluateJavaScript_`
   → `IPC WKCommandDecodeEncodeSizes`）。主线程被 eval 占满 → 鼠标事件排队 → 窗口无响应。
2. 部署事件量（`{dataDir}/deploy-logs/{deployId}.log`，每条进度一行）：

   | 部署 | 总行数 | 峰值事件/秒 |
   |---|---|---|
   | 旅投前端（uni-app） | 22506 | **10102** |
   | mall 聚合根 maven | 3985 | 694 |
   | 商城后端 maven | 1145 | 633 |
   | 小改动（跳过构建） | 31 | 6 |

   峰值全部来自 `run_maven_build` / `run_npm_build` / `install_dependencies` 的
   **stdout 逐行 emit**（`emit("maven", "building", trimmed)`）。所以「部署大的时候才卡」。
3. 前端早就有 50ms 日志批量（`DeployPanel.vue` 的 `pendingLogLines`），但它只压住 Vue 重渲染，
   **压不住事件投递本身** —— 事件仍是一条条打到主线程。这是之前修过一次仍然卡的原因。

## 修复

攒批必须放在**后端**（`tauri/src/commands/cicd.rs`）：

- `DeployProgressBatcher`：高频行（`status` 为 `building` / `installing`）进缓冲，
  到达最小间隔 `PROGRESS_BATCH_INTERVAL_MS=200ms` 才发一个 `stage:"batch"` 事件，
  `lines:[{stage,status,message}]` 携带整批 → 主线程 eval 次数从 700~10000/秒 压到 ≤5/秒。
- 单批上限 `PROGRESS_BATCH_MAX_BUFFER=200` 行，超出丢最旧行并在批次头部插入
  「… 输出过快，已省略 N 行（完整内容见部署日志）」；**部署日志文件仍是全量**。
- 单行截断 `PROGRESS_LINE_MAX_CHARS=400`，避免超长路径行撑爆 IPC。
- 状态类事件（`connecting` / `uploading` / `success` / `failed` / `queue` / `restart` / `health`）
  **先冲缓冲再立即发送**，保证日志顺序；报错行（`looks_like_error`：`[ERROR]`、`ERROR:`、`ERR!`、
  `BUILD FAILURE`、`FAILED`、`FATAL`、`异常`）不参与攒批，构建失败原因不会被延迟或裁剪。
- 兜底定时器（250ms）在构建输出暂停时冲缓冲，避免稀疏日志延迟到下一阶段才显示；
  用 `AtomicBool` 停止标志 + `TickerStopGuard`（Drop）协作退出，不用 `abort()`
  —— abort 可能落在「已取走缓冲、尚未发出」之间丢尾部批次，panic 回卷时也会泄漏定时器。
- 所有 `deploy-progress` 发送都在**持有 batcher 锁期间**完成 → 批次与状态事件严格 FIFO。
- `core/src/logic/cicd_deploy.rs` 与 CLI 未改动：`execute_deploy` 的 `on_progress` 回调签名不变，
  批量只发生在 GUI 投递层。

前端 `DeployPanel.vue::progressHandler` 增加 `stage==='batch'` 分支：整批写入既有 50ms 渲染缓冲，
`progress` 仅在本批次真的带值时更新（不再被逐行事件重置）。

## 验证

- 单测 `tauri/src/commands/cicd.rs::progress_batcher_tests`（4 项）：时间窗才发、超上限裁剪并标注省略、
  报错行/状态事件不攒批、超长行截断。
- `cargo check --workspace`、`npx vue-tsc --noEmit`、`npm run build` 全绿。

## 注意

- 新增构建工具或新的高频 `status` 时，要同步 `is_noisy_progress`，否则又会退化成逐行 emit。
- 不要把批量逻辑挪回前端：前端批量对主线程 eval 次数没有任何削减作用。
