"use strict";
const db_core_1 = require("./db-core");
function rowToServer(row) {
    return {
        id: row.id,
        name: row.name,
        host: row.host,
        port: row.port,
        username: row.username,
        sshKeyPath: row.sshKeyPath,
        password: row.password,
        description: row.description,
        tags: row.tags ? row.tags.split(',') : [],
        groupId: row.groupId,
        requiresApproval: !!row.requiresApproval,
        createdAt: row.createdAt,
        updatedAt: row.updatedAt
    };
}
function getAllServers() {
    const stmt = (0, db_core_1.getDatabase)().prepare('SELECT * FROM servers ORDER BY createdAt DESC');
    const rows = stmt.all();
    return rows.map(rowToServer);
}
function getServerById(serverId) {
    const stmt = (0, db_core_1.getDatabase)().prepare('SELECT * FROM servers WHERE id = ?');
    const row = stmt.get(serverId);
    if (!row)
        return null;
    return rowToServer(row);
}
function addServer(server) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    INSERT INTO servers (id, name, host, port, username, sshKeyPath, password, description, tags, groupId, requiresApproval, createdAt, updatedAt)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);
    stmt.run(server.id, server.name, server.host, server.port || 22, server.username, server.sshKeyPath || null, server.password || null, server.description || '', server.tags ? server.tags.join(',') : '', server.groupId || null, server.requiresApproval ? 1 : 0, server.createdAt, server.updatedAt);
    return getServerById(server.id);
}
function updateServer(server) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    UPDATE servers SET
      name = ?,
      host = ?,
      port = ?,
      username = ?,
      sshKeyPath = ?,
      password = ?,
      description = ?,
      tags = ?,
      groupId = ?,
      requiresApproval = ?,
      updatedAt = ?
    WHERE id = ?
  `);
    stmt.run(server.name, server.host, server.port, server.username, server.sshKeyPath, server.password, server.description, server.tags ? server.tags.join(',') : '', server.groupId, server.requiresApproval ? 1 : 0, server.updatedAt, server.id);
    return getServerById(server.id);
}
function deleteServer(serverId) {
    const stmt = (0, db_core_1.getDatabase)().prepare('DELETE FROM servers WHERE id = ?');
    stmt.run(serverId);
    return { success: true };
}
// 获取服务器分组
function rowToServerGroup(row) {
    return {
        id: row.id,
        name: row.name,
        description: row.description || '',
        parentId: row.parentId || null,
        color: row.color || '#6c63ff',
        createdAt: row.createdAt,
        updatedAt: row.updatedAt
    };
}
function getAllServerGroups() {
    const stmt = (0, db_core_1.getDatabase)().prepare('SELECT * FROM server_groups ORDER BY createdAt ASC');
    const rows = stmt.all();
    return rows.map(rowToServerGroup);
}
function addServerGroup(group) {
    const now = new Date().toISOString();
    const id = group.id || crypto.randomUUID();
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    INSERT INTO server_groups (id, name, description, parentId, color, createdAt, updatedAt)
    VALUES (?, ?, ?, ?, ?, ?, ?)
  `);
    stmt.run(id, group.name, group.description || '', group.parentId || null, group.color || '#6c63ff', now, now);
    return getAllServerGroups().find(g => g.id === id);
}
function updateServerGroup(groupId, updates) {
    const fields = [];
    const values = [];
    const now = new Date().toISOString();
    if (updates.name !== undefined) {
        fields.push('name = ?');
        values.push(updates.name);
    }
    if (updates.description !== undefined) {
        fields.push('description = ?');
        values.push(updates.description);
    }
    if (updates.parentId !== undefined) {
        fields.push('parentId = ?');
        values.push(updates.parentId || null);
    }
    if (updates.color !== undefined) {
        fields.push('color = ?');
        values.push(updates.color);
    }
    fields.push('updatedAt = ?');
    values.push(now);
    values.push(groupId);
    const stmt = (0, db_core_1.getDatabase)().prepare(`UPDATE server_groups SET ${fields.join(', ')} WHERE id = ?`);
    stmt.run(...values);
    return getAllServerGroups().find(g => g.id === groupId);
}
function deleteServerGroup(groupId) {
    // 递归删除子分组
    const children = getAllServerGroups().filter(g => g.parentId === groupId);
    children.forEach(child => deleteServerGroup(child.id));
    // 将分组下的服务器移至未分组
    const servers = getAllServers();
    servers.filter(s => s.groupId === groupId).forEach(s => {
        const stmt = (0, db_core_1.getDatabase)().prepare('UPDATE servers SET groupId = NULL WHERE id = ?');
        stmt.run(s.id);
    });
    // 删除分组本身
    const stmt = (0, db_core_1.getDatabase)().prepare('DELETE FROM server_groups WHERE id = ?');
    stmt.run(groupId);
    return { success: true };
}
module.exports = {
    rowToServer,
    getAllServers,
    getServerById,
    addServer,
    updateServer,
    deleteServer,
    rowToServerGroup,
    getAllServerGroups,
    addServerGroup,
    updateServerGroup,
    deleteServerGroup,
};
//# sourceMappingURL=db-servers.js.map