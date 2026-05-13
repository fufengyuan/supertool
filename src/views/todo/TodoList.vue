<template>
  <div class="flex flex-col gap-3 p-5 w-full outline-none" ref="containerRef" tabindex="0" @keydown="handleKeyboardNav">
    <!-- 快速输入框 -->
    <div class="flex items-center gap-2.5 px-4 py-3 bg-base-100 border-2 border-primary rounded-xl shadow-sm flex-shrink-0 transition-all duration-200 focus-within:border-primary/80 focus-within:shadow-[0_4px_20px_rgba(136,57,239,0.2)]">
      <SvgIcon name="plus" size="18" class="text-primary shrink-0 opacity-70" />
      <input
        ref="quickAddInput"
        v-model="quickAddText"
        class="flex-1 bg-transparent border-none outline-none text-[15px] text-base-content font-medium placeholder:text-base-content/40"
        placeholder="添加新任务，回车即可保存…"
        @keyup.enter="handleQuickAdd"
        @keydown.escape="quickAddText = ''"
      />
      <button v-if="quickAddText" class="btn btn-ghost btn-xs gap-1" @click="cycleQuickPriority" :title="'优先级: ' + quickPriorityLabel">
        <span :class="`w-2 h-2 rounded-full ${quickAddPriority === 'low' ? 'bg-success' : quickAddPriority === 'medium' ? 'bg-warning' : 'bg-error'}`"></span>
        {{ quickPriorityLabel }}
      </button>
      <select v-if="quickAddText" v-model="quickAddProjectId" class="select select-ghost w-auto max-w-[180px]">
        <option value="">无项目</option>
        <option v-for="p in projectStore.projects" :key="p.id" :value="p.id">{{ p.name }}</option>
      </select>
    </div>

    <!-- 顶部工具栏：搜索 + 排序 + 添加按钮 -->
    <div class="flex items-center justify-between gap-3 py-1 pb-2 flex-shrink-0">
      <div class="flex items-center gap-2 flex-1 min-w-0">
        <input
          v-model="searchQueryValue"
          placeholder="搜索任务…"
          class="input flex-1 min-w-[180px] max-w-[520px] text-[13px] ps-8 bg-[length:14px]"
          style="background-image: url('data:image/svg+xml,%3Csvg xmlns=\\'http://www.w3.org/2000/svg\\' width=\\'14\\' height=\\'14\\' viewBox=\\'0 0 24 24\\' fill=\\'none\\' stroke=\\'%239ca3af\\' stroke-width=\\'2\\' stroke-linecap=\\'round\\'%3E%3Ccircle cx=\\'11\\' cy=\\'11\\' r=\\'8\\'/%3E%3Cline x1=\\'21\\' y1=\\'21\\' x2=\\'16.65\\' y2=\\'16.65\\'/%3E%3C/svg%3E'); background-position: 10px center; background-repeat: no-repeat;"
          @keydown.escape="searchQueryValue = ''"
        />
        <select v-model="filterValue" class="select">
          <option value="all">全部</option>
          <option value="active">进行中</option>
          <option value="completed">已完成</option>
        </select>
        <select v-model="sortValue" class="select">
          <option value="">默认排序</option>
          <option value="priority">按优先级</option>
          <option value="dueDate">按截止日期</option>
          <option value="createdAt">按创建时间</option>
        </select>
      </div>
      <div class="flex items-center gap-3 shrink-0">
        <span class="badge badge-ghost text-[13px] font-medium">{{ todoStore.activeCount }} 进行中</span>
        <button class="btn btn-outline btn-sm gap-1.5" @click="showAddModal = true" title="添加任务 (Ctrl+N)">
          <SvgIcon name="plus" size="14" :stroke-width="2.5" />
          更多
        </button>
      </div>
    </div>

    <!-- 今日进度 -->
    <div class="flex flex-col gap-1.5" v-if="todoStore.todos.length > 0">
      <div class="flex justify-between items-center">
        <span class="text-sm font-medium text-base-content/70">今日进度</span>
        <span class="text-xs text-base-content/50">{{ todoStore.completedCount }}/{{ todoStore.todos.length }}</span>
      </div>
      <div class="h-2 bg-base-200 rounded-full overflow-hidden">
        <div class="h-full bg-primary rounded-full transition-all duration-300 ease-out" :style="{ width: progressPercent + '%' }"></div>
      </div>
    </div>

    <!-- 键盘导航提示 -->
    <div class="flex items-center gap-1.5 px-2.5 py-1.5 bg-base-200/60 rounded-lg text-xs text-base-content/60" v-if="keyboardFocusedIndex >= 0">
      <kbd class="kbd kbd-xs">↑↓</kbd> 浏览 <kbd class="kbd kbd-xs">Enter</kbd> 展开 <kbd class="kbd kbd-xs">Space</kbd> 完成
      <kbd class="kbd kbd-xs">Del</kbd> 删除 <kbd class="kbd kbd-xs">1</kbd><kbd class="kbd kbd-xs">2</kbd><kbd class="kbd kbd-xs">3</kbd> 优先级
    </div>

    <!-- 任务卡片容器 -->
    <div>
      <div>
        <!-- 活跃任务列表（按项目分组） -->
        <template v-for="group in groupedActiveTodos" :key="group.projectId">
          <!-- 项目分组头部 -->
          <div class="flex items-center gap-2 px-1 py-1.5 mt-1 mb-0.5 border-b border-base-content/10 first:mt-0">
            <span
              v-if="group.project"
              class="flex items-center gap-1.5 text-sm font-semibold text-base-content/80 pl-2 border-l-[3px] border-transparent"
              :style="group.project.color ? { borderLeftColor: group.project.color } : {}"
            >
              <span v-if="group.project.color" class="inline-block w-2 h-2 rounded-full" :style="{ backgroundColor: group.project.color }"></span>
              {{ group.project.name }}
              <span class="badge badge-ghost badge-xs ml-0.5">{{ group.todos.length }}</span>
            </span>
            <span v-else class="flex items-center gap-1.5 text-sm font-semibold text-base-content/50 pl-2 border-l-[3px] border-transparent">
              <span class="inline-block w-2 h-2 rounded-full bg-base-content/30"></span>
              无项目
              <span class="badge badge-ghost badge-xs ml-0.5">{{ group.todos.length }}</span>
            </span>
          </div>
          <!-- 分组内的任务列表 -->
          <ul class="list-none p-0 m-0 space-y-0.5">
            <TodoItem
              v-for="todo in group.todos"
              :key="todo.id"
              :todo="todo"
              :search-query="todoStore.searchQuery"
              :is-selected="selectedTodoIds.includes(todo.id)"
              :expanded="expandedTodo === todo.id"
              :editing-id="editingId" v-model:edit-text="editText"
              :tags="todoStore.tags" :projects="(projectStore.projects as any[])"
              :is-markdown-editing="isMarkdownEditing"
              :editing-markdown-id="editingMarkdownId"
              v-model:editing-markdown-content="editingMarkdownContent"
              :collaborating-user="collab.collaboratingUsers[todo.id] || ''"
              :comments="collab.taskComments[todo.id] || []"
              :comment-input="collab.commentInputs[todo.id] || ''"
              :highlighted="highlightTodoId === todo.id"
              :data-todo-id="todo.id"
              :class="{ 'keyboard-focused': keyboardFocusedIndex >= 0 && flatActiveTodos[keyboardFocusedIndex]?.id === todo.id }"
              @update:comment-input="(val) => collab.commentInputs[todo.id] = val"
              @toggle="todoStore.toggleTodo" @delete="deleteTodo"
              @toggle-selected="batch.toggleSelected" @toggle-expand="toggleExpand"
              @start-edit="startEdit" @save-edit="saveEdit" @cancel-edit="cancelEdit"
              @update-tag="updateTodoTag" @add-new-tag="addNewTag"
              @start-markdown-edit="startMarkdownEdit"
              @handle-markdown-double-click="(t) => collab.handleMarkdownDoubleClick(t, startMarkdownEdit)"
              @save-markdown="saveMarkdown" @cancel-markdown="cancelMarkdownEdit"
              @add-comment="collab.addComment" @subtask-completed="handleSubtaskCompletion"
            />
          </ul>
        </template>

        <!-- 已完成折叠区（按项目分组） -->
        <div v-if="groupedCompletedTodos.length > 0 && filterValue === 'all' && !searchQueryValue" class="mt-4">
          <button class="btn btn-ghost btn-sm gap-1.5 text-base-content/60 w-full justify-start" @click="showCompleted = !showCompleted">
            <SvgIcon name="chevronDown" size="14" :class="{ 'rotate-180': showCompleted }" class="transition-transform duration-200" />
            已完成 ({{ completedTodos.length }})
          </button>
          <Transition name="slide">
            <div v-if="showCompleted" class="overflow-hidden">
              <template v-for="group in groupedCompletedTodos" :key="'completed-' + group.projectId">
                <div v-if="group.project" class="flex items-center gap-2 px-1 py-1.5 mt-2 mb-0.5 border-b border-base-content/5">
                  <span class="flex items-center gap-1.5 text-sm font-medium text-base-content/60 pl-2 border-l-[3px] border-transparent" :style="group.project.color ? { borderLeftColor: group.project.color } : {}">
                    <span v-if="group.project.color" class="inline-block w-2 h-2 rounded-full" :style="{ backgroundColor: group.project.color }"></span>
                    {{ group.project.name }}
                    <span class="badge badge-ghost badge-xs ml-0.5">{{ group.todos.length }}</span>
                  </span>
                </div>
                <div v-else class="flex items-center gap-2 px-1 py-1.5 mt-2 mb-0.5 border-b border-base-content/5">
                  <span class="flex items-center gap-1.5 text-sm font-medium text-base-content/40 pl-2 border-l-[3px] border-transparent">
                    <span class="inline-block w-2 h-2 rounded-full bg-base-content/20"></span>
                    无项目
                    <span class="badge badge-ghost badge-xs ml-0.5">{{ group.todos.length }}</span>
                  </span>
                </div>
                <ul class="list-none p-0 m-0 space-y-0.5">
                  <TodoItem
                    v-for="todo in group.todos"
                    :key="todo.id"
                    :todo="todo"
                    :search-query="todoStore.searchQuery"
                    :is-selected="selectedTodoIds.includes(todo.id)"
                    :expanded="expandedTodo === todo.id"
                    :editing-id="editingId" v-model:edit-text="editText"
                    :tags="todoStore.tags" :projects="(projectStore.projects as any[])"
                    :is-markdown-editing="isMarkdownEditing"
                    :editing-markdown-id="editingMarkdownId"
                    v-model:editing-markdown-content="editingMarkdownContent"
                    :collaborating-user="collab.collaboratingUsers[todo.id] || ''"
                    :comments="collab.taskComments[todo.id] || []"
                    :comment-input="collab.commentInputs[todo.id] || ''"
                    :highlighted="highlightTodoId === todo.id"
                    @toggle="todoStore.toggleTodo" @delete="deleteTodo"
                    @toggle-selected="batch.toggleSelected" @toggle-expand="toggleExpand"
                    @start-edit="startEdit" @save-edit="saveEdit" @cancel-edit="cancelEdit"
                    @update-tag="updateTodoTag" @add-new-tag="addNewTag"
                    @start-markdown-edit="startMarkdownEdit"
                    @save-markdown="saveMarkdown" @cancel-markdown="cancelMarkdownEdit"
                    @add-comment="collab.addComment" @subtask-completed="handleSubtaskCompletion"
                  />
                </ul>
              </template>
            </div>
          </Transition>
        </div>
      </div>

      <!-- 空状态提示 -->
      <div v-if="activeTodos.length === 0 && completedTodos.length === 0 && todoStore.todos.length === 0 && !searchQueryValue" class="flex flex-col items-center justify-center py-16 text-base-content/40">
        <SvgIcon name="check" size="64" class="mb-4 opacity-30" stroke-width="1" />
        <p class="text-lg font-medium text-base-content/60">还没有任务</p>
        <p class="text-sm text-base-content/40 mt-1">在上方输入框添加你的第一个任务</p>
      </div>
      <div v-else-if="activeTodos.length === 0 && completedTodos.length === 0" class="flex flex-col items-center justify-center py-12 text-base-content/40">
        <p class="text-sm">没有找到匹配的任务，尝试调整筛选条件</p>
      </div>
    </div>

    <!-- 批量操作和清空已完成 -->
    <div class="flex items-center justify-between gap-3 mt-1" v-if="selectedTodoIds.length > 0 || (todoStore.completedCount > 0 && filterValue === 'all')">
      <div v-if="selectedTodoIds.length > 0" class="flex items-center gap-2">
        <button @click="batch.batchComplete" class="btn btn-success btn-sm">完成 ({{ selectedTodoIds.length }})</button>
        <button @click="batch.batchDelete" class="btn btn-error btn-sm">删除 ({{ selectedTodoIds.length }})</button>
      </div>
      <div v-if="todoStore.completedCount > 0 && filterValue === 'all'" class="ml-auto">
        <button @click="todoStore.clearCompleted" class="btn btn-ghost btn-sm text-base-content/50">清空已完成</button>
      </div>
    </div>

    <!-- 添加任务弹出框 -->
    <div v-if="showAddModal" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" @click.self="showAddModal = false">
      <div class="bg-base-100 rounded-2xl shadow-2xl w-full max-w-lg mx-4 overflow-hidden">
        <div class="flex items-center justify-between px-6 pt-5 pb-3">
          <h3 class="text-lg font-semibold text-base-content">添加任务</h3>
          <button class="btn btn-circle btn-ghost btn-sm" @click="showAddModal = false"><SvgIcon name="x" size="14" /></button>
        </div>
        <div class="px-6 space-y-4">
          <div class="flex flex-col gap-1.5">
            <label class="text-xs font-medium text-base-content/60">任务内容</label>
            <input
              ref="addTaskInput"
              v-model="newTaskText"
              @keyup.enter="handleAddFromModal"
              placeholder="输入任务内容…"
              class="input input-bordered w-full"
              autofocus
            />
          </div>
          <div class="flex gap-4 flex-wrap">
            <div class="flex flex-col gap-1.5 flex-1 min-w-[120px]">
              <label class="text-xs font-medium text-base-content/60">优先级</label>
              <select v-model="newTaskPriority" class="select select-bordered w-full">
                <option value="low">低</option>
                <option value="medium">中</option>
                <option value="high">高</option>
              </select>
            </div>
            <div class="flex flex-col gap-1.5 flex-1 min-w-[120px]">
              <label class="text-xs font-medium text-base-content/60">截止日期</label>
              <input v-model="newTaskDueDate" type="date" class="input input-bordered w-full" />
            </div>
            <div class="flex flex-col gap-1.5 flex-1 min-w-[120px]">
              <label class="text-xs font-medium text-base-content/60">标签</label>
              <select v-model="newTaskTag" class="select select-bordered w-full">
                <option value="">无</option>
                <option v-for="tag in todoStore.tags" :key="tag" :value="tag">{{ tag }}</option>
                <option value="__custom__">自定义</option>
              </select>
            </div>
          </div>
          <div class="flex flex-col gap-1.5" v-if="newTaskTag === '__custom__'">
            <label class="text-xs font-medium text-base-content/60">自定义标签</label>
            <input v-model="newTaskCustomTag" placeholder="输入标签名称" class="input input-bordered w-full" />
          </div>
          <div class="flex gap-4 flex-wrap">
            <div class="flex flex-col gap-1.5 flex-1 min-w-[120px]">
              <label class="text-xs font-medium text-base-content/60">项目</label>
              <select v-model="newTaskProjectId" class="select select-bordered w-full">
                <option value="">无</option>
                <option v-for="p in projectStore.projects" :key="p.id" :value="p.id">{{ p.name }}</option>
              </select>
            </div>
            <div class="flex flex-col gap-1.5 flex-1 min-w-[120px]">
              <label class="text-xs font-medium text-base-content/60">重复</label>
              <select v-model="newTaskRepeat" class="select select-bordered w-full">
                <option value="">不重复</option>
                <option value="daily">每天</option>
                <option value="weekly">每周</option>
                <option value="monthly">每月</option>
              </select>
            </div>
          </div>
          <div class="flex flex-col gap-1.5">
            <label class="text-xs font-medium text-base-content/60">描述</label>
            <textarea v-model="newTaskDesc" placeholder="任务描述（可选）" class="textarea textarea-bordered w-full" rows="2"></textarea>
          </div>
        </div>
        <div class="flex justify-end gap-2 px-6 py-4 mt-2 border-t border-base-content/10">
          <button class="btn btn-ghost" @click="showAddModal = false">取消</button>
          <button class="btn btn-primary" @click="handleAddFromModal">添加任务</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { getTauriAPI } from '../../utils/tauri-api'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import TodoItem from './TodoItem.vue'
