<template>
  <div class="flex flex-col h-full overflow-hidden">
    <!-- Top Tab Bar -->
    <div role="tablist" class="tabs tabs-bordered bg-base-200 flex-shrink-0">
      <button role="tab" class="tab" :class="{ 'tab-active': cicdTab === 'deploy' }" @click="cicdTab = 'deploy'">
        <SvgIcon name="rocket" :size="14" class="inline-block align-text-bottom" /> 一键部署
      </button>
      <button role="tab" class="tab" :class="{ 'tab-active': cicdTab === 'config' }" @click="cicdTab = 'config'">
        <SvgIcon name="settings" :size="14" class="inline-block align-text-bottom" /> 部署配置
      </button>
    </div>

    <div v-if="cicdTab === 'deploy'" class="flex-1 overflow-y-auto">
      <DeployPanel />
    </div>

    <div v-else class="flex h-full overflow-hidden bg-base-200">
      <!-- Left Sidebar: Config List -->
      <aside :class="[
        'border-r border-base-content/10 bg-base-100 flex flex-col flex-shrink-0 transition-all duration-300 overflow-hidden',
        sidebarCollapsed ? 'w-[52px] min-w-[52px] border-r-transparent hover:border-r-base-content/10' : 'w-[300px] min-w-[260px] max-w-[360px]'
      ]">
        <div :class="sidebarCollapsed ? 'flex flex-col items-center px-2 pt-3' : 'flex items-center justify-between px-5 pt-4 pb-3'">
          <h3 v-show="!sidebarCollapsed" class="m-0 text-base font-bold text-base-content flex items-center gap-1.5"><SvgIcon name="rocket" :size="16" /> 部署配置</h3>
          <div :class="sidebarCollapsed ? 'flex flex-col items-center gap-2.5 w-full' : 'flex gap-2'">
            <button @click="createNewConfig" :class="['btn btn-primary', sidebarCollapsed ? 'p-0 w-9 h-9 rounded-xl' : '']" title="新建配置">
              <SvgIcon name="plus" :size="16" :stroke-width="2.5" />
              <span v-show="!sidebarCollapsed">新建配置</span>
            </button>
            <button @click="sidebarCollapsed = !sidebarCollapsed" :class="['btn btn-ghost', sidebarCollapsed ? 'p-0 w-9 h-9 rounded-xl' : '']" :title="sidebarCollapsed ? '展开列表' : '收起列表'">
              <SvgIcon :name="sidebarCollapsed ? 'chevronRight' : 'chevronLeft'" :size="16" />
              <span v-show="!sidebarCollapsed">收起</span>
            </button>
          </div>
        </div>

        <div class="relative px-4 pb-3" v-show="!sidebarCollapsed">
          <SvgIcon name="search" :size="14" class="absolute left-7 top-1/2 -translate-y-1/2 text-base-content/60 pointer-events-none" />
          <input v-model="searchQuery" placeholder="搜索配置..." class="input input-bordered w-full pl-8 h-9 text-sm bg-base-200" />
        </div>

        <div class="flex-1 overflow-y-auto px-3 pb-3" v-show="!sidebarCollapsed">
          <!-- Empty state -->
          <div v-if="groupedConfigs.size === 0" class="flex flex-col items-center py-10 px-5 text-center text-base-content/60 gap-3">
            <SvgIcon name="folder" :size="40" :stroke-width="1.5" class="opacity-30" />
            <p class="m-0 text-sm">{{ configs.length === 0 ? '还没有部署配置' : '没有匹配的搜索结果' }}</p>
            <button @click="createNewConfig" class="btn btn-primary btn-sm">创建第一个</button>
          </div>

          <!-- Grouped config cards -->
          <template v-for="[groupName, groupConfigs] in groupedConfigs" :key="groupName">
            <div class="mb-1">
              <div class="flex items-center gap-1.5 px-2 py-1.5 cursor-pointer rounded-lg transition-colors duration-150 select-none hover:bg-white/5" @click="toggleGroup(groupName)">
                <SvgIcon name="chevronDown" :size="14" class="transition-transform duration-200" :class="{ '-rotate-90': !expandedGroups.has(groupName) }" />
                <span class="text-xs font-semibold text-base-content/60 uppercase tracking-wider flex-1">{{ groupName }}</span>
                <span class="text-xs text-base-content/60 opacity-60 bg-white/5 rounded-full px-1.5 py-0.5 min-w-[18px] text-center">{{ groupConfigs.length }}</span>
                <button v-if="groupName !== '未分组'" @click.stop="renameGroup(groupName)" class="btn btn-ghost btn-xs px-0.5 py-0 min-h-0 h-auto opacity-0 group-hover:opacity-60 hover:!opacity-100 hover:bg-white/10" title="重命名分组">
                  <SvgIcon name="pencil" :size="12" />
                </button>
              </div>
              <div :class="!expandedGroups.has(groupName) ? 'max-h-0 opacity-0 overflow-hidden transition-all duration-300' : 'max-h-[2000px] opacity-100 overflow-hidden transition-all duration-300'">
                <div
                  v-for="cfg in groupConfigs"
                  :key="cfg.id"
                  class="px-4 py-3.5 rounded-xl cursor-pointer mb-1.5 border border-transparent transition-all duration-150 hover:bg-base-200 hover:border-base-content/10 group"
                  :class="{ 'bg-primary/10 border-primary': selectedConfigId === cfg.id }"
                  @click="openEditWizard(cfg.id)"
                >
                  <div class="flex items-center gap-1.5 mb-1.5">
                    <span class="text-sm font-semibold text-base-content flex-1 min-w-0" :title="cfg.name || getGitRepoName(cfg.gitRepoId)">{{ cfg.name || getGitRepoName(cfg.gitRepoId) }}</span>
                    <span class="text-xs px-2 py-0.5 rounded bg-base-200 text-base-content/60 whitespace-nowrap flex-shrink-0" :class="{ 'bg-white/20 text-primary': selectedConfigId === cfg.id }">{{ cfg.deployBranch || 'main' }}</span>
                    <span v-if="cfg.requiresApproval" class="flex-shrink-0" title="需要审核确认"><SvgIcon name="lock" :size="12" class="inline-block align-text-bottom" /></span>
                  </div>
                  <div class="flex items-center justify-between mb-1.5">
                    <span class="text-xs text-base-content/60 truncate">{{ getServerLabel(cfg) }}</span>
                    <span class="text-sm flex-shrink-0">{{ getToolBadge(cfg.buildTool) }}</span>
                  </div>
                  <div class="flex items-center justify-between">
                    <span class="text-xs text-base-content/60 opacity-60">{{ formatTime(cfg.updatedAt) }}</span>
                    <div class="flex items-center gap-0.5">
                      <button @click.stop="copyConfig(cfg.id)" class="bg-transparent border-none cursor-pointer p-1 rounded text-base-content/60 opacity-0 group-hover:opacity-60 transition-all duration-150 hover:!opacity-100 hover:text-primary hover:bg-primary/10" title="复制配置">
                        <SvgIcon name="copy" :size="12" />
                      </button>
                      <button @click.stop="deleteConfig(cfg.id)" class="bg-transparent border-none cursor-pointer p-1 rounded text-base-content/60 opacity-0 group-hover:opacity-100 transition-all duration-150 hover:!opacity-100 hover:text-error hover:bg-error/10" title="删除">
                        <SvgIcon name="trash" :size="12" />
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </template>
        </div>
      </aside>

      <!-- Right Main Area -->
      <main class="flex-1 overflow-y-auto flex flex-col">
        <!-- No config selected -->
        <div v-if="!selectedConfigId && !isNewConfig" class="flex-1 flex flex-col items-center justify-center gap-4 text-base-content/60">
          <div class="opacity-20">
            <SvgIcon name="folderPlus" :size="64" :stroke-width="1.5" />
          </div>
          <h3 class="m-0 text-xl text-base-content">选择或创建部署配置</h3>
          <p class="m-0 text-sm">从左侧选择一个已有配置，或创建新的部署配置</p>
          <button @click="createNewConfig" class="btn btn-primary btn-lg">＋ 新建部署配置</button>
        </div>

        <!-- Config Wizard: 新建 + 编辑共用 -->
        <div v-else class="flex-1 flex flex-col">
          <CicdConfigWizard
            :git-repos="gitRepos"
            :groups="groups"
            :servers="servers"
            :server-groups="serverGroups"
            :build-tools="availableBuildTools"
            :initial="editWizardInitial"
            @complete="applyWizardPayload"
            @cancel="cancelWizard"
          />
        </div>
      </main>

      <!-- Group Name Dialog -->
      <div v-if="showGroupDialog" class="fixed inset-0 bg-black/60 flex items-center justify-center z-[10000]" @click.self="cancelGroupDialog">
        <div class="bg-base-200 border border-base-content/10 rounded-xl p-6 w-[360px] max-w-[90vw] shadow-[0_20px_60px_rgba(0,0,0,0.5)] animate-[slideUp_0.2s_ease]" @keydown.escape="cancelGroupDialog">
          <h4 class="m-0 mb-4 text-base text-base-content">{{ groupDialogMode === 'add' ? '新建分组' : '重命名分组' }}</h4>
          <input
            v-model="groupNameInput"
            ref="groupNameInputRef"
            class="input input-bordered w-full bg-base-200 text-sm mb-4"
            placeholder="输入分组名称"
            @keydown.enter="confirmGroupDialog"
            autofocus
          />
          <div class="flex justify-end gap-2">
            <button class="btn btn-ghost" @click="cancelGroupDialog">取消</button>
            <button class="btn btn-primary" @click="confirmGroupDialog" :disabled="!groupNameInput.trim()">确定</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'CiCdConfig' })
