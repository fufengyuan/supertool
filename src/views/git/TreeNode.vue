<template>
  <div class="tree-node">
    <div
      class="node-row"
      :class="{ selected: selectedPath === entry.path, expanded: isExpanded, loading: isLoadingChildren }"
      :style="{ paddingLeft: depth * 16 + 8 + 'px' }"
      @click="handleClick"
    >
      <!-- 展开/折叠按钮 -->
      <span
        v-if="entry.isDir"
        class="expand-icon"
        @click.stop="toggleExpand"
      >
        <SvgIcon v-if="isLoadingChildren" name="refresh" :size="10" class="animate-spin" />
        <SvgIcon v-else :name="isExpanded ? 'chevronDown' : 'chevronRight'" :size="10" />
      </span>
      <span v-else class="expand-icon placeholder"></span>

      <!-- 文件图标 -->
      <SvgIcon :name="getFileIconName(entry.name, entry.isDir)" :size="14" class="file-icon" :class="iconClass" />

      <!-- 文件名 -->
      <span class="file-name">{{ entry.name }}</span>
    </div>

    <!-- 子节点 -->
    <div v-if="entry.isDir && isExpanded && hasChildren" class="node-children">
      <TreeNode
        v-for="child in children"
        :key="child.path"
        :entry="child"
        :depth="depth + 1"
        :selected-path="selectedPath"
        :repo-path="repoPath"
        @select="$emit('select', $event)"
        @expand="$emit('expand', $event)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, inject, Ref, ref, watch } from 'vue'
import { tauriCall } from '@/utils/tauri-api'
import SvgIcon from '@/components/ui/SvgIcon.vue'

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
  repoPath: string
}>()

const emit = defineEmits<{
  'select': [path: string]
  'expand': [path: string]
}>()

// 从父组件获取 expandedPaths (是一个 Ref<Set<string>>)
const expandedPathsRef = inject<Ref<Set<string>>>('expandedPaths')

// 懒加载的子节点数据
const children = ref<FileTreeEntry[]>([])
const isLoadingChildren = ref(false)

const isExpanded = computed(() => expandedPathsRef?.value?.has(props.entry.path) || false)

// 判断是否有子节点（已加载或需要加载）
const hasChildren = computed(() => {
  if (props.entry.children && props.entry.children.length > 0) {
    return true
  }
  if (children.value && children.value.length > 0) {
    return true
  }
  return isExpanded.value // 展开状态时可能有子节点（懒加载后）
})

const iconClass = computed(() => {
  if (props.entry.isDir) {return 'directory'}
  const ext = props.entry.name.split('.').pop()?.toLowerCase() || ''
  return `file-${ext}`
})

function getFileIconName(name: string, isDir: boolean): string {
  if (isDir) {return 'folder'}
  const ext = name.split('.').pop()?.toLowerCase() || ''
  const iconMap: Record<string, string> = {
    'ts': 'file',
    'tsx': 'file',
    'js': 'file',
    'jsx': 'file',
    'vue': 'file',
    'html': 'file',
    'css': 'file',
    'scss': 'file',
    'json': 'file',
    'md': 'file',
    'txt': 'file',
    'rs': 'file',
    'py': 'file',
    'java': 'file',
    'go': 'file',
    'yaml': 'file',
    'yml': 'file',
    'toml': 'file',
    'sh': 'file',
    'gitignore': 'file',
  }
  return iconMap[ext] || 'file'
}

function handleClick() {
  if (!props.entry.isDir) {
    emit('select', props.entry.path)
  } else {
    toggleExpand()
  }
}

async function toggleExpand() {
  const expandedPaths = expandedPathsRef?.value
  if (!expandedPaths) {return}
  
  const path = props.entry.path
  
  if (expandedPaths.has(path)) {
    // 折叠：移除展开状态
    expandedPaths.delete(path)
  } else {
    // 展开：添加展开状态，懒加载子节点
    expandedPaths.add(path)
    
    // 如果 entry.children 为空或未定义，需要懒加载
    if (!props.entry.children || props.entry.children.length === 0) {
      if (!children.value || children.value.length === 0) {
        await loadChildren()
      }
    }
  }
}

async function loadChildren() {
  if (!props.entry.isDir || isLoadingChildren.value) {return}
  
  isLoadingChildren.value = true
  try {
    const subChildren = await tauriCall<FileTreeEntry[]>('get_file_tree', {
      repoPath: props.repoPath,
      subdir: props.entry.path
    })
    children.value = subChildren || []
  } catch (err) {
    console.error('加载子目录失败:', err)
    children.value = []
  } finally {
    isLoadingChildren.value = false
  }
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

.node-row.loading {
  opacity: 0.7;
}

.expand-icon {
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  margin-right: 2px;
  color: color-mix(in oklab, var(--color-base-content) 50%, transparent);
}

.expand-icon:hover {
  color: color-mix(in oklab, var(--color-base-content) 70%, transparent);
}

.expand-icon.placeholder {
  visibility: hidden;
}

.file-icon {
  margin-right: 4px;
  flex-shrink: 0;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.node-row.selected .file-icon {
  color: var(--color-primary);
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