<template>
  <div class="api-debugger">
    <!-- Left Sidebar: Saved Requests -->
    <aside class="sidebar">
      <div class="sidebar-header">
        <h3>📁 已保存接口</h3>
        <button class="btn-new" @click="createNewRequest">+ 新建</button>
      </div>
      <div class="sidebar-search">
        <input v-model="searchQuery" placeholder="搜索接口..." class="search-input" />
      </div>
      <div class="sidebar-list">
        <div
          v-for="req in filteredRequests"
          :key="req.id"
          class="request-item"
          :class="{ active: currentRequestId === req.id }"
          @click="loadRequest(req)"
        >
          <span class="method-badge" :class="req.method.toLowerCase()">{{ req.method }}</span>
          <span class="request-name">{{ req.name || '未命名' }}</span>
          <button class="btn-del" @click.stop="deleteRequest(req.id)" title="删除">×</button>
        </div>
        <div v-if="filteredRequests.length === 0" class="empty-hint">
          {{ searchQuery ? '无匹配结果' : '点击 + 新建接口' }}
        </div>
      </div>
    </aside>

    <!-- Main Area -->
    <main class="main-area">
      <!-- Smart Paste Area -->
      <div class="smart-paste" :class="{ collapsed: !pasteCollapsed }">
        <div class="paste-header" @click="pasteCollapsed = !pasteCollapsed">
          <span class="paste-title">📋 智能粘贴报文</span>
          <span class="paste-arrow">{{ pasteCollapsed ? '▼' : '▲' }}</span>
        </div>
        <div v-show="!pasteCollapsed" class="paste-content">
          <textarea
            v-model="pasteText"
            placeholder="粘贴 curl 命令、HTTP 请求报文或纯 URL，自动解析..."
            class="paste-textarea"
            rows="6"
          />
          <div class="paste-actions">
            <button class="btn-parse" @click="parseSmartPaste">🔍 智能解析</button>
            <button class="btn-clear-paste" @click="pasteText = ''">清空</button>
          </div>
          <div v-if="parseResult" class="parse-result">
            ✅ 已解析: <strong>{{ parseResult.method }}</strong> {{ parseResult.url }}
            <span v-if="parseResult.headersCount"> ({{ parseResult.headersCount }} 个请求头)</span>
            <span v-if="parseResult.bodyType"> 报文: {{ parseResult.bodyType }}</span>
          </div>
        </div>
      </div>

      <!-- Request Bar: method + URL + send + save -->
      <div class="request-bar">
        <select v-model="request.method" class="method-select">
          <option>GET</option>
          <option>POST</option>
          <option>PUT</option>
          <option>DELETE</option>
          <option>PATCH</option>
          <option>HEAD</option>
          <option>OPTIONS</option>
        </select>
        <input
          v-model="request.url"
          class="url-input"
          placeholder="输入请求 URL，如 https://api.example.com/users"
          @keydown.enter="sendRequest"
        />
        <button
          class="btn-save"
          :class="{ saved: isSaved }"
          :title="isSaved ? '已保存 (Ctrl+S)' : '保存当前请求 (Ctrl+S)'"
          @click="saveCurrentRequest"
        >
          {{ isSaved ? '✓ 已保存' : '💾 保存' }}
        </button>
        <button
          class="btn-send"
          :class="{ loading: isSending }"
          :disabled="isSending || !request.url"
          @click="sendRequest"
        >
          {{ isSending ? '⏳' : '▶ 发送' }}
        </button>
      </div>

      <!-- Request Name -->
      <div class="request-name-row">
        <label>名称：</label>
        <input
          v-model="request.name"
          class="name-input"
          placeholder="给接口取个名字"
          @keydown.enter="saveCurrentRequest"
        />
        <span class="save-status" :class="{ visible: saveFeedbackVisible }">{{ saveFeedbackText }}</span>
      </div>

      <!-- Request Tabs -->
      <div class="request-tabs">
        <div class="tabs-header">
          <button
            v-for="tab in requestTabs"
            :key="tab.key"
            class="tab-btn"
            :class="{ active: activeRequestTab === tab.key }"
            @click="activeRequestTab = tab.key"
          >
            {{ tab.label }}
            <span v-if="tab.key === 'headers'" class="tab-count">{{ parsedHeaders.length }}</span>
          </button>
        </div>

        <!-- Headers Tab -->
        <div v-show="activeRequestTab === 'headers'" class="tab-content">
          <div class="headers-table">
            <div class="header-row header-labels">
              <input class="header-input" placeholder="Key" disabled />
              <input class="header-input" placeholder="Value" disabled />
              <div class="header-del"></div>
            </div>
            <div
              v-for="(h, idx) in parsedHeaders"
              :key="idx"
              class="header-row"
            >
              <input
                :value="h.key"
                class="header-input"
                placeholder="Header 名称"
                @input="updateHeader(idx, 'key', ($event.target as HTMLInputElement).value)"
                @change="onHeadersChange"
              />
              <input
                :value="h.value"
                class="header-input"
                placeholder="Header 值"
                @input="updateHeader(idx, 'value', ($event.target as HTMLInputElement).value)"
                @change="onHeadersChange"
              />
              <button class="btn-del-row" @click="removeHeader(idx)" title="删除">×</button>
            </div>
            <button class="btn-add-header" @click="addHeader">+ 添加请求头</button>
          </div>
          <!-- Quick Headers -->
          <div class="quick-headers">
            <span class="quick-label">快速添加：</span>
            <button
              v-for="qh in quickHeaders"
              :key="qh.key"
              class="btn-quick"
              :class="{ added: hasHeader(qh.key) }"
              @click="toggleQuickHeader(qh)"
            >
              {{ qh.label }}
            </button>
          </div>
        </div>

        <!-- Body Tab -->
        <div v-show="activeRequestTab === 'body'" class="tab-content">
          <div class="body-type-selector">
            <label v-for="bt in bodyTypes" :key="bt.key" class="body-type-radio">
              <input
                type="radio"
                name="bodyType"
                :value="bt.key"
                v-model="request.contentType"
              />
              <span>{{ bt.label }}</span>
            </label>
          </div>
          <textarea
            v-show="request.contentType !== 'none'"
            v-model="request.body"
            class="body-textarea"
            :placeholder="bodyPlaceholder"
            rows="12"
            spellcheck="false"
          />
          <div v-if="request.contentType === 'json'" class="body-actions">
            <button class="btn-sm" @click="formatJsonBody">📐 格式化</button>
            <button class="btn-sm" @click="compressJsonBody">📦 压缩</button>
          </div>
        </div>
      </div>

      <!-- Response Panel -->
      <div class="response-panel" v-if="response">
        <div class="response-header">
          <h4>📨 响应结果</h4>
          <div class="response-stats">
            <span class="status-badge" :class="responseStatusClass">
              {{ response.status }} {{ response.statusText }}
            </span>
            <span class="stat-item">⏱️ {{ response.time }}ms</span>
            <span class="stat-item">📦 {{ formatSize(response.size) }}</span>
          </div>
        </div>
        <div class="response-tabs">
          <button
            v-for="rt in responseTabs"
            :key="rt.key"
            class="tab-btn-sm"
            :class="{ active: activeResponseTab === rt.key }"
            @click="activeResponseTab = rt.key"
          >
            {{ rt.label }}
          </button>
        </div>
        <div class="response-content">
          <pre v-show="activeResponseTab === 'body'" class="response-body" :class="{ 'is-json': isJsonResponse }">{{ formatResponseBody }}</pre>
          <pre v-show="activeResponseTab === 'headers'" class="response-headers">{{ formatResponseHeaders }}</pre>
          <pre v-show="activeResponseTab === 'raw'" class="response-body">{{ response.rawBody }}</pre>
        </div>
      </div>

      <!-- Empty State -->
      <div v-else class="response-empty">
        <div class="response-empty-icon">🚀</div>
        <h3>准备发送请求</h3>
        <p>输入 URL 或粘贴报文后点击"发送"</p>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { getTauriAPI } from '../../../utils/tauri-api'

