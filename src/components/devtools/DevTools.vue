<template>
  <div class="devtools-container">
    <!-- Left Sidebar: Tool Navigation -->
    <aside class="devtools-sidebar">
      <div class="sidebar-header">
        <h3>🛠️ 开发工具</h3>
        <div class="sidebar-search">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" />
          </svg>
          <input v-model="searchQuery" placeholder="搜索工具..." class="search-input" />
        </div>
      </div>

      <div class="tool-categories">
        <div v-for="cat in filteredCategories" :key="cat.key" class="category-group">
          <div class="category-header">{{ cat.label }}</div>
          <div class="category-tools">
            <div
              v-for="tool in cat.tools"
              :key="tool.id"
              class="tool-item"
              :class="{ active: activeTool === tool.id }"
              @click="activeTool = tool.id"
            >
              <span class="tool-icon">{{ tool.icon }}</span>
              <span class="tool-name">{{ tool.name }}</span>
              <span v-if="!tool.offline" class="tool-online-badge" title="需要联网">🌐</span>
            </div>
          </div>
        </div>
      </div>
    </aside>

    <!-- Right Content Area -->
    <main class="devtools-content">
      <component
        :is="currentToolComponent"
        v-if="currentToolComponent"
        :key="activeTool"
      />
      <div v-else class="content-empty">
        <div class="content-empty-icon">🛠️</div>
        <h3>选择左侧工具开始使用</h3>
        <p>{{ tools.length }} 个开发工具，全部支持离线使用</p>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, defineAsyncComponent } from 'vue'
import { DEV_TOOL_REGISTRY, getToolById, type DevTool } from './DevToolRegistry'

const searchQuery = ref('')
const activeTool = ref('')

const tools = computed(() => {
  if (!searchQuery.value) return DEV_TOOL_REGISTRY
  const q = searchQuery.value.toLowerCase()
  return DEV_TOOL_REGISTRY.filter(t =>
    t.name.toLowerCase().includes(q) ||
    t.description.toLowerCase().includes(q)
  )
})

const categoryMap: Record<string, string> = {
  crypto: '加密/哈希',
  encode: '编码/转换',
  time: '时间/日期',
  code: '代码/JSON',
  text: '文本处理',
  network: '网络工具',
  convert: '进制/单位',
  format: '格式化',
  misc: '其他工具',
}

const categoryOrder = ['crypto', 'encode', 'time', 'code', 'text', 'network', 'convert', 'format', 'misc']

const filteredCategories = computed(() => {
  return categoryOrder
    .map(key => ({
      key,
      label: categoryMap[key] || key,
      tools: tools.value.filter(t => t.category === key),
    }))
    .filter(cat => cat.tools.length > 0)
})

