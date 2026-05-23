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
            <span class="text-xs"><SvgIcon name="calendar" :size="14" />  创建于 {{ formatDate(project.createdAt) }}</span>
          </span>
          <span class="text-xs text-base-content/60 flex items-center gap-1" v-if="project.updatedAt">
            <span class="text-xs"><SvgIcon name="pencil" :size="14" />  更新于 {{ formatDate(project.updatedAt) }}</span>
          </span>
        </div>
        <!-- Git 仓库 -->
        <div class="flex flex-col gap-1" v-if="hasGitRepos">
          <div v-if="project.repoPath" class="flex items-center gap-1.5 text-xs text-base-content/60">
            <SvgIcon name="folder" :size="14" />
            <span class="flex-1 whitespace-nowrap overflow-hidden text-ellipsis font-mono text-[11px]">{{ project.repoPath.split('/').pop() }}</span>
            <span v-if="project.branch" class="badge badge-sm badge-primary">{{ project.branch }}</span>
          </div>
          <div v-if="project.repoPath2" class="flex items-center gap-1.5 text-xs text-base-content/60">
            <SvgIcon name="folder" :size="14" />
            <span class="flex-1 whitespace-nowrap overflow-hidden text-ellipsis font-mono text-[11px]">{{ project.repoPath2.split('/').pop() }}</span>
            <span v-if="project.branch2" class="badge badge-sm badge-primary">{{ project.branch2 }}</span>
          </div>
          <div v-if="project.gitUrl1" class="flex items-center gap-1.5 text-xs text-base-content/60">
            <SvgIcon name="globe" :size="14" />
            <span class="flex-1 whitespace-nowrap overflow-hidden text-ellipsis font-mono text-[11px]">{{ project.gitUrl1 }}</span>
          </div>
          <div v-if="project.gitUrl2" class="flex items-center gap-1.5 text-xs text-base-content/60">
            <SvgIcon name="globe" :size="14" />
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
          <template v-if="project.archived"><SvgIcon name="undo" :size="14" class="inline-block align-text-bottom" /></template><template v-else><SvgIcon name="archive" :size="14" /></template>
        </button>
        <button class="btn btn-ghost btn-sm text-error hover:bg-error hover:text-white" @click.stop="$emit('delete', project)" title="删除"><SvgIcon name="trash" :size="14" /></button>
        <button class="btn btn-primary btn-sm" @click.stop="$emit('edit', project)" title="编辑"><SvgIcon name="pencil" :size="14" /> </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { computed } from 'vue';

const props = defineProps({
  project: { type: Object, required: true },
  stats: { type: Object, default: () => ({ total: 0, completed: 0, progress: 0 }) },
});

defineEmits(['select', 'edit', 'toggle-archive', 'delete']);

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
  if (!dateStr) {return '';}
  return new Date(dateStr).toLocaleDateString('zh-CN', { month: '2-digit', day: '2-digit' });
};
</script>
