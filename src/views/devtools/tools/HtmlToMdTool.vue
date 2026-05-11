<template>
  <div class="max-w-[800px]">
    <h3 class="text-lg font-bold text-base-content mb-5"><SvgIcon name="download" size="14" class="align-text-bottom" /> HTML 转 Markdown</h3>

    <div class="mb-4">
      <label class="label-text text-xs text-base-content/60 mb-1 block">输入网址或直接粘贴 HTML</label>

      <!-- URL Input -->
      <div class="flex items-center gap-2 mb-3">
        <div class="flex-1 relative">
          <SvgIcon name="link" size="14" class="absolute left-[10px] top-1/2 -translate-y-1/2 text-base-content/60 pointer-events-none" />
          <input
            v-model="urlInput"
            placeholder="输入网址（如 https://example.com）..."
            class="w-full py-[7px] pl-[30px] pr-[10px] border border-base-content/10 rounded-md text-xs bg-base-200 text-base-content outline-none focus:border-primary"
            @keyup.enter="fetchUrl"
          />
        </div>
        <button class="btn btn-primary btn-sm gap-1.5" @click="fetchUrl" :disabled="loading">
          <span v-if="loading" class="loading loading-spinner loading-xs"></span>
          <span v-else><SvgIcon name="globe" size="14" class="align-text-bottom" /></span>
          {{ loading ? '获取中...' : '获取并转换' }}
        </button>
      </div>

      <!-- OR separator -->
      <div class="flex items-center gap-3 mb-3">
        <div class="flex-1 h-px bg-base-content/10"></div>
        <span class="text-xs text-base-content/40">或者直接粘贴 HTML</span>
        <div class="flex-1 h-px bg-base-content/10"></div>
      </div>

      <!-- HTML Input -->
      <textarea
        v-model="htmlInput"
        class="textarea textarea-bordered w-full text-xs bg-base-200 font-mono min-h-[150px]"
        placeholder="粘贴 HTML 代码到这里..."
        rows="6"
      ></textarea>

      <!-- Action Buttons -->
      <div class="flex flex-wrap gap-2.5 mt-3">
        <button class="btn btn-primary btn-sm gap-1.5" @click="convert" :disabled="loading">
          <span v-if="loading" class="loading loading-spinner loading-xs"></span>
          <SvgIcon name="check" size="14" class="align-text-bottom" v-else />
          转换为 Markdown
        </button>
        <button class="btn btn-ghost btn-sm gap-1.5" @click="copyResult" :disabled="!output">
          <SvgIcon name="file" size="14" class="align-text-bottom" /> 复制结果
        </button>
        <button class="btn btn-ghost btn-sm gap-1.5" @click="clear">
          <SvgIcon name="trash" size="14" class="align-text-bottom" /> 清空
        </button>
      </div>
    </div>

    <!-- Markdown Output -->
    <label class="label-text text-xs text-base-content/60 mb-1 block">Markdown 输出</label>
    <div
      class="mt-2 p-3 bg-base-200 border border-base-content/10 rounded-box font-mono text-xs text-base-content whitespace-pre-wrap break-all max-h-[500px] overflow-y-auto"
    >{{ output || 'Markdown 结果将显示在这里...' }}</div>

    <!-- Char/Line Stats -->
    <div v-if="output" class="mt-2 text-[11px] text-base-content/40 flex gap-4">
      <span>{{ output.length }} 字符</span>
      <span>{{ output.split('\n').length }} 行</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref } from 'vue'
import { useToast } from '@/composables/useToast'
import { copyText } from '../toolUtils'
import { getTauriAPI } from '../../../utils/tauri-api'

const toast = useToast()
const htmlInput = ref('')
const urlInput = ref('')
const output = ref('')
const loading = ref(false)

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
    const markdown = await getTauriAPI().convertHtmlToMd(html)
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
