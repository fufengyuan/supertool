<template>
  <div class="redis-manager">
    <!-- Search Bar -->
    <div class="redis-topbar">
      <div class="search-row">
        <div class="search-input-wrap">
          <svg class="search-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="11" cy="11" r="8" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
          </svg>
          <input
            v-model="searchPattern"
            @keydown.enter="loadKeys"
            class="search-input"
            placeholder="搜索键 (支持 * ? 通配符)"
            spellcheck="false"
          />
        </div>
        <button @click="loadKeys" class="btn btn-ghost btn-sm" :disabled="loading" title="刷新">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="23 4 23 10 17 10" />
            <polyline points="1 20 1 14 7 14" />
            <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
          </svg>
          刷新
        </button>
        <button @click="showAddKeyDialog = true" class="btn btn-primary btn-sm" title="添加键">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19" />
            <line x1="5" y1="12" x2="19" y2="12" />
          </svg>
          添加键
        </button>
      </div>
      <div class="stats-row">
        <span class="stats-text" v-if="totalKeys > 0">搜索到 {{ totalKeys }} 个键</span>
        <span v-if="selectedKey" class="selected-info">| 已选择: {{ selectedKey }}</span>
      </div>
    </div>

    <!-- Key Editor (full width — tree nav is in ConnectionTree) -->
    <div class="value-editor-panel">
        <template v-if="!selectedKey">
          <div class="editor-empty">
            <svg viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4" />
            </svg>
            <p>输入键名搜索，或从左侧树形结构中选择</p>
          </div>
        </template>
        <template v-else-if="keyLoading">
          <div class="editor-loading">加载中...</div>
        </template>
        <template v-else-if="keyData">
          <!-- Key Info Bar -->
          <div class="key-info-bar">
            <span class="key-info-name" :title="selectedKey">{{ selectedKey }}</span>
            <span class="key-info-type" :class="'type-' + keyInfo.type">{{ typeLabel(keyInfo.type) }}</span>
            <span class="key-info-ttl">TTL: {{ formatTTL(keyInfo.ttl) }}</span>
            <span class="key-info-length">长度: {{ keyInfo.length }}</span>
            <div class="key-info-actions">
              <button @click="saveKey" class="btn btn-primary btn-sm" :disabled="saving">
                💾 保存
              </button>
              <button @click="deleteSelectedKey" class="btn btn-danger btn-sm" :disabled="deleting">
                🗑️ 删除
              </button>
            </div>
          </div>

      <!-- Value Editors by Type -->
      <!-- String -->
      <div v-if="keyInfo.type === 'string'" class="value-editor">
        <textarea
          v-model="stringValue"
          class="value-textarea"
          placeholder="字符串值..."
          spellcheck="false"
        />
      </div>

      <!-- Hash -->
      <div v-else-if="keyInfo.type === 'hash'" class="value-editor">
        <div class="hash-table-wrap">
          <table class="hash-table">
            <thead>
              <tr>
                <th class="hash-field">字段</th>
                <th class="hash-value">值</th>
                <th class="hash-actions"></th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(item, idx) in hashValue" :key="idx">
                <td>
                  <input v-model="item.field" class="hash-input" placeholder="field" spellcheck="false" />
                </td>
                <td>
                  <input v-model="item.value" class="hash-input" placeholder="value" spellcheck="false" />
                </td>
                <td>
                  <button @click="removeHashRow(idx)" class="btn-icon" title="删除行">
                    <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                      <line x1="18" y1="6" x2="6" y2="18" />
                      <line x1="6" y1="6" x2="18" y2="18" />
                    </svg>
                  </button>
                </td>
              </tr>
            </tbody>
          </table>
          <button @click="addHashRow" class="btn btn-ghost btn-sm add-row-btn">+ 添加字段</button>
        </div>
      </div>

      <!-- List -->
      <div v-else-if="keyInfo.type === 'list'" class="value-editor">
        <div class="list-items-wrap">
          <div class="list-index">#</div>
          <div class="list-item-value">值</div>
          <div class="list-item-actions"></div>
          <template v-for="(item, idx) in listValue" :key="idx">
            <div class="list-index">{{ idx }}</div>
            <div class="list-item-value">
              <input v-model="listValue[idx]" class="list-input" placeholder="值" spellcheck="false" />
            </div>
            <div class="list-item-actions">
              <button @click="removeListItem(idx)" class="btn-icon" title="删除">
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                  <line x1="18" y1="6" x2="6" y2="18" />
                  <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
              </button>
            </div>
          </template>
          <button @click="addListItem" class="btn btn-ghost btn-sm add-row-btn" style="grid-column: 1 / -1;">+ 添加项</button>
        </div>
      </div>

      <!-- Set -->
      <div v-else-if="keyInfo.type === 'set'" class="value-editor">
        <div class="set-items-wrap">
          <div class="set-index">#</div>
          <div class="set-item-value">成员</div>
          <div class="set-item-actions"></div>
          <template v-for="(item, idx) in setValue" :key="idx">
            <div class="set-index">{{ idx }}</div>
            <div class="set-item-value">
              <input v-model="setValue[idx]" class="set-input" placeholder="成员" spellcheck="false" />
            </div>
            <div class="set-item-actions">
              <button @click="removeSetItem(idx)" class="btn-icon" title="删除">
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                  <line x1="18" y1="6" x2="6" y2="18" />
                  <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
              </button>
            </div>
          </template>
          <button @click="addSetItem" class="btn btn-ghost btn-sm add-row-btn" style="grid-column: 1 / -1;">+ 添加成员</button>
        </div>
      </div>

      <!-- ZSet -->
      <div v-else-if="keyInfo.type === 'zset'" class="value-editor">
        <div class="zset-items-wrap">
          <div class="zset-index">#</div>
          <div class="zset-item-member">成员</div>
          <div class="zset-item-score">分数</div>
          <div class="zset-item-actions"></div>
          <template v-for="(item, idx) in zsetValue" :key="idx">
            <div class="zset-index">{{ idx }}</div>
            <div class="zset-item-member">
              <input v-model="item.member" class="zset-input" placeholder="member" spellcheck="false" />
            </div>
            <div class="zset-item-score">
              <input v-model.number="item.score" class="zset-input zset-score" type="number" step="any" placeholder="score" />
            </div>
            <div class="zset-item-actions">
              <button @click="removeZSetItem(idx)" class="btn-icon" title="删除">
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                  <line x1="18" y1="6" x2="6" y2="18" />
                  <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
              </button>
            </div>
          </template>
          <button @click="addZSetItem" class="btn btn-ghost btn-sm add-row-btn" style="grid-column: 1 / -1;">+ 添加成员</button>
        </div>
      </div>

      <!-- Stream -->
      <div v-else-if="keyInfo.type === 'stream'" class="value-editor">
        <div class="stream-entries-wrap">
          <div class="stream-entry-id">Entry ID</div>
          <div class="stream-entry-fields">Fields</div>
          <template v-for="(entry, idx) in streamValue" :key="entry.id || idx">
            <div class="stream-entry-id">{{ entry.id }}</div>
            <div class="stream-entry-fields">
              <div class="stream-field-row" v-for="(val, field) in entry.fields" :key="field">
                <span class="stream-field-name">{{ field }}:</span>
                <span class="stream-field-value">{{ val }}</span>
              </div>
            </div>
          </template>
        </div>
      </div>
    </template>
    </div>

    <!-- Bottom: Redis Console -->
    <div class="redis-console">
      <div class="console-header">
        <span class="console-title">🔴 Redis 控制台</span>
        <button @click="consoleMessages = []" class="btn btn-ghost btn-xs" title="清空">清空</button>
      </div>
      <div class="console-output" ref="consoleOutputRef">
        <div v-for="(msg, idx) in consoleMessages" :key="idx" class="console-msg" :class="msg.type">
          <span class="console-msg-prefix">{{ msg.prefix }}</span>
          <span class="console-msg-content">{{ msg.content }}</span>
        </div>
        <div v-if="consoleMessages.length === 0" class="console-empty">
          输入 Redis 命令，例如: GET key, KEYS *, INFO
        </div>
      </div>
      <div class="console-input-row">
        <span class="console-prompt">&gt;</span>
        <input
          v-model="consoleCommand"
          @keydown.enter="executeConsole"
          class="console-input"
          placeholder="输入 Redis 命令..."
          spellcheck="false"
        />
        <button @click="executeConsole" class="btn btn-primary btn-sm" :disabled="consoleExecuting">
          执行
        </button>
      </div>
    </div>

    <!-- Add Key Dialog -->
    <div v-if="showAddKeyDialog" class="modal-overlay" @click.self="showAddKeyDialog = false">
      <div class="modal-dialog">
        <h3 class="modal-title">添加新键</h3>
        <div class="modal-body">
          <div class="form-row">
            <div class="form-group">
              <label>键名</label>
              <input v-model="newKey.name" class="form-input" placeholder="例如: mykey" spellcheck="false" />
            </div>
            <div class="form-group">
              <label>类型</label>
            <select v-model="newKey.type" class="form-select">
              <option value="string">String</option>
              <option value="hash">Hash</option>
              <option value="list">List</option>
              <option value="set">Set</option>
              <option value="zset">ZSet</option>
            </select>
            </div>
          </div>
          <div class="form-group" v-if="newKey.type === 'string'">
            <label>值</label>
            <textarea v-model="newKey.value" class="form-textarea" placeholder="字符串值"></textarea>
          </div>
          <div class="form-group" v-if="newKey.type === 'hash'">
            <label>字段</label>
            <input v-model="newKey.field" class="form-input" placeholder="field" />
            <label style="margin-top: 8px;">值</label>
            <input v-model="newKey.value" class="form-input" placeholder="value" />
          </div>
          <div class="form-group" v-if="newKey.type === 'list' || newKey.type === 'set'">
            <label>初始值</label>
            <input v-model="newKey.value" class="form-input" placeholder="值" />
          </div>
          <div v-if="newKey.type === 'zset'" class="form-row">
            <div class="form-group">
              <label>成员</label>
              <input v-model="newKey.member" class="form-input" placeholder="member" />
            </div>
            <div class="form-group">
              <label>分数</label>
              <input v-model.number="newKey.score" class="form-input" type="number" step="any" placeholder="0" />
            </div>
          </div>
        </div>
        <div class="modal-footer">
          <button @click="showAddKeyDialog = false" class="btn btn-ghost">取消</button>
          <button @click="addNewKey" class="btn btn-primary">添加</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { getTauriAPI } from '@/utils/tauri-api'
