<template>
  <div v-if="project" class="flex flex-col h-full p-5 overflow-hidden">
    <!-- 面包屑导航 -->
    <div class="flex items-center gap-2 mb-4 text-sm shrink-0">
      <button class="btn btn-ghost btn-xs gap-1 px-2 text-primary font-medium no-underline" @click="goBack" title="返回项目列表">
        <SvgIcon name="chevronLeft" :size="16" />
        项目
      </button>
      <span class="text-base-content/60 text-lg leading-none">›</span>
      <span class="text-base-content/60 font-medium">{{ project.name }}</span>
    </div>

    <!-- 项目头部 -->
    <div class="bg-base-100 p-5 rounded-xl mb-4 shadow-sm shrink-0">
      <div class="flex items-start gap-3 mb-4">
        <div class="w-4 h-4 rounded-full shrink-0 mt-1" :style="{ backgroundColor: project.color }"></div>
        <div class="min-w-0 flex-1">
          <h2 class="text-2xl font-bold text-base-content m-0 mb-2">{{ project.name }}</h2>
          <p v-if="project.description" class="text-sm text-base-content/60 leading-relaxed m-0 mb-2">{{ project.description }}</p>
          <div class="flex items-center gap-3">
            <span v-if="project.category" class="badge badge-ghost badge-sm">{{ categoryLabel(project.category) }}</span>
            <span v-if="project.createdAt" class="text-xs text-base-content/60"><SvgIcon name="calendar" :size="14" />  {{ formatDate(project.createdAt) }}</span>
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
    <div role="tablist" class="tabs tabs-boxed mb-4 shrink-0">
      <button role="tab" class="tab flex-1" :class="{ 'tab-active': activeTab === 'active' }" @click="activeTab = 'active'">
        <SvgIcon name="file" :size="14" />  进行中 ({{ activeTasks.length }})
      </button>
      <button role="tab" class="tab flex-1" :class="{ 'tab-active': activeTab === 'completed' }" @click="activeTab = 'completed'">
        <SvgIcon name="check" :size="14" />  已完成 ({{ completedTasks.length }})
      </button>
      <button role="tab" class="tab flex-1" :class="{ 'tab-active': activeTab === 'git' }" @click="activeTab = 'git'">
        📜 Git 提交
      </button>
    </div>

    <!-- 进行中任务 -->
    <div v-if="activeTab === 'active'" class="flex-1 min-h-0 overflow-y-auto">
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
    <div v-if="activeTab === 'completed'" class="flex-1 min-h-0 overflow-y-auto">
      <div v-if="completedTasks.length === 0" class="text-center px-5 py-10 text-base-content/60">
        <span class="text-5xl block mb-3 opacity-50"><SvgIcon name="search" :size="14" /> </span>
        <p class="text-sm m-0 mb-4">暂无已完成任务</p>
      </div>
      <div v-else class="flex flex-col gap-2">
        <div v-for="task in completedTasks" :key="task.id" class="flex items-center justify-between p-3 bg-base-200 rounded-lg gap-3">
          <div class="flex items-center gap-2.5 flex-1 min-w-0">
            <input type="checkbox" :checked="true" @change="handleToggleTask(task)" class="checkbox checkbox-primary checkbox-sm" />
            <span class="text-sm text-base-content/60 line-through">{{ task.text }}</span>
          </div>
          <div class="flex items-center gap-2.5 shrink-0">
            <span v-if="task.completedAt" class="text-xs text-emerald-500 whitespace-nowrap"><SvgIcon name="check" :size="14" />  {{ formatDate(task.completedAt) }}</span>
            <span v-if="task.priority" class="badge badge-sm" :class="{ 'badge-error': task.priority === 'high', 'badge-warning': task.priority === 'medium', 'badge-info': task.priority === 'low' }">{{ priorityLabel(task.priority) }}</span>
            <button class="btn btn-ghost btn-xs" @click="handleToggleTask(task)" title="恢复为未完成"><SvgIcon name="undo" :size="14" class="inline-block align-text-bottom" /> 恢复</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Git 面板 -->
    <div v-if="activeTab === 'git'" class="flex-1 min-h-0 overflow-hidden">
      <ProjectGitPanel v-if="hasGitRepos" :project="project" class="h-full" />
      <div v-else class="text-center px-5 py-10 text-base-content/60">
        <span class="text-5xl block mb-3 opacity-50">📜</span>
        <p class="text-sm m-0 mb-4">此项目未配置 Git 仓库</p>
        <button class="btn btn-outline btn-primary btn-sm" @click="editProject"><SvgIcon name="pencil" :size="14" />  编辑项目配置</button>
      </div>
    </div>

    <!-- 操作按钮 -->
    <div class="flex gap-3 mt-4">
      <UiButton variant="ghost" @click="goBack">
        <SvgIcon name="chevronLeft" :size="16" />
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
defineOptions({ name: 'ProjectDetail' })
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import SvgIcon from '@/components/ui/SvgIcon.vue'
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
  if (!project.value) {return}
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
  if (!project.value) {return}
  try {
    const updated = { ...project.value, ...formData, updatedAt: new Date().toISOString() }
    await projectsApi.updateProject(updated as unknown as Project)
    project.value = updated as unknown as Project
    resetEditModal()
  } catch (error) { handleError(error, { context: 'saveProject' }) }
}

const toggleArchive = async () => {
  if (!project.value) {return}
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
  if (!project.value) {resolveProject()}
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
