<template>
  <div class="mt-1">
    <div class="flex items-center gap-2 px-3 bg-base-100 border-[1.5px] border-base-content/20 rounded-xl transition-all duration-150 ease focus-within:border-primary focus-within:shadow-[0_0_0_3px_rgba(99,102,241,0.1)]">
      <svg class="text-base-content/60 shrink-0" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8" />
        <path d="m21 21-4.35-4.35" />
      </svg>
      <input
        v-model="searchQuery"
        type="text"
        placeholder="搜索仓库名称或路径..."
        class="flex-1 py-2.5 border-none bg-transparent text-base-content text-sm outline-none placeholder:text-base-content/60 placeholder:opacity-70"
        @focus="loadRepos"
      />
      <button
        type="button"
        @click="loadRepos"
        class="w-8 h-8 border-none rounded-lg bg-transparent text-base-content/60 cursor-pointer flex items-center justify-center transition-all duration-150 ease shrink-0 hover:bg-primary/10 hover:text-primary"
        :class="{ '[&_svg]:animate-spin': loading }"
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

    <!-- 扫描目录配置 -->
    <div class="mt-1.5 px-3">
      <button type="button" class="text-[11px] text-base-content/60 bg-none border-none p-0.5 cursor-pointer hover:text-primary" @click="showScanDirs = !showScanDirs">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg> {{ showScanDirs ? '收起' : '设置扫描目录' }}
      </button>
      <div v-if="showScanDirs" class="mt-1 flex gap-1.5 items-start">
        <textarea
          v-model="scanDirsText"
          class="flex-1 text-[11px] p-1.5 resize-y bg-base-200 border border-base-content/15 rounded-md text-base-content font-mono"
          placeholder="每行一个目录，留空则自动扫描常用目录&#10;~/projects&#10;~/IdeaProjects&#10;~/WebstormProjects"
          rows="3"
          @blur="saveScanDirs(scanDirsText)"
        ></textarea>
        <button type="button" class="text-[11px] px-2.5 py-1.5 rounded-md bg-primary text-primary-content border-none whitespace-nowrap" @click="scanWithCustomDirs"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg> 扫描</button>
      </div>
    </div>

    <!-- 手动输入路径 -->
    <div class="flex items-center gap-2 mt-2 px-3 bg-base-200 border-[1.5px] border-base-content/10 rounded-xl transition-all duration-150 ease focus-within:border-primary focus-within:shadow-[0_0_0_3px_rgba(99,102,241,0.1)]">
      <svg class="text-base-content/60 shrink-0" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
      </svg>
      <input
        v-model="manualPath"
        type="text"
        class="flex-1 py-2.5 border-none bg-transparent text-[13px] text-base-content outline-none placeholder:text-base-content/60 placeholder:opacity-50 font-['SF_Mono','Fira_Code',monospace]"
        placeholder="或手动输入本地 Git 仓库路径..."
        @input="onManualPathInput"
        @blur="validateManualPath"
      />
      <button
        type="button"
        v-if="manualPath"
        @click="clearManualPath"
        class="w-[22px] h-[22px] border-none rounded-full bg-transparent text-base-content/60 cursor-pointer flex items-center justify-center text-[11px] transition-all duration-150 ease shrink-0 hover:bg-error hover:text-white"
        title="清除"
      ><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" class="inline-block"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg></button>
    </div>
    <div v-if="manualPathValid" class="flex items-center gap-1 mt-1.5 px-2.5 py-1 text-xs text-[#22c55e]">
      <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" class="shrink-0">
        <polyline points="20 6 9 17 4 12" />
      </svg>
      <span>✓ 有效路径</span>
    </div>
    <div v-else-if="manualPathInvalid" class="flex items-center gap-1 mt-1.5 px-2.5 py-1 text-xs text-error">
      <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" class="shrink-0">
        <circle cx="12" cy="12" r="10" /><line x1="15" y1="9" x2="9" y2="15" /><line x1="9" y1="9" x2="15" y2="15" />
      </svg>
      <span>{{ manualPathInvalid }}</span>
    </div>

    <!-- 骨架屏 -->
    <div v-if="loading && !repos.length" class="mt-2">
      <div v-for="i in 3" :key="i" class="h-10 bg-gradient-to-r from-base-100 via-base-200 to-base-100 bg-[length:200%_100%] animate-[shimmer_1.5s_infinite] rounded-lg mb-1.5"></div>
    </div>

    <!-- 仓库列表 -->
    <div v-else-if="filteredRepos.length > 0" class="mt-2 max-h-[200px] overflow-y-auto rounded-xl bg-base-100 border border-base-content/10">
      <div
        v-for="repo in filteredRepos"
        :key="repo.path"
        class="flex items-center gap-3 px-3.5 py-3 cursor-pointer transition-[background] duration-150 ease border-b border-base-content/10 last:border-b-0 hover:bg-base-200"
        :class="{ 'bg-primary/10': modelValue === repo.path }"
        @click="selectRepo(repo)"
      >
        <div class="w-5 h-5 rounded-md border-2 border-base-content/10 flex items-center justify-center shrink-0 transition-all duration-150 ease"
          :class="{ 'bg-primary border-primary text-white': modelValue === repo.path }">
          <svg v-if="modelValue === repo.path" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.5">
            <polyline points="20 6 9 17 4 12" />
          </svg>
        </div>
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-1.5 font-semibold text-[13px] text-base-content">
            <svg class="text-base-content/60 shrink-0" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22" />
            </svg>
            {{ repo.name }}
          </div>
          <div class="text-xs text-base-content/60 mt-0.5 font-['SF_Mono','Fira_Code',monospace]">{{ repo.relativePath }}</div>
        </div>
      </div>
    </div>

    <!-- 无结果 -->
    <div v-else-if="searchQuery" class="flex flex-col items-center py-7 px-5 gap-2 text-base-content/60">
      <svg viewBox="0 0 24 24" width="32" height="32" fill="none" stroke="currentColor" stroke-width="1.5" class="opacity-30">
        <circle cx="11" cy="11" r="8" /><path d="m21 21-4.35-4.35" /><path d="M8 11h6" />
      </svg>
      <p class="text-[13px] m-0">未找到匹配的仓库</p>
    </div>

    <!-- 默认提示 -->
    <div v-else class="flex flex-col items-center py-7 px-5 gap-2 text-base-content/60">
      <svg viewBox="0 0 24 24" width="32" height="32" fill="none" stroke="currentColor" stroke-width="1.5" class="opacity-30">
        <path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z" />
      </svg>
      <p class="text-[13px] m-0">输入关键词搜索或手动输入路径</p>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, computed } from 'vue';
