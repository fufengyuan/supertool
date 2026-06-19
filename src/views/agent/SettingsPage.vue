<template>
  <div class="w-full px-6">
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
      <button v-if="isClawMode" class="tab tab-bordered tab-sm flex items-center gap-1"
        :class="tab === 'models' ? 'tab-active' : ''"
        @click="tab = 'models'">
        <IconBrain :size="16" />
        <span>Models</span>
      </button>
      <button class="tab tab-bordered tab-sm flex items-center gap-1"
        :class="tab === 'about' ? 'tab-active' : ''"
        @click="tab = 'about'">
        <IconInfoCircle :size="16" />
        <span>About</span>
      </button>
      <!-- 子功能导航 -->
      <button class="tab tab-bordered tab-sm flex items-center gap-1"
        :class="tab === 'profiles' ? 'tab-active' : ''"
        @click="tab = 'profiles'">
        <IconUsers :size="16" />
        <span>配置文件</span>
      </button>
      <button class="tab tab-bordered tab-sm flex items-center gap-1"
        :class="tab === 'providers' ? 'tab-active' : ''"
        @click="tab = 'providers'">
        <IconBuildingStore :size="16" />
        <span>模型提供商</span>
      </button>
      <button class="tab tab-bordered tab-sm flex items-center gap-1"
        :class="tab === 'tools' ? 'tab-active' : ''"
        @click="tab = 'tools'">
        <IconTools :size="16" />
        <span>工具</span>
      </button>
      <button class="tab tab-bordered tab-sm flex items-center gap-1"
        :class="tab === 'cron' ? 'tab-active' : ''"
        @click="tab = 'cron'">
        <IconClock :size="16" />
        <span>定时任务</span>
      </button>
      <button class="tab tab-bordered tab-sm flex items-center gap-1"
        :class="tab === 'skills' ? 'tab-active' : ''"
        @click="tab = 'skills'">
        <IconBrain :size="16" />
        <span>技能</span>
      </button>
      <button class="tab tab-bordered tab-sm flex items-center gap-1"
        :class="tab === 'memory' ? 'tab-active' : ''"
        @click="tab = 'memory'">
        <IconCpu :size="16" />
        <span>记忆</span>
      </button>
      <button v-if="!isClawMode" class="tab tab-bordered tab-sm flex items-center gap-1"
        :class="tab === 'kanban' ? 'tab-active' : ''"
        @click="tab = 'kanban'">
        <IconLayoutColumns :size="16" />
        <span>看板</span>
      </button>
    </div>

    <!-- ==================== General Tab ==================== -->
    <div v-if="tab === 'general'" class="space-y-4">
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

    <!-- ==================== Models Tab (Claw only) ==================== -->
    <div v-if="tab === 'models'" class="space-y-4">
      <div class="bg-base-100 border border-base-300 rounded-xl p-5">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-base font-semibold flex items-center gap-2">
            <IconBrain :size="18" />
            模型管理
          </h2>
          <div class="flex items-center gap-2">
            <button class="btn btn-ghost btn-sm" @click="loadModels" :disabled="modelsLoading">
              <IconRefresh :size="16" :class="{ 'animate-spin': modelsLoading }" />
            </button>
            <button class="btn btn-primary btn-sm gap-1" @click="openAddModel">
              <IconPlus :size="14" />
              添加模型
            </button>
          </div>
        </div>

        <!-- Error/Success -->
        <div v-if="modelsError" class="alert alert-error mb-4 text-sm py-2">
          <IconAlertCircle :size="16" />
          <span>{{ modelsError }}</span>
        </div>
        <div v-if="modelsSuccess" class="alert alert-success mb-4 text-sm py-2">
          <IconCheck :size="16" />
          <span>{{ modelsSuccess }}</span>
        </div>

        <!-- Loading -->
        <div v-if="modelsLoading" class="flex items-center justify-center py-12">
          <span class="loading loading-spinner loading-md text-primary" />
        </div>

        <!-- Model List -->
        <div v-else-if="models.length === 0" class="text-center py-12">
          <IconBox :size="32" class="mx-auto mb-2 text-base-content/20" />
          <p class="text-sm text-base-content/50">暂无配置的模型</p>
          <p class="text-xs text-base-content/30 mt-1">点击「添加模型」按钮开始配置</p>
        </div>

        <div v-else class="space-y-3">
          <div
            v-for="(m, idx) in models"
            :key="idx"
            class="border border-base-300 rounded-lg p-4 hover:bg-base-200/30 transition-colors"
            :class="{ 'border-primary/30 bg-primary/5': m.model === activeModel }"
          >
            <div class="flex items-start justify-between mb-2">
              <div class="flex items-center gap-2 min-w-0">
                <IconStar v-if="m.model === activeModel" :size="16" class="text-warning shrink-0" />
                <span class="font-semibold text-sm truncate">{{ m.name || m.model }}</span>
                <span v-if="m.model === activeModel" class="badge badge-primary badge-xs">当前</span>
              </div>
              <div class="flex items-center gap-1 shrink-0">
                <button class="btn btn-ghost btn-xs" @click="openEditModel(m)">
                  <IconEdit :size="14" />
                </button>
                <button class="btn btn-ghost btn-xs text-error hover:bg-error/10" @click="deleteModel(idx)">
                  <IconTrash :size="14" />
                </button>
                <button
                  v-if="m.model !== activeModel"
                  class="btn btn-ghost btn-xs text-primary"
                  @click="setActiveModel(m.model)"
                >
                  设为当前
                </button>
              </div>
            </div>
            <div class="grid grid-cols-2 sm:grid-cols-3 gap-x-4 gap-y-1.5 text-xs">
              <div>
                <span class="text-base-content/50">模型 ID</span>
                <code class="ml-1 font-mono">{{ m.model }}</code>
              </div>
              <div>
                <span class="text-base-content/50">提供商</span>
                <span class="ml-1">{{ m.provider || '-' }}</span>
              </div>
              <div class="col-span-2 sm:col-span-1 truncate">
                <span class="text-base-content/50">Base URL</span>
                <code class="ml-1 font-mono text-[10px]">{{ m.baseUrl || '-' }}</code>
              </div>
              <div>
                <span class="text-base-content/50">上下文窗口</span>
                <span class="ml-1">{{ m.contextWindow?.toLocaleString() || '-' }}</span>
              </div>
              <div>
                <span class="text-base-content/50">最大输出</span>
                <span class="ml-1">{{ m.maxTokens?.toLocaleString() || '-' }}</span>
              </div>
              <div>
                <span class="text-base-content/50">压缩阈值</span>
                <span class="ml-1">{{ m.compactionThreshold?.toLocaleString() || '-' }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Add/Edit Model Modal -->
      <div v-if="showModelModal" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" @click.self="showModelModal = false">
        <div class="bg-base-100 rounded-xl p-5 w-full max-w-md shadow-2xl mx-4 max-h-[85vh] overflow-y-auto">
          <div class="flex items-center justify-between mb-4">
            <h3 class="text-lg font-bold flex items-center gap-2">
              <IconBrain :size="18" />
              {{ editingIndex >= 0 ? '编辑模型' : '添加模型' }}
            </h3>
            <button class="btn btn-ghost btn-sm btn-circle" @click="showModelModal = false">
              <IconX :size="16" />
            </button>
          </div>

          <div class="space-y-3">
            <div>
              <label class="text-xs text-base-content/70 mb-1 block">名称 *</label>
              <input v-model="form.name" type="text" class="input input-bordered input-sm w-full" placeholder="如：Claude Sonnet 4" />
            </div>
            <div>
              <label class="text-xs text-base-content/70 mb-1 block">模型 ID *</label>
              <input v-model="form.model" type="text" class="input input-bordered input-sm w-full font-mono" placeholder="如：claude-sonnet-4-6" />
            </div>
            <div class="grid grid-cols-2 gap-3">
              <div>
                <label class="text-xs text-base-content/70 mb-1 block">提供商 *</label>
                <input v-model="form.provider" type="text" class="input input-bordered input-sm w-full" placeholder="如：anthropic" />
              </div>
              <div>
                <label class="text-xs text-base-content/70 mb-1 block">Base URL</label>
                <input v-model="form.baseUrl" type="text" class="input input-bordered input-sm w-full font-mono text-xs" placeholder="可选" />
              </div>
            </div>
            <div class="grid grid-cols-3 gap-3">
              <div>
                <label class="text-xs text-base-content/70 mb-1 block">上下文窗口</label>
                <input v-model.number="form.contextWindow" type="number" class="input input-bordered input-sm w-full" placeholder="0" min="0" />
              </div>
              <div>
                <label class="text-xs text-base-content/70 mb-1 block">最大输出</label>
                <input v-model.number="form.maxTokens" type="number" class="input input-bordered input-sm w-full" placeholder="0" min="0" />
              </div>
              <div>
                <label class="text-xs text-base-content/70 mb-1 block">压缩阈值</label>
                <input v-model.number="form.compactionThreshold" type="number" class="input input-bordered input-sm w-full" placeholder="0" min="0" />
              </div>
            </div>
          </div>

          <div v-if="formError" class="alert alert-error text-sm py-2 mt-3">
            <IconAlertCircle :size="14" />
            <span>{{ formError }}</span>
          </div>

          <div class="flex gap-2 justify-end mt-4">
            <button class="btn btn-ghost btn-sm" @click="showModelModal = false">取消</button>
            <button class="btn btn-primary btn-sm" @click="saveModel" :disabled="formSaving">
              <IconRefresh v-if="formSaving" :size="12" class="animate-spin" />
              {{ editingIndex >= 0 ? '保存修改' : '添加' }}
            </button>
          </div>
        </div>
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

    <!-- ==================== 配置文件 ==================== -->
    <div v-if="tab === 'profiles'" class="space-y-4">
      <AgentProfiles />
    </div>

    <!-- ==================== 模型提供商 ==================== -->
    <div v-if="tab === 'providers'" class="-mx-3">
      <ProviderManager />
    </div>

    <!-- ==================== 工具 ==================== -->
    <div v-if="tab === 'tools'" class="-mx-3">
      <ToolsManager />
    </div>

    <!-- ==================== 定时任务 ==================== -->
    <div v-if="tab === 'cron'" class="-mx-3">
      <CronManager />
    </div>

    <!-- ==================== 技能 ==================== -->
    <div v-if="tab === 'skills'" class="-mx-3">
      <SkillsBrowser />
    </div>

    <!-- ==================== 记忆 ==================== -->
    <div v-if="tab === 'memory'" class="-mx-3">
      <MemoryManager />
    </div>

    <!-- ==================== 看板（仅 Hermes） ==================== -->
    <div v-if="tab === 'kanban' && !isClawMode" class="-mx-3">
      <KanbanBoard />
    </div>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'SettingsPage' })
