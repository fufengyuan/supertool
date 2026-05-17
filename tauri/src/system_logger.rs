/// 系统日志模块 — 同时输出到 stdout 和文件
///
/// 日志文件路径: ~/.supertool/logs/supertool-YYYY-MM-DD.log
/// 与 Electron 版本共享日志目录，每日自动轮转，无需额外依赖。
use log::{Level, Log, Metadata, Record};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct SystemLogger {
    file: Mutex<File>,
    #[allow(dead_code)]
    log_dir: PathBuf,
}

impl SystemLogger {
    /// 创建日志文件并初始化 logger
    pub fn init(supertool_dir: &Path) {
        let log_dir = supertool_dir.join("logs");
        fs::create_dir_all(&log_dir).expect("[SystemLogger] Failed to create log directory");

        let log_path = log_dir.join(format!(
            "supertool-{}.log",
            chrono::Local::now().format("%Y-%m-%d")
        ));

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .expect("[SystemLogger] Failed to open log file");

        let logger = SystemLogger {
            file: Mutex::new(file),
            log_dir,
        };

        let max_level = if cfg!(debug_assertions) {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        };

        log::set_boxed_logger(Box::new(logger))
            .expect("[SystemLogger] Failed to set global logger");
        log::set_max_level(max_level);

        log::info!("[SystemLogger] Log file: {}", log_path.display());
    }

    /// 供前端 console.log 调用的写入接口
    pub fn write_frontend_log(level: &str, prefix: &str, message: &str) {
        let level_upper = level.to_uppercase();
        let log_level = match level_upper.as_str() {
            "ERROR" => Level::Error,
            "WARN" => Level::Warn,
            "INFO" => Level::Info,
            "DEBUG" => Level::Debug,
            _ => Level::Info,
        };
        log::log!(log_level, "[{}] {}", prefix, message);
    }
}

impl Log for SystemLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let now = chrono::Local::now();
        let level_str = match record.level() {
            Level::Error => "ERROR",
            Level::Warn => "WARN ",
            Level::Info => "INFO ",
            Level::Debug => "DEBUG",
            Level::Trace => "TRACE",
        };

        let msg = format!(
            "[{}] [{}] {}\n",
            now.format("%Y-%m-%d %H:%M:%S%.3f"),
            level_str,
            record.args()
        );

        // 输出到控制台
        print!("{}", msg);

        // 写入文件
        if let Ok(mut file) = self.file.lock() {
            let _ = file.write_all(msg.as_bytes());
            let _ = file.flush();
        }
    }

    fn flush(&self) {
        if let Ok(mut file) = self.file.lock() {
            let _ = file.flush();
        }
    }
}
