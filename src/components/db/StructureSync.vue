<template>
  <div class="structure-sync">
    <div class="sync-header">
      <h3 class="sync-title">🔧 结构同步</h3>
      <p class="sync-desc">对比并同步两个数据库之间的表结构（Navicat 风格多表对比）</p>
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
        <h4 class="table-select-title">📋 选择要对比的表</h4>
        <div class="table-select-controls">
          <button @click="selectAllTables" class="btn btn-ghost btn-xs">全选</button>
          <button @click="selectCommonTables" class="btn btn-ghost btn-xs">仅共有表</button>
          <button @click="selectNone" class="btn btn-ghost btn-xs">清空</button>
        </div>
      </div>

      <div class="table-grid">
        <!-- Source-only tables -->
        <template v-if="sourceOnlyTables.length > 0">
          <div class="table-section-label">仅源端有</div>
          <div v-for="table in sourceOnlyTables" :key="'src-' + table" class="table-checkbox" :class="{ 'source-only': true }">
            <label>
              <input type="checkbox" v-model="selectedTables" :value="table" />
              <span class="table-name">{{ table }}</span>
              <span class="table-badge badge-source">仅源</span>
            </label>
          </div>
        </template>

        <!-- Target-only tables -->
        <template v-if="targetOnlyTables.length > 0">
          <div class="table-section-label">仅目标端有</div>
          <div v-for="table in targetOnlyTables" :key="'tgt-' + table" class="table-checkbox" :class="{ 'target-only': true }">
            <label>
              <input type="checkbox" v-model="selectedTables" :value="table" />
              <span class="table-name">{{ table }}</span>
              <span class="table-badge badge-target">仅目标</span>
            </label>
          </div>
        </template>

        <!-- Common tables -->
        <template v-if="commonTablesList.length > 0">
          <div class="table-section-label">共有表（{{ commonTablesList.length }}）</div>
          <div v-for="table in commonTablesList" :key="'common-' + table" class="table-checkbox" :class="{ 'common': true }">
            <label>
              <input type="checkbox" v-model="selectedTables" :value="table" />
              <span class="table-name">{{ table }}</span>
              <span class="table-badge badge-common">共有</span>
            </label>
          </div>
        </template>

        <div v-if="sourceTables.length === 0 && targetTables.length === 0" class="table-empty">
          两个数据库都没有表
        </div>
      </div>

      <div class="table-select-footer">
        <span class="selected-count-text">已选 {{ selectedTables.length }} 张表</span>
        <div class="footer-btns">
          <button @click="step = 1" class="btn btn-ghost">← 返回</button>
          <button @click="startCompare" :disabled="selectedTables.length === 0 || comparing" class="btn btn-primary">
            {{ comparing ? '对比中...' : `🔍 对比 ${selectedTables.length} 张表` }}
          </button>
        </div>
      </div>
    </div>

    <!-- Comparing State -->
    <div v-if="comparing" class="sync-loading">
      <div class="loading-spinner"></div>
      <p>正在对比表结构... ({{ compareProgress }}/{{ selectedTables.length }})</p>
    </div>

    <!-- Results -->
    <div v-if="result && !comparing" class="sync-results">
      <div class="results-summary">
        <h4>对比结果</h4>
        <div class="summary-stats">
          <span class="stat stat-diffs">差异项: {{ result.diffs.length }}</span>
          <span class="stat stat-source">涉及表: {{ affectedTablesList.length }}</span>
        </div>
        <!-- Diff type breakdown -->
        <div class="diff-type-summary">
          <span v-for="(count, type) in diffTypeCounts" :key="type" class="diff-type-stat" :class="getDiffTypeClass(type)">
            {{ getDiffTypeLabel(type) }}: {{ count }}
          </span>
        </div>
      </div>

      <!-- Filter by table -->
      <div class="results-filter" v-if="affectedTablesList.length > 1">
        <span class="filter-label-text">按表筛选:</span>
        <label v-for="table in affectedTablesList" :key="table" class="filter-chip">
          <input type="checkbox" v-model="filterTables" :value="table" />
          {{ table }}
        </label>
      </div>

      <!-- Navicat-style Grouped Diff Table -->
      <div class="diff-grouped-view">
        <div v-for="(group, tIdx) in groupedDiffs" :key="tIdx" class="diff-table-group">
          <!-- Table group header -->
          <div class="diff-group-header" @click="toggleTableExpand(group.tableName)">
            <span class="expand-icon">{{ isTableExpanded(group.tableName) ? '▼' : '▶' }}</span>
            <span class="group-table-name">{{ group.tableName }}</span>
            <span class="group-diff-count">{{ group.diffs.length }} 项差异</span>
            <span class="group-type-badges">
              <span v-for="(count, type) in group.typeCounts" :key="type" class="mini-badge" :class="getDiffTypeClass(type)">
                {{ getDiffTypeLabel(type) }} ×{{ count }}
              </span>
            </span>
          </div>

          <!-- Expanded diff rows -->
          <div v-if="isTableExpanded(group.tableName)" class="diff-group-body">
            <table class="diff-compare-table">
              <thead>
                <tr>
                  <th class="col-select"></th>
                  <th class="col-type">差异类型</th>
                  <th class="col-source">源</th>
                  <th class="col-target">目标</th>
                  <th class="col-sql">SQL</th>
                </tr>
              </thead>
              <tbody>
                <template v-for="(diff, dIdx) in group.diffs" :key="dIdx">
                  <tr class="diff-row" :class="diff.diffType">
                    <td class="col-select">
                      <label class="row-checkbox">
                        <input type="checkbox" :checked="selectedSqls.has(diff.sql)" @change="toggleSql(diff.sql)" />
                      </label>
                    </td>
                    <td class="col-type">
                      <span class="type-tag" :class="getDiffTypeClass(diff.diffType)">
                        {{ getDiffTypeLabel(diff.diffType) }}
                      </span>
                    </td>
                    <td class="col-source">
                      <span class="compact-value">{{ getCompactValue(diff.sourceValue) }}</span>
                    </td>
                    <td class="col-target">
                      <span class="compact-value">{{ getCompactValue(diff.targetValue) }}</span>
                    </td>
                    <td class="col-sql">
                      <button class="sql-toggle-btn" @click="toggleSqlRow(tIdx + '-' + dIdx)">
                        {{ isSqlRowExpanded(tIdx + '-' + dIdx) ? '隐藏' : '查看' }} SQL
                      </button>
                    </td>
                  </tr>
                  <!-- Expandable SQL row -->
                  <tr v-if="isSqlRowExpanded(tIdx + '-' + dIdx)" class="sql-detail-row">
                    <td colspan="5">
                      <pre class="sql-code">{{ diff.sql }}</pre>
                    </td>
                  </tr>
                </template>
              </tbody>
            </table>
          </div>
        </div>

        <div v-if="filteredDiffs.length === 0" class="diff-empty">
          所选表结构完全一致 ✅
        </div>
      </div>

      <!-- Execute Actions -->
      <div class="execute-actions">
        <span class="selected-count">已选择 {{ selectedSqls.size }} 项更改</span>
        <div class="execute-btns">
          <button @click="selectAll" class="btn btn-ghost">全选</button>
          <button @click="reset" class="btn btn-ghost">重新对比</button>
          <button @click="showSqlDialog = true" :disabled="selectedSqls.size === 0" class="btn btn-ghost">📄 查看SQL</button>
          <button @click="executeSync" :disabled="selectedSqls.size === 0 || executing" class="btn btn-primary">
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
            <h3>📄 待执行 SQL ({{ selectedSqls.size }} 条)</h3>
            <div class="sql-dialog-actions">
              <button @click="copyAllSql" class="btn btn-ghost btn-sm">📋 复制全部</button>
              <button @click="showSqlDialog = false" class="sql-dialog-close">×</button>
            </div>
          </div>
          <div class="sql-dialog-body">
            <div v-for="(sql, idx) in selectedSqlArray" :key="idx" class="sql-item">
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
      <p>已执行 {{ execResult.executed }} 项更改</p>
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
import { ref, computed, watch } from 'vue'
import { getTauriAPI } from '../../utils/tauri-api'
import { useDBManager } from '../../composables/useDBManager'
import { useToast } from '../../composables/useToast'

