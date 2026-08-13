<template>
  <ToolPage
    icon="download"
    name="HTML 转 Markdown"
    description="输入网址自动抓取，或直接粘贴 HTML，一键转换为 Markdown"
    @back="$emit('back')"
  >
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <h4 class="text-xs font-semibold text-base-content/70 mb-3 flex items-center gap-1.5"><SvgIcon name="globe" size="12" /> 输入网址或直接粘贴 HTML</h4>
      <div class="flex items-center gap-2 mb-3">
        <div class="flex-1 relative">
          <SvgIcon name="link" size="14" class="absolute left-[10px] top-1/2 -translate-y-1/2 text-base-content/60 pointer-events-none" />
          <input
            v-model="urlInput"
            placeholder="输入网址（如 https://example.com）..."
            class="w-full py-2 pl-8 pr-3 border border-base-content/10 rounded-lg text-xs bg-base-200/60 text-base-content outline-none focus:border-primary"
            @keyup.enter="fetchUrl"
          />
        </div>
        <button class="btn btn-primary btn-sm gap-1.5" @click="fetchUrl" :disabled="loading">
          <span v-if="loading" class="loading loading-spinner loading-xs"></span>
          <span v-else><SvgIcon name="globe" size="13" /></span>
          {{ loading ? '获取中...' : '获取并转换' }}
        </button>
      </div>

      <!-- 加载进度（SPA 页面 WebView 渲染抓取时提示等待） -->
      <div v-if="loading" class="flex items-center gap-1.5 mb-3 text-[11px] text-base-content/50">
        <span class="loading loading-spinner loading-xs"></span>{{ loadingText }}
      </div>

      <!-- SPA 动态渲染页面提示 -->
      <div v-if="spaWarn" class="flex items-start gap-2 mb-3 px-3 py-2 bg-warning/10 border border-warning/30 rounded-lg text-xs text-warning">
        <SvgIcon name="alertTriangle" size="14" class="mt-0.5 shrink-0" />
        <div class="flex-1">
          <p class="leading-relaxed">{{ spaWarn }}</p>
          <button class="btn btn-xs btn-outline mt-1.5" @click="openInBrowser"><SvgIcon name="globe" size="11" /> 在浏览器中打开</button>
        </div>
      </div>

      <div class="flex items-center gap-3 mb-3">
        <div class="flex-1 h-px bg-base-content/10"></div>
        <span class="text-[11px] text-base-content/40">或者直接粘贴 HTML</span>
        <div class="flex-1 h-px bg-base-content/10"></div>
      </div>

      <textarea
        v-model="htmlInput"
        class="textarea textarea-bordered w-full text-xs bg-base-200/60 font-mono min-h-[140px] resize-none"
        placeholder="粘贴 HTML 代码到这里..."
      ></textarea>

      <div class="flex flex-wrap gap-2 mt-3">
        <button class="btn btn-primary btn-sm gap-1.5" @click="convert" :disabled="loading">
          <span v-if="loading" class="loading loading-spinner loading-xs"></span>
          <SvgIcon name="check" size="12" v-else />
          转换为 Markdown
        </button>
        <button class="btn btn-outline btn-sm gap-1.5" @click="copyResult" :disabled="!output">
          <SvgIcon name="copy" size="12" /> 复制结果
        </button>
        <button class="btn btn-ghost btn-sm gap-1.5" @click="clear" :disabled="!htmlInput && !output">
          <SvgIcon name="trash" size="12" /> 清空
        </button>
      </div>
    </div>

    <div v-if="output" class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <div class="flex items-center justify-between mb-2.5">
        <h4 class="text-xs font-semibold text-base-content/70 flex items-center gap-1.5"><SvgIcon name="fileText" size="12" /> Markdown 输出</h4>
        <div class="flex items-center gap-2">
          <div class="join">
            <button class="btn btn-xs join-item" :class="viewMode === 'md' ? 'btn-primary' : 'btn-ghost'" @click="viewMode = 'md'">Markdown</button>
            <button class="btn btn-xs join-item" :class="viewMode === 'preview' ? 'btn-primary' : 'btn-ghost'" @click="viewMode = 'preview'">预览</button>
          </div>
          <span class="text-[11px] text-base-content/40">{{ output.length }} 字符 · {{ output.split('\n').length }} 行</span>
        </div>
      </div>
      <div v-if="viewMode === 'md'" class="p-3 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-xs text-base-content whitespace-pre-wrap break-all max-h-[480px] overflow-y-auto">{{ output }}</div>
      <div v-else class="markdown-body p-3 bg-base-200/60 border border-base-content/10 rounded-lg max-h-[480px] overflow-y-auto" v-html="previewHtml" @click="onPreviewClick"></div>
    </div>
    <div v-else class="py-14 text-center text-xs text-base-content/40 bg-base-100/50 border border-dashed border-base-content/15 rounded-xl">
      Markdown 结果将显示在这里
    </div>
  </ToolPage>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { ref, computed } from 'vue'
import TurndownService from 'turndown'
import { gfm } from 'turndown-plugin-gfm'
import { renderMarkdown, setupCopyCode } from '../../../composables/useMarkdownRenderer'
import { copyText } from '../toolUtils'
import { getTauriAPI } from '../../../utils/tauri-api'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

const toast = useToast()
const htmlInput = ref('')
const urlInput = ref('')
const output = ref('')
const loading = ref(false)
// 加载进度文案（SPA 页面降级 WebView 抓取时显示）
const loadingText = ref('')
// 输出视图：Markdown 源码 / 渲染预览
const viewMode = ref<'md' | 'preview'>('md')

// SPA 动态渲染页面检测：抓到的 HTML 是空壳（正文靠 JS 渲染，reqwest 拿不到）
const spaWarn = ref('')
// 最近一次抓取成功的 URL（openInBrowser 用它，避免用户改输入框后打开无关地址）
const fetchedUrl = ref('')

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

// 渲染预览（marked + DOMPurify 消毒 + 代码高亮）
const previewHtml = computed(() => (output.value ? renderMarkdown(output.value) : ''))

// 复制按钮事件委托（setupCopyCode 返回 handler，绑到预览容器 click）
const onPreviewClick = setupCopyCode()

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

function validateHtml(html: string): boolean {
  return html.trim().length > 0
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
    await doConvert(text)
    // SPA 空壳检测：转换结果几乎为空且原始 HTML 是动态渲染壳 → 明确提示
    if (isSpaShell(text) && output.value.trim().length < 80) {
      spaWarn.value = '该页面正文可能由 JS 动态渲染（SPA）或尚未渲染完成，直接抓取只能拿到空壳。请点击下方按钮在浏览器中打开，全选复制正文后粘贴到下方 HTML 输入框。'
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

async function convert() {
  if (!validateHtml(htmlInput.value)) {
    toast.warning('请粘贴 HTML 内容')
    return
  }
  await doConvert(htmlInput.value)
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

async function doConvert(html: string) {
  try {
    const markdown = converter.turndown(html)
    output.value = markdown
    if (!markdown.trim()) {
      toast.warning('转换结果为空，请检查 HTML 内容')
    } else {
      toast.success(`转换完成，共 ${markdown.length} 字符`)
    }
  } catch (err: any) {
    toast.error(`转换失败: ${err.message || String(err)}`)
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
  viewMode.value = 'md'
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
