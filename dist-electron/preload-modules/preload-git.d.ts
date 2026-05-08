declare const _default: {
    getGitCommits: (gitUrl: string, sinceDate: string, branch?: string) => Promise<any>;
    getGitBranches: (gitUrl: string) => Promise<any>;
    scanLocalGitRepos: () => Promise<any>;
    validateGitRepoPath: (path: string) => Promise<any>;
    gitStatus: (repoPath: string) => Promise<any>;
    gitLog: (repoPath: string, options?: {
        limit?: number;
        branch?: string;
        author?: string;
    }) => Promise<any>;
    gitBranches: (repoPath: string) => Promise<any>;
    gitCurrentBranch: (repoPath: string) => Promise<any>;
    gitDiff: (repoPath: string, file?: string) => Promise<any>;
    gitCommitDiff: (repoPath: string, commitHash: string) => Promise<any>;
    gitCommit: (repoPath: string, message: string, files?: string[]) => Promise<any>;
    gitAdd: (repoPath: string, files: string | string[]) => Promise<any>;
    gitReset: (repoPath: string, file?: string) => Promise<any>;
    gitCheckout: (repoPath: string, branch: string) => Promise<any>;
    gitCreateBranch: (repoPath: string, branchName: string, from?: string) => Promise<any>;
    gitDeleteBranch: (repoPath: string, branchName: string, force?: boolean) => Promise<any>;
    gitMerge: (repoPath: string, branch: string) => Promise<any>;
    gitPull: (repoPath: string) => Promise<any>;
    gitPush: (repoPath: string) => Promise<any>;
    gitDiscardChanges: (repoPath: string, file: string) => Promise<any>;
    gitStashSave: (repoPath: string, options?: {
        message?: string;
        includeUntracked?: boolean;
        keepIndex?: boolean;
    }) => Promise<any>;
    gitStashList: (repoPath: string) => Promise<any>;
    gitStashApply: (repoPath: string, stashRef?: string) => Promise<any>;
    gitStashDrop: (repoPath: string, stashRef?: string) => Promise<any>;
    gitStashPop: (repoPath: string, stashRef?: string) => Promise<any>;
    gitStashShow: (repoPath: string, stashRef?: string) => Promise<any>;
    gitCherryPick: (repoPath: string, commitHash: string, options?: {
        noCommit?: boolean;
    }) => Promise<any>;
    gitRevert: (repoPath: string, commitHash: string, options?: {
        noCommit?: boolean;
        noEdit?: boolean;
    }) => Promise<any>;
    gitListTags: (repoPath: string) => Promise<any>;
    gitCreateTag: (repoPath: string, tagName: string, options?: {
        message?: string;
        commit?: string;
        force?: boolean;
        annotated?: boolean;
    }) => Promise<any>;
    gitDeleteTag: (repoPath: string, tagName: string) => Promise<any>;
    gitFileHistory: (repoPath: string, filePath: string, options?: {
        limit?: number;
        branch?: string;
    }) => Promise<any>;
    gitCompareBranches: (repoPath: string, targetBranch: string, sourceBranch?: string) => Promise<any>;
    gitLogGraph: (repoPath: string, limit?: number) => Promise<any>;
    gitExec: (repoPath: string, args: string[]) => Promise<any>;
    gitAmendCommit: (repoPath: string, message?: string) => Promise<any>;
    gitResetToCommit: (repoPath: string, commitHash: string, mode?: string) => Promise<any>;
    gitRenameBranch: (repoPath: string, oldName: string, newName: string) => Promise<any>;
    gitFetch: (repoPath: string, remote?: string) => Promise<any>;
    gitForcePush: (repoPath: string) => Promise<any>;
    gitUnpushedCommits: (repoPath: string) => Promise<any>;
    gitIncomingCommits: (repoPath: string) => Promise<any>;
    gitCheckoutRemoteBranch: (repoPath: string, remote: string, branch: string) => Promise<any>;
    gitConflictFiles: (repoPath: string) => Promise<any>;
    gitAcceptConflict: (repoPath: string, file: string, strategy: string) => Promise<any>;
    gitInteractiveRebaseList: (repoPath: string, commitHash: string) => Promise<any>;
    gitInteractiveRebase: (repoPath: string, commitHash: string, actions: string[]) => Promise<any>;
    gitRemotes: (repoPath: string) => Promise<any>;
    gitAddRemote: (repoPath: string, name: string, url: string) => Promise<any>;
    gitRemoveRemote: (repoPath: string, name: string) => Promise<any>;
    gitSetRemoteUrl: (repoPath: string, name: string, url: string) => Promise<any>;
    gitAddGitignore: (repoPath: string, pattern: string) => Promise<any>;
    gitGetGitignore: (repoPath: string) => Promise<any>;
    gitSubmodules: (repoPath: string) => Promise<any>;
    gitUndoLastCommit: (repoPath: string) => Promise<any>;
    gitPushTags: (repoPath: string, remote?: string) => Promise<any>;
    gitDeleteRemoteBranch: (repoPath: string, remote: string, branchName: string) => Promise<any>;
    gitClean: (repoPath: string, options?: {
        dryRun?: boolean;
        force?: boolean;
    }) => Promise<any>;
    gitFileAtRevision: (repoPath: string, filePath: string, revision: string) => Promise<any>;
    gitDiffFileRevision: (repoPath: string, filePath: string, rev1: string, rev2?: string) => Promise<any>;
    gitCompareCommits: (repoPath: string, commit1: string, commit2: string, filePath?: string) => Promise<any>;
    gitCreatePatch: (repoPath: string, commitRange?: string, filePaths?: string[]) => Promise<any>;
    gitApplyPatch: (repoPath: string, patchContent: string, options?: {
        dryRun?: boolean;
        index?: boolean;
    }) => Promise<any>;
    gitCherryPickMultiple: (repoPath: string, hashes: string[], options?: {
        noCommit?: boolean;
    }) => Promise<any>;
    gitBranchFromTag: (repoPath: string, branchName: string, tagName: string) => Promise<any>;
    gitSubmoduleList: (repoPath: string) => Promise<any>;
    gitSubmoduleInit: (repoPath: string, recursive?: boolean) => Promise<any>;
    gitCommitCount: (repoPath: string, branch?: string) => Promise<any>;
    gitChangedFiles: (repoPath: string, commit1: string, commit2?: string) => Promise<any>;
    gitRebase: (repoPath: string, targetBranch: string, options?: {
        onto?: string;
        interactive?: boolean;
    }) => Promise<any>;
    gitRebaseAbort: (repoPath: string) => Promise<any>;
    gitRebaseContinue: (repoPath: string) => Promise<any>;
    gitFileBlame: (repoPath: string, filePath: string) => Promise<any>;
    gitSyncConfigure: (config: Record<string, string>) => Promise<any>;
    gitSyncInit: () => Promise<any>;
    gitSyncPull: () => Promise<any>;
    gitSyncPush: () => Promise<any>;
    gitSyncStatus: () => Promise<any>;
    onGitSyncStatusUpdated: (callback: (data: any) => void) => Electron.IpcRenderer;
};
export default _default;
