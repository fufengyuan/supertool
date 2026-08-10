<template>
  <!-- IDEA 风格：分组标题栏 + 服务器平铺 -->
  <div class="mb-2">
    <!-- 分组标题栏 - 简洁扁平，带分组背景色 -->
    <div class="flex items-center gap-2 px-3 py-1.5 rounded cursor-pointer select-none transition-colors group/header"
      :style="isExpanded ? { backgroundColor: groupBgColor } : {}"
      :class="isExpanded ? '' : 'hover:bg-base-100/50'"
      @click="toggle">
      <!-- 展开/折叠箭头 -->
      <SvgIcon class="text-base-content/50 transition-transform flex-shrink-0" 
        :class="{ 'rotate-180': isExpanded }" 
        name="chevronDown" size="12" strokeWidth="2.5" />
      <!-- 分组颜色标记 -->
      <span class="w-2 h-2 rounded-full flex-shrink-0" :style="{ backgroundColor: group.color || '#6c63ff' }"></span>
      <!-- 分组名称 -->
      <span class="font-medium text-[11px] text-base-content">{{ group.name }}</span>
      <!-- 服务器数量 -->
      <span class="text-[10px] px-1.5 py-0 rounded bg-base-200 text-base-content/60 leading-tight">
        {{ serversInGroup.length }}
      </span>
      <!-- 在线数量 -->
      <span class="flex items-center gap-1 text-[10px] text-success ml-auto" v-if="onlineCount > 0">
        <span class="w-1 h-1 rounded-full bg-success"></span>
        {{ onlineCount }}
      </span>
    </div>

    <!-- 服务器列表 - 直接平铺，无嵌套卡片 -->
    <Transition
      enter-active-class="transition-all duration-200 ease-out"
      leave-active-class="transition-all duration-200 ease-in"
      enter-from-class="opacity-0 max-h-0"
      leave-to-class="opacity-0 max-h-0"
    >
      <div v-show="isExpanded" class="mt-1 pl-5">
        <!-- 该分组下的服务器 -->
        <div v-if="serversInGroup.length > 0" class="grid grid-cols-[repeat(auto-fill,minmax(180px,1fr))] gap-1.5">
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
        <div v-if="childGroups.length > 0" class="mt-1.5">
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
        <!-- 空状态 -->
        <div v-if="serversInGroup.length === 0 && childGroups.length === 0" class="text-center py-2 text-base-content/50 text-[11px]">
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

const onlineCount = computed(() => {
  return serversInGroup.value.filter(s => props.connectionStatusMap[s.id] === 'online').length;
});

const isExpanded = computed(() => {
  return props.expandedGroups.has(props.group.id as string | null);
});

// 将 hex 颜色转为 rgba（10% 透明度）
const groupBgColor = computed(() => {
  const hex = props.group.color || '#6b7280';
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return `rgba(${r}, ${g}, ${b}, 0.1)`;
});

function toggle() {
  emit('toggle', props.group.id);
}
</script>
