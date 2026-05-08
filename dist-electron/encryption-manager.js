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
exports.ensureEncryptionSecretWarning = ensureEncryptionSecretWarning;
exports.encryptPassword = encryptPassword;
exports.decryptPassword = decryptPassword;
const logger_1 = require("./logger");
const path = __importStar(require("path"));
const crypto = __importStar(require("crypto"));
const fs = __importStar(require("fs"));
const app_bootstrap_1 = require("./app-bootstrap");
// ============ 密码加密/解密 ============
// 使用持久化的加密密钥，确保重启后已保存的密码仍可解密
function getEncryptionSecret() {
    // 优先使用环境变量
    if (process.env.TODO_LIST_ENCRYPTION_SECRET) {
        return process.env.TODO_LIST_ENCRYPTION_SECRET;
    }
    // 从磁盘读取或生成持久化密钥 — stored in ~/.supertool/
    const secretPath = path.join((0, app_bootstrap_1.getSuperToolDataDir)(), '.encryption_key');
    try {
        if (fs.existsSync(secretPath)) {
            return fs.readFileSync(secretPath, 'utf8').trim();
        }
    }
    catch {
        // 读取失败，回退到生成新密钥
    }
    const newSecret = crypto.randomBytes(32).toString('hex');
    try {
        fs.writeFileSync(secretPath, newSecret, { mode: 0o600 });
        (0, logger_1.info)('[Encryption] Generated and saved new persistent key');
    }
    catch (err) {
        console.error('[Encryption] Failed to save encryption key:', err);
    }
    return newSecret;
}
const ENCRYPTION_SECRET = getEncryptionSecret();
let encryptionSecretWarningShown = false;
function ensureEncryptionSecretWarning() {
    if (!encryptionSecretWarningShown && !process.env.TODO_LIST_ENCRYPTION_SECRET) {
        encryptionSecretWarningShown = true;
    }
}
// 密码解密失败去重：同一密码只 warn 一次，避免刷屏
const failedDecryptCache = new Set();
function encryptPassword(plaintext) {
    if (!plaintext)
        return plaintext;
    const salt = crypto.randomBytes(16);
    const iv = crypto.randomBytes(12);
    const key = crypto.scryptSync(ENCRYPTION_SECRET, salt, 32);
    const cipher = crypto.createCipheriv('aes-256-gcm', key, iv);
    let encrypted = cipher.update(plaintext, 'utf8', 'base64');
    encrypted += cipher.final('base64');
    const authTag = cipher.getAuthTag();
    return `${salt.toString('base64')}:${iv.toString('base64')}:${authTag.toString('base64')}:${encrypted}`;
}
function decryptPassword(stored) {
    if (!stored)
        return stored;
    if (!stored.includes(':'))
        return stored;
    const parts = stored.split(':');
    if (parts.length !== 4)
        return stored;
    try {
        const salt = Buffer.from(parts[0], 'base64');
        const iv = Buffer.from(parts[1], 'base64');
        const authTag = Buffer.from(parts[2], 'base64');
        const encryptedData = parts[3];
        const key = crypto.scryptSync(ENCRYPTION_SECRET, salt, 32);
        const decipher = crypto.createDecipheriv('aes-256-gcm', key, iv);
        decipher.setAuthTag(authTag);
        let decrypted = decipher.update(encryptedData, 'base64', 'utf8');
        decrypted += decipher.final('utf8');
        return decrypted;
    }
    catch (err) {
        // 同一失败密码只 warn 一次
        if (!failedDecryptCache.has(stored)) {
            failedDecryptCache.add(stored);
            console.warn('[Encryption] Password decryption failed:', err.message);
        }
        // 返回原始值，让调用方决定如何处理
        return undefined;
    }
}
//# sourceMappingURL=encryption-manager.js.map