<template>
  <div class="h-full flex flex-col">
    <!-- OMP mode overlay -->
    <div v-show="isOmpMode" class="flex-1 flex items-center justify-center">
      <div class="text-center max-w-md px-6">
        <SvgIcon name="terminal" :size="40" class="mx-auto text-base-content/20 mb-4" />
        <p class="text-sm font-medium text-base-content/50">OMP 配置文件</p>
        <p class="text-xs text-base-content/30 mt-2 leading-relaxed">
          OMP 不使用 Hermes 的 profile 系统。OMP 配置通过 config.yaml 管理。
        </p>
      </div>
    </div>

    <div v-show="!isOmpMode">
      <h1 class="text-sm font-medium">Agent Profiles</h1>
      <div class="flex items-center gap-2">
        <button class="btn btn-sm btn-ghost" @click="refreshProfiles">
          <SvgIcon name="refresh" size="14" />
        </button>
        <button class="btn btn-sm btn-primary" @click="showCreateProfile = true">
          新建 Profile
        </button>
      </div>
    </div>

    <!-- Dispatcher status -->
    <div class="px-4 py-2 bg-base-200/50 border-b border-base-content/10">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <span class="text-xs text-base-content/60">Dispatcher:</span>
          <span v-if="dispatcherRunning" class="text-xs text-success">● 运行中</span>
          <span v-else class="text-xs text-base-content/50">○ 未运行</span>
        </div>
        <div class="flex items-center gap-2">
          <button 
            class="btn btn-xs btn-ghost"
            @click="manualDispatch"
            :disabled="dispatching"
          >
            {{ dispatching ? '调度中...' : '手动调度' }}
          </button>
          <button 
            class="btn btn-xs btn-ghost"
            @click="dryRunDispatch"
          >
            模拟调度
          </button>
        </div>
      </div>
    </div>

    <!-- Profiles grid -->
    <div class="flex-1 overflow-y-auto p-4">
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
        <div 
          v-for="profile in profiles" 
          :key="profile.name"
          class="bg-base-100 rounded-lg border border-base-content/10 p-3 hover:border-primary/30 transition-colors"
        >
          <!-- Profile header -->
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center gap-2">
              <span v-if="profile.isDefault" class="text-primary">◆</span>
              <span class="font-medium">{{ profile.name }}</span>
            </div>
            <div class="flex items-center gap-1">
              <button 
                v-if="!profile.isDefault"
                class="btn btn-xs btn-ghost"
                @click="setDefault(profile.name)"
                title="设为默认"
              >
                设为默认
              </button>
              <button 
                class="btn btn-xs btn-ghost text-error"
                @click="deleteProfile(profile.name)"
                title="删除"
              >
                删除
              </button>
            </div>
          </div>

          <!-- Profile info -->
          <div class="space-y-1 text-xs">
            <div class="flex items-center justify-between">
              <span class="text-base-content/50">模型</span>
              <span class="flex items-center gap-1">
                {{ profile.model || '未设置' }}
                <button 
                  class="btn btn-xs btn-ghost btn-circle"
                  @click="openSetModel(profile.name, profile.model)"
                >
                  <SvgIcon name="pencil" size="12" />
                </button>
              </span>
            </div>
            <div class="flex items-center justify-between">
              <span class="text-base-content/50">Gateway</span>
              <span :class="gatewayClass(profile.gatewayStatus)">
                {{ profile.gatewayStatus || '未运行' }}
              </span>
            </div>
            <div class="flex items-center justify-between">
              <span class="text-base-content/50">任务数</span>
              <span>{{ getWorkload(profile.name) }}</span>
            </div>
            <div v-if="profile.description" class="mt-2">
              <span class="text-base-content/50">描述:</span>
              <span class="ml-1">{{ profile.description }}</span>
            </div>
          </div>

          <!-- Actions -->
          <div class="mt-3 flex items-center gap-1">
            <button 
              class="btn btn-xs btn-ghost"
              @click="showProfileDetail(profile.name)"
            >
              详情
            </button>
            <button 
              class="btn btn-xs btn-ghost"
              @click="editDescription(profile.name)"
            >
              编辑描述
            </button>
            <button 
              class="btn btn-xs btn-ghost"
              @click="updateProfile(profile.name)"
              v-if="profile.distribution"
            >
              更新
            </button>
          </div>
        </div>

        <!-- Empty state -->
        <div v-if="profiles.length === 0" class="col-span-full text-center py-8 text-base-content/40">
          暂无 Profile，点击"新建 Profile"创建
        </div>
      </div>
    </div>

    <!-- Create profile modal -->
    <div v-if="showCreateProfile" class="fixed inset-0 bg-base-content/20 z-50 flex items-center justify-center">
      <div class="bg-base-100 rounded-lg shadow-xl w-[480px]">
        <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
          <span class="text-sm font-medium">新建 Profile</span>
          <button class="btn btn-sm btn-ghost btn-circle" @click="showCreateProfile = false">
            <SvgIcon name="close" size="14" />
          </button>
        </div>
        <div class="p-4 space-y-3">
          <div>
            <label class="text-xs text-base-content/50 block mb-1">名称 *</label>
            <input 
              v-model="newProfile.name"
              type="text"
              class="input input-sm input-bordered w-full"
              placeholder="如: docker-worker, researcher"
            />
          </div>
          <div>
            <label class="text-xs text-base-content/50 block mb-1">描述（用于任务路由）</label>
            <textarea 
              v-model="newProfile.description"
              class="textarea textarea-sm textarea-bordered w-full"
              placeholder="描述该 profile 的能力，如: 专注于 Docker 容器运维..."
              rows="3"
            ></textarea>
          </div>
        </div>
        <div class="px-4 py-3 border-t border-base-content/10 flex items-center justify-end gap-2">
          <button class="btn btn-sm btn-ghost" @click="showCreateProfile = false">取消</button>
          <button 
            class="btn btn-sm btn-primary"
            :disabled="!newProfile.name.trim()"
            @click="createProfile"
          >
            创建
          </button>
        </div>
      </div>
    </div>

    <!-- Edit description modal -->
    <div v-if="editingProfile" class="fixed inset-0 bg-base-content/20 z-50 flex items-center justify-center">
      <div class="bg-base-100 rounded-lg shadow-xl w-[480px]">
        <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
          <span class="text-sm font-medium">编辑 Profile 描述</span>
          <button class="btn btn-sm btn-ghost btn-circle" @click="editingProfile = null">
            <SvgIcon name="close" size="14" />
          </button>
        </div>
        <div class="p-4">
          <div class="text-xs text-base-content/50 mb-2">{{ editingProfile }}</div>
          <textarea 
            v-model="editDescriptionText"
            class="textarea textarea-sm textarea-bordered w-full"
            placeholder="描述该 profile 的能力..."
            rows="4"
          ></textarea>
          <div class="text-xs text-base-content/40 mt-2">
            描述用于 Kanban Orchestrator 任务路由
          </div>
        </div>
        <div class="px-4 py-3 border-t border-base-content/10 flex items-center justify-end gap-2">
          <button class="btn btn-sm btn-ghost" @click="editingProfile = null">取消</button>
          <button 
            class="btn btn-sm btn-primary"
            @click="saveDescription"
          >
            保存
          </button>
        </div>
      </div>
    </div>

    <!-- Set model modal -->
    <div v-if="settingModelProfile" class="fixed inset-0 bg-base-content/20 z-50 flex items-center justify-center">
      <div class="bg-base-100 rounded-lg shadow-xl w-[480px]">
        <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
          <span class="text-sm font-medium">设置 Profile 模型</span>
          <button class="btn btn-sm btn-ghost btn-circle" @click="settingModelProfile = null">
            <SvgIcon name="close" size="14" />
          </button>
        </div>
        <div class="p-4">
          <div class="text-xs text-base-content/50 mb-2">{{ settingModelProfile }}</div>
          <input 
            v-model="settingModelValue"
            class="input input-sm input-bordered w-full"
            placeholder="模型名称 (如 glm-5, claude-sonnet-4)"
          />
          <div class="text-xs text-base-content/40 mt-2">
            模型名称需与 Hermes 配置中的 provider 支持的模型匹配
          </div>
        </div>
        <div class="px-4 py-3 border-t border-base-content/10 flex items-center justify-end gap-2">
          <button class="btn btn-sm btn-ghost" @click="settingModelProfile = null">取消</button>
          <button 
            class="btn btn-sm btn-primary"
            @click="saveModel"
          >
            保存
          </button>
        </div>
      </div>
    </div>

    <!-- Profile detail modal -->
    <div v-if="profileDetail" class="fixed inset-0 bg-base-content/20 z-50 flex items-center justify-center">
      <div class="bg-base-100 rounded-lg shadow-xl w-96 max-h-[80vh] overflow-hidden flex flex-col">
        <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
          <span class="text-sm font-medium">{{ profileDetail.name }}</span>
          <button class="btn btn-sm btn-ghost btn-circle" @click="profileDetail = null">
            <SvgIcon name="close" size="14" />
          </button>
        </div>
        <div class="flex-1 overflow-y-auto p-4">
          <div class="space-y-2">
            <div v-for="(value, key) in profileDetail" :key="key" class="flex items-start gap-2">
              <span class="text-xs text-base-content/50 w-24 shrink-0">{{ key }}</span>
              <span class="text-xs break-all">{{ value }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useAgentModeStore } from '@/stores/agentModeStore'
