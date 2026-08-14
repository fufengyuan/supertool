<template>
  <div class="flex flex-col h-full min-h-0 bg-base-200">
    <!-- 顶部：集群健康概览条 -->
    <div class="flex items-center gap-3 px-3 py-2 bg-base-100 border-b border-base-content/10 flex-wrap">
      <span class="flex items-center gap-1.5 text-[11px] font-semibold px-2 py-0.5 rounded-full border" :class="healthBadgeClass">
        <span class="w-1.5 h-1.5 rounded-full" :class="healthDotClass"></span>
        {{ health?.cluster_name || 'Elasticsearch' }} · {{ health?.status || '...' }}
      </span>
      <span class="text-xs text-base-content/60 flex items-center gap-1"><SvgIcon name="server" size="12" /> {{ nodes.length }} 节点</span>
      <span class="text-xs text-base-content/60 flex items-center gap-1"><SvgIcon name="layers" size="12" /> {{ indices.length }} 索引</span>
      <span class="text-xs text-base-content/60 flex items-center gap-1"><SvgIcon name="fileText" size="12" /> {{ totalDocs.toLocaleString() }} 文档</span>
      <span class="text-xs text-base-content/60 flex items-center gap-1"><SvgIcon name="database" size="12" /> {{ formatBytes(totalStore) }}</span>
      <div class="ml-auto flex gap-1.5">
        <button @click="loadAll" class="btn btn-ghost btn-xs gap-1" :disabled="loading">
          <SvgIcon name="refresh" size="12" :class="{ 'animate-spin': loading }" /> 刷新
        </button>
        <button @click="openCreateIndex" class="btn btn-primary btn-xs gap-1">
          <SvgIcon name="plus" size="12" /> 创建索引
        </button>
      </div>
    </div>

    <div v-if="error" class="px-3 py-2 bg-error/10 border-b border-error/20 text-error text-xs flex items-center gap-1.5">
      <SvgIcon name="alertTriangle" size="13" /> {{ error }}
    </div>

    <div class="flex flex-1 min-h-0">
      <!-- 左栏：索引列表 -->
      <aside class="w-60 shrink-0 flex flex-col border-r border-base-content/10 bg-base-100 min-h-0">
        <div class="p-2 border-b border-base-content/10">
          <div class="relative">
            <SvgIcon name="search" size="13" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-base-content/40" />
            <input v-model="indexSearch" class="input input-bordered input-xs w-full pl-7" placeholder="搜索索引..." />
          </div>
        </div>
        <div class="flex-1 overflow-y-auto min-h-0">
          <div v-if="filteredIndices.length === 0" class="text-center text-xs text-base-content/40 py-8">
            {{ loading ? '加载中...' : '无索引' }}
          </div>
          <div
            v-for="idx in filteredIndices"
            :key="idx.index"
            class="flex items-center gap-1.5 px-2.5 py-1.5 cursor-pointer text-xs hover:bg-base-200 transition-colors border-l-2"
            :class="selectedIndexName === idx.index ? 'bg-base-200 border-primary' : 'border-transparent'"
            @click="selectIndex(idx.index)"
            @dblclick="openDiscover(idx.index)"
            :title="idx.index"
          >
            <span class="w-1.5 h-1.5 rounded-full shrink-0" :class="healthDotClassFor(idx.health)"></span>
            <span class="flex-1 truncate font-medium">{{ idx.index }}</span>
            <span class="text-[10px] text-base-content/40 tabular-nums shrink-0">{{ formatDocs(idx) }}</span>
            <span class="text-[10px] text-base-content/30 tabular-nums shrink-0">{{ formatBytes(parseNum(idx['store.size'])) }}</span>
          </div>
        </div>
        <div class="p-2 border-t border-base-content/10 flex gap-1">
          <button @click="selectIndex(null); loadAll()" class="btn btn-ghost btn-xs flex-1 gap-1">
            <SvgIcon name="grid" size="12" /> 概览
          </button>
        </div>
      </aside>

      <!-- 右侧内容区 -->
      <main class="flex-1 min-w-0 flex flex-col min-h-0">
        <!-- 概览（未选索引） -->
        <template v-if="!selectedIndexName">
          <div class="flex-1 overflow-y-auto p-4">
            <div class="bg-base-100 border rounded-xl p-4 mb-4">
              <h3 class="text-sm font-semibold m-0 mb-3 flex items-center gap-1.5"><SvgIcon name="server" size="14" class="text-primary" /> 集群健康</h3>
              <div class="grid grid-cols-2 md:grid-cols-4 gap-3">
                <StatCard label="状态" :value="health?.status || '-'" :accent="health?.status === 'green' ? 'text-success' : health?.status === 'yellow' ? 'text-warning' : 'text-error'" />
                <StatCard label="节点数" :value="String(health?.number_of_nodes ?? '-')" />
                <StatCard label="活跃分片" :value="String(health?.active_shards ?? '-')" />
                <StatCard label="未分配分片" :value="String(health?.unassigned_shards ?? '-')" />
              </div>
            </div>
            <div class="bg-base-100 border rounded-xl p-4">
              <h3 class="text-sm font-semibold m-0 mb-3 flex items-center gap-1.5"><SvgIcon name="serverRack" size="14" class="text-primary" /> 节点列表</h3>
              <div class="overflow-x-auto">
                <table class="table table-xs">
                  <thead>
                    <tr>
                      <th>名称</th><th>IP</th><th>CPU</th><th>堆内存</th><th>RAM</th><th>负载</th><th>磁盘</th><th>主节点</th><th>版本</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="n in nodes" :key="n.name">
                      <td class="font-medium">{{ n.name }}</td>
                      <td class="tabular-nums">{{ n.ip }}</td>
                      <td>{{ n.cpu ?? '-' }}%</td>
                      <td>{{ n['heap.percent'] ?? '-' }}%</td>
                      <td>{{ n['ram.percent'] ?? '-' }}%</td>
                      <td>{{ n.load_1m ?? '-' }}</td>
                      <td>{{ n['disk.used_percent'] ?? '-' }}%</td>
                      <td>{{ n.master === '*' ? '是' : '' }}</td>
                      <td class="tabular-nums">{{ n.version }}</td>
                    </tr>
                    <tr v-if="nodes.length === 0"><td colspan="9" class="text-center text-base-content/40">暂无节点信息</td></tr>
                  </tbody>
                </table>
              </div>
            </div>
          </div>
        </template>

        <!-- 已选索引 -->
        <template v-else>
          <!-- 索引页签 -->
          <div class="flex items-center gap-1 px-3 pt-2 bg-base-100 border-b border-base-content/10">
            <button
              class="px-3 py-1.5 text-xs rounded-t-lg border-b-2 transition-colors"
              :class="detailTab === 'overview' ? 'border-primary text-primary font-medium' : 'border-transparent text-base-content/60 hover:text-base-content'"
              @click="detailTab = 'overview'"
            ><SvgIcon name="grid" size="12" class="inline mr-1" />索引详情</button>
            <button
              class="px-3 py-1.5 text-xs rounded-t-lg border-b-2 transition-colors"
              :class="detailTab === 'discover' ? 'border-primary text-primary font-medium' : 'border-transparent text-base-content/60 hover:text-base-content'"
              @click="detailTab = 'discover'"
            ><SvgIcon name="search" size="12" class="inline mr-1" />文档浏览</button>
            <div class="ml-auto flex gap-1.5 pb-1.5">
              <button @click="openAliases" class="btn btn-outline btn-xs gap-1"><SvgIcon name="link" size="12" /> 别名</button>
              <button @click="openReindex" class="btn btn-outline btn-xs gap-1"><SvgIcon name="refresh" size="12" /> reindex</button>
              <button @click="confirmDeleteIndex" class="btn btn-outline btn-error btn-xs gap-1"><SvgIcon name="trash" size="12" /> 删除</button>
            </div>
          </div>

          <!-- 索引详情 -->
          <div v-if="detailTab === 'overview'" class="flex-1 overflow-y-auto p-4">
            <div class="grid grid-cols-2 md:grid-cols-4 gap-3 mb-4">
              <StatCard label="文档数" :value="indexStats?.docs?.count != null ? Number(indexStats.docs.count).toLocaleString() : '-'" />
              <StatCard label="存储" :value="formatBytes(parseNum(indexStats?.store?.size_in_bytes))" />
              <StatCard label="分片" :value="indexSettings?.number_of_shards ?? '-'" />
              <StatCard label="副本" :value="indexSettings?.number_of_replicas ?? '-'" />
            </div>

            <div class="bg-base-100 border rounded-xl p-4 mb-4">
              <h3 class="text-sm font-semibold m-0 mb-2 flex items-center gap-1.5"><SvgIcon name="layers" size="14" class="text-primary" /> Mappings 字段结构</h3>
              <div class="overflow-x-auto max-h-64 overflow-y-auto bg-base-200/50 rounded-lg p-3 font-mono text-xs leading-relaxed">
                <template v-if="mappingFields.length > 0">
                  <div v-for="f in mappingFields" :key="f.name" class="flex items-center gap-2 py-0.5">
                    <span class="text-primary">{{ f.name }}</span>
                    <span class="badge badge-outline badge-xs" :class="typeBadgeClass(f.type)">{{ f.type }}</span>
                  </div>
                </template>
                <span v-else class="text-base-content/40">无字段（动态映射或 mappings 为空）</span>
              </div>
            </div>

            <div class="bg-base-100 border rounded-xl p-4 mb-4">
              <div class="flex items-center justify-between mb-2">
                <h3 class="text-sm font-semibold m-0 flex items-center gap-1.5"><SvgIcon name="settings" size="14" class="text-primary" /> Index Settings</h3>
                <button @click="saveSettings" class="btn btn-primary btn-xs gap-1" :disabled="savingSettings">
                  <SvgIcon name="save" size="12" /> 保存
                </button>
              </div>
              <textarea v-model="settingsText" rows="8" spellcheck="false" class="textarea textarea-bordered w-full font-mono text-xs bg-base-200/60" />
            </div>

            <div class="bg-base-100 border rounded-xl p-4">
              <h3 class="text-sm font-semibold m-0 mb-2 flex items-center gap-1.5"><SvgIcon name="link" size="14" class="text-primary" /> 别名</h3>
              <div v-if="indexAliases.length === 0" class="text-xs text-base-content/40">无别名</div>
              <div v-else class="flex flex-wrap gap-2">
                <span v-for="a in indexAliases" :key="a" class="badge badge-outline gap-1">{{ a }}</span>
              </div>
            </div>
          </div>

          <!-- 文档浏览（Discover 式） -->
          <div v-else class="flex-1 min-h-0 flex flex-col">
            <!-- 过滤条件构建区 -->
            <div class="bg-base-100 border-b border-base-content/10 p-3 space-y-2">
              <div class="flex items-center gap-2 flex-wrap">
                <div class="flex items-center gap-2 flex-1 min-w-[280px]">
                  <SvgIcon name="search" size="13" class="text-base-content/40" />
                  <input v-model="keyword" class="input input-bordered input-xs flex-1" placeholder="全文关键词（query_string 语法，如 error AND status:400）" @keydown.enter="search" />
                </div>
                <button @click="search" class="btn btn-primary btn-xs gap-1" :disabled="searching">
                  <SvgIcon name="search" size="12" /> 搜索
                </button>
                <button @click="resetFilters" class="btn btn-ghost btn-xs">重置</button>
              </div>
              <div v-for="(f, i) in filters" :key="i" class="flex items-center gap-1.5 flex-wrap">
                <select v-model="f.field" class="select select-bordered select-xs w-44">
                  <option value="">-- 字段 --</option>
                  <option v-for="fd in mappingFields" :key="fd.name" :value="fd.name">{{ fd.name }} ({{ fd.type }})</option>
                </select>
                <select v-model="f.op" class="select select-bordered select-xs w-32">
                  <option v-for="op in opsForField(f.field)" :key="op[0]" :value="op[0]">{{ op[1] }}</option>
                </select>
                <template v-if="f.op === 'between'">
                  <input v-model="f.value" class="input input-bordered input-xs w-32" placeholder="最小值" />
                  <span class="text-xs text-base-content/40">~</span>
                  <input v-model="f.value2" class="input input-bordered input-xs w-32" placeholder="最大值" />
                </template>
                <input v-else v-model="f.value" class="input input-bordered input-xs w-48" placeholder="值" @keydown.enter="search" />
                <button @click="filters.splice(i, 1)" class="btn btn-ghost btn-xs px-1.5"><SvgIcon name="x" size="12" /></button>
              </div>
              <div class="flex items-center gap-2">
                <button @click="addFilter" class="btn btn-outline btn-xs gap-1"><SvgIcon name="plus" size="12" /> 添加过滤条件</button>
                <button @click="showDsl = !showDsl" class="btn btn-ghost btn-xs">查看 DSL</button>
              </div>
              <pre v-if="showDsl" class="bg-base-200/60 rounded-lg p-2 font-mono text-[10px] overflow-x-auto max-h-40">{{ dslPreview }}</pre>
            </div>

            <!-- 结果区 -->
            <div class="flex-1 min-h-0 flex flex-col">
              <div class="flex items-center justify-between px-3 py-1.5 text-xs text-base-content/60 border-b border-base-content/10 bg-base-100">
                <span>共 <span class="font-semibold text-base-content">{{ total }}</span> 条 · 每页 {{ pageSize }} 条</span>
                <div class="flex items-center gap-2">
                  <button @click="page--; search()" class="btn btn-ghost btn-xs" :disabled="page <= 1"><SvgIcon name="chevronLeft" size="12" /></button>
                  <span class="tabular-nums">第 {{ page }} / {{ totalPages }} 页</span>
                  <button @click="page++; search()" class="btn btn-ghost btn-xs" :disabled="page >= totalPages"><SvgIcon name="chevronRight" size="12" /></button>
                  <button @click="openCreateDoc" class="btn btn-primary btn-xs gap-1 ml-2"><SvgIcon name="plus" size="12" /> 新增文档</button>
                </div>
              </div>
              <div class="flex-1 overflow-auto min-h-0">
                <table class="table table-xs">
                  <thead class="sticky top-0 bg-base-100 z-10">
                    <tr>
                      <th class="w-10">#</th>
                      <th>_id</th>
                      <th v-for="col in resultColumns" :key="col">{{ col }}</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="(hit, i) in hits" :key="hit._id" class="cursor-pointer hover:bg-base-200" @click="openDocDetail(hit)">
                      <td class="text-base-content/40 tabular-nums">{{ (page - 1) * pageSize + i + 1 }}</td>
                      <td class="font-mono text-xs max-w-[160px] truncate">{{ hit._id }}</td>
                      <td v-for="col in resultColumns" :key="col" class="max-w-[220px] truncate text-xs" :title="cellText(hit._source?.[col])">{{ cellText(hit._source?.[col]) }}</td>
                    </tr>
                    <tr v-if="hits.length === 0 && !searching">
                      <td :colspan="resultColumns.length + 2" class="text-center text-base-content/40 py-8">无匹配文档</td>
                    </tr>
                    <tr v-if="searching"><td :colspan="resultColumns.length + 2" class="text-center text-base-content/40 py-8">搜索中...</td></tr>
                  </tbody>
                </table>
              </div>
            </div>
          </div>
        </template>
      </main>
    </div>

    <!-- 创建索引 -->
    <Modal :model-value="showCreateIndex" @update:model-value="v => (showCreateIndex = v)" title="创建索引" width="560px">
      <div class="space-y-3">
        <div>
          <label class="label"><span class="label-text">索引名称 <span class="text-error">*</span></span></label>
          <input v-model="createIndexName" class="input input-bordered w-full text-xs font-mono" placeholder="my-index-2026.01" />
        </div>
        <div>
          <label class="label"><span class="label-text">Settings (JSON，可选)</span></label>
          <textarea v-model="createSettingsText" rows="5" spellcheck="false" class="textarea textarea-bordered w-full font-mono text-xs bg-base-200/60" placeholder='{"number_of_shards": 1, "number_of_replicas": 0}' />
        </div>
        <div>
          <label class="label"><span class="label-text">Mappings (JSON，可选)</span></label>
          <textarea v-model="createMappingsText" rows="6" spellcheck="false" class="textarea textarea-bordered w-full font-mono text-xs bg-base-200/60" placeholder='{"properties": {"name": {"type": "text"}, "age": {"type": "integer"}}}' />
        </div>
        <p v-if="createError" class="text-error text-xs">{{ createError }}</p>
      </div>
      <template #footer>
        <button @click="showCreateIndex = false" class="btn btn-ghost btn-sm">取消</button>
        <button @click="doCreateIndex" class="btn btn-primary btn-sm gap-1" :disabled="creating"><SvgIcon name="plus" size="12" /> 创建</button>
      </template>
    </Modal>

    <!-- 删除索引确认 -->
    <Modal :model-value="showDeleteConfirm" @update:model-value="v => (showDeleteConfirm = v)" title="删除索引" width="440px">
      <p class="text-sm leading-relaxed">
        确定要删除索引 <span class="font-mono font-semibold text-error">{{ selectedIndexName }}</span> 吗？<br />
        <span class="text-xs text-base-content/50">该操作会永久删除索引及其全部数据，且不可恢复。</span>
      </p>
      <template #footer>
        <button @click="showDeleteConfirm = false" class="btn btn-ghost btn-sm">取消</button>
        <button @click="doDeleteIndex" class="btn btn-error btn-sm gap-1" :disabled="deleting"><SvgIcon name="trash" size="12" /> 确认删除</button>
      </template>
    </Modal>

    <!-- 别名管理 -->
    <Modal :model-value="showAliases" @update:model-value="v => (showAliases = v)" title="索引别名" width="440px">
      <p class="text-xs text-base-content/50 mb-2">索引：<span class="font-mono">{{ selectedIndexName }}</span></p>
      <div class="flex items-center gap-2 mb-3">
        <input v-model="newAlias" class="input input-bordered input-sm flex-1" placeholder="新别名" />
        <button @click="addAlias" class="btn btn-primary btn-sm gap-1"><SvgIcon name="plus" size="12" /> 添加</button>
      </div>
      <div v-if="indexAliases.length === 0" class="text-xs text-base-content/40">无别名</div>
      <div v-for="a in indexAliases" :key="a" class="flex items-center gap-2 py-1.5 border-b border-base-content/5">
        <span class="flex-1 font-mono text-sm">{{ a }}</span>
        <button @click="removeAlias(a)" class="btn btn-ghost btn-xs text-error"><SvgIcon name="trash" size="12" /></button>
      </div>
      <template #footer>
        <button @click="showAliases = false" class="btn btn-ghost btn-sm">关闭</button>
      </template>
    </Modal>

    <!-- Reindex -->
    <Modal :model-value="showReindex" @update:model-value="v => (showReindex = v)" title="索引重建 (reindex)" width="440px">
      <div class="space-y-3">
        <div>
          <label class="label"><span class="label-text">源索引</span></label>
          <input v-model="reindexSource" class="input input-bordered w-full text-xs font-mono" />
        </div>
        <div>
          <label class="label"><span class="label-text">目标索引 <span class="text-error">*</span></span></label>
          <input v-model="reindexDest" class="input input-bordered w-full text-xs font-mono" placeholder="my-index-new" />
        </div>
        <p v-if="reindexResult" class="bg-success/10 text-success text-xs rounded-lg p-2 font-mono whitespace-pre-wrap">{{ reindexResult }}</p>
        <p v-if="reindexError" class="text-error text-xs">{{ reindexError }}</p>
      </div>
      <template #footer>
        <button @click="showReindex = false" class="btn btn-ghost btn-sm">关闭</button>
        <button @click="doReindex" class="btn btn-primary btn-sm gap-1" :disabled="reindexing"><SvgIcon name="refresh" size="12" /> 执行</button>
      </template>
    </Modal>

    <!-- 文档详情 / 编辑 -->
    <Modal :model-value="showDocEdit" @update:model-value="v => (showDocEdit = v)" :title="docEditMode === 'create' ? '新增文档' : `编辑文档 _id: ${editingDocId || ''}`" width="640px" max-height="80vh">
      <div class="space-y-3">
        <div v-if="docEditMode === 'create'">
          <label class="label"><span class="label-text">文档 ID（留空自动生成）</span></label>
          <input v-model="editingDocId" class="input input-bordered w-full text-xs font-mono" placeholder="留空则 POST 自动生成 _id" />
        </div>
        <div>
          <label class="label"><span class="label-text">_source (JSON)</span></label>
          <textarea v-model="docJson" rows="12" spellcheck="false" class="textarea textarea-bordered w-full font-mono text-xs bg-base-200/60" />
        </div>
        <p v-if="docError" class="text-error text-xs">{{ docError }}</p>
        <p v-if="docSuccess" class="bg-success/10 text-success text-xs rounded-lg p-2 font-mono whitespace-pre-wrap">{{ docSuccess }}</p>
      </div>
      <template #footer>
        <template v-if="docEditMode === 'edit'">
          <button @click="deleteEditingDoc" class="btn btn-error btn-sm gap-1 mr-auto"><SvgIcon name="trash" size="12" /> 删除文档</button>
        </template>
        <button @click="showDocEdit = false" class="btn btn-ghost btn-sm">关闭</button>
        <button @click="saveDoc" class="btn btn-primary btn-sm gap-1" :disabled="savingDoc"><SvgIcon name="save" size="12" /> 保存</button>
      </template>
    </Modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, h } from 'vue'
