<template>
  <div class="db-manager">
    <div class="db-header">
      <h2 class="db-title">🗄️ 数据库管理</h2>
      <div class="header-actions" v-if="db.activeConnection.value">
        <template v-if="db.activeConnection.value.type === 'redis'">
          <button
            @click="openRedisManager"
            class="btn btn-primary btn-sm"
            title="Redis 管理器"
          >
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4" />
            </svg>
            Redis 管理器
          </button>
        </template>
        <template v-else>
          <button
            @click="db.openSqlTab(db.activeConnection.value.id, db.activeConnection.value.name)"
            class="btn btn-ghost btn-sm"
            title="新建查询"
          >
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
              <polyline points="14 2 14 8 20 8" />
              <line x1="12" y1="11" x2="12" y2="17" />
              <line x1="9" y1="14" x2="15" y2="14" />
            </svg>
            新建查询
          </button>
          <button
            @click="db.openStructureSyncTab()"
            class="btn btn-ghost btn-sm"
            title="结构同步"
          >
            🔧 结构同步
          </button>
          <button
            @click="db.openDataSyncTab()"
            class="btn btn-ghost btn-sm"
            title="数据同步"
          >
            📦 数据同步
          </button>
          <button
            @click="openBackupTab"
            class="btn btn-ghost btn-sm"
            title="数据库备份"
          >
            🗂️ 数据库备份
          </button>
        </template>
      </div>
    </div>

    <div class="db-layout">
      <!-- Left sidebar: Connection tree -->
      <aside class="db-sidebar">
        <div class="db-sidebar-header">
          <span>连接</span>
          <button @click="db.openAddForm()" class="btn btn-primary btn-sm" title="添加连接">+</button>
        </div>
        <ConnectionTree
          ref="treeRef"
          :sorted-connections="db.sortedConnections.value"
          :active-connection-id="db.activeConnectionId.value"
          :selected-table="db.selectedTable.value"
          :is-connection-expanded="db.isConnectionExpanded"
          :are-tables-expanded="db.areTablesExpanded"
          :is-database-expanded="db.isDatabaseExpanded"
          :are-db-tables-expanded="db.areDbTablesExpanded"
          :are-db-views-expanded="db.areDbViewsExpanded"
          :is-redis-database-expanded="db.isRedisDatabaseExpanded"
          :is-redis-folder-expanded="db.isRedisFolderExpanded"
          @select="db.setActiveConnection"
          @select-table="handleSelectTable"
          @open-table-data="handleOpenTableData"
          @open-table-structure="handleOpenTableStructure"
          @open-sql="handleOpenSql"
          @refresh-tables="handleRefreshTables"
          @toggle="db.toggleConnection"
          @edit="db.openEditForm"
          @toggle-tables="db.toggleTables"
          @toggle-database="db.toggleDatabase"
          @toggle-db-tables="db.toggleDbTables"
          @toggle-db-views="db.toggleDbViews"
          @toggle-redis-database="db.toggleRedisDatabase"
          @toggle-redis-folder="db.toggleRedisFolder"
          @open-redis-key="handleOpenRedisKey"
          @open-redis-queue="handleOpenRedisQueue"
          @add-connection="db.openAddForm"
          @delete="handleDeleteConnection"
        />
      </aside>

      <!-- Main area -->
      <main class="db-main">
        <!-- No connection selected -->
        <div v-if="!db.activeConnection.value" class="db-empty">
          <svg viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="currentColor" stroke-width="1.5">
            <ellipse cx="12" cy="5" rx="9" ry="3" />
            <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3" />
            <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5" />
          </svg>
          <p class="db-empty-title">选择或添加数据库连接</p>
          <p class="db-empty-sub">从左侧选择已有连接，或添加新的数据库连接</p>
          <button @click="db.openAddForm()" class="btn btn-primary">添加连接</button>
        </div>

        <!-- No tabs open -->
        <div v-else-if="db.tabs.value.length === 0" class="db-empty">
          <template v-if="db.activeConnection.value?.type === 'redis'">
            <svg viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4" />
            </svg>
            <p class="db-empty-title">Redis 连接已就绪</p>
            <p class="db-empty-sub">浏览和管理 Redis 键值数据</p>
            <button
              @click="openRedisManager"
              class="btn btn-primary"
            >
              🔑 打开 Redis 管理器
            </button>
          </template>
          <template v-else>
            <svg viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
              <polyline points="14 2 14 8 20 8" />
              <line x1="16" y1="13" x2="8" y2="13" />
              <line x1="16" y1="17" x2="8" y2="17" />
            </svg>
            <p class="db-empty-title">打开一个工作区</p>
            <p class="db-empty-sub">从左侧树中选择一个表，或点击"新建查询"开始</p>
            <button
              @click="db.openSqlTab(db.activeConnection.value.id, db.activeConnection.value.name)"
              class="btn btn-primary"
            >
              新建查询
            </button>
          </template>
        </div>

        <!-- Tabbed workspace -->
        <template v-else>
          <!-- Tab bar -->
          <div class="tab-bar">
            <div
              v-for="(tab, idx) in db.tabs.value"
              :key="tab.id"
              class="tab-item"
              :class="{ active: db.activeTabIndex.value === idx }"
              @click="db.setActiveTab(idx)"
            >
              <span class="tab-icon">{{ getTabIcon(tab) }}</span>
              <span class="tab-title" :title="tab.title">{{ tab.title }}</span>
              <button class="tab-close" @click.stop="db.closeTab(tab.id)" title="关闭">
                <svg viewBox="0 0 24 24" width="10" height="10" fill="none" stroke="currentColor" stroke-width="2.5">
                  <line x1="18" y1="6" x2="6" y2="18" />
                  <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
              </button>
            </div>
          </div>

          <!-- Active tab content -->
          <div class="tab-content">
            <!-- SQL Query tab -->
            <template v-if="activeTab?.type === 'sql'">
              <div class="sql-workspace">
                <SqlEditor
                  ref="sqlEditorRef"
                  :connection="getTabConnection(activeTab)"
                  :executing="executing"
                  :error="error"
                  :history="db.queryHistory.value"
                  :initialSql="activeTab.sql"
                  @execute="handleExecute"
                  @clear="clearError"
                  @rerun="handleRerun"
                  @clear-history="db.queryHistory.value = []"
                />
                <DataGrid
                  v-if="resultRows.length > 0 || executing"
                  :rows="resultRows"
                  :total="resultTotal"
                  :page="currentPage"
                  :page-size="pageSize"
                  :loading="executing"
                  :column-comments="columnComments"
                  :sort-column="activeSort?.column || null"
                  :sort-direction="activeSort?.direction || 'asc'"
                  @prev-page="handlePrevPage"
                  @next-page="handleNextPage"
                  @filter="handleFilterApply"
                  @filter-clear="handleFilterClear"
                  @batch-update="handleBatchUpdate"
                  @insert-row="handleInsertRow"
                  @delete-row="handleDeleteRow"
                  @refresh="loadTableData"
                  @sort="handleSort"
                />
              </div>
            </template>

            <!-- Table Data tab -->
            <template v-else-if="activeTab?.type === 'tableData'">
              <div class="table-workspace">
                <div class="workspace-header">
                  <span class="workspace-title">
                    📊 {{ activeTab.tableName }}
                    <span class="workspace-subtitle">- {{ activeTab.connectionName }}</span>
                  </span>
                  <div class="workspace-actions">
                    <button
                      @click="loadTableData"
                      class="btn btn-ghost btn-sm"
                      :disabled="tableLoading"
                    >
                      🔄 刷新
                    </button>
                  </div>
                </div>
                <div v-if="tableLoading" class="workspace-loading">加载中...</div>
                <DataGrid
                  v-else
                  :rows="resultRows"
                  :total="resultTotal"
                  :page="currentPage"
                  :page-size="pageSize"
                  :loading="false"
                  :primary-key-columns="tablePrimaryKeyColumns"
                  :column-comments="columnComments"
                  :sort-column="activeSort?.column || null"
                  :sort-direction="activeSort?.direction || 'asc'"
                  @prev-page="handlePrevPage"
                  @next-page="handleNextPage"
                  @filter="handleFilterApply"
                  @filter-clear="handleFilterClear"
                  @batch-update="handleBatchUpdate"
                  @insert-row="handleInsertRow"
                  @delete-row="handleDeleteRow"
                  @refresh="loadTableData"
                  @sort="handleSort"
                />
              </div>
            </template>

            <!-- Table Structure tab -->
            <template v-else-if="activeTab?.type === 'tableStructure'">
              <TableStructure
                :connection-id="activeTab.connectionId"
                :table-name="activeTab.tableName || ''"
                :db-name="activeTab.dbName"
                :db-type="getTabDbType(activeTab)"
              />
            </template>

            <!-- Redis Console tab -->
            <template v-else-if="activeTab?.type === 'redisConsole'">
              <div class="redis-workspace">
                <div class="workspace-header">
                  <span class="workspace-title">
                    🔴 Redis 控制台
                    <span class="workspace-subtitle">- {{ activeTab.connectionName }}</span>
                  </span>
                </div>
                <div class="redis-console-body">
                  <div class="redis-output" ref="redisOutputRef">
                    <div v-for="(msg, idx) in redisMessages" :key="idx" class="redis-msg" :class="msg.type">
                      <span class="redis-msg-prefix">{{ msg.prefix }}</span>
                      <span class="redis-msg-content">{{ msg.content }}</span>
                    </div>
                    <div v-if="redisMessages.length === 0" class="redis-empty">
                      输入 Redis 命令，例如: GET key, KEYS *, INFO
                    </div>
                  </div>
                  <div class="redis-input-row">
                    <span class="redis-prompt">&gt;</span>
                    <input
                      v-model="redisCommand"
                      @keydown.enter="executeRedis"
                      class="redis-input"
                      placeholder="输入 Redis 命令..."
                      spellcheck="false"
                    />
                    <button @click="executeRedis" class="btn btn-primary btn-sm" :disabled="redisExecuting">
                      执行
                    </button>
                  </div>
                </div>
              </div>
            </template>

            <!-- Redis Manager tab (Navicat-like) -->
            <template v-else-if="activeTab?.type === 'redisManager'">
              <RedisManager
                :connection-id="activeTab.connectionId"
                :connection-name="activeTab.connectionName"
                :connection="db.connections.value.find(c => c.id === activeTab.connectionId)"
                :initial-key="activeTab.initialKey"
                :redis-db-index="activeTab.redisDbIndex"
              />
            </template>

            <!-- Redis Queue tab -->
            <template v-else-if="activeTab?.type === 'redisQueue'">
              <RedisQueueManager
                :connection-id="activeTab.connectionId"
                :connection-name="activeTab.connectionName"
                :connection="db.connections.value.find(c => c.id === activeTab.connectionId)"
                :redis-db-index="activeTab.redisDbIndex"
              />
            </template>

            <!-- Structure Sync tab -->
            <template v-else-if="activeTab?.type === 'structureSync'">
              <StructureSync />
            </template>

            <!-- Data Sync tab -->
            <template v-else-if="activeTab?.type === 'dataSync'">
              <DataSync />
            </template>

            <!-- Database Backup tab -->
            <template v-else-if="activeTab?.type === 'backup'">
              <DBBackup :connection-id="activeTab.connectionId" :connection-name="activeTab.connectionName" />
            </template>
          </div>
        </template>
      </main>
    </div>

    <!-- Connection form modal -->
    <ConnectionForm
      v-if="db.showConnectionForm.value"
      :form="connectionForm"
      :is-editing="!!db.editingConnection.value"
      :test-result="testResult"
      :testing="testing"
      @close="db.closeForm"
      @save="handleSaveConnection"
      @test="handleTestConnection"
    />
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
console.log("[components/DBManager.vue] component loaded")
import * as logger from '../services/logger'
import { ref, computed, watch, nextTick, onMounted } from 'vue'
import { useDBManager, type DBConnection, type WorkspaceTab } from '../composables/useDBManager'
import ConnectionTree from '@/components/db/ConnectionTree.vue'
import ConnectionForm from '@/components/db/ConnectionForm.vue'
import SqlEditor from '@/components/db/SqlEditor.vue'
import DataGrid from '@/components/db/DataGrid.vue'
import TableStructure from '@/components/db/TableStructure.vue'
import RedisManager from '@/components/db/RedisManager.vue'
import RedisQueueManager from '@/components/db/RedisQueueManager.vue'
import StructureSync from '@/components/db/StructureSync.vue'
import DataSync from '@/components/db/DataSync.vue'
import DBBackup from '@/components/db/DBBackup.vue'
import type { FilterCondition } from '@/components/db/FilterBar.vue'
import { useToast } from '../composables/useToast'
import { getTauriAPI } from '../utils/tauri-api'

