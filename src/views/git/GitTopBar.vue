<template>
  <!-- ===== 顶部工具栏 ===== -->
  <div class="git-topbar">
    <div class="topbar-left">
      <button class="btn btn-ghost btn-sm back-btn" @click="$emit('close')" title="返回仓库列表">
        <SvgIcon name="chevronLeft" width="16" height="16" />
      </button>
      <div class="repo-info">
        <span class="repo-name">{{ repo.name }}</span>
        <span class="repo-path" :title="repo.path">{{ repo.path }}</span>
      </div>
    </div>

    <div class="topbar-center">
      <div class="branch-selector" @click="$emit('open-branches')">
        <SvgIcon name="gitBranch" width="14" height="14" />
        <span class="branch-name">{{ currentBranch || '...' }}</span>
        <SvgIcon name="chevronDown" width="12" height="12" />
      </div>
      <span v-if="statusData" class="ahead-behind">
        <span v-if="statusData.ahead > 0" class="ahead" title="Ahead of remote">↑{{ statusData.ahead }}</span>
        <span v-if="statusData.behind > 0" class="behind" title="Behind remote">↓{{ statusData.behind }}</span>
      </span>
    </div>

    <div class="topbar-right">
      <!-- Stash 下拉按钮 -->
      <div class="dropdown-wrap">
        <button class="btn btn-ghost btn-sm" @click="$emit('update:showStashMenu', !showStashMenu)" title="Stash">
          <SvgIcon name="archive" width="14" height="14" />
          Stash
          <SvgIcon class="dropdown-arrow" name="chevronDown" width="10" height="10" />
        </button>
        <div v-if="showStashMenu" class="dropdown-menu" @click.stop>
          <div class="dropdown-menu-item" @click="$emit('stash-save')">
            <SvgIcon name="plus" width="14" height="14" />
            Save Stash...
          </div>
          <div class="dropdown-menu-item" @click="$emit('stash-save-untracked')">
            <SvgIcon name="plus" width="14" height="14" />
            Stash All (incl. untracked)
          </div>
          <div class="dropdown-menu-separator"></div>
          <div class="dropdown-menu-item" @click="$emit('toggle-stash-panel')">
            <SvgIcon name="archive" width="14" height="14" />
            Show Stash List
          </div>
        </div>
      </div>
      <!-- Git 菜单按钮 -->
      <div class="dropdown-wrap">
        <button class="btn btn-ghost btn-sm" @click="$emit('update:showGitMenu', !showGitMenu)" title="Git Operations">
          <SvgIcon name="gitPullRequest" width="14" height="14" />
          Git
          <SvgIcon class="dropdown-arrow" name="chevronDown" width="10" height="10" />
        </button>
        <div v-if="showGitMenu" class="dropdown-menu" @click.stop>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'rebase')">
            <SvgIcon name="gitMerge" width="14" height="14" />
            Rebase...
          </div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'compare-branches')">
            <SvgIcon name="barChart" width="14" height="14" />
            Compare Branches...
          </div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'tags')">
            <SvgIcon name="tag" width="14" height="14" />
            Tags...
          </div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'submodules')">
            <SvgIcon name="grid" width="14" height="14" />
            Submodules...
          </div>
          <div class="dropdown-menu-separator"></div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'push-dialog')">
            <SvgIcon name="upload" width="14" height="14" />
            Push...
          </div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'pull-dialog')">
            <SvgIcon name="download" width="14" height="14" />
            Pull...
          </div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'fetch')">
            <SvgIcon name="refresh" width="14" height="14" />
            Fetch
          </div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'push-tags')">
            <SvgIcon name="tag" width="14" height="14" />
            Push Tags
          </div>
          <div class="dropdown-menu-separator"></div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'undo-last-commit')">
            <SvgIcon name="undo" width="14" height="14" />
            Undo Last Commit
          </div>
          <div class="dropdown-menu-separator"></div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'amend')">
            <SvgIcon name="pencil" width="14" height="14" />
            Amend Last Commit...
          </div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'reset')">
            <SvgIcon name="undo" width="14" height="14" />
            Reset to Commit...
          </div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'interactive-rebase')">
            <SvgIcon name="menu" width="14" height="14" />
            Interactive Rebase...
          </div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'remotes')">
            <SvgIcon name="globe" width="14" height="14" />
            Remotes...
          </div>
          <div class="dropdown-menu-separator"></div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'create-patch')">
            <SvgIcon name="file" width="14" height="14" />
            Create Patch...
          </div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'apply-patch')">
            <SvgIcon name="file" width="14" height="14" />
            Apply Patch...
          </div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'git-clean')">
            <SvgIcon name="trash" width="14" height="14" />
            Clean Working Tree...
          </div>
        </div>
      </div>
      <button class="btn btn-ghost btn-sm" @click="$emit('pull')" :disabled="pulling" title="Pull">
        <SvgIcon v-if="!pulling" name="download" width="14" height="14" />
        <SvgIcon v-else name="refresh" class="spin-icon" width="14" height="14" />
        Pull
      </button>
      <button class="btn btn-ghost btn-sm" @click="$emit('push')" :disabled="pushing" title="Push">
        <SvgIcon v-if="!pushing" name="upload" width="14" height="14" />
        <SvgIcon v-else name="refresh" class="spin-icon" width="14" height="14" />
        Push
      </button>
      <button class="btn btn-ghost btn-sm btn-error" @click="$emit('force-push')" :disabled="pushing" title="Force Push">
        <SvgIcon v-if="!pushing" name="upload" width="14" height="14" />
        <SvgIcon v-else name="refresh" class="spin-icon" width="14" height="14" />
        Force Push
      </button>
      <button class="btn btn-ghost btn-sm" @click="$emit('open-branches')" title="Merge/Branches">
        <SvgIcon name="gitBranch" width="14" height="14" />
        Branches
      </button>
      <button class="btn btn-ghost btn-sm" @click="$emit('refresh')" :disabled="loading" title="Refresh">
        <SvgIcon :class="{ spin: loading }" name="refresh" width="14" height="14" />
        Refresh
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
defineProps<{
  repo: { name: string; path: string }
  currentBranch: string
  statusData: { ahead: number; behind: number } | null
  loading: boolean
  pulling: boolean
  pushing: boolean
  showStashMenu: boolean
  showGitMenu: boolean
}>()

