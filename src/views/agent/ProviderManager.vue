<template>
  <div class="max-w-4xl mx-auto">
    <!-- Header -->
    <div class="flex items-center justify-between mb-6">
      <div>
        <h1 class="text-2xl font-bold">模型提供商</h1>
        <p class="text-sm text-base-content/60 mt-1">管理 API Key、查看提供商状态</p>
      </div>
      <button class="btn btn-ghost btn-sm" @click="loadProviders" :disabled="loading">
        <IconRefresh :size="16" :class="{ 'animate-spin': loading }" />
      </button>
    </div>

    <!-- Error -->
    <div v-if="error" class="alert alert-error mb-4 text-sm py-2">
      <IconAlertCircle :size="16" />
      <span>{{ error }}</span>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="flex items-center justify-center py-16">
      <span class="loading loading-spinner loading-md text-primary"></span>
    </div>

    <!-- Providers Grid -->
    <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div
        v-for="provider in providers"
        :key="provider.id"
        class="bg-base-100 border border-base-300 rounded-xl p-4 hover:border-primary/30 transition-colors"
      >
        <!-- Provider Header -->
        <div class="flex items-center justify-between mb-2">
          <div class="flex items-center gap-2">
            <IconRobot :size="20" class="text-primary shrink-0" />
            <span class="font-semibold text-sm">{{ provider.name }}</span>
          </div>
          <span
            class="badge badge-sm"
            :class="provider.configured ? 'badge-success' : 'badge-ghost'"
          >
            {{ provider.configured ? '已配置' : '未配置' }}
          </span>
        </div>

        <!-- Provider ID -->
        <div class="text-xs text-base-content/50 mb-3 font-mono">{{ provider.id }}</div>

        <!-- Key preview -->
        <div v-if="provider.configured && provider.apiKeyPreview" class="mb-3">
          <div class="flex items-center gap-2">
            <code class="text-xs bg-base-200 px-2 py-1 rounded font-mono">{{ provider.apiKeyPreview }}</code>
            <button
              class="btn btn-ghost btn-xs"
              @click="toggleVisibility(provider.id)"
              :title="visibleKeys.has(provider.id) ? '隐藏' : '显示'"
            >
              <IconEye v-if="!visibleKeys.has(provider.id)" :size="14" />
              <IconEyeOff v-else :size="14" />
            </button>
          </div>
        </div>

        <!-- Show full key when visible -->
        <div v-if="visibleKeys.has(provider.id) && displayedKeys[provider.id]" class="mb-3">
          <code class="text-xs bg-base-200 px-2 py-1 rounded block font-mono break-all">
            {{ displayedKeys[provider.id] }}
          </code>
        </div>

        <!-- Add / Edit Key -->
        <div v-if="editing.has(provider.id)">
          <div class="flex gap-2 items-center">
            <input
              v-model="newKeys[provider.id]"
              type="password"
              class="input input-bordered input-sm flex-1 font-mono text-xs"
              placeholder="sk-..."
              @keyup.enter="saveKey(provider.id)"
            />
            <button
              class="btn btn-primary btn-sm"
              @click="saveKey(provider.id)"
              :disabled="!newKeys[provider.id]?.trim()"
              :title="provider.configured ? '覆盖已有 Key' : '添加 Key'"
            >
              <IconCheck :size="14" />
            </button>
            <button class="btn btn-ghost btn-sm" @click="cancelEdit(provider.id)">
              <IconX :size="14" />
            </button>
          </div>
        </div>

        <!-- Actions -->
        <div class="flex gap-2 mt-3">
          <button
            v-if="!editing.has(provider.id)"
            class="btn btn-outline btn-xs gap-1"
            @click="startEdit(provider.id)"
          >
            <IconKey :size="12" />
            {{ provider.configured ? '更换 Key' : '添加 Key' }}
          </button>
          <button
            v-if="provider.configured"
            class="btn btn-ghost btn-xs text-error gap-1"
            @click="confirmRemove(provider.id)"
          >
            <IconTrash :size="12" />
            移除
          </button>
        </div>
      </div>
    </div>

    <!-- Empty state -->
    <div v-if="!loading && providers.length === 0" class="flex flex-col items-center justify-center py-16 text-center">
      <IconRobotOff :size="48" class="text-base-content/20 mb-4" />
      <p class="text-base-content/60 text-sm">暂无可用提供商</p>
      <p class="text-xs text-base-content/40 mt-1">请确保 Hermes 已安装并已配置提供商</p>
    </div>

    <!-- Remove confirmation modal -->
    <div v-if="removingProvider" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div class="bg-base-100 rounded-xl p-5 max-w-sm w-full shadow-xl">
        <h3 class="text-lg font-bold mb-2">确认移除</h3>
        <p class="text-sm text-base-content/70 mb-4">
          确定要移除「{{ getProviderName(removingProvider) }}」的 API Key 吗？
        </p>
        <div class="flex gap-2 justify-end">
          <button class="btn btn-ghost btn-sm" @click="removingProvider = null">取消</button>
          <button class="btn btn-error btn-sm" @click="removeKey" :disabled="saving">
            {{ saving ? '移除中...' : '确认移除' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'ProviderManager' })