const db = useDBManager()
const toast = useToast()
const connections = computed(() => db.connections.value)

const step = ref(1)
const sourceId = ref('')
const targetId = ref('')
const sourceDb = ref('')
const targetDb = ref('')
const tableName = ref('')
const comparing = ref(false)
const compareProgress = ref(0)
const executing = ref(false)

// Database lists
const sourceDatabases = ref<string[]>([])
const targetDatabases = ref<string[]>([])
const loadingSourceDb = ref(false)
const loadingTargetDb = ref(false)

// Table lists
const sourceTables = ref<string[]>([])
const targetTables = ref<string[]>([])

// Selected tables for comparison
const selectedTables = ref<string[]>([])

interface StructureDiffItem {
  tableName: string
  diffType: string
  sourceValue?: any
  targetValue?: any
  sql: string
}

interface StructureSyncResult {
  diffs: StructureDiffItem[]
  sourceTables: string[]
  targetTables: string[]
  commonTables: string[]
}

// Computed: affected tables from diffs
const affectedTablesList = computed(() => {
  if (!result.value) return []
  return [...new Set(result.value.diffs.map(d => d.tableName))]
})

const result = ref<StructureSyncResult | null>(null)
const selectedSqls = ref<Set<string>>(new Set())
const showSqlDialog = ref(false)
const filterTables = ref<string[]>([])
const execResult = ref<{ success: boolean; executed: number; errors: string[] } | null>(null)

