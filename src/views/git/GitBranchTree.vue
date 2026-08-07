<template>
  <div class="branch-tree-panel">
    <div class="panel-header">
      <span class="panel-title">
        <SvgIcon name="gitBranch" :size="12" class="text-primary" />
        分支
      </span>
      <button class="btn btn-ghost btn-xs" @click="$emit('open-branches')" title="分支管理">
        <SvgIcon name="plus" :size="12" />
      </button>
    </div>

    <div class="branch-tree-content">
      <!-- 本地分支 -->
      <div class="branch-group">
        <div class="group-header" @click="localCollapsed = !localCollapsed">
          <SvgIcon name="chevronDown" :size="12" class="group-arrow" :class="{ collapsed: localCollapsed }" />
          <span class="group-label">本地分支 ({{ localBranches.length }})</span>
        </div>
        <div v-show="!localCollapsed" class="group-list">
          <div
            v-for="b in localBranches"
            :key="b.name"
            class="branch-tree-item"
            :class="{ 'branch-current': b.current, 'branch-selected': selectedBranch === b.name }"
            :title="b.current ? '当前分支' : '双击切换，单击筛选日志'"
            @click="onSelectBranch(b)"
            @dblclick="onCheckoutBranch(b)"
            @contextmenu.prevent="$emit('branch-context-menu', { event: $event, branch: b, isRemote: false })"
          >
            <SvgIcon name="gitBranch" :size="12" class="shrink-0" :class="b.current ? 'text-primary' : 'text-base-content/40'" />
            <span class="branch-name">{{ b.name }}</span>
            <span v-if="b.ahead > 0 || b.behind > 0" class="branch-badges">
              <span v-if="b.ahead > 0" class="badge ahead">↑{{ b.ahead }}</span>
              <span v-if="b.behind > 0" class="badge behind">↓{{ b.behind }}</span>
            </span>
            <SvgIcon v-if="b.current" name="star" :size="10" class="text-yellow-500 shrink-0" />
          </div>
        </div>
      </div>

      <!-- 远程分支 -->
      <div class="branch-group" v-if="remoteBranches.length > 0">
        <div class="group-header" @click="remoteCollapsed = !remoteCollapsed">
          <SvgIcon name="chevronDown" :size="12" class="group-arrow" :class="{ collapsed: remoteCollapsed }" />
          <span class="group-label">远程分支 ({{ remoteBranches.length }})</span>
        </div>
        <div v-show="!remoteCollapsed" class="group-list">
          <div
            v-for="b in remoteBranches"
            :key="b.name"
            class="branch-tree-item"
            :class="{ 'branch-selected': selectedBranch === b.name }"
            :title="'双击检出为本地分支'"
            @click="onSelectBranch(b)"
            @dblclick="$emit('checkout-remote-branch', b.name)"
            @contextmenu.prevent="$emit('branch-context-menu', { event: $event, branch: b, isRemote: true })"
          >
            <SvgIcon name="globe" :size="12" class="shrink-0 text-base-content/40" />
            <span class="branch-name">{{ b.name }}</span>
          </div>
        </div>
      </div>

      <div v-if="localBranches.length === 0 && remoteBranches.length === 0" class="branch-empty">
        <p>没有分支</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'

defineProps<{
  localBranches: any[]
  remoteBranches: any[]
  currentBranch: string
  selectedBranch: string | null
}>()

const emit = defineEmits<{
  'open-branches': []
  'checkout-branch': [name: string]
  'checkout-remote-branch': [name: string]
  'select-branch': [name: string | null]
  'branch-context-menu': [payload: { event: MouseEvent; branch: any; isRemote: boolean }]
}>()

const localCollapsed = ref(false)
const remoteCollapsed = ref(true)

function onSelectBranch(b: any) {
  if (b.current) { return }
  emit('select-branch', b.name)
}

function onCheckoutBranch(b: any) {
  if (b.current) { return }
  emit('checkout-branch', b.name)
}
</script>

<style>
.branch-tree-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-base-100);
  border-right: 1px solid color-mix(in oklab, var(--color-base-content) 8%, transparent);
}

.branch-tree-content {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}

.branch-group {
  padding: 2px 0;
}

.group-header {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  cursor: pointer;
  user-select: none;
}
.group-header:hover {
  background: var(--color-base-200);
}

.group-arrow {
  color: color-mix(in oklab, var(--color-base-content) 50%, transparent);
  transition: transform 0.15s;
  flex-shrink: 0;
}
.group-arrow.collapsed {
  transform: rotate(-90deg);
}

.group-label {
  font-size: 11px;
  font-weight: 600;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.group-list {
  padding: 0 4px;
}

.branch-tree-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.12s;
  min-height: 24px;
}
.branch-tree-item:hover {
  background: var(--color-base-200);
}
.branch-tree-item.branch-current {
  background: color-mix(in oklab, var(--color-primary) 8%, transparent);
}
.branch-tree-item.branch-selected {
  background: color-mix(in oklab, var(--color-primary) 12%, transparent);
}

.branch-name {
  font-size: 12px;
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.branch-badges {
  display: inline-flex;
  gap: 2px;
  flex-shrink: 0;
}
.badge {
  font-size: 9px;
  padding: 0 4px;
  border-radius: 3px;
  font-weight: 600;
}
.badge.ahead {
  background: color-mix(in oklab, #22c55e 15%, transparent);
  color: #22c55e;
}
.badge.behind {
  background: color-mix(in oklab, #f59e0b 15%, transparent);
  color: #f59e0b;
}

.branch-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  color: color-mix(in oklab, var(--color-base-content) 45%, transparent);
  font-size: 11px;
}
</style>
