<template>
  <div class="flex h-full overflow-hidden bg-base-200">
    <!-- Left Sidebar: Saved Requests -->
    <aside class="w-[260px] min-w-[220px] max-w-[320px] border-r border-base-content/10 bg-base-100 flex flex-col shrink-0">
      <div class="flex items-center justify-between p-[14px_16px_10px]">
        <h3 class="m-0 text-sm font-bold text-base-content"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg> 已保存接口</h3>
        <button class="btn btn-outline btn-primary btn-xs" @click="createNewRequest">+ 新建</button>
      </div>
      <div class="px-3 pb-2.5">
        <input v-model="searchQuery" placeholder="搜索接口..." class="w-full px-2.5 py-1.5 border border-base-content/10 rounded text-xs bg-base-200 text-base-content outline-none box-border focus:border-primary" />
      </div>
      <div class="flex-1 overflow-y-auto px-2 pb-3">
        <div
          v-for="req in filteredRequests"
          :key="req.id"
          class="flex items-center gap-1.5 px-2.5 py-2 rounded cursor-pointer transition-all duration-100 text-xs group"
          :class="{ 'bg-primary/10': currentRequestId === req.id, 'hover:bg-base-200': true }"
          @click="loadRequest(req)"
        >
          <span class="px-[5px] py-[1px] rounded text-[10px] font-bold shrink-0 min-w-[40px] text-center" :class="req.method === 'GET' ? 'bg-green-100 text-green-800' : req.method === 'POST' ? 'bg-blue-100 text-blue-800' : req.method === 'PUT' ? 'bg-yellow-100 text-yellow-800' : req.method === 'DELETE' ? 'bg-red-100 text-red-700' : req.method === 'PATCH' ? 'bg-indigo-100 text-indigo-800' : req.method === 'HEAD' ? 'bg-gray-100 text-gray-700' : 'bg-gray-50 text-gray-600'">{{ req.method }}</span>
          <span class="flex-1 whitespace-nowrap overflow-hidden text-ellipsis text-base-content">{{ req.name || '未命名' }}</span>
          <button class="w-[18px] h-[18px] border-none bg-none text-base-content/60 cursor-pointer text-sm rounded flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity duration-100 hover:bg-red-100 hover:text-red-700" @click.stop="deleteRequest(req.id)" title="删除">×</button>
        </div>
        <div v-if="filteredRequests.length === 0" class="p-4 text-center text-base-content/60 text-xs">
          {{ searchQuery ? '无匹配结果' : '点击 + 新建接口' }}
        </div>
      </div>
    </aside>

    <!-- Main Area -->
    <main class="flex-1 flex flex-col overflow-y-auto p-[16px_20px] gap-3.5">
      <!-- Smart Paste Area -->
      <div class="border border-dashed border-base-content/10 rounded-lg bg-base-100 transition-colors duration-200" :class="{ 'border-primary': !pasteCollapsed }">
        <div class="flex items-center justify-between px-3.5 py-2 cursor-pointer select-none" @click="pasteCollapsed = !pasteCollapsed">
          <span class="text-xs font-semibold text-base-content"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg> 智能粘贴报文</span>
          <span class="text-[10px] text-base-content/60">{{ pasteCollapsed ? '▼' : '▲' }}</span>
        </div>
        <div v-show="!pasteCollapsed" class="px-3.5 pb-3">
          <textarea
            v-model="pasteText"
            placeholder="粘贴 curl 命令、HTTP 请求报文或纯 URL，自动解析..."
            class="w-full p-2.5 border border-base-content/10 rounded font-mono text-xs bg-base-200 text-base-content resize-y outline-none box-border focus:border-primary"
            rows="6"
          />
          <div class="flex gap-2 mt-2">
            <button class="btn btn-outline btn-primary btn-xs" @click="parseSmartPaste"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg> 智能解析</button>
            <button class="btn btn-ghost btn-xs" @click="pasteText = ''">清空</button>
          </div>
          <div v-if="parseResult" class="mt-2 px-2.5 py-1.5 text-xs bg-green-100 text-green-800 rounded">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" class="inline-block align-text-bottom"><polyline points="20 6 9 17 4 12"/></svg> 已解析: <strong>{{ parseResult.method }}</strong> {{ parseResult.url }}
            <span v-if="parseResult.headersCount"> ({{ parseResult.headersCount }} 个请求头)</span>
            <span v-if="parseResult.bodyType"> 报文: {{ parseResult.bodyType }}</span>
          </div>
        </div>
      </div>

      <!-- Request Bar: method + URL + send + save -->
      <div class="flex gap-2 items-center">
        <select v-model="request.method" class="px-2.5 py-2 border border-base-content/10 rounded text-xs font-semibold bg-base-200 text-primary cursor-pointer outline-none">
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
          class="flex-1 px-3 py-2 border border-base-content/10 rounded text-xs bg-base-200 text-base-content outline-none font-mono focus:border-primary"
          placeholder="输入请求 URL，如 https://api.example.com/users"
          @keydown.enter="sendRequest"
        />
        <button
          class="btn btn-ghost btn-sm text-xs font-semibold"
          :class="{ 'btn-success': isSaved }"
          :title="isSaved ? '已保存 (Ctrl+S)' : '保存当前请求 (Ctrl+S)'"
          @click="saveCurrentRequest"
        >
          <template v-if="isSaved">✓ 已保存</template><template v-else><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg> 保存</template>
        </button>
        <button
          class="btn btn-primary btn-sm min-h-0 h-auto px-[18px] py-2 text-xs font-semibold"
          :class="{ loading: isSending }"
          :disabled="isSending || !request.url"
          @click="sendRequest"
        >
          {{ isSending ? '' : '▶ 发送' }}
        </button>
      </div>

      <!-- Request Name -->
      <div class="flex items-center gap-2 text-xs text-base-content/60">
        <label>名称：</label>
        <input
          v-model="request.name"
          class="px-2 py-1 border border-base-content/10 rounded text-xs bg-base-200 text-base-content outline-none w-[200px]"
          placeholder="给接口取个名字"
          @keydown.enter="saveCurrentRequest"
        />
        <span class="text-[11px] opacity-0 transition-opacity duration-300 text-green-600 font-medium" :class="{ 'opacity-100': saveFeedbackVisible }">{{ saveFeedbackText }}</span>
      </div>

      <!-- Request Tabs -->
      <div class="tabs tabs-bordered">
        <button
          v-for="tab in requestTabs"
          :key="tab.key"
          class="tab"
          :class="{ 'tab-active': activeRequestTab === tab.key }"
          @click="activeRequestTab = tab.key"
        >
          {{ tab.label }}
          <span v-if="tab.key === 'headers'" class="badge badge-sm ml-1">{{ parsedHeaders.length }}</span>
        </button>

        <!-- Headers Tab -->
        <div v-show="activeRequestTab === 'headers'" class="border border-base-content/10 rounded-lg bg-base-100 p-3">
          <div class="flex flex-col gap-1.5">
            <div class="flex items-center gap-2">
              <input class="flex-1 px-2 py-1.5 border border-base-content/10 rounded text-xs bg-base-200 text-base-content outline-none" placeholder="Key" disabled />
              <input class="flex-1 px-2 py-1.5 border border-base-content/10 rounded text-xs bg-base-200 text-base-content outline-none" placeholder="Value" disabled />
              <div class="w-8 shrink-0"></div>
            </div>
            <div
              v-for="(h, idx) in parsedHeaders"
              :key="idx"
              class="flex items-center gap-2"
            >
              <input
                :value="h.key"
                class="flex-1 px-2 py-1.5 border border-base-content/10 rounded text-xs bg-base-200 text-base-content outline-none focus:border-primary"
                placeholder="Header 名称"
                @input="updateHeader(idx, 'key', ($event.target as HTMLInputElement).value)"
                @change="onHeadersChange"
              />
              <input
                :value="h.value"
                class="flex-1 px-2 py-1.5 border border-base-content/10 rounded text-xs bg-base-200 text-base-content outline-none focus:border-primary"
                placeholder="Header 值"
                @input="updateHeader(idx, 'value', ($event.target as HTMLInputElement).value)"
                @change="onHeadersChange"
              />
              <button class="btn btn-ghost btn-xs text-base-content/60 hover:text-red-500 shrink-0" @click="removeHeader(idx)" title="删除">×</button>
            </div>
            <button class="btn btn-ghost btn-xs text-primary justify-start pl-1" @click="addHeader">+ 添加请求头</button>
          </div>
          <!-- Quick Headers -->
          <div class="flex items-center gap-2 mt-3 flex-wrap">
            <span class="text-xs text-base-content/60">快速添加：</span>
            <button
              v-for="qh in quickHeaders"
              :key="qh.key"
              class="btn btn-ghost btn-xs"
              :class="{ 'btn-primary': hasHeader(qh.key) }"
              @click="toggleQuickHeader(qh)"
            >
              {{ qh.label }}
            </button>
          </div>
        </div>

        <!-- Body Tab -->
        <div v-show="activeRequestTab === 'body'" class="border border-base-content/10 rounded-lg bg-base-100 p-3">
          <div class="flex items-center gap-3 mb-2">
            <label v-for="bt in bodyTypes" :key="bt.key" class="flex items-center gap-1.5 text-xs text-base-content cursor-pointer">
              <input
                type="radio"
                name="bodyType"
                :value="bt.key"
                v-model="request.contentType"
                class="radio radio-sm"
              />
              <span>{{ bt.label }}</span>
            </label>
          </div>
          <textarea
            v-show="request.contentType !== 'none'"
            v-model="request.body"
            class="w-full p-3 border border-base-content/10 rounded-lg font-mono text-xs bg-base-200 text-base-content resize-y outline-none box-border focus:border-primary"
            :placeholder="bodyPlaceholder"
            rows="12"
            spellcheck="false"
          />
          <div v-if="request.contentType === 'json'" class="flex gap-2 mt-2">
            <button class="btn btn-ghost btn-xs" @click="formatJsonBody">📐 格式化</button>
            <button class="btn btn-ghost btn-xs" @click="compressJsonBody"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><line x1="16.5" y1="9.4" x2="7.5" y2="4.21"/><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg> 压缩</button>
          </div>
        </div>
      </div>

      <!-- Response Panel -->
      <div class="border border-base-content/10 rounded-lg bg-base-100 overflow-hidden" v-if="response">
        <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
          <h4 class="text-sm font-semibold text-base-content">📨 响应结果</h4>
          <div class="flex items-center gap-3">
            <span class="badge" :class="responseStatusClass">
              {{ response.status }} {{ response.statusText }}
            </span>
            <span class="text-xs text-base-content/60"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg> {{ response.time }}ms</span>
            <span class="text-xs text-base-content/60"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><line x1="16.5" y1="9.4" x2="7.5" y2="4.21"/><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg> {{ formatSize(response.size) }}</span>
          </div>
        </div>
        <div class="tabs tabs-bordered border-b-0 px-3 pt-1">
          <button
            v-for="rt in responseTabs"
            :key="rt.key"
            class="tab tab-sm"
            :class="{ 'tab-active': activeResponseTab === rt.key }"
            @click="activeResponseTab = rt.key"
          >
            {{ rt.label }}
          </button>
        </div>
        <div>
          <pre v-show="activeResponseTab === 'body'" class="p-4 text-xs font-mono overflow-x-auto bg-base-200 max-h-[500px] overflow-y-auto" :class="{ 'text-green-600': isJsonResponse }">{{ formatResponseBody }}</pre>
          <pre v-show="activeResponseTab === 'headers'" class="p-4 text-xs font-mono overflow-x-auto bg-base-200 max-h-[500px] overflow-y-auto">{{ formatResponseHeaders }}</pre>
          <pre v-show="activeResponseTab === 'raw'" class="p-4 text-xs font-mono overflow-x-auto bg-base-200 max-h-[500px] overflow-y-auto">{{ response.rawBody }}</pre>
        </div>
      </div>

      <!-- Empty State -->
      <div v-else class="flex flex-col items-center justify-center py-16 text-base-content/60">
        <div class="text-5xl mb-4"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09z"/><path d="M12 15l-3-3a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 0 1-4 2z"/><path d="M9 12H4s.55-3.03 2-4c1.62-1.08 5 0 5 0"/><path d="M12 15v5s3.03-.55 4-2c1.08-1.62 0-5 0-5"/></svg></div>
        <h3 class="text-lg font-semibold text-base-content mb-2">准备发送请求</h3>
        <p class="text-sm">输入 URL 或粘贴报文后点击"发送"</p>
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
  if (s >= 200 && s < 300) return 'badge-success'
  if (s >= 300 && s < 400) return 'badge-warning'
  if (s >= 400 && s < 500) return 'badge-error'
  if (s >= 500) return 'badge-error'
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
