<template>
  <div class="chat-empty flex flex-col items-center justify-center py-16 px-8 text-center">
    <div class="chat-empty-icon mb-4 opacity-40">
      <SvgIcon name="bot" size="64" />
    </div>
    <div class="text-lg font-medium text-base-content/70 mb-2">Hermes Chat</div>
    <div class="text-sm text-base-content/40 mb-6 max-w-sm">
      Ask me anything — I can search the web, run code, manage files, and more.
    </div>
    <div class="grid grid-cols-2 gap-2 max-w-md">
      <button
        v-for="suggestion in suggestions"
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
import SvgIcon from '@/components/ui/SvgIcon.vue';

defineEmits<{
  selectSuggestion: [text: string];
}>();

const suggestions = [
  { icon: 'globe', label: 'Search the web', text: 'Search the web for today\'s top tech news' },
  { icon: 'bell', label: 'Set a reminder', text: 'Set a reminder to check emails every day at 9 AM' },
  { icon: 'mail', label: 'Read emails', text: 'Read my latest emails and summarize them' },
  { icon: 'code', label: 'Write a script', text: 'Write a Python script to rename all files in a folder' },
  { icon: 'clock', label: 'Schedule a cron job', text: 'Schedule a cron job to back up my database every night' },
  { icon: 'barChart', label: 'Analyze data', text: 'Analyze this CSV file and show key insights' },
];
</script>
