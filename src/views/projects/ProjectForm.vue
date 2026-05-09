<template>
  <form @submit.prevent="handleSave" class="p-0">
    <!-- 基本信息 -->
    <div class="mb-6">
      <div class="flex gap-4">
        <div class="flex-1">
          <UiInput v-model="formData.name" label="" type="text" :placeholder="$t('projectForm.name')" required />
        </div>
        <div class="w-[140px]">
          <ColorPicker v-model="formData.color" />
        </div>
      </div>
      <UiInput v-model="formData.description" type="textarea" label="" :placeholder="$t('projectForm.descriptionPlaceholder')" :rows="2" />

      <!-- 分类选择 -->
      <div class="mb-4">
        <UiInput v-model="formData.category" type="select" label="">
          <option value="">{{ $t('projectForm.uncategorized') }}</option>
          <option value="frontend"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="13.5" cy="6.5" r="0.5" fill="currentColor"/><circle cx="17.5" cy="10.5" r="0.5" fill="currentColor"/><circle cx="8.5" cy="7.5" r="0.5" fill="currentColor"/><circle cx="6.5" cy="12.5" r="0.5" fill="currentColor"/><path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.93 0 1.5-.67 1.5-1.5 0-.39-.15-.74-.39-1.01-.23-.26-.38-.61-.38-1 0-.83.67-1.5 1.5-1.5H16c3.31 0 6-2.69 6-6 0-4.5-4.22-8-10-8z"/></svg> {{ $t('projectForm.frontend') }}</option>
          <option value="backend"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg> {{ $t('projectForm.backend') }}</option>
          <option value="infrastructure"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg> {{ $t('projectForm.infrastructure') }}</option>
          <option value="other"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M12 2L15.09 8.26L22 9.27L17 14.14L18.18 21.02L12 17.77L5.82 21.02L7 14.14L2 9.27L8.91 8.26L12 2z"/></svg> {{ $t('projectForm.other') }}</option>
        </UiInput>
      </div>
    </div>

    <!-- Git 远程仓库地址 -->
    <div class="mb-6 p-5 bg-base-200 rounded-xl border border-base-content/10">
      <div class="flex items-center gap-2 mb-3.5 text-sm font-semibold text-base-content">
        <span class="text-lg"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg></span>
        <span>{{ $t('projectForm.gitUrls') }}</span>
      </div>
      <div class="mb-4">
        <UiInput v-model="formData.gitUrl1" label="" type="text" :placeholder="$t('projectForm.gitUrl1')" />
      </div>
      <div class="mb-4">
        <UiInput v-model="formData.gitUrl2" label="" type="text" :placeholder="$t('projectForm.gitUrl2')" />
      </div>
    </div>

    <!-- Git 仓库选择 -->
    <div class="mb-6 p-5 bg-base-200 rounded-xl border border-base-content/10">
      <div class="flex items-center gap-2 mb-3.5 text-sm font-semibold text-base-content">
        <span class="text-lg"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg></span>
        <span>{{ $t('projectForm.localGit') }} <span class="text-xs font-normal text-base-content/60">（仓库 1）</span></span>
      </div>
      <GitRepoSelector v-model="formData.repoPath" @select="onRepoSelect" />

      <!-- 已选仓库标签 -->
      <div v-if="formData.repoPath" class="flex items-center gap-2 px-3.5 py-2 mt-3 bg-primary/10 border border-primary rounded-xl text-sm">
        <svg class="text-primary flex-shrink-0" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22" />
        </svg>
        <span class="flex-1 font-medium text-primary overflow-hidden text-ellipsis whitespace-nowrap">{{ getRepoNameByPath(formData.repoPath) }}</span>
        <button type="button" @click="clearRepoSelection" class="btn btn-ghost btn-xs btn-square text-base-content/60 hover:bg-error hover:text-white" title="移除">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6 6 18" /><path d="m6 6 12 12" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Git 仓库选择 2 -->
    <div class="mb-6 p-5 bg-base-200 rounded-xl border border-primary/10">
      <div class="flex items-center gap-2 mb-3.5 text-sm font-semibold text-base-content">
        <span class="text-lg"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg></span>
        <span>{{ $t('projectForm.localGit') }} <span class="text-xs font-normal text-base-content/60">（仓库 2 / 后端）</span></span>
      </div>
      <div class="text-xs italic text-base-content/60 mb-3">可选 — 适用于前后端分离项目</div>
      <GitRepoSelector v-model="formData.repoPath2" @select="onRepoSelect2" />

      <div v-if="formData.repoPath2" class="flex items-center gap-2 px-3.5 py-2 mt-3 bg-primary/10 border border-primary rounded-xl text-sm">
        <svg class="text-primary flex-shrink-0" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22" />
        </svg>
        <span class="flex-1 font-medium text-primary overflow-hidden text-ellipsis whitespace-nowrap">{{ getRepoNameByPath(formData.repoPath2) }}</span>
        <button type="button" @click="clearRepoSelection2" class="btn btn-ghost btn-xs btn-square text-base-content/60 hover:bg-error hover:text-white" title="移除">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6 6 18" /><path d="m6 6 12 12" />
          </svg>
        </button>
      </div>
    </div>

    <!-- 分支选择 -->
    <div class="mb-4" v-if="formData.repoPath">
      <label class="flex items-center gap-1.5 mb-2 text-xs font-medium text-base-content">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" class="text-base-content/60">
          <line x1="6" y1="3" x2="6" y2="15" /><circle cx="18" cy="6" r="3" />
          <circle cx="6" cy="18" r="3" /><path d="M18 9a9 9 0 0 1-9 9" />
        </svg>
        选择分支（仓库 1）
      </label>
      <div class="flex items-center gap-2.5">
        <UiInput v-model="formData.branch" type="select">
          <option value="">默认分支 (HEAD)</option>
          <option v-for="branch in availableBranches" :key="branch" :value="branch">{{ branch }}</option>
        </UiInput>
        <div v-if="branchesLoading" class="flex items-center gap-1.5 text-xs text-base-content/60">
          <span class="loading loading-spinner loading-xs"></span><span>加载中...</span>
        </div>
      </div>
    </div>

    <!-- 分支选择 2 -->
    <div class="mb-4" v-if="formData.repoPath2">
      <label class="flex items-center gap-1.5 mb-2 text-xs font-medium text-base-content">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" class="text-base-content/60">
          <line x1="6" y1="3" x2="6" y2="15" /><circle cx="18" cy="6" r="3" />
          <circle cx="6" cy="18" r="3" /><path d="M18 9a9 9 0 0 1-9 9" />
        </svg>
        选择分支（仓库 2）
      </label>
      <div class="flex items-center gap-2.5">
        <UiInput v-model="formData.branch2" type="select">
          <option value="">默认分支 (HEAD)</option>
          <option v-for="branch in availableBranches2" :key="branch" :value="branch">{{ branch }}</option>
        </UiInput>
        <div v-if="branchesLoading2" class="flex items-center gap-1.5 text-xs text-base-content/60">
          <span class="loading loading-spinner loading-xs"></span><span>加载中...</span>
        </div>
      </div>
    </div>
  </form>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, reactive, watch } from 'vue';
