<template>
  <div class="disk-cleaner h-full flex flex-col bg-base-200">
    <!-- 顶部工具栏 -->
    <div class="flex-none bg-base-100 border-b border-base-300 p-4">
      <div class="flex items-center justify-between mb-3">
        <h2 class="text-lg font-bold flex items-center gap-2">
          🧹 磁盘清理
        </h2>
        <div class="flex gap-2">
          <button class="btn btn-sm btn-outline gap-1" @click="activeTab = 'directory'" :class="{ 'btn-primary': activeTab === 'directory' }">
            📂 目录浏览
          </button>
          <button class="btn btn-sm btn-outline gap-1" @click="activeTab = 'category'" :class="{ 'btn-primary': activeTab === 'category' }">
            📊 文件分类
          </button>
          <button class="btn btn-sm btn-outline gap-1" @click="activeTab = 'cache'" :class="{ 'btn-primary': activeTab === 'cache' }">
          🗑️ 缓存清理
          </button>
          <button class="btn btn-sm btn-outline gap-1" @click="activeTab = 'duplicate'" :class="{ 'btn-primary': activeTab === 'duplicate' }">
            🔍 重复文件
          </button>
        </div>
      </div>
      <!-- 磁盘信息 -->
      <div v-if="diskInfo.length" class="flex gap-4 text-sm">
        <div v-for="disk in diskInfo" :key="disk.mountPoint" class="flex items-center gap-2">
          <span class="font-mono">{{ disk.mountPoint }}</span>
          <div class="w-48 bg-base-300 rounded-full h-2">
            <div class="h-2 rounded-full transition-all" :class="usageColor(disk.usagePercent)" :style="{ width: disk.usagePercent + '%' }"></div>
          </div>
          <span class="text-xs text-base-content/60">{{ formatSize(disk.used) }} / {{ formatSize(disk.total) }} ({{ disk.usagePercent.toFixed(1) }}%)</span>
        </div>
      </div>
    </div>

    <!-- 选中项操作栏 -->
    <div v-if="selectedPaths.size > 0" class="flex-none bg-warning/10 border-b border-warning/30 px-4 py-2 flex items-center justify-between">
      <span class="text-sm">已选择 <strong>{{ selectedPaths.size }}</strong> 项，可释放 <strong>{{ formatSize(selectedTotalSize) }}</strong></span>
      <div class="flex gap-2">
        <button class="btn btn-sm btn-warning gap-1" @click="deleteSelected">
          🗑️ 删除选中
        </button>
        <button class="btn btn-sm btn-ghost" @click="clearSelection">取消</button>
      </div>
    </div>

    <!-- 内容区 -->
    <div class="flex-1 overflow-y-auto p-4">
      <!-- Shared error display -->
      <div v-if="errorMessage" class="alert alert-error mb-3">
        <span>{{ errorMessage }}</span>
        <button class="btn btn-sm btn-ghost" @click="errorMessage = ''">✕</button>
      </div>
      <!-- 目录浏览 -->
      <div v-if="activeTab === 'directory'">
        <div class="flex items-center gap-2 mb-4">
          <button class="btn btn-sm btn-ghost" @click="goUp" :disabled="!currentPath || currentPath === '/'">
            ⬆️ 上一级
          </button>
          <div class="breadcrumbs text-sm flex-1">
            <ul>
              <li v-for="(part, idx) in breadcrumbParts" :key="idx" class="cursor-pointer hover:underline" @click="navigateToBreadcrumb(idx)">
                {{ part || '/' }}
              </li>
            </ul>
          </div>
          <button class="btn btn-sm btn-primary gap-1" @click="scanCurrentDir" :disabled="dirScanning">
            <span v-if="dirScanning" class="loading loading-spinner loading-xs"></span>
            {{ dirScanning ? '扫描中...' : '🔍 扫描' }}
          </button>
        </div>

        <div v-if="dirEntries.length === 0 && !dirScanning" class="text-center py-20 text-base-content/40">
          <div class="text-6xl mb-4">📂</div>
          <p class="text-lg">点击"扫描"开始分析目录</p>
          <p class="text-sm mt-2">默认从当前用户目录开始</p>
        </div>

        <div v-else class="space-y-1">
          <div v-for="entry in dirEntries" :key="entry.path" class="flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-base-300/50 cursor-pointer group" @dblclick="onEntryDblClick(entry)">
            <input type="checkbox" class="checkbox checkbox-sm" :checked="selectedPaths.has(entry.path)" @change="toggleSelect(entry.path, entry.size)" @click.stop />
            <span class="text-lg">{{ entry.fileType === 'directory' ? '📁' : getFileIcon(entry.name) }}</span>
            <span class="flex-1 truncate text-sm">{{ entry.name }}</span>
            <span class="text-xs text-base-content/50 font-mono">{{ formatSize(entry.size) }}</span>
            <span v-if="entry.fileType === 'directory'" class="text-xs text-base-content/40">
              {{ entry.childrenCount ? entry.childrenCount + ' 项' : '' }}
            </span>
            <button class="btn btn-xs btn-ghost opacity-0 group-hover:opacity-100" @click.stop="deleteSingle(entry)">删除</button>
          </div>
        </div>
      </div>

      <!-- 文件分类 -->
      <div v-if="activeTab === 'category'">
        <div class="flex items-center gap-2 mb-4">
          <input type="text" v-model="categoryScanPath" placeholder="扫描路径（如 /Users）" class="input input-bordered input-sm flex-1" />
          <button class="btn btn-sm btn-primary gap-1" @click="scanCategories" :disabled="catScanning">
            <span v-if="catScanning" class="loading loading-spinner loading-xs"></span>
            {{ catScanning ? '扫描中...' : '🔍 扫描分类' }}
          </button>
        </div>

        <div v-if="categories.length === 0 && !catScanning" class="text-center py-20 text-base-content/40">
          <div class="text-6xl mb-4">📊</div>
          <p class="text-lg">按文件类型分类查看大文件</p>
        </div>

        <div v-else class="space-y-4">
          <div v-for="cat in categories" :key="cat.extension" class="bg-base-100 rounded-lg border border-base-300 overflow-hidden">
            <details class="collapse collapse-arrow">
              <summary class="collapse-title flex items-center gap-3 px-4 py-3 cursor-pointer">
                <span class="text-2xl">{{ cat.icon }}</span>
                <div class="flex-1">
                  <div class="font-semibold">{{ cat.label }}</div>
                  <div class="text-xs text-base-content/50">.{{ cat.extension }} · {{ cat.count }} 个文件</div>
                </div>
                <span class="font-mono text-sm">{{ formatSize(cat.totalSize) }}</span>
              </summary>
              <div class="collapse-content">
                <div class="space-y-1">
                  <div v-for="file in cat.files" :key="file.path" class="flex items-center gap-2 px-2 py-1 rounded hover:bg-base-200">
                    <input type="checkbox" class="checkbox checkbox-xs" :checked="selectedPaths.has(file.path)" @change="toggleSelect(file.path, file.size)" @click.stop />
                    <span class="flex-1 truncate text-xs font-mono">{{ file.path }}</span>
                    <span class="text-xs text-base-content/50">{{ formatSize(file.size) }}</span>
                  </div>
                </div>
              </div>
            </details>
          </div>
        </div>
      </div>

      <!-- 缓存清理 -->
      <div v-if="activeTab === 'cache'">
        <div class="flex items-center justify-between mb-4">
          <p class="text-sm text-base-content/60">自动识别的系统缓存路径，已按大小排序</p>
          <div class="flex gap-2">
            <button class="btn btn-sm btn-warning gap-1" @click="cleanAllSafe">
              🧹 一键清理安全缓存
            </button>
          </div>
        </div>

        <div v-if="cachePaths.length === 0 && !cacheScanning" class="text-center py-20 text-base-content/40">
          <div class="text-6xl mb-4">🗑️</div>
          <p class="text-lg">正在检测缓存路径...</p>
        </div>

        <div v-else-if="cacheScanning" class="text-center py-10">
          <span class="loading loading-spinner loading-lg"></span>
          <p class="mt-3 text-sm text-base-content/50">扫描缓存路径...</p>
        </div>

        <div v-else class="space-y-1">
          <div v-for="cache in cachePaths" :key="cache.path" class="flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-base-300/50">
            <input type="checkbox" class="checkbox checkbox-sm" :checked="selectedPaths.has(cache.path)" @change="toggleSelect(cache.path, cache.size)" @click.stop />
            <span class="text-lg">{{ cache.safeToClean ? '🟢' : '🟡' }}</span>
            <div class="flex-1 min-w-0">
              <div class="font-semibold text-sm">{{ cache.name }}</div>
              <div class="text-xs text-base-content/40 truncate font-mono">{{ cache.path }}</div>
              <div class="text-xs text-base-content/50">{{ cache.description }}</div>
            </div>
            <span class="font-mono text-sm">{{ formatSize(cache.size) }}</span>
            <span v-if="cache.safeToClean" class="badge badge-sm badge-success gap-1">安全</span>
          </div>
        </div>
      </div>

      <!-- 重复文件 -->
      <div v-if="activeTab === 'duplicate'">
        <div class="flex items-center gap-2 mb-4">
          <input type="text" v-model="dupScanPath" placeholder="扫描路径" class="input input-bordered input-sm flex-1" />
          <input type="number" v-model.number="dupMinSize" placeholder="最小大小(KB)" class="input input-bordered input-sm w-32" />
          <button class="btn btn-sm btn-primary gap-1" @click="scanDuplicates" :disabled="dupScanning">
            <span v-if="dupScanning" class="loading loading-spinner loading-xs"></span>
            {{ dupScanning ? '扫描中...' : '🔍 查找重复' }}
          </button>
        </div>

        <div v-if="duplicateGroups.length === 0 && !dupScanning" class="text-center py-20 text-base-content/40">
          <div class="text-6xl mb-4">🔍</div>
          <p class="text-lg">点击"查找重复"开始扫描</p>
        </div>

        <div v-else-if="dupScanning" class="text-center py-10">
          <span class="loading loading-spinner loading-lg"></span>
          <p class="mt-3 text-sm text-base-content/50">查找重复文件中...</p>
        </div>

        <div v-else-if="duplicateGroups.length === 0" class="text-center py-10 text-success">
          <div class="text-4xl mb-2">✅</div>
          <p>未找到重复文件</p>
        </div>

        <div v-else class="space-y-4">
          <div v-for="(group, idx) in duplicateGroups" :key="idx" class="bg-base-100 rounded-lg border border-base-300 p-4">
            <div class="flex items-center justify-between mb-2">
              <span class="font-semibold text-sm">{{ group.files[0]?.name }}</span>
              <span class="text-xs text-error">浪费空间: {{ formatSize(group.wastedSpace) }}</span>
            </div>
            <div class="text-xs text-base-content/40 mb-2">共 {{ group.files.length }} 份副本 · 每份 {{ formatSize(group.files[0]?.size || 0) }}</div>
            <div class="space-y-1">
              <div v-for="file in group.files" :key="file.path" class="flex items-center gap-2 px-2 py-1 rounded hover:bg-base-200">
                <input type="checkbox" class="checkbox checkbox-xs" :checked="selectedPaths.has(file.path)" @change="toggleSelect(file.path, file.size)" @click.stop />
                <span class="flex-1 truncate text-xs font-mono">{{ file.path }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 删除确认对话框 -->
    <dialog ref="deleteDialog" class="modal">
      <div class="modal-box">
        <h3 class="text-lg font-bold">⚠️ 确认删除</h3>
        <p class="py-4">即将删除 <strong>{{ selectedPaths.size }}</strong> 项，释放 <strong>{{ formatSize(selectedTotalSize) }}</strong> 空间。</p>
        <p class="text-sm text-error">删除后无法恢复！</p>
        <div class="modal-action">
          <button class="btn btn-error" @click="confirmDelete">确认删除</button>
          <button class="btn" @click="deleteDialog?.close()">取消</button>
        </div>
      </div>
      <form method="dialog" class="modal-backdrop"><button>close</button></form>
    </dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface DirEntry { path: string; name: string; size: number; fileType: string; modified?: number; childrenCount?: number }
interface FileCategory { extension: string; icon: string; label: string; count: number; totalSize: number; files: DirEntry[] }
interface CachePath { path: string; name: string; description: string; size: number; safeToClean: boolean }
interface DuplicateGroup { key: string; files: DirEntry[]; totalSize: number; wastedSpace: number }
interface DiskInfo { mountPoint: string; total: number; used: number; free: number; usagePercent: number }

const activeTab = ref('directory')
const dirScanning = ref(false)
const catScanning = ref(false)
const cacheScanning = ref(false)
const dupScanning = ref(false)
const errorMessage = ref('')
const currentPath = ref('/')
const defaultScanPath = ref('/')
const dirEntries = ref<DirEntry[]>([])
const categories = ref<FileCategory[]>([])
const cachePaths = ref<CachePath[]>([])
const duplicateGroups = ref<DuplicateGroup[]>([])
const diskInfo = ref<DiskInfo[]>([])
const selectedPaths = ref<Map<string, number>>(new Map())
const selectedTotalSize = ref(0)

const categoryScanPath = ref('/')
const dupScanPath = ref('/')
const dupMinSize = ref(100) // KB

const deleteDialog = ref<HTMLDialogElement | null>(null)

const breadcrumbParts = computed(() => {
  if (!currentPath.value) return []
  const parts = currentPath.value.split('/').filter(Boolean)
  return ['/', ...parts]
})

function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return (bytes / Math.pow(1024, i)).toFixed(i > 1 ? 1 : 0) + ' ' + units[i]
}

