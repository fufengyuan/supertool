<template>
  <div>
    <h2>服务器管理</h2>

    <div class="flex gap-1.5 mb-3 items-center flex-wrap">
      <button @click="showAddServer = true" class="btn btn-primary btn-sm">+ 添加服务器</button>
      <button @click="refreshServers" class="btn btn-ghost btn-sm gap-1.5"><SvgIcon name="refresh" size="14" /> 刷新</button>
      <button @click="showGroupManager = true" class="btn btn-ghost btn-sm gap-1.5"><SvgIcon name="folder" size="14" /> 管理分组</button>
      <div class="w-px h-6 bg-base-content/10 mx-1"></div>
      <button @click="viewMode = viewMode === 'card' ? 'list' : 'card'" class="btn btn-ghost btn-sm gap-1.5"
        :title="viewMode === 'card' ? '切换为列表（更密）' : '切换为卡片'">
        <SvgIcon :name="viewMode === 'card' ? 'list' : 'grid'" size="14" />
        {{ viewMode === 'card' ? '列表视图' : '卡片视图' }}
      </button>
    </div>

    <!-- 环境（顶层分组）作筛选器 + 搜索 -->
    <div class="flex gap-3 mb-4 items-center flex-wrap">
      <div class="flex gap-0.5 bg-base-100 border border-base-content/10 rounded-lg p-0.5">
        <button class="px-2.5 py-1 rounded-md text-xs flex items-center gap-1.5 transition-colors"
          :class="envFilter === 'all' ? 'bg-base-200 font-semibold text-base-content' : 'text-base-content/55 hover:text-base-content'"
          @click="envFilter = 'all'">
          全部 <span class="text-[10px] opacity-70 tabular-nums">{{ servers.length }}</span>
        </button>
        <button v-for="root in rootGroups" :key="root.id"
          class="px-2.5 py-1 rounded-md text-xs flex items-center gap-1.5 transition-colors"
          :class="envFilter === root.id ? 'bg-base-200 font-semibold text-base-content' : 'text-base-content/55 hover:text-base-content'"
          @click="envFilter = root.id">
          <span class="w-1.5 h-1.5 rounded-full shrink-0" :style="{ backgroundColor: root.color || '#6b7280' }"></span>
          {{ root.name }} <span class="text-[10px] opacity-70 tabular-nums">{{ countByEnv(root.id) }}</span>
        </button>
      </div>
      <div class="flex-1 relative flex items-center min-w-[220px]">
        <SvgIcon name="search" size="14" class="absolute left-3 text-base-content/50 pointer-events-none" />
        <input v-model="searchQuery" placeholder="搜索名称 / 主机 / 用户…" class="input input-bordered input-sm w-full pl-8" />
      </div>
    </div>

    <!-- 卡片视图：一层分段（环境 → 业务分组只作小标题，卡片直接平铺） -->
    <div v-if="viewMode === 'card'" class="flex flex-col gap-4">
      <section v-for="sec in sections" :key="sec.key">
        <div class="flex items-center gap-2 mb-1.5">
          <span class="w-2 h-2 rounded-full shrink-0" :style="{ backgroundColor: sec.color }"></span>
          <span class="text-[13px] font-semibold text-base-content">{{ sec.name }}</span>
          <span class="text-[10px] px-1.5 rounded bg-base-content/5 text-base-content/55 tabular-nums">{{ sec.servers.length }}</span>
          <span v-if="sec.envName && sec.envName !== sec.name" class="text-[10px] text-base-content/40">{{ sec.envName }}</span>
          <span v-if="sec.online > 0" class="flex items-center gap-1 text-[10px] text-success">
            <span class="w-1 h-1 rounded-full bg-success"></span>{{ sec.online }} 在线
          </span>
        </div>
        <div class="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-2">
          <ServerItem
            v-for="server in sec.servers"
            :key="server.id"
            :server="server"
            :connection-status="connectionStatusMap[server.id] || 'offline'"
            @terminal="openTerminal"
            @sftp="openSftp"
            @edit="editServer"
            @delete="deleteServer"
          />
        </div>
      </section>
    </div>

    <!-- 列表视图：一行一台，主机/用户/所属分组/状态同屏可扫 -->
    <div v-else class="bg-base-100 border border-base-content/10 rounded-xl overflow-hidden">
      <table class="w-full text-xs">
        <thead>
          <tr class="text-[11px] text-base-content/50 bg-base-200/60">
            <th class="text-left font-medium px-3 py-2">名称</th>
            <th class="text-left font-medium px-3 py-2">主机</th>
            <th class="text-left font-medium px-3 py-2">用户</th>
            <th class="text-left font-medium px-3 py-2">分组</th>
            <th class="text-left font-medium px-3 py-2 w-24">状态</th>
            <th class="text-right font-medium px-3 py-2 w-40">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="s in visibleServers" :key="s.id" class="border-t border-base-content/5 hover:bg-base-200/50">
            <td class="px-3 py-1.5 font-medium text-base-content whitespace-nowrap">
              {{ s.name }}
              <SvgIcon v-if="s.requiresApproval" name="lock" size="11" class="inline opacity-60" title="执行审核已开启" />
            </td>
            <td class="px-3 py-1.5 font-mono text-base-content/65 whitespace-nowrap">{{ s.host }}:{{ s.port }}</td>
            <td class="px-3 py-1.5 font-mono text-base-content/65">{{ s.username }}</td>
            <td class="px-3 py-1.5 text-base-content/55 whitespace-nowrap">{{ groupPath(s.groupId) }}</td>
            <td class="px-3 py-1.5">
              <span class="text-[10px] px-1.5 py-0.5 rounded-full whitespace-nowrap" :class="statusClass(s)">{{ statusText(s) }}</span>
            </td>
            <td class="px-3 py-1.5 text-right whitespace-nowrap">
              <button @click="openTerminal(s)" class="btn btn-ghost btn-xs gap-1"><SvgIcon name="terminal" size="12" /> 终端</button>
              <button @click="openSftp(s)" class="btn btn-ghost btn-xs gap-1"><SvgIcon name="download" size="12" /> SFTP</button>
              <button @click="editServer(s)" class="btn btn-ghost btn-xs px-1" title="编辑"><SvgIcon name="pencil" size="12" /></button>
              <button @click="deleteServer(s.id)" class="btn btn-ghost btn-xs px-1 hover:text-error" title="删除"><SvgIcon name="trash" size="12" /></button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div v-if="visibleServers.length === 0" class="text-center py-10 text-base-content/60 bg-base-100 rounded-xl">
      <template v-if="servers.length === 0 && !searchQuery">
        <!-- 真正空状态 -->
        <SvgIcon class="opacity-20 mb-4" name="serverRack" size="48" stroke-width="1.5" />
        <p class="text-base font-semibold text-base-content m-0 mb-2">暂无服务器</p>
        <p class="text-sm m-0 mb-4">点击上方「添加服务器」按钮，管理你的远程服务器</p>
        <button @click="showAddServer = true" class="btn btn-primary">+ 添加第一个服务器</button>
      </template>
      <template v-else>
        <p>{{ searchQuery ? '没有找到匹配的服务器' : '暂无服务器，点击上方按钮添加' }}</p>
      </template>
    </div>

    <!-- 添加/编辑服务器弹窗 -->
    <ServerForm
      v-if="showAddServer || editingServer"
      :form="serverForm"
      :is-editing="!!editingServer"
      :test-result="testResult"
      :groups="groups"
      @close="closeModal"
      @test-connection="testConnection"
      @save="saveServer"
      @update:form="serverForm = $event"
    />

    <!-- 分组管理弹窗 -->
    <Modal v-model="showGroupManager" title="管理分组" width="520px">
      <div class="py-2">
        <div class="mb-5">
          <div v-for="group in groups" :key="group.id" class="flex items-center gap-2.5 py-2.5 px-3 rounded-lg bg-base-200 mb-2" :style="{ paddingLeft: `${getGroupDepth(group) * 20}px` }">
            <span v-if="group.parentId" class="inline-block">└ </span>
            <span class="inline-block w-3 h-3 rounded-full" :style="{ backgroundColor: group.color || '#6c63ff' }"></span>
            <span class="flex-1 font-medium">{{ group.name }}</span>
            <span class="text-xs text-base-content/60">{{ getServersByGroup(group.id).length }} 台</span>
            <button @click="addGroupAsChild(group.id)" class="btn btn-ghost btn-xs" title="添加子分组">+</button>
            <button @click="editGroup(group.id)" class="btn btn-ghost btn-xs px-1" title="编辑"><SvgIcon name="pencil" size="12" /></button>
            <button @click="deleteGroup(group.id)" class="btn btn-error btn-xs px-1" title="删除"><SvgIcon name="trash" size="12" /></button>
          </div>
          <div v-if="groups.length === 0" class="text-center py-5 text-base-content/60">暂无分组</div>
        </div>
        <div class="border-t border-base-content/10 pt-4">
          <h4 class="m-0 mb-3 text-sm text-base-content">{{ editingGroupId ? '编辑分组' : (addingChildTo ? '添加子分组' : '添加新分组') }}</h4>
          <div class="flex gap-2.5 mb-2.5">
            <input v-model="newGroupName" placeholder="分组名称" class="input input-bordered flex-1" @keyup.enter="saveGroup" />
            <input v-model="newGroupColor" type="color" class="w-10 h-9 border border-base-content/10 rounded-md cursor-pointer p-0.5 bg-base-200" value="#6c63ff" />
          </div>
          <div v-if="!editingGroupId" class="flex gap-2.5 mb-2.5">
            <select v-model="newGroupParent" class="select select-bordered flex-1">
              <option :value="null">无父分组（顶级）</option>
              <option v-for="group in groups" :key="group.id" :value="group.id">
                {{ group.name }}
              </option>
            </select>
          </div>
          <div class="flex gap-2">
            <button @click="saveGroup" class="btn btn-primary" :disabled="!newGroupName.trim()">{{ editingGroupId ? '保存' : '添加' }}</button>
            <button v-if="editingGroupId || addingChildTo" @click="cancelEditGroup" class="btn btn-ghost">取消</button>
          </div>
        </div>
      </div>
    </Modal>

    <!-- 终端窗口 -->
    <TerminalPanel v-if="terminalServer" :server="terminalServer" :servers="servers" @close="terminalServer = null" @openSftp="onTerminalOpenSftp" />

    <!-- SFTP窗口 — 支持同时打开多个 -->
    <SftpPanel
      v-for="panel in sftpPanels"
      :key="panel.id"
      :server="panel.server"
      :initial-path="panel.initialPath"
      :initial-position="panel.position"
      @close="closeSftpPanel(panel.id)"
    />
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'ServerManager' })
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import { getTauriAPI } from '../../utils/tauri-api'
import ServerItem from './ServerItem.vue';
import ServerForm from './ServerForm.vue';
import TerminalPanel from './TerminalPanel.vue';
import SftpPanel from './SftpPanel.vue';
import Modal from '../../components/ui/Modal.vue';
import { useToast } from '../../composables/useToast';
import { useErrorHandler } from '../../composables/useErrorHandler';
import type { Server, ServerGroup } from '../../types';

