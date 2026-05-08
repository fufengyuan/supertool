<template>
  <div class="project-todo-list">
    <div class="task-input-section">
      <input
        v-model="newTaskText"
        @keyup.enter="addTask"
        placeholder="添加任务到此项目..."
        class="task-input"
      />
      <button @click="addTask" class="add-task-btn">添加</button>
    </div>

    <div class="tasks-list">
      <div
        v-for="task in tasks"
        :key="task.id"
        class="task-item"
        :class="{ completed: task.completed }"
      >
        <div class="task-content">
          <input
            type="checkbox"
            :checked="task.completed"
            @change="toggleTask(task)"
            class="task-checkbox"
          />
          <span class="task-text">{{ task.text }}</span>
        </div>
        <div class="task-meta">
          <span v-if="task.dueDate" class="due-date">📅 {{ formatDate(task.dueDate) }}</span>
          <span class="priority-badge" :class="task.priority || 'medium'">
            {{ priorityLabel(task.priority) }}
          </span>
        </div>
        <button @click="deleteTask(task)" class="delete-task-btn">🗑️</button>
      </div>
    </div>

    <!-- 进度条 -->
    <div class="progress-container">
      <div class="progress-bar">
        <div
          class="progress-fill"
          :style="{ width: stats.progress + '%', backgroundColor: projectColor }"
        ></div>
      </div>
      <span class="progress-text">{{ stats.completed }} / {{ stats.total }} 任务完成</span>
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

<style scoped>
.task-input-section { display: flex; gap: 12px; margin-bottom: 20px; }
.task-input { flex: 1; padding: 12px 16px; border: 2px solid color-mix(in oklab, var(--color-base-content) 20%, transparent); border-radius: 8px; background-color: var(--color-base-200); color: var(--color-base-content); font-size: 16px; }
.add-task-btn { padding: 12px 24px; background: var(--color-primary); color: white; border: none; border-radius: 8px; cursor: pointer; font-weight: 500; transition: all 0.3s ease; }
.add-task-btn:hover { background: color-mix(in oklab, var(--color-primary) 80%, transparent); transform: translateY(-2px); }
.tasks-list { margin-bottom: 20px; }
.task-item { background: var(--color-base-100); padding: 16px; border-radius: 8px; margin-bottom: 12px; display: flex; align-items: center; gap: 12px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
.task-item.completed { opacity: 0.7; background: var(--color-base-200); }
.task-content { flex: 1; display: flex; align-items: center; gap: 12px; }
.task-checkbox { width: 18px; height: 18px; cursor: pointer; }
.task-text { flex: 1; font-size: 16px; }
.task-item.completed .task-text { text-decoration: line-through; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); }
.task-meta { display: flex; gap: 12px; align-items: center; }
.due-date { font-size: 14px; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); white-space: nowrap; }
.priority-badge { padding: 4px 8px; border-radius: 4px; font-size: 12px; font-weight: 500; white-space: nowrap; }
.priority-badge.low { background: rgba(34, 197, 94, 0.1); color: var(--color-success); }
.priority-badge.medium { background: rgba(245, 158, 11, 0.1); color: var(--color-warning); }
.priority-badge.high { background: rgba(239, 68, 68, 0.1); color: var(--color-error); }
.delete-task-btn { background: none; border: none; cursor: pointer; font-size: 16px; opacity: 0.6; transition: opacity 0.3s ease; }
.delete-task-btn:hover { opacity: 1; }
.progress-container { display: flex; align-items: center; gap: 12px; }
.progress-bar { flex: 1; height: 12px; background: var(--color-base-200); border-radius: 6px; overflow: hidden; }
.progress-fill { height: 100%; transition: width 0.3s ease; }
.progress-text { font-size: 14px; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); white-space: nowrap; }
</style>
