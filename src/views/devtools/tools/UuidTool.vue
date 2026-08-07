<template>
  <ToolPage
    icon="plus"
    name="UUID 生成器"
    description="批量生成 UUID v4，支持复制全部与下载 TXT"
    @back="$emit('back')"
  >
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <div class="flex flex-wrap items-end gap-3">
        <div>
          <span class="text-[11px] font-medium text-base-content/50 mb-1 block">生成数量</span>
          <input
            v-model.number="uuidCount"
            type="number"
            class="input input-bordered input-sm font-mono bg-base-200/60"
            style="width: 88px"
            min="1"
            max="100"
          />
        </div>
        <button class="btn btn-primary btn-sm" @click="generateUUIDs"><SvgIcon name="refresh" size="12" /> 生成</button>
        <button class="btn btn-outline btn-sm" @click="copyText(uuidOutput, toast)" :disabled="!uuidOutput"><SvgIcon name="copy" size="12" /> 复制全部</button>
        <button class="btn btn-ghost btn-sm" @click="downloadUUIDs" :disabled="!uuidOutput"><SvgIcon name="download" size="12" /> 下载 TXT</button>
      </div>
    </div>

    <div v-if="uuids.length > 0" class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <h4 class="text-xs font-semibold text-base-content/70 mb-2.5">{{ uuids.length }} 个 UUID</h4>
      <div class="max-h-72 overflow-y-auto flex flex-col gap-1">
        <div
          v-for="(u, i) in uuids"
          :key="i"
          class="px-3 py-1.5 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-sm text-base-content cursor-pointer hover:border-primary/40 transition-colors break-all"
          :title="`复制 ${u}`"
          @click="copyText(u, toast, '已复制')"
        >{{ u }}</div>
      </div>
    </div>
    <div v-else class="py-14 text-center text-xs text-base-content/40 bg-base-100/50 border border-dashed border-base-content/15 rounded-xl">
      点击「生成」批量创建 UUID v4，点击单个 UUID 可复制
    </div>
  </ToolPage>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { ref } from 'vue'
import { v4 as uuidv4 } from 'uuid'
import { copyText, downloadFile } from '../toolUtils'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

const toast = useToast()

const uuidCount = ref(5)
const uuids = ref<string[]>([])
const uuidOutput = ref('')

function generateUUIDs() {
  let count = uuidCount.value
  if (count < 1) {count = 1}
  if (count > 100) {
    count = 100
    toast.warning('最多生成 100 个')
  }
  uuids.value = []
  for (let i = 0; i < count; i++) {
    uuids.value.push(uuidv4())
  }
  uuidOutput.value = uuids.value.join('\n')
  toast.success(`已生成 ${count} 个 UUID`)
}

function downloadUUIDs() {
  if (uuids.value.length === 0) { toast.error('请先生成 UUID'); return }
  downloadFile(uuidOutput.value, 'uuids.txt', 'text/plain')
  toast.success('UUID 文件已下载')
}
</script>
