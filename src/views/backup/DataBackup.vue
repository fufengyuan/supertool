<template>
  <div class="p-6 space-y-4">
    <h3 class="text-lg font-semibold"><SvgIcon name="download" size="14" />  {{ $t('backup.title') }}</h3>

    <div class="grid grid-cols-[repeat(auto-fit,minmax(280px,1fr))] gap-4">
      <!-- 导出 -->
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-5 flex flex-col gap-3">
        <div class="flex items-center gap-2 font-semibold text-sm text-base-content">
          <SvgIcon name="upload" size="14" />
          <span>数据导出</span>
        </div>

        <div class="text-sm text-base-content/60">
          导出所有模块数据为压缩包，包含：待办、项目、笔记、服务器、CI/CD、MFA、周报等 22 个表。
        </div>

        <button @click="exportFullBackup" class="btn btn-primary" :disabled="isExporting">
          <template v-if="isExporting">导出中...</template>
          <template v-else><SvgIcon name="save" size="14" /> 完整备份 (.stbackup)</template>
        </button>
      </div>

      <!-- 导入 -->
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-5 flex flex-col gap-3">
        <div class="flex items-center gap-2 font-semibold text-sm text-base-content">
          <span class="text-lg"><SvgIcon name="download" size="14" /> </span>
          <span>数据导入</span>
        </div>

        <div class="text-sm text-base-content/60">
          支持从完整备份 (.stbackup) 恢复数据。
        </div>

        <div class="flex flex-col gap-1">
          <label class="text-xs text-base-content/60">导入模式</label>
          <select v-model="importMode" class="select select-bordered select-sm">
            <option value="merge">合并（跳过重复数据）</option>
            <option value="replace">覆盖（清空现有数据后导入）</option>
          </select>
        </div>

        <button
          @click="importFullBackup"
          class="btn btn-success self-start"
          :disabled="isImporting"
        >
          <template v-if="isImporting">导入中...</template>
          <template v-else><SvgIcon name="folder" size="14" /> 导入备份</template>
        </button>
      </div>

      <!-- 自动备份 -->
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-5 flex flex-col gap-3 col-span-full">
        <div class="flex items-center gap-2 font-semibold text-sm text-base-content">
          <span class="text-lg"><SvgIcon name="clock" size="14" /> </span>
          <span>自动备份</span>
        </div>

        <div class="flex flex-col gap-4">
          <div class="flex items-center gap-3 flex-wrap">
            <label class="flex items-center justify-between w-full cursor-pointer">
              <span class="font-medium text-base-content">启用自动备份</span>
              <input
                type="checkbox"
                class="toggle toggle-primary toggle-sm"
                v-model="autoBackup.enabled"
                @change="saveAutoBackupSettings"
              />
            </label>
          </div>

          <template v-if="autoBackup.enabled">
            <div class="flex items-center gap-3 flex-wrap">
              <label class="min-w-20 text-xs text-base-content/60">备份频率</label>
              <select v-model="autoBackup.frequency" @change="saveAutoBackupSettings" class="select select-bordered select-sm">
                <option value="daily">每天</option>
                <option value="weekly">每周</option>
              </select>
            </div>

            <div class="flex items-center gap-3 flex-wrap">
              <label class="min-w-20 text-xs text-base-content/60">备份时间</label>
              <input
                type="time"
                v-model="autoBackup.time"
                @change="saveAutoBackupSettings"
                class="input input-bordered input-sm w-[120px]"
              />
            </div>

            <div class="flex items-center gap-3 flex-wrap">
              <label class="min-w-20 text-xs text-base-content/60">备份路径</label>
              <div class="flex gap-2 flex-1">
                <input
                  type="text"
                  v-model="autoBackup.path"
                  placeholder="默认为应用数据目录"
                  class="input input-bordered input-sm flex-1 min-w-[150px]"
                />
                <button @click="selectBackupPath" class="btn btn-ghost btn-sm">选择</button>
              </div>
            </div>

            <div
              v-if="lastBackupStatus"
              class="px-3 py-1.5 rounded-md text-xs border"
              :class="{
                'bg-success/10 text-success border-success/30': lastBackupStatus.type === 'success',
                'bg-error/10 text-error border-error/30': lastBackupStatus.type === 'error',
                'bg-info/10 text-info border-info/30': lastBackupStatus.type === 'info',
                'bg-warning/10 text-warning border-warning/30': lastBackupStatus.type === 'warning',
              }"
            >
              {{ lastBackupStatus.message }}
            </div>
          </template>
        </div>
      </div>
    </div>

    <!-- 消息 -->
    <div
      v-if="message"
      class="px-3 py-1.5 rounded-md text-xs border"
      :class="{
        'bg-success/10 text-success border-success/30': messageType === 'success',
        'bg-error/10 text-error border-error/30': messageType === 'error',
        'bg-info/10 text-info border-info/30': messageType === 'info',
        'bg-warning/10 text-warning border-warning/30': messageType === 'warning',
      }"
    >
      {{ message }}
    </div>

    <!-- 数据目录 -->
    <div class="bg-base-200 border border-base-content/10 rounded-xl p-5 flex flex-col gap-3">
      <div class="flex items-center gap-2 font-semibold text-sm text-base-content">
        <span class="text-lg"><SvgIcon name="folder" size="14" /> </span>
        <span>数据目录</span>
        <span v-if="dataDir.isCustom" class="badge badge-info badge-sm">自定义</span>
        <span v-else class="badge badge-ghost badge-sm">默认</span>
      </div>

      <div class="flex flex-col gap-3">
        <div class="flex items-center gap-3 flex-wrap">
          <label class="min-w-20 text-xs text-base-content/60">当前路径</label>
          <div class="flex gap-2 flex-1">
            <input
              type="text"
              v-model="dataDir.editPath"
              :placeholder="dataDir.defaultPath"
              class="input input-bordered input-sm flex-1 min-w-[150px]"
            />
            <button @click="selectDataDir" class="btn btn-ghost btn-sm">选择</button>
          </div>
        </div>

        <div class="flex items-center gap-3 flex-wrap">
          <label class="min-w-20 text-xs text-base-content/60">默认路径</label>
          <span class="font-mono text-xs text-base-content/70">{{ dataDir.defaultPath }}</span>
        </div>

        <div class="flex items-center justify-end gap-3 mt-2">
          <button @click="saveDataDir" class="btn btn-primary" :disabled="dataDir.saving">
            {{ dataDir.saving ? '保存中...' : '💾 保存' }}
          </button>
          <button v-if="dataDir.isCustom" @click="resetDataDir" class="btn btn-ghost">
            恢复默认
          </button>
        </div>

        <div v-if="dataDir.needRestart" class="px-3 py-1.5 rounded-md text-xs border bg-warning/10 text-warning border-warning/30">
          ⚠️ 数据目录已更新，需要重启应用才能生效
        </div>
      </div>
    </div>

    <!-- Git 同步 -->
    <div class="bg-base-200 border border-base-content/10 rounded-xl p-5 flex flex-col gap-3">
      <div class="flex items-center gap-2 font-semibold text-sm text-base-content">
        <span class="text-lg"><SvgIcon name="refresh" size="14" /> </span>
        <span>Git 数据同步</span>
        <span
          v-if="gitSyncStatus.enabled"
          class="badge badge-sm"
          :class="gitSyncStatus.status === 'ok' ? 'badge-success' : 'badge-error'"
        >
          {{ gitSyncStatus.status === 'ok' ? '✓ 正常' : '✗ 错误' }}
        </span>
      </div>

      <div class="flex flex-col gap-4">
        <div class="flex items-center gap-3 flex-wrap">
          <label class="flex items-center justify-between w-full cursor-pointer">
            <span class="font-medium text-base-content">启用 Git 同步</span>
            <input
              type="checkbox"
              class="toggle toggle-primary toggle-sm"
              v-model="gitSyncConfig.enabled"
              @change="saveGitSyncConfig"
            />
          </label>
        </div>

        <template v-if="gitSyncConfig.enabled">
          <div class="flex items-center gap-3 flex-wrap">
            <label class="min-w-20 text-xs text-base-content/60">远程仓库地址</label>
            <div class="flex gap-2 flex-1">
              <input
                type="text"
                v-model="gitSyncConfig.remoteUrl"
                placeholder="git@github.com:user/repo.git"
                class="input input-bordered input-sm flex-1 min-w-[150px]"
              />
              <button @click="initGitSync" class="btn btn-ghost btn-sm" :disabled="isGitSyncing">
                {{ isGitSyncing ? '初始化中...' : '初始化' }}
              </button>
            </div>
          </div>

          <div class="flex items-center gap-3 flex-wrap">
            <label class="min-w-20 text-xs text-base-content/60">同步分支</label>
            <input
              type="text"
              v-model="gitSyncConfig.branch"
              class="input input-bordered input-sm w-24"
            />
          </div>

          <div class="flex items-center gap-3 flex-wrap">
            <label class="min-w-20 text-xs text-base-content/60">同步间隔（分钟）</label>
            <input
              type="number"
              v-model.number="gitSyncConfig.interval"
              min="1"
              max="60"
              class="input input-bordered input-sm w-20"
              @change="saveGitSyncConfig"
            />
          </div>

          <div class="flex items-center gap-3 flex-wrap">
            <label class="min-w-20 text-xs text-base-content/60">SSH 私钥路径（可选）</label>
            <input
              type="text"
              v-model="gitSyncConfig.sshKey"
              placeholder="~/.ssh/id_rsa"
              class="input input-bordered input-sm flex-1 min-w-[150px]"
              @change="saveGitSyncConfig"
            />
          </div>

          <div class="flex items-center justify-end gap-3 mt-2">
            <button @click="pullGit" class="btn btn-ghost" :disabled="isGitSyncing">
              {{ isGitSyncing ? '同步中...' : '⬇️ 拉取' }}
            </button>
            <button @click="pushGit" class="btn btn-primary" :disabled="isGitSyncing">
              {{ isGitSyncing ? '同步中...' : '⬆️ 推送' }}
            </button>
          </div>

          <div v-if="gitSyncStatus.lastSync" class="px-3 py-1.5 rounded-md text-xs border bg-info/10 text-info border-info/30">
            上次同步: {{ formatGitSyncTime(gitSyncStatus.lastSync) }}
          </div>
          <div v-if="gitSyncStatus.error" class="px-3 py-1.5 rounded-md text-xs border bg-error/10 text-error border-error/30">
            {{ gitSyncStatus.error }}
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
defineOptions({ name: 'DataBackup' })
import * as logger from '../../services/logger'
import { getTauriAPI } from '../../utils/tauri-api'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { useErrorHandler } from '../../composables/useErrorHandler';

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
    const result = await getTauriAPI().exportData({}) as any;

    if (result.success) {
      const warnings = result.warnings && result.warnings.length
        ? `（${result.warnings.length} 张表导出失败：${result.warnings.slice(0, 2).join('、')}${result.warnings.length > 2 ? '...' : ''}）`
        : '';
      const type = warnings ? 'warning' : 'success';
      message.value = `完整备份已导出: ${result.path}（${result.tableCount || 1} 个表，${result.totalItems || 0} 条记录）${warnings}`;
      messageType.value = type;
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
    console.log("[importFullBackup] called, importMode =", importMode.value)
    // 后端会自动弹出文件选择框
    const result = await getTauriAPI().importJson({
      import_mode: importMode.value,
    }) as any;

    if (result.success) {
      message.value = `成功导入 ${result.importedCount} 条记录，跳过 ${result.skippedCount || 0} 条重复数据`;
      messageType.value = 'success';
    } else if (result.importedCount > 0) {
      // 部分成功：有导入记录但也存在错误
      const errSummary = result.errors && result.errors.length
        ? `（${result.errors.length} 条错误：${result.errors.slice(0, 2).join('；')}${result.errors.length > 2 ? '...' : ''}）`
        : '';
      message.value = `部分导入成功：导入 ${result.importedCount} 条，跳过 ${result.skippedCount || 0} 条${errSummary}`;
      messageType.value = 'warning';
    } else {
      const errDetail = result.errors && result.errors.length
        ? result.errors.slice(0, 3).join('；')
        : (result.error || result.message || '未知错误');
      message.value = `导入失败: ${errDetail}`;
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
  try {
    const enabled = await getTauriAPI().getSetting('auto_backup_enabled');
    const frequency = await getTauriAPI().getSetting('auto_backup_frequency');
    const time = await getTauriAPI().getSetting('auto_backup_time');
    const path = await getTauriAPI().getSetting('auto_backup_path');

    autoBackup.value.enabled = enabled === 'true';
    if (frequency) {autoBackup.value.frequency = frequency;}
    if (time) {autoBackup.value.time = time;}
    if (path) {autoBackup.value.path = path;}
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
  if (mins < 1) {return '刚刚';}
  if (mins < 60) {return `${mins} 分钟前`;}
  const hours = Math.floor(mins / 60);
  if (hours < 24) {return `${hours} 小时前`;}
  return d.toLocaleDateString('zh-CN');
}
</script>
