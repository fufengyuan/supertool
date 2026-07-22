<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition-opacity duration-150 ease"
      leave-active-class="transition-opacity duration-150 ease"
      enter-from-class="opacity-0"
      leave-to-class="opacity-0"
    >
      <div
        v-if="isOpen"
        class="fixed inset-0 bg-black/50 flex items-start justify-center z-[10001] pt-[15vh] [-webkit-app-region:no-drag]"
        @mousedown.self="close"
      >
        <div class="w-[520px] max-h-[60vh] bg-base-100 border border-base-content/10 rounded-2xl shadow-[0_20px_60px_rgba(0,0,0,0.3)] overflow-hidden flex flex-col">
          <!-- 搜索框 -->
          <div class="flex items-center gap-3 px-4 py-3 border-b border-base-content/10">
            <SvgIcon name="search" size="16" class="text-base-content/60 shrink-0" />
            <input
              ref="inputRef"
              v-model="query"
              placeholder="搜索开发工具..."
              class="flex-1 bg-transparent outline-none text-sm text-base-content placeholder-base-content/40"
              @keydown.down.prevent="navigate(1)"
              @keydown.up.prevent="navigate(-1)"
              @keydown.enter.prevent="select"
              @keydown.esc.prevent="close"
            />
            <kbd class="text-[10px] px-1.5 py-0.5 rounded bg-base-200 border border-base-content/10 text-base-content/50 shrink-0">ESC</kbd>
          </div>

          <!-- 搜索结果 -->
          <div class="flex-1 overflow-y-auto p-2">
            <!-- 无搜索词时：显示收藏 + 最近使用 -->
            <template v-if="!query">
              <div v-if="favoriteTools.length > 0" class="mb-1">
                <div class="text-[10px] font-semibold text-base-content/50 uppercase tracking-wider px-2 py-1.5 flex items-center gap-1">
                  <SvgIcon name="star" size="11" /> 收藏
                </div>
                <div
                  v-for="(tool, index) in favoriteTools"
                  :key="tool.id"
                  class="flex items-center gap-3 px-3 py-2 rounded-xl cursor-pointer transition-all duration-100"
                  :class="{ 'bg-primary/10': index === selectedIndex }"
                  @click="selectTool(tool.id)"
                  @mouseenter="selectedIndex = index"
                >
                  <span class="w-7 h-7 flex items-center justify-center rounded-lg bg-primary/15 shrink-0"><SvgIcon :name="tool.icon" size="14" /></span>
                  <span class="text-sm font-medium text-base-content">{{ tool.name }}</span>
                  <span class="text-[11px] text-base-content/50 truncate ml-auto">{{ tool.description }}</span>
                </div>
              </div>
              <div v-if="recentTools.length > 0">
                <div class="text-[10px] font-semibold text-base-content/50 uppercase tracking-wider px-2 py-1.5 flex items-center gap-1">
                  <SvgIcon name="clock" size="11" /> 最近使用
                </div>
                <div
                  v-for="(tool, index) in recentTools"
                  :key="tool.id"
                  class="flex items-center gap-3 px-3 py-2 rounded-xl cursor-pointer transition-all duration-100"
                  :class="{ 'bg-primary/10': index + favoriteTools.length === selectedIndex }"
                  @click="selectTool(tool.id)"
                  @mouseenter="selectedIndex = index + favoriteTools.length"
                >
                  <span class="w-7 h-7 flex items-center justify-center rounded-lg bg-primary/15 shrink-0"><SvgIcon :name="tool.icon" size="14" /></span>
                  <span class="text-sm font-medium text-base-content">{{ tool.name }}</span>
                  <span class="text-[11px] text-base-content/50 truncate ml-auto">{{ tool.description }}</span>
                </div>
              </div>
            </template>

            <!-- 有搜索词时：显示匹配结果 -->
            <template v-else>
              <div
                v-for="(tool, index) in searchResults"
                :key="tool.id"
                class="flex items-center gap-3 px-3 py-2 rounded-xl cursor-pointer transition-all duration-100"
                :class="{ 'bg-primary/10': index === selectedIndex }"
                @click="selectTool(tool.id)"
                @mouseenter="selectedIndex = index"
              >
                <span class="w-7 h-7 flex items-center justify-center rounded-lg bg-primary/15 shrink-0"><SvgIcon :name="tool.icon" size="14" /></span>
                <div class="flex-1 min-w-0">
                  <div class="text-sm font-medium text-base-content">{{ tool.name }}</div>
                  <div class="text-[11px] text-base-content/50 truncate">{{ tool.description }}</div>
                </div>
                <span class="text-[10px] text-base-content/40 shrink-0">{{ categoryLabel(tool.category) }}</span>
              </div>
              <div v-if="searchResults.length === 0" class="px-3 py-8 text-center text-sm text-base-content/50">
                未找到匹配工具
              </div>
            </template>
          </div>

          <!-- 快捷键提示 -->
          <div class="flex gap-4 px-4 py-2.5 border-t border-base-content/10 text-xs text-base-content/50 bg-base-200">
            <span>↑↓ 选择</span>
            <span>Enter 确认</span>
            <span>ESC 取消</span>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { useDevTools } from '@/views/devtools/composables/useDevTools'
import type { DevTool } from '@/views/devtools/DevToolRegistry'

const isOpen = ref(false)
const query = ref('')
const selectedIndex = ref(0)
const inputRef = ref<HTMLInputElement | null>(null)
const router = useRouter()

const { favoriteTools, recentTools, searchTools } = useDevTools()

const searchResults = computed<DevTool[]>(() => searchTools(query.value))

const allResults = computed<DevTool[]>(() => {
  if (!query.value) {
    return [...favoriteTools.value, ...recentTools.value]
  }
  return searchResults.value
})

watch(query, () => {
  selectedIndex.value = 0
})

watch(isOpen, (val) => {
  if (val) {
    query.value = ''
    selectedIndex.value = 0
    nextTick(() => {
      inputRef.value?.focus()
    })
  }
})

function open(): void {
  isOpen.value = true
}

function close(): void {
  isOpen.value = false
}

function navigate(direction: number): void {
  const max = allResults.value.length
  if (max === 0) return
  selectedIndex.value = (selectedIndex.value + direction + max) % max
}

function select(): void {
  const tool = allResults.value[selectedIndex.value]
  if (tool) {
    selectTool(tool.id)
  }
}

function selectTool(id: string): void {
  close()
  router.push({ path: '/devtools', query: { tool: id } })
}

const categoryLabels: Record<string, string> = {
  crypto: '加密/哈希',
  encode: '编码/转换',
  time: '时间/日期',
  code: '代码/JSON',
  text: '文本处理',
  network: '网络工具',
  convert: '进制/单位',
  format: '格式化',
  misc: '其他',
}

function categoryLabel(category: string): string {
  return categoryLabels[category] || category
}

defineExpose({ open, close })
</script>
