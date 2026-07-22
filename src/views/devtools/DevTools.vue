<template>
  <div class="flex h-full overflow-hidden bg-base-200">
    <!-- Left Sidebar: Tool Navigation -->
    <aside class="w-[260px] min-w-[220px] max-w-[300px] border-r border-base-content/10 bg-base-100 flex flex-col shrink-0">
      <div class="px-4 pt-4 pb-3">
        <h3 class="m-0 mb-3 text-base font-bold text-base-content"><SvgIcon name="tool" size="14" />  开发工具</h3>
        <div class="relative">
          <SvgIcon name="search" size="14" class="absolute left-[10px] top-1/2 -translate-y-1/2 text-base-content/60" />
          <input
            ref="searchInputRef"
            v-model="searchQuery"
            placeholder="搜索工具... (Ctrl+K)"
            class="w-full py-[7px] pl-[30px] pr-[10px] border border-base-content/10 rounded-md text-xs bg-base-200 text-base-content outline-none focus:border-primary"
            @focus="showSearchDropdown = true"
            @blur="onSearchBlur"
            @keydown.down.prevent="navigateSearch(1)"
            @keydown.up.prevent="navigateSearch(-1)"
            @keydown.enter.prevent="selectFromSearch"
          />
        </div>
      </div>

      <div class="flex-1 overflow-y-auto px-2 pb-3">
        <!-- 搜索下拉面板 -->
        <div
          v-if="showSearchDropdown && searchQuery"
          class="mb-2 rounded-md border border-base-content/10 bg-base-100 shadow-lg overflow-hidden"
        >
          <div
            v-for="(tool, index) in searchResults"
            :key="tool.id"
            class="flex items-center gap-2 px-[10px] py-[7px] cursor-pointer transition-all duration-100 text-sm"
            :class="{ 'bg-primary/10 text-primary': index === searchSelectedIndex }"
            @mousedown.prevent="activateTool(tool.id)"
            @mouseenter="searchSelectedIndex = index"
          >
            <span class="shrink-0 w-5 flex items-center justify-center"><SvgIcon :name="tool.icon" size="15" /></span>
            <span class="flex-1 truncate">{{ tool.name }}</span>
            <span class="text-[10px] text-base-content/50 truncate max-w-[120px]">{{ tool.description }}</span>
          </div>
          <div v-if="searchResults.length === 0" class="px-3 py-4 text-center text-xs text-base-content/50">
            未找到匹配工具
          </div>
        </div>

        <!-- 收藏分区 -->
        <div v-if="favoriteTools.length > 0 && !searchQuery" class="mb-2">
          <div class="text-[10px] font-semibold text-base-content/60 uppercase tracking-[0.5px] px-2 py-1 flex items-center gap-1">
            <SvgIcon name="star" size="12" /> 收藏
          </div>
          <div class="flex flex-col">
            <div
              v-for="tool in favoriteTools"
              :key="tool.id"
              class="flex items-center gap-2 px-[10px] py-[7px] rounded-md cursor-pointer transition-all duration-100 text-sm text-base-content hover:bg-base-200 group"
              :class="{ 'bg-primary/10 text-primary': activeTool === tool.id }"
              :title="tool.description"
              @click="activateTool(tool.id)"
            >
              <span class="shrink-0 w-5 flex items-center justify-center"><SvgIcon :name="tool.icon" size="15" /></span>
              <span class="flex-1 truncate">{{ tool.name }}</span>
              <span
                class="shrink-0 opacity-50 hover:opacity-100 transition-opacity"
                @click.stop="toggleFavorite(tool.id)"
              >
                <SvgIcon name="star" size="13" class="text-yellow-500" />
              </span>
            </div>
          </div>
        </div>

        <!-- 最近使用分区 -->
        <div v-if="recentTools.length > 0 && !searchQuery" class="mb-2">
          <div class="text-[10px] font-semibold text-base-content/60 uppercase tracking-[0.5px] px-2 py-1 flex items-center gap-1">
            <SvgIcon name="clock" size="12" /> 最近使用
          </div>
          <div class="flex flex-col">
            <div
              v-for="tool in recentTools"
              :key="tool.id"
              class="flex items-center gap-2 px-[10px] py-[7px] rounded-md cursor-pointer transition-all duration-100 text-sm text-base-content hover:bg-base-200 group"
              :class="{ 'bg-primary/10 text-primary': activeTool === tool.id }"
              :title="tool.description"
              @click="activateTool(tool.id)"
            >
              <span class="shrink-0 w-5 flex items-center justify-center"><SvgIcon :name="tool.icon" size="15" /></span>
              <span class="flex-1 truncate">{{ tool.name }}</span>
              <span
                class="shrink-0 opacity-0 group-hover:opacity-50 hover:!opacity-100 transition-opacity"
                @click.stop="toggleFavorite(tool.id)"
              >
                <SvgIcon :name="isFavorite(tool.id) ? 'star' : 'starOutline'" size="13" :class="isFavorite(tool.id) ? 'text-yellow-500' : ''" />
              </span>
            </div>
          </div>
        </div>

        <!-- 分类工具列表 -->
        <div v-for="cat in filteredCategories" :key="cat.key" class="mb-2">
          <div
            class="text-[10px] font-semibold text-base-content/60 uppercase tracking-[0.5px] px-2 py-1 cursor-pointer select-none flex items-center gap-1 hover:text-base-content"
            @click="toggleCategoryCollapsed(cat.key)"
          >
            <SvgIcon :name="isCategoryCollapsed(cat.key) ? 'chevronRight' : 'chevronDown'" size="10" />
            {{ cat.label }}
          </div>
          <div v-show="!isCategoryCollapsed(cat.key)" class="flex flex-col">
            <div
              v-for="tool in cat.tools"
              :key="tool.id"
              class="flex items-center gap-2 px-[10px] py-[7px] rounded-md cursor-pointer transition-all duration-100 text-sm text-base-content hover:bg-base-200 group"
              :class="{ 'bg-primary/10 text-primary': activeTool === tool.id }"
              :title="tool.description"
              @click="activateTool(tool.id)"
            >
              <span class="shrink-0 w-5 flex items-center justify-center"><SvgIcon :name="tool.icon" size="15" /></span>
              <span class="flex-1 truncate">{{ tool.name }}</span>
              <span v-if="!tool.offline" class="text-[10px] shrink-0" title="需要联网"><SvgIcon name="globe" size="14" class="align-text-bottom" /></span>
              <span
                class="shrink-0 opacity-0 group-hover:opacity-50 hover:!opacity-100 transition-opacity"
                @click.stop="toggleFavorite(tool.id)"
              >
                <SvgIcon :name="isFavorite(tool.id) ? 'star' : 'starOutline'" size="13" :class="isFavorite(tool.id) ? 'text-yellow-500' : ''" />
              </span>
            </div>
          </div>
        </div>
      </div>
    </aside>

    <!-- Right Content Area -->
    <main class="flex-1 overflow-y-auto p-6">
      <component
        :is="currentToolComponent"
        v-if="currentToolComponent"
        :key="activeTool"
      />
      <div v-else class="flex flex-col items-center justify-center h-full text-base-content/60 text-center gap-3">
        <div class="text-[64px] opacity-30"><SvgIcon name="tool" size="14" /> </div>
        <h3 class="text-lg font-semibold text-base-content m-0">选择左侧工具开始使用</h3>
        <p class="text-sm m-0">{{ tools.length }} 个开发工具，全部支持离线使用</p>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'DevTools' })
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref, computed, defineAsyncComponent, watch, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { DEV_TOOL_REGISTRY, type DevTool } from './DevToolRegistry'
import { useDevTools } from './composables/useDevTools'

