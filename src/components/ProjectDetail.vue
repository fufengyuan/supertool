<template>
  <div class="project-detail-container" v-if="project">
    <!-- 面包屑导航 -->
    <div class="breadcrumb">
      <button class="breadcrumb-link" @click="goBack" title="返回项目列表">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="15 18 9 12 15 6" />
        </svg>
        项目
      </button>
      <span class="breadcrumb-separator">\u203A</span>
      <span class="breadcrumb-current">{{ project.name }}</span>
    </div>

    <!-- 项目头部 -->
    <div class="project-header">
      <div class="project-info">
        <div class="project-color-dot" :style="{ backgroundColor: project.color }"></div>
        <div class="project-title-section">
          <h2>{{ project.name }}</h2>
          <p v-if="project.description" class="project-description">{{ project.description }}</p>
          <div class="project-meta">
            <span v-if="project.category" class="category-badge" :class="'category-' + project.category">{{ categoryLabel(project.category) }}</span>
            <span class="meta-tag" v-if="project.createdAt">📅 {{ formatDate(project.createdAt) }}</span>
          </div>
        </div>
      </div>
      <div class="project-stats">
        <div class="stats-row">
          <div class="stat-item">
            <span class="stat-value">{{ projectStats.total }}</span>
            <span class="stat-label">总任务</span>
          </div>
          <div class="stat-item completed">
            <span class="stat-value">{{ projectStats.completed }}</span>
            <span class="stat-label">已完成</span>
          </div>
          <div class="stat-item active">
            <span class="stat-value">{{ projectStats.total - projectStats.completed }}</span>
            <span class="stat-label">进行中</span>
          </div>
          <div class="stat-item">
            <span class="stat-value">{{ projectStats.progress }}%</span>
            <span class="stat-label">完成率</span>
          </div>
        </div>
        <div class="progress-container">
          <div class="progress-bar">
            <div class="progress-fill" :style="{ width: projectStats.progress + '%', backgroundColor: project.color }"></div>
          </div>
          <span class="progress-pct">{{ projectStats.progress }}%</span>
        </div>
      </div>
    </div>

    <!-- 标签页切换 -->
    <div class="tab-bar">
      <button class="tab-btn" :class="{ active: activeTab === 'active' }" @click="activeTab = 'active'">
        📋 进行中 ({{ activeTasks.length }})
      </button>
      <button class="tab-btn" :class="{ active: activeTab === 'completed' }" @click="activeTab = 'completed'">
        ✅ 已完成 ({{ completedTasks.length }})
      </button>
      <button class="tab-btn" :class="{ active: activeTab === 'git' }" @click="activeTab = 'git'">
        📜 Git 提交
      </button>
    </div>

    <!-- 进行中任务 -->
    <div v-if="activeTab === 'active'" class="tab-content">
      <ProjectTodoList
        :project="project"
        :tasks="activeTasks"
        :stats="projectStats"
        :project-color="project.color"
        @task-added="handleAddTask"
        @task-toggled="handleToggleTask"
        @task-deleted="handleDeleteTask"
        @data-reload="loadProjectData"
      />
    </div>

    <!-- 已完成任务 -->
    <div v-if="activeTab === 'completed'" class="tab-content">
      <div v-if="completedTasks.length === 0" class="empty-tasks">
        <span class="empty-icon">🔍</span>
        <p>暂无已完成任务</p>
      </div>
      <div v-else class="completed-tasks-list">
        <div v-for="task in completedTasks" :key="task.id" class="completed-task-item">
          <div class="task-info">
            <input type="checkbox" :checked="true" @change="handleToggleTask(task)" class="task-checkbox" />
            <span class="task-text completed-text">{{ task.text }}</span>
          </div>
          <div class="task-meta">
            <span v-if="task.completedAt" class="completed-date">✅ {{ formatDate(task.completedAt) }}</span>
            <span v-if="task.priority" class="priority-badge" :class="task.priority">{{ priorityLabel(task.priority) }}</span>
            <button class="undo-btn" @click="handleToggleTask(task)" title="恢复为未完成">↩️ 恢复</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Git 面板 -->
    <div v-if="activeTab === 'git'" class="tab-content">
      <ProjectGitPanel v-if="hasGitRepos" :project="project" />
      <div v-else class="empty-git">
        <span class="empty-icon">📜</span>
        <p>此项目未配置 Git 仓库</p>
        <button class="edit-git-btn" @click="editProject">✏️ 编辑项目配置</button>
      </div>
    </div>

    <!-- 操作按钮 -->
    <div class="project-actions">
      <UiButton variant="ghost" @click="goBack">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="15 18 9 12 15 6" />
        </svg>
        返回列表
      </UiButton>
      <UiButton variant="primary" @click="editProject">编辑项目</UiButton>
      <UiButton :variant="project.archived ? 'success' : 'warning'" @click="toggleArchive">
        {{ project.archived ? '取消归档' : '归档项目' }}
      </UiButton>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, computed, onMounted, watch } from 'vue'
