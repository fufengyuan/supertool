<template>
  <!-- Git 仓库图形化管理 -->
  <GitManager v-if="selectedRepo" :repo="selectedRepo" @close="closeGitManager" />

  <!-- Git 仓库列表 -->
  <div v-else class="max-w-[1200px] mx-auto">
    <div class="flex justify-between items-center mb-5">
      <h2 class="m-0 text-2xl font-semibold text-base-content">Git 仓库</h2>
      <div class="flex gap-2">
        <UiButton @click="showScanSection = !showScanSection">
          <SvgIcon name="search" :size="14" class="inline-block align-text-bottom" /> 扫描本地目录
        </UiButton>
        <UiButton @click="openAddModal">+ 添加仓库</UiButton>
      </div>
    </div>

    <!-- 扫描本地目录面板 -->
    <div v-if="showScanSection" class="mb-4 border border-base-content/10 rounded-lg bg-base-100 overflow-hidden">
      <div class="flex justify-between items-center px-3 py-2 bg-base-200/50 border-b border-base-content/10">
        <span class="text-xs font-medium text-base-content flex items-center gap-1.5">
          <SvgIcon name="folder" :size="14" />
          扫描本地目录
        </span>
        <button class="w-6 h-6 flex items-center justify-center rounded hover:bg-base-200 text-base-content/50 hover:text-base-content" @click="showScanSection = false">
          <SvgIcon name="x" :size="12" />
        </button>
      </div>
      <div class="p-3">
        <p class="m-0 mb-2 text-xs text-base-content/50">输入工作目录路径，每行一个</p>
        <textarea
          v-model="scanDirectories"
          class="w-full p-2 text-xs font-mono bg-base-100 border border-base-content/10 rounded-lg resize-y focus:border-primary focus:outline-none"
          placeholder="/home/user/projects&#10;/home/user/workspace"
          rows="3"
        ></textarea>
        <div class="flex items-center gap-3 mt-2">
          <button class="flex items-center gap-1.5 px-3 py-1.5 bg-primary text-white rounded-lg text-xs font-medium hover:bg-primary/90 transition-colors" @click="doScan" :disabled="scanning">
            <SvgIcon v-if="scanning" name="refresh" :size="12" class="animate-spin" />
            <SvgIcon v-else name="search" :size="12" />
            {{ scanning ? '扫描中...' : '扫描' }}
          </button>
          <span v-if="scanResult !== null" class="text-xs text-base-content/60">
            {{ scanResult === 0 ? '未找到仓库' : `找到 ${scanResult} 个仓库` }}
          </span>
        </div>
        <!-- 扫描结果列表 -->
        <div v-if="scannedRepos.length > 0" class="mt-3 flex flex-col gap-1.5 max-h-[200px] overflow-y-auto">
          <div v-for="repo in scannedRepos" :key="repo.path" class="flex justify-between items-center px-3 py-2 rounded-lg bg-base-200/50 border border-base-content/5">
            <div class="flex flex-col gap-0.5 min-w-0">
              <span class="text-xs font-medium text-base-content">{{ repo.name }}</span>
              <span class="text-[11px] text-base-content/40 font-mono truncate">{{ repo.path }}</span>
            </div>
            <button
              class="px-2 py-1 text-xs font-medium rounded-lg transition-colors"
              :class="isRepoAlreadyAdded(repo.path) ? 'bg-base-200 text-base-content/40' : 'bg-primary/10 text-primary hover:bg-primary hover:text-white'"
              :disabled="isRepoAlreadyAdded(repo.path)"
              @click="addScannedRepo(repo)"
            >
              {{ isRepoAlreadyAdded(repo.path) ? '已添加' : '添加' }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 搜索栏 -->
    <div class="flex gap-3 mb-4 flex-wrap items-center">
      <div class="relative flex-1 min-w-[200px]">
        <SvgIcon name="search" class="absolute left-3 top-1/2 -translate-y-1/2 text-base-content/40 w-4 h-4" />
        <input v-model="searchQuery" type="text" class="w-full h-9 pl-9 text-sm bg-base-100 border border-base-content/10 rounded-lg focus:border-primary focus:outline-none transition-colors" placeholder="搜索仓库..." />
      </div>
    </div>

    <!-- 仓库列表 - 卡片形式 -->
    <div v-if="filteredRepos.length > 0" class="flex flex-col gap-2">
      <div
        v-for="repo in filteredRepos"
        :key="repo.id"
        class="flex items-center gap-4 px-4 py-3 bg-base-100 border border-base-content/10 rounded-lg transition-all duration-150 cursor-pointer hover:border-primary/40 hover:bg-primary/5"
        @dblclick="openRepo(repo)"
      >
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2 mb-1">
            <SvgIcon name="gitBranch" :size="16" class="text-primary shrink-0" />
            <span class="text-sm font-semibold text-base-content truncate">{{ repo.name }}</span>
          </div>
          <div class="flex items-center gap-1.5 text-xs text-base-content/50 truncate" :title="repo.path">
            <SvgIcon name="folder" :size="12" class="shrink-0" />
            <span>{{ repo.path }}</span>
          </div>
        </div>

        <div class="flex flex-col gap-1 min-w-[160px] max-w-[240px]">
          <div v-if="repo.remote" class="flex items-center gap-1.5 text-xs text-base-content/50 truncate" :title="repo.remote">
            <SvgIcon name="link" :size="12" />
            <span class="truncate">{{ repo.remote }}</span>
          </div>
          <div v-if="repo.branch" class="flex items-center gap-1.5 text-xs text-base-content/50">
            <SvgIcon name="gitBranch" :size="12" />
            <span class="px-1.5 py-0.5 bg-primary/10 text-primary rounded text-[11px] font-medium">{{ repo.branch }}</span>
          </div>
          <div v-if="repo.lastOpened" class="flex items-center gap-1.5 text-xs text-base-content/40">
            <SvgIcon name="clock" :size="12" />
            <span>{{ formatTime(repo.lastOpened) }}</span>
          </div>
        </div>

        <div class="flex gap-1.5 shrink-0">
          <button class="flex items-center gap-1 px-2.5 py-1.5 bg-primary/10 text-primary rounded-lg text-xs font-medium hover:bg-primary hover:text-white transition-colors" @click="openRepo(repo)" title="打开仓库">
            <SvgIcon name="externalLink" :size="12" />
            打开
          </button>
          <button class="w-8 h-8 flex items-center justify-center rounded-lg text-base-content/50 hover:text-base-content hover:bg-base-200 transition-colors" @click="openEditModal(repo)" title="编辑">
            <SvgIcon name="pencil" :size="12" />
          </button>
          <button class="w-8 h-8 flex items-center justify-center rounded-lg text-base-content/50 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors" @click="deleteRepo(repo)" title="删除">
            <SvgIcon name="trash" :size="12" />
          </button>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <UiEmptyState v-else :text="searchQuery ? '没有找到匹配的仓库' : '暂无 Git 仓库'" :subtext="searchQuery ? '尝试其他搜索词' : '点击下方按钮添加第一个仓库'">
      <template #icon>
        <SvgIcon name="gitBranch" :size="64" stroke-width="1.5" />
      </template>
      <template #action v-if="!searchQuery">
        <UiButton @click="openAddModal">+ 添加仓库</UiButton>
      </template>
    </UiEmptyState>

    <!-- 添加/编辑仓库模态框 -->
    <UiModal
      v-model="showModal"
      :title="editingRepo ? '编辑仓库' : '添加仓库'"
      @close="resetModal"
      width="640px"
    >
      <div class="mb-4">
        <label for="repo-path" class="flex items-center gap-1.5 mb-2 text-xs font-medium text-base-content">
          <SvgIcon name="folder" :size="14" />
          本地路径 <span class="text-error">*</span>
        </label>
        <div class="flex gap-2">
          <input
            id="repo-path"
            v-model="formData.path"
            type="text"
            class="input input-bordered flex-1"
            placeholder="/path/to/your/repo"
            @input="onPathChange"
          />
          <UiButton variant="ghost" size="sm" @click="selectDirectory" :disabled="validating">
            <SvgIcon name="folder" :size="16" />
            选择
          </UiButton>
        </div>
        <div v-if="validationStatus"
          class="mt-1.5 text-xs font-medium px-2 py-1 rounded-md"
          :class="{
            'text-success bg-success/10': validationStatus.type === 'success',
            'text-error bg-error/10': validationStatus.type === 'error',
            'text-warning': validationStatus.type === 'validating'
          }">
          {{ validationStatus.message }}
        </div>
        <small class="block mt-1 text-xs text-base-content/60">输入或选择一个 Git 仓库的本地路径</small>
      </div>

      <div class="mb-4">
        <label for="repo-name" class="flex items-center gap-1.5 mb-2 text-xs font-medium text-base-content">
          <SvgIcon name="user" :size="14" />
          仓库名称
        </label>
        <input id="repo-name" v-model="formData.name" type="text" class="input input-bordered w-full" placeholder="仓库名称（留空将自动从路径提取）" />
      </div>

      <div class="flex gap-4">
        <div class="flex-1 mb-4">
          <label for="repo-remote" class="flex items-center gap-1.5 mb-2 text-xs font-medium text-base-content">
            <SvgIcon name="globe" :size="14" />
            远程 URL
          </label>
          <input id="repo-remote" v-model="formData.remote" type="text" class="input input-bordered w-full" placeholder="https://github.com/user/repo.git" />
        </div>
        <div class="flex-1 mb-4">
          <label for="repo-branch" class="flex items-center gap-1.5 mb-2 text-xs font-medium text-base-content">
            <SvgIcon name="gitBranch" :size="14" />
            当前分支
          </label>
          <input id="repo-branch" v-model="formData.branch" type="text" class="input input-bordered w-full" placeholder="main" />
        </div>
      </div>

      <template #footer>
        <UiButton variant="ghost" @click="resetModal">取消</UiButton>
        <UiButton variant="primary" @click="saveRepo" :loading="saving">
          {{ editingRepo ? '保存' : '添加' }}
        </UiButton>
      </template>
    </UiModal>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'GitRepoList' })
