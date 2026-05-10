<template>
  <div class="flex flex-1 flex-col overflow-y-auto p-5 px-6 min-h-0">
    <div class="mb-4 shrink-0">
      <h3 class="m-0 mb-1 text-lg font-semibold"><SvgIcon name="folder" size="14" class="inline-block align-text-bottom" /> 数据库备份</h3>
      <p class="m-0 text-xs text-base-content/60">备份和还原数据库结构和数据（.nb3 格式）</p>
    </div>

    <div class="flex flex-1 gap-6 min-h-0" v-if="!backupInProgress">
      <!-- Left: Config + Object Selection -->
      <div class="flex flex-1 flex-col min-w-0">
        <div class="flex shrink-0 items-end gap-3 mb-4">
          <div class="flex-1 min-w-0">
            <label class="mb-1.5 block text-xs font-medium text-base-content/60">连接</label>
            <select v-model="localConnectionId" @change="onConnectionChange" class="select select-bordered select-sm w-full">
              <option value="">选择连接</option>
              <option v-for="conn in nonRedisConnections" :key="conn.id" :value="conn.id">
                {{ conn.name }} ({{ conn.type }})
              </option>
            </select>
          </div>
          <div class="flex-1 min-w-0">
            <label class="mb-1.5 block text-xs font-medium text-base-content/60">数据库</label>
            <select v-model="selectedDb" @change="loadObjects" :disabled="!localConnectionId || loadingObjects" class="select select-bordered select-sm w-full">
              <option value="">{{ loadingObjects ? '加载中...' : '选择数据库' }}</option>
              <option v-for="db in databases" :key="db" :value="db">{{ db }}</option>
            </select>
          </div>
          <div class="flex shrink-0 flex-col items-center gap-1">
            <button @click="createBackup" :disabled="!canBackup || creating" class="btn btn-primary whitespace-nowrap">
              <template v-if="creating">备份中...</template>
              <template v-else><SvgIcon name="save" size="14" /> 新建备份</template>
            </button>
            <span v-if="selectedCount > 0" class="text-[11px] text-base-content/60">已选 {{ selectedCount }} 项</span>
          </div>
        </div>

        <!-- Object Selection -->
        <div v-if="objects.length > 0" class="flex flex-1 flex-col overflow-hidden rounded-lg border border-base-content/10 min-h-0">
          <div class="flex shrink-0 items-center justify-between border-b border-base-content/10 bg-base-200 px-3.5 py-2.5 text-sm font-semibold">
            <span>选择要备份的对象</span>
            <div class="flex gap-1">
              <button @click="selectAll" class="btn btn-ghost btn-xs">全选</button>
              <button @click="selectNone" class="btn btn-ghost btn-xs">清空</button>
            </div>
          </div>
          <div class="flex flex-1 flex-col overflow-y-auto py-1">
            <!-- Tables -->
            <div v-if="tables.length > 0" class="border-t border-base-content/10 first:border-t-0">
              <div class="sticky top-0 z-[1] bg-base-100 px-3.5 py-1.5 text-[11px] font-semibold text-base-content/60"><SvgIcon name="barChart" size="14" />  表 ({{ tables.length }})</div>
              <div class="grid grid-cols-[repeat(auto-fill,minmax(180px,1fr))] gap-0.5 p-1 px-2">
                <label v-for="table in tables" :key="'t-' + table"
                  class="flex cursor-pointer items-center gap-1.5 rounded px-2 py-1 text-xs transition-colors duration-100 hover:bg-base-200"
                  :class="isSelected(table) ? 'bg-primary/10' : ''">
                  <input type="checkbox" :checked="isSelected(table)" @change="toggleObject(table, 'table')" class="checkbox checkbox-xs" />
                  <span class="min-w-0 flex-1 truncate font-medium">{{ table }}</span>
                  <span v-if="isSelected(table)" class="flex shrink-0 items-center gap-0.5 text-[10px] text-base-content/60">
                    <input type="checkbox" :checked="includeData(table)" @change="toggleData(table)" class="checkbox checkbox-xs" />
                    <span>含数据</span>
                  </span>
                </label>
              </div>
            </div>

            <!-- Views -->
            <div v-if="views.length > 0" class="border-t border-base-content/10 first:border-t-0">
              <div class="sticky top-0 z-[1] bg-base-100 px-3.5 py-1.5 text-[11px] font-semibold text-base-content/60"><SvgIcon name="eye" size="14" />  视图 ({{ views.length }})</div>
              <div class="grid grid-cols-[repeat(auto-fill,minmax(180px,1fr))] gap-0.5 p-1 px-2">
                <label v-for="view in views" :key="'v-' + view"
                  class="flex cursor-pointer items-center gap-1.5 rounded px-2 py-1 text-xs transition-colors duration-100 hover:bg-base-200"
                  :class="isSelected(view) ? 'bg-primary/10' : ''">
                  <input type="checkbox" :checked="isSelected(view)" @change="toggleObject(view, 'view')" class="checkbox checkbox-xs" />
                  <span class="min-w-0 flex-1 truncate font-medium">{{ view }}</span>
                </label>
              </div>
            </div>
          </div>
        </div>

        <div v-else-if="selectedDb" class="px-5 py-10 text-center text-sm text-base-content/60">
          该数据库下没有表或视图
        </div>
      </div>

      <!-- Right: Backup History -->
      <div class="flex shrink-0 flex-col border-l border-base-content/10 ps-6 w-[340px]">
        <div class="mb-3 flex shrink-0 items-center justify-between text-sm font-semibold">
          <span><SvgIcon name="folder" size="14" />  备份历史</span>
          <span class="text-[11px] font-normal text-base-content/60" v-if="backups.length">{{ backups.length }} 个备份</span>
        </div>
        <div v-if="loadingHistory" class="px-2.5 py-[30px] text-center text-xs text-base-content/60">加载中...</div>
        <div v-else-if="backups.length === 0" class="px-2.5 py-[30px] text-center text-xs text-base-content/60">暂无备份</div>
        <div v-else class="flex flex-1 flex-col gap-1 overflow-y-auto">
          <div
            v-for="backup in backups"
            :key="backup.file"
            class="cursor-pointer rounded-md border border-transparent px-3 py-2.5 transition-colors duration-100 hover:bg-base-200"
            :class="selectedBackup?.file === backup.file ? 'bg-primary/10 border-primary/20' : ''"
            @contextmenu.prevent="showContextMenu($event, backup)"
            @click="selectBackup(backup)"
          >
            <div class="mb-1.5">
              <div class="mb-0.5 flex items-center gap-1.5 text-xs font-semibold">
                <span class="shrink-0 text-sm"><SvgIcon name="database" size="14" /> </span>
                <span class="text-base-content">{{ backup.databaseName }}</span>
                <span class="max-w-[140px] truncate rounded-sm bg-base-100 px-1.5 py-0.5 text-[10px] text-base-content/60">{{ backup.connectionName }}</span>
              </div>
              <div class="truncate ps-[22px] text-[10px] text-base-content/60">{{ backup.name }}</div>
            </div>
            <div class="flex items-center gap-2.5 ps-[22px] text-[10px] text-base-content/60">
              <span>{{ formatSize(backup.size) }}</span>
              <span>{{ backup.objects.length }} 项</span>
              <span>{{ formatTime(backup.backupTime) }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Backup Progress -->
    <div v-if="backupInProgress" class="flex flex-col items-center gap-4 p-10">
      <span class="loading loading-spinner loading-md text-primary"></span>
      <p class="m-0 text-sm text-base-content/60">{{ progressMessage }}</p>
    </div>

    <!-- Context Menu -->
    <Teleport to="body">
      <div v-if="contextMenu.visible" class="fixed z-[3000] min-w-[160px] rounded-lg border border-base-content/10 bg-base-100 p-1 shadow-xl" :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }" @click="contextMenu.visible = false">
        <div class="cursor-pointer rounded px-3 py-2 text-sm transition-colors duration-100 hover:bg-base-200" @click="restoreBackup(contextMenu.backup)"><SvgIcon name="refresh" size="14" />  还原此备份</div>
        <div class="cursor-pointer rounded px-3 py-2 text-sm text-error transition-colors duration-100 hover:bg-base-200" @click="deleteBackup(contextMenu.backup)"><SvgIcon name="trash" size="14" />  删除备份</div>
      </div>
    </Teleport>

    <!-- Restore Confirmation -->
    <Teleport to="body">
      <div v-if="restoreConfirm" class="fixed inset-0 z-[2000] flex items-center justify-center bg-black/50" @click="restoreConfirm = null">
        <div class="w-[400px] max-w-[90vw] rounded-xl bg-base-100 p-6 shadow-2xl" @click.stop>
          <h3 class="m-0 mb-3 text-base font-semibold"><SvgIcon name="alertTriangle" size="14" /> 确认还原</h3>
          <p class="m-0 mb-2 text-sm">将 <strong>{{ restoreConfirm.name }}</strong> 还原到当前连接？</p>
          <p class="m-0 mb-2 text-sm font-medium text-error">此操作将覆盖现有数据，请谨慎操作！</p>
          <div class="mt-5 flex justify-end gap-2">
            <button @click="restoreConfirm = null" class="btn btn-ghost">取消</button>
            <button @click="doRestore" class="btn btn-error">确认还原</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'// @ts-nocheck
