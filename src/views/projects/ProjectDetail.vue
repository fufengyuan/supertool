<template>
  <div v-if="project" class="p-5">
    <!-- 面包屑导航 -->
    <div class="flex items-center gap-2 mb-4 text-sm">
      <button class="btn btn-ghost btn-xs gap-1 px-2 text-primary font-medium no-underline" @click="goBack" title="返回项目列表">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="15 18 9 12 15 6" />
        </svg>
        项目
      </button>
      <span class="text-base-content/60 text-lg leading-none">›</span>
      <span class="text-base-content/60 font-medium">{{ project.name }}</span>
    </div>

    <!-- 项目头部 -->
    <div class="bg-base-100 p-5 rounded-xl mb-4 shadow-sm">
      <div class="flex items-start gap-3 mb-4">
        <div class="w-4 h-4 rounded-full shrink-0 mt-1" :style="{ backgroundColor: project.color }"></div>
        <div class="min-w-0 flex-1">
          <h2 class="text-2xl font-bold text-base-content m-0 mb-2">{{ project.name }}</h2>
          <p v-if="project.description" class="text-sm text-base-content/60 leading-relaxed m-0 mb-2">{{ project.description }}</p>
          <div class="flex items-center gap-3">
            <span v-if="project.category" class="badge badge-ghost badge-sm">{{ categoryLabel(project.category) }}</span>
            <span v-if="project.createdAt" class="text-xs text-base-content/60"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="4" width="18" height="18" rx="2" ry="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/></svg>  {{ formatDate(project.createdAt) }}</span>
          </div>
        </div>
      </div>
      <div>
        <div class="flex gap-6 mb-3">
          <div class="flex flex-col items-center">
            <span class="text-2xl font-bold text-base-content">{{ projectStats.total }}</span>
            <span class="text-xs text-base-content/60">总任务</span>
          </div>
          <div class="flex flex-col items-center">
            <span class="text-2xl font-bold text-emerald-500">{{ projectStats.completed }}</span>
            <span class="text-xs text-base-content/60">已完成</span>
          </div>
          <div class="flex flex-col items-center">
            <span class="text-2xl font-bold text-amber-500">{{ projectStats.total - projectStats.completed }}</span>
            <span class="text-xs text-base-content/60">进行中</span>
          </div>
          <div class="flex flex-col items-center">
            <span class="text-2xl font-bold text-base-content">{{ projectStats.progress }}%</span>
            <span class="text-xs text-base-content/60">完成率</span>
          </div>
        </div>
        <div class="flex items-center gap-3">
          <div class="flex-1 h-3 bg-base-200 rounded-full overflow-hidden">
            <div class="h-full transition-all duration-300" :style="{ width: projectStats.progress + '%', backgroundColor: project.color }"></div>
          </div>
          <span class="text-sm font-semibold text-base-content whitespace-nowrap min-w-10 text-right">{{ projectStats.progress }}%</span>
        </div>
      </div>
    </div>

    <!-- 标签页切换 -->
    <div role="tablist" class="tabs tabs-boxed mb-4">
      <button role="tab" class="tab flex-1" :class="{ 'tab-active': activeTab === 'active' }" @click="activeTab = 'active'">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>  进行中 ({{ activeTasks.length }})
      </button>
      <button role="tab" class="tab flex-1" :class="{ 'tab-active': activeTab === 'completed' }" @click="activeTab = 'completed'">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>  已完成 ({{ completedTasks.length }})
      </button>
      <button role="tab" class="tab flex-1" :class="{ 'tab-active': activeTab === 'git' }" @click="activeTab = 'git'">
        📜 Git 提交
      </button>
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
      <div v-if="completedTasks.length === 0" class="text-center px-5 py-10 text-base-content/60">
        <span class="text-5xl block mb-3 opacity-50"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg> </span>
        <p class="text-sm m-0 mb-4">暂无已完成任务</p>
      </div>
      <div v-else class="flex flex-col gap-2">
        <div v-for="task in completedTasks" :key="task.id" class="flex items-center justify-between p-3 bg-base-200 rounded-lg gap-3">
          <div class="flex items-center gap-2.5 flex-1 min-w-0">
            <input type="checkbox" :checked="true" @change="handleToggleTask(task)" class="checkbox checkbox-primary checkbox-sm" />
            <span class="text-sm text-base-content/60 line-through">{{ task.text }}</span>
          </div>
          <div class="flex items-center gap-2.5 shrink-0">
            <span v-if="task.completedAt" class="text-xs text-emerald-500 whitespace-nowrap"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>  {{ formatDate(task.completedAt) }}</span>
            <span v-if="task.priority" class="badge badge-sm" :class="{ 'badge-error': task.priority === 'high', 'badge-warning': task.priority === 'medium', 'badge-info': task.priority === 'low' }">{{ priorityLabel(task.priority) }}</span>
            <button class="btn btn-ghost btn-xs" @click="handleToggleTask(task)" title="恢复为未完成"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><polyline points="1 4 1 10 7 10"/><path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10"/></svg> 恢复</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Git 面板 -->
    <div v-if="activeTab === 'git'">
      <ProjectGitPanel v-if="hasGitRepos" :project="project" />
      <div v-else class="text-center px-5 py-10 text-base-content/60">
        <span class="text-5xl block mb-3 opacity-50">📜</span>
        <p class="text-sm m-0 mb-4">此项目未配置 Git 仓库</p>
        <button class="btn btn-outline btn-primary btn-sm" @click="editProject"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>  编辑项目配置</button>
      </div>
    </div>

    <!-- 操作按钮 -->
    <div class="flex gap-3 mt-4">
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

    <!-- 编辑项目模态框 -->
    <UiModal
      v-model="showEditModal"
