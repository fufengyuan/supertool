//! AI 配置助手
//!
//! 分层：
//! - `llm`     —— OpenAI / Anthropic 双协议流式客户端（纯函数构造 + SSE 累加器）
//! - `safety`  —— 安全红线：进上下文前脱敏、文件读取白名单、提案密钥字段黑名单
//! - `tools`   —— 助手可调用的内部工具注册表（只读配置 + 错误分析 + 教学 + 提案，无文件/命令/SQL）
//! - `agent`   —— 多轮工具调用循环 + 上下文窗口预算裁剪
//! - `commands`—— Tauri 命令与事件出口
pub mod agent;
pub mod commands;
pub mod context;
pub mod knowledge;
pub mod llm;
pub mod safety;
pub mod tools;
