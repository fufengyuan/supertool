<template>
  <div class="max-w-4xl mx-auto">
    <!-- Header -->
    <div class="flex items-center justify-between mb-6">
      <div>
        <h1 class="text-2xl font-bold">{{ t('settings.title') }}</h1>
        <p class="text-sm text-base-content/60 mt-1">{{ t('settings.description') }}</p>
      </div>
    </div>

    <!-- Tabs -->
    <div class="tabs tabs-bordered mb-6">
      <button class="tab tab-bordered tab-sm flex items-center gap-1"
        :class="tab === 'general' ? 'tab-active' : ''"
        @click="tab = 'general'">
        <SvgIcon name="settings" size="16" />
        <span>{{ t('settings.tabs.general') }}</span>
      </button>
      <button class="tab tab-bordered tab-sm flex items-center gap-1"
        :class="tab === 'notifications' ? 'tab-active' : ''"
        @click="tab = 'notifications'">
        <SvgIcon name="bell" size="16" />
        <span>{{ t('settings.tabs.notifications') }}</span>
      </button>
      <button class="tab tab-bordered tab-sm flex items-center gap-1"
        :class="tab === 'shortcuts' ? 'tab-active' : ''"
        @click="tab = 'shortcuts'">
        <SvgIcon name="keyboard" size="16" />
        <span>{{ t('settings.tabs.shortcuts') }}</span>
      </button>
      <button class="tab tab-bordered tab-sm flex items-center gap-1"
        :class="tab === 'ai' ? 'tab-active' : ''"
        @click="tab = 'ai'">
        <SvgIcon name="sparkles" size="16" />
        <span>AI 模型</span>
      </button>
      <button class="tab tab-bordered tab-sm flex items-center gap-1"
        :class="tab === 'data' ? 'tab-active' : ''"
        @click="tab = 'data'">
        <SvgIcon name="download" size="16" />
        <span>数据与维护</span>
      </button>
      <button class="tab tab-bordered tab-sm flex items-center gap-1"
        :class="tab === 'about' ? 'tab-active' : ''"
        @click="tab = 'about'">
        <SvgIcon name="info" size="16" />
        <span>{{ t('settings.tabs.about') }}</span>
      </button>
    </div>

    <!-- ==================== AI Model Tab ==================== -->
    <AiModelSettings v-if="tab === 'ai'" />

    <!-- ==================== General Tab ==================== -->
    <div v-if="tab === 'general'" class="space-y-4">
      <!-- Theme -->
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-5">
        <h2 class="flex items-center gap-2 text-base font-semibold text-base-content m-0 mb-4">
          <SvgIcon name="sun" size="16" />
          {{ t('settings.theme') }}
        </h2>
        <div class="flex flex-col gap-3">
          <div class="flex items-center justify-between">
            <span class="text-sm font-medium text-base-content/70">{{ t('settings.currentTheme') }}</span>
            <div class="flex items-center gap-2">
              <span class="text-xs" :class="settingsStore.theme === 'cupcake' ? 'font-semibold text-base-content' : 'text-base-content/50'">{{ t('settings.themes.cupcake') }}</span>
              <input type="checkbox" class="toggle toggle-primary toggle-sm" :checked="settingsStore.theme === 'sunset'" @change="toggleTheme" />
              <span class="text-xs" :class="settingsStore.theme === 'sunset' ? 'font-semibold text-base-content' : 'text-base-content/50'">{{ t('settings.themes.sunset') }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Language -->
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-5">
        <h2 class="flex items-center gap-2 text-base font-semibold text-base-content m-0 mb-4">
          <SvgIcon name="globe" size="16" />
          {{ t('settings.language') }}
        </h2>
        <div class="flex items-center justify-between">
          <span class="text-sm font-medium text-base-content/70">{{ t('settings.selectLanguage') }}</span>
          <select class="select select-bordered select-sm w-40" :value="settingsStore.language" @change="handleLanguageChange">
            <option value="zh-CN">中文</option>
            <option value="en-US">English</option>
          </select>
        </div>
      </div>
    </div>

    <!-- ==================== Notifications Tab ==================== -->
    <div v-if="tab === 'notifications'">
      <NotificationSettings compact />
    </div>

    <!-- ==================== Shortcuts Tab ==================== -->
    <div v-if="tab === 'shortcuts'">
      <ShortcutSettings />
    </div>

    <!-- ==================== 数据与维护 Tab（原独立菜单页合并：备份/磁盘清理/审计） ==================== -->
    <div v-if="tab === 'data'" class="space-y-4">
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-5">
        <DataBackup />
      </div>
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-5">
        <DiskCleaner />
      </div>
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-0 overflow-hidden">
        <AuditView />
      </div>
    </div>

    <!-- ==================== About Tab ==================== -->
    <div v-if="tab === 'about'" class="space-y-4">
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-5">
        <h2 class="flex items-center gap-2 text-base font-semibold text-base-content m-0 mb-4">
          <SvgIcon name="info" size="16" />
          {{ t('settings.aboutTitle') }}
        </h2>
        <div class="flex flex-col gap-3 text-sm">
          <div class="flex items-center justify-between">
            <span class="text-base-content/70">{{ t('settings.appVersion') }}</span>
            <span class="font-mono text-xs">{{ appVersion || '-' }}</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-base-content/70">{{ t('settings.dataDirectory') }}</span>
            <code class="text-xs bg-base-200 px-2 py-1 rounded truncate max-w-[280px]">{{ dataDir || '-' }}</code>
          </div>
        </div>
      </div>

      <!-- Tech Stack -->
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-5">
        <h2 class="flex items-center gap-2 text-base font-semibold text-base-content m-0 mb-3">
          <SvgIcon name="code" size="16" />
          {{ t('settings.techStack') }}
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
defineOptions({ name: 'SettingsView' })
import SvgIcon from '@/components/ui/SvgIcon.vue'
import NotificationSettings from './NotificationSettings.vue'
import ShortcutSettings from './ShortcutSettings.vue'
import AiModelSettings from './AiModelSettings.vue'
import DataBackup from '@/views/backup/DataBackup.vue'
import DiskCleaner from '@/components/DiskCleaner.vue'
import AuditView from '@/views/audit/AuditView.vue'
import { ref, watch, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '@/utils/settings'
import { useRoute } from 'vue-router'
import { useTheme } from '@/utils/theme'
import { useLanguage } from '@/utils/i18n'
import { getTauriAPI } from '@/utils/tauri-api'
import { appConfigDir } from '@tauri-apps/api/path'

const route = useRoute()

const { t } = useI18n()
const settingsStore = useSettingsStore()
const { toggleTheme } = useTheme()
const { switchLanguage } = useLanguage()

type SettingsTab = 'general' | 'notifications' | 'shortcuts' | 'about' | 'ai' | 'data'
// 支持 /settings?tab=ai 直达（AI 助手的「去配置模型」按钮会带这个参数过来）
const initialTab = (route.query?.tab as SettingsTab) || 'general'
const tab = ref<SettingsTab>(['general', 'notifications', 'shortcuts', 'about', 'ai', 'data'].includes(initialTab) ? initialTab : 'general')
const appVersion = ref(__APP_VERSION__ || '')
const dataDir = ref('')

async function loadAppInfo() {
  try {
    const api = getTauriAPI()
    const v = await api.getAppVersion()
    if (v) {appVersion.value = v}
    dataDir.value = await appConfigDir()
  } catch {
    dataDir.value = '-'
  }
}

const handleLanguageChange = async (event: Event) => {
  const target = event.target as HTMLSelectElement
  await switchLanguage(target.value as 'zh-CN' | 'en-US')
}

watch(
  () => route.query?.tab,
  value => {
    const next = value as SettingsTab | undefined
    if (next && ['general', 'notifications', 'shortcuts', 'about', 'ai', 'data'].includes(next)) {tab.value = next}
  },
)

onMounted(async () => {
  await loadAppInfo()
})
</script>
