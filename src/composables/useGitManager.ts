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
  const statusData = ref<{
    modified: string[]
    added: string[]
    deleted: string[]
    untracked: string[]
    conflicted: string[]
    ahead: number
    behind: number
  } | null>(null)
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
  const contextMenu = ref({ show: false, x: 0, y: 0, file: '', fileType: '' })
  const logContextMenu = ref({ show: false, x: 0, y: 0, commit: null as any })
  const stashContextMenu = ref({ show: false, x: 0, y: 0, stash: null as any })

  // Log 扩展状态
  const logTotalEstimate = ref(0)
  const logViewMode = ref<'list' | 'graph'>('list')
  const logDateFrom = ref('')
  const logDateTo = ref('')
  const showAuthorFilter = ref(false)
  const logAuthors = ref<string[]>([])
  const selectedAuthors = ref<Set<string>>(new Set())
  const selectedLogCommits = ref<Set<string>>(new Set())
  const graphLog = ref<any[]>([])
  const graphLoading = ref(false)
  const graphHoveredIndex = ref(-1)
  const graphSelectedCommit = ref<any>(null)
  const BRANCH_COLORS: Record<string, string> = {}
  const graphCanvasRef = ref<HTMLCanvasElement | null>(null)

  // Console 状态
  const consoleHistory = ref<string[]>([])
  const consoleInput = ref('')
  const consoleInputRef = ref<HTMLInputElement | null>(null)
  const consoleOutputRef = ref<HTMLDivElement | null>(null)
  const consoleHistoryIndex = ref(-1)
  const consoleInputHistory = ref<string[]>([])
  const resizeObserver = ref<ResizeObserver | null>(null)

  // Stash 扩展状态
  const stashShowContent = ref('')
  const stashSaveInput = ref<HTMLInputElement | null>(null)

  // Tag 扩展状态
  const newTagCommit = ref('')

  // 对话框状态
  const showCompareBranchesDialog = ref(false)
  const compareBranchTarget = ref('')
  const compareResult = ref<any>(null)
  const rebaseInProgress = ref(false)
  const showCompareWithDialog = ref(false)
  const compareWithTarget = ref('')
  const compareWithCommit = ref('')
  const pushRemote = ref('')
  const pushBranch = ref('')
  const pushSetUpstream = ref(false)
  const pushUnpushedCommits = ref<number[]>([])
  const pullRemote = ref('')
  const pullBranch = ref('')
  const pullRebase = ref(false)
  const pullAutoStash = ref(false)
  const amendNoEdit = ref(false)
  const showInteractiveRebaseDialog = ref(false)
  const interactiveRebaseBase = ref('')
  const irCommits = ref<any[]>([])
  const irSelectedIndex = ref(-1)
  const irLoading = ref(false)
  const remoteUrls = ref<Record<string, { fetch: string; push: string }>>({})
  const showAddRemoteForm = ref(false)
  const newRemoteName = ref('')
  const newRemoteUrl = ref('')
  const showBranchRenameDialog = ref(false)
  const branchRenameOld = ref('')
  const branchRenameNew = ref('')
  const commitSignOff = ref(false)
  const commitNoVerify = ref(false)
  const showCompareCommitsDialog = ref(false)
  const compareCommitFrom = ref('')
  const compareCommitTo = ref('')
  const compareCommitsDiff = ref<any>(null)
  const ccLoading = ref(false)
  const showGetFileRevisionDialog = ref(false)
  const getFileCommit = ref('')
  const getFilePath = ref('')
  const getFileContent = ref('')
  const showGetFilePreviewDialog = ref(false)
  const showCreatePatchDialog = ref(false)
  const patchFrom = ref('')
  const patchTo = ref('')
  const patchOutputDir = ref('')
  const showApplyPatchDialog = ref(false)
  const applyPatchFile = ref('')
  const applyPatchCheck = ref(false)
  const applyPatchSign = ref(false)
  const applyPatch3way = ref(false)
  const applyPatchResult = ref('')
  const applyPatchError = ref('')
  const showCherryPickMultiDialog = ref(false)
  const cherryPickMultiNoCommit = ref(false)
  const showGitCleanDialog = ref(false)
  const gitCleanIncludeIgnored = ref(false)
  const gitCleanForceDirectories = ref(false)
  const gitCleanFiles = ref<any[]>([])
  const gcLoading = ref(false)
  const selectedTagForBranch = ref('')
  const deleteRemoteBranchTarget = ref('')
  const deletingBranch = ref(false)
  const showMergeDialog = ref(false)

  // 布局状态
  const commitPanelWidth = ref(300)
  const isResizing = ref(false)

  // Repo path
  const repoPath = computed(() => repo?.path || '')

  // ============ 核心加载函数 ============

  async function loadStatus() {
    if (!repoPath.value) return
    loading.value = true
    try {
      const res = await api.gitStatus(repoPath.value)
      const files = (res as any).files || []
      // 转换数据结构：后端返回 { files: [{path, type}] }，前端需要 { modified: [], added: [], ... }
      const grouped: any = {
        modified: [],
        added: [],
        deleted: [],
        untracked: [],
        conflicted: [],
        ahead: 0,
        behind: 0
      }
      for (const f of files) {
        const type = f.type || 'untracked'
        if (grouped[type]) {
          grouped[type].push(f.path)
        }
      }
      statusData.value = grouped
      totalChanges.value = files.length
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
    if (!statusData.value) return
    const allFiles = [
      ...statusData.value.modified,
      ...statusData.value.added,
      ...statusData.value.deleted,
      ...statusData.value.untracked,
      ...statusData.value.conflicted
    ]
    allFiles.forEach(f => selectedFiles.value.add(f))
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

  // ============ 右键菜单操作 ============

  function showFileContextMenu(event: MouseEvent, file: string, type: string) {
    event.preventDefault()
    contextMenu.value = {
      show: true,
      x: event.clientX,
      y: event.clientY,
      file,
      fileType: type
    }
  }

  function showLogContextMenu(event: MouseEvent, commit: any) {
    event.preventDefault()
    logContextMenu.value = {
      show: true,
      x: event.clientX,
      y: event.clientY,
      commit
    }
  }

  function closeContextMenu() {
    contextMenu.value.show = false
    logContextMenu.value.show = false
  }

  function contextMenuAction(action: string) {
    const file = contextMenu.value.file
    switch (action) {
      case 'diff':
        // TODO: show diff
        break
      case 'history':
        showFileHistory(file)
        break
      case 'blame':
        showFileBlame(file)
        break
      case 'discard':
        doDiscardChanges(file)
        break
      case 'add':
        // git add file
        break
      case 'reset':
        // git reset file
        break
      case 'gitignore':
        addToGitignore(file)
        break
    }
    closeContextMenu()
  }

  function logContextAction(action: string) {
    const commit = logContextMenu.value.commit
    switch (action) {
      case 'cherry-pick':
        cherryPickTarget.value = commit?.hash || ''
        doCherryPick()
        break
      case 'revert':
        revertTarget.value = commit?.hash || ''
        doRevert()
        break
      case 'create-tag':
        newTagName.value = ''
        newTagMessage.value = ''
        showCreateTagDialog.value = true
        break
    }
    closeContextMenu()
  }

  function addToGitignore(file: string) {
    // TODO: implement
    toast.info('添加到 .gitignore: ' + file)
  }

  // ============ 占位函数（未实现） ============

  function switchToGraphView() { logViewMode.value = 'graph' }
  function loadGraphLog() { /* TODO */ }
  function drawGraph() { /* TODO */ }
  function onGraphMouseMove(_e: MouseEvent) { /* TODO */ }
  function onGraphClick(_e: MouseEvent) { /* TODO */ }
  function execConsoleCommand() { /* TODO */ }
  function scrollToConsoleBottom() { /* TODO */ }
  function consoleHistoryUp() { /* TODO */ }
  function consoleHistoryDown() { /* TODO */ }
  function getAuthorName(commit: any): string { return commit?.authorName || '' }
  function parseRefs(refs: string): string[] { return refs?.split(',').map(r => r.trim()).filter(Boolean) || [] }
  function selectStash(stash: any) { selectedStash.value = stash }
  function showStashContextMenu(event: MouseEvent, stash: any) {
    event.preventDefault()
    stashContextMenu.value = { show: true, x: event.clientX, y: event.clientY, stash }
  }
  function stashContextAction(action: string) {
    const stash = stashContextMenu.value.stash
    switch (action) {
      case 'pop': doStashPop(stash?.ref); break
      case 'drop': doStashDrop(stash?.ref); break
    }
    stashContextMenu.value.show = false
  }
  function openStashSaveIncludeUntracked() {
    showStashSaveDialog.value = true
    stashSaveMessage.value = ''
    stashIncludeUntracked.value = true
  }
  function confirmDeleteBranch(_name: string) { /* TODO: show confirm dialog */ }
  function openCompareBranchesDialog() { showCompareBranchesDialog.value = true }
  async function doCompareBranches() { /* TODO */ }
  function openRebaseDialog() { showRebaseDialog.value = true }
  function doCompareWith() { /* TODO */ }
  function openPushDialog() { showPushDialog.value = true }
  function openPullDialog() { showPullDialog.value = true }
  async function doPushWithOptions() { /* TODO */ }
  function doFetchRemote(_remote: string) { /* TODO */ }
  function openAmendDialog() { showAmendDialog.value = true }
  function openResetDialog() { showResetDialog.value = true }
  function openInteractiveRebaseDialog() { /* TODO */ }
  function loadInteractiveRebaseCommits() { /* TODO */ }
  function irMoveUp() { /* TODO */ }
  function irMoveDown() { /* TODO */ }
  function doInteractiveRebase() { /* TODO */ }
  function openRemotesDialog() { showRemotesDialog.value = true; loadRemotes() }
  function openAddRemote() { showAddRemoteForm.value = true }
  async function doAddRemote() { /* TODO */ }
  function confirmDeleteRemote(_name: string) { /* TODO */ }
  async function doDeleteRemote(_name: string) { /* TODO */ }
  function openBranchRename() { showBranchRenameDialog.value = true }
  function doBranchRename() { /* TODO */ }
  function toggleAuthor(_author: string) { /* TODO */ }
  function doCommitWithOptions() { /* TODO */ }
  function doUndoLastCommit() { /* TODO */ }
  function openSubmodulesDialog() { showSubmodulesDialog.value = true; loadSubmodules() }
  function doSubmoduleUpdate(_path?: string) { /* TODO */ }
  function doSubmoduleInitAll() { /* TODO */ }
  function doSubmoduleUpdateAll() { /* TODO */ }
  function openSubmodulePath(_path: string) { /* TODO */ }
  async function doPushTags() { /* TODO */ }
  function openCompareCommitsDialog() { showCompareCommitsDialog.value = true }
  async function doCompareCommits() { /* TODO */ }
  function openGetFileRevisionDialog() { showGetFileRevisionDialog.value = true }
  async function doGetFileAtRevision() { /* TODO */ }
  function copyFileContent() { /* TODO */ }
  async function doCreatePatch() { /* TODO */ }
  function selectPatchFile() { /* TODO */ }
  async function doApplyPatch() { /* TODO */ }
  function toggleLogCommitSelect(_hash: string) { /* TODO */ }
  function toggleSelectAllLogCommits() { /* TODO */ }
  function getCommitMessage(_commit: any): string { return '' }
  function doCherryPickMulti() { /* TODO */ }
  function openGitCleanDialog() { showGitCleanDialog.value = true }
  function doGitCleanDryRun() { /* TODO */ }
  function doGitClean() { /* TODO */ }
  function openCreateBranchFromTag() { /* TODO */ }
  function confirmDeleteRemoteBranch(_name: string) { /* TODO */ }
  function doDeleteRemoteBranch(_name: string) { /* TODO */ }
  function checkoutRemoteBranch(_name: string) { /* TODO */ }
  function startResize(_e: MouseEvent) { isResizing.value = true }
  function doPullWithOptions() { /* TODO */ }
  function openTagsDialog() { showTagsDialog.value = true; loadTags() }
  function openCreateTag() { showCreateTagDialog.value = true }
  function confirmDeleteTag(_name: string) { /* TODO */ }

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
    logCount, logTotalEstimate, filteredLog, hasMoreLog,
    logViewMode, logDateFrom, logDateTo, showAuthorFilter, logAuthors, selectedAuthors,
    graphLog, graphLoading, graphHoveredIndex, graphSelectedCommit, BRANCH_COLORS, graphCanvasRef,
    switchToGraphView, loadGraphLog, drawGraph, onGraphMouseMove, onGraphClick,
    consoleHistory, consoleInput, consoleInputRef, consoleOutputRef,
    consoleHistoryIndex, consoleInputHistory, execConsoleCommand,
    scrollToConsoleBottom, consoleHistoryUp, consoleHistoryDown, resizeObserver,
    getAuthorName, parseRefs, selectedLogCommits,
    loadLog, loadMoreLog, selectCommit, loadCommitDiff,

    // Branch
    branchesData, localBranches, remoteBranches, showBranchesPopup, showCreateBranch,
    newBranchName, newBranchFrom, mergeTarget, merging, showMergeDialog,
    loadBranches, loadCurrentBranch, checkoutBranch, doCreateBranch,
    confirmDeleteBranch, doDeleteBranch, doMerge,

    // Push/Pull
    pushing, pulling, showPushDialog, showPullDialog, pushForce,
    pushRemote, pushBranch, pushSetUpstream, pushUnpushedCommits,
    pullRemote, pullBranch, pullRebase, pullAutoStash,
    doPush, doPull, doForcePush, doFetch, doFetchRemote,
    openPushDialog, openPullDialog, doPushWithOptions, doPullWithOptions, loadUnpushedCommits,

    // Commit
    commitSignOff, commitNoVerify,
    doCommit, doCommitAndPush, doCommitWithOptions, doUndoLastCommit,

    // File
    toggleGroup, toggleFileSelect, selectAllFiles, doDiscardChanges,

    // Stash
    showStashPanel, stashList, selectedStash, stashShowContent,
    showStashSaveDialog, stashSaveMessage, stashIncludeUntracked, stashSaveInput, stashContextMenu,
    loadStashList, openStashSave, openStashSaveIncludeUntracked,
    doStashSave, doStashPop, doStashDrop, selectStash,
    showStashContextMenu, stashContextAction,

    // Tags
    showTagsDialog, tagsList, showCreateTagDialog, newTagName, newTagCommit, newTagMessage,
    selectedTagForBranch,
    openTagsDialog, openCreateTag, loadTags, doCreateTag, confirmDeleteTag, doDeleteTag,
    openCreateBranchFromTag,

    // Advanced
    cherryPickTarget, cherryPicking, revertTarget, reverting,
    showRebaseDialog, rebaseTarget, rebasing, rebaseInProgress,
    showAmendDialog, amendMessage, amendNoEdit, amending,
    showResetDialog, resetTarget, resetMode, resetting,
    showInteractiveRebaseDialog, interactiveRebaseBase, irCommits, irSelectedIndex, irLoading,
    openInteractiveRebaseDialog, loadInteractiveRebaseCommits, irMoveUp, irMoveDown, doInteractiveRebase,
    showRemotesDialog, remotesList, remoteUrls, showAddRemoteForm, newRemoteName, newRemoteUrl,
    openRemotesDialog, openAddRemote, doAddRemote, confirmDeleteRemote, doDeleteRemote,
    showBranchRenameDialog, branchRenameOld, branchRenameNew,
    openBranchRename, doBranchRename,
    showCompareBranchesDialog, compareBranchTarget, compareResult,
    showCompareWithDialog, compareWithTarget, compareWithCommit,
    showCompareCommitsDialog, compareCommitFrom, compareCommitTo, compareCommitsDiff, ccLoading,
    showGetFileRevisionDialog, getFileCommit, getFilePath, getFileContent, showGetFilePreviewDialog,
    showCreatePatchDialog, patchFrom, patchTo, patchOutputDir,
    showApplyPatchDialog, applyPatchFile, applyPatchCheck, applyPatchSign, applyPatch3way, applyPatchResult, applyPatchError,
    showCherryPickMultiDialog, cherryPickMultiNoCommit,
    showGitCleanDialog, gitCleanIncludeIgnored, gitCleanForceDirectories, gitCleanFiles, gcLoading,
    deleteRemoteBranchTarget, deletingBranch,
    showFileHistoryDialog, fileHistoryFile, fileHistoryData,
    showBlameDialog, blameFile, blameData,
    showSubmodulesDialog, submodulesList, smLoading,
    openSubmodulesDialog, loadSubmodules, doSubmoduleInit, doSubmoduleUpdate,
    doSubmoduleInitAll, doSubmoduleUpdateAll, openSubmodulePath,
    contextMenu, logContextMenu,
    showFileContextMenu, showLogContextMenu, closeContextMenu,
    contextMenuAction, logContextAction, addToGitignore,
    toggleAuthor, toggleLogCommitSelect, toggleSelectAllLogCommits, getCommitMessage,
    doCompareBranches,
    doRebase, doRebaseAbort, doRebaseContinue, doCompareWith,
    doAmend, doReset,
    doCherryPick, doRevert, doCherryPickMulti,
    openGetFileRevisionDialog, doGetFileAtRevision, copyFileContent,
    openCompareCommitsDialog, doCompareCommits,
    doCreatePatch, selectPatchFile, doApplyPatch,
    openGitCleanDialog, doGitCleanDryRun, doGitClean,
    confirmDeleteRemoteBranch, doDeleteRemoteBranch, checkoutRemoteBranch,
    doPushTags,

    // Layout
    commitPanelWidth, isResizing, startResize,

    // Functions
    showFileHistory, showFileBlame,
    refreshAll, loadStatus,

    // Utils
    formatRelativeDate, formatFullDate,
  }
}