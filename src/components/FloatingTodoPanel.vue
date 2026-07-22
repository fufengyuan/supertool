<template>
  <div class="floating-todo-panel" :data-theme="theme" :class="{ 'ball-mode': collapsed }">
    <!-- 小球模式：收起后只占一个球的大小 -->
    <div
      v-if="collapsed"
      class="ball"
      @mousedown="onBallMouseDown"
      title="点击展开，拖拽移动"
    >
      <SvgIcon name="check" size="26" :stroke-width="3" class="ball-icon" />
      <span v-if="pendingCount > 0" class="ball-badge">{{ pendingCount > 99 ? '99+' : pendingCount }}</span>
    </div>

    <!-- 展开模式 -->
    <template v-else>
      <div class="expanded-wrapper bg-base-100">
      <div
        class="title-bar flex items-center justify-between px-3 py-2 bg-base-200 border-b border-base-content/10 select-none rounded-t-lg"
        style="cursor: grab !important;"
        @mousedown="onTitleMouseDown"
      >
        <span class="text-xs font-semibold text-base-content tracking-wider flex items-center gap-1.5">
          <SvgIcon name="checkCircle" size="14" class="text-primary" />
          <span>待办</span>
          <span class="text-[10px] text-base-content/40">({{ pendingCount }})</span>
        </span>
        <button
          @click.stop="togglePin"
          class="btn btn-xs btn-ghost px-1"
          :class="{ 'text-primary': pinned }"
          :title="pinned ? '取消置顶' : '置顶'"
        ><SvgIcon name="mapPin" size="13" /></button>
      </div>

      <div class="content-area p-3 space-y-3">
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
        <div class="todo-list max-h-[350px] overflow-y-auto">
          <template v-for="group in groupedTodos" :key="group.projectId">
            <!-- 分组头部 -->
            <div class="flex items-center gap-2 px-1 py-1.5 mt-1.5 mb-0.5 border-b border-base-content/10 first:mt-0">
              <span
                v-if="group.project"
                class="flex items-center gap-1.5 text-[11px] font-semibold text-base-content/70 pl-2 border-l-[3px]"
                :style="group.project.color ? { borderLeftColor: group.project.color } : {}"
              >
                <span v-if="group.project.color" class="inline-block w-2 h-2 rounded-full" :style="{ backgroundColor: group.project.color }"></span>
                {{ group.project.name }}
                <span class="badge badge-ghost badge-xs ml-0.5">{{ group.todos.length }}</span>
              </span>
              <span v-else class="flex items-center gap-1.5 text-[11px] font-semibold text-base-content/50 pl-2 border-l-[3px] border-transparent">
                <span class="inline-block w-2 h-2 rounded-full bg-base-content/30"></span>
                无项目
                <span class="badge badge-ghost badge-xs ml-0.5">{{ group.todos.length }}</span>
              </span>
            </div>
            <!-- 分组内的任务 -->
            <div
              v-for="todo in group.todos"
              :key="todo.id"
              class="todo-item flex items-center gap-2 px-2 py-1.5 rounded-lg hover:bg-base-200/50 transition-colors group"
            >
              <button
                @click="toggleDone(todo)"
                class="w-4 h-4 rounded border border-base-content/30 flex items-center justify-center flex-shrink-0 hover:border-primary transition-colors"
                :class="{ 'bg-primary border-primary': todo.completed }"
              >
                <SvgIcon v-if="todo.completed" name="check" size="10" class="text-white" />
              </button>
              <span
                class="flex-1 text-xs text-base-content truncate"
                :class="{ 'line-through text-base-content/40': todo.completed }"
              >{{ todo.text }}</span>
              <button
                @click="deleteTodo(todo)"
                class="opacity-0 group-hover:opacity-100 text-base-content/30 hover:text-error transition-all px-1"
              ><SvgIcon name="x" size="11" /></button>
            </div>
          </template>
          <div v-if="groupedTodos.length === 0" class="text-[11px] text-base-content/40 text-center py-6">
            ✨ 暂无待办任务
          </div>
        </div>
      </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { getTauriAPI } from '../utils/tauri-api'
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
import SvgIcon from './ui/SvgIcon.vue'