import { useCicdConfig } from './composables/useCicdConfig';
import ModuleTreeNode from './ModuleTreeNode.vue';
import DeployModeSelector from './DeployModeSelector.vue';
import GroupedServerSelector from '../server/GroupedServerSelector.vue';
import CicdConfigWizard from './CicdConfigWizard.vue';
import { computed, defineAsyncComponent, onBeforeUnmount, ref } from 'vue'
import SvgIcon from '@/components/ui/SvgIcon.vue';

const DeployPanel = defineAsyncComponent(() => import('./DeployPanel.vue'));

const cicdTab = ref<'deploy' | 'config'>('deploy')

// Listen for DeployPanel's "go to config" event
const _onSwitchCicdTab = ((e: CustomEvent) => {
  cicdTab.value = e.detail
}) as EventListener
if (typeof window !== 'undefined') {
  window.addEventListener('switch-cicd-tab', _onSwitchCicdTab)
}
onBeforeUnmount(() => {
  if (typeof window !== 'undefined') {
    window.removeEventListener('switch-cicd-tab', _onSwitchCicdTab)
  }
})

// 保存/测试连接防重复点击
const cicd = useCicdConfig();

// 编辑模式预填数据：config + modules（合成向导 initial）
const editWizardInitial = computed<Record<string, unknown> | null>(() => {
  if (cicd.isNewConfig.value || !selectedConfigId.value) {return null;}
  return { ...cicd.config.value, modules: cicd.modules.value, servers: cicd.deployServers.value };
});

