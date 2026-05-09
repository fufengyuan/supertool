<template>
  <Teleport to="body">
    <div
      v-if="isVisible"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-[10000] animate-[fadeIn_0.2s_ease]"
      @click.self="handleClose"
    >
      <div
        class="bg-base-100 rounded-2xl w-[90%] max-w-[600px] max-h-[85vh] overflow-y-auto shadow-2xl animate-[slideUp_0.3s_ease]"
        @click.stop
      >
        <div class="flex items-center justify-between p-[18px_24px] border-b border-base-content/10 sticky top-0 bg-base-100 rounded-t-2xl z-10">
          <h3 class="m-0 text-[17px] font-semibold text-base-content">📤 {{ $t('lan.assignTitle', { name: peer?.name }) }}</h3>
          <button @click="handleClose" class="w-8 h-8 border-none rounded-lg bg-transparent text-base-content/60 text-xl cursor-pointer flex items-center justify-center transition-all duration-150 hover:bg-primary/10 hover:text-primary" aria-label="关闭">×</button>
        </div>
        <div class="p-6">
          <div class="mb-4">
            <label class="label"><span class="label-text">{{ $t('lan.taskContent') }} <span class="text-error">*</span></span></label>
            <input v-model="task.text" class="input input-bordered w-full" :placeholder="$t('lan.taskContentPlaceholder')" />
          </div>

          <div class="flex gap-4">
            <div class="flex-1">
              <label class="label"><span class="label-text">{{ $t('lan.priority') }}</span></label>
              <select v-model="task.priority" class="select select-bordered w-full">
                <option value="high"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="6" fill="currentColor"/></svg> {{ $t('todo.priority.high') }}</option>
                <option value="medium"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="6" fill="currentColor"/></svg> {{ $t('todo.priority.medium') }}</option>
                <option value="low"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="6" fill="currentColor"/></svg> {{ $t('todo.priority.low') }}</option>
              </select>
            </div>
            <div class="flex-1">
              <label class="label"><span class="label-text">{{ $t('lan.dueDate') }}</span></label>
              <input type="date" v-model="task.dueDate" class="input input-bordered w-full" />
            </div>
          </div>

          <div class="mt-4">
            <label class="label"><span class="label-text">{{ $t('lan.note') }}</span></label>
            <textarea
              v-model="task.note"
              class="textarea textarea-bordered w-full"
              :placeholder="$t('lan.notePlaceholder')"
              rows="3"
            ></textarea>
          </div>
        </div>

        <div class="flex justify-end gap-3 p-4 border-t border-base-content/10">
          <button class="btn btn-ghost" @click="handleClose">{{ $t('lan.cancel') }}</button>
          <button class="btn btn-primary" @click="assign">{{ $t('lan.send') }}</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">// @ts-nocheck
console.log("[components/lan/TaskAssign.vue] component loaded")
import { ref, computed } from 'vue';
import { getTauriAPI } from '../../utils/tauri-api'

const props = defineProps<{
  peer?: { id: string; name: string } | null;
}>();

const emit = defineEmits<{
  close: [];
  assigned: [task: { text: string; priority: string; dueDate: string; note: string }];
}>();

const task = ref({
  text: '',
  priority: 'medium',
  dueDate: '',
  note: '',
});

// 双重防护：只有在 peer 明确存在时才渲染，避免 Ubuntu Linux 上的渲染异常
const isVisible = computed(() => !!props.peer);

function handleClose() {
  emit('close');
}

async function assign() {
  if (!task.value.text.trim() || !props.peer) return;

  // 发送任务
  await getTauriAPI().assignTask(props.peer.id, {
    text: task.value.text,
    priority: task.value.priority,
    dueDate: task.value.dueDate,
    note: task.value.note,
  });

  emit('assigned', task.value);
  emit('close');

  // 清空表单
  task.value = {
    text: '',
    priority: 'medium',
    dueDate: '',
    note: '',
  };
}
</script>
