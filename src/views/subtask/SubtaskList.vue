<template>
  <div class="subtask-list" :class="{ collapsed: !expanded }">
    <div class="subtask-header" @click="toggleExpanded">
      <div class="progress-bar">
        <div class="progress-track">
          <div class="progress-fill" :style="{ width: progressPercentage + '%' }"></div>
        </div>
        <span class="progress-text">{{ completedCount }}/{{ totalCount }}</span>
      </div>
      <div class="expand-icon">{{ expanded ? '▼' : '►' }}</div>
    </div>

    <div class="subtask-content" v-show="expanded">
      <SubtaskInput v-if="showAddForm" @add="handleAdd" @cancel="showAddForm = false" />
      <button v-else @click="showAddForm = true" class="add-subtask-btn">+ 添加子任务</button>

      <div class="subtasks">
      <SubtaskItem
        v-for="subtask in subtasks"
        :key="subtask.id"
        :subtask="subtask"
        :is-editing="editingSubtasks.has(subtask.id)"
        @toggle="toggleSubtask"
        @edit="startEditSubtask"
        @save="saveSubtaskEdit"
        @cancel="cancelSubtaskEdit"
        @delete="deleteSubtask"
      />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import type { Subtask } from '../../types'
import { ref, computed, onMounted } from 'vue';
import { useErrorHandler } from '../../composables/useErrorHandler';
import { getTauriAPI } from '../../utils/tauri-api';
import SubtaskItem from './SubtaskItem.vue';
import SubtaskInput from './SubtaskInput.vue';

const { handleError } = useErrorHandler();

const props = defineProps({ todoId: { type: String, required: true } });
const emit = defineEmits(['subtask-completed']);

const subtasks = ref<Subtask[]>([]);
const expanded = ref(false);
const showAddForm = ref(false);
const editingId = ref(null);
const editText = ref('');
const editingSubtasks = ref(new Set<string>());

const completedCount = computed(() => subtasks.value.filter((s: any) => s.completed).length);
const totalCount = computed(() => subtasks.value.length);
const progressPercentage = computed(() => totalCount.value === 0 ? 0 : Math.round((completedCount.value / totalCount.value) * 100));

const api = getTauriAPI();

const loadSubtasks = async () => {
  try {
    console.log("[components/subtask/SubtaskList.vue] loadSubtasks() called");
    subtasks.value = await api.getSubtasksForTodo(props.todoId); }
  catch (error) { handleError(error, { context: '加载子任务', showToast: true }); }
};

const toggleExpanded = () => {
  expanded.value = !expanded.value;
  if (expanded.value) loadSubtasks();
};

const handleAdd = async (text: string) => {
  const newSubtask = {
    id: `subtask_${Date.now()}`, todoId: props.todoId, text, completed: false,
    orderNum: subtasks.value.length, createdAt: new Date().toISOString(), updatedAt: new Date().toISOString(),
  };
  try {
    await api.addSubtask(newSubtask);
    subtasks.value.push(newSubtask);
    showAddForm.value = false;
    await updateTodoCompletion();
  } catch (error) { handleError(error, { context: '添加子任务', showToast: true }); }
};

const toggleSubtask = async (subtask: any) => {
  try {
    console.log("[components/subtask/SubtaskList.vue] toggleSubtask() called");
    const updated = { ...subtask, completed: !subtask.completed, updatedAt: new Date().toISOString() };
    await api.updateSubtask(updated);
    subtask.completed = updated.completed;
    subtask.updatedAt = updated.updatedAt;
    await updateTodoCompletion();
    emit('subtask-completed', { todoId: props.todoId, allCompleted: progressPercentage.value === 100 });
  } catch (error) { handleError(error, { context: '更新子任务', showToast: true }); }
};

const startEditSubtask = (subtask: any) => {
  editingSubtasks.value.add(subtask.id);
  editingId.value = subtask.id;
  editText.value = subtask.text;
};

const saveSubtaskEdit = async (subtask: any) => {
  const text = editText.value.trim();
  if (!text) {
    console.log("[components/subtask/SubtaskList.vue] saveSubtaskEdit() called");
    cancelSubtaskEdit(); return; }
  const prevText = subtask.text;
  try {
    const updated = { ...subtask, text, updatedAt: new Date().toISOString() };
    await api.updateSubtask(updated);
    subtask.text = text;
    subtask.updatedAt = updated.updatedAt;
  } catch (error) {
    handleError(error, { context: '保存子任务编辑', showToast: true });
    subtask.text = prevText;
  }
  editingSubtasks.value.delete(subtask.id);
  editingId.value = null;
  editText.value = '';
};

const cancelSubtaskEdit = () => {
  if (editingId.value) {
    editingSubtasks.value.delete(editingId.value);
  }
  editingId.value = null;
  editText.value = '';
};

const deleteSubtask = async (subtask: any) => {
  try {
    console.log("[components/subtask/SubtaskList.vue] deleteSubtask() called");
    await api.deleteSubtask(subtask.id);
    const idx = subtasks.value.findIndex((s: any) => s.id === subtask.id);
    if (idx !== -1) subtasks.value.splice(idx, 1);
    await updateTodoCompletion();
  } catch (error) { handleError(error, { context: '删除子任务', showToast: true }); }
};

const updateTodoCompletion = async () => {
  try {
    console.log("[components/subtask/SubtaskList.vue] updateTodoCompletion() called");
    const allCompleted = await api.updateTodoCompletionBasedOnSubtasks(props.todoId);
    emit('subtask-completed', { todoId: props.todoId, allCompleted });
  } catch (error) { handleError(error, { context: '更新任务完成状态', showToast: true }); }
};

onMounted(() => { if (expanded.value) loadSubtasks(); });
</script>

<style scoped>
.subtask-list { margin-top: 8px; border-radius: 8px; overflow: hidden; transition: all 0.3s ease; }
.subtask-list.collapsed { opacity: 0.8; }
.subtask-header {
  display: flex; justify-content: space-between; align-items: center;
  padding: 8px 12px; background-color: var(--color-base-200); cursor: pointer; user-select: none;
}
.progress-bar { display: flex; align-items: center; gap: 8px; flex: 1; }
.progress-track {
  flex: 1; height: 12px; background-color: color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 6px; overflow: hidden; position: relative;
}
.progress-fill {
  height: 100%; background: linear-gradient(90deg, #4ade80, #22c55e);
  border-radius: 6px; transition: width 0.3s ease;
}
.progress-text {
  font-size: 12px; font-weight: 600; color: var(--color-base-content);
  min-width: 40px; text-align: right;
}
.expand-icon { font-size: 12px; margin-left: 8px; color: var(--color-base-content); }
.subtask-content {
  padding: 12px; background-color: var(--color-base-100); border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}
.add-subtask-btn {
  width: 100%; padding: 8px; background-color: var(--color-base-200); color: var(--color-base-content);
  border: 1px dashed color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 6px; cursor: pointer;
  font-size: 14px; margin-bottom: 12px;
}
.add-subtask-btn:hover { background-color: color-mix(in oklab, var(--color-success) 10%, transparent); }
.subtasks { display: flex; flex-direction: column; gap: 6px; }
</style>
