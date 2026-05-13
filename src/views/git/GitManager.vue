<template>
  <div class="flex flex-col h-full overflow-hidden bg-base-200 text-base-content text-[11px]">
    <GitTopBar
      :repo="repo"
      :current-branch="currentBranch"
      :status-data="statusData"
      :loading="loading"
      :pulling="pulling"
      :pushing="pushing"
      :show-stash-menu="showStashMenu"
      :show-git-menu="showGitMenu"
      :show-file-browser="showFileBrowser"
      @close="$emit('close')"
      @update:show-stash-menu="showStashMenu = $event"
      @update:show-git-menu="showGitMenu = $event"
      @open-branches="showBranchesPopup = true"
      @git-action="handleGitAction"
      @stash-save="openStashSave()"
      @stash-save-untracked="openStashSaveIncludeUntracked()"
      @toggle-stash-panel="showStashPanel = !showStashPanel"
      @toggle-file-browser="showFileBrowser = !showFileBrowser"
      @pull="doPull()"
      @push="doPush()"
      @force-push="doForcePush()"
      @refresh="refreshAll()"
    />

    <!-- ===== 主内容区域 ===== -->
    <div class="flex flex-1 overflow-hidden">
      <!-- ===== 文件浏览模式 ===== -->
      <template v-if="showFileBrowser">
        <GitFileTree
          :repo-path="repo.path"
          @select-file="handleSelectFile"
        />
        <div class="w-1 shrink-0 bg-base-content/8 hover:bg-base-content/12 transition-colors duration-150"></div>
        <GitCodeEditor
          v-if="selectedFilePath"
          :repo-path="repo.path"
          :file-path="selectedFilePath"
          @close="selectedFilePath = null"
          @saved="handleFileSaved"
        />
        <div v-else class="flex-1 flex items-center justify-center bg-base-100 text-base-content/50 text-xs">
          <span>选择文件查看内容</span>
        </div>
      </template>

      <!-- ===== Git 模式 ===== -->
      <template v-else>
        <!-- ===== 左侧：提交面板 ===== -->
        <GitCommitPanel
        :status-data="statusData"
        :loading="loading"
        :selected-files="selectedFiles"
        :collapsed-groups="collapsedGroups"
        :commit-message="commitMessage"
        :committing="committing"
        :total-changes="totalChanges"
        :commit-sign-off="commitSignOff"
        :commit-no-verify="commitNoVerify"
        :preview-diff="previewDiff"
        :selected-preview-file="selectedPreviewFile"
        :loading-preview="loadingPreview"
        @update:commit-message="commitMessage = $event"
        @update:commit-sign-off="commitSignOff = $event"
        @update:commit-no-verify="commitNoVerify = $event"
        @toggle-group="toggleGroup"
        @toggle-file-select="toggleFileSelect"
        @select-all-files="selectAllFiles"
        @commit="handleCommit"
        @file-context-menu="showFileContextMenu($event.event, $event.file, $event.type)"
        @preview-file="previewCommitFile"
        @clear-preview="clearPreview"
      />

      <!-- ===== Stash 面板 ===== -->
      <GitStashPanel
        v-if="showStashPanel"
        :stash-list="stashList"
        :selected-stash="selectedStash"
        :stash-show-content="stashShowContent"
        :loading="loading"
        @update:selected-stash="selectedStash = $event"
        @select-stash="selectStash"
        @stash-context-menu="showStashContextMenu($event.event, $event.stash)"
        @open-stash-save="openStashSave"
      />

      <!-- ===== 中间分割条 ===== -->
      <div class="w-1 cursor-col-resize shrink-0 bg-base-content/10 hover:bg-primary transition-colors duration-150" @mousedown="startResize"></div>

      <!-- ===== 右侧：日志面板 ===== -->
      <GitLogPanel
        :log-view-mode="logViewMode"
        :log-search="logSearch"
        :log-date-from="logDateFrom"
        :log-date-to="logDateTo"
        :show-author-filter="showAuthorFilter"
        :log-authors="logAuthors"
        :selected-authors="selectedAuthors"
        :filtered-log="filteredLog"
        :selected-log-commits="selectedLogCommits"
        :selected-commit="selectedCommit"
        :commit-diff="commitDiff"
        :loading-diff="loadingDiff"
        :loading="loading"
        :has-more-log="hasMoreLog"
        :log-count="logCount"
        :log-total-estimate="logTotalEstimate"
        :graph-log="graphLog"
        :graph-loading="graphLoading"
        :graph-hovered-index="graphHoveredIndex"
        :graph-selected-commit="graphSelectedCommit"
        :branch-colors="BRANCH_COLORS"
        :console-history="consoleHistory"
        :console-input="consoleInput"
        :local-branches="localBranches"
        :get-author-name="getAuthorName"
        :format-relative-date="formatRelativeDate"
        :format-full-date="formatFullDate"
        :parse-refs="parseRefs"
        @update:log-view-mode="logViewMode = $event"
        @update:log-search="logSearch = $event"
        @update:log-date-from="logDateFrom = $event"
        @update:log-date-to="logDateTo = $event"
        @update:show-author-filter="showAuthorFilter = $event"
        @update:selected-commit="selectedCommit = $event"
        @update:console-input="consoleInput = $event"
        @toggle-author="toggleAuthor($event)"
        @load-log="loadLog()"
        @load-more-log="loadMoreLog()"
        @select-commit="selectCommit($event)"
        @toggle-log-commit-select="toggleLogCommitSelect($event)"
        @toggle-select-all-log-commits="toggleSelectAllLogCommits()"
        @log-context-menu="showLogContextMenu($event.event, $event.commit)"
        @exec-console-command="execConsoleCommand()"
        @console-history-up="consoleHistoryUp()"
        @console-history-down="consoleHistoryDown()"
        @switch-to-graph-view="switchToGraphView()"
        @on-graph-mouse-move="onGraphMouseMove($event)"
        @on-graph-click="onGraphClick($event)"
        @load-commit-diff="loadCommitDiff()"
      />
      </template>
    </div>

    <!-- ===== 右键菜单 ===== -->
    <div
      v-if="contextMenu.show"
      class="fixed z-[1000] bg-base-100 border border-base-content/10 rounded-btn shadow-lg min-w-[180px] py-1"
      :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
      @click.stop
    >
      <div class="flex items-center gap-2 px-3 py-1.5 cursor-pointer text-xs hover:bg-[var(--hover-bg)]" @click="contextMenuAction('diff')">
        <SvgIcon name="pencil" size="14" />
        查看差异
      </div>
      <div class="flex items-center gap-2 px-3 py-1.5 cursor-pointer text-xs hover:bg-[var(--hover-bg)]" @click="contextMenuAction('history')">
        <SvgIcon name="clock" size="14" />
        查看历史
      </div>
      <div class="flex items-center gap-2 px-3 py-1.5 cursor-pointer text-xs hover:bg-[var(--hover-bg)]" @click="contextMenuAction('blame')">
        <SvgIcon name="user" size="14" />
        查看作者
      </div>
      <div class="h-px bg-base-content/10 my-1"></div>
      <div class="flex items-center gap-2 px-3 py-1.5 cursor-pointer text-xs hover:bg-[var(--hover-bg)]" @click="contextMenuAction('compareBranch')">
        <SvgIcon name="barChart" size="14" />
        与分支比较...
      </div>
      <div class="h-px bg-base-content/10 my-1"></div>
      <div class="flex items-center gap-2 px-3 py-1.5 cursor-pointer text-xs hover:bg-[var(--hover-bg)]" @click="contextMenuAction('discard')" v-if="contextMenu.fileType !== 'untracked'">
        <SvgIcon name="undo" size="14" />
        撤销更改
      </div>
      <div class="flex items-center gap-2 px-3 py-1.5 cursor-pointer text-xs hover:bg-[var(--hover-bg)]" @click="contextMenuAction('add')" v-if="contextMenu.fileType === 'untracked'">
        <SvgIcon name="plus" size="14" />
        添加到版本控制
      </div>
      <div class="flex items-center gap-2 px-3 py-1.5 cursor-pointer text-xs hover:bg-[var(--hover-bg)]" @click="contextMenuAction('reset')" v-if="contextMenu.fileType !== 'untracked'">
        <SvgIcon name="x" size="14" />
        从版本控制移除
      </div>
      <div class="h-px bg-base-content/10 my-1"></div>
      <div class="flex items-center gap-2 px-3 py-1.5 cursor-pointer text-xs hover:bg-[var(--hover-bg)]" @click="contextMenuAction('gitignore')">
        <SvgIcon name="file" :size="14" />
        添加到 .gitignore
      </div>
    </div>

    <!-- ===== 日志行右键菜单 ===== -->
    <div
      v-if="logContextMenu.show"
      class="fixed z-[1000] bg-base-100 border border-base-content/10 rounded-btn shadow-lg min-w-[180px] py-1"
      :style="{ left: logContextMenu.x + 'px', top: logContextMenu.y + 'px' }"
      @click.stop
    >
      <div class="flex items-center gap-2 px-3 py-1.5 cursor-pointer text-xs hover:bg-[var(--hover-bg)]" @click="logContextAction('cherry-pick')">
        <SvgIcon name="arrowDown" :size="14" />
        拣选提交
      </div>
      <div class="flex items-center gap-2 px-3 py-1.5 cursor-pointer text-xs hover:bg-[var(--hover-bg)]" @click="logContextAction('revert')">
        <SvgIcon name="undo" size="14" />
        回退提交
      </div>
      <div class="h-px bg-base-content/10 my-1"></div>
      <div class="flex items-center gap-2 px-3 py-1.5 cursor-pointer text-xs hover:bg-[var(--hover-bg)]" @click="logContextAction('create-tag')">
        <SvgIcon name="tag" size="14" />
        创建标签...
      </div>
      <div class="flex items-center gap-2 px-3 py-1.5 cursor-pointer text-xs hover:bg-[var(--hover-bg)]" @click="logContextAction('compare-commits')">
        <SvgIcon name="barChart" size="14" />
        与其他提交比较...
      </div>
      <div class="flex items-center gap-2 px-3 py-1.5 cursor-pointer text-xs hover:bg-[var(--hover-bg)]" @click="logContextAction('get-file')">
        <SvgIcon name="download" :size="14" />
        获取该版本文件...
      </div>
      <div class="h-px bg-base-content/10 my-1"></div>
      <div class="flex items-center gap-2 px-3 py-1.5 cursor-pointer text-xs hover:bg-[var(--hover-bg)]" @click="logContextAction('cherry-pick-multi')">
        <SvgIcon name="arrowDown" :size="14" />
        批量拣选提交...
      </div>
      <div class="h-px bg-base-content/10 my-1"></div>
      <div class="flex items-center gap-2 px-3 py-1.5 cursor-pointer text-xs hover:bg-[var(--hover-bg)]" @click="logContextAction('compare-with')">
        <SvgIcon name="barChart" size="14" />
        与指定提交比较...
      </div>
    </div>

    <!-- ===== Stash 右键菜单 ===== -->
    <div
      v-if="stashContextMenu.show"
      class="fixed z-[1000] bg-base-100 border border-base-content/10 rounded-btn shadow-lg min-w-[180px] py-1"
      :style="{ left: stashContextMenu.x + 'px', top: stashContextMenu.y + 'px' }"
      @click.stop
    >
      <div class="flex items-center gap-2 px-3 py-1.5 cursor-pointer text-xs hover:bg-[var(--hover-bg)]" @click="stashContextAction('apply')">
        <SvgIcon name="arrowRight" :size="14" />
        应用
      </div>
      <div class="flex items-center gap-2 px-3 py-1.5 cursor-pointer text-xs hover:bg-[var(--hover-bg)]" @click="stashContextAction('pop')">
        <SvgIcon name="upload" size="14" />
        弹出
      </div>
      <div class="h-px bg-base-content/10 my-1"></div>
      <div class="flex items-center gap-2 px-3 py-1.5 cursor-pointer text-xs hover:bg-[var(--hover-bg)] hover:text-red-500 hover:bg-red-500/10" @click="stashContextAction('drop')">
        <SvgIcon name="trash" size="14" />
        删除
      </div>
    </div>

    <!-- ===== 分支管理 (提取为子组件) ===== -->
    <GitBranchPopup
      :show-branches-popup="showBranchesPopup"
      :show-create-branch="showCreateBranch"
      :show-branch-rename-dialog="showBranchRenameDialog"
      :local-branches="localBranches"
      :remote-branches="remoteBranches"
      :current-branch="currentBranch"
      :new-branch-name="newBranchName"
      :new-branch-from="newBranchFrom"
      :branch-rename-old="branchRenameOld"
      :branch-rename-new="branchRenameNew"
      :merge-target="mergeTarget"
      :merging="merging"
      @update:show-branches-popup="showBranchesPopup = $event"
      @update:show-create-branch="showCreateBranch = $event"
      @update:show-branch-rename-dialog="showBranchRenameDialog = $event"
      @update:new-branch-name="newBranchName = $event"
      @update:new-branch-from="newBranchFrom = $event"
      @update:branch-rename-new="branchRenameNew = $event"
      @update:merge-target="mergeTarget = $event"
      @checkout-branch="checkoutBranch"
      @create-branch="doCreateBranch"
      @delete-branch="confirmDeleteBranch"
      @show-merge-dialog="showMergeDialog"
      @open-branch-rename="openBranchRename"
      @do-branch-rename="doBranchRename"
      @checkout-remote-branch="checkoutRemoteBranch"
      @delete-remote-branch="confirmDeleteRemoteBranch"
      @merge="doMerge"
    />

    <!-- ===== 对话框子组件 ===== -->
    <GitConfirmDialogs
      :cherry-pick-target="cherryPickTarget"
      :revert-target="revertTarget"
      :delete-remote-branch-target="deleteRemoteBranchTarget"
      :cherry-picking="cherryPicking"
      :reverting="reverting"
      :deleting-branch="deletingBranch"
      @update:cherry-pick-target="cherryPickTarget = $event"
      @update:revert-target="revertTarget = $event"
      @update:delete-remote-branch-target="deleteRemoteBranchTarget = $event"
      @cherry-pick="doCherryPick"
      @revert="doRevert"
      @delete-remote-branch="doDeleteRemoteBranch"
    />
    <GitFormDialogs
      :show-stash-save-dialog="showStashSaveDialog"
      :stash-save-message="stashSaveMessage"
      :stash-include-untracked="stashIncludeUntracked"
      :show-create-tag-dialog="showCreateTagDialog"
      :new-tag-name="newTagName"
      :new-tag-commit="newTagCommit"
      :new-tag-message="newTagMessage"
      :show-tags-dialog="showTagsDialog"
      :tags-list="tagsList"
      :selected-tag-for-branch="selectedTagForBranch"
      :show-compare-branches-dialog="showCompareBranchesDialog"
      :local-branches="localBranches"
      :current-branch="currentBranch"
      :compare-branch-target="compareBranchTarget"
      :compare-result="compareResult"
      :show-push-dialog="showPushDialog"
      :remotes-list="remotesList"
      :push-remote="pushRemote"
      :push-branch="pushBranch"
      :push-force="pushForce"
      :push-set-upstream="pushSetUpstream"
      :push-unpushed-commits="pushUnpushedCommits"
      :pushing="pushing"
      :show-pull-dialog="showPullDialog"
      :pull-remote="pullRemote"
      :pull-branch="pullBranch"
      :pull-rebase="pullRebase"
      :pull-auto-stash="pullAutoStash"
      :pulling="pulling"
      :show-rebase-dialog="showRebaseDialog"
      :rebase-target="rebaseTarget"
      :rebasing="rebasing"
      :rebase-in-progress="rebaseInProgress"
      :show-reset-dialog="showResetDialog"
      :reset-target="resetTarget"
      :reset-mode="resetMode"
      :resetting="resetting"
      :show-amend-dialog="showAmendDialog"
      :amend-message="amendMessage"
      :amend-no-edit="amendNoEdit"
      :amending="amending"
      :loading="loading"
      @update:show-stash-save-dialog="showStashSaveDialog = $event"
      @update:stash-save-message="stashSaveMessage = $event"
      @update:stash-include-untracked="stashIncludeUntracked = $event"
      @stash-save="doStashSave"
      @update:show-create-tag-dialog="showCreateTagDialog = $event"
      @update:new-tag-name="newTagName = $event"
      @update:new-tag-commit="newTagCommit = $event"
      @update:new-tag-message="newTagMessage = $event"
      @create-tag="doCreateTag"
      @update:show-tags-dialog="showTagsDialog = $event"
      @update:selected-tag-for-branch="selectedTagForBranch = $event"
      @create-tag-open="openCreateTag"
      @create-branch-from-tag="openCreateBranchFromTag"
      @delete-tag="confirmDeleteTag"
      @update:show-compare-branches-dialog="showCompareBranchesDialog = $event"
      @update:compare-branch-target="compareBranchTarget = $event"
      @compare-branches="doCompareBranches"
      @update:show-push-dialog="showPushDialog = $event"
      @update:push-remote="pushRemote = $event"
      @update:push-branch="pushBranch = $event"
      @update:push-force="pushForce = $event"
      @update:push-set-upstream="pushSetUpstream = $event"
      @push-with-options="doPushWithOptions"
      @update:show-pull-dialog="showPullDialog = $event"
      @update:pull-remote="pullRemote = $event"
      @update:pull-branch="pullBranch = $event"
      @update:pull-rebase="pullRebase = $event"
      @update:pull-auto-stash="pullAutoStash = $event"
      @pull-with-options="doPullWithOptions"
      @update:show-rebase-dialog="showRebaseDialog = $event"
      @update:rebase-target="rebaseTarget = $event"
      @rebase="doRebase"
      @rebase-abort="doRebaseAbort"
      @rebase-continue="doRebaseContinue"
      @update:show-reset-dialog="showResetDialog = $event"
      @update:reset-target="resetTarget = $event"
      @update:reset-mode="resetMode = $event"
      @reset="doReset"
      @update:show-amend-dialog="showAmendDialog = $event"
      @update:amend-message="amendMessage = $event"
      @update:amend-no-edit="amendNoEdit = $event"
      @amend="doAmend"
    />
    <GitAdvancedDialogs
      :show-file-history-dialog="showFileHistoryDialog"
      :file-history-file="fileHistoryFile"
      :file-history-data="fileHistoryData"
      :show-blame-dialog="showBlameDialog"
      :blame-file="blameFile"
      :blame-data="blameData"
      :show-interactive-rebase-dialog="showInteractiveRebaseDialog"
      :interactive-rebase-base="interactiveRebaseBase"
      :ir-commits="irCommits"
      :ir-selected-index="irSelectedIndex"
      :ir-loading="irLoading"
      :show-remotes-dialog="showRemotesDialog"
      :remotes-list="remotesList"
      :remote-urls="remoteUrls"
      :show-add-remote-form="showAddRemoteForm"
      :new-remote-name="newRemoteName"
      :new-remote-url="newRemoteUrl"
      :show-submodules-dialog="showSubmodulesDialog"
      :submodules-list="submodulesList"
      :sm-loading="smLoading"
      :show-compare-commits-dialog="showCompareCommitsDialog"
      :compare-commit-from="compareCommitFrom"
      :compare-commit-to="compareCommitTo"
      :compare-commits-diff="compareCommitsDiff"
      :cc-loading="ccLoading"
      :show-get-file-revision-dialog="showGetFileRevisionDialog"
      :get-file-commit="getFileCommit"
      :get-file-path="getFilePath"
      :show-get-file-preview-dialog="showGetFilePreviewDialog"
      :get-file-content="getFileContent"
      :show-create-patch-dialog="showCreatePatchDialog"
      :patch-from="patchFrom"
      :patch-to="patchTo"
      :patch-output-dir="patchOutputDir"
      :show-apply-patch-dialog="showApplyPatchDialog"
      :apply-patch-file="applyPatchFile"
      :apply-patch-check="applyPatchCheck"
      :apply-patch-sign="applyPatchSign"
      :apply-patch-3way="applyPatch3way"
      :apply-patch-result="applyPatchResult"
      :apply-patch-error="applyPatchError"
      :show-cherry-pick-multi-dialog="showCherryPickMultiDialog"
      :selected-log-commits="selectedLogCommits"
      :cherry-pick-multi-no-commit="cherryPickMultiNoCommit"
      :cherry-picking="cherryPicking"
      :get-commit-message="getCommitMessage"
      :show-git-clean-dialog="showGitCleanDialog"
      :git-clean-include-ignored="gitCleanIncludeIgnored"
      :git-clean-force-directories="gitCleanForceDirectories"
      :git-clean-files="gitCleanFiles"
      :gc-loading="gcLoading"
      :loading="loading"
      :get-author-name="getAuthorName"
      :format-relative-date="formatRelativeDate"
      :format-full-date="formatFullDate"
      @update:show-file-history-dialog="showFileHistoryDialog = $event"
      @update:show-blame-dialog="showBlameDialog = $event"
      @update:show-interactive-rebase-dialog="showInteractiveRebaseDialog = $event"
      @update:interactive-rebase-base="interactiveRebaseBase = $event"
      @load-ir-commits="loadInteractiveRebaseCommits"
      @ir-move-up="irMoveUp($event)"
      @ir-move-down="irMoveDown($event)"
      @start-interactive-rebase="doInteractiveRebase"
      @update:show-remotes-dialog="showRemotesDialog = $event"
      @update:show-add-remote-form="showAddRemoteForm = $event"
      @update:new-remote-name="newRemoteName = $event"
      @update:new-remote-url="newRemoteUrl = $event"
      @add-remote="doAddRemote"
      @fetch-remote="doFetchRemote($event)"
      @delete-remote="confirmDeleteRemote($event)"
      @update:show-submodules-dialog="showSubmodulesDialog = $event"
      @submodule-init-all="doSubmoduleInitAll"
      @submodule-update-all="doSubmoduleUpdateAll"
      @refresh-submodules="loadSubmodules"
      @submodule-init="doSubmoduleInit($event)"
      @submodule-update="doSubmoduleUpdate($event)"
      @open-submodule-path="openSubmodulePath($event)"
      @update:show-compare-commits-dialog="showCompareCommitsDialog = $event"
      @update:compare-commit-from="compareCommitFrom = $event"
      @update:compare-commit-to="compareCommitTo = $event"
      @compare-commits="doCompareCommits"
      @update:show-get-file-revision-dialog="showGetFileRevisionDialog = $event"
      @update:get-file-commit="getFileCommit = $event"
      @update:get-file-path="getFilePath = $event"
      @get-file-at-revision="doGetFileAtRevision"
      @update:show-get-file-preview-dialog="showGetFilePreviewDialog = $event"
      @copy-file-content="copyFileContent"
      @update:show-create-patch-dialog="showCreatePatchDialog = $event"
      @update:patch-from="patchFrom = $event"
      @update:patch-to="patchTo = $event"
      @update:patch-output-dir="patchOutputDir = $event"
      @create-patch="doCreatePatch"
      @update:show-apply-patch-dialog="showApplyPatchDialog = $event"
      @update:apply-patch-file="applyPatchFile = $event"
      @update:apply-patch-check="applyPatchCheck = $event"
      @update:apply-patch-sign="applyPatchSign = $event"
      @update:apply-patch-3way="applyPatch3way = $event"
      @select-patch-file="selectPatchFile"
      @apply-patch="doApplyPatch"
      @update:show-cherry-pick-multi-dialog="showCherryPickMultiDialog = $event"
      @update:cherry-pick-multi-no-commit="cherryPickMultiNoCommit = $event"
      @cherry-pick-multi="doCherryPickMulti"
      @update:show-git-clean-dialog="showGitCleanDialog = $event"
      @update:git-clean-include-ignored="gitCleanIncludeIgnored = $event"
      @update:git-clean-force-directories="gitCleanForceDirectories = $event"
      @git-clean-dry-run="doGitCleanDryRun"
      @git-clean="doGitClean"
    />

    <!-- ===== Compare with (commit) 对话框 ===== -->
    <div v-if="showCompareWithDialog" class="fixed inset-0 bg-black/40 z-[900] flex items-center justify-center" @click="showCompareWithDialog = false">
      <div class="max-w-lg w-full bg-base-100 border border-base-content/10 rounded-xl shadow-2xl p-6" @click.stop>
        <h3 class="text-lg font-semibold m-0 mb-4">比较提交</h3>
        <div class="flex flex-col gap-2.5 mb-4">
          <label class="text-xs font-semibold text-base-content/60">对比分支</label>
          <select v-model="compareWithTarget" class="p-2 px-2.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[13px] outline-none focus:border-primary">
            <option value="">选择分支...</option>
            <option v-for="b in localBranches" :key="b.name" :value="b.name">{{ b.name }}</option>
          </select>
        </div>
        <div class="flex justify-end gap-2">
          <button class="btn btn-ghost btn-sm" @click="showCompareWithDialog = false">取消</button>
          <button class="btn btn-primary btn-sm" @click="doCompareWith" :disabled="!compareWithTarget">比较</button>
        </div>
      </div>
    </div>


  </div>
