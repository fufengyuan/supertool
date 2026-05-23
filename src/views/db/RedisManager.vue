<template>
  <div class="flex flex-col h-full overflow-hidden">
    <!-- Search Bar -->
    <div class="flex flex-col gap-1.5 px-4 py-2.5 border-b border-base-content/10 bg-base-100">
      <div class="flex items-center gap-2">
        <div class="relative flex-1">
          <SvgIcon name="search" size="14" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-base-content/60 pointer-events-none" />
          <input
            v-model="searchPattern"
            @keydown.enter="loadKeys"
            class="w-full py-1.5 pl-8 pr-2.5 border border-base-content/20 rounded-md bg-base-200 text-base-content text-xs outline-none transition-[border-color] duration-150 ease-in-out focus:border-primary focus:ring-[3px] focus:ring-primary/10 placeholder:text-base-content/60 placeholder:opacity-50"
            placeholder="搜索键 (支持 * ? 通配符)"
            spellcheck="false"
          />
        </div>
        <button @click="loadKeys" class="btn btn-ghost btn-sm" :disabled="loading" title="刷新">
          <SvgIcon name="refresh" size="14" />
          刷新
        </button>
        <button @click="showAddKeyDialog = true" class="btn btn-primary btn-sm" title="添加键">
          <SvgIcon name="plus" size="14" />
          添加键
        </button>
      </div>
      <div class="text-xs text-base-content/60">
        <span v-if="totalKeys > 0">搜索到 {{ totalKeys }} 个键</span>
        <span v-if="selectedKey" class="text-primary ml-1">| 已选择: {{ selectedKey }}</span>
      </div>
    </div>

    <!-- Key Editor (full width — tree nav is in ConnectionTree) -->
    <div class="flex-1 flex flex-col overflow-hidden bg-base-200">
        <template v-if="!selectedKey">
          <div class="flex flex-col items-center justify-center flex-1 text-base-content/60 gap-3">
            <SvgIcon name="key" size="48" stroke-width="1.5" />
            <p>输入键名搜索，或从左侧树形结构中选择</p>
          </div>
        </template>
        <template v-else-if="keyLoading">
          <div class="flex flex-col items-center justify-center flex-1 text-base-content/60 gap-3">加载中...</div>
        </template>
        <template v-else-if="keyData">
          <!-- Key Info Bar -->
          <div class="flex items-center gap-2.5 px-4 py-2 border-b border-base-content/10 bg-base-100 flex-wrap">
            <span class="font-mono text-xs font-semibold text-base-content max-w-[480px] truncate" :title="selectedKey">{{ selectedKey }}</span>
            <span class="text-[11px] px-1.5 py-0.5 rounded bg-primary/10 text-primary font-medium">{{ typeLabel(keyInfo.type) }}</span>
            <span class="text-xs text-base-content/60">TTL: {{ formatTTL(keyInfo.ttl) }}</span>
            <span class="text-xs text-base-content/60">长度: {{ keyInfo.length }}</span>
            <div class="ml-auto flex gap-1.5">
              <button @click="saveKey" class="btn btn-primary btn-sm" :disabled="saving">
                <SvgIcon name="download" size="14" />  保存
              </button>
              <button @click="deleteSelectedKey" class="btn btn-error btn-sm" :disabled="deleting">
                <SvgIcon name="trash" size="14" />  删除
              </button>
            </div>
          </div>

      <!-- Value Editors by Type -->
      <!-- String -->
      <div v-if="keyInfo.type === 'string'" class="flex-1 overflow-auto p-3">
        <textarea
          v-model="stringValue"
          class="w-full h-full min-h-[200px] p-3 border border-base-content/20 rounded-lg bg-base-200 text-base-content font-mono text-xs leading-relaxed resize-y outline-none focus:border-primary focus:ring-[3px] focus:ring-primary/10"
          placeholder="字符串值..."
          spellcheck="false"
        />
      </div>

      <!-- Hash -->
      <div v-else-if="keyInfo.type === 'hash'" class="flex-1 overflow-auto p-3">
        <div class="flex flex-col gap-1">
          <table class="table table-xs w-full">
            <thead>
              <tr>
                <th class="text-left px-3 py-2 text-xs font-semibold text-base-content/60 border-b-2 border-base-content/10">字段</th>
                <th class="text-left px-3 py-2 text-xs font-semibold text-base-content/60 border-b-2 border-base-content/10">值</th>
                <th class="text-left px-3 py-2 text-xs font-semibold text-base-content/60 border-b-2 border-base-content/10"></th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(item, idx) in hashValue" :key="idx">
                <td class="px-2 py-1 border-b border-base-content/10">
                  <input v-model="item.field" class="w-full px-2 py-1.5 border border-base-content/20 rounded bg-base-200 text-base-content text-xs font-mono outline-none focus:border-primary" placeholder="field" spellcheck="false" />
                </td>
                <td class="px-2 py-1 border-b border-base-content/10">
                  <input v-model="item.value" class="w-full px-2 py-1.5 border border-base-content/20 rounded bg-base-200 text-base-content text-xs font-mono outline-none focus:border-primary" placeholder="value" spellcheck="false" />
                </td>
                <td class="px-2 py-1 border-b border-base-content/10">
                  <button @click="removeHashRow(idx)" class="btn btn-ghost btn-xs btn-square" title="删除行">
                    <SvgIcon name="x" size="14" />
                  </button>
                </td>
              </tr>
            </tbody>
          </table>
          <button @click="addHashRow" class="btn btn-ghost btn-sm mt-2">+ 添加字段</button>
        </div>
      </div>

      <!-- List -->
      <div v-else-if="keyInfo.type === 'list'" class="flex-1 overflow-auto p-3">
        <div class="grid gap-1 grid-cols-[40px_1fr_30px]">
          <div class="flex items-center justify-center text-xs text-base-content/60 p-1.5 font-mono">#</div>
          <div class="p-0.5 text-xs text-base-content/60">值</div>
          <div class="flex items-center justify-center"></div>
          <template v-for="(item, idx) in listValue" :key="idx">
            <div class="flex items-center justify-center text-xs text-base-content/60 p-1.5 font-mono">{{ idx }}</div>
            <div class="p-0.5">
              <input v-model="listValue[idx]" class="w-full px-2 py-1.5 border border-base-content/20 rounded bg-base-200 text-base-content text-xs font-mono outline-none focus:border-primary" placeholder="值" spellcheck="false" />
            </div>
            <div class="flex items-center justify-center">
              <button @click="removeListItem(idx)" class="btn btn-ghost btn-xs btn-square" title="删除">
                <SvgIcon name="x" size="14" />
              </button>
            </div>
          </template>
          <button @click="addListItem" class="btn btn-ghost btn-sm mt-2 col-span-full">+ 添加项</button>
        </div>
      </div>

      <!-- Set -->
      <div v-else-if="keyInfo.type === 'set'" class="flex-1 overflow-auto p-3">
        <div class="grid gap-1 grid-cols-[40px_1fr_30px]">
          <div class="flex items-center justify-center text-xs text-base-content/60 p-1.5 font-mono">#</div>
          <div class="p-0.5 text-xs text-base-content/60">成员</div>
          <div class="flex items-center justify-center"></div>
          <template v-for="(item, idx) in setValue" :key="idx">
            <div class="flex items-center justify-center text-xs text-base-content/60 p-1.5 font-mono">{{ idx }}</div>
            <div class="p-0.5">
              <input v-model="setValue[idx]" class="w-full px-2 py-1.5 border border-base-content/20 rounded bg-base-200 text-base-content text-xs font-mono outline-none focus:border-primary" placeholder="成员" spellcheck="false" />
            </div>
            <div class="flex items-center justify-center">
              <button @click="removeSetItem(idx)" class="btn btn-ghost btn-xs btn-square" title="删除">
                <SvgIcon name="x" size="14" />
              </button>
            </div>
          </template>
          <button @click="addSetItem" class="btn btn-ghost btn-sm mt-2 col-span-full">+ 添加成员</button>
        </div>
      </div>

      <!-- ZSet -->
      <div v-else-if="keyInfo.type === 'zset'" class="flex-1 overflow-auto p-3">
        <div class="grid gap-1 grid-cols-[40px_1fr_100px_30px]">
          <div class="flex items-center justify-center text-xs text-base-content/60 p-1.5 font-mono">#</div>
          <div class="p-0.5 text-xs text-base-content/60">成员</div>
          <div class="p-0.5 text-xs text-base-content/60">分数</div>
          <div class="flex items-center justify-center"></div>
          <template v-for="(item, idx) in zsetValue" :key="idx">
            <div class="flex items-center justify-center text-xs text-base-content/60 p-1.5 font-mono">{{ idx }}</div>
            <div class="p-0.5">
              <input v-model="item.member" class="w-full px-2 py-1.5 border border-base-content/20 rounded bg-base-200 text-base-content text-xs font-mono outline-none focus:border-primary" placeholder="member" spellcheck="false" />
            </div>
            <div class="p-0.5">
              <input v-model.number="item.score" class="w-full px-2 py-1.5 border border-base-content/20 rounded bg-base-200 text-base-content text-xs font-mono outline-none focus:border-primary" type="number" step="any" placeholder="score" />
            </div>
            <div class="flex items-center justify-center">
              <button @click="removeZSetItem(idx)" class="btn btn-ghost btn-xs btn-square" title="删除">
                <SvgIcon name="x" size="14" />
              </button>
            </div>
          </template>
          <button @click="addZSetItem" class="btn btn-ghost btn-sm mt-2 col-span-full">+ 添加成员</button>
        </div>
      </div>

      <!-- Stream -->
      <div v-else-if="keyInfo.type === 'stream'" class="flex-1 overflow-auto p-3">
        <div class="grid grid-cols-[220px_1fr] gap-0.5">
          <div class="flex items-center px-2 py-1.5 text-xs text-primary font-mono font-semibold bg-base-200 border border-base-content/20 rounded">Entry ID</div>
          <div class="px-2 py-1.5 bg-base-200 border border-base-content/20 rounded text-xs font-semibold text-base-content/60">Fields</div>
          <template v-for="(entry, idx) in streamValue" :key="entry.id || idx">
            <div class="flex items-center px-2 py-1.5 text-xs text-primary font-mono font-semibold bg-base-200 border border-base-content/20 rounded">{{ entry.id }}</div>
            <div class="px-2 py-1.5 bg-base-200 border border-base-content/20 rounded">
              <div class="flex gap-1.5 py-0.5 border-b border-base-content/10 last:border-b-0" v-for="(val, field) in entry.fields" :key="field">
                <span class="text-xs text-base-content/60 font-mono font-medium shrink-0 min-w-[80px]">{{ field }}:</span>
                <span class="text-xs text-base-content font-mono break-all">{{ val }}</span>
              </div>
            </div>
          </template>
        </div>
      </div>
    </template>
    </div>

    <!-- Bottom: Redis Console -->
    <div class="h-[180px] min-h-[100px] border-t-2 border-base-content/10 flex flex-col bg-base-100">
      <div class="flex items-center justify-between px-3 py-1.5 border-b border-base-content/10">
        <span class="text-xs font-semibold text-base-content/60">🔴 Redis 控制台</span>
        <button @click="consoleMessages = []" class="btn btn-ghost btn-xs" title="清空">清空</button>
      </div>
      <div class="flex-1 overflow-y-auto px-3 py-2 font-mono text-xs leading-relaxed" ref="consoleOutputRef">
        <div v-for="(msg, idx) in consoleMessages" :key="idx" class="py-0.5" :class="{'text-primary': msg.type === 'input', 'text-base-content': msg.type === 'output', 'text-error': msg.type === 'error'}">
          <span class="font-semibold">{{ msg.prefix }}</span>
          <span class="break-all">{{ msg.content }}</span>
        </div>
        <div v-if="consoleMessages.length === 0" class="text-base-content/60 text-center py-6 italic">
          输入 Redis 命令，例如: GET key, KEYS *, INFO
        </div>
      </div>
      <div class="flex items-center gap-2 px-3 py-2 border-t border-base-content/10">
        <span class="font-mono text-sm font-bold text-primary">&gt;</span>
        <input
          v-model="consoleCommand"
          @keydown.enter="executeConsole"
          class="flex-1 px-2.5 py-1.5 border border-base-content/20 rounded-md bg-base-200 text-base-content font-mono text-xs outline-none transition-[border-color] duration-150 ease-in-out focus:border-primary focus:ring-[3px] focus:ring-primary/10 placeholder:text-base-content/60 placeholder:opacity-50"
          placeholder="输入 Redis 命令..."
          spellcheck="false"
        />
        <button @click="executeConsole" class="btn btn-primary btn-sm" :disabled="consoleExecuting">
          执行
        </button>
      </div>
    </div>

    <!-- Add Key Dialog -->
    <div v-if="showAddKeyDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div class="bg-base-100 rounded-xl p-5 w-[600px] max-w-[90vw] shadow-lg relative">
        <button @click="showAddKeyDialog = false" class="absolute top-3 right-3 btn btn-ghost btn-sm btn-square rounded-full" title="关闭">
          <SvgIcon name="x" size="16" />
        </button>
        <h3 class="text-base font-semibold text-base-content m-0 mb-4">添加新键</h3>
        <div class="flex flex-col gap-3">
          <div class="grid grid-cols-2 gap-3">
            <div class="flex flex-col gap-1">
              <label class="text-xs font-medium text-base-content/60">键名</label>
              <input v-model="newKey.name" class="px-2.5 py-2 border border-base-content/20 rounded-md bg-base-200 text-base-content text-xs outline-none focus:border-primary" placeholder="例如: mykey" spellcheck="false" />
            </div>
            <div class="flex flex-col gap-1">
              <label class="text-xs font-medium text-base-content/60">类型</label>
            <select v-model="newKey.type" class="px-2.5 py-2 border border-base-content/20 rounded-md bg-base-200 text-base-content text-xs outline-none focus:border-primary">
              <option value="string">String</option>
              <option value="hash">Hash</option>
              <option value="list">List</option>
              <option value="set">Set</option>
              <option value="zset">ZSet</option>
            </select>
            </div>
          </div>
          <div class="flex flex-col gap-1" v-if="newKey.type === 'string'">
            <label class="text-xs font-medium text-base-content/60">值</label>
            <textarea v-model="newKey.value" class="px-2.5 py-2 border border-base-content/20 rounded-md bg-base-200 text-base-content text-xs min-h-[80px] resize-y outline-none focus:border-primary" placeholder="字符串值"></textarea>
          </div>
          <div class="flex flex-col gap-1" v-if="newKey.type === 'hash'">
            <label class="text-xs font-medium text-base-content/60">字段</label>
            <input v-model="newKey.field" class="px-2.5 py-2 border border-base-content/20 rounded-md bg-base-200 text-base-content text-xs outline-none focus:border-primary" placeholder="field" />
            <label class="text-xs font-medium text-base-content/60 mt-2">值</label>
            <input v-model="newKey.value" class="px-2.5 py-2 border border-base-content/20 rounded-md bg-base-200 text-base-content text-xs outline-none focus:border-primary" placeholder="value" />
          </div>
          <div class="flex flex-col gap-1" v-if="newKey.type === 'list' || newKey.type === 'set'">
            <label class="text-xs font-medium text-base-content/60">初始值</label>
            <input v-model="newKey.value" class="px-2.5 py-2 border border-base-content/20 rounded-md bg-base-200 text-base-content text-xs outline-none focus:border-primary" placeholder="值" />
          </div>
          <div v-if="newKey.type === 'zset'" class="grid grid-cols-2 gap-3">
            <div class="flex flex-col gap-1">
              <label class="text-xs font-medium text-base-content/60">成员</label>
              <input v-model="newKey.member" class="px-2.5 py-2 border border-base-content/20 rounded-md bg-base-200 text-base-content text-xs outline-none focus:border-primary" placeholder="member" />
            </div>
            <div class="flex flex-col gap-1">
              <label class="text-xs font-medium text-base-content/60">分数</label>
              <input v-model.number="newKey.score" class="px-2.5 py-2 border border-base-content/20 rounded-md bg-base-200 text-base-content text-xs outline-none focus:border-primary" type="number" step="any" placeholder="0" />
            </div>
          </div>
        </div>
        <div class="flex justify-end gap-2 mt-4">
          <button @click="showAddKeyDialog = false" class="btn btn-ghost">取消</button>
          <button @click="addNewKey" class="btn btn-primary">添加</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import * as logger from '../../services/logger'
