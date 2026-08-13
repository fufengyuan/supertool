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
        <span class="text-[11px] text-base-content/40">{{ output.length }} 字符 · {{ output.split('\n').length }} 行</span>
      </div>
      <div class="p-3 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-xs text-base-content whitespace-pre-wrap break-all max-h-[480px] overflow-y-auto">{{ output }}</div>
    </div>
    <div v-else class="py-14 text-center text-xs text-base-content/40 bg-base-100/50 border border-dashed border-base-content/15 rounded-xl">
      Markdown 结果将显示在这里
    </div>
  </ToolPage>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { ref } from 'vue'
import TurndownService from 'turndown'
import { gfm } from 'turndown-plugin-gfm'
import { copyText } from '../toolUtils'
import { getTauriAPI } from '../../../utils/tauri-api'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

const toast = useToast()
const htmlInput = ref('')
const urlInput = ref('')
const output = ref('')
const loading = ref(false)

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
  try {
    const text = await getTauriAPI().fetchPageContent(url)
    htmlInput.value = text
    await doConvert(text)
  } catch (err: any) {
    toast.error(`获取网页失败: ${err.message || String(err)}`)
  } finally {
    loading.value = false
  }
}

async function convert() {
  if (!validateHtml(htmlInput.value)) {
    toast.warning('请粘贴 HTML 内容')
    return
  }
  await doConvert(htmlInput.value)
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
}
</script>
