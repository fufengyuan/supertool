<template>
  <div class="border-b border-base-content/10">
    <!-- Filter toggle & controls -->
    <div class="flex items-center justify-between py-1.5">
      <button @click="toggleFilter" :class="[enabled ? 'bg-primary text-white border-primary' : 'bg-transparent text-base-content/60 border-base-content/10']" class="flex items-center gap-1.5 px-3 py-1.25 border rounded-md text-xs cursor-pointer transition-all duration-150 hover:bg-primary/10 hover:text-primary">
        <SvgIcon name="filter" size="14" />
        <span>筛选</span>
        <span v-if="activeCount > 0" :class="enabled ? 'bg-white/30 text-white' : 'bg-primary text-white'" class="px-1.5 py-0.5 rounded-full text-[11px] font-semibold">{{ activeCount }}</span>
      </button>
      <div v-if="enabled" class="flex gap-1.5 items-center">
        <button @click="addCondition" class="btn btn-ghost btn-xs gap-1.5"><SvgIcon name="plus" size="14" /> 添加条件</button>
        <button @click="clearAll" class="btn btn-ghost btn-xs" :disabled="conditions.length === 0">清除全部</button>
        <button @click="apply" class="btn btn-primary btn-xs gap-1.5" :disabled="conditions.length === 0"><SvgIcon name="search" size="14" /> 应用筛选</button>
      </div>
    </div>

    <!-- Filter conditions -->
    <div v-if="enabled && conditions.length > 0" class="flex flex-col gap-1 py-1.5">
      <div v-for="(cond, idx) in conditions" :key="cond.id" class="flex items-center gap-1.5 py-1">
        <!-- Logic connector -->
        <div class="w-[60px] shrink-0">
          <select v-model="cond.logic" class="select select-bordered select-xs w-full" v-if="idx > 0">
            <option value="AND">AND</option>
            <option value="OR">OR</option>
          </select>
          <span v-else class="text-xs font-bold text-primary px-1">WHERE</span>
        </div>

        <!-- Column selector -->
        <div class="min-w-[120px] shrink-0">
          <select v-model="cond.column" class="select select-bordered select-xs w-full" @change="onColumnChange(cond)">
            <option value="">选择列</option>
            <option v-for="col in columns" :key="col" :value="col">{{ col }}</option>
          </select>
        </div>

        <!-- Operator selector -->
        <div class="min-w-[130px] shrink-0">
          <select v-model="cond.operator" class="select select-bordered select-xs w-full">
            <option value="">操作符</option>
            <optgroup label="比较">
              <option value="=">= 等于</option>
              <option value="!=">!= 不等于</option>
              <option value=">">&gt; 大于</option>
              <option value="<">&lt; 小于</option>
              <option value=">=">&gt;= 大于等于</option>
              <option value="<=">&lt;= 小于等于</option>
            </optgroup>
            <optgroup label="模糊匹配">
              <option value="LIKE">LIKE 包含</option>
              <option value="NOT LIKE">NOT LIKE 不包含</option>
            </optgroup>
            <optgroup label="范围/集合">
              <option value="IN">IN 在列表中</option>
              <option value="NOT IN">NOT IN 不在列表中</option>
              <option value="BETWEEN">BETWEEN 范围</option>
            </optgroup>
            <optgroup label="空值">
              <option value="IS NULL">IS NULL 为空</option>
              <option value="IS NOT NULL">IS NOT NULL 不为空</option>
            </optgroup>
          </select>
        </div>

        <!-- Value input -->
        <div class="flex-1 flex items-center gap-1.5" v-if="needsValue(cond.operator)">
          <input
            v-if="cond.operator !== 'BETWEEN'"
            v-model="cond.value"
            class="input input-bordered input-xs w-full"
            :placeholder="getValuePlaceholder(cond.operator)"
            @keydown.enter="apply"
          />
          <template v-else>
            <input v-model="cond.value" class="input input-bordered input-xs w-[100px]" placeholder="最小值" />
            <span class="text-xs text-base-content/60 whitespace-nowrap">至</span>
            <input v-model="cond.value2" class="input input-bordered input-xs w-[100px]" placeholder="最大值" />
          </template>
        </div>

        <!-- Remove button -->
        <button @click="removeCondition(idx)" class="flex items-center justify-center w-6 h-6 border-none bg-transparent rounded cursor-pointer text-base-content/60 shrink-0 transition-all duration-100 hover:bg-error/10 hover:text-error" title="删除条件">
          <SvgIcon name="x" size="14" />
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref, computed } from 'vue'

export interface FilterCondition {
  id: string
  column: string
  operator: string
  value: string
  value2?: string
  logic: 'AND' | 'OR'
}

const props = defineProps<{
  columns: string[]
}>()

const emit = defineEmits<{
  apply: [conditions: FilterCondition[]]
  clear: []
}>()

const enabled = ref(false)
const conditions = ref<FilterCondition[]>([])

const activeCount = computed(() =>
  conditions.value.filter(c => c.column && c.operator).length
)

function generateId(): string {
  return 'fc_' + Date.now().toString(36) + Math.random().toString(36).slice(2, 6)
}

function toggleFilter() {
  enabled.value = !enabled.value
  if (!enabled.value) {
    emit('clear')
  }
}

function addCondition() {
  conditions.value.push({
    id: generateId(),
    column: '',
    operator: '',
    value: '',
    value2: '',
    logic: conditions.value.length > 0 ? 'AND' : 'AND'
  })
}

function removeCondition(idx: number) {
  conditions.value.splice(idx, 1)
}

function clearAll() {
  conditions.value = []
  emit('clear')
}

function apply() {
  const valid = conditions.value.filter(c => c.column && c.operator)
  emit('apply', valid)
}

function needsValue(op: string): boolean {
  return !['IS NULL', 'IS NOT NULL'].includes(op)
}

function getValuePlaceholder(op: string): string {
  switch (op) {
    case 'LIKE':
    case 'NOT LIKE':
      return '输入搜索文本（自动添加 %）'
    case 'IN':
    case 'NOT IN':
      return '值1, 值2, 值3...'
    default:
      return '输入值'
  }
}

function onColumnChange(_cond: FilterCondition) {
  // Could auto-select operator based on column type in future
}

// Expose for parent component
defineExpose({ enabled, conditions, activeCount })
</script>
