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
exports.getLoadedShellEnv = getLoadedShellEnv;
exports.isShellEnvLoaded = isShellEnvLoaded;
exports.loadUserShellEnv = loadUserShellEnv;
const async_exec_1 = require("./async-exec");
const logger_1 = require("./logger");
const os = __importStar(require("os"));
const path = __importStar(require("path"));
// ============ 启动时加载用户 Shell 环境变量 ============
// 从用户 Shell 配置文件中加载环境变量，注入 process.env
// macOS: login shell 会加载 ~/.zprofile → ~/.zshrc（或 ~/.bash_profile → ~/.bashrc）
// Linux: login shell 会加载 ~/.profile 或 ~/.bashrc 或 ~/.zshrc
// 这样 Electron 启动的子进程（终端、构建工具等）能继承用户配置的环境变量
// 不覆盖的关键环境变量（Electron/OS 已正确设置）
const PROTECTED_ENV_KEYS = new Set(['HOME', 'USER', 'LOGNAME', 'SHELL', 'TERM', 'TMPDIR', 'LANG', 'LC_ALL', 'DISPLAY', 'XDG_SESSION_TYPE', 'XDG_CURRENT_DESKTOP', 'GNOME_DESKTOP_SESSION_ID']);
// 已加载标记，避免重复执行
let shellEnvLoaded = false;
const loadedShellEnv = {};
function getLoadedShellEnv() {
    return loadedShellEnv;
}
function isShellEnvLoaded() {
    return shellEnvLoaded;
}
function getShellRcInfo(shellCmd) {
    const home = os.homedir();
    if (shellCmd === 'zsh') {
        const files = ['.zshrc', '.zprofile', '.zshenv']
            .filter(f => require('fs').existsSync(require('path').join(home, f)))
            .join(', ');
        return files || '~/.zshrc (default)';
    }
    const files = ['.bash_profile', '.bashrc', '.profile']
        .filter(f => require('fs').existsSync(require('path').join(home, f)))
        .join(', ');
    return files || '~/.bashrc (default)';
}
function parseShellRcFiles(home, shellCmd) {
    const { readFileSync } = require('fs');
    const rcFiles = shellCmd === 'zsh'
        ? ['.zshrc', '.zprofile', '.zshenv', '.profile']
        : ['.bash_profile', '.bashrc', '.profile'];
    const exportRegex = /(?:^|\n)\s*export\s+([A-Za-z_][A-Za-z0-9_]*)=(?:"([^"]*)"|'([^']*)'|(\S+))/g;
    const parsedVars = {};
    for (const rcFile of rcFiles) {
        const filePath = require('path').join(home, rcFile);
        try {
            const content = readFileSync(filePath, 'utf-8');
            let match;
            while ((match = exportRegex.exec(content)) !== null) {
                const key = match[1];
                let val = match[2] ?? match[3] ?? match[4] ?? '';
                if (!val)
                    continue;
                if (PROTECTED_ENV_KEYS.has(key))
                    continue;
                if (process.env[key] !== val) {
                    parsedVars[key] = val;
                    process.env[key] = val;
                }
            }
        }
        catch {
            // 文件不存在，跳过
        }
    }
    if (Object.keys(parsedVars).length > 0) {
        Object.assign(loadedShellEnv, parsedVars);
        (0, logger_1.info)(`[ShellEnv] Parsed ${Object.keys(parsedVars).length} env vars from ${shellCmd} rc files (fallback)`);
        (0, logger_1.info)('[ShellEnv] Keys:', Object.keys(parsedVars).slice(0, 15).join(', '));
    }
    else {
        (0, logger_1.info)('[ShellEnv] No env vars found in shell rc files (fallback)');
    }
}
async function loadUserShellEnv() {
    if (shellEnvLoaded)
        return loadedShellEnv;
    shellEnvLoaded = true;
    const platform = process.platform;
    if (platform === 'win32') {
        (0, logger_1.info)('[ShellEnv] Windows detected, skipping shell env loading');
        return loadedShellEnv;
    }
    let shellCmd = platform === 'darwin' ? 'zsh' : 'bash';
    try {
        const userShell = process.env.SHELL || '';
        if (userShell.includes('zsh'))
            shellCmd = 'zsh';
        else if (userShell.includes('bash'))
            shellCmd = 'bash';
        else if (userShell.includes('fish'))
            shellCmd = 'fish';
    }
    catch { /* ignore */ }
    try {
        const envDump = await (0, async_exec_1.runCommand)(`${shellCmd} -l -i -c 'env'`, {
            timeout: 10000,
            env: { ...process.env, BASH_SILENCE_DEPRECATION_WARNING: '1' },
        });
        const vars = {};
        for (const line of envDump.stdout.split('\n')) {
            const eqIdx = line.indexOf('=');
            if (eqIdx <= 0)
                continue;
            const key = line.substring(0, eqIdx);
            const val = line.substring(eqIdx + 1);
            if (!key || !val)
                continue;
            if (PROTECTED_ENV_KEYS.has(key))
                continue;
            if (process.env[key] !== val) {
                vars[key] = val;
            }
        }
        for (const [key, val] of Object.entries(vars)) {
            if (key === 'PATH') {
                const currentPaths = (process.env.PATH || '').split(path.delimiter);
                const shellPaths = val.split(path.delimiter);
                const existing = new Set(currentPaths);
                const extraPaths = shellPaths.filter((p) => p && !existing.has(p));
                if (extraPaths.length > 0) {
                    process.env.PATH = [...currentPaths, ...extraPaths].join(path.delimiter);
                    (0, logger_1.info)(`[ShellEnv] PATH extended: +${extraPaths.length} dirs (${extraPaths.slice(0, 3).join(', ')}${extraPaths.length > 3 ? '...' : ''})`);
                }
            }
            else {
                process.env[key] = val;
            }
        }
        Object.assign(loadedShellEnv, vars);
        const rcInfo = getShellRcInfo(shellCmd);
        (0, logger_1.info)(`[ShellEnv] Loaded ${Object.keys(vars).length} env vars via ${shellCmd} -l -i (${rcInfo})`);
        if (Object.keys(vars).length > 0) {
            (0, logger_1.info)('[ShellEnv] Keys:', Object.keys(vars).slice(0, 20).join(', ') + (Object.keys(vars).length > 20 ? ' ...' : ''));
        }
    }
    catch (e) {
        console.warn(`[ShellEnv] Login shell failed (${e.message}), falling back to file parsing`);
        const home = os.homedir();
        parseShellRcFiles(home, shellCmd);
    }
    return loadedShellEnv;
}
//# sourceMappingURL=shell-env-manager.js.map