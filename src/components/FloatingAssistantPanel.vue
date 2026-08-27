<template>
  <div class="floating-assistant-panel" :data-theme="theme" :class="{ 'ball-mode': collapsed }">
    <!-- 小球 -->
    <button
      v-if="collapsed"
      class="fa-ball"
      title="AI 配置助手（点击展开，拖动可移动）"
      @mousedown="onBallMouseDown"
    >
      <SvgIcon name="bot" size="24" />
      <span v-if="running" class="fa-ball__dot" />
    </button>

    <!-- 展开面板 -->
    <div v-else class="fa-card bg-base-100 border border-base-content/10">
      <div class="fa-title" @mousedown="onTitleMouseDown">
        <SvgIcon name="bot" size="14" class="shrink-0" />
        <span class="text-xs font-bold truncate flex-1">AI 配置助手</span>
        <span v-if="modelInfo" class="text-[10px] opacity-70 shrink-0 max-w-[92px] truncate">
          {{ modelInfo.modelId }}
        </span>
        <button class="fa-title__btn" :title="pinned ? '取消置顶' : '保持置顶'" @click.stop="togglePin">
          <SvgIcon name="mapPin" size="12" :class="pinned ? '' : 'opacity-50'" />
        </button>
        <button class="fa-title__btn" title="收起为小球" @click.stop="collapsed = true">
          <SvgIcon name="minus" size="12" />
        </button>
        <button class="fa-title__btn" title="关闭" @click.stop="close">
          <SvgIcon name="x" size="12" />
        </button>
      </div>

      <div class="fa-body">
        <AssistantChat compact />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
import SvgIcon from './ui/SvgIcon.vue'
import AssistantChat from '../views/assistant/components/AssistantChat.vue'
import { getTauriAPI } from '../utils/tauri-api'
import { useSettingsStore } from '../utils/settings'

const BALL_SIZE = 56
const PANEL_WIDTH = 400
const PANEL_HEIGHT = 560
const CLICK_THRESHOLD_PX = 5
const CLICK_THRESHOLD_MS = 300

const theme = useSettingsStore().theme as string
const collapsed = ref(true)
const pinned = ref(true)
const running = ref(false)
const modelInfo = ref<Record<string, unknown> | null>(null)

async function applyWindowSize() {
  try {
    const win = getCurrentWindow()
    if (collapsed.value) {
      await win.setMinSize(new LogicalSize(BALL_SIZE, BALL_SIZE))
      await win.setSize(new LogicalSize(BALL_SIZE, BALL_SIZE))
    } else {
      await win.setMinSize(new LogicalSize(PANEL_WIDTH, PANEL_HEIGHT))
      await win.setSize(new LogicalSize(PANEL_WIDTH, PANEL_HEIGHT))
    }
  } catch { /* 非 Tauri 环境 */ }
}

watch(collapsed, applyWindowSize)

async function drag() {
  try {
    await getCurrentWindow().startDragging()
  } catch { /* ignore */ }
}

/** 拖动 vs 点击：按下即交给原生拖拽，抬起时若几乎没动则视为点击 */
function clickOrDrag(e: MouseEvent, onClick: () => void) {
  if (e.button !== 0) {return}
  const startX = e.clientX
  const startY = e.clientY
  const startTime = Date.now()
  void drag()
  const onUp = (ev: MouseEvent) => {
    window.removeEventListener('mouseup', onUp)
    const moved = Math.abs(ev.clientX - startX) + Math.abs(ev.clientY - startY)
    if (moved < CLICK_THRESHOLD_PX && Date.now() - startTime < CLICK_THRESHOLD_MS) {onClick()}
  }
  window.addEventListener('mouseup', onUp)
}

function onBallMouseDown(e: MouseEvent) {
  clickOrDrag(e, () => {collapsed.value = false})
}

function onTitleMouseDown(e: MouseEvent) {
  const target = e.target as HTMLElement
  if (target.closest('button')) {return}
  clickOrDrag(e, () => {collapsed.value = true})
}

async function togglePin() {
  pinned.value = !pinned.value
  try {
    await (getTauriAPI() as any).setFloatingAssistantPinned?.(pinned.value)
  } catch { /* ignore */ }
}

async function close() {
  try {
    await (getTauriAPI() as any).closeFloatingAssistant?.()
  } catch { /* ignore */ }
  await getCurrentWindow().close().catch(() => {})
}

onMounted(async () => {
  // 默认小球形态：启动时只占 56×56，不闪大窗
  applyWindowSize()
  try {
    const state = await (getTauriAPI() as any).assistantGetState?.()
    modelInfo.value = state?.active || null
    running.value = (state?.runningTurns || 0) > 0
  } catch { /* 未配置模型时忽略 */ }
})
</script>

<style scoped>
.floating-assistant-panel {
  width: 100vw;
  height: 100vh;
  display: flex;
  align-items: flex-end;
  justify-content: flex-end;
  background: transparent;
  user-select: none;
  overflow: hidden;
}

.fa-ball {
  width: 56px;
  height: 56px;
  border-radius: 999px;
  border: 0;
  cursor: pointer;
  background: linear-gradient(135deg, #667eea, #764ba2);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  box-shadow: 0 6px 18px rgb(0 0 0 / 25%);
}

.fa-ball__dot {
  position: absolute;
  top: 6px;
  right: 6px;
  width: 8px;
  height: 8px;
  border-radius: 999px;
  background: #f59e0b;
  animation: fa-pulse 1.2s ease-in-out infinite;
}

@keyframes fa-pulse {
  0%, 100% { opacity: 0.4; }
  50% { opacity: 1; }
}

.fa-card {
  width: 100%;
  height: 100%;
  position: relative;
  display: flex;
  flex-direction: column;
  min-height: 0;
  border-radius: 14px;
  overflow: hidden;
  box-shadow: 0 10px 30px rgb(0 0 0 / 22%);
}


.fa-title {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 10px;
  background: linear-gradient(135deg, #667eea, #764ba2);
  color: #fff;
  cursor: grab;
  flex-shrink: 0;
}

.fa-title__btn {
  border: 0;
  background: rgb(255 255 255 / 15%);
  color: inherit;
  width: 20px;
  height: 20px;
  border-radius: 6px;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.fa-title__btn:hover {
  background: rgb(255 255 255 / 30%);
}

.fa-body {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}
</style>
