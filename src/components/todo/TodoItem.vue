<template>
  <li
    class="todo-item"
    :class="{ completed: todo.completed, 'keyboard-focused': isSelected && false }"
    :data-todo-id="todo.id"
    @click="$emit('toggle-expand', todo.id)"
  >
    <!-- 优先级色条 -->
    <span class="priority-bar" :class="'priority-' + (todo.priority || 'medium')"></span>

    <!-- 复选框 -->
    <input
      type="checkbox"
      :checked="todo.completed"
      @change.stop="$emit('toggle', todo.id)"
      class="todo-check"
      @click.stop
    />

    <!-- 任务文字 -->
    <span class="todo-text" :class="{ done: todo.completed }">
      <span v-html="highlightedText"></span>
    </span>

    <!-- 元信息（右侧） -->
    <div class="todo-meta">
      <span v-if="todo.dueDate" class="meta-due" :class="{ overdue: isOverdue }">
        {{ formatDue(todo.dueDate) }}
      </span>
      <span v-if="todo.priority" class="meta-priority" :class="todo.priority">
        {{ priorityShort(todo.priority) }}
      </span>
      <span v-if="todo.tag" class="meta-tag">#{{ todo.tag }}</span>
      <span v-if="todo.projectId" class="meta-project" :style="{ background: projectColor }">
        {{ projectName }}
      </span>
    </div>

    <!-- 行内操作（hover显示） -->
    <div class="todo-actions" @click.stop>
      <button class="act-btn" @click="$emit('start-edit', todo)" title="编辑">
        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
      </button>
      <button class="act-btn" @click="$emit('delete', todo.id)" title="删除">
        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
      </button>
    </div>
  </li>

  <!-- 展开详情 -->
  <li v-if="expanded" class="todo-detail">
    <div class="detail-inner">
      <div v-if="todo.description" class="detail-section">
        <label>描述</label>
        <p>{{ todo.description }}</p>
      </div>
      <SubtaskList :todo-id="todo.id" @subtask-completed="$emit('subtask-completed', $event)" />
    </div>
  </li>

  <!-- 行内编辑模式 -->
  <li v-if="editingId === todo.id" class="todo-edit">
    <div class="edit-inner">
      <div class="edit-field">
        <label>任务内容</label>
        <input v-model="editLocalText" class="edit-input" @keyup.enter="$emit('save-edit')" @keyup.escape="$emit('cancel-edit')" ref="editInputRef" />
      </div>
      <div class="edit-row">
        <div class="edit-field">
          <label>项目</label>
          <select v-model="editLocalProjectId" class="edit-select">
            <option value="">无</option>
            <option v-for="p in projects" :key="p.id" :value="p.id">{{ p.name }}</option>
          </select>
        </div>
        <div class="edit-field">
          <label>优先级</label>
          <select v-model="editLocalPriority" class="edit-select">
            <option value="low">低</option>
            <option value="medium">中</option>
            <option value="high">高</option>
          </select>
        </div>
        <div class="edit-field">
          <label>标签</label>
          <select v-model="editLocalTag" class="edit-select">
            <option value="">无</option>
            <option v-for="t in tags" :key="t" :value="t">#{{ t }}</option>
          </select>
        </div>
      </div>
      <div class="edit-actions">
        <button class="edit-btn edit-save" @click="emitSaveEdit">保存</button>
        <button class="edit-btn edit-cancel" @click="$emit('cancel-edit')">取消</button>
      </div>
    </div>
  </li>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import DOMPurify from 'dompurify'
import SubtaskList from '@/components/subtask/SubtaskList.vue'

defineOptions({ inheritAttrs: false })

const props = defineProps({
  todo: { type: Object, required: true },
  searchQuery: { type: String, default: '' },
  isSelected: { type: Boolean, default: false },
  expanded: { type: Boolean, default: false },
  editingId: { type: String, default: null },
  editText: { type: String, default: '' },
  tags: { type: Array as () => string[], default: () => [] },
  projects: { type: Array as () => { id: string; name: string; color: string }[], default: () => [] },
  isMarkdownEditing: { type: Boolean, default: false },
  editingMarkdownId: { type: String, default: null },
  editingMarkdownContent: { type: String, default: '' },
  collaboratingUser: { type: String, default: '' },
  comments: { type: Array, default: () => [] },
  commentInput: { type: String, default: '' },
  highlighted: { type: Boolean, default: false },
})

const emit = defineEmits([
  'toggle', 'delete', 'toggle-selected', 'toggle-expand',
  'start-edit', 'save-edit', 'cancel-edit', 'update:editText',
  'update:editingMarkdownContent', 'update:commentInput',
  'update-tag', 'add-new-tag', 'handle-markdown-double-click',
  'save-markdown', 'cancel-markdown', 'add-comment', 'subtask-completed',
  'update-project', 'update-priority', 'startMarkdownEdit',
])

