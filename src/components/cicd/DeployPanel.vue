<template>
  <div class="deploy-panel">
    <!-- Header -->
    <div class="panel-header">
      <h2>🚀 一键部署</h2>
      <p class="panel-subtitle">选择部署配置，快速将项目部署到目标服务器</p>
    </div>

    <!-- Main Layout: Left Config + Right Log/History -->
    <div class="deploy-layout" v-if="configs.length > 0">
      <!-- Left: Config Selector + Info + Actions -->
      <div class="deploy-sidebar">
        <!-- Config Selector -->
        <div class="card config-selector">
          <label>选择部署配置</label>
          <div class="config-tree">
            <template v-for="[groupName, groupItems] in groupedDeployConfigs" :key="groupName">
              <div class="config-tree-group">
                <div class="config-tree-group-header" @click="toggleDeployGroup(groupName)">
                  <svg class="tree-chevron" :class="{ expanded: expandedDeployGroups.has(groupName) }" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="6 9 12 15 18 9" />
                  </svg>
                  <span class="tree-group-name">{{ groupName }}</span>
                  <span class="tree-group-count">{{ groupItems.length }}</span>
                </div>
                <div class="config-tree-group-body" v-show="expandedDeployGroups.has(groupName)">
                  <div
                    v-for="cfg in groupItems"
                    :key="cfg.id"
                    class="config-tree-item"
                    :class="{ active: selectedConfigId === cfg.id, 'recently-deployed': cfg.lastDeployedAt }"
                    @click="selectDeployConfig(cfg)"
                  >
                    <span class="config-tree-item-name">
                      {{ cfg.name || getProjectName(cfg.projectId) }}
                    </span>
                    <span class="config-tree-item-meta">
                      <span class="meta-branch">{{ cfg.deployBranch || 'main' }}</span>
                      <span class="meta-dot">·</span>
                      <span class="meta-servers">{{ getServerCount(cfg) }}台</span>
                    </span>
                    <span v-if="cfg.lastDeployedAt" class="config-tree-item-check" title="最近部署过">✓</span>
                    <span v-if="cfg.requiresApproval" class="config-tree-item-badge" title="需要审核确认">🔒</span>
                  </div>
                </div>
              </div>
            </template>
          </div>
        </div>

        <!-- Config Details Card -->
        <template v-if="config && project">
          <div class="card config-info">
            <div class="card-title">配置详情</div>
            <div class="info-grid">
              <div class="info-item" v-if="config.name">
                <span class="info-label">名称</span>
                <span class="info-value">{{ config.name }}</span>
              </div>
              <div class="info-item">
                <span class="info-label">项目</span>
                <span class="info-value">{{ project.name }}</span>
              </div>
              <div class="info-item">
                <span class="info-label">分支</span>
                <span class="info-value branch-badge">{{ config.deployBranch || 'main' }}</span>
              </div>
              <div class="info-item">
                <span class="info-label">服务器</span>
                <span class="info-value">{{ getServersInfo(config) }}</span>
              </div>
              <div class="info-item">
                <span class="info-label">构建工具</span>
                <span class="info-value">{{ getBuildToolName(config.buildTool) }}</span>
              </div>
              <div class="info-item" v-if="config.deployPath">
                <span class="info-label">部署路径</span>
                <span class="info-value code">{{ config.deployPath }}</span>
              </div>
              <div class="info-item" v-if="config.restartScript">
                <span class="info-label">重启脚本</span>
                <span class="info-value code">{{ config.restartScript }}</span>
              </div>
            </div>
          </div>

          <!-- Deploy Actions -->
          <div class="card deploy-actions">
            <button @click="runPreflight" :disabled="deploying" class="btn btn-ghost">
              🔍 部署预检
            </button>
            <button @click="startDeploy" :disabled="deploying || !selectedConfigId" class="btn btn-primary btn-deploy" :class="{ 'btn-requires-approval': config?.requiresApproval }">
              {{ deploying ? '部署中...' : (config?.requiresApproval ? '🔒 审核部署' : '🚀 开始部署') }}
            </button>
          </div>

          <!-- Pre-flight Results -->
          <div v-if="preflightResults.length > 0" class="card preflight-results">
            <div class="card-title">预检结果</div>
            <div v-for="(r, i) in preflightResults" :key="i" class="preflight-item" :class="r.passed ? 'passed' : 'failed'">
              <span class="preflight-icon">{{ r.passed ? '✅' : '❌' }}</span>
              <span class="preflight-name">{{ r.name }}</span>
              <span class="preflight-message">{{ r.message }}</span>
            </div>
          </div>

          <!-- Progress -->
          <div class="card deploy-progress" v-if="deploying || progress > 0">
            <div class="progress-header">
              <span class="progress-label">{{ currentStep || '准备部署...' }}</span>
              <button v-if="deploying" @click="cancelDeploy" class="btn-cancel-deploy">⏹ 取消</button>
            </div>
            <div class="progress-bar">
              <div class="progress-fill" :style="{ width: progress + '%' }" :class="{ 'progress-cancelled': deployCancelled }"></div>
            </div>
            <span class="progress-pct">{{ Math.round(progress) }}%</span>
          </div>
        </template>
      </div>

      <!-- Right: Log + History -->
      <div class="deploy-main">
        <!-- Real-time Log -->
        <div class="card realtime-log" v-if="deploying || realtimeLogs.length > 0">
          <div class="card-title-row">
            <span class="card-title">📋 实时日志</span>
            <button @click="clearRealtimeLogs" class="btn-clear-logs">清空</button>
          </div>
          <div ref="logContainer" class="log-output">
            <div v-for="(line, i) in realtimeLogs" :key="i" class="log-line" :class="'log-' + (line.stage || 'info')">
              <span class="log-time">{{ line.time }}</span>
              <span class="log-stage">[{{ line.stage || 'info' }}]</span>
              <span class="log-msg">{{ line.message }}</span>
            </div>
            <div v-if="deploying" class="log-line log-info">
              <span class="log-time">{{ currentTime }}</span>
              <span class="log-stage">[deploy]</span>
              <span class="log-msg log-spinner">⠋ 部署进行中...</span>
            </div>
          </div>
        </div>

        <!-- Deploy History -->
        <div class="card deploy-history">
          <div class="card-title-row">
            <span class="card-title">部署历史</span>
            <span class="history-count">{{ logs.length }} 条记录</span>
          </div>

          <div class="logs-list">
            <div v-for="log in logs" :key="log.id" class="log-item" :class="log.status">
              <div class="log-header">
                <span class="log-status">
                  {{
                    log.status === 'success' ? '✅'
                    : log.status === 'failed' ? '❌'
                    : log.status === 'running' ? '🔄'
                    : log.status === 'cancelled' ? '⏹️'
                    : log.status === 'rolled_back' ? '↩️'
                    : '⏳'
                  }}
                </span>
                <span class="log-config-name" v-if="log.configName">{{ log.configName }}</span>
                <span class="log-config-group" v-if="log.configGroupName">{{ log.configGroupName }}</span>
                <span class="log-project-name" v-if="log.projectName">({{ log.projectName }})</span>
                <span class="log-time">{{ formatDate(log.createdAt) }}</span>
                <span class="log-trigger">触发: {{ log.triggeredBy }}</span>
                <span class="log-branch" v-if="log.deployBranch">🔀 {{ log.deployBranch }}</span>
                <button
                  v-if="log.status === 'success'"
                  @click="rollbackDeploy(log)"
                  :disabled="rollingBackId === log.id"
                  class="btn-rollback"
                  title="回滚到此版本"
                >
                  {{ rollingBackId === log.id ? '⏳ 回滚中' : '🔄 回滚' }}
                </button>
              </div>

              <div class="log-details" v-if="log.status === 'failed'">
                <p class="error-message">{{ log.errorMessage || '未知错误' }}</p>
              </div>

              <div class="log-details" v-if="log.status === 'cancelled'">
                <p class="cancelled-message">部署已取消</p>
              </div>

              <div class="log-details" v-if="expandedLog === log.id">
                <!-- Full log file viewer -->
                <div v-if="fullLogContent !== null" class="full-log-viewer">
                  <div class="full-log-header">
                    <span>📋 完整日志</span>
                    <button @click="closeFullLog" class="btn-close-log">✕ 关闭</button>
                  </div>
                  <pre class="full-log-content">{{ fullLogContent }}</pre>
                </div>

                <!-- Step logs -->
                <div v-if="!fullLogContent && stepLogs[log.id] && stepLogs[log.id].length > 0" class="step-logs">
                  <div
                    v-for="step in stepLogs[log.id]"
                    :key="step.id"
                    class="step-item"
                    :class="step.status"
                  >
                    <div class="step-header">
                      <span class="step-name">{{ step.stepName }}</span>
                      <span class="step-status-badge" :class="step.status">
                        {{ step.status === 'success' ? '✅' : step.status === 'failed' ? '❌' : step.status === 'running' ? '⏳' : '⏸️' }}
                        {{ step.status }}
                      </span>
                    </div>
                    <div class="step-meta" v-if="step.startTime || step.endTime">
                      <span class="step-time">{{ step.startTime ? formatDate(step.startTime) : '-' }} → {{ step.endTime ? formatDate(step.endTime) : '进行中' }}</span>
                    </div>
                    <pre v-if="step.output" class="step-output">{{ step.output }}</pre>
                    <p v-if="step.errorMessage" class="step-error">{{ step.errorMessage }}</p>
                  </div>
                </div>

                <!-- Raw log output (fallback when no step logs but logOutput exists) -->
                <div v-else-if="!fullLogContent && log.logOutput" class="raw-log-output">
                  <div class="raw-log-header">📋 部署日志</div>
                  <pre class="step-output">{{ log.logOutput }}</pre>
                </div>

                <!-- Auto-load log file when step logs are empty -->
                <div v-else-if="!fullLogContent && log.logFilePath" class="raw-log-output">
                  <div class="raw-log-header">📋 部署日志</div>
                  <pre class="step-output" v-if="loadedLogContent[log.id]">{{ loadedLogContent[log.id] }}</pre>
                  <pre class="step-output" v-else-if="loadingLogContent[log.id]">⏳ 读取日志中...</pre>
                  <button
                    v-else
                    @click="loadLogContent(log)"
                    class="btn-view-full-log"
                  >📄 点击加载日志</button>
                </div>

                <!-- No details available -->
                <div v-else-if="!fullLogContent" class="no-details">
                  <p>📭 暂无日志数据</p>
                </div>

                <!-- View full log button -->
                <div v-if="log.logFilePath && fullLogContent === null" class="view-full-log-row">
                  <button @click="viewFullLog(log)" :disabled="loadingLogFile" class="btn-view-full-log">
                    {{ loadingLogFile ? '⏳ 加载中...' : '📄 查看完整日志' }}
                  </button>
                </div>
              </div>

              <button @click="toggleLogDetails(log.id)" class="btn-toggle">
                {{ expandedLog === log.id ? '收起' : '展开详情' }}
              </button>
            </div>

            <div v-if="logs.length === 0" class="empty-logs">暂无部署记录</div>
          </div>
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <div v-else class="empty-state">
      <div class="empty-icon">📦</div>
      <h3>暂无 CI/CD 配置</h3>
      <p>请先在 CI/CD 配置页面创建部署配置</p>
      <button @click="goToConfig" class="btn btn-primary">前往配置页面</button>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, computed, onMounted, onUnmounted, onBeforeUnmount, nextTick, watch } from 'vue';
