<template>
  <div class="data-sync">
    <div class="sync-header">
      <h3 class="sync-title">📦 数据同步</h3>
      <p class="sync-desc">对比并同步两个数据库之间的表数据（Navicat 风格多表对比）</p>
    </div>

    <!-- Step 1: Connection & Database Selection -->
    <div class="sync-config" v-if="step === 1">
      <div class="config-grid">
        <div class="config-item">
          <label>🟢 源连接</label>
          <select v-model="sourceId" @change="onSourceChange" class="select-input">
            <option value="">选择源连接</option>
            <option v-for="conn in connections" :key="conn.id" :value="conn.id">
              {{ conn.name }} ({{ conn.type }})
            </option>
          </select>
        </div>
        <div class="config-item">
          <label>🔴 目标连接</label>
          <select v-model="targetId" @change="onTargetChange" class="select-input">
            <option value="">选择目标连接</option>
            <option v-for="conn in connections" :key="conn.id" :value="conn.id">
              {{ conn.name }} ({{ conn.type }})
            </option>
          </select>
        </div>
        <div class="config-item">
          <label>📂 源数据库</label>
          <select v-model="sourceDb" @change="loadSourceTables" :disabled="!sourceId || loadingSourceDb" class="select-input">
            <option value="">{{ loadingSourceDb ? '加载中...' : '选择数据库' }}</option>
            <option v-for="db in sourceDatabases" :key="db" :value="db">{{ db }}</option>
          </select>
        </div>
        <div class="config-item">
          <label>📂 目标数据库</label>
          <select v-model="targetDb" @change="loadTargetTables" :disabled="!targetId || loadingTargetDb" class="select-input">
            <option value="">{{ loadingTargetDb ? '加载中...' : '选择数据库' }}</option>
            <option v-for="db in targetDatabases" :key="db" :value="db">{{ db }}</option>
          </select>
        </div>
        <div class="config-item">
          <label>同步模式</label>
          <select v-model="syncMode" class="select-input">
            <option value="full">完整同步（INSERT + UPDATE + DELETE）</option>
            <option value="insert_only">仅插入（INSERT only）</option>
            <option value="update_only">仅更新（UPDATE only）</option>
          </select>
        </div>
        <div class="config-item checkbox-item">
          <label class="checkbox-label">
            <input type="checkbox" v-model="useTransaction" />
            使用事务
          </label>
        </div>
      </div>

      <div class="sync-actions">
        <button @click="goToStep2" :disabled="!sourceId || !targetId || !sourceDb || !targetDb" class="btn btn-primary">
          下一步：选择表 →
        </button>
      </div>
    </div>

    <!-- Step 2: Multi-Table Selection -->
    <div class="sync-config" v-if="step === 2">
      <div class="table-select-header">
        <h4 class="table-select-title">📋 选择要对比数据的表</h4>
        <div class="table-select-controls">
          <button @click="selectAllTables" class="btn btn-ghost btn-xs">全选</button>
          <button @click="selectCommonTables" class="btn btn-ghost btn-xs">仅共有表</button>
          <button @click="selectNone" class="btn btn-ghost btn-xs">清空</button>
        </div>
      </div>

      <div class="table-grid">
        <!-- Common tables only (data sync requires same table in both) -->
        <template v-if="commonTablesList.length > 0">
          <div class="table-section-label">共有表（{{ commonTablesList.length }}）</div>
          <div v-for="table in commonTablesList" :key="'common-' + table" class="table-checkbox">
            <label>
              <input type="checkbox" v-model="selectedTables" :value="table" />
              <span class="table-name">{{ table }}</span>
              <span class="table-badge badge-common">共有</span>
            </label>
          </div>
        </template>

        <!-- Source-only tables -->
        <template v-if="sourceOnlyTables.length > 0">
          <div class="table-section-label">仅源端有（无法同步数据）</div>
          <div v-for="table in sourceOnlyTables" :key="'src-' + table" class="table-checkbox disabled-table">
            <label>
              <span class="table-name">{{ table }}</span>
              <span class="table-badge badge-source">仅源</span>
            </label>
          </div>
        </template>

        <!-- Target-only tables -->
        <template v-if="targetOnlyTables.length > 0">
          <div class="table-section-label">仅目标端有（无法同步数据）</div>
          <div v-for="table in targetOnlyTables" :key="'tgt-' + table" class="table-checkbox disabled-table">
            <label>
              <span class="table-name">{{ table }}</span>
              <span class="table-badge badge-target">仅目标</span>
            </label>
          </div>
        </template>

        <div v-if="commonTablesList.length === 0" class="table-empty">
          两个数据库没有共有的表，无法进行数据同步
        </div>
      </div>

      <!-- Compare key configuration per table (auto-detected PKs, editable with column dropdown) -->
      <div v-if="selectedTables.length > 0" class="pk-config">
        <h4 class="pk-title">🔑 对比字段 <span class="pk-hint">（默认主键，可切换为其他业务字段）</span></h4>
        <div v-for="table in selectedTables" :key="'pk-' + table" class="pk-row">
          <span class="pk-table-name">{{ table }}</span>
          <!-- Multi-select dropdown for columns -->
          <div class="pk-select-wrapper" v-if="tableColumns[table]">
            <div
              class="pk-select"
              @click="toggleDropdown(table)"
              :class="{ open: openDropdown === table }"
            >
              <span v-if="compareKeys[table]?.length" class="pk-tags">
                <span
                  v-for="key in compareKeys[table]"
                  :key="key"
                  class="pk-tag"
                  :class="{ 'is-pk': tablePrimaryKeys[table]?.includes(key) }"
                >
                  {{ key }}
                  <span class="pk-tag-remove" @click.stop="removeCompareKey(table, key)">×</span>
                </span>
              </span>
              <span v-else class="pk-placeholder">选择对比字段</span>
              <span class="pk-arrow">▾</span>
            </div>
            <!-- Dropdown menu -->
            <div v-if="openDropdown === table" class="pk-dropdown">
              <div
                v-for="col in tableColumns[table]"
                :key="col"
                class="pk-option"
                :class="{ selected: compareKeys[table]?.includes(col) }"
                @click="toggleCompareKey(table, col)"
              >
                <span class="pk-checkbox" :class="{ checked: compareKeys[table]?.includes(col) }">
                  {{ compareKeys[table]?.includes(col) ? '✓' : '' }}
                </span>
                <span class="pk-option-name">{{ col }}</span>
                <span v-if="tablePrimaryKeys[table]?.includes(col)" class="pk-badge">PK</span>
              </div>
            </div>
          </div>
          <!-- Loading state -->
          <span v-else class="pk-status loading">⏳ 加载中...</span>
        </div>
      </div>

      <div class="table-select-footer">
        <span class="selected-count-text">已选 {{ selectedTables.length }} 张表</span>
        <div class="footer-btns">
          <button @click="step = 1" class="btn btn-ghost">← 返回</button>
          <button @click="startCompare" :disabled="!canCompare || comparing" class="btn btn-primary">
            {{ comparing ? '对比中...' : `🔍 对比 ${selectedTables.length} 张表` }}
          </button>
        </div>
      </div>
    </div>

    <!-- Comparing State -->
    <div v-if="comparing" class="sync-loading">
      <div class="loading-spinner"></div>
      <p>正在对比表数据... ({{ compareProgress }}/{{ selectedTables.length }})</p>
    </div>

    <!-- Results -->
    <div v-if="result && !comparing" class="sync-results">
      <div class="results-summary">
        <h4>数据对比结果</h4>
        <div class="summary-stats">
          <span class="stat stat-insert">新增: {{ totalInserts }}</span>
          <span class="stat stat-update">更新: {{ totalUpdates }}</span>
          <span class="stat stat-delete">删除: {{ totalDeletes }}</span>
          <span class="stat stat-total">总计: {{ result.diffs.length }}</span>
        </div>
      </div>

      <!-- Diff Filter -->
      <div class="diff-filter">
        <label class="filter-label">
          <input type="checkbox" :checked="filterTypes.has('insert')" @change="toggleFilter('insert')" />
          <span class="filter-badge insert">新增 ({{ result.diffs.filter(d => d.diffType === 'insert').length }})</span>
        </label>
        <label class="filter-label">
          <input type="checkbox" :checked="filterTypes.has('update')" @change="toggleFilter('update')" />
          <span class="filter-badge update">更新 ({{ result.diffs.filter(d => d.diffType === 'update').length }})</span>
        </label>
        <label class="filter-label">
          <input type="checkbox" :checked="filterTypes.has('delete')" @change="toggleFilter('delete')" />
          <span class="filter-badge delete">删除 ({{ result.diffs.filter(d => d.diffType === 'delete').length }})</span>
        </label>
      </div>

      <!-- Filter by table -->
      <div class="results-filter" v-if="affectedTableList.length > 1">
        <span class="filter-label-text">按表筛选:</span>
        <label v-for="table in affectedTableList" :key="table" class="filter-chip">
          <input type="checkbox" v-model="filterTables" :value="table" />
          {{ table }}
        </label>
      </div>

      <!-- Navicat-style Grouped Data Diff View -->
      <div class="diff-grouped-view">
        <div v-for="(group, tIdx) in groupedDiffs" :key="tIdx" class="diff-table-group">
          <!-- Table group header -->
          <div class="diff-group-header" @click="toggleTableExpand(group.tableName)">
            <span class="expand-icon">{{ isTableExpanded(group.tableName) ? '▼' : '▶' }}</span>
            <span class="group-table-name">{{ group.tableName }}</span>
            <span class="group-diff-count">{{ group.diffs.length }} 行差异</span>
            <span class="group-type-badges">
              <span v-for="(count, type) in group.typeCounts" :key="type" class="mini-badge" :class="getDiffTypeBadgeClass(type)">
                {{ getDiffTypeLabel(type) }} ×{{ count }}
              </span>
            </span>
          </div>

          <!-- Expanded diff rows -->
          <div v-if="isTableExpanded(group.tableName)" class="diff-group-body">
            <table class="diff-compare-table">
              <thead>
                <tr>
                  <th class="col-pk">主键值</th>
                  <th class="col-source">源数据</th>
                  <th class="col-target">目标数据</th>
                  <th class="col-action">操作</th>
                </tr>
              </thead>
              <tbody>
                <template v-for="(diff, dIdx) in group.diffs" :key="dIdx">
                  <tr class="diff-row" :class="diff.diffType">
                    <td class="col-pk">
                      <span class="pk-display">{{ formatPrimaryKey(diff.primaryKey) }}</span>
                    </td>
                    <td class="col-source">
                      <template v-if="diff.sourceRow">
                        <span class="compact-row-preview">{{ getRowPreview(diff.sourceRow) }}</span>
                      </template>
                      <span v-else class="null-value">—</span>
                    </td>
                    <td class="col-target">
                      <template v-if="diff.targetRow">
                        <span class="compact-row-preview">{{ getRowPreview(diff.targetRow) }}</span>
                      </template>
                      <span v-else class="null-value">—</span>
                    </td>
                    <td class="col-action">
                      <button class="sql-toggle-btn" @click="toggleSqlRow(tIdx + '-' + dIdx)">
                        {{ isSqlRowExpanded(tIdx + '-' + dIdx) ? '收起' : '详情' }}
                      </button>
                    </td>
                  </tr>
                  <!-- Expandable detail row showing changed columns -->
                  <tr v-if="isSqlRowExpanded(tIdx + '-' + dIdx)" class="sql-detail-row">
                    <td colspan="4">
                      <div class="data-detail-panel">
                        <!-- For updates: show column-by-column comparison -->
                        <template v-if="diff.diffType === 'update' && diff.sourceRow && diff.targetRow">
                          <div class="detail-title">列值对比（仅显示变化的列）</div>
                          <table class="column-diff-table">
                            <thead>
                              <tr>
                                <th>列名</th>
                                <th>源值</th>
                                <th>目标值</th>
                              </tr>
                            </thead>
                            <tbody>
                              <tr v-for="col in getChangedColumns(diff.sourceRow, diff.targetRow)" :key="col.name" class="changed-col-row">
                                <td class="col-name">{{ col.name }}</td>
                                <td class="col-old">{{ formatCellValue(col.sourceVal) }}</td>
                                <td class="col-new">{{ formatCellValue(col.targetVal) }}</td>
                              </tr>
                              <tr v-if="getChangedColumns(diff.sourceRow, diff.targetRow).length === 0">
                                <td colspan="3" class="no-changes">所有列值相同</td>
                              </tr>
                            </tbody>
                          </table>
                        </template>
                        <!-- For inserts: show source row -->
                        <template v-else-if="diff.diffType === 'insert' && diff.sourceRow">
                          <div class="detail-title">新增行数据</div>
                          <pre class="row-json">{{ formatRow(diff.sourceRow) }}</pre>
                        </template>
                        <!-- For deletes: show target row -->
                        <template v-else-if="diff.diffType === 'delete' && diff.targetRow">
                          <div class="detail-title">待删除行数据</div>
                          <pre class="row-json">{{ formatRow(diff.targetRow) }}</pre>
                        </template>
                      </div>
                    </td>
                  </tr>
                </template>
              </tbody>
            </table>
          </div>
        </div>

        <div v-if="filteredDiffs.length === 0" class="diff-empty">
          所选表数据完全一致 ✅
        </div>
      </div>

      <!-- Execute Actions -->
      <div class="execute-actions">
        <span class="selected-count">将同步 {{ filteredDiffs.length }} 项更改</span>
        <div class="execute-btns">
          <button @click="reset" class="btn btn-ghost">重新对比</button>
          <button @click="showSqlDialog = true" :disabled="filteredDiffs.length === 0" class="btn btn-ghost">📄 查看SQL</button>
          <button @click="executeSync" :disabled="filteredDiffs.length === 0 || executing" class="btn btn-primary">
            {{ executing ? '执行中...' : '🚀 执行同步' }}
          </button>
        </div>
      </div>
    </div>

    <!-- SQL Preview Dialog -->
    <Teleport to="body">
      <div v-if="showSqlDialog" class="sql-dialog-overlay" @click="showSqlDialog = false">
        <div class="sql-dialog" @click.stop>
          <div class="sql-dialog-header">
            <h3>📄 待执行 SQL ({{ generatedSqlList.length }} 条)</h3>
            <div class="sql-dialog-actions">
              <button @click="copyAllSql" class="btn btn-ghost btn-sm">📋 复制全部</button>
              <button @click="showSqlDialog = false" class="sql-dialog-close">×</button>
            </div>
          </div>
          <div class="sql-dialog-body">
            <div v-for="(sql, idx) in generatedSqlList" :key="idx" class="sql-item">
              <span class="sql-number">{{ idx + 1 }}</span>
              <pre class="sql-text">{{ sql }}</pre>
              <button @click="copySingleSql(idx)" class="sql-copy-btn" title="复制">📋</button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Execution Result -->
    <div v-if="execResult" class="exec-result" :class="{ success: execResult.success, error: !execResult.success }">
      <h4>{{ execResult.success ? '✅ 同步成功' : '❌ 同步失败' }}</h4>
      <div class="exec-stats">
        <span>新增: {{ execResult.inserted }}</span>
        <span>更新: {{ execResult.updated }}</span>
        <span>删除: {{ execResult.deleted }}</span>
        <span>耗时: {{ (execResult.duration / 1000).toFixed(2) }}s</span>
      </div>
      <div v-if="execResult.errors.length > 0" class="exec-errors">
        <p>错误信息:</p>
        <ul>
          <li v-for="(err, idx) in execResult.errors" :key="idx">{{ err }}</li>
        </ul>
      </div>
      <button @click="reset" class="btn btn-primary">完成</button>
    </div>
  </div>
