import { type CicdConfig, type DeployModule, type DeployLog, type DeployStepLog, type DeployHistory } from './db-core';
declare function rowToCicdConfig(row: any): CicdConfig;
declare function getAllCicdConfigs(): CicdConfig[];
declare function getCicdGroups(): string[];
declare function getCicdConfig(projectId: string): CicdConfig | null;
declare function getCicdConfigByConfigId(configId: string): CicdConfig | null;
declare function addCicdConfig(config: CicdConfig): CicdConfig | null;
declare function touchCicdConfigDeploy(configId: string): void;
declare function updateCicdConfig(config: CicdConfig): CicdConfig | null;
declare function deleteCicdConfig(configId: string): {
    success: boolean;
};
declare function rowToDeployModule(row: any): DeployModule;
declare function getDeployModules(configId: string): DeployModule[];
declare function addDeployModule(module: DeployModule): DeployModule[];
declare function updateDeployModule(module: DeployModule): DeployModule[];
declare function deleteDeployModule(moduleId: string): {
    success: boolean;
};
declare function rowToDeployLog(row: any): DeployLog;
declare function getDeployLogs(projectId: string, limit?: number): DeployLog[];
declare function getDeployLogById(id: string): DeployLog | undefined;
declare function addDeployLog(log: DeployLog): DeployLog;
declare function updateDeployLog(log: {
    id: string;
    status?: string;
    endTime?: string;
    currentStep?: string;
    progress?: number;
    errorMessage?: string;
    logOutput?: string;
    logFilePath?: string;
    artifactPaths?: string;
}): {
    success: boolean;
};
declare function rowToDeployStepLog(row: any): DeployStepLog;
declare function getDeployStepLogs(deployLogId: string): DeployStepLog[];
declare function addDeployStepLog(stepLog: DeployStepLog): {
    success: boolean;
};
declare function updateDeployStepLog(stepLog: {
    id: string;
    status?: string;
    endTime?: string;
    output?: string;
    errorMessage?: string;
}): {
    success: boolean;
};
declare function rowToDeployHistory(row: any): DeployHistory;
declare function getDeployHistory(projectId: string, limit?: number): DeployHistory[];
declare function addDeployHistory(record: DeployHistory): DeployHistory;
declare function updateDeployRecord(id: string, updates: {
    status?: string;
    version?: string;
    gitCommit?: string;
    deployedAt?: string;
    rolledBack?: boolean;
    rolledBackAt?: string;
}): {
    success: boolean;
};
declare const _default: {
    rowToCicdConfig: typeof rowToCicdConfig;
    getAllCicdConfigs: typeof getAllCicdConfigs;
    getCicdGroups: typeof getCicdGroups;
    getCicdConfig: typeof getCicdConfig;
    getCicdConfigByConfigId: typeof getCicdConfigByConfigId;
    addCicdConfig: typeof addCicdConfig;
    touchCicdConfigDeploy: typeof touchCicdConfigDeploy;
    updateCicdConfig: typeof updateCicdConfig;
    deleteCicdConfig: typeof deleteCicdConfig;
    rowToDeployModule: typeof rowToDeployModule;
    getDeployModules: typeof getDeployModules;
    addDeployModule: typeof addDeployModule;
    updateDeployModule: typeof updateDeployModule;
    deleteDeployModule: typeof deleteDeployModule;
    rowToDeployLog: typeof rowToDeployLog;
    getDeployLogs: typeof getDeployLogs;
    getDeployLogById: typeof getDeployLogById;
    addDeployLog: typeof addDeployLog;
    updateDeployLog: typeof updateDeployLog;
    rowToDeployStepLog: typeof rowToDeployStepLog;
    getDeployStepLogs: typeof getDeployStepLogs;
    addDeployStepLog: typeof addDeployStepLog;
    updateDeployStepLog: typeof updateDeployStepLog;
    rowToDeployHistory: typeof rowToDeployHistory;
    getDeployHistory: typeof getDeployHistory;
    addDeployHistory: typeof addDeployHistory;
    updateDeployRecord: typeof updateDeployRecord;
};
export = _default;