import { ref, computed, onMounted } from 'vue';
import UiButton from '@/components/ui/Button.vue';
import UiModal from '@/components/ui/Modal.vue';
import UiEmptyState from '@/components/ui/EmptyState.vue';
import SvgIcon from '@/components/ui/SvgIcon.vue';
import GitManager from './GitManager.vue';
import { useToast } from '../../composables/useToast'
import { useErrorHandler } from '../../composables/useErrorHandler'
import { getTauriAPI } from '../../utils/tauri-api'
import type { GitRepo } from '../../types'

interface ValidationStatus {
  type: 'success' | 'error' | 'validating';
  message: string;
}

interface FormData {
  name: string;
  path: string;
  remote: string;
  branch: string;
}

const toast = useToast();
const { handleError } = useErrorHandler();

const repos = ref<GitRepo[]>([]);
const selectedRepo = ref<GitRepo | null>(null);
const searchQuery = ref('');
const showModal = ref(false);
const editingRepo = ref<GitRepo | null>(null);
const saving = ref(false);
const validating = ref(false);
const validationStatus = ref<ValidationStatus | null>(null);
const formData = ref<FormData>({ name: '', path: '', remote: '', branch: '' });

// Scan section
const showScanSection = ref(false);
const scanDirectories = ref('');
const scanning = ref(false);
const scanResult = ref<number | null>(null);
const scannedRepos = ref<RepoScanResult[]>([]);