import * as logger from '@/services/logger'
import { ref, computed, watch, nextTick, onMounted } from 'vue'
import { useToast } from '@/composables/useToast'

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
  if (!keyInfo.value.type || keyInfo.value.type === 'none') return null
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
  if (ttl === -1) return '∞'
  if (ttl === -2) return '已过期'
  if (ttl < 60) return `${ttl}s`
  if (ttl < 3600) return `${Math.floor(ttl / 60)}m ${ttl % 60}s`
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
  if (!selectedKey.value) return
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
          if (item.field) value[item.field] = item.value
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
  if (!selectedKey.value) return
  if (!confirm(`确定要删除键 "${selectedKey.value}" 吗？`)) return

  deleting.value = true
  try {
    const result = await getTauriAPI().dbRedisDeleteKey(props.connectionId, selectedKey.value)
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
  if (!consoleCommand.value.trim()) return

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

<style scoped>
.redis-manager {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

/* Top Bar */
.redis-topbar {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 10px 16px;
  border-bottom: 1px solid var(--border-color);
  background: var(--card-bg);
}

.search-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.search-input-wrap {
  position: relative;
  flex: 1;
}

.search-icon {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--main-text-secondary);
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: 6px 10px 6px 32px;
  border: 1.5px solid var(--input-border);
  border-radius: 6px;
  background: var(--input-bg);
  color: var(--main-text);
  font-size: 13px;
  outline: none;
  transition: border-color 0.15s ease;
}

.search-input:focus {
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px var(--primary-light);
}

.search-input::placeholder {
  color: var(--main-text-secondary);
  opacity: 0.5;
}

.stats-row {
  font-size: 12px;
  color: var(--main-text-secondary);
}

.selected-info {
  color: var(--primary-color);
  margin-left: 4px;
}

/* Value Editor Panel */
.value-editor-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--main-bg);
}

