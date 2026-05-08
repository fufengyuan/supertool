<template>
  <div class="time-tool">
    <h3>⏰ 时间戳转换</h3>

    <!-- Current Timestamp -->
    <div class="tool-section">
      <h4>当前时间戳</h4>
      <div class="current-timestamp">
        <span class="ts-value">{{ currentTsMs }}</span>
        <span class="ts-label">毫秒</span>
        <span class="ts-value sec">{{ currentTsSec }}</span>
        <span class="ts-label">秒</span>
        <button class="tool-btn" @click="copyText(currentTsMs, toast, '已复制毫秒时间戳')">📋 复制</button>
      </div>
      <div class="current-date">{{ currentFormatted }}</div>
    </div>

    <hr class="tool-divider" />

    <!-- Timestamp → Date -->
    <div class="tool-section">
      <h4>时间戳 → 日期</h4>
      <div class="tool-row">
        <input
          v-model.number="timestampInput"
          type="number"
          class="tool-input mono"
          placeholder="输入时间戳（秒或毫秒）"
          @input="convertTsToDate"
        />
        <select v-model="inputTimezone" class="tool-select" @change="convertTsToDate">
          <option v-for="tz in timezones" :key="tz" :value="tz">{{ tz }}</option>
        </select>
        <button class="tool-btn" @click="copyText(dateOutput, toast)">📋 复制</button>
      </div>
      <div v-if="dateOutput" class="tool-result">{{ dateOutput }}</div>
    </div>

    <hr class="tool-divider" />

    <!-- Date → Timestamp -->
    <div class="tool-section">
      <h4>日期 → 时间戳</h4>
      <div class="tool-row">
        <input
          v-model="datetimeInput"
          type="datetime-local"
          class="tool-input mono"
          @input="convertDateToTs"
        />
        <select v-model="outputTimezone" class="tool-select" @change="convertDateToTs">
          <option v-for="tz in timezones" :key="tz" :value="tz">{{ tz }}</option>
        </select>
      </div>
      <div v-if="tsOutput" class="tool-result">{{ tsOutput }}</div>
      <div v-if="tsOutput" class="tool-row" style="margin-top: 8px">
        <button class="tool-btn" @click="copyText(tsOutput, toast)">📋 复制</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
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

<style scoped>

.time-tool {
  max-width: 700px;
}

.time-tool h3 {
  font-size: 18px;
  font-weight: 700;
  color: var(--color-base-content);
  margin: 0 0 20px 0;
}

.current-timestamp {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.ts-value {
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 20px;
  font-weight: 700;
  color: var(--color-primary);
}

.ts-value.sec {
  font-size: 16px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.ts-label {
  font-size: 12px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.current-date {
  margin-top: 6px;
  font-size: 13px;
  color: var(--color-base-content);
}

.mono {
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
}

.tool-section h4 {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-base-content);
  margin: 0 0 10px 0;
}

.tool-input {
  flex: 1;
  min-width: 180px;
}

.tool-result {
  margin-top: 10px;
  padding: 10px 12px;
  background: var(--color-base-200);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 8px;
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  color: var(--color-base-content);
  white-space: pre-wrap;
  word-break: break-all;
}

.tool-row {
  display: flex;
  gap: 10px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.tool-btn {
  padding: 7px 16px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  background: var(--color-base-100);
  color: var(--color-base-content);
  transition: all 0.15s;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.tool-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.tool-divider {
  border: none;
  border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  margin: 20px 0;
}

.tool-select {
  padding: 7px 10px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 6px;
  font-size: 13px;
  background: var(--color-base-200);
  color: var(--color-base-content);
  outline: none;
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
