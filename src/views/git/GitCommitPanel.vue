<template>
  <div class="commit-panel">
    <div class="panel-header">
      <span class="panel-title">
        变更
        <span class="change-count" v-if="totalChanges > 0">{{ totalChanges }}</span>
      </span>
      <div class="panel-header-actions">
        <button class="btn btn-ghost btn-xs" @click="$emit('select-all-files')" :disabled="totalChanges === 0" title="全选">
          <SvgIcon name="checkCircle" :size="12" />
          全选
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
          <span class="group-label">已修改</span>
          <span class="group-count">{{ statusData.modified.length }}</span>
        </div>
        <div class="group-files" v-show="!collapsedGroups.has('modified')">
          <div
            v-for="file in statusData.modified"
            :key="'M:' + file"
            class="file-item"
            :class="{ selected: selectedFiles.has(file), previewing: selectedPreviewFile === file }"
            @contextmenu.prevent="showFileContextMenu($event, file, 'modified')"
          >
            <input type="checkbox" :checked="selectedFiles.has(file)" @change="toggleFileSelect(file)" class="file-checkbox" />
            <span class="file-icon modified">M</span>
            <span class="file-name" :title="file" @dblclick="previewFile(file)">{{ file }}</span>
          </div>
        </div>
      </div>

      <!-- Added -->
      <div class="file-group" v-if="statusData.added.length">
        <div class="group-header" @click="$emit('toggle-group', 'added')">
          <SvgIcon name="chevronDown" size="12" class="group-arrow" :class="{ collapsed: collapsedGroups.has('added') }" />
          <span class="group-icon added">A</span>
          <span class="group-label">已添加</span>
          <span class="group-count">{{ statusData.added.length }}</span>
        </div>
        <div class="group-files" v-show="!collapsedGroups.has('added')">
          <div
            v-for="file in statusData.added"
            :key="'A:' + file"
            class="file-item"
            :class="{ selected: selectedFiles.has(file), previewing: selectedPreviewFile === file }"
            @contextmenu.prevent="showFileContextMenu($event, file, 'added')"
          >
            <input type="checkbox" :checked="selectedFiles.has(file)" @change="toggleFileSelect(file)" class="file-checkbox" />
            <span class="file-icon added">A</span>
            <span class="file-name" :title="file" @dblclick="previewFile(file)">{{ file }}</span>
          </div>
        </div>
      </div>

      <!-- Deleted -->
      <div class="file-group" v-if="statusData.deleted.length">
        <div class="group-header" @click="$emit('toggle-group', 'deleted')">
          <SvgIcon name="chevronDown" size="12" class="group-arrow" :class="{ collapsed: collapsedGroups.has('deleted') }" />
          <span class="group-icon deleted">D</span>
          <span class="group-label">已删除</span>
          <span class="group-count">{{ statusData.deleted.length }}</span>
        </div>
        <div class="group-files" v-show="!collapsedGroups.has('deleted')">
          <div
            v-for="file in statusData.deleted"
            :key="'D:' + file"
            class="file-item"
            :class="{ selected: selectedFiles.has(file), previewing: selectedPreviewFile === file }"
            @contextmenu.prevent="showFileContextMenu($event, file, 'deleted')"
          >
            <input type="checkbox" :checked="selectedFiles.has(file)" @change="toggleFileSelect(file)" class="file-checkbox" />
            <span class="file-icon deleted">D</span>
            <span class="file-name" :title="file" @dblclick="previewFile(file)">{{ file }}</span>
          </div>
        </div>
      </div>

      <!-- Untracked -->
      <div class="file-group" v-if="statusData.untracked.length">
        <div class="group-header" @click="$emit('toggle-group', 'untracked')">
          <SvgIcon name="chevronDown" size="12" class="group-arrow" :class="{ collapsed: collapsedGroups.has('untracked') }" />
          <span class="group-icon untracked">U</span>
          <span class="group-label">未跟踪</span>
          <span class="group-count">{{ statusData.untracked.length }}</span>
        </div>
        <div class="group-files" v-show="!collapsedGroups.has('untracked')">
          <div
            v-for="file in statusData.untracked"
            :key="'U:' + file"
            class="file-item"
            :class="{ selected: selectedFiles.has(file), previewing: selectedPreviewFile === file }"
            @contextmenu.prevent="showFileContextMenu($event, file, 'untracked')"
          >
            <input type="checkbox" :checked="selectedFiles.has(file)" @change="toggleFileSelect(file)" class="file-checkbox" />
            <span class="file-icon untracked">U</span>
            <span class="file-name" :title="file" @dblclick="previewFile(file)">{{ file }}</span>
          </div>
        </div>
      </div>

      <!-- Conflicted -->
      <div class="file-group" v-if="statusData.conflicted.length">
        <div class="group-header" @click="$emit('toggle-group', 'conflicted')">
          <SvgIcon name="chevronDown" size="12" class="group-arrow" :class="{ collapsed: collapsedGroups.has('conflicted') }" />
          <span class="group-icon conflicted">C</span>
          <span class="group-label">冲突</span>
          <span class="group-count">{{ statusData.conflicted.length }}</span>
        </div>
        <div class="group-files" v-show="!collapsedGroups.has('conflicted')">
          <div
            v-for="file in statusData.conflicted"
            :key="'C:' + file"
            class="file-item conflicted-item"
            :class="{ selected: selectedFiles.has(file), previewing: selectedPreviewFile === file }"
            @contextmenu.prevent="showFileContextMenu($event, file, 'conflicted')"
          >
            <input type="checkbox" :checked="selectedFiles.has(file)" @change="toggleFileSelect(file)" class="file-checkbox" />
            <span class="file-icon conflicted">C</span>
            <span class="file-name" :title="file" @dblclick="previewFile(file)">{{ file }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Diff 预览区域 -->
    <div v-if="previewDiff" class="diff-preview-area">
      <div class="diff-preview-header">
        <span class="preview-title">{{ selectedPreviewFile }}</span>
        <button class="btn btn-ghost btn-xs" @click="$emit('clear-preview')" title="关闭预览">
          <SvgIcon name="x" size="12" />
        </button>
      </div>
      <div class="diff-preview-content">
        <SplitDiffViewer :files="null" :diff="previewDiff" :loading="loadingPreview" />
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
          签署 (-s)
        </label>
        <label class="form-checkbox-label commit-option-label">
          <input type="checkbox" :checked="commitNoVerify" @change="$emit('update:commitNoVerify', ($event.target as HTMLInputElement).checked)" />
          跳过验证 (--no-verify)
        </label>
      </div>
      <div class="commit-actions">
        <button
          class="btn btn-primary btn-sm"
          @click="$emit('commit', false)"
          :disabled="committing || selectedFiles.size === 0 || !commitMessage.trim()"
        >
          提交
        </button>
        <button
          class="btn btn-success btn-sm"
          @click="$emit('commit', true)"
          :disabled="committing || selectedFiles.size === 0 || !commitMessage.trim()"
        >
          提交并推送
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import SplitDiffViewer from '@/components/ui/SplitDiffViewer.vue'

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
  previewDiff: string | null
  selectedPreviewFile: string | null
  loadingPreview: boolean
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
  'preview-file': [file: string]
  'clear-preview': []
}>()

