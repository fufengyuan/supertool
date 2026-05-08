<template>
  <div class="git-repo-list-container">
    <div class="git-repo-header">
      <h2>Git 仓库</h2>
      <div class="header-actions">
        <UiButton @click="showScanSection = !showScanSection">
          🔍 扫描本地目录
        </UiButton>
        <UiButton @click="openAddModal">+ 添加仓库</UiButton>
      </div>
    </div>

    <!-- 扫描本地目录面板 -->
    <div v-if="showScanSection" class="scan-panel">
      <div class="scan-panel-header">
        <span class="scan-panel-title">📂 扫描本地目录</span>
        <button class="btn btn-ghost btn-xs" @click="showScanSection = false">✕</button>
      </div>
      <div class="scan-panel-body">
        <p class="scan-hint">输入工作目录路径（每行一个），点击搜索将自动发现该目录下的 Git 仓库</p>
        <textarea
          v-model="scanDirectories"
          class="scan-directories-input"
          placeholder="/home/fufengyuan/projects&#10;/home/fufengyuan/workspace&#10;/home/fufengyuan/code"
          rows="4"
        ></textarea>
        <div class="scan-actions">
          <UiButton variant="primary" @click="doScan" :loading="scanning">
            {{ scanning ? '扫描中...' : '🔍 扫描' }}
          </UiButton>
          <span v-if="scanResult !== null" class="scan-result-text">
            {{ scanResult === 0 ? '未找到仓库' : `找到 ${scanResult} 个仓库` }}
          </span>
        </div>
        <!-- 扫描结果列表 -->
        <div v-if="scannedRepos.length > 0" class="scanned-repos">
          <div v-for="repo in scannedRepos" :key="repo.path" class="scanned-repo-item">
            <div class="scanned-repo-info">
              <span class="scanned-repo-name">{{ repo.name }}</span>
              <span class="scanned-repo-path">{{ repo.path }}</span>
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
    <div class="filters-bar">
      <div class="search-wrapper">
        <svg class="search-icon" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8" /><path d="m21 21-4.35-4.35" />
        </svg>
        <input v-model="searchQuery" type="text" class="search-input" placeholder="搜索仓库名称、路径或远程地址..." />
      </div>
    </div>

    <!-- 仓库列表 - 卡片形式 -->
    <div v-if="filteredRepos.length > 0" class="repo-list">
      <div
        v-for="repo in filteredRepos"
        :key="repo.id"
        class="repo-card"
        @dblclick="openRepo(repo)"
      >
        <div class="repo-main">
          <div class="repo-name-row">
            <svg class="repo-icon" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22" />
            </svg>
            <span class="repo-name">{{ repo.name }}</span>
          </div>
          <div class="repo-path" :title="repo.path">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z" />
            </svg>
            <span>{{ repo.path }}</span>
          </div>
        </div>

        <div class="repo-meta">
          <div v-if="repo.remote" class="meta-item" :title="repo.remote">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" />
              <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
            </svg>
            <span class="meta-text">{{ repo.remote }}</span>
          </div>
          <div v-if="repo.branch" class="meta-item">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="6" y1="3" x2="6" y2="15" />
              <circle cx="18" cy="6" r="3" />
              <circle cx="6" cy="18" r="3" />
              <path d="M18 9a9 9 0 0 1-9 9" />
            </svg>
            <span class="branch-badge">{{ repo.branch }}</span>
          </div>
          <div v-if="repo.lastOpened" class="meta-item">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10" />
              <polyline points="12 6 12 12 16 14" />
            </svg>
            <span class="meta-text">{{ formatTime(repo.lastOpened) }}</span>
          </div>
        </div>

        <div class="repo-actions">
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
      <div class="form-field">
        <label for="repo-path">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z" />
          </svg>
          本地路径 <span class="required">*</span>
        </label>
        <div class="path-input-group">
          <input
            id="repo-path"
            v-model="formData.path"
            type="text"
            class="form-input"
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
        <div v-if="validationStatus" class="validation-msg" :class="validationStatus.type">
          {{ validationStatus.message }}
        </div>
        <small class="form-field-hint">输入或选择一个 Git 仓库的本地路径</small>
      </div>

      <div class="form-field">
        <label for="repo-name">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
            <circle cx="12" cy="7" r="4" />
          </svg>
          仓库名称
        </label>
        <input id="repo-name" v-model="formData.name" type="text" class="form-input" placeholder="仓库名称（留空将自动从路径提取）" />
      </div>

      <div class="form-row">
        <div class="form-field">
          <label for="repo-remote">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10" />
              <line x1="2" y1="12" x2="22" y2="12" />
              <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
            </svg>
            远程 URL
          </label>
          <input id="repo-remote" v-model="formData.remote" type="text" class="form-input" placeholder="https://github.com/user/repo.git" />
        </div>
        <div class="form-field">
          <label for="repo-branch">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="6" y1="3" x2="6" y2="15" />
              <circle cx="18" cy="6" r="3" />
              <circle cx="6" cy="18" r="3" />
              <path d="M18 9a9 9 0 0 1-9 9" />
            </svg>
            当前分支
          </label>
          <input id="repo-branch" v-model="formData.branch" type="text" class="form-input" placeholder="main" />
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
import { useToast } from '../composables/useToast';
import { useErrorHandler } from '../composables/useErrorHandler';
import { getTauriAPI } from '../utils/tauri-api';
import type { GitRepo } from '../types';

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

