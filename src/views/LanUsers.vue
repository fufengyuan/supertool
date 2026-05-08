<template>
  <div class="lan-panel">
    <!-- 我的资料卡片 -->
    <div class="profile-card" @click="showProfileEditor = !showProfileEditor">
      <div class="profile-bg"></div>
      <div class="profile-content">
        <div class="profile-avatar">
          <span class="avatar-emoji">{{ myAvatar }}</span>
          <span class="my-status-dot" :class="currentStatus"></span>
        </div>
        <div class="profile-info">
          <div class="profile-name">
            {{ myDisplayName }}
            <svg class="edit-icon" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
          </div>
          <div class="profile-id">{{ myUserId }}</div>
        </div>
        <div class="status-pills" @click.stop>
          <span class="pill" :class="{ active: currentStatus === 'online' }" @click="setStatus('online')" title="在线">
            <span class="pill-dot online"></span>在线
          </span>
          <span class="pill" :class="{ active: currentStatus === 'busy' }" @click="setStatus('busy')" title="忙碌">
            <span class="pill-dot busy"></span>忙碌
          </span>
          <span class="pill" :class="{ active: currentStatus === 'away' }" @click="setStatus('away')" title="离开">
            <span class="pill-dot away"></span>离开
          </span>
        </div>
      </div>
    </div>

    <!-- 资料编辑面板 -->
    <Transition name="slide">
      <div v-if="showProfileEditor" class="editor-panel">
        <div class="editor-header">编辑资料</div>
        <div class="editor-body">
          <div class="editor-section">
            <label>头像</label>
            <div class="emoji-grid">
              <span v-for="emoji in avatarOptions" :key="emoji"
                    class="emoji-opt"
                    :class="{ selected: editAvatar === emoji }"
                    @click="editAvatar = emoji">{{ emoji }}</span>
            </div>
          </div>
          <div class="editor-section">
            <label>昵称</label>
            <input v-model="editNickName" class="editor-input" placeholder="留空使用系统用户名" @keydown.enter="saveProfile" />
          </div>
          <div class="editor-footer">
            <button class="btn-ghost btn-sm" @click="showProfileEditor = false">取消</button>
            <button class="btn-primary btn-sm" @click="saveProfile">保存</button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 网络权限提示（macOS Local Network Privacy） -->
    <div v-if="permissionWarning" class="permission-banner" :class="permissionWarning.type">
      <div class="banner-content">
        <span class="banner-icon">{{ permissionWarning.type === 'error' ? '🚫' : '⚠️' }}</span>
        <div class="banner-text">
          <div class="banner-title">{{ permissionWarning.title }}</div>
          <div class="banner-detail">{{ permissionWarning.message }}</div>
        </div>
      </div>
      <div class="banner-actions">
        <button class="btn-retry" @click="recheckPermission" :disabled="checkingPermission">
          {{ checkingPermission ? '检测中...' : '重新检测' }}
        </button>
      </div>
    </div>

    <!-- 头部 -->
    <div class="section-header">
      <div class="header-left">
        <span class="header-icon">🌐</span>
        <span class="header-title">局域网用户</span>
        <span class="online-badge" :class="{ hasPeers: peers.length > 0 }">
          <span class="badge-pulse" v-if="peers.length > 0"></span>
          {{ peers.length }}
        </span>
      </div>
      <button class="scan-btn" @click="refreshDiscovery" :class="{ scanning }" title="重新扫描">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 2v6h-6M3 12a9 9 0 0 1 15-6.7L21 8M3 22v-6h6M21 12a9 9 0 0 1-15 6.7L3 16"/>
        </svg>
      </button>
    </div>

    <!-- 用户列表 -->
    <div class="user-list" v-if="peers.length > 0">
      <div
        class="user-card"
        v-for="peer in sortedPeers"
        :key="peer.id"
        @click="openChat(peer)"
        :class="{ selected: selectedPeer?.id === peer.id, hasUnread: unreadCounts[peer.id] > 0 }"
      >
        <div class="unread-indicator" v-if="unreadCounts[peer.id] > 0"></div>
        <div class="user-avatar-wrap">
          <div class="user-avatar">
            <span>{{ peer.avatar || '😀' }}</span>
          </div>
          <span class="user-status-dot" :class="getStatusClass(peer)"></span>
        </div>
        <div class="user-info">
          <div class="user-name-row">
            <span class="user-name">{{ peer.name }}</span>
            <span v-if="unreadCounts[peer.id] > 0" class="msg-badge">
              {{ unreadCounts[peer.id] > 99 ? '99+' : unreadCounts[peer.id] }}
            </span>
          </div>
          <div class="user-meta">
            <span class="user-status-text">{{ getStatusText(peer) }}</span>
            <span class="user-sep">·</span>
            <span class="user-address">{{ peer.address }}</span>
            <span v-if="peer.version" class="user-version" :class="{ 'version-incompatible': peer.version && peer.version.split('.')[0] !== '2' }" :title="peer.version === '2.0' ? '✅ 兼容' : '⚠️ 版本过低，无法发送消息'">v{{ peer.version }}</span>
          </div>
        </div>
        <div class="user-action">
          <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18l6-6-6-6"/></svg>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div class="empty-state" v-else>
      <div class="radar-container">
        <div class="radar">
          <div class="radar-sweep"></div>
          <div class="radar-ring"></div>
          <div class="radar-dot"></div>
        </div>
      </div>
      <p class="empty-title">正在搜索局域网用户</p>
      <p class="empty-hint">确保其他设备在同一 WiFi 网络下</p>
      <details class="network-details">
        <summary>网络信息</summary>
        <div class="network-info" v-if="networkInfo">
          <div class="info-row">
            <span class="info-label">本机 IP</span>
            <span class="info-value">{{ networkInfo.address }}</span>
          </div>
          <div class="info-row">
            <span class="info-label">端口</span>
            <span class="info-value">{{ networkInfo.ports }}</span>
          </div>
          <div class="info-row">
            <span class="info-label">版本</span>
            <span class="info-value">v{{ networkInfo.version }}</span>
          </div>
        </div>
      </details>
    </div>

    <!-- 文件接收路径设置 -->
    <div class="receive-path-section">
      <div class="path-label">📂 文件保存路径</div>
      <div class="path-row">
        <span class="path-value" :title="receivePath">{{ receivePath || '默认' }}</span>
        <button class="path-btn" @click="chooseReceivePath">选择目录</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { getTauriAPI } from '@/utils/tauri-api'

