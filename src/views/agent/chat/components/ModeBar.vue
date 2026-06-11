<template>
  <div class="border-b border-base-content/10 bg-base-100">
    <!-- Plan mode bar -->
    <div
      v-if="planMode"
      class="flex items-center gap-3 px-4 py-2 bg-primary/5 border-l-2 border-primary"
    >
      <div class="flex items-center gap-1.5 shrink-0">
        <div class="w-2 h-2 rounded-full bg-primary animate-pulse" />
        <span class="text-xs font-semibold text-primary uppercase tracking-wider">Plan</span>
      </div>
      <span class="text-xs text-base-content/60 truncate flex-1">
        Read-only mode — agent is analyzing before writing
      </span>
      <button
        class="btn btn-ghost btn-xs text-primary hover:text-primary/80"
        title="Exit plan mode"
        @click="$emit('exitPlan')"
      >
        <SvgIcon name="x" size="12" />
        Exit
      </button>
    </div>

    <!-- Goal mode bar -->
    <div
      v-if="goalMode || goalStatus === 'paused'"
      class="flex items-center gap-3 px-4 py-2"
      :class="goalStatus === 'paused' ? 'border-l-2 border-warning bg-warning/5' : 'border-l-2 border-info bg-info/5'"
    >
      <div class="flex items-center gap-1.5 shrink-0">
        <div
          class="w-2 h-2 rounded-full"
          :class="{
            'bg-info animate-pulse': goalStatus === 'active',
            'bg-warning': goalStatus === 'paused',
            'bg-emerald-500': goalStatus === 'complete',
            'bg-error': goalStatus === 'budget-limited',
          }"
        />
        <span class="text-xs font-semibold uppercase tracking-wider"
          :class="{
            'text-info': goalStatus === 'active',
            'text-warning': goalStatus === 'paused',
            'text-emerald-500': goalStatus === 'complete',
            'text-error': goalStatus === 'budget-limited',
          }"
        >
          {{ statusLabel }}
        </span>
      </div>

      <!-- Goal objective (truncated) -->
      <span class="text-xs text-base-content/80 truncate flex-1 min-w-0" :title="goalText">
        {{ goalText }}
      </span>

      <!-- Token budget bar -->
      <div
        v-if="tokenBudget != null && tokenBudget > 0"
        class="hidden sm:flex items-center gap-1.5 shrink-0"
      >
        <div class="w-20 h-1.5 rounded-full bg-base-200 overflow-hidden">
          <div
            class="h-full rounded-full transition-all duration-500"
            :class="budgetPercent > 80 ? 'bg-error' : budgetPercent > 50 ? 'bg-warning' : 'bg-info'"
            :style="{ width: Math.min(budgetPercent, 100) + '%' }"
          />
        </div>
        <span class="text-[10px] text-base-content/40 tabular-nums">
          {{ tokensUsed }}/{{ tokenBudget }}
        </span>
      </div>

      <!-- Token count (no budget) -->
      <span
        v-else-if="(tokensUsed || 0) > 0"
        class="text-[10px] text-base-content/40 tabular-nums shrink-0 hidden sm:inline"
      >
        {{ tokensUsed || 0 }} tokens
      </span>

      <!-- Status badge -->
      <span
        class="text-[10px] font-medium px-1.5 py-0.5 rounded shrink-0"
        :class="statusBadgeClass"
      >
        {{ goalTurnsUsed }}/{{ goalMaxTurns }} turns
      </span>

      <!-- Controls -->
      <div class="flex items-center gap-1 shrink-0">
        <button
          v-if="goalStatus === 'paused'"
          class="btn btn-ghost btn-xs text-success"
          title="Resume goal"
          @click="$emit('resumeGoal')"
        >
          <SvgIcon name="play" size="12" />
          Resume
        </button>
        <button
          v-if="goalStatus === 'active'"
          class="btn btn-ghost btn-xs text-warning"
          title="Pause goal"
          @click="$emit('pauseGoal')"
        >
          <SvgIcon name="pause" size="12" />
          Pause
        </button>
        <button
          class="btn btn-ghost btn-xs text-base-content/40 hover:text-error"
          title="Drop goal"
          @click="$emit('dropGoal')"
        >
          <SvgIcon name="x" size="12" />
        </button>
      </div>
    </div>

    <!-- Loop mode bar -->
    <div
      v-if="loopMode"
      class="flex items-center gap-3 px-4 py-2 border-l-2 border-success bg-success/5"
    >
      <div class="flex items-center gap-1.5 shrink-0">
        <SvgIcon name="refresh" size="12" class="text-success animate-spin" />
        <span class="text-xs font-semibold text-success uppercase tracking-wider">Loop</span>
      </div>

      <!-- Iteration counter -->
      <span class="text-xs tabular-nums text-base-content/70 shrink-0">
        #{{ loopIterations }}
        <span v-if="loopMaxIterations > 0">/{{ loopMaxIterations }}</span>
      </span>

      <!-- Loop prompt preview -->
      <span
        v-if="loopPrompt"
        class="text-xs text-base-content/60 truncate flex-1 min-w-0"
        :title="loopPrompt"
      >
        "{{ loopPrompt }}"
      </span>
      <span v-else class="text-xs text-base-content/30 italic flex-1">
        Next message will auto-resubmit after each turn
      </span>

      <!-- Controls -->
      <div class="flex items-center gap-1 shrink-0">
        <button
          class="btn btn-ghost btn-xs text-warning"
          title="Pause loop (keep mode)"
          @click="$emit('pauseLoop')"
        >
          <SvgIcon name="pause" size="12" />
        </button>
        <button
          class="btn btn-ghost btn-xs text-error"
          title="Stop loop"
          @click="$emit('stopLoop')"
        >
          <SvgIcon name="square" size="12" />
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import SvgIcon from '@/components/ui/SvgIcon.vue';

const props = defineProps<{
  planMode: boolean;
  goalMode: boolean;
  goalText: string;
  goalStatus: string;
  goalTurnsUsed: number;
  goalMaxTurns: number;
  tokensUsed?: number;
  tokenBudget?: number | null;
  loopMode: boolean;
  loopIterations: number;
  loopMaxIterations: number;
  loopPrompt: string | null;
}>();

defineEmits<{
  exitPlan: [];
  resumeGoal: [];
  pauseGoal: [];
  dropGoal: [];
  pauseLoop: [];
  stopLoop: [];
}>();

const statusLabel = computed(() => {
  const map: Record<string, string> = {
    active: 'Goal Active',
    paused: 'Goal Paused',
    complete: 'Goal Complete',
    'budget-limited': 'Budget Exhausted',
    dropped: 'Goal Dropped',
  };
  return map[props.goalStatus] || 'Goal';
});

const budgetPercent = computed(() => {
  if (!props.tokenBudget || props.tokenBudget <= 0) return 0;
  return Math.round(((props.tokensUsed || 0) / props.tokenBudget) * 100);
});

const statusBadgeClass = computed(() => {
  if (props.goalStatus === 'paused') return 'text-warning bg-warning/10';
  if (props.goalStatus === 'complete') return 'text-success bg-success/10';
  if (props.goalStatus === 'budget-limited') return 'text-error bg-error/10';
  if (props.goalStatus === 'dropped') return 'text-base-content/30 bg-base-200';
  return 'text-info bg-info/10';
});
</script>
