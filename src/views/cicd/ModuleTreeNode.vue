<template>
  <div class="border-b border-base-content/10 last:border-b-0">
    <div
      class="flex items-center gap-2 px-3.5 py-2 cursor-pointer transition-colors duration-150 hover:bg-base-200"
      :style="depth > 0 ? { paddingLeft: `${14 + depth * 20}px` } : {}"
      @click="toggleTreeNode"
    >
      <!-- Expand arrow -->
      <svg v-if="node.children && node.children.length > 0" class="transition-transform duration-200 text-base-content/60 flex-shrink-0" :class="{ 'rotate-90': isExpanded }" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="6 9 12 15 18 9" />
      </svg>
      <span v-else class="w-[14px] flex-shrink-0"></span>

      <!-- Type badge -->
      <span class="text-xs flex-shrink-0">
        {{ node.type === 'maven' ? '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" class="inline-block align-text-bottom"><polygon points="12 2 22 8.5 22 15.5 12 22 2 15.5 2 8.5" fill="currentColor"/></svg>' : node.type === 'npm' ? '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="6" fill="currentColor"/></svg>' : '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><line x1="16.5" y1="9.4" x2="7.5" y2="4.21"/><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>' }}
      </span>

      <!-- Name & path -->
      <span class="font-semibold text-xs text-base-content min-w-[100px] flex-shrink-0">{{ node.name }}</span>
      <span v-if="node.path && node.path !== '.'" class="font-mono text-[11px] text-base-content/60 flex-1 overflow-hidden text-ellipsis whitespace-nowrap">{{ node.path }}</span>

      <!-- Add button -->
      <button
        class="btn btn-ghost btn-xs text-primary flex-shrink-0"
        :class="{ 'text-success border-success cursor-default': isAlreadyAdded }"
        @click.stop="$emit('add', node)"
      >
        {{ isAlreadyAdded ? '✓ 已添加' : '+ 添加' }}
      </button>
    </div>

    <!-- Children (recursive) -->
    <div v-if="node.children && node.children.length > 0 && isExpanded">
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