.editor-empty,
.editor-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  color: var(--main-text-secondary);
  gap: 12px;
}

.key-info-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 16px;
  border-bottom: 1px solid var(--border-color);
  background: var(--card-bg);
  flex-wrap: wrap;
}

.key-info-name {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  font-weight: 600;
  color: var(--main-text);
  max-width: 480px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.key-info-type {
  font-size: 11px;
  padding: 2px 6px;
  border-radius: 3px;
  background: var(--primary-light);
  color: var(--primary-color);
  font-weight: 500;
}

.key-info-ttl,
.key-info-length {
  font-size: 12px;
  color: var(--main-text-secondary);
}

.key-info-actions {
  margin-left: auto;
  display: flex;
  gap: 6px;
}

/* Value Editor */
.value-editor {
  flex: 1;
  overflow: auto;
  padding: 12px;
}

.value-textarea {
  width: 100%;
  height: 100%;
  min-height: 200px;
  padding: 12px;
  border: 1.5px solid var(--input-border);
  border-radius: 8px;
  background: var(--input-bg);
  color: var(--main-text);
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  line-height: 1.6;
  resize: vertical;
  outline: none;
}

.value-textarea:focus {
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px var(--primary-light);
}

/* Hash Table */
.hash-table-wrap {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.hash-table {
  width: 100%;
  border-collapse: collapse;
}

.hash-table th {
  text-align: left;
  padding: 8px 12px;
  font-size: 12px;
  font-weight: 600;
  color: var(--main-text-secondary);
  border-bottom: 2px solid var(--border-color);
}

.hash-table td {
  padding: 4px 8px;
  border-bottom: 1px solid var(--border-color);
}

.hash-input {
  width: 100%;
  padding: 6px 8px;
  border: 1px solid var(--input-border);
  border-radius: 4px;
  background: var(--input-bg);
  color: var(--main-text);
  font-size: 12px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  outline: none;
}

.hash-input:focus {
  border-color: var(--primary-color);
}

/* List Items */
.list-items-wrap,
.set-items-wrap,
.zset-items-wrap {
  display: grid;
  gap: 4px;
}

.list-items-wrap {
  grid-template-columns: 40px 1fr 30px;
}

.set-items-wrap {
  grid-template-columns: 40px 1fr 30px;
}

.zset-items-wrap {
  grid-template-columns: 40px 1fr 100px 30px;
}

.list-index,
.set-index,
.zset-index {
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  color: var(--main-text-secondary);
  padding: 6px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}

.list-item-value,
.set-item-value,
.zset-item-member,
.zset-item-score {
  padding: 2px;
}

.list-item-actions,
.set-item-actions,
.zset-item-actions {
  display: flex;
  align-items: center;
  justify-content: center;
}

.list-input,
.set-input,
.zset-input {
  width: 100%;
  padding: 6px 8px;
  border: 1px solid var(--input-border);
  border-radius: 4px;
  background: var(--input-bg);
  color: var(--main-text);
  font-size: 12px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  outline: none;
}

.list-input:focus,
.set-input:focus,
.zset-input:focus {
  border-color: var(--primary-color);
}

.zset-score {
  width: 100%;
}

.stream-entries-wrap {
  display: grid;
  grid-template-columns: 220px 1fr;
  gap: 2px;
}

.stream-entry-id {
  display: flex;
  align-items: center;
  padding: 6px 8px;
  font-size: 12px;
  color: var(--primary-color);
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-weight: 600;
  background: var(--input-bg);
  border: 1px solid var(--input-border);
  border-radius: 4px;
}

.stream-entry-fields {
  padding: 6px 8px;
  background: var(--input-bg);
  border: 1px solid var(--input-border);
  border-radius: 4px;
}

.stream-field-row {
  display: flex;
  gap: 6px;
  padding: 3px 0;
  border-bottom: 1px solid var(--border-color);
}

.stream-field-row:last-child {
  border-bottom: none;
}

.stream-field-name {
  font-size: 12px;
  color: var(--main-text-secondary);
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-weight: 500;
  flex-shrink: 0;
  min-width: 80px;
}

.stream-field-value {
  font-size: 12px;
  color: var(--main-text);
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  word-break: break-all;
}

.add-row-btn {
  margin-top: 8px;
}

/* Console */
.redis-console {
  height: 180px;
  min-height: 100px;
  border-top: 2px solid var(--border-color);
  display: flex;
  flex-direction: column;
  background: var(--card-bg);
}

.console-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 12px;
  border-bottom: 1px solid var(--border-color);
}

.console-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--main-text-secondary);
}

