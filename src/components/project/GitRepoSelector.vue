<template>
  <div class="git-repo-selector">
    <div class="repo-search-bar">
      <svg class="search-icon" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8" />
        <path d="m21 21-4.35-4.35" />
      </svg>
      <input
        v-model="searchQuery"
        type="text"
        placeholder="搜索仓库名称或路径..."
        class="repo-search-input"
        @focus="loadRepos"
      />
      <button
        type="button"
        @click="loadRepos"
        class="refresh-btn"
        :class="{ spinning: loading }"
        title="刷新仓库列表"
      >
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 2v6h-6" />
          <path d="M3 12a9 9 0 0 1 15-6.7L21 8" />
          <path d="M3 22v-6h6" />
          <path d="M21 12a9 9 0 0 1-15 6.7L3 16" />
        </svg>
      </button>
    </div>

    <!-- 手动输入路径 -->
    <div class="manual-path-row">
      <svg class="path-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
      </svg>
      <input
        v-model="manualPath"
        type="text"
        class="manual-path-input"
        placeholder="或手动输入本地 Git 仓库路径..."
        @input="onManualPathInput"
        @blur="validateManualPath"
      />
      <button
        type="button"
        v-if="manualPath"
        @click="clearManualPath"
        class="clear-path-btn"
        title="清除"
      >✕</button>
    </div>
    <div v-if="manualPathValid" class="manual-path-valid">
      <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="20 6 9 17 4 12" />
      </svg>
      <span>✓ 有效路径</span>
    </div>
    <div v-else-if="manualPathInvalid" class="manual-path-invalid">
      <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10" /><line x1="15" y1="9" x2="9" y2="15" /><line x1="9" y1="9" x2="15" y2="15" />
      </svg>
      <span>{{ manualPathInvalid }}</span>
    </div>

    <!-- 骨架屏 -->
    <div v-if="loading && !repos.length" class="repo-skeleton">
      <div class="skeleton-line" v-for="i in 3" :key="i"></div>
    </div>

    <!-- 仓库列表 -->
    <div v-else-if="filteredRepos.length > 0" class="repo-list">
      <div
        v-for="repo in filteredRepos"
        :key="repo.path"
        class="repo-item"
        :class="{ selected: modelValue === repo.path }"
        @click="selectRepo(repo)"
      >
        <div class="repo-check">
          <svg v-if="modelValue === repo.path" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.5">
            <polyline points="20 6 9 17 4 12" />
          </svg>
        </div>
        <div class="repo-info">
          <div class="repo-name">
            <svg class="git-icon-small" viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
              <path d="M12 2C6.477 2 2 6.477 2 12c0 4.42 2.865 8.166 6.839 9.489.5.092.682-.217.682-.482 0-.237-.009-.866-.013-1.7-2.782.603-3.369-1.342-3.369-1.342-.454-1.155-1.11-1.462-1.11-1.462-.908-.62.069-.608.069-.608 1.003.07 1.531 1.03 1.531 1.03.892 1.529 2.341 1.087 2.91.831.092-.646.35-1.086.636-1.337-2.22-.253-4.555-1.11-4.555-4.943 0-1.091.39-1.984 1.029-2.683-.103-.253-.446-1.27.098-2.647 0 0 .84-.268 2.75 1.026A9.578 9.578 0 0 1 12 6.836c.85.004 1.705.115 2.504.337 1.909-1.294 2.747-1.026 2.747-1.026.546 1.377.203 2.394.1 2.647.64.699 1.028 1.592 1.028 2.683 0 3.842-2.339 4.687-4.566 4.935.359.309.678.919.678 1.852 0 1.336-.012 2.415-.012 2.743 0 .267.18.578.688.48C19.138 20.161 22 16.416 22 12c0-5.523-4.477-10-10-10z" />
            </svg>
            {{ repo.name }}
          </div>
          <div class="repo-path">{{ repo.relativePath }}</div>
        </div>
      </div>
    </div>

    <!-- 无结果 -->
    <div v-else-if="searchQuery" class="repo-empty">
      <svg viewBox="0 0 24 24" width="32" height="32" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="11" cy="11" r="8" /><path d="m21 21-4.35-4.35" /><path d="M8 11h6" />
      </svg>
      <p>未找到匹配的仓库</p>
    </div>

    <!-- 默认提示 -->
    <div v-else class="repo-empty">
      <svg viewBox="0 0 24 24" width="32" height="32" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z" />
      </svg>
      <p>输入关键词搜索或手动输入路径</p>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, computed } from 'vue';