import { getTauriAPI } from '../../utils/tauri-api'
import { useToast } from '../../composables/useToast';
import { useErrorHandler } from '../../composables/useErrorHandler';
import { useDeployPreflight } from '../../composables/useDeployPreflight';
import type { Project, Server } from '../../types';

interface CicdConfigEntry {
  id: string;
  projectId: string;
  deployBranch: string;
  servers?: string;
  lastDeployedAt?: string | null;
  groupName?: string;
  [key: string]: unknown;
}

interface DeployLog {
  id: string;
  projectId: string;
  configId: string;
  status: string;
  createdAt: string;
  triggeredBy: string;
  errorMessage?: string;
  logOutput?: string;
  currentStep?: string;
  progress?: number;
  startTime?: string;
  endTime?: string;
  logFilePath?: string;
  artifactPaths?: string;
  deployBranch?: string;
  // Enriched fields (not from DB)
  projectName?: string;
  configName?: string;
  configGroupName?: string;
  projectCategory?: string;
}

interface DeployStep {
  id: string;
  stepName: string;
  status: string;
  output?: string;
  startTime?: string;
  endTime?: string;
  errorMessage?: string;
}

const toast = useToast();
const { handleError } = useErrorHandler();
const { runAll: runPreflightCheck } = useDeployPreflight();

