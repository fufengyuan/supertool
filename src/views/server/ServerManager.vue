<template>
  <div class="px-4 py-3">
    <h2>服务器管理</h2>

    <div class="flex gap-1.5 mb-3 items-center flex-wrap">
      <button @click="showAddServer = true" class="btn btn-primary">+ 添加服务器</button>
      <button @click="refreshServers" class="btn btn-ghost">🔄 刷新</button>
      <button @click="showGroupManager = true" class="btn btn-ghost">📁 管理分组</button>
      <div class="w-px h-7 bg-base-content/10 mx-1"></div>
      <button @click="expandAllGroups" class="btn btn-ghost btn-xs" title="全部展开">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.5">
          <polyline points="6 9 12 15 18 9"/>
        </svg>
        全部展开
      </button>
      <button @click="collapseAllGroups" class="btn btn-ghost btn-xs" title="全部折叠">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.5">
          <polyline points="6 15 12 9 18 15"/>
        </svg>
        全部折叠
      </button>
    </div>

    <!-- 搜索和分组筛选 -->
    <div class="flex gap-3 mb-5 items-center">
      <div class="flex-1 relative flex items-center">
        <svg class="absolute left-3 text-base-content/60 pointer-events-none" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"/>
          <path d="m21 21-4.35-4.35"/>
        </svg>
        <input
          v-model="searchQuery"
          placeholder="搜索服务器名称或地址..."
          class="input input-bordered w-full pl-9"
        />
      </div>
      <select v-model="selectedGroup" class="select select-bordered min-w-[150px]">
        <option value="">全部分组</option>
        <template v-for="group in groups" :key="group.id">
          <option v-if="!group.parentId" :value="group.id">
            {{ group.name }}
          </option>
          <option v-else :value="group.id">
            {{ '  ' + '└ ' + group.name }}
          </option>
        </template>
      </select>
    </div>

    <!-- 按分组树形折叠显示 -->
    <template v-if="selectedGroup === ''">
      <div v-if="getServersByGroup(null).length > 0" class="rounded-xl mb-1" :class="{ 'mb-2': expandedGroups.has(null) }">
        <div @click="toggleGroup(null)"
          class="flex items-center justify-between px-3 py-[7px] rounded-lg cursor-pointer select-none transition-all bg-base-100 border border-base-content/10 relative overflow-hidden hover:border-primary hover:shadow-[0_2px_12px_rgba(108,99,255,0.1)] hover:-translate-y-px before:content-[''] before:absolute before:left-0 before:top-0 before:bottom-0 before:w-[3px] before:rounded-r-[3px] before:bg-gradient-to-b before:from-[#6c63ff] before:to-[#4834d4]">
          <div class="flex items-center gap-2 relative z-[1]">
            <svg class="text-base-content/60 transition-transform duration-250 ease-[cubic-bezier(0.4,0,0.2,1)] shrink-0" :class="{ 'rotate-180 text-primary': expandedGroups.has(null) }" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="6 9 12 15 18 9"/>
            </svg>
            <span class="text-sm leading-none">🖥️</span>
            <span class="text-[13px] font-semibold text-base-content">未分组</span>
            <span class="text-[11px] font-semibold px-[7px] py-0.5 rounded-full bg-[#6c63ff]/[0.13] text-[#6c63ff]">{{ getServersByGroup(null).length }}</span>
          </div>
          <div class="flex items-center gap-2 relative z-[1]">
            <span class="flex items-center gap-1 text-[11px] text-success font-medium" v-if="getOnlineCount(null) > 0">
              <span class="inline-block w-2 h-2 rounded-full bg-success"></span>
              {{ getOnlineCount(null) }} 在线
            </span>
          </div>
        </div>
        <Transition name="drawer-expand">
          <div v-show="expandedGroups.has(null)" class="mt-1 py-2 px-2.5 rounded-lg bg-base-100 border border-base-content/10 border-t-0">
            <div class="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-2">
              <ServerItem
                v-for="server in getFilteredServers(getServersByGroup(null))"
                :key="server.id"
                :server="server"
                :connection-status="connectionStatusMap[server.id] || 'offline'"
                @terminal="openTerminal"
                @sftp="openSftp"
                @edit="editServer"
                @delete="deleteServer"
              />
            </div>
          </div>
        </Transition>
      </div>

      <!-- 分组树（支持多级嵌套） -->
      <GroupTree
        v-for="rootGroup in rootGroups"
        :key="rootGroup.id"
        :group="rootGroup"
        :groups="groups"
        :depth="0"
        :expanded-groups="expandedGroups"
        :servers="servers"
        :connection-status-map="connectionStatusMap"
        @toggle="toggleGroup"
        @terminal="openTerminal"
        @sftp="openSftp"
        @edit="editServer"
        @delete="deleteServer"
      />
    </template>

    <!-- 单个分组筛选视图 -->
    <template v-else>
      <div class="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-2">
        <ServerItem
          v-for="server in getFilteredServers(getServersByGroup(selectedGroup))"
          :key="server.id"
          :server="server"
          :connection-status="connectionStatusMap[server.id] || 'offline'"
          @terminal="openTerminal"
          @sftp="openSftp"
          @edit="editServer"
          @delete="deleteServer"
        />
      </div>
    </template>

    <div v-if="allFilteredServers.length === 0" class="text-center py-10 text-base-content/60 bg-base-100 rounded-xl">
      <template v-if="servers.length === 0 && !searchQuery">
        <!-- 真正空状态 -->
        <svg class="opacity-20 mb-4" viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="currentColor" stroke-width="1.5">
          <rect x="2" y="2" width="20" height="8" rx="2" ry="2" />
          <rect x="2" y="14" width="20" height="8" rx="2" ry="2" />
          <line x1="6" y1="6" x2="6.01" y2="6" />
          <line x1="6" y1="18" x2="6.01" y2="18" />
        </svg>
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
            <button @click="editGroup(group.id)" class="btn btn-ghost btn-xs" title="编辑">✎</button>
            <button @click="deleteGroup(group.id)" class="btn btn-error btn-xs" title="删除">✕</button>
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