import { confirm } from '@tauri-apps/plugin-dialog'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import Modal from '@/components/ui/Modal.vue'
import { getTauriAPI } from '../../utils/tauri-api'
import type { DBConnection } from '../../composables/useDBManager'

defineOptions({ name: 'EsManager' })

// 统计卡片（内联函数式组件）
const StatCard = (props: { label: string; value: string; accent?: string }) =>
  h('div', { class: 'bg-base-200/50 rounded-lg p-3' }, [
    h('div', { class: 'text-[11px] text-base-content/50 mb-1' }, props.label),
    h('div', { class: `text-lg font-semibold ${props.accent || 'text-base-content'}` }, props.value),
  ])

const props = defineProps<{
  connectionId: string
  connectionName: string
  connection?: DBConnection
}>()

// ============ 集群/索引状态 ============
const loading = ref(false)
const error = ref('')
const health = ref<any>(null)
const nodes = ref<any[]>([])
const indices = ref<any[]>([])
const indexSearch = ref('')
const selectedIndexName = ref<string | null>(null)
const detailTab = ref<'overview' | 'discover'>('overview')
const indexInfo = ref<any>(null)
const mappingFields = ref<Array<{ name: string; type: string }>>([])
const settingsText = ref('')
const savingSettings = ref(false)