const configs = ref<CicdConfigEntry[]>([]);
const selectedConfigId = ref('');
const config = ref<CicdConfigEntry | null>(null);
const project = ref<Project | null>(null);
const projects = ref<Project[]>([]);
const servers = ref<Server[]>([]);
const serverGroups = ref<Array<{ id: string; name: string; color: string; parentId: string | null }>>([]);
const logs = ref<DeployLog[]>([]);
const stepLogs = ref<Record<string, DeployStep[]>>({});
const expandedLog = ref<string | null>(null);
const loadedLogContent = ref<Record<string, string>>({});
const loadingLogContent = ref<Record<string, boolean>>({});

// Tree selector state
const expandedDeployGroups = ref<Set<string>>(new Set());

// Deploy state — individual refs (same as working Electron version)
const deploying = ref(false);
const deployCancelled = ref(false);
const progress = ref(0);
const currentStep = ref('');
const realtimeLogs = ref<{ time: string; stage: string; message: string }[]>([]);
const activeDeployLogId = ref<string | null>(null);
const lastLoggedProgress = ref(-1);

const rollingBack = ref(false);
const rollingBackId = ref<string | null>(null);
const preflightResults = ref<{ name: string; passed: boolean; message: string }[]>([]);
const logContainer = ref(null);

function resetDeployState() {
  deploying.value = false;
  deployCancelled.value = false;
  progress.value = 0;
  currentStep.value = '';
  realtimeLogs.value = [];
  activeDeployLogId.value = null;
  lastLoggedProgress.value = -1;
}

// Full log file viewer state
const fullLogContent = ref<string | null>(null);
const loadingLogFile = ref(false);

const currentTime = computed(() => {
  return new Date().toLocaleTimeString('zh-CN');
});

// 按最后部署时间排序：最近部署的排在最前面
const sortedConfigs = computed(() => {
  return [...configs.value].sort((a, b) => {
    const aTime = ((a as CicdConfigEntry).lastDeployedAt as string) || ''
    const bTime = ((b as CicdConfigEntry).lastDeployedAt as string) || ''
    if (aTime && bTime) return bTime.localeCompare(aTime)
    if (aTime) return -1
    if (bTime) return 1
    return 0
  })
})

// Configs grouped by groupName (for tree selector)
const groupedDeployConfigs = computed(() => {
  const map = new Map<string, CicdConfigEntry[]>();
  const sorted = sortedConfigs.value;
  for (const cfg of sorted) {
    const group = (cfg as CicdConfigEntry).groupName || '未分组';
    if (!map.has(group)) map.set(group, []);
    map.get(group)!.push(cfg);
  }
  return map;
});

// 默认所有分组收起，用户手动点击切换
function toggleDeployGroup(groupName: string) {
  const set = new Set(expandedDeployGroups.value);
  if (set.has(groupName)) {
    set.delete(groupName);
  } else {
    set.add(groupName);
  }
  expandedDeployGroups.value = set;
}

function getProjectName(projectId: string) {
  const proj = projects.value.find(p => p.id === projectId);
  return proj ? proj.name : 'Project ' + projectId;
}

function getBuildToolName(tool: unknown) {
  const names: Record<string, string> = { maven: 'Maven', npm: 'npm', pnpm: 'pnpm', yarn: 'Yarn', gradle: 'Gradle' };
  return names[String(tool || '')] || '未设置';
}

// Get server names from config's servers JSON
function getServerNames(cfg: CicdConfigEntry | null): string[] {
  if (!cfg?.servers) return [];
  try {
    const parsed = JSON.parse(cfg.servers);
    if (!Array.isArray(parsed) || parsed.length === 0) return [];
    return parsed.map((s: any) => {
      const srv = servers.value.find(sv => sv.id === s.serverId);
      if (!srv) return s.label || s.serverId;
      // Find group name
      const group = srv.groupId ? serverGroups.value.find(g => g.id === srv.groupId) : null;
      const groupTag = group ? ` [${group.name}]` : '';
      return `${srv.name}${groupTag} (${srv.host}:${srv.port || 22})`;
    });
  } catch {
    return [];
  }
}

function getServerLabel(cfg: CicdConfigEntry): string {
  const names = getServerNames(cfg);
  return names.length > 0 ? names.join(', ') : '未配置服务器';
}

function getServerCount(cfg: CicdConfigEntry | null): number {
  return getServerNames(cfg).length;
}

async function selectDeployConfig(cfg: CicdConfigEntry) {
  selectedConfigId.value = cfg.id;
  resetDeployState();
  await loadConfigData(cfg.id);
}

function getServersInfo(cfg: CicdConfigEntry | null): string {
  const names = getServerNames(cfg);
  return names.length > 0 ? names.join(', ') : '未配置';
}

function goToConfig() {
  window.dispatchEvent(new CustomEvent('switch-cicd-tab', { detail: 'config' }));
}

const progressHandler = (data: { progress?: number; message?: string; stage?: string; status?: string; configId?: string }) => {
  const cfgId = data.configId || selectedConfigId.value;
  if (!cfgId || cfgId !== selectedConfigId.value) return;
  const pct = data.progress || 0;
  const isUploadProgress = data.stage === 'ssh' && data.status === 'uploading' && pct > 0;
  const shouldThrottle = isUploadProgress && (pct - lastLoggedProgress.value < 5) && pct < 100;
  if (!shouldThrottle) {
    if (isUploadProgress) lastLoggedProgress.value = pct;
    const now = new Date().toLocaleTimeString('zh-CN');
    realtimeLogs.value = [...realtimeLogs.value, { time: now, stage: data.stage || 'info', message: data.message || '' }];
    scrollToBottom();
  }
  progress.value = pct;
  currentStep.value = data.message || currentStep.value;
};

