<template>
  <ToolPage
    icon="download"
    name="HTML 转 Markdown"
    description="左侧输入 HTML，右侧实时预览与 Markdown 转换结果"
    no-scroll
    @back="$emit('back')"
  >
    <div class="flex flex-col h-full min-h-0">
      <!-- 顶部工具条：网址抓取 + 实时转换开关 + 全局操作 -->
      <div class="shrink-0 px-4 py-3 border-b border-base-content/10 flex flex-col gap-2.5">
        <div class="flex items-center gap-2 flex-wrap">
          <div class="flex-1 relative min-w-[200px]">
            <SvgIcon name="link" size="14" class="absolute left-[10px] top-1/2 -translate-y-1/2 text-base-content/50 pointer-events-none" />
            <input
              v-model="urlInput"
              placeholder="输入网址（如 https://example.com）..."
              class="w-full py-1.5 pl-8 pr-3 border border-base-content/10 rounded-lg text-xs bg-base-200/60 text-base-content outline-none focus:border-primary"
              @keyup.enter="fetchUrl"
            />
          </div>
          <button class="btn btn-primary btn-sm gap-1.5 shrink-0" @click="fetchUrl" :disabled="loading">
            <span v-if="loading" class="loading loading-spinner loading-xs"></span>
            <SvgIcon v-else name="globe" size="13" />
            {{ loading ? '获取中...' : '抓取网页' }}
          </button>
          <label class="flex items-center gap-1.5 shrink-0 text-[11px] text-base-content/60 cursor-pointer select-none" title="输入停止 350ms 后自动转换；关闭后需手动点击「转换」">
            <input v-model="autoConvert" type="checkbox" class="checkbox checkbox-xs checkbox-primary" />
            实时转换
          </label>
          <div class="w-px h-5 bg-base-content/10 shrink-0"></div>
          <button class="btn btn-ghost btn-sm gap-1.5 shrink-0" @click="convert" :disabled="!htmlInput.trim()" title="立即转换（不等防抖）">
            <SvgIcon name="refresh" size="12" /> 转换
          </button>
          <button class="btn btn-outline btn-sm gap-1.5 shrink-0" @click="copyResult" :disabled="!output">
            <SvgIcon name="copy" size="12" /> 复制 MD
          </button>
          <button class="btn btn-ghost btn-sm gap-1.5 shrink-0" @click="clear" :disabled="!htmlInput && !output">
            <SvgIcon name="trash" size="12" /> 清空
          </button>
        </div>

        <!-- 加载进度（SPA 页面 WebView 渲染抓取时提示等待） -->
        <div v-if="loading" class="flex items-center gap-1.5 text-[11px] text-base-content/50">
          <span class="loading loading-spinner loading-xs"></span>{{ loadingText }}
        </div>

        <!-- SPA 动态渲染页面提示 -->
        <div v-if="spaWarn" class="flex items-start gap-2 px-3 py-2 bg-warning/10 border border-warning/30 rounded-lg text-xs text-warning">
          <SvgIcon name="alertTriangle" size="14" class="mt-0.5 shrink-0" />
          <div class="flex-1">
            <p class="leading-relaxed">{{ spaWarn }}</p>
            <button class="btn btn-xs btn-outline mt-1.5" @click="openInBrowser"><SvgIcon name="globe" size="11" /> 在浏览器中打开</button>
          </div>
        </div>

        <!-- 超大文档：自动转换被跳过时的提示 -->
        <div v-if="autoSkipped" class="flex items-center gap-2 px-3 py-2 bg-warning/10 border border-warning/30 rounded-lg text-[11px] text-warning">
          <SvgIcon name="alertTriangle" size="13" class="shrink-0" />
          <span class="flex-1">内容超过 {{ formatSize(AUTO_CONVERT_LIMIT) }}，已暂停自动转换（避免卡顿）。</span>
          <button class="btn btn-xs btn-outline" @click="convert">手动转换</button>
        </div>
      </div>

      <!-- 双栏主体：左 HTML / 右 Markdown，各自可切 源码 / 预览 -->
      <div ref="splitWrap" class="flex-1 min-h-0 flex">
        <!-- 左栏：HTML -->
        <section class="flex flex-col shrink-0 min-w-0 h-full" :style="{ width: splitRatio + '%' }">
          <header class="shrink-0 flex items-center justify-between gap-2 px-3 py-2 border-b border-base-content/10 bg-base-200/40">
            <div class="join">
              <button class="btn btn-xs join-item gap-1" :class="leftTab === 'code' ? 'btn-primary' : 'btn-ghost'" @click="leftTab = 'code'">
                <SvgIcon name="code" size="11" /> HTML 源码
              </button>
              <button class="btn btn-xs join-item gap-1" :class="leftTab === 'preview' ? 'btn-primary' : 'btn-ghost'" @click="leftTab = 'preview'">
                <SvgIcon name="eye" size="11" /> 预览
              </button>
            </div>
            <span class="text-[11px] text-base-content/40 shrink-0">{{ htmlInput ? `${htmlInput.length} 字符 · ${lineCount(htmlInput)} 行` : '未输入' }}</span>
          </header>
          <div class="flex-1 min-h-0 relative">
            <textarea
              v-show="leftTab === 'code'"
              v-model="htmlInput"
              class="block w-full h-full resize-none border-0 outline-none bg-transparent p-3 font-mono text-[12px] leading-relaxed text-base-content"
              placeholder="在此粘贴 HTML，或上方输入网址抓取。右侧会实时输出 Markdown。"
              spellcheck="false"
            ></textarea>
            <template v-if="leftTab === 'preview'">
              <!-- 沙箱 iframe：脚本/表单/跳转全部禁用，样式与父窗口隔离，所见即浏览器渲染效果 -->
              <iframe
                v-if="showHtmlPreview"
                class="w-full h-full border-0 bg-base-100"
                sandbox="allow-popups"
                :srcdoc="htmlPreviewDoc"
                title="HTML 预览"
              ></iframe>
              <div v-else class="h-full flex flex-col items-center justify-center gap-2 px-6 text-center">
                <SvgIcon name="eyeOff" size="22" class="text-base-content/25" />
                <!-- 用 debouncedHtml 判断：防抖未完成时别误报成「内容较大」 -->
                <p v-if="!debouncedHtml.trim()" class="text-xs text-base-content/40">输入 HTML 后这里会实时渲染</p>
                <template v-else>
                  <p class="text-xs text-base-content/50">内容较大（{{ formatSize(debouncedHtml.length) }}），自动预览已暂停</p>
                  <button class="btn btn-xs btn-outline" @click="forceHtmlPreview = true"><SvgIcon name="play" size="11" /> 仍要预览</button>
                </template>
              </div>
            </template>
          </div>
        </section>

        <!-- 可拖拽分隔条 -->
        <div
          class="w-1.5 shrink-0 cursor-col-resize bg-base-200/60 hover:bg-primary/50 transition-colors focus:outline-none focus:bg-primary/60"
          :class="dragging ? 'bg-primary/60' : ''"
          role="separator"
          aria-orientation="vertical"
          aria-label="调整左右栏宽度"
          tabindex="0"
          title="拖动调整宽度，双击恢复等分（方向键亦可）"
          @mousedown="startDrag"
          @keydown="onSplitKeydown"
        ></div>

        <!-- 右栏：Markdown -->
        <section class="flex-1 flex flex-col min-w-0 h-full">
          <header class="shrink-0 flex items-center justify-between gap-2 px-3 py-2 border-b border-base-content/10 bg-base-200/40">
            <div class="join">
              <button class="btn btn-xs join-item gap-1" :class="rightTab === 'code' ? 'btn-primary' : 'btn-ghost'" @click="rightTab = 'code'">
                <SvgIcon name="code" size="11" /> Markdown
              </button>
              <button class="btn btn-xs join-item gap-1" :class="rightTab === 'preview' ? 'btn-primary' : 'btn-ghost'" @click="rightTab = 'preview'">
                <SvgIcon name="eye" size="11" /> 预览
              </button>
            </div>
            <span class="text-[11px] text-base-content/40 shrink-0">{{ output ? `${output.length} 字符 · ${lineCount(output)} 行` : '暂无结果' }}</span>
          </header>
          <div class="flex-1 min-h-0 relative">
            <pre
              v-if="rightTab === 'code'"
              class="h-full overflow-auto whitespace-pre-wrap break-words m-0 p-3 font-mono text-[12px] leading-relaxed text-base-content"
            >{{ output || 'Markdown 结果将实时显示在这里' }}</pre>
            <template v-else>
              <!-- v-if 包住容器：不预览时不触发 marked 渲染，避免大文档卡顿 -->
              <div v-if="showMdPreview" class="markdown-body h-full overflow-auto p-3" v-html="previewHtml" @click="onPreviewClick"></div>
              <div v-else class="h-full flex flex-col items-center justify-center gap-2 px-6 text-center">
                <SvgIcon name="eyeOff" size="22" class="text-base-content/25" />
                <p v-if="!output" class="text-xs text-base-content/40">转换后这里会实时渲染 Markdown</p>
                <template v-else>
                  <p class="text-xs text-base-content/50">结果较大（{{ formatSize(output.length) }}），自动预览已暂停</p>
                  <button class="btn btn-xs btn-outline" @click="forceMdPreview = true"><SvgIcon name="play" size="11" /> 仍要预览</button>
                </template>
              </div>
            </template>
          </div>
        </section>
      </div>
    </div>

    <!-- 拖拽时的全屏遮罩：防止鼠标划过 iframe 时丢失 mousemove -->
    <div v-if="dragging" class="fixed inset-0 z-50 cursor-col-resize"></div>
  </ToolPage>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import TurndownService from 'turndown'