interface LanPeer {
  id: string;
  name: string;
  avatar?: string;
  address: string;
  messagePort?: number;
  version?: string;
  fileTransferPort?: number;
  fileTransferUrl?: string;
  lastSeen?: number;
  online?: boolean;
  status?: 'online' | 'busy' | 'away';
}

const peers = ref<LanPeer[]>([]);
const selectedPeer = ref<LanPeer | null>(null);
const unreadCounts = ref<Record<string, number>>({});
const lastSeenTimes = ref<Record<string, number>>({});
const scanning = ref(false);
const networkInfo = ref<{ address: string; ports: string; version: string } | null>(null);
const receivePath = ref<string>('');

// 网络权限状态（macOS Local Network Privacy）
const permissionWarning = ref<{ type: 'warning' | 'error'; title: string; message: string } | null>(null)
const checkingPermission = ref(false)

// 我的资料
const myUserId = ref('');
const myDisplayName = ref('');
const myAvatar = ref('😀');
const currentStatus = ref<'online' | 'busy' | 'away'>('online');
const showProfileEditor = ref(false);
const editNickName = ref('');
const editAvatar = ref('😀');
const avatarOptions = ['😀','😎','🤓','👨‍💻','👩‍💻','🐱','🐶','🦊','🐼','🐨','🦁','🐸','🐵','🤖','👾','🎮'];
const emit = defineEmits<{
  'select-peer': [peer: LanPeer];
  'open-chat': [peer: LanPeer];
  'open-assign': [peer: LanPeer];
}>();

// 排序：有未读消息 > 在线 > 按名称
const sortedPeers = computed(() => {
  return [...peers.value].sort((a, b) => {
    const aUnread = unreadCounts.value[a.id] || 0;
    const bUnread = unreadCounts.value[b.id] || 0;
    if (aUnread > 0 && bUnread === 0) return -1;
    if (aUnread === 0 && bUnread > 0) return 1;
    const aOnline = getStatusClass(a) === 'online' ? 1 : 0;
    const bOnline = getStatusClass(b) === 'online' ? 1 : 0;
    if (bOnline !== aOnline) return bOnline - aOnline;
    // Fallback: alphabetical by name
    return a.name.localeCompare(b.name);
  });
});

