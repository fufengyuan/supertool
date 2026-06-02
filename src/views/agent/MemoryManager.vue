<template>
  <div class="max-w-3xl mx-auto">
    <!-- OMP mode overlay -->
    <div v-show="isOmpMode">
      <div class="flex items-center justify-between mb-6">
        <div>
          <h1 class="text-2xl font-bold">OMP 会话统计</h1>
          <p class="text-sm text-base-content/60 mt-1">来自 OMP history.db</p>
        </div>
        <button class="btn btn-ghost btn-sm" @click="loadOmpStats" :disabled="ompLoading">
          <IconRefresh :size="16" :class="{ 'animate-spin': ompLoading }" />
        </button>
      </div>
      <div v-if="ompLoading" class="flex items-center justify-center py-16">
        <span class="loading loading-spinner loading-md text-primary" />
      </div>
      <div v-else class="grid grid-cols-2 gap-4">
        <div class="stat bg-base-100 rounded-lg p-4 border border-base-300">
          <div class="stat-value text-2xl font-bold text-primary">{{ ompStats.sessions }}</div>
          <div class="stat-title text-xs text-base-content/60 mt-1">会话</div>
        </div>
        <div class="stat bg-base-100 rounded-lg p-4 border border-base-300">
          <div class="stat-value text-2xl font-bold text-secondary">{{ ompStats.messages }}</div>
          <div class="stat-title text-xs text-base-content/60 mt-1">消息</div>
        </div>
      </div>
    </div>

    <div v-show="!isOmpMode">
    <div class="flex items-center justify-between mb-6">
      <div>
        <h1 class="text-2xl font-bold">记忆管理</h1>
        <p class="text-sm text-base-content/60 mt-1">管理 Agent 的系统记忆和用户画像</p>
      </div>
      <button class="btn btn-ghost btn-sm" @click="loadData" :disabled="loading">
        <IconRefresh :size="16" :class="{ 'animate-spin': loading }" />
      </button>
    </div>

    <!-- Stats -->
    <div class="grid grid-cols-3 gap-4 mb-6">
      <div class="stat bg-base-100 rounded-lg p-4 border border-base-300">
        <div class="stat-value text-2xl font-bold text-primary">{{ data?.stats.totalSessions ?? 0 }}</div>
        <div class="stat-title text-xs text-base-content/60 mt-1">会话</div>
      </div>
      <div class="stat bg-base-100 rounded-lg p-4 border border-base-300">
        <div class="stat-value text-2xl font-bold text-secondary">{{ data?.stats.totalMessages ?? 0 }}</div>
        <div class="stat-title text-xs text-base-content/60 mt-1">消息</div>
      </div>
      <div class="stat bg-base-100 rounded-lg p-4 border border-base-300">
        <div class="stat-value text-2xl font-bold text-accent">{{ data?.memory.entries.length ?? 0 }}</div>
        <div class="stat-title text-xs text-base-content/60 mt-1">记忆条目</div>
      </div>
    </div>

    <!-- Capacity bars -->
    <div class="grid grid-cols-2 gap-4 mb-6">
      <div>
        <div class="flex justify-between text-xs mb-1">
          <span>Agent 记忆</span>
          <span>{{ (data?.memory.charCount ?? 0).toLocaleString() }} / {{ (data?.memory.charLimit ?? 0).toLocaleString() }} chars</span>
        </div>
        <div class="h-2 bg-base-300 rounded-full overflow-hidden">
          <div class="h-full rounded-full transition-all duration-300"
            :class="memBarClass"
            :style="{ width: memBarWidth + '%' }" />
        </div>
      </div>
      <div>
        <div class="flex justify-between text-xs mb-1">
          <span>用户画像</span>
          <span>{{ (data?.user.charCount ?? 0).toLocaleString() }} / {{ (data?.user.charLimit ?? 0).toLocaleString() }} chars</span>
        </div>
        <div class="h-2 bg-base-300 rounded-full overflow-hidden">
          <div class="h-full rounded-full transition-all duration-300"
            :class="userBarClass"
            :style="{ width: userBarWidth + '%' }" />
        </div>
      </div>
    </div>

    <!-- Error -->
    <div v-if="error" class="alert alert-error mb-4 text-sm py-2">
      <IconAlertCircle :size="16" />
      <span>{{ error }}</span>
    </div>

    <!-- Tabs -->
    <div class="tabs tabs-bordered mb-6">
      <button class="tab tab-bordered tab-sm" :class="tab === 'entries' ? 'tab-active' : ''"
        @click="tab = 'entries'">
        记忆条目
        <span v-if="data?.memory.lastModified" class="text-xs opacity-50 ml-1">
          ({{ timeAgo(data.memory.lastModified) }})
        </span>
      </button>
      <button class="tab tab-bordered tab-sm" :class="tab === 'profile' ? 'tab-active' : ''"
        @click="tab = 'profile'">
        用户画像
        <span v-if="data?.user.lastModified" class="text-xs opacity-50 ml-1">
          ({{ timeAgo(data.user.lastModified) }})
        </span>
      </button>
      <button class="tab tab-bordered tab-sm" :class="tab === 'providers' ? 'tab-active' : ''"
        @click="tab = 'providers'">
        记忆提供商
        <span v-if="memoryProvider" class="text-xs opacity-50 ml-1">
          ({{ memoryProvider }})
        </span>
      </button>
    </div>

    <!-- Loading state -->
    <div v-if="loading" class="flex justify-center py-12">
      <span class="loading loading-spinner loading-md" />
    </div>

    <!-- ==================== ENTRIES TAB ==================== -->
    <div v-if="!loading && tab === 'entries'">
      <div class="flex items-center justify-between mb-4">
        <span class="text-sm text-base-content/60">{{ data?.memory.entries.length ?? 0 }} 条记忆</span>
        <button class="btn btn-primary btn-sm" @click="showAdd = !showAdd">
          <IconPlus :size="14" />
          添加记忆
        </button>
      </div>

      <!-- Add form -->
      <div v-if="showAdd" class="bg-base-100 border border-base-300 rounded-lg p-4 mb-4">
        <textarea v-model="newEntry" class="textarea textarea-bordered w-full text-sm" rows="3"
          placeholder="输入新的记忆内容（对 Agent 有用的信息）" />
        <div class="flex justify-between items-center mt-2">
          <span class="text-xs text-base-content/40">{{ newEntry.length }} chars</span>
          <div class="flex gap-2">
            <button class="btn btn-ghost btn-sm" @click="showAdd = false; newEntry = ''">取消</button>
            <button class="btn btn-primary btn-sm" :disabled="!newEntry.trim()" @click="handleAddEntry">保存</button>
          </div>
        </div>
      </div>

      <!-- Empty state -->
      <div v-if="(data?.memory.entries.length ?? 0) === 0" class="text-center py-12 text-base-content/40">
        <IconBrain :size="40" class="mx-auto mb-3 opacity-30" />
        <p class="text-sm">暂无数记忆条目</p>
        <p class="text-xs mt-1">点击"添加记忆"添加对 Agent 有用的信息</p>
      </div>

      <!-- Entry cards -->
      <div v-for="entry in data?.memory.entries ?? []" :key="entry.index" class="bg-base-100 border border-base-300 rounded-lg p-4 mb-3">
        <!-- Edit mode -->
        <div v-if="editingIndex === entry.index" class="memory-entry-form">
          <textarea v-model="editContent" class="textarea textarea-bordered w-full text-sm" rows="3" />
          <div class="flex justify-between items-center mt-2">
            <span class="text-xs text-base-content/40">{{ editContent.length }} chars</span>
            <div class="flex gap-2">
              <button class="btn btn-ghost btn-sm" @click="editingIndex = null">取消</button>
              <button class="btn btn-primary btn-sm" @click="handleSaveEdit">保存</button>
            </div>
          </div>
        </div>
        <!-- View mode -->
        <div v-else>
          <div class="text-sm whitespace-pre-wrap break-words">{{ entry.content }}</div>
          <div class="flex items-center gap-3 mt-3">
            <button class="btn btn-ghost btn-xs text-base-content/50 hover:text-primary"
              @click="startEdit(entry.index, entry.content)">
              <IconEdit :size="13" class="mr-1" />编辑
            </button>
            <template v-if="confirmDelete === entry.index">
              <span class="text-xs text-error">确定删除？</span>
              <button class="btn btn-ghost btn-xs text-error" @click="handleDeleteEntry(entry.index)">是</button>
              <button class="btn btn-ghost btn-xs" @click="confirmDelete = null">否</button>
            </template>
            <button v-else class="btn btn-ghost btn-xs text-base-content/50 hover:text-error"
              @click="confirmDelete = entry.index">
              <IconTrash :size="13" />
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- ==================== PROFILE TAB ==================== -->
    <div v-if="!loading && tab === 'profile'">
      <div class="bg-base-100 border border-base-300 rounded-lg p-4">
        <p class="text-xs text-base-content/60 mb-3">
          用户画像告诉 Agent 关于你的信息，编辑 USER.md 文件。修改会在保存后生效。
        </p>
        <div v-if="userSaved" class="text-xs text-success mb-2 flex items-center gap-1">
          <IconCheck :size="13" /> 已保存
        </div>
        <textarea v-model="userContent" class="textarea textarea-bordered w-full text-sm font-mono" rows="8"
          placeholder="描述你自己、你的偏好、工作方式等..." @input="userEditing = true" />
        <div class="flex justify-between items-center mt-3">
          <span class="text-xs text-base-content/40">
            {{ userContent.length }} / {{ data?.user.charLimit ?? 0 }} chars
            <span v-if="userContent.length > (data?.user.charLimit ?? 0)" class="text-error">（超出限制！）</span>
          </span>
          <button v-if="userEditing" class="btn btn-primary btn-sm" @click="handleSaveUserProfile">
            <IconDeviceFloppy :size="14" class="mr-1" />保存画像
          </button>
        </div>
      </div>
    </div>

    <!-- ==================== PROVIDERS TAB ==================== -->
    <div v-if="!loading && tab === 'providers'">
      <div class="mb-4 text-sm text-base-content/60">
        记忆提供商可以为 Agent 增加外部记忆存储能力。
        <template v-if="memoryProvider">
          当前活跃提供商：<strong class="text-base-content">{{ memoryProvider }}</strong>
        </template>
        <template v-else>
          <strong>当前未启用外部记忆提供商。</strong>
        </template>
      </div>

      <div v-if="!providerData" class="text-center py-12 text-base-content/40">
        <p class="text-sm">加载提供商信息...</p>
      </div>

      <div v-else-if="providerData.providers.length === 0" class="text-center py-12 text-base-content/40">
        <p class="text-sm">未发现记忆提供商</p>
      </div>

      <div v-else class="grid gap-4">
        <div v-for="p in providerData.providers" :key="p.name"
          class="bg-base-100 border rounded-lg p-4"
          :class="p.active ? 'border-primary ring-1 ring-primary/20' : 'border-base-300'">
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center gap-2">
              <span class="font-semibold text-sm">{{ p.name }}</span>
              <span v-if="p.active" class="badge badge-primary badge-xs">活跃</span>
            </div>
            <!-- External link -->
            <a v-if="providerUrls[p.name]" :href="providerUrls[p.name]" target="_blank"
              class="text-base-content/30 hover:text-primary transition-colors" :title="`打开 ${p.name} 网站`">
              <IconExternalLink :size="14" />
            </a>
          </div>
          <p class="text-xs text-base-content/60 mb-3">{{ p.description }}</p>

          <!-- Env vars -->
          <div v-if="p.envVars.length > 0" class="space-y-2 mb-3">
            <div v-for="envKey in p.envVars" :key="envKey" class="flex flex-col gap-1">
              <label class="text-xs font-medium text-base-content/60">{{ envKey }}</label>
              <input type="password" :value="providerEnv[envKey] || ''"
                @input="(e: any) => setProviderEnvValue(envKey, (e.target as HTMLInputElement).value)"
                @blur="() => saveProviderEnv(envKey)"
                class="input input-bordered input-xs w-full"
                :placeholder="`输入 ${envKey}`" />
              <span v-if="providerSavedKey === envKey" class="text-xs text-success flex items-center gap-1">
                <IconCheck :size="10" /> 已保存
              </span>
            </div>
          </div>

          <!-- Activate/Deactivate -->
          <div class="flex gap-2">
            <button v-if="p.active" class="btn btn-outline btn-sm btn-block"
              :disabled="activating !== null"
              @click="handleDeactivate(p.name)">
              停用
            </button>
            <button v-else class="btn btn-primary btn-sm btn-block"
              :disabled="activating !== null"
              @click="handleActivate(p.name)">
              激活
            </button>
            <button v-if="!p.installed" class="btn btn-ghost btn-xs text-base-content/30 cursor-default" disabled>
              未配置 API 密钥
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { getTauriAPI } from '@/utils/tauri-api'
import { useAgentModeStore } from '@/stores/agentModeStore'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import type { MemoryInfo, MemoryProviderResult, MemoryWriteResult } from '@/types'
import {
  IconRefresh,
  IconPlus,
  IconEdit,
  IconTrash,
  IconAlertCircle,
  IconBrain,
  IconDeviceFloppy,
  IconCheck,
  IconExternalLink,
} from '@tabler/icons-vue'

