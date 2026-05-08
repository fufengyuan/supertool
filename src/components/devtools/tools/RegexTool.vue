<template>
  <div class="regex-tool">
    <h3>正则表达式</h3>

    <div class="tool-section">
      <h4>正则表达式</h4>
      <div class="tool-row">
        <input
          v-model="pattern"
          class="tool-input"
          style="flex: 1;"
          placeholder="输入正则表达式，如 \d+"
          @input="runTest"
        />
        <div class="tool-btn-group">
          <button class="tool-btn" :class="{ active: flags.includes('g') }" @click="toggleFlag('g')">g</button>
          <button class="tool-btn" :class="{ active: flags.includes('i') }" @click="toggleFlag('i')">i</button>
          <button class="tool-btn" :class="{ active: flags.includes('m') }" @click="toggleFlag('m')">m</button>
          <button class="tool-btn" :class="{ active: flags.includes('s') }" @click="toggleFlag('s')">s</button>
          <button class="tool-btn" :class="{ active: flags.includes('u') }" @click="toggleFlag('u')">u</button>
        </div>
      </div>
      <div class="regex-flags-info" v-if="pattern">
        正则: <code>/{{ pattern }}/{{ flags }}</code>
        <span v-if="regexError" class="regex-error"> ⚠️ {{ regexError }}</span>
      </div>
    </div>

    <div class="tool-section">
      <h4>测试文本</h4>
      <textarea v-model="testText" class="tool-textarea" placeholder="输入测试文本..." rows="5" @input="runTest"></textarea>
    </div>

    <div class="tool-section" v-if="matchCount !== null">
      <h4>匹配结果</h4>
      <div class="tool-result">
        <div class="match-summary">
          共找到 <strong>{{ matchCount }}</strong> 个匹配
          <span v-if="groups.length > 0">, {{ groups.length }} 个捕获组</span>
        </div>
        <div v-if="matches.length > 0" class="match-list">
          <div v-for="(m, idx) in matches.slice(0, 50)" :key="idx" class="match-item">
            <span class="match-index">#{{ idx + 1}}</span>
            <span class="match-value">{{ m }}</span>
          </div>
          <div v-if="matches.length > 50" class="match-more">... 还有 {{ matches.length - 50 }} 个匹配</div>
        </div>
        <div v-if="matches.length === 0 && matchCount === 0" class="match-empty">
          没有匹配结果
        </div>
      </div>
    </div>

    <hr class="tool-divider" />

    <!-- Replace section -->
    <div class="tool-section">
      <h4>替换</h4>
      <div class="tool-row">
        <input
          v-model="replacement"
          class="tool-input"
          style="flex: 1;"
          placeholder="替换内容，支持 $1, $2 等引用"
        />
        <button class="tool-btn primary" @click="runReplace" :disabled="!pattern || !testText">替换</button>
        <button class="tool-btn" @click="copyReplaced" :disabled="!replacedResult">复制结果</button>
      </div>

      <div class="tool-section" v-if="replacedResult !== null">
        <h4>替换结果</h4>
        <textarea v-model="replacedResult" class="tool-textarea" readonly rows="4"></textarea>
      </div>
    </div>

    <hr class="tool-divider" />

    <!-- Highlighted text -->
    <div class="tool-section" v-if="highlightedHtml">
      <h4>匹配高亮</h4>
      <div class="highlighted-text" v-html="highlightedHtml"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
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