</template>

<script setup lang="ts">
console.log("[DataSync.vue] component loaded")
import { getTauriAPI } from '@/utils/tauri-api'
import { ref, computed, watch } from 'vue'
import { useDBManager } from '@/composables/useDBManager'
import { useToast } from '@/composables/useToast'

const db = useDBManager()
const toast = useToast()
const connections = computed(() => db.connections.value)

const step = ref(1)
const sourceId = ref('')
const targetId = ref('')
const sourceDb = ref('')
const targetDb = ref('')
const syncMode = ref('full')
const useTransaction = ref(true)
const comparing = ref(false)
const compareProgress = ref(0)
const executing = ref(false)
const loadingSourceDb = ref(false)
const loadingTargetDb = ref(false)

// Database lists
const sourceDatabases = ref<string[]>([])
const targetDatabases = ref<string[]>([])

// Table lists
const sourceTables = ref<string[]>([])
const targetTables = ref<string[]>([])

// Selected tables
const selectedTables = ref<string[]>([])

// Per-table data: columns and comparison keys
const tableColumns = ref<Record<string, string[]>>({})
const tablePrimaryKeys = ref<Record<string, string[]>>({})
const compareKeys = ref<Record<string, string[]>>({})

// Dropdown state for column selection
const openDropdown = ref<string | null>(null)

