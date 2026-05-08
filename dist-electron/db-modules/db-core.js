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
exports.db = void 0;
exports.initDatabase = initDatabase;
exports.getDatabase = getDatabase;
exports.migrateSchema = migrateSchema;
exports.closeDatabase = closeDatabase;
exports.rowToTodo = rowToTodo;
const logger_1 = require("../logger");
const Database = require("better-sqlite3");
const path = __importStar(require("path"));
const os = __importStar(require("os"));
// ============ 类型定义 ============
// 数据库迁移配置：定义每个表期望的列
// 新增版本时，在这里添加缺失的列，运行 migrateSchema() 会自动执行 ALTER TABLE
const EXPECTED_SCHEMA = {
    todos: {
        id: 'TEXT', text: 'TEXT', completed: 'INTEGER', priority: 'TEXT',
        dueDate: 'TEXT', description: 'TEXT', markdownDescription: 'TEXT',
        tag: 'TEXT', createdAt: 'TEXT', updatedAt: 'TEXT', completedAt: 'TEXT',
        assignedTo: 'TEXT', assignedBy: 'TEXT', assignedAt: 'TEXT',
        owner: 'TEXT', orderNum: 'INTEGER', repeatType: 'TEXT',
        repeatInterval: 'INTEGER', repeatEndDate: 'TEXT', repeatCount: 'INTEGER',
        parentTodoId: 'TEXT', projectId: 'TEXT',
    },
    tags: {
        id: 'INTEGER', name: 'TEXT', createdAt: 'TEXT',
    },
    settings: {
        key: 'TEXT', value: 'TEXT',
    },
    users: {
        id: 'TEXT', name: 'TEXT', ip: 'TEXT', port: 'INTEGER',
        lastSeen: 'TEXT', isOnline: 'INTEGER',
    },
    messages: {
        id: 'TEXT', fromUserId: 'TEXT', fromUserName: 'TEXT',
        toUserId: 'TEXT', toUserName: 'TEXT', content: 'TEXT',
        type: 'TEXT', createdAt: 'TEXT', read: 'INTEGER',
    },
    chat_messages: {
        id: 'TEXT', fromUserId: 'TEXT', fromUserName: 'TEXT',
        toUserId: 'TEXT', toUserName: 'TEXT', content: 'TEXT',
        type: 'TEXT', fileName: 'TEXT', fileSize: 'INTEGER',
        filePath: 'TEXT', status: 'TEXT', progress: 'INTEGER',
        createdAt: 'TEXT', read: 'INTEGER',
    },
    file_transfers: {
        id: 'TEXT', fromUserId: 'TEXT', fromUserName: 'TEXT',
        toUserId: 'TEXT', toUserName: 'TEXT', fileName: 'TEXT',
        fileSize: 'INTEGER', filePath: 'TEXT', status: 'TEXT',
        progress: 'INTEGER', createdAt: 'TEXT', completedAt: 'TEXT',
    },
    subtasks: {
        id: 'TEXT', todoId: 'TEXT', text: 'TEXT', completed: 'INTEGER',
        orderNum: 'INTEGER', createdAt: 'TEXT', updatedAt: 'TEXT',
    },
    projects: {
        id: 'TEXT', name: 'TEXT', description: 'TEXT', color: 'TEXT',
        repoPath: 'TEXT', branch: 'TEXT', repoPath2: 'TEXT', branch2: 'TEXT',
        gitUrl1: 'TEXT', gitUrl2: 'TEXT', category: 'TEXT', createdAt: 'TEXT', updatedAt: 'TEXT',
        archived: 'INTEGER',
    },
    cicd_configs: {
        id: 'TEXT', projectId: 'TEXT', deployBranch: 'TEXT',
        mavenSettings: 'TEXT', mavenProfile: 'TEXT',
        deployPath: 'TEXT', libSeparate: 'INTEGER',
        restartScript: 'TEXT', healthCheckUrl: 'TEXT',
        healthCheckTimeout: 'INTEGER', createdAt: 'TEXT', updatedAt: 'TEXT',
        buildTool: 'TEXT', buildCommand: 'TEXT', buildPath: 'TEXT',
        repoUrl: 'TEXT', localPath: 'TEXT', npmScript: 'TEXT',
        npmCustomScript: 'TEXT', mavenHome: 'TEXT', npmHome: 'TEXT', javaHome: 'TEXT',
        nodeHome: 'TEXT', servers: 'TEXT', groupName: 'TEXT',
    },
    deploy_modules: {
        id: 'TEXT', configId: 'TEXT', moduleName: 'TEXT',
        modulePath: 'TEXT', artifactName: 'TEXT', deployOrder: 'INTEGER',
        enabled: 'INTEGER', createdAt: 'TEXT', updatedAt: 'TEXT',
        buildCommand: 'TEXT', buildPath: 'TEXT', outputPath: 'TEXT',
        buildTool: 'TEXT',
    },
    deploy_logs: {
        id: 'TEXT', projectId: 'TEXT', configId: 'TEXT', status: 'TEXT',
        startTime: 'TEXT', endTime: 'TEXT', currentStep: 'TEXT',
        progress: 'INTEGER', errorMessage: 'TEXT', logOutput: 'TEXT',
        triggeredBy: 'TEXT', createdAt: 'TEXT',
        logFilePath: 'TEXT', artifactPaths: 'TEXT',
    },
    servers: {
        id: 'TEXT', name: 'TEXT', host: 'TEXT', port: 'INTEGER',
        username: 'TEXT', sshKeyPath: 'TEXT', password: 'TEXT',
        description: 'TEXT', tags: 'TEXT', groupId: 'TEXT',
        createdAt: 'TEXT', updatedAt: 'TEXT',
    },
    server_groups: {
        id: 'TEXT', name: 'TEXT', description: 'TEXT', parentId: 'TEXT', color: 'TEXT',
        createdAt: 'TEXT', updatedAt: 'TEXT',
    },
    weekly_reports: {
        id: 'INTEGER', startDate: 'TEXT', endDate: 'TEXT',
        data: 'TEXT', createdAt: 'TEXT',
    },
    deploy_history: {
        id: 'TEXT', configId: 'TEXT', projectId: 'TEXT', status: 'TEXT',
        version: 'TEXT', gitCommit: 'TEXT', deployedAt: 'TEXT',
        rolledBack: 'INTEGER', rolledBackAt: 'TEXT',
    },
    deploy_step_logs: {
        id: 'TEXT', deployLogId: 'TEXT', stepName: 'TEXT',
        stepOrder: 'INTEGER', status: 'TEXT', startTime: 'TEXT',
        endTime: 'TEXT', output: 'TEXT', errorMessage: 'TEXT',
        createdAt: 'TEXT',
    },
    mfa_secrets: {
        id: 'TEXT', name: 'TEXT', secret: 'TEXT', digits: 'INTEGER',
        period: 'INTEGER', algorithm: 'TEXT', account: 'TEXT',
        issuer: 'TEXT', createdAt: 'TEXT', updatedAt: 'TEXT',
    },
    note_groups: {
        id: 'TEXT', name: 'TEXT', icon: 'TEXT', sortOrder: 'INTEGER', createdAt: 'TEXT',
    },
    notes: {
        id: 'TEXT', title: 'TEXT', content: 'TEXT', tags: 'TEXT',
        pinned: 'INTEGER', createdAt: 'TEXT', updatedAt: 'TEXT',
        groupId: 'TEXT',
    },
    accounting_categories: {
        id: 'TEXT', name: 'TEXT', type: 'TEXT', icon: 'TEXT',
        sortOrder: 'INTEGER', createdAt: 'TEXT',
    },
    accounting_records: {
        id: 'TEXT', date: 'TEXT', type: 'TEXT', category: 'TEXT',
        amount: 'REAL', description: 'TEXT', status: 'TEXT',
        attachmentPath: 'TEXT', createdBy: 'TEXT',
        createdAt: 'TEXT', updatedAt: 'TEXT',
        voucher_number: 'TEXT', receipt_type: 'TEXT', receipt_path: 'TEXT',
        entity: 'TEXT', project: 'TEXT', supplier: 'TEXT',
        invoice_number: 'TEXT', tax_amount: 'REAL', payment_method: 'TEXT',
        approver: 'TEXT', attachments_json: 'TEXT',
    },
    calculator_history: {
        id: 'TEXT', expression: 'TEXT', result: 'TEXT',
        createdAt: 'TEXT',
    },
    git_repos: {
        id: 'TEXT', name: 'TEXT', path: 'TEXT', remote: 'TEXT',
        branch: 'TEXT', lastOpened: 'TEXT', createdAt: 'TEXT', updatedAt: 'TEXT',
    },
    api_requests: {
        id: 'TEXT', name: 'TEXT', method: 'TEXT',
        url: 'TEXT', headers: 'TEXT', body: 'TEXT',
        contentType: 'TEXT', createdAt: 'TEXT', updatedAt: 'TEXT',
    },
};
// 当前 schema 版本号（每次新增字段或表时递增）
const SCHEMA_VERSION = 8;
// 数据库文件路径
exports.db = null;
// 初始化数据库 — unified under ~/.supertool/
function initDatabase() {
    const dbPath = path.join(os.homedir(), '.supertool', 'supertool.db');
    (0, logger_1.info)('Database path:', dbPath);
    exports.db = new Database(dbPath);
    // SECURITY: Enable foreign key constraints
    exports.db.exec('PRAGMA foreign_keys = ON');
    // 创建表
    exports.db.exec(`
    CREATE TABLE IF NOT EXISTS todos (
      id TEXT PRIMARY KEY,
      text TEXT NOT NULL,
      completed INTEGER DEFAULT 0,
      priority TEXT DEFAULT 'medium',
      dueDate TEXT,
      description TEXT,
      markdownDescription TEXT,
      tag TEXT DEFAULT '',
      createdAt TEXT NOT NULL,
      updatedAt TEXT NOT NULL,
      completedAt TEXT,
      assignedTo TEXT DEFAULT '',
      assignedBy TEXT DEFAULT '',
      assignedAt TEXT,
      owner TEXT DEFAULT '',
      orderNum INTEGER DEFAULT 0,
      repeatType TEXT DEFAULT '',
      repeatInterval INTEGER DEFAULT 1,
      repeatEndDate TEXT,
      repeatCount INTEGER DEFAULT -1,
      parentTodoId TEXT,
      projectId TEXT DEFAULT NULL
    );
  `);
    // 统一迁移：检查所有表缺失列并自动添加（在所有 CREATE TABLE 之后执行）
    // 创建其他表
    exports.db.exec(`

    CREATE TABLE IF NOT EXISTS tags (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      name TEXT UNIQUE NOT NULL,
      createdAt TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS settings (
      key TEXT PRIMARY KEY,
      value TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS users (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      ip TEXT NOT NULL,
      port INTEGER NOT NULL,
      lastSeen TEXT NOT NULL,
      isOnline INTEGER DEFAULT 0
    );

    CREATE TABLE IF NOT EXISTS messages (
      id TEXT PRIMARY KEY,
      fromUserId TEXT NOT NULL,
      fromUserName TEXT NOT NULL,
      toUserId TEXT NOT NULL,
      toUserName TEXT NOT NULL,
      content TEXT NOT NULL,
      type TEXT DEFAULT 'text',
      createdAt TEXT NOT NULL,
      read INTEGER DEFAULT 0
    );

    CREATE TABLE IF NOT EXISTS chat_messages (
      id TEXT PRIMARY KEY,
      fromUserId TEXT NOT NULL,
      fromUserName TEXT NOT NULL,
      toUserId TEXT NOT NULL,
      toUserName TEXT NOT NULL,
      content TEXT,
      type TEXT DEFAULT 'text',
      fileName TEXT,
      fileSize INTEGER,
      filePath TEXT,
      status TEXT DEFAULT 'sent',
      progress INTEGER DEFAULT 0,
      createdAt TEXT NOT NULL,
      read INTEGER DEFAULT 0
    );

    CREATE TABLE IF NOT EXISTS file_transfers (
      id TEXT PRIMARY KEY,
      fromUserId TEXT NOT NULL,
      fromUserName TEXT NOT NULL,
      toUserId TEXT NOT NULL,
      toUserName TEXT NOT NULL,
      fileName TEXT NOT NULL,
      fileSize INTEGER NOT NULL,
      filePath TEXT,
      status TEXT DEFAULT 'pending',
      progress INTEGER DEFAULT 0,
      createdAt TEXT NOT NULL,
      completedAt TEXT,
      localUserId TEXT
    );

    CREATE TABLE IF NOT EXISTS subtasks (
      id TEXT PRIMARY KEY,
      todoId TEXT NOT NULL,
      text TEXT NOT NULL,
      completed INTEGER DEFAULT 0,
      orderNum INTEGER DEFAULT 0,
      createdAt TEXT NOT NULL,
      updatedAt TEXT NOT NULL,
      FOREIGN KEY (todoId) REFERENCES todos (id) ON DELETE CASCADE
    );
  `);
    // 创建projects表
    exports.db.exec(`
    CREATE TABLE IF NOT EXISTS projects (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      description TEXT,
      color TEXT DEFAULT '#6366f1',
      repoPath TEXT,
      branch TEXT,
      repoPath2 TEXT,
      branch2 TEXT,
      gitUrl1 TEXT,
      gitUrl2 TEXT,
      category TEXT,
      createdAt TEXT NOT NULL,
      updatedAt TEXT NOT NULL,
      archived INTEGER DEFAULT 0
    );
  `);
    // 迁移：为已有数据库添加 repoPath2 和 branch2 列
    try {
        exports.db.exec('ALTER TABLE projects ADD COLUMN repoPath2 TEXT');
    }
    catch { }
    try {
        exports.db.exec('ALTER TABLE projects ADD COLUMN branch2 TEXT');
    }
    catch { }
    // 迁移：为 file_transfers 添加 localUserId 列
    try {
        exports.db.exec('ALTER TABLE file_transfers ADD COLUMN localUserId TEXT');
    }
    catch { }
    // 迁移：为 cicd_configs 添加配置名称字段
    try {
        exports.db.exec('ALTER TABLE cicd_configs ADD COLUMN name TEXT');
    }
    catch { }
    // 迁移：为 cicd_configs 添加最后部署时间（用于排序）
    try {
        exports.db.exec('ALTER TABLE cicd_configs ADD COLUMN lastDeployedAt TEXT');
    }
    catch { }
    // 迁移：为 cicd_configs 添加构建配置字段
    try {
        exports.db.exec('ALTER TABLE cicd_configs ADD COLUMN buildTool TEXT');
    }
    catch { }
    try {
        exports.db.exec('ALTER TABLE cicd_configs ADD COLUMN buildCommand TEXT');
    }
    catch { }
    try {
        exports.db.exec('ALTER TABLE cicd_configs ADD COLUMN buildPath TEXT');
    }
    catch { }
    try {
        exports.db.exec('ALTER TABLE cicd_configs ADD COLUMN repoUrl TEXT');
    }
    catch { }
    try {
        exports.db.exec('ALTER TABLE cicd_configs ADD COLUMN localPath TEXT');
    }
    catch { }
    try {
        exports.db.exec('ALTER TABLE cicd_configs ADD COLUMN npmScript TEXT');
    }
    catch { }
    try {
        exports.db.exec('ALTER TABLE cicd_configs ADD COLUMN npmCustomScript TEXT');
    }
    catch { }
    try {
        exports.db.exec('ALTER TABLE cicd_configs ADD COLUMN mavenHome TEXT');
    }
    catch { }
    try {
        exports.db.exec('ALTER TABLE cicd_configs ADD COLUMN npmHome TEXT');
    }
    catch { }
    try {
        exports.db.exec('ALTER TABLE cicd_configs ADD COLUMN javaHome TEXT');
    }
    catch { }
    try {
        exports.db.exec('ALTER TABLE cicd_configs ADD COLUMN nodeHome TEXT');
    }
    catch { }
    try {
        exports.db.exec('ALTER TABLE cicd_configs ADD COLUMN servers TEXT');
    }
    catch { } // JSON array of multi-server configs
    // 迁移：确保 deployPath 列存在（旧表可能缺少此列）
    try {
        exports.db.exec('ALTER TABLE cicd_configs ADD COLUMN deployPath TEXT DEFAULT \'/\'');
    }
    catch { }
    // 迁移：添加分组字段
    try {
        exports.db.exec('ALTER TABLE cicd_configs ADD COLUMN groupName TEXT DEFAULT \'未分组\'');
    }
    catch { }
    // 迁移：为 cicd_configs 添加父子模块构建模式字段
    try {
        exports.db.exec('ALTER TABLE cicd_configs ADD COLUMN parentBuildMode INTEGER DEFAULT 0');
    }
    catch { }
    try {
        exports.db.exec('ALTER TABLE cicd_configs ADD COLUMN parentBuildPath TEXT DEFAULT \'\'');
    }
    catch { }
    // 迁移：为 cicd_configs 添加部署审核开关
    try {
        exports.db.exec('ALTER TABLE cicd_configs ADD COLUMN requiresApproval INTEGER DEFAULT 0');
    }
    catch { }
    // 迁移：为 deploy_modules 添加模块构建配置字段
    try {
        exports.db.exec('ALTER TABLE deploy_modules ADD COLUMN buildCommand TEXT');
    }
    catch { }
    try {
        exports.db.exec('ALTER TABLE deploy_modules ADD COLUMN buildPath TEXT');
    }
    catch { }
    try {
        exports.db.exec('ALTER TABLE deploy_modules ADD COLUMN outputPath TEXT');
    }
    catch { }
    try {
        exports.db.exec('ALTER TABLE deploy_modules ADD COLUMN buildTool TEXT');
    }
    catch { }
    try {
        exports.db.exec('ALTER TABLE deploy_modules ADD COLUMN artifactType TEXT');
    }
    catch { }
    try {
        exports.db.exec('ALTER TABLE deploy_modules ADD COLUMN libFilterRules TEXT');
    }
    catch { }
    try {
        exports.db.exec('ALTER TABLE deploy_modules ADD COLUMN deployPath TEXT');
    }
    catch { }
    // 清理旧字段：开发阶段直接移除过时的 name 列
    try {
        exports.db.exec('ALTER TABLE deploy_modules RENAME COLUMN name TO moduleName');
    }
    catch {
        try {
            exports.db.exec('ALTER TABLE deploy_modules DROP COLUMN name');
        }
        catch { }
    }
    // 创建CI/CD配置表
    exports.db.exec(`
    CREATE TABLE IF NOT EXISTS cicd_configs (
      id TEXT PRIMARY KEY,
      projectId TEXT NOT NULL,
      name TEXT,
      deployBranch TEXT DEFAULT 'main',
      mavenSettings TEXT,
      mavenProfile TEXT DEFAULT 'prod',
      deployPath TEXT NOT NULL,
      libSeparate INTEGER DEFAULT 1,
      restartScript TEXT DEFAULT './restart.sh',
      healthCheckUrl TEXT,
      healthCheckTimeout INTEGER DEFAULT 30,
      createdAt TEXT NOT NULL,
      updatedAt TEXT NOT NULL,
      groupName TEXT DEFAULT '未分组',
      parentBuildMode INTEGER DEFAULT 0,
      parentBuildPath TEXT DEFAULT '',
      requiresApproval INTEGER DEFAULT 0,
      FOREIGN KEY (projectId) REFERENCES projects (id) ON DELETE CASCADE
    );
  `);
    // 创建部署模块表
    exports.db.exec(`
    CREATE TABLE IF NOT EXISTS deploy_modules (
      id TEXT PRIMARY KEY,
      configId TEXT NOT NULL,
      moduleName TEXT NOT NULL,
      modulePath TEXT NOT NULL,
      artifactName TEXT,
      deployOrder INTEGER DEFAULT 0,
      deployPath TEXT,
      enabled INTEGER DEFAULT 1,
      createdAt TEXT NOT NULL,
      updatedAt TEXT NOT NULL,
      libFilterRules TEXT,
      FOREIGN KEY (configId) REFERENCES cicd_configs (id) ON DELETE CASCADE
    );
  `);
    // 创建部署日志表
    exports.db.exec(`
    CREATE TABLE IF NOT EXISTS deploy_logs (
      id TEXT PRIMARY KEY,
      projectId TEXT NOT NULL,
      configId TEXT NOT NULL,
      status TEXT DEFAULT 'pending',
      startTime TEXT,
      endTime TEXT,
      currentStep TEXT,
      progress INTEGER DEFAULT 0,
      errorMessage TEXT,
      logOutput TEXT,
      triggeredBy TEXT DEFAULT 'manual',
      createdAt TEXT NOT NULL,
      logFilePath TEXT,
      artifactPaths TEXT,
      FOREIGN KEY (projectId) REFERENCES projects (id) ON DELETE CASCADE,
      FOREIGN KEY (configId) REFERENCES cicd_configs (id) ON DELETE CASCADE
    );
  `);
    // 创建服务器管理表
    exports.db.exec(`
    CREATE TABLE IF NOT EXISTS servers (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      host TEXT NOT NULL,
      port INTEGER DEFAULT 22,
      username TEXT NOT NULL,
      sshKeyPath TEXT,
      password TEXT,
      description TEXT,
      tags TEXT,
      groupId TEXT,
      createdAt TEXT NOT NULL,
      updatedAt TEXT NOT NULL
    );
  `);
    // 创建服务器分组表
    exports.db.exec(`
    CREATE TABLE IF NOT EXISTS server_groups (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      description TEXT,
      parentId TEXT,
      color TEXT DEFAULT '#6c63ff',
      createdAt TEXT NOT NULL,
      updatedAt TEXT NOT NULL
    );
  `);
    // 迁移：为 server_groups 添加 parentId 和 color 字段
    try {
        exports.db.exec('ALTER TABLE server_groups ADD COLUMN parentId TEXT');
    }
    catch { }
    try {
        exports.db.exec('ALTER TABLE server_groups ADD COLUMN color TEXT DEFAULT \'#6c63ff\'');
    }
    catch { }
    // 迁移：为 log_presets 添加分组字段
    try {
        exports.db.exec('ALTER TABLE log_presets ADD COLUMN presetGroup TEXT DEFAULT \'未分组\'');
    }
    catch { }
    // 迁移：为 servers 添加 requiresApproval 字段
    try {
        exports.db.exec('ALTER TABLE servers ADD COLUMN requiresApproval INTEGER DEFAULT 0');
    }
    catch { }
    // 创建周报历史存档表
    exports.db.exec(`
    CREATE TABLE IF NOT EXISTS weekly_reports (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      startDate TEXT NOT NULL,
      endDate TEXT NOT NULL,
      data TEXT NOT NULL,
      createdAt TEXT NOT NULL
    );
  `);
    // 创建部署历史表（用于回滚追踪）
    exports.db.exec(`
    CREATE TABLE IF NOT EXISTS deploy_history (
      id TEXT PRIMARY KEY,
      configId TEXT NOT NULL,
      projectId TEXT NOT NULL,
      status TEXT DEFAULT 'pending',
      version TEXT,
      gitCommit TEXT,
      deployedAt TEXT,
      rolledBack INTEGER DEFAULT 0,
      rolledBackAt TEXT,
      FOREIGN KEY (projectId) REFERENCES projects (id) ON DELETE CASCADE,
      FOREIGN KEY (configId) REFERENCES cicd_configs (id) ON DELETE CASCADE
    );
  `);
    // 创建部署步骤日志表
    exports.db.exec(`
    CREATE TABLE IF NOT EXISTS deploy_step_logs (
      id TEXT PRIMARY KEY,
      deployLogId TEXT NOT NULL,
      stepName TEXT NOT NULL,
      stepOrder INTEGER DEFAULT 0,
      status TEXT DEFAULT 'pending',
      startTime TEXT,
      endTime TEXT,
      output TEXT,
      errorMessage TEXT,
      createdAt TEXT NOT NULL,
      FOREIGN KEY (deployLogId) REFERENCES deploy_logs (id) ON DELETE CASCADE
    );
  `);
    // MFA secrets table
    exports.db.exec(`
    CREATE TABLE IF NOT EXISTS mfa_secrets (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      secret TEXT NOT NULL,
      digits INTEGER DEFAULT 6,
      period INTEGER DEFAULT 30,
      algorithm TEXT DEFAULT 'sha1',
      account TEXT DEFAULT '',
      issuer TEXT DEFAULT '',
      createdAt TEXT NOT NULL,
      updatedAt TEXT NOT NULL
    );
  `);
    // Notes table
    exports.db.exec(`
    CREATE TABLE IF NOT EXISTS notes (
      id TEXT PRIMARY KEY,
      title TEXT NOT NULL DEFAULT '',
      content TEXT NOT NULL DEFAULT '',
      tags TEXT NOT NULL DEFAULT '[]',
      pinned INTEGER DEFAULT 0,
      groupId TEXT,
      createdAt TEXT NOT NULL,
      updatedAt TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS note_groups (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL DEFAULT '',
      icon TEXT NOT NULL DEFAULT '',
      sortOrder INTEGER DEFAULT 0,
      createdAt TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS accounting_budgets (
      id TEXT PRIMARY KEY,
      category TEXT NOT NULL,
      amount REAL NOT NULL DEFAULT 0,
      period TEXT NOT NULL DEFAULT 'monthly',
      createdAt TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS accounting_templates (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      type TEXT NOT NULL DEFAULT 'expense',
      category TEXT NOT NULL,
      amount REAL NOT NULL DEFAULT 0,
      description TEXT DEFAULT '',
      entity TEXT DEFAULT '',
      project TEXT DEFAULT '',
      supplier TEXT DEFAULT '',
      payment_method TEXT DEFAULT '',
      tax_rate REAL DEFAULT 0,
      useCount INTEGER DEFAULT 0,
      createdAt TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS accounting_categories (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL DEFAULT '',
      type TEXT NOT NULL DEFAULT 'expense',
      icon TEXT NOT NULL DEFAULT '',
      sortOrder INTEGER DEFAULT 0,
      createdAt TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS accounting_records (
      id TEXT PRIMARY KEY,
      date TEXT NOT NULL,
      type TEXT NOT NULL DEFAULT 'expense',
      category TEXT NOT NULL DEFAULT '',
      amount REAL NOT NULL DEFAULT 0,
      description TEXT DEFAULT '',
      status TEXT DEFAULT 'completed',
      attachmentPath TEXT,
      createdBy TEXT DEFAULT '',
      createdAt TEXT NOT NULL,
      updatedAt TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS openvpn_configs (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL DEFAULT '',
      filePath TEXT NOT NULL DEFAULT '',
      content TEXT NOT NULL,
      createdAt TEXT NOT NULL,
      updatedAt TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS calculator_history (
      id TEXT PRIMARY KEY,
      expression TEXT NOT NULL,
      result TEXT NOT NULL,
      createdAt TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS log_presets (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      presetGroup TEXT NOT NULL DEFAULT '未分组',
      serverIds TEXT NOT NULL,
      logPath TEXT NOT NULL,
      logType TEXT NOT NULL DEFAULT 'file',
      keywords TEXT DEFAULT '[]',
      maxLines INTEGER DEFAULT 500,
      createdAt TEXT NOT NULL,
      updatedAt TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS api_requests (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL DEFAULT '',
      method TEXT NOT NULL DEFAULT 'GET',
      url TEXT NOT NULL,
      headers TEXT NOT NULL DEFAULT '{}',
      body TEXT,
      contentType TEXT DEFAULT 'none',
      createdAt TEXT NOT NULL,
      updatedAt TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS git_repos (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      path TEXT NOT NULL UNIQUE,
      remote TEXT,
      branch TEXT,
      lastOpened TEXT,
      createdAt TEXT NOT NULL,
      updatedAt TEXT NOT NULL
    );
  `);
    // 统一迁移：检查所有表缺失列并自动添加
    migrateSchema();
    // ========== 性能优化：为常用查询字段添加索引 ==========
    try {
        exports.db.exec(`
      CREATE INDEX IF NOT EXISTS idx_todos_status ON todos (completed);
      CREATE INDEX IF NOT EXISTS idx_todos_priority ON todos (priority);
      CREATE INDEX IF NOT EXISTS idx_todos_projectId ON todos (projectId);
      CREATE INDEX IF NOT EXISTS idx_todos_tag ON todos (tag);
      CREATE INDEX IF NOT EXISTS idx_todos_createdAt ON todos (createdAt);
      CREATE INDEX IF NOT EXISTS idx_todos_updatedAt ON todos (updatedAt);
      CREATE INDEX IF NOT EXISTS idx_todos_status_createdAt ON todos (completed, createdAt DESC);
      CREATE INDEX IF NOT EXISTS idx_todos_projectId_status ON todos (projectId, completed);
      CREATE INDEX IF NOT EXISTS idx_messages_toUserId ON messages (toUserId);
      CREATE INDEX IF NOT EXISTS idx_messages_read ON messages (toUserId, read);
      CREATE INDEX IF NOT EXISTS idx_subtasks_todoId ON subtasks (todoId);
      CREATE INDEX IF NOT EXISTS idx_deploy_logs_projectId ON deploy_logs (projectId);
      CREATE INDEX IF NOT EXISTS idx_servers_groupId ON servers (groupId);
      CREATE UNIQUE INDEX IF NOT EXISTS idx_accounting_categories_name_type ON accounting_categories (name, type);
      CREATE INDEX IF NOT EXISTS idx_accounting_records_date ON accounting_records (date DESC);
      CREATE INDEX IF NOT EXISTS idx_accounting_records_type ON accounting_records (type);
    `);
    }
    catch (err) {
        console.warn('Failed to create indexes:', err.message);
    }
    // 迁移：为 deploy_logs 添加日志文件路径和制品路径列
    try {
        exports.db.exec('ALTER TABLE deploy_logs ADD COLUMN logFilePath TEXT');
    }
    catch { }
    try {
        exports.db.exec('ALTER TABLE deploy_logs ADD COLUMN artifactPaths TEXT');
    }
    catch { }
    // 插入默认标签
    const defaultTags = ['工作', '生活', '学习', '其他'];
    const insertTag = exports.db.prepare('INSERT OR IGNORE INTO tags (name, createdAt) VALUES (?, ?)');
    defaultTags.forEach(tag => {
        insertTag.run(tag, new Date().toISOString());
    });
    return exports.db;
}
// 获取数据库实例
function getDatabase() {
    if (!exports.db) {
        initDatabase();
    }
    return exports.db;
}
// 迁移数据库 schema：检查所有表是否缺少列，自动添加
// 使用 schema_version 表跟踪当前版本，避免重复执行
function migrateSchema() {
    const database = getDatabase();
    // 创建版本追踪表
    database.exec(`
    CREATE TABLE IF NOT EXISTS schema_version (
      version INTEGER PRIMARY KEY
    );
  `);
    const versionRow = database.prepare('SELECT MAX(version) as v FROM schema_version').get();
    const currentVersion = versionRow?.v ?? 0;
    if (currentVersion >= SCHEMA_VERSION)
        return; // 已是最新
    (0, logger_1.info)(`[DB Migration] Upgrading schema from v${currentVersion} to v${SCHEMA_VERSION}...`);
    // 获取数据库中所有表
    const tables = database.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name != 'schema_version'").all();
    let totalAdded = 0;
    for (const { name: tableName } of tables) {
        const expected = EXPECTED_SCHEMA[tableName];
        if (!expected)
            continue; // 不在迁移清单中的表跳过
        // 获取当前表的列
        const columns = database.pragma(`table_info(${tableName})`);
        const existingColumns = new Set(columns.map(c => c.name));
        // 添加缺失的列
        for (const [colName, colType] of Object.entries(expected)) {
            if (!existingColumns.has(colName)) {
                try {
                    // SQLite ALTER TABLE ADD COLUMN 需要默认值来填充已有行
                    let defaultClause = '';
                    if (colType === 'INTEGER')
                        defaultClause = ' DEFAULT 0';
                    else if (colType === 'TEXT')
                        defaultClause = " DEFAULT ''";
                    database.exec(`ALTER TABLE ${tableName} ADD COLUMN ${colName} ${colType}${defaultClause}`);
                    (0, logger_1.info)(`  [DB Migration] Added column '${colName} ${colType}' to ${tableName}`);
                    totalAdded++;
                }
                catch (err) {
                    // SQLite 限制：某些情况无法添加（如 PRIMARY KEY、NOT NULL 无默认值等）
                    console.warn(`  [DB Migration] Failed to add '${colName}' to ${tableName}:`, err.message);
                }
            }
        }
    }
    // 更新版本号
    database.prepare('INSERT INTO schema_version (version) VALUES (?)').run(SCHEMA_VERSION);
    (0, logger_1.info)(`[DB Migration] Schema upgraded to v${SCHEMA_VERSION} (${totalAdded} columns added)`);
}
// 关闭数据库
function closeDatabase() {
    if (exports.db) {
        exports.db.close();
        exports.db = null;
    }
}
// ============ 辅助函数 ============
function rowToTodo(row) {
    return {
        id: row.id,
        text: row.text,
        completed: row.completed === 1,
        priority: row.priority,
        dueDate: row.dueDate,
        description: row.description,
        markdownDescription: row.markdownDescription,
        tag: row.tag,
        createdAt: row.createdAt,
        updatedAt: row.updatedAt,
        completedAt: row.completedAt,
        orderNum: row.orderNum,
        repeatType: row.repeatType,
        repeatInterval: row.repeatInterval,
        repeatEndDate: row.repeatEndDate,
        repeatCount: row.repeatCount,
        parentTodoId: row.parentTodoId,
        projectId: row.projectId,
        assignedTo: row.assignedTo,
        assignedBy: row.assignedBy,
        assignedAt: row.assignedAt,
        owner: row.owner
    };
}
//# sourceMappingURL=db-core.js.map