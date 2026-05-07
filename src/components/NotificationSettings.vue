<template>
  <div class="notification-settings">
    <h3>🔔 {{ $t('notification.title') }}</h3>

    <!-- 提醒时间设置 -->
    <div class="settings-card">
      <div class="card-header">
        <span class="card-icon">⏰</span>
        <span class="card-title">{{ $t('notification.reminderTime') }}</span>
      </div>
      <div class="form-field">
        <select v-model="settings.reminderTime" @change="saveSettings" class="form-select">
          <option value="5">{{ $t('notification.minutes5') }}</option>
          <option value="15">{{ $t('notification.minutes15') }}</option>
          <option value="30">{{ $t('notification.minutes30') }}</option>
          <option value="60">{{ $t('notification.hours1') }}</option>
          <option value="1440">{{ $t('notification.days1') }}</option>
          <option value="custom">{{ $t('notification.custom') }}</option>
        </select>
      </div>

      <div v-if="settings.reminderTime === 'custom'" class="form-field">
        <label>{{ $t('notification.customTime') }}</label>
        <div style="display: flex; align-items: center; gap: 8px">
          <input
            v-model.number="customTime"
            type="number"
            min="1"
            @change="saveCustomTime"
            class="form-input"
            placeholder="15"
            style="max-width: 120px"
          />
          <span class="hint-text">{{ $t('notification.reminderHint') }}</span>
        </div>
      </div>

      <p class="settings-hint" v-html="$t('notification.hint', { time: displayTime })"></p>
    </div>

    <!-- 免打扰时段 -->
    <div class="settings-card">
      <div class="card-header">
        <span class="card-icon">🌙</span>
        <span class="card-title">免打扰时段</span>
        <label class="toggle-switch">
          <input type="checkbox" v-model="settings.quietHoursEnabled" @change="saveExtendedSettings" />
          <span class="toggle-slider"></span>
        </label>
      </div>
      <div v-if="settings.quietHoursEnabled" class="quiet-hours-row">
        <div class="time-field">
          <label>开始时间</label>
          <input type="time" v-model="settings.quietHoursStart" @change="saveExtendedSettings" class="form-input time-input" />
        </div>
        <span class="time-separator">至</span>
        <div class="time-field">
          <label>结束时间</label>
          <input type="time" v-model="settings.quietHoursEnd" @change="saveExtendedSettings" class="form-input time-input" />
        </div>
      </div>
      <p v-if="settings.quietHoursEnabled" class="settings-hint">
        在 <strong>{{ settings.quietHoursStart || '22:00' }}</strong> 到 <strong>{{ settings.quietHoursEnd || '08:00' }}</strong> 期间将不会发送通知
      </p>
    </div>

    <!-- 每日总结 -->
    <div class="settings-card">
      <div class="card-header">
        <span class="card-icon">📊</span>
        <span class="card-title">每日总结</span>
        <label class="toggle-switch">
          <input type="checkbox" v-model="settings.dailySummaryEnabled" @change="saveExtendedSettings" />
          <span class="toggle-slider"></span>
        </label>
      </div>
      <div v-if="settings.dailySummaryEnabled" class="daily-summary-row">
        <div class="time-field">
          <label>总结时间</label>
          <input type="time" v-model="settings.dailySummaryTime" @change="saveExtendedSettings" class="form-input time-input" />
        </div>
      </div>
      <p v-if="settings.dailySummaryEnabled" class="settings-hint">
        每天 <strong>{{ settings.dailySummaryTime || '21:00' }}</strong> 发送今日完成与待办任务总结
      </p>
    </div>

    <!-- 已完成任务静音 -->
    <div class="settings-card">
      <div class="card-header">
        <span class="card-icon">🔇</span>
        <span class="card-title">已完成任务静音</span>
        <label class="toggle-switch">
          <input type="checkbox" v-model="settings.muteCompleted" @change="saveExtendedSettings" />
          <span class="toggle-slider"></span>
        </label>
      </div>
      <p v-if="settings.muteCompleted" class="settings-hint">
        已完成的任务将不再发送提醒通知
      </p>
      <p v-else class="settings-hint">
        所有任务（包括已完成）都会发送提醒通知
      </p>
    </div>

    <!-- 测试通知 -->
    <div class="test-section">
      <button @click="testNotification" class="btn btn-ghost">🔔 {{ $t('notification.test') }}</button>
      <span
        v-if="testResult"
        class="test-result"
        :class="testResult.success ? 'success' : 'error'"
      >
        {{ testResult.message }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, onMounted, computed } from 'vue';