interface DataDiffItem {
  diffType: 'insert' | 'update' | 'delete'
  primaryKey: Record<string, any>
  sourceRow?: Record<string, any>
  targetRow?: Record<string, any>
  tableName: string
}

interface DataSyncResult {
  diffs: DataDiffItem[]
  totalInserts: number
  totalUpdates: number
  totalDeletes: number
}

const result = ref<DataSyncResult | null>(null)
const filterTypes = ref<Set<string>>(new Set(['insert', 'update', 'delete']))

// Sync mode → pre-filter diff types
watch(syncMode, (mode) => {
  if (mode === 'insert_only') {
    filterTypes.value = new Set(['insert'])
  } else if (mode === 'update_only') {
    filterTypes.value = new Set(['update'])
  } else {
    filterTypes.value = new Set(['insert', 'update', 'delete'])
  }
})

// Auto-fetch table metadata when tables are selected/deselected
watch(selectedTables, (newTables, oldTables) => {
  const added = newTables.filter(t => !oldTables.includes(t))
  if (added.length > 0) {
    autoFetchTableMeta(added)
  }
})
const filterTables = ref<string[]>([])
const execResult = ref<{
  success: boolean
  inserted: number
  updated: number
  deleted: number
  errors: string[]
  duration: number
} | null>(null)
const showSqlDialog = ref(false)

