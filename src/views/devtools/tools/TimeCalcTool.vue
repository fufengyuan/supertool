<template>
  <ToolPage
    icon="calendar"
    name="时间计算器"
    description="日期加减、日期差值、工作日（跳过周末）计算"
    @back="$emit('back')"
  >
    <!-- 日期加减 -->
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <h4 class="text-xs font-semibold text-base-content/70 mb-3 flex items-center gap-1.5"><SvgIcon name="calendar" size="12" /> 日期加减</h4>
      <div class="flex flex-wrap gap-2 items-center">
        <input v-model="dateMathStart" type="date" class="input input-bordered input-sm bg-base-200/60" />
        <select v-model="dateMathUnit" class="select select-bordered select-sm">
          <option value="days">天</option>
          <option value="months">月</option>
          <option value="years">年</option>
        </select>
        <input v-model.number="dateMathAmount" type="number" class="input input-bordered input-sm font-mono w-20 bg-base-200/60" placeholder="数量" />
        <select v-model="dateMathOp" class="select select-bordered select-sm">
          <option value="add">加</option>
          <option value="sub">减</option>
        </select>
        <button class="btn btn-primary btn-sm" @click="calcDateMath">计算</button>
        <button class="btn btn-outline btn-sm" @click="copyText(dateMathResult, toast)" :disabled="!dateMathResult"><SvgIcon name="copy" size="11" /> 复制</button>
      </div>
      <div v-if="dateMathResult" class="mt-3 p-3 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-sm whitespace-pre-wrap break-all text-base-content">{{ dateMathResult }}</div>
    </div>

    <!-- 日期差值 -->
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <h4 class="text-xs font-semibold text-base-content/70 mb-3 flex items-center gap-1.5"><SvgIcon name="clock" size="12" /> 日期差值</h4>
      <div class="flex flex-wrap gap-2 items-end">
        <div>
          <span class="text-[11px] font-medium text-base-content/50 mb-1 block">开始日期</span>
          <input v-model="diffStart" type="date" class="input input-bordered input-sm bg-base-200/60" />
        </div>
        <div>
          <span class="text-[11px] font-medium text-base-content/50 mb-1 block">结束日期</span>
          <input v-model="diffEnd" type="date" class="input input-bordered input-sm bg-base-200/60" />
        </div>
        <button class="btn btn-primary btn-sm" @click="calcDateDiff">计算</button>
        <button class="btn btn-outline btn-sm" @click="copyText(diffResult, toast)" :disabled="!diffResult"><SvgIcon name="copy" size="11" /> 复制</button>
      </div>
      <div v-if="diffResult" class="mt-3 p-3 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-sm whitespace-pre-wrap break-all text-base-content">{{ diffResult }}</div>
    </div>

    <!-- 工作日计算 -->
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <h4 class="text-xs font-semibold text-base-content/70 mb-3 flex items-center gap-1.5"><SvgIcon name="filter" size="12" /> 工作日计算（跳过周末）</h4>
      <div class="flex flex-wrap gap-2 items-end">
        <div>
          <span class="text-[11px] font-medium text-base-content/50 mb-1 block">开始日期</span>
          <input v-model="workdayStart" type="date" class="input input-bordered input-sm bg-base-200/60" />
        </div>
        <div>
          <span class="text-[11px] font-medium text-base-content/50 mb-1 block">工作日数量</span>
          <input v-model.number="workdayCount" type="number" class="input input-bordered input-sm font-mono w-24 bg-base-200/60" min="1" />
        </div>
        <select v-model="workdayOp" class="select select-bordered select-sm">
          <option value="add">加</option>
          <option value="sub">减</option>
        </select>
        <button class="btn btn-primary btn-sm" @click="calcWorkday">计算</button>
        <button class="btn btn-outline btn-sm" @click="copyText(workdayResult, toast)" :disabled="!workdayResult"><SvgIcon name="copy" size="11" /> 复制</button>
      </div>
      <div v-if="workdayResult" class="mt-3 p-3 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-sm whitespace-pre-wrap break-all text-base-content">{{ workdayResult }}</div>
    </div>
  </ToolPage>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { ref } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