import { gfm } from 'turndown-plugin-gfm'
import { renderMarkdown, setupCopyCode } from '../../../composables/useMarkdownRenderer'
import { copyText } from '../toolUtils'
import { getTauriAPI } from '../../../utils/tauri-api'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

/** 超过该长度不自动转换（turndown 是同步阻塞的，大文档会冻 UI） */
const AUTO_CONVERT_LIMIT = 200_000
/** 超过该长度不自动渲染预览，需用户手动确认 */
const PREVIEW_LIMIT = 150_000
const DEBOUNCE_MS = 350
const LS_PREFIX = 'supertool.html2md.'

const toast = useToast()
const htmlInput = ref('')
const urlInput = ref('')
const output = ref('')
const loading = ref(false)
// 加载进度文案（SPA 页面降级 WebView 抓取时显示）
const loadingText = ref('')
// 左右两栏的显示模式：源码 / 预览
const leftTab = ref<'code' | 'preview'>('code')
const rightTab = ref<'code' | 'preview'>('preview')
// 实时转换开关（关闭后需手动点「转换」）
const autoConvert = ref(true)
// 大文档手动确认预览
const forceHtmlPreview = ref(false)
const forceMdPreview = ref(false)
// 大文档跳过自动转换的提示
const autoSkipped = ref(false)

