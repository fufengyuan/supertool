"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
/** Projects, Git Repo Management API */
const electron_1 = require("electron");
exports.default = {
    // Project API
    getProjects: (onlyActive) => electron_1.ipcRenderer.invoke('projects:get-all', onlyActive),
    addProject: (project) => electron_1.ipcRenderer.invoke('projects:add', project),
    updateProject: (project) => electron_1.ipcRenderer.invoke('projects:update', project),
    deleteProject: (id) => electron_1.ipcRenderer.invoke('projects:delete', id),
    getProjectStats: (projectId) => electron_1.ipcRenderer.invoke('projects:get-stats', projectId),
    getProjectTodos: (projectId) => electron_1.ipcRenderer.invoke('projects:get-todos', projectId),
    // Git Repo Management API
    getGitRepos: () => electron_1.ipcRenderer.invoke('git:repos:get-all'),
    addGitRepo: (repo) => electron_1.ipcRenderer.invoke('git:repos:add', repo),
    updateGitRepo: (id, updates) => electron_1.ipcRenderer.invoke('git:repos:update', id, updates),
    deleteGitRepo: (id) => electron_1.ipcRenderer.invoke('git:repos:delete', id),
};
//# sourceMappingURL=preload-projects.js.map