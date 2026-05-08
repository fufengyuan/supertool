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
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.getRepoStatus = getRepoStatus;
exports.getRepoLog = getRepoLog;
exports.getRepoBranches = getRepoBranches;
exports.getCurrentBranch = getCurrentBranch;
exports.getRepoDiff = getRepoDiff;
exports.getCommitDiff = getCommitDiff;
exports.commit = commit;
exports.add = add;
exports.reset = reset;
exports.checkout = checkout;
exports.createBranch = createBranch;
exports.deleteBranch = deleteBranch;
exports.merge = merge;
exports.pull = pull;
exports.push = push;
exports.discardChanges = discardChanges;
exports.isValidGitRepo = isValidGitRepo;
exports.getRemoteUrl = getRemoteUrl;
exports.gitStashSave = gitStashSave;
exports.gitStashList = gitStashList;
exports.gitStashApply = gitStashApply;
exports.gitStashDrop = gitStashDrop;
exports.gitStashPop = gitStashPop;
exports.gitStashShow = gitStashShow;
exports.gitCherryPick = gitCherryPick;
exports.gitRevert = gitRevert;
exports.gitListTags = gitListTags;
exports.gitCreateTag = gitCreateTag;
exports.gitDeleteTag = gitDeleteTag;
exports.gitFileHistory = gitFileHistory;
exports.gitCompareBranches = gitCompareBranches;
exports.changedFiles = changedFiles;
exports.getRepoLogGraph = getRepoLogGraph;
exports.execGit = execGit;
exports.amendCommit = amendCommit;
exports.resetToCommit = resetToCommit;
exports.renameBranch = renameBranch;
exports.fetchRepo = fetchRepo;
exports.forcePush = forcePush;
exports.unpushedCommits = unpushedCommits;
exports.incomingCommits = incomingCommits;
exports.checkoutRemoteBranch = checkoutRemoteBranch;
exports.conflictFiles = conflictFiles;
exports.acceptConflict = acceptConflict;
exports.interactiveRebaseList = interactiveRebaseList;
exports.interactiveRebase = interactiveRebase;
exports.remotes = remotes;
exports.addRemote = addRemote;
exports.removeRemote = removeRemote;
exports.setRemoteUrl = setRemoteUrl;
exports.addGitignore = addGitignore;
exports.getGitignore = getGitignore;
exports.submodules = submodules;
exports.undoLastCommit = undoLastCommit;
exports.pushTags = pushTags;
exports.deleteRemoteBranch = deleteRemoteBranch;
exports.clean = clean;
exports.fileAtRevision = fileAtRevision;
exports.diffFileRevision = diffFileRevision;
exports.compareCommits = compareCommits;
exports.createPatch = createPatch;
exports.applyPatch = applyPatch;
exports.cherryPickMultiple = cherryPickMultiple;
exports.branchFromTag = branchFromTag;
exports.submoduleList = submoduleList;
exports.submoduleInit = submoduleInit;
exports.commitCount = commitCount;
exports.gitRebase = gitRebase;
exports.gitRebaseAbort = gitRebaseAbort;
exports.gitRebaseContinue = gitRebaseContinue;
exports.gitFileBlame = gitFileBlame;
const simple_git_1 = __importDefault(require("simple-git"));
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
// ============ 类型定义 ============
/** Safely extract error message from unknown error */
function getErrorMessage(error) {
    if (error instanceof Error)
        return error.message;
    if (typeof error === 'string')
        return error;
    return String(error);
}
// ============ 仓库信息查询 ============
/**
 * 获取工作区状态
 */
async function getRepoStatus(repoPath) {
    const git = (0, simple_git_1.default)(repoPath);
    const status = await git.status();
    return {
        modified: status.modified,
        added: status.staged,
        deleted: status.deleted,
        untracked: status.not_added,
        conflicted: status.conflicted,
        staged: status.staged,
        currentBranch: status.current || '',
        ahead: status.ahead,
        behind: status.behind,
    };
}
/**
 * 获取提交历史
 */
async function getRepoLog(repoPath, options) {
    const git = (0, simple_git_1.default)(repoPath);
    const logOptions = {};
    if (options?.limit)
        logOptions['--max-count'] = options.limit;
    if (options?.branch)
        logOptions[options.branch] = null;
    // Support multiple authors
    if (options?.authors && options.authors.length > 0) {
        logOptions['--author'] = options.authors.join('|');
    }
    else if (options?.author) {
        logOptions['--author'] = options.author;
    }
    // Support date range filtering
    if (options?.dateFrom)
        logOptions['--since'] = options.dateFrom;
    if (options?.dateTo)
        logOptions['--until'] = options.dateTo;
    const log = await git.log(logOptions);
    return log.all.map((entry) => ({
        hash: entry.hash,
        date: entry.date,
        author: entry.author_name,
        message: entry.message,
        refs: entry.refs || '',
    }));
}
/**
 * 获取所有分支（本地 + 远程）
 */
