import { getTauriAPI } from '../utils/tauri-api'
import { ref } from 'vue';
import type { Ref } from 'vue';
import { useErrorHandler } from './useErrorHandler';
import type { Project } from '../types';

/**
 * useProjects - 统一项目数据库操作 composable
 * 封装所有 Tauri API 的项目相关调用
 */
export function useProjects() {
  const loading: Ref<boolean> = ref(false);
  const error: Ref<string | null> = ref(null);
  const { handleError } = useErrorHandler();

  // ============ 基础 CRUD ============

  const fetchProjects = async (onlyActive = true): Promise<Project[]> => {
    console.log("[useProjects.ts] fetchProjects() called")
    loading.value = true;
    error.value = null;
    try {
      const projects = await getTauriAPI().getProjects(onlyActive);
      return projects || [];
    } catch (err: unknown) {
      error.value = (err as Error).message;
      handleError(err, { context: 'fetchProjects' });
      return [];
    } finally {
      loading.value = false;
    }
  };

  const addProject = async (projectData: Partial<Project>): Promise<Project> => {
    console.log("[useProjects.ts] addProject() called")
    error.value = null;
    try {
      return await getTauriAPI().addProject(JSON.parse(JSON.stringify(projectData)));
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'addProject', rethrow: true });
      throw err; // safety fallback for TS
    }
  };

  const updateProject = async (projectData: Project): Promise<Project> => {
    console.log("[useProjects.ts] updateProject() called")
    error.value = null;
    try {
      return await getTauriAPI().updateProject(JSON.parse(JSON.stringify(projectData)));
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'updateProject', rethrow: true });
      throw err; // safety fallback for TS
    }
  };

  const deleteProject = async (id: string): Promise<void> => {
    console.log("[useProjects.ts] deleteProject() called")
    error.value = null;
    try {
      return await getTauriAPI().deleteProject(id);
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'deleteProject', rethrow: true });
    }
  };

  // ============ 项目统计 ============

  const getProjectStats = async (projectId: string): Promise<{ total: number; completed: number; progress: number }> => {
    error.value = null;
    try {
      return (
        (await getTauriAPI().getProjectStats(projectId)) || {
          total: 0,
          completed: 0,
          progress: 0,
        }
      );
    } catch (err: unknown) {
      error.value = (err as Error).message;
      handleError(err, { context: 'getProjectStats', showToast: false });
      return { total: 0, completed: 0, progress: 0 };
    }
  };

  const getProjectTodos = async (projectId: string): Promise<unknown[]> => {
    console.log("[useProjects.ts] getProjectTodos() called")
    error.value = null;
    try {
      return (await getTauriAPI().getProjectTodos(projectId)) || [];
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'getProjectTodos' });
      return [];
    }
  };

  // ============ Git 相关 ============

  const getGitCommits = async (gitUrl: string, sinceDate: string): Promise<unknown[]> => {
    console.log("[useProjects.ts] getGitCommits() called")
    error.value = null;
    try {
      return (await getTauriAPI().getGitCommits(gitUrl, sinceDate)) || [];
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'getGitCommits', showToast: false });
      return [];
    }
  };

  const getGitBranches = async (gitUrl: string): Promise<string[]> => {
    console.log("[useProjects.ts] getGitBranches() called")
    error.value = null;
    try {
      return ((await getTauriAPI().getGitBranches(gitUrl)) || []).map((b: any) => typeof b === 'string' ? b : b.name);
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'getGitBranches', showToast: false });
      return [];
    }
  };

  const scanLocalGitRepos = async (directories: string[]): Promise<unknown[]> => {
    console.log("[useProjects.ts] scanLocalGitRepos() called")
    error.value = null;
    try {
      return (await getTauriAPI().scanLocalGitRepos(directories)) || [];
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'scanLocalGitRepos', showToast: false });
      return [];
    }
  };

  // ============ 项目列表加载（含统计） ============

  const loadProjectsWithStats = async (onlyActive = true): Promise<(Project & { stats: { total: number; completed: number; progress: number } })[]> => {
    loading.value = true;
    error.value = null;
    try {
      const projects = await fetchProjects(onlyActive);
      // 为每个项目加载统计信息
      const projectsWithStats = await Promise.all(
        projects.map(async (project) => {
          const stats = await getProjectStats(project.id);
          return { ...project, stats };
        })
      );
      return projectsWithStats;
    } catch (err: unknown) {
      error.value = (err as Error).message;
      handleError(err, { context: 'loadProjectsWithStats' });
      return [];
    } finally {
      loading.value = false;
    }
  };

  return {
    // 状态
    loading,
    error,
    // CRUD
    fetchProjects,
    addProject,
    updateProject,
    deleteProject,
    // 统计
    getProjectStats,
    getProjectTodos,
    // Git
    getGitCommits,
    getGitBranches,
    scanLocalGitRepos,
    // 组合操作
    loadProjectsWithStats,
  };
}
