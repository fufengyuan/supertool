import mysql from 'mysql2/promise';
import { Client } from 'pg';
import Redis from 'ioredis';
export interface DBConfig {
    id: string;
    name: string;
    type: 'mysql' | 'postgresql' | 'redis' | 'sqlite';
    host: string;
    port: number;
    user?: string;
    password?: string;
    database?: string;
    path?: string;
    dbIndex?: number;
    requiresApproval?: boolean;
}
export interface ColumnInfo {
    name: string;
    type: string;
    length: number | null;
    decimals: number | null;
    nullable: boolean;
    defaultValue: string | null;
    isPrimaryKey: boolean;
    isAutoIncrement: boolean;
    comment?: string;
    ordinalPosition: number;
}
export interface IndexInfo {
    name: string;
    columns: string[];
    isUnique: boolean;
    isPrimary: boolean;
}
export interface TableStructure {
    tableName: string;
    columns: ColumnInfo[];
    indexes: IndexInfo[];
    primaryKey: string[];
    comment?: string;
}
export interface StructureDiffItem {
    tableName: string;
    diffType: 'table_only_in_source' | 'table_only_in_target' | 'column_added' | 'column_removed' | 'column_modified' | 'index_added' | 'index_removed' | 'index_modified' | 'primary_key_changed';
    sourceValue?: any;
    targetValue?: any;
    sql: string;
}
export interface StructureSyncResult {
    diffs: StructureDiffItem[];
    sourceTables: string[];
    targetTables: string[];
    commonTables: string[];
}
export interface ColumnDef {
    name: string;
    type: string;
    nullable: boolean;
    defaultValue?: string | null;
    comment?: string;
    primaryKey?: boolean;
    autoIncrement?: boolean;
}
export interface IndexDef {
    name: string;
    columns: string[];
    unique?: boolean;
    type?: string;
}
export type AlterOperationType = 'addColumn' | 'dropColumn' | 'renameColumn' | 'modifyColumn' | 'addIndex' | 'dropIndex' | 'renameTable';
export interface AlterOperation {
    type: AlterOperationType;
    column?: ColumnDef;
    oldColumn?: string;
    newColumn?: string;
    columnName?: string;
    indexDef?: IndexDef;
    indexName?: string;
    newName?: string;
}
export interface DataSyncOptions {
    sourceConnectionId: string;
    targetConnectionId: string;
    tableName: string;
    primaryKeys: string[];
    columns: string[];
    mode: 'full' | 'insert_only' | 'update_only';
    batchSize?: number;
}
export interface DataDiffItem {
    diffType: 'insert' | 'update' | 'delete';
    primaryKey: Record<string, any>;
    sourceRow?: Record<string, any>;
    targetRow?: Record<string, any>;
}
export interface DataSyncResult {
    diffs: DataDiffItem[];
    totalInserts: number;
    totalUpdates: number;
    totalDeletes: number;
    estimatedTime?: string;
}
export interface DataSyncExecuteOptions {
    sourceConnectionId: string;
    targetConnectionId: string;
    tableName: string;
    primaryKeys: string[];
    /** Table's actual primary keys (from dbGetTablePrimaryKeys). When compare keys differ from PKs,
     *  PKs are excluded from INSERT/UPDATE to avoid polluting auto-increment IDs. */
    tablePrimaryKeys?: string[];
    columns: string[];
    diffs: DataDiffItem[];
    useTransaction?: boolean;
    batchSize?: number;
    sourceDbName?: string;
    targetDbName?: string;
}
export interface DataSyncExecuteResult {
    success: boolean;
    inserted: number;
    updated: number;
    deleted: number;
    errors: string[];
    duration: number;
}
export interface FilterCondition {
    id: string;
    column: string;
    operator: string;
    value: string;
    value2?: string;
    logic: 'AND' | 'OR';
}
export interface FilteredQueryOptions {
    connectionId: string;
    tableName: string;
    columns?: string[];
    filters: FilterCondition[];
    limit: number;
    offset: number;
    dbName?: string;
}
export type DBClient = mysql.Connection | Client | Redis | any;
export interface ScanState {
    cursor: string;
    folders: Map<string, number>;
    leaves: Set<string>;
    finished: boolean;
    timer: any;
}
export interface StreamScanState {
    cursor: string;
    foundStreams: any[];
    finished: boolean;
    timer: any;
}