async function getRepoBranches(repoPath) {
    const git = (0, simple_git_1.default)(repoPath);
    const branches = await git.branch(['--all']);
    const result = [];
    for (const [name, branch] of Object.entries(branches.branches)) {
        result.push({
            name: branch.name || name,
            current: branch.current,
            linked: branch.linkedWorkTree,
            commit: branch.commit,
            label: branch.label,
        });
    }
    return result;
}
/**
 * 获取当前分支
 */
async function getCurrentBranch(repoPath) {
    const git = (0, simple_git_1.default)(repoPath);
    const branch = await git.branch();
    return branch.current;
}
/**
 * 获取文件差异
 */
async function getRepoDiff(repoPath, file) {
    const git = (0, simple_git_1.default)(repoPath);
    if (file) {
        return git.diff(['--', file]);
    }
    return git.diff();
}
/**
 * 获取指定提交的差异 (git show <hash>)
 */
async function getCommitDiff(repoPath, commitHash) {
    const git = (0, simple_git_1.default)(repoPath);
    try {
        return await git.show([commitHash]);
    }
    catch {
        // 如果 show 失败，尝试 diff 父提交
        return git.diff([`${commitHash}^`, commitHash]);
    }
}
// ============ Git 操作 ============
/**
 * 提交更改
 */
async function commit(repoPath, message, files) {
    const git = (0, simple_git_1.default)(repoPath);
    if (files && files.length > 0) {
        await git.add(files);
    }
    const result = await git.commit(message);
    return {
        success: true,
        hash: result.commit || undefined,
    };
}
/**
 * 暂存文件
 */
async function add(repoPath, files) {
    const git = (0, simple_git_1.default)(repoPath);
    await git.add(files);
}
/**
 * 取消暂存
 */
async function reset(repoPath, file) {
    const git = (0, simple_git_1.default)(repoPath);
    if (file) {
        await git.reset(['HEAD', '--', file]);
    }
    else {
        await git.reset();
    }
}
/**
 * 切换分支
 */
async function checkout(repoPath, branch) {
    const git = (0, simple_git_1.default)(repoPath);
    await git.checkout(branch);
}
/**
 * 创建分支
 */
async function createBranch(repoPath, branchName, from) {
    const git = (0, simple_git_1.default)(repoPath);
    if (from) {
        await git.checkout(['-b', branchName, from]);
    }
    else {
        await git.checkoutLocalBranch(branchName);
    }
}
/**
 * 删除分支
 */
async function deleteBranch(repoPath, branchName, force = false) {
    const git = (0, simple_git_1.default)(repoPath);
    await git.deleteLocalBranch(branchName, force);
}
/**
 * 合并分支
 */
async function merge(repoPath, branch) {
    const git = (0, simple_git_1.default)(repoPath);
    await git.merge([branch]);
}
/**
 * 拉取
 */
async function pull(repoPath) {
    const git = (0, simple_git_1.default)(repoPath);
    await git.pull();
}
/**
 * 推送
 */
async function push(repoPath) {
    const git = (0, simple_git_1.default)(repoPath);
    await git.push();
}
/**
 * 丢弃更改
 */
async function discardChanges(repoPath, file) {
    const git = (0, simple_git_1.default)(repoPath);
    // 检查文件是否在暂存区或已修改
    const status = await git.status();
    if (status.staged.includes(file) || status.modified.includes(file)) {
        // 如果已暂存，先取消暂存
        if (status.staged.includes(file)) {
            await git.reset(['HEAD', '--', file]);
        }
        // 恢复文件到 HEAD 版本
        await git.checkout(['--', file]);
    }
    else if (status.not_added.includes(file)) {
        // 未跟踪的文件，直接删除
        const fullPath = path.join(repoPath, file);
        if (fs.existsSync(fullPath)) {
            fs.unlinkSync(fullPath);
        }
    }
}
/**
 * 检查目录是否为有效的 Git 仓库
 */
async function isValidGitRepo(repoPath) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        await git.revparse(['--git-dir']);
        return true;
    }
    catch {
        return false;
    }
}
/**
 * 获取远程 URL
 */
