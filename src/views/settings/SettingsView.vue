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
        :class="tab === 'agent' ? 'tab-active' : ''"
        @click="tab = 'agent'">
        <SvgIcon name="terminal" size="16" />
        <span>Agent</span>
      </button>
      <button class="tab tab-bordered tab-sm flex items-center gap-1"
        :class="tab === 'network' ? 'tab-active' : ''"
        @click="tab = 'network'">
        <SvgIcon name="globe" size="16" />
        <span>{{ t('settings.tabs.network') }}</span>
      </button>
      <button class="tab tab-bordered tab-sm flex items-center gap-1"
        :class="tab === 'about' ? 'tab-active' : ''"
        @click="tab = 'about'">
        <SvgIcon name="info" size="16" />
        <span>{{ t('settings.tabs.about') }}</span>
      </button>
    </div>

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
          <select class="select select-bordered select-sm w-40" @change="handleLanguageChange">
            <option value="zh-CN" :selected="settingsStore.language === 'zh-CN'">中文</option>
            <option value="en-US" :selected="settingsStore.language === 'en-US'">English</option>
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

    <!-- ==================== Agent Tab ==================== -->
    <div v-if="tab === 'agent'" class="space-y-4">
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-5">
        <h2 class="flex items-center gap-2 text-base font-semibold text-base-content m-0 mb-4">
          <SvgIcon name="terminal" size="16" />
          Agent 行为配置
        </h2>
        <div class="flex flex-col gap-4">
          <!-- Max iterations -->
          <div class="flex items-center justify-between">
            <div>
              <span class="text-sm font-medium text-base-content/70">最大工具循环次数</span>
              <p class="text-xs text-base-content/50 mt-0.5">每次对话允许的最大工具调用迭代次数 (1-200)</p>
            </div>
            <input
              v-model.number="agentConfig.maxIterations"
              type="number"
              min="1"
              max="200"
              class="input input-bordered input-sm w-20 text-center"
              @change="saveAgentConfig"
            />
          </div>

          <!-- Max retries -->
          <div class="flex items-center justify-between">
            <div>
              <span class="text-sm font-medium text-base-content/70">流式错误重试次数</span>
              <p class="text-xs text-base-content/50 mt-0.5">遇到临时流式错误时自动重试次数 (0-10)</p>
            </div>
            <input
              v-model.number="agentConfig.maxRetries"
              type="number"
              min="0"
              max="10"
              class="input input-bordered input-sm w-20 text-center"
              @change="saveAgentConfig"
            />
          </div>

          <!-- Skill bytes cap -->
          <div class="flex items-center justify-between">
            <div>
              <span class="text-sm font-medium text-base-content/70">技能注入上限 (KB)</span>
              <p class="text-xs text-base-content/50 mt-0.5">Hermes 技能注入系统提示词的最大字节数 (10-2048 KB)</p>
            </div>
            <input
              v-model.number="skillBytesKB"
              type="number"
              min="10"
              max="2048"
              class="input input-bordered input-sm w-24 text-center"
              @change="saveAgentConfigWithKB"
            />
          </div>

          <!-- Tool output truncation -->
          <div class="flex items-center justify-between">
            <div>
              <span class="text-sm font-medium text-base-content/70">工具输出截断 (字符)</span>
              <p class="text-xs text-base-content/50 mt-0.5">超过此长度的工具输出将被截断 (10K-10M)</p>
            </div>
            <input
              v-model.number="toolOutputKB"
              type="number"
              min="10"
              max="10000"
              class="input input-bordered input-sm w-24 text-center"
              @change="saveAgentConfigWithOutputKB"
            />
          </div>

          <!-- Reasoning effort -->
          <div class="flex items-center justify-between">
            <div>
              <span class="text-sm font-medium text-base-content/70">推理强度</span>
              <p class="text-xs text-base-content/50 mt-0.5">控制模型的推理深度 (留空=默认)</p>
            </div>
            <select
              v-model="agentConfig.reasoningEffort"
              class="select select-bordered select-sm w-32"
              @change="saveAgentConfig"
            >
              <option value="">默认</option>
              <option value="low">Low</option>
              <option value="medium">Medium</option>
              <option value="high">High</option>
            </select>
          </div>

          <!-- Auto compaction -->
          <div class="flex items-center justify-between">
            <div>
              <span class="text-sm font-medium text-base-content/70">自动压缩上下文</span>
              <p class="text-xs text-base-content/50 mt-0.5">对话过长时自动压缩旧消息以释放上下文空间</p>
            </div>
            <input
              type="checkbox"
              class="toggle toggle-primary toggle-sm"
              v-model="agentConfig.autoCompaction"
              @change="saveAgentConfig"
            />
          </div>

          <div v-if="agentSaved" class="text-xs text-success mt-1">
            已保存
          </div>
        </div>
      </div>
    </div>

    <!-- ==================== Network Tab ==================== -->
    <div v-if="tab === 'network'" class="space-y-4">
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-5">
        <h2 class="flex items-center gap-2 text-base font-semibold text-base-content m-0 mb-4">
          <SvgIcon name="globe" size="16" />
          {{ t('settings.proxyTitle') }}
        </h2>
        <div class="flex flex-col gap-4">
          <!-- Proxy enable toggle -->
          <div class="flex items-center justify-between">
            <div>
              <span class="text-sm font-medium text-base-content/70">{{ t('settings.proxyEnabled') }}</span>
              <p class="text-xs text-base-content/50 mt-0.5">{{ t('settings.proxyHint') }}</p>
            </div>
            <input type="checkbox" class="toggle toggle-primary toggle-sm" v-model="proxyEnabled" @change="saveNetworkSettings" />
          </div>

          <!-- Proxy URL -->
          <div v-if="proxyEnabled" class="flex flex-col gap-1.5">
            <label class="text-sm font-medium text-base-content/70">{{ t('settings.httpProxy') }}</label>
            <input
              v-model="proxyUrl"
              type="text"
              class="input input-bordered w-full"
              :placeholder="t('settings.httpProxyPlaceholder')"
              @change="saveNetworkSettings"
            />
            <p class="text-xs text-base-content/50">{{ t('settings.httpProxyHint') }}</p>
          </div>

          <div v-if="networkSaved" class="text-xs text-success mt-1">
            {{ t('settings.saved') }}
          </div>
        </div>
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
          <div class="flex items-center justify-between">
            <span class="text-base-content/70">{{ t('settings.configVersion') }}</span>
            <span class="font-mono text-xs">{{ packageVersion }}</span>
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
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '@/utils/settings'
import { useTheme } from '@/utils/theme'
import { useLanguage } from '@/utils/i18n'
import { getTauriAPI } from '@/utils/tauri-api'
import { appConfigDir } from '@tauri-apps/api/path'