.console-output {
  flex: 1;
  overflow-y: auto;
  padding: 8px 12px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
  line-height: 1.6;
}

.console-msg {
  padding: 2px 0;
}

.console-msg.input {
  color: var(--primary-color);
}

.console-msg.output {
  color: var(--main-text);
}

.console-msg.error {
  color: var(--danger-color);
}

.console-msg-prefix {
  font-weight: 600;
}

.console-msg-content {
  word-break: break-all;
}

.console-empty {
  color: var(--main-text-secondary);
  text-align: center;
  padding: 24px;
  font-style: italic;
}

.console-input-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-top: 1px solid var(--border-color);
}

.console-prompt {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 14px;
  font-weight: 700;
  color: var(--primary-color);
}

.console-input {
  flex: 1;
  padding: 6px 10px;
  border: 1.5px solid var(--input-border);
  border-radius: 6px;
  background: var(--input-bg);
  color: var(--main-text);
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
  outline: none;
  transition: border-color 0.15s ease;
}

.console-input:focus {
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px var(--primary-light);
}

.console-input::placeholder {
  color: var(--main-text-secondary);
  opacity: 0.5;
}

/* Modal */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-dialog {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 20px;
  width: 600px;
  max-width: 90vw;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
}

.modal-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--main-text);
  margin: 0 0 16px 0;
}

