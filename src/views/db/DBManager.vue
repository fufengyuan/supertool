<template>
  <div class="flex flex-col h-full min-h-0 max-w-full overflow-hidden bg-base-200">
    <!-- Header bar -->
    <div class="flex items-center justify-between px-4 py-2 bg-base-100 border-b border-base-content/10 gap-3">
      <div class="flex items-center gap-3 min-w-0">
        <h2 class="text-sm font-semibold text-base-content m-0 whitespace-nowrap flex items-center gap-2">
          <span class="w-7 h-7 rounded-lg bg-primary/10 text-primary flex items-center justify-center">
            <SvgIcon name="database" size="14" />
          </span>
          <span>数据库</span>
        </h2>
        <span v-if="db.activeConnection.value" class="flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[11px] font-medium border"
          :class="connectionBadgeClass">
          <span class="w-1.5 h-1.5 rounded-full animate-pulse" :class="connectionDotClass"></span>
          <span class="truncate max-w-[180px]">{{ db.activeConnection.value.name }}</span>
          <span class="opacity-60 uppercase text-[9px]">{{ db.activeConnection.value.type }}</span>
        </span>
      </div>
      <div class="flex gap-1.5 shrink-0" v-if="db.activeConnection.value">
        <template v-if="db.activeConnection.value.type === 'redis'">
          <button @click="openRedisManager" class="btn btn-primary btn-sm gap-1.5" title="Redis 管理器">
            <SvgIcon name="key" size="14" />
            Redis 管理器
          </button>
        </template>
        <template v-else>
          <div class="flex gap-0.5 p-0.5 bg-base-200 rounded-lg">
            <button
              @click="db.openSqlTab(db.activeConnection.value.id, db.activeConnection.value.name)"
              class="btn btn-ghost btn-sm rounded-md gap-1.5 px-3"
              title="新建查询"
            >
              <SvgIcon name="pencil" size="14" />
              新建查询
            </button>
            <button
              @click="db.openStructureSyncTab()"
              class="btn btn-ghost btn-sm rounded-md gap-1.5 px-3"
              title="结构同步"
            >
              <SvgIcon name="tool" size="14" />
              结构同步
            </button>
            <button
              @click="db.openDataSyncTab()"
              class="btn btn-ghost btn-sm rounded-md gap-1.5 px-3"
              title="数据同步"
            >
              <SvgIcon name="package" size="14" />
              数据同步
            </button>
            <button
              @click="openBackupTab"
              class="btn btn-ghost btn-sm rounded-md gap-1.5 px-3"
              title="数据库备份"
            >
              <SvgIcon name="folder" size="14" />
              备份
            </button>
          </div>
        </template>
      </div>
    </div>

    <div class="flex flex-1 overflow-hidden min-w-0">
      <!-- Left sidebar: Connection tree -->
      <aside class="w-72 min-w-[260px] max-w-[500px] border-r border-base-content/10 bg-base-100 flex flex-col overflow-hidden">
        <div class="flex items-center justify-between px-3 py-2.5 border-b border-base-content/10">
          <div class="flex items-center gap-2 text-xs font-semibold text-base-content/70">
            <SvgIcon name="server" size="14" class="text-base-content/40" />
            <span>连接</span>
            <span class="text-[10px] font-normal text-base-content/30 bg-base-200 px-1.5 py-0.5 rounded-full">{{ db.sortedConnections.value.length }}</span>
          </div>
          <button @click="db.openAddForm()" class="w-6 h-6 flex items-center justify-center rounded-md bg-primary/10 text-primary hover:bg-primary/20 transition-colors cursor-pointer border-0" title="添加连接">
            <SvgIcon name="plus" size="12" />
          </button>
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
          @delete-table="handleDeleteTable"
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
      <main class="flex-1 flex flex-col overflow-hidden min-w-0">
        <!-- No connection selected -->
        <div v-if="!db.activeConnection.value" class="flex-1 flex items-center justify-center p-8 bg-gradient-to-b from-base-100 to-base-200">
          <div class="flex flex-col items-center gap-5 max-w-[320px] text-center">
            <div class="w-20 h-20 rounded-2xl bg-primary/10 border border-primary/20 flex items-center justify-center shadow-sm">
              <SvgIcon name="database" size="40" stroke-width="1.5" class="text-primary/60" />
            </div>
            <div>
              <p class="text-base font-semibold text-base-content m-0 mb-1.5">选择或添加数据库连接</p>
              <p class="text-xs text-base-content/50 m-0 leading-relaxed">从左侧选择一个已保存的连接，或添加一个新的数据库连接开始工作</p>
            </div>
            <button @click="db.openAddForm()" class="btn btn-primary btn-sm gap-1.5">
              <SvgIcon name="plus" size="14" /> 添加连接
            </button>
          </div>
        </div>

        <!-- No tabs open -->
        <div v-else-if="db.tabs.value.length === 0" class="flex-1 flex items-center justify-center p-8 bg-gradient-to-b from-base-100 to-base-200">
          <template v-if="db.activeConnection.value?.type === 'redis'">
            <div class="flex flex-col items-center gap-5 max-w-[320px] text-center">
              <div class="w-20 h-20 rounded-2xl bg-red-500/10 border border-red-500/20 flex items-center justify-center shadow-sm">
                <SvgIcon name="key" size="40" stroke-width="1.5" class="text-red-500/60" />
              </div>
              <div>
                <p class="text-base font-semibold text-base-content m-0 mb-1.5">Redis 连接已就绪</p>
                <p class="text-xs text-base-content/50 m-0 leading-relaxed">浏览和管理 Redis 键值数据</p>
              </div>
              <button @click="openRedisManager" class="btn btn-primary btn-sm gap-1.5">
                <SvgIcon name="key" size="14" /> 打开 Redis 管理器
              </button>
            </div>
          </template>
          <template v-else>
            <div class="flex flex-col items-center gap-5 max-w-[320px] text-center">
              <div class="w-20 h-20 rounded-2xl bg-primary/10 border border-primary/20 flex items-center justify-center shadow-sm">
                <SvgIcon name="file" size="40" stroke-width="1.5" class="text-primary/60" />
              </div>
              <div>
                <p class="text-base font-semibold text-base-content m-0 mb-1.5">打开一个工作区</p>
                <p class="text-xs text-base-content/50 m-0 leading-relaxed">从左侧树中选择一个表，或点击"新建查询"开始</p>
              </div>
              <button
                @click="db.openSqlTab(db.activeConnection.value.id, db.activeConnection.value.name)"
                class="btn btn-primary btn-sm gap-1.5"
              >
                <SvgIcon name="pencil" size="14" /> 新建查询
              </button>
            </div>
          </template>
        </div>

        <!-- Tabbed workspace -->
        <template v-else>
          <!-- Tab bar -->
          <div class="flex items-center px-2 py-1.5 bg-base-100 border-b border-base-content/10 overflow-x-auto gap-1">
            <div
              v-for="(tab, idx) in db.tabs.value"
              :key="tab.id"
              class="group flex items-center gap-2 pl-2.5 pr-1.5 py-1.5 min-w-[100px] max-w-[200px] cursor-pointer text-xs select-none rounded-md transition-all"
              :class="[db.activeTabIndex.value === idx
                ? 'bg-primary/10 text-primary border border-primary/20'
                : 'text-base-content/60 hover:text-base-content hover:bg-base-200/70 border border-transparent']"
              @click="db.setActiveTab(idx)"
            >
              <SvgIcon :name="getTabIcon(tab)" size="13" class="shrink-0" :class="getTabIconClass(tab)" />
              <span class="flex-1 overflow-hidden text-ellipsis whitespace-nowrap font-medium" :title="tab.title">{{ tab.title }}</span>
              <button class="flex items-center justify-center w-5 h-5 border-0 bg-transparent rounded-md cursor-pointer text-base-content/40 shrink-0 opacity-0 group-hover:opacity-100 hover:text-error hover:bg-error/10 transition-all" @click.stop="db.closeTab(tab.id)" title="关闭">
                <SvgIcon name="x" size="11" stroke-width="2" />
              </button>
            </div>
          </div>

          <!-- Active tab content -->
          <div class="flex-1 overflow-hidden flex flex-col min-w-0 bg-base-200">
            <!-- SQL Query tab -->
            <template v-if="activeTab?.type === 'sql'">
              <div class="flex flex-col h-full gap-0 overflow-hidden">
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
                  :paginated="false"
                  :column-comments="columnComments"
                  :sort-column="activeSort?.column || null"
                  :sort-direction="activeSort?.direction || 'asc'"
                  @prev-page="handlePrevPage"
                  @next-page="handleNextPage"
                  @page-size-change="handlePageSizeChange"
                  @jump-page="handleJumpPage"
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
              <div class="flex flex-col h-full overflow-hidden">
                <div class="flex items-center justify-between px-3 py-2 border-b border-base-content/10 bg-base-100 shrink-0">
                  <div class="flex items-center gap-2 min-w-0">
                    <span class="w-6 h-6 rounded-md bg-primary/10 text-primary flex items-center justify-center shrink-0">
                      <SvgIcon name="barChart" size="12" />
                    </span>
                    <span class="text-sm font-semibold text-base-content truncate">{{ activeTab.tableName }}</span>
                    <span class="text-xs text-base-content/30">·</span>
                    <span class="text-xs text-base-content/50 truncate">{{ activeTab.connectionName }}</span>
                    <span v-if="resultTotal > 0" class="text-[10px] text-base-content/40 px-1.5 py-0.5 rounded bg-base-200">{{ resultTotal }} 行</span>
                  </div>
                  <button
                    @click="loadTableData"
                    class="btn btn-ghost btn-sm gap-1.5"
                    :disabled="tableLoading"
                  >
                    <SvgIcon name="refresh" size="13" :class="{ 'animate-spin': tableLoading }" />
                    刷新
                  </button>
                </div>
                <DataGrid
                  :rows="resultRows"
                  :total="resultTotal"
                  :page="currentPage"
                  :page-size="pageSize"
                  :loading="tableLoading"
                  :primary-key-columns="tablePrimaryKeyColumns"
                  :column-comments="columnComments"
                  :sort-column="activeSort?.column || null"
                  :sort-direction="activeSort?.direction || 'asc'"
                  @prev-page="handlePrevPage"
                  @next-page="handleNextPage"
                  @page-size-change="handlePageSizeChange"
                  @jump-page="handleJumpPage"
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
              <div class="flex flex-col h-full overflow-hidden">
                <div class="flex items-center gap-2 px-3 py-2 border-b border-base-content/10 bg-base-100 shrink-0">
                  <span class="w-6 h-6 rounded-md bg-red-500/10 text-red-500 flex items-center justify-center shrink-0">
                    <SvgIcon name="terminal" size="12" />
                  </span>
                  <span class="text-sm font-semibold text-base-content">Redis 控制台</span>
                  <span class="text-xs text-base-content/30">·</span>
                  <span class="text-xs text-base-content/50 truncate">{{ activeTab.connectionName }}</span>
                  <span v-if="redisExecuting" class="ml-auto text-[10px] text-base-content/40 flex items-center gap-1">
                    <span class="loading loading-spinner loading-xs"></span> 执行中
                  </span>
                </div>
                <div class="flex-1 flex flex-col p-3 gap-2 overflow-hidden bg-base-200">
                  <div class="flex-1 overflow-y-auto p-3 bg-[#1e1e2e] border border-base-content/10 rounded-lg font-mono text-xs leading-relaxed text-base-content/90 shadow-inner" ref="redisOutputRef">
                    <div v-for="(msg, idx) in redisMessages" :key="idx" class="py-0.5" :class="{'text-emerald-400': msg.type === 'input', 'text-base-content/90': msg.type === 'output', 'text-red-400': msg.type === 'error'}">
                      <span class="font-semibold opacity-70">{{ msg.prefix }}</span>
                      <span class="break-all">{{ msg.content }}</span>
                    </div>
                    <div v-if="redisMessages.length === 0" class="text-base-content/30 text-center p-8 italic">
                      输入 Redis 命令，例如: GET key, KEYS *, INFO
                    </div>
                  </div>
                  <div class="flex items-center gap-2 bg-[#1e1e2e] border border-base-content/10 rounded-lg px-3 py-2">
                    <span class="font-mono text-sm font-bold text-emerald-400">&gt;</span>
                    <input
                      v-model="redisCommand"
                      @keydown.enter="executeRedis"
                      @keydown.arrow-up.prevent="redisHistoryUp"
                      @keydown.arrow-down.prevent="redisHistoryDown"
                      class="flex-1 bg-transparent border-0 outline-none font-mono text-xs text-base-content placeholder:text-base-content/30"
                      placeholder="输入 Redis 命令... (↑/↓ 切换历史)"
                      spellcheck="false"
                    />
                    <button @click="executeRedis" class="btn btn-primary btn-xs gap-1" :disabled="redisExecuting">
                      <SvgIcon name="play" size="10" /> 执行
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
                :connection="db.connections.value.find(c => c.id === activeTab!.connectionId)"
                :initial-key="activeTab.initialKey"
                :redis-db-index="activeTab.redisDbIndex"
              />
            </template>

            <!-- Redis Queue tab -->
            <template v-else-if="activeTab?.type === 'redisQueue'">
              <RedisQueueManager
                :connection-id="activeTab.connectionId"
                :connection-name="activeTab.connectionName"
                :connection="db.connections.value.find(c => c.id === activeTab!.connectionId)"
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

    <!-- Confirm dialog (replaces native confirm) -->
    <Modal :model-value="confirmDialog.show" @update:model-value="(v: boolean) => confirmDialog.show = v" :title="confirmDialog.title" width="440px">
      <div class="flex gap-3 items-start">
        <div class="w-9 h-9 rounded-full flex items-center justify-center shrink-0"
          :class="confirmDialog.danger ? 'bg-error/10 text-error' : 'bg-warning/10 text-warning'">
          <SvgIcon :name="confirmDialog.danger ? 'alertTriangle' : 'alertCircle'" size="18" />
        </div>
        <p class="text-sm text-base-content/80 m-0 whitespace-pre-wrap leading-relaxed">{{ confirmDialog.message }}</p>
      </div>
      <template #footer>
        <button class="btn btn-ghost btn-sm" @click="confirmDialog.show = false">取消</button>
        <button class="btn btn-sm gap-1.5" :class="confirmDialog.danger ? 'btn-error' : 'btn-primary'" @click="handleConfirm">
          <SvgIcon name="check" size="14" /> 确认
        </button>
      </template>
    </Modal>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'DBManager' })
