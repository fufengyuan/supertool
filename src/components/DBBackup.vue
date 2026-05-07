<template>
  <div class="db-backup">
    <div class="backup-header">
      <h3 class="backup-title">🗂️ 数据库备份</h3>
      <p class="backup-desc">备份和还原数据库结构和数据（.nb3 格式）</p>
    </div>

    <div class="backup-content" v-if="!backupInProgress">
      <!-- Left: Config + Object Selection -->
      <div class="backup-left">
        <div class="config-row">
          <div class="config-item">
            <label>连接</label>
            <select v-model="localConnectionId" @change="onConnectionChange" class="select-input">
              <option value="">选择连接</option>
              <option v-for="conn in nonRedisConnections" :key="conn.id" :value="conn.id">
                {{ conn.name }} ({{ conn.type }})
              </option>
            </select>
          </div>
          <div class="config-item">
            <label>数据库</label>
            <select v-model="selectedDb" @change="loadObjects" :disabled="!localConnectionId || loadingObjects" class="select-input">
              <option value="">{{ loadingObjects ? '加载中...' : '选择数据库' }}</option>
              <option v-for="db in databases" :key="db" :value="db">{{ db }}</option>
            </select>
          </div>
          <div class="backup-btn-wrap">
            <button @click="createBackup" :disabled="!canBackup || creating" class="btn btn-primary">
              {{ creating ? '备份中...' : '💾 新建备份' }}
            </button>
            <span v-if="selectedCount > 0" class="selected-info">已选 {{ selectedCount }} 项</span>
          </div>
        </div>

        <!-- Object Selection -->
        <div class="object-selection" v-if="objects.length > 0">
          <div class="object-header">
            <span>选择要备份的对象</span>
            <div class="object-controls">
              <button @click="selectAll" class="btn btn-ghost btn-xs">全选</button>
              <button @click="selectNone" class="btn btn-ghost btn-xs">清空</button>
            </div>
          </div>
          <div class="object-list">
            <!-- Tables -->
            <div v-if="tables.length > 0" class="object-section">
              <div class="section-label">📊 表 ({{ tables.length }})</div>
              <div class="object-grid">
                <label v-for="table in tables" :key="'t-' + table" class="object-item" :class="{ selected: isSelected(table) }">
                  <input type="checkbox" :checked="isSelected(table)" @change="toggleObject(table, 'table')" />
                  <span class="object-name">{{ table }}</span>
                  <span v-if="isSelected(table)" class="include-data">
                    <input type="checkbox" :checked="includeData(table)" @change="toggleData(table)" />
                    <span>含数据</span>
                  </span>
                </label>
              </div>
            </div>

            <!-- Views -->
            <div v-if="views.length > 0" class="object-section">
              <div class="section-label">👁️ 视图 ({{ views.length }})</div>
              <div class="object-grid">
                <label v-for="view in views" :key="'v-' + view" class="object-item" :class="{ selected: isSelected(view) }">
                  <input type="checkbox" :checked="isSelected(view)" @change="toggleObject(view, 'view')" />
                  <span class="object-name">{{ view }}</span>
                </label>
              </div>
            </div>
          </div>
        </div>

        <div v-else-if="selectedDb" class="no-objects">
          该数据库下没有表或视图
        </div>
      </div>

      <!-- Right: Backup History -->
      <div class="backup-right">
        <div class="history-header">
          <span>📁 备份历史</span>
          <span class="history-count" v-if="backups.length">{{ backups.length }} 个备份</span>
        </div>
        <div v-if="loadingHistory" class="history-loading">加载中...</div>
        <div v-else-if="backups.length === 0" class="history-empty">暂无备份</div>
        <div v-else class="history-list">
          <div
            v-for="backup in backups"
            :key="backup.file"
            class="history-item"
            :class="{ selected: selectedBackup?.file === backup.file }"
            @contextmenu.prevent="showContextMenu($event, backup)"
            @click="selectBackup(backup)"
          >
            <div class="item-main">
              <div class="item-name-row">
                <span class="item-icon">📦</span>
                <span class="item-db">{{ backup.databaseName }}</span>
                <span class="item-conn">{{ backup.connectionName }}</span>
              </div>
              <div class="item-file">{{ backup.name }}</div>
            </div>
            <div class="item-meta">
              <span class="item-size">{{ formatSize(backup.size) }}</span>
              <span class="item-objects">{{ backup.objects.length }} 项</span>
              <span class="item-time">{{ formatTime(backup.backupTime) }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Backup Progress -->
    <div v-if="backupInProgress" class="backup-progress">
      <div class="progress-spinner"></div>
      <p class="progress-text">{{ progressMessage }}</p>
    </div>

    <!-- Context Menu -->
    <Teleport to="body">
      <div v-if="contextMenu.visible" class="context-menu" :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }" @click="contextMenu.visible = false">
        <div class="context-menu-item" @click="restoreBackup(contextMenu.backup)">🔄 还原此备份</div>
        <div class="context-menu-item danger" @click="deleteBackup(contextMenu.backup)">🗑️ 删除备份</div>
      </div>
    </Teleport>

    <!-- Restore Confirmation -->
    <Teleport to="body">
      <div v-if="restoreConfirm" class="confirm-overlay" @click="restoreConfirm = null">
        <div class="confirm-dialog" @click.stop>
          <h3>⚠️ 确认还原</h3>
          <p>将 <strong>{{ restoreConfirm.name }}</strong> 还原到当前连接？</p>
          <p class="confirm-warn">此操作将覆盖现有数据，请谨慎操作！</p>
          <div class="confirm-actions">
            <button @click="restoreConfirm = null" class="btn btn-ghost">取消</button>
            <button @click="doRestore" class="btn btn-danger">确认还原</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
