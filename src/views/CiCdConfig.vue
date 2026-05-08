<template>
  <div class="cicd-manager">
    <!-- Left Sidebar: Config List -->
    <aside class="cicd-sidebar" :class="{ collapsed: sidebarCollapsed }">
      <div class="sidebar-header">
        <h3 v-show="!sidebarCollapsed">🚀 部署配置</h3>
        <div class="sidebar-header-actions">
          <button @click="createNewConfig" class="btn btn-primary btn-action" title="新建配置">
            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.5">
              <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
            </svg>
            <span>新建配置</span>
          </button>
          <button @click="sidebarCollapsed = !sidebarCollapsed" class="btn btn-ghost btn-action-collapse" :title="sidebarCollapsed ? '展开列表' : '收起列表'">
            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
              <polyline :points="sidebarCollapsed ? '9 18 15 12 9 6' : '15 18 9 12 15 6'" />
            </svg>
            <span v-show="!sidebarCollapsed">收起</span>
          </button>
        </div>
      </div>

      <div class="sidebar-search" v-show="!sidebarCollapsed">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" />
        </svg>
        <input v-model="searchQuery" placeholder="搜索配置..." class="search-input" />
      </div>

      <div class="config-list" v-show="!sidebarCollapsed">
        <!-- Empty state -->
        <div v-if="groupedConfigs.size === 0" class="empty-list">
          <svg viewBox="0 0 24 24" width="40" height="40" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
          </svg>
          <p>{{ configs.length === 0 ? '还没有部署配置' : '没有匹配的搜索结果' }}</p>
          <button @click="createNewConfig" class="btn btn-primary btn-sm">创建第一个</button>
        </div>

        <!-- Grouped config cards -->
        <template v-for="[groupName, groupConfigs] in groupedConfigs" :key="groupName">
          <div class="config-group" :class="{ collapsed: !expandedGroups.has(groupName) }">
            <div class="config-group-header" @click="toggleGroup(groupName)">
              <svg class="group-chevron" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="6 9 12 15 18 9" />
              </svg>
              <span class="group-name">{{ groupName }}</span>
              <span class="group-count">{{ groupConfigs.length }}</span>
              <button v-if="groupName !== '未分组'" @click.stop="renameGroup(groupName)" class="group-btn" title="重命名分组">
                <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
                  <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
                </svg>
              </button>
            </div>
            <div class="config-group-body">
              <div
                v-for="cfg in groupConfigs"
                :key="cfg.id"
                class="config-card"
                :class="{ active: selectedConfigId === cfg.id }"
                @click="selectConfig(cfg.id)"
              >
                <div class="config-card-header">
                  <span class="config-name">{{ cfg.name || getProjectName(cfg.projectId) }}</span>
                  <span class="config-project" v-if="cfg.name">{{ getProjectName(cfg.projectId) }}</span>
                  <span class="config-branch">{{ cfg.deployBranch || 'main' }}</span>
                  <span v-if="cfg.requiresApproval" class="config-approval-badge" title="需要审核确认">🔒</span>
                </div>
                <div class="config-card-body">
                  <span class="config-server">{{ getServerLabel(cfg) }}</span>
                  <span class="config-tool-badge">{{ getToolBadge(cfg.buildTool) }}</span>
                </div>
                <div class="config-card-footer">
                  <span class="config-time">{{ formatTime(cfg.updatedAt) }}</span>
                  <button @click.stop="deleteConfig(cfg.id)" class="config-delete" title="删除">
                    <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
                      <polyline points="3 6 5 6 21 6" /><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                    </svg>
                  </button>
                </div>
              </div>
            </div>
          </div>
        </template>
      </div>
    </aside>

    <!-- Right Main Area: Config Editor -->
    <main class="cicd-editor">
      <!-- No config selected -->
      <div v-if="!selectedConfigId && !isNewConfig" class="editor-empty">
        <div class="editor-empty-icon">
          <svg viewBox="0 0 24 24" width="64" height="64" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
            <line x1="12" y1="11" x2="12" y2="17" /><line x1="9" y1="14" x2="15" y2="14" />
          </svg>
        </div>
        <h3>选择或创建部署配置</h3>
        <p>从左侧选择一个已有配置，或创建新的部署配置</p>
        <button @click="createNewConfig" class="btn btn-primary btn-lg">＋ 新建部署配置</button>
      </div>

      <!-- Config Editor -->
      <div v-else class="editor-content">
        <!-- Editor Header -->
        <div class="editor-header">
          <div class="editor-title-row">
            <h3 class="editor-title">{{ isNewConfig ? '新建部署配置' : '编辑部署配置' }}</h3>
            <div class="editor-actions">
              <button @click="testConnection" class="btn btn-ghost btn-sm" :disabled="!deployServers.some(s => s.serverId)">
                🔗 测试连接
              </button>
              <button @click="saveConfig" class="btn btn-primary btn-sm">
                💾 保存
              </button>
            </div>
          </div>

          <!-- Pipeline Visualization -->
          <div class="pipeline-bar" v-if="selectedProject">
            <div class="pipeline-node" :class="{ filled: config.projectId }">
              <span class="pipeline-icon">📂</span>
              <span class="pipeline-label">{{ selectedProject.name }}</span>
            </div>
            <div class="pipeline-arrow">→</div>
            <div class="pipeline-node" :class="{ filled: config.buildTool }">
              <span class="pipeline-icon">{{ getBuildToolIcon(config.buildTool) }}</span>
              <span class="pipeline-label">{{ getBuildToolName(config.buildTool) || '构建' }}</span>
            </div>
            <div class="pipeline-arrow">→</div>
            <div class="pipeline-node" :class="{ filled: deployServers.some(s => s.serverId) }">
              <span class="pipeline-icon">🖥️</span>
              <span class="pipeline-label">{{ deployServers.length > 0 ? deployServers.map(s => getServerName(s.serverId) || s.label).filter(Boolean).join(', ') || '服务器' : '服务器' }}</span>
            </div>
            <div class="pipeline-arrow">→</div>
            <div class="pipeline-node deploy-node">
              <span class="pipeline-icon">🚀</span>
              <span class="pipeline-label">部署</span>
            </div>
          </div>
        </div>

        <!-- Editor Body: Three Column Grid -->
        <div class="editor-body">
          <!-- Column 1: Project & Git -->
          <div class="editor-column">
            <div class="column-header">
              <span class="column-icon">📂</span>
              <span class="column-title">项目与仓库</span>
            </div>

            <div class="form-field">
              <label>配置名称 <span class="optional">(自定义名称，便于区分多个配置)</span></label>
              <input v-model="config.name" class="form-input" placeholder="例如：前端部署、后端API、定时任务..." />
            </div>

            <div class="form-field">
              <label>关联项目 <span class="required">*</span></label>
              <select v-model="config.projectId" @change="onProjectChange" class="form-input">
                <option value="">选择项目...</option>
                <option v-for="proj in projects" :key="proj.id" :value="proj.id">
                  {{ proj.name }}
                </option>
              </select>
            </div>

            <div class="form-field">
              <label>分组</label>
              <div class="group-selector">
                <select v-model="config.groupName" class="form-input">
                  <option v-for="g in groups" :key="g" :value="g">{{ g }}</option>
                </select>
                <button @click="addGroup" class="btn btn-ghost btn-sm" title="新建分组">
                  <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.5">
                    <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
                  </svg>
                </button>
              </div>
            </div>

            <div class="form-field" v-if="hasAnyGitSource">
              <label>部署来源</label>
              <div class="deploy-source-tabs">
                <button
                  v-for="source in gitSources"
                  :key="source.key"
                  class="source-tab"
                  :class="{ active: config.repoUrl === source.url }"
                  @click="config.repoUrl = source.url"
                >
                  <span class="source-tab-icon">{{ source.icon }}</span>
                  <span class="source-tab-label">{{ source.label }}</span>
                  <span class="source-tab-path" :title="source.url">{{ source.path }}</span>
                </button>
              </div>
              <div class="git-group" style="margin-top: 8px;">
                <input v-model="config.repoUrl" class="form-input" readonly placeholder="选择上方来源后自动填充" />
                <button @click="copyGitUrl" class="btn btn-ghost btn-sm" :disabled="!config.repoUrl" title="复制">📋</button>
              </div>
            </div>

            <div class="form-field">
              <label>本地项目目录 <span class="optional">(可选，优先使用本地已构建产物)</span></label>
              <div class="local-path-group">
                <input v-model="config.localPath" class="form-input" :placeholder="selectedProject?.repoPath || '选择本地项目目录...'" />
                <button @click="selectLocalDir" class="btn btn-ghost btn-sm" title="选择目录">📁</button>
                <button v-if="config.localPath" @click="config.localPath = ''" class="btn btn-ghost btn-sm" title="清空">✕</button>
              </div>
              <div v-if="config.localPath" class="local-path-hint">
                <span class="hint-icon">⚡</span>
                <span>部署时将跳过 Git 克隆，直接使用本地目录进行构建</span>
              </div>
              <template v-else>
                <div v-if="selectedProject?.repoPath" class="local-path-hint">
                  <span class="hint-icon">📁</span>
                  <span>前端目录：<code>{{ selectedProject.repoPath }}</code>
                    <button @click="config.localPath = selectedProject.repoPath" class="hint-use-btn">使用</button>
                  </span>
                </div>
                <div v-if="selectedProject?.repoPath2" class="local-path-hint">
                  <span class="hint-icon">📁</span>
                  <span>后端目录：<code>{{ selectedProject.repoPath2 }}</code>
                    <button @click="config.localPath = selectedProject.repoPath2" class="hint-use-btn">使用</button>
                  </span>
                </div>
              </template>
            </div>

            <div class="form-field">
              <label>部署分支</label>
              <div class="branch-select-group">
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                  <line x1="6" y1="3" x2="6" y2="15" /><circle cx="18" cy="6" r="3" /><circle cx="6" cy="18" r="3" />
                  <path d="M18 9a9 9 0 0 1-9 9" />
                </svg>
                <select v-model="config.deployBranch" class="form-input branch-select" :disabled="!config.repoUrl && !config.localPath">
                  <option value="main">main</option>
                  <option value="master">master</option>
                  <option v-for="branch in availableBranches" :key="branch" :value="branch">{{ branch }}</option>
                </select>
                <button 
                  @click="loadBranches" 
                  class="btn btn-ghost btn-sm branch-refresh" 
                  :disabled="!config.repoUrl && !config.localPath"
                  title="刷新分支列表"
                >
                  <svg :class="{ spinning: loadingBranches }" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 2v6h-6" />
                    <path d="M3 12a9 9 0 0 1 15-6.7L21 8" />
                    <path d="M3 22v-6h6" />
                    <path d="M21 12a9 9 0 0 1-15 6.7L3 16" />
                  </svg>
                </button>
              </div>
              <div v-if="config.localPath && availableBranches.length === 0 && !loadingBranches" class="branch-hint">
                <span class="hint-icon">💡</span>
                <span>点击右侧刷新按钮加载分支列表</span>
              </div>
            </div>

            <!-- Project info card -->
            <div v-if="selectedProject" class="project-info-card">
              <div class="info-row" v-if="selectedProject.description">
                <span class="info-label">描述</span>
                <span class="info-value">{{ selectedProject.description }}</span>
              </div>
              <div class="info-row">
                <span class="info-label">仓库路径</span>
                <span class="info-value">{{ selectedProject.repoPath || '未设置' }}</span>
              </div>
            </div>
          </div>

          <!-- Column 2: Servers -->
          <div class="editor-column">
            <div class="column-header">
              <span class="column-icon">🖥️</span>
              <span class="column-title">目标服务器</span>
              <span class="server-count-badge">{{ deployServers.length }}</span>
            </div>

            <!-- Server list -->
            <div class="server-list">
              <div
                v-for="(srv, idx) in deployServers"
                :key="idx"
                class="server-entry"
                :class="{ active: activeServerIdx === idx }"
                @click="activeServerIdx = idx"
              >
                <div class="server-entry-header">
                  <span class="server-entry-label">
                    <span class="server-index">{{ idx + 1 }}</span>
                    <span v-if="getServerName(srv.serverId)" class="server-chip-name">
                      {{ getServerName(srv.serverId) }}
                    </span>
                    <input
                      v-else
                      v-model="srv.label"
                      class="server-label-input"
                      placeholder="自定义节点名称"
                      @click.stop
                    />
                  </span>
                  <button
                    v-if="deployServers.length > 1"
                    @click.stop="removeServer(idx)"
                    class="server-remove-btn"
                    title="移除"
                  >
                    <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                      <polyline points="3 6 5 6 21 6" />
                      <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                    </svg>
                  </button>
                </div>

                <div class="server-entry-body" v-show="activeServerIdx === idx">
                  <!-- Server selector -->
                  <div class="form-field">
                    <label>从已有服务器选择</label>
                    <GroupedServerSelector
                      :servers="servers"
                      :groups="serverGroups"
                      v-model="srv.serverId"
                      mode="single"
                    />
                  </div>

                  <div class="form-field">
                    <label>部署路径</label>
                    <input v-model="srv.deployDir" class="form-input" placeholder="如 /opt/app 或留空使用默认路径" @click.stop />
                  </div>

                  <!-- Test button -->
                  <button
                    v-if="srv.serverId"
                    @click.stop="testServerById(srv)"
                    class="btn btn-ghost btn-sm btn-full"
                  >
                    🔗 测试连接
                  </button>
                  <div v-if="srv.testResult" class="test-mini" :class="srv.testResult.success ? 'success' : 'error'">
                    {{ srv.testResult.success ? '✅ 连接成功' : '❌ ' + srv.testResult.error }}
                  </div>
                </div>
              </div>
            </div>

            <button @click="addServer" class="btn btn-ghost btn-sm btn-full add-server-btn">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.5">
                <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
              </svg>
              添加服务器节点
            </button>

            <!-- Global test result -->
            <div v-if="testResult" class="test-result-banner" :class="testResult.success ? 'success' : 'error'" style="margin-top: 12px">
              <span class="test-icon">{{ testResult.success ? '✅' : '❌' }}</span>
              <span>{{ testResult.success ? '连接成功' : testResult.error }}</span>
            </div>
          </div>

          <!-- Column 3: Build & Deploy -->
          <div class="editor-column">
            <div class="column-header">
              <span class="column-icon">🛠️</span>
              <span class="column-title">构建与部署</span>
            </div>

            <!-- Build tool cards -->
            <div class="build-tool-grid">
              <div
                v-for="tool in availableBuildTools"
                :key="tool.key"
                class="build-tool-card"
                :class="{ selected: config.buildTool === tool.key }"
                @click="config.buildTool = tool.key"
              >
                <span class="tool-icon">{{ tool.icon }}</span>
                <span class="tool-name">{{ tool.name }}</span>
                <span v-if="tool.version" class="tool-version">{{ tool.version.split(' ')[0] }}</span>
              </div>
            </div>

            <!-- Maven options -->
            <template v-if="config.buildTool === 'maven'">
              <div class="build-section-divider">构建工具</div>
              <div class="form-field">
                <label>Maven 路径</label>
                <div class="path-input-with-detect">
                  <input v-model="config.mavenHome" class="form-input" :class="{ 'auto-detected': config.mavenHome === defaultPaths.mavenHome && defaultPaths.mavenHome }" :placeholder="defaultPaths.mavenHome ? `已检测: ${defaultPaths.mavenHome}` : '自动检测 / 如 /opt/homebrew/opt/maven'" />
                  <button @click="reDetectToolPaths" class="btn btn-ghost btn-sm detect-btn" :disabled="detectingPaths" title="重新检测">
                    <svg :class="{ spinning: detectingPaths }" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                      <circle cx="11" cy="11" r="8" /><path d="m21 21-4.35-4.35" />
                    </svg>
                    <span>检测</span>
                  </button>
                </div>
              </div>
              <div class="form-field">
                <label>JDK 路径</label>
                <div class="path-input-with-select">
                  <input v-model="config.javaHome" class="form-input" :class="{ 'auto-detected': config.javaHome === defaultPaths.javaHome && defaultPaths.javaHome }" :placeholder="defaultPaths.javaHome ? `已检测: ${defaultPaths.javaHome}` : '自动检测 / 如 /opt/homebrew/opt/openjdk'" />
                  <select
                    v-if="sdkVersions.sdkman.java.length > 0"
                    v-model="selectedJavaVersion"
                    @change="onJavaVersionSelected"
                    class="version-select"
                    title="SDKMAN 版本"
                  >
                    <option value="">版本…</option>
                    <option v-for="v in sdkVersions.sdkman.java" :key="v.path" :value="v.path">
                      {{ v.name }}{{ v.isCurrent ? ' ★' : '' }}
                    </option>
                  </select>
                </div>
              </div>
              <!-- 父子模块构建时，Profile/settings.xml 很重要 — 直接展示 -->
              <template v-if="config.parentBuildMode">
                <div class="build-section-divider">构建参数 <span class="section-hint-inline">（父模块统一构建，影响所有子模块）</span></div>
                <div class="form-row">
                  <div class="form-field">
                    <label>Profile</label>
                    <input v-model="config.mavenProfile" class="form-input" placeholder="prod" />
                  </div>
                  <div class="form-field">
                    <label>settings.xml</label>
                    <input v-model="config.mavenSettings" class="form-input" placeholder="~/.m2/settings.xml" />
                  </div>
                </div>
              </template>
              <!-- 非父子模块时，折叠为高级选项 -->
              <template v-else>
                <div class="advanced-toggle" @click="config.showAdvanced = !config.showAdvanced">
                  <svg class="advanced-chevron" :class="{ open: config.showAdvanced }" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="6 9 12 15 18 9" />
                  </svg>
                  <span>高级选项</span>
                </div>
                <div v-show="config.showAdvanced" class="advanced-content">
                  <div class="form-row">
                    <div class="form-field">
                      <label>Profile</label>
                      <input v-model="config.mavenProfile" class="form-input" placeholder="prod" />
                    </div>
                    <div class="form-field">
                      <label>settings.xml</label>
                      <input v-model="config.mavenSettings" class="form-input" placeholder="~/.m2/settings.xml" />
                    </div>
                  </div>
                </div>
              </template>
            </template>

            <!-- NPM options -->
            <template v-if="['npm', 'pnpm', 'yarn'].includes(config.buildTool)">
              <div class="build-section-divider">构建工具</div>
              <div class="form-field">
                <label>{{ config.buildTool }} 路径</label>
                <div class="path-input-with-detect">
                  <input v-model="config[`${config.buildTool}Home`]" class="form-input" :class="{ 'auto-detected': config[`${config.buildTool}Home`] === defaultPaths[`${config.buildTool}Home`] && defaultPaths[`${config.buildTool}Home`] }" :placeholder="defaultPaths[`${config.buildTool}Home`] ? `已检测: ${defaultPaths[`${config.buildTool}Home`]}` : `自动检测 / 如 /usr/local/bin/${config.buildTool}`" />
                  <button @click="reDetectToolPaths" class="btn btn-ghost btn-sm detect-btn" :disabled="detectingPaths" title="重新检测">
                    <svg :class="{ spinning: detectingPaths }" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                      <circle cx="11" cy="11" r="8" /><path d="m21 21-4.35-4.35" />
                    </svg>
                    <span>检测</span>
                  </button>
                </div>
              </div>
              <div class="form-field">
                <label>Node.js 路径</label>
                <div class="path-input-with-select">
                  <input v-model="config.nodeHome" class="form-input" :class="{ 'auto-detected': config.nodeHome === defaultPaths.nodeHome && defaultPaths.nodeHome }" :placeholder="defaultPaths.nodeHome ? `已检测: ${defaultPaths.nodeHome}` : '自动检测 / 如 ~/.nvm/versions/node/v20.x'" />
                  <select
                    v-if="sdkVersions.nvm.node.length > 0"
                    v-model="selectedNodeVersion"
                    @change="onNodeVersionSelected"
                    class="version-select"
                    title="NVM 版本"
                  >
                    <option value="">版本…</option>
                    <option v-for="v in sdkVersions.nvm.node" :key="v.path" :value="v.path">
                      {{ v.name }}{{ v.isCurrent ? ' ★' : '' }}
                    </option>
                  </select>
                </div>
              </div>
              <!-- 高级选项折叠 -->
              <div class="advanced-toggle" @click="config.showAdvanced = !config.showAdvanced">
                <svg class="advanced-chevron" :class="{ open: config.showAdvanced }" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="6 9 12 15 18 9" />
                </svg>
                <span>高级选项</span>
              </div>
              <div v-show="config.showAdvanced" class="advanced-content">
                <div class="form-field">
                  <label>构建脚本</label>
                  <select v-model="config.npmScript" class="form-input">
                    <option value="build">build</option>
                    <option value="build:prod">build:prod</option>
                    <option value="custom">自定义...</option>
                  </select>
                  <input
                    v-if="config.npmScript === 'custom'"
                    v-model="config.npmCustomScript"
                    class="form-input"
                    placeholder="脚本名称"
                    style="margin-top: 8px"
                  />
                </div>
              </div>
            </template>

            <!-- Deploy settings -->
            <div class="deploy-settings">
              <div class="build-section-divider">部署</div>
              <div class="form-field">
                <label>部署路径</label>
                <input v-model="config.deployPath" class="form-input" :placeholder="`/opt/${projectShortName}`" />
              </div>

              <div class="form-field">
                <label>重启脚本</label>
                <input v-model="config.restartScript" class="form-input" placeholder="./restart.sh" />
              </div>

              <label class="form-checkbox">
                <input v-model="config.libSeparate" type="checkbox" />
                依赖库分离部署
              </label>

              <!-- 高级选项折叠 -->
              <div class="advanced-toggle" @click="config.showAdvanced = !config.showAdvanced">
                <svg class="advanced-chevron" :class="{ open: config.showAdvanced }" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="6 9 12 15 18 9" />
                </svg>
                <span>高级选项</span>
              </div>
              <div v-show="config.showAdvanced" class="advanced-content">
                <div class="form-field">
                  <label>健康检查 URL</label>
                  <input v-model="config.healthCheckUrl" class="form-input" placeholder="http://localhost:8080/health" />
                </div>

                <div class="approval-divider" style="margin: 12px 0; border-top: 1px solid oklch(var(--bc) / 0.1);"></div>

                <label class="form-checkbox form-checkbox-approval">
                  <input v-model="config.requiresApproval" type="checkbox" />
                  <span class="approval-label">
                    <span>🔒 部署审核</span>
                    <span class="approval-desc">开启后部署前需要人工确认，防止误操作</span>
                  </span>
                </label>
              </div>
            </div>
          </div>
        </div>

        <!-- Deploy Modules Section -->
        <!-- 父子模块构建模式 -->
        <div class="parent-build-section" :class="{ active: config.parentBuildMode, 'auto-detected': parentBuildAutoDetected }">
          <div class="build-section-divider">🏗️ 父子模块构建</div>
          <div class="parent-build-toggle">
            <label class="toggle-label">
              <input type="checkbox" v-model="config.parentBuildMode" class="toggle-input" />
              <span class="toggle-switch"></span>
              <span class="toggle-text">是否为父子模块项目（父 POM 统一构建）</span>
            </label>
            <span v-if="parentBuildAutoDetected" class="auto-detected-badge" title="由模块扫描自动检测">🔍 已自动检测</span>
            <span v-else class="toggle-hint">开启后，所有子模块统一在父模块目录下执行一次构建，子模块只需设置远程部署路径</span>
          </div>
          <div v-if="config.parentBuildMode" class="form-field parent-build-path-field">
            <label>父模块构建目录 <span class="optional">(相对于项目本地路径)</span></label>
            <div class="parent-build-path-group">
              <input v-model="config.parentBuildPath" class="form-input" :class="{ 'auto-detected': config.parentBuildPath === parentBuildDetectedPath && parentBuildDetectedPath }" :placeholder="parentBuildDetectedPath ? `已检测: ${parentBuildDetectedPath}` : '留空使用项目根目录，或填写如 ./yudao-framework'" />
              <button v-if="scannedModules.length > 0" @click="autoDetectParentBuild" class="btn btn-ghost btn-sm detect-btn" title="重新检测">🔍</button>
            </div>
            <span class="field-hint">在该目录下执行 <code>mvn clean package</code> 构建所有子模块</span>
          </div>
        </div>

        <div class="modules-section">
          <div class="section-header">
            <div class="section-title">
              <span class="section-icon">🧩</span>
              <span>部署模块</span>
              <span class="section-hint">多模块项目配置每个模块的构建路径、命令和产物路径</span>
              <!-- Parent unified build mode indicator -->
              <span v-if="config.parentBuildMode" class="parent-build-badge" title="父模块统一构建：一次 mvn 构建所有子模块，每个模块部署到独立远程路径">
                🏗️ 父模块统一构建
              </span>
            </div>
            <div class="section-actions">
              <button @click="scanModules" class="btn btn-ghost btn-sm" :disabled="scanningModules || !selectedProject?.repoPath" :title="!selectedProject?.repoPath ? '请先选择有本地路径的项目' : ''">
                <svg :class="{ spinning: scanningModules }" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="11" cy="11" r="8" /><path d="m21 21-4.35-4.35" />
                </svg>
                {{ scanningModules ? '扫描中...' : '🔍 自动识别模块' }}
              </button>
              <button @click="addModule" class="btn btn-ghost btn-sm">+ 手动添加</button>
            </div>
          </div>

          <!-- Module Tree Dropdown -->
          <div v-if="scannedModules.length > 0" class="module-tree-dropdown" :class="{ open: showModuleTree }">
            <div class="tree-trigger" @click="showModuleTree = !showModuleTree">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z" />
              </svg>
              <span>已识别 {{ scannedModules.length }} 个模块，点击展开选择</span>
              <svg class="tree-arrow" :class="{ open: showModuleTree }" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="6 9 12 15 18 9" />
              </svg>
            </div>

            <div v-if="showModuleTree" class="tree-content">
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
              <div class="tree-bulk-actions">
                <button @click="addAllDetectedModules" class="btn btn-primary btn-sm">
                  📋 添加全部未添加的模块
                </button>
              </div>
            </div>
          </div>

          <div v-if="modules.length > 0" class="modules-list">
            <div v-for="(module, idx) in modules" :key="module.id || idx" class="module-card">
              <div class="module-card-header" @click="toggleModuleExpand(idx)">
                <div class="module-title-row">
                  <svg class="expand-icon" :class="{ expanded: expandedModules.includes(idx) }" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="6 9 12 15 18 9" />
                  </svg>
                  <span class="module-index">#{{ idx + 1 }}</span>
                  <input v-model="module.moduleName" class="module-name-input" placeholder="模块名称" @click.stop />
                  <label class="toggle-switch" @click.stop>
                    <input type="checkbox" v-model="module.enabled" />
                    <span class="toggle-slider"></span>
                  </label>
                  <button @click.stop="deleteModule(module.id)" class="btn btn-icon btn-danger btn-sm" title="删除">
                    <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                      <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
                    </svg>
                  </button>
                </div>
              </div>

              <div v-if="expandedModules.includes(idx)" class="module-card-body">
                <div class="module-fields">
                  <div class="form-field">
                    <label>模块路径</label>
                    <input v-model="module.modulePath" class="form-input" placeholder="./sub-module 或留空使用项目根目录" />
                    <span class="field-hint">{{ config.parentBuildMode ? '用于定位该模块的产物目录（如 yudao-server/target/），构建在父模块目录统一执行' : '用于定位该模块的产物目录（如 yudao-server/target/）' }}</span>
                  </div>
                  <div v-if="!config.parentBuildMode" class="form-field">
                    <label>构建路径</label>
                    <input v-model="module.buildPath" class="form-input" placeholder="子目录路径，如 frontend/admin" />
                    <span class="field-hint">在该目录下执行构建命令</span>
                  </div>
                  <div v-if="config.parentBuildMode" class="form-field">
                    <label>产物路径</label>
                    <input v-model="module.outputPath" class="form-input" placeholder="target/ (默认)" />
                    <span class="field-hint">相对于模块路径的构建产物目录</span>
                  </div>
                </div>

                <div v-if="!config.parentBuildMode" class="module-fields">
                  <div class="form-field">
                    <label>构建工具</label>
                    <select v-model="module.buildTool" class="form-input">
                      <option value="">继承全局</option>
                      <option value="maven">Maven</option>
                      <option value="npm">npm</option>
                      <option value="pnpm">pnpm</option>
                      <option value="yarn">Yarn</option>
                      <option value="gradle">Gradle</option>
                      <option value="custom">自定义命令</option>
                    </select>
                  </div>
                  <div class="form-field">
                    <label>构建命令</label>
                    <input v-model="module.buildCommand" class="form-input" placeholder="留空使用默认: mvn clean package / npm run build" />
                    <span class="field-hint">为空时自动使用全局构建工具</span>
                  </div>
                </div>

                <div class="module-fields">
                  <div v-if="!config.parentBuildMode" class="form-field">
                    <label>产物路径</label>
                    <input v-model="module.outputPath" class="form-input" placeholder="target/ 或 dist/ 或 build/" />
                    <span class="field-hint">构建完成后产物所在的目录</span>
                  </div>
                  <div class="form-field">
                    <label>远程部署路径 <span class="optional">(留空使用全局部署路径)</span></label>
                    <input v-model="module.deployPath" class="form-input" placeholder="/opt/app-gateway 或留空使用全局" />
                    <span class="field-hint">设置后，该模块产物将上传到指定远程目录，而非全局部署路径。父模块统一构建模式下，每个模块可独立设置</span>
                  </div>
                  <div class="form-field">
                    <label>产物类型</label>
                    <select v-model="module.artifactType" class="form-input">
                      <option value="">自动检测</option>
                      <option value="jar">单个 JAR 包</option>
                      <option value="jar-plus-lib">JAR + lib 目录（薄包部署）</option>
                      <option value="dist">前端构建产物目录</option>
                    </select>
                    <span class="field-hint">薄包部署时选择「JAR + lib 目录」，将同时上传 JAR 和依赖库</span>
                  </div>
                  <div class="form-field" v-if="module.artifactType === 'jar' || module.artifactType === 'jar-plus-lib'">
                    <label>JAR 文件名 <span class="optional">(留空自动检测)</span></label>
                    <input v-model="module.artifactName" class="form-input" placeholder="app.jar 或留空自动检测" />
                  </div>
                  <div class="form-field" v-if="module.artifactType === 'jar-plus-lib'">
                    <label>Lib 过滤规则 <span class="optional">(可选，一行一个，支持 * 通配符)</span></label>
                    <textarea v-model="module.libFilterRules" class="form-input lib-filter-input" rows="4" placeholder="yudao-*&#10;my-service-*&#10;留空表示全量上传" />
                    <span class="field-hint">只上传匹配规则的 lib 文件，不配置则上传全部</span>
                  </div>
                  <div class="form-field">
                    <label>部署顺序</label>
                    <input v-model.number="module.deployOrder" type="number" class="form-input" placeholder="0" />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </main>

    <!-- Group Name Dialog -->
    <div v-if="showGroupDialog" class="group-dialog-overlay" @click.self="cancelGroupDialog">
      <div class="group-dialog" @keydown.escape="cancelGroupDialog">
        <h4>{{ groupDialogMode === 'add' ? '新建分组' : '重命名分组' }}</h4>
        <input
          v-model="groupNameInput"
          ref="groupNameInputRef"
          class="group-dialog-input form-input"
          placeholder="输入分组名称"
          @keydown.enter="confirmGroupDialog"
          autofocus
        />
        <div class="group-dialog-actions">
          <button class="btn btn-ghost" @click="cancelGroupDialog">取消</button>
          <button class="btn btn-primary" @click="confirmGroupDialog" :disabled="!groupNameInput.trim()">确定</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { useCicdConfig } from '@/composables/useCicdConfig';
