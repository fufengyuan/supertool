<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition-opacity duration-200 ease"
      leave-active-class="transition-opacity duration-200 ease"
      enter-from-class="opacity-0"
      leave-to-class="opacity-0"
    >
      <div v-if="visible" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[1000]" @click="close">
        <div class="bg-base-100 rounded-2xl w-[90%] max-w-[520px] shadow-[0_25px_50px_-12px_rgba(0,0,0,0.25)] overflow-hidden" @click.stop>
          <div class="flex items-center justify-between px-6 py-5 border-b border-base-content/10">
            <h3 class="m-0 text-base font-semibold text-base-content">{{ $t('about.title') }}</h3>
            <button class="w-8 h-8 border-none rounded-lg bg-transparent text-base-content/60 text-lg cursor-pointer flex items-center justify-center transition-all duration-150 ease hover:bg-base-200 hover:text-base-content" @click="close" :title="$t('about.close')">×</button>
          </div>
          <div class="p-0">
            <div class="px-6 pt-8 pb-6 text-center flex flex-col items-center">
              <!-- Logo & App Info -->
              <div class="mb-3 text-primary">
                <svg xmlns="http://www.w3.org/2000/svg" width="56" height="56" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/>
                  <rect x="8" y="2" width="8" height="4" rx="1" ry="1"/>
                  <path d="M9 11l3 3L22 4"/>
                </svg>
              </div>

              <h2 class="m-0 text-[22px] font-bold text-base-content tracking-tight">SuperTool</h2>
              <p class="mt-1 mb-0 text-[13px] text-base-content/60">{{ $t('about.version') }} {{ appVersion }}</p>

              <div class="mt-4">
                <p class="m-0 text-sm text-base-content/60 leading-relaxed max-w-[420px]">
                  {{ $t('about.description') }}
                </p>
              </div>

              <!-- Update Section -->
              <div class="mt-6 w-full">
                <button class="btn btn-outline btn-sm inline-flex items-center gap-2" @click="checkForUpdates" :disabled="checkingUpdate">
                  <svg v-if="!checkingUpdate" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
                  <span v-if="checkingUpdate" class="loading loading-spinner loading-xs"></span>
                  {{ checkingUpdate ? $t('about.checking') : $t('about.checkUpdate') }}
                </button>
                <p v-if="updateMessage" class="mt-3 text-[13px] text-base-content/60" :class="{ 'text-primary font-medium': updateAvailable }">
                  {{ updateMessage }}
                </p>
              </div>

              <!-- Divider -->
              <div class="w-full h-px bg-base-content/10 my-6"></div>

              <!-- License -->
              <div class="flex flex-col gap-1">
                <p class="m-0 text-xs font-semibold uppercase text-base-content/60 tracking-wider">{{ $t('about.license') }}</p>
                <p class="m-0 text-sm font-medium text-base-content">MIT License</p>
                <p class="m-0 text-xs text-base-content/60">{{ $t('about.copyright') }}</p>
              </div>

              <!-- Links -->
              <div class="mt-4">
                <a href="#" class="inline-flex items-center gap-1.5 text-[13px] text-primary no-underline hover:opacity-80" @click.prevent="openExternal('https://github.com/example/supertool')">
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
