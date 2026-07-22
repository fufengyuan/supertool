<template>
  <div class="flex flex-col h-full">
    <h3 class="text-lg font-bold text-base-content mb-4">正则表达式</h3>

    <!-- 正则输入 -->
    <div class="mb-4">
      <div class="flex flex-wrap gap-2 mb-2">
        <input
          v-model="pattern"
          class="input input-bordered flex-1 font-mono text-sm"
          placeholder="输入正则表达式，如 \d+"
          @input="runTest"
        />
        <div class="join">
          <button v-for="f in flagOptions" :key="f" class="btn btn-ghost join-item btn-sm" :class="{ 'btn-active': flags.includes(f) }" @click="toggleFlag(f)" :title="flagDescriptions[f]">{{ f }}</button>
        </div>
      </div>
      <div v-if="pattern" class="mt-1 text-xs opacity-60">
        正则: <code class="bg-base-200 px-1.5 py-0.5 rounded font-mono">/{{ pattern }}/{{ flags }}</code>
        <span v-if="regexError" class="text-error"> <SvgIcon name="alertTriangle" size="14" class="align-text-bottom" /> {{ regexError }}</span>
      </div>
    </div>

    <!-- 常用正则预设 -->
    <div class="mb-4">
      <div class="text-xs text-base-content/60 mb-1.5">常用正则：</div>
      <div class="flex flex-wrap gap-1.5">
        <button v-for="preset in commonRegex" :key="preset.name" class="btn btn-ghost btn-xs" @click="applyPreset(preset)" :title="preset.pattern">{{ preset.name }}</button>
      </div>
    </div>

    <!-- 测试文本 -->
    <div class="mb-4 flex flex-col flex-1 min-h-0">
      <h4 class="text-sm font-semibold text-base-content mb-1.5">测试文本</h4>
      <textarea v-model="testText" class="textarea textarea-bordered w-full font-mono text-sm flex-1 min-h-[120px] resize-none" placeholder="输入测试文本..." @input="runTest"></textarea>
    </div>

    <!-- 匹配结果 -->
    <div v-if="matchCount !== null" class="mb-4">
      <h4 class="text-sm font-semibold text-base-content mb-2">匹配结果（{{ matchCount }} 个）</h4>
      <div v-if="matches.length > 0" class="bg-base-200 border border-base-content/10 rounded-box max-h-60 overflow-y-auto">
        <div v-for="(m, idx) in matches" :key="idx" class="px-3 py-1.5 border-b border-base-content/10 last:border-b-0 text-sm hover:bg-base-100/50">
          <div class="flex items-center gap-2">
            <span class="opacity-60 font-semibold min-w-[36px] text-xs">#{{ idx + 1 }}</span>
            <span class="font-mono break-all flex-1">{{ m.match }}</span>
            <span class="text-xs text-base-content/40 whitespace-nowrap">{{ m.line }}:{{ m.col }}</span>
          </div>
          <div v-if="m.groups.length > 0" class="ml-12 mt-0.5 flex flex-wrap gap-1.5">
            <span v-for="(g, gi) in m.groups" :key="gi" class="text-xs bg-base-100/60 px-1.5 py-0.5 rounded font-mono">
              <span class="opacity-50">${{ gi + 1 }}:</span> {{ g }}
            </span>
          </div>
        </div>
      </div>
      <div v-else class="text-sm text-base-content/60 italic px-3 py-4 text-center bg-base-200 rounded-box">没有匹配结果</div>
    </div>

    <hr v-if="matchCount !== null" class="border-base-content/10 my-4" />

    <!-- 替换 -->
    <div class="mb-4">
      <h4 class="text-sm font-semibold text-base-content mb-2">替换</h4>
      <div class="flex flex-wrap gap-2 mb-2">
        <input v-model="replacement" class="input input-bordered flex-1 font-mono text-sm" placeholder="替换内容，支持 $1, $2 等引用" @input="runReplace" />
        <button class="btn btn-ghost btn-xs" @click="copyReplaced" :disabled="!replacedResult">复制</button>
      </div>
      <textarea v-if="replacedResult !== null" v-model="replacedResult" class="textarea textarea-bordered w-full font-mono text-sm min-h-[80px] resize-none" readonly></textarea>
    </div>

    <hr class="border-base-content/10 my-4" />

    <!-- 匹配高亮 -->
    <div v-if="highlightedHtml" class="mb-4">
      <h4 class="text-sm font-semibold text-base-content mb-2">匹配高亮</h4>
      <div class="p-3 bg-base-200 border border-base-content/10 rounded-box font-mono text-sm leading-relaxed whitespace-pre-wrap break-all max-h-60 overflow-y-auto" v-html="highlightedHtml"></div>
    </div>

    <hr v-if="highlightedHtml" class="border-base-content/10 my-4" />

    <!-- 语法速查 -->
    <details class="collapse collapse-arrow bg-base-200 rounded-box">
      <summary class="collapse-title text-sm font-semibold py-2 min-h-0">正则语法速查</summary>
      <div class="collapse-content text-xs">
        <div class="grid grid-cols-2 gap-x-4 gap-y-1 font-mono">
          <div><code class="text-primary">.</code> 任意字符（不含换行）</div>
          <div><code class="text-primary">\d</code> 数字 [0-9]</div>
          <div><code class="text-primary">\w</code> 单词字符 [a-zA-Z0-9_]</div>
          <div><code class="text-primary">\s</code> 空白字符</div>
          <div><code class="text-primary">\D \W \S</code> 上述反义</div>
          <div><code class="text-primary">^ $</code> 行首/行尾</div>
          <div><code class="text-primary">\b</code> 单词边界</div>
          <div><code class="text-primary">[abc]</code> 字符集</div>
          <div><code class="text-primary">[^abc]</code> 排除字符集</div>
          <div><code class="text-primary">[a-z]</code> 范围</div>
          <div><code class="text-primary">*</code> 0次或多次</div>
          <div><code class="text-primary">+</code> 1次或多次</div>
          <div><code class="text-primary">?</code> 0次或1次</div>
          <div><code class="text-primary">{n}</code> 恰好n次</div>
          <div><code class="text-primary">{n,}</code> 至少n次</div>
          <div><code class="text-primary">{n,m}</code> n到m次</div>
          <div><code class="text-primary">()</code> 捕获组</div>
          <div><code class="text-primary">(?:)</code> 非捕获组</div>
          <div><code class="text-primary">(?=)</code> 正向前瞻</div>
          <div><code class="text-primary">(?!)</code> 负向前瞻</div>
        </div>
      </div>
    </details>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref, computed } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const toast = useToast()

