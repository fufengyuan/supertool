import { type Project, type Todo } from './db-core';
declare function rowToProject(row: any): Project;
declare function getAllProjects(onlyActive?: boolean): Project[];
declare function addProject(project: Project): Project;
declare function updateProject(project: Project): Project;
declare function deleteProject(id: string): string;
declare function getProjectStats(projectId: string): {
    total: number;
    completed: number;
    progress: number;
};
declare function getTodosByProject(projectId: string): Todo[];
declare const _default: {
    rowToProject: typeof rowToProject;
    getAllProjects: typeof getAllProjects;
    addProject: typeof addProject;
    updateProject: typeof updateProject;
    deleteProject: typeof deleteProject;
    getProjectStats: typeof getProjectStats;
    getTodosByProject: typeof getTodosByProject;
};
export = _default;
