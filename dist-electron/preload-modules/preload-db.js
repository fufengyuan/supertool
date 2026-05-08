"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
/** Database Manager, Redis, Table CRUD, Structure/Data Sync API */
const electron_1 = require("electron");
exports.default = {
    // Database Manager
    dbConnect: (config) => electron_1.ipcRenderer.invoke('db:connect', config),
    dbDisconnect: (id) => electron_1.ipcRenderer.invoke('db:disconnect', id),
    dbQuery: (id, sql) => electron_1.ipcRenderer.invoke('db:query', id, sql),
    dbGetTables: (id, dbName) => electron_1.ipcRenderer.invoke('db:get-tables', id, dbName),
    dbGetDatabases: (id) => electron_1.ipcRenderer.invoke('db:get-databases', id),
    dbGetViews: (id, dbName) => electron_1.ipcRenderer.invoke('db:get-views', id, dbName),
    dbGetTableData: (id, table, limit, offset, dbName, orderBy, orderDir) => electron_1.ipcRenderer.invoke('db:get-table-data', id, table, limit, offset, dbName, orderBy, orderDir),
    dbTest: (config) => electron_1.ipcRenderer.invoke('db:test', config),
    // Redis
    dbRedisKeys: (id, pattern) => electron_1.ipcRenderer.invoke('db:redis-keys', id, pattern),
    dbRedisGet: (id, key) => electron_1.ipcRenderer.invoke('db:redis-get', id, key),
    dbRedisKeyInfo: (id, key) => electron_1.ipcRenderer.invoke('db:redis-key-info', id, key),
    dbRedisKeyValue: (id, key) => electron_1.ipcRenderer.invoke('db:redis-key-value', id, key),
    dbRedisSetKey: (id, key, type, value) => electron_1.ipcRenderer.invoke('db:redis-set-key', id, key, type, value),
    dbRedisAddKey: (id, key, type, value) => electron_1.ipcRenderer.invoke('db:redis-add-key', id, key, type, value),
    dbRedisDeleteKey: (id, key) => electron_1.ipcRenderer.invoke('db:redis-delete-key', id, key),
    dbRedisExec: (id, command) => electron_1.ipcRenderer.invoke('db:redis-exec', id, command),
    dbRedisDatabases: (id) => electron_1.ipcRenderer.invoke('db:redis-databases', id),
    dbRedisKeysByType: (id, dbIndex, pattern) => electron_1.ipcRenderer.invoke('db:redis-keys-by-type', id, dbIndex, pattern),
    dbRedisKeysTree: (id, dbIndex, prefix, limit) => electron_1.ipcRenderer.invoke('db:redis-keys-tree', id, dbIndex, prefix, limit),
    // Redis Streams
    dbRedisStreams: (id, dbIndex, pattern, loadMore) => electron_1.ipcRenderer.invoke('db:redis-streams', id, dbIndex, pattern, loadMore),
    dbRedisStreamInfo: (id, dbIndex, key) => electron_1.ipcRenderer.invoke('db:redis-stream-info', id, dbIndex, key),
    dbRedisStreamConsumers: (id, dbIndex, key, group) => electron_1.ipcRenderer.invoke('db:redis-stream-consumers', id, dbIndex, key, group),
    dbRedisStreamMessages: (id, dbIndex, key, start, end, count) => electron_1.ipcRenderer.invoke('db:redis-stream-messages', id, dbIndex, key, start, end, count),
    dbRedisStreamAdd: (id, dbIndex, key, fields, maxlen) => electron_1.ipcRenderer.invoke('db:redis-stream-add', id, dbIndex, key, fields, maxlen),
    dbRedisStreamDel: (id, dbIndex, key, messageId) => electron_1.ipcRenderer.invoke('db:redis-stream-del', id, dbIndex, key, messageId),
    dbRedisStreamGroupCreate: (id, dbIndex, key, group, startId) => electron_1.ipcRenderer.invoke('db:redis-stream-group-create', id, dbIndex, key, group, startId),
    dbRedisStreamGroupDestroy: (id, dbIndex, key, group) => electron_1.ipcRenderer.invoke('db:redis-stream-group-destroy', id, dbIndex, key, group),
    dbRedisStreamPending: (id, dbIndex, key, group, startId, endId, count) => electron_1.ipcRenderer.invoke('db:redis-stream-pending', id, dbIndex, key, group, startId ?? '-', endId ?? '+', count ?? 100),
    dbRedisStreamClaim: (id, dbIndex, key, group, consumer, messageId, minIdleTime) => electron_1.ipcRenderer.invoke('db:redis-stream-claim', id, dbIndex, key, group, consumer, messageId, minIdleTime),
    dbRedisStreamAck: (id, dbIndex, key, group, messageId) => electron_1.ipcRenderer.invoke('db:redis-stream-ack', id, dbIndex, key, group, messageId),
    dbRedisStreamDelete: (id, dbIndex, key) => electron_1.ipcRenderer.invoke('db:redis-stream-delete', id, dbIndex, key),
    // Redis Stream Extended
    dbRedisScanKeys: (id, dbIndex, pattern, type) => electron_1.ipcRenderer.invoke('db:redis-scan-keys', id, dbIndex, pattern, type),
    dbRedisZSetRange: (id, dbIndex, key, minScore, maxScore, count) => electron_1.ipcRenderer.invoke('db:redis-zset-range', id, dbIndex, key, minScore, maxScore, count),
    dbRedisZSetRemove: (id, dbIndex, key, value) => electron_1.ipcRenderer.invoke('db:redis-zset-remove', id, dbIndex, key, value),
    dbRedisZSetAdd: (id, dbIndex, key, value, score) => electron_1.ipcRenderer.invoke('db:redis-zset-add', id, dbIndex, key, value, score),
    dbRedisStreamTrim: (id, dbIndex, key, maxLen) => electron_1.ipcRenderer.invoke('db:redis-stream-trim', id, dbIndex, key, maxLen),
    dbRedisStreamRetry: (id, dbIndex, key, group, messageId) => electron_1.ipcRenderer.invoke('db:redis-stream-retry', id, dbIndex, key, group, messageId),
    // Table Row CRUD
    dbGetCreateSql: (id, table, dbName) => electron_1.ipcRenderer.invoke('db:get-create-sql', id, table, dbName),
    dbGetTablePrimaryKeys: (id, table, dbName) => electron_1.ipcRenderer.invoke('db:get-table-primary-keys', id, table, dbName),
    dbUpdateTableRow: (id, table, oldRow, newRow, dbName) => electron_1.ipcRenderer.invoke('db:update-table-row', id, table, oldRow, newRow, dbName),
    dbInsertTableRow: (id, table, row, dbName) => electron_1.ipcRenderer.invoke('db:insert-table-row', id, table, row, dbName),
    dbDeleteTableRow: (id, table, row, dbName) => electron_1.ipcRenderer.invoke('db:delete-table-row', id, table, row, dbName),
    // Database Structure Sync
    dbGetTableStructure: (id, tableName, dbName) => electron_1.ipcRenderer.invoke('db:get-table-structure', id, tableName, dbName),
    dbCompareStructures: (sourceId, targetId, tableName, sourceDbName, targetDbName) => electron_1.ipcRenderer.invoke('db:compare-structures', sourceId, targetId, tableName, sourceDbName, targetDbName),
    dbExecuteStructureSync: (targetId, sqls, targetDbName) => electron_1.ipcRenderer.invoke('db:execute-structure-sync', targetId, sqls, targetDbName),
    // Database Data Sync
    dbCompareData: (sourceId, targetId, tableName, primaryKeys, columns, sourceDbName, targetDbName, tablePrimaryKeys) => electron_1.ipcRenderer.invoke('db:compare-data', sourceId, targetId, tableName, primaryKeys, columns, sourceDbName, targetDbName, tablePrimaryKeys),
    dbExecuteDataSync: (options) => electron_1.ipcRenderer.invoke('db:execute-data-sync', options),
    // Database Filtered Query
    dbGetTableDataFiltered: (options) => electron_1.ipcRenderer.invoke('db:get-table-data-filtered', options),
    // Database Backup/Restore
    dbBackupCreate: (connectionId, dbName, objects, outputPath) => electron_1.ipcRenderer.invoke('db:backup:create', connectionId, dbName, objects, outputPath),
    dbBackupRestore: (connectionId, nb3FilePath) => electron_1.ipcRenderer.invoke('db:backup:restore', connectionId, nb3FilePath),
    dbBackupList: (connectionId) => electron_1.ipcRenderer.invoke('db:backup:list', connectionId),
    dbBackupDelete: (filePath) => electron_1.ipcRenderer.invoke('db:backup:delete', filePath),
    dbBackupOnProgress: (callback) => electron_1.ipcRenderer.on('db:backup:progress', callback),
    dbBackupOffProgress: (callback) => electron_1.ipcRenderer.removeListener('db:backup:progress', callback),
    // Table Structure Alteration
    dbAddColumn: (id, dbName, tableName, column) => electron_1.ipcRenderer.invoke('db:add-column', id, dbName, tableName, column),
    dbModifyColumn: (id, dbName, tableName, oldColumn, newColumn) => electron_1.ipcRenderer.invoke('db:modify-column', id, dbName, tableName, oldColumn, newColumn),
    dbDropColumn: (id, dbName, tableName, columnName) => electron_1.ipcRenderer.invoke('db:drop-column', id, dbName, tableName, columnName),
    dbRenameColumn: (id, dbName, tableName, oldName, newName) => electron_1.ipcRenderer.invoke('db:rename-column', id, dbName, tableName, oldName, newName),
    dbAddIndex: (id, dbName, tableName, indexDef) => electron_1.ipcRenderer.invoke('db:add-index', id, dbName, tableName, indexDef),
    dbDropIndex: (id, dbName, tableName, indexName) => electron_1.ipcRenderer.invoke('db:drop-index', id, dbName, tableName, indexName),
    dbRenameTable: (id, dbName, oldName, newName) => electron_1.ipcRenderer.invoke('db:rename-table', id, dbName, oldName, newName),
    dbAlterTable: (id, dbName, tableName, operations) => electron_1.ipcRenderer.invoke('db:alter-table', id, dbName, tableName, operations),
};
//# sourceMappingURL=preload-db.js.map