const pattern = ref('')
const flags = ref('g')
const testText = ref('')
const replacement = ref('')
const replacedResult = ref<string | null>(null)
const regexError = ref('')

const flagOptions = ['g', 'i', 'm', 's', 'u']
const flagDescriptions: Record<string, string> = {
  g: '全局匹配',
  i: '忽略大小写',
  m: '多行模式',
  s: '. 匹配换行符',
  u: 'Unicode 模式',
}

const commonRegex = [
  { name: '邮箱', pattern: '[\\w.+-]+@[\\w-]+\\.[\\w.-]+', flags: 'g' },
  { name: '手机号', pattern: '1[3-9]\\d{9}', flags: 'g' },
  { name: 'URL', pattern: 'https?://[\\w\\-._~:/?#\\[\\]@!$&\'()*+,;=%]+', flags: 'g' },
  { name: 'IPv4', pattern: '\\b(?:\\d{1,3}\\.){3}\\d{1,3}\\b', flags: 'g' },
  { name: '身份证', pattern: '\\d{17}[\\dXx]', flags: 'g' },
  { name: '日期', pattern: '\\d{4}[-/]\\d{2}[-/]\\d{2}', flags: 'g' },
  { name: '时间', pattern: '\\d{2}:\\d{2}(:\\d{2})?', flags: 'g' },
  { name: '数字', pattern: '-?\\d+(\\.\\d+)?', flags: 'g' },
  { name: '中文字符', pattern: '[\\u4e00-\\u9fa5]+', flags: 'g' },
  { name: 'QQ号', pattern: '[1-9]\\d{4,10}', flags: 'g' },
]

