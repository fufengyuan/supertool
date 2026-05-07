<template>
  <div class="todo-container" ref="containerRef" tabindex="0" @keydown="handleKeyboardNav">
    <!-- 快速输入框 -->
    <div class="quick-add-bar">
      <svg class="quick-add-icon" xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
      <input
        ref="quickAddInput"
        v-model="quickAddText"
        class="quick-add-input"
        placeholder="添加新任务，回车即可保存…"
        @keyup.enter="handleQuickAdd"
        @keydown.escape="quickAddText = ''"
      />
      <button v-if="quickAddText" class="quick-add-priority" @click="cycleQuickPriority" :title="'优先级: ' + quickPriorityLabel">
        <span :class="'priority-dot-' + quickAddPriority"></span>
        {{ quickPriorityLabel }}
      </button>
      <select v-if="quickAddText" v-model="quickAddProjectId" class="quick-add-project" @change="handleQuickAdd">
        <option value="">无项目</option>
        <option v-for="p in projectStore.projects" :key="p.id" :value="p.id">{{ p.name }}</option>
      </select>
    </div>

    <!-- 顶部工具栏：搜索 + 排序 + 添加按钮 -->
    <div class="todo-toolbar">
      <div class="toolbar-left">
        <input
          v-model="searchQueryValue"
          placeholder="搜索任务…"
          class="toolbar-search"
          @keydown.escape="searchQueryValue = ''"
        />
        <select v-model="filterValue" class="toolbar-select">
          <option value="all">全部</option>
          <option value="active">进行中</option>
          <option value="completed">已完成</option>
        </select>
        <select v-model="sortValue" class="toolbar-select">
          <option value="">默认排序</option>
          <option value="priority">按优先级</option>
          <option value="dueDate">按截止日期</option>
          <option value="createdAt">按创建时间</option>
        </select>
      </div>
      <div class="toolbar-right">
        <span class="toolbar-count">{{ todoStore.activeCount }} 进行中</span>
        <button class="add-task-btn" @click="showAddModal = true" title="添加任务 (Ctrl+N)">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
          更多
        </button>
      </div>
    </div>

    <!-- 今日进度 -->
    <div class="progress-bar-container" v-if="todoStore.todos.length > 0">
      <div class="progress-info">
        <span class="progress-label">今日进度</span>
        <span class="progress-count">{{ todoStore.completedCount }}/{{ todoStore.todos.length }}</span>
      </div>
      <div class="progress-track">
        <div class="progress-fill" :style="{ width: progressPercent + '%' }"></div>
      </div>
    </div>

    <!-- 键盘导航提示 -->
    <div class="keyboard-hint" v-if="keyboardFocusedIndex >= 0">
      <kbd>↑↓</kbd> 浏览 <kbd>Enter</kbd> 展开 <kbd>Space</kbd> 完成 <kbd>Del</kbd> 删除 <kbd>1</kbd><kbd>2</kbd><kbd>3</kbd> 优先级
    </div>

    <!-- 任务卡片容器 -->
    <div class="todo-card">
      <div class="todo-list-area">
        <!-- 活跃任务列表（按项目分组） -->
        <template v-for="group in groupedActiveTodos" :key="group.projectId">
          <!-- 项目分组头部 -->
          <div class="project-group-header">
            <span
              v-if="group.project"
              class="project-header-name"
              :style="group.project.color ? { borderLeftColor: group.project.color } : {}"
            >
              <span v-if="group.project.color" class="project-color-dot" :style="{ backgroundColor: group.project.color }"></span>
              {{ group.project.name }}
              <span class="project-header-count">{{ group.todos.length }}</span>
            </span>
            <span v-else class="project-header-name no-project">
              <span class="project-color-dot" style="background-color: var(--main-text-secondary)"></span>
              无项目
              <span class="project-header-count">{{ group.todos.length }}</span>
            </span>
          </div>
          <!-- 分组内的任务列表 -->
          <ul class="todo-list">
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
        <div v-if="groupedCompletedTodos.length > 0 && filterValue === 'all' && !searchQueryValue" class="completed-section">
          <button class="completed-toggle" @click="showCompleted = !showCompleted">
            <svg :class="{ rotated: showCompleted }" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="6 9 12 15 18 9"/></svg>
            已完成 ({{ completedTodos.length }})
          </button>
          <Transition name="slide">
            <div v-if="showCompleted" class="completed-groups-wrapper">
              <template v-for="group in groupedCompletedTodos" :key="'completed-' + group.projectId">
                <div v-if="group.project" class="project-group-header completed-group-header">
                  <span class="project-header-name" :style="group.project.color ? { borderLeftColor: group.project.color } : {}">
                    <span v-if="group.project.color" class="project-color-dot" :style="{ backgroundColor: group.project.color }"></span>
                    {{ group.project.name }}
                    <span class="project-header-count">{{ group.todos.length }}</span>
                  </span>
                </div>
                <div v-else class="project-group-header completed-group-header">
                  <span class="project-header-name no-project">
                    <span class="project-color-dot" style="background-color: var(--main-text-secondary)"></span>
                    无项目
                    <span class="project-header-count">{{ group.todos.length }}</span>
                  </span>
                </div>
                <ul class="completed-list">
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
      <div v-if="activeTodos.length === 0 && completedTodos.length === 0 && todoStore.todos.length === 0 && !searchQueryValue" class="empty-state">
        <svg class="empty-state-icon" viewBox="0 0 24 24" width="64" height="64" fill="none" stroke="currentColor" stroke-width="1">
          <path d="M9 11l3 3L22 4" />
          <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11" />
        </svg>
        <p class="empty-state-title">还没有任务</p>
        <p class="empty-state-hint">在上方输入框添加你的第一个任务</p>
      </div>
      <div v-else-if="activeTodos.length === 0 && completedTodos.length === 0" class="empty-state">
        <p>没有找到匹配的任务，尝试调整筛选条件</p>
      </div>
    </div>

    <!-- 批量操作和清空已完成 -->
    <div class="todo-actions" v-if="selectedTodoIds.length > 0 || (todoStore.completedCount > 0 && filterValue === 'all')">
      <div v-if="selectedTodoIds.length > 0" class="batch-actions">
        <button @click="batch.batchComplete" class="batch-btn complete">完成 ({{ selectedTodoIds.length }})</button>
        <button @click="batch.batchDelete" class="batch-btn delete">删除 ({{ selectedTodoIds.length }})</button>
      </div>
      <div v-if="todoStore.completedCount > 0 && filterValue === 'all'" class="single-actions">
        <button @click="todoStore.clearCompleted" class="clear-btn">清空已完成</button>
      </div>
    </div>

    <!-- 添加任务弹出框 -->
    <div v-if="showAddModal" class="modal-overlay" @click.self="showAddModal = false">
      <div class="modal-dialog">
        <div class="modal-header">
          <h3>添加任务</h3>
          <button class="modal-close" @click="showAddModal = false">×</button>
        </div>
        <div class="modal-body">
          <div class="form-row">
            <label>任务内容</label>
            <input
              ref="addTaskInput"
              v-model="newTaskText"
              @keyup.enter="handleAddFromModal"
              placeholder="输入任务内容…"
              class="form-input"
              autofocus
            />
          </div>
          <div class="form-row form-row-inline">
            <div class="form-field">
              <label>优先级</label>
              <select v-model="newTaskPriority" class="form-select">
                <option value="low">低</option>
                <option value="medium">中</option>
                <option value="high">高</option>
              </select>
            </div>
            <div class="form-field">
              <label>截止日期</label>
              <input v-model="newTaskDueDate" type="date" class="form-input" />
            </div>
            <div class="form-field">
              <label>标签</label>
              <select v-model="newTaskTag" class="form-select">
                <option value="">无</option>
                <option v-for="tag in todoStore.tags" :key="tag" :value="tag">{{ tag }}</option>
                <option value="__custom__">自定义</option>
              </select>
            </div>
          </div>
          <div class="form-row" v-if="newTaskTag === '__custom__'">
            <label>自定义标签</label>
            <input v-model="newTaskCustomTag" placeholder="输入标签名称" class="form-input" />
          </div>
          <div class="form-row form-row-inline">
            <div class="form-field">
              <label>项目</label>
              <select v-model="newTaskProjectId" class="form-select">
                <option value="">无</option>
                <option v-for="p in projectStore.projects" :key="p.id" :value="p.id">{{ p.name }}</option>
              </select>
            </div>
            <div class="form-field">
              <label>重复</label>
              <select v-model="newTaskRepeat" class="form-select">
                <option value="">不重复</option>
                <option value="daily">每天</option>
                <option value="weekly">每周</option>
                <option value="monthly">每月</option>
              </select>
            </div>
          </div>
          <div class="form-row">
            <label>描述</label>
            <textarea v-model="newTaskDesc" placeholder="任务描述（可选）" class="form-input textarea" rows="2"></textarea>
          </div>
        </div>
        <div class="modal-footer">
          <button class="modal-cancel" @click="showAddModal = false">取消</button>
          <button class="modal-confirm" @click="handleAddFromModal">添加任务</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { getTauriAPI } from '../../utils/tauri-api'
