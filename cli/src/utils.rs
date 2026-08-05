pub fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace("'", "'\\''"))
}

/// 检测高危命令（CLI 层防护，防止 AI 误操作生产服务器）
pub fn is_dangerous_command(cmd: &str) -> bool {
    let patterns = [
        // 文件系统破坏
        "rm -rf",
        "rm -fr",
        "rm -r /",
        "rm -f /",
        "dd if=",
        "mkfs",
        "fdisk",
        "parted",
        // 系统级破坏
        "kill -9",
        "killall",
        "pkill -9",
        "shutdown",
        "reboot",
        "poweroff",
        "halt",
        // 网络危险
        "iptables -F",
        "iptables --flush",
        "ufw disable",
        "firewall-cmd --stop",
        // 数据擦除
        "shred",
        "wipe",
        "blkdiscard",
        // 权限篡改
        "chmod 777 /",
        "chown -R /",
        // 覆盖写
        "> /etc/",
        "> /var/",
        "> /boot/",
        // 管道到 bash（远程执行）
        "curl ",
        "wget ",
    ];
    let lower = cmd.to_lowercase();
    for p in &patterns {
        if lower.contains(p) {
            // 特殊放行：curl/wget 仅用于健康检查/下载时放行（不含管道到 shell）
            if (*p == "curl " || *p == "wget ")
                && !lower.contains("| sh")
                && !lower.contains("| bash")
                && !lower.contains("|| sh")
                && !lower.contains("|| bash")
            {
                continue;
            }
            return true;
        }
    }
    false
}

pub fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1}K", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1}M", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1}G", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

pub fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    // Simple base64 decoder (standard + URL-safe)
    let table = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = Vec::new();
    let mut buf = [0u8; 4];
    let mut idx = 0;
    for c in input.bytes() {
        if c == b'=' || c == b'\n' || c == b'\r' || c == b' ' {
            continue;
        }
        let pos = table
            .iter()
            .position(|&b| b == c)
            .ok_or_else(|| "Invalid base64".to_string())?;
        buf[idx] = pos as u8;
        idx += 1;
        if idx == 4 {
            result.push((buf[0] << 2) | (buf[1] >> 4));
            if buf[2] < 64 {
                result.push((buf[1] << 4) | (buf[2] >> 2));
            }
            if buf[3] < 64 {
                result.push((buf[2] << 6) | buf[3]);
            }
            idx = 0;
        }
    }
    Ok(result)
}

/// SQL 是否只读（只读语句在审批连接上放行，写语句拦截）
/// - 白名单前缀需后跟空白/结尾（防止 SELECTOR 等误判）
/// - 不含 WITH（`WITH cte AS (...) DELETE ...` 是合法写语句，可绕过）
/// - PRAGMA 仅放行查询形式（`PRAGMA x=y` 赋值会写库，判为写）
pub fn is_read_only_sql(sql: &str) -> bool {
    let s = sql.trim_start();
    let upper = s.to_uppercase();
    let bound = |p: &str| -> bool {
        upper.starts_with(p)
            && (upper.len() == p.len() || upper.as_bytes()[p.len()].is_ascii_whitespace())
    };
    if bound("SELECT")
        || bound("SHOW")
        || bound("EXPLAIN")
        || bound("DESC")
        || bound("DESCRIBE")
        || bound("VALUES")
    {
        return true;
    }
    if upper.starts_with("PRAGMA") {
        // 取 PRAGMA 名（第一个标识符），只拦已知会修改库/会话状态的 PRAGMA
        let rest = upper["PRAGMA".len()..].trim_start();
        let name = rest
            .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
            .next()
            .unwrap_or("");
        return !matches!(
            name,
            "JOURNAL_MODE"
                | "WRITABLE_SCHEMA"
                | "SYNCHRONOUS"
                | "CACHE_SIZE"
                | "PAGE_SIZE"
                | "AUTO_VACUUM"
                | "ENCODING"
                | "FOREIGN_KEYS"
                | "TEMP_STORE"
                | "LOCKING_MODE"
                | "JOURNAL_SIZE_LIMIT"
                | "MMAP_SIZE"
                | "SECURE_DELETE"
                | "TRUSTED_SCHEMA"
                | "OPTIMIZE"
                | "VACUUM"
                | "ANALYZE"
        );
    }
    false
}