// SPA 动态渲染页面检测：抓到的 HTML 是空壳（正文靠 JS 渲染，reqwest 拿不到）
const spaWarn = ref('')
// 最近一次抓取成功的 URL（openInBrowser 用它，避免用户改输入框后打开无关地址）
const fetchedUrl = ref('')

// ---------- 持久化：栏宽 / tab / 实时开关 ----------
function loadNumber(key: string, fallback: number): number {
  try {
    const v = localStorage.getItem(LS_PREFIX + key)
    return v === null ? fallback : Number(v)
  } catch {
    return fallback
  }
}
function save(key: string, value: string | number | boolean) {
  try {
    localStorage.setItem(LS_PREFIX + key, String(value))
  } catch {
    /* localStorage 不可用时静默降级 */
  }
}

const splitRatio = ref(loadNumber('splitRatio', 50))
const dragging = ref(false)
const splitWrap = ref<HTMLElement | null>(null)

function startDrag(e: MouseEvent) {
  // 双击恢复等分：不能用 dblclick 事件——首次 mousedown 就挂出全屏遮罩，
  // 第二次点击会落在遮罩上，dblclick 永远到不了分隔条（已实测踩坑）。
  // 用 mousedown 自带的连击计数（detail=2）判断。
  if (e.detail >= 2) {
    resetSplit()
    return
  }
  e.preventDefault()
  dragging.value = true
  window.addEventListener('mousemove', onDrag)
  window.addEventListener('mouseup', stopDrag)
}
function onDrag(e: MouseEvent) {
  const el = splitWrap.value
  if (!el) {return}
  const rect = el.getBoundingClientRect()
  const pct = ((e.clientX - rect.left) / rect.width) * 100
  // 限制在 20%~80%，避免某一栏被拖没
  splitRatio.value = Math.min(80, Math.max(20, pct))
}
function stopDrag() {
  dragging.value = false
  window.removeEventListener('mousemove', onDrag)
  window.removeEventListener('mouseup', stopDrag)
  save('splitRatio', Math.round(splitRatio.value))
}
function resetSplit() {
  splitRatio.value = 50
  save('splitRatio', 50)
}
/** 键盘调整栏宽（每次 5%），保证不用鼠标也能分栏 */
function onSplitKeydown(e: KeyboardEvent) {
  if (e.key === 'ArrowLeft') {
    splitRatio.value = Math.max(20, splitRatio.value - 5)
  } else if (e.key === 'ArrowRight') {
    splitRatio.value = Math.min(80, splitRatio.value + 5)
  } else if (e.key === 'Enter' || e.key === ' ') {
    resetSplit()
  } else {
    return
  }
  e.preventDefault()
  save('splitRatio', Math.round(splitRatio.value))
}
onBeforeUnmount(stopDrag)

