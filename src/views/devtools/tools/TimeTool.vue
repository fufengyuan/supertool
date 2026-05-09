<template>
  <div class="max-w-[700px]">
    <h3 class="text-lg font-bold text-base-content mb-5">⏰ 时间戳转换</h3>

    <!-- Current Timestamp -->
    <div class="mb-5">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">当前时间戳</h4>
      <div class="flex items-center gap-2.5 flex-wrap">
        <span class="font-mono text-xl font-bold text-primary">{{ currentTsMs }}</span>
        <span class="text-xs opacity-60">毫秒</span>
        <span class="font-mono text-base opacity-60">{{ currentTsSec }}</span>
        <span class="text-xs opacity-60">秒</span>
        <button class="btn btn-ghost" @click="copyText(currentTsMs, toast, '已复制毫秒时间戳')"><SvgIcon name="file" size="14" class="align-text-bottom" /> 复制</button>
      </div>
      <div class="mt-1.5 text-sm text-base-content">{{ currentFormatted }}</div>
    </div>

    <hr class="border-base-content/10 my-5" />

    <!-- Timestamp → Date -->
    <div class="mb-5">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">时间戳 → 日期</h4>
      <div class="flex flex-wrap gap-2.5 mb-3">
        <input
          v-model.number="timestampInput"
          type="number"
          class="input input-bordered font-mono flex-1 min-w-[180px]"
          placeholder="输入时间戳（秒或毫秒）"
          @input="convertTsToDate"
        />
        <select v-model="inputTimezone" class="select select-bordered" @change="convertTsToDate">
          <option v-for="tz in timezones" :key="tz" :value="tz">{{ tz }}</option>
        </select>
        <button class="btn btn-ghost" @click="copyText(dateOutput, toast)"><SvgIcon name="file" size="14" class="align-text-bottom" /> 复制</button>
      </div>
      <div v-if="dateOutput" class="bg-base-200 border border-base-content/10 rounded-box p-3 font-mono text-sm whitespace-pre-wrap break-all">{{ dateOutput }}</div>
    </div>

    <hr class="border-base-content/10 my-5" />

    <!-- Date → Timestamp -->
    <div class="mb-5">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">日期 → 时间戳</h4>
      <div class="flex flex-wrap gap-2.5 mb-3">
        <input
          v-model="datetimeInput"
          type="datetime-local"
          class="input input-bordered font-mono flex-1 min-w-[180px]"
          @input="convertDateToTs"
        />
        <select v-model="outputTimezone" class="select select-bordered" @change="convertDateToTs">
          <option v-for="tz in timezones" :key="tz" :value="tz">{{ tz }}</option>
        </select>
      </div>
      <div v-if="tsOutput" class="bg-base-200 border border-base-content/10 rounded-box p-3 font-mono text-sm whitespace-pre-wrap break-all">{{ tsOutput }}</div>
      <div v-if="tsOutput" class="flex flex-wrap gap-2.5 mb-3 mt-2">
        <button class="btn btn-ghost" @click="copyText(tsOutput, toast)"><SvgIcon name="file" size="14" class="align-text-bottom" /> 复制</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref, onMounted, onUnmounted } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const toast = useToast()

const timezones = [
  'UTC',
  'Asia/Shanghai',
  'America/New_York',
  'America/Chicago',
  'America/Los_Angeles',
  'America/Denver',
  'Europe/London',
  'Europe/Paris',
  'Europe/Berlin',
  'Asia/Tokyo',
  'Asia/Hong_Kong',
  'Asia/Singapore',
  'Australia/Sydney',
  'Pacific/Auckland',
]

/* Current timestamp */
const currentTsMs = ref('')
const currentTsSec = ref('')
const currentFormatted = ref('')
let timer: ReturnType<typeof setInterval> | null = null

function updateCurrentTs() {
  const now = Date.now()
  currentTsMs.value = String(now)
  currentTsSec.value = String(Math.floor(now / 1000))
  currentFormatted.value = new Date(now).toLocaleString('zh-CN', {
    timeZone: 'Asia/Shanghai',
    year: 'numeric', month: '2-digit', day: '2-digit',
    hour: '2-digit', minute: '2-digit', second: '2-digit',
    hour12: false,
  })
}

/* Timestamp → Date */
const timestampInput = ref<number | null>(null)
const inputTimezone = ref('Asia/Shanghai')
const dateOutput = ref('')

function convertTsToDate() {
  if (!timestampInput.value) { dateOutput.value = ''; return }
  let ts = timestampInput.value
  // Auto-detect: if < 1e12, treat as seconds
  if (ts < 1e12) ts *= 1000
  try {
    const d = new Date(ts)
    if (isNaN(d.getTime())) { dateOutput.value = '错误: 无效的时间戳'; return }
    const options: Intl.DateTimeFormatOptions = {
      timeZone: inputTimezone.value,
      year: 'numeric', month: '2-digit', day: '2-digit',
      hour: '2-digit', minute: '2-digit', second: '2-digit',
      hour12: false,
    }
    const formatted = d.toLocaleString('zh-CN', options)
    const weekdays = ['星期日', '星期一', '星期二', '星期三', '星期四', '星期五', '星期六']
    const weekday = weekdays[d.getDay()]
    const offset = getTimezoneOffset(inputTimezone.value, d)
    dateOutput.value = `日期: ${formatted}\n星期: ${weekday}\n时区: ${inputTimezone.value} (${offset})\nISO: ${d.toISOString()}`
  } catch {
    dateOutput.value = '错误: 转换失败'
  }
}

function getTimezoneOffset(tz: string, date: Date): string {
  try {
    const str = date.toLocaleString('en-US', { timeZone: tz, timeZoneName: 'shortOffset' })
    const match = str.match(/GMT([+-]\d{1,2}(?::\d{2})?)/)
    return match ? `UTC${match[1]}` : tz
  } catch {
    return tz
  }
}

/* Date → Timestamp */
const datetimeInput = ref('')
const outputTimezone = ref('Asia/Shanghai')
const tsOutput = ref('')

function convertDateToTs() {
  if (!datetimeInput.value) { tsOutput.value = ''; return }
  try {
    // Create date in selected timezone
    const d = new Date(datetimeInput.value)
    if (isNaN(d.getTime())) { tsOutput.value = '错误: 无效的日期'; return }
    const sec = Math.floor(d.getTime() / 1000)
    const ms = d.getTime()
    tsOutput.value = `秒: ${sec}\n毫秒: ${ms}\nISO: ${d.toISOString()}`
  } catch {
    tsOutput.value = '错误: 转换失败'
  }
}

onMounted(() => {
  updateCurrentTs()
  timer = setInterval(updateCurrentTs, 1000)
})

onUnmounted(() => {
  if (timer) clearInterval(timer)
})
</script>
