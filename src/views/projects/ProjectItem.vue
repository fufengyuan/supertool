<template>
  <div
    class="flex items-stretch gap-4 p-4 px-5 border border-base-content/10 border-l-4 rounded-xl bg-base-100 cursor-pointer transition-all duration-150 shadow-sm hover:border-primary hover:shadow-md hover:-translate-y-0.5"
    :style="{ borderLeftColor: project.color }"
    @click="$emit('select', project)"
  >
    <!-- 左侧：颜色+核心信息 -->
    <div class="flex gap-3.5 flex-1 min-w-0">
      <div class="w-3 h-3 rounded-full flex-shrink-0 mt-1.5" :style="{ backgroundColor: project.color }"></div>
      <div class="flex-1 min-w-0">
        <!-- 标题行 -->
        <div class="flex items-center gap-2.5 flex-wrap mb-1.5">
          <h3 class="m-0 text-base-content text-lg font-bold">{{ project.name }}</h3>
          <span v-if="project.category" class="badge badge-sm badge-primary">{{ categoryLabel(project.category) }}</span>
          <span v-if="project.archived" class="badge badge-sm badge-warning">已归档</span>
        </div>
        <!-- 描述 -->
        <p v-if="project.description" class="m-0 mb-2 text-base-content/60 text-sm leading-relaxed line-clamp-2">{{ project.description }}</p>
        <!-- 元信息行 -->
        <div class="flex gap-4 mb-2 flex-wrap">
          <span class="text-xs text-base-content/60 flex items-center gap-1" v-if="project.createdAt">
            <span class="text-xs"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="4" width="18" height="18" rx="2" ry="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/></svg>  创建于 {{ formatDate(project.createdAt) }}</span>
          </span>
          <span class="text-xs text-base-content/60 flex items-center gap-1" v-if="project.updatedAt">
            <span class="text-xs"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>  更新于 {{ formatDate(project.updatedAt) }}</span>
          </span>
        </div>
        <!-- Git 仓库 -->
        <div class="flex flex-col gap-1" v-if="hasGitRepos">
          <div v-if="project.repoPath" class="flex items-center gap-1.5 text-xs text-base-content/60">
            <span class="flex-shrink-0 text-sm"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg> </span>
            <span class="flex-1 whitespace-nowrap overflow-hidden text-ellipsis font-mono text-[11px]">{{ project.repoPath.split('/').pop() }}</span>
            <span v-if="project.branch" class="badge badge-sm badge-primary">{{ project.branch }}</span>
          </div>
          <div v-if="project.repoPath2" class="flex items-center gap-1.5 text-xs text-base-content/60">
            <span class="flex-shrink-0 text-sm"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg> </span>
            <span class="flex-1 whitespace-nowrap overflow-hidden text-ellipsis font-mono text-[11px]">{{ project.repoPath2.split('/').pop() }}</span>
            <span v-if="project.branch2" class="badge badge-sm badge-primary">{{ project.branch2 }}</span>
          </div>
          <div v-if="project.gitUrl1" class="flex items-center gap-1.5 text-xs text-base-content/60">
            <span class="flex-shrink-0 text-sm"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg></span>
            <span class="flex-1 whitespace-nowrap overflow-hidden text-ellipsis font-mono text-[11px]">{{ project.gitUrl1 }}</span>
          </div>
          <div v-if="project.gitUrl2" class="flex items-center gap-1.5 text-xs text-base-content/60">
            <span class="flex-shrink-0 text-sm"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg></span>
            <span class="flex-1 whitespace-nowrap overflow-hidden text-ellipsis font-mono text-[11px]">{{ project.gitUrl2 }}</span>
          </div>
        </div>
      </div>
    </div>
    <!-- 右侧：统计+操作 -->
    <div class="flex flex-col items-end justify-between flex-shrink-0 gap-3 min-w-[200px]">
      <div class="flex flex-col gap-2 w-full">
        <div class="flex gap-4">
          <div class="flex flex-col items-center">
            <span class="text-lg font-bold text-base-content">{{ stats?.total || 0 }}</span>
            <span class="text-[11px] text-base-content/60">总任务</span>
          </div>
          <div class="flex flex-col items-center">
            <span class="text-lg font-bold text-emerald-500">{{ stats?.completed || 0 }}</span>
            <span class="text-[11px] text-base-content/60">已完成</span>
          </div>
          <div class="flex flex-col items-center">
            <span class="text-lg font-bold text-amber-400">{{ (stats?.total || 0) - (stats?.completed || 0) }}</span>
            <span class="text-[11px] text-base-content/60">进行中</span>
          </div>
        </div>
        <div class="flex items-center gap-2">
          <div class="flex-1 h-2 bg-base-200 rounded-full overflow-hidden min-w-[100px]">
            <div class="h-full rounded-full transition-[width] duration-300" :style="{ width: (stats?.progress || 0) + '%', backgroundColor: project.color }"></div>
          </div>
          <span class="text-sm font-semibold text-base-content whitespace-nowrap min-w-[36px] text-right">{{ stats?.progress || 0 }}%</span>
        </div>
      </div>
      <div class="flex gap-1.5">
        <button class="btn btn-ghost btn-sm" @click.stop="$emit('toggle-archive', project)" :title="project.archived ? '取消归档' : '归档'">
          {{ project.archived ? '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><polyline points="1 4 1 10 7 10"/><path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10"/></svg>' : '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg> ' }}
        </button>
        <button class="btn btn-primary btn-sm" @click.stop="$emit('edit', project)" title="编辑"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg> </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps({
  project: { type: Object, required: true },
  stats: { type: Object, default: () => ({ total: 0, completed: 0, progress: 0 }) },
});

defineEmits(['select', 'edit', 'toggle-archive']);

const categoryMap: Record<string, string> = {
  'frontend': '前端',
  'backend': '后端',
  'infrastructure': '基础设施',
  'other': '其他',
};

const categoryLabel = (cat: string) => categoryMap[cat] || cat;

const hasGitRepos = computed(() =>
  props.project.repoPath || props.project.repoPath2 || props.project.gitUrl1 || props.project.gitUrl2
);

const formatDate = (dateStr: string) => {
  if (!dateStr) return '';
  return new Date(dateStr).toLocaleDateString('zh-CN', { month: '2-digit', day: '2-digit' });
};
</script>
