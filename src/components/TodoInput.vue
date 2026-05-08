<template>
  <div class="todo-input">
    <div class="input-row">
      <input
        v-model="newTodo"
        @keyup.enter="addTodo"
        :placeholder="$t('todo.addNew')"
        class="todo-input-field"
      />
      <select v-model="newPriority" class="priority-select">
        <option value="low">{{ $t('todo.priority.low') }}</option>
        <option value="medium">{{ $t('todo.priority.medium') }}</option>
        <option value="high">{{ $t('todo.priority.high') }}</option>
      </select>
    </div>
    <div class="input-row">
      <input v-model="newDueDate" type="date" class="todo-input-field" :placeholder="$t('todo.dueDate')" />
      <select v-model="newTag" class="priority-select">
        <option value="">{{ $t('todo.tag') }}</option>
        <option v-for="tag in tags" :key="tag" :value="tag">{{ tag }}</option>
        <option value="custom">{{ $t('todo.custom') }}</option>
      </select>
    </div>
    <div class="input-row">
      <select v-model="newProjectId" class="priority-select">
        <option value="">{{ $t('todo.noProject') }}</option>
        <option v-for="project in projects" :key="project.id" :value="project.id">
          {{ project.name }}
        </option>
      </select>
    </div>
    <div class="input-row">
      <select v-model="newRepeatType" class="priority-select">
        <option value="">{{ $t('todo.repeat.none') }}</option>
        <option value="daily">{{ $t('todo.repeat.daily') }}</option>
        <option value="weekly">{{ $t('todo.repeat.weekly') }}</option>
        <option value="monthly">{{ $t('todo.repeat.monthly') }}</option>
        <option value="yearly">{{ $t('todo.repeat.yearly') }}</option>
        <option value="custom">{{ $t('todo.repeat.custom') }}</option>
      </select>
      <input
        v-if="newRepeatType === 'custom'"
        v-model="newRepeatInterval"
        type="number"
        min="1"
        class="todo-input-field"
        :placeholder="$t('todo.repeat.interval')"
      />
    </div>
    <textarea
      v-model="newDescription"
      :placeholder="$t('todo.taskDescription')"
      class="todo-input-field textarea"
      rows="2"
    ></textarea>
    <button @click="addTodo" class="add-btn">{{ $t('todo.addTask') }}</button>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

const props = defineProps({
  tags: {
    type: Array as () => string[],
    default: () => [],
  },
  projects: {
    type: Array as () => { id: string; name: string }[],
    default: () => [],
  },
});

const emit = defineEmits(['add']);

const newTodo = ref('');
const newPriority = ref('medium');
const newDueDate = ref('');
const newDescription = ref('');
const newTag = ref('');
const newProjectId = ref('');
const newRepeatType = ref('');
const newRepeatInterval = ref(1);

const addTodo = () => {
  const text = newTodo.value.trim();
  if (!text) return;

  const newTodoObj = {
    id: crypto.randomUUID(),
    text,
    completed: false,
    priority: newPriority.value,
    dueDate: newDueDate.value || null,
    description: newDescription.value || '',
    markdownDescription: '',
    tag: newTag.value || t('todo.uncategorized'),
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    repeatType: newRepeatType.value,
    repeatInterval: newRepeatType.value === 'custom' ? newRepeatInterval.value || 1 : 1,
    repeatEndDate: null,
    repeatCount: -1,
    parentTodoId: null,
    projectId: newProjectId.value || null,
  };

  emit('add', newTodoObj);

  // 重置表单
  newTodo.value = '';
  newPriority.value = 'medium';
  newDueDate.value = '';
  newDescription.value = '';
  newTag.value = '';
  newProjectId.value = '';
  newRepeatType.value = '';
  newRepeatInterval.value = 1;
};
</script>

<style scoped>
.todo-input {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 20px;
  background: oklch(var(--b1));
  padding: 16px;
  border-radius: 10px;
  border: 1px solid oklch(var(--bc) / 0.1);
}
.input-row {
  display: grid;
  grid-template-columns: 1fr auto auto;
  gap: 8px;
  align-items: center;
}
.todo-input-field {
  padding: 8px 12px;
  border: 1px solid oklch(var(--bc) / 0.2);
  border-radius: 6px;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  font-size: 14px;
  outline: none;
}
.todo-input-field:focus {
  border-color: oklch(var(--p));
  box-shadow: 0 0 0 2px oklch(var(--p) / 0.1);
}
.todo-input-field.textarea {
  resize: vertical;
  min-height: 36px;
}
.priority-select {
  padding: 8px 10px;
  border: 1px solid oklch(var(--bc) / 0.2);
  border-radius: 6px;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  font-size: 13px;
  cursor: pointer;
}
.add-btn {
  align-self: flex-end;
  padding: 8px 20px;
  background: oklch(var(--p));
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
}
.add-btn:hover {
  background: oklch(var(--p) / 0.8);
  transform: translateY(-1px);
}
</style>