<style scoped>
.git-repo-list-container {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

/* 头部 */
.git-repo-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.git-repo-header h2 {
  margin: 0;
  color: var(--color-base-content);
  font-size: 24px;
  font-weight: 600;
}

.header-actions {
  display: flex;
  gap: 8px;
}

/* 扫描面板 */
.scan-panel {
  margin-bottom: 20px;
  border: 1.5px solid color-mix(in oklab, var(--color-base-content) 20%, transparent);
  border-radius: 12px;
  background: var(--color-base-100);
  overflow: hidden;
}

.scan-panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: var(--color-base-200);
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.scan-panel-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-base-content);
}

.scan-panel-body {
  padding: 16px;
}

.scan-hint {
  margin: 0 0 12px;
  font-size: 13px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.scan-directories-input {
  width: 100%;
  padding: 10px 12px;
  border: 1.5px solid color-mix(in oklab, var(--color-base-content) 20%, transparent);
  border-radius: 8px;
  background: var(--color-base-200);
  color: var(--color-base-content);
  font-size: 13px;
  font-family: monospace;
  resize: vertical;
  outline: none;
  transition: border-color 0.15s ease;
}

.scan-directories-input:focus {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px color-mix(in oklab, var(--color-primary) 10%, transparent);
}

.scan-directories-input::placeholder {
  color: color-mix(in oklab, var(--color-base-content) 40%, transparent);
}

.scan-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 12px;
}

.scan-result-text {
  font-size: 13px;
  color: color-mix(in oklab, var(--color-base-content) 70%, transparent);
}

/* 扫描结果列表 */
.scanned-repos {
  margin-top: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 300px;
  overflow-y: auto;
}

.scanned-repo-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 12px;
  border-radius: 8px;
  background: var(--color-base-200);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.scanned-repo-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.scanned-repo-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-base-content);
}