import { getTauriAPI } from '../../utils/tauri-api'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref, computed, watch, nextTick, onMounted } from 'vue'
import { useToast } from '../../composables/useToast'

const props = defineProps<{
  connectionId: string
  connectionName: string
  connection?: DBConnection  // Full config for dbConnect
  initialKey?: string        // Key to auto-select on mount
  redisDbIndex?: number      // Redis database index to use
}>()

interface DBConnection {
  id: string
  name: string
  type: string
  host: string
  port: number
  user?: string
  password?: string
  database?: string
  dbIndex?: number
}

const toast = useToast()

// Key list state - grouped by type
const searchPattern = ref('*')
const keysByType = ref<Record<string, string[]>>({})
const totalKeys = ref(0)
const loading = ref(false)

// Selected key state
const selectedKey = ref<string | null>(null)
const keyLoading = ref(false)
const keyInfo = ref<{ type: string; ttl: number; length: number }>({ type: '', ttl: -1, length: 0 })
const saving = ref(false)
const deleting = ref(false)

// Value states
const stringValue = ref('')
const hashValue = ref<Array<{ field: string; value: string }>>([])
const listValue = ref<string[]>([])
const setValue = ref<string[]>([])
const zsetValue = ref<Array<{ member: string; score: number }>>([])
const streamValue = ref<Array<{ id: string; fields: Record<string, string> }>>([])