// 恢复上次的 tab 与开关状态（必须在下面的 watch 注册之前，否则恢复动作会被当成用户改动写回）
try {
  const lt = localStorage.getItem(LS_PREFIX + 'leftTab')
  if (lt === 'code' || lt === 'preview') {leftTab.value = lt}
  const rt = localStorage.getItem(LS_PREFIX + 'rightTab')
  if (rt === 'code' || rt === 'preview') {rightTab.value = rt}
  const ac = localStorage.getItem(LS_PREFIX + 'autoConvert')
  if (ac !== null) {autoConvert.value = ac === 'true'}
} catch {
  /* localStorage 不可用时静默降级 */
}
watch(leftTab, v => save('leftTab', v))
watch(rightTab, v => save('rightTab', v))
watch(autoConvert, v => {
  save('autoConvert', v)
  // 打开开关时立刻补一次转换，避免关着的时候输入的内容没同步
  if (v && htmlInput.value.trim()) {syncFromInput(false)}
})

// ---------- 防抖：输入停止后才做重活 ----------
// debouncedHtml 同时驱动「自动转换」与「HTML 预览」，避免每次击键都跑 turndown / 重建 iframe。
// 自动转换只由这里的定时器触发（不额外 watch debouncedHtml），否则手动转换/抓取赋值会
// 与 watch 回调重复跑一遍 turndown——大文档上一次转换要几百毫秒，重复即肉眼可见的卡顿。
const debouncedHtml = ref('')
let debounceTimer: ReturnType<typeof setTimeout> | null = null

/** 取消待执行的自动任务，并把当前输入同步给预览（不触发转换） */
function flushInput() {
  if (debounceTimer) {
    clearTimeout(debounceTimer)
    debounceTimer = null
  }
  debouncedHtml.value = htmlInput.value
}

/** 用当前输入立即跑一次转换（手动按钮 / 开关打开 / 抓取完成都走这里） */
function syncFromInput(verbose: boolean) {
  flushInput()
  // 手动转换时超限也照转（用户点按钮就是明确要求），但不再挂「已暂停自动转换」的提示
  autoSkipped.value = !verbose && htmlInput.value.length > AUTO_CONVERT_LIMIT
  runConvert(debouncedHtml.value, verbose)
}

watch(htmlInput, () => {
  if (debounceTimer) {clearTimeout(debounceTimer)}
  debounceTimer = setTimeout(() => {
    debounceTimer = null
    debouncedHtml.value = htmlInput.value
    if (!autoConvert.value) {return}
    if (htmlInput.value.length > AUTO_CONVERT_LIMIT) {
      autoSkipped.value = true
      return
    }
    autoSkipped.value = false
    runConvert(debouncedHtml.value, false)
  }, DEBOUNCE_MS)
})
watch(debouncedHtml, () => {forceHtmlPreview.value = false})
watch(output, () => {forceMdPreview.value = false})
onBeforeUnmount(() => {
  if (debounceTimer) {clearTimeout(debounceTimer)}
})

// ---------- 预览渲染 ----------
const showHtmlPreview = computed(() =>
  leftTab.value === 'preview' &&
  !!debouncedHtml.value.trim() &&
  (debouncedHtml.value.length <= PREVIEW_LIMIT || forceHtmlPreview.value),
)
const showMdPreview = computed(() =>
  rightTab.value === 'preview' &&
  !!output.value &&
  (output.value.length <= PREVIEW_LIMIT || forceMdPreview.value),
)