function toggleFileSelect(file: string) {
  emit('toggle-file-select', file)
}

function previewFile(file: string) {
  emit('preview-file', file)
}

function showFileContextMenu(event: MouseEvent, file: string, type: string) {
  emit('file-context-menu', { event, file, type })
}
</script>

<style>
/* GitCommitPanel 样式 - IDEA 风格 */
/* ===================== 提交面板布局 ===================== */
.commit-panel {
  display: flex;
  flex-direction: column;
  border-right: none;
  width: 100%;
  flex-shrink: 0;
  background: var(--color-base-200);
}

/* ===================== 面板头部 ===================== */
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 3px 8px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 8%, transparent);
  background: var(--color-base-200);
  flex-shrink: 0;
  height: 24px;
}

.panel-title {
  font-weight: 500;
  font-size: 12px;
  display: flex;
  align-items: center;
  gap: 4px;
}

.change-count {
  background: color-mix(in oklab, var(--color-primary) 15%, transparent);
  color: var(--color-primary);
  padding: 0 4px;
  border-radius: 8px;
  font-size: 10px;
  font-weight: 500;
}

/* ===================== 文件列表 ===================== */
.file-list {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

.file-group {
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 6%, transparent);
}

.group-header {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  cursor: pointer;
  background: var(--color-base-200);
  user-select: none;
  transition: background 0.1s;
  height: 20px;
}

.group-header:hover {
  background: var(--hover-bg);
}

.group-arrow {
  transition: transform 0.1s;
  color: color-mix(in oklab, var(--color-base-content) 50%, transparent);
  font-size: 10px;
}

.group-arrow.collapsed {
  transform: rotate(-90deg);
}

.group-icon {
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 2px;
  font-size: 9px;
  font-weight: 700;
  font-family: 'JetBrains Mono', 'Consolas', monospace;
}

.group-icon.modified {
  background: color-mix(in oklab, var(--color-warning) 15%, transparent);
  color: var(--color-warning);
}

.group-icon.added {
  background: color-mix(in oklab, var(--color-success) 15%, transparent);
  color: var(--color-success);
}

.group-icon.deleted {
  background: color-mix(in oklab, var(--color-error) 15%, transparent);
  color: var(--color-error);
}

