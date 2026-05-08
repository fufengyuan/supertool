"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
/** Notes + Note Groups API */
const electron_1 = require("electron");
exports.default = {
    // Note Groups API
    getNoteGroups: () => electron_1.ipcRenderer.invoke('note-groups:get-all'),
    addNoteGroup: (data) => electron_1.ipcRenderer.invoke('note-groups:add', data),
    updateNoteGroup: (id, updates) => electron_1.ipcRenderer.invoke('note-groups:update', id, updates),
    deleteNoteGroup: (id) => electron_1.ipcRenderer.invoke('note-groups:delete', id),
    // Notes
    getAllNotes: (query, groupId) => electron_1.ipcRenderer.invoke('notes:get-all', query, groupId),
    getNoteById: (id) => electron_1.ipcRenderer.invoke('notes:get-by-id', id),
    addNote: (data) => electron_1.ipcRenderer.invoke('notes:add', data),
    updateNote: (id, updates) => electron_1.ipcRenderer.invoke('notes:update', id, updates),
    deleteNote: (id) => electron_1.ipcRenderer.invoke('notes:delete', id),
};
//# sourceMappingURL=preload-notes.js.map