import SvgIcon from '@/components/ui/SvgIcon.vue'
import Modal from '@/components/ui/Modal.vue'
import * as logger from '../../services/logger'
import { ref, computed, watch, nextTick, onMounted } from 'vue'
import { useDBManager, type DBConnection, type WorkspaceTab } from '../../composables/useDBManager'
import ConnectionTree from './ConnectionTree.vue'
import ConnectionForm from './ConnectionForm.vue'
import SqlEditor from './SqlEditor.vue'
import DataGrid from './DataGrid.vue'
import TableStructure from './TableStructure.vue'
import RedisManager from './RedisManager.vue'
import RedisQueueManager from './RedisQueueManager.vue'
import StructureSync from './StructureSync.vue'
import DataSync from './DataSync.vue'
import DBBackup from './DBBackup.vue'
import type { FilterCondition } from './FilterBar.vue'
import { useToast } from '../../composables/useToast'
import { getTauriAPI } from '../../utils/tauri-api'

const db = useDBManager()
const toast = useToast()
const treeRef = ref<InstanceType<typeof ConnectionTree> | null>(null)

// Convert Date objects and ISO strings to MySQL-compatible format before IPC
// JSON.stringify turns Date into ISO string ('2026-04-21T16:00:00.000Z') which MySQL DATE columns reject
function sanitizeForDB(obj: Record<string, unknown> | null): Record<string, unknown> | null {
  if (!obj) {return obj}
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
const pageSize = ref(100)
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
  user: '',
  password: '',
  database: ''
})

