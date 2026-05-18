<template>
  <div class="p-5">
    <div class="flex flex-col gap-5">
      <!-- 页面标题 -->
      <div class="flex items-center gap-2.5 px-5 py-4 bg-base-100 border border-base-content/10 rounded-xl">
        <SvgIcon name="settings" size="20" class="text-primary shrink-0" />
        <div class="flex flex-col gap-0.5">
          <h1 class="text-lg font-bold text-base-content m-0">{{ t("settings.title") }}</h1>
          <p class="text-xs text-base-content/60 m-0">{{ t("settings.description") }}</p>
        </div>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <!-- Theme Settings -->
        <div class="bg-base-100 border border-base-content/10 rounded-xl p-5">
          <h2 class="flex items-center gap-2 text-base font-semibold text-base-content m-0 mb-4">
            <SvgIcon name="sun" size="16" />
            {{ t("settings.theme") }}
          </h2>
          <div class="flex flex-col gap-3">
            <div class="flex items-center justify-between">
              <span class="text-sm font-medium text-base-content/70">{{ t("settings.currentTheme") }}</span>
              <span class="text-xs px-2.5 py-1 rounded-full bg-base-200 text-base-content/70">{{ t(`settings.themes.${settingsStore.theme}`) }}</span>
            </div>
            <div class="flex items-center justify-between">
              <span class="text-sm font-medium text-base-content/70">{{ t("settings.toggleTheme") }}</span>
              <div class="flex items-center gap-2">
                <span class="text-xs" :class="settingsStore.theme === 'cupcake' ? 'font-semibold text-base-content' : 'text-base-content/50'">{{ t("settings.themes.cupcake") }}</span>
                <input type="checkbox" class="toggle toggle-primary toggle-sm" :checked="settingsStore.theme === 'sunset'" @change="toggleTheme" />
                <span class="text-xs" :class="settingsStore.theme === 'sunset' ? 'font-semibold text-base-content' : 'text-base-content/50'">{{ t("settings.themes.sunset") }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Language Settings -->
        <div class="bg-base-100 border border-base-content/10 rounded-xl p-5">
          <h2 class="flex items-center gap-2 text-base font-semibold text-base-content m-0 mb-4">
            <SvgIcon name="globe" size="16" />
            {{ t("settings.language") }}
          </h2>
          <div class="flex items-center justify-between">
            <span class="text-sm font-medium text-base-content/70">{{ t("settings.selectLanguage") }}</span>
            <select class="select select-bordered select-sm w-40" @change="handleLanguageChange">
              <option value="zh-CN" :selected="settingsStore.language === 'zh-CN'">中文</option>
              <option value="en-US" :selected="settingsStore.language === 'en-US'">English</option>
            </select>
          </div>
        </div>
      </div>

      <!-- Status Section -->
      <div class="flex gap-6 px-5 py-4 bg-base-100 border border-base-content/10 rounded-xl">
        <div class="flex items-center gap-2.5">
          <SvgIcon name="checkCircle" size="20" class="text-primary" />
          <div class="flex flex-col gap-0.5">
            <span class="text-xs text-base-content/60">{{ t("settings.status") }}</span>
            <span class="text-sm font-semibold text-base-content">{{ t("settings.active") }}</span>
            <span class="text-xs text-base-content/50">{{ t("settings.statusDescription") }}</span>
          </div>
        </div>
        <div class="w-px bg-base-content/10"></div>
        <div class="flex items-center gap-2.5">
          <span class="text-base"><SvgIcon name="save" size="14" class="inline-block align-text-bottom" /></span>
          <div class="flex flex-col gap-0.5">
            <span class="text-xs text-base-content/60">{{ t("settings.storage") }}</span>
            <span class="text-sm font-semibold text-base-content">{{ t("settings.local") }}</span>
            <span class="text-xs text-base-content/50">{{ t("settings.saveLocation") }}{{ saveLocation }}</span>
          </div>
        </div>
      </div>

      <!-- Auto-save info -->
      <div class="flex items-center gap-2 px-5 py-3 bg-base-100 border border-base-content/10 rounded-xl text-sm text-base-content/70">
        <SvgIcon name="info" size="16" class="shrink-0" />
        <span>{{ t("settings.autoSave") }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
defineOptions({ name: 'SettingsView' })
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { useSettingsStore } from "../../utils/settings";
import { useTheme } from "../../utils/theme";
import { useLanguage } from "../../utils/i18n";
import { appConfigDir } from '@tauri-apps/api/path';
import { ref, onMounted } from "vue";

const settingsStore = useSettingsStore();
const { toggleTheme } = useTheme();
const { switchLanguage, t } = useLanguage();

const saveLocation = ref("");

onMounted(async () => {
    const appConfigDirPath = await appConfigDir();
    saveLocation.value = `${appConfigDirPath}/settings.json`;
});

const handleLanguageChange = async (event: Event) => {
    const target = event.target as HTMLSelectElement;
    await switchLanguage(target.value as "zh-CN" | "en-US");
};
</script>
