<template>
  <div class="disk-cleaner h-full flex flex-col bg-base-200">
    <!-- 顶部工具栏 -->
    <div class="flex-none bg-base-100 border-b border-base-300 p-4">
      <div class="flex items-center justify-between mb-3">
        <h2 class="text-lg font-bold flex items-center gap-2">
          <SvgIcon name="broom" size="14" /> 磁盘清理
        </h2>
        <div class="flex gap-1 bg-base-200 p-0.5 rounded-lg">
          <button class="btn btn-sm" :class="activeTab === 'directory' ? 'btn-primary' : 'btn-ghost'" @click="activeTab = 'directory'">
            <SvgIcon name="folder" size="14" /> 目录浏览
          </button>
          <button class="btn btn-sm" :class="activeTab === 'category' ? 'btn-primary' : 'btn-ghost'" @click="activeTab = 'category'">
            <SvgIcon name="barChart" size="14" /> 文件分类
          </button>
          <button class="btn btn-sm" :class="activeTab === 'cache' ? 'btn-primary' : 'btn-ghost'" @click="activeTab = 'cache'">
            <SvgIcon name="trash" size="14" /> 缓存清理
          </button>
          <button class="btn btn-sm" :class="activeTab === 'duplicate' ? 'btn-primary' : 'btn-ghost'" @click="activeTab = 'duplicate'">
            <SvgIcon name="search" size="14" /> 重复文件
          </button>
        </div>
      </div>
      <!-- 磁盘信息 -->
      <div v-if="diskInfo.length" class="flex gap-4 text-sm">
        <div v-for="disk in diskInfo" :key="disk.mountPoint" class="flex items-center gap-2">
          <span class="font-mono">{{ disk.mountPoint }}</span>
          <div class="w-48 bg-base-300 rounded-full h-2">
            <div class="h-2 rounded-full transition-all" :class="usageColor(disk.usagePercent ?? 0)" :style="{ width: (disk.usagePercent ?? 0) + '%' }"></div>
          </div>
          <span class="text-xs text-base-content/60">{{ formatSize(disk.used) }} / {{ formatSize(disk.total) }} ({{ (disk.usagePercent ?? 0).toFixed(1) }}%)</span>
        </div>
      </div>
    </div>

    <!-- 选中项操作栏 -->
    <div v-if="selectedPaths.size > 0" class="flex-none bg-warning/10 border-b border-warning/30 px-4 py-2 flex items-center justify-between">
      <span class="text-sm">已选择 <strong>{{ selectedPaths.size }}</strong> 项，可释放 <strong>{{ formatSize(selectedTotalSize) }}</strong></span>
      <div class="flex gap-2">
        <button class="btn btn-sm btn-warning gap-1" @click="deleteSelected" :disabled="deleting">
          <span v-if="deleting" class="loading loading-spinner loading-xs"></span>
          <SvgIcon v-else name="trash" size="14" /> {{ deleting ? '删除中...' : '删除选中' }}
        </button>
        <button class="btn btn-sm btn-ghost" @click="clearSelection">取消</button>
      </div>
    </div>

    <!-- 内容区 -->
    <div class="flex-1 overflow-y-auto p-4">
      <!-- Shared error display -->
      <div v-if="errorMessage" class="alert alert-error mb-3">
        <span>{{ errorMessage }}</span>
        <button class="btn btn-sm btn-ghost" @click="errorMessage = ''"><SvgIcon name="x" size="14" /></button>
      </div>
      <!-- 目录浏览 -->
      <div v-if="activeTab === 'directory'">
        <div class="flex items-center gap-2 mb-4">
          <button class="btn btn-sm btn-ghost" @click="goUp" :disabled="!currentPath || currentPath === '/'">
            <SvgIcon name="arrowUp" size="14" /> 上一级
          </button>
          <div class="breadcrumbs text-sm flex-1">
            <ul>
              <li v-for="(part, idx) in breadcrumbParts" :key="idx" class="cursor-pointer hover:underline" @click="navigateToBreadcrumb(idx)">
                {{ part || '/' }}
              </li>
            </ul>
          </div>
          <div class="join">
            <input type="text" v-model="filterQuery" placeholder="过滤名称…" class="input input-bordered input-sm join-item w-40" />
            <button class="btn btn-sm btn-primary join-item gap-1" @click="scanCurrentDir" :disabled="dirScanning">
              <span v-if="dirScanning" class="loading loading-spinner loading-xs"></span>
              <span v-if="dirScanning">扫描中...</span>
              <span v-else><SvgIcon name="search" size="14" /> 扫描</span>
            </button>
          </div>
        </div>

        <div v-if="dirEntries.length === 0 && !dirScanning" class="text-center py-20 text-base-content/40">
          <div class="mb-4 flex justify-center"><SvgIcon name="folder" size="48" class="text-base-content/20" /></div>
          <p class="text-lg">点击"扫描"开始分析目录</p>
          <p class="text-sm mt-2">默认从当前用户目录开始</p>
        </div>

        <div v-else class="space-y-1">
          <div v-if="filterQuery && filteredDirEntries.length === 0" class="text-center py-10 text-base-content/40">
            <SvgIcon name="search" size="32" class="mx-auto mb-2 text-base-content/20" />
            <p>未找到匹配"{{ filterQuery }}"的条目</p>
          </div>
          <template v-else>
            <div class="flex items-center gap-3 px-3 py-1.5 text-xs text-base-content/40 border-b border-base-300/50">
              <input type="checkbox" class="checkbox checkbox-sm" :checked="selectAllChecked" @change="toggleSelectAll" />
              <span class="flex-1">全选 {{ filteredDirEntries.length }} 项</span>
              <span class="font-mono w-20 text-right">大小</span>
              <span class="w-8"></span>
            </div>
            <div v-for="entry in filteredDirEntries" :key="entry.path" class="flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-base-300/50 cursor-pointer group" @dblclick="onEntryDblClick(entry)">
            <input type="checkbox" class="checkbox checkbox-sm" :checked="selectedPaths.has(entry.path)" @change="toggleSelect(entry.path, entry.size)" @click.stop />
            <span class="shrink-0"><template v-if="entry.fileType === 'directory'"><SvgIcon name="folder" size="16" class="text-base-content/60" /></template><template v-else><SvgIcon :name="getFileIconName(extFromName(entry.name))" size="16" class="text-base-content/60" /></template></span>
            <span class="flex-1 truncate text-sm">{{ entry.name }}</span>
            <span class="text-xs text-base-content/50 font-mono">{{ formatSize(entry.size) }}</span>
            <span v-if="entry.fileType === 'directory'" class="text-xs text-base-content/40">
              {{ entry.childrenCount ? entry.childrenCount + ' 项' : '' }}
            </span>
            <button class="btn btn-xs btn-ghost opacity-0 group-hover:opacity-100" @click.stop="deleteSingle(entry)">删除</button>
          </div>
          </template>
        </div>
      </div>

      <!-- 文件分类 -->
      <div v-if="activeTab === 'category'">
        <div class="flex items-center gap-2 mb-4">
          <input type="text" v-model="categoryScanPath" placeholder="扫描路径（如 /Users）" class="input input-bordered input-sm flex-1" />
          <button class="btn btn-sm btn-primary gap-1" @click="scanCategories" :disabled="catScanning">
            <span v-if="catScanning" class="loading loading-spinner loading-xs"></span>
            <template v-if="catScanning">扫描中...</template><template v-else><SvgIcon name="search" size="14" /> 扫描分类</template>
          </button>
        </div>

        <div v-if="categories.length === 0 && !catScanning" class="text-center py-20 text-base-content/40">
          <div class="mb-4 flex justify-center"><SvgIcon name="barChart" size="48" class="text-base-content/20" /></div>
          <p class="text-lg">按文件类型分类查看大文件</p>
        </div>

        <div v-else class="space-y-4">
          <div v-for="cat in categories" :key="cat.extension" class="bg-base-100 rounded-lg border border-base-300 overflow-hidden">
            <details class="collapse collapse-arrow">
              <summary class="collapse-title flex items-center gap-3 px-4 py-3 cursor-pointer">
                <SvgIcon :name="getCategoryIconName(cat.extension)" size="20" class="text-base-content/60 shrink-0" />
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
          <div class="flex items-center gap-2 text-sm text-base-content/60">
            <SvgIcon name="info" size="14" />
            <span>自动识别的系统缓存路径，已按大小排序</span>
            <span v-if="cacheSafeCount > 0" class="badge badge-sm badge-success gap-1">
              <SvgIcon name="checkCircle" size="10" /> {{ cacheSafeCount }} 项可安全清理（{{ formatSize(cacheSafeTotal) }}）
            </span>
          </div>
          <div class="flex gap-2">
            <button class="btn btn-sm btn-warning gap-1" @click="cleanAllSafe">
              <SvgIcon name="broom" size="14" /> 一键清理安全缓存
            </button>
          </div>
        </div>

        <div v-if="cachePaths.length === 0 && !cacheScanning" class="text-center py-20 text-base-content/40">
          <div class="mb-4 flex justify-center"><SvgIcon name="trash" size="48" class="text-base-content/20" /></div>
          <p class="text-lg">正在检测缓存路径...</p>
        </div>

        <div v-else-if="cacheScanning" class="text-center py-10">
          <span class="loading loading-spinner loading-lg"></span>
          <p class="mt-3 text-sm text-base-content/50">扫描缓存路径...</p>
        </div>

        <div v-else class="space-y-1">
          <div v-for="cache in cachePaths" :key="cache.path" class="flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-base-300/50">
            <input type="checkbox" class="checkbox checkbox-sm" :checked="selectedPaths.has(cache.path)" @change="toggleSelect(cache.path, cache.size)" @click.stop />
            <span class="shrink-0"><template v-if="cache.safeToClean"><SvgIcon name="checkCircle" size="16" class="text-success" /></template><template v-else><SvgIcon name="alertCircle" size="16" class="text-warning" /></template></span>
            <div class="flex-1 min-w-0">
              <div class="font-semibold text-sm">{{ cache.name }}</div>
              <div class="text-xs text-base-content/40 truncate font-mono">{{ cache.path }}</div>
              <div class="text-xs text-base-content/50">{{ cache.description }}</div>
            </div>
            <span class="font-mono text-sm">{{ formatSize(cache.size) }}</span>
            <span v-if="cache.safeToClean" class="badge badge-sm badge-success gap-1 tooltip" data-tip="可安全清理的缓存目录，删除不会影响系统运行">安全</span>
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
            <template v-if="dupScanning">扫描中...</template><template v-else><SvgIcon name="search" size="14" /> 查找重复</template>
          </button>
        </div>

        <div v-if="duplicateGroups.length === 0 && !dupScanning" class="text-center py-20 text-base-content/40">
          <div class="mb-4 flex justify-center"><SvgIcon name="search" size="48" class="text-base-content/20" /></div>
          <p class="text-lg">点击"查找重复"开始扫描</p>
        </div>

        <div v-else-if="dupScanning" class="text-center py-10">
          <span class="loading loading-spinner loading-lg"></span>
          <p class="mt-3 text-sm text-base-content/50">查找重复文件中...</p>
        </div>

        <div v-else-if="duplicateGroups.length === 0" class="text-center py-10 text-success">
          <div class="mb-2 flex justify-center"><SvgIcon name="checkCircle" size="40" /></div>
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
      <div class="modal-box max-w-lg">
        <h3 class="text-lg font-bold flex items-center gap-2">
          <SvgIcon name="alertTriangle" size="18" class="text-warning" />
          <span>确认删除</span>
        </h3>
        <p class="py-2" v-if="pendingDelete">即将删除 <strong>{{ pendingDelete.name }}</strong></p>
        <p class="text-sm text-error flex items-center gap-1 mb-3">
          <SvgIcon name="alertCircle" size="14" /> 删除后无法恢复！
        </p>
        <div v-if="pendingDelete && pendingDelete.paths.length > 0" class="max-h-40 overflow-y-auto bg-base-200 rounded-lg p-2 text-xs font-mono space-y-1">
          <div v-for="p in pendingDelete.paths.slice(0, 20)" :key="p" class="truncate">{{ p }}</div>
          <div v-if="pendingDelete.paths.length > 20" class="text-base-content/40 italic pt-1">…以及另外 {{ pendingDelete.paths.length - 20 }} 项</div>
        </div>
        <div class="modal-action">
          <button class="btn btn-error gap-1" @click="confirmDelete" :disabled="deleting">
            <span v-if="deleting" class="loading loading-spinner loading-xs"></span>
            <SvgIcon name="trash" size="14" /> 确认删除
          </button>
          <button class="btn" @click="deleteDialog?.close()" :disabled="deleting">取消</button>
        </div>
      </div>
      <form method="dialog" class="modal-backdrop"><button>close</button></form>
    </dialog>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'DiskCleaner' })
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { useToast } from '../composables/useToast'