function usageColor(percent: number): string {
  if (percent > 90) return 'bg-error'
  if (percent > 70) return 'bg-warning'
  return 'bg-success'
}

function getFileIcon(name: string): string {
  const ext = name.split('.').pop()?.toLowerCase() || ''
  const map: Record<string, string> = {
    js: '📜', ts: '📜', py: '🐍', rs: '🦀', go: '🔵', java: '☕',
    jpg: '🖼️', jpeg: '🖼️', png: '🖼️', gif: '🖼️', svg: '🖼️',
    mp4: '🎬', mp3: '🎵', pdf: '📄', doc: '📝', docx: '📝',
    zip: '📦', rar: '📦', dmg: '💿', iso: '💿',
  }
  return map[ext] || '📄'
}

function toggleSelect(path: string, size: number) {
  const map = new Map(selectedPaths.value)
  if (map.has(path)) {
    map.delete(path)
    selectedTotalSize.value -= size
  } else {
    map.set(path, size)
    selectedTotalSize.value += size
  }
  selectedPaths.value = map
}

function clearSelection() {
  selectedPaths.value = new Map()
  selectedTotalSize.value = 0
}

async function scanCurrentDir() {
  dirScanning.value = true
  errorMessage.value = ''
  try {
    dirEntries.value = await invoke('scan_directory', { path: currentPath.value })
  } catch (e: any) {
    errorMessage.value = e.message || '扫描失败'
  } finally {
    dirScanning.value = false
  }
}