// ─── Types ───
interface SavedRequest {
  id: string
  name: string
  method: string
  url: string
  headers: string
  body: string | null
  contentType: string
  createdAt: string
  updatedAt: string
}

interface HeaderEntry {
  key: string
  value: string
}

interface HttpResponse {
  status: number
  statusText: string
  headers: Record<string, string>
  body: string
  rawBody: string
  time: number
  size: number
  error?: string
}

// ─── State ───
const searchQuery = ref('')
const savedRequests = ref<SavedRequest[]>([])
const currentRequestId = ref<string | null>(null)
const isSending = ref(false)
const response = ref<HttpResponse | null>(null)
const activeRequestTab = ref('body')
const activeResponseTab = ref('body')
const pasteText = ref('')
const pasteCollapsed = ref(true)
const parseResult = ref<{ method: string; url: string; headersCount: number; bodyType: string } | null>(null)
const isSaved = ref(false)
const saveFeedbackText = ref('')
const saveFeedbackVisible = ref(false)
let saveFeedbackTimer: ReturnType<typeof setTimeout> | null = null

const request = ref({
  name: '',
  method: 'GET',
  url: '',
  headers: '[]',
  body: '',
  contentType: 'none',
})

// ─── Computed ───
const parsedHeaders = computed({
  get: (): HeaderEntry[] => {
    try {
      const arr = JSON.parse(request.value.headers)
      if (!Array.isArray(arr)) return []
      // 显示所有行（包括空行），不在此处过滤
      return arr as HeaderEntry[]
    } catch { return [] }
  },
  set: (val: HeaderEntry[]) => {
    // 保存时过滤掉 key 和 value 都为空的行
    request.value.headers = JSON.stringify(val.filter(h => h.key || h.value))
  },
})