import ModuleTreeNode from '@/components/cicd/ModuleTreeNode.vue';
import GroupedServerSelector from '@/components/server/GroupedServerSelector.vue';

const cicd = useCicdConfig();

// Destructure all refs, computed, and functions for template access
const {
  configs, projects, servers, serverGroups, selectedConfigId, isNewConfig, searchQuery, sidebarCollapsed,
  selectedServerId, deployServers, activeServerIdx, groups, expandedGroups,
  showGroupDialog, groupNameInput, groupDialogMode, groupDialogOldName,
  showGroupEditor, newGroupName,
  config, modules, testResult, detectedTools, availableBranches, loadingBranches,
  expandedModules, scannedModules, scanningModules, showModuleTree, expandedTreeNodes,
  defaultPaths, sdkVersions, selectedJavaVersion, selectedNodeVersion, detectingPaths,
  filteredConfigs, groupedConfigs, selectedProject, hasAnyGitSource, gitSources,
  projectShortName, availableBuildTools, addedModulePaths, buildToolDefs,
  parentBuildAutoDetected, parentBuildDetectedPath,
  openGroupDialog, confirmGroupDialog, cancelGroupDialog, initExpandedGroups,
  makeDefaultServer, getServerName, onServerSelect, addServer, removeServer,
  testServerById, onJavaVersionSelected, onNodeVersionSelected, reDetectToolPaths,
  addModuleFromScan, addAllDetectedModules, autoDetectParentBuild,
  getProjectName, getToolBadge, getBuildToolIcon, getBuildToolName, formatTime,
  toggleGroup, renameGroup, addGroup, getServerLabel,
  loadConfigs, createNewConfig, selectConfig, onProjectChange, selectLocalDir,
  selectServer, copyGitUrl, loadBranches, testConnection,
  addModule, toggleModuleExpand, scanModules, toggleTreeNode, isModuleAlreadyAdded,
  flattenModuleTree, deleteModule,
  saveConfig, deleteConfig, loadConfig, loadServers, loadProjects,
  defaultConfig,
} = cicd;

