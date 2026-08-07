import { defineStore } from "pinia";
import { Store } from "@tauri-apps/plugin-store";

export interface UserSettings {
    theme: "cupcake" | "sunset";
    language: "en-US" | "zh-CN";
    httpProxy: string;
}

const defaultSettings: UserSettings = {
    theme: "cupcake",
    language: "zh-CN",
    httpProxy: "",
};

let diskStore: Store | null = null;

export const useSettingsStore = defineStore("settings", {
    state: (): UserSettings => ({
        ...defaultSettings,
    }),

    actions: {
        async initializeSettings() {
            try {
                if (!diskStore) {
                    diskStore = await Store.load("settings.json");
                }
                const loadedSettings = await diskStore.get<UserSettings>("userSettings");
                if (loadedSettings) {
                    this.$patch(loadedSettings);
                }
            } catch (error) {
                console.error("Failed to load settings:", error);
            }
        },

        async updateAndSaveSettings(newSettings: Partial<UserSettings>) {
            try {
                if (!diskStore) {
                    diskStore = await Store.load("settings.json");
                }
                this.$patch(newSettings);
                await diskStore.set("userSettings", this.$state);
                await diskStore.save();
            } catch (error) {
                console.error("Failed to save settings:", error);
            }
        },
    },
});