const filteredRequests = computed(() => {
  if (!searchQuery.value) return savedRequests.value
  const q = searchQuery.value.toLowerCase()
  return savedRequests.value.filter(r =>
    r.name.toLowerCase().includes(q) ||
    r.url.toLowerCase().includes(q) ||
    r.method.toLowerCase().includes(q)
  )
})

const requestTabs = [
  { key: 'headers', label: 'Headers' },
  { key: 'body', label: 'Body' },
]

const bodyTypes = [
  { key: 'none', label: '无' },
  { key: 'json', label: 'JSON' },
  { key: 'xml', label: 'XML' },
  { key: 'form-data', label: 'Form-Data' },
  { key: 'x-www-form-urlencoded', label: 'x-www-form-urlencoded' },
  { key: 'text', label: 'Text' },
]

const bodyPlaceholder = computed(() => {
  const map: Record<string, string> = {
    json: '{\n  "key": "value"\n}',
    xml: '<?xml version="1.0"?>\n<root>\n  <key>value</key>\n</root>',
    'form-data': 'key1=value1\nkey2=value2',
    'x-www-form-urlencoded': 'key1=value1&key2=value2',
    text: '输入文本内容...',
  }
  return map[request.value.contentType] || ''
})

const responseTabs = [
  { key: 'body', label: 'Body' },
  { key: 'headers', label: 'Headers' },
  { key: 'raw', label: 'Raw' },
]

const responseStatusClass = computed(() => {
  if (!response.value) return ''
  const s = response.value.status
  if (s >= 200 && s < 300) return 'success'
  if (s >= 300 && s < 400) return 'redirect'
  if (s >= 400 && s < 500) return 'client-error'
  if (s >= 500) return 'server-error'
  return ''
})

const isJsonResponse = computed(() => {
  if (!response.value) return false
  const ct = (response.value.headers['content-type'] || '').toLowerCase()
  return ct.includes('json') || ct.includes('application/hal+json')
})

const formatResponseBody = computed(() => {
  if (!response.value) return ''
  try {
    return JSON.stringify(JSON.parse(response.value.body), null, 2)
  } catch {
    return response.value.body
  }
})

const formatResponseHeaders = computed(() => {
  if (!response.value) return ''
  return JSON.stringify(response.value.headers, null, 2)
})

const quickHeaders = [
  { key: 'Authorization', label: 'Auth Token', valuePrefix: 'Bearer ' },
  { key: 'Content-Type', label: 'JSON', value: 'application/json' },
  { key: 'Accept', label: 'Accept JSON', value: 'application/json' },
  { key: 'X-Requested-With', label: 'AJAX', value: 'XMLHttpRequest' },
]

// ─── Methods ───
function addHeader() {
  // 直接操作 request.value.headers，不经过 computed setter 的 filter
  // getter 每次返回新数组(JSON.parse)，所以必须直接写入底层数据
  let headers: HeaderEntry[] = []
  try {
    headers = JSON.parse(request.value.headers)
    if (!Array.isArray(headers)) headers = []
  } catch { headers = [] }
  headers.push({ key: '', value: '' })
  request.value.headers = JSON.stringify(headers)
}

function removeHeader(idx: number) {
  parsedHeaders.value = parsedHeaders.value.filter((_, i) => i !== idx)
}