const api = getTauriAPI()
const agentModeStore = useAgentModeStore()
const isOmpMode = computed(() => agentModeStore.mode === 'omp')
const ompStats = ref({ sessions: 0, messages: 0 })
const ompLoading = ref(false)

async function loadOmpStats() {
  ompLoading.value = true
  try {
    const api = getTauriAPI()
    ompStats.value = await api.ompReadStats()
  } catch {
    ompStats.value = { sessions: 0, messages: 0 }
  } finally {
    ompLoading.value = false
  }
}

const loading = ref(true)
const error = ref('')
const data = ref<MemoryInfo | null>(null)
const tab = ref<'entries' | 'profile' | 'providers'>('entries')

// Entry management
const showAdd = ref(false)
const newEntry = ref('')
const editingIndex = ref<number | null>(null)
const editContent = ref('')
const confirmDelete = ref<number | null>(null)

// Profile
const userContent = ref('')
const userEditing = ref(false)
const userSaved = ref(false)

// Providers
const providerData = ref<MemoryProviderResult | null>(null)
const memoryProvider = ref('')
const providerEnv = ref<Record<string, string>>({})
const providerSavedKey = ref<string | null>(null)
const activating = ref<string | null>(null)

const providerUrls: Record<string, string> = {
  honcho: 'https://app.honcho.dev',
  hindsight: 'https://ui.hindsight.vectorize.io',
  mem0: 'https://app.mem0.ai',
  retaindb: 'https://retaindb.com',
  supermemory: 'https://supermemory.ai',
  byterover: 'https://app.byterover.dev',
}

