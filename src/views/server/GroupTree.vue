<template>
  <div class="mb-1 rounded-xl" :class="{ 'mb-2': isExpanded }">
    <div class="flex items-center justify-between p-[7px_12px] rounded-xl cursor-pointer select-none bg-base-100 border border-base-content/10 relative overflow-hidden transition-all hover:shadow-[0_2px_12px_rgba(108,99,255,0.1)] hover:-translate-y-px"
      :style="{ '--group-color': group.color || '#6c63ff', borderColor: expandedGroups.has(group.id) ? 'var(--group-color, var(--color-primary))' : undefined }"
      @click="toggle">
      <div class="absolute left-0 top-0 bottom-0 w-[3px] rounded-r-[3px]" :style="{ background: group.color || '#6c63ff' }"></div>
      <div class="flex items-center gap-2 relative z-[1]">
        <SvgIcon class="text-base-content/60 transition-transform flex-shrink-0" :class="{ 'rotate-180': isExpanded }" :style="{ color: isExpanded ? (group.color || '#6c63ff') : undefined }" name="chevronDown" size="14" strokeWidth="2.5" />
        <span class="text-sm leading-none"><SvgIcon :name="getGroupIconName(depth)" size="14" /></span>
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
import SvgIcon from '@/components/ui/SvgIcon.vue'
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

function getGroupIconName(depth: number): string {
  const icons = ['folder', 'folder', 'file', 'upload', 'tag']
  return icons[depth % icons.length]
}

function toggle() {
  emit('toggle', props.group.id);
}
</script>
