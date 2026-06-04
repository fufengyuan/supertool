<template>
  <div class="max-w-3xl mx-auto">
    <!-- Header -->
    <div class="flex items-center justify-between mb-6">
      <div>
        <h1 class="text-2xl font-bold">{{ t('agentSettings.title') }}</h1>
        <p class="text-sm text-base-content/60 mt-1">{{ t('agentSettings.description') }}</p>
      </div>
      <button class="btn btn-ghost btn-sm" @click="loadConfig" :disabled="loading">
        <IconRefresh :size="16" :class="{ 'animate-spin': loading }" />
      </button>
    </div>

    <!-- Error -->
    <div v-if="error" class="alert alert-error mb-4 text-sm py-2">
      <IconAlertCircle :size="16" />
      <span>{{ error }}</span>
    </div>

    <!-- Success -->
    <div v-if="successMsg" class="alert alert-success mb-4 text-sm py-2">
      <IconCheck :size="16" />
      <span>{{ successMsg }}</span>
    </div>

    <!-- Tabs -->
    <div class="tabs tabs-bordered mb-6">
      <button class="tab tab-bordered tab-sm flex items-center gap-1"
        :class="tab === 'general' ? 'tab-active' : ''"
        @click="tab = 'general'">
        <IconSettings :size="16" />
        <span>{{ t('agentSettings.tabs.general') }}</span>
      </button>
      <button class="tab tab-bordered tab-sm flex items-center gap-1"
        :class="tab === 'appearance' ? 'tab-active' : ''"
        @click="tab = 'appearance'">
        <IconPalette :size="16" />
        <span>Appearance</span>
      </button>
      <button class="tab tab-bordered tab-sm flex items-center gap-1"
        :class="tab === 'language' ? 'tab-active' : ''"
        @click="tab = 'language'">
        <IconLanguage :size="16" />
        <span>Language</span>
      </button>
      <button class="tab tab-bordered tab-sm flex items-center gap-1"
        :class="tab === 'about' ? 'tab-active' : ''"
        @click="tab = 'about'">
        <IconInfoCircle :size="16" />
        <span>About</span>
      </button>
    </div>

    <!-- ==================== General Tab ==================== -->
    <div v-if="tab === 'general'" class="space-y-4">
      <!-- Claw Configuration (Claw mode only) -->
      <div v-if="isClawMode" class="bg-base-100 border border-base-300 rounded-xl p-5">
        <h2 class="text-base font-semibold mb-4 flex items-center gap-2">
          <IconCode :size="18" />
          Claw Agent
          <span class="text-xs text-base-content/40">~/.claw/config.json</span>
        </h2>

        <div class="space-y-3">
          <!-- API Key -->
          <div>
            <label class="text-xs font-medium text-base-content/70 mb-1 block">API Key</label>
            <input
              v-model="clawForm.apiKey"
              type="password"
              class="input input-bordered input-sm w-full text-xs"
              placeholder="sk-..."
              autocomplete="off"
            />
            <p v-if="clawInfoSaved.apiKey" class="text-[10px] text-base-content/40 mt-0.5">
              已保存: {{ clawInfoSaved.apiKey }}
            </p>
          </div>

          <!-- Base URL -->
          <div>
            <label class="text-xs font-medium text-base-content/70 mb-1 block">
              Base URL
              <span class="text-base-content/40">(可选，留空走官方 API)</span>
            </label>
            <input
              v-model="clawForm.baseUrl"
              type="url"
              class="input input-bordered input-sm w-full text-xs"
              placeholder="https://api.openai.com/v1"
            />
          </div>

          <!-- Model -->
          <div>
            <label class="text-xs font-medium text-base-content/70 mb-1 block">Model</label>
            <input
              v-model="clawForm.model"
              type="text"
              class="input input-bordered input-sm w-full text-xs"
              placeholder="claude-sonnet-4-6"
            />
          </div>

          <!-- Provider -->
          <div>
            <label class="text-xs font-medium text-base-content/70 mb-1 block">
              Provider
              <span class="text-base-content/40">(可选，如 anthropic / openai)</span>
            </label>
            <input
              v-model="clawForm.provider"
              type="text"
              class="input input-bordered input-sm w-full text-xs"
              placeholder="anthropic"
            />
          </div>

          <!-- Save -->
          <div class="flex items-center gap-3 pt-2">
            <button
              class="btn btn-primary btn-sm gap-1.5"
              :disabled="clawSaving"
              @click="saveClawConfig"
            >
              <SvgIcon name="save" :size="14" />
              {{ clawSaving ? '保存中...' : '保存' }}
            </button>
            <span v-if="clawSaveMsg" class="text-xs" :class="clawSaveMsg.includes('✅') ? 'text-success' : 'text-error'">
              {{ clawSaveMsg }}
            </span>
          </div>
        </div>
      </div>

      <!-- Hermes Config Info -->
      <div v-if="!isClawMode" class="bg-base-100 border border-base-300 rounded-xl p-5">
        <h2 class="text-base font-semibold mb-4 flex items-center gap-2">
          <IconSettings :size="18" />
          {{ t('agentSettings.general.hermesHome') }}
        </h2>
        <div class="space-y-3 text-sm">
          <div class="flex items-center justify-between">
            <span class="text-base-content/70">{{ t('agentSettings.general.hermesHome') }}</span>
            <code class="text-xs bg-base-200 px-2 py-1 rounded truncate max-w-[240px]">{{ configInfo?.hermesHome || '-' }}</code>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-base-content/70">{{ t('agentSettings.general.version') }}</span>
            <span class="font-mono text-xs">{{ configInfo?.version || '-' }}</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-base-content/70">{{ t('agentSettings.general.configExists') }}</span>
            <span :class="configInfo?.configExists ? 'text-success' : 'text-error'">
              {{ configInfo?.configExists ? t('agentSettings.general.yes') : t('agentSettings.general.no') }}
            </span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-base-content/70">{{ t('agentSettings.general.installed') }}</span>
            <span :class="configInfo?.installed ? 'text-success' : 'text-error'">
              {{ configInfo?.installed ? t('agentSettings.general.yes') : t('agentSettings.general.notInstalled') }}
            </span>
          </div>
        </div>
      </div>

      <!-- API Server -->
      <div v-if="!isClawMode" class="bg-base-100 border border-base-300 rounded-xl p-5">
        <h2 class="text-base font-semibold mb-4 flex items-center gap-2">
          <IconServer :size="18" />
          API Server
        </h2>
        <div class="space-y-3 text-sm">
          <div class="flex items-center justify-between">
            <span class="text-base-content/70">状态</span>
            <div class="flex items-center gap-2">
              <span :class="apiRunning ? 'text-success' : 'text-base-content/50'" class="text-xs font-medium">
                {{ apiRunning ? '运行中' : '未运行' }}
              </span>
              <button class="btn btn-ghost btn-xs" @click="loadApiStatus">
                <IconRefresh :size="12" />
              </button>
            </div>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-base-content/70">API Key</span>
            <div class="flex items-center gap-2">
              <code class="text-xs bg-base-200 px-2 py-1 rounded font-mono">{{ apiKeyDisplay }}</code>
              <button class="btn btn-ghost btn-xs" @click="showApiKeyModal = true">
                <IconKey :size="12" />
                配置
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- API Key 配置弹窗 -->
      <div v-if="showApiKeyModal" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" @click.self="showApiKeyModal = false">
        <div class="bg-base-100 rounded-xl p-5 w-full max-w-sm shadow-2xl">
          <h3 class="text-lg font-bold mb-3">配置 Hermes API Key</h3>
          <p class="text-xs text-base-content/60 mb-3">
            API Key 用于 SuperTool 与 Hermes Gateway 之间的通信认证。
          </p>
          <div class="mb-4">
            <label class="text-xs text-base-content/70 mb-1 block">API Key</label>
            <input
              v-model="newApiKey"
              type="text"
              class="input input-bordered input-sm w-full font-mono text-xs"
              placeholder="输入自定义 API Key（留空自动生成）"
            />
          </div>
          <div class="flex gap-2 justify-end">
            <button class="btn btn-ghost btn-sm" @click="showApiKeyModal = false">取消</button>
            <button class="btn btn-primary btn-sm" @click="saveApiKey" :disabled="savingApiKey">
              <IconRefresh v-if="savingApiKey" :size="12" class="animate-spin" />
              保存
            </button>
          </div>
        </div>
      </div>

      <!-- Export / Import -->
      <div v-if="!isClawMode" class="bg-base-100 border border-base-300 rounded-xl p-5">
        <h2 class="text-base font-semibold mb-4 flex items-center gap-2">
          <IconFileExport :size="18" />
          {{ t('agentSettings.exportImport.export') }} / {{ t('agentSettings.exportImport.import') }}
        </h2>
        <div class="space-y-3">
          <div class="flex gap-2">
            <button class="btn btn-outline btn-sm" @click="handleExport" :disabled="exportLoading">
              <IconDownload :size="14" />
              {{ t('agentSettings.exportImport.export') }}
            </button>
            <button class="btn btn-outline btn-sm" @click="showImport = true">
              <IconUpload :size="14" />
              {{ t('agentSettings.exportImport.import') }}
            </button>
          </div>
          <!-- Export preview -->
          <div v-if="exportContent !== null" class="relative">
            <pre class="text-xs bg-base-200 p-3 rounded-lg overflow-x-auto max-h-48 whitespace-pre-wrap font-mono">{{ exportContent }}</pre>
            <button class="btn btn-ghost btn-xs absolute top-2 right-2" @click="copyExport">
              {{ copied ? t('agentSettings.exportImport.copied') : t('agentSettings.exportImport.copy') }}
            </button>
          </div>
        </div>
      </div>

      <!-- Import Modal -->
      <div v-if="!isClawMode && showImport" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" @click.self="showImport = false">
        <div class="bg-base-100 rounded-xl p-5 w-full max-w-lg max-h-[80vh] overflow-y-auto shadow-2xl">
          <h3 class="text-base font-semibold mb-3">{{ t('agentSettings.exportImport.import') }}</h3>
          <p class="text-xs text-base-content/60 mb-3">{{ t('agentSettings.exportImport.importHint') }}</p>
          <textarea v-model="importContent" class="textarea textarea-bordered w-full font-mono text-xs h-40" placeholder="Paste YAML..."></textarea>
          <p v-if="importError" class="text-error text-xs mt-1">{{ importError }}</p>
          <div class="flex justify-end gap-2 mt-3">
            <button class="btn btn-ghost btn-sm" @click="showImport = false">{{ t('agentSettings.exportImport.cancel') }}</button>
            <button class="btn btn-primary btn-sm" @click="handleImport" :disabled="!importContent.trim() || importLoading">
              <IconUpload :size="14" />
              {{ t('agentSettings.exportImport.confirm') }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- ==================== Appearance Tab ==================== -->
    <div v-if="tab === 'appearance'" class="space-y-4">
      <div class="bg-base-100 border border-base-300 rounded-xl p-5">
        <h2 class="text-base font-semibold mb-4 flex items-center gap-2">
          <IconPalette :size="18" />
          Theme
        </h2>
        <div class="grid grid-cols-3 gap-3">
          <button class="flex flex-col items-center gap-2 p-4 rounded-xl border-2 transition-all"
            :class="theme === 'light' ? 'border-primary bg-primary/5' : 'border-base-300 hover:border-base-content/30'"
            @click="setTheme('light')">
            <div class="w-full h-16 rounded-lg bg-base-100 border border-base-300 flex items-center justify-center">
              <IconSun :size="24" />
            </div>
            <span class="text-sm font-medium">Light</span>
          </button>
          <button class="flex flex-col items-center gap-2 p-4 rounded-xl border-2 transition-all"
            :class="theme === 'dark' ? 'border-primary bg-primary/5' : 'border-base-300 hover:border-base-content/30'"
            @click="setTheme('dark')">
            <div class="w-full h-16 rounded-lg bg-neutral text-neutral-content flex items-center justify-center">
              <IconMoon :size="24" />
            </div>
            <span class="text-sm font-medium">Dark</span>
          </button>
          <button class="flex flex-col items-center gap-2 p-4 rounded-xl border-2 transition-all"
            :class="theme === 'system' ? 'border-primary bg-primary/5' : 'border-base-300 hover:border-base-content/30'"
            @click="setTheme('system')">
            <div class="w-full h-16 rounded-lg bg-gradient-to-br from-base-100 to-neutral flex items-center justify-center border border-base-300">
              <IconDeviceDesktop :size="24" />
            </div>
            <span class="text-sm font-medium">System</span>
          </button>
        </div>
      </div>

      <!-- Analytics Consent -->
      <div class="bg-base-100 border border-base-300 rounded-xl p-5">
        <h2 class="text-base font-semibold mb-4 flex items-center gap-2">
          <IconChartBar :size="18" />
          Analytics
        </h2>
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium">Analytics Consent</p>
            <p class="text-xs text-base-content/60 mt-0.5">Help us improve by sending anonymous usage data</p>
          </div>
          <input type="checkbox" class="toggle toggle-primary toggle-sm" v-model="analyticsConsent" @change="saveAnalyticsConsent" />
        </div>
      </div>
    </div>

    <!-- ==================== Language Tab ==================== -->
    <div v-if="tab === 'language'" class="space-y-4">
      <div class="bg-base-100 border border-base-300 rounded-xl p-5">
        <h2 class="text-base font-semibold mb-4 flex items-center gap-2">
          <IconLanguage :size="18" />
          Language
        </h2>
        <div class="space-y-2">
          <label class="flex items-center gap-3 p-3 rounded-lg cursor-pointer hover:bg-base-200 transition-colors"
            :class="locale === 'zh-CN' ? 'bg-primary/5 border border-primary/20' : ''">
            <input type="radio" name="locale" value="zh-CN" class="radio radio-primary radio-sm" v-model="locale" @change="saveLocale" />
            <div>
              <span class="text-sm font-medium">简体中文</span>
              <span class="text-xs text-base-content/50 ml-2">zh-CN</span>
            </div>
          </label>
          <label class="flex items-center gap-3 p-3 rounded-lg cursor-pointer hover:bg-base-200 transition-colors"
            :class="locale === 'en' ? 'bg-primary/5 border border-primary/20' : ''">
            <input type="radio" name="locale" value="en" class="radio radio-primary radio-sm" v-model="locale" @change="saveLocale" />
            <div>
              <span class="text-sm font-medium">English</span>
              <span class="text-xs text-base-content/50 ml-2">en</span>
            </div>
          </label>
          <label class="flex items-center gap-3 p-3 rounded-lg cursor-pointer hover:bg-base-200 transition-colors"
            :class="locale === 'ja' ? 'bg-primary/5 border border-primary/20' : ''">
            <input type="radio" name="locale" value="ja" class="radio radio-primary radio-sm" v-model="locale" @change="saveLocale" />
            <div>
              <span class="text-sm font-medium">日本語</span>
              <span class="text-xs text-base-content/50 ml-2">ja</span>
            </div>
          </label>
        </div>
        <p class="text-xs text-base-content/50 mt-3">Select interface language. Some languages may use fallback translations.</p>
      </div>
    </div>

    <!-- ==================== About Tab ==================== -->
    <div v-if="tab === 'about'" class="space-y-4">
      <!-- Version Info -->
      <div class="bg-base-100 border border-base-300 rounded-xl p-5">
        <h2 class="text-base font-semibold mb-4 flex items-center gap-2">
          <IconInfoCircle :size="18" />
          About
        </h2>
        <div class="space-y-3 text-sm">
          <div class="flex items-center justify-between">
            <span class="text-base-content/70">SuperTool Version</span>
            <span class="font-mono text-xs">{{ appVersion }}</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-base-content/70">Hermes Agent</span>
            <span class="font-mono text-xs">{{ configInfo?.version || '-' }}</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-base-content/70">Config Path</span>
            <code class="text-xs bg-base-200 px-2 py-1 rounded truncate max-w-[240px]">{{ configInfo?.hermesHome || '-' }}/config.yaml</code>
          </div>
        </div>
      </div>


      <!-- Community -->
      <div class="bg-base-100 border border-base-300 rounded-xl p-5">
        <h2 class="text-base font-semibold mb-3 flex items-center gap-2">
          <IconUsers :size="18" />
          Community
        </h2>
        <a :href="TELEGRAM_URL" target="_blank" class="flex items-center gap-3 p-3 rounded-lg hover:bg-base-200 transition-colors group cursor-pointer">
          <div class="w-9 h-9 rounded-lg bg-primary/10 flex items-center justify-center shrink-0">
            <IconSend :size="18" class="text-primary" />
          </div>
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium group-hover:text-primary transition-colors">Telegram Community</p>
            <p class="text-xs text-base-content/50 truncate">Join the discussion</p>
          </div>
          <IconExternalLink :size="16" class="text-base-content/30 shrink-0" />
        </a>
      </div>

      <!-- Tech Stack -->
      <div class="bg-base-100 border border-base-300 rounded-xl p-5">
        <h2 class="text-base font-semibold mb-3 flex items-center gap-2">
          <IconCode :size="18" />
          Tech Stack
        </h2>
        <div class="flex flex-wrap gap-2">
          <span class="badge badge-outline badge-sm">Tauri</span>
          <span class="badge badge-outline badge-sm">Rust</span>
          <span class="badge badge-outline badge-sm">Vue 3</span>
          <span class="badge badge-outline badge-sm">TypeScript</span>
          <span class="badge badge-outline badge-sm">SQLite</span>
          <span class="badge badge-outline badge-sm">Tailwind CSS</span>
          <span class="badge badge-outline badge-sm">daisyUI</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'SettingsPage' })
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  IconRefresh,
  IconSettings,
  IconPalette,
  IconLanguage,
  IconInfoCircle,
  IconAlertCircle,
  IconCheck,
  IconFileExport,
  IconDownload,
  IconUpload,
  IconSun,
  IconMoon,
  IconDeviceDesktop,
  IconChartBar,
  IconUsers,
  IconSend,
  IconExternalLink,
  IconCode,
  IconServer,
  IconKey,
} from '@tabler/icons-vue'
import { getTauriAPI } from '@/utils/tauri-api'
import { useSettingsStore } from '@/utils/settings'
import { useAgentModeStore } from '@/stores/agentModeStore'
import { invoke } from '@tauri-apps/api/core'
import type { HermesConfigInfo } from '@/types'