// 主题变化时让 iframe 内的配色跟着变（daisyUI 把 data-theme 挂在 <html> 上）
const themeTick = ref(0)
let themeObserver: MutationObserver | null = null
onMounted(() => {
  themeObserver = new MutationObserver(() => {themeTick.value++})
  themeObserver.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme', 'class'] })
})
onBeforeUnmount(() => {themeObserver?.disconnect()})

/** 从 CSS 变量取当前主题色，用于 iframe 内联样式（沙箱内拿不到外部样式表） */
function cssVar(name: string, fallback: string): string {
  const v = getComputedStyle(document.documentElement).getPropertyValue(name).trim()
  return v || fallback
}

const htmlPreviewDoc = computed(() => {
  void themeTick.value // 依赖主题
  const raw = debouncedHtml.value
  if (!raw.trim()) {return ''}
  // 沙箱内样式独立：配色跟随主题，字体/间距模拟常规浏览器默认排版
  const css = `:root{color-scheme:light dark}
html,body{margin:0;padding:14px;background:${cssVar('--color-base-100', '#ffffff')};color:${cssVar('--color-base-content', '#1f2937')};font-family:-apple-system,BlinkMacSystemFont,'Segoe UI','PingFang SC','Microsoft YaHei',sans-serif;font-size:14px;line-height:1.7;word-break:break-word}
img,video{max-width:100%;height:auto}
table{border-collapse:collapse;max-width:100%}
th,td{border:1px solid ${cssVar('--color-base-content', '#1f2937')}22;padding:6px 10px;text-align:left}
th{background:${cssVar('--color-base-200', '#f3f4f6')}}
pre{overflow-x:auto;padding:12px;border-radius:6px;background:${cssVar('--color-base-200', '#f3f4f6')}}
code{font-family:'SF Mono',ui-monospace,Menlo,monospace;font-size:13px}
a{color:${cssVar('--color-primary', '#2563eb')}}
blockquote{margin:12px 0;padding-left:12px;border-left:3px solid ${cssVar('--color-base-content', '#1f2937')}33}`
  const s = raw.trim()
  // 贴的是完整文档（含 doctype/html）时保留原结构，只把主题样式注进 head。
  // 替换串必须用函数形式：css 里若出现 $& / $1 会被当成替换模式而破坏输出。
  if (/^<!doctype\s+html|<html[\s>]/i.test(s)) {
    if (/<\/head>/i.test(s)) {return s.replace(/<\/head>/i, () => `<style>${css}</style></head>`)}
    if (/<body[^>]*>/i.test(s)) {return s.replace(/<body([^>]*)>/i, (_m, attrs: string) => `<head><meta charset="utf-8"><base target="_blank"><style>${css}</style></head><body${attrs}>`)}
  }
  return `<!DOCTYPE html><html><head><meta charset="utf-8"><base target="_blank"><style>${css}</style></head><body>${s}</body></html>`
})

// Markdown 渲染预览（marked + DOMPurify 消毒 + 代码高亮）
const previewHtml = computed(() => (showMdPreview.value ? renderMarkdown(output.value) : ''))

// 复制按钮事件委托（setupCopyCode 返回 handler，绑到预览容器 click）
const onPreviewClick = setupCopyCode()

// ---------- 转换核心 ----------
// 前端 HTML→Markdown：turndown（CommonMark/GFM 工业标准），替代后端 html2md
// （html2md 不剥离 script/style，实体解码错误，已弃用）
function createConverter(): TurndownService {
  const td = new TurndownService({
    headingStyle: 'atx',        // # 标题（通用性优于 setext 下划线式）
    bulletListMarker: '-',
    codeBlockStyle: 'fenced',
    fence: '```',
    emDelimiter: '*',
    strongDelimiter: '**',
  })
  // GFM 表格支持
  td.use(gfm)
  // 含合并单元格（rowspan/colspan）的表格：GFM 无法表达合并单元格，转换会列数错乱
  // 渲染异常——保留原始 HTML（Markdown 预览可正常渲染）
  td.addRule('tableMergedCells', {
    filter: node => node.nodeName === 'TABLE' && !!node.querySelector('td[rowspan], td[colspan], th[rowspan], th[colspan]'),
    replacement: (_content, node) => `\n\n${node.outerHTML}\n\n`,
  })
  // turndown 默认不处理这些标签——显式剥离页面骨架/脚本/交互元素。
  // 用 nodeName 判断，规避 svg 不在 HTMLElementTagNameMap 的类型限制。
  // 注意：不剥 form 容器本身（部分站点正文在 form 内，如搜索页），只剥控件
  const NON_CONTENT_TAGS = new Set([
    'SCRIPT', 'STYLE', 'HEAD', 'TITLE', 'META', 'LINK', 'IFRAME', 'NOSCRIPT', 'TEMPLATE',
    'INPUT', 'BUTTON', 'SELECT', 'OPTION', 'TEXTAREA', 'LABEL',
    'SVG', 'CANVAS', 'AUDIO', 'VIDEO', 'OBJECT', 'EMBED',
  ])
  td.addRule('stripNonContent', {
    filter: node => NON_CONTENT_TAGS.has(node.nodeName.toUpperCase()),
    replacement: () => '',
  })
  return td
}

