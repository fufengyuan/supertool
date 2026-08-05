<template>
  <div class="flex items-center bg-base-100 border-b border-base-300 h-9 overflow-x-auto tab-scrollbar select-none">
    <div
      v-for="tab in tabStore.tabs"
      :key="tab.id"
      class="group flex items-center gap-1.5 px-3 h-full cursor-pointer border-r border-base-300 transition-colors shrink-0"
      :class="tab.id === tabStore.activeTabId
        ? 'bg-base-200 text-base-content font-medium'
        : 'text-base-content/60 hover:bg-base-200/50 hover:text-base-content'"
      @click="onTabClick(tab)"
      @mousedown.middle.prevent="onCloseTab(tab.id)"
    >
      <component
        :is="iconMap[tab.viewId]"
        v-if="tab.viewId && iconMap[tab.viewId]"
        :size="14"
        class="shrink-0 opacity-60"
        stroke-width="2"
      />
      <span class="text-xs truncate max-w-[120px]">{{ tab.label }}</span>
      <button
        class="ml-1 w-4 h-4 flex items-center justify-center rounded hover:bg-base-300 opacity-0 group-hover:opacity-100 transition-opacity"
        @click.stop="onCloseTab(tab.id)"
        title="关闭"
      >
        <SvgIcon name="x" :size="10" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRouter, useRoute } from 'vue-router'
import { useTabStore, type Tab } from '@/stores/tabStore'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import {
  IconDashboard, IconList, IconCalendarWeek, IconFolders, IconCoin,
  IconServer, IconRocket, IconNotes, IconWorld, IconDatabase,
  IconMessage, IconTool, IconClock, IconUser, IconRobot,
  IconBrain, IconHistory, IconSettings, IconCalendar, IconAlertTriangle,
  IconTerminal, IconNotebook, IconGitBranch, IconLock, IconPlug,
  IconCloud, IconTrash, IconChartBar, IconPhoto,
} from '@tabler/icons-vue'

const router = useRouter()
const route = useRoute()
const tabStore = useTabStore()

// viewId → icon component map
const iconMap: Record<string, any> = {
  'dashboard': IconDashboard, 'todo': IconList, 'weekly-report': IconCalendarWeek,
  'projects': IconFolders, 'accounting': IconCoin, 'servers': IconServer,
  'cicd': IconRocket, 'log-aggregator': IconNotes, nginx: IconWorld,
  'database': IconDatabase, alert: IconAlertTriangle,
  devtools: IconTerminal, notes: IconNotebook, git: IconGitBranch,
  mfa: IconLock, vpn: IconPlug, 'data-backup': IconCloud,
  'disk-cleaner': IconTrash, report: IconChartBar, settings: IconSettings,
  'image-processor': IconPhoto, kanban: IconCalendar,
  tools: IconTool, cron: IconClock, providers: IconServer,
  models: IconRobot, skills: IconBrain, memory: IconBrain,

}

function onTabClick(tab: Tab) {
  if (tab.id === tabStore.activeTabId) return
  tabStore.activate(tab.id)
  if (route.fullPath !== tab.currentPath) {
    router.push(tab.currentPath)
  }
}

// 关闭标签页：closeTab 返回需跳转的新路径（关闭当前 tab 时），路由必须同步跳转，
// 否则 keep-alive 的 include 虽已移除组件名，但 router-view 仍在渲染旧组件 -> 页面"没关"
function onCloseTab(id: string) {
  const newPath = tabStore.closeTab(id)
  if (newPath && route.fullPath !== newPath) {
    router.push(newPath)
  }
}
</script>

<style scoped>
.tab-scrollbar::-webkit-scrollbar { height: 0; }
</style>
