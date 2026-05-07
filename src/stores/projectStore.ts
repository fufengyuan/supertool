import { defineStore } from 'pinia';
import { ref } from 'vue';
import { useProjects } from '../composables/useProjects';
import { useErrorHandler } from '../composables/useErrorHandler';
import type { Project } from '../types';

/**
 * projectStore - 项目状态管理
 * 管理 projects 数组、加载状态、当前选中的项目等
 */
export const useProjectStore = defineStore('projects', () => {
  const projectsApi = useProjects();
  const { handleError } = useErrorHandler();

  // ============ 状态 ============
  const projects = ref<(Project & { stats?: { total: number; completed: number; progress: number } })[]>([]);
  const currentProject = ref<Project | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  // ============ 操作 ============

  const loadProjects = async (onlyActive = true): Promise<void> => {
    loading.value = true;
    error.value = null;
    try {
      projects.value = await projectsApi.loadProjectsWithStats(onlyActive);
    } catch (err: unknown) {
      error.value = (err as Error).message;
      handleError(err, { context: 'loadProjects' });
    } finally {
      loading.value = false;
    }
  };

  const addProject = async (projectData: Partial<Project>): Promise<Project | undefined> => {
    try {
      const saved = await projectsApi.addProject(projectData);
      projects.value.push(saved);
      return saved;
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'addProject', rethrow: true });
    }
  };

  const updateProject = async (projectData: Project): Promise<Project | undefined> => {
    try {
      const updated = await projectsApi.updateProject(projectData);
      const index = projects.value.findIndex((p) => p.id === projectData.id);
      if (index !== -1) {
        projects.value[index] = updated;
      }
      // 同步更新当前选中的项目
      if (currentProject.value?.id === projectData.id) {
        currentProject.value = updated;
      }
      return updated;
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'updateProject', rethrow: true });
    }
  };

  const deleteProject = async (id: string): Promise<void> => {
    try {
      await projectsApi.deleteProject(id);
      projects.value = projects.value.filter((p) => p.id !== id);
      if (currentProject.value?.id === id) {
        currentProject.value = null;
      }
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'deleteProject', rethrow: true });
    }
  };

  const selectProject = (project: Project | null): void => {
    currentProject.value = project;
  };

  const clearCurrentProject = (): void => {
    currentProject.value = null;
  };

  const getProjectById = (id: string): Project | undefined => {
    return projects.value.find((p) => p.id === id);
  };

  const getProjectNameById = (id: string): string => {
    const project = projects.value.find((p) => p.id === id);
    return project ? project.name : '未知项目';
  };

  const getProjectColorById = (id: string): string => {
    const project = projects.value.find((p) => p.id === id);
    return project ? project.color ?? '#6366f1' : '#6366f1';
  };

  const refreshProjectStats = async (projectId: string): Promise<{ total: number; completed: number; progress: number } | null> => {
    try {
      const stats = await projectsApi.getProjectStats(projectId);
      const index = projects.value.findIndex((p) => p.id === projectId);
      if (index !== -1) {
        projects.value[index] = { ...projects.value[index], stats };
      }
      if (currentProject.value?.id === projectId) {
        currentProject.value = { ...currentProject.value, stats };
      }
      return stats;
    } catch (err) {
      handleError(err, { context: 'refreshProjectStats', showToast: false });
      return null;
    }
  };

  return {
    // 状态
    projects,
    currentProject,
    loading,
    error,
    // 操作
    loadProjects,
    addProject,
    updateProject,
    deleteProject,
    selectProject,
    clearCurrentProject,
    getProjectById,
    getProjectNameById,
    getProjectColorById,
    refreshProjectStats,
  };
});
