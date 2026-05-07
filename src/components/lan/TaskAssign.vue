<template>
  <Teleport to="body">
    <div
      v-if="isVisible"
      class="task-assign-overlay"
      @click.self="handleClose"
    >
      <div class="task-assign-modal" @click.stop>
        <div class="task-assign-header">
          <h3>📤 {{ $t('lan.assignTitle', { name: peer?.name }) }}</h3>
          <button @click="handleClose" class="task-assign-close-btn" aria-label="关闭">×</button>
        </div>
        <div class="task-assign-body">
          <div class="form-field">
            <label>{{ $t('lan.taskContent') }} <span class="required">*</span></label>
            <input v-model="task.text" class="form-input" :placeholder="$t('lan.taskContentPlaceholder')" />
          </div>

          <div class="form-row">
            <div class="form-field">
              <label>{{ $t('lan.priority') }}</label>
              <select v-model="task.priority" class="form-select">
                <option value="high">🔴 {{ $t('todo.priority.high') }}</option>
                <option value="medium">🟡 {{ $t('todo.priority.medium') }}</option>
                <option value="low">🟢 {{ $t('todo.priority.low') }}</option>
              </select>
            </div>
            <div class="form-field">
              <label>{{ $t('lan.dueDate') }}</label>
              <input type="date" v-model="task.dueDate" class="form-input" />
            </div>
          </div>

          <div class="form-field">
            <label>{{ $t('lan.note') }}</label>
            <textarea
              v-model="task.note"
              class="form-textarea"
              :placeholder="$t('lan.notePlaceholder')"
              rows="3"
            ></textarea>
          </div>
        </div>

        <div class="task-assign-footer">
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

<style scoped>
/* 任务分配弹窗 — 独立 scoped 样式，不依赖全局 .form-modal-overlay
   修复 Ubuntu Linux 上 backdrop-filter 导致的事件拦截问题 */

.task-assign-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  /* backdrop-filter 在部分 Linux GPU 驱动上有渲染 bug，
     移除以避免 overlay 异常显示且事件不响应 */
  display: flex !important;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  animation: fadeIn 0.2s ease;
  pointer-events: auto;
}

.task-assign-modal {
  background: var(--card-bg);
  border-radius: 16px;
  width: 90%;
  max-width: 600px;
  max-height: 85vh;
  overflow-y: auto;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
  animation: slideUp 0.3s ease;
  pointer-events: auto;
}

.task-assign-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 18px 24px;
  border-bottom: 1px solid var(--border-color);
  position: sticky;
  top: 0;
  background: var(--card-bg);
  border-radius: 16px 16px 0 0;
  z-index: 1;
}

.task-assign-header h3 {
  margin: 0;
  font-size: 17px;
  font-weight: 600;
  color: var(--main-text);
}

.task-assign-close-btn {
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--main-text-secondary);
  font-size: 20px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
  pointer-events: auto;
}

.task-assign-close-btn:hover {
  background: var(--primary-light);
  color: var(--primary-color);
}

.task-assign-body {
  padding: 24px;
}

.task-assign-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 24px;
  border-top: 1px solid var(--border-color);
}
</style>
