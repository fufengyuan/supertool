import { Client, SFTPWrapper } from 'ssh2';
import EventEmitter = require('events');
interface ServerConfig {
    host: string;
    port?: number;
    username: string;
    password?: string;
    privateKey?: Buffer | string;
    deployDir: string;
    libDir?: string | null;
}
interface Artifact {
    name: string;
    localPath: string;
    module?: string;
    isLib: boolean;
    isCompressed?: boolean;
    extractPath?: string;
    deployPath?: string;
}
interface ModuleConfig {
    name: string;
    path: string;
    buildPath?: string;
    buildCommand?: string;
    buildTool?: string;
    outputPath?: string;
    artifactName?: string;
    artifactType?: string;
    libFilterRules?: string;
    deployOrder?: number;
    deployPath?: string;
}
interface DeployConfig {
    repoUrl: string;
    branch: string;
    localPath?: string;
    buildTool?: string;
    buildCommand?: string;
    buildPath?: string;
    npmScript?: string;
    npmCustomScript?: string;
    mavenHome?: string;
    javaHome?: string;
    npmHome?: string;
    nodeHome?: string;
    modules?: ModuleConfig[] | string[];
    skipTests?: boolean;
    libSeparate?: boolean;
    parentBuildMode?: boolean;
    parentBuildPath?: string;
    mavenProfile?: string;
    mavenSettings?: string;
    servers: ServerConfig[];
    deployDir: string;
    libDir?: string | null;
    restartScript?: string;
}
interface ProgressEvent {
    stage: string;
    status: string;
    message: string;
}
declare class CicdService extends EventEmitter {
    private workDir;
    private deployProgress;
    constructor(userDataPath?: string);
    gitSync(repoUrl: string, branch: string, localPath?: string): Promise<{
        success: boolean;
        path: string;
    }>;
    mavenBuild(projectPath: string, mavenHome: string | undefined, modules?: string[], skipTests?: boolean, javaHome?: string): Promise<{
        success: boolean;
        output: string;
    }>;
    npmBuild(projectPath: string, tool?: string, script?: string, npmHome?: string, nodeHome?: string): Promise<{
        success: boolean;
        output: string;
    }>;
    gradleBuild(projectPath: string, modules?: string[], globalConfig?: DeployConfig): Promise<{
        success: boolean;
        output: string;
    }>;
    buildModule(projectPath: string, module: ModuleConfig, globalConfig: DeployConfig): Promise<{
        success: boolean;
        output: string;
    }>;
    sshDeploy(config: ServerConfig, artifacts: Artifact[], abortSignal?: AbortSignal): Promise<{
        success: boolean;
    }>;
    sshExec(conn: Client, command: string): Promise<string>;
    sshUpload(conn: Client, localPath: string, remotePath: string, onProgress?: (bytesTransferred: number, totalBytes: number) => void, cachedSftp?: SFTPWrapper): Promise<void>;
    private _doUpload;
    /** Get remote file size (returns 0 if file doesn't exist) */
    private _getRemoteFileInfo;
    /** Get remote file size and mtime via SFTP (avoids opening exec channels) */
    private _getRemoteFileInfoCached;
    /** Get remote file MD5 (returns empty string on failure) */
    private _getRemoteMD5;
    /** Verify a portion of the remote file matches the local file at the same offset */
    private _verifyPartialMatch;
    /** Compute MD5 of a specific byte range in a local file */
    private _computeLocalRangeMD5;
    private _streamUpload;
    private _doStreamUpload;
    private _computeLocalMD5;
    executeRestartScript(config: ServerConfig, scriptPath: string): Promise<{
        success: boolean;
        output: string;
    }>;
    deployFull(config: DeployConfig, dataDir?: string, abortSignal?: AbortSignal, deployId?: string): Promise<{
        deployId: string;
        success: boolean;
        logFilePath: string;
        artifactPaths: string[];
    }>;
    private deploySingleServer;
    collectArtifacts(projectPath: string, modules: string[] | ModuleConfig[], libSeparate?: boolean): Promise<Artifact[]>;
    private findJars;
    private collectDirectory;
    private compressDirectory;
    getRepoName(repoUrl: string): string;
    getDeployProgress(deployId: string): {
        status: string;
        stages: ProgressEvent[];
        error?: string;
    } | undefined;
    rollback(config: ServerConfig): Promise<{
        success: boolean;
        method?: string;
        restoredFrom?: string;
        currentBackup?: string;
        commit?: string;
    }>;
    rollbackWithArtifacts(server: ServerConfig, artifactDir: string, abortSignal?: AbortSignal): Promise<{
        success: boolean;
    }>;
}
export = CicdService;