import ProjectGitPanel from './project/ProjectGitPanel.vue'
import ProjectTodoList from './project/ProjectTodoList.vue'
import UiButton from './ui/Button.vue'
import { useTodoStore } from '../stores/todoStore'
import { useProjectStore } from '../stores/projectStore'
import { useProjects } from '../composables/useProjects'
import { useErrorHandler } from '../composables/useErrorHandler'
import type { Project } from '../types'

const props = defineProps({
  projectId: { type: String, required: true }
})

const emit = defineEmits(['goBackToList', 'editProject', 'projectUpdated'])

const todoStore = useTodoStore()
const projectStore = useProjectStore()
const projectsApi = useProjects()
const { handleError } = useErrorHandler()

const project = ref<Project | null>(null)

const projectStats = ref({ total: 0, completed: 0, progress: 0 })
const projectTasks = ref<any[]>([])
const activeTab = ref('active')

const activeTasks = computed(() => projectTasks.value.filter(t => !t.completed))
const completedTasks = computed(() => {
  const sorted = projectTasks.value.filter(t => t.completed)
  return sorted.sort((a, b) => {
    const da = a.completedAt ? new Date(a.completedAt).getTime() : 0
    const db = b.completedAt ? new Date(b.completedAt).getTime() : 0
    return db - da
  })
})
const hasGitRepos = computed(() =>
  project.value?.gitUrl1 || project.value?.gitUrl2 || project.value?.repoPath || project.value?.repoPath2
)