// Navicat-style: track expanded table groups and SQL rows
const expandedTableGroups = ref<Set<string>>(new Set())
const expandedSqlRows = ref<Set<string>>(new Set())

// Computed table lists — case-insensitive matching
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

const selectedSqlArray = computed(() => Array.from(selectedSqls.value))

const filteredDiffs = computed(() => {
  if (!result.value) return []
  if (filterTables.value.length === 0) return result.value.diffs
  return result.value.diffs.filter(d => filterTables.value.includes(d.tableName))
})

const diffTypeCounts = computed(() => {
  const counts: Record<string, number> = {}
  if (!result.value) return counts
  for (const diff of result.value.diffs) {
    counts[diff.diffType] = (counts[diff.diffType] || 0) + 1
  }
  return counts
})

// Navicat-style: group diffs by table
const groupedDiffs = computed(() => {
  const groups: { tableName: string; diffs: StructureDiffItem[]; typeCounts: Record<string, number> }[] = []
  const tableMap = new Map<string, StructureDiffItem[]>()

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
    console.log("[onSourceChange] called")
    const conn = connections.value.find(c => c.id === sourceId.value)
    if (conn) {
      await getTauriAPI().dbConnect(JSON.parse(JSON.stringify(conn)))
    }
    const res = await getTauriAPI().dbGetDatabases(sourceId.value)
    if (res?.success) {
      sourceDatabases.value = res.databases || []
    } else if (res?.error) {
      toast.error('获取源数据库列表失败: ' + res.error)
    }
  } catch (e: unknown) {
    toast.error('获取源数据库列表失败: ' + (e instanceof Error ? e.message : String(e)))
  } finally {
    loadingSourceDb.value = false
  }
}

async function onTargetChange() {
  if (!targetId.value) return
  loadingTargetDb.value = true
  try {
    console.log("[onTargetChange] called")
    const conn = connections.value.find(c => c.id === targetId.value)
    if (conn) {
      await getTauriAPI().dbConnect(JSON.parse(JSON.stringify(conn)))
    }
    const res = await getTauriAPI().dbGetDatabases(targetId.value)
    if (res?.success) {
      targetDatabases.value = res.databases || []
    } else if (res?.error) {
      toast.error('获取目标数据库列表失败: ' + res.error)
    }
  } catch (e: unknown) {
    toast.error('获取目标数据库列表失败: ' + (e instanceof Error ? e.message : String(e)))
  } finally {
    loadingTargetDb.value = false
  }
}