import TodoItem from './TodoItem.vue'
import VirtualList from './VirtualList.vue'
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
  e.onMenuAbout(() => { toast.info('SuperTool 应用 v1.0.0') })
  e.onMenuShortcutsHelp(() => {
    toast.info('Cmd/Ctrl+N 新建 | Cmd/Ctrl+F 搜索 | Cmd/Ctrl+D 完成 | Cmd/Ctrl+A 全选')
  })
  e.onMenuCheckUpdate(() => { toast.info('当前已是最新版本') })
}
setupMenuListeners()
</script>

<style scoped>
.todo-container {
  width: 100%;
  outline: none;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 20px;
}

/* ===== 快速输入框 ===== */
.quick-add-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  background: var(--card-bg);
  border: 2px solid var(--primary-color);
  border-radius: 14px;
  box-shadow: 0 2px 12px var(--primary-light);
  transition: all 0.2s ease;
  flex-shrink: 0;
}
.quick-add-bar:focus-within {
  box-shadow: 0 4px 20px rgba(136, 57, 239, 0.2);
  border-color: var(--primary-hover);
}
.quick-add-icon {
  color: var(--primary-color);
  flex-shrink: 0;
  opacity: 0.7;
}
.quick-add-input {
  flex: 1;
  border: none;
  outline: none;
  background: transparent;
  font-size: 15px;
  color: var(--main-text);
  font-weight: 500;
}
.quick-add-input::placeholder {
  color: var(--empty-color);
  font-weight: 400;
}
.quick-add-priority {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 4px 10px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--input-bg);
  color: var(--main-text-secondary);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
  flex-shrink: 0;
}
.quick-add-priority:hover {
  border-color: var(--primary-color);
  color: var(--primary-color);
}
.quick-add-project {
  padding: 4px 8px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--input-bg);
  color: var(--main-text);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
  flex-shrink: 0;
  max-width: 180px;
}
.quick-add-project:focus {
  outline: none;
  border-color: var(--primary-color);
}
.priority-dot-low { width: 8px; height: 8px; border-radius: 50%; background: var(--success-color); }
.priority-dot-medium { width: 8px; height: 8px; border-radius: 50%; background: var(--warning-color); }
.priority-dot-high { width: 8px; height: 8px; border-radius: 50%; background: var(--danger-color); }

