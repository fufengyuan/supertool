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
const db_core_1 = require("./db-core");
const path = __importStar(require("path"));
const os = __importStar(require("os"));
const fs = __importStar(require("fs"));
function initDefaultAccountingCategories() {
    const db = (0, db_core_1.getDatabase)();
    const count = db.prepare('SELECT COUNT(*) as c FROM accounting_categories').get().c;
    if (count > 0)
        return; // already seeded
    const now = new Date().toISOString();
    const defaults = [
        // Enterprise income categories
        { id: 'ac_client_payment', name: '客户回款', type: 'income', icon: '💰', sortOrder: 1 },
        { id: 'ac_project_income', name: '项目收入', type: 'income', icon: '📋', sortOrder: 2 },
        { id: 'ac_invest_income', name: '投资收益', type: 'income', icon: '📈', sortOrder: 3 },
        { id: 'ac_other_income', name: '其他收入', type: 'income', icon: '📝', sortOrder: 4 },
        // Enterprise expense categories
        { id: 'ac_server_cost', name: '服务器费用', type: 'expense', icon: '🖥️', sortOrder: 1 },
        { id: 'ac_office_purchase', name: '办公采购', type: 'expense', icon: '🛒', sortOrder: 2 },
        { id: 'ac_travel', name: '差旅费', type: 'expense', icon: '✈️', sortOrder: 3 },
        { id: 'ac_entertainment', name: '业务招待', type: 'expense', icon: '🤝', sortOrder: 4 },
        { id: 'ac_salary', name: '工资薪酬', type: 'expense', icon: '💼', sortOrder: 5 },
        { id: 'ac_tech_service', name: '技术服务费', type: 'expense', icon: '⚙️', sortOrder: 6 },
        { id: 'ac_rent_utility', name: '房租水电', type: 'expense', icon: '🏢', sortOrder: 7 },
        { id: 'ac_marketing', name: '营销推广', type: 'expense', icon: '📢', sortOrder: 8 },
        { id: 'ac_tax', name: '税费', type: 'expense', icon: '🧾', sortOrder: 9 },
        { id: 'ac_other_expense', name: '其他支出', type: 'expense', icon: '📝', sortOrder: 10 },
    ];
    const insert = db.prepare('INSERT INTO accounting_categories (id, name, type, icon, sortOrder, createdAt) VALUES (?, ?, ?, ?, ?, ?)');
    const insertAll = db.transaction((cats) => {
        for (const cat of cats) {
            insert.run(cat.id, cat.name, cat.type, cat.icon, cat.sortOrder, now);
        }
    });
    insertAll(defaults);
}
function getAccountingCategories() {
    return (0, db_core_1.getDatabase)().prepare('SELECT * FROM accounting_categories ORDER BY type ASC, sortOrder ASC, createdAt ASC').all();
}
function addAccountingCategory(data) {
    const id = data.id || 'acat_' + Date.now().toString(36) + Math.random().toString(36).slice(2, 6);
    const now = new Date().toISOString();
    (0, db_core_1.getDatabase)().prepare('INSERT INTO accounting_categories (id, name, type, icon, sortOrder, createdAt) VALUES (?, ?, ?, ?, ?, ?)').run(id, data.name, data.type || 'expense', data.icon || '', data.sortOrder || 0, now);
    return (0, db_core_1.getDatabase)().prepare('SELECT * FROM accounting_categories WHERE id = ?').get(id);
}
function updateAccountingCategory(id, updates) {
    const fields = [];
    const values = [];
    if (updates.name !== undefined) {
        fields.push('name = ?');
        values.push(updates.name);
    }
    if (updates.type !== undefined) {
        fields.push('type = ?');
        values.push(updates.type);
    }
    if (updates.icon !== undefined) {
        fields.push('icon = ?');
        values.push(updates.icon);
    }
    if (updates.sortOrder !== undefined) {
        fields.push('sortOrder = ?');
        values.push(updates.sortOrder);
    }
    if (fields.length === 0) {
        return (0, db_core_1.getDatabase)().prepare('SELECT * FROM accounting_categories WHERE id = ?').get(id) || null;
    }
    values.push(id);
    (0, db_core_1.getDatabase)().prepare(`UPDATE accounting_categories SET ${fields.join(', ')} WHERE id = ?`).run(...values);
    return (0, db_core_1.getDatabase)().prepare('SELECT * FROM accounting_categories WHERE id = ?').get(id) || null;
}
function deleteAccountingCategory(id) {
    const db = (0, db_core_1.getDatabase)();
    // Check if any records reference this category
    const cat = db.prepare('SELECT name FROM accounting_categories WHERE id = ?').get(id);
    if (!cat)
        return { success: false, error: '分类不存在' };
    const count = db.prepare('SELECT COUNT(*) as c FROM accounting_records WHERE category = ?').get(cat.name).c;
    if (count > 0)
        return { success: false, error: `该分类下有 ${count} 条记录，无法删除` };
    const result = db.prepare('DELETE FROM accounting_categories WHERE id = ?').run(id);
    return { success: result.changes > 0 };
}
/** Ensure the accounting receipts directory exists */
function ensureAccountingReceiptsDir() {
    const dir = path.join(os.homedir(), '.supertool', 'accounting-receipts');
    if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
    }
    return dir;
}
/** Generate a voucher number in PZ-YYYYMMDD-XXX format */
function generateVoucherNumber(dateStr) {
    const d = dateStr ? new Date(dateStr) : new Date();
    const yyyy = d.getFullYear();
    const mm = String(d.getMonth() + 1).padStart(2, '0');
    const dd = String(d.getDate()).padStart(2, '0');
    const datePrefix = `${yyyy}${mm}${dd}`;
    // Count existing vouchers for this date to get next sequence number
    const row = (0, db_core_1.getDatabase)().prepare("SELECT COUNT(*) as c FROM accounting_records WHERE voucher_number LIKE ?").get(`PZ-${datePrefix}-%`);
    const seq = String(row.c + 1).padStart(3, '0');
    return `PZ-${datePrefix}-${seq}`;
}
function getAccountingRecords(options) {
    const conditions = [];
    const params = [];
    if (options?.startDate) {
        conditions.push('date >= ?');
        params.push(options.startDate);
    }
    if (options?.endDate) {
        conditions.push('date <= ?');
        params.push(options.endDate);
    }
    if (options?.type) {
        conditions.push('type = ?');
        params.push(options.type);
    }
    if (options?.category) {
        conditions.push('category = ?');
        params.push(options.category);
    }
    if (options?.status) {
        conditions.push('status = ?');
        params.push(options.status);
    }
    if (options?.payment_method) {
        conditions.push('payment_method = ?');
        params.push(options.payment_method);
    }
    if (options?.entity) {
        conditions.push('entity = ?');
        params.push(options.entity);
    }
    if (options?.project) {
        conditions.push('project = ?');
        params.push(options.project);
    }
    if (options?.search) {
        conditions.push('(description LIKE ? OR category LIKE ? OR voucher_number LIKE ? OR entity LIKE ? OR project LIKE ? OR supplier LIKE ? OR invoice_number LIKE ?)');
        params.push(`%${options.search}%`, `%${options.search}%`, `%${options.search}%`, `%${options.search}%`, `%${options.search}%`, `%${options.search}%`, `%${options.search}%`);
    }
    const where = conditions.length > 0 ? ' WHERE ' + conditions.join(' AND ') : '';
    // Get total count
    const countResult = (0, db_core_1.getDatabase)().prepare(`SELECT COUNT(*) as c FROM accounting_records${where}`).get(params);
    const total = countResult.c;
    // Get paginated records
    let sql = `SELECT * FROM accounting_records${where} ORDER BY date DESC, createdAt DESC`;
    if (options?.pageSize && options.pageSize > 0) {
        sql += ' LIMIT ? OFFSET ?';
        const page = options.page || 1;
        params.push(options.pageSize, (page - 1) * options.pageSize);
    }
    const records = (0, db_core_1.getDatabase)().prepare(sql).all(...params);
    // Deserialize attachments_json from string to array
    for (const r of records) {
        try {
            const raw = r.attachments_json;
            r.attachments_json = raw && typeof raw === 'string' && raw !== '[]' ? JSON.parse(raw) : [];
        }
        catch {
            ;
            r.attachments_json = [];
        }
    }
    return { records, total };
}
function addAccountingRecord(data) {
    const id = data.id || 'arec_' + Date.now().toString(36) + Math.random().toString(36).slice(2, 6);
    const now = new Date().toISOString();
    // 金额校验：必须大于 0
    if (data.amount <= 0)
        throw new Error('金额必须大于 0');
    // Auto-generate voucher number if not provided
    const voucherNumber = data.voucher_number || generateVoucherNumber(data.date);
    (0, db_core_1.getDatabase)().prepare(`INSERT INTO accounting_records (id, date, type, category, amount, description, status, attachmentPath, createdBy, createdAt, updatedAt,
     voucher_number, receipt_type, receipt_path, entity, project, supplier, invoice_number, tax_amount, payment_method, approver, attachments_json)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`).run(id, data.date, data.type || 'expense', data.category || '', data.amount, data.description || '', data.status || 'completed', data.attachmentPath || null, data.createdBy || '', now, now, voucherNumber, data.receipt_type || '', data.receipt_path || '', data.entity || '', data.project || '', data.supplier || '', data.invoice_number || '', data.tax_amount || 0, data.payment_method || '', data.approver || '', data.attachments_json || '[]');
    return (0, db_core_1.getDatabase)().prepare('SELECT * FROM accounting_records WHERE id = ?').get(id);
}
function updateAccountingRecord(id, updates) {
    // 金额校验
    if (updates.amount !== undefined && updates.amount <= 0)
        throw new Error('金额必须大于 0');
    const fields = [];
    const values = [];
    if (updates.date !== undefined) {
        fields.push('date = ?');
        values.push(updates.date);
    }
    if (updates.type !== undefined) {
        fields.push('type = ?');
        values.push(updates.type);
    }
    if (updates.category !== undefined) {
        fields.push('category = ?');
        values.push(updates.category);
    }
    if (updates.amount !== undefined) {
        fields.push('amount = ?');
        values.push(updates.amount);
    }
    if (updates.description !== undefined) {
        fields.push('description = ?');
        values.push(updates.description);
    }
    if (updates.status !== undefined) {
        fields.push('status = ?');
        values.push(updates.status);
    }
    if (updates.attachmentPath !== undefined) {
        fields.push('attachmentPath = ?');
        values.push(updates.attachmentPath);
    }
    // Enterprise accounting fields
    if (updates.voucher_number !== undefined) {
        fields.push('voucher_number = ?');
        values.push(updates.voucher_number);
    }
    if (updates.receipt_type !== undefined) {
        fields.push('receipt_type = ?');
        values.push(updates.receipt_type);
    }
    if (updates.receipt_path !== undefined) {
        fields.push('receipt_path = ?');
        values.push(updates.receipt_path);
    }
    if (updates.entity !== undefined) {
        fields.push('entity = ?');
        values.push(updates.entity);
    }
    if (updates.project !== undefined) {
        fields.push('project = ?');
        values.push(updates.project);
    }
    if (updates.supplier !== undefined) {
        fields.push('supplier = ?');
        values.push(updates.supplier);
    }
    if (updates.invoice_number !== undefined) {
        fields.push('invoice_number = ?');
        values.push(updates.invoice_number);
    }
    if (updates.tax_amount !== undefined) {
        fields.push('tax_amount = ?');
        values.push(updates.tax_amount);
    }
    if (updates.payment_method !== undefined) {
        fields.push('payment_method = ?');
        values.push(updates.payment_method);
    }
    if (updates.approver !== undefined) {
        fields.push('approver = ?');
        values.push(updates.approver);
    }
    if (updates.attachments_json !== undefined) {
        fields.push('attachments_json = ?');
        values.push(updates.attachments_json);
    }
    if (fields.length === 0) {
        return (0, db_core_1.getDatabase)().prepare('SELECT * FROM accounting_records WHERE id = ?').get(id) || null;
    }
    fields.push('updatedAt = ?');
    values.push(new Date().toISOString());
    values.push(id);
    (0, db_core_1.getDatabase)().prepare(`UPDATE accounting_records SET ${fields.join(', ')} WHERE id = ?`).run(...values);
    return (0, db_core_1.getDatabase)().prepare('SELECT * FROM accounting_records WHERE id = ?').get(id) || null;
}
function deleteAccountingRecord(id) {
    // Read existing attachments before deletion to clean up disk files
    try {
        const record = (0, db_core_1.getDatabase)().prepare('SELECT attachments_json FROM accounting_records WHERE id = ?').get(id);
        if (record?.attachments_json) {
            try {
                const attachments = JSON.parse(record.attachments_json);
                for (const att of attachments) {
                    if (att?.path && fs.existsSync(att.path)) {
                        fs.unlinkSync(att.path);
                    }
                }
            }
            catch { } // Ignore JSON parse errors during cleanup
        }
    }
    catch { } // Ignore read errors during cleanup
    const result = (0, db_core_1.getDatabase)().prepare('DELETE FROM accounting_records WHERE id = ?').run(id);
    return result.changes > 0;
}
function getAccountingStats(startDate, endDate) {
    const dateConditions = [];
    const dateParams = [];
    if (startDate) {
        dateConditions.push('date >= ?');
        dateParams.push(startDate);
    }
    if (endDate) {
        dateConditions.push('date <= ?');
        dateParams.push(endDate);
    }
    const dateWhere = dateConditions.length > 0 ? ' WHERE ' + dateConditions.join(' AND ') : '';
    // Income total
    const incomeWhere = dateConditions.length > 0
        ? ' WHERE ' + dateConditions.join(' AND ') + " AND type = 'income'"
        : " WHERE type = 'income'";
    const incomeResult = (0, db_core_1.getDatabase)().prepare(`SELECT COALESCE(SUM(amount), 0) as total FROM accounting_records${incomeWhere}`).get(dateParams);
    // Expense total
    const expenseWhere = dateConditions.length > 0
        ? ' WHERE ' + dateConditions.join(' AND ') + " AND type = 'expense'"
        : " WHERE type = 'expense'";
    const expenseResult = (0, db_core_1.getDatabase)().prepare(`SELECT COALESCE(SUM(amount), 0) as total FROM accounting_records${expenseWhere}`).get(dateParams);
    const totalIncome = incomeResult.total;
    const totalExpense = expenseResult.total;
    // Pending amount (status = 'pending')
    const pendingWhere = dateConditions.length > 0
        ? ' WHERE ' + dateConditions.join(' AND ') + " AND status = 'pending'"
        : " WHERE status = 'pending'";
    const pendingResult = (0, db_core_1.getDatabase)().prepare(`SELECT COALESCE(SUM(amount), 0) as total FROM accounting_records${pendingWhere}`).get(dateParams);
    // Reimbursed amount (status = 'reimbursed')
    const reimbursedWhere = dateConditions.length > 0
        ? ' WHERE ' + dateConditions.join(' AND ') + " AND status = 'reimbursed'"
        : " WHERE status = 'reimbursed'";
    const reimbursedResult = (0, db_core_1.getDatabase)().prepare(`SELECT COALESCE(SUM(amount), 0) as total FROM accounting_records${reimbursedWhere}`).get(dateParams);
    // Category breakdown (expense only, for spending visualization)
    const catWhere = dateConditions.length > 0
        ? ' WHERE ' + dateConditions.join(' AND ') + " AND type = 'expense'"
        : " WHERE type = 'expense'";
    const byCategory = (0, db_core_1.getDatabase)().prepare(`SELECT category, SUM(amount) as amount FROM accounting_records${catWhere} AND category != '' GROUP BY category ORDER BY amount DESC`).all(dateParams);
    return {
        totalIncome,
        totalExpense,
        balance: totalIncome - totalExpense,
        pendingAmount: pendingResult.total,
        reimbursedAmount: reimbursedResult.total,
        byCategory,
    };
}
function exportAccountingRecordsCSV(options) {
    const conditions = [];
    const params = [];
    if (options?.startDate) {
        conditions.push('date >= ?');
        params.push(options.startDate);
    }
    if (options?.endDate) {
        conditions.push('date <= ?');
        params.push(options.endDate);
    }
    if (options?.type) {
        conditions.push('type = ?');
        params.push(options.type);
    }
    if (options?.category) {
        conditions.push('category = ?');
        params.push(options.category);
    }
    if (options?.status) {
        conditions.push('status = ?');
        params.push(options.status);
    }
    if (options?.payment_method) {
        conditions.push('payment_method = ?');
        params.push(options.payment_method);
    }
    if (options?.entity) {
        conditions.push('entity = ?');
        params.push(options.entity);
    }
    if (options?.project) {
        conditions.push('project = ?');
        params.push(options.project);
    }
    if (options?.search) {
        conditions.push('(description LIKE ? OR category LIKE ? OR voucher_number LIKE ? OR entity LIKE ? OR project LIKE ? OR supplier LIKE ? OR invoice_number LIKE ?)');
        params.push(`%${options.search}%`, `%${options.search}%`, `%${options.search}%`, `%${options.search}%`, `%${options.search}%`, `%${options.search}%`, `%${options.search}%`);
    }
    const where = conditions.length > 0 ? ' WHERE ' + conditions.join(' AND ') : '';
    const records = (0, db_core_1.getDatabase)().prepare(`SELECT * FROM accounting_records${where} ORDER BY date DESC`).all(params);
    const headers = ['日期', '类型', '分类', '金额', '描述', '状态', '凭证号', '凭证类型', '凭证路径', '企业主体', '所属项目', '供应商', '发票号', '税额', '支付方式', '审批人', '创建时间'];
    const lines = [headers.join(',')];
    for (const r of records) {
        const escape = (v) => {
            const s = String(v ?? '');
            return s.includes(',') || s.includes('"') || s.includes('\n') ? `"${s.replace(/"/g, '""')}"` : s;
        };
        lines.push([
            r.date, r.type, r.category, r.amount, r.description, r.status,
            r.voucher_number || '',
            r.receipt_type || '',
            r.receipt_path || '',
            r.entity || '',
            r.project || '',
            r.supplier || '',
            r.invoice_number || '',
            r.tax_amount || 0,
            r.payment_method || '',
            r.approver || '',
            r.createdAt
        ].map(escape).join(','));
    }
    return lines.join('\n');
}
function getBudgets() {
    return (0, db_core_1.getDatabase)().prepare('SELECT * FROM accounting_budgets ORDER BY category ASC').all();
}
function addBudget(data) {
    const id = 'abud_' + Date.now().toString(36) + Math.random().toString(36).slice(2, 6);
    const now = new Date().toISOString();
    (0, db_core_1.getDatabase)().prepare('INSERT INTO accounting_budgets (id, category, amount, period, createdAt) VALUES (?, ?, ?, ?, ?)').run(id, data.category, data.amount, data.period || 'monthly', now);
    return (0, db_core_1.getDatabase)().prepare('SELECT * FROM accounting_budgets WHERE id = ?').get(id);
}
function updateBudget(id, updates) {
    const fields = [];
    const values = [];
    if (updates.category !== undefined) {
        fields.push('category = ?');
        values.push(updates.category);
    }
    if (updates.amount !== undefined) {
        fields.push('amount = ?');
        values.push(updates.amount);
    }
    if (updates.period !== undefined) {
        fields.push('period = ?');
        values.push(updates.period);
    }
    if (fields.length === 0)
        return getBudgets().find(b => b.id === id) || null;
    values.push(id);
    (0, db_core_1.getDatabase)().prepare(`UPDATE accounting_budgets SET ${fields.join(', ')} WHERE id = ?`).run(...values);
    return (0, db_core_1.getDatabase)().prepare('SELECT * FROM accounting_budgets WHERE id = ?').get(id) || null;
}
function deleteBudget(id) {
    // Check if budget exists
    const budget = (0, db_core_1.getDatabase)().prepare('SELECT category FROM accounting_budgets WHERE id = ?').get(id);
    if (!budget)
        return { success: false, error: '预算不存在' };
    // Check if category is referenced in records
    const count = (0, db_core_1.getDatabase)().prepare("SELECT COUNT(*) as c FROM accounting_records WHERE category = ? AND type = 'expense'").get(budget.category);
    if (count.c > 0) {
        // Budget has records, warn but allow deletion
    }
    const result = (0, db_core_1.getDatabase)().prepare('DELETE FROM accounting_budgets WHERE id = ?').run(id);
    return { success: result.changes > 0 };
}
function checkBudgetAlerts() {
    const now = new Date();
    const monthStart = new Date(now.getFullYear(), now.getMonth(), 1).toISOString().slice(0, 10);
    const monthEnd = new Date(now.getFullYear(), now.getMonth() + 1, 0).toISOString().slice(0, 10);
    const budgets = getBudgets();
    const alerts = [];
    for (const budget of budgets) {
        const spentResult = (0, db_core_1.getDatabase)().prepare(`SELECT COALESCE(SUM(amount), 0) as total FROM accounting_records WHERE type = 'expense' AND category = ? AND date >= ? AND date <= ?`).get(budget.category, monthStart, monthEnd);
        const spent = spentResult.total;
        const percent = budget.amount > 0 ? Math.round((spent / budget.amount) * 100) : 0;
        if (percent > 0) {
            alerts.push({ category: budget.category, budget: budget.amount, spent, percent, over: spent > budget.amount });
        }
    }
    return alerts.sort((a, b) => b.percent - a.percent);
}
function getTemplates() {
    return (0, db_core_1.getDatabase)().prepare('SELECT * FROM accounting_templates ORDER BY useCount DESC, createdAt DESC').all();
}
function addTemplate(data) {
    const id = 'atpl_' + Date.now().toString(36) + Math.random().toString(36).slice(2, 6);
    const now = new Date().toISOString();
    (0, db_core_1.getDatabase)().prepare('INSERT INTO accounting_templates (id, name, type, category, amount, description, entity, project, supplier, payment_method, tax_rate, useCount, createdAt) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?)').run(id, data.name, data.type, data.category, data.amount, data.description || '', data.entity || '', data.project || '', data.supplier || '', data.payment_method || '', data.tax_rate || 0, now);
    return (0, db_core_1.getDatabase)().prepare('SELECT * FROM accounting_templates WHERE id = ?').get(id);
}
function updateTemplate(id, updates) {
    const fields = [];
    const values = [];
    if (updates.name !== undefined) {
        fields.push('name = ?');
        values.push(updates.name);
    }
    if (updates.type !== undefined) {
        fields.push('type = ?');
        values.push(updates.type);
    }
    if (updates.category !== undefined) {
        fields.push('category = ?');
        values.push(updates.category);
    }
    if (updates.amount !== undefined) {
        fields.push('amount = ?');
        values.push(updates.amount);
    }
    if (updates.description !== undefined) {
        fields.push('description = ?');
        values.push(updates.description);
    }
    if (updates.entity !== undefined) {
        fields.push('entity = ?');
        values.push(updates.entity);
    }
    if (updates.project !== undefined) {
        fields.push('project = ?');
        values.push(updates.project);
    }
    if (updates.supplier !== undefined) {
        fields.push('supplier = ?');
        values.push(updates.supplier);
    }
    if (updates.payment_method !== undefined) {
        fields.push('payment_method = ?');
        values.push(updates.payment_method);
    }
    if (updates.tax_rate !== undefined) {
        fields.push('tax_rate = ?');
        values.push(updates.tax_rate);
    }
    if (fields.length === 0)
        return null;
    values.push(id);
    (0, db_core_1.getDatabase)().prepare(`UPDATE accounting_templates SET ${fields.join(', ')} WHERE id = ?`).run(...values);
    return (0, db_core_1.getDatabase)().prepare('SELECT * FROM accounting_templates WHERE id = ?').get(id) || null;
}
function deleteTemplate(id) {
    // Check if template exists
    const tpl = (0, db_core_1.getDatabase)().prepare('SELECT name FROM accounting_templates WHERE id = ?').get(id);
    if (!tpl)
        return { success: false, error: '模板不存在' };
    const result = (0, db_core_1.getDatabase)().prepare('DELETE FROM accounting_templates WHERE id = ?').run(id);
    return { success: result.changes > 0 };
}
function useTemplate(id) {
    const result = (0, db_core_1.getDatabase)().prepare('UPDATE accounting_templates SET useCount = useCount + 1 WHERE id = ?').run(id);
    return { success: result.changes > 0 };
}
function getAccountingTrend(months) {
    const n = months || 12;
    const now = new Date();
    const results = [];
    for (let i = n - 1; i >= 0; i--) {
        const d = new Date(now.getFullYear(), now.getMonth() - i, 1);
        const monthStart = new Date(d.getFullYear(), d.getMonth(), 1).toISOString().slice(0, 10);
        const monthEnd = new Date(d.getFullYear(), d.getMonth() + 1, 0).toISOString().slice(0, 10);
        const monthLabel = `${d.getFullYear()}/${String(d.getMonth() + 1).padStart(2, '0')}`;
        const incomeResult = (0, db_core_1.getDatabase)().prepare(`SELECT COALESCE(SUM(amount), 0) as total FROM accounting_records WHERE type = 'income' AND date >= ? AND date <= ?`).get(monthStart, monthEnd);
        const expenseResult = (0, db_core_1.getDatabase)().prepare(`SELECT COALESCE(SUM(amount), 0) as total FROM accounting_records WHERE type = 'expense' AND date >= ? AND date <= ?`).get(monthStart, monthEnd);
        const countResult = (0, db_core_1.getDatabase)().prepare(`SELECT COUNT(*) as c FROM accounting_records WHERE date >= ? AND date <= ?`).get(monthStart, monthEnd);
        results.push({
            month: monthLabel,
            income: incomeResult.total,
            expense: expenseResult.total,
            count: countResult.c
        });
    }
    return results;
}
module.exports = {
    initDefaultAccountingCategories,
    getAccountingCategories,
    addAccountingCategory,
    updateAccountingCategory,
    deleteAccountingCategory,
    ensureAccountingReceiptsDir,
    generateVoucherNumber,
    getAccountingRecords,
    addAccountingRecord,
    updateAccountingRecord,
    deleteAccountingRecord,
    getAccountingStats,
    exportAccountingRecordsCSV,
    getBudgets,
    addBudget,
    updateBudget,
    deleteBudget,
    checkBudgetAlerts,
    getTemplates,
    addTemplate,
    updateTemplate,
    deleteTemplate,
    useTemplate,
    getAccountingTrend,
};
//# sourceMappingURL=db-accounting.js.map