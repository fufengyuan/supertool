<template>
  <div class="max-w-4xl mx-auto">
    <!-- Claw mode overlay -->
    <div v-show="isClawMode">
      <div v-if="clawLoading" class="flex items-center justify-center py-20">
        <span class="loading loading-spinner loading-md text-primary" />
      </div>
      <div v-else-if="clawError" class="flex items-center justify-center py-20">
        <div class="text-center max-w-md px-6">
          <SvgIcon name="terminal" :size="40" class="mx-auto text-base-content/20 mb-4" />
          <p class="text-sm font-medium text-base-content/50">Claw 模型管理</p>
          <p class="text-xs text-base-content/30 mt-2">{{ clawError }}</p>
        </div>
      </div>
      <div v-else>
        <div class="flex items-center justify-between mb-6">
          <div>
            <h1 class="text-2xl font-bold">Claw 模型管理</h1>
            <p class="text-sm text-base-content/60 mt-1">来自 ~/.omp/agent/models.yaml</p>
          </div>
          <button class="btn btn-ghost btn-sm" @click="loadClawModels" :disabled="clawLoading">
            <IconRefresh :size="16" :class="{ 'animate-spin': clawLoading }" />
          </button>
        </div>
        <div v-for="(prov, name) in clawProviders" :key="name" class="mb-6">
          <h2 class="text-base font-semibold mb-3 flex items-center gap-2">
            <IconCloud :size="18" />
            {{ name }}
            <span
              class="badge badge-sm"
              :class="prov.apiKey ? 'badge-success' : 'badge-ghost'"
            >{{ prov.apiKey ? '已配置' : '未配置' }}</span>
          </h2>
          <div v-if="prov.models?.length" class="grid grid-cols-1 sm:grid-cols-2 gap-2">
            <div
              v-for="m in prov.models"
              :key="m.id || m"
              class="bg-base-100 border border-base-300 rounded-lg p-3"
            >
              <div class="text-sm font-medium">{{ m.id || m }}</div>
              <div v-if="m.maxTokens || m.contextWindow" class="text-[10px] text-base-content/50 mt-1">
                <span v-if="m.contextWindow">CTX {{ m.contextWindow }}</span>
                <span v-if="m.maxTokens" class="ml-2">max {{ m.maxTokens }}</span>
              </div>
            </div>
          </div>
          <p v-else class="text-xs text-base-content/40">无模型配置</p>
        </div>
      </div>
    </div>

    <div v-show="!isOmpMode">
    <!-- Header -->
    <div class="flex items-center justify-between mb-6">
      <div>
        <h1 class="text-2xl font-bold">模型管理</h1>
        <p class="text-sm text-base-content/60 mt-1">浏览、设置默认模型、添加或删除自定义模型</p>
      </div>
      <div class="flex items-center gap-2">
        <button class="btn btn-ghost btn-sm" @click="loadModels" :disabled="loading">
          <IconRefresh :size="16" :class="{ 'animate-spin': loading }" />
        </button>
        <button class="btn btn-primary btn-sm gap-1.5" @click="showAddModel = true">
          <IconPlus :size="14" />
          添加模型
        </button>
      </div>
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

    <!-- Loading -->
    <div v-if="loading" class="flex items-center justify-center py-16">
      <span class="loading loading-spinner loading-md text-primary"></span>
    </div>

    <template v-if="!loading">
      <!-- Default Model -->
      <div class="bg-base-100 border border-base-300 rounded-xl p-4 mb-4 flex items-center justify-between">
        <div class="flex items-center gap-2">
          <IconStar :size="18" class="text-warning" />
          <span class="text-sm font-medium">默认模型</span>
          <code v-if="defaultModel" class="text-xs bg-base-200 px-2 py-0.5 rounded font-mono">{{ defaultModel }}</code>
          <span v-else class="text-xs text-base-content/40 italic">未设置</span>
        </div>
        <span class="text-xs text-base-content/40">点击模型名称可设为默认</span>
      </div>

      <!-- Provider Models -->
      <div v-for="group in providerGroups" :key="group.provider" class="bg-base-100 border border-base-300 rounded-xl mb-3 overflow-hidden">
        <!-- Provider Header (collapsible) -->
        <button
          class="w-full flex items-center justify-between px-4 py-3 hover:bg-base-200 transition-colors cursor-pointer"
          @click="toggleGroup(group.provider)"
        >
          <div class="flex items-center gap-2">
            <IconRobot :size="18" class="text-primary shrink-0" />
            <span class="font-semibold text-sm">{{ group.provider }}</span>
            <span class="badge badge-ghost badge-xs">{{ group.models.length }} 个模型</span>
          </div>
          <IconChevronDown
            :size="16"
            class="transition-transform duration-200"
            :class="{ 'rotate-180': !expandedGroups[group.provider] }"
          />
        </button>

        <!-- Model List -->
        <div v-show="expandedGroups[group.provider]" class="border-t border-base-300 divide-y divide-base-200">
          <div
            v-for="m in group.models"
            :key="m.fullId"
            class="flex items-center justify-between px-4 py-2.5 hover:bg-base-200/50 transition-colors"
          >
            <div class="flex items-center gap-2 min-w-0 flex-1">
              <IconStar
                v-if="m.fullId === defaultModel || m.name === defaultModel"
                :size="14"
                class="text-warning shrink-0"
              />
              <div v-else class="w-3.5 shrink-0" />
              <span
                class="text-sm font-mono truncate cursor-pointer hover:text-primary transition-colors"
                :class="m.fullId === defaultModel || m.name === defaultModel ? 'text-primary font-semibold' : ''"
                :title="m.fullId"
                @click="setAsDefault(m.fullId)"
              >{{ m.name }}</span>
            </div>
            <div class="flex items-center gap-1.5 shrink-0">
              <button
                class="btn btn-ghost btn-xs text-xs"
                :disabled="m.fullId === defaultModel || m.name === defaultModel || settingDefault"
                @click="setAsDefault(m.fullId)"
              >设为默认</button>
              <button
                class="btn btn-ghost btn-xs text-xs"
                @click="showDetail(m)"
              >
                <IconInfoCircle :size="14" />
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Custom Models -->
      <div class="bg-base-100 border border-base-300 rounded-xl overflow-hidden">
        <div class="px-4 py-3 border-b border-base-300 flex items-center justify-between">
          <div class="flex items-center gap-2">
            <IconPuzzle :size="18" class="text-secondary shrink-0" />
            <span class="font-semibold text-sm">自定义模型</span>
            <span class="badge badge-ghost badge-xs">{{ customModels.length }} 个</span>
          </div>
          <button class="btn btn-ghost btn-xs gap-1" @click="showAddModel = true">
            <IconPlus :size="12" />
            添加
          </button>
        </div>

        <!-- Add model inline -->
        <div v-if="showAddModel" class="px-4 py-3 border-b border-base-200 bg-base-200/30">
          <div class="flex items-center gap-2">
            <input
              v-model="newModelName"
              type="text"
              class="input input-bordered input-sm flex-1 text-sm"
              placeholder="输入模型名称..."
              @keyup.enter="addModel"
              ref="newModelInput"
            />
            <button class="btn btn-primary btn-sm" @click="addModel" :disabled="addingModel">
              <IconCheck :size="14" />
              确认
            </button>
            <button class="btn btn-ghost btn-sm" @click="cancelAddModel">
              <IconX :size="14" />
            </button>
          </div>
          <p class="text-xs text-base-content/40 mt-1.5">输入完整的模型标识符，如 <code class="bg-base-300 px-1 rounded text-[10px]">gpt-4</code> 或 <code class="bg-base-300 px-1 rounded text-[10px]">anthropic/claude-3-opus</code></p>
        </div>

        <!-- Custom model list -->
        <div v-if="customModels.length === 0" class="px-4 py-8 text-center text-sm text-base-content/40">
          <IconBox :size="32" class="mx-auto mb-2 text-base-content/20" />
          <p>暂无自定义模型</p>
          <p class="text-xs mt-1">点击上方「添加」按钮添加</p>
        </div>
        <div v-else class="divide-y divide-base-200">
          <div
            v-for="cm in customModels"
            :key="cm"
            class="flex items-center justify-between px-4 py-2.5 hover:bg-base-200/50 transition-colors"
          >
            <div class="flex items-center gap-2 min-w-0 flex-1">
              <IconStar
                v-if="cm === defaultModel"
                :size="14"
                class="text-warning shrink-0"
              />
              <div v-else class="w-3.5 shrink-0" />
              <span
                class="text-sm font-mono truncate cursor-pointer hover:text-primary transition-colors"
                :class="cm === defaultModel ? 'text-primary font-semibold' : ''"
                @click="setAsDefault(cm)"
              >{{ cm }}</span>
            </div>
            <div class="flex items-center gap-1.5 shrink-0">
              <button
                class="btn btn-ghost btn-xs text-xs"
                :disabled="cm === defaultModel || settingDefault"
                @click="setAsDefault(cm)"
              >设为默认</button>
              <button
                class="btn btn-ghost btn-xs text-xs text-error hover:bg-error/10"
                @click="removeModel(cm)"
              >
                <IconTrash :size="14" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </template>

    <!-- Model Detail Modal -->
    <div v-if="detailModel" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="detailModel = null">
      <div class="bg-base-100 rounded-xl p-5 max-w-md w-full shadow-xl mx-4">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-lg font-bold flex items-center gap-2">
            <IconInfoCircle :size="18" />
            模型详情
          </h3>
          <button class="btn btn-ghost btn-sm btn-circle" @click="detailModel = null">
            <IconX :size="16" />
          </button>
        </div>
        <div class="space-y-3 text-sm">
          <div class="flex items-start justify-between py-2 border-b border-base-200">
            <span class="text-base-content/60">名称</span>
            <code class="text-right font-mono text-xs max-w-[200px] break-all">{{ detailModel.name }}</code>
          </div>
          <div class="flex items-start justify-between py-2 border-b border-base-200">
            <span class="text-base-content/60">提供商</span>
            <span class="text-right text-xs">{{ detailModel.provider || '-' }}</span>
          </div>
          <div class="flex items-start justify-between py-2 border-b border-base-200">
            <span class="text-base-content/60">完整 ID</span>
            <code class="text-right font-mono text-xs max-w-[200px] break-all">{{ detailModel.fullId }}</code>
          </div>
          <div class="flex items-start justify-between py-2 border-b border-base-200">
            <span class="text-base-content/60">类型</span>
            <span class="badge badge-sm" :class="detailModel.isCustom ? 'badge-secondary' : 'badge-primary'">
              {{ detailModel.isCustom ? '自定义' : '提供商' }}
            </span>
          </div>
          <div class="flex items-start justify-between py-2">
            <span class="text-base-content/60">默认模型</span>
            <span :class="(detailModel.fullId === defaultModel || detailModel.name === defaultModel) ? 'text-success' : 'text-base-content/40'">
              {{ (detailModel.fullId === defaultModel || detailModel.name === defaultModel) ? '是' : '否' }}
            </span>
          </div>
        </div>
        <div class="flex justify-end mt-4">
          <button
            v-if="detailModel.fullId !== defaultModel && detailModel.name !== defaultModel"
            class="btn btn-primary btn-sm"
            @click="setAsDefault(detailModel.fullId); detailModel = null"
          >设为默认</button>
          <button v-else class="btn btn-ghost btn-sm" @click="detailModel = null">关闭</button>
        </div>
      </div>
    </div>
  </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useAgentModeStore } from '@/stores/agentModeStore'
