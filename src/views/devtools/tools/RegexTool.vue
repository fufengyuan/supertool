<template>
  <div>
    <h3 class="text-lg font-bold text-base-content mb-5">正则表达式</h3>

    <div class="mb-5">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">正则表达式</h4>
      <div class="flex flex-wrap gap-2.5 mb-3">
        <input
          v-model="pattern"
          class="input input-bordered flex-1"
          placeholder="输入正则表达式，如 \d+"
          @input="runTest"
        />
        <div class="join">
          <button class="btn btn-ghost join-item" :class="{ 'btn-active': flags.includes('g') }" @click="toggleFlag('g')">g</button>
          <button class="btn btn-ghost join-item" :class="{ 'btn-active': flags.includes('i') }" @click="toggleFlag('i')">i</button>
          <button class="btn btn-ghost join-item" :class="{ 'btn-active': flags.includes('m') }" @click="toggleFlag('m')">m</button>
          <button class="btn btn-ghost join-item" :class="{ 'btn-active': flags.includes('s') }" @click="toggleFlag('s')">s</button>
          <button class="btn btn-ghost join-item" :class="{ 'btn-active': flags.includes('u') }" @click="toggleFlag('u')">u</button>
        </div>
      </div>
      <div v-if="pattern" class="mt-1.5 text-xs opacity-60">
        正则: <code class="bg-base-200 px-1.5 py-0.5 rounded font-mono">/{{ pattern }}/{{ flags }}</code>
        <span v-if="regexError" class="text-error"> <SvgIcon name="alertTriangle" size="14" class="align-text-bottom" /> {{ regexError }}</span>
      </div>
    </div>

    <div class="mb-5">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">测试文本</h4>
      <textarea v-model="testText" class="textarea textarea-bordered w-full font-mono text-sm min-h-[120px]" placeholder="输入测试文本..." rows="5" @input="runTest"></textarea>
    </div>

    <div class="mb-5" v-if="matchCount !== null">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">匹配结果</h4>
      <div class="bg-base-200 border border-base-content/10 rounded-box p-3 font-mono text-sm whitespace-pre-wrap break-all max-h-72 overflow-y-auto">
        <div class="mb-2 text-sm text-base-content">
          共找到 <strong>{{ matchCount }}</strong> 个匹配
          <span v-if="groups.length > 0">, {{ groups.length }} 个捕获组</span>
        </div>
        <div v-if="matches.length > 0" class="max-h-[200px] overflow-y-auto">
          <div v-for="(m, idx) in matches.slice(0, 50)" :key="idx" class="flex gap-2 py-1 border-b border-base-content/10 text-sm">
            <span class="opacity-60 font-semibold min-w-[32px]">#{{ idx + 1}}</span>
            <span class="font-mono break-all">{{ m }}</span>
          </div>
          <div v-if="matches.length > 50" class="opacity-60 py-1">... 还有 {{ matches.length - 50 }} 个匹配</div>
        </div>
        <div v-if="matches.length === 0 && matchCount === 0" class="opacity-60 italic">
          没有匹配结果
        </div>
      </div>
    </div>

    <hr class="border-base-content/10 my-5" />

    <!-- Replace section -->
    <div class="mb-5">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">替换</h4>
      <div class="flex flex-wrap gap-2.5 mb-3">
        <input
          v-model="replacement"
          class="input input-bordered flex-1"
          placeholder="替换内容，支持 $1, $2 等引用"
        />
        <button class="btn btn-primary" @click="runReplace" :disabled="!pattern || !testText">替换</button>
        <button class="btn btn-ghost" @click="copyReplaced" :disabled="!replacedResult">复制结果</button>
      </div>

      <div class="mb-5" v-if="replacedResult !== null">
        <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">替换结果</h4>
        <textarea v-model="replacedResult" class="textarea textarea-bordered w-full font-mono text-sm min-h-[120px]" readonly rows="4"></textarea>
      </div>
    </div>

    <hr class="border-base-content/10 my-5" />

    <!-- Highlighted text -->
    <div class="mb-5" v-if="highlightedHtml">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">匹配高亮</h4>
      <div class="p-3 bg-base-200 border border-base-content/10 rounded-box font-mono text-sm leading-relaxed whitespace-pre-wrap break-all max-h-72 overflow-y-auto" v-html="highlightedHtml"></div>
    </div>
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
        result += `<mark class="bg-yellow-300/30 text-yellow-800 px-0.5 rounded">${escapeHtml(match[0])}</mark>`
        lastIndex = match.index + match[0].length
      }
    } else {
      const match = regex.exec(testText.value)
      if (match) {
        result += escapeHtml(testText.value.substring(0, match.index))
        result += `<mark class="bg-yellow-300/30 text-yellow-800 px-0.5 rounded">${escapeHtml(match[0])}</mark>`
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