// Navicat-style: track expanded table groups and detail rows
const expandedTableGroups = ref<Set<string>>(new Set())
const expandedSqlRows = ref<Set<string>>(new Set())

// Computed — case-insensitive matching
const commonTablesList = computed(() => {
  const targetLower = new Set(targetTables.value.map(t => t.toLowerCase()))
  return sourceTables.value.filter(t => targetLower.has(t.toLowerCase()))
})

const sourceOnlyTables = computed(() => {
  const targetLower = new Set(targetTables.value.map(t => t.toLowerCase()))
  return sourceTables.value.filter(t => !targetLower.has(t.toLowerCase()))
})

const targetOnlyTables = computed(() => {
  const sourceLower = new Set(sourceTables.value.map(t => t.toLowerCase()))
  return targetTables.value.filter(t => !sourceLower.has(t.toLowerCase()))
})

const affectedTableList = computed(() => {
  if (!result.value) return []
  return [...new Set(result.value.diffs.map(d => d.tableName))]
})

const filteredDiffs = computed(() => {
  if (!result.value) return []
  let diffs = result.value.diffs.filter(d => filterTypes.value.has(d.diffType))
  if (filterTables.value.length > 0) {
    diffs = diffs.filter(d => filterTables.value.includes(d.tableName))
  }
  return diffs
})

const totalInserts = computed(() => {
  if (!result.value) return 0
  return result.value.diffs.filter(d => d.diffType === 'insert').length
})

const totalUpdates = computed(() => {
  if (!result.value) return 0
  return result.value.diffs.filter(d => d.diffType === 'update').length
})

const totalDeletes = computed(() => {
  if (!result.value) return 0
  return result.value.diffs.filter(d => d.diffType === 'delete').length
})

const canCompare = computed(() => {
  return selectedTables.value.length > 0 &&
    selectedTables.value.every(t => compareKeys.value[t] && compareKeys.value[t].length > 0)
})

// Navicat-style: group diffs by table
const groupedDiffs = computed(() => {
  const groups: { tableName: string; diffs: DataDiffItem[]; typeCounts: Record<string, number> }[] = []
  const tableMap = new Map<string, DataDiffItem[]>()

  for (const diff of filteredDiffs.value) {
    if (!tableMap.has(diff.tableName)) {
      tableMap.set(diff.tableName, [])
    }
    tableMap.get(diff.tableName)!.push(diff)
  }

  for (const [tableName, diffs] of tableMap) {
    const typeCounts: Record<string, number> = {}
    for (const diff of diffs) {
      typeCounts[diff.diffType] = (typeCounts[diff.diffType] || 0) + 1
    }
    groups.push({ tableName, diffs, typeCounts })
  }

  return groups
})

// Watch connection changes
watch(sourceId, () => {
  sourceDatabases.value = []
  sourceDb.value = ''
  sourceTables.value = []
})

watch(targetId, () => {
  targetDatabases.value = []
  targetDb.value = ''
  targetTables.value = []
})

async function onSourceChange() {
  if (!sourceId.value) return
  loadingSourceDb.value = true
  try {
    const conn = connections.value.find(c => c.id === sourceId.value)
    if (conn) {
      await getTauriAPI().dbConnect(JSON.parse(JSON.stringify(conn)))
    }
    const res = await getTauriAPI().dbGetDatabases(sourceId.value)
    if (res?.success) {
      sourceDatabases.value = (res as any).databases || []
    }
  } catch {
    // silently fail
  } finally {
    loadingSourceDb.value = false
  }
}

async function onTargetChange() {
  if (!targetId.value) return
  loadingTargetDb.value = true
  try {
    const conn = connections.value.find(c => c.id === targetId.value)
    if (conn) {
      await getTauriAPI().dbConnect(JSON.parse(JSON.stringify(conn)))
    }
    const res = await getTauriAPI().dbGetDatabases(targetId.value)
    if (res?.success) {
      targetDatabases.value = (res as any).databases || []
    }
  } catch {
    // silently fail
  } finally {
    loadingTargetDb.value = false
  }
}

async function loadSourceTables() {
  if (!sourceId.value || !sourceDb.value) return
  try {
    const res = await getTauriAPI().dbGetTables(sourceId.value, sourceDb.value)
    if (res?.success) {
      sourceTables.value = (res as any).tables || []
    }
  } catch {
    // silently fail
  }
}

async function loadTargetTables() {
  if (!targetId.value || !targetDb.value) return
  try {
    const res = await getTauriAPI().dbGetTables(targetId.value, targetDb.value)
    if (res?.success) {
      targetTables.value = (res as any).tables || []
    }
  } catch {
    // silently fail
  }
}

async function goToStep2() {
  step.value = 2
  tableColumns.value = {}
  tablePrimaryKeys.value = {}
  compareKeys.value = {}
  if (commonTablesList.value.length > 0) {
    selectedTables.value = [...commonTablesList.value]
    // Auto-fetch columns + primary keys for all common tables
    await autoFetchTableMeta(commonTablesList.value)
  } else {
    selectedTables.value = []
  }
}

async function autoFetchTableMeta(tables: string[]) {
  for (const table of tables) {
    if (tableColumns.value[table]) continue // already fetched
    try {
      // Fetch table structure for columns
      const structRes = await getTauriAPI().dbGetTableStructure(targetId.value, table, targetDb.value)
      if (structRes?.success && structRes.structure?.columns) {
        tableColumns.value[table] = structRes.structure.columns.map((c: any) => c.name)
      }
      // Fetch primary keys
      const pkRes = await getTauriAPI().dbGetTablePrimaryKeys(targetId.value, table, targetDb.value)
      if (pkRes?.success && pkRes.primaryKeys && pkRes.primaryKeys.length > 0) {
        tablePrimaryKeys.value[table] = pkRes.primaryKeys
        compareKeys.value[table] = [...pkRes.primaryKeys] // default to PKs
      }
    } catch {
      // fallback: user will need to manually configure
    }
  }
}

function selectAllTables() {
  selectedTables.value = [...commonTablesList.value]
  autoFetchTableMeta(commonTablesList.value)
}

