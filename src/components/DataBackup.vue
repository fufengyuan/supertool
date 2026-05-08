<template>
  <div class="data-backup-panel">
    <h3>💾 {{ $t('backup.title') }}</h3>

    <div class="backup-sections">
      <div class="backup-card">
        <div class="section-title">
          <span class="icon">📤</span>
          <span>数据导出</span>
        </div>

        <div class="backup-desc">
          导出所有模块数据为压缩包，包含：待办、项目、笔记、服务器、CI/CD、MFA、周报等 22 个表。
        </div>

        <button @click="exportFullBackup" class="btn btn-primary" :disabled="isExporting">
          {{ isExporting ? '导出中...' : '💾 完整备份 (.stbackup)' }}
        </button>
      </div>

      <div class="backup-card">
        <div class="section-title">
          <span class="icon">📥</span>
          <span>数据导入</span>
        </div>

        <div class="backup-desc">
          支持从完整备份 (.stbackup) 恢复数据。
        </div>

        <div class="form-field">
          <label>导入模式</label>
          <select v-model="importMode" class="form-select">
            <option value="merge">合并（跳过重复数据）</option>
            <option value="replace">覆盖（清空现有数据后导入）</option>
          </select>
        </div>

        <button
          @click="importFullBackup"
          class="btn btn-success"
          :disabled="isImporting"
          style="align-self: flex-start"
        >
          {{ isImporting ? '导入中...' : '📂 导入备份' }}
        </button>
      </div>

      <!-- 自动备份设置 -->
      <div class="backup-card auto-backup-card">
        <div class="section-title">
          <span class="icon">⏰</span>
          <span>自动备份</span>
        </div>

        <div class="auto-backup-settings">
          <div class="setting-row">
            <label class="toggle-label">
              <span>启用自动备份</span>
              <label class="toggle-switch">
                <input type="checkbox" v-model="autoBackup.enabled" @change="saveAutoBackupSettings" />
                <span class="toggle-slider"></span>
              </label>
            </label>
          </div>

          <template v-if="autoBackup.enabled">
            <div class="setting-row">
              <label>备份频率</label>
              <select v-model="autoBackup.frequency" @change="saveAutoBackupSettings" class="form-select">
                <option value="daily">每天</option>
                <option value="weekly">每周</option>
              </select>
            </div>

            <div class="setting-row">
              <label>备份时间</label>
              <input 
                type="time" 
                v-model="autoBackup.time" 
                @change="saveAutoBackupSettings" 
                class="form-input time-input" 
              />
            </div>

            <div class="setting-row">
              <label>备份路径</label>
              <div class="path-input-group">
                <input 
                  type="text" 
                  v-model="autoBackup.path" 
                  placeholder="默认为应用数据目录"
                  class="form-input"
                />
                <button @click="selectBackupPath" class="btn btn-ghost btn-sm">选择</button>
              </div>
            </div>

            <div v-if="lastBackupStatus" class="backup-status" :class="lastBackupStatus.type">
              {{ lastBackupStatus.message }}
            </div>
          </template>
        </div>
      </div>
    </div>

    <div v-if="message" class="test-result" :class="messageType">
      {{ message }}
    </div>

    <!-- 数据目录设置 -->
    <div class="backup-card data-dir-card">
      <div class="section-title">
        <span class="icon">📁</span>
        <span>数据目录</span>
        <span v-if="dataDir.isCustom" class="status-badge custom">自定义</span>
        <span v-else class="status-badge default">默认</span>
      </div>

      <div class="data-dir-settings">
        <div class="setting-row">
          <label>当前路径</label>
          <div class="path-input-group">
            <input
              type="text"
              v-model="dataDir.editPath"
              :placeholder="dataDir.defaultPath"
              class="form-input"
            />
            <button @click="selectDataDir" class="btn btn-ghost btn-sm">选择</button>
          </div>
        </div>

        <div class="setting-row">
          <label>默认路径</label>
          <span class="path-display">{{ dataDir.defaultPath }}</span>
        </div>

        <div class="setting-row data-dir-actions">
          <button @click="saveDataDir" class="btn btn-primary" :disabled="dataDir.saving">
            {{ dataDir.saving ? '保存中...' : '💾 保存' }}
          </button>
          <button v-if="dataDir.isCustom" @click="resetDataDir" class="btn btn-ghost">
            恢复默认
          </button>
        </div>

        <div v-if="dataDir.needRestart" class="backup-status warning">
          ⚠️ 数据目录已更新，需要重启应用才能生效
        </div>
      </div>
    </div>

    <!-- Git 同步设置 -->
    <div class="backup-card git-sync-card">
      <div class="section-title">
        <span class="icon">🔄</span>
        <span>Git 数据同步</span>
        <span v-if="gitSyncStatus.enabled" class="status-badge" :class="gitSyncStatus.status">
          {{ gitSyncStatus.status === 'ok' ? '✓ 正常' : '✗ 错误' }}
        </span>
      </div>

      <div class="git-sync-settings">
        <div class="setting-row">
          <label class="toggle-label">
            <span>启用 Git 同步</span>
            <label class="toggle-switch">
              <input type="checkbox" v-model="gitSyncConfig.enabled" @change="saveGitSyncConfig" />
              <span class="toggle-slider"></span>
            </label>
          </label>
        </div>

        <template v-if="gitSyncConfig.enabled">
          <div class="setting-row">
            <label>远程仓库地址</label>
            <div class="path-input-group">
              <input type="text" v-model="gitSyncConfig.remoteUrl" placeholder="git@github.com:user/repo.git" class="form-input" />
              <button @click="initGitSync" class="btn btn-ghost btn-sm" :disabled="isGitSyncing">{{ isGitSyncing ? '初始化中...' : '初始化' }}</button>
            </div>
          </div>

          <div class="setting-row">
            <label>同步分支</label>
            <input type="text" v-model="gitSyncConfig.branch" class="form-input" style="width: 100px" />
          </div>

          <div class="setting-row">
            <label>同步间隔（分钟）</label>
            <input type="number" v-model.number="gitSyncConfig.interval" min="1" max="60" class="form-input" style="width: 80px" @change="saveGitSyncConfig" />
          </div>

          <div class="setting-row">
            <label>SSH 私钥路径（可选）</label>
            <input type="text" v-model="gitSyncConfig.sshKey" placeholder="~/.ssh/id_rsa" class="form-input" @change="saveGitSyncConfig" />
          </div>

          <div class="setting-row git-sync-actions">
            <button @click="pullGit" class="btn btn-ghost" :disabled="isGitSyncing">
              {{ isGitSyncing ? '同步中...' : '⬇️ 拉取' }}
            </button>
            <button @click="pushGit" class="btn btn-primary" :disabled="isGitSyncing">
              {{ isGitSyncing ? '同步中...' : '⬆️ 推送' }}
            </button>
          </div>

          <div v-if="gitSyncStatus.lastSync" class="backup-status info">
            上次同步: {{ formatGitSyncTime(gitSyncStatus.lastSync) }}
          </div>
          <div v-if="gitSyncStatus.error" class="backup-status error">
            {{ gitSyncStatus.error }}
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import * as logger from '../services/logger'
import { getTauriAPI } from '../utils/tauri-api'
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { useErrorHandler } from '../composables/useErrorHandler';