/* ===== 顶部工具栏 ===== */
.todo-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 4px 0 8px;
  flex-shrink: 0;
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.toolbar-search {
  flex: 1;
  min-width: 180px;
  max-width: 520px;
  padding: 7px 12px 7px 32px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--card-bg) url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='14' height='14' viewBox='0 0 24 24' fill='none' stroke='%239ca3af' stroke-width='2' stroke-linecap='round'%3E%3Ccircle cx='11' cy='11' r='8'/%3E%3Cline x1='21' y1='21' x2='16.65' y2='16.65'/%3E%3C/svg%3E") 10px center no-repeat;
  color: var(--main-text);
  font-size: 13px;
  outline: none;
  transition: border-color 0.15s ease, box-shadow 0.15s ease;
}
.toolbar-search:focus {
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px var(--primary-light);
}
.toolbar-search::placeholder {
  color: var(--empty-color);
}

.toolbar-select {
  padding: 7px 24px 7px 10px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--card-bg);
  color: var(--main-text);
  font-size: 13px;
  cursor: pointer;
  outline: none;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%239ca3af' stroke-width='2' stroke-linecap='round'%3E%3Cpolyline points='6 9 12 15 18 9'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 8px center;
  transition: border-color 0.15s ease;
}
.toolbar-select:focus {
  border-color: var(--primary-color);
}
.toolbar-select:hover {
  border-color: var(--main-text-secondary);
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
}