import draggable from 'vuedraggable'
import { useTodoStore } from '../../stores/todoStore'
import { useProjectStore } from '../../stores/projectStore'
import { useTodos } from '../../composables/useTodos'
import { useTodoFilters } from '../../composables/useTodoFilters'
import { useTodoCollaboration } from '../../composables/useTodoCollaboration'
import { useTodoBatch } from '../../composables/useTodoBatch'
import { usePerformance } from '../../composables/usePerformance'
import { useToast } from '../../composables/useToast'
import { useErrorHandler } from '../../composables/useErrorHandler'

const todoStore = useTodoStore()
const projectStore = useProjectStore()
const todosApi = useTodos()
const perf = usePerformance()
const toast = useToast()
const { handleError } = useErrorHandler()

// ===== 快速添加 =====
const quickAddInput = ref<HTMLInputElement | null>(null)
const quickAddText = ref('')
const quickAddPriority = ref<'low' | 'medium' | 'high'>('medium')
const quickAddProjectId = ref('')
const quickPriorityLabel = computed(() => {
  const map: Record<string, string> = { low: '低', medium: '中', high: '高' }
  return map[quickAddPriority.value]
})

const cycleQuickPriority = () => {
  const order: Array<'low' | 'medium' | 'high'> = ['low', 'medium', 'high']
  const idx = order.indexOf(quickAddPriority.value)
  quickAddPriority.value = order[(idx + 1) % 3]
}

