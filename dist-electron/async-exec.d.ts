/**
 * 异步子进程执行工具
 * 替代 execSync/spawnSync，避免阻塞 Electron 主进程
 */
import { ExecOptions } from 'child_process';
export interface ExecResult {
    stdout: string;
    stderr: string;
}
/**
 * 异步执行 shell 命令（不阻塞 Electron 主进程）
 */
export declare function runCommand(command: string, options?: ExecOptions): Promise<ExecResult>;
/**
 * 异步执行命令，失败时返回 null（不抛异常）
 */
export declare function tryCommand(command: string, options?: ExecOptions): Promise<ExecResult | null>;
/**
 * 获取命令的绝对路径
 */
export declare function which(cmd: string): Promise<string | null>;