function updateHeader(idx: number, field: 'key' | 'value', val: string) {
  const headers = [...parsedHeaders.value]
  headers[idx] = { ...headers[idx], [field]: val }
  parsedHeaders.value = headers
}

function onHeadersChange() {
  saveCurrentRequest()
}

function hasHeader(key: string): boolean {
  return parsedHeaders.value.some(h => h.key.toLowerCase() === key.toLowerCase())
}

function toggleQuickHeader(qh: typeof quickHeaders[0]) {
  // 直接读写 request.value.headers，绕过 computed setter filter 和 getter 返回新数组的问题
  let headers: HeaderEntry[] = []
  try {
    headers = JSON.parse(request.value.headers)
    if (!Array.isArray(headers)) headers = []
  } catch { headers = [] }

  if (hasHeader(qh.key)) {
    headers = headers.filter(h => h.key.toLowerCase() !== qh.key.toLowerCase())
  } else {
    headers.push({
      key: qh.key,
      value: qh.value || qh.valuePrefix || '',
    })
  }
  request.value.headers = JSON.stringify(headers)
  onHeadersChange()
}

function formatJsonBody() {
  try {
    request.value.body = JSON.stringify(JSON.parse(request.value.body), null, 2)
  } catch { /* invalid json */ }
}

function compressJsonBody() {
  try {
    request.value.body = JSON.stringify(JSON.parse(request.value.body))
  } catch { /* invalid json */ }
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

async function loadRequest(req: SavedRequest) {
  currentRequestId.value = req.id
  isSaved.value = true
  request.value.name = req.name
  request.value.method = req.method
  request.value.url = req.url
  request.value.headers = req.headers
  request.value.body = req.body || ''
  request.value.contentType = req.contentType || 'none'
  response.value = null
}

function createNewRequest() {
  currentRequestId.value = null
  isSaved.value = false
  request.value = {
    name: '',
    method: 'GET',
    url: '',
    headers: '[]',
    body: '',
    contentType: 'none',
  }
  response.value = null
}

async function saveCurrentRequest() {
  if (!request.value.url) {
    showSaveFeedback('请先输入 URL', 'warn')
    return
  }
  const now = new Date().toISOString()
  try {
    if (currentRequestId.value) {
      // 更新已有接口
      await getTauriAPI().apiRequestsUpdate(currentRequestId.value, {
        name: request.value.name,
        method: request.value.method,
        url: request.value.url,
        headers: request.value.headers,
        body: request.value.body || null,
        contentType: request.value.contentType,
        updatedAt: now,
      })
      showSaveFeedback('✓ 已保存')
    } else {
      // 新建接口
      const insertResult: any = await getTauriAPI().apiRequestsAdd({
        name: request.value.name || extractUrlName(request.value.url),
        method: request.value.method,
        url: request.value.url,
        headers: request.value.headers,
        body: request.value.body || null,
        contentType: request.value.contentType,
      })
      // 直接从插入结果获取 ID（如果 API 返回了的话）
      if (insertResult?.id) {
        currentRequestId.value = insertResult.id
      } else {
        // fallback: 重新加载并匹配
        await loadRequests()
        const newReq = savedRequests.value.find(r => r.url === request.value.url && r.method === request.value.method)
        if (newReq) currentRequestId.value = newReq.id
      }
      showSaveFeedback('✓ 已保存为新接口')
    }
    isSaved.value = true
    await loadRequests()
  } catch (e: any) {
    console.error('[ApiDebugger] Save failed:', e.message)
    showSaveFeedback('保存失败: ' + e.message, 'error')
  }
}

function showSaveFeedback(text: string, type: 'success' | 'warn' | 'error' = 'success') {
  saveFeedbackText.value = text
  saveFeedbackVisible.value = true
  if (saveFeedbackTimer) clearTimeout(saveFeedbackTimer)
  saveFeedbackTimer = setTimeout(() => {
    saveFeedbackVisible.value = false
  }, 2500)
}

function extractUrlName(url: string): string {
  try {
    const u = new URL(url)
    const parts = u.pathname.split('/').filter(Boolean)
    return parts[parts.length - 1] || u.hostname
  } catch {
    return url.slice(0, 30)
  }
}

async function deleteRequest(id: string) {
  if (!confirm('确定删除此接口？')) return
  await getTauriAPI().apiRequestsDelete(id)
  if (currentRequestId.value === id) createNewRequest()
  await loadRequests()
}

async function loadRequests() {
  try {
    const result: any = await getTauriAPI().apiRequestsGetAll()
    savedRequests.value = (result.success && result.rows) ? result.rows : []
  } catch (e: any) {
    console.error('[ApiDebugger] Load failed:', e.message)
  }
}

async function sendRequest() {
  if (!request.value.url || isSending.value) return
  isSending.value = true
  response.value = null

  try {
    const headers: Record<string, string> = {}
    for (const h of parsedHeaders.value) {
      if (h.key && h.value) headers[h.key] = h.value
    }

    if (request.value.contentType === 'json' && request.value.body && !hasHeader('Content-Type')) {
      headers['Content-Type'] = 'application/json'
    }

    const result: any = await getTauriAPI().apiHttpRequest({
      method: request.value.method,
      url: request.value.url,
      headers,
      body: request.value.body || undefined,
      contentType: request.value.contentType !== 'none' ? request.value.contentType : undefined,
      timeout: 30000,
    })

    if (result.error) {
      response.value = {
        status: 0,
        statusText: 'Error',
        headers: {},
        body: result.error,
        rawBody: result.error,
        time: result.time || 0,
        size: 0,
        error: result.error,
      }
    } else {
      response.value = {
        status: result.status,
        statusText: result.statusText,
        headers: result.headers || {},
        body: result.body || '',
        rawBody: result.rawBody || result.body || '',
        time: result.time || 0,
        size: result.size || 0,
      }
    }

    await saveCurrentRequest()
  } catch (e: any) {
    response.value = {
      status: 0,
      statusText: 'Error',
      headers: {},
      body: e.message,
      rawBody: e.message,
      time: 0,
      size: 0,
      error: e.message,
    }
  } finally {
    isSending.value = false
  }
}

// ─── Smart Paste Parser ───
function parseSmartPaste() {
  if (!pasteText.value.trim()) return

  const text = pasteText.value.trim()
  let parsed: { method: string; url: string; headers: HeaderEntry[]; body: string | null; contentType: string } | null = null

  if (text.startsWith('curl')) {
    parsed = parseCurl(text)
  } else if (/^(GET|POST|PUT|DELETE|PATCH|HEAD|OPTIONS)\s+\S+\s+HTTP\/[\d.]+/im.test(text.split('\n')[0])) {
    parsed = parseHttpRequest(text)
  } else if (/^https?:\/\//.test(text)) {
    parsed = { method: 'GET', url: text, headers: [], body: null, contentType: 'none' }
  } else if (text.startsWith('{') || text.startsWith('[')) {
    parsed = {
      method: request.value.method || 'POST',
      url: request.value.url,
      headers: [],
      body: text,
      contentType: 'json',
    }
  }

  if (parsed) {
    request.value.method = parsed.method
    request.value.url = parsed.url
    if (parsed.headers.length > 0) {
      request.value.headers = JSON.stringify(parsed.headers)
    }
    if (parsed.body) {
      request.value.body = parsed.body
    }
    request.value.contentType = parsed.contentType

    parseResult.value = {
      method: parsed.method,
      url: parsed.url,
      headersCount: parsed.headers.length,
      bodyType: parsed.contentType !== 'none' ? parsed.contentType : 'none',
    }
  } else {
    parseResult.value = null
  }
}

function parseCurl(curl: string): { method: string; url: string; headers: HeaderEntry[]; body: string | null; contentType: string } {
  const result: { method: string; url: string; headers: HeaderEntry[]; body: string | null; contentType: string } = {
    method: 'GET', url: '', headers: [], body: null, contentType: 'none',
  }

  const methodMatch = curl.match(/-X\s+(GET|POST|PUT|DELETE|PATCH|HEAD|OPTIONS)/i)
  if (methodMatch) result.method = methodMatch[1].toUpperCase()

  const urlMatch = curl.match(/(?:curl\s+)?['"]?(https?:\/\/[^'"\s]+)['"]?/)
  if (urlMatch) result.url = urlMatch[1]

  const headerRegex = /-H\s+['"]([^'"]+)['"]/g
  let hMatch: RegExpExecArray | null
  while ((hMatch = headerRegex.exec(curl)) !== null) {
    const parts = hMatch[1].split(':')
    if (parts.length >= 2) {
      const key = parts[0].trim()
      const value = parts.slice(1).join(':').trim()
      result.headers.push({ key, value })
      if (key.toLowerCase() === 'content-type') {
        result.contentType = classifyContentType(value)
      }
    }
  }

  const dataMatch = curl.match(/(?:--data(?:-raw|-urlencode|-binary)?|-d)\s+['"]([^'"]*)['"]/)
  if (dataMatch) {
    result.body = dataMatch[1]
    if (result.contentType === 'none' && result.body.startsWith('{')) {
      result.contentType = 'json'
    }
  }

  return result
}

function parseHttpRequest(text: string): { method: string; url: string; headers: HeaderEntry[]; body: string | null; contentType: string } {
  const result: { method: string; url: string; headers: HeaderEntry[]; body: string | null; contentType: string } = {
    method: 'GET', url: '', headers: [], body: null, contentType: 'none',
  }

  const lines = text.split('\n')
  const firstLine = lines[0].trim()
  const reqMatch = firstLine.match(/^(GET|POST|PUT|DELETE|PATCH|HEAD|OPTIONS)\s+(\S+)/i)
  if (reqMatch) {
    result.method = reqMatch[1].toUpperCase()
    result.url = reqMatch[2]
  }

  let i = 1
  while (i < lines.length && lines[i].trim() !== '') {
    const colonIdx = lines[i].indexOf(':')
    if (colonIdx > 0) {
      const key = lines[i].substring(0, colonIdx).trim()
      const value = lines[i].substring(colonIdx + 1).trim()
      result.headers.push({ key, value })
      if (key.toLowerCase() === 'content-type') {
        result.contentType = classifyContentType(value)
      }
    }
    i++
  }

  if (i < lines.length) {
    result.body = lines.slice(i + 1).join('\n').trim()
    if (!result.body) result.body = null
  }

  if (result.url && !result.url.startsWith('http')) {
    result.url = 'http://' + result.url
  }

  return result
}

function classifyContentType(value: string): string {
  const v = value.toLowerCase()
  if (v.includes('json')) return 'json'
  if (v.includes('xml')) return 'xml'
  if (v.includes('multipart')) return 'form-data'
  if (v.includes('x-www-form-urlencoded')) return 'x-www-form-urlencoded'
  if (v.includes('text')) return 'text'
  return 'none'
}

// ─── Lifecycle ───
onMounted(async () => {
  await loadRequests()
  if (savedRequests.value.length > 0) {
    loadRequest(savedRequests.value[0])
  }

  // Ctrl+S / Cmd+S 快捷键保存
  document.addEventListener('keydown', (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 's') {
      e.preventDefault()
      saveCurrentRequest()
    }
  })
})
</script>