const toast = useToast();
const { handleError } = useErrorHandler();

const servers = ref<Server[]>([]);
const groups = ref<ServerGroup[]>([]);
const connectionStatusMap = ref<Record<string, string>>({});
const showAddServer = ref(false);
const editingServer = ref<Server | null>(null);
const testResult = ref<{ success: boolean; error?: string } | null>(null);
const terminalServer = ref<Server | null>(null);
interface SftpPanelEntry {
  id: string;
  server: Server;
  initialPath: string;
  position: { x: number; y: number };
}
const sftpPanels = ref<SftpPanelEntry[]>([]);
const searchQuery = ref('');
// 环境（顶层分组）筛选：'all' 或某个顶层分组 id
const envFilter = ref<string>('all');
const viewMode = ref<'card' | 'list'>('card');
const showGroupManager = ref(false);
const newGroupName = ref('');
const newGroupColor = ref('#6c63ff');
const newGroupParent = ref<string | null>(null);
const editingGroupId = ref<string | null>(null);
const addingChildTo = ref<string | null>(null);

const defaultForm = () => ({
  id: null as string | null,
  name: '',
  host: '',
  port: 22,
  username: '',
  // 认证方式：密码 / SSH 密钥 二选一
  authType: 'password' as 'password' | 'key',
  sshKeyPath: '',
  password: '',
  tagsInput: '',
  description: '',
  groupId: null as string | null,
  requiresApproval: false,
  createdAt: '',
  updatedAt: '',
});