const db = useDBManager()
const toast = useToast()
const treeRef = ref<InstanceType<typeof ConnectionTree> | null>(null)

// Convert Date objects and ISO strings to MySQL-compatible format before IPC
// JSON.stringify turns Date into ISO string ('2026-04-21T16:00:00.000Z') which MySQL DATE columns reject
function sanitizeForDB(obj: Record<string, unknown> | null): Record<string, unknown> | null {
  if (!obj) return obj
  // 深度克隆剥离 Vue Proxy，避免嵌套对象/数组触发 IPC 克隆错误
  const cloned = JSON.parse(JSON.stringify(obj))
  const result: Record<string, unknown> = {}
  for (const [key, val] of Object.entries(cloned)) {
    if (val instanceof Date && !isNaN(val.getTime())) {
      const pad = (n: number) => String(n).padStart(2, '0')
      result[key] = `${val.getFullYear()}-${pad(val.getMonth() + 1)}-${pad(val.getDate())} ${pad(val.getHours())}:${pad(val.getMinutes())}:${pad(val.getSeconds())}`
    } else if (typeof val === 'string' && /^\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}/.test(val)) {
      const d = new Date(val)
      if (!isNaN(d.getTime())) {
        const pad = (n: number) => String(n).padStart(2, '0')
        result[key] = `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
      } else {
        result[key] = val
      }
    } else {
      result[key] = val
    }
  }
  return result
}
const sqlEditorRef = ref<InstanceType<typeof SqlEditor> | null>(null)
const redisOutputRef = ref<HTMLDivElement | null>(null)

// UI state
const executing = ref(false)
const testing = ref(false)
const error = ref<string | null>(null)
const testResult = ref<{ success: boolean; error?: string } | null>(null)
const resultRows = ref<Record<string, unknown>[]>([])
const resultTotal = ref(0)
const currentPage = ref(1)
const pageSize = 100
const tableLoading = ref(false)
const activeFilters = ref<FilterCondition[]>([])
const filterApplied = ref(false)
const columnComments = ref<Record<string, string>>({})
const activeSort = ref<{ column: string; direction: 'asc' | 'desc' } | null>(null)

// Connection form state
const connectionForm = ref<DBConfig>({
  id: '',
  name: '',
  type: 'mysql',
  host: '127.0.0.1',
  port: 3306,
  username: '',
  password: '',
  dbName: ''
})

// Redis console state
const redisCommand = ref('')
const redisExecuting = ref(false)
const redisMessages = ref<Array<{ type: string; prefix: string; content: string }>>([])

const activeTab = computed(() => db.activeTab.value)

function getTabIcon(tab: WorkspaceTab | null): string {
  if (!tab) return '📄'
  switch (tab.type) {
    case 'sql': return '📝'
    case 'tableData': return '📊'
    case 'tableStructure': return '🏗️'
    case 'redisConsole': return '🔴'
    case 'redisManager': return '🔑'
    case 'structureSync': return '🔧'
    case 'dataSync': return '📦'
    case 'backup': return '🗂️'
    default: return '📄'
  }
}

function getTabConnection(tab: WorkspaceTab | null): DBConnection | null {
  if (!tab) return null
  return db.connections.value.find(c => c.id === tab.connectionId) || null
}

function getTabDbType(tab: WorkspaceTab | null): 'mysql' | 'postgresql' | 'sqlite' | undefined {
  if (!tab) return undefined
  const conn = db.connections.value.find(c => c.id === tab.connectionId)
  return conn?.type as 'mysql' | 'postgresql' | 'sqlite' | undefined
}

function resetForm() {
  connectionForm.value = {
    id: '',
    name: '',
    type: 'mysql',
    host: '127.0.0.1',
    port: 3306,
    username: '',
    password: '',
    dbName: ''
  }
}

// Watch editing connection changes
watch(() => db.editingConnection.value, (conn) => {
  if (conn) {
    connectionForm.value = { ...conn }
  } else {
    resetForm()
  }
})

async function handleSaveConnection(form: DBConfig) {
  if (!form.name.trim()) {
    toast.info('请输入连接名称')
    return
  }

  if (db.editingConnection.value) {
    db.updateConnection(db.editingConnection.value.id, form)
    toast.info('连接已更新')
  } else {
    const conn = db.addConnection(form)
    db.setActiveConnection(conn.id)
    toast.info('连接已添加')
  }
  db.closeForm()
  resetForm()
  testResult.value = null
}

async function handleTestConnection(form: DBConfig) {
  testing.value = true
  testResult.value = null
  try {
    // ⚠️ 剥离 Vue Proxy，否则 Tauri IPC 的 structuredClone 会失败
    const plainForm = JSON.parse(JSON.stringify(form))
    const result = await getTauriAPI().dbTest(plainForm)
    testResult.value = result || { success: false, error: '无法调用测试接口' }
  } catch (e: any) {
    testResult.value = { success: false, error: e?.message || '测试失败' }
  } finally {
    testing.value = false
  }
}

async function handleDeleteConnection(id: string) {
  if (confirm('确定要删除此连接吗？')) {
    db.deleteConnection(id)
    toast.info('连接已删除')
  }
}

function openRedisManager() {
  if (!db.activeConnection.value) return
  db.openRedisManagerTab(db.activeConnection.value.id, db.activeConnection.value.name)
}

function openBackupTab() {
  if (!db.activeConnection.value) return
  db.openBackupTab(db.activeConnection.value.id, db.activeConnection.value.name)
}

function handleOpenRedisQueue(connId: string, dbIndex: number) {
  const conn = db.connections.value.find(c => c.id === connId)
  if (conn) {
    db.openRedisQueueTab(connId, conn.name, dbIndex)
  }
}

async function handleExecute(sqlText: string) {
  if (!db.activeConnection.value) return

  // Check if connection requires approval
  if ((db.activeConnection.value as any).requiresApproval) {
    const proceed = confirm(
      `⚠️ 安全审核\n\n数据库「${db.activeConnection.value.name}」已开启安全审核。\n\n请确认你要执行以下 SQL：\n\n${sqlText.substring(0, 200)}${sqlText.length > 200 ? '...' : ''}`
    )
    if (!proceed) return
  }

  executing.value = true
  error.value = null
  currentPage.value = 1

  const startTime = performance.now()
  try {
    // ⚠️ 剥离 Vue Proxy，否则 Tauri IPC 的 structuredClone 会失败
    const plainSql = sqlText
    const queryResult = await getTauriAPI().dbQuery(db.activeConnection.value.id, plainSql)
    const execTime = Math.round(performance.now() - startTime)

    // dbQuery 返回 { success, rows } 或 { success, error }
    let rows: Record<string, unknown>[] = []
    if (queryResult && typeof queryResult === 'object' && 'rows' in queryResult) {
      rows = (queryResult as any).success ? ((queryResult as any).rows as Record<string, unknown>[]) || [] : []
    } else if (Array.isArray(queryResult)) {
      rows = queryResult as Record<string, unknown>[]
    }

    resultRows.value = rows
    resultTotal.value = rows.length

    // Update SqlEditor status bar
    sqlEditorRef.value?.recordExecution(execTime, rows.length)

    db.addQueryRecord({
      sql: sqlText,
      connectionId: db.activeConnection.value.id,
      success: true,
      rowCount: rows.length,
      executionTime: execTime
    })
  } catch (e: any) {
    const execTime = Math.round(performance.now() - startTime)
    error.value = e?.message || '查询执行失败'
    db.addQueryRecord({
      sql: sqlText,
      connectionId: db.activeConnection.value!.id,
      success: false,
      error: error.value,
      executionTime: execTime
    })
  } finally {
    executing.value = false
  }
}

function handleRerun(sqlText: string) {
  // Update the active tab's SQL if it's a SQL tab
  if (activeTab.value?.type === 'sql') {
    activeTab.value.sql = sqlText
  }
}

function handleSelectTable(connId: string, table: string, dbName?: string) {
  db.selectTable(table)
  // Open table data in a new tab
  const conn = db.connections.value.find(c => c.id === connId)
  if (conn) {
    db.openTableDataTab(connId, conn.name, table, dbName)
    // Also auto-load the data
    loadTableDataForTab(connId, table)
    loadTablePrimaryKeys()
  }
}

function handleOpenSql(connId: string, table?: string, dbName?: string) {
  const conn = db.connections.value.find(c => c.id === connId)
  if (conn) {
    if (table) {
      db.openSqlTab(connId, conn.name, `-- ${conn.name} - ${table}\nSELECT * FROM \`${table}\` LIMIT 100;`)
    } else {
      db.openSqlTab(connId, conn.name, `-- ${conn.name}\nSELECT * FROM table_name LIMIT 100;`)
    }
  }
}