.scanned-repo-path {
  font-size: 12px;
  color: color-mix(in oklab, var(--color-base-content) 50%, transparent);
  font-family: monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 筛选栏 */
.filters-bar {
  display: flex;
  gap: 12px;
  margin-bottom: 20px;
  flex-wrap: wrap;
  align-items: center;
}

.search-wrapper {
  position: relative;
  flex: 1;
  min-width: 200px;
}

.search-icon {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: 10px 14px 10px 36px;
  border: 1.5px solid color-mix(in oklab, var(--color-base-content) 20%, transparent);
  border-radius: 10px;
  background: var(--color-base-200);
  color: var(--color-base-content);
  font-size: 14px;
  outline: none;
  transition: all 0.15s ease;
}

.search-input:focus {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px color-mix(in oklab, var(--color-primary) 10%, transparent);
}

.search-input::placeholder {
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  opacity: 0.7;
}

/* 仓库卡片列表 */
.repo-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.repo-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px 20px;
  background: var(--color-base-100);
  border: 1.5px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 12px;
  transition: all 0.2s ease;
  cursor: pointer;
}

.repo-card:hover {
  border-color: var(--color-primary);
  box-shadow: 0 4px 12px rgba(0,0,0,0.15);
  transform: translateY(-1px);
}

.repo-main {
  flex: 1;
  min-width: 0;
}

.repo-name-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.repo-icon {
  color: var(--color-primary);
  flex-shrink: 0;
}

.repo-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-base-content);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.repo-path {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.repo-path svg {
  flex-shrink: 0;
  opacity: 0.6;
}

/* 元信息 */
.repo-meta {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 180px;
  max-width: 280px;
}

.meta-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  white-space: nowrap;
  overflow: hidden;
}

.meta-item svg {
  flex-shrink: 0;
  opacity: 0.6;
}

.meta-text {
  overflow: hidden;
  text-overflow: ellipsis;
}

.branch-badge {
  padding: 2px 8px;
  border-radius: 10px;
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
  color: var(--color-primary);
  font-weight: 500;
  font-size: 11px;
}

/* 操作按钮 */
.repo-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

/* 表单 */
.form-field {
  margin-bottom: 16px;
}

.form-field label {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 8px;
  color: var(--color-base-content);
  font-size: 13px;
  font-weight: 500;
}

.form-field .required {
  color: var(--color-error);
}

.form-field-hint {
  display: block;
  margin-top: 4px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-size: 12px;
}

.path-input-group {
  display: flex;
  gap: 8px;
}

.path-input-group .form-input {
  flex: 1;
  padding: 10px 14px;
  border: 1.5px solid color-mix(in oklab, var(--color-base-content) 20%, transparent);
  border-radius: 10px;
  background: var(--color-base-200);
  color: var(--color-base-content);
  font-size: 14px;
  font-family: inherit;
  transition: all 0.15s ease;
  outline: none;
}

.path-input-group .form-input:focus {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px color-mix(in oklab, var(--color-primary) 10%, transparent);
}

.path-input-group .form-input::placeholder {
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  opacity: 0.7;
}

.form-row {
  display: flex;
  gap: 16px;
}

.form-row > * {
  flex: 1;
}

.form-row .form-input {
  width: 100%;
  padding: 10px 14px;
  border: 1.5px solid color-mix(in oklab, var(--color-base-content) 20%, transparent);
  border-radius: 10px;
  background: var(--color-base-200);
  color: var(--color-base-content);
  font-size: 14px;
  font-family: inherit;
  transition: all 0.15s ease;
  outline: none;
}

.form-row .form-input:focus {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px color-mix(in oklab, var(--color-primary) 10%, transparent);
}

.form-row .form-input::placeholder {
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  opacity: 0.7;
}

/* 验证消息 */
.validation-msg {
  margin-top: 6px;
  font-size: 12px;
  font-weight: 500;
  padding: 4px 8px;
  border-radius: 6px;
}

.validation-msg.success {
  color: var(--color-success);
  background: rgba(64, 160, 43, 0.1);
}

:root.dark .validation-msg.success {
  background: rgba(166, 227, 161, 0.1);
}

.validation-msg.error {
  color: var(--color-error);
  background: rgba(210, 15, 57, 0.1);
}

:root.dark .validation-msg.error {
  background: rgba(243, 139, 168, 0.1);
}

.validation-msg.validating {
  color: var(--color-warning);
}
</style>