import { getTauriAPI } from '@/utils/tauri-api'
import { useErrorHandler } from '@/composables/useErrorHandler';

const { handleError } = useErrorHandler();

const modelValue = defineModel({ type: String, default: '' });
const emit = defineEmits(['select']);

const repos = ref<any[]>([]);
const loading = ref(false);
const searchQuery = ref('');

// 扫描目录配置
const showScanDirs = ref(false);
const scanDirsText = ref('');

// 加载保存的扫描目录
const loadScanDirs = async () => {
  try {
    const saved = await getTauriAPI().getSetting('git_scan_directories');
    if (saved) scanDirsText.value = saved;
  } catch {}
};
loadScanDirs();

// 保存扫描目录
const saveScanDirs = async (dirs: string) => {
  try {
    await getTauriAPI().setSetting('git_scan_directories', dirs);
  } catch {}
};

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
    console.log("[GitRepoSelector.vue] loadRepos() called")
    repos.value = (await getTauriAPI().scanLocalGitRepos()) || [];
  } catch (error) {
    handleError(error, { context: '扫描本地仓库', showToast: true });
  } finally {
    loading.value = false;
  }
};

// 使用自定义目录扫描
const scanWithCustomDirs = async () => {
  const dirs = scanDirsText.value.split('\n').map(s => s.trim()).filter(Boolean);
  saveScanDirs(scanDirsText.value);
  loading.value = true;
  try {
    repos.value = (await getTauriAPI().scanLocalGitRepos(dirs.length > 0 ? dirs : undefined)) || [];
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
    console.log("[GitRepoSelector.vue] validateManualPath() called")
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
