"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
/** MFA / TOTP API */
const electron_1 = require("electron");
exports.default = {
    getMfaSecrets: () => electron_1.ipcRenderer.invoke('mfa:get-secrets'),
    addMfaSecret: (data) => electron_1.ipcRenderer.invoke('mfa:add-secret', data),
    updateMfaSecret: (id, updates) => electron_1.ipcRenderer.invoke('mfa:update-secret', id, updates),
    deleteMfaSecret: (id) => electron_1.ipcRenderer.invoke('mfa:delete-secret', id),
    generateMfaCode: (secret, digits, period, algorithm) => electron_1.ipcRenderer.invoke('mfa:generate-code', secret, digits, period, algorithm),
    parseOtpAuthUri: (uri) => electron_1.ipcRenderer.invoke('mfa:parse-uri', uri),
};
//# sourceMappingURL=preload-mfa.js.map