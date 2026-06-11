<template>
  <div class="flex flex-col items-center justify-center py-16 px-8 text-center">
    <div class="mb-4 opacity-40">
      <SvgIcon name="bot" size="64" />
    </div>
    <div class="text-lg font-medium text-base-content/70 mb-2">
      {{ isClawMode ? 'Claw' : 'Hermes Chat' }}
    </div>
    <div class="text-sm text-base-content/40 mb-6 max-w-sm">
      {{ isClawMode
        ? 'I can help you code, debug, deploy, and manage infrastructure. Select a task below or type your own.'
        : 'Ask me anything — I can search the web, run code, manage files, and more.'
      }}
    </div>
    <div class="grid grid-cols-2 gap-2 max-w-md">
      <button
        v-for="suggestion in displaySuggestions"
        :key="suggestion.text"
        class="flex items-center gap-2 px-3 py-2 rounded-lg border border-base-content/10 bg-base-200/40 hover:bg-base-200 text-sm text-left transition-colors"
        @click="$emit('selectSuggestion', suggestion.text)"
      >
        <SvgIcon :name="suggestion.icon" size="16" class="shrink-0 text-base-content/50" />
        <span class="text-base-content/60">{{ suggestion.label }}</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import SvgIcon from '@/components/ui/SvgIcon.vue';

const props = defineProps<{
  isClawMode?: boolean;
}>();

defineEmits<{
  selectSuggestion: [text: string];
}>();

const hermesSuggestions = [
  { icon: 'globe', label: 'Search the web', text: 'Search the web for today\'s top tech news' },
  { icon: 'bell', label: 'Set a reminder', text: 'Set a reminder to check emails every day at 9 AM' },
  { icon: 'mail', label: 'Read emails', text: 'Read my latest emails and summarize them' },
  { icon: 'code', label: 'Write a script', text: 'Write a Python script to rename all files in a folder' },
  { icon: 'clock', label: 'Schedule a cron job', text: 'Schedule a cron job to back up my database every night' },
  { icon: 'barChart', label: 'Analyze data', text: 'Analyze this CSV file and show key insights' },
];

const clawSuggestions = [
  { icon: 'terminal', label: 'Check git status', text: 'Run git status and show me the current branch state' },
  { icon: 'search', label: 'Code search', text: 'Search for all TODO comments in the current project' },
  { icon: 'code', label: 'Run tests', text: 'Run all unit tests and report failures' },
  { icon: 'server', label: 'Deploy status', text: 'Check CI/CD deployment status for the current project' },
  { icon: 'bug', label: 'Debug issue', text: 'Find the root cause of this error in the logs' },
  { icon: 'barChart', label: 'Project stats', text: 'Show codebase statistics (LOC, files, languages)' },
];

const displaySuggestions = computed(() => props.isClawMode ? clawSuggestions : hermesSuggestions);
</script>