const toast = useToast()

/* Date Math */
const dateMathStart = ref(new Date().toISOString().slice(0, 10))
const dateMathUnit = ref<'days' | 'months' | 'years'>('days')
const dateMathAmount = ref(1)
const dateMathOp = ref<'add' | 'sub'>('add')
const dateMathResult = ref('')

function calcDateMath() {
  if (!dateMathStart.value) { toast.error('请选择开始日期'); return }
  try {
    const d = new Date(dateMathStart.value)
    const amount = dateMathOp.value === 'sub' ? -dateMathAmount.value : dateMathAmount.value
    switch (dateMathUnit.value) {
      case 'days':
        d.setDate(d.getDate() + amount)
        break
      case 'months':
        d.setMonth(d.getMonth() + amount)
        break
      case 'years':
        d.setFullYear(d.getFullYear() + amount)
        break
    }
    const weekdays = ['星期日', '星期一', '星期二', '星期三', '星期四', '星期五', '星期六']
    dateMathResult.value = `结果: ${d.toLocaleDateString('zh-CN')}\n星期: ${weekdays[d.getDay()]} (${d.toISOString().slice(0, 10)})`
  } catch {
    dateMathResult.value = '错误: 计算失败'
    toast.error('日期计算失败')
  }
}

/* Date Diff */
const diffStart = ref(new Date().toISOString().slice(0, 10))
const diffEnd = ref(new Date(Date.now() + 30 * 86400000).toISOString().slice(0, 10))
const diffResult = ref('')

function calcDateDiff() {
  if (!diffStart.value || !diffEnd.value) { toast.error('请选择两个日期'); return }
  try {
    const start = new Date(diffStart.value)
    const end = new Date(diffEnd.value)
    let diffMs = Math.abs(end.getTime() - start.getTime())
    const days = Math.floor(diffMs / 86400000)
    const weeks = Math.floor(days / 7)
    const hours = Math.floor(diffMs / 3600000)
    const minutes = Math.floor(diffMs / 60000)
    const months = Math.floor(days / 30.44)
    const years = Math.floor(days / 365.25)
    diffResult.value = `天数: ${days} 天\n周数: ${weeks} 周 ${days % 7} 天\n月数: 约 ${months} 个月\n年数: 约 ${years} 年\n小时: ${hours} 小时\n分钟: ${minutes} 分钟`
  } catch {
    diffResult.value = '错误: 计算失败'
    toast.error('日期差值计算失败')
  }
}

/* Workday Calculator */
const workdayStart = ref(new Date().toISOString().slice(0, 10))
const workdayCount = ref(10)
const workdayOp = ref<'add' | 'sub'>('add')
const workdayResult = ref('')

function calcWorkday() {
  if (!workdayStart.value) { toast.error('请选择开始日期'); return }
  try {
    const d = new Date(workdayStart.value)
    const count = workdayOp.value === 'sub' ? -workdayCount.value : workdayCount.value
    let remaining = Math.abs(count)
    const dir = count >= 0 ? 1 : -1
    while (remaining > 0) {
      d.setDate(d.getDate() + dir)
      const dow = d.getDay()
      if (dow !== 0 && dow !== 6) {
        remaining--
      }
    }
    const weekdays = ['星期日', '星期一', '星期二', '星期三', '星期四', '星期五', '星期六']
    workdayResult.value = `结果: ${d.toLocaleDateString('zh-CN')}\n星期: ${weekdays[d.getDay()]} (${d.toISOString().slice(0, 10)})`
  } catch {
    workdayResult.value = '错误: 计算失败'
    toast.error('工作日计算失败')
  }
}
</script>