interface Project {
  id: string
  name: string
  color?: string
}

interface Todo {
  id: string
  text: string
  completed: boolean
  priority?: string
  projectId?: string
}

interface TodoGroup {
  projectId: string | null
  project: Project | null
  todos: Todo[]
}

const EXPANDED_WIDTH = 340
const EXPANDED_HEIGHT = 500
const BALL_SIZE = 56
// 点击 vs 拖拽判定：mouseup 时若移动距离 < 此值且耗时 < 300ms，视为点击
const CLICK_THRESHOLD_PX = 5
const CLICK_THRESHOLD_MS = 300

const theme = ref('dark')
const collapsed = ref(false)
const pinned = ref(false)
const newTodoText = ref('')
const selectedProjectId = ref('')
const projects = ref<Project[]>([])
const todos = ref<Todo[]>([])

const pendingTodos = computed(() => todos.value.filter(t => !t.completed))
const pendingCount = computed(() => pendingTodos.value.length)

const groupedTodos = computed<TodoGroup[]>(() => {
  const active = pendingTodos.value
  if (active.length === 0) return []

  // Build project map
  const projectMap = new Map<string, Project>()
  for (const p of projects.value) {
    projectMap.set(p.id, p)
  }

  // Group by projectId
  const groupMap = new Map<string | null, Todo[]>()
  for (const todo of active) {
    const pid = todo.projectId || null
    if (!groupMap.has(pid)) groupMap.set(pid, [])
    groupMap.get(pid)!.push(todo)
  }

  // Build group array
  const groups: TodoGroup[] = []
  for (const [pid, todos] of groupMap) {
    groups.push({
      projectId: pid,
      project: pid ? projectMap.get(pid) || null : null,
      todos,
    })
  }

  // Sort: projects first (by order), no-project last
  const projectOrder = new Map<string, number>()
  projects.value.forEach((p, i) => projectOrder.set(p.id, i))
  groups.sort((a, b) => {
    if (a.projectId === null) return 1
    if (b.projectId === null) return -1
    return (projectOrder.get(a.projectId) ?? Infinity) - (projectOrder.get(b.projectId) ?? Infinity)
  })

  return groups
})

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

async function startDragging() {
  try {
    // Tauri 原生拖拽：无需窗口焦点即可跨应用拖动
    await getCurrentWindow().startDragging()
  } catch { /* non-Tauri env */ }
}

async function setWindowSize(w: number, h: number) {
  try {
    await getCurrentWindow().setSize(new LogicalSize(w, h))
  } catch { /* non-Tauri env */ }
}

async function setMinSize(w: number, h: number) {
  try {
    await getCurrentWindow().setMinSize(new LogicalSize(w, h))
  } catch { /* non-Tauri env */ }
}

// 展开模式：标题栏按下立即触发原生拖拽（无需焦点，跨应用可用）
// 鼠标抬起时若未拖动，则视为点击 → 折叠为小球
function onTitleMouseDown(e: MouseEvent) {
  if (e.button !== 0) return
  const target = e.target as HTMLElement
  // 置顶按钮有自己的 click 处理，不参与折叠/拖拽
  if (target.closest('button')) return

  const startX = e.clientX
  const startY = e.clientY
  const startTime = Date.now()
  startDragging()

  const onUp = (ev: MouseEvent) => {
    window.removeEventListener('mouseup', onUp)
    const dx = Math.abs(ev.clientX - startX)
    const dy = Math.abs(ev.clientY - startY)
    const dt = Date.now() - startTime
    // 移动距离很小且时间很短 → 视为点击 → 折叠
    if (dx < CLICK_THRESHOLD_PX && dy < CLICK_THRESHOLD_PX && dt < CLICK_THRESHOLD_MS) {
      collapsed.value = true
    }
  }
  window.addEventListener('mouseup', onUp)
}

