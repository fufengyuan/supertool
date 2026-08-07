<template>
  <ToolPage
    icon="notebook"
    name="文本处理"
    description="大小写 / 标点 / 行操作 / 排序去重 / 文本统计，一站式处理"
    @back="$emit('back')"
  >
    <!-- 输入输出 -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
      <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4 min-h-[220px]">
        <div class="flex items-center justify-between mb-2.5">
          <h4 class="text-xs font-semibold text-base-content/70 flex items-center gap-1.5"><SvgIcon name="arrowDown" size="12" /> 输入</h4>
          <button class="btn btn-ghost btn-xs" @click="clearAll" :disabled="!input && !output">全部清空</button>
        </div>
        <textarea v-model="input" class="textarea textarea-bordered w-full font-mono text-sm flex-1 resize-none bg-base-200/60 min-h-[140px]" placeholder="在此输入文本..."></textarea>
      </div>
      <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4 min-h-[220px]">
        <div class="flex items-center justify-between mb-2.5">
          <h4 class="text-xs font-semibold text-base-content/70 flex items-center gap-1.5"><SvgIcon name="arrowUp" size="12" /> 输出</h4>
          <div class="flex gap-1.5">
            <button class="btn btn-ghost btn-xs" @click="useAsInput" :disabled="!output" title="将输出移至输入">→ 输入</button>
            <button class="btn btn-primary btn-xs" @click="copyOutput" :disabled="!output"><SvgIcon name="copy" size="11" /> 复制</button>
          </div>
        </div>
        <textarea v-model="output" readonly class="textarea textarea-bordered w-full font-mono text-sm flex-1 resize-none bg-base-200/60 min-h-[140px]" placeholder="转换结果将显示在这里..."></textarea>
      </div>
    </div>

    <!-- 操作按钮组 -->
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <h4 class="text-xs font-semibold text-base-content/70 mb-2.5 flex items-center gap-1.5"><SvgIcon name="filter" size="12" /> 操作</h4>
      <div class="flex flex-col gap-3">
        <div class="flex flex-wrap gap-1.5">
          <span class="text-[11px] text-base-content/40 leading-7 mr-1 w-14 shrink-0">大小写</span>
          <button class="btn btn-outline btn-xs" @click="toUpper" :disabled="!input">大写</button>
          <button class="btn btn-outline btn-xs" @click="toLower" :disabled="!input">小写</button>
          <button class="btn btn-outline btn-xs" @click="toTitleCase" :disabled="!input">首字母大写</button>
          <button class="btn btn-outline btn-xs" @click="toCamelCase" :disabled="!input">驼峰式</button>
          <button class="btn btn-outline btn-xs" @click="toSnakeCase" :disabled="!input">蛇形命名</button>
          <button class="btn btn-outline btn-xs" @click="toSentenceCase" :disabled="!input">句子首字母</button>
          <button class="btn btn-outline btn-xs" @click="toAlternatingCase" :disabled="!input">交替大小写</button>
          <button class="btn btn-outline btn-xs" @click="toInverseCase" :disabled="!input">反转大小写</button>
        </div>
        <div class="flex flex-wrap gap-1.5">
          <span class="text-[11px] text-base-content/40 leading-7 mr-1 w-14 shrink-0">标点</span>
          <button class="btn btn-outline btn-xs" @click="cnToEnPunct" :disabled="!input">中文标点 → 英文</button>
          <button class="btn btn-outline btn-xs" @click="enToCnPunct" :disabled="!input">英文标点 → 中文</button>
        </div>
        <div class="flex flex-wrap gap-1.5">
          <span class="text-[11px] text-base-content/40 leading-7 mr-1 w-14 shrink-0">排序</span>
          <button class="btn btn-outline btn-xs" @click="dedupLines" :disabled="!input">行去重</button>
          <button class="btn btn-outline btn-xs" @click="sortLines" :disabled="!input">字母排序</button>
          <button class="btn btn-outline btn-xs" @click="sortLinesReverse" :disabled="!input">逆序排序</button>
          <button class="btn btn-outline btn-xs" @click="sortLinesRandom" :disabled="!input">随机排序</button>
          <button class="btn btn-outline btn-xs" @click="sortLinesByLength" :disabled="!input">按长度排序</button>
        </div>
        <div class="flex flex-wrap gap-1.5">
          <span class="text-[11px] text-base-content/40 leading-7 mr-1 w-14 shrink-0">行处理</span>
          <button class="btn btn-outline btn-xs" @click="trimLines" :disabled="!input">去除首尾空格</button>
          <button class="btn btn-outline btn-xs" @click="removeEmptyLines" :disabled="!input">删除空行</button>
          <button class="btn btn-outline btn-xs" @click="addLineNumbers" :disabled="!input">添加行号</button>
          <button class="btn btn-outline btn-xs" @click="removeLineNumbers" :disabled="!input">删除行号</button>
          <button class="btn btn-outline btn-xs" @click="reverseLines" :disabled="!input">行序反转</button>
          <button class="btn btn-outline btn-xs" @click="mergeLines" :disabled="!input">合并单行</button>
          <button class="btn btn-outline btn-xs" @click="splitByComma" :disabled="!input">逗号分行</button>
        </div>
        <div class="flex flex-wrap gap-1.5">
          <span class="text-[11px] text-base-content/40 leading-7 mr-1 w-14 shrink-0">其他</span>
          <button class="btn btn-outline btn-xs" @click="reverseText" :disabled="!input">文本反转</button>
          <button class="btn btn-outline btn-xs" @click="uniqueWords" :disabled="!input">词去重</button>
        </div>
      </div>
    </div>

    <!-- 统计 -->
    <div v-if="stats" class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3">
      <div v-for="s in statRows" :key="s.label" class="flex flex-col items-center gap-1 p-3.5 bg-base-100 border border-base-content/10 rounded-xl">
        <span class="text-xl font-bold text-primary">{{ s.value }}</span>
        <span class="text-xs text-base-content/50">{{ s.label }}</span>
      </div>
    </div>
  </ToolPage>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { ref, computed } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