function onEntryDblClick(entry: DirEntry) {
  if (entry.fileType === 'directory') {
    currentPath.value = entry.path
    scanCurrentDir()
  }
}

function goUp() {
  if (!currentPath.value || currentPath.value === '/') return
  const parts = currentPath.value.split('/').filter(Boolean)
  parts.pop()
  currentPath.value = '/' + parts.join('/')
  scanCurrentDir()
}

function navigateToBreadcrumb(idx: number) {
  if (idx === 0) {
    currentPath.value = '/'
  } else {
    currentPath.value = '/' + breadcrumbParts.value.slice(1, idx + 1).join('/')
  }
  scanCurrentDir()
}

async function scanCategories() {
  catScanning.value = true
  errorMessage.value = ''
  try {
    categories.value = await invoke('scan_by_category', { path: categoryScanPath.value, limit: 50 })
  } catch (e: any) {
    errorMessage.value = e.message || '分类扫描失败'
  } finally {
    catScanning.value = false
  }
}

async function loadCachePaths() {
  cacheScanning.value = true
  try {
    cachePaths.value = await invoke('get_cache_paths')
  } catch (e: any) {
    errorMessage.value = '缓存路径加载失败: ' + (e.message || '')
  } finally {
    cacheScanning.value = false
  }
}

