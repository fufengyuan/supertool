<template>
  <div class="text-tool">
    <h3>文本处理</h3>

    <div class="tool-section">
      <h4>输入文本</h4>
      <textarea v-model="input" class="tool-textarea" placeholder="在此输入文本..." rows="6"></textarea>
    </div>

    <!-- Case Conversion -->
    <div class="tool-section">
      <h4>大小写转换</h4>
      <div class="tool-row">
        <button class="tool-btn" @click="toUpper" :disabled="!input">大写 UPPER</button>
        <button class="tool-btn" @click="toLower" :disabled="!input">小写 lower</button>
        <button class="tool-btn" @click="toTitleCase" :disabled="!input">首字母大写</button>
        <button class="tool-btn" @click="toCamelCase" :disabled="!input">驼峰式</button>
        <button class="tool-btn" @click="toSnakeCase" :disabled="!input">蛇形命名</button>
        <button class="tool-btn" @click="toSentenceCase" :disabled="!input">句子首字母大写</button>
        <button class="tool-btn" @click="toAlternatingCase" :disabled="!input">交替大小写</button>
        <button class="tool-btn" @click="toInverseCase" :disabled="!input">反转大小写</button>
      </div>
    </div>

    <!-- Punctuation -->
    <div class="tool-section">
      <h4>标点符号转换</h4>
      <div class="tool-row">
        <button class="tool-btn" @click="cnToEnPunct" :disabled="!input">中文标点 → 英文</button>
        <button class="tool-btn" @click="enToCnPunct" :disabled="!input">英文标点 → 中文</button>
      </div>
    </div>

    <!-- Text Operations -->
    <div class="tool-section">
      <h4>文本操作</h4>
      <div class="tool-row">
        <button class="tool-btn" @click="dedupLines" :disabled="!input">行去重</button>
        <button class="tool-btn" @click="sortLines" :disabled="!input">字母排序</button>
        <button class="tool-btn" @click="sortLinesReverse" :disabled="!input">逆序排序</button>
        <button class="tool-btn" @click="sortLinesRandom" :disabled="!input">随机排序</button>
        <button class="tool-btn" @click="sortLinesByLength" :disabled="!input">按长度排序</button>
      </div>
      <div class="tool-row">
        <button class="tool-btn" @click="trimLines" :disabled="!input">去除首尾空格</button>
        <button class="tool-btn" @click="removeEmptyLines" :disabled="!input">删除空行</button>
        <button class="tool-btn" @click="addLineNumbers" :disabled="!input">添加行号</button>
        <button class="tool-btn" @click="removeLineNumbers" :disabled="!input">删除行号</button>
        <button class="tool-btn" @click="reverseText" :disabled="!input">文本反转</button>
        <button class="tool-btn" @click="reverseLines" :disabled="!input">行序反转</button>
      </div>
      <div class="tool-row">
        <button class="tool-btn" @click="mergeLines" :disabled="!input">合并为单行</button>
        <button class="tool-btn" @click="splitByComma" :disabled="!input">逗号分行</button>
        <button class="tool-btn" @click="uniqueWords" :disabled="!input">词去重</button>
      </div>
    </div>

    <div class="tool-row">
      <button class="tool-btn primary" @click="copyOutput" :disabled="!input">复制结果</button>
      <button class="tool-btn" @click="clearAll">清空</button>
    </div>

    <!-- Statistics -->
    <div class="tool-section" v-if="stats">
      <h4>文本统计</h4>
      <div class="stats-grid">
        <div class="stat-item">
          <span class="stat-value">{{ stats.chars }}</span>
          <span class="stat-label">字符数</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">{{ stats.noSpaceChars }}</span>
          <span class="stat-label">无空格字符</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">{{ stats.words }}</span>
          <span class="stat-label">单词数</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">{{ stats.lines }}</span>
          <span class="stat-label">行数</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">{{ stats.bytes }}</span>
          <span class="stat-label">字节数</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">{{ stats.chineseChars }}</span>
          <span class="stat-label">中文字符</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
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
  [/；/g, ';'], [/：/g, ':'], [/"/g, '"'], [/"/g, '"'],
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

<style scoped>

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
  gap: 12px;
  padding: 12px;
  background: var(--color-base-200);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 8px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.stat-value {
  font-size: 20px;
  font-weight: 700;
  color: var(--color-primary);
}

.stat-label {
  font-size: 11px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.tool-section { margin-bottom: 20px; }
.tool-section h4 { font-size: 14px; font-weight: 600; color: var(--color-base-content); margin: 0 0 10px 0; display: flex; align-items: center; gap: 6px; }
.tool-textarea { width: 100%; min-height: 120px; padding: 10px 12px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 8px; font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; font-size: 13px; background: var(--color-base-200); color: var(--color-base-content); resize: vertical; outline: none; }
.tool-textarea:focus { border-color: var(--color-primary); }
.tool-input { width: 100%; padding: 8px 12px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 6px; font-size: 13px; background: var(--color-base-200); color: var(--color-base-content); outline: none; }
.tool-input:focus { border-color: var(--color-primary); }
.tool-row { display: flex; gap: 10px; margin-bottom: 12px; flex-wrap: wrap; }
.tool-btn { padding: 7px 16px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 6px; font-size: 13px; font-weight: 500; cursor: pointer; background: var(--color-base-100); color: var(--color-base-content); transition: all 0.15s; display: inline-flex; align-items: center; gap: 4px; }
.tool-btn:hover { border-color: var(--color-primary); color: var(--color-primary); }
.tool-btn.primary { background: var(--color-primary); color: white; border-color: var(--color-primary); }
.tool-btn.primary:hover { opacity: 0.9; }
.tool-btn-group { display: flex; gap: 4px; }
.tool-btn-group .tool-btn { border-radius: 0; }
.tool-btn-group .tool-btn:first-child { border-radius: 6px 0 0 6px; }
.tool-btn-group .tool-btn:last-child { border-radius: 0 6px 6px 0; }
.tool-btn-group .tool-btn.active { background: var(--color-primary); color: white; border-color: var(--color-primary); position: relative; z-index: 1; }
.tool-result { margin-top: 10px; padding: 10px 12px; background: var(--color-base-200); border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 8px; font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; font-size: 13px; color: var(--color-base-content); white-space: pre-wrap; word-break: break-all; max-height: 300px; overflow-y: auto; }
.tool-label { font-size: 12px; font-weight: 500; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); margin-bottom: 4px; display: block; }
.tool-select { padding: 7px 10px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 6px; font-size: 13px; background: var(--color-base-200); color: var(--color-base-content); outline: none; }
.tool-select:focus { border-color: var(--color-primary); }
.tool-checkbox { display: flex; align-items: center; gap: 6px; font-size: 13px; color: var(--color-base-content); cursor: pointer; }
.tool-divider { border: none; border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); margin: 20px 0; }
</style>