const { t, locale } = useI18n()
const settingsStore = useSettingsStore()
const agentModeStore = useAgentModeStore()
const isClawMode = computed(() => agentModeStore.mode === 'claw')

const TELEGRAM_URL = 'https://t.me/hermes_agent_desktop'

const loading = ref(false)
const error = ref('')
const successMsg = ref('')
const tab = ref<'general' | 'appearance' | 'language' | 'about'>('general')
const configInfo = ref<HermesConfigInfo | null>(null)
const appVersion = ref('')
const theme = ref<'light' | 'dark' | 'system'>('light')
const analyticsConsent = ref(false)

// Export/Import
const exportContent = ref<string | null>(null)
const exportLoading = ref(false)
const copied = ref(false)
const showImport = ref(false)
const importContent = ref('')
const importError = ref('')
const importLoading = ref(false)

// API Server
const apiRunning = ref(false)
const currentApiKey = ref('')
const apiKeyDisplay = ref('')
const showApiKeyModal = ref(false)
const newApiKey = ref('')
const savingApiKey = ref(false)

async function loadAppVersion() {
  try {
    const api = getTauriAPI()
    appVersion.value = await api.getAppVersion()
  } catch {
    appVersion.value = '-'
  }
}

async function loadConfig() {
  loading.value = true
  error.value = ''
  try {
    const api = getTauriAPI()
    configInfo.value = await api.getHermesConfigInfo()

    // Load theme
    const savedTheme = localStorage.getItem('theme') || 'light'
    theme.value = savedTheme as 'light' | 'dark' | 'system'

    // Load analytics consent
    const consent = localStorage.getItem('analytics-consent')
    analyticsConsent.value = consent === 'true'
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    loading.value = false
  }
}

