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
      <select v-model="formData.repoPath" @change="onRepoSelect" class="select select-bordered w-full text-sm">
        <option value="">— 从 Git 仓库管理中选择 —</option>
        <option v-for="repo in managedRepos" :key="repo.id" :value="repo.path">
          {{ repo.name }} — {{ repo.path }}
        </option>
      </select>

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
      <select v-model="formData.repoPath2" @change="onRepoSelect2" class="select select-bordered w-full text-sm">
        <option value="">— 从 Git 仓库管理中选择 —</option>
        <option v-for="repo in managedRepos" :key="repo.id" :value="repo.path">
          {{ repo.name }} — {{ repo.path }}
        </option>
      </select>

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

<script setup lang="ts">
import { ref, reactive, watch, onMounted } from 'vue';
import SvgIcon from '@/components/ui/SvgIcon.vue';
import UiInput from '../../components/ui/Input.vue';
import ColorPicker from './ColorPicker.vue';
import { useErrorHandler } from '../../composables/useErrorHandler';
import { useToast } from '../../composables/useToast';
import { getTauriAPI } from '../../utils/tauri-api';
import type { Project } from '../../types';

const props = defineProps<{ project: Project | null }>();
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
  branch2: '',
  gitRepoId: '',
  gitRepoId2: '',
});
const availableBranches = ref<string[]>([]);
const branchesLoading = ref(false);
const availableBranches2 = ref<string[]>([]);
const branchesLoading2 = ref(false);

// 从 Git 仓库管理功能加载已保存的仓库列表
const managedRepos = ref<any[]>([]);

const loadManagedRepos = async () => {
  try {
    const result = await getTauriAPI().getGitRepos();
    if (Array.isArray(result)) {
      managedRepos.value = result;
    }
  } catch {
    // Git 仓库管理功能可能不可用，静默失败
  }
};
onMounted(loadManagedRepos);

// 选择仓库后加载分支
const loadBranches = async (repoPath: string) => {
  if (!repoPath) {return [];}
  try {
    return ((await getTauriAPI().getGitBranches(repoPath)) || []).map((b: any) => typeof b === 'string' ? b : b.name);
  } catch {
    return [];
  }
};

// 根据 gitRepoId 获取仓库详情
const getRepoById = (id: string) => {
  return managedRepos.value.find(r => r.id === id);
};

// 选择仓库后自动设置路径和加载分支
const onRepoSelect = async () => {
  formData.branch = '';
  availableBranches.value = [];
  if (!formData.repoPath) {
    formData.gitRepoId = '';
    return;
  }
  // 从 managedRepos 中找到匹配的仓库，设置 gitRepoId
  const repo = managedRepos.value.find(r => r.path === formData.repoPath);
  formData.gitRepoId = repo?.id || '';
  branchesLoading.value = true;
  try {
    availableBranches.value = await loadBranches(formData.repoPath);
  } finally {
    branchesLoading.value = false;
  }
};

const onRepoSelect2 = async () => {
  formData.branch2 = '';
  availableBranches2.value = [];
  if (!formData.repoPath2) {
    formData.gitRepoId2 = '';
    return;
  }
  const repo = managedRepos.value.find(r => r.path === formData.repoPath2);
  formData.gitRepoId2 = repo?.id || '';
  branchesLoading2.value = true;
  try {
    availableBranches2.value = await loadBranches(formData.repoPath2);
  } finally {
    branchesLoading2.value = false;
  }
};

const clearRepoSelection = () => {
  formData.repoPath = '';
  formData.branch = '';
  availableBranches.value = [];
};

const clearRepoSelection2 = () => {
  formData.repoPath2 = '';
  formData.branch2 = '';
  availableBranches2.value = [];
};

const getRepoNameByPath = (path: string) => path.split('/').pop() || path;

const loadBranchesForRepo = async (repoPath: string, currentBranch: string) => {
  if (!repoPath) {return;}
  branchesLoading.value = true;
  try {
    availableBranches.value = await loadBranches(repoPath);
    formData.branch = currentBranch || '';
  } finally {
    branchesLoading.value = false;
  }
};

const loadBranchesForRepo2 = async (repoPath: string, currentBranch: string) => {
  if (!repoPath) {return;}
  branchesLoading2.value = true;
  try {
    availableBranches2.value = await loadBranches(repoPath);
    formData.branch2 = currentBranch || '';
  } finally {
    branchesLoading2.value = false;
  }
};

const initForm = () => {
  if (props.project) {
    formData.name = props.project.name;
    formData.description = props.project.description || '';
    formData.color = props.project.color || '#6366f1';
    formData.category = props.project.category || '';
    formData.gitUrl1 = props.project.gitUrl1 || '';
    formData.gitUrl2 = props.project.gitUrl2 || '';
    formData.repoPath = props.project.repoPath || '';
    formData.branch = props.project.branch || '';
    formData.repoPath2 = props.project.repoPath2 || '';
    formData.branch2 = props.project.branch2 || '';
    formData.gitRepoId = props.project.gitRepoId || '';
    formData.gitRepoId2 = props.project.gitRepoId2 || '';
    if (props.project.repoPath) {loadBranchesForRepo(props.project.repoPath, props.project.branch ?? '');}
    if (props.project.repoPath2) {loadBranchesForRepo2(props.project.repoPath2, props.project.branch2 ?? '');}
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
      branch2: '',
      gitRepoId: '',
      gitRepoId2: ''
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