const serverForm = ref(defaultForm());

onMounted(async () => {
  loadServers();
  loadGroups();

  getTauriAPI().onServerConnected?.((data) => {
    connectionStatusMap.value[data.serverId] = 'online';
  });

  getTauriAPI().onServerDisconnected?.((data) => {
    delete connectionStatusMap.value[data.serverId];
  });

  getTauriAPI().onServerHeartbeatFailed?.((data) => {
    connectionStatusMap.value[data.serverId] = 'heartbeat_failed';
    toast.error('服务器连接已断开');
  });

  // Auto-refresh when server data changes elsewhere
  const cleanupDataChanged = await getTauriAPI().onDataChanged?.(({ type }) => {
    if (type === 'servers') {refreshServers();}
  });
  if (cleanupDataChanged) {_cleanupDataChanged = cleanupDataChanged;}
});

let _cleanupDataChanged: (() => void) | undefined;

onBeforeUnmount(() => {
  _cleanupDataChanged?.();
});

async function loadServers() {
  try {
    servers.value = await getTauriAPI().getAllServers();
  } catch (error) {
    handleError(error, { context: 'loadServers' });
  }
}

async function loadGroups(): Promise<void> {
  try {
    groups.value = await getTauriAPI().getServerGroups();
  } catch (error) {
    handleError(error, { context: 'loadGroups' });
  }
}