import { ref, computed, onMounted, watch } from 'vue'
import { getTauriAPI } from '../../utils/tauri-api'
import { useDBManager } from '../../composables/useDBManager'
import { useToast } from '../../composables/useToast'

const db = useDBManager()
const toast = useToast()

interface BackupObject {
  type: 'table' | 'view'
  name: string
  includeData: boolean
}

interface BackupFileRaw {
  file: string
  name: string
  size: number
  backupTime: string
  objects: { type: string; name: string; includeData: boolean }[]
  connectionType: string
  connectionName: string
  databaseName: string
}

interface BackupFile {
  file: string
  name: string
  size: number
  backupTime: string
  objects: BackupObject[]
  connectionType: string
  connectionName: string
  databaseName: string
}

const props = defineProps<{
  connectionId: string
  connectionName?: string
}>()

const connections = computed(() => db.connections.value)
const nonRedisConnections = computed(() => connections.value.filter(c => c.type !== 'redis'))

const localConnectionId = ref(props.connectionId || '')
const selectedDb = ref('')
const databases = ref<string[]>([])
const loadingObjects = ref(false)
const objects = ref<BackupObject[]>([])
const selectedObjects = ref<Set<string>>(new Set())
const dataIncluded = ref<Set<string>>(new Set())

const tables = computed(() => objects.value.filter(o => o.type === 'table').map(o => o.name))
const views = computed(() => objects.value.filter(o => o.type === 'view').map(o => o.name))

