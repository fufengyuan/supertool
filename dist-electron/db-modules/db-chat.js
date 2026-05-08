"use strict";
const db_core_1 = require("./db-core");
function getAllUsers() {
    const stmt = (0, db_core_1.getDatabase)().prepare('SELECT * FROM users ORDER BY lastSeen DESC');
    const rows = stmt.all();
    return rows.map((row) => ({
        id: row.id,
        name: row.name,
        ip: row.ip,
        port: row.port,
        lastSeen: row.lastSeen,
        isOnline: row.isOnline === 1
    }));
}
function upsertUser(user) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    INSERT INTO users (id, name, ip, port, lastSeen, isOnline)
    VALUES (?, ?, ?, ?, ?, ?)
    ON CONFLICT(id) DO UPDATE SET
      name = ?, ip = ?, port = ?, lastSeen = ?, isOnline = ?
  `);
    stmt.run(user.id, user.name, user.ip, user.port, user.lastSeen, user.isOnline ? 1 : 0, user.name, user.ip, user.port, user.lastSeen, user.isOnline ? 1 : 0);
    return user;
}
function updateUserOnlineStatus(userId, isOnline) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    UPDATE users SET isOnline = ?, lastSeen = ? WHERE id = ?
  `);
    stmt.run(isOnline ? 1 : 0, new Date().toISOString(), userId);
    return userId;
}
function deleteUser(userId) {
    const stmt = (0, db_core_1.getDatabase)().prepare('DELETE FROM users WHERE id = ?');
    stmt.run(userId);
    return userId;
}
function rowToMessage(row) {
    return {
        id: row.id,
        fromUserId: row.fromUserId,
        fromUserName: row.fromUserName,
        toUserId: row.toUserId,
        toUserName: row.toUserName,
        content: row.content,
        type: row.type,
        createdAt: row.createdAt,
        read: row.read === 1
    };
}
function getAllMessages() {
    const stmt = (0, db_core_1.getDatabase)().prepare('SELECT * FROM messages ORDER BY createdAt DESC');
    const rows = stmt.all();
    return rows.map(rowToMessage);
}
function getMessagesWithUser(userId, currentUserId) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    SELECT * FROM messages
    WHERE (fromUserId = ? AND toUserId = ?) OR (fromUserId = ? AND toUserId = ?)
    ORDER BY createdAt ASC
  `);
    const rows = stmt.all(userId, currentUserId, currentUserId, userId);
    return rows.map(rowToMessage);
}
function addMessage(message) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    INSERT INTO messages (id, fromUserId, fromUserName, toUserId, toUserName, content, type, createdAt, read)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);
    stmt.run(message.id, message.fromUserId, message.fromUserName, message.toUserId, message.toUserName, message.content, message.type || 'text', message.createdAt, message.read ? 1 : 0);
    return message;
}
function markMessageRead(messageId) {
    const stmt = (0, db_core_1.getDatabase)().prepare('UPDATE messages SET read = 1 WHERE id = ?');
    stmt.run(messageId);
    return messageId;
}
function getUnreadMessageCount(userId) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    SELECT COUNT(*) as count FROM messages WHERE toUserId = ? AND read = 0
  `);
    const row = stmt.get(userId);
    return row ? row.count : 0;
}
function rowToFileTransfer(row) {
    return {
        id: row.id,
        fromUserId: row.fromUserId,
        fromUserName: row.fromUserName,
        toUserId: row.toUserId,
        toUserName: row.toUserName,
        fileName: row.fileName,
        fileSize: row.fileSize,
        filePath: row.filePath,
        status: row.status,
        progress: row.progress,
        createdAt: row.createdAt,
        completedAt: row.completedAt
    };
}
function getAllFileTransfers() {
    const stmt = (0, db_core_1.getDatabase)().prepare('SELECT * FROM file_transfers ORDER BY createdAt DESC');
    const rows = stmt.all();
    return rows.map(rowToFileTransfer);
}
function addFileTransfer(transfer) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    INSERT INTO file_transfers (id, fromUserId, fromUserName, toUserId, toUserName, fileName, fileSize, filePath, status, progress, createdAt)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);
    stmt.run(transfer.id, transfer.fromUserId, transfer.fromUserName, transfer.toUserId, transfer.toUserName, transfer.fileName, transfer.fileSize, transfer.filePath || '', transfer.status || 'pending', transfer.progress || 0, transfer.createdAt);
    return transfer;
}
function updateFileTransferProgress(transferId, progress, status) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    UPDATE file_transfers SET progress = ?, status = ? WHERE id = ?
  `);
    stmt.run(progress, status, transferId);
    return transferId;
}
function completeFileTransfer(transferId, filePath) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    UPDATE file_transfers SET status = 'completed', progress = 100, filePath = ?, completedAt = ? WHERE id = ?
  `);
    stmt.run(filePath, new Date().toISOString(), transferId);
    return transferId;
}
function deleteFileTransfer(transferId) {
    const stmt = (0, db_core_1.getDatabase)().prepare('DELETE FROM file_transfers WHERE id = ?');
    stmt.run(transferId);
    return transferId;
}
function saveChatMessage(message) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    INSERT INTO chat_messages (id, fromUserId, fromUserName, toUserId, toUserName, content, type, fileName, fileSize, filePath, status, progress, createdAt, read)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);
    stmt.run(message.id, message.fromUserId, message.fromUserName, message.toUserId, message.toUserName, message.content || null, message.type || 'text', message.fileName || null, message.fileSize || null, message.filePath || null, message.status || 'sent', message.progress || 0, message.createdAt, message.read ? 1 : 0);
    return message;
}
function getChatMessages(limit = 100, offset = 0) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    SELECT * FROM chat_messages
    ORDER BY createdAt DESC
    LIMIT ? OFFSET ?
  `);
    return stmt.all(limit, offset);
}
function getChatMessagesBetween(userId1, userId2, limit = 100, offset = 0) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    SELECT * FROM chat_messages
    WHERE (fromUserId = ? AND toUserId = ?) OR (fromUserId = ? AND toUserId = ?)
    ORDER BY createdAt DESC
    LIMIT ? OFFSET ?
  `);
    return stmt.all(userId1, userId2, userId2, userId1, limit, offset);
}
function markMessagesAsRead(userId1, userId2) {
    const result = (0, db_core_1.getDatabase)().prepare(`
    UPDATE chat_messages SET read = 1
    WHERE toUserId = ? AND fromUserId = ? AND (read = 0 OR read IS NULL)
  `).run(userId1, userId2);
    return result.changes || 0;
}
function getUnreadCount(userId1, userId2) {
    const row = (0, db_core_1.getDatabase)().prepare(`
    SELECT COUNT(*) as count FROM chat_messages
    WHERE toUserId = ? AND fromUserId = ? AND (read = 0 OR read IS NULL)
  `).get(userId1, userId2);
    return row?.count || 0;
}
function getAllUnreadCounts(myUserId) {
    const rows = (0, db_core_1.getDatabase)().prepare(`
    SELECT fromUserId, COUNT(*) as count FROM chat_messages
    WHERE toUserId = ? AND (read = 0 OR read IS NULL)
    GROUP BY fromUserId
  `).all(myUserId);
    const result = {};
    for (const row of rows) {
        result[row.fromUserId] = row.count;
    }
    return result;
}
module.exports = {
    getAllUsers,
    upsertUser,
    updateUserOnlineStatus,
    deleteUser,
    rowToMessage,
    getAllMessages,
    getMessagesWithUser,
    addMessage,
    markMessageRead,
    getUnreadMessageCount,
    rowToFileTransfer,
    getAllFileTransfers,
    addFileTransfer,
    updateFileTransferProgress,
    completeFileTransfer,
    deleteFileTransfer,
    saveChatMessage,
    getChatMessages,
    getChatMessagesBetween,
    markMessagesAsRead,
    getUnreadCount,
    getAllUnreadCounts,
};
//# sourceMappingURL=db-chat.js.map