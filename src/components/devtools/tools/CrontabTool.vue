<template>
  <div class="crontab-tool">
    <h3>⏱️ Crontab 校验</h3>

    <!-- Input -->
    <div class="tool-section">
      <h4>Cron 表达式（5 字段）</h4>
      <div class="tool-row">
        <input
          v-model="cronInput"
          type="text"
          class="tool-input mono"
          placeholder="* * * * *"
          @input="validateCron"
        />
        <button class="tool-btn primary" @click="validateCron">校验</button>
        <button class="tool-btn" @click="copyText(cronDescription, toast)">📋 复制</button>
      </div>

      <!-- Description -->
      <div v-if="cronDescription" class="tool-result desc">{{ cronDescription }}</div>
      <div v-if="cronError" class="tool-result error">{{ cronError }}</div>

      <!-- Next execution times -->
      <div v-if="nextTimes.length > 0" class="tool-section">
        <h4>接下来 10 次执行时间</h4>
        <div class="tool-result">
          <div v-for="(t, i) in nextTimes" :key="i" class="next-time-item">
            <span class="next-time-index">{{ i + 1 }}.</span>
            <span class="next-time-value">{{ t }}</span>
          </div>
        </div>
      </div>
    </div>

    <hr class="tool-divider" />

    <!-- Examples -->
    <div class="tool-section">
      <h4>常见示例</h4>
      <div class="examples-grid">
        <div
          v-for="ex in examples"
          :key="ex.expr"
          class="example-item"
          @click="cronInput = ex.expr; validateCron()"
        >
          <code class="example-expr">{{ ex.expr }}</code>
          <span class="example-desc">{{ ex.desc }}</span>
        </div>
      </div>
    </div>

    <hr class="tool-divider" />

    <!-- Field Explanation -->
    <div class="tool-section">
      <h4>字段说明</h4>
      <div class="field-table">
        <div class="field-row header">
          <span>字段</span><span>允许值</span><span>特殊字符</span>
        </div>
        <div class="field-row" v-for="f in fieldRules" :key="f.name">
          <span class="field-name">{{ f.name }}</span>
          <span class="field-range">{{ f.range }}</span>
          <span class="field-chars">{{ f.chars }}</span>
        </div>
      </div>
      <div class="field-note">
        <p><strong>特殊字符说明：</strong></p>
        <p><code>*</code> = 任意值 &nbsp; <code>,</code> = 列举 &nbsp; <code>-</code> = 范围 &nbsp; <code>/</code> = 步长 &nbsp; <code>?</code> = 不指定</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import cronstrue from 'cronstrue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const toast = useToast()

const cronInput = ref('* * * * *')
const cronDescription = ref('')
const cronError = ref('')
const nextTimes = ref<string[]>([])

function getNextExecutionTimes(expression: string, count = 10): string[] {
  const parts = expression.trim().split(/\s+/)
  if (parts.length !== 5) return []

  const [minute, hour, dom, month, dow] = parts

  function matchesField(value: number, field: string, max: number): boolean {
    if (field === '*') return true
    if (field === '?') return true

    const segments = field.split(',')
    for (const seg of segments) {
      if (seg.includes('/')) {
        const [range, step] = seg.split('/')
        const stepNum = parseInt(step)
        const start = range === '*' ? 0 : parseInt(range)
        for (let i = start; i <= max; i += stepNum) {
          if (i === value) return true
        }
      } else if (seg.includes('-')) {
        const [start, end] = seg.split('-').map(Number)
        if (value >= start && value <= end) return true
      } else {
        if (parseInt(seg) === value) return true
      }
    }
    return false
  }

  const now = new Date()
  const results: string[] = []
  const check = new Date(now.getFullYear(), now.getMonth(), now.getDate(), now.getHours(), now.getMinutes() + 1, 0)

  let attempts = 0
  while (results.length < count && attempts < 525600 * 2) {
    attempts++
    const m = check.getMinutes()
    const h = check.getHours()
    const d = check.getDate()
    const mo = check.getMonth() + 1
    const dw = check.getDay()

    if (
      matchesField(m, minute, 59) &&
      matchesField(h, hour, 23) &&
      matchesField(d, dom, 31) &&
      matchesField(mo, month, 12) &&
      matchesField(dw, dow, 6)
    ) {
      results.push(check.toLocaleString('zh-CN', {
        year: 'numeric', month: '2-digit', day: '2-digit',
        hour: '2-digit', minute: '2-digit', second: '2-digit',
        hour12: false,
      }))
    }
    check.setMinutes(check.getMinutes() + 1)
  }

  return results
}

