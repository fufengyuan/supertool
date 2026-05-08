import { type AccountingCategoryRecord, type AccountingRecord, type AccountingBudget, type AccountingTemplate } from './db-core';
declare function initDefaultAccountingCategories(): void;
declare function getAccountingCategories(): AccountingCategoryRecord[];
declare function addAccountingCategory(data: {
    id?: string;
    name: string;
    type?: string;
    icon?: string;
    sortOrder?: number;
}): AccountingCategoryRecord;
declare function updateAccountingCategory(id: string, updates: {
    name?: string;
    type?: string;
    icon?: string;
    sortOrder?: number;
}): AccountingCategoryRecord | null;
declare function deleteAccountingCategory(id: string): {
    success: boolean;
    error?: string;
};
/** Ensure the accounting receipts directory exists */
declare function ensureAccountingReceiptsDir(): string;
/** Generate a voucher number in PZ-YYYYMMDD-XXX format */
declare function generateVoucherNumber(dateStr?: string): string;
declare function getAccountingRecords(options?: {
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
}): {
    records: AccountingRecord[];
    total: number;
};
declare function addAccountingRecord(data: {
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
}): AccountingRecord;
declare function updateAccountingRecord(id: string, updates: {
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
}): AccountingRecord | null;
declare function deleteAccountingRecord(id: string): boolean;
declare function getAccountingStats(startDate?: string, endDate?: string): {
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
declare function exportAccountingRecordsCSV(options?: {
    startDate?: string;
    endDate?: string;
    type?: string;
    category?: string;
    status?: string;
    payment_method?: string;
    entity?: string;
    project?: string;
    search?: string;
}): string;
declare function getBudgets(): AccountingBudget[];
declare function addBudget(data: {
    category: string;
    amount: number;
    period?: string;
}): AccountingBudget;
declare function updateBudget(id: string, updates: {
    category?: string;
    amount?: number;
    period?: string;
}): AccountingBudget | null;
declare function deleteBudget(id: string): {
    success: boolean;
    error?: string;
};
declare function checkBudgetAlerts(): {
    category: string;
    budget: number;
    spent: number;
    percent: number;
    over: boolean;
}[];
declare function getTemplates(): AccountingTemplate[];
declare function addTemplate(data: {
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
}): AccountingTemplate;
declare function updateTemplate(id: string, updates: {
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
}): AccountingTemplate | null;
declare function deleteTemplate(id: string): {
    success: boolean;
    error?: string;
};
declare function useTemplate(id: string): {
    success: boolean;
    error?: string;
};
declare function getAccountingTrend(months?: number): {
    month: string;
    income: number;
    expense: number;
    count: number;
}[];
declare const _default: {
    initDefaultAccountingCategories: typeof initDefaultAccountingCategories;
    getAccountingCategories: typeof getAccountingCategories;
    addAccountingCategory: typeof addAccountingCategory;
    updateAccountingCategory: typeof updateAccountingCategory;
    deleteAccountingCategory: typeof deleteAccountingCategory;
    ensureAccountingReceiptsDir: typeof ensureAccountingReceiptsDir;
    generateVoucherNumber: typeof generateVoucherNumber;
    getAccountingRecords: typeof getAccountingRecords;
    addAccountingRecord: typeof addAccountingRecord;
    updateAccountingRecord: typeof updateAccountingRecord;
    deleteAccountingRecord: typeof deleteAccountingRecord;
    getAccountingStats: typeof getAccountingStats;
    exportAccountingRecordsCSV: typeof exportAccountingRecordsCSV;
    getBudgets: typeof getBudgets;
    addBudget: typeof addBudget;
    updateBudget: typeof updateBudget;
    deleteBudget: typeof deleteBudget;
    checkBudgetAlerts: typeof checkBudgetAlerts;
    getTemplates: typeof getTemplates;
    addTemplate: typeof addTemplate;
    updateTemplate: typeof updateTemplate;
    deleteTemplate: typeof deleteTemplate;
    useTemplate: typeof useTemplate;
    getAccountingTrend: typeof getAccountingTrend;
};
export = _default;
