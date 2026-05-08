export interface RepoStatus {
    modified: string[];
    added: string[];
    deleted: string[];
    untracked: string[];
    conflicted: string[];
    staged: string[];
    currentBranch: string;
    ahead: number;
    behind: number;
}
export interface LogEntry {
    hash: string;
    date: string;
    author: string;
    message: string;
    refs: string;
}
export interface BranchInfo {
    name: string;
    current: boolean;
    linked: boolean;
    commit: string;
    label: string;
}
export interface GitLogOptions {
    limit?: number;
    branch?: string;
    author?: string;
    authors?: string[];
    dateFrom?: string;
    dateTo?: string;
}
/**
 * 获取工作区状态
 */
export declare function getRepoStatus(repoPath: string): Promise<RepoStatus>;
/**
 * 获取提交历史
 */
export declare function getRepoLog(repoPath: string, options?: GitLogOptions): Promise<LogEntry[]>;
/**
 * 获取所有分支（本地 + 远程）
 */
export declare function getRepoBranches(repoPath: string): Promise<BranchInfo[]>;
/**
 * 获取当前分支
 */
export declare function getCurrentBranch(repoPath: string): Promise<string>;
/**
 * 获取文件差异
 */
export declare function getRepoDiff(repoPath: string, file?: string): Promise<string>;
/**
 * 获取指定提交的差异 (git show <hash>)
 */
export declare function getCommitDiff(repoPath: string, commitHash: string): Promise<string>;
/**
 * 提交更改
 */
export declare function commit(repoPath: string, message: string, files?: string[]): Promise<{
    success: boolean;
    hash?: string;
}>;
/**
 * 暂存文件
 */
export declare function add(repoPath: string, files: string | string[]): Promise<void>;
/**
 * 取消暂存
 */
export declare function reset(repoPath: string, file?: string): Promise<void>;
/**
 * 切换分支
 */
export declare function checkout(repoPath: string, branch: string): Promise<void>;
/**
 * 创建分支
 */
export declare function createBranch(repoPath: string, branchName: string, from?: string): Promise<void>;
/**
 * 删除分支
 */
export declare function deleteBranch(repoPath: string, branchName: string, force?: boolean): Promise<void>;
/**
 * 合并分支
 */
export declare function merge(repoPath: string, branch: string): Promise<void>;
/**
 * 拉取
 */
export declare function pull(repoPath: string): Promise<void>;
/**
 * 推送
 */
export declare function push(repoPath: string): Promise<void>;
/**
 * 丢弃更改
 */
export declare function discardChanges(repoPath: string, file: string): Promise<void>;
/**
 * 检查目录是否为有效的 Git 仓库
 */
export declare function isValidGitRepo(repoPath: string): Promise<boolean>;
/**
 * 获取远程 URL
 */
export declare function getRemoteUrl(repoPath: string, remoteName?: string): Promise<string | null>;
/**
 * 保存 stash
 */
export declare function gitStashSave(repoPath: string, options?: {
    message?: string;
    includeUntracked?: boolean;
    keepIndex?: boolean;
}): Promise<{
    success: boolean;
    data?: string;
    error?: string;
}>;
/**
 * 列出 stash
 */
export declare function gitStashList(repoPath: string): Promise<{
    success: boolean;
    data?: string[];
    error?: string;
}>;
/**
 * 应用 stash
 */
export declare function gitStashApply(repoPath: string, stashRef?: string): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 删除 stash
 */
export declare function gitStashDrop(repoPath: string, stashRef?: string): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 弹出 stash (apply + drop)
 */
export declare function gitStashPop(repoPath: string, stashRef?: string): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 显示 stash 内容
 */
export declare function gitStashShow(repoPath: string, stashRef?: string): Promise<{
    success: boolean;
    data?: string;
    error?: string;
}>;
/**
 * Cherry-pick 提交
 */
export declare function gitCherryPick(repoPath: string, commitHash: string, options?: {
    noCommit?: boolean;
}): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 撤销提交
 */
export declare function gitRevert(repoPath: string, commitHash: string, options?: {
    noCommit?: boolean;
    noEdit?: boolean;
}): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 列出标签
 */
export declare function gitListTags(repoPath: string): Promise<{
    success: boolean;
    data?: string[];
    error?: string;
}>;
/**
 * 创建标签
 */
export declare function gitCreateTag(repoPath: string, tagName: string, options?: {
    message?: string;
    commit?: string;
    force?: boolean;
    annotated?: boolean;
}): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 删除标签
 */
export declare function gitDeleteTag(repoPath: string, tagName: string): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 获取文件历史
 */
export declare function gitFileHistory(repoPath: string, filePath: string, options?: {
    limit?: number;
    branch?: string;
}): Promise<{
    success: boolean;
    data?: LogEntry[];
    error?: string;
}>;
/**
 * 比较两个分支的差异
 */
export declare function gitCompareBranches(repoPath: string, targetBranch: string, sourceBranch?: string): Promise<{
    success: boolean;
    data?: string;
    error?: string;
}>;
export declare function changedFiles(repoPath: string, commit1: string, commit2: string): Promise<{
    success: boolean;
    data?: Array<{
        file: string;
        status: string;
    }>;
    error?: string;
}>;
/**
 * 获取带图形可视化的提交历史
 */
export declare function getRepoLogGraph(repoPath: string, options?: {
    limit?: number;
    branch?: string;
}): Promise<{
    success: boolean;
    data?: string;
    error?: string;
}>;
/**
 * 执行任意 git 命令
 */
export declare function execGit(repoPath: string, args: string[]): Promise<{
    success: boolean;
    data?: {
        stdout: string;
        stderr: string;
    };
    error?: string;
}>;
/**
 * 修改最后一次提交
 */
