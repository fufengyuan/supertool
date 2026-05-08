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
const async_exec_1 = require("../async-exec");
const simple_git_1 = require("simple-git");
const ssh2_1 = require("ssh2");
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const os = __importStar(require("os"));
const EventEmitter = require("events");
// SECURITY: Shell escape function to prevent command injection in sshExec
function shellEscape(s) {
    if (!s)
        return "''";
    return "'" + s.replace(/'/g, "'\\''") + "'";
}
/**
 * 远程路径解析：将 ~/ 开头的路径替换为实际 home 目录
 * 注意：必须在传入 shellEscape 之前完成替换，否则单引号内 $HOME 不展开
 */
function resolveRemotePath(p, homeDir) {
    if (!p || !homeDir)
        return p;
    if (p === '~')
        return homeDir;
    if (p.startsWith('~/'))
        return homeDir + p.slice(1);
    return p;
}
/**
 * SFTP 路径解析（与 resolveRemotePath 相同语义）
 */
function resolveSftpPath(p, homeDir) {
    return resolveRemotePath(p, homeDir);
}
// Safely extract error message from unknown type
function getErrorMessage(error) {
    return error instanceof Error ? error.message : String(error);
}
// Detect Java home directory — ONLY used for whereis fallback in config UI, not during deployment
function detectJavaHome(configured) {
    // Configured path takes absolute priority
    if (configured && fs.existsSync(configured))
        return configured;
    return undefined;
}
// Detect Maven home directory — ONLY used for whereis fallback in config UI, not during deployment
function detectMavenHome(configured) {
    if (configured && fs.existsSync(configured))
        return configured;
    return undefined;
}
// Detect Node.js home directory — ONLY used for whereis fallback in config UI, not during deployment
function detectNodeHome(configured) {
    if (configured && fs.existsSync(configured))
        return configured;
    return undefined;
}
// Resolve mvn executable path — ONLY use configured mavenHome, never fallback
function resolveMvnPath(mavenHome) {
    if (mavenHome && fs.existsSync(path.join(mavenHome, 'bin', 'mvn'))) {
        return path.join(mavenHome, 'bin', 'mvn');
    }
    // If mavenHome is set but invalid, return it anyway (will fail with clear error)
    if (mavenHome)
        return mavenHome;
    return 'mvn'; // Last resort: rely on system PATH
}
class CicdService extends EventEmitter {
    constructor(userDataPath) {
        super();
        this.deployProgress = new Map();
        // Use ~/.supertool/cicd-workspace for unified data management
        const baseDir = userDataPath || path.join(os.homedir(), '.supertool');
        this.workDir = path.join(baseDir, 'cicd-workspace');
        if (!fs.existsSync(this.workDir)) {
            fs.mkdirSync(this.workDir, { recursive: true });
        }
    }
    // Git 同步 / 本地仓库检测
    async gitSync(repoUrl, branch, localPath) {
        const isLocal = !repoUrl.startsWith('http') && !repoUrl.startsWith('git@') && fs.existsSync(repoUrl);
        if (isLocal) {
            this.emit('progress', { stage: 'git', status: 'success', message: `使用本地仓库 ${repoUrl}` });
            return { success: true, path: repoUrl };
        }
        const git = (0, simple_git_1.simpleGit)(localPath || this.workDir);
        const repoName = this.getRepoName(repoUrl);
        const targetPath = path.join(localPath || this.workDir, repoName);
        this.emit('progress', { stage: 'git', status: 'starting', message: `开始同步 ${repoUrl}` });
        try {
            if (fs.existsSync(targetPath)) {
                this.emit('progress', { stage: 'git', status: 'pulling', message: `拉取最新代码` });
                const repoGit = (0, simple_git_1.simpleGit)(targetPath);
                await repoGit.fetch('origin');
                await repoGit.checkout(branch);
                await repoGit.pull('origin', branch);
                this.emit('progress', { stage: 'git', status: 'success', message: `代码已更新` });
            }
            else {
                this.emit('progress', { stage: 'git', status: 'cloning', message: `克隆仓库` });
                await git.clone(repoUrl, targetPath, ['-b', branch]);
                this.emit('progress', { stage: 'git', status: 'success', message: `仓库已克隆` });
            }
            return { success: true, path: targetPath };
        }
        catch (error) {
            this.emit('progress', { stage: 'git', status: 'error', message: getErrorMessage(error) });
            throw error;
        }
    }
    // Maven 构建
    async mavenBuild(projectPath, mavenHome, modules = [], skipTests = true, javaHome) {
        const resolvedMvn = resolveMvnPath(mavenHome);
        const args = ['clean', 'package'];
        if (skipTests) {
            args.push('-DskipTests');
        }
        if (modules.length > 0) {
            args.push('-pl');
            args.push(modules.join(','));
            args.push('-am');
        }
        this.emit('progress', { stage: 'maven', status: 'starting', message: `开始 Maven 构建` });
        return new Promise((resolve, reject) => {
            const { spawn } = require('child_process');
            // ONLY use configured paths — no auto-detection during deployment
            const extraPaths = [
                javaHome ? path.join(javaHome, 'bin') : null,
                mavenHome ? path.join(mavenHome, 'bin') : null,
                '/usr/local/bin',
                '/opt/homebrew/bin',
                '/usr/bin',
                '/bin',
                '/usr/sbin',
                '/sbin',
            ].filter(Boolean);
            const existingPath = process.env.PATH || '';
            const newPath = [...extraPaths, ...existingPath.split(':').filter(Boolean)].join(':');
            const env = {
                ...process.env,
                PATH: newPath,
            };
            if (javaHome)
                env.JAVA_HOME = javaHome;
            if (mavenHome)
                env.MAVEN_HOME = mavenHome;
            const proc = spawn(resolvedMvn, args, {
                cwd: projectPath,
                env
            });
            let output = '';
            proc.stdout.on('data', (data) => {
                output += data.toString();
                this.emit('progress', { stage: 'maven', status: 'building', message: data.toString().slice(0, 200) });
            });
            proc.stderr.on('data', (data) => {
                output += data.toString();
            });
            proc.on('error', (err) => {
                this.emit('progress', { stage: 'maven', status: 'error', message: `Maven 构建启动失败: ${err.message}` });
                reject(new Error(`Maven build failed to start: ${err.message}`));
            });
            proc.on('close', (code) => {
                if (code === 0) {
                    this.emit('progress', { stage: 'maven', status: 'success', message: '构建成功' });
                    resolve({ success: true, output });
                }
                else {
                    const lastLines = output.split('\n').slice(-10).join('\n').trim();
                    this.emit('progress', { stage: 'maven', status: 'error', message: `构建失败 (exit ${code})` });
                    reject(new Error(`Maven build failed with code ${code}\n\n最近 10 行输出:\n${lastLines || '(无输出)'}`));
                }
            });
        });
    }
    // NPM/PNPM/Yarn 构建
    async npmBuild(projectPath, tool = 'npm', script = 'build', npmHome, nodeHome) {
        // Resolve command path: if npmHome is a directory, append bin/{tool}
        // If it's already a full path to the executable, use it directly
        let cmd;
        if (npmHome) {
            const withBin = path.join(npmHome, 'bin', tool);
            cmd = fs.existsSync(npmHome) && fs.statSync(npmHome).isDirectory() && fs.existsSync(withBin)
                ? withBin
                : npmHome; // Assume it's already a full path
        }
        else {
            cmd = tool; // Rely on system PATH
        }
        const args = ['run', script];
        this.emit('progress', { stage: 'npm', status: 'starting', message: `开始 ${tool} ${script} 构建` });
        return new Promise((resolve, reject) => {
            const { spawn } = require('child_process');
            // ONLY use configured paths — no auto-detection during deployment
            const extraPaths = [
                nodeHome ? path.join(nodeHome, 'bin') : null,
                npmHome && fs.existsSync(npmHome) && fs.statSync(npmHome).isDirectory() ? path.join(npmHome, 'bin') : null,
                '/usr/local/bin',
                '/opt/homebrew/bin',
                '/usr/bin',
                '/bin',
                '/usr/sbin',
                '/sbin',
            ].filter(Boolean);
            const existingPath = process.env.PATH || '';
            const newPath = [...extraPaths, ...existingPath.split(':').filter(Boolean)].join(':');
            const env = {
                ...process.env,
                PATH: newPath,
            };
            const proc = spawn(cmd, args, { cwd: projectPath, env });
            let output = '';
            proc.stdout.on('data', (data) => {
                output += data.toString();
                this.emit('progress', { stage: 'npm', status: 'building', message: data.toString().slice(0, 200) });
            });
            proc.stderr.on('data', (data) => {
                output += data.toString();
            });
            proc.on('error', (err) => {
                this.emit('progress', { stage: 'npm', status: 'error', message: `${tool} 构建启动失败: ${err.message}` });
                reject(new Error(`${tool} build failed to start: ${err.message}`));
            });
            proc.on('close', (code) => {
                if (code === 0) {
                    this.emit('progress', { stage: 'npm', status: 'success', message: `${tool} 构建成功` });
                    resolve({ success: true, output });
                }
                else {
                    const lastLines = output.split('\n').slice(-10).join('\n').trim();
                    this.emit('progress', { stage: 'npm', status: 'error', message: `${tool} 构建失败 (exit ${code})` });
                    reject(new Error(`${tool} build failed with code ${code}\n\n最近 10 行输出:\n${lastLines || '(无输出)'}`));
                }
            });
        });
    }
    // Gradle 构建
    async gradleBuild(projectPath, modules = [], globalConfig) {
        const args = ['clean', 'build'];
        if (modules.length > 0) {
            for (const mod of modules) {
                args.push(`:${mod}:build`);
            }
        }
        // Resolve gradle executable: try wrapper first, fall back to system gradle
        const gradleWrapper = process.platform === 'win32' ? 'gradlew.bat' : './gradlew';
        const resolvedGradle = fs.existsSync(path.join(projectPath, process.platform === 'win32' ? 'gradlew.bat' : 'gradlew'))
            ? gradleWrapper
            : 'gradle';
        this.emit('progress', { stage: 'gradle', status: 'starting', message: `开始 Gradle 构建` });
        return new Promise((resolve, reject) => {
            const { spawn } = require('child_process');
            // Inject env for Gradle (same pattern as Maven/NPM)
            const envPaths = [
                globalConfig?.javaHome ? path.join(globalConfig.javaHome, 'bin') : null,
                '/usr/local/bin', '/opt/homebrew/bin', '/usr/bin', '/bin',
            ].filter(Boolean);
            const existingPath = process.env.PATH || '';
            const env = {
                ...Object.fromEntries(Object.entries(process.env).filter(([, v]) => v !== undefined)),
                PATH: [...envPaths, ...existingPath.split(':').filter(Boolean)].join(':'),
            };
            if (globalConfig?.javaHome)
                env.JAVA_HOME = globalConfig.javaHome;
            const proc = spawn(resolvedGradle, args, { cwd: projectPath, env });
            let output = '';
            proc.stdout.on('data', (data) => {
                output += data.toString();
                this.emit('progress', { stage: 'gradle', status: 'building', message: data.toString().slice(0, 200) });
            });
            proc.stderr.on('data', (data) => {
                output += data.toString();
            });
            proc.on('error', (err) => {
                this.emit('progress', { stage: 'gradle', status: 'error', message: `Gradle 构建启动失败: ${err.message}` });
                reject(new Error(`Gradle build failed to start: ${err.message}`));
            });
            proc.on('close', (code) => {
                if (code === 0) {
                    this.emit('progress', { stage: 'gradle', status: 'success', message: `Gradle 构建成功` });
                    resolve({ success: true, output });
                }
                else {
                    const lastLines = output.split('\n').slice(-10).join('\n').trim();
                    this.emit('progress', { stage: 'gradle', status: 'error', message: `Gradle 构建失败 (exit ${code})` });
                    reject(new Error(`Gradle build failed with code ${code}\n\n最近 10 行输出:\n${lastLines || '(无输出)'}`));
                }
            });
        });
    }
    // 单个模块构建（支持子目录、自定义命令）
    async buildModule(projectPath, module, globalConfig) {
        const buildPath = module.buildPath
            ? path.join(projectPath, module.buildPath)
            : module.path
                ? path.join(projectPath, module.path)
                : projectPath;
        // 如果模块有自定义构建命令，直接执行
        if (module.buildCommand || globalConfig.buildCommand) {
            const cmd = module.buildCommand || globalConfig.buildCommand;
            this.emit('progress', {
                stage: 'build',
                status: 'starting',
                message: `执行构建命令: ${cmd}`
            });
            return new Promise((resolve, reject) => {
                const { spawn } = require('child_process');
                const shell = process.platform === 'win32' ? 'cmd.exe' : 'bash';
                const args = process.platform === 'win32' ? ['/c', cmd] : ['-c', cmd];
                // Inject env for custom build commands (same pattern as Maven/NPM/Gradle)
                const envPaths = [
                    globalConfig?.javaHome ? path.join(globalConfig.javaHome, 'bin') : null,
                    globalConfig.mavenHome ? path.join(globalConfig.mavenHome, 'bin') : null,
                    globalConfig.nodeHome ? path.join(globalConfig.nodeHome, 'bin') : null,
                    '/usr/local/bin', '/opt/homebrew/bin', '/usr/bin', '/bin',
                ].filter(Boolean);
                const existingPath = process.env.PATH || '';
                const env = {
                    ...Object.fromEntries(Object.entries(process.env).filter(([, v]) => v !== undefined)),
                    PATH: [...envPaths, ...existingPath.split(':').filter(Boolean)].join(':'),
                };
                if (globalConfig?.javaHome)
                    env.JAVA_HOME = globalConfig.javaHome;
                if (globalConfig.mavenHome)
                    env.MAVEN_HOME = globalConfig.mavenHome;
                const proc = spawn(shell, args, { cwd: buildPath, env });
                let output = '';
                proc.stdout.on('data', (data) => {
                    output += data.toString();
                    this.emit('progress', {
                        stage: 'build',
                        status: 'building',
                        message: data.toString().slice(0, 200)
                    });
                });
                proc.stderr.on('data', (data) => { output += data.toString(); });
                proc.on('error', (err) => {
                    this.emit('progress', { stage: 'build', status: 'error', message: `模块 ${module.name || '构建'} 启动失败: ${err.message}` });
                    reject(new Error(`Build failed to start: ${err.message}`));
                });
                proc.on('close', (code) => {
                    if (code === 0) {
                        this.emit('progress', { stage: 'build', status: 'success', message: `模块 ${module.name || '构建'} 成功` });
                        resolve({ success: true, output });
                    }
                    else {
                        const lastErrorLines = output.split('\n').slice(-10).join('\n').trim();
                        this.emit('progress', { stage: 'build', status: 'error', message: `模块 ${module.name || '构建'} 失败 (exit ${code})` });
                        reject(new Error(`Build failed with code ${code}\n\n最近 10 行输出:\n${lastErrorLines || '(无输出)'}`));
                    }
                });
            });
        }
        // 否则根据构建工具执行默认构建
        const tool = module.buildTool || globalConfig.buildTool || 'npm';
        if (tool === 'maven') {
            const resolvedMvn = resolveMvnPath(globalConfig.mavenHome);
            const args = ['clean', 'package'];
            if (globalConfig.skipTests !== false)
                args.push('-DskipTests');
            this.emit('progress', { stage: 'maven', status: 'starting', message: `Maven 构建: ${module.name || buildPath}` });
            return new Promise((resolve, reject) => {
                const { spawn } = require('child_process');
                const javaPath = globalConfig.javaHome || undefined;
                const envPaths = [
                    javaPath ? path.join(javaPath, 'bin') : null,
                    globalConfig.mavenHome ? path.join(globalConfig.mavenHome, 'bin') : null,
                    '/usr/local/bin', '/opt/homebrew/bin', '/usr/bin', '/bin',
                ].filter(Boolean);
                const existingPath = process.env.PATH || '';
                const env = {
                    ...process.env,
                    PATH: [...envPaths, ...existingPath.split(':').filter(Boolean)].join(':'),
                };
                if (javaPath)
                    env.JAVA_HOME = javaPath;
                if (globalConfig.mavenHome)
                    env.MAVEN_HOME = globalConfig.mavenHome;
                const proc = spawn(resolvedMvn, args, { cwd: buildPath, env });
                let output = '';
                proc.stdout.on('data', (data) => {
                    output += data.toString();
                    this.emit('progress', { stage: 'maven', status: 'building', message: data.toString().slice(0, 200) });
                });
                proc.stderr.on('data', (data) => { output += data.toString(); });
                proc.on('error', (err) => {
                    this.emit('progress', { stage: 'maven', status: 'error', message: `模块 ${module.name || '构建'} 启动失败: ${err.message}` });
                    reject(new Error(`Maven build failed to start: ${err.message}`));
                });
                proc.on('close', (code) => {
                    if (code === 0) {
                        this.emit('progress', { stage: 'maven', status: 'success', message: `模块 ${module.name || '构建'} 成功` });
                        resolve({ success: true, output });
                    }
                    else {
                        const lastLines = output.split('\n').slice(-10).join('\n').trim();
                        this.emit('progress', { stage: 'maven', status: 'error', message: `Maven 构建失败 (exit ${code})` });
                        reject(new Error(`Maven build failed with code ${code}\n\n最近 10 行输出:\n${lastLines || '(无输出)'}`));
                    }
                });
            });
        }
        else if (['npm', 'pnpm', 'yarn'].includes(tool)) {
            const script = globalConfig.npmCustomScript || globalConfig.npmScript || 'build';
            this.emit('progress', { stage: 'npm', status: 'starting', message: `${tool} ${script} 构建: ${module.name || buildPath}` });
            return new Promise((resolve, reject) => {
                const { spawn } = require('child_process');
                const nodePath = globalConfig.nodeHome || undefined;
                const envPaths = [
                    nodePath ? path.join(nodePath, 'bin') : null,
                    globalConfig.npmHome && fs.existsSync(globalConfig.npmHome) && fs.statSync(globalConfig.npmHome).isDirectory()
                        ? path.join(globalConfig.npmHome, 'bin') : null,
                    '/usr/local/bin', '/opt/homebrew/bin', '/usr/bin', '/bin',
                ].filter(Boolean);
                const existingPath = process.env.PATH || '';
                const env = {
                    ...process.env,
                    PATH: [...envPaths, ...existingPath.split(':').filter(Boolean)].join(':'),
                };
                const proc = spawn(tool, ['run', script], { cwd: buildPath, env });
                let output = '';
                proc.stdout.on('data', (data) => {
                    output += data.toString();
                    this.emit('progress', { stage: 'npm', status: 'building', message: data.toString().slice(0, 200) });
                });
                proc.stderr.on('data', (data) => { output += data.toString(); });
                proc.on('error', (err) => {
                    this.emit('progress', { stage: 'npm', status: 'error', message: `模块 ${module.name || '构建'} 启动失败: ${err.message}` });
                    reject(new Error(`${tool} build failed to start: ${err.message}`));
                });
                proc.on('close', (code) => {
                    if (code === 0) {
                        this.emit('progress', { stage: 'npm', status: 'success', message: `模块 ${module.name || '构建'} 成功` });
                        resolve({ success: true, output });
                    }
                    else {
                        const lastLines = output.split('\n').slice(-10).join('\n').trim();
                        this.emit('progress', { stage: 'npm', status: 'error', message: `${tool} 构建失败 (exit ${code})` });
                        reject(new Error(`${tool} build failed with code ${code}\n\n最近 10 行输出:\n${lastLines || '(无输出)'}`));
                    }
                });
            });
        }
        else if (tool === 'gradle') {
            const args = ['clean', 'build'];
            // Resolve gradle executable: try wrapper first, fall back to system gradle
            const gradleWrapper = process.platform === 'win32' ? 'gradlew.bat' : './gradlew';
            const resolvedGradle = fs.existsSync(path.join(buildPath, process.platform === 'win32' ? 'gradlew.bat' : 'gradlew'))
                ? gradleWrapper
                : 'gradle';
            this.emit('progress', { stage: 'gradle', status: 'starting', message: `Gradle 构建: ${module.name || buildPath}` });
            return new Promise((resolve, reject) => {
                const { spawn } = require('child_process');
                const proc = spawn(resolvedGradle, args, { cwd: buildPath });
                let output = '';
                proc.stdout.on('data', (data) => {
                    output += data.toString();
                    this.emit('progress', { stage: 'gradle', status: 'building', message: data.toString().slice(0, 200) });
                });
                proc.stderr.on('data', (data) => { output += data.toString(); });
                proc.on('error', (err) => {
                    this.emit('progress', { stage: 'gradle', status: 'error', message: `模块 ${module.name || '构建'} 启动失败: ${err.message}` });
                    reject(new Error(`Gradle build failed to start: ${err.message}`));
                });
                proc.on('close', (code) => {
                    if (code === 0) {
                        this.emit('progress', { stage: 'gradle', status: 'success', message: `模块 ${module.name || '构建'} 成功` });
                        resolve({ success: true, output });
                    }
                    else {
                        const lastLines = output.split('\n').slice(-10).join('\n').trim();
                        this.emit('progress', { stage: 'gradle', status: 'error', message: `Gradle 构建失败 (exit ${code})` });
                        reject(new Error(`Gradle build failed with code ${code}\n\n最近 10 行输出:\n${lastLines || '(无输出)'}`));
                    }
                });
            });
        }
        throw new Error(`未知的构建工具: ${tool}`);
    }
    // SSH 部署
    async sshDeploy(config, artifacts, abortSignal) {
        const { host, port, username, password, privateKey, deployDir, libDir } = config;
        if (!deployDir) {
            throw new Error('部署路径 (deployDir) 未配置');
        }
        this.emit('progress', { stage: 'ssh', status: 'info', message: `📌 部署配置: deployDir=${deployDir || '未设置'}, libDir=${libDir || '未设置'}` });
        this.emit('progress', { stage: 'ssh', status: 'connecting', message: `连接服务器 ${host}` });
        return new Promise((resolve, reject) => {
            const conn = new ssh2_1.Client();
            let settled = false;
            conn.on('error', (err) => {
                if (settled)
                    return;
                settled = true;
                this.emit('progress', { stage: 'ssh', status: 'error', message: err.message });
                conn.end();
                reject(err);
            });
            conn.on('ready', async () => {
                this.emit('progress', { stage: 'ssh', status: 'connected', message: `已连接到 ${host}` });
                try {
                    // Fetch remote $HOME once for tilde resolution in SFTP paths
                    let remoteHome = '';
                    try {
                        remoteHome = await this.sshExec(conn, 'echo $HOME').then(s => s.trim());
                    }
                    catch { /* fallback: tilde paths won't resolve for SFTP */ }
                    await this.sshExec(conn, `mkdir -p ${shellEscape(resolveRemotePath(deployDir, remoteHome))}`);
                    if (abortSignal?.aborted) {
                        conn.end();
                        return reject(new Error('部署已取消'));
                    }
                    // 仅当有 lib 产物且没有模块 deployPath（即依赖共享 lib 目录）时才创建 lib 目录
                    const hasLibWithoutDeployPath = artifacts.some(a => a.isLib && !a.deployPath);
                    if (hasLibWithoutDeployPath && libDir) {
                        await this.sshExec(conn, `mkdir -p ${shellEscape(resolveRemotePath(libDir, remoteHome))}`);
                    }
                    // 创建每个模块的独立部署路径目录（父模块统一构建模式）
                    const uniqueDeployPaths = new Set();
                    for (const a of artifacts) {
                        if (a.deployPath && a.deployPath !== deployDir) {
                            uniqueDeployPaths.add(a.deployPath);
                        }
                    }
                    for (const dp of uniqueDeployPaths) {
                        await this.sshExec(conn, `mkdir -p ${shellEscape(resolveRemotePath(dp, remoteHome))}`);
                    }
                    const artifactCount = artifacts.length;
                    this.emit('progress', { stage: 'ssh', status: 'info', message: `\n========== 上传目标解析 ==========` });
                    for (const a of artifacts) {
                        const typeTag = a.isLib ? '[lib]' : a.isCompressed ? '[zip]' : '[jar]';
                        let resolvedTarget;
                        if (a.deployPath) {
                            resolvedTarget = a.isLib ? path.posix.join(a.deployPath, 'lib') : a.deployPath;
                        }
                        else if (a.isLib && libDir) {
                            resolvedTarget = libDir;
                        }
                        else {
                            resolvedTarget = deployDir;
                        }
                        this.emit('progress', { stage: 'ssh', status: 'info', message: `  ${a.module || '主模块'} | ${typeTag} ${a.name} → ${resolvedTarget}` });
                    }
                    this.emit('progress', { stage: 'ssh', status: 'info', message: `===============================\n` });
                    this.emit('progress', { stage: 'ssh', status: 'info', message: `准备上传 ${artifactCount} 个产物` });
                    if (artifactCount === 0) {
                        throw new Error('未收集到任何构建产物，请检查模块配置和产物路径');
                    }
                    // Calculate total upload size
                    let totalBytes = 0;
                    for (const a of artifacts) {
                        totalBytes += fs.statSync(a.localPath).size;
                    }
                    const totalMB = (totalBytes / 1024 / 1024).toFixed(1);
                    this.emit('progress', { stage: 'ssh', status: 'info', message: `总上传大小: ${totalMB}MB` });
                    // Track cumulative upload progress with atomic counter
                    let uploadedBytes = 0;
                    const atomicAdd = (n) => { uploadedBytes += n; return uploadedBytes; };
                    // Phase 1: Parallel upload all artifacts — each gets its own SFTP channel for max throughput
                    this.emit('progress', { stage: 'ssh', status: 'info', message: `🚀 上传 ${artifactCount} 个产物` });
                    // Create one SFTP channel and reuse it for all uploads (fixes channel leak)
                    const uploadSftp = await new Promise((res, rej) => {
                        conn.sftp((err, sftp) => err ? rej(err) : res(sftp));
                    });
                    // Parallel upload with limited concurrency (each file uses 2 exec channels: rm + mv)
                    // 3 concurrent × 2 exec + 1 shared SFTP = 7 channels < MaxSessions(10)
                    const maxConcurrent = Math.min(3, artifactCount);
                    this.emit('progress', { stage: 'ssh', status: 'info', message: `🚀 上传 ${artifactCount} 个产物 (并发 ${maxConcurrent})` });
                    async function asyncPool(poolLimit, items, fn) {
                        const results = [];
                        const inProgress = new Set();
                        for (const item of items) {
                            const p = fn(item);
                            results.push(p);
                            inProgress.add(p);
                            void p.finally(() => inProgress.delete(p));
                            if (inProgress.size >= poolLimit) {
                                await Promise.race(inProgress);
                            }
                        }
                        await Promise.allSettled(results);
                    }
                    try {
                        await asyncPool(maxConcurrent, artifacts, async (artifact) => {
                            if (abortSignal?.aborted)
                                return;
                            const fileSize = fs.statSync(artifact.localPath).size;
                            const fileMB = (fileSize / 1024 / 1024).toFixed(1);
                            const uploadStart = atomicAdd(0); // snapshot current position
                            if (artifact.isCompressed) {
                                const effectiveDeployDir = artifact.deployPath || deployDir;
                                const zipRemote = resolveSftpPath(effectiveDeployDir.endsWith('/') ? effectiveDeployDir + artifact.name : effectiveDeployDir + '/' + artifact.name, remoteHome);
                                this.emit('progress', { stage: 'ssh', status: 'uploading', message: `↑ ${artifact.name} (${fileMB}MB)` });
                                await this.sshUpload(conn, artifact.localPath, zipRemote, (transferred) => {
                                    const currentTotal = uploadStart + transferred;
                                    const pct = Math.round((currentTotal / totalBytes) * 100);
                                    this.emit('progress', {
                                        stage: 'ssh', status: 'uploading',
                                        message: `上传中 ${pct}% (${(currentTotal / 1024 / 1024).toFixed(1)}MB/${totalMB}MB)`,
                                        progress: pct
                                    });
                                }, uploadSftp);
                                atomicAdd(fileSize);
                                this.emit('progress', { stage: 'ssh', status: 'success', message: `✅ ${artifact.name} 上传完成` });
                            }
                            else {
                                const targetPath = artifact.deployPath
                                    ? (artifact.isLib ? path.posix.join(artifact.deployPath, 'lib') : artifact.deployPath)
                                    : (artifact.isLib && libDir ? libDir : deployDir);
                                const remotePath = resolveSftpPath(targetPath.endsWith('/') ? targetPath + artifact.name : targetPath + '/' + artifact.name, remoteHome);
                                this.emit('progress', { stage: 'ssh', status: 'uploading', message: `↑ ${artifact.name} (${fileMB}MB)` });
                                await this.sshUpload(conn, artifact.localPath, remotePath, (transferred) => {
                                    const currentTotal = uploadStart + transferred;
                                    const pct = Math.round((currentTotal / totalBytes) * 100);
                                    this.emit('progress', {
                                        stage: 'ssh', status: 'uploading',
                                        message: `上传中 ${pct}% (${(currentTotal / 1024 / 1024).toFixed(1)}MB/${totalMB}MB)`,
                                        progress: pct
                                    });
                                }, uploadSftp);
                                atomicAdd(fileSize);
                                this.emit('progress', { stage: 'ssh', status: 'success', message: `✅ ${artifact.name} → ${targetPath}` });
                            }
                        });
                    }
                    finally {
                        uploadSftp.end();
                    }
                    if (abortSignal?.aborted) {
                        conn.end();
                        return reject(new Error('部署已取消'));
                    }
                    // Phase 2: Extract compressed artifacts sequentially (avoid concurrent unzip conflicts)
                    const compressedArtifacts = artifacts.filter(a => a.isCompressed);
                    for (const artifact of compressedArtifacts) {
                        if (abortSignal?.aborted) {
                            conn.end();
                            return reject(new Error('部署已取消'));
                        }
                        const effectiveDeployDir = artifact.deployPath || deployDir;
                        const zipRemote = resolveSftpPath(effectiveDeployDir.endsWith('/') ? effectiveDeployDir + artifact.name : effectiveDeployDir + '/' + artifact.name, remoteHome);
                        let remoteExtractDir;
                        if (artifact.deployPath) {
                            remoteExtractDir = artifact.isLib
                                ? path.posix.join(artifact.deployPath, 'lib')
                                : artifact.deployPath;
                        }
                        else if (artifact.isLib && libDir) {
                            remoteExtractDir = libDir;
                        }
                        else {
                            remoteExtractDir = effectiveDeployDir;
                        }
                        this.emit('progress', { stage: 'ssh', status: 'info', message: `解压 ${artifact.name} → ${remoteExtractDir}` });
                        const resolvedExtractDir = resolveRemotePath(remoteExtractDir, remoteHome);
                        const unzipOutput = await this.sshExec(conn, `mkdir -p ${shellEscape(resolvedExtractDir)} && cd ${shellEscape(resolvedExtractDir)} && unzip -o ${shellEscape(zipRemote)} 2>&1`);
                        this.emit('progress', { stage: 'ssh', status: 'info', message: `解压输出: ${unzipOutput.trim().slice(0, 200)}` });
                        const verifyFiles = await this.sshExec(conn, `ls -la ${shellEscape(resolvedExtractDir)} | head -5`);
                        this.emit('progress', { stage: 'ssh', status: 'info', message: `远程目录: ${verifyFiles.trim().slice(0, 200)}` });
                        await this.sshExec(conn, `rm -f ${shellEscape(zipRemote)}`);
                        this.emit('progress', { stage: 'ssh', status: 'success', message: `✅ ${artifact.name} 解压完成 → ${remoteExtractDir}` });
                    }
                    // Set progress to 100% after all uploads done
                    this.emit('progress', { stage: 'ssh', status: 'info', message: `\n========== 部署清单 ==========` });
                    for (const a of artifacts) {
                        const typeTag = a.isLib ? '[lib]' : a.isCompressed ? '[zip]' : '[jar]';
                        const resolved = a.deployPath
                            ? (a.isLib ? path.posix.join(a.deployPath, 'lib') : a.deployPath)
                            : (a.isLib && libDir ? libDir : deployDir);
                        this.emit('progress', { stage: 'ssh', status: 'info', message: `  ✅ ${typeTag} ${a.name} → ${resolved}` });
                    }
                    this.emit('progress', { stage: 'ssh', status: 'info', message: `===============================\n` });
                    // Post-deploy verification: create one SFTP channel for all verifications
                    const verifySftp = await new Promise((res, rej) => {
                        conn.sftp((err, sftp) => err ? rej(err) : res(sftp));
                    });
                    let verifyFailed = false;
                    try {
                        this.emit('progress', { stage: 'ssh', status: 'info', message: `\n========== 部署验证 ==========` });
                        for (const a of artifacts) {
                            const resolved = a.deployPath
                                ? (a.isLib ? path.posix.join(a.deployPath, 'lib') : a.deployPath)
                                : (a.isLib && libDir ? libDir : deployDir);
                            const targetPath = resolveSftpPath((a.isLib || a.isCompressed) ? resolved : (resolved.endsWith('/') ? resolved + a.name : resolved + '/' + a.name), remoteHome);
                            const exists = await new Promise((resolve) => {
                                verifySftp.stat(targetPath, (e, stats) => {
                                    if (e)
                                        return resolve(false);
                                    if (a.isLib || a.isCompressed) {
                                        resolve(stats.isDirectory());
                                    }
                                    else {
                                        const localSize = fs.statSync(a.localPath).size;
                                        resolve(stats.size === localSize);
                                    }
                                });
                            });
                            const status = exists ? 'success' : 'error';
                            const icon = exists ? '✅' : '❌';
                            this.emit('progress', { stage: 'ssh', status, message: `  ${icon} ${a.module || '主模块'} | ${a.name} → ${targetPath}` });
                            if (!exists)
                                verifyFailed = true;
                        }
                    }
                    finally {
                        verifySftp.end();
                    }
                    this.emit('progress', { stage: 'ssh', status: 'info', message: `===============================\n` });
                    if (verifyFailed) {
                        this.emit('progress', { stage: 'ssh', status: 'error', message: '部分产物验证失败，请检查远程服务器' });
                    }
                    this.emit('progress', { stage: 'ssh', status: 'success', message: `部署完成`, progress: 100 });
                    conn.end();
                    settled = true;
                    resolve({ success: true });
                }
                catch (error) {
                    this.emit('progress', { stage: 'ssh', status: 'error', message: getErrorMessage(error) });
                    conn.end();
                    settled = true;
                    reject(error);
                }
            });
            conn.on('error', (err) => {
                if (settled)
                    return;
                settled = true;
                this.emit('progress', { stage: 'ssh', status: 'error', message: err.message });
                conn.end();
                reject(err);
            });
            conn.connect({
                host,
                port: port || 22,
                username,
                password: password || undefined,
                privateKey: privateKey || undefined
            });
        });
    }
    // SSH 执行命令
    async sshExec(conn, command) {
        return new Promise((resolve, reject) => {
            conn.exec(command, (err, stream) => {
                if (err)
                    return reject(err);
                let output = '';
                stream.on('data', (data) => {
                    output += data.toString();
                });
                stream.stderr?.on('data', (data) => {
                    output += data.toString();
                });
                stream.on('close', () => {
                    resolve(output);
                });
            });
        });
    }
    // SSH 上传文件（带进度回调、重试、完整性校验、临时文件 + 原子移动）
    async sshUpload(conn, localPath, remotePath, onProgress, cachedSftp) {
        const localSize = fs.statSync(localPath).size;
        const maxRetries = 3;
        const retryDelay = [2000, 5000, 10000]; // 2s, 5s, 10s backoff
        for (let attempt = 0; attempt < maxRetries; attempt++) {
            try {
                await this._doUpload(conn, localPath, remotePath, localSize, onProgress, attempt > 0, cachedSftp);
                return; // Success
            }
            catch (error) {
                // Clean up potentially corrupted remote file before retry
                try {
                    await this.sshExec(conn, `rm -f ${shellEscape(remotePath)}.uploading 2>/dev/null`);
                }
                catch { /* ignore cleanup error */ }
                const isLastAttempt = attempt === maxRetries - 1;
                if (isLastAttempt)
                    throw error;
                this.emit('progress', {
                    stage: 'ssh', status: 'warning',
                    message: `上传失败 (${attempt + 1}/${maxRetries}), ${retryDelay[attempt] / 1000}s 后重试: ${getErrorMessage(error)}`
                });
                await new Promise(r => setTimeout(r, retryDelay[attempt]));
            }
        }
    }
    // Internal upload implementation with resume support
    async _doUpload(conn, localPath, remotePath, totalBytes, onProgress, isRetry, cachedSftp) {
        const tempPath = remotePath + '.uploading';
        // Step 1: Check if final remote file already exists and is complete
        const finalInfo = await this._getRemoteFileInfoCached(conn, remotePath, cachedSftp);
        if (finalInfo.size === totalBytes) {
            // Check modification time to detect same-size rebuilds
            const localMtime = Math.floor(fs.statSync(localPath).mtimeMs / 1000);
            if (finalInfo.mtime > 0 && finalInfo.mtime === localMtime) {
                this.emit('progress', {
                    stage: 'ssh', status: 'info',
                    message: `✓ ${path.basename(remotePath)} 已存在且未修改，跳过上传`
                });
                if (onProgress)
                    onProgress(totalBytes, totalBytes);
                return;
            }
            // Same size but different mtime — could be a rebuild, force upload
            this.emit('progress', {
                stage: 'ssh', status: 'info',
                message: `⟳ ${path.basename(remotePath)} 远程文件较旧，重新上传`
            });
        }
        if (finalInfo.size > 0 && finalInfo.size !== totalBytes) {
            // Incomplete final file — delete it
            await this.sshExec(conn, `rm -f ${shellEscape(remotePath)}`);
        }
        // Step 2: Check for leftover temp file from previous failed attempt
        const tempInfo = await this._getRemoteFileInfoCached(conn, tempPath, cachedSftp);
        let startOffset = 0;
        if (tempInfo.size > 0 && tempInfo.size < totalBytes) {
            // Verify the partial data before resuming — check last 4KB
            const verifySize = Math.min(4096, tempInfo.size);
            const partialOk = await this._verifyPartialMatch(conn, localPath, tempPath, tempInfo.size - verifySize, verifySize);
            if (partialOk) {
                startOffset = tempInfo.size;
                this.emit('progress', {
                    stage: 'ssh', status: 'info',
                    message: `断点续传: 已验证 ${(startOffset / 1024 / 1024).toFixed(1)}MB, 继续上传剩余 ${((totalBytes - startOffset) / 1024 / 1024).toFixed(1)}MB`
                });
            }
            else {
                this.emit('progress', {
                    stage: 'ssh', status: 'info',
                    message: `临时文件校验失败，从头上传`
                });
                await this.sshExec(conn, `rm -f ${shellEscape(tempPath)}`);
                startOffset = 0;
            }
        }
        // Step 3: Upload to temp file (uses SFTP, not exec channels)
        await this._streamUpload(conn, localPath, tempPath, totalBytes, startOffset, onProgress, cachedSftp);
        // Step 4: Verify uploaded temp file (size check only)
        const uploadedInfo = await this._getRemoteFileInfoCached(conn, tempPath, cachedSftp);
        if (uploadedInfo.size !== totalBytes) {
            throw new Error(`上传后大小不一致: 本地=${totalBytes}, 远程=${uploadedInfo.size}`);
        }
        // Step 5: Atomic move to final path
        await this.sshExec(conn, `mv -f ${shellEscape(tempPath)} ${shellEscape(remotePath)}`);
        this.emit('progress', {
            stage: 'ssh', status: 'info',
            message: `✓ ${path.basename(remotePath)} 上传完成 (${(totalBytes / 1024 / 1024).toFixed(1)}MB)`
        });
    }
    /** Get remote file size (returns 0 if file doesn't exist) */
    async _getRemoteFileInfo(conn, remotePath) {
        try {
            const sizeStr = await this.sshExec(conn, `stat -c %s ${shellEscape(remotePath)} 2>/dev/null || stat -f %z ${shellEscape(remotePath)} 2>/dev/null`);
            return { size: parseInt(sizeStr.trim(), 10) || 0 };
        }
        catch {
            return { size: 0 };
        }
    }
    /** Get remote file size and mtime via SFTP (avoids opening exec channels) */
    _getRemoteFileInfoCached(conn, remotePath, cachedSftp) {
        return new Promise((resolve) => {
            const tryStat = (sftp) => {
                sftp.stat(remotePath, (err, stats) => {
                    if (err)
                        return resolve({ size: 0, mtime: 0 });
                    resolve({ size: stats.size, mtime: Math.floor(stats.mtime || 0) });
                });
            };
            if (cachedSftp) {
                tryStat(cachedSftp);
            }
            else {
                conn.sftp((err, sftp) => {
                    if (err)
                        return resolve({ size: 0, mtime: 0 });
                    tryStat(sftp);
                });
            }
        });
    }
    /** Get remote file MD5 (returns empty string on failure) */
    async _getRemoteMD5(conn, remotePath) {
        try {
            const md5Str = await this.sshExec(conn, `md5sum ${shellEscape(remotePath)} 2>/dev/null || md5 -q ${shellEscape(remotePath)} 2>/dev/null`);
            return md5Str.trim().split(/\s+/)[0] || md5Str.trim();
        }
        catch {
            return '';
        }
    }
    /** Verify a portion of the remote file matches the local file at the same offset */
    async _verifyPartialMatch(conn, localPath, remotePath, offset, length) {
        try {
            // Compute local hash of the portion
            const localHash = await this._computeLocalRangeMD5(localPath, offset, length);
            // Get remote hash of the same portion
            const remoteHash = await this.sshExec(conn, `dd if=${shellEscape(remotePath)} bs=1 skip=${offset} count=${length} 2>/dev/null | md5sum | awk '{print $1}'`);
            return remoteHash.trim().split(/\s+/)[0] === localHash;
        }
        catch {
            return false;
        }
    }
    /** Compute MD5 of a specific byte range in a local file */
    async _computeLocalRangeMD5(localPath, offset, length) {
        const { createHash } = require('crypto');
        return new Promise((resolve, reject) => {
            const hash = createHash('md5');
            const stream = fs.createReadStream(localPath, { start: offset, end: offset + length - 1 });
            stream.on('data', (data) => hash.update(data));
            stream.on('end', () => resolve(hash.digest('hex')));
            stream.on('error', reject);
        });
    }
    // Stream-based upload with progress tracking and resume support
    // 全量上传使用 sftp.fastPut（分块传输 + SFTP 确认），断点续传使用 createWriteStream
    _streamUpload(conn, localPath, remotePath, totalBytes, startOffset, onProgress, cachedSftp) {
        return new Promise((resolve, reject) => {
            const trySftp = () => {
                if (cachedSftp) {
                    this._doStreamUpload(cachedSftp, localPath, remotePath, totalBytes, startOffset, onProgress, conn, resolve, reject);
                }
                else {
                    conn.sftp((err, sftp) => {
                        if (err)
                            return reject(err);
                        this._doStreamUpload(sftp, localPath, remotePath, totalBytes, startOffset, onProgress, conn, resolve, reject);
                    });
                }
            };
            trySftp();
        });
    }
    _doStreamUpload(sftp, localPath, remotePath, totalBytes, startOffset, onProgress, conn, resolve, reject) {
        if (startOffset > 0) {
            // Resume: append to existing partial file
            const stream = sftp.createWriteStream(remotePath, { flags: 'a', autoClose: true });
            const fileStream = fs.createReadStream(localPath, { start: startOffset });
            let transferred = startOffset;
            stream.on('close', () => resolve());
            stream.on('error', reject);
            fileStream.on('data', (chunk) => {
                transferred += chunk.length;
                if (onProgress)
                    onProgress(transferred, totalBytes);
            });
            fileStream.on('error', reject);
            fileStream.pipe(stream);
        }
        else {
            // Full upload: use fastPut for reliable chunked transfer
            const concurrency = Math.min(4, Math.max(1, Math.ceil(totalBytes / (256 * 1024))));
            sftp.fastPut(localPath, remotePath, { concurrency }, (fastErr) => {
                if (fastErr)
                    return reject(fastErr);
                if (onProgress)
                    onProgress(totalBytes, totalBytes);
                resolve();
            });
            // Track progress for fastPut: poll remote file size via SFTP (NOT exec channel)
            if (onProgress) {
                const pollRemote = () => {
                    sftp.stat(remotePath, (err, stats) => {
                        if (!err && stats && stats.size > 0) {
                            onProgress(stats.size, totalBytes);
                        }
                        if (!err && stats && stats.size < totalBytes) {
                            setTimeout(pollRemote, 300);
                        }
                    });
                };
                pollRemote();
            }
        }
    }
    async _computeLocalMD5(localPath) {
        const { createHash } = require('crypto');
        return new Promise((resolve, reject) => {
            const hash = createHash('md5');
            const stream = fs.createReadStream(localPath);
            stream.on('data', (data) => hash.update(data));
            stream.on('end', () => resolve(hash.digest('hex')));
            stream.on('error', reject);
        });
    }
    // SSH 执行重启脚本（支持绝对路径和相对路径）
    async executeRestartScript(config, scriptPath) {
        const { host, port, username, password, privateKey, deployDir } = config;
        this.emit('progress', { stage: 'restart', status: 'starting', message: `执行重启脚本: ${scriptPath}` });
        return new Promise((resolve, reject) => {
            const conn = new ssh2_1.Client();
            conn.on('ready', async () => {
                try {
                    // Fetch remote $HOME once for tilde resolution
                    let remoteHome = '';
                    try {
                        remoteHome = await this.sshExec(conn, 'echo $HOME').then(s => s.trim());
                    }
                    catch { /* fallback: tilde paths won't resolve */ }
                    // 绝对路径直接执行（不cd，bash -l 加载用户环境变量如 .profile/.bashrc）
                    let execCmd;
                    // Parse script path: first token is the script file, rest are arguments
                    const parts = scriptPath.trim().split(/\s+/);
                    const scriptFile = parts[0];
                    const scriptArgs = parts.slice(1).join(' ');
                    if (scriptPath.startsWith('/')) {
                        execCmd = `chmod +x ${shellEscape(resolveRemotePath(scriptFile, remoteHome))} && bash -l -c ${shellEscape(resolveRemotePath(scriptPath, remoteHome))} 2>&1`;
                    }
                    else {
                        // 相对路径需要先 cd 到 deployDir 再执行
                        execCmd = `cd ${shellEscape(resolveRemotePath(deployDir, remoteHome))} && chmod +x ${shellEscape(resolveRemotePath(scriptFile, remoteHome))} && bash -l -c ${shellEscape(resolveRemotePath(scriptPath, remoteHome))} 2>&1`;
                    }
                    this.emit('progress', { stage: 'restart', status: 'info', message: `执行命令: ${execCmd.slice(0, 120)}` });
                    const output = await this.sshExec(conn, execCmd);
                    this.emit('progress', { stage: 'restart', status: 'success', message: `应用已重启 (输出: ${output.trim().slice(0, 200)})` });
                    conn.end();
                    resolve({ success: true, output });
                }
                catch (error) {
                    this.emit('progress', { stage: 'restart', status: 'error', message: `重启失败: ${getErrorMessage(error)}` });
                    conn.end();
                    reject(error);
                }
            });
            conn.on('error', (err) => {
                this.emit('progress', { stage: 'restart', status: 'error', message: `SSH连接失败: ${err.message}` });
                conn.end();
                reject(err);
            });
            conn.connect({
                host,
                port: port || 22,
                username,
                password: password || undefined,
                privateKey: privateKey || undefined
            });
        });
    }
    // 完整部署流程
    async deployFull(config, dataDir, abortSignal, deployId) {
        const resolvedDeployId = deployId || Date.now().toString();
        this.deployProgress.set(resolvedDeployId, { status: 'running', stages: [] });
        // Set up log file and artifact paths (outside try so they're in catch scope)
        const homeDir = os.homedir();
        const resolvedDataDir = dataDir || path.join(homeDir, '.supertool');
        const logDir = path.join(resolvedDataDir, 'deploy-logs');
        const logFilePath = path.join(logDir, `${resolvedDeployId}.log`);
        const artifactDir = path.join(resolvedDataDir, 'deploy-artifacts', resolvedDeployId);
        if (!fs.existsSync(logDir))
            fs.mkdirSync(logDir, { recursive: true });
        const logEntry = (stage, status, message) => {
            const timestamp = new Date().toISOString();
            const line = `[${timestamp}] [${stage}] [${status}] ${message}\n`;
            fs.appendFileSync(logFilePath, line);
        };
        const progressHandler = (event) => { logEntry(event.stage, event.status, event.message); };
        const deployCompleteHandler = (event) => { logEntry('deploy', event.success ? 'complete' : 'failed', JSON.stringify(event)); };
        const deployNotificationHandler = (event) => { logEntry('deploy', 'notification', JSON.stringify(event)); };
        const cleanupListeners = () => {
            this.removeListener('progress', progressHandler);
            this.removeListener('deploy-complete', deployCompleteHandler);
            this.removeListener('deploy:notification', deployNotificationHandler);
        };
        try {
            logEntry('deploy', 'starting', `开始部署 ${config.repoUrl || config.localPath || 'unknown'}`);
            this.on('progress', progressHandler);
            this.on('deploy-complete', deployCompleteHandler);
            this.on('deploy:notification', deployNotificationHandler);
            // Step 1: Git sync or use local path
            let projectPath;
            if (config.localPath) {
                const localGit = (0, simple_git_1.simpleGit)(config.localPath);
                try {
                    // 规范化分支名：去掉 origin/ 前缀，避免 checkout 远程分支导致 detached HEAD
                    let targetBranch = config.branch;
                    if (targetBranch.startsWith('origin/')) {
                        targetBranch = targetBranch.replace('origin/', '');
                    }
                    await localGit.fetch('origin');
                    const status = await localGit.status();
                    const currentBranch = status.current;
                    // Reject deployment if working directory is dirty — don't auto-commit
                    if (status.modified.length > 0 || status.not_added.length > 0 || status.deleted.length > 0 || status.staged.length > 0) {
                        const dirtyCount = status.modified.length + status.not_added.length + status.deleted.length + status.staged.length;
                        const modified = status.modified.length > 0 ? ` ${status.modified.length} 个已修改` : '';
                        const added = status.not_added.length > 0 ? ` ${status.not_added.length} 个未跟踪` : '';
                        const deleted = status.deleted.length > 0 ? ` ${status.deleted.length} 个已删除` : '';
                        const staged = status.staged.length > 0 ? ` ${status.staged.length} 个已暂存` : '';
                        throw new Error(`工作目录有 ${dirtyCount} 个未提交的变更（${modified}${added}${deleted}${staged}），请先提交代码再部署。禁止自动 commit 污染 git 历史。`);
                    }
                    if (currentBranch !== targetBranch) {
                        this.emit('progress', { stage: 'git', status: 'switching', message: `当前分支 ${currentBranch}，切换到 ${targetBranch}...`, progress: 5 });
                        // 先尝试切换到已有分支，如果不存在则从 origin 创建
                        const branches = await localGit.branchLocal();
                        if (branches.all.includes(targetBranch)) {
                            await localGit.checkout(targetBranch);
                        }
                        else {
                            await localGit.checkout(['-b', targetBranch, `origin/${targetBranch}`, '--track']);
                        }
                    }
                    // 无论是否切换，都合并远程最新代码（保留本地修改）
                    this.emit('progress', { stage: 'git', status: 'pulling', message: `合并远程最新代码...`, progress: 5 });
                    await localGit.pull('origin', targetBranch, ['--no-edit', '--rebase']);
                    const msg = currentBranch !== targetBranch
                        ? `使用本地目录: ${config.localPath} (已切换到 ${targetBranch})`
                        : `使用本地目录: ${config.localPath} (当前分支 ${targetBranch} 已更新)`;
                    this.emit('progress', { stage: 'git', status: 'success', message: msg, progress: 5 });
                }
                catch (err) {
                    const msg = getErrorMessage(err);
                    if (msg.includes('not a git repository') || msg.includes('not a git repo')) {
                        this.emit('progress', { stage: 'git', status: 'warning', message: `使用本地目录: ${config.localPath} (非 Git 仓库，跳过分支切换)`, progress: 5 });
                    }
                    else {
                        throw err;
                    }
                }
                projectPath = config.localPath;
            }
            else {
                this.emit('progress', { stage: 'git', status: 'syncing', message: `同步代码...`, progress: 5 });
                const gitResult = await this.gitSync(config.repoUrl, config.branch, config.localPath);
                projectPath = gitResult.path;
                this.emit('progress', { stage: 'git', status: 'success', message: `代码同步完成`, progress: 15 });
            }
            if (abortSignal?.aborted) {
                throw new Error('部署已取消');
            }
            // Step 2: Build
            const hasModules = Array.isArray(config.modules) && config.modules.length > 0;
            const isModuleConfig = hasModules && typeof config.modules[0] === 'object';
            if (hasModules && isModuleConfig) {
                // Per-module or parent unified build
                const moduleList = config.modules.sort((a, b) => (a.deployOrder || 0) - (b.deployOrder || 0));
                // Check if parent unified build mode is explicitly enabled
                const isParentBuildMode = config.parentBuildMode && config.buildTool === 'maven';
                if (isParentBuildMode) {
                    // Parent POM unified build: run mvn once in parent directory, builds all sub-modules
                    this.emit('progress', { stage: 'maven', status: 'starting', message: `父模块统一构建 (Maven multi-module)...`, progress: 20 });
                    const resolvedMvn = resolveMvnPath(config.mavenHome);
                    const args = ['clean', 'package'];
                    if (config.skipTests !== false)
                        args.push('-DskipTests');
                    // Respect user-configured Maven profile and settings
                    if (config.mavenProfile)
                        args.push('-P', config.mavenProfile);
                    if (config.mavenSettings)
                        args.push('-s', config.mavenSettings);
                    // Parent build directory: if set, use it relative to projectPath; otherwise use projectPath
                    const parentCwd = config.parentBuildPath ? path.join(projectPath, config.parentBuildPath) : projectPath;
                    this.emit('progress', { stage: 'maven', status: 'starting', message: `执行: ${resolvedMvn} ${args.join(' ')} (目录: ${parentCwd})` });
                    await new Promise((resolve, reject) => {
                        const { spawn } = require('child_process');
                        const javaPath = config.javaHome || undefined;
                        const envPaths = [
                            javaPath ? path.join(javaPath, 'bin') : null,
                            config.mavenHome ? path.join(config.mavenHome, 'bin') : null,
                            '/usr/local/bin', '/opt/homebrew/bin', '/usr/bin', '/bin',
                        ].filter(Boolean);
                        const existingPath = process.env.PATH || '';
                        const env = {
                            ...process.env,
                            PATH: [...envPaths, ...existingPath.split(':').filter(Boolean)].join(':'),
                        };
                        if (javaPath)
                            env.JAVA_HOME = javaPath;
                        if (config.mavenHome)
                            env.MAVEN_HOME = config.mavenHome;
                        const proc = spawn(resolvedMvn, args, { cwd: parentCwd, env });
                        let output = '';
                        proc.stdout.on('data', (data) => {
                            output += data.toString();
                            this.emit('progress', { stage: 'maven', status: 'building', message: data.toString().slice(0, 200) });
                        });
                        proc.stderr.on('data', (data) => { output += data.toString(); });
                        proc.on('error', (err) => {
                            this.emit('progress', { stage: 'maven', status: 'error', message: `父模块构建失败: ${err.message}` });
                            reject(new Error(`Maven build failed to start: ${err.message}`));
                        });
                        proc.on('close', (code) => {
                            if (code === 0) {
                                this.emit('progress', { stage: 'maven', status: 'success', message: `父模块构建成功 (${moduleList.length} 个子模块)` });
                                resolve();
                            }
                            else {
                                const lastLines = output.split('\n').slice(-10).join('\n').trim();
                                this.emit('progress', { stage: 'maven', status: 'error', message: `父模块构建失败 (exit ${code})` });
                                reject(new Error(`Maven build failed with code ${code}\n\n最近 10 行输出:\n${lastLines || '(无输出)'}`));
                            }
                        });
                    });
                }
                else {
                    // Independent per-module build (current behavior)
                    this.emit('progress', { stage: 'maven', status: 'starting', message: `开始构建...`, progress: 20 });
                    for (const mod of moduleList) {
                        if (abortSignal?.aborted)
                            throw new Error('部署已取消');
                        await this.buildModule(projectPath, mod, config);
                    }
                }
            }
            else if (hasModules) {
                // Legacy string[] modules - use old behavior
                const modulePaths = config.modules;
                const buildTool = config.buildTool || (config.mavenHome ? 'maven' : 'npm');
                if (buildTool === 'maven') {
                    await this.mavenBuild(projectPath, config.mavenHome, modulePaths, config.skipTests !== false, config.javaHome);
                }
                else if (['npm', 'pnpm', 'yarn'].includes(buildTool)) {
                    await this.npmBuild(projectPath, buildTool, config.npmScript || 'build', config.npmHome, config.nodeHome);
                }
                else if (buildTool === 'gradle') {
                    await this.gradleBuild(projectPath, modulePaths);
                }
            }
            else {
                // Single project build
                const buildPath = config.buildPath ? path.join(projectPath, config.buildPath) : projectPath;
                const buildTool = config.buildTool || (config.mavenHome ? 'maven' : 'npm');
                if (buildTool === 'maven') {
                    await this.mavenBuild(buildPath, config.mavenHome, [], config.skipTests !== false, config.javaHome);
                }
                else if (['npm', 'pnpm', 'yarn'].includes(buildTool)) {
                    await this.npmBuild(buildPath, buildTool, config.npmScript || 'build', config.npmHome, config.nodeHome);
                }
                else if (buildTool === 'gradle') {
                    await this.gradleBuild(buildPath, []);
                }
            }
            if (abortSignal?.aborted) {
                throw new Error('部署已取消');
            }
            const artifacts = await this.collectArtifacts(projectPath, config.modules || [], config.libSeparate);
            if (artifacts.length > 0) {
                this.emit('progress', { stage: 'collect', status: 'info', message: `\n========== 收集产物清单 ==========` });
                for (const a of artifacts) {
                    const sizeMB = (fs.statSync(a.localPath).size / 1024 / 1024).toFixed(1);
                    const typeTag = a.isLib ? '[lib]' : a.isCompressed ? '[zip]' : '[jar]';
                    const deployTarget = a.deployPath || (a.isLib && config.libDir ? config.libDir : config.deployDir) || '??';
                    this.emit('progress', { stage: 'collect', status: 'info', message: `  ${a.module || '主模块'} | ${typeTag} ${a.name} (${sizeMB}MB) → ${deployTarget}` });
                }
                this.emit('progress', { stage: 'collect', status: 'info', message: `==================================\n` });
            }
            this.emit('progress', { stage: 'collect', status: 'success', message: `产物收集完成 (${artifacts.length} 个)`, progress: 60 });
            if (abortSignal?.aborted) {
                throw new Error('部署已取消');
            }
            // Copy artifacts to deploy-artifacts directory and create manifest
            if (!fs.existsSync(artifactDir))
                fs.mkdirSync(artifactDir, { recursive: true });
            const artifactPaths = [];
            const manifestArtifacts = [];
            for (const artifact of artifacts) {
                const destPath = path.join(artifactDir, artifact.name);
                fs.copyFileSync(artifact.localPath, destPath);
                const stat = fs.statSync(destPath);
                artifactPaths.push(destPath);
                manifestArtifacts.push({
                    name: artifact.name, size: stat.size, path: destPath,
                    module: artifact.module, isLib: artifact.isLib,
                    isCompressed: artifact.isCompressed, extractPath: artifact.extractPath,
                    deployPath: artifact.deployPath,
                });
            }
            fs.writeFileSync(path.join(artifactDir, 'manifest.json'), JSON.stringify(manifestArtifacts, null, 2));
            // Multi-server deployment (parallel)
            const deployServers = config.servers;
            this.emit('progress', {
                stage: 'ssh',
                status: 'starting',
                message: `并行部署到 ${deployServers.length} 个服务器...`,
                progress: 65
            });
            const deployResults = await Promise.allSettled(deployServers.map((srv, i) => this.deploySingleServer(srv, artifacts, config.restartScript, abortSignal)));
            // Check for failures
            const failedServers = [];
            for (let i = 0; i < deployResults.length; i++) {
                const srv = deployServers[i];
                const serverLabel = srv.label || `服务器 ${i + 1}`;
                const result = deployResults[i];
                if (result.status === 'rejected') {
                    failedServers.push(`${serverLabel} (${srv.host}): ${getErrorMessage(result.reason)}`);
                    this.emit('progress', {
                        stage: 'ssh',
                        status: 'error',
                        message: `${serverLabel} 部署失败: ${getErrorMessage(result.reason)}`
                    });
                }
                else {
                    this.emit('progress', {
                        stage: 'ssh',
                        status: 'success',
                        message: `${serverLabel} 部署完成 (${i + 1}/${deployServers.length})`
                    });
                }
            }
            if (failedServers.length > 0) {
                throw new Error(`部分服务器部署失败:\n${failedServers.join('\n')}`);
            }
            if (abortSignal?.aborted) {
                throw new Error('部署已取消');
            }
            this.deployProgress.set(resolvedDeployId, { status: 'success', stages: [] });
            this.emit('deploy-complete', { deployId: resolvedDeployId, success: true });
            this.emit('deploy:notification', { success: true, deployId: resolvedDeployId });
            logEntry('deploy', 'complete', '部署成功完成');
            cleanupListeners();
            // Clean up progress entry after a delay to allow UI to read final state
            setTimeout(() => this.deployProgress.delete(resolvedDeployId), 60000);
            return { deployId: resolvedDeployId, success: true, logFilePath, artifactPaths };
        }
        catch (error) {
            const errMsg = getErrorMessage(error);
            const cancelled = abortSignal?.aborted || errMsg === '部署已取消';
            const status = cancelled ? 'cancelled' : 'failed';
            this.deployProgress.set(resolvedDeployId, { status, stages: [], error: errMsg });
            this.emit('deploy-complete', { deployId: resolvedDeployId, success: false, error: errMsg, cancelled });
            this.emit('deploy:notification', { success: false, deployId: resolvedDeployId, error: errMsg, cancelled });
            logEntry('deploy', status, `部署${cancelled ? '已取消' : '失败'}: ${errMsg}`);
            cleanupListeners();
            // Clean up progress entry after a delay
            setTimeout(() => this.deployProgress.delete(resolvedDeployId), 60000);
            throw error;
        }
    }
    // 单台服务器部署（供并行调用）
    async deploySingleServer(srv, artifacts, restartScript, abortSignal) {
        if (abortSignal?.aborted)
            throw new Error('部署已取消');
        const serverLabel = srv.label || `服务器`;
        this.emit('progress', {
            stage: 'ssh',
            status: 'starting',
            message: `正在部署到 ${serverLabel} (${srv.host})`
        });
        await this.sshDeploy(srv, artifacts, abortSignal);
        if (abortSignal?.aborted)
            throw new Error('部署已取消');
        if (restartScript) {
            this.emit('progress', {
                stage: 'restart',
                status: 'starting',
                message: `在 ${serverLabel} 执行重启脚本`
            });
            await this.executeRestartScript(srv, restartScript);
        }
    }
    // 收集构建产物
    async collectArtifacts(projectPath, modules, libSeparate) {
        const artifacts = [];
        const isModuleConfig = modules.length > 0 && typeof modules[0] === 'object';
        if (modules.length > 0 && isModuleConfig) {
            // ModuleConfig[] - use per-module output paths
            for (const mod of modules) {
                // Determine output path:
                // For Maven multi-module: build happens in buildPath (parent POM dir),
                // but artifacts land in modulePath/target/ (child module dir).
                // Strategy: use modulePath as artifact root, append outputPath (default 'target').
                let outputDir;
                if (mod.path) {
                    // modulePath is the primary anchor for finding artifacts
                    const artifactRoot = path.join(projectPath, mod.path);
                    outputDir = mod.outputPath
                        ? path.join(artifactRoot, mod.outputPath)
                        : path.join(artifactRoot, 'target');
                }
                else if (mod.buildPath) {
                    // Fallback: if no modulePath, use buildPath + outputPath
                    outputDir = mod.outputPath
                        ? path.join(projectPath, mod.buildPath, mod.outputPath)
                        : path.join(projectPath, mod.buildPath, 'target');
                }
                else {
                    outputDir = path.join(projectPath, mod.outputPath || 'target');
                }
                this.emit('progress', { stage: 'collect', status: 'info', message: `模块 ${mod.name || mod.path} 产物目录: ${outputDir}${mod.deployPath ? ' → 远程路径: ' + mod.deployPath : ''}` });
                if (!fs.existsSync(outputDir)) {
                    this.emit('progress', { stage: 'collect', status: 'warning', message: `模块 ${mod.name || mod.path} 的产物目录 ${outputDir} 不存在` });
                    continue;
                }
                // If artifactName is specified, look for that specific file
                if (mod.artifactName) {
                    const artifactPath = path.join(outputDir, mod.artifactName);
                    if (fs.existsSync(artifactPath)) {
                        const stat = fs.statSync(artifactPath);
                        this.emit('progress', { stage: 'collect', status: 'info', message: `  📦 ${mod.artifactName} (${(stat.size / 1024 / 1024).toFixed(1)}MB) → deploy: ${mod.deployPath || '默认'}` });
                        artifacts.push({
                            name: mod.artifactName,
                            localPath: artifactPath,
                            module: mod.name || mod.path,
                            isLib: false,
                            deployPath: mod.deployPath || undefined
                        });
                    }
                    else {
                        this.emit('progress', { stage: 'collect', status: 'warning', message: `产物 ${mod.artifactName} 不存在于 ${outputDir}` });
                    }
                    // Also collect lib/ if jar-plus-lib
                    if (mod.artifactType === 'jar-plus-lib') {
                        const libDir = path.join(outputDir, 'lib');
                        if (fs.existsSync(libDir) && fs.statSync(libDir).isDirectory()) {
                            const zipName = `${mod.name || mod.path}-lib.zip`.replace(/[\\/]/g, '_');
                            const zipPath = path.join(outputDir, zipName);
                            // Parse filter rules
                            const filterRules = mod.libFilterRules
                                ? mod.libFilterRules.split('\n').map(r => r.trim()).filter(r => r)
                                : [];
                            const filterDesc = filterRules.length > 0
                                ? ` (过滤: ${filterRules.join(', ')})`
                                : ' (全量)';
                            this.emit('progress', { stage: 'collect', status: 'info', message: `压缩 lib/${filterDesc} → ${zipName}` });
                            await this.compressDirectory(libDir, zipPath, filterRules);
                            const stat = fs.statSync(zipPath);
                            this.emit('progress', { stage: 'collect', status: 'info', message: `✓ ${zipName} (${(stat.size / 1024 / 1024).toFixed(1)}MB)` });
                            artifacts.push({
                                name: zipName,
                                localPath: zipPath,
                                module: mod.name || mod.path,
                                isLib: true,
                                isCompressed: true,
                                extractPath: libDir,
                                deployPath: mod.deployPath || undefined
                            });
                        }
                    }
                }
                else {
                    // Auto-detect or use explicit artifactType
                    const artifactType = mod.artifactType || '';
                    if (artifactType === 'dist') {
                        // Frontend: compress entire dist into a single zip to avoid SSH channel exhaustion
                        // Uploading hundreds of small files one-by-one opens too many SFTP channels
                        const buildOutputDir = outputDir;
                        if (fs.existsSync(buildOutputDir) && fs.statSync(buildOutputDir).isDirectory()) {
                            const zipName = `${mod.name || mod.path || 'dist'}.zip`.replace(/[\\/]/g, '_');
                            const zipPath = path.join(buildOutputDir, zipName);
                            this.emit('progress', { stage: 'collect', status: 'info', message: `压缩前端产物 ${path.basename(buildOutputDir)}/ → ${zipName}` });
                            await this.compressDirectory(buildOutputDir, zipPath);
                            const stat = fs.statSync(zipPath);
                            this.emit('progress', { stage: 'collect', status: 'info', message: `✓ ${zipName} (${(stat.size / 1024 / 1024).toFixed(1)}MB)` });
                            artifacts.push({
                                name: zipName,
                                localPath: zipPath,
                                module: mod.name || mod.path,
                                isLib: false,
                                isCompressed: true,
                                extractPath: buildOutputDir,
                                deployPath: mod.deployPath || undefined
                            });
                        }
                        else {
                            this.emit('progress', { stage: 'collect', status: 'warning', message: `前端产物目录不存在: ${buildOutputDir}` });
                        }
                    }
                    else if (artifactType === 'jar') {
                        // Single JAR only
                        const jars = this.findJars(outputDir, mod.name || mod.path);
                        if (jars.length > 0) {
                            this.emit('progress', { stage: 'collect', status: 'info', message: `  找到 ${jars.length} 个 JAR: ${jars.map(j => j.name).join(', ')}` });
                            for (const jar of jars) {
                                this.emit('progress', { stage: 'collect', status: 'info', message: `  📦 [jar] ${jar.name} → deploy: ${mod.deployPath || '默认'}` });
                                artifacts.push({
                                    name: jar.name,
                                    localPath: jar.localPath,
                                    module: mod.name || mod.path,
                                    isLib: false,
                                    deployPath: mod.deployPath || undefined
                                });
                            }
                        }
                    }
                    else if (artifactType === 'jar-plus-lib') {
                        // JAR + lib directory (thin-jar deployment)
                        this.emit('progress', { stage: 'collect', status: 'info', message: `[${mod.name || mod.path}] jar-plus-lib 模式: outputDir=${outputDir}, deployPath=${mod.deployPath || '默认'}` });
                        const jars = this.findJars(outputDir, mod.name || mod.path);
                        for (const jar of jars) {
                            this.emit('progress', { stage: 'collect', status: 'info', message: `  📦 [jar] ${jar.name} → deploy: ${mod.deployPath || '默认'}` });
                            artifacts.push({
                                name: jar.name,
                                localPath: jar.localPath,
                                module: mod.name || mod.path,
                                isLib: false,
                                deployPath: mod.deployPath || undefined
                            });
                        }
                        const libDir = path.join(outputDir, 'lib');
                        if (fs.existsSync(libDir) && fs.statSync(libDir).isDirectory()) {
                            const zipName = `${mod.name || mod.path}-lib.zip`.replace(/[\\/]/g, '_');
                            const zipPath = path.join(outputDir, zipName);
                            const filterRules = mod.libFilterRules
                                ? mod.libFilterRules.split('\n').map(r => r.trim()).filter(r => r)
                                : [];
                            const filterDesc = filterRules.length > 0
                                ? ` (过滤: ${filterRules.join(', ')})`
                                : ' (全量)';
                            this.emit('progress', { stage: 'collect', status: 'info', message: `压缩 lib/${filterDesc} → ${zipName}` });
                            await this.compressDirectory(libDir, zipPath, filterRules);
                            const stat = fs.statSync(zipPath);
                            this.emit('progress', { stage: 'collect', status: 'info', message: `  📦 [lib] ${zipName} (${(stat.size / 1024 / 1024).toFixed(1)}MB) → deploy: ${mod.deployPath || '默认'}/lib` });
                            artifacts.push({
                                name: zipName,
                                localPath: zipPath,
                                module: mod.name || mod.path,
                                isLib: true,
                                isCompressed: true,
                                extractPath: libDir,
                                deployPath: mod.deployPath || undefined
                            });
                        }
                    }
                    else {
                        // Auto-detect (legacy behavior)
                        const files = fs.readdirSync(outputDir);
                        const jars = files.filter(f => f.endsWith('.jar') && !f.includes('original'));
                        if (jars.length > 0) {
                            this.emit('progress', { stage: 'collect', status: 'info', message: `  [auto-detect] 找到 ${jars.length} 个 JAR: ${jars.join(', ')}` });
                            for (const jar of jars) {
                                const isLib = !!libSeparate && !jar.includes((mod.name || mod.path).split('/').pop());
                                this.emit('progress', { stage: 'collect', status: 'info', message: `  📦 ${isLib ? '[lib]' : '[jar]'} ${jar} → deploy: ${mod.deployPath || '默认'}${isLib ? '/lib' : ''}` });
                                artifacts.push({
                                    name: jar,
                                    localPath: path.join(outputDir, jar),
                                    module: mod.name || mod.path,
                                    isLib,
                                    deployPath: mod.deployPath || undefined
                                });
                            }
                        }
                        // Auto-detect: check outputDir itself first (outputPath IS the build output),
                        // then fall back to common subdirectories like dist/, build/, public/
                        // Frontend: compress into zip to avoid SSH channel exhaustion
                        const hasFilesInOutput = fs.readdirSync(outputDir).some(f => {
                            const fp = path.join(outputDir, f);
                            return fs.statSync(fp).isFile();
                        });
                        if (hasFilesInOutput) {
                            // outputDir itself contains files — treat it as build output root
                            // Only compress if it looks like frontend output (no JARs)
                            if (jars.length === 0) {
                                const dirName = path.basename(outputDir);
                                const zipName = `${mod.name || mod.path || dirName}-dist.zip`.replace(/[\\/]/g, '_');
                                const zipPath = path.join(outputDir, zipName);
                                this.emit('progress', { stage: 'collect', status: 'info', message: `压缩前端产物 ${dirName}/ → ${zipName}` });
                                await this.compressDirectory(outputDir, zipPath);
                                const stat = fs.statSync(zipPath);
                                this.emit('progress', { stage: 'collect', status: 'info', message: `✓ ${zipName} (${(stat.size / 1024 / 1024).toFixed(1)}MB)` });
                                artifacts.push({
                                    name: zipName,
                                    localPath: zipPath,
                                    module: mod.name || mod.path,
                                    isLib: false,
                                    isCompressed: true,
                                    extractPath: outputDir,
                                    deployPath: mod.deployPath || undefined
                                });
                            }
                        }
                        else {
                            // Look for common output subdirectories
                            const commonDirs = ['dist', 'build', 'public'];
                            let foundDir = '';
                            for (const d of commonDirs) {
                                const candidate = path.join(outputDir, d);
                                if (fs.existsSync(candidate) && fs.statSync(candidate).isDirectory()) {
                                    foundDir = candidate;
                                    break;
                                }
                            }
                            if (foundDir) {
                                const dirName = path.basename(foundDir);
                                const zipName = `${mod.name || mod.path || dirName}-dist.zip`.replace(/[\\/]/g, '_');
                                const zipPath = path.join(foundDir, zipName);
                                this.emit('progress', { stage: 'collect', status: 'info', message: `压缩前端产物 ${dirName}/ → ${zipName}` });
                                await this.compressDirectory(foundDir, zipPath);
                                const stat = fs.statSync(zipPath);
                                this.emit('progress', { stage: 'collect', status: 'info', message: `✓ ${zipName} (${(stat.size / 1024 / 1024).toFixed(1)}MB)` });
                                artifacts.push({
                                    name: zipName,
                                    localPath: zipPath,
                                    module: mod.name || mod.path,
                                    isLib: false,
                                    isCompressed: true,
                                    extractPath: foundDir,
                                    deployPath: mod.deployPath || undefined
                                });
                            }
                            if (jars.length === 0 && !foundDir) {
                                this.emit('progress', { stage: 'collect', status: 'warning', message: `模块 ${mod.name || mod.path} 未发现构建产物` });
                            }
                        }
                    }
                }
            }
        }
        else if (modules.length > 0) {
            // Legacy string[] modules
            for (const module of modules) {
                const modulePath = path.join(projectPath, module, 'target');
                if (!fs.existsSync(modulePath)) {
                    this.emit('progress', { stage: 'collect', status: 'warning', message: `模块 ${module} 的 target 目录不存在` });
                    continue;
                }
                const jars = fs.readdirSync(modulePath).filter(f => f.endsWith('.jar') && !f.includes('original'));
                for (const jar of jars) {
                    const isLib = !!libSeparate && !jar.includes(module.split('/').pop());
                    artifacts.push({
                        name: jar,
                        localPath: path.join(modulePath, jar),
                        module,
                        isLib
                    });
                }
            }
        }
        else {
            // Check for Java/Maven artifacts
            const targetPath = path.join(projectPath, 'target');
            if (fs.existsSync(targetPath)) {
                const jars = fs.readdirSync(targetPath).filter(f => f.endsWith('.jar') && !f.includes('original'));
                for (const jar of jars) {
                    artifacts.push({
                        name: jar,
                        localPath: path.join(targetPath, jar),
                        isLib: false
                    });
                }
            }
            // Check for npm/frontend artifacts — check common output dirs + project root itself
            const commonOutputDirs = ['dist', 'build', 'public'];
            let foundOutputDir = '';
            for (const d of commonOutputDirs) {
                const candidate = path.join(projectPath, d);
                if (fs.existsSync(candidate) && fs.statSync(candidate).isDirectory()) {
                    foundOutputDir = candidate;
                    break;
                }
            }
            // If no common output dir found, check if project root itself has build files
            // (some tools output directly to project root or a custom dir)
            if (!foundOutputDir) {
                const rootFiles = fs.readdirSync(projectPath);
                const hasBuildFiles = rootFiles.some(f => {
                    if (f === 'node_modules' || f === '.git' || f.startsWith('.'))
                        return false;
                    const fp = path.join(projectPath, f);
                    if (!fs.statSync(fp).isDirectory())
                        return f.endsWith('.html') || f.endsWith('.js') || f.endsWith('.css');
                    return false;
                });
                // Only use root if it has actual build output files and doesn't look like a source project
                // (i.e., no package.json or it has .html files suggesting a built output)
                if (hasBuildFiles && !fs.existsSync(path.join(projectPath, 'package.json'))) {
                    foundOutputDir = projectPath;
                }
            }
            if (foundOutputDir) {
                // Compress frontend output into a single zip to avoid SSH channel exhaustion
                const dirName = path.basename(foundOutputDir);
                const zipName = `${dirName}.zip`;
                const zipPath = path.join(foundOutputDir, zipName);
                this.emit('progress', { stage: 'collect', status: 'info', message: `压缩前端产物 ${dirName}/ → ${zipName}` });
                await this.compressDirectory(foundOutputDir, zipPath);
                const stat = fs.statSync(zipPath);
                this.emit('progress', { stage: 'collect', status: 'info', message: `✓ ${zipName} (${(stat.size / 1024 / 1024).toFixed(1)}MB)` });
                artifacts.push({
                    name: zipName,
                    localPath: zipPath,
                    isLib: false,
                    isCompressed: true,
                    extractPath: foundOutputDir
                });
            }
            else {
                // Last resort: check project root for any HTML/build files
                this.emit('progress', { stage: 'collect', status: 'warning', message: '未发现前端构建产物 (dist/、build/、public/)' });
            }
        }
        if (libSeparate) {
            const libPath = path.join(projectPath, 'target', 'lib');
            if (fs.existsSync(libPath)) {
                const libJars = fs.readdirSync(libPath).filter(f => f.endsWith('.jar'));
                this.emit('progress', { stage: 'collect', status: 'info', message: `[libSeparate] target/lib 下找到 ${libJars.length} 个依赖 JAR: ${libJars.join(', ')}` });
                for (const libJar of libJars) {
                    this.emit('progress', { stage: 'collect', status: 'info', message: `  📦 [lib] ${libJar} → deploy: 全局 libDir` });
                    artifacts.push({
                        name: libJar,
                        localPath: path.join(libPath, libJar),
                        isLib: true
                    });
                }
            }
        }
        return artifacts;
    }
    // Find JAR files in output directory, filtering out 'original' copies
    findJars(outputDir, moduleName) {
        const files = fs.readdirSync(outputDir);
        const jars = files.filter(f => f.endsWith('.jar') && !f.includes('original'));
        return jars.map(jar => ({
            name: jar,
            localPath: path.join(outputDir, jar),
        }));
    }
    // Recursively collect files from a directory
    collectDirectory(dirPath, relativeBase, artifacts) {
        const entries = fs.readdirSync(dirPath, { withFileTypes: true });
        for (const entry of entries) {
            const fullPath = path.join(dirPath, entry.name);
            const relativePath = path.join(relativeBase, entry.name);
            if (entry.isDirectory()) {
                this.collectDirectory(fullPath, relativePath, artifacts);
            }
            else {
                artifacts.push({
                    name: relativePath,
                    localPath: fullPath,
                    isLib: false
                });
            }
        }
    }
    // Compress a directory into a zip file using system zip command (async — non-blocking)
    // Supports optional filter rules (glob patterns, one per line)
    async compressDirectory(dirPath, zipPath, filterRules) {
        // Remove existing zip if present
        if (fs.existsSync(zipPath))
            fs.unlinkSync(zipPath);
        const q = (s) => "'" + s.replace(/'/g, "'\\''") + "'";
        if (filterRules && filterRules.length > 0) {
            // Build zip include patterns: -i "mall-*" -i "my-service-*"
            const findPatterns = filterRules.map(r => `-name '${r.replace(/'/g, "'\\''")}'`).join(' -o ');
            // Use find + zip to filter rules
            await (0, async_exec_1.runCommand)(`cd ${q(dirPath)} && find . -type f \\( ${findPatterns} \\) | zip -rq ${q(zipPath)} -@`, {
                timeout: 120000,
            });
        }
        else {
            // No filters - zip everything
            await (0, async_exec_1.runCommand)(`cd ${q(dirPath)} && zip -rq ${q(zipPath)} .`, {
                timeout: 120000,
            });
        }
    }
    // 获取仓库名称
    getRepoName(repoUrl) {
        const parts = repoUrl.split('/');
        const lastPart = parts[parts.length - 1];
        return lastPart.replace('.git', '');
    }
    // 获取部署进度
    getDeployProgress(deployId) {
        return this.deployProgress.get(deployId);
    }
    // 回滚到上一个版本
    async rollback(config) {
        const { host, port, username, password, privateKey, deployDir } = config;
        this.emit('progress', { stage: 'rollback', status: 'starting', message: `开始回滚到上一个版本` });
        return new Promise((resolve, reject) => {
            const conn = new ssh2_1.Client();
            conn.on('ready', async () => {
                this.emit('progress', { stage: 'rollback', status: 'connected', message: `已连接到 ${host}` });
                try {
                    let remoteHome = '';
                    try {
                        remoteHome = await this.sshExec(conn, 'echo $HOME').then(s => s.trim());
                    }
                    catch { /* fallback: tilde paths won't resolve */ }
                    const resolvedDeployDir = resolveRemotePath(deployDir, remoteHome);
                    const listBackups = `ls -dt ${shellEscape(resolvedDeployDir)}.bak* ${shellEscape(resolvedDeployDir)}_* 2>/dev/null | head -5`;
                    const backups = await this.sshExec(conn, listBackups);
                    const backupDirs = backups.trim().split('\n').filter(d => d.trim());
                    if (backupDirs.length === 0) {
                        this.emit('progress', { stage: 'rollback', status: 'info', message: '未找到备份目录，尝试 Git 回滚' });
                        try {
                            const gitLog = await this.sshExec(conn, `cd ${shellEscape(resolvedDeployDir)} && git log --format='%H' -2 2>/dev/null`);
                            const commits = gitLog.trim().split('\n').filter(c => c.trim());
                            if (commits.length >= 2) {
                                const prevCommit = commits[1].trim();
                                await this.sshExec(conn, `cd ${shellEscape(resolvedDeployDir)} && git reset --hard ${shellEscape(prevCommit)}`);
                                this.emit('progress', { stage: 'rollback', status: 'success', message: `Git 回滚到 ${prevCommit}` });
                                conn.end();
                                return resolve({ success: true, method: 'git', commit: prevCommit });
                            }
                        }
                        catch (gitError) {
                            this.emit('progress', { stage: 'rollback', status: 'warning', message: `Git 回滚失败: ${getErrorMessage(gitError)}` });
                        }
                        conn.end();
                        return reject(new Error('未找到可回滚的备份版本'));
                    }
                    const latestBackup = backupDirs[0].trim();
                    this.emit('progress', { stage: 'rollback', status: 'restoring', message: `恢复到备份: ${latestBackup}` });
                    const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
                    const currentBackup = `${deployDir}.pre_rollback_${timestamp}`;
                    await this.sshExec(conn, `cp -rp ${shellEscape(resolvedDeployDir)} ${shellEscape(resolveRemotePath(currentBackup, remoteHome))}`);
                    await this.sshExec(conn, `rm -rf ${shellEscape(resolvedDeployDir)} && cp -rp ${shellEscape(latestBackup)} ${shellEscape(resolvedDeployDir)}`);
                    this.emit('progress', { stage: 'rollback', status: 'success', message: `回滚完成，原版本已备份至 ${currentBackup}` });
                    conn.end();
                    resolve({ success: true, method: 'backup', restoredFrom: latestBackup, currentBackup });
                }
                catch (error) {
                    this.emit('progress', { stage: 'rollback', status: 'error', message: getErrorMessage(error) });
                    conn.end();
                    reject(error);
                }
            });
            conn.on('error', (err) => {
                this.emit('progress', { stage: 'rollback', status: 'error', message: err.message });
                conn.end();
                reject(err);
            });
            conn.connect({
                host,
                port: port || 22,
                username,
                password: password || undefined,
                privateKey: privateKey || undefined
            });
        });
    }
    // Rollback: deploy using saved artifacts (skip build step)
    async rollbackWithArtifacts(server, artifactDir, abortSignal) {
        this.emit('progress', { stage: 'rollback', status: 'starting', message: `开始回滚，使用历史产物: ${artifactDir}` });
        if (!fs.existsSync(artifactDir)) {
            throw new Error(`产物目录不存在: ${artifactDir}`);
        }
        // Read manifest
        const manifestPath = path.join(artifactDir, 'manifest.json');
        if (!fs.existsSync(manifestPath)) {
            throw new Error(`产物清单不存在: ${manifestPath}`);
        }
        const manifestArtifacts = JSON.parse(fs.readFileSync(manifestPath, 'utf-8'));
        if (!Array.isArray(manifestArtifacts))
            throw new Error('产物清单格式错误');
        if (manifestArtifacts.length === 0) {
            throw new Error('产物清单为空');
        }
        // Rebuild artifact objects for sshDeploy
        // ?? fallbacks for old-format manifests that lack these fields
        const artifactList = manifestArtifacts.map((a) => ({
            name: a.name,
            localPath: path.join(artifactDir, a.name),
            module: a.module,
            isLib: a.isLib ?? false,
            isCompressed: a.isCompressed,
            extractPath: a.extractPath,
            deployPath: a.deployPath,
        }));
        // Validate all artifact files exist before starting deployment
        for (const a of artifactList) {
            if (!fs.existsSync(a.localPath)) {
                throw new Error(`产物文件不存在: ${a.localPath}`);
            }
        }
        this.emit('progress', { stage: 'rollback', status: 'info', message: `找到 ${artifactList.length} 个历史产物` });
        await this.sshDeploy(server, artifactList, abortSignal);
        if (server && server.restartScript) {
            this.emit('progress', { stage: 'rollback', status: 'info', message: '执行重启脚本' });
            await this.executeRestartScript(server, server.restartScript);
        }
        this.emit('progress', { stage: 'rollback', status: 'success', message: '回滚完成' });
        return { success: true };
    }
}
module.exports = CicdService;
//# sourceMappingURL=cicd-service.js.map