const creating = ref(false)
const backupInProgress = ref(false)
const progressMessage = ref('')

const backups = ref<BackupFile[]>([])
const loadingHistory = ref(false)
const selectedBackup = ref<BackupFile | null>(null)

const contextMenu = ref({ visible: false, x: 0, y: 0, backup: null as BackupFile | null })
const restoreConfirm = ref<BackupFile | null>(null)

const selectedCount = computed(() => selectedObjects.value.size)
const canBackup = computed(() => localConnectionId.value && selectedDb.value && selectedObjects.value.size > 0)

async function onConnectionChange() {
  selectedDb.value = ''
  objects.value = []
  selectedObjects.value = new Set()
  dataIncluded.value = new Set()
  databases.value = []
  if (!localConnectionId.value) return

  loadingObjects.value = true
  try {
    console.log("[onConnectionChange] called")
    const conn = nonRedisConnections.value.find(c => c.id === localConnectionId.value)
    if (conn) {
      const connectResult = await getTauriAPI().dbConnect(JSON.parse(JSON.stringify(conn)))
      if (!connectResult?.success) {
        toast.error('连接失败: ' + (connectResult?.error || '未知错误'))
        return
      }
    } else {
      toast.error('未找到连接配置')
      return
    }
    const result = await getTauriAPI().dbGetDatabases(localConnectionId.value)
    if (result?.success) {
      databases.value = result.databases || []
      if (databases.value.length === 0) {
        toast.warning('该连接下没有用户数据库')
      }
    } else {
      toast.error('获取数据库列表失败: ' + (result?.error || '未知错误'))
    }
  } catch (e: any) {
    toast.error('获取数据库列表失败: ' + (e?.message || '未知错误'))
  } finally {
    loadingObjects.value = false
  }
}

async function loadObjects() {
  if (!localConnectionId.value || !selectedDb.value) return
  loadingObjects.value = true
  objects.value = []
  selectedObjects.value = new Set()
  dataIncluded.value = new Set()
  try {
    console.log("[loadObjects] called")
    const [tablesResult, viewsResult] = await Promise.all([
      getTauriAPI().dbGetTables(localConnectionId.value, selectedDb.value),
      getTauriAPI().dbGetViews(localConnectionId.value, selectedDb.value),
    ])
    const objs: BackupObject[] = []
    if (tablesResult?.success && tablesResult.tables) {
      for (const t of tablesResult.tables) objs.push({ type: 'table', name: t, includeData: true })
    }
    if (viewsResult?.success && viewsResult.views) {
      for (const v of viewsResult.views) objs.push({ type: 'view', name: v, includeData: false })
    }
    objects.value = objs
    // Select all by default
    selectedObjects.value = new Set(objs.map(o => o.name))
    dataIncluded.value = new Set(objs.filter(o => o.type === 'table').map(o => o.name))
  } catch (e) {
    toast.error('获取对象列表失败')
  } finally {
    loadingObjects.value = false
  }
}

function isSelected(name: string) { return selectedObjects.value.has(name) }
function includeData(name: string) { return dataIncluded.value.has(name) }

