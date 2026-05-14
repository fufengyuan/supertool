<template>
  <div>
    <div class="flex items-center justify-between mb-3">
      <h3 class="text-base font-semibold m-0">全局基础参数</h3>
      <button @click="addRow" class="btn btn-primary btn-sm">
        <SvgIcon name="plus" size="14" /> 新增参数
      </button>
    </div>

    <div class="text-xs text-base-content/50 mb-2">
      此处添加任意 nginx 全局指令，例如：<code>sendfile on</code>、<code>tcp_nopush on</code>、<code>keepalive_timeout 65</code>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="flex items-center justify-center py-8 text-base-content/50">
      <SvgIcon name="clock" size="14" /> 加载中...
    </div>

    <!-- 空状态 -->
    <div v-else-if="settings.length === 0" class="flex flex-col items-center justify-center py-8 text-base-content/50">
      <SvgIcon name="tool" size="24" class="mb-2 opacity-50" />
      <p class="text-sm">暂无全局参数，点击上方按钮添加</p>
    </div>

    <!-- 参数表格 -->
    <div v-else class="overflow-x-auto">
      <table class="table table-zebra table-xs">
        <thead>
          <tr>
            <th class="w-8 text-center">#</th>
            <th class="w-1/3">参数名</th>
            <th>参数值</th>
            <th class="w-20 text-center">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(item, index) in settings" :key="item._key">
            <td class="text-center text-base-content/50 text-xs">{{ index + 1 }}</td>
            <td>
              <input
                v-model="item.name"
                placeholder="参数名 (如 worker_processes)"
                class="input input-bordered input-xs w-full font-mono"
                @input="markDirty"
              />
            </td>
            <td>
              <input
                v-model="item.value"
                placeholder="参数值 (如 auto)"
                class="input input-bordered input-xs w-full font-mono"
                @input="markDirty"
              />
            </td>
            <td class="text-center">
              <div class="flex items-center justify-center gap-0.5">
                <div class="flex flex-col gap-0">
                  <button
                    @click="moveUp(index)"
                    :disabled="index === 0"
                    class="btn btn-ghost btn-xs btn-square"
                    title="上移"
                  >
                    <SvgIcon name="chevronUp" size="10" />
                  </button>
                  <button
                    @click="moveDown(index)"
                    :disabled="index === settings.length - 1"
                    class="btn btn-ghost btn-xs btn-square"
                    title="下移"
                  >
                    <SvgIcon name="chevronDown" size="10" />
                  </button>
                </div>
                <button @click="removeRow(index)" class="btn btn-ghost btn-xs btn-square text-error" title="删除">
                  <SvgIcon name="trash" size="12" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 操作按钮 -->
    <div class="flex items-center gap-2 mt-3">
      <button @click="onSave" class="btn btn-primary btn-sm" :disabled="loading || !loaded">
        <SvgIcon name="check" size="12" /> 保存设置
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

const toast = useToast()
const loading = ref(false)
const loaded = ref(false)
const saved = ref(false)
const dirty = ref(false)
const api = getTauriAPI()

interface BasicItem {
  id: string
  presetId: string
  name: string
  value: string
  sort: number
  _key: number
}

let keyCounter = 0
const settings = ref<BasicItem[]>([])

function createItem(name = '', value = '', sort = 0): BasicItem {
  return {
    id: crypto.randomUUID(),
    presetId: props.presetId,
    name,
    value,
    sort,
    _key: keyCounter++,
  }
}

function addRow() {
  settings.value.push(createItem('', '', settings.value.length))
  dirty.value = true
}

function removeRow(index: number) {
  settings.value.splice(index, 1)
  updateSort()
  dirty.value = true
}

function moveUp(index: number) {
  if (index <= 0) return
  const arr = settings.value
  const temp = arr[index]
  arr[index] = arr[index - 1]
  arr[index - 1] = temp
  updateSort()
  dirty.value = true
}

function moveDown(index: number) {
  if (index >= settings.value.length - 1) return
  const arr = settings.value
  const temp = arr[index]
  arr[index] = arr[index + 1]
  arr[index + 1] = temp
  updateSort()
  dirty.value = true
}

function updateSort() {
  settings.value.forEach((item, i) => {
    item.sort = i
  })
}

function markDirty() {
  dirty.value = true
}

async function loadSettings() {
  if (!props.presetId) return
  loading.value = true
  try {
    const data = await api.getBasicSettings(props.presetId)
    if (data && Array.isArray(data)) {
      settings.value = data.map((item: any, i: number) => ({
        id: item.id || crypto.randomUUID(),
        presetId: props.presetId,
        name: item.name || '',
        value: item.value || '',
        sort: item.sort ?? i,
        _key: keyCounter++,
      }))
      updateSort()
    } else {
      settings.value = []
    }
    loaded.value = true
  } catch (err: any) {
    toast.error('加载基本设置失败: ' + (err?.message || err))
    settings.value = []
    loaded.value = true
  } finally {
    loading.value = false
  }
}

watch(() => props.presetId, () => {
  loaded.value = false
  dirty.value = false
  settings.value = []
  loadSettings()
}, { immediate: true })

async function onSave() {
  saved.value = false
  loading.value = true
  try {
    // Filter out empty rows
    const items = settings.value
      .filter(item => item.name.trim() !== '')
      .map(item => ({
        id: item.id,
        presetId: props.presetId,
        name: item.name.trim(),
        value: item.value,
        sort: item.sort,
        createdAt: new Date().toISOString(),
      }))
    await api.saveBasicSettings(props.presetId, items)
    dirty.value = false
    saved.value = true
    toast.success('基本设置已保存')
    setTimeout(() => { saved.value = false }, 3000)
  } catch (err: any) {
    toast.error('保存失败: ' + (err?.message || err))
  } finally {
    loading.value = false
  }
}
</script>
