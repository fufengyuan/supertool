<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="visible" class="modal-overlay" @click="close">
        <div class="modal-content" @click.stop>
          <div class="modal-header">
            <h3>⌨️ 快捷键</h3>
            <button class="modal-close-btn" @click="close" title="关闭">×</button>
          </div>
          <div class="modal-body">
            <div class="shortcuts-list">
              <div class="shortcut-group">
                <h4>通用</h4>
                <div class="shortcut-item">
                  <span>新建任务</span>
                  <kbd>Ctrl+N</kbd>
                </div>
                <div class="shortcut-item">
                  <span>搜索任务</span>
                  <kbd>Ctrl+F</kbd>
                </div>
                <div class="shortcut-item">
                  <span>折叠侧边栏</span>
                  <kbd>Ctrl+B</kbd>
                </div>
                <div class="shortcut-item">
                  <span>切换主题</span>
                  <kbd>Ctrl+D</kbd>
                </div>
              </div>
              <div class="shortcut-group">
                <h4>视图切换</h4>
                <div class="shortcut-item">
                  <span>任务视图</span>
                  <kbd>Ctrl+1</kbd>
                </div>
                <div class="shortcut-item">
                  <span>周报视图</span>
                  <kbd>Ctrl+2</kbd>
                </div>
                <div class="shortcut-item">
                  <span>项目视图</span>
                  <kbd>Ctrl+3</kbd>
                </div>
                <div class="shortcut-item">
                  <span>服务器视图</span>
                  <kbd>Ctrl+4</kbd>
                </div>
                <div class="shortcut-item">
                  <span>数据备份</span>
                  <kbd>Ctrl+5</kbd>
                </div>
                <div class="shortcut-item">
                  <span>通知设置</span>
                  <kbd>Ctrl+6</kbd>
                </div>
              </div>
              <div class="shortcut-group">
                <h4>其他</h4>
                <div class="shortcut-item">
                  <span>关闭弹窗</span>
                  <kbd>Esc</kbd>
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

<style scoped>
.modal-overlay {
  position: fixed; top: 0; left: 0; right: 0; bottom: 0;
  display: flex; align-items: center; justify-content: center; z-index: 1000;
}
.modal-content {
  background: var(--card-bg); border-radius: 16px; width: 90%; max-width: 680px;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25); overflow: hidden;
}
.modal-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 20px 24px; border-bottom: 1px solid var(--border-color);
}
.modal-header h3 { margin: 0; font-size: 16px; font-weight: 600; color: var(--main-text); }
.modal-close-btn {
  width: 32px; height: 32px; border: none; border-radius: 8px;
  background: transparent; color: var(--main-text-secondary); font-size: 20px;
  cursor: pointer; display: flex; align-items: center; justify-content: center;
  transition: all 0.15s ease;
}
.modal-close-btn:hover { background: var(--input-bg); color: var(--main-text); }
.modal-body { padding: 24px; }
.shortcuts-list { display: flex; flex-direction: column; gap: 20px; }
.shortcut-group h4 {
  margin: 0 0 8px; font-size: 13px; font-weight: 600; color: var(--main-text-secondary);
  text-transform: uppercase; letter-spacing: 0.5px;
}
.shortcut-item {
  display: flex; justify-content: space-between; align-items: center;
  padding: 8px 0; border-bottom: 1px solid var(--border-color);
}
.shortcut-item:last-child { border-bottom: none; }
.shortcut-item span { font-size: 14px; color: var(--main-text); }
.shortcut-item kbd {
  padding: 4px 10px; background: var(--input-bg); border: 1px solid var(--border-color);
  border-radius: 6px; font-size: 12px; font-family: monospace; color: var(--main-text);
  box-shadow: 0 1px 0 var(--border-color);
}
.modal-enter-active, .modal-leave-active { transition: opacity 0.2s ease; }
.modal-enter-from, .modal-leave-to { opacity: 0; }
</style>