const handleQuickAdd = async () => {
  const text = quickAddText.value.trim()
  if (!text) return
  const newTodo = {
    id: crypto.randomUUID(),
    text,
    completed: false,
    priority: quickAddPriority.value,
    dueDate: null,
    description: '',
    markdownDescription: '',
    tag: '未分类',
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    repeatType: null,
    repeatInterval: 1,
    repeatEndDate: null,
    repeatCount: -1,
    parentTodoId: null,
    projectId: quickAddProjectId.value || null,
  }
  try {
    await todoStore.addTodo(newTodo)
    quickAddText.value = ''
    quickAddPriority.value = 'medium'
    // Don't reset quickAddProjectId so user can keep adding to same project
  } catch (error) { handleError(error, { context: '快速添加任务', showToast: true }) }
}

// ===== 活跃/已完成任务分离 =====
const activeTodos = computed(() => todoStore.filteredTodos.filter(t => !t.completed))
const completedTodos = computed(() => todoStore.filteredTodos.filter(t => t.completed))
const showCompleted = ref(false)

// ===== 项目分组 =====
// 展平的活动任务列表（用于键盘导航）
const flatActiveTodos = computed(() => activeTodos.value)

interface TodoGroup {
  projectId: string | null
  project: (typeof projectStore.projects)[number] | null
  todos: typeof activeTodos.value
}