const { handleError } = useErrorHandler();

const importMode = ref('merge');
const isExporting = ref(false);
const isImporting = ref(false);
const message = ref('');
const messageType = ref('info');

// Git sync state
const isGitSyncing = ref(false);
const gitSyncConfig = ref({
  enabled: false,
  remoteUrl: '',
  branch: 'main',
  interval: 5,
  sshKey: ''
});
const gitSyncStatus = ref<{
  enabled: boolean;
  remoteUrl: string | null;
  branch: string;
  interval: number;
  lastSync: string | null;
  status: string;
  error: string | null;
  sshKey: string;
}>({
  enabled: false,
  remoteUrl: null,
  branch: 'main',
  interval: 5,
  lastSync: null,
  status: '',
  error: null,
  sshKey: ''
});

// 自动备份设置
const autoBackup = ref({
  enabled: false,
  frequency: 'daily',
  time: '02:00',
  path: ''
});

// 数据目录设置
const dataDir = ref({
  path: '',
  editPath: '',
  defaultPath: '',
  isCustom: false,
  saving: false,
  needRestart: false
});

const lastBackupStatus = ref(null);

// 导出JSON
const exportFullBackup = async () => {
  isExporting.value = true;
  message.value = '';

  try {
    console.log("[exportFullBackup] called")
    const result = await getTauriAPI().exportData({}) as any;

    if (result.success) {
      message.value = `✅ 完整备份已导出: ${result.path}（${result.tableCount || 1} 个表，${result.totalItems || 0} 条记录）`;
      messageType.value = 'success';
    } else {
      message.value = `导出失败: ${result.error || result.message || '未知错误'}`;
      messageType.value = 'error';
    }
  } catch (error) {
    handleError(error, { context: '导出JSON', showToast: true });
    message.value = `导出失败: ${error.message}`;
    messageType.value = 'error';
  } finally {
    isExporting.value = false;
  }
};

