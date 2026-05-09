<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition-opacity duration-200 ease-out"
      leave-active-class="transition-opacity duration-200 ease-in"
      enter-from-class="opacity-0"
      leave-to-class="opacity-0"
    >
      <div v-if="visible" class="fixed inset-0 z-[1000] flex items-center justify-center" @click="close">
        <div class="w-[90%] max-w-[680px] overflow-hidden rounded-2xl bg-base-100 shadow-2xl" @click.stop>
          <div class="flex items-center justify-between border-b border-base-content/10 px-6 py-5">
            <h3 class="m-0 text-base font-semibold text-base-content"><SvgIcon name="keyboard" size="14" class="inline-block align-text-bottom" /> 快捷键</h3>
            <button class="btn btn-ghost btn-square btn-sm text-lg text-base-content/60 hover:text-base-content" @click="close" title="关闭">×</button>
          </div>
          <div class="p-6">
            <div class="flex flex-col gap-5">
              <div>
                <h4 class="m-0 mb-2 text-xs font-semibold uppercase tracking-wide text-base-content/60">通用</h4>
                <div class="flex items-center justify-between border-b border-base-content/10 py-2 last:border-b-0">
                  <span class="text-sm text-base-content">新建任务</span>
                  <kbd class="kbd kbd-sm">Ctrl+N</kbd>
                </div>
                <div class="flex items-center justify-between border-b border-base-content/10 py-2 last:border-b-0">
                  <span class="text-sm text-base-content">搜索任务</span>
                  <kbd class="kbd kbd-sm">Ctrl+F</kbd>
                </div>
                <div class="flex items-center justify-between border-b border-base-content/10 py-2 last:border-b-0">
                  <span class="text-sm text-base-content">折叠侧边栏</span>
                  <kbd class="kbd kbd-sm">Ctrl+B</kbd>
                </div>
                <div class="flex items-center justify-between border-b border-base-content/10 py-2 last:border-b-0">
                  <span class="text-sm text-base-content">切换主题</span>
                  <kbd class="kbd kbd-sm">Ctrl+D</kbd>
                </div>
              </div>
              <div>
                <h4 class="m-0 mb-2 text-xs font-semibold uppercase tracking-wide text-base-content/60">视图切换</h4>
                <div class="flex items-center justify-between border-b border-base-content/10 py-2 last:border-b-0">
                  <span class="text-sm text-base-content">任务视图</span>
                  <kbd class="kbd kbd-sm">Ctrl+1</kbd>
                </div>
                <div class="flex items-center justify-between border-b border-base-content/10 py-2 last:border-b-0">
                  <span class="text-sm text-base-content">周报视图</span>
                  <kbd class="kbd kbd-sm">Ctrl+2</kbd>
                </div>
                <div class="flex items-center justify-between border-b border-base-content/10 py-2 last:border-b-0">
                  <span class="text-sm text-base-content">项目视图</span>
                  <kbd class="kbd kbd-sm">Ctrl+3</kbd>
                </div>
                <div class="flex items-center justify-between border-b border-base-content/10 py-2 last:border-b-0">
                  <span class="text-sm text-base-content">服务器视图</span>
                  <kbd class="kbd kbd-sm">Ctrl+4</kbd>
                </div>
                <div class="flex items-center justify-between border-b border-base-content/10 py-2 last:border-b-0">
                  <span class="text-sm text-base-content">数据备份</span>
                  <kbd class="kbd kbd-sm">Ctrl+5</kbd>
                </div>
                <div class="flex items-center justify-between border-b border-base-content/10 py-2 last:border-b-0">
                  <span class="text-sm text-base-content">通知设置</span>
                  <kbd class="kbd kbd-sm">Ctrl+6</kbd>
                </div>
              </div>
              <div>
                <h4 class="m-0 mb-2 text-xs font-semibold uppercase tracking-wide text-base-content/60">其他</h4>
                <div class="flex items-center justify-between border-b border-base-content/10 py-2 last:border-b-0">
                  <span class="text-sm text-base-content">关闭弹窗</span>
                  <kbd class="kbd kbd-sm">Esc</kbd>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps({
  modelValue: { type: Boolean, default: false }
})

const emit = defineEmits(['update:modelValue'])

const visible = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
})

const close = () => { visible.value = false }
</script>