// 按项目分组的活动任务
const groupedActiveTodos = computed<TodoGroup[]>(() => {
  const todos = activeTodos.value
  if (todos.length === 0) return []

  // 构建项目映射
  const projectMap = new Map<string, (typeof projectStore.projects)[number]>()
  for (const p of projectStore.projects) {
    projectMap.set(p.id, p)
  }

  // 按 projectId 分组
  const groupMap = new Map<string | null, typeof todos>()
  for (const todo of todos) {
    const pid = todo.projectId || null
    if (!groupMap.has(pid)) {
      groupMap.set(pid, [])
    }
    groupMap.get(pid)!.push(todo)
  }

  // 构建组数组
  const groups: TodoGroup[] = []
  for (const [pid, todos] of groupMap) {
    groups.push({
      projectId: pid,
      project: pid ? projectMap.get(pid) || null : null,
      todos,
    })
  }

  // 排序：有项目的在前（按项目顺序），无项目的在最后
  const projectOrder = new Map<string, number>()
  projectStore.projects.forEach((p, i) => projectOrder.set(p.id, i))

  groups.sort((a, b) => {
    // 无项目的排最后
    if (a.projectId === null) return 1
    if (b.projectId === null) return -1
    // 有项目的按项目顺序排
    const aOrder = projectOrder.get(a.projectId) ?? Infinity
    const bOrder = projectOrder.get(b.projectId) ?? Infinity
    return aOrder - bOrder
  })

  return groups
})

