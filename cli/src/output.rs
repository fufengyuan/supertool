use serde::Serialize;
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};

/// JSON 模式开关：全局 `--json` 或命令级 `-j` 任一开启时置位。
/// 控制 print_error/print_success 的输出格式（错误/成功 envelope 化）。
static JSON_MODE: AtomicBool = AtomicBool::new(false);

/// 业务错误码规范（对应进程 exit code）
pub const EXIT_OK: i32 = 0;
pub const EXIT_BUSINESS: i32 = 1; // 业务错误（默认）
pub const EXIT_USAGE: i32 = 2;    // 参数错误（clap 自动处理）
pub const EXIT_UNAUTHORIZED: i32 = 3; // 需审批 / 未授权
pub const EXIT_CONNECT: i32 = 4;  // 连接失败
pub const EXIT_DANGEROUS: i32 = 5; // 高危命令拦截

pub fn set_json_mode(on: bool) {
    JSON_MODE.store(on, Ordering::Relaxed);
}

pub fn is_json_mode() -> bool {
    JSON_MODE.load(Ordering::Relaxed)
}

/// 构造带错误码的 anyhow::Error，消息带 `[E{n}]` 前缀供 main 解析 exit code。
/// 新代码统一用本函数抛业务错误，旧 bail! 消息缺省为 EXIT_BUSINESS。
pub fn fail(code: i32, msg: impl Into<String>) -> anyhow::Error {
    anyhow::anyhow!("[E{}] {}", code, msg.into())
}

/// 从错误消息解析 exit code（识别 `[E{n}]` 前缀，缺省业务错误）
pub fn exit_code_for(err: &anyhow::Error) -> i32 {
    let msg = err.to_string();
    if let Some(rest) = msg.strip_prefix("[E") {
        if let Some(end) = rest.find(']') {
            if let Ok(n) = rest[..end].trim().parse::<i32>() {
                return n;
            }
        }
    }
    EXIT_BUSINESS
}

/// 剥离错误码前缀（`[E5] xxx` → `xxx`），用于面向用户的错误展示
pub fn strip_error_code_prefix(msg: &str) -> &str {
    if let Some(rest) = msg.strip_prefix("[E") {
        if let Some(end) = rest.find(']') {
            return rest[end + 1..].trim_start();
        }
    }
    msg
}

/// 打印 JSON 结果（envelope：`{"ok": true, "data": ...}`，pretty）
pub fn print_json<T: Serialize>(data: &T) {
    let out = serde_json::to_string_pretty(&json!({"ok": true, "data": data}))
        .unwrap_or_else(|_| "{}".into());
    println!("{}", out);
}

/// 打印紧凑 JSON envelope（AI token 效率，单行）
pub fn print_compact_json<T: Serialize>(data: &T) {
    let out = serde_json::to_string(&json!({"ok": true, "data": data}))
        .unwrap_or_else(|_| "{}".into());
    println!("{}", out);
}

/// 打印裸 JSON（不包 envelope）—— 仅供命令内部组合时使用，正常输出请用 print_json
#[allow(dead_code)]
pub fn print_raw_json<T: Serialize>(data: &T) {
    println!(
        "{}",
        serde_json::to_string_pretty(data).unwrap_or_default()
    );
}

pub fn print_error(msg: &str) {
    if is_json_mode() {
        eprintln!(
            "{}",
            serde_json::to_string(&json!({"ok": false, "error": {"message": msg}}))
                .unwrap_or_default()
        );
    } else {
        eprintln!("\x1b[31m✗\x1b[0m {}", msg);
    }
}

/// JSON 模式下的结构化错误输出（含 exit code），由 main 统一调用
pub fn print_error_json(code: i32, msg: &str) {
    eprintln!(
        "{}",
        serde_json::to_string(&json!({"ok": false, "error": {"code": code, "message": msg}}))
            .unwrap_or_default()
    );
}

pub fn print_success(msg: &str) {
    if is_json_mode() {
        println!(
            "{}",
            serde_json::to_string(&json!({"ok": true, "data": {"message": msg}})).unwrap_or_default()
        );
    } else {
        println!("\x1b[32m✓\x1b[0m {}", msg);
    }
}
