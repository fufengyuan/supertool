//! SQL 安全工具：只读 SQL 判定（CLI 审批连接与 MCP/GUI 查询共用）

/// SQL 是否只读（只读语句放行，写语句拦截）
/// - 白名单前缀需后跟空白/结尾（防止 SELECTOR 等误判）
/// - 不含 WITH（`WITH cte AS (...) DELETE ...` 是合法写语句，可绕过）
/// - PRAGMA 仅放行查询形式（`PRAGMA x=y` / `PRAGMA journal_mode(WAL)` 等改库状态，判为写）
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_read_only() {
        assert!(is_read_only_sql("SELECT * FROM t"));
        assert!(is_read_only_sql("select 1"));
        assert!(is_read_only_sql("SHOW TABLES"));
        assert!(is_read_only_sql("EXPLAIN SELECT 1"));
        assert!(is_read_only_sql("DESC t"));
        assert!(is_read_only_sql("DESCRIBE t"));
        assert!(is_read_only_sql("SELECT\n1"));
        assert!(is_read_only_sql("SELECT\t1"));
        assert!(is_read_only_sql("PRAGMA table_info(t)"));
    }

    #[test]
    fn blocks_writes() {
        assert!(!is_read_only_sql("DELETE FROM t"));
        assert!(!is_read_only_sql("UPDATE t SET a=1"));
        assert!(!is_read_only_sql("INSERT INTO t VALUES (1)"));
        assert!(!is_read_only_sql("DROP TABLE t"));
        assert!(!is_read_only_sql("ALTER TABLE t ADD COLUMN x"));
        assert!(!is_read_only_sql("TRUNCATE t"));
        // WITH 可携带写语句，不放行
        assert!(!is_read_only_sql("WITH cte AS (SELECT 1) DELETE FROM t"));
        assert!(!is_read_only_sql("WITH cte AS (SELECT 1) UPDATE t SET a=1"));
        // SELECTOR 前缀误判防护
        assert!(!is_read_only_sql("SELECTOR bad"));
        // PRAGMA 改状态形式
        assert!(!is_read_only_sql("PRAGMA journal_mode=WAL"));
        assert!(!is_read_only_sql("PRAGMA journal_mode(WAL)"));
        assert!(!is_read_only_sql("PRAGMA writable_schema(ON)"));
    }
}
