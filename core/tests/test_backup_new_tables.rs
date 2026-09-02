/// 验证备份导出/导入对新增补丁表（db_connections / lan_settings / audit_logs）的覆盖，
/// 以及 replace 模式「只清空备份里带键的表」的安全行为。
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use tokio::runtime::Runtime;

use supertool_core::db::Database;
use supertool_core::logic::CoreService;static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

fn setup_core(seed_extra: bool) -> (CoreService, PathBuf) {
    let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("supertool_backup_tables_test_{counter}"));
    let _ = std::fs::create_dir_all(&dir);
    let db_path = dir.join("test.db");
    let _ = std::fs::remove_file(&db_path);

    let db = Database::new(&db_path).expect("create db");

    // 预置少量种子数据，确保导出非空、导入可见
    db.conn()
        .execute(
            "INSERT INTO servers (id, name, host, port, username, password, groupId, createdAt, updatedAt) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            rusqlite::params!["srv-1", "t1", "127.0.0.1", 22, "root", "enc:abc", "", "n", "n"],
        )
        .unwrap();
    db.conn()
        .execute(
            "INSERT INTO db_connections (id, name, type, host, port, username, password, createdAt, updatedAt) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            rusqlite::params!["db-1", "pg-conn", "postgres", "10.0.0.1", 5432, "pguser", "enc:pw", "n", "n"],
        )
        .unwrap();
    db.conn()
        .execute(
            "INSERT INTO lan_settings (key, value) VALUES (?1,?2)",
            rusqlite::params!["peer_remark:user_x", "张三"],
        )
        .unwrap();
    db.conn()
        .execute(
            "INSERT INTO audit_logs (id, actor_type, actor_name, command, result, created_at) \
             VALUES (?1,?2,?3,?4,?5,?6)",
            rusqlite::params![1i64, "user", "tester", "SELECT 1", "ok", "n"],
        )
        .unwrap();

    // 控制是否额外塞一张「导出端认识、备份里可能没有」的表（用于 replace 安全测试）
    if seed_extra {
        // 模拟较新版本才有的表：nginx_deny_allows 是导出清单里的表之一。
        // 它的 presetId 外键指向 nginx_presets，需先插 preset 满足约束。
        db.conn()
            .execute(
                "INSERT INTO nginx_presets (id, name, serverId, configPath, description, groupName, isActive, createdAt, updatedAt) \
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                rusqlite::params!["preset-1", "测试预设", "", "/etc/nginx/nginx.conf", "", "default", 0, "n", "n"],
            )
            .unwrap();
        db.conn()
            .execute(
                "INSERT INTO nginx_deny_allows (id, presetId, name, ip, createdAt) \
                 VALUES (?1,?2,?3,?4,?5)",
                rusqlite::params!["deny-1", "preset-1", "测试", "1.2.3.4", "n"],
            )
            .unwrap();
    }

    let core = CoreService::new(db, dir.clone());
    (core, dir)
}

#[test]
fn backup_exports_and_restores_new_tables() {
    let rt = Runtime::new().unwrap();
    let (core, _dir) = setup_core(false);

    // 导出全部
    let data = rt.block_on(core.export_all_tables()).unwrap();
    let obj = data.as_object().unwrap();

    // 三张补丁表都在导出结果里，且非空
    assert!(
        obj.contains_key("dbConnections"),
        "导出结果应含 dbConnections，实际键: {:?}",
        obj.keys().collect::<Vec<_>>()
    );
    assert!(
        obj.contains_key("lanSettings"),
        "导出结果应含 lanSettings"
    );
    assert!(obj.contains_key("auditLogs"), "导出结果应含 auditLogs");

    let db_rows = obj["dbConnections"].as_array().unwrap();
    assert_eq!(db_rows.len(), 1);
    assert_eq!(db_rows[0]["name"], "pg-conn");

    let lan = obj["lanSettings"].as_array().unwrap();
    assert_eq!(lan.len(), 1);
    assert_eq!(lan[0]["key"], "peer_remark:user_x");

    let audit = obj["auditLogs"].as_array().unwrap();
    assert_eq!(audit.len(), 1);
    assert_eq!(audit[0]["command"], "SELECT 1");
}

