"use strict";
const db_core_1 = require("./db-core");
const _db_todos = require("./db-todos");
const _db_subtasks = require("./db-subtasks");
const _db_projects = require("./db-projects");
const _db_notes = require("./db-notes");
const _db_mfa = require("./db-mfa");
const _db_servers = require("./db-servers");
const _db_cicd = require("./db-cicd");
const _db_chat = require("./db-chat");
function saveWeeklyReport(report) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    INSERT INTO weekly_reports (startDate, endDate, data, createdAt)
    VALUES (?, ?, ?, ?)
  `);
    stmt.run(report.startDate, report.endDate, typeof report.data === 'string' ? report.data : JSON.stringify(report.data), new Date().toISOString());
    return { success: true, id: stmt.lastInsertRowid };
}
function getWeeklyReports(limit = 20) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    SELECT id, startDate, endDate, createdAt FROM weekly_reports
    ORDER BY createdAt DESC
    LIMIT ?
  `);
    return stmt.all(limit);
}
function getWeeklyReport(id) {
    const stmt = (0, db_core_1.getDatabase)().prepare('SELECT * FROM weekly_reports WHERE id = ?');
    const row = stmt.get(id);
    if (!row)
        return null;
    return {
        ...row,
        data: typeof row.data === 'string' ? JSON.parse(row.data) : row.data
    };
}
function exportAllData() {
    const todos = _db_todos.getAllTodos();
    const tags = _db_todos.getAllTags();
    const subtasks = [];
    for (const todo of todos) {
        const todoSubtasks = _db_subtasks.getSubtasksForTodo(todo.id);
        subtasks.push(...todoSubtasks);
    }
    const settings = {};
    const allSettings = (0, db_core_1.getDatabase)().prepare('SELECT * FROM settings').all();
    for (const s of allSettings) {
        settings[s.key] = s.value;
    }
    const projects = _db_projects.getAllProjects(false);
    const notes = _db_notes.getAllNotes();
    const noteGroups = _db_notes.getAllNoteGroups();
    const mfaSecrets = _db_mfa.getAllMfaSecrets();
    const servers = _db_servers.getAllServers();
    const serverGroups = _db_servers.getAllServerGroups();
    const cicdConfigs = _db_cicd.getAllCicdConfigs();
    const deployModules = (0, db_core_1.getDatabase)().prepare('SELECT * FROM deploy_modules').all();
    const deployLogs = (0, db_core_1.getDatabase)().prepare('SELECT * FROM deploy_logs').all();
    const deployHistory = (0, db_core_1.getDatabase)().prepare('SELECT * FROM deploy_history').all();
    const deployStepLogs = (0, db_core_1.getDatabase)().prepare('SELECT * FROM deploy_step_logs').all();
    const weeklyReports = (0, db_core_1.getDatabase)().prepare('SELECT * FROM weekly_reports').all();
    const users = _db_chat.getAllUsers();
    const messages = _db_chat.getAllMessages();
    const chatMessages = (0, db_core_1.getDatabase)().prepare('SELECT * FROM chat_messages').all();
    const fileTransfers = _db_chat.getAllFileTransfers();
    const accountingCategories = (0, db_core_1.getDatabase)().prepare('SELECT * FROM accounting_categories').all();
    const accountingRecords = (0, db_core_1.getDatabase)().prepare('SELECT * FROM accounting_records').all();
    const accountingBudgets = (0, db_core_1.getDatabase)().prepare('SELECT * FROM accounting_budgets').all();
    const accountingTemplates = (0, db_core_1.getDatabase)().prepare('SELECT * FROM accounting_templates').all();
    return {
        version: '3.0',
        exportedAt: new Date().toISOString(),
        todos, subtasks, tags, settings, projects,
        notes, noteGroups, mfaSecrets,
        servers, serverGroups,
        cicdConfigs, deployModules, deployLogs, deployHistory, deployStepLogs,
        weeklyReports, users, messages, chatMessages, fileTransfers,
        accountingCategories, accountingRecords,
        accountingBudgets, accountingTemplates,
    };
}
function importAllData(data, mode = 'merge') {
    let imported = 0;
    let skipped = 0;
    if (mode === 'replace') {
        // Clear all tables in reverse dependency order
        (0, db_core_1.getDatabase)().exec(`
      DELETE FROM deploy_step_logs;
      DELETE FROM deploy_logs;
      DELETE FROM deploy_modules;
      DELETE FROM cicd_configs;
      DELETE FROM deploy_history;
      DELETE FROM chat_messages;
      DELETE FROM file_transfers;
      DELETE FROM messages;
      DELETE FROM subtasks;
      DELETE FROM notes;
      DELETE FROM note_groups;
      DELETE FROM mfa_secrets;
      DELETE FROM servers;
      DELETE FROM server_groups;
      DELETE FROM weekly_reports;
      DELETE FROM todos;
      DELETE FROM projects;
      DELETE FROM users;
      DELETE FROM tags;
      DELETE FROM settings;
      DELETE FROM accounting_records;
      DELETE FROM accounting_categories;
      DELETE FROM accounting_budgets;
      DELETE FROM accounting_templates;
    `);
    }
    const db = (0, db_core_1.getDatabase)();
    // Settings (merge only, replace clears above)
    if (data.settings) {
        for (const [key, value] of Object.entries(data.settings)) {
            if (mode === 'merge') {
                const existing = db.prepare('SELECT key FROM settings WHERE key = ?').get(key);
                if (existing) {
                    skipped++;
                    continue;
                }
            }
            db.prepare('INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)').run(key, value);
            imported++;
        }
    }
    // Projects
    if (data.projects) {
        for (const p of data.projects) {
            if (mode === 'merge') {
                const existing = db.prepare('SELECT id FROM projects WHERE id = ?').get(p.id);
                if (existing) {
                    skipped++;
                    continue;
                }
            }
            db.prepare(`INSERT INTO projects (id, name, description, color, repoPath, branch, gitUrl1, gitUrl2, category, createdAt, updatedAt, archived)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`).run(p.id, p.name, p.description || '', p.color || '', p.repoPath || null, p.branch || null, p.gitUrl1 || null, p.gitUrl2 || null, p.category || null, p.createdAt, p.updatedAt, p.archived ? 1 : 0);
            imported++;
        }
    }
    // Todos
    if (data.todos) {
        for (const t of data.todos) {
            if (mode === 'merge') {
                const existing = db.prepare('SELECT id FROM todos WHERE id = ?').get(t.id);
                if (existing) {
                    skipped++;
                    continue;
                }
            }
            db.prepare(`INSERT INTO todos (id, text, completed, priority, dueDate, description, markdownDescription, tag, createdAt, updatedAt, completedAt, assignedTo, assignedBy, assignedAt, owner, orderNum, repeatType, repeatInterval, repeatEndDate, repeatCount, parentTodoId, projectId)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`).run(t.id, t.text, t.completed ? 1 : 0, t.priority, t.dueDate || null, t.description || '', t.markdownDescription || null, t.tag || '', t.createdAt, t.updatedAt, t.completedAt || null, t.assignedTo || null, t.assignedBy || null, t.assignedAt || null, t.owner || '', t.orderNum || 0, t.repeatType || null, t.repeatInterval || 0, t.repeatEndDate || null, t.repeatCount || 0, t.parentTodoId || null, t.projectId || null);
            imported++;
        }
    }
    // Subtasks
    if (data.subtasks) {
        for (const s of data.subtasks) {
            if (mode === 'merge') {
                const existing = db.prepare('SELECT id FROM subtasks WHERE id = ?').get(s.id);
                if (existing) {
                    skipped++;
                    continue;
                }
            }
            db.prepare('INSERT INTO subtasks (id, todoId, text, completed, orderNum, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?, ?)').run(s.id, s.todoId, s.text, s.completed ? 1 : 0, s.orderNum || 0, s.createdAt, s.updatedAt);
            imported++;
        }
    }
    // Tags
    if (data.tags) {
        const existingTags = _db_todos.getAllTags();
        for (const tag of data.tags) {
            if (existingTags.includes(tag)) {
                skipped++;
                continue;
            }
            db.prepare('INSERT OR IGNORE INTO tags (name) VALUES (?)').run(tag);
            imported++;
        }
    }
    // Notes
    if (data.notes) {
        for (const n of data.notes) {
            if (mode === 'merge') {
                const existing = db.prepare('SELECT id FROM notes WHERE id = ?').get(n.id);
                if (existing) {
                    skipped++;
                    continue;
                }
            }
            db.prepare('INSERT INTO notes (id, title, content, tags, pinned, groupId, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?, ?, ?)').run(n.id, n.title || '', n.content || '', n.tags || '[]', n.pinned || 0, n.groupId || null, n.createdAt, n.updatedAt);
            imported++;
        }
    }
    // Note Groups
    if (data.noteGroups) {
        for (const g of data.noteGroups) {
            if (mode === 'merge') {
                const existing = db.prepare('SELECT id FROM note_groups WHERE id = ?').get(g.id);
                if (existing) {
                    skipped++;
                    continue;
                }
            }
            db.prepare('INSERT INTO note_groups (id, name, icon, sortOrder, createdAt) VALUES (?, ?, ?, ?, ?)').run(g.id, g.name, g.icon || '', g.sortOrder || 0, g.createdAt);
            imported++;
        }
    }
    // MFA Secrets
    if (data.mfaSecrets) {
        for (const m of data.mfaSecrets) {
            if (mode === 'merge') {
                const existing = db.prepare('SELECT id FROM mfa_secrets WHERE id = ?').get(m.id);
                if (existing) {
                    skipped++;
                    continue;
                }
            }
            db.prepare('INSERT INTO mfa_secrets (id, name, secret, digits, period, algorithm, account, issuer, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)').run(m.id, m.name, m.secret, m.digits || 6, m.period || 30, m.algorithm || 'sha1', m.account || '', m.issuer || '', m.createdAt, m.updatedAt);
            imported++;
        }
    }
    // Servers
    if (data.servers) {
        for (const s of data.servers) {
            if (mode === 'merge') {
                const existing = db.prepare('SELECT id FROM servers WHERE id = ?').get(s.id);
                if (existing) {
                    skipped++;
                    continue;
                }
            }
            db.prepare('INSERT INTO servers (id, name, host, port, username, sshKeyPath, password, description, tags, groupId, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)').run(s.id, s.name, s.host, s.port, s.username, s.sshKeyPath || null, s.password || null, s.description || '', Array.isArray(s.tags) ? s.tags.join(',') : (s.tags || ''), s.groupId || null, s.createdAt, s.updatedAt);
            imported++;
        }
    }
    // Server Groups
    if (data.serverGroups) {
        for (const g of data.serverGroups) {
            if (mode === 'merge') {
                const existing = db.prepare('SELECT id FROM server_groups WHERE id = ?').get(g.id);
                if (existing) {
                    skipped++;
                    continue;
                }
            }
            db.prepare('INSERT INTO server_groups (id, name, description, parentId, color, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?, ?)').run(g.id, g.name, g.description || '', g.parentId || null, g.color || null, g.createdAt, g.updatedAt);
            imported++;
        }
    }
    // CICD Configs
    if (data.cicdConfigs) {
        for (const c of data.cicdConfigs) {
            if (mode === 'merge') {
                const existing = db.prepare('SELECT id FROM cicd_configs WHERE id = ?').get(c.id);
                if (existing) {
                    skipped++;
                    continue;
                }
            }
            db.prepare('INSERT INTO cicd_configs (id, projectId, name, deployBranch, mavenSettings, mavenProfile, deployPath, libSeparate, restartScript, healthCheckUrl, healthCheckTimeout, createdAt, updatedAt, groupName) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)').run(c.id, c.projectId, c.name || null, c.deployBranch || '', c.mavenSettings || null, c.mavenProfile || '', c.deployPath, c.libSeparate ? 1 : 0, c.restartScript || '', c.healthCheckUrl || null, c.healthCheckTimeout || 30, c.createdAt, c.updatedAt, c.groupName || '未分组');
            imported++;
        }
    }
    // Deploy Modules
    if (data.deployModules) {
        for (const d of data.deployModules) {
            if (mode === 'merge') {
                const existing = db.prepare('SELECT id FROM deploy_modules WHERE id = ?').get(d.id);
                if (existing) {
                    skipped++;
                    continue;
                }
            }
            db.prepare('INSERT INTO deploy_modules (id, configId, moduleName, modulePath, artifactName, deployOrder, enabled, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)').run(d.id, d.configId, d.moduleName, d.modulePath || '', d.artifactName || '', d.deployOrder || 0, d.enabled ? 1 : 0, d.createdAt, d.updatedAt);
            imported++;
        }
    }
    // Deploy Logs
    if (data.deployLogs) {
        for (const d of data.deployLogs) {
            if (mode === 'merge') {
                const existing = db.prepare('SELECT id FROM deploy_logs WHERE id = ?').get(d.id);
                if (existing) {
                    skipped++;
                    continue;
                }
            }
            db.prepare('INSERT INTO deploy_logs (id, projectId, configId, status, startTime, endTime, currentStep, progress, errorMessage, logOutput, triggeredBy, createdAt, logFilePath, artifactPaths) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)').run(d.id, d.projectId, d.configId, d.status, d.startTime || '', d.endTime || '', d.currentStep || '', d.progress || 0, d.errorMessage || '', d.logOutput || '', d.triggeredBy || '', d.createdAt, d.logFilePath || '', d.artifactPaths || '');
            imported++;
        }
    }
    // Deploy History
    if (data.deployHistory) {
        for (const d of data.deployHistory) {
            if (mode === 'merge') {
                const existing = db.prepare('SELECT id FROM deploy_history WHERE id = ?').get(d.id);
                if (existing) {
                    skipped++;
                    continue;
                }
            }
            db.prepare('INSERT INTO deploy_history (id, configId, projectId, status, version, gitCommit, deployedAt, rolledBack, rolledBackAt) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)').run(d.id, d.configId, d.projectId, d.status, d.version || '', d.gitCommit || '', d.deployedAt || '', d.rolledBack ? 1 : 0, d.rolledBackAt || null);
            imported++;
        }
    }
    // Deploy Step Logs
    if (data.deployStepLogs) {
        for (const d of data.deployStepLogs) {
            if (mode === 'merge') {
                const existing = db.prepare('SELECT id FROM deploy_step_logs WHERE id = ?').get(d.id);
                if (existing) {
                    skipped++;
                    continue;
                }
            }
            db.prepare('INSERT INTO deploy_step_logs (id, deployLogId, stepName, stepOrder, status, startTime, endTime, output, errorMessage, createdAt) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)').run(d.id, d.deployLogId, d.stepName, d.stepOrder || 0, d.status, d.startTime || '', d.endTime || '', d.output || '', d.errorMessage || '', d.createdAt);
            imported++;
        }
    }
    // Weekly Reports
    if (data.weeklyReports) {
        for (const w of data.weeklyReports) {
            if (mode === 'merge') {
                const existing = db.prepare('SELECT id FROM weekly_reports WHERE id = ?').get(w.id);
                if (existing) {
                    skipped++;
                    continue;
                }
            }
            db.prepare('INSERT INTO weekly_reports (id, startDate, endDate, data, createdAt) VALUES (?, ?, ?, ?, ?)').run(w.id, w.startDate, w.endDate, w.data, w.createdAt);
            imported++;
        }
    }
    // Users
    if (data.users) {
        for (const u of data.users) {
            if (mode === 'merge') {
                const existing = db.prepare('SELECT id FROM users WHERE id = ?').get(u.id);
                if (existing) {
                    skipped++;
                    continue;
                }
            }
            db.prepare('INSERT INTO users (id, name, ip, port, lastSeen, isOnline) VALUES (?, ?, ?, ?, ?, ?)').run(u.id, u.name, u.ip, u.port, u.lastSeen, u.isOnline ? 1 : 0);
            imported++;
        }
    }
    // Messages
    if (data.messages) {
        for (const m of data.messages) {
            if (mode === 'merge') {
                const existing = db.prepare('SELECT id FROM messages WHERE id = ?').get(m.id);
                if (existing) {
                    skipped++;
                    continue;
                }
            }
            db.prepare('INSERT INTO messages (id, fromUserId, fromUserName, toUserId, toUserName, content, type, createdAt, read) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)').run(m.id, m.fromUserId, m.fromUserName, m.toUserId, m.toUserName, m.content, m.type, m.createdAt, m.read ? 1 : 0);
            imported++;
        }
    }
    // Chat Messages
    if (data.chatMessages) {
        for (const c of data.chatMessages) {
            if (mode === 'merge') {
                const existing = db.prepare('SELECT id FROM chat_messages WHERE id = ?').get(c.id);
                if (existing) {
                    skipped++;
                    continue;
                }
            }
            db.prepare('INSERT INTO chat_messages (id, fromUserId, fromUserName, toUserId, toUserName, content, type, fileName, fileSize, filePath, status, progress, createdAt, read) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)').run(c.id, c.fromUserId, c.fromUserName, c.toUserId, c.toUserName, c.content, c.type, c.fileName || null, c.fileSize || 0, c.filePath || null, c.status || '', c.progress || 0, c.createdAt, c.read ? 1 : 0);
            imported++;
        }
    }
    // File Transfers
    if (data.fileTransfers) {
        for (const f of data.fileTransfers) {
            if (mode === 'merge') {
                const existing = db.prepare('SELECT id FROM file_transfers WHERE id = ?').get(f.id);
                if (existing) {
                    skipped++;
                    continue;
                }
            }
            // Check if localUserId column exists
            const hasLocalUserId = db.prepare("SELECT COUNT(*) as c FROM pragma_table_info('file_transfers') WHERE name='localUserId'").get();
            if (hasLocalUserId.c > 0) {
                db.prepare('INSERT INTO file_transfers (id, fromUserId, fromUserName, toUserId, toUserName, fileName, fileSize, filePath, status, progress, createdAt, completedAt, localUserId) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)').run(f.id, f.fromUserId, f.fromUserName, f.toUserId, f.toUserName, f.fileName, f.fileSize, f.filePath || null, f.status, f.progress || 0, f.createdAt, f.completedAt || null, f.localUserId || null);
            }
            else {
                db.prepare('INSERT INTO file_transfers (id, fromUserId, fromUserName, toUserId, toUserName, fileName, fileSize, filePath, status, progress, createdAt, completedAt) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)').run(f.id, f.fromUserId, f.fromUserName, f.toUserId, f.toUserName, f.fileName, f.fileSize, f.filePath || null, f.status, f.progress || 0, f.createdAt, f.completedAt || null);
            }
            imported++;
        }
    }
    // Accounting Categories
    if (data.accountingCategories) {
        for (const c of data.accountingCategories) {
            if (mode === 'merge') {
                const existing = db.prepare('SELECT id FROM accounting_categories WHERE id = ?').get(c.id);
                if (existing) {
                    skipped++;
                    continue;
                }
            }
            db.prepare('INSERT INTO accounting_categories (id, name, type, icon, sortOrder, createdAt) VALUES (?, ?, ?, ?, ?, ?)').run(c.id, c.name, c.type || 'expense', c.icon || '', c.sortOrder || 0, c.createdAt);
            imported++;
        }
    }
    // Accounting Records
    if (data.accountingRecords) {
        for (const r of data.accountingRecords) {
            if (mode === 'merge') {
                const existing = db.prepare('SELECT id FROM accounting_records WHERE id = ?').get(r.id);
                if (existing) {
                    skipped++;
                    continue;
                }
            }
            db.prepare(`INSERT INTO accounting_records (id, date, type, category, amount, description, status, attachmentPath, createdBy, createdAt, updatedAt, voucher_number, entity, project, supplier, invoice_number, tax_amount, payment_method, approver, attachments_json) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`).run(r.id, r.date, r.type || 'expense', r.category || '', r.amount || 0, r.description || '', r.status || 'completed', r.attachmentPath || null, r.createdBy || '', r.createdAt, r.updatedAt, r.voucher_number || '', r.entity || '', r.project || '', r.supplier || '', r.invoice_number || '', r.tax_amount || 0, r.payment_method || '', r.approver || '', r.attachments_json || '[]');
            imported++;
        }
    }
    // Accounting Budgets
    if (data.accountingBudgets) {
        for (const b of data.accountingBudgets) {
            if (mode === 'merge') {
                const existing = db.prepare('SELECT id FROM accounting_budgets WHERE id = ?').get(b.id);
                if (existing) {
                    skipped++;
                    continue;
                }
            }
            db.prepare('INSERT INTO accounting_budgets (id, category, amount, period, createdAt) VALUES (?, ?, ?, ?, ?)').run(b.id, b.category, b.amount, b.period || 'monthly', b.createdAt);
            imported++;
        }
    }
    // Accounting Templates
    if (data.accountingTemplates) {
        for (const t of data.accountingTemplates) {
            if (mode === 'merge') {
                const existing = db.prepare('SELECT id FROM accounting_templates WHERE id = ?').get(t.id);
                if (existing) {
                    skipped++;
                    continue;
                }
            }
            db.prepare('INSERT INTO accounting_templates (id, name, type, category, amount, description, entity, project, supplier, payment_method, tax_rate, useCount, createdAt) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)').run(t.id, t.name, t.type, t.category, t.amount, t.description || '', t.entity || '', t.project || '', t.supplier || '', t.payment_method || '', t.tax_rate || 0, t.useCount || 0, t.createdAt);
            imported++;
        }
    }
    return { imported, skipped };
}
module.exports = {
    saveWeeklyReport,
    getWeeklyReports,
    getWeeklyReport,
    exportAllData,
    importAllData,
};
//# sourceMappingURL=db-backup.js.map