async function loadSourceTables() {
  if (!sourceId.value || !sourceDb.value) return
  try {
    console.log("[loadSourceTables] called")
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
    console.log("[loadTargetTables] called")
    const res = await getTauriAPI().dbGetTables(targetId.value, targetDb.value)
    if (res?.success) {
      targetTables.value = (res as any).tables || []
    }
  } catch {
    // silently fail
  }
}

function goToStep2() {
  step.value = 2
  if (commonTablesList.value.length > 0) {
    selectedTables.value = [...commonTablesList.value]
  } else {
    selectedTables.value = [...new Set([...sourceTables.value, ...targetTables.value])]
  }
}

function selectAllTables() {
  const all = [...new Set([...sourceTables.value, ...targetTables.value])]
  selectedTables.value = all
}

function selectCommonTables() {
    console.log("[selectCommonTables] called")
  selectedTables.value = [...commonTablesList.value]
}

function selectNone() {
    console.log("[selectNone] called")
  selectedTables.value = []
}

async function startCompare() {
  comparing.value = true
  result.value = null
  execResult.value = null
  selectedSqls.value = new Set()
  filterTables.value = []
  compareProgress.value = 0
  expandedTableGroups.value = new Set()
  expandedSqlRows.value = new Set()

  // Listen for progress events from backend
  let unlisten: (() => void) | null = null
  try {
    const { listen } = await import('@tauri-apps/api/event')
    unlisten = await listen<{ current: number; total: number; table: string }>('db:compare-progress', (event) => {
      compareProgress.value = event.payload.current
    })
  } catch {}

  const allDiffs: StructureDiffItem[] = []

  try {
    console.log("[startCompare] called")
    const res = await getTauriAPI().dbCompareStructures(
      sourceId.value,
      sourceDb.value,
      targetId.value,
      targetDb.value
    )

    if (res?.success && res.diffs) {
      // Filter diffs to only include selected tables
      const selectedSet = new Set(selectedTables.value)
      const filtered = (res.diffs as StructureDiffItem[]).filter(d => selectedSet.has(d.tableName))
      allDiffs.push(...filtered)
    }

    compareProgress.value = selectedTables.value.length

    result.value = { diffs: allDiffs, sourceTables: [], targetTables: [], commonTables: [] }
    // Auto-expand all table groups on first load
    const tables = [...new Set(allDiffs.map(d => d.tableName))]
    expandedTableGroups.value = new Set(tables)
  } catch (e: any) {
    toast.error('对比失败: ' + (e?.message || '未知错误'))
  } finally {
    comparing.value = false
    if (unlisten) unlisten()
  }
}

function toggleSql(sql: string) {
  if (selectedSqls.value.has(sql)) {
    selectedSqls.value.delete(sql)
  } else {
    selectedSqls.value.add(sql)
  }
}

function selectAll() {
  if (result.value) {
    const allSqls = result.value.diffs
      .map(d => d.sql)
      .filter(sql => !sql.trim().startsWith('--'))
    selectedSqls.value = new Set(allSqls)
  }
}

async function executeSync() {
  executing.value = true
  try {
    console.log("[executeSync] called")
    const sqls = Array.from(selectedSqls.value)
    const res = await getTauriAPI().dbExecuteStructureSync(targetId.value, sqls, targetDb.value)
    if (res) {
      execResult.value = res
    }
  } catch (e: any) {
    execResult.value = { success: false, executed: 0, errors: [e?.message || '执行失败'] }
  } finally {
    executing.value = false
  }
}

function reset() {
  result.value = null
  execResult.value = null
  selectedSqls.value = new Set()
  filterTables.value = []
  expandedTableGroups.value = new Set()
  expandedSqlRows.value = new Set()
  comparing.value = false
  step.value = 2
}

async function copyAllSql() {
  const text = Array.from(selectedSqls.value).join('\n\n')
  await navigator.clipboard.writeText(text)
  toast.success('已复制全部 SQL')
}

async function copySingleSql(idx: number) {
  const sql = Array.from(selectedSqls.value)[idx]
  await navigator.clipboard.writeText(sql)
  toast.success('已复制')
}

