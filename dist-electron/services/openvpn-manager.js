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
exports.openVPNManager = void 0;
const child_process_1 = require("child_process");
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
const os = __importStar(require("os"));
const net = __importStar(require("net"));
const electron_1 = require("electron");
class OpenVPNManager {
    constructor() {
        this.process = null;
        this.status = {
            connected: false,
            configId: null,
            configName: null,
            state: 'disconnected',
            log: [],
        };
        this.tempConfigPath = null;
        this.logBuffer = [];
        this.MAX_LOG_LINES = 500;
        this.pendingConnect = null;
        // Management socket connection
        this.mgmtSocket = null;
        this.mgmtSocketPath = null;
        // Sudo password cache — we store the actual password briefly for auto-reconnect
        // within the same session. It's cleared on disconnect.
        this.cachedSudoPassword = null;
    }
    /**
     * Get the path to the bundled OpenVPN binary
     */
    getBundledPath() {
        const platform = os.platform();
        const arch = os.arch();
        let platformDir;
        let binaryName;
        if (platform === 'win32') {
            platformDir = 'win-x64';
            binaryName = 'openvpn.exe';
        }
        else if (platform === 'darwin') {
            platformDir = arch === 'arm64' ? 'macos-arm64' : 'macos-x64';
            binaryName = 'openvpn';
        }
        else {
            // Linux
            platformDir = arch === 'arm64' ? 'linux-arm64' : 'linux-x64';
            binaryName = 'openvpn';
        }
        // Development mode: look in project resources/
        // Use app.getAppPath() instead of process.cwd() — more reliable in Electron dev
        if (!electron_1.app.isPackaged) {
            const appPath = electron_1.app.getAppPath();
            const devPath = path.join(appPath, 'resources', 'openvpn', platformDir, binaryName);
            if (fs.existsSync(devPath))
                return devPath;
        }
        // Production mode: look in process.resourcesPath
        if (process.resourcesPath) {
            const prodPath = path.join(process.resourcesPath, 'openvpn', platformDir, binaryName);
            if (fs.existsSync(prodPath))
                return prodPath;
        }
        // macOS fallback: try Homebrew-installed OpenVPN
        if (platform === 'darwin') {
            const brewPaths = arch === 'arm64'
                ? ['/opt/homebrew/opt/openvpn/sbin/openvpn']
                : ['/usr/local/opt/openvpn/sbin/openvpn'];
            for (const bp of brewPaths) {
                if (fs.existsSync(bp))
                    return bp;
            }
        }
        // Fallback: system openvpn
        return 'openvpn';
    }
    async checkAvailable() {
        try {
            const bin = this.getBundledPath();
            // Ensure it's executable before testing (Linux/macOS)
            if (process.platform !== 'win32' && fs.existsSync(bin)) {
                try {
                    fs.chmodSync(bin, 0o755);
                }
                catch { /* ignore */ }
            }
            try {
                await new Promise((resolve, reject) => {
                    (0, child_process_1.exec)(`${bin} --version`, { timeout: 5000 }, (err) => {
                        if (err)
                            reject(err);
                        else
                            resolve();
                    });
                });
                return { available: true };
            }
            catch {
                // Try fallback to system openvpn
                if (bin !== 'openvpn') {
                    try {
                        await new Promise((resolve, reject) => {
                            (0, child_process_1.exec)('openvpn --version', { timeout: 5000 }, (err) => {
                                if (err)
                                    reject(err);
                                else
                                    resolve();
                            });
                        });
                        return { available: true };
                    }
                    catch { /* ignore */ }
                }
                return { available: false, error: 'OpenVPN 二进制不可用，请确认已安装 OpenVPN' };
            }
        }
        catch (e) {
            return { available: false, error: e.message };
        }
    }
    /**
     * Validate .ovpn config content for common issues
     */
    validateConfig(content) {
        if (!content || content.trim().length === 0) {
            return { valid: false, error: '配置文件内容为空' };
        }
        // Check for basic OpenVPN directives
        const hasClient = content.includes('client') || content.includes('pull');
        const hasDev = content.includes('dev ') || content.includes('dev-type');
        const hasProto = content.includes('proto ');
        const hasRemote = content.includes('remote ');
        if (!hasClient && !hasDev && !hasProto && !hasRemote) {
            return { valid: false, error: '配置文件格式无效，请检查是否为正确的 .ovpn 文件' };
        }
        if (!hasRemote) {
            return { valid: false, error: '配置文件缺少 remote 指令（服务器地址）' };
        }
        return { valid: true };
    }
    /**
     * Check if sudo requires a password for the current user.
     */
    async checkSudoNeedsPassword() {
        try {
            await new Promise((resolve, reject) => {
                (0, child_process_1.exec)('sudo -n true', { timeout: 3000 }, (err) => {
                    if (err)
                        reject(err);
                    else
                        resolve();
                });
            });
            return { needsPassword: false };
        }
        catch {
            return { needsPassword: true };
        }
    }
    /**
     * Cache sudo password for this session auto-reconnect.
     * Cleared on disconnect for security.
     */
    cacheSudoPassword(password) {
        this.cachedSudoPassword = password;
    }
    getStatus() {
        return { ...this.status, log: [...this.logBuffer] };
    }
    getTrafficStats() {
        if (!this.status.connected || this.status.bytesSent === undefined)
            return null;
        return {
            bytesSent: this.status.bytesSent || 0,
            bytesReceived: this.status.bytesReceived || 0,
            bytesSentHuman: this.humanBytes(this.status.bytesSent || 0),
            bytesReceivedHuman: this.humanBytes(this.status.bytesReceived || 0),
        };
    }
    humanBytes(bytes) {
        if (bytes < 1024)
            return bytes + ' B';
        if (bytes < 1024 * 1024)
            return (bytes / 1024).toFixed(1) + ' KB';
        if (bytes < 1024 * 1024 * 1024)
            return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
        return (bytes / (1024 * 1024 * 1024)).toFixed(2) + ' GB';
    }
    addLog(line) {
        this.logBuffer.push(line);
        if (this.logBuffer.length > this.MAX_LOG_LINES) {
            this.logBuffer = this.logBuffer.slice(-this.MAX_LOG_LINES);
        }
    }
    /**
     * Connect to OpenVPN management socket for real-time stats.
     */
    async connectManagement(socketPath) {
        return new Promise((resolve) => {
            // Wait a moment for the socket to be created
            setTimeout(() => {
                try {
                    this.mgmtSocket = net.createConnection({ path: socketPath });
                    this.mgmtSocketPath = socketPath;
                    this.mgmtSocket.on('data', (data) => {
                        const text = data.toString();
                        const lines = text.split('\n');
                        for (const line of lines) {
                            // Parse traffic stats: >INFO:BYTECOUNT:...,<bytes_in>,<bytes_out>,...
                            if (line.startsWith('>INFO:BYTECOUNT:')) {
                                const parts = line.split(':');
                                if (parts.length >= 5) {
                                    this.status.bytesReceived = parseInt(parts[2]) || 0;
                                    this.status.bytesSent = parseInt(parts[3]) || 0;
                                }
                            }
                        }
                    });
                    this.mgmtSocket.on('error', () => {
                        this.mgmtSocket = null;
                    });
                    // Request traffic updates every 5 seconds
                    this.mgmtSocket.on('connect', () => {
                        this.mgmtSocket?.write('hold release\n');
                        this.mgmtSocket?.write('bytecount 5\n');
                    });
                    resolve();
                }
                catch {
                    resolve(); // Don't fail connection if management socket fails
                }
            }, 1000);
        });
    }
    /**
     * Close management socket
     */
    closeManagement() {
        if (this.mgmtSocket) {
            this.mgmtSocket.end();
            this.mgmtSocket.destroy();
            this.mgmtSocket = null;
        }
        if (this.mgmtSocketPath && fs.existsSync(this.mgmtSocketPath)) {
            try {
                fs.unlinkSync(this.mgmtSocketPath);
            }
            catch { /* ignore */ }
        }
        this.mgmtSocketPath = null;
    }
    async connect(configId, configName, content, sudoPassword) {
        // Validate config
        const validation = this.validateConfig(content);
        if (!validation.valid) {
            return { success: false, error: validation.error };
        }
        // Auto-disconnect if already connected
        if (this.status.connected || this.process) {
            await this.disconnect();
            await new Promise(r => setTimeout(r, 500));
        }
        this.status = {
            connected: false,
            configId,
            configName,
            state: 'connecting',
            log: [],
            bytesSent: 0,
            bytesReceived: 0,
        };
        this.logBuffer = [];
        this.addLog(`正在连接 ${configName}...`);
        try {
            // Write config to temp file — use ~/.supertool/tmp/ for unified data management
            const tmpDir = path.join(os.homedir(), '.supertool', 'tmp');
            const timestamp = Date.now();
            const tmpFile = path.join(tmpDir, `supertool_${timestamp}.ovpn`);
            fs.writeFileSync(tmpFile, content, { encoding: 'utf-8', mode: 0o644 });
            if (!fs.existsSync(tmpFile)) {
                throw new Error(`临时配置文件写入失败: ${tmpFile}`);
            }
            this.tempConfigPath = tmpFile;
            this.addLog(`配置文件已写入: ${Buffer.byteLength(content, 'utf-8')} bytes`);
            const openvpnBin = this.getBundledPath();
            this.addLog(`使用 OpenVPN 二进制: ${openvpnBin}`);
            // Ensure the bundled binary is executable
            if (process.platform !== 'win32') {
                try {
                    fs.chmodSync(openvpnBin, 0o755);
                }
                catch { /* ignore */ }
            }
            const mgmtSocketPath = path.join(tmpDir, `supertool_mgmt_${timestamp}.sock`);
            const args = [
                '--config', tmpFile,
                // NO --daemon, NO --log: foreground process with stdout/stderr for connection detection
                '--management', mgmtSocketPath, 'unix',
                '--script-security', '2',
            ];
            const isMac = os.platform() === 'darwin';
            const isLinux = os.platform() === 'linux';
            const needsSudo = isMac || isLinux;
            let useSudoNoPassword = false;
            if (needsSudo) {
                const check = await this.checkSudoNeedsPassword();
                if (!check.needsPassword) {
                    useSudoNoPassword = true;
                }
                else if (!sudoPassword && this.cachedSudoPassword) {
                    // Use cached password from previous connection (cleared on disconnect)
                    sudoPassword = this.cachedSudoPassword;
                    this.addLog('使用缓存的 sudo 密码（上次连接）');
                }
            }
            let cmd;
            let spawnArgs;
            if (needsSudo) {
                if (sudoPassword) {
                    cmd = 'sudo';
                    spawnArgs = ['-S', openvpnBin, ...args];
                    this.addLog('使用 sudo -S 模式');
                    this.cacheSudoPassword(sudoPassword);
                }
                else if (useSudoNoPassword) {
                    cmd = 'sudo';
                    spawnArgs = ['-n', openvpnBin, ...args];
                    this.addLog('使用 sudo -n 模式（无需密码）');
                }
                else {
                    this.status.state = 'password_required';
                    this.pendingConnect = { configId, configName, content };
                    this.addLog('⚠️ 需要 sudo 密码才能启动 OpenVPN');
                    return { success: false, needsPassword: true, error: '需要 sudo 密码' };
                }
            }
            else {
                cmd = openvpnBin;
                spawnArgs = args;
            }
            this.addLog(`启动命令: ${cmd} ${spawnArgs.join(' ')}`);
            return this.spawnOpenVPN(cmd, spawnArgs, sudoPassword, mgmtSocketPath);
        }
        catch (e) {
            this.addLog(`连接失败: ${e.message}`);
            this.status.state = 'error';
            this.status.connected = false;
            this.cleanup();
            return { success: false, error: e.message };
        }
    }
    spawnOpenVPN(cmd, args, sudoPassword, mgmtSocketPath) {
        return new Promise((resolve) => {
            // 移除 --log 参数，让我们直接捕获 stdout/stderr（--log 会将所有输出重定向到文件，
            // 导致 spawn 的 stdout/stderr 永远为空，handleOutput 收不到任何数据）
            const logArgIdx = args.indexOf('--log');
            const spawnArgs = [...args];
            if (logArgIdx >= 0) {
                spawnArgs.splice(logArgIdx, 2); // remove --log and its value
            }
            this.process = (0, child_process_1.spawn)(cmd, spawnArgs, {
                stdio: ['pipe', 'pipe', 'pipe'],
            });
            if (!this.process.pid) {
                this.cleanup();
                this.closeManagement();
                resolve({ success: false, error: '无法启动 OpenVPN 进程' });
                return;
            }
            this.addLog(`OpenVPN 进程 PID: ${this.process.pid}`);
            // Listen for sudo prompt before sending password (more reliable than immediate write)
            let passwordSent = false;
            const sendSudoPassword = () => {
                if (!passwordSent && sudoPassword && this.process?.stdin?.writable) {
                    this.process.stdin.write(sudoPassword + '\n');
                    this.addLog('已发送 sudo 密码');
                    passwordSent = true;
                }
            };
            // Send immediately as fallback, but also on prompt detection
            if (sudoPassword) {
                setTimeout(sendSudoPassword, 50); // slight delay to let sudo initialize stdin
            }
            let resolved = false;
            let connectTimeout;
            const doResolve = (result) => {
                if (resolved)
                    return;
                resolved = true;
                clearTimeout(connectTimeout);
                resolve(result);
            };
            // Connect to management socket for traffic stats
            if (mgmtSocketPath) {
                this.connectManagement(mgmtSocketPath).catch(() => { });
            }
            const handleOutput = (data) => {
                const text = data.toString();
                const lines = text.split('\n');
                for (const line of lines) {
                    const trimmed = line.trim();
                    if (!trimmed)
                        continue;
                    this.addLog(trimmed);
                    // sudo password prompt — send password when we see it
                    if (trimmed.toLowerCase().includes('password') || trimmed.toLowerCase().includes('authenticate')) {
                        sendSudoPassword();
                    }
                    // Wrong sudo password
                    if (trimmed.includes('sudo: ') && (trimmed.toLowerCase().includes('incorrect') || trimmed.toLowerCase().includes('authentication failed'))) {
                        this.addLog('❌ sudo 密码错误');
                        this.status.state = 'error';
                        this.status.connected = false;
                        doResolve({ success: false, error: 'sudo 密码错误' });
                        return;
                    }
                    // Connection success
                    if (trimmed.includes('Initialization Sequence Completed')) {
                        this.status.state = 'connected';
                        this.status.connected = true;
                        this.status.connectedSince = new Date().toISOString();
                        this.addLog('✅ 连接成功');
                        doResolve({ success: true });
                    }
                    // Connection errors
                    if (trimmed.includes('AUTH_FAILED') || trimmed.includes('TLS Error')) {
                        this.status.state = 'error';
                        this.addLog(`❌ ${trimmed}`);
                        doResolve({ success: false, error: trimmed });
                    }
                    // Config errors — only treat as fatal if not a server-push option warning
                    // (redirect-gateway, dhcp-option, block-outside-dns are Windows-only, safe to ignore on Linux)
                    if (trimmed.includes('Options error:')) {
                        const isPushOptionWarning = trimmed.includes('[PUSH-OPTIONS]') ||
                            trimmed.includes('redirect-gateway') ||
                            trimmed.includes('dhcp-option') ||
                            trimmed.includes('block-outside-dns');
                        if (isPushOptionWarning) {
                            this.addLog(`⚠️ ${trimmed}（服务器推送选项，可忽略）`);
                        }
                        else {
                            this.status.state = 'error';
                            this.addLog(`❌ ${trimmed}`);
                            doResolve({ success: false, error: trimmed });
                        }
                    }
                    // Detect remote server
                    const remoteMatch = trimmed.match(/(UDP|TCP)(v4|v6)? link (local|remote):/);
                    if (remoteMatch && !this.status.remote) {
                        const remoteInfo = trimmed.split('>').pop()?.trim();
                        if (remoteInfo)
                            this.status.remote = remoteInfo;
                    }
                }
            };
            this.process.stdout?.on('data', handleOutput);
            this.process.stderr?.on('data', handleOutput);
            this.process.on('error', (err) => {
                this.addLog(`进程错误: ${err.message}`);
                this.status.state = 'error';
                this.status.connected = false;
                doResolve({ success: false, error: err.message });
            });
            this.process.on('exit', (code, signal) => {
                this.addLog(`进程退出，代码: ${code}${signal ? `, 信号: ${signal}` : ''}`);
                // If still connecting and process died — report error
                if (this.status.state === 'connecting' && (code !== 0 || signal)) {
                    this.status.state = 'error';
                    this.status.connected = false;
                    const errMsg = `OpenVPN 异常退出 (code: ${code}, signal: ${signal || 'none'})`;
                    doResolve({ success: false, error: errMsg });
                    this.process = null;
                    return;
                }
                if (this.status.state === 'connected') {
                    this.process = null;
                    return;
                }
                this.process = null;
            });
            // Wait up to 30s for connection
            connectTimeout = setTimeout(() => {
                if (this.status.state === 'connecting') {
                    this.status.state = 'error';
                    this.status.connected = false;
                    this.addLog('⏳ 连接超时（30秒未收到服务器响应）');
                    doResolve({ success: false, error: '连接超时' });
                }
            }, 30000);
        });
    }
    /**
     * Retry connection with a password
     */
    async retryWithPassword(password) {
        if (!this.pendingConnect) {
            return { success: false, error: '没有待重试的连接' };
        }
        const { configId, configName, content } = this.pendingConnect;
        this.pendingConnect = null;
        this.cleanup();
        return this.connect(configId, configName, content, password);
    }
    async disconnect() {
        if (!this.process && !this.status.connected && this.status.state !== 'password_required') {
            this.status.state = 'disconnected';
            return { success: true };
        }
        this.status.state = 'disconnecting';
        this.addLog('正在断开连接...');
        try {
            if (this.process && this.process.pid) {
                try {
                    process.kill(this.process.pid, 'SIGTERM');
                    this.addLog(`已发送 SIGTERM 到进程 ${this.process.pid}`);
                }
                catch {
                    // Already dead
                }
                // Wait for process to actually exit before returning
                await new Promise((resolve) => {
                    const forceKill = setTimeout(() => {
                        try {
                            if (this.process?.pid) {
                                process.kill(this.process.pid, 'SIGKILL');
                                this.addLog(`已发送 SIGKILL 到进程 ${this.process.pid}`);
                            }
                        }
                        catch { /* ignore */ }
                        resolve();
                    }, 3000);
                    this.process.on('exit', () => {
                        clearTimeout(forceKill);
                        resolve();
                    });
                    // Already exited
                    if (!this.process.pid) {
                        clearTimeout(forceKill);
                        resolve();
                    }
                });
            }
            this.process = null;
            this.pendingConnect = null;
            this.status.state = 'disconnected';
            this.status.connected = false;
            this.status.configId = null;
            this.status.configName = null;
            this.status.bytesSent = 0;
            this.status.bytesReceived = 0;
            this.cachedSudoPassword = null;
            this.logBuffer = [];
            this.addLog('✅ 已断开连接');
            this.cleanup();
            this.closeManagement();
            return { success: true };
        }
        catch (e) {
            this.addLog(`断开失败: ${e.message}`);
            this.cleanup();
            this.closeManagement();
            return { success: false, error: e.message };
        }
    }
    cleanup() {
        if (this.tempConfigPath) {
            try {
                fs.unlinkSync(this.tempConfigPath);
            }
            catch { /* ignore */ }
            this.tempConfigPath = null;
        }
        // Also clean up log file and management socket
        const tmpDir = path.join(os.homedir(), '.supertool', 'tmp');
        try {
            const files = fs.readdirSync(tmpDir);
            for (const file of files) {
                if (file.startsWith('supertool_') && (file.endsWith('.log') || file.endsWith('.sock'))) {
                    fs.unlinkSync(path.join(tmpDir, file));
                }
            }
        }
        catch { /* ignore */ }
    }
}
exports.openVPNManager = new OpenVPNManager();
//# sourceMappingURL=openvpn-manager.js.map