// 状态相关
const getStatusClass = (peer: LanPeer): string => {
  if (peer.status === 'busy') return 'busy';
  if (peer.status === 'away') return 'away';
  if (peer.online === false) return 'offline';
  const lastSeen = peer.lastSeen || lastSeenTimes.value[peer.id];
  if (lastSeen) {
    const diff = Date.now() - lastSeen;
    if (diff > 5 * 60 * 1000) return 'away';
  }
  return 'online';
};

const getStatusText = (peer: LanPeer): string => {
  const statusClass = getStatusClass(peer);
  const map: Record<string, string> = { online: '在线', busy: '忙碌', away: '离开', offline: '离线' };
  return map[statusClass] || '在线';
};

async function setStatus(status: 'online' | 'busy' | 'away') {
  currentStatus.value = status;
  try {
    await getTauriAPI().setStatus(status);
  } catch (e) {
    console.warn('Failed to change status:', e);
  }
}

/** 检测 macOS 局域网权限 */
async function checkNetworkPermission() {
  if (!getTauriAPI().checkNetworkPermission) return
  checkingPermission.value = true
  try {
    const result = await getTauriAPI().checkNetworkPermission("0.0.0.0", 0)
    if (!result.success) {
      // 判断是否为 macOS TCC 阻止
      const isTccBlocked = result.error?.includes('EHOSTUNREACH') ||
        result.error?.includes('Local Network Privacy') ||
        result.error?.includes('blocked') ||
        result.error?.includes('Permission')
      permissionWarning.value = {
        type: isTccBlocked ? 'error' : 'warning',
        title: isTccBlocked ? '局域网访问被阻止' : '网络权限检测异常',
        message: isTccBlocked
          ? `macOS 局域网隐私设置阻止了消息发送。请前往：系统设置 → 隐私与安全性 → 本地网络 → 启用 SuperTool。或临时关闭防火墙测试。`
          : result.error || '无法确认网络权限状态'
      }
    } else {
      permissionWarning.value = null
    }
  } catch (e) {
    permissionWarning.value = {
      type: 'warning',
      title: '权限检测失败',
      message: `检测异常: ${e instanceof Error ? e.message : String(e)}`
    }
  } finally {
    checkingPermission.value = false
  }
}

async function recheckPermission() {
  await checkNetworkPermission()
}

async function refreshDiscovery() {
  scanning.value = true;
  try {
    await getTauriAPI().refreshDiscovery();
    setTimeout(() => { scanning.value = false }, 3000);
  } catch (e) {
    scanning.value = false;
    console.warn('Failed to refresh discovery:', e);
  }
}

// 未读消息计数刷新（模块级，供 onMounted / onUnmounted 共用）
async function loadUnreadCounts() {
  try {
    const userInfo = await getTauriAPI().getUserInfo('');
    const counts = await getTauriAPI().getAllUnreadCounts();
    unreadCounts.value = counts;
  } catch (e) {
    console.warn('Failed to load unread counts:', e);
  }
}

// IPC 监听器清理函数
let cleanupIpcListeners: (() => void)[] = [];

