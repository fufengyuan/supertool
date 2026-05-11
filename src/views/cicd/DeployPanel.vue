<template>
  <div class="px-5 py-4 w-full min-h-full">
    <!-- Loading skeleton (non-blocking initial render) -->
    <div v-if="initialLoading" class="flex flex-col gap-4">
      <div class="skeleton h-8 w-48 rounded-lg"></div>
      <div class="skeleton h-4 w-64 rounded"></div>
      <div class="grid grid-cols-[340px_1fr] gap-4 mt-4">
        <div class="skeleton h-64 rounded-xl"></div>
        <div class="skeleton h-64 rounded-xl"></div>
      </div>
    </div>

    <!-- Header -->
    <template v-else>
    <div class="mb-4">
      <h2 class="text-xl font-bold m-0 mb-1 text-base-content"><SvgIcon name="rocket" size="16" class="inline-block align-text-bottom" /> 一键部署</h2>
      <p class="text-sm text-base-content/60 m-0">选择部署配置，快速将项目部署到目标服务器</p>
    </div>

    <!-- Main Layout: Left Config + Right Log/History -->
    <div class="grid grid-cols-[340px_1fr] gap-4 items-start w-full" v-if="configs.length > 0">
      <!-- Left: Config Selector + Info + Actions -->
      <div class="flex flex-col gap-3 sticky top-0">
        <!-- Config Selector -->
        <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
          <label class="block mb-2 font-semibold text-base-content text-sm">选择部署配置</label>
          <div class="max-h-72 overflow-y-auto border border-base-content/10 rounded-lg bg-base-200">
            <template v-for="[groupName, groupItems] in groupedDeployConfigs" :key="groupName">
              <div class="border-b border-base-content/10 last:border-b-0">
                <div class="flex items-center gap-1.5 px-2.5 py-2 cursor-pointer select-none text-xs font-semibold text-base-content bg-black/5 hover:bg-black/10" @click="toggleDeployGroup(groupName)">
                  <SvgIcon name="chevronDown" size="14" class="text-base-content/60 transition-transform duration-200" :class="expandedDeployGroups.has(groupName) ? 'rotate-90' : ''" />
                  <span class="flex-1">{{ groupName }}</span>
                  <span class="text-xs font-normal text-base-content/60 bg-black/5 px-1.5 py-0.5 rounded-full">{{ groupItems.length }}</span>
                </div>
                <div v-show="expandedDeployGroups.has(groupName)">
                  <div
                    v-for="cfg in groupItems"
                    :key="cfg.id"
                    class="flex items-center gap-2 px-2.5 py-1.5 pl-7 cursor-pointer transition-colors duration-100 text-xs hover:bg-primary/5"
                    :class="{ 'bg-primary/10 text-primary': selectedConfigId === cfg.id }"
                    @click="selectDeployConfig(cfg)"
                  >
                    <span class="flex-1 font-medium truncate min-w-0">
                      {{ cfg.name || getGitRepoName(cfg.gitRepoId) || getProjectName(cfg.projectId) }}
                    </span>
                    <span class="flex items-center gap-1 text-[11px] text-base-content/60 shrink-0">
                      <span class="font-mono text-[10px] bg-black/5 px-1.5 py-0.5 rounded-sm">{{ cfg.deployBranch || 'main' }}</span>
                      <span class="opacity-40">·</span>
                      <span>{{ getServerCount(cfg) }}台</span>
                    </span>
                    <span v-if="cfg.lastDeployedAt" class="text-xs text-success font-bold shrink-0" title="最近部署过">✓</span>
                    <span v-if="cfg.requiresApproval" class="text-xs ml-0.5 shrink-0" title="需要审核确认"><SvgIcon name="lock" size="12" class="inline-block align-text-bottom" /></span>
                  </div>
                </div>
              </div>
            </template>
          </div>
        </div>

        <!-- Config Details Card -->
        <template v-if="config && (project || selectedGitRepoObj)">
          <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
            <div class="text-sm font-semibold text-base-content mb-3">配置详情</div>
            <div class="flex flex-col gap-2.5">
              <div class="flex justify-between items-center py-1.5 border-b border-base-content/10 last:border-b-0" v-if="config.name">
                <span class="text-xs font-medium text-base-content/60">名称</span>
                <span class="text-sm text-base-content font-medium text-right">{{ config.name }}</span>
              </div>
              <div class="flex justify-between items-center py-1.5 border-b border-base-content/10 last:border-b-0">
                <span class="text-xs font-medium text-base-content/60">仓库</span>
                <span class="text-sm text-base-content font-medium text-right">{{ selectedGitRepoObj?.name || project?.name || '-' }}</span>
              </div>
              <div class="flex justify-between items-center py-1.5 border-b border-base-content/10 last:border-b-0">
                <span class="text-xs font-medium text-base-content/60">分支</span>
                <span class="inline-flex px-2 py-0.5 bg-primary/10 text-primary rounded-full text-xs font-semibold">{{ config.deployBranch || 'main' }}</span>
              </div>
              <div class="flex justify-between items-center py-1.5 border-b border-base-content/10 last:border-b-0">
                <span class="text-xs font-medium text-base-content/60">服务器</span>
                <span class="text-sm text-base-content font-medium text-right">{{ getServersInfo(config) }}</span>
              </div>
              <div class="flex justify-between items-center py-1.5 border-b border-base-content/10 last:border-b-0">
                <span class="text-xs font-medium text-base-content/60">构建工具</span>
                <span class="text-sm text-base-content font-medium text-right">{{ getBuildToolName(config.buildTool) }}</span>
              </div>
              <div class="flex justify-between items-center py-1.5 border-b border-base-content/10 last:border-b-0" v-if="config.deployPath">
                <span class="text-xs font-medium text-base-content/60">部署路径</span>
                <span class="text-sm font-medium text-right font-mono text-xs bg-base-200 px-2 py-0.5 rounded break-all max-w-[200px] truncate">{{ config.deployPath }}</span>
              </div>
              <!-- 重启脚本：仅 Maven 后端项目显示 -->
              <div class="flex justify-between items-center py-1.5 border-b border-base-content/10 last:border-b-0" v-if="config.restartScript && config.buildTool === 'maven'">
                <span class="text-xs font-medium text-base-content/60">重启脚本</span>
                <span class="text-sm font-medium text-right font-mono text-xs bg-base-200 px-2 py-0.5 rounded break-all max-w-[200px] truncate">{{ config.restartScript }}</span>
              </div>
            </div>
          </div>

          <!-- Deploy Actions -->
          <div class="bg-base-100 border border-base-content/10 rounded-xl p-4 flex gap-2.5">
            <button @click="runPreflight" :disabled="deploying" class="btn btn-ghost border border-base-content/10 flex-1 justify-center">
              <SvgIcon name="search" size="14" class="inline-block align-text-bottom" /> 部署预检
            </button>
            <button @click="startDeploy" :disabled="deploying || !selectedConfigId" class="btn flex-1 justify-center"
              :class="config?.requiresApproval ? 'bg-gradient-to-br from-warning to-amber-600 border-warning text-white hover:from-warning/90 hover:to-amber-600/90' : 'btn-primary'">
              <template v-if="deploying">部署中...</template>
              <template v-else-if="config?.requiresApproval">
                <SvgIcon name="lock" size="12" class="inline-block align-text-bottom" /> 审核部署
              </template>
              <template v-else>
                <SvgIcon name="rocket" size="12" class="inline-block align-text-bottom" /> 开始部署
              </template>
            </button>
          </div>

          <!-- Pre-flight Results -->
          <div v-if="preflightResults.length > 0" class="bg-base-100 border border-base-content/10 rounded-xl p-4">
            <div class="text-sm font-semibold text-base-content mb-2">预检结果</div>
            <div v-for="(r, i) in preflightResults" :key="i" class="flex items-center gap-2 py-1.5 border-b border-base-content/10 last:border-b-0 text-sm">
              <span><SvgIcon v-if="r.passed" name="check" size="14" class="text-success" /><SvgIcon v-else name="x" size="14" class="text-error" /></span>
              <span :class="r.passed ? 'text-success' : 'text-error'" class="font-medium">{{ r.name }}</span>
              <span class="ml-auto text-base-content/60 text-xs">{{ r.message }}</span>
            </div>
          </div>

          <!-- Progress -->
          <div class="bg-base-100 border border-base-content/10 rounded-xl px-4 py-3.5" v-if="deploying || progress > 0">
            <div class="flex justify-between items-center mb-2">
              <span class="text-sm text-base-content font-medium">{{ currentStep || '准备部署...' }}</span>
              <button v-if="deploying" @click="cancelDeploy" class="px-2.5 py-1 bg-error text-white border-0 rounded cursor-pointer text-xs font-medium hover:opacity-85"><SvgIcon name="stopSquare" size="14" class="inline-block align-text-bottom" /> 取消</button>
            </div>
            <div class="h-1.5 bg-base-content/10 rounded-full overflow-hidden">
              <div class="h-full bg-primary transition-all duration-300" :style="{ width: progress + '%' }" :class="{ 'bg-base-content/60': deployCancelled }"></div>
            </div>
            <span class="text-sm font-semibold text-primary">{{ Math.round(progress) }}%</span>
          </div>
        </template>
      </div>

      <!-- Right: Log + History -->
      <div class="flex flex-col gap-3 min-w-0">
        <!-- Real-time Log -->
        <div class="bg-base-100 border border-base-content/10 rounded-xl overflow-hidden" v-if="deploying || realtimeLogs.length > 0">
          <div class="flex justify-between items-center px-4 py-3 bg-base-200 border-b border-base-content/10">
            <span class="text-sm font-semibold text-base-content"><SvgIcon name="file" size="14" class="inline-block align-text-bottom" /> 实时日志</span>
            <button @click="clearRealtimeLogs" class="px-2 py-0.5 bg-transparent text-base-content/60 border border-base-content/10 rounded cursor-pointer text-xs hover:bg-base-100 hover:text-base-content transition-colors">清空</button>
          </div>
          <div ref="logContainer" class="max-h-[500px] overflow-y-auto px-4 py-3 font-mono text-xs leading-relaxed">
            <div v-for="(line, i) in realtimeLogs" :key="i" class="flex gap-2 py-0.5">
              <span class="text-base-content/60 shrink-0 min-w-[75px]">{{ line.time }}</span>
              <span class="shrink-0 min-w-[55px]" :class="{
                'text-success': line.stage === 'git',
                'text-warning': line.stage === 'maven' || line.stage === 'rollback',
                'text-primary': line.stage === 'ssh' || line.stage === 'deploy' || line.stage === 'info' || !line.stage,
                'text-purple-500': line.stage === 'restart',
                'text-error': line.stage === 'error',
                'text-orange-500': line.stage === 'collect'
              }">[{{ line.stage || 'info' }}]</span>
              <span class="text-base-content break-all" :class="{ 'text-error': line.stage === 'error' }">{{ line.message }}</span>
            </div>
            <div v-if="deploying" class="flex gap-2 py-0.5">
              <span class="text-base-content/60 shrink-0 min-w-[75px]">{{ currentTime }}</span>
              <span class="shrink-0 min-w-[55px] text-primary">[deploy]</span>
              <span class="text-base-content break-all opacity-70">⠋ 部署进行中...</span>
            </div>
          </div>
        </div>

        <!-- Deploy History -->
        <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
          <div class="flex justify-between items-center">
            <span class="text-sm font-semibold text-base-content">部署历史</span>
            <span class="text-xs text-base-content/60">{{ logs.length }} 条记录</span>
          </div>

          <div class="flex flex-col gap-2 mt-3">
            <div v-for="log in logs" :key="log.id" class="px-3.5 py-3 bg-base-200 rounded-lg border-l-4 border-transparent"
              :class="{
                'border-l-success': log.status === 'success',
                'border-l-error': log.status === 'failed',
                'border-l-primary': log.status === 'running',
                'border-l-base-content/60': log.status === 'cancelled',
                'border-l-amber-500': log.status === 'rolled_back'
              }">
              <div class="flex gap-3 items-center">
                <span class="text-base">
                  <SvgIcon v-if="log.status === 'success'" name="check" size="14" class="inline-block text-success" />
                  <SvgIcon v-else-if="log.status === 'failed'" name="x" size="14" class="inline-block text-error" />
                  <SvgIcon v-else-if="log.status === 'running'" name="refresh" size="14" class="inline-block animate-spin" />
                  <SvgIcon v-else-if="log.status === 'cancelled'" name="stopSquare" size="14" class="inline-block" />
                  <SvgIcon v-else-if="log.status === 'rolled_back'" name="undo" size="14" class="inline-block" />
                  <SvgIcon v-else name="clock" size="14" class="inline-block" />
                </span>
                <span class="text-primary text-sm font-bold" v-if="log.configName">{{ log.configName }}</span>
                <span class="text-[10px] font-semibold text-base-content/60 bg-base-content/10 px-1.5 py-0.5 rounded whitespace-nowrap" v-if="log.configGroupName">{{ log.configGroupName }}</span>
                <span class="text-xs text-base-content/60 opacity-70" v-if="log.projectName">({{ log.projectName }})</span>
                <span class="text-xs text-base-content/60">{{ formatDate(log.createdAt) }}</span>
                <span class="text-xs text-base-content/60">触发: {{ log.triggeredBy }}</span>
                <span class="text-xs text-base-content/60 bg-base-content/10 px-1.5 py-0.5 rounded inline-flex items-center gap-1" v-if="log.deployBranch"><SvgIcon name="gitBranch" size="12" /> {{ log.deployBranch }}</span>
                <button
                  v-if="log.status === 'success'"
                  @click="rollbackDeploy(log)"
                  :disabled="rollingBackId === log.id"
                  class="ml-auto px-2.5 py-0.5 bg-primary text-white border-0 rounded cursor-pointer text-xs font-medium transition-colors hover:bg-primary/80 disabled:opacity-50 disabled:cursor-not-allowed"
                  title="回滚到此版本"
                >
                  <template v-if="rollingBackId === log.id"><SvgIcon name="clock" size="14" class="inline-block align-text-bottom" /> 回滚中</template>
                  <template v-else><SvgIcon name="refresh" size="14" class="inline-block align-text-bottom" /> 回滚</template>
                </button>
              </div>

              <div class="mt-2 px-2.5 py-2 bg-base-content/10 rounded" v-if="log.status === 'failed'">
                <p class="text-error font-medium text-sm m-0">{{ log.errorMessage || '未知错误' }}</p>
              </div>

              <div class="mt-2 px-2.5 py-2 bg-base-content/10 rounded" v-if="log.status === 'cancelled'">
                <p class="text-base-content/60 italic text-sm m-0">部署已取消</p>
              </div>

              <div class="mt-2 px-2.5 py-2 bg-base-content/10 rounded" v-if="expandedLog === log.id">
                <!-- Full log file viewer -->
                <div v-if="fullLogContent !== null" class="mb-2.5">
                  <div class="flex justify-between items-center px-2.5 py-1.5 bg-base-100 border border-base-content/10 border-b-0 rounded-t font-semibold text-sm">
                    <span><SvgIcon name="file" size="14" class="inline-block align-text-bottom" /> 完整日志</span>
                    <button @click="closeFullLog" class="px-2 py-0.5 bg-transparent border border-base-content/10 rounded text-base-content/60 text-xs cursor-pointer hover:bg-error hover:text-white hover:border-error transition-colors inline-flex items-center gap-1"><SvgIcon name="x" size="12" /> 关闭</button>
                  </div>
                  <pre class="m-0 p-2.5 max-h-[500px] overflow-auto bg-[#1e1e1e] text-[#d4d4d4] text-xs leading-relaxed border border-base-content/10 rounded-b whitespace-pre-wrap break-all">{{ fullLogContent }}</pre>
                </div>

                <!-- Step logs -->
                <div v-if="!fullLogContent && stepLogs[log.id] && stepLogs[log.id].length > 0" class="flex flex-col gap-2">
                  <div
                    v-for="step in stepLogs[log.id]"
                    :key="step.id"
                    class="flex flex-col px-2.5 py-2 bg-base-200 rounded-lg border-l-4 text-xs"
                    :class="{
                      'border-l-success': step.status === 'success',
                      'border-l-error': step.status === 'failed',
                      'border-l-primary': step.status === 'running',
                      'border-l-base-content/60': step.status === 'pending'
                    }"
                  >
                    <div class="flex justify-between items-center mb-1">
                      <span class="min-w-[120px] font-semibold text-base-content">{{ step.stepName }}</span>
                      <span class="text-xs px-2 py-0.5 rounded-full font-medium capitalize"
                        :class="{
                          'bg-green-500/15 text-success': step.status === 'success',
                          'bg-red-500/15 text-error': step.status === 'failed',
                          'bg-blue-500/15 text-primary': step.status === 'running',
                          'bg-gray-500/15 text-base-content/60': step.status === 'pending'
                        }">
                        <SvgIcon v-if="step.status === 'success'" name="check" size="14" class="inline-block align-text-bottom text-success" />
                        <SvgIcon v-else-if="step.status === 'failed'" name="x" size="14" class="inline-block align-text-bottom text-error" />
                        <SvgIcon v-else-if="step.status === 'running'" name="clock" size="14" class="inline-block align-text-bottom" />
                        <SvgIcon v-else name="pause" size="14" class="inline-block align-text-bottom" />
                        {{ step.status }}
                      </span>
                    </div>
                    <div class="text-xs text-base-content/60 mb-1" v-if="step.startTime || step.endTime">
                      <span>{{ step.startTime ? formatDate(step.startTime) : '-' }} → {{ step.endTime ? formatDate(step.endTime) : '进行中' }}</span>
                    </div>
                    <pre v-if="step.output" class="mt-1 p-2 bg-base-100 rounded overflow-x-auto text-xs max-h-72 whitespace-pre-wrap break-all text-base-content">{{ step.output }}</pre>
                    <p v-if="step.errorMessage" class="mt-1 text-error text-xs font-medium">{{ step.errorMessage }}</p>
                  </div>
                </div>

                <!-- Raw log output (fallback when no step logs but logOutput exists) -->
                <div v-else-if="!fullLogContent && log.logOutput" class="mt-1">
                  <div class="text-sm font-semibold text-base-content mb-1.5"><SvgIcon name="file" size="14" class="inline-block align-text-bottom" /> 部署日志</div>
                  <pre class="mt-1 p-2 bg-base-100 rounded overflow-x-auto text-xs max-h-72 whitespace-pre-wrap break-all text-base-content">{{ log.logOutput }}</pre>
                </div>

                <!-- Auto-load log file when step logs are empty -->
                <div v-else-if="!fullLogContent && log.logFilePath" class="mt-1">
                  <div class="text-sm font-semibold text-base-content mb-1.5"><SvgIcon name="file" size="14" class="inline-block align-text-bottom" /> 部署日志</div>
                  <pre class="mt-1 p-2 bg-base-100 rounded overflow-x-auto text-xs max-h-72 whitespace-pre-wrap break-all text-base-content" v-if="loadedLogContent[log.id]">{{ loadedLogContent[log.id] }}</pre>
                  <pre class="mt-1 p-2 bg-base-100 rounded overflow-x-auto text-xs max-h-72 whitespace-pre-wrap break-all text-base-content" v-else-if="loadingLogContent[log.id]"><SvgIcon name="clock" size="14" class="inline-block align-text-bottom animate-spin" /> 读取日志中...</pre>
                  <button
                    v-else
                    @click="loadLogContent(log)"
                    class="px-3.5 py-1 bg-base-content/10 text-base-content border border-base-content/10 rounded text-xs cursor-pointer hover:bg-primary hover:text-white transition-colors"
                  ><SvgIcon name="file" size="12" class="inline-block align-text-bottom" /> 点击加载日志</button>
                </div>

                <!-- No details available -->
                <div v-else-if="!fullLogContent" class="text-center p-5 text-base-content/60">
                  <p class="m-1 text-sm"><SvgIcon name="mail" size="14" class="inline-block align-text-bottom" /> 暂无日志数据</p>
                </div>

                <!-- View full log button -->
                <div v-if="log.logFilePath && fullLogContent === null" class="mt-2 flex justify-end">
                  <button @click="viewFullLog(log)" :disabled="loadingLogFile" class="px-3.5 py-1 bg-base-content/10 text-base-content border border-base-content/10 rounded text-xs cursor-pointer hover:bg-primary hover:text-white disabled:opacity-50 disabled:cursor-not-allowed transition-colors">
                    <template v-if="loadingLogFile"><SvgIcon name="clock" size="14" class="inline-block align-text-bottom animate-spin" /> 加载中...</template>
                    <template v-else><SvgIcon name="file" size="14" class="inline-block align-text-bottom" /> 查看完整日志</template>
                  </button>
                </div>
              </div>

              <button @click="toggleLogDetails(log.id)" class="mt-2 px-2.5 py-1 bg-transparent text-base-content/60 border border-base-content/10 rounded cursor-pointer text-xs hover:border-primary hover:text-primary transition-colors">
                {{ expandedLog === log.id ? '收起' : '展开详情' }}
              </button>
            </div>

            <div v-if="logs.length === 0" class="text-center p-5 text-base-content/60 text-sm">暂无部署记录</div>
          </div>
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <div v-else class="p-16 text-center bg-base-100 rounded-xl border border-base-content/10">
      <div class="mb-4"><SvgIcon name="package" size="48" class="text-base-content/30" /></div>
      <h3 class="m-0 mb-2 text-lg text-base-content">暂无 CI/CD 配置</h3>
      <p class="m-0 mb-5 text-sm text-base-content/60">请先在 CI/CD 配置页面创建部署配置</p>
      <button @click="goToConfig" class="btn btn-primary">前往配置页面</button>
    </div>
    </template>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, computed, onMounted, onUnmounted, onBeforeUnmount, nextTick, watch } from 'vue';
