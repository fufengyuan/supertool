import { initDatabase, getDatabase, closeDatabase } from './db-modules/db-core';
import type { Todo, Subtask, Project, CicdConfig, DeployModule, DeployLog, DeployStepLog, DeployHistory, Server, ServerGroup, User, Message, FileTransfer, ChatMessage, WeeklyReport, GitRepoRecord, ExportData, ApiRequest, AccountingCategoryRecord, AccountingRecord, AccountingBudget, AccountingTemplate, LogPresetRecord, OpenVPNConfigRecord } from './db-modules/db-core';
import _db_mfa = require('./db-modules/db-mfa');
import _db_notes = require('./db-modules/db-notes');
import _db_todos = require('./db-modules/db-todos');
declare const _default: {
    initDatabase: typeof initDatabase;
    getDatabase: typeof getDatabase;
    closeDatabase: typeof closeDatabase;
    initDefaultAccountingCategories: () => void;
    getAccountingCategories: () => AccountingCategoryRecord[];
    addAccountingCategory: (data: {
        id?: string;
        name: string;
        type?: string;
        icon?: string;
        sortOrder?: number;
    }) => AccountingCategoryRecord;
    updateAccountingCategory: (id: string, updates: {
        name?: string;
        type?: string;
        icon?: string;
        sortOrder?: number;
    }) => AccountingCategoryRecord | null;
    deleteAccountingCategory: (id: string) => {
        success: boolean;
        error?: string;
    };
    ensureAccountingReceiptsDir: () => string;
    generateVoucherNumber: (dateStr?: string) => string;
    getAccountingRecords: (options?: {
        startDate?: string;
        endDate?: string;
        type?: string;
        category?: string;
        status?: string;
        payment_method?: string;
        entity?: string;
        project?: string;
        page?: number;
        pageSize?: number;
        search?: string;
    }) => {
        records: AccountingRecord[];
        total: number;
    };
    addAccountingRecord: (data: {
        id?: string;
        date: string;
        type?: string;
        category?: string;
        amount: number;
        description?: string;
        status?: string;
        attachmentPath?: string;
        createdBy?: string;
        voucher_number?: string;
        receipt_type?: string;
        receipt_path?: string;
        entity?: string;
        project?: string;
        supplier?: string;
        invoice_number?: string;
        tax_amount?: number;
        payment_method?: string;
        approver?: string;
        attachments_json?: string;
    }) => AccountingRecord;
    updateAccountingRecord: (id: string, updates: {
        date?: string;
        type?: string;
        category?: string;
        amount?: number;
        description?: string;
        status?: string;
        attachmentPath?: string | null;
        voucher_number?: string;
        receipt_type?: string;
        receipt_path?: string;
        entity?: string;
        project?: string;
        supplier?: string;
        invoice_number?: string;
        tax_amount?: number;
        payment_method?: string;
        approver?: string;
        attachments_json?: string;
    }) => AccountingRecord | null;
    deleteAccountingRecord: (id: string) => boolean;
    getAccountingStats: (startDate?: string, endDate?: string) => {
        totalIncome: number;
        totalExpense: number;
        balance: number;
        pendingAmount: number;
        reimbursedAmount: number;
        byCategory: {
            category: string;
            amount: number;
        }[];
    };
    exportAccountingRecordsCSV: (options?: {
        startDate?: string;
        endDate?: string;
        type?: string;
        category?: string;
        status?: string;
        payment_method?: string;
        entity?: string;
        project?: string;
        search?: string;
    }) => string;
    getBudgets: () => AccountingBudget[];
    addBudget: (data: {
        category: string;
        amount: number;
        period?: string;
    }) => AccountingBudget;
    updateBudget: (id: string, updates: {
        category?: string;
        amount?: number;
        period?: string;
    }) => AccountingBudget | null;
    deleteBudget: (id: string) => {
        success: boolean;
        error?: string;
    };
    checkBudgetAlerts: () => {
        category: string;
        budget: number;
        spent: number;
        percent: number;
        over: boolean;
    }[];
    getTemplates: () => AccountingTemplate[];
    addTemplate: (data: {
        name: string;
        type: string;
        category: string;
        amount: number;
        description?: string;
        entity?: string;
        project?: string;
        supplier?: string;
        payment_method?: string;
        tax_rate?: number;
    }) => AccountingTemplate;
    updateTemplate: (id: string, updates: {
        name?: string;
        type?: string;
        category?: string;
        amount?: number;
        description?: string;
        entity?: string;
        project?: string;
        supplier?: string;
        payment_method?: string;
        tax_rate?: number;
    }) => AccountingTemplate | null;
    deleteTemplate: (id: string) => {
        success: boolean;
        error?: string;
    };
    useTemplate: (id: string) => {
        success: boolean;
        error?: string;
    };
    getAccountingTrend: (months?: number) => {
        month: string;
        income: number;
        expense: number;
        count: number;
    }[];
    saveWeeklyReport: (report: {
        startDate: string;
        endDate: string;
        data: Record<string, unknown>;
    }) => {
        success: boolean;
        id: number;
    };
    getWeeklyReports: (limit?: number) => any[];
    getWeeklyReport: (id: number) => WeeklyReport | null;
    exportAllData: () => ExportData;
    importAllData: (data: ExportData, mode?: "replace" | "merge") => {
        imported: number;
        skipped: number;
    };
    getAllUsers: () => User[];
    upsertUser: (user: User) => User;
    updateUserOnlineStatus: (userId: string, isOnline: boolean) => string;
    deleteUser: (userId: string) => string;
    rowToMessage: (row: any) => Message;
    getAllMessages: () => Message[];
    getMessagesWithUser: (userId: string, currentUserId: string) => Message[];
    addMessage: (message: Message) => Message;
    markMessageRead: (messageId: string) => string;
    getUnreadMessageCount: (userId: string) => number;
    rowToFileTransfer: (row: any) => FileTransfer;
    getAllFileTransfers: () => FileTransfer[];
    addFileTransfer: (transfer: FileTransfer) => FileTransfer;
    updateFileTransferProgress: (transferId: string, progress: number, status: string) => string;
    completeFileTransfer: (transferId: string, filePath: string) => string;
    deleteFileTransfer: (transferId: string) => string;
    saveChatMessage: (message: ChatMessage) => ChatMessage;
    getChatMessages: (limit?: number, offset?: number) => any[];
    getChatMessagesBetween: (userId1: string, userId2: string, limit?: number, offset?: number) => any[];
    markMessagesAsRead: (userId1: string, userId2: string) => number;
    getUnreadCount: (userId1: string, userId2: string) => number;
    getAllUnreadCounts: (myUserId: string) => Record<string, number>;
    rowToCicdConfig: (row: any) => CicdConfig;
    getAllCicdConfigs: () => CicdConfig[];
    getCicdGroups: () => string[];
    getCicdConfig: (projectId: string) => CicdConfig | null;
    getCicdConfigByConfigId: (configId: string) => CicdConfig | null;
    addCicdConfig: (config: CicdConfig) => CicdConfig | null;
    touchCicdConfigDeploy: (configId: string) => void;
    updateCicdConfig: (config: CicdConfig) => CicdConfig | null;
    deleteCicdConfig: (configId: string) => {
        success: boolean;
    };
    rowToDeployModule: (row: any) => DeployModule;
    getDeployModules: (configId: string) => DeployModule[];
    addDeployModule: (module: DeployModule) => DeployModule[];
    updateDeployModule: (module: DeployModule) => DeployModule[];
    deleteDeployModule: (moduleId: string) => {
        success: boolean;
    };
    rowToDeployLog: (row: any) => DeployLog;
    getDeployLogs: (projectId: string, limit?: number) => DeployLog[];
    getDeployLogById: (id: string) => DeployLog | undefined;
    addDeployLog: (log: DeployLog) => DeployLog;
    updateDeployLog: (log: {
        id: string;
        status?: string;
        endTime?: string;
        currentStep?: string;
        progress?: number;
        errorMessage?: string;
        logOutput?: string;
        logFilePath?: string;
        artifactPaths?: string;
    }) => {
        success: boolean;
    };
    rowToDeployStepLog: (row: any) => DeployStepLog;
    getDeployStepLogs: (deployLogId: string) => DeployStepLog[];
    addDeployStepLog: (stepLog: DeployStepLog) => {
        success: boolean;
    };
    updateDeployStepLog: (stepLog: {
        id: string;
        status?: string;
        endTime?: string;
        output?: string;
        errorMessage?: string;
    }) => {
        success: boolean;
    };
    rowToDeployHistory: (row: any) => DeployHistory;
    getDeployHistory: (projectId: string, limit?: number) => DeployHistory[];
    addDeployHistory: (record: DeployHistory) => DeployHistory;
    updateDeployRecord: (id: string, updates: {
        status?: string;
        version?: string;
        gitCommit?: string;
        deployedAt?: string;
        rolledBack?: boolean;
        rolledBackAt?: string;
    }) => {
        success: boolean;
    };
    rowToGitRepo: (row: any) => GitRepoRecord;
    getAllGitRepos: () => GitRepoRecord[];
    getGitRepoById: (id: string) => GitRepoRecord | null;
    addGitRepo: (repo: {
        id: string;
        name: string;
        path: string;
        remote?: string;
        branch?: string;
    }) => GitRepoRecord;
    updateGitRepo: (id: string, updates: Partial<GitRepoRecord>) => GitRepoRecord | null;
    deleteGitRepo: (id: string) => boolean;
    getAllMfaSecrets: typeof _db_mfa.getAllMfaSecrets;
    addMfaSecret: typeof _db_mfa.addMfaSecret;
    updateMfaSecret: typeof _db_mfa.updateMfaSecret;
    deleteMfaSecret: typeof _db_mfa.deleteMfaSecret;
    getAllNoteGroups: typeof _db_notes.getAllNoteGroups;
    addNoteGroup: typeof _db_notes.addNoteGroup;
    updateNoteGroup: typeof _db_notes.updateNoteGroup;
    deleteNoteGroup: typeof _db_notes.deleteNoteGroup;
    getAllNotes: typeof _db_notes.getAllNotes;
    getNoteById: typeof _db_notes.getNoteById;
    addNote: typeof _db_notes.addNote;
    updateNote: typeof _db_notes.updateNote;
    deleteNote: typeof _db_notes.deleteNote;
    getOpenVPNConfigs: () => OpenVPNConfigRecord[];
    addOpenVPNConfig: (name: string, filePath: string, content: string) => {
        id: string;
    };
    updateOpenVPNConfig: (id: string, updates: {
        name?: string;
        filePath?: string;
        content?: string;
    }) => void;
    deleteOpenVPNConfig: (id: string) => {
        success: boolean;
        error?: string;
    };
    rowToProject: (row: any) => Project;
    getAllProjects: (onlyActive?: boolean) => Project[];
    addProject: (project: Project) => Project;
    updateProject: (project: Project) => Project;
    deleteProject: (id: string) => string;
    getProjectStats: (projectId: string) => {
        total: number;
        completed: number;
        progress: number;
    };
    getTodosByProject: (projectId: string) => Todo[];
    rowToServer: (row: any) => Server;
    getAllServers: () => Server[];
    getServerById: (serverId: string) => Server | null;
    addServer: (server: Server) => Server | null;
    updateServer: (server: Server) => Server | null;
    deleteServer: (serverId: string) => {
        success: boolean;
    };
    rowToServerGroup: (row: any) => ServerGroup;
    getAllServerGroups: () => ServerGroup[];
    addServerGroup: (group: {
        id?: string;
        name: string;
        description?: string;
        parentId?: string | null;
        color?: string;
    }) => ServerGroup | undefined;
    updateServerGroup: (groupId: string, updates: {
        name?: string;
        description?: string;
        parentId?: string | null;
        color?: string;
    }) => ServerGroup | undefined;
    deleteServerGroup: (groupId: string) => {
        success: boolean;
    };
    rowToSubtask: (row: any) => Subtask;
    getSubtasksForTodo: (todoId: string) => Subtask[];
    addSubtask: (subtask: Subtask) => Subtask;
    updateSubtask: (subtask: Subtask) => Subtask;
    deleteSubtask: (subtaskId: string) => string;
    deleteSubtasksForTodo: (todoId: string) => string;
    updateTodoOrder: (todos: Todo[]) => boolean;
    calculateNextDate: (currentDate: string, repeatType: string, repeatInterval: number) => string;
    shouldCreateRepeatInstance: (todo: Todo) => boolean;
    createRepeatInstance: (originalTodo: Todo) => Todo | null;
    updateTodoCompletionBasedOnSubtasks: (todoId: string) => boolean;
    getAllTodos: typeof _db_todos.getAllTodos;
    addTodo: typeof _db_todos.addTodo;
    getTodoById: typeof _db_todos.getTodoById;
    updateTodo: typeof _db_todos.updateTodo;
    deleteTodo: typeof _db_todos.deleteTodo;
    deleteTodos: typeof _db_todos.deleteTodos;
    getAllTags: typeof _db_todos.getAllTags;
    addTag: typeof _db_todos.addTag;
    deleteTag: typeof _db_todos.deleteTag;
    getSetting: typeof _db_todos.getSetting;
    setSetting: typeof _db_todos.setSetting;
    getLogPresets: () => LogPresetRecord[];
    addLogPreset: (name: string, serverIds: string[], logPath: string, logType: string, keywords: string[], maxLines: number, presetGroup?: string) => {
        id: string;
    };
    updateLogPreset: (id: string, updates: {
        name?: string;
        presetGroup?: string;
        serverIds?: string[];
        logPath?: string;
        logType?: string;
        keywords?: string[];
        maxLines?: number;
    }) => void;
    deleteLogPreset: (id: string) => {
        success: boolean;
        error?: string;
    };
    getCalculatorHistory: (limit?: number) => Array<{
        id: string;
        expression: string;
        result: string;
        createdAt: string;
    }>;
    addCalculatorHistory: (expression: string, result: string) => {
        id: string;
        expression: string;
        result: string;
        createdAt: string;
    };
    clearCalculatorHistory: () => void;
    rowToApiRequest: (row: any) => ApiRequest;
    addApiRequest: (request: ApiRequest) => ApiRequest;
    getApiRequests: () => ApiRequest[];
    getApiRequestById: (id: string) => ApiRequest | null;
    updateApiRequest: (id: string, updates: Partial<ApiRequest>) => ApiRequest | null;
    deleteApiRequest: (id: string) => {
        success: boolean;
    };
};
export = _default;
