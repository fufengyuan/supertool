"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const logger_1 = require("./logger");
const electron_1 = require("electron");
(0, logger_1.info)('[Editor Preload] Loading...');
(0, logger_1.info)(`[Editor Preload] contextIsolation: ${process.contextIsolated}`);
electron_1.contextBridge.exposeInMainWorld('editorAPI', {
    sendEditorReady: (sessionId) => {
        (0, logger_1.info)(`[Editor Preload] sendEditorReady called, sessionId: ${sessionId}`);
        electron_1.ipcRenderer.send('editor-ready', sessionId);
    },
    onFileContent: (callback) => {
        (0, logger_1.info)('[Editor Preload] onFileContent listener registered');
        electron_1.ipcRenderer.on('file-content', (_event, data) => {
            (0, logger_1.info)(`[Editor Preload] file-content received, fileName: ${data?.fileName}`);
            callback(data);
        });
    },
    onSaveStatus: (callback) => {
        (0, logger_1.info)('[Editor Preload] onSaveStatus listener registered');
        electron_1.ipcRenderer.on('save-status', (_event, data) => callback(data));
    },
    invokeSave: (data) => {
        (0, logger_1.info)(`[Editor Preload] invokeSave called, sessionId: ${data.sessionId}, content length: ${data.content?.length}`);
        return electron_1.ipcRenderer.invoke('save-file-content', data);
    },
    sendCloseAfterSave: (sessionId) => {
        (0, logger_1.info)(`[Editor Preload] sendCloseAfterSave called, sessionId: ${sessionId}`);
        electron_1.ipcRenderer.send('close-window-after-save', sessionId);
    },
});
(0, logger_1.info)('[Editor Preload] contextBridge.exposeInMainWorld done');
//# sourceMappingURL=editor-preload.js.map