import { ref, computed, onMounted, defineAsyncComponent } from 'vue'
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
  IconBrain,
  IconEdit,
  IconPlus,
  IconTrash,
  IconBox,
  IconBuildingStore,
  IconTools,
  IconClock,
  IconCpu,
  IconLayoutColumns,
} from '@tabler/icons-vue'
import { getTauriAPI } from '@/utils/tauri-api'
import { useSettingsStore } from '@/utils/settings'
import { useAgentModeStore } from '@/stores/agentModeStore'
import { invoke } from '@tauri-apps/api/core'
import type { HermesConfigInfo } from '@/types'

// 延迟加载各子功能组件（只在对应 tab 选中时加载）
const AgentProfiles = defineAsyncComponent(() => import('./AgentProfiles.vue'))
const ProviderManager = defineAsyncComponent(() => import('./ProviderManager.vue'))
const ToolsManager = defineAsyncComponent(() => import('./ToolsManager.vue'))
const CronManager = defineAsyncComponent(() => import('./CronManager.vue'))
const SkillsBrowser = defineAsyncComponent(() => import('./SkillsBrowser.vue'))
const MemoryManager = defineAsyncComponent(() => import('./MemoryManager.vue'))
const KanbanBoard = defineAsyncComponent(() => import('@/components/kanban/KanbanBoard.vue'))

