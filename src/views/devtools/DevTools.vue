<template>
  <div class="flex flex-col h-full overflow-hidden bg-base-200">
    <!-- 网格首页 -->
    <template v-if="!activeTool">
      <div class="flex-1 overflow-y-auto">
        <div class="max-w-[1240px] mx-auto">
          <!-- 头部：标题 + 搜索 -->
          <div class="flex flex-wrap items-end justify-between gap-4 mb-7">
            <div>
              <h1 class="m-0 text-xl font-bold text-base-content flex items-center gap-2.5">
                <span class="w-9 h-9 rounded-xl bg-primary/15 text-primary flex items-center justify-center">
                  <SvgIcon name="tool" size="18" />
                </span>
                开发工具
              </h1>
              <p class="text-xs text-base-content/50 mt-1.5">{{ filteredCount }} 个工具 · 全部本地运行 · 即开即用</p>
            </div>
            <div class="relative w-full max-w-sm">
              <SvgIcon name="search" size="15" class="absolute left-3 top-1/2 -translate-y-1/2 text-base-content/50 pointer-events-none" />
              <input
                ref="searchInputRef"
                v-model="searchQuery"
                placeholder="搜索工具（名称 / 拼音 / 关键词）..."
                class="w-full py-2.5 pl-9 pr-4 rounded-lg border border-base-content/10 bg-base-100 text-sm text-base-content outline-none focus:border-primary focus:ring-2 focus:ring-primary/20 transition-shadow"
              />
              <button
                v-if="searchQuery"
                class="absolute right-2 top-1/2 -translate-y-1/2 btn btn-ghost btn-xs btn-circle"
                title="清空"
                @click="searchQuery = ''"
              >
                <SvgIcon name="close" size="12" />
              </button>
            </div>
          </div>

          <!-- 空结果 -->
          <div v-if="filteredCategories.length === 0" class="flex flex-col items-center justify-center py-24 text-base-content/40 gap-3">
            <SvgIcon name="search" size="40" class="opacity-30" />
            <p class="text-sm">未找到匹配「{{ searchQuery }}」的工具</p>
          </div>

          <!-- 分类分区 + 卡片网格 -->
          <div v-for="cat in filteredCategories" :key="cat.key" class="mb-8">
            <div class="flex items-center gap-2.5 mb-3.5">
              <span class="w-7 h-7 rounded-lg bg-primary/12 text-primary flex items-center justify-center">
                <SvgIcon :name="cat.icon" size="14" />
              </span>
              <h2 class="m-0 text-sm font-semibold text-base-content">{{ cat.label }}</h2>
              <span class="text-[11px] text-base-content/40">{{ cat.tools.length }} 个</span>
            </div>
            <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-3">
              <button
                v-for="tool in cat.tools"
                :key="tool.id"
                class="group flex flex-col items-start gap-2.5 p-4 rounded-xl bg-base-100 border border-base-content/10 hover:border-primary/50 hover:shadow-lg hover:-translate-y-0.5 transition-all duration-150 text-left cursor-pointer"
                :title="tool.description"
                @click="activateTool(tool.id)"
              >
                <span class="w-10 h-10 rounded-lg bg-primary/10 text-primary flex items-center justify-center group-hover:bg-primary group-hover:text-primary-content transition-colors duration-150">
                  <SvgIcon :name="tool.icon" size="18" />
                </span>
                <span class="text-sm font-medium text-base-content leading-tight">{{ tool.name }}</span>
                <span class="text-[11px] text-base-content/50 leading-snug line-clamp-2 min-h-[2.4em]">{{ tool.description }}</span>
                <span
                  v-if="!tool.offline"
                  class="text-[10px] text-warning flex items-center gap-1 border border-warning/25 rounded-full px-1.5 py-px"
                >
                  <SvgIcon name="globe" size="9" /> 需联网
                </span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </template>

    <!-- 工具页 -->
    <template v-else>
      <component
        :is="currentToolComponent"
        v-if="currentToolComponent"
        :key="activeTool"
        @back="activeTool = ''"
      />
    </template>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'DevTools' })
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref, computed, defineAsyncComponent, watch, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { DEV_TOOL_REGISTRY } from './DevToolRegistry'

const route = useRoute()
const searchQuery = ref('')
const activeTool = ref('')
const searchInputRef = ref<HTMLInputElement | null>(null)

const categoryMap: Record<string, { label: string; icon: string }> = {
  crypto: { label: '加密 / 哈希', icon: 'lock' },
  encode: { label: '编码 / 转换', icon: 'refresh' },
  time: { label: '时间 / 日期', icon: 'clock' },
  code: { label: '代码 / JSON', icon: 'code' },
  text: { label: '文本处理', icon: 'notebook' },
  network: { label: '网络工具', icon: 'globe' },
  convert: { label: '进制 / 单位', icon: 'layers' },
  format: { label: '格式化', icon: 'pencil' },
  misc: { label: '其他工具', icon: 'grid' },
}

const categoryOrder = ['crypto', 'encode', 'time', 'code', 'text', 'network', 'convert', 'format', 'misc']

const filteredTools = computed(() => {
  const q = searchQuery.value.trim().toLowerCase()
  if (!q) { return DEV_TOOL_REGISTRY }
  return DEV_TOOL_REGISTRY.filter(t => {
    if (t.name.toLowerCase().includes(q) || t.description.toLowerCase().includes(q)) { return true }
    return t.keywords.toLowerCase().split(' ').some(k => k.startsWith(q))
  })
})

const filteredCount = computed(() => filteredTools.value.length)

const filteredCategories = computed(() =>
  categoryOrder
    .map(key => ({
      key,
      label: categoryMap[key]?.label || key,
      icon: categoryMap[key]?.icon || 'tool',
      tools: filteredTools.value.filter(t => t.category === key),
    }))
    .filter(cat => cat.tools.length > 0)
)

function activateTool(id: string): void {
  activeTool.value = id
  searchQuery.value = ''
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

// 快捷键：在工具页按 Esc 返回列表（输入框内不拦截）
watch(activeTool, (v) => {
  if (!v) { searchInputRef.value?.focus() }
})

const toolComponents: Record<string, ReturnType<typeof defineAsyncComponent>> = {
  crypto: defineAsyncComponent(() => import('./tools/CryptoTool.vue')),
  encrypt: defineAsyncComponent(() => import('./tools/EncryptTool.vue')),
  navicat: defineAsyncComponent(() => import('./tools/NavicatTool.vue')),
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
  image: defineAsyncComponent(() => import('./tools/ImageTool.vue')),
}

const currentToolComponent = computed(() => {
  if (!activeTool.value) { return null }
  return toolComponents[activeTool.value] || null
})
</script>
