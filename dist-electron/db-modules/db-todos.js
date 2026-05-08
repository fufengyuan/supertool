"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getAllTodos = getAllTodos;
exports.addTodo = addTodo;
exports.getTodoById = getTodoById;
exports.updateTodo = updateTodo;
exports.deleteTodo = deleteTodo;
exports.deleteTodos = deleteTodos;
exports.getAllTags = getAllTags;
exports.addTag = addTag;
exports.deleteTag = deleteTag;
exports.getSetting = getSetting;
exports.setSetting = setSetting;
const db_core_1 = require("./db-core");
const crypto_1 = require("crypto");
function getAllTodos() {
    const stmt = (0, db_core_1.getDatabase)().prepare('SELECT * FROM todos ORDER BY orderNum ASC, createdAt DESC');
    const rows = stmt.all();
    return rows.map(db_core_1.rowToTodo);
}
function addTodo(todo) {
    const now = new Date().toISOString();
    if (!todo.id)
        todo.id = (0, crypto_1.randomUUID)();
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    INSERT INTO todos (id, text, completed, priority, dueDate, description, markdownDescription, tag, createdAt, updatedAt, assignedTo, assignedBy, assignedAt, owner, repeatType, repeatInterval, repeatEndDate, repeatCount, parentTodoId, projectId)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);
    stmt.run(todo.id, todo.text, todo.completed ? 1 : 0, todo.priority || 'medium', todo.dueDate || null, todo.description || '', todo.markdownDescription || '', todo.tag || '', todo.createdAt || now, todo.updatedAt || now, todo.assignedTo || '', todo.assignedBy || '', todo.assignedAt || null, todo.owner || '', todo.repeatType || '', todo.repeatInterval || 1, todo.repeatEndDate || null, todo.repeatCount || -1, todo.parentTodoId || null, todo.projectId || null);
    return getTodoById(todo.id);
}
function getTodoById(id) {
    const stmt = (0, db_core_1.getDatabase)().prepare('SELECT * FROM todos WHERE id = ?');
    const row = stmt.get(id);
    if (!row)
        return null;
    return (0, db_core_1.rowToTodo)(row);
}
function updateTodo(todo) {
    // Fetch existing todo to preserve fields not in update
    const existing = (0, db_core_1.getDatabase)().prepare('SELECT * FROM todos WHERE id = ?').get(todo.id);
    if (!existing)
        return todo;
    const now = new Date().toISOString();
    const fields = { updatedAt: now };
    if (todo.text !== undefined)
        fields.text = todo.text;
    if (todo.completed !== undefined)
        fields.completed = todo.completed ? 1 : 0;
    if (todo.priority !== undefined)
        fields.priority = todo.priority;
    if (todo.dueDate !== undefined)
        fields.dueDate = todo.dueDate || null;
    if (todo.description !== undefined)
        fields.description = todo.description || '';
    if (todo.markdownDescription !== undefined)
        fields.markdownDescription = todo.markdownDescription || '';
    if (todo.tag !== undefined)
        fields.tag = todo.tag || '';
    if (todo.updatedAt !== undefined)
        fields.updatedAt = todo.updatedAt;
    if (todo.completedAt !== undefined)
        fields.completedAt = todo.completedAt || null;
    if (todo.assignedTo !== undefined)
        fields.assignedTo = todo.assignedTo || '';
    if (todo.assignedBy !== undefined)
        fields.assignedBy = todo.assignedBy || '';
    if (todo.assignedAt !== undefined)
        fields.assignedAt = todo.assignedAt || null;
    if (todo.owner !== undefined)
        fields.owner = todo.owner || '';
    if (todo.repeatType !== undefined)
        fields.repeatType = todo.repeatType || '';
    if (todo.repeatInterval !== undefined)
        fields.repeatInterval = todo.repeatInterval || 1;
    if (todo.repeatEndDate !== undefined)
        fields.repeatEndDate = todo.repeatEndDate || null;
    if (todo.repeatCount !== undefined)
        fields.repeatCount = todo.repeatCount || -1;
    if (todo.parentTodoId !== undefined)
        fields.parentTodoId = todo.parentTodoId || null;
    if (todo.projectId !== undefined)
        fields.projectId = todo.projectId || null;
    const setClause = Object.keys(fields).map(k => `${k} = ?`).join(', ');
    const values = [...Object.values(fields), todo.id];
    const stmt = (0, db_core_1.getDatabase)().prepare(`UPDATE todos SET ${setClause} WHERE id = ?`);
    stmt.run(...values);
    return { ...todo, updatedAt: now };
}
function deleteTodo(id) {
    const stmt = (0, db_core_1.getDatabase)().prepare('DELETE FROM todos WHERE id = ?');
    stmt.run(id);
    return id;
}
function deleteTodos(ids) {
    const stmt = (0, db_core_1.getDatabase)().prepare('DELETE FROM todos WHERE id = ?');
    const deleteMany = (0, db_core_1.getDatabase)().transaction((ids) => {
        ids.forEach(id => stmt.run(id));
    });
    deleteMany(ids);
    return ids;
}
function getAllTags() {
    const stmt = (0, db_core_1.getDatabase)().prepare('SELECT name FROM tags ORDER BY name');
    return stmt.all().map((row) => row.name);
}
function addTag(name) {
    const stmt = (0, db_core_1.getDatabase)().prepare('INSERT INTO tags (name, createdAt) VALUES (?, ?)');
    stmt.run(name, new Date().toISOString());
    return name;
}
function deleteTag(name) {
    const stmt = (0, db_core_1.getDatabase)().prepare('DELETE FROM tags WHERE name = ?');
    stmt.run(name);
    return name;
}
function getSetting(key) {
    const stmt = (0, db_core_1.getDatabase)().prepare('SELECT value FROM settings WHERE key = ?');
    const row = stmt.get(key);
    return row ? row.value : null;
}
function setSetting(key, value) {
    const stmt = (0, db_core_1.getDatabase)().prepare('INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)');
    stmt.run(key, value);
    return { key, value };
}
//# sourceMappingURL=db-todos.js.map