onMounted(async () => {
  await loadMyProfile();
  try {
    const statusInfo = await getTauriAPI().getStatus('');
    currentStatus.value = (statusInfo.status as 'online' | 'busy' | 'away') || 'online';
  } catch {}
  try {
    networkInfo.value = await getTauriAPI().getNetworkInfo();
  } catch {}
  try {
    receivePath.value = await getTauriAPI().getReceivePath();
  } catch {}
  peers.value = (await getTauriAPI().getPeers()) as LanPeer[];

  // Check macOS Local Network Privacy permission
  await checkNetworkPermission()

  peers.value.forEach(p => {
    if (p.lastSeen) {
      lastSeenTimes.value[p.id] = p.lastSeen;
    } else {
      lastSeenTimes.value[p.id] = Date.now();
    }
  });

  await loadUnreadCounts();
  window.addEventListener('lan:reload-unread', loadUnreadCounts);

  cleanupIpcListeners.push(getTauriAPI().onLanPeerDiscovered((peer: any) => {
    const exists = peers.value.find(p => p.id === peer.id);
    if (!exists) {
      peers.value.push(peer);
      lastSeenTimes.value[peer.id] = Date.now();
    } else {
      // 更新已知 peer 的状态（心跳刷新）
      exists.lastSeen = peer.lastSeen;
      exists.online = peer.online;
      if (peer.avatar) exists.avatar = peer.avatar;
      if (peer.status) exists.status = peer.status;
      if (peer.version) exists.version = peer.version;
      if (peer.messagePort) exists.messagePort = peer.messagePort;
    }
  }));

  cleanupIpcListeners.push(getTauriAPI().onLanPeerLost((peer: any) => {
    peers.value = peers.value.filter((p) => p.id !== peer.id);
    if (selectedPeer.value?.id === peer.id) {
      selectedPeer.value = null;
    }
  }));

  cleanupIpcListeners.push(getTauriAPI().onMessage((data: any) => {
    if (data && data.from) {
      const senderId = data.from;
      // Only increment unread if this peer's chat is NOT currently open
      // (ChatPanel handles its own message display)
      if (selectedPeer.value?.id !== senderId) {
        if (!unreadCounts.value[senderId]) {
          unreadCounts.value[senderId] = 0;
        }
        unreadCounts.value[senderId]++;
      }
      lastSeenTimes.value[senderId] = Date.now();
      // Also update peer lastSeen for the discovered peer
      const peer = peers.value.find(p => p.id === senderId);
      if (peer) {
        peer.lastSeen = Date.now();
      }
    }
  }));
});

onUnmounted(() => {
  window.removeEventListener('lan:reload-unread', loadUnreadCounts);
  cleanupIpcListeners.forEach(fn => fn());
  cleanupIpcListeners = [];
});

// 加载我的资料
async function loadMyProfile() {
  try {
    const info = await getTauriAPI().getUserInfo('');
    myUserId.value = info.id;
    myDisplayName.value = info.name || info.id;
    myAvatar.value = info.avatar || '😀';
    editNickName.value = '';
    editAvatar.value = info.avatar || '😀';
  } catch (e) {
    console.warn('Failed to load profile:', e);
  }
}

// 保存资料
async function saveProfile() {
  try {
    await getTauriAPI().setNickName(editNickName.value);
    await getTauriAPI().setAvatar(editAvatar.value);
    myAvatar.value = editAvatar.value;
    myDisplayName.value = editNickName.value || myUserId.value;
    showProfileEditor.value = false;
  } catch (e: any) {
    console.error('Failed to save profile:', e);
  }
}

function selectPeer(peer: LanPeer) {
  selectedPeer.value = peer;
  // Clear unread count when selecting a peer
  unreadCounts.value[peer.id] = 0;
  emit('select-peer', peer);
}

function openChat(peer: LanPeer) {
  selectedPeer.value = peer;
  unreadCounts.value[peer.id] = 0;
  emit('open-chat', peer);
}

function openAssign() {
  if (selectedPeer.value) {
    emit('open-assign', selectedPeer.value);
  }
}

async function chooseReceivePath() {
  try {
    const result = await getTauriAPI().showOpenDialogForDirs?.() as { filePaths?: string[] } | undefined
    if (result?.filePaths?.[0]) {
      await getTauriAPI().setReceivePath(result.filePaths[0])
      receivePath.value = result.filePaths[0]
    }
  } catch (e) {
    console.warn('Failed to choose receive path:', e)
  }
}
</script>

<style scoped>
.lan-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 4px;
}

