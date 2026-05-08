"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
/**
 * Preload script — IPC bridge entry point
 * Imports all business modules and exposes them via contextBridge
 */
const preload_core_1 = require("./preload-modules/preload-core");
// Import all business modules
const preload_todos_1 = __importDefault(require("./preload-modules/preload-todos"));
const preload_projects_1 = __importDefault(require("./preload-modules/preload-projects"));
const preload_git_1 = __importDefault(require("./preload-modules/preload-git"));
const preload_cicd_1 = __importDefault(require("./preload-modules/preload-cicd"));
const preload_servers_1 = __importDefault(require("./preload-modules/preload-servers"));
const preload_db_1 = __importDefault(require("./preload-modules/preload-db"));
const preload_ui_1 = __importDefault(require("./preload-modules/preload-ui"));
const preload_lan_1 = __importDefault(require("./preload-modules/preload-lan"));
const preload_notes_1 = __importDefault(require("./preload-modules/preload-notes"));
const preload_accounting_1 = __importDefault(require("./preload-modules/preload-accounting"));
const preload_mfa_1 = __importDefault(require("./preload-modules/preload-mfa"));
const preload_openvpn_1 = __importDefault(require("./preload-modules/preload-openvpn"));
const preload_logs_1 = __importDefault(require("./preload-modules/preload-logs"));
const preload_api_1 = __importDefault(require("./preload-modules/preload-api"));
// Merge all modules and expose to renderer world
(0, preload_core_1.exposeAPI)([
    preload_todos_1.default,
    preload_projects_1.default,
    preload_git_1.default,
    preload_cicd_1.default,
    preload_servers_1.default,
    preload_db_1.default,
    preload_ui_1.default,
    preload_lan_1.default,
    preload_notes_1.default,
    preload_accounting_1.default,
    preload_mfa_1.default,
    preload_openvpn_1.default,
    preload_logs_1.default,
    preload_api_1.default,
]);
// Setup native drag & drop event relay
(0, preload_core_1.setupDragDrop)();
//# sourceMappingURL=preload.js.map