<template>
  <div class="commit-panel">
    <div class="panel-header">
      <span class="panel-title">
        变更
        <span class="change-count" v-if="totalChanges > 0">{{ totalChanges }}</span>
      </span>
      <div class="panel-header-actions">
        <button class="btn btn-ghost btn-xs" @click="$emit('select-all-files')" :disabled="totalChanges === 0" title="全选">
          ☑ 全选
        </button>
      </div>
    </div>

    <!-- 文件变更列表 -->
    <div class="file-list" v-if="statusData">
      <!-- Modified -->
      <div class="file-group" v-if="statusData.modified.length">
        <div class="group-header" @click="$emit('toggle-group', 'modified')">
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
        <div class="group-header" @click="$emit('toggle-group', 'added')">
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
        <div class="group-header" @click="$emit('toggle-group', 'deleted')">
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
        <div class="group-header" @click="$emit('toggle-group', 'untracked')">
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
        <div class="group-header" @click="$emit('toggle-group', 'conflicted')">
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
        :value="commitMessage"
        @input="$emit('update:commitMessage', ($event.target as HTMLTextAreaElement).value)"
        class="commit-message-input"
        placeholder="输入提交信息..."
        rows="4"
        spellcheck="false"
      />
      <div class="commit-options">
        <label class="form-checkbox-label commit-option-label">
          <input type="checkbox" :checked="commitSignOff" @change="$emit('update:commitSignOff', ($event.target as HTMLInputElement).checked)" />
          Sign-off (-s)
        </label>
        <label class="form-checkbox-label commit-option-label">
          <input type="checkbox" :checked="commitNoVerify" @change="$emit('update:commitNoVerify', ($event.target as HTMLInputElement).checked)" />
          No Verify (--no-verify)
        </label>
      </div>
      <div class="commit-actions">
        <button
          class="btn btn-primary btn-sm"
          @click="$emit('commit', false)"
          :disabled="committing || selectedFiles.size === 0 || !commitMessage.trim()"
        >
          Commit
        </button>
        <button
          class="btn btn-success btn-sm"
          @click="$emit('commit', true)"
          :disabled="committing || selectedFiles.size === 0 || !commitMessage.trim()"
        >
          Commit & Push
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  statusData: any | null
  loading: boolean
  selectedFiles: Set<string>
  collapsedGroups: Set<string>
  commitMessage: string
  committing: boolean
  totalChanges: number
  commitSignOff: boolean
  commitNoVerify: boolean
}>()

const emit = defineEmits<{
  'update:commitMessage': [value: string]
  'update:commitSignOff': [value: boolean]
  'update:commitNoVerify': [value: boolean]
  'toggle-group': [name: string]
  'toggle-file-select': [file: string]
  'select-all-files': []
  'commit': [push: boolean]
  'file-context-menu': [payload: { event: MouseEvent; file: string; type: string }]
}>()

function toggleFileSelect(file: string) {
  emit('toggle-file-select', file)
}

function showFileContextMenu(event: MouseEvent, file: string, type: string) {
  emit('file-context-menu', { event, file, type })
}
</script>