const toast = useToast()

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
const deleting = ref(false)
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

// 搜索过滤
const filterQuery = ref('')
const selectAllChecked = ref(false)

const deleteDialog = ref<HTMLDialogElement | null>(null)
// 单个删除待确认项
const pendingDelete = ref<{ paths: string[]; name: string } | null>(null)

const breadcrumbParts = computed(() => {
  if (!currentPath.value) return []
  const parts = currentPath.value.split('/').filter(Boolean)
  return ['/', ...parts]
})

// 目录过滤结果
const filteredDirEntries = computed(() => {
  if (!filterQuery.value) return dirEntries.value
  const q = filterQuery.value.toLowerCase()
  return dirEntries.value.filter(e => e.name.toLowerCase().includes(q))
})

const cacheSafeCount = computed(() => cachePaths.value.filter(c => c.safeToClean).length)
const cacheSafeTotal = computed(() => cachePaths.value.filter(c => c.safeToClean).reduce((s, c) => s + c.size, 0))

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

// 文件扩展名 → SvgIcon name 映射
function getFileIconName(ext: string): string {
  const map: Record<string, string> = {
    js: 'code', ts: 'code', py: 'code', rs: 'code', go: 'code', java: 'code',
    cpp: 'code', c: 'code', h: 'code', cs: 'code', swift: 'code', kt: 'code',
    jpg: 'camera', jpeg: 'camera', png: 'camera', gif: 'camera', svg: 'camera', webp: 'camera', ico: 'camera', bmp: 'camera',
    mp4: 'film', mkv: 'film', avi: 'film', mov: 'film', webm: 'film', flv: 'film',
    mp3: 'music', wav: 'music', flac: 'music', aac: 'music', ogg: 'music', m4a: 'music',
    pdf: 'file', doc: 'file', docx: 'file', xls: 'file', xlsx: 'file', ppt: 'file', pptx: 'file',
    zip: 'package', rar: 'package', '7z': 'package', tar: 'package', gz: 'package',
    dmg: 'cd', iso: 'cd', img: 'cd',
    log: 'clipboard', tmp: 'ban', temp: 'ban', cache: 'ban',
  }
  return map[ext] || 'file'
}

