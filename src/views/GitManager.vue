<template>
  <div class="git-manager">
    <!-- ===== 顶部工具栏 ===== -->
    <div class="git-topbar">
      <div class="topbar-left">
        <button class="btn btn-ghost btn-sm back-btn" @click="$emit('close')" title="返回仓库列表">
          <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="15 18 9 12 15 6" />
          </svg>
        </button>
        <div class="repo-info">
          <span class="repo-name">{{ repo.name }}</span>
          <span class="repo-path" :title="repo.path">{{ repo.path }}</span>
        </div>
      </div>

      <div class="topbar-center">
        <div class="branch-selector" @click="showBranchesPopup = true">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="6" y1="3" x2="6" y2="15" />
            <circle cx="18" cy="6" r="3" />
            <circle cx="6" cy="18" r="3" />
            <path d="M18 9a9 9 0 0 1-9 9" />
          </svg>
          <span class="branch-name">{{ currentBranch || '...' }}</span>
          <svg class="dropdown-arrow" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="6 9 12 15 18 9" />
          </svg>
        </div>
        <span v-if="statusData" class="ahead-behind">
          <span v-if="statusData.ahead > 0" class="ahead" title="Ahead of remote">↑{{ statusData.ahead }}</span>
          <span v-if="statusData.behind > 0" class="behind" title="Behind remote">↓{{ statusData.behind }}</span>
        </span>
      </div>

      <div class="topbar-right">
        <!-- Stash 下拉按钮 -->
        <div class="dropdown-wrap">
          <button class="btn btn-ghost btn-sm" @click="showStashMenu = !showStashMenu" title="Stash">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
              <line x1="3" y1="9" x2="21" y2="9" />
              <line x1="9" y1="21" x2="9" y2="9" />
            </svg>
            Stash
            <svg class="dropdown-arrow" viewBox="0 0 24 24" width="10" height="10" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="6 9 12 15 18 9" />
            </svg>
          </button>
          <div v-if="showStashMenu" class="dropdown-menu" @click.stop>
            <div class="dropdown-menu-item" @click="openStashSave">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
              </svg>
              Save Stash...
            </div>
            <div class="dropdown-menu-item" @click="openStashSaveIncludeUntracked">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
              </svg>
              Stash All (incl. untracked)
            </div>
            <div class="dropdown-menu-separator"></div>
            <div class="dropdown-menu-item" @click="showStashPanel = !showStashPanel">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
                <line x1="3" y1="9" x2="21" y2="9" />
              </svg>
              Show Stash List
            </div>
          </div>
        </div>
        <!-- Git 菜单按钮 -->
        <div class="dropdown-wrap">
          <button class="btn btn-ghost btn-sm" @click="showGitMenu = !showGitMenu" title="Git Operations">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="18" cy="18" r="3" /><circle cx="6" cy="6" r="3" />
              <path d="M6 21V9a9 9 0 0 0 9 9" />
            </svg>
            Git
            <svg class="dropdown-arrow" viewBox="0 0 24 24" width="10" height="10" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="6 9 12 15 18 9" />
            </svg>
          </button>
          <div v-if="showGitMenu" class="dropdown-menu" @click.stop>
            <div class="dropdown-menu-item" @click="openRebaseDialog">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="17 1 21 5 17 9" /><path d="M3 11V9a4 4 0 0 1 4-4h14" />
                <polyline points="7 23 3 19 7 15" /><path d="M21 13v2a4 4 0 0 1-4 4H3" />
              </svg>
              Rebase...
            </div>
            <div class="dropdown-menu-item" @click="openCompareBranchesDialog">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="18" y1="20" x2="18" y2="10" /><line x1="12" y1="20" x2="12" y2="4" />
                <line x1="6" y1="20" x2="6" y2="14" />
              </svg>
              Compare Branches...
            </div>
            <div class="dropdown-menu-item" @click="openTagsDialog">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z" />
                <line x1="7" y1="7" x2="7.01" y2="7" />
              </svg>
              Tags...
            </div>
            <div class="dropdown-menu-item" @click="openSubmodulesDialog">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="2" y="2" width="9" height="9" rx="1" /><rect x="13" y="2" width="9" height="9" rx="1" />
                <rect x="2" y="13" width="9" height="9" rx="1" /><rect x="13" y="13" width="9" height="9" rx="1" />
              </svg>
              Submodules...
            </div>
            <div class="dropdown-menu-separator"></div>
            <div class="dropdown-menu-item" @click="openPushDialog">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                <polyline points="17 8 12 3 7 8" />
                <line x1="12" y1="3" x2="12" y2="15" />
              </svg>
              Push...
            </div>
            <div class="dropdown-menu-item" @click="openPullDialog">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                <polyline points="7 10 12 15 17 10" />
                <line x1="12" y1="15" x2="12" y2="3" />
              </svg>
              Pull...
            </div>
            <div class="dropdown-menu-item" @click="doFetch">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="23 4 23 10 17 10" />
                <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
              </svg>
              Fetch
            </div>
            <div class="dropdown-menu-item" @click="doPushTags">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z" />
                <line x1="7" y1="7" x2="7.01" y2="7" />
                <polyline points="17 8 12 3 7 8" />
                <line x1="12" y1="3" x2="12" y2="15" />
              </svg>
              Push Tags
            </div>
            <div class="dropdown-menu-separator"></div>
            <div class="dropdown-menu-item" @click="doUndoLastCommit">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="1 4 1 10 7 10" /><path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
              </svg>
              Undo Last Commit
            </div>
            <div class="dropdown-menu-separator"></div>
            <div class="dropdown-menu-item" @click="openAmendDialog">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M12 20h9" /><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
              </svg>
              Amend Last Commit...
            </div>
            <div class="dropdown-menu-item" @click="openResetDialog">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="1 4 1 10 7 10" /><path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
              </svg>
              Reset to Commit...
            </div>
            <div class="dropdown-menu-item" @click="openInteractiveRebaseDialog">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="8" y1="6" x2="21" y2="6" /><line x1="8" y1="12" x2="21" y2="12" /><line x1="8" y1="18" x2="21" y2="18" />
                <line x1="3" y1="6" x2="3.01" y2="6" /><line x1="3" y1="12" x2="3.01" y2="12" /><line x1="3" y1="18" x2="3.01" y2="18" />
              </svg>
              Interactive Rebase...
            </div>
            <div class="dropdown-menu-item" @click="openRemotesDialog">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10" /><line x1="2" y1="12" x2="22" y2="12" />
                <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
              </svg>
              Remotes...
            </div>
            <div class="dropdown-menu-separator"></div>
            <div class="dropdown-menu-item" @click="showCreatePatchDialog = true">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
                <polyline points="14 2 14 8 20 8" /><line x1="16" y1="13" x2="8" y2="13" />
                <line x1="16" y1="17" x2="8" y2="17" />
              </svg>
              Create Patch...
            </div>
            <div class="dropdown-menu-item" @click="showApplyPatchDialog = true">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
                <polyline points="14 2 14 8 20 8" />
                <polyline points="9 15 12 18 16 13" />
              </svg>
              Apply Patch...
            </div>
            <div class="dropdown-menu-item" @click="openGitCleanDialog">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3 6 5 6 21 6" /><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
              </svg>
              Clean Working Tree...
            </div>
          </div>
        </div>
        <button class="btn btn-ghost btn-sm" @click="doPull" :disabled="pulling" title="Pull">
          <svg v-if="!pulling" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
            <polyline points="7 10 12 15 17 10" />
            <line x1="12" y1="15" x2="12" y2="3" />
          </svg>
          <svg v-else class="spin-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 12a9 9 0 1 1-6.219-8.56" />
          </svg>
          Pull
        </button>
        <button class="btn btn-ghost btn-sm" @click="doPush" :disabled="pushing" title="Push">
          <svg v-if="!pushing" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
            <polyline points="17 8 12 3 7 8" />
            <line x1="12" y1="3" x2="12" y2="15" />
          </svg>
          <svg v-else class="spin-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 12a9 9 0 1 1-6.219-8.56" />
          </svg>
          Push
        </button>
        <button class="btn btn-ghost btn-sm btn-danger" @click="doForcePush" :disabled="pushing" title="Force Push">
          <svg v-if="!pushing" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
            <polyline points="17 8 12 3 7 8" />
            <line x1="12" y1="3" x2="12" y2="15" />
            <line x1="5" y1="3" x2="19" y2="3" stroke-width="3" />
          </svg>
          <svg v-else class="spin-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 12a9 9 0 1 1-6.219-8.56" />
          </svg>
          Force Push
        </button>
        <button class="btn btn-ghost btn-sm" @click="showBranchesPopup = true" title="Merge/Branches">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="6" y1="3" x2="6" y2="15" />
            <circle cx="18" cy="6" r="3" />
            <circle cx="6" cy="18" r="3" />
            <path d="M18 9a9 9 0 0 1-9 9" />
          </svg>
          Branches
        </button>
        <button class="btn btn-ghost btn-sm" @click="refreshAll" :disabled="loading" title="Refresh">
          <svg :class="{ spin: loading }" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="23 4 23 10 17 10" />
            <polyline points="1 20 1 14 7 14" />
            <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
          </svg>
          Refresh
        </button>
      </div>
    </div>

    <!-- ===== 主内容区域 ===== -->
    <div class="git-main">
      <!-- ===== 左侧：提交面板 ===== -->
      <div class="commit-panel">
        <div class="panel-header">
          <span class="panel-title">
            变更
            <span class="change-count" v-if="totalChanges > 0">{{ totalChanges }}</span>
          </span>
          <div class="panel-header-actions">
            <button class="btn btn-ghost btn-xs" @click="selectAllFiles" :disabled="totalChanges === 0" title="全选">
              ☑ 全选
            </button>
          </div>
        </div>

        <!-- 文件变更列表 -->
        <div class="file-list" v-if="statusData">
          <!-- Modified -->
          <div class="file-group" v-if="statusData.modified.length">
            <div class="group-header" @click="toggleGroup('modified')">
              <svg class="group-arrow" :class="{ collapsed: collapsedGroups.has('modified') }" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="6 9 12 15 18 9" />
              </svg>
              <span class="group-icon modified">M</span>
              <span class="group-label">Modified</span>
              <span class="group-count">{{ statusData.modified.length }}</span>
            </div>
            <div class="group-files" v-show="!collapsedGroups.has('modified')">
              <div
                v-for="file in statusData.modified"
                :key="'M:' + file"
                class="file-item"
                :class="{ selected: selectedFiles.has(file) }"
                @click="toggleFileSelect(file)"
                @contextmenu.prevent="showFileContextMenu($event, file, 'modified')"
              >
                <input type="checkbox" :checked="selectedFiles.has(file)" @click.stop="toggleFileSelect(file)" class="file-checkbox" />
                <span class="file-icon modified">M</span>
                <span class="file-name" :title="file">{{ file }}</span>
              </div>
            </div>
          </div>

          <!-- Added -->
          <div class="file-group" v-if="statusData.added.length">
            <div class="group-header" @click="toggleGroup('added')">
              <svg class="group-arrow" :class="{ collapsed: collapsedGroups.has('added') }" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="6 9 12 15 18 9" />
              </svg>
              <span class="group-icon added">A</span>
              <span class="group-label">Added</span>
              <span class="group-count">{{ statusData.added.length }}</span>
            </div>
            <div class="group-files" v-show="!collapsedGroups.has('added')">
              <div
                v-for="file in statusData.added"
                :key="'A:' + file"
                class="file-item"
                :class="{ selected: selectedFiles.has(file) }"
                @click="toggleFileSelect(file)"
                @contextmenu.prevent="showFileContextMenu($event, file, 'added')"
              >
                <input type="checkbox" :checked="selectedFiles.has(file)" @click.stop="toggleFileSelect(file)" class="file-checkbox" />
                <span class="file-icon added">A</span>
                <span class="file-name" :title="file">{{ file }}</span>
              </div>
            </div>
          </div>

          <!-- Deleted -->
          <div class="file-group" v-if="statusData.deleted.length">
            <div class="group-header" @click="toggleGroup('deleted')">
              <svg class="group-arrow" :class="{ collapsed: collapsedGroups.has('deleted') }" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="6 9 12 15 18 9" />
              </svg>
              <span class="group-icon deleted">D</span>
              <span class="group-label">Deleted</span>
              <span class="group-count">{{ statusData.deleted.length }}</span>
            </div>
            <div class="group-files" v-show="!collapsedGroups.has('deleted')">
              <div
                v-for="file in statusData.deleted"
                :key="'D:' + file"
                class="file-item"
                :class="{ selected: selectedFiles.has(file) }"
                @click="toggleFileSelect(file)"
                @contextmenu.prevent="showFileContextMenu($event, file, 'deleted')"
              >
                <input type="checkbox" :checked="selectedFiles.has(file)" @click.stop="toggleFileSelect(file)" class="file-checkbox" />
                <span class="file-icon deleted">D</span>
                <span class="file-name" :title="file">{{ file }}</span>
              </div>
            </div>
          </div>

          <!-- Untracked -->
          <div class="file-group" v-if="statusData.untracked.length">
            <div class="group-header" @click="toggleGroup('untracked')">
              <svg class="group-arrow" :class="{ collapsed: collapsedGroups.has('untracked') }" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="6 9 12 15 18 9" />
              </svg>
              <span class="group-icon untracked">U</span>
              <span class="group-label">Unversioned</span>
              <span class="group-count">{{ statusData.untracked.length }}</span>
            </div>
            <div class="group-files" v-show="!collapsedGroups.has('untracked')">
              <div
                v-for="file in statusData.untracked"
                :key="'U:' + file"
                class="file-item"
                :class="{ selected: selectedFiles.has(file) }"
                @click="toggleFileSelect(file)"
                @contextmenu.prevent="showFileContextMenu($event, file, 'untracked')"
              >
                <input type="checkbox" :checked="selectedFiles.has(file)" @click.stop="toggleFileSelect(file)" class="file-checkbox" />
                <span class="file-icon untracked">U</span>
                <span class="file-name" :title="file">{{ file }}</span>
              </div>
            </div>
          </div>

          <!-- Conflicted -->
          <div class="file-group" v-if="statusData.conflicted.length">
            <div class="group-header" @click="toggleGroup('conflicted')">
              <svg class="group-arrow" :class="{ collapsed: collapsedGroups.has('conflicted') }" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="6 9 12 15 18 9" />
              </svg>
              <span class="group-icon conflicted">C</span>
              <span class="group-label">Conflicted</span>
              <span class="group-count">{{ statusData.conflicted.length }}</span>
            </div>
            <div class="group-files" v-show="!collapsedGroups.has('conflicted')">
              <div
                v-for="file in statusData.conflicted"
                :key="'C:' + file"
                class="file-item conflicted-item"
                :class="{ selected: selectedFiles.has(file) }"
                @click="toggleFileSelect(file)"
                @contextmenu.prevent="showFileContextMenu($event, file, 'conflicted')"
              >
                <input type="checkbox" :checked="selectedFiles.has(file)" @click.stop="toggleFileSelect(file)" class="file-checkbox" />
                <span class="file-icon conflicted">C</span>
                <span class="file-name" :title="file">{{ file }}</span>
              </div>
            </div>
          </div>
        </div>

        <div v-else-if="!loading" class="empty-files">
          <svg viewBox="0 0 24 24" width="36" height="36" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
            <polyline points="22 4 12 14.01 9 11.01" />
          </svg>
          <p>没有文件变更</p>
        </div>

        <!-- 提交信息区域 -->
        <div class="commit-area">
          <div class="commit-header">
            <span class="commit-title">提交信息</span>
          </div>
          <textarea
            v-model="commitMessage"
            class="commit-message-input"
            placeholder="输入提交信息..."
            rows="4"
            spellcheck="false"
          />
          <div class="commit-options">
            <label class="form-checkbox-label commit-option-label">
              <input type="checkbox" v-model="commitSignOff" />
              Sign-off (-s)
            </label>
            <label class="form-checkbox-label commit-option-label">
              <input type="checkbox" v-model="commitNoVerify" />
              No Verify (--no-verify)
            </label>
          </div>
          <div class="commit-actions">
            <button
              class="btn btn-primary btn-sm"
              @click="() => doCommit(false)"
              :disabled="committing || selectedFiles.size === 0 || !commitMessage.trim()"
            >
              Commit
            </button>
            <button
              class="btn btn-success btn-sm"
              @click="() => doCommit(true)"
              :disabled="committing || selectedFiles.size === 0 || !commitMessage.trim()"
            >
              Commit & Push
            </button>
          </div>
        </div>
      </div>

      <!-- ===== Stash 面板 ===== -->
      <div v-if="showStashPanel" class="stash-panel">
        <div class="panel-header">
          <span class="panel-title">
            Stash
            <span class="change-count" v-if="stashList.length > 0">{{ stashList.length }}</span>
          </span>
          <button class="btn btn-ghost btn-xs" @click="openStashSave" title="Stash Changes">
            <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
            </svg>
          </button>
        </div>
        <div class="stash-list">
          <div
            v-for="stash in stashList"
            :key="stash.name"
            class="stash-item"
            :class="{ selected: selectedStash?.name === stash.name }"
            @click="selectStash(stash)"
            @contextmenu.prevent="showStashContextMenu($event, stash)"
          >
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
              <line x1="3" y1="9" x2="21" y2="9" />
            </svg>
            <span class="stash-name">{{ stash.name }}</span>
            <span class="stash-desc" :title="stash.description">{{ stash.description }}</span>
          </div>
          <div v-if="stashList.length === 0 && !loading" class="stash-empty">
            <p>没有 Stash</p>
          </div>
        </div>
        <!-- Stash 预览 -->
        <div v-if="selectedStash" class="stash-preview">
          <div class="detail-header">
            <span class="detail-title">Stash 预览</span>
            <button class="btn btn-ghost btn-xs" @click="selectedStash = null" title="关闭">✕</button>
          </div>
          <pre class="diff-content">{{ stashShowContent || '加载中...' }}</pre>
        </div>
      </div>

      <!-- ===== 中间分割条 ===== -->
      <div class="splitter" @mousedown="startResize"></div>

      <!-- ===== 右侧：日志面板 ===== -->
      <div class="log-panel">
        <div class="panel-header">
          <span class="panel-title">日志</span>
          <div class="log-filters">
            <!-- View mode toggle: Table / Graph / Console -->
            <div class="log-view-toggle">
              <button
                class="view-toggle-btn"
                :class="{ active: logViewMode === 'table' }"
                @click="logViewMode = 'table'"
                title="表格视图"
              >
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="3" y="3" width="18" height="18" rx="2" />
                  <line x1="3" y1="9" x2="21" y2="9" />
                  <line x1="3" y1="15" x2="21" y2="15" />
                  <line x1="9" y1="3" x2="9" y2="21" />
                </svg>
              </button>
              <button
                class="view-toggle-btn"
                :class="{ active: logViewMode === 'graph' }"
                @click="switchToGraphView"
                title="图形视图 (Git Graph)"
              >
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                  <line x1="6" y1="3" x2="6" y2="21" />
                  <circle cx="6" cy="6" r="2" fill="currentColor" />
                  <circle cx="6" cy="12" r="2" fill="currentColor" />
                  <path d="M6 8 Q12 8 12 12" />
                  <circle cx="12" cy="12" r="2" fill="currentColor" />
                  <path d="M12 14 Q12 18 6 18" />
                </svg>
              </button>
              <button
                class="view-toggle-btn"
                :class="{ active: logViewMode === 'console' }"
                @click="logViewMode = 'console'"
                title="Git Console"
              >
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="4 17 10 11 4 5" />
                  <line x1="12" y1="19" x2="20" y2="19" />
                </svg>
              </button>
            </div>
            <div class="search-input-wrap">
              <svg class="search-icon" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="11" cy="11" r="8" />
                <line x1="21" y1="21" x2="16.65" y2="16.65" />
              </svg>
              <input
                v-model="logSearch"
                class="log-search-input"
                placeholder="搜索提交..."
                spellcheck="false"
              />
            </div>
            <select v-model="logBranchFilter" class="log-branch-filter">
              <option value="">所有分支</option>
              <option v-for="b in localBranches" :key="b.name" :value="b.name">{{ b.name }}</option>
            </select>
            <input v-model="logDateFrom" type="date" class="log-date-input" title="From date" />
            <input v-model="logDateTo" type="date" class="log-date-input" title="To date" />
            <button class="btn btn-ghost btn-xs" @click="showAuthorFilter = !showAuthorFilter" :class="{ active: selectedAuthors.size > 0 }" title="Filter by author">
              <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" /><circle cx="12" cy="7" r="4" />
              </svg>
              Authors
            </button>
            <div v-if="showAuthorFilter" class="author-filter-dropdown" @click.stop>
              <div v-for="a in logAuthors" :key="a" class="author-checkbox-item" @click.stop="toggleAuthor(a)">
                <input type="checkbox" :checked="selectedAuthors.has(a)" />
                <span>{{ getAuthorName(a) }}</span>
              </div>
              <div v-if="logAuthors.length === 0" class="author-filter-empty">No authors loaded</div>
            </div>
            <button class="btn btn-ghost btn-xs" @click="loadLog" :disabled="loading" title="刷新日志">
              <svg :class="{ spin: loading }" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="23 4 23 10 17 10" />
                <polyline points="1 20 1 14 7 14" />
                <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
              </svg>
            </button>
          </div>
        </div>

        <!-- 提交历史表格 -->
        <div v-if="logViewMode === 'table'" class="log-table-wrap">
          <table class="log-table">
            <thead>
              <tr>
                <th class="col-select"><input type="checkbox" :checked="selectedLogCommits.size > 0 && selectedLogCommits.size === filteredLog.length" @click="toggleSelectAllLogCommits" class="file-checkbox" title="Select all" /></th>
                <th class="col-hash">Hash</th>
                <th class="col-author">Author</th>
                <th class="col-date">Date</th>
                <th class="col-message">Message</th>
                <th class="col-files">Files</th>
                <th class="col-refs">Refs</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="commit in filteredLog"
                :key="commit.hash"
                class="log-row"
                :class="{ selected: selectedCommit?.hash === commit.hash, 'multi-selected': selectedLogCommits.has(commit.hash) }"
                @click="selectCommit(commit)"
                @contextmenu.prevent="showLogContextMenu($event, commit)"
              >
                <td class="col-select">
                  <input type="checkbox" :checked="selectedLogCommits.has(commit.hash)" @click.stop="toggleLogCommitSelect(commit.hash)" class="file-checkbox" />
                </td>
                <td class="col-hash">
                  <code class="hash-code">{{ commit.hash.substring(0, 7) }}</code>
                </td>
                <td class="col-author">
                  <span class="author-name">{{ getAuthorName(commit.author) }}</span>
                </td>
                <td class="col-date">
                  <span class="commit-date" :title="formatFullDate(commit.date)">{{ formatRelativeDate(commit.date) }}</span>
                </td>
                <td class="col-message">
                  <span class="commit-msg">{{ commit.message }}</span>
                </td>
                <td class="col-files">
                  <span v-if="commit.fileCount !== undefined" class="file-count-badge" :title="commit.fileCount + ' 个文件'">
                    {{ commit.fileCount }}
                  </span>
                </td>
                <td class="col-refs">
                  <span v-if="commit.refs" class="ref-tags">
                    <span
                      v-for="(ref, idx) in parseRefs(commit.refs)"
                      :key="idx"
                      class="ref-tag"
                      :class="ref.includes('HEAD') ? 'ref-head' : ref.includes('origin') ? 'ref-remote' : 'ref-local'"
                    >
                      {{ ref.replace('HEAD -> ', '') }}
                    </span>
                  </span>
                </td>
              </tr>
            </tbody>
          </table>
          <div v-if="filteredLog.length === 0 && !loading" class="empty-log">
            <p>没有提交记录</p>
          </div>
        </div>

        <!-- 加载更多 (table view only) -->
        <div v-if="logViewMode === 'table' && hasMoreLog" class="load-more-wrap">
          <button class="btn btn-ghost btn-sm" @click="loadMoreLog" :disabled="loading">
            加载更多 ({{ logCount }}/{{ logTotalEstimate }})
          </button>
        </div>

        <!-- ===== Git Graph 图形视图 ===== -->
        <div v-if="logViewMode === 'graph'" class="git-graph-wrap">
          <canvas
            ref="graphCanvasRef"
            class="git-graph-canvas"
            @mousemove="onGraphMouseMove"
            @click="onGraphClick"
          ></canvas>
          <div v-if="graphLoading" class="graph-loading">
            <svg class="spin" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="23 4 23 10 17 10" />
              <polyline points="1 20 1 14 7 14" />
              <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
            </svg>
            <span>加载中...</span>
          </div>
          <div v-if="!graphLoading && graphLog.length === 0" class="graph-empty">
            <p>没有提交记录</p>
          </div>
        </div>

        <!-- ===== Git Console 控制台 ===== -->
        <div v-if="logViewMode === 'console'" class="git-console-wrap">
          <div class="git-console-output" ref="consoleOutputRef">
            <div
              v-for="(line, idx) in consoleHistory"
              :key="idx"
              class="console-entry"
              :class="{ error: line.isError }"
            >
              <div class="console-cmd">
                <span class="console-prompt">λ</span>
                <span class="console-cmd-text">{{ line.command }}</span>
              </div>
              <pre v-if="line.output" class="console-output-text">{{ line.output }}</pre>
            </div>
            <div v-if="consoleHistory.length === 0" class="console-welcome">
              <p>Git Console — 输入任意 git 命令</p>
              <p class="console-hint">例如: <code>status</code>, <code>log --oneline -10</code>, <code>branch -vv</code></p>
            </div>
          </div>
          <div class="git-console-input-wrap">
            <span class="console-input-prompt">λ git</span>
            <input
              v-model="consoleInput"
              ref="consoleInputRef"
              class="git-console-input"
              placeholder="输入 git 命令参数..."
              @keydown.enter="execConsoleCommand"
              @keydown.up="consoleHistoryUp"
              @keydown.down="consoleHistoryDown"
              spellcheck="false"
              autocomplete="off"
            />
          </div>
        </div>

        <!-- 提交详情面板 -->
        <div v-if="selectedCommit" class="commit-detail">
          <div class="detail-header">
            <span class="detail-title">提交详情</span>
            <button class="btn btn-ghost btn-xs" @click="selectedCommit = null" title="关闭">✕</button>
          </div>
          <div class="detail-body">
            <div class="detail-row">
              <span class="detail-label">Hash</span>
              <code class="detail-value hash-full">{{ selectedCommit.hash }}</code>
            </div>
            <div class="detail-row">
              <span class="detail-label">Author</span>
              <span class="detail-value">{{ selectedCommit.author }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Date</span>
              <span class="detail-value">{{ formatFullDate(selectedCommit.date) }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Message</span>
              <span class="detail-value commit-msg-detail">{{ selectedCommit.message }}</span>
            </div>
            <div class="detail-row" v-if="selectedCommit.refs">
              <span class="detail-label">Refs</span>
              <span class="detail-value">{{ selectedCommit.refs }}</span>
            </div>
          </div>
          <div class="detail-diff">
            <div class="diff-header">
              <span class="diff-title">Diff</span>
              <button class="btn btn-ghost btn-xs" @click="loadCommitDiff" :disabled="loadingDiff">
                {{ loadingDiff ? '加载中...' : '查看 Diff' }}
              </button>
            </div>
            <pre v-if="commitDiff" class="diff-content">{{ commitDiff }}</pre>
            <div v-else-if="!loadingDiff" class="diff-empty">点击"查看 Diff"加载变更详情</div>
          </div>
        </div>
      </div>
    </div>

    <!-- ===== 右键菜单 ===== -->
    <div
      v-if="contextMenu.show"
      class="context-menu"
      :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
      @click.stop
    >
      <div class="context-menu-item" @click="contextMenuAction('diff')">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 20h9" /><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
        </svg>
        Diff
      </div>
      <div class="context-menu-item" @click="contextMenuAction('history')">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10" /><polyline points="12 6 12 12 16 14" />
        </svg>
        Show History
      </div>
      <div class="context-menu-item" @click="contextMenuAction('blame')">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" /><circle cx="12" cy="7" r="4" />
        </svg>
        Blame
      </div>
      <div class="context-menu-separator"></div>
      <div class="context-menu-item" @click="contextMenuAction('compareBranch')">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="20" x2="18" y2="10" /><line x1="12" y1="20" x2="12" y2="4" />
          <line x1="6" y1="20" x2="6" y2="14" />
        </svg>
        Compare with Branch...
      </div>
      <div class="context-menu-separator"></div>
      <div class="context-menu-item" @click="contextMenuAction('discard')" v-if="contextMenu.fileType !== 'untracked'">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="1 4 1 10 7 10" /><path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
        </svg>
        Discard Changes
      </div>
      <div class="context-menu-item" @click="contextMenuAction('add')" v-if="contextMenu.fileType === 'untracked'">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
        </svg>
        Add to VCS
      </div>
      <div class="context-menu-item" @click="contextMenuAction('reset')" v-if="contextMenu.fileType !== 'untracked'">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
        </svg>
        Remove from VCS
      </div>
      <div class="context-menu-separator"></div>
      <div class="context-menu-item" @click="contextMenuAction('gitignore')">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
          <polyline points="14 2 14 8 20 8" />
          <line x1="12" y1="18" x2="12" y2="12" />
          <line x1="9" y1="15" x2="15" y2="15" />
        </svg>
        Add to .gitignore
      </div>
    </div>

    <!-- ===== 日志行右键菜单 ===== -->
    <div
      v-if="logContextMenu.show"
      class="context-menu"
      :style="{ left: logContextMenu.x + 'px', top: logContextMenu.y + 'px' }"
      @click.stop
    >
      <div class="context-menu-item" @click="logContextAction('cherry-pick')">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="18" r="3" /><circle cx="12" cy="6" r="3" />
          <path d="m17 11-5 5-5-5" />
        </svg>
        Cherry-pick
      </div>
      <div class="context-menu-item" @click="logContextAction('revert')">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="1 4 1 10 7 10" /><path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
        </svg>
        Revert Commit
      </div>
      <div class="context-menu-separator"></div>
      <div class="context-menu-item" @click="logContextAction('create-tag')">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z" />
          <line x1="7" y1="7" x2="7.01" y2="7" />
        </svg>
        Create Tag...
      </div>
      <div class="context-menu-item" @click="logContextAction('compare-commits')">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="20" x2="18" y2="10" /><line x1="12" y1="20" x2="12" y2="4" />
          <line x1="6" y1="20" x2="6" y2="14" />
        </svg>
        Compare with Another Commit...
      </div>
      <div class="context-menu-item" @click="logContextAction('get-file')">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
          <polyline points="14 2 14 8 20 8" />
          <polyline points="12 18 12 12 16 12" />
        </svg>
        Get File at Revision...
      </div>
      <div class="context-menu-separator"></div>
      <div class="context-menu-item" @click="logContextAction('cherry-pick-multi')">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="18" r="3" /><circle cx="12" cy="6" r="3" />
          <path d="m17 11-5 5-5-5" />
          <circle cx="18" cy="18" r="3" /><circle cx="18" cy="6" r="3" />
        </svg>
        Cherry-pick Multiple...
      </div>
      <div class="context-menu-separator"></div>
      <div class="context-menu-item" @click="logContextAction('compare-with')">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="20" x2="18" y2="10" /><line x1="12" y1="20" x2="12" y2="4" />
          <line x1="6" y1="20" x2="6" y2="14" />
        </svg>
        Compare with...
      </div>
    </div>

    <!-- ===== Stash 右键菜单 ===== -->
    <div
      v-if="stashContextMenu.show"
      class="context-menu"
      :style="{ left: stashContextMenu.x + 'px', top: stashContextMenu.y + 'px' }"
      @click.stop
    >
      <div class="context-menu-item" @click="stashContextAction('apply')">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="15 10 20 15 15 20" /><path d="M4 4v7a4 4 0 0 0 4 4h12" />
        </svg>
        Apply
      </div>
      <div class="context-menu-item" @click="stashContextAction('pop')">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
          <polyline points="17 8 12 3 7 8" />
          <line x1="12" y1="3" x2="12" y2="15" />
        </svg>
        Pop
      </div>
      <div class="context-menu-separator"></div>
      <div class="context-menu-item context-menu-item-danger" @click="stashContextAction('drop')">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="3 6 5 6 21 6" /><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
        </svg>
        Drop
      </div>
    </div>

    <!-- ===== 分支管理弹窗 ===== -->
    <div v-if="showBranchesPopup" class="modal-overlay" @click="showBranchesPopup = false">
      <div class="branches-popup" @click.stop>
        <div class="popup-header">
          <span class="popup-title">分支管理</span>
          <button class="btn btn-ghost btn-xs" @click="showBranchesPopup = false">✕</button>
        </div>

        <div class="popup-actions">
          <button class="btn btn-primary btn-sm" @click="showCreateBranch = true">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
            </svg>
            新建分支
          </button>
        </div>

        <div class="branches-content">
          <div class="branch-section">
            <h4 class="section-label">本地分支</h4>
            <div class="branch-list">
              <div
                v-for="b in localBranches"
                :key="b.name"
                class="branch-item"
                :class="{ current: b.current }"
                @click="checkoutBranch(b.name)"
              >
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                  <line x1="6" y1="3" x2="6" y2="15" />
                  <circle cx="18" cy="6" r="3" />
                  <circle cx="6" cy="18" r="3" />
                  <path d="M18 9a9 9 0 0 1-9 9" />
                </svg>
                <span class="branch-label">{{ b.name }}</span>
                <span v-if="b.current" class="current-badge">当前</span>
                <div class="branch-actions" v-if="!b.current">
                  <button class="btn btn-ghost btn-xs" @click.stop="openBranchRename(b.name)" title="重命名分支">Rename</button>
                  <button class="btn btn-ghost btn-xs" @click.stop="showMergeDialog(b.name)" title="合并到此分支">Merge</button>
                  <button class="btn btn-ghost btn-xs btn-danger" @click.stop="confirmDeleteBranch(b.name)" title="删除分支">✕</button>
                </div>
              </div>
            </div>
          </div>

          <div class="branch-section">
            <h4 class="section-label">远程分支</h4>
            <div class="branch-list">
              <div
                v-for="b in remoteBranches"
                :key="b.name"
                class="branch-item remote"
              >
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="18" r="3" /><circle cx="12" cy="6" r="3" />
                  <line x1="12" y1="9" x2="12" y2="15" />
                </svg>
                <span class="branch-label">{{ b.name }}</span>
                <div class="branch-actions">
                  <button class="btn btn-ghost btn-xs" @click.stop="checkoutRemoteBranch(b.name)" title="Checkout as new local branch">Checkout</button>
                  <button class="btn btn-ghost btn-xs btn-danger" @click.stop="confirmDeleteRemoteBranch(b.name)" title="删除远程分支">✕</button>
                </div>
              </div>
              <div v-if="remoteBranches.length === 0" class="branch-empty">没有远程分支</div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- ===== 创建分支对话框 ===== -->
    <div v-if="showCreateBranch" class="modal-overlay" @click="showCreateBranch = false">
      <div class="create-branch-dialog" @click.stop>
        <h3 class="dialog-title">新建分支</h3>
        <div class="dialog-form">
          <label class="form-label">分支名称</label>
          <input
            v-model="newBranchName"
            class="form-input"
            placeholder="feature/xxx"
            @keydown.enter="doCreateBranch"
            spellcheck="false"
          />
          <label class="form-label">基于</label>
          <select v-model="newBranchFrom" class="form-input">
            <option value="">当前分支</option>
            <option v-for="b in localBranches" :key="b.name" :value="b.name">{{ b.name }}</option>
          </select>
        </div>
        <div class="dialog-actions">
          <button class="btn btn-ghost btn-sm" @click="showCreateBranch = false">取消</button>
          <button class="btn btn-primary btn-sm" @click="doCreateBranch" :disabled="!newBranchName.trim()">创建</button>
        </div>
      </div>
    </div>

    <!-- ===== 合并确认对话框 ===== -->
    <div v-if="mergeTarget" class="modal-overlay" @click="mergeTarget = null">
      <div class="merge-dialog" @click.stop>
        <h3 class="dialog-title">合并分支</h3>
        <p class="dialog-text">
          将分支 <code class="code-highlight">{{ mergeTarget }}</code> 合并到当前分支
          <code class="code-highlight">{{ currentBranch }}</code>？
        </p>
        <div class="dialog-actions">
          <button class="btn btn-ghost btn-sm" @click="mergeTarget = null">取消</button>
          <button class="btn btn-primary btn-sm" @click="doMerge" :disabled="merging">合并</button>
        </div>
      </div>
    </div>

    <!-- ===== Stash 保存对话框 ===== -->
    <div v-if="showStashSaveDialog" class="modal-overlay" @click="showStashSaveDialog = false">
      <div class="stash-save-dialog" @click.stop>
        <h3 class="dialog-title">保存 Stash</h3>
        <div class="dialog-form">
          <label class="form-label">描述</label>
          <input
            v-model="stashSaveMessage"
            class="form-input"
            placeholder="输入 stash 描述..."
            @keydown.enter="doStashSave"
            spellcheck="false"
            ref="stashSaveInput"
          />
          <label class="form-checkbox-label">
            <input type="checkbox" v-model="stashIncludeUntracked" />
            包含未跟踪文件
          </label>
        </div>
        <div class="dialog-actions">
          <button class="btn btn-ghost btn-sm" @click="showStashSaveDialog = false">取消</button>
          <button class="btn btn-primary btn-sm" @click="doStashSave" :disabled="!stashSaveMessage.trim()">保存</button>
        </div>
      </div>
    </div>

    <!-- ===== 文件历史对话框 ===== -->
    <div v-if="showFileHistoryDialog" class="modal-overlay" @click="showFileHistoryDialog = false">
      <div class="file-history-dialog" @click.stop>
        <div class="popup-header">
          <span class="popup-title">文件历史: {{ fileHistoryFile }}</span>
          <button class="btn btn-ghost btn-xs" @click="showFileHistoryDialog = false">✕</button>
        </div>
        <div class="file-history-list">
          <div v-for="commit in fileHistoryData" :key="commit.hash" class="file-history-item">
            <code class="hash-code">{{ commit.hash.substring(0, 7) }}</code>
            <span class="commit-date" :title="formatFullDate(commit.date)">{{ formatRelativeDate(commit.date) }}</span>
            <span class="commit-msg">{{ commit.message }}</span>
            <span class="author-name">{{ getAuthorName(commit.author) }}</span>
          </div>
          <div v-if="fileHistoryData.length === 0 && !loading" class="file-history-empty">没有历史记录</div>
        </div>
      </div>
    </div>

    <!-- ===== Blame 对话框 ===== -->
    <div v-if="showBlameDialog" class="modal-overlay" @click="showBlameDialog = false">
      <div class="blame-dialog" @click.stop>
        <div class="popup-header">
          <span class="popup-title">Blame: {{ blameFile }}</span>
          <button class="btn btn-ghost btn-xs" @click="showBlameDialog = false">✕</button>
        </div>
        <div class="blame-content">
          <pre>{{ blameData }}</pre>
        </div>
      </div>
    </div>

    <!-- ===== Cherry-pick 确认对话框 ===== -->
    <div v-if="cherryPickTarget" class="modal-overlay" @click="cherryPickTarget = null">
      <div class="cherry-pick-dialog" @click.stop>
        <h3 class="dialog-title">Cherry-pick</h3>
        <p class="dialog-text">
          将提交 <code class="code-highlight">{{ cherryPickTarget.substring(0, 7) }}</code> 应用到当前分支？
        </p>
        <div class="dialog-actions">
          <button class="btn btn-ghost btn-sm" @click="cherryPickTarget = null">取消</button>
          <button class="btn btn-primary btn-sm" @click="doCherryPick" :disabled="cherryPicking">Cherry-pick</button>
        </div>
      </div>
    </div>

    <!-- ===== Revert 确认对话框 ===== -->
    <div v-if="revertTarget" class="modal-overlay" @click="revertTarget = null">
      <div class="revert-dialog" @click.stop>
        <h3 class="dialog-title">Revert Commit</h3>
        <p class="dialog-text">
          创建一个新的提交来撤销 <code class="code-highlight">{{ revertTarget.substring(0, 7) }}</code> 的变更？
        </p>
        <div class="dialog-actions">
          <button class="btn btn-ghost btn-sm" @click="revertTarget = null">取消</button>
          <button class="btn btn-primary btn-sm" @click="doRevert" :disabled="reverting">Revert</button>
        </div>
      </div>
    </div>

    <!-- ===== Tag 创建对话框 ===== -->
    <div v-if="showCreateTagDialog" class="modal-overlay" @click="showCreateTagDialog = false">
      <div class="create-tag-dialog" @click.stop>
        <h3 class="dialog-title">创建 Tag</h3>
        <div class="dialog-form">
          <label class="form-label">Tag 名称</label>
          <input
            v-model="newTagName"
            class="form-input"
            placeholder="v1.0.0"
            @keydown.enter="doCreateTag"
            spellcheck="false"
          />
          <label class="form-label">指向提交</label>
          <input
            v-model="newTagCommit"
            class="form-input"
            placeholder="HEAD"
            spellcheck="false"
          />
          <label class="form-label">消息 (可选)</label>
          <input
            v-model="newTagMessage"
            class="form-input"
            placeholder="Tag 描述..."
            spellcheck="false"
          />
        </div>
        <div class="dialog-actions">
          <button class="btn btn-ghost btn-sm" @click="showCreateTagDialog = false">取消</button>
          <button class="btn btn-primary btn-sm" @click="doCreateTag" :disabled="!newTagName.trim()">创建</button>
        </div>
      </div>
    </div>

    <!-- ===== Tag 管理对话框 ===== -->
    <div v-if="showTagsDialog" class="modal-overlay" @click="showTagsDialog = false">
      <div class="tags-dialog" @click.stop>
        <div class="popup-header">
          <span class="popup-title">标签管理</span>
          <button class="btn btn-ghost btn-xs" @click="showTagsDialog = false">✕</button>
        </div>
        <div class="popup-actions">
          <button class="btn btn-primary btn-sm" @click="openCreateTag">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
            </svg>
            新建 Tag
          </button>
          <button class="btn btn-ghost btn-sm" @click="openCreateBranchFromTag" :disabled="!selectedTagForBranch">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="6" y1="3" x2="6" y2="15" /><circle cx="18" cy="6" r="3" /><circle cx="6" cy="18" r="3" />
              <path d="M18 9a9 9 0 0 1-9 9" />
            </svg>
            从 Tag 创建分支
          </button>
        </div>
        <div class="tags-content">
          <div
            v-for="tag in tagsList"
            :key="tag.name"
            class="tag-item"
            :class="{ selected: selectedTagForBranch === tag.name }"
            @click="selectedTagForBranch = tag.name"
          >
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z" />
              <line x1="7" y1="7" x2="7.01" y2="7" />
            </svg>
            <span class="tag-label">{{ tag.name }}</span>
            <span class="tag-commit" :title="tag.commit">{{ tag.commit?.substring(0, 7) || '' }}</span>
            <button class="btn btn-ghost btn-xs btn-danger" @click.stop="confirmDeleteTag(tag.name)" title="删除">✕</button>
          </div>
          <div v-if="tagsList.length === 0 && !loading" class="tags-empty">没有标签</div>
        </div>
      </div>
    </div>

    <!-- ===== Compare Branches 对话框 ===== -->
    <div v-if="showCompareBranchesDialog" class="modal-overlay" @click="showCompareBranchesDialog = false">
      <div class="compare-branches-dialog" @click.stop>
        <div class="popup-header">
          <span class="popup-title">比较分支</span>
          <button class="btn btn-ghost btn-xs" @click="showCompareBranchesDialog = false">✕</button>
        </div>
        <div class="dialog-form" style="padding: 12px 16px;">
          <div class="compare-row">
            <label class="form-label">当前分支</label>
            <span class="compare-branch-name">{{ currentBranch }}</span>
          </div>
          <div class="compare-row">
            <label class="form-label">对比分支</label>
            <select v-model="compareBranchTarget" class="form-input" @change="doCompareBranches">
              <option value="">选择分支...</option>
              <option v-for="b in localBranches" :key="b.name" :value="b.name" :disabled="b.name === currentBranch">{{ b.name }}</option>
            </select>
          </div>
        </div>
        <div v-if="compareResult" class="compare-result">
          <div class="compare-stat">
            <span class="stat-label">变更文件:</span>
            <span class="stat-value">{{ compareResult.changedFiles }}</span>
          </div>
          <div class="compare-stat">
            <span class="stat-label">新增行数:</span>
            <span class="stat-value insertions">{{ compareResult.insertions }}</span>
          </div>
          <div class="compare-stat">
            <span class="stat-label">删除行数:</span>
            <span class="stat-value deletions">{{ compareResult.deletions }}</span>
          </div>
        </div>
        <div class="dialog-actions" style="padding: 8px 16px 12px;">
          <button class="btn btn-ghost btn-sm" @click="showCompareBranchesDialog = false">关闭</button>
        </div>
      </div>
    </div>

    <!-- ===== Rebase 对话框 ===== -->
    <div v-if="showRebaseDialog" class="modal-overlay" @click="showRebaseDialog = false">
      <div class="rebase-dialog" @click.stop>
        <h3 class="dialog-title">Rebase</h3>
        <div v-if="rebaseInProgress" class="rebase-status">
          <p class="dialog-text">Rebase 进行中，存在冲突。请解决冲突后选择：</p>
          <div class="dialog-actions">
            <button class="btn btn-ghost btn-sm" @click="doRebaseAbort">中止 Rebase</button>
            <button class="btn btn-primary btn-sm" @click="doRebaseContinue">继续</button>
          </div>
        </div>
        <div v-else>
          <div class="dialog-form">
            <label class="form-label">Rebase 到</label>
            <select v-model="rebaseTarget" class="form-input">
              <option value="">选择分支...</option>
              <option v-for="b in localBranches" :key="b.name" :value="b.name" :disabled="b.name === currentBranch">{{ b.name }}</option>
            </select>
          </div>
          <div class="dialog-actions">
            <button class="btn btn-ghost btn-sm" @click="showRebaseDialog = false">取消</button>
            <button class="btn btn-primary btn-sm" @click="doRebase" :disabled="!rebaseTarget || rebasing">Rebase</button>
          </div>
        </div>
      </div>
    </div>

    <!-- ===== Compare with (commit) 对话框 ===== -->
    <div v-if="showCompareWithDialog" class="modal-overlay" @click="showCompareWithDialog = false">
      <div class="compare-with-dialog" @click.stop>
        <h3 class="dialog-title">比较提交</h3>
        <div class="dialog-form">
          <label class="form-label">对比分支</label>
          <select v-model="compareWithTarget" class="form-input">
            <option value="">选择分支...</option>
            <option v-for="b in localBranches" :key="b.name" :value="b.name">{{ b.name }}</option>
          </select>
        </div>
        <div class="dialog-actions">
          <button class="btn btn-ghost btn-sm" @click="showCompareWithDialog = false">取消</button>
          <button class="btn btn-primary btn-sm" @click="doCompareWith" :disabled="!compareWithTarget">比较</button>
        </div>
      </div>
    </div>

    <!-- ===== Push 对话框 ===== -->
    <div v-if="showPushDialog" class="modal-overlay" @click="showPushDialog = false">
      <div class="push-dialog" @click.stop>
        <div class="popup-header">
          <span class="popup-title">推送到远程</span>
          <button class="btn btn-ghost btn-xs" @click="showPushDialog = false">✕</button>
        </div>
        <div class="dialog-form" style="padding: 12px 16px;">
          <label class="form-label">远程仓库</label>
          <select v-model="pushRemote" class="form-input">
            <option v-for="r in remotesList" :key="r" :value="r">{{ r }}</option>
          </select>
          <label class="form-label">目标分支</label>
          <input v-model="pushBranch" class="form-input" placeholder="默认当前分支" />
          <label class="form-checkbox-label">
            <input type="checkbox" v-model="pushForce" />
            强制推送 (--force)
          </label>
          <label class="form-checkbox-label">
            <input type="checkbox" v-model="pushSetUpstream" />
            设置上游分支 (--set-upstream)
          </label>
        </div>
        <div v-if="pushUnpushedCommits.length > 0" class="push-commits-list">
          <div class="push-commits-header">待推送提交 ({{ pushUnpushedCommits.length }})</div>
          <div v-for="c in pushUnpushedCommits" :key="c.hash" class="push-commit-item">
            <code class="hash-code">{{ c.hash.substring(0, 7) }}</code>
            <span class="commit-msg">{{ c.message }}</span>
          </div>
        </div>
        <div class="dialog-actions" style="padding: 8px 16px 12px;">
          <button class="btn btn-ghost btn-sm" @click="showPushDialog = false">取消</button>
          <button class="btn btn-primary btn-sm" @click="doPushWithOptions" :disabled="pushing">
            <svg v-if="pushing" class="spin-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 12a9 9 0 1 1-6.219-8.56" />
            </svg>
            {{ pushForce ? 'Force Push' : 'Push' }}
          </button>
        </div>
      </div>
    </div>

    <!-- ===== Pull 对话框 ===== -->
    <div v-if="showPullDialog" class="modal-overlay" @click="showPullDialog = false">
      <div class="pull-dialog" @click.stop>
        <div class="popup-header">
          <span class="popup-title">从远程拉取</span>
          <button class="btn btn-ghost btn-xs" @click="showPullDialog = false">✕</button>
        </div>
        <div class="dialog-form" style="padding: 12px 16px;">
          <label class="form-label">远程仓库</label>
          <select v-model="pullRemote" class="form-input">
            <option v-for="r in remotesList" :key="r" :value="r">{{ r }}</option>
          </select>
          <label class="form-label">来源分支</label>
          <input v-model="pullBranch" class="form-input" placeholder="默认当前分支" />
          <label class="form-label">拉取方式</label>
          <select v-model="pullRebase" class="form-input">
            <option :value="false">Merge</option>
            <option :value="true">Rebase</option>
          </select>
          <label class="form-checkbox-label">
            <input type="checkbox" v-model="pullAutoStash" />
            自动 Stash 未提交的变更 (Auto-stash)
          </label>
        </div>
        <div class="dialog-actions" style="padding: 8px 16px 12px;">
          <button class="btn btn-ghost btn-sm" @click="showPullDialog = false">取消</button>
          <button class="btn btn-primary btn-sm" @click="doPullWithOptions" :disabled="pulling">
            <svg v-if="pulling" class="spin-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 12a9 9 0 1 1-6.219-8.56" />
            </svg>
            Pull
          </button>
        </div>
      </div>
    </div>

    <!-- ===== Amend 对话框 ===== -->
    <div v-if="showAmendDialog" class="modal-overlay" @click="showAmendDialog = false">
      <div class="amend-dialog" @click.stop>
        <h3 class="dialog-title">修改最后一次提交</h3>
        <div class="dialog-form">
          <label class="form-label">新的提交信息</label>
          <textarea
            v-model="amendMessage"
            class="commit-message-input"
            rows="4"
            placeholder="输入新的提交信息 (留空则使用原信息)"
            spellcheck="false"
          />
          <label class="form-checkbox-label">
            <input type="checkbox" v-model="amendNoEdit" />
            不修改提交信息 (仅添加变更)
          </label>
        </div>
        <div class="dialog-actions">
          <button class="btn btn-ghost btn-sm" @click="showAmendDialog = false">取消</button>
          <button class="btn btn-primary btn-sm" @click="doAmend" :disabled="amending">Amend</button>
        </div>
      </div>
    </div>

    <!-- ===== Reset 对话框 ===== -->
    <div v-if="showResetDialog" class="modal-overlay" @click="showResetDialog = false">
      <div class="reset-dialog" @click.stop>
        <h3 class="dialog-title">重置到提交</h3>
        <div class="dialog-form">
          <label class="form-label">提交 Hash / 引用</label>
          <input
            v-model="resetTarget"
            class="form-input"
            placeholder="HEAD~1, abc1234, etc."
            @keydown.enter="doReset"
            spellcheck="false"
          />
          <label class="form-label">重置模式</label>
          <select v-model="resetMode" class="form-input">
            <option value="soft">Soft — 保留变更到暂存区</option>
            <option value="mixed">Mixed — 保留变更到工作区 (默认)</option>
            <option value="hard">Hard — 丢弃所有变更</option>
          </select>
          <div v-if="resetMode === 'hard'" class="reset-warning">
            ⚠️ Hard Reset 将丢弃所有未提交的变更，此操作不可撤销！
          </div>
        </div>
        <div class="dialog-actions">
          <button class="btn btn-ghost btn-sm" @click="showResetDialog = false">取消</button>
          <button class="btn btn-primary btn-sm" @click="doReset" :disabled="!resetTarget.trim() || resetting">Reset</button>
        </div>
      </div>
    </div>

    <!-- ===== Interactive Rebase 对话框 ===== -->
    <div v-if="showInteractiveRebaseDialog" class="modal-overlay" @click="showInteractiveRebaseDialog = false">
      <div class="interactive-rebase-dialog" @click.stop>
        <div class="popup-header">
          <span class="popup-title">交互式 Rebase</span>
          <button class="btn btn-ghost btn-xs" @click="showInteractiveRebaseDialog = false">✕</button>
        </div>
        <div class="dialog-form" style="padding: 8px 16px;">
          <label class="form-label">Rebase 起点</label>
          <input
            v-model="interactiveRebaseBase"
            class="form-input"
            placeholder="HEAD~6, 提交hash, 分支名..."
            spellcheck="false"
          />
          <button class="btn btn-ghost btn-sm" @click="loadInteractiveRebaseCommits" :disabled="!interactiveRebaseBase.trim() || irLoading">
            加载提交列表
          </button>
        </div>
        <div v-if="irCommits.length > 0" class="ir-commits-list">
          <div
            v-for="(c, idx) in irCommits"
            :key="c.hash"
            class="ir-commit-item"
            :class="{ selected: irSelectedIndex === idx }"
            @click="irSelectedIndex = idx"
          >
            <select v-model="c.action" class="ir-action-select" @click.stop>
              <option value="pick">pick</option>
              <option value="reword">reword</option>
              <option value="edit">edit</option>
              <option value="squash">squash</option>
              <option value="fixup">fixup</option>
              <option value="drop">drop</option>
            </select>
            <code class="hash-code">{{ c.hash.substring(0, 7) }}</code>
            <span class="commit-msg">{{ c.message }}</span>
            <button class="btn btn-ghost btn-xs ir-move-btn" @click.stop="irMoveUp(idx)" :disabled="idx === 0" title="上移">↑</button>
            <button class="btn btn-ghost btn-xs ir-move-btn" @click.stop="irMoveDown(idx)" :disabled="idx === irCommits.length - 1" title="下移">↓</button>
          </div>
        </div>
        <div class="dialog-actions" style="padding: 8px 16px 12px;">
          <button class="btn btn-ghost btn-sm" @click="showInteractiveRebaseDialog = false">取消</button>
          <button class="btn btn-primary btn-sm" @click="doInteractiveRebase" :disabled="irCommits.length === 0 || irLoading">开始 Rebase</button>
        </div>
      </div>
    </div>

    <!-- ===== Remotes 对话框 ===== -->
    <div v-if="showRemotesDialog" class="modal-overlay" @click="showRemotesDialog = false">
      <div class="remotes-dialog" @click.stop>
        <div class="popup-header">
          <span class="popup-title">远程仓库管理</span>
          <button class="btn btn-ghost btn-xs" @click="showRemotesDialog = false">✕</button>
        </div>
        <div class="popup-actions">
          <button class="btn btn-primary btn-sm" @click="openAddRemote">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
            </svg>
            添加远程
          </button>
        </div>
        <div v-if="showAddRemoteForm" class="dialog-form" style="padding: 8px 16px;">
          <label class="form-label">名称</label>
          <input v-model="newRemoteName" class="form-input" placeholder="origin" spellcheck="false" />
          <label class="form-label">URL</label>
          <input v-model="newRemoteUrl" class="form-input" placeholder="git@github.com:user/repo.git" spellcheck="false" />
          <div class="dialog-actions">
            <button class="btn btn-ghost btn-sm" @click="showAddRemoteForm = false">取消</button>
            <button class="btn btn-primary btn-sm" @click="doAddRemote" :disabled="!newRemoteName.trim() || !newRemoteUrl.trim()">添加</button>
          </div>
        </div>
        <div class="remotes-content">
          <div v-for="r in remotesList" :key="r" class="remote-item">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10" /><line x1="2" y1="12" x2="22" y2="12" />
              <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
            </svg>
            <span class="remote-name">{{ r }}</span>
            <span class="remote-url" :title="remoteUrls[r]">{{ remoteUrls[r] }}</span>
            <button class="btn btn-ghost btn-xs" @click.stop="doFetchRemote(r)" title="Fetch">Fetch</button>
            <button class="btn btn-ghost btn-xs btn-danger" @click.stop="confirmDeleteRemote(r)" title="删除">✕</button>
          </div>
          <div v-if="remotesList.length === 0 && !loading" class="remotes-empty">没有远程仓库</div>
        </div>
      </div>
    </div>

    <!-- ===== 分支重命名对话框 ===== -->
    <div v-if="showBranchRenameDialog" class="modal-overlay" @click="showBranchRenameDialog = false">
      <div class="branch-rename-dialog" @click.stop>
        <h3 class="dialog-title">重命名分支</h3>
        <div class="dialog-form">
          <label class="form-label">当前名称</label>
          <input v-model="branchRenameOld" class="form-input" disabled />
          <label class="form-label">新名称</label>
          <input
            v-model="branchRenameNew"
            class="form-input"
            placeholder="新分支名称"
            @keydown.enter="doBranchRename"
            spellcheck="false"
            ref="branchRenameInput"
          />
        </div>
        <div class="dialog-actions">
          <button class="btn btn-ghost btn-sm" @click="showBranchRenameDialog = false">取消</button>
          <button class="btn btn-primary btn-sm" @click="doBranchRename" :disabled="!branchRenameNew.trim()">重命名</button>
        </div>
      </div>
    </div>

    <!-- ===== Submodule 管理对话框 ===== -->
    <div v-if="showSubmodulesDialog" class="modal-overlay" @click="showSubmodulesDialog = false">
      <div class="submodules-dialog" @click.stop>
        <div class="popup-header">
          <span class="popup-title">Submodules</span>
          <button class="btn btn-ghost btn-xs" @click="showSubmodulesDialog = false">✕</button>
        </div>
        <div class="popup-actions">
          <button class="btn btn-primary btn-sm" @click="doSubmoduleInitAll" :disabled="smLoading">Init All</button>
          <button class="btn btn-ghost btn-sm" @click="doSubmoduleUpdateAll" :disabled="smLoading">Update All</button>
          <button class="btn btn-ghost btn-sm" @click="loadSubmodules" :disabled="smLoading">Refresh</button>
        </div>
        <div class="submodules-content">
          <div
            v-for="sm in submodulesList"
            :key="sm.name"
            class="submodule-item"
            :class="{ 'sm-initialized': sm.initialized }"
          >
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="2" y="2" width="9" height="9" rx="1" /><rect x="13" y="2" width="9" height="9" rx="1" />
              <rect x="2" y="13" width="9" height="9" rx="1" /><rect x="13" y="13" width="9" height="9" rx="1" />
            </svg>
            <span class="sm-name" :title="sm.name">{{ sm.name }}</span>
            <span class="sm-path" @click="openSubmodulePath(sm.path)" :title="sm.path">{{ sm.path }}</span>
            <span class="sm-hash" :title="sm.hash">{{ sm.hash ? sm.hash.substring(0, 7) : '-' }}</span>
            <span class="sm-status" :class="sm.initialized ? 'status-ok' : 'status-warn'">
              {{ sm.initialized ? 'Initialized' : 'Not initialized' }}
            </span>
            <div class="sm-actions">
              <button v-if="!sm.initialized" class="btn btn-ghost btn-xs" @click.stop="doSubmoduleInit(sm.name)">Init</button>
              <button v-if="sm.initialized" class="btn btn-ghost btn-xs" @click.stop="doSubmoduleUpdate(sm.name)">Update</button>
            </div>
          </div>
          <div v-if="submodulesList.length === 0 && !smLoading" class="submodules-empty">No submodules defined in .gitmodules</div>
          <div v-if="smLoading" class="submodules-loading">Loading...</div>
        </div>
        <div class="dialog-actions" style="padding: 8px 16px 12px;">
          <button class="btn btn-ghost btn-sm" @click="showSubmodulesDialog = false">关闭</button>
        </div>
      </div>
    </div>

    <!-- ===== Compare Two Commits 对话框 ===== -->
    <div v-if="showCompareCommitsDialog" class="modal-overlay" @click="showCompareCommitsDialog = false">
      <div class="compare-commits-dialog" @click.stop>
        <div class="popup-header">
          <span class="popup-title">比较两个提交</span>
          <button class="btn btn-ghost btn-xs" @click="showCompareCommitsDialog = false">✕</button>
        </div>
        <div class="dialog-form" style="padding: 12px 16px;">
          <label class="form-label">From Commit</label>
          <input v-model="compareCommitFrom" class="form-input" placeholder="Commit hash or ref (e.g., HEAD~5)" spellcheck="false" />
          <label class="form-label">To Commit</label>
          <input v-model="compareCommitTo" class="form-input" placeholder="Commit hash or ref (e.g., HEAD)" spellcheck="false" />
          <button class="btn btn-ghost btn-sm" @click="doCompareCommits" :disabled="!compareCommitFrom || !compareCommitTo || ccLoading">比较</button>
        </div>
        <div v-if="compareCommitsDiff" class="compare-commits-result">
          <pre class="diff-content">{{ compareCommitsDiff }}</pre>
        </div>
        <div class="dialog-actions" style="padding: 8px 16px 12px;">
          <button class="btn btn-ghost btn-sm" @click="showCompareCommitsDialog = false">关闭</button>
        </div>
      </div>
    </div>

    <!-- ===== Get File at Revision 对话框 ===== -->
    <div v-if="showGetFileRevisionDialog" class="modal-overlay" @click="showGetFileRevisionDialog = false">
      <div class="get-file-revision-dialog" @click.stop>
        <div class="popup-header">
          <span class="popup-title">获取指定版本的文件</span>
          <button class="btn btn-ghost btn-xs" @click="showGetFileRevisionDialog = false">✕</button>
        </div>
        <div class="dialog-form" style="padding: 12px 16px;">
          <label class="form-label">Commit</label>
          <input v-model="getFileCommit" class="form-input" placeholder="Commit hash or ref" spellcheck="false" />
          <label class="form-label">File Path</label>
          <input v-model="getFilePath" class="form-input" placeholder="path/to/file.txt" spellcheck="false" />
        </div>
        <div class="dialog-actions" style="padding: 8px 16px;">
          <button class="btn btn-ghost btn-sm" @click="showGetFileRevisionDialog = false">取消</button>
          <button class="btn btn-primary btn-sm" @click="doGetFileAtRevision" :disabled="!getFileCommit || !getFilePath">预览文件</button>
        </div>
      </div>
    </div>

    <!-- ===== File Preview 对话框 ===== -->
    <div v-if="showGetFilePreviewDialog" class="modal-overlay" @click="showGetFilePreviewDialog = false">
      <div class="file-preview-dialog" @click.stop>
        <div class="popup-header">
          <span class="popup-title">文件预览: {{ getFilePath }}</span>
          <div style="display:flex;gap:4px;">
            <button class="btn btn-ghost btn-xs" @click="copyFileContent" title="复制">📋 复制</button>
            <button class="btn btn-ghost btn-xs" @click="showGetFilePreviewDialog = false">✕</button>
          </div>
        </div>
        <pre class="file-preview-content">{{ getFileContent }}</pre>
      </div>
    </div>

    <!-- ===== Create Patch 对话框 ===== -->
    <div v-if="showCreatePatchDialog" class="modal-overlay" @click="showCreatePatchDialog = false">
      <div class="create-patch-dialog" @click.stop>
        <div class="popup-header">
          <span class="popup-title">创建 Patch</span>
          <button class="btn btn-ghost btn-xs" @click="showCreatePatchDialog = false">✕</button>
        </div>
        <div class="dialog-form" style="padding: 12px 16px;">
          <label class="form-label">From (exclusive)</label>
          <input v-model="patchFrom" class="form-input" placeholder="HEAD~5 or commit hash" spellcheck="false" />
          <label class="form-label">To (inclusive)</label>
          <input v-model="patchTo" class="form-input" placeholder="HEAD or commit hash" spellcheck="false" />
          <label class="form-label">Output Directory</label>
          <input v-model="patchOutputDir" class="form-input" placeholder="Leave empty for repo root" spellcheck="false" />
        </div>
        <div class="dialog-actions" style="padding: 8px 16px 12px;">
          <button class="btn btn-ghost btn-sm" @click="showCreatePatchDialog = false">取消</button>
          <button class="btn btn-primary btn-sm" @click="doCreatePatch" :disabled="!patchFrom || !patchTo">创建 Patch</button>
        </div>
      </div>
    </div>

    <!-- ===== Apply Patch 对话框 ===== -->
    <div v-if="showApplyPatchDialog" class="modal-overlay" @click="showApplyPatchDialog = false">
      <div class="apply-patch-dialog" @click.stop>
        <div class="popup-header">
          <span class="popup-title">应用 Patch</span>
          <button class="btn btn-ghost btn-xs" @click="showApplyPatchDialog = false">✕</button>
        </div>
        <div class="dialog-form" style="padding: 12px 16px;">
          <label class="form-label">Patch File</label>
          <div class="file-input-wrap">
            <input v-model="applyPatchFile" class="form-input" placeholder="选择或输入 patch 文件路径" spellcheck="false" />
            <button class="btn btn-ghost btn-xs" @click="selectPatchFile" title="选择文件">📁</button>
          </div>
          <label class="form-checkbox-label">
            <input type="checkbox" v-model="applyPatchCheck" />
            先检查 (--check, 不实际应用)
          </label>
          <label class="form-checkbox-label">
            <input type="checkbox" v-model="applyPatchSign" />
            添加 Signed-off-by (--signoff)
          </label>
          <label class="form-checkbox-label">
            <input type="checkbox" v-model="applyPatch3way" />
            三路合并 (--3way)
          </label>
        </div>
        <div v-if="applyPatchResult" class="patch-result" :class="{ 'result-error': applyPatchError }">
          <pre>{{ applyPatchResult }}</pre>
        </div>
        <div class="dialog-actions" style="padding: 8px 16px 12px;">
          <button class="btn btn-ghost btn-sm" @click="showApplyPatchDialog = false">关闭</button>
          <button class="btn btn-primary btn-sm" @click="doApplyPatch" :disabled="!applyPatchFile">应用</button>
        </div>
      </div>
    </div>

    <!-- ===== Cherry-pick Multiple 对话框 ===== -->
    <div v-if="showCherryPickMultiDialog" class="modal-overlay" @click="showCherryPickMultiDialog = false">
      <div class="cherry-pick-multi-dialog" @click.stop>
        <div class="popup-header">
          <span class="popup-title">Cherry-pick 多个提交 ({{ selectedLogCommits.size }})</span>
          <button class="btn btn-ghost btn-xs" @click="showCherryPickMultiDialog = false">✕</button>
        </div>
        <div class="cherry-pick-multi-list">
          <div v-for="hash in Array.from(selectedLogCommits)" :key="hash" class="cp-multi-item">
            <code class="hash-code">{{ hash.substring(0, 7) }}</code>
            <span class="commit-msg">{{ getCommitMessage(hash) }}</span>
          </div>
        </div>
        <label class="form-checkbox-label" style="padding: 8px 16px;">
          <input type="checkbox" v-model="cherryPickMultiNoCommit" />
          不自动提交 (--no-commit)
        </label>
        <div class="dialog-actions" style="padding: 8px 16px 12px;">
          <button class="btn btn-ghost btn-sm" @click="showCherryPickMultiDialog = false">取消</button>
          <button class="btn btn-primary btn-sm" @click="doCherryPickMulti" :disabled="selectedLogCommits.size === 0 || cherryPicking">
            Cherry-pick All
          </button>
        </div>
      </div>
    </div>

    <!-- ===== Git Clean 对话框 ===== -->
    <div v-if="showGitCleanDialog" class="modal-overlay" @click="showGitCleanDialog = false">
      <div class="git-clean-dialog" @click.stop>
        <div class="popup-header">
          <span class="popup-title">Clean Working Tree</span>
          <button class="btn btn-ghost btn-xs" @click="showGitCleanDialog = false">✕</button>
        </div>
        <div class="dialog-form" style="padding: 8px 16px;">
          <label class="form-checkbox-label">
            <input type="checkbox" v-model="gitCleanIncludeIgnored" />
            包含忽略的文件 (-x)
          </label>
          <label class="form-checkbox-label">
            <input type="checkbox" v-model="gitCleanForceDirectories" />
            删除目录 (-d)
          </label>
          <button class="btn btn-ghost btn-sm" @click="doGitCleanDryRun" :disabled="gcLoading">Dry Run (预览)</button>
        </div>
        <div v-if="gitCleanFiles.length > 0" class="git-clean-list">
          <div class="git-clean-header">将删除以下 {{ gitCleanFiles.length }} 个文件/目录:</div>
          <div v-for="f in gitCleanFiles" :key="f" class="git-clean-item">
            <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="3 6 5 6 21 6" /><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
            </svg>
            <span>{{ f }}</span>
          </div>
        </div>
        <div v-if="gitCleanFiles.length === 0 && !gcLoading" class="git-clean-empty">没有未跟踪的文件需要清理</div>
        <div class="dialog-actions" style="padding: 8px 16px 12px;">
          <button class="btn btn-ghost btn-sm" @click="showGitCleanDialog = false">取消</button>
          <button class="btn btn-danger btn-sm" @click="doGitClean" :disabled="gitCleanFiles.length === 0 || gcLoading">
            确认清理
          </button>
        </div>
      </div>
    </div>

    <!-- ===== Delete Remote Branch 确认对话框 ===== -->
    <div v-if="deleteRemoteBranchTarget" class="modal-overlay" @click="deleteRemoteBranchTarget = null">
      <div class="delete-remote-branch-dialog" @click.stop>
        <h3 class="dialog-title">删除远程分支</h3>
        <p class="dialog-text">
          确定要删除远程分支 <code class="code-highlight">{{ deleteRemoteBranchTarget }}</code>？此操作不可撤销！
        </p>
        <div class="dialog-actions">
          <button class="btn btn-ghost btn-sm" @click="deleteRemoteBranchTarget = null">取消</button>
          <button class="btn btn-danger btn-sm" @click="doDeleteRemoteBranch" :disabled="deletingBranch">删除</button>
        </div>
      </div>
    </div>

  </div>
</template>
<script setup lang="ts">
// @ts-nocheck
import type { GitRepo } from '../types'
import { useGitManager } from '../composables/useGitManager'

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
  logBranchFilter,
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
  branchRenameInput,
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
  showStashMenu,
  showGitMenu,
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
} = useGitManager(props.repo, () => emit("close"))
</script>
<style scoped>
/* ===================== 下拉菜单 ===================== */
.dropdown-wrap {
  position: relative;
}

.dropdown-menu {
  position: absolute;
  top: 100%;
  right: 0;
  z-index: 1000;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 6px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);
  min-width: 200px;
  padding: 4px 0;
  margin-top: 4px;
}

