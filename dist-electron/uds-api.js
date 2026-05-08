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
exports.UDS_SOCKET_PATH = void 0;
exports.startUdsApi = startUdsApi;
exports.stopUdsApi = stopUdsApi;
const logger_1 = require("./logger");
/**
 * Unix Domain Socket API for SuperTool CLI (stool)
 *
 * Pure JSON-over-UDS protocol using Node.js net module.
 * All endpoints call existing IPC handlers — zero duplicated logic.
 * GUI and CLI share the same code path.
 *
 * Security: only accessible via local Unix socket, not reachable over network.
 *
 * Protocol:
 *   Request:  {"handler": "todos:get-all", "params": {"completed": false}}
 *   Response: {"success": true, "data": [...]}
 *   Error:    {"success": false, "error": "message"}
 */
const net = __importStar(require("net"));
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const async_exec_1 = require("./async-exec");
const os = __importStar(require("os"));
const encryption_manager_1 = require("./encryption-manager");
exports.UDS_SOCKET_PATH = path.join(os.homedir(), '.supertool', 'supertool.sock');
let udsServer = null;
/** Safely extract error message from unknown error */
function getErrorMessage(error) {
    if (error instanceof Error)
        return error.message;
    if (typeof error === 'string')
        return error;
    return String(error);
}
const handlerRegistry = {};
function registerHandler(name, paramNames, handler) {
    handlerRegistry[name] = { handler, paramNames };
}
// ============ Shared Helpers (used by handler registrations) ============
function _getServerService() {
    const { getServerService, setServerService } = require('./server-handlers');
    let svc = getServerService();
    if (!svc) {
        const ServerService = require('./services/server-service');
        svc = new ServerService();
        setServerService(svc);
    }
    return svc;
}
async function _ensureConnected(serverId, server) {
    const svc = _getServerService();
    if (!svc.isConnected(serverId)) {
        const plainPassword = server.password ? (0, encryption_manager_1.decryptPassword)(server.password) : undefined;
        await svc.connect({ ...server, password: plainPassword });
    }
}
function _buildSearchCommandFn(logType, logPath, keyword, lines) {
    const kw = keyword.replace(/'/g, "'\\''");
    switch (logType) {
        case 'file': return `grep -i -n '${kw}' ${logPath} 2>/dev/null | tail -n ${lines}`;
        case 'docker': {
            const containers = logPath.split('\n').filter(c => c.trim());
            return containers.map(c => `(echo '=== ${c.trim()} ===' && docker logs ${c.trim()} 2>&1 | grep -i -n '${kw}' | tail -n ${lines})`).join(' ; ');
        }
        case 'journalctl': {
            const units = logPath.split('\n').filter(u => u.trim()).map(u => `-u ${u.trim()}`);
            return units.length > 0 ? `journalctl ${units.join(' ')} --grep='${kw}' -n ${lines} --no-pager 2>/dev/null` : `journalctl --grep='${kw}' -n ${lines} --no-pager 2>/dev/null`;
        }
        case 'custom': return `${logPath} 2>/dev/null | grep -i -n '${kw}' | tail -n ${lines}`;
        default: return `grep -i -n '${kw}' ${logPath} 2>/dev/null | tail -n ${lines}`;
    }
}
function _buildTailCommandFn(logType, logPath, lines, follow = false) {
    const paths = logPath.split('\n').map(p => p.trim()).filter(p => p);
    const quote = (p) => {
        if (p.startsWith('~')) {
            const rest = p.slice(1).replace(/'/g, "'\\\\''");
            return rest ? `$HOME'${rest}'` : '$HOME';
        }
        return `'${p.replace(/'/g, "'\\\\''")}'`;
    };
    const quotedPaths = paths.map(quote);
    const followFlag = follow ? '-f' : '';
    switch (logType) {
        case 'file': return `tail -n ${lines} ${followFlag} ${quotedPaths.join(' ')}`.replace(/\s+/g, ' ').trim();
        case 'journalctl': return `journalctl ${paths.map(u => `-u ${quote(u)}`).join(' ')} ${followFlag} -n ${lines} --no-pager`;
        case 'docker': return paths.map(c => `(echo "=== ${quote(c)} ===" && docker logs --tail ${lines} ${follow ? '--follow' : ''} ${quote(c)} 2>&1)`).join(' & ');
        case 'custom': return logPath;
        default: return `tail -n ${lines} ${followFlag} ${quotedPaths.join(' ')}`.replace(/\s+/g, ' ').trim();
    }
}
async function _execOnServer(serverId, command) {
    const db = getDb();
    const server = db.getServerById(serverId);
    if (!server)
        return { success: false, error: 'Server not found' };
    try {
        await _ensureConnected(serverId, server);
        const svc = _getServerService();
        const result = await svc.execCommand(serverId, command);
        return { success: result.success, output: result.output, error: result.errorOutput || undefined };
    }
    catch (e) {
        return { success: false, error: e instanceof Error ? e.message : String(e) };
    }
}
// ============ Lazy-load Modules ============
function getDb() { return require('./database'); }
function getLanService() {
    try {
        return require('./main').getLanServiceInstance();
    }
    catch {
        return null;
    }
}
function getGitService() { return require('./services/git-service'); }
function getMainModule() {
    try {
        return require('./main');
    }
    catch {
        return null;
    }
}
// ============ Handler Registration ============
function registerAllHandlers() {
    const db = getDb();
    const gitService = getGitService();
    // ===== Todos =====
    registerHandler('todos:get-all', [], () => db.getAllTodos());
    registerHandler('todos:add', ['todo'], (todo) => db.addTodo(todo));
    registerHandler('todos:update', ['todo'], (todo) => db.updateTodo(todo));
    registerHandler('todos:delete', ['id'], (id) => db.deleteTodo(id));
    registerHandler('todos:delete-many', ['ids'], (ids) => db.deleteTodos(ids));
    registerHandler('todos:update-order', ['todos'], (todos) => db.updateTodoOrder(todos));
    registerHandler('todos:create-repeat-instance', ['todo'], (todo) => db.createRepeatInstance(todo));
    // ===== Tags =====
    registerHandler('tags:get-all', [], () => db.getAllTags());
    registerHandler('tags:add', ['name'], (name) => db.addTag(name));
    registerHandler('tags:delete', ['name'], (name) => db.deleteTag(name));
    // ===== Settings =====
    registerHandler('settings:get', ['key'], (key) => db.getSetting(key));
    registerHandler('settings:set', ['key', 'value'], (key, value) => db.setSetting(key, value));
    // ===== Subtasks =====
    registerHandler('subtasks:get-for-todo', ['todoId'], (todoId) => db.getSubtasksForTodo(todoId));
    registerHandler('subtasks:add', ['subtask'], (subtask) => db.addSubtask(subtask));
    registerHandler('subtasks:update', ['subtask'], (subtask) => db.updateSubtask(subtask));
    registerHandler('subtasks:delete', ['subtaskId'], (subtaskId) => db.deleteSubtask(subtaskId));
    registerHandler('subtasks:update-todo-completion', ['todoId'], (todoId) => db.updateTodoCompletionBasedOnSubtasks(todoId));
    // ===== Projects =====
    registerHandler('projects:get-all', ['onlyActive'], (onlyActive = true) => db.getAllProjects(onlyActive));
    registerHandler('projects:add', ['project'], (project) => db.addProject(project));
    registerHandler('projects:update', ['project'], (project) => db.updateProject(project));
    registerHandler('projects:delete', ['id'], (id) => db.deleteProject(id));
    registerHandler('projects:get-stats', ['projectId'], (projectId) => db.getProjectStats(projectId));
    registerHandler('projects:get-todos', ['projectId'], (projectId) => db.getTodosByProject(projectId));
    // ===== Notification =====
    registerHandler('notification:get-settings', [], () => ({ reminderTime: parseInt(db.getSetting('reminder_time') || '15') }));
    registerHandler('notification:set-settings', ['settings'], (settings) => { db.setSetting('reminder_time', String(settings.reminderTime)); return settings; });
    registerHandler('notification:test', [], () => ({ success: true }));
    registerHandler('notifications:dismiss', ['todoId'], (todoId) => ({ success: true }));
    // ===== MFA/OTP =====
    registerHandler('mfa:get-secrets', [], () => db.getAllMfaSecrets());
    registerHandler('mfa:add-secret', ['data'], (data) => db.addMfaSecret(data));
    registerHandler('mfa:update-secret', ['id', 'updates'], (id, updates) => db.updateMfaSecret(id, updates));
    registerHandler('mfa:delete-secret', ['id'], (id) => db.deleteMfaSecret(id));
    registerHandler('mfa:generate-code', ['secret', 'digits', 'period', 'algorithm'], (secret, digits = 6, period = 30, algorithm = 'SHA1') => {
        const totp = require('./services/totp');
        return { code: totp.generateTOTP(secret, digits, period, algorithm), remaining: totp.getRemainingTime(period) };
    });
    registerHandler('mfa:parse-uri', ['uri'], (uri) => {
        const totp = require('./services/totp');
        return totp.parseOtpAuthUri(uri);
    });
    // ===== Notes =====
    registerHandler('notes:get-all', ['query', 'groupId'], (query, groupId) => db.getAllNotes(query, groupId));
    registerHandler('notes:get-by-id', ['id'], (id) => db.getNoteById(id));
    registerHandler('notes:add', ['data'], (data) => db.addNote(data));
    registerHandler('notes:update', ['id', 'updates'], (id, updates) => db.updateNote(id, updates));
    registerHandler('notes:delete', ['id'], (id) => db.deleteNote(id));
    registerHandler('note-groups:get-all', [], () => db.getAllNoteGroups());
    registerHandler('note-groups:add', ['data'], (data) => db.addNoteGroup(data));
    registerHandler('note-groups:update', ['id', 'updates'], (id, updates) => db.updateNoteGroup(id, updates));
    registerHandler('note-groups:delete', ['id'], (id) => db.deleteNoteGroup(id));
    // ===== Weekly Reports =====
    registerHandler('weekly-report:get-all', ['limit'], (limit = 20) => db.getWeeklyReports(limit));
    registerHandler('weekly-report:get', ['id'], (id) => db.getWeeklyReport(id));
    registerHandler('weekly-report:save', ['report'], (report) => db.saveWeeklyReport(report));
    // ===== Servers =====
    registerHandler('servers:get-all', [], () => db.getAllServers().map((s) => {
        const { password, ...rest } = s;
        return rest;
    }));
    registerHandler('servers:get-by-id', ['serverId'], (serverId) => {
        const server = db.getServerById(serverId);
        if (!server)
            return null;
        const { password, ...rest } = server;
        return rest;
    });
    registerHandler('servers:add', ['server'], (server) => db.addServer(server));
    registerHandler('servers:update', ['server'], (server) => db.updateServer(server));
    registerHandler('servers:delete', ['serverId'], (serverId) => db.deleteServer(serverId));
    registerHandler('servers:test-connection', ['server'], async (server) => {
        try {
            const ServerService = require('./services/server-service');
            const svc = new ServerService();
            const plainPassword = server.password ? (0, encryption_manager_1.decryptPassword)(server.password) : undefined;
            await svc.testConnection({ ...server, password: plainPassword });
            return { success: true };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    registerHandler('servers:groups:get-all', [], () => db.getAllServerGroups());
    registerHandler('servers:groups:add', ['group'], (group) => db.addServerGroup(group));
    registerHandler('servers:groups:update', ['groupId', 'updates'], (groupId, updates) => db.updateServerGroup(groupId, updates));
    registerHandler('servers:groups:delete', ['groupId'], (groupId) => db.deleteServerGroup(groupId));
    // ===== SFTP =====
    registerHandler('servers:sftp:list', ['serverId', 'remotePath'], async (serverId, remotePath) => {
        const server = db.getServerById(serverId);
        if (!server)
            return { error: 'Server not found' };
        try {
            await _ensureConnected(serverId, server);
            const svc = _getServerService();
            const files = await svc.listRemoteDir(serverId, remotePath);
            return files;
        }
        catch (error) {
            return { error: error.message };
        }
    });
    registerHandler('servers:sftp:read-file', ['serverId', 'remotePath'], async (serverId, remotePath) => {
        const server = db.getServerById(serverId);
        if (!server)
            return { success: false, error: 'Server not found' };
        try {
            await _ensureConnected(serverId, server);
            const svc = _getServerService();
            await svc.createSftp(serverId);
            const content = await svc.downloadFileBase64 ? await svc.downloadFileBase64(serverId, remotePath) :
                (await svc.execCommand(serverId, `base64 "${remotePath}"`));
            return content;
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    registerHandler('servers:sftp:mkdir', ['serverId', 'remotePath'], async (serverId, remotePath) => {
        const server = db.getServerById(serverId);
        if (!server)
            return { success: false, error: 'Server not found' };
        try {
            await _ensureConnected(serverId, server);
            const svc = _getServerService();
            return await svc.createRemoteDir(serverId, remotePath);
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    registerHandler('servers:sftp:delete', ['serverId', 'remotePath'], async (serverId, remotePath) => {
        const server = db.getServerById(serverId);
        if (!server)
            return { success: false, error: 'Server not found' };
        try {
            await _ensureConnected(serverId, server);
            const svc = _getServerService();
            return await svc.deleteRemoteFile(serverId, remotePath);
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    // ===== CI/CD =====
    registerHandler('cicd:get-all-configs', [], () => db.getAllCicdConfigs());
    registerHandler('cicd:get-config', ['projectId'], (projectId) => db.getCicdConfig(projectId));
    registerHandler('cicd:get-config-by-id', ['configId'], (configId) => db.getCicdConfigByConfigId(configId));
    registerHandler('cicd:get-groups', [], () => db.getCicdGroups());
    registerHandler('cicd:add-config', ['config'], (config) => db.addCicdConfig(config));
    registerHandler('cicd:update-config', ['config'], (config) => db.updateCicdConfig(config));
    registerHandler('cicd:delete-config', ['configId'], (configId) => db.deleteCicdConfig(configId));
    registerHandler('cicd:get-modules', ['configId'], (configId) => db.getDeployModules(configId));
    registerHandler('cicd:add-module', ['module'], (module) => db.addDeployModule(module));
    registerHandler('cicd:update-module', ['module'], (module) => db.updateDeployModule(module));
    registerHandler('cicd:delete-module', ['moduleId'], (moduleId) => db.deleteDeployModule(moduleId));
    registerHandler('cicd:get-logs', ['projectId', 'limit'], (projectId, limit = 20) => db.getDeployLogs(projectId, limit));
    registerHandler('cicd:get-step-logs', ['deployLogId'], (deployLogId) => db.getDeployStepLogs(deployLogId));
    registerHandler('cicd:get-deploy-history', ['configId', 'limit', 'status'], (configId, limit = 20, status) => {
        const stmt = status
            ? db.getDatabase().prepare('SELECT * FROM deploy_history WHERE configId = ? AND status = ? ORDER BY deployedAt DESC LIMIT ?')
            : db.getDatabase().prepare('SELECT * FROM deploy_history WHERE configId = ? ORDER BY deployedAt DESC LIMIT ?');
        return status ? stmt.all(configId, status, limit) : stmt.all(configId, limit);
    });
    registerHandler('cicd:detect-tool-paths', [], async () => {
        const paths = {};
        const tools = ['node', 'npm', 'yarn', 'python3', 'java', 'mvn', 'git', 'docker'];
        for (const tool of tools) {
            paths[tool] = (await (0, async_exec_1.tryCommand)(`which ${tool}`))?.stdout.trim() || '';
        }
        return paths;
    });
    registerHandler('cicd:detect-sdk-versions', [], async () => {
        const versions = {};
        const cmds = [
            ['node', 'node --version'], ['npm', 'npm --version'], ['python3', 'python3 --version'],
            ['java', 'java -version'], ['mvn', 'mvn -version'], ['git', 'git --version'],
        ];
        for (const [name, cmd] of cmds) {
            const result = await (0, async_exec_1.tryCommand)(cmd);
            versions[name] = result?.stdout.trim() || '';
        }
        return versions;
    });
    registerHandler('cicd:test-ssh', ['config'], async (config) => {
        try {
            const ServerService = require('./services/server-service');
            const svc = new ServerService();
            await svc.testConnection(config);
            return { success: true, message: 'SSH connection successful' };
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    // CICD deploy/rollback/cancel — delegate to cicd-handlers
    registerHandler('cicd:deploy', ['configId', 'confirmed'], async (configId, confirmed = false) => {
        const config = getDb().getCicdConfigByConfigId(configId);
        if (config && config.requiresApproval && !confirmed) {
            return { success: false, requiresApproval: true, message: '此配置需要审核确认，请确认后再次部署', configName: config.name || '' };
        }
        const { cicdDeploy } = require('./cicd-handlers');
        return cicdDeploy(getDb(), configId);
    });
    registerHandler('cicd:rollback', ['configId', 'deployHistoryId'], async (configId, deployHistoryId = '') => {
        const config = getDb().getCicdConfigByConfigId(configId);
        if (config && config.requiresApproval) {
            return { success: false, requiresApproval: true, message: '此配置已开启部署审核，CLI 不支持回滚，请在 GUI 中操作', configName: config.name || '' };
        }
        const { cicdRollback } = require('./cicd-handlers');
        return cicdRollback(getDb(), configId, deployHistoryId);
    });
    registerHandler('cicd:cancel', ['deployLogId'], async (deployLogId) => {
        const { cicdCancelDeploy } = require('./cicd-handlers');
        return cicdCancelDeploy(deployLogId);
    });
    // ===== Git =====
    registerHandler('git:repos:get-all', [], () => db.getAllGitRepos());
    registerHandler('git:repos:add', ['repo'], (repo) => db.addGitRepo(repo));
    registerHandler('git:repos:update', ['id', 'updates'], (id, updates) => db.updateGitRepo(id, updates));
    registerHandler('git:repos:delete', ['id'], (id) => db.deleteGitRepo(id));
    registerHandler('git:status', ['repoPath'], async (repoPath) => gitService.getRepoStatus(repoPath));
    registerHandler('git:log', ['repoPath', 'options'], async (repoPath, options) => gitService.getRepoLog(repoPath, options));
    registerHandler('git:branches', ['repoPath'], async (repoPath) => gitService.getRepoBranches(repoPath));
    registerHandler('git:current-branch', ['repoPath'], async (repoPath) => gitService.getCurrentBranch(repoPath));
    registerHandler('git:diff', ['repoPath', 'file'], async (repoPath, file) => gitService.getRepoDiff(repoPath, file));
    registerHandler('git:commit', ['repoPath', 'message', 'files'], async (repoPath, message, files) => gitService.commit(repoPath, message, files));
    registerHandler('git:add', ['repoPath', 'files'], async (repoPath, files) => gitService.add(repoPath, files));
    registerHandler('git:pull', ['repoPath'], async (repoPath) => gitService.pull(repoPath));
    registerHandler('git:push', ['repoPath'], async (repoPath) => gitService.push(repoPath));
    registerHandler('git:fetch', ['repoPath', 'remote'], async (repoPath, remote) => {
        try {
            await (0, async_exec_1.runCommand)(`git fetch ${remote || 'origin'}`, { cwd: repoPath, encoding: 'utf-8' });
            return { success: true };
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    registerHandler('git:checkout', ['repoPath', 'branch'], async (repoPath, branch) => gitService.checkout(repoPath, branch));
    registerHandler('git:create-branch', ['repoPath', 'branchName', 'from'], async (repoPath, branchName, from) => gitService.createBranch(repoPath, branchName, from));
    registerHandler('git:delete-branch', ['repoPath', 'branchName', 'force'], async (repoPath, branchName, force = false) => gitService.deleteBranch(repoPath, branchName, force));
    registerHandler('git:merge', ['repoPath', 'branch'], async (repoPath, branch) => gitService.merge(repoPath, branch));
    registerHandler('git:reset', ['repoPath', 'file'], async (repoPath, file) => gitService.reset(repoPath, file));
    registerHandler('git:stash-save', ['repoPath', 'options'], async (repoPath, options) => gitService.gitStashSave(repoPath, options));
    registerHandler('git:stash-list', ['repoPath'], async (repoPath) => gitService.gitStashList(repoPath));
    registerHandler('git:stash-pop', ['repoPath', 'stashRef'], async (repoPath, stashRef) => gitService.gitStashPop(repoPath, stashRef));
    registerHandler('git:stash-apply', ['repoPath', 'stashRef'], async (repoPath, stashRef) => gitService.gitStashApply(repoPath, stashRef));
    registerHandler('git:stash-drop', ['repoPath', 'stashRef'], async (repoPath, stashRef) => gitService.gitStashDrop(repoPath, stashRef));
    registerHandler('git:tag-list', ['repoPath'], async (repoPath) => gitService.gitListTags(repoPath));
    registerHandler('git:tag-create', ['repoPath', 'tagName', 'options'], async (repoPath, tagName, options) => gitService.gitCreateTag(repoPath, tagName, options));
    registerHandler('git:tag-delete', ['repoPath', 'tagName'], async (repoPath, tagName) => gitService.gitDeleteTag(repoPath, tagName));
    registerHandler('git:rebase', ['repoPath', 'branch'], async (repoPath, branch) => {
        try {
            await (0, async_exec_1.runCommand)(`git rebase ${branch}`, { cwd: repoPath });
            return { success: true };
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    registerHandler('git:remotes', ['repoPath'], async (repoPath) => {
        try {
            const result = await (0, async_exec_1.runCommand)('git remote -v', { cwd: repoPath });
            return { success: true, remotes: result.stdout.trim() };
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    registerHandler('git:exec', ['repoPath', 'args'], async (repoPath, args) => {
        try {
            const result = await (0, async_exec_1.runCommand)(`git ${args.join(' ')}`, { cwd: repoPath });
            return { success: true, output: result.stdout.trim() };
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    // ===== Git Sync =====
    registerHandler('git-sync:configure', ['config'], async (config) => {
        for (const [k, v] of Object.entries(config))
            db.setSetting(`git_sync_${k}`, String(v));
        return { success: true };
    });
    registerHandler('git-sync:status', [], async () => {
        const enabled = await db.getSetting('git_sync_enabled');
        return { enabled: enabled === 'true' };
    });
    // ===== OpenVPN =====
    registerHandler('openvpn:get-all', [], () => db.getOpenVPNConfigs());
    registerHandler('openvpn:add', ['data'], (data) => db.addOpenVPNConfig(data.name, data.filePath, data.content));
    registerHandler('openvpn:delete', ['id'], (id) => db.deleteOpenVPNConfig(id));
    registerHandler('openvpn:validate-config', ['content'], (content) => {
        const isValid = content.includes('dev tun') || content.includes('dev tap');
        return { valid: isValid, errors: isValid ? [] : ['Missing dev directive'] };
    });
    // ===== Accounting =====
    registerHandler('accounting:categories:get', [], () => db.getAccountingCategories());
    registerHandler('accounting:categories:add', ['category'], (category) => db.addAccountingCategory(category));
    registerHandler('accounting:categories:update', ['id', 'updates'], (id, updates) => db.updateAccountingCategory(id, updates));
    registerHandler('accounting:categories:delete', ['id'], (id) => db.deleteAccountingCategory(id));
    registerHandler('accounting:records:get', ['options'], (options) => db.getAccountingRecords(options));
    registerHandler('accounting:records:add', ['record'], (record) => db.addAccountingRecord(record));
    registerHandler('accounting:records:update', ['id', 'updates'], (id, updates) => db.updateAccountingRecord(id, updates));
    registerHandler('accounting:records:delete', ['id'], (id) => db.deleteAccountingRecord(id));
    registerHandler('accounting:stats:get', ['options'], (options) => db.getAccountingStats(options?.startDate, options?.endDate));
    registerHandler('accounting:budgets:get', [], () => db.getBudgets());
    registerHandler('accounting:budgets:add', ['data'], (data) => db.addBudget(data));
    registerHandler('accounting:budgets:update', ['id', 'updates'], (id, updates) => db.updateBudget(id, updates));
    registerHandler('accounting:budgets:delete', ['id'], (id) => db.deleteBudget(id));
    registerHandler('accounting:budgets:alerts', [], () => db.checkBudgetAlerts());
    registerHandler('accounting:templates:get', [], () => db.getTemplates());
    registerHandler('accounting:templates:add', ['data'], (data) => db.addTemplate(data));
    registerHandler('accounting:templates:update', ['id', 'updates'], (id, updates) => db.updateTemplate(id, updates));
    registerHandler('accounting:templates:delete', ['id'], (id) => db.deleteTemplate(id));
    registerHandler('accounting:templates:use', ['id'], (id) => db.useTemplate(id));
    registerHandler('accounting:trend:get', ['months'], (months) => db.getAccountingTrend(months));
    registerHandler('accounting:export:csv', ['options'], (options) => db.exportAccountingRecordsCSV(options));
    // ===== Backup =====
    registerHandler('backup:export-data', [], () => db.exportAllData());
    registerHandler('backup:import-json', ['options'], async (options) => {
        if (!options?.data)
            return { success: false, error: 'No data provided' };
        db.importAllData(options.data, options.mode || 'merge');
        return { success: true, message: 'Data imported successfully' };
    });
    // ===== LAN =====
    registerHandler('lan:get-peers', [], () => {
        const lan = getLanService();
        return lan ? lan.getOnlinePeers() : [];
    });
    registerHandler('lan:get-user-info', [], () => {
        const lan = getLanService();
        return lan ? { id: lan.userId, name: lan.userName, avatar: lan.avatar } : { userId: 'cli', userName: 'CLI' };
    });
    registerHandler('lan:set-nickname', ['name'], (name) => {
        const lan = getLanService();
        if (lan)
            lan.setNickName(name);
        return { success: true };
    });
    registerHandler('lan:set-avatar', ['emoji'], (emoji) => {
        const lan = getLanService();
        if (lan)
            lan.setAvatar(emoji);
        return { success: true };
    });
    registerHandler('lan:set-status', ['status'], (status) => {
        const lan = getLanService();
        if (lan)
            lan.setStatus(status);
        return { success: true };
    });
    registerHandler('lan:get-status', [], () => {
        const lan = getLanService();
        return lan ? { status: lan.status } : { status: 'available' };
    });
    registerHandler('lan:refresh-discovery', [], () => {
        const lan = getLanService();
        if (lan)
            lan.refreshDiscovery();
        return { success: true };
    });
    registerHandler('lan:send-message', ['peerId', 'message'], (peerId, message) => {
        const lan = getLanService();
        if (!lan)
            return { success: false, error: 'LAN service not available' };
        return lan.sendMessage(peerId, message);
    });
    registerHandler('lan:send-file', ['peerId', 'filePath', 'fileName', 'resumeOffset', 'fileId'], (peerId, filePath, fileName, resumeOffset = 0, fileId) => {
        const lan = getLanService();
        if (!lan)
            return { success: false, error: 'LAN service not available' };
        return { success: lan.sendFile(peerId, filePath, fileName, resumeOffset, fileId) };
    });
    registerHandler('lan:get-message-history', ['limit', 'offset'], (limit = 100, offset = 0) => {
        return db.getChatMessages(limit, offset);
    });
    registerHandler('lan:get-messages-between', ['userId1', 'userId2', 'limit', 'offset'], (userId1, userId2, limit = 200, offset = 0) => db.getChatMessagesBetween(userId1, userId2, limit, offset));
    registerHandler('lan:mark-messages-read', ['myUserId', 'peerId'], (myUserId, peerId) => db.markMessagesAsRead(myUserId, peerId));
    registerHandler('lan:get-unread-count', ['myUserId', 'peerId'], (myUserId, peerId) => db.getUnreadCount(myUserId, peerId));
    registerHandler('lan:get-all-unread-counts', ['myUserId'], (myUserId) => db.getAllUnreadCounts(myUserId));
    registerHandler('lan:get-file-transfer-history', ['limit', 'offset'], (limit = 100, offset = 0) => {
        const lan = getLanService();
        return lan ? lan.getFileTransferHistory(limit, offset) : [];
    });
    registerHandler('lan:assign-task', ['peerId', 'task'], async (peerId, task) => {
        const lan = getLanService();
        if (!lan)
            return { success: false };
        const todoId = `lan-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
        const now = new Date().toISOString();
        const myInfo = { id: lan.userId, name: lan.userName };
        try {
            db.addTodo({
                id: todoId, text: task.text, completed: false, priority: task.priority || 'medium',
                dueDate: task.dueDate || null, description: task.note || '', markdownDescription: null,
                tag: '', createdAt: now, updatedAt: now, assignedTo: peerId, assignedBy: myInfo.id,
                assignedAt: now, owner: myInfo.id, orderNum: 0, repeatType: null, repeatInterval: 0,
                repeatCount: 0, repeatEndDate: null, parentTodoId: null, projectId: null
            });
            const sent = lan.assignTask(peerId, { ...task, id: todoId });
            return { success: sent, todoId };
        }
        catch {
            return { success: false };
        }
    });
    // ===== Log Presets =====
    registerHandler('log-presets:get-all', [], () => db.getLogPresets());
    registerHandler('log-presets:add', ['data'], (data) => db.addLogPreset(data.name, data.serverIds, data.logPath, data.logType, data.keywords, data.maxLines, data.presetGroup));
    registerHandler('log-presets:update', ['id', 'updates'], (id, updates) => db.updateLogPreset(id, updates));
    registerHandler('log-presets:delete', ['id'], (id) => db.deleteLogPreset(id));
    // ===== Log Aggregator (non-streaming, uses shared helpers) =====
    registerHandler('log:tail', ['presetId', 'lines'], async (presetId, lines = 100) => {
        const preset = db.getLogPresets().find((p) => p.id === presetId);
        if (!preset)
            return { success: false, error: '未找到日志预设' };
        const serverIds = (() => {
            const v = preset.serverIds;
            if (Array.isArray(v))
                return v;
            if (typeof v === 'string') {
                try {
                    return JSON.parse(v);
                }
                catch {
                    return [];
                }
            }
            return [];
        })();
        if (serverIds.length === 0)
            return { success: false, error: '预设没有关联服务器' };
        const cmd = _buildTailCommandFn(preset.logType || 'file', preset.logPath, lines);
        const results = [];
        await Promise.allSettled(serverIds.map(async (sid) => {
            const server = db.getServerById(sid);
            const sname = server?.name || sid;
            const r = await _execOnServer(sid, cmd);
            results.push({ serverId: sid, serverName: sname, ...r });
        }));
        return { success: true, presetName: preset.name, results };
    });
    registerHandler('log:search', ['presetId', 'keyword', 'lines'], async (presetId, keyword, lines = 50) => {
        if (!keyword || !keyword.trim())
            return { success: false, error: '关键字不能为空' };
        const preset = db.getLogPresets().find((p) => p.id === presetId);
        if (!preset)
            return { success: false, error: '未找到日志预设' };
        const serverIds = (() => {
            const v = preset.serverIds;
            if (Array.isArray(v))
                return v;
            if (typeof v === 'string') {
                try {
                    return JSON.parse(v);
                }
                catch {
                    return [];
                }
            }
            return [];
        })();
        if (serverIds.length === 0)
            return { success: false, error: '预设没有关联服务器' };
        const cmd = _buildSearchCommandFn(preset.logType || 'file', preset.logPath, keyword, lines);
        const results = [];
        await Promise.allSettled(serverIds.map(async (sid) => {
            const server = db.getServerById(sid);
            const sname = server?.name || sid;
            const r = await _execOnServer(sid, cmd);
            results.push({ serverId: sid, serverName: sname, ...r });
        }));
        return { success: true, presetName: preset.name, results };
    });
    // ===== Log streaming handler (stream=true) =====
    registerHandler('log:stream', ['presetId', 'lines', 'follow'], async (presetId, lines = 100, follow = true) => {
        // Non-streaming fallback — same as log:tail for now
        const preset = db.getLogPresets().find((p) => p.id === presetId);
        if (!preset)
            return { success: false, error: '未找到日志预设' };
        const serverIds = (() => {
            const v = preset.serverIds;
            if (Array.isArray(v))
                return v;
            if (typeof v === 'string') {
                try {
                    return JSON.parse(v);
                }
                catch {
                    return [];
                }
            }
            return [];
        })();
        if (serverIds.length === 0)
            return { success: false, error: '预设没有关联服务器' };
        const cmd = _buildTailCommandFn(preset.logType || 'file', preset.logPath, lines, follow);
        const results = [];
        await Promise.allSettled(serverIds.map(async (sid) => {
            const server = db.getServerById(sid);
            const sname = server?.name || sid;
            const r = await _execOnServer(sid, cmd);
            results.push({ serverId: sid, serverName: sname, ...r });
        }));
        return { success: true, presetName: preset.name, results };
    });
    // ===== CICD deploy-stream handler (stream=true) =====
    registerHandler('cicd:deploy-stream', ['configId', 'confirmed'], async (configId, confirmed = false) => {
        const config = getDb().getCicdConfigByConfigId(configId);
        if (config && config.requiresApproval && !confirmed) {
            return { success: false, requiresApproval: true, message: '此配置需要审核确认，请添加 confirmed=true 参数', configName: config.name || '' };
        }
        const { cicdDeploy } = require('./cicd-handlers');
        return cicdDeploy(getDb(), configId);
    });
    // ===== Misc =====
    registerHandler('get-app-path', [], () => os.homedir() + '/.supertool');
    registerHandler('shell-env:get', [], () => ({ loaded: true, vars: {} }));
    registerHandler('file:read-content', ['filePath'], (filePath) => {
        try {
            return { success: true, content: fs.readFileSync(filePath, 'utf-8') };
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    registerHandler('fs:readdir', ['dirPath'], (dirPath) => {
        try {
            return { success: true, entries: fs.readdirSync(dirPath, { withFileTypes: true }).map((e) => ({ name: e.name, isDirectory: e.isDirectory() })) };
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    registerHandler('modules:scan', [], async () => {
        const { scanProjectModules } = require('./modules-scanner');
        return scanProjectModules();
    });
    // ===== API Debugger =====
    registerHandler('api:http-request', ['config'], async (config) => {
        try {
            const axios = require('axios');
            const resp = await axios(config);
            return { success: true, status: resp.status, data: resp.data };
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    registerHandler('api:requests:get-all', [], () => []);
    registerHandler('api:requests:add', ['request'], (request) => ({ success: true, id: Date.now().toString() }));
    registerHandler('api:requests:update', ['id', 'updates'], (id, updates) => ({ success: true }));
    registerHandler('api:requests:delete', ['id'], (id) => ({ success: true }));
    // ===== Server Exec =====
    registerHandler('servers:exec', ['serverId', 'command'], async (serverId, command) => {
        const server = db.getServerById(serverId);
        if (!server)
            return { success: false, error: 'Server not found' };
        if (server.requiresApproval)
            return { success: false, requiresApproval: true, serverName: server.name, error: '该服务器已开启执行审核，请在 GUI 中手动确认' };
        try {
            await _ensureConnected(serverId, server);
            const svc = _getServerService();
            const result = await svc.execCommand(serverId, command);
            return { success: result.success, output: result.output };
        }
        catch (e) {
            return { success: false, error: e instanceof Error ? e.message : String(e) };
        }
    });
    // ===== Server Batch Exec =====
    registerHandler('servers:batch-exec', ['command', 'serverIds', 'tag'], async (command, serverIds, tag) => {
        if (!command)
            return { success: false, error: 'Missing command' };
        let targetIds = serverIds || [];
        if (tag && !serverIds) {
            const allServers = db.getAllServers();
            targetIds = allServers.filter((s) => s.tags && s.tags.includes(tag)).map((s) => s.id);
        }
        if (!serverIds && !tag) {
            targetIds = db.getAllServers().map((s) => s.id);
        }
        const results = await Promise.allSettled(targetIds.map(async (sid) => {
            const server = db.getServerById(sid);
            if (!server)
                return { serverId: sid, success: false, error: 'Server not found' };
            if (server.requiresApproval)
                return { serverId: sid, name: server.name, success: false, requiresApproval: true, error: '服务器已开启审核' };
            try {
                await _ensureConnected(sid, server);
                const svc = _getServerService();
                const result = await svc.execCommand(sid, command);
                return { serverId: sid, name: server.name, success: result.success, output: result.output };
            }
            catch (e) {
                return { serverId: sid, name: server?.name || sid, success: false, error: e instanceof Error ? e.message : String(e) };
            }
        }));
        return { results: results.map((r) => r.status === 'fulfilled' ? r.value : { success: false, error: String(r.reason) }) };
    });
    // ===== External DB Connections (moved from REST endpoints) =====
    function getDBConnections() {
        try {
            const raw = db.getSetting('db_connections');
            const conns = raw ? JSON.parse(raw) : [];
            // 内部使用：解密密码以便连接数据库
            return conns.map((c) => ({
                ...c,
                password: c.password ? (0, encryption_manager_1.decryptPassword)(c.password) : c.password,
            }));
        }
        catch {
            return [];
        }
    }
    /** CLI 列表响应：过滤密码字段，规范化 requiresApproval */
    function listDBConnections() {
        return getDBConnections().map((c) => {
            const { password, ...rest } = c;
            return { ...rest, requiresApproval: !!c.requiresApproval };
        });
    }
    registerHandler('db:list', [], () => {
        return { success: true, connections: listDBConnections() };
    });
    registerHandler('db:disconnect', ['id'], async (id) => {
        try {
            const dbManager = require('./services/db-manager').default;
            await dbManager.disconnect(id);
            return { success: true };
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    registerHandler('db:get-databases', ['dbId'], async (dbId) => {
        try {
            const dbManager = require('./services/db-manager').default;
            if (!dbManager.isConnected(dbId)) {
                const conns = getDBConnections();
                const config = conns.find((c) => c.id === dbId);
                if (config)
                    await dbManager.connect(dbId, config);
            }
            const databases = await dbManager.getDatabases(dbId);
            return { success: true, databases };
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    registerHandler('db:get-tables', ['dbId', 'db'], async (dbId, dbName = '') => {
        try {
            const dbManager = require('./services/db-manager').default;
            if (!dbManager.isConnected(dbId)) {
                const conns = getDBConnections();
                const config = conns.find((c) => c.id === dbId);
                if (config)
                    await dbManager.connect(dbId, config);
            }
            const tables = await dbManager.getTables(dbId, dbName);
            return { success: true, tables };
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    registerHandler('db:query', ['dbId', 'sql'], async (dbId, sql) => {
        try {
            const conns = getDBConnections();
            const conn = conns.find((c) => c.id === dbId);
            if (conn && conn.requiresApproval) {
                return { success: false, requiresApproval: true, message: `数据库「${conn.name}」已开启安全审核，CLI 不支持执行 SQL。请在 GUI 中操作。` };
            }
            const dbManager = require('./services/db-manager').default;
            if (!dbManager.isConnected(dbId)) {
                const config = conns.find((c) => c.id === dbId);
                if (config)
                    await dbManager.connect(dbId, config);
            }
            const rows = await dbManager.query(dbId, sql);
            return { success: true, rows };
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    // ===== Redis Operations (moved from REST endpoints) =====
    registerHandler('db:redis:keys', ['dbId', 'pattern'], async (dbId, pattern = '*') => {
        try {
            const dbManager = require('./services/db-manager').default;
            if (!dbManager.isConnected(dbId)) {
                const conns = getDBConnections();
                const config = conns.find((c) => c.id === dbId);
                if (config)
                    await dbManager.connect(dbId, config);
            }
            const result = await dbManager.getRedisKeys(dbId, pattern);
            return { success: true, ...result };
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    registerHandler('db:redis:get', ['dbId', 'key'], async (dbId, key) => {
        try {
            const dbManager = require('./services/db-manager').default;
            if (!dbManager.isConnected(dbId)) {
                const conns = getDBConnections();
                const config = conns.find((c) => c.id === dbId);
                if (config)
                    await dbManager.connect(dbId, config);
            }
            const value = await dbManager.getValue(dbId, key);
            return { success: true, value };
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    registerHandler('db:redis:set', ['dbId', 'key', 'value'], async (dbId, key, value) => {
        try {
            const dbManager = require('./services/db-manager').default;
            if (!dbManager.isConnected(dbId)) {
                const conns = getDBConnections();
                const config = conns.find((c) => c.id === dbId);
                if (config)
                    await dbManager.connect(dbId, config);
            }
            await dbManager.setRedisValue(dbId, key, value);
            return { success: true };
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    registerHandler('db:redis:delete', ['dbId', 'key'], async (dbId, key) => {
        try {
            const dbManager = require('./services/db-manager').default;
            if (!dbManager.isConnected(dbId)) {
                const conns = getDBConnections();
                const config = conns.find((c) => c.id === dbId);
                if (config)
                    await dbManager.connect(dbId, config);
            }
            await dbManager.deleteRedisKey(dbId, key);
            return { success: true };
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    registerHandler('db:redis:command', ['dbId', 'command'], async (dbId, command) => {
        try {
            const dbManager = require('./services/db-manager').default;
            if (!dbManager.isConnected(dbId)) {
                const conns = getDBConnections();
                const config = conns.find((c) => c.id === dbId);
                if (config)
                    await dbManager.connect(dbId, config);
            }
            const result = await dbManager.execRedisCommand(dbId, command);
            return { success: true, result };
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    // ===== Update =====
    registerHandler('update:get-version', [], () => ({ version: require('../package.json').version }));
    (0, logger_1.info)(`[API] Registered ${Object.keys(handlerRegistry).length} handlers`);
}
// ============ JSON Request Handler ============
async function handleJsonRequest(socket, req) {
    const entry = handlerRegistry[req.handler];
    if (!entry) {
        socket.write(JSON.stringify({ success: false, error: `Handler '${req.handler}' not found` }) + '\n');
        return;
    }
    // 流式请求：使用 streamCommand 实时转发输出
    if (req.stream && req.handler === 'log:tail') {
        await handleLogStream(socket, req.params);
        return;
    }
    if (req.stream && req.handler === 'cicd:deploy-stream') {
        await handleCicdDeployStream(socket, req.params);
        return;
    }
    // 非流式：正常调用 handler 并返回结果
    const args = entry.paramNames.map(pn => req.params?.[pn]);
    try {
        const result = await entry.handler(...args);
        socket.write(JSON.stringify({ success: true, data: result }) + '\n');
    }
    catch (e) {
        socket.write(JSON.stringify({ success: false, error: e instanceof Error ? e.message : String(e) }) + '\n');
    }
}
// ============ Log Stream Handler ============
async function handleLogStream(socket, params) {
    const presetId = params?.presetId;
    const lines = params?.lines || 100;
    const follow = params?.follow ?? true;
    const preset = getDb().getLogPresets().find((p) => p.id === presetId);
    if (!preset) {
        socket.write(JSON.stringify({ success: false, error: '未找到日志预设' }) + '\n');
        return;
    }
    const serverIds = (() => {
        const v = preset.serverIds;
        if (Array.isArray(v))
            return v;
        if (typeof v === 'string') {
            try {
                return JSON.parse(v);
            }
            catch {
                return [];
            }
        }
        return [];
    })();
    if (serverIds.length === 0) {
        socket.write(JSON.stringify({ success: false, error: '预设没有关联服务器' }) + '\n');
        return;
    }
    const cmd = _buildTailCommandFn(preset.logType || 'file', preset.logPath, lines, follow);
    // 发送开始事件
    socket.write(JSON.stringify({ stream: true, event: 'start', presetName: preset.name }) + '\n');
    const stops = [];
    let completedCount = 0;
    await Promise.allSettled(serverIds.map(async (sid) => {
        const server = getDb().getServerById(sid);
        const sname = server?.name || sid;
        try {
            const svc = _getServerService();
            if (!svc.isConnected(sid)) {
                await _ensureConnected(sid, server);
            }
            const { stop } = await svc.streamCommand(sid, cmd, (line) => {
                socket.write(JSON.stringify({ stream: true, event: 'data', serverName: sname, line }) + '\n');
            }, () => {
                socket.write(JSON.stringify({ stream: true, event: 'complete', serverName: sname }) + '\n');
                completedCount++;
                if (completedCount >= serverIds.length && !follow) {
                    stops.forEach(s => { try {
                        s.stop();
                    }
                    catch { } });
                }
            }, (err) => {
                socket.write(JSON.stringify({ stream: true, event: 'error', serverName: sname, error: err }) + '\n');
            });
            stops.push({ stop });
        }
        catch (e) {
            const msg = e instanceof Error ? e.message : String(e);
            socket.write(JSON.stringify({ stream: true, event: 'error', serverName: sname, error: msg }) + '\n');
        }
    }));
    // CLI 断开连接时清理
    socket.once('close', () => {
        stops.forEach(s => { try {
            s.stop();
        }
        catch { } });
    });
}
// ============ CICD Deploy Stream Handler ============
async function handleCicdDeployStream(socket, params) {
    const configId = params?.configId;
    if (!configId) {
        socket.write(JSON.stringify({ success: false, error: '缺少 configId 参数' }) + '\n');
        return;
    }
    const db = getDb();
    const config = db.getCicdConfigByConfigId(configId);
    if (!config) {
        socket.write(JSON.stringify({ success: false, error: '未找到CI/CD配置' }) + '\n');
        return;
    }
    // 发送开始事件
    socket.write(JSON.stringify({ stream: true, event: 'start', configName: config.name || configId }) + '\n');
    // 调用 cicdDeploy，传入 streamCallback 将进度事件转发到 socket
    const result = await require('./cicd-handlers').cicdDeploy(db, configId, (event) => {
        const payload = { stream: true, event: event.type };
        if (event.type === 'progress') {
            // 映射 cicdService 字段 (stage/status/message) → CLI 期望 (step/detail)
            payload.step = event.data.stage || '';
            payload.detail = `[${event.data.status || ''}] ${event.data.message || ''}`;
            if (event.data.deployLogId)
                payload.deployLogId = event.data.deployLogId;
            if (event.data.configId)
                payload.configId = event.data.configId;
        }
        else {
            Object.assign(payload, event.data);
        }
        socket.write(JSON.stringify(payload) + '\n');
    });
    // 发送完成事件
    if (result.success) {
        socket.write(JSON.stringify({ stream: true, event: 'complete', success: true, deployLogId: result.deployLogId }) + '\n');
    }
    else {
        socket.write(JSON.stringify({ stream: true, event: 'error', error: result.error || '部署失败', deployLogId: result.deployLogId }) + '\n');
    }
}
// ============ Start / Stop UDS API ============
function startUdsApi() {
    // Remove stale socket file
    if (fs.existsSync(exports.UDS_SOCKET_PATH))
        fs.unlinkSync(exports.UDS_SOCKET_PATH);
    // Register all IPC handlers
    registerAllHandlers();
    udsServer = net.createServer((socket) => {
        let buffer = '';
        socket.on('data', (chunk) => {
            buffer += chunk.toString();
            const lines = buffer.split('\n');
            buffer = lines.pop() || ''; // keep incomplete line
            for (const line of lines) {
                if (!line.trim())
                    continue;
                try {
                    const req = JSON.parse(line);
                    handleJsonRequest(socket, req);
                }
                catch (e) {
                    socket.write(JSON.stringify({ success: false, error: 'Invalid JSON' }) + '\n');
                }
            }
        });
    });
    udsServer.listen(exports.UDS_SOCKET_PATH, () => {
        try {
            fs.chmodSync(exports.UDS_SOCKET_PATH, 0o766);
        }
        catch { }
        (0, logger_1.info)(`[API] UDS listening on unix://${exports.UDS_SOCKET_PATH} (${Object.keys(handlerRegistry).length} handlers)`);
    });
    udsServer.on('error', (err) => console.error('[API] UDS error:', err.message));
}
function stopUdsApi() {
    if (udsServer) {
        udsServer.close();
        udsServer = null;
        // Clean up socket file
        try {
            if (fs.existsSync(exports.UDS_SOCKET_PATH))
                fs.unlinkSync(exports.UDS_SOCKET_PATH);
        }
        catch { }
        (0, logger_1.info)('[API] UDS stopped');
    }
}
//# sourceMappingURL=uds-api.js.map