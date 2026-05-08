<template>
  <div class="flex flex-col gap-0.5" :class="{ 'selector-mode': mode === 'single' }">
    <!-- 未分组服务器 -->
    <div v-if="ungroupedServers.length > 0" class="rounded-lg overflow-hidden">
      <div class="flex items-center gap-1.5 px-2 py-1.5 rounded-lg cursor-pointer select-none bg-base-200 border border-base-content/10 transition-all hover:border-primary" @click="toggleGroup(null)">
        <svg class="text-base-content/60 transition-transform flex-shrink-0" :class="{ 'rotate-90': expandedGroups.has(null) }" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2.5">
          <polyline points="6 9 12 15 18 9"/>
        </svg>
        <span class="text-xs font-semibold text-base-content flex-1">🖥️ 未分组</span>
        <span class="text-[10px] font-semibold px-1.5 py-px rounded-full bg-primary/15 text-primary">{{ ungroupedServers.length }}</span>
      </div>
      <Transition
        enter-active-class="transition-all duration-200 ease-out"
        leave-active-class="transition-all duration-200 ease-in"
        enter-from-class="opacity-0"
        leave-to-class="opacity-0"
      >
        <div v-show="expandedGroups.has(null)" class="py-0.5 pl-2">
          <template v-if="mode === 'multi'">
            <label v-for="server in ungroupedServers" :key="server.id" class="flex items-center gap-1.5 px-2 py-1 rounded cursor-pointer text-xs hover:bg-base-content/5">
              <input type="checkbox" :value="server.id" :checked="modelValue.includes(server.id)" @change="onMultiToggle(server.id, $event)" class="checkbox checkbox-primary checkbox-xs" />
              <span class="text-base-content font-medium min-w-[60px] truncate">{{ server.name }}</span>
              <span class="text-base-content/40 text-[11px] font-mono ml-auto">{{ server.host }}:{{ server.port || 22 }}</span>
            </label>
          </template>
          <template v-else>
            <div v-for="server in ungroupedServers" :key="server.id" class="flex items-center gap-1.5 px-2 py-1.5 rounded cursor-pointer text-xs hover:bg-base-content/5"
              :class="{ 'bg-primary text-white': modelValue === server.id }"
              @click="onSingleSelect(server.id)">
              <span class="font-medium text-base-content min-w-[60px] truncate" :class="{ '!text-white': modelValue === server.id }">{{ server.name }}</span>
              <span class="text-base-content/40 text-[11px] font-mono ml-auto" :class="{ '!text-white/70': modelValue === server.id }">{{ server.host }}:{{ server.port || 22 }}</span>
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

    <div v-if="allServers.length === 0" class="text-center p-4 text-base-content/60 text-xs">暂无服务器</div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { Server } from '../../types'
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
