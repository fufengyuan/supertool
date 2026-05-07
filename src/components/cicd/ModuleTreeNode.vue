<template>
  <div class="tree-item" :class="{ [`tree-depth-${depth}`]: depth > 0 }">
    <div class="tree-item-header" @click="toggleTreeNode">
      <!-- Expand arrow -->
      <svg v-if="node.children && node.children.length > 0" class="tree-expand" :class="{ expanded: isExpanded }" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="6 9 12 15 18 9" />
      </svg>
      <span v-else class="tree-indent"></span>

      <!-- Type badge -->
      <span class="tree-type-badge" :class="node.type">
        {{ node.type === 'maven' ? '🔶' : node.type === 'npm' ? '🔴' : '📦' }}
      </span>

      <!-- Name & path -->
      <span class="tree-name">{{ node.name }}</span>
      <span class="tree-path" v-if="node.path && node.path !== '.'">{{ node.path }}</span>

      <!-- Add button -->
      <button class="tree-add-btn" @click.stop="$emit('add', node)" :class="{ added: isAlreadyAdded }">
        {{ isAlreadyAdded ? '✓ 已添加' : '+ 添加' }}
      </button>
    </div>

    <!-- Children (recursive) -->
    <div v-if="node.children && node.children.length > 0 && isExpanded" class="tree-children">
      <ModuleTreeNode
        v-for="child in node.children"
        :key="child.path"
        :node="child"
        :depth="depth + 1"
        :expanded-nodes="expandedNodes"
        :added-paths="addedPaths"
        @toggle="$emit('toggle', $event)"
        @add="$emit('add', $event)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  node: any
  depth: number
  expandedNodes: string[]
  addedPaths: Set<string>
}>()

const emit = defineEmits<{
  toggle: [path: string]
  add: [node: any]
}>()

const isExpanded = computed(() => props.expandedNodes.includes(props.node.path))
const isAlreadyAdded = computed(() => props.addedPaths.has(props.node.path))

function toggleTreeNode() {
  if (props.node.children && props.node.children.length > 0) {
    emit('toggle', props.node.path)
  }
}
</script>

<style scoped>
.tree-item {
  border-bottom: 1px solid var(--border-color);
}

.tree-item:last-child {
  border-bottom: none;
}

.tree-item-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  cursor: pointer;
  transition: background 0.15s;
}

.tree-item-header:hover {
  background: var(--input-bg);
}

.tree-expand {
  transition: transform 0.2s ease;
  color: var(--main-text-secondary);
  flex-shrink: 0;
}

.tree-expand.expanded {
  transform: rotate(90deg);
}

.tree-indent {
  width: 14px;
  flex-shrink: 0;
}

.tree-type-badge {
  font-size: 12px;
  flex-shrink: 0;
}

.tree-name {
  font-weight: 600;
  font-size: 13px;
  color: var(--main-text);
  min-width: 100px;
  flex-shrink: 0;
}

.tree-path {
  font-family: 'SF Mono', 'Fira Code', monospace;
  font-size: 11px;
  color: var(--main-text-secondary);
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tree-add-btn {
  padding: 4px 10px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: transparent;
  color: var(--primary-color);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
  flex-shrink: 0;
}

.tree-add-btn:hover {
  background: var(--primary-color);
  color: white;
  border-color: var(--primary-color);
}

.tree-add-btn.added {
  color: #10b981;
  border-color: #10b981;
  cursor: default;
}

.tree-children {
  /* no background — keep flat look */
}

/* Depth-based indentation */
.tree-depth-1 > .tree-item-header { padding-left: 28px; }
.tree-depth-2 > .tree-item-header { padding-left: 48px; }
.tree-depth-3 > .tree-item-header { padding-left: 68px; }
.tree-depth-4 > .tree-item-header { padding-left: 88px; }
</style>
