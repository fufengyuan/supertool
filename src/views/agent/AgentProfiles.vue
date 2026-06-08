<template>
  <div class="h-full flex flex-col">
    <!-- Claw mode: Profile overview -->
    <div v-show="isClawMode" class="flex-1 overflow-y-auto">
      <div class="px-4 py-3 border-b border-base-content/10 flex items-center justify-between">
        <h1 class="text-sm font-medium">Claw 配置概览</h1>
        <button class="btn btn-sm btn-ghost" @click="loadClawProfile" :disabled="clawLoading">
          <SvgIcon name="refresh" size="14" />
        </button>
      </div>
      <div v-if="clawLoading" class="flex items-center justify-center py-12">
        <span class="loading loading-spinner loading-sm"></span>
      </div>
      <div v-else-if="clawProfile" class="p-4 space-y-4">
        <!-- Config home -->
        <div class="bg-base-100 rounded-lg border border-base-content/10 p-4">
          <h3 class="text-xs font-semibold uppercase tracking-wider text-base-content/60 mb-3">配置目录</h3>
          <p class="text-sm font-mono text-base-content/80">{{ clawProfile.configHome }}</p>
        </div>
        <!-- Stats grid -->
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
          <div class="bg-base-100 rounded-lg border border-base-content/10 p-3 text-center">
            <div class="text-2xl font-bold text-primary">{{ clawProfile.mcpServerCount }}</div>
            <div class="text-xs text-base-content/50 mt-1">MCP 服务器</div>
          </div>
          <div class="bg-base-100 rounded-lg border border-base-content/10 p-3 text-center">
            <div class="text-2xl font-bold text-secondary">{{ clawProfile.pluginCount }}</div>
            <div class="text-xs text-base-content/50 mt-1">已安装插件</div>
          </div>
          <div class="bg-base-100 rounded-lg border border-base-content/10 p-3 text-center">
            <div class="text-2xl font-bold" :class="clawProfile.hasPermissions ? 'text-success' : 'text-base-content/30'">{{ clawProfile.hasPermissions ? '✓' : '—' }}</div>
            <div class="text-xs text-base-content/50 mt-1">权限规则</div>
          </div>
          <div class="bg-base-100 rounded-lg border border-base-content/10 p-3 text-center">
            <div class="text-2xl font-bold" :class="clawProfile.hasHooks ? 'text-success' : 'text-base-content/30'">{{ clawProfile.hasHooks ? '✓' : '—' }}</div>
            <div class="text-xs text-base-content/50 mt-1">Hooks</div>
          </div>
        </div>
        <!-- Raw settings -->
        <div v-if="clawProfile.rawSettings" class="bg-base-100 rounded-lg border border-base-content/10 p-4">
          <h3 class="text-xs font-semibold uppercase tracking-wider text-base-content/60 mb-3">settings.json</h3>
          <pre class="text-xs text-base-content/70 overflow-x-auto max-h-64 overflow-y-auto">{{ JSON.stringify(clawProfile.rawSettings, null, 2) }}</pre>
        </div>
        <div v-else class="bg-base-100 rounded-lg border border-base-content/10 p-4 text-center">
          <p class="text-sm text-base-content/40">尚未创建 settings.json</p>
        </div>

        <!-- Agent Configuration List -->
        <div class="bg-base-100 rounded-lg border border-base-content/10 p-4">
          <div class="flex items-center justify-between mb-3">
            <h3 class="text-xs font-semibold uppercase tracking-wider text-base-content/60">Agent 配置</h3>
            <button class="btn btn-xs btn-ghost" @click="loadClawAgents" :disabled="agentsLoading">
              <SvgIcon name="refresh" size="12" />
            </button>
          </div>
          <div v-if="agentsLoading" class="flex items-center justify-center py-4">
            <span class="loading loading-spinner loading-xs"></span>
          </div>
          <div v-else-if="agentsError" class="text-xs text-error/70 py-2">{{ agentsError }}</div>
          <div v-else-if="agents.length === 0" class="text-xs text-base-content/40 py-2 text-center">暂无 agent 配置</div>
          <div v-else class="space-y-2 max-h-48 overflow-y-auto">
            <div
              v-for="agent in agents"
              :key="agent.name"
              class="flex items-start justify-between p-2 rounded hover:bg-base-200/50 transition-colors"
            >
              <div class="flex-1 min-w-0">
                <div class="text-xs font-medium truncate">{{ agent.name }}</div>
                <div v-if="agent.description" class="text-[10px] text-base-content/50 truncate">{{ agent.description }}</div>
                <div v-if="agent.model" class="text-[10px] text-base-content/40 mt-0.5">模型: {{ agent.model }}</div>
              </div>
              <div class="text-[10px] text-base-content/30 ml-2 truncate max-w-[120px]" :title="agent.path">
                {{ agent.path.split('/').pop() || agent.path }}
              </div>
            </div>
          </div>
        </div>

        <!-- Permission Mode Selector -->
        <div class="bg-base-100 rounded-lg border border-base-content/10 p-4">
          <h3 class="text-xs font-semibold uppercase tracking-wider text-base-content/60 mb-3">权限模式</h3>
          <div class="flex items-center gap-2">
            <select
              v-model="permissionMode"
              class="select select-sm select-bordered flex-1"
              :disabled="permissionModeLoading"
              @change="setPermissionMode"
            >
              <option value="ask">Ask (每次询问)</option>
              <option value="allow">Allow (自动允许)</option>
              <option value="deny">Deny (自动拒绝)</option>
            </select>
            <button
              class="btn btn-xs btn-ghost"
              :class="{ 'loading': permissionModeLoading }"
              :disabled="permissionModeLoading"
              @click="loadPermissionMode"
            >
              <SvgIcon name="refresh" size="12" />
            </button>
          </div>
          <div v-if="permissionModeError" class="text-xs text-error/70 mt-2">{{ permissionModeError }}</div>
          <div v-if="permissionModeSaved" class="text-xs text-success/70 mt-2">已保存</div>
        </div>
      </div>
      <div v-else class="text-center py-12 text-base-content/30 text-sm">加载失败</div>
    </div>

    <div v-show="!isClawMode" class="h-full flex flex-col">
      <div class="px-4 py-3 border-b border-base-content/10 flex items-center justify-between">
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useAgentModeStore } from '@/stores/agentModeStore'
import { invoke } from '@tauri-apps/api/core';
import SvgIcon from '@/components/ui/SvgIcon.vue';
import { getTauriAPI } from '@/utils/tauri-api';

