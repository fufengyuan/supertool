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
          <SvgIcon name="chevronDown" size="12" class="group-arrow" :class="{ collapsed: collapsedGroups.has('modified') }" />
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
          <SvgIcon name="chevronDown" size="12" class="group-arrow" :class="{ collapsed: collapsedGroups.has('added') }" />
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
          <SvgIcon name="chevronDown" size="12" class="group-arrow" :class="{ collapsed: collapsedGroups.has('deleted') }" />
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
          <SvgIcon name="chevronDown" size="12" class="group-arrow" :class="{ collapsed: collapsedGroups.has('untracked') }" />
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
          <SvgIcon name="chevronDown" size="12" class="group-arrow" :class="{ collapsed: collapsedGroups.has('conflicted') }" />
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
      <SvgIcon name="checkCircle" size="36" :strokeWidth="1.5" />
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
import SvgIcon from '@/components/ui/SvgIcon.vue'
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

<style>
/* GitCommitPanel 样式（从 GitManager.vue 提取） */
/* ===================== 提交面板布局 ===================== */
.commit-panel {
  display: flex;
  flex-direction: column;
  border-right: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  min-width: 200px;
  flex-shrink: 0;
}

/* ===================== 面板头部 ===================== */
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 10px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  background: var(--color-base-100);
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
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
  color: var(--color-primary);
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
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.group-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  cursor: pointer;
  background: var(--color-base-100);
  user-select: none;
  transition: background 0.1s;
}

.group-header:hover {
  background: var(--hover-bg);
}

.group-arrow {
  transition: transform 0.15s;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
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
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
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
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
}

.file-item.conflicted-item {
  background: rgba(168, 85, 247, 0.05);
}

.file-checkbox {
  flex-shrink: 0;
  width: 14px;
  height: 14px;
  accent-color: var(--color-primary);
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
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
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
  border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  background: var(--color-base-100);
  flex-shrink: 0;
}

.commit-header {
  padding: 6px 10px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.commit-title {
  font-weight: 600;
  font-size: 12px;
}

.commit-message-input {
  width: 100%;
  padding: 8px 10px;
  border: none;
  background: var(--color-base-200);
  color: var(--color-base-content);
  font-size: 13px;
  font-family: inherit;
  resize: vertical;
  outline: none;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.commit-message-input:focus {
  background: var(--color-base-200);
}

.commit-message-input::placeholder {
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  opacity: 0.5;
}

.commit-actions {
  display: flex;
  gap: 6px;
  padding: 8px 10px;
}

/* ===================== 多选标签 ===================== */
.form-checkbox-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--color-base-content);
  cursor: pointer;
}

.form-checkbox-label input[type="checkbox"] {
  accent-color: var(--color-primary);
}

/* ===================== 滚动条 ===================== */
.file-list::-webkit-scrollbar {
  width: 6px;
}

.file-list::-webkit-scrollbar-track {
  background: transparent;
}

.file-list::-webkit-scrollbar-thumb {
  background: color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 3px;
}

.file-list::-webkit-scrollbar-thumb:hover {
  background: color-mix(in oklab, var(--color-base-content) 20%, transparent);
}

/* ===================== 响应式 ===================== */
@media (max-width: 768px) {
  .commit-panel {
    border-right: none;
    border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
    max-height: 40%;
  }
}
</style>
