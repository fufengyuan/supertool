<template>
  <aside
    :class="[
      'flex flex-col border-r border-base-content/10 bg-base-100 flex-shrink-0 transition-all duration-300 overflow-hidden',
      collapsed ? 'w-[44px] min-w-[44px] items-center' : 'w-[240px] min-w-[200px] max-w-[280px]',
    ]"
  >
    <!-- 工具栏 -->
    <div class="flex items-center gap-1 px-2.5 py-2.5 border-b border-base-content/10" :class="collapsed ? 'flex-col gap-2 border-b-0' : ''">
      <button
        class="btn btn-primary btn-xs gap-1 flex-1"
        :title="collapsed ? '新建会话' : ''"
        @click="onNew"
      >
        <SvgIcon name="plus" size="13" :stroke-width="2.5" />
        <span v-show="!collapsed">新会话</span>
      </button>
      <button
        class="btn btn-ghost btn-xs btn-square shrink-0"
        :title="collapsed ? '展开会话列表' : '收起列表'"
        @click="toggleCollapsed"
      >
        <SvgIcon :name="collapsed ? 'chevronRight' : 'chevronLeft'" size="13" />
      </button>
    </div>

    <!-- 会话列表 -->
    <div v-show="!collapsed" class="flex-1 overflow-y-auto px-2 py-2 flex flex-col gap-1">
      <div v-if="!sessions.length" class="text-[11px] text-base-content/50 px-2 py-6 text-center leading-relaxed">
        还没有历史会话。发送第一条消息后会自动保存，之后可在这里切换继续。
      </div>

      <div
        v-for="s in sessions"
        :key="s.id"
        class="group flex items-center gap-1.5 px-2 py-2 rounded-lg cursor-pointer border border-transparent transition-all duration-150 hover:bg-base-200 hover:border-base-content/10"
        :class="{ 'bg-primary/10 border-primary': currentId === s.id }"
        @click="onSelect(s.id)"
      >
        <SvgIcon name="history" size="13" class="text-base-content/40 shrink-0" />
        <div class="flex-1 min-w-0">
          <div class="text-xs font-medium text-base-content truncate" :title="s.title">{{ s.title }}</div>
          <div class="text-[10px] text-base-content/40">{{ formatTime(s.updatedAt) }}</div>
        </div>
        <!-- 重命名 / 删除（悬浮显示） -->
        <div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity shrink-0" @click.stop>
          <button class="p-0.5 rounded text-base-content/50 hover:text-primary" title="重命名" @click="beginRename(s)">
            <SvgIcon name="pencil" size="11" />
          </button>
          <button class="p-0.5 rounded text-base-content/50 hover:text-error" title="删除会话" @click="onDelete(s)">
            <SvgIcon name="trash" size="11" />
          </button>
        </div>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import SvgIcon from '../../../components/ui/SvgIcon.vue'
import { useAssistantSessions, type AssistantSessionMeta } from '../../../composables/useAssistantSessions'

const props = defineProps<{
  sessions: AssistantSessionMeta[]
  currentId: string | null
  collapsed: boolean
}>()

const emit = defineEmits<{
  (e: 'update:collapsed', v: boolean): void
  (e: 'new'): void
  (e: 'select', id: string): void
  (e: 'delete', id: string): void
}>()

const renameStore = useAssistantSessions()

const collapsed = ref(props.collapsed)
watch(() => props.collapsed, (v) => { collapsed.value = v })
watch(collapsed, (v) => { emit('update:collapsed', v) })
function toggleCollapsed() { collapsed.value = !collapsed.value }

/** 重命名：prompt 取值后直接调 store 落库，并在本地列表中更新标题 */
async function beginRename(s: AssistantSessionMeta) {
  const name = window.prompt('重命名会话', s.title)
  if (!name || !name.trim()) {return}
  await renameStore.renameSession(s.id, name.trim())
}

function onNew() { emit('new') }
function onSelect(id: string) { emit('select', id) }
function onDelete(s: AssistantSessionMeta) {
  if (window.confirm(`删除会话「${s.title}」？`)) { emit('delete', s.id) }
}

function formatTime(iso: string) {
  if (!iso) {return ''}
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) {return ''}
  return d.toLocaleString('zh-CN', { month: 'numeric', day: 'numeric', hour: '2-digit', minute: '2-digit', hour12: false })
}
</script>