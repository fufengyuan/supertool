<template>
  <div class="max-w-[700px]">
    <h3 class="text-lg font-bold text-base-content mb-5">🔣 进制转换</h3>

    <div class="mb-5">
      <div class="flex flex-wrap gap-2.5 mb-3">
        <div>
          <label class="label-text text-xs text-base-content/60 mb-1 block">输入进制</label>
          <select v-model.number="inputBase" class="select select-bordered text-xs bg-base-200">
            <option v-for="b in baseOptions" :key="b" :value="b">{{ b }}</option>
          </select>
        </div>
      </div>

      <label class="label-text text-xs text-base-content/60 mb-1 block" style="margin-top: 12px">输入数值</label>
      <input
        v-model="input"
        class="input input-bordered w-full text-xs bg-base-200"
        :placeholder="`请输入 ${inputBase} 进制数...`"
        @input="convert"
      />

      <div class="grid grid-cols-[repeat(auto-fill,minmax(250px,1fr))] gap-3 mt-4">
        <div
          v-for="item in results"
          :key="item.label"
          class="p-3 bg-base-200 border border-base-content/10 rounded-box cursor-pointer hover:border-primary transition-all"
          @click="copyValue(item.value)"
        >
          <div class="text-xs font-semibold text-primary mb-1.5">{{ item.label }}</div>
          <div class="font-mono text-sm text-base-content break-all min-h-5">{{ item.value || '—' }}</div>
          <div class="text-[11px] text-base-content/60 mt-1">点击复制</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { baseConvert, copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const toast = useToast()
const input = ref('')
const inputBase = ref(10)

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
  if (!input.value.trim()) return targetBases.map(b => ({ ...b, value: '' }))
  return targetBases.map(b => ({
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