const toolComponents: Record<string, ReturnType<typeof defineAsyncComponent>> = {
  crypto: defineAsyncComponent(() => import('./tools/CryptoTool.vue')),
  encrypt: defineAsyncComponent(() => import('./tools/EncryptTool.vue')),
  base64: defineAsyncComponent(() => import('./tools/Base64Tool.vue')),
  url: defineAsyncComponent(() => import('./tools/UrlTool.vue')),
  unicode: defineAsyncComponent(() => import('./tools/UnicodeTool.vue')),
  hex: defineAsyncComponent(() => import('./tools/HexTool.vue')),
  time: defineAsyncComponent(() => import('./tools/TimeTool.vue')),
  timecalc: defineAsyncComponent(() => import('./tools/TimeCalcTool.vue')),
  crontab: defineAsyncComponent(() => import('./tools/CrontabTool.vue')),
  qrcode: defineAsyncComponent(() => import('./tools/QrCodeTool.vue')),
  barcode: defineAsyncComponent(() => import('./tools/BarcodeTool.vue')),
  pinyin: defineAsyncComponent(() => import('./tools/PinyinTool.vue')),
  ip: defineAsyncComponent(() => import('./tools/IpTool.vue')),
  ipcalc: defineAsyncComponent(() => import('./tools/IpCalcTool.vue')),
  codeformat: defineAsyncComponent(() => import('./tools/CodeFormatTool.vue')),
  json: defineAsyncComponent(() => import('./tools/JsonTool.vue')),
  serial: defineAsyncComponent(() => import('./tools/SerialTool.vue')),
  diff: defineAsyncComponent(() => import('./tools/DiffTool.vue')),
  regex: defineAsyncComponent(() => import('./tools/RegexTool.vue')),
  random: defineAsyncComponent(() => import('./tools/RandomTool.vue')),
  text: defineAsyncComponent(() => import('./tools/TextTool.vue')),
  html: defineAsyncComponent(() => import('./tools/HtmlEntityTool.vue')),
  baseconv: defineAsyncComponent(() => import('./tools/BaseConvTool.vue')),
  variable: defineAsyncComponent(() => import('./tools/VariableTool.vue')),
  jwt: defineAsyncComponent(() => import('./tools/JwtTool.vue')),
  ascii: defineAsyncComponent(() => import('./tools/AsciiTool.vue')),
  complement: defineAsyncComponent(() => import('./tools/ComplementTool.vue')),
  armhex: defineAsyncComponent(() => import('./tools/ArmHexTool.vue')),
  unit: defineAsyncComponent(() => import('./tools/UnitTool.vue')),
  uuid: defineAsyncComponent(() => import('./tools/UuidTool.vue')),
  ws: defineAsyncComponent(() => import('./tools/WsTool.vue')),
  apidebugger: defineAsyncComponent(() => import('./tools/ApiDebugger.vue')),
}

const currentToolComponent = computed(() => {
  if (!activeTool.value) return null
  return toolComponents[activeTool.value] || null
})
</script>

<style scoped>
.devtools-container {
  display: flex;
  height: 100%;
  overflow: hidden;
  background: oklch(var(--b2));
}

/* Sidebar */
.devtools-sidebar {
  width: 260px;
  min-width: 220px;
  max-width: 300px;
  border-right: 1px solid oklch(var(--bc) / 0.1);
  background: oklch(var(--b1));
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.sidebar-header {
  padding: 16px 16px 12px;
}

.sidebar-header h3 {
  margin: 0 0 12px 0;
  font-size: 16px;
  font-weight: 700;
  color: oklch(var(--bc));
}

.sidebar-search {
  position: relative;
}

.sidebar-search svg {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  color: oklch(var(--bc) / 0.6);
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: 7px 10px 7px 30px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 6px;
  font-size: 12px;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  outline: none;
}

.search-input:focus {
  border-color: oklch(var(--p));
}

/* Categories */
.tool-categories {
  flex: 1;
  overflow-y: auto;
  padding: 0 8px 12px;
}

.category-group {
  margin-bottom: 8px;
}

.category-header {
  font-size: 10px;
  font-weight: 600;
  color: oklch(var(--bc) / 0.6);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  padding: 4px 8px;
}

.category-tools {
  display: flex;
  flex-direction: column;
}

.tool-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 10px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.1s;
  font-size: 13px;
  color: oklch(var(--bc));
}

.tool-item:hover {
  background: oklch(var(--b2));
}

.tool-item.active {
  background: oklch(var(--p) / 0.1);
  color: oklch(var(--p));
}

.tool-icon {
  font-size: 15px;
  flex-shrink: 0;
  width: 20px;
  text-align: center;
}

.tool-name {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tool-online-badge {
  font-size: 10px;
  flex-shrink: 0;
}

/* Content */
.devtools-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
}

.content-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: oklch(var(--bc) / 0.6);
  text-align: center;
  gap: 12px;
}

.content-empty-icon {
  font-size: 64px;
  opacity: 0.3;
}

.content-empty h3 {
  font-size: 18px;
  font-weight: 600;
  color: oklch(var(--bc));
  margin: 0;
}

.content-empty p {
  font-size: 13px;
  margin: 0;
}
</style>