// Capacity bar helpers
const memBarWidth = computed(() => {
  if (!data.value) {return 0}
  return Math.min(100, Math.round((data.value.memory.charCount / data.value.memory.charLimit) * 100))
})
const userBarWidth = computed(() => {
  if (!data.value) {return 0}
  return Math.min(100, Math.round((data.value.user.charCount / data.value.user.charLimit) * 100))
})
const memBarClass = computed(() => {
  const pct = memBarWidth.value
  if (pct > 90) {return 'bg-error'}
  if (pct > 70) {return 'bg-warning'}
  return 'bg-success'
})
const userBarClass = computed(() => {
  const pct = userBarWidth.value
  if (pct > 90) {return 'bg-error'}
  if (pct > 70) {return 'bg-warning'}
  return 'bg-success'
})

function timeAgo(ts: number | null): string {
  if (!ts) {return ''}
  const diff = Math.floor(Date.now() / 1000) - ts
  if (diff < 60) {return '刚刚'}
  if (diff < 3600) {return `${Math.floor(diff / 60)}m 前`}
  if (diff < 86400) {return `${Math.floor(diff / 3600)}h 前`}
  return `${Math.floor(diff / 86400)}d 前`
}

async function loadData() {
  loading.value = true
  error.value = ''
  try {
    const [mem, prov] = await Promise.all([
      api.readMemory(),
      api.listMemoryProviders(),
    ])
    data.value = mem
    userContent.value = mem.user.content
    providerData.value = prov
    memoryProvider.value = prov.activeProvider

    // Read current env var values from the process
    const envKeys: string[] = []
    for (const p of prov.providers) {
      for (const envKey of p.envVars) {
        if (!envKeys.includes(envKey)) {envKeys.push(envKey)}
      }
    }
    if (envKeys.length > 0) {
      try {
        const envVals = await api.readEnvVars(envKeys)
        for (const [k, v] of Object.entries(envVals)) {
          providerEnv.value[k] = v || ''
        }
      } catch {
        // Env read failed — leave fields empty
      }
    }
  } catch (e: any) {
    error.value = e.message || '加载失败'
  } finally {
    loading.value = false
  }
}

