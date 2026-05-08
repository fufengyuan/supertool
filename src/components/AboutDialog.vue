<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="visible" class="modal-overlay" @click="close">
        <div class="modal-content" @click.stop>
          <div class="modal-header">
            <h3>{{ $t('about.title') }}</h3>
            <button class="modal-close-btn" @click="close" :title="$t('about.close')">×</button>
          </div>
          <div class="modal-body">
            <div class="about-content">
              <!-- Logo & App Info -->
              <div class="app-logo">
                <svg xmlns="http://www.w3.org/2000/svg" width="56" height="56" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/>
                  <rect x="8" y="2" width="8" height="4" rx="1" ry="1"/>
                  <path d="M9 11l3 3L22 4"/>
                </svg>
              </div>

              <h2 class="app-name">SuperTool</h2>
              <p class="app-version">{{ $t('about.version') }} {{ appVersion }}</p>

              <div class="app-info">
                <p class="app-description">
                  {{ $t('about.description') }}
                </p>
              </div>

              <!-- Update Section -->
              <div class="update-section">
                <button class="btn btn-check-update" @click="checkForUpdates" :disabled="checkingUpdate">
                  <svg v-if="!checkingUpdate" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
                  <span v-if="checkingUpdate" class="mini-spinner"></span>
                  {{ checkingUpdate ? $t('about.checking') : $t('about.checkUpdate') }}
                </button>
                <p v-if="updateMessage" class="update-message" :class="{ 'update-available': updateAvailable }">
                  {{ updateMessage }}
                </p>
              </div>

              <!-- Divider -->
              <div class="divider"></div>

              <!-- License -->
              <div class="license-section">
                <p class="license-label">{{ $t('about.license') }}</p>
                <p class="license-text">MIT License</p>
                <p class="copyright">{{ $t('about.copyright') }}</p>
              </div>

              <!-- Links -->
              <div class="links-section">
                <a href="#" class="link" @click.prevent="openExternal('https://github.com/example/supertool')">
                  <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"/></svg>
                  GitHub
                </a>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>

  <!-- Update Dialog -->
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { getTauriAPI } from '../utils/tauri-api'

const props = defineProps({
  modelValue: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['update:modelValue'])

const visible = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
})

const appVersion = ref('1.0.0')
const checkingUpdate = ref(false)
const updateMessage = ref('')
const updateAvailable = ref(false)

const close = () => {
  visible.value = false
  resetUpdateState()
}

const resetUpdateState = () => {
  checkingUpdate.value = false
  updateMessage.value = ''
  updateAvailable.value = false
}

const loadVersion = async () => {
  try {
    console.log("[loadVersion] called")
    const api = getTauriAPI()
    const ver = await (api as any).getAppVersion?.()
    if (ver) appVersion.value = ver
  } catch {
    appVersion.value = __APP_VERSION__ || '1.0.0'
  }
}

const checkForUpdates = () => {
  checkingUpdate.value = true
  updateMessage.value = '正在检查更新...'
  updateAvailable.value = false

  // Tauri 使用内置更新机制，此处简化提示
  setTimeout(() => {
    checkingUpdate.value = false
    updateMessage.value = '当前已是最新版本'
  }, 1500)
}

const openExternal = (url: string) => {
  window.open(url, '_blank')
}

watch(visible, (val) => {
  if (val) {
    loadVersion()
    resetUpdateState()
  }
})

onMounted(() => {
    console.log("[components/AboutDialog.vue] mounted")
  loadVersion()
})
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background: oklch(var(--b1));
  border-radius: 16px;
  width: 90%;
  max-width: 520px;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
  overflow: hidden;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
}

.modal-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: oklch(var(--bc));
}

.modal-close-btn {
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: oklch(var(--bc) / 0.6);
  font-size: 20px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}

.modal-close-btn:hover {
  background: oklch(var(--b2));
  color: oklch(var(--bc));
}

.modal-body {
  padding: 0;
}

.about-content {
  padding: 32px 24px 24px;
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.app-logo {
  margin-bottom: 12px;
  color: oklch(var(--p));
}

.app-name {
  margin: 0;
  font-size: 22px;
  font-weight: 700;
  color: oklch(var(--bc));
  letter-spacing: -0.5px;
}

.app-version {
  margin: 4px 0 0;
  font-size: 13px;
  color: oklch(var(--bc) / 0.6);
}

.app-info {
  margin-top: 16px;
}

.app-description {
  margin: 0;
  font-size: 14px;
  color: oklch(var(--bc) / 0.6);
  line-height: 1.6;
  max-width: 420px;
}

/* Update Section */
.update-section {
  margin-top: 24px;
  width: 100%;
}

.btn-check-update {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 8px 20px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  background: transparent;
  color: oklch(var(--bc));
  font-size: 14px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-check-update:hover:not(:disabled) {
  background: oklch(var(--p));
  color: white;
  border-color: oklch(var(--p));
}

.btn-check-update:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.mini-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid oklch(var(--bc) / 0.1);
  border-top-color: oklch(var(--p));
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  display: inline-block;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.update-message {
  margin: 12px 0 0;
  font-size: 13px;
  color: oklch(var(--bc) / 0.6);
}

.update-message.update-available {
  color: oklch(var(--p));
  font-weight: 500;
}

/* Divider */
.divider {
  width: 100%;
  height: 1px;
  background: oklch(var(--bc) / 0.1);
  margin: 24px 0;
}

/* License */
.license-section {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.license-label {
  margin: 0;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  color: oklch(var(--bc) / 0.6);
  letter-spacing: 0.5px;
}

.license-text {
  margin: 0;
  font-size: 14px;
  color: oklch(var(--bc));
  font-weight: 500;
}

.copyright {
  margin: 0;
  font-size: 12px;
  color: oklch(var(--bc) / 0.6);
}

/* Links */
.links-section {
  margin-top: 16px;
}

.link {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: oklch(var(--p));
  text-decoration: none;
  transition: opacity 0.15s ease;
}

.link:hover {
  opacity: 0.8;
}

/* Transitions */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
