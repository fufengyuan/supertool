import { ref, computed, watch } from 'vue'
import { getTauriAPI } from '../utils/tauri-api'
import { useToast } from './useToast'

export interface GitRepo {
  id: string
  name: string
  path: string
}

export function useGitManager(repo: GitRepo | null, _onClose: () => void) {
  const toast = useToast()
  const api = getTauriAPI()

  // 核心状态
  const loading = ref(false)
  const currentBranch = ref('')
  const statusData = ref<{ files: any[] } | null>(null)
  const selectedFiles = ref(new Set<string>())
  const collapsedGroups = ref(new Set<string>())
  const commitMessage = ref('')
  const committing = ref(false)
  const totalChanges = ref(0)

  // Log 状态
  const logData = ref<any[]>([])
  const logSearch = ref('')
  const logBranchFilter = ref('')
  const selectedCommit = ref<any>(null)
  const commitDiff = ref<any>(null)
  const loadingDiff = ref(false)
  const logCount = ref(0)
  const hasMoreLog = ref(false)
  const filteredLog = computed(() => {
    if (!logSearch.value) return logData.value
    return logData.value.filter(c => 
      c.message?.toLowerCase().includes(logSearch.value.toLowerCase()) ||
      c.hash?.includes(logSearch.value)
    )
  })

  // Branch 状态
  const branchesData = ref<any[]>([])
  const localBranches = ref<any[]>([])
  const remoteBranches = ref<any[]>([])
  const showBranchesPopup = ref(false)
  const showCreateBranch = ref(false)
  const newBranchName = ref('')
  const newBranchFrom = ref('')
  const mergeTarget = ref('')
  const merging = ref(false)

  // Stash 状态
  const showStashPanel = ref(false)
  const stashList = ref<any[]>([])
  const selectedStash = ref<any>(null)
  const showStashSaveDialog = ref(false)
  const stashSaveMessage = ref('')
  const stashIncludeUntracked = ref(false)

  // Tags 状态
  const showTagsDialog = ref(false)
  const tagsList = ref<any[]>([])
  const showCreateTagDialog = ref(false)
  const newTagName = ref('')
  const newTagMessage = ref('')

  // Push/Pull 状态
  const pushing = ref(false)
  const pulling = ref(false)
  const showPushDialog = ref(false)
  const showPullDialog = ref(false)
  const pushForce = ref(false)

  // Rebase 状态
  const showRebaseDialog = ref(false)
  const rebaseTarget = ref('')
  const rebasing = ref(false)

  // 其他对话框状态
  const cherryPickTarget = ref('')
  const cherryPicking = ref(false)
  const revertTarget = ref('')
  const reverting = ref(false)
  const showFileHistoryDialog = ref(false)
  const fileHistoryFile = ref('')
  const fileHistoryData = ref<any>(null)
  const showBlameDialog = ref(false)
  const blameFile = ref('')
  const blameData = ref<any>(null)
  const showAmendDialog = ref(false)
  const amendMessage = ref('')
  const amending = ref(false)
  const showResetDialog = ref(false)
  const resetTarget = ref('')
  const resetMode = ref('hard')
  const resetting = ref(false)
  const showRemotesDialog = ref(false)
  const remotesList = ref<any[]>([])
  const showSubmodulesDialog = ref(false)
  const submodulesList = ref<any[]>([])
  const smLoading = ref(false)

  // Context menu
  const contextMenu = ref(false)
  const logContextMenu = ref(false)

  // Repo path
  const repoPath = computed(() => repo?.path || '')

  // ============ 核心加载函数 ============

  async function loadStatus() {
    if (!repoPath.value) return
    loading.value = true
    try {
      const res = await api.gitStatus(repoPath.value)
      statusData.value = res as { files: any[] }
      totalChanges.value = (res as any).files?.length || 0
    } catch (e: any) {
      toast.error('加载状态失败: ' + e.message)
    } finally {
      loading.value = false
    }
  }

  async function loadCurrentBranch() {
    if (!repoPath.value) return
    try {
      const res = await api.gitCurrentBranch(repoPath.value)
      currentBranch.value = (res as any).branch || ''
    } catch (e: any) {
      console.error('加载当前分支失败:', e)
    }
  }

  async function loadBranches() {
    if (!repoPath.value) return
    try {
      const res = await api.gitBranches(repoPath.value)
      const branches = (res as any).branches || []
      branchesData.value = branches
      localBranches.value = branches.filter((b: any) => !b.name?.includes('remotes/'))
      remoteBranches.value = branches.filter((b: any) => b.name?.includes('remotes/'))
    } catch (e: any) {
      toast.error('加载分支失败: ' + e.message)
    }
  }

  async function loadLog(opts?: { limit?: number }) {
    if (!repoPath.value) return
    loading.value = true
    try {
      const limit = opts?.limit || 50
      const res = await api.gitLog(repoPath.value, limit)
      logData.value = (res as any).commits || []
      logCount.value = logData.value.length
      hasMoreLog.value = logData.value.length >= limit
    } catch (e: any) {
      toast.error('加载日志失败: ' + e.message)
    } finally {
      loading.value = false
    }
  }

  async function loadMoreLog() {
    if (!repoPath.value || !hasMoreLog.value) return
    try {
      const res = await api.gitLog(repoPath.value, logCount.value + 50)
      logData.value = (res as any).commits || []
      logCount.value = logData.value.length
    } catch (e: any) {
      toast.error('加载更多日志失败: ' + e.message)
    }
  }

  async function selectCommit(commit: any) {
    selectedCommit.value = commit
    await loadCommitDiff()
  }

  async function loadCommitDiff() {
    if (!repoPath.value || !selectedCommit.value) return
    loadingDiff.value = true
    try {
      const res = await api.getGitCommitDetail(repoPath.value, selectedCommit.value.hash)
      commitDiff.value = res
    } catch (e: any) {
      toast.error('加载提交详情失败: ' + e.message)
    } finally {
      loadingDiff.value = false
    }
  }

  async function refreshAll() {
    await Promise.all([
      loadStatus(),
      loadCurrentBranch(),
      loadBranches(),
      loadLog(),
    ])
  }

  // ============ 文件操作 ============

  function toggleGroup(group: string) {
    if (collapsedGroups.value.has(group)) {
      collapsedGroups.value.delete(group)
    } else {
      collapsedGroups.value.add(group)
    }
  }

  function toggleFileSelect(file: string) {
    if (selectedFiles.value.has(file)) {
      selectedFiles.value.delete(file)
    } else {
      selectedFiles.value.add(file)
    }
  }

  function selectAllFiles() {
    const files = statusData.value?.files || []
    files.forEach(f => selectedFiles.value.add(f.path))
  }

  async function doCommit(noVerify: boolean = false) {
    if (!repoPath.value || !commitMessage.value.trim()) {
      toast.error('请输入提交信息')
      return
    }
    committing.value = true
    try {
      const filesToCommit = Array.from(selectedFiles.value)
      if (filesToCommit.length > 0) {
        await api.gitAdd(repoPath.value, filesToCommit)
      }
      await api.gitCommit(repoPath.value, commitMessage.value, filesToCommit.length > 0 ? filesToCommit : undefined)
      toast.success('提交成功')
      commitMessage.value = ''
      selectedFiles.value.clear()
      await refreshAll()
    } catch (e: any) {
      toast.error('提交失败: ' + e.message)
    } finally {
      committing.value = false
    }
  }

  async function doCommitAndPush() {
    await doCommit()
    await doPush()
  }

  // ============ 分支操作 ============

  async function checkoutBranch(branch: string) {
    if (!repoPath.value) return
    try {
      await api.gitCheckout(repoPath.value, branch)
      toast.success('切换分支成功')
      await refreshAll()
    } catch (e: any) {
      toast.error('切换分支失败: ' + e.message)
    }
  }

  async function doCreateBranch() {
    if (!repoPath.value || !newBranchName.value.trim()) {
      toast.error('请输入分支名')
      return
    }
    try {
      await api.gitCreateBranch(repoPath.value, newBranchName.value, newBranchFrom.value || undefined)
      toast.success('创建分支成功')
      showCreateBranch.value = false
      newBranchName.value = ''
      newBranchFrom.value = ''
      await loadBranches()
    } catch (e: any) {
      toast.error('创建分支失败: ' + e.message)
    }
  }

  async function doDeleteBranch(branchName: string, force: boolean = false) {
    if (!repoPath.value) return
    try {
      await api.gitDeleteBranch(repoPath.value, branchName, force)
      toast.success('删除分支成功')
      await loadBranches()
    } catch (e: any) {
      toast.error('删除分支失败: ' + e.message)
    }
  }

  async function doMerge() {
    if (!repoPath.value || !mergeTarget.value) return
    merging.value = true
    try {
      await api.gitMerge(repoPath.value, mergeTarget.value)
      toast.success('合并成功')
      showBranchesPopup.value = false
      await refreshAll()
    } catch (e: any) {
      toast.error('合并失败: ' + e.message)
    } finally {
      merging.value = false
    }
  }

  // ============ Push/Pull 操作 ============

  async function doPush(opts?: { force?: boolean }) {
    if (!repoPath.value) return
    pushing.value = true
    try {
      if (opts?.force) {
        await api.gitForcePush(repoPath.value)
      } else {
        await api.gitPush(repoPath.value)
      }
      toast.success('推送成功')
      await loadLog()
    } catch (e: any) {
      toast.error('推送失败: ' + e.message)
    } finally {
      pushing.value = false
    }
  }

  async function doPull(opts?: { rebase?: boolean }) {
    if (!repoPath.value) return
    pulling.value = true
    try {
      await api.gitPull(repoPath.value)
      toast.success('拉取成功')
      await refreshAll()
    } catch (e: any) {
      toast.error('拉取失败: ' + e.message)
    } finally {
      pulling.value = false
    }
  }

  async function doFetch() {
    if (!repoPath.value) return
    try {
      await api.gitFetch(repoPath.value)
      toast.success('获取远程信息成功')
      await loadBranches()
    } catch (e: any) {
      toast.error('获取失败: ' + e.message)
    }
  }

  async function doForcePush() {
    await doPush({ force: true })
  }

  // ============ Stash 操作 ============

  async function loadStashList() {
    if (!repoPath.value) return
    try {
      const res = await api.gitStashList(repoPath.value)
      stashList.value = (res as any).stashes || []
    } catch (e: any) {
      toast.error('加载stash失败: ' + e.message)
    }
  }

  function openStashSave() {
    showStashSaveDialog.value = true
    stashSaveMessage.value = ''
    stashIncludeUntracked.value = false
  }

  async function doStashSave() {
    if (!repoPath.value) return
    try {
      await api.gitStashSave(repoPath.value, stashSaveMessage.value || undefined, stashIncludeUntracked.value, false)
      toast.success('Stash保存成功')
      showStashSaveDialog.value = false
      await loadStashList()
      await loadStatus()
    } catch (e: any) {
      toast.error('Stash失败: ' + e.message)
    }
  }

  async function doStashPop(stashRef?: string) {
    if (!repoPath.value) return
    try {
      await api.gitStashPop(repoPath.value, stashRef)
      toast.success('Stash弹出成功')
      await loadStashList()
      await loadStatus()
    } catch (e: any) {
      toast.error('Stash弹出失败: ' + e.message)
    }
  }

  async function doStashDrop(stashRef?: string) {
    if (!repoPath.value) return
    try {
      await api.gitStashDrop(repoPath.value, stashRef)
      toast.success('Stash删除成功')
      await loadStashList()
    } catch (e: any) {
      toast.error('Stash删除失败: ' + e.message)
    }
  }

  // ============ Tag 操作 ============

  async function loadTags() {
    if (!repoPath.value) return
    try {
      const res = await api.gitListTags(repoPath.value)
      tagsList.value = (res as any).tags || []
    } catch (e: any) {
      toast.error('加载标签失败: ' + e.message)
    }
  }

  async function doCreateTag() {
    if (!repoPath.value || !newTagName.value.trim()) {
      toast.error('请输入标签名')
      return
    }
    try {
      await api.gitCreateTag(repoPath.value, newTagName.value, newTagMessage.value || undefined, false)
      toast.success('创建标签成功')
      showCreateTagDialog.value = false
      newTagName.value = ''
      newTagMessage.value = ''
      await loadTags()
    } catch (e: any) {
      toast.error('创建标签失败: ' + e.message)
    }
  }

  async function doDeleteTag(tagName: string) {
    if (!repoPath.value) return
    try {
      await api.gitDeleteTag(repoPath.value, tagName)
      toast.success('删除标签成功')
      await loadTags()
    } catch (e: any) {
      toast.error('删除标签失败: ' + e.message)
    }
  }

  // ============ 高级操作 ============

  async function doCherryPick() {
    if (!repoPath.value || !cherryPickTarget.value) return
    cherryPicking.value = true
    try {
      await api.gitCherryPick(repoPath.value, cherryPickTarget.value, false)
      toast.success('Cherry-pick成功')
      cherryPickTarget.value = ''
      await refreshAll()
    } catch (e: any) {
      toast.error('Cherry-pick失败: ' + e.message)
    } finally {
      cherryPicking.value = false
    }
  }

  async function doRevert() {
    if (!repoPath.value || !revertTarget.value) return
    reverting.value = true
    try {
      await api.gitRevert(repoPath.value, revertTarget.value, false)
      toast.success('Revert成功')
      revertTarget.value = ''
      await refreshAll()
    } catch (e: any) {
      toast.error('Revert失败: ' + e.message)
    } finally {
      reverting.value = false
    }
  }

  async function doRebase() {
    if (!repoPath.value || !rebaseTarget.value) return
    rebasing.value = true
    try {
      await api.gitRebase(repoPath.value, rebaseTarget.value)
      toast.success('Rebase成功')
      showRebaseDialog.value = false
      await refreshAll()
    } catch (e: any) {
      toast.error('Rebase失败: ' + e.message)
    } finally {
      rebasing.value = false
    }
  }

  async function doRebaseAbort() {
    if (!repoPath.value) return
    try {
      await api.gitRebaseAbort(repoPath.value)
      toast.success('Rebase中止')
      await refreshAll()
    } catch (e: any) {
      toast.error('中止失败: ' + e.message)
    }
  }

  async function doRebaseContinue() {
    if (!repoPath.value) return
    try {
      await api.gitRebaseContinue(repoPath.value)
      toast.success('Rebase继续')
      await refreshAll()
    } catch (e: any) {
      toast.error('继续失败: ' + e.message)
    }
  }

  async function doAmend() {
    if (!repoPath.value) return
    amending.value = true
    try {
      await api.gitAmendCommit(repoPath.value, amendMessage.value)
      toast.success('修改提交成功')
      showAmendDialog.value = false
      amendMessage.value = ''
      await loadLog()
    } catch (e: any) {
      toast.error('修改失败: ' + e.message)
    } finally {
      amending.value = false
    }
  }

  async function doReset() {
    if (!repoPath.value || !resetTarget.value) return
    resetting.value = true
    try {
      await api.gitResetToCommit(repoPath.value, resetTarget.value, resetMode.value)
      toast.success('重置成功')
      showResetDialog.value = false
      await refreshAll()
    } catch (e: any) {
      toast.error('重置失败: ' + e.message)
    } finally {
      resetting.value = false
    }
  }

  async function loadRemotes() {
    if (!repoPath.value) return
    try {
      const res = await api.gitRemotes(repoPath.value)
      remotesList.value = (res as any).remotes || []
    } catch (e: any) {
      toast.error('加载远程仓库失败: ' + e.message)
    }
  }

  async function loadSubmodules() {
    if (!repoPath.value) return
    smLoading.value = true
    try {
      const res = await api.gitSubmoduleList(repoPath.value)
      submodulesList.value = (res as any).submodules || []
    } catch (e: any) {
      toast.error('加载子模块失败: ' + e.message)
    } finally {
      smLoading.value = false
    }
  }

  async function doSubmoduleInit(recursive: boolean = true) {
    if (!repoPath.value) return
    try {
      await api.gitSubmoduleInit(repoPath.value, recursive)
      toast.success('初始化子模块成功')
      await loadSubmodules()
    } catch (e: any) {
      toast.error('初始化失败: ' + e.message)
    }
  }

  async function showFileHistory(file: string) {
    if (!repoPath.value) return
    fileHistoryFile.value = file
    showFileHistoryDialog.value = true
    try {
      const res = await api.gitFileHistory(repoPath.value, file, 20)
      fileHistoryData.value = res
    } catch (e: any) {
      toast.error('加载文件历史失败: ' + e.message)
    }
  }

  async function showFileBlame(file: string) {
    if (!repoPath.value) return
    blameFile.value = file
    showBlameDialog.value = true
    try {
      const res = await api.gitFileBlame(repoPath.value, file)
      blameData.value = res
    } catch (e: any) {
      toast.error('加载Blame失败: ' + e.message)
    }
  }

  async function doDiscardChanges(file: string) {
    if (!repoPath.value) return
    try {
      await api.gitDiscardChanges(repoPath.value, file)
      toast.success('丢弃更改成功')
      await loadStatus()
    } catch (e: any) {
      toast.error('丢弃更改失败: ' + e.message)
    }
  }

  async function loadUnpushedCommits() {
    if (!repoPath.value) return
    try {
      const res = await api.gitUnpushedCommits(repoPath.value)
      return (res as any).count || 0
    } catch (e: any) {
      return 0
    }
  }

  // 辅助函数
  function formatRelativeDate(dateStr: string): string {
    const date = new Date(dateStr)
    const now = new Date()
    const diff = now.getTime() - date.getTime()
    const days = Math.floor(diff / (1000 * 60 * 60 * 24))
    if (days === 0) return '今天'
    if (days === 1) return '昨天'
    if (days < 7) return `${days}天前`
    if (days < 30) return `${Math.floor(days / 7)}周前`
    return `${Math.floor(days / 30)}月前`
  }

  function formatFullDate(dateStr: string): string {
    return new Date(dateStr).toLocaleString('zh-CN')
  }

  // 初始化加载
  watch(repoPath, (path) => {
    if (path) refreshAll()
  }, { immediate: true })

  return {
    // 状态
    toast, api, loading, currentBranch, statusData, selectedFiles, collapsedGroups,
    commitMessage, committing, totalChanges,
    repoPath,

    // Log
    logData, logSearch, logBranchFilter, selectedCommit, commitDiff, loadingDiff,
    logCount, filteredLog, hasMoreLog,
    loadLog, loadMoreLog, selectCommit, loadCommitDiff,

    // Branch
    branchesData, localBranches, remoteBranches, showBranchesPopup, showCreateBranch,
    newBranchName, newBranchFrom, mergeTarget, merging,
    loadBranches, loadCurrentBranch, checkoutBranch, doCreateBranch, doDeleteBranch, doMerge,

    // Push/Pull
    pushing, pulling, showPushDialog, showPullDialog, pushForce,
    doPush, doPull, doFetch, doForcePush,

    // Commit
    doCommit, doCommitAndPush,

    // File
    toggleGroup, toggleFileSelect, selectAllFiles, doDiscardChanges,

    // Stash
    showStashPanel, stashList, selectedStash, showStashSaveDialog, stashSaveMessage, stashIncludeUntracked,
    loadStashList, openStashSave, doStashSave, doStashPop, doStashDrop,

    // Tags
    showTagsDialog, tagsList, showCreateTagDialog, newTagName, newTagMessage,
    loadTags, doCreateTag, doDeleteTag,

    // Advanced
    cherryPickTarget, cherryPicking, revertTarget, reverting,
    showRebaseDialog, rebaseTarget, rebasing,
    showAmendDialog, amendMessage, amending,
    showResetDialog, resetTarget, resetMode, resetting,
    showRemotesDialog, remotesList, showSubmodulesDialog, submodulesList, smLoading,
    showFileHistoryDialog, fileHistoryFile, fileHistoryData,
    showBlameDialog, blameFile, blameData,
    contextMenu, logContextMenu,

    // Functions
    doCherryPick, doRevert, doRebase, doRebaseAbort, doRebaseContinue, doAmend, doReset,
    loadRemotes, loadSubmodules, doSubmoduleInit,
    showFileHistory, showFileBlame, loadUnpushedCommits,
    refreshAll, loadStatus,

    // Utils
    formatRelativeDate, formatFullDate,
  }
}