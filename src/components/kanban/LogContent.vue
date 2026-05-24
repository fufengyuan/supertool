<template>
  <div class="log-content">
    <template v-for="(line, idx) in parsedLines" :key="idx">
      <!-- User message -->
      <div v-if="line.type === 'user'" class="flex items-start gap-2 my-2">
        <div class="w-5 h-5 rounded-full bg-primary flex items-center justify-center text-xs">U</div>
        <div class="flex-1">
          <div class="text-xs text-base-content/50 mb-1">用户</div>
          <div class="text-sm bg-primary/10 rounded px-2 py-1">{{ line.content }}</div>
        </div>
      </div>
      
      <!-- Tool call -->
      <div v-if="line.type === 'tool'" class="flex items-center gap-1.5 my-1 px-2 py-1 bg-base-200/50 rounded">
        <span v-if="line.emoji" class="text-sm">{{ line.emoji }}</span>
        <span v-else class="w-4 h-4 rounded bg-info/20 flex items-center justify-center text-xs">T</span>
        <span class="text-xs font-medium text-info">{{ line.name }}</span>
        <span v-if="line.status" :class="statusClass(line.status)" class="text-xs ml-1">{{ line.status }}</span>
      </div>
      
      <!-- Error -->
      <div v-if="line.type === 'error'" class="my-1 px-2 py-1 bg-error/10 rounded border border-error/30">
        <div class="flex items-center gap-1 text-error text-xs">
          <SvgIcon name="alert-circle" size="12" />
          <span class="font-medium">错误</span>
        </div>
        <div class="text-xs text-error/80 mt-1 whitespace-pre-wrap">{{ line.content }}</div>
      </div>
      
      <!-- Warning -->
      <div v-if="line.type === 'warning'" class="my-1 px-2 py-1 bg-warning/10 rounded border border-warning/30">
        <div class="flex items-center gap-1 text-warning text-xs">
          <span class="font-medium">⚠️ 警告</span>
        </div>
        <div class="text-xs text-warning/80 mt-1">{{ line.content }}</div>
      </div>
      
      <!-- Success -->
      <div v-if="line.type === 'success'" class="my-1 px-2 py-1 bg-success/10 rounded">
        <div class="flex items-center gap-1 text-success text-xs">
          <span class="font-medium">✓ 成功</span>
        </div>
        <div class="text-xs text-success/80 mt-1">{{ line.content }}</div>
      </div>
      
      <!-- Info/System -->
      <div v-if="line.type === 'info'" class="my-0.5 text-xs text-base-content/50 italic">{{ line.content }}</div>
      
      <!-- Divider -->
      <div v-if="line.type === 'divider'" class="my-2 border-t border-base-content/10"></div>
      
      <!-- Plain text -->
      <div v-if="line.type === 'plain'" class="text-xs text-base-content/70">{{ line.content }}</div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import SvgIcon from '@/components/ui/SvgIcon.vue';

const props = defineProps<{
  content: string;
}>();

interface ParsedLine {
  type: 'user' | 'tool' | 'error' | 'warning' | 'success' | 'info' | 'divider' | 'plain';
  content?: string;
  emoji?: string;
  name?: string;
  status?: string;
}

const parsedLines = computed(() => {
  const lines = props.content.split('\n');
  const result: ParsedLine[] = [];
  
  for (const line of lines) {
    const trimmed = line.trim();
    
    // Empty lines
    if (!trimmed) continue;
    
    // User message (Query: ...)
    if (trimmed.startsWith('Query:')) {
      result.push({ type: 'user', content: trimmed.replace('Query:', '').trim() });
      continue;
    }
    
    // Divider
    if (trimmed.startsWith('──') || trimmed.startsWith('─') || trimmed.includes('────')) {
      result.push({ type: 'divider' });
      continue;
    }
    
    // Errors
    if (trimmed.includes('❌') || trimmed.includes('Error:') || trimmed.includes('error:')) {
      result.push({ type: 'error', content: trimmed.replace(/❌|Error:|error:/g, '').trim() });
      continue;
    }
    
    // Warnings
    if (trimmed.includes('⚠') || trimmed.includes('⚠️') || trimmed.includes('WARNING')) {
      result.push({ type: 'warning', content: trimmed.replace(/⚠|⚠️|WARNING/g, '').trim() });
      continue;
    }
    
    // Success
    if (trimmed.includes('✓') || trimmed.includes('✅') || trimmed.includes('Success')) {
      result.push({ type: 'success', content: trimmed.replace(/✓|✅|Success/g, '').trim() });
      continue;
    }
    
    // Tool calls (lines with emoji + tool name)
    const toolMatch = trimmed.match(/^([^\s]+)\s+(\w+)\s*(.*)$/);
    if (toolMatch && toolMatch[1].length <= 2) {
      // Likely emoji + tool name
      result.push({
        type: 'tool',
        emoji: toolMatch[1],
        name: toolMatch[2],
        status: toolMatch[3]?.trim() || undefined,
      });
      continue;
    }
    
    // System info (Initializing, Resume, etc.)
    if (trimmed.startsWith('Initializing') || trimmed.startsWith('Resume') || trimmed.startsWith('Session:') || trimmed.startsWith('Duration:') || trimmed.startsWith('Messages:')) {
      result.push({ type: 'info', content: trimmed });
      continue;
    }
    
    // Plain text
    result.push({ type: 'plain', content: trimmed });
  }
  
  return result;
});

function statusClass(status: string): string {
  if (status.includes('success') || status.includes('completed')) return 'text-success';
  if (status.includes('error') || status.includes('failed')) return 'text-error';
  if (status.includes('running') || status.includes('pending')) return 'text-info';
  return 'text-base-content/60';
}
</script>

<style scoped>
.log-content {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}
</style>