<template>
  <div class="p-5">
    <div class="flex justify-between items-center mb-5">
      <h2 class="m-0 text-2xl text-base-content">{{ $t('project.list') }}</h2>
      <UiButton @click="openAddModal">+ {{ $t('project.add') }}</UiButton>
    </div>

    <!-- 搜索和筛选 -->
    <div class="flex gap-3 mb-5 flex-wrap items-center">
      <div class="relative flex-1 min-w-[200px]">
        <svg class="absolute left-3 top-1/2 -translate-y-1/2 text-base-content/60 pointer-events-none" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8" /><path d="m21 21-4.35-4.35" />
        </svg>
        <input v-model="searchQuery" type="text" class="input input-bordered w-full pl-9 text-sm placeholder:text-base-content/60 placeholder:opacity-70" :placeholder="$t('projectList.searchPlaceholder')" />
      </div>
      <div class="flex gap-2">
        <select v-model="archiveFilter" class="select select-bordered text-sm">
          <option value="active">{{ $t('projectList.archive.active') }}</option>
          <option value="archived">{{ $t('projectList.archive.archived') }}</option>
          <option value="all">{{ $t('projectList.archive.all') }}</option>
        </select>
        <select v-model="categoryFilter" class="select select-bordered text-sm">
          <option value="all">{{ $t('projectList.category.all') }}</option>
          <option value="frontend"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="13.5" cy="6.5" r="0.5" fill="currentColor"/><circle cx="17.5" cy="10.5" r="0.5" fill="currentColor"/><circle cx="8.5" cy="7.5" r="0.5" fill="currentColor"/><circle cx="6.5" cy="12.5" r="0.5" fill="currentColor"/><path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.93 0 1.5-.67 1.5-1.5 0-.39-.15-.74-.39-1.01-.23-.26-.38-.61-.38-1 0-.83.67-1.5 1.5-1.5H16c3.31 0 6-2.69 6-6 0-4.5-4.22-8-10-8z"/></svg> {{ $t('projectForm.frontend') }}</option>
          <option value="backend"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg> {{ $t('projectForm.backend') }}</option>
          <option value="infrastructure"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg> {{ $t('projectForm.infrastructure') }}</option>
          <option value="other"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M12 2L15.09 8.26L22 9.27L17 14.14L18.18 21.02L12 17.77L5.82 21.02L7 14.14L2 9.27L8.91 8.26L12 2z"/></svg> {{ $t('projectForm.other') }}</option>
          <option value="">{{ $t('projectForm.uncategorized') }}</option>
        </select>
      </div>
    </div>

    <!-- 项目列表 - 一行一个 -->
    <div v-if="filteredProjects.length > 0" class="flex flex-col gap-4">
      <ProjectItem
        v-for="project in filteredProjects"
        :key="project.id"
        :project="project"
        :stats="project.stats"
        @select="selectProject"
        @edit="openEditModal"
        @toggle-archive="toggleArchive"
      />
    </div>

    <!-- 空状态 -->
    <UiEmptyState v-else :text="searchQuery || categoryFilter !== 'all' ? '没有找到匹配的项目' : $t('project.noProjects')" :subtext="searchQuery ? '尝试其他搜索词' : $t('project.noProjectsSub')">
      <template #icon>
        <svg v-if="!searchQuery && categoryFilter === 'all'" viewBox="0 0 24 24" width="64" height="64" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
        </svg>
      </template>
      <template #action>
        <div class="flex gap-2 justify-center mb-3" v-if="!searchQuery && categoryFilter === 'all'">
          <span class="inline-flex items-center gap-1 text-xs text-base-content/60"><kbd class="px-1.5 py-0.5 rounded bg-base-200 border border-base-content/10 text-xs font-[inherit]">Enter</kbd> 创建项目</span>
          <span class="inline-flex items-center gap-1 text-xs text-base-content/60"><kbd class="px-1.5 py-0.5 rounded bg-base-200 border border-base-content/10 text-xs font-[inherit]">Esc</kbd> 关闭弹窗</span>
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
        <UiButton variant="ghost" @click="resetModal">{{ $t('common.cancel') }}</UiButton>
        <UiButton variant="primary" @click="projectFormRef?.submit()">
          {{ editingProject ? $t('project.saveChanges') : $t('project.create') }}
        </UiButton>
      </template>
    </UiModal>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import ProjectItem from '@/views/projects/ProjectItem.vue';
import ProjectForm from '@/views/projects/ProjectForm.vue';
import UiButton from '@/components/ui/Button.vue';
import UiModal from '@/components/ui/Modal.vue';
import UiEmptyState from '@/components/ui/EmptyState.vue';
import { useProjects } from '../../composables/useProjects';
import { useErrorHandler } from '../../composables/useErrorHandler';
import { getTauriAPI } from '../../utils/tauri-api';
import type { Project } from '../../types';

const router = useRouter();

const projectsApi = useProjects();
const { handleError } = useErrorHandler();

const projects = ref<Project[]>([]);
const searchQuery = ref('');
const archiveFilter = ref('active');
const categoryFilter = ref('all');
const showModal = ref(false);
const editingProject = ref<Project | null>(null);
const projectFormRef = ref<InstanceType<typeof ProjectForm> | null>(null);

const filteredProjects = computed(() => {
  let result = projects.value;

  // 归档筛选
  if (archiveFilter.value === 'active') {
    result = result.filter(p => !p.archived);
  } else if (archiveFilter.value === 'archived') {
    result = result.filter(p => p.archived);
  }

  // 分类筛选
  if (categoryFilter.value !== 'all') {
    result = result.filter(p => p.category === categoryFilter.value);
  }

  // 搜索
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
    console.log("[loadProjects] called")
    const allProjects = await projectsApi.fetchProjects(false);
    // 加载统计
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
  router.push(`/project/${project.id}`);
};

const openAddModal = () => {
  editingProject.value = null;
  showModal.value = true;
};

const openEditModal = (project: Project) => {
  editingProject.value = { ...project };
  showModal.value = true;
};

const closeModal = () => {
  showModal.value = false;
  editingProject.value = null;
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
    console.log("[saveProject] called")
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
    console.log("[toggleArchive] called")
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

onMounted(async () => {
    console.log("[components/ProjectList.vue] mounted")
  await loadProjects();
});
</script>
