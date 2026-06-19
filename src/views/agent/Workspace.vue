<template>
  <div class="flex h-full overflow-hidden bg-base-100">
    <!-- Left: Sessions Panel (collapsible) -->
    <div v-show="showSessions" class="w-72 shrink-0 h-full overflow-hidden">
      <SessionsPanel />
    </div>

    <!-- Resize handle -->
    <div
      v-show="showSessions"
      class="w-1 shrink-0 cursor-col-resize bg-transparent hover:bg-primary/30 transition-colors relative z-10"
      @mousedown="startResize"
    />

    <!-- Right: Chat -->
    <div class="flex-1 min-w-0 h-full overflow-hidden">
      <Chat />
    </div>

    <!-- Toggle sessions sidebar (floating) -->
    <button
      class="fixed top-20 left-0 z-20 px-1.5 py-2 bg-base-100 border border-base-content/10 rounded-r-md shadow-sm text-base-content/60 hover:text-base-content cursor-pointer transition-colors"
      :title="showSessions ? '隐藏会话列表' : '显示会话列表'"
      @click="showSessions = !showSessions"
    >
      <SvgIcon :name="showSessions ? 'chevronLeft' : 'chevronRight'" :size="14" />
    </button>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'AgentWorkspace' })
import { ref } from 'vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import SessionsPanel from './SessionsPanel.vue'
import Chat from './chat/Chat.vue'

const showSessions = ref(true)

// Simple resize via mouse drag
function startResize(e: MouseEvent) {
  const startX = e.clientX
  const startWidth = 288 // 72 * 4 (w-72)
  // We don't actually resize dynamically, just toggle
  // For a more polished experience, user can toggle on/off
}
</script>