// 分类 → SvgIcon name
function getCategoryIconName(ext: string): string {
  const map: Record<string, string> = {
    zip: 'package', rar: 'package', '7z': 'package', tar: 'package', gz: 'package', bz2: 'package', xz: 'package',
    mp4: 'film', mkv: 'film', avi: 'film', mov: 'film', wmv: 'film', flv: 'film', webm: 'film',
    mp3: 'music', wav: 'music', flac: 'music', aac: 'music', ogg: 'music', m4a: 'music',
    jpg: 'camera', jpeg: 'camera', png: 'camera', gif: 'camera', bmp: 'camera', svg: 'camera', webp: 'camera', ico: 'camera',
    pdf: 'file', doc: 'file', docx: 'file', xls: 'file', xlsx: 'file', ppt: 'file', pptx: 'file',
    dmg: 'cd', iso: 'cd', img: 'cd',
    apk: 'package', ipa: 'package',
    exe: 'tool', msi: 'tool', app: 'tool',
    js: 'code', ts: 'code', py: 'code', go: 'code', rs: 'code', java: 'code', cpp: 'code', c: 'code', h: 'code',
    log: 'clipboard',
    tmp: 'ban', temp: 'ban', cache: 'ban',
    woff: 'pencil', woff2: 'pencil', ttf: 'pencil', otf: 'pencil', eot: 'pencil',
  }
  return map[ext] || 'file'
}