// 导入备份
const importFullBackup = async () => {
  isImporting.value = true;
  message.value = '';

  try {
    console.log("[importFullBackup] called")
    // 后端会自动弹出文件选择框
    const result = await getTauriAPI().importJson({
      import_mode: importMode.value,
    }) as any;

    if (result.success) {
      message.value = `成功导入 ${result.importedCount} 条记录，跳过 ${result.skippedCount || 0} 条重复数据`;
      messageType.value = 'success';
    } else {
      message.value = `导入失败: ${result.error || result.message || '未知错误'}`;
      messageType.value = 'error';
    }
  } catch (error) {
    handleError(error, { context: '导入JSON', showToast: true });
    message.value = `导入失败: ${error.message}`;
    messageType.value = 'error';
  } finally {
    isImporting.value = false;
  }
};

// 保存自动备份设置
const saveAutoBackupSettings = async () => {
  try {
    console.log("[saveAutoBackupSettings] called")
    await getTauriAPI().setAutoBackup({
      enabled: autoBackup.value.enabled,
      frequency: autoBackup.value.frequency,
      time: autoBackup.value.time,
      path: autoBackup.value.path
    });
  } catch (error) {
    handleError(error, { context: '保存自动备份设置', showToast: true });
  }
};

// 选择备份路径
const selectBackupPath = async () => {
  try {
    console.log("[selectBackupPath] called")
    const result = await getTauriAPI().showOpenDialogForDirs?.() as { filePaths?: string[] } | undefined
    if (result?.filePaths?.[0]) {
      autoBackup.value.path = result.filePaths[0]
      await saveAutoBackupSettings()
    }
  } catch (error) {
    console.error('选择备份路径失败:', error)
  }
}

// 加载自动备份设置
onMounted(async () => {
    console.log("[components/DataBackup.vue] mounted")
  try {
    const enabled = await getTauriAPI().getSetting('auto_backup_enabled');
    const frequency = await getTauriAPI().getSetting('auto_backup_frequency');
    const time = await getTauriAPI().getSetting('auto_backup_time');
    const path = await getTauriAPI().getSetting('auto_backup_path');

    autoBackup.value.enabled = enabled === 'true';
    if (frequency) autoBackup.value.frequency = frequency;
    if (time) autoBackup.value.time = time;
    if (path) autoBackup.value.path = path;
  } catch (error) {
    console.error('Failed to load auto backup settings:', error);
  }

  // 加载数据目录设置
  try {
    const result = await getTauriAPI().getDataDir();
    if (result?.success) {
      dataDir.value.path = result.path;
      dataDir.value.editPath = result.path;
      dataDir.value.defaultPath = result.defaultPath;
      dataDir.value.isCustom = result.isCustom;
    }
  } catch (error) {
    console.error('Failed to load data dir settings:', error);
  }

  // Load Git sync status
  try {
    const status = await getTauriAPI().gitSyncStatus();
    gitSyncStatus.value = status;
    gitSyncConfig.value.enabled = status.enabled;
    gitSyncConfig.value.remoteUrl = status.remoteUrl || '';
    gitSyncConfig.value.branch = status.branch;
    gitSyncConfig.value.interval = status.interval;
    gitSyncConfig.value.sshKey = status.sshKey || '';
  } catch (e) {
    console.error('Failed to load Git sync status:', e);
  }

  // Listen for Git sync status updates
  getTauriAPI().onGitSyncStatusUpdated?.((data) => {
    gitSyncStatus.value.status = data.status;
    gitSyncStatus.value.error = data.error || null;
  });

  // 监听自动备份完成事件
  if (getTauriAPI().onAutoBackupCompleted) {
    /* TODO(tauri-events): autoBackupUnsubscribe = getTauriAPI().onAutoBackupCompleted((data: any) => {
      if (data.success) {
        lastBackupStatus.value = { type: 'success', message: `自动备份成功: ${data.path}` };
      } else {
        lastBackupStatus.value = { type: 'error', message: `自动备份失败: ${data.error}` };
      }
    });
    */}
});

