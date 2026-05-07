<template>
    <div class="min-h-screen bg-base-200 p-8">
        <div class="max-w-4xl mx-auto">
            <div class="hero bg-base-100 rounded-box shadow-xl mb-8">
                <div class="hero-content text-center py-8">
                    <div>
                        <h1 class="text-4xl font-bold text-base-content mb-4">
                            {{ t("settings.title") }}
                        </h1>
                        <p class="text-base-content/70">
                            {{ t("settings.description") }}
                        </p>
                    </div>
                </div>
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                <!-- Theme Settings -->
                <div class="card bg-base-100 shadow-xl">
                    <div class="card-body">
                        <h2 class="card-title flex items-center gap-2">
                            <svg
                                class="w-5 h-5"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"
                                />
                            </svg>
                            {{ t("settings.theme") }}
                        </h2>
                        <div class="form-control space-y-1">
                            <!-- 当前主题 -->
                            <div class="flex items-center justify-between">
                                <span class="label-text font-medium">{{
                                    t("settings.currentTheme")
                                }}</span>
                                <div class="badge badge-outline">
                                    {{ t(`settings.themes.${settingsStore.theme}`) }}
                                </div>
                            </div>
                            <div class="divider my-0"></div>
                            <!-- 切换主题 -->
                            <div class="flex items-center justify-between">
                                <span class="label-text font-medium">{{
                                    t("settings.toggleTheme")
                                }}</span>
                                <div class="flex items-center gap-2">
                                    <span
                                        :class="
                                            settingsStore.theme === 'cupcake'
                                                ? 'font-bold'
                                                : 'text-base-content/50'
                                        "
                                        >{{ t("settings.themes.cupcake") }}</span
                                    >
                                    <input
                                        type="checkbox"
                                        class="toggle toggle-primary"
                                        :checked="settingsStore.theme === 'sunset'"
                                        @change="toggleTheme"
                                    />
                                    <span
                                        :class="
                                            settingsStore.theme === 'sunset'
                                                ? 'font-bold'
                                                : 'text-base-content/50'
                                        "
                                        >{{ t("settings.themes.sunset") }}</span
                                    >
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Language Settings -->
                <div class="card bg-base-100 shadow-xl">
                    <div class="card-body">
                        <h2 class="card-title flex items-center gap-2">
                            <svg
                                class="w-5 h-5"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M3 5h12M9 3v2m1.048 9.5A18.022 18.022 0 016.412 9m6.088 9h7M11 21l5-10 5 10M12.751 5C11.783 10.77 8.07 15.61 3 18.129"
                                />
                            </svg>
                            {{ t("settings.language") }}
                        </h2>
                        <div class="form-control space-y-4">
                            <div class="flex items-center justify-between">
                                <span class="label-text font-medium">{{
                                    t("settings.selectLanguage")
                                }}</span>
                                <div class="w-40">
                                    <select
                                        class="select select-primary w-full"
                                        @change="handleLanguageChange"
                                    >
                                        <option
                                            value="zh-CN"
                                            :selected="settingsStore.language === 'zh-CN'"
                                        >
                                            中文
                                        </option>
                                        <option
                                            value="en-US"
                                            :selected="settingsStore.language === 'en-US'"
                                        >
                                            English
                                        </option>
                                    </select>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Stats Section -->
            <div class="stats bg-base-100 shadow-xl w-full mt-8">
                <div class="stat">
                    <div class="stat-figure text-primary">
                        <svg class="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                            />
                        </svg>
                    </div>
                    <div class="stat-title">{{ t("settings.status") }}</div>
                    <div class="stat-value text-primary">{{ t("settings.active") }}</div>
                    <div class="stat-desc">{{ t("settings.statusDescription") }}</div>
                </div>

                <div class="stat">
                    <div class="stat-figure text-secondary"></div>
                    <div class="stat-title">{{ t("settings.storage") }}</div>
                    <div class="stat-value text-secondary">{{ t("settings.local") }}</div>
                    <div class="stat-desc">{{ t("settings.saveLocation") }}{{ saveLocation }}</div>
                </div>
            </div>

            <!-- Footer -->
            <div class="text-center mt-8">
                <div class="alert alert-info">
                    <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                        />
                    </svg>
                    <span>{{ t("settings.autoSave") }}</span>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
    import { useSettingsStore } from "../utils/settings";
    import { useTheme } from "../utils/theme";
    import { useLanguage } from "../utils/i18n";
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
