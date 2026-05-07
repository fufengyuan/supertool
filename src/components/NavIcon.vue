<script setup lang="ts">
console.log("[NavIcon.vue] component loaded")
import { getTauriAPI } from '@/utils/tauri-api'
import { ref, watch, onMounted } from 'vue'

const props = defineProps<{ name: string }>()
const src = ref<string | null>(null)
const loading = ref(false)

const iconMap: Record<string, string> = {
  'todo': 'todo',
  'weekly-report': 'weekly-report',
  'projects': 'projects',
  'accounting': 'accounting',
  'servers': 'servers',
  'cicd': 'cicd',
  'log-aggregator': 'log-aggregator',
  'database': 'database',
  'devtools': 'devtools',
  'notes': 'notes',
  'git': 'git',
  'mfa': 'mfa',
  'vpn': 'vpn',
  'data-backup': 'data-backup',
  'lan': 'favorites',
  'collapse': 'quick-switch',
  'search': 'search',
}

async function loadIcon(name: string) {
  const key = iconMap[name] || name
  loading.value = true
  try {
    src.value = await getTauriAPI().getMenuIcon(key) ?? null
  } catch {
    src.value = null
  } finally {
    loading.value = false
  }
}

onMounted(() => loadIcon(props.name))
watch(() => props.name, (n) => loadIcon(n))
</script>

<template>
  <img
    v-if="src"
    class="nav-icon-img"
    :src="src"
    :alt="name"
    draggable="false"
  />
  <span v-else-if="loading" class="nav-icon-loading">⏳</span>
  <span v-else class="nav-icon-fallback">•</span>
</template>

<style scoped>
.nav-icon-img {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  filter: brightness(0) saturate(100%) var(--nav-icon-filter, none);
}
.dark .nav-icon-img {
  --nav-icon-filter: invert(1);
}
.nav-icon-loading,
.nav-icon-fallback {
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  flex-shrink: 0;
}
</style>
