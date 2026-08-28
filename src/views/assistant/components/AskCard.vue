<template>
  <div class="border rounded-xl overflow-hidden bg-base-100 border-base-content/10">
    <div class="flex items-center gap-2 px-3 py-2 bg-base-200 border-b border-base-content/10">
      <SvgIcon name="messageSquare" size="14" class="shrink-0 text-primary" />
      <span class="text-xs font-semibold text-base-content truncate flex-1">助手提问</span>
      <span class="text-[10px] px-1.5 py-0.5 rounded-full shrink-0" :class="badgeClass">{{ statusLabel }}</span>
    </div>

    <div class="p-3 flex flex-col gap-2.5">
      <p class="text-xs font-medium text-base-content m-0 leading-relaxed">{{ ask.question }}</p>
      <p v-if="ask.description" class="text-[11px] text-base-content/60 m-0 leading-relaxed">{{ ask.description }}</p>

      <template v-if="ask.status === 'pending'">
        <!-- 单选 -->
        <div v-if="ask.type === 'single'" class="flex flex-col gap-1.5">
          <label
            v-for="o in ask.options"
            :key="o"
            class="flex items-center gap-2 px-2.5 py-1.5 rounded-lg border cursor-pointer select-none transition-colors"
            :class="singlePicked === o ? 'border-primary bg-primary/5' : 'border-base-content/10 hover:border-base-content/25'"
          >
            <input v-model="singlePicked" type="radio" :value="o" class="radio radio-xs radio-primary" />
            <span class="text-[11px] text-base-content">{{ o }}</span>
          </label>
        </div>

        <!-- 多选 -->
        <div v-else-if="ask.type === 'multiple'" class="flex flex-col gap-1.5">
          <label
            v-for="o in ask.options"
            :key="o"
            class="flex items-center gap-2 px-2.5 py-1.5 rounded-lg border cursor-pointer select-none transition-colors"
            :class="multiPicked.includes(o) ? 'border-primary bg-primary/5' : 'border-base-content/10 hover:border-base-content/25'"
          >
            <input v-model="multiPicked" type="checkbox" :value="o" class="checkbox checkbox-xs checkbox-primary" />
            <span class="text-[11px] text-base-content">{{ o }}</span>
          </label>
        </div>

        <!-- 自定义答案（非 text 时作为「其他」补充，text 时为唯一输入） -->
        <div class="flex flex-col gap-1">
          <label
            v-if="ask.type !== 'text'"
            class="text-[11px] text-base-content/60"
            :class="customOpen || custom ? 'text-primary/80' : ''"
          >
            不想选上面的？自定义输入答案
          </label>
          <textarea
            v-if="ask.type !== 'text' && !customOpen && !custom"
            rows="1"
            class="textarea textarea-bordered textarea-xs w-full text-[11px] resize-none cursor-pointer"
            placeholder="点这里自己输入答案…"
            @click="customOpen = true"
          ></textarea>
          <textarea
            v-else
            v-model="custom"
            rows="2"
            :placeholder="ask.type === 'text' ? '请输入你的答案…' : '自定义答案（会与勾选内容一起提交）'"
            class="textarea textarea-bordered textarea-xs w-full text-[11px] leading-snug resize-none"
          ></textarea>
        </div>

        <p v-if="localError" class="text-[11px] text-error m-0">{{ localError }}</p>

        <div class="flex items-center gap-2 pt-0.5">
          <button class="btn btn-primary btn-xs" @click="submit">
            <SvgIcon name="check" size="12" /> 提交答案
          </button>
        </div>
      </template>

      <p v-else class="text-[11px] text-success flex items-center gap-1 m-0">
        <SvgIcon name="check" size="12" /> 已回答，助手正在处理
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import SvgIcon from '../../../components/ui/SvgIcon.vue'
import type { AssistantAsk } from '../../../composables/useAssistantChat'

const props = defineProps<{ ask: AssistantAsk }>()
const emit = defineEmits<{ (e: 'submit', answer: string | string[]): void }>()

const singlePicked = ref('')
const multiPicked = ref<string[]>([])
const custom = ref('')
const customOpen = ref(false)
const localError = ref('')

const statusLabel = computed(() => (props.ask.status === 'submitted' ? '已回答' : '待回答'))
const badgeClass = computed(() =>
  props.ask.status === 'submitted' ? 'bg-success/10 text-success' : 'bg-primary/10 text-primary',
)

function submit() {
  const c = custom.value.trim()
  if (props.ask.type === 'text') {
    if (!c) {
      localError.value = '请输入答案'
      return
    }
    emit('submit', c)
    return
  }
  const selected = props.ask.type === 'single' ? singlePicked.value : [...multiPicked.value]
  if (!selected && !c) {
    localError.value = props.ask.type === 'single' ? '请勾选一个选项或输入自定义答案' : '请至少勾选一个选项或输入自定义答案'
    return
  }
  const answer =
    props.ask.type === 'single'
      ? c || selected
      : c
        ? [...selected, c]
        : selected
  emit('submit', answer)
}
</script>
