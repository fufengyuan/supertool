<template>
  <div class="mb-3 space-y-4">
    <!-- 搜索栏 -->
    <div class="flex gap-2 items-center">
      <div class="relative flex-1">
        <input
          :value="searchQuery"
          @input="$emit('update:searchQuery', ($event.target as HTMLInputElement).value)"
          :placeholder="$t('todo.search')"
          class="input input-bordered w-full text-sm ps-8"
        />
        <SvgIcon name="search" size="14" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-base-content/40 pointer-events-none" />
        <button v-if="searchQuery" @click="$emit('clear-search')" class="absolute right-2 top-1/2 -translate-y-1/2 w-5 h-5 flex items-center justify-center border-none bg-transparent text-base-content/60 text-base cursor-pointer rounded hover:bg-base-content/10">
          ×
        </button>
      </div>
      <div class="flex gap-1.5">
        <select
          :value="priorityFilter"
          @change="$emit('update:priorityFilter', ($event.target as HTMLSelectElement).value)"
          class="select select-bordered select-sm text-sm"
        >
          <option value="all">{{ $t('todo.priority.all') }}</option>
          <option value="high">{{ $t('todo.priority.highLabel') }}</option>
          <option value="medium">{{ $t('todo.priority.mediumLabel') }}</option>
          <option value="low">{{ $t('todo.priority.lowLabel') }}</option>
        </select>
        <select
          :value="statusFilter"
          @change="$emit('update:statusFilter', ($event.target as HTMLSelectElement).value)"
          class="select select-bordered select-sm text-sm"
        >
          <option value="all">{{ $t('todo.status.all') }}</option>
          <option value="active">{{ $t('todo.filter.active') }}</option>
          <option value="completed">{{ $t('todo.filter.completed') }}</option>
        </select>
      </div>
    </div>

    <!-- 过滤选项 -->
    <div class="flex gap-1 flex-wrap">
      <button
        :class="['btn btn-sm', filter === 'all' && tagFilter === 'all' ? 'btn-primary' : 'btn-ghost']"
        @click="
          $emit('update:filter', 'all');
          $emit('update:tagFilter', 'all');
          $emit('reset-filters');
        "
      >
        {{ $t('todo.filter.all') }} ({{ totalCount }})
      </button>
      <button
        :class="['btn btn-sm', filter === 'active' && tagFilter === 'all' ? 'btn-primary' : 'btn-ghost']"
        @click="
          $emit('update:filter', 'active');
          $emit('update:tagFilter', 'all');
          $emit('reset-filters');
        "
      >
        {{ $t('todo.filter.active') }} ({{ activeCount }})
      </button>
      <button
        :class="['btn btn-sm', filter === 'completed' && tagFilter === 'all' ? 'btn-primary' : 'btn-ghost']"
        @click="
          $emit('update:filter', 'completed');
          $emit('update:tagFilter', 'all');
          $emit('reset-filters');
        "
      >
        {{ $t('todo.filter.completed') }} ({{ completedCount }})
      </button>
    </div>

    <!-- 标签过滤 -->
    <div class="flex gap-1 flex-wrap">
      <button
        v-for="tag in ['all', ...tags]"
        :key="tag"
        :class="['btn btn-sm', tagFilter === tag ? 'btn-primary' : 'btn-ghost']"
        @click="
          $emit('update:tagFilter', tag);
          if (tag !== 'all') $emit('update:filter', 'all');
        "
      >
        {{ tag === 'all' ? $t('todo.allTags') : tag }} ({{ tagCountFn(tag) }})
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'

defineProps({
  totalCount: { type: Number, default: 0 },
  activeCount: { type: Number, default: 0 },
  completedCount: { type: Number, default: 0 },
  tags: { type: Array as () => string[], default: () => [] },
  filter: { type: String, default: 'all' },
  tagFilter: { type: String, default: 'all' },
  searchQuery: { type: String, default: '' },
  priorityFilter: { type: String, default: 'all' },
  statusFilter: { type: String, default: 'all' },
  tagCountFn: { type: Function, required: true },
});

defineEmits([
  'update:searchQuery',
  'update:priorityFilter',
  'update:statusFilter',
  'update:filter',
  'update:tagFilter',
  'clear-search',
  'reset-filters',
]);
</script>
