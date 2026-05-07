import { computed } from 'vue';
import type { ComputedRef } from 'vue';

interface TodoStoreFilters {
  filter: string;
  tagFilter: string;
  searchQuery: string;
  priorityFilter: string;
  statusFilter: string;
  tagCounts: Record<string, number>;
  setFilter: (val: string) => void;
  setTagFilter: (val: string) => void;
  setSearchQuery: (val: string) => void;
  setPriorityFilter: (val: string) => void;
  setStatusFilter: (val: string) => void;
  resetFilters: () => void;
}

/**
 * useTodoFilters - 抽取搜索、筛选、排序逻辑
 * 管理与 todoStore 同步的筛选状态代理和计数
 */
export function useTodoFilters(todoStore: TodoStoreFilters) {
  const filterProxy: ComputedRef<string> = computed({
    get: () => todoStore.filter,
    set: (val: string) => todoStore.setFilter(val),
  });

  const tagFilterProxy: ComputedRef<string> = computed({
    get: () => todoStore.tagFilter,
    set: (val: string) => todoStore.setTagFilter(val),
  });

  const searchQueryProxy: ComputedRef<string> = computed({
    get: () => todoStore.searchQuery,
    set: (val: string) => todoStore.setSearchQuery(val),
  });

  const priorityFilterProxy: ComputedRef<string> = computed({
    get: () => todoStore.priorityFilter,
    set: (val: string) => todoStore.setPriorityFilter(val),
  });

  const statusFilterProxy: ComputedRef<string> = computed({
    get: () => todoStore.statusFilter,
    set: (val: string) => todoStore.setStatusFilter(val),
  });

  const clearSearch = (): void => {
    todoStore.setSearchQuery('');
  };

  const resetFilters = (): void => {
    todoStore.resetFilters();
  };

  const tagCount = (tag: string): number => {
    return todoStore.tagCounts[tag] || 0;
  };

  return {
    filterProxy,
    tagFilterProxy,
    searchQueryProxy,
    priorityFilterProxy,
    statusFilterProxy,
    clearSearch,
    resetFilters,
    tagCount,
  };
}