/* ========== 我的资料卡片 ========== */
.profile-card {
  position: relative;
  border-radius: 16px;
  overflow: hidden;
  cursor: pointer;
  transition: all 0.25s ease;
  border: 1px solid oklch(var(--bc) / 0.1));
}
.profile-card:hover {
  transform: translateY(-1px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
  border-color: oklch(var(--p));
}

.profile-bg {
  position: absolute;
  inset: 0;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  opacity: 0.15;
}

.profile-content {
  position: relative;
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px;
}

.profile-avatar {
  position: relative;
  width: 48px;
  height: 48px;
  border-radius: 16px;
  background: linear-gradient(135deg, #667eea, #764ba2);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
  flex-shrink: 0;
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
}

.my-status-dot {
  position: absolute;
  bottom: -2px;
  right: -2px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  border: 3px solid oklch(var(--b1));
  transition: all 0.3s ease;
}
.my-status-dot.online { background: #22c55e; box-shadow: 0 0 6px rgba(34, 197, 94, 0.5); }
.my-status-dot.busy { background: #f59e0b; box-shadow: 0 0 6px rgba(245, 158, 11, 0.5); }
.my-status-dot.away { background: #6b7280; }

.profile-info {
  flex: 1;
  min-width: 0;
}

.profile-name {
  font-size: 15px;
  font-weight: 600;
  color: oklch(var(--bc));
  display: flex;
  align-items: center;
  gap: 4px;
}

.edit-icon {
  opacity: 0.3;
  transition: opacity 0.2s;
}
.profile-card:hover .edit-icon {
  opacity: 0.8;
}

.profile-id {
  font-size: 12px;
  color: oklch(var(--bc) / 0.6);
  opacity: 0.7;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-top: 2px;
}

/* 状态 Pill */
.status-pills {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.pill {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 500;
  color: oklch(var(--bc) / 0.6);
  background: rgba(255, 255, 255, 0.04);
  cursor: pointer;
  transition: all 0.2s ease;
  user-select: none;
}
.pill:hover {
  background: rgba(255, 255, 255, 0.08);
}
.pill.active {
  background: rgba(255, 255, 255, 0.12);
  color: oklch(var(--bc));
}

.pill-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: currentColor;
  opacity: 0.6;
}
.pill.active .pill-dot { opacity: 1; }
.pill-dot.online { background: #22c55e; }
.pill-dot.busy { background: #f59e0b; }
.pill-dot.away { background: #6b7280; }

/* ========== 资料编辑面板 ========== */
.editor-panel {
  background: oklch(var(--b1));
  border-radius: 14px;
  border: 1px solid oklch(var(--bc) / 0.1));
  overflow: hidden;
}

.editor-header {
  padding: 14px 16px;
  font-size: 14px;
  font-weight: 600;
  color: oklch(var(--bc));
  border-bottom: 1px solid oklch(var(--bc) / 0.1));
}

.editor-body {
  padding: 16px;
}

.editor-section {
  margin-bottom: 16px;
}
.editor-section:last-of-type {
  margin-bottom: 0;
}

.editor-section label {
  display: block;
  font-size: 12px;
  font-weight: 500;
  color: oklch(var(--bc) / 0.6);
  margin-bottom: 8px;
}

.emoji-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.emoji-opt {
  width: 38px;
  height: 38px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.15s ease;
  background: rgba(255, 255, 255, 0.03);
}
.emoji-opt:hover {
  background: rgba(255, 255, 255, 0.08);
  transform: scale(1.1);
}
.emoji-opt.selected {
  background: rgba(102, 126, 234, 0.2);
  box-shadow: inset 0 0 0 2px #667eea;
}

.editor-input {
  width: 100%;
  padding: 10px 14px;
  border-radius: 10px;
  border: 1px solid oklch(var(--bc) / 0.1));
  background: rgba(255, 255, 255, 0.04);
  color: oklch(var(--bc));
  font-size: 13px;
  outline: none;
  transition: border-color 0.2s;
}
.editor-input:focus {
  border-color: oklch(var(--p));
}

.editor-footer {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  margin-top: 16px;
  padding-top: 14px;
  border-top: 1px solid oklch(var(--bc) / 0.1));
}

/* ========== 网络权限提示 ========== */
.permission-banner {
  margin: 8px 0;
  border-radius: 10px;
  padding: 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  animation: fadeIn 0.3s ease;
}
.permission-banner.error {
  background: rgba(255, 59, 48, 0.12);
  border: 1px solid rgba(255, 59, 48, 0.3);
}
.permission-banner.warning {
  background: rgba(255, 149, 0, 0.12);
  border: 1px solid rgba(255, 149, 0, 0.3);
}
.banner-content {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  flex: 1;
  min-width: 0;
}
.banner-icon {
  font-size: 20px;
  flex-shrink: 0;
}
.banner-text {
  min-width: 0;
}
.banner-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}
.banner-detail {
  font-size: 11px;
  color: var(--text-secondary);
  margin-top: 2px;
  line-height: 1.4;
}
.banner-actions {
  flex-shrink: 0;
}
.btn-retry {
  padding: 6px 12px;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-primary);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}
.btn-retry:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.2);
  border-color: rgba(255, 255, 255, 0.3);
}
.btn-retry:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* ========== 头部 ========== */
.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 4px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.header-icon {
  font-size: 16px;
}