interface MatchResult {
  match: string
  groups: string[]
  line: number
  col: number
}

const matchCount = ref<number | null>(null)
const matches = ref<MatchResult[]>([])

function toggleFlag(flag: string) {
  if (flags.value.includes(flag)) {
    flags.value = flags.value.replace(flag, '')
  } else {
    flags.value += flag
  }
  runTest()
}

function createRegex(): RegExp | null {
  if (!pattern.value) return null
  try {
    regexError.value = ''
    return new RegExp(pattern.value, flags.value)
  } catch (e: any) {
    regexError.value = e.message
    return null
  }
}

function getLineCol(text: string, index: number): { line: number; col: number } {
  const textBefore = text.substring(0, index)
  const line = (textBefore.match(/\n/g) || []).length + 1
  const col = index - textBefore.lastIndexOf('\n')
  return { line, col }
}

function runTest() {
  const regex = createRegex()
  if (!regex || !testText.value) {
    matchCount.value = null
    matches.value = []
    return
  }

  try {
    const results: MatchResult[] = []

    if (flags.value.includes('g')) {
      const regex2 = new RegExp(regex.source, regex.flags)
      let m: RegExpExecArray | null
      while ((m = regex2.exec(testText.value)) !== null) {
        const { line, col } = getLineCol(testText.value, m.index)
        results.push({
          match: m[0],
          groups: m.length > 1 ? m.slice(1).map(g => g ?? '') : [],
          line,
          col,
        })
        if (m[0] === '') regex2.lastIndex++ // Avoid infinite loop
      }
    } else {
      const m = regex.exec(testText.value)
      if (m) {
        const { line, col } = getLineCol(testText.value, m.index)
        results.push({
          match: m[0],
          groups: m.length > 1 ? m.slice(1).map(g => g ?? '') : [],
          line,
          col,
        })
      }
    }

    matchCount.value = results.length
    matches.value = results
  } catch {
    matchCount.value = 0
    matches.value = []
  }

  runReplace()
}

function runReplace() {
  const regex = createRegex()
  if (!regex || !testText.value) {
    replacedResult.value = null
    return
  }
  try {
    replacedResult.value = testText.value.replace(regex, replacement.value)
  } catch {
    replacedResult.value = null
  }
}

function applyPreset(preset: typeof commonRegex[0]) {
  pattern.value = preset.pattern
  flags.value = preset.flags
  runTest()
}

const highlightedHtml = computed(() => {
  const regex = createRegex()
  if (!regex || !testText.value) return ''

  try {
    let result = ''
    let lastIndex = 0

    if (flags.value.includes('g')) {
      const regex2 = new RegExp(regex.source, regex.flags)
      let m: RegExpExecArray | null
      while ((m = regex2.exec(testText.value)) !== null) {
        if (m[0] === '') {
          regex2.lastIndex++
          continue
        }
        result += escapeHtml(testText.value.substring(lastIndex, m.index))
        result += `<mark class="bg-yellow-300/30 text-yellow-800 dark:text-yellow-200 px-0.5 rounded">${escapeHtml(m[0])}</mark>`
        lastIndex = m.index + m[0].length
      }
    } else {
      const m = regex.exec(testText.value)
      if (m) {
        result += escapeHtml(testText.value.substring(0, m.index))
        result += `<mark class="bg-yellow-300/30 text-yellow-800 dark:text-yellow-200 px-0.5 rounded">${escapeHtml(m[0])}</mark>`
        lastIndex = m.index + m[0].length
      }
    }

    result += escapeHtml(testText.value.substring(lastIndex))
    return result
  } catch {
    return ''
  }
})

function escapeHtml(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
}

async function copyReplaced() {
  if (!replacedResult.value) return
  await copyText(replacedResult.value, toast)
}
</script>