// Console state
const consoleCommand = ref('')
const consoleExecuting = ref(false)
const consoleMessages = ref<Array<{ type: string; prefix: string; content: string }>>([])
const consoleOutputRef = ref<HTMLDivElement | null>(null)

// Add key dialog
const showAddKeyDialog = ref(false)
const newKey = ref({
  name: '',
  type: 'string',
  value: '',
  field: '',
  member: '',
  score: 0
})

const keyData = computed(() => {
  if (!keyInfo.value.type || keyInfo.value.type === 'none') {return null}
  return true
})

function typeLabel(type: string): string {
  switch (type) {
    case 'string': return 'String'
    case 'hash': return 'Hash'
    case 'list': return 'List'
    case 'set': return 'Set'
    case 'zset': return 'ZSet'
    default: return type
  }
}

function formatTTL(ttl: number): string {
  if (ttl === -1) {return '∞'}
  if (ttl === -2) {return '已过期'}
  if (ttl < 60) {return `${ttl}s`}
  if (ttl < 3600) {return `${Math.floor(ttl / 60)}m ${ttl % 60}s`}
  return `${Math.floor(ttl / 3600)}h ${Math.floor((ttl % 3600) / 60)}m`
}

// Ensure connection is established before making Redis API calls
async function ensureConnected() {
  if (props.connection) {
    try {
      // If a specific dbIndex is provided, update the connection config
      const connConfig = JSON.parse(JSON.stringify(props.connection))
      if (props.redisDbIndex !== undefined) {
        connConfig.dbIndex = props.redisDbIndex
      }
      await getTauriAPI().dbConnect(connConfig)
    } catch (e) {
      console.warn('[RedisManager] Failed to connect:', e)
    }
  }
}