import { invoke } from '@tauri-apps/api/core';
import SvgIcon from '@/components/ui/SvgIcon.vue';

interface HermesProfile {
  name: string;
  model?: string;
  gatewayStatus?: string;
  alias?: string;
  distribution?: string;
  description?: string;
  isDefault: boolean;
}

const agentModeStore = useAgentModeStore()
const isOmpMode = computed(() => agentModeStore.mode === 'omp')

// State
const profiles = ref<HermesProfile[]>([]);
const workload = ref<Record<string, number>>({});
const dispatcherRunning = ref(false);
const dispatching = ref(false);
const showCreateProfile = ref(false);
const newProfile = ref({ name: '', description: '' });
const editingProfile = ref<string | null>(null);
const editDescriptionText = ref('');
const settingModelProfile = ref<string | null>(null);
const settingModelValue = ref('');
const profileDetail = ref<Record<string, string> | null>(null);

// Methods
async function refreshProfiles() {
  try {
    profiles.value = await invoke('profile_list');
    
    // Get workload
    const wl = await invoke<Array<{ name: string; counts: Record<string, number> }>>('kanban_workload');
    workload.value = {};
    for (const w of wl) {
      const total = Object.values(w.counts || {}).reduce((a, b) => a + b, 0);
      workload.value[w.name] = total;
    }
    
    // Get dispatcher status
    const status = await invoke<Record<string, unknown>>('kanban_dispatcher_status');
    dispatcherRunning.value = status?.['running'] === true;
  } catch (e) {
    console.error('Failed to refresh profiles:', e);
  }
}

