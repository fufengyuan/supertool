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
                  @click="selectConfig(cfg.id)"
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

      <!-- Right Main Area: Config Editor -->
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

        <!-- Config Editor -->
        <div v-else class="flex-1 flex flex-col">
          <!-- Editor Header -->
          <div class="px-6 pt-5 pb-0 border-b border-base-content/10 bg-base-100">
            <div class="flex items-center justify-between mb-4">
              <h3 class="m-0 text-lg font-bold text-base-content">{{ isNewConfig ? '新建部署配置' : '编辑部署配置' }}</h3>
              <div class="flex gap-2">
                <button @click="testConnection" class="btn btn-ghost btn-sm" :disabled="!deployServers.some(s => s.serverId)">
                  <SvgIcon name="link" :size="14" class="inline-block align-text-bottom" /> 测试连接
                </button>
                <button @click="saveConfig" class="btn btn-primary btn-sm">
                  <SvgIcon name="save" :size="14" class="inline-block align-text-bottom" /> 保存
                </button>
              </div>
            </div>

            <!-- Pipeline Visualization -->
            <div class="flex items-center gap-1 py-3" v-if="selectedGitRepo">
              <div class="flex flex-col items-center gap-1 px-4 py-2 rounded-lg bg-base-200 border border-dashed border-base-content/10 min-w-[80px]" :class="{ 'bg-primary/10 border-primary border-solid': config.gitRepoId }">
                <SvgIcon name="folder" :size="18" />
                <span class="text-xs font-medium text-base-content">{{ selectedGitRepo?.name || '仓库' }}</span>
              </div>
              <div class="text-base text-base-content/60 opacity-40">→</div>
              <div class="flex flex-col items-center gap-1 px-4 py-2 rounded-lg bg-base-200 border border-dashed border-base-content/10 min-w-[80px]" :class="{ 'bg-primary/10 border-primary border-solid': config.buildTool }">
                <span class="text-lg">{{ getBuildToolIcon(config.buildTool) }}</span>
                <span class="text-xs font-medium text-base-content">{{ getBuildToolName(config.buildTool) || '构建' }}</span>
              </div>
              <div class="text-base text-base-content/60 opacity-40">→</div>
              <div class="flex flex-col items-center gap-1 px-4 py-2 rounded-lg bg-base-200 border border-dashed border-base-content/10 min-w-[80px]" :class="{ 'bg-primary/10 border-primary border-solid': deployServers.some(s => s.serverId) }">
                <SvgIcon name="serverRack" :size="18" />
                <span class="text-xs font-medium text-base-content">{{ deployServers.length > 0 ? deployServers.map(s => getServerName(s.serverId) || s.label).filter(Boolean).join(', ') || '服务器' : '服务器' }}</span>
              </div>
              <div class="text-base text-base-content/60 opacity-40">→</div>
              <div class="flex flex-col items-center gap-1 px-4 py-2 rounded-lg bg-primary border border-solid border-primary min-w-[80px]">
                <SvgIcon name="rocket" size="18" />
                <span class="text-xs font-medium text-white">部署</span>
              </div>
            </div>
          </div>

          <!-- Editor Body: Three Column Grid -->
          <div class="grid grid-cols-[1fr_1fr_1.3fr] gap-4 px-6 py-5 flex-1">
            <!-- Column 1: Project & Git -->
            <div class="bg-base-100 border border-base-content/10 rounded-xl p-5">
              <div class="flex items-center gap-2 mb-4 text-sm font-semibold text-base-content pb-3 border-b border-base-content/10">
                <SvgIcon name="folder" :size="18" />
                <span class="truncate">项目与仓库</span>
              </div>

              <div class="mb-3.5">
                <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">配置名称 <span class="text-xs font-normal text-base-content/60 normal-case tracking-normal ml-1">(自定义名称，便于区分多个配置)</span></label>
                <input v-model="config.name" class="input input-bordered w-full bg-base-200 text-sm" placeholder="例如：前端部署、后端API、定时任务..." />
              </div>

              <div class="mb-3.5">
                <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">Git 仓库 <span class="text-error normal-case tracking-normal">*</span></label>
                <select v-model="config.gitRepoId" @change="onGitRepoChange" class="select select-bordered w-full bg-base-200 text-sm">
                  <option value="">选择 Git 仓库...</option>
                  <option v-for="repo in gitRepos" :key="repo.id" :value="repo.id">
                    {{ repo.name }} — {{ repo.path }}
                  </option>
                </select>
              </div>

              <div class="mb-3.5">
                <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">分组</label>
                <div class="flex gap-1.5 items-center">
                  <select v-model="config.groupName" class="select select-bordered w-full bg-base-200 text-sm flex-1">
                    <option v-for="g in groups" :key="g" :value="g">{{ g }}</option>
                  </select>
                  <button @click="addGroup" class="btn btn-ghost btn-sm" title="新建分组">
                    <SvgIcon name="plus" size="14" stroke-width="2.5" />
                  </button>
                </div>
              </div>

              <!-- 打包模式 -->
              <div class="mb-3.5">
                <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">打包模式</label>
                <div class="flex gap-2">
                  <button
                    class="flex-1 px-3 py-2.5 rounded-xl border-2 text-sm font-medium transition-all duration-150 flex items-center gap-2"
                    :class="config.buildMode === 'local'
                      ? 'border-primary bg-primary/10 text-primary'
                      : 'border-base-content/10 text-base-content/60 hover:border-base-content/30'"
                    @click="config.buildMode = 'local'"
                  >
                    <SvgIcon name="folder" size="16" />
                    <div class="text-left">
                      <div class="font-semibold">本地目录</div>
                      <div class="text-[10px] opacity-70 font-normal">在项目目录直接构建</div>
                    </div>
                  </button>
                  <button
                    class="flex-1 px-3 py-2.5 rounded-xl border-2 text-sm font-medium transition-all duration-150 flex items-center gap-2"
                    :class="config.buildMode === 'git_clone'
                      ? 'border-primary bg-primary/10 text-primary'
                      : 'border-base-content/10 text-base-content/60 hover:border-base-content/30'"
                    @click="switchToGitCloneMode"
                  >
                    <SvgIcon name="gitBranch" size="16" />
                    <div class="text-left">
                      <div class="font-semibold">Git 克隆</div>
                      <div class="text-[10px] opacity-70 font-normal">克隆到隔离工作空间</div>
                    </div>
                  </button>
                </div>
              </div>

              <div class="mb-3.5">
                <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">
                  {{ config.buildMode === 'git_clone' ? '远程仓库地址' : '本地项目目录' }}
                </label>
                <!-- Git 克隆模式：显示可编辑的远程仓库地址 -->
                <template v-if="config.buildMode === 'git_clone'">
                  <div class="flex gap-1.5">
                    <input
                      v-model="config.repoUrl"
                      class="input input-bordered w-full bg-base-200 text-sm flex-1 min-w-0 font-mono"
                      placeholder="git@git.example.com:user/repo.git 或 https://..."
                    />
                    <button
                      @click="fetchGitRemoteUrl"
                      class="btn btn-ghost btn-sm gap-1 text-xs px-2.5 py-1.5 flex-shrink-0 whitespace-nowrap"
                      :disabled="!selectedGitRepo?.path"
                      title="从本地仓库获取远程地址"
                    >
                      <SvgIcon name="gitBranch" size="14" />
                      <span>获取</span>
                    </button>
                  </div>
                  <div v-if="!config.repoUrl && selectedGitRepo?.path" class="flex items-center gap-1.5 mt-1.5 px-2.5 py-1 text-xs text-base-content/60">
                    <SvgIcon name="lightbulb" size="14" />
                    <span>点击"获取"按钮自动填充远程仓库地址</span>
                  </div>
                </template>
                <!-- 本地目录模式：显示本地路径 -->
                <template v-else>
                  <div v-if="selectedGitRepo" class="flex items-center gap-2 px-3 py-2.5 bg-base-200 rounded-xl border border-base-content/10 text-sm font-mono text-base-content">
                    <SvgIcon name="folder" size="14" class="shrink-0 text-base-content/60" />
                    <span class="flex-1 truncate">{{ selectedGitRepo.path }}</span>
                    <button @click="openInFileManager(selectedGitRepo.path)" class="btn btn-ghost btn-xs px-1.5" title="打开文件夹"><SvgIcon name="externalLink" size="14" /></button>
                  </div>
                  <div v-else class="flex items-center gap-2 px-3 py-2.5 bg-base-200 rounded-xl text-xs text-base-content/60">
                    <SvgIcon name="folder" size="14" class="shrink-0 opacity-50" />
                    <span class="flex-1 truncate">尚未选择项目目录</span>
                    <button @click="selectLocalDir" class="btn btn-ghost btn-xs" title="选择本地目录并自动识别构建工具"><SvgIcon name="externalLink" size="12" /></button>
                  </div>
                </template>
              </div>

              <div class="mb-3.5">
                <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">部署分支</label>
                <div class="relative flex items-center gap-1.5">
                  <SvgIcon name="gitBranch" size="14" class="absolute left-2.5 text-base-content/60 pointer-events-none z-[1]" />
                  <select v-model="config.deployBranch" class="select select-bordered w-full bg-base-200 text-sm pl-8 pr-8 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed flex-1" :disabled="!selectedGitRepo">
                    <option value="main">main</option>
                    <option value="master">master</option>
                    <option v-for="branch in availableBranches" :key="branch" :value="branch">{{ branch }}</option>
                  </select>
                  <button
                    @click="loadBranches"
                    class="btn btn-ghost btn-sm p-1.5 min-w-[32px] h-8 disabled:opacity-50 disabled:cursor-not-allowed"
                    :disabled="!selectedGitRepo"
                    title="刷新分支列表"
                  >
                    <SvgIcon name="refresh" size="14" :class="{ 'animate-spin': loadingBranches }" />
                  </button>
                </div>
                <div v-if="selectedGitRepo && availableBranches.length === 0 && !loadingBranches" class="flex items-center gap-1.5 mt-1.5 px-2.5 py-1 text-xs text-base-content/60">
                  <SvgIcon name="lightbulb" size="14" />
                  <span>点击右侧刷新按钮加载分支列表</span>
                </div>
              </div>
            </div>

            <!-- Column 2: Servers -->
            <div class="bg-base-100 border border-base-content/10 rounded-xl p-5">
              <div class="flex items-center gap-2 mb-4 text-sm font-semibold text-base-content pb-3 border-b border-base-content/10">
                <SvgIcon name="serverRack" size="18" />
                <span class="truncate">目标服务器</span>
                <span class="ml-auto text-xs px-2 py-0.5 rounded-full bg-primary/10 text-primary font-bold">{{ deployServers.length }}</span>
              </div>

              <!-- Server list -->
              <div class="flex flex-col gap-2 max-h-[calc(100vh-400px)] overflow-y-auto pr-1">
                <div
                  v-for="(srv, idx) in deployServers"
                  :key="idx"
                  class="border border-base-content/10 rounded-xl bg-base-200 cursor-pointer transition-all duration-200 hover:border-primary group"
                  :class="{ 'border-primary bg-base-100 shadow-[0_2px_8px_rgba(46,171,124,0.08)]': activeServerIdx === idx }"
                  @click="activeServerIdx = idx"
                >
                  <div class="flex items-center justify-between px-3 py-2.5">
                    <span class="flex items-center gap-2 flex-1 min-w-0">
                      <span class="inline-flex items-center justify-center w-5 h-5 rounded-full bg-primary text-white text-xs font-bold flex-shrink-0">{{ idx + 1 }}</span>
                      <span v-if="getServerName(srv.serverId)" class="font-semibold text-sm text-base-content truncate">
                        {{ getServerName(srv.serverId) }}
                      </span>
                      <input
                        v-else
                        v-model="srv.label"
                        class="border-none bg-transparent text-sm font-semibold text-base-content w-full min-w-0 outline-none placeholder:text-base-content/60 placeholder:font-normal"
                        placeholder="自定义节点名称"
                        @click.stop
                      />
                    </span>
                    <button
                      v-if="deployServers.length > 1"
                      @click.stop="removeServer(idx)"
                      class="bg-transparent border-none cursor-pointer p-1 rounded-md text-base-content/60 opacity-0 group-hover:opacity-60 transition-all duration-150 hover:!opacity-100 hover:text-error hover:bg-error/10 flex-shrink-0"
                      title="移除"
                    >
                      <SvgIcon name="trash" size="14" />
                    </button>
                  </div>

                  <div class="px-3 pb-3" v-show="activeServerIdx === idx">
                    <!-- Server selector -->
                    <div class="mb-3.5">
                      <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">从已有服务器选择</label>
                      <GroupedServerSelector
                        :servers="servers"
                        :groups="serverGroups"
                        v-model="srv.serverId"
                        mode="single"
                      />
                    </div>

                    <div class="mb-3.5">
                      <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">部署路径</label>
                      <input v-model="srv.deployDir" class="input input-bordered w-full bg-base-200 text-sm" placeholder="如 /opt/app 或留空使用默认路径" @click.stop />
                    </div>

                    <!-- Test button -->
                    <button
                      v-if="srv.serverId"
                      @click.stop="testServerById(srv)"
                      class="btn btn-ghost btn-sm w-full"
                    >
                      <SvgIcon name="link" size="14" class="inline-block align-text-bottom" /> 测试连接
                    </button>
                    <div v-if="srv.testResult" class="mt-2 px-2.5 py-1.5 rounded-md text-xs font-medium" :class="srv.testResult.success ? 'bg-green-500/10 text-green-600' : 'bg-red-500/10 text-red-600'">
                      <template v-if="srv.testResult.success"><SvgIcon name="check" size="14" class="inline-block align-text-bottom" /> 连接成功</template><template v-else><SvgIcon name="x" size="14" class="inline-block align-text-bottom" /> {{ srv.testResult.error }}</template>
                    </div>
                  </div>
                </div>
              </div>

              <button @click="addServer" class="btn btn-ghost btn-sm w-full mt-2 border border-dashed border-base-content/10 text-base-content/60 hover:border-primary hover:text-primary hover:bg-primary/10">
                <SvgIcon name="plus" size="14" stroke-width="2.5" />
                添加服务器节点
              </button>

              <!-- Global test result -->
              <div v-if="testResult" class="flex items-center gap-2 px-3.5 py-2.5 rounded-lg text-sm mt-3" :class="testResult.success ? 'bg-green-500/10 text-success border border-green-500/20' : 'bg-red-500/10 text-error border border-red-500/20'">
                <span v-if="testResult.success"><SvgIcon name="check" size="14" class="inline-block align-text-bottom" /></span><span v-else><SvgIcon name="x" size="14" class="inline-block align-text-bottom" /></span>
                <span>{{ testResult.success ? '连接成功' : testResult.error }}</span>
              </div>
            </div>

            <!-- Column 3: Build & Deploy -->
            <div class="bg-base-100 border border-base-content/10 rounded-xl p-5">
              <div class="flex items-center gap-2 mb-4 text-sm font-semibold text-base-content pb-3 border-b border-base-content/10">
                <SvgIcon name="tool" size="18" />
                <span class="truncate">构建与部署</span>
              </div>

              <!-- Build tool cards -->
              <div class="grid grid-cols-[1fr_1fr_1.3fr] gap-2 mb-4">
                <div
                  v-for="tool in availableBuildTools"
                  :key="tool.key"
                  class="flex flex-col items-center px-2 py-3 border-2 border-base-content/10 rounded-xl cursor-pointer transition-all duration-150 relative hover:border-primary group"
                  :class="{
                    'border-primary bg-primary/10': config.buildTool === tool.key,
                    'opacity-40': !tool.available && tool.key !== 'cargo'
                  }"
                  :title="tool.path ? `${tool.name} → ${tool.path}` : tool.available ? tool.name : `${tool.name}（未安装）`"
                  @click="config.buildTool = tool.key"
                >
                  <span class="text-2xl mb-1">{{ tool.icon }}</span>
                  <span class="text-xs font-semibold text-base-content">{{ tool.name }}</span>
                  <span v-if="tool.version" class="text-[10px] text-base-content/60 mt-0.5">{{ tool.version.split(' ')[0] }}</span>
                  <span v-else-if="!tool.available && tool.key !== 'cargo'" class="text-[10px] text-base-content/40 mt-0.5">未安装</span>
                </div>
              </div>

              <!-- Maven options -->
              <template v-if="config.buildTool === 'maven'">
                <div class="mb-3.5 mt-2 text-[11px] font-semibold text-base-content/60 uppercase tracking-wider opacity-70 flex items-center gap-2.5">
                  <span>构建工具</span>
                  <span class="flex-1 h-px bg-base-content/10"></span>
                </div>
                <div class="mb-3.5">
                  <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">Maven 路径</label>
                  <div class="flex gap-1.5">
                    <input v-model="config.mavenHome" class="input input-bordered w-full bg-base-200 text-sm flex-1 min-w-0" :class="{ '!border-success !bg-green-500/5': config.mavenHome === defaultPaths.mavenHome && defaultPaths.mavenHome }" :placeholder="defaultPaths.mavenHome ? `已检测: ${defaultPaths.mavenHome}` : '自动检测 / 如 /opt/homebrew/opt/maven'" />
                    <button @click="reDetectToolPaths" class="btn btn-ghost btn-sm gap-1 text-xs px-2.5 py-1.5 flex-shrink-0 whitespace-nowrap" :disabled="detectingPaths" title="重新检测">
                      <SvgIcon name="search" size="14" :class="{ 'animate-spin': detectingPaths }" />
                      <span>检测</span>
                    </button>
                  </div>
                </div>
                <div class="mb-3.5">
                  <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">JDK 路径</label>
                  <div class="flex gap-1.5">
                    <input v-model="config.javaHome" class="input input-bordered w-full bg-base-200 text-sm flex-1 min-w-0" :class="{ '!border-success !bg-green-500/5': config.javaHome === defaultPaths.javaHome && defaultPaths.javaHome }" :placeholder="defaultPaths.javaHome ? `已检测: ${defaultPaths.javaHome}` : '自动检测 / 如 /opt/homebrew/opt/openjdk'" />
                    <select
                      v-if="sdkVersions.sdkman.java.length > 0"
                      v-model="selectedJavaVersion"
                      @change="onJavaVersionSelected"
                      class="select select-bordered bg-base-200 text-xs w-[200px] min-w-[120px] cursor-pointer truncate"
                      :title="selectedJavaVersion ? `${sdkVersions.sdkman.java.find(v => v.path === selectedJavaVersion)?.name || ''} → ${selectedJavaVersion}` : 'SDKMAN 版本'"
                    >
                      <option value="">版本…</option>
                      <option v-for="v in sdkVersions.sdkman.java" :key="v.path" :value="v.path" :title="`${v.name} → ${v.path}`">
                        {{ v.name }}{{ v.isCurrent ? ' ★' : '' }} → {{ v.path }}
                      </option>
                    </select>
                    <!-- SDKMAN 安装指引 -->
                    <div v-if="sdkmanInstallGuide.show && config.buildTool === 'maven'" class="mt-2 p-3 rounded-xl bg-amber-500/5 border border-amber-500/20">
                      <div class="flex items-start gap-2">
                        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 shrink-0 mt-0.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>
                        <div class="text-xs text-base-content/80 leading-relaxed">
                          <p class="m-0 mb-1 font-semibold text-amber-600">未检测到 SDKMAN</p>
                          <p class="m-0 mb-1.5">SDKMAN 可方便地管理 JDK 版本，推荐安装：</p>
                          <code class="block font-mono text-[11px] bg-base-200 p-2 rounded-lg leading-relaxed whitespace-pre-wrap">
                            <template v-for="(step, i) in sdkmanInstallGuide.steps" :key="i">
                              $ {{ step }}<br v-if="i < sdkmanInstallGuide.steps.length - 1" />
                            </template>
                          </code>
                          <button @click="reDetectToolPaths" class="mt-2 text-xs btn btn-ghost btn-xs gap-1">
                            <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>
                            检测后刷新
                          </button>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
                <!-- 父子模块构建时，Profile/settings.xml 很重要 — 直接展示 -->
                <template v-if="config.parentBuildMode">
                  <div class="mb-3.5 mt-2 text-[11px] font-semibold text-base-content/60 uppercase tracking-wider opacity-70 flex items-center gap-2.5">
                    <span>构建参数 <span class="text-[11px] font-normal text-primary ml-2 normal-case tracking-normal">（父模块统一构建，影响所有子模块）</span></span>
                    <span class="flex-1 h-px bg-base-content/10"></span>
                  </div>
                  <div class="flex gap-3">
                    <div class="mb-3.5 flex-1">
                      <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">Profile</label>
                      <input v-model="config.mavenProfile" class="input input-bordered w-full bg-base-200 text-sm" placeholder="prod" />
                    </div>
                    <div class="mb-3.5 flex-1">
                      <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">settings.xml</label>
                      <input v-model="config.mavenSettings" class="input input-bordered w-full bg-base-200 text-sm" placeholder="~/.m2/settings.xml" />
                    </div>
                  </div>
                </template>
                <!-- 非父子模块时，折叠为高级选项 -->
                <template v-else>
                  <div class="flex items-center gap-1.5 py-2 cursor-pointer text-xs font-semibold text-base-content/60 uppercase tracking-wider select-none transition-colors duration-150 hover:text-primary" @click="config.showAdvanced = !config.showAdvanced">
                    <SvgIcon name="chevronRight" size="14" class="transition-transform duration-200 flex-shrink-0" :class="{ 'rotate-90': config.showAdvanced }" />
                    <span>高级选项</span>
                  </div>
                  <div v-show="config.showAdvanced" class="overflow-hidden animate-[slideDown_0.2s_ease]">
                    <div class="flex gap-3">
                      <div class="mb-3.5 flex-1">
                        <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">Profile</label>
                        <input v-model="config.mavenProfile" class="input input-bordered w-full bg-base-200 text-sm" placeholder="prod" />
                      </div>
                      <div class="mb-3.5 flex-1">
                        <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">settings.xml</label>
                        <input v-model="config.mavenSettings" class="input input-bordered w-full bg-base-200 text-sm" placeholder="~/.m2/settings.xml" />
                      </div>
                    </div>
                  </div>
                </template>
              </template>

              <!-- NPM options -->
              <template v-if="['npm', 'pnpm', 'yarn'].includes(config.buildTool)">
                <div class="flex items-center gap-2.5 mb-3.5 mt-2 text-[11px] font-semibold text-base-content/60 uppercase tracking-wider opacity-70">
                  <span>构建工具</span>
                  <span class="flex-1 h-px bg-base-content/10"></span>
                </div>
                <div class="mb-3.5">
                  <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">{{ config.buildTool }} 路径</label>
                  <div class="flex gap-1.5">
                    <input v-model="currentHomePath" class="input input-bordered w-full bg-base-200 text-sm flex-1 min-w-0" :class="{ '!border-success !bg-green-500/5': currentHomePath === getDefaultPathFor(config.buildTool) && getDefaultPathFor(config.buildTool) }" :placeholder="getDefaultPathFor(config.buildTool) ? `已检测: ${getDefaultPathFor(config.buildTool)}` : `自动检测 / 如 /usr/local/bin/${config.buildTool}`" />
                    <button @click="reDetectToolPaths" class="btn btn-ghost btn-sm gap-1 text-xs px-2.5 py-1.5 flex-shrink-0 whitespace-nowrap" :disabled="detectingPaths" title="重新检测">
                      <SvgIcon name="search" size="14" :class="{ 'animate-spin': detectingPaths }" />
                      <span>检测</span>
                    </button>
                  </div>
                </div>
                <div class="mb-3.5">
                  <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">Node.js 路径</label>
                  <div class="flex gap-1.5">
                    <input v-model="config.nodeHome" class="input input-bordered w-full bg-base-200 text-sm flex-1 min-w-0" :class="{ '!border-success !bg-green-500/5': config.nodeHome === defaultPaths.nodeHome && defaultPaths.nodeHome }" :placeholder="defaultPaths.nodeHome ? `已检测: ${defaultPaths.nodeHome}` : '自动检测 / 如 ~/.nvm/versions/node/v20.x'" />
                    <select
                      v-if="sdkVersions.nvm.node.length > 0"
                      v-model="selectedNodeVersion"
                      @change="onNodeVersionSelected"
                      class="select select-bordered bg-base-200 text-xs w-[200px] min-w-[120px] cursor-pointer truncate"
                      :title="selectedNodeVersion ? `${sdkVersions.nvm.node.find(v => v.path === selectedNodeVersion)?.name || ''} → ${selectedNodeVersion}` : 'NVM 版本'"
                    >
                      <option value="">版本…</option>
                      <option v-for="v in sdkVersions.nvm.node" :key="v.path" :value="v.path" :title="`${v.name} → ${v.path}`">
                        {{ v.name }}{{ v.isCurrent ? ' ★' : '' }} → {{ v.path }}
                      </option>
                    </select>
                    <!-- NVM 安装指引 -->
                    <div v-if="nvmInstallGuide.show && ['npm','pnpm','yarn'].includes(config.buildTool)" class="mt-2 p-3 rounded-xl bg-amber-500/5 border border-amber-500/20">
                      <div class="flex items-start gap-2">
                        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 shrink-0 mt-0.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>
                        <div class="text-xs text-base-content/80 leading-relaxed">
                          <p class="m-0 mb-1 font-semibold text-amber-600">未检测到 NVM</p>
                          <p class="m-0 mb-1.5">NVM 可方便地管理 Node.js 版本，推荐安装：</p>
                          <code class="block font-mono text-[11px] bg-base-200 p-2 rounded-lg leading-relaxed whitespace-pre-wrap">
                            <template v-for="(step, i) in nvmInstallGuide.steps" :key="i">
                              $ {{ step }}<br v-if="i < nvmInstallGuide.steps.length - 1" />
                            </template>
                          </code>
                          <button @click="reDetectToolPaths" class="mt-2 text-xs btn btn-ghost btn-xs gap-1">
                            <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>
                            检测后刷新
                          </button>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
                <!-- 高级选项折叠 -->
                <div class="flex items-center gap-1.5 py-2 cursor-pointer text-xs font-semibold text-base-content/60 uppercase tracking-wider select-none transition-colors duration-150 hover:text-primary" @click="config.showAdvanced = !config.showAdvanced">
                  <SvgIcon name="chevronRight" size="14" class="transition-transform duration-200 flex-shrink-0" :class="{ 'rotate-90': config.showAdvanced }" />
                  <span>高级选项</span>
                </div>
                <div v-show="config.showAdvanced" class="overflow-hidden animate-[slideDown_0.2s_ease]">
                  <div class="mb-3.5">
                    <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">构建脚本</label>
                    <select v-model="config.npmScript" class="select select-bordered w-full bg-base-200 text-sm">
                      <option value="build">build</option>
                      <option value="build:prod">build:prod</option>
                      <option value="custom">自定义...</option>
                    </select>
                    <input
                      v-if="config.npmScript === 'custom'"
                      v-model="config.npmCustomScript"
                      class="input input-bordered w-full bg-base-200 text-sm mt-2"
                      placeholder="脚本名称"
                    />
                  </div>
                </div>
              </template>

              <!-- Cargo options -->
              <template v-if="config.buildTool === 'cargo'">
                <div class="mb-3.5 mt-2 text-[11px] font-semibold text-base-content/60 uppercase tracking-wider opacity-70 flex items-center gap-2.5">
                  <span>构建工具</span>
                  <span class="flex-1 h-px bg-base-content/10"></span>
                </div>
                <div class="mb-3.5">
                  <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">构建命令</label>
                  <div class="flex gap-2">
                    <button
                      class="flex-1 px-3 py-2 rounded-xl border-2 text-sm font-medium transition-all"
                      :class="config.buildCommand === 'release' || !config.buildCommand
                        ? 'border-primary bg-primary/10 text-primary'
                        : 'border-base-content/10 text-base-content/60'"
                      @click="config.buildCommand = 'release'"
                    >
                      cargo build --release
                    </button>
                    <button
                      class="flex-1 px-3 py-2 rounded-xl border-2 text-sm font-medium transition-all"
                      :class="config.buildCommand === 'debug'
                        ? 'border-primary bg-primary/10 text-primary'
                        : 'border-base-content/10 text-base-content/60'"
                      @click="config.buildCommand = 'debug'"
                    >
                      cargo build
                    </button>
                  </div>
                </div>
                <div class="mb-3.5">
                  <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">自定义构建命令</label>
                  <input v-model="config.buildCommand" class="input input-bordered w-full bg-base-200 text-sm font-mono" placeholder="cargo build --release --features xxx" />
                  <span class="block text-xs text-base-content/60 mt-1">留空使用上方选择的命令，填写后优先使用自定义命令</span>
                </div>
                <div class="px-3 py-2.5 rounded-xl bg-base-200 border border-base-content/10 text-xs text-base-content/60">
                  <span class="flex items-center gap-1.5">
                    <SvgIcon name="lightbulb" size="14" />
                    Cargo 构建产物默认在 <code class="bg-base-100 px-1 rounded">target/release/</code> 目录
                  </span>
                </div>
              </template>

              <!-- Deploy settings -->
              <div class="pt-3.5 border-t border-base-content/10">
                <div class="flex items-center gap-2.5 mb-3.5 text-[11px] font-semibold text-base-content/60 uppercase tracking-wider opacity-70">
                  <span>部署</span>
                  <span class="flex-1 h-px bg-base-content/10"></span>
                </div>
                <div class="mb-3.5">
                  <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">部署路径</label>
                  <input v-model="config.deployPath" class="input input-bordered w-full bg-base-200 text-sm" :placeholder="`/opt/${projectShortName}`" />
                </div>

                <!-- 重启脚本：仅 Maven 后端项目需要 -->
                <div class="mb-3.5" v-if="config.buildTool === 'maven'">
                  <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">重启脚本</label>
                  <input v-model="config.restartScript" class="input input-bordered w-full bg-base-200 text-sm" placeholder="./restart.sh" />
                </div>

                <!-- 依赖库分离：仅 Maven 后端项目需要 -->
                <label class="flex items-center gap-2 text-sm text-base-content cursor-pointer mt-2" v-if="config.buildTool === 'maven'">
                  <input v-model="config.libSeparate" type="checkbox" class="checkbox checkbox-primary" />
                  依赖库分离部署
                </label>

                <!-- 高级选项折叠 -->
                <div class="flex items-center gap-1.5 py-2 cursor-pointer text-xs font-semibold text-base-content/60 uppercase tracking-wider select-none transition-colors duration-150 hover:text-primary" @click="config.showAdvanced = !config.showAdvanced">
                  <SvgIcon name="chevronRight" size="14" class="transition-transform duration-200 flex-shrink-0" :class="{ 'rotate-90': config.showAdvanced }" />
                  <span>高级选项</span>
                </div>
                <div v-show="config.showAdvanced" class="overflow-hidden animate-[slideDown_0.2s_ease]">
                  <div class="mb-3.5">
                    <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">健康检查 URL</label>
                    <input v-model="config.healthCheckUrl" class="input input-bordered w-full bg-base-200 text-sm" placeholder="http://localhost:8080/health" />
                  </div>

                  <div class="my-3 border-t border-base-content/10"></div>

                  <label class="flex items-center gap-2 text-sm text-base-content cursor-pointer mt-2 px-2.5 py-2 bg-amber-500/10 rounded-md border border-amber-500/20">
                    <input v-model="config.requiresApproval" type="checkbox" class="checkbox checkbox-primary" />
                    <span class="flex flex-col gap-0.5">
                      <span class="flex items-center gap-1"><SvgIcon name="lock" size="14" /> 部署审核</span>
                      <span class="text-xs text-base-content/60">开启后部署前需要人工确认，防止误操作</span>
                    </span>
                  </label>
                </div>
              </div>
            </div>
          </div>

          <!-- Deploy Modules Section -->
          <!-- 父子模块构建模式 -->
          <div class="px-6 pb-3 border-l-3 border-transparent transition-[border-color,background] duration-300" :class="{
            'border-l-primary bg-primary/[0.05] mx-3 p-3 rounded-lg': config.parentBuildMode,
            'border-l-success bg-success/[0.05]': parentBuildAutoDetected
          }">
            <div class="flex items-center gap-2.5 mb-3.5 text-[11px] font-semibold text-base-content/60 uppercase tracking-wider opacity-70">
              <span class="flex items-center gap-1"><SvgIcon name="gitMerge" size="14" /> 父子模块构建</span>
              <span class="flex-1 h-px bg-base-content/10"></span>
            </div>
            <div class="mb-2">
              <label class="flex items-center gap-2.5 cursor-pointer select-none">
                <input type="checkbox" v-model="config.parentBuildMode" class="toggle toggle-primary" />
                <span class="text-sm font-medium text-base-content">是否为父子模块项目（父 POM 统一构建）</span>
              </label>
              <span v-if="parentBuildAutoDetected" class="inline-flex items-center gap-1 text-xs font-semibold text-success bg-green-500/10 border border-green-500/30 px-2 py-0.5 rounded-full ml-2.5 mt-1" title="由模块扫描自动检测"><SvgIcon name="search" size="12" /> 已自动检测</span>
              <span v-else class="block text-xs text-base-content/60 mt-1 ml-[54px]">开启后，所有子模块统一在父模块目录下执行一次构建，子模块只需设置远程部署路径</span>
            </div>
            <div v-if="config.parentBuildMode" class="mb-3.5 mt-2 ml-[54px]">
              <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">父模块构建目录 <span class="text-xs font-normal text-base-content/60 normal-case tracking-normal ml-1">(相对于项目本地路径)</span></label>
              <div class="flex gap-1.5">
                <input v-model="config.parentBuildPath" class="input input-bordered w-full bg-base-200 text-sm flex-1" :class="{ '!border-success !bg-green-500/5': config.parentBuildPath === parentBuildDetectedPath && parentBuildDetectedPath }" :placeholder="parentBuildDetectedPath ? `已检测: ${parentBuildDetectedPath}` : '留空使用项目根目录，或填写如 ./mall-framework'" />
                <button v-if="scannedModules.length > 0" @click="autoDetectParentBuild" class="btn btn-ghost btn-sm flex-shrink-0 whitespace-nowrap gap-1 text-xs px-2.5 py-1.5" title="重新检测"><SvgIcon name="search" size="12" /></button>
              </div>
              <span class="block text-xs text-base-content/60 mt-1">在该目录下执行 <code class="bg-base-200 px-1 rounded text-xs text-primary break-all">mvn clean package</code> 构建所有子模块</span>
            </div>
          </div>

          <div class="px-6 pb-6">
            <div class="flex items-center justify-between mb-3">
              <div class="flex items-center gap-2 text-sm font-semibold text-base-content">
                <SvgIcon name="grid" size="18" />
                <span>部署模块</span>
                <span class="text-xs font-normal text-base-content/60 ml-2">多模块项目配置每个模块的构建路径、命令和产物路径</span>
                <!-- Parent unified build mode indicator -->
                <span v-if="config.parentBuildMode" class="text-xs font-semibold text-amber-500 bg-amber-500/10 border border-amber-500/30 px-2 py-0.5 rounded-full ml-2 animate-pulse" title="父模块统一构建：一次 mvn 构建所有子模块，每个模块部署到独立远程路径">
                  <SvgIcon name="gitMerge" size="14" class="inline-block align-text-bottom" /> 父模块统一构建
                </span>
              </div>
              <div class="flex gap-2">
                <button @click="scanModules" class="btn btn-ghost btn-sm" :disabled="scanningModules || !config.localPath" :title="!config.localPath ? '请先选择有本地路径的项目' : ''">
                  <template v-if="scanningModules">
                    <SvgIcon name="search" size="14" class="animate-spin" />
                    扫描中...
                  </template>
                  <template v-else>
                    <SvgIcon name="search" size="14" class="inline-block align-text-bottom" />
                    自动识别模块
                  </template>
                </button>
                <button @click="addModule" class="btn btn-ghost btn-sm">+ 手动添加</button>
              </div>
            </div>

            <!-- Module Tree Dropdown -->
            <div v-if="scannedModules.length > 0" class="mb-3 border border-base-content/10 rounded-xl bg-base-100 overflow-hidden">
              <div class="flex items-center gap-2.5 px-3.5 py-2.5 cursor-pointer text-sm text-base-content transition-colors duration-150 hover:bg-base-200" @click="showModuleTree = !showModuleTree">
                <SvgIcon name="folder" size="14" />
                <span>已识别 {{ scannedModules.length }} 个模块，点击展开选择</span>
                <SvgIcon name="chevronDown" size="16" class="ml-auto transition-transform duration-200 text-base-content/60" :class="{ 'rotate-180': showModuleTree }" />
              </div>

              <div v-if="showModuleTree" class="border-t border-base-content/10 max-h-[400px] overflow-y-auto">
                <ModuleTreeNode
                  v-for="mod in scannedModules"
                  :key="mod.path"
                  :node="mod"
                  :depth="0"
                  :expanded-nodes="expandedTreeNodes"
                  :added-paths="addedModulePaths"
                  @toggle="toggleTreeNode"
                  @add="addModuleFromScan"
                />

                <!-- Bulk add button -->
                <div class="px-3.5 py-3 flex justify-end border-t border-base-content/10 bg-base-200">
                  <button @click="addAllDetectedModules" class="btn btn-primary btn-sm">
                    <SvgIcon name="clipboard" size="14" class="inline-block align-text-bottom" /> 添加全部未添加的模块
                  </button>
                </div>
              </div>
            </div>

            <div v-if="modules.length > 0" class="flex flex-col gap-2">
              <div v-for="(module, idx) in modules" :key="module.id || idx" class="border border-base-content/10 rounded-xl overflow-hidden bg-base-100 transition-all duration-150 hover:border-primary group">
                <div class="cursor-pointer px-3.5 py-2.5 transition-colors duration-150 hover:bg-base-200" @click="toggleModuleExpand(idx)">
                  <div class="flex items-center gap-2.5">
                    <SvgIcon name="chevronDown" size="16" class="transition-transform duration-200 text-base-content/60 flex-shrink-0" :class="{ 'rotate-180': expandedModules.includes(idx) }" />
                    <span class="text-xs font-semibold text-base-content/60 flex-shrink-0">#{{ idx + 1 }}</span>
                    <input v-model="module.moduleName" class="flex-1 px-2.5 py-1.5 border border-base-content/10 rounded-md bg-base-200 text-base-content text-sm font-medium min-w-0 focus:border-primary focus:outline-none focus:shadow-[0_0_0_2px_rgba(64,158,255,0.1)]" placeholder="模块名称" @click.stop />
                    <label class="toggle" @click.stop>
                      <input type="checkbox" v-model="module.enabled" />
                    </label>
                    <button @click.stop="deleteModule(module.id)" class="btn btn-ghost btn-sm btn-square text-error hover:bg-error/10" title="删除">
                      <SvgIcon name="x" size="14" />
                    </button>
                  </div>
                </div>

                <div v-if="expandedModules.includes(idx)" class="px-3.5 pb-3.5 border-t border-base-content/10 bg-base-200">
                  <div class="grid grid-cols-2 gap-3 py-3">
                    <div class="mb-3.5">
                      <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">模块路径</label>
                      <input v-model="module.modulePath" class="input input-bordered w-full bg-base-200 text-sm" placeholder="./sub-module 或留空使用项目根目录" />
                      <span class="block text-xs text-base-content/60 mt-1">{{ config.parentBuildMode ? '用于定位该模块的产物目录（如 mall-server/target/），构建在父模块目录统一执行' : '用于定位该模块的产物目录（如 mall-server/target/）' }}</span>
                    </div>
                    <div v-if="!config.parentBuildMode" class="mb-3.5">
                      <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">构建路径</label>
                      <input v-model="module.buildPath" class="input input-bordered w-full bg-base-200 text-sm" placeholder="子目录路径，如 frontend/admin" />
                      <span class="block text-xs text-base-content/60 mt-1">在该目录下执行构建命令</span>
                    </div>
                    <div v-if="config.parentBuildMode" class="mb-3.5">
                      <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">产物路径</label>
                      <input v-model="module.outputPath" class="input input-bordered w-full bg-base-200 text-sm" placeholder="target/ (默认)" />
                      <span class="block text-xs text-base-content/60 mt-1">相对于模块路径的构建产物目录</span>
                    </div>
                  </div>

                  <div v-if="!config.parentBuildMode" class="grid grid-cols-2 gap-3 py-3">
                    <div class="mb-3.5">
                      <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">构建工具</label>
                      <select v-model="module.buildTool" class="select select-bordered w-full bg-base-200 text-sm">
                        <option value="">继承全局</option>
                        <option value="maven">Maven</option>
                        <option value="npm">npm</option>
                        <option value="pnpm">pnpm</option>
                        <option value="yarn">Yarn</option>
                        <option value="gradle">Gradle</option>
                        <option value="custom">自定义命令</option>
                      </select>
                    </div>
                    <div class="mb-3.5">
                      <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">构建命令</label>
                      <input v-model="module.buildCommand" class="input input-bordered w-full bg-base-200 text-sm" placeholder="留空使用默认: mvn clean package / npm run build" />
                      <span class="block text-xs text-base-content/60 mt-1">为空时自动使用全局构建工具</span>
                    </div>
                  </div>

                  <div class="grid grid-cols-2 gap-3 py-3 last:pb-1">
                    <div v-if="!config.parentBuildMode" class="mb-3.5">
                      <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">产物路径</label>
                      <input v-model="module.outputPath" class="input input-bordered w-full bg-base-200 text-sm" placeholder="target/ 或 dist/ 或 build/" />
                      <span class="block text-xs text-base-content/60 mt-1">构建完成后产物所在的目录</span>
                    </div>
                    <div class="mb-3.5">
                      <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">远程部署路径 <span class="text-xs font-normal text-base-content/60 normal-case tracking-normal ml-1">(留空使用全局部署路径)</span></label>
                      <input v-model="module.deployPath" class="input input-bordered w-full bg-base-200 text-sm" placeholder="/opt/app-gateway 或留空使用全局" />
                      <span class="block text-xs text-base-content/60 mt-1">设置后，该模块产物将上传到指定远程目录，而非全局部署路径。父模块统一构建模式下，每个模块可独立设置</span>
                    </div>
                    <div class="mb-3.5">
                      <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">产物类型</label>
                      <select v-model="module.artifactType" class="select select-bordered w-full bg-base-200 text-sm">
                        <option value="">自动检测</option>
                        <option value="jar">单个 JAR 包</option>
                        <option value="jar-plus-lib">JAR + lib 目录（薄包部署）</option>
                        <option value="dist">前端构建产物目录</option>
                      </select>
                      <span class="block text-xs text-base-content/60 mt-1">薄包部署时选择「JAR + lib 目录」，将同时上传 JAR 和依赖库</span>
                    </div>
                    <div class="mb-3.5" v-if="module.artifactType === 'jar' || module.artifactType === 'jar-plus-lib'">
                      <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">JAR 文件名 <span class="text-xs font-normal text-base-content/60 normal-case tracking-normal ml-1">(留空自动检测)</span></label>
                      <input v-model="module.artifactName" class="input input-bordered w-full bg-base-200 text-sm" placeholder="app.jar 或留空自动检测" />
                    </div>
                    <div class="mb-3.5" v-if="module.artifactType === 'jar-plus-lib'">
                      <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">Lib 过滤规则 <span class="text-xs font-normal text-base-content/60 normal-case tracking-normal ml-1">(可选，一行一个，支持 * 通配符)</span></label>
                      <textarea v-model="module.libFilterRules" class="textarea textarea-bordered w-full bg-base-200 text-xs font-mono resize-y leading-relaxed" rows="4" placeholder="mall-*&#10;my-service-*&#10;留空表示全量上传" />
                      <span class="block text-xs text-base-content/60 mt-1">只上传匹配规则的 lib 文件，不配置则上传全部</span>
                    </div>
                    <div class="mb-3.5">
                      <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">部署顺序</label>
                      <input v-model.number="module.deployOrder" type="number" class="input input-bordered w-full bg-base-200 text-sm" placeholder="0" />
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
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
import GroupedServerSelector from '../server/GroupedServerSelector.vue';
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