async function setTheme(newTheme: 'light' | 'dark' | 'system') {
  theme.value = newTheme
  localStorage.setItem('theme', newTheme)

  let appliedTheme: string
  if (newTheme === 'system') {
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
    appliedTheme = prefersDark ? 'sunset' : 'cupcake'
  } else {
    appliedTheme = newTheme === 'dark' ? 'sunset' : 'cupcake'
  }
  document.documentElement.setAttribute('data-theme', appliedTheme)
  await settingsStore.updateAndSaveSettings({ theme: appliedTheme as 'cupcake' | 'sunset' })
}

async function saveAnalyticsConsent() {
  localStorage.setItem('analytics-consent', String(analyticsConsent.value))
}

async function saveLocale() {
  try {
    await settingsStore.updateAndSaveSettings({ language: locale.value as 'zh-CN' | 'en-US' })
  } catch {
    // Fall back to localStorage persist
    localStorage.setItem('locale', locale.value)
  }
}

async function handleExport() {
  exportLoading.value = true
  try {
    const api = getTauriAPI()
    const result = await api.exportHermesConfig()
    if (result?.success && typeof result?.content === 'string') {
      exportContent.value = result.content
    } else {
      exportContent.value = result?.message || 'No config content'
    }
  } catch (e: unknown) {
    error.value = t('agentSettings.exportImport.exportFailed', { error: e instanceof Error ? e.message : String(e) })
  } finally {
    exportLoading.value = false
  }
}