</template>

<script setup lang="ts">
// @ts-nocheck — TODO: 需要修复 useGitManager composable 的类型系统
import { ref } from 'vue'
import { onMounted, onUnmounted } from 'vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import type { GitRepo } from '../../types'
import { useGitManager } from '../../composables/useGitManager'
import GitTopBar from './GitTopBar.vue'
import GitCommitPanel from './GitCommitPanel.vue'
import GitLogPanel from './GitLogPanel.vue'
import GitStashPanel from './GitStashPanel.vue'
import GitBranchPopup from './GitBranchPopup.vue'
import GitConfirmDialogs from './GitConfirmDialogs.vue'
import GitFormDialogs from './GitFormDialogs.vue'
import GitAdvancedDialogs from './GitAdvancedDialogs.vue'
import GitFileTree from './GitFileTree.vue'
import GitCodeEditor from './GitCodeEditor.vue'

// 文件浏览状态
const showFileBrowser = ref(false)
const selectedFilePath = ref<string | null>(null)

function handleSelectFile(path: string) {
  selectedFilePath.value = path
}

function handleFileSaved(path: string) {
  // 文件保存后刷新 Git status
  loadStatus()
}

const props = defineProps<{
  repo: GitRepo
}>()

const emit = defineEmits<{
  'close': []
}>()