<style scoped>
.api-debugger {
  display: flex;
  height: 100%;
  overflow: hidden;
  background: var(--color-base-200);
}

/* ─── Sidebar ─── */
.sidebar {
  width: 260px;
  min-width: 220px;
  max-width: 320px;
  border-right: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  background: var(--color-base-100);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px 10px;
}

.sidebar-header h3 {
  margin: 0;
  font-size: 14px;
  font-weight: 700;
  color: var(--color-base-content);
}

.btn-new {
  padding: 4px 10px;
  font-size: 12px;
  border: 1px solid var(--color-primary);
  border-radius: 4px;
  background: transparent;
  color: var(--color-primary);
  cursor: pointer;
  transition: all 0.15s;
}

.btn-new:hover {
  background: var(--color-primary);
  color: white;
}

.sidebar-search {
  padding: 0 12px 10px;
}

.search-input {
  width: 100%;
  padding: 6px 10px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 4px;
  font-size: 12px;
  background: var(--color-base-200);
  color: var(--color-base-content);
  outline: none;
  box-sizing: border-box;
}

.search-input:focus {
  border-color: var(--color-primary);
}

.sidebar-list {
  flex: 1;
  overflow-y: auto;
  padding: 0 8px 12px;
}

.request-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 10px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.1s;
  font-size: 12px;
}