async function copyExport() {
  if (exportContent.value) {
    try {
      await navigator.clipboard.writeText(exportContent.value)
      copied.value = true
      setTimeout(() => { copied.value = false }, 2000)
    } catch {
      // Fallback: select text
      error.value = 'Failed to copy'
    }
  }
}

async function handleImport() {
  if (!importContent.value.trim()) {return}
  importLoading.value = true
  importError.value = ''
  try {
    const api = getTauriAPI()
    const result = await api.importHermesConfig(importContent.value)
    if (result?.success) {
      successMsg.value = t('agentSettings.exportImport.importSuccess')
      showImport.value = false
      importContent.value = ''
      setTimeout(() => { successMsg.value = '' }, 3000)
      // Reload config info
      await loadConfig()
    } else {
      importError.value = result?.message || t('agentSettings.exportImport.importFailed', { error: 'Unknown error' })
    }
  } catch (e: unknown) {
    importError.value = t('agentSettings.exportImport.importFailed', { error: e instanceof Error ? e.message : String(e) })
  } finally {
    importLoading.value = false
  }
}

async function loadApiStatus() {
  try {
    const result = await invoke<{ installed: boolean; configured: boolean; running: boolean; api_key: string }>('agent_api_server_status')
    apiRunning.value = result.running
    currentApiKey.value = result.api_key || ''
    if (currentApiKey.value && currentApiKey.value.length > 8) {
      apiKeyDisplay.value = currentApiKey.value.slice(0, 4) + '...' + currentApiKey.value.slice(-4)
    } else {
      apiKeyDisplay.value = currentApiKey.value || '未配置'
    }
  } catch {
    apiRunning.value = false
    apiKeyDisplay.value = '未配置'
  }
}