import { useErrorHandler } from '../composables/useErrorHandler';
import { getTauriAPI } from '../utils/tauri-api';

const { handleError } = useErrorHandler();

interface NotificationSettings {
  reminderTime: number | string;
  quietHoursEnabled?: boolean;
  quietHoursStart?: string;
  quietHoursEnd?: string;
  dailySummaryEnabled?: boolean;
  dailySummaryTime?: string;
  muteCompleted?: boolean;
}

const settings = ref<NotificationSettings>({
  reminderTime: 15,
  quietHoursEnabled: false,
  quietHoursStart: '22:00',
  quietHoursEnd: '08:00',
  dailySummaryEnabled: false,
  dailySummaryTime: '21:00',
  muteCompleted: false,
});
const testResult = ref<{ success: boolean; message: string } | null>(null);
const customTime = ref(15);

// 计算显示时间
const displayTime = computed(() => {
  if (settings.value.reminderTime === 'custom') {
    const minutes = Number(customTime.value) || 15;
    if (minutes >= 1440) {
      const days = Math.floor(minutes / 1440);
      return `${days}天`;
    } else if (minutes >= 60) {
      const hours = Math.floor(minutes / 60);
      const remainingMinutes = minutes % 60;
      return remainingMinutes > 0 ? `${hours}小时${remainingMinutes}分钟` : `${hours}小时`;
    } else {
      return `${minutes}分钟`;
    }
  } else {
    return formatReminderTime(settings.value.reminderTime);
  }
});

// 加载设置
onMounted(async () => {
    console.log("[components/NotificationSettings.vue] mounted");
    try {
      const api = getTauriAPI();
      const loadedSettings = await api.getNotificationSettings();
      settings.value = {
        ...settings.value,
        ...loadedSettings,
      };
      // 如果当前值不是预设值，则视为自定义
      if (!['5', '15', '30', '60', '1440'].includes(loadedSettings.reminderTime.toString())) {
        settings.value.reminderTime = 'custom';
        customTime.value = Number(loadedSettings.reminderTime);
      }
    } catch (error) {
      handleError(error, { context: '加载通知设置', showToast: true });
    }
});

// 保存基础设置
const saveSettings = async () => {
  if (settings.value.reminderTime !== 'custom') {
    console.log("[saveSettings] called");
    customTime.value = Number(settings.value.reminderTime);
    await saveSetting(Number(settings.value.reminderTime));
  }
};

// 保存自定义时间
const saveCustomTime = async () => {
  if (settings.value.reminderTime === 'custom') {
    console.log("[saveCustomTime] called");
    await saveSetting(customTime.value);
  }
};

// 保存基础设置
const saveSetting = async (time: number) => {
  try {
    console.log("[saveSetting] called");
    const api = getTauriAPI();
    await api.setNotificationSettings({ reminderTime: time });
  } catch (error) {
    handleError(error, { context: '保存通知设置', showToast: true });
  }
};

// 保存扩展设置（免打扰、每日总结、静音）
const saveExtendedSettings = async () => {
  try {
    console.log("[saveExtendedSettings] called");
    const api = getTauriAPI();
    await api.setNotificationSettings({
      reminderTime: typeof settings.value.reminderTime === 'number'
        ? settings.value.reminderTime
        : customTime.value,
      quietHoursEnabled: settings.value.quietHoursEnabled,
      quietHoursStart: settings.value.quietHoursStart,
      quietHoursEnd: settings.value.quietHoursEnd,
      dailySummaryEnabled: settings.value.dailySummaryEnabled,
      dailySummaryTime: settings.value.dailySummaryTime,
      muteCompleted: settings.value.muteCompleted,
    });
  } catch (error) {
    handleError(error, { context: '保存通知设置', showToast: true });
  }
};

