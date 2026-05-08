"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
/** CI/CD API + Events */
const electron_1 = require("electron");
exports.default = {
    getCicdConfigs: () => electron_1.ipcRenderer.invoke('cicd:get-all-configs'),
    getCicdGroups: () => electron_1.ipcRenderer.invoke('cicd:get-groups'),
    getCicdConfig: (projectId) => electron_1.ipcRenderer.invoke('cicd:get-config', projectId),
    getCicdConfigById: (configId) => electron_1.ipcRenderer.invoke('cicd:get-config-by-id', configId),
    addCicdConfig: (config) => electron_1.ipcRenderer.invoke('cicd:add-config', config),
    updateCicdConfig: (config) => electron_1.ipcRenderer.invoke('cicd:update-config', config),
    deleteCicdConfig: (configId) => electron_1.ipcRenderer.invoke('cicd:delete-config', configId),
    getDeployModules: (configId) => electron_1.ipcRenderer.invoke('cicd:get-modules', configId),
    addDeployModule: (module) => electron_1.ipcRenderer.invoke('cicd:add-module', module),
    updateDeployModule: (module) => electron_1.ipcRenderer.invoke('cicd:update-module', module),
    deleteDeployModule: (moduleId) => electron_1.ipcRenderer.invoke('cicd:delete-module', moduleId),
    getDeployLogs: (projectId, limit) => electron_1.ipcRenderer.invoke('cicd:get-logs', projectId, limit),
    getDeployStepLogs: (deployLogId) => electron_1.ipcRenderer.invoke('cicd:get-step-logs', deployLogId),
    deploy: (configId, confirmed = false) => electron_1.ipcRenderer.invoke('cicd:deploy', configId, confirmed),
    cancelDeploy: (configId) => electron_1.ipcRenderer.invoke('cicd:cancel-deploy', configId),
    rollback: (configId, deployHistoryId) => electron_1.ipcRenderer.invoke('cicd:rollback', configId, deployHistoryId),
    rollbackDeploy: (deployLogId) => electron_1.ipcRenderer.invoke('cicd:rollback-deploy', deployLogId),
    readLogFile: (filePath) => electron_1.ipcRenderer.invoke('cicd:read-log-file', filePath),
    testSsh: (config) => electron_1.ipcRenderer.invoke('cicd:test-ssh', config),
    detectBuildTools: () => electron_1.ipcRenderer.invoke('cicd:detect-tools'),
    detectToolPaths: () => electron_1.ipcRenderer.invoke('cicd:detect-tool-paths'),
    detectSdkVersions: () => electron_1.ipcRenderer.invoke('cicd:detect-sdk-versions'),
    getDeployHistory: (projectId, limit) => electron_1.ipcRenderer.invoke('cicd:get-deploy-history', projectId, limit),
    checkMavenAvailable: () => electron_1.ipcRenderer.invoke('cicd:check-maven'),
    checkJavaAvailable: (javaHome) => electron_1.ipcRenderer.invoke('cicd:check-java', javaHome),
    checkNodeAvailable: (nodeHome) => electron_1.ipcRenderer.invoke('cicd:check-node', nodeHome),
    // Module Scanner
    scanProjectModules: (projectPath) => electron_1.ipcRenderer.invoke('modules:scan', projectPath),
    // CI/CD Events
    onDeployProgress: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('cicd:progress', wrapper);
        return () => electron_1.ipcRenderer.removeListener('cicd:progress', wrapper);
    },
    onDeployNotification: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('cicd:deploy-notification', wrapper);
        return () => electron_1.ipcRenderer.removeListener('cicd:deploy-notification', wrapper);
    },
    onDeployLogIdCreated: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('cicd:deploy-log-id-created', wrapper);
        return () => electron_1.ipcRenderer.removeListener('cicd:deploy-log-id-created', wrapper);
    },
};
//# sourceMappingURL=preload-cicd.js.map