#[test]
fn replace_import_only_clears_tables_present_in_backup() {
    let rt = Runtime::new().unwrap();

    // 源库：有 3 张补丁表 + servers + nginx_deny_allows
    let (src, _dir) = setup_core(true);
    let mut data = rt.block_on(src.export_all_tables()).unwrap();

    // 目标库：同样结构，但 deny-2 是本地独有的记录（备份里没有对应键时不应被清空）
    let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir2 = std::env::temp_dir().join(format!("supertool_backup_tables_test_{counter}"));
    let _ = std::fs::create_dir_all(&dir2);
    let db_path = dir2.join("test2.db");
    let _ = std::fs::remove_file(&db_path);
    let db2 = Database::new(&db_path).unwrap();
    // 本地独有数据：db_connections 表有一条本地记录 db-local，备份无此键时不得被清空
    db2.conn()
        .execute(
            "INSERT INTO db_connections (id, name, type, host, port, username, password, createdAt, updatedAt) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            rusqlite::params!["db-local", "本地连接", "mysql", "127.0.0.1", 3306, "root", "enc:pw", "n", "n"],
        )
        .unwrap();
    db2.conn()
        .execute(
            "INSERT INTO nginx_presets (id, name, serverId, configPath, description, groupName, isActive, createdAt, updatedAt) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            rusqlite::params!["preset-2", "本地预设", "", "/etc/nginx/nginx.conf", "", "default", 0, "n", "n"],
        )
        .unwrap();
    db2.conn()
        .execute(
            "INSERT INTO nginx_deny_allows (id, presetId, name, ip, createdAt) \
             VALUES (?1,?2,?3,?4,?5)",
            rusqlite::params!["deny-2", "preset-2", "本地", "9.9.9.9", "n"],
        )
        .unwrap();
    let core2 = CoreService::new(db2, dir2.clone());

    // 模拟「旧版本备份」：导出后人为删掉 dbConnections / lanSettings / auditLogs 三个键，
    // 表示这份备份在生成时根本不认识这几张表（跨版本恢复的典型场景）。
    let obj = data.as_object_mut().unwrap();
    obj.remove("dbConnections");
    obj.remove("lanSettings");
    obj.remove("auditLogs");

    // replace 导入
    let res = rt
        .block_on(core2.import_all_tables(data, "replace"))
        .unwrap();
    // (imported, skipped, errors, path_rewritten)
    assert!(res.2.is_empty(), "不应有导入错误: {:?}", res.2);

    // 备份没带 db_connections：本地数据应原样保留（不被清空也不被覆盖）
    let read = core2.db_read(|c| {
        use rusqlite::OptionalExtension;
        // db_connections 已插 db-1
        let n: i64 = c
            .query_row("SELECT COUNT(*) FROM db_connections", [], |r| r.get(0))
            .unwrap();
        // lan_settings 本测试未插 → 仍是 0；audit_logs 未插 → 0
        let lan: i64 = c
            .query_row("SELECT COUNT(*) FROM lan_settings", [], |r| r.get(0))
            .unwrap();
        let audit: i64 = c
            .query_row("SELECT COUNT(*) FROM audit_logs", [], |r| r.get(0))
            .unwrap();
        // 本地 deny-2 在 replace 后应仍在（备份无 nginx_deny_allows 键 → 不清空）
        let deny: i64 = c
            .query_row("SELECT COUNT(*) FROM nginx_deny_allows", [], |r| r.get(0))
            .optional()
            .unwrap()
            .unwrap_or(-1);
        Ok::<_, String>((n, lan, audit, deny))
    });
    let (n, lan, audit, deny) = read.unwrap().unwrap();
    // 关键安全断言：备份里没有 dbConnections 键，不得清空本地 db-1（replace 模式的基础防御）
    assert_eq!(n, 1, "备份无 dbConnections 键时，本地 db-1 应保留");
    // 备份无 nginx_deny_allows 键 → 本地 deny-2 必须保留
    assert_eq!(deny, 1, "备份无 nginx_deny_allows 键时，本地 deny-2 应保留");
    // lan/audit 从未插入，替换后仍为 0
    assert_eq!(lan, 0);
    assert_eq!(audit, 0);
}
