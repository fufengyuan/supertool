<template>
  <div class="h-full flex flex-col">
    <div>
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-2 border-b border-base-content/10">
      <h1 class="text-sm font-medium">技能管理</h1>
      <div class="flex items-center gap-2">
        <button class="btn btn-sm btn-ghost" @click="loadAll" :disabled="loading">
          <SvgIcon name="refresh" size="14" />
        </button>
      </div>
    </div>

    <!-- Tab bar -->
    <div class="flex items-center px-4 pt-3 pb-2 border-b border-base-content/10 gap-4">
      <button
        v-for="t in tabs"
        :key="t.key"
        class="text-sm font-medium pb-1 border-b-2 transition-colors"
        :class="tab === t.key ? 'border-primary text-base-content' : 'border-transparent text-base-content/50 hover:text-base-content/80'"
        @click="tab = t.key"
      >
        {{ t.label }}
      </button>
    </div>

    <!-- Search + Filter -->
    <div class="px-4 py-3 border-b border-base-content/5 space-y-2">
      <div class="relative">
        <SvgIcon name="search" size="14" class="absolute left-3 top-1/2 -translate-y-1/2 text-base-content/40" />
        <input
          v-model="search"
          type="text"
          placeholder="搜索技能..."
          class="w-full h-8 pl-8 pr-3 text-sm bg-base-200 border border-base-content/10 rounded-lg outline-none focus:border-primary/50 transition-colors"
        />
      </div>
      <!-- Category pills (only in browse tab) -->
      <div v-if="tab === 'browse' && categories.length > 0" class="flex flex-wrap gap-1.5">
        <button
          v-for="cat in categories"
          :key="cat"
          class="text-[11px] px-2 py-0.5 rounded-full border transition-colors"
          :class="categoryFilter === cat
            ? 'bg-primary/20 border-primary/40 text-primary'
            : 'border-base-content/20 text-base-content/50 hover:border-base-content/40'"
          @click="categoryFilter = categoryFilter === cat ? null : cat"
        >
          {{ cat }}
        </button>
      </div>
    </div>

    <!-- Error -->
    <div v-if="error" class="mx-4 mt-3 px-3 py-2 text-xs text-error bg-error/10 rounded-lg flex items-center justify-between">
      <span>{{ error }}</span>
      <button class="btn-ghost btn-xs" @click="error = ''"><SvgIcon name="x" size="12" /></button>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="flex-1 flex items-center justify-center">
      <div class="loading-spinner" />
    </div>

    <!-- Skill Grid -->
    <div v-else class="flex-1 overflow-y-auto p-4">
      <div v-if="filteredSkills.length === 0" class="flex items-center justify-center h-full text-sm text-base-content/30">
        没有匹配的技能
      </div>
      <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
        <div
          v-for="skill in filteredSkills"
          :key="skill.path"
          class="bg-base-100 rounded-lg border border-base-content/10 p-3 hover:border-primary/30 transition-colors cursor-pointer flex flex-col"
          @click="viewDetail(skill)"
        >
          <div class="flex items-start justify-between mb-2">
            <div class="flex items-center gap-2 min-w-0">
              <SvgIcon name="brain" size="18" class="shrink-0 text-base-content/70" />
              <span class="text-sm font-medium truncate">{{ skill.name }}</span>
            </div>
            <span
              class="text-[10px] font-semibold uppercase px-1.5 py-0.5 rounded shrink-0 ml-1"
              :class="skill.source === 'installed'
                ? 'bg-success/15 text-success'
                : 'bg-info/15 text-info'"
            >
              {{ skill.source === 'installed' ? '已安装' : '可安装' }}
            </span>
          </div>
          <p class="text-xs text-base-content/50 leading-relaxed line-clamp-2 mb-2">{{ skill.description || '暂无描述' }}</p>
          <div class="mt-auto flex items-center gap-2">
            <span
              class="text-[10px] font-medium px-1.5 py-0.5 rounded bg-base-200 text-base-content/60"
            >{{ skill.category }}</span>
            <div class="ml-auto">
              <button
                v-if="skill.source === 'installed'"
                class="btn btn-xs btn-ghost text-error/70 hover:text-error"
                :disabled="actionInProgress === skill.name"
                @click.stop="handleUninstall(skill)"
              >
                {{ actionInProgress === skill.name ? '...' : '卸载' }}
              </button>
              <button
                v-else
                class="btn btn-xs btn-ghost text-primary"
                :disabled="actionInProgress === skill.name"
                @click.stop="handleInstall(skill)"
              >
                {{ actionInProgress === skill.name ? '...' : '安装' }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Detail Overlay -->
    <Teleport to="body">
      <div v-if="detailSkill" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" @click.self="closeDetail">
        <div class="bg-base-100 rounded-xl border border-base-content/10 w-full max-w-2xl max-h-[80vh] flex flex-col mx-4 shadow-xl" @click.stop>
          <div class="flex items-center justify-between px-5 py-4 border-b border-base-content/10">
            <div>
              <h2 class="text-base font-semibold">{{ detailSkill.name }}</h2>
              <span class="text-xs text-base-content/50">{{ detailSkill.category }}</span>
            </div>
            <div class="flex items-center gap-2">
              <button
                v-if="detailSkill.source === 'installed'"
                class="btn btn-sm btn-ghost text-error/70 hover:text-error"
                :disabled="actionInProgress === detailSkill.name"
                @click="handleUninstall(detailSkill)"
              >
                {{ actionInProgress === detailSkill.name ? '卸载中...' : '卸载' }}
              </button>
              <button
                v-else
                class="btn btn-sm btn-ghost text-primary"
                :disabled="actionInProgress === detailSkill.name"
                @click="handleInstall(detailSkill)"
              >
                {{ actionInProgress === detailSkill.name ? '安装中...' : '安装' }}
              </button>
              <button class="btn btn-sm btn-ghost" @click="closeDetail">
                <SvgIcon name="x" size="14" />
              </button>
            </div>
          </div>
          <div class="flex-1 overflow-y-auto p-5">
            <pre class="text-xs leading-relaxed whitespace-pre-wrap font-sans">{{ detailContent || '（空的技能文件）' }}</pre>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useAgentModeStore } from '@/stores/agentModeStore'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { getTauriAPI } from '@/utils/tauri-api'
