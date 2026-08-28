<template>
  <div class="max-w-4xl mx-auto">
    <div class="mb-6">
      <h1 class="text-2xl font-bold">更多功能</h1>
      <p class="text-sm text-base-content/60 mt-1">低频使用的工具与开关，从菜单收进来集中跳转</p>
    </div>

    <div class="grid grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-4">
      <button
        v-for="item in items"
        :key="item.path"
        class="bg-base-100 border border-base-content/10 rounded-xl p-5 text-left hover:border-primary/50 hover:shadow-md transition-all cursor-pointer"
        @click="router.push(item.path)"
      >
        <div class="flex items-center gap-2.5 mb-2">
          <span class="text-2xl">{{ item.icon }}</span>
          <span class="font-semibold text-base-content">{{ item.label }}</span>
        </div>
        <p class="text-xs text-base-content/55 m-0 leading-relaxed">{{ item.description }}</p>
      </button>
    </div>

    <!-- 开关区：从侧栏底部挪过来的低频开关 -->
    <div class="mt-8">
      <h2 class="text-sm font-bold text-base-content mb-3">开关</h2>
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-5 flex items-center justify-between">
        <div>
          <div class="flex items-center gap-2 font-semibold text-sm text-base-content mb-1">
            <SvgIcon name="checkCircle" size="16" />
            <span>悬浮待办</span>
          </div>
          <p class="text-xs text-base-content/55 m-0">打开或关闭桌面悬浮待办小球；关闭后在这里随时重新打开。</p>
        </div>
        <button class="btn btn-primary btn-sm" @click="toggleFloatingTodo">打开 / 关闭悬浮球</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'MoreToolsHub' })
import { useRouter } from 'vue-router'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { getTauriAPI } from '@/utils/tauri-api'

const router = useRouter()

const items = [
  {
    path: '/accounting',
    icon: '💰',
    label: '记账本',
    description: '日常收支记录与统计',
  },
  {
    path: '/alert',
    icon: '🔔',
    label: '告警',
    description: '服务器资源与服务异常监控事件，集中查看与跟进',
  },
  {
    path: '/weekly',
    icon: '📊',
    label: '周报',
    description: '汇总本周任务与工作量，生成周报',
  },
  {
    path: '/report',
    icon: '📜',
    label: '任务报告',
    description: '按时间维度查看任务完成情况与统计',
  },
  {
    path: '/kanban',
    icon: '🗂️',
    label: '看板',
    description: '任务看板视图，拖拽管理任务状态',
  },
]

/** 打开/关闭待办悬浮球（关闭后同样的入口可重新打开） */
function toggleFloatingTodo() {
  getTauriAPI().toggleFloatingTodo().catch(() => {})
}
</script>
