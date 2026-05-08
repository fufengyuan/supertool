<template>
  <div class="p-5" v-if="project">
    <!-- 面包屑导航 -->
    <div class="flex items-center gap-2 mb-4 text-sm">
      <button class="inline-flex items-center gap-1 bg-none border-0 text-primary cursor-pointer text-sm font-medium p-1 rounded-md hover:bg-primary/10" @click="goBack" title="返回项目列表">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="15 18 9 12 15 6" />
        </svg>
        项目
      </button>
      <span class="text-base-content/70 text-lg">\u203A</span>
      <span class="text-base-content/70 font-medium">{{ project.name }}</span>
    </div>

    <!-- 项目头部 -->
    <div class="card bg-base-200 p-5 mb-4">
      <div class="flex items-start gap-3 mb-4">
        <div class="w-4 h-4 rounded-full flex-shrink-0 mt-1" :style="{ backgroundColor: project.color }"></div>
        <div>
          <h2 class="m-0 mb-2 text-2xl text-base-content">{{ project.name }}</h2>
          <p v-if="project.description" class="m-0 mb-2 text-sm text-base-content/70 leading-snug">{{ project.description }}</p>
          <div class="flex items-center gap-3">
            <span v-if="project.category" class="inline-block px-3 py-0.5 rounded-full text-xs font-medium bg-primary/10 text-primary">{{ categoryLabel(project.category) }}</span>
            <span class="text-xs text-base-content/70" v-if="project.createdAt">📅 {{ formatDate(project.createdAt) }}</span>
          </div>
        </div>
      </div>

      <div>
        <div class="flex gap-6 mb-3">
          <div class="flex flex-col items-center">
            <span class="text-xl font-bold text-base-content">{{ projectStats.total }}</span>
            <span class="text-xs text-base-content/70">总任务</span>
          </div>
          <div class="flex flex-col items-center">
            <span class="text-xl font-bold text-emerald-500">{{ projectStats.completed }}</span>
            <span class="text-xs text-base-content/70">已完成</span>
          </div>
          <div class="flex flex-col items-center">
            <span class="text-xl font-bold text-amber-500">{{ projectStats.total - projectStats.completed }}</span>
            <span class="text-xs text-base-content/70">进行中</span>
          </div>
          <div class="flex flex-col items-center">
            <span class="text-xl font-bold text-base-content">{{ projectStats.progress }}%</span>
            <span class="text-xs text-base-content/70">完成率</span>
          </div>
        </div>
        <div class="flex items-center gap-3">
          <div class="flex-1 h-3 bg-base-200 rounded-md overflow-hidden">
            <div class="h-full transition-all" :style="{ width: projectStats.progress + '%', backgroundColor: project.color }"></div>
          </div>
          <span class="text-sm font-semibold text-base-content whitespace-nowrap min-w-10 text-right">{{ projectStats.progress }}%</span>
        </div>
      </div>
    </div>

    <!-- 标签页切换 -->
    <div class="tabs tabs-boxed mb-4">
      <a class="tab" :class="activeTab === 'active' ? 'tab-active bg-base-100 shadow' : ''" @click="activeTab = 'active'">
        📋 进行中 ({{ activeTasks.length }})
      </a>
      <a class="tab" :class="activeTab === 'completed' ? 'tab-active bg-base-100 shadow' : ''" @click="activeTab = 'completed'">
        ✅ 已完成 ({{ completedTasks.length }})
      </a>
      <a class="tab" :class="activeTab === 'git' ? 'tab-active bg-base-100 shadow' : ''" @click="activeTab = 'git'">
        📜 Git 提交
      </a>
    </div>

    <!-- 进行中任务 -->
    <div v-if="activeTab === 'active'">
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
    <div v-if="activeTab === 'completed'">
      <div v-if="completedTasks.length === 0" class="text-center py-10 text-base-content/70">
        <span class="text-4xl block mb-3 opacity-50">🔍</span>
        <p class="text-sm m-0 mb-4">暂无已完成任务</p>
      </div>
      <div v-else class="flex flex-col gap-2">
        <div v-for="task in completedTasks" :key="task.id" class="flex items-center justify-between p-3 bg-base-200 rounded-lg gap-3">
          <div class="flex items-center gap-2.5 flex-1 min-w-0">
            <input type="checkbox" :checked="true" @change="handleToggleTask(task)" class="checkbox checkbox-sm checkbox-primary" />
            <span class="text-sm text-base-content/70 line-through">{{ task.text }}</span>
          </div>
          <div class="flex items-center gap-2.5 flex-shrink-0">
            <span v-if="task.completedAt" class="text-xs text-emerald-500 whitespace-nowrap">✅ {{ formatDate(task.completedAt) }}</span>
            <span v-if="task.priority" class="text-[11px] font-bold px-2 py-0.5 rounded-full"
              :class="task.priority === 'high' ? 'bg-red-100 text-red-600' : task.priority === 'medium' ? 'bg-amber-100 text-amber-600' : 'bg-blue-100 text-blue-600'">{{ priorityLabel(task.priority) }}</span>
            <button class="btn btn-xs btn-outline" @click="handleToggleTask(task)" title="恢复为未完成">↩️ 恢复</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Git 面板 -->
    <div v-if="activeTab === 'git'">
      <ProjectGitPanel v-if="hasGitRepos" :project="project" />
      <div v-else class="text-center py-10 px-5 text-base-content/70">
        <span class="text-4xl block mb-3 opacity-50">📜</span>
        <p class="text-sm m-0 mb-4">此项目未配置 Git 仓库</p>
        <button class="btn btn-outline btn-primary" @click="editProject">✏️ 编辑项目配置</button>
      </div>
    </div>

    <!-- 操作按钮 -->
    <div class="flex gap-3 justify-start mt-4">
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
import ProjectGitPanel from '@/components/project/ProjectGitPanel.vue'
import ProjectTodoList from '@/components/project/ProjectTodoList.vue'
import UiButton from '@/components/ui/Button.vue'
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
