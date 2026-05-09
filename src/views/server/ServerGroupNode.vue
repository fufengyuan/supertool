<template>
  <div class="rounded-lg overflow-hidden">
    <div class="flex items-center gap-1.5 px-2 py-1.5 rounded-lg cursor-pointer select-none bg-base-200 border border-base-content/10 transition-all hover:border-primary"
      @click="toggleGroup" :style="{ '--group-color': group.color || '#6c63ff' }">
      <SvgIcon name="chevronDown" size="12" :strokeWidth="2.5" class="text-base-content/60 transition-transform shrink-0" :class="{ 'rotate-90': expanded }" />
      <span class="text-xs font-semibold text-base-content flex-1">{{ group.name }}</span>
      <span class="text-[10px] font-semibold px-1.5 py-px rounded-full" :style="{ background: (group.color || '#6c63ff') + '22', color: group.color || '#6c63ff' }">
        {{ directServers.length }}
      </span>
    </div>
    <Transition
      enter-active-class="transition-all duration-200 ease-out"
      leave-active-class="transition-all duration-200 ease-in"
      enter-from-class="opacity-0"
      leave-to-class="opacity-0"
    >
      <div v-show="expanded" class="py-0.5 pl-3">
        <!-- 该分组直属的服务器 -->
        <template v-if="mode === 'multi'">
          <label v-for="server in directServers" :key="server.id" class="flex items-center gap-1.5 px-2 py-1 rounded cursor-pointer text-xs hover:bg-base-content/5">
            <input type="checkbox" :value="server.id" :checked="modelValue.includes(server.id)" @change="onMultiToggle(server.id, $event)" class="checkbox checkbox-primary checkbox-xs" />
            <span class="text-base-content font-medium min-w-[60px] truncate">{{ server.name }}</span>
            <span class="text-base-content/40 text-[11px] font-mono ml-auto">{{ server.host }}:{{ server.port || 22 }}</span>
          </label>
        </template>
        <template v-else>
          <div v-for="server in directServers" :key="server.id" class="flex items-center gap-1.5 px-2 py-1.5 rounded cursor-pointer text-xs hover:bg-base-content/5"
            :class="{ 'bg-primary text-white': modelValue === server.id }"
            @click="onSingleSelect(server.id)">
            <span class="font-medium text-base-content min-w-[60px] truncate" :class="{ '!text-white': modelValue === server.id }">{{ server.name }}</span>
            <span class="text-base-content/40 text-[11px] font-mono ml-auto" :class="{ '!text-white/70': modelValue === server.id }">{{ server.host }}:{{ server.port || 22 }}</span>
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
import SvgIcon from '@/components/ui/SvgIcon.vue'

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
