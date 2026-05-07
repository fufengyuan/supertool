<template>
  <div class="server-card" :class="statusClass">
    <div class="server-status-bar" :style="{ background: statusGradient }"></div>
    <div class="server-content">
      <div class="server-header">
        <div class="server-title-row">
          <div class="server-title-left">
            <span class="server-name">{{ server.name }}</span>
            <span v-if="server.requiresApproval" class="approval-badge" title="执行审核已开启">🔒</span>
          </div>
          <div class="status-badge" :class="connectionStatus">
            <span class="status-pulse" :class="connectionStatus"></span>
            {{ statusLabel }}
          </div>
        </div>
      </div>

      <div class="server-info">
        <div class="info-row">
          <svg class="info-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="2" y="3" width="20" height="14" rx="2" ry="2"/>
            <line x1="8" y1="21" x2="16" y2="21"/>
            <line x1="12" y1="17" x2="12" y2="21"/>
          </svg>
          <span class="info-text">{{ server.host }}<span class="info-port">:{{ server.port }}</span></span>
        </div>
        <div class="info-row">
          <svg class="info-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
            <circle cx="12" cy="7" r="4"/>
          </svg>
          <span class="info-text">{{ server.username }}</span>
        </div>
      </div>

      <div v-if="server.tags && server.tags.length > 0" class="server-tags">
        <span v-for="tag in server.tags" :key="tag" class="tag">{{ tag }}</span>
      </div>

      <div class="server-actions">
        <button @click="$emit('terminal', server)" class="action-btn terminal" title="终端">
          <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/>
          </svg>
        </button>
        <button @click="$emit('sftp', server)" class="action-btn sftp" title="SFTP">
          <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
        </button>
        <div class="action-separator"></div>
        <button @click="$emit('edit', server)" class="action-btn edit" title="编辑">
          <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
            <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
          </svg>
        </button>
        <button @click="$emit('delete', server.id)" class="action-btn delete" title="删除">
          <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3 6 5 6 21 6"/>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps({
  server: { type: Object, required: true },
  connectionStatus: { type: String, default: 'offline' },
});

defineEmits(['connect', 'terminal', 'sftp', 'edit', 'delete']);

const statusClass = computed(() => ({
  connected: props.connectionStatus === 'online',
  connecting: props.connectionStatus === 'connecting',
  'heartbeat-failed': props.connectionStatus === 'heartbeat_failed',
}));

const statusLabel = computed(() => {
  switch (props.connectionStatus) {
    case 'online': return '已连接';
    case 'connecting': return '连接中...';
    case 'heartbeat_failed': return '已断开';
    case 'offline': default: return '未连接';
  }
});

const statusGradient = computed(() => {
  switch (props.connectionStatus) {
    case 'online': return 'linear-gradient(180deg, #a6e3a1, #40c057)';
    case 'connecting': return 'linear-gradient(180deg, #f9a825, #f59e0b)';
    case 'heartbeat_failed': return 'linear-gradient(180deg, #f38ba8, #e04560)';
    default: return 'linear-gradient(180deg, var(--border-color), transparent)';
  }
});
</script>

<style scoped>
.server-card {
  position: relative;
  background: var(--card-bg);
  border-radius: 8px;
  border: 1.5px solid var(--border-color);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  overflow: hidden;
}

.server-card:hover {
  box-shadow: 0 3px 14px rgba(0, 0, 0, 0.06);
  transform: translateY(-1px);
  border-color: var(--primary-color);
}

.server-card.connected {
  border-color: rgba(166, 227, 161, 0.4);
}

.server-card.connecting {
  border-color: rgba(249, 168, 37, 0.4);
  animation: card-pulse 2s ease-in-out infinite;
}

.server-card.heartbeat-failed {
  border-color: rgba(243, 139, 168, 0.4);
}

@keyframes card-pulse {
  0%, 100% { border-color: rgba(249, 168, 37, 0.4); }
  50% { border-color: rgba(249, 168, 37, 0.15); }
}

.server-status-bar {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
}

.server-content {
  padding: 8px 10px;
}

.server-header {
  margin-bottom: 6px;
}

.server-title-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.server-name {
  font-weight: 600;
  font-size: 12px;
  color: var(--main-text);
  line-height: 1.3;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.approval-badge {
  font-size: 10px;
  margin-left: 4px;
  opacity: 0.8;
}

/* 连接状态徽章 */
.status-badge {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 9px;
  padding: 2px 7px;
  border-radius: 12px;
  font-weight: 500;
  white-space: nowrap;
}

.status-badge.online {
  background: rgba(166, 227, 161, 0.12);
  color: var(--success-color);
}

.status-badge.connecting {
  background: rgba(249, 168, 37, 0.12);
  color: var(--warning-color, #f9a825);
}

.status-badge.offline {
  background: var(--input-bg);
  color: var(--main-text-secondary);
}

.status-badge.heartbeat_failed {
  background: rgba(243, 139, 168, 0.12);
  color: var(--danger-color);
}

.status-pulse {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-pulse.online {
  background: var(--success-color);
  box-shadow: 0 0 6px var(--success-color);
}

.status-pulse.connecting {
  background: var(--warning-color, #f9a825);
  animation: dot-blink 0.8s ease-in-out infinite;
}

.status-pulse.offline {
  background: var(--main-text-secondary);
}

.status-pulse.heartbeat_failed {
  background: var(--danger-color);
  box-shadow: 0 0 6px var(--danger-color);
  animation: dot-blink-danger 1s ease-in-out infinite;
}

@keyframes dot-blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}

@keyframes dot-blink-danger {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}

.server-info {
  display: flex;
  flex-direction: column;
  gap: 3px;
  margin-bottom: 6px;
}

.info-row {
  display: flex;
  align-items: center;
  gap: 5px;
  color: var(--main-text-secondary);
  font-size: 11px;
}

.info-icon {
  opacity: 0.5;
  flex-shrink: 0;
  width: 12px;
  height: 12px;
}

.info-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.info-port {
  color: var(--main-text-tertiary, #666);
}

.server-tags {
  display: flex;
  gap: 3px;
  margin-bottom: 6px;
  flex-wrap: wrap;
}

.server-tags .tag {
  padding: 1px 5px;
  background: var(--input-bg);
  border-radius: 3px;
  font-size: 9px;
  color: var(--main-text-secondary);
  border: 1px solid var(--border-color);
}

.server-actions {
  display: flex;
  align-items: center;
  gap: 1px;
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  background: transparent;
  color: var(--main-text-secondary);
  transition: all 0.15s ease;
}

.action-btn:hover {
  background: var(--input-bg);
  color: var(--main-text);
}

.action-btn.terminal:hover {
  background: rgba(108, 99, 255, 0.12);
  color: var(--primary-color);
}

.action-btn.sftp:hover {
  background: rgba(108, 99, 255, 0.12);
  color: var(--primary-color);
}

.action-btn.delete:hover {
  background: rgba(243, 139, 168, 0.12);
  color: var(--danger-color);
}

.action-separator {
  width: 1px;
  height: 14px;
  background: var(--border-color);
  margin: 0 3px;
}
</style>
