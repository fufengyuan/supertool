<template>
  <div class="server-group-section">
    <div class="server-group-header" @click="toggleGroup" :style="{ '--group-color': group.color || '#6c63ff' }">
      <svg class="group-chevron" :class="{ expanded: expanded }" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2.5">
        <polyline points="6 9 12 15 18 9"/>
      </svg>
      <span class="group-label">{{ group.name }}</span>
      <span class="group-count" :style="{ background: (group.color || '#6c63ff') + '22', color: group.color || '#6c63ff' }">
        {{ directServers.length }}
      </span>
    </div>
    <Transition name="server-group-expand">
      <div v-show="expanded" class="server-group-body">
        <!-- 该分组直属的服务器 -->
        <template v-if="mode === 'multi'">
          <label v-for="server in directServers" :key="server.id" class="server-check-item">
            <input type="checkbox" :value="server.id" :checked="modelValue.includes(server.id)" @change="onMultiToggle(server.id, $event)" />
            <span class="server-check-name">{{ server.name }}</span>
            <span class="server-check-addr">{{ server.host }}:{{ server.port || 22 }}</span>
          </label>
        </template>
        <template v-else>
          <div v-for="server in directServers" :key="server.id" class="server-option-item" :class="{ active: modelValue === server.id }" @click="onSingleSelect(server.id)">
            <span class="server-option-name">{{ server.name }}</span>
            <span class="server-option-addr">{{ server.host }}:{{ server.port || 22 }}</span>
          </div>
        </template>
        <!-- 递归渲染子分组 -->
        <ServerGroupNode
          v-for="child in childGroups"
          :key="child.id"
          :group="child"
          :groups="groups"
          :servers="servers"
          :model-value="modelValue"
          :mode="mode"
          :expanded-groups="expandedGroups"
          @update:model-value="$emit('update:modelValue', $event)"
          @toggle-group="$emit('toggleGroup', $event)"
        />
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Server } from '../../types'

// Vue 3 <script setup> auto-registers recursive components by filename.
// The component references itself in the template as <ServerGroupNode>.
import ServerGroupNode from './ServerGroupNode.vue'

interface GroupNode {
  id: string
  name: string
  color: string
  parentId?: string | null
}

const props = defineProps<{
  group: GroupNode
  groups: GroupNode[]
  servers: Server[]
  modelValue: string | string[]
  mode: 'single' | 'multi'
  expandedGroups: Set<string | null>
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string | string[]]
  'toggleGroup': [groupId: string]
}>()

const childGroups = computed(() => props.groups.filter(g => g.parentId === props.group.id))
const directServers = computed(() => props.servers.filter(s => s.groupId === props.group.id))
const expanded = computed(() => props.expandedGroups.has(props.group.id))

function toggleGroup() {
  emit('toggleGroup', props.group.id)
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
  background: var(--input-bg);
  border: 1px solid var(--border-color);
}

.server-group-header:hover {
  border-color: var(--primary-color);
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
  padding: 2px 0 2px 12px;
}

/* 多级缩进 */
.server-group-section .server-group-section .server-group-body {
  padding-left: 12px;
}

/* 多选模式 */
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
  accent-color: var(--primary-color);
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

/* 单选模式 */
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
  background: var(--primary-color);
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
