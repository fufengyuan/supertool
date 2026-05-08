"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.getDeployAbortControllers = getDeployAbortControllers;
exports.cicdDeploy = cicdDeploy;
exports.cicdCancelDeploy = cicdCancelDeploy;
exports.cicdRollback = cicdRollback;
exports.registerCicdHandlers = registerCicdHandlers;
const logger_1 = require("./logger");
const electron_1 = require("electron");
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const os = __importStar(require("os"));
const maven_detector_1 = require("./maven-detector");
const app_bootstrap_1 = require("./app-bootstrap");
const deploy_server_parser_1 = require("./deploy-server-parser");
const window_manager_1 = require("./window-manager");
const notification_manager_1 = require("./notification-manager");
// Map of configId to AbortController for cancelable deploys
const deployAbortControllers = new Map();
function getDeployAbortControllers() { return deployAbortControllers; }
// ============ Exportable CICD operation functions (used by both IPC and HTTP API) ============
async function cicdDeploy(db, configId, streamCallback) {
    const config = db.getCicdConfigByConfigId(configId);
    (0, logger_1.info)('[cicd:deploy] Loaded config:', JSON.stringify({ id: config?.id, deployPath: config?.deployPath, servers: config?.servers, projectId: config?.projectId }));
    if (!config)
        return { success: false, error: '未找到CI/CD配置' };
    db.touchCicdConfigDeploy(config.id);
    const projectId = config.projectId;
    const modules = db.getDeployModules(config.id);
    const allProjects = db.getAllProjects();
    const project = allProjects.find((p) => p.id === projectId);
    const deployLog = db.addDeployLog({
        id: Date.now().toString(), projectId, configId: config.id, status: 'running',
        startTime: new Date().toISOString(), triggeredBy: 'manual', createdAt: new Date().toISOString()
    });
    if ((0, window_manager_1.getMainWindow)())
        (0, window_manager_1.getMainWindow)().webContents.send('cicd:deploy-log-id-created', { deployLogId: deployLog.id });
    const abortController = new AbortController();
    deployAbortControllers.set(deployLog.id, abortController);
    let cicdService = null;
    let progressHandler = null;
    let notificationHandler = null;
    try {
        const repoUrl = project?.gitUrl1 || project?.repoPath;
        const localPath = config.localPath || project?.repoPath || '';
        if (!localPath && !repoUrl) {
            deployAbortControllers.delete(deployLog.id);
            return { success: false, error: '项目缺少本地路径或远程仓库地址', deployLogId: deployLog.id };
        }
        const validatedServers = (0, deploy_server_parser_1.parseDeployServers)(config.servers, config.deployPath || '', config.libSeparate);
        if ('error' in validatedServers) {
            deployAbortControllers.delete(deployLog.id);
            return { success: false, error: validatedServers.error, deployLogId: deployLog.id };
        }
        const CicdService = requireService('cicd-service');
        cicdService = new CicdService((0, app_bootstrap_1.getSuperToolDataDir)());
        progressHandler = (progress) => { if ((0, window_manager_1.getMainWindow)())
            (0, window_manager_1.getMainWindow)().webContents.send('cicd:progress', { deployLogId: deployLog.id, configId: config.id, ...progress }); streamCallback?.({ type: 'progress', data: { deployLogId: deployLog.id, configId: config.id, ...progress } }); };
        notificationHandler = (data) => {
            const projName = project?.name || '项目';
            if (data.success) {
                streamCallback?.({ type: 'deploy:notification', data: { success: true, projectName: projName, configId: config.id } });
                if (electron_1.Notification.isSupported()) {
                    (0, notification_manager_1.playNotificationSound)();
                    new electron_1.Notification({ title: '🚀 部署成功', body: `${projName} 已成功部署到 ${validatedServers.length} 个节点` }).show();
                }
                if ((0, window_manager_1.getMainWindow)())
                    (0, window_manager_1.getMainWindow)().webContents.send('cicd:deploy-notification', { success: true, projectName: projName, configId: config.id });
            }
            else {
                streamCallback?.({ type: 'deploy:notification', data: { success: false, error: data.error, cancelled: data.cancelled, configId: config.id } });
                if (electron_1.Notification.isSupported()) {
                    (0, notification_manager_1.playNotificationSound)();
                    new electron_1.Notification({ title: data.cancelled ? '⏹️ 部署已取消' : '❌ 部署失败', body: data.cancelled ? `${projName} 部署已取消` : `${projName} 部署失败: ${data.error || '未知错误'}` }).show();
                }
                if ((0, window_manager_1.getMainWindow)())
                    (0, window_manager_1.getMainWindow)().webContents.send('cicd:deploy-notification', { success: false, error: data.error, cancelled: data.cancelled, configId: config.id });
            }
        };
        cicdService.on('progress', progressHandler);
        cicdService.on('deploy:notification', notificationHandler);
        const deployConfig = {
            repoUrl: '', branch: config.deployBranch, localPath: localPath || undefined,
            buildTool: config.buildTool || '', buildCommand: config.buildCommand || '', buildPath: config.buildPath || '',
            npmScript: config.npmScript || 'build', npmCustomScript: config.npmCustomScript || '',
            mavenHome: config.mavenHome || undefined, javaHome: config.javaHome || undefined,
            npmHome: config.npmHome || undefined, nodeHome: config.nodeHome || undefined,
            mavenProfile: config.mavenProfile || undefined, mavenSettings: config.mavenSettings || undefined,
            modules: modules.filter((m) => m.enabled).map((m) => ({ name: m.moduleName, path: m.modulePath, buildPath: m.buildPath, buildCommand: m.buildCommand, buildTool: m.buildTool, outputPath: m.outputPath, artifactName: m.artifactName, artifactType: m.artifactType, libFilterRules: m.libFilterRules, deployOrder: m.deployOrder, deployPath: m.deployPath })),
            skipTests: true, parentBuildMode: config.parentBuildMode || false, parentBuildPath: config.parentBuildPath || '',
            servers: validatedServers, deployDir: config.deployPath, libDir: config.libSeparate ? config.deployPath + '/lib' : undefined,
            restartScript: config.restartScript, libSeparate: config.libSeparate
        };
        const effectiveRepoUrl = localPath ? '' : repoUrl;
        deployConfig.repoUrl = effectiveRepoUrl || repoUrl;
        const deployResult = await cicdService.deployFull(deployConfig, (0, app_bootstrap_1.getSuperToolDataDir)(), abortController.signal, deployLog.id);
        db.addDeployHistory({ id: Date.now().toString() + '-h', configId: config.id, projectId, status: 'success', deployedAt: new Date().toISOString() });
        const firstArtifact = deployResult.artifactPaths[0] || '';
        const artifactDirResolved = firstArtifact ? path.dirname(firstArtifact) : path.join((0, app_bootstrap_1.getSuperToolDataDir)(), 'deploy-artifacts', deployLog.id);
        db.updateDeployLog({ id: deployLog.id, status: 'success', endTime: new Date().toISOString(), progress: 100, logFilePath: deployResult.logFilePath, artifactPaths: JSON.stringify({ deployId: deployLog.id, artifactDir: artifactDirResolved, manifest: path.join(artifactDirResolved, 'manifest.json') }) });
        return { success: true, deployLogId: deployLog.id };
    }
    catch (error) {
        const errMsg = error instanceof Error ? error.message : String(error);
        const cancelled = errMsg === '部署已取消';
        db.updateDeployLog({ id: deployLog.id, status: cancelled ? 'cancelled' : 'failed', endTime: new Date().toISOString(), errorMessage: errMsg, logFilePath: path.join((0, app_bootstrap_1.getSuperToolDataDir)(), 'deploy-logs', deployLog.id + '.log') });
        db.addDeployHistory({ id: Date.now().toString() + '-h', configId: config.id, projectId, status: cancelled ? 'cancelled' : 'failed', deployedAt: new Date().toISOString() });
        return { success: false, error: errMsg, deployLogId: deployLog.id, ...(cancelled ? { cancelled: true } : {}) };
    }
    finally {
        deployAbortControllers.delete(deployLog.id);
        const CicdService = requireService('cicd-service');
        if (cicdService && progressHandler)
            cicdService.removeListener('progress', progressHandler);
        if (cicdService && notificationHandler)
            cicdService.removeListener('deploy:notification', notificationHandler);
    }
}
async function cicdCancelDeploy(deployLogId) {
    const controller = deployAbortControllers.get(deployLogId);
    if (controller) {
        controller.abort();
        deployAbortControllers.delete(deployLogId);
        return { success: true, message: '部署取消请求已发送' };
    }
    return { success: false, error: '未找到正在进行的部署' };
}
async function cicdRollback(db, configId, deployHistoryId) {
    const config = db.getCicdConfigByConfigId(configId);
    if (!config)
        return { success: false, error: '未找到CI/CD配置' };
    db.touchCicdConfigDeploy(config.id);
    const history = db.getDeployHistory(config.projectId, 50);
    const lastSuccess = history.find((h) => h.status === 'success' && !h.rolledBack && h.id !== deployHistoryId);
    if (!lastSuccess && !deployHistoryId)
        return { success: false, error: '未找到可回滚的成功部署记录' };
    const rollbackRecord = { id: Date.now().toString() + '-rb', configId: config.id, projectId: config.projectId, status: 'rolled-back', deployedAt: new Date().toISOString() };
    let cicdService = null;
    try {
        const CicdService = requireService('cicd-service');
        cicdService = new CicdService((0, app_bootstrap_1.getSuperToolDataDir)());
        const servers = (0, deploy_server_parser_1.parseDeployServers)(config.servers, config.deployPath || '', config.libSeparate);
        if ('error' in servers)
            return { success: false, error: servers.error };
        for (const srv of servers) {
            await cicdService.rollback(srv);
        }
        if (lastSuccess)
            db.updateDeployRecord(lastSuccess.id, { rolledBack: true, rolledBackAt: new Date().toISOString() });
        db.addDeployHistory(rollbackRecord);
        if (electron_1.Notification.isSupported() && (0, window_manager_1.getMainWindow)()) {
            (0, notification_manager_1.playNotificationSound)();
            new electron_1.Notification({ title: '🔄 回滚成功', body: '项目已回滚到上一个版本' }).show();
            (0, window_manager_1.getMainWindow)().webContents.send('cicd:deploy-notification', { success: true, message: '回滚成功', configId: config.id });
        }
        return { success: true };
    }
    catch (error) {
        const errMsg = error instanceof Error ? error.message : String(error);
        db.addDeployHistory({ ...rollbackRecord, status: 'rollback-failed' });
        if (electron_1.Notification.isSupported() && (0, window_manager_1.getMainWindow)()) {
            (0, notification_manager_1.playNotificationSound)();
            new electron_1.Notification({ title: '❌ 回滚失败', body: `回滚失败: ${errMsg}` }).show();
            (0, window_manager_1.getMainWindow)().webContents.send('cicd:deploy-notification', { success: false, error: errMsg, configId: config.id });
        }
        return { success: false, error: errMsg };
    }
}
function getServicePath() {
    if (electron_1.app.isPackaged) {
        return path.join(electron_1.app.getAppPath(), 'dist-electron', 'services');
    }
    return path.join(__dirname, 'services');
}
function requireService(name) {
    const p = path.join(getServicePath(), name);
    const mod = require(p);
    if (mod && mod.default)
        return mod.default;
    return mod;
}
const async_exec_1 = require("./async-exec");
function registerCicdHandlers(db, notifyDataChange) {
    // ============ CI/CD 构建工具检测 ============
    electron_1.ipcMain.handle('cicd:detect-tools', async () => {
        const tools = {};
        const mavenHome = await (0, maven_detector_1.detectMavenHome)();
        const cmdMap = {};
        if (mavenHome)
            cmdMap['mvn'] = path.join(mavenHome, 'bin', 'mvn');
        try {
            const nodePath = await (0, async_exec_1.tryCommand)('which node');
            if (nodePath)
                cmdMap['node'] = nodePath.stdout.trim();
            const npmPath = await (0, async_exec_1.tryCommand)('which npm');
            if (npmPath)
                cmdMap['npm'] = npmPath.stdout.trim();
        }
        catch { }
        const toolList = [
            { cmd: 'mvn', name: 'maven' },
            { cmd: 'npm', name: 'npm' },
            { cmd: 'node', name: 'node' },
            { cmd: 'java', name: 'java' },
            { cmd: 'gradle', name: 'gradle' },
            { cmd: 'pnpm', name: 'pnpm' },
            { cmd: 'yarn', name: 'yarn' },
        ];
        for (const { cmd, name } of toolList) {
            try {
                const resolvedCmd = cmdMap[cmd] || cmd;
                const result = await (0, async_exec_1.runCommand)(`${resolvedCmd} --version`, { timeout: 5000 });
                const version = result.stdout.trim().split('\n')[0];
                tools[name] = { available: true, version };
            }
            catch {
                tools[name] = { available: false };
            }
        }
        if (mavenHome)
            tools.maven.path = mavenHome;
        if (cmdMap['node'])
            tools.node.path = cmdMap['node'];
        return tools;
    });
    // ============ CI/CD 路径自动检测（whereis/which）============
    electron_1.ipcMain.handle('cicd:detect-tool-paths', async () => {
        async function findPath(cmd) {
            let result = await (0, async_exec_1.tryCommand)(`which ${cmd}`, { timeout: 2000 });
            if (result?.stdout.trim())
                return result.stdout.trim();
            result = await (0, async_exec_1.tryCommand)(`whereis ${cmd}`, { timeout: 2000 });
            if (result) {
                const parts = result.stdout.trim().split(/\s+/);
                if (parts.length > 1)
                    return parts[1];
            }
            return '';
        }
        function resolveHomeDir(cmdPath) {
            if (!cmdPath)
                return null;
            const resolved = cmdPath.replace(/^~/, os.homedir());
            if (!fs.existsSync(resolved))
                return null;
            const binIdx = resolved.lastIndexOf('/bin/');
            if (binIdx > 0)
                return resolved.slice(0, binIdx);
            return resolved;
        }
        const mvnBin = await findPath('mvn');
        const javaBin = await findPath('java');
        const nodeBin = await findPath('node');
        const npmBin = await findPath('npm');
        return {
            mavenHome: mvnBin ? resolveHomeDir(mvnBin) : '',
            javaHome: javaBin ? resolveHomeDir(javaBin) : '',
            nodeHome: nodeBin ? resolveHomeDir(nodeBin) : '',
            npmHome: npmBin ? resolveHomeDir(npmBin) : '',
        };
    });
    // ============ CI/CD 多版本工具检测（SDKMAN / NVM）============
    electron_1.ipcMain.handle('cicd:detect-sdk-versions', async () => {
        const home = os.homedir();
        const result = {
            sdkman: { java: [], maven: [], gradle: [] },
            nvm: { node: [] },
        };
        const sdkmanBase = path.join(home, '.sdkman', 'candidates');
        const sdkmanCandidates = ['java', 'maven', 'gradle'];
        for (const candidate of sdkmanCandidates) {
            const dir = path.join(sdkmanBase, candidate);
            if (!fs.existsSync(dir))
                continue;
            try {
                const currentLink = path.join(dir, 'current');
                let currentTarget = '';
                if (fs.existsSync(currentLink) && fs.lstatSync(currentLink).isSymbolicLink()) {
                    currentTarget = fs.readlinkSync(currentLink);
                }
                const entries = fs.readdirSync(dir, { withFileTypes: true });
                for (const entry of entries) {
                    if (!entry.isDirectory())
                        continue;
                    const verPath = path.join(dir, entry.name);
                    const shortName = entry.name;
                    const isCurrent = currentTarget.endsWith(entry.name) || (!!currentLink && fs.existsSync(currentLink) && fs.realpathSync(currentLink).endsWith(entry.name));
                    result.sdkman[candidate].push({
                        name: shortName,
                        path: verPath,
                        isCurrent,
                    });
                }
            }
            catch { }
        }
        const nvmDirEnv = process.env.NVM_DIR;
        const nvmPaths = [];
        if (nvmDirEnv) {
            nvmPaths.push(path.join(nvmDirEnv, 'versions', 'node'));
        }
        nvmPaths.push(path.join(home, '.nvm', 'versions', 'node'));
        nvmPaths.push('/opt/homebrew/opt/nvm/versions/node');
        nvmPaths.push('/usr/local/opt/nvm/versions/node');
        nvmPaths.push('/usr/local/nvm/versions/node');
        nvmPaths.push('/usr/share/nvm/versions/node');
        const uniquePaths = [...new Set(nvmPaths)].filter(p => fs.existsSync(p));
        const fallbackPaths = [...new Set(nvmPaths)].filter(p => !fs.existsSync(p));
        const allPaths = [...uniquePaths, ...fallbackPaths];
        const seenVersions = new Set();
        for (const nvmBase of allPaths) {
            if (!fs.existsSync(nvmBase))
                continue;
            try {
                const currentLink = path.join(nvmBase, 'current');
                let currentTarget = '';
                if (fs.existsSync(currentLink) && fs.lstatSync(currentLink).isSymbolicLink()) {
                    currentTarget = fs.readlinkSync(currentLink);
                }
                const entries = fs.readdirSync(nvmBase, { withFileTypes: true });
                for (const entry of entries) {
                    if (!entry.isDirectory())
                        continue;
                    if (entry.name === 'current')
                        continue;
                    if (seenVersions.has(entry.name))
                        continue;
                    const verPath = path.join(nvmBase, entry.name);
                    if (!fs.existsSync(path.join(verPath, 'bin', 'node')))
                        continue;
                    seenVersions.add(entry.name);
                    const isCurrent = currentTarget.endsWith(entry.name) || (!!currentLink && fs.existsSync(currentLink) && fs.realpathSync(currentLink).endsWith(entry.name));
                    result.nvm.node.push({
                        name: entry.name,
                        path: verPath,
                        isCurrent,
                    });
                }
            }
            catch { }
        }
        for (const key of ['java', 'maven', 'gradle']) {
            result.sdkman[key].sort((a, b) => {
                if (a.isCurrent)
                    return -1;
                if (b.isCurrent)
                    return 1;
                return b.name.localeCompare(a.name, undefined, { numeric: true });
            });
        }
        result.nvm.node.sort((a, b) => {
            if (a.isCurrent)
                return -1;
            if (b.isCurrent)
                return 1;
            return b.name.localeCompare(a.name, undefined, { numeric: true });
        });
        return result;
    });
    // ============ CI/CD ============
    electron_1.ipcMain.handle('cicd:get-all-configs', () => db.getAllCicdConfigs());
    electron_1.ipcMain.handle('cicd:get-groups', () => db.getCicdGroups());
    electron_1.ipcMain.handle('cicd:get-config', (_event, projectId) => db.getCicdConfig(projectId));
    electron_1.ipcMain.handle('cicd:get-config-by-id', (_event, configId) => db.getCicdConfigByConfigId(configId));
    electron_1.ipcMain.handle('cicd:add-config', (_event, config) => {
        const result = db.addCicdConfig(config);
        notifyDataChange('cicd');
        return result;
    });
    electron_1.ipcMain.handle('cicd:update-config', (_event, config) => {
        const result = db.updateCicdConfig(config);
        notifyDataChange('cicd');
        return result;
    });
    electron_1.ipcMain.handle('cicd:delete-config', (_event, configId) => {
        const result = db.deleteCicdConfig(configId);
        notifyDataChange('cicd');
        return result;
    });
    electron_1.ipcMain.handle('cicd:get-modules', (_event, configId) => db.getDeployModules(configId));
    electron_1.ipcMain.handle('cicd:add-module', (_event, module) => db.addDeployModule(module));
    electron_1.ipcMain.handle('cicd:update-module', (_event, module) => db.updateDeployModule(module));
    electron_1.ipcMain.handle('cicd:delete-module', (_event, moduleId) => db.deleteDeployModule(moduleId));
    electron_1.ipcMain.handle('cicd:get-logs', (_event, projectId, limit) => db.getDeployLogs(projectId, limit));
    electron_1.ipcMain.handle('cicd:get-step-logs', (_event, deployLogId) => db.getDeployStepLogs(deployLogId));
    electron_1.ipcMain.handle('cicd:get-deploy-history', (_event, projectId, limit) => db.getDeployHistory(projectId, limit || 20));
    electron_1.ipcMain.handle('cicd:deploy', async (_event, configId, confirmed = false) => {
        const config = db.getCicdConfigByConfigId(configId);
        (0, logger_1.info)('[cicd:deploy] Loaded config:', JSON.stringify({ id: config?.id, deployPath: config?.deployPath, servers: config?.servers, projectId: config?.projectId }));
        if (!config)
            return { success: false, error: '未找到CI/CD配置' };
        // Check if approval is required and not yet confirmed
        if (config.requiresApproval && !confirmed) {
            return { success: false, requiresApproval: true, message: '此配置需要审核确认，请确认后再次部署', configName: config.name || '' };
        }
        db.touchCicdConfigDeploy(config.id);
        const projectId = config.projectId;
        const modules = db.getDeployModules(config.id);
        const allProjects = db.getAllProjects();
        const project = allProjects.find((p) => p.id === projectId);
        const deployLog = db.addDeployLog({
            id: Date.now().toString(), projectId, configId: config.id, status: 'running',
            startTime: new Date().toISOString(), triggeredBy: 'manual', createdAt: new Date().toISOString()
        });
        if ((0, window_manager_1.getMainWindow)())
            (0, window_manager_1.getMainWindow)().webContents.send('cicd:deploy-log-id-created', { deployLogId: deployLog.id });
        const abortController = new AbortController();
        deployAbortControllers.set(deployLog.id, abortController);
        let cicdService = null;
        let progressHandler = null;
        let notificationHandler = null;
        try {
            const repoUrl = project?.gitUrl1 || project?.repoPath;
            const localPath = config.localPath || project?.repoPath || '';
            const effectiveRepoUrl = localPath ? '' : repoUrl;
            if (!localPath && !repoUrl) {
                deployAbortControllers.delete(deployLog.id);
                return { success: false, error: '项目缺少本地路径或远程仓库地址', deployLogId: deployLog.id };
            }
            const validatedServers = (0, deploy_server_parser_1.parseDeployServers)(config.servers, config.deployPath || '', config.libSeparate);
            if ('error' in validatedServers) {
                deployAbortControllers.delete(deployLog.id);
                return { success: false, error: validatedServers.error, deployLogId: deployLog.id };
            }
            const CicdService = requireService('cicd-service');
            cicdService = new CicdService((0, app_bootstrap_1.getSuperToolDataDir)());
            progressHandler = (progress) => {
                if ((0, window_manager_1.getMainWindow)())
                    (0, window_manager_1.getMainWindow)().webContents.send('cicd:progress', { deployLogId: deployLog.id, configId: config.id, ...progress });
            };
            notificationHandler = (data) => {
                if (electron_1.Notification.isSupported()) {
                    (0, notification_manager_1.playNotificationSound)();
                    const projName = project?.name || '项目';
                    if (data.success) {
                        const srvCount = config.servers ? JSON.parse(config.servers).length : 1;
                        new electron_1.Notification({ title: '🚀 部署成功', body: `${projName} 已成功部署到 ${srvCount} 个节点` }).show();
                        if ((0, window_manager_1.getMainWindow)())
                            (0, window_manager_1.getMainWindow)().webContents.send('cicd:deploy-notification', { success: true, projectName: projName, configId: config.id });
                    }
                    else {
                        new electron_1.Notification({
                            title: data.cancelled ? '⏹️ 部署已取消' : '❌ 部署失败',
                            body: data.cancelled ? `${projName} 部署已取消` : `${projName} 部署失败: ${data.error || '未知错误'}`
                        }).show();
                        if ((0, window_manager_1.getMainWindow)())
                            (0, window_manager_1.getMainWindow)().webContents.send('cicd:deploy-notification', { success: false, error: data.error, cancelled: data.cancelled, configId: config.id });
                    }
                }
            };
            cicdService.on('progress', progressHandler);
            cicdService.on('deploy:notification', notificationHandler);
            (0, logger_1.info)('[cicd:deploy] config.deployPath:', config.deployPath);
            (0, logger_1.info)('[cicd:deploy] config.servers:', config.servers);
            (0, logger_1.info)('[cicd:deploy] config.libSeparate:', config.libSeparate);
            const deployConfig = {
                repoUrl: effectiveRepoUrl || repoUrl,
                branch: config.deployBranch,
                localPath: localPath || undefined,
                buildTool: config.buildTool || '',
                buildCommand: config.buildCommand || '',
                buildPath: config.buildPath || '',
                npmScript: config.npmScript || 'build',
                npmCustomScript: config.npmCustomScript || '',
                mavenHome: config.mavenHome || undefined,
                javaHome: config.javaHome || undefined,
                npmHome: config.npmHome || undefined,
                nodeHome: config.nodeHome || undefined,
                mavenProfile: config.mavenProfile || undefined,
                mavenSettings: config.mavenSettings || undefined,
                modules: modules.filter((m) => m.enabled).map((m) => ({
                    name: m.moduleName,
                    path: m.modulePath,
                    buildPath: m.buildPath,
                    buildCommand: m.buildCommand,
                    buildTool: m.buildTool,
                    outputPath: m.outputPath,
                    artifactName: m.artifactName,
                    artifactType: m.artifactType,
                    libFilterRules: m.libFilterRules,
                    deployOrder: m.deployOrder,
                    deployPath: m.deployPath,
                })),
                skipTests: true,
                parentBuildMode: config.parentBuildMode || false,
                parentBuildPath: config.parentBuildPath || '',
                servers: validatedServers,
                deployDir: config.deployPath,
                libDir: config.libSeparate ? config.deployPath + '/lib' : undefined,
                restartScript: config.restartScript, libSeparate: config.libSeparate
            };
            const deployResult = await cicdService.deployFull(deployConfig, (0, app_bootstrap_1.getSuperToolDataDir)(), abortController.signal, deployLog.id);
            db.addDeployHistory({
                id: Date.now().toString() + '-h', configId: config.id, projectId,
                status: 'success', deployedAt: new Date().toISOString()
            });
            const firstArtifact = deployResult.artifactPaths[0] || '';
            const artifactDirResolved = firstArtifact ? path.dirname(firstArtifact) : path.join((0, app_bootstrap_1.getSuperToolDataDir)(), 'deploy-artifacts', deployLog.id);
            db.updateDeployLog({
                id: deployLog.id, status: 'success', endTime: new Date().toISOString(), progress: 100,
                logFilePath: deployResult.logFilePath,
                artifactPaths: JSON.stringify({ deployId: deployLog.id, artifactDir: artifactDirResolved, manifest: path.join(artifactDirResolved, 'manifest.json') }),
            });
            return { success: true, deployLogId: deployLog.id };
        }
        catch (error) {
            const errMsg = error instanceof Error ? error.message : String(error);
            const cancelled = errMsg === '部署已取消';
            const failedLogFile = path.join((0, app_bootstrap_1.getSuperToolDataDir)(), 'deploy-logs', deployLog.id + '.log');
            db.updateDeployLog({ id: deployLog.id, status: cancelled ? 'cancelled' : 'failed', endTime: new Date().toISOString(), errorMessage: errMsg, logFilePath: failedLogFile });
            db.addDeployHistory({
                id: Date.now().toString() + '-h', configId: config.id, projectId,
                status: cancelled ? 'cancelled' : 'failed', deployedAt: new Date().toISOString()
            });
            return { success: false, error: errMsg, deployLogId: deployLog.id, cancelled };
        }
        finally {
            deployAbortControllers.delete(deployLog.id);
            const CicdService = requireService('cicd-service');
            if (cicdService && progressHandler)
                cicdService.removeListener('progress', progressHandler);
            if (cicdService && notificationHandler)
                cicdService.removeListener('deploy:notification', notificationHandler);
        }
    });
    electron_1.ipcMain.handle('cicd:cancel-deploy', async (_event, deployLogId) => {
        const controller = deployAbortControllers.get(deployLogId);
        if (controller) {
            controller.abort();
            deployAbortControllers.delete(deployLogId);
            return { success: true, message: '部署取消请求已发送' };
        }
        return { success: false, error: '未找到正在进行的部署' };
    });
    // Rollback: deploy using saved artifacts (skip build)
    electron_1.ipcMain.handle('cicd:rollback-deploy', async (_event, deployLogId) => {
        const deployLog = db.getDeployLogById(deployLogId);
        if (!deployLog)
            return { success: false, error: '部署日志不存在' };
        if (deployLog.status !== 'success')
            return { success: false, error: '只能回滚到成功的部署记录' };
        const config = db.getCicdConfigByConfigId(deployLog.configId);
        if (!config)
            return { success: false, error: '未找到CI/CD配置' };
        // Check if approval is required
        if (config.requiresApproval) {
            return { success: false, requiresApproval: true, message: '此配置已开启部署审核，请在 GUI 中确认后回滚' };
        }
        db.touchCicdConfigDeploy(config.id);
        const servers = (0, deploy_server_parser_1.parseDeployServers)(config.servers, config.deployPath || '', config.libSeparate);
        if ('error' in servers)
            return { success: false, error: servers.error };
        const rollbackLog = db.addDeployLog({
            id: Date.now().toString(), projectId: deployLog.projectId, configId: config.id,
            status: 'running', startTime: new Date().toISOString(), triggeredBy: 'rollback',
            createdAt: new Date().toISOString(),
        });
        if ((0, window_manager_1.getMainWindow)())
            (0, window_manager_1.getMainWindow)().webContents.send('cicd:deploy-log-id-created', { deployLogId: rollbackLog.id });
        const abortController = new AbortController();
        deployAbortControllers.set(rollbackLog.id, abortController);
        let cicdService = null;
        let progressHandler = null;
        let notificationHandler = null;
        try {
            const CicdService = requireService('cicd-service');
            cicdService = new CicdService((0, app_bootstrap_1.getSuperToolDataDir)());
            let artifactDir = null;
            if (deployLog.artifactPaths) {
                try {
                    const parsed = typeof deployLog.artifactPaths === 'string' ? JSON.parse(deployLog.artifactPaths) : deployLog.artifactPaths;
                    artifactDir = parsed.artifactDir || null;
                }
                catch { }
            }
            if (!artifactDir) {
                deployAbortControllers.delete(rollbackLog.id);
                return { success: false, error: '未找到产物路径，无法回滚' };
            }
            progressHandler = (progress) => {
                if ((0, window_manager_1.getMainWindow)())
                    (0, window_manager_1.getMainWindow)().webContents.send('cicd:progress', { deployLogId: rollbackLog.id, configId: deployLog.configId, ...progress });
            };
            notificationHandler = (data) => {
                if (electron_1.Notification.isSupported()) {
                    (0, notification_manager_1.playNotificationSound)();
                    const projName = db.getAllProjects().find((p) => p.id === deployLog.projectId)?.name || '项目';
                    if (data.success) {
                        new electron_1.Notification({ title: '🔄 回滚成功', body: `${projName} 已回滚到历史版本` }).show();
                        if ((0, window_manager_1.getMainWindow)())
                            (0, window_manager_1.getMainWindow)().webContents.send('cicd:deploy-notification', { success: true, message: '回滚成功', configId: deployLog.configId });
                    }
                    else {
                        new electron_1.Notification({ title: '❌ 回滚失败', body: `${projName} 回滚失败: ${data.error || '未知错误'}` }).show();
                        if ((0, window_manager_1.getMainWindow)())
                            (0, window_manager_1.getMainWindow)().webContents.send('cicd:deploy-notification', { success: false, error: data.error, cancelled: data.cancelled, configId: deployLog.configId });
                    }
                }
            };
            cicdService.on('progress', progressHandler);
            cicdService.on('deploy:notification', notificationHandler);
            const config2 = db.getCicdConfigByConfigId(deployLog.configId);
            const servers2 = (0, deploy_server_parser_1.parseDeployServers)(config2.servers, config2?.deployPath || '', config2?.libSeparate);
            if ('error' in servers2) {
                deployAbortControllers.delete(rollbackLog.id);
                cicdService.removeListener('progress', progressHandler);
                cicdService.removeListener('deploy:notification', notificationHandler);
                return { success: false, error: servers2.error };
            }
            // 并行回滚多台服务器
            cicdService.emit('progress', { stage: 'rollback', status: 'info', message: `并行回滚 ${servers2.length} 个服务器` });
            const rollbackResults = await Promise.allSettled(servers2.map(async (srv, i) => {
                const label = srv.label || `服务器 ${i + 1}`;
                cicdService.emit('progress', { stage: 'rollback', status: 'info', message: `回滚 ${label} (${srv.host})` });
                await cicdService.rollbackWithArtifacts(srv, artifactDir, abortController.signal);
                if (config2?.restartScript) {
                    cicdService.emit('progress', { stage: 'rollback', status: 'info', message: `在 ${label} 执行重启脚本` });
                    await cicdService.executeRestartScript(srv, config2.restartScript);
                }
            }));
            // 检查回滚失败
            const failedRollbacks = [];
            for (let i = 0; i < rollbackResults.length; i++) {
                const srv = servers2[i];
                const label = srv.label || `服务器 ${i + 1}`;
                const result = rollbackResults[i];
                if (result.status === 'rejected') {
                    failedRollbacks.push(`${label} (${srv.host}): ${result.reason instanceof Error ? result.reason.message : String(result.reason)}`);
                }
            }
            if (failedRollbacks.length > 0) {
                throw new Error(`部分服务器回滚失败:\n${failedRollbacks.join('\n')}`);
            }
            db.addDeployHistory({
                id: Date.now().toString() + '-h', configId: config.id, projectId: deployLog.projectId,
                status: 'success', deployedAt: new Date().toISOString()
            });
            const logFile = path.join((0, app_bootstrap_1.getSuperToolDataDir)(), 'deploy-logs', rollbackLog.id + '.log');
            const artifactDir2 = path.join((0, app_bootstrap_1.getSuperToolDataDir)(), 'deploy-artifacts', rollbackLog.id);
            db.updateDeployLog({
                id: rollbackLog.id, status: 'success', endTime: new Date().toISOString(), progress: 100,
                logFilePath: logFile,
                artifactPaths: JSON.stringify({ deployId: rollbackLog.id, artifactDir: artifactDir2, manifest: path.join(artifactDir2, 'manifest.json') }),
            });
            db.updateDeployLog({ id: deployLog.id, status: 'rolled_back' });
            cicdService.emit('deploy-complete', { deployId: rollbackLog.id, success: true });
            cicdService.emit('deploy:notification', { success: true, deployId: rollbackLog.id });
            return { success: true, deployLogId: rollbackLog.id };
        }
        catch (error) {
            const errMsg = error instanceof Error ? error.message : String(error);
            const cancelled = errMsg === '部署已取消';
            const status = cancelled ? 'cancelled' : 'failed';
            cicdService.emit('deploy-complete', { deployId: rollbackLog.id, success: false, error: errMsg, cancelled });
            cicdService.emit('deploy:notification', { success: false, deployId: rollbackLog.id, error: errMsg, cancelled });
            db.updateDeployLog({ id: rollbackLog.id, status: status, endTime: new Date().toISOString(), errorMessage: errMsg,
                logFilePath: path.join((0, app_bootstrap_1.getSuperToolDataDir)(), 'deploy-logs', rollbackLog.id + '.log')
            });
            db.addDeployHistory({
                id: Date.now().toString() + '-h', configId: config.id, projectId: deployLog.projectId,
                status: cancelled ? 'cancelled' : 'failed', deployedAt: new Date().toISOString()
            });
            return { success: false, error: errMsg, deployLogId: rollbackLog.id, cancelled };
        }
        finally {
            deployAbortControllers.delete(rollbackLog.id);
            if (cicdService && progressHandler)
                cicdService.removeListener('progress', progressHandler);
            if (cicdService && notificationHandler)
                cicdService.removeListener('deploy:notification', notificationHandler);
        }
    });
    // Read deployment log file from disk
    electron_1.ipcMain.handle('cicd:read-log-file', async (_event, filePath) => {
        try {
            if (!fs.existsSync(filePath))
                return { success: false, error: '日志文件不存在: ' + filePath };
            const content = fs.readFileSync(filePath, 'utf-8');
            return { success: true, content };
        }
        catch (error) {
            return { success: false, error: error instanceof Error ? error.message : String(error) };
        }
    });
    electron_1.ipcMain.handle('cicd:rollback', async (_event, configId, deployHistoryId) => {
        const config = db.getCicdConfigByConfigId(configId);
        if (!config)
            return { success: false, error: '未找到CI/CD配置' };
        // Check if approval is required
        if (config.requiresApproval) {
            return { success: false, requiresApproval: true, message: '此配置已开启部署审核，CLI 不支持回滚，请在 GUI 中操作' };
        }
        db.touchCicdConfigDeploy(config.id);
        const history = db.getDeployHistory(config.projectId, 50);
        const lastSuccess = history.find((h) => h.status === 'success' && !h.rolledBack && h.id !== deployHistoryId);
        if (!lastSuccess && !deployHistoryId)
            return { success: false, error: '未找到可回滚的成功部署记录' };
        const rollbackRecord = { id: Date.now().toString() + '-rb', configId: config.id, projectId: config.projectId, status: 'rolled-back', deployedAt: new Date().toISOString() };
        let cicdService = null;
        let rollbackProgress = null;
        try {
            const CicdService = requireService('cicd-service');
            cicdService = new CicdService((0, app_bootstrap_1.getSuperToolDataDir)());
            rollbackProgress = (progress) => { if ((0, window_manager_1.getMainWindow)())
                (0, window_manager_1.getMainWindow)().webContents.send('cicd:progress', { configId: config.id, ...progress }); };
            cicdService.on('progress', rollbackProgress);
            const servers = (0, deploy_server_parser_1.parseDeployServers)(config.servers, config.deployPath || '', config.libSeparate);
            if ('error' in servers) {
                if (cicdService && rollbackProgress)
                    cicdService.removeListener('progress', rollbackProgress);
                return { success: false, error: servers.error };
            }
            for (const srv of servers) {
                await cicdService.rollback(srv);
            }
            if (lastSuccess)
                db.updateDeployRecord(lastSuccess.id, { rolledBack: true, rolledBackAt: new Date().toISOString() });
            db.addDeployHistory(rollbackRecord);
            if (cicdService && rollbackProgress)
                cicdService.removeListener('progress', rollbackProgress);
            if (electron_1.Notification.isSupported() && (0, window_manager_1.getMainWindow)()) {
                (0, notification_manager_1.playNotificationSound)();
                new electron_1.Notification({ title: '🔄 回滚成功', body: '项目已回滚到上一个版本' }).show();
                (0, window_manager_1.getMainWindow)().webContents.send('cicd:deploy-notification', { success: true, message: '回滚成功', configId: config.id });
            }
            return { success: true };
        }
        catch (error) {
            if (cicdService && rollbackProgress)
                cicdService.removeListener('progress', rollbackProgress);
            const errMsg2 = error instanceof Error ? error.message : String(error);
            db.addDeployHistory({ ...rollbackRecord, status: 'rollback-failed' });
            if (electron_1.Notification.isSupported() && (0, window_manager_1.getMainWindow)()) {
                (0, notification_manager_1.playNotificationSound)();
                new electron_1.Notification({ title: '❌ 回滚失败', body: `回滚失败: ${errMsg2}` }).show();
                (0, window_manager_1.getMainWindow)().webContents.send('cicd:deploy-notification', { success: false, error: errMsg2, configId: config.id });
            }
            return { success: false, error: errMsg2 };
        }
    });
    electron_1.ipcMain.handle('cicd:test-ssh', async (_event, config) => {
        const CicdService = requireService('cicd-service');
        const cicdService = new CicdService((0, app_bootstrap_1.getSuperToolDataDir)());
        try {
            let host, port, username, password, privateKey;
            if (config.sshHost) {
                host = config.sshHost;
                port = config.sshPort || 22;
                username = config.sshUser;
                password = config.sshPassword;
                privateKey = config.sshKeyPath ? fs.readFileSync(config.sshKeyPath) : undefined;
            }
            else {
                const servers = (0, deploy_server_parser_1.parseDeployServers)(config.servers, config.deployPath || '', config.libSeparate);
                if ('error' in servers)
                    return { success: false, error: servers.error };
                host = servers[0].host;
                port = servers[0].port;
                username = servers[0].username;
                password = servers[0].password;
                privateKey = servers[0].privateKey;
            }
            const conn = await new Promise((resolve, reject) => {
                const { Client } = require('ssh2');
                const conn = new Client();
                conn.on('ready', () => resolve(conn));
                conn.on('error', (err) => reject(err));
                conn.connect({ host, port, username, password, privateKey });
            });
            const output = await cicdService.sshExec(conn, 'echo "SSH连接测试成功" && uname -a');
            conn.end();
            return { success: true, output: output.trim() };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('cicd:check-maven', async () => {
        const { spawn } = require('child_process');
        const mavenHome = await (0, maven_detector_1.detectMavenHome)();
        const mvnPath = mavenHome ? path.join(mavenHome, 'bin', 'mvn') : 'mvn';
        return new Promise((resolve) => {
            const proc = spawn(mvnPath, ['-v'], { timeout: 10000 });
            let output = '';
            proc.stdout.on('data', (data) => { output += data.toString(); });
            proc.stderr.on('data', (data) => { output += data.toString(); });
            proc.on('close', (code) => {
                if (code === 0) {
                    const versionMatch = output.match(/Apache Maven ([\d.]+)/);
                    resolve({ success: true, version: versionMatch ? versionMatch[1] : 'unknown' });
                }
                else
                    resolve({ success: false, error: 'Maven 未安装或不在 PATH 中' });
            });
            proc.on('error', (err) => resolve({ success: false, error: `无法执行 Maven 命令: ${err.message}` }));
        });
    });
    electron_1.ipcMain.handle('cicd:check-java', async (_event, javaHome) => {
        const { spawn } = require('child_process');
        const javaBin = javaHome ? path.join(javaHome, 'bin', 'java') : 'java';
        return new Promise((resolve) => {
            const proc = spawn(javaBin, ['-version'], { timeout: 10000 });
            let output = '';
            proc.stdout.on('data', (data) => { output += data.toString(); });
            proc.stderr.on('data', (data) => { output += data.toString(); });
            proc.on('close', (code) => {
                if (code === 0) {
                    const versionMatch = output.match(/version "([^"]+)"/) || output.match(/version '([^']+)'/) || output.match(/version (.+?) /);
                    resolve({ success: true, version: versionMatch ? versionMatch[1] : 'unknown' });
                }
                else
                    resolve({ success: false, error: javaHome ? `Java 路径不可用: ${javaHome}` : 'Java 未安装或不在 PATH 中' });
            });
            proc.on('error', (err) => resolve({ success: false, error: `无法执行 Java 命令: ${err.message}` }));
        });
    });
    electron_1.ipcMain.handle('cicd:check-node', async (_event, nodeHome) => {
        const { spawn } = require('child_process');
        const nodeBin = nodeHome ? path.join(nodeHome, 'bin', 'node') : 'node';
        return new Promise((resolve) => {
            const proc = spawn(nodeBin, ['--version'], { timeout: 10000 });
            let output = '';
            proc.stdout.on('data', (data) => { output += data.toString(); });
            proc.stderr.on('data', (data) => { output += data.toString(); });
            proc.on('close', (code) => {
                if (code === 0) {
                    const versionMatch = output.match(/v?(\d+\.\d+\.\d+)/);
                    resolve({ success: true, version: versionMatch ? versionMatch[1] : 'unknown' });
                }
                else
                    resolve({ success: false, error: nodeHome ? `Node.js 路径不可用: ${nodeHome}` : 'Node.js 未安装或不在 PATH 中' });
            });
            proc.on('error', (err) => resolve({ success: false, error: `无法执行 Node 命令: ${err.message}` }));
        });
    });
} // end registerCicdHandlers
//# sourceMappingURL=cicd-handlers.js.map