async function refreshServers() {
  await loadServers();
  await loadGroups();
}

// 搜索：名称 / 主机 / 用户
const searchLower = computed(() => searchQuery.value.trim().toLowerCase());
function matchSearch(s: Server) {
  const q = searchLower.value;
  if (!q) {return true;}
  return s.name.toLowerCase().includes(q) || s.host.toLowerCase().includes(q) || (s.username || '').toLowerCase().includes(q);
}

// 根分组（顶层 = 环境）
const rootGroups = computed(() => groups.value.filter((g: ServerGroup) => !g.parentId));

function childGroupsOf(groupId: string) {
  return groups.value.filter((g: ServerGroup) => g.parentId === groupId);
}

// 某顶层分组下的全部子孙分组（深度优先，保持声明顺序）
function descendantsOf(root: ServerGroup): ServerGroup[] {
  const out: ServerGroup[] = [];
  const seen = new Set<string>();
  const walk = (g: ServerGroup) => {
    for (const c of childGroupsOf(g.id)) {
      if (seen.has(c.id)) {continue;}
      seen.add(c.id);
      out.push(c);
      walk(c);
    }
  };
  walk(root);
  return out;
}

// 分组管理弹窗里的直属台数（不做搜索过滤）
function getServersByGroup(groupId: string | null) {
  return servers.value.filter((s: any) => (s.groupId || null) === (groupId || null));
}

function serversOfGroup(groupId: string | null) {
  return servers.value.filter((s: any) => (s.groupId || null) === (groupId || null)).filter(matchSearch);
}

function onlineOf(list: Server[]) {
  return list.filter(s => connectionStatusMap.value[s.id] === 'online').length;
}

// 环境（含子分组）服务器总数，不受搜索影响 —— 用于分段上的计数
function countByEnv(rootId: string) {
  const root = groups.value.find(g => g.id === rootId);
  if (!root) {return 0;}
  const ids = new Set<string>([root.id, ...descendantsOf(root).map(g => g.id)]);
  return servers.value.filter(s => s.groupId && ids.has(s.groupId)).length;
}

interface Section {
  key: string;
  name: string;
  color: string;
  envName: string;
  servers: Server[];
  online: number;
}

// 一层分段：环境 → 业务分组只作小标题；分组层级再深也不会嵌套出多层盒子
const sections = computed<Section[]>(() => {
  const out: Section[] = [];
  const roots = envFilter.value === 'all'
    ? rootGroups.value
    : rootGroups.value.filter(g => g.id === envFilter.value);
  const assigned = new Set<string>();
  for (const root of roots) {
    const own = serversOfGroup(root.id);
    if (own.length > 0) {
      out.push({ key: `g-${root.id}`, name: root.name, color: root.color || '#6b7280', envName: root.name, servers: own, online: onlineOf(own) });
    }
    for (const g of descendantsOf(root)) {
      const list = serversOfGroup(g.id);
      if (list.length > 0) {
        out.push({ key: `g-${g.id}`, name: g.name, color: g.color || root.color || '#6b7280', envName: root.name, servers: list, online: onlineOf(list) });
      }
    }
    assigned.add(root.id);
    descendantsOf(root).forEach(g => assigned.add(g.id));
  }
  // 未分组 + 指向已删除分组的服务器
  const known = new Set(groups.value.map(g => g.id));
  const loose = servers.value.filter(s => !s.groupId || !known.has(s.groupId) || !assigned.has(s.groupId)).filter(matchSearch);
  if (envFilter.value === 'all' && loose.length > 0) {
    out.push({ key: 'g-none', name: '未分组', color: '#94a3b8', envName: '', servers: loose, online: onlineOf(loose) });
  }
  return out;
});

