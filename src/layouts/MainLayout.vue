<template>
    <div class="h-screen flex flex-col">
        <TitleBar />
        <div class="flex-1 p-4 bg-base-200">
            <router-view />
        </div>
    </div>
</template>

<script setup lang="ts">
    import TitleBar from "../components/TitleBar.vue";
</script>