async function getRemoteUrl(repoPath, remoteName = 'origin') {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const remotes = await git.getRemotes(true);
        const remote = remotes.find(r => r.name === remoteName);
        if (remote && remote.refs) {
            return remote.refs.fetch || remote.refs.push || null;
        }
        return null;
    }
    catch {
        return null;
    }
}
// ============ Stash 操作 ============
/**
 * 保存 stash
 */
async function gitStashSave(repoPath, options) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const args = [];
        if (options?.includeUntracked)
            args.push('-u');
        if (options?.keepIndex)
            args.push('--keep-index');
        if (options?.message)
            args.push('-m', options.message);
        const result = await git.stash(args);
        return { success: true, data: result || undefined };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 列出 stash
 */
async function gitStashList(repoPath) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const result = await git.stashList();
        const entries = result.all.map((entry) => entry.hash || '');
        return { success: true, data: entries };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 应用 stash
 */
async function gitStashApply(repoPath, stashRef) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const args = ['apply'];
        if (stashRef)
            args.push(stashRef);
        await git.stash(args);
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 删除 stash
 */
async function gitStashDrop(repoPath, stashRef) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const args = ['drop'];
        if (stashRef)
            args.push(stashRef);
        await git.stash(args);
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 弹出 stash (apply + drop)
 */
async function gitStashPop(repoPath, stashRef) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const args = ['pop'];
        if (stashRef)
            args.push(stashRef);
        await git.stash(args);
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 显示 stash 内容
 */
async function gitStashShow(repoPath, stashRef) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const ref = stashRef || 'stash@{0}';
        const diff = await git.show([ref]);
        return { success: true, data: diff };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
// ============ Cherry-pick 操作 ============
/**
 * Cherry-pick 提交
 */
async function gitCherryPick(repoPath, commitHash, options) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const args = ['cherry-pick'];
        if (options?.noCommit)
            args.push('--no-commit');
        args.push(commitHash);
        await git.raw(args);
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
// ============ Revert 操作 ============
/**
 * 撤销提交
 */
async function gitRevert(repoPath, commitHash, options) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const args = ['revert'];
        if (options?.noCommit)
            args.push('--no-commit');
        if (options?.noEdit)
            args.push('--no-edit');
        args.push(commitHash);
        await git.raw(args);
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
// ============ Tag 操作 ============
/**
 * 列出标签
 */
async function gitListTags(repoPath) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const tags = await git.tags();
        return { success: true, data: tags.all };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 创建标签
 */
async function gitCreateTag(repoPath, tagName, options) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const args = [];
        if (options?.force)
            args.push('--force');
        if (options?.annotated || options?.message)
            args.push('-a');
        if (options?.message)
            args.push('-m', options.message);
        args.push(tagName);
        if (options?.commit)
            args.push(options.commit);
        await git.addTag(args.join(' '));
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 删除标签
 */
async function gitDeleteTag(repoPath, tagName) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        await git.raw('tag', '-d', tagName);
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
// ============ File History ============
/**
 * 获取文件历史
 */
async function gitFileHistory(repoPath, filePath, options) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const logOptions = {
            file: filePath,
        };
        if (options?.limit)
            logOptions['--max-count'] = options.limit;
        if (options?.branch)
            logOptions[options.branch] = null;
        const log = await git.log(logOptions);
        const entries = log.all.map((entry) => ({
            hash: entry.hash,
            date: entry.date,
            author: entry.author_name,
            message: entry.message,
            refs: entry.refs || '',
        }));
        return { success: true, data: entries };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
// ============ Compare Branches ============
/**
 * 比较两个分支的差异
 */
async function gitCompareBranches(repoPath, targetBranch, sourceBranch) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const range = sourceBranch ? `${sourceBranch}..${targetBranch}` : targetBranch;
        const diff = await git.diff([range]);
        return { success: true, data: diff };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
// Get list of changed files between two commits
async function changedFiles(repoPath, commit1, commit2) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const result = await git.diff(['--name-status', commit1, commit2]);
        const files = result
            .trim()
            .split('\n')
            .filter(Boolean)
            .map(line => {
            const [status, ...fileParts] = line.split('\t');
            return { file: fileParts.join('\t'), status: status.trim() };
        });
        return { success: true, data: files };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
// ============ 更多 Git 操作 ============
/**
 * 获取带图形可视化的提交历史
 */
async function getRepoLogGraph(repoPath, options) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const args = ['--graph', '--oneline'];
        if (options?.limit)
            args.push(`--max-count=${options.limit}`);
        if (options?.branch)
            args.push(options.branch);
        const result = await git.raw('log', ...args);
        return { success: true, data: result };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 执行任意 git 命令
 */