// Redis console state
const redisCommand = ref('')
const redisExecuting = ref(false)
const redisMessages = ref<Array<{ type: string; prefix: string; content: string }>>([])
const redisHistory = ref<string[]>([])
const redisHistoryIndex = ref(-1)

// Confirm dialog state (replaces native confirm())
const confirmDialog = ref<{ show: boolean; title: string; message: string; danger: boolean; onConfirm: (() => void) | null }>({
  show: false, title: '', message: '', danger: false, onConfirm: null,
})

function showConfirm(title: string, message: string, onConfirm: () => void, danger = false) {
  confirmDialog.value = { show: true, title, message, danger, onConfirm }
}

function handleConfirm() {
  confirmDialog.value.onConfirm?.()
  confirmDialog.value.show = false
  confirmDialog.value.onConfirm = null
}

const activeTab = computed(() => db.activeTab.value)

// Database type → badge/dot color classes (continues tree colors into header)
const connectionBadgeClass = computed(() => {
  const t = db.activeConnection.value?.type
  switch (t) {
    case 'redis': return 'bg-red-500/10 text-red-500 border-red-500/20'
    case 'sqlite': return 'bg-emerald-500/10 text-emerald-500 border-emerald-500/20'
    case 'postgresql': return 'bg-blue-500/10 text-blue-500 border-blue-500/20'
    default: return 'bg-primary/10 text-primary border-primary/20'
  }
})
const connectionDotClass = computed(() => {
  const t = db.activeConnection.value?.type
  switch (t) {
    case 'redis': return 'bg-red-500'
    case 'sqlite': return 'bg-emerald-500'
    case 'postgresql': return 'bg-blue-500'
    default: return 'bg-primary'
  }
})