interface RepoScanResult {
  path: string;
  name: string;
}

let validateTimer: ReturnType<typeof setTimeout> | null = null;

const filteredRepos = computed(() => {
  if (!searchQuery.value.trim()) {return repos.value;}
  const q = searchQuery.value.toLowerCase();
  return repos.value.filter(
    (r) =>
      r.name.toLowerCase().includes(q) ||
      r.path.toLowerCase().includes(q) ||
      (r.remote && r.remote.toLowerCase().includes(q)) ||
      (r.branch && r.branch.toLowerCase().includes(q))
  );
});

const loadRepos = async () => {
  try {
    const api = getTauriAPI();
    if (!api?.getGitRepos) {
      toast.warning('Git 仓库管理功能在当前环境不可用');
      return;
    }
    const result = await api.getGitRepos();
    repos.value = Array.isArray(result) ? result : (result?.data ?? []);
  } catch (error) {
    handleError(error, { context: 'loadGitRepos' });
  }
};

const formatTime = (dateStr: string): string => {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  if (diffMins < 1) {return '刚刚';}
  if (diffMins < 60) {return `${diffMins} 分钟前`;}
  if (diffHours < 24) {return `${diffHours} 小时前`;}
  if (diffDays < 7) {return `${diffDays} 天前`;}
  return date.toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' });
};

