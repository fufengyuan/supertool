<template>
  <div class="git-file-tree">
    <!-- 搜索框 -->
    <div class="tree-search">
      <input
        v-model="searchQuery"
        type="text"
        placeholder="搜索文件..."
        class="search-input"
        @input="handleSearch"
      />
    </div>

    <!-- 文件树 -->
    <div class="tree-content" v-if="!loading">
      <BaseTree
        ref="treeRef"
        :value="filteredTree"
        :children-key="childrenKey"
        :text-key="textKey"
        :indent="16"
        :virtualization="true"
        :virtualization-prerender-count="20"
        :default-open="false"
        v-slot="{ stat }"
        @open:node="handleOpenNode"
        @click:node="handleClickNode"
      >
        <div
          class="tree-node-row"
          :class="{
            selected: selectedPath === stat.data.path,
            'is-directory': stat.data.isDir
          }"
        >
          <!-- 展开/折叠图标 -->
          <span class="expand-icon" @click.stop="toggleNode(stat)">
            <SvgIcon
              v-if="stat.data.isDir && isLoading(stat.data.path)"
              name="refresh"
              :size="10"
              class="animate-spin"
            />
            <SvgIcon
              v-else-if="stat.data.isDir"
              :name="stat.open ? 'chevronDown' : 'chevronRight'"
              :size="10"
            />
          </span>

          <!-- 文件图标 -->
          <SvgIcon
            :name="getFileIcon(stat.data.name, stat.data.isDir)"
            :size="14"
            class="file-icon"
          />

          <!-- 文件名 -->
          <span class="file-name">{{ stat.data.name }}</span>
        </div>
      </BaseTree>
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
import { ref, computed, onMounted, watch } from 'vue'
import { BaseTree, walkTreeData } from '@he-tree/vue'
import '@he-tree/vue/style/default.css'
import { tauriCall } from '@/utils/tauri-api'
import SvgIcon from '@/components/ui/SvgIcon.vue'

interface FileTreeEntry {
  path: string
  name: string
  isDir: boolean
  children?: FileTreeEntry[]
}

const props = defineProps<{
  repoPath: string
}>()

const emit = defineEmits<{
  'select-file': [path: string]
}>()

// 配置
const childrenKey = 'children'
const textKey = 'name'

// 状态
const treeRef = ref<InstanceType<typeof BaseTree> | null>(null)
const fileTree = ref<FileTreeEntry[]>([])
const loading = ref(false)
const searchQuery = ref('')
const selectedPath = ref<string | null>(null)
const loadingPaths = ref<Set<string>>(new Set())

// 搜索过滤后的树
const filteredTree = computed(() => {
  if (!searchQuery.value) return fileTree.value
  return filterTree(fileTree.value, searchQuery.value.toLowerCase())
})

// 过滤树数据
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

// 处理搜索
function handleSearch() {
  // 搜索时展开所有匹配的节点
  if (searchQuery.value && treeRef.value) {
    treeRef.value.openAll()
  }
}

// 加载文件树
async function loadFileTree() {
  if (!props.repoPath) return
  loading.value = true
  try {
    const tree = await tauriCall<FileTreeEntry[]>('get_file_tree', { repoPath: props.repoPath })
    fileTree.value = tree || []
  } catch (err) {
    console.error('加载文件树失败:', err)
    fileTree.value = []
  } finally {
    loading.value = false
  }
}

// 判断路径是否正在加载
function isLoading(path: string): boolean {
  return loadingPaths.value.has(path)
}

// 切换节点展开/折叠
function toggleNode(stat: any) {
  if (!stat.data.isDir) return

  const tree = treeRef.value
  if (!tree) return

  if (stat.open) {
    // 折叠
    tree.getStat(stat.data).open = false
  } else {
    // 展开
    handleOpenNode({ stat })
  }
}

