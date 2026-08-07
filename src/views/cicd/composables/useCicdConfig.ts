import { getTauriAPI } from '../../../utils/tauri-api'
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue';
import { useToast } from '../../../composables/useToast';
import { useErrorHandler } from '../../../composables/useErrorHandler';
import { useSharedCicdData } from '../../../composables/useSharedCicdData';
import type { Project, Server } from '../../../types';

// ─── Interfaces ───
export interface CicdConfigEntry {
  id: string;
  name?: string;
  gitRepoId?: string;
  deployBranch?: string;
  buildTool?: string;
  updatedAt?: string;
  createdAt?: string;
  localPath?: string;
  repoUrl?: string;
  npmScript?: string;
  npmCustomScript?: string;
  mavenSettings?: string;
  mavenProfile?: string;
  deployPath?: string;
  libSeparate?: boolean;
  restartScript?: string;
  healthCheckUrl?: string;
  healthCheckTimeout?: number;
  servers?: string;
  groupName?: string;
  requiresApproval?: boolean;
  [key: string]: unknown;
}

export interface DeployModule {
  id: string | null;
  configId: string | null;
  moduleName: string;
  modulePath: string;
  artifactName: string;
  artifactType?: string;
  libFilterRules?: string;
  deployOrder: number;
  deployPath?: string;
  enabled: boolean;
  createdAt?: string;
  updatedAt?: string;
  buildCommand?: string;
  buildPath?: string;
  outputPath?: string;
  buildTool?: string;
}

export interface DeployServerEntry {
  serverId: string;
  label: string;
  deployDir: string;
  testResult?: { success: boolean; error?: string } | null;
}

export interface ScannedModule {
  name: string;
  path: string;
  type: string;
  artifactId?: string;
  children?: ScannedModule[];
}

export interface ConfigForm {
  id: string | null;
  name: string;
  localPath: string;
  gitRepoId: string;
  repoUrl: string;
  deployBranch: string;
  buildTool: string;
  npmScript: string;
  npmCustomScript: string;
  mavenSettings: string;
  mavenProfile: string;
  mavenHome: string;
  javaHome: string;
  npmHome: string;
  pnpmHome: string;
  yarnHome: string;
  nodeHome: string;
  deployPath: string;
  libSeparate: boolean;
  restartScript: string;
  healthCheckUrl: string;
  healthCheckTimeout: number;
  groupName: string;
  createdAt?: string;
  updatedAt?: string;
  parentBuildMode: boolean;
  parentBuildPath: string;
  requiresApproval: boolean;
  showAdvanced: boolean;
  buildMode: string;
  buildCommand: string;
}

