use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(name = "stool", version = env!("CARGO_PKG_VERSION"), about = "SuperTool CLI - AI Agent 运维工具箱", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 任务管理
    Todo { #[command(subcommand)] action: TodoCommands },
    /// 子任务管理
    Subtask { #[command(subcommand)] action: SubtaskCommands },
    /// 项目管理
    Project { #[command(subcommand)] action: ProjectCommands },
    /// 服务器管理
    Server { #[command(subcommand)] action: ServerCommands },
    /// CI/CD 部署管理
    Cicd { #[command(subcommand)] action: CicdCommands },
    /// 数据库管理
    Db { #[command(subcommand)] action: DbCommands },
    /// 日志聚合器
    Log { #[command(subcommand)] action: LogCommands },
    /// Git 仓库管理
    Git { #[command(subcommand)] action: GitCommands },
    /// 显示版本号
    Version,
    /// 查看 CLI 使用指南
    Guide,
}

#[derive(Subcommand)]
pub enum TodoCommands {
    Add { text: String, #[arg(short, long, default_value = "medium")] priority: String, #[arg(short, long)] due: Option<String>, #[arg(short, long, default_value = "")] tag: String, #[arg(long)] description: Option<String> },
    List { #[arg(short, long)] completed: Option<bool>, #[arg(short, long)] tag: Option<String>, #[arg(short = 'l', long, default_value = "100")] limit: usize, #[arg(short = 'j', long)] json: bool },
    Complete { id: String },
    Delete { id: String },
    Show { id: String, #[arg(long)] json: bool },
    Stats { #[arg(short = 'j', long)] json: bool },
    Clear,
    Search { keyword: String, #[arg(short = 'j', long)] json: bool },
    Edit { id: String, #[arg(short, long)] text: Option<String>, #[arg(short, long)] priority: Option<String>, #[arg(long)] due: Option<String>, #[arg(short, long)] tag: Option<String>, #[arg(long)] description: Option<String> },
    Uncomplete { id: String },
}

#[derive(Subcommand)]
pub enum SubtaskCommands {
    List { todo_id: String, #[arg(short = 'j', long)] json: bool },
    Add { todo_id: String, text: String, #[arg(long)] description: Option<String> },
    Complete { id: String },
    Delete { id: String },
}

#[derive(Subcommand)]
pub enum ProjectCommands {
    List { #[arg(short = 'j', long)] json: bool },
    Add { name: String, #[arg(short, long)] description: Option<String> },
    Show { id: String, #[arg(short = 'j', long)] json: bool },
    Update { id: String, #[arg(short, long)] name: Option<String>, #[arg(long)] description: Option<String> },
    Delete { id: String },
    Stats { id: String, #[arg(short = 'j', long)] json: bool },
    Todos { id: String, #[arg(short = 'j', long)] json: bool },
}

#[derive(Subcommand)]
pub enum ServerCommands {
    List { #[arg(short = 'j', long)] json: bool },
    Add { name: String, host: String, port: Option<u16>, user: Option<String> },
    Delete { id: String },
    Test { id: String },
    Exec { id: String, command: String, #[arg(long)] timeout: Option<u64> },
    Health { id: String, #[arg(short, long)] json: bool },
    Diagnose { id: String, #[arg(short, long)] json: bool },
    Read { id: String, path: String },
    Ls { id: String, #[arg(long, default_value = ".")] path: String, #[arg(short = 'j', long)] json: bool },
    Download { id: String, remote: String, #[arg(long)] output: Option<String> },
    Mkdir { id: String, path: String },
    JavaPs { id: String, #[arg(short = 'j', long)] json: bool },
}

#[derive(Subcommand)]
pub enum CicdCommands {
    List { #[arg(short = 'j', long)] json: bool },
    Status { project_id: String, #[arg(short = 'j', long)] json: bool },
    Deploy { config_id: String, #[arg(long, help = "SSE 实时流式输出（推荐）")] stream: bool, #[arg(long, help = "每 5 秒轮询状态直到完成（最长 10 分钟）")] watch: bool },
    Logs { project_id: String, #[arg(short = 'l', long, default_value = "20")] limit: usize },
    StepLogs { deploy_log_id: String, #[arg(short = 'j', long)] json: bool },
    Rollback { config_id: String, deploy_log_id: String },
    Cancel { config_id: String },
    Modules { config_id: String, #[arg(short = 'j', long)] json: bool },
    History { config_id: String, #[arg(short = 'l', long, default_value = "20")] limit: usize, #[arg(long)] status: Option<String>, #[arg(short = 'j', long)] json: bool },
}

#[derive(Subcommand)]
pub enum LogCommands {
    List { #[arg(short = 'j', long)] json: bool },
    Add { name: String, #[arg(long)] server_ids: String, #[arg(long)] log_path: String, #[arg(long, default_value = "tail")] log_type: String },
    Delete { id: String },
    Search { preset_id: String, keyword: String, #[arg(short, long, default_value = "50")] lines: usize },
    Tail { preset_id: String, #[arg(short, long, default_value = "100")] lines: usize },
}

#[derive(Subcommand)]
pub enum DbCommands {
    List { #[arg(short = 'j', long)] json: bool },
    Disconnect { id: String },
    Query { #[arg(long, short = 'd')] db_id: String, sql: String, #[arg(short = 'j', long)] json: bool },
    Tables { #[arg(long, short = 'd')] db_id: String, #[arg(long)] db: Option<String>, #[arg(short = 'j', long)] json: bool },
    Databases { #[arg(long, short = 'd')] db_id: String, #[arg(short = 'j', long)] json: bool },
    Redis { #[arg(long, short = 'd')] db_id: String, #[command(subcommand)] action: RedisCommands },
}

#[derive(Subcommand)]
pub enum RedisCommands {
    Get { key: String },
    Set { key: String, value: String },
    Keys { pattern: String },
    Delete { key: String },
    Type { key: String },
    Ttl { key: String },
    HGet { key: String, field: String },
    HGetAll { key: String },
    HLen { key: String },
    LRange { key: String, start: Option<i64>, stop: Option<i64> },
    LLen { key: String },
    SMembers { key: String },
    SCard { key: String },
}

#[derive(Subcommand)]
pub enum GitCommands {
    List { #[arg(short = 'j', long)] json: bool },
    Status { #[arg(long)] path: String, #[arg(short = 'j', long)] json: bool },
    Log { #[arg(long)] path: String, #[arg(short = 'l', long, default_value = "20")] limit: usize, #[arg(short = 'j', long)] json: bool },
    Branches { #[arg(long)] path: String, #[arg(short = 'j', long)] json: bool },
    Pull { #[arg(long)] path: String },
    Push { #[arg(long)] path: String },
    Commit { #[arg(long)] path: String, #[arg(short, long)] message: String, #[arg(long)] files: Vec<String> },
    Checkout { #[arg(long)] path: String, #[arg(long)] branch: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    #[serde(default, rename = "projectId")]
    pub project_id: Option<String>,
}
