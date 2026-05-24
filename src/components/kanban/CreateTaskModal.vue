<template>
  <div class="fixed inset-0 bg-base-content/20 z-50 flex items-center justify-center">
    <div class="bg-base-100 rounded-lg shadow-xl w-96 max-h-[90vh] flex flex-col">
      <!-- Header -->
      <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
        <span class="text-sm font-medium">新建任务</span>
        <button class="btn btn-sm btn-ghost btn-circle" @click="$emit('close')">
          <SvgIcon name="close" size="14" />
        </button>
      </div>

      <!-- Form -->
      <div class="flex-1 overflow-y-auto p-4 space-y-3">
        <!-- Title -->
        <div>
          <label class="text-xs text-base-content/50 block mb-1">标题 *</label>
          <input 
            v-model="form.title"
            type="text"
            class="input input-sm input-bordered w-full"
            placeholder="任务标题"
          />
        </div>

        <!-- Body -->
        <div>
          <label class="text-xs text-base-content/50 block mb-1">描述</label>
          <textarea 
            v-model="form.body"
            class="textarea textarea-sm textarea-bordered w-full"
            placeholder="任务详细描述..."
            rows="4"
          ></textarea>
        </div>

        <!-- Assignee -->
        <div>
          <label class="text-xs text-base-content/50 block mb-1">分配给</label>
          <select v-model="form.assignee" class="select select-sm select-bordered w-full">
            <option value="">未分配（手动分配）</option>
            <option v-for="a in assignees" :key="a.profile" :value="a.profile">
              {{ a.profile }} ({{ a.count }} 个任务)
            </option>
          </select>
        </div>

        <!-- Parents -->
        <div>
          <label class="text-xs text-base-content/50 block mb-1">依赖任务</label>
          <input 
            v-model="parentsInput"
            type="text"
            class="input input-sm input-bordered w-full"
            placeholder="任务 ID，多个用逗号分隔"
          />
        </div>
      </div>

      <!-- Footer -->
      <div class="px-4 py-3 border-t border-base-content/10 flex items-center justify-end gap-2">
        <button class="btn btn-sm btn-ghost" @click="$emit('close')">取消</button>
        <button 
          class="btn btn-sm btn-primary"
          :disabled="!form.title.trim()"
          @click="handleCreate"
        >
          创建
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import SvgIcon from '@/components/ui/SvgIcon.vue';

const props = defineProps<{
  assignees: Array<{ profile: string; count: number }>;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'create', data: { title: string; body?: string; assignee?: string; parents?: string[] }): void;
}>();

const form = ref({
  title: '',
  body: '',
  assignee: '',
});

const parentsInput = ref('');

const parents = computed(() => {
  if (!parentsInput.value.trim()) return [];
  return parentsInput.value.split(',').map(s => s.trim()).filter(Boolean);
});

function handleCreate() {
  if (!form.value.title.trim()) return;
  
  emit('create', {
    title: form.value.title.trim(),
    body: form.value.body.trim() || undefined,
    assignee: form.value.assignee || undefined,
    parents: parents.value.length > 0 ? parents.value : undefined,
  });
}
</script>