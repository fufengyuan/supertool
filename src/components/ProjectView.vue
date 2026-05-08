<template>
  <div class="project-list-container">
    <div class="project-header">
      <h2>项目管理</h2>
      <UiButton variant="primary" @click="openAddModal">+ 新建项目</UiButton>
    </div>

    <!-- 搜索和筛选 -->
    <div class="filters-bar">
      <div class="search-wrapper">
        <svg class="search-icon" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8" /><path d="m21 21-4.35-4.35" />
        </svg>
        <input v-model="searchQuery" type="text" class="search-input" placeholder="搜索项目名称、描述或路径..." />
      </div>
      <div class="filter-group">
        <select v-model="archiveFilter" class="filter-select">
          <option value="active">活跃项目</option>
          <option value="archived">已归档</option>
          <option value="all">全部</option>
        </select>
        <select v-model="categoryFilter" class="filter-select">
          <option value="all">全部分类</option>
          <option value="frontend">🎨 前端</option>
          <option value="backend">⚙️ 后端</option>
          <option value="infrastructure">🏗️ 基础设施</option>
          <option value="other">📌 其他</option>
          <option value="">未分类</option>
        </select>
      </div>
    </div>

    <!-- 项目卡片网格 -->
    <div v-if="filteredProjects.length > 0" class="project-grid">
      <div
        v-for="project in filteredProjects"
        :key="project.id"
        class="project-card"
        :class="{ archived: project.archived }"
      >
        <div class="card-header">
          <span class="color-dot" :style="{ background: project.color || '#6366f1' }"></span>
          <h3>{{ project.name }}</h3>
          <span v-if="project.archived" class="archive-badge">归档</span>
          <span v-if="project.category" class="category-badge">{{ categoryLabel(project.category) }}</span>
        </div>
        <p class="project-desc">{{ project.description || '暂无描述' }}</p>

        <!-- Git 信息 -->
        <div v-if="project.repoPath || project.gitUrl1" class="git-info">
          <span v-if="project.repoPath" class="git-tag" :title="project.repoPath">
            📁 {{ repoName(project.repoPath) }}
          </span>
          <span v-if="project.gitUrl1" class="git-tag" :title="project.gitUrl1">
            🔗 {{ repoName(project.gitUrl1) }}
          </span>
        </div>

        <div class="project-stats">
          <div class="stat-item">
            <span class="stat-value">{{ project.stats?.total ?? 0 }}</span>
            <span class="stat-label">任务</span>
          </div>
          <div class="stat-item">
            <span class="stat-value done">{{ project.stats?.completed ?? 0 }}</span>
            <span class="stat-label">已完成</span>
          </div>
          <div class="stat-item">
            <span class="stat-value">{{ project.stats?.progress ?? 0 }}%</span>
            <span class="stat-label">进度</span>
          </div>
        </div>

        <div class="progress-bar">
          <div
            class="progress-fill"
            :style="{ width: `${project.stats?.progress ?? 0}%`, background: project.color || '#6366f1' }"
          ></div>
        </div>

        <div class="card-actions">
          <button class="action-btn" @click="selectProject(project)" title="进入项目">
            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18l6-6-6-6"/></svg>
            进入
          </button>
          <button class="action-btn" @click="openEditModal(project)" title="编辑">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
            编辑
          </button>
          <button class="action-btn" @click="toggleArchive(project)" :title="project.archived ? '恢复' : '归档'">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><polyline points="21 8 21 21 3 21 3 8"/><rect x="1" y="3" width="22" height="5"/><line x1="10" y1="12" x2="14" y2="12"/></svg>
            {{ project.archived ? '恢复' : '归档' }}
          </button>
          <button class="action-btn danger" @click="handleDelete(project.id)" title="删除">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
            删除
          </button>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <UiEmptyState v-else>
      <template #icon>
        <svg viewBox="0 0 24 24" width="64" height="64" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
        </svg>
      </template>
      <template #default>
        <p v-if="searchQuery || archiveFilter !== 'active' || categoryFilter !== 'all'">没有找到匹配的项目</p>
        <p v-else>暂无项目，点击上方按钮创建第一个项目</p>
      </template>
      <template v-if="!searchQuery && archiveFilter === 'active' && categoryFilter === 'all'" #action>
        <div class="empty-state-shortcuts">
          <button class="btn-primary" @click="openAddModal">+ 创建项目</button>
        </div>
      </template>
    </UiEmptyState>

    <!-- 添加/编辑项目模态框 -->
    <UiModal
      v-model="showModal"
      :title="editingProject ? '✏️ 编辑项目' : '✨ 新建项目'"
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
import ProjectForm from '@/components/ProjectForm.vue';
import UiButton from '@/components/ui/Button.vue';
import UiModal from '@/components/ui/Modal.vue';
import UiEmptyState from '@/components/ui/EmptyState.vue';
import { getTauriAPI } from '../utils/tauri-api';
import { useProjects } from '../composables/useProjects';
import { useErrorHandler } from '../composables/useErrorHandler';
import type { Project } from '../types';

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
    frontend: '🎨 前端',
    backend: '⚙️ 后端',
    infrastructure: '🏗️ 基础设施',
    other: '📌 其他',
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
  if (!confirm('确定删除此项目？所有关联任务也将被删除。')) return;
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

