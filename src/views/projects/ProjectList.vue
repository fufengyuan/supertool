<template>
  <div class="p-5">
    <div class="flex justify-between items-center mb-5">
      <h2 class="m-0 text-2xl text-base-content">{{ $t('project.list') }}</h2>
      <UiButton @click="openAddModal">+ {{ $t('project.add') }}</UiButton>
    </div>

    <!-- 搜索和筛选 -->
    <div class="flex gap-3 mb-5 flex-wrap items-center">
      <div class="relative flex-1 min-w-[200px]">
        <SvgIcon class="absolute left-3 top-1/2 -translate-y-1/2 text-base-content/60 pointer-events-none" name="search" :size="16" />
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
          <option value="frontend"><SvgIcon name="palette" :size="14" class="inline-block align-text-bottom" /> {{ $t('projectForm.frontend') }}</option>
          <option value="backend"><SvgIcon name="settings" :size="14" class="inline-block align-text-bottom" /> {{ $t('projectForm.backend') }}</option>
          <option value="infrastructure"><SvgIcon name="build" :size="14" class="inline-block align-text-bottom" /> {{ $t('projectForm.infrastructure') }}</option>
          <option value="other"><SvgIcon name="star" :size="14" class="inline-block align-text-bottom" /> {{ $t('projectForm.other') }}</option>
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
        <SvgIcon v-if="!searchQuery && categoryFilter === 'all'" name="folder" size="64" stroke-width="1.5" />
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
import SvgIcon from '@/components/ui/SvgIcon.vue';
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
