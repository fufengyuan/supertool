<template>
  <div class="drawer-group" :class="{ 'drawer-expanded': isExpanded, [`drawer-depth-${depth}`]: true }">
    <div class="drawer-handle" @click="toggle" :style="{ '--group-color': group.color || '#6c63ff' }">
      <div class="drawer-handle-left">
        <svg class="drawer-chevron" :class="{ expanded: isExpanded }" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="6 9 12 15 18 9"/>
        </svg>
        <span class="drawer-icon">{{ getGroupIcon(depth) }}</span>
        <span class="drawer-name">{{ group.name }}</span>
        <span class="drawer-count" :style="{ background: (group.color || '#6c63ff') + '22', color: group.color || '#6c63ff' }">
          {{ serversInGroup.length }}
        </span>
      </div>
      <div class="drawer-handle-right">
        <span class="drawer-online" v-if="onlineCount > 0">
          <span class="online-dot"></span>
          {{ onlineCount }} 在线
        </span>
      </div>
    </div>

    <Transition name="drawer-expand">
      <div v-show="isExpanded" class="drawer-body">
        <!-- 该分组下的服务器 -->
        <div v-if="serversInGroup.length > 0" class="drawer-servers">
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
        <div v-if="childGroups.length > 0" class="drawer-subgroups">
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
        <div v-if="serversInGroup.length === 0 && childGroups.length === 0" class="drawer-empty">
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
  const icons = ['📂', '📁', '🗂️', '📦', '🏷️'];
  return icons[depth % icons.length];
}

function toggle() {
  emit('toggle', props.group.id);
}
</script>

<style scoped>
/* ── 抽屉容器 ── */
.drawer-group {
  margin-bottom: 4px;
  border-radius: 10px;
}

.drawer-group.drawer-expanded {
  margin-bottom: 8px;
}

/* ── 抽屉把手（标题栏）── */
.drawer-handle {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 7px 12px;
  border-radius: 8px;
  cursor: pointer;
  user-select: none;
  transition: all 0.2s ease;
  background: var(--color-base-100);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  position: relative;
  overflow: hidden;
}

.drawer-handle::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
  border-radius: 0 3px 3px 0;
  background: var(--group-color, #6c63ff);
}

.drawer-handle:hover {
  border-color: var(--group-color, var(--color-primary));
  box-shadow: 0 2px 12px rgba(108, 99, 255, 0.1);
  transform: translateY(-1px);
}

.drawer-handle-left {
  display: flex;
  align-items: center;
  gap: 8px;
  position: relative;
  z-index: 1;
}

.drawer-chevron {
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  flex-shrink: 0;
}

.drawer-chevron.expanded {
  transform: rotate(180deg);
  color: var(--group-color, var(--color-primary));
}

.drawer-icon {
  font-size: 14px;
  line-height: 1;
}

.drawer-name {
  font-weight: 600;
  font-size: 13px;
  color: var(--color-base-content);
}

.drawer-count {
  font-size: 11px;
  font-weight: 600;
  padding: 1px 7px;
  border-radius: 12px;
  line-height: 1.4;
}

.drawer-handle-right {
  display: flex;
  align-items: center;
  gap: 8px;
  position: relative;
  z-index: 1;
}

.drawer-online {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--color-success);
  font-weight: 500;
}

.online-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--color-success);
  box-shadow: 0 0 4px var(--color-success);
}

/* ── 抽屉面板（内容区）── */
.drawer-body {
  margin-top: 4px;
  padding: 8px 10px;
  border-radius: 8px;
  background: color-mix(in srgb, var(--color-base-100) 80%, color-mix(in oklab, var(--color-base-content) 10%, transparent) 20%);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-top: none;
}

/* 卡片置于抽屉面板之上 */
.drawer-servers {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 8px;
}

/* 子分组缩进 */
.drawer-subgroups {
  margin-top: 6px;
  padding-left: 4px;
}

.drawer-empty {
  text-align: center;
  padding: 12px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-size: 12px;
  background: var(--color-base-200);
  border-radius: 6px;
  border: 1px dashed color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

/* 多级缩进 */
.drawer-depth-0 .drawer-body { padding-left: 10px; }
.drawer-depth-1 .drawer-body { padding-left: 16px; }
.drawer-depth-2 .drawer-body { padding-left: 22px; }
.drawer-depth-3 .drawer-body { padding-left: 28px; }

/* ── 展开/折叠动画 ── */
.drawer-expand-enter-active,
.drawer-expand-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  overflow: hidden;
}

.drawer-expand-enter-from,
.drawer-expand-leave-to {
  opacity: 0;
  max-height: 0;
  transform: translateY(-6px);
}

.drawer-expand-enter-to,
.drawer-expand-leave-from {
  opacity: 1;
  max-height: 5000px;
  transform: translateY(0);
}
</style>
