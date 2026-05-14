<template>
  <div class="flex flex-col gap-3 p-1">
    <!-- 我的资料卡片 -->
    <div class="group relative rounded-2xl overflow-hidden cursor-pointer transition-all duration-200 border border-base-content/10 hover:-translate-y-0.5 hover:shadow-[0_8px_24px_rgba(0,0,0,0.12)] hover:border-primary" @click="showProfileEditor = !showProfileEditor">
      <div class="absolute inset-0 bg-gradient-to-br from-[#667eea] to-[#764ba2] opacity-15"></div>
      <div class="relative flex items-center gap-3.5 p-4">
        <div class="relative shrink-0">
          <div class="size-12 rounded-2xl bg-gradient-to-br from-[#667eea] to-[#764ba2] flex items-center justify-center text-2xl shadow-[0_4px_12px_rgba(102,126,234,0.3)] shrink-0">
            <img v-if="myAvatarPath && !myAvatarIsEmoji" :src="convertFileSrc(myAvatarPath)" class="size-full rounded-2xl object-cover" />
            <span v-else>{{ myAvatar }}</span>
          </div>
          <span class="absolute -bottom-0.5 -right-0.5 size-3.5 rounded-full border-[3px] border-base-100 transition-all duration-300"
                :class="{
                  'bg-green-500 shadow-[0_0_6px_rgba(34,197,94,0.5)]': currentStatus === 'online',
                  'bg-amber-500 shadow-[0_0_6px_rgba(245,158,11,0.5)]': currentStatus === 'busy',
                  'bg-gray-500': currentStatus === 'away',
                }"></span>
        </div>
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-1 text-sm font-semibold text-base-content">
            {{ myDisplayName }}
            <SvgIcon name="pencil" size="12" class="opacity-30 transition-opacity duration-200 group-hover:opacity-80" />
          </div>
          <div class="text-xs text-base-content/60 opacity-70 truncate mt-0.5">{{ myUserId }}</div>
        </div>
        <div class="flex gap-1 shrink-0" @click.stop>
          <span class="flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-medium cursor-pointer transition-all select-none"
                :class="currentStatus === 'online' ? 'bg-white/12 text-base-content' : 'bg-white/4 text-base-content/60 hover:bg-white/8'"
                @click="setStatus('online')" title="在线">
            <span class="size-1.5 rounded-full bg-green-500" :class="currentStatus === 'online' ? 'opacity-100' : 'opacity-60'"></span>在线
          </span>
          <span class="flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-medium cursor-pointer transition-all select-none"
                :class="currentStatus === 'busy' ? 'bg-white/12 text-base-content' : 'bg-white/4 text-base-content/60 hover:bg-white/8'"
                @click="setStatus('busy')" title="忙碌">
            <span class="size-1.5 rounded-full bg-amber-500" :class="currentStatus === 'busy' ? 'opacity-100' : 'opacity-60'"></span>忙碌
          </span>
          <span class="flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-medium cursor-pointer transition-all select-none"
                :class="currentStatus === 'away' ? 'bg-white/12 text-base-content' : 'bg-white/4 text-base-content/60 hover:bg-white/8'"
                @click="setStatus('away')" title="离开">
            <span class="size-1.5 rounded-full bg-gray-500" :class="currentStatus === 'away' ? 'opacity-100' : 'opacity-60'"></span>离开
          </span>
        </div>
      </div>
    </div>

    <!-- 资料编辑面板 -->
    <Transition name="slide">
      <div v-if="showProfileEditor" class="bg-base-100 rounded-xl border border-base-content/10 overflow-hidden">
        <div class="px-4 py-3.5 text-sm font-semibold text-base-content border-b border-base-content/10">编辑资料</div>
        <div class="p-4">
          <div class="mb-4 last:mb-0">
            <label class="block text-xs font-medium text-base-content/60 mb-2">头像</label>
            <div class="flex flex-wrap gap-1.5 mb-2">
              <span v-for="emoji in avatarOptions" :key="emoji"
                    class="size-[38px] flex items-center justify-center text-xl rounded-xl cursor-pointer transition-all bg-white/3 hover:bg-white/8 hover:scale-110"
                    :class="{ 'bg-[#667eea]/20 shadow-[inset_0_0_0_2px_#667eea]': editAvatar === emoji && !editAvatarPath }"
                    @click="editAvatar = emoji; editAvatarPath = ''">{{ emoji }}</span>
            </div>
            <!-- 上传图片按钮 -->
            <button class="btn btn-outline btn-sm w-full gap-2" @click="uploadAvatar" :disabled="uploadingAvatar">
              <SvgIcon name="upload" size="16" />
              {{ uploadingAvatar ? '上传中...' : '上传图片' }}
            </button>
            <p v-if="editAvatarPath" class="text-xs text-base-content/50 mt-2">
              已选择图片: {{ editAvatarPath.split('/').pop() }}
            </p>
          </div>
          <div class="mb-4 last:mb-0">
            <label class="block text-xs font-medium text-base-content/60 mb-2">昵称</label>
            <input v-model="editNickName" class="input input-bordered w-full text-sm" placeholder="留空使用系统用户名" @keydown.enter="saveProfile" />
          </div>
          <div class="flex gap-2 justify-end mt-4 pt-3.5 border-t border-base-content/10">
            <button class="btn btn-ghost btn-sm" @click="showProfileEditor = false">取消</button>
            <button class="btn btn-primary btn-sm" @click="saveProfile">保存</button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 网络权限提示（macOS Local Network Privacy） -->
    <div v-if="permissionWarning" class="my-2 rounded-xl p-3 flex items-center justify-between gap-2.5 animate-[fadeIn_0.3s_ease]"
         :class="permissionWarning.type === 'error' ? 'bg-red-500/12 border border-red-500/30' : 'bg-amber-500/12 border border-amber-500/30'">
      <div class="flex items-start gap-2.5 flex-1 min-w-0">
        <span class="text-xl shrink-0"><template v-if="permissionWarning.type === 'error'"><SvgIcon name="ban" size="14" class="inline-block align-text-bottom" /></template><template v-else><SvgIcon name="alertTriangle" size="14" class="inline-block align-text-bottom" /></template></span>
        <div class="min-w-0">
          <div class="text-sm font-semibold text-base-content">{{ permissionWarning.title }}</div>
          <div class="text-xs text-base-content/60 mt-0.5 leading-relaxed">{{ permissionWarning.message }}</div>
        </div>
      </div>
      <div class="shrink-0">
        <button class="btn btn-outline btn-xs" @click="recheckPermission" :disabled="checkingPermission">
          {{ checkingPermission ? '检测中...' : '重新检测' }}
        </button>
      </div>
    </div>

    <!-- 头部 -->
    <div class="flex items-center justify-between px-1 py-2">
      <div class="flex items-center gap-2">
        <span class="text-base"><SvgIcon name="globe" size="14" class="inline-block align-text-bottom" /></span>
        <span class="text-sm font-semibold text-base-content">局域网用户</span>
        <span class="badge badge-sm" :class="peers.length > 0 ? 'badge-success' : 'badge-ghost'">
          <span v-if="peers.length > 0" class="size-1.5 rounded-full bg-green-500 animate-pulse mr-1"></span>
          {{ peers.length }}
        </span>
      </div>
      <button class="btn btn-ghost btn-square btn-sm" @click="refreshDiscovery" :class="{ 'animate-spin text-primary': scanning }" title="重新扫描">
        <SvgIcon name="refresh" size="16" />
      </button>
    </div>

    <!-- 用户列表 -->
    <div class="flex flex-col gap-1" v-if="peers.length > 0">
      <div
        class="group flex items-center gap-3 p-3 rounded-xl cursor-pointer transition-all duration-200 relative border"
        v-for="peer in sortedPeers"
        :key="peer.id"
        @click="openChat(peer)"
        :class="{
          'bg-[#667eea]/10 border-[#667eea]/30 p-[11px]': selectedPeer?.id === peer.id,
          'border-transparent hover:bg-white/4': selectedPeer?.id !== peer.id,
          'hasUnread': unreadCounts[peer.id] > 0
        }"
      >
        <div v-if="unreadCounts[peer.id] > 0" class="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 h-3/5 rounded-r-sm bg-gradient-to-b from-[#667eea] to-[#764ba2]"></div>
        <div class="relative shrink-0">
          <div class="size-11 rounded-xl bg-gradient-to-br from-[#667eea]/30 to-[#764ba2]/30 flex items-center justify-center text-2xl">
            <span>{{ peer.avatar || '😀' }}</span>
          </div>
          <span class="absolute -bottom-px -right-px size-3 rounded-full border-[2.5px] border-base-100 transition-all duration-300"
                :class="{
                  'bg-green-500 shadow-[0_0_5px_rgba(34,197,94,0.5)]': getStatusClass(peer) === 'online',
                  'bg-amber-500 shadow-[0_0_5px_rgba(245,158,11,0.5)]': getStatusClass(peer) === 'busy',
                  'bg-gray-500': getStatusClass(peer) === 'away',
                  'bg-gray-600': getStatusClass(peer) === 'offline',
                }"></span>
        </div>
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2">
            <span class="text-sm font-medium text-base-content truncate">{{ peer.name }}</span>
            <span v-if="unreadCounts[peer.id] > 0" class="badge badge-error badge-sm gap-1 min-w-5 h-5 shadow-[0_2px_8px_rgba(239,68,68,0.4)]">
              {{ unreadCounts[peer.id] > 99 ? '99+' : unreadCounts[peer.id] }}
            </span>
          </div>
          <div class="flex items-center gap-1 mt-0.5 text-xs text-base-content/60">
            <span class="font-medium" :class="{
              'text-green-500': getStatusClass(peer) === 'online',
              'text-amber-500': getStatusClass(peer) === 'busy',
              'text-gray-500': getStatusClass(peer) === 'away',
              'text-gray-600': getStatusClass(peer) === 'offline',
            }">{{ getStatusText(peer) }}</span>
            <span class="opacity-40">·</span>
            <span class="font-mono text-[11px] opacity-70">{{ peer.address }}</span>
            <span v-if="peer.version"
                  class="font-mono text-[10px] px-1 py-[1px] rounded-sm"
                  :class="peer.version && peer.version.split('.')[0] !== '2' ? 'bg-amber-400/15 text-amber-400' : 'bg-green-400/15 text-green-400'"
                  :title="peer.version === '2.0' ? '✅ 兼容' : '⚠️ 版本过低，无法发送消息'">v{{ peer.version }}</span>
          </div>
        </div>
        <div class="shrink-0 text-base-content/60 opacity-30 transition-all duration-200 group-hover:opacity-70">
          <SvgIcon name="chevronRight" size="16" />
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div class="flex flex-col items-center px-4 py-8 text-center" v-else>
      <div class="relative size-[100px] mb-5">
        <div class="radar relative w-full h-full rounded-full border border-[#667eea]/20 overflow-hidden">
          <div class="absolute inset-[25%] rounded-full border border-[#667eea]/15"></div>
          <div class="absolute inset-[40%] rounded-full border border-[#667eea]/15"></div>
          <div class="absolute inset-0 animate-[radarSpin_3s_linear_infinite]" style="background: conic-gradient(from 0deg, transparent 0deg, rgba(102,126,234,0.15) 60deg, transparent 60deg);"></div>
          <div class="absolute top-[30%] right-[25%] size-1.5 rounded-full bg-[#667eea] animate-[radarDotBlink_2s_ease-in-out_infinite]"></div>
        </div>
      </div>
      <p class="m-0 mb-1.5 text-sm font-medium text-base-content">正在搜索局域网用户</p>
      <p class="m-0 mb-4 text-xs text-base-content/60 opacity-70">确保其他设备在同一 WiFi 网络下</p>
      <details class="w-full max-w-[280px]">
        <summary class="text-xs text-base-content/60 cursor-pointer opacity-60 transition-opacity duration-200 hover:opacity-100 py-1">网络信息</summary>
        <div class="flex flex-col gap-2 mt-2.5 px-4 py-3 bg-white/3 rounded-xl text-xs" v-if="networkInfo">
          <div class="flex justify-between items-center">
            <span class="text-base-content/60 opacity-70">本机 IP</span>
            <span class="text-base-content font-mono text-[11px]">{{ networkInfo.address }}</span>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-base-content/60 opacity-70">端口</span>
            <span class="text-base-content font-mono text-[11px]">{{ networkInfo.ports }}</span>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-base-content/60 opacity-70">版本</span>
            <span class="text-base-content font-mono text-[11px]">v{{ networkInfo.version }}</span>
          </div>
        </div>
      </details>
    </div>

    <!-- 文件接收路径设置 -->
    <div class="px-3.5 py-3 bg-white/3 rounded-xl border border-base-content/10">
      <div class="text-xs font-semibold text-base-content/60 mb-2"><SvgIcon name="folder" size="14" class="inline-block align-text-bottom" /> 文件保存路径</div>
      <div class="flex items-center gap-2">
        <span class="flex-1 text-xs text-base-content font-mono truncate opacity-80" :title="receivePath">{{ receivePath || '默认' }}</span>
        <button class="btn btn-outline btn-sm" @click="chooseReceivePath">选择目录</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { getTauriAPI } from '@/utils/tauri-api'
