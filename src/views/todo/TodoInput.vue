<template>
  <div class="flex flex-col gap-2 mb-5 bg-base-100 p-4 rounded-xl border border-base-content/10">
    <div class="grid grid-cols-[1fr_auto_auto] gap-2 items-center">
      <input
        v-model="newTodo"
        @keyup.enter="addTodo"
        :placeholder="$t('todo.addNew')"
        class="input input-bordered w-full text-sm"
      />
      <select v-model="newPriority" class="select select-bordered text-sm">
        <option value="low">{{ $t('todo.priority.low') }}</option>
        <option value="medium">{{ $t('todo.priority.medium') }}</option>
        <option value="high">{{ $t('todo.priority.high') }}</option>
      </select>
    </div>
    <div class="grid grid-cols-[1fr_auto_auto] gap-2 items-center">
      <input v-model="newDueDate" type="date" class="input input-bordered w-full text-sm" :placeholder="$t('todo.dueDate')" />
      <select v-model="newTag" class="select select-bordered text-sm">
        <option value="">{{ $t('todo.tag') }}</option>
        <option v-for="tag in tags" :key="tag" :value="tag">{{ tag }}</option>
        <option value="custom">{{ $t('todo.custom') }}</option>
      </select>
    </div>
    <div class="grid grid-cols-[1fr_auto_auto] gap-2 items-center">
      <select v-model="newProjectId" class="select select-bordered text-sm col-span-full">
        <option value="">{{ $t('todo.noProject') }}</option>
        <option v-for="project in projects" :key="project.id" :value="project.id">
          {{ project.name }}
        </option>
      </select>
    </div>
    <div class="grid grid-cols-[1fr_auto_auto] gap-2 items-center">
      <select v-model="newRepeatType" class="select select-bordered text-sm">
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
        class="input input-bordered w-full text-sm"
        :placeholder="$t('todo.repeat.interval')"
      />
    </div>
    <textarea
      v-model="newDescription"
      :placeholder="$t('todo.taskDescription')"
      class="textarea textarea-bordered w-full text-sm"
      rows="2"
    ></textarea>
    <button @click="addTodo" class="btn btn-primary self-end">{{ $t('todo.addTask') }}</button>
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
  if (!text) {return;}

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