title="✏️ 编辑项目"
      @close="resetEditModal"
      width="640px"
    >
      <ProjectForm ref="projectFormRef" :project="project" @save="saveProject" />
      <template #footer>
        <UiButton variant="ghost" @click="resetEditModal">取消</UiButton>
        <UiButton variant="primary" @click="projectFormRef?.submit()">保存修改</UiButton>
      </template>
    </UiModal>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import ProjectGitPanel from '@/views/projects/ProjectGitPanel.vue'
import ProjectTodoList from '@/views/projects/ProjectTodoList.vue'
import ProjectForm from '@/views/projects/ProjectForm.vue'
import UiButton from '@/components/ui/Button.vue'
import UiModal from '@/components/ui/Modal.vue'
import { useTodoStore } from '../../stores/todoStore'
import { useProjectStore } from '../../stores/projectStore'
import { useProjects } from '../../composables/useProjects'
import { useErrorHandler } from '../../composables/useErrorHandler'
import type { Project } from '../../types'

const props = defineProps({
  id: { type: String, required: true }
})

const router = useRouter()
const todoStore = useTodoStore()
const projectStore = useProjectStore()
const projectsApi = useProjects()
const { handleError } = useErrorHandler()

const showEditModal = ref(false)
const projectFormRef = ref<InstanceType<typeof ProjectForm> | null>(null)

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

const goBack = () => { router.push('/projects') }
const editProject = () => { showEditModal.value = true }
const resetEditModal = () => {
  showEditModal.value = false
  projectFormRef.value?.reset()
}
const saveProject = async (formData: any) => {
  if (!project.value) return
  try {
    const updated = { ...project.value, ...formData, updatedAt: new Date().toISOString() }
    await projectsApi.updateProject(updated as unknown as Project)
    project.value = updated as unknown as Project
    resetEditModal()
  } catch (error) { handleError(error, { context: 'saveProject' }) }
}

const toggleArchive = async () => {
  if (!project.value) return
  try {
    console.log("[toggleArchive] called");
    const updated = { ...project.value, archived: !project.value.archived, updatedAt: new Date().toISOString() }
    await projectsApi.updateProject(updated as unknown as Project)
    project.value = updated as unknown as Project
  } catch (error) { handleError(error, { context: 'toggleArchive' }) }
}

const resolveProject = () => {
  const found = projectStore.projects.find(p => p.id === props.id)
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