// 小球模式：mousedown 立即触发原生拖拽（跨应用无需焦点）
// 同时记录起点，用全局 mouseup 判定是否为点击（没拖动就展开）
// 原理：startDragging 是原生级别，若用户只是点击没移动，鼠标抬起后 JS 的 mouseup 仍会触发
function onBallMouseDown(e: MouseEvent) {
  if (e.button !== 0) return
  const startX = e.clientX
  const startY = e.clientY
  const startTime = Date.now()

  // 立即启动原生拖拽 —— Tauri 的 startDragging 会在用户真正移动时拖动窗口
  // 若用户只是点击没移动，鼠标抬起后返回，不影响后续 mouseup 判定
  startDragging()

  const onUp = (ev: MouseEvent) => {
    window.removeEventListener('mouseup', onUp)
    const dx = Math.abs(ev.clientX - startX)
    const dy = Math.abs(ev.clientY - startY)
    const dt = Date.now() - startTime
    // 移动距离很小且时间很短 → 视为点击 → 展开
    if (dx < CLICK_THRESHOLD_PX && dy < CLICK_THRESHOLD_PX && dt < CLICK_THRESHOLD_MS) {
      collapsed.value = false
    }
  }
  window.addEventListener('mouseup', onUp)
}

// Watch collapsed state and resize window
watch(collapsed, async (val) => {
  if (val) {
    await setMinSize(BALL_SIZE, BALL_SIZE)
    await setWindowSize(BALL_SIZE, BALL_SIZE)
  } else {
    await setWindowSize(EXPANDED_WIDTH, EXPANDED_HEIGHT)
    await setMinSize(EXPANDED_WIDTH, EXPANDED_HEIGHT)
  }
})

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
  // 透明窗口：清除 body 默认背景，让圆角外区域显示桌面
  document.documentElement.style.background = 'transparent'
  document.body.style.background = 'transparent'
  setWindowSize(EXPANDED_WIDTH, EXPANDED_HEIGHT)
  setMinSize(EXPANDED_WIDTH, EXPANDED_HEIGHT)
  loadTodos()
  try {
    const raw = await getTauriAPI().getProjects?.(true) || []
    projects.value = raw as Project[]
  } catch { /* ignore */ }
  const htmlTheme = document.documentElement.getAttribute('data-theme')
  if (htmlTheme) theme.value = htmlTheme
  unlistenTodos = await getTauriAPI().onTodosChanged(() => loadTodos()).catch(() => null)
})

onUnmounted(() => {
  unlistenTodos?.()
})
</script>

<style scoped>
.floating-todo-panel {
  background: transparent;
}
.floating-todo-panel.ball-mode {
  background: transparent;
  overflow: visible;
  width: 56px;
  height: 56px;
}
.expanded-wrapper {
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 4px 24px rgba(0,0,0,0.15);
}

/* 悬浮球：深色背景 + 白色对勾，任何主题下都清晰可见 */
.ball {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  background: linear-gradient(135deg, #3b82f6, #1d4ed8);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: grab !important;
  position: relative;
  user-select: none;
  box-shadow: 0 1px 4px rgba(0,0,0,0.1);
  transition: box-shadow 0.12s ease;
}
.ball:hover {
  box-shadow: 0 2px 10px rgba(59,130,246,0.5);
}
.ball:active {
  cursor: grabbing !important;
  box-shadow: 0 1px 4px rgba(59,130,246,0.3);
}
.ball-icon {
  color: #ffffff;
  pointer-events: none;
}
/* 角标：右上角内嵌的小圆点，白底蓝字 */
.ball-badge {
  position: absolute;
  top: 2px;
  right: 2px;
  min-width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #ffffff;
  color: #1d4ed8;
  font-size: 10px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 3px;
  pointer-events: none;
}

.todo-list {
  scrollbar-width: thin;
}
.todo-item:active {
  transform: scale(0.98);
}
</style>