async function execGit(repoPath, args) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const result = await git.raw(args);
        return { success: true, data: { stdout: result, stderr: '' } };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 修改最后一次提交
 */
async function amendCommit(repoPath, message) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        await git.raw('commit', '--amend', '-m', message);
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 重置到指定提交
 */
async function resetToCommit(repoPath, commitHash, mode = '--hard') {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        await git.raw('reset', mode, commitHash);
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 重命名分支
 */
async function renameBranch(repoPath, oldName, newName) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        await git.raw('branch', '-m', oldName, newName);
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 获取远程更新
 */
async function fetchRepo(repoPath, remote) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        if (remote) {
            await git.fetch(remote);
        }
        else {
            await git.fetch();
        }
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 强制推送
 */
async function forcePush(repoPath) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        await git.raw('push', '--force-with-lease');
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 获取未推送的提交
 */
async function unpushedCommits(repoPath) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const log = await git.log(['origin/HEAD..HEAD']);
        const entries = log.all.map((entry) => ({
            hash: entry.hash,
            message: entry.message,
            author: entry.author_name || '',
            date: entry.date,
        }));
        return { success: true, data: entries };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 获取待拉取的提交
 */
async function incomingCommits(repoPath) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const log = await git.log(['HEAD..origin/HEAD']);
        const entries = log.all.map((entry) => ({
            hash: entry.hash,
            message: entry.message,
            author: entry.author_name || '',
            date: entry.date,
        }));
        return { success: true, data: entries };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 检出远程分支
 */
async function checkoutRemoteBranch(repoPath, remote, branch) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        await git.checkout(['-b', branch, `${remote}/${branch}`]);
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 获取冲突文件列表
 */
async function conflictFiles(repoPath) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const result = await git.raw('diff', '--name-only', '--diff-filter=U');
        const files = result.trim().split('\n').filter(Boolean);
        return { success: true, data: files };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 接受冲突解决方案
 */
async function acceptConflict(repoPath, file, strategy) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        await git.raw('checkout', `--${strategy}`, '--', file);
        await git.add(file);
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 获取交互式 rebase 的提交列表
 */
async function interactiveRebaseList(repoPath, commitHash) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const result = await git.raw('log', '--oneline', `${commitHash}..HEAD`);
        return { success: true, data: result };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 执行交互式 rebase
 */
async function interactiveRebase(repoPath, commitHash, actions) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        // Build the rebase todo script
        const todoLines = actions.map((a) => `${a.action} ${a.hash}`).join('\n');
        // Use GIT_SEQUENCE_EDITOR to automate the interactive rebase
        const editorScript = `echo '${todoLines.replace(/'/g, "'\\''")}' > "$1"`;
        await git.env({ GIT_SEQUENCE_EDITOR: editorScript }).raw('rebase', '-i', commitHash);
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 获取远程仓库列表
 */
async function remotes(repoPath) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const raw = await git.raw('remote', '-v');
        const lines = raw.trim().split('\n').filter(Boolean);
        const result = lines.map((line) => {
            const parts = line.trim().split(/\s+/);
            return {
                name: parts[0],
                url: parts[1],
                type: parts[2].replace(/[()]/g, ''),
            };
        });
        return { success: true, data: result };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 添加远程仓库
 */
async function addRemote(repoPath, name, url) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        await git.remote(['add', name, url]);
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 删除远程仓库
 */
async function removeRemote(repoPath, name) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        await git.remote(['remove', name]);
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 设置远程仓库 URL
 */
async function setRemoteUrl(repoPath, name, url) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        await git.remote(['set-url', name, url]);
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 添加 .gitignore 模式
 */
async function addGitignore(repoPath, pattern) {
    try {
        const gitignorePath = path.join(repoPath, '.gitignore');
        fs.appendFileSync(gitignorePath, pattern.endsWith('\n') ? pattern : pattern + '\n');
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 获取 .gitignore 内容
 */
async function getGitignore(repoPath) {
    try {
        const gitignorePath = path.join(repoPath, '.gitignore');
        if (!fs.existsSync(gitignorePath)) {
            return { success: true, data: '' };
        }
        const content = fs.readFileSync(gitignorePath, 'utf-8');
        return { success: true, data: content };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 获取子模块状态
 */
async function submodules(repoPath) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const raw = await git.raw('submodule', 'status');
        const lines = raw.trim().split('\n').filter(Boolean);
        if (lines.length === 0)
            return { success: true, data: [] };
        const result = lines.map((line) => {
            const trimmed = line.trim();
            const hash = trimmed.substring(1, 41);
            const rest = trimmed.substring(41).trim();
            const parts = rest.split(/\s+/);
            return {
                hash,
                path: parts[0] || '',
                branch: parts[1] ? parts[1].replace(/[()]/g, '') : '',
                description: parts.slice(2).join(' ') || '',
            };
        });
        return { success: true, data: result };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 撤销最后一次提交（保留更改）
 */
async function undoLastCommit(repoPath) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        await git.raw('reset', '--soft', 'HEAD~1');
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 推送标签
 */
async function pushTags(repoPath, remote) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        await git.raw('push', remote || 'origin', '--tags');
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 删除远程分支
 */
async function deleteRemoteBranch(repoPath, remote, branchName) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        await git.raw('push', remote, '--delete', branchName);
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 清理未跟踪的文件
 */
async function clean(repoPath, options) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const args = ['clean'];
        args.push('-f');
        if (options?.directories)
            args.push('-d');
        if (options?.dryRun)
            args.push('-n');
        const result = await git.raw(...args);
        return { success: true, data: result };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 获取指定版本的文件内容
 */
async function fileAtRevision(repoPath, filePath, revision) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const content = await git.raw('show', `${revision}:${filePath}`);
        return { success: true, data: content };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 比较文件在两个版本间的差异
 */
async function diffFileRevision(repoPath, filePath, rev1, rev2) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const diff = await git.diff([rev1, rev2, '--', filePath]);
        return { success: true, data: diff };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 比较两个提交的差异
 */
async function compareCommits(repoPath, commit1, commit2, filePath) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const args = [commit1, commit2];
        if (filePath)
            args.push('--', filePath);
        const diff = await git.diff(args);
        return { success: true, data: diff };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 创建补丁
 */
async function createPatch(repoPath, commitRange, filePaths) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        let result;
        if (filePaths && filePaths.length > 0) {
            result = await git.diff([commitRange, '--', ...filePaths]);
        }
        else {
            result = await git.diff([commitRange]);
        }
        return { success: true, data: result };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 应用补丁
 */
async function applyPatch(repoPath, patchContent, options) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const args = ['apply'];
        if (options?.check)
            args.push('--check');
        // Write patch to temp file and apply
        const tempPath = path.join(repoPath, '.temp_patch.diff');
        fs.writeFileSync(tempPath, patchContent);
        try {
            await git.raw(...args, tempPath);
        }
        finally {
            if (fs.existsSync(tempPath)) {
                fs.unlinkSync(tempPath);
            }
        }
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 批量 cherry-pick
 */
async function cherryPickMultiple(repoPath, hashes, options) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const args = ['cherry-pick'];
        if (options?.noCommit)
            args.push('--no-commit');
        args.push(...hashes);
        await git.raw(args);
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 从标签创建分支
 */
async function branchFromTag(repoPath, branchName, tagName) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        await git.checkout(['-b', branchName, tagName]);
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 列出子模块路径
 */
async function submoduleList(repoPath) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const raw = await git.raw('submodule', 'foreach', '--quiet', 'echo $path');
        const paths = raw.trim().split('\n').filter(Boolean);
        return { success: true, data: paths };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 初始化子模块
 */
async function submoduleInit(repoPath, recursive = false) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const args = ['submodule', 'update', '--init'];
        if (recursive)
            args.push('--recursive');
        await git.raw(args);
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 获取提交数量
 */
async function commitCount(repoPath, branch) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const result = await git.raw('rev-list', '--count', branch || 'HEAD');
        return { success: true, data: parseInt(result.trim(), 10) };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
// ============ Rebase 操作 ============
/**
 * 开始 rebase
 */
async function gitRebase(repoPath, targetBranch, options) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const args = [];
        if (options?.interactive)
            args.push('-i');
        if (options?.onto) {
            // git rebase --onto <newbase> <upstream> <branch>
            const upstream = options.upstream || targetBranch;
            args.push('--onto', options.onto, upstream, targetBranch);
        }
        else {
            args.push(targetBranch);
        }
        await git.rebase(args);
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 中止 rebase
 */
async function gitRebaseAbort(repoPath) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        await git.rebase(['--abort']);
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
/**
 * 继续 rebase
 */
async function gitRebaseContinue(repoPath) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        await git.rebase(['--continue']);
        return { success: true };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
// ============ Blame 操作 ============
/**
 * 获取文件 blame 信息
 */
async function gitFileBlame(repoPath, filePath) {
    try {
        const git = (0, simple_git_1.default)(repoPath);
        const blame = await git.raw('blame', '--', filePath);
        return { success: true, data: blame };
    }
    catch (error) {
        return { success: false, error: getErrorMessage(error) };
    }
}
//# sourceMappingURL=git-service.js.map