.header-title {
  font-size: 14px;
  font-weight: 600;
  color: oklch(var(--bc));
}

.online-badge {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 2px 10px;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.05);
  font-size: 12px;
  font-weight: 600;
  color: oklch(var(--bc) / 0.6);
  transition: all 0.3s ease;
}
.online-badge.hasPeers {
  background: rgba(34, 197, 94, 0.12);
  color: #22c55e;
}

.badge-pulse {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #22c55e;
  animation: pulse 2s ease-in-out infinite;
}
@keyframes pulse {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.5; transform: scale(0.8); }
}

.scan-btn {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  border: none;
  background: rgba(255, 255, 255, 0.04);
  color: oklch(var(--bc) / 0.6);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}
.scan-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: oklch(var(--bc));
}
.scan-btn.scanning {
  animation: spin 1s linear infinite;
  color: oklch(var(--p));
}
@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* ========== 用户列表 ========== */
.user-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.user-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;
  background: transparent;
}
.user-card:hover {
  background: rgba(255, 255, 255, 0.04);
}
.user-card.selected {
  background: rgba(102, 126, 234, 0.1);
  border: 1px solid rgba(102, 126, 234, 0.3);
  padding: 11px;
}
.user-card:not(.selected) {
  border: 1px solid transparent;
}

.unread-indicator {
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 60%;
  border-radius: 0 3px 3px 0;
  background: linear-gradient(180deg, #667eea, #764ba2);
}

.user-avatar-wrap {
  position: relative;
  flex-shrink: 0;
}

.user-avatar {
  width: 44px;
  height: 44px;
  border-radius: 14px;
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.3), rgba(118, 75, 162, 0.3));
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 22px;
}

