<template>
  <div>
    <h3 class="text-lg font-bold text-base-content mb-5">文本处理</h3>

    <div class="mb-5">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">输入文本</h4>
      <textarea v-model="input" class="textarea textarea-bordered w-full font-mono text-sm min-h-[120px]" placeholder="在此输入文本..." rows="6"></textarea>
    </div>

    <!-- Case Conversion -->
    <div class="mb-5">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">大小写转换</h4>
      <div class="flex flex-wrap gap-2.5 mb-3">
        <button class="btn btn-ghost" @click="toUpper" :disabled="!input">大写 UPPER</button>
        <button class="btn btn-ghost" @click="toLower" :disabled="!input">小写 lower</button>
        <button class="btn btn-ghost" @click="toTitleCase" :disabled="!input">首字母大写</button>
        <button class="btn btn-ghost" @click="toCamelCase" :disabled="!input">驼峰式</button>
        <button class="btn btn-ghost" @click="toSnakeCase" :disabled="!input">蛇形命名</button>
        <button class="btn btn-ghost" @click="toSentenceCase" :disabled="!input">句子首字母大写</button>
        <button class="btn btn-ghost" @click="toAlternatingCase" :disabled="!input">交替大小写</button>
        <button class="btn btn-ghost" @click="toInverseCase" :disabled="!input">反转大小写</button>
      </div>
    </div>

    <!-- Punctuation -->
    <div class="mb-5">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">标点符号转换</h4>
      <div class="flex flex-wrap gap-2.5 mb-3">
        <button class="btn btn-ghost" @click="cnToEnPunct" :disabled="!input">中文标点 → 英文</button>
        <button class="btn btn-ghost" @click="enToCnPunct" :disabled="!input">英文标点 → 中文</button>
      </div>
    </div>

    <!-- Text Operations -->
    <div class="mb-5">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">文本操作</h4>
      <div class="flex flex-wrap gap-2.5 mb-3">
        <button class="btn btn-ghost" @click="dedupLines" :disabled="!input">行去重</button>
        <button class="btn btn-ghost" @click="sortLines" :disabled="!input">字母排序</button>
        <button class="btn btn-ghost" @click="sortLinesReverse" :disabled="!input">逆序排序</button>
        <button class="btn btn-ghost" @click="sortLinesRandom" :disabled="!input">随机排序</button>
        <button class="btn btn-ghost" @click="sortLinesByLength" :disabled="!input">按长度排序</button>
      </div>
      <div class="flex flex-wrap gap-2.5 mb-3">
        <button class="btn btn-ghost" @click="trimLines" :disabled="!input">去除首尾空格</button>
        <button class="btn btn-ghost" @click="removeEmptyLines" :disabled="!input">删除空行</button>
        <button class="btn btn-ghost" @click="addLineNumbers" :disabled="!input">添加行号</button>
        <button class="btn btn-ghost" @click="removeLineNumbers" :disabled="!input">删除行号</button>
        <button class="btn btn-ghost" @click="reverseText" :disabled="!input">文本反转</button>
        <button class="btn btn-ghost" @click="reverseLines" :disabled="!input">行序反转</button>
      </div>
      <div class="flex flex-wrap gap-2.5 mb-3">
        <button class="btn btn-ghost" @click="mergeLines" :disabled="!input">合并为单行</button>
        <button class="btn btn-ghost" @click="splitByComma" :disabled="!input">逗号分行</button>
        <button class="btn btn-ghost" @click="uniqueWords" :disabled="!input">词去重</button>
      </div>
    </div>

    <div class="flex flex-wrap gap-2.5 mb-3">
      <button class="btn btn-primary" @click="copyOutput" :disabled="!input">复制结果</button>
      <button class="btn btn-ghost" @click="clearAll">清空</button>
    </div>

    <!-- Statistics -->
    <div class="mb-5" v-if="stats">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">文本统计</h4>
      <div class="grid grid-cols-[repeat(auto-fit,minmax(100px,1fr))] gap-3 p-3 bg-base-200 border border-base-content/10 rounded-box">
        <div class="flex flex-col items-center gap-1">
          <span class="text-xl font-bold text-primary">{{ stats.chars }}</span>
          <span class="text-xs opacity-60">字符数</span>
        </div>
        <div class="flex flex-col items-center gap-1">
          <span class="text-xl font-bold text-primary">{{ stats.noSpaceChars }}</span>
          <span class="text-xs opacity-60">无空格字符</span>
        </div>
        <div class="flex flex-col items-center gap-1">
          <span class="text-xl font-bold text-primary">{{ stats.words }}</span>
          <span class="text-xs opacity-60">单词数</span>
        </div>
        <div class="flex flex-col items-center gap-1">
          <span class="text-xl font-bold text-primary">{{ stats.lines }}</span>
          <span class="text-xs opacity-60">行数</span>
        </div>
        <div class="flex flex-col items-center gap-1">
          <span class="text-xl font-bold text-primary">{{ stats.bytes }}</span>
          <span class="text-xs opacity-60">字节数</span>
        </div>
        <div class="flex flex-col items-center gap-1">
          <span class="text-xl font-bold text-primary">{{ stats.chineseChars }}</span>
          <span class="text-xs opacity-60">中文字符</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref, computed } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const toast = useToast()

const input = ref('')

function transform(fn: (text: string) => string) {
  if (!input.value.trim()) {
    toast.warning('请输入文本')
    return
  }
  input.value = fn(input.value)
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
      if (i === 0) return w.toLowerCase()
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
      if (c === c.toUpperCase()) return c.toLowerCase()
      return c.toUpperCase()
    }).join('')
  })
}

/* ─── Punctuation ─── */
const cnPunctMap: [RegExp, string][] = [
  [/，/g, ','], [/。/g, '.'], [/！/g, '!'], [/？/g, '?'],
  [/；/g, ';'], [/：/g, ':'], [/"'/g, '"'], [/"'/g, '"'],
  [/'/g, "'"], [/'/g, "'"], [/（/g, '('], [/）/g, ')'],
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
  if (!input.value) return null

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

async function copyOutput() {
  if (!input.value) return
  await copyText(input.value, toast)
}

function clearAll() {
  input.value = ''
}
</script>