function getTabIcon(tab: WorkspaceTab | null): string {
  if (!tab) {return 'file'}
  switch (tab.type) {
    case 'sql': return 'pencil'
    case 'tableData': return 'barChart'
    case 'tableStructure': return 'grid'
    case 'redisConsole': return 'terminal'
    case 'redisManager': return 'key'
    case 'structureSync': return 'tool'
    case 'dataSync': return 'package'
    case 'backup': return 'archive'
    default: return 'file'
  }
}

function getTabIconClass(tab: WorkspaceTab | null): string {
  if (!tab) {return ''}
  switch (tab.type) {
    case 'redisConsole': return 'text-red-500'
    case 'redisManager': return 'text-red-500'
    case 'backup': return 'text-orange-500'
    case 'tableStructure': return 'text-primary'
    default: return ''
  }
}

function getTabConnection(tab: WorkspaceTab | null): DBConnection | null {
  if (!tab) {return null}
  return db.connections.value.find(c => c.id === tab.connectionId) || null
}

function getTabDbType(tab: WorkspaceTab | null): 'mysql' | 'postgresql' | 'sqlite' | undefined {
  if (!tab) {return undefined}
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
    user: '',
    password: '',
    database: ''
  }
}

// Watch editing connection changes
watch(() => db.editingConnection.value, (conn) => {
  if (conn) {
    connectionForm.value = { ...conn, user: conn.user || '' }
  } else {
    resetForm()
  }
})

