"use strict";
const db_core_1 = require("./db-core");
function rowToCicdConfig(row) {
    return {
        id: row.id,
        projectId: row.projectId,
        deployBranch: row.deployBranch,
        mavenSettings: row.mavenSettings,
        mavenProfile: row.mavenProfile,
        deployPath: row.deployPath,
        libSeparate: row.libSeparate === 1,
        restartScript: row.restartScript,
        healthCheckUrl: row.healthCheckUrl,
        healthCheckTimeout: row.healthCheckTimeout,
        createdAt: row.createdAt,
        updatedAt: row.updatedAt,
        buildTool: row.buildTool,
        buildCommand: row.buildCommand,
        buildPath: row.buildPath,
        repoUrl: row.repoUrl,
        localPath: row.localPath,
        npmScript: row.npmScript,
        npmCustomScript: row.npmCustomScript,
        mavenHome: row.mavenHome,
        npmHome: row.npmHome,
        javaHome: row.javaHome,
        nodeHome: row.nodeHome,
        servers: row.servers,
        groupName: row.groupName || '未分组',
        name: row.name || '',
        lastDeployedAt: row.lastDeployedAt || null,
        parentBuildMode: row.parentBuildMode === 1,
        parentBuildPath: row.parentBuildPath || '',
        requiresApproval: row.requiresApproval === 1,
    };
}
function getAllCicdConfigs() {
    const stmt = (0, db_core_1.getDatabase)().prepare('SELECT * FROM cicd_configs ORDER BY lastDeployedAt DESC, updatedAt DESC');
    return stmt.all().map(rowToCicdConfig);
}
function getCicdGroups() {
    const rows = (0, db_core_1.getDatabase)().prepare('SELECT DISTINCT groupName FROM cicd_configs ORDER BY groupName').all();
    return rows.map(r => r.groupName || '未分组');
}
function getCicdConfig(projectId) {
    const stmt = (0, db_core_1.getDatabase)().prepare('SELECT * FROM cicd_configs WHERE projectId = ?');
    const row = stmt.get(projectId);
    if (!row)
        return null;
    return rowToCicdConfig(row);
}
function getCicdConfigByConfigId(configId) {
    const stmt = (0, db_core_1.getDatabase)().prepare('SELECT * FROM cicd_configs WHERE id = ?');
    const row = stmt.get(configId);
    if (!row)
        return null;
    return rowToCicdConfig(row);
}
function addCicdConfig(config) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    INSERT INTO cicd_configs (id, projectId, name, deployBranch, mavenSettings, mavenProfile, deployPath, libSeparate, restartScript, healthCheckUrl, healthCheckTimeout, createdAt, updatedAt, buildTool, buildCommand, buildPath, repoUrl, localPath, npmScript, npmCustomScript, mavenHome, npmHome, javaHome, nodeHome, servers, groupName, parentBuildMode, parentBuildPath, requiresApproval)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);
    stmt.run(config.id, config.projectId, config.name || null, config.deployBranch || 'main', config.mavenSettings || null, config.mavenProfile || 'prod', config.deployPath || '/', config.libSeparate ? 1 : 0, config.restartScript || './restart.sh', config.healthCheckUrl || null, config.healthCheckTimeout || 30, config.createdAt, config.updatedAt, config.buildTool || null, config.buildCommand || null, config.buildPath || null, config.repoUrl || null, config.localPath || null, config.npmScript || null, config.npmCustomScript || null, config.mavenHome || null, config.npmHome || null, config.javaHome || null, config.nodeHome || null, config.servers || null, config.groupName || '未分组', config.parentBuildMode ? 1 : 0, config.parentBuildPath || '', config.requiresApproval ? 1 : 0);
    return rowToCicdConfig((0, db_core_1.getDatabase)().prepare('SELECT * FROM cicd_configs WHERE id = ?').get(config.id));
}
// 更新配置的最后部署时间（用于智能排序）
function touchCicdConfigDeploy(configId) {
    const now = new Date().toISOString();
    (0, db_core_1.getDatabase)().prepare('UPDATE cicd_configs SET lastDeployedAt = ? WHERE id = ?').run(now, configId);
}
function updateCicdConfig(config) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    UPDATE cicd_configs SET
      projectId = ?,
      name = ?,
      deployBranch = ?,
      mavenSettings = ?,
      mavenProfile = ?,
      deployPath = ?,
      libSeparate = ?,
      restartScript = ?,
      healthCheckUrl = ?,
      healthCheckTimeout = ?,
      updatedAt = ?,
      buildTool = ?,
      buildCommand = ?,
      buildPath = ?,
      repoUrl = ?,
      localPath = ?,
      npmScript = ?,
      npmCustomScript = ?,
      mavenHome = ?,
      npmHome = ?,
      javaHome = ?,
      nodeHome = ?,
      servers = ?,
      groupName = ?,
      parentBuildMode = ?,
      parentBuildPath = ?,
      requiresApproval = ?
    WHERE id = ?
  `);
    stmt.run(config.projectId, config.name || null, config.deployBranch, config.mavenSettings, config.mavenProfile, config.deployPath || '/', config.libSeparate ? 1 : 0, config.restartScript, config.healthCheckUrl, config.healthCheckTimeout, config.updatedAt, config.buildTool || null, config.buildCommand || null, config.buildPath || null, config.repoUrl || null, config.localPath || null, config.npmScript || null, config.npmCustomScript || null, config.mavenHome || null, config.npmHome || null, config.javaHome || null, config.nodeHome || null, config.servers || null, config.groupName || '未分组', config.parentBuildMode ? 1 : 0, config.parentBuildPath || '', config.requiresApproval ? 1 : 0, config.id);
    return getCicdConfigByConfigId(config.id);
}
function deleteCicdConfig(configId) {
    const stmt = (0, db_core_1.getDatabase)().prepare('DELETE FROM cicd_configs WHERE id = ?');
    stmt.run(configId);
    return { success: true };
}
function rowToDeployModule(row) {
    return {
        id: row.id,
        configId: row.configId,
        moduleName: row.moduleName,
        modulePath: row.modulePath,
        artifactName: row.artifactName,
        artifactType: row.artifactType,
        deployOrder: row.deployOrder,
        deployPath: row.deployPath,
        enabled: row.enabled === 1,
        createdAt: row.createdAt,
        updatedAt: row.updatedAt,
        buildCommand: row.buildCommand,
        buildPath: row.buildPath,
        outputPath: row.outputPath,
        buildTool: row.buildTool,
        libFilterRules: row.libFilterRules,
    };
}
function getDeployModules(configId) {
    const stmt = (0, db_core_1.getDatabase)().prepare('SELECT * FROM deploy_modules WHERE configId = ? ORDER BY deployOrder ASC');
    const rows = stmt.all(configId);
    return rows.map(rowToDeployModule);
}
function addDeployModule(module) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    INSERT INTO deploy_modules (id, configId, moduleName, modulePath, artifactName, artifactType, deployOrder, deployPath, enabled, createdAt, updatedAt, buildCommand, buildPath, outputPath, buildTool, libFilterRules)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);
    stmt.run(module.id, module.configId, module.moduleName ?? '', module.modulePath ?? '', module.artifactName || null, module.artifactType || null, module.deployOrder ?? 0, module.deployPath || null, module.enabled !== false ? 1 : 0, module.createdAt || new Date().toISOString(), module.updatedAt || new Date().toISOString(), module.buildCommand || null, module.buildPath || null, module.outputPath || null, module.buildTool || null, module.libFilterRules || null);
    return getDeployModules(module.configId);
}
function updateDeployModule(module) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    UPDATE deploy_modules SET
      moduleName = ?,
      modulePath = ?,
      artifactName = ?,
      artifactType = ?,
      deployOrder = ?,
      deployPath = ?,
      enabled = ?,
      updatedAt = ?,
      buildCommand = ?,
      buildPath = ?,
      outputPath = ?,
      buildTool = ?,
      libFilterRules = ?
    WHERE id = ?
  `);
    stmt.run(module.moduleName, module.modulePath, module.artifactName, module.artifactType || null, module.deployOrder, module.deployPath || null, module.enabled ? 1 : 0, module.updatedAt, module.buildCommand || null, module.buildPath || null, module.outputPath || null, module.buildTool || null, module.libFilterRules || null, module.id);
    return getDeployModules(module.configId);
}
function deleteDeployModule(moduleId) {
    const stmt = (0, db_core_1.getDatabase)().prepare('DELETE FROM deploy_modules WHERE id = ?');
    stmt.run(moduleId);
    return { success: true };
}
function rowToDeployLog(row) {
    return {
        id: row.id,
        projectId: row.projectId,
        configId: row.configId,
        status: row.status,
        startTime: row.startTime,
        endTime: row.endTime,
        currentStep: row.currentStep,
        progress: row.progress,
        errorMessage: row.errorMessage,
        logOutput: row.logOutput,
        triggeredBy: row.triggeredBy,
        createdAt: row.createdAt,
        logFilePath: row.logFilePath,
        artifactPaths: row.artifactPaths
    };
}
function getDeployLogs(projectId, limit = 20) {
    const stmt = (0, db_core_1.getDatabase)().prepare('SELECT * FROM deploy_logs WHERE projectId = ? ORDER BY createdAt DESC LIMIT ?');
    const rows = stmt.all(projectId, limit);
    return rows.map(rowToDeployLog);
}
function getDeployLogById(id) {
    const row = (0, db_core_1.getDatabase)().prepare('SELECT * FROM deploy_logs WHERE id = ?').get(id);
    return row ? rowToDeployLog(row) : undefined;
}
function addDeployLog(log) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    INSERT INTO deploy_logs (id, projectId, configId, status, startTime, currentStep, progress, triggeredBy, createdAt, logFilePath, artifactPaths)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);
    stmt.run(log.id, log.projectId, log.configId, log.status || 'pending', log.startTime || null, log.currentStep || null, log.progress || 0, log.triggeredBy || 'manual', log.createdAt, log.logFilePath || null, log.artifactPaths || null);
    return getDeployLogs(log.projectId, 1)[0];
}
function updateDeployLog(log) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    UPDATE deploy_logs SET
      status = ?,
      endTime = ?,
      currentStep = ?,
      progress = ?,
      errorMessage = ?,
      logOutput = ?,
      logFilePath = ?,
      artifactPaths = ?
    WHERE id = ?
  `);
    stmt.run(log.status, log.endTime, log.currentStep, log.progress, log.errorMessage, log.logOutput, log.logFilePath, log.artifactPaths, log.id);
    return { success: true };
}
function rowToDeployStepLog(row) {
    return {
        id: row.id,
        deployLogId: row.deployLogId,
        stepName: row.stepName,
        stepOrder: row.stepOrder,
        status: row.status,
        startTime: row.startTime,
        endTime: row.endTime,
        output: row.output,
        errorMessage: row.errorMessage,
        createdAt: row.createdAt
    };
}
function getDeployStepLogs(deployLogId) {
    const stmt = (0, db_core_1.getDatabase)().prepare('SELECT * FROM deploy_step_logs WHERE deployLogId = ? ORDER BY stepOrder ASC');
    const rows = stmt.all(deployLogId);
    return rows.map(rowToDeployStepLog);
}
function addDeployStepLog(stepLog) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    INSERT INTO deploy_step_logs (id, deployLogId, stepName, stepOrder, status, startTime, createdAt)
    VALUES (?, ?, ?, ?, ?, ?, ?)
  `);
    stmt.run(stepLog.id, stepLog.deployLogId, stepLog.stepName, stepLog.stepOrder, stepLog.status || 'pending', stepLog.startTime || null, stepLog.createdAt);
    return { success: true };
}
function updateDeployStepLog(stepLog) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    UPDATE deploy_step_logs SET
      status = ?,
      endTime = ?,
      output = ?,
      errorMessage = ?
    WHERE id = ?
  `);
    stmt.run(stepLog.status, stepLog.endTime, stepLog.output, stepLog.errorMessage, stepLog.id);
    return { success: true };
}
function rowToDeployHistory(row) {
    return {
        id: row.id,
        configId: row.configId,
        projectId: row.projectId,
        status: row.status,
        version: row.version,
        gitCommit: row.gitCommit,
        deployedAt: row.deployedAt,
        rolledBack: row.rolledBack === 1,
        rolledBackAt: row.rolledBackAt
    };
}
function getDeployHistory(projectId, limit = 20) {
    const stmt = (0, db_core_1.getDatabase)().prepare('SELECT * FROM deploy_history WHERE projectId = ? ORDER BY deployedAt DESC LIMIT ?');
    const rows = stmt.all(projectId, limit);
    return rows.map(rowToDeployHistory);
}
function addDeployHistory(record) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    INSERT INTO deploy_history (id, configId, projectId, status, version, gitCommit, deployedAt, rolledBack, rolledBackAt)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);
    stmt.run(record.id, record.configId, record.projectId, record.status || 'pending', record.version || null, record.gitCommit || null, record.deployedAt || new Date().toISOString(), record.rolledBack ? 1 : 0, record.rolledBackAt || null);
    return getDeployHistory(record.projectId, 1)[0];
}
function updateDeployRecord(id, updates) {
    const fields = [];
    const values = [];
    if (updates.status !== undefined) {
        fields.push('status = ?');
        values.push(updates.status);
    }
    if (updates.version !== undefined) {
        fields.push('version = ?');
        values.push(updates.version);
    }
    if (updates.gitCommit !== undefined) {
        fields.push('gitCommit = ?');
        values.push(updates.gitCommit);
    }
    if (updates.deployedAt !== undefined) {
        fields.push('deployedAt = ?');
        values.push(updates.deployedAt);
    }
    if (updates.rolledBack !== undefined) {
        fields.push('rolledBack = ?');
        values.push(updates.rolledBack ? 1 : 0);
    }
    if (updates.rolledBackAt !== undefined) {
        fields.push('rolledBackAt = ?');
        values.push(updates.rolledBackAt);
    }
    if (fields.length === 0)
        return { success: true };
    values.push(id);
    const stmt = (0, db_core_1.getDatabase)().prepare(`UPDATE deploy_history SET ${fields.join(', ')} WHERE id = ?`);
    stmt.run(...values);
    return { success: true };
}
module.exports = {
    rowToCicdConfig,
    getAllCicdConfigs,
    getCicdGroups,
    getCicdConfig,
    getCicdConfigByConfigId,
    addCicdConfig,
    touchCicdConfigDeploy,
    updateCicdConfig,
    deleteCicdConfig,
    rowToDeployModule,
    getDeployModules,
    addDeployModule,
    updateDeployModule,
    deleteDeployModule,
    rowToDeployLog,
    getDeployLogs,
    getDeployLogById,
    addDeployLog,
    updateDeployLog,
    rowToDeployStepLog,
    getDeployStepLogs,
    addDeployStepLog,
    updateDeployStepLog,
    rowToDeployHistory,
    getDeployHistory,
    addDeployHistory,
    updateDeployRecord,
};
//# sourceMappingURL=db-cicd.js.map