import { open } from '@tauri-apps/plugin-dialog'
import { convertFileSrc } from '@tauri-apps/api/core'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { useToast } from '@/composables/useToast'

const toast = useToast()

interface LanPeer {
  id: string;
  name: string;
  avatar?: string;
  address: string;
  messagePort?: number;
  version?: string;
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
const myAvatarPath = ref<string>('');
const myAvatarIsEmoji = ref(true);
const currentStatus = ref<'online' | 'busy' | 'away'>('online');
const showProfileEditor = ref(false);
const editNickName = ref('');
const editAvatar = ref('😀');
const editAvatarPath = ref('');
const uploadingAvatar = ref(false);
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
    await getTauriAPI().lanSetStatus(status);
  } catch (e) {
    console.warn('Failed to change status:', e);
  }
}

/** 检测 macOS 局域网权限 */
async function checkNetworkPermission() {
  checkingPermission.value = true
  try {
    const result = await getTauriAPI().lanCheckNetworkPermission()
    if (!result.success) {
      const isTccBlocked = result.error?.includes('EHOSTUNREACH')
        || result.error?.includes('Local Network Privacy')
        || result.error?.includes('blocked')
        || result.error?.includes('Permission')
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
    await getTauriAPI().lanRefreshDiscovery();
    setTimeout(() => { scanning.value = false }, 3000);
  } catch (e) {
    scanning.value = false;
    console.warn('Failed to refresh discovery:', e);
  }
}

// 未读消息计数刷新（模块级，供 onMounted / onUnmounted 共用）
async function loadUnreadCounts() {
  try {
    const userInfo = await getTauriAPI().lanGetUserInfo();
    const counts = await getTauriAPI().lanGetAllUnreadCounts(userInfo.id);
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
    const statusInfo = await getTauriAPI().lanGetStatus();
    currentStatus.value = (statusInfo.status as 'online' | 'busy' | 'away') || 'online';
  } catch {}
  try {
    networkInfo.value = await getTauriAPI().lanGetNetworkInfo();
  } catch {}
  try {
    receivePath.value = await getTauriAPI().lanGetReceivePath();
  } catch {}
  peers.value = (await getTauriAPI().lanGetPeers()) as LanPeer[];
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
  cleanupIpcListeners.push(getTauriAPI().onLanPeerAvatarUpdated((data: any) => {
    // 收到其他用户的头像更新广播，更新本地 peer 列表中的头像
    const peer = peers.value.find(p => p.id === data.userId);
    if (peer) {
      peer.avatar = data.avatarRef || `file:${data.avatarPath}`;
    }
  }));
  cleanupIpcListeners.push(getTauriAPI().lanOnMessage((data: any) => {
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
    const info = await getTauriAPI().lanGetUserInfo();
    myUserId.value = info.id;
    myDisplayName.value = info.name || info.id;
    myAvatar.value = info.avatar || '😀';
    editNickName.value = '';
    editAvatar.value = info.avatar || '😀';
    // 解析头像路径
    if (info.avatar && info.avatar.startsWith('avatar:')) {
      const avatarInfo = await getTauriAPI().lanGetAvatarPath(info.avatar);
      myAvatarIsEmoji.value = avatarInfo.isEmoji;
      myAvatarPath.value = avatarInfo.path;
      editAvatarPath.value = avatarInfo.isEmoji ? '' : avatarInfo.path;
    } else {
      myAvatarIsEmoji.value = true;
      myAvatarPath.value = '';
      editAvatarPath.value = '';
    }
  } catch (e) {
    console.warn('Failed to load profile:', e);
  }
}

// 上传头像图片
async function uploadAvatar() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: '图片', extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp'] }],
      title: '选择头像图片',
    });
    if (!selected) return;
    const filePath = typeof selected === 'string' ? selected : selected.path;
    uploadingAvatar.value = true;
    const result = await getTauriAPI().lanUploadAvatar(filePath);
    editAvatar.value = result.path;
    editAvatarPath.value = result.fullPath;
    toast.success('头像上传成功');
  } catch (e: any) {
    console.error('Failed to upload avatar:', e);
    toast.error(`上传失败: ${e.message || e}`);
  } finally {
    uploadingAvatar.value = false;
  }
}

