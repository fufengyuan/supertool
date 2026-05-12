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
import { computed, inject, ref } from 'vue'

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

const expandedPaths = inject<Set<string>>('expandedPaths', new Set())

const isExpanded = computed(() => expandedPaths.has(props.entry.path))

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
.tree-node {
  user-select: none;
}

.node-row {
  display: flex;
  align-items: center;
  height: 24px;
  padding: 2px 8px 2px 0;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.15s;
}

.node-row:hover {
  background: color-mix(in oklab, var(--color-base-content) 8%, transparent);
}

.node-row.selected {
  background: color-mix(in oklab, var(--color-primary) 15%, transparent);
}

.expand-icon {
  width: 16px;
  font-size: 10px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  cursor: pointer;
}

.expand-icon.placeholder {
  visibility: hidden;
}

.file-icon {
  width: 20px;
  font-size: 14px;
  margin-right: 4px;
}

.file-name {
  font-size: 13px;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.node-children {
  /* 子节点样式 */
}
</style>