// Re-export types for template
import type { CicdConfigEntry, DeployModule, DeployServerEntry, ConfigForm } from '@/composables/useCicdConfig';
</script>

<style scoped>
/* ─── Layout ─── */
.cicd-manager {
  display: flex;
  height: 100%;
  overflow: hidden;
  background: oklch(var(--b2));
}

/* ─── Sidebar ─── */
.cicd-sidebar {
  width: 300px;
  min-width: 260px;
  max-width: 360px;
  border-right: 1px solid oklch(var(--bc) / 0.1);
  background: oklch(var(--b1));
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  transition: width 0.3s ease, min-width 0.3s ease, max-width 0.3s ease;
  overflow: hidden;
}

.cicd-sidebar.collapsed {
  width: 52px;
  min-width: 52px;
  max-width: 52px;
  border-right-color: transparent;
}

.cicd-sidebar.collapsed:hover {
  border-right-color: oklch(var(--bc) / 0.1);
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px 12px;
}

.cicd-sidebar.collapsed .sidebar-header {
  flex-direction: column;
  align-items: center;
  padding: 12px 8px;
}

.sidebar-header-actions {
  display: flex;
  gap: 8px;
}

/* Ensure SVG icons inside btn-icon are visible */
.btn-icon svg,
.btn-icon svg line,
.btn-icon svg polyline {
  stroke: currentColor !important;
  fill: none !important;
  display: block;
}