const extractNameFromPath = (path: string): string => {
  const trimmed = path.replace(/\/+$/, '');
  const parts = trimmed.split(/[/\\]/);
  const last = parts[parts.length - 1];
  return last.replace(/\.git$/, '') || trimmed;
};

const validateRepoPath = async (path: string) => {
  if (!path.trim()) {
    validationStatus.value = null;
    return false;
  }
  try {
    const api = getTauriAPI();
    if (!api?.validateGitRepoPath) {return true;}
    validating.value = true;
    const result = await api.validateGitRepoPath(path.trim());
    if (result.valid) {
      validationStatus.value = { type: 'success', message: result.error || '✓ 有效的 Git 仓库' };
      // 自动填充名称
      if (result.name && !formData.value.name) {
        formData.value.name = result.name;
      }
      return true;
    } else {
      validationStatus.value = { type: 'error', message: `✗ ${result.error || '无效的 Git 仓库路径'}` };
      return false;
    }
  } catch {
    // 如果验证 API 不可用，跳过验证
    return true;
  } finally {
    validating.value = false;
  }
};

const onPathChange = () => {
  if (validateTimer) {clearTimeout(validateTimer);}
  validationStatus.value = null;

  if (formData.value.path.trim()) {
    // 自动填充名称
    if (!editingRepo.value && !formData.value.name) {
      formData.value.name = extractNameFromPath(formData.value.path);
    }
    // 延迟验证
    validateTimer = setTimeout(() => {
      validateRepoPath(formData.value.path);
    }, 500);
  }
};

const selectDirectory = async () => {
  try {
    const api = getTauriAPI();
    if (!api?.showOpenDialogForDirs && !api?.showOpenDialog) {
      toast.warning('目录选择功能在当前环境不可用');
      return;
    }
    let result: { canceled: boolean; filePaths: string[] };
    if (api.showOpenDialogForDirs) {
      result = await api.showOpenDialogForDirs();
    } else {
      result = await api.showOpenDialog({ properties: ['openDirectory'] });
    }
    if (!result.canceled && result.filePaths?.length > 0) {
      formData.value.path = result.filePaths[0];
      onPathChange();
    }
  } catch (error) {
    handleError(error, { context: 'selectDirectory' });
  }
};

const openAddModal = () => {
  editingRepo.value = null;
  formData.value = { name: '', path: '', remote: '', branch: '' };
  validationStatus.value = null;
  showModal.value = true;
};

const openEditModal = (repo: GitRepo) => {
  editingRepo.value = { ...repo };
  formData.value = {
    name: repo.name,
    path: repo.path,
    remote: repo.remote || '',
    branch: repo.branch || '',
  };
  validationStatus.value = null;
  showModal.value = true;
};

const resetModal = () => {
  showModal.value = false;
  editingRepo.value = null;
  formData.value = { name: '', path: '', remote: '', branch: '' };
  validationStatus.value = null;
  if (validateTimer) {clearTimeout(validateTimer);}
};

