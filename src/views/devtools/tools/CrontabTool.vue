<template>
  <ToolPage
    icon="timer"
    name="Crontab 校验"
    description="Cron 表达式校验、中文描述、未来 10 次执行时间预览"
    @back="$emit('back')"
  >
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <h4 class="text-xs font-semibold text-base-content/70 mb-2.5 flex items-center gap-1.5"><SvgIcon name="clock" size="12" /> Cron 表达式（5 字段）</h4>
      <div class="flex gap-2 flex-wrap items-center">
        <input
          v-model="cronInput"
          type="text"
          class="input input-bordered input-sm font-mono flex-1 min-w-[180px] bg-base-200/60"
          placeholder="* * * * *"
          @input="validateCron"
        />
        <button class="btn btn-primary btn-sm" @click="validateCron">校验</button>
        <button class="btn btn-outline btn-sm" @click="copyText(cronDescription, toast)" :disabled="!cronDescription"><SvgIcon name="copy" size="11" /> 复制</button>
      </div>
      <div v-if="cronDescription" class="mt-3 p-3 bg-success/10 border border-success/25 rounded-lg text-sm text-success">{{ cronDescription }}</div>
      <div v-if="cronError" class="mt-3 p-3 bg-error/10 border border-error/25 rounded-lg text-sm text-error">{{ cronError }}</div>
    </div>

    <div v-if="nextTimes.length > 0" class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <h4 class="text-xs font-semibold text-base-content/70 mb-2.5 flex items-center gap-1.5"><SvgIcon name="calendar" size="12" /> 接下来 10 次执行时间</h4>
      <div class="max-h-60 overflow-y-auto flex flex-col gap-1">
        <div
          v-for="(t, i) in nextTimes"
          :key="i"
          class="flex items-center gap-2.5 px-3 py-1.5 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-xs text-base-content"
        >
          <span class="text-base-content/40 w-6 text-right shrink-0">{{ i + 1 }}.</span>
          <span class="flex-1">{{ t }}</span>
        </div>
      </div>
    </div>

    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <h4 class="text-xs font-semibold text-base-content/70 mb-2.5 flex items-center gap-1.5"><SvgIcon name="sparkles" size="12" /> 常见示例</h4>
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
        <div
          v-for="ex in examples"
          :key="ex.expr"
          class="p-2.5 bg-base-200/60 border border-base-content/10 rounded-lg cursor-pointer hover:border-primary/50 transition-colors flex items-center gap-2.5"
          @click="cronInput = ex.expr; validateCron()"
        >
          <code class="font-mono text-primary text-xs shrink-0">{{ ex.expr }}</code>
          <span class="text-xs text-base-content/60 truncate">{{ ex.desc }}</span>
        </div>
      </div>
    </div>

    <div class="bg-base-100 border border-base-content/10 rounded-xl overflow-hidden">
      <h4 class="text-xs font-semibold text-base-content/70 px-4 pt-4 pb-2.5">字段说明</h4>
      <div class="overflow-x-auto pb-4 px-4">
        <table class="w-full border-collapse text-xs">
          <thead>
            <tr class="text-left text-primary">
              <th class="py-1.5 pr-3 font-semibold">字段</th>
              <th class="py-1.5 pr-3 font-semibold">允许值</th>
              <th class="py-1.5 font-semibold">特殊字符</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="f in fieldRules" :key="f.name" class="border-t border-base-content/5">
              <td class="py-1.5 pr-3 font-mono text-base-content">{{ f.name }}</td>
              <td class="py-1.5 pr-3 font-mono text-base-content/80">{{ f.range }}</td>
              <td class="py-1.5 font-mono text-base-content/80">{{ f.chars }}</td>
            </tr>
          </tbody>
        </table>
      </div>
      <div class="px-4 pb-4 pt-2 text-xs text-base-content/60 border-t border-base-content/10 leading-relaxed">
        <strong class="text-base-content/80">特殊字符说明：</strong><br />
        <code class="text-primary">*</code> 任意值 · <code class="text-primary">,</code> 列举 · <code class="text-primary">-</code> 范围 · <code class="text-primary">/</code> 步长 · <code class="text-primary">?</code> 不指定
      </div>
    </div>
  </ToolPage>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { ref } from 'vue'
import cronstrue from 'cronstrue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

const toast = useToast()

const cronInput = ref('* * * * *')
const cronDescription = ref('')
const cronError = ref('')
const nextTimes = ref<string[]>([])

function getNextExecutionTimes(expression: string, count = 10): string[] {
  const parts = expression.trim().split(/\s+/)
  if (parts.length !== 5) {return []}

  const [minute, hour, dom, month, dow] = parts

  function matchesField(value: number, field: string, max: number): boolean {
    if (field === '*') {return true}
    if (field === '?') {return true}

    const segments = field.split(',')
    for (const seg of segments) {
      if (seg.includes('/')) {
        const [range, step] = seg.split('/')
        const stepNum = parseInt(step)
        const start = range === '*' ? 0 : parseInt(range)
        for (let i = start; i <= max; i += stepNum) {
          if (i === value) {return true}
        }
      } else if (seg.includes('-')) {
        const [start, end] = seg.split('-').map(Number)
        if (value >= start && value <= end) {return true}
      } else {
        if (parseInt(seg) === value) {return true}
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

  if (!cronInput.value.trim()) {return}

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