async function scanDuplicates() {
  dupScanning.value = true
  errorMessage.value = ''
  try {
    const minSizeBytes = (dupMinSize.value || 100) * 1024
    duplicateGroups.value = await invoke('find_duplicates', { path: dupScanPath.value, minSize: minSizeBytes })
  } catch (e: any) {
    errorMessage.value = e.message || '重复文件扫描失败'
  } finally {
    dupScanning.value = false
  }
}

async function loadDiskInfo() {
  try {
    diskInfo.value = await invoke('get_disk_info')
  } catch {}
}

function deleteSelected() {
  deleteDialog.value?.showModal()
}

async function confirmDelete() {
  const paths = Array.from(selectedPaths.value.keys())
  try {
    const result: any = await invoke('delete_items', { paths })
    const successCount = result.success?.length || 0
    const failedCount = result.failed?.length || 0
    const freedSize = formatSize(result.totalFreed || 0)

    if (successCount > 0 && failedCount === 0) {
      alert(`成功删除 ${successCount} 项，释放 ${freedSize}`)
    } else if (successCount > 0 && failedCount > 0) {
      const failedList = result.failed.map((f: any) => `${f.path}: ${f.reason}`).join('\n')
      alert(`成功 ${successCount} 项，释放 ${freedSize}\n失败 ${failedCount} 项：\n${failedList}`)
    } else if (failedCount > 0) {
      const failedList = result.failed.map((f: any) => `${f.path}: ${f.reason}`).join('\n')
      alert(`全部删除失败（${failedCount} 项）：\n${failedList}`)
    }
    // 刷新当前视图
    refreshCurrentView()
  } catch (e: any) {
    alert('删除失败: ' + e.message)
  } finally {
    clearSelection()
    deleteDialog.value?.close()
  }
}

