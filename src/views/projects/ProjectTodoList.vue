<template>
  <div>
    <div class="flex gap-3 mb-5">
      <input
        v-model="newTaskText"
        @keyup.enter="addTask"
        placeholder="添加任务到此项目..."
        class="input input-bordered flex-1 text-lg"
      />
      <button @click="addTask" class="btn btn-primary px-6 py-3 font-medium">添加</button>
    </div>

    <div class="mb-5">
      <div
        v-for="task in tasks"
        :key="task.id"
        class="card bg-base-200 p-4 mb-3 flex items-center gap-3"
        :class="task.completed ? 'opacity-70' : ''"
      >
        <div class="flex-1 flex items-center gap-3">
          <input
            type="checkbox"
            :checked="task.completed"
            @change="toggleTask(task)"
            class="checkbox checkbox-sm cursor-pointer"
          />
          <span class="text-lg" :class="task.completed ? 'line-through text-base-content/50' : ''">{{ task.text }}</span>
        </div>
        <div class="flex gap-3 items-center">
          <span v-if="task.dueDate" class="text-sm text-base-content/70 whitespace-nowrap"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><rect x="3" y="4" width="18" height="18" rx="2" ry="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/></svg> {{ formatDate(task.dueDate) }}</span>
          <span class="text-xs font-medium px-2 py-1 rounded whitespace-nowrap"
            :class="task.priority === 'low' ? 'bg-success/10 text-success' : task.priority === 'high' ? 'bg-error/10 text-error' : 'bg-warning/10 text-warning'">
            {{ priorityLabel(task.priority) }}
          </span>
        </div>
        <button @click="deleteTask(task)" class="btn btn-ghost btn-xs opacity-60 hover:opacity-100"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg></button>
      </div>
    </div>

    <!-- 进度条 -->
    <div class="flex items-center gap-3">
      <progress class="progress progress-primary flex-1 h-3" :value="stats.progress" max="100"></progress>
      <span class="text-sm text-base-content/70 whitespace-nowrap">{{ stats.completed }} / {{ stats.total }} 任务完成</span>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref } from 'vue'

const props = defineProps({
  project: { type: Object, required: true },
  tasks: { type: Array as () => any[], required: true },
  stats: { type: Object, default: () => ({ total: 0, completed: 0, progress: 0 }) },
  projectColor: { type: String, default: '#4f46e5' }
})

const emit = defineEmits(['task-added', 'task-toggled', 'task-deleted', 'data-reload'])

const newTaskText = ref('')

const formatDate = (dateString) => {
  if (!dateString) return ''
  return new Date(dateString).toLocaleDateString('zh-CN')
}

const priorityLabel = (priority) => {
  switch (priority) {
    case 'low': return '低'
    case 'medium': return '中'
    case 'high': return '高'
    default: return '中'
  }
}

const addTask = async () => {
  const text = newTaskText.value.trim()
  if (!text) return
  emit('task-added', text)
  newTaskText.value = ''
}

const toggleTask = (task) => { emit('task-toggled', task) }
const deleteTask = (task) => { emit('task-deleted', task) }
</script>
