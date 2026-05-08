<template>
  <div class="server-manager">
    <h2>服务器管理</h2>

    <div class="server-toolbar">
      <button @click="showAddServer = true" class="btn-add">+ 添加服务器</button>
      <button @click="refreshServers" class="btn-refresh">🔄 刷新</button>
      <button @click="showGroupManager = true" class="btn-groups">📁 管理分组</button>
      <div class="toolbar-separator"></div>
      <button @click="expandAllGroups" class="btn-toggle-groups" title="全部展开">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.5">
          <polyline points="6 9 12 15 18 9"/>
        </svg>
        全部展开
      </button>
      <button @click="collapseAllGroups" class="btn-toggle-groups" title="全部折叠">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.5">
          <polyline points="6 15 12 9 18 15"/>
        </svg>
        全部折叠
      </button>
    </div>

    <!-- 搜索和分组筛选 -->
    <div class="server-filters">
      <div class="search-box">
        <svg class="search-icon" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"/>
          <path d="m21 21-4.35-4.35"/>
        </svg>
        <input
          v-model="searchQuery"
          placeholder="搜索服务器名称或地址..."
          class="search-input"
        />
      </div>
      <select v-model="selectedGroup" class="group-filter">
        <option value="">全部分组</option>
        <template v-for="group in groups" :key="group.id">
          <option v-if="!group.parentId" :value="group.id">
            {{ group.name }}
          </option>
          <option v-else :value="group.id" class="group-suboption">
            {{ '  ' + '└ ' + group.name }}
          </option>
        </template>
      </select>
    </div>

    <!-- 按分组树形折叠显示 -->
    <template v-if="selectedGroup === ''">
      <div v-if="getServersByGroup(null).length > 0" class="drawer-group" :class="{ 'drawer-expanded': expandedGroups.has(null) }">
        <div class="drawer-handle" @click="toggleGroup(null)">
          <div class="drawer-handle-left">
            <svg class="drawer-chevron" :class="{ expanded: expandedGroups.has(null) }" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="6 9 12 15 18 9"/>
            </svg>
            <span class="drawer-icon">🖥️</span>
            <span class="drawer-name">未分组</span>
            <span class="drawer-count">{{ getServersByGroup(null).length }}</span>
          </div>
          <div class="drawer-handle-right">
            <span class="drawer-online" v-if="getOnlineCount(null) > 0">
              <span class="online-dot"></span>
              {{ getOnlineCount(null) }} 在线
            </span>
          </div>
        </div>
        <Transition name="drawer-expand">
          <div v-show="expandedGroups.has(null)" class="drawer-body">
            <div class="drawer-servers">
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
      <div class="server-list">
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

    <div v-if="allFilteredServers.length === 0" class="empty-state">
      <template v-if="servers.length === 0 && !searchQuery">
        <svg class="empty-state-icon" viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="currentColor" stroke-width="1.5">
          <rect x="2" y="2" width="20" height="8" rx="2" ry="2" />
          <rect x="2" y="14" width="20" height="8" rx="2" ry="2" />
          <line x1="6" y1="6" x2="6.01" y2="6" />
          <line x1="6" y1="18" x2="6.01" y2="18" />
        </svg>
        <p class="empty-state-title">暂无服务器</p>
        <p class="empty-state-subtitle">点击上方「添加服务器」按钮，管理你的远程服务器</p>
        <button @click="showAddServer = true" class="empty-state-action-btn">+ 添加第一个服务器</button>
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
      <div class="group-manager">
        <div class="group-manager-list">
          <div v-for="group in groups" :key="group.id" class="group-manager-item" :style="{ paddingLeft: `${getGroupDepth(group) * 20}px` }">
            <span v-if="group.parentId" class="group-indent">└ </span>
            <span class="group-color-dot" :style="{ backgroundColor: group.color || '#6c63ff' }"></span>
            <span class="group-manager-name">{{ group.name }}</span>
            <span class="group-manager-count">{{ getServersByGroup(group.id).length }} 台</span>
            <button @click="addGroupAsChild(group.id)" class="btn-add-subgroup" title="添加子分组">+</button>
            <button @click="editGroup(group.id)" class="btn-edit-group" title="编辑">✎</button>
            <button @click="deleteGroup(group.id)" class="btn-delete-group" title="删除">✕</button>
          </div>
          <div v-if="groups.length === 0" class="empty-group-msg">暂无分组</div>
        </div>
        <div class="group-manager-form">
          <h4>{{ editingGroupId ? '编辑分组' : (addingChildTo ? '添加子分组' : '添加新分组') }}</h4>
          <div class="group-form-row">
            <input v-model="newGroupName" placeholder="分组名称" class="form-input" @keyup.enter="saveGroup" />
            <input v-model="newGroupColor" type="color" class="color-picker" value="#6c63ff" />
          </div>
          <div v-if="!editingGroupId" class="group-form-row">
            <select v-model="newGroupParent" class="form-select">
              <option :value="null">无父分组（顶级）</option>
              <option v-for="group in groups" :key="group.id" :value="group.id">
                {{ group.name }}
              </option>
            </select>
          </div>
          <div class="group-form-actions">
            <button @click="saveGroup" class="btn-add-group" :disabled="!newGroupName.trim()">{{ editingGroupId ? '保存' : '添加' }}</button>
            <button v-if="editingGroupId || addingChildTo" @click="cancelEditGroup" class="btn-cancel-group">取消</button>
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
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import ServerItem from '@/components/server/ServerItem.vue';
import ServerForm from '@/components/server/ServerForm.vue';
import TerminalPanel from '@/components/server/TerminalPanel.vue';
import SftpPanel from './server/SftpPanel.vue';
import GroupTree from './server/GroupTree.vue';
import Modal from '@/components/ui/Modal.vue';
import { getTauriAPI } from '../utils/tauri-api';
import { useToast } from '../composables/useToast';
import { useErrorHandler } from '../composables/useErrorHandler';
import type { Server } from '../types';

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
  if (!searchQuery.value.trim()) return serverList;
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
  if (sftpPanels.value.some(p => p.server.id === server.id)) return;
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
  if (!newGroupName.value.trim()) return;
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

