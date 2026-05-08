"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getActiveLogStreams = getActiveLogStreams;
exports.registerLogHandlers = registerLogHandlers;
const electron_1 = require("electron");
const server_handlers_1 = require("./server-handlers");
const window_manager_1 = require("./window-manager");
const encryption_manager_1 = require("./encryption-manager");
const activeLogStreams = new Map();
function getActiveLogStreams() { return activeLogStreams; }
function registerLogHandlers(db, requireService) {
    // ============ 日志聚合流式查询 ============
    electron_1.ipcMain.handle('logs:start-stream', async (_event, streamId, params) => {
        const wasNew = !(0, server_handlers_1.getServerService)();
        if (!(0, server_handlers_1.getServerService)()) {
            const ServerService = requireService('server-service');
            const { setServerService } = require('./server-handlers');
            const svc = new ServerService();
            setServerService(svc);
            if (wasNew) {
                svc.on('connected', (data) => {
                    if ((0, window_manager_1.getMainWindow)())
                        (0, window_manager_1.getMainWindow)().webContents.send('server:connected', data);
                });
                svc.on('disconnected', (data) => {
                    if ((0, window_manager_1.getMainWindow)())
                        (0, window_manager_1.getMainWindow)().webContents.send('server:disconnected', data);
                });
                svc.on('connection-error', (data) => {
                    if ((0, window_manager_1.getMainWindow)())
                        (0, window_manager_1.getMainWindow)().webContents.send('server:connection-error', data);
                });
                svc.on('terminal-data', (data) => {
                    if ((0, window_manager_1.getMainWindow)())
                        (0, window_manager_1.getMainWindow)().webContents.send('terminal:data', data);
                });
                svc.on('terminal-close', (data) => {
                    if ((0, window_manager_1.getMainWindow)())
                        (0, window_manager_1.getMainWindow)().webContents.send('terminal:close', data);
                });
            }
        }
        const stops = [];
        if (activeLogStreams.has(streamId)) {
            const oldStops = activeLogStreams.get(streamId);
            oldStops.forEach(s => { try {
                s.stop();
            }
            catch { } });
            activeLogStreams.delete(streamId);
        }
        activeLogStreams.set(streamId, stops);
        const blockedCommands = ['rm', 'wget', 'curl', 'chmod', 'chown', 'sudo', 'dd', 'mkfs', 'reboot', 'shutdown', 'kill', 'iptables', 'mount', 'umount', 'mktemp', 'crontab', 'passwd', 'useradd', 'userdel', 'groupadd', 'visudo', 'chroot', 'systemctl', 'service', 'nohup', 'at', 'batch'];
        const cmdTokens = params.command.trim().split(/\s+/);
        const logCmd = cmdTokens[0].toLowerCase().replace(/^['"/\\]/g, '');
        const isDockerLogs = cmdTokens[0].toLowerCase() === 'docker' && cmdTokens[1]?.toLowerCase() === 'logs';
        const isJournalctl = logCmd === 'journalctl';
        const allowedLogCommands = ['tail', 'grep', 'cat', 'head', 'wc', 'awk', 'sed', 'less', 'more', 'jq'];
        if (blockedCommands.includes(logCmd)) {
            return { success: false, error: `Command blocked for security: ${logCmd}` };
        }
        if (!isDockerLogs && !isJournalctl && !allowedLogCommands.includes(logCmd)) {
            return { success: false, error: `Command not allowed for log streaming: ${logCmd}. Allowed: ${allowedLogCommands.join(', ')}, docker logs, journalctl` };
        }
        let startedCount = 0;
        await Promise.allSettled(params.serverIds.map(async (serverId) => {
            try {
                const server = db.getServerById(serverId);
                if (!server) {
                    if ((0, window_manager_1.getMainWindow)())
                        (0, window_manager_1.getMainWindow)().webContents.send('logs:error', { streamId, serverId, error: '服务器不存在' });
                    return;
                }
                if (!(0, server_handlers_1.getServerService)().isConnected(serverId)) {
                    const plainPassword = server.password ? (0, encryption_manager_1.decryptPassword)(server.password) : undefined;
                    await (0, server_handlers_1.getServerService)().connect({ ...server, password: plainPassword });
                }
                const { stop } = await (0, server_handlers_1.getServerService)().streamCommand(serverId, params.command, (line) => {
                    try {
                        if ((0, window_manager_1.getMainWindow)())
                            (0, window_manager_1.getMainWindow)().webContents.send('logs:line', { streamId, serverId, serverName: server.name, line });
                    }
                    catch { }
                }, () => {
                    try {
                        if ((0, window_manager_1.getMainWindow)())
                            (0, window_manager_1.getMainWindow)().webContents.send('logs:server-end', { streamId, serverId });
                    }
                    catch { }
                }, (err) => {
                    try {
                        if ((0, window_manager_1.getMainWindow)())
                            (0, window_manager_1.getMainWindow)().webContents.send('logs:error', { streamId, serverId, error: err });
                    }
                    catch { }
                });
                stops.push({ stop });
                startedCount++;
            }
            catch (e) {
                try {
                    if ((0, window_manager_1.getMainWindow)())
                        (0, window_manager_1.getMainWindow)().webContents.send('logs:error', { streamId, serverId, error: e.message });
                }
                catch { }
            }
        }));
        if (startedCount === 0 && params.serverIds.length > 0)
            return { success: false, error: '所有服务器连接失败' };
        return { success: true, streamId };
    });
    electron_1.ipcMain.handle('logs:stop-stream', (_event, streamId) => {
        const stops = activeLogStreams.get(streamId);
        if (stops) {
            stops.forEach(s => { try {
                s.stop();
            }
            catch { } });
            activeLogStreams.delete(streamId);
        }
        if ((0, window_manager_1.getMainWindow)())
            (0, window_manager_1.getMainWindow)().webContents.send('logs:stream-stopped', { streamId });
        return { success: true };
    });
    // ============ 日志搜索 ============
    // CLI build_search_command 保持一致：同样的 grep 命令构建逻辑
    electron_1.ipcMain.handle('logs:search', async (_event, params) => {
        const wasNew = !(0, server_handlers_1.getServerService)();
        if (!(0, server_handlers_1.getServerService)()) {
            const ServerService = requireService('server-service');
            const { setServerService } = require('./server-handlers');
            const svc = new ServerService();
            setServerService(svc);
            if (wasNew) {
                svc.on('connected', (data) => { if ((0, window_manager_1.getMainWindow)())
                    (0, window_manager_1.getMainWindow)().webContents.send('server:connected', data); });
                svc.on('disconnected', (data) => { if ((0, window_manager_1.getMainWindow)())
                    (0, window_manager_1.getMainWindow)().webContents.send('server:disconnected', data); });
                svc.on('connection-error', (data) => { if ((0, window_manager_1.getMainWindow)())
                    (0, window_manager_1.getMainWindow)().webContents.send('server:connection-error', data); });
            }
        }
        const { keyword, contextLines } = params;
        if (!keyword.trim())
            return { success: false, error: '关键字不能为空' };
        function buildGrepCmd(logType, logPath, kw, ctx) {
            const escapedKw = kw.replace(/'/g, "'\\''");
            const grepCtx = ctx > 0 ? ` -C ${ctx}` : '';
            // 注意：和 CLI 保持命令一致，但加 -i 支持大小写不敏感
            const grep = `grep${grepCtx} -i -n '${escapedKw}'`;
            switch (logType) {
                case 'file': {
                    // CLI: grep -n 'kw' path 2>/dev/null | tail -N
                    const paths = logPath.split('\n').map((p) => p.trim()).filter((p) => p);
                    const q = (p) => `'${p.replace(/'/g, "'\\''")}'`;
                    return `${grep} ${paths.map(q).join(' ')} 2>/dev/null`;
                }
                case 'docker': {
                    // CLI: docker logs name 2>&1 | grep -n 'kw' | tail -N
                    const containers = logPath.split('\n').filter((c) => c.trim());
                    return containers.map((c) => {
                        const name = c.trim();
                        return `docker logs '${name}' 2>&1 | grep${grepCtx} -i -n '${escapedKw}'`;
                    }).join(' ; ');
                }
                case 'journalctl': {
                    // CLI: journalctl -u X --grep='kw' -n N --no-pager 2>/dev/null
                    const units = logPath.split('\n')
                        .map((u) => u.trim())
                        .filter((u) => u);
                    if (units.length === 0) {
                        return `journalctl --grep='${escapedKw}' --no-pager 2>/dev/null`;
                    }
                    return `journalctl ${units.map((u) => `-u '${u.replace(/'/g, "'\\''")}'`).join(' ')} --grep='${escapedKw}' --no-pager 2>/dev/null`;
                }
                default: {
                    const paths = logPath.split('\n').map((p) => p.trim()).filter((p) => p);
                    const q = (p) => `'${p.replace(/'/g, "'\\''")}'`;
                    return `${grep} ${paths.map(q).join(' ')} 2>/dev/null`;
                }
            }
        }
        function parseGrepOutput(output, kw, ctx) {
            const lines = output.split('\n').filter(l => l.trim() && l.trim() !== '--');
            const kwLower = kw.toLowerCase();
            const result = [];
            for (const line of lines) {
                // grep -n output: "filename:lineNum:content" or "lineNum:content"
                // 匹配行: lineNum:content，上下文行: lineNum-content
                const matchLine = line.match(/^(?:[^:]*:)?(\d+):(.*)$/);
                const contextLine = line.match(/^(?:[^:]*:)?(\d+)-(.*)$/);
                const parsed = matchLine || contextLine;
                if (!parsed)
                    continue;
                const lineNum = parsed[1];
                let content = parsed[2];
                content = content.replace(/\x1b\[[0-9;]*m/g, '');
                const isMatch = content.toLowerCase().includes(kwLower);
                result.push({ content, isMatch, lineNum });
            }
            return result;
        }
        const matches = [];
        await Promise.allSettled(params.serverIds.map(async (serverId) => {
            try {
                const server = db.getServerById(serverId);
                if (!server) {
                    matches.push({ serverId, serverName: serverId, matchCount: 0, lines: [] });
                    return;
                }
                if (!(0, server_handlers_1.getServerService)().isConnected(serverId)) {
                    const plainPassword = server.password ? (0, encryption_manager_1.decryptPassword)(server.password) : undefined;
                    await (0, server_handlers_1.getServerService)().connect({ ...server, password: plainPassword });
                }
                const cmd = buildGrepCmd(params.logType, params.logPath, keyword, contextLines);
                const result = await (0, server_handlers_1.getServerService)().execCommand(serverId, cmd);
                const output = (result.output || '').trim();
                const lines = output ? parseGrepOutput(output, keyword, contextLines) : [];
                const matchCount = lines.filter(l => l.isMatch).length;
                matches.push({
                    serverId,
                    serverName: server.name,
                    matchCount,
                    lines
                });
            }
            catch {
                matches.push({ serverId, serverName: serverId, matchCount: 0, lines: [] });
            }
        }));
        return { success: true, matches };
    });
} // end registerLogHandlers
//# sourceMappingURL=log-handlers.js.map