// Cleanup listeners on unmount
let autoBackupUnsubscribe: (() => void) | undefined;

onBeforeUnmount(() => {
  autoBackupUnsubscribe?.();
});

// 选择数据目录
async function selectDataDir() {
  try {
    const result = await getTauriAPI().showOpenDialogForDirs?.() as { filePaths?: string[] } | undefined
    if (result?.filePaths?.[0]) {
      dataDir.value.editPath = result.filePaths[0]
    }
  } catch (error) {
    console.error('选择数据目录失败:', error)
  }
}

// 保存数据目录
async function saveDataDir() {
  dataDir.value.saving = true
  try {
    const result = await getTauriAPI().setDataDir(dataDir.value.editPath)
    if (result?.success) {
      dataDir.value.needRestart = true
      dataDir.value.isCustom = !!dataDir.value.editPath
      dataDir.value.path = result.path || dataDir.value.editPath
      message.value = result.message || '数据目录已更新'
      messageType.value = 'success'
    } else {
      message.value = `保存失败: ${result?.error || '未知错误'}`
      messageType.value = 'error'
    }
  } catch (error: any) {
    message.value = `保存失败: ${error.message}`
    messageType.value = 'error'
  } finally {
    dataDir.value.saving = false
  }
}

// 恢复默认数据目录
async function resetDataDir() {
  dataDir.value.saving = true
  try {
    const result = await getTauriAPI().setDataDir('')
    if (result?.success) {
      dataDir.value.needRestart = true
      dataDir.value.isCustom = false
      dataDir.value.editPath = dataDir.value.defaultPath
      dataDir.value.path = dataDir.value.defaultPath
      message.value = result.message || '已恢复默认数据目录'
      messageType.value = 'success'
    }
  } catch (error: any) {
    message.value = `恢复失败: ${error.message}`
    messageType.value = 'error'
  } finally {
    dataDir.value.saving = false
  }
}

// Git sync functions
async function saveGitSyncConfig() {
  try {
    console.log("[saveGitSyncConfig] called")
    await getTauriAPI().gitSyncConfigure({
      enabled: String(gitSyncConfig.value.enabled),
      remote_url: gitSyncConfig.value.remoteUrl,
      branch: gitSyncConfig.value.branch,
      interval: String(gitSyncConfig.value.interval),
      ssh_key_path: gitSyncConfig.value.sshKey
    });
    const status = await getTauriAPI().gitSyncStatus();
    gitSyncStatus.value = status;
  } catch (e: any) {
    handleError(e, { context: '保存Git同步配置', showToast: true });
  }
}

async function initGitSync() {
  if (!gitSyncConfig.value.remoteUrl) {
    message.value = '请先输入远程仓库地址';
    messageType.value = 'error';
    return;
  }
  isGitSyncing.value = true;
  try {
    await getTauriAPI().gitSyncConfigure({
      remote_url: gitSyncConfig.value.remoteUrl,
      branch: gitSyncConfig.value.branch,
      ssh_key_path: gitSyncConfig.value.sshKey
    });
    const result = await getTauriAPI().gitSyncInit();
    if (result.success) {
      message.value = 'Git 同步初始化成功';
      messageType.value = 'success';
      const status = await getTauriAPI().gitSyncStatus();
      gitSyncStatus.value = status;
    } else {
      message.value = `初始化失败: ${result.message}`;
      messageType.value = 'error';
    }
  } catch (e: any) {
    handleError(e, { context: '初始化Git同步', showToast: true });
  } finally {
    isGitSyncing.value = false;
  }
}

async function pullGit() {
  isGitSyncing.value = true;
  try {
    console.log("[pullGit] called")
    const result = await getTauriAPI().gitSyncPull();
    if (result.success) {
      message.value = `拉取成功，导入 ${result.importedCount} 条，跳过 ${result.skippedCount} 条`;
      messageType.value = 'success';
    } else {
      message.value = `拉取失败: ${result.message}`;
      messageType.value = 'error';
    }
    const status = await getTauriAPI().gitSyncStatus();
    gitSyncStatus.value = status;
  } catch (e: any) {
    handleError(e, { context: 'Git拉取', showToast: true });
  } finally {
    isGitSyncing.value = false;
  }
}