function validateCron() {
  cronDescription.value = ''
  cronError.value = ''
  nextTimes.value = []

  if (!cronInput.value.trim()) return

  try {
    const desc = cronstrue.toString(cronInput.value.trim(), { locale: 'zh_CN' })
    cronDescription.value = desc
    nextTimes.value = getNextExecutionTimes(cronInput.value.trim())
  } catch (e: any) {
    cronError.value = `错误: ${e.message || '无效的 Cron 表达式'}`
    toast.error('Cron 表达式校验失败')
  }
}

const examples = [
  { expr: '* * * * *', desc: '每分钟' },
  { expr: '*/5 * * * *', desc: '每5分钟' },
  { expr: '0 * * * *', desc: '每小时整点' },
  { expr: '0 */2 * * *', desc: '每2小时' },
  { expr: '0 0 * * *', desc: '每天午夜' },
  { expr: '0 9 * * *', desc: '每天早上9点' },
  { expr: '0 0 * * 1', desc: '每周一午夜' },
  { expr: '0 0 1 * *', desc: '每月1日午夜' },
  { expr: '0 0 1 1 *', desc: '每年1月1日午夜' },
  { expr: '30 8 * * 1-5', desc: '工作日早8:30' },
]

const fieldRules = [
  { name: '分钟', range: '0-59', chars: '* , - /' },
  { name: '小时', range: '0-23', chars: '* , - /' },
  { name: '日', range: '1-31', chars: '* , - / ? L W' },
  { name: '月', range: '1-12', chars: '* , - /' },
  { name: '星期', range: '0-6 (0=周日)', chars: '* , - / ? L #' },
]
</script>

<style scoped>

.crontab-tool {
  max-width: 700px;
}

.crontab-tool h3 {
  font-size: 18px;
  font-weight: 700;
  color: oklch(var(--bc));
  margin: 0 0 20px 0;
}

.tool-section h4 {
  font-size: 14px;
  font-weight: 600;
  color: oklch(var(--bc));
  margin: 0 0 10px 0;
}

.tool-row {
  display: flex;
  gap: 10px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.tool-result {
  margin-top: 10px;
  padding: 10px 12px;
  background: oklch(var(--b2));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  color: oklch(var(--bc));
  white-space: pre-wrap;
  word-break: break-all;
}

.tool-result.error {
  border-color: #e74c3c;
  color: #e74c3c;
}

.tool-result.desc {
  border-color: oklch(var(--p));
}

.mono {
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
}

.next-time-item {
  display: flex;
  gap: 8px;
  padding: 2px 0;
}

.next-time-index {
  color: oklch(var(--bc) / 0.6);
  min-width: 24px;
}

.examples-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 8px;
}

.example-item {
  padding: 8px 12px;
  background: oklch(var(--b2));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
  display: flex;
  align-items: center;
  gap: 10px;
}

.example-item:hover {
  border-color: oklch(var(--p));
  background: oklch(var(--p) / 0.1);
}

.example-expr {
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  color: oklch(var(--p));
  white-space: nowrap;
}

.example-desc {
  font-size: 12px;
  color: oklch(var(--bc) / 0.6);
}

.field-table {
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  overflow: hidden;
}

.field-row {
  display: grid;
  grid-template-columns: 80px 1fr 1fr;
  padding: 8px 12px;
  font-size: 13px;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
}

.field-row:last-child {
  border-bottom: none;
}

.field-row.header {
  background: oklch(var(--b2));
  font-weight: 600;
  color: oklch(var(--bc) / 0.6);
}

.field-name {
  font-weight: 500;
}

.field-range, .field-chars {
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
}

.field-note {
  margin-top: 12px;
  font-size: 13px;
  color: oklch(var(--bc) / 0.6);
}

.field-note p {
  margin: 4px 0;
}

.field-note code {
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  background: oklch(var(--b2));
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 12px;
}

.tool-btn {
  padding: 7px 16px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  background: oklch(var(--b1));
  color: oklch(var(--bc));
  transition: all 0.15s;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.tool-btn:hover {
  border-color: oklch(var(--p));
  color: oklch(var(--p));
}

.tool-btn.primary {
  background: oklch(var(--p));
  color: white;
  border-color: oklch(var(--p));
}

.tool-btn.primary:hover {
  opacity: 0.9;
}

.tool-divider {
  border: none;
  border-top: 1px solid oklch(var(--bc) / 0.1);
  margin: 20px 0;
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