function handleOpenTableData(connId: string, table: string, dbName?: string) {
  const conn = db.connections.value.find(c => c.id === connId)
  if (conn) {
    db.openTableDataTab(connId, conn.name, table, dbName)
    loadTableDataForTab(connId, table)
  }
}

function handleOpenTableStructure(connId: string, table: string, dbName?: string) {
  const conn = db.connections.value.find(c => c.id === connId)
  if (conn) {
    db.openTableStructureTab(connId, conn.name, table, dbName)
  }
}

function handleOpenRedisKey(connId: string, dbIndex: number, key: string) {
  logger.info(`[DBManager] handleOpenRedisKey called: ${JSON.stringify({ connId, dbIndex, key })}`)
  const conn = db.connections.value.find(c => c.id === connId)
  logger.info('[DBManager] connection found:', conn ? conn.name : 'NOT FOUND')
  if (conn) {
    const tab = db.openRedisManagerTab(connId, conn.name, key, dbIndex)
    logger.info(`[DBManager] tab opened/activated: ${tab ? tab.id : 'null'}, initialKey: ${tab?.initialKey}`)
  }
}

function handleRefreshTables(connId: string) {
  treeRef.value?.refreshTables(connId)
  toast.info('表列表已刷新')
}

async function fetchColumnComments(connId: string, table: string, dbName?: string) {
  try {
    const conn = db.connections.value.find(c => c.id === connId)
    if (!conn || conn.type === 'redis') return

    const safeTable = table.replace(/'/g, "''")
    let sql: string
    if (conn.type === 'mysql') {
      const safeDb = dbName ? dbName.replace(/'/g, "''") : ''
      const dbCond = safeDb ? `TABLE_SCHEMA = '${safeDb}'` : 'TABLE_SCHEMA = DATABASE()'
      sql = `SELECT COLUMN_NAME, COLUMN_COMMENT FROM INFORMATION_SCHEMA.COLUMNS WHERE ${dbCond} AND TABLE_NAME = '${safeTable}' AND COLUMN_COMMENT != ''`
    } else if (conn.type === 'postgresql') {
      sql = `SELECT a.attname AS column_name, col_description(a.attrelid, a.attnum) AS column_comment FROM pg_attribute a JOIN pg_class c ON a.attrelid = c.oid JOIN pg_namespace n ON c.relnamespace = n.oid WHERE n.nspname = 'public' AND c.relname = '${safeTable}' AND a.attnum > 0 AND NOT a.attisdropped AND col_description(a.attrelid, a.attnum) IS NOT NULL AND col_description(a.attrelid, a.attnum) != ''`
    } else {
      // SQLite: no native column comment support
      return
    }

    const result = await getTauriAPI().dbQuery(connId, sql)
    let rows: any[] = []
    if (result && typeof result === 'object' && 'rows' in result) {
      rows = (result as any).success ? ((result as any).rows || []) : []
    } else if (Array.isArray(result)) {
      rows = result
    }

    const comments: Record<string, string> = {}
    for (const row of rows) {
      const name = row.COLUMN_NAME || row.column_name
      const comment = row.COLUMN_COMMENT || row.column_comment
      if (name && comment && comment.trim()) {
        comments[name] = comment.trim()
      }
    }
    columnComments.value = comments
  } catch {
    // Silently fail — comments are optional
  }
}

function clearError() {
  error.value = null
}

async function loadTableData() {
  if (!activeTab.value || activeTab.value.type !== 'tableData') return
  const tab = activeTab.value
  await loadTableDataForTab(tab.connectionId, tab.tableName || '')
}

async function loadTableDataForTab(connId: string, table: string) {
  tableLoading.value = true
  try {
    // Get dbName from the active tab
    const tab = db.activeTab.value
    const dbName = tab?.dbName

    // Fetch column comments (async, don't block data loading)
    fetchColumnComments(connId, table, dbName)

    // If filters are active, use filtered query
    if (filterApplied.value && activeFilters.value.length > 0) {
      const filterPayload = JSON.parse(JSON.stringify({
        connectionId: connId,
        tableName: table,
        filters: activeFilters.value,
        limit: pageSize,
        offset: (currentPage.value - 1) * pageSize,
        dbName,
      }))
      const result = await getTauriAPI().dbGetTablesFiltered(filterPayload)
      if (result?.success) {
        resultRows.value = (result.rows as Record<string, unknown>[]) || []
        resultTotal.value = result.total || 0
      } else {
        toast.error('筛选查询失败: ' + (result?.error || '未知错误'))
      }
    } else {
      const result = await getTauriAPI().dbGetTableDataFiltered({
        connId, table, pageSize, offset: (currentPage.value - 1) * pageSize, dbName,
        sortColumn: activeSort.value?.column, sortDirection: activeSort.value?.direction
      })
      // dbGetTableData 返回 { success, rows, total }
      if (result && typeof result === 'object' && 'rows' in result) {
        if ((result as any).success) {
          resultRows.value = (result as any).rows || []
          resultTotal.value = (result as any).total || 0
        } else {
          toast.error('加载表数据失败: ' + ((result as any).error || '未知错误'))
        }
      } else if (Array.isArray(result)) {
        // 兼容旧版直接返回数组的情况
        resultRows.value = result
        resultTotal.value = result.length
      } else {
        // Fallback to raw query
        // SECURITY: Sanitize table name to prevent SQL injection
        const safeTable = table.replace(/`/g, '``')
        const tableRef = dbName
          ? `\`${dbName.replace(/`/g, '``')}\`.\`${safeTable}\``
          : `\`${safeTable}\``
        const rows = await getTauriAPI().dbQuery(connId, `SELECT * FROM ${tableRef} LIMIT ${pageSize} OFFSET ${(currentPage.value - 1) * pageSize}`)
        let dataRows: Record<string, unknown>[] = []
        if (rows && typeof rows === 'object' && 'rows' in rows) {
          dataRows = (rows as any).success ? ((rows as any).rows as Record<string, unknown>[]) || [] : []
        } else if (Array.isArray(rows)) {
          dataRows = rows as Record<string, unknown>[]
        }
        resultRows.value = dataRows
        resultTotal.value = dataRows.length
      }
    }
  } catch (e: any) {
    toast.error('加载表数据失败: ' + (e?.message || '未知错误'))
  } finally {
    tableLoading.value = false
  }
}

function handleFilterApply(conditions: FilterCondition[]) {
  activeFilters.value = conditions
  filterApplied.value = true
  currentPage.value = 1 // Reset to first page when filter changes
  loadTableData()
}

function handleFilterClear() {
  activeFilters.value = []
  filterApplied.value = false
  activeSort.value = null
  currentPage.value = 1
  loadTableData()
}

async function handleSort(column: string, direction: 'asc' | 'desc') {
  activeSort.value = { column, direction }
  currentPage.value = 1
  await loadTableData()
}

function handlePrevPage() {
  if (currentPage.value > 1) {
    currentPage.value--
    loadTableData()
  }
}

function handleNextPage() {
  currentPage.value++
  loadTableData()
}

// ============ Table Row CRUD ============

const tablePrimaryKeyColumns = ref<string[]>([])

async function loadTablePrimaryKeys() {
  if (!activeTab.value || activeTab.value.type !== 'tableData') return
  const tab = activeTab.value
  try {
    const res = await getTauriAPI().dbGetTablePrimaryKeys(tab.connectionId, tab.tableName || '', tab.dbName)
    if (res?.success && res.primaryKeys) {
      tablePrimaryKeyColumns.value = res.primaryKeys
    } else {
      tablePrimaryKeyColumns.value = []
    }
  } catch {
    tablePrimaryKeyColumns.value = []
  }
}

async function handleUpdateRow(index: number, oldRow: Record<string, unknown>, newRow: Record<string, unknown>) {
  if (!activeTab.value || activeTab.value.type !== 'tableData') return
  const tab = activeTab.value
  try {
    const plainOld = sanitizeForDB(oldRow)
    const plainNew = sanitizeForDB(newRow)
    const res = await getTauriAPI().dbUpdateTableRow(tab.connectionId, tab.tableName || '', plainOld!, plainNew!)
    if (res?.success) {
      return true
    } else {
      toast.error('更新失败: ' + (res?.error || '未知错误'))
      return false
    }
  } catch (e: any) {
    toast.error('更新失败: ' + (e?.message || '未知错误'))
    return false
  }
}

async function handleBatchUpdate(updates: Array<{ oldRow: Record<string, unknown>; newRow: Record<string, unknown> }>) {
  let successCount = 0
  let failCount = 0
  for (const { oldRow, newRow } of updates) {
    const ok = await handleUpdateRow(0, oldRow, newRow)
    if (ok) successCount++
    else failCount++
  }
  if (successCount > 0) {
    toast.success(`已保存 ${successCount} 行${failCount > 0 ? `，${failCount} 行失败` : ''}`)
    await loadTableData()
  }
  if (failCount > 0 && successCount === 0) {
    toast.error('保存失败')
  }
}

async function handleInsertRow(row: Record<string, unknown>) {
  if (!activeTab.value || activeTab.value.type !== 'tableData') return
  const tab = activeTab.value
  try {
    const plainRow = sanitizeForDB(row)
    const res = await getTauriAPI().dbInsertTableRow(tab.connectionId, tab.tableName || '', plainRow!)
    if (res?.success) {
      toast.success('行已插入')
      // Reload data to include new row
      await loadTableData()
    } else {
      toast.error('插入失败: ' + (res?.error || '未知错误'))
    }
  } catch (e: any) {
    toast.error('插入失败: ' + (e?.message || '未知错误'))
  }
}

async function handleDeleteRow(row: Record<string, unknown>, _index: number) {
  if (!activeTab.value || activeTab.value.type !== 'tableData') return
  const tab = activeTab.value
  try {
    const plainRow = sanitizeForDB(row)
    const res = await getTauriAPI().dbDeleteTableRow(tab.connectionId, tab.tableName || '', plainRow!)
    if (res?.success) {
      toast.success('行已删除')
      // Reload data to remove deleted row
      await loadTableData()
    } else {
      toast.error('删除失败: ' + (res?.error || '未知错误'))
    }
  } catch (e: any) {
    toast.error('删除失败: ' + (e?.message || '未知错误'))
  }
}

// Watch active tab changes to load data
watch(() => db.activeTabIndex.value, () => {
  if (activeTab.value?.type === 'tableData') {
    const tab = activeTab.value
    // Clear filters when switching to a new table tab
    if (activeFilters.value.length > 0) {
      activeFilters.value = []
      filterApplied.value = false
    }
    activeSort.value = null
    currentPage.value = 1
    if (tab.tableName) {
      loadTableDataForTab(tab.connectionId, tab.tableName)
      loadTablePrimaryKeys()
    }
  }
})

// Redis console
async function executeRedis() {
  if (!redisCommand.value.trim() || !activeTab.value) return

  const cmd = redisCommand.value.trim()
  redisMessages.value.push({ type: 'input', prefix: '> ', content: cmd })
  redisCommand.value = ''
  redisExecuting.value = true

  try {
    if (!activeTab.value) return
    const connId = activeTab.value.connectionId
    const result = await getTauriAPI().dbRedisExec(connId, activeTab.value.dbIndex || 0, cmd)
    if (result?.success) {
      const output = typeof result.result === 'object' ? JSON.stringify(result.result, null, 2) : String(result.result ?? '')
      redisMessages.value.push({ type: 'output', prefix: '', content: output })
    } else {
      redisMessages.value.push({ type: 'error', prefix: 'ERR ', content: result?.error || '命令执行失败' })
    }
  } catch (e: any) {
    redisMessages.value.push({
      type: 'error',
      prefix: 'ERR ',
      content: e?.message || '命令执行失败'
    })
  } finally {
    redisExecuting.value = false
    // Scroll to bottom
    nextTick(() => {
      if (redisOutputRef.value) {
        redisOutputRef.value.scrollTop = redisOutputRef.value.scrollHeight
      }
    })
  }
}

onMounted(async () => {
  await db.loadConnections()
})

// Auto-open RedisManager when a Redis connection is selected
watch(() => db.activeConnection.value, (conn) => {
  if (conn?.type === 'redis' && db.tabs.value.length === 0) {
    db.openRedisManagerTab(conn.id, conn.name)
  }
})
</script>

<style scoped>
.db-manager {
  display: flex;
  flex-direction: column;
  height: calc(100vh - 68px);
  min-height: 0;
  max-width: 100vw;
  overflow: hidden;
}

.db-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  min-height: 0;
  gap: 12px;
}

