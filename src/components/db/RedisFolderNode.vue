<template>
  <div class="redis-folder-group">
    <!-- Folder node with children -->
    <template v-if="!node.isLeaf">
      <div
        class="tree-item tree-folder"
        :class="{ expanded: isExpanded }"
        @click.stop="onToggleFolder"
        @contextmenu.prevent="onFolderCtx"
      >
        <span class="tree-toggle">{{ isExpanded ? '▼' : '▶' }}</span>
        <span class="tree-icon">📁</span>
        <span class="tree-label">{{ node.segment }}</span>
        <span class="tree-count">{{ node.totalCount }}</span>
      </div>

      <Transition name="accordion">
        <div v-show="isExpanded" class="tree-children">
          <RedisFolderNode
            v-for="child in node.children.values()"
            :key="child.segment"
            :node="child"
            :conn="conn"
            :db-index="dbIndex"
            :parent-path="folderPath"
            @open-key="$emit('open-key', $event)"
            @folder-context="onFolderCtxChild"
            @key-context="onKeyCtxChild"
            @toggle-folder="(path, expanded) => $emit('toggle-folder', path, expanded)"
          />
        </div>
      </Transition>
    </template>

    <!-- Leaf key node -->
    <div
      v-else
      class="tree-item tree-redis-key"
      @click.stop="$emit('open-key', node.key!)"
      @contextmenu.prevent="$emit('key-context', $event, conn, dbIndex, node.key!, node.type!)"
    >
      <span class="tree-key-type-dot" :class="'type-' + (node.type || 'default')"></span>
      <span class="tree-icon">{{ typeIcon(node.type || '') }}</span>
      <span class="tree-label">{{ node.segment }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { DBConnection } from '../../composables/useDBManager'

interface RedisTreeNode {
  segment: string
  children: Map<string, RedisTreeNode>
  isLeaf: boolean
  key: string | null
  type: string | null
  totalCount: number
}

const props = defineProps<{
  node: RedisTreeNode
  conn: DBConnection
  dbIndex: number
  parentPath: string
}>()

const emit = defineEmits<{
  'open-key': [key: string]
  'folder-context': [event: MouseEvent, conn: DBConnection, dbIndex: number, folderPath: string]
  'key-context': [event: MouseEvent, conn: DBConnection, dbIndex: number, key: string, type: string]
  'toggle-folder': [folderPath: string, isExpanded: boolean]
}>()

// Build full folder path for this node
const folderPath = computed(() => {
  return props.parentPath ? `${props.parentPath}:${props.node.segment}` : props.node.segment
})

// Local expansion state for this folder
const isExpanded = ref(false)

function onToggleFolder() {
  isExpanded.value = !isExpanded.value
  emit('toggle-folder', folderPath.value, isExpanded.value)
}

function onFolderCtx(event: MouseEvent) {
  emit('folder-context', event, props.conn, props.dbIndex, folderPath.value)
}

function onFolderCtxChild(event: MouseEvent, conn: DBConnection, dbIndex: number, folderPath: string) {
  emit('folder-context', event, conn, dbIndex, folderPath)
}

function onKeyCtxChild(event: MouseEvent, conn: DBConnection, dbIndex: number, key: string, type: string) {
  emit('key-context', event, conn, dbIndex, key, type)
}

function typeIcon(type: string): string {
  const icons: Record<string, string> = {
    string: '📝',
    hash: '🗂️',
    list: '📃',
    set: '🔵',
    zset: '📊'
  }
  return icons[type] || '🔑'
}
</script>

<style scoped>
/* ============================================================
   Redis Folder Node — IDE Navicat-style
   ============================================================ */

.redis-folder-group {
  display: flex;
  flex-direction: column;
}

/* ── Folder node ─────────────────────────────────────────── */
.tree-item.tree-folder {
  position: relative;
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 3px 6px;
  margin: 0 2px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  line-height: 1.5;
  color: oklch(var(--bc));
  font-weight: 500;
  transition: background 0.12s ease, color 0.12s ease;
  user-select: none;
  white-space: nowrap;
  min-height: 22px;
}

.tree-item.tree-folder:hover {
  background: oklch(var(--p) / 0.1));
}

.tree-item.tree-folder.expanded {
  font-weight: 500;
}

/* Folder icon — slightly different tint when expanded */
.tree-item.tree-folder .tree-icon {
  font-size: 13px;
  width: 16px;
  height: 16px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  line-height: 1;
}

/* Folder count badge */
.tree-item.tree-folder .tree-count {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 10px;
  background: oklch(var(--b2));
  color: oklch(var(--bc) / 0.6);
  flex-shrink: 0;
  line-height: 1.4;
  transition: background 0.12s ease, color 0.12s ease;
  max-width: 50px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: center;
}

.tree-item.tree-folder:hover .tree-count {
  background: oklch(var(--bc) / 0.1);
}

/* Folder label */
.tree-item.tree-folder .tree-label {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Folder chevron */
.tree-item.tree-folder .tree-toggle {
  width: 14px;
  height: 14px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 8px;
  flex-shrink: 0;
  color: oklch(var(--bc) / 0.6);
  transition: transform 0.15s ease, color 0.12s ease;
}

.tree-item.tree-folder .tree-toggle:hover {
  color: oklch(var(--bc));
}

/* ── Leaf key node ───────────────────────────────────────── */
.tree-item.tree-redis-key {
  position: relative;
  display: flex;
  align-items: center;
  gap: 3px;
  padding: 3px 6px;
  margin: 0 2px;
  border-radius: 4px;
  cursor: pointer;
  font-family: 'SF Mono', 'Fira Code', 'Cascadia Code', 'JetBrains Mono', Consolas, monospace;
  font-size: 11.5px;
  line-height: 1.5;
  color: oklch(var(--bc));
  transition: background 0.12s ease, color 0.12s ease;
  user-select: none;
  white-space: nowrap;
  min-height: 22px;
}

.tree-item.tree-redis-key:hover {
  background: oklch(var(--p) / 0.1));
}

/* Key type dot */
.tree-key-type-dot {
  display: inline-block;
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
  transition: transform 0.1s ease;
}

.tree-item.tree-redis-key:hover .tree-key-type-dot {
  transform: scale(1.2);
}

.tree-key-type-dot.type-string  { background: #52c41a; }
.tree-key-type-dot.type-hash    { background: #1677ff; }
.tree-key-type-dot.type-list    { background: #fa8c16; }
.tree-key-type-dot.type-set     { background: #722ed1; }
.tree-key-type-dot.type-zset    { background: #eb2f96; }
.tree-key-type-dot.type-default { background: #8c8c8c; }

/* Key icon */
.tree-item.tree-redis-key .tree-icon {
  width: 16px;
  height: 16px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  font-size: 12px;
  line-height: 1;
}

/* Key label */
.tree-item.tree-redis-key .tree-label {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* ── Children container with guide line ─────────────────── */
.tree-children {
  position: relative;
  padding-left: 16px;
}

.tree-children::before {
  content: '';
  position: absolute;
  left: 8px;
  top: 0;
  bottom: 0;
  width: 1px;
  background: oklch(var(--bc) / 0.1);
  opacity: 0.6;
  pointer-events: none;
}

/* ── Accordion animation ────────────────────────────────── */
.accordion-enter-active {
  transition: opacity 0.18s ease, max-height 0.25s ease;
}

.accordion-leave-active {
  transition: opacity 0.12s ease, max-height 0.2s ease;
}

.accordion-enter-from,
.accordion-leave-to {
  opacity: 0;
}
</style>