defineEmits<{
  'close': []
  'update:showStashMenu': [value: boolean]
  'update:showGitMenu': [value: boolean]
  'open-branches': []
  'git-action': [action: string]
  'stash-save': []
  'stash-save-untracked': []
  'toggle-stash-panel': []
  'pull': []
  'push': []
  'force-push': []
  'refresh': []
}>()
</script>

<style>
/* GitTopBar 子组件样式 — 从 GitManager.vue 复制 */
/* ===================== 下拉菜单 ===================== */
.dropdown-wrap {
  position: relative;
}

.dropdown-menu {
  position: absolute;
  top: 100%;
  right: 0;
  z-index: 1000;
  background: var(--color-base-100);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
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
  color: var(--color-base-content);
}

.dropdown-menu-item:hover {
  background: var(--hover-bg);
}

.dropdown-menu-item svg {
  flex-shrink: 0;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.dropdown-menu-separator {
  height: 1px;
  background: color-mix(in oklab, var(--color-base-content) 10%, transparent);
  margin: 4px 0;
}

/* ===================== 顶部工具栏 ===================== */
.git-topbar {
  display: flex;
  align-items: center;
  padding: 6px 12px;
  gap: 12px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  background: var(--color-base-100);
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
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
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
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 4px;
  background: var(--color-base-200);
  cursor: pointer;
  transition: border-color 0.15s ease;
}

.branch-selector:hover {
  border-color: var(--color-primary);
}

.branch-name {
  font-weight: 500;
  color: var(--color-primary);
}

.dropdown-arrow {
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
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

/* ===================== 动画 ===================== */
@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.spin-icon {
  animation: spin 1s linear infinite;
}
</style>
