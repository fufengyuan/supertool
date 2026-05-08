"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
/** Accounting: Categories, Records, Stats, Trend, Budget, Templates, Receipts */
const electron_1 = require("electron");
exports.default = {
    getAccountingCategories: () => electron_1.ipcRenderer.invoke('accounting:categories:get'),
    addAccountingCategory: (category) => electron_1.ipcRenderer.invoke('accounting:categories:add', category),
    updateAccountingCategory: (id, updates) => electron_1.ipcRenderer.invoke('accounting:categories:update', id, updates),
    deleteAccountingCategory: (id) => electron_1.ipcRenderer.invoke('accounting:categories:delete', id),
    getAccountingRecords: (options) => electron_1.ipcRenderer.invoke('accounting:records:get', options),
    addAccountingRecord: (record) => electron_1.ipcRenderer.invoke('accounting:records:add', record),
    updateAccountingRecord: (id, updates) => electron_1.ipcRenderer.invoke('accounting:records:update', id, updates),
    deleteAccountingRecord: (id) => electron_1.ipcRenderer.invoke('accounting:records:delete', id),
    getAccountingStats: (options) => electron_1.ipcRenderer.invoke('accounting:stats:get', options),
    exportAccountingCSV: (options) => electron_1.ipcRenderer.invoke('accounting:export:csv', options),
    initDefaultAccountingCategories: () => electron_1.ipcRenderer.invoke('accounting:init-categories'),
    // Receipt File Upload/Preview
    uploadAccountingReceipt: (fileName, base64Data) => electron_1.ipcRenderer.invoke('accounting:upload-receipt', fileName, base64Data),
    getAccountingReceiptFile: (filePath) => electron_1.ipcRenderer.invoke('accounting:get-receipt-file', filePath),
    // Trend, Budget & Templates
    getAccountingTrend: (months) => electron_1.ipcRenderer.invoke('accounting:trend:get', months),
    getBudgets: () => electron_1.ipcRenderer.invoke('accounting:budgets:get'),
    addBudget: (data) => electron_1.ipcRenderer.invoke('accounting:budgets:add', data),
    updateBudget: (id, updates) => electron_1.ipcRenderer.invoke('accounting:budgets:update', id, updates),
    deleteBudget: (id) => electron_1.ipcRenderer.invoke('accounting:budgets:delete', id),
    checkBudgetAlerts: () => electron_1.ipcRenderer.invoke('accounting:budgets:alerts'),
    getTemplates: () => electron_1.ipcRenderer.invoke('accounting:templates:get'),
    addTemplate: (data) => electron_1.ipcRenderer.invoke('accounting:templates:add', data),
    updateTemplate: (id, updates) => electron_1.ipcRenderer.invoke('accounting:templates:update', id, updates),
    deleteTemplate: (id) => electron_1.ipcRenderer.invoke('accounting:templates:delete', id),
    useTemplate: (id) => electron_1.ipcRenderer.invoke('accounting:templates:use', id),
};
//# sourceMappingURL=preload-accounting.js.map