function getWorkload(name: string): number {
  return workload.value[name] || 0;
}

function gatewayClass(status?: string): string {
  if (status === 'running') {return 'text-success';}
  if (status === 'stopped') {return 'text-base-content/50';}
  return 'text-warning';
}

async function setDefault(name: string) {
  try {
    await invoke('profile_use', { name });
    await refreshProfiles();
  } catch (e) {
    console.error('Failed to set default:', e);
  }
}

async function deleteProfile(name: string) {
  if (!confirm(`确定删除 Profile "${name}"？`)) {return;}
  try {
    await invoke('profile_delete', { name });
    await refreshProfiles();
  } catch (e) {
    console.error('Failed to delete profile:', e);
  }
}

async function createProfile() {
  if (!newProfile.value.name.trim()) {return;}
  try {
    await invoke('profile_create', {
      name: newProfile.value.name.trim(),
      description: newProfile.value.description.trim() || null,
    });
    showCreateProfile.value = false;
    newProfile.value = { name: '', description: '' };
    await refreshProfiles();
  } catch (e) {
    console.error('Failed to create profile:', e);
  }
}

async function editDescription(name: string) {
  // Fetch current description first
  try {
    console.log('Fetching description for:', name);
    const d = await invoke<string>('profile_get_description', { name });
    console.log('Got description:', d);
    editDescriptionText.value = d || '';
  } catch (e) {
    console.error('Failed to get description:', e);
    editDescriptionText.value = '';
  }
  editingProfile.value = name;
}

async function saveDescription() {
  if (!editingProfile.value) {return;}
  try {
    await invoke('profile_describe', {
      name: editingProfile.value,
      description: editDescriptionText.value.trim(),
    });
    editingProfile.value = null;
    await refreshProfiles();
  } catch (e) {
    console.error('Failed to save description:', e);
  }
}

function openSetModel(name: string, currentModel?: string) {
  settingModelProfile.value = name;
  settingModelValue.value = currentModel || '';
}

async function saveModel() {
  if (!settingModelProfile.value) {return;}
  try {
    await invoke('profile_set_model', {
      name: settingModelProfile.value,
      model: settingModelValue.value.trim(),
    });
    settingModelProfile.value = null;
    await refreshProfiles();
  } catch (e) {
    console.error('Failed to set model:', e);
  }
}

async function showProfileDetail(name: string) {
  try {
    const detail = await invoke('profile_show', { name });
    profileDetail.value = detail as Record<string, string>;
  } catch (e) {
    console.error('Failed to show profile:', e);
  }
}

async function updateProfile(name: string) {
  try {
    await invoke('profile_update', { name });
    await refreshProfiles();
  } catch (e) {
    console.error('Failed to update profile:', e);
  }
}

async function manualDispatch() {
  dispatching.value = true;
  try {
    const result = await invoke('kanban_dispatch', { dryRun: false, maxSpawns: null });
    console.log('Dispatch result:', result);
    await refreshProfiles();
  } catch (e) {
    console.error('Failed to dispatch:', e);
  }
  dispatching.value = false;
}

async function dryRunDispatch() {
  try {
    const result = await invoke('kanban_dispatch', { dryRun: true, maxSpawns: null });
    console.log('Dry run result:', result);
    alert(JSON.stringify(result, null, 2));
  } catch (e) {
    console.error('Failed to dry run:', e);
  }
}

onMounted(refreshProfiles);
</script>