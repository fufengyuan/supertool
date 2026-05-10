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
          <option value="frontend"><SvgIcon name="palette" :size="14" class="inline-block align-text-bottom" /> {{ $t('projectForm.frontend') }}</option>
          <option value="backend"><SvgIcon name="settings" :size="14" class="inline-block align-text-bottom" /> {{ $t('projectForm.backend') }}</option>
          <option value="infrastructure"><SvgIcon name="build" :size="14" class="inline-block align-text-bottom" /> {{ $t('projectForm.infrastructure') }}</option>
          <option value="other"><SvgIcon name="star" :size="14" class="inline-block align-text-bottom" /> {{ $t('projectForm.other') }}</option>
        </UiInput>
      </div>
    </div>

    <!-- Git 远程仓库地址 -->
    <div class="mb-6 p-5 bg-base-200 rounded-xl border border-base-content/10">
      <div class="flex items-center gap-2 mb-3.5 text-sm font-semibold text-base-content">
        <span class="text-lg"><SvgIcon name="link" :size="14" class="inline-block align-text-bottom" /></span>
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
        <span class="text-lg"><SvgIcon name="folder" :size="14" class="inline-block align-text-bottom" /></span>
        <span>{{ $t('projectForm.localGit') }} <span class="text-xs font-normal text-base-content/60">（仓库 1）</span></span>
      </div>
      <GitRepoSelector v-model="formData.repoPath" @select="onRepoSelect" />

      <!-- 已选仓库标签 -->
      <div v-if="formData.repoPath" class="flex items-center gap-2 px-3.5 py-2 mt-3 bg-primary/10 border border-primary rounded-xl text-sm">
        <SvgIcon name="github" size="14" class="text-primary flex-shrink-0" />
        <span class="flex-1 font-medium text-primary overflow-hidden text-ellipsis whitespace-nowrap">{{ getRepoNameByPath(formData.repoPath) }}</span>
        <button type="button" @click="clearRepoSelection" class="btn btn-ghost btn-xs btn-square text-base-content/60 hover:bg-error hover:text-white" title="移除">
          <SvgIcon name="x" :size="14" />
        </button>
      </div>
    </div>

    <!-- Git 仓库选择 2 -->
    <div class="mb-6 p-5 bg-base-200 rounded-xl border border-primary/10">
      <div class="flex items-center gap-2 mb-3.5 text-sm font-semibold text-base-content">
        <span class="text-lg"><SvgIcon name="folder" :size="14" class="inline-block align-text-bottom" /></span>
        <span>{{ $t('projectForm.localGit') }} <span class="text-xs font-normal text-base-content/60">（仓库 2 / 后端）</span></span>
      </div>
      <div class="text-xs italic text-base-content/60 mb-3">可选 — 适用于前后端分离项目</div>
      <GitRepoSelector v-model="formData.repoPath2" @select="onRepoSelect2" />

      <div v-if="formData.repoPath2" class="flex items-center gap-2 px-3.5 py-2 mt-3 bg-primary/10 border border-primary rounded-xl text-sm">
        <SvgIcon name="github" size="14" class="text-primary flex-shrink-0" />
        <span class="flex-1 font-medium text-primary overflow-hidden text-ellipsis whitespace-nowrap">{{ getRepoNameByPath(formData.repoPath2) }}</span>
        <button type="button" @click="clearRepoSelection2" class="btn btn-ghost btn-xs btn-square text-base-content/60 hover:bg-error hover:text-white" title="移除">
          <SvgIcon name="x" :size="14" />
        </button>
      </div>
    </div>

    <!-- 分支选择 -->
    <div class="mb-4" v-if="formData.repoPath">
      <label class="flex items-center gap-1.5 mb-2 text-xs font-medium text-base-content">
        <SvgIcon name="gitBranch" :size="14" class="text-base-content/60" />
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
        <SvgIcon name="gitBranch" :size="14" class="text-base-content/60" />
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
import SvgIcon from '@/components/ui/SvgIcon.vue';
import UiInput from '../../components/ui/Input.vue';
import ColorPicker from './ColorPicker.vue';
import GitRepoSelector from '../git/GitRepoSelector.vue';
import { useErrorHandler } from '../../composables/useErrorHandler';
import { useToast } from '../../composables/useToast';
import { getTauriAPI } from '../../utils/tauri-api';

const props = defineProps({ project: { type: Object, default: null } });
const emit = defineEmits(['save', 'cancel']);

const { handleError } = useErrorHandler();
const toast = useToast();

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

const handleSave = () => {
  if (!formData.name.trim()) {
    toast.error('项目名称不能为空');
    return;
  }
  emit('save', { ...formData });
};

watch(() => props.project, initForm, { immediate: true });
defineExpose({ reset: initForm, submit: handleSave });
</script>