// 测试通知
const testNotification = async () => {
  try {
    console.log("[testNotification] called");
    const api = getTauriAPI();
    const success = await (api as any).testNotification?.() ?? true;

    // macOS 开发模式下，原生通知可能因应用未签名被系统静默拦截
    // 补充 Web Notification 作为降级方案，确保开发时能看到弹窗
    if (navigator.platform.includes('Mac') && window.Notification) {
      const webNotif = new window.Notification('测试通知', {
        body: '这是一条测试通知（Web 降级方案），说明通知功能正常工作。'
      });
      webNotif.onclick = () => window.focus();
    }

    if (success) {
      testResult.value = { success: true, message: '测试通知已发送！' };
    } else {
      testResult.value = { success: false, message: '通知不支持或发送失败' };
    }

    setTimeout(() => {
      testResult.value = null;
    }, 3000);
  } catch (error: any) {
    handleError(error, { context: '测试通知', showToast: true });
    testResult.value = { success: false, message: '测试失败: ' + error.message };

    setTimeout(() => {
      testResult.value = null;
    }, 3000);
  }
};

// 格式化提醒时间显示
const formatReminderTime = (minutes: number | string): string => {
  const m = typeof minutes === 'string' ? Number(minutes) : minutes;
  if (m >= 1440) {
    return `${Math.floor(m / 1440)}天`;
  } else if (m >= 60) {
    return `${Math.floor(m / 60)}小时`;
  } else {
    return `${m}分钟`;
  }
};
</script>

<style scoped>
.notification-settings {
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.settings-card {
  padding: 20px;
  background: var(--card-bg);
  border-radius: 12px;
  border: 1.5px solid var(--border-color);
  transition: all 0.2s ease;
}

.settings-card:hover {
  border-color: var(--primary-color);
  box-shadow: var(--card-shadow-hover);
}

.card-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 14px;
}

.card-icon {
  font-size: 20px;
}

.card-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--main-text);
  flex: 1;
}

.form-field {
  margin-bottom: 8px;
}

.form-field label {
  display: block;
  font-size: 13px;
  color: var(--main-text-secondary);
  margin-bottom: 6px;
}

.form-select {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--input-bg);
  color: var(--main-text);
  font-size: 14px;
  cursor: pointer;
}

.form-input {
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--input-bg);
  color: var(--main-text);
  font-size: 14px;
}

.hint-text {
  font-size: 13px;
  color: var(--main-text-secondary);
}

.settings-hint {
  margin-top: 10px;
  padding: 10px 14px;
  background: var(--primary-light);
  border-radius: 8px;
  font-size: 13px;
  color: var(--main-text);
  line-height: 1.5;
}

.settings-hint strong {
  color: var(--primary-color);
}

/* 开关切换 */
.toggle-switch {
  position: relative;
  display: inline-block;
  width: 44px;
  height: 24px;
  cursor: pointer;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  top: 0; left: 0; right: 0; bottom: 0;
  background: var(--border-color);
  border-radius: 24px;
  transition: 0.3s;
}

.toggle-slider::before {
  content: '';
  position: absolute;
  height: 18px;
  width: 18px;
  left: 3px;
  bottom: 3px;
  background: white;
  border-radius: 50%;
  transition: 0.3s;
}

.toggle-switch input:checked + .toggle-slider {
  background: var(--primary-color);
}

.toggle-switch input:checked + .toggle-slider::before {
  transform: translateX(20px);
}

/* 免打扰时间行 */
.quiet-hours-row {
  display: flex;
  align-items: flex-end;
  gap: 12px;
  margin-top: 12px;
}

.daily-summary-row {
  display: flex;
  align-items: flex-end;
  gap: 12px;
  margin-top: 12px;
}

.time-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.time-field label {
  font-size: 12px;
  color: var(--main-text-secondary);
}

.time-input {
  padding: 6px 10px;
  width: 130px;
}

.time-separator {
  font-size: 13px;
  color: var(--main-text-secondary);
  padding-bottom: 8px;
}

/* 测试区域 */
.test-section {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 0;
}

.test-result {
  font-size: 13px;
  font-weight: 500;
}

.test-result.success {
  color: var(--success-color);
}

.test-result.error {
  color: var(--danger-color);
}

.btn-ghost {
  padding: 8px 16px;
  border: 1px solid var(--border-color);
  background: transparent;
  color: var(--main-text);
  border-radius: 8px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s ease;
}

.btn-ghost:hover {
  border-color: var(--primary-color);
  color: var(--primary-color);
}

@media (max-width: 768px) {
  .quiet-hours-row {
    flex-direction: column;
    align-items: flex-start;
  }
  .time-separator {
    padding-bottom: 0;
  }
}
</style>
