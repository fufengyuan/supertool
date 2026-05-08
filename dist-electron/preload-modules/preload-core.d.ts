/**
 * 核心类型定义 + 参数净化 + contextBridge 合并逻辑
 * 供所有 preload 模块共享使用
 */
export interface Todo {
    id: string;
    text: string;
    completed: boolean;
    priority?: string;
    dueDate?: string;
    description?: string;
    markdownDescription?: string;
    tag?: string;
    createdAt: string;
    updatedAt: string;
    completedAt?: string;
    orderNum?: number;
    repeatType?: string;
    repeatInterval?: number;
    repeatEndDate?: string;
    repeatCount?: number;
    parentTodoId?: string;
    projectId?: string;
    assignedTo?: string;
    assignedBy?: string;
    assignedAt?: string;
    owner?: string;
}
export interface Subtask {
    id: string;
    todoId: string;
    text: string;
    completed: boolean;
    orderNum?: number;
    createdAt: string;
    updatedAt: string;
}
export interface Project {
    id: string;
    name: string;
    description?: string;
    color?: string;
    repoPath?: string;
    branch?: string;
    gitUrl1?: string;
    gitUrl2?: string;
    category?: string;
    createdAt: string;
    updatedAt: string;
    archived?: boolean;
}
export interface Server {
    id: string;
    name: string;
    host: string;
    port?: number;
    username: string;
    sshKeyPath?: string;
    password?: string;
    description?: string;
    tags?: string[];
    groupId?: string;
    createdAt: string;
    updatedAt: string;
}
export interface CicdConfig {
    id: string;
    projectId: string;
    deployBranch?: string;
    mavenSettings?: string;
    mavenProfile?: string;
    deployPath: string;
    libSeparate?: boolean;
    restartScript?: string;
    healthCheckUrl?: string;
    healthCheckTimeout?: number;
    lastDeployedAt?: string;
    createdAt?: string;
    updatedAt?: string;
    groupName?: string;
}
export interface DeployModule {
    id: string;
    configId: string;
    moduleName: string;
    modulePath: string;
    artifactName?: string;
    deployOrder?: number;
    enabled?: boolean;
    createdAt: string;
    updatedAt: string;
}
export interface WeeklyReport {
    startDate: string;
    endDate: string;
    data: Record<string, unknown>;
}
export interface AutoBackupSettings {
    enabled?: boolean;
    frequency?: string;
    time?: string;
    path?: string;
}
export interface Peer {
    id: string;
    name: string;
    address: string;
    wsPort?: number;
    fileTransferPort?: number;
    wsUrl?: string;
    fileTransferUrl?: string;
    lastSeen?: number;
    online?: boolean;
}
export interface FileTransfer {
    id: string;
    fromUserId: string;
    fromUserName: string;
    toUserId: string;
    toUserName: string;
    fileName: string;
    fileSize: number;
    filePath?: string;
    status: string;
    progress: number;
    createdAt: string;
    completedAt?: string;
}
export interface LanMessage {
    type: string;
    from?: string;
    fromName?: string;
    to?: string;
    toName?: string;
    content?: string;
    timestamp?: number;
    [key: string]: unknown;
}
export interface ServerGroup {
    id: string;
    name: string;
    description?: string;
    createdAt: string;
    updatedAt: string;
}
export interface GitCommit {
    hash: string;
    date: string;
    author: string;
    message: string;
}
/** 全局 IPC 参数净化 —— 渲染进程侧 patch，根治 "An object could not be cloned" 错误 */
export declare function sanitizeArgs(fn: (...args: any[]) => any): (...args: any[]) => any;
/** 合并所有模块并暴露到 contextBridge */
export declare function exposeAPI(modules: Record<string, unknown>[]): void;
/** 设置原生拖拽事件监听 */
export declare function setupDragDrop(): void;
