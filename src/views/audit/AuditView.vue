<template>
  <div class="h-full flex flex-col p-4 bg-base-200 text-base-content">
    <div class="flex-1 min-h-0 bg-base-100 rounded-box overflow-hidden flex flex-col">
      <!-- 头部：标题 + 筛选 -->
      <div class="flex items-center gap-3 px-4 py-3 border-b border-base-content/10">
        <h3 class="text-sm font-semibold flex items-center gap-2">
          <SvgIcon name="fileText" size="16" class="text-primary" /> 操作审计
        </h3>
        <div class="flex items-center gap-2 ml-4">
          <select v-model="actorFilter" class="select select-sm select-bordered h-8 min-h-0 text-xs" @change="load">
            <option value="">全部发起方</option>
            <option value="cli">CLI</option>
            <option value="gui">GUI</option>
            <option value="ai">AI</option>
            <option value="user">用户</option>
          </select>
          <select v-model="resultFilter" class="select select-sm select-bordered h-8 min-h-0 text-xs" @change="load">
            <option value="">全部结果</option>
            <option value="success">成功</option>
            <option value="failed">失败</option>
            <option value="blocked">被拦截</option>
          </select>
          <button class="btn btn-ghost btn-sm" @click="load" title="刷新"><SvgIcon name="refresh" size="14" /></button>
        </div>
        <span class="ml-auto text-xs text-base-content/50">{{ entries.length }} 条记录</span>
      </div>

      <!-- 列表 -->
      <div class="flex-1 overflow-y-auto" ref="listRef">
        <div v-if="loading" class="flex items-center justify-center h-full text-base-content/40 text-sm">加载中...</div>
        <div v-else-if="entries.length === 0" class="flex items-center justify-center h-full text-base-content/40 text-sm">
          暂无审计记录（CLI 写操作会自动记录）
        </div>
        <div v-else class="divide-y divide-base-content/5">
          <div
            v-for="e in entries"
            :key="e.id"
            class="px-4 py-2.5 hover:bg-base-200/50 transition-colors"
          >
            <div class="flex items-center gap-2 text-xs">
              <span class="shrink-0 w-4 text-center">{{ statusIcon(e.result) }}</span>
              <span class="shrink-0 font-mono text-base-content/50">{{ e.createdAt }}</span>
              <span class="shrink-0 badge badge-xs" :class="actorBadgeClass(e.actorType)">{{ e.actorType }}</span>
              <span class="shrink-0 text-base-content/40 font-mono">{{ e.durationMs }}ms</span>
            </div>
            <div class="mt-1 ml-6 font-mono text-[11px] text-base-content/80 break-all leading-relaxed" :title="e.command">
              {{ e.command }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { getTauriAPI } from '../../utils/tauri-api'

interface AuditEntry {
  id: number
  actorType: string
  actorName: string
  command: string
  argsJson: string
  target: string
  result: string
  durationMs: number
  createdAt: string
}

const entries = ref<AuditEntry[]>([])
const actorFilter = ref('')
const resultFilter = ref('')
const loading = ref(false)

function statusIcon(result: string): string {
  return result === 'success' ? '✅' : result === 'failed' ? '❌' : result === 'blocked' ? '🚫' : '•'
}

function actorBadgeClass(actor: string): string {
  switch (actor) {
    case 'ai': return 'badge-info'
    case 'cli': return 'badge-primary'
    case 'gui': return 'badge-success'
    default: return 'badge-ghost'
  }
}

async function load() {
  loading.value = true
  try {
    const res = await getTauriAPI().auditList(
      actorFilter.value || undefined,
      resultFilter.value || undefined,
      200,
    )
    entries.value = res?.success ? (res.data ?? []) : []
  } catch (e) {
    entries.value = []
    console.error('audit load failed:', e)
  } finally {
    loading.value = false
  }
}

onMounted(load)
</script>
