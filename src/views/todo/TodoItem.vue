<template>
  <!-- 主任务行 -->
  <li
    class="group flex items-center px-3.5 h-11 border-b border-base-content/10 bg-base-100 relative transition-all duration-150 ease cursor-default list-none"
    :class="{ 'opacity-45 hover:opacity-65': todo.completed }"
    :data-todo-id="todo.id"
    @click="$emit('toggle-expand', todo.id)"
  >
    <!-- 优先级色条 -->
    <span
      class="absolute left-0 top-1.5 bottom-1.5 w-[3px] rounded opacity-0 transition-opacity duration-150 group-hover:opacity-100"
      :class="{
        'bg-error': (todo.priority || 'medium') === 'high',
        'bg-warning': (todo.priority || 'medium') === 'medium',
        'bg-success': (todo.priority || 'medium') === 'low',
      }"
    ></span>

    <!-- 复选框 -->
    <input
      type="checkbox"
      :checked="todo.completed"
      @change.stop="$emit('toggle', todo.id)"
      class="checkbox checkbox-primary checkbox-sm mx-2.5 ms-1 shrink-0"
      @click.stop
    />

    <!-- 任务文字 -->
    <span class="flex-1 min-w-0 text-sm text-base-content truncate leading-tight font-normal" :class="{ 'line-through text-base-content/60': todo.completed }">
      <span class="[&_mark]:bg-yellow-300/35 [&_mark]:rounded-sm [&_mark]:px-0.5" v-html="highlightedText"></span>
    </span>

    <!-- 元信息（右侧） -->
    <div class="flex items-center gap-1.5 shrink-0 ml-2.5">
      <span v-if="todo.dueDate" class="text-xs text-base-content/60 whitespace-nowrap font-medium" :class="{ 'text-error font-semibold': isOverdue }">
        {{ formatDue(todo.dueDate) }}
      </span>
      <span v-if="todo.priority" class="text-[10px] font-bold px-1.5 py-0.5 rounded uppercase tracking-wide whitespace-nowrap" :class="{
        'text-error bg-error/10': todo.priority === 'high',
        'text-warning bg-warning/10': todo.priority === 'medium',
        'text-success bg-success/10': todo.priority === 'low',
      }">
        {{ priorityShort(todo.priority) }}
      </span>
      <span v-if="todo.tag" class="text-[11px] text-primary px-1.5 py-0.5 rounded bg-primary/10 whitespace-nowrap font-medium">#{{ todo.tag }}</span>
      <span v-if="todo.projectId" class="text-[10px] font-medium text-white px-1.5 py-0.5 rounded opacity-85 whitespace-nowrap" :style="{ background: projectColor }">
        {{ projectName }}
      </span>
    </div>

    <!-- 行内操作（hover显示） -->
    <div class="flex items-center shrink-0 ml-2 opacity-0 transition-opacity duration-100 group-hover:opacity-100" @click.stop>
      <button class="w-7 h-7 border-0 rounded-lg bg-transparent text-base-content/60 cursor-pointer flex items-center justify-center transition-all duration-100 hover:bg-error/10 hover:text-error" @click="$emit('start-edit', todo)" title="编辑">
        <SvgIcon name="pencil" size="14" />
      </button>
      <button class="w-7 h-7 border-0 rounded-lg bg-transparent text-base-content/60 cursor-pointer flex items-center justify-center transition-all duration-100 hover:bg-error/10 hover:text-error" @click="$emit('delete', todo.id)" title="删除">
        <SvgIcon name="trash" size="14" />
      </button>
    </div>
  </li>

  <!-- 展开详情 -->
  <li v-if="expanded" class="list-none bg-base-200 border-b border-base-content/10 p-3 pl-12">
    <div class="text-sm text-base-content space-y-2">
      <div v-if="todo.description" class="detail-section">
        <label class="text-[11px] font-semibold uppercase text-base-content/60 tracking-wider block mb-1">描述</label>
        <p class="m-0 leading-normal text-base-content">{{ todo.description }}</p>
      </div>
      <SubtaskList :todo-id="todo.id" @subtask-completed="$emit('subtask-completed', $event)" />
    </div>
  </li>

  <!-- 行内编辑模式 -->
  <li v-if="editingId === todo.id" class="list-none bg-base-200 border-b-2 border-primary p-3">
    <div class="flex flex-col gap-2.5">
      <div class="flex flex-col gap-1">
        <label class="text-[11px] font-semibold text-base-content/60">任务内容</label>
        <input v-model="editLocalText" class="input input-bordered w-full text-sm" @keyup.enter="emitSaveEdit" @keyup.escape="$emit('cancel-edit')" ref="editInputRef" />
      </div>
      <div class="grid grid-cols-4 gap-2.5">
        <div class="flex flex-col gap-1">
          <label class="text-[11px] font-semibold text-base-content/60">项目</label>
          <select v-model="editLocalProjectId" class="select select-bordered select-sm w-full text-sm">
            <option value="">无</option>
            <option v-for="p in projects" :key="p.id" :value="p.id">{{ p.name }}</option>
          </select>
        </div>
        <div class="flex flex-col gap-1">
          <label class="text-[11px] font-semibold text-base-content/60">截止日期</label>
          <input v-model="editLocalDueDate" type="date" class="input input-bordered input-sm w-full text-sm" />
        </div>
        <div class="flex flex-col gap-1">
          <label class="text-[11px] font-semibold text-base-content/60">优先级</label>
          <select v-model="editLocalPriority" class="select select-bordered select-sm w-full text-sm">
            <option value="low">低</option>
            <option value="medium">中</option>
            <option value="high">高</option>
          </select>
        </div>
        <div class="flex flex-col gap-1">
          <label class="text-[11px] font-semibold text-base-content/60">标签</label>
          <select v-model="editLocalTag" class="select select-bordered select-sm w-full text-sm">
            <option value="">无</option>
            <option v-for="t in tags" :key="t" :value="t">#{{ t }}</option>
          </select>
        </div>
      </div>
      <div class="flex gap-2 justify-end mt-1">
        <button class="btn btn-primary btn-sm" @click="emitSaveEdit">保存</button>
        <button class="btn btn-ghost btn-sm" @click="$emit('cancel-edit')">取消</button>
      </div>
    </div>
  </li>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import DOMPurify from 'dompurify'
import SubtaskList from '@/views/subtask/SubtaskList.vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'

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
const editLocalDueDate = ref(props.todo.dueDate || '')

// Watch editingId to initialize local state when entering edit mode
watch(() => props.editingId, (val) => {
  if (val === props.todo.id) {
    editLocalText.value = props.todo.text
    editLocalProjectId.value = props.todo.projectId || ''
    editLocalPriority.value = props.todo.priority || 'medium'
    editLocalTag.value = props.todo.tag || ''
    editLocalDueDate.value = props.todo.dueDate || ''
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
    dueDate: editLocalDueDate.value || null,
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