// Load keys scoped to current db index
async function loadKeys() {
  loading.value = true
  try {
    await ensureConnected()
    const dbIndex = props.redisDbIndex ?? 0
    logger.info(`[RedisManager] loadKeys: connectionId=${props.connectionId}, dbIndex=${dbIndex}, pattern=${searchPattern.value}`)
    const result = await getTauriAPI().dbRedisKeysByType(props.connectionId, dbIndex, searchPattern.value)
    logger.info(`[RedisManager] loadKeys result:`, result?.success ? `types=${JSON.stringify(Object.fromEntries(Object.entries(result.keysByType || {}).map(([k,v]) => [k, (v as any[]).length])))}` : result?.error)
    if (result) {
      keysByType.value = result.keysByType || {}
      totalKeys.value = Object.values(keysByType.value).reduce((sum, arr) => sum + (arr as any[]).length, 0)
    } else {
      toast.error(result?.error || '加载键列表失败')
    }
  } catch (e: any) {
    toast.error('加载键列表失败: ' + (e?.message || '未知错误'))
  } finally {
    loading.value = false
  }
}

// Select key
async function selectKey(key: string) {
  selectedKey.value = key
  keyLoading.value = true
  try {
    await ensureConnected()
    // Get key info
    const info = await getTauriAPI().dbRedisKeyInfo(props.connectionId, props.dbIndex, key)
    logger.info('[RedisManager] keyInfo result:', JSON.stringify(info))
    if (info?.success) {
      keyInfo.value = { type: info.type || '', ttl: info.ttl ?? -1, length: info.length ?? 0 }
      logger.info('[RedisManager] keyInfo set to:', JSON.stringify(keyInfo.value))
    } else {
      console.error('[RedisManager] keyInfo failed:', info?.error)
      toast.error('获取键信息失败: ' + (info?.error || '未知错误'))
      return
    }

    // Get key value
    const valResult = await getTauriAPI().dbRedisKeyValue(props.connectionId, props.dbIndex, key)
    logger.info(`[RedisManager] keyValue result: ${valResult?.success ? '(type=' + keyInfo.value.type + ', length=' + keyInfo.value.length + ')' : valResult?.error}`)
    if (valResult?.success) {
      const val = valResult.value as any
      switch (keyInfo.value.type) {
        case 'string':
          stringValue.value = typeof val === 'object' ? JSON.stringify(val, null, 2) : (val ?? '')
          break
        case 'hash':
          hashValue.value = val ? Object.entries(val).map(([field, value]) => ({ field, value: String(value) })) : []
          break
        case 'list':
          listValue.value = val ? val.map((v: any) => String(v)) : []
          break
        case 'set':
          setValue.value = val ? val.map((v: any) => String(v)) : []
          break
        case 'zset':
          zsetValue.value = val ? [...val] : []
          break
        case 'stream':
          streamValue.value = val ? val : []
          break
      }
      logger.info(`[RedisManager] value loaded, type: ${keyInfo.value.type}`)
    } else {
      console.error('[RedisManager] keyValue failed:', valResult?.error)
      toast.error('获取键值失败: ' + (valResult?.error || '未知错误'))
    }
  } catch (e: any) {
    console.error('[RedisManager] selectKey error:', e)
    toast.error('加载键值失败: ' + (e?.message || '未知错误'))
  } finally {
    keyLoading.value = false
  }
}

