"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
/** API Debugger (HTTP Request) API */
const electron_1 = require("electron");
exports.default = {
    apiHttpRequest: (config) => electron_1.ipcRenderer.invoke('api:http-request', JSON.parse(JSON.stringify(config))),
    apiRequestsGetAll: () => electron_1.ipcRenderer.invoke('api:requests:get-all'),
    apiRequestsAdd: (request) => electron_1.ipcRenderer.invoke('api:requests:add', JSON.parse(JSON.stringify(request))),
    apiRequestsUpdate: (id, updates) => electron_1.ipcRenderer.invoke('api:requests:update', id, JSON.parse(JSON.stringify(updates))),
    apiRequestsDelete: (id) => electron_1.ipcRenderer.invoke('api:requests:delete', id),
};
//# sourceMappingURL=preload-api.js.map