.dropdown-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  cursor: pointer;
  font-size: 13px;
  transition: background 0.1s;
  color: oklch(var(--bc));
}

.dropdown-menu-item:hover {
  background: var(--hover-bg);
}

.dropdown-menu-item svg {
  flex-shrink: 0;
  color: oklch(var(--bc) / 0.6);
}

.dropdown-menu-separator {
  height: 1px;
  background: oklch(var(--bc) / 0.1);
  margin: 4px 0;
}

/* ===================== 右键菜单分隔线 ===================== */
.context-menu-separator {
  height: 1px;
  background: oklch(var(--bc) / 0.1);
  margin: 4px 0;
}

.context-menu-item-danger:hover {
  color: #ef4444;
  background: rgba(239, 68, 68, 0.1);
}

/* ===================== Stash 面板 ===================== */
.stash-panel {
  display: flex;
  flex-direction: column;
  border-top: 1px solid oklch(var(--bc) / 0.1);
  max-height: 40%;
  flex-shrink: 0;
  background: oklch(var(--b1));
}

.stash-list {
  flex: 1;
  overflow-y: auto;
  max-height: 180px;
}

.stash-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  cursor: pointer;
  transition: background 0.1s;
  font-size: 12px;
}

.stash-item:hover {
  background: var(--hover-bg);
}