const {
  toast,
  api,
  loading,
  currentBranch,
  statusData,
  selectedFiles,
  collapsedGroups,
  commitMessage,
  committing,
  totalChanges,
  toggleGroup,
  toggleFileSelect,
  selectAllFiles,
  logData,
  logSearch,
  selectedCommit,
  commitDiff,
  loadingDiff,
  logCount,
  logTotalEstimate,
  filteredLog,
  hasMoreLog,
  logViewMode,
  graphLog,
  graphLoading,
  graphCanvasRef,
  graphHoveredIndex,
  graphSelectedCommit,
  BRANCH_COLORS,
  switchToGraphView,
  loadGraphLog,
  drawGraph,
  onGraphMouseMove,
  onGraphClick,
  consoleHistory,
  consoleInput,
  consoleInputRef,
  consoleOutputRef,
  consoleHistoryIndex,
  consoleInputHistory,
  execConsoleCommand,
  scrollToConsoleBottom,
  consoleHistoryUp,
  consoleHistoryDown,
  resizeObserver,
  getAuthorName,
  formatRelativeDate,
  formatFullDate,
  parseRefs,
  selectCommit,
  loadCommitDiff,
  loadLog,
  loadMoreLog,
  branchesData,
  showBranchesPopup,
  showCreateBranch,
  newBranchName,
  newBranchFrom,
  mergeTarget,
  merging,
  showStashPanel,
  stashList,
  selectedStash,
  stashShowContent,
  showStashSaveDialog,
  stashSaveMessage,
  stashIncludeUntracked,
  stashSaveInput,
  stashContextMenu,
  showTagsDialog,
  tagsList,
  showCreateTagDialog,
  newTagName,
  newTagCommit,
  newTagMessage,
  cherryPickTarget,
  cherryPicking,
  revertTarget,
  reverting,
  showFileHistoryDialog,
  fileHistoryFile,
  fileHistoryData,
  showBlameDialog,
  blameFile,
  blameData,
  showCompareBranchesDialog,
  compareBranchTarget,
  compareResult,
  showRebaseDialog,
  rebaseTarget,
  rebasing,
  rebaseInProgress,
  showCompareWithDialog,
  compareWithTarget,
  compareWithCommit,
  showPushDialog,
  pushRemote,
  pushBranch,
  pushForce,
  pushSetUpstream,
  pushUnpushedCommits,
  showPullDialog,
  pullRemote,
  pullBranch,
  pullRebase,
  pullAutoStash,
  showAmendDialog,
  amendMessage,
  amendNoEdit,
  amending,
  showResetDialog,
  resetTarget,
  resetMode,
  resetting,
  showInteractiveRebaseDialog,
  interactiveRebaseBase,
  irCommits,
  irSelectedIndex,
  irLoading,
  showRemotesDialog,
  remotesList,
  remoteUrls,
  showAddRemoteForm,
  newRemoteName,
  newRemoteUrl,
  showBranchRenameDialog,
  branchRenameOld,
  branchRenameNew,
  logContextMenu,
  logDateFrom,
  logDateTo,
  showAuthorFilter,
  logAuthors,
  selectedAuthors,
  commitSignOff,
  commitNoVerify,
  showSubmodulesDialog,
  submodulesList,
  smLoading,
  showCompareCommitsDialog,
  compareCommitFrom,
  compareCommitTo,
  compareCommitsDiff,
  ccLoading,
  showGetFileRevisionDialog,
  getFileCommit,
  getFilePath,
  getFileContent,
  showGetFilePreviewDialog,
  showCreatePatchDialog,
  patchFrom,
  patchTo,
  patchOutputDir,
  showApplyPatchDialog,
  applyPatchFile,
  applyPatchCheck,
  applyPatchSign,
  applyPatch3way,
  applyPatchResult,
  applyPatchError,
  showCherryPickMultiDialog,
  selectedLogCommits,
  cherryPickMultiNoCommit,
  showGitCleanDialog,
  gitCleanIncludeIgnored,
  gitCleanForceDirectories,
  gitCleanFiles,
  gcLoading,
  selectedTagForBranch,
  deleteRemoteBranchTarget,
  deletingBranch,
  localBranches,
  remoteBranches,
  loadBranches,
  loadCurrentBranch,
  checkoutBranch,
  doCreateBranch,
  confirmDeleteBranch,
  doDeleteBranch,
  showMergeDialog,
  doMerge,
  pulling,
  pushing,
  doPull,
  doPush,
  doCommit,
  doCommitAndPush,
  contextMenu,
  showFileContextMenu,
  contextMenuAction,
  closeContextMenu,
  loadStashList,
  openStashSave,
  openStashSaveIncludeUntracked,
  doStashSave,
  selectStash,
  showStashContextMenu,
  stashContextAction,
  loadTags,
  openTagsDialog,
  openCreateTag,
  doCreateTag,
  confirmDeleteTag,
  doDeleteTag,
  doCherryPick,
  doRevert,
  showFileHistory,
  showFileBlame,
  openCompareBranchesDialog,
  doCompareBranches,
  openRebaseDialog,
  doRebase,
  doRebaseAbort,
  doRebaseContinue,
  doCompareWith,
  openPushDialog,
  openPullDialog,
  loadRemotes,
  loadUnpushedCommits,
  doPushWithOptions,
  doForcePush,
  doPullWithOptions,
  doFetch,
  doFetchRemote,
  openAmendDialog,
  doAmend,
  openResetDialog,
  doReset,
  openInteractiveRebaseDialog,
  loadInteractiveRebaseCommits,
  irMoveUp,
  irMoveDown,
  doInteractiveRebase,
  openRemotesDialog,
  openAddRemote,
  doAddRemote,
  confirmDeleteRemote,
  doDeleteRemote,
  openBranchRename,
  doBranchRename,
  addToGitignore,
  showLogContextMenu,
  logContextAction,
  commitPanelWidth,
  isResizing,
  startResize,
  toggleAuthor,
  doCommitWithOptions,
  doUndoLastCommit,
  openSubmodulesDialog,
  loadSubmodules,
  doSubmoduleInit,
  doSubmoduleUpdate,
  doSubmoduleInitAll,
  doSubmoduleUpdateAll,
  openSubmodulePath,
  doPushTags,
  openCompareCommitsDialog,
  doCompareCommits,
  openGetFileRevisionDialog,
  doGetFileAtRevision,
  copyFileContent,
  doCreatePatch,
  selectPatchFile,
  doApplyPatch,
  toggleLogCommitSelect,
  toggleSelectAllLogCommits,
  getCommitMessage,
  doCherryPickMulti,
  openGitCleanDialog,
  doGitCleanDryRun,
  doGitClean,
  openCreateBranchFromTag,
  confirmDeleteRemoteBranch,
  doDeleteRemoteBranch,
  checkoutRemoteBranch,
  repoPath,
  loadStatus,
  refreshAll,
  previewDiff,
  selectedPreviewFile,
  loadingPreview,
  previewCommitFile,
  clearPreview,
} = useGitManager(props.repo, () => emit("close"))

