/** Database interface for git repo operations */
interface GitRepoDb {
    getAllGitRepos(): unknown[];
    addGitRepo(repo: GitRepoAddInput): unknown;
    updateGitRepo(id: string, updates: Record<string, unknown>): unknown;
    deleteGitRepo(id: string): unknown;
}
interface GitRepoAddInput {
    path: string;
    name: string;
    [key: string]: unknown;
}
export declare function registerGitHandlers(db: GitRepoDb): void;
export {};
