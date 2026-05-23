<template>
  <div>
    <div class="flex gap-3 mb-5">
      <input
        v-model="newTaskText"
        @keyup.enter="addTask"
        placeholder="添加任务到此项目..."
        class="input input-bordered flex-1"
      />
      <button @click="addTask" class="btn btn-primary px-6 font-medium">添加</button>
    </div>

    <div v-if="tasks.length === 0" class="text-center px-5 py-10 text-base-content/60">
      <SvgIcon name="file" :size="32" class="mx-auto mb-2 opacity-50" />
      <p class="text-sm m-0">暂无进行中的任务</p>
    </div>
    <div v-else class="flex flex-col gap-2 mb-5">
      <div
        v-for="task in tasks"
        :key="task.id"
        class="flex items-center justify-between p-3 bg-base-200 rounded-lg gap-3 transition-all duration-150 hover:bg-base-300/60"
      >
        <div class="flex items-center gap-2.5 flex-1 min-w-0">
          <input
            type="checkbox"
            :checked="task.completed"
            @change="toggleTask(task)"
            class="checkbox checkbox-primary checkbox-sm shrink-0"
          />
          <span class="text-sm text-base-content">{{ task.text }}</span>
        </div>
        <div class="flex items-center gap-2.5 shrink-0">
          <span v-if="task.dueDate" class="text-xs text-base-content/60 whitespace-nowrap"><SvgIcon name="calendar" :size="14" class="inline-block align-text-bottom" /> {{ formatDate(task.dueDate) }}</span>
          <span class="text-xs font-medium px-2 py-0.5 rounded whitespace-nowrap"
            :class="task.priority === 'low' ? 'bg-success/10 text-success' : task.priority === 'high' ? 'bg-error/10 text-error' : 'bg-warning/10 text-warning'">
            {{ priorityLabel(task.priority) }}
          </span>
          <button @click="deleteTask(task)" class="btn btn-ghost btn-xs opacity-50 hover:opacity-100" title="删除任务"><SvgIcon name="trash" :size="14" class="inline-block align-text-bottom" /></button>
        </div>
      </div>
    </div>

    <!-- 进度条 -->
    <div class="flex items-center gap-3 p-3 bg-base-200 rounded-lg" v-if="stats.total > 0">
      <div class="flex-1 h-2 bg-base-300 rounded-full overflow-hidden">
        <div class="h-full rounded-full transition-all duration-300" :style="{ width: stats.progress + '%', backgroundColor: projectColor }"></div>
      </div>
      <span class="text-xs text-base-content/60 whitespace-nowrap">{{ stats.completed }} / {{ stats.total }} 任务完成</span>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import SvgIcon from '@/components/ui/SvgIcon.vue'
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
  if (!dateString) {return ''}
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
  if (!text) {return}
  emit('task-added', text)
  newTaskText.value = ''
}

const toggleTask = (task) => { emit('task-toggled', task) }
const deleteTask = (task) => { emit('task-deleted', task) }
</script>
