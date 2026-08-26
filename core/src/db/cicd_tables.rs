use rusqlite::Connection;

/// 删除表中指定列（兼容所有 SQLite 版本）
/// 先尝试 ALTER TABLE DROP COLUMN（3.35.0+），失败则回退到表重建
fn drop_column_if_exists(conn: &Connection, table: &str, column: &str) -> rusqlite::Result<()> {
    // 检查列是否存在
    let exists: bool = conn
        .query_row(
            &format!(
                "SELECT COUNT(*) > 0 FROM pragma_table_info('{}') WHERE name=?1",
                table
            ),
            [column],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !exists {
        return Ok(()); // 列已不存在，无需操作
    }

    // 方法一：ALTER TABLE DROP COLUMN
    let drop_sql = format!("ALTER TABLE {} DROP COLUMN {}", table, column);
    match conn.execute(&drop_sql, []) {
        Ok(_) => {
            log::info!("[Schema] Dropped column '{}' from {}", column, table);
            return Ok(());
        }
        Err(e) => {
            log::warn!(
                "[Schema] ALTER TABLE DROP COLUMN failed for {}.{}: {}. Falling back to table recreation.",
                table,
                column,
                e
            );
        }
    }

    // 方法二：表重建（兼容 SQLite < 3.35.0）
    // 1. 获取当前表的列信息（排除要删除的列）
    let mut stmt = conn.prepare(&format!(
        "SELECT name, type, \"notnull\", dflt_value, pk FROM pragma_table_info('{}') WHERE name != ?1 ORDER BY cid",
        table
    ))?;
    let cols: Vec<(String, Option<String>, i64, Option<String>, i64)> = stmt
        .query_map([column], |row| {
            Ok((
                row.get::<_, String>(0)?,         // name
                row.get::<_, Option<String>>(1)?, // type
                row.get::<_, i64>(2)?,            // notnull
                row.get::<_, Option<String>>(3)?, // dflt_value
                row.get::<_, i64>(4)?,            // pk
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();

    if cols.is_empty() {
        return Err(rusqlite::Error::InvalidColumnName(table.to_string()));
    }

    // 2. 构建新表 DDL
    let mut col_defs: Vec<String> = Vec::new();
    let mut col_names: Vec<String> = Vec::new();
    for (name, ty, notnull, dflt, pk) in &cols {
        let mut def = name.clone();
        if let Some(t) = ty {
            def.push(' ');
            def.push_str(t);
        }
        if *notnull != 0 && *pk == 0 {
            def.push_str(" NOT NULL");
        }
        if let Some(d) = dflt {
            // SQLite stores default values as strings like 'value' or NULL
            if d != "NULL" && d != "null" {
                def.push_str(&format!(" DEFAULT {}", d));
            }
        }
        if *pk != 0 {
            def.push_str(" PRIMARY KEY");
        }
        col_defs.push(def);
        col_names.push(name.clone());
    }

    let tmp_table = format!("{}_new", table);

    // 3. 执行重建
    conn.execute_batch(&format!(
        "BEGIN TRANSACTION;
         CREATE TABLE {tmp_table} ({col_defs});
         INSERT INTO {tmp_table} ({col_names}) SELECT {col_names} FROM {table};
         DROP TABLE {table};
         ALTER TABLE {tmp_table} RENAME TO {table};
         COMMIT;",
        tmp_table = tmp_table,
        col_defs = col_defs.join(", "),
        col_names = col_names.join(", "),
        table = table,
    ))?;

    log::info!(
        "[Schema] Recreated table '{}' without column '{}'",
        table,
        column
    );
    Ok(())
}

pub fn init_cicd_tables(conn: &Connection) -> rusqlite::Result<()> {
    // ── Drop unused projectId column with fallback ──
    for table in &["cicd_configs", "deploy_logs", "deploy_history"] {
        if let Err(e) = drop_column_if_exists(conn, table, "projectId") {
            log::warn!(
                "[Schema] Failed to clean up projectId from {}: {}",
                table,
                e
            );
        }
    }

    // ── Drop legacy per-config SSH columns（2026-08 清理）──
    // 多服务器架构（servers JSON 列 + servers 表）落地后，
    // cicd_configs 上的 sshHost/sshPort/sshUser/sshKeyPath/sshPassword 已无任何读写路径
    for column in &["sshHost", "sshPort", "sshUser", "sshKeyPath", "sshPassword"] {
        if let Err(e) = drop_column_if_exists(conn, "cicd_configs", column) {
            log::warn!("[Schema] Failed to clean up {} from cicd_configs: {}", column, e);
        }
    }

    // Legacy migrations (safe to re-run)
    let migrations = [
        "ALTER TABLE cicd_configs ADD COLUMN pnpmHome TEXT DEFAULT ''",
        "ALTER TABLE cicd_configs ADD COLUMN yarnHome TEXT DEFAULT ''",
        "ALTER TABLE cicd_configs ADD COLUMN buildMode TEXT DEFAULT 'local'",
        // 多环境支持：JSON 数组 [{name, deployPath, servers, envVars, healthCheckUrl, ...}]
        "ALTER TABLE cicd_configs ADD COLUMN environments TEXT",
        // 增量上传开关（默认开启）
        "ALTER TABLE cicd_configs ADD COLUMN incrementalUpload INTEGER NOT NULL DEFAULT 1",
        // 配置级健康检查重试次数（默认 3）
        "ALTER TABLE cicd_configs ADD COLUMN healthCheckRetries INTEGER NOT NULL DEFAULT 3",
        // 单体前端的产物输出目录（相对代码目录，如 build/h5；npm 收集 dist 用）
        "ALTER TABLE cicd_configs ADD COLUMN outputPath TEXT DEFAULT ''",
        // 单体（单产物）部署的 lib 分离过滤规则（每行一个通配模式，仅打包匹配依赖）
        "ALTER TABLE cicd_configs ADD COLUMN libFilterRules TEXT DEFAULT ''",
        // 部署日志记录环境名
        "ALTER TABLE deploy_logs ADD COLUMN environment TEXT",
    ];
    for sql in migrations {
        let _ = conn.execute(sql, []); // ignore "duplicate column" errors
    }

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS cicd_configs (
            id TEXT PRIMARY KEY,
            name TEXT DEFAULT '',
            deployBranch TEXT NOT NULL DEFAULT 'main',
            mavenSettings TEXT,
            mavenProfile TEXT DEFAULT 'prod',
            deployPath TEXT DEFAULT '/',
            libSeparate INTEGER NOT NULL DEFAULT 0,
            restartScript TEXT DEFAULT './restart.sh',
            healthCheckUrl TEXT,
            healthCheckTimeout INTEGER NOT NULL DEFAULT 30,
            createdAt TEXT NOT NULL,
            updatedAt TEXT NOT NULL,
            buildTool TEXT,
            buildCommand TEXT,
            buildPath TEXT,
            repoUrl TEXT,
            localPath TEXT,
            npmScript TEXT,
            npmCustomScript TEXT,
            mavenHome TEXT,
            npmHome TEXT,
            javaHome TEXT,
            nodeHome TEXT,
            servers TEXT,
            groupName TEXT DEFAULT '未分组',
            lastDeployedAt TEXT,
            parentBuildMode INTEGER NOT NULL DEFAULT 0,
            parentBuildPath TEXT DEFAULT '',
            requiresApproval INTEGER NOT NULL DEFAULT 0,
            pnpmHome TEXT,
            yarnHome TEXT,
            buildMode TEXT NOT NULL DEFAULT 'local',
            gitRepoId TEXT DEFAULT '',
            environments TEXT,
            incrementalUpload INTEGER NOT NULL DEFAULT 1,
            healthCheckRetries INTEGER NOT NULL DEFAULT 3,
            outputPath TEXT DEFAULT '',
            libFilterRules TEXT DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS deploy_modules (
            id TEXT PRIMARY KEY,
            configId TEXT NOT NULL,
            moduleName TEXT NOT NULL DEFAULT '',
            modulePath TEXT NOT NULL DEFAULT '',
            buildPath TEXT,
            buildCommand TEXT,
            buildTool TEXT,
            outputPath TEXT,
            artifactName TEXT DEFAULT '',
            artifactType TEXT DEFAULT 'jar',
            libFilterRules TEXT,
            deployOrder INTEGER NOT NULL DEFAULT 0,
            deployPath TEXT,
            enabled INTEGER NOT NULL DEFAULT 1,
            createdAt TEXT NOT NULL,
            updatedAt TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS deploy_logs (
            id TEXT PRIMARY KEY,
            configId TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'running',
            startTime TEXT NOT NULL,
            endTime TEXT,
            errorMessage TEXT,
            progress INTEGER NOT NULL DEFAULT 0,
            triggeredBy TEXT DEFAULT 'manual',
            createdAt TEXT NOT NULL,
            logFilePath TEXT,
            artifactPaths TEXT,
            environment TEXT
        );

        CREATE TABLE IF NOT EXISTS deploy_step_logs (
            id TEXT PRIMARY KEY,
            deployLogId TEXT NOT NULL,
            stage TEXT NOT NULL,
            status TEXT NOT NULL,
            message TEXT,
            timestamp TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS deploy_history (
            id TEXT PRIMARY KEY,
            configId TEXT NOT NULL,
            status TEXT NOT NULL,
            deployedAt TEXT NOT NULL,
            rolledBack INTEGER NOT NULL DEFAULT 0,
            rolledBackAt TEXT
        );
        "#,
    )?;
    Ok(())
}
