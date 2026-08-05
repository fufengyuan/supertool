use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "stool",
    about = concat!(
        "SuperTool CLI v",
        env!("CARGO_PKG_VERSION"),
        " — AI Agent 运维工具箱\\n服务器·CI/CD·数据库·日志·Git·MFA·审计 | JSON结构化输出 | MCP接入 | 操作审计"
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
    /// 数据库管理 — SQL 查询/表结构/Redis 操作
    Db {
        #[command(subcommand)]
        action: DbCommands,
    },
    /// 日志管理 — 流式查询/搜索/上下文定位/预设
    Log {
        #[command(subcommand)]
        action: LogCommands,
    },
    /// Git 仓库操作 — 状态/提交/拉取推送/分支切换
    Git {
        #[command(subcommand)]
        action: GitCommands,
    },
    /// MFA 管理 — 查看密钥、生成 TOTP 验证码
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
        /// UDS 控制 socket 路径
        #[arg(long)]
        uds: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum TodoCommands {
    Add {
        text: String,
        #[arg(short, long)]
        priority: Option<String>,
        #[arg(short = 'd', long)]
        due: Option<String>,
        #[arg(short = 't', long)]
        tag: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        project_id: Option<String>,
    },
    List {
        #[arg(short = 'c', long)]
        completed: Option<String>,
        #[arg(short = 't', long)]
        tag: Option<String>,
        #[arg(short = 'l', long, default_value = "50")]
        limit: usize,
        #[arg(short, long)]
        json: bool,
    },
    Complete {
        id: String,
    },
    Uncomplete {
        id: String,
    },
    Delete {
        id: String,
    },
    Show {
        id: String,
        #[arg(short, long)]
        json: bool,
    },
    Edit {
        id: String,
        #[arg(short = 't', long)]
        text: Option<String>,
        #[arg(short = 'p', long)]
        priority: Option<String>,
        #[arg(long)]
        due: Option<String>,
        #[arg(short = 'g', long)]
        tag: Option<String>,
        #[arg(long)]
        description: Option<String>,
    },
    Search {
        keyword: String,
        #[arg(short, long)]
        json: bool,
    },
    Stats {
        #[arg(short, long)]
        json: bool,
    },
    Clear,
}

#[derive(Subcommand, Debug)]
pub enum SubtaskCommands {
    List {
        todo_id: String,
        #[arg(short, long)]
        json: bool,
    },
    Add {
        todo_id: String,
        text: String,
        #[arg(long)]
        description: Option<String>,
    },
    Complete {
        id: String,
    },
    Delete {
        id: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProjectCommands {
    List {
        #[arg(short, long)]
        json: bool,
    },
    Add {
        name: String,
        #[arg(short = 'd', long)]
        description: Option<String>,
    },
    Show {
        id: String,
        #[arg(short, long)]
        json: bool,
    },
    Update {
        id: String,
        #[arg(short = 'n', long)]
        name: Option<String>,
        #[arg(long)]
        description: Option<String>,
    },
    Delete {
        id: String,
    },
    Stats {
        id: String,
        #[arg(short, long)]
        json: bool,
    },
    Todos {
        id: String,
        #[arg(short, long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum ServerCommands {
    List {
        #[arg(short, long)]
        json: bool,
    },
    Add {
        name: String,
        host: String,
        port: Option<u16>,
        user: Option<String>,
    },
    Test {
        id: String,
    },
    Exec {
        id: String,
        command: String,
        #[arg(long, default_value = "60")]
        timeout: u64,
        #[arg(short, long)]
        json: bool,
    },
    Health {
        id: String,
        #[arg(short, long)]
        json: bool,
    },
    Diagnose {
        id: String,
        #[arg(short, long)]
        json: bool,
    },
    Delete {
        id: String,
    },
    Read {
        id: String,
        path: String,
        #[arg(short, long)]
        json: bool,
    },
    Ls {
        id: String,
        #[arg(long)]
        path: Option<String>,
        #[arg(short, long)]
        json: bool,
    },
    Download {
        id: String,
        remote: String,
        #[arg(long)]
        output: Option<String>,
        #[arg(short, long)]
        json: bool,
    },
    Mkdir {
        id: String,
        path: String,
        #[arg(short, long)]
        json: bool,
    },
    JavaPs {
        id: String,
        #[arg(short, long)]
        json: bool,
    },
    ExecBatch {
        id: String,
        #[arg(long)]
        script: String,
        #[arg(long, default_value = "120")]
        timeout: u64,
        #[arg(short, long)]
        json: bool,
    },
    Rm {
        id: String,
        path: String,
        #[arg(short, long)]
        json: bool,
    },
    JavaRestart {
        id: String,
        name: String,
        #[arg(long, default_value = "60")]
        timeout: u64,
        #[arg(short, long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum CicdCommands {
    List {
        #[arg(short, long)]
        json: bool,
    },
    Status {
        project_id: String,
        #[arg(short, long)]
        json: bool,
    },
    Deploy {
        config_id: String,
        #[arg(long)]
        stream: bool,
        #[arg(long)]
        watch: bool,
        /// 部署分支，覆盖配置中的 deployBranch
        #[arg(short, long)]
        branch: Option<String>,
        #[arg(short, long)]
        json: bool,
    },
    History {
        config_id: String,
        #[arg(short = 'l', long, default_value = "20")]
        limit: usize,
        #[arg(long)]
        status: Option<String>,
        #[arg(short, long)]
        json: bool,
    },
    StepLogs {
        deploy_log_id: String,
        #[arg(short, long)]
        json: bool,
    },
    Rollback {
        config_id: String,
        deploy_log_id: String,
        #[arg(short, long)]
        json: bool,
    },
    Cancel {
        config_id: String,
        #[arg(short, long)]
        json: bool,
    },
    Modules {
        config_id: String,
        #[arg(short, long)]
        json: bool,
    },
    Logs {
        config_id: String,
        #[arg(short = 'l', long, default_value = "20")]
        limit: usize,
        #[arg(short, long)]
        json: bool,
    },
    Tools {
        #[arg(long)]
        scan_path: Option<String>,
        #[arg(short, long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum DbCommands {
    List {
        #[arg(short, long)]
        json: bool,
    },
    Disconnect {
        id: String,
    },
    Query {
        #[arg(short = 'd')]
        db_id: String,
        sql: String,
        #[arg(short, long)]
        json: bool,
    },
    Tables {
        #[arg(short = 'd')]
        db_id: String,
        #[arg(long)]
        db: Option<String>,
        #[arg(short, long)]
        json: bool,
    },
    Databases {
        #[arg(short = 'd')]
        db_id: String,
        #[arg(short, long)]
        json: bool,
    },
    Structure {
        #[arg(short = 'd')]
        db_id: String,
        #[arg(long)]
        db: Option<String>,
        table: String,
        #[arg(short, long)]
        json: bool,
    },
    Data {
        #[arg(short = 'd')]
        db_id: String,
        #[arg(long)]
        db: Option<String>,
        table: String,
        #[arg(short = 'l', long, default_value = "100")]
        limit: i64,
        #[arg(long, default_value = "0")]
        offset: i64,
        #[arg(short, long)]
        json: bool,
    },
    Redis {
        #[arg(short = 'd')]
        db_id: String,
        #[arg(short, long)]
        json: bool,
        #[command(subcommand)]
        action: RedisCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum RedisCommands {
    Keys {
        pattern: Option<String>,
    },
    Get {
        key: String,
    },
    Type {
        key: String,
    },
    Ttl {
        key: String,
    },
    HGet {
        key: String,
        field: String,
    },
    HGetAll {
        key: String,
    },
    HLen {
        key: String,
    },
    LRange {
        key: String,
        start: Option<i64>,
        stop: Option<i64>,
    },
    LLen {
        key: String,
    },
    SMembers {
        key: String,
    },
    SCard {
        key: String,
    },
    Set {
        key: String,
        value: String,
    },
    Delete {
        key: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum LogCommands {
    List {
        #[arg(short, long)]
        json: bool,
    },
    Search {
        preset_id: String,
        keyword: String,
        #[arg(short = 'l', long, default_value = "50")]
        lines: usize,
        #[arg(short, long)]
        json: bool,
    },
    Tail {
        preset_id: String,
        #[arg(short = 'l', long, default_value = "100")]
        lines: usize,
        #[arg(short, long)]
        json: bool,
    },
    Context {
        preset_id: String,
        server_id: String,
        line_num: usize,
        #[arg(short = 'c', long, default_value = "20")]
        context_lines: usize,
        #[arg(short, long)]
        json: bool,
    },
    Add {
        name: String,
        #[arg(long)]
        server_ids: String,
        #[arg(long)]
        log_path: String,
        #[arg(long, default_value = "tail")]
        log_type: String,
    },
    Delete {
        id: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum GitCommands {
    List {
        #[arg(short, long)]
        json: bool,
    },
    Status {
        #[arg(long)]
        path: String,
        #[arg(short, long)]
        json: bool,
    },
    Log {
        #[arg(long)]
        path: String,
        #[arg(short = 'l', long, default_value = "20")]
        limit: usize,
        #[arg(short, long)]
        json: bool,
    },
    Branches {
        #[arg(long)]
        path: String,
        #[arg(short, long)]
        json: bool,
    },
    Pull {
        #[arg(long)]
        path: String,
        #[arg(short, long)]
        json: bool,
    },
    Push {
        #[arg(long)]
        path: String,
        #[arg(short, long)]
        json: bool,
    },
    Commit {
        #[arg(long)]
        path: String,
        #[arg(short = 'm')]
        message: String,
        #[arg(long)]
        files: Option<Vec<String>>,
        #[arg(short, long)]
        json: bool,
    },
    Checkout {
        #[arg(long)]
        path: String,
        #[arg(long)]
        branch: String,
        #[arg(short, long)]
        json: bool,
    },
}

// ============ 新增命令枚举 ============

#[derive(Subcommand, Debug)]
pub enum MfaCommands {
    /// 列出所有 MFA 密钥
    List {
        #[arg(short, long)]
        json: bool,
    },
    /// 添加 MFA 密钥
    Add {
        name: String,
        secret: String,
        #[arg(long)]
        issuer: Option<String>,
        #[arg(long, default_value = "6")]
        digits: u32,
        #[arg(long, default_value = "30")]
        period: u32,
        #[arg(long, default_value = "SHA1")]
        algorithm: String,
    },
    /// 删除 MFA 密钥
    Delete { id: String },
    /// 生成 TOTP 验证码（支持按 ID / 序号 / 名称关键字）
    Code {
        identifier: String,
        #[arg(short, long)]
        json: bool,
    },
    /// 批量输出所有密钥的当前验证码（AI 登录被 MFA 拦截时直接挑选）
    Codes {
        #[arg(short, long)]
        json: bool,
    },
    /// 解析 otpauth:// URI
    ParseUri {
        uri: String,
        #[arg(short, long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum NoteCommands {
    /// 列出笔记
    List {
        #[arg(long)]
        query: Option<String>,
        #[arg(long)]
        group_id: Option<String>,
        #[arg(short, long)]
        json: bool,
    },
    /// 添加笔记
    Add {
        title: String,
        #[arg(long)]
        content: Option<String>,
        #[arg(long)]
        group_id: Option<String>,
        #[arg(long)]
        tags: Option<String>,
    },
    /// 更新笔记
    Update {
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        content: Option<String>,
        #[arg(long)]
        group_id: Option<String>,
        #[arg(long)]
        tags: Option<String>,
    },
    /// 删除笔记
    Delete { id: String },
    /// 列出分组
    Groups {
        #[arg(short, long)]
        json: bool,
    },
    /// 添加分组
    AddGroup {
        name: String,
        #[arg(long)]
        color: Option<String>,
    },
    /// 更新分组
    UpdateGroup {
        id: String,
        #[arg(long)]
        name: Option<String>,
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
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        r#type: Option<String>,
        #[arg(long)]
        year: Option<i32>,
        #[arg(long)]
        month: Option<u32>,
        #[arg(short, long)]
        json: bool,
    },
    /// 添加账单记录
    Add {
        amount: f64,
        #[arg(long)]
        category: String,
        #[arg(long)]
        r#type: String,
        #[arg(long)]
        note: Option<String>,
        #[arg(long)]
        date: Option<String>,
    },
    /// 更新账单记录
    Update {
        id: String,
        #[arg(long)]
        amount: Option<f64>,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        r#type: Option<String>,
        #[arg(long)]
        note: Option<String>,
    },
    /// 删除账单记录
    Delete { id: String },
    /// 列出分类
    Categories {
        #[arg(short, long)]
        json: bool,
    },
    /// 添加分类
    AddCategory {
        name: String,
        #[arg(long)]
        icon: Option<String>,
        #[arg(long)]
        color: Option<String>,
    },
    /// 删除分类
    DeleteCategory { id: String },
    /// 列出预算
    Budgets {
        #[arg(short, long)]
        json: bool,
    },
    /// 添加预算
    AddBudget {
        category: String,
        amount: f64,
        #[arg(long)]
        month: Option<String>,
    },
    /// 删除预算
    DeleteBudget { id: String },
    /// 统计
    Stats {
        #[arg(long)]
        year: Option<i32>,
        #[arg(short, long)]
        json: bool,
    },
    /// 趋势（近 N 月）
    Trend {
        #[arg(long, default_value = "12")]
        months: usize,
        #[arg(short, long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum WeeklyCommands {
    /// 列出周报
    List {
        #[arg(short = 'l', long, default_value = "10")]
        limit: usize,
        #[arg(short, long)]
        json: bool,
    },
    /// 查看单条周报
    Show {
        id: i64,
        #[arg(short, long)]
        json: bool,
    },
    /// 保存周报
    Save {
        title: String,
        #[arg(long)]
        content: String,
        #[arg(long)]
        start_date: Option<String>,
        #[arg(long)]
        end_date: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum NginxCommands {
    /// 列出配置预设
    List {
        #[arg(short, long)]
        json: bool,
    },
    /// 添加配置预设
    Add {
        name: String,
        #[arg(long)]
        server_id: Option<String>,
        #[arg(long)]
        config_path: Option<String>,
        #[arg(long)]
        content: Option<String>,
    },
    /// 更新配置预设
    Update {
        id: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        server_id: Option<String>,
        #[arg(long)]
        config_path: Option<String>,
    },
    /// 删除配置预设
    Delete { id: String },
    /// 从远程服务器拉取 Nginx 配置
    Fetch {
        server_id: String,
        config_path: String,
        #[arg(short, long)]
        json: bool,
    },
    /// 测试远程 Nginx 配置
    Test {
        server_id: String,
        config_path: String,
        #[arg(short, long)]
        json: bool,
    },
    /// 部署配置到远程服务器
    Deploy {
        server_id: String,
        config_path: String,
        content: String,
        #[arg(short, long)]
        json: bool,
    },
    /// 列出配置版本历史
    Versions {
        preset_id: String,
        #[arg(short, long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum BackupCommands {
    /// 导出所有数据到文件
    Export {
        #[arg(long)]
        output: Option<String>,
        #[arg(short, long)]
        json: bool,
    },
    /// 从文件导入数据
    Import {
        file: String,
        #[arg(long, default_value = "merge")]
        mode: String,
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
        #[arg(short = 'l', long, default_value = "50")]
        limit: usize,
        #[arg(short, long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum McpCommands {
    /// 启动 MCP stdio server（每行一条 JSON-RPC 消息，供 AI 客户端接入）
    Serve {
        #[arg(long, default_value = "stool")]
        name: String,
    },
    /// 打印 MCP 工具清单（调试用）
    ListTools,
}
