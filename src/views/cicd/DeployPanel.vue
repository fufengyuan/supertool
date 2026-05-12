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
                      {{ cfg.name || getGitRepoName(cfg.gitRepoId) }}
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
        <template v-if="config && selectedGitRepoObj">
          <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
            <div class="text-sm font-semibold text-base-content mb-3">配置详情</div>
            <div class="flex flex-col gap-2.5">
              <div class="flex justify-between items-center py-1.5 border-b border-base-content/10 last:border-b-0" v-if="config.name">
                <span class="text-xs font-medium text-base-content/60">名称</span>
                <span class="text-sm text-base-content font-medium text-right">{{ config.name }}</span>
              </div>
              <div class="flex justify-between items-center py-1.5 border-b border-base-content/10 last:border-b-0">
                <span class="text-xs font-medium text-base-content/60">仓库</span>
                <span class="text-sm text-base-content font-medium text-right">{{ selectedGitRepoObj?.name || '-' }}</span>
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
            <button @click="startDeploy" :disabled="deploying || !selectedConfigId || loadingConfig || !config" class="btn flex-1 justify-center"
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
            <span class="text-xs text-base-content/60">{{ combinedLogs.length }} 条记录</span>
          </div>

          <div class="flex flex-col gap-2 mt-3">
            <div v-for="log in combinedLogs" :key="log.id" class="px-3.5 py-3 bg-base-200 rounded-lg border-l-4 border-transparent"
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

              <!-- 正在进行的部署显示进度 -->
              <div class="mt-2 px-2.5 py-2 bg-primary/10 rounded" v-if="log.status === 'running'">
                <div class="flex justify-between items-center mb-1.5">
                  <span class="text-sm font-medium text-primary">{{ log.currentStep || '部署中...' }}</span>
                  <span class="text-xs font-semibold text-primary">{{ Math.round(log.progress || 0) }}%</span>
                </div>
                <div class="h-1.5 bg-base-content/10 rounded-full overflow-hidden">
                  <div class="h-full bg-primary transition-all duration-300" :style="{ width: (log.progress || 0) + '%' }"></div>
                </div>
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

            <div v-if="combinedLogs.length === 0" class="text-center p-5 text-base-content/60 text-sm">暂无部署记录</div>
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
import { useSharedCicdData } from '../../composables/useSharedCicdData';
import type { CicdConfigEntry, ServerGroupEntry } from '../../composables/useSharedCicdData';
import SvgIcon from '../../components/ui/SvgIcon.vue';
import type { Project } from '../../types';

interface DeployLog {
  id: string;
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

const shared = useSharedCicdData();
const { configs, projects, servers, serverGroups, gitRepos } = shared;
const selectedConfigId = ref('');
const config = ref<CicdConfigEntry | null>(null);
const loadingConfig = ref(false);
const project = ref<Project | null>(null);
const logs = ref<DeployLog[]>([]);
const stepLogs = ref<Record<string, DeployStep[]>>({});
const expandedLog = ref<string | null>(null);
const loadedLogContent = ref<Record<string, string>>({});
const loadingLogContent = ref<Record<string, boolean>>({});

// Tree selector state
const expandedDeployGroups = ref<Set<string>>(new Set());

// Deploy state — Map 结构支持多配置并行部署
interface DeployState {
  deploying: boolean;
  deployCancelled: boolean;
  progress: number;
  currentStep: string;
  realtimeLogs: { time: string; stage: string; message: string }[];
  deployLogId: string | null;
  lastLoggedProgress: number;
  startTime: number; // 用于排序
}

const deployStates = ref<Map<string, DeployState>>(new Map());

// 获取当前选中配置的部署状态
const currentDeployState = computed(() => {
  if (!selectedConfigId.value) return null;
  return deployStates.value.get(selectedConfigId.value) || null;
});

// 便捷访问当前状态的 computed
const deploying = computed(() => currentDeployState.value?.deploying ?? false);
const deployCancelled = computed(() => currentDeployState.value?.deployCancelled ?? false);
const progress = computed(() => currentDeployState.value?.progress ?? 0);
const currentStep = computed(() => currentDeployState.value?.currentStep ?? '');
const realtimeLogs = computed(() => currentDeployState.value?.realtimeLogs ?? []);
const activeDeployLogId = computed(() => currentDeployState.value?.deployLogId ?? null);
const lastLoggedProgress = computed(() => currentDeployState.value?.lastLoggedProgress ?? -1);

// 所有正在部署的配置列表（用于历史记录显示）
const allRunningDeploys = computed(() => {
  const running: DeployLog[] = [];
  for (const [configId, state] of deployStates.value) {
    if (state.deploying) {
      const cfg = configs.value.find(c => c.id === configId);
      running.push({
        id: state.deployLogId || `running-${configId}`,
        configId,
        status: 'running',
        createdAt: new Date(state.startTime).toISOString(),
        triggeredBy: 'user',
        currentStep: state.currentStep,
        progress: state.progress,
        configName: cfg?.name || '',
        configGroupName: cfg?.groupName || '',
      });
    }
  }
  // 按开始时间排序，最新的在前
  return running.sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime());
});