import { useErrorHandler } from '../../composables/useErrorHandler';
import { getTauriAPI } from '../../utils/tauri-api';

const { handleError } = useErrorHandler();

const modelValue = defineModel({ type: String, default: '' });
const emit = defineEmits(['select']);

const repos = ref<any[]>([]);
const loading = ref(false);
const searchQuery = ref('');

// 手动输入路径
const manualPath = ref('');
const manualPathValid = ref(false);
const manualPathInvalid = ref('');

const filteredRepos = computed(() => {
  if (!searchQuery.value) return repos.value;
  const q = searchQuery.value.toLowerCase();
  return repos.value.filter(
    (r) => r.name.toLowerCase().includes(q) || r.relativePath.toLowerCase().includes(q) || r.url.toLowerCase().includes(q)
  );
});

const loadRepos = async () => {
  if (repos.value.length > 0 || loading.value) return;
  loading.value = true;
  try {
    console.log("[components/project/GitRepoSelector.vue] loadRepos() called")
    repos.value = (await getTauriAPI().scanLocalGitRepos()) || [];
  } catch (error) {
    handleError(error, { context: '扫描本地仓库', showToast: true });
  } finally {
    loading.value = false;
  }
};

const selectRepo = (repo) => {
  manualPath.value = '';
  manualPathValid.value = false;
  manualPathInvalid.value = '';
  modelValue.value = repo.path;
  emit('select', repo);
};

// 手动输入路径处理
const onManualPathInput = () => {
  manualPathValid.value = false;
  manualPathInvalid.value = '';
  if (manualPath.value.trim()) {
    modelValue.value = manualPath.value.trim();
  }
};

const validateManualPath = async () => {
  const p = manualPath.value.trim();
  if (!p) {
    console.log("[components/project/GitRepoSelector.vue] validateManualPath() called")
    manualPathValid.value = false;
    manualPathInvalid.value = '';
    return;
  }
  try {
    const result = await getTauriAPI().validateGitRepoPath(p);
    if (result.valid) {
      manualPathValid.value = true;
      manualPathInvalid.value = '';
      modelValue.value = p;
    } else {
      manualPathValid.value = false;
      manualPathInvalid.value = result.error || '无效的 Git 仓库路径';
    }
  } catch {
    manualPathValid.value = false;
    manualPathInvalid.value = '无法验证路径';
  }
};

const clearManualPath = () => {
  manualPath.value = '';
  manualPathValid.value = false;
  manualPathInvalid.value = '';
  if (modelValue.value && !repos.value.find((r) => r.path === modelValue.value)) {
    modelValue.value = '';
  }
};
</script>

<style scoped>
.git-repo-selector { margin-top: 4px; }

.repo-search-bar {
  display: flex; align-items: center; gap: 8px; padding: 0 12px;
  background: oklch(var(--b1)); border: 1.5px solid oklch(var(--bc) / 0.2);
  border-radius: 10px; transition: all 0.15s ease;
}
.repo-search-bar:focus-within {
  border-color: oklch(var(--p)); box-shadow: 0 0 0 3px oklch(var(--p) / 0.1);
}
.search-icon { color: oklch(var(--bc) / 0.6); flex-shrink: 0; }
.repo-search-input {
  flex: 1; padding: 10px 0; border: none; background: transparent;
  color: oklch(var(--bc)); font-size: 14px; outline: none;
}
.repo-search-input::placeholder { color: oklch(var(--bc) / 0.6); opacity: 0.7; }
.refresh-btn {
  width: 32px; height: 32px; border: none; border-radius: 8px;
  background: transparent; color: oklch(var(--bc) / 0.6); cursor: pointer;
  display: flex; align-items: center; justify-content: center;
  transition: all 0.15s ease; flex-shrink: 0;
}
.refresh-btn:hover { background: oklch(var(--p) / 0.1); color: oklch(var(--p)); }
.refresh-btn.spinning svg { animation: spin 1s linear infinite; }
@keyframes spin { from { transform: rotate(0); } to { transform: rotate(360deg); } }

