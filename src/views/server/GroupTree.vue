<template>
  <!-- 一个分组 = 一块中性面板（左侧 2px 分组色条作标识），标题行 + 卡片 + 子分组都嵌在里面 -->
  <div class="mb-1 rounded-lg border border-base-content/5 border-l-2 bg-base-100/40" :style="{ borderLeftColor: group.color || '#6b7280' }">
    <!-- 分组标题栏 -->
    <div class="flex items-center gap-2 px-2 py-1 rounded-lg cursor-pointer select-none transition-colors hover:bg-base-100"
      @click="toggle">
      <!-- 展开/折叠箭头 -->
      <SvgIcon class="text-base-content/50 transition-transform flex-shrink-0" 
        :class="{ 'rotate-180': isExpanded }" 
        name="chevronDown" size="12" strokeWidth="2.5" />
      <!-- 分组颜色标记 -->
      <span class="w-2 h-2 rounded-full flex-shrink-0" :style="{ backgroundColor: group.color || '#6c63ff' }"></span>
      <!-- 分组名称 -->
      <span class="font-medium text-[11px] text-base-content">{{ group.name }}</span>
      <!-- 服务器数量（含子分组） -->
      <span class="text-[10px] px-1.5 py-0 rounded bg-base-content/5 text-base-content/55 leading-tight tabular-nums"
        :title="`含子分组共 ${allServersInGroup.length} 台`">
        {{ allServersInGroup.length }}
      </span>
      <!-- 在线数量 -->
      <span class="flex items-center gap-1 text-[10px] text-success ml-auto" v-if="onlineCount > 0">
        <span class="w-1 h-1 rounded-full bg-success"></span>
        {{ onlineCount }}
      </span>
    </div>

    <!-- 面板内容：本组服务器 + 子分组（子分组各自再是一块面板） -->
    <Transition
      enter-active-class="transition-all duration-200 ease-out"
      leave-active-class="transition-all duration-200 ease-in"
      enter-from-class="opacity-0 max-h-0"
      leave-to-class="opacity-0 max-h-0"
    >
      <div v-show="isExpanded" class="px-1.5 pb-1.5">
        <!-- 该分组直属的服务器 -->
        <div v-if="serversInGroup.length > 0" class="grid grid-cols-[repeat(auto-fill,minmax(180px,1fr))] gap-1.5 px-0.5">
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
        <div v-if="childGroups.length > 0" class="mt-1">
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
        <!-- 空状态：整棵子树都没有服务器 -->
        <div v-if="allServersInGroup.length === 0" class="text-center py-1.5 text-base-content/50 text-[11px]">
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

import type { ServerGroup } from '../../types'

const props = defineProps<{
  group: ServerGroup;
  groups: ServerGroup[];
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

// 子孙分组 id（含防御环引用）：分组计数必须递归统计，否则服务器都挂在
// 子分组下时，顶层分组会显示 0
const descendantGroupIds = computed(() => {
  const ids = new Set<string | null>();
  const queue = childGroups.value.map(g => g);
  while (queue.length > 0) {
    const g = queue.shift()!;
    const key = g.id as string | null;
    if (ids.has(key)) {continue}
    ids.add(key);
    props.groups.filter(c => c.parentId === g.id).forEach(c => queue.push(c));
  }
  return ids;
});

const allServersInGroup = computed(() => {
  const ids = descendantGroupIds.value;
  return props.servers.filter(s => s.groupId === props.group.id || ids.has(s.groupId));
});

const onlineCount = computed(() => {
  return allServersInGroup.value.filter(s => props.connectionStatusMap[s.id] === 'online').length;
});

const isExpanded = computed(() => {
  return props.expandedGroups.has(props.group.id as string | null);
});

function toggle() {
  emit('toggle', props.group.id);
}
</script>
