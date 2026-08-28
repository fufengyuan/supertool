<template>
  <div class="max-w-4xl mx-auto">
    <div class="mb-6">
      <h1 class="text-2xl font-bold">更多功能</h1>
      <p class="text-sm text-base-content/60 mt-1">低频使用的工具与报表，从菜单收进来集中跳转</p>
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
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'MoreToolsHub' })
import { useRouter } from 'vue-router'

const router = useRouter()

const items = [
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
</script>
