<template>
  <div class="flex flex-col">
    <!-- Folder node with children -->
    <template v-if="!node.isLeaf">
      <div
        class="relative flex items-center gap-1 px-1.5 py-[3px] mx-0.5 rounded cursor-pointer text-xs leading-5 text-base-content font-medium select-none whitespace-nowrap min-h-[22px] transition-[background,color] duration-100 ease-in-out hover:bg-primary/10"
        :class="{ 'font-medium': isExpanded }"
        @click.stop="onToggleFolder"
        @contextmenu.prevent="onFolderCtx"
      >
        <span class="w-[14px] h-[14px] inline-flex items-center justify-center text-[8px] shrink-0 text-base-content/60 transition-[transform,color] duration-150 ease-in-out hover:text-base-content">{{ isExpanded ? '▼' : '▶' }}</span>
        <span class="text-[13px] w-4 h-4 inline-flex items-center justify-center shrink-0 leading-none"><SvgIcon name="folder" size="14" class="inline-block align-text-bottom" /></span>
        <span class="flex-1 min-w-0 overflow-hidden text-ellipsis whitespace-nowrap">{{ node.segment }}</span>
        <span class="text-[10px] px-1.5 py-[1px] rounded-full bg-base-200 text-base-content/60 shrink-0 leading-[1.4] transition-[background,color] duration-100 ease-in-out max-w-[50px] overflow-hidden text-ellipsis whitespace-nowrap text-center hover:bg-base-content/10">{{ node.totalCount }}</span>
      </div>

      <Transition
        enter-active-class="transition-all duration-[180ms] ease overflow-hidden"
        leave-active-class="transition-all duration-[120ms] ease overflow-hidden"
        enter-from-class="opacity-0 max-h-0"
        enter-to-class="opacity-100 max-h-[1000px]"
        leave-from-class="opacity-100 max-h-[1000px]"
        leave-to-class="opacity-0 max-h-0"
      >
        <div v-show="isExpanded" class="relative ps-4 before:content-[''] before:absolute before:left-2 before:top-0 before:bottom-0 before:w-px before:bg-base-content/10 before:opacity-60 before:pointer-events-none">
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
      class="relative flex items-center gap-[3px] px-1.5 py-[3px] mx-0.5 rounded cursor-pointer font-mono text-[11.5px] leading-5 text-base-content select-none whitespace-nowrap min-h-[22px] transition-[background,color] duration-100 ease-in-out hover:bg-primary/10"
      @click.stop="$emit('open-key', node.key!, node.keyB64)"
      @contextmenu.prevent="$emit('key-context', $event, conn, dbIndex, node.key!, node.type!, node.keyB64)"
    >
      <span
        class="inline-block w-[7px] h-[7px] rounded-full shrink-0 transition-transform duration-100 ease-in-out group-hover:scale-110"
        :class="{
          'bg-[#52c41a]': node.type === 'string',
          'bg-[#1677ff]': node.type === 'hash',
          'bg-[#fa8c16]': node.type === 'list',
          'bg-[#722ed1]': node.type === 'set',
          'bg-[#eb2f96]': node.type === 'zset',
          'bg-[#8c8c8c]': !node.type || node.type === 'default'
        }"
      ></span>
      <span class="w-4 h-4 inline-flex items-center justify-center shrink-0 text-xs leading-none">{{ typeIcon(node.type || '') }}</span>
      <span class="flex-1 min-w-0 overflow-hidden text-ellipsis whitespace-nowrap">{{ node.segment }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref, computed } from 'vue'
import type { DBConnection } from '../../composables/useDBManager'

interface RedisTreeNode {
  segment: string
  children: Map<string, RedisTreeNode>
  isLeaf: boolean
  key: string | null
  keyB64?: string | null
  pathB64?: string | null
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
  'open-key': [key: string, keyB64?: string | null]
  'folder-context': [event: MouseEvent, conn: DBConnection, dbIndex: number, folderPath: string]
  'key-context': [event: MouseEvent, conn: DBConnection, dbIndex: number, key: string, type: string, keyB64?: string | null]
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

function onKeyCtxChild(event: MouseEvent, conn: DBConnection, dbIndex: number, key: string, type: string, keyB64?: string | null) {
  emit('key-context', event, conn, dbIndex, key, type, keyB64)
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