const filteredIndices = computed(() => {
  const q = indexSearch.value.trim().toLowerCase()
  if (!q) {return indices.value}
  return indices.value.filter((i: any) => String(i.index || '').toLowerCase().includes(q))
})
const totalDocs = computed(() => indices.value.reduce((s: number, i: any) => s + parseNum(i['docs.count']), 0))
const totalStore = computed(() => indices.value.reduce((s: number, i: any) => s + parseNum(i['store.size']), 0))
const healthBadgeClass = computed(() => {
  switch (health.value?.status) {
    case 'green': return 'bg-success/10 text-success border-success/20'
    case 'yellow': return 'bg-warning/10 text-warning border-warning/20'
    case 'red': return 'bg-error/10 text-error border-error/20'
    default: return 'bg-base-200 text-base-content/50 border-base-content/10'
  }
})
const healthDotClass = computed(() => {
  switch (health.value?.status) {
    case 'green': return 'bg-success'
    case 'yellow': return 'bg-warning'
    case 'red': return 'bg-error'
    default: return 'bg-base-content/30'
  }
})

const indexSettings = computed(() => {
  if (!indexInfo.value?.info) {return null}
  const keys = Object.keys(indexInfo.value.info)
  const idx = keys[0] as string | undefined
  if (!idx) {return null}
  return indexInfo.value.info[idx]?.settings?.index || null
})
const indexStats = computed(() => {
  const name = selectedIndexName.value
  if (!name) {return null}
  // ES _stats 响应：indices[name] 下直接是 primaries/total
  return indexInfo.value?.stats?.indices?.[name]?.total || null
})
const indexAliases = computed(() => {
  if (!indexInfo.value?.info) {return []}
  const keys = Object.keys(indexInfo.value.info)
  const idx = keys[0] as string | undefined
  if (!idx) {return []}
  const aliases = indexInfo.value.info[idx]?.aliases || {}
  return Object.keys(aliases)
})