interface HermesProfile {
  name: string;
  model?: string;
  gatewayStatus?: string;
  alias?: string;
  distribution?: string;
  description?: string;
  isDefault: boolean;
}

interface AgentInfo {
  name: string;
  description: string;
  model: string;
  path: string;
  config?: unknown;
}

const agentModeStore = useAgentModeStore()
const isClawMode = computed(() => agentModeStore.mode === 'claw')

// Claw profile state
const clawProfile = ref<{ configHome: string; settingsExists: boolean; mcpServerCount: number; pluginCount: number; hasPermissions: boolean; hasHooks: boolean; hasFeatures: boolean; rawSettings: unknown | null } | null>(null)
const clawLoading = ref(false)

async function loadClawProfile() {
  clawLoading.value = true
  try {
    const api = getTauriAPI()
    clawProfile.value = await api.clawGetProfile()
  } catch (e) {
    console.error('[AgentProfiles] ❌ Failed to load Claw profile:', e)
  } finally {
    clawLoading.value = false
  }
}

// Claw agent list state
const agents = ref<AgentInfo[]>([])
const agentsLoading = ref(false)
const agentsError = ref<string | null>(null)

async function loadClawAgents() {
  agentsLoading.value = true
  agentsError.value = null
  try {
    const api = getTauriAPI()
    agents.value = await api.clawListAgents()
  } catch (e) {
    agentsError.value = String(e)
    console.error('[AgentProfiles] ❌ Failed to load agents:', e)
  } finally {
    agentsLoading.value = false
  }
}

// Permission mode state
const permissionMode = ref('ask')
const permissionModeLoading = ref(false)
const permissionModeError = ref<string | null>(null)
const permissionModeSaved = ref(false)

async function loadPermissionMode() {
  permissionModeLoading.value = true
  permissionModeError.value = null
  permissionModeSaved.value = false
  try {
    const api = getTauriAPI()
    const result = await api.clawGetPermissionMode()
    permissionMode.value = result.mode || 'ask'
  } catch (e) {
    permissionModeError.value = String(e)
    console.error('[AgentProfiles] ❌ Failed to load permission mode:', e)
  } finally {
    permissionModeLoading.value = false
  }
}

async function setPermissionMode() {
  permissionModeLoading.value = true
  permissionModeError.value = null
  permissionModeSaved.value = false
  try {
    const api = getTauriAPI()
    await api.clawSetPermissionMode(permissionMode.value)
    permissionModeSaved.value = true
    setTimeout(() => { permissionModeSaved.value = false }, 3000)
  } catch (e) {
    permissionModeError.value = String(e)
    console.error('[AgentProfiles] ❌ Failed to set permission mode:', e)
  } finally {
    permissionModeLoading.value = false
  }
}

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
  if (status === 'running') return 'text-success';
  if (status === 'stopped') return 'text-base-content/50';
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
  if (!confirm(`确定删除 Profile "${name}"？`)) return;
  try {
    await invoke('profile_delete', { name });
    await refreshProfiles();
  } catch (e) {
    console.error('Failed to delete profile:', e);
  }
}

async function createProfile() {
  if (!newProfile.value.name.trim()) return;
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
  try {
    const d = await invoke<string>('profile_get_description', { name });
    editDescriptionText.value = d || '';
  } catch (e) {
    console.error('Failed to get description:', e);
    editDescriptionText.value = '';
  }
  editingProfile.value = name;
}

async function saveDescription() {
  if (!editingProfile.value) return;
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
  if (!settingModelProfile.value) return;
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
    alert(JSON.stringify(result, null, 2));
  } catch (e) {
    console.error('Failed to dry run:', e);
  }
}

onMounted(() => {
  refreshProfiles();
  if (isClawMode.value) {
    loadClawProfile();
    loadClawAgents();
    loadPermissionMode();
  }
});

watch(isClawMode, (claw) => {
  if (claw) {
    loadClawProfile();
    loadClawAgents();
    loadPermissionMode();
  } else {
    refreshProfiles();
  }
});
</script>