function selectCommonTables() {
  selectedTables.value = [...commonTablesList.value]
  autoFetchTableMeta(commonTablesList.value)
}

function selectNone() {
  selectedTables.value = []
  tableColumns.value = {}
  tablePrimaryKeys.value = {}
  compareKeys.value = {}
  openDropdown.value = null
}

// Dropdown helpers for compare key selection
function toggleDropdown(table: string) {
  openDropdown.value = openDropdown.value === table ? null : table
}

function toggleCompareKey(table: string, col: string) {
  if (!compareKeys.value[table]) compareKeys.value[table] = []
  const idx = compareKeys.value[table].indexOf(col)
  if (idx >= 0) {
    compareKeys.value[table].splice(idx, 1)
  } else {
    compareKeys.value[table].push(col)
  }
}

function removeCompareKey(table: string, key: string) {
  if (!compareKeys.value[table]) return
  compareKeys.value[table] = compareKeys.value[table].filter(k => k !== key)
}

// Close dropdown when clicking outside
function closeDropdown(event: MouseEvent) {
  const target = event.target as HTMLElement
  if (!target.closest('.pk-select-wrapper')) {
    openDropdown.value = null
  }
}
if (typeof document !== 'undefined') {
  document.addEventListener('click', closeDropdown)
}

async function startCompare() {
  // Check for missing compare keys before starting
  const missingKeyTables = selectedTables.value.filter(t => !compareKeys.value[t] || compareKeys.value[t].length === 0)
  if (missingKeyTables.length > 0) {
    toast.error(`以下表未配置对比字段，无法对比数据：${missingKeyTables.join(', ')}`)
    return
  }

  comparing.value = true
  result.value = null
  execResult.value = null
  filterTypes.value = new Set(['insert', 'update', 'delete'])
  filterTables.value = []
  compareProgress.value = 0
  expandedTableGroups.value = new Set()
  expandedSqlRows.value = new Set()

  const allDiffs: DataDiffItem[] = []
  let totalInserts = 0
  let totalUpdates = 0
  let totalDeletes = 0

  try {
    for (const table of selectedTables.value) {
      const pks = compareKeys.value[table]
      if (!pks || pks.length === 0) continue

      // Get table structure to determine columns
      const structRes = await getTauriAPI().dbGetTableStructure(sourceId.value, table, sourceDb.value)
      let columns: string[] = []
      if (structRes?.success && structRes.structure?.columns) {
        columns = structRes.structure.columns.map((c: any) => c.name)
      }

      const res = await getTauriAPI().dbCompareData({
        sourceId: sourceId.value,
        targetId: targetId.value,
        table,
        primaryKeys: JSON.parse(JSON.stringify(pks)),
        columns: JSON.parse(JSON.stringify(columns)),
        sourceDb: sourceDb.value,
        targetDb: targetDb.value,
        tablePrimaryKeys: JSON.parse(JSON.stringify(tablePrimaryKeys.value[table] || []))
      })
      compareProgress.value++

      if (res?.success && res.result?.diffs) {
        const tableDiffs = res.result.diffs as any[]
        // Add tableName to each diff
        for (const diff of tableDiffs) {
          diff.tableName = table
        }
        allDiffs.push(...tableDiffs)
        totalInserts += res.result.totalInserts || 0
        totalUpdates += res.result.totalUpdates || 0
        totalDeletes += res.result.totalDeletes || 0
      }
    }

    result.value = {
      diffs: allDiffs,
      totalInserts,
      totalUpdates,
      totalDeletes,
    }
    // Auto-expand all table groups on first load
    const tables = [...new Set(allDiffs.map(d => d.tableName))]
    expandedTableGroups.value = new Set(tables)
  } catch (e: any) {
    toast.error('对比失败: ' + (e?.message || '未知错误'))
  } finally {
    comparing.value = false
  }
}

function toggleFilter(type: string) {
  if (filterTypes.value.has(type)) {
    filterTypes.value.delete(type)
  } else {
    filterTypes.value.add(type)
  }
}

async function executeSync() {
  executing.value = true
  try {
    // Execute sync per table (since the backend expects a single table)
    const tablesToSync = [...new Set(filteredDiffs.value.map(d => d.tableName))]
    let totalInserted = 0
    let totalUpdated = 0
    let totalDeleted = 0
    const allErrors: string[] = []

    for (const table of tablesToSync) {
      const pks = compareKeys.value[table]
      if (!pks || pks.length === 0) {
        allErrors.push(`No compare keys configured for table '${table}'`)
        continue
      }

      const structRes = await getTauriAPI().dbGetTableStructure(sourceId.value, table, sourceDb.value)
      let columns: string[] = []
      if (structRes?.success && structRes.structure?.columns) {
        columns = structRes.structure.columns.map((c: any) => c.name)
      }

      const tableDiffs = filteredDiffs.value.filter(d => d.tableName === table)

      const syncPayload = JSON.parse(JSON.stringify({
        sourceConnectionId: sourceId.value,
        targetConnectionId: targetId.value,
        tableName: table,
        primaryKeys: pks,
        tablePrimaryKeys: tablePrimaryKeys.value[table] || [],
        columns,
        diffs: tableDiffs,
        useTransaction: useTransaction.value,
        batchSize: 100,
        sourceDbName: sourceDb.value,
        targetDbName: targetDb.value,
      }))
      const res = await getTauriAPI().dbExecuteDataSync(syncPayload)
      if (res?.success) {
        totalInserted += res.inserted || 0
        totalUpdated += res.updated || 0
        totalDeleted += res.deleted || 0
      } else {
        const r = res as any
        if (r?.errors) allErrors.push(...r.errors)
        if (r?.error) allErrors.push(r.error)
        if (allErrors.length === 0) allErrors.push('同步失败')
      }
    }

    execResult.value = {
      success: allErrors.length === 0,
      inserted: totalInserted,
      updated: totalUpdated,
      deleted: totalDeleted,
      errors: allErrors,
      duration: 0,
    }
  } catch (e: any) {
    execResult.value = {
      success: false,
      inserted: 0,
      updated: 0,
      deleted: 0,
      errors: [e?.message || '执行失败'],
      duration: 0,
    }
  } finally {
    executing.value = false
  }
}