// Local dropdown state (composable returns noop stubs for these)
const showStashMenu = ref(false)
const showGitMenu = ref(false)

// 点击页面空白处关闭所有右键菜单
function handleDocumentClick() {
  if (contextMenu.value.show) contextMenu.value.show = false
  if (logContextMenu.value.show) logContextMenu.value.show = false
  if (stashContextMenu.value.show) stashContextMenu.value.show = false
}

onMounted(() => document.addEventListener('click', handleDocumentClick))
onUnmounted(() => document.removeEventListener('click', handleDocumentClick))

function handleGitAction(action: string) {
  switch (action) {
    case 'rebase': openRebaseDialog(); break
    case 'compare-branches': openCompareBranchesDialog(); break
    case 'tags': openTagsDialog(); break
    case 'submodules': openSubmodulesDialog(); break
    case 'push-dialog': openPushDialog(); break
    case 'pull-dialog': openPullDialog(); break
    case 'fetch': doFetch(); break
    case 'push-tags': doPushTags(); break
    case 'undo-last-commit': doUndoLastCommit(); break
    case 'amend': openAmendDialog(); break
    case 'reset': openResetDialog(); break
    case 'interactive-rebase': openInteractiveRebaseDialog(); break
    case 'remotes': openRemotesDialog(); break
    case 'create-patch': showCreatePatchDialog.value = true; break
    case 'apply-patch': showApplyPatchDialog.value = true; break
    case 'git-clean': openGitCleanDialog(); break
  }
}

function handleCommit(shouldPush: boolean) {
  if (shouldPush) {
    doCommitAndPush()
  } else {
    doCommitWithOptions()
  }
}

</script>
<style>
/* CSS moved to sub-components */
</style>