// 点击已有配置：加载后进入编辑向导
async function openEditWizard(id: string) {
  cicd.isNewConfig.value = false;
  cicd.selectedConfigId.value = id;
  try { await cicd.loadConfig(id); } catch {/* 忽略 */}
}

function cancelWizard() {
  cicd.isNewConfig.value = false;
  cicd.selectedConfigId.value = '';
}

// 向导完成：新建 or 更新（payload 携带 id 即为编辑）
async function applyWizardPayload(payload: Record<string, unknown>) {
  const p = payload as {
    id?: string | null; name?: string; gitRepoId?: string; groupName?: string; deployBranch?: string;
    buildTool?: string; mavenProfile?: string; npmScript?: string; npmCustomScript?: string;
    restartScript?: string; deployPath?: string; localPath?: string; repoUrl?: string;
    mavenHome?: string; javaHome?: string; nodeHome?: string; mavenSettings?: string; buildCommand?: string;
    parentBuildMode?: boolean; parentBuildPath?: string; outputPath?: string; libSeparate?: boolean;
    incrementalUpload?: boolean; requiresApproval?: boolean;
    healthCheckUrl?: string; healthCheckTimeout?: number; healthCheckRetries?: number;
    environments?: Record<string, unknown>[];
    servers?: { serverId: string; label?: string; deployDir?: string }[];
    modules?: (DeployModule | { moduleName: string; modulePath: string; enabled: boolean })[];
  };
  Object.assign(cicd.config.value, {
    name: p.name || '',
    gitRepoId: p.gitRepoId || '',
    groupName: p.groupName || '未分组',
    deployBranch: p.deployBranch || 'main',
    buildTool: p.buildTool || '',
    mavenProfile: p.mavenProfile || 'prod',
    npmScript: p.npmScript || 'build',
    npmCustomScript: p.npmCustomScript || '',
    restartScript: p.restartScript || './restart.sh',
    deployPath: p.deployPath || '',
    repoUrl: p.repoUrl || '',
    mavenHome: p.mavenHome || '',
    javaHome: p.javaHome || '',
    nodeHome: p.nodeHome || '',
    mavenSettings: p.mavenSettings || '',
    buildCommand: p.buildCommand || '',
    incrementalUpload: p.incrementalUpload ?? cicd.config.value.incrementalUpload ?? true,
    requiresApproval: p.requiresApproval ?? cicd.config.value.requiresApproval ?? false,
    healthCheckUrl: p.healthCheckUrl || cicd.config.value.healthCheckUrl || '',
    healthCheckTimeout: p.healthCheckTimeout ?? cicd.config.value.healthCheckTimeout ?? 30,
    healthCheckRetries: p.healthCheckRetries ?? cicd.config.value.healthCheckRetries ?? 2,
    // 部署模式：向导显式传入，覆盖默认（避免多模块一律强制成父模块单 jar）
    parentBuildMode: p.parentBuildMode ?? cicd.config.value.parentBuildMode,
    parentBuildPath: p.parentBuildPath ?? cicd.config.value.parentBuildPath,
    outputPath: p.outputPath ?? (cicd.config.value.outputPath || ''),
    libSeparate: p.libSeparate ?? cicd.config.value.libSeparate,
    environments: p.environments && p.environments.length ? p.environments : cicd.config.value.environments || [],
  });
  // 代码实际目录（可能不在 git 仓库根目录，如 src/xxx）
  if (p.localPath) { cicd.config.value.localPath = p.localPath; }
  cicd.deployServers.value = (p.servers || []).map(s => {
    // 编辑时保留已有 deployDir（向导不涉及该字段）
    const existing = cicd.deployServers.value.find(d => d.serverId === s.serverId);
    return { serverId: s.serverId, label: s.label || '', deployDir: s.deployDir || existing?.deployDir || '' };
  });
  // 多模块：编辑时保留已有模块 id（复用 src 原字段仅调 enabled）；新建时重建
  cicd.modules.value = (p.modules || []).map(m => {
    const src = (m as DeployModule);
    const isExisting = src.id != null;
    return {
      id: isExisting ? src.id : null,
      configId: isExisting ? (cicd.config.value.id == null ? null : cicd.config.value.id) : null,
      moduleName: (m as { moduleName: string }).moduleName,
      modulePath: (m as { modulePath: string }).modulePath || (m as { moduleName: string }).moduleName,
      artifactName: src.artifactName || '',
      artifactType: src.artifactType || '',
      buildCommand: src.buildCommand || '',
      buildPath: src.buildPath || '',
      outputPath: src.outputPath || '',
      buildTool: src.buildTool || '',
      deployPath: src.deployPath || '',
      enabled: (m as { enabled: boolean }).enabled !== false,
      deployOrder: src.deployOrder ?? 0,
      createdAt: src.createdAt || new Date().toISOString(),
      updatedAt: src.updatedAt || new Date().toISOString(),
    } as DeployModule;
  });
  // 单 jar 模式（parentBuildMode=true）：补充父构建目录，取 git 仓库本地路径。
  // 仅 maven 场景需要：父统一构建要求 parentBuildPath 指向父 POM 目录；
  // npm 单体项目留空即表示「主模块目录/localPath 本身」，填绝对路径会被
  // single_deploy_root 错误 join（Rust PathBuf::join 遇绝对路径整体替换），导致打包原路径。
  if (
    cicd.config.value.parentBuildMode &&
    !cicd.config.value.parentBuildPath &&
    (cicd.config.value.buildTool === 'maven' ||
      (!cicd.config.value.buildTool && cicd.config.value.javaHome))
  ) {
    const repo = gitRepos.value.find((r: any) => r.id === p.gitRepoId);
    cicd.config.value.parentBuildPath = repo?.path || cicd.config.value.localPath || '';
  }
  await cicd.saveConfig();
  // 保存成功（saveConfig 内部已把 isNewConfig 置 false）后退出向导，停在被编辑配置的列表选中态
  charmAfterSave();
}