const route = useRoute()
const searchQuery = ref('')
const activeTool = ref('')
const showSearchDropdown = ref(false)
const searchSelectedIndex = ref(0)
const searchInputRef = ref<HTMLInputElement | null>(null)

const {
  favoriteTools,
  recentTools,
  isFavorite,
  toggleFavorite,
  recordUsage,
  isCategoryCollapsed,
  toggleCategoryCollapsed,
  searchTools,
} = useDevTools()

const searchResults = computed<DevTool[]>(() => searchTools(searchQuery.value))

watch(searchQuery, () => {
  searchSelectedIndex.value = 0
})

const tools = computed(() => {
  if (!searchQuery.value) {return DEV_TOOL_REGISTRY}
  return searchResults.value
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

function activateTool(id: string): void {
  activeTool.value = id
  recordUsage(id)
  searchQuery.value = ''
  showSearchDropdown.value = false
}

// 处理路由 ?tool=id 参数（来自全局 Cmd+K 弹窗）
onMounted(() => {
  const toolId = route.query.tool as string
  if (toolId && toolComponents[toolId]) {
    activateTool(toolId)
  }
})

watch(() => route.query.tool, (toolId) => {
  if (toolId && typeof toolId === 'string' && toolComponents[toolId]) {
    activateTool(toolId)
  }
})

function onSearchBlur(): void {
  // 延迟关闭，让 mousedown 事件先触发
  setTimeout(() => {
    showSearchDropdown.value = false
  }, 150)
}

function navigateSearch(direction: number): void {
  const max = searchResults.value.length
  if (max === 0) return
  searchSelectedIndex.value = (searchSelectedIndex.value + direction + max) % max
}

function selectFromSearch(): void {
  const tool = searchResults.value[searchSelectedIndex.value]
  if (tool) {
    activateTool(tool.id)
  }
}

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
  calculator: defineAsyncComponent(() => import('./tools/CalculatorTool.vue')),
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
  html2md: defineAsyncComponent(() => import('./tools/HtmlToMdTool.vue')),
}

const currentToolComponent = computed(() => {
  if (!activeTool.value) {return null}
  return toolComponents[activeTool.value] || null
})
</script>
