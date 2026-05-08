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
          <option value="frontend">🎨 {{ $t('projectForm.frontend') }}</option>
          <option value="backend">⚙️ {{ $t('projectForm.backend') }}</option>
          <option value="infrastructure">🏗️ {{ $t('projectForm.infrastructure') }}</option>
          <option value="other">📌 {{ $t('projectForm.other') }}</option>
        </UiInput>
      </div>
    </div>

    <!-- Git 远程仓库地址 -->
    <div class="mb-6 p-5 bg-base-200 rounded-xl border border-base-content/10">
      <div class="flex items-center gap-2 mb-3.5 text-sm font-semibold text-base-content">
        <span class="text-lg">🔗</span>
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
        <span class="text-lg">📂</span>
        <span>{{ $t('projectForm.localGit') }} <span class="text-xs font-normal text-base-content/60">（仓库 1）</span></span>
      </div>
      <GitRepoSelector v-model="formData.repoPath" @select="onRepoSelect" />

      <!-- 已选仓库标签 -->
      <div v-if="formData.repoPath" class="flex items-center gap-2 px-3.5 py-2 mt-3 bg-primary/10 border border-primary rounded-xl text-sm">
        <svg class="text-primary flex-shrink-0" viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
          <path d="M12 2C6.477 2 2 6.477 2 12c0 4.42 2.865 8.166 6.839 9.489.5.092.682-.217.682-.482 0-.237-.009-.866-.013-1.7-2.782.603-3.369-1.342-3.369-1.342-.454-1.155-1.11-1.462-1.11-1.462-.908-.62.069-.608.069-.608 1.003.07 1.531 1.03 1.531 1.03.892 1.529 2.341 1.087 2.91.831.092-.646.35-1.086.636-1.337-2.22-.253-4.555-1.11-4.555-4.943 0-1.091.39-1.984 1.029-2.683-.103-.253-.446-1.27.098-2.647 0 0 .84-.268 2.75 1.026A9.578 9.578 0 0 1 12 6.836c.85.004 1.705.115 2.504.337 1.909-1.294 2.747-1.026 2.747-1.026.546 1.377.203 2.394.1 2.647.64.699 1.028 1.592 1.028 2.683 0 3.842-2.339 4.687-4.566 4.935.359.309.678.919.678 1.852 0 1.336-.012 2.415-.012 2.743 0 .267.18.578.688.48C19.138 20.161 22 16.416 22 12c0-5.523-4.477-10-10-10z" />
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
        <span class="text-lg">📂</span>
        <span>{{ $t('projectForm.localGit') }} <span class="text-xs font-normal text-base-content/60">（仓库 2 / 后端）</span></span>
      </div>
      <div class="text-xs italic text-base-content/60 mb-3">可选 — 适用于前后端分离项目</div>
      <GitRepoSelector v-model="formData.repoPath2" @select="onRepoSelect2" />

      <div v-if="formData.repoPath2" class="flex items-center gap-2 px-3.5 py-2 mt-3 bg-primary/10 border border-primary rounded-xl text-sm">
        <svg class="text-primary flex-shrink-0" viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
          <path d="M12 2C6.477 2 2 6.477 2 12c0 4.42 2.865 8.166 6.839 9.489.5.092.682-.217.682-.482 0-.237-.009-.866-.013-1.7-2.782.603-3.369-1.342-3.369-1.342-.454-1.155-1.11-1.462-1.11-1.462-.908-.62.069-.608.069-.608 1.003.07 1.531 1.03 1.531 1.03.892 1.529 2.341 1.087 2.91.831.092-.646.35-1.086.636-1.337-2.22-.253-4.555-1.11-4.555-4.943 0-1.091.39-1.984 1.029-2.683-.103-.253-.446-1.27.098-2.647 0 0 .84-.268 2.75 1.026A9.578 9.578 0 0 1 12 6.836c.85.004 1.705.115 2.504.337 1.909-1.294 2.747-1.026 2.747-1.026.546 1.377.203 2.394.1 2.647.64.699 1.028 1.592 1.028 2.683 0 3.842-2.339 4.687-4.566 4.935.359.309.678.919.678 1.852 0 1.336-.012 2.415-.012 2.743 0 .267.18.578.688.48C19.138 20.161 22 16.416 22 12c0-5.523-4.477-10-10-10z" />
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