export declare function amendCommit(repoPath: string, message: string): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 重置到指定提交
 */
export declare function resetToCommit(repoPath: string, commitHash: string, mode?: string): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 重命名分支
 */
export declare function renameBranch(repoPath: string, oldName: string, newName: string): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 获取远程更新
 */
export declare function fetchRepo(repoPath: string, remote?: string): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 强制推送
 */
export declare function forcePush(repoPath: string): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 获取未推送的提交
 */
export declare function unpushedCommits(repoPath: string): Promise<{
    success: boolean;
    data?: Array<{
        hash: string;
        message: string;
        author: string;
        date: string;
    }>;
    error?: string;
}>;
/**
 * 获取待拉取的提交
 */
export declare function incomingCommits(repoPath: string): Promise<{
    success: boolean;
    data?: Array<{
        hash: string;
        message: string;
        author: string;
        date: string;
    }>;
    error?: string;
}>;
/**
 * 检出远程分支
 */
export declare function checkoutRemoteBranch(repoPath: string, remote: string, branch: string): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 获取冲突文件列表
 */
export declare function conflictFiles(repoPath: string): Promise<{
    success: boolean;
    data?: string[];
    error?: string;
}>;
/**
 * 接受冲突解决方案
 */
export declare function acceptConflict(repoPath: string, file: string, strategy: 'ours' | 'theirs'): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 获取交互式 rebase 的提交列表
 */
export declare function interactiveRebaseList(repoPath: string, commitHash: string): Promise<{
    success: boolean;
    data?: string;
    error?: string;
}>;
/**
 * 执行交互式 rebase
 */
export declare function interactiveRebase(repoPath: string, commitHash: string, actions: Array<{
    hash: string;
    action: string;
}>): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 获取远程仓库列表
 */
export declare function remotes(repoPath: string): Promise<{
    success: boolean;
    data?: Array<{
        name: string;
        url: string;
        type: 'fetch' | 'push';
    }>;
    error?: string;
}>;
/**
 * 添加远程仓库
 */
export declare function addRemote(repoPath: string, name: string, url: string): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 删除远程仓库
 */
export declare function removeRemote(repoPath: string, name: string): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 设置远程仓库 URL
 */
export declare function setRemoteUrl(repoPath: string, name: string, url: string): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 添加 .gitignore 模式
 */
export declare function addGitignore(repoPath: string, pattern: string): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 获取 .gitignore 内容
 */
export declare function getGitignore(repoPath: string): Promise<{
    success: boolean;
    data?: string;
    error?: string;
}>;
/**
 * 获取子模块状态
 */
export declare function submodules(repoPath: string): Promise<{
    success: boolean;
    data?: Array<{
        hash: string;
        path: string;
        branch: string;
        description: string;
    }>;
    error?: string;
}>;
/**
 * 撤销最后一次提交（保留更改）
 */
export declare function undoLastCommit(repoPath: string): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 推送标签
 */
export declare function pushTags(repoPath: string, remote?: string): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 删除远程分支
 */
export declare function deleteRemoteBranch(repoPath: string, remote: string, branchName: string): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 清理未跟踪的文件
 */
export declare function clean(repoPath: string, options?: {
    dryRun?: boolean;
    directories?: boolean;
}): Promise<{
    success: boolean;
    data?: string;
    error?: string;
}>;
/**
 * 获取指定版本的文件内容
 */
export declare function fileAtRevision(repoPath: string, filePath: string, revision: string): Promise<{
    success: boolean;
    data?: string;
    error?: string;
}>;
/**
 * 比较文件在两个版本间的差异
 */
export declare function diffFileRevision(repoPath: string, filePath: string, rev1: string, rev2: string): Promise<{
    success: boolean;
    data?: string;
    error?: string;
}>;
/**
 * 比较两个提交的差异
 */
export declare function compareCommits(repoPath: string, commit1: string, commit2: string, filePath?: string): Promise<{
    success: boolean;
    data?: string;
    error?: string;
}>;
/**
 * 创建补丁
 */
export declare function createPatch(repoPath: string, commitRange: string, filePaths?: string[]): Promise<{
    success: boolean;
    data?: string;
    error?: string;
}>;
/**
 * 应用补丁
 */
export declare function applyPatch(repoPath: string, patchContent: string, options?: {
    check?: boolean;
}): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 批量 cherry-pick
 */
export declare function cherryPickMultiple(repoPath: string, hashes: string[], options?: {
    noCommit?: boolean;
}): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 从标签创建分支
 */
export declare function branchFromTag(repoPath: string, branchName: string, tagName: string): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 列出子模块路径
 */
export declare function submoduleList(repoPath: string): Promise<{
    success: boolean;
    data?: string[];
    error?: string;
}>;
/**
 * 初始化子模块
 */
export declare function submoduleInit(repoPath: string, recursive?: boolean): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 获取提交数量
 */
export declare function commitCount(repoPath: string, branch?: string): Promise<{
    success: boolean;
    data?: number;
    error?: string;
}>;
/**
 * 开始 rebase
 */
export declare function gitRebase(repoPath: string, targetBranch: string, options?: {
    onto?: string;
    upstream?: string;
    interactive?: boolean;
}): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 中止 rebase
 */
export declare function gitRebaseAbort(repoPath: string): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 继续 rebase
 */
export declare function gitRebaseContinue(repoPath: string): Promise<{
    success: boolean;
    error?: string;
}>;
/**
 * 获取文件 blame 信息
 */
export declare function gitFileBlame(repoPath: string, filePath: string): Promise<{
    success: boolean;
    data?: string;
    error?: string;
}>;