.btn-primary.btn-icon svg {
  stroke: white !important;
}

/* New action buttons: icon + text */
.btn-action,
.btn-action-collapse {
  height: 34px;
  padding: 0 14px;
  gap: 6px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  border: none;
  cursor: pointer;
  transition: all 0.15s ease;
  white-space: nowrap;
}

.btn-action {
  background: oklch(var(--p));
  color: white;
  box-shadow: 0 2px 8px rgba(64, 158, 255, 0.3);
}

.btn-action:hover {
  filter: brightness(1.1);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(64, 158, 255, 0.4);
}

.btn-action svg {
  stroke: white !important;
  flex-shrink: 0;
}

.btn-action-collapse {
  background: transparent;
  color: oklch(var(--bc) / 0.6);
  border: 1px solid oklch(var(--bc) / 0.1);
}

.btn-action-collapse:hover {
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  border-color: oklch(var(--bc) / 0.6);
}

.btn-action-collapse svg {
  stroke: currentColor !important;
  flex-shrink: 0;
}

/* Collapsed state: just icon, no text */
.cicd-sidebar.collapsed .sidebar-header-actions {
  flex-direction: column;
  align-items: center;
  gap: 10px;
  width: 100%;
}

.cicd-sidebar.collapsed .btn-action span {
  display: none;
}