function getDiffTypeLabel(type: string): string {
  const labels: Record<string, string> = {
    table_only_in_source: '源独有（需创建）',
    table_only_in_target: '目标独有',
    column_added: '新增列',
    column_removed: '删除列',
    column_modified: '修改列',
    index_added: '新增索引',
    index_removed: '删除索引',
    index_modified: '修改索引',
    primary_key_changed: '主键变更',
  }
  return labels[type] || type
}

function getDiffTypeClass(type: string): string {
  const classes: Record<string, string> = {
    table_only_in_source: 'diff-green',
    column_added: 'diff-blue',
    column_modified: 'diff-orange',
    index_added: 'diff-blue',
    index_modified: 'diff-orange',
    table_only_in_target: 'diff-red',
    column_removed: 'diff-red',
    index_removed: 'diff-red',
    primary_key_changed: 'diff-red',
  }
  return classes[type] || ''
}

// Navicat-style: compact value display (no full JSON dumps)
function getCompactValue(value: any): string {
  if (value == null) return '—'
  if (typeof value === 'string') return value
  if (typeof value === 'object') {
    // Column-like object: show name, type, nullable, default
    // Support both Electron format (name/type) and information_schema format (COLUMN_NAME/COLUMN_TYPE)
    const parts: string[] = []
    const colName = value.name || value.COLUMN_NAME
    const colType = value.type || value.COLUMN_TYPE
    const nullable = value.nullable !== undefined ? value.nullable : (value.IS_NULLABLE === 'YES')
    const defaultVal = value.default !== undefined ? value.default : value.COLUMN_DEFAULT
    const comment = value.comment || value.COLUMN_COMMENT
    if (colName) parts.push(colName)
    if (colType) parts.push(colType)
    if (!nullable) parts.push('NOT NULL')
    if (defaultVal != null) parts.push(`DEFAULT ${defaultVal}`)
    if (comment) parts.push(`COMMENT '${comment}'`)
    if (parts.length > 0) return parts.join(' ')
    // Fallback: short JSON
    const json = JSON.stringify(value)
    return json.length > 80 ? json.slice(0, 80) + '…' : json
  }
  return String(value)
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

// Expand/collapse SQL rows
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
.structure-sync {
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
  color: var(--main-text-secondary);
  margin: 0;
}

.sync-config {
  background: var(--card-bg);
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
  color: var(--main-text-secondary);
}

.select-input,
.text-input {
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--input-bg);
  color: var(--main-text);
  font-size: 13px;
}

.sync-actions {
  display: flex;
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
  max-height: 400px;
  overflow-y: auto;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 8px;
  background: var(--input-bg);
  margin-bottom: 12px;
}

.table-section-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--main-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  padding: 6px 8px 4px;
  border-bottom: 1px solid var(--border-color);
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
  background: var(--primary-light);
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
  color: var(--main-text-secondary);
  font-style: italic;
}