import { getTauriAPI } from '@/utils/tauri-api'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { invoke } from '@tauri-apps/api/core'
import {
  IconRefresh,
  IconPlus,
  IconCloud,
  IconCheck,
  IconX,
  IconAlertCircle,
  IconStar,
  IconRobot,
  IconChevronDown,
  IconPuzzle,
  IconInfoCircle,
  IconTrash,
  IconBox,
} from '@tabler/icons-vue'

interface ModelDetail {
  name: string
  provider: string
  fullId: string
  isCustom: boolean
}

interface ProviderGroup {
  provider: string
  models: Array<{ name: string; fullId: string }>
}

const loading = ref(false)
const error = ref('')
const agentModeStore = useAgentModeStore()
const isClawMode = computed(() => agentModeStore.mode === 'claw')
const clawProviders = ref<Record<string, any>>({})
const clawLoading = ref(false)
const clawError = ref('')

async function loadClawModels() {
  clawLoading.value = true
  clawError.value = ''
  try {
    const api = getTauriAPI()
    const raw = await api.clawReadModelsConfig() as any
    clawProviders.value = raw?.providers || {}
  } catch (e: any) {
    clawError.value = String(e?.message || e)
    clawProviders.value = {}
  } finally {
    clawLoading.value = false
  }
}
const successMsg = ref('')
const defaultModel = ref('')
const activeProvider = ref('')