// Save key
async function saveKey() {
  if (!selectedKey.value) {return}
  saving.value = true
  try {
    let value: any
    switch (keyInfo.value.type) {
      case 'string':
        value = stringValue.value
        break
      case 'hash':
        value = {}
        for (const item of hashValue.value) {
          if (item.field) {value[item.field] = item.value}
        }
        break
      case 'list':
        value = listValue.value
        break
      case 'set':
        value = setValue.value
        break
      case 'zset':
        value = zsetValue.value
        break
    }

    // ⚠️ 剥离 Vue Proxy，否则 Tauri IPC 的 structuredClone 会失败
    const plainValue = JSON.parse(JSON.stringify(value))
    const result = await getTauriAPI().dbRedisSetKey(props.connectionId, selectedKey.value, keyInfo.value.type, plainValue)
    if (result) {
      toast.info('键已保存')
      // Reload key info
      await selectKey(selectedKey.value)
    } else {
      toast.error('保存失败')
    }
  } catch (e: any) {
    toast.error('保存失败: ' + (e?.message || '未知错误'))
  } finally {
    saving.value = false
  }
}

// Delete selected key
async function deleteSelectedKey() {
  if (!selectedKey.value) {return}
  if (!confirm(`确定要删除键 "${selectedKey.value}" 吗？`)) {return}

  deleting.value = true
  try {
    const result = await getTauriAPI().dbRedisDeleteKey(props.connectionId, props.dbIndex, selectedKey.value)
    if (result) {
      toast.info('键已删除')
      selectedKey.value = null
      keyInfo.value = { type: '', ttl: -1, length: 0 }
      await loadKeys()
    } else {
      toast.error('删除失败')
    }
  } catch (e: any) {
    toast.error('删除失败: ' + (e?.message || '未知错误'))
  } finally {
    deleting.value = false
  }
}

