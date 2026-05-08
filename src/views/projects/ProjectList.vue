<template>
  <div class="project-list-container">
    <div class="project-header">
      <h2>{{ $t('project.list') }}</h2>
      <UiButton @click="openAddModal">+ {{ $t('project.add') }}</UiButton>
    </div>

    <!-- 搜索和筛选 -->
    <div class="filters-bar">
      <div class="search-wrapper">
        <svg class="search-icon" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8" /><path d="m21 21-4.35-4.35" />
        </svg>
        <input v-model="searchQuery" type="text" class="search-input" :placeholder="$t('projectList.searchPlaceholder')" />
      </div>
      <div class="filter-group">
        <select v-model="archiveFilter" class="filter-select">
          <option value="active">{{ $t('projectList.archive.active') }}</option>
          <option value="archived">{{ $t('projectList.archive.archived') }}</option>
          <option value="all">{{ $t('projectList.archive.all') }}</option>
        </select>
        <select v-model="categoryFilter" class="filter-select">
          <option value="all">{{ $t('projectList.category.all') }}</option>
          <option value="frontend">🎨 {{ $t('projectForm.frontend') }}</option>
          <option value="backend">⚙️ {{ $t('projectForm.backend') }}</option>
          <option value="infrastructure">🏗️ {{ $t('projectForm.infrastructure') }}</option>
          <option value="other">📌 {{ $t('projectForm.other') }}</option>
          <option value="">{{ $t('projectForm.uncategorized') }}</option>
        </select>
      </div>
    </div>

    <!-- 项目列表 - 一行一个 -->
    <div v-if="filteredProjects.length > 0" class="project-list">
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
        <div class="empty-state-shortcuts" v-if="!searchQuery && categoryFilter === 'all'">
          <span class="shortcut-tag"><kbd>Enter</kbd> 创建项目</span>
          <span class="shortcut-tag"><kbd>Esc</kbd> 关闭弹窗</span>
        </div>
      </template>
    </UiEmptyState>

    <!-- 添加/编辑项目模态框 -->
    <UiModal
      v-model="showModal"
      :title="editingProject ? '✏️ ' + $t('project.edit') : '✨ ' + $t('project.add')"
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
import { useRouter, useRoute } from 'vue-router';
import ProjectItem from '@/views/projects/ProjectItem.vue';
import ProjectForm from '@/views/projects/ProjectForm.vue';
import UiButton from '@/components/ui/Button.vue';
import UiModal from '@/components/ui/Modal.vue';
import UiEmptyState from '@/components/ui/EmptyState.vue';
import { useProjects } from '../../composables/useProjects';
import { useErrorHandler } from '../../composables/useErrorHandler';
import type { Project } from '../../types';

const router = useRouter();
const route = useRoute();

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
    await projectsApi.updateProject(updated as unknown as Project);
    await loadProjects();
  } catch (error) {
    handleError(error, { context: 'toggleArchive' });
  }
};

onMounted(async () => {
    console.log("[views/projects/ProjectList.vue] mounted")
  await loadProjects();
  // 从详情页点编辑跳回来时自动打开编辑弹窗
  const editId = route.query.edit as string;
  if (editId) {
    const target = projects.value.find(p => p.id === editId);
    if (target) openEditModal(target);
  }
});
</script>

<style scoped>
.project-list-container { padding: 20px; }
.project-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
.project-header h2 { margin: 0; color: var(--color-base-content); font-size: 24px; }

/* 筛选栏 */
.filters-bar { display: flex; gap: 12px; margin-bottom: 20px; flex-wrap: wrap; align-items: center; }
.search-wrapper { position: relative; flex: 1; min-width: 200px; }
.search-icon { position: absolute; left: 12px; top: 50%; transform: translateY(-50%); color: color-mix(in oklab, var(--color-base-content) 60%, transparent); pointer-events: none; }
.search-input { width: 100%; padding: 10px 14px 10px 36px; border: 1.5px solid color-mix(in oklab, var(--color-base-content) 20%, transparent); border-radius: 10px; background: var(--color-base-200); color: var(--color-base-content); font-size: 14px; outline: none; transition: all 0.15s ease; }
.search-input:focus { border-color: var(--color-primary); box-shadow: 0 0 0 3px color-mix(in oklab, var(--color-primary) 10%, transparent); }
.search-input::placeholder { color: color-mix(in oklab, var(--color-base-content) 60%, transparent); opacity: 0.7; }
.filter-group { display: flex; gap: 8px; }
.filter-select { padding: 10px 14px; border: 1.5px solid color-mix(in oklab, var(--color-base-content) 20%, transparent); border-radius: 10px; background: var(--color-base-200); color: var(--color-base-content); font-size: 13px; cursor: pointer; outline: none; transition: all 0.15s ease; }
.filter-select:focus { border-color: var(--color-primary); box-shadow: 0 0 0 3px color-mix(in oklab, var(--color-primary) 10%, transparent); }

/* 项目列表 - 一行一个，全宽 */
.project-list { display: flex; flex-direction: column; gap: 16px; }

.empty-state-shortcuts { display: flex; gap: 8px; justify-content: center; margin-bottom: 12px; }
.shortcut-tag { display: inline-flex; align-items: center; gap: 4px; font-size: 12px; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); }
.shortcut-tag kbd { padding: 2px 6px; border-radius: 4px; background: var(--color-base-200); border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); font-size: 11px; font-family: inherit; }
</style>