export function useCicdConfig() {
  const toast = useToast();
  const { handleError } = useErrorHandler();

  // ─── Data ───
  const shared = useSharedCicdData();
  const { configs, projects, servers, serverGroups, gitRepos } = shared;
  const selectedConfigId = ref('');
  const isNewConfig = ref(false);
  const searchQuery = ref('');
  const sidebarCollapsed = ref(false);
  const selectedServerId = ref('');
  const deployServers = ref<DeployServerEntry[]>([makeDefaultServer()]);
  const activeServerIdx = ref(0);

  // Group management
  const groups = ref<string[]>([]);
  const expandedGroups = ref<Set<string>>(new Set());

  // ─── Group Name Dialog ───
  const showGroupDialog = ref(false);
  const groupNameInput = ref('');
  const groupDialogMode = ref<'add' | 'rename'>('add');
  const groupDialogOldName = ref('');
  const groupDialogResolve = ref<((value: string | null) => void) | null>(null);

  function openGroupDialog(mode: 'add' | 'rename', oldName?: string): Promise<string | null> {
    groupDialogMode.value = mode;
    groupDialogOldName.value = oldName || '';
    groupNameInput.value = mode === 'rename' ? (oldName || '') : '';
    showGroupDialog.value = true;
    return new Promise((resolve) => { groupDialogResolve.value = resolve; });
  }

  function confirmGroupDialog() {
    const name = groupNameInput.value.trim();
    showGroupDialog.value = false;
    groupDialogResolve.value?.(name || null);
    groupDialogResolve.value = null;
  }

  function cancelGroupDialog() {
    showGroupDialog.value = false;
    groupDialogResolve.value?.(null);
    groupDialogResolve.value = null;
  }

  const showGroupEditor = ref(false);
  const newGroupName = ref('');

  function initExpandedGroups() { expandedGroups.value = new Set(groups.value); }

  function makeDefaultServer(): DeployServerEntry { return { serverId: '', label: '', deployDir: '' }; }

  function getServerName(id: string): string {
    if (!id) {return '';}
    const s = servers.value.find(srv => srv.id === id);
    if (!s) {return '';}
    const group = s.groupId ? serverGroups.value.find(g => g.id === s.groupId) : null;
    return group ? `${s.name} [${group.name}]` : s.name;
  }

  function onServerSelect(srv: DeployServerEntry) {
    const found = servers.value.find(s => s.id === srv.serverId);
    if (found) {
      const group = found.groupId ? serverGroups.value.find(g => g.id === found.groupId) : null;
      srv.label = group ? `${found.name} [${group.name}]` : found.name;
    }
  }

  function addServer() {
    const existing = deployServers.value.length;
    deployServers.value.push({ serverId: '', label: existing === 0 ? '主节点' : `节点 ${existing + 1}`, deployDir: '' });
    activeServerIdx.value = existing;
  }

  function removeServer(idx: number) {
    if (deployServers.value.length <= 1) {return;}
    deployServers.value.splice(idx, 1);
    if (activeServerIdx.value >= deployServers.value.length) {activeServerIdx.value = deployServers.value.length - 1;}
  }

  async function testServerById(srv: DeployServerEntry) {
    if (!srv.serverId) {return;}
    const srvObj = servers.value.find(s => s.id === srv.serverId);
    if (!srvObj) { srv.testResult = { success: false, error: '服务器未找到' }; return; }
    try {
      const result = await getTauriAPI().testSsh({
        host: srvObj.host, port: srvObj.port, username: srvObj.username,
        sshKeyPath: srvObj.sshKeyPath || undefined, password: srvObj.password || undefined,
      });
      srv.testResult = result;
    } catch (error: unknown) { srv.testResult = { success: false, error: error instanceof Error ? error.message : String(error) }; }
  }

  const testResult = ref<{ success: boolean; error?: string } | null>(null);
  const detectedTools = ref<Record<string, { available: boolean; version?: string; path?: string }>>({});
  const availableBranches = ref<string[]>([]);
  const loadingBranches = ref(false);
  const expandedModules = ref<number[]>([]);
  const scannedModules = ref<ScannedModule[]>([]);
  const scanningModules = ref(false);
  const showModuleTree = ref(false);
  const expandedTreeNodes = ref<string[]>([]);

  function defaultConfig(): ConfigForm {
    return {
      id: null, name: '', localPath: '', gitRepoId: '', repoUrl: '', deployBranch: 'main',
      buildTool: '', npmScript: 'build', npmCustomScript: '', mavenSettings: '', mavenProfile: 'prod',
      mavenHome: '', javaHome: '', npmHome: '', pnpmHome: '', yarnHome: '', nodeHome: '', deployPath: '', libSeparate: true,
      restartScript: './restart.sh', healthCheckUrl: '', healthCheckTimeout: 30, groupName: '未分组',
      parentBuildMode: false, parentBuildPath: '', requiresApproval: false, showAdvanced: false, buildMode: 'local', buildCommand: '',
    };
  }

  const config = ref<ConfigForm>(defaultConfig());
  const modules = ref<DeployModule[]>([]);
  const defaultPaths = ref<{ mavenHome: string; javaHome: string; nodeHome: string; npmHome: string; pnpmHome: string; yarnHome: string }>({
    mavenHome: '', javaHome: '', nodeHome: '', npmHome: '', pnpmHome: '', yarnHome: '',
  });
  const sdkVersions = ref<{
    sdkman: { java: { name: string; path: string; isCurrent?: boolean }[]; maven: { name: string; path: string; isCurrent?: boolean }[]; gradle: { name: string; path: string; isCurrent?: boolean }[] }
    nvm: { node: { name: string; path: string; isCurrent?: boolean; npm?: string; pnpm?: string; yarn?: string }[] }
  }>({ sdkman: { java: [], maven: [], gradle: [] }, nvm: { node: [] } });
  const selectedJavaVersion = ref('');
  const selectedNodeVersion = ref('');
  const detectingPaths = ref(false);

  // 平台检测 & SDK 安装指引
  const platform = ref<'mac' | 'linux' | 'win'>('linux');
  if (typeof navigator !== 'undefined') {
    const ua = navigator.userAgent.toLowerCase();
    if (ua.includes('mac')) {platform.value = 'mac';}
    else if (ua.includes('win')) {platform.value = 'win';}
  }
  const sdkmanInstallGuide = computed(() => ({
    show: sdkVersions.value.sdkman?.java?.length === 0,
    steps: platform.value === 'mac'
      ? ['curl -s "https://get.sdkman.io" | bash', 'source "$HOME/.sdkman/bin/sdkman-init.sh"', 'sdk install java']
      : platform.value === 'win'
        ? ['# SDKMAN 不支持 Windows 原生运行，推荐使用 WSL', '# 或在 WSL 中执行：', 'curl -s "https://get.sdkman.io" | bash', 'source "$HOME/.sdkman/bin/sdkman-init.sh"', 'sdk install java']
        : ['curl -s "https://get.sdkman.io" | bash', 'source "$HOME/.sdkman/bin/sdkman-init.sh"', 'sdk install java'],
  }));
  const nvmInstallGuide = computed(() => ({
    show: sdkVersions.value.nvm?.node?.length === 0,
    steps: platform.value === 'mac'
      ? ['curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash', 'source "$HOME/.nvm/nvm.sh"', 'nvm install --lts']
      : platform.value === 'win'
        ? ['# Windows 推荐使用 nvm-windows', '# 下载安装: https://github.com/coreybutler/nvm-windows/releases', 'nvm install lts', 'nvm use lts']
        : ['curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash', 'source "$HOME/.nvm/nvm.sh"', 'nvm install --lts'],
  }));

  async function reDetectToolPaths() {
    detectingPaths.value = true;
    try {
      // 并行检测工具路径和 SDK 版本
      const [pathsResult, sdkResult] = await Promise.all([
        getTauriAPI().detectToolPaths?.().catch(() => null),
        getTauriAPI().detectSdkVersions?.().catch(() => null),
      ]);
      
      // 更新工具路径
      if (pathsResult && typeof pathsResult === 'object') {
        Object.assign(defaultPaths.value, pathsResult as typeof defaultPaths.value);
        // Always fill current config with detected paths — overwrite whatever was there
        const c = config.value;
        if (c.buildTool === 'maven' && defaultPaths.value.mavenHome) {c.mavenHome = defaultPaths.value.mavenHome;}
        if (c.buildTool === 'maven' && defaultPaths.value.javaHome) {c.javaHome = defaultPaths.value.javaHome;}
        if ((c.buildTool === 'npm' || c.buildTool === 'pnpm' || c.buildTool === 'yarn') && defaultPaths.value.nodeHome) {c.nodeHome = defaultPaths.value.nodeHome;}
        if (c.buildTool === 'npm' && defaultPaths.value.npmHome) {c.npmHome = defaultPaths.value.npmHome;}
        if (c.buildTool === 'pnpm' && defaultPaths.value.pnpmHome) {c.pnpmHome = defaultPaths.value.pnpmHome;}
        if (c.buildTool === 'yarn' && defaultPaths.value.yarnHome) {c.yarnHome = defaultPaths.value.yarnHome;}
      }
      
      // 更新 SDK 版本列表
      if (sdkResult && typeof sdkResult === 'object') {
        sdkVersions.value = {
          ...sdkResult,
          sdkman: { java: [], maven: [], gradle: [], ...(sdkResult as any)?.sdkman },
          nvm: { node: [], ...(sdkResult as any)?.nvm },
        };
      }
      
      toast.success('工具路径已自动检测并填充');
    } catch (error) { handleError(error, { context: '检测工具路径' }); }
    finally { detectingPaths.value = false; }
  }

  function onJavaVersionSelected() { if (selectedJavaVersion.value) {config.value.javaHome = selectedJavaVersion.value;} }
  function onNodeVersionSelected() {
    if (selectedNodeVersion.value) {
      const nodeEntry = sdkVersions.value.nvm.node.find(
        (v: { name: string; path: string }) => v.path === selectedNodeVersion.value
      );
      if (nodeEntry) {
        config.value.nodeHome = nodeEntry.path;
        if (nodeEntry.npm) {config.value.npmHome = nodeEntry.npm;}
        if (nodeEntry.pnpm) {config.value.pnpmHome = nodeEntry.pnpm;}
        if (nodeEntry.yarn) {config.value.yarnHome = nodeEntry.yarn;}
      }
    }
  }

  // ─── Computed ───
  const filteredConfigs = computed(() => {
    let result = configs.value;
    if (searchQuery.value) {
      const q = searchQuery.value.toLowerCase();
      result = configs.value.filter(c =>
        (c.name || getGitRepoName(c.gitRepoId)).toLowerCase().includes(q) || c.groupName?.toLowerCase().includes(q)
      );
    }
    return [...result].sort((a, b) => {
      const aTime = (a as Record<string, string>).lastDeployedAt || '';
      const bTime = (b as Record<string, string>).lastDeployedAt || '';
      if (aTime && bTime) {return bTime.localeCompare(aTime);}
      if (aTime) {return -1;} if (bTime) {return 1;} return 0;
    });
  });

  const groupedConfigs = computed(() => {
    const filtered = filteredConfigs.value;
    const map = new Map<string, CicdConfigEntry[]>();
    for (const cfg of filtered) {
      const group = cfg.groupName || '未分组';
      if (!map.has(group)) {map.set(group, []);}
      map.get(group)!.push(cfg);
    }
    return map;
  });

  const hasAnyGitSource = computed(() => selectedGitRepo.value && (selectedGitRepo.value.gitUrl1 || selectedGitRepo.value.gitUrl2 || selectedGitRepo.value.repoPath || selectedGitRepo.value.repoPath2));
  const gitSources = computed(() => {
    const p = selectedGitRepo.value; if (!p) {return [];}
    const sources: { key: string; label: string; icon: string; url: string; path: string }[] = [];
    if (p.gitUrl1) {sources.push({ key: 'remote1', label: '远程仓库 1', icon: '🌐', url: p.gitUrl1, path: p.gitUrl1.split('/').pop() || p.gitUrl1 });}
    if (p.gitUrl2) {sources.push({ key: 'remote2', label: '远程仓库 2', icon: '🌐', url: p.gitUrl2, path: p.gitUrl2.split('/').pop() || p.gitUrl2 });}
    if (p.repoPath) {sources.push({ key: 'local1', label: '本地仓库 1', icon: '📂', url: p.repoPath, path: p.repoPath.split('/').pop() || p.repoPath });}
    if (p.repoPath2) {sources.push({ key: 'local2', label: '本地仓库 2', icon: '📂', url: p.repoPath2, path: p.repoPath2.split('/').pop() || p.repoPath2 });}
    return sources;
  });
  const projectShortName = computed(() => selectedGitRepo.value?.name?.toLowerCase().replace(/[^a-z0-9]/g, '-').replace(/-+/g, '-').slice(0, 30) || 'app');

  const buildToolDefs = [
    { key: 'maven', name: 'Maven', icon: '🔶' }, { key: 'npm', name: 'npm', icon: '🔴' },
    { key: 'pnpm', name: 'pnpm', icon: '🟢' }, { key: 'yarn', name: 'Yarn', icon: '🔵' },
    { key: 'gradle', name: 'Gradle', icon: '🟠' }, { key: 'cargo', name: 'Cargo', icon: '🦀' },
  ];
  const availableBuildTools = computed(() => buildToolDefs.map(td => {
    const det = detectedTools.value[td.key];
    return { ...td, version: det?.version, path: det?.path, available: det?.available ?? false };
  }));
  const addedModulePaths = computed(() => new Set(modules.value.map(m => m.modulePath || m.buildPath || '')));

  // Parent-build auto-detection state
  const parentBuildAutoDetected = ref(false);
  const parentBuildDetectedPath = computed(() => {
    // Find the root parent module from scanned modules
    for (const mod of scannedModules.value) {
      if (mod.children && mod.children.length > 0) {
        return mod.path && mod.path !== '.' ? mod.path : '';
      }
    }
    return '';
  });

  // 当前选中的 Git 仓库对象（用于支持 "本地项目目录" 和分支编辑）
  const selectedGitRepo = computed(() => {
    if (config.value.gitRepoId) {
      return gitRepos.value.find((r: any) => r.id === config.value.gitRepoId) || null;
    }
    return null;
  });

  // ─── UI Helpers ───
  function getGitRepoName(id?: string) {
    if (!id) {return '';}
    const repo = gitRepos.value.find((r: any) => r.id === id);
    return repo ? repo.name : '';
  }
  function getProjectName(_projectId?: string) { return getGitRepoName(config.value?.gitRepoId) || '项目 ?'; }
  function getToolBadge(tool?: string) { const icons: Record<string, string> = { maven: '🔶', npm: '🔴', pnpm: '🟢', yarn: '🔵', gradle: '🟠', cargo: '🦀' }; return icons[tool || ''] || ''; }
  function getBuildToolIcon(tool?: string) { const icons: Record<string, string> = { maven: '🔶', npm: '🔴', pnpm: '🟢', yarn: '🔵', gradle: '🟠', cargo: '🦀' }; return icons[tool || ''] || '🛠️'; }
  function getBuildToolName(tool?: string) { const names: Record<string, string> = { maven: 'Maven', npm: 'npm', pnpm: 'pnpm', yarn: 'Yarn', gradle: 'Gradle', cargo: 'Cargo' }; return names[tool || ''] || ''; }
  function formatTime(iso?: string) {
    if (!iso) {return '';} const d = new Date(iso); const now = new Date(); const diff = now.getTime() - d.getTime();
    if (diff < 60000) {return '刚刚';} if (diff < 3600000) {return Math.floor(diff / 60000) + '分钟前';}
    if (diff < 86400000) {return Math.floor(diff / 3600000) + '小时前';} return d.toLocaleDateString('zh-CN');
  }

  // ─── Group Management ───
  function toggleGroup(name: string) { if (expandedGroups.value.has(name)) {expandedGroups.value.delete(name);} else {expandedGroups.value.add(name);} }

  async function renameGroup(oldName: string) {
    const newName = await openGroupDialog('rename', oldName);
    if (!newName || newName === oldName) {return;}
    const updated = configs.value.filter(c => c.groupName === oldName);
    for (const cfg of updated) {cfg.groupName = newName;}
    Promise.all(updated.map(cfg => getTauriAPI().updateCicdConfig({ ...cfg, updatedAt: new Date().toISOString() })))
      .then(() => { loadConfigs(); toast.success(`分组已重命名为 "${newName}"`); })
      .catch(err => handleError(err, { context: '重命名分组' }));
  }

  async function addGroup() {
    const name = await openGroupDialog('add');
    if (!name) {return;}
    if (groups.value.includes(name)) { toast.error('分组已存在'); return; }
    groups.value.push(name); config.value.groupName = name; expandedGroups.value.add(name);
    toast.success(`分组 "${name}" 已创建，保存配置后生效`);
  }

  function getServerLabel(cfg: CicdConfigEntry): string {
    if (cfg.servers) {
      try {
        const parsed = JSON.parse(cfg.servers);
        if (Array.isArray(parsed) && parsed.length > 0) {
          const names = parsed.map((s: { serverId?: string; label?: string }) => getServerName(s.serverId || '') || s.label).filter(Boolean);
          if (names.length > 0) {return names.join(', ');}
        }
      } catch (e) { console.warn('[CICD] 解析服务器列表失败', e); }
    }
    return '未配置服务器';
  }

  // ─── Config CRUD ───
  async function loadConfigs() {
    configs.value = (await getTauriAPI().getCicdConfigs()) as CicdConfigEntry[];
    groups.value = await getTauriAPI().getCicdGroups() as string[];
    initExpandedGroups();
  }

  function createNewConfig() {
    // 立即显示新建配置界面（不阻塞）
    isNewConfig.value = true; selectedConfigId.value = ''; config.value = defaultConfig();
    modules.value = []; selectedServerId.value = ''; testResult.value = null;
    deployServers.value = [makeDefaultServer()]; activeServerIdx.value = 0;
    config.value.localPath = ''; availableBranches.value = [];
    scannedModules.value = []; showModuleTree.value = false; expandedTreeNodes.value = [];

    // 异步检测工具路径和SDK版本（不阻塞界面显示）
    runToolDetection();
  }

  // 工具检测：只在用户点击新建配置时才执行，异步填充
  async function runToolDetection() {
    // 并行执行所有检测
    const [toolsResult, pathsResult, sdkResult] = await Promise.all([
      getTauriAPI().detectBuildTools?.().catch(() => null),
      getTauriAPI().detectToolPaths?.().catch(() => null),
      getTauriAPI().detectSdkVersions?.().catch(() => null),
    ]);

    // 填充检测结果
    if (toolsResult && typeof toolsResult === 'object') {
      detectedTools.value = toolsResult as unknown as Record<string, { available: boolean; version?: string }>;
      // 自动选择构建工具
      if (!config.value.buildTool) {
        if (detectedTools.value.maven?.available) {config.value.buildTool = 'maven';}
        else if (detectedTools.value.npm?.available) {config.value.buildTool = 'npm';}
        else if (detectedTools.value.pnpm?.available) {config.value.buildTool = 'pnpm';}
      }
    }

    if (pathsResult && typeof pathsResult === 'object') {
      defaultPaths.value = pathsResult as typeof defaultPaths.value;
      // 填充默认路径
      config.value.mavenHome = defaultPaths.value.mavenHome || '';
      config.value.javaHome = defaultPaths.value.javaHome || '';
      config.value.npmHome = defaultPaths.value.npmHome || '';
      config.value.pnpmHome = defaultPaths.value.pnpmHome || '';
      config.value.yarnHome = defaultPaths.value.yarnHome || '';
      config.value.nodeHome = defaultPaths.value.nodeHome || '';
    }

    if (sdkResult && typeof sdkResult === 'object') {
      sdkVersions.value = {
        ...sdkResult,
        sdkman: { java: [], maven: [], gradle: [], ...(sdkResult as any)?.sdkman },
        nvm: { node: [], ...(sdkResult as any)?.nvm },
      };
      // 选择当前 SDK 版本
      const currentJava = sdkVersions.value.sdkman?.java?.find?.((v: { isCurrent?: boolean }) => v.isCurrent);
      if (currentJava && !config.value.javaHome) {
        config.value.javaHome = currentJava.path;
        selectedJavaVersion.value = currentJava.path;
      }
      const currentNode = sdkVersions.value.nvm?.node?.find?.((v: { isCurrent?: boolean; npm?: string; pnpm?: string; yarn?: string }) => v.isCurrent);
      if (currentNode && !config.value.nodeHome) {
        config.value.nodeHome = currentNode.path;
        selectedNodeVersion.value = currentNode.path;
        if (!config.value.npmHome && currentNode.npm) {config.value.npmHome = currentNode.npm;}
        if (!config.value.pnpmHome && currentNode.pnpm) {config.value.pnpmHome = currentNode.pnpm;}
        if (!config.value.yarnHome && currentNode.yarn) {config.value.yarnHome = currentNode.yarn;}
      }
    }

    // 设置默认部署路径
    if (config.value.buildTool === 'maven') {config.value.deployPath = '~/apphome';}
    else if (['npm', 'pnpm', 'yarn'].includes(config.value.buildTool)) {config.value.deployPath = '/home/nginxWebUI/ui';}
  }

  function selectConfig(id: string) { isNewConfig.value = false; selectedConfigId.value = id; loadConfig(id); }

  function onProjectChange() {
    availableBranches.value = []; scannedModules.value = []; showModuleTree.value = false; expandedTreeNodes.value = [];
    const repo = selectedGitRepo.value;
    if (repo?.path && !config.value.localPath) {config.value.localPath = repo.path;}
    if (repo?.branch && !config.value.deployBranch) {config.value.deployBranch = repo.branch;}
    const localPath = config.value.localPath || repo?.path;
    if (localPath) {scanLocalProject(localPath);}
    if (config.value.repoUrl || config.value.localPath) {loadBranches();}
  }

  function onGitRepoChange() {
    // 用户切换 Git 仓库后，自动补填本地路径
    const repo = gitRepos.value.find((r: any) => r.id === config.value.gitRepoId);
    if (repo) {
      if (repo.path) {config.value.localPath = repo.path;}
      if (repo.branch && !config.value.deployBranch) {config.value.deployBranch = repo.branch;}
      const path = repo.path || config.value.localPath;
      if (path) {
        scanLocalProject(path);
        loadBranches();
      }
    }
  }

  async function selectLocalDir() {
    try {
      const result = await getTauriAPI().showOpenDialogForDirs();
      if (!result.canceled && result.filePaths.length > 0) {
        config.value.localPath = result.filePaths[0];
        await scanLocalProject(config.value.localPath);
      }
    } catch (error) { handleError(error, { context: '选择目录' }); }
  }

  // 智能扫描本地项目，自动填充所有可识别字段
  async function scanLocalProject(localPath: string) {
    if (!localPath) {return;}
    try {
      const scan = await getTauriAPI().scanProject(localPath);
      if (!scan || Object.keys(scan).length === 0) {return;}

      // 配置名称（从项目名推导）
      if (scan.projectName && !config.value.name) {config.value.name = scan.projectName;}

      // Git 远程仓库
      if (scan.gitRemoteUrl && !config.value.repoUrl) {config.value.repoUrl = scan.gitRemoteUrl;}

      // 构建工具
      if (scan.buildTool) {
        config.value.buildTool = scan.buildTool;
      }

      // 部署分支
      if (scan.currentBranch && !config.value.deployBranch) {config.value.deployBranch = scan.currentBranch;}

      // npm 脚本
      if (scan.recommendedScript && !config.value.npmScript) {config.value.npmScript = scan.recommendedScript;}

      // 包管理器
      if (scan.packageManager && config.value.buildTool === 'npm') {
        config.value.buildTool = scan.packageManager;
      }

      // Maven Profile
      if (scan.recommendedProfile && !config.value.mavenProfile) {config.value.mavenProfile = scan.recommendedProfile;}

      // 部署路径
      if (scan.suggestedDeployPath && (!config.value.deployPath || ['~/apphome', '/home/nginxWebUI/ui'].includes(config.value.deployPath))) {
        config.value.deployPath = scan.suggestedDeployPath;
      }

      // 多模块检测
      if (scan.isMultiModule && scan.moduleNames) {
        config.value.parentBuildMode = true;
        config.value.parentBuildPath = localPath;
        for (const modName of scan.moduleNames) {
          if (!modules.value.some(m => m.moduleName === modName)) {
            modules.value.push({ id: null, configId: config.value.id || '', moduleName: modName, modulePath: modName, createdAt: new Date().toISOString() } as DeployModule);
          }
        }
      }
    } catch (e) { console.warn('[CICD] 本地项目扫描异常', e); }
  }

  function selectServer(srv: Server) {
    selectedServerId.value = srv.id;
    if (deployServers.value.length === 0) { deployServers.value = [makeDefaultServer()]; activeServerIdx.value = 0; }
    deployServers.value[0].serverId = srv.id;
  }

  function copyGitUrl() { if (config.value.repoUrl) { navigator.clipboard?.writeText(config.value.repoUrl).catch(() => {}); toast.success('Git 地址已复制'); } }

  async function loadBranches() {
    const repoPath = config.value.localPath || selectedGitRepo.value?.path;
    const gitUrl = config.value.repoUrl;
    // 本地路径优先；没有本地路径时用远程 URL（后端通过 git ls-remote 拉取）
    const path = repoPath || gitUrl;
    if (!path) {return;}
    loadingBranches.value = true;
    try {
      const branches = await getTauriAPI().getGitBranches(path);
      availableBranches.value = (branches?.branches || branches || []).map((b: any) => typeof b === 'string' ? b : b.name);
      if (config.value.deployBranch && !availableBranches.value.includes(config.value.deployBranch)) {
        if (selectedGitRepo.value?.branch) {availableBranches.value.push(selectedGitRepo.value.branch);}
      }
    } catch (error) { console.error('Failed to load branches:', error); availableBranches.value = []; }
    finally { loadingBranches.value = false; }
  }

  async function testConnection() {
    testResult.value = null;
    // 头部「测试连接」：测第一个已配置 serverId 的部署服务器，而非整个配置对象
    const target = deployServers.value.find(s => s.serverId);
    if (!target) { toast.warning('请先在「部署服务器」中添加服务器'); return; }
    await testServerById(target);
    if (target.testResult?.success) { toast.success('SSH 连接测试成功'); }
    else { toast.error('连接失败: ' + (target.testResult?.error ?? '未知错误')); }
  }

  // ─── Module Management ───
  function addModule() {
    modules.value.push({
      id: null, configId: config.value.id, moduleName: '', modulePath: '', artifactName: '',
      deployOrder: modules.value.length, enabled: true, buildCommand: '', buildPath: '', outputPath: '', buildTool: '',
      artifactType: '', deployPath: '',
    });
    expandedModules.value.push(modules.value.length - 1);
  }

  function toggleModuleExpand(idx: number) {
    const pos = expandedModules.value.indexOf(idx);
    if (pos >= 0) {expandedModules.value.splice(pos, 1);} else {expandedModules.value.push(idx);}
  }

  async function scanModules() {
    const projectPath = config.value.localPath || selectedGitRepo.value?.path;
    if (!projectPath) { toast.error('请先选择有本地路径的项目'); return; }
    scanningModules.value = true;
    try {
      const result = await getTauriAPI().scanProjectModules(projectPath) as { success: boolean; modules?: ScannedModule[]; error?: string };
      if (result && result.success) {
        scannedModules.value = result.modules || []; showModuleTree.value = scannedModules.value.length > 0; expandedTreeNodes.value = [];
        if (scannedModules.value.length > 0) {
          toast.success(`识别到 ${scannedModules.value.length} 个模块`);
          autoDetectParentBuild();
        } else {toast.info('未发现可识别的模块');}
      } else { toast.error(result?.error || '扫描失败'); }
    } catch (error) { handleError(error, { context: '扫描项目模块' }); }
    finally { scanningModules.value = false; }
  }

  function toggleTreeNode(nodePath: string) {
    const pos = expandedTreeNodes.value.indexOf(nodePath);
    if (pos >= 0) {expandedTreeNodes.value.splice(pos, 1);} else {expandedTreeNodes.value.push(nodePath);}
  }

  function isModuleAlreadyAdded(modPath: string): boolean { return modules.value.some(m => m.modulePath === modPath || m.buildPath === modPath); }

  function addModuleFromScan(mod: ScannedModule) {
    if (isModuleAlreadyAdded(mod.path)) {return;}
    const isParentBuild = config.value.parentBuildMode;
    modules.value.push({
      id: null, configId: config.value.id, moduleName: mod.name || mod.path, modulePath: mod.path || '',
      artifactName: mod.artifactId || '', artifactType: mod.type === 'maven' ? 'jar-plus-lib' : mod.type === 'npm' ? 'dist' : '',
      deployOrder: modules.value.length, enabled: true, buildCommand: '',
      buildPath: isParentBuild ? '' : (mod.path || ''), outputPath: mod.type === 'maven' ? 'target' : 'dist',
      buildTool: mod.type === 'unknown' ? '' : mod.type, deployPath: '',
    });
    expandedModules.value.push(modules.value.length - 1);
    toast.success(`已添加模块: ${mod.name || mod.path}`);
  }

  function addAllDetectedModules() {
    let addedCount = 0; const allModules = flattenModuleTree(scannedModules.value);
    const isParentBuild = config.value.parentBuildMode;
    for (const mod of allModules) {
      if (!isModuleAlreadyAdded(mod.path)) {
        modules.value.push({
          id: null, configId: config.value.id, moduleName: mod.name || mod.path, modulePath: mod.path || '',
          artifactName: mod.artifactId || '', artifactType: mod.type === 'maven' ? 'jar-plus-lib' : mod.type === 'npm' ? 'dist' : '',
          deployOrder: modules.value.length, enabled: true, buildCommand: '',
          buildPath: isParentBuild ? '' : (mod.path || ''), outputPath: mod.type === 'maven' ? 'target' : 'dist',
          buildTool: mod.type === 'unknown' ? '' : mod.type, deployPath: '',
        }); addedCount++;
      }
    }
    if (addedCount > 0) {toast.success(`已添加 ${addedCount} 个模块`);} else {toast.info('所有模块已添加');}
  }

  function flattenModuleTree(mods: ScannedModule[]): ScannedModule[] {
    const result: ScannedModule[] = []; for (const mod of mods) { result.push(mod); if (mod.children) {result.push(...flattenModuleTree(mod.children));} } return result;
  }

  // Check if scanned modules have a parent-child (multi-module Maven) structure
  function hasParentChildStructure(): boolean {
    for (const mod of scannedModules.value) {
      if (mod.children && mod.children.length > 0) {return true;}
    }
    return false;
  }

  // Auto-detect and enable parent-build mode from scanner results
  function autoDetectParentBuild(): void {
    if (hasParentChildStructure()) {
      config.value.parentBuildMode = true;
      parentBuildAutoDetected.value = true;
      // Find the root parent module (the one with children)
      for (const mod of scannedModules.value) {
        if (mod.children && mod.children.length > 0) {
          // Root parent is at project root — parentBuildPath stays empty (use project root)
          // If the parent is in a subdirectory (e.g., scanned from a sub-pom), set the path
          if (mod.path && mod.path !== '.') {
            config.value.parentBuildPath = mod.path;
          } else {
            config.value.parentBuildPath = '';
          }
          break;
        }
      }
      toast.success('已自动检测到 Maven 父子模块结构，启用统一构建');
    }
  }

  async function deleteModule(moduleId: string | null) {
    if (moduleId) { try { await getTauriAPI().deleteDeployModule(moduleId); } catch (error) { handleError(error, { context: '删除模块' }); } }
    modules.value = modules.value.filter(m => m.id !== moduleId);
    expandedModules.value = expandedModules.value.map(i => i > modules.value.length - 1 ? -1 : i).filter(i => i >= 0).sort((a, b) => a - b);
  }

  async function saveConfig() {
    try {
      if (!deployServers.value.some(s => s.serverId)) { toast.error('请选择服务器'); return; }
      const now = new Date().toISOString();
      const serversJson = deployServers.value.length > 0 ? JSON.stringify(deployServers.value.map(s => ({ serverId: s.serverId, label: s.label, deployDir: s.deployDir }))) : null;
      const plainConfig = { ...JSON.parse(JSON.stringify(config.value)), servers: serversJson };
      if (!config.value.id) {
        config.value.id = Date.now().toString(); config.value.createdAt = now; config.value.updatedAt = now;
        plainConfig.id = config.value.id; plainConfig.createdAt = now; plainConfig.updatedAt = now;
        await getTauriAPI().addCicdConfig(plainConfig);
      } else { config.value.updatedAt = now; plainConfig.updatedAt = now; await getTauriAPI().updateCicdConfig(plainConfig); }
      const currentIds = new Set(modules.value.filter(m => m.id).map(m => m.id));
      const existingMods = await getTauriAPI().getDeployModules(config.value.id as string);
      for (const existing of (existingMods as DeployModule[] || [])) { if (existing.id && !currentIds.has(existing.id)) {await getTauriAPI().deleteDeployModule(existing.id);} }
      for (const mod of modules.value) {
        if (!mod.id) {
          mod.id = Date.now().toString() + Math.random().toString(36).substr(2, 9); mod.configId = config.value.id;
          mod.createdAt = now; mod.updatedAt = now; await getTauriAPI().addDeployModule(JSON.parse(JSON.stringify(mod)));
        } else { mod.updatedAt = now; await getTauriAPI().updateDeployModule(JSON.parse(JSON.stringify(mod))); }
      }
      toast.success('配置保存成功'); await loadConfigs(); selectedConfigId.value = config.value.id!; isNewConfig.value = false;
    } catch (error) { handleError(error, { context: '保存配置' }); }
  }

  async function deleteConfig(id: string) {
    if (!confirm('确定删除此 CI/CD 配置吗？')) {return;}
    try {
      await getTauriAPI().deleteCicdConfig(id); await loadConfigs();
      if (selectedConfigId.value === id) {createNewConfig();}
      toast.success('配置已删除');
    } catch (error) { handleError(error, { context: '删除配置' }); }
  }

  async function copyConfig(sourceId: string) {
    try {
      // 1. 获取源配置
      const source = await getTauriAPI().getCicdConfigById(sourceId) as CicdConfigEntry | undefined;
      if (!source || !source.id) { toast.error('源配置不存在'); return; }

      // 2. 获取源配置的子模块
      const sourceModules = await getTauriAPI().getDeployModules(sourceId) as DeployModule[];

      // 3. 深拷贝配置，生成新 ID 和时间戳
      const now = new Date().toISOString();
      const newId = Date.now().toString();
      const newConfig = { ...JSON.parse(JSON.stringify(source)) };

      // 重置 ID 和元数据
      newConfig.id = newId;
      newConfig.createdAt = now;
      newConfig.updatedAt = now;
      // 清空部署时间（新配置还未部署过）
      newConfig.lastDeployedAt = null;
      // 修改名称: "源名称 - 副本"
      newConfig.name = (source.name || getGitRepoName(source.gitRepoId) || '未命名配置') + ' - 副本';

      // 4. 先保存配置（不传 modules，与 saveConfig 新建逻辑保持一致）
      await getTauriAPI().addCicdConfig(newConfig);

      // 5. 逐个保存子模块（与 saveConfig 新建逻辑保持一致）
      for (const mod of sourceModules) {
        const newMod = { ...JSON.parse(JSON.stringify(mod)) };
        newMod.id = `${newId}-mod-${Math.random().toString(36).substr(2, 9)}`;
        newMod.configId = newId;
        newMod.createdAt = now;
        newMod.updatedAt = now;
        await getTauriAPI().addDeployModule(newMod);
      }

      // 6. 重新加载配置列表
      await loadConfigs();

      // 7. 选中新配置并打开到编辑器
      selectedConfigId.value = newId;
      isNewConfig.value = false;
      await loadConfig(newId);

      // 8. 异步运行工具路径检测（新配置打开后自动填充路径）
      runToolDetection();

      toast.success('配置已复制');
    } catch (error) { handleError(error, { context: '复制配置' }); }
  }

  // Normalize old saved /bin/java or /bin/node paths to home directories
  function normalizeHomeDir(p: string): string {
    if (!p) {return '';}
    const binIdx = p.lastIndexOf('/bin/');
    return binIdx > 0 ? p.slice(0, binIdx) : p;
  }

  async function loadConfig(configId: string) {
    try {
      const existing = await getTauriAPI().getCicdConfigById(configId) as CicdConfigEntry | undefined;
      if (existing && existing.id) {
        config.value = { ...defaultConfig(), ...existing } as ConfigForm;
        // 修复：Rust Option::None 序列化为 JSON null，会覆盖 defaultConfig 的空字符串
        // 将所有 null 字符串字段转回空字符串
        for (const key of ['gitRepoId', 'deployBranch', 'groupName', 'name', 'localPath', 'repoUrl',
          'mavenHome', 'javaHome', 'npmHome', 'pnpmHome', 'yarnHome', 'nodeHome',
          'deployPath', 'restartScript', 'mavenProfile', 'mavenSettings',
          'buildCommand', 'buildPath', 'npmScript', 'npmCustomScript',
        ] as const) {
          if ((config.value as any)[key] === null || (config.value as any)[key] === undefined) {
            (config.value as any)[key] = '';
          }
        }
        // Normalize old saved /bin/java → JAVA_HOME, /bin/node → NVM_HOME
        config.value.javaHome = normalizeHomeDir(config.value.javaHome);
        config.value.nodeHome = normalizeHomeDir(config.value.nodeHome);
        // Derive package manager paths from nodeHome (use detected paths from SDK versions if available)
        const matchedNode = sdkVersions.value.nvm.node.find(
          (v: { name: string; path: string }) => v.path === config.value.nodeHome
        );
        if (matchedNode) {
          if (!config.value.npmHome && matchedNode.npm) {config.value.npmHome = matchedNode.npm;}
          if (!config.value.pnpmHome && matchedNode.pnpm) {config.value.pnpmHome = matchedNode.pnpm;}
          if (!config.value.yarnHome && matchedNode.yarn) {config.value.yarnHome = matchedNode.yarn;}
        }
        if (existing.servers && typeof existing.servers === 'string') {
          try {
            const parsed = JSON.parse(existing.servers);
            deployServers.value = parsed.map((s: { serverId?: string; label?: string; deployDir?: string }) => ({ serverId: s.serverId || '', label: s.label || getServerName(s.serverId || '') || '', deployDir: s.deployDir || '', testResult: null }));
            if (deployServers.value.length === 0) {deployServers.value = [makeDefaultServer()];}
          } catch { deployServers.value = [makeDefaultServer()]; }
        } else { deployServers.value = [makeDefaultServer()]; }
        activeServerIdx.value = 0;
        const mods = await getTauriAPI().getDeployModules(existing.id as string);
        modules.value = (mods as DeployModule[]) || [];
        if (!config.value.deployPath && config.value.buildTool) {
          if (config.value.buildTool === 'maven') {config.value.deployPath = '~/apphome';}
          else if (['npm', 'pnpm', 'yarn'].includes(config.value.buildTool)) {config.value.deployPath = '/home/nginxWebUI/ui';}
        }
        const dp = defaultPaths.value;
        if (!config.value.mavenHome && dp.mavenHome) {config.value.mavenHome = dp.mavenHome;}
        if (!config.value.javaHome && dp.javaHome) {config.value.javaHome = dp.javaHome;}
        if (!config.value.npmHome && dp.npmHome) {config.value.npmHome = dp.npmHome;}
        if (!config.value.pnpmHome && dp.pnpmHome) {config.value.pnpmHome = dp.pnpmHome;}
        if (!config.value.yarnHome && dp.yarnHome) {config.value.yarnHome = dp.yarnHome;}
        if (!config.value.nodeHome && dp.nodeHome) {config.value.nodeHome = dp.nodeHome;}
        const currentJava = sdkVersions.value.sdkman?.java?.find?.((v: { name: string; path: string; isCurrent?: boolean }) => v.isCurrent);
        if (currentJava && !config.value.javaHome) { config.value.javaHome = currentJava.path; selectedJavaVersion.value = currentJava.path; }
        const currentNode = sdkVersions.value.nvm?.node?.find?.((v: { name: string; path: string; isCurrent?: boolean; npm?: string; pnpm?: string; yarn?: string }) => v.isCurrent);
        if (currentNode && !config.value.nodeHome) { config.value.nodeHome = currentNode.path; selectedNodeVersion.value = currentNode.path; }
        if (currentNode) {
          if (!config.value.npmHome && currentNode.npm) {config.value.npmHome = currentNode.npm;}
          if (!config.value.pnpmHome && currentNode.pnpm) {config.value.pnpmHome = currentNode.pnpm;}
          if (!config.value.yarnHome && currentNode.yarn) {config.value.yarnHome = currentNode.yarn;}
        }
        if (config.value.localPath) {loadBranches();}
      }
    } catch (error) { handleError(error, { context: '加载配置' }); }
  }

  // ─── Watch localPath changes — auto-scan project ───
  let _scanDebounce: ReturnType<typeof setTimeout> | undefined;
  const _stopWatchLocalPath = watch(() => config.value.localPath, (newPath) => {
    if (!newPath) {return;}
    clearTimeout(_scanDebounce);
    _scanDebounce = setTimeout(() => scanLocalProject(newPath), 300);
  });

  // ─── Watch build tool changes ───
  const _stopWatchBuildTool = watch(() => config.value.buildTool, async (newTool, oldTool) => {
    if (newTool === oldTool || !newTool) {return;}
    const c = config.value;
    const dp = defaultPaths.value;

    // Clear old tool paths
    if (oldTool === 'maven') { c.mavenHome = ''; c.javaHome = ''; }
    if (['npm', 'pnpm', 'yarn'].includes(oldTool || '')) { c.npmHome = ''; c.pnpmHome = ''; c.yarnHome = ''; c.nodeHome = ''; }

    // Update deploy path
    const oldDefaults = ['~/apphome', '/home/nginxWebUI/ui'];
    if (!c.deployPath || oldDefaults.includes(c.deployPath)) {
      c.deployPath = newTool === 'maven' ? '~/apphome' : '/home/nginxWebUI/ui';
    }

    // Ensure defaultPaths are populated, trigger detection if needed
    if (!dp.mavenHome && !dp.npmHome && !dp.nodeHome) {
      try {
        const paths = await getTauriAPI().detectToolPaths() as typeof dp;
        if (paths) {Object.assign(dp, paths);}
      } catch (e) { console.warn('[CICD] 工具路径检测失败', e); }
    }

    // Auto-fill new tool paths
    if (newTool === 'maven') {
      if (!c.mavenHome && dp.mavenHome) {c.mavenHome = dp.mavenHome;}
      if (!c.javaHome && dp.javaHome) {c.javaHome = dp.javaHome;}
      // Try SDKMAN for Java
      if (!c.javaHome) {
        const cur = sdkVersions.value.sdkman?.java?.find?.((v: { isCurrent?: boolean }) => v.isCurrent);
        if (cur) { c.javaHome = cur.path; selectedJavaVersion.value = cur.path; }
      }
    }
    if (['npm', 'pnpm', 'yarn'].includes(newTool)) {
      if (!c.nodeHome && dp.nodeHome) {c.nodeHome = dp.nodeHome;}
      if (newTool === 'npm' && !c.npmHome && dp.npmHome) {c.npmHome = dp.npmHome;}
      if (newTool === 'pnpm' && !c.pnpmHome && dp.pnpmHome) {c.pnpmHome = dp.pnpmHome;}
      if (newTool === 'yarn' && !c.yarnHome && dp.yarnHome) {c.yarnHome = dp.yarnHome;}
      // Try NVM for Node
      if (!c.nodeHome) {
        const cur = sdkVersions.value.nvm.node.find((v: { isCurrent?: boolean }) => v.isCurrent);
        if (cur) {
          c.nodeHome = cur.path; selectedNodeVersion.value = cur.path;
          if (!c.npmHome && cur.npm) {c.npmHome = cur.npm;}
          if (!c.pnpmHome && cur.pnpm) {c.pnpmHome = cur.pnpm;}
          if (!c.yarnHome && cur.yarn) {c.yarnHome = cur.yarn;}
        }
      }
    }
  });

  let _cleanupDataChanged: (() => void) | undefined;
  let _loadFirstConfigTimer: ReturnType<typeof setTimeout> | undefined;

  // ─── Init ───
  // 页面立即渲染，不做任何阻塞式加载
  const pageLoading = ref(false);

  onMounted(async () => {
    // 第一步：加载核心数据（配置列表、服务器、仓库等）- 这是轻量级DB查询，很快
    shared.load().then(() => {
      // CICD 独有分组数据
      getTauriAPI().getCicdGroups?.().then(groupsResult => {
        groups.value = (groupsResult as string[]) || [];
        initExpandedGroups();
      }).catch(() => {});
      
      // 如果有配置，自动选中第一个
      if (configs.value.length > 0) {
        selectedConfigId.value = configs.value[0].id;
        isNewConfig.value = false;
        // 延迟加载配置详情，不阻塞列表渲染
        _loadFirstConfigTimer = setTimeout(() => loadConfig(configs.value[0].id).catch(() => {}), 100);
      }
    }).catch(err => handleError(err, { context: '加载CI/CD配置' }));

    // ⚠️ 关键优化：完全不调用 detect_build_tools/detect_tool_paths/detect_sdk_versions
    // 这些检测会阻塞 IPC 通道（运行7个shell命令），只在用户点击新建配置时才触发
  });

  onBeforeUnmount(() => {
    _cleanupDataChanged?.();
    // 清理 watch 停止句柄
    _stopWatchLocalPath();
    _stopWatchBuildTool();
    // 清理未触发的定时器
    clearTimeout(_scanDebounce);
    clearTimeout(_loadFirstConfigTimer);
  });

  async function loadServers() {
    try { servers.value = (await getTauriAPI().getAllServers?.()) as Server[] | undefined || []; serverGroups.value = (await getTauriAPI().getServerGroups?.()) as Array<{ id: string; name: string; color: string; parentId: string | null }> | undefined || []; }
    catch (error) { handleError(error, { context: 'loadServers' }); servers.value = []; serverGroups.value = []; }
  }

  async function loadProjects() {
    try { projects.value = (await getTauriAPI().getProjects?.()) as Project[] | undefined || []; }
    catch (error) { handleError(error, { context: 'loadProjects' }); }
  }

  async function loadGitRepos() {
    try {
      gitRepos.value = (await getTauriAPI().getGitRepos?.()) || [];
    }
    catch (error) { handleError(error, { context: 'loadGitRepos' }); }
  }

  // 切换到 Git 克隆模式并自动获取远程仓库地址
  async function switchToGitCloneMode() {
    config.value.buildMode = 'git_clone';
    // 如果已选择本地仓库，自动获取其远程地址
    if (selectedGitRepo.value?.path && !config.value.repoUrl) {
      await fetchGitRemoteUrl();
    }
  }

  // 从本地仓库获取 git remote URL
  async function fetchGitRemoteUrl() {
    const localPath = selectedGitRepo.value?.path || config.value.localPath;
    if (!localPath) {
      toast.warning('请先选择本地仓库');
      return;
    }
    try {
      const result = await getTauriAPI().gitRemotes?.(localPath);
      if (result && result.remotes && Array.isArray(result.remotes)) {
        // 优先使用 origin 的 fetchUrl
        const origin = result.remotes.find((r: { name: string }) => r.name === 'origin');
        if (origin && origin.fetchUrl) {
          config.value.repoUrl = origin.fetchUrl;
          toast.success(`已获取远程地址: ${origin.fetchUrl}`);
        } else if (result.remotes.length > 0 && result.remotes[0].fetchUrl) {
          // 没有 origin，使用第一个 remote
          config.value.repoUrl = result.remotes[0].fetchUrl;
          toast.success(`已获取远程地址: ${result.remotes[0].fetchUrl}`);
        } else {
          toast.warning('未找到远程仓库地址，请手动输入');
        }
      }
    } catch (error) {
      handleError(error, { context: '获取远程仓库地址' });
    }
  }

  return {
    // State
    configs, projects, gitRepos, servers, serverGroups, selectedConfigId, isNewConfig, searchQuery, sidebarCollapsed,
    selectedServerId, deployServers, activeServerIdx, groups, expandedGroups,
    showGroupDialog, groupNameInput, groupDialogMode, groupDialogOldName,
    showGroupEditor, newGroupName,
    config, modules, testResult, detectedTools, availableBranches, loadingBranches,
    expandedModules, scannedModules, scanningModules, showModuleTree, expandedTreeNodes,
    defaultPaths, sdkVersions, selectedJavaVersion, selectedNodeVersion, detectingPaths,
    sdkmanInstallGuide, nvmInstallGuide,
    // Computed
    filteredConfigs, groupedConfigs, hasAnyGitSource, gitSources,
    projectShortName, availableBuildTools, addedModulePaths, buildToolDefs,
    parentBuildAutoDetected, parentBuildDetectedPath, selectedGitRepo,
    // Functions
    openGroupDialog, confirmGroupDialog, cancelGroupDialog, initExpandedGroups,
    makeDefaultServer, getServerName, onServerSelect, addServer, removeServer,
    testServerById, onJavaVersionSelected, onNodeVersionSelected, reDetectToolPaths,
    getProjectName, getGitRepoName, getToolBadge, getBuildToolIcon, getBuildToolName, formatTime,
    toggleGroup, renameGroup, addGroup, getServerLabel,
    loadConfigs, createNewConfig, selectConfig, onProjectChange, onGitRepoChange, selectLocalDir,
    selectServer, copyGitUrl, loadBranches, testConnection,
    addModule, toggleModuleExpand, scanModules, toggleTreeNode, isModuleAlreadyAdded,
    addModuleFromScan, addAllDetectedModules, flattenModuleTree, autoDetectParentBuild, deleteModule,
    saveConfig, deleteConfig, copyConfig, loadConfig, loadServers, loadProjects, loadGitRepos,
    switchToGitCloneMode, fetchGitRemoteUrl,
    defaultConfig,
    pageLoading,
  };
}