const converter = createConverter()

/** 执行转换；verbose=false 时为实时自动转换，不弹 toast（否则每次输入都提示） */
function runConvert(html: string, verbose: boolean) {
  if (!html.trim()) {
    output.value = ''
    return
  }
  try {
    const markdown = converter.turndown(html)
    output.value = markdown
    if (!verbose) {return}
    if (!markdown.trim()) {
      toast.warning('转换结果为空，请检查 HTML 内容')
    } else {
      toast.success(`转换完成，共 ${markdown.length} 字符`)
    }
  } catch (err: any) {
    if (verbose) {toast.error(`转换失败: ${err.message || String(err)}`)}
  }
}

function convert() {
  if (!htmlInput.value.trim()) {
    toast.warning('请先输入或粘贴 HTML 内容')
    return
  }
  // 手动转换：跳过防抖立即执行（flushInput 会取消待跑的自动任务，不会重复转换）
  syncFromInput(true)
}

// ---------- 工具函数 ----------
function lineCount(s: string): number {
  return s ? s.split('\n').length : 0
}
function formatSize(n: number): string {
  return n >= 1024 ? `${(n / 1024).toFixed(0)} KB` : `${n} 字符`
}

// 去掉 script/style/标签后统计可见文本长度
function extractTextLength(html: string): number {
  const body = html
    .replace(/<script[\s\S]*?<\/script>/g, ' ')
    .replace(/<style[\s\S]*?<\/style>/g, ' ')
    .replace(/<[^>]+>/g, ' ')
  return body.replace(/\s+/g, ' ').trim().length
}

// 判定：HTML 较大但可见文本极少 → SPA 空壳
function isSpaShell(html: string): boolean {
  return html.length > 1500 && extractTextLength(html) < 200
}

async function fetchUrl() {
  const url = urlInput.value.trim()
  if (!url) {
    toast.warning('请输入网址')
    return
  }
  if (!/^https?:\/\/.+/.test(url)) {
    toast.warning('请输入有效的网址（以 http:// 或 https:// 开头）')
    return
  }

  loading.value = true
  loadingText.value = '正在获取页面...'
  try {
    let text = await getTauriAPI().fetchPageContent(url)
    // SPA 空壳（正文靠 JS 渲染）→ 自动降级：开隐藏 WebView 执行 JS 后抓取
    if (isSpaShell(text)) {
      loadingText.value = '检测到 JS 动态渲染页面，正在浏览器内核中渲染抓取（约 5-15 秒）...'
      try {
        text = await getTauriAPI().fetchPageContentJs(url)
      } catch {
        // WebView 抓取失败：保留原始空壳，跳过转换，直接走 SPA 提示引导
        fetchedUrl.value = url
        htmlInput.value = text
        output.value = ''
        spaWarn.value = '该页面正文可能由 JS 动态渲染（SPA）或需要登录，浏览器内核抓取失败。请点击下方按钮在浏览器中打开，全选复制正文后粘贴到下方 HTML 输入框。'
        return
      }
    }
    fetchedUrl.value = url
    htmlInput.value = text
    // 抓取是显式操作：无论实时开关状态都转换一次（flushInput 会取消待跑的自动任务，不会重复）
    syncFromInput(false)
    // SPA 空壳检测：转换结果几乎为空且原始 HTML 是动态渲染壳 → 明确提示
    if (isSpaShell(text) && output.value.trim().length < 80) {
      spaWarn.value = '该页面正文可能由 JS 动态渲染（SPA）或尚未渲染完成，直接抓取只能拿到空壳。请点击下方按钮在浏览器中打开，全选复制正文后粘贴到左侧 HTML 输入框。'
    } else {
      spaWarn.value = ''
    }
  } catch (err: any) {
    toast.error(`获取网页失败: ${err.message || String(err)}`)
    spaWarn.value = ''
  } finally {
    loading.value = false
    loadingText.value = ''
  }
}

