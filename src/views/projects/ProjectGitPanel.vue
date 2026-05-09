<template>
  <div class="my-4">
    <!-- 仓库信息 - 所有仓库都显示 -->
    <div class="grid grid-cols-[repeat(auto-fit,minmax(250px,1fr))] gap-3 mb-5">
      <div v-if="project.repoPath" class="p-3 bg-base-200 rounded-lg border-l-4 border-primary">
        <div class="flex items-center justify-between mb-1.5">
          <span class="text-xs font-semibold text-base-content"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg> 本地仓库 1</span>
          <span v-if="project.branch" class="badge badge-sm badge-primary">{{ project.branch }}</span>
        </div>
        <div class="text-xs text-base-content/60 font-mono break-all">{{ project.repoPath }}</div>
      </div>
      <div v-if="project.repoPath2" class="p-3 bg-base-200 rounded-lg border-l-4 border-primary">
        <div class="flex items-center justify-between mb-1.5">
          <span class="text-xs font-semibold text-base-content"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg> 本地仓库 2</span>
          <span v-if="project.branch2" class="badge badge-sm badge-primary">{{ project.branch2 }}</span>
        </div>
        <div class="text-xs text-base-content/60 font-mono break-all">{{ project.repoPath2 }}</div>
      </div>
      <div v-if="project.gitUrl1" class="p-3 bg-base-200 rounded-lg border-l-4 border-primary">
        <div class="flex items-center mb-1.5">
          <span class="text-xs font-semibold text-base-content"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg> 远程仓库 1</span>
        </div>
        <div class="text-xs text-base-content/60 font-mono break-all">{{ project.gitUrl1 }}</div>
      </div>
      <div v-if="project.gitUrl2" class="p-3 bg-base-200 rounded-lg border-l-4 border-primary">
        <div class="flex items-center mb-1.5">
          <span class="text-xs font-semibold text-base-content"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg> 远程仓库 2</span>
        </div>
        <div class="text-xs text-base-content/60 font-mono break-all">{{ project.gitUrl2 }}</div>
      </div>
    </div>

    <!-- 提交记录 - 分仓库显示 -->
    <div class="p-5 bg-base-100 rounded-xl shadow-sm">
      <div class="flex justify-between items-center mb-4">
        <h3 class="m-0 text-base-content text-lg">📜 提交记录</h3>
        <div class="commits-filter">
          <select v-model="repoFilter" class="select select-bordered select-sm">
            <option value="all"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><line x1="6" y1="3" x2="6" y2="15"/><circle cx="18" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><path d="M18 9a9 9 0 0 1-9 9"/></svg> 全部仓库</option>
            <option v-for="repo in repos" :key="repo.key" :value="repo.key">{{ repo.label }}</option>
          </select>
        </div>
      </div>

      <div v-if="loading" class="text-center py-6 text-base-content/60 text-sm"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg> 加载中...</div>
      <div v-else-if="filteredCommits.length === 0" class="text-center py-6 text-base-content/60 text-sm"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg> 暂无提交记录</div>
      <div v-else class="max-h-[400px] overflow-y-auto">
        <div
          v-for="commit in filteredCommits"
          :key="commit.repo + commit.hash"
          class="p-3 mb-2.5 bg-base-200 rounded-lg border-l-4 border-primary"
        >
          <div class="flex justify-between items-center mb-1">
            <div class="flex items-center gap-2">
              <span v-if="commit.repo" class="text-[11px] px-1.5 py-0.5 bg-primary/10 text-primary rounded font-semibold">{{ commit.repo }}</span>
              <span class="font-mono text-xs text-base-content/60">{{ commit.hash }}</span>
            </div>
            <span class="text-xs text-base-content/60 whitespace-nowrap">{{ formatDate(commit.date) }}</span>
          </div>
          <div class="text-xs text-primary mb-1">{{ commit.author }}</div>
          <div class="text-sm text-base-content break-words">{{ commit.message }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
console.log("[views/projects/ProjectGitPanel.vue] component loaded")
import { ref, computed, onMounted } from 'vue'
import { useErrorHandler } from '../../composables/useErrorHandler'
import { getTauriAPI } from '../../utils/tauri-api'

const { handleError } = useErrorHandler()

const props = defineProps({
  project: { type: Object, required: true }
})

const commits = ref<any[]>([])
const loading = ref(false)
const repoFilter = ref('all')

// 仓库列表
const repos = computed(() => {
  const r: { key: string; label: string; path: string; branch?: string }[] = []
  if (props.project.repoPath) r.push({ key: 'repo1', label: '本地仓库 1', path: props.project.repoPath, branch: props.project.branch })
  if (props.project.repoPath2) r.push({ key: 'repo2', label: '本地仓库 2', path: props.project.repoPath2, branch: props.project.branch2 })
  if (props.project.gitUrl1) r.push({ key: 'remote1', label: '远程仓库 1', path: props.project.gitUrl1 })
  if (props.project.gitUrl2) r.push({ key: 'remote2', label: '远程仓库 2', path: props.project.gitUrl2 })
  return r
})

const filteredCommits = computed(() => {
  if (repoFilter.value === 'all') return commits.value
  return commits.value.filter(c => c.repoKey === repoFilter.value)
})

const formatDate = (dateString: string) => {
  if (!dateString) return ''
  return new Date(dateString).toLocaleDateString('zh-CN', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
}

const loadGitCommits = async () => {
  if (repos.value.length === 0) {
    commits.value = []
    return
  }

  loading.value = true
  try {
    const today = new Date()
    const firstDayOfWeek = new Date(today)
    firstDayOfWeek.setDate(today.getDate() - today.getDay())
    firstDayOfWeek.setHours(0, 0, 0, 0)
    const sinceDate = firstDayOfWeek.toISOString().split('T')[0]

    const allCommits: any[] = []

    for (const repo of repos.value) {
      try {
        const c = await getTauriAPI().getGitCommits(repo.path, sinceDate) || []
        allCommits.push(...c.map((item: any) => ({
          ...item,
          repo: repo.label.split(' ')[0], // 取 emoji
          repoKey: repo.key,
        })))
      } catch (e) {
        // 单个仓库失败不影响其他
        console.warn(`Failed to load commits for ${repo.label}:`, e)
      }
    }

    // 按日期排序
    commits.value = allCommits.sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime())
  } catch (error) {
    handleError(error, { context: '加载Git提交记录', showToast: true })
    commits.value = []
  } finally {
    loading.value = false
  }
}

onMounted(loadGitCommits)
</script>
