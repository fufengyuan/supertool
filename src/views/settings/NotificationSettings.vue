<template>
  <div class="flex flex-col gap-4 text-base-content" :class="compact ? '' : 'p-6'">
    <h3 v-if="!compact"><SvgIcon name="bell" size="14" />  {{ $t('notification.title') }}</h3>

    <!-- 提醒时间设置 -->
    <div class="p-5 bg-base-100 rounded-xl border border-base-content/10 transition-all duration-200 hover:border-primary hover:shadow-lg">
      <div class="flex items-center gap-2.5 mb-3.5">
        <span class="text-xl"><SvgIcon name="clock" size="14" /> </span>
        <span class="text-sm font-semibold text-base-content flex-1">{{ $t('notification.reminderTime') }}</span>
      </div>
      <div class="mb-2">
        <select v-model="settings.reminderTime" @change="saveSettings" class="select select-bordered w-full">
          <option value="5">{{ $t('notification.minutes5') }}</option>
          <option value="15">{{ $t('notification.minutes15') }}</option>
          <option value="30">{{ $t('notification.minutes30') }}</option>
          <option value="60">{{ $t('notification.hours1') }}</option>
          <option value="1440">{{ $t('notification.days1') }}</option>
          <option value="custom">{{ $t('notification.custom') }}</option>
        </select>
      </div>

      <div v-if="settings.reminderTime === 'custom'" class="mb-2">
        <label class="block text-xs text-base-content/60 mb-1.5">{{ $t('notification.customTime') }}</label>
        <div class="flex items-center gap-2">
          <input
            v-model.number="customTime"
            type="number"
            min="1"
            class="input input-bordered max-w-[120px]"
            placeholder="15"
            @change="saveCustomTime"
          />
          <span class="text-xs text-base-content/60">{{ $t('notification.reminderHint') }}</span>
        </div>
      </div>

      <p class="mt-2.5 px-3.5 py-2.5 bg-primary/10 rounded-lg text-sm text-base-content leading-relaxed" v-html="$t('notification.hint', { time: displayTime })"></p>
    </div>

    <!-- 免打扰时段 -->
    <div class="p-5 bg-base-100 rounded-xl border border-base-content/10 transition-all duration-200 hover:border-primary hover:shadow-lg">
      <div class="flex items-center gap-2.5 mb-3.5">
        <span class="text-xl">🌙</span>
        <span class="text-sm font-semibold text-base-content flex-1">免打扰时段</span>
        <input type="checkbox" v-model="settings.quietHoursEnabled" @change="saveExtendedSettings" class="toggle toggle-sm" />
      </div>
      <div v-if="settings.quietHoursEnabled" class="flex items-end gap-3 mt-3">
        <div class="flex flex-col gap-1">
          <label class="text-xs text-base-content/60">开始时间</label>
          <input type="time" v-model="settings.quietHoursStart" @change="saveExtendedSettings" class="input input-bordered w-[130px]" />
        </div>
        <span class="text-xs text-base-content/60 pb-2">至</span>
        <div class="flex flex-col gap-1">
          <label class="text-xs text-base-content/60">结束时间</label>
          <input type="time" v-model="settings.quietHoursEnd" @change="saveExtendedSettings" class="input input-bordered w-[130px]" />
        </div>
      </div>
      <p v-if="settings.quietHoursEnabled" class="mt-2.5 px-3.5 py-2.5 bg-primary/10 rounded-lg text-sm text-base-content leading-relaxed">
        在 <strong class="text-primary">{{ settings.quietHoursStart || '22:00' }}</strong> 到 <strong class="text-primary">{{ settings.quietHoursEnd || '08:00' }}</strong> 期间将不会发送通知
      </p>
    </div>

    <!-- 每日总结 -->
    <div class="p-5 bg-base-100 rounded-xl border border-base-content/10 transition-all duration-200 hover:border-primary hover:shadow-lg">
      <div class="flex items-center gap-2.5 mb-3.5">
        <span class="text-xl"><SvgIcon name="barChart" size="14" /> </span>
        <span class="text-sm font-semibold text-base-content flex-1">每日总结</span>
        <input type="checkbox" v-model="settings.dailySummaryEnabled" @change="saveExtendedSettings" class="toggle toggle-sm" />
      </div>
      <div v-if="settings.dailySummaryEnabled" class="flex items-end gap-3 mt-3">
        <div class="flex flex-col gap-1">
          <label class="text-xs text-base-content/60">总结时间</label>
          <input type="time" v-model="settings.dailySummaryTime" @change="saveExtendedSettings" class="input input-bordered w-[130px]" />
        </div>
      </div>
      <p v-if="settings.dailySummaryEnabled" class="mt-2.5 px-3.5 py-2.5 bg-primary/10 rounded-lg text-sm text-base-content leading-relaxed">
        每天 <strong class="text-primary">{{ settings.dailySummaryTime || '21:00' }}</strong> 发送今日完成与待办任务总结
      </p>
    </div>

    <!-- 已完成任务静音 -->
    <div class="p-5 bg-base-100 rounded-xl border border-base-content/10 transition-all duration-200 hover:border-primary hover:shadow-lg">
      <div class="flex items-center gap-2.5 mb-3.5">
        <span class="text-xl">🔇</span>
        <span class="text-sm font-semibold text-base-content flex-1">已完成任务静音</span>
        <input type="checkbox" v-model="settings.muteCompleted" @change="saveExtendedSettings" class="toggle toggle-sm" />
      </div>
      <p v-if="settings.muteCompleted" class="mt-2.5 px-3.5 py-2.5 bg-primary/10 rounded-lg text-sm text-base-content leading-relaxed">
        已完成的任务将不再发送提醒通知
      </p>
      <p v-else class="mt-2.5 px-3.5 py-2.5 bg-primary/10 rounded-lg text-sm text-base-content leading-relaxed">
        所有任务（包括已完成）都会发送提醒通知
      </p>
    </div>

    <!-- 测试通知 -->
    <div class="flex items-center gap-3 py-3">
      <button @click="testNotification" class="btn btn-ghost"><SvgIcon name="bell" size="14" />  {{ $t('notification.test') }}</button>
      <span
        v-if="testResult"
        class="text-sm font-medium"
        :class="testResult.success ? 'text-success' : 'text-error'"
      >
        {{ testResult.message }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref, onMounted, computed } from 'vue';
import { useErrorHandler } from '../../composables/useErrorHandler';
import { getTauriAPI } from '../../utils/tauri-api';

const { handleError } = useErrorHandler();

defineProps<{
  compact?: boolean
}>()

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
    // 调用后端 notification_test 命令（带系统通知 + 提示音）
    const result = await api.notificationTest();
    const success = result?.success ?? false;

    if (success) {
      testResult.value = { success: true, message: result?.data || '测试通知已发送！' };
    } else {
      testResult.value = { success: false, message: result?.data || '通知不支持或发送失败' };
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