import type { SkillInfo } from '@/types'

const tabs = [
  { key: 'installed', label: '已安装' },
  { key: 'browse', label: '浏览' },
] as const

type Tab = (typeof tabs)[number]['key']

const tab = ref<Tab>('installed')
const searchInput = ref('')
const search = ref('')
const agentModeStore = useAgentModeStore()
const isClawMode = computed(() => agentModeStore.mode === 'claw')
const categoryFilter = ref<string | null>(null)
const loading = ref(true)
const error = ref('')
const installedSkills = ref<SkillInfo[]>([])
const bundledSkills = ref<SkillInfo[]>([])
const actionInProgress = ref<string | null>(null)
const detailSkill = ref<SkillInfo | null>(null)
const detailContent = ref('')

async function loadAll() {
  loading.value = true
  error.value = ''
  try {
    const api = getTauriAPI()
    const [installed, bundled] = await Promise.all([
      api.listInstalledSkills(),
      api.listBundledSkills(),
    ])
    installedSkills.value = installed
    bundledSkills.value = bundled
  } catch (e: any) {
    error.value = e?.message || '加载失败'
  }
  loading.value = false
}

const categories = computed(() => {
  const cats = new Set(bundledSkills.value.map(s => s.category))
  return Array.from(cats).sort()
})

const filteredSkills = computed(() => {
  const list = tab.value === 'installed' ? installedSkills.value : bundledSkills.value
  const q = search.value.toLowerCase().trim()
  return list.filter(s => {
    if (categoryFilter.value && tab.value === 'browse') {
      if (s.category !== categoryFilter.value) {return false}
    }
    if (q) {
      return (
        s.name.toLowerCase().includes(q) ||
        s.description.toLowerCase().includes(q) ||
        s.category.toLowerCase().includes(q)
      )
    }
    return true
  })
})

async function viewDetail(skill: SkillInfo) {
  detailSkill.value = skill
  detailContent.value = ''
  try {
    const api = getTauriAPI()
    detailContent.value = await api.getSkillContent(skill.path)
  } catch (e: any) {
    detailContent.value = '加载失败: ' + (e?.message || '未知错误')
  }
}

function closeDetail() {
  detailSkill.value = null
  detailContent.value = ''
}

async function handleInstall(skill: SkillInfo) {
  actionInProgress.value = skill.name
  error.value = ''
  try {
    const api = getTauriAPI()
    const result = await api.installSkill(`${skill.category}/${skill.name}`)
    if (result.success) {
      closeDetail()
      await loadAll()
    } else {
      error.value = result.error || '安装失败'
    }
  } catch (e: any) {
    error.value = e?.message || '安装失败'
  }
  actionInProgress.value = null
}

async function handleUninstall(skill: SkillInfo) {
  actionInProgress.value = skill.name
  error.value = ''
  try {
    const api = getTauriAPI()
    const result = await api.uninstallSkill(`${skill.category}/${skill.name}`)
    if (result.success) {
      closeDetail()
      await loadAll()
    } else {
      error.value = result.error || '卸载失败'
    }
  } catch (e: any) {
    error.value = e?.message || '卸载失败'
  }
  actionInProgress.value = null
}

onMounted(() => {
  loadAll()
})
</script>