// 按项目分组的已完成任务
const groupedCompletedTodos = computed<TodoGroup[]>(() => {
  const todos = completedTodos.value
  if (todos.length === 0) return []

  // 构建项目映射
  const projectMap = new Map<string, (typeof projectStore.projects)[number]>()
  for (const p of projectStore.projects) {
    projectMap.set(p.id, p)
  }

  // 按 projectId 分组
  const groupMap = new Map<string | null, typeof todos>()
  for (const todo of todos) {
    const pid = todo.projectId || null
    if (!groupMap.has(pid)) {
      groupMap.set(pid, [])
    }
    groupMap.get(pid)!.push(todo)
  }

  // 构建组数组
  const groups: TodoGroup[] = []
  for (const [pid, todos] of groupMap) {
    groups.push({
      projectId: pid,
      project: pid ? projectMap.get(pid) || null : null,
      todos,
    })
  }

  // 排序：有项目的在前（按项目顺序），无项目的在最后
  const projectOrder = new Map<string, number>()
  projectStore.projects.forEach((p, i) => projectOrder.set(p.id, i))

  groups.sort((a, b) => {
    if (a.projectId === null) return 1
    if (b.projectId === null) return -1
    const aOrder = projectOrder.get(a.projectId) ?? Infinity
    const bOrder = projectOrder.get(b.projectId) ?? Infinity
    return aOrder - bOrder
  })

  return groups
})

// 虚拟滚动：分组后禁用（简化处理）
const useVirtualScroll = computed(() => false)
const virtualListRef = ref(null)
const containerRef = ref<HTMLElement | null>(null)

// 同步虚拟滚动状态到性能监控
watch(useVirtualScroll, (enabled) => {
  perf.setVirtualListEnabled(enabled)
}, { immediate: true })

// ===== Composables =====
const filters = useTodoFilters(todoStore as any)
const collab = useTodoCollaboration(todosApi)
const batch = useTodoBatch(todoStore as any, todosApi)

// Unwrap filter ComputedRefs for v-model compatibility (cast to bypass readonly)
const filterValue = computed({ get: () => filters.filterProxy.value, set: (v: string) => { (filters.filterProxy as any).value = v } })
const tagFilterValue = computed({ get: () => filters.tagFilterProxy.value, set: (v: string) => { (filters.tagFilterProxy as any).value = v } })
const searchQueryValue = computed({ get: () => filters.searchQueryProxy.value, set: (v: string) => { (filters.searchQueryProxy as any).value = v } })
const priorityFilterValue = computed({ get: () => filters.priorityFilterProxy.value, set: (v: string) => { (filters.priorityFilterProxy as any).value = v } })
const statusFilterValue = computed({ get: () => filters.statusFilterProxy.value, set: (v: string) => { (filters.statusFilterProxy as any).value = v } })