// ============ 初始化 ============
async function ensureConnected() {
  if (!props.connection) {return}
  try {
    // 深拷贝剥离 Vue Proxy，避免 IPC 克隆错误（同 RedisManager 做法）
    const connConfig = JSON.parse(JSON.stringify(props.connection))
    await getTauriAPI().dbConnect(connConfig)
  } catch (e) {
    console.warn('[EsManager] Failed to connect:', e)
  }
}

async function loadAll() {
  loading.value = true
  error.value = ''
  try {
    await ensureConnected()
    const [h, idx, n] = await Promise.all([
      getTauriAPI().esClusterHealth(props.connectionId),
      getTauriAPI().esListIndices(props.connectionId),
      getTauriAPI().esNodes(props.connectionId),
    ])
    if (h?.success === false) {throw new Error(h.error)}
    if (idx?.success === false) {throw new Error(idx.error)}
    if (n?.success === false) {throw new Error(n.error)}
    health.value = h
    indices.value = Array.isArray(idx) ? idx : []
    nodes.value = Array.isArray(n) ? n : []
    if (selectedIndexName.value) {
      await loadIndexDetail(selectedIndexName.value)
    }
  } catch (e: any) {
    error.value = e?.message || String(e)
  } finally {
    loading.value = false
  }
}