.toolbar-count {
  font-size: 13px;
  font-weight: 500;
  color: var(--main-text-secondary);
  padding: 4px 10px;
  background: var(--input-bg);
  border-radius: 6px;
}

.add-task-btn {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 7px 14px;
  background: var(--card-bg);
  color: var(--main-text);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.15s ease;
}
.add-task-btn:hover {
  border-color: var(--primary-color);
  color: var(--primary-color);
}
.add-task-btn svg {
  flex-shrink: 0;
}

/* ===== 进度条 ===== */
.progress-bar-container {
  padding: 0 2px;
  flex-shrink: 0;
}
.progress-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
}
.progress-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--main-text-secondary);
}
.progress-count {
  font-size: 12px;
  font-weight: 600;
  color: var(--primary-color);
}
.progress-track {
  height: 4px;
  background: var(--input-bg);
  border-radius: 4px;
  overflow: hidden;
}
.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--primary-color), var(--primary-hover));
  border-radius: 4px;
  transition: width 0.4s ease;
}

/* ===== 键盘导航 ===== */
.keyboard-hint {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 3px 10px;
  background: var(--primary-light);
  border-radius: 6px;
  font-size: 11px;
  color: var(--main-text-secondary);
  flex-shrink: 0;
}
.keyboard-hint kbd {
  display: inline-block;
  padding: 1px 5px;
  border-radius: 3px;
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  font-size: 10px;
  font-weight: 600;
  color: var(--main-text);
  font-family: inherit;
}
:deep(.todo-item.keyboard-focused) {
  outline: 2px solid var(--primary-color);
  outline-offset: -2px;
  border-radius: 10px;
  background: var(--primary-light);
}

/* ===== 任务卡片容器 ===== */
.todo-card {
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
  display: flex;
  flex-direction: column;
}

/* ===== 列表区域 ===== */
.todo-list-area {
  display: flex;
  flex-direction: column;
  padding: 4px 0;
}

.todo-list {
  list-style: none;
  padding: 0;
  margin: 0;
}

/* ===== 项目分组头部 ===== */
.project-group-header {
  display: flex;
  align-items: center;
  padding: 8px 16px 4px;
  position: sticky;
  top: 0;
  z-index: 2;
  background: var(--card-bg);
}

.completed-group-header {
  opacity: 0.6;
  padding: 6px 16px 2px;
}

.project-header-name {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 600;
  color: var(--main-text);
  padding: 2px 0 2px 10px;
  border-left: 3px solid var(--primary-color);
}

.project-header-name.no-project {
  color: var(--main-text-secondary);
  border-left-color: var(--main-text-secondary);
}

.project-color-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.project-header-count {
  font-size: 11px;
  font-weight: 400;
  color: var(--main-text-secondary);
  margin-left: 2px;
}

.draggable-list { min-height: 10px; }
.virtual-todo-list { border-radius: 8px; }
.virtual-todo-list > .virtual-list { height: 100% !important; }
.drag-ghost { opacity: 0.4; background: var(--primary-light); }
.drag-chosen { box-shadow: 0 0 0 2px var(--primary-color); }

/* ===== 已完成折叠区 ===== */
.completed-section {
  border-top: 1px solid var(--border-color);
  margin-top: 4px;
  padding: 0;
}
.completed-toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  padding: 10px 16px;
  border: none;
  background: transparent;
  color: var(--main-text-secondary);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: color 0.15s ease;
}
.completed-toggle:hover {
  color: var(--main-text);
}
.completed-toggle svg {
  transition: transform 0.2s ease;
}
.completed-toggle svg.rotated {
  transform: rotate(90deg);
}
.completed-list {
  list-style: none;
  padding: 0;
  margin: 0;
  opacity: 0.6;
}

