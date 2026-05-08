"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const electron_1 = require("electron");
const path_1 = require("path");
const fs_1 = require("fs");
const zlib_1 = require("zlib");
class DBBackupManager {
    constructor(dbManager) {
        this.dbManager = dbManager;
        this.backupDir = (0, path_1.join)(electron_1.app.getPath('userData'), 'backups');
        if (!(0, fs_1.existsSync)(this.backupDir)) {
            (0, fs_1.mkdirSync)(this.backupDir, { recursive: true });
        }
    }
    async createBackup(connectionId, dbName, objects, outputPath) {
        try {
            // Get connection info
            const tables = await this.dbManager.getTables(connectionId, dbName);
            const views = await this.dbManager.getViews(connectionId, dbName);
            const metadata = {
                version: '1.0',
                connectionType: await this.getConnectionType(connectionId),
                connectionName: await this.getConnectionName(connectionId),
                databaseName: dbName,
                backupTime: new Date().toISOString(),
                objects,
                generator: 'SuperTool Backup',
            };
            const sqlFiles = {};
            for (const obj of objects) {
                if (obj.type === 'table') {
                    // Get CREATE TABLE
                    const createSql = await this.dbManager.getCreateTableSql(connectionId, obj.name, dbName);
                    let sql = `-- Table: ${obj.name}\n`;
                    sql += `${createSql};\n\n`;
                    // Get data if requested
                    if (obj.includeData !== false) {
                        sql += `-- Data for table: ${obj.name}\n`;
                        const rows = await this.dbManager.query(connectionId, `SELECT * FROM ${this.quoteIdentifier(obj.name, metadata.connectionType)}`);
                        if (rows && rows.length > 0) {
                            for (const row of rows) {
                                sql += this.generateInsertSql(obj.name, row, metadata.connectionType);
                            }
                        }
                        sql += '\n';
                    }
                    sqlFiles[`${obj.name}.sql`] = sql;
                }
                else if (obj.type === 'view') {
                    const createSql = await this.dbManager.getCreateTableSql(connectionId, obj.name, dbName);
                    sqlFiles[`${obj.name}.sql`] = `-- View: ${obj.name}\n${createSql};\n\n`;
                }
            }
            // Pack into .nb3 format (gzipped JSON)
            const payload = {
                metadata,
                files: sqlFiles,
            };
            const jsonStr = JSON.stringify(payload, null, 2);
            const compressed = (0, zlib_1.gzipSync)(Buffer.from(jsonStr, 'utf8'));
            // Determine output path
            const fileName = outputPath || this.generateFileName(metadata);
            const fullPath = (0, path_1.join)(this.backupDir, fileName);
            (0, fs_1.writeFileSync)(fullPath, compressed);
            return {
                success: true,
                file: fullPath,
                size: compressed.length,
            };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    }
    async restoreBackup(connectionId, nb3FilePath) {
        try {
            if (!(0, fs_1.existsSync)(nb3FilePath)) {
                return { success: false, executed: 0, errors: ['Backup file not found'] };
            }
            const compressed = (0, fs_1.readFileSync)(nb3FilePath);
            const jsonStr = (0, zlib_1.gunzipSync)(compressed).toString('utf8');
            const payload = JSON.parse(jsonStr);
            const { metadata, files } = payload;
            const errors = [];
            let executed = 0;
            // First pass: create structures (tables/views without data)
            const structureSqls = [];
            const dataSqls = [];
            for (const [fileName, sqlContent] of Object.entries(files)) {
                const lines = sqlContent.split('\n');
                const isDataFile = lines.some(l => l.startsWith('-- Data for table:'));
                if (isDataFile) {
                    // Split into structure + data
                    const structurePart = lines.filter(l => !l.startsWith('-- Data') && !l.startsWith('INSERT'));
                    const dataPart = lines.filter(l => l.startsWith('INSERT'));
                    structureSqls.push(structurePart.join('\n'));
                    dataSqls.push(...dataPart);
                }
                else {
                    structureSqls.push(sqlContent);
                }
            }
            // Execute structure first
            for (const sql of structureSqls) {
                const cleaned = sql.trim();
                if (!cleaned || cleaned.startsWith('--'))
                    continue;
                try {
                    await this.dbManager.query(connectionId, cleaned);
                    executed++;
                }
                catch (e) {
                    errors.push(`Structure: ${e.message}`);
                }
            }
            // Then execute data
            for (const sql of dataSqls) {
                const cleaned = sql.trim();
                if (!cleaned)
                    continue;
                try {
                    await this.dbManager.query(connectionId, cleaned);
                    executed++;
                }
                catch (e) {
                    errors.push(`Data: ${e.message}`);
                }
            }
            return {
                success: errors.length === 0,
                executed,
                errors,
            };
        }
        catch (error) {
            return { success: false, executed: 0, errors: [error.message] };
        }
    }
    async listBackups(connectionId) {
        try {
            const files = (0, fs_1.readdirSync)(this.backupDir)
                .filter(f => f.endsWith('.nb3'))
                .map(f => ({
                name: f,
                path: (0, path_1.join)(this.backupDir, f),
                stat: (0, fs_1.statSync)((0, path_1.join)(this.backupDir, f)),
            }))
                .sort((a, b) => b.stat.mtimeMs - a.stat.mtimeMs);
            const result = [];
            for (const { name, path, stat } of files) {
                try {
                    const compressed = (0, fs_1.readFileSync)(path);
                    const jsonStr = (0, zlib_1.gunzipSync)(compressed).toString('utf8');
                    const payload = JSON.parse(jsonStr);
                    const meta = payload.metadata;
                    if (connectionId && meta.connectionName && meta.connectionName !== connectionId)
                        continue;
                    result.push({
                        file: path,
                        name,
                        size: stat.size,
                        backupTime: meta.backupTime,
                        objects: meta.objects,
                        connectionType: meta.connectionType,
                        connectionName: meta.connectionName,
                        databaseName: meta.databaseName,
                    });
                }
                catch {
                    // Skip corrupted files
                }
            }
            return result;
        }
        catch {
            return [];
        }
    }
    async deleteBackup(filePath) {
        try {
            if ((0, fs_1.existsSync)(filePath)) {
                (0, fs_1.unlinkSync)(filePath);
            }
            return { success: true };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    }
    // Private helpers
    async getConnectionType(connectionId) {
        try {
            const config = this.dbManager.getConfig?.(connectionId);
            return config?.type || 'unknown';
        }
        catch {
            return 'unknown';
        }
    }
    async getConnectionName(connectionId) {
        try {
            const config = this.dbManager.getConfig?.(connectionId);
            return config?.name || connectionId;
        }
        catch {
            return connectionId;
        }
    }
    generateFileName(meta) {
        const date = meta.backupTime.replace(/[:.]/g, '-').substring(0, 19);
        return `${meta.connectionName || 'db'}_${meta.databaseName || 'all'}_${date}.nb3`;
    }
    generateInsertSql(tableName, row, connType) {
        const columns = Object.keys(row);
        const values = Object.values(row).map(v => this.formatSqlValue(v, connType));
        const colList = columns.map(c => this.quoteIdentifier(c, connType)).join(', ');
        const valList = values.join(', ');
        return `INSERT INTO ${this.quoteIdentifier(tableName, connType)} (${colList}) VALUES (${valList});\n`;
    }
    formatSqlValue(val, connType) {
        if (val === null || val === undefined)
            return 'NULL';
        if (typeof val === 'number')
            return String(val);
        if (typeof val === 'boolean')
            return val ? '1' : '0';
        if (val instanceof Date) {
            if (connType === 'mysql')
                return `'${val.toISOString().replace('T', ' ').substring(0, 19)}'`;
            return `'${val.toISOString()}'`;
        }
        const str = String(val).replace(/'/g, "''").replace(/\\/g, '\\\\');
        return `'${str}'`;
    }
    quoteIdentifier(name, connType) {
        if (connType === 'postgresql')
            return `\"${name}\"`;
        if (connType === 'mysql')
            return `\`${name}\``;
        return `\"${name}\"`;
    }
}
exports.default = DBBackupManager;
//# sourceMappingURL=db-backup-manager.js.map