const { t, locale } = useI18n()
const settingsStore = useSettingsStore()
const agentModeStore = useAgentModeStore()
const isClawMode = computed(() => agentModeStore.mode === 'claw')

const TELEGRAM_URL = 'https://t.me/hermes_agent_desktop'

const loading = ref(false)
const error = ref('')
const successMsg = ref('')
const tab = ref<'general' | 'appearance' | 'language' | 'models' | 'about' | 'profiles' | 'providers' | 'tools' | 'cron' | 'skills' | 'memory' | 'kanban'>('general')
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

// ── Model Management (Claw mode only) ──────────────────────────────
interface ModelConfig {
  name: string
  model: string
  provider: string
  apiKey?: string
  baseUrl?: string
  contextWindow: number
  maxTokens: number
  compactionThreshold: number
}

const models = ref<ModelConfig[]>([])
const activeModel = ref('')
const modelsLoading = ref(false)
const modelsError = ref('')
const modelsSuccess = ref('')

// Add/Edit modal state
const showModelModal = ref(false)
const editingIndex = ref(-1)
const formSaving = ref(false)
const formError = ref('')
const form = ref<ModelConfig>({
  name: '',
  model: '',
  provider: '',
  apiKey: '',
  baseUrl: '',
  contextWindow: 0,
  maxTokens: 0,
  compactionThreshold: 0,
})

