"use strict";
const db_core_1 = require("./db-core");
const db_todos_1 = require("./db-todos");
function rowToSubtask(row) {
    return {
        id: row.id,
        todoId: row.todoId,
        text: row.text,
        completed: row.completed === 1,
        orderNum: row.orderNum,
        createdAt: row.createdAt,
        updatedAt: row.updatedAt
    };
}
function getSubtasksForTodo(todoId) {
    const stmt = (0, db_core_1.getDatabase)().prepare('SELECT * FROM subtasks WHERE todoId = ? ORDER BY orderNum ASC');
    const rows = stmt.all(todoId);
    return rows.map(rowToSubtask);
}
function addSubtask(subtask) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    INSERT INTO subtasks (id, todoId, text, completed, orderNum, createdAt, updatedAt)
    VALUES (?, ?, ?, ?, ?, ?, ?)
  `);
    stmt.run(subtask.id, subtask.todoId, subtask.text, subtask.completed ? 1 : 0, subtask.orderNum || 0, subtask.createdAt, subtask.updatedAt);
    return subtask;
}
function updateSubtask(subtask) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    UPDATE subtasks
    SET text = ?, completed = ?, orderNum = ?, updatedAt = ?
    WHERE id = ?
  `);
    stmt.run(subtask.text, subtask.completed ? 1 : 0, subtask.orderNum || 0, subtask.updatedAt, subtask.id);
    return subtask;
}
function deleteSubtask(subtaskId) {
    const stmt = (0, db_core_1.getDatabase)().prepare('DELETE FROM subtasks WHERE id = ?');
    stmt.run(subtaskId);
    return subtaskId;
}
function deleteSubtasksForTodo(todoId) {
    const stmt = (0, db_core_1.getDatabase)().prepare('DELETE FROM subtasks WHERE todoId = ?');
    stmt.run(todoId);
    return todoId;
}
// 更新任务的排序
function updateTodoOrder(todos) {
    const db = (0, db_core_1.getDatabase)();
    const updateStmt = db.prepare('UPDATE todos SET orderNum = ?, updatedAt = ? WHERE id = ?');
    const updateTransaction = db.transaction((todosArray) => {
        todosArray.forEach((todo, index) => {
            updateStmt.run(index, new Date().toISOString(), todo.id);
        });
    });
    updateTransaction(todos);
    return true;
}
// 根据重复规则计算下一个日期
function calculateNextDate(currentDate, repeatType, repeatInterval) {
    const date = new Date(currentDate);
    switch (repeatType) {
        case 'daily':
            date.setDate(date.getDate() + repeatInterval);
            break;
        case 'weekly':
            date.setDate(date.getDate() + (repeatInterval * 7));
            break;
        case 'monthly':
            date.setMonth(date.getMonth() + repeatInterval);
            break;
        case 'yearly':
            date.setFullYear(date.getFullYear() + repeatInterval);
            break;
        default:
            date.setDate(date.getDate() + repeatInterval);
    }
    return date.toISOString().split('T')[0];
}
// 检查重复任务是否应该生成新实例
function shouldCreateRepeatInstance(todo) {
    if (!todo.repeatType || todo.repeatType === '')
        return false;
    if (todo.repeatCount === 0)
        return false;
    if (todo.repeatEndDate) {
        const endDate = new Date(todo.repeatEndDate);
        const now = new Date();
        now.setHours(0, 0, 0, 0);
        if (now > endDate)
            return false;
    }
    return true;
}
// 创建重复任务的新实例
function createRepeatInstance(originalTodo) {
    if (!shouldCreateRepeatInstance(originalTodo)) {
        return null;
    }
    let newDueDate = null;
    if (originalTodo.dueDate) {
        newDueDate = calculateNextDate(originalTodo.dueDate, originalTodo.repeatType, originalTodo.repeatInterval);
    }
    const newTodo = {
        id: Date.now().toString() + '_' + Math.random().toString(36).substr(2, 5),
        text: originalTodo.text,
        completed: false,
        priority: originalTodo.priority,
        dueDate: newDueDate || undefined,
        description: originalTodo.description,
        tag: originalTodo.tag,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        completedAt: undefined,
        assignedTo: originalTodo.assignedTo,
        assignedBy: originalTodo.assignedBy,
        assignedAt: originalTodo.assignedAt,
        owner: originalTodo.owner,
        orderNum: originalTodo.orderNum,
        repeatType: originalTodo.repeatType,
        repeatInterval: originalTodo.repeatInterval,
        repeatEndDate: originalTodo.repeatEndDate,
        repeatCount: originalTodo.repeatCount > 0 ? originalTodo.repeatCount - 1 : -1,
        parentTodoId: originalTodo.parentTodoId || originalTodo.id,
        projectId: originalTodo.projectId,
        markdownDescription: originalTodo.markdownDescription
    };
    return (0, db_todos_1.addTodo)(newTodo);
}
// 更新任务的完成状态
function updateTodoCompletionBasedOnSubtasks(todoId) {
    const db = (0, db_core_1.getDatabase)();
    const statsStmt = db.prepare(`
    SELECT
      COUNT(*) as total,
      SUM(CASE WHEN completed = 1 THEN 1 ELSE 0 END) as completed
    FROM subtasks WHERE todoId = ?
  `);
    const stats = statsStmt.get(todoId);
    if (!stats || stats.total === 0) {
        return false;
    }
    const allCompleted = stats.total > 0 && stats.completed === stats.total;
    const updateStmt = db.prepare('UPDATE todos SET completed = ?, updatedAt = ? WHERE id = ?');
    updateStmt.run(allCompleted ? 1 : 0, new Date().toISOString(), todoId);
    return allCompleted;
}
module.exports = {
    rowToSubtask,
    getSubtasksForTodo,
    addSubtask,
    updateSubtask,
    deleteSubtask,
    deleteSubtasksForTodo,
    updateTodoOrder,
    calculateNextDate,
    shouldCreateRepeatInstance,
    createRepeatInstance,
    updateTodoCompletionBasedOnSubtasks,
};
//# sourceMappingURL=db-subtasks.js.map