function charmAfterSave() {
  // 保存后停在向导态，不跳转
}

// ─── 多环境：环境内服务器管理 ───
function addEnvServer() {
  const env = cicd.config.value.environments[cicd.activeEnvIdx.value];
  if (env) { env.servers.push({ serverId: '', deployDir: '' }); }
}
function removeEnvServer(i: number) {
  const env = cicd.config.value.environments[cicd.activeEnvIdx.value];
  if (env) { env.servers.splice(i, 1); }
}

// 删除模块前确认（已保存模块会删库）
function confirmDeleteModule(module: { id: string | null; moduleName: string }) {
  if (module.id && !confirm(`确定删除模块「${module.moduleName || '未命名'}」吗？此操作会从数据库中删除。`)) { return; }
  cicd.deleteModule(module.id);
}

// Git 仓库名称查找函数（供模板使用）
function openInFileManager(path: string) {
  import('../../utils/tauri-api').then(({ getTauriAPI }) => {
    getTauriAPI().openInFileManager(path).catch(() => {});
  });
}

// Computed getter/setter for the dynamic `config.*Home` property based on buildTool
const currentHomePath = computed({
  get: () => (cicd.config.value as Record<string, any>)[`${cicd.config.value.buildTool}Home`] ?? '',
  set: (val: string) => { (cicd.config.value as Record<string, any>)[`${cicd.config.value.buildTool}Home`] = val; },
});

