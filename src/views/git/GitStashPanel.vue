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
        <button class="btn btn-ghost btn-xs" @click="$emit('update:selectedStash', null)" title="关闭">✕</button>
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
