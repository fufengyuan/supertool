<template>
  <div>
    <div class="flex items-center justify-between mb-3">
      <h3 class="text-base font-semibold m-0">密码文件 (auth_basic)</h3>
      <button @click="addRow" class="btn btn-primary btn-sm">
        <SvgIcon name="plus" size="14" /> 新增密码
      </button>
    </div>

    <div v-if="loading" class="flex items-center justify-center py-8 text-base-content/50">
      <SvgIcon name="clock" size="14" /> 加载中...
    </div>

    <div v-else-if="items.length === 0" class="flex flex-col items-center justify-center py-8 text-base-content/50">
      <SvgIcon name="lock" size="24" class="mb-2 opacity-50" />
      <p class="text-sm">暂无密码文件</p>
    </div>

    <div v-else class="overflow-x-auto">
      <table class="table table-zebra table-xs">
        <thead>
          <tr>
            <th class="w-28">用户名</th>
            <th class="w-28">密码</th>
            <th>描述</th>
            <th>文件路径</th>
            <th class="w-16 text-center">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(item, index) in items" :key="item._key">
            <td>
              <input v-model="item.name" placeholder="admin" class="input input-bordered input-xs w-full font-mono" />
            </td>
            <td>
              <input v-model="item.pass" type="password" placeholder="密码" class="input input-bordered input-xs w-full" />
            </td>
            <td>
              <input v-model="item.descr" placeholder="Restricted Area" class="input input-bordered input-xs w-full" />
            </td>
            <td>
              <input v-model="item.path" placeholder="/etc/nginx/.htpasswd" class="input input-bordered input-xs w-full font-mono" />
            </td>
            <td class="text-center">
              <div class="flex items-center justify-center gap-0.5">
                <button @click="items.splice(index, 1)" class="btn btn-ghost btn-xs btn-square text-error" title="删除">
                  <SvgIcon name="trash" size="12" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div class="flex items-center gap-2 mt-3">
      <button @click="onSave" class="btn btn-primary btn-sm" :disabled="loading">
        <SvgIcon name="check" size="12" /> 保存
      </button>
      <span v-if="saved" class="text-xs text-success">已保存</span>
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

interface PwItem {
  id: string
  presetId: string
  name: string
  pass: string
  descr: string
  path: string
  createdAt: string
  _key: number
}

let keyCounter = 0
const items = ref<PwItem[]>([])

function addRow() {
  items.value.push({
    id: crypto.randomUUID(),
    presetId: props.presetId,
    name: '',
    pass: '',
    descr: '',
    path: '',
    createdAt: new Date().toISOString(),
    _key: keyCounter++,
  })
}

async function load() {
  if (!props.presetId) {return}
  loading.value = true
  try {
    const data = await api.getPasswordsByPreset(props.presetId)
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
    const existing = await api.getPasswordsByPreset(props.presetId)
    if (Array.isArray(existing)) {
      for (const e of existing) {
        try { await api.deleteNginxPassword(e.id) } catch {}
      }
    }
    for (const item of items.value) {
      if (item.name.trim()) {
        await api.addNginxPassword({
          id: item.id || crypto.randomUUID(),
          presetId: props.presetId,
          name: item.name.trim(),
          pass: item.pass,
          descr: item.descr,
          path: item.path,
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