import { getTauriAPI } from '../../utils/tauri-api'
import { useToast } from '../../composables/useToast';
import { useErrorHandler } from '../../composables/useErrorHandler';
import { useDeployPreflight } from '../../composables/useDeployPreflight';
import SvgIcon from '../../components/ui/SvgIcon.vue';
import type { Project, Server } from '../../types';

interface CicdConfigEntry {
  id: string;
  projectId: string;
  gitRepoId?: string;
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
const gitRepos = ref<any[]>([]);
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
const activeDeployConfigId = ref<string | null>(null);
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
  activeDeployConfigId.value = null;
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

function getGitRepoName(id?: string) {
  if (!id) return '';
  const repo = gitRepos.value.find((r: any) => r.id === id);
  return repo ? repo.name : '';
}

const selectedGitRepoObj = computed(() => {
  if (!config.value?.gitRepoId) return null;
  return gitRepos.value.find((r: any) => r.id === config.value!.gitRepoId) || null;
});

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
  const cfgId = data.configId || activeDeployConfigId.value;
  if (!cfgId || cfgId !== activeDeployConfigId.value) return;
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

// ─── Loading state for non-blocking render ───
const initialLoading = ref(true);

onMounted(() => {
    console.log("[components/cicd/DeployPanel.vue] mounted")
  // 异步加载，不阻塞渲染
  (async () => {
    try {
      const [allConfigs, allProjects, allServers, allSGroups, allGitRepos] = await Promise.all([
        getTauriAPI().getCicdConfigs?.() as Promise<CicdConfigEntry[]> | undefined,
        getTauriAPI().getProjects?.() as Promise<Project[]> | undefined,
        getTauriAPI().getAllServers?.() as Promise<Server[]> | undefined,
        getTauriAPI().getServerGroups?.() as Promise<Array<{ id: string; name: string; color: string; parentId: string | null }>> | undefined,
        getTauriAPI().getGitRepos?.() as Promise<any> | undefined,
      ]);
      configs.value = (allConfigs as CicdConfigEntry[]) || [];
      projects.value = (allProjects as Project[]) || [];
      servers.value = (allServers as Server[]) || [];
      serverGroups.value = (allSGroups as Array<{ id: string; name: string; color: string; parentId: string | null }>) || [];
      // Load git repos for gitRepoId resolution
      const repoResult = allGitRepos as any;
      gitRepos.value = repoResult?.success && repoResult?.data ? repoResult.data : [];

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
        // 非阻塞加载配置详情
        loadConfigData(selectedConfigId.value).catch(() => {});
      }

      // Enable deploy progress events from Tauri backend
      cleanupDeployProgress = await getTauriAPI().onDeployProgress?.(progressHandler);
      cleanupDeployNotification = await getTauriAPI().onDeployNotification?.((data) => {
      const cfgId = (data as any).configId;
      // 使用 activeDeployConfigId 匹配，这样即使用户切换了配置也能正确处理通知
      if (!cfgId || cfgId !== activeDeployConfigId.value) return;
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
      initialLoading.value = false; // 数据加载完成，显示 UI
    } catch (error) {
      handleError(error, { context: '加载部署面板' });
      initialLoading.value = false;
    }
  })();
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
    if (config.value) {
      if (config.value.projectId) {
        project.value = projects.value.find((p) => p.id === config.value.projectId) || null;
      } else {
        project.value = null;
      }
      const rawResult = await getTauriAPI().getDeployLogs(config.value.id) as any;
      const rawLogs = (Array.isArray(rawResult) ? rawResult : rawResult?.data || []) as DeployLog[];
      // Enrich logs with project name and per-config info
      const configMap = new Map<string, CicdConfigEntry>()
      for (const c of configs.value) configMap.set(c.id, c)
      logs.value = rawLogs.map(log => {
        const logConfig = configMap.get(log.configId)
        const repoName = logConfig?.gitRepoId ? getGitRepoName(logConfig.gitRepoId) : '';
        return {
          ...log,
          projectName: repoName || project.value?.name || '',
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
  activeDeployConfigId.value = selectedConfigId.value;
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
    `确定要回滚到 ${formatDate(log.createdAt)} 的部署版本吗？\\n\\n此操作将把服务器恢复到该版本，当前部署将被备份。`
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
      `⚠️ 审核确认\\n\\n配置「${config.value.name || getProjectName(config.value.projectId)}」已开启部署审核。\\n\\n请确认你要回滚到 ${formatDate(log.createdAt)} 的版本，是否继续？`
    );
    if (!proceed) return;
  } else {
    const confirmed = confirm(
      `确定要回滚到 ${formatDate(log.createdAt)} 的部署版本吗？\\n\\n此操作将把服务器恢复到该版本。`
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