async function handleAddEntry() {
  if (!newEntry.value.trim()) {return}
  error.value = ''
  try {
    const result: MemoryWriteResult = await api.addMemoryEntry(newEntry.value.trim())
    if (result.success) {
      newEntry.value = ''
      showAdd.value = false
      await loadData()
    } else {
      error.value = result.error || '添加失败'
    }
  } catch (e: any) {
    error.value = e.message || '添加失败'
  }
}

async function handleSaveEdit() {
  if (editingIndex.value === null) {return}
  error.value = ''
  try {
    const result: MemoryWriteResult = await api.updateMemoryEntry(editingIndex.value, editContent.value.trim())
    if (result.success) {
      editingIndex.value = null
      editContent.value = ''
      await loadData()
    } else {
      error.value = result.error || '更新失败'
    }
  } catch (e: any) {
    error.value = e.message || '更新失败'
  }
}

async function handleDeleteEntry(index: number) {
  try {
    await api.removeMemoryEntry(index)
    confirmDelete.value = null
    await loadData()
  } catch (e: any) {
    error.value = e.message || '删除失败'
  }
}

function startEdit(index: number, content: string) {
  editingIndex.value = index
  editContent.value = content
}

async function handleSaveUserProfile() {
  error.value = ''
  try {
    const result: MemoryWriteResult = await api.writeUserProfile(userContent.value)
    if (result.success) {
      userEditing.value = false
      userSaved.value = true
      setTimeout(() => userSaved.value = false, 2000)
      await loadData()
    } else {
      error.value = result.error || '保存失败'
    }
  } catch (e: any) {
    error.value = e.message || '保存失败'
  }
}

