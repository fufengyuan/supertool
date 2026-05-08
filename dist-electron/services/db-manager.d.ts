import Redis from 'ioredis';
import type { DBConfig, TableStructure, StructureSyncResult, ColumnDef, IndexDef, AlterOperation, DataSyncResult, DataSyncExecuteOptions, DataSyncExecuteResult, FilterCondition, FilteredQueryOptions } from './db-manager-types';
export type { DBConfig, ColumnInfo, IndexInfo, TableStructure, StructureDiffItem, StructureSyncResult, ColumnDef, IndexDef, AlterOperationType, AlterOperation, DataSyncOptions, DataDiffItem, DataSyncResult, DataSyncExecuteOptions, DataSyncExecuteResult, FilterCondition, FilteredQueryOptions, DBClient, ScanState, StreamScanState, } from './db-manager-types';
declare class DBManager {
    private connections;
    private scanStates;
    private streamScanStates;
    connect(id: string, config: DBConfig): Promise<void>;
    disconnect(id: string): Promise<void>;
    query(id: string, sql: string): Promise<any[]>;
    getTables(id: string, dbName?: string): Promise<string[]>;
    getDatabases(id: string): Promise<string[]>;
    getViews(id: string, dbName: string): Promise<string[]>;
    getTableData(id: string, table: string, limit: number, offset: number, dbName?: string, orderBy?: string, orderDir?: 'asc' | 'desc'): Promise<{
        rows: any[];
        total: number;
    }>;
    testConnection(config: DBConfig): Promise<{
        success: boolean;
        error?: string;
    }>;
    getRedisKeys(id: string, pattern: string, count?: number): Promise<{
        keys: string[];
        cursor: string;
    }>;
    getRedisKeyInfo(id: string, key: string): Promise<{
        type: string;
        ttl: number;
        length: number;
    }>;
    getRedisKeyValue(id: string, key: string): Promise<any>;
    setRedisKey(id: string, key: string, type: string, value: any): Promise<boolean>;
    addRedisKey(id: string, key: string, type: string, value?: any): Promise<boolean>;
    deleteRedisKey(id: string, key: string): Promise<boolean>;
    execRedisCommand(id: string, command: string): Promise<any>;
    getRedisDatabases(id: string): Promise<Array<{
        db: number;
        keys: number;
    }>>;
    getRedisKeysByType(id: string, dbIndex: number, pattern?: string, maxKeysPerType?: number): Promise<Record<string, string[]>>;
    getRedisKeysTree(id: string, dbIndex: number, prefix?: string, loadMore?: boolean): Promise<{
        folders: Array<{
            name: string;
            isFolder: true;
            count: number;
        }>;
        leaves: Array<{
            name: string;
            isFolder: false;
            type: string;
        }>;
        hasMore: boolean;
    }>;
    private scanChunk;
    private resetStateTimer;
    getRedisStreamsIncremental(id: string, dbIndex: number, pattern?: string, loadMore?: boolean): Promise<{
        streams: any[];
        hasMore: boolean;
    }>;
    getKeys(id: string, pattern: string): Promise<string[]>;
    private scanKeys;
    getValue(id: string, key: string): Promise<any>;
    private createClient;
    getConfig(id: string): DBConfig | undefined;
    isConnected(id: string): boolean;
    /** Get Redis client from connection pool with type checking */
    getRedisClient(id: string): Redis;
    /** Check if Redis connection is alive (responds to PING) */
    isRedisConnected(id: string): Promise<boolean>;
    getTableStructure(id: string, tableName: string, dbName?: string): Promise<TableStructure>;
    getCreateTableSql(id: string, tableName: string, dbName?: string): Promise<string>;
    compareStructures(sourceId: string, targetId: string, tableName: string, sourceDbName?: string, targetDbName?: string): Promise<StructureSyncResult>;
    /** Compare structure of a single table and push diffs */
    private compareTableStructure;
    executeStructureSync(targetId: string, sqls: string[], targetDbName?: string): Promise<{
        success: boolean;
        executed: number;
        errors: string[];
    }>;
    compareData(sourceId: string, targetId: string, tableName: string, primaryKeys: string[], columns: string[], sourceDbName?: string, targetDbName?: string, tablePrimaryKeys?: string[]): Promise<DataSyncResult>;
    executeDataSync(options: DataSyncExecuteOptions): Promise<DataSyncExecuteResult>;
    private getAllTableData;
    private getRowKey;
    private extractPrimaryKey;
    private rowsDiffer;
    private compareColumns;
    /** Build column type string with length/precision/scale for MySQL */
    private buildColumnType;
    /** Escape a default value for SQL */
    private formatDefaultValue;
    private generateCreateTableSql;
    private generateAddColumnSql;
    private generateModifyColumnSql;
    private generateCreateIndexSql;
    private escapeSqlValue;
    private generateInsertSql;
    private generateUpdateSql;
    private generateDeleteSql;
    private quoteIdentifier;
    private quoteTableName;
    private executeSql;
    getTablePrimaryKeys(id: string, table: string, dbName?: string): Promise<string[]>;
    updateTableRow(id: string, table: string, oldRow: Record<string, any>, newRow: Record<string, any>, dbName?: string): Promise<boolean>;
    insertTableRow(id: string, table: string, row: Record<string, any>, dbName?: string): Promise<boolean>;
    deleteTableRow(id: string, table: string, row: Record<string, any>, dbName?: string): Promise<boolean>;
    /**
     * Generate column definition SQL fragment for ADD COLUMN / MODIFY COLUMN
     */
    private buildColumnDef;
    /**
     * Build full table reference like `db`.`table` or "schema"."table" or "table"
     */
    private buildTableRef;
    addColumn(id: string, dbName: string | undefined, tableName: string, column: ColumnDef): Promise<{
        success: boolean;
        sql: string;
    }>;
    modifyColumn(id: string, dbName: string | undefined, tableName: string, oldColumn: string, newColumn: ColumnDef): Promise<{
        success: boolean;
        sql: string;
    }>;
    dropColumn(id: string, dbName: string | undefined, tableName: string, columnName: string): Promise<{
        success: boolean;
        sql: string;
    }>;
    renameColumn(id: string, dbName: string | undefined, tableName: string, oldName: string, newName: string): Promise<{
        success: boolean;
        sql: string;
    }>;
    addIndex(id: string, dbName: string | undefined, tableName: string, indexDef: IndexDef): Promise<{
        success: boolean;
        sql: string;
    }>;
    dropIndex(id: string, dbName: string | undefined, tableName: string, indexName: string): Promise<{
        success: boolean;
        sql: string;
    }>;
    renameTable(id: string, dbName: string | undefined, oldName: string, newName: string): Promise<{
        success: boolean;
        sql: string;
    }>;
    alterTable(id: string, dbName: string | undefined, tableName: string, operations: AlterOperation[]): Promise<{
        success: boolean;
        sqls: string[];
        errors: string[];
    }>;
    /**
     * Generate SQL WHERE clause from filter conditions
     * Returns { clause, params } for parameterized queries
     */
    buildWhereClause(filters: FilterCondition[], dbType: string): {
        clause: string;
        params: any[];
    };
    getTableDataFiltered(options: FilteredQueryOptions): Promise<{
        rows: any[];
        total: number;
    }>;
    private parseValue;
    getRedisQueues(id: string, pattern?: string, count?: number): Promise<Array<{
        name: string;
        length: number;
    }>>;
    getRedisQueueMessages(id: string, queueName: string, start?: number, count?: number): Promise<Array<{
        index: number;
        value: string;
        timestamp?: string;
    }>>;
    pushToRedisQueue(id: string, queueName: string, message: string, direction?: 'left' | 'right'): Promise<number>;
    popFromRedisQueue(id: string, queueName: string, direction?: 'left' | 'right'): Promise<string | null>;
    clearRedisQueue(id: string, queueName: string): Promise<boolean>;
    publishToRedisTopic(id: string, topic: string, message: string): Promise<number>;
    getRedisTopics(id: string, pattern?: string): Promise<string[]>;
    private topicSubscriptions;
    subscribeToRedisTopic(id: string, topic: string): Promise<string>;
    unsubscribeFromRedisTopic(subId: string): Promise<boolean>;
}
declare const _default: DBManager;
export default _default;