/* 手动输入路径 */
.manual-path-row {
  display: flex; align-items: center; gap: 8px; margin-top: 8px; padding: 0 12px;
  background: oklch(var(--b2)); border: 1.5px solid oklch(var(--bc) / 0.1);
  border-radius: 10px; transition: all 0.15s ease;
}
.manual-path-row:focus-within {
  border-color: oklch(var(--p)); box-shadow: 0 0 0 3px oklch(var(--p) / 0.1);
}
.path-icon { color: oklch(var(--bc) / 0.6); flex-shrink: 0; }
.manual-path-input {
  flex: 1; padding: 10px 0; border: none; background: transparent;
  color: oklch(var(--bc)); font-size: 13px; outline: none;
  font-family: 'SF Mono', 'Fira Code', monospace;
}
.manual-path-input::placeholder { color: oklch(var(--bc) / 0.6); opacity: 0.5; }
.clear-path-btn {
  width: 22px; height: 22px; border: none; border-radius: 50%;
  background: transparent; color: oklch(var(--bc) / 0.6); cursor: pointer;
  display: flex; align-items: center; justify-content: center; font-size: 11px;
  transition: all 0.15s ease; flex-shrink: 0;
}
.clear-path-btn:hover { background: oklch(var(--er)); color: white; }
.manual-path-valid {
  display: flex; align-items: center; gap: 4px; margin-top: 6px; padding: 4px 10px;
  font-size: 12px; color: #22c55e;
}
.manual-path-valid svg { flex-shrink: 0; }
.manual-path-invalid {
  display: flex; align-items: center; gap: 4px; margin-top: 6px; padding: 4px 10px;
  font-size: 12px; color: oklch(var(--er));
}
.manual-path-invalid svg { flex-shrink: 0; }

.repo-skeleton { margin-top: 8px; }
.skeleton-line {
  height: 40px;
  background: linear-gradient(90deg, oklch(var(--b1)) 25%, oklch(var(--b2)) 50%, oklch(var(--b1)) 75%);
  background-size: 200% 100%; animation: shimmer 1.5s infinite;
  border-radius: 8px; margin-bottom: 6px;
}
@keyframes shimmer { 0% { background-position: 200% 0; } 100% { background-position: -200% 0; } }

.repo-list {
  margin-top: 8px; max-height: 200px; overflow-y: auto; border-radius: 10px;
  background: oklch(var(--b1)); border: 1px solid oklch(var(--bc) / 0.1);
}
.repo-item {
  display: flex; align-items: center; gap: 12px; padding: 12px 14px;
  cursor: pointer; transition: background 0.15s ease; border-bottom: 1px solid oklch(var(--bc) / 0.1);
}
.repo-item:last-child { border-bottom: none; }
.repo-item:hover { background: oklch(var(--b2)); }
.repo-item.selected { background: oklch(var(--p) / 0.1); }
.repo-check {
  width: 20px; height: 20px; border-radius: 6px; border: 2px solid oklch(var(--bc) / 0.1);
  display: flex; align-items: center; justify-content: center; flex-shrink: 0;
  transition: all 0.15s ease;
}
.repo-item.selected .repo-check { background: oklch(var(--p)); border-color: oklch(var(--p)); color: white; }
.repo-info { flex: 1; min-width: 0; }
.repo-name { display: flex; align-items: center; gap: 6px; font-weight: 600; font-size: 13px; color: oklch(var(--bc)); }
.git-icon-small { color: oklch(var(--bc) / 0.6); flex-shrink: 0; }
.repo-path {
  font-size: 12px; color: oklch(var(--bc) / 0.6); margin-top: 2px;
  font-family: 'SF Mono', 'Fira Code', monospace;
}
.repo-empty {
  display: flex; flex-direction: column; align-items: center; padding: 28px 20px;
  gap: 8px; color: oklch(var(--bc) / 0.6);
}
.repo-empty svg { opacity: 0.3; }
.repo-empty p { font-size: 13px; margin: 0; }
</style>
