<template>
  <div class="p-5 max-w-[1200px] mx-auto">
    <div class="flex justify-between items-center mb-5">
      <h2 class="m-0 text-2xl font-semibold text-base-content">Git 仓库</h2>
      <div class="flex gap-2">
        <UiButton @click="showScanSection = !showScanSection">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg> 扫描本地目录
        </UiButton>
        <UiButton @click="openAddModal">+ 添加仓库</UiButton>
      </div>
    </div>

    <!-- 扫描本地目录面板 -->
    <div v-if="showScanSection" class="mb-5 border border-base-content/20 rounded-xl bg-base-100 overflow-hidden">
      <div class="flex justify-between items-center px-4 py-3 bg-base-200 border-b border-base-content/10">
        <span class="text-sm font-semibold text-base-content"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg> 扫描本地目录</span>
        <button class="btn btn-ghost btn-xs" @click="showScanSection = false"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" class="inline-block"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg></button>
      </div>
      <div class="p-4">
        <p class="m-0 mb-3 text-xs text-base-content/60">输入工作目录路径（每行一个），点击搜索将自动发现该目录下的 Git 仓库</p>
        <textarea
          v-model="scanDirectories"
          class="textarea textarea-bordered w-full font-mono text-xs resize-y"
          placeholder="/home/fufengyuan/projects&#10;/home/fufengyuan/workspace&#10;/home/fufengyuan/code"
          rows="4"
        ></textarea>
        <div class="flex items-center gap-3 mt-3">
          <UiButton variant="primary" @click="doScan" :loading="scanning">
            <template v-if="scanning">扫描中...</template>
            <template v-else><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg> 扫描</template>
          </UiButton>
          <span v-if="scanResult !== null" class="text-xs text-base-content/70">
            {{ scanResult === 0 ? '未找到仓库' : `找到 ${scanResult} 个仓库` }}
          </span>
        </div>
        <!-- 扫描结果列表 -->
        <div v-if="scannedRepos.length > 0" class="mt-4 flex flex-col gap-2 max-h-[300px] overflow-y-auto">
          <div v-for="repo in scannedRepos" :key="repo.path" class="flex justify-between items-center px-3 py-2.5 rounded-lg bg-base-200 border border-base-content/10">
            <div class="flex flex-col gap-0.5 min-w-0">
              <span class="text-sm font-medium text-base-content">{{ repo.name }}</span>
              <span class="text-xs text-base-content/50 font-mono truncate">{{ repo.path }}</span>
            </div>
            <UiButton
              variant="success"
              size="sm"
              :disabled="isRepoAlreadyAdded(repo.path)"
              @click="addScannedRepo(repo)"
            >
              {{ isRepoAlreadyAdded(repo.path) ? '已添加' : '+ 添加' }}
            </UiButton>
          </div>
        </div>
      </div>
    </div>

    <!-- 搜索栏 -->
    <div class="flex gap-3 mb-5 flex-wrap items-center">
      <div class="relative flex-1 min-w-[200px]">
        <svg class="absolute left-3 top-1/2 -translate-y-1/2 text-base-content/60 pointer-events-none w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8" /><path d="m21 21-4.35-4.35" />
        </svg>
        <input v-model="searchQuery" type="text" class="input input-bordered w-full pl-9" placeholder="搜索仓库名称、路径或远程地址..." />
      </div>
    </div>

    <!-- 仓库列表 - 卡片形式 -->
    <div v-if="filteredRepos.length > 0" class="flex flex-col gap-3">
      <div
        v-for="repo in filteredRepos"
        :key="repo.id"
        class="flex items-center gap-4 px-5 py-4 bg-base-100 border border-base-content/10 rounded-xl transition-all duration-200 cursor-pointer hover:border-primary hover:shadow-lg hover:-translate-y-px"
        @dblclick="openRepo(repo)"
      >
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2 mb-1.5">
            <svg class="text-primary shrink-0" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22" />
            </svg>
            <span class="text-base font-semibold text-base-content truncate">{{ repo.name }}</span>
          </div>
          <div class="flex items-center gap-1.5 text-xs text-base-content/60 truncate" :title="repo.path">
            <svg class="shrink-0 opacity-60" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z" />
            </svg>
            <span>{{ repo.path }}</span>
          </div>
        </div>

        <div class="flex flex-col gap-1.5 min-w-[180px] max-w-[280px]">
          <div v-if="repo.remote" class="flex items-center gap-1.5 text-xs text-base-content/60 truncate" :title="repo.remote">
            <svg class="shrink-0 opacity-60" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" />
              <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
            </svg>
            <span class="truncate">{{ repo.remote }}</span>
          </div>
          <div v-if="repo.branch" class="flex items-center gap-1.5 text-xs text-base-content/60 truncate">
            <svg class="shrink-0 opacity-60" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="6" y1="3" x2="6" y2="15" />
              <circle cx="18" cy="6" r="3" />
              <circle cx="6" cy="18" r="3" />
              <path d="M18 9a9 9 0 0 1-9 9" />
            </svg>
            <span class="badge badge-sm">{{ repo.branch }}</span>
          </div>
          <div v-if="repo.lastOpened" class="flex items-center gap-1.5 text-xs text-base-content/60 truncate">
            <svg class="shrink-0 opacity-60" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10" />
              <polyline points="12 6 12 12 16 14" />
            </svg>
            <span class="truncate">{{ formatTime(repo.lastOpened) }}</span>
          </div>
        </div>

        <div class="flex gap-2 shrink-0">
          <UiButton variant="success" size="sm" @click="openRepo(repo)" title="打开仓库">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7" />
            </svg>
            打开
          </UiButton>
          <UiButton variant="ghost" size="sm" @click="openEditModal(repo)" title="编辑">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
              <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
            </svg>
          </UiButton>
          <UiButton variant="danger" size="sm" @click="deleteRepo(repo)" title="删除">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="3 6 5 6 21 6" />
              <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
            </svg>
          </UiButton>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <UiEmptyState v-else :text="searchQuery ? '没有找到匹配的仓库' : '暂无 Git 仓库'" :subtext="searchQuery ? '尝试其他搜索词' : '点击下方按钮添加第一个仓库'">
      <template #icon>
        <svg viewBox="0 0 24 24" width="64" height="64" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22" />
        </svg>
      </template>
      <template #action v-if="!searchQuery">
        <UiButton @click="openAddModal">+ 添加仓库</UiButton>
      </template>
    </UiEmptyState>

    <!-- 添加/编辑仓库模态框 -->
    <UiModal
      v-model="showModal"
      :title="editingRepo ? '✏️ 编辑仓库' : '✨ 添加 Git 仓库'"
      @close="resetModal"
      width="640px"
    >
      <div class="mb-4">
        <label for="repo-path" class="flex items-center gap-1.5 mb-2 text-xs font-medium text-base-content">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z" />
          </svg>
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
            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
            </svg>
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
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
            <circle cx="12" cy="7" r="4" />
          </svg>
          仓库名称
        </label>
        <input id="repo-name" v-model="formData.name" type="text" class="input input-bordered w-full" placeholder="仓库名称（留空将自动从路径提取）" />
      </div>

      <div class="flex gap-4">
        <div class="flex-1 mb-4">
          <label for="repo-remote" class="flex items-center gap-1.5 mb-2 text-xs font-medium text-base-content">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10" />
              <line x1="2" y1="12" x2="22" y2="12" />
              <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
            </svg>
            远程 URL
          </label>
          <input id="repo-remote" v-model="formData.remote" type="text" class="input input-bordered w-full" placeholder="https://github.com/user/repo.git" />
        </div>
        <div class="flex-1 mb-4">
          <label for="repo-branch" class="flex items-center gap-1.5 mb-2 text-xs font-medium text-base-content">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="6" y1="3" x2="6" y2="15" />
              <circle cx="18" cy="6" r="3" />
              <circle cx="6" cy="18" r="3" />
              <path d="M18 9a9 9 0 0 1-9 9" />
            </svg>
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
import { ref, computed, onMounted } from 'vue';
import UiButton from '@/components/ui/Button.vue';
import UiModal from '@/components/ui/Modal.vue';
import UiEmptyState from '@/components/ui/EmptyState.vue';
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