const toast = useToast()

const input = ref('')
const output = ref('')

function transform(fn: (text: string) => string) {
  if (!input.value.trim()) {
    toast.warning('请输入文本')
    return
  }
  output.value = fn(input.value)
}

/* ─── Case Conversion ─── */
function toUpper() {
  transform(t => t.toUpperCase())
}

function toLower() {
  transform(t => t.toLowerCase())
}

function toTitleCase() {
  transform(t => {
    return t.replace(/\b\w/g, c => c.toUpperCase())
  })
}

function toCamelCase() {
  transform(t => {
    const words = t.trim().split(/[\s_-]+/)
    return words.map((w, i) => {
      if (i === 0) {return w.toLowerCase()}
      return w.charAt(0).toUpperCase() + w.slice(1).toLowerCase()
    }).join('')
  })
}

function toSnakeCase() {
  transform(t => {
    return t
      .replace(/([a-z])([A-Z])/g, '$1_$2')
      .replace(/[\s-]+/g, '_')
      .toLowerCase()
  })
}

function toSentenceCase() {
  transform(t => {
    return t.toLowerCase().replace(/(^\s*|[.!?]\s+)([a-z])/g, (match, p1, p2) => p1 + p2.toUpperCase())
  })
}

function toAlternatingCase() {
  transform(t => {
    return t.split('').map((c, i) => i % 2 === 0 ? c.toLowerCase() : c.toUpperCase()).join('')
  })
}

function toInverseCase() {
  transform(t => {
    return t.split('').map(c => {
      if (c === c.toUpperCase()) {return c.toLowerCase()}
      return c.toUpperCase()
    }).join('')
  })
}

