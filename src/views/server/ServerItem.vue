<template>
  <div class="relative bg-base-100 rounded-xl border border-base-content/10 transition-all duration-300 overflow-hidden hover:shadow-md hover:-translate-y-px hover:border-primary"
    :class="{
      'border-success/40': connectionStatus === 'online',
      'border-warning/40 animate-pulse': connectionStatus === 'connecting',
      'border-error/40': connectionStatus === 'heartbeat_failed',
    }">
    <div class="absolute top-0 left-0 right-0 h-0.5" :style="{ background: statusGradient }"></div>
    <div class="p-2 px-2.5">
      <div class="mb-1.5">
        <div class="flex justify-between items-center">
          <div class="flex items-center gap-1 min-w-0">
            <span class="font-semibold text-xs text-base-content truncate">{{ server.name }}</span>
            <span v-if="server.requiresApproval" class="text-[10px] opacity-80 flex-shrink-0" title="执行审核已开启">🔒</span>
          </div>
          <div class="flex items-center gap-1 text-[9px] px-1.5 py-0.5 rounded-full font-medium whitespace-nowrap flex-shrink-0"
            :class="{
              'bg-success/15 text-success': connectionStatus === 'online',
              'bg-warning/15 text-warning': connectionStatus === 'connecting',
              'bg-base-200 text-base-content/60': connectionStatus === 'offline',
              'bg-error/15 text-error': connectionStatus === 'heartbeat_failed',
            }">
            <span class="w-1 h-1 rounded-full flex-shrink-0"
              :class="{
                'bg-success shadow-[0_0_6px_var(--color-success)]': connectionStatus === 'online',
                'bg-warning animate-pulse': connectionStatus === 'connecting',
                'bg-base-content/60': connectionStatus === 'offline',
                'bg-error shadow-[0_0_6px_var(--color-error)] animate-pulse': connectionStatus === 'heartbeat_failed',
              }"></span>
            {{ statusLabel }}
          </div>
        </div>
      </div>

      <div class="flex flex-col gap-0.5 mb-1.5">
        <div class="flex items-center gap-1 text-[11px] text-base-content/60">
          <svg class="opacity-50 flex-shrink-0 w-3 h-3" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="2" y="3" width="20" height="14" rx="2" ry="2"/>
            <line x1="8" y1="21" x2="16" y2="21"/>
            <line x1="12" y1="17" x2="12" y2="21"/>
          </svg>
          <span class="truncate">{{ server.host }}<span class="text-base-content/40">:{{ server.port }}</span></span>
        </div>
        <div class="flex items-center gap-1 text-[11px] text-base-content/60">
          <svg class="opacity-50 flex-shrink-0 w-3 h-3" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
            <circle cx="12" cy="7" r="4"/>
          </svg>
          <span class="truncate">{{ server.username }}</span>
        </div>
      </div>

      <div v-if="server.tags && server.tags.length > 0" class="flex gap-0.5 mb-1.5 flex-wrap">
        <span v-for="tag in server.tags" :key="tag" class="px-1 py-px bg-base-200 rounded text-[9px] text-base-content/60 border border-base-content/10">{{ tag }}</span>
      </div>

      <div class="flex items-center gap-px">
        <button @click="$emit('terminal', server)" class="flex items-center justify-center w-7 h-7 border-none rounded-lg cursor-pointer bg-transparent text-base-content/60 transition-all hover:bg-base-200 hover:text-base-content hover:bg-primary/15 hover:text-primary" title="终端">
          <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/>
          </svg>
        </button>
        <button @click="$emit('sftp', server)" class="flex items-center justify-center w-7 h-7 border-none rounded-lg cursor-pointer bg-transparent text-base-content/60 transition-all hover:bg-base-200 hover:text-base-content hover:bg-primary/15 hover:text-primary" title="SFTP">
          <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
        </button>
        <div class="w-px h-3.5 bg-base-content/10 mx-0.5"></div>
        <button @click="$emit('edit', server)" class="flex items-center justify-center w-7 h-7 border-none rounded-lg cursor-pointer bg-transparent text-base-content/60 transition-all hover:bg-base-200 hover:text-base-content" title="编辑">
          <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
            <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
          </svg>
        </button>
        <button @click="$emit('delete', server.id)" class="flex items-center justify-center w-7 h-7 border-none rounded-lg cursor-pointer bg-transparent text-base-content/60 transition-all hover:bg-error/15 hover:text-error" title="删除">
          <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3 6 5 6 21 6"/>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps({
  server: { type: Object, required: true },
  connectionStatus: { type: String, default: 'offline' },
});

defineEmits(['connect', 'terminal', 'sftp', 'edit', 'delete']);

const statusLabel = computed(() => {
  switch (props.connectionStatus) {
    case 'online': return '已连接';
    case 'connecting': return '连接中...';
    case 'heartbeat_failed': return '已断开';
    case 'offline': default: return '未连接';
  }
});

const statusGradient = computed(() => {
  switch (props.connectionStatus) {
    case 'online': return 'linear-gradient(180deg, #a6e3a1, #40c057)';
    case 'connecting': return 'linear-gradient(180deg, #f9a825, #f59e0b)';
    case 'heartbeat_failed': return 'linear-gradient(180deg, #f38ba8, #e04560)';
    default: return 'linear-gradient(180deg, color-mix(in oklab, var(--color-base-content) 10%, transparent), transparent)';
  }
});
</script>
