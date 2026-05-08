"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.runCommand = runCommand;
exports.tryCommand = tryCommand;
exports.which = which;
/**
 * 异步子进程执行工具
 * 替代 execSync/spawnSync，避免阻塞 Electron 主进程
 */
const child_process_1 = require("child_process");
const util_1 = require("util");
const execAsync = (0, util_1.promisify)(child_process_1.exec);
/**
 * 异步执行 shell 命令（不阻塞 Electron 主进程）
 */
async function runCommand(command, options) {
    const { stdout, stderr } = await execAsync(command, {
        encoding: 'utf-8',
        ...options,
    });
    return { stdout: stdout, stderr: stderr };
}
/**
 * 异步执行命令，失败时返回 null（不抛异常）
 */
async function tryCommand(command, options) {
    try {
        return await runCommand(command, options);
    }
    catch {
        return null;
    }
}
/**
 * 获取命令的绝对路径
 */
async function which(cmd) {
    const result = await tryCommand(`which ${cmd}`, { timeout: 2000 });
    return result?.stdout.trim() || null;
}
//# sourceMappingURL=async-exec.js.map