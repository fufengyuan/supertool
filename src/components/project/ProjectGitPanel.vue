<template>
  <div class="git-panel">
    <!-- 仓库信息 - 所有仓库都显示 -->
    <div class="repos-container">
      <div v-if="project.repoPath" class="git-repo-card">
        <div class="repo-header">
          <span class="repo-label">📂 本地仓库 1</span>
          <span v-if="project.branch" class="branch-badge">{{ project.branch }}</span>
        </div>
        <div class="repo-path">{{ project.repoPath }}</div>
      </div>
      <div v-if="project.repoPath2" class="git-repo-card">
        <div class="repo-header">
          <span class="repo-label">📂 本地仓库 2</span>
          <span v-if="project.branch2" class="branch-badge">{{ project.branch2 }}</span>
        </div>
        <div class="repo-path">{{ project.repoPath2 }}</div>
      </div>
      <div v-if="project.gitUrl1" class="git-repo-card">
        <div class="repo-header">
          <span class="repo-label">🌐 远程仓库 1</span>
        </div>
        <div class="repo-path">{{ project.gitUrl1 }}</div>
      </div>
      <div v-if="project.gitUrl2" class="git-repo-card">
        <div class="repo-header">
          <span class="repo-label">🌐 远程仓库 2</span>
        </div>
        <div class="repo-path">{{ project.gitUrl2 }}</div>
      </div>
    </div>

    <!-- 提交记录 - 分仓库显示 -->
    <div class="git-commits-section">
      <div class="commits-header">
        <h3>📜 提交记录</h3>
        <div class="commits-filter">
          <select v-model="repoFilter" class="repo-select">
            <option value="all">🔀 全部仓库</option>
            <option v-for="repo in repos" :key="repo.key" :value="repo.key">{{ repo.label }}</option>
          </select>
        </div>
      </div>

      <div v-if="loading" class="loading-commits">\u23F3 加载中...</div>
      <div v-else-if="filteredCommits.length === 0" class="no-commits">📝 暂无提交记录</div>
      <div v-else class="commits-list">
        <div
          v-for="commit in filteredCommits"
          :key="commit.repo + commit.hash"
          class="commit-item"
          :class="{ 'cross-repo': commit.repo && commit.repo.length > 1 }"
        >
          <div class="commit-header">
            <div class="commit-left">
              <span v-if="commit.repo" class="commit-repo-tag">{{ commit.repo }}</span>
              <span class="commit-hash">{{ commit.hash }}</span>
            </div>
            <span class="commit-date">{{ formatDate(commit.date) }}</span>
          </div>
          <div class="commit-author">{{ commit.author }}</div>
          <div class="commit-message">{{ commit.message }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
console.log("[components/project/ProjectGitPanel.vue] component loaded")
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

<style scoped>
.git-panel { margin: 16px 0; }

.repos-container { display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 12px; margin-bottom: 20px; }
.git-repo-card { padding: 12px; background: var(--color-base-200); border-radius: 8px; border-left: 3px solid var(--color-primary); }
.repo-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 6px; }
.repo-label { font-size: 13px; font-weight: 600; color: var(--color-base-content); }
.branch-badge { padding: 1px 8px; background: color-mix(in oklab, var(--color-primary) 10%, transparent); color: var(--color-primary); border-radius: 10px; font-size: 11px; font-weight: 600; }
.repo-path { font-size: 12px; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); font-family: 'SF Mono', monospace; word-break: break-all; }

.git-commits-section { padding: 20px; background: var(--color-base-100); border-radius: 12px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
.commits-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.commits-header h3 { margin: 0; color: var(--color-base-content); font-size: 18px; }
.repo-select { padding: 6px 12px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 6px; background: var(--color-base-200); color: var(--color-base-content); font-size: 13px; outline: none; }

.loading-commits, .no-commits { text-align: center; padding: 24px; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); font-size: 14px; }
.commits-list { max-height: 400px; overflow-y: auto; }
.commit-item { padding: 12px; margin-bottom: 10px; background: var(--color-base-200); border-radius: 8px; border-left: 3px solid var(--color-primary); }
.commit-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px; }
.commit-left { display: flex; align-items: center; gap: 8px; }
.commit-repo-tag { font-size: 11px; padding: 1px 6px; background: color-mix(in oklab, var(--color-primary) 10%, transparent); color: var(--color-primary); border-radius: 4px; font-weight: 600; }
.commit-hash { font-family: 'SF Mono', monospace; font-size: 12px; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); }
.commit-date { font-size: 12px; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); white-space: nowrap; }
.commit-author { font-size: 13px; color: var(--color-primary); margin-bottom: 4px; }
.commit-message { font-size: 14px; color: var(--color-base-content); word-break: break-word; }
</style>