.cicd-sidebar.collapsed .btn-action {
  width: 36px;
  height: 36px;
  padding: 0;
  border-radius: 10px;
}

.cicd-sidebar.collapsed .btn-action-collapse span {
  display: none;
}

.cicd-sidebar.collapsed .btn-action-collapse {
  width: 36px;
  height: 36px;
  padding: 0;
  border-radius: 10px;
}

.sidebar-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 700;
  color: oklch(var(--bc));
}

.btn-icon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  padding: 0;
  color: white;
  background: oklch(var(--p));
  transition: all 0.15s ease;
}

.btn-icon:hover {
  background: var(--primary-dark);
  transform: scale(1.05);
}

.btn-icon svg {
  width: 16px;
  height: 16px;
  stroke: currentColor;
}

.btn-ghost.btn-icon {
  background: transparent;
  color: oklch(var(--bc) / 0.6);
  border: 1px solid oklch(var(--bc) / 0.1);
}
.btn-ghost.btn-icon:hover {
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  border-color: oklch(var(--bc) / 0.6);
}

.sidebar-search {
  position: relative;
  padding: 0 16px 12px;
}

.sidebar-search svg {
  position: absolute;
  left: 28px;
  top: 50%;
  transform: translateY(-50%);
  color: oklch(var(--bc) / 0.6);
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: 8px 12px 8px 32px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  font-size: 13px;
}

.search-input:focus {
  outline: none;
  border-color: oklch(var(--p));
}

.config-list {
  flex: 1;
  overflow-y: auto;
  padding: 0 12px 12px;
}

.empty-list {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 40px 20px;
  text-align: center;
  color: oklch(var(--bc) / 0.6);
  gap: 12px;
}

.empty-list svg { opacity: 0.3; }
.empty-list p { margin: 0; font-size: 14px; }

.config-card {
  padding: 14px 16px;
  border-radius: 10px;
  cursor: pointer;
  margin-bottom: 6px;
  border: 1px solid transparent;
  transition: all 0.15s ease;
}

.config-card:hover {
  background: oklch(var(--b2));
  border-color: oklch(var(--bc) / 0.1);
}

.config-card.active {
  background: oklch(var(--p) / 0.1);
  border-color: oklch(var(--p));
}

.config-card-header {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 6px;
}

.config-name {
  font-size: 14px;
  font-weight: 600;
  color: oklch(var(--bc));
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
  min-width: 0;
}

.config-project {
  font-size: 11px;
  color: oklch(var(--bc) / 0.6);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex-shrink: 0;
  max-width: 80px;
}

.config-branch {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 4px;
  background: oklch(var(--b2));
  color: oklch(var(--bc) / 0.6);
  flex-shrink: 0;
}

.config-card.active .config-branch {
  background: rgba(255,255,255,0.2);
  color: oklch(var(--p));
}

.config-approval-badge {
  font-size: 11px;
  flex-shrink: 0;
}