async function selectIndex(name: string | null) {
  selectedIndexName.value = name
  detailTab.value = 'overview'
  if (name) {
    await loadIndexDetail(name)
  }
}

async function loadIndexDetail(name: string) {
  error.value = ''
  try {
    const [info, mapping] = await Promise.all([
      getTauriAPI().esIndexInfo(props.connectionId, name),
      getTauriAPI().esIndexMapping(props.connectionId, name),
    ])
    if (info?.success === false) {throw new Error(info.error)}
    if (mapping?.success === false) {throw new Error(mapping.error)}
    indexInfo.value = info
    mappingFields.value = parseMapping(mapping)
    const st = indexSettings.value
    settingsText.value = JSON.stringify(st ?? {}, null, 2)
  } catch (e: any) {
    error.value = e?.message || String(e)
  }
}

function openDiscover(name: string) {
  selectedIndexName.value = name
  detailTab.value = 'discover'
  void loadIndexDetail(name)
}

// ============ mapping 字段解析 ============
function parseMapping(mapping: any): Array<{ name: string; type: string }> {
  const out: Array<{ name: string; type: string }> = []
  const idxKey = Object.keys(mapping || {})[0]
  const props = mapping?.[idxKey]?.mappings?.properties
  if (!props) {return out}
  const walk = (obj: any, prefix: string) => {
    for (const [name, def] of Object.entries<any>(obj)) {
      const full = prefix ? `${prefix}.${name}` : name
      const type = def?.type || (def?.properties ? 'object' : 'object')
      if (def?.properties) {
        walk(def.properties, full)
        continue
      }
      out.push({ name: full, type })
      if (type === 'text' && def?.fields?.keyword) {
        out.push({ name: `${full}.keyword`, type: 'keyword' })
      }
    }
  }
  walk(props, '')
  return out
}