/* ─── Punctuation ─── */
const cnPunctMap: [RegExp, string][] = [
  [/，/g, ','], [/。/g, '.'], [/！/g, '!'], [/？/g, '?'],
  [/；/g, ';'], [/：/g, ':'], [/“/g, '"'], [/”/g, '"'],
  [/‘/g, "'"], [/’/g, "'"], [/（/g, '('], [/）/g, ')'],
  [/【/g, '['], [/】/g, ']'], [/《/g, '<'], [/》/g, '>'],
  [/……/g, '...'], [/—/g, '-'], [/·/g, '.'],
]

function cnToEnPunct() {
  transform(t => {
    let result = t
    for (const [re, val] of cnPunctMap) {
      result = result.replace(re, val)
    }
    return result
  })
}

function enToCnPunct() {
  transform(t => {
    let result = t
    const enToCn: [RegExp, string][] = [
      [/,/g, '，'], [/\./g, '。'], [/!/g, '！'], [/\?/g, '？'],
      [/;/g, '；'], [/:/g, '：'], [/"/g, '"'], [/'/g, "'"],
      [/\(/g, '（'], [/\)/g, '）'], [/\[/g, '【'], [/\]/g, '】'],
      [/</g, '《'], [/>/g, '》'],
    ]
    for (const [re, val] of enToCn) {
      result = result.replace(re, val)
    }
    return result
  })
}

/* ─── Line Operations ─── */
function dedupLines() {
  transform(t => {
    const lines = t.split('\n')
    return [...new Set(lines)].join('\n')
  })
}

function sortLines() {
  transform(t => t.split('\n').sort().join('\n'))
}

function sortLinesReverse() {
  transform(t => t.split('\n').sort().reverse().join('\n'))
}

function sortLinesRandom() {
  transform(t => {
    const lines = t.split('\n')
    for (let i = lines.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1))
      ;[lines[i], lines[j]] = [lines[j], lines[i]]
    }
    return lines.join('\n')
  })
}

function sortLinesByLength() {
  transform(t => t.split('\n').sort((a, b) => a.length - b.length).join('\n'))
}

function trimLines() {
  transform(t => t.split('\n').map(l => l.trim()).join('\n'))
}

function removeEmptyLines() {
  transform(t => t.split('\n').filter(l => l.trim()).join('\n'))
}

function addLineNumbers() {
  transform(t => {
    return t.split('\n').map((l, i) => `${String(i + 1).padStart(4, ' ')}  ${l}`).join('\n')
  })
}

function removeLineNumbers() {
  transform(t => {
    return t.split('\n').map(l => l.replace(/^\s*\d+\s+/, '')).join('\n')
  })
}

function reverseText() {
  transform(t => t.split('').reverse().join(''))
}

function reverseLines() {
  transform(t => t.split('\n').reverse().join('\n'))
}

function mergeLines() {
  transform(t => t.split('\n').map(l => l.trim()).filter(Boolean).join(' '))
}

function splitByComma() {
  transform(t => t.replace(/,\s*/g, '\n'))
}

function uniqueWords() {
  transform(t => {
    const words = t.split(/\s+/).filter(Boolean)
    return [...new Set(words)].join(' ')
  })
}

/* ─── Statistics ─── */
const stats = computed(() => {
  if (!input.value) {return null}

  const text = input.value
  return {
    chars: text.length,
    noSpaceChars: text.replace(/\s/g, '').length,
    words: text.split(/\s+/).filter(Boolean).length,
    lines: text.split('\n').length,
    bytes: new Blob([text]).size,
    chineseChars: (text.match(/[\u4e00-\u9fff]/g) || []).length,
  }
})

const statRows = computed(() => {
  const s = stats.value
  if (!s) { return [] }
  return [
    { label: '字符数', value: s.chars },
    { label: '无空格字符', value: s.noSpaceChars },
    { label: '单词数', value: s.words },
    { label: '行数', value: s.lines },
    { label: '字节数', value: s.bytes },
    { label: '中文字符', value: s.chineseChars },
  ]
})

async function copyOutput() {
  if (!output.value) {return}
  await copyText(output.value, toast)
}

function useAsInput() {
  if (!output.value) {return}
  input.value = output.value
  output.value = ''
}

function clearAll() {
  input.value = ''
  output.value = ''
}
</script>
