<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="visible" class="modal-overlay" @click="close">
        <div class="modal-content" @click.stop>
          <div class="modal-header">
            <h3>
              <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="vertical-align: -3px; margin-right: 6px;"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
              检查更新
            </h3>
            <button class="modal-close-btn" @click="close" title="关闭">×</button>
          </div>
          <div class="modal-body">
            <!-- 检查中 -->
            <div v-if="state === 'checking'" class="update-state">
              <div class="spinner"></div>
              <p>正在检查更新...</p>
            </div>

            <!-- 当前版本 -->
            <div v-else-if="state === 'up-to-date'" class="update-state">
              <div class="success-icon">
                <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
              </div>
              <p>当前版本: v{{ currentVersion }}</p>
              <p class="update-hint">SuperTool Tauri 版本，通过 GitHub Releases 获取更新</p>
              <div class="update-actions">
                <button class="btn btn-primary" @click="openReleases">查看 Releases</button>
                <button class="btn btn-secondary" @click="close">关闭</button>
              </div>
            </div>

            <!-- 错误 -->
            <div v-else-if="state === 'error'" class="update-state">
              <div class="error-icon">
                <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
              </div>
              <p>检查更新失败</p>
              <p class="error-message">{{ errorMessage }}</p>
              <div class="update-actions">
                <button class="btn btn-primary" @click="retryCheck">重试</button>
                <button class="btn btn-secondary" @click="close">关闭</button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { getTauriAPI } from '../utils/tauri-api'

const props = defineProps({
  modelValue: { type: Boolean, default: false }
})

const emit = defineEmits(['update:modelValue'])

const visible = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
})

const state = ref('checking')
const currentVersion = ref('')
const errorMessage = ref('')

onMounted(async () => {
  try {
    const tauri = getTauriAPI()
    currentVersion.value = await tauri.getAppVersion()
    state.value = 'up-to-date'
  } catch (e: any) {
    state.value = 'error'
    errorMessage.value = e?.message || '未知错误'
  }
})

const retryCheck = async () => {
  state.value = 'checking'
  try {
    const tauri = getTauriAPI()
    currentVersion.value = await tauri.getAppVersion()
    state.value = 'up-to-date'
  } catch (e: any) {
    state.value = 'error'
    errorMessage.value = e?.message || '未知错误'
  }
}

const openReleases = () => {
  window.open('https://github.com/fufengyuan/supertool/releases', '_blank')
}

const close = () => {
  visible.value = false
}
</script>

<style scoped>
.modal-overlay {
  position: fixed; inset: 0; background: rgba(0,0,0,0.5);
  display: flex; align-items: center; justify-content: center; z-index: 10000;
}
.modal-content {
  background: var(--card-bg); border-radius: 16px; padding: 24px;
  max-width: 460px; width: 90%; box-shadow: 0 8px 32px rgba(0,0,0,0.2);
}
.modal-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
.modal-header h3 { margin: 0; font-size: 18px; color: var(--main-text); }
.modal-close-btn { background: none; border: none; font-size: 24px; cursor: pointer; color: var(--main-text-secondary); padding: 0 4px; line-height: 1; }
.modal-body { padding: 10px 0; }
.update-state { text-align: center; padding: 20px 0; }
.spinner { width: 40px; height: 40px; border: 3px solid var(--border-color); border-top-color: var(--primary-color); border-radius: 50%; animation: spin 0.8s linear infinite; margin: 0 auto 16px; }
@keyframes spin { to { transform: rotate(360deg); } }
.success-icon, .error-icon { color: var(--primary-color); margin-bottom: 16px; }
.error-icon { color: #ef4444; }
.update-hint { color: var(--main-text-secondary); font-size: 13px; margin: 8px 0; }
.error-message { color: #ef4444; font-size: 13px; margin: 8px 0; }
.update-actions { display: flex; gap: 12px; justify-content: center; margin-top: 20px; }
.btn { padding: 10px 24px; border-radius: 8px; font-size: 14px; font-weight: 500; cursor: pointer; transition: all 0.2s; border: none; }
.btn-primary { background: var(--primary-color); color: white; }
.btn-primary:hover { opacity: 0.9; }
.btn-secondary { background: var(--input-bg); color: var(--main-text-secondary); border: 1px solid var(--border-color); }
.btn-secondary:hover { border-color: var(--primary-color); }
.modal-enter-active, .modal-leave-active { transition: opacity 0.25s ease; }
.modal-enter-from, .modal-leave-to { opacity: 0; }
</style>