// ===== 排序 =====
const sortValue = computed({
  get: () => todoStore.sortBy || '',
  set: (val) => todoStore.setSortBy(val || null)
})

// ===== 今日进度 =====
const progressPercent = computed(() => {
  const total = todoStore.todos.length
  if (total === 0) return 0
  return Math.round((todoStore.completedCount / total) * 100)
})

// ===== 本地 UI 状态 =====
const expandedTodo = ref<string | null>(null)
const editingId = ref<string | null>(null)
const editText = ref('')
const editingMarkdownId = ref<string | null>(null)
const editingMarkdownContent = ref('')
const isMarkdownEditing = ref(false)
const newTagName = ref('')

// Unwrap batch.selectedTodos ref for template compatibility
const selectedTodoIds = computed(() => batch.selectedTodos.value)

// ===== 虚拟滚动数据 =====
const activeTodosForVirtual = computed(() => activeTodos.value)

// ===== 拖拽数据 =====
const activeTodosForDrag = computed({
  get: () => [...activeTodos.value],
  set: (value) => {
    value.forEach((todo, index) => {
      const original = todoStore.todos.find(t => t.id === todo.id)
      if (original) {
        original.orderNum = index
        original.updatedAt = new Date().toISOString()
      }
    })
  }
})

// ===== 添加任务弹窗 =====
const showAddModal = ref(false)
const addTaskInput = ref<HTMLInputElement | null>(null)
const newTaskText = ref('')
const newTaskPriority = ref('medium')
const newTaskDueDate = ref('')
const newTaskTag = ref('')
const newTaskCustomTag = ref('')
const newTaskProjectId = ref('')
const newTaskRepeat = ref('')
const newTaskDesc = ref('')

watch(showAddModal, (val) => {
  if (val) {
    nextTick(() => addTaskInput.value?.focus())
  }
})

const resetModal = () => {
  newTaskText.value = ''
  newTaskPriority.value = 'medium'
  newTaskDueDate.value = ''
  newTaskTag.value = ''
  newTaskCustomTag.value = ''
  newTaskProjectId.value = ''
  newTaskRepeat.value = ''
  newTaskDesc.value = ''
}

const handleAddFromModal = () => {
  const text = newTaskText.value.trim()
  if (!text) return
  const tag = newTaskTag.value === '__custom__' ? newTaskCustomTag.value.trim() : newTaskTag.value
  const newTodoObj = {
    id: crypto.randomUUID(),
    text,
    completed: false,
    priority: newTaskPriority.value,
    dueDate: newTaskDueDate.value || null,
    description: newTaskDesc.value || '',
    markdownDescription: '',
    tag: tag || '未分类',
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    repeatType: newTaskRepeat.value,
    repeatInterval: newTaskRepeat.value === 'custom' ? 1 : 1,
    repeatEndDate: null,
    repeatCount: -1,
    parentTodoId: null,
    projectId: newTaskProjectId.value || null,
  }
  handleAddTodo(newTodoObj)
  showAddModal.value = false
  resetModal()
}

// ===== 键盘导航 =====
const keyboardFocusedIndex = ref(-1)

// 当过滤变化时重置焦点
watch(() => activeTodos.value.length, () => {
  keyboardFocusedIndex.value = -1
})

