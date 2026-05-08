<template>
    <div class="flex flex-col items-center h-screen space-y-4 p-20">
        <h1 class="text-3xl font-bold">{{ t("home.welcome") }}</h1>
        <div class="divider"></div>
        <p class="text-lg text-center max-w-md">{{ t("home.description") }}</p>
    </div>
</template>

<script setup lang="ts">
    import { useLanguage } from "../../utils/i18n";

    const { t } = useLanguage();
</script>