.stash-item.selected {
  background: oklch(var(--p) / 0.1);
}

.stash-item svg {
  flex-shrink: 0;
  color: oklch(var(--bc) / 0.6);
}

.stash-name {
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  font-size: 11px;
  color: oklch(var(--p));
  background: oklch(var(--p) / 0.1);
  padding: 1px 4px;
  border-radius: 2px;
  flex-shrink: 0;
}

.stash-desc {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: oklch(var(--bc) / 0.6);
}

.stash-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  color: oklch(var(--bc) / 0.6);
  font-size: 12px;
}

.stash-preview {
  border-top: 1px solid oklch(var(--bc) / 0.1);
  max-height: 200px;
  overflow-y: auto;
}

/* ===================== Stash 保存对话框 ===================== */
.stash-save-dialog {
  width: 500px;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  padding: 20px;
}

.form-checkbox-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: oklch(var(--bc));
  cursor: pointer;
}

.form-checkbox-label input[type="checkbox"] {
  accent-color: oklch(var(--p));
}

/* ===================== 文件历史对话框 ===================== */
.file-history-dialog {
  width: 600px;
  max-height: 70vh;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.file-history-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.file-history-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: 4px;
  font-size: 12px;
  transition: background 0.1s;
}

.file-history-item:hover {
  background: var(--hover-bg);
}