async function handleSaveConnection(form: DBConfig) {
  if (!form.name.trim()) {
    toast.warning('请输入连接名称')
    return
  }

  if (db.editingConnection.value) {
    db.updateConnection(db.editingConnection.value.id, form)
    toast.success('连接已更新')
  } else {
    const conn = db.addConnection(form)
    db.setActiveConnection(conn.id)
    toast.success('连接已添加')
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
  showConfirm(
    '删除连接',
    '确定要删除此连接吗？此操作不可撤销。',
    () => {
      db.deleteConnection(id)
      toast.success('连接已删除')
    },
    true,
  )
}

function openRedisManager() {
  if (!db.activeConnection.value) {return}
  db.openRedisManagerTab(db.activeConnection.value.id, db.activeConnection.value.name)
}

function openBackupTab() {
  if (!db.activeConnection.value) {return}
  db.openBackupTab(db.activeConnection.value.id, db.activeConnection.value.name)
}

function handleOpenRedisQueue(connId: string, dbIndex: number) {
  const conn = db.connections.value.find(c => c.id === connId)
  if (conn) {
    db.openRedisQueueTab(connId, conn.name, dbIndex)
  }
}

async function handleExecute(sqlText: string) {
  if (!db.activeConnection.value) {return}

  // Check if connection requires approval
  if ((db.activeConnection.value as any).requiresApproval) {
    const preview = sqlText.length > 300 ? sqlText.substring(0, 300) + '...' : sqlText
    showConfirm(
      '安全审核',
      `数据库「${db.activeConnection.value.name}」已开启安全审核，请确认执行以下 SQL：\n\n${preview}`,
      () => doExecute(sqlText),
      false,
    )
    return
  }

  doExecute(sqlText)
}

async function doExecute(sqlText: string) {
  if (!db.activeConnection.value) {return}
  executing.value = true
  error.value = null
  currentPage.value = 1

  const startTime = performance.now()
  try {
    // ⚠️ 剥离 Vue Proxy，否则 Tauri IPC 的 structuredClone 会失败
    const plainSql = sqlText
    // 只读语句走 dbQuery（白名单），写语句走 dbExecuteWrite（GUI 写通道）
    const upper = plainSql.trim().toUpperCase()
    const isReadOnly = ['SELECT', 'SHOW', 'EXPLAIN', 'DESC', 'DESCRIBE', 'PRAGMA'].some(p => {
      return upper === p || (upper.startsWith(p) && (upper.length === p.length || /\s/.test(upper.slice(p.length, p.length + 1))))
    })
    const queryResult = isReadOnly
      ? await getTauriAPI().dbQuery(db.activeConnection.value.id, plainSql)
      : await getTauriAPI().dbExecuteWrite(db.activeConnection.value.id, plainSql)
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
      error: error.value ?? undefined,
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
  toast.success('表列表已刷新')
}

async function handleDeleteTable(connId: string, table: string, dbName?: string) {
  const conn = db.connections.value.find(c => c.id === connId)
  if (!conn) {return}

  // Check if connection requires approval
  if ((conn as any).requiresApproval) {
    toast.error('此连接已开启安全审核，删除表操作被禁止')
    return
  }

  showConfirm(
    '删除表',
    `确定要删除表「${table}」吗？\n\n此操作不可撤销，表中的所有数据都将被永久删除。`,
    async () => {
      try {
        // Build DROP TABLE SQL based on database type
        let sql: string
        if (conn.type === 'mysql') {
          const tableRef = dbName ? `\`${dbName}\`.\`${table}\`` : `\`${table}\``
          sql = `DROP TABLE ${tableRef}`
        } else if (conn.type === 'postgresql') {
          const tableRef = dbName ? `${dbName}.${table}` : table
          sql = `DROP TABLE ${tableRef}`
        } else if (conn.type === 'sqlite') {
          sql = `DROP TABLE "${table}"`
        } else {
          toast.error('不支持此数据库类型的删除表操作')
          return
        }

        const result = await getTauriAPI().dbExecuteWrite(connId, sql)
        if (result && typeof result === 'object' && 'success' in result && (result as any).success) {
          toast.success(`表「${table}」已删除`)
          // Refresh table list
          treeRef.value?.refreshTables(connId)
        } else {
          toast.error('删除失败: ' + ((result as any)?.error || '未知错误'))
        }
      } catch (e: any) {
        toast.error('删除失败: ' + (e?.message || '未知错误'))
      }
    },
    true,
  )
}

async function fetchColumnComments(connId: string, table: string, dbName?: string) {
  try {
    const conn = db.connections.value.find(c => c.id === connId)
    if (!conn || conn.type === 'redis') {return}

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
  if (!activeTab.value || activeTab.value.type !== 'tableData') {return}
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
        limit: pageSize.value,
        offset: (currentPage.value - 1) * pageSize.value,
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
        connId, table, pageSize: pageSize.value, offset: (currentPage.value - 1) * pageSize.value, dbName,
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
        const rows = await getTauriAPI().dbQuery(connId, `SELECT * FROM ${tableRef} LIMIT ${pageSize.value} OFFSET ${(currentPage.value - 1) * pageSize.value}`)
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

function handlePageSizeChange(size: number) {
  pageSize.value = size
  currentPage.value = 1
  loadTableData()
}

function handleJumpPage(page: number) {
  currentPage.value = page
  loadTableData()
}

// ============ Table Row CRUD ============

const tablePrimaryKeyColumns = ref<string[]>([])

async function loadTablePrimaryKeys() {
  if (!activeTab.value || activeTab.value.type !== 'tableData') {return}
  const tab = activeTab.value
  try {
    const res = await getTauriAPI().dbGetTablePrimaryKeys(tab.connectionId, tab.tableName || '', tab.dbName || '')
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
  if (!activeTab.value || activeTab.value.type !== 'tableData') {return}
  const tab = activeTab.value
  try {
    const plainOld = sanitizeForDB(oldRow)
    const plainNew = sanitizeForDB(newRow)
    const res = await getTauriAPI().dbUpdateTableRow(tab.connectionId, tab.tableName || '', plainOld!, plainNew!, tab.dbName)
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
    if (ok) {successCount++}
    else {failCount++}
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
  if (!activeTab.value || activeTab.value.type !== 'tableData') {return}
  const tab = activeTab.value
  try {
    const plainRow = sanitizeForDB(row)
    const res = await getTauriAPI().dbInsertTableRow(tab.connectionId, tab.tableName || '', plainRow!, tab.dbName)
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
  if (!activeTab.value || activeTab.value.type !== 'tableData') {return}
  const tab = activeTab.value
  try {
    const plainRow = sanitizeForDB(row)
    const res = await getTauriAPI().dbDeleteTableRow(tab.connectionId, tab.tableName || '', plainRow!, tab.dbName)
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
  if (!redisCommand.value.trim() || !activeTab.value) {return}

  const cmd = redisCommand.value.trim()
  redisMessages.value.push({ type: 'input', prefix: '> ', content: cmd })
  // Push to history (dedupe consecutive duplicates)
  if (redisHistory.value[redisHistory.value.length - 1] !== cmd) {
    redisHistory.value.push(cmd)
    if (redisHistory.value.length > 50) {redisHistory.value.shift()}
  }
  redisHistoryIndex.value = -1
  redisCommand.value = ''
  redisExecuting.value = true

  try {
    if (!activeTab.value) {return}
    const connId = activeTab.value.connectionId
    const result = await getTauriAPI().dbRedisExec(connId, activeTab.value.redisDbIndex || 0, cmd)
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

// Redis command history navigation (↑/↓)
function redisHistoryUp() {
  if (redisHistory.value.length === 0) {return}
  if (redisHistoryIndex.value === -1) {
    redisHistoryIndex.value = redisHistory.value.length - 1
  } else if (redisHistoryIndex.value > 0) {
    redisHistoryIndex.value--
  }
  redisCommand.value = redisHistory.value[redisHistoryIndex.value] || ''
}

function redisHistoryDown() {
  if (redisHistoryIndex.value === -1) {return}
  if (redisHistoryIndex.value < redisHistory.value.length - 1) {
    redisHistoryIndex.value++
    redisCommand.value = redisHistory.value[redisHistoryIndex.value] || ''
  } else {
    redisHistoryIndex.value = -1
    redisCommand.value = ''
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