console.log("[DBBackup.vue] component loaded")
import { getTauriAPI } from '@/utils/tauri-api'
import { ref, computed, onMounted, watch } from 'vue'
import { useDBManager } from '@/composables/useDBManager'
import { useToast } from '@/composables/useToast'

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
    if (result) {
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
  if (!canBackup.value) return
  creating.value = true
  backupInProgress.value = true
  progressMessage.value = '正在生成备份...'

  try {
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
    if (result) {
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
  if (!restoreConfirm.value || !localConnectionId.value) return
  const backup = restoreConfirm.value
  restoreConfirm.value = null
  backupInProgress.value = true
  progressMessage.value = '正在还原备份...'

  try {
    const result = await getTauriAPI().dbBackupRestore(localConnectionId.value, backup.file)
    if (result) {
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

<style scoped>
.db-backup {
  padding: 20px 24px;
  flex: 1;
  overflow-y: auto;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.backup-header {
  margin-bottom: 16px;
  flex-shrink: 0;
}

.backup-title {
  margin: 0 0 4px;
  font-size: 18px;
  font-weight: 600;
}

.backup-desc {
  margin: 0;
  font-size: 12px;
  color: var(--main-text-secondary);
}

/* Two-column layout */
.backup-content {
  display: flex;
  gap: 24px;
  flex: 1;
  min-height: 0;
}

.backup-left {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.backup-right {
  width: 340px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  border-left: 1px solid var(--border-color);
  padding-left: 24px;
}

/* Config row */
.config-row {
  display: flex;
  gap: 12px;
  align-items: flex-end;
  flex-shrink: 0;
  margin-bottom: 16px;
}

.config-item {
  flex: 1;
  min-width: 0;
}

.config-item label {
  display: block;
  margin-bottom: 6px;
  font-size: 12px;
  font-weight: 500;
  color: var(--main-text-secondary);
}

.select-input {
  width: 100%;
  padding: 7px 10px;
  border: 1.5px solid var(--input-border);
  border-radius: 6px;
  background: var(--input-bg);
  color: var(--main-text);
  font-size: 13px;
  outline: none;
}

.select-input:focus {
  border-color: var(--primary-color);
}

.select-input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.backup-btn-wrap {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.backup-btn-wrap .btn-primary {
  white-space: nowrap;
}

.selected-info {
  font-size: 11px;
  color: var(--main-text-secondary);
}

/* Object selection */
.object-selection {
  flex: 1;
  min-height: 0;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.object-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  background: var(--input-bg);
  border-bottom: 1px solid var(--border-color);
  font-size: 13px;
  font-weight: 600;
  flex-shrink: 0;
}

.object-controls {
  display: flex;
  gap: 4px;
}

.object-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}

.object-section + .object-section {
  border-top: 1px solid var(--border-color);
}

.section-label {
  padding: 6px 14px;
  font-size: 11px;
  font-weight: 600;
  color: var(--main-text-secondary);
  background: var(--card-bg);
  position: sticky;
  top: 0;
  z-index: 1;
}

.object-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 2px;
  padding: 4px 8px;
}

.object-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 8px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  transition: background 0.1s;
}

.object-item:hover {
  background: var(--input-bg);
}

.object-item.selected {
  background: rgba(59, 130, 246, 0.08);
}

.object-item input[type="checkbox"] {
  accent-color: var(--primary-color);
  flex-shrink: 0;
}

.object-name {
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  min-width: 0;
}

.include-data {
  display: flex;
  align-items: center;
  gap: 3px;
  font-size: 10px;
  color: var(--main-text-secondary);
  flex-shrink: 0;
}

.include-data input {
  accent-color: var(--primary-color);
}

.no-objects {
  text-align: center;
  padding: 40px 20px;
  color: var(--main-text-secondary);
  font-size: 13px;
}

/* Backup history */
.history-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 14px;
  font-weight: 600;
  flex-shrink: 0;
  margin-bottom: 12px;
}

.history-count {
  font-size: 11px;
  font-weight: 400;
  color: var(--main-text-secondary);
}

.history-loading, .history-empty {
  text-align: center;
  padding: 30px 10px;
  color: var(--main-text-secondary);
  font-size: 12px;
}

.history-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.history-item {
  padding: 10px 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.12s;
  border: 1px solid transparent;
}

.history-item:hover {
  background: var(--input-bg);
}

.history-item.selected {
  background: rgba(59, 130, 246, 0.08);
  border-color: rgba(59, 130, 246, 0.2);
}

.item-main {
  margin-bottom: 6px;
}

.item-name-row {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 600;
  margin-bottom: 3px;
}

.item-icon {
  font-size: 14px;
  flex-shrink: 0;
}

.item-db {
  color: var(--main-text);
}

.item-conn {
  font-size: 10px;
  color: var(--main-text-secondary);
  background: var(--card-bg);
  padding: 1px 6px;
  border-radius: 3px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 140px;
}

.item-file {
  font-size: 10px;
  color: var(--main-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  padding-left: 22px;
}

.item-meta {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 10px;
  color: var(--main-text-secondary);
  padding-left: 22px;
}

/* Progress */
.backup-progress {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 40px;
  gap: 16px;
}

.progress-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--border-color);
  border-top-color: var(--primary-color);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.progress-text {
  margin: 0;
  font-size: 14px;
  color: var(--main-text-secondary);
}

/* Context menu */
.context-menu {
  position: fixed;
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.2);
  z-index: 3000;
  padding: 4px;
  min-width: 160px;
}

.context-menu-item {
  padding: 8px 12px;
  font-size: 13px;
  cursor: pointer;
  border-radius: 4px;
  transition: background 0.1s;
}

.context-menu-item:hover {
  background: var(--input-bg);
}

.context-menu-item.danger {
  color: var(--danger-color);
}

/* Confirm dialog */
.confirm-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}

.confirm-dialog {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 24px;
  width: 400px;
  max-width: 90vw;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

.confirm-dialog h3 {
  margin: 0 0 12px;
  font-size: 16px;
}

.confirm-dialog p {
  margin: 0 0 8px;
  font-size: 14px;
}

.confirm-warn {
  color: var(--danger-color);
  font-weight: 500;
}

.confirm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 20px;
}
</style>