let cleanupDeployProgress: (() => void) | undefined;
let cleanupDeployNotification: (() => void) | undefined;
let _cleanupDataChanged: (() => void) | undefined;

onMounted(async () => {
    console.log("[components/cicd/DeployPanel.vue] mounted")
  try {
    const [allConfigs, allProjects, allServers, allSGroups] = await Promise.all([
      getTauriAPI().getCicdConfigs?.() as Promise<CicdConfigEntry[]> | undefined,
      getTauriAPI().getProjects?.() as Promise<Project[]> | undefined,
      getTauriAPI().getAllServers?.() as Promise<Server[]> | undefined,
      getTauriAPI().getServerGroups?.() as Promise<Array<{ id: string; name: string; color: string; parentId: string | null }>> | undefined,
    ]);
    configs.value = (allConfigs as CicdConfigEntry[]) || [];
    projects.value = (allProjects as Project[]) || [];
    servers.value = (allServers as Server[]) || [];
    serverGroups.value = (allSGroups as Array<{ id: string; name: string; color: string; parentId: string | null }>) || [];
    // deploy state managed per-config via resetDeployState()

    if (configs.value.length > 0) {
      // 默认选中最近部署的配置（已按 lastDeployedAt 排序）
      const sorted = [...configs.value].sort((a, b) => {
        const aTime = ((a as CicdConfigEntry).lastDeployedAt as string) || ''
        const bTime = ((b as CicdConfigEntry).lastDeployedAt as string) || ''
        if (aTime && bTime) return bTime.localeCompare(aTime)
        if (aTime) return -1
        if (bTime) return 1
        return 0
      })
      selectedConfigId.value = sorted[0].id;
      await loadConfigData(selectedConfigId.value);
    }

    // Enable deploy progress events from Tauri backend
    cleanupDeployProgress = await getTauriAPI().onDeployProgress?.(progressHandler);
    cleanupDeployNotification = await getTauriAPI().onDeployNotification?.((data) => {
      const cfgId = (data as any).configId || selectedConfigId.value;
      if (!cfgId || cfgId !== selectedConfigId.value) return;
      if (data.success) {
        deploying.value = false;
        progress.value = 100;
        currentStep.value = '部署成功！';
        realtimeLogs.value = [...realtimeLogs.value, { time: new Date().toLocaleTimeString('zh-CN'), stage: 'deploy', message: '✅ 部署成功完成' }];
        toast.success(`部署完成`);
        refreshLogs();
      } else if (data.cancelled) {
        deploying.value = false;
        deployCancelled.value = true;
        currentStep.value = '⏹️ 部署已取消';
        toast.info('部署已取消');
      } else {
        deploying.value = false;
        currentStep.value = '部署失败: ' + (data.error || '未知错误');
        realtimeLogs.value = [...realtimeLogs.value, { time: new Date().toLocaleTimeString('zh-CN'), stage: 'error', message: '❌ 部署失败: ' + (data.error || '未知错误') }];
        toast.error(`部署失败: ${data.error || '未知错误'}`, 6000);
        refreshLogs();
      }
    });
    // TODO(tauri-events): const cleanupDataChanged = getTauriAPI().onDataChanged?.(({ type }) => {
    //   if (type === 'cicd') loadConfigs();
    // });
    // if (cleanupDataChanged) _cleanupDataChanged = cleanupDataChanged;
  } catch (error) {
    handleError(error, { context: '加载部署面板' });
  }
});

onUnmounted(() => {
  cleanupDeployProgress?.();
  cleanupDeployNotification?.();
  _cleanupDataChanged?.();
});

async function loadConfigData(configId: string) {
  try {
    console.log("[loadConfigData] called")
    config.value = await getTauriAPI().getCicdConfigById(configId) as CicdConfigEntry | null;
    if (config.value && config.value.projectId) {
      project.value = projects.value.find((p) => p.id === config.value.projectId) || null;
      const rawResult = await getTauriAPI().getDeployLogs(config.value.id) as any;
      const rawLogs = (Array.isArray(rawResult) ? rawResult : rawResult?.data || []) as DeployLog[];
      // Enrich logs with project name and per-config info
      const configMap = new Map<string, CicdConfigEntry>()
      for (const c of configs.value) configMap.set(c.id, c)
      logs.value = rawLogs.map(log => {
        const logConfig = configMap.get(log.configId)
        return {
          ...log,
          projectName: project.value?.name || '',
          projectCategory: project.value?.category || '',
          configName: logConfig ? String(logConfig.name || '') : String(config.value?.name || ''),
          configGroupName: logConfig ? String(logConfig.groupName || '') : '',
          deployBranch: logConfig ? String(logConfig.deployBranch || '') : String(log.deployBranch || config.value?.deployBranch || ''),
        }
      });
    }
  } catch (error) {
    handleError(error, { context: '加载配置数据' });
  }
}

async function loadConfigs() {
  try {
    console.log("[loadConfigs] called")
    const [allConfigs, allProjects, allServers, allSGroups] = await Promise.all([
      getTauriAPI().getCicdConfigs?.() as Promise<CicdConfigEntry[]> | undefined,
      getTauriAPI().getProjects?.() as Promise<Project[]> | undefined,
      getTauriAPI().getAllServers?.() as Promise<Server[]> | undefined,
      getTauriAPI().getServerGroups?.() as Promise<Array<{ id: string; name: string; color: string; parentId: string | null }>> | undefined,
    ]);
    configs.value = (allConfigs as CicdConfigEntry[]) || [];
    projects.value = (allProjects as Project[]) || [];
    servers.value = (allServers as Server[]) || [];
    serverGroups.value = (allSGroups as Array<{ id: string; name: string; color: string; parentId: string | null }>) || [];
  } catch (error) {
    handleError(error, { context: 'loadConfigs' });
  }
}

async function runPreflight() {
  if (!config.value) {
    toast.error('请先选择配置');
    return;
  }
  preflightResults.value = [];
  const result = await runPreflightCheck(config.value);
  preflightResults.value = result.results;
  return result.passed;
}