<style scoped>
.server-manager {
  padding: 12px 16px;
}

.server-toolbar {
  display: flex;
  gap: 6px;
  margin-bottom: 12px;
  align-items: center;
  flex-wrap: wrap;
}

.btn-add,
.btn-groups {
  padding: 10px 20px;
  border-radius: 6px;
  cursor: pointer;
  background: oklch(var(--p));
  color: white;
  border: none;
  font-size: 14px;
}

.btn-groups {
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  border: 1px solid oklch(var(--bc) / 0.1);
}

.btn-refresh {
  padding: 10px 20px;
  border-radius: 6px;
  cursor: pointer;
  background: oklch(var(--b1));
  color: oklch(var(--bc));
  border: 1px solid oklch(var(--bc) / 0.1);
  font-size: 14px;
}

.toolbar-separator {
  width: 1px;
  height: 28px;
  background: oklch(var(--bc) / 0.1);
  margin: 0 4px;
}

.btn-toggle-groups {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 8px 14px;
  border-radius: 6px;
  cursor: pointer;
  background: transparent;
  color: oklch(var(--bc) / 0.6);
  border: 1px solid oklch(var(--bc) / 0.1);
  font-size: 12px;
  transition: all 0.15s ease;
}

.btn-toggle-groups:hover {
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  border-color: oklch(var(--p));
}

/* 抽屉式分组 */
.drawer-group {
  margin-bottom: 4px;
  border-radius: 10px;
}

.drawer-group.drawer-expanded {
  margin-bottom: 8px;
}

.drawer-handle {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 7px 12px;
  border-radius: 8px;
  cursor: pointer;
  user-select: none;
  transition: all 0.2s ease;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  position: relative;
  overflow: hidden;
}

.drawer-handle::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
  border-radius: 0 3px 3px 0;
  background: linear-gradient(180deg, #6c63ff, #4834d4);
}

.drawer-handle:hover {
  border-color: oklch(var(--p));
  box-shadow: 0 2px 12px rgba(108, 99, 255, 0.1);
  transform: translateY(-1px);
}

.drawer-handle-left {
  display: flex;
  align-items: center;
  gap: 8px;
  position: relative;
  z-index: 1;
}

.drawer-chevron {
  color: oklch(var(--bc) / 0.6);
  transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  flex-shrink: 0;
}

.drawer-chevron.expanded {
  transform: rotate(180deg);
  color: oklch(var(--p));
}

