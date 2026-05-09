<template>
  <div class="p-5">
    <div class="flex flex-col gap-5">
      <!-- 页面标题 -->
      <div class="flex items-center gap-2.5 px-5 py-4 bg-base-100 border border-base-content/10 rounded-xl">
        <svg class="w-5 h-5 text-primary shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/>
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
        </svg>
        <div class="flex flex-col gap-0.5">
          <h1 class="text-lg font-bold text-base-content m-0">{{ t("settings.title") }}</h1>
          <p class="text-xs text-base-content/60 m-0">{{ t("settings.description") }}</p>
        </div>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <!-- Theme Settings -->
        <div class="bg-base-100 border border-base-content/10 rounded-xl p-5">
          <h2 class="flex items-center gap-2 text-base font-semibold text-base-content m-0 mb-4">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"/>
            </svg>
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
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 5h12M9 3v2m1.048 9.5A18.022 18.022 0 016.412 9m6.088 9h7M11 21l5-10 5 10M12.751 5C11.783 10.77 8.07 15.61 3 18.129"/>
            </svg>
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
          <svg class="w-5 h-5 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
          </svg>
          <div class="flex flex-col gap-0.5">
            <span class="text-xs text-base-content/60">{{ t("settings.status") }}</span>
            <span class="text-sm font-semibold text-base-content">{{ t("settings.active") }}</span>
            <span class="text-xs text-base-content/50">{{ t("settings.statusDescription") }}</span>
          </div>
        </div>
        <div class="w-px bg-base-content/10"></div>
        <div class="flex items-center gap-2.5">
          <span class="text-base"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg></span>
          <div class="flex flex-col gap-0.5">
            <span class="text-xs text-base-content/60">{{ t("settings.storage") }}</span>
            <span class="text-sm font-semibold text-base-content">{{ t("settings.local") }}</span>
            <span class="text-xs text-base-content/50">{{ t("settings.saveLocation") }}{{ saveLocation }}</span>
          </div>
        </div>
      </div>

      <!-- Auto-save info -->
      <div class="flex items-center gap-2 px-5 py-3 bg-base-100 border border-base-content/10 rounded-xl text-sm text-base-content/70">
        <svg class="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
        </svg>
        <span>{{ t("settings.autoSave") }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
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