const handleKeyboardNav = async (event: KeyboardEvent) => {
  // 如果正在输入框或编辑框中，不拦截按键
  const target = event.target as HTMLElement
  const tagName = target.tagName.toLowerCase()
  if (tagName === 'input' || tagName === 'textarea' || tagName === 'select' || target.isContentEditable) {
    if (event.key !== 'Escape') return
  }

  const todos = activeTodos.value
  if (todos.length === 0) return

  const key = event.key

  // 方向键上下
  if (key === 'ArrowDown' || key === 'ArrowUp') {
    event.preventDefault()
    if (keyboardFocusedIndex.value < 0) {
      keyboardFocusedIndex.value = key === 'ArrowDown' ? 0 : todos.length - 1
    } else {
      const step = key === 'ArrowDown' ? 1 : -1
      keyboardFocusedIndex.value = Math.max(0, Math.min(todos.length - 1, keyboardFocusedIndex.value + step))
    }
    scrollToFocusedTodo()
    return
  }

  if (keyboardFocusedIndex.value < 0) return

  const focusedTodo = todos[keyboardFocusedIndex.value]
  if (!focusedTodo) return

  if (key === 'Enter') {
    event.preventDefault()
    toggleExpand(focusedTodo.id)
    return
  }

  if (key === ' ') {
    event.preventDefault()
    todoStore.toggleTodo(focusedTodo.id)
    return
  }

  if (key === 'Delete' || key === 'Backspace') {
    event.preventDefault()
    deleteTodo(focusedTodo.id)
    if (keyboardFocusedIndex.value >= todos.length - 1) {
      keyboardFocusedIndex.value = Math.max(-1, todos.length - 2)
    }
    return
  }

  if (key === '1' || key === '2' || key === '3') {
    event.preventDefault()
    const priorityMap: Record<string, 'high' | 'medium' | 'low'> = {
      '1': 'high',
      '2': 'medium',
      '3': 'low',
    }
    const newPriority = priorityMap[key]
    if (newPriority) {
      await todoStore.updateTodo({ ...focusedTodo, priority: newPriority })
      todosApi.broadcastTaskUpdate(focusedTodo)
      toast.info(`优先级已设为 ${key === '1' ? '高' : key === '2' ? '中' : '低'}`)
    }
    return
  }

  if (key === 'Escape') {
    keyboardFocusedIndex.value = -1
    return
  }
}

const scrollToFocusedTodo = () => {
  const todos = activeTodos.value
  if (keyboardFocusedIndex.value < 0 || keyboardFocusedIndex.value >= todos.length) return
  const todo = todos[keyboardFocusedIndex.value]
  if (!todo) return

  setTimeout(() => {
    const el = document.querySelector(`[data-todo-id="${todo.id}"]`)
    if (el) {
      el.scrollIntoView({ behavior: 'smooth', block: 'nearest' })
    }
  }, 50)
}

// ===== 拖拽排序 =====
const onDragEnd = async () => {
  try {
    console.log("[components/todo/TodoList.vue] onDragEnd() called");
    await todoStore.updateTodo(todoStore.todos) }
  catch (error) { handleError(error, { context: '保存排序', showToast: false }) }
}

// ===== 任务操作 =====
const handleAddTodo = async (newTodoObj) => {
  try {
    console.log("[components/todo/TodoList.vue] handleAddTodo() called");
    await todoStore.addTodo(newTodoObj) }
  catch (error) { handleError(error, { context: '添加任务', showToast: true }) }
}

const deleteTodo = async (id) => {
  try {
    console.log("[components/todo/TodoList.vue] deleteTodo() called");
    await todoStore.deleteTodo(id) }
  catch (error) { handleError(error, { context: '删除任务', showToast: true }) }
}

const toggleExpand = (id) => {
  expandedTodo.value = expandedTodo.value === id ? null : id
}

// ===== 编辑 =====
const startEdit = (todo) => {
  editingId.value = todo.id
  editText.value = todo.text
}

const saveEdit = async (data: { id: string; text: string; projectId?: string | null; priority?: string; tag?: string | null }) => {
  const id = typeof data === 'string' ? data : data.id
  const text = typeof data === 'string' ? editText.value.trim() : data.text.trim()
  
  if (!text || !id) { editingId.value = null; return }
  
  const todo = todoStore.todos.find(t => t.id === id)
  if (todo) {
    const updates: Record<string, any> = { ...todo, text }
    
    // Apply project, priority, tag from edit mode if provided
    if (typeof data !== 'string') {
      if (data.projectId !== undefined) updates.projectId = data.projectId || null
      if (data.priority) updates.priority = data.priority
      if (data.tag !== undefined) updates.tag = data.tag || null
    }
    
    try { await todoStore.updateTodo(updates as any) }
    catch (error) { handleError(error, { context: '保存编辑', showToast: false }) }
  }
  editingId.value = null
  editText.value = ''
}

const cancelEdit = () => {
  editingId.value = null
  editText.value = ''
}

// ===== 标签 =====
const updateTodoTag = async (todo) => {
  try {
    console.log("[components/todo/TodoList.vue] updateTodoTag() called");
    await todoStore.updateTodo(todo) }
  catch (error) { handleError(error, { context: '更新任务标签', showToast: false }) }
  try { await todosApi.broadcastTaskUpdate(todo) } catch {}
}

const addNewTag = async (todo) => {
  const tagName = newTagName.value.trim()
  if (tagName && !todoStore.tags.includes(tagName)) {
    console.log("[components/todo/TodoList.vue] addNewTag() called");
    await todoStore.addTag(tagName)
    try { await todoStore.updateTodo({ ...todo, tag: tagName }) }
    catch (error) { handleError(error, { context: '保存新标签', showToast: false }) }
  }
  newTagName.value = ''
}

// ===== Markdown =====
const startMarkdownEdit = (todo) => {
  editingMarkdownId.value = todo.id
  editingMarkdownContent.value = (todo as any).markdownDescription || ''
  isMarkdownEditing.value = true
}