const { locale: i18nLocale } = useI18n()

// Edit mode local state
const editInputRef = ref<HTMLInputElement | null>(null)
const editLocalText = ref(props.todo.text)
const editLocalProjectId = ref(props.todo.projectId || '')
const editLocalPriority = ref(props.todo.priority || 'medium')
const editLocalTag = ref(props.todo.tag || '')

// Watch editingId to initialize local state when entering edit mode
watch(() => props.editingId, (val) => {
  if (val === props.todo.id) {
    editLocalText.value = props.todo.text
    editLocalProjectId.value = props.todo.projectId || ''
    editLocalPriority.value = props.todo.priority || 'medium'
    editLocalTag.value = props.todo.tag || ''
    nextTick(() => {
      editInputRef.value?.focus()
      editInputRef.value?.select()
    })
  }
}, { immediate: true })

// Sync editText v-model
watch(editLocalText, (val) => emit('update:editText', val))

// Emit save with all edit fields
function emitSaveEdit() {
  emit('save-edit', {
    id: props.todo.id,
    text: editLocalText.value,
    projectId: editLocalProjectId.value || null,
    priority: editLocalPriority.value,
    tag: editLocalTag.value || null,
  })
}

const isOverdue = computed(() => {
  if (!props.todo?.dueDate || props.todo.completed) return false
  const today = new Date(); today.setHours(0, 0, 0, 0)
  const due = new Date(props.todo.dueDate); due.setHours(0, 0, 0, 0)
  return due < today
})

const highlightedText = computed(() => {
  if (!props.searchQuery) return DOMPurify.sanitize(props.todo.text)
  const q = props.searchQuery.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  return DOMPurify.sanitize(props.todo.text.replace(new RegExp(`(${q})`, 'gi'), '<mark>$1</mark>'))
})

const projectName = computed(() => {
  const p = props.projects.find(p => p.id === props.todo.projectId)
  return p ? p.name : ''
})
const projectColor = computed(() => {
  const p = props.projects.find(p => p.id === props.todo.projectId)
  return p ? p.color : '#6366f1'
})

const formatDue = (d: string) => {
  const date = new Date(d)
  const now = new Date(); now.setHours(0, 0, 0, 0)
  const due = new Date(d); due.setHours(0, 0, 0, 0)
  const diff = Math.round((due.getTime() - now.getTime()) / 86400000)
  if (diff === 0) return '今天'
  if (diff === 1) return '明天'
  if (diff === -1) return '昨天'
  return `${date.getMonth() + 1}/${date.getDate()}`
}

const priorityShort = (p: string) => {
  const map: Record<string, string> = { high: '高', medium: '中', low: '低' }
  return map[p] || p
}
</script>

<style scoped>
/* ===== 任务行 ===== */
.todo-item {
  display: flex;
  align-items: center;
  gap: 0;
  padding: 0 14px;
  height: 44px;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
  background: oklch(var(--b1));
  cursor: default;
  position: relative;
  transition: all 0.15s ease;
  list-style: none;
}
.todo-item:last-child {
  border-bottom: none;
}
.todo-item:hover {
  background: oklch(var(--b2));
  box-shadow: inset 0 0 0 1px oklch(var(--bc) / 0.1);
}
.todo-item:hover .drag-handle { opacity: 1; }
.todo-item:hover .todo-actions { opacity: 1; }

.todo-item.completed { opacity: 0.45; }
.todo-item.completed:hover { opacity: 0.65; }

/* 优先级色条 */
.priority-bar {
  position: absolute;
  left: 0;
  top: 6px;
  bottom: 6px;
  width: 3px;
  border-radius: 2px;
  opacity: 0;
  transition: opacity 0.15s ease;
}
.todo-item:hover .priority-bar { opacity: 1; }
.priority-bar.priority-high { background: oklch(var(--er)); }
.priority-bar.priority-medium { background: oklch(var(--wa)); }
.priority-bar.priority-low { background: oklch(var(--su)); }

/* 拖拽 */
.drag-handle {
  width: 18px;
  text-align: center;
  font-size: 14px;
  color: oklch(var(--bc) / 0.6);
  cursor: grab;
  opacity: 0;
  transition: opacity 0.15s;
  flex-shrink: 0;
  user-select: none;
}

/* 复选框 */
.todo-check {
  appearance: none;
  width: 20px;
  height: 20px;
  min-width: 20px;
  border: 2px solid oklch(var(--bc) / 0.1);
  border-radius: 50%;
  cursor: pointer;
  margin: 0 10px 0 4px;
  position: relative;
  transition: all 0.2s ease;
  flex-shrink: 0;
}
.todo-check:hover {
  border-color: oklch(var(--p));
  box-shadow: 0 0 0 3px oklch(var(--p) / 0.1);
}
.todo-check:checked {
  background: oklch(var(--p));
  border-color: oklch(var(--p));
  animation: checkPop 0.25s ease;
}
.todo-check:checked::after {
  content: '';
  position: absolute;
  left: 5px; top: 2px;
  width: 5px; height: 9px;
  border: solid #fff;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
}