.db-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-base-content);
  margin: 0;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.header-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.db-layout {
  display: flex;
  flex: 1;
  overflow: hidden;
  min-width: 0;
}

.db-sidebar {
  width: 260px;
  min-width: 200px;
  max-width: 400px;
  border-right: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  background: var(--color-base-100);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.db-sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  font-size: 13px;
  font-weight: 600;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.db-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 0;
}

.db-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 48px 24px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  text-align: center;
}

.db-empty svg {
  opacity: 0.3;
}

.db-empty-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-base-content);
  margin: 0;
}

.db-empty-sub {
  font-size: 13px;
  margin: 0;
}

/* Tab bar */
.tab-bar {
  display: flex;
  align-items: flex-end;
  gap: 1px;
  padding: 0 8px;
  background: var(--color-base-200);
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  overflow-x: auto;
  min-height: 34px;
}

.tab-bar::-webkit-scrollbar {
  height: 2px;
}

.tab-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  min-width: 100px;
  max-width: 200px;
  border-radius: 6px 6px 0 0;
  cursor: pointer;
  font-size: 12px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  background: transparent;
  border: 1px solid transparent;
  border-bottom: none;
  transition: all 0.1s ease;
  user-select: none;
}

.tab-item:hover {
  background: var(--color-base-100);
  color: var(--color-base-content);
}