.modal-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.form-group label {
  font-size: 12px;
  font-weight: 500;
  color: var(--main-text-secondary);
}

.form-input,
.form-select {
  padding: 8px 10px;
  border: 1.5px solid var(--input-border);
  border-radius: 6px;
  background: var(--input-bg);
  color: var(--main-text);
  font-size: 13px;
  outline: none;
}

.form-input:focus,
.form-select:focus {
  border-color: var(--primary-color);
}

.form-textarea {
  padding: 8px 10px;
  border: 1.5px solid var(--input-border);
  border-radius: 6px;
  background: var(--input-bg);
  color: var(--main-text);
  font-size: 13px;
  min-height: 80px;
  resize: vertical;
  outline: none;
}

.form-textarea:focus {
  border-color: var(--primary-color);
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 16px;
}

/* Buttons */
.btn-sm {
  padding: 4px 10px;
  font-size: 12px;
  line-height: 1;
  border-radius: 4px;
}

.btn-xs {
  padding: 2px 6px;
  font-size: 11px;
  line-height: 1;
  border-radius: 3px;
}

.btn-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  background: transparent;
  border-radius: 4px;
  cursor: pointer;
  color: var(--main-text-secondary);
  transition: all 0.1s;
}

.btn-icon:hover {
  background: var(--danger-light);
  color: var(--danger-color);
}

.btn-danger {
  background: var(--danger-color);
  color: white;
}

.btn-danger:hover {
  opacity: 0.9;
}

.btn-danger:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