.drawer-icon {
  font-size: 14px;
  line-height: 1;
}

.drawer-name {
  font-weight: 600;
  font-size: 13px;
  color: oklch(var(--bc));
}

.drawer-count {
  font-size: 11px;
  font-weight: 600;
  padding: 1px 7px;
  border-radius: 12px;
  background: #6c63ff22;
  color: #6c63ff;
  line-height: 1.4;
}

.drawer-handle-right {
  display: flex;
  align-items: center;
  gap: 8px;
  position: relative;
  z-index: 1;
}

.drawer-online {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: oklch(var(--su));
  font-weight: 500;
}

.drawer-body {
  margin-top: 4px;
  padding: 8px 10px;
  border-radius: 8px;
  background: color-mix(in srgb, oklch(var(--b1)) 80%, oklch(var(--bc) / 0.1) 20%);
  border: 1px solid oklch(var(--bc) / 0.1);
  border-top: none;
}

.drawer-servers {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 8px;
}

/* 展开/折叠动画 */
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

/* 搜索和筛选 */
.server-filters {
  display: flex;
  gap: 12px;
  margin-bottom: 20px;
  align-items: center;
}

.search-box {
  flex: 1;
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 12px;
  color: oklch(var(--bc) / 0.6);
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: 10px 12px 10px 36px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  font-size: 14px;
  transition: border-color 0.15s ease;
}

.search-input:focus {
  outline: none;
  border-color: oklch(var(--p));
}

.group-filter {
  padding: 10px 12px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  font-size: 14px;
  min-width: 150px;
}

/* 单分组筛选视图 */
.server-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 8px;
}

.empty-state {
  text-align: center;
  padding: 40px;
  color: oklch(var(--bc) / 0.6);
  background: oklch(var(--b1));
  border-radius: 12px;
}
.empty-state-icon { opacity: 0.2; margin-bottom: 16px; }
.empty-state-title { font-size: 16px; font-weight: 600; color: oklch(var(--bc)); margin: 0 0 8px 0; }
.empty-state-subtitle { font-size: 13px; margin: 0 0 16px 0; }
.empty-state-action-btn {
  padding: 10px 24px;
  border-radius: 8px;
  cursor: pointer;
  background: oklch(var(--p));
  color: white;
  border: none;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s ease;
}
.empty-state-action-btn:hover { opacity: 0.9; transform: translateY(-1px); }

/* 分组管理器 */
.group-manager {
  padding: 8px 0;
}

.group-manager-list {
  margin-bottom: 20px;
}

.group-manager-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 8px;
  background: oklch(var(--b2));
  margin-bottom: 8px;
}

.group-color-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}

.group-manager-name {
  flex: 1;
  font-weight: 500;
}

.group-manager-count {
  font-size: 12px;
  color: oklch(var(--bc) / 0.6);
}

.btn-add-subgroup,
.btn-edit-group {
  padding: 4px 8px;
  border-radius: 4px;
  cursor: pointer;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  border: 1px solid oklch(var(--bc) / 0.1);
  font-size: 12px;
}

.btn-delete-group {
  padding: 4px 12px;
  border-radius: 4px;
  cursor: pointer;
  background: oklch(var(--er));
  color: white;
  border: none;
  font-size: 12px;
}

.empty-group-msg {
  text-align: center;
  padding: 20px;
  color: oklch(var(--bc) / 0.6);
}

.group-manager-form {
  border-top: 1px solid oklch(var(--bc) / 0.1);
  padding-top: 16px;
}

.group-manager-form h4 {
  margin: 0 0 12px;
  font-size: 14px;
  color: oklch(var(--bc));
}

.group-form-row {
  display: flex;
  gap: 10px;
  margin-bottom: 10px;
}

.group-form-row .form-input,
.group-form-row .form-select {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 6px;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  font-size: 14px;
}

.color-picker {
  width: 40px;
  height: 36px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 6px;
  cursor: pointer;
  padding: 2px;
  background: oklch(var(--b2));
}

.group-form-actions {
  display: flex;
  gap: 8px;
}

.btn-add-group {
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
  background: oklch(var(--p));
  color: white;
  border: none;
  font-size: 13px;
}

.btn-add-group:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-cancel-group {
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  border: 1px solid oklch(var(--bc) / 0.1);
  font-size: 13px;
}
</style>
