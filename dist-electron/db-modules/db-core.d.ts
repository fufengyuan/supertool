import Database = require('better-sqlite3');
export interface Todo {
    id: string;
    text: string;
    completed: boolean;
    priority: string;
    dueDate?: string;
    description: string;
    markdownDescription: string;
    tag: string;
    createdAt: string;
    updatedAt: string;
    completedAt?: string;
    assignedTo: string;
    assignedBy: string;
    assignedAt?: string;
    owner: string;
    orderNum: number;
    repeatType: string;
    repeatInterval: number;
    repeatEndDate?: string;
    repeatCount: number;
    parentTodoId?: string;
    projectId?: string;
}
export interface Subtask {
    id: string;
    todoId: string;
    text: string;
    completed: boolean;
    orderNum: number;
    createdAt: string;
    updatedAt: string;
}
export interface Project {
    id: string;
    name: string;
    description: string;
    color: string;
    repoPath?: string;
    branch?: string;
    repoPath2?: string;
    branch2?: string;
    gitUrl1?: string;
    gitUrl2?: string;
    category?: string;
    createdAt: string;
    updatedAt: string;
    archived: boolean;
}
export interface CicdConfig {
    id: string;
    projectId: string;
    name?: string;
    deployBranch: string;
    mavenSettings?: string;
    mavenProfile: string;
    mavenHome?: string;
    npmHome?: string;
    javaHome?: string;
    nodeHome?: string;
    servers?: string;
    deployPath: string;
    libSeparate: boolean;
    restartScript: string;
    healthCheckUrl?: string;
    healthCheckTimeout: number;
    createdAt: string;
    updatedAt: string;
    groupName?: string;
    buildTool?: string;
    buildCommand?: string;
    parentBuildMode?: boolean;
    parentBuildPath?: string;
    buildPath?: string;
    repoUrl?: string;
    localPath?: string;
    npmScript?: string;
    npmCustomScript?: string;
    lastDeployedAt?: string;
    requiresApproval?: boolean;
}
export interface DeployModule {
    id: string;
    configId: string;
    moduleName: string;
    modulePath: string;
    artifactName?: string;
    artifactType?: string;
    deployOrder: number;
    deployPath?: string;
    enabled: boolean;
    createdAt: string;
    updatedAt: string;
    buildCommand?: string;
    buildPath?: string;
    outputPath?: string;
    buildTool?: string;
    libFilterRules?: string;
}
export interface DeployLog {
    id: string;
    projectId: string;
    configId: string;
    status: string;
    startTime?: string;
    endTime?: string;
    currentStep?: string;
    progress: number;
    errorMessage?: string;
    logOutput?: string;
    triggeredBy: string;
    createdAt: string;
    logFilePath?: string;
    artifactPaths?: string;
}
export interface DeployStepLog {
    id: string;
    deployLogId: string;
    stepName: string;
    stepOrder: number;
    status: string;
    startTime?: string;
    endTime?: string;
    output?: string;
    errorMessage?: string;
    createdAt: string;
}
export interface DeployHistory {
    id: string;
    configId: string;
    projectId: string;
    status: string;
    version?: string;
    gitCommit?: string;
    deployedAt: string;
    rolledBack: boolean;
    rolledBackAt?: string;
}
export interface Server {
    id: string;
    name: string;
    host: string;
    port: number;
    username: string;
    sshKeyPath?: string;
    password?: string;
    description: string;
    tags: string[];
    groupId?: string;
    requiresApproval?: boolean;
    createdAt: string;
    updatedAt: string;
}
export interface ServerGroup {
    id: string;
    name: string;
    description: string;
    parentId: string | null;
    color?: string;
    createdAt: string;
    updatedAt: string;
}
export interface User {
    id: string;
    name: string;
    ip: string;
    port: number;
    lastSeen: string;
    isOnline: boolean;
}
export interface Message {
    id: string;
    fromUserId: string;
    fromUserName: string;
    toUserId: string;
    toUserName: string;
    content: string;
    type: string;
    createdAt: string;
    read: boolean;
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
export interface ChatMessage {
    id: string;
    fromUserId: string;
    fromUserName: string;
    toUserId: string;
    toUserName: string;
    content?: string;
    type: string;
    fileName?: string;
    fileSize?: number;
    filePath?: string;
    status: string;
    progress: number;
    createdAt: string;
    read?: boolean;
}
export interface WeeklyReport {
    startDate: string;
    endDate: string;
    data: Record<string, unknown>;
}
export interface GitCommit {
    hash: string;
    date: string;
    author: string;
    message: string;
}
export interface GitRepo {
    path: string;
    name: string;
    relativePath: string;
    url: string;
}
export interface GitRepoRecord {
    id: string;
    name: string;
    path: string;
    remote?: string;
    branch?: string;
    lastOpened?: string;
    createdAt: string;
    updatedAt: string;
}
export interface ExportData {
    version: string;
    exportedAt: string;
    todos: Todo[];
    subtasks: Subtask[];
    tags: string[];
    settings: Record<string, string>;
    projects: Project[];
    notes: any[];
    noteGroups: any[];
    mfaSecrets: any[];
    servers: Server[];
    serverGroups: ServerGroup[];
    cicdConfigs: CicdConfig[];
    deployModules: any[];
    deployLogs: any[];
    deployHistory: any[];
    deployStepLogs: any[];
    weeklyReports: any[];
    users: any[];
    messages: any[];
    chatMessages: any[];
    fileTransfers: any[];
    accountingCategories: any[];
    accountingRecords: any[];
    accountingBudgets: any[];
    accountingTemplates: any[];
}
export interface CalculatorHistory {
    id: string;
    expression: string;
    result: string;
    createdAt: string;
}
export interface ApiRequest {
    id: string;
    name: string;
    method: string;
    url: string;
    headers: string;
    body?: string;
    contentType: string;
    createdAt: string;
    updatedAt: string;
}
export interface AccountingCategoryRecord {
    id: string;
    name: string;
    type: string;
    icon: string;
    sortOrder: number;
    createdAt: string;
}
export interface AccountingRecord {
    id: string;
    date: string;
    type: string;
    category: string;
    amount: number;
    description: string;
    status: string;
    attachmentPath: string | null;
    createdBy: string;
    createdAt: string;
}
export interface AccountingBudget {
    id: string;
    category: string;
    amount: number;
    period: string;
    createdAt: string;
}
export interface AccountingTemplate {
    id: string;
    name: string;
    type: string;
    category: string;
    amount: number;
    description: string;
    entity: string;
    project: string;
    supplier: string;
    payment_method: string;
    tax_rate: number;
    createdAt: string;
}
export interface LogPresetRecord {
    id: string;
    name: string;
    presetGroup: string;
    serverIds: string;
    logPath: string;
    logType: string;
    keywords: string;
    maxLines: number;
    createdAt: string;
    updatedAt: string;
}
export interface OpenVPNConfigRecord {
    id: string;
    name: string;
    filePath: string;
    content: string;
    createdAt: string;
    updatedAt: string;
}
export interface NoteRecord {
    id: string;
    title: string;
    content: string;
    tags: string;
    pinned: number;
    groupId: string | null;
    createdAt: string;
    updatedAt: string;
}
export interface NoteGroupRecord {
    id: string;
    name: string;
    icon: string;
    sortOrder: number;
    createdAt: string;
}
export interface MfaSecretRecord {
    id: string;
    name: string;
    secret: string;
    digits: number;
    period: number;
    algorithm: string;
    account: string;
    issuer: string;
    createdAt: string;
    updatedAt: string;
}
export declare let db: Database.Database | null;
export declare function initDatabase(): Database.Database;
export declare function getDatabase(): Database.Database;
export declare function migrateSchema(): void;
export declare function closeDatabase(): void;
export declare function rowToTodo(row: any): Todo;