async function deleteSingle(entry: DirEntry) {
  if (!confirm(`确认删除 "${entry.name}"？`)) return
  try {
    await invoke('delete_items', { paths: [entry.path] })
    refreshCurrentView()
  } catch (e: any) {
    alert('删除失败: ' + e.message)
  }
}

async function cleanAllSafe() {
  const safePaths = cachePaths.value.filter(c => c.safeToClean).map(c => c.path)
  if (safePaths.length === 0) {
    alert('没有可安全清理的缓存')
    return
  }
  const totalSize = cachePaths.value.filter(c => c.safeToClean).reduce((s, c) => s + c.size, 0)
  if (!confirm(`确认清理 ${safePaths.length} 项安全缓存，释放 ${formatSize(totalSize)}？\n清理后无法恢复！`)) return
  try {
    const result: any = await invoke('delete_items', { paths: safePaths })
    alert(`已清理 ${result.success?.length || 0} 项，释放 ${formatSize(result.totalFreed || 0)}`)
    loadCachePaths()
  } catch (e: any) {
    alert('清理失败: ' + e.message)
  }
}

function refreshCurrentView() {
  if (activeTab.value === 'directory') scanCurrentDir()
  else if (activeTab.value === 'cache') loadCachePaths()
  else if (activeTab.value === 'category') scanCategories()
  else if (activeTab.value === 'duplicate') scanDuplicates()
}

onMounted(async () => {
  loadDiskInfo()
  loadCachePaths()
  // Detect home directory for default scan paths
  try {
    const home = await invoke<string>('get_home_dir')
    if (home) {
      defaultScanPath.value = home
      categoryScanPath.value = home
      dupScanPath.value = home
      currentPath.value = home
    }
  } catch {}
})
</script>
