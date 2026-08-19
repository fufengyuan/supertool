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
        <template v-if="config">
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
                <div class="flex items-center gap-1">
                  <select v-model="selectedBranch"
                    class="select select-bordered select-xs w-28 bg-base-200 text-xs cursor-pointer"
                    :disabled="!selectedGitRepoObj || loadingBranches"
                    @click.stop>
                    <option v-for="b in branches" :key="b" :value="b">{{ b }}</option>
                  </select>
                  <button v-if="selectedGitRepoObj" @click.stop="loadBranchesForConfig(config!)" class="btn btn-xs btn-ghost px-1" :class="{ 'loading': loadingBranches }" title="刷新分支列表">⟳</button>
                </div>
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
            <button @click="runPreflight" :disabled="deploying || preflightRunning" class="btn btn-ghost border border-base-content/10 flex-1 justify-center">
              <span v-if="preflightRunning" class="loading loading-spinner loading-xs" />
              <SvgIcon v-else name="search" size="14" class="inline-block align-text-bottom" /> {{ preflightRunning ? '预检中...' : '部署预检' }}
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

      <!-- Approval Confirmation Modal -->
      <div v-if="showApprovalDialog" class="fixed inset-0 z-[10000] flex items-center justify-center bg-black/60" @click.self="cancelApproval">
        <div class="bg-base-200 border border-base-content/10 rounded-xl p-6 w-[420px] max-w-[90vw] shadow-xl animate-[slideUp_0.2s_ease]">
          <div class="flex items-center gap-3 mb-4">
            <div class="w-10 h-10 rounded-full bg-warning/20 flex items-center justify-center text-warning">
              <SvgIcon name="lock" size="20" />
            </div>
            <div>
              <h4 class="m-0 text-lg font-bold text-base-content">审核确认</h4>
              <p class="m-0 text-sm text-base-content/60 mt-0.5">此配置已开启部署审核</p>
            </div>
          </div>
          <p class="text-sm text-base-content/80 mb-5 leading-relaxed">
            配置「<strong>{{ config?.name || getGitRepoName(config?.gitRepoId) }}</strong>」开启了审核模式。
            <br />请确认你已准备好部署，是否继续？
          </p>
          <div class="flex justify-end gap-2">
            <button @click="cancelApproval" class="btn btn-ghost">取消</button>
            <button @click="confirmApproval" class="btn bg-gradient-to-br from-warning to-amber-600 border-warning text-white hover:from-warning/90 hover:to-amber-600/90">
              <SvgIcon name="rocket" size="14" class="inline-block align-text-bottom" /> 确认部署
            </button>
          </div>
        </div>
      </div>

      <!-- Right: Log + History -->
      <div class="flex flex-col gap-3 min-w-0">
        <!-- Real-time Log -->
        <div class="bg-base-100 border border-base-content/10 rounded-xl overflow-hidden relative" v-if="deploying || realtimeLogs.length > 0">
          <div class="flex justify-between items-center px-4 py-3 bg-base-200 border-b border-base-content/10">
            <span class="text-sm font-semibold text-base-content"><SvgIcon name="file" size="14" class="inline-block align-text-bottom" /> 实时日志</span>
            <button @click="clearRealtimeLogs" class="px-2 py-0.5 bg-transparent text-base-content/60 border border-base-content/10 rounded cursor-pointer text-xs hover:bg-base-100 hover:text-base-content transition-colors">清空</button>
          </div>
          <div ref="logContainer" @scroll="onRealtimeLogScroll" class="max-h-[500px] overflow-y-auto px-4 py-3 font-mono text-xs leading-relaxed">
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
          <button
            v-if="realtimeUserScrolledUp"
            @click="scrollToBottom(true)"
            class="btn btn-primary btn-sm rounded-full absolute bottom-2 right-2 z-10 shadow-lg hover:scale-105 transition-all"
            title="回到底部"
          >
            <SvgIcon name="arrowDown" size="14" /> 回到底部
          </button>
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
                <button
                  v-if="log.status === 'running'"
                  @click="cancelRunningDeploy(log)"
                  class="ml-auto px-2.5 py-0.5 bg-error text-white border-0 rounded cursor-pointer text-xs font-medium transition-colors hover:opacity-85"
                  title="取消部署"
                >
                  <SvgIcon name="stopSquare" size="14" class="inline-block align-text-bottom" /> 取消
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
                <!-- Step logs (结构化步骤日志，优先级最高) -->
                <div v-if="stepLogs[log.id] && stepLogs[log.id].length > 0" class="flex flex-col gap-2">
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

                <!-- 正在部署：显示实时日志 -->
                <div v-else-if="log.status === 'running' && runningRealtimeLogs(log)" class="mt-1">
                  <div class="flex justify-between items-center px-2.5 py-1.5 bg-base-100 border border-base-content/10 border-b-0 rounded-t font-semibold text-sm">
                    <span><SvgIcon name="file" size="14" class="inline-block align-text-bottom" /> 实时日志</span>
                    <span class="text-xs text-base-content/60">{{ runningRealtimeLogs(log)?.length }} 行</span>
                  </div>
                  <div class="m-0 p-2.5 max-h-[500px] overflow-y-auto overflow-x-auto bg-[#1e1e1e] text-[#d4d4d4] text-xs leading-relaxed border border-base-content/10 rounded-b whitespace-pre-wrap break-all font-mono">
                    <div v-for="(line, i) in runningRealtimeLogs(log)" :key="i" class="flex gap-2 py-0.5">
                      <span class="text-gray-500 shrink-0 min-w-[75px]">{{ line.time }}</span>
                      <span class="text-gray-500 shrink-0 min-w-[55px]">[{{ line.stage }}]</span>
                      <span class="text-gray-300 break-all">{{ line.message }}</span>
                    </div>
                  </div>
                </div>

                <!-- 日志文件内容（暗色主题展示，与完整日志效果一致） -->
                <div v-else-if="loadedLogContent[log.id]" class="mt-1">
                  <div class="flex justify-between items-center px-2.5 py-1.5 bg-base-100 border border-base-content/10 border-b-0 rounded-t font-semibold text-sm">
                    <span><SvgIcon name="file" size="14" class="inline-block align-text-bottom" /> 部署日志</span>
                    <span class="text-xs text-base-content/60">{{ loadedLogContent[log.id].split('\n').length }} 行</span>
                  </div>
                  <pre class="m-0 p-2.5 max-h-[500px] overflow-y-auto overflow-x-auto bg-[#1e1e1e] text-[#d4d4d4] text-xs leading-relaxed border border-base-content/10 rounded-b whitespace-pre-wrap break-all font-mono">{{ loadedLogContent[log.id] }}</pre>
                </div>

                <!-- 原始 logOutput（无日志文件时的后备展示） -->
                <div v-else-if="log.logOutput" class="mt-1">
                  <div class="flex justify-between items-center px-2.5 py-1.5 bg-base-100 border border-base-content/10 border-b-0 rounded-t font-semibold text-sm">
                    <span><SvgIcon name="file" size="14" class="inline-block align-text-bottom" /> 部署日志</span>
                  </div>
                  <pre class="m-0 p-2.5 max-h-[500px] overflow-y-auto overflow-x-auto bg-[#1e1e1e] text-[#d4d4d4] text-xs leading-relaxed border border-base-content/10 rounded-b whitespace-pre-wrap break-all font-mono">{{ log.logOutput }}</pre>
                </div>

                <!-- 正在加载日志文件 -->
                <div v-else-if="log.logFilePath" class="mt-1 text-center p-5 text-base-content/60">
                  <SvgIcon name="clock" size="14" class="inline-block align-text-bottom animate-spin" /> 读取日志中...
                </div>

                <!-- 没有任何数据 -->
                <div v-else class="text-center p-5 text-base-content/60">
                  <p class="m-1 text-sm"><SvgIcon name="mail" size="14" class="inline-block align-text-bottom" /> 暂无日志数据</p>
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

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue';
import { confirm } from '@tauri-apps/plugin-dialog';
import { getTauriAPI } from '../../utils/tauri-api'
import { useToast } from '../../composables/useToast';
import { useErrorHandler } from '../../composables/useErrorHandler';
import { useDeployPreflight } from '../../composables/useDeployPreflight';
import { useSharedCicdData } from '../../composables/useSharedCicdData';
import type { CicdConfigEntry } from '../../composables/useSharedCicdData';
import SvgIcon from '../../components/ui/SvgIcon.vue';

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
const { configs, servers, serverGroups, gitRepos, getGitRepoName } = shared;
const selectedConfigId = ref('');
const config = ref<CicdConfigEntry | null>(null);
const loadingConfig = ref(false);
const selectedBranch = ref('');
const branches = ref<string[]>([]);
const loadingBranches = ref(false);
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