const saveMarkdown = async (id) => {
  try {
    console.log("[components/todo/TodoList.vue] saveMarkdown() called");
    const todo = todoStore.todos.find(t => t.id === id)
    if (todo) {
      await todoStore.updateTodo({ ...todo, markdownDescription: editingMarkdownContent.value } as any)
      try { await todosApi.broadcastTaskUpdate(todo) } catch {}
      await collab.endCollaborationEdit(id)
      isMarkdownEditing.value = false
      editingMarkdownId.value = null
      editingMarkdownContent.value = ''
    }
  } catch (error) { handleError(error, { context: '保存Markdown', showToast: true }) }
}

const cancelMarkdownEdit = () => {
  isMarkdownEditing.value = false
  editingMarkdownId.value = null
  editingMarkdownContent.value = ''
}

// ===== 子任务 =====
const handleSubtaskCompletion = async (data) => {
  if (data.allCompleted) {
    console.log("[components/todo/TodoList.vue] handleSubtaskCompletion() called");
    const todo = todoStore.todos.find(t => t.id === data.todoId)
    if (todo && !todo.completed) {
      try { await todoStore.toggleTodo(todo.id) }
      catch (error) { handleError(error, { context: '子任务完成自动更新', showToast: false }) }
    }
  }
}

// ===== 通知高亮 =====
const highlightTodoId = ref<string | null>(null)

function navigateToTodo(todoId) {
  if (!todoId) return
  let todo = todoStore.todos.find(t => t.id === todoId)
  if (!todo) return
  const inFilteredResults = todoStore.filteredTodos.some(t => t.id === todoId)
  if (!inFilteredResults) {
    if (todoStore.filter !== 'all') {
      todoStore.setFilter('all')
    }
  }
  highlightTodoId.value = todoId
  setTimeout(() => { highlightTodoId.value = null }, 9000)
  setTimeout(() => {
    const el = document.querySelector(`[data-todo-id="${todoId}"]`)
    if (el) {
      el.scrollIntoView({ behavior: 'smooth', block: 'center' })
    }
  }, 200)
}

onMounted(async () => {
  await todoStore.loadTodos()
  await projectStore.loadProjects()
  window.addEventListener('navigate-to-todo', (event: Event) => {
    const ce = event as CustomEvent
    if (ce.detail && ce.detail.todoId) {
      navigateToTodo(ce.detail.todoId)
    }
  })
})

// ===== LAN & 菜单监听 =====
collab.setupLanListeners(todoStore as any)

const setupMenuListeners = () => {
  const e = getTauriAPI()
  e.onMenuNewTask(() => { (document.querySelector('.todo-input-field') as HTMLElement | null)?.focus() })
  e.onMenuExportMarkdown(() => { })
  e.onMenuExportWord(() => { })
  e.onMenuExportJson(() => { })
  e.onMenuImportJson(() => { })
  e.onMenuClearCompleted(() => { todoStore.clearCompleted() })
  e.onMenuSearchTasks(() => { (document.querySelector('.search-input') as HTMLElement | null)?.focus() })
  e.onMenuSelectAll(() => { batch.selectAll() })
  e.onMenuDeleteSelected(() => { batch.batchDelete() })
  e.onMenuToggleComplete(() => { if (batch.selectedTodos.value.length > 0) batch.batchComplete() })
  e.onMenuSetPriority(async (priority) => {
    for (const id of batch.selectedTodos.value) {
      const todo = todoStore.todos.find(t => t.id === id)
      if (todo) {
        await todoStore.updateTodo({ ...todo, priority: priority as 'high' | 'medium' | 'low' })
        todosApi.broadcastTaskUpdate(todo)
      }
    }
  })
  e.onMenuSetTag(() => { })
  e.onMenuShortcutsHelp(() => {
    toast.info('Cmd/Ctrl+N 新建 | Cmd/Ctrl+F 搜索 | Cmd/Ctrl+D 完成 | Cmd/Ctrl+A 全选')
  })
}
setupMenuListeners()
</script>

<style scoped>
/* Keyboard focus on TodoItem (applied via :deep) — cannot be expressed in Tailwind */
:deep(.todo-item.keyboard-focused) {
  outline: 2px solid var(--color-primary);
  outline-offset: -2px;
  border-radius: 10px;
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
}

/* Slide transition for completed section — Vue <Transition name="slide"> needs these */
.slide-enter-active,
.slide-leave-active {
  transition: all 0.3s ease;
  overflow: hidden;
}
.slide-enter-from,
.slide-leave-to {
  opacity: 0;
  max-height: 0;
}
.slide-enter-to,
.slide-leave-from {
  opacity: 1;
  max-height: 500px;
}
</style>
