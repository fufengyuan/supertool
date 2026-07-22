<template>
  <div class="border-b border-base-content/10 last:border-b-0">
    <div
      class="flex items-center gap-2 px-3.5 py-2 cursor-pointer transition-colors duration-150 hover:bg-base-200"
      :style="depth > 0 ? { paddingLeft: `${14 + depth * 20}px` } : {}"
      @click="toggleTreeNode"
    >
      <!-- Expand arrow -->
      <SvgIcon v-if="node.children && node.children.length > 0" name="chevronDown" size="14" class="transition-transform duration-200 text-base-content/60 flex-shrink-0" :class="{ 'rotate-90': isExpanded }" />
      <span v-else class="w-[14px] flex-shrink-0"></span>

      <!-- Type badge -->
      <span class="text-xs flex-shrink-0">
        <template v-if="node.type === 'maven'"><SvgIcon name="layers" size="14" class="inline-block align-text-bottom" /></template><template v-else-if="node.type === 'npm'"><SvgIcon name="dot" size="14" class="inline-block align-text-bottom" /></template><template v-else><SvgIcon name="package" size="14" class="inline-block align-text-bottom" /></template>
      </span>

      <!-- Name & path -->
      <span class="font-semibold text-xs text-base-content min-w-[100px] flex-shrink-0" :title="node.name">{{ node.name }}</span>
      <span v-if="node.path && node.path !== '.'" class="font-mono text-[11px] text-base-content/60 flex-1 overflow-hidden text-ellipsis whitespace-nowrap" :title="node.path">{{ node.path }}</span>

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
import SvgIcon from '@/components/ui/SvgIcon.vue'

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