async function startDeploy() {
  if (!selectedConfigId.value) return;
  if (deploying.value) return;

  if (config.value?.requiresApproval) {
    const proceed = confirm(
      `⚠️ 审核确认\\n\\n配置「${config.value.name || getProjectName(config.value.projectId)}」已开启部署审核。\\n\\n请确认你已准备好部署到生产环境，是否继续？`
    );
    if (!proceed) return;
  }

  const preflightOk = await runPreflight();
  if (!preflightOk) {
    const proceed = confirm('预检未通过，是否继续部署？');
    if (!proceed) return;
  }

  resetDeployState();
  deploying.value = true;
  currentStep.value = '开始部署...';
  realtimeLogs.value = [{ time: new Date().toLocaleTimeString('zh-CN'), stage: 'deploy', message: '部署任务已启动' }];

  const deployLogIdHandler = (data: { deployLogId: string }) => { activeDeployLogId.value = data.deployLogId; };
  const cleanupLogId = await getTauriAPI().onDeployLogIdCreated?.(deployLogIdHandler);

  try {
    const confirmed = !!config.value?.requiresApproval;
    const result = await getTauriAPI().deploy(selectedConfigId.value, confirmed || undefined);
    cleanupLogId?.();

    if (!result.success) {
      deploying.value = false;
      currentStep.value = '部署失败: ' + (result.error || '配置异常');
      realtimeLogs.value = [...realtimeLogs.value, { time: new Date().toLocaleTimeString('zh-CN'), stage: 'error', message: '❌ 部署失败: ' + (result.error || '配置异常') }];
      if (result.requiresApproval) {
        toast.warning(result.message || '此配置需要审核确认', 5000);
      } else {
        toast.error('部署失败: ' + (result.error || '配置异常'), 6000);
      }
    }
  } catch (error) {
    deploying.value = false;
    currentStep.value = '部署失败: ' + error.message;
    realtimeLogs.value = [...realtimeLogs.value, { time: new Date().toLocaleTimeString('zh-CN'), stage: 'error', message: '❌ 部署异常: ' + error.message }];
    handleError(error, { context: '部署' });
  }
}

async function cancelDeploy() {
  if (!deploying.value || !activeDeployLogId.value) return;

  const confirmed = confirm('确定要取消当前部署吗？');
  if (!confirmed) return;

  try {
    const result = await getTauriAPI().cancelDeploy(activeDeployLogId.value);
    if (result.success) {
      deploying.value = false;
      deployCancelled.value = true;
      currentStep.value = '⏹️ 部署已取消';
      realtimeLogs.value = [...realtimeLogs.value, { time: new Date().toLocaleTimeString('zh-CN'), stage: 'info', message: '⏹️ 部署取消请求已发送' }];
      toast.info('部署取消请求已发送');
    }
  } catch (error) {
    handleError(error, { context: '取消部署' });
  }
}

function confirmRollback(log: DeployLog) {
  const confirmed = confirm(
    `确定要回滚到 ${formatDate(log.createdAt)} 的部署版本吗？\n\n此操作将把服务器恢复到该版本，当前部署将被备份。`
  );
  if (!confirmed) return;
  doRollback(log);
}

async function doRollback(log: DeployLog) {
  if (!config.value || !selectedConfigId.value) return;

  rollingBack.value = true;
  resetDeployState();
  currentStep.value = '开始回滚...';
  realtimeLogs.value = [{ time: new Date().toLocaleTimeString('zh-CN'), stage: 'rollback', message: '回滚任务已启动' }];

  try {
    const result = await getTauriAPI().rollback(config.value!.id, log.id) as { success: boolean; error?: string };

    if (result.success) {
      progress.value = 100;
      currentStep.value = '🔄 回滚成功！';
      realtimeLogs.value = [...realtimeLogs.value, { time: new Date().toLocaleTimeString('zh-CN'), stage: 'rollback', message: '✅ 回滚成功完成' }];
      toast.success('回滚成功！');
    } else {
      currentStep.value = '回滚失败: ' + (result.error || '未知错误');
      realtimeLogs.value = [...realtimeLogs.value, { time: new Date().toLocaleTimeString('zh-CN'), stage: 'error', message: '❌ 回滚失败: ' + (result.error || '未知错误') }];
      toast.error('回滚失败: ' + (result.error || '未知错误'), 6000);
    }

    await refreshLogs();
  } catch (error) {
    currentStep.value = '回滚异常: ' + error.message;
    realtimeLogs.value = [...realtimeLogs.value, { time: new Date().toLocaleTimeString('zh-CN'), stage: 'error', message: '❌ 回滚异常: ' + error.message }];
    handleError(error, { context: '回滚' });
  }

  rollingBack.value = false;
}

async function rollbackDeploy(log: DeployLog) {
  // Check if config requires approval
  if (config.value?.requiresApproval) {
    const proceed = confirm(
      `⚠️ 审核确认\n\n配置「${config.value.name || getProjectName(config.value.projectId)}」已开启部署审核。\n\n请确认你要回滚到 ${formatDate(log.createdAt)} 的版本，是否继续？`
    );
    if (!proceed) return;
  } else {
    const confirmed = confirm(
      `确定要回滚到 ${formatDate(log.createdAt)} 的部署版本吗？\n\n此操作将把服务器恢复到该版本。`
    );
    if (!confirmed) return;
  }

  rollingBackId.value = log.id;
  try {
    const result = await getTauriAPI().rollback(config.value!.id, log.id) as { success: boolean; error?: string; requiresApproval?: boolean; message?: string };
    if (result.requiresApproval) {
      toast.warning(result.message || '此配置需要审核确认，CLI 不支持回滚', 5000);
      return;
    }
    if (result.success) {
      toast.success('回滚成功！');
    } else {
      toast.error('回滚失败: ' + (result.error || '未知错误'), 6000);
    }
    await refreshLogs();
  } catch (error) {
    handleError(error, { context: '回滚部署' });
  }
  rollingBackId.value = null;
}

async function viewFullLog(log: DeployLog) {
  if (!log.logFilePath) return;
  loadingLogFile.value = true;
  fullLogContent.value = null;
  try {
    console.log("[viewFullLog] called")
    const content = await getTauriAPI().readLogFile(log.logFilePath!) as { success: boolean; content?: string; error?: string };
    if (content.success && content.content !== undefined) {
      fullLogContent.value = content.content;
    } else {
      toast.error('读取日志失败: ' + (content.error || '未知错误'));
    }
  } catch (error) {
    handleError(error, { context: '读取日志文件' });
  }
  loadingLogFile.value = false;
}