const providerModels = ref<string[]>([])
const customModels = ref<string[]>([])

const showAddModel = ref(false)
const newModelName = ref('')
const addingModel = ref(false)
const settingDefault = ref(false)

const detailModel = ref<ModelDetail | null>(null)

const expandedGroups = ref<Record<string, boolean>>({})

// Parse provider models into groups
const providerGroups = computed<ProviderGroup[]>(() => {
  const groups = new Map<string, Array<{ name: string; fullId: string }>>()

  for (const fullId of providerModels.value) {
    const slashIdx = fullId.indexOf('/')
    if (slashIdx > 0) {
      const provider = fullId.substring(0, slashIdx)
      const name = fullId.substring(slashIdx + 1)
      if (!groups.has(provider)) {
        groups.set(provider, [])
      }
      groups.get(provider)!.push({ name, fullId })
    } else {
      if (!groups.has('other')) {
        groups.set('other', [])
      }
      groups.get('other')!.push({ name: fullId, fullId })
    }
  }

  return Array.from(groups.entries())
    .map(([provider, models]) => ({ provider, models }))
    .sort((a, b) => a.provider.localeCompare(b.provider))
})

function initExpandedGroups() {
  const providerSet = new Set<string>()
  for (const fullId of providerModels.value) {
    const slashIdx = fullId.indexOf('/')
    const provider = slashIdx > 0 ? fullId.substring(0, slashIdx) : 'other'
    providerSet.add(provider)
  }
  for (const provider of providerSet) {
    if (expandedGroups.value[provider] === undefined) {
      expandedGroups.value[provider] = true
    }
  }
}