function reset() {
  result.value = null
  execResult.value = null
  filterTypes.value = new Set(['insert', 'update', 'delete'])
  filterTables.value = []
  expandedTableGroups.value = new Set()
  expandedSqlRows.value = new Set()
  comparing.value = false
  step.value = 2
}

// Generate SQL from diffs for preview
const generatedSqlList = computed(() => {
  const sqls: string[] = []
  for (const diff of filteredDiffs.value) {
    const table = escapeIdentifier(diff.tableName)
    if (diff.diffType === 'insert' && diff.sourceRow) {
      const cols = Object.keys(diff.sourceRow).map(escapeIdentifier).join(', ')
      const vals = Object.values(diff.sourceRow).map(formatSqlValue).join(', ')
      sqls.push(`INSERT INTO ${table} (${cols}) VALUES (${vals});`)
    } else if (diff.diffType === 'update' && diff.sourceRow && diff.primaryKey) {
      const sets = Object.entries(diff.sourceRow)
        .map(([k, v]) => `${escapeIdentifier(k)} = ${formatSqlValue(v)}`)
        .join(', ')
      const where = Object.entries(diff.primaryKey)
        .map(([k, v]) => `${escapeIdentifier(k)} = ${formatSqlValue(v)}`)
        .join(' AND ')
      sqls.push(`UPDATE ${table} SET ${sets} WHERE ${where};`)
    } else if (diff.diffType === 'delete' && diff.primaryKey) {
      const where = Object.entries(diff.primaryKey)
        .map(([k, v]) => `${escapeIdentifier(k)} = ${formatSqlValue(v)}`)
        .join(' AND ')
      sqls.push(`DELETE FROM ${table} WHERE ${where};`)
    }
  }
  return sqls
})

function escapeIdentifier(name: string): string {
  return name.includes('-') || name.includes(' ') || /^[0-9]/.test(name) ? `"${name}"` : name
}

function formatSqlValue(val: unknown): string {
  if (val === null || val === undefined) return 'NULL'
  if (typeof val === 'number') return String(val)
  if (typeof val === 'boolean') return val ? '1' : '0'
  return "'" + String(val).replace(/'/g, "''") + "'"
}

async function copyAllSql() {
  const text = generatedSqlList.value.join('\n\n')
  await navigator.clipboard.writeText(text)
  toast.success('已复制全部 SQL')
}

async function copySingleSql(idx: number) {
    console.log("[copySingleSql] called")
  toast.success('已复制')
}

function getDiffTypeLabel(type: string): string {
  const labels: Record<string, string> = {
    insert: '新增',
    update: '更新',
    delete: '删除',
  }
  return labels[type] || type
}

function getDiffTypeBadgeClass(type: string): string {
  const classes: Record<string, string> = {
    insert: 'diff-green',
    update: 'diff-orange',
    delete: 'diff-red',
  }
  return classes[type] || ''
}

function formatPrimaryKey(pk: Record<string, any>): string {
  return Object.entries(pk)
    .map(([k, v]) => `${k}=${v}`)
    .join(', ')
}

function formatRow(row: Record<string, any>): string {
  return JSON.stringify(row, null, 2)
}

// Navicat-style: compact row preview showing only non-PK columns
function getRowPreview(row: Record<string, any>): string {
  const entries = Object.entries(row)
  if (entries.length === 0) return '—'
  // Show first 3 key-value pairs as preview
  const shown = entries.slice(0, 3).map(([k, v]) => `${k}=${formatCellValue(v)}`)
  const preview = shown.join(', ')
  if (entries.length > 3) return preview + ` (+${entries.length - 3} more)`
  return preview
}

function formatCellValue(val: any): string {
  if (val == null) return 'NULL'
  if (typeof val === 'string') return val.length > 50 ? val.slice(0, 50) + '…' : val
  return String(val)
}

// Get columns that differ between source and target rows
function getChangedColumns(sourceRow: Record<string, any>, targetRow: Record<string, any>): { name: string; sourceVal: any; targetVal: any }[] {
  const changed: { name: string; sourceVal: any; targetVal: any }[] = []
  const allKeys = new Set([...Object.keys(sourceRow), ...Object.keys(targetRow)])
  for (const key of allKeys) {
    const sVal = sourceRow[key]
    const tVal = targetRow[key]
    if (JSON.stringify(sVal) !== JSON.stringify(tVal)) {
      changed.push({ name: key, sourceVal: sVal, targetVal: tVal })
    }
  }
  return changed
}

// Expand/collapse table groups
function isTableExpanded(tableName: string): boolean {
  return expandedTableGroups.value.has(tableName)
}

function toggleTableExpand(tableName: string) {
  const next = new Set(expandedTableGroups.value)
  if (next.has(tableName)) {
    next.delete(tableName)
  } else {
    next.add(tableName)
  }
  expandedTableGroups.value = next
}

// Expand/collapse detail rows
function isSqlRowExpanded(key: string): boolean {
  return expandedSqlRows.value.has(key)
}

function toggleSqlRow(key: string) {
  const next = new Set(expandedSqlRows.value)
  if (next.has(key)) {
    next.delete(key)
  } else {
    next.add(key)
  }
  expandedSqlRows.value = next
}
</script>

<style scoped>
.data-sync {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 16px;
  overflow: auto;
}

.sync-header {
  margin-bottom: 16px;
}

.sync-title {
  font-size: 16px;
  font-weight: 600;
  margin: 0 0 4px 0;
}

.sync-desc {
  font-size: 13px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  margin: 0;
}

.sync-config {
  background: var(--color-base-100);
  border-radius: 8px;
  padding: 16px;
}

.config-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 12px;
  margin-bottom: 16px;
}