function setProviderEnvValue(key: string, value: string) {
  providerEnv.value[key] = value
}

async function saveProviderEnv(envKey: string) {
  const val = providerEnv.value[envKey] || ''
  error.value = ''
  try {
    const result = await api.saveEnvVar(envKey, val)
    if (result.success) {
      providerSavedKey.value = envKey
      setTimeout(() => providerSavedKey.value = null, 2000)
    } else {
      error.value = result.error || '保存失败'
    }
  } catch (e: any) {
    error.value = e.message || '保存失败'
  }
}

async function handleActivate(providerName: string) {
  activating.value = providerName
  error.value = ''
  try {
    const result: MemoryWriteResult = await api.setMemoryProvider(providerName)
    if (result.success) {
      memoryProvider.value = providerName
      if (providerData.value) {
        providerData.value.providers = providerData.value.providers.map(p => ({
          ...p,
          active: p.name === providerName,
        }))
        providerData.value.activeProvider = providerName
      }
    } else {
      error.value = result.error || '激活失败'
    }
  } catch (e: any) {
    error.value = e.message || '激活失败'
  } finally {
    activating.value = null
  }
}

async function handleDeactivate(providerName: string) {
  activating.value = providerName
  error.value = ''
  try {
    const result: MemoryWriteResult = await api.setMemoryProvider('')
    if (result.success) {
      memoryProvider.value = ''
      if (providerData.value) {
        providerData.value.providers = providerData.value.providers.map(p => ({
          ...p,
          active: false,
        }))
        providerData.value.activeProvider = ''
      }
    } else {
      error.value = result.error || '停用失败'
    }
  } catch (e: any) {
    error.value = e.message || '停用失败'
  } finally {
    activating.value = null
  }
}

onMounted(() => {
  if (isOmpMode.value) {
    loadOmpStats()
  } else {
    loadData()
  }
})

watch(isOmpMode, (omp) => {
  if (omp) { loadOmpStats() }
  else { loadData() }
})
</script>