// 缓存上限：防止长时间使用后内存无界增长
const MAX_REALTIME_LOGS = 500;    // 每条部署状态最多保留的实时日志条数
const MAX_LOADED_LOG_CONTENT = 3; // loadedLogContent 最多缓存的日志文件数（LRU）
const MAX_STEP_LOGS = 5;          // stepLogs 最多缓存的步骤日志数（LRU）
const DEPLOY_STATE_TTL = 30_000;  // 部署完成后保留状态 30s 再清理

// 获取当前选中配置的部署状态
const currentDeployState = computed(() => {
  if (!selectedConfigId.value) {return null;}
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

const rollingBackId = ref<string | null>(null);
const preflightResults = ref<{ name: string; passed: boolean; message: string }[]>([]);
const logContainer = ref<HTMLElement | null>(null);
// 实时日志智能吸底：用户上翻查看历史时暂停自动跟随，点"回到底部"恢复
const realtimeUserScrolledUp = ref(false);

function onRealtimeLogScroll() {
  const el = logContainer.value;
  if (!el) {return;}
  const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 50;
  realtimeUserScrolledUp.value = !atBottom;
}

// Approval confirmation modal
const showApprovalDialog = ref(false);
let approvalResolve: ((value: boolean) => void) | null = null;

function showApprovalConfirm(): Promise<boolean> {
  showApprovalDialog.value = true;
  return new Promise(resolve => {
    approvalResolve = resolve;
  });
}

function confirmApproval() {
  showApprovalDialog.value = false;
  approvalResolve?.(true);
  approvalResolve = null;
}

function cancelApproval() {
  showApprovalDialog.value = false;
  approvalResolve?.(false);
  approvalResolve = null;
}

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


const currentTime = computed(() => {
  return new Date().toLocaleTimeString('zh-CN');
});

// 按最后部署时间排序：最近部署的排在最前面
const sortedConfigs = computed(() => {
  return [...configs.value].sort((a, b) => {
    const aTime = ((a as CicdConfigEntry).lastDeployedAt as string) || ''
    const bTime = ((b as CicdConfigEntry).lastDeployedAt as string) || ''
    if (aTime && bTime) {return bTime.localeCompare(aTime)}
    if (aTime) {return -1}
    if (bTime) {return 1}
    return 0
  })
})

// Configs grouped by groupName (for tree selector)
const groupedDeployConfigs = computed(() => {
  const map = new Map<string, CicdConfigEntry[]>();
  const sorted = sortedConfigs.value;
  for (const cfg of sorted) {
    const group = (cfg as CicdConfigEntry).groupName || '未分组';
    if (!map.has(group)) {map.set(group, []);}
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

const selectedGitRepoObj = computed(() => {
  if (!config.value?.gitRepoId) {return null;}
  return gitRepos.value.find((r: any) => r.id === config.value!.gitRepoId) || null;
});

function getBuildToolName(tool: unknown) {
  const names: Record<string, string> = { maven: 'Maven', npm: 'npm', pnpm: 'pnpm', yarn: 'Yarn', gradle: 'Gradle' };
  return names[String(tool || '')] || '未设置';
}

// Get server names from config's servers JSON
function getServerNames(cfg: CicdConfigEntry | null): string[] {
  if (!cfg?.servers) {return [];}
  try {
    const parsed = JSON.parse(cfg.servers);
    if (!Array.isArray(parsed) || parsed.length === 0) {return [];}
    return parsed.map((s: any) => {
      const srv = servers.value.find(sv => sv.id === s.serverId);
      if (!srv) {return s.label || s.serverId;}
      // Find group name
      const group = srv.groupId ? serverGroups.value.find(g => g.id === srv.groupId) : null;
      const groupTag = group ? ` [${group.name}]` : '';
      return `${srv.name}${groupTag} (${srv.host}:${srv.port || 22})`;
    });
  } catch {
    return [];
  }
}

function getServerCount(cfg: CicdConfigEntry | null): number {
  return getServerNames(cfg).length;
}

async function selectDeployConfig(cfg: CicdConfigEntry) {
  selectedConfigId.value = cfg.id;
  config.value = null;
  loadingConfig.value = true;
  // 切换配置时加载分支
  loadBranchesForConfig(cfg);
  await loadConfigData(cfg.id);
  loadingConfig.value = false;
}

async function loadBranchesForConfig(cfg: CicdConfigEntry) {
  if (!cfg.gitRepoId) {
    branches.value = [];
    selectedBranch.value = cfg.deployBranch || 'main';
    return;
  }
  loadingBranches.value = true;
  try {
    // 找到仓库对象，获取本地路径
    const repo = gitRepos.value.find((r: any) => r.id === cfg.gitRepoId);
    const repoPath = repo?.path || repo?.repoPath || repo?.localPath || '';
    if (!repoPath) {
      branches.value = [cfg.deployBranch || 'main'];
      selectedBranch.value = cfg.deployBranch || 'main';
      return;
    }
    const result = await getTauriAPI().getGitBranches(repoPath);
    // 结果可能是 { branches: [...] } 或直接数组
    const rawBranches: any[] = result?.branches || result || [];
    // 去重，去掉 remotes/origin/ 前缀但保留本地分支优先
    const seen = new Set<string>();
    const local: string[] = [];
    const remote: string[] = [];
    for (const b of rawBranches) {
      const name = typeof b === 'string' ? b : (b.name || '');
      if (name.startsWith('remotes/origin/')) {
        const short = name.replace('remotes/origin/', '');
        if (!seen.has(short)) { seen.add(short); remote.push(short); }
      } else if (name !== 'HEAD' && !seen.has(name)) {
        seen.add(name); local.push(name);
      }
    }
    branches.value = [...local, ...remote];
    selectedBranch.value = cfg.deployBranch || 'main';
  } catch {
    branches.value = [cfg.deployBranch || 'main'];
    selectedBranch.value = cfg.deployBranch || 'main';
  } finally {
    loadingBranches.value = false;
  }
}

function getServersInfo(cfg: CicdConfigEntry | null): string {
  const names = getServerNames(cfg);
  return names.length > 0 ? names.join(', ') : '未配置';
}

function goToConfig() {
  window.dispatchEvent(new CustomEvent('switch-cicd-tab', { detail: 'config' }));
}

// ─── 日志批量渲染节流 ───
// maven/npm 构建是逐行输出（几百行/秒），后端每行 emit 一次 deploy-progress。
// 若逐行 push + 响应式更新，事件风暴会打满 JS 主线程 → 全局 UI 卡顿（点击无响应）。
// 改为按时间窗口批量追加日志行；progress/currentStep 仍即时更新。
const pendingLogLines = new Map<string, Array<{ time: string; stage: string; message: string }>>();
let logFlushTimer: number | null = null;
const LOG_FLUSH_MS = 50;

function flushPendingLogs() {
  if (logFlushTimer !== null) {
    clearTimeout(logFlushTimer);
    logFlushTimer = null;
  }
  if (pendingLogLines.size === 0) {return;}
  const needScroll: string[] = [];
  pendingLogLines.forEach((lines, cfgId) => {
    const state = deployStates.value.get(cfgId);
    if (!state || lines.length === 0) {return;}
    state.realtimeLogs.push(...lines);
    if (state.realtimeLogs.length > MAX_REALTIME_LOGS) {
      state.realtimeLogs.splice(0, state.realtimeLogs.length - MAX_REALTIME_LOGS);
    }
    updateDeployState(cfgId, { realtimeLogs: state.realtimeLogs });
    needScroll.push(cfgId);
  });
  pendingLogLines.clear();
  needScroll.forEach(cfgId => scrollToBottom());
}

function scheduleLogFlush() {
  if (logFlushTimer !== null) {return;}
  logFlushTimer = window.setTimeout(flushPendingLogs, LOG_FLUSH_MS);
}

const progressHandler = (data: { progress?: number; message?: string; stage?: string; status?: string; configId?: string; deployLogId?: string }) => {
  // 从事件中获取 configId，如果没有则使用当前选中的配置
  const cfgId = data.configId || selectedConfigId.value;
  if (!cfgId) {return;}

  const state = deployStates.value.get(cfgId);
  if (!state || !state.deploying) {return;} // 只处理正在部署的状态

  const pct = data.progress || 0;
  const isUploadProgress = data.stage === 'ssh' && data.status === 'uploading' && pct > 0;
  const shouldThrottle = isUploadProgress && (pct - state.lastLoggedProgress < 5) && pct < 100;
  
  // 构建更新对象（不直接修改 state，改用 updateDeployState 触发响应式）
  const updates: Partial<DeployState> = { progress: pct };
  if (!shouldThrottle) {
    if (isUploadProgress) {updates.lastLoggedProgress = pct;}
    const now = new Date().toLocaleTimeString('zh-CN');
    // 日志行先进缓冲，由 flushPendingLogs 批量追加（避免逐行渲染打满主线程）
    const arr = pendingLogLines.get(cfgId) || [];
    arr.push({ time: now, stage: data.stage || 'info', message: data.message || '' });
    pendingLogLines.set(cfgId, arr);
    scheduleLogFlush();
  }
  updates.currentStep = data.message || state.currentStep;
  if (data.deployLogId && !state.deployLogId) {
    updates.deployLogId = data.deployLogId;
  }
  updateDeployState(cfgId, updates);
  // 滚动由 flushPendingLogs 统一处理
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
          if (aTime && bTime) {return bTime.localeCompare(aTime)}
          if (aTime) {return -1}
          if (bTime) {return 1}
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
      if (!cfgId) {return;}
      
      // 先 flush 缓冲日志，保证成功/失败消息排在所有日志之后
      flushPendingLogs();

      const state = deployStates.value.get(cfgId);
      if (!state) {return;}
      
      if (data.success) {
        state.realtimeLogs.push({ time: new Date().toLocaleTimeString('zh-CN'), stage: 'deploy', message: '✅ 部署成功完成' });
        updateDeployState(cfgId, {
          deploying: false,
          progress: 100,
          currentStep: '部署成功！',
          realtimeLogs: state.realtimeLogs,
        });
        toast.success(`部署完成`);
        refreshLogs();
      } else if (data.cancelled) {
        updateDeployState(cfgId, {
          deploying: false,
          deployCancelled: true,
          currentStep: '⏹️ 部署已取消',
        });
        toast.info('部署已取消');
      } else {
        state.realtimeLogs.push({ time: new Date().toLocaleTimeString('zh-CN'), stage: 'error', message: '❌ 部署失败: ' + (data.error || '未知错误') });
        updateDeployState(cfgId, {
          deploying: false,
          currentStep: '部署失败: ' + (data.error || '未知错误'),
          realtimeLogs: state.realtimeLogs,
        });
        toast.error(`部署失败: ${data.error || '未知错误'}`, 6000);
        refreshLogs();
      }
      // 部署完成（成功/取消/失败）后延迟清理 deployStates，避免无界增长
      if (cfgId) {
        setTimeout(() => { deployStates.value.delete(cfgId); }, DEPLOY_STATE_TTL);
      }
    });
    getTauriAPI().onDataChanged?.(({ type }) => {
      if (type === 'cicd') { loadConfigs(); }
    }).then((cleanup) => { _cleanupDataChanged = cleanup; }).catch(() => {});
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
  // 清理挂起的日志 flush 定时器，避免卸载后写入已销毁状态
  if (logFlushTimer !== null) {
    clearTimeout(logFlushTimer);
    logFlushTimer = null;
  }
  pendingLogLines.clear();
});

async function loadConfigData(configId: string) {
  try {
    config.value = await getTauriAPI().getCicdConfigById(configId) as CicdConfigEntry | null;
    if (config.value) {
      const rawResult = await getTauriAPI().getDeployLogs(config.value.id) as any;
      const rawLogs = (Array.isArray(rawResult) ? rawResult : rawResult?.data || []) as DeployLog[];
      // Enrich logs with project name and per-config info
      const configMap = new Map<string, CicdConfigEntry>()
      for (const c of configs.value) {configMap.set(c.id, c)}
      logs.value = rawLogs.map(log => {
        const logConfig = configMap.get(log.configId)
        const repoName = logConfig?.gitRepoId ? getGitRepoName(logConfig.gitRepoId) : '';
        return {
          ...log,
          projectName: repoName || '',
          projectCategory: '',
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
    await shared.refresh();
  } catch (error) {
    handleError(error, { context: 'loadConfigs' });
  }
}

const preflightRunning = ref(false);
async function runPreflight() {
  if (!config.value) {
    toast.error('请先选择配置');
    return;
  }
  if (preflightRunning.value) { return; }
  preflightRunning.value = true;
  try {
    preflightResults.value = [];
    const result = await runPreflightCheck(config.value);
    preflightResults.value = result.results;
    return result.passed;
  } finally {
    preflightRunning.value = false;
  }
}

async function startDeploy() {
  if (!selectedConfigId.value) {return;}

  const currentState = deployStates.value.get(selectedConfigId.value);
  if (currentState?.deploying) {return;} // 该配置已经在部署中

  if (config.value?.requiresApproval) {
    const proceed = await showApprovalConfirm();
    if (!proceed) {return;}
  }

  const preflightOk = await runPreflight();
  if (!preflightOk) {
    const proceed = await confirm('预检未通过，是否继续部署？');
    if (!proceed) {return;}
  }

  // 初始化当前配置的部署状态（通过 updateDeployState 保证响应式）
  // 新部署替换 realtimeLogs 数组，重置上翻状态让新日志自动跟随到底部
  realtimeUserScrolledUp.value = false;
  updateDeployState(selectedConfigId.value, {
    deploying: true,
    currentStep: '开始部署...',
    realtimeLogs: [{ time: new Date().toLocaleTimeString('zh-CN'), stage: 'deploy', message: '部署任务已启动' }],
    startTime: Date.now(),
  });

  const deployLogIdHandler = (data: { deployLogId: string }) => {
    if (data.deployLogId) {
      updateDeployState(selectedConfigId.value, { deployLogId: data.deployLogId });
    }
  };
  const cleanupLogId = await getTauriAPI().onDeployLogIdCreated?.(deployLogIdHandler);

  try {
    const confirmed = !!config.value?.requiresApproval;
    const result = await getTauriAPI().deploy(selectedConfigId.value, confirmed || undefined, selectedBranch.value || undefined);
    cleanupLogId?.();

    if (!result.success) {
      const prevLogs = deployStates.value.get(selectedConfigId.value)?.realtimeLogs || [];
      updateDeployState(selectedConfigId.value, {
        deploying: false,
        currentStep: '部署失败: ' + (result.error || '配置异常'),
        realtimeLogs: [...prevLogs, { time: new Date().toLocaleTimeString('zh-CN'), stage: 'error', message: '❌ 部署失败: ' + (result.error || '配置异常') }],
      });
      if (result.requiresApproval) {
        toast.warning(result.message || '此配置需要审核确认', 5000);
      } else {
        toast.error('部署失败: ' + (result.error || '配置异常'), 6000);
      }
    }
  } catch (error) {
    // 先 flush 缓冲日志，保证错误消息排在构建日志之后且不污染后续部署
    flushPendingLogs();
    cleanupLogId?.(); // 异常路径也释放事件监听，避免泄漏
    updateDeployState(selectedConfigId.value, {
      deploying: false,
      currentStep: '部署失败: ' + (error as Error).message,
      realtimeLogs: [...(deployStates.value.get(selectedConfigId.value)?.realtimeLogs || []), { time: new Date().toLocaleTimeString('zh-CN'), stage: 'error', message: '❌ 部署异常: ' + (error as Error).message }],
    });
    handleError(error, { context: '部署' });
  }
}

async function cancelDeploy() {
  if (!selectedConfigId.value) {return;}
  const state = deployStates.value.get(selectedConfigId.value);
  if (!state?.deploying || !state.deployLogId) {return;}

  const confirmed = await confirm('确定要取消当前部署吗？');
  if (!confirmed) {return;}

  try {
    const result = await getTauriAPI().cancelDeploy(state.deployLogId);
    if (result.success) {
      updateDeployState(selectedConfigId.value, {
        deploying: false,
        deployCancelled: true,
        currentStep: '⏹️ 部署已取消',
        realtimeLogs: [...state.realtimeLogs, { time: new Date().toLocaleTimeString('zh-CN'), stage: 'info', message: '⏹️ 部署取消请求已发送' }],
      });
      toast.info('部署取消请求已发送');
    }
  } catch (error) {
    handleError(error, { context: '取消部署' });
  }
}

/** 从历史列表中取消正在运行的部署 */
async function cancelRunningDeploy(log: DeployLog) {
  const state = deployStates.value.get(log.configId);
  const deployLogId = state?.deployLogId || log.id;
  if (!deployLogId) {return;}

  const confirmed = await confirm('确定要取消此部署吗？');
  if (!confirmed) {return;}

  try {
    const result = await getTauriAPI().cancelDeploy(deployLogId);
    if (result.success) {
      // 更新本地状态
      if (state) {
        updateDeployState(log.configId, {
          deploying: false,
          deployCancelled: true,
          currentStep: '⏹️ 部署已取消',
        });
      }
      toast.info('部署取消请求已发送');
      await refreshLogs();
    }
  } catch (error) {
    handleError(error, { context: '取消部署' });
  }
}

async function rollbackDeploy(log: DeployLog) {
  // Check if config requires approval
  if (config.value?.requiresApproval) {
    const proceed = await confirm(
      `⚠️ 审核确认

配置「${config.value.name || getGitRepoName(config.value.gitRepoId)}」已开启部署审核。

请确认你要回滚到 ${formatDate(log.createdAt)} 的版本，是否继续？`
    );
    if (!proceed) {return;}
  } else {
    const confirmed = await confirm(
      `确定要回滚到 ${formatDate(log.createdAt)} 的部署版本吗？

此操作将把服务器恢复到该版本。`
    );
    if (!confirmed) {return;}
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

async function refreshLogs() {
  if (config.value?.id) {
    const rawResult = await getTauriAPI().getDeployLogs(config.value.id) as any;
    const rawLogs = (Array.isArray(rawResult) ? rawResult : rawResult?.data || []) as DeployLog[];
    const configMap = new Map<string, CicdConfigEntry>()
    for (const c of configs.value) {configMap.set(c.id, c)}
    logs.value = rawLogs.map(log => {
      const logConfig = configMap.get(log.configId)
      return {
        ...log,
        projectName: config.value ? getGitRepoName(config.value.gitRepoId) : '',
        projectCategory: '',
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
      // LRU：超过上限时删除最旧的条目
      const keys = Object.keys(stepLogs.value);
      if (keys.length >= MAX_STEP_LOGS) {
        delete stepLogs.value[keys[0]];
      }
      stepLogs.value[logId] = (await getTauriAPI().getDeployStepLogs(logId)) as DeployStep[];
    }
    // 自动加载日志文件（如果有 logFilePath 且尚未加载）
    const log = combinedLogs.value.find(l => l.id === logId);
    if (log?.logFilePath && !loadedLogContent.value[logId] && !loadingLogContent.value[logId]) {
      await loadLogContent(log);
    }
  }
}

/** Get real-time logs for a running deploy from its deploy state */
function runningRealtimeLogs(log: DeployLog) {
  if (log.status !== 'running') {return null;}
  const state = deployStates.value.get(log.configId);
  return state?.realtimeLogs || null;
}

async function loadLogContent(log: DeployLog) {
  if (!log.logFilePath || loadingLogContent.value[log.id]) {return;}
  loadingLogContent.value[log.id] = true;
  try {
    const content = await getTauriAPI().readLogFile(log.logFilePath!) as { success: boolean; content?: string; error?: string };
    if (content.success && content.content !== undefined) {
      // LRU：超过上限时删除最旧的条目
      const keys = Object.keys(loadedLogContent.value);
      if (keys.length >= MAX_LOADED_LOG_CONTENT) {
        delete loadedLogContent.value[keys[0]];
      }
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
  realtimeUserScrolledUp.value = false;
  if (selectedConfigId.value) {
    updateDeployState(selectedConfigId.value, { realtimeLogs: [] });
  }
}

function scrollToBottom(force = false) {
  // 用户已上翻查看历史时不再自动吸底（除非点"回到底部"按钮强制）
  if (!force && realtimeUserScrolledUp.value) {return;}
  // 程序化赋值 scrollTop 不触发 scroll 事件，必须手动重置，否则按钮不消失、后续吸底被永久跳过
  realtimeUserScrolledUp.value = false;
  nextTick(() => {
    if (logContainer.value) {
      logContainer.value.scrollTop = logContainer.value.scrollHeight
    }
  });
}
</script>
