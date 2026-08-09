import { watch } from "vue";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "./settings";

export function useLanguage() {
    const settingsStore = useSettingsStore();
    const { locale, t } = useI18n();

    // Watch for settings changes and update locale
    watch(
        () => settingsStore.language,
        (newLanguage) => {
            // i18n messages 的 key 是 'zh-CN' / 'en'，映射 en-US → en
            locale.value = (newLanguage === "en-US" ? "en" : "zh-CN") as "zh-CN" | "en";
        },
    );

    // Switch language and save to settings + localStorage（main.ts 启动时读 localStorage）
    const switchLanguage = async (newLanguage: "zh-CN" | "en-US") => {
        await settingsStore.updateAndSaveSettings({ language: newLanguage });
        try {
            localStorage.setItem("locale", newLanguage === "en-US" ? "en" : "zh-CN");
        } catch { /* ignore */ }
        locale.value = (newLanguage === "en-US" ? "en" : "zh-CN") as "zh-CN" | "en";
    };

    // Toggle between supported languages
    const toggleLanguage = async () => {
        const newLanguage = settingsStore.language === "zh-CN" ? "en-US" : "zh-CN";
        await switchLanguage(newLanguage);
    };

    return {
        switchLanguage,
        toggleLanguage,
        currentLanguage: () => settingsStore.language,
        t,
    };
}
