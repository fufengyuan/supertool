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
exports.parseDeployServers = parseDeployServers;
const logger_1 = require("./logger");
const fs = __importStar(require("fs"));
const db = require("./database");
const encryption_manager_1 = require("./encryption-manager");
// Parse multi-server config: lookup server details by ID from database
function parseDeployServers(serversJson, globalDeployPath, libSeparate) {
    (0, logger_1.info)(`[parseDeployServers] serversJson: ${serversJson}`);
    (0, logger_1.info)(`[parseDeployServers] globalDeployPath: ${globalDeployPath}`);
    if (!serversJson)
        return { error: '未配置服务器' };
    try {
        const parsed = JSON.parse(serversJson);
        (0, logger_1.info)('[parseDeployServers] parsed:', JSON.stringify(parsed));
        if (!Array.isArray(parsed) || parsed.length === 0)
            return { error: '未配置服务器' };
        const results = [];
        const allServers = db.getAllServers();
        (0, logger_1.info)(`[parseDeployServers] DB has ${allServers.length} servers, IDs: ${JSON.stringify(allServers.map(s => ({ id: s.id, type: typeof s.id })))}`);
        for (const s of parsed) {
            if (s.serverId) {
                // Lookup from database — use loose comparison to handle string/number mismatch
                const srv = allServers.find((x) => String(x.id) === String(s.serverId));
                (0, logger_1.info)(`[parseDeployServers] serverId: ${s.serverId} (type: ${typeof s.serverId}) found: ${!!srv} s.deployDir: ${s.deployDir}`);
                if (srv) {
                    // Server-level deployDir overrides global deployPath; fall back to global if empty
                    const deployDir = s.deployDir || globalDeployPath || '';
                    (0, logger_1.info)(`[parseDeployServers] deployDir resolved to: ${deployDir}`);
                    results.push({
                        host: srv.host,
                        port: srv.port || 22,
                        username: srv.username,
                        password: (srv.password ? (0, encryption_manager_1.decryptPassword)(srv.password) : undefined),
                        privateKey: srv.sshKeyPath ? fs.readFileSync(srv.sshKeyPath) : undefined,
                        deployDir,
                        libDir: libSeparate ? deployDir + '/lib' : null,
                    });
                }
                else {
                    // ServerId set but not found in DB — report which one
                    const dbIds = allServers.map(x => x.id);
                    return { error: `服务器不存在 (ID: ${s.serverId})，数据库中有 ${allServers.length} 台服务器: ${JSON.stringify(dbIds)}` };
                }
            }
        }
        return results.length > 0 ? results : { error: '未配置服务器' };
    }
    catch {
        return { error: '服务器配置解析失败' };
    }
}
//# sourceMappingURL=deploy-server-parser.js.map