.config-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.config-item label {
  font-size: 12px;
  font-weight: 500;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.select-input,
.text-input {
  padding: 8px 12px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 6px;
  background: var(--color-base-200);
  color: var(--color-base-content);
  font-size: 13px;
}

.checkbox-item {
  flex-direction: row;
  align-items: center;
  padding-top: 20px;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  cursor: pointer;
}

.sync-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

/* Step 2: Table Selection */
.table-select-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.table-select-title {
  font-size: 14px;
  font-weight: 600;
  margin: 0;
}

.table-select-controls {
  display: flex;
  gap: 6px;
}

.table-grid {
  max-height: 300px;
  overflow-y: auto;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 8px;
  padding: 8px;
  background: var(--color-base-200);
  margin-bottom: 12px;
}

.table-section-label {
  font-size: 11px;
  font-weight: 600;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  padding: 6px 8px 4px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  margin-bottom: 4px;
  margin-top: 8px;
}

.table-section-label:first-child {
  margin-top: 0;
}

.table-checkbox label {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  transition: background 0.1s;
}

.table-checkbox label:hover {
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
}

.table-checkbox.disabled-table label {
  cursor: default;
  opacity: 0.6;
}

.table-checkbox.disabled-table label:hover {
  background: transparent;
}

.table-name {
  flex: 1;
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
}

.table-badge {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 3px;
  font-weight: 500;
}

.badge-source {
  background: #e3f2fd;
  color: #1565c0;
}

.badge-target {
  background: #f3e5f5;
  color: #7b1fa2;
}

.badge-common {
  background: #e8f5e9;
  color: #2e7d32;
}

.table-empty {
  text-align: center;
  padding: 24px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-style: italic;
}

/* Compare Key Config */
.pk-config {
  background: var(--color-base-200);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 8px;
  padding: 12px;
  margin-bottom: 12px;
}

.pk-title {
  font-size: 13px;
  font-weight: 600;
  margin: 0 0 8px 0;
}

.pk-hint {
  font-weight: 400;
  font-size: 11px;
  color: var(--secondary-text, #888);
}

.pk-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 0;
}

.pk-table-name {
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  min-width: 120px;
  font-weight: 500;
}

/* Multi-select dropdown */
.pk-select-wrapper {
  flex: 1;
  position: relative;
}

.pk-select {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 4px;
  padding: 4px 8px;
  min-height: 28px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 4px;
  background: var(--color-base-100);
  cursor: pointer;
  transition: border-color 0.2s;
}

.pk-select:hover {
  border-color: var(--color-primary);
}

.pk-select.open {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px rgba(100, 108, 255, 0.15);
}

.pk-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  flex: 1;
}

.pk-tag {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  padding: 1px 6px;
  font-size: 11px;
  font-family: 'JetBrains Mono', monospace;
  border-radius: 3px;
  background: rgba(100, 108, 255, 0.12);
  color: var(--color-primary);
  border: 1px solid rgba(100, 108, 255, 0.25);
}

.pk-tag.is-pk {
  background: rgba(76, 175, 80, 0.12);
  color: var(--color-success);
  border-color: rgba(76, 175, 80, 0.25);
}

.pk-tag-remove {
  cursor: pointer;
  opacity: 0.6;
  font-size: 13px;
  line-height: 1;
}

.pk-tag-remove:hover {
  opacity: 1;
}

