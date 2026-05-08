<template>
  <div class="crontab-tool">
    <h3 class="text-lg font-bold text-base-content mb-5">⏱️ Crontab 校验</h3>

    <!-- Input -->
    <div class="mb-5">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">Cron 表达式（5 字段）</h4>
      <div class="flex gap-2.5 mb-3 flex-wrap items-center">
        <input
          v-model="cronInput"
          type="text"
          class="tool-input mono"
          placeholder="* * * * *"
          @input="validateCron"
        />
        <button class="btn btn-primary btn-sm" @click="validateCron">校验</button>
        <button class="btn btn-ghost btn-sm" @click="copyText(cronDescription, toast)">📋 复制</button>
      </div>

      <!-- Description -->
      <div v-if="cronDescription" class="tool-result desc">{{ cronDescription }}</div>
      <div v-if="cronError" class="tool-result error">{{ cronError }}</div>

      <!-- Next execution times -->
      <div v-if="nextTimes.length > 0" class="mb-5">
        <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">接下来 10 次执行时间</h4>
        <div class="mt-2.5 p-2.5 bg-base-200 border border-base-content/10 rounded-lg font-mono text-xs whitespace-pre-wrap break-all max-h-[300px] overflow-y-auto">
          <div v-for="(t, i) in nextTimes" :key="i" class="next-time-item">
            <span class="next-time-index">{{ i + 1 }}.</span>
            <span class="next-time-value">{{ t }}</span>
          </div>
        </div>
      </div>
    </div>

    <hr class="border-t border-base-content/10 my-5" />

    <!-- Examples -->
    <div class="mb-5">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">常见示例</h4>
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

    <hr class="border-t border-base-content/10 my-5" />

    <!-- Field Explanation -->
    <div class="mb-5">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">字段说明</h4>
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