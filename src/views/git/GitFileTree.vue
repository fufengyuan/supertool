<template>
  <div class="git-file-tree">
    <!-- 搜索框 -->
    <div class="tree-search">
      <input
        v-model="searchQuery"
        type="text"
        placeholder="搜索文件..."
        class="search-input"
      />
    </div>

    <!-- 文件树 -->
    <div class="tree-content" v-if="!loading">
      <TreeNode
        v-for="entry in filteredTree"
        :key="entry.path"
        :entry="entry"
        :depth="0"
        :selected-path="selectedPath"
        @select="handleSelect"
        @expand="handleExpand"
      />
    </div>

    <!-- 加载状态 -->
    <div v-else class="tree-loading">
      <span>加载文件树...</span>
    </div>

    <!-- 空状态 -->
    <div v-if="!loading && filteredTree.length === 0" class="tree-empty">
      <span>没有找到文件</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, inject } from 'vue'
import { tauriCall } from '@/utils/tauri-api'

interface FileTreeEntry {
  path: string
  name: string
  isDir: boolean
  children?: FileTreeEntry[]
}

import TreeNode from './TreeNode.vue'

const props = defineProps<{
  repoPath: string
}>()

const emit = defineEmits<{
  'select-file': [path: string]
}>()

const fileTree = ref<FileTreeEntry[]>([])
const loading = ref(false)
const searchQuery = ref('')
const selectedPath = ref<string | null>(null)
const expandedPaths = ref<Set<string>>(new Set())

// 过滤文件树
const filteredTree = computed(() => {
  if (!searchQuery.value) return fileTree.value
  return filterTree(fileTree.value, searchQuery.value.toLowerCase())
})

function filterTree(entries: FileTreeEntry[], query: string): FileTreeEntry[] {
  return entries.reduce((acc: FileTreeEntry[], entry) => {
    // 文件名匹配
    if (entry.name.toLowerCase().includes(query)) {
      acc.push(entry)
    }
    // 目录：搜索子节点
    if (entry.isDir && entry.children) {
      const filteredChildren = filterTree(entry.children, query)
      if (filteredChildren.length > 0) {
        acc.push({ ...entry, children: filteredChildren })
      }
    }
    return acc
  }, [])
}

// 加载文件树
async function loadFileTree() {
  if (!props.repoPath) return
  loading.value = true
  try {
    const tree = await tauriCall<FileTreeEntry[]>('get_file_tree', { repoPath: props.repoPath })
    fileTree.value = tree
  } catch (err) {
    console.error('加载文件树失败:', err)
    fileTree.value = []
  } finally {
    loading.value = false
  }
}

// 处理选择
function handleSelect(path: string) {
  selectedPath.value = path
  emit('select-file', path)
}

// 处理展开/折叠
function handleExpand(path: string) {
  if (expandedPaths.value.has(path)) {
    expandedPaths.value.delete(path)
  } else {
    expandedPaths.value.add(path)
  }
}

// 监听 repoPath 变化
watch(() => props.repoPath, loadFileTree, { immediate: true })

onMounted(loadFileTree)
</script>

<style scoped>
.git-file-tree {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-base-100);
  border-right: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.tree-search {
  padding: 8px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.search-input {
  width: 100%;
  padding: 6px 10px;
  border-radius: 6px;
  background: var(--color-base-200);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 15%, transparent);
  font-size: 13px;
}

.search-input:focus {
  outline: none;
  border-color: var(--color-primary);
}

.tree-content {
  flex: 1;
  overflow-y: auto;
  padding: 4px;
}

.tree-loading,
.tree-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-size: 13px;
}
</style>