.request-item:hover {
  background: var(--color-base-200);
}

.request-item.active {
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
}

.method-badge {
  padding: 1px 5px;
  border-radius: 3px;
  font-size: 10px;
  font-weight: 700;
  flex-shrink: 0;
  min-width: 40px;
  text-align: center;
}

.method-badge.get { background: #d4edda; color: #155724; }
.method-badge.post { background: #cce5ff; color: #004085; }
.method-badge.put { background: #fff3cd; color: #856404; }
.method-badge.delete { background: #f8d7da; color: #721c24; }
.method-badge.patch { background: #e2e3f1; color: #383d5e; }
.method-badge.head { background: #e9ecef; color: #495057; }
.method-badge.options { background: #f0f0f0; color: #666; }

.request-name {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--color-base-content);
}

.btn-del {
  width: 18px;
  height: 18px;
  border: none;
  background: none;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  cursor: pointer;
  font-size: 14px;
  border-radius: 3px;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.1s;
}

.request-item:hover .btn-del {
  opacity: 1;
}

.btn-del:hover {
  background: #f8d7da;
  color: #721c24;
}

.empty-hint {
  padding: 16px;
  text-align: center;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-size: 12px;
}

/* ─── Main Area ─── */
.main-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  padding: 16px 20px;
  gap: 14px;
}

/* ─── Smart Paste ─── */
.smart-paste {
  border: 1px dashed color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 8px;
  background: var(--color-base-100);
  transition: border-color 0.2s;
}

.smart-paste:not(.collapsed) {
  border-color: var(--color-primary);
}

.paste-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 14px;
  cursor: pointer;
  user-select: none;
}

.paste-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-base-content);
}

.paste-arrow {
  font-size: 10px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.paste-content {
  padding: 0 14px 12px;
}

.paste-textarea {
  width: 100%;
  padding: 10px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 6px;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 12px;
  background: var(--color-base-200);
  color: var(--color-base-content);
  resize: vertical;
  outline: none;
  box-sizing: border-box;
}

.paste-textarea:focus {
  border-color: var(--color-primary);
}

.paste-actions {
  display: flex;
  gap: 8px;
  margin-top: 8px;
}

.btn-parse {
  padding: 6px 14px;
  font-size: 12px;
  border: 1px solid var(--color-primary);
  border-radius: 4px;
  background: transparent;
  color: var(--color-primary);
  cursor: pointer;
  transition: all 0.15s;
}

.btn-parse:hover {
  background: var(--color-primary);
  color: white;
}

.btn-clear-paste {
  padding: 6px 12px;
  font-size: 12px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 4px;
  background: transparent;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  cursor: pointer;
}

.parse-result {
  margin-top: 8px;
  padding: 6px 10px;
  font-size: 12px;
  background: #d4edda;
  color: #155724;
  border-radius: 4px;
}

/* ─── Request Bar ─── */
.request-bar {
  display: flex;
  gap: 8px;
  align-items: center;
}

.method-select {
  padding: 8px 10px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
  background: var(--color-base-200);
  color: var(--color-primary);
  cursor: pointer;
  outline: none;
}

.url-input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 6px;
  font-size: 13px;
  background: var(--color-base-200);
  color: var(--color-base-content);
  outline: none;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
}

.url-input:focus {
  border-color: var(--color-primary);
}

.btn-send {
  padding: 8px 18px;
  font-size: 13px;
  font-weight: 600;
  border: none;
  border-radius: 6px;
  background: var(--color-primary);
  color: white;
  cursor: pointer;
  transition: all 0.15s;
  white-space: nowrap;
}

.btn-send:hover:not(:disabled) {
  filter: brightness(1.1);
}

.btn-send:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* ─── Save Button ─── */
.btn-save {
  padding: 8px 14px;
  font-size: 12px;
  font-weight: 600;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 6px;
  background: var(--color-base-100);
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  cursor: pointer;
  transition: all 0.15s;
  white-space: nowrap;
}

.btn-save:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
}

.btn-save.saved {
  border-color: #28a745;
  color: #28a745;
  background: #d4edda;
}

/* ─── Save Status ─── */
.save-status {
  font-size: 11px;
  opacity: 0;
  transition: opacity 0.3s;
  color: #28a745;
  font-weight: 500;
}

.save-status.visible {
  opacity: 1;
}

/* ─── Request Name ─── */
.request-name-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.name-input {
  padding: 4px 8px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 4px;
  font-size: 12px;
  background: var(--color-base-200);
  color: var(--color-base-content);
  outline: none;
  width: 200px;
}

/* ─── Request Tabs ─── */
.request-tabs {
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 8px;
  overflow: hidden;
  background: var(--color-base-100);
}

.tabs-header {
  display: flex;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.tab-btn {
  padding: 8px 16px;
  border: none;
  background: none;
  font-size: 12px;
  font-weight: 500;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: all 0.15s;
}

.tab-btn:hover {
  color: var(--color-base-content);
}

.tab-btn.active {
  color: var(--color-primary);
  border-bottom-color: var(--color-primary);
}

.tab-count {
  font-size: 10px;
  background: color-mix(in oklab, var(--color-base-content) 10%, transparent);
  padding: 0 5px;
  border-radius: 8px;
  margin-left: 4px;
}

.tab-content {
  padding: 12px;
}

/* ─── Headers ─── */
.headers-table {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.header-row {
  display: flex;
  gap: 6px;
  align-items: center;
}

.header-labels input {
  font-weight: 600;
  font-size: 11px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  background: transparent;
}

.header-input {
  flex: 1;
  padding: 6px 8px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 4px;
  font-size: 12px;
  background: var(--color-base-200);
  color: var(--color-base-content);
  outline: none;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
}

.header-input:focus {
  border-color: var(--color-primary);
}

.header-del {
  width: 24px;
}

.btn-del-row {
  width: 24px;
  height: 28px;
  border: none;
  background: none;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  cursor: pointer;
  font-size: 16px;
  border-radius: 4px;
}

.btn-del-row:hover {
  background: #f8d7da;
  color: #721c24;
}

.btn-add-header {
  margin-top: 6px;
  padding: 5px 10px;
  border: 1px dashed color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 4px;
  background: none;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-size: 12px;
  cursor: pointer;
  align-self: flex-start;
}

.btn-add-header:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.quick-headers {
  margin-top: 12px;
  padding-top: 10px;
  border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.quick-label {
  font-size: 11px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  margin-right: 4px;
}

.btn-quick {
  padding: 3px 8px;
  font-size: 11px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 3px;
  background: transparent;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  cursor: pointer;
}

.btn-quick:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.btn-quick.added {
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
  border-color: var(--color-primary);
  color: var(--color-primary);
}

/* ─── Body ─── */
.body-type-selector {
  display: flex;
  gap: 12px;
  margin-bottom: 10px;
  flex-wrap: wrap;
}

.body-type-radio {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  cursor: pointer;
  color: var(--color-base-content);
}

.body-type-radio input {
  accent-color: var(--color-primary);
}

.body-textarea {
  width: 100%;
  padding: 10px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 6px;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 12px;
  background: var(--color-base-200);
  color: var(--color-base-content);
  resize: vertical;
  outline: none;
  box-sizing: border-box;
}

.body-textarea:focus {
  border-color: var(--color-primary);
}

.body-actions {
  display: flex;
  gap: 6px;
  margin-top: 8px;
}

.btn-sm {
  padding: 4px 10px;
  font-size: 11px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 3px;
  background: transparent;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  cursor: pointer;
}

.btn-sm:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

/* ─── Response Panel ─── */
.response-panel {
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 8px;
  overflow: hidden;
  background: var(--color-base-100);
}

.response-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.response-header h4 {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
}

.response-stats {
  display: flex;
  gap: 12px;
  align-items: center;
}

.status-badge {
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 700;
}

.status-badge.success { background: #d4edda; color: #155724; }
.status-badge.redirect { background: #cce5ff; color: #004085; }
.status-badge.client-error { background: #fff3cd; color: #856404; }
.status-badge.server-error { background: #f8d7da; color: #721c24; }

.stat-item {
  font-size: 11px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.response-tabs {
  display: flex;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.tab-btn-sm {
  padding: 6px 14px;
  border: none;
  background: none;
  font-size: 11px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  cursor: pointer;
  border-bottom: 2px solid transparent;
}

.tab-btn-sm:hover { color: var(--color-base-content); }

.tab-btn-sm.active {
  color: var(--color-primary);
  border-bottom-color: var(--color-primary);
}

.response-content {
  max-height: 400px;
  overflow: auto;
}

.response-body, .response-headers {
  margin: 0;
  padding: 12px 14px;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 12px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-all;
  color: var(--color-base-content);
}

.response-body.is-json {
  color: #e06c75;
}

/* ─── Empty State ─── */
.response-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  text-align: center;
  gap: 8px;
  flex: 1;
}

.response-empty-icon {
  font-size: 48px;
  opacity: 0.3;
}

.response-empty h3 {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-base-content);
  margin: 0;
}

.response-empty p {
  font-size: 12px;
  margin: 0;
}
</style>
