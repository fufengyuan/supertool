<template>
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
      <div class="branch-selector" @click="$emit('open-branches')">
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
        <button class="btn btn-ghost btn-sm" @click="$emit('update:showStashMenu', !showStashMenu)" title="Stash">
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
          <div class="dropdown-menu-item" @click="$emit('stash-save')">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
            </svg>
            Save Stash...
          </div>
          <div class="dropdown-menu-item" @click="$emit('stash-save-untracked')">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
            </svg>
            Stash All (incl. untracked)
          </div>
          <div class="dropdown-menu-separator"></div>
          <div class="dropdown-menu-item" @click="$emit('toggle-stash-panel')">
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
        <button class="btn btn-ghost btn-sm" @click="$emit('update:showGitMenu', !showGitMenu)" title="Git Operations">
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
          <div class="dropdown-menu-item" @click="$emit('git-action', 'rebase')">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="17 1 21 5 17 9" /><path d="M3 11V9a4 4 0 0 1 4-4h14" />
              <polyline points="7 23 3 19 7 15" /><path d="M21 13v2a4 4 0 0 1-4 4H3" />
            </svg>
            Rebase...
          </div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'compare-branches')">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="20" x2="18" y2="10" /><line x1="12" y1="20" x2="12" y2="4" />
              <line x1="6" y1="20" x2="6" y2="14" />
            </svg>
            Compare Branches...
          </div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'tags')">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z" />
              <line x1="7" y1="7" x2="7.01" y2="7" />
            </svg>
            Tags...
          </div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'submodules')">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="2" y="2" width="9" height="9" rx="1" /><rect x="13" y="2" width="9" height="9" rx="1" />
              <rect x="2" y="13" width="9" height="9" rx="1" /><rect x="13" y="13" width="9" height="9" rx="1" />
            </svg>
            Submodules...
          </div>
          <div class="dropdown-menu-separator"></div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'push-dialog')">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
              <polyline points="17 8 12 3 7 8" />
              <line x1="12" y1="3" x2="12" y2="15" />
            </svg>
            Push...
          </div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'pull-dialog')">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
              <polyline points="7 10 12 15 17 10" />
              <line x1="12" y1="15" x2="12" y2="3" />
            </svg>
            Pull...
          </div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'fetch')">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="23 4 23 10 17 10" />
              <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
            </svg>
            Fetch
          </div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'push-tags')">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z" />
              <line x1="7" y1="7" x2="7.01" y2="7" />
              <polyline points="17 8 12 3 7 8" />
              <line x1="12" y1="3" x2="12" y2="15" />
            </svg>
            Push Tags
          </div>
          <div class="dropdown-menu-separator"></div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'undo-last-commit')">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="1 4 1 10 7 10" /><path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
            </svg>
            Undo Last Commit
          </div>
          <div class="dropdown-menu-separator"></div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'amend')">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 20h9" /><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
            </svg>
            Amend Last Commit...
          </div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'reset')">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="1 4 1 10 7 10" /><path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
            </svg>
            Reset to Commit...
          </div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'interactive-rebase')">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="8" y1="6" x2="21" y2="6" /><line x1="8" y1="12" x2="21" y2="12" /><line x1="8" y1="18" x2="21" y2="18" />
              <line x1="3" y1="6" x2="3.01" y2="6" /><line x1="3" y1="12" x2="3.01" y2="12" /><line x1="3" y1="18" x2="3.01" y2="18" />
            </svg>
            Interactive Rebase...
          </div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'remotes')">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10" /><line x1="2" y1="12" x2="22" y2="12" />
              <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
            </svg>
            Remotes...
          </div>
          <div class="dropdown-menu-separator"></div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'create-patch')">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
              <polyline points="14 2 14 8 20 8" /><line x1="16" y1="13" x2="8" y2="13" />
              <line x1="16" y1="17" x2="8" y2="17" />
            </svg>
            Create Patch...
          </div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'apply-patch')">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
              <polyline points="14 2 14 8 20 8" />
              <polyline points="9 15 12 18 16 13" />
            </svg>
            Apply Patch...
          </div>
          <div class="dropdown-menu-item" @click="$emit('git-action', 'git-clean')">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="3 6 5 6 21 6" /><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
            </svg>
            Clean Working Tree...
          </div>
        </div>
      </div>
      <button class="btn btn-ghost btn-sm" @click="$emit('pull')" :disabled="pulling" title="Pull">
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
      <button class="btn btn-ghost btn-sm" @click="$emit('push')" :disabled="pushing" title="Push">
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
      <button class="btn btn-ghost btn-sm btn-error" @click="$emit('force-push')" :disabled="pushing" title="Force Push">
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
      <button class="btn btn-ghost btn-sm" @click="$emit('open-branches')" title="Merge/Branches">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="6" y1="3" x2="6" y2="15" />
          <circle cx="18" cy="6" r="3" />
          <circle cx="6" cy="18" r="3" />
          <path d="M18 9a9 9 0 0 1-9 9" />
        </svg>
        Branches
      </button>
      <button class="btn btn-ghost btn-sm" @click="$emit('refresh')" :disabled="loading" title="Refresh">
        <svg :class="{ spin: loading }" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="23 4 23 10 17 10" />
          <polyline points="1 20 1 14 7 14" />
          <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
        </svg>
        Refresh
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
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