<script setup lang="ts">// @ts-nocheck
console.log("[components/server/ServerManager.vue] component loaded")
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import { getTauriAPI } from '../../utils/tauri-api'
import ServerItem from './ServerItem.vue';
import ServerForm from './ServerForm.vue';
import TerminalPanel from './TerminalPanel.vue';
import SftpPanel from './SftpPanel.vue';
import GroupTree from './GroupTree.vue';
import Modal from '../../components/ui/Modal.vue';
import { useToast } from '../../composables/useToast';
import { useErrorHandler } from '../../composables/useErrorHandler';
import type { Server } from '../../types';

const toast = useToast();
const { handleError } = useErrorHandler();

const servers = ref<Server[]>([]);
const groups = ref<Array<{ id: string; name: string; color: string; parentId: string | null }>>([]);
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
const selectedGroup = ref('');
const expandedGroups = ref(new Set<string | null>([null])); // 默认全部展开 —— onMounted 后会被 expandAllGroups() 覆盖
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
  loadGroups().then(() => {
    // 加载完成后默认展开所有分组
    expandAllGroups();
  });

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
    if (type === 'servers') refreshServers();
  });
  if (cleanupDataChanged) _cleanupDataChanged = cleanupDataChanged;
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
  await loadGroups().then(() => {
    // 刷新后保持全展开
    expandAllGroups();
  });
}

// 搜索过滤
const allFilteredServers = computed(() => {
  let filtered = servers.value;
  if (searchQuery.value.trim()) {
    const q = searchQuery.value.trim().toLowerCase();
    filtered = filtered.filter(
      (s) =>
        s.name.toLowerCase().includes(q) ||
        s.host.toLowerCase().includes(q)
    );
  }
  return filtered;
});

