<template>
  <div class="p-3 px-4">
    <h2 class="text-xl font-bold text-base-content mb-4">服务器管理</h2>

    <div class="flex gap-1.5 mb-3 items-center flex-wrap">
      <button @click="showAddServer = true" class="btn btn-primary btn-sm gap-1">+ 添加服务器</button>
      <button @click="refreshServers" class="btn btn-ghost btn-sm gap-1.5"><SvgIcon name="refresh" size="14" /> 刷新</button>
      <button @click="showGroupManager = true" class="btn btn-ghost btn-sm gap-1.5"><SvgIcon name="folder" size="14" /> 管理分组</button>
      <div class="w-px h-7 bg-base-content/10 mx-1"></div>
      <button @click="expandAllGroups" class="btn btn-ghost btn-sm gap-1" title="全部展开">
        <SvgIcon name="chevronDown" size="14" stroke-width="2.5" />
        全部展开
      </button>
      <button @click="collapseAllGroups" class="btn btn-ghost btn-sm gap-1" title="全部折叠">
        <SvgIcon name="chevronUp" size="14" stroke-width="2.5" />
        全部折叠
      </button>
    </div>

    <!-- 搜索和分组筛选 -->
    <div class="flex gap-3 mb-5 items-center">
      <div class="flex-1 relative flex items-center">
        <SvgIcon name="search" size="16" class="absolute left-3 text-base-content/60 pointer-events-none" />
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
      <!-- 未分组 - IDEA 风格简洁标题栏 -->
      <div v-if="getServersByGroup(null).length > 0" class="mb-2">
        <div class="flex items-center gap-2 px-3 py-1.5 rounded cursor-pointer select-none transition-colors"
          :class="expandedGroups.has(null) ? 'bg-base-100' : 'hover:bg-base-100/50'"
          @click="toggleGroup(null)">
          <SvgIcon name="chevronDown" size="12" stroke-width="2.5" class="text-base-content/50 transition-transform flex-shrink-0" :class="{ 'rotate-180': expandedGroups.has(null) }" />
          <span class="w-2 h-2 rounded-full flex-shrink-0 bg-primary"></span>
          <span class="font-medium text-[11px] text-base-content">未分组</span>
          <span class="text-[10px] px-1.5 py-0 rounded bg-base-200 text-base-content/60 leading-tight">{{ getServersByGroup(null).length }}</span>
          <span class="flex items-center gap-1 text-[10px] text-success ml-auto" v-if="getOnlineCount(null) > 0">
            <span class="w-1 h-1 rounded-full bg-success"></span>
            {{ getOnlineCount(null) }}
          </span>
        </div>
        <Transition
          enter-active-class="transition-all duration-200 ease-out"
          leave-active-class="transition-all duration-200 ease-in"
          enter-from-class="opacity-0 max-h-0"
          leave-to-class="opacity-0 max-h-0"
        >
          <div v-show="expandedGroups.has(null)" class="mt-1 pl-5">
            <div class="grid grid-cols-[repeat(auto-fill,minmax(180px,1fr))] gap-1.5">
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

    <div v-if="allFilteredServers.length === 0" class="text-center p-10 text-base-content/60 bg-base-100 rounded-xl">
      <template v-if="servers.length === 0 && !searchQuery">
        <SvgIcon class="opacity-20 mb-4 mx-auto" name="serverRack" size="48" stroke-width="1.5" />
        <p class="text-base font-semibold text-base-content m-0 mb-2">暂无服务器</p>
        <p class="text-sm m-0 mb-4">点击上方「添加服务器」按钮，管理你的远程服务器</p>
        <button @click="showAddServer = true" class="btn btn-primary">+ 添加第一个服务器</button>
      </template>
      <template v-else>
        <p class="m-0">{{ searchQuery ? '没有找到匹配的服务器' : '暂无服务器，点击上方按钮添加' }}</p>
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
          <div v-for="group in groups" :key="group.id" class="flex items-center gap-2.5 p-3 rounded-xl bg-base-200 mb-2" :style="{ paddingLeft: `${getGroupDepth(group) * 20 + 12}px` }">
            <span v-if="group.parentId" class="text-base-content/60">└ </span>
            <span class="w-2.5 h-2.5 rounded-full flex-shrink-0" :style="{ backgroundColor: group.color || '#6c63ff' }"></span>
            <span class="flex-1 font-medium text-sm">{{ group.name }}</span>
            <span class="text-xs text-base-content/60">{{ getServersByGroup(group.id).length }} 台</span>
            <button @click="addGroupAsChild(group.id)" class="btn btn-ghost btn-xs" title="添加子分组">+</button>
            <button @click="editGroup(group.id)" class="btn btn-ghost btn-xs px-1" title="编辑"><SvgIcon name="pencil" size="12" /></button>
            <button @click="deleteGroup(group.id)" class="btn btn-error btn-xs px-1" title="删除"><SvgIcon name="trash" size="12" /></button>
          </div>
          <div v-if="groups.length === 0" class="text-center p-5 text-base-content/60">暂无分组</div>
        </div>
        <div class="border-t border-base-content/10 pt-4">
          <h4 class="text-sm text-base-content mb-3 m-0">{{ editingGroupId ? '编辑分组' : (addingChildTo ? '添加子分组' : '添加新分组') }}</h4>
          <div class="flex gap-2.5 mb-2.5">
            <input v-model="newGroupName" placeholder="分组名称" class="input input-bordered flex-1" @keyup.enter="saveGroup" />
            <input v-model="newGroupColor" type="color" class="w-10 h-9 border border-base-content/10 rounded-lg cursor-pointer p-0.5 bg-base-200" value="#6c63ff" />
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
            <button @click="saveGroup" class="btn btn-primary btn-sm" :disabled="!newGroupName.trim()">{{ editingGroupId ? '保存' : '添加' }}</button>
            <button v-if="editingGroupId || addingChildTo" @click="cancelEditGroup" class="btn btn-ghost btn-sm">取消</button>
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
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import ServerItem from '@/views/server/ServerItem.vue';
import ServerForm from '@/views/server/ServerForm.vue';
import TerminalPanel from '@/views/server/TerminalPanel.vue';
import SftpPanel from './SftpPanel.vue';
import GroupTree from './GroupTree.vue';
import Modal from '@/components/ui/Modal.vue';
import { getTauriAPI } from '../../utils/tauri-api';
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
const expandedGroups = ref(new Set<string | null>([null]));
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
    expandAllGroups();
  });

  getTauriAPI().onServerConnected?.((data: any) => {
    connectionStatusMap.value[data.serverId] = 'online';
  });

  getTauriAPI().onServerDisconnected?.((data: any) => {
    delete connectionStatusMap.value[data.serverId];
  });

  getTauriAPI().onServerHeartbeatFailed?.((data: any) => {
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
  await loadGroups().then(() => {
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

function getServersByGroup(groupId: string | null): Server[] {
  return servers.value.filter((s) => (s.groupId || null) === (groupId || null));
}

function getOnlineCount(groupId: string | null): number {
  return getServersByGroup(groupId).filter(s => connectionStatusMap.value[s.id] === 'online').length;
}

function getFilteredServers(serverList: Server[]): Server[] {
  if (!searchQuery.value.trim()) {return serverList;}
  const q = searchQuery.value.trim().toLowerCase();
  return serverList.filter(
    (s) =>
      s.name.toLowerCase().includes(q) ||
      s.host.toLowerCase().includes(q)
  );
}

function expandAllGroups() {
  const allIds = new Set<string | null>([null]);
  for (const g of groups.value) {
    allIds.add(g.id);
  }
  expandedGroups.value = allIds;
}

function collapseAllGroups() {
  expandedGroups.value = new Set<string | null>();
}

function toggleGroup(groupId: string | null) {
  if (expandedGroups.value.has(groupId)) {
    expandedGroups.value.delete(groupId);
  } else {
    expandedGroups.value.add(groupId);
  }
  expandedGroups.value = new Set(expandedGroups.value);
}

function openTerminal(server: Server) {
  terminalServer.value = server;
}
function openSftp(server: Server, initialPath?: string) {
  if (sftpPanels.value.some(p => p.server.id === server.id)) {return;}
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

function onTerminalOpenSftp(server: Server, path: string) {
  openSftp(server, path);
}

function editServer(server: Server) {
  editingServer.value = server;
  const { password, ...serverWithoutPassword } = server;
  serverForm.value = {
    ...serverWithoutPassword,
    password: '',
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
      await getTauriAPI().updateServerGroup(editingGroupId.value, {
        name: newGroupName.value.trim(),
        color: newGroupColor.value,
      });
      toast.success('分组已更新');
    } else {
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

async function deleteGroup(groupId: string) {
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
        .map((t: string) => t.trim())
        .filter((t: string) => t),
    };
    testResult.value = await getTauriAPI().testServerConnection(server);
  } catch (error: any) {
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
    password: serverForm.value.password || undefined,
    tags: serverForm.value.tagsInput
      .split(',')
      .map((t: string) => t.trim())
      .filter((t: string) => t),
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
