<template>
  <div class="grouped-server-selector" :class="{ 'selector-mode': mode === 'single' }">
    <!-- 未分组服务器 -->
    <div v-if="ungroupedServers.length > 0" class="server-group-section">
      <div class="server-group-header" @click="toggleGroup(null)">
        <svg class="group-chevron" :class="{ expanded: expandedGroups.has(null) }" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2.5">
          <polyline points="6 9 12 15 18 9"/>
        </svg>
        <span class="group-label">🖥️ 未分组</span>
        <span class="group-count">{{ ungroupedServers.length }}</span>
      </div>
      <Transition name="server-group-expand">
        <div v-show="expandedGroups.has(null)" class="server-group-body">
          <template v-if="mode === 'multi'">
            <label v-for="server in ungroupedServers" :key="server.id" class="server-check-item">
              <input type="checkbox" :value="server.id" :checked="modelValue.includes(server.id)" @change="onMultiToggle(server.id, $event)" />
              <span class="server-check-name">{{ server.name }}</span>
              <span class="server-check-addr">{{ server.host }}:{{ server.port || 22 }}</span>
            </label>
          </template>
          <template v-else>
            <div v-for="server in ungroupedServers" :key="server.id" class="server-option-item" :class="{ active: modelValue === server.id }" @click="onSingleSelect(server.id)">
              <span class="server-option-name">{{ server.name }}</span>
              <span class="server-option-addr">{{ server.host }}:{{ server.port || 22 }}</span>
            </div>
          </template>
        </div>
      </Transition>
    </div>

    <!-- 根分组（递归渲染，支持多级嵌套） -->
    <ServerGroupNode
      v-for="group in rootGroups"
      :key="group.id"
      :group="group"
      :groups="groups"
      :servers="servers"
      :model-value="modelValue"
      :mode="mode"
      :expanded-groups="expandedGroups"
      @update:model-value="$emit('update:modelValue', $event)"
      @toggle-group="toggleGroup"
    />

    <div v-if="allServers.length === 0" class="empty-servers">暂无服务器</div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { Server } from '@/types'
import ServerGroupNode from './ServerGroupNode.vue'

interface GroupNode {
  id: string
  name: string
  color: string
  parentId?: string | null
}

const props = defineProps<{
  servers: Server[]
  groups: GroupNode[]
  modelValue: string | string[]  // string for single, string[] for multi
  mode?: 'single' | 'multi'  // default: 'multi'
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string | string[]]
}>()

const mode = computed(() => props.mode || 'multi')
const allServers = computed(() => props.servers)

// 未分组的服务器
const ungroupedServers = computed(() => props.servers.filter(s => !s.groupId))

// 根分组：parentId 为 null/undefined 或指向不存在的分组
const rootGroups = computed(() => {
  const groupIds = new Set(props.groups.map(g => g.id))
  return props.groups.filter(g => {
    const pid = g.parentId
    return !pid || !groupIds.has(pid)
  })
})

// 默认展开所有分组
const expandedGroups = ref(new Set<string | null>([null]))
for (const g of props.groups) {
  expandedGroups.value.add(g.id)
}

function toggleGroup(groupId: string | null) {
  if (expandedGroups.value.has(groupId)) {
    expandedGroups.value.delete(groupId)
  } else {
    expandedGroups.value.add(groupId)
  }
  expandedGroups.value = new Set(expandedGroups.value)
}

function onMultiToggle(serverId: string, event: Event) {
  const checked = (event.target as HTMLInputElement).checked
  const current = Array.isArray(props.modelValue) ? [...props.modelValue] : []
  if (checked) {
    if (!current.includes(serverId)) current.push(serverId)
  } else {
    const idx = current.indexOf(serverId)
    if (idx >= 0) current.splice(idx, 1)
  }
  emit('update:modelValue', current)
}

function onSingleSelect(serverId: string) {
  emit('update:modelValue', serverId)
}
</script>

<style scoped>
.grouped-server-selector {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.server-group-section {
  border-radius: 6px;
  overflow: hidden;
}

.server-group-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 8px;
  border-radius: 6px;
  cursor: pointer;
  user-select: none;
  transition: all 0.15s ease;
  background: var(--color-base-200);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.server-group-header:hover {
  border-color: var(--color-primary);
}

.group-chevron {
  color: var(--text-secondary);
  transition: transform 0.2s ease;
  flex-shrink: 0;
}

.group-chevron.expanded {
  transform: rotate(90deg);
}

.group-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
  flex: 1;
}

.group-count {
  font-size: 10px;
  font-weight: 600;
  padding: 1px 6px;
  border-radius: 10px;
  background: #6c63ff22;
  color: #6c63ff;
}

.server-group-body {
  padding: 2px 0 2px 8px;
}

/* 多选模式 — 复选框 */
.server-check-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.1s;
  font-size: 12px;
}

.server-check-item:hover {
  background: var(--hover-bg);
}

.server-check-item input[type="checkbox"] {
  accent-color: var(--color-primary);
  flex-shrink: 0;
}

.server-check-name {
  color: var(--text-primary);
  font-weight: 500;
  min-width: 60px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.server-check-addr {
  color: var(--text-tertiary);
  font-size: 11px;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  margin-left: auto;
}

/* 单选模式 — 选项 */
.server-option-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 8px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.1s;
  font-size: 12px;
}

.server-option-item:hover {
  background: var(--hover-bg);
}

.server-option-item.active {
  background: var(--color-primary);
  color: white;
}

.server-option-item.active .server-option-addr {
  color: rgba(255, 255, 255, 0.7);
}

.server-option-name {
  font-weight: 500;
  color: var(--text-primary);
  min-width: 60px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.server-option-addr {
  color: var(--text-tertiary);
  font-size: 11px;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  margin-left: auto;
}

.empty-servers {
  text-align: center;
  padding: 16px;
  color: var(--text-secondary);
  font-size: 12px;
}

/* 动画 */
.server-group-expand-enter-active,
.server-group-expand-leave-active {
  transition: all 0.2s ease;
  overflow: hidden;
}

.server-group-expand-enter-from,
.server-group-expand-leave-to {
  opacity: 0;
  max-height: 0;
}

.server-group-expand-enter-to,
.server-group-expand-leave-from {
  opacity: 1;
  max-height: 500px;
}
</style>
