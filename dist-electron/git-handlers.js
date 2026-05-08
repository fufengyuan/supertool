"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.registerGitHandlers = registerGitHandlers;
const async_exec_1 = require("./async-exec");
const electron_1 = require("electron");
const gitService = __importStar(require("./services/git-service"));
/** Safely extract error message from unknown error */
function getErrorMessage(error) {
    if (error instanceof Error)
        return error.message;
    if (typeof error === 'string')
        return error;
    return String(error);
}
function registerGitHandlers(db) {
    // ============ Git Repos Management ============
    electron_1.ipcMain.handle('git:repos:get-all', () => {
        try {
            return { success: true, data: db.getAllGitRepos() };
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:repos:add', (_event, repo) => {
        try {
            if (!repo.path)
                return { success: false, error: '仓库路径不能为空' };
            if (!repo.name)
                return { success: false, error: '仓库名称不能为空' };
            const result = db.addGitRepo(repo);
            return { success: true, data: result };
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:repos:update', (_event, id, updates) => {
        try {
            const result = db.updateGitRepo(id, updates);
            if (!result)
                return { success: false, error: '仓库不存在' };
            return { success: true, data: result };
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:repos:delete', (_event, id) => {
        try {
            const result = db.deleteGitRepo(id);
            if (!result)
                return { success: false, error: '仓库不存在' };
            return { success: true };
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    // ============ Git Operations ============
    electron_1.ipcMain.handle('git:status', async (_event, repoPath) => {
        try {
            const status = await gitService.getRepoStatus(repoPath);
            return { success: true, data: status };
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:log', async (_event, repoPath, options) => {
        try {
            const log = await gitService.getRepoLog(repoPath, options);
            return { success: true, data: log };
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:log-graph', async (_event, repoPath, limit) => {
        try {
            const { getRepoLogGraph } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await getRepoLogGraph(repoPath, { limit });
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:exec', async (_event, repoPath, args) => {
        try {
            const { execGit } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await execGit(repoPath, args);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:branches', async (_event, repoPath) => {
        try {
            const branches = await gitService.getRepoBranches(repoPath);
            return { success: true, data: branches };
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:current-branch', async (_event, repoPath) => {
        try {
            const branch = await gitService.getCurrentBranch(repoPath);
            return { success: true, data: branch };
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:diff', async (_event, repoPath, file) => {
        try {
            const diff = await gitService.getRepoDiff(repoPath, file);
            return { success: true, data: diff };
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:commit-diff', async (_event, repoPath, commitHash) => {
        try {
            const diff = await gitService.getCommitDiff(repoPath, commitHash);
            return { success: true, data: diff };
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:commit', async (_event, repoPath, message, files) => {
        try {
            if (!message?.trim())
                return { success: false, error: '提交信息不能为空' };
            const result = await gitService.commit(repoPath, message, files);
            return { success: true, data: result };
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:add', async (_event, repoPath, files) => {
        try {
            if (!files || (Array.isArray(files) && files.length === 0))
                return { success: false, error: '文件列表不能为空' };
            await gitService.add(repoPath, files);
            return { success: true };
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:reset', async (_event, repoPath, file) => {
        try {
            await gitService.reset(repoPath, file);
            return { success: true };
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:checkout', async (_event, repoPath, branch) => {
        try {
            if (!branch?.trim())
                return { success: false, error: '分支名不能为空' };
            await gitService.checkout(repoPath, branch);
            return { success: true };
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:create-branch', async (_event, repoPath, branchName, from) => {
        try {
            if (!branchName?.trim())
                return { success: false, error: '分支名不能为空' };
            await gitService.createBranch(repoPath, branchName, from);
            return { success: true };
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:delete-branch', async (_event, repoPath, branchName, force) => {
        try {
            if (!branchName?.trim())
                return { success: false, error: '分支名不能为空' };
            await gitService.deleteBranch(repoPath, branchName, force);
            return { success: true };
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:merge', async (_event, repoPath, branch) => {
        try {
            if (!branch?.trim())
                return { success: false, error: '分支名不能为空' };
            await gitService.merge(repoPath, branch);
            return { success: true };
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:pull', async (_event, repoPath) => {
        try {
            await gitService.pull(repoPath);
            return { success: true };
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:push', async (_event, repoPath) => {
        try {
            await gitService.push(repoPath);
            return { success: true };
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:discard-changes', async (_event, repoPath, file) => {
        try {
            if (!file?.trim())
                return { success: false, error: '文件名不能为空' };
            await gitService.discardChanges(repoPath, file);
            return { success: true };
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    // ============ Git Stash ============
    electron_1.ipcMain.handle('git:stash-save', async (_event, repoPath, options) => {
        try {
            return await gitService.gitStashSave(repoPath, options);
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:stash-list', async (_event, repoPath) => {
        try {
            return await gitService.gitStashList(repoPath);
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:stash-apply', async (_event, repoPath, stashRef) => {
        try {
            return await gitService.gitStashApply(repoPath, stashRef);
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:stash-drop', async (_event, repoPath, stashRef) => {
        try {
            return await gitService.gitStashDrop(repoPath, stashRef);
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:stash-pop', async (_event, repoPath, stashRef) => {
        try {
            return await gitService.gitStashPop(repoPath, stashRef);
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:stash-show', async (_event, repoPath, stashRef) => {
        try {
            return await gitService.gitStashShow(repoPath, stashRef);
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    // ============ Git Cherry-pick ============
    electron_1.ipcMain.handle('git:cherry-pick', async (_event, repoPath, commitHash, options) => {
        try {
            return await gitService.gitCherryPick(repoPath, commitHash, options);
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:revert', async (_event, repoPath, commitHash, options) => {
        try {
            return await gitService.gitRevert(repoPath, commitHash, options);
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    // ============ Git Tag ============
    electron_1.ipcMain.handle('git:tag-list', async (_event, repoPath) => {
        try {
            return await gitService.gitListTags(repoPath);
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:tag-create', async (_event, repoPath, tagName, options) => {
        try {
            return await gitService.gitCreateTag(repoPath, tagName, options);
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:tag-delete', async (_event, repoPath, tagName) => {
        try {
            return await gitService.gitDeleteTag(repoPath, tagName);
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    // ============ Git File History ============
    electron_1.ipcMain.handle('git:file-history', async (_event, repoPath, filePath, options) => {
        try {
            return await gitService.gitFileHistory(repoPath, filePath, options);
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:compare-branches', async (_event, repoPath, targetBranch, sourceBranch) => {
        try {
            return await gitService.gitCompareBranches(repoPath, targetBranch, sourceBranch);
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    // ============ Git Rebase ============
    electron_1.ipcMain.handle('git:rebase', async (_event, repoPath, targetBranch, options) => {
        try {
            return await gitService.gitRebase(repoPath, targetBranch, options);
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:rebase-abort', async (_event, repoPath) => {
        try {
            return await gitService.gitRebaseAbort(repoPath);
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:rebase-continue', async (_event, repoPath) => {
        try {
            return await gitService.gitRebaseContinue(repoPath);
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    electron_1.ipcMain.handle('git:file-blame', async (_event, repoPath, filePath) => {
        try {
            return await gitService.gitFileBlame(repoPath, filePath);
        }
        catch (error) {
            return { success: false, error: getErrorMessage(error) };
        }
    });
    // ============ Git Advanced Operations ============
    electron_1.ipcMain.handle('git:amend-commit', async (_event, repoPath, message) => {
        try {
            const { amendCommit } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await amendCommit(repoPath, message || '');
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:reset-to-commit', async (_event, repoPath, commitHash, mode) => {
        try {
            const { resetToCommit } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await resetToCommit(repoPath, commitHash, mode);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:rename-branch', async (_event, repoPath, oldName, newName) => {
        try {
            const { renameBranch } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await renameBranch(repoPath, oldName, newName);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:fetch', async (_event, repoPath, remote) => {
        try {
            const { fetchRepo } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await fetchRepo(repoPath, remote);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:force-push', async (_event, repoPath) => {
        try {
            const { forcePush } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await forcePush(repoPath);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:unpushed-commits', async (_event, repoPath) => {
        try {
            const { unpushedCommits } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await unpushedCommits(repoPath);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:incoming-commits', async (_event, repoPath) => {
        try {
            const { incomingCommits } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await incomingCommits(repoPath);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:checkout-remote-branch', async (_event, repoPath, remote, branch) => {
        try {
            const { checkoutRemoteBranch } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await checkoutRemoteBranch(repoPath, remote, branch);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:conflict-files', async (_event, repoPath) => {
        try {
            const { conflictFiles } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await conflictFiles(repoPath);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:accept-conflict', async (_event, repoPath, file, strategy) => {
        try {
            const { acceptConflict } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await acceptConflict(repoPath, file, strategy);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:interactive-rebase-list', async (_event, repoPath, commitHash) => {
        try {
            const { interactiveRebaseList } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await interactiveRebaseList(repoPath, commitHash);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:interactive-rebase', async (_event, repoPath, commitHash, actions) => {
        try {
            const { interactiveRebase } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await interactiveRebase(repoPath, commitHash, actions);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:remotes', async (_event, repoPath) => {
        try {
            const { remotes } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await remotes(repoPath);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:add-remote', async (_event, repoPath, name, url) => {
        try {
            const { addRemote } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await addRemote(repoPath, name, url);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:remove-remote', async (_event, repoPath, name) => {
        try {
            const { removeRemote } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await removeRemote(repoPath, name);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:set-remote-url', async (_event, repoPath, name, url) => {
        try {
            const { setRemoteUrl } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await setRemoteUrl(repoPath, name, url);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:add-gitignore', async (_event, repoPath, pattern) => {
        try {
            const { addGitignore } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await addGitignore(repoPath, pattern);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:get-gitignore', async (_event, repoPath) => {
        try {
            const { getGitignore } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await getGitignore(repoPath);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:submodules', async (_event, repoPath) => {
        try {
            const { submodules } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await submodules(repoPath);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:undo-last-commit', async (_event, repoPath) => {
        try {
            const { undoLastCommit } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await undoLastCommit(repoPath);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:push-tags', async (_event, repoPath, remote) => {
        try {
            const { pushTags } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await pushTags(repoPath, remote);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:delete-remote-branch', async (_event, repoPath, remote, branchName) => {
        try {
            const { deleteRemoteBranch } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await deleteRemoteBranch(repoPath, remote, branchName);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:clean', async (_event, repoPath, options) => {
        try {
            const { clean } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await clean(repoPath, options);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:file-at-revision', async (_event, repoPath, filePath, revision) => {
        try {
            const { fileAtRevision } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await fileAtRevision(repoPath, filePath, revision);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:diff-file-revision', async (_event, repoPath, filePath, rev1, rev2) => {
        try {
            const { diffFileRevision } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await diffFileRevision(repoPath, filePath, rev1, rev2);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:compare-commits', async (_event, repoPath, commit1, commit2, filePath) => {
        try {
            const { compareCommits } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await compareCommits(repoPath, commit1, commit2, filePath);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:create-patch', async (_event, repoPath, commitRange, filePaths) => {
        try {
            const { createPatch } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await createPatch(repoPath, commitRange, filePaths);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:apply-patch', async (_event, repoPath, patchContent, options) => {
        try {
            const { applyPatch } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await applyPatch(repoPath, patchContent, options);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:cherry-pick-multiple', async (_event, repoPath, hashes, options) => {
        try {
            const { cherryPickMultiple } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await cherryPickMultiple(repoPath, hashes, options);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:branch-from-tag', async (_event, repoPath, branchName, tagName) => {
        try {
            const { branchFromTag } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await branchFromTag(repoPath, branchName, tagName);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:submodule-list', async (_event, repoPath) => {
        try {
            const { submoduleList } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await submoduleList(repoPath);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:submodule-init', async (_event, repoPath, recursive) => {
        try {
            const { submoduleInit } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await submoduleInit(repoPath, recursive);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:commit-count', async (_event, repoPath, branch) => {
        try {
            const { commitCount } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await commitCount(repoPath, branch);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:changed-files', async (_event, repoPath, commit1, commit2) => {
        try {
            const { changedFiles } = await Promise.resolve().then(() => __importStar(require('./services/git-service')));
            return await changedFiles(repoPath, commit1, commit2);
        }
        catch (e) {
            return { success: false, error: getErrorMessage(e) };
        }
    });
    // Git Repo List helpers
    electron_1.ipcMain.handle('git:validate-repo-path', async (_event, repoPath) => {
        try {
            await (0, async_exec_1.runCommand)('git rev-parse --git-dir', { cwd: repoPath });
            const nameResult = await (0, async_exec_1.runCommand)('basename "$(git rev-parse --show-toplevel)"', { cwd: repoPath });
            return { valid: true, name: nameResult.stdout.trim(), error: '' };
        }
        catch (e) {
            return { valid: false, name: '', error: getErrorMessage(e) };
        }
    });
    electron_1.ipcMain.handle('git:scan-local-repos', async () => {
        // Return empty array — user can add repos manually or use 'Scan' in GitRepoList
        return [];
    });
    electron_1.ipcMain.handle('git:get-commits', async (_event, _gitUrl, _branch, _sinceDate) => {
        return [];
    });
    electron_1.ipcMain.handle('git:get-branches', async (_event, _gitUrl) => {
        return [];
    });
} // end registerGitHandlers
//# sourceMappingURL=git-handlers.js.map