function toggleObject(name: string, type: 'table' | 'view') {
  if (selectedObjects.value.has(name)) {
    selectedObjects.value.delete(name)
    dataIncluded.value.delete(name)
  } else {
    selectedObjects.value.add(name)
    if (type === 'table') dataIncluded.value.add(name)
  }
}

function toggleData(name: string) {
  if (dataIncluded.value.has(name)) dataIncluded.value.delete(name)
  else dataIncluded.value.add(name)
}

function selectAll() {
  objects.value.forEach(o => {
    selectedObjects.value.add(o.name)
    if (o.type === 'table' && o.includeData !== false) dataIncluded.value.add(o.name)
  })
}

function selectNone() {
  selectedObjects.value = new Set()
  dataIncluded.value = new Set()
}

async function createBackup() {
    console.log("[selectNone] called")
  if (!canBackup.value) return
  creating.value = true
  backupInProgress.value = true
  progressMessage.value = '正在生成备份...'

  try {
    console.log("[createBackup] called")
    const backupObjects: BackupObject[] = []
    for (const name of selectedObjects.value) {
      const obj = objects.value.find(o => o.name === name)
      if (obj) {
        backupObjects.push({
          type: obj.type,
          name: obj.name,
          includeData: obj.type === 'table' ? dataIncluded.value.has(name) : false,
        })
      }
    }

    const result = await getTauriAPI().dbBackupCreate(localConnectionId.value, selectedDb.value, backupObjects)
    if (result?.success) {
      toast.success(`备份成功 (${formatSize(result.size)})`)
      loadBackupHistory()
    } else {
      toast.error('备份失败: ' + (result?.error || '未知错误'))
    }
  } catch (e: any) {
    toast.error('备份失败: ' + (e?.message || '未知错误'))
  } finally {
    creating.value = false
    backupInProgress.value = false
    progressMessage.value = ''
  }
}

async function loadBackupHistory() {
  loadingHistory.value = true
  try {
    console.log("[loadBackupHistory] called")
    const result = await getTauriAPI().dbBackupList(localConnectionId.value || undefined)
    backups.value = (result || []).map((b: any) => ({
      file: b.file,
      name: b.name,
      size: b.size,
      backupTime: b.backupTime,
      objects: b.objects.map((o: any) => ({ type: o.type as 'table' | 'view', name: o.name, includeData: o.includeData })),
      connectionType: b.connectionType,
      connectionName: b.connectionName,
      databaseName: b.databaseName,
    }))
  } catch {
    backups.value = []
  } finally {
    loadingHistory.value = false
  }
}

function selectBackup(backup: BackupFile) {
  selectedBackup.value = backup
}

function showContextMenu(event: MouseEvent, backup: BackupFile) {
  contextMenu.value = { visible: true, x: event.clientX, y: event.clientY, backup }
}

function restoreBackup(backup: BackupFile | null) {
  if (!backup) return
  contextMenu.value.visible = false
  restoreConfirm.value = backup
}

async function doRestore() {
    console.log("[restoreBackup] called")
  if (!restoreConfirm.value || !localConnectionId.value) return
  const backup = restoreConfirm.value
  restoreConfirm.value = null
  backupInProgress.value = true
  progressMessage.value = '正在还原备份...'

  try {
    console.log("[doRestore] called")
    const result = await getTauriAPI().dbBackupRestore(localConnectionId.value, backup.file)
    if (result?.success) {
      toast.success(`还原成功 (执行了 ${result.executed} 条 SQL)`)
    } else {
      const errors = result?.errors?.join('\n') || '未知错误'
      toast.error('还原失败:\n' + errors)
    }
  } catch (e: any) {
    toast.error('还原失败: ' + (e?.message || '未知错误'))
  } finally {
    backupInProgress.value = false
    progressMessage.value = ''
  }
}

async function deleteBackup(backup: BackupFile | null) {
  if (!backup) return
  contextMenu.value.visible = false
  try {
    console.log("[deleteBackup] called")
    const result = await getTauriAPI().dbBackupDelete(backup.file)
    if (result) {
      toast.success('已删除备份')
      loadBackupHistory()
    } else {
      toast.error('删除失败')
    }
  } catch {
    toast.error('删除失败')
  }
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

function formatTime(iso: string): string {
  const d = new Date(iso)
  return d.toLocaleString('zh-CN', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
}

onMounted(() => {
    console.log("[components/db/DBBackup.vue] mounted")
  loadBackupHistory()
  if (props.connectionId) {
    localConnectionId.value = props.connectionId
    onConnectionChange()
  }
})

watch(() => props.connectionId, (newId) => {
  if (newId && newId !== localConnectionId.value) {
    localConnectionId.value = newId
    onConnectionChange()
  }
})
</script>