.pk-placeholder {
  font-size: 12px;
  color: var(--secondary-text, #888);
}

.pk-arrow {
  font-size: 10px;
  margin-left: auto;
  color: var(--secondary-text, #888);
  transition: transform 0.2s;
}

.pk-select.open .pk-arrow {
  transform: rotate(180deg);
}

.pk-dropdown {
  position: absolute;
  top: calc(100% + 2px);
  left: 0;
  right: 0;
  max-height: 240px;
  overflow-y: auto;
  background: var(--color-base-100);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 6px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
  z-index: 1000;
  padding: 4px 0;
}

.pk-option {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 12px;
  cursor: pointer;
  transition: background 0.1s;
}

.pk-option:hover {
  background: rgba(100, 108, 255, 0.08);
}

.pk-option.selected {
  background: rgba(100, 108, 255, 0.05);
}

.pk-checkbox {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border: 1.5px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 3px;
  font-size: 10px;
  color: transparent;
  transition: all 0.15s;
  flex-shrink: 0;
}

.pk-checkbox.checked {
  background: var(--color-primary);
  border-color: var(--color-primary);
  color: white;
}

.pk-option-name {
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  flex: 1;
}

.pk-badge {
  font-size: 9px;
  font-weight: 700;
  padding: 1px 5px;
  border-radius: 3px;
  background: rgba(76, 175, 80, 0.15);
  color: var(--color-success);
  letter-spacing: 0.5px;
}

.pk-status {
  font-size: 11px;
  min-width: 70px;
  text-align: right;
}

.pk-status.auto {
  color: var(--color-success);
}

.pk-status.missing {
  color: var(--color-warning);
}

.pk-status.loading {
  color: var(--secondary-text, #888);
}

.table-select-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.selected-count-text {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-primary);
}

.footer-btns {
  display: flex;
  gap: 8px;
}

/* Loading */
.sync-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px;
  gap: 12px;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Results */
.sync-results {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.results-summary {
  background: var(--color-base-100);
  border-radius: 8px;
  padding: 12px 16px;
}

.results-summary h4 {
  margin: 0 0 8px 0;
  font-size: 14px;
}

.summary-stats {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.stat {
  padding: 4px 10px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
}

.stat-insert {
  background: #e8f5e9;
  color: #2e7d32;
}

.stat-update {
  background: #fff3e0;
  color: #e65100;
}

.stat-delete {
  background: #ffebee;
  color: #c62828;
}

.stat-total {
  background: #e3f2fd;
  color: #1565c0;
}

.diff-filter {
  display: flex;
  gap: 16px;
  padding: 8px 0;
}

.filter-label {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  font-size: 13px;
}

.filter-badge {
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
}

.filter-badge.insert {
  background: #e8f5e9;
  color: #2e7d32;
}

.filter-badge.update {
  background: #fff3e0;
  color: #e65100;
}

.filter-badge.delete {
  background: #ffebee;
  color: #c62828;
}

/* Filter by table */
.results-filter {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
  padding: 8px 0;
}

.filter-label-text {
  font-size: 12px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-weight: 500;
}

.filter-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 12px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  font-size: 11px;
  cursor: pointer;
  font-family: 'JetBrains Mono', monospace;
}

.filter-chip:hover {
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
}

/* ===== Navicat-style Grouped Diff View ===== */
.diff-grouped-view {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 500px;
  overflow-y: auto;
}

.diff-table-group {
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 8px;
  overflow: hidden;
}

.diff-group-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  background: var(--color-base-100);
  cursor: pointer;
  user-select: none;
  transition: background 0.1s;
}

.diff-group-header:hover {
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
}

.expand-icon {
  font-size: 10px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  width: 12px;
  text-align: center;
}

.group-table-name {
  font-family: 'JetBrains Mono', monospace;
  font-size: 13px;
  font-weight: 600;
}

.group-diff-count {
  font-size: 11px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.group-type-badges {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
  margin-left: auto;
}

.mini-badge {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 3px;
  font-weight: 500;
}

.diff-group-body {
  border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

/* Compact comparison table */
.diff-compare-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
}

.diff-compare-table th {
  text-align: left;
  padding: 6px 10px;
  font-size: 11px;
  font-weight: 600;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  background: var(--color-base-200);
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  position: sticky;
  top: 0;
}

.diff-compare-table td {
  padding: 6px 10px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  vertical-align: middle;
}

.diff-compare-table tr:last-child td {
  border-bottom: none;
}

/* Color-coded rows by diff type */
.diff-row {
  transition: background 0.1s;
}

.diff-row:hover {
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
}

.diff-row.insert { border-left: 3px solid #4caf50; }
.diff-row.update { border-left: 3px solid #ff9800; }
.diff-row.delete { border-left: 3px solid #f44336; }

/* Column widths */
.col-pk { width: 25%; }
.col-source { width: 30%; }
.col-target { width: 30%; }
.col-action { width: 70px; text-align: center; }

.pk-display {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  font-weight: 500;
}

.compact-row-preview {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  color: var(--color-base-content);
  word-break: break-all;
}

.null-value {
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-style: italic;
  font-size: 11px;
}

.sql-toggle-btn {
  font-size: 11px;
  padding: 2px 8px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 3px;
  background: var(--color-base-200);
  color: var(--color-primary);
  cursor: pointer;
  white-space: nowrap;
}

.sql-toggle-btn:hover {
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
}

.sql-detail-row td {
  padding: 0 !important;
  background: rgba(0, 0, 0, 0.03);
}

/* Data detail panel */
.data-detail-panel {
  padding: 12px 14px;
}

.detail-title {
  font-size: 12px;
  font-weight: 600;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  margin-bottom: 8px;
}

/* Column-by-column diff table for updates */
.column-diff-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 11px;
}

.column-diff-table th {
  text-align: left;
  padding: 4px 8px;
  font-size: 10px;
  font-weight: 600;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  background: var(--color-base-200);
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.column-diff-table td {
  padding: 4px 8px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  font-family: 'JetBrains Mono', monospace;
}

.changed-col-row {
  background: rgba(255, 152, 0, 0.05);
}

.changed-col-row:hover {
  background: rgba(255, 152, 0, 0.1);
}

.col-name {
  font-weight: 600;
  color: var(--color-base-content);
}

.col-old {
  color: #c62828;
  background: rgba(244, 67, 54, 0.05);
}

.col-new {
  color: #2e7d32;
  background: rgba(76, 175, 80, 0.05);
}

.no-changes {
  text-align: center;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-style: italic;
  padding: 8px;
}

.row-json {
  margin: 0;
  padding: 8px 10px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  line-height: 1.5;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
  background: var(--color-base-200);
  border-radius: 4px;
  color: var(--color-base-content);
}

.diff-empty {
  text-align: center;
  padding: 32px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-size: 14px;
}

.execute-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 0;
  border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.selected-count {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-primary);
}

.execute-btns {
  display: flex;
  gap: 8px;
}

.exec-result {
  background: var(--color-base-100);
  border-radius: 8px;
  padding: 16px;
  text-align: center;
}

.exec-result.success {
  border: 1px solid #4caf50;
}

.exec-result.error {
  border: 1px solid #f44336;
}

.exec-result h4 {
  margin: 0 0 8px 0;
}

.exec-stats {
  display: flex;
  gap: 16px;
  justify-content: center;
  font-size: 13px;
  margin-bottom: 12px;
}

.exec-errors {
  text-align: left;
  margin: 12px 0;
  padding: 12px;
  background: #ffebee;
  border-radius: 4px;
}

.exec-errors ul {
  margin: 8px 0 0 0;
  padding-left: 20px;
}

.exec-errors li {
  font-size: 12px;
  color: #c62828;
  margin-bottom: 4px;
}

.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-primary {
  background: var(--color-primary);
  color: white;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-ghost {
  background: transparent;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.btn-sm {
  padding: 4px 10px;
  font-size: 12px;
}

.btn-xs {
  padding: 3px 8px;
  font-size: 11px;
  border-radius: 4px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  background: transparent;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  cursor: pointer;
}

/* ==================== SQL Dialog ==================== */
.sql-dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}

.sql-dialog {
  background: var(--color-base-100);
  border-radius: 12px;
  width: 720px;
  max-width: 90vw;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
}

.sql-dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.sql-dialog-header h3 {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
}

.sql-dialog-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.sql-dialog-close {
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-size: 18px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.sql-dialog-close:hover {
  background: var(--color-base-200);
  color: var(--color-base-content);
}

.sql-dialog-body {
  padding: 16px 20px;
  overflow-y: auto;
  flex: 1;
}

.sql-item {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 10px 12px;
  background: var(--color-base-200);
  border-radius: 8px;
  margin-bottom: 8px;
  font-size: 12px;
}

.sql-number {
  flex-shrink: 0;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: var(--color-primary);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 600;
  margin-top: 2px;
}

.sql-text {
  flex: 1;
  margin: 0;
  padding: 0;
  white-space: pre-wrap;
  word-break: break-all;
  font-family: monospace;
  font-size: 12px;
  color: var(--color-base-content);
  background: transparent;
}

.sql-copy-btn {
  flex-shrink: 0;
  padding: 4px 8px;
  border: none;
  border-radius: 4px;
  background: transparent;
  cursor: pointer;
  font-size: 12px;
}

.sql-copy-btn:hover {
  background: color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

</style>