const visibleServers = computed(() => sections.value.flatMap(sec => sec.servers));

// 列表视图里的「分组」列：完整路径，例如 生产环境 › 卡券集群
function groupPath(groupId?: string | null) {
  const chain: string[] = [];
  let cur = groupId ? groups.value.find(g => g.id === groupId) : undefined;
  const seen = new Set<string>();
  while (cur && !seen.has(cur.id)) {
    seen.add(cur.id);
    chain.unshift(cur.name);
    cur = cur.parentId ? groups.value.find(g => g.id === cur!.parentId) : undefined;
  }
  return chain.length > 0 ? chain.join(' › ') : '未分组';
}

function statusText(s: Server) {
  const st = connectionStatusMap.value[s.id];
  if (st === 'online') {return '已连接';}
  if (st === 'connecting') {return '连接中';}
  if (st === 'heartbeat_failed') {return '心跳失败';}
  return '未连接';
}

function statusClass(s: Server) {
  const st = connectionStatusMap.value[s.id];
  if (st === 'online') {return 'bg-success/15 text-success';}
  if (st === 'connecting') {return 'bg-warning/15 text-warning';}
  if (st === 'heartbeat_failed') {return 'bg-error/15 text-error';}
  return 'bg-base-content/5 text-base-content/50';
}

function openTerminal(server: any) {
  terminalServer.value = server;
}
function openSftp(server: Server, initialPath?: string) {
  // 如果该服务器的 SFTP 已打开，不重复打开
  if (sftpPanels.value.some(p => p.server.id === server.id)) {return;}
  // 为每个新面板生成级联偏移位置
  const idx = sftpPanels.value.length;
  const id = `sftp-${server.id}-${Date.now()}`;
  sftpPanels.value.push({
    id,
    server,
    initialPath: initialPath || '',
    position: { x: Math.max(50, (window.innerWidth - 800) / 2 + (idx * 30)), y: 80 + (idx * 30) }
  });
}

function closeSftpPanel(id: string) {
  sftpPanels.value = sftpPanels.value.filter(p => p.id !== id);
}

// 终端快捷打开 SFTP（带当前路径）
function onTerminalOpenSftp(server: Server, path: string) {
  // 不关闭终端，SFTP 浮在终端上方
  openSftp(server, path);
}

async function editServer(server: any) {
  editingServer.value = server;
  // 列表接口不返回密码（get_all_servers 会移除 password），编辑时单独按 id 获取解密后的密码回填
  let password = '';
  try {
    const full = await getTauriAPI().getServerById(server.id);
    // 竞态保护：等待期间用户可能已切换编辑其他服务器，旧响应不得覆盖新表单
    if (editingServer.value !== server) { return; }
    const p = full?.password || '';
    // 解密失败时后端会原样返回密文（Electron 格式 salt:iv:authTag:data 含冒号），
    // 不回填以免保存时被当作明文二次加密、破坏原密码
    if (p && !p.includes(':')) {
      password = p;
    }
  } catch (e) {
    // 获取密码失败时静默留空，保存时仍会保留原密码
  }
  const { password: _pw, ...serverWithoutPassword } = server;
  serverForm.value = {
    ...serverWithoutPassword,
    password,
    // 有密钥路径即视为密钥认证，否则按密码认证
    // （与后端 normalize_server_auth 的推导规则保持一致）
    authType: server.sshKeyPath && server.sshKeyPath.trim() ? 'key' : 'password',
    tagsInput: server.tags?.join(',') || '',
  };
}

async function deleteServer(serverId: string) {
  try {
    await getTauriAPI().deleteServer(serverId);
    delete connectionStatusMap.value[serverId];
    await loadServers();
    toast.success('服务器已删除');
  } catch (error) {
    handleError(error, { context: 'deleteServer' });
  }
}

// 分组管理
async function saveGroup() {
  if (!newGroupName.value.trim()) {return;}
  try {
    if (editingGroupId.value) {
      // 编辑模式
      await getTauriAPI().updateServerGroup(editingGroupId.value, {
        name: newGroupName.value.trim(),
        color: newGroupColor.value,
      });
      toast.success('分组已更新');
    } else {
      // 添加模式
      await getTauriAPI().addServerGroup({
        name: newGroupName.value.trim(),
        color: newGroupColor.value,
        parentId: addingChildTo.value || newGroupParent.value,
      });
      toast.success('分组已添加');
    }
    cancelEditGroup();
    await loadGroups();
  } catch (error) {
    handleError(error, { context: 'saveGroup' });
  }
}