.user-status-dot {
  position: absolute;
  bottom: -1px;
  right: -1px;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: 2.5px solid oklch(var(--b1));
  transition: all 0.3s ease;
}
.user-status-dot.online { background: #22c55e; box-shadow: 0 0 5px rgba(34, 197, 94, 0.5); }
.user-status-dot.busy { background: #f59e0b; box-shadow: 0 0 5px rgba(245, 158, 11, 0.5); }
.user-status-dot.away { background: #6b7280; }
.user-status-dot.offline { background: #4b5563; }

.user-info {
  flex: 1;
  min-width: 0;
}

.user-name-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.user-name {
  font-size: 14px;
  font-weight: 500;
  color: oklch(var(--bc));
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.msg-badge {
  min-width: 20px;
  height: 20px;
  padding: 0 6px;
  border-radius: 10px;
  background: linear-gradient(135deg, #ef4444, #dc2626);
  color: white;
  font-size: 10px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 8px rgba(239, 68, 68, 0.4);
}

.user-meta {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-top: 3px;
  font-size: 12px;
  color: oklch(var(--bc) / 0.6);
}

.user-status-text {
  font-weight: 500;
}
.user-status-text.online { color: #22c55e; }
.user-status-text.busy { color: #f59e0b; }
.user-status-text.away { color: #6b7280; }
.user-status-text.offline { color: #4b5563; }

.user-sep {
  opacity: 0.4;
}

.user-address {
  font-family: var(--font-mono, ui-monospace, monospace);
  font-size: 11px;
  opacity: 0.7;
}

.user-version {
  font-family: var(--font-mono, ui-monospace, monospace);
  font-size: 10px;
  padding: 1px 4px;
  border-radius: 3px;
  background: rgba(74, 222, 128, 0.15);
  color: #4ade80;
}
.user-version.version-incompatible {
  background: rgba(251, 191, 36, 0.15);
  color: #fbbf24;
}

.user-action {
  flex-shrink: 0;
  color: oklch(var(--bc) / 0.6);
  opacity: 0.3;
  transition: all 0.2s ease;
}
.user-card:hover .user-action {
  opacity: 0.7;
}

/* ========== 空状态 ========== */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 32px 16px;
  text-align: center;
}

.radar-container {
  position: relative;
  width: 100px;
  height: 100px;
  margin-bottom: 20px;
}

.radar {
  position: relative;
  width: 100%;
  height: 100%;
  border-radius: 50%;
  border: 1px solid rgba(102, 126, 234, 0.2);
  overflow: hidden;
}
.radar::before,
.radar::after {
  content: '';
  position: absolute;
  inset: 25%;
  border-radius: 50%;
  border: 1px solid rgba(102, 126, 234, 0.15);
}
.radar::after {
  inset: 40%;
}

.radar-sweep {
  position: absolute;
  inset: 0;
  background: conic-gradient(from 0deg, transparent 0deg, rgba(102, 126, 234, 0.15) 60deg, transparent 60deg);
  animation: radarSpin 3s linear infinite;
}
@keyframes radarSpin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.radar-dot {
  position: absolute;
  top: 30%;
  right: 25%;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #667eea;
  animation: radarDotBlink 2s ease-in-out infinite;
}
@keyframes radarDotBlink {
  0%, 100% { opacity: 0.3; transform: scale(0.8); }
  50% { opacity: 1; transform: scale(1.2); }
}

.empty-title {
  margin: 0 0 6px 0;
  font-size: 14px;
  font-weight: 500;
  color: oklch(var(--bc));
}

.empty-hint {
  margin: 0 0 16px 0;
  font-size: 12px;
  color: oklch(var(--bc) / 0.6);
  opacity: 0.7;
}

.network-details {
  width: 100%;
  max-width: 280px;
}
.network-details summary {
  font-size: 12px;
  color: oklch(var(--bc) / 0.6);
  cursor: pointer;
  opacity: 0.6;
  transition: opacity 0.2s;
  padding: 4px 0;
}
.network-details summary:hover {
  opacity: 1;
}

.network-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 10px;
  padding: 12px 16px;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 10px;
  font-size: 12px;
}

.info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.info-label {
  color: oklch(var(--bc) / 0.6);
  opacity: 0.7;
}

.info-value {
  color: oklch(var(--bc));
  font-family: var(--font-mono, ui-monospace, monospace);
  font-size: 11px;
}

/* ========== 按钮通用 ========== */
.btn-sm {
  padding: 7px 16px;
  font-size: 12px;
  border-radius: 8px;
  border: none;
  cursor: pointer;
  font-weight: 600;
  transition: all 0.15s;
}
.btn-primary {
  background: linear-gradient(135deg, #667eea, #764ba2);
  color: white;
}
.btn-primary:hover { opacity: 0.9; }
.btn-ghost {
  background: transparent;
  color: oklch(var(--bc) / 0.6);
  border: 1px solid oklch(var(--bc) / 0.1));
}
.btn-ghost:hover {
  background: rgba(255, 255, 255, 0.04);
}

/* ========== 动画 ========== */
.slide-enter-active { transition: all 0.25s ease; }
.slide-leave-active { transition: all 0.2s ease; }
.slide-enter-from { opacity: 0; transform: translateY(-10px); }
.slide-leave-to { opacity: 0; transform: translateY(-10px); }

.fade-enter-active { transition: all 0.25s ease; }
.fade-leave-active { transition: all 0.2s ease; }
.fade-enter-from { opacity: 0; transform: translateY(10px); }
.fade-leave-to { opacity: 0; transform: translateY(10px); }

/* ========== 文件接收路径 ========== */
.receive-path-section {
  padding: 12px 14px;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 12px;
  border: 1px solid oklch(var(--bc) / 0.1));
}

.path-label {
  font-size: 12px;
  font-weight: 600;
  color: oklch(var(--bc) / 0.6);
  margin-bottom: 8px;
}

.path-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.path-value {
  flex: 1;
  font-size: 12px;
  color: oklch(var(--bc));
  font-family: var(--font-mono, ui-monospace, monospace);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  opacity: 0.8;
}

.path-btn {
  padding: 5px 12px;
  border-radius: 8px;
  border: 1px solid oklch(var(--bc) / 0.1));
  background: rgba(255, 255, 255, 0.06);
  color: oklch(var(--bc));
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
  flex-shrink: 0;
}
.path-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  border-color: oklch(var(--p));
}
</style>