// 合并正在进行的部署和已完成的日志（用于历史记录显示）
const combinedLogs = computed(() => {
  // 正在进行的部署显示在最前面
  const running = allRunningDeploys.value;
  // 已完成的日志
  const completed = logs.value;
  // 合并，running 在前
  return [...running, ...completed];
});

const rollingBack = ref(false);
const rollingBackId = ref<string | null>(null);
const preflightResults = ref<{ name: string; passed: boolean; message: string }[]>([]);
const logContainer = ref(null);

function initDeployState(configId: string): DeployState {
  return {
    deploying: false,
    deployCancelled: false,
    progress: 0,
    currentStep: '',
    realtimeLogs: [],
    deployLogId: null,
    lastLoggedProgress: -1,
    startTime: Date.now(),
  };
}

function resetDeployState(configId: string) {
  deployStates.value.set(configId, initDeployState(configId));
}

function updateDeployState(configId: string, updates: Partial<DeployState>) {
  const existing = deployStates.value.get(configId) || initDeployState(configId);
  deployStates.value.set(configId, { ...existing, ...updates });
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

function getProjectName(_projectId: string) {
  return config.value ? getGitRepoName(config.value.gitRepoId) : 'Project ?';
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
  config.value = null;
  loadingConfig.value = true;
  // 切换配置时不清空其他配置的部署状态，只切换显示
  await loadConfigData(cfg.id);
  loadingConfig.value = false;
}

function getServersInfo(cfg: CicdConfigEntry | null): string {
  const names = getServerNames(cfg);
  return names.length > 0 ? names.join(', ') : '未配置';
}

function goToConfig() {
  window.dispatchEvent(new CustomEvent('switch-cicd-tab', { detail: 'config' }));
}

const progressHandler = (data: { progress?: number; message?: string; stage?: string; status?: string; configId?: string; deployLogId?: string }) => {
  // 从事件中获取 configId，如果没有则使用当前选中的配置
  const cfgId = data.configId || selectedConfigId.value;
  if (!cfgId) return;

  const state = deployStates.value.get(cfgId);
  if (!state || !state.deploying) return; // 只处理正在部署的状态

  const pct = data.progress || 0;
  const isUploadProgress = data.stage === 'ssh' && data.status === 'uploading' && pct > 0;
  const shouldThrottle = isUploadProgress && (pct - state.lastLoggedProgress < 5) && pct < 100;
  if (!shouldThrottle) {
    if (isUploadProgress) state.lastLoggedProgress = pct;
    const now = new Date().toLocaleTimeString('zh-CN');
    state.realtimeLogs = [...state.realtimeLogs, { time: now, stage: data.stage || 'info', message: data.message || '' }];
    scrollToBottom();
  }
  state.progress = pct;
  state.currentStep = data.message || state.currentStep;
  
  // 如果有 deployLogId，更新它
  if (data.deployLogId && !state.deployLogId) {
    state.deployLogId = data.deployLogId;
  }
};

let cleanupDeployProgress: (() => void) | undefined;
let cleanupDeployNotification: (() => void) | undefined;
let _cleanupDataChanged: (() => void) | undefined;

// ─── Loading state for non-blocking render ───
const initialLoading = ref(true);

onMounted(() => {
  // 异步加载，不阻塞渲染
  (async () => {
    try {
      // 共享数据由模块级单例缓存，已加载则直接跳过
      await shared.load();

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
        loadingConfig.value = true;
        loadConfigData(selectedConfigId.value).then(() => { loadingConfig.value = false; }).catch(() => { loadingConfig.value = false; });
      }

      // Enable deploy progress events from Tauri backend
      cleanupDeployProgress = await getTauriAPI().onDeployProgress?.(progressHandler);
      cleanupDeployNotification = await getTauriAPI().onDeployNotification?.((data) => {
      const cfgId = (data as any).configId;
      if (!cfgId) return;
      
      const state = deployStates.value.get(cfgId);
      if (!state) return;
      
      if (data.success) {
        state.deploying = false;
        state.progress = 100;
        state.currentStep = '部署成功！';
        state.realtimeLogs = [...state.realtimeLogs, { time: new Date().toLocaleTimeString('zh-CN'), stage: 'deploy', message: '✅ 部署成功完成' }];
        toast.success(`部署完成`);
        refreshLogs();
      } else if (data.cancelled) {
        state.deploying = false;
        state.deployCancelled = true;
        state.currentStep = '⏹️ 部署已取消';
        toast.info('部署已取消');
      } else {
        state.deploying = false;
        state.currentStep = '部署失败: ' + (data.error || '未知错误');
        state.realtimeLogs = [...state.realtimeLogs, { time: new Date().toLocaleTimeString('zh-CN'), stage: 'error', message: '❌ 部署失败: ' + (data.error || '未知错误') }];
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
      project.value = null;
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
    await shared.refresh();
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

  const currentState = deployStates.value.get(selectedConfigId.value);
  if (currentState?.deploying) return; // 该配置已经在部署中

  if (config.value?.requiresApproval) {
    const proceed = confirm(
      `⚠️ 审核确认\\n\\\\n配置「${config.value.name || getGitRepoName(config.value.gitRepoId)}」已开启部署审核。\\\\n\\\\n请确认你已准备好部署到生产环境，是否继续？`
    );
    if (!proceed) return;
  }

  const preflightOk = await runPreflight();
  if (!preflightOk) {
    const proceed = confirm('预检未通过，是否继续部署？');
    if (!proceed) return;
  }

  // 初始化当前配置的部署状态
  const newState = initDeployState(selectedConfigId.value);
  newState.deploying = true;
  newState.currentStep = '开始部署...';
  newState.realtimeLogs = [{ time: new Date().toLocaleTimeString('zh-CN'), stage: 'deploy', message: '部署任务已启动' }];
  newState.startTime = Date.now();
  deployStates.value.set(selectedConfigId.value, newState);

  const deployLogIdHandler = (data: { deployLogId: string }) => {
    const state = deployStates.value.get(selectedConfigId.value);
    if (state) state.deployLogId = data.deployLogId;
  };
  const cleanupLogId = await getTauriAPI().onDeployLogIdCreated?.(deployLogIdHandler);

  try {
    const confirmed = !!config.value?.requiresApproval;
    const result = await getTauriAPI().deploy(selectedConfigId.value, confirmed || undefined);
    cleanupLogId?.();

    if (!result.success) {
      const state = deployStates.value.get(selectedConfigId.value);
      if (state) {
        state.deploying = false;
        state.currentStep = '部署失败: ' + (result.error || '配置异常');
        state.realtimeLogs = [...state.realtimeLogs, { time: new Date().toLocaleTimeString('zh-CN'), stage: 'error', message: '❌ 部署失败: ' + (result.error || '配置异常') }];
      }
      if (result.requiresApproval) {
        toast.warning(result.message || '此配置需要审核确认', 5000);
      } else {
        toast.error('部署失败: ' + (result.error || '配置异常'), 6000);
      }
    }
  } catch (error) {
    const state = deployStates.value.get(selectedConfigId.value);
    if (state) {
      state.deploying = false;
      state.currentStep = '部署失败: ' + error.message;
      state.realtimeLogs = [...state.realtimeLogs, { time: new Date().toLocaleTimeString('zh-CN'), stage: 'error', message: '❌ 部署异常: ' + error.message }];
    }
    handleError(error, { context: '部署' });
  }
}

async function cancelDeploy() {
  if (!selectedConfigId.value) return;
  const state = deployStates.value.get(selectedConfigId.value);
  if (!state?.deploying || !state.deployLogId) return;

  const confirmed = confirm('确定要取消当前部署吗？');
  if (!confirmed) return;

  try {
    const result = await getTauriAPI().cancelDeploy(state.deployLogId);
    if (result.success) {
      state.deploying = false;
      state.deployCancelled = true;
      state.currentStep = '⏹️ 部署已取消';
      state.realtimeLogs = [...state.realtimeLogs, { time: new Date().toLocaleTimeString('zh-CN'), stage: 'info', message: '⏹️ 部署取消请求已发送' }];
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
  resetDeployState(selectedConfigId.value);
  const state = deployStates.value.get(selectedConfigId.value);
  if (state) {
    state.currentStep = '开始回滚...';
    state.realtimeLogs = [{ time: new Date().toLocaleTimeString('zh-CN'), stage: 'rollback', message: '回滚任务已启动' }];
  }

  try {
    const result = await getTauriAPI().rollback(config.value!.id, log.id) as { success: boolean; error?: string };

    if (result.success) {
      if (state) {
        state.progress = 100;
        state.currentStep = '🔄 回滚成功！';
        state.realtimeLogs = [...state.realtimeLogs, { time: new Date().toLocaleTimeString('zh-CN'), stage: 'rollback', message: '✅ 回滚成功完成' }];
      }
      toast.success('回滚成功！');
    } else {
      if (state) {
        state.currentStep = '回滚失败: ' + (result.error || '未知错误');
        state.realtimeLogs = [...state.realtimeLogs, { time: new Date().toLocaleTimeString('zh-CN'), stage: 'error', message: '❌ 回滚失败: ' + (result.error || '未知错误') }];
      }
      toast.error('回滚失败: ' + (result.error || '未知错误'), 6000);
    }

    await refreshLogs();
  } catch (error) {
    if (state) {
      state.currentStep = '回滚异常: ' + error.message;
      state.realtimeLogs = [...state.realtimeLogs, { time: new Date().toLocaleTimeString('zh-CN'), stage: 'error', message: '❌ 回滚异常: ' + error.message }];
    }
    handleError(error, { context: '回滚' });
  }

  rollingBack.value = false;
}

async function rollbackDeploy(log: DeployLog) {
  // Check if config requires approval
  if (config.value?.requiresApproval) {
    const proceed = confirm(
      `⚠️ 审核确认\\\\n\\\\n配置「${config.value.name || getGitRepoName(config.value.gitRepoId)}」已开启部署审核。\\\\n\\\\n请确认你要回滚到 ${formatDate(log.createdAt)} 的版本，是否继续？`
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
    console.log("[refreshLogs] called")
  if (config.value?.id) {
    const rawResult = await getTauriAPI().getDeployLogs(config.value.id) as any;
    const rawLogs = (Array.isArray(rawResult) ? rawResult : rawResult?.data || []) as DeployLog[];
    const configMap = new Map<string, CicdConfigEntry>()
    for (const c of configs.value) configMap.set(c.id, c)
    logs.value = rawLogs.map(log => {
      const logConfig = configMap.get(log.configId)
      return {
        ...log,
        projectName: config.value ? getGitRepoName(config.value.gitRepoId) : '',
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
    const state = deployStates.value.get(selectedConfigId.value);
    if (state) state.realtimeLogs = [];
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
