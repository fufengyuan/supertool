<template>
  <div class="stash-panel">
    <div class="panel-header">
      <span class="panel-title">
        Stash
        <span class="change-count" v-if="stashList.length > 0">{{ stashList.length }}</span>
      </span>
      <button class="btn btn-ghost btn-xs" @click="$emit('open-stash-save')" title="Stash Changes">
        <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
        </svg>
      </button>
    </div>
    <div class="stash-list">
      <div
        v-for="stash in stashList"
        :key="stash.name"
        class="stash-item"
        :class="{ selected: selectedStash?.name === stash.name }"
        @click="$emit('select-stash', stash)"
        @contextmenu.prevent="$emit('stash-context-menu', { event: $event, stash })"
      >
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
          <line x1="3" y1="9" x2="21" y2="9" />
        </svg>
        <span class="stash-name">{{ stash.name }}</span>
        <span class="stash-desc" :title="stash.description">{{ stash.description }}</span>
      </div>
      <div v-if="stashList.length === 0 && !loading" class="stash-empty">
        <p>没有 Stash</p>
      </div>
    </div>
    <!-- Stash 预览 -->
    <div v-if="selectedStash" class="stash-preview">
      <div class="detail-header">
        <span class="detail-title">Stash 预览</span>
        <button class="btn btn-ghost btn-xs" @click="$emit('update:selectedStash', null)" title="关闭"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" class="inline-block"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg></button>
      </div>
      <pre class="diff-content">{{ stashShowContent || '加载中...' }}</pre>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  stashList: any[]
  selectedStash: any | null
  stashShowContent: string
  loading: boolean
}>()

defineEmits<{
  'update:selectedStash': [value: any | null]
  'select-stash': [stash: any]
  'stash-context-menu': [payload: { event: MouseEvent; stash: any }]
  'open-stash-save': []
}>()
</script>

<style>
/* ===================== Stash 面板 — 从 GitManager.vue 提取 ===================== */
.stash-panel {
  display: flex;
  flex-direction: column;
  border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  max-height: 40%;
  flex-shrink: 0;
  background: var(--color-base-100);
}

.stash-list {
  flex: 1;
  overflow-y: auto;
  max-height: 180px;
}

.stash-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  cursor: pointer;
  transition: background 0.1s;
  font-size: 12px;
}

.stash-item:hover {
  background: var(--hover-bg);
}

.stash-item.selected {
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
}

.stash-item svg {
  flex-shrink: 0;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.stash-name {
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  font-size: 11px;
  color: var(--color-primary);
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
  padding: 1px 4px;
  border-radius: 2px;
  flex-shrink: 0;
}

.stash-desc {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.stash-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-size: 12px;
}

.stash-preview {
  border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  max-height: 200px;
  overflow-y: auto;
}

/* ===================== 面板头部 ===================== */
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 10px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  background: var(--color-base-100);
  flex-shrink: 0;
}

.panel-title {
  font-weight: 600;
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 6px;
}

/* ===================== Stash 预览 ===================== */
.detail-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 10px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.detail-title {
  font-weight: 600;
  font-size: 12px;
}

.diff-content {
  background: var(--color-base-200);
  padding: 10px;
  border-radius: 4px;
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  font-size: 11px;
  line-height: 1.5;
  overflow-x: auto;
  max-height: 300px;
  white-space: pre-wrap;
  color: var(--color-base-content);
}

.diff-content::-webkit-scrollbar {
  width: 6px;
}

.diff-content::-webkit-scrollbar-track {
  background: transparent;
}

.diff-content::-webkit-scrollbar-thumb {
  background: color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 3px;
}

.diff-content::-webkit-scrollbar-thumb:hover {
  background: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}
</style>