function closeFullLog() {
  fullLogContent.value = null;
}

async function refreshLogs() {
    console.log("[closeFullLog] called")
  if (config.value?.projectId) {
    console.log("[refreshLogs] called")
    const rawResult = await getTauriAPI().getDeployLogs(config.value.id) as any;
    const rawLogs = (Array.isArray(rawResult) ? rawResult : rawResult?.data || []) as DeployLog[];
    const proj = projects.value.find(p => p.id === config.value?.projectId);
    const configMap = new Map<string, CicdConfigEntry>()
    for (const c of configs.value) configMap.set(c.id, c)
    logs.value = rawLogs.map(log => {
      const logConfig = configMap.get(log.configId)
      return {
        ...log,
        projectName: proj?.name || '',
        projectCategory: proj?.category || '',
        configName: logConfig ? String(logConfig.name || '') : String(config.value?.name || ''),
        configGroupName: logConfig ? String(logConfig.groupName || '') : '',
        deployBranch: logConfig ? String(logConfig.deployBranch || '') : String(log.deployBranch || config.value?.deployBranch || ''),
      }
    });
  }
}

async function toggleLogDetails(logId: string) {
  if (expandedLog.value === logId) {
    expandedLog.value = null;
  } else {
    expandedLog.value = logId;
    if (!stepLogs.value[logId]) {
      stepLogs.value[logId] = (await getTauriAPI().getDeployStepLogs(logId, "")) as DeployStep[];
    }
  }
}

async function loadLogContent(log: DeployLog) {
  if (!log.logFilePath || loadingLogContent.value[log.id]) return;
  loadingLogContent.value[log.id] = true;
  try {
    console.log("[loadLogContent] called")
    const content = await getTauriAPI().readLogFile(log.logFilePath!) as { success: boolean; content?: string; error?: string };
    if (content.success && content.content !== undefined) {
      loadedLogContent.value[log.id] = content.content;
    } else {
      toast.error('读取日志失败: ' + (content.error || '未知错误'));
    }
  } catch (error) {
    handleError(error, { context: '读取日志文件' });
  }
  loadingLogContent.value[log.id] = false;
}

function formatDate(dateStr: string) {
  const date = new Date(dateStr);
  return date.toLocaleString('zh-CN');
}

function clearRealtimeLogs() {
  if (selectedConfigId.value) {
    realtimeLogs.value = [];
  }
}

function scrollToBottom() {
  nextTick(() => {
    if (logContainer.value) {
      logContainer.value.scrollTop = logContainer.value.scrollHeight;
    }
  });
}
</script>

<style scoped>
.deploy-panel {
  padding: 16px 20px;
  width: 100%;
  min-height: 100%;
}

.panel-header {
  margin-bottom: 16px;
}

.panel-header h2 {
  margin: 0 0 4px 0;
  font-size: 20px;
  font-weight: 700;
  color: oklch(var(--bc));
}

.panel-subtitle {
  margin: 0;
  font-size: 13px;
  color: oklch(var(--bc) / 0.6);
}

/* ============ Two-Column Layout ============ */
.deploy-layout {
  display: grid;
  grid-template-columns: 340px 1fr;
  gap: 16px;
  align-items: start;
  width: 100%;
}

.deploy-sidebar {
  display: flex;
  flex-direction: column;
  gap: 12px;
  position: sticky;
  top: 0;
}

.deploy-main {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 0;
}

/* ============ Card ============ */
.card {
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 10px;
  padding: 16px;
}

.card-title {
  font-size: 14px;
  font-weight: 600;
  color: oklch(var(--bc));
  margin-bottom: 12px;
}

.card-title-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

/* ============ Config Selector ============ */
.config-selector label {
  display: block;
  margin-bottom: 8px;
  font-weight: 600;
  color: oklch(var(--bc));
  font-size: 13px;
}

.config-selector .form-input {
  width: 100%;
  padding: 10px 12px;
  font-size: 13px;
  border-radius: 8px;
  border: 1px solid oklch(var(--bc) / 0.1);
  background: oklch(var(--b2));
  color: oklch(var(--bc));
}

/* ============ Config Tree Selector ============ */
.config-tree {
  max-height: 280px;
  overflow-y: auto;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  background: oklch(var(--b2));
}

.config-tree-group {
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
}

.config-tree-group:last-child {
  border-bottom: none;
}

.config-tree-group-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 10px;
  cursor: pointer;
  user-select: none;
  font-size: 12px;
  font-weight: 600;
  color: oklch(var(--bc));
  background: rgba(0, 0, 0, 0.04);
  transition: background 0.15s;
}

.config-tree-group-header:hover {
  background: rgba(0, 0, 0, 0.08);
}

.tree-chevron {
  flex-shrink: 0;
  transition: transform 0.2s;
  color: oklch(var(--bc) / 0.6);
}

.config-tree-group-body {
  /* v-show controls display:none, no animation needed */
}

.tree-chevron {
  transition: transform 0.15s ease;
}

.tree-chevron.expanded {
  transform: rotate(90deg);
}

.tree-group-name {
  flex: 1;
}

.tree-group-count {
  font-size: 11px;
  font-weight: 400;
  color: oklch(var(--bc) / 0.6);
  background: rgba(0, 0, 0, 0.06);
  padding: 1px 6px;
  border-radius: 8px;
}

.config-tree-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 10px 7px 28px;
  cursor: pointer;
  transition: background 0.12s;
  font-size: 12px;
}

.config-tree-item:hover {
  background: rgba(59, 130, 246, 0.06);
}

.config-tree-item.active {
  background: rgba(59, 130, 246, 0.12);
  color: oklch(var(--p));
}

.config-tree-item-name {
  flex: 1;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}

.config-tree-item-meta {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: oklch(var(--bc) / 0.6);
  flex-shrink: 0;
}

.meta-branch {
  font-family: 'SF Mono', 'Fira Code', monospace;
  font-size: 10px;
  background: rgba(0, 0, 0, 0.06);
  padding: 1px 5px;
  border-radius: 3px;
}

.meta-dot {
  opacity: 0.4;
}

