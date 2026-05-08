"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.registerDbHandlers = registerDbHandlers;
const logger_1 = require("./logger");
const electron_1 = require("electron");
const db_backup_manager_1 = __importDefault(require("./services/db-backup-manager"));
let backupManager = null;
function registerDbHandlers(dbManager, db, decryptPassword) {
    backupManager = new db_backup_manager_1.default(dbManager);
    // ============ Database Manager ============
    electron_1.ipcMain.handle('db:connect', async (_event, config) => {
        try {
            await dbManager.connect(config.id, config);
            return { success: true };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:disconnect', async (_event, id) => {
        try {
            await dbManager.disconnect(id);
            return { success: true };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:query', async (_event, id, sql) => {
        try {
            const conns = require('./uds-api');
            // Get connection config to check approval
            // The connections are stored in settings as 'db_connections'
            const rawConns = db.getSetting('db_connections');
            const connsList = rawConns ? JSON.parse(rawConns) : [];
            const conn = connsList.find((c) => c.id === id);
            if (conn && conn.requiresApproval) {
                return { success: false, requiresApproval: true, message: `数据库「${conn.name}」已开启安全审核，请在 GUI 中确认后执行。` };
            }
            const trimmed = sql.trim().toUpperCase();
            const dangerousPatterns = ['DELETE', 'DROP', 'INSERT', 'UPDATE', 'ALTER', 'CREATE', 'EXEC', 'EXECUTE', 'TRUNCATE', 'GRANT', 'REVOKE'];
            for (const pattern of dangerousPatterns) {
                const cleaned = trimmed.replace(/^\/\*[\s\S]*?\*\/\s*/g, '').replace(/^--[^\n]*\n\s*/g, '').trim().toUpperCase();
                if (cleaned.startsWith(pattern) || cleaned.startsWith('/*')) {
                    const firstWord = cleaned.split(/\s+/)[0];
                    if (dangerousPatterns.includes(firstWord)) {
                        return { success: false, error: `Only SELECT queries allowed. Blocked: ${firstWord}` };
                    }
                }
            }
            if (!trimmed.startsWith('SELECT') && !trimmed.startsWith('EXPLAIN') && !trimmed.startsWith('WITH') && !trimmed.startsWith('PRAGMA') && !trimmed.startsWith('DESCRIBE') && !trimmed.startsWith('DESC ') && !trimmed.startsWith('SHOW ')) {
                return { success: false, error: 'Only SELECT queries allowed' };
            }
            const rows = await dbManager.query(id, sql);
            return { success: true, rows };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:get-tables', async (_event, id, dbName) => {
        try {
            const tables = await dbManager.getTables(id, dbName);
            return { success: true, tables };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:get-databases', async (_event, id) => {
        try {
            const databases = await dbManager.getDatabases(id);
            return { success: true, databases };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:get-views', async (_event, id, dbName) => {
        try {
            const views = await dbManager.getViews(id, dbName);
            return { success: true, views };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:get-table-data', async (_event, id, table, limit, offset, dbName, orderBy, orderDir) => {
        try {
            const result = await dbManager.getTableData(id, table, limit, offset, dbName, orderBy, orderDir);
            // Sanitize: 数据库行可能包含 BigInt/Buffer 等不可克隆类型
            const sanitized = JSON.parse(JSON.stringify(result, (_k, v) => typeof v === 'bigint' ? v.toString() : (typeof v === 'object' && v !== null && v.type === 'Buffer' && Array.isArray(v.data) ? Buffer.from(v.data).toString('base64') : v)));
            return { success: true, ...sanitized };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:test', async (_event, config) => {
        return dbManager.testConnection(config);
    });
    // ============ Table Row CRUD Operations ============
    electron_1.ipcMain.handle('db:get-create-sql', async (_event, id, table, dbName) => {
        try {
            const sql = await dbManager.getCreateTableSql(id, table, dbName);
            return { success: true, sql };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:get-table-primary-keys', async (_event, id, table, dbName) => {
        try {
            const pks = await dbManager.getTablePrimaryKeys(id, table, dbName);
            return { success: true, primaryKeys: pks };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:update-table-row', async (_event, id, table, oldRow, newRow, dbName) => {
        try {
            await dbManager.updateTableRow(id, table, oldRow, newRow, dbName);
            return { success: true };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:insert-table-row', async (_event, id, table, row, dbName) => {
        try {
            await dbManager.insertTableRow(id, table, row, dbName);
            return { success: true };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:delete-table-row', async (_event, id, table, row, dbName) => {
        try {
            await dbManager.deleteTableRow(id, table, row, dbName);
            return { success: true };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    // ============ Redis ============
    electron_1.ipcMain.handle('db:redis-keys', async (_event, id, pattern) => {
        try {
            const result = await dbManager.getRedisKeys(id, pattern);
            return { success: true, ...result };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:redis-get', async (_event, id, key) => {
        try {
            const value = await dbManager.getValue(id, key);
            return { success: true, value };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:redis-key-info', async (_event, id, key) => {
        try {
            const info = await dbManager.getRedisKeyInfo(id, key);
            return { success: true, ...info };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:redis-key-value', async (_event, id, key) => {
        try {
            const value = await dbManager.getRedisKeyValue(id, key);
            return { success: true, value };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:redis-set-key', async (_event, id, key, type, value) => {
        try {
            const result = await dbManager.setRedisKey(id, key, type, value);
            return { success: true, result };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:redis-add-key', async (_event, id, key, type, value) => {
        try {
            const result = await dbManager.addRedisKey(id, key, type, value);
            return { success: true, result };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:redis-delete-key', async (_event, id, key) => {
        try {
            const result = await dbManager.deleteRedisKey(id, key);
            return { success: true, result };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:redis-exec', async (_event, id, command) => {
        try {
            const result = await dbManager.execRedisCommand(id, command);
            return { success: true, result };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:redis-databases', async (_event, id) => {
        try {
            const databases = await dbManager.getRedisDatabases(id);
            return { success: true, databases };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:redis-keys-by-type', async (_event, id, dbIndex, pattern) => {
        try {
            const keysByType = await dbManager.getRedisKeysByType(id, dbIndex, pattern);
            return { success: true, keysByType };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    // Get Redis keys tree (Incremental scan with state management)
    electron_1.ipcMain.handle('db:redis-keys-tree', async (_event, id, dbIndex, prefix, loadMore = false) => {
        (0, logger_1.info)(`[IPC] db:redis-keys-tree → id=${id}, dbIndex=${dbIndex}, prefix="${prefix}", loadMore=${loadMore}`);
        try {
            const tree = await dbManager.getRedisKeysTree(id, dbIndex, prefix, loadMore);
            (0, logger_1.info)(`[IPC] db:redis-keys-tree → success, folders=${tree.folders.length}, leaves=${tree.leaves.length}, hasMore=${tree.hasMore}`);
            return { success: true, ...tree };
        }
        catch (error) {
            console.error(`[IPC] db:redis-keys-tree → error: ${error.message}`, error.stack);
            return { success: false, error: error.message };
        }
    });
    // Scan keys by pattern and type (for discovering delay:* ZSet)
    electron_1.ipcMain.handle('db:redis-scan-keys', async (_event, id, dbIndex, pattern, type) => {
        try {
            const config = dbManager.getConfig(id);
            if (!config)
                throw new Error(`Connection '${id}' not found`);
            if (config.type !== 'redis')
                throw new Error('Only available for Redis connections');
            const Redis = require('ioredis');
            const tempClient = new Redis({
                host: config.host,
                port: config.port,
                password: config.password ? decryptPassword(config.password) : undefined,
                db: dbIndex ?? config.dbIndex ?? 0,
                connectTimeout: 5000,
                retryStrategy: () => null,
            });
            await tempClient.ping();
            try {
                const keys = [];
                let cursor = '0';
                const scanArgs = ['MATCH', pattern, 'COUNT', 1000];
                if (type)
                    scanArgs.push('TYPE', type);
                do {
                    const result = await tempClient.scan(cursor, ...scanArgs);
                    if (!result)
                        break;
                    cursor = result[0];
                    keys.push(...(result[1] || []));
                } while (cursor !== '0');
                return { success: true, keys };
            }
            finally {
                try {
                    tempClient.quit();
                }
                catch { }
            }
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    // ============ Database Structure Sync ============
    electron_1.ipcMain.handle('db:get-table-structure', async (_event, id, tableName, dbName) => {
        try {
            const structure = await dbManager.getTableStructure(id, tableName, dbName);
            return { success: true, structure };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:compare-structures', async (_event, sourceId, targetId, tableName, sourceDbName, targetDbName) => {
        try {
            const result = await dbManager.compareStructures(sourceId, targetId, tableName, sourceDbName, targetDbName);
            return { success: true, ...result };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:execute-structure-sync', async (_event, targetId, sqls, targetDbName) => {
        try {
            const result = await dbManager.executeStructureSync(targetId, sqls, targetDbName);
            return { success: result.success, executed: result.executed, errors: result.errors };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    // ============ Database Data Sync ============
    electron_1.ipcMain.handle('db:compare-data', async (_event, sourceId, targetId, tableName, primaryKeys, columns, sourceDbName, targetDbName, tablePrimaryKeys) => {
        try {
            const result = await dbManager.compareData(sourceId, targetId, tableName, primaryKeys, columns, sourceDbName, targetDbName, tablePrimaryKeys);
            // Sanitize for IPC: 数据库行可能包含 BigInt/Buffer/Date 等不可克隆类型
            // JSON.stringify 能处理 Buffer→{type,data[]}、Date→ISO string，BigInt 需 replacer
            const sanitized = JSON.parse(JSON.stringify(result, (_k, v) => typeof v === 'bigint' ? v.toString() : (v instanceof Buffer ? v.toString('base64') : v)));
            return { success: true, result: sanitized };
        }
        catch (error) {
            console.error('[db:compare-data] Error:', error.message, error.stack?.slice(0, 500));
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:execute-data-sync', async (_event, options) => {
        try {
            const result = await dbManager.executeDataSync(options);
            return { success: result.success, inserted: result.inserted, updated: result.updated, deleted: result.deleted, errors: result.errors, duration: result.duration };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    // ============ Database Filtered Query ============
    electron_1.ipcMain.handle('db:get-table-data-filtered', async (_event, options) => {
        try {
            const result = await dbManager.getTableDataFiltered(options);
            // Sanitize: 数据库行可能包含 BigInt/Buffer 等不可克隆类型
            const sanitized = JSON.parse(JSON.stringify(result, (_k, v) => typeof v === 'bigint' ? v.toString() : v));
            return { success: true, ...sanitized };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    // ============ Table Structure Alteration ============
    electron_1.ipcMain.handle('db:add-column', async (_event, id, dbName, tableName, column) => {
        try {
            const result = await dbManager.addColumn(id, dbName, tableName, column);
            return { success: true, sql: result.sql };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:modify-column', async (_event, id, dbName, tableName, oldColumn, newColumn) => {
        try {
            const result = await dbManager.modifyColumn(id, dbName, tableName, oldColumn, newColumn);
            return { success: true, sql: result.sql };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:drop-column', async (_event, id, dbName, tableName, columnName) => {
        try {
            const result = await dbManager.dropColumn(id, dbName, tableName, columnName);
            return { success: true, sql: result.sql };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:rename-column', async (_event, id, dbName, tableName, oldName, newName) => {
        try {
            const result = await dbManager.renameColumn(id, dbName, tableName, oldName, newName);
            return { success: true, sql: result.sql };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:add-index', async (_event, id, dbName, tableName, indexDef) => {
        try {
            const result = await dbManager.addIndex(id, dbName, tableName, indexDef);
            return { success: true, sql: result.sql };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:drop-index', async (_event, id, dbName, tableName, indexName) => {
        try {
            const result = await dbManager.dropIndex(id, dbName, tableName, indexName);
            return { success: true, sql: result.sql };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:rename-table', async (_event, id, dbName, oldName, newName) => {
        try {
            const result = await dbManager.renameTable(id, dbName, oldName, newName);
            return { success: true, sql: result.sql };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:alter-table', async (_event, id, dbName, tableName, operations) => {
        try {
            const result = await dbManager.alterTable(id, dbName, tableName, operations);
            return { success: result.success, sqls: result.sqls, errors: result.errors };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    // ============ Database Backup/Restore ============
    electron_1.ipcMain.handle('db:backup:create', async (_event, connectionId, dbName, objects, outputPath) => {
        try {
            if (!backupManager)
                return { success: false, error: 'Backup manager not initialized' };
            const result = await backupManager.createBackup(connectionId, dbName, objects, outputPath);
            return result;
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('db:backup:restore', async (_event, connectionId, nb3FilePath) => {
        try {
            if (!backupManager)
                return { success: false, errors: ['Backup manager not initialized'] };
            const result = await backupManager.restoreBackup(connectionId, nb3FilePath);
            return result;
        }
        catch (error) {
            return { success: false, errors: [error.message] };
        }
    });
    electron_1.ipcMain.handle('db:backup:list', async (_event, connectionId) => {
        try {
            if (!backupManager)
                return [];
            const result = await backupManager.listBackups(connectionId);
            return result;
        }
        catch (error) {
            return [];
        }
    });
    electron_1.ipcMain.handle('db:backup:delete', async (_event, filePath) => {
        try {
            if (!backupManager)
                return { success: false, error: 'Backup manager not initialized' };
            const result = await backupManager.deleteBackup(filePath);
            return result;
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
} // end registerDbHandlers
//# sourceMappingURL=db-handlers.js.map