async function pushGit() {
  isGitSyncing.value = true;
  try {
    console.log("[pushGit] called")
    const result = await getTauriAPI().gitSyncPush();
    if (result.success) {
      message.value = '推送成功';
      messageType.value = 'success';
    } else {
      message.value = `推送失败: ${result.message}`;
      messageType.value = 'error';
    }
    const status = await getTauriAPI().gitSyncStatus();
    gitSyncStatus.value = status;
  } catch (e: any) {
    handleError(e, { context: 'Git推送', showToast: true });
  } finally {
    isGitSyncing.value = false;
  }
}

function formatGitSyncTime(iso: string): string {
  const d = new Date(iso);
  const now = new Date();
  const diff = now.getTime() - d.getTime();
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return '刚刚';
  if (mins < 60) return `${mins} 分钟前`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours} 小时前`;
  return d.toLocaleDateString('zh-CN');
}
</script>

<style scoped>
.data-backup-panel {
  padding: 24px;
}

.backup-sections {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 16px;
  margin-top: 16px;
}

.backup-card {
  padding: 20px;
  background: var(--color-base-200);
  border-radius: 12px;
  border: 1.5px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 600;
  color: var(--color-base-content);
}

.section-title .icon {
  font-size: 18px;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none !important;
}

/* 自动备份设置 */
.auto-backup-card {
  grid-column: 1 / -1;
}

.auto-backup-settings {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.setting-row {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.setting-row label {
  font-size: 13px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  min-width: 80px;
}

.toggle-label {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  cursor: pointer;
}

.toggle-label span:first-child {
  color: var(--color-base-content);
  font-weight: 500;
}

/* 开关样式 */
.toggle-switch {
  position: relative;
  display: inline-block;
  width: 44px;
  height: 24px;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: color-mix(in oklab, var(--color-base-content) 10%, transparent);
  transition: 0.3s;
  border-radius: 24px;
}

.toggle-slider:before {
  position: absolute;
  content: "";
  height: 18px;
  width: 18px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  transition: 0.3s;
  border-radius: 50%;
}

.toggle-switch input:checked + .toggle-slider {
  background-color: var(--color-primary);
}

.toggle-switch input:checked + .toggle-slider:before {
  transform: translateX(20px);
}

.form-select {
  padding: 6px 10px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 6px;
  background: var(--color-base-100);
  color: var(--color-base-content);
  font-size: 13px;
}

.form-input {
  padding: 6px 10px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 6px;
  background: var(--color-base-100);
  color: var(--color-base-content);
  font-size: 13px;
  flex: 1;
  min-width: 150px;
}

.time-input {
  width: 120px;
  flex: none;
}

.path-input-group {
  display: flex;
  gap: 8px;
  flex: 1;
}

.btn-sm {
  padding: 6px 12px;
  font-size: 12px;
  white-space: nowrap;
}

.backup-status {
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 12px;
  margin-top: 4px;
}

.backup-status.success {
  background: rgba(34, 197, 94, 0.1);
  color: var(--color-success);
  border: 1px solid rgba(34, 197, 94, 0.3);
}

.backup-status.error {
  background: rgba(239, 68, 68, 0.1);
  color: var(--color-error);
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.backup-status.info {
  background: rgba(59, 130, 246, 0.1);
  color: #3b82f6;
  border: 1px solid rgba(59, 130, 246, 0.3);
}

/* Git Sync */
.git-sync-card {
  grid-column: 1 / -1;
}

.git-sync-settings {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.status-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 10px;
  margin-left: 8px;
}

.status-badge.ok {
  background: rgba(34, 197, 94, 0.15);
  color: #22c55e;
}

.status-badge.error {
  background: rgba(239, 68, 68, 0.15);
  color: #ef4444;
}

.status-badge.custom {
  background: rgba(59, 130, 246, 0.15);
  color: #3b82f6;
}

.status-badge.default {
  background: rgba(107, 114, 128, 0.15);
  color: #6b7280;
}

.backup-status.warning {
  background: color-mix(in oklab, var(--color-warning) 10%, transparent);
  color: var(--color-warning);
  border: 1px solid var(--color-warning);
}

.path-display {
  font-family: monospace;
  font-size: 13px;
  color: var(--color-base-content);
  opacity: 0.7;
}

.data-dir-settings {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.data-dir-actions {
  justify-content: flex-end;
  gap: 12px;
  margin-top: 8px;
}

.git-sync-actions {
  justify-content: flex-end;
  gap: 12px;
  margin-top: 8px;
}
</style>
