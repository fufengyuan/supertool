<template>
  <div class="time-calc-tool">
    <h3>📅 时间计算器</h3>

    <!-- Date Math -->
    <div class="tool-section">
      <h4>日期加减</h4>
      <div class="tool-row">
        <input v-model="dateMathStart" type="date" class="tool-input" />
        <select v-model="dateMathUnit" class="tool-select">
          <option value="days">天</option>
          <option value="months">月</option>
          <option value="years">年</option>
        </select>
        <input v-model.number="dateMathAmount" type="number" class="tool-input mono" style="max-width: 100px" placeholder="数量" />
        <select v-model="dateMathOp" class="tool-select">
          <option value="add">加</option>
          <option value="sub">减</option>
        </select>
        <button class="tool-btn primary" @click="calcDateMath">计算</button>
        <button class="tool-btn" @click="copyText(dateMathResult, toast)">📋</button>
      </div>
      <div v-if="dateMathResult" class="tool-result">{{ dateMathResult }}</div>
    </div>

    <hr class="tool-divider" />

    <!-- Date Difference -->
    <div class="tool-section">
      <h4>日期差值</h4>
      <div class="tool-row">
        <div>
          <label class="tool-label">开始日期</label>
          <input v-model="diffStart" type="date" class="tool-input" />
        </div>
        <div>
          <label class="tool-label">结束日期</label>
          <input v-model="diffEnd" type="date" class="tool-input" />
        </div>
        <button class="tool-btn primary" @click="calcDateDiff" style="align-self: flex-end">计算</button>
        <button class="tool-btn" @click="copyText(diffResult, toast)" style="align-self: flex-end">📋</button>
      </div>
      <div v-if="diffResult" class="tool-result">{{ diffResult }}</div>
    </div>

    <hr class="tool-divider" />

    <!-- Workday Calculator -->
    <div class="tool-section">
      <h4>工作日计算（跳过周末）</h4>
      <div class="tool-row">
        <div>
          <label class="tool-label">开始日期</label>
          <input v-model="workdayStart" type="date" class="tool-input" />
        </div>
        <div>
          <label class="tool-label">工作日数量</label>
          <input v-model.number="workdayCount" type="number" class="tool-input mono" style="max-width: 100px" min="1" />
        </div>
        <select v-model="workdayOp" class="tool-select" style="align-self: flex-end">
          <option value="add">加</option>
          <option value="sub">减</option>
        </select>
        <button class="tool-btn primary" @click="calcWorkday" style="align-self: flex-end">计算</button>
        <button class="tool-btn" @click="copyText(workdayResult, toast)" style="align-self: flex-end">📋</button>
      </div>
      <div v-if="workdayResult" class="tool-result">{{ workdayResult }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

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

<style scoped>

.time-calc-tool {
  max-width: 700px;
}

.time-calc-tool h3 {
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
  align-items: flex-end;
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

.tool-select {
  padding: 7px 10px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 6px;
  font-size: 13px;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  outline: none;
}

.tool-label {
  font-size: 12px;
  font-weight: 500;
  color: oklch(var(--bc) / 0.6);
  margin-bottom: 4px;
  display: block;
}

.mono {
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
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
