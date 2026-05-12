<template>
  <div class="my-4">
    <!-- 仓库信息 - 所有仓库都显示 -->
    <div class="grid grid-cols-[repeat(auto-fit,minmax(250px,1fr))] gap-3 mb-5">
      <div v-if="project.repoPath" class="p-3 bg-base-200 rounded-lg border-l-4 border-primary">
        <div class="flex items-center justify-between mb-1.5">
          <span class="text-xs font-semibold text-base-content"><SvgIcon name="folder" :size="14" class="inline-block align-text-bottom" /> 本地仓库 1</span>
          <span v-if="project.branch" class="badge badge-sm badge-primary">{{ project.branch }}</span>
        </div>
        <div class="text-xs text-base-content/60 font-mono break-all">{{ project.repoPath }}</div>
      </div>
      <div v-if="project.repoPath2" class="p-3 bg-base-200 rounded-lg border-l-4 border-primary">
        <div class="flex items-center justify-between mb-1.5">
          <span class="text-xs font-semibold text-base-content"><SvgIcon name="folder" :size="14" class="inline-block align-text-bottom" /> 本地仓库 2</span>
          <span v-if="project.branch2" class="badge badge-sm badge-primary">{{ project.branch2 }}</span>
        </div>
        <div class="text-xs text-base-content/60 font-mono break-all">{{ project.repoPath2 }}</div>
      </div>
      <div v-if="project.gitUrl1" class="p-3 bg-base-200 rounded-lg border-l-4 border-primary">
        <div class="flex items-center mb-1.5">
          <span class="text-xs font-semibold text-base-content"><SvgIcon name="globe" :size="14" class="inline-block align-text-bottom" /> 远程仓库 1</span>
        </div>
        <div class="text-xs text-base-content/60 font-mono break-all">{{ project.gitUrl1 }}</div>
      </div>
      <div v-if="project.gitUrl2" class="p-3 bg-base-200 rounded-lg border-l-4 border-primary">
        <div class="flex items-center mb-1.5">
          <span class="text-xs font-semibold text-base-content"><SvgIcon name="globe" :size="14" class="inline-block align-text-bottom" /> 远程仓库 2</span>
        </div>
        <div class="text-xs text-base-content/60 font-mono break-all">{{ project.gitUrl2 }}</div>
      </div>
    </div>

    <!-- 提交记录 - 仅支持本地仓库 -->
    <div class="p-5 bg-base-100 rounded-xl shadow-sm">
      <div class="flex justify-between items-center mb-4">
        <h3 class="m-0 text-base-content text-lg"><SvgIcon name="gitCommit" :size="18" class="inline-block align-text-bottom" /> 提交记录</h3>
        <div class="flex items-center gap-2">
          <select v-if="localRepos.length > 1" v-model="repoFilter" class="select select-bordered select-sm">
            <option value="all">全部仓库</option>
            <option v-for="repo in localRepos" :key="repo.key" :value="repo.key">{{ repo.label }}</option>
          </select>
          <button class="btn btn-ghost btn-sm" @click="loadGitCommits" title="刷新"><SvgIcon name="refresh" :size="14" /></button>
        </div>
      </div>

      <div v-if="localRepos.length === 0" class="text-center py-6 text-base-content/60 text-sm">
        <SvgIcon name="folder" :size="24" class="mx-auto mb-2 opacity-50" />
        <p class="m-0">项目未配置本地仓库路径，无法拉取 Git 提交记录</p>
      </div>
      <div v-else-if="loading" class="text-center py-6 text-base-content/60 text-sm">
        <SvgIcon name="clock" :size="14" class="inline-block align-text-bottom" /> 加载中...
      </div>
      <div v-else-if="filteredCommits.length === 0" class="text-center py-6 text-base-content/60 text-sm">
        <SvgIcon name="gitCommit" :size="24" class="mx-auto mb-2 opacity-50" />
        <p class="m-0">暂无提交记录</p>
      </div>
      <div v-else class="max-h-[80vh] overflow-y-auto space-y-2">
        <div
          v-for="commit in filteredCommits"
          :key="commit.repo + commit.hash"
          class="p-3 bg-base-200 rounded-lg border-l-4 border-primary cursor-pointer transition-colors hover:bg-base-300/50"
          @click="toggleCommitDetail(commit)"
        >
          <div class="flex justify-between items-center mb-1">
            <div class="flex items-center gap-2 min-w-0">
              <span class="text-base-content/40 shrink-0 transition-transform" :class="{ 'rotate-90': expandedCommit === commit.hash + commit.repoKey }">
                <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 18 15 12 9 6"/></svg>
              </span>
              <span v-if="commit.repo" class="text-[11px] px-1.5 py-0.5 bg-primary/10 text-primary rounded font-semibold shrink-0">{{ commit.repo }}</span>
              <span class="font-mono text-xs text-base-content/50 truncate" :title="commit.hash">{{ commit.hash.substring(0, 8) }}</span>
            </div>
            <span class="text-xs text-base-content/60 whitespace-nowrap shrink-0 ml-2">{{ formatDate(commit.date) }}</span>
          </div>
          <div class="text-xs text-primary mb-0.5">{{ commit.author }}</div>
          <div class="text-sm text-base-content">{{ commit.message }}</div>
          <!-- 展开详情 -->
          <div v-if="expandedCommit === commit.hash + commit.repoKey" class="mt-2 pt-2 border-t border-base-content/10 h-[60vh]">
            <div v-if="loadingDetail && !commitDetails[commit.hash + commit.repoKey]" class="text-xs text-base-content/50 py-2 text-center">
              <span class="loading loading-spinner loading-xs mr-1"></span> 加载中...
            </div>
            <SplitDiffViewer
              v-else
              class="h-full"
              :files="commitDetails[commit.hash + commit.repoKey]?.files || null"
              :diff="commitDetails[commit.hash + commit.repoKey]?.diff || null"
              :loading="false"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