/* slide 动画 */
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

/* ===== 空状态 ===== */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 20px;
  color: var(--main-text-secondary);
}
.empty-state-icon { opacity: 0.12; margin-bottom: 20px; }
.empty-state-title { font-size: 16px; font-weight: 600; color: var(--main-text); margin: 0 0 8px 0; }
.empty-state-hint { font-size: 13px; margin: 0; opacity: 0.7; }

/* ===== 批量操作 ===== */
.todo-actions {
  display: flex;
  gap: 6px;
  align-items: center;
  flex-shrink: 0;
  padding: 8px 0;
  flex-wrap: wrap;
}
.batch-actions { display: flex; gap: 6px; }
.batch-btn {
  padding: 5px 12px;
  border: 1px solid var(--border-color);
  background: var(--card-bg);
  color: var(--main-text);
  border-radius: 6px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 500;
  transition: all 0.15s ease;
}
.batch-btn.complete:hover { border-color: var(--success-color); color: var(--success-color); background: rgba(34, 197, 94, 0.05); }
.batch-btn.delete:hover { border-color: var(--danger-color); color: var(--danger-color); background: rgba(239, 68, 68, 0.05); }
.single-actions { margin-left: auto; }
.clear-btn {
  padding: 5px 12px;
  background: transparent;
  color: var(--main-text-secondary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.15s ease;
}
.clear-btn:hover { color: var(--warning-color); border-color: var(--warning-color); }

/* ===== 弹窗 ===== */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: fadeIn 0.15s ease;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes slideUp {
  from { opacity: 0; transform: translateY(16px) scale(0.98); }
  to { opacity: 1; transform: translateY(0) scale(1); }
}

.modal-dialog {
  width: 540px;
  max-height: 85vh;
  background: var(--card-bg);
  border-radius: 16px;
  border: 1px solid var(--border-color);
  box-shadow: 0 24px 80px rgba(0, 0, 0, 0.25);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  animation: slideUp 0.2s ease;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 24px;
  border-bottom: 1px solid var(--border-color);
}

.modal-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--main-text);
}

.modal-close {
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  color: var(--main-text-secondary);
  font-size: 20px;
  cursor: pointer;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}
.modal-close:hover {
  background: var(--input-bg);
  color: var(--main-text);
}

.modal-body {
  padding: 20px 24px;
  overflow-y: auto;
  flex: 1;
}

.form-row {
  margin-bottom: 14px;
}
.form-row label {
  display: block;
  font-size: 12px;
  font-weight: 600;
  color: var(--main-text-secondary);
  margin-bottom: 5px;
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.form-row-inline {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 12px;
}

.form-field {
  display: flex;
  flex-direction: column;
}

.form-input, .form-select {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--input-bg);
  color: var(--main-text);
  font-size: 13px;
  outline: none;
  transition: border-color 0.15s ease, box-shadow 0.15s ease;
}
.form-input:focus, .form-select:focus {
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px var(--primary-light);
}
.form-input::placeholder {
  color: var(--empty-color);
}
.form-input.textarea {
  resize: vertical;
  min-height: 56px;
  font-family: inherit;
}

.form-select {
  appearance: none;
  cursor: pointer;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%239ca3af' stroke-width='2' stroke-linecap='round'%3E%3Cpolyline points='6 9 12 15 18 9'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 10px center;
  padding-right: 28px;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 14px 24px;
  border-top: 1px solid var(--border-color);
  background: var(--input-bg);
}

.modal-cancel {
  padding: 8px 20px;
  background: transparent;
  color: var(--main-text-secondary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  transition: all 0.15s ease;
}
.modal-cancel:hover {
  background: var(--card-bg);
  color: var(--main-text);
}

.modal-confirm {
  padding: 8px 24px;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 600;
  transition: all 0.15s ease;
}
.modal-confirm:hover {
  background: var(--primary-hover);
  transform: translateY(-1px);
}
.modal-confirm:active {
  transform: translateY(0);
}
</style>