import { ref, onMounted } from 'vue'
import {
  IconRefresh,
  IconAlertCircle,
  IconRobot,
  IconEye,
  IconEyeOff,
  IconCheck,
  IconX,
  IconKey,
  IconTrash,
  IconRobotOff,
} from '@tabler/icons-vue'
import { getTauriAPI } from '@/utils/tauri-api'
import type { ProviderInfo } from '@/types'

const loading = ref(false)
const error = ref('')
const providers = ref<ProviderInfo[]>([])
const editing = ref<Set<string>>(new Set())
const visibleKeys = ref<Set<string>>(new Set())
const newKeys = ref<Record<string, string>>({})
const displayedKeys = ref<Record<string, string>>({})
const saving = ref(false)
const removingProvider = ref<string | null>(null)

async function loadProviders() {
  loading.value = true
  error.value = ''
  try {
    const api = getTauriAPI()
    const result = await api.listProviders()
    providers.value = result.providers || []
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    loading.value = false
  }
}

function getProviderName(id: string): string {
  return providers.value.find(p => p.id === id)?.name || id
}

function startEdit(providerId: string) {
  editing.value = new Set([...editing.value, providerId])
  newKeys.value = { ...newKeys.value, [providerId]: '' }
  // Auto-reveal full key for editing
  if (!visibleKeys.value.has(providerId)) {
    toggleVisibility(providerId)
  }
}

function cancelEdit(providerId: string) {
  const next = new Set(editing.value)
  next.delete(providerId)
  editing.value = next
  const keys = { ...newKeys.value }
  delete keys[providerId]
  newKeys.value = keys
}

async function saveKey(providerId: string) {
  const key = newKeys.value[providerId]?.trim()
  if (!key) {return}
  saving.value = true
  error.value = ''
  try {
    const api = getTauriAPI()
    await api.saveProviderCredential(providerId, key)
    // Refresh to show updated status
    await loadProviders()
    cancelEdit(providerId)
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    saving.value = false
  }
}

function confirmRemove(providerId: string) {
  removingProvider.value = providerId
}

async function removeKey() {
  const providerId = removingProvider.value
  if (!providerId) {return}
  saving.value = true
  error.value = ''
  try {
    const api = getTauriAPI()
    await api.removeProviderCredential(providerId)
    await loadProviders()
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    saving.value = false
    removingProvider.value = null
  }
}

async function toggleVisibility(providerId: string) {
  const next = new Set(visibleKeys.value)
  if (next.has(providerId)) {
    next.delete(providerId)
  } else {
    next.add(providerId)
    // Fetch full key via env var
    try {
      const api = getTauriAPI()
      const provider = providers.value.find(p => p.id === providerId)
      if (provider) {
        const envKey = `${providerId.toUpperCase().replace(/-/g, '_')}_API_KEY`
        const vars = await api.readEnvVars([envKey])
        displayedKeys.value = {
          ...displayedKeys.value,
          [providerId]: vars[envKey] || '（不可读取 — Key 存储在 credential pool 中）'
        }
      }
    } catch {
      displayedKeys.value = {
        ...displayedKeys.value,
        [providerId]: providerId
      }
    }
  }
  visibleKeys.value = next
}

onMounted(() => {
  loadProviders()
})
</script>
