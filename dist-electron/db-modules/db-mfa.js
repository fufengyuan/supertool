"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getAllMfaSecrets = getAllMfaSecrets;
exports.addMfaSecret = addMfaSecret;
exports.updateMfaSecret = updateMfaSecret;
exports.deleteMfaSecret = deleteMfaSecret;
const db_core_1 = require("./db-core");
function getAllMfaSecrets() {
    const rows = (0, db_core_1.getDatabase)().prepare('SELECT * FROM mfa_secrets ORDER BY createdAt ASC').all();
    return rows;
}
function addMfaSecret(data) {
    const now = new Date().toISOString();
    const id = data.id || `mfa_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    const stmt = (0, db_core_1.getDatabase)().prepare(`INSERT INTO mfa_secrets (id, name, secret, digits, period, algorithm, account, issuer, createdAt, updatedAt)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`);
    stmt.run(id, data.name, data.secret, data.digits || 6, data.period || 30, (data.algorithm || 'sha1').toLowerCase(), data.account || '', data.issuer || '', now, now);
    const row = (0, db_core_1.getDatabase)().prepare('SELECT * FROM mfa_secrets WHERE id = ?').get(id);
    return row;
}
function updateMfaSecret(id, updates) {
    const existing = (0, db_core_1.getDatabase)().prepare('SELECT * FROM mfa_secrets WHERE id = ?').get(id);
    if (!existing)
        return null;
    const now = new Date().toISOString();
    const name = updates.name !== undefined ? updates.name : existing.name;
    const account = updates.account !== undefined ? updates.account : existing.account;
    const issuer = updates.issuer !== undefined ? updates.issuer : existing.issuer;
    (0, db_core_1.getDatabase)().prepare('UPDATE mfa_secrets SET name = ?, account = ?, issuer = ?, updatedAt = ? WHERE id = ?').run(name, account, issuer, now, id);
    const row = (0, db_core_1.getDatabase)().prepare('SELECT * FROM mfa_secrets WHERE id = ?').get(id);
    return row;
}
function deleteMfaSecret(id) {
    const result = (0, db_core_1.getDatabase)().prepare('DELETE FROM mfa_secrets WHERE id = ?').run(id);
    return result.changes > 0;
}
//# sourceMappingURL=db-mfa.js.map