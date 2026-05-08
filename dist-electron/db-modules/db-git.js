"use strict";
const db_core_1 = require("./db-core");
function rowToGitRepo(row) {
    return {
        id: row.id,
        name: row.name,
        path: row.path,
        remote: row.remote || undefined,
        branch: row.branch || undefined,
        lastOpened: row.lastOpened || undefined,
        createdAt: row.createdAt,
        updatedAt: row.updatedAt,
    };
}
function getAllGitRepos() {
    const rows = (0, db_core_1.getDatabase)().prepare('SELECT * FROM git_repos ORDER BY updatedAt DESC').all();
    return rows.map(rowToGitRepo);
}
function getGitRepoById(id) {
    const row = (0, db_core_1.getDatabase)().prepare('SELECT * FROM git_repos WHERE id = ?').get(id);
    return row ? rowToGitRepo(row) : null;
}
function addGitRepo(repo) {
    const now = new Date().toISOString();
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    INSERT INTO git_repos (id, name, path, remote, branch, lastOpened, createdAt, updatedAt)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
  `);
    stmt.run(repo.id, repo.name, repo.path, repo.remote || null, repo.branch || null, now, now, now);
    return { ...repo, lastOpened: now, createdAt: now, updatedAt: now };
}
function updateGitRepo(id, updates) {
    const existing = getGitRepoById(id);
    if (!existing)
        return null;
    const now = new Date().toISOString();
    const fields = [];
    const values = [];
    if (updates.name !== undefined) {
        fields.push('name = ?');
        values.push(updates.name);
    }
    if (updates.path !== undefined) {
        fields.push('path = ?');
        values.push(updates.path);
    }
    if (updates.remote !== undefined) {
        fields.push('remote = ?');
        values.push(updates.remote || null);
    }
    if (updates.branch !== undefined) {
        fields.push('branch = ?');
        values.push(updates.branch || null);
    }
    if (updates.lastOpened !== undefined) {
        fields.push('lastOpened = ?');
        values.push(updates.lastOpened);
    }
    fields.push('updatedAt = ?');
    values.push(now);
    values.push(id);
    const stmt = (0, db_core_1.getDatabase)().prepare(`UPDATE git_repos SET ${fields.join(', ')} WHERE id = ?`);
    stmt.run(...values);
    return getGitRepoById(id);
}
function deleteGitRepo(id) {
    const result = (0, db_core_1.getDatabase)().prepare('DELETE FROM git_repos WHERE id = ?').run(id);
    return result.changes > 0;
}
module.exports = {
    rowToGitRepo,
    getAllGitRepos,
    getGitRepoById,
    addGitRepo,
    updateGitRepo,
    deleteGitRepo,
};
//# sourceMappingURL=db-git.js.map