// Hash operations
function addHashRow() {
  hashValue.value.push({ field: '', value: '' })
}
function removeHashRow(idx: number) {
  hashValue.value.splice(idx, 1)
}

// List operations
function addListItem() {
  listValue.value.push('')
}
function removeListItem(idx: number) {
  listValue.value.splice(idx, 1)
}

// Set operations
function addSetItem() {
  setValue.value.push('')
}
function removeSetItem(idx: number) {
  setValue.value.splice(idx, 1)
}

// ZSet operations
function addZSetItem() {
  zsetValue.value.push({ member: '', score: 0 })
}
function removeZSetItem(idx: number) {
  zsetValue.value.splice(idx, 1)
}

// Add new key
async function addNewKey() {
  if (!newKey.value.name.trim()) {
    toast.info('请输入键名')
    return
  }
  try {
    let value: any
    switch (newKey.value.type) {
      case 'string':
        value = newKey.value.value
        break
      case 'hash':
        value = { field: newKey.value.field, value: newKey.value.value }
        break
      case 'list':
      case 'set':
        value = newKey.value.value
        break
      case 'zset':
        value = { member: newKey.value.member, score: newKey.value.score }
        break
    }

    const plainValue = JSON.parse(JSON.stringify(value))
    const result = await getTauriAPI().dbRedisAddKey(
      props.connectionId,
      props.dbIndex,
      newKey.value.type,
      newKey.value.name,
      plainValue
    )
    if (result) {
      const createdName = newKey.value.name
      toast.info('键已添加')
      showAddKeyDialog.value = false
      newKey.value = { name: '', type: 'string', value: '', field: '', member: '', score: 0 }
      await loadKeys()
      await selectKey(createdName)
    } else {
      toast.error('添加失败')
    }
  } catch (e: any) {
    toast.error('添加失败: ' + (e?.message || '未知错误'))
  }
}

