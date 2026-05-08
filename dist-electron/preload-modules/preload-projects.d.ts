import type { Project } from './preload-core';
declare const _default: {
    getProjects: (onlyActive: boolean) => Promise<any>;
    addProject: (project: Project) => Promise<any>;
    updateProject: (project: Project) => Promise<any>;
    deleteProject: (id: string) => Promise<any>;
    getProjectStats: (projectId: string) => Promise<any>;
    getProjectTodos: (projectId: string) => Promise<any>;
    getGitRepos: () => Promise<any>;
    addGitRepo: (repo: {
        id: string;
        name: string;
        path: string;
        remote?: string;
        branch?: string;
    }) => Promise<any>;
    updateGitRepo: (id: string, updates: Record<string, unknown>) => Promise<any>;
    deleteGitRepo: (id: string) => Promise<any>;
};
export default _default;