// 保存资料
async function saveProfile() {
  try {
    await getTauriAPI().lanSetNickName(editNickName.value);
    await getTauriAPI().lanSetAvatar(editAvatar.value);
    myAvatar.value = editAvatar.value;
    myAvatarPath.value = editAvatarPath.value;
    myAvatarIsEmoji.value = !editAvatarPath.value;
    myDisplayName.value = editNickName.value || myUserId.value;
    showProfileEditor.value = false;
    editAvatarPath.value = '';
  } catch (e: any) {
    console.error('Failed to save profile:', e);
    toast.error(`保存失败: ${e.message || e}`);
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
    const result = await getTauriAPI().lanShowOpenDialogForDirs() as { filePaths?: string[] } | undefined
    if (result?.filePaths?.[0]) {
      await getTauriAPI().lanSetReceivePath(result.filePaths[0])
      receivePath.value = result.filePaths[0]
    }
  } catch (e) {
    console.warn('Failed to choose receive path:', e)
  }
}
</script>

<style>
/* Vue Transition — slide */
.slide-enter-active {
  transition: all 0.25s ease;
}
.slide-leave-active {
  transition: all 0.2s ease;
}
.slide-enter-from,
.slide-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}

/* Radar animations */
@keyframes radarSpin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
@keyframes radarDotBlink {
  0%, 100% { opacity: 0.3; transform: scale(0.8); }
  50% { opacity: 1; transform: scale(1.2); }
}
</style>
