<template>
  <div>
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-base font-semibold m-0">额外参数</h3>
      <button @click="addRow" class="btn btn-primary btn-sm">
        <SvgIcon name="plus" size="14" /> 新增参数
      </button>
    </div>

    <div class="text-xs text-base-content/50 mb-3">
      添加自定义 nginx 参数，可附加到 Server / Location / Upstream。position=1 时参数出现在指令之前，=0 时追加到末尾。
    </div>

    <div v-if="loading" class="flex items-center justify-center py-8 text-base-content/50">
      <SvgIcon name="clock" size="14" /> 加载中...
    </div>

    <div v-else-if="items.length === 0" class="flex flex-col items-center justify-center py-8 text-base-content/50">
      <SvgIcon name="list" size="24" class="mb-2 opacity-50" />
      <p class="text-sm">暂无额外参数</p>
    </div>

    <div v-else class="overflow-x-auto">
      <table class="table table-zebra table-sm">
        <thead>
          <tr>
            <th>参数名</th>
            <th>参数值</th>
            <th class="w-20 text-center">位置</th>
            <th class="w-20 text-center">排序</th>
            <th class="w-24 text-center">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(item, index) in items" :key="item._key">
            <td><input v-model="item.name" placeholder="sendfile" class="input input-bordered input-sm w-full font-mono" /></td>
            <td><input v-model="item.value" placeholder="on" class="input input-bordered input-sm w-full font-mono" /></td>
            <td class="text-center">
              <select v-model.number="item.position" class="select select-bordered select-sm">
                <option :value="0">追加</option>
                <option :value="1">前置</option>
              </select>
            </td>
            <td class="text-center">
              <input v-model.number="item.sort" type="number" class="input input-bordered input-sm w-16 text-center" />
            </td>
            <td class="text-center">
              <button @click="items.splice(index, 1)" class="btn btn-ghost btn-xs btn-square text-error" title="删除">
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

interface ParamItem {
  id: string
  presetId: string
  serverId: string
  locationId: string
  upstreamId: string
  name: string
  value: string
  position: number
  templateValue: string
  sort: number
  createdAt: string
  _key: number
}

let keyCounter = 0
const items = ref<ParamItem[]>([])

function addRow() {
  items.value.push({
    id: crypto.randomUUID(),
    presetId: props.presetId,
    serverId: '',
    locationId: '',
    upstreamId: '',
    name: '',
    value: '',
    position: 0,
    templateValue: '',
    sort: items.value.length,
    createdAt: new Date().toISOString(),
    _key: keyCounter++,
  })
}

async function load() {
  if (!props.presetId) return
  loading.value = true
  try {
    const data = await api.getParamsByPreset(props.presetId)
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
    // Delete all existing params and re-insert
    const existing = await api.getParamsByPreset(props.presetId)
    if (Array.isArray(existing)) {
      for (const e of existing) {
        try { await api.deleteNginxParam(e.id) } catch {}
      }
    }
    for (const item of items.value) {
      if (item.name.trim()) {
        await api.addNginxParam({
          id: item.id || crypto.randomUUID(),
          presetId: props.presetId,
          serverId: '',
          locationId: '',
          upstreamId: '',
          name: item.name.trim(),
          value: item.value,
          position: item.position ?? 0,
          templateValue: '',
          sort: item.sort ?? 0,
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
</script>