// 根分组（没有父分组的）
const rootGroups = computed(() => {
  return groups.value.filter(g => !g.parentId);
});

// 获取子分组
function getChildGroups(groupId: string) {
  return groups.value.filter(g => g.parentId === groupId);
}

function getServersByGroup(groupId) {
  return servers.value.filter((s) => (s.groupId || null) === (groupId || null));
}

function getOnlineCount(groupId) {
  return getServersByGroup(groupId).filter(s => connectionStatusMap.value[s.id] === 'online').length;
}

function getFilteredServers(serverList) {
  if (!searchQuery.value.trim()) return serverList;
  const q = searchQuery.value.trim().toLowerCase();
  return serverList.filter(
    (s) =>
      s.name.toLowerCase().includes(q) ||
      s.host.toLowerCase().includes(q)
  );
}

function expandAllGroups() {
  const allIds = new Set<string | null>([null]); // null = 未分组
  for (const g of groups.value) {
    allIds.add(g.id);
  }
  expandedGroups.value = allIds;
}

function collapseAllGroups() {
  expandedGroups.value = new Set<string | null>();
}

function toggleGroup(groupId) {
  if (expandedGroups.value.has(groupId)) {
    expandedGroups.value.delete(groupId);
  } else {
    expandedGroups.value.add(groupId);
  }
  expandedGroups.value = new Set(expandedGroups.value);
}

function openTerminal(server) {
  terminalServer.value = server;
}
function openSftp(server: Server, initialPath?: string) {
  // 如果该服务器的 SFTP 已打开，不重复打开
  if (sftpPanels.value.some(p => p.server.id === server.id)) return;
  // 为每个新面板生成级联偏移位置
  const idx = sftpPanels.value.length;
  const id = `sftp-${server.id}-${Date.now()}`;
  sftpPanels.value.push({
    id,
    server,
    initialPath: initialPath || '',
    position: { x: window.innerWidth - 620 - (idx * 30), y: 80 + (idx * 30) }
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

function editServer(server) {
  editingServer.value = server;
  // BUG 6 FIX: Do NOT populate password from server object — leave it blank
  // so user can optionally set a new password. Existing password is preserved on save.
  const { password, ...serverWithoutPassword } = server;
  serverForm.value = {
    ...serverWithoutPassword,
    password: '',
    tagsInput: server.tags?.join(',') || '',
  };
}

async function deleteServer(serverId) {
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
  if (!newGroupName.value.trim()) return;
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
  const group = groups.value.find(g => g.id === groupId);
  if (!group) return;
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
function getGroupDepth(group: { parentId: string | null }): number {
  let depth = 0;
  let current = group.parentId;
  while (current) {
    depth++;
    const parent = groups.value.find(g => g.id === current);
    current = parent?.parentId || null;
  }
  return depth;
}

async function deleteGroup(groupId) {
  try {
    await getTauriAPI().deleteServerGroup(groupId);
    await loadGroups();
    await loadServers();
    if (selectedGroup.value === groupId) {
      selectedGroup.value = '';
    }
    toast.success('分组已删除');
  } catch (error) {
    handleError(error, { context: 'deleteGroup' });
  }
}

async function testConnection() {
  testResult.value = null;
  try {
    const server = {
      ...serverForm.value,
      tags: serverForm.value.tagsInput
        .split(',')
        .map((t) => t.trim())
        .filter((t) => t),
    };
    testResult.value = await getTauriAPI().testServerConnection(server);
  } catch (error) {
    testResult.value = { success: false, error: error.message };
  }
}

async function saveServer() {
  const now = new Date().toISOString();
  const server = {
    id: serverForm.value.id || Date.now().toString(),
    name: serverForm.value.name,
    host: serverForm.value.host,
    port: serverForm.value.port,
    username: serverForm.value.username,
    sshKeyPath: serverForm.value.sshKeyPath,
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
