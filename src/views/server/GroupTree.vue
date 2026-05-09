<template>
  <div class="mb-1 rounded-xl" :class="{ 'mb-2': isExpanded }">
    <div class="flex items-center justify-between p-[7px_12px] rounded-xl cursor-pointer select-none bg-base-100 border border-base-content/10 relative overflow-hidden transition-all hover:shadow-[0_2px_12px_rgba(108,99,255,0.1)] hover:-translate-y-px"
      :style="{ '--group-color': group.color || '#6c63ff', borderColor: expandedGroups.has(group.id) ? 'var(--group-color, var(--color-primary))' : undefined }"
      @click="toggle">
      <div class="absolute left-0 top-0 bottom-0 w-[3px] rounded-r-[3px]" :style="{ background: group.color || '#6c63ff' }"></div>
      <div class="flex items-center gap-2 relative z-[1]">
        <svg class="text-base-content/60 transition-transform flex-shrink-0" :class="{ 'rotate-180': isExpanded }" :style="{ color: isExpanded ? (group.color || '#6c63ff') : undefined }" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="6 9 12 15 18 9"/>
        </svg>
        <span class="text-sm leading-none" v-html="getGroupIcon(depth)"></span>
        <span class="font-semibold text-xs text-base-content">{{ group.name }}</span>
        <span class="text-[11px] font-semibold px-1.5 py-px rounded-full leading-tight" :style="{ background: (group.color || '#6c63ff') + '22', color: group.color || '#6c63ff' }">
          {{ serversInGroup.length }}
        </span>
      </div>
      <div class="flex items-center gap-2 relative z-[1]">
        <span class="flex items-center gap-1 text-xs text-success font-medium" v-if="onlineCount > 0">
          <span class="w-1.5 h-1.5 rounded-full bg-success shadow-[0_0_4px_var(--color-success)]"></span>
          {{ onlineCount }} 在线
        </span>
      </div>
    </div>

    <Transition
      enter-active-class="transition-all duration-300 ease-out"
      leave-active-class="transition-all duration-300 ease-in"
      enter-from-class="opacity-0 -translate-y-1.5"
      leave-to-class="opacity-0 -translate-y-1.5"
    >
      <div v-show="isExpanded" class="mt-1 p-2 rounded-xl bg-base-100/80 border border-base-content/10 border-t-0" :style="{ paddingLeft: `${10 + depth * 6}px` }">
        <!-- 该分组下的服务器 -->
        <div v-if="serversInGroup.length > 0" class="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-2">
          <ServerItem
            v-for="server in serversInGroup"
            :key="server.id"
            :server="server"
            :connection-status="connectionStatusMap[server.id] || 'offline'"
            @terminal="$emit('terminal', server)"
            @sftp="$emit('sftp', server)"
            @edit="$emit('edit', server)"
            @delete="$emit('delete', server.id)"
          />
        </div>
        <!-- 子分组递归 -->
        <div v-if="childGroups.length > 0" class="mt-1.5 pl-1">
          <GroupTree
            v-for="child in childGroups"
            :key="child.id"
            :group="child"
            :groups="groups"
            :depth="depth + 1"
            :expanded-groups="expandedGroups"
            :servers="servers"
            :connection-status-map="connectionStatusMap"
            @toggle="$emit('toggle', $event)"
            @terminal="$emit('terminal', $event)"
            @sftp="$emit('sftp', $event)"
            @edit="$emit('edit', $event)"
            @delete="$emit('delete', $event)"
          />
        </div>
        <div v-if="serversInGroup.length === 0 && childGroups.length === 0" class="text-center p-3 text-base-content/60 text-xs bg-base-200 rounded-lg border border-dashed border-base-content/10">
          暂无服务器
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import ServerItem from './ServerItem.vue';

interface GroupNode {
  id: string;
  name: string;
  color: string;
  parentId: string | null;
}

const props = defineProps<{
  group: GroupNode;
  groups: GroupNode[];
  depth: number;
  expandedGroups: Set<string | null>;
  servers: any[];
  connectionStatusMap: Record<string, string>;
}>();

const emit = defineEmits(['toggle', 'terminal', 'sftp', 'edit', 'delete']);

const childGroups = computed(() => {
  return props.groups.filter(g => g.parentId === props.group.id);
});

const serversInGroup = computed(() => {
  return props.servers.filter(s => s.groupId === props.group.id);
});

const onlineCount = computed(() => {
  return serversInGroup.value.filter(s => props.connectionStatusMap[s.id] === 'online').length;
});

const isExpanded = computed(() => {
  return props.expandedGroups.has(props.group.id as string | null);
});

function getGroupIcon(depth: number): string {
  const icons = [
    '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>',
    '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>',
    '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>',
    '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 16 12 12 8 16"/><line x1="12" y1="12" x2="12" y2="21"/><path d="M20.39 18.39A5 5 0 0 0 18 9h-1.26A8 8 0 1 0 3 16.3"/></svg>',
    '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/><line x1="7" y1="7" x2="7.01" y2="7"/></svg>',
  ];
  return icons[depth % icons.length];
}

function toggle() {
  emit('toggle', props.group.id);
}
</script>