.config-card-body {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.config-server {
  font-size: 12px;
  color: oklch(var(--bc) / 0.6);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.config-tool-badge {
  font-size: 14px;
  flex-shrink: 0;
}

.config-card-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.config-time {
  font-size: 11px;
  color: oklch(var(--bc) / 0.6);
  opacity: 0.6;
}

.config-delete {
  background: none;
  border: none;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  color: oklch(var(--bc) / 0.6);
  opacity: 0;
  transition: all 0.15s;
}

.config-card:hover .config-delete { opacity: 1; }
.config-delete:hover { color: oklch(var(--er)); background: rgba(239, 68, 68, 0.1); }

/* ─── Group Styles ─── */
.config-group {
  margin-bottom: 4px;
}

.config-group-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  cursor: pointer;
  border-radius: 6px;
  transition: background 0.15s;
  user-select: none;
}

.config-group-header:hover {
  background: rgba(255, 255, 255, 0.05);
}

.group-chevron {
  transition: transform 0.2s;
  flex-shrink: 0;
}

.config-group.collapsed .group-chevron {
  transform: rotate(-90deg);
}

.group-name {
  font-size: 12px;
  font-weight: 600;
  color: oklch(var(--bc) / 0.6);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  flex: 1;
}

.group-count {
  font-size: 11px;
  color: oklch(var(--bc) / 0.6);
  opacity: 0.6;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  padding: 1px 7px;
  min-width: 18px;
  text-align: center;
}

.group-btn {
  background: none;
  border: none;
  cursor: pointer;
  padding: 3px;
  border-radius: 4px;
  color: oklch(var(--bc) / 0.6);
  opacity: 0;
  transition: all 0.15s;
  display: flex;
  align-items: center;
}

.config-group-header:hover .group-btn { opacity: 0.6; }
.group-btn:hover { opacity: 1 !important; background: rgba(255, 255, 255, 0.1); }

.config-group-body {
  overflow: hidden;
  max-height: 2000px;
  transition: max-height 0.3s ease, opacity 0.2s;
  opacity: 1;
}

.config-group.collapsed .config-group-body {
  max-height: 0;
  opacity: 0;
}

/* Group selector in editor */
.group-selector {
  display: flex;
  gap: 6px;
  align-items: center;
}

.group-selector select {
  flex: 1;
}

/* ─── Editor ─── */
.cicd-editor {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.editor-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  color: oklch(var(--bc) / 0.6);
}

.editor-empty-icon { opacity: 0.2; }
.editor-empty h3 { margin: 0; font-size: 20px; color: oklch(var(--bc)); }
.editor-empty p { margin: 0; font-size: 14px; }

.editor-content {
  flex: 1;
  display: flex;
  flex-direction: column;
}

/* Editor Header */
.editor-header {
  padding: 20px 24px 0;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
  background: oklch(var(--b1));
}

.editor-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.editor-title {
  margin: 0;
  font-size: 18px;
  font-weight: 700;
  color: oklch(var(--bc));
}

.editor-actions {
  display: flex;
  gap: 8px;
}

/* Pipeline Bar */
.pipeline-bar {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 12px 0;
}

.pipeline-node {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 8px 16px;
  border-radius: 8px;
  background: oklch(var(--b2));
  border: 1px dashed oklch(var(--bc) / 0.1);
  min-width: 80px;
}

.pipeline-node.filled {
  background: oklch(var(--p) / 0.1);
  border: 1px solid oklch(var(--p));
}

.pipeline-node.deploy-node {
  background: oklch(var(--p));
  border: 1px solid oklch(var(--p));
}

.pipeline-node.deploy-node .pipeline-label { color: white; }

.pipeline-icon { font-size: 18px; }
.pipeline-label { font-size: 12px; font-weight: 500; color: oklch(var(--bc)); }

.pipeline-arrow {
  color: oklch(var(--bc) / 0.6);
  font-size: 16px;
  opacity: 0.4;
}

/* Editor Body */
.editor-body {
  display: grid;
  grid-template-columns: 1fr 1fr 1.3fr;
  gap: 16px;
  padding: 20px 24px;
  flex: 1;
}

.editor-column {
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 12px;
  padding: 20px;
}

.column-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 16px;
  font-size: 14px;
  font-weight: 600;
  color: oklch(var(--bc));
  padding-bottom: 12px;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
}

.column-icon { font-size: 18px; }

/* Form */
.form-field {
  margin-bottom: 14px;
}

.form-field label {
  display: block;
  margin-bottom: 5px;
  font-size: 12px;
  font-weight: 500;
  color: oklch(var(--bc) / 0.6);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.form-field .required { color: oklch(var(--er)); text-transform: none; letter-spacing: 0; }

.form-input {
  width: 100%;
  padding: 9px 12px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  font-size: 13px;
  transition: border-color 0.2s;
}

.form-input:focus { outline: none; border-color: oklch(var(--p)); }

/* ─── Version selector next to path input ─── */
.path-input-with-select {
  display: flex;
  gap: 6px;
}
.path-input-with-select .form-input {
  flex: 1;
  min-width: 0;
}

/* ─── Detect button next to path input ─── */
.path-input-with-detect {
  display: flex;
  gap: 6px;
}
.path-input-with-detect .form-input {
  flex: 1;
  min-width: 0;
}
.detect-btn {
  flex-shrink: 0;
  white-space: nowrap;
  gap: 4px;
  font-size: 12px;
  padding: 6px 10px;
}
.detect-btn svg {
  transition: transform 0.3s ease;
}
.detect-btn.spinning svg {
  animation: spin 1s linear infinite;
}

/* Auto-detected input highlight */
.form-input.auto-detected {
  border-color: oklch(var(--su));
  background: rgba(16, 185, 129, 0.05);
}

/* ─── Advanced options collapsible ─── */
.advanced-toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 0;
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
  color: oklch(var(--bc) / 0.6);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  user-select: none;
  transition: color 0.15s;
}
.advanced-toggle:hover {
  color: oklch(var(--p));
}
.advanced-chevron {
  transition: transform 0.2s ease;
  flex-shrink: 0;
}
.advanced-chevron.open {
  transform: rotate(90deg);
}
.advanced-content {
  overflow: hidden;
  animation: slideDown 0.2s ease;
}
@keyframes slideDown {
  from { opacity: 0; max-height: 0; }
  to { opacity: 1; max-height: 500px; }
}
.version-select {
  width: 120px;
  min-width: 80px;
  padding: 9px 8px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  font-size: 12px;
  cursor: pointer;
  transition: border-color 0.2s;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.version-select:focus {
  outline: none;
  border-color: oklch(var(--p));
}

.lib-filter-input {
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
  resize: vertical;
  line-height: 1.6;
}

/* Local Path Group */
.local-path-group {
  display: flex;
  gap: 6px;
}
.local-path-group .form-input {
  flex: 1;
  font-size: 12px;
  color: oklch(var(--bc));
}
.local-path-hint {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  margin-top: 6px;
  font-size: 11px;
  color: oklch(var(--bc) / 0.6);
  line-height: 1.4;
}
.hint-icon { font-size: 14px; flex-shrink: 0; }
.local-path-hint code {
  background: oklch(var(--b2));
  padding: 1px 4px;
  border-radius: 3px;
  font-size: 11px;
  color: oklch(var(--p));
  word-break: break-all;
}
.hint-use-btn {
  background: oklch(var(--p));
  color: white;
  border: none;
  padding: 1px 6px;
  border-radius: 3px;
  font-size: 10px;
  cursor: pointer;
  margin-left: 4px;
}
.hint-use-btn:hover { opacity: 0.9; }

.optional {
  font-size: 11px;
  font-weight: 400;
  color: oklch(var(--bc) / 0.6);
  margin-left: 4px;
}

.form-row { display: flex; gap: 12px; }
.form-row .form-field { flex: 1; }
.form-row .form-field.small { max-width: 90px; }

.build-section-divider {
  display: flex;
  align-items: center;
  gap: 10px;
  margin: 8px 0 14px;
  font-size: 11px;
  font-weight: 600;
  color: oklch(var(--bc) / 0.6);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  opacity: 0.7;
}
.build-section-divider::after {
  content: '';
  flex: 1;
  height: 1px;
  background: oklch(var(--bc) / 0.1);
}

/* ─── Server List ─── */
.server-count-badge {
  margin-left: auto;
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 10px;
  background: oklch(var(--p) / 0.1);
  color: oklch(var(--p));
  font-weight: 700;
}

.server-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: calc(100vh - 400px);
  overflow-y: auto;
  padding-right: 4px;
}

.server-entry {
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 10px;
  background: oklch(var(--b2));
  cursor: pointer;
  transition: all 0.2s;
}

.server-entry:hover {
  border-color: oklch(var(--p));
}

.server-entry.active {
  border-color: oklch(var(--p));
  background: oklch(var(--b1));
  box-shadow: 0 2px 8px rgba(46, 171, 124, 0.08);
}

.server-entry-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
}

.server-index {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: oklch(var(--p));
  color: white;
  font-size: 11px;
  font-weight: 700;
  flex-shrink: 0;
}

