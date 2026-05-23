<template>
  <div class="p-5">
    <div class="flex justify-between items-center mb-5">
      <h2 class="text-2xl font-bold text-base-content m-0">项目管理</h2>
      <UiButton variant="primary" @click="openAddModal">+ 新建项目</UiButton>
    </div>

    <!-- 搜索和筛选 -->
    <div class="flex gap-3 mb-5 flex-wrap items-center">
      <div class="relative flex-1 min-w-[200px]">
        <SvgIcon class="absolute left-3 top-1/2 -translate-y-1/2 text-base-content/60 pointer-events-none" name="search" :size="16" />
        <input v-model="searchQuery" type="text" class="input input-bordered w-full pl-9" placeholder="搜索项目名称、描述或路径..." />
      </div>
      <div class="flex gap-2">
        <select v-model="archiveFilter" class="select select-bordered">
          <option value="active">活跃项目</option>
          <option value="archived">已归档</option>
          <option value="all">全部</option>
        </select>
        <select v-model="categoryFilter" class="select select-bordered">
          <option value="all">全部分类</option>
          <option value="frontend"><SvgIcon name="palette" :size="14" class="inline-block align-text-bottom" /> 前端</option>
          <option value="backend"><SvgIcon name="settings" :size="14" class="inline-block align-text-bottom" /> 后端</option>
          <option value="infrastructure"><SvgIcon name="build" :size="14" class="inline-block align-text-bottom" /> 基础设施</option>
          <option value="other"><SvgIcon name="tool" :size="14" class="inline-block align-text-bottom" /> 其他</option>
          <option value="">未分类</option>
        </select>
      </div>
    </div>

    <!-- 项目卡片网格 -->
    <div v-if="filteredProjects.length > 0" class="grid grid-cols-[repeat(auto-fill,minmax(320px,1fr))] gap-4">
      <div
        v-for="project in filteredProjects"
        :key="project.id"
        class="card bg-base-100 border border-base-content/10 rounded-xl p-5 transition-all duration-150 hover:-translate-y-0.5 hover:shadow-md"
        :class="{ 'opacity-70': project.archived }"
      >
        <div class="flex items-center gap-2.5 mb-2 flex-wrap">
          <span class="w-3 h-3 rounded-full flex-shrink-0" :style="{ background: project.color || '#6366f1' }"></span>
          <h3 class="text-base font-semibold text-base-content m-0 flex-1">{{ project.name }}</h3>
          <span v-if="project.archived" class="badge badge-error badge-sm">归档</span>
          <span v-if="project.category" class="badge badge-primary badge-sm">{{ categoryLabel(project.category) }}</span>
        </div>
        <p class="text-sm text-base-content/60 mb-3 min-h-[20px] leading-relaxed">{{ project.description || '暂无描述' }}</p>

        <!-- Git 信息 -->
        <div v-if="project.repoPath || project.gitUrl1" class="flex gap-1.5 mb-3 flex-wrap">
          <span v-if="project.repoPath" class="badge badge-ghost badge-sm gap-1 max-w-[200px] truncate" :title="project.repoPath">
            <SvgIcon name="folder" :size="14" />  {{ repoName(project.repoPath) }}
          </span>
          <span v-if="project.gitUrl1" class="badge badge-ghost badge-sm gap-1 max-w-[200px] truncate" :title="project.gitUrl1">
            <SvgIcon name="link" :size="14" />  {{ repoName(project.gitUrl1) }}
          </span>
        </div>

        <div class="flex gap-4 mb-2.5">
          <div class="text-center">
            <span class="text-lg font-bold text-base-content block">{{ project.stats?.total ?? 0 }}</span>
            <span class="text-xs text-base-content/60">任务</span>
          </div>
          <div class="text-center">
            <span class="text-lg font-bold text-success block">{{ project.stats?.completed ?? 0 }}</span>
            <span class="text-xs text-base-content/60">已完成</span>
          </div>
          <div class="text-center">
            <span class="text-lg font-bold text-base-content block">{{ project.stats?.progress ?? 0 }}%</span>
            <span class="text-xs text-base-content/60">进度</span>
          </div>
        </div>

        <div class="h-1.5 bg-base-content/10 rounded-full overflow-hidden mb-4">
          <div
            class="h-full rounded-full transition-all duration-300"
            :style="{ width: `${project.stats?.progress ?? 0}%`, background: project.color || '#6366f1' }"
          ></div>
        </div>

        <div class="flex gap-1.5 flex-wrap">
          <button class="btn btn-ghost btn-sm gap-1" @click="selectProject(project)" title="进入项目">
            <SvgIcon name="chevronRight" :size="16" />
            进入
          </button>
          <button class="btn btn-ghost btn-sm gap-1" @click="openEditModal(project)" title="编辑">
            <SvgIcon name="pencil" :size="14" />
            编辑
          </button>
          <button class="btn btn-ghost btn-sm gap-1" @click="toggleArchive(project)" :title="project.archived ? '恢复' : '归档'">
            <SvgIcon :name="project.archived ? 'undo' : 'archive'" :size="14" />
            {{ project.archived ? '恢复' : '归档' }}
          </button>
          <button class="btn btn-ghost btn-sm gap-1 text-error hover:bg-error/10 hover:text-error" @click="handleDelete(project.id)" title="删除">
            <SvgIcon name="trash" :size="14" />
            删除
          </button>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <UiEmptyState v-else>
      <template #icon>
        <SvgIcon name="folder" :size="64" :stroke-width="1.5" />
      </template>
      <template #default>
        <p v-if="searchQuery || archiveFilter !== 'active' || categoryFilter !== 'all'">没有找到匹配的项目</p>
        <p v-else>暂无项目，点击上方按钮创建第一个项目</p>
      </template>
      <template v-if="!searchQuery && archiveFilter === 'active' && categoryFilter === 'all'" #action>
        <div class="flex gap-2 justify-center mt-3">
          <button class="btn btn-primary" @click="openAddModal">+ 创建项目</button>
        </div>
      </template>
    </UiEmptyState>

    <!-- 添加/编辑项目模态框 -->
    <UiModal
      v-model="showModal"
      :title="editingProject ? $t('project.edit') : $t('project.add')"
      @close="resetModal"
      width="640px"
    >
      <ProjectForm ref="projectFormRef" :project="editingProject" @save="saveProject" />
      <template #footer>
        <UiButton variant="ghost" @click="resetModal">取消</UiButton>
        <UiButton variant="primary" @click="projectFormRef?.submit()">
          {{ editingProject ? '保存修改' : '创建项目' }}
        </UiButton>
      </template>
    </UiModal>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, computed, onMounted } from 'vue';