console.log("[views/projects/ProjectGitPanel.vue] component loaded")
import { ref, computed, onMounted } from 'vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import SplitDiffViewer from '@/components/ui/SplitDiffViewer.vue'
import { useErrorHandler } from '../../composables/useErrorHandler'
import { getTauriAPI } from '../../utils/tauri-api'

const { handleError } = useErrorHandler()

const props = defineProps({
  project: { type: Object, required: true }
})

const commits = ref<any[]>([])
const loading = ref(false)
const repoFilter = ref('all')
const expandedCommit = ref<string | null>(null)
// 存储完整的 commit detail 对象（包含 files 和 diff）
const commitDetails = ref<Record<string, { files: any[]; diff: string } | null>>({})
const loadingDetail = ref(false)

// 仅本地仓库支持拉取 Git 记录
const localRepos = computed(() => {
  const r: { key: string; label: string; path: string; branch?: string }[] = []
  if (props.project.repoPath) r.push({ key: 'repo1', label: '本地仓库 1', path: props.project.repoPath, branch: props.project.branch })
  if (props.project.repoPath2) r.push({ key: 'repo2', label: '本地仓库 2', path: props.project.repoPath2, branch: props.project.branch2 })
  return r
})

const filteredCommits = computed(() => {
  if (repoFilter.value === 'all' || localRepos.value.length <= 1) return commits.value
  return commits.value.filter(c => c.repoKey === repoFilter.value)
})

const formatDate = (dateString: string) => {
  if (!dateString) return ''
  return new Date(dateString).toLocaleDateString('zh-CN', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
}

const toggleCommitDetail = async (commit: any) => {
  const key = commit.hash + commit.repoKey
  if (expandedCommit.value === key) {
    expandedCommit.value = null
    return
  }
  expandedCommit.value = key
  if (commitDetails.value[key]) return // already loaded
  loadingDetail.value = true
  try {
    const result = await getTauriAPI().getGitCommitDetail(commit.repoPath || commit.repo, commit.hash)
    // 存储完整的 commit detail 对象
    commitDetails.value[key] = result ? { files: result.files || [], diff: result.diff || '' } : null
  } catch (e: any) {
    commitDetails.value[key] = null
    console.error('加载提交详情失败:', e)
  } finally {
    loadingDetail.value = false
  }
}

const loadGitCommits = async () => {
  if (localRepos.value.length === 0) {
    commits.value = []
    return
  }

  loading.value = true
  try {
    const allCommits: any[] = []

    for (const repo of localRepos.value) {
      try {
        const c = await getTauriAPI().getGitCommits(repo.path) || []
        allCommits.push(...c.map((item: any) => ({
          ...item,
          repo: repo.label,
          repoKey: repo.key,
          repoPath: repo.path,
        })))
      } catch (e) {
        console.warn(`Failed to load commits for ${repo.label}:`, e)
      }
    }

    // 按日期排序（最新的在前）
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