const emit = defineEmits<{
  'open-repo': [repo: GitRepo];
}>();

const repos = ref<GitRepo[]>([]);
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
  if (!searchQuery.value.trim()) return repos.value;
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
    console.log("[loadRepos] called")
    const api = getTauriAPI();
    if (!api?.getGitRepos) {
      toast.warning('Git 仓库管理功能在当前环境不可用');
      return;
    }
    const result = await api.getGitRepos();
    if (result.success && result.data) {
      repos.value = result.data;
    } else if (result.error) {
      toast.error(`加载仓库列表失败: ${result.error}`);
    }
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

  if (diffMins < 1) return '刚刚';
  if (diffMins < 60) return `${diffMins} 分钟前`;
  if (diffHours < 24) return `${diffHours} 小时前`;
  if (diffDays < 7) return `${diffDays} 天前`;
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
    if (!api?.validateGitRepoPath) return true;
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
  if (validateTimer) clearTimeout(validateTimer);
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
    console.log("[selectDirectory] called")
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
  if (validateTimer) clearTimeout(validateTimer);
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
      const result = await api.updateGitRepo(editingRepo.value.id, repoData);
      if (result.success) {
        toast.success('仓库信息已更新');
      } else {
        toast.error(`更新失败: ${result.error}`);
        return;
      }
    } else {
      const result = await api.addGitRepo({ id: crypto.randomUUID(), ...repoData });
      if (result.success) {
        toast.success('仓库已添加');
      } else {
        toast.error(`添加失败: ${result.error}`);
        return;
      }
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
    const result = await api.deleteGitRepo(repo.id);
    if (result.success) {
      toast.success(`已删除仓库「${repo.name}」`);
      await loadRepos();
    } else {
      toast.error(`删除失败: ${result.error}`);
    }
  } catch (error) {
    handleError(error, { context: 'deleteGitRepo' });
  }
};

const openRepo = (repo: GitRepo) => {
  emit('open-repo', repo);
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
  if (isRepoAlreadyAdded(repo.path)) return;

  const api = getTauriAPI();
  const result = await api.addGitRepo({
    id: crypto.randomUUID(),
    name: repo.name,
    path: repo.path,
    remote: undefined,
    branch: undefined,
  });

  if (result.success) {
    toast.success(`已添加仓库「${repo.name}」`);
    await loadRepos();
  } else {
    toast.error(`添加失败: ${result.error}`);
  }
};

onMounted(async () => {
    console.log("[components/GitRepoList.vue] mounted")
  await loadRepos();
});
</script>