<style scoped>
.project-list-container {
  padding: 20px;
}

.project-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.project-header h2 {
  margin: 0;
  color: var(--color-base-content);
  font-size: 24px;
}

/* 筛选栏 */
.filters-bar {
  display: flex;
  gap: 12px;
  margin-bottom: 20px;
  flex-wrap: wrap;
  align-items: center;
}

.search-wrapper {
  position: relative;
  flex: 1;
  min-width: 200px;
}

.search-icon {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: 10px 14px 10px 36px;
  border: 1.5px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 10px;
  background: var(--color-base-200);
  color: var(--color-base-content);
  font-size: 14px;
  outline: none;
  transition: all 0.15s ease;
}

.search-input:focus {
  border-color: var(--color-primary);
}

.search-input::placeholder {
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  opacity: 0.7;
}

.filter-group {
  display: flex;
  gap: 8px;
}

.filter-select {
  padding: 10px 14px;
  border: 1.5px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 10px;
  background: var(--color-base-200);
  color: var(--color-base-content);
  font-size: 13px;
  cursor: pointer;
  outline: none;
  transition: all 0.15s ease;
}

.filter-select:focus {
  border-color: var(--color-primary);
}

/* 项目卡片网格 */
.project-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
}

.project-card {
  background: var(--color-base-100);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 12px;
  padding: 20px;
  transition: transform 0.15s, box-shadow 0.15s;
}

.project-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 25px rgba(0, 0, 0, 0.15);
}

.project-card.archived {
  opacity: 0.7;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
  flex-wrap: wrap;
}

.color-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  flex-shrink: 0;
}

.card-header h3 {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-base-content);
  margin: 0;
  flex: 1;
}

.archive-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 10px;
  background: var(--danger-bg, #fef2f2);
  color: var(--color-error);
  font-weight: 500;
}

.category-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 10px;
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
  color: var(--color-primary);
  font-weight: 500;
}

.project-desc {
  font-size: 13px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  margin-bottom: 12px;
  min-height: 20px;
  line-height: 1.5;
}

.git-info {
  display: flex;
  gap: 6px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.git-tag {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 6px;
  background: var(--color-base-200);
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.project-stats {
  display: flex;
  gap: 16px;
  margin-bottom: 10px;
}

.stat-item {
  text-align: center;
}

.stat-value {
  font-size: 18px;
  font-weight: 700;
  color: var(--color-base-content);
  display: block;
}

.stat-value.done {
  color: var(--color-success);
}

.stat-label {
  font-size: 11px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.progress-bar {
  height: 6px;
  background: color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 3px;
  overflow: hidden;
  margin-bottom: 16px;
}

.progress-fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.3s;
}

.card-actions {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.action-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  border-radius: 6px;
  cursor: pointer;
  background: var(--color-base-200);
  color: var(--color-base-content);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  font-size: 12px;
  transition: all 0.15s ease;
}

.action-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.action-btn.danger:hover {
  border-color: var(--color-error);
  color: var(--color-error);
}

.empty-state-shortcuts {
  display: flex;
  gap: 8px;
  justify-content: center;
  margin-top: 12px;
}

.btn-primary {
  padding: 8px 16px;
  background: var(--color-primary);
  color: #fff;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
}

.btn-primary:hover {
  opacity: 0.9;
}
</style>
