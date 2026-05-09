<template>
  <div class="uuid-tool">
    <h3 class="text-lg font-bold text-base-content mb-5">🆔 UUID 生成器</h3>

    <div class="mb-5">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">批量生成 UUID v4</h4>
      <div class="flex gap-2.5 mb-3 flex-wrap items-center">
        <div>
          <label class="text-xs font-medium text-base-content/60 mb-1 block">生成数量</label>
          <input
            v-model.number="uuidCount"
            type="number"
            class="tool-input mono"
            style="width: 80px"
            min="1"
            max="100"
          />
        </div>
        <button class="btn btn-primary btn-sm" @click="generateUUIDs"><SvgIcon name="refresh" size="14" class="align-text-bottom" /> 生成</button>
        <button class="btn btn-ghost btn-sm" @click="copyText(uuidOutput, toast)"><SvgIcon name="file" size="14" class="align-text-bottom" /> 复制全部</button>
        <button class="btn btn-ghost btn-sm" @click="downloadUUIDs"><SvgIcon name="arrowDown" size="14" class="align-text-bottom" /> 下载 TXT</button>
      </div>

      <div v-if="uuids.length > 0" class="tool-result uuid-list">
        <div v-for="(u, i) in uuids" :key="i" class="uuid-item">{{ u }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref } from 'vue'
import { v4 as uuidv4 } from 'uuid'
import { copyText, downloadFile } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const toast = useToast()

const uuidCount = ref(5)
const uuids = ref<string[]>([])
const uuidOutput = ref('')

function generateUUIDs() {
  let count = uuidCount.value
  if (count < 1) count = 1
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