// Helper to access defaultPaths.*Home dynamically
function getDefaultPathFor(tool: string): string {
  return (cicd.defaultPaths.value as Record<string, any>)[`${tool}Home`] ?? '';
}

// Destructure all refs, computed, and functions for template access
const {
  configs, projects, gitRepos, servers, serverGroups, selectedConfigId, isNewConfig, searchQuery, sidebarCollapsed,
  selectedServerId, deployServers, activeServerIdx, groups, expandedGroups,
  showGroupDialog, groupNameInput, groupDialogMode, groupDialogOldName,
  showGroupEditor, newGroupName,
  config, modules, testResult, detectedTools, availableBranches, loadingBranches,
  scannedModules, scanningModules, showModuleTree, expandedTreeNodes,
  defaultPaths, sdkVersions, selectedJavaVersion, selectedNodeVersion, detectingPaths,
  sdkmanInstallGuide, nvmInstallGuide,
  filteredConfigs, groupedConfigs, hasAnyGitSource, gitSources,
  projectShortName, availableBuildTools, addedModulePaths, buildToolDefs,
  parentBuildAutoDetected, parentBuildDetectedPath, selectedGitRepo,
  openGroupDialog, confirmGroupDialog, cancelGroupDialog, initExpandedGroups,
  makeDefaultServer, getServerName, onServerSelect, addServer, removeServer,
  testServerById, onJavaVersionSelected, onNodeVersionSelected, reDetectToolPaths,
  getProjectName, getGitRepoName, getToolBadge, getBuildToolIcon, getBuildToolName, formatTime,
  toggleGroup, renameGroup, addGroup, getServerLabel,
  loadConfigs, createNewConfig, selectConfig, onProjectChange, onGitRepoChange, selectLocalDir,
  selectServer, copyGitUrl, loadBranches, testConnection,
  addModule, scanModules, toggleTreeNode, isModuleAlreadyAdded,
  addModuleFromScan, addAllDetectedModules, flattenModuleTree, autoDetectParentBuild, deleteModule,
  saveConfig, deleteConfig, copyConfig, loadConfig, loadServers, loadProjects, loadGitRepos,
  switchToGitCloneMode, fetchGitRemoteUrl,
  activeEnvIdx, addEnvironment, removeEnvironment,
  defaultConfig, pageLoading,
} = cicd;

// Re-export types for template
import type { CicdConfigEntry, DeployModule, DeployServerEntry, ConfigForm } from './composables/useCicdConfig';
</script>
