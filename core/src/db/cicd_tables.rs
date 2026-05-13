use rusqlite::Connection;

pub fn init_cicd_tables(conn: &Connection) -> rusqlite::Result<()> {
    // ── Schema migration: backward compat ──
    // Best-effort: try to drop projectId for clean DBs, then re-add with DEFAULT for safety
    let migrations = [
        "ALTER TABLE cicd_configs DROP COLUMN projectId",
        "ALTER TABLE deploy_logs DROP COLUMN projectId",
        "ALTER TABLE deploy_history DROP COLUMN projectId",
        // If DROP didn't work (or column never existed), ensure it exists with a default
        "ALTER TABLE cicd_configs ADD COLUMN projectId TEXT DEFAULT ''",
        "ALTER TABLE deploy_logs ADD COLUMN projectId TEXT DEFAULT ''",
        "ALTER TABLE deploy_history ADD COLUMN projectId TEXT DEFAULT ''",
        // Legacy migrations (safe to re-run)
        "ALTER TABLE cicd_configs ADD COLUMN pnpmHome TEXT DEFAULT ''",
        "ALTER TABLE cicd_configs ADD COLUMN yarnHome TEXT DEFAULT ''",
    ];
    for sql in migrations {
        let _ = conn.execute(sql, []); // ignore errors
    }

    conn.execute_batch(
        r#"
        -- CI/CD configuration profiles
        CREATE TABLE IF NOT EXISTS cicd_configs (
            id TEXT PRIMARY KEY,
            projectId TEXT DEFAULT '',
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
            yarnHome TEXT
        );

        -- Deploy modules (multi-module project support)
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

        -- Deploy logs (each deployment attempt)
        CREATE TABLE IF NOT EXISTS deploy_logs (
            id TEXT PRIMARY KEY,
            projectId TEXT DEFAULT '',
            configId TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'running',
            startTime TEXT NOT NULL,
            endTime TEXT,
            errorMessage TEXT,
            progress INTEGER NOT NULL DEFAULT 0,
            triggeredBy TEXT DEFAULT 'manual',
            createdAt TEXT NOT NULL,
            logFilePath TEXT,
            artifactPaths TEXT
        );

        -- Deploy step logs (detailed step-by-step logs)
        CREATE TABLE IF NOT EXISTS deploy_step_logs (
            id TEXT PRIMARY KEY,
            deployLogId TEXT NOT NULL,
            stage TEXT NOT NULL,
            status TEXT NOT NULL,
            message TEXT,
            timestamp TEXT NOT NULL
        );

        -- Deploy history (quick history list)
        CREATE TABLE IF NOT EXISTS deploy_history (
            id TEXT PRIMARY KEY,
            configId TEXT NOT NULL,
            projectId TEXT DEFAULT '',
            status TEXT NOT NULL,
            deployedAt TEXT NOT NULL,
            rolledBack INTEGER NOT NULL DEFAULT 0,
            rolledBackAt TEXT
        );
        "#,
    )?;
    Ok(())
}