function extFromName(name: string): string {
  return name.split('.').pop()?.toLowerCase() || ''
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
  selectAllChecked.value = filteredDirEntries.value.length > 0 && filteredDirEntries.value.every(e => selectedPaths.value.has(e.path))
}

function toggleSelectAll() {
  const map = new Map(selectedPaths.value)
  const entries = filteredDirEntries.value
  const allSelected = entries.every(e => map.has(e.path))
  if (allSelected) {
    for (const e of entries) {
      if (map.has(e.path)) {
        map.delete(e.path)
        selectedTotalSize.value -= e.size
      }
    }
  } else {
    for (const e of entries) {
      if (!map.has(e.path)) {
        map.set(e.path, e.size)
        selectedTotalSize.value += e.size
      }
    }
  }
  selectedPaths.value = map
  selectAllChecked.value = !allSelected
}

function clearSelection() {
  selectedPaths.value = new Map()
  selectedTotalSize.value = 0
  selectAllChecked.value = false
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

function showDeleteDialog(paths: string[], name?: string) {
  pendingDelete.value = { paths, name: name || `${paths.length} 项` }
  deleteDialog.value?.showModal()
}

async function confirmDelete() {
  if (!pendingDelete.value) return
  const { paths } = pendingDelete.value
  deleting.value = true
  try {
    const result: any = await invoke('delete_items', { paths })
    const successCount = result.success?.length || 0
    const failedCount = result.failed?.length || 0
    const freedSize = formatSize(result.totalFreed || 0)

    if (successCount > 0 && failedCount === 0) {
      toast.success(`成功删除 ${successCount} 项，释放 ${freedSize}`)
    } else if (successCount > 0 && failedCount > 0) {
      toast.warning(`已删除 ${successCount} 项（释放 ${freedSize}），${failedCount} 项失败`)
    } else if (failedCount > 0) {
      toast.error(`删除失败（${failedCount} 项）`)
    }
    refreshCurrentView()
  } catch (e: any) {
    toast.error('删除失败: ' + (e.message || '未知错误'))
  } finally {
    deleting.value = false
    pendingDelete.value = null
    clearSelection()
    deleteDialog.value?.close()
  }
}

function deleteSelected() {
  const paths = Array.from(selectedPaths.value.keys())
  if (paths.length === 0) {
    toast.warning('请先选择要删除的文件')
    return
  }
  showDeleteDialog(paths)
}

function deleteSingle(entry: DirEntry) {
  showDeleteDialog([entry.path], entry.name)
}

async function cleanAllSafe() {
  const safePaths = cachePaths.value.filter(c => c.safeToClean).map(c => c.path)
  if (safePaths.length === 0) {
    toast.info('没有可安全清理的缓存')
    return
  }
  showDeleteDialog(safePaths, `${safePaths.length} 项安全缓存（${formatSize(cacheSafeTotal.value)}）`)
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