function toggleGroup(provider: string) {
  expandedGroups.value[provider] = !expandedGroups.value[provider]
}

async function loadModels() {
  loading.value = true
  error.value = ''
  try {
    const result = await invoke<{
      customModels: string[]
      defaultModel: string | null
      activeProvider: string | null
      providerModels: string[]
    }>('agent_get_models')

    customModels.value = result.customModels || []
    providerModels.value = result.providerModels || []
    initExpandedGroups()
    defaultModel.value = result.defaultModel || ''
    activeProvider.value = result.activeProvider || ''
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    loading.value = false
  }
}

async function setAsDefault(model: string) {
  if (model === defaultModel.value) return
  if (settingDefault.value) return
  settingDefault.value = true
  try {
    await invoke('agent_set_model', { model })
    defaultModel.value = model
    successMsg.value = `已切换默认模型为「${model}」`
    clearSuccessAfterDelay()
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    settingDefault.value = false
  }
}

async function addModel() {
  const name = newModelName.value.trim()
  if (!name) {
    error.value = '请输入模型名称'
    return
  }

  addingModel.value = true
  error.value = ''
  try {
    const result = await invoke<{ success: boolean; customModels: string[] }>('agent_add_model', {
      model: name,
    })
    if (result.success) {
      customModels.value = result.customModels
      newModelName.value = ''
      showAddModel.value = false
      successMsg.value = `已添加模型「${name}」`
      clearSuccessAfterDelay()
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    addingModel.value = false
  }
}

async function removeModel(model: string) {
  if (!window.confirm(`确定删除模型「${model}」？`)) return

  error.value = ''
  try {
    const result = await invoke<{ success: boolean; customModels: string[] }>('agent_remove_model', {
      model,
    })
    if (result.success) {
      customModels.value = result.customModels
      if (defaultModel.value === model) {
        defaultModel.value = ''
      }
      successMsg.value = `已删除模型「${model}」`
      clearSuccessAfterDelay()
    } else {
      error.value = '删除模型失败：服务器返回错误'
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  }
}

function showDetail(m: { name: string; fullId: string }) {
  const slashIdx = m.fullId.indexOf('/')
  detailModel.value = {
    name: m.name,
    provider: slashIdx > 0 ? m.fullId.substring(0, slashIdx) : '-',
    fullId: m.fullId,
    isCustom: false,
  }
}

function cancelAddModel() {
  showAddModel.value = false
  newModelName.value = ''
}

let successTimer: ReturnType<typeof setTimeout> | null = null
function clearSuccessAfterDelay() {
  if (successTimer) clearTimeout(successTimer)
  successTimer = setTimeout(() => {
    successMsg.value = ''
  }, 3000)
}

onMounted(() => {
  if (isOmpMode.value) {
    loadOmpModels()
  } else {
    loadModels()
  }
})

watch(isOmpMode, (omp) => {
  if (omp) { loadOmpModels() }
  else { loadModels() }
})

onUnmounted(() => {
  if (successTimer) clearTimeout(successTimer)
})
</script>