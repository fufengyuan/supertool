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
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.HTTP_API_PORT = void 0;
exports.startHttpApi = startHttpApi;
exports.stopHttpApi = stopHttpApi;
const logger_1 = require("./logger");
/**
 * HTTP REST API for SuperTool CLI (stool)
 *
 * All endpoints call existing IPC handlers — zero duplicated logic.
 * GUI and CLI share the same code path.
 *
 * Security: only listens on localhost (127.0.0.1), not accessible from network.
 */
const express_1 = __importDefault(require("express"));
const fs = __importStar(require("fs"));
const os = __importStar(require("os"));
const encryption_manager_1 = require("./encryption-manager");
exports.HTTP_API_PORT = 8686;
let httpServer = null;
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
// ============ Shared Helpers (used by both registerAllHandlers and registerRestEndpoints) ============
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
// ============ Start HTTP API ============
function startHttpApi() {
    const app = (0, express_1.default)();
    app.use(express_1.default.json({ limit: '100mb' }));
    // CORS for local development
    app.use((_req, res, next) => {
        res.set('Access-Control-Allow-Origin', '*');
        res.set('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS');
        res.set('Access-Control-Allow-Headers', 'Content-Type');
        if (_req.method === 'OPTIONS') {
            res.sendStatus(200);
            return;
        }
        next();
    });
    // Health check
    app.get('/api/health', (_req, res) => {
        res.json({ status: 'ok', service: 'SuperTool', timestamp: new Date().toISOString() });
    });
    // ============ Register ALL IPC Handlers ============
    registerAllHandlers(app);
    // ============ Specific REST endpoints (CLI) ============
    registerRestEndpoints(app);
    // ============ Catch-all ============
    app.use((_req, res) => {
        res.status(404).json({ error: 'Not found' });
    });
    // Start server
    httpServer = app.listen(exports.HTTP_API_PORT, '127.0.0.1', () => {
        (0, logger_1.info)(`[HTTP API] Listening on http://127.0.0.1:${exports.HTTP_API_PORT} (${Object.keys(handlerRegistry).length} handlers)`);
    });
    httpServer.on('error', (err) => {
        console.error('[HTTP API] Server error:', err.message);
    });
}
function stopHttpApi() {
    if (httpServer) {
        httpServer.close();
        httpServer = null;
        (0, logger_1.info)('[HTTP API] Stopped');
    }
}
// ============ Handler Registration ============
// Lazy-load modules to avoid circular dependencies
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
    // Access the main module's exports (lanService, etc.)
    try {
        return require('./main');
    }
    catch {
        return null;
    }
}
function registerAllHandlers(app) {
    const db = getDb();
    // ============ SSE: Real-time Log Streaming ============
    app.get('/api/logs/stream/:presetId', async (req, res) => {
        const presetId = req.params.presetId;
        const lines = parseInt(req.query.lines) || 100;
        const follow = req.query.follow !== 'false'; // default true
        const preset = db.getLogPresets().find((p) => p.id === presetId);
        if (!preset) {
            res.status(404).json({ error: '未找到日志预设' });
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
            res.status(400).json({ error: '预设没有关联服务器' });
            return;
        }
        const logType = preset.logType || 'file';
        const logPath = preset.logPath;
        // Same command-building logic as LogAggregator.vue buildCommand()
        function buildLogCommand(type, path, n, f) {
            const paths = path.split('\n').map(p => p.trim()).filter(p => p);
            const quote = (p) => {
                if (p.startsWith('~')) {
                    const rest = p.slice(1).replace(/'/g, "'\\''");
                    return rest ? `$HOME'${rest}'` : '$HOME';
                }
                return `'${p.replace(/'/g, "'\\''")}'`;
            };
            const quotedPaths = paths.map(quote);
            const followFlag = f ? '-f' : '';
            switch (type) {
                case 'file':
                    return `tail -n ${n} ${followFlag} ${quotedPaths.join(' ')}`.replace(/\s+/g, ' ').trim();
                case 'journalctl':
                    return `journalctl ${paths.map(u => `-u ${quote(u)}`).join(' ')} ${followFlag} -n ${n} --no-pager`;
                case 'docker':
                    return paths.map(c => `(echo "=== ${quote(c)} ===" && docker logs --tail ${n} ${f ? '--follow' : ''} ${quote(c)} 2>&1)`).join(' & ');
                case 'custom':
                    return path;
                default:
                    return `tail -n ${n} ${followFlag} ${quotedPaths.join(' ')}`.replace(/\s+/g, ' ').trim();
            }
        }
        const cmd = buildLogCommand(logType, logPath, lines, follow);
        // Use shared serverService — same connection pool as all other HTTP handlers
        const serverService = _getServerService();
        // Set SSE headers
        res.setHeader('Content-Type', 'text/event-stream');
        res.setHeader('Cache-Control', 'no-cache');
        res.setHeader('Connection', 'keep-alive');
        res.setHeader('X-Accel-Buffering', 'no');
        const streamId = `sse_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
        res.write(`data: ${JSON.stringify({ type: 'start', streamId, presetName: preset.name, command: cmd })}\n\n`);
        // Use Promise.allSettled for parallel streaming across servers (same as IPC handler)
        const stops = [];
        await Promise.allSettled(serverIds.map(async (sid) => {
            const server = db.getServerById(sid);
            if (!server) {
                res.write(`data: ${JSON.stringify({ type: 'error', serverId: sid, serverName: sid, error: 'Server not found' })}\n\n`);
                return;
            }
            try {
                await _ensureConnected(sid, server);
                const { stop } = await serverService.streamCommand(sid, cmd, (line) => {
                    try {
                        res.write(`data: ${JSON.stringify({ type: 'line', serverId: sid, serverName: server.name, line })}\n\n`);
                    }
                    catch { }
                }, () => {
                    try {
                        res.write(`data: ${JSON.stringify({ type: 'end', serverId: sid, serverName: server.name })}\n\n`);
                    }
                    catch { }
                }, (err) => {
                    try {
                        res.write(`data: ${JSON.stringify({ type: 'error', serverId: sid, serverName: server.name, error: err })}\n\n`);
                    }
                    catch { }
                });
                stops.push(stop);
            }
            catch (e) {
                res.write(`data: ${JSON.stringify({ type: 'error', serverId: sid, serverName: server.name, error: e instanceof Error ? e.message : String(e) })}\n\n`);
            }
        }));
        if (!follow) {
            res.write(`data: ${JSON.stringify({ type: 'complete' })}\n\n`);
            res.end();
        }
        // Handle client disconnect — stop all streams
        req.on('close', () => {
            stops.forEach(s => { try {
                s();
            }
            catch { } });
        });
    });
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
    registerHandler('servers:get-all', [], () => db.getAllServers());
    registerHandler('servers:get-by-id', ['serverId'], (serverId) => db.getServerById(serverId));
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
    registerHandler('cicd:get-deploy-history', ['projectId', 'limit'], (projectId, limit = 20) => db.getDeployHistory(projectId, limit));
    registerHandler('cicd:detect-tool-paths', [], () => {
        const paths = {};
        const tools = ['node', 'npm', 'yarn', 'python3', 'java', 'mvn', 'git', 'docker'];
        for (const tool of tools) {
            try {
                paths[tool] = require('child_process').execSync(`which ${tool}`, { encoding: 'utf-8' }).trim();
            }
            catch {
                paths[tool] = '';
            }
        }
        return paths;
    });
    registerHandler('cicd:detect-sdk-versions', [], () => {
        const versions = {};
        const cmds = [
            ['node', 'node --version'], ['npm', 'npm --version'], ['python3', 'python3 --version'],
            ['java', 'java -version'], ['mvn', 'mvn -version'], ['git', 'git --version'],
        ];
        const { execSync } = require('child_process');
        for (const [name, cmd] of cmds) {
            try {
                versions[name] = execSync(cmd, { encoding: 'utf-8', stdio: ['pipe', 'pipe', 'pipe'] }).trim();
            }
            catch {
                versions[name] = '';
            }
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
        // Check if this config requires approval
        const config = getDb().getCicdConfigByConfigId(configId);
        if (config && config.requiresApproval && !confirmed) {
            return { success: false, requiresApproval: true, message: '此配置需要审核确认，请确认后再次部署', configName: config.name || '' };
        }
        const { cicdDeploy } = require('./cicd-handlers');
        return cicdDeploy(getDb(), configId);
    });
    registerHandler('cicd:rollback', ['configId', 'deployHistoryId'], async (configId, deployHistoryId = '') => {
        // Check if approval is required
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
        const { execSync } = require('child_process');
        try {
            execSync(`git fetch ${remote || 'origin'}`, { cwd: repoPath, encoding: 'utf-8' });
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
        const { execSync } = require('child_process');
        try {
            execSync(`git rebase ${branch}`, { cwd: repoPath, encoding: 'utf-8' });
            return { success: true };
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    registerHandler('git:remotes', ['repoPath'], async (repoPath) => {
        const { execSync } = require('child_process');
        try {
            return { success: true, remotes: execSync('git remote -v', { cwd: repoPath, encoding: 'utf-8' }).trim() };
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    registerHandler('git:exec', ['repoPath', 'args'], async (repoPath, args) => {
        const { execSync } = require('child_process');
        try {
            return { success: true, output: execSync(`git ${args.join(' ')}`, { cwd: repoPath, encoding: 'utf-8' }).trim() };
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
    // ===== DB (query, tables, etc.) =====
    registerHandler('db:query', ['sql'], (sql) => {
        const dbInstance = db.getDatabase();
        try {
            const result = dbInstance.prepare(sql).all();
            return { success: true, data: result };
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    registerHandler('db:get-tables', [], () => {
        const dbInstance = db.getDatabase();
        return dbInstance.prepare("SELECT name FROM sqlite_master WHERE type='table'").all();
    });
    registerHandler('db:get-table-data', ['tableName', 'limit'], (tableName, limit = 100) => {
        const dbInstance = db.getDatabase();
        return dbInstance.prepare(`SELECT * FROM "${tableName}" LIMIT ${limit}`).all();
    });
    // ===== Redis =====
    registerHandler('db:redis-get', ['connectionId', 'key'], async () => ({ error: 'Redis requires GUI connection' }));
    registerHandler('db:redis-keys', ['connectionId', 'pattern'], async () => ({ error: 'Redis requires GUI connection' }));
    registerHandler('db:redis-set', ['connectionId', 'key', 'value'], async () => ({ error: 'Redis requires GUI connection' }));
    registerHandler('db:redis-delete', ['connectionId', 'key'], async () => ({ error: 'Redis requires GUI connection' }));
    registerHandler('db:redis-scan-keys', ['connectionId', 'pattern'], async () => ({ error: 'Redis requires GUI connection' }));
    registerHandler('db:redis-databases', ['connectionId'], async () => ({ error: 'Redis requires GUI connection' }));
    // ===== Log Presets =====
    // ===== Log Aggregator =====
    registerHandler('logPresets:get-all', [], () => db.getLogPresets());
    registerHandler('logPresets:add', ['data'], (data) => db.addLogPreset(data.name, data.serverIds, data.logPath, data.logType, data.keywords, data.maxLines, data.presetGroup));
    registerHandler('logPresets:update', ['id', 'updates'], (id, updates) => db.updateLogPreset(id, updates));
    registerHandler('logPresets:delete', ['id'], (id) => db.deleteLogPreset(id));
    // Helper: build tail command on server side
    function _buildTailCommand(logType, logPath, lines) {
        switch (logType) {
            case 'file': return `tail -n ${lines} ${logPath} 2>/dev/null`;
            case 'docker': {
                const containers = logPath.split('\n').filter(c => c.trim());
                return containers.map(c => `(echo '=== ${c.trim()} ===' && docker logs --tail ${lines} ${c.trim()} 2>&1)`).join(' ; ');
            }
            case 'journalctl': {
                const units = logPath.split('\n').filter(u => u.trim()).map(u => `-u ${u.trim()}`);
                return units.length > 0
                    ? `journalctl ${units.join(' ')} -n ${lines} --no-pager 2>/dev/null`
                    : `journalctl -n ${lines} --no-pager 2>/dev/null`;
            }
            case 'custom': return logPath;
            default: return `tail -n ${lines} ${logPath} 2>/dev/null`;
        }
    }
    // Helper: build grep search command on server side
    function _buildSearchCommand(logType, logPath, keyword, lines) {
        const kw = keyword.replace(/'/g, "'\\''");
        switch (logType) {
            case 'file': return `grep -i -n '${kw}' ${logPath} 2>/dev/null | tail -n ${lines}`;
            case 'docker': {
                const containers = logPath.split('\n').filter(c => c.trim());
                return containers.map(c => `(echo '=== ${c.trim()} ===' && docker logs ${c.trim()} 2>&1 | grep -i -n '${kw}' | tail -n ${lines})`).join(' ; ');
            }
            case 'journalctl': {
                const units = logPath.split('\n').filter(u => u.trim()).map(u => `-u ${u.trim()}`);
                return units.length > 0
                    ? `journalctl ${units.join(' ')} --grep='${kw}' -n ${lines} --no-pager 2>/dev/null`
                    : `journalctl --grep='${kw}' -n ${lines} --no-pager 2>/dev/null`;
            }
            case 'custom': return `${logPath} 2>/dev/null | grep -i -n '${kw}' | tail -n ${lines}`;
            default: return `grep -i -n '${kw}' ${logPath} 2>/dev/null | tail -n ${lines}`;
        }
    }
    // Helper: execute command using shared serverService connection pool
    async function execOnServer(serverId, command) {
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
    registerHandler('servers:exec', ['id', 'command'], async (id, command) => {
        const server = db.getServerById(id);
        if (!server)
            return { success: false, error: 'Server not found' };
        if (server.requiresApproval)
            return { success: false, requiresApproval: true, serverName: server.name, error: '该服务器已开启执行审核，请在 GUI 中手动确认' };
        try {
            await _ensureConnected(id, server);
            const svc = _getServerService();
            const result = await svc.execCommand(id, command);
            return { success: result.success, output: result.output };
        }
        catch (e) {
            return { success: false, error: e instanceof Error ? e.message : String(e) };
        }
    });
    // ===== Update =====
    registerHandler('update:get-version', [], () => ({ version: require('../package.json').version }));
    (0, logger_1.info)(`[HTTP API] Registered ${Object.keys(handlerRegistry).length} handlers`);
}
// ============ Helper: call IPC handler from HTTP endpoint ============
async function callHandler(name, req) {
    const entry = handlerRegistry[name];
    if (!entry)
        throw new Error(`Handler '${name}' not found`);
    const args = entry.paramNames.map((pn, idx) => {
        // 1) Check params, query, body for named argument
        if (Object.prototype.hasOwnProperty.call(req.params, pn))
            return req.params[pn];
        if (Object.prototype.hasOwnProperty.call(req.query, pn))
            return req.query[pn];
        if (req.body && Object.prototype.hasOwnProperty.call(req.body, pn))
            return req.body[pn];
        // 2) Fallback: if this is the only param and body exists, pass entire body
        if (entry.paramNames.length === 1 && req.body && idx === 0)
            return req.body;
        return undefined;
    });
    return entry.handler(...args);
}
// ============ Specific REST Endpoints ============
function registerRestEndpoints(app) {
    const db = getDb();
    // Health
    app.get('/api/health', (_req, res) => res.json({ status: 'ok', service: 'SuperTool', timestamp: new Date().toISOString() }));
    // ===== Todos =====
    app.get('/api/todos', async (_req, res) => res.json(await callHandler('todos:get-all', _req)));
    app.post('/api/todos', async (req, res) => res.json(await callHandler('todos:add', req)));
    app.put('/api/todos/:id', async (req, res) => res.json(await callHandler('todos:update', req)));
    app.delete('/api/todos/:id', async (req, res) => res.json(await callHandler('todos:delete', req)));
    app.post('/api/todos/bulk-delete', async (req, res) => res.json(await callHandler('todos:delete-many', req)));
    app.put('/api/todos/reorder', async (req, res) => res.json(await callHandler('todos:update-order', req)));
    app.post('/api/todos/repeat', async (req, res) => res.json(await callHandler('todos:create-repeat-instance', req)));
    // ===== Tags =====
    app.get('/api/tags', async (_req, res) => res.json(await callHandler('tags:get-all', _req)));
    app.post('/api/tags', async (req, res) => res.json(await callHandler('tags:add', req)));
    app.delete('/api/tags/:name', async (req, res) => res.json(await callHandler('tags:delete', req)));
    // ===== Settings =====
    app.get('/api/settings/:key', async (req, res) => res.json(await callHandler('settings:get', req)));
    app.put('/api/settings/:key', async (req, res) => res.json(await callHandler('settings:set', req)));
    // ===== Subtasks =====
    app.get('/api/todos/:todoId/subtasks', async (req, res) => res.json(await callHandler('subtasks:get-for-todo', req)));
    app.post('/api/subtasks', async (req, res) => res.json(await callHandler('subtasks:add', req)));
    app.put('/api/subtasks/:id', async (req, res) => res.json(await callHandler('subtasks:update', req)));
    app.delete('/api/subtasks/:id', async (req, res) => res.json(await callHandler('subtasks:delete', req)));
    // ===== Projects =====
    app.get('/api/projects', async (req, res) => res.json(await callHandler('projects:get-all', req)));
    app.post('/api/projects', async (req, res) => res.json(await callHandler('projects:add', req)));
    app.put('/api/projects/:id', async (req, res) => res.json(await callHandler('projects:update', req)));
    app.delete('/api/projects/:id', async (req, res) => res.json(await callHandler('projects:delete', req)));
    // ===== Servers =====
    app.get('/api/servers', async (_req, res) => res.json(await callHandler('servers:get-all', _req)));
    app.get('/api/servers/:id', async (req, res) => res.json(await callHandler('servers:get-by-id', req)));
    app.post('/api/servers', async (req, res) => res.json(await callHandler('servers:add', req)));
    app.put('/api/servers/:id', async (req, res) => res.json(await callHandler('servers:update', req)));
    app.delete('/api/servers/:id', async (req, res) => res.json(await callHandler('servers:delete', req)));
    app.post('/api/servers/test-connection', async (req, res) => res.json(await callHandler('servers:test-connection', req)));
    app.get('/api/server-groups', async (_req, res) => res.json(await callHandler('servers:groups:get-all', _req)));
    // Batch exec MUST be before :id routes to avoid Express matching "batch" as :id
    app.post('/api/servers/batch/exec', async (req, res) => {
        const { command, serverIds, tag } = req.body;
        if (!command)
            return res.status(400).json({ error: 'Missing command' });
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
        res.json({ results: results.map((r) => r.status === 'fulfilled' ? r.value : { success: false, error: String(r.reason) }) });
    });
    app.post('/api/servers/:id/exec', async (req, res) => res.json(await callHandler('servers:exec', req)));
    // Read remote file via SSH
    app.post('/api/servers/:id/read-file', async (req, res) => {
        const server = db.getServerById(req.params.id);
        if (!server)
            return res.status(404).json({ success: false, error: 'Server not found' });
        try {
            await _ensureConnected(req.params.id, server);
            const svc = _getServerService();
            const result = await svc.execCommand(req.params.id, `cat ${req.body.path}`);
            if (result.success)
                return res.json({ success: true, content: result.output });
            return res.json({ success: false, error: result.error || 'Failed to read file' });
        }
        catch (e) {
            res.status(500).json({ success: false, error: e instanceof Error ? e.message : String(e) });
        }
    });
    // ===== CI/CD =====
    app.get('/api/cicd/configs', async (_req, res) => res.json(await callHandler('cicd:get-all-configs', _req)));
    app.get('/api/cicd/logs/:projectId', async (req, res) => res.json(await callHandler('cicd:get-logs', req)));
    app.get('/api/cicd/history/:projectId', async (req, res) => res.json(await callHandler('cicd:get-deploy-history', req)));
    app.get('/api/cicd/tools', async (_req, res) => res.json(await callHandler('cicd:detect-tool-paths', _req)));
    app.get('/api/cicd/modules/:configId', async (req, res) => res.json(await callHandler('cicd:get-modules', req)));
    app.post('/api/cicd/modules/:configId', async (req, res) => res.json(await callHandler('cicd:add-module', req)));
    app.delete('/api/cicd/modules/:moduleId', async (req, res) => res.json(await callHandler('cicd:delete-module', req)));
    app.post('/api/cicd/deploy/:configId', async (req, res) => res.json(await callHandler('cicd:deploy', req)));
    // SSE stream deploy with approval check
    app.get('/api/cicd/deploy-stream/:configId', async (req, res) => {
        const configId = req.params.configId;
        const confirmed = req.query.confirmed === 'true';
        // Check approval requirement
        const config = getDb().getCicdConfigByConfigId(configId);
        if (config && config.requiresApproval && !confirmed) {
            return res.status(403).json({ success: false, requiresApproval: true, message: '此配置需要审核确认，请添加 ?confirmed=true 参数', configName: config.name || '' });
        }
        // SSE stream: trigger deploy and stream progress
        res.setHeader('Content-Type', 'text/event-stream');
        res.setHeader('Cache-Control', 'no-cache');
        res.setHeader('Connection', 'keep-alive');
        res.flushHeaders?.();
        // Send start event
        res.write(`data: ${JSON.stringify({ type: 'start', configId })}\n\n`);
        try {
            const { cicdDeploy } = require('./cicd-handlers');
            const result = await cicdDeploy(getDb(), configId);
            res.write(`data: ${JSON.stringify({ type: 'complete', success: result.success, error: result.error })}\n\n`);
        }
        catch (e) {
            res.write(`data: ${JSON.stringify({ type: 'error', error: getErrorMessage(e) })}\n\n`);
        }
        res.end();
    });
    app.post('/api/cicd/rollback/:configId', async (req, res) => res.json(await callHandler('cicd:rollback', req)));
    app.post('/api/cicd/cancel/:deployLogId', async (req, res) => res.json(await callHandler('cicd:cancel', req)));
    // ===== Git =====
    app.get('/api/git/repos', async (_req, res) => res.json(await callHandler('git:repos:get-all', _req)));
    app.post('/api/git/repos', async (req, res) => res.json(await callHandler('git:repos:add', req)));
    app.put('/api/git/repos/:id', async (req, res) => res.json(await callHandler('git:repos:update', req)));
    app.delete('/api/git/repos/:id', async (req, res) => res.json(await callHandler('git:repos:delete', req)));
    // ===== MFA =====
    app.get('/api/mfa/secrets', async (_req, res) => res.json(await callHandler('mfa:get-secrets', _req)));
    app.post('/api/mfa/secrets', async (req, res) => res.json(await callHandler('mfa:add-secret', req)));
    app.delete('/api/mfa/secrets/:id', async (req, res) => res.json(await callHandler('mfa:delete-secret', req)));
    app.post('/api/mfa/generate', async (req, res) => res.json(await callHandler('mfa:generate-code', req)));
    // ===== Notes =====
    app.get('/api/notes', async (req, res) => res.json(await callHandler('notes:get-all', req)));
    app.delete('/api/notes/:id', async (req, res) => res.json(await callHandler('notes:delete', req)));
    app.get('/api/note-groups', async (_req, res) => res.json(await callHandler('note-groups:get-all', _req)));
    // ===== Weekly Reports =====
    app.get('/api/weekly-reports', async (req, res) => res.json(await callHandler('weekly-report:get-all', req)));
    app.get('/api/weekly-reports/:id', async (req, res) => res.json(await callHandler('weekly-report:get', req)));
    // ===== Accounting =====
    app.get('/api/accounting/categories', async (_req, res) => res.json(await callHandler('accounting:categories:get', _req)));
    app.get('/api/accounting/records', async (req, res) => res.json(await callHandler('accounting:records:get', req)));
    app.get('/api/accounting/stats', async (req, res) => res.json(await callHandler('accounting:stats:get', req)));
    app.get('/api/accounting/budgets', async (_req, res) => res.json(await callHandler('accounting:budgets:get', _req)));
    app.get('/api/accounting/templates', async (_req, res) => res.json(await callHandler('accounting:templates:get', _req)));
    app.get('/api/accounting/trend', async (req, res) => res.json(await callHandler('accounting:trend:get', req)));
    app.delete('/api/accounting/records/:id', async (req, res) => res.json(await callHandler('accounting:records:delete', req)));
    app.delete('/api/accounting/budgets/:id', async (req, res) => res.json(await callHandler('accounting:budgets:delete', req)));
    app.delete('/api/accounting/templates/:id', async (req, res) => res.json(await callHandler('accounting:templates:delete', req)));
    app.post('/api/accounting/templates/:id/use', async (req, res) => res.json(await callHandler('accounting:templates:use', req)));
    // ===== OpenVPN =====
    app.get('/api/openvpn', async (_req, res) => res.json(await callHandler('openvpn:get-all', _req)));
    app.delete('/api/openvpn/:id', async (req, res) => res.json(await callHandler('openvpn:delete', req)));
    // ===== Backup =====
    app.get('/api/backup/export', async (_req, res) => res.json(await callHandler('backup:export-data', _req)));
    app.post('/api/backup/import', async (req, res) => res.json(await callHandler('backup:import-json', req)));
    // ===== DB =====
    // Helper: get all DB connection configs from settings
    function getDBConnections() {
        try {
            const raw = db.getSetting('db_connections');
            return raw ? JSON.parse(raw) : [];
        }
        catch {
            return [];
        }
    }
    // Helper: auto-connect to a saved DB config by its ID
    async function autoConnectDb(dbId) {
        const dbManager = require('./services/db-manager').default;
        const connections = getDBConnections();
        const config = connections.find((c) => c.id === dbId);
        if (!config)
            return { success: false, error: `DB config '${dbId}' not found` };
        try {
            await dbManager.connect(config.id, config);
            return { success: true };
        }
        catch (e) {
            return { success: false, error: e instanceof Error ? e.message : String(e) };
        }
    }
    // === External DB: list saved connections ===
    app.get('/api/db/connections', async (_req, res) => {
        const connections = getDBConnections();
        res.json({ success: true, connections });
    });
    // === External DB: connect ===
    app.post('/api/db/connect', async (req, res) => {
        const dbManager = require('./services/db-manager').default;
        try {
            const config = req.body;
            await dbManager.connect(config.id, config);
            res.json({ success: true });
        }
        catch (e) {
            res.status(500).json({ success: false, error: getErrorMessage(e) });
        }
    });
    // === External DB: disconnect ===
    app.post('/api/db/disconnect', async (req, res) => {
        const dbManager = require('./services/db-manager').default;
        try {
            await dbManager.disconnect(req.body.id);
            res.json({ success: true });
        }
        catch (e) {
            res.status(500).json({ success: false, error: getErrorMessage(e) });
        }
    });
    // === External DB: query ===
    app.post('/api/db/query', async (req, res) => {
        const dbManager = require('./services/db-manager').default;
        const { dbId, sql } = req.body;
        if (!dbId)
            return res.status(400).json({ error: 'Missing dbId' });
        try {
            // Check if connection requires approval
            const conns = getDBConnections();
            const conn = conns.find((c) => c.id === dbId);
            if (conn && conn.requiresApproval) {
                return res.status(403).json({ success: false, requiresApproval: true, message: `数据库「${conn.name}」已开启安全审核，CLI 不支持执行 SQL。请在 GUI 中操作。` });
            }
            if (!dbManager.isConnected(dbId)) {
                const result = await autoConnectDb(dbId);
                if (!result.success)
                    return res.status(500).json({ error: result.error });
            }
            const rows = await dbManager.query(dbId, sql);
            res.json({ success: true, rows });
        }
        catch (e) {
            res.status(500).json({ success: false, error: getErrorMessage(e) });
        }
    });
    // === External DB: get databases ===
    app.get('/api/db/databases', async (req, res) => {
        const dbManager = require('./services/db-manager').default;
        const { dbId } = req.query;
        if (!dbId)
            return res.status(400).json({ error: 'Missing dbId' });
        try {
            if (!dbManager.isConnected(dbId)) {
                const result = await autoConnectDb(dbId);
                if (!result.success)
                    return res.status(500).json({ error: result.error });
            }
            const databases = await dbManager.getDatabases(dbId);
            res.json({ success: true, databases });
        }
        catch (e) {
            res.status(500).json({ success: false, error: getErrorMessage(e) });
        }
    });
    // === External DB: get tables ===
    app.get('/api/db/tables', async (req, res) => {
        const dbManager = require('./services/db-manager').default;
        const { dbId, db: dbName } = req.query;
        if (!dbId)
            return res.status(400).json({ error: 'Missing dbId' });
        try {
            if (!dbManager.isConnected(dbId)) {
                const result = await autoConnectDb(dbId);
                if (!result.success)
                    return res.status(500).json({ error: result.error });
            }
            const tables = await dbManager.getTables(dbId, dbName || '');
            res.json({ success: true, tables });
        }
        catch (e) {
            res.status(500).json({ success: false, error: getErrorMessage(e) });
        }
    });
    // === External DB: export table data ===
    app.get('/api/db/tables/:tableName', async (req, res) => {
        const dbManager = require('./services/db-manager').default;
        const { dbId, db: dbName } = req.query;
        if (!dbId)
            return res.status(400).json({ error: 'Missing dbId' });
        try {
            if (!dbManager.isConnected(dbId)) {
                const result = await autoConnectDb(dbId);
                if (!result.success)
                    return res.status(500).json({ error: result.error });
            }
            const result = await dbManager.getTableData(dbId, req.params.tableName, 500, 0, dbName || '');
            res.json({ success: true, ...result });
        }
        catch (e) {
            res.status(500).json({ success: false, error: getErrorMessage(e) });
        }
    });
    // === External DB: Redis operations ===
    app.get('/api/db/redis/keys', async (req, res) => {
        const dbManager = require('./services/db-manager').default;
        const { dbId, pattern } = req.query;
        if (!dbId)
            return res.status(400).json({ error: 'Missing dbId' });
        try {
            if (!dbManager.isConnected(dbId)) {
                const result = await autoConnectDb(dbId);
                if (!result.success)
                    return res.status(500).json({ error: result.error });
            }
            const result = await dbManager.getRedisKeys(dbId, pattern || '*');
            res.json({ success: true, ...result });
        }
        catch (e) {
            res.status(500).json({ success: false, error: getErrorMessage(e) });
        }
    });
    app.get('/api/db/redis/:key', async (req, res) => {
        const dbManager = require('./services/db-manager').default;
        const { dbId } = req.query;
        if (!dbId)
            return res.status(400).json({ error: 'Missing dbId' });
        try {
            if (!dbManager.isConnected(dbId)) {
                const result = await autoConnectDb(dbId);
                if (!result.success)
                    return res.status(500).json({ error: result.error });
            }
            const value = await dbManager.getValue(dbId, req.params.key);
            res.json({ success: true, value });
        }
        catch (e) {
            res.status(500).json({ success: false, error: getErrorMessage(e) });
        }
    });
    app.post('/api/db/redis', async (req, res) => {
        const dbManager = require('./services/db-manager').default;
        const { dbId, key, value, expiry } = req.body;
        if (!dbId)
            return res.status(400).json({ error: 'Missing dbId' });
        try {
            if (!dbManager.isConnected(dbId)) {
                const result = await autoConnectDb(dbId);
                if (!result.success)
                    return res.status(500).json({ error: result.error });
            }
            await dbManager.setRedisValue(dbId, key, value, expiry);
            res.json({ success: true });
        }
        catch (e) {
            res.status(500).json({ success: false, error: getErrorMessage(e) });
        }
    });
    app.delete('/api/db/redis/:key', async (req, res) => {
        const dbManager = require('./services/db-manager').default;
        const { dbId } = req.query;
        if (!dbId)
            return res.status(400).json({ error: 'Missing dbId' });
        try {
            if (!dbManager.isConnected(dbId)) {
                const result = await autoConnectDb(dbId);
                if (!result.success)
                    return res.status(500).json({ error: result.error });
            }
            await dbManager.deleteRedisKey(dbId, req.params.key);
            res.json({ success: true });
        }
        catch (e) {
            res.status(500).json({ success: false, error: getErrorMessage(e) });
        }
    });
    // Generic Redis command execution (TYPE, TTL, HGET, HGETALL, HLEN, LRANGE, LLEN, SMEMBERS, SCARD, etc.)
    app.post('/api/db/redis/command', async (req, res) => {
        const dbManager = require('./services/db-manager').default;
        const { dbId, command, args } = req.body;
        if (!dbId)
            return res.status(400).json({ success: false, error: 'Missing dbId' });
        if (!command)
            return res.status(400).json({ success: false, error: 'Missing command' });
        try {
            if (!dbManager.isConnected(dbId)) {
                const result = await autoConnectDb(dbId);
                if (!result.success)
                    return res.status(500).json({ error: result.error });
            }
            const result = await dbManager.execRedisCommand(dbId, command);
            res.json({ success: true, result });
        }
        catch (e) {
            res.status(500).json({ success: false, error: getErrorMessage(e) });
        }
    });
    // ===== Log Presets =====
    app.get('/api/log-presets', async (_req, res) => res.json(await callHandler('logPresets:get-all', _req)));
    // ===== SFTP REST Endpoints =====
    app.get('/api/servers/:id/sftp/ls', async (req, res) => {
        const { id } = req.params;
        const remotePath = req.query.path || '.';
        res.json(await callHandler('servers:sftp:list', { ...req, params: { serverId: id, remotePath } }));
    });
    app.get('/api/servers/:id/sftp/read', async (req, res) => {
        const { id } = req.params;
        const remotePath = req.query.path;
        if (!remotePath)
            return res.status(400).json({ error: 'Missing path parameter' });
        const result = await callHandler('servers:sftp:read-file', { ...req, params: { serverId: id, remotePath } });
        res.json(result);
    });
    app.post('/api/servers/:id/sftp/mkdir', async (req, res) => {
        const { id } = req.params;
        req.params.remotePath = req.body.path;
        res.json(await callHandler('servers:sftp:mkdir', req));
    });
    app.delete('/api/servers/:id/sftp', async (req, res) => {
        const { id } = req.params;
        const remotePath = req.query.path;
        if (!remotePath)
            return res.status(400).json({ error: 'Missing path query parameter' });
        req.params.remotePath = remotePath;
        res.json(await callHandler('servers:sftp:delete', req));
    });
    // ===== Git REST Endpoints =====
    app.get('/api/git/status', async (req, res) => {
        const repoPath = req.query.path;
        if (!repoPath)
            return res.status(400).json({ error: 'Missing path' });
        res.json(await callHandler('git:status', { ...req, params: { repoPath } }));
    });
    app.get('/api/git/log', async (req, res) => {
        const repoPath = req.query.path;
        if (!repoPath)
            return res.status(400).json({ error: 'Missing path' });
        const limit = parseInt(req.query.limit) || 20;
        res.json(await callHandler('git:log', { ...req, params: { repoPath }, body: { options: { limit } } }));
    });
    app.get('/api/git/branches', async (req, res) => {
        const repoPath = req.query.path;
        if (!repoPath)
            return res.status(400).json({ error: 'Missing path' });
        res.json(await callHandler('git:branches', { ...req, params: { repoPath } }));
    });
    app.post('/api/git/pull', async (req, res) => {
        const repoPath = req.body.path;
        if (!repoPath)
            return res.status(400).json({ error: 'Missing path' });
        res.json(await callHandler('git:pull', { ...req, params: { repoPath } }));
    });
    app.post('/api/git/push', async (req, res) => {
        const repoPath = req.body.path;
        if (!repoPath)
            return res.status(400).json({ error: 'Missing path' });
        res.json(await callHandler('git:push', { ...req, params: { repoPath } }));
    });
    app.post('/api/git/commit', async (req, res) => {
        const { path: repoPath, message, files } = req.body;
        if (!repoPath || !message)
            return res.status(400).json({ error: 'Missing path or message' });
        res.json(await callHandler('git:commit', { ...req, params: { repoPath }, body: { message, files } }));
    });
    app.post('/api/git/checkout', async (req, res) => {
        const { path: repoPath, branch } = req.body;
        if (!repoPath || !branch)
            return res.status(400).json({ error: 'Missing path or branch' });
        res.json(await callHandler('git:checkout', { ...req, params: { repoPath }, body: { branch } }));
    });
    // ===== CICD Step Logs =====
    app.get('/api/cicd/step-logs/:deployLogId', async (req, res) => res.json(await callHandler('cicd:get-step-logs', req)));
    // ===== Project Stats =====
    app.get('/api/projects/:id/stats', async (req, res) => res.json(await callHandler('projects:get-stats', req)));
    app.get('/api/projects/:id/todos', async (req, res) => res.json(await callHandler('projects:get-todos', req)));
    app.post('/api/log-presets', async (req, res) => res.json(await callHandler('logPresets:add', req)));
    app.delete('/api/log-presets/:id', async (req, res) => res.json(await callHandler('logPresets:delete', req)));
    // ===== Logs =====
    app.post('/api/logs/search', async (req, res) => res.json(await callHandler('log:search', req)));
    // ===== Notification =====
    app.post('/api/notification/test', async (_req, res) => res.json(await callHandler('notification:test', _req)));
    app.put('/api/notification/settings', async (req, res) => res.json(await callHandler('notification:set-settings', req)));
    app.get('/api/notification/settings', async (_req, res) => res.json(await callHandler('notification:get-settings', _req)));
    // ===== Endpoints list =====
    app.get('/api/endpoints', (_req, res) => res.json({ handlers: Object.keys(handlerRegistry).sort(), count: Object.keys(handlerRegistry).length }));
}
//# sourceMappingURL=http-api.js.map