const categoryMap: Record<string, string> = {
  'frontend': '前端', 'backend': '后端', 'infrastructure': '基础设施', 'other': '其他'
}
const categoryLabel = (cat: string) => categoryMap[cat] || cat
const priorityLabel = (p: string) => ({ low: '低', medium: '中', high: '高' }[p] || p)
const formatDate = (d: string) => d ? new Date(d).toLocaleDateString('zh-CN', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' }) : ''

const loadProjectData = async () => {
  if (!project.value) return
  try {
    console.log("[loadProjectData] called");
    projectStats.value = await projectsApi.getProjectStats(project.value.id)
    projectTasks.value = await projectsApi.getProjectTodos(project.value.id)
  } catch (error) {
    handleError(error, { context: 'loadProjectData' })
  }
}

const handleAddTask = async (text: string) => {
  try {
    console.log("[handleAddTask] called");
    const newTask = {
      id: crypto.randomUUID(), text, completed: false, priority: 'medium' as const,
      dueDate: null, description: '', tag: '默认', createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(), projectId: project.value?.id
    }
    await todoStore.addTodo(newTask)
    await loadProjectData()
  } catch (error) { handleError(error, { context: 'handleAddTask' }) }
}

const handleToggleTask = async (task: any) => {
  try {
    console.log("[handleToggleTask] called");
    await todoStore.toggleTodo(task.id)
    await loadProjectData()
  } catch (error) { handleError(error, { context: 'handleToggleTask' }) }
}

const handleDeleteTask = async (task: any) => {
  try {
    console.log("[handleDeleteTask] called");
    await todoStore.deleteTodo(task.id)
    await loadProjectData()
  } catch (error) { handleError(error, { context: 'handleDeleteTask' }) }
}

const goBack = () => { emit('goBackToList') }
const editProject = () => { emit('editProject', project.value) }

const toggleArchive = async () => {
  if (!project.value) return
  try {
    console.log("[toggleArchive] called");
    const updated = { ...project.value, archived: !project.value.archived, updatedAt: new Date().toISOString() }
    await projectsApi.updateProject(updated as unknown as Project)
    emit('projectUpdated', updated)
  } catch (error) { handleError(error, { context: 'toggleArchive' }) }
}

const resolveProject = () => {
  const found = projectStore.projects.find(p => p.id === props.projectId)
  if (found) {
    project.value = found as unknown as Project
  }
}

// 当 projects 加载完成后自动解析
watch(() => projectStore.projects, () => {
  if (!project.value) resolveProject()
}, { immediate: true })

onMounted(async () => {
    console.log("[components/ProjectDetail.vue] mounted");
    // 确保项目列表已加载
    if (projectStore.projects.length === 0) {
      await projectStore.loadProjects()
    }
    resolveProject()
    await loadProjectData()
})
</script>

<style scoped>
.project-detail-container { padding: 20px; }
.breadcrumb { display: flex; align-items: center; gap: 8px; margin-bottom: 16px; font-size: 14px; }
.breadcrumb-link { display: inline-flex; align-items: center; gap: 4px; background: none; border: none; color: var(--primary-color); cursor: pointer; font-size: 14px; font-weight: 500; padding: 4px 8px; border-radius: 6px; transition: all 0.15s ease; }
.breadcrumb-link:hover { background: var(--primary-light); }
.breadcrumb-separator { color: var(--main-text-secondary); font-size: 18px; }
.breadcrumb-current { color: var(--main-text-secondary); font-weight: 500; }

.project-header { background: var(--card-bg); padding: 20px; border-radius: 12px; margin-bottom: 16px; box-shadow: var(--card-shadow); }
.project-info { display: flex; align-items: flex-start; gap: 12px; margin-bottom: 16px; }
.project-color-dot { width: 16px; height: 16px; border-radius: 50%; flex-shrink: 0; margin-top: 4px; }
.project-title-section h2 { margin: 0 0 8px 0; color: var(--main-text); font-size: 24px; }
.project-description { margin: 0 0 8px 0; color: var(--main-text-secondary); font-size: 14px; line-height: 1.4; }
.project-meta { display: flex; align-items: center; gap: 12px; }
.category-badge { display: inline-block; padding: 2px 10px; border-radius: 12px; font-size: 12px; font-weight: 500; background: var(--primary-light); color: var(--primary-color); }
.meta-tag { font-size: 12px; color: var(--main-text-secondary); }

.stats-row { display: flex; gap: 24px; margin-bottom: 12px; }
.stat-item { display: flex; flex-direction: column; align-items: center; }
.stat-value { font-size: 22px; font-weight: 700; color: var(--main-text); }
.stat-label { font-size: 12px; color: var(--main-text-secondary); }
.stat-item.completed .stat-value { color: #10b981; }
.stat-item.active .stat-value { color: #f59e0b; }

.progress-container { display: flex; align-items: center; gap: 12px; }
.progress-bar { flex: 1; height: 12px; background: var(--input-bg); border-radius: 6px; overflow: hidden; }
.progress-fill { height: 100%; transition: width 0.3s ease; }
.progress-pct { font-size: 14px; font-weight: 600; color: var(--main-text); white-space: nowrap; min-width: 40px; text-align: right; }

/* 标签页 */
.tab-bar { display: flex; gap: 4px; margin-bottom: 16px; background: var(--input-bg); border-radius: 10px; padding: 4px; }
.tab-btn { flex: 1; padding: 10px 16px; border: none; border-radius: 8px; background: transparent; color: var(--main-text-secondary); cursor: pointer; font-size: 14px; font-weight: 500; transition: all 0.15s; display: flex; align-items: center; justify-content: center; gap: 6px; }
.tab-btn:hover { color: var(--main-text); }
.tab-btn.active { background: var(--card-bg); color: var(--main-text); box-shadow: 0 1px 3px rgba(0,0,0,0.1); }

/* 已完成任务 */
.completed-tasks-list { display: flex; flex-direction: column; gap: 8px; }
.completed-task-item { display: flex; align-items: center; justify-content: space-between; padding: 10px 14px; background: var(--input-bg); border-radius: 8px; gap: 12px; }
.task-info { display: flex; align-items: center; gap: 10px; flex: 1; min-width: 0; }
.task-checkbox { width: 16px; height: 16px; cursor: pointer; accent-color: var(--primary-color); }
.task-text { font-size: 14px; color: var(--main-text); }
.completed-text { text-decoration: line-through; color: var(--main-text-secondary); }
.task-meta { display: flex; align-items: center; gap: 10px; flex-shrink: 0; }
.completed-date { font-size: 12px; color: #10b981; white-space: nowrap; }
.priority-badge { padding: 1px 8px; border-radius: 8px; font-size: 11px; font-weight: 600; }
.priority-badge.high { background: #fee2e2; color: #dc2626; }
.priority-badge.medium { background: #fef3c7; color: #d97706; }
.priority-badge.low { background: #dbeafe; color: #3b82f6; }
.undo-btn { padding: 4px 10px; border: 1px solid var(--border-color); border-radius: 6px; background: var(--card-bg); color: var(--main-text-secondary); cursor: pointer; font-size: 12px; transition: all 0.15s; }
.undo-btn:hover { border-color: var(--primary-color); color: var(--primary-color); }

.empty-tasks, .empty-git { text-align: center; padding: 40px 20px; color: var(--main-text-secondary); }
.empty-icon { font-size: 48px; display: block; margin-bottom: 12px; opacity: 0.5; }
.empty-tasks p, .empty-git p { font-size: 14px; margin: 0 0 16px; }
.edit-git-btn { padding: 8px 16px; border: 1px solid var(--primary-color); border-radius: 8px; background: transparent; color: var(--primary-color); cursor: pointer; font-size: 13px; transition: all 0.15s; }
.edit-git-btn:hover { background: var(--primary-color); color: white; }

.project-actions { display: flex; gap: 12px; justify-content: flex-start; margin-top: 16px; }
</style>
