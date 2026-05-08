"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
/** Log Aggregator API + Events */
const electron_1 = require("electron");
exports.default = {
    logPresetsGetAll: () => electron_1.ipcRenderer.invoke('logPresets:get-all'),
    logPresetsAdd: (data) => electron_1.ipcRenderer.invoke('logPresets:add', data),
    logPresetsUpdate: (id, updates) => electron_1.ipcRenderer.invoke('logPresets:update', id, updates),
    logPresetsDelete: (id) => electron_1.ipcRenderer.invoke('logPresets:delete', id),
    logsStartStream: (streamId, params) => electron_1.ipcRenderer.invoke('logs:start-stream', streamId, params),
    logsStopStream: (streamId) => electron_1.ipcRenderer.invoke('logs:stop-stream', streamId),
    logsSearch: (params) => electron_1.ipcRenderer.invoke('logs:search', params),
    onLogsLine: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('logs:line', wrapper);
        return () => electron_1.ipcRenderer.removeListener('logs:line', wrapper);
    },
    onLogsServerEnd: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('logs:server-end', wrapper);
        return () => electron_1.ipcRenderer.removeListener('logs:server-end', wrapper);
    },
    onLogsError: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('logs:error', wrapper);
        return () => electron_1.ipcRenderer.removeListener('logs:error', wrapper);
    },
    onLogsStreamStopped: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('logs:stream-stopped', wrapper);
        return () => electron_1.ipcRenderer.removeListener('logs:stream-stopped', wrapper);
    },
};
//# sourceMappingURL=preload-logs.js.map