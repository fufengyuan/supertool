<template>
  <div>
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-base font-semibold m-0">IP 黑白名单</h3>
      <button @click="addRow" class="btn btn-primary btn-sm">
        <SvgIcon name="plus" size="14" /> 新增名单
      </button>
    </div>

    <div v-if="loading" class="flex items-center justify-center py-8 text-base-content/50">
      <SvgIcon name="clock" size="14" /> 加载中...
    </div>

    <div v-else-if="items.length === 0" class="flex flex-col items-center justify-center py-8 text-base-content/50">
      <SvgIcon name="shield" size="24" class="mb-2 opacity-50" />
      <p class="text-sm">暂无黑白名单</p>
    </div>

    <div v-else class="overflow-x-auto">
      <table class="table table-zebra table-sm">
        <thead>
          <tr>
            <th class="w-44">名称</th>
            <th>IP 列表 (每行一个)</th>
            <th class="w-24 text-center">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(item, index) in items" :key="item._key">
            <td>
              <input v-model="item.name" placeholder="名单名称" class="input input-bordered input-sm w-full" />
            </td>
            <td>
              <textarea
                v-model="item.ip"
                placeholder="10.0.0.1&#10;192.168.1.0/24&#10;..."
                class="textarea textarea-bordered textarea-sm w-full font-mono"
                rows="3"
              ></textarea>
            </td>
            <td class="text-center">
              <button @click="onDelete(index)" class="btn btn-ghost btn-xs btn-square text-error" title="删除">
                <SvgIcon name="trash" size="14" />
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div class="flex items-center gap-3 mt-4">
      <button @click="onSave" class="btn btn-primary" :disabled="loading">
        <SvgIcon name="check" size="14" /> 保存
      </button>
      <span v-if="saved" class="text-sm text-success">已保存</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { getTauriAPI } from '../../utils/tauri-api'
import { useToast } from '../../composables/useToast'
import SvgIcon from '@/components/ui/SvgIcon.vue'

const props = defineProps<{ presetId: string }>()
const api = getTauriAPI()
const toast = useToast()
const loading = ref(false)
const saved = ref(false)

interface DenyItem {
  id: string
  presetId: string
  name: string
  ip: string
  createdAt: string
  _key: number
}

let keyCounter = 0
const items = ref<DenyItem[]>([])

function addRow() {
  items.value.push({
    id: crypto.randomUUID(),
    presetId: props.presetId,
    name: '',
    ip: '',
    createdAt: new Date().toISOString(),
    _key: keyCounter++,
  })
}

async function load() {
  if (!props.presetId) return
  loading.value = true
  try {
    const data = await api.getDenyAllowsByPreset(props.presetId)
    if (Array.isArray(data)) {
      items.value = data.map((i: any) => ({ ...i, _key: keyCounter++ }))
    } else {
      items.value = []
    }
  } catch (err: any) {
    toast.error('加载失败: ' + (err?.message || err))
  } finally {
    loading.value = false
  }
}

watch(() => props.presetId, () => { items.value = []; load() }, { immediate: true })

async function onSave() {
  loading.value = true
  saved.value = false
  try {
    const existing = await api.getDenyAllowsByPreset(props.presetId)
    if (Array.isArray(existing)) {
      for (const e of existing) {
        try { await api.deleteNginxDenyAllow(e.id) } catch {}
      }
    }
    for (const item of items.value) {
      if (item.name.trim() && item.ip.trim()) {
        await api.addNginxDenyAllow({
          id: item.id || crypto.randomUUID(),
          presetId: props.presetId,
          name: item.name.trim(),
          ip: item.ip,
          createdAt: new Date().toISOString(),
        })
      }
    }
    saved.value = true
    toast.success('已保存')
    setTimeout(() => { saved.value = false }, 3000)
  } catch (err: any) {
    toast.error('保存失败: ' + (err?.message || err))
  } finally {
    loading.value = false
  }
}

function onDelete(index: number) {
  items.value.splice(index, 1)
}
</script>
