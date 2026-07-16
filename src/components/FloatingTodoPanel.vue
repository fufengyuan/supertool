<template>
  <div class="floating-todo-panel min-h-[200px] bg-base-100" :data-theme="theme">
    <!-- Title bar: always visible -->
    <div
      class="title-bar flex items-center justify-between px-3 py-2 bg-base-200 border-b border-base-content/10 cursor-pointer select-none"
      :class="{ 'rounded-t-lg': !collapsed }"
      @click="collapsed = !collapsed"
    >
      <span class="text-xs font-semibold text-base-content tracking-wider flex items-center gap-1.5">
        <span>📋</span>
        <span>待办</span>
        <span class="text-[10px] text-base-content/40">({{ pendingCount }})</span>
      </span>
      <div class="flex items-center gap-1">
        <button
          @click.stop="togglePin"
          class="btn btn-xs btn-ghost px-1"
          :class="{ 'text-primary': pinned }"
          :title="pinned ? '取消置顶' : '置顶'"
        >📌</button>
        <button
          @click.stop="collapsed = !collapsed"
          class="btn btn-xs btn-ghost px-1 text-base-content/60"
        >{{ collapsed ? '▲' : '▼' }}</button>
      </div>
    </div>

    <!-- Content: hidden when collapsed -->
    <div v-show="!collapsed" class="content-area p-3 space-y-3">
      <!-- Quick add -->
      <div class="flex gap-1.5">
        <select
          v-model="selectedProjectId"
          class="select select-bordered select-xs w-24 bg-base-200 text-xs cursor-pointer"
        >
          <option value="">无项目</option>
          <option v-for="p in projects" :key="p.id" :value="p.id">{{ p.name }}</option>
        </select>
        <input
          v-model="newTodoText"
          @keydown.enter="addTodo"
          placeholder="快速添加待办，回车确认"
          class="input input-bordered input-xs flex-1 bg-base-200 text-xs"
        />
        <button @click="addTodo" class="btn btn-xs btn-primary" :disabled="!newTodoText.trim()">
          添加
        </button>
      </div>

      <!-- Todo list -->
      <div class="todo-list space-y-1 max-h-[350px] overflow-y-auto">
        <div
          v-for="todo in pendingTodos"
          :key="todo.id"
          class="todo-item flex items-center gap-2 px-2 py-1.5 rounded-lg hover:bg-base-200/50 transition-colors group"
        >
          <button
            @click="toggleDone(todo)"
            class="w-4 h-4 rounded border border-base-content/30 flex items-center justify-center flex-shrink-0 hover:border-primary transition-colors"
            :class="{ 'bg-primary border-primary': todo.completed }"
          >
            <span v-if="todo.completed" class="text-white text-[10px]">✓</span>
          </button>
          <span
            class="flex-1 text-xs text-base-content truncate"
            :class="{ 'line-through text-base-content/40': todo.completed }"
          >{{ todo.text }}</span>
          <span v-if="todo.projectId" class="text-[9px] text-base-content/30 mr-1">{{ getProjectName(todo.projectId) }}</span>
          <button
            @click="deleteTodo(todo)"
            class="opacity-0 group-hover:opacity-100 text-[10px] text-base-content/30 hover:text-error transition-all px-1"
          >✕</button>
        </div>
        <div v-if="pendingTodos.length === 0" class="text-[11px] text-base-content/40 text-center py-6">
          ✨ 暂无待办任务
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { getTauriAPI } from '../utils/tauri-api'

interface Project {
  id: string
  name: string
}

interface Todo {
  id: string
  text: string
  completed: boolean
  priority?: string
  projectId?: string
}

const theme = ref('dark')
const collapsed = ref(false)
const pinned = ref(false)
const newTodoText = ref('')
const selectedProjectId = ref('')
const projects = ref<Project[]>([])
const todos = ref<Todo[]>([])

const pendingTodos = computed(() => todos.value.filter(t => !t.completed))
const pendingCount = computed(() => pendingTodos.value.length)

function getProjectName(projectId: string): string {
  return projects.value.find(p => p.id === projectId)?.name || ''
}

async function loadTodos() {
  try {
    const raw = await getTauriAPI().getTodos()
    todos.value = (Array.isArray(raw) ? raw : (raw?.todos || raw?.data || [])).filter((t: any) => t && t.id)
  } catch {
    // ignore
  }
}

async function addTodo() {
  const text = newTodoText.value.trim()
  if (!text) return
  newTodoText.value = ''
  try {
    const todoData: any = { text, priority: 'medium' }
    if (selectedProjectId.value) {
      todoData.projectId = selectedProjectId.value
    }
    await getTauriAPI().addTodo(todoData)
    await loadTodos()
  } catch {
    // ignore
  }
}

async function toggleDone(todo: Todo) {
  try {
    await getTauriAPI().updateTodo?.({ id: todo.id, completed: !todo.completed } as any)
    todo.completed = !todo.completed
  } catch {
    // ignore
  }
}

async function deleteTodo(todo: Todo) {
  try {
    await getTauriAPI().deleteTodo?.(todo.id as any)
    todos.value = todos.value.filter(t => t.id !== todo.id)
  } catch {
    // ignore
  }
}

async function togglePin() {
  pinned.value = !pinned.value
  try {
    await getTauriAPI().setFloatingTodoPinned(pinned.value)
  } catch {
    pinned.value = !pinned.value
  }
}

let unlistenTodos: (() => void) | null = null

onMounted(async () => {
  loadTodos()
  // Load projects
  try {
    const raw = await getTauriAPI().getProjects?.(true) || []
    projects.value = raw as Project[]
  } catch { /* ignore */ }
  // Read theme from document
  const htmlTheme = document.documentElement.getAttribute('data-theme')
  if (htmlTheme) theme.value = htmlTheme
  // Listen for cross-window todo changes
  unlistenTodos = await getTauriAPI().onTodosChanged(() => loadTodos()).catch(() => null)
})

onUnmounted(() => {
  unlistenTodos?.()
})
</script>

<style scoped>
.floating-todo-panel {
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 4px 24px rgba(0,0,0,0.15);
}
.todo-list {
  scrollbar-width: thin;
}
.todo-item:active {
  transform: scale(0.98);
}
</style>
