<template>
  <ToolPage
    icon="pencil"
    name="进制转换"
    description="2-64 进制任意互转，实时显示常用进制结果，点击复制"
    @back="$emit('back')"
  >
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <div class="flex flex-wrap items-end gap-3">
        <div>
          <span class="text-[11px] font-medium text-base-content/50 mb-1 block">输入进制</span>
          <select v-model.number="inputBase" class="select select-bordered select-sm bg-base-200/60">
            <option v-for="b in baseOptions" :key="b" :value="b">{{ b }}</option>
          </select>
        </div>
        <div>
          <span class="text-[11px] font-medium text-base-content/50 mb-1 block">自定义目标进制 <span class="text-base-content/30">(2-64，可选)</span></span>
          <input
            v-model.number="targetBase"
            type="number"
            min="2"
            max="64"
            placeholder="留空使用常用进制"
            class="input input-bordered input-sm w-[130px] font-mono text-xs bg-base-200/60"
            @input="convert"
          />
        </div>
      </div>
      <div class="mt-3">
        <span class="text-[11px] font-medium text-base-content/50 mb-1 block">输入数值</span>
        <input
          v-model="input"
          class="input input-bordered w-full font-mono text-sm bg-base-200/60"
          :placeholder="`请输入 ${inputBase} 进制数...`"
          @input="convert"
        />
      </div>
    </div>

    <div v-if="input" class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
      <div
        v-for="item in results"
        :key="item.label"
        class="group bg-base-100 border border-base-content/10 rounded-xl p-4 cursor-pointer hover:border-primary/40 hover:shadow-md transition-all"
        title="点击复制"
        @click="copyValue(item.value)"
      >
        <div class="text-[11px] font-semibold text-primary mb-1.5">{{ item.label }}</div>
        <div class="font-mono text-sm text-base-content break-all min-h-5">{{ item.value || '—' }}</div>
        <div class="text-[10px] text-base-content/40 mt-1.5 flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
          <SvgIcon name="copy" size="10" /> 点击复制
        </div>
      </div>
    </div>
    <div v-else class="py-14 text-center text-xs text-base-content/40 bg-base-100/50 border border-dashed border-base-content/15 rounded-xl">
      输入数值后，实时显示 2 / 8 / 10 / 16 / 32 / 64 进制结果
    </div>
  </ToolPage>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { ref, computed } from 'vue'
import { baseConvert, copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

const toast = useToast()
const input = ref('')
const inputBase = ref(10)
// 自定义目标进制（2-64），空 = 仅显示常用进制
const targetBase = ref<number | null>(null)

const baseOptions = computed(() => {
  const bases: number[] = []
  for (let i = 2; i <= 64; i++) {
    bases.push(i)
  }
  return bases
})

const targetBases = [
  { label: '二进制 (2)', base: 2 },
  { label: '八进制 (8)', base: 8 },
  { label: '十进制 (10)', base: 10 },
  { label: '十六进制 (16)', base: 16 },
  { label: 'Base32 (32)', base: 32 },
  { label: 'Base64 (64)', base: 64 },
]

const results = computed(() => {
  const baseItems = [...targetBases]
  // 自定义目标进制（2-64 整数），与预设去重后追加显示
  const tb = targetBase.value
  if (typeof tb === 'number' && Number.isInteger(tb) && tb >= 2 && tb <= 64 && !baseItems.some(b => b.base === tb)) {
    baseItems.push({ label: `自定义 (${tb})`, base: tb })
  }
  if (!input.value.trim()) {return baseItems.map(b => ({ ...b, value: '' }))}
  return baseItems.map(b => ({
    ...b,
    value: baseConvert(input.value, inputBase.value, b.base),
  }))
})

function convert() {
  // reactive via computed
}

function copyValue(value: string) {
  if (!value) {
    toast.warning('没有可复制的内容')
    return
  }
  copyText(value, toast)
}
</script>
