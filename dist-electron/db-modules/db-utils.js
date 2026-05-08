"use strict";
const db_core_1 = require("./db-core");
function getLogPresets() {
    return (0, db_core_1.getDatabase)().prepare('SELECT * FROM log_presets ORDER BY presetGroup, name COLLATE NOCASE').all();
}
function addLogPreset(name, serverIds, logPath, logType, keywords, maxLines, presetGroup = '未分组') {
    const id = `lp_${Date.now().toString(36)}_${Math.random().toString(36).slice(2, 8)}`;
    const now = new Date().toISOString();
    (0, db_core_1.getDatabase)().prepare('INSERT INTO log_presets (id, name, presetGroup, serverIds, logPath, logType, keywords, maxLines, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)').run(id, name, presetGroup, JSON.stringify(serverIds), logPath, logType, JSON.stringify(keywords), maxLines, now, now);
    return { id };
}
function updateLogPreset(id, updates) {
    const parts = [];
    const params = [];
    if (updates.name !== undefined) {
        parts.push('name = ?');
        params.push(updates.name);
    }
    if (updates.presetGroup !== undefined) {
        parts.push('presetGroup = ?');
        params.push(updates.presetGroup);
    }
    if (updates.serverIds !== undefined) {
        parts.push('serverIds = ?');
        params.push(JSON.stringify(updates.serverIds));
    }
    if (updates.logPath !== undefined) {
        parts.push('logPath = ?');
        params.push(updates.logPath);
    }
    if (updates.logType !== undefined) {
        parts.push('logType = ?');
        params.push(updates.logType);
    }
    if (updates.keywords !== undefined) {
        parts.push('keywords = ?');
        params.push(JSON.stringify(updates.keywords));
    }
    if (updates.maxLines !== undefined) {
        parts.push('maxLines = ?');
        params.push(updates.maxLines);
    }
    parts.push('updatedAt = ?');
    params.push(new Date().toISOString());
    params.push(id);
    const result = (0, db_core_1.getDatabase)().prepare(`UPDATE log_presets SET ${parts.join(', ')} WHERE id = ?`).run(...params);
    if (result.changes === 0) {
        throw new Error(`预设不存在: ${id}`);
    }
}
function deleteLogPreset(id) {
    try {
        (0, db_core_1.getDatabase)().prepare('DELETE FROM log_presets WHERE id = ?').run(id);
        return { success: true };
    }
    catch (e) {
        return { success: false, error: e.message };
    }
}
function getCalculatorHistory(limit = 50) {
    return (0, db_core_1.getDatabase)().prepare('SELECT * FROM calculator_history ORDER BY createdAt DESC LIMIT ?').all(limit);
}
function addCalculatorHistory(expression, result) {
    const id = crypto.randomUUID();
    const createdAt = new Date().toISOString();
    (0, db_core_1.getDatabase)().prepare('INSERT INTO calculator_history (id, expression, result, createdAt) VALUES (?, ?, ?, ?)').run(id, expression, result, createdAt);
    return { id, expression, result, createdAt };
}
function clearCalculatorHistory() {
    (0, db_core_1.getDatabase)().prepare('DELETE FROM calculator_history').run();
}
function rowToApiRequest(row) {
    return {
        id: row.id,
        name: row.name,
        method: row.method,
        url: row.url,
        headers: row.headers,
        body: row.body,
        contentType: row.contentType,
        createdAt: row.createdAt,
        updatedAt: row.updatedAt,
    };
}
function addApiRequest(request) {
    const now = request.createdAt || new Date().toISOString();
    const id = request.id || `apir_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    INSERT INTO api_requests (id, name, method, url, headers, body, contentType, createdAt, updatedAt)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);
    stmt.run(id, request.name || '', request.method || 'GET', request.url, request.headers || '[]', request.body || null, request.contentType || 'none', now, request.updatedAt || now);
    return { ...request, id, createdAt: now, updatedAt: request.updatedAt || now };
}
function getApiRequests() {
    const stmt = (0, db_core_1.getDatabase)().prepare('SELECT * FROM api_requests ORDER BY updatedAt DESC');
    const rows = stmt.all();
    return rows.map(rowToApiRequest);
}
function getApiRequestById(id) {
    const stmt = (0, db_core_1.getDatabase)().prepare('SELECT * FROM api_requests WHERE id = ?');
    const row = stmt.get(id);
    if (!row)
        return null;
    return rowToApiRequest(row);
}
function updateApiRequest(id, updates) {
    const parts = [];
    const params = [];
    if (updates.name !== undefined) {
        parts.push('name = ?');
        params.push(updates.name);
    }
    if (updates.method !== undefined) {
        parts.push('method = ?');
        params.push(updates.method);
    }
    if (updates.url !== undefined) {
        parts.push('url = ?');
        params.push(updates.url);
    }
    if (updates.headers !== undefined) {
        parts.push('headers = ?');
        params.push(updates.headers);
    }
    if (updates.body !== undefined) {
        parts.push('body = ?');
        params.push(updates.body);
    }
    if (updates.contentType !== undefined) {
        parts.push('contentType = ?');
        params.push(updates.contentType);
    }
    parts.push('updatedAt = ?');
    params.push(new Date().toISOString());
    params.push(id);
    if (parts.length <= 1)
        return getApiRequestById(id);
    const result = (0, db_core_1.getDatabase)().prepare(`UPDATE api_requests SET ${parts.join(', ')} WHERE id = ?`).run(...params);
    if (result.changes === 0)
        return null;
    return getApiRequestById(id);
}
function deleteApiRequest(id) {
    (0, db_core_1.getDatabase)().prepare('DELETE FROM api_requests WHERE id = ?').run(id);
    return { success: true };
}
module.exports = {
    getLogPresets,
    addLogPreset,
    updateLogPreset,
    deleteLogPreset,
    getCalculatorHistory,
    addCalculatorHistory,
    clearCalculatorHistory,
    rowToApiRequest,
    addApiRequest,
    getApiRequests,
    getApiRequestById,
    updateApiRequest,
    deleteApiRequest,
};
//# sourceMappingURL=db-utils.js.map