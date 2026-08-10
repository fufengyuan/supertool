<template>
  <div class="mt-2 rounded-xl overflow-hidden transition-all duration-300" :class="{ 'opacity-80': !expanded }">
    <div class="flex justify-between items-center p-2 px-3 bg-base-200 cursor-pointer select-none" @click="toggleExpanded">
      <div class="flex items-center gap-2 flex-1">
        <div class="flex-1 h-3 bg-base-content/10 rounded-full overflow-hidden relative">
          <div class="h-full bg-gradient-to-r from-green-400 to-green-500 rounded-full transition-all duration-300" :style="{ width: progressPercentage + '%' }"></div>
        </div>
        <span class="text-xs font-semibold text-base-content min-w-[40px] text-right">{{ completedCount }}/{{ totalCount }}</span>
      </div>
      <div class="text-xs ms-2 text-base-content">{{ expanded ? '▼' : '►' }}</div>
    </div>

    <div class="p-3 bg-base-100 border-t border-base-content/10" v-show="expanded">
      <SubtaskInput v-if="showAddForm" @add="handleAdd" @cancel="showAddForm = false" />
      <button v-else @click="showAddForm = true" class="btn btn-ghost w-full border border-dashed border-base-content/10 mb-3">+ 添加子任务</button>

      <div class="flex flex-col gap-1.5">
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

<script setup lang="ts">
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
    subtasks.value = await api.getSubtasksForTodo(props.todoId); }
  catch (error) { handleError(error, { context: '加载子任务', showToast: true }); }
};

const toggleExpanded = () => {
  expanded.value = !expanded.value;
  if (expanded.value) {loadSubtasks();}
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
    await api.deleteSubtask(subtask.id);
    const idx = subtasks.value.findIndex((s: any) => s.id === subtask.id);
    if (idx !== -1) {subtasks.value.splice(idx, 1);}
    await updateTodoCompletion();
  } catch (error) { handleError(error, { context: '删除子任务', showToast: true }); }
};

const updateTodoCompletion = async () => {
  try {
    const allCompleted = await api.updateTodoCompletionBasedOnSubtasks(props.todoId);
    emit('subtask-completed', { todoId: props.todoId, allCompleted });
  } catch (error) { handleError(error, { context: '更新任务完成状态', showToast: true }); }
};

onMounted(() => { if (expanded.value) {loadSubtasks();} });
</script>
