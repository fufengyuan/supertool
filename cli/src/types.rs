use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "stool",
    about = concat!(
        "SuperTool CLI v",
        env!("CARGO_PKG_VERSION"),
        " — AI Agent 运维工具箱\n服务器·CI/CD·数据库·日志·Git·MFA·审计 | JSON结构化输出 | MCP接入 | 操作审计"
    )
)]
pub struct Cli {
    /// 全局 JSON 输出模式（等价于各命令的 -j；开启后所有命令输出 `{"ok": ..., "data": ...}` envelope）
    #[arg(global = true, long, help = "以 JSON envelope 格式输出（与各命令 -j 等价）")]
    pub json: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 显示版本号
    Version,
    /// 使用指南 — 命令速查与 JSON 输出规范
    Guide,
    /// 任务管理 — 增删改查/搜索/统计/子任务
    Todo {
        #[command(subcommand)]
        action: TodoCommands,
    },
    /// 子任务管理 — 属于任务的子项增删改查
    Subtask {
        #[command(subcommand)]
        action: SubtaskCommands,
    },
    /// 项目管理 — 项目 CRUD/统计/任务聚合
    Project {
        #[command(subcommand)]
        action: ProjectCommands,
    },
    /// 服务器管理 — SSH 命令执行/文件操作/健康诊断/审批
    Server {
        #[command(subcommand)]
        action: ServerCommands,
    },
    /// CI/CD 部署管理 — 部署/回滚/取消/历史/模块
    Cicd {
        #[command(subcommand)]
        action: CicdCommands,
    },
    /// 数据库管理 — SQL 查询（审批连接只读白名单）/表结构/Redis 操作
    Db {
        /// 数据库管理 — SQL 查询（审批连接只读白名单）/表结构/Redis 操作
        #[command(subcommand)]
        action: DbCommands,
    },
    /// 日志管理 — 搜索（含历史轮转日志 --date/--days）/流式查询/上下文定位/预设
    Log {
        /// 日志管理 — 流式查询/搜索（含历史轮转日志 --date/--days）/上下文定位/预设
        #[command(subcommand)]
        action: LogCommands,
    },
    /// Git 仓库操作 — 状态/提交/拉取推送/分支切换
    Git {
        #[command(subcommand)]
        action: GitCommands,
    },
    /// MFA 管理 — 查看密钥、生成 TOTP 验证码（Base32 校验）/批量 codes
    Mfa {
        #[command(subcommand)]
        action: MfaCommands,
    },
    /// 笔记管理 — CRUD + 分组
    Note {
        #[command(subcommand)]
        action: NoteCommands,
    },
    /// 记账管理 — 收支记录、分类、预算、统计
    Accounting {
        #[command(subcommand)]
        action: AccountingCommands,
    },
    /// 周报管理 — 生成/查看周报
    Weekly {
        #[command(subcommand)]
        action: WeeklyCommands,
    },
    /// Nginx 配置管理 — 预设/拉取/测试/部署
    Nginx {
        #[command(subcommand)]
        action: NginxCommands,
    },
    /// 数据备份/恢复 — 导出/导入所有数据
    Backup {
        #[command(subcommand)]
        action: BackupCommands,
    },
    /// 操作审计 — 查询 CLI/GUI 的写操作记录（参数已脱敏）
    Audit {
        #[command(subcommand)]
        action: AuditCommands,
    },
    /// MCP server — 供 Claude Code / Cursor 等 AI 工具原生调用 stool 能力
    Mcp {
        #[command(subcommand)]
        action: McpCommands,
    },
    /// WireGuard tunnel 后台进程 — 由 GUI 通过 sudo 调起，普通用户不要直接运行
    #[command(name = "wg-tunnel", hide = true)]
    WgTunnel {
        /// 配置 JSON 文件路径
        #[arg(long)]
        conf: String,
        /// 主进程 fd 传递 socket 路径（SCM_RIGHTS）
        #[arg(long)]
        socket: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum TodoCommands {
    /// 新增任务 — 文本 + 优先级/截止日期/标签/描述/所属项目
    Add {
        /// 任务文本
        text: String,
        /// 优先级（high/medium/low）
        #[arg(short, long)]
        priority: Option<String>,
        /// 截止日期
        #[arg(short = 'd', long)]
        due: Option<String>,
        /// 标签
        #[arg(short = 't', long)]
        tag: Option<String>,
        /// 描述
        #[arg(long)]
        description: Option<String>,
        /// 所属项目 ID
        #[arg(long)]
        project_id: Option<String>,
    },
    /// 列出任务 — 支持按完成状态、标签过滤
    List {
        /// 按完成状态过滤（true/false）
        #[arg(short = 'c', long)]
        completed: Option<String>,
        /// 标签
        #[arg(short = 't', long)]
        tag: Option<String>,
        /// 返回条数上限
        #[arg(short = 'l', long, default_value = "50")]
        limit: usize,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 标记任务完成
    Complete {
        /// 记录 ID
        id: String,
    },
    /// 取消完成标记
    Uncomplete {
        /// 记录 ID
        id: String,
    },
    /// 删除任务
    Delete {
        /// 记录 ID
        id: String,
    },
    /// 查看任务详情
    Show {
        /// 记录 ID
        id: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 编辑任务 — 文本/优先级/截止日期/标签/描述
    Edit {
        /// 记录 ID
        id: String,
        /// 任务文本
        #[arg(short = 't', long)]
        text: Option<String>,
        /// 优先级（high/medium/low）
        #[arg(short = 'p', long)]
        priority: Option<String>,
        /// 截止日期
        #[arg(long)]
        due: Option<String>,
        /// 标签
        #[arg(short = 'g', long)]
        tag: Option<String>,
        /// 描述
        #[arg(long)]
        description: Option<String>,
    },
    /// 按关键字搜索任务
    Search {
        /// 搜索关键字
        keyword: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 任务统计
    Stats {
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 清空已完成任务
    Clear,
}

#[derive(Subcommand, Debug)]
pub enum SubtaskCommands {
    /// 列出指定任务的子任务
    List {
        /// 任务 ID
        todo_id: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 新增子任务
    Add {
        /// 任务 ID
        todo_id: String,
        /// 任务文本
        text: String,
        /// 描述
        #[arg(long)]
        description: Option<String>,
    },
    /// 完成子任务
    Complete {
        /// 记录 ID
        id: String,
    },
    /// 删除子任务
    Delete {
        /// 记录 ID
        id: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProjectCommands {
    /// 列出项目
    List {
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 新增项目
    Add {
        /// 名称
        name: String,
        /// 描述
        #[arg(short = 'd', long)]
        description: Option<String>,
    },
    /// 查看项目详情
    Show {
        /// 记录 ID
        id: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 更新项目 — 名称/描述
    Update {
        /// 记录 ID
        id: String,
        /// 名称
        #[arg(short = 'n', long)]
        name: Option<String>,
        /// 描述
        #[arg(long)]
        description: Option<String>,
    },
    /// 删除项目
    Delete {
        /// 记录 ID
        id: String,
    },
    /// 项目统计
    Stats {
        /// 记录 ID
        id: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 列出项目下的任务
    Todos {
        /// 记录 ID
        id: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum ServerCommands {
    /// 列出服务器
    List {
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 新增服务器
    Add {
        /// 名称
        name: String,
        /// 主机地址
        host: String,
        /// SSH 端口
        port: Option<u16>,
        /// 登录用户名
        user: Option<String>,
    },
    /// 测试 SSH 连通性
    Test {
        /// 记录 ID
        id: String,
    },
    /// 执行远程命令（危险命令会被拦截）
    Exec {
        /// 记录 ID
        id: String,
        /// 要执行的命令
        command: String,
        /// 超时时间（秒）
        #[arg(long, default_value = "60")]
        timeout: u64,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 健康检查 — 负载/磁盘/内存
    Health {
        /// 记录 ID
        id: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 详细诊断 — 系统信息与常见问题
    Diagnose {
        /// 记录 ID
        id: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 删除服务器
    Delete {
        /// 记录 ID
        id: String,
    },
    /// 读取远程文件
    Read {
        /// 记录 ID
        id: String,
        /// 远程文件路径
        path: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 列出远程目录
    Ls {
        /// 记录 ID
        id: String,
        /// 远程目录（不填则为当前目录）
        #[arg(long)]
        path: Option<String>,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 下载远程文件
    Download {
        /// 记录 ID
        id: String,
        /// 远程文件路径
        remote: String,
        /// 输出文件路径
        #[arg(long)]
        output: Option<String>,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 创建远程目录
    Mkdir {
        /// 记录 ID
        id: String,
        /// 要创建的远程目录
        path: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// Java 进程详情 — PID/端口/堆内存/运行时长
    JavaPs {
        /// 记录 ID
        id: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 批量执行多行脚本（按行顺序执行，首条失败即停）
    ExecBatch {
        /// 记录 ID
        id: String,
        /// 多行脚本（按行顺序执行，# 开头为注释）
        #[arg(long)]
        script: String,
        /// 超时时间（秒）
        #[arg(long, default_value = "120")]
        timeout: u64,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 删除远程文件（系统目录会被拦截）
    Rm {
        /// 记录 ID
        id: String,
        /// 要删除的远程文件（系统目录会被拦截）
        path: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 按 jar 名停止 Java 进程（kill → 等待 → SIGKILL，不会自动拉起）
    JavaRestart {
        /// 记录 ID
        id: String,
        /// 名称
        name: String,
        /// 超时时间（秒）
        #[arg(long, default_value = "60")]
        timeout: u64,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum CicdCommands {
    /// 列出部署配置
    List {
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 查看部署配置状态
    Status {
        /// 项目 ID
        project_id: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 执行部署 — --watch 可轮询至结束
    Deploy {
        /// 部署配置 ID
        config_id: String,
        /// 流式输出部署进度事件
        #[arg(long)]
        stream: bool,
        /// 轮询直到部署结束（每 5 秒，最长 10 分钟）
        #[arg(long)]
        watch: bool,
        /// 部署分支，覆盖配置中的 deployBranch
        #[arg(short, long)]
        branch: Option<String>,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 部署历史（可按状态过滤）
    History {
        /// 部署配置 ID
        config_id: String,
        /// 返回条数上限
        #[arg(short = 'l', long, default_value = "20")]
        limit: usize,
        /// 按状态过滤（success/failed/rolled_back/cancelled）
        #[arg(long)]
        status: Option<String>,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 查看指定部署的阶段日志
    StepLogs {
        /// 部署记录 ID
        deploy_log_id: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 回滚到指定部署
    Rollback {
        /// 部署配置 ID
        config_id: String,
        /// 部署记录 ID
        deploy_log_id: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 取消进行中的部署
    Cancel {
        /// 部署配置 ID
        config_id: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 列出配置的部署模块
    Modules {
        /// 部署配置 ID
        config_id: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 查看配置的最近部署日志
    Logs {
        /// 部署配置 ID
        config_id: String,
        /// 返回条数上限
        #[arg(short = 'l', long, default_value = "20")]
        limit: usize,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 检测构建工具/SDK 版本/项目模块（--scan-path 扫描项目）
    Tools {
        /// 要扫描的项目目录
        #[arg(long)]
        scan_path: Option<String>,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum DbCommands {
    /// 列出数据库连接
    List {
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 断开连接（CLI 无状态，空操作）
    Disconnect {
        /// 记录 ID
        id: String,
    },
    /// 执行 SQL 查询（审批连接仅放行只读白名单）
    Query {
        /// 数据库连接 ID
        #[arg(short = 'd')]
        db_id: String,
        sql: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 列出数据表
    Tables {
        /// 数据库连接 ID
        #[arg(short = 'd')]
        db_id: String,
        /// 数据库名（不填则用连接配置的默认库）
        #[arg(long)]
        db: Option<String>,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 列出数据库
    Databases {
        /// 数据库连接 ID
        #[arg(short = 'd')]
        db_id: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 查看表结构
    Structure {
        /// 数据库连接 ID
        #[arg(short = 'd')]
        db_id: String,
        /// 数据库名（不填则用连接配置的默认库）
        #[arg(long)]
        db: Option<String>,
        table: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 浏览表数据（分页）
    Data {
        /// 数据库连接 ID
        #[arg(short = 'd')]
        db_id: String,
        /// 数据库名（不填则用连接配置的默认库）
        #[arg(long)]
        db: Option<String>,
        table: String,
        /// 返回条数上限
        #[arg(short = 'l', long, default_value = "100")]
        limit: i64,
        /// 偏移量
        #[arg(long, default_value = "0")]
        offset: i64,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// Redis 操作 — 见子命令
    Redis {
        /// 数据库连接 ID
        #[arg(short = 'd')]
        db_id: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
        #[command(subcommand)]
        action: RedisCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum RedisCommands {
    /// 列出 key（默认通配 *）
    Keys {
        /// 匹配模式（默认 *）
        pattern: Option<String>,
    },
    /// 读取 key 值（显示类型 + 值）
    Get {
        /// key 名
        key: String,
    },
    /// 查询 key 类型
    Type {
        /// key 名
        key: String,
    },
    /// 查询过期时间（-1 不过期，-2 不存在）
    Ttl {
        /// key 名
        key: String,
    },
    /// 读取 Hash 字段
    HGet {
        /// key 名
        key: String,
        /// Hash 字段名
        field: String,
    },
    /// 读取 Hash 全部字段
    HGetAll {
        /// key 名
        key: String,
    },
    /// Hash 字段数量
    HLen {
        /// key 名
        key: String,
    },
    /// 读取 List 区间（默认 0 -1）
    LRange {
        /// key 名
        key: String,
        /// 起始下标（默认 0）
        start: Option<i64>,
        /// 结束下标（默认 -1）
        stop: Option<i64>,
    },
    /// List 长度
    LLen {
        /// key 名
        key: String,
    },
    /// Set 成员列表
    SMembers {
        /// key 名
        key: String,
    },
    /// Set 成员数量
    SCard {
        /// key 名
        key: String,
    },
    /// 写入字符串 key（审批连接会被拦截）
    Set {
        /// key 名
        key: String,
        /// 值
        value: String,
    },
    /// 删除 key（审批连接会被拦截）
    Delete {
        /// key 名
        key: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum LogCommands {
    /// 列出日志查询预设（含分组）
    List {
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 搜索日志 — 支持多关键词 OR 与历史轮转日志
    Search {
        /// 日志预设 ID（也可填 list 里的序号）
        preset_id: String,
        /// 搜索关键字
        keyword: String,
        /// 行数（search 为上下文范围，tail 为末尾行数）
        #[arg(short = 'l', long, default_value = "50")]
        lines: usize,
        /// 搜索历史日志：指定日期 YYYY-MM-DD（查该天写入的轮转日志文件）
        #[arg(long, conflicts_with = "days")]
        date: Option<String>,
        /// 搜索最近 N 天（含今天）的日志（1=仅今天）；默认只查当前日志文件
        #[arg(long)]
        days: Option<u64>,
        /// 整串按 ERE 正则匹配（默认按字面量；关键字含 | 时自动按多关键词 OR 拆分）
        #[arg(short = 'E', long)]
        regex: bool,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 查看末尾 N 行（静态，非流式）
    Tail {
        /// 日志预设 ID（也可填 list 里的序号）
        preset_id: String,
        /// 行数（search 为上下文范围，tail 为末尾行数）
        #[arg(short = 'l', long, default_value = "100")]
        lines: usize,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 查看指定行号周边上下文（命中行标 ▶）
    Context {
        /// 日志预设 ID（也可填 list 里的序号）
        preset_id: String,
        /// 服务器 ID
        server_id: String,
        /// 目标行号（命中行标 ▶）
        line_num: usize,
        /// 上下文行数（目标行上下各取一半）
        #[arg(short = 'c', long, default_value = "20")]
        context_lines: usize,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 新增日志预设
    Add {
        /// 名称
        name: String,
        /// 服务器 ID 列表（逗号分隔）
        #[arg(long)]
        server_ids: String,
        /// 日志文件路径
        #[arg(long)]
        log_path: String,
        /// 日志类型（file/docker/journalctl）
        #[arg(long, default_value = "tail")]
        log_type: String,
    },
    /// 删除日志预设
    Delete {
        /// 记录 ID
        id: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum GitCommands {
    /// 列出 Git 仓库
    List {
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 查看工作区状态
    Status {
        /// Git 仓库路径
        #[arg(long)]
        path: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 查看提交记录
    Log {
        /// Git 仓库路径
        #[arg(long)]
        path: String,
        /// 返回条数上限
        #[arg(short = 'l', long, default_value = "20")]
        limit: usize,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 列出分支
    Branches {
        /// Git 仓库路径
        #[arg(long)]
        path: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 拉取更新
    Pull {
        /// Git 仓库路径
        #[arg(long)]
        path: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 推送提交
    Push {
        /// Git 仓库路径
        #[arg(long)]
        path: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 提交改动（-m 提交信息，--files 指定文件）
    Commit {
        /// Git 仓库路径
        #[arg(long)]
        path: String,
        /// 提交信息
        #[arg(short = 'm')]
        message: String,
        /// 指定提交的文件（不填则提交全部改动）
        #[arg(long)]
        files: Option<Vec<String>>,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 切换分支
    Checkout {
        /// Git 仓库路径
        #[arg(long)]
        path: String,
        /// 分支名
        #[arg(long)]
        branch: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
}

// ============ 新增命令枚举 ============

#[derive(Subcommand, Debug)]
pub enum MfaCommands {
    /// 列出所有 MFA 密钥
    List {
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 添加 MFA 密钥
    Add {
        /// 名称
        name: String,
        /// Base32 密钥
        secret: String,
        /// 发行方名称（显示在验证器 App 中）
        #[arg(long)]
        issuer: Option<String>,
        /// 验证码位数
        #[arg(long, default_value = "6")]
        digits: u32,
        /// 验证码有效期（秒）
        #[arg(long, default_value = "30")]
        period: u32,
        /// 哈希算法（SHA1/SHA256/SHA512）
        #[arg(long, default_value = "SHA1")]
        algorithm: String,
    },
    /// 删除 MFA 密钥
    Delete { id: String },
    /// 生成 TOTP 验证码（支持按 ID / 序号 / 名称关键字）
    Code {
        /// ID / 序号 / 名称关键字
        identifier: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 批量输出所有密钥的当前验证码（AI 登录被 MFA 拦截时直接挑选）
    Codes {
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 解析 otpauth:// URI
    ParseUri {
        /// otpauth:// 链接
        uri: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum NoteCommands {
    /// 列出笔记
    List {
        /// 查询关键字
        #[arg(long)]
        query: Option<String>,
        /// 分组 ID
        #[arg(long)]
        group_id: Option<String>,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 添加笔记
    Add {
        /// 标题
        title: String,
        /// 内容
        #[arg(long)]
        content: Option<String>,
        /// 分组 ID
        #[arg(long)]
        group_id: Option<String>,
        /// 标签（逗号分隔）
        #[arg(long)]
        tags: Option<String>,
    },
    /// 更新笔记
    Update {
        /// 记录 ID
        id: String,
        /// 标题
        #[arg(long)]
        title: Option<String>,
        /// 内容
        #[arg(long)]
        content: Option<String>,
        /// 分组 ID
        #[arg(long)]
        group_id: Option<String>,
        /// 标签（逗号分隔）
        #[arg(long)]
        tags: Option<String>,
    },
    /// 删除笔记
    Delete { id: String },
    /// 列出分组
    Groups {
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 添加分组
    AddGroup {
        /// 名称
        name: String,
        /// 颜色（如 #ff0000）
        #[arg(long)]
        color: Option<String>,
    },
    /// 更新分组
    UpdateGroup {
        /// 记录 ID
        id: String,
        /// 名称
        #[arg(long)]
        name: Option<String>,
        /// 颜色（如 #ff0000）
        #[arg(long)]
        color: Option<String>,
    },
    /// 删除分组
    DeleteGroup { id: String },
}

#[derive(Subcommand, Debug)]
pub enum AccountingCommands {
    /// 列出账单记录
    List {
        /// 分类
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        r#type: Option<String>,
        /// 年份（如 2026）
        #[arg(long)]
        year: Option<i32>,
        /// 月份（1-12）
        #[arg(long)]
        month: Option<u32>,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 添加账单记录
    Add {
        /// 金额
        amount: f64,
        /// 分类
        #[arg(long)]
        category: String,
        #[arg(long)]
        r#type: String,
        /// 备注
        #[arg(long)]
        note: Option<String>,
        /// 日期（YYYY-MM-DD）
        #[arg(long)]
        date: Option<String>,
    },
    /// 更新账单记录
    Update {
        /// 记录 ID
        id: String,
        /// 金额
        #[arg(long)]
        amount: Option<f64>,
        /// 分类
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        r#type: Option<String>,
        /// 备注
        #[arg(long)]
        note: Option<String>,
    },
    /// 删除账单记录
    Delete { id: String },
    /// 列出分类
    Categories {
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 添加分类
    AddCategory {
        /// 名称
        name: String,
        /// 图标
        #[arg(long)]
        icon: Option<String>,
        /// 颜色（如 #ff0000）
        #[arg(long)]
        color: Option<String>,
    },
    /// 删除分类
    DeleteCategory { id: String },
    /// 列出预算
    Budgets {
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 添加预算
    AddBudget {
        /// 分类名称
        category: String,
        /// 金额
        amount: f64,
        /// 月份（1-12）
        #[arg(long)]
        month: Option<String>,
    },
    /// 删除预算
    DeleteBudget { id: String },
    /// 统计
    Stats {
        /// 年份（如 2026）
        #[arg(long)]
        year: Option<i32>,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 趋势（近 N 月）
    Trend {
        /// 统计最近 N 个月
        #[arg(long, default_value = "12")]
        months: usize,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum WeeklyCommands {
    /// 列出周报
    List {
        /// 返回条数上限
        #[arg(short = 'l', long, default_value = "10")]
        limit: usize,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 查看单条周报
    Show {
        /// 记录 ID
        id: i64,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 保存周报
    Save {
        /// 标题
        title: String,
        /// 内容
        #[arg(long)]
        content: String,
        /// 开始日期（YYYY-MM-DD）
        #[arg(long)]
        start_date: Option<String>,
        /// 结束日期（YYYY-MM-DD）
        #[arg(long)]
        end_date: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum NginxCommands {
    /// 列出配置预设
    List {
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 添加配置预设
    Add {
        /// 名称
        name: String,
        /// 服务器 ID
        #[arg(long)]
        server_id: Option<String>,
        /// 配置文件路径
        #[arg(long)]
        config_path: Option<String>,
        /// 内容
        #[arg(long)]
        content: Option<String>,
    },
    /// 更新配置预设
    Update {
        /// 记录 ID
        id: String,
        /// 名称
        #[arg(long)]
        name: Option<String>,
        /// 服务器 ID
        #[arg(long)]
        server_id: Option<String>,
        /// 配置文件路径
        #[arg(long)]
        config_path: Option<String>,
    },
    /// 删除配置预设
    Delete { id: String },
    /// 从远程服务器拉取 Nginx 配置
    Fetch {
        /// 服务器 ID
        server_id: String,
        /// 远程配置文件路径
        config_path: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 测试远程 Nginx 配置
    Test {
        /// 服务器 ID
        server_id: String,
        /// 配置文件路径
        config_path: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 部署配置到远程服务器
    Deploy {
        /// 服务器 ID
        server_id: String,
        /// 配置文件路径
        config_path: String,
        /// 文件内容
        content: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 列出配置版本历史
    Versions {
        /// 日志预设 ID（也可填 list 里的序号）
        preset_id: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum BackupCommands {
    /// 导出所有数据到文件
    Export {
        /// 输出文件路径
        #[arg(long)]
        output: Option<String>,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 从文件导入数据
    Import {
        /// 备份文件路径（.stbackup）
        file: String,
        /// 导入模式（merge=合并保留 / replace=清空后导入）
        #[arg(long, default_value = "merge")]
        mode: String,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
    /// 导出 CSV（todo 数据）
    ExportCsv,
}

/// Claw Agent 对话子命令
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Todo {
    pub id: String,
    pub text: String,
    pub completed: bool,
    pub priority: String,
    #[serde(default)]
    pub due_date: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tag: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub project_id: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum AuditCommands {
    /// 列出审计记录（写操作，参数已脱敏）
    List {
        /// 按发起方过滤：cli / gui / ai / user
        #[arg(long)]
        actor: Option<String>,
        /// 按结果过滤：success / failed / blocked
        #[arg(long)]
        result: Option<String>,
        /// 返回条数上限
        #[arg(short = 'l', long, default_value = "50")]
        limit: usize,
        /// 以 JSON 格式输出
        #[arg(short, long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum McpCommands {
    /// 启动 MCP stdio server（每行一条 JSON-RPC 消息，供 AI 客户端接入）
    Serve {
        /// 名称
        #[arg(long, default_value = "stool")]
        name: String,
    },
    /// 打印 MCP 工具清单（调试用）
    ListTools,
}