function addGroupAsChild(parentId: string) {
  addingChildTo.value = parentId;
  editingGroupId.value = null;
  newGroupParent.value = null;
  newGroupName.value = '';
  newGroupColor.value = '#6c63ff';
}

function editGroup(groupId: string) {
  const group = groups.value.find((g: ServerGroup) => g.id === groupId);
  if (!group) {return;}
  editingGroupId.value = groupId;
  addingChildTo.value = null;
  newGroupParent.value = null;
  newGroupName.value = group.name;
  newGroupColor.value = group.color || '#6c63ff';
}

function cancelEditGroup() {
  editingGroupId.value = null;
  addingChildTo.value = null;
  newGroupParent.value = null;
  newGroupName.value = '';
  newGroupColor.value = '#6c63ff';
}

// 计算分组深度（用于缩进显示）
function getGroupDepth(group: { parentId?: string | null }): number {
  let depth = 0;
  let current = group.parentId;
  while (current) {
    depth++;
    const parent = groups.value.find((g: ServerGroup) => g.id === current);
    current = parent?.parentId || null;
  }
  return depth;
}

async function deleteGroup(groupId: string | null) {
  try {
    await getTauriAPI().deleteServerGroup(groupId ?? '');
    await loadGroups();
    await loadServers();
    if (envFilter.value === groupId) {envFilter.value = 'all';}
    toast.success('分组已删除');
  } catch (error) {
    handleError(error, { context: 'deleteGroup' });
  }
}

async function testConnection() {
  testResult.value = null;
  try {
    // 白名单字段（serverForm 含 tagsInput 等表单专用字段，不适合直接透传）
    const server: Partial<Server> = {
      id: serverForm.value.id ?? undefined,
      name: serverForm.value.name,
      host: serverForm.value.host,
      port: serverForm.value.port,
      username: serverForm.value.username,
      sshKeyPath: serverForm.value.sshKeyPath || '',
      password: serverForm.value.password || '',
      authType: serverForm.value.authType,
      tags: serverForm.value.tagsInput
        .split(',')
        .map((t) => t.trim())
        .filter((t) => t),
      description: serverForm.value.description || '',
      groupId: serverForm.value.groupId || null,
      requiresApproval: !!serverForm.value.requiresApproval,
    };
    testResult.value = await getTauriAPI().testServerConnection(server);
  } catch (error) {
    testResult.value = { success: false, error: (error as Error).message };
  }
}

async function saveServer() {
  const now = new Date().toISOString();
  // 白名单字段（去掉 tagsInput 等表单专用字段，满足 Partial<Server> 类型）
  const server: Partial<Server> = {
    id: serverForm.value.id || Date.now().toString(),
    name: serverForm.value.name,
    host: serverForm.value.host,
    port: serverForm.value.port,
    username: serverForm.value.username,
    sshKeyPath: serverForm.value.sshKeyPath,
    // 认证方式：让后端据此把另一项清成 NULL，避免残留空串
    authType: serverForm.value.authType,
    // BUG 6/7 FIX: Only include password if user actually entered a new one
    password: serverForm.value.password || undefined,
    tags: serverForm.value.tagsInput
      .split(',')
      .map((t) => t.trim())
      .filter((t) => t),
    description: serverForm.value.description,
    groupId: serverForm.value.groupId || null,
    requiresApproval: serverForm.value.requiresApproval || false,
    createdAt: serverForm.value.id ? serverForm.value.createdAt : now,
    updatedAt: now,
  };

  try {
    if (editingServer.value) {
      await getTauriAPI().updateServer(server);
      toast.success('服务器已更新');
    } else {
      await getTauriAPI().addServer(server);
      toast.success('服务器已添加');
    }
    closeModal();
    await loadServers();
  } catch (error) {
    handleError(error, { context: 'saveServer' });
  }
}

function closeModal() {
  showAddServer.value = false;
  editingServer.value = null;
  testResult.value = null;
  serverForm.value = defaultForm();
}
</script>

<style>
.drawer-expand-enter-active,
.drawer-expand-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  overflow: hidden;
}

.drawer-expand-enter-from,
.drawer-expand-leave-to {
  opacity: 0;
  max-height: 0;
  transform: translateY(-6px);
}

.drawer-expand-enter-to,
.drawer-expand-leave-from {
  opacity: 1;
  max-height: 5000px;
  transform: translateY(0);
}
</style>
