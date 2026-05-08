import { type GitRepoRecord } from './db-core';
declare function rowToGitRepo(row: any): GitRepoRecord;
declare function getAllGitRepos(): GitRepoRecord[];
declare function getGitRepoById(id: string): GitRepoRecord | null;
declare function addGitRepo(repo: {
    id: string;
    name: string;
    path: string;
    remote?: string;
    branch?: string;
}): GitRepoRecord;
declare function updateGitRepo(id: string, updates: Partial<GitRepoRecord>): GitRepoRecord | null;
declare function deleteGitRepo(id: string): boolean;
declare const _default: {
    rowToGitRepo: typeof rowToGitRepo;
    getAllGitRepos: typeof getAllGitRepos;
    getGitRepoById: typeof getGitRepoById;
    addGitRepo: typeof addGitRepo;
    updateGitRepo: typeof updateGitRepo;
    deleteGitRepo: typeof deleteGitRepo;
};
export = _default;
