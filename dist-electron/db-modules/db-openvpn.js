"use strict";
const db_core_1 = require("./db-core");
function getOpenVPNConfigs() {
    return (0, db_core_1.getDatabase)().prepare('SELECT * FROM openvpn_configs ORDER BY name COLLATE NOCASE').all();
}
function addOpenVPNConfig(name, filePath, content) {
    const id = `ovpn_${Date.now().toString(36)}_${Math.random().toString(36).slice(2, 8)}`;
    const now = new Date().toISOString();
    (0, db_core_1.getDatabase)().prepare('INSERT INTO openvpn_configs (id, name, filePath, content, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?)').run(id, name, filePath, content, now, now);
    return { id };
}
function updateOpenVPNConfig(id, updates) {
    const parts = [];
    const params = [];
    if (updates.name !== undefined) {
        parts.push('name = ?');
        params.push(updates.name);
    }
    if (updates.filePath !== undefined) {
        parts.push('filePath = ?');
        params.push(updates.filePath);
    }
    if (updates.content !== undefined) {
        parts.push('content = ?');
        params.push(updates.content);
    }
    parts.push('updatedAt = ?');
    params.push(new Date().toISOString());
    params.push(id);
    (0, db_core_1.getDatabase)().prepare(`UPDATE openvpn_configs SET ${parts.join(', ')} WHERE id = ?`).run(...params);
}
function deleteOpenVPNConfig(id) {
    try {
        (0, db_core_1.getDatabase)().prepare('DELETE FROM openvpn_configs WHERE id = ?').run(id);
        return { success: true };
    }
    catch (e) {
        return { success: false, error: e.message };
    }
}
module.exports = {
    getOpenVPNConfigs,
    addOpenVPNConfig,
    updateOpenVPNConfig,
    deleteOpenVPNConfig,
};
//# sourceMappingURL=db-openvpn.js.map