function resetForm() {
  form.value = { name: '', model: '', provider: '', apiKey: '', baseUrl: '', contextWindow: 0, maxTokens: 0, compactionThreshold: 0 }
  editingIndex.value = -1
  formError.value = ''
}

function openAddModel() {
  resetForm()
  showModelModal.value = true
}

function openEditModel(m: ModelConfig) {
  form.value = { ...m }
  editingIndex.value = models.value.indexOf(m)
  showModelModal.value = true
}

async function loadModels() {
  modelsLoading.value = true
  modelsError.value = ''
  try {
    const api = getTauriAPI()
    const raw = await api.clawConfigGet() as any
    models.value = (raw?.models || []) as ModelConfig[]
    activeModel.value = raw?.activeModel || ''
  } catch (e: unknown) {
    modelsError.value = e instanceof Error ? e.message : String(e)
    models.value = []
  } finally {
    modelsLoading.value = false
  }
}

async function saveModel() {
  if (!form.value.name.trim() || !form.value.model.trim() || !form.value.provider.trim()) {
    formError.value = '名称、模型 ID 和提供商为必填项'
    return
  }
  formSaving.value = true
  formError.value = ''
  try {
    if (editingIndex.value >= 0) {
      models.value[editingIndex.value] = { ...form.value }
    } else {
      models.value.push({ ...form.value })
    }
    const api = getTauriAPI()
    await api.clawConfigSet({ models: models.value, activeModel: activeModel.value } as any)
    showModelModal.value = false
    modelsSuccess.value = editingIndex.value >= 0 ? '模型已更新' : '模型已添加'
    clearModelsSuccess()
  } catch (e: unknown) {
    formError.value = e instanceof Error ? e.message : String(e)
  } finally {
    formSaving.value = false
  }
}

async function deleteModel(index: number) {
  if (!window.confirm(`确定删除模型「${models.value[index].name || models.value[index].model}」？`)) return
  modelsError.value = ''
  try {
    const deleted = models.value[index].model
    models.value.splice(index, 1)
    if (activeModel.value === deleted) {
      activeModel.value = ''
    }
    const api = getTauriAPI()
    await api.clawConfigSet({ models: models.value, activeModel: activeModel.value } as any)
    modelsSuccess.value = '模型已删除'
    clearModelsSuccess()
  } catch (e: unknown) {
    modelsError.value = e instanceof Error ? e.message : String(e)
    // Reload to restore state
    await loadModels()
  }
}

async function setActiveModel(model: string) {
  activeModel.value = model
  modelsError.value = ''
  try {
    const api = getTauriAPI()
    await api.clawConfigSet({ models: models.value, activeModel } as any)
    modelsSuccess.value = `已切换当前模型为「${model}」`
    clearModelsSuccess()
  } catch (e: unknown) {
    modelsError.value = e instanceof Error ? e.message : String(e)
  }
}

let modelsSuccessTimer: ReturnType<typeof setTimeout> | null = null
function clearModelsSuccess() {
  if (modelsSuccessTimer) clearTimeout(modelsSuccessTimer)
  modelsSuccessTimer = setTimeout(() => { modelsSuccess.value = '' }, 3000)
}

onMounted(async () => {
  await loadAppVersion()
  await loadConfig()
  await loadApiStatus()
  if (isClawMode.value) {
    await loadModels()
  }
})
</script>