// Console
async function executeConsole() {
  if (!consoleCommand.value.trim()) {return}

  const cmd = consoleCommand.value.trim()
  consoleMessages.value.push({ type: 'input', prefix: '> ', content: cmd })
  consoleCommand.value = ''
  consoleExecuting.value = true

  try {
    const result = await getTauriAPI().dbRedisExec(props.connectionId, props.dbIndex, cmd)
    if (result) {
      const output = typeof result.result === 'object' ? JSON.stringify(result.result, null, 2) : String(result.result ?? '')
      consoleMessages.value.push({ type: 'output', prefix: '', content: output })
    } else {
      consoleMessages.value.push({ type: 'error', prefix: 'ERR ', content: result?.error || '命令执行失败' })
    }
  } catch (e: any) {
    consoleMessages.value.push({ type: 'error', prefix: 'ERR ', content: e?.message || '命令执行失败' })
  } finally {
    consoleExecuting.value = false
    nextTick(() => {
      if (consoleOutputRef.value) {
        consoleOutputRef.value.scrollTop = consoleOutputRef.value.scrollHeight
      }
    })
  }
}

onMounted(async () => {
  // Don't auto-load all keys on mount — user can search explicitly
  // Auto-select the initial key if provided
  if (props.initialKey) {
    await selectKey(props.initialKey)
  }
})

// Watch for initialKey changes (e.g., when navigating between keys from the tree)
watch(() => props.initialKey, async (newKey) => {
  logger.info(`[RedisManager] watch initialKey triggered: ${newKey}, selectedKey: ${selectedKey.value}`)
  if (newKey && newKey !== selectedKey.value) {
    logger.info('[RedisManager] calling selectKey from watch:', newKey)
    await selectKey(newKey)
  } else {
    logger.info('[RedisManager] skipping selectKey from watch (same key or empty)')
  }
})

// Watch for dbIndex changes — reload keys when user switches database
watch(() => props.redisDbIndex, async (newIdx, oldIdx) => {
  logger.info(`[RedisManager] watch redisDbIndex: ${oldIdx} -> ${newIdx}`)
  if (newIdx !== undefined && newIdx !== oldIdx) {
    selectedKey.value = ''
    await loadKeys()
  }
})
</script>