import UiInput from '../../components/ui/Input.vue';
import ColorPicker from './ColorPicker.vue';
import GitRepoSelector from '../git/GitRepoSelector.vue';
import { useErrorHandler } from '../../composables/useErrorHandler';
import { getTauriAPI } from '../../utils/tauri-api';

const props = defineProps({ project: { type: Object, default: null } });
const emit = defineEmits(['save', 'cancel']);

const { handleError } = useErrorHandler();

const formData = reactive({
  name: '',
  description: '',
  color: '#6366f1',
  category: '',
  gitUrl1: '',
  gitUrl2: '',
  repoPath: '',
  branch: '',
  repoPath2: '',
  branch2: ''
});
const availableBranches = ref<string[]>([]);
const branchesLoading = ref(false);
const availableBranches2 = ref<string[]>([]);
const branchesLoading2 = ref(false);

// 选择仓库后加载分支
const onRepoSelect = async (repo) => {
  formData.branch = '';
  branchesLoading.value = true;
  try {
    console.log("[views/projects/ProjectForm.vue] onRepoSelect() called")
    availableBranches.value = ((await getTauriAPI().getGitBranches(repo.path)) || []).map((b: any) => typeof b === 'string' ? b : b.name);
  } catch (error) {
    handleError(error, { context: 'onRepoSelect', showToast: false });
  } finally {
    branchesLoading.value = false;
  }
};

