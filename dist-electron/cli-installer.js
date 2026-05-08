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
exports.installStoolCliBinary = installStoolCliBinary;
exports.installHermesSkills = installHermesSkills;
const logger_1 = require("./logger");
const async_exec_1 = require("./async-exec");
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const os = __importStar(require("os"));
// 打包后 stool 二进制在 Contents/Resources/stool（extraResources 注入）
// DMG 没有 postinstall 脚本，所以在 App 首次启动时自动安装
async function installStoolCliBinary() {
    if (process.platform !== 'darwin')
        return;
    const stoolTarget = '/usr/local/bin/stool';
    const stoolSource = process.resourcesPath
        ? path.join(process.resourcesPath, 'stool')
        : null;
    // 1. 检查源文件是否存在
    if (!stoolSource || !fs.existsSync(stoolSource)) {
        (0, logger_1.info)('[CLI] stool binary not found, skipping install');
        return;
    }
    // 2. 比较文件修改时间：源文件比目标文件新才更新（与 skills 分发逻辑一致）
    if (fs.existsSync(stoolTarget)) {
        try {
            const sourceMtime = fs.statSync(stoolSource).mtimeMs;
            const targetMtime = fs.statSync(stoolTarget).mtimeMs;
            if (sourceMtime <= targetMtime) {
                (0, logger_1.info)('[CLI] stool binary is up-to-date (mtime check), skipping');
                return;
            }
            (0, logger_1.info)('[CLI] stool binary mtime differs, will update');
        }
        catch (e) {
            (0, logger_1.info)(`[CLI] Stat check failed: ${e instanceof Error ? e.message : String(e)}`);
        }
    }
    // 3. 检查是否需要 sudo 密码
    let needsPassword = false;
    try {
        const result = await (0, async_exec_1.runCommand)('sudo -n true 2>&1 || echo needs_password', { timeout: 3000 });
        needsPassword = result.stdout.includes('needs_password');
    }
    catch {
        needsPassword = true;
    }
    if (needsPassword) {
        // 使用 AppleScript 弹出密码输入框（支持文本输入）
        const tmpScript = path.join(os.tmpdir(), `stool-pwd-${Date.now()}.scpt`);
        const appleScript = `display dialog "安装 SuperTool CLI 到 /usr/local/bin 需要管理员权限。" default answer "" with title "输入管理员密码" with hidden answer with icon caution`;
        fs.writeFileSync(tmpScript, appleScript);
        try {
            const output = await (0, async_exec_1.runCommand)(`/usr/bin/osascript "${tmpScript}" 2>/dev/null || echo "cancel"`, { timeout: 60000 });
            fs.unlinkSync(tmpScript);
            if (output.stdout.trim() === 'cancel') {
                (0, logger_1.info)('[CLI] User cancelled password prompt');
                return; // Fall through to fallback
            }
            // 解析 AppleScript 输出: "button returned:OK, text returned:密码"
            const pwdMatch = output.stdout.match(/text returned:(.*)/);
            if (!pwdMatch) {
                (0, logger_1.info)('[CLI] Failed to parse password from AppleScript output');
                return; // Fall through
            }
            const sudoPassword = pwdMatch[1].trim();
            // 使用 sudo -S 执行安装
            (0, logger_1.info)('[CLI] Attempting sudo -S install with provided password');
            const safeSrc = stoolSource.replace(/'/g, "'\\''");
            const safeDst = stoolTarget.replace(/'/g, "'\\''");
            const { spawn } = require('child_process');
            const sudoProc = spawn('sudo', ['-S', 'cp', safeSrc, safeDst], { stdio: ['pipe', 'pipe', 'pipe'] });
            // 监听 sudo 密码提示
            let passwordSent = false;
            const sendPwd = () => {
                if (!passwordSent && sudoProc.stdin.writable) {
                    sudoProc.stdin.write(sudoPassword + '\n');
                    passwordSent = true;
                }
            };
            setTimeout(sendPwd, 100); // Send password immediately (or wait for prompt)
            sudoProc.stdout.on('data', (data) => {
                const txt = data.toString();
                (0, logger_1.info)(`[CLI] sudo stdout: ${txt}`);
                if (txt.toLowerCase().includes('password'))
                    sendPwd();
            });
            sudoProc.stderr.on('data', (data) => {
                const txt = data.toString();
                (0, logger_1.info)(`[CLI] sudo stderr: ${txt}`);
                if (txt.toLowerCase().includes('password'))
                    sendPwd();
            });
            await new Promise((resolve, reject) => {
                sudoProc.on('exit', async (code) => {
                    if (code === 0) {
                        await (0, async_exec_1.runCommand)(`chmod +x '${safeDst}'`, { timeout: 5000 });
                        (0, logger_1.info)(`[CLI] stool installed to ${stoolTarget}`);
                        resolve();
                    }
                    else {
                        reject(new Error(`sudo cp failed with code ${code}`));
                    }
                });
                sudoProc.on('error', reject);
            });
            return;
        }
        catch (e) {
            (0, logger_1.info)(`[CLI] AppleScript password install failed: ${e.message}`);
        }
    }
    else {
        // 不需要密码，直接 sudo 拷贝
        try {
            const safeSrc = stoolSource.replace(/'/g, "'\\''");
            const safeDst = stoolTarget.replace(/'/g, "'\\''");
            await (0, async_exec_1.runCommand)(`sudo cp '${safeSrc}' '${safeDst}' && chmod +x '${safeDst}'`, { timeout: 10000 });
            (0, logger_1.info)(`[CLI] stool installed to ${stoolTarget}`);
            return;
        }
        catch (e) {
            (0, logger_1.info)(`[CLI] Direct sudo install failed: ${e.message}`);
        }
    }
    // 4. 降级安装到 ~/.local/bin
    try {
        const fallback = path.join(os.homedir(), '.local', 'bin', 'stool');
        fs.mkdirSync(path.dirname(fallback), { recursive: true });
        fs.copyFileSync(stoolSource, fallback);
        fs.chmodSync(fallback, 0o755);
        (0, logger_1.info)(`[CLI] stool installed to fallback: ${fallback}`);
    }
    catch (e2) {
        console.error('[CLI] Fallback install failed:', e2.message);
    }
}
// ============ 自动分发 stool-cli 技能到 Hermes 技能目录 ============
// 每次启动时检查源文件 mtime，如果比目标文件新就覆盖（无需授权）
// 技能源文件打包在 ASAR 内的 skills/stool-cli/SKILL.md + extraResources
function getSkillSourcePath() {
    // 开发模式: 项目根目录的 skills/
    const devPath = path.join(__dirname, '..', 'skills', 'stool-cli', 'SKILL.md');
    if (fs.existsSync(devPath))
        return devPath;
    // 生产模式: extraResources（未打包，在 Contents/Resources/skills/）
    if (process.resourcesPath) {
        const resPath = path.join(process.resourcesPath, 'skills', 'stool-cli', 'SKILL.md');
        if (fs.existsSync(resPath))
            return resPath;
    }
    // 回退: ASAR 内部
    const asarPath = path.join(__dirname, '..', '..', 'skills', 'stool-cli', 'SKILL.md');
    if (fs.existsSync(asarPath))
        return asarPath;
    return devPath;
}
async function installHermesSkills() {
    const sourcePath = getSkillSourcePath();
    if (!fs.existsSync(sourcePath)) {
        console.warn('[Skills] stool-cli SKILL.md not found at:', sourcePath);
        return;
    }
    const skillsDir = path.join(os.homedir(), '.hermes', 'skills', 'stool-cli');
    const skillFile = path.join(skillsDir, 'SKILL.md');
    try {
        // 比较文件修改时间：源文件比目标文件新才更新
        if (fs.existsSync(skillFile)) {
            const sourceMtime = fs.statSync(sourcePath).mtimeMs;
            const targetMtime = fs.statSync(skillFile).mtimeMs;
            if (sourceMtime <= targetMtime) {
                (0, logger_1.info)('[Skills] stool-cli skill is up-to-date (mtime check)');
                return;
            }
        }
        fs.mkdirSync(skillsDir, { recursive: true });
        const source = fs.readFileSync(sourcePath, 'utf-8');
        fs.writeFileSync(skillFile, source, { mode: 0o644 });
        // 保持源文件的修改时间，方便下次 mtime 比对
        const sourceStat = fs.statSync(sourcePath);
        fs.utimesSync(skillFile, sourceStat.atime, sourceStat.mtime);
        (0, logger_1.info)(`[Skills] stool-cli skill updated: ${sourcePath} → ${skillFile}`);
    }
    catch (e) {
        console.error('[Skills] Failed to install stool-cli skill:', e.message);
    }
}
//# sourceMappingURL=cli-installer.js.map