<template>
  <div class="fixed inset-0 bg-base-content/20 z-50 flex items-center justify-center" @click.self="$emit('close')">
    <div class="bg-base-100 rounded-lg shadow-xl w-96 max-h-[90vh] flex flex-col">
      <!-- Header -->
      <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
        <span class="text-sm font-medium">New Task</span>
        <button class="btn btn-sm btn-ghost btn-circle" @click="$emit('close')">
          <SvgIcon name="close" size="14" />
        </button>
      </div>

      <!-- Form -->
      <div class="flex-1 overflow-y-auto p-4 space-y-3">
        <!-- Title -->
        <div>
          <label class="text-xs text-base-content/50 block mb-1">Title *</label>
          <input
            v-model="form.title"
            type="text"
            class="input input-sm input-bordered w-full"
            placeholder="Task title"
            autofocus
            @keydown.enter="handleCreate"
          />
        </div>

        <!-- Body -->
        <div>
          <label class="text-xs text-base-content/50 block mb-1">Description</label>
          <textarea
            v-model="form.body"
            class="textarea textarea-sm textarea-bordered w-full"
            placeholder="Task description..."
            rows="4"
          ></textarea>
        </div>

        <!-- Assignee -->
        <div>
          <label class="text-xs text-base-content/50 block mb-1">Assign to</label>
          <select v-model="form.assignee" class="select select-sm select-bordered w-full">
            <option value="">Unassigned</option>
            <option v-for="a in assignees" :key="a.name" :value="a.name">
              {{ a.name }} ({{ totalAssigneeCount(a) }} tasks)
            </option>
          </select>
        </div>

        <!-- Priority -->
        <div>
          <label class="text-xs text-base-content/50 block mb-1">Priority</label>
          <select v-model="form.priority" class="select select-sm select-bordered w-full">
            <option :value="0">Normal</option>
            <option :value="1">P2 (Low)</option>
            <option :value="5">P1 (High)</option>
            <option :value="10">P0 (Urgent)</option>
          </select>
        </div>

        <!-- Parents -->
        <div>
          <label class="text-xs text-base-content/50 block mb-1">Dependencies</label>
          <input
            v-model="parentsInput"
            type="text"
            class="input input-sm input-bordered w-full"
            placeholder="Task IDs, comma-separated"
          />
        </div>

        <!-- Triage checkbox -->
        <div class="flex items-center gap-2">
          <input
            id="create-triage"
            v-model="form.triage"
            type="checkbox"
            class="checkbox checkbox-sm"
          />
          <label for="create-triage" class="text-xs text-base-content/70 cursor-pointer">
            Send to Triage (requires clarification)
          </label>
        </div>
      </div>

      <!-- Footer -->
      <div class="px-4 py-3 border-t border-base-content/10 flex items-center justify-end gap-2">
        <button class="btn btn-sm btn-ghost" @click="$emit('close')">Cancel</button>
        <button
          class="btn btn-sm btn-primary"
          :disabled="!form.title.trim()"
          @click="handleCreate"
        >
          Create
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'

const props = defineProps<{
  assignees: Array<{ name: string; counts: Record<string, number> }>
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'create', data: { title: string; body?: string; assignee?: string; parents?: string[]; priority?: number; triage?: boolean }): void
}>()

const form = ref({
  title: '',
  body: '',
  assignee: '',
  priority: 0,
  triage: false,
})

const parentsInput = ref('')

const parents = computed(() => {
  if (!parentsInput.value.trim()) {return []}
  return parentsInput.value.split(',').map(s => s.trim()).filter(Boolean)
})

function totalAssigneeCount(a: { counts: Record<string, number> }): number {
  return Object.values(a.counts || {}).reduce((sum, c) => sum + c, 0)
}

function handleCreate() {
  if (!form.value.title.trim()) {return}
  emit('create', {
    title: form.value.title.trim(),
    body: form.value.body.trim() || undefined,
    assignee: form.value.assignee || undefined,
    parents: parents.value.length > 0 ? parents.value : undefined,
    priority: form.value.priority || undefined,
    triage: form.value.triage || undefined,
  })
}
</script>
