<template>
  <div class="flex flex-col h-full p-4 overflow-auto">
    <div class="mb-4">
      <h3 class="text-base font-semibold m-0 mb-1"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/></svg>  结构同步</h3>
      <p class="text-sm text-base-content/60 m-0">对比并同步两个数据库之间的表结构（Navicat 风格多表对比）</p>
    </div>

    <!-- Step 1: Connection & Database Selection -->
    <div v-if="step === 1" class="bg-base-100 rounded-lg p-4">
      <div class="grid grid-cols-[repeat(auto-fit,minmax(200px,1fr))] gap-3 mb-4">
        <div class="flex flex-col gap-1">
          <label class="text-xs font-medium text-base-content/60">🟢 源连接</label>
          <select v-model="sourceId" @change="onSourceChange" class="select select-bordered select-sm w-full">
            <option value="">选择源连接</option>
            <option v-for="conn in connections" :key="conn.id" :value="conn.id">
              {{ conn.name }} ({{ conn.type }})
            </option>
          </select>
        </div>
        <div class="flex flex-col gap-1">
          <label class="text-xs font-medium text-base-content/60">🔴 目标连接</label>
          <select v-model="targetId" @change="onTargetChange" class="select select-bordered select-sm w-full">
            <option value="">选择目标连接</option>
            <option v-for="conn in connections" :key="conn.id" :value="conn.id">
              {{ conn.name }} ({{ conn.type }})
            </option>
          </select>
        </div>
        <div class="flex flex-col gap-1">
          <label class="text-xs font-medium text-base-content/60"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>  源数据库</label>
          <select v-model="sourceDb" @change="loadSourceTables" :disabled="!sourceId || loadingSourceDb" class="select select-bordered select-sm w-full">
            <option value="">{{ loadingSourceDb ? '加载中...' : '选择数据库' }}</option>
            <option v-for="db in sourceDatabases" :key="db" :value="db">{{ db }}</option>
          </select>
        </div>
        <div class="flex flex-col gap-1">
          <label class="text-xs font-medium text-base-content/60"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>  目标数据库</label>
          <select v-model="targetDb" @change="loadTargetTables" :disabled="!targetId || loadingTargetDb" class="select select-bordered select-sm w-full">
            <option value="">{{ loadingTargetDb ? '加载中...' : '选择数据库' }}</option>
            <option v-for="db in targetDatabases" :key="db" :value="db">{{ db }}</option>
          </select>
        </div>
      </div>

      <div class="flex justify-end">
        <button @click="goToStep2" :disabled="!sourceId || !targetId || !sourceDb || !targetDb" class="btn btn-primary">
          下一步：选择表 →
        </button>
      </div>
    </div>

    <!-- Step 2: Multi-Table Selection -->
    <div v-if="step === 2" class="bg-base-100 rounded-lg p-4">
      <div class="flex items-center justify-between mb-3">
        <h4 class="text-sm font-semibold m-0"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>  选择要对比的表</h4>
        <div class="flex gap-1.5">
          <button @click="selectAllTables" class="btn btn-ghost btn-xs">全选</button>
          <button @click="selectCommonTables" class="btn btn-ghost btn-xs">仅共有表</button>
          <button @click="selectNone" class="btn btn-ghost btn-xs">清空</button>
        </div>
      </div>

      <div class="max-h-[400px] overflow-y-auto border border-base-content/10 rounded-lg p-2 bg-base-200 mb-3">
        <!-- Source-only tables -->
        <template v-if="sourceOnlyTables.length > 0">
          <div class="text-xs font-semibold text-base-content/60 uppercase tracking-wider px-2 py-1.5 pb-1 border-b border-base-content/10 mb-1 mt-2 first:mt-0">仅源端有</div>
          <div v-for="table in sourceOnlyTables" :key="'src-' + table">
            <label class="flex items-center gap-2 px-2 py-1.5 rounded cursor-pointer text-sm hover:bg-primary/10">
              <input type="checkbox" v-model="selectedTables" :value="table" />
              <span class="flex-1 font-mono text-xs">{{ table }}</span>
              <span class="text-[10px] px-1.5 py-0.5 rounded font-medium bg-blue-100 text-blue-700">仅源</span>
            </label>
          </div>
        </template>

        <!-- Target-only tables -->
        <template v-if="targetOnlyTables.length > 0">
          <div class="text-xs font-semibold text-base-content/60 uppercase tracking-wider px-2 py-1.5 pb-1 border-b border-base-content/10 mb-1 mt-2 first:mt-0">仅目标端有</div>
          <div v-for="table in targetOnlyTables" :key="'tgt-' + table">
            <label class="flex items-center gap-2 px-2 py-1.5 rounded cursor-pointer text-sm hover:bg-primary/10">
              <input type="checkbox" v-model="selectedTables" :value="table" />
              <span class="flex-1 font-mono text-xs">{{ table }}</span>
              <span class="text-[10px] px-1.5 py-0.5 rounded font-medium bg-purple-100 text-purple-700">仅目标</span>
            </label>
          </div>
        </template>

        <!-- Common tables -->
        <template v-if="commonTablesList.length > 0">
          <div class="text-xs font-semibold text-base-content/60 uppercase tracking-wider px-2 py-1.5 pb-1 border-b border-base-content/10 mb-1 mt-2 first:mt-0">共有表（{{ commonTablesList.length }}）</div>
          <div v-for="table in commonTablesList" :key="'common-' + table">
            <label class="flex items-center gap-2 px-2 py-1.5 rounded cursor-pointer text-sm hover:bg-primary/10">
              <input type="checkbox" v-model="selectedTables" :value="table" />
              <span class="flex-1 font-mono text-xs">{{ table }}</span>
              <span class="text-[10px] px-1.5 py-0.5 rounded font-medium bg-green-100 text-green-700">共有</span>
            </label>
          </div>
        </template>

        <div v-if="sourceTables.length === 0 && targetTables.length === 0" class="text-center p-6 text-base-content/60 italic">
          两个数据库都没有表
        </div>
      </div>

      <div class="flex items-center justify-between">
        <span class="text-sm font-medium text-primary">已选 {{ selectedTables.length }} 张表</span>
        <div class="flex gap-2">
          <button @click="step = 1" class="btn btn-ghost">← 返回</button>
          <button @click="startCompare" :disabled="selectedTables.length === 0 || comparing" class="btn btn-primary">
            {{ comparing ? '对比中...' : `🔍 对比 ${selectedTables.length} 张表` }}
          </button>
        </div>
      </div>
    </div>

    <!-- Comparing State -->
    <div v-if="comparing" class="flex flex-col items-center justify-center p-12 gap-3">
      <span class="loading loading-spinner loading-md"></span>
      <p>正在对比表结构... ({{ compareProgress }}/{{ selectedTables.length }})</p>
    </div>

    <!-- Results -->
    <div v-if="result && !comparing" class="flex flex-col gap-4">
      <div class="bg-base-100 rounded-lg p-3 px-4">
        <h4 class="m-0 mb-2 text-sm">对比结果</h4>
        <div class="flex gap-3 flex-wrap">
          <span class="px-2.5 py-1 rounded text-xs font-medium bg-orange-100 text-orange-700">差异项: {{ result.diffs.length }}</span>
          <span class="px-2.5 py-1 rounded text-xs font-medium bg-blue-100 text-blue-700">涉及表: {{ affectedTablesList.length }}</span>
        </div>
        <div class="flex gap-2.5 flex-wrap mt-2">
          <span v-for="(count, type) in diffTypeCounts" :key="type" class="text-xs px-2 py-0.5 rounded" :class="getDiffTypeClass(type)">
            {{ getDiffTypeLabel(type) }}: {{ count }}
          </span>
        </div>
      </div>

      <!-- Filter by table -->
      <div v-if="affectedTablesList.length > 1" class="flex items-center gap-1.5 flex-wrap py-2">
        <span class="text-xs text-base-content/60 font-medium">按表筛选:</span>
        <label v-for="table in affectedTablesList" :key="table" class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full border border-base-content/10 text-xs cursor-pointer font-mono hover:bg-primary/10">
          <input type="checkbox" v-model="filterTables" :value="table" />
          {{ table }}
        </label>
      </div>

      <!-- Navicat-style Grouped Diff Table -->
      <div class="flex flex-col gap-1">
        <div v-for="(group, tIdx) in groupedDiffs" :key="tIdx" class="border border-base-content/10 rounded-lg overflow-hidden">
          <!-- Table group header -->
          <div class="flex items-center gap-2.5 px-3.5 py-2.5 bg-base-100 cursor-pointer select-none hover:bg-primary/10" @click="toggleTableExpand(group.tableName)">
            <span class="text-[10px] text-base-content/60 w-3 text-center">{{ isTableExpanded(group.tableName) ? '▼' : '▶' }}</span>
            <span class="font-mono text-sm font-semibold">{{ group.tableName }}</span>
            <span class="text-xs text-base-content/60">{{ group.diffs.length }} 项差异</span>
            <span class="flex gap-1.5 flex-wrap ml-auto">
              <span v-for="(count, type) in group.typeCounts" :key="type" class="text-[10px] px-1.5 py-0.5 rounded font-medium" :class="getDiffTypeClass(type)">
                {{ getDiffTypeLabel(type) }} ×{{ count }}
              </span>
            </span>
          </div>

          <!-- Expanded diff rows -->
          <div v-if="isTableExpanded(group.tableName)" class="border-t border-base-content/10">
            <table class="w-full border-collapse text-xs">
              <thead>
                <tr>
                  <th class="text-left px-2.5 py-1.5 text-xs font-semibold text-base-content/60 bg-base-200 border-b border-base-content/10 sticky top-0 w-8"></th>
                  <th class="text-left px-2.5 py-1.5 text-xs font-semibold text-base-content/60 bg-base-200 border-b border-base-content/10 sticky top-0 w-[120px]">差异类型</th>
                  <th class="text-left px-2.5 py-1.5 text-xs font-semibold text-base-content/60 bg-base-200 border-b border-base-content/10 sticky top-0 w-[30%]">源</th>
                  <th class="text-left px-2.5 py-1.5 text-xs font-semibold text-base-content/60 bg-base-200 border-b border-base-content/10 sticky top-0 w-[30%]">目标</th>
                  <th class="text-left px-2.5 py-1.5 text-xs font-semibold text-base-content/60 bg-base-200 border-b border-base-content/10 sticky top-0 w-20 text-center">SQL</th>
                </tr>
              </thead>
              <tbody>
                <template v-for="(diff, dIdx) in group.diffs" :key="dIdx">
                  <tr class="hover:bg-primary/10" :class="getDiffRowClass(diff.diffType)">
                    <td class="px-2.5 py-1.5 border-b border-base-content/10 align-middle">
                      <label class="flex items-center justify-center">
                        <input type="checkbox" :checked="selectedSqls.has(diff.sql)" @change="toggleSql(diff.sql)" />
                      </label>
                    </td>
                    <td class="px-2.5 py-1.5 border-b border-base-content/10 align-middle">
                      <span class="inline-block text-xs px-1.5 py-0.5 rounded whitespace-nowrap" :class="getDiffTypeClass(diff.diffType)">
                        {{ getDiffTypeLabel(diff.diffType) }}
                      </span>
                    </td>
                    <td class="px-2.5 py-1.5 border-b border-base-content/10 align-middle">
                      <span class="font-mono text-xs text-base-content break-all">{{ getCompactValue(diff.sourceValue) }}</span>
                    </td>
                    <td class="px-2.5 py-1.5 border-b border-base-content/10 align-middle">
                      <span class="font-mono text-xs text-base-content break-all">{{ getCompactValue(diff.targetValue) }}</span>
                    </td>
                    <td class="px-2.5 py-1.5 border-b border-base-content/10 align-middle text-center">
                      <button class="text-xs px-2 py-0.5 border border-base-content/10 rounded bg-base-200 text-primary cursor-pointer whitespace-nowrap hover:bg-primary/10" @click="toggleSqlRow(tIdx + '-' + dIdx)">
                        {{ isSqlRowExpanded(tIdx + '-' + dIdx) ? '隐藏' : '查看' }} SQL
                      </button>
                    </td>
                  </tr>
                  <!-- Expandable SQL row -->
                  <tr v-if="isSqlRowExpanded(tIdx + '-' + dIdx)">
                    <td colspan="5" class="!p-0 bg-black/[0.03]">
                      <pre class="m-0 p-2.5 px-3.5 font-mono text-xs leading-relaxed overflow-x-auto whitespace-pre-wrap break-all text-base-content">{{ diff.sql }}</pre>
                    </td>
                  </tr>
                </template>
              </tbody>
            </table>
          </div>
        </div>

        <div v-if="filteredDiffs.length === 0" class="text-center p-8 text-base-content/60 text-sm">
          所选表结构完全一致 <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg> 
        </div>
      </div>

      <!-- Execute Actions -->
      <div class="flex items-center justify-between px-3 py-3 border-t border-base-content/10">
        <span class="text-sm font-medium text-primary">已选择 {{ selectedSqls.size }} 项更改</span>
        <div class="flex gap-2">
          <button @click="selectAll" class="btn btn-ghost">全选</button>
          <button @click="reset" class="btn btn-ghost">重新对比</button>
          <button @click="showSqlDialog = true" :disabled="selectedSqls.size === 0" class="btn btn-ghost"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>  查看SQL</button>
          <button @click="executeSync" :disabled="selectedSqls.size === 0 || executing" class="btn btn-primary">
            {{ executing ? '执行中...' : '🚀 执行同步' }}
          </button>
        </div>
      </div>
    </div>

    <!-- SQL Preview Dialog -->
    <Teleport to="body">
      <div v-if="showSqlDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[2000]" @click="showSqlDialog = false">
        <div class="bg-base-100 rounded-xl w-[720px] max-w-[90vw] max-h-[80vh] flex flex-col shadow-2xl" @click.stop>
          <div class="flex items-center justify-between px-5 py-4 border-b border-base-content/10">
            <h3 class="m-0 text-base font-semibold"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>  待执行 SQL ({{ selectedSqls.size }} 条)</h3>
            <div class="flex items-center gap-2">
              <button @click="copyAllSql" class="btn btn-ghost btn-sm"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>  复制全部</button>
              <button @click="showSqlDialog = false" class="w-7 h-7 border-none rounded-md bg-transparent text-base-content/60 text-lg cursor-pointer flex items-center justify-center hover:bg-base-200 hover:text-base-content">×</button>
            </div>
          </div>
          <div class="px-5 py-4 overflow-y-auto flex-1">
            <div v-for="(sql, idx) in selectedSqlArray" :key="idx" class="flex items-start gap-2.5 px-3 py-2.5 bg-base-200 rounded-lg mb-2 text-xs">
              <span class="shrink-0 w-[22px] h-[22px] rounded-full bg-primary text-white flex items-center justify-center text-xs font-semibold mt-0.5">{{ idx + 1 }}</span>
              <pre class="flex-1 m-0 p-0 whitespace-pre-wrap break-all font-mono text-xs text-base-content bg-transparent">{{ sql }}</pre>
              <button @click="copySingleSql(idx)" class="shrink-0 px-2 py-1 border-none rounded bg-transparent cursor-pointer text-xs hover:bg-base-content/10" title="复制"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg> </button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Execution Result -->
    <div v-if="execResult" class="bg-base-100 rounded-lg p-4 text-center" :class="{ 'border border-green-500': execResult.success, 'border border-red-500': !execResult.success }">
      <h4 class="m-0 mb-2">{{ execResult.success ? '✅ 同步成功' : '❌ 同步失败' }}</h4>
      <p>已执行 {{ execResult.executed }} 项更改</p>
      <div v-if="execResult.errors.length > 0" class="text-left m-3 p-3 bg-red-100 rounded">
        <p>错误信息:</p>
        <ul class="m-2 mt-0 pl-5">
          <li v-for="(err, idx) in execResult.errors" :key="idx" class="text-xs text-red-700 mb-1">{{ err }}</li>
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
    table_only_in_source: 'bg-green-100 text-green-700',
    column_added: 'bg-blue-100 text-blue-700',
    column_modified: 'bg-orange-100 text-orange-700',
    index_added: 'bg-blue-100 text-blue-700',
    index_modified: 'bg-orange-100 text-orange-700',
    table_only_in_target: 'bg-red-100 text-red-700',
    column_removed: 'bg-red-100 text-red-700',
    index_removed: 'bg-red-100 text-red-700',
    primary_key_changed: 'bg-red-100 text-red-700',
  }
  return classes[type] || ''
}

function getDiffRowClass(type: string): string {
  const classes: Record<string, string> = {
    table_only_in_source: 'border-l-[3px] border-green-500',
    column_added: 'border-l-[3px] border-blue-500',
    column_modified: 'border-l-[3px] border-orange-500',
    index_added: 'border-l-[3px] border-blue-500',
    index_modified: 'border-l-[3px] border-orange-500',
    table_only_in_target: 'border-l-[3px] border-red-500',
    column_removed: 'border-l-[3px] border-red-500',
    index_removed: 'border-l-[3px] border-red-500',
    primary_key_changed: 'border-l-[3px] border-purple-500',
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