.config-tree-item-check {
  font-size: 12px;
  color: oklch(var(--su));
  font-weight: 700;
  flex-shrink: 0;
}

.config-tree-item-badge {
  font-size: 11px;
  margin-left: 2px;
  flex-shrink: 0;
}

/* ============ Config Info ============ */
.config-info .info-grid {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.info-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 0;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
}

.info-item:last-child {
  border-bottom: none;
}

.info-label {
  font-size: 12px;
  font-weight: 500;
  color: oklch(var(--bc) / 0.6);
}

.info-value {
  font-size: 13px;
  color: oklch(var(--bc));
  font-weight: 500;
  text-align: right;
}

.info-value.code {
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
  background: oklch(var(--b2));
  padding: 2px 8px;
  border-radius: 4px;
  word-break: break-all;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.branch-badge {
  display: inline-flex;
  padding: 2px 8px;
  background: oklch(var(--p) / 0.1);
  color: oklch(var(--p));
  border-radius: 10px;
  font-size: 12px;
  font-weight: 600;
}

/* ============ Deploy Actions ============ */
.deploy-actions {
  display: flex;
  gap: 10px;
}

.deploy-actions .btn {
  flex: 1;
  justify-content: center;
}

.btn {
  padding: 10px 16px;
  border: none;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background: oklch(var(--p));
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: oklch(var(--p) / 0.8);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px oklch(var(--p) / 0.1);
}

.btn-ghost {
  background: transparent;
  color: oklch(var(--bc));
  border: 1px solid oklch(var(--bc) / 0.1);
}

.btn-ghost:hover:not(:disabled) {
  background: oklch(var(--b2));
  border-color: oklch(var(--p));
  color: oklch(var(--p));
}

.btn-deploy {
  padding: 10px 16px;
  font-size: 14px;
}

.btn-requires-approval {
  background: linear-gradient(135deg, oklch(var(--wa)), #d97706) !important;
  border-color: oklch(var(--wa)) !important;
}

/* ============ Pre-flight ============ */
.preflight-results .card-title {
  margin-bottom: 8px;
}

.preflight-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 0;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
  font-size: 13px;
}

.preflight-item:last-child {
  border-bottom: none;
}

.preflight-item.passed .preflight-name {
  color: oklch(var(--su));
}

.preflight-item.failed .preflight-name {
  color: oklch(var(--er));
}

.preflight-message {
  margin-left: auto;
  color: oklch(var(--bc) / 0.6);
  font-size: 12px;
}

/* ============ Progress ============ */
.deploy-progress {
  padding: 14px 16px !important;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.progress-label {
  font-size: 13px;
  color: oklch(var(--bc));
  font-weight: 500;
}

.progress-pct {
  font-size: 13px;
  font-weight: 600;
  color: oklch(var(--p));
}

.progress-bar {
  height: 6px;
  background: oklch(var(--bc) / 0.1);
  border-radius: 3px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: oklch(var(--p));
  transition: width 0.3s ease;
}

.progress-fill.progress-cancelled {
  background: oklch(var(--bc) / 0.6);
}

.btn-cancel-deploy {
  padding: 4px 10px;
  background: oklch(var(--er));
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 500;
  transition: all 0.15s ease;
}

.btn-cancel-deploy:hover {
  opacity: 0.85;
}

/* ============ Real-time Log ============ */
.realtime-log {
  padding: 0 !important;
  overflow: hidden;
}

.realtime-log .card-title-row {
  padding: 12px 16px;
  background: oklch(var(--b2));
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
  margin-bottom: 0;
}

.realtime-log .card-title {
  margin-bottom: 0;
  font-size: 13px;
}

.btn-clear-logs {
  padding: 3px 8px;
  background: transparent;
  color: oklch(var(--bc) / 0.6);
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 4px;
  cursor: pointer;
  font-size: 11px;
  transition: all 0.15s ease;
}

.btn-clear-logs:hover {
  background: oklch(var(--b1));
  color: oklch(var(--bc));
}

.log-output {
  max-height: 500px;
  overflow-y: auto;
  padding: 12px 16px;
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
  line-height: 1.7;
}

.log-line {
  display: flex;
  gap: 8px;
  padding: 1px 0;
}

.log-time {
  color: oklch(var(--bc) / 0.6);
  flex-shrink: 0;
  min-width: 75px;
}

.log-stage {
  flex-shrink: 0;
  min-width: 55px;
  color: oklch(var(--p));
}

.log-git .log-stage { color: oklch(var(--su)); }
.log-maven .log-stage { color: oklch(var(--wa)); }
.log-ssh .log-stage { color: oklch(var(--p)); }
.log-restart .log-stage { color: #8b5cf6; }
.log-rollback .log-stage { color: oklch(var(--wa)); }
.log-error .log-stage, .log-error .log-msg { color: oklch(var(--er)); }
.log-collect .log-stage { color: #f97316; }

.log-msg {
  color: oklch(var(--bc));
  word-break: break-all;
}

.log-spinner {
  opacity: 0.7;
}

/* ============ Deploy History ============ */
.deploy-history .card-title-row {
  margin-bottom: 0;
}

.history-count {
  font-size: 12px;
  color: oklch(var(--bc) / 0.6);
}

.logs-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.log-item {
  padding: 12px 14px;
  background: oklch(var(--b2));
  border-radius: 8px;
  border-left: 3px solid transparent;
}

.log-item.success {
  border-left-color: oklch(var(--su));
}

.log-item.failed {
  border-left-color: oklch(var(--er));
}

.log-item.running {
  border-left-color: oklch(var(--p));
}

.log-item.cancelled {
  border-left-color: oklch(var(--bc) / 0.6);
}

.log-item.rolled_back {
  border-left-color: #f59e0b;
}

.log-header {
  display: flex;
  gap: 12px;
  align-items: center;
}

.log-status {
  font-size: 16px;
}

.log-time {
  color: oklch(var(--bc) / 0.6);
  font-size: 12px;
}

.log-trigger {
  color: oklch(var(--bc) / 0.6);
  font-size: 12px;
}

.btn-rollback {
  margin-left: auto;
  padding: 3px 10px;
  background: oklch(var(--p));
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 11px;
  font-weight: 500;
  transition: all 0.15s ease;
}

.btn-rollback:hover:not(:disabled) {
  background: oklch(var(--p) / 0.8);
}

.btn-rollback:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Full log file viewer */
.view-full-log-row {
  margin-top: 8px;
  display: flex;
  justify-content: flex-end;
}

.btn-view-full-log {
  padding: 5px 14px;
  background: oklch(var(--bc) / 0.1);
  color: oklch(var(--bc));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-view-full-log:hover:not(:disabled) {
  background: oklch(var(--p));
  color: #fff;
}

.btn-view-full-log:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.full-log-viewer {
  margin-bottom: 10px;
}

.full-log-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 10px;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-bottom: none;
  border-radius: 4px 4px 0 0;
  font-size: 13px;
  font-weight: 600;
}

.btn-close-log {
  padding: 2px 8px;
  background: transparent;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 3px;
  color: oklch(var(--bc) / 0.6);
  font-size: 12px;
  cursor: pointer;
}

.btn-close-log:hover {
  background: oklch(var(--er));
  color: #fff;
  border-color: oklch(var(--er));
}

.full-log-content {
  margin: 0;
  padding: 10px;
  max-height: 500px;
  overflow: auto;
  background: #1e1e1e;
  color: #d4d4d4;
  font-size: 12px;
  line-height: 1.5;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 0 0 4px 4px;
  white-space: pre-wrap;
  word-break: break-all;
}

.log-config-name {
  color: oklch(var(--p));
  font-size: 13px;
  font-weight: 700;
}

.log-config-group {
  font-size: 10px;
  font-weight: 600;
  color: oklch(var(--bc) / 0.6);
  background: oklch(var(--bc) / 0.1);
  padding: 1px 6px;
  border-radius: 3px;
  white-space: nowrap;
}

.log-project-name {
  color: oklch(var(--bc) / 0.6);
  font-size: 11px;
  opacity: 0.7;
}

.log-branch {
  color: oklch(var(--bc) / 0.6);
  font-size: 11px;
  background: oklch(var(--bc) / 0.1);
  padding: 1px 6px;
  border-radius: 3px;
}

.log-details {
  margin-top: 8px;
  padding: 8px 10px;
  background: oklch(var(--bc) / 0.1);
  border-radius: 4px;
}

.error-message {
  color: oklch(var(--er));
  font-weight: 500;
  font-size: 13px;
  margin: 0;
}

.cancelled-message {
  color: oklch(var(--bc) / 0.6);
  font-style: italic;
  font-size: 13px;
  margin: 0;
}

/* Step logs */
.step-logs {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.step-item {
  display: flex;
  flex-direction: column;
  padding: 8px 10px;
  background: oklch(var(--b2));
  border-radius: 6px;
  border-left: 3px solid transparent;
  font-size: 12px;
}

.step-item.success { border-left-color: oklch(var(--su)); }
.step-item.failed { border-left-color: oklch(var(--er)); }
.step-item.running { border-left-color: oklch(var(--p)); }
.step-item.pending { border-left-color: oklch(var(--bc) / 0.6); }

.step-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
}

.step-name {
  min-width: 120px;
  font-weight: 600;
  color: oklch(var(--bc));
}

.step-status-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 10px;
  font-weight: 500;
  text-transform: capitalize;
}

.step-status-badge.success { background: rgba(34, 197, 94, 0.15); color: oklch(var(--su)); }
.step-status-badge.failed { background: rgba(239, 68, 68, 0.15); color: oklch(var(--er)); }
.step-status-badge.running { background: rgba(59, 130, 246, 0.15); color: oklch(var(--p)); }
.step-status-badge.pending { background: rgba(107, 114, 128, 0.15); color: oklch(var(--bc) / 0.6); }

.step-meta {
  font-size: 11px;
  color: oklch(var(--bc) / 0.6);
  margin-bottom: 4px;
}

.step-output,
.step-item pre {
  margin-top: 4px;
  padding: 8px;
  background: oklch(var(--b1));
  border-radius: 4px;
  overflow-x: auto;
  font-size: 11px;
  max-height: 300px;
  white-space: pre-wrap;
  word-break: break-all;
  color: oklch(var(--bc));
}

.step-error {
  margin-top: 4px;
  color: oklch(var(--er));
  font-size: 12px;
  font-weight: 500;
}

/* Raw log output fallback */
.raw-log-output {
  margin-top: 4px;
}

.raw-log-header {
  font-size: 13px;
  font-weight: 600;
  color: oklch(var(--bc));
  margin-bottom: 6px;
}

/* No details available */
.no-details {
  text-align: center;
  padding: 20px;
  color: oklch(var(--bc) / 0.6);
}

.no-details p {
  margin: 4px 0;
  font-size: 13px;
}

.no-details .hint {
  font-size: 11px;
  opacity: 0.7;
  font-style: italic;
}

.step-status {
  color: oklch(var(--bc) / 0.6);
}

.btn-toggle {
  color: oklch(var(--bc) / 0.6);
}

.btn-toggle {
  margin-top: 8px;
  padding: 4px 10px;
  background: transparent;
  color: oklch(var(--bc) / 0.6);
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}

.btn-toggle:hover {
  border-color: oklch(var(--p));
  color: oklch(var(--p));
}

.empty-logs {
  text-align: center;
  padding: 20px;
  color: oklch(var(--bc) / 0.6);
  font-size: 13px;
}

/* ============ Empty State ============ */
.empty-state {
  padding: 60px 32px;
  text-align: center;
  background: oklch(var(--b1));
  border-radius: 12px;
  border: 1px solid oklch(var(--bc) / 0.1);
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.empty-state h3 {
  margin: 0 0 8px 0;
  font-size: 18px;
  color: oklch(var(--bc));
}

.empty-state p {
  margin: 0 0 20px 0;
  font-size: 14px;
  color: oklch(var(--bc) / 0.6);
}

/* ============ Scrollbar ============ */
.log-output::-webkit-scrollbar {
  width: 6px;
}

.log-output::-webkit-scrollbar-track {
  background: transparent;
}

.log-output::-webkit-scrollbar-thumb {
  background: oklch(var(--bc) / 0.1);
  border-radius: 3px;
}

.log-output::-webkit-scrollbar-thumb:hover {
  background: oklch(var(--bc) / 0.6);
}
</style>