@keyframes checkPop {
  0% { transform: scale(1); }
  40% { transform: scale(1.25); }
  100% { transform: scale(1); }
}

/* 文字 */
.todo-text {
  flex: 1;
  min-width: 0;
  font-size: 14px;
  color: oklch(var(--bc));
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 1.4;
  font-weight: 400;
}
.todo-text.done {
  text-decoration: line-through;
  color: oklch(var(--bc) / 0.6);
}
.todo-text :deep(mark) {
  background: rgba(250, 204, 21, 0.35);
  border-radius: 2px;
  padding: 0 1px;
}

/* 元信息 */
.todo-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
  margin-left: 10px;
}

.meta-due {
  font-size: 11px;
  color: oklch(var(--bc) / 0.6);
  white-space: nowrap;
  font-weight: 500;
}
.meta-due.overdue {
  color: oklch(var(--er));
  font-weight: 600;
}

.meta-priority {
  font-size: 10px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 4px;
  text-transform: uppercase;
  letter-spacing: 0.2px;
  white-space: nowrap;
}
.meta-priority.high { color: oklch(var(--er)); background: rgba(239, 68, 68, 0.1); }
.meta-priority.medium { color: oklch(var(--wa)); background: rgba(245, 158, 11, 0.1); }
.meta-priority.low { color: oklch(var(--su)); background: rgba(34, 197, 94, 0.1); }

.meta-tag {
  font-size: 11px;
  color: oklch(var(--p));
  padding: 2px 6px;
  border-radius: 4px;
  background: oklch(var(--p) / 0.1);
  white-space: nowrap;
  font-weight: 500;
}

.meta-project {
  font-size: 10px;
  font-weight: 500;
  color: #fff;
  padding: 2px 6px;
  border-radius: 4px;
  opacity: 0.85;
  white-space: nowrap;
}

/* 操作按钮 */
.todo-actions {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  margin-left: 8px;
  opacity: 0;
  transition: opacity 0.12s;
}
.act-btn {
  width: 28px; height: 28px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: oklch(var(--bc) / 0.6);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.1s;
}
.act-btn:hover {
  background: rgba(239, 68, 68, 0.1);
  color: oklch(var(--er));
}

/* ===== 展开详情 ===== */
.todo-detail {
  list-style: none;
  background: oklch(var(--b2));
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
  padding: 12px 14px 12px 48px;
}
.detail-inner { font-size: 13px; color: oklch(var(--bc)); }
.detail-section label {
  display: block;
  font-size: 11px;
  font-weight: 600;
  color: oklch(var(--bc) / 0.6);
  text-transform: uppercase;
  letter-spacing: 0.3px;
  margin-bottom: 4px;
}
.detail-section p {
  margin: 0;
  line-height: 1.5;
  color: oklch(var(--bc));
}

/* ===== 行内编辑模式 ===== */
.todo-edit {
  background: oklch(var(--b2));
  border-bottom: 2px solid oklch(var(--p));
  padding: 12px 14px;
  list-style: none;
}

.edit-inner {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.edit-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.edit-field label {
  font-size: 11px;
  font-weight: 600;
  color: oklch(var(--bc) / 0.6);
}

.edit-input {
  width: 100%;
  padding: 8px 10px;
  font-size: 14px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  background: oklch(var(--b1));
  color: oklch(var(--bc));
  transition: all 0.15s ease;
}

.edit-input:focus {
  outline: none;
  border-color: oklch(var(--p));
  box-shadow: 0 0 0 3px rgba(66, 133, 244, 0.15);
}

.edit-row {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 10px;
}

.edit-select {
  width: 100%;
  padding: 7px 10px;
  font-size: 13px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  background: oklch(var(--b1));
  color: oklch(var(--bc));
  cursor: pointer;
}

.edit-select:focus {
  outline: none;
  border-color: oklch(var(--p));
}

.edit-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  margin-top: 4px;
}

.edit-btn {
  padding: 6px 16px;
  font-size: 13px;
  font-weight: 600;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.edit-save {
  background: oklch(var(--p));
  color: white;
}

.edit-save:hover {
  opacity: 0.9;
}

.edit-cancel {
  background: transparent;
  color: oklch(var(--bc) / 0.6);
  border: 1px solid oklch(var(--bc) / 0.1);
}

.edit-cancel:hover {
  background: oklch(var(--b1));
}
</style>