const { t } = useI18n()
const settingsStore = useSettingsStore()
const { toggleTheme } = useTheme()
const { switchLanguage } = useLanguage()

const tab = ref<'general' | 'notifications' | 'shortcuts' | 'agent' | 'network' | 'about'>('general')
const appVersion = ref(__APP_VERSION__ || '')
const dataDir = ref('')
const packageVersion = ref(__APP_VERSION__ || '')

// Network settings
const proxyEnabled = ref(false)
const proxyUrl = ref('')
const networkSaved = ref(false)

// Agent settings
const agentConfig = ref({
  maxIterations: 25,
  skillBytesCap: 204800,
  maxRetries: 1,
  reasoningEffort: '',
  toolOutputTruncation: 100000,
  autoCompaction: true,
})
const skillBytesKB = ref(200)
const toolOutputKB = ref(100)
const agentSaved = ref(false)

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

async function loadNetworkSettings() {
  try {
    const api = getTauriAPI()
    const enabled = await api.getSetting('proxy_enabled')
    proxyEnabled.value = enabled === 'true'
    const url = await api.getSetting('proxy_url')
    proxyUrl.value = url || ''
  } catch {
    // Defaults
  }
}

async function saveNetworkSettings() {
  try {
    const api = getTauriAPI()
    await api.setSetting('proxy_enabled', proxyEnabled.value ? 'true' : 'false')
    await api.setSetting('proxy_url', proxyUrl.value)
    networkSaved.value = true
    setTimeout(() => { networkSaved.value = false }, 2000)
  } catch {
    // ignore
  }
}

async function loadAgentConfig() {
  try {
    const api = getTauriAPI()
    const config = await api.clawConfigGet()
    agentConfig.value = {
      maxIterations: config.maxIterations,
      skillBytesCap: config.skillBytesCap,
      maxRetries: config.maxRetries,
      reasoningEffort: config.reasoningEffort,
      toolOutputTruncation: config.toolOutputTruncation,
      autoCompaction: config.autoCompaction,
    }
    skillBytesKB.value = Math.round(config.skillBytesCap / 1024)
    toolOutputKB.value = Math.round(config.toolOutputTruncation / 1024)
  } catch {
    // defaults already set
  }
}

async function saveAgentConfig() {
  try {
    const api = getTauriAPI()
    await api.clawConfigSet({
      maxIterations: agentConfig.value.maxIterations,
      skillBytesCap: agentConfig.value.skillBytesCap,
      maxRetries: agentConfig.value.maxRetries,
      reasoningEffort: agentConfig.value.reasoningEffort,
      toolOutputTruncation: agentConfig.value.toolOutputTruncation,
      autoCompaction: agentConfig.value.autoCompaction,
    })
    skillBytesKB.value = Math.round(agentConfig.value.skillBytesCap / 1024)
    toolOutputKB.value = Math.round(agentConfig.value.toolOutputTruncation / 1024)
    agentSaved.value = true
    setTimeout(() => { agentSaved.value = false }, 2000)
  } catch {
    // ignore
  }
}

function saveAgentConfigWithKB() {
  agentConfig.value.skillBytesCap = skillBytesKB.value * 1024
  saveAgentConfig()
}

function saveAgentConfigWithOutputKB() {
  agentConfig.value.toolOutputTruncation = toolOutputKB.value * 1024
  saveAgentConfig()
}

const handleLanguageChange = async (event: Event) => {
  const target = event.target as HTMLSelectElement
  await switchLanguage(target.value as 'zh-CN' | 'en-US')
}

onMounted(async () => {
  await loadAppInfo()
  await loadNetworkSettings()
  await loadAgentConfig()
})
</script>
