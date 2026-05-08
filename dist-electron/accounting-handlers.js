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
exports.registerAccountingHandlers = registerAccountingHandlers;
const electron_1 = require("electron");
const path = __importStar(require("path"));
const os = __importStar(require("os"));
const fs = __importStar(require("fs"));
function registerAccountingHandlers(db) {
    electron_1.ipcMain.handle('accounting:categories:get', () => db.getAccountingCategories());
    electron_1.ipcMain.handle('accounting:categories:add', (_event, category) => db.addAccountingCategory(category));
    electron_1.ipcMain.handle('accounting:categories:update', (_event, id, updates) => db.updateAccountingCategory(id, updates));
    electron_1.ipcMain.handle('accounting:categories:delete', (_event, id) => db.deleteAccountingCategory(id));
    electron_1.ipcMain.handle('accounting:records:get', (_event, options) => db.getAccountingRecords(options));
    electron_1.ipcMain.handle('accounting:records:add', (_event, record) => db.addAccountingRecord(record));
    electron_1.ipcMain.handle('accounting:records:update', (_event, id, updates) => db.updateAccountingRecord(id, updates));
    electron_1.ipcMain.handle('accounting:records:delete', (_event, id) => db.deleteAccountingRecord(id));
    electron_1.ipcMain.handle('accounting:stats:get', (_event, options) => db.getAccountingStats(options?.startDate, options?.endDate));
    electron_1.ipcMain.handle('accounting:export:csv', (_event, options) => db.exportAccountingRecordsCSV(options));
    electron_1.ipcMain.handle('accounting:init-categories', () => db.initDefaultAccountingCategories());
    electron_1.ipcMain.handle('accounting:trend:get', (_event, months) => db.getAccountingTrend(months));
    electron_1.ipcMain.handle('accounting:budgets:get', () => db.getBudgets());
    electron_1.ipcMain.handle('accounting:budgets:add', (_event, data) => db.addBudget(data));
    electron_1.ipcMain.handle('accounting:budgets:update', (_event, id, updates) => db.updateBudget(id, updates));
    electron_1.ipcMain.handle('accounting:budgets:delete', (_event, id) => db.deleteBudget(id));
    electron_1.ipcMain.handle('accounting:budgets:alerts', () => db.checkBudgetAlerts());
    electron_1.ipcMain.handle('accounting:templates:get', () => db.getTemplates());
    electron_1.ipcMain.handle('accounting:templates:add', (_event, data) => db.addTemplate(data));
    electron_1.ipcMain.handle('accounting:templates:update', (_event, id, updates) => db.updateTemplate(id, updates));
    electron_1.ipcMain.handle('accounting:templates:delete', (_event, id) => db.deleteTemplate(id));
    electron_1.ipcMain.handle('accounting:templates:use', (_event, id) => db.useTemplate(id));
    // Receipt File Handlers
    electron_1.ipcMain.handle('accounting:upload-receipt', async (_event, fileName, base64Data) => {
        try {
            const homeDir = os.homedir();
            const receiptDir = path.join(homeDir, '.supertool', 'accounting-receipts');
            if (!fs.existsSync(receiptDir))
                fs.mkdirSync(receiptDir, { recursive: true });
            const safeName = fileName.replace(/[^a-zA-Z0-9._-]/g, '_');
            const randomSuffix = Math.random().toString(36).slice(2, 8);
            const savedName = `${Date.now()}_${randomSuffix}_${safeName}`;
            const savedPath = path.join(receiptDir, savedName);
            const base64Content = base64Data.includes(',') ? base64Data.split(',')[1] : base64Data;
            fs.writeFileSync(savedPath, Buffer.from(base64Content, 'base64'));
            const ext = path.extname(savedName).toLowerCase();
            let type;
            if (ext === '.pdf')
                type = 'pdf';
            else if (['.png', '.jpg', '.jpeg', '.gif', '.webp'].includes(ext))
                type = 'image';
            else
                type = 'file';
            const stats = fs.statSync(savedPath);
            return { success: true, path: savedPath, type, name: savedName, size: stats.size };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('accounting:get-receipt-file', async (_event, filePath) => {
        try {
            if (!fs.existsSync(filePath))
                return { success: false, error: 'File not found' };
            const ext = path.extname(filePath).toLowerCase();
            let mimeType;
            if (ext === '.pdf')
                mimeType = 'application/pdf';
            else if (ext === '.png')
                mimeType = 'image/png';
            else if (ext === '.jpg' || ext === '.jpeg')
                mimeType = 'image/jpeg';
            else if (ext === '.gif')
                mimeType = 'image/gif';
            else if (ext === '.webp')
                mimeType = 'image/webp';
            else
                mimeType = 'application/octet-stream';
            const fileBuffer = fs.readFileSync(filePath);
            const base64 = fileBuffer.toString('base64');
            const dataUrl = `data:${mimeType};base64,${base64}`;
            return { success: true, dataUrl };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
} // end registerAccountingHandlers
//# sourceMappingURL=accounting-handlers.js.map