// 在系统浏览器中打开抓取成功的网址（SPA 页面提示引导用）
async function openInBrowser() {
  const url = fetchedUrl.value.trim()
  if (!/^https?:\/\/.+/.test(url)) {
    return
  }
  try {
    await openUrl(url)
  } catch (e: any) {
    toast.error(`打开浏览器失败: ${e?.message || String(e)}`)
  }
}

function copyResult() {
  if (!output.value) {
    toast.warning('没有可复制的内容')
    return
  }
  copyText(output.value, toast)
}

function clear() {
  htmlInput.value = ''
  urlInput.value = ''
  output.value = ''
  debouncedHtml.value = ''
  autoSkipped.value = false
  forceHtmlPreview.value = false
  forceMdPreview.value = false
  spaWarn.value = ''
  fetchedUrl.value = ''
}
</script>

<style scoped>
/* 预览渲染样式（v-html 内容用 :deep 穿透；不依赖 NoteManager 是否访问过） */
.markdown-body {
  line-height: 1.7;
  font-size: 14px;
  color: var(--color-base-content);
}
.markdown-body :deep(h1) { font-size: 24px; font-weight: 700; margin: 18px 0 10px; border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); padding-bottom: 6px; }
.markdown-body :deep(h2) { font-size: 20px; font-weight: 600; margin: 16px 0 8px; }
.markdown-body :deep(h3) { font-size: 17px; font-weight: 600; margin: 14px 0 6px; }
.markdown-body :deep(p) { margin: 8px 0; }
.markdown-body :deep(code) { background: var(--color-base-200); padding: 2px 6px; border-radius: 4px; font-family: 'SF Mono', ui-monospace, monospace; font-size: 13px; }
.markdown-body :deep(pre) { background: var(--color-base-200); padding: 14px; border-radius: 8px; overflow-x: auto; margin: 12px 0; }
.markdown-body :deep(pre code) { background: none; padding: 0; }
.markdown-body :deep(blockquote) { border-left: 3px solid var(--color-primary); padding-left: 14px; margin: 12px 0; color: color-mix(in oklab, var(--color-base-content) 65%, transparent); }
.markdown-body :deep(ul), .markdown-body :deep(ol) { padding-left: 24px; margin: 8px 0; }
.markdown-body :deep(li) { margin: 4px 0; }
.markdown-body :deep(a) { color: var(--color-primary); text-decoration: underline; }
.markdown-body :deep(img) { max-width: 100%; border-radius: 8px; margin: 12px 0; }
.markdown-body :deep(table) { border-collapse: collapse; width: 100%; margin: 12px 0; }
.markdown-body :deep(th), .markdown-body :deep(td) { border: 1px solid color-mix(in oklab, var(--color-base-content) 12%, transparent); padding: 8px 12px; text-align: left; }
.markdown-body :deep(th) { background: var(--color-base-200); font-weight: 600; }
/* 代码块复制按钮 */
.markdown-body :deep(.code-block-wrapper) { background: var(--color-base-200); border-radius: 8px; overflow: hidden; margin: 12px 0; }
.markdown-body :deep(.code-block-wrapper pre) { margin: 0; border-radius: 0; background: transparent; }
.markdown-body :deep(.code-header) { display: flex; justify-content: space-between; align-items: center; padding: 6px 12px; background: color-mix(in oklab, var(--color-base-content) 6%, transparent); font-size: 11px; color: var(--color-base-content); }
.markdown-body :deep(.copy-btn) { background: none; border: none; color: var(--color-base-content); opacity: 0.6; cursor: pointer; padding: 2px; display: inline-flex; }
.markdown-body :deep(.copy-btn:hover) { opacity: 1; }
.markdown-body :deep(.copy-btn.copied) { color: var(--color-success, #4ade80); opacity: 1; }
</style>