function typeBadgeClass(type: string): string {
  switch (type) {
    case 'text': return 'badge-info'
    case 'keyword': return 'badge-primary'
    case 'long': case 'integer': case 'short': case 'byte': case 'double': case 'float': case 'half_float': case 'scaled_float': return 'badge-success'
    case 'date': return 'badge-warning'
    case 'boolean': return 'badge-secondary'
    default: return ''
  }
}

// ============ 工具 ============
function parseNum(v: any): number {
  const n = Number(v)
  return Number.isFinite(n) ? n : 0
}
function formatBytes(bytes: number): string {
  if (!bytes || bytes <= 0) {return '0 B'}
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1)
  return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`
}
function formatDocs(idx: any): string {
  const n = parseNum(idx['docs.count'])
  return n >= 1000 ? `${(n / 1000).toFixed(1)}k` : String(n)
}
function healthDotClassFor(h: string): string {
  switch (h) {
    case 'green': return 'bg-success'
    case 'yellow': return 'bg-warning'
    case 'red': return 'bg-error'
    default: return 'bg-base-content/30'
  }
}

// ============ 创建索引 ============
const showCreateIndex = ref(false)
const createIndexName = ref('')
const createSettingsText = ref('')
const createMappingsText = ref('')
const createError = ref('')
const creating = ref(false)

function openCreateIndex() {
  createIndexName.value = ''
  createSettingsText.value = ''
  createMappingsText.value = ''
  createError.value = ''
  showCreateIndex.value = true
}

async function doCreateIndex() {
  createError.value = ''
  const name = createIndexName.value.trim()
  if (!name) {createError.value = '索引名称不能为空'; return}
  const body: any = {}
  if (createSettingsText.value.trim()) {
    try { body.settings = JSON.parse(createSettingsText.value) } catch { createError.value = 'Settings 不是合法 JSON'; return }
  }
  if (createMappingsText.value.trim()) {
    try { body.mappings = JSON.parse(createMappingsText.value) } catch { createError.value = 'Mappings 不是合法 JSON'; return }
  }
  creating.value = true
  try {
    const res = await getTauriAPI().esCreateIndex(props.connectionId, name, Object.keys(body).length ? body : undefined)
    if (res?.success === false) {throw new Error(res.error)}
    if (res?.acknowledged === false) {throw new Error('ES 未确认创建')}
    showCreateIndex.value = false
    await loadAll()
  } catch (e: any) {
    createError.value = e?.message || String(e)
  } finally {
    creating.value = false
  }
}

// ============ 删除索引 ============
const showDeleteConfirm = ref(false)
const deleting = ref(false)
function confirmDeleteIndex() {
  showDeleteConfirm.value = true
}
async function doDeleteIndex() {
  if (!selectedIndexName.value) {return}
  deleting.value = true
  try {
    const res = await getTauriAPI().esDeleteIndex(props.connectionId, selectedIndexName.value)
    if (res?.success === false) {throw new Error(res.error)}
    showDeleteConfirm.value = false
    selectedIndexName.value = null
    await loadAll()
  } catch (e: any) {
    error.value = e?.message || String(e)
    showDeleteConfirm.value = false
  } finally {
    deleting.value = false
  }
}

// ============ Settings 保存 ============
async function saveSettings() {
  if (!selectedIndexName.value) {return}
  savingSettings.value = true
  error.value = ''
  try {
    const parsed = JSON.parse(settingsText.value)
    // 只发送可动态更新的设置（number_of_replicas 等），静态设置（number_of_shards）会导致 400
    const res = await getTauriAPI().esUpdateIndexSettings(props.connectionId, selectedIndexName.value, parsed)
    if (res?.success === false) {throw new Error(res.error)}
    await loadIndexDetail(selectedIndexName.value)
  } catch (e: any) {
    error.value = e?.message || String(e)
  } finally {
    savingSettings.value = false
  }
}

// ============ 别名 ============
const showAliases = ref(false)
const newAlias = ref('')
function openAliases() {
  newAlias.value = ''
  showAliases.value = true
}
async function addAlias() {
  const alias = newAlias.value.trim()
  if (!alias || !selectedIndexName.value) {return}
  const res = await getTauriAPI().esUpdateAliases(props.connectionId, [{ add: { index: selectedIndexName.value, alias } }])
  if (res?.success === false) {error.value = res.error; return}
  newAlias.value = ''
  await loadIndexDetail(selectedIndexName.value)
}
async function removeAlias(alias: string) {
  if (!selectedIndexName.value) {return}
  const res = await getTauriAPI().esUpdateAliases(props.connectionId, [{ remove: { index: selectedIndexName.value, alias } }])
  if (res?.success === false) {error.value = res.error; return}
  await loadIndexDetail(selectedIndexName.value)
}

// ============ Reindex ============
const showReindex = ref(false)
const reindexSource = ref('')
const reindexDest = ref('')
const reindexResult = ref('')
const reindexError = ref('')
const reindexing = ref(false)
function openReindex() {
  reindexSource.value = selectedIndexName.value || ''
  reindexDest.value = ''
  reindexResult.value = ''
  reindexError.value = ''
  showReindex.value = true
}
async function doReindex() {
  const src = reindexSource.value.trim()
  const dest = reindexDest.value.trim()
  reindexError.value = ''
  if (!src || !dest) {reindexError.value = '源索引和目标索引不能为空'; return}
  reindexing.value = true
  try {
    const res = await getTauriAPI().esReindex(props.connectionId, src, dest)
    if (res?.success === false) {throw new Error(res.error)}
    reindexResult.value = JSON.stringify({ took: res.took, created: res.created, updated: res.updated, batches: res.batches, failures: (res.failures || []).length }, null, 2)
  } catch (e: any) {
    reindexError.value = e?.message || String(e)
  } finally {
    reindexing.value = false
  }
}

// ============ 文档浏览（Discover） ============
const keyword = ref('')
const showDsl = ref(false)
const filters = ref<Array<{ field: string; op: string; value: string; value2: string }>>([])
const searching = ref(false)
const total = ref(0)
const hits = ref<any[]>([])
const page = ref(1)
const pageSize = 20

const totalPages = computed(() => Math.max(1, Math.ceil(total.value / pageSize)))

const OP_GROUPS: Record<string, Array<[string, string]>> = {
  text: [['contains', '包含 (match)'], ['not_contains', '不包含']],
  keyword: [['equals', '等于 (term)'], ['not_equals', '不等于']],
  numeric: [['equals', '等于'], ['gt', '>'], ['gte', '>='], ['lt', '<'], ['lte', '<=']],
  date: [['equals', '等于'], ['gt', '>'], ['gte', '>='], ['lt', '<'], ['lte', '<='], ['between', '区间']],
  boolean: [['is_true', '是 true'], ['is_false', '是 false']],
}

function fieldType(field: string): string {
  const f = mappingFields.value.find(x => x.name === field)
  const t = f?.type || 'text'
  if (['long', 'integer', 'short', 'byte', 'double', 'float', 'half_float', 'scaled_float'].includes(t)) {return 'numeric'}
  if (t === 'date') {return 'date'}
  if (t === 'boolean') {return 'boolean'}
  if (t === 'keyword') {return 'keyword'}
  return 'text'
}

function opsForField(field: string): Array<[string, string]> {
  return OP_GROUPS[fieldType(field)] || OP_GROUPS.text
}

function addFilter() {
  filters.value.push({ field: mappingFields.value[0]?.name || '', op: 'equals', value: '', value2: '' })
}

function resetFilters() {
  filters.value = []
  keyword.value = ''
  page.value = 1
  total.value = 0
  hits.value = []
}

function buildQuery(): any {
  const must: any[] = []
  const mustNot: any[] = []
  const filter: any[] = []
  for (const f of filters.value) {
    if (!f.field) {continue}
    const gt = fieldType(f.field)
    if (f.op === 'between') {
      if (!f.value && !f.value2) {continue}
      const range: any = {}
      if (f.value) {range.gte = f.value}
      if (f.value2) {range.lte = f.value2}
      filter.push({ range: { [f.field]: range } })
      continue
    }
    if (f.op === 'is_true' || f.op === 'is_false') {
      must.push({ term: { [f.field]: f.op === 'is_true' } })
      continue
    }
    if (!f.value && f.value !== '0') {continue}
    switch (f.op) {
      case 'contains': must.push({ match: { [f.field]: f.value } }); break
      case 'not_contains': mustNot.push({ match: { [f.field]: f.value } }); break
      case 'equals': must.push({ term: { [f.field]: f.value } }); break
      case 'not_equals': mustNot.push({ term: { [f.field]: f.value } }); break
      case 'gt': case 'gte': case 'lt': case 'lte':
        filter.push({ range: { [f.field]: { [f.op]: f.value } } }); break
    }
  }
  if (keyword.value.trim()) {
    must.push({ query_string: { query: keyword.value.trim() } })
  }
  if (must.length === 0 && mustNot.length === 0 && filter.length === 0) {
    return { match_all: {} }
  }
  const bool: any = {}
  if (must.length) {bool.must = must}
  if (mustNot.length) {bool.must_not = mustNot}
  if (filter.length) {bool.filter = filter}
  return { bool }
}

const dslPreview = computed(() => JSON.stringify(buildQuery(), null, 2))

const resultColumns = computed(() => {
  const cols: string[] = []
  for (const hit of hits.value.slice(0, 30)) {
    const src = hit._source || {}
    for (const k of Object.keys(src)) {
      if (!cols.includes(k)) {cols.push(k)}
      if (cols.length >= 8) {break}
    }
    if (cols.length >= 8) {break}
  }
  return cols
})

function cellText(v: any): string {
  if (v === null || v === undefined) {return ''}
  if (typeof v === 'object') {return JSON.stringify(v)}
  return String(v)
}

async function search() {
  if (!selectedIndexName.value) {return}
  searching.value = true
  error.value = ''
  try {
    const res = await getTauriAPI().esSearch(
      props.connectionId,
      selectedIndexName.value,
      { query: buildQuery(), track_total_hits: true },
      (page.value - 1) * pageSize,
      pageSize,
    )
    if (res?.success === false) {throw new Error(res.error)}
    total.value = Number(res?.hits?.total?.value ?? res?.hits?.total ?? 0)
    hits.value = Array.isArray(res?.hits?.hits) ? res.hits.hits : []
  } catch (e: any) {
    error.value = e?.message || String(e)
  } finally {
    searching.value = false
  }
}

// ============ 文档 CRUD ============
const showDocEdit = ref(false)
const docEditMode = ref<'create' | 'edit'>('create')
const editingDocId = ref('')
const docJson = ref('')
const docError = ref('')
const docSuccess = ref('')
const savingDoc = ref(false)

function openCreateDoc() {
  docEditMode.value = 'create'
  editingDocId.value = ''
  docJson.value = '{\n  \n}'
  docError.value = ''
  docSuccess.value = ''
  showDocEdit.value = true
}

async function openDocDetail(hit: any) {
  docEditMode.value = 'edit'
  editingDocId.value = hit._id
  docError.value = ''
  docSuccess.value = ''
  try {
    const res = await getTauriAPI().esGetDocument(props.connectionId, selectedIndexName.value!, hit._id)
    if (res?.success === false) {throw new Error(res.error)}
    docJson.value = JSON.stringify(res._source ?? {}, null, 2)
  } catch (e: any) {
    docJson.value = JSON.stringify(hit._source ?? {}, null, 2)
    docError.value = e?.message || String(e)
  }
  showDocEdit.value = true
}

async function saveDoc() {
  if (!selectedIndexName.value) {return}
  docError.value = ''
  docSuccess.value = ''
  let body: any
  try {
    body = JSON.parse(docJson.value)
  } catch {
    docError.value = '文档内容不是合法 JSON'
    return
  }
  savingDoc.value = true
  try {
    const res = await getTauriAPI().esIndexDocument(
      props.connectionId,
      selectedIndexName.value,
      docEditMode.value === 'create' ? (editingDocId.value.trim() || undefined) : editingDocId.value,
      body,
    )
    if (res?.success === false) {throw new Error(res.error)}
    docSuccess.value = `保存成功 result=${res.result} _id=${res._id}`
    if (docEditMode.value === 'create' && res._id) {
      editingDocId.value = res._id
      docEditMode.value = 'edit'
    }
    await search()
  } catch (e: any) {
    docError.value = e?.message || String(e)
  } finally {
    savingDoc.value = false
  }
}

async function deleteEditingDoc() {
  if (!selectedIndexName.value || !editingDocId.value) {return}
  // 项目约定：原生 confirm 在 Tauri 下不弹窗，用 plugin-dialog confirm
  const ok = await confirm(`确定删除文档 _id: ${editingDocId.value} 吗？该操作不可恢复。`, { title: '删除文档', kind: 'warning' })
  if (!ok) {return}
  docError.value = ''
  docSuccess.value = ''
  const res = await getTauriAPI().esDeleteDocument(props.connectionId, selectedIndexName.value, editingDocId.value)
  if (res?.success === false) {docError.value = res.error; return}
  docSuccess.value = `删除成功 result=${res.result}`
  showDocEdit.value = false
  // 末页删空后回退页码
  if (page.value > 1 && (page.value - 1) * pageSize >= total.value) {
    page.value = Math.max(1, page.value - 1)
  }
  await search()
}

onMounted(() => {
  void loadAll()
})
</script>

<style scoped>
:deep(.table) { font-size: 12px; }
</style>
