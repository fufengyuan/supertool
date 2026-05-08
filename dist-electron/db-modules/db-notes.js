"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getAllNoteGroups = getAllNoteGroups;
exports.addNoteGroup = addNoteGroup;
exports.updateNoteGroup = updateNoteGroup;
exports.deleteNoteGroup = deleteNoteGroup;
exports.getAllNotes = getAllNotes;
exports.getNoteById = getNoteById;
exports.addNote = addNote;
exports.updateNote = updateNote;
exports.deleteNote = deleteNote;
const db_core_1 = require("./db-core");
function getAllNoteGroups() {
    const rows = (0, db_core_1.getDatabase)().prepare('SELECT * FROM note_groups ORDER BY sortOrder ASC, createdAt ASC').all();
    return rows;
}
function addNoteGroup(data) {
    const id = data.id || 'ng_' + Date.now().toString(36) + Math.random().toString(36).slice(2, 6);
    const now = new Date().toISOString();
    (0, db_core_1.getDatabase)().prepare(`
    INSERT INTO note_groups (id, name, icon, sortOrder, createdAt)
    VALUES (?, ?, ?, ?, ?)
  `).run(id, data.name, data.icon || '', data.sortOrder || 0, now);
    return (0, db_core_1.getDatabase)().prepare('SELECT * FROM note_groups WHERE id = ?').get(id);
}
function updateNoteGroup(id, updates) {
    const fields = [];
    const values = [];
    if (updates.name !== undefined) {
        fields.push('name = ?');
        values.push(updates.name);
    }
    if (updates.icon !== undefined) {
        fields.push('icon = ?');
        values.push(updates.icon);
    }
    if (updates.sortOrder !== undefined) {
        fields.push('sortOrder = ?');
        values.push(updates.sortOrder);
    }
    if (fields.length === 0) {
        return (0, db_core_1.getDatabase)().prepare('SELECT * FROM note_groups WHERE id = ?').get(id) || null;
    }
    values.push(id);
    (0, db_core_1.getDatabase)().prepare(`UPDATE note_groups SET ${fields.join(', ')} WHERE id = ?`).run(...values);
    return (0, db_core_1.getDatabase)().prepare('SELECT * FROM note_groups WHERE id = ?').get(id) || null;
}
function deleteNoteGroup(id) {
    const result = (0, db_core_1.getDatabase)().prepare('DELETE FROM note_groups WHERE id = ?').run(id);
    return result.changes > 0;
}
function getAllNotes(query, groupId) {
    let sql = 'SELECT * FROM notes';
    const params = [];
    const conditions = [];
    if (groupId === '__ungrouped__') {
        conditions.push("(groupId IS NULL OR groupId = '')");
    }
    else if (groupId) {
        conditions.push('groupId = ?');
        params.push(groupId);
    }
    if (query) {
        conditions.push('(title LIKE ? OR content LIKE ?)');
        params.push(`%${query}%`, `%${query}%`);
    }
    if (conditions.length > 0) {
        sql += ' WHERE ' + conditions.join(' AND ');
    }
    sql += ' ORDER BY pinned DESC, updatedAt DESC';
    const rows = (0, db_core_1.getDatabase)().prepare(sql).all(...params);
    return rows;
}
function getNoteById(id) {
    const row = (0, db_core_1.getDatabase)().prepare('SELECT * FROM notes WHERE id = ?').get(id);
    return row || null;
}
function addNote(data) {
    const id = data.id || 'note_' + Date.now().toString(36) + Math.random().toString(36).slice(2, 6);
    const now = new Date().toISOString();
    (0, db_core_1.getDatabase)().prepare(`
    INSERT INTO notes (id, title, content, tags, pinned, groupId, createdAt, updatedAt)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
  `).run(id, data.title || '', data.content || '', data.tags || '[]', data.pinned ? 1 : 0, data.groupId || null, now, now);
    return (0, db_core_1.getDatabase)().prepare('SELECT * FROM notes WHERE id = ?').get(id);
}
function updateNote(id, updates) {
    const now = new Date().toISOString();
    const fields = [];
    const values = [];
    if (updates.title !== undefined) {
        fields.push('title = ?');
        values.push(updates.title);
    }
    if (updates.content !== undefined) {
        fields.push('content = ?');
        values.push(updates.content);
    }
    if (updates.tags !== undefined) {
        fields.push('tags = ?');
        values.push(updates.tags);
    }
    if (updates.pinned !== undefined) {
        fields.push('pinned = ?');
        values.push(updates.pinned ? 1 : 0);
    }
    if (updates.groupId !== undefined) {
        fields.push('groupId = ?');
        values.push(updates.groupId || null);
    }
    if (fields.length === 0)
        return getNoteById(id);
    fields.push('updatedAt = ?');
    values.push(now);
    values.push(id);
    (0, db_core_1.getDatabase)().prepare(`UPDATE notes SET ${fields.join(', ')} WHERE id = ?`).run(...values);
    return getNoteById(id);
}
function deleteNote(id) {
    const result = (0, db_core_1.getDatabase)().prepare('DELETE FROM notes WHERE id = ?').run(id);
    return result.changes > 0;
}
//# sourceMappingURL=db-notes.js.map