.group-icon.untracked {
  background: color-mix(in oklab, var(--color-base-content) 15%, transparent);
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.group-icon.conflicted {
  background: color-mix(in oklab, var(--color-secondary) 15%, transparent);
  color: var(--color-secondary);
}

.group-label {
  font-weight: 500;
  font-size: 11px;
}

.group-count {
  margin-left: auto;
  font-size: 10px;
  color: color-mix(in oklab, var(--color-base-content) 50%, transparent);
}

.group-files {
  padding-left: 8px;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 1px 8px;
  transition: background 0.1s;
  user-select: none;
  height: 20px;
}

.file-item:hover {
  background: var(--hover-bg);
}

/* 文件图标不响应点击 */
.file-icon {
  pointer-events: none;
}

/* checkbox 可点击 */
.file-checkbox {
  pointer-events: auto;
}

.file-item.selected {
  background: color-mix(in oklab, var(--color-primary) 8%, transparent);
}

.file-item.conflicted-item {
  background: color-mix(in oklab, var(--color-secondary) 5%, transparent);
}

.file-checkbox {
  flex-shrink: 0;
  width: 12px;
  height: 12px;
  accent-color: var(--color-primary);
  cursor: pointer !important;
  margin: 0;
  padding: 0;
}

.file-icon {
  width: 14px;
  height: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 2px;
  font-size: 8px;
  font-weight: 700;
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  flex-shrink: 0;
}

.file-icon.modified {
  background: color-mix(in oklab, var(--color-warning) 15%, transparent);
  color: var(--color-warning);
}

.file-icon.added {
  background: color-mix(in oklab, var(--color-success) 15%, transparent);
  color: var(--color-success);
}

.file-icon.deleted {
  background: color-mix(in oklab, var(--color-error) 15%, transparent);
  color: var(--color-error);
}

.file-icon.untracked {
  background: color-mix(in oklab, var(--color-base-content) 15%, transparent);
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.file-icon.conflicted {
  background: color-mix(in oklab, var(--color-secondary) 15%, transparent);
  color: var(--color-secondary);
}

.file-name {
  font-size: 11px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  cursor: pointer;
  flex: 1;
}

.file-name:hover {
  color: var(--color-primary);
}

.empty-files {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  color: color-mix(in oklab, var(--color-base-content) 50%, transparent);
  gap: 6px;
  padding: 16px;
}

.empty-files svg {
  color: var(--color-success);
}

.empty-files p {
  font-size: 12px;
}

/* ===================== Diff 预览区域 ===================== */
.diff-preview-area {
  display: flex;
  flex-direction: column;
  border-top: 1px solid color-mix(in oklab, var(--color-base-content) 8%, transparent);
  background: var(--color-base-100);
  max-height: 180px;
  flex-shrink: 0;
}

.diff-preview-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 2px 8px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 8%, transparent);
  background: var(--color-base-200);
  height: 22px;
}

.preview-title {
  font-size: 10px;
  font-weight: 500;
  color: var(--color-base-content);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.diff-preview-content {
  flex: 1;
  overflow-y: auto;
  padding: 2px;
}

.file-item.previewing {
  background: color-mix(in oklab, var(--color-primary) 8%, transparent);
}

/* ===================== 提交区域 ===================== */
.commit-area {
  display: flex;
  flex-direction: column;
  border-top: 1px solid color-mix(in oklab, var(--color-base-content) 8%, transparent);
  background: var(--color-base-100);
  flex-shrink: 0;
}

.commit-header {
  padding: 3px 8px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 8%, transparent);
  height: 22px;
}

.commit-title {
  font-weight: 500;
  font-size: 11px;
}

.commit-message-input {
  width: 100%;
  padding: 4px 8px;
  border: none;
  background: var(--color-base-200);
  color: var(--color-base-content);
  font-size: 12px;
  font-family: inherit;
  resize: vertical;
  outline: none;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 8%, transparent);
  min-height: 40px;
}

.commit-message-input:focus {
  background: var(--color-base-200);
}

.commit-message-input::placeholder {
  color: color-mix(in oklab, var(--color-base-content) 50%, transparent);
  opacity: 0.6;
}

.commit-actions {
  display: flex;
  gap: 4px;
  padding: 4px 8px;
}

/* ===================== 多选标签 ===================== */
.form-checkbox-label {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--color-base-content);
  cursor: pointer;
}

.form-checkbox-label input[type="checkbox"] {
  accent-color: var(--color-primary);
  width: 12px;
  height: 12px;
}

/* ===================== 滚动条 ===================== */
.file-list::-webkit-scrollbar {
  width: 4px;
}

.file-list::-webkit-scrollbar-track {
  background: transparent;
}

.file-list::-webkit-scrollbar-thumb {
  background: color-mix(in oklab, var(--color-base-content) 8%, transparent);
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