const saveRepo = async () => {
  const path = formData.value.path.trim();
  const name = formData.value.name.trim() || extractNameFromPath(path);

  if (!path) {
    toast.error('请输入仓库路径');
    return;
  }

  if (!editingRepo.value) {
    // 添加新仓库时验证路径
    const isValid = await validateRepoPath(path);
    if (!isValid) {
      toast.error('所选路径不是有效的 Git 仓库');
      return;
    }
  }

  saving.value = true;
  try {
    const api = getTauriAPI();
    if (!api) {
      toast.error('API 不可用');
      return;
    }

    const repoData = {
      name,
      path,
      remote: formData.value.remote.trim() || undefined,
      branch: formData.value.branch.trim() || undefined,
    };

    if (editingRepo.value) {
      await api.updateGitRepo(editingRepo.value.id, repoData);
      toast.success('仓库信息已更新');
    } else {
      await api.addGitRepo({ id: crypto.randomUUID(), ...repoData });
      toast.success('仓库已添加');
    }

    resetModal();
    await loadRepos();
  } catch (error) {
    handleError(error, { context: 'saveGitRepo' });
  } finally {
    saving.value = false;
  }
};

const deleteRepo = async (repo: GitRepo) => {
  if (!confirm(`确定要删除仓库「${repo.name}」吗？\n\n此操作仅从列表中移除，不会删除本地仓库文件。`)) {
    return;
  }

  try {
    const api = getTauriAPI();
    if (!api?.deleteGitRepo) {
      toast.error('删除功能在当前环境不可用');
      return;
    }
    await api.deleteGitRepo(repo.id);
    toast.success(`已删除仓库「${repo.name}」`);
    await loadRepos();
  } catch (error) {
    handleError(error, { context: 'deleteGitRepo' });
  }
};

const openRepo = async (repo: GitRepo) => {
  // 验证路径有效性
  try {
    const api = getTauriAPI();
    if (api?.validateGitRepoPath) {
      const result = await api.validateGitRepoPath(repo.path);
      if (!result.valid) {
        toast.error(`仓库路径无效: ${result.error || '路径不存在或不是 Git 仓库'}`);
        return;
      }
    }
  } catch {
    // 验证失败时继续打开，让后端处理
  }
  selectedRepo.value = repo;
};

const closeGitManager = () => {
  selectedRepo.value = null;
  loadRepos(); // 回到列表时刷新
};

// ===== Scan local repos =====
const doScan = async () => {
  const dirs = scanDirectories.value
    .split('\n')
    .map(d => d.trim())
    .filter(d => d.length > 0);

  if (dirs.length === 0) {
    toast.warning('请至少输入一个工作目录路径');
    return;
  }

  scanning.value = true;
  scannedRepos.value = [];
  scanResult.value = null;

  try {
    const api = getTauriAPI();
    const result = await api.scanLocalGitRepos(dirs);
    if (result && Array.isArray(result)) {
      scannedRepos.value = result;
      scanResult.value = result.length;
      if (result.length > 0) {
        toast.success(`发现 ${result.length} 个 Git 仓库`);
      }
    } else {
      scanResult.value = 0;
      toast.info('未找到 Git 仓库');
    }
  } catch (error) {
    handleError(error, { context: 'scanLocalRepos' });
    scanResult.value = 0;
  } finally {
    scanning.value = false;
  }
};

const isRepoAlreadyAdded = (path: string): boolean => {
  return repos.value.some(r => r.path === path);
};

const addScannedRepo = async (repo: RepoScanResult) => {
  if (isRepoAlreadyAdded(repo.path)) {return;}

  const api = getTauriAPI();
  await api.addGitRepo({
    id: crypto.randomUUID(),
    name: repo.name,
    path: repo.path,
    remote: undefined,
    branch: undefined,
  });
  toast.success(`已添加仓库「${repo.name}」`);
  await loadRepos();
};

onMounted(async () => {
  await loadRepos();
});
</script>
