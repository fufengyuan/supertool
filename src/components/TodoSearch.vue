<template>
  <div class="todo-search">
    <!-- 搜索栏 -->
    <div class="search-bar">
      <div class="search-input-container">
        <input
          :value="searchQuery"
          @input="$emit('update:searchQuery', ($event.target as HTMLInputElement).value)"
          :placeholder="$t('todo.search')"
          class="search-input"
        />
        <button v-if="searchQuery" @click="$emit('clear-search')" class="clear-search-btn">
          ×
        </button>
      </div>
      <div class="search-filters">
        <select
          :value="priorityFilter"
          @change="$emit('update:priorityFilter', ($event.target as HTMLSelectElement).value)"
          class="filter-select"
        >
          <option value="all">{{ $t('todo.priority.all') }}</option>
          <option value="high">{{ $t('todo.priority.highLabel') }}</option>
          <option value="medium">{{ $t('todo.priority.mediumLabel') }}</option>
          <option value="low">{{ $t('todo.priority.lowLabel') }}</option>
        </select>
        <select
          :value="statusFilter"
          @change="$emit('update:statusFilter', ($event.target as HTMLSelectElement).value)"
          class="filter-select"
        >
          <option value="all">{{ $t('todo.status.all') }}</option>
          <option value="active">{{ $t('todo.filter.active') }}</option>
          <option value="completed">{{ $t('todo.filter.completed') }}</option>
        </select>
      </div>
    </div>

    <!-- 过滤选项 -->
    <div class="todo-filters">
      <button
        :class="{ active: filter === 'all' && tagFilter === 'all' }"
        @click="
          $emit('update:filter', 'all');
          $emit('update:tagFilter', 'all');
          $emit('reset-filters');
        "
      >
        {{ $t('todo.filter.all') }} ({{ totalCount }})
      </button>
      <button
        :class="{ active: filter === 'active' && tagFilter === 'all' }"
        @click="
          $emit('update:filter', 'active');
          $emit('update:tagFilter', 'all');
          $emit('reset-filters');
        "
      >
        {{ $t('todo.filter.active') }} ({{ activeCount }})
      </button>
      <button
        :class="{ active: filter === 'completed' && tagFilter === 'all' }"
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
    <div class="todo-filters">
      <button
        v-for="tag in ['all', ...tags]"
        :key="tag"
        :class="{ active: tagFilter === tag }"
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

<style scoped>
.todo-search {
  margin-bottom: 12px;
}
.search-bar {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
  align-items: center;
}
.search-input-container {
  flex: 1;
  position: relative;
}
.search-input {
  width: 100%;
  padding: 8px 32px 8px 12px;
  border: 1px solid var(--input-border);
  border-radius: 6px;
  background: var(--input-bg);
  color: var(--main-text);
  font-size: 14px;
  outline: none;
}
.search-input:focus {
  border-color: var(--primary-color);
  box-shadow: 0 0 0 2px var(--primary-light);
}
.clear-search-btn {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  width: 20px;
  height: 20px;
  border: none;
  background: none;
  color: var(--main-text-secondary);
  font-size: 16px;
  cursor: pointer;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
}
.clear-search-btn:hover {
  background: var(--border-color);
}
.search-filters {
  display: flex;
  gap: 6px;
}
.filter-select {
  padding: 6px 10px;
  border: 1px solid var(--input-border);
  border-radius: 6px;
  background: var(--input-bg);
  color: var(--main-text);
  font-size: 13px;
  cursor: pointer;
}
.todo-filters {
  display: flex;
  gap: 4px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}
.todo-filters button {
  padding: 6px 14px;
  border: 1px solid var(--border-color);
  background: var(--card-bg);
  color: var(--main-text-secondary);
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
}
.todo-filters button:hover {
  border-color: var(--primary-color);
  color: var(--primary-color);
}
.todo-filters button.active {
  background: var(--primary-color);
  color: white;
  border-color: var(--primary-color);
}
</style>