.file-history-item .commit-msg {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-history-empty {
  padding: 20px;
  text-align: center;
  color: oklch(var(--bc) / 0.6);
}

/* ===================== Blame 对话框 ===================== */
.blame-dialog {
  width: 700px;
  max-height: 80vh;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.blame-content {
  flex: 1;
  overflow: auto;
  padding: 10px;
}

.blame-content pre {
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  font-size: 11px;
  line-height: 1.6;
  white-space: pre;
  color: oklch(var(--bc));
}

/* ===================== Cherry-pick / Revert 对话框 ===================== */
.cherry-pick-dialog,
.revert-dialog {
  width: 480px;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  padding: 20px;
}

/* ===================== Tag 创建对话框 ===================== */
.create-tag-dialog {
  width: 560px;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  padding: 20px;
}

/* ===================== Tag 管理对话框 ===================== */
.tags-dialog {
  width: 480px;
  max-height: 70vh;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.tags-content {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.tag-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 8px;
  border-radius: 4px;
  font-size: 13px;
  transition: background 0.1s;
}

.tag-item:hover {
  background: var(--hover-bg);
}

.tag-item svg {
  flex-shrink: 0;
  color: oklch(var(--bc) / 0.6);
}

.tag-label {
  flex: 1;
  font-family: 'JetBrains Mono', 'Consolas', monospace;
}

.tag-commit {
  font-size: 11px;
  color: oklch(var(--bc) / 0.6);
  font-family: 'JetBrains Mono', 'Consolas', monospace;
}

.tags-empty {
  padding: 20px;
  text-align: center;
  color: oklch(var(--bc) / 0.6);
}

/* ===================== Compare Branches 对话框 ===================== */
.compare-branches-dialog {
  width: 420px;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.compare-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.compare-row .form-label {
  min-width: 80px;
  margin-bottom: 0;
}

.compare-branch-name {
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  color: oklch(var(--p));
  font-weight: 500;
}

.compare-result {
  padding: 12px 16px;
  border-top: 1px solid oklch(var(--bc) / 0.1);
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
}

.compare-stat {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 0;
  font-size: 13px;
}

.stat-label {
  color: oklch(var(--bc) / 0.6);
  min-width: 80px;
}

.stat-value {
  font-weight: 600;
}

.stat-value.insertions {
  color: #22c55e;
}

.stat-value.deletions {
  color: #ef4444;
}

/* ===================== Rebase 对话框 ===================== */
.rebase-dialog {
  width: 480px;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  padding: 20px;
}

.rebase-status {
  text-align: center;
}

.rebase-status .dialog-text {
  margin-bottom: 16px;
}

.rebase-status .dialog-actions {
  justify-content: center;
}

/* ===================== Compare with 对话框 ===================== */
.compare-with-dialog {
  width: 480px;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  padding: 20px;
}

/* ===================== 日志文件数 ===================== */
.col-files {
  width: 50px;
  text-align: center;
}

.file-count-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 20px;
  height: 18px;
  padding: 0 4px;
  border-radius: 9px;
  background: oklch(var(--p) / 0.1);
  color: oklch(var(--p));
  font-size: 10px;
  font-weight: 600;
  font-family: 'JetBrains Mono', 'Consolas', monospace;
}

/* ===================== 基础布局 ===================== */
.git-manager {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  font-size: 13px;
}

/* ===================== 顶部工具栏 ===================== */
.git-topbar {
  display: flex;
  align-items: center;
  padding: 6px 12px;
  gap: 12px;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
  background: oklch(var(--b1));
  min-height: 40px;
  flex-shrink: 0;
}

.topbar-left {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 200px;
}

.back-btn {
  flex-shrink: 0;
  padding: 4px 6px;
  border-radius: 4px;
}

.repo-info {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.repo-name {
  font-weight: 600;
  font-size: 14px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.repo-path {
  font-size: 11px;
  color: oklch(var(--bc) / 0.6);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 480px;
}

.topbar-center {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  justify-content: center;
}

.branch-selector {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 4px;
  background: oklch(var(--b2));
  cursor: pointer;
  transition: border-color 0.15s ease;
}

.branch-selector:hover {
  border-color: oklch(var(--p));
}

.branch-name {
  font-weight: 500;
  color: oklch(var(--p));
}

.dropdown-arrow {
  color: oklch(var(--bc) / 0.6);
}

.ahead-behind {
  display: flex;
  gap: 6px;
  font-size: 12px;
}

.ahead {
  color: #f59e0b;
  font-weight: 500;
}

.behind {
  color: #3b82f6;
  font-weight: 500;
}

.topbar-right {
  display: flex;
  align-items: center;
  gap: 4px;
}

/* ===================== 主内容区 ===================== */
.git-main {
  display: flex;
  flex: 1;
  overflow: hidden;
}

/* ===================== 左侧：提交面板 ===================== */
.commit-panel {
  display: flex;
  flex-direction: column;
  border-right: 1px solid oklch(var(--bc) / 0.1);
  min-width: 200px;
  flex-shrink: 0;
}

/* ===================== 分割条 ===================== */
.splitter {
  width: 4px;
  background: oklch(var(--bc) / 0.1);
  cursor: col-resize;
  transition: background 0.15s;
  flex-shrink: 0;
}

.splitter:hover {
  background: oklch(var(--p));
}

/* ===================== 面板头部 ===================== */
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 10px;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
  background: oklch(var(--b1));
  flex-shrink: 0;
}

.panel-title {
  font-weight: 600;
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.change-count {
  background: oklch(var(--p) / 0.1);
  color: oklch(var(--p));
  padding: 1px 6px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 500;
}

/* ===================== 文件列表 ===================== */
.file-list {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

.file-group {
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
}

.group-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  cursor: pointer;
  background: oklch(var(--b1));
  user-select: none;
  transition: background 0.1s;
}

.group-header:hover {
  background: var(--hover-bg);
}

.group-arrow {
  transition: transform 0.15s;
  color: oklch(var(--bc) / 0.6);
}

.group-arrow.collapsed {
  transform: rotate(-90deg);
}

.group-icon {
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 3px;
  font-size: 10px;
  font-weight: 700;
  font-family: 'JetBrains Mono', 'Consolas', monospace;
}

.group-icon.modified {
  background: rgba(245, 158, 11, 0.15);
  color: #f59e0b;
}

.group-icon.added {
  background: rgba(34, 197, 94, 0.15);
  color: #22c55e;
}

.group-icon.deleted {
  background: rgba(239, 68, 68, 0.15);
  color: #ef4444;
}

.group-icon.untracked {
  background: rgba(156, 163, 175, 0.15);
  color: #9ca3af;
}

.group-icon.conflicted {
  background: rgba(168, 85, 247, 0.15);
  color: #a855f7;
}

.group-label {
  font-weight: 500;
  font-size: 12px;
}

.group-count {
  margin-left: auto;
  font-size: 11px;
  color: oklch(var(--bc) / 0.6);
}

.group-files {
  padding-left: 12px;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 3px 10px;
  cursor: pointer;
  transition: background 0.1s;
  user-select: none;
}

.file-item:hover {
  background: var(--hover-bg);
}

.file-item.selected {
  background: oklch(var(--p) / 0.1);
}

.file-item.conflicted-item {
  background: rgba(168, 85, 247, 0.05);
}

.file-checkbox {
  flex-shrink: 0;
  width: 14px;
  height: 14px;
  accent-color: oklch(var(--p));
  cursor: pointer;
}

.file-icon {
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 2px;
  font-size: 9px;
  font-weight: 700;
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  flex-shrink: 0;
}

.file-icon.modified {
  background: rgba(245, 158, 11, 0.15);
  color: #f59e0b;
}

.file-icon.added {
  background: rgba(34, 197, 94, 0.15);
  color: #22c55e;
}

.file-icon.deleted {
  background: rgba(239, 68, 68, 0.15);
  color: #ef4444;
}

.file-icon.untracked {
  background: rgba(156, 163, 175, 0.15);
  color: #9ca3af;
}

.file-icon.conflicted {
  background: rgba(168, 85, 247, 0.15);
  color: #a855f7;
}

.file-name {
  font-size: 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: 'JetBrains Mono', 'Consolas', monospace;
}

.empty-files {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  color: oklch(var(--bc) / 0.6);
  gap: 8px;
  padding: 24px;
}

.empty-files svg {
  color: #22c55e;
}

.empty-files p {
  font-size: 13px;
}

/* ===================== 提交区域 ===================== */
.commit-area {
  display: flex;
  flex-direction: column;
  border-top: 1px solid oklch(var(--bc) / 0.1);
  background: oklch(var(--b1));
  flex-shrink: 0;
}

.commit-header {
  padding: 6px 10px;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
}

.commit-title {
  font-weight: 600;
  font-size: 12px;
}

.commit-message-input {
  width: 100%;
  padding: 8px 10px;
  border: none;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  font-size: 13px;
  font-family: inherit;
  resize: vertical;
  outline: none;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
}

.commit-message-input:focus {
  background: oklch(var(--b2));
}

.commit-message-input::placeholder {
  color: oklch(var(--bc) / 0.6);
  opacity: 0.5;
}

.commit-actions {
  display: flex;
  gap: 6px;
  padding: 8px 10px;
}

/* ===================== 右侧：日志面板 ===================== */
.log-panel {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 300px;
  overflow: hidden;
}

.log-filters {
  display: flex;
  align-items: center;
  gap: 6px;
}

.search-input-wrap {
  position: relative;
}

.search-icon {
  position: absolute;
  left: 8px;
  top: 50%;
  transform: translateY(-50%);
  color: oklch(var(--bc) / 0.6);
  pointer-events: none;
}

.log-search-input {
  padding: 3px 8px 3px 26px;
  border: 1px solid oklch(var(--bc) / 0.2);
  border-radius: 4px;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  font-size: 12px;
  outline: none;
  width: 150px;
  transition: border-color 0.15s;
}

.log-search-input:focus {
  border-color: oklch(var(--p));
  width: 200px;
}

.log-branch-filter {
  padding: 3px 6px;
  border: 1px solid oklch(var(--bc) / 0.2);
  border-radius: 4px;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  font-size: 12px;
  outline: none;
  cursor: pointer;
}

/* ===================== 日志表格 ===================== */
.log-table-wrap {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

.log-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
}

.log-table thead {
  position: sticky;
  top: 0;
  z-index: 1;
}

.log-table th {
  padding: 5px 10px;
  text-align: left;
  font-weight: 600;
  color: oklch(var(--bc) / 0.6);
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
  background: oklch(var(--b1));
  white-space: nowrap;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.col-hash { width: 70px; }
.col-author { width: 120px; }
.col-date { width: 100px; }
.col-message { min-width: 200px; }
.col-refs { width: 120px; }

.log-row {
  cursor: pointer;
  transition: background 0.1s;
}

.log-row:hover {
  background: var(--hover-bg);
}

.log-row.selected {
  background: oklch(var(--p) / 0.1);
}

.log-row td {
  padding: 4px 10px;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
  vertical-align: middle;
}

.hash-code {
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  font-size: 11px;
  color: oklch(var(--p));
  background: oklch(var(--p) / 0.1);
  padding: 1px 4px;
  border-radius: 2px;
}

.author-name {
  color: oklch(var(--bc));
}

.commit-date {
  color: oklch(var(--bc) / 0.6);
  font-size: 11px;
}

.commit-msg {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: block;
  max-width: 560px;
}

.ref-tags {
  display: flex;
  gap: 3px;
  flex-wrap: wrap;
}

.ref-tag {
  font-size: 10px;
  padding: 1px 5px;
  border-radius: 3px;
  white-space: nowrap;
}

.ref-head {
  background: rgba(34, 197, 94, 0.2);
  color: #22c55e;
}

.ref-local {
  background: rgba(59, 130, 246, 0.2);
  color: #3b82f6;
}

.ref-remote {
  background: rgba(168, 85, 247, 0.2);
  color: #a855f7;
}

.empty-log {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100px;
  color: oklch(var(--bc) / 0.6);
}

.load-more-wrap {
  display: flex;
  justify-content: center;
  padding: 6px;
  border-top: 1px solid oklch(var(--bc) / 0.1);
  background: oklch(var(--b1));
}

/* ===================== 提交详情 ===================== */
.commit-detail {
  border-top: 1px solid oklch(var(--bc) / 0.1);
  background: oklch(var(--b1));
  max-height: 40%;
  overflow-y: auto;
  flex-shrink: 0;
}

.detail-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 10px;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
}

.detail-title {
  font-weight: 600;
  font-size: 12px;
}

.detail-body {
  padding: 8px 10px;
}

.detail-row {
  display: flex;
  gap: 8px;
  margin-bottom: 4px;
  font-size: 12px;
}

.detail-label {
  font-weight: 600;
  color: oklch(var(--bc) / 0.6);
  min-width: 50px;
  flex-shrink: 0;
}

.detail-value {
  color: oklch(var(--bc));
}

.hash-full {
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  font-size: 11px;
}

.commit-msg-detail {
  display: block;
}

.detail-diff {
  border-top: 1px solid oklch(var(--bc) / 0.1);
  padding: 8px 10px;
}

.diff-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.diff-title {
  font-weight: 600;
  font-size: 12px;
}

.diff-content {
  background: oklch(var(--b2));
  padding: 10px;
  border-radius: 4px;
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  font-size: 11px;
  line-height: 1.5;
  overflow-x: auto;
  max-height: 300px;
  white-space: pre-wrap;
  color: oklch(var(--bc));
}

.diff-empty {
  text-align: center;
  color: oklch(var(--bc) / 0.6);
  font-size: 12px;
  padding: 16px;
}

/* ===================== 右键菜单 ===================== */
.context-menu {
  position: fixed;
  z-index: 1000;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 6px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);
  min-width: 180px;
  padding: 4px 0;
}

.context-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  cursor: pointer;
  font-size: 12px;
  transition: background 0.1s;
}

.context-menu-item:hover {
  background: var(--hover-bg);
}

.context-menu-item svg {
  flex-shrink: 0;
  color: oklch(var(--bc) / 0.6);
}

/* ===================== 弹窗 ===================== */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  z-index: 900;
  display: flex;
  align-items: center;
  justify-content: center;
}

.branches-popup {
  width: 420px;
  max-height: 70vh;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.popup-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
}

.popup-title {
  font-weight: 600;
  font-size: 14px;
}

.popup-actions {
  padding: 8px 16px;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
}

.branches-content {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.branch-section {
  padding: 4px 0;
}

.section-label {
  font-size: 11px;
  font-weight: 600;
  color: oklch(var(--bc) / 0.6);
  padding: 4px 16px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin: 0;
}

.branch-list {
  padding: 0 8px;
}

.branch-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 8px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.1s;
}

.branch-item:hover {
  background: var(--hover-bg);
}

.branch-item.current {
  background: oklch(var(--p) / 0.1);
}

.branch-item.remote {
  opacity: 0.8;
  cursor: default;
}

.branch-label {
  font-size: 13px;
  flex: 1;
}

.current-badge {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 3px;
  background: oklch(var(--p));
  color: white;
  font-weight: 600;
}

.branch-actions {
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.15s;
}

.branch-item:hover .branch-actions {
  opacity: 1;
}

.btn-danger:hover {
  color: #ef4444;
}

.branch-empty {
  padding: 8px 16px;
  color: oklch(var(--bc) / 0.6);
  font-size: 12px;
}

/* ===================== 对话框 ===================== */
.create-branch-dialog,
.merge-dialog {
  width: 480px;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  padding: 20px;
}

.dialog-title {
  font-size: 16px;
  font-weight: 600;
  margin: 0 0 16px;
}

.dialog-form {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 16px;
}

.dialog-form .form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

.dialog-form .form-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.form-label {
  font-size: 12px;
  font-weight: 600;
  color: oklch(var(--bc) / 0.6);
}

.form-input {
  padding: 8px 10px;
  border: 1px solid oklch(var(--bc) / 0.2);
  border-radius: 4px;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  font-size: 13px;
  outline: none;
}

.form-input:focus {
  border-color: oklch(var(--p));
}

.dialog-text {
  margin: 0 0 16px;
  font-size: 13px;
  line-height: 1.5;
}

.code-highlight {
  background: oklch(var(--p) / 0.1);
  color: oklch(var(--p));
  padding: 1px 4px;
  border-radius: 3px;
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  font-size: 12px;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

/* ===================== 工具按钮 ===================== */
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  border: 1px solid transparent;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
  white-space: nowrap;
  font-family: inherit;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-xs {
  padding: 2px 6px;
  font-size: 11px;
}

.btn-sm {
  padding: 4px 10px;
  font-size: 12px;
}

.btn-ghost {
  background: transparent;
  color: oklch(var(--bc));
  border-color: transparent;
}

.btn-ghost:hover:not(:disabled) {
  background: var(--hover-bg);
  border-color: oklch(var(--bc) / 0.1);
}

.btn-primary {
  background: oklch(var(--p));
  color: white;
  border-color: oklch(var(--p));
}

.btn-primary:hover:not(:disabled) {
  filter: brightness(1.1);
}

.btn-success {
  background: #22c55e;
  color: white;
  border-color: #22c55e;
}

.btn-success:hover:not(:disabled) {
  filter: brightness(1.1);
}

.btn-danger {
  color: #ef4444;
}

.btn-danger:hover:not(:disabled) {
  background: rgba(239, 68, 68, 0.1);
}

/* ===================== 动画 ===================== */
@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.spin,
.spin-icon {
  animation: spin 1s linear infinite;
}

/* ===================== 滚动条 ===================== */
.file-list::-webkit-scrollbar,
.log-table-wrap::-webkit-scrollbar,
.branches-content::-webkit-scrollbar,
.diff-content::-webkit-scrollbar {
  width: 6px;
}

.file-list::-webkit-scrollbar-track,
.log-table-wrap::-webkit-scrollbar-track,
.branches-content::-webkit-scrollbar-track,
.diff-content::-webkit-scrollbar-track {
  background: transparent;
}

.file-list::-webkit-scrollbar-thumb,
.log-table-wrap::-webkit-scrollbar-thumb,
.branches-content::-webkit-scrollbar-thumb,
.diff-content::-webkit-scrollbar-thumb {
  background: oklch(var(--bc) / 0.1);
  border-radius: 3px;
}

.file-list::-webkit-scrollbar-thumb:hover,
.log-table-wrap::-webkit-scrollbar-thumb:hover,
.branches-content::-webkit-scrollbar-thumb:hover,
.diff-content::-webkit-scrollbar-thumb:hover {
  background: oklch(var(--bc) / 0.6);
}

/* ===================== 响应式 ===================== */
@media (max-width: 768px) {
  .git-main {
    flex-direction: column;
  }

  .commit-panel {
    border-right: none;
    border-bottom: 1px solid oklch(var(--bc) / 0.1);
    max-height: 40%;
  }

  .splitter {
    width: 100%;
    height: 4px;
    cursor: row-resize;
  }

  .repo-path {
    display: none;
  }

  .log-filters {
    flex-wrap: wrap;
  }
}

/* ===================== 视图切换按钮 ===================== */
.log-view-toggle {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 2px;
  background: oklch(var(--b2));
  border-radius: 6px;
  border: 1px solid oklch(var(--bc) / 0.2);
}

.view-toggle-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 24px;
  padding: 0;
  border: none;
  background: transparent;
  color: oklch(var(--bc) / 0.6);
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.15s;
}

.view-toggle-btn:hover {
  color: oklch(var(--bc));
  background: var(--hover-bg);
}

.view-toggle-btn.active {
  color: oklch(var(--p));
  background: oklch(var(--p) / 0.1);
}

/* ===================== Git Graph ===================== */
.git-graph-wrap {
  position: relative;
  flex: 1;
  overflow: hidden;
  background: #1a1a2e;
  border-radius: 4px;
}

.git-graph-canvas {
  display: block;
  width: 100%;
  height: 100%;
  cursor: pointer;
}

.graph-loading,
.graph-empty {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  color: oklch(var(--bc) / 0.6);
}

.graph-loading svg {
  animation: spin 1s linear infinite;
}

/* ===================== Git Console ===================== */
.git-console-wrap {
  display: flex;
  flex-direction: column;
  flex: 1;
  overflow: hidden;
  background: #0d1117;
  border-radius: 4px;
  font-family: 'JetBrains Mono', 'Consolas', monospace;
}

.git-console-output {
  flex: 1;
  overflow-y: auto;
  padding: 8px 12px;
  font-size: 12px;
  line-height: 1.5;
}

.console-entry {
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.console-cmd {
  display: flex;
  align-items: center;
  gap: 6px;
  color: #58a6ff;
  font-weight: 500;
}

.console-prompt {
  color: #3fb950;
  font-weight: bold;
}

.console-cmd-text {
  color: #e6edf3;
}

.console-output-text {
  margin: 4px 0 0 18px;
  padding: 8px;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 4px;
  white-space: pre-wrap;
  word-break: break-all;
  color: #c9d1d9;
  font-size: 11px;
  max-height: 300px;
  overflow-y: auto;
}

.console-entry.error .console-cmd-text {
  color: #f85149;
}

.console-entry.error .console-output-text {
  color: #f85149;
  background: rgba(248, 81, 73, 0.05);
}

.console-welcome {
  padding: 20px;
  text-align: center;
  color: #8b949e;
}

.console-welcome p {
  margin: 4px 0;
}

.console-hint {
  font-size: 11px;
}

.console-hint code {
  background: rgba(255, 255, 255, 0.08);
  padding: 2px 6px;
  border-radius: 3px;
  color: #58a6ff;
}

.git-console-input-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: rgba(255, 255, 255, 0.03);
  border-top: 1px solid rgba(255, 255, 255, 0.08);
}

.console-input-prompt {
  color: #3fb950;
  font-weight: bold;
  font-size: 13px;
  flex-shrink: 0;
}

.git-console-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: #e6edf3;
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  font-size: 12px;
}

.git-console-input::placeholder {
  color: #484f58;
}

/* ===================== Push 对话框 ===================== */
.push-dialog {
  width: 460px;
  max-height: 70vh;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.push-commits-list {
  max-height: 200px;
  overflow-y: auto;
  border-top: 1px solid oklch(var(--bc) / 0.1);
}

.push-commits-header {
  padding: 8px 16px;
  font-size: 12px;
  font-weight: 600;
  color: oklch(var(--bc) / 0.6);
  background: oklch(var(--b2));
  position: sticky;
  top: 0;
}

.push-commit-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 16px;
  font-size: 12px;
  transition: background 0.1s;
}

.push-commit-item:hover {
  background: var(--hover-bg);
}

.push-commit-item .commit-msg {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* ===================== Pull 对话框 ===================== */
.pull-dialog {
  width: 420px;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* ===================== Amend 对话框 ===================== */
.amend-dialog {
  width: 420px;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  padding: 20px;
}

/* ===================== Reset 对话框 ===================== */
.reset-dialog {
  width: 420px;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  padding: 20px;
}

.reset-warning {
  margin-top: 8px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 4px;
  color: #ef4444;
  font-size: 12px;
}

/* ===================== Interactive Rebase 对话框 ===================== */
.interactive-rebase-dialog {
  width: 600px;
  max-height: 80vh;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.ir-commits-list {
  flex: 1;
  overflow-y: auto;
  max-height: 400px;
  padding: 4px 8px;
}

.ir-commit-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  transition: background 0.1s;
  cursor: pointer;
}

.ir-commit-item:hover {
  background: var(--hover-bg);
}

.ir-commit-item.selected {
  background: oklch(var(--p) / 0.1);
}

.ir-action-select {
  background: oklch(var(--b2));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 3px;
  color: oklch(var(--bc));
  font-size: 11px;
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  padding: 2px 4px;
  cursor: pointer;
  flex-shrink: 0;
}

.ir-action-select option[value="pick"] { color: #22c55e; }
.ir-action-select option[value="reword"] { color: #3b82f6; }
.ir-action-select option[value="edit"] { color: #f59e0b; }
.ir-action-select option[value="squash"] { color: #a855f7; }
.ir-action-select option[value="fixup"] { color: #8b5cf6; }
.ir-action-select option[value="drop"] { color: #ef4444; }

.ir-commit-item .commit-msg {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ir-move-btn {
  padding: 1px 4px;
  font-size: 14px;
  line-height: 1;
  flex-shrink: 0;
}

/* ===================== Remotes 对话框 ===================== */
.remotes-dialog {
  width: 520px;
  max-height: 70vh;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.remotes-content {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.remote-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: 4px;
  font-size: 13px;
  transition: background 0.1s;
}

.remote-item:hover {
  background: var(--hover-bg);
}

.remote-item svg {
  flex-shrink: 0;
  color: oklch(var(--bc) / 0.6);
}

.remote-name {
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  font-weight: 600;
  color: oklch(var(--p));
  min-width: 60px;
}

.remote-url {
  flex: 1;
  font-size: 11px;
  color: oklch(var(--bc) / 0.6);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: 'JetBrains Mono', 'Consolas', monospace;
}

.remotes-empty {
  padding: 20px;
  text-align: center;
  color: oklch(var(--bc) / 0.6);
}

/* ===================== 分支重命名对话框 ===================== */
.branch-rename-dialog {
  width: 380px;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  padding: 20px;
}

/* ===================== Force Push 按钮 ===================== */
.btn-danger {
  color: #ef4444;
}

.btn-danger:hover {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

/* ===================== Branch Rename 按钮 ===================== */
.branch-actions {
  display: flex;
  gap: 4px;
  align-items: center;
}

/* ===================== File Preview Dialog ===================== */
.file-preview-dialog {
  width: 800px;
  max-height: 80vh;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.file-preview-content {
  flex: 1;
  overflow: auto;
  padding: 12px 16px;
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  font-size: 12px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-all;
  color: oklch(var(--bc));
  background: oklch(var(--b2));
  margin: 0;
}

</style>