import SvgIcon from '@/components/ui/SvgIcon.vue';
import ProjectForm from '@/views/projects/ProjectForm.vue';
import UiButton from '@/components/ui/Button.vue';
import UiModal from '@/components/ui/Modal.vue';
import UiEmptyState from '@/components/ui/EmptyState.vue';
import { getTauriAPI } from '../../utils/tauri-api';
import { useProjects } from '../../composables/useProjects';
import { useErrorHandler } from '../../composables/useErrorHandler';
import type { Project } from '../../types';

const projectsApi = useProjects();
const { handleError } = useErrorHandler();

const projects = ref<(Project & { stats?: { total: number; completed: number; progress: number } })[]>([]);
const searchQuery = ref('');
const archiveFilter = ref('active');
const categoryFilter = ref('all');
const showModal = ref(false);
const editingProject = ref<Project | null>(null);
const projectFormRef = ref<InstanceType<typeof ProjectForm> | null>(null);

const categoryLabel = (cat: string) => {
  const map: Record<string, string> = {
    frontend: '前端',
    backend: '后端',
    infrastructure: '基础设施',
    other: '其他',
  };
  return map[cat] || cat;
};

const repoName = (path: string) => {
  const parts = path.replace(/\/$/, '').split('/');
  return parts[parts.length - 1] || path;
};

const filteredProjects = computed(() => {
  let result = projects.value;

  if (archiveFilter.value === 'active') {
    result = result.filter(p => !p.archived);
  } else if (archiveFilter.value === 'archived') {
    result = result.filter(p => p.archived);
  }

  if (categoryFilter.value !== 'all') {
    result = result.filter(p => p.category === categoryFilter.value);
  }

  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase();
    result = result.filter(p =>
      p.name.toLowerCase().includes(q) ||
      (p.description && p.description.toLowerCase().includes(q)) ||
      (p.repoPath && p.repoPath.toLowerCase().includes(q)) ||
      (p.gitUrl1 && p.gitUrl1.toLowerCase().includes(q)) ||
      (p.gitUrl2 && p.gitUrl2.toLowerCase().includes(q))
    );
  }

  return result;
});

const loadProjects = async () => {
  try {
    const allProjects = await projectsApi.fetchProjects(false);
    projects.value = await Promise.all(
      allProjects.map(async (p) => {
        try {
          const stats = await projectsApi.getProjectStats(p.id);
          return { ...p, stats };
        } catch {
          return { ...p, stats: { total: 0, completed: 0, progress: 0 } };
        }
      })
    );
  } catch (error) {
    handleError(error, { context: 'loadProjects' });
  }
};

const selectProject = (project: Project) => {
  emit('selectProject', project);
};

const openAddModal = () => {
  editingProject.value = null;
  showModal.value = true;
};

const openEditModal = (project: Project) => {
  editingProject.value = { ...project };
  showModal.value = true;
};

const resetModal = () => {
  editingProject.value = null;
  showModal.value = false;
  if (projectFormRef.value) {
    projectFormRef.value.reset();
  }
};

const saveProject = async (formData: any) => {
  try {
    const projectData: any = {
      ...formData,
      archived: editingProject.value?.archived ?? false,
      updatedAt: new Date().toISOString(),
    };
    if (editingProject.value) {
      projectData.id = editingProject.value.id;
      projectData.createdAt = editingProject.value.createdAt;
      await projectsApi.updateProject(projectData);
    } else {
      projectData.id = crypto.randomUUID();
      projectData.createdAt = new Date().toISOString();
      await projectsApi.addProject(projectData);
    }
    resetModal();
    await loadProjects();
  } catch (error) {
    handleError(error, { context: 'saveProject' });
  }
};

const toggleArchive = async (project: Project) => {
  try {
    const updated = {
      ...project,
      archived: !project.archived,
      updatedAt: new Date().toISOString(),
    };
    await getTauriAPI().updateProject(updated);
    await loadProjects();
  } catch (error) {
    handleError(error, { context: 'toggleArchive' });
  }
};

const handleDelete = async (id: string) => {
  if (!confirm('确定删除此项目？所有关联任务也将被删除。')) {return;}
  try {
    await projectsApi.deleteProject(id);
    await loadProjects();
  } catch (error) {
    handleError(error, { context: 'deleteProject' });
  }
};

const emit = defineEmits(['selectProject']);

onMounted(async () => {
  await loadProjects();
});
</script>