async function saveApiKey() {
  savingApiKey.value = true
  try {
    const result = await invoke<{ success: boolean; apiKey: string }>('agent_configure_api_server', {
      customKey: newApiKey.value || null,
    })
    if (result.success) {
      currentApiKey.value = result.apiKey
      if (result.apiKey.length > 8) {
        apiKeyDisplay.value = result.apiKey.slice(0, 4) + '...' + result.apiKey.slice(-4)
      } else {
        apiKeyDisplay.value = result.apiKey
      }
      showApiKeyModal.value = false
      newApiKey.value = ''
      await loadApiStatus()
    }
  } catch (e) {
    console.error('Failed to save API key:', e)
  }
  savingApiKey.value = false
}

const clawForm = ref({ apiKey: '', baseUrl: '', model: '', provider: '' })
const clawInfoSaved = ref({ apiKey: '', baseUrl: '', model: '', provider: '' })
const clawSaving = ref(false)
const clawSaveMsg = ref('')

async function loadClawConfig() {
  try {
    const api = getTauriAPI()
    const info = await api.clawConfigGet()
    clawInfoSaved.value = {
      apiKey: info?.apiKey || '',
      baseUrl: info?.baseUrl || '',
      model: info?.model || '',
      provider: info?.provider || '',
    }
    // 不预填 API key（脱敏值会覆盖真实 key），仅填其他字段
    if (info?.hasApiKey) {
      clawForm.value.apiKey = ''
      clawForm.value.baseUrl = info.baseUrl || ''
      clawForm.value.model = info.model || 'claude-sonnet-4-6'
      clawForm.value.provider = info.provider || ''
    }
  } catch {
    // Config not available yet
  }
}