const clearRepoSelection = () => {
  formData.repoPath = '';
  formData.branch = '';
  availableBranches.value = [];
};

const onRepoSelect2 = async (repo) => {
  formData.branch2 = '';
  branchesLoading2.value = true;
  try {
    console.log("[views/projects/ProjectForm.vue] onRepoSelect2() called")
    availableBranches2.value = ((await getTauriAPI().getGitBranches(repo.path)) || []).map((b: any) => typeof b === 'string' ? b : b.name);
  } catch (error) {
    handleError(error, { context: 'onRepoSelect2', showToast: false });
  } finally {
    branchesLoading2.value = false;
  }
};

const clearRepoSelection2 = () => {
  formData.repoPath2 = '';
  formData.branch2 = '';
  availableBranches2.value = [];
};

const getRepoNameByPath = (path) => path.split('/').pop() || path;

const loadBranchesForRepo = async (repoPath, currentBranch) => {
  branchesLoading.value = true;
  try {
    console.log("[views/projects/ProjectForm.vue] loadBranchesForRepo() called")
    availableBranches.value = ((await getTauriAPI().getGitBranches(repoPath)) || []).map((b: any) => typeof b === 'string' ? b : b.name);
    formData.branch = currentBranch || '';
  } catch (error) {
    handleError(error, { context: 'loadBranchesForRepo', showToast: false });
  } finally {
    branchesLoading.value = false;
  }
};

const loadBranchesForRepo2 = async (repoPath, currentBranch) => {
  branchesLoading2.value = true;
  try {
    console.log("[views/projects/ProjectForm.vue] loadBranchesForRepo2() called")
    availableBranches2.value = ((await getTauriAPI().getGitBranches(repoPath)) || []).map((b: any) => typeof b === 'string' ? b : b.name);
    formData.branch2 = currentBranch || '';
  } catch (error) {
    handleError(error, { context: 'loadBranchesForRepo2', showToast: false });
  } finally {
    branchesLoading2.value = false;
  }
};

const initForm = () => {
  if (props.project) {
    formData.name = props.project.name;
    formData.description = props.project.description;
    formData.color = props.project.color || '#6366f1';
    formData.category = props.project.category || '';
    formData.gitUrl1 = props.project.gitUrl1 || '';
    formData.gitUrl2 = props.project.gitUrl2 || '';
    formData.repoPath = props.project.repoPath || '';
    formData.branch = props.project.branch || '';
    formData.repoPath2 = props.project.repoPath2 || '';
    formData.branch2 = props.project.branch2 || '';
    if (props.project.repoPath) loadBranchesForRepo(props.project.repoPath, props.project.branch);
    if (props.project.repoPath2) loadBranchesForRepo2(props.project.repoPath2, props.project.branch2);
  } else {
    Object.assign(formData, {
      name: '',
      description: '',
      color: '#6366f1',
      category: '',
      gitUrl1: '',
      gitUrl2: '',
      repoPath: '',
      branch: '',
      repoPath2: '',
      branch2: ''
    });
  }
  availableBranches.value = [];
  availableBranches2.value = [];
};

const handleSave = () => emit('save', { ...formData });

watch(() => props.project, initForm, { immediate: true });
defineExpose({ reset: initForm, submit: handleSave });
</script>
