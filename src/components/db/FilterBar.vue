<template>
  <div class="filter-bar">
    <!-- Filter toggle & controls -->
    <div class="filter-header">
      <button @click="toggleFilter" class="filter-toggle-btn" :class="{ active: enabled }">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3" />
        </svg>
        <span>筛选</span>
        <span v-if="activeCount > 0" class="filter-badge">{{ activeCount }}</span>
      </button>
      <div v-if="enabled" class="filter-actions">
        <button @click="addCondition" class="btn btn-ghost btn-xs">+ 添加条件</button>
        <button @click="clearAll" class="btn btn-ghost btn-xs" :disabled="conditions.length === 0">清除全部</button>
        <button @click="apply" class="btn btn-primary btn-xs" :disabled="conditions.length === 0">🔍 应用筛选</button>
      </div>
    </div>

    <!-- Filter conditions -->
    <div v-if="enabled && conditions.length > 0" class="filter-conditions">
      <div v-for="(cond, idx) in conditions" :key="cond.id" class="filter-row">
        <!-- Logic connector -->
        <div class="filter-logic">
          <select v-model="cond.logic" class="filter-select" v-if="idx > 0">
            <option value="AND">AND</option>
            <option value="OR">OR</option>
          </select>
          <span v-else class="where-label">WHERE</span>
        </div>

        <!-- Column selector -->
        <div class="filter-column">
          <select v-model="cond.column" class="filter-select" @change="onColumnChange(cond)">
            <option value="">选择列</option>
            <option v-for="col in columns" :key="col" :value="col">{{ col }}</option>
          </select>
        </div>

        <!-- Operator selector -->
        <div class="filter-operator">
          <select v-model="cond.operator" class="filter-select">
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
        <div class="filter-value" v-if="needsValue(cond.operator)">
          <input
            v-if="cond.operator !== 'BETWEEN'"
            v-model="cond.value"
            class="filter-input"
            :placeholder="getValuePlaceholder(cond.operator)"
            @keydown.enter="apply"
          />
          <template v-else>
            <input v-model="cond.value" class="filter-input filter-input-between" placeholder="最小值" />
            <span class="between-sep">至</span>
            <input v-model="cond.value2" class="filter-input filter-input-between" placeholder="最大值" />
          </template>
        </div>

        <!-- Remove button -->
        <button @click="removeCondition(idx)" class="filter-remove" title="删除条件">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
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

<style scoped>
.filter-bar {
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
}

.filter-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 0;
}

.filter-toggle-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 12px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 6px;
  background: transparent;
  color: oklch(var(--bc) / 0.6);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.filter-toggle-btn:hover {
  background: oklch(var(--p) / 0.1);
  color: oklch(var(--p));
}

.filter-toggle-btn.active {
  background: oklch(var(--p));
  color: white;
  border-color: oklch(var(--p));
}

.filter-badge {
  background: rgba(255, 255, 255, 0.3);
  color: white;
  padding: 1px 6px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 600;
}

.filter-toggle-btn:not(.active) .filter-badge {
  background: oklch(var(--p));
  color: white;
}

.filter-actions {
  display: flex;
  gap: 6px;
  align-items: center;
}

.filter-conditions {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 6px 0;
}

.filter-row {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 0;
}

.filter-logic {
  width: 60px;
  flex-shrink: 0;
}

.where-label {
  font-size: 12px;
  font-weight: 700;
  color: oklch(var(--p));
  padding: 0 4px;
}

.filter-column {
  min-width: 120px;
  flex-shrink: 0;
}

.filter-operator {
  min-width: 130px;
  flex-shrink: 0;
}

.filter-value {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 6px;
}

.filter-select,
.filter-input {
  padding: 5px 8px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 4px;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  font-size: 12px;
  outline: none;
  transition: border-color 0.15s ease;
}

.filter-select:focus,
.filter-input:focus {
  border-color: oklch(var(--p));
  box-shadow: 0 0 0 2px oklch(var(--p) / 0.1);
}

.filter-input {
  width: 100%;
}

.filter-input-between {
  width: 100px;
}

.between-sep {
  font-size: 12px;
  color: oklch(var(--bc) / 0.6);
  white-space: nowrap;
}

.filter-remove {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  background: transparent;
  border-radius: 4px;
  cursor: pointer;
  color: oklch(var(--bc) / 0.6);
  flex-shrink: 0;
  transition: all 0.1s ease;
}

.filter-remove:hover {
  background: #ffebee;
  color: #f44336;
}

.btn-xs {
  padding: 4px 8px;
  font-size: 11px;
  border-radius: 4px;
  border: 1px solid oklch(var(--bc) / 0.1);
  background: transparent;
  color: oklch(var(--bc) / 0.6);
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-xs:hover {
  background: oklch(var(--p) / 0.1);
  color: oklch(var(--p));
}

.btn-xs:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-primary.btn-xs {
  background: oklch(var(--p));
  color: white;
  border-color: oklch(var(--p));
}

.btn-primary.btn-xs:hover {
  background: var(--primary-dark, #4338ca);
  color: white;
}

.btn-primary.btn-xs:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
