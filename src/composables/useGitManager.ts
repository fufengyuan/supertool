import { ref, computed } from 'vue'
import { getTauriAPI } from '../utils/tauri-api'
import { useToast } from './useToast'

export function useGitManager(_repo: any, _onClose: () => void) {
  const toast = useToast()
  const api = getTauriAPI()
  const loading = ref(false)
  const currentBranch = ref('')
  const statusData = ref<any>(null)
  const selectedFiles = ref(new Set<string>())
  const collapsedGroups = ref(new Set<string>())
  const commitMessage = ref('')
  const committing = ref(false)
  const totalChanges = ref(0)
  const logData = ref<any[]>([])
  const logSearch = ref('')
  const logBranchFilter = ref('')
  const selectedCommit = ref<any>(null)
  const commitDiff = ref<any>(null)
  const loadingDiff = ref(false)
  const logCount = ref(0)
  const logTotalEstimate = ref(0)
  const filteredLog = computed(() => logData.value)
  const hasMoreLog = ref(false)
  const logViewMode = ref('list')
  const graphLog = ref<any[]>([])
  const graphLoading = ref(false)
  const graphCanvasRef = ref<HTMLCanvasElement | null>(null)
  const graphHoveredIndex = ref(-1)
  const graphSelectedCommit = ref<any>(null)
  const BRANCH_COLORS = ['#e06c75', '#61afef', '#98c379', '#e5c07b', '#c678dd', '#56b6c2']

  const branchesData = ref<any[]>([])
  const showBranchesPopup = ref(false)
  const showCreateBranch = ref(false)
  const newBranchName = ref('')
  const newBranchFrom = ref('')
  const mergeTarget = ref('')
  const merging = ref(false)

  // Stub functions - all return resolved promises
  const noop = (..._args: any[]) => {}
  const asyncNoop = async () => {}
  const asyncTrue = async () => true
  const asyncEmpty = async () => []
  const asyncNull = async () => null

  return {
    toast, api, loading, currentBranch, statusData, selectedFiles, collapsedGroups,
    commitMessage, committing, totalChanges,
    toggleGroup: (_g: string) => {}, toggleFileSelect: (_f: string) => {}, selectAllFiles: noop,
    logData, logSearch, logBranchFilter, selectedCommit, commitDiff, loadingDiff,
    logCount, logTotalEstimate, filteredLog, hasMoreLog, logViewMode,
    graphLog, graphLoading, graphCanvasRef, graphHoveredIndex, graphSelectedCommit,
    BRANCH_COLORS,
    switchToGraphView: noop, loadGraphLog: asyncNoop, drawGraph: noop,
    onGraphMouseMove: noop, onGraphClick: noop,
    consoleHistory: ref<string[]>([]), consoleInput: ref(''), consoleInputRef: ref<any>(null),
    consoleOutputRef: ref<any>(null), consoleHistoryIndex: ref(0), consoleInputHistory: ref<string[]>([]),
    execConsoleCommand: asyncNoop, scrollToConsoleBottom: noop,
    consoleHistoryUp: noop, consoleHistoryDown: noop,
    resizeObserver: ref<any>(null),
    getAuthorName: async (_author: string) => "", formatRelativeDate: (d: any) => String(d),
    formatFullDate: (d: any) => String(d), parseRefs: (s: string) => [],
    selectCommit: async (_commit: any) => {}, loadCommitDiff: asyncNoop, loadLog: async (_opts?: any) => {}, loadMoreLog: asyncNoop,
    branchesData, showBranchesPopup, showCreateBranch, newBranchName, newBranchFrom,
    mergeTarget, merging,
    showStashPanel: ref(false), stashList: ref([]), selectedStash: ref(null),
    stashShowContent: ref(''), showStashSaveDialog: ref(false), stashSaveMessage: ref(''),
    stashIncludeUntracked: ref(false), stashSaveInput: ref(false), stashContextMenu: ref(false),
    showTagsDialog: ref(false), tagsList: ref([]), showCreateTagDialog: ref(false),
    newTagName: ref(''), newTagCommit: ref(''), newTagMessage: ref(''),
    cherryPickTarget: ref(''), cherryPicking: ref(false), revertTarget: ref(''), reverting: ref(false),
    showFileHistoryDialog: ref(false), fileHistoryFile: ref(''), fileHistoryData: ref(null),
    showBlameDialog: ref(false), blameFile: ref(''), blameData: ref(null),
    showCompareBranchesDialog: ref(false), compareBranchTarget: ref(''), compareResult: ref(null),
    showRebaseDialog: ref(false), rebaseTarget: ref(''), rebasing: ref(false), rebaseInProgress: ref(false),
    showCompareWithDialog: ref(false), compareWithTarget: ref(''), compareWithCommit: ref(''),
    showPushDialog: ref(false), pushRemote: ref(''), pushBranch: ref(''), pushForce: ref(false),
    pushSetUpstream: ref(false), pushUnpushedCommits: ref(0),
    showPullDialog: ref(false), pullRemote: ref(''), pullBranch: ref(''), pullRebase: ref(false),
    pullAutoStash: ref(false),
    showAmendDialog: ref(false), amendMessage: ref(''), amendNoEdit: ref(false), amending: ref(false),
    showResetDialog: ref(false), resetTarget: ref(''), resetMode: ref('hard'), resetting: ref(false),
    showInteractiveRebaseDialog: ref(false), interactiveRebaseBase: ref('HEAD~5'),
    irCommits: ref([]), irSelectedIndex: ref(-1), irLoading: ref(false),
    showRemotesDialog: ref(false), remotesList: ref([]), remoteUrls: ref({}),
    showAddRemoteForm: ref(false), newRemoteName: ref(''), newRemoteUrl: ref(''),
    showBranchRenameDialog: ref(false), branchRenameOld: ref(''), branchRenameNew: ref(''),
    branchRenameInput: ref(''),
    logContextMenu: ref(false), logDateFrom: ref(''), logDateTo: ref(''),
    showAuthorFilter: ref(false), logAuthors: ref([]), selectedAuthors: ref([]),
    commitSignOff: ref(false), commitNoVerify: ref(false),
    showSubmodulesDialog: ref(false), submodulesList: ref([]), smLoading: ref(false),
    showCompareCommitsDialog: ref(false), compareCommitFrom: ref(''), compareCommitTo: ref(''),
    compareCommitsDiff: ref(null), ccLoading: ref(false),
    showGetFileRevisionDialog: ref(false), getFileCommit: ref(''), getFilePath: ref(''),
    getFileContent: ref(''), showGetFilePreviewDialog: ref(false),
    showCreatePatchDialog: ref(false), patchFrom: ref(''), patchTo: ref(''), patchOutputDir: ref(''),
    showApplyPatchDialog: ref(false), applyPatchFile: ref(''), applyPatchCheck: ref(false),
    applyPatchSign: ref(false), applyPatch3way: ref(false), applyPatchResult: ref(null), applyPatchError: ref(''),
    showCherryPickMultiDialog: ref(false), selectedLogCommits: ref([]), cherryPickMultiNoCommit: ref(false),
    showGitCleanDialog: ref(false), gitCleanIncludeIgnored: ref(false), gitCleanForceDirectories: ref(false),
    gitCleanFiles: ref([]), gcLoading: ref(false), selectedTagForBranch: ref(''),
    deleteRemoteBranchTarget: ref(''), deletingBranch: ref(false),
    localBranches: ref([]), remoteBranches: ref([]),
    loadBranches: asyncNoop, loadCurrentBranch: asyncNoop, checkoutBranch: async (_branch: string) => {},
    doCreateBranch: asyncNoop, confirmDeleteBranch: asyncNoop, doDeleteBranch: asyncNoop,
    showMergeDialog: noop, doMerge: asyncNoop,
    pulling: ref(false), pushing: ref(false), doPull: async (_opts?: any) => {}, doPush: async (_opts?: any) => {},
    doCommit: async (_noVerify: boolean) => {}, doCommitAndPush: asyncNoop,
    contextMenu: ref(false), showFileContextMenu: noop, contextMenuAction: noop, closeContextMenu: noop,
    loadStashList: asyncNoop, openStashSave: noop, openStashSaveIncludeUntracked: noop,
    doStashSave: asyncNoop, selectStash: noop, showStashContextMenu: noop, stashContextAction: noop,
    loadTags: asyncNoop, openTagsDialog: noop, openCreateTag: noop, doCreateTag: asyncNoop,
    confirmDeleteTag: asyncNoop, doDeleteTag: asyncNoop, doCherryPick: asyncNoop, doRevert: asyncNoop,
    showFileHistory: noop, showFileBlame: noop,
    openCompareBranchesDialog: noop, doCompareBranches: asyncNoop,
    openRebaseDialog: noop, doRebase: asyncNoop, doRebaseAbort: asyncNoop, doRebaseContinue: asyncNoop,
    doCompareWith: asyncNoop, openPushDialog: noop, openPullDialog: noop,
    loadRemotes: asyncNoop, loadUnpushedCommits: asyncNoop,
    doPushWithOptions: asyncNoop, doForcePush: asyncNoop, doPullWithOptions: asyncNoop,
    doFetch: asyncNoop, doFetchRemote: asyncNoop,
    openAmendDialog: noop, doAmend: asyncNoop, openResetDialog: noop, doReset: asyncNoop,
    openInteractiveRebaseDialog: noop, loadInteractiveRebaseCommits: asyncNoop,
    irMoveUp: noop, irMoveDown: noop, doInteractiveRebase: asyncNoop,
    openRemotesDialog: noop, openAddRemote: noop, doAddRemote: asyncNoop,
    confirmDeleteRemote: asyncNoop, doDeleteRemote: asyncNoop,
    openBranchRename: noop, doBranchRename: asyncNoop,
    addToGitignore: asyncNoop, showLogContextMenu: noop, logContextAction: noop,
    commitPanelWidth: ref('400px'), isResizing: ref(false), startResize: noop,
    toggleAuthor: noop, doCommitWithOptions: asyncNoop, doUndoLastCommit: asyncNoop,
    openSubmodulesDialog: noop, loadSubmodules: asyncNoop, doSubmoduleInit: asyncNoop,
    doSubmoduleUpdate: asyncNoop, doSubmoduleInitAll: asyncNoop, doSubmoduleUpdateAll: asyncNoop,
    openSubmodulePath: noop, doPushTags: asyncNoop,
    openCompareCommitsDialog: noop, doCompareCommits: asyncNoop,
    openGetFileRevisionDialog: noop, doGetFileAtRevision: asyncNoop, copyFileContent: noop,
    doCreatePatch: asyncNoop, selectPatchFile: asyncNoop, doApplyPatch: asyncNoop,
    toggleLogCommitSelect: noop, toggleSelectAllLogCommits: noop,
    getCommitMessage: asyncNoop, doCherryPickMulti: asyncNoop,
    openGitCleanDialog: noop, doGitCleanDryRun: asyncNoop, doGitClean: asyncNoop,
    openCreateBranchFromTag: noop, confirmDeleteRemoteBranch: asyncNoop,
    doDeleteRemoteBranch: asyncNoop, checkoutRemoteBranch: asyncNoop,
    repoPath: computed(() => ''), loadStatus: asyncNoop, refreshAll: asyncNoop,
  }
}