// 处理节点展开（懒加载）
async function handleOpenNode({ stat }: { stat: any }) {
  const entry = stat.data as FileTreeEntry
  if (!entry.isDir) return

  // 已有子节点，不需要加载
  if (entry.children && entry.children.length > 0) return

  // 正在加载
  if (loadingPaths.value.has(entry.path)) return

  loadingPaths.value.add(entry.path)

  try {
    const subChildren = await tauriCall<FileTreeEntry[]>('get_file_tree', {
      repoPath: props.repoPath,
      subdir: entry.path
    })

// 更新节点的 children（原始数据）
      entry.children = subChildren
  } catch (err) {
    console.error('加载子目录失败:', err)
    entry.children = []
  } finally {
    loadingPaths.value.delete(entry.path)
  }
}

// 处理节点点击
function handleClickNode({ stat }: { stat: any }) {
  const entry = stat.data as FileTreeEntry

  if (!entry.isDir) {
    // 文件：选中并发送事件
    selectedPath.value = entry.path
    emit('select-file', entry.path)
  } else {
    // 目录：切换展开
    toggleNode(stat)
  }
}

// 获取文件图标
function getFileIcon(name: string, isDir: boolean): string {
  if (isDir) return 'folder'

  const ext = name.split('.').pop()?.toLowerCase() || ''
  const iconMap: Record<string, string> = {
    ts: 'file',
    tsx: 'file',
    js: 'file',
    jsx: 'file',
    vue: 'file',
    html: 'file',
    css: 'file',
    scss: 'file',
    json: 'file',
    md: 'file',
    txt: 'file',
    rs: 'file',
    py: 'file',
    java: 'file',
    go: 'file',
    yaml: 'file',
    yml: 'file',
    toml: 'file',
    sh: 'file',
    gitignore: 'file',
    png: 'file',
    jpg: 'file',
    jpeg: 'file',
    gif: 'file',
    svg: 'file',
    ico: 'file'
  }
  return iconMap[ext] || 'file'
}

// 监听 repoPath 变化
watch(() => props.repoPath, loadFileTree, { immediate: true })

onMounted(loadFileTree)
</script>

<style scoped>
/* IDEA 风格文件树 - 支持主题切换 */
.git-file-tree {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-base-200);
  border-right: 1px solid color-mix(in oklab, var(--color-base-content) 8%, transparent);
  color: var(--color-base-content);
}

.tree-search {
  padding: 4px 8px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 8%, transparent);
  background: var(--color-base-200);
}

.search-input {
  width: 100%;
  padding: 4px 8px;
  border-radius: 3px;
  background: var(--color-base-100);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 12%, transparent);
  font-size: 12px;
  color: var(--color-base-content);
}

.search-input:focus {
  outline: none;
  border-color: var(--color-primary);
}

.search-input::placeholder {
  color: color-mix(in oklab, var(--color-base-content) 40%, transparent);
}

.tree-content {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}

.tree-loading,
.tree-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: color-mix(in oklab, var(--color-base-content) 50%, transparent);
  font-size: 12px;
}

/* 自定义节点样式 - Octotree 风格 */
.tree-node-row {
  display: flex;
  align-items: center;
  height: 22px;
  padding: 0 4px;
  cursor: pointer;
  font-size: 12px;
  color: var(--color-base-content);
  transition: background 0.1s;
}

.tree-node-row:hover {
  background: color-mix(in oklab, var(--color-base-content) 6%, transparent);
}

.tree-node-row.selected {
  background: color-mix(in oklab, var(--color-primary) 15%, transparent);
}

.tree-node-row.selected:hover {
  background: color-mix(in oklab, var(--color-primary) 20%, transparent);
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

.file-icon {
  margin-right: 4px;
  flex-shrink: 0;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.tree-node-row.selected .file-icon {
  color: var(--color-primary);
}

.file-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 22px;
}

.tree-node-row.selected .file-name {
  color: var(--color-primary);
}

/* 覆盖 he-tree 默认样式 */
.he-tree {
  font-size: 12px;
}

.he-tree .tree-node {
  padding: 0 !important;
}

.he-tree .he-tree__open-icon {
  display: none; /* 我们自己渲染展开图标 */
}
</style>