.tab-item.active {
  background: var(--color-base-200);
  color: var(--color-base-content);
  border-color: color-mix(in oklab, var(--color-base-content) 10%, transparent);
  font-weight: 500;
}

.tab-icon {
  font-size: 13px;
  flex-shrink: 0;
}

.tab-title {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tab-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border: none;
  background: transparent;
  border-radius: 3px;
  cursor: pointer;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  flex-shrink: 0;
  opacity: 0;
  transition: all 0.1s ease;
}

.tab-item:hover .tab-close {
  opacity: 0.6;
}

.tab-close:hover {
  opacity: 1 !important;
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
  color: var(--color-primary);
}

/* Tab content */
.tab-content {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  background: var(--color-base-200);
  min-width: 0;
}

/* SQL workspace */
.sql-workspace {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 16px;
  gap: 12px;
  overflow: hidden;
}

/* Table workspace */
.table-workspace {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 16px;
  gap: 12px;
  overflow: hidden;
  max-width: 100%;
  box-sizing: border-box;
}

.workspace-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 8px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.workspace-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-base-content);
}

.workspace-subtitle {
  font-size: 12px;
  font-weight: 400;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  margin-left: 8px;
}

.workspace-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

/* Redis workspace */
.redis-workspace {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.redis-console-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 16px;
  gap: 8px;
  overflow: hidden;
}

.redis-output {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
  background: var(--color-base-200);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 8px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
  line-height: 1.6;
}

.redis-msg {
  padding: 2px 0;
}

.redis-msg.input {
  color: var(--color-primary);
}

.redis-msg.output {
  color: var(--color-base-content);
}

.redis-msg.error {
  color: var(--color-error);
}

.redis-msg-prefix {
  font-weight: 600;
}

.redis-msg-content {
  word-break: break-all;
}

.redis-empty {
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  text-align: center;
  padding: 24px;
  font-style: italic;
}

.redis-input-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.redis-prompt {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 14px;
  font-weight: 700;
  color: var(--color-primary);
}

.redis-input {
  flex: 1;
  padding: 8px 12px;
  border: 1.5px solid color-mix(in oklab, var(--color-base-content) 20%, transparent);
  border-radius: 6px;
  background: var(--color-base-200);
  color: var(--color-base-content);
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  outline: none;
  transition: border-color 0.15s ease;
}

.redis-input:focus {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px color-mix(in oklab, var(--color-primary) 10%, transparent);
}

.redis-input::placeholder {
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  opacity: 0.5;
}

.btn-sm {
  padding: 4px 10px;
  font-size: 12px;
  line-height: 1;
  border-radius: 4px;
}
</style>