const cicd = useCicdConfig();

// Git 仓库名称查找函数（供模板使用）
function getGitRepoName(id?: string) {
  if (!id) {return '';}
  const repo = cicd.gitRepos.value.find((r: any) => r.id === id);
  return repo ? repo.name : '';
}

// openInFileManager — opens a directory in the native file manager
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
  expandedModules, scannedModules, scanningModules, showModuleTree, expandedTreeNodes,
  defaultPaths, sdkVersions, selectedJavaVersion, selectedNodeVersion, detectingPaths,
  sdkmanInstallGuide, nvmInstallGuide,
  filteredConfigs, groupedConfigs, hasAnyGitSource, gitSources,
  projectShortName, availableBuildTools, addedModulePaths, buildToolDefs,
  parentBuildAutoDetected, parentBuildDetectedPath, selectedGitRepo,
  openGroupDialog, confirmGroupDialog, cancelGroupDialog, initExpandedGroups,
  makeDefaultServer, getServerName, onServerSelect, addServer, removeServer,
  testServerById, onJavaVersionSelected, onNodeVersionSelected, reDetectToolPaths,
  getProjectName, getToolBadge, getBuildToolIcon, getBuildToolName, formatTime,
  toggleGroup, renameGroup, addGroup, getServerLabel,
  loadConfigs, createNewConfig, selectConfig, onProjectChange, onGitRepoChange, selectLocalDir,
  selectServer, copyGitUrl, loadBranches, testConnection,
  addModule, toggleModuleExpand, scanModules, toggleTreeNode, isModuleAlreadyAdded,
  addModuleFromScan, addAllDetectedModules, flattenModuleTree, autoDetectParentBuild, deleteModule,
  saveConfig, deleteConfig, copyConfig, loadConfig, loadServers, loadProjects, loadGitRepos,
  switchToGitCloneMode, fetchGitRemoteUrl,
  defaultConfig, pageLoading,
} = cicd;

// Re-export types for template
import type { CicdConfigEntry, DeployModule, DeployServerEntry, ConfigForm } from './composables/useCicdConfig';
</script>