function toggleFlag(flag: string) {
  if (flags.value.includes(flag)) {
    flags.value = flags.value.replace(flag, '')
    if (!flags.value) flags.value = ''
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

const matchCount = ref<number | null>(null)
const matches = ref<string[]>([])
const groups = ref<string[]>([])

function runTest() {
  const regex = createRegex()
  if (!regex || !testText.value) {
    matchCount.value = null
    matches.value = []
    groups.value = []
    return
  }

  try {
    const allMatches: string[] = []

    if (flags.value.includes('g')) {
      let match
      const regex2 = new RegExp(regex.source, regex.flags)
      while ((match = regex2.exec(testText.value)) !== null) {
        allMatches.push(match[0])
        if (match.length > 1) {
          groups.value = match.slice(1)
        }
        if (match[0] === '') regex2.lastIndex++ // Avoid infinite loop
      }
    } else {
      const match = regex.exec(testText.value)
      if (match) {
        allMatches.push(match[0])
        if (match.length > 1) {
          groups.value = match.slice(1)
        }
      }
    }

    matchCount.value = allMatches.length
    matches.value = allMatches
  } catch (e: any) {
    matchCount.value = 0
    matches.value = []
  }
}

function runReplace() {
  const regex = createRegex()
  if (!regex || !testText.value) return

  try {
    replacedResult.value = testText.value.replace(regex, replacement.value)
    toast.success('替换完成')
  } catch (e: any) {
    toast.error(`替换失败: ${e.message}`)
  }
}

const highlightedHtml = computed(() => {
  const regex = createRegex()
  if (!regex || !testText.value) return ''

  try {
    const escaped = testText.value
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')

    // We need to use the original text for regex matching, then replace with highlighted HTML
    let result = ''
    let lastIndex = 0

    if (flags.value.includes('g')) {
      const regex2 = new RegExp(regex.source, regex.flags)
      let match
      while ((match = regex2.exec(testText.value)) !== null) {
        if (match[0] === '') {
          regex2.lastIndex++
          continue
        }

        // Add text before match
        result += escapeHtml(testText.value.substring(lastIndex, match.index))
        // Add highlighted match
        result += `<mark class="regex-match">${escapeHtml(match[0])}</mark>`
        lastIndex = match.index + match[0].length
      }
    } else {
      const match = regex.exec(testText.value)
      if (match) {
        result += escapeHtml(testText.value.substring(0, match.index))
        result += `<mark class="regex-match">${escapeHtml(match[0])}</mark>`
        lastIndex = match.index + match[0].length
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

<style scoped>

.regex-flags-info {
  margin-top: 6px;
  font-size: 12px;
  color: oklch(var(--bc) / 0.6);
}

.regex-flags-info code {
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  background: oklch(var(--b2));
  padding: 2px 6px;
  border-radius: 4px;
}

.regex-error {
  color: #ef4444;
}

.match-summary {
  margin-bottom: 8px;
  font-size: 13px;
  color: oklch(var(--bc));
}

.match-list {
  max-height: 200px;
  overflow-y: auto;
}

.match-item {
  display: flex;
  gap: 8px;
  padding: 4px 0;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
  font-size: 13px;
}

.match-index {
  color: oklch(var(--bc) / 0.6);
  font-weight: 600;
  min-width: 32px;
}

.match-value {
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  word-break: break-all;
}

.match-empty {
  color: oklch(var(--bc) / 0.6);
  font-style: italic;
}

.match-more {
  color: oklch(var(--bc) / 0.6);
  padding: 4px 0;
}

.highlighted-text {
  padding: 12px;
  background: oklch(var(--b2));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 300px;
  overflow-y: auto;
}

:deep(.regex-match) {
  background: rgba(250, 204, 21, 0.3);
  color: #854d0e;
  padding: 1px 2px;
  border-radius: 2px;
}

.tool-section { margin-bottom: 20px; }
.tool-section h4 { font-size: 14px; font-weight: 600; color: oklch(var(--bc)); margin: 0 0 10px 0; display: flex; align-items: center; gap: 6px; }
.tool-textarea { width: 100%; min-height: 120px; padding: 10px 12px; border: 1px solid oklch(var(--bc) / 0.1); border-radius: 8px; font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; font-size: 13px; background: oklch(var(--b2)); color: oklch(var(--bc)); resize: vertical; outline: none; }
.tool-textarea:focus { border-color: oklch(var(--p)); }
.tool-input { width: 100%; padding: 8px 12px; border: 1px solid oklch(var(--bc) / 0.1); border-radius: 6px; font-size: 13px; background: oklch(var(--b2)); color: oklch(var(--bc)); outline: none; }
.tool-input:focus { border-color: oklch(var(--p)); }
.tool-row { display: flex; gap: 10px; margin-bottom: 12px; flex-wrap: wrap; }
.tool-btn { padding: 7px 16px; border: 1px solid oklch(var(--bc) / 0.1); border-radius: 6px; font-size: 13px; font-weight: 500; cursor: pointer; background: oklch(var(--b1)); color: oklch(var(--bc)); transition: all 0.15s; display: inline-flex; align-items: center; gap: 4px; }
.tool-btn:hover { border-color: oklch(var(--p)); color: oklch(var(--p)); }
.tool-btn.primary { background: oklch(var(--p)); color: white; border-color: oklch(var(--p)); }
.tool-btn.primary:hover { opacity: 0.9; }
.tool-btn-group { display: flex; gap: 4px; }
.tool-btn-group .tool-btn { border-radius: 0; }
.tool-btn-group .tool-btn:first-child { border-radius: 6px 0 0 6px; }
.tool-btn-group .tool-btn:last-child { border-radius: 0 6px 6px 0; }
.tool-btn-group .tool-btn.active { background: oklch(var(--p)); color: white; border-color: oklch(var(--p)); position: relative; z-index: 1; }
.tool-result { margin-top: 10px; padding: 10px 12px; background: oklch(var(--b2)); border: 1px solid oklch(var(--bc) / 0.1); border-radius: 8px; font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; font-size: 13px; color: oklch(var(--bc)); white-space: pre-wrap; word-break: break-all; max-height: 300px; overflow-y: auto; }
.tool-label { font-size: 12px; font-weight: 500; color: oklch(var(--bc) / 0.6); margin-bottom: 4px; display: block; }
.tool-select { padding: 7px 10px; border: 1px solid oklch(var(--bc) / 0.1); border-radius: 6px; font-size: 13px; background: oklch(var(--b2)); color: oklch(var(--bc)); outline: none; }
.tool-select:focus { border-color: oklch(var(--p)); }
.tool-checkbox { display: flex; align-items: center; gap: 6px; font-size: 13px; color: oklch(var(--bc)); cursor: pointer; }
.tool-divider { border: none; border-top: 1px solid oklch(var(--bc) / 0.1); margin: 20px 0; }
</style>
