<template>
  <div
    class="tab-bar flex items-center gap-0 overflow-x-auto overflow-y-hidden"
    @mouseenter="hovering = true"
    @mouseleave="hovering = false"
  >
    <div
      v-for="tab in tabStore.tabs"
      :key="tab.id"
      class="tab-item group flex items-center gap-1.5 px-3 py-1.5 cursor-pointer select-none text-xs font-medium rounded-t-lg transition-all duration-150 shrink-0 border-b-2"
      :class="[
        tab.id === tabStore.activeTabId
          ? 'border-primary text-primary bg-primary/5'
          : 'border-transparent text-base-content/50 hover:text-base-content hover:bg-base-200/50 hover:border-base-content/20'
      ]"
      @click="onTabClick(tab.id)"
      @mouseup.right.prevent="onTabContextMenu(tab.id, $event)"
      :title="tab.label"
    >
      <!-- 图标 -->
      <component
        :is="iconMap[tab.viewId]"
        v-if="iconMap[tab.viewId]"
        :size="14"
        stroke-width="1.5"
        class="shrink-0"
      />
      <!-- 标签文字 -->
      <span class="truncate max-w-[120px]">{{ tab.label }}</span>
      <!-- 关闭按钮 -->
      <button
        v-if="tabStore.tabs.length > 1"
        class="close-btn flex items-center justify-center w-4 h-4 rounded-full opacity-0 group-hover:opacity-100 hover:bg-base-content/10 transition-opacity ml-0.5"
        @click.stop="onCloseTab(tab.id)"
        title="关闭标签页"
      >
        <SvgIcon name="x" size="10" />
      </button>
    </div>

    <!-- 无标签页时显示占位 -->
    <div
      v-if="tabStore.tabs.length === 0"
      class="flex items-center px-3 py-1.5 text-xs text-base-content/30"
    >
      无打开的标签页
    </div>

    <!-- 滚动指示器（右侧渐变） -->
    <div
      v-show="tabStore.tabs.length > 0 && showScrollIndicator"
      class="scroll-fade pointer-events-none absolute right-0 top-0 bottom-0 w-8 bg-gradient-to-l from-base-200 to-transparent"
    ></div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { useTabStore, VIEW_ID_TO_PATH } from '@/stores/tabStore'
import { useRouter, useRoute } from 'vue-router'
import { ref, nextTick } from 'vue'

import {
  IconLayoutDashboard,
  IconChecklist,
  IconCalendarWeek,
  IconFolder,
  IconCoin,
  IconServer,
  IconRocket,
  IconFileText,
  IconWorld,
  IconDatabase,
  IconTool,
  IconNotebook,
  IconGitBranch,
  IconLock,
  IconShieldLock,
  IconCloudDownload,
  IconTrash,
  IconBell,
  IconRobot,
} from '@tabler/icons-vue'

const iconMap: Record<string, any> = {
  'dashboard': IconLayoutDashboard,
  'todo': IconChecklist,
  'weekly-report': IconCalendarWeek,
  'projects': IconFolder,
  'accounting': IconCoin,
  'servers': IconServer,
  'cicd': IconRocket,
  'log-aggregator': IconFileText,
  'nginx': IconWorld,
  'database': IconDatabase,
  'alert': IconBell,
  'devtools': IconTool,
  'notes': IconNotebook,
  'git': IconGitBranch,
  'mfa': IconLock,
  'vpn': IconShieldLock,
  'data-backup': IconCloudDownload,
  'disk-cleaner': IconTrash,
  'agent': IconRobot,
}

const tabStore = useTabStore()
const router = useRouter()
const route = useRoute()

const hovering = ref(false)
const showScrollIndicator = ref(false)

// 点击标签页 → 激活 + 导航
function onTabClick(tabId: string) {
  const tab = tabStore.tabs.find(t => t.id === tabId)
  if (!tab) return
  tabStore.activate(tabId)
  // 如果已经是当前路由，跳过 push 避免不必要的重渲染（主因：侧边栏 20+ 个 router-link 重算 active）
  if (route.fullPath === tab.currentPath) return
  // 先更新 v-show 显示新标签，下一帧再更新路由 → 避免路由变化导致的同步重渲染卡顿
  nextTick(() => {
    router.push(tab.currentPath)
  })
}

// 右键标签页 → 关闭
function onTabContextMenu(tabId: string, _event: MouseEvent) {
  if (tabStore.tabs.length <= 1) return
  tabStore.closeTab(tabId)
  // 关闭后导航到新活跃标签页
  navigateToActiveTab()
}

// 关闭标签页
function onCloseTab(tabId: string) {
  tabStore.closeTab(tabId)
  navigateToActiveTab()
}

// 导航到当前活跃标签页的路径
function navigateToActiveTab() {
  const active = tabStore.activeTab
  if (active) {
    router.push(active.currentPath)
  } else {
    // 没有标签页了，导航到看板
    router.push('/').then(() => {
      // 在看板打开一个默认标签页
      tabStore.openOrActivate('/', '综合看板', 'dashboard')
    })
  }
}
</script>

<style scoped>
.tab-bar {
  scrollbar-width: thin;
  scrollbar-color: transparent transparent;
  position: relative;
}
.tab-bar:hover {
  scrollbar-color: oklch(0.75 0 0) transparent;
}
.tab-bar::-webkit-scrollbar {
  height: 4px;
}
.tab-bar::-webkit-scrollbar-track {
  background: transparent;
}
.tab-bar::-webkit-scrollbar-thumb {
  background-color: transparent;
  border-radius: 2px;
}
.tab-bar:hover::-webkit-scrollbar-thumb {
  background-color: oklch(0.75 0 0);
}

.close-btn {
  flex-shrink: 0;
}

/* 鼠标仅悬停在 tab-bar 上时显示关闭按钮 */
.tab-item:not(:hover) .close-btn {
  opacity: 0;
}
.tab-item:hover .close-btn {
  opacity: 0.6;
}
.tab-item:hover .close-btn:hover {
  opacity: 1;
}
</style>