.table-select-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.selected-count-text {
  font-size: 13px;
  font-weight: 500;
  color: var(--primary-color);
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
  border: 3px solid var(--border-color);
  border-top-color: var(--primary-color);
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
  background: var(--card-bg);
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

.stat-source {
  background: #e3f2fd;
  color: #1565c0;
}

.stat-target {
  background: #f3e5f5;
  color: #7b1fa2;
}

.stat-common {
  background: #e8f5e9;
  color: #2e7d32;
}

.stat-diffs {
  background: #fff3e0;
  color: #e65100;
}

.diff-type-summary {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
  margin-top: 8px;
}

.diff-type-stat {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 4px;
}

.diff-green { background: #e8f5e9; color: #2e7d32; }
.diff-blue { background: #e3f2fd; color: #1565c0; }
.diff-orange { background: #fff3e0; color: #e65100; }
.diff-red { background: #ffebee; color: #c62828; }

/* Filter */
.results-filter {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
  padding: 8px 0;
}

.filter-label-text {
  font-size: 12px;
  color: var(--main-text-secondary);
  font-weight: 500;
}

.filter-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 12px;
  border: 1px solid var(--border-color);
  font-size: 11px;
  cursor: pointer;
  font-family: 'JetBrains Mono', monospace;
}

.filter-chip:hover {
  background: var(--primary-light);
}

/* ===== Navicat-style Grouped Diff View ===== */
.diff-grouped-view {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.diff-table-group {
  border: 1px solid var(--border-color);
  border-radius: 8px;
  overflow: hidden;
}

.diff-group-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  background: var(--card-bg);
  cursor: pointer;
  user-select: none;
  transition: background 0.1s;
}

.diff-group-header:hover {
  background: var(--primary-light);
}

.expand-icon {
  font-size: 10px;
  color: var(--main-text-secondary);
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
  color: var(--main-text-secondary);
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
  border-top: 1px solid var(--border-color);
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
  color: var(--main-text-secondary);
  background: var(--input-bg);
  border-bottom: 1px solid var(--border-color);
  position: sticky;
  top: 0;
}

.diff-compare-table td {
  padding: 6px 10px;
  border-bottom: 1px solid var(--border-color);
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
  background: var(--primary-light);
}

.diff-row.table_only_in_source { border-left: 3px solid #4caf50; }
.diff-row.column_added,
.diff-row.index_added { border-left: 3px solid #2196f3; }
.diff-row.column_modified,
.diff-row.index_modified { border-left: 3px solid #ff9800; }
.diff-row.table_only_in_target,
.diff-row.column_removed,
.diff-row.index_removed { border-left: 3px solid #f44336; }
.diff-row.primary_key_changed { border-left: 3px solid #9c27b0; }

/* Column widths */
.col-select { width: 32px; }
.col-type { width: 120px; }
.col-source { width: 30%; }
.col-target { width: 30%; }
.col-sql { width: 80px; text-align: center; }

.row-checkbox {
  display: flex;
  align-items: center;
  justify-content: center;
}

.type-tag {
  display: inline-block;
  font-size: 11px;
  padding: 2px 6px;
  border-radius: 3px;
  white-space: nowrap;
}

.compact-value {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  color: var(--main-text);
  word-break: break-all;
}

.sql-toggle-btn {
  font-size: 11px;
  padding: 2px 8px;
  border: 1px solid var(--border-color);
  border-radius: 3px;
  background: var(--input-bg);
  color: var(--primary-color);
  cursor: pointer;
  white-space: nowrap;
}

.sql-toggle-btn:hover {
  background: var(--primary-light);
}

.sql-detail-row td {
  padding: 0 !important;
  background: rgba(0, 0, 0, 0.03);
}

.sql-code {
  margin: 0;
  padding: 10px 14px;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 11px;
  line-height: 1.5;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
  color: var(--main-text);
}

.diff-empty {
  text-align: center;
  padding: 32px;
  color: var(--main-text-secondary);
  font-size: 14px;
}

.execute-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 0;
  border-top: 1px solid var(--border-color);
}

.selected-count {
  font-size: 13px;
  font-weight: 500;
  color: var(--primary-color);
}

.execute-btns {
  display: flex;
  gap: 8px;
}

.exec-result {
  background: var(--card-bg);
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
  background: var(--primary-color);
  color: white;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-ghost {
  background: transparent;
  color: var(--main-text-secondary);
  border: 1px solid var(--border-color);
}

.btn-sm {
  padding: 4px 10px;
  font-size: 12px;
}

.btn-xs {
  padding: 3px 8px;
  font-size: 11px;
  border-radius: 4px;
  border: 1px solid var(--border-color);
  background: transparent;
  color: var(--main-text-secondary);
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
  background: var(--card-bg);
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
  border-bottom: 1px solid var(--border-color);
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
  color: var(--main-text-secondary);
  font-size: 18px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.sql-dialog-close:hover {
  background: var(--input-bg);
  color: var(--main-text);
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
  background: var(--input-bg);
  border-radius: 8px;
  margin-bottom: 8px;
  font-size: 12px;
}

.sql-number {
  flex-shrink: 0;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: var(--primary-color);
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
  color: var(--main-text);
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
  background: var(--border-color);
}

</style>