async function saveClawConfig() {
  clawSaving.value = true
  clawSaveMsg.value = ''
  try {
    const api = getTauriAPI()
    const params: Record<string, string> = {}
    if (clawForm.value.apiKey.trim()) params.apiKey = clawForm.value.apiKey.trim()
    if (clawForm.value.baseUrl.trim()) params.baseUrl = clawForm.value.baseUrl.trim()
    if (clawForm.value.model.trim()) params.model = clawForm.value.model.trim()
    if (clawForm.value.provider.trim()) params.provider = clawForm.value.provider.trim()
    const result = await api.clawConfigSet(params as any)
    if (result?.success) {
      clawSaveMsg.value = '✅ 已保存'
      await loadClawConfig() // 刷新保存状态
    } else {
      clawSaveMsg.value = '❌ 保存失败'
    }
  } catch (e: any) {
    clawSaveMsg.value = `❌ ${e?.message || String(e)}`
  }
  clawSaving.value = false
  setTimeout(() => { clawSaveMsg.value = '' }, 3000)
}

onMounted(async () => {
  await loadAppVersion()
  await loadConfig()
  await loadApiStatus()
  if (isClawMode.value) {
    await loadClawConfig()
  }
})

watch(isClawMode, (claw) => {
  if (claw) { loadClawConfig() }
})
</script>
