//! supertool-omp — OMP agent process management crate
//!
//! Provides [`OmpManager`] for spawning, communicating with, and
//! terminating OMP CLI (`omp`) subprocesses.  No Tauri dependency;
//! events are delivered via a generic [`EventHandler`] callback.

pub mod llm;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// 进程输出事件
#[derive(Clone, Debug)]
pub enum ProcessEvent {
    Stdout(String),
    Stderr(String),
    Exit(Option<i32>),
}

/// 事件回调（线程安全，可跨 .await 调用）
pub type EventHandler = Arc<dyn Fn(ProcessEvent) + Send + Sync>;

/// OMP 管理器错误
#[derive(Debug)]
pub enum OmpError {
    Io(String),
    NotFound(String),
    AlreadyExists(String),
}

impl std::fmt::Display for OmpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OmpError::Io(msg) => write!(f, "IO: {msg}"),
            OmpError::NotFound(id) => write!(f, "Session not found: {id}"),
            OmpError::AlreadyExists(id) => write!(f, "Session already exists: {id}"),
        }
    }
}

impl std::error::Error for OmpError {}

impl From<std::io::Error> for OmpError {
    fn from(e: std::io::Error) -> Self {
        OmpError::Io(e.to_string())
    }
}

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

/// 运行时进程信息
struct ProcInner {
    stdin: tokio::process::ChildStdin,
}

// ---------------------------------------------------------------------------
// OmpManager
// ---------------------------------------------------------------------------

/// OMP 进程管理器
///
/// 管理多个 `omp` 子进程 session，每个 session 由其 id 标识。
/// stdout/stderr 通过 [`EventHandler`] 回调推送，不依赖 Tauri。
///
/// # 示例（Tauri command 层使用）
///
/// ```ignore
/// let omp = OmpManager::new("/path/to/omp".into());
/// omp.start("sess-1", &["launch"], None, Arc::new(|ev| {
///     match ev {
///         ProcessEvent::Stdout(line) => { /* emit to frontend */ }
///         ProcessEvent::Exit(code)   => { /* notify */ }
///         _ => {}
///     }
/// })).await?;
/// omp.write("sess-1", "my prompt\n").await?;
/// omp.stop("sess-1").await?;
/// ```
pub struct OmpManager {
    omp_bin: PathBuf,
    sessions: Mutex<HashMap<String, ProcInner>>,
}

impl OmpManager {
    /// 创建管理器，指定 `omp` 二进制路径
    pub fn new(omp_bin: PathBuf) -> Self {
        Self {
            omp_bin,
            sessions: Mutex::new(HashMap::new()),
        }
    }

    /// 返回 omp 二进制路径
    pub fn omp_path(&self) -> &PathBuf {
        &self.omp_bin
    }

    /// 启动一个新 session
    ///
    /// - `id` — 唯一 session 标识
    /// - `args` — 传给 `omp` 的参数
    /// - `cwd` — 工作目录（None = 继承当前）
    /// - `handler` — 输出/退出事件回调
    pub async fn start(
        &self,
        id: &str,
        args: &[String],
        cwd: Option<PathBuf>,
        handler: EventHandler,
    ) -> Result<(), OmpError> {
        // 防止重复
        {
            let map = self.sessions.lock().await;
            if map.contains_key(id) {
                return Err(OmpError::AlreadyExists(id.to_string()));
            }
        }

        // spawn 子进程
        let mut cmd = Command::new(&self.omp_bin);
        cmd.args(args);
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        cmd.stdin(std::process::Stdio::piped());
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        let mut child = cmd.spawn()?;

        let stdout = child.stdout.take().ok_or_else(|| OmpError::Io("No stdout".into()))?;
        let stderr = child.stderr.take().ok_or_else(|| OmpError::Io("No stderr".into()))?;
        let stdin = child.stdin.take().ok_or_else(|| OmpError::Io("No stdin".into()))?;

        let sid = id.to_string();

        // 注册 session
        {
            let mut map = self.sessions.lock().await;
            map.insert(sid.clone(), ProcInner { stdin });
        }

        // ── 后台任务：读 stdout ──
        let h1 = handler.clone();
        let sid1 = sid.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                h1(ProcessEvent::Stdout(line));
            }
            drop(sid1);
        });

        // ── 后台任务：读 stderr ──
        let h2 = handler.clone();
        let sid2 = sid.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                h2(ProcessEvent::Stderr(line));
            }
            drop(sid2);
        });

        // ── 后台任务：等待退出 ──
        tokio::spawn(async move {
            let status = child.wait().await;
            let code = status.map(|s| s.code()).ok().flatten();
            handler(ProcessEvent::Exit(code));
        });

        Ok(())
    }

    /// 向 session 写入数据（行需包含换行符 `\n`）
    pub async fn write(&self, id: &str, data: &str) -> Result<(), OmpError> {
        use tokio::sync::MutexGuard;
        let mut map: MutexGuard<'_, HashMap<String, ProcInner>> = self.sessions.lock().await;
        let entry = map.get_mut(id).ok_or_else(|| OmpError::NotFound(id.to_string()))?;
        // tokio::sync::MutexGuard 是 Send，可以跨 await
        let stdin = &mut entry.stdin;
        stdin.write_all(data.as_bytes()).await?;
        stdin.flush().await?;
        Ok(())
    }

    /// 终止 session（关闭 stdin 发送 EOF，让进程自然退出）
    pub async fn stop(&self, id: &str) -> Result<(), OmpError> {
        let mut map = self.sessions.lock().await;
        if let Some(inner) = map.remove(id) {
            // 关闭 stdin → EOF → 进程自然退出
            let mut stdin = inner.stdin;
            let _ = stdin.shutdown().await;
        }
        Ok(())
    }

    /// 检查 session 是否存活（仍在管理器中）
    pub async fn is_running(&self, id: &str) -> bool {
        let map = self.sessions.lock().await;
        map.contains_key(id)
    }

    /// 当前 session 数量
    pub async fn session_count(&self) -> usize {
        let map = self.sessions.lock().await;
        map.len()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_echo_exit() {
        let mgr = OmpManager::new(PathBuf::from("/bin/echo"));
        let exit_count = Arc::new(AtomicUsize::new(0));
        let ec = exit_count.clone();

        mgr.start(
            "t1",
            &["hello".into()],
            None,
            Arc::new(move |ev| {
                if let ProcessEvent::Exit(_) = ev {
                    ec.fetch_add(1, Ordering::SeqCst);
                }
            }),
        )
        .await
        .expect("start");

        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        assert_eq!(exit_count.load(Ordering::SeqCst), 1, "exit fired");
        assert!(mgr.is_running("t1").await, "handle remains until stop()");
        mgr.stop("t1").await.ok();
    }

    #[tokio::test]
    async fn test_not_found() {
        let mgr = OmpManager::new(PathBuf::from("/usr/bin/true"));
        let err = mgr.write("nonexistent", "hi\n").await.unwrap_err();
        assert!(matches!(err, OmpError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_duplicate_id() {
        let mgr = OmpManager::new(PathBuf::from("/usr/bin/true"));
        mgr.start("dup", &[], None, Arc::new(|_| {}))
            .await
            .expect("first start");
        let err = mgr.start("dup", &[], None, Arc::new(|_| {})).await.unwrap_err();
        assert!(matches!(err, OmpError::AlreadyExists(_)));
    }
}
