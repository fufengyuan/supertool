interface BackupObject {
    type: 'table' | 'view';
    name: string;
    includeData?: boolean;
}
interface BackupFile {
    file: string;
    name: string;
    size: number;
    backupTime: string;
    objects: BackupObject[];
    connectionType: string;
    connectionName: string;
    databaseName: string;
}
export interface BackupCreateResult {
    success: boolean;
    file?: string;
    size?: number;
    error?: string;
}
export interface BackupRestoreResult {
    success: boolean;
    executed: number;
    errors: string[];
}
export default class DBBackupManager {
    private dbManager;
    private backupDir;
    constructor(dbManager: any);
    createBackup(connectionId: string, dbName: string, objects: BackupObject[], outputPath?: string): Promise<BackupCreateResult>;
    restoreBackup(connectionId: string, nb3FilePath: string): Promise<BackupRestoreResult>;
    listBackups(connectionId?: string): Promise<BackupFile[]>;
    deleteBackup(filePath: string): Promise<{
        success: boolean;
        error?: string;
    }>;
    private getConnectionType;
    private getConnectionName;
    private generateFileName;
    private generateInsertSql;
    private formatSqlValue;
    private quoteIdentifier;
}
export {};