.server-entry-label {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.server-label-input {
  border: none;
  background: transparent;
  font-size: 13px;
  font-weight: 600;
  color: oklch(var(--bc));
  width: 100%;
  min-width: 0;
  outline: none;
}

.server-label-input::placeholder {
  color: oklch(var(--bc) / 0.6);
  font-weight: 400;
}

.server-remove-btn {
  background: none;
  border: none;
  cursor: pointer;
  padding: 4px;
  border-radius: 6px;
  color: oklch(var(--bc) / 0.6);
  opacity: 0;
  transition: all 0.15s;
  flex-shrink: 0;
}

.server-entry:hover .server-remove-btn { opacity: 0.6; }
.server-remove-btn:hover { opacity: 1 !important; color: oklch(var(--er)); background: rgba(239, 68, 68, 0.1); }

.server-entry-body {
  padding: 0 12px 12px;
}

.add-server-btn {
  margin-top: 8px;
  border: 1px dashed oklch(var(--bc) / 0.1);
  color: oklch(var(--bc) / 0.6);
}

.add-server-btn:hover {
  border-color: oklch(var(--p));
  color: oklch(var(--p));
  background: oklch(var(--p) / 0.1);
}

.test-mini {
  margin-top: 8px;
  padding: 6px 10px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
}
.test-mini.success { background: rgba(34, 197, 94, 0.1); color: #16a34a; }
.test-mini.error { background: rgba(239, 68, 68, 0.1); color: #dc2626; }

.btn-full { width: 100%; }

/* Git Group */
.git-group { display: flex; gap: 6px; }
.git-group .form-input { flex: 1; }

/* Deploy Source Tabs */
.deploy-source-tabs {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.source-tab {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border: 1.5px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  background: oklch(var(--b2));
  cursor: pointer;
  transition: all 0.15s;
  text-align: left;
  font-size: 13px;
}
.source-tab:hover { border-color: oklch(var(--p)); }
.source-tab.active {
  border-color: oklch(var(--p));
  background: oklch(var(--p) / 0.1);
}
.source-tab-icon { font-size: 16px; flex-shrink: 0; }
.source-tab-label { font-weight: 600; color: oklch(var(--bc)); flex-shrink: 0; min-width: 70px; }
.source-tab-path {
  flex: 1;
  font-family: 'SF Mono', monospace;
  font-size: 11px;
  color: oklch(var(--bc) / 0.6);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.source-tab.active .source-tab-path { color: oklch(var(--p)); }

/* Branch Select Group */
.branch-select-group {
  position: relative;
  display: flex;
  align-items: center;
  gap: 6px;
}

.branch-select-group > svg {
  position: absolute;
  left: 10px;
  color: oklch(var(--bc) / 0.6);
  pointer-events: none;
  z-index: 1;
}

.branch-select {
  flex: 1;
  padding-left: 32px;
  padding-right: 32px;
  cursor: pointer;
}

.branch-select:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.branch-refresh {
  padding: 6px;
  min-width: 32px;
  height: 32px;
}

.branch-refresh svg {
  transition: transform 0.3s ease;
}

.branch-refresh.spinning svg {
  animation: spin 1s linear infinite;
}

.branch-refresh:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.branch-hint {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 6px;
  padding: 4px 10px;
  font-size: 12px;
  color: oklch(var(--bc) / 0.6);
}

.branch-hint .hint-icon {
  flex-shrink: 0;
}

@keyframes spin {
  from { transform: rotate(0); }
  to { transform: rotate(360deg); }
}

/* Project Info Card */
.project-info-card {
  margin-top: 16px;
  padding: 12px;
  background: oklch(var(--b2));
  border-radius: 8px;
  border: 1px solid oklch(var(--bc) / 0.1);
}

.info-row {
  display: flex;
  gap: 8px;
  font-size: 12px;
  margin-bottom: 6px;
}

.info-row:last-child { margin-bottom: 0; }
.info-label { color: oklch(var(--bc) / 0.6); flex-shrink: 0; min-width: 60px; }
.info-value { color: oklch(var(--bc)); word-break: break-all; }

/* Server Quick Select */
.server-quick-select { margin-bottom: 14px; }
.server-quick-select label {
  display: block;
  margin-bottom: 8px;
  font-size: 12px;
  font-weight: 500;
  color: oklch(var(--bc) / 0.6);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.server-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.server-chip {
  padding: 6px 12px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 6px;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  font-size: 12px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 6px;
  transition: all 0.15s;
}

.server-chip:hover { border-color: oklch(var(--p)); }
.server-chip.active { border-color: oklch(var(--p)); background: oklch(var(--p) / 0.1); }

.server-chip-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: oklch(var(--su));
}

/* Auth Toggle */
.auth-toggle { display: flex; gap: 8px; }

.auth-btn {
  flex: 1;
  padding: 8px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 6px;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  cursor: pointer;
  font-size: 12px;
  transition: all 0.15s;
}

.auth-btn.active { border-color: oklch(var(--p)); background: oklch(var(--p) / 0.1); color: oklch(var(--p)); }

/* Test Result */
.test-result-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  border-radius: 8px;
  font-size: 13px;
  margin-top: 12px;
}

.test-result-banner.success { background: rgba(16, 185, 129, 0.1); color: oklch(var(--su)); border: 1px solid rgba(16, 185, 129, 0.2); }
.test-result-banner.error { background: rgba(239, 68, 68, 0.1); color: oklch(var(--er)); border: 1px solid rgba(239, 68, 68, 0.2); }

/* Build Tool Grid */
.build-tool-grid {
  display: grid;
  grid-template-columns: 1fr 1fr 1.3fr;
  gap: 8px;
  margin-bottom: 16px;
}

.build-tool-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 12px 8px;
  border: 2px solid oklch(var(--bc) / 0.1);
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.15s;
  position: relative;
}

.build-tool-card:hover { border-color: oklch(var(--p)); }
.build-tool-card.selected { border-color: oklch(var(--p)); background: oklch(var(--p) / 0.1); }

.build-tool-card .tool-icon { font-size: 22px; margin-bottom: 4px; }
.build-tool-card .tool-name { font-size: 12px; font-weight: 600; color: oklch(var(--bc)); }
.build-tool-card .tool-version { font-size: 10px; color: oklch(var(--bc) / 0.6); margin-top: 2px; }

/* Deploy Settings */
.deploy-settings {
  padding-top: 14px;
  border-top: 1px solid oklch(var(--bc) / 0.1);
}

/* Form Checkbox */
.form-checkbox {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: oklch(var(--bc));
  cursor: pointer;
  margin-top: 8px;
}

.form-checkbox input { accent-color: oklch(var(--p)); }

/* Approval checkbox */
.form-checkbox-approval {
  padding: 8px 10px;
  background: rgba(245, 158, 11, 0.06);
  border-radius: 6px;
  border: 1px solid rgba(245, 158, 11, 0.15);
}

.approval-label {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.approval-desc {
  font-size: 11px;
  color: var(--text-secondary);
}

/* Parent Build Section */
.parent-build-section {
  padding: 0 24px 12px;
  border-left: 3px solid transparent;
  transition: border-color 0.3s, background 0.3s;
}
.parent-build-section.active {
  border-left-color: oklch(var(--p));
  background: color-mix(in srgb, oklch(var(--p)) 5%, transparent);
  margin: 0 12px;
  padding: 12px;
  border-radius: 8px;
}
.parent-build-section.auto-detected {
  border-left-color: oklch(var(--su));
  background: color-mix(in srgb, oklch(var(--su)) 5%, transparent);
}
.parent-build-toggle {
  margin-bottom: 8px;
}
.toggle-label {
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
  user-select: none;
}
.toggle-input {
  display: none;
}
.toggle-switch {
  width: 44px;
  height: 24px;
  background: oklch(var(--bc) / 0.6);
  border-radius: 12px;
  position: relative;
  transition: background 0.3s;
  flex-shrink: 0;
}
.toggle-switch::after {
  content: '';
  position: absolute;
  width: 18px;
  height: 18px;
  background: white;
  border-radius: 50%;
  top: 3px;
  left: 3px;
  transition: transform 0.3s;
}
.toggle-input:checked + .toggle-switch {
  background: oklch(var(--p));
}
.toggle-input:checked + .toggle-switch::after {
  transform: translateX(20px);
}
.toggle-text {
  font-size: 14px;
  font-weight: 500;
  color: oklch(var(--bc));
}
.toggle-hint {
  display: block;
  font-size: 12px;
  color: oklch(var(--bc) / 0.6);
  margin-top: 4px;
  margin-left: 54px;
}
.auto-detected-badge {
  display: inline-block;
  font-size: 11px;
  font-weight: 600;
  color: oklch(var(--su));
  background: rgba(16, 185, 129, 0.1);
  border: 1px solid rgba(16, 185, 129, 0.3);
  padding: 2px 8px;
  border-radius: 10px;
  margin-left: 10px;
  margin-top: 4px;
}
.parent-build-path-field {
  margin-top: 8px;
  margin-left: 54px;
}
.parent-build-path-group {
  display: flex;
  gap: 6px;
}
.parent-build-path-group .form-input {
  flex: 1;
}

/* Modules Section */
.modules-section {
  padding: 0 24px 24px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 600;
  color: oklch(var(--bc));
}

.section-icon { font-size: 18px; }
.section-hint { font-size: 12px; font-weight: 400; color: oklch(var(--bc) / 0.6); margin-left: 8px; }
.section-hint-inline {
  font-size: 11px;
  font-weight: 400;
  color: oklch(var(--p));
  margin-left: 8px;
}

.parent-build-badge {
  font-size: 11px;
  font-weight: 600;
  color: #f59e0b;
  background: rgba(245, 158, 11, 0.1);
  border: 1px solid rgba(245, 158, 11, 0.3);
  padding: 2px 8px;
  border-radius: 10px;
  margin-left: 8px;
  animation: badge-pulse 2s ease-in-out infinite;
}

@keyframes badge-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.7; }
}

.section-actions {
  display: flex;
  gap: 8px;
}

.section-actions .btn svg {
  transition: transform 0.5s linear;
}

.section-actions .btn svg.spinning {
  animation: spin 1s linear infinite;
}

/* Module Tree Dropdown */
.module-tree-dropdown {
  margin-bottom: 12px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 10px;
  background: oklch(var(--b1));
  overflow: hidden;
}

.tree-trigger {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  cursor: pointer;
  font-size: 13px;
  color: oklch(var(--bc));
  transition: background 0.15s;
}

.tree-trigger:hover {
  background: oklch(var(--b2));
}

.tree-arrow {
  margin-left: auto;
  transition: transform 0.2s ease;
  color: oklch(var(--bc) / 0.6);
}

.tree-arrow.open {
  transform: rotate(180deg);
}

.tree-content {
  border-top: 1px solid oklch(var(--bc) / 0.1);
  max-height: 400px;
  overflow-y: auto;
}

.tree-item {
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
}

.tree-item:last-child {
  border-bottom: none;
}

.tree-item-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  cursor: pointer;
  transition: background 0.15s;
}

.tree-item-header:hover {
  background: oklch(var(--b2));
}

.tree-expand {
  transition: transform 0.2s ease;
  color: oklch(var(--bc) / 0.6);
  flex-shrink: 0;
}

.tree-expand.expanded {
  transform: rotate(90deg);
}

.tree-indent {
  width: 14px;
  flex-shrink: 0;
}

.tree-type-badge {
  font-size: 12px;
  flex-shrink: 0;
}

.tree-type-badge.maven { opacity: 0.8; }
.tree-type-badge.npm { opacity: 0.8; }

.tree-name {
  font-weight: 600;
  font-size: 13px;
  color: oklch(var(--bc));
  min-width: 100px;
}

.tree-path {
  font-family: 'SF Mono', 'Fira Code', monospace;
  font-size: 11px;
  color: oklch(var(--bc) / 0.6);
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tree-add-btn {
  padding: 4px 10px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 6px;
  background: transparent;
  color: oklch(var(--p));
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
  flex-shrink: 0;
}

.tree-add-btn:hover {
  background: oklch(var(--p) / 0.1);
  border-color: oklch(var(--p));
}

.tree-add-btn.added {
  background: oklch(var(--p));
  color: white;
  border-color: oklch(var(--p));
  cursor: default;
}

.tree-children {
  /* no background — keep flat look across all levels */
}

/* Depth-based indentation (supports 5 levels: depth 0-4) */
.tree-depth-1 > .tree-item-header { padding-left: 28px; }
.tree-depth-2 > .tree-item-header { padding-left: 48px; }
.tree-depth-3 > .tree-item-header { padding-left: 68px; }
.tree-depth-4 > .tree-item-header { padding-left: 88px; }

.tree-bulk-actions {
  padding: 12px 14px;
  display: flex;
  justify-content: flex-end;
  border-top: 1px solid oklch(var(--bc) / 0.1);
  background: oklch(var(--b2));
}

/* Detected Modules */
.detected-modules {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  background: oklch(var(--p) / 0.1);
  border-radius: 8px;
  border: 1px solid oklch(var(--p));
  margin-bottom: 10px;
}

.detected-label { font-size: 12px; color: oklch(var(--p)); font-weight: 500; white-space: nowrap; }

.detected-chips { display: flex; flex-wrap: wrap; gap: 6px; }

.detected-chip {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 6px;
  background: oklch(var(--b1));
  cursor: pointer;
  font-size: 12px;
  transition: all 0.15s;
}

.detected-chip.selected { border-color: oklch(var(--p)); background: oklch(var(--p) / 0.1); }
.chip-check { accent-color: oklch(var(--p)); }

/* Modules Table */
.modules-table {
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 10px;
  overflow: hidden;
}

.modules-table-header {
  display: grid;
  grid-template-columns: 1.5fr 1.5fr 1.2fr 60px 60px 36px;
  gap: 8px;
  padding: 10px 14px;
  background: oklch(var(--b2));
  font-size: 11px;
  font-weight: 600;
  color: oklch(var(--bc) / 0.6);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

/* Modules Cards */
.modules-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.module-card {
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 10px;
  overflow: hidden;
  background: oklch(var(--b1));
  transition: all 0.15s ease;
}

.module-card:hover {
  border-color: oklch(var(--p));
}

.module-card-header {
  cursor: pointer;
  padding: 10px 14px;
  transition: background 0.15s;
}

.module-card-header:hover {
  background: oklch(var(--b2));
}

.module-title-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.expand-icon {
  transition: transform 0.2s ease;
  color: oklch(var(--bc) / 0.6);
  flex-shrink: 0;
}

.expand-icon.expanded {
  transform: rotate(180deg);
}

.module-index {
  font-size: 12px;
  font-weight: 600;
  color: oklch(var(--bc) / 0.6);
  flex-shrink: 0;
}

.module-name-input {
  flex: 1;
  padding: 6px 10px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 6px;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  font-size: 13px;
  font-weight: 500;
  min-width: 0;
}

.module-name-input:focus {
  border-color: oklch(var(--p));
  outline: none;
  box-shadow: 0 0 0 2px oklch(var(--p) / 0.1);
}

.module-card-body {
  padding: 0 14px 14px;
  border-top: 1px solid oklch(var(--bc) / 0.1);
  background: oklch(var(--b2));
}

.module-fields {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  padding: 12px 0;
}

.module-fields:last-child {
  padding-bottom: 4px;
}

.field-hint {
  display: block;
  font-size: 11px;
  color: oklch(var(--bc) / 0.6);
  margin-top: 4px;
}

.order-input { text-align: center; }

/* Toggle Switch */
.toggle-switch {
  position: relative;
  display: inline-block;
  width: 36px;
  height: 20px;
  flex-shrink: 0;
}

.toggle-switch input { opacity: 0; width: 0; height: 0; }

.toggle-slider {
  position: absolute;
  inset: 0;
  background: oklch(var(--bc) / 0.1);
  border-radius: 20px;
  cursor: pointer;
  transition: 0.2s;
}

.toggle-slider::before {
  content: '';
  position: absolute;
  width: 16px;
  height: 16px;
  left: 2px;
  bottom: 2px;
  background: white;
  border-radius: 50%;
  transition: 0.2s;
}

.toggle-switch input:checked + .toggle-slider { background: oklch(var(--p)); }
.toggle-switch input:checked + .toggle-slider::before { transform: translateX(16px); }

/* Buttons */
.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.btn:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-primary { background: oklch(var(--p)); color: white; }
.btn-primary:hover:not(:disabled) { opacity: 0.9; }
.btn-ghost { background: transparent; border: 1px solid oklch(var(--bc) / 0.1); color: oklch(var(--bc)); }
.btn-danger { background: transparent; color: oklch(var(--er)); }
.btn-danger:hover { background: rgba(239, 68, 68, 0.1); }
.btn-sm { padding: 6px 12px; font-size: 12px; }
.btn-lg { padding: 12px 32px; font-size: 16px; }

/* Group Name Dialog */
.group-dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
}

.group-dialog {
  background: oklch(var(--b2));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 12px;
  padding: 24px;
  width: 360px;
  max-width: 90vw;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
  animation: slideUp 0.2s ease;
}

.group-dialog h4 {
  margin: 0 0 16px;
  font-size: 16px;
  color: oklch(var(--bc));
}

.group-dialog-input {
  width: 100%;
  margin-bottom: 16px;
}

.group-dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes slideUp {
  from { opacity: 0; transform: translateY(20px) scale(0.95); }
  to { opacity: 1; transform: translateY(0) scale(1); }
}
</style>
