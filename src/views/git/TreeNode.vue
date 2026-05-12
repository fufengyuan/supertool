<template>
  <div class="tree-node">
    <div
      class="node-row"
      :class="{ selected: selectedPath === entry.path, expanded: isExpanded }"
      :style="{ paddingLeft: depth * 16 + 8 + 'px' }"
      @click="handleClick"
    >
      <!-- 展开/折叠按钮 -->
      <span
        v-if="entry.isDir"
        class="expand-icon"
        @click.stop="toggleExpand"
      >
        {{ isExpanded ? '▼' : '▶' }}
      </span>
      <span v-else class="expand-icon placeholder"></span>

      <!-- 文件图标 -->
      <span class="file-icon" :class="iconClass">
        {{ entry.isDir ? '📁' : getFileIcon(entry.name) }}
      </span>

      <!-- 文件名 -->
      <span class="file-name">{{ entry.name }}</span>
    </div>

    <!-- 子节点 -->
    <div v-if="entry.isDir && isExpanded && entry.children" class="node-children">
      <TreeNode
        v-for="child in entry.children"
        :key="child.path"
        :entry="child"
        :depth="depth + 1"
        :selected-path="selectedPath"
        @select="$emit('select', $event)"
        @expand="$emit('expand', $event)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, inject, Ref } from 'vue'

interface FileTreeEntry {
  path: string
  name: string
  isDir: boolean
  children?: FileTreeEntry[]
}

const props = defineProps<{
  entry: FileTreeEntry
  depth: number
  selectedPath: string | null
}>()

const emit = defineEmits<{
  'select': [path: string]
  'expand': [path: string]
}>()

// 从父组件获取 expandedPaths (是一个 Ref<Set<string>>)
const expandedPathsRef = inject<Ref<Set<string>>>('expandedPaths')

const isExpanded = computed(() => expandedPathsRef?.value?.has(props.entry.path) || false)

const iconClass = computed(() => {
  if (props.entry.isDir) return 'directory'
  const ext = props.entry.name.split('.').pop()?.toLowerCase() || ''
  return `file-${ext}`
})

function getFileIcon(name: string): string {
  const ext = name.split('.').pop()?.toLowerCase() || ''
  const iconMap: Record<string, string> = {
    'ts': '📘',
    'tsx': '📘',
    'js': '📙',
    'jsx': '📙',
    'vue': '💚',
    'html': '📄',
    'css': '🎨',
    'scss': '🎨',
    'json': '📋',
    'md': '📝',
    'txt': '📝',
    'rs': '🦀',
    'py': '🐍',
    'java': '☕',
    'go': '🐹',
    'yaml': '⚙️',
    'yml': '⚙️',
    'toml': '⚙️',
    'sh': '💻',
    'gitignore': '🙈',
  }
  return iconMap[ext] || '📄'
}

function handleClick() {
  if (!props.entry.isDir) {
    emit('select', props.entry.path)
  } else {
    toggleExpand()
  }
}

function toggleExpand() {
  emit('expand', props.entry.path)
}
</script>

<style scoped>
/* IDEA 风格文件节点 - 支持主题切换 */
.tree-node {
  user-select: none;
}

.node-row {
  display: flex;
  align-items: center;
  height: 22px;
  padding: 0 8px;
  border-radius: 0;
  cursor: pointer;
  transition: background 0.1s;
  font-size: 12px;
  color: var(--color-base-content);
}

.node-row:hover {
  background: color-mix(in oklab, var(--color-base-content) 6%, transparent);
}

.node-row.selected {
  background: color-mix(in oklab, var(--color-primary) 15%, transparent);
}

.node-row.selected:hover {
  background: color-mix(in oklab, var(--color-primary) 20%, transparent);
}

.expand-icon {
  width: 14px;
  height: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 8px;
  color: color-mix(in oklab, var(--color-base-content) 50%, transparent);
  cursor: pointer;
  margin-right: 2px;
}

.expand-icon:hover {
  color: color-mix(in oklab, var(--color-base-content) 70%, transparent);
}

.expand-icon.placeholder {
  visibility: hidden;
}

.file-icon {
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  margin-right: 4px;
}

.file-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 22px;
}

.node-row.selected .file-name {
  color: var(--color-primary);
}
</style>