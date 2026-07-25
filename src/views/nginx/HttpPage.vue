<template>
  <div>
    <!-- 工具栏 -->
    <div class="flex items-center justify-between mb-3">
      <h3 class="text-base font-semibold m-0">HTTP 全局参数</h3>
      <button @click="onAddParam" class="btn btn-primary btn-sm">
        <SvgIcon name="plus" size="14" /> 新增参数
      </button>
    </div>

    <!-- 常用参数快捷设置 -->
    <div class="bg-base-200/50 border border-base-content/10 rounded-lg mb-3 overflow-hidden">
      <div
        class="flex items-center justify-between px-3 py-2 cursor-pointer select-none hover:bg-base-200"
        @click="showGuide = !showGuide"
      >
        <span class="text-xs font-semibold text-base-content/80 flex items-center gap-1">
          <SvgIcon name="settings" size="12" /> 常用参数快捷设置
        </span>
        <span class="text-xs text-base-content/50">{{ showGuide ? '收起' : '展开' }}</span>
      </div>
      <div v-if="showGuide" class="px-3 pb-3 grid grid-cols-2 gap-2">
        <div v-for="item in commonParams" :key="item.key" class="flex items-center gap-2 px-2 py-1.5 rounded-lg bg-base-100 border border-base-content/5">
          <input
            type="checkbox"
            :checked="isParamEnabled(item.key)"
            @change="toggleCommonParam(item)"
            class="checkbox checkbox-xs"
          />
          <span class="text-xs font-mono text-base-content/70 min-w-[100px]">{{ item.key }}</span>
          <input
            v-if="item.hasValue !== false"
            :value="getParamValue(item.key)"
            @change="updateCommonParamValue(item.key, ($event.target as HTMLInputElement).value)"
            class="input input-xs input-bordered w-24 font-mono"
            :placeholder="item.defaultValue"
          />
          <span v-else class="text-xs text-base-content/40">开/关</span>
        </div>
      </div>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="flex items-center justify-center py-8 text-base-content/50">
      <SvgIcon name="clock" size="14" /> 加载中...
    </div>

    <!-- 空状态 -->
    <div v-else-if="params.length === 0" class="flex flex-col items-center justify-center py-8 text-base-content/50">
      <SvgIcon name="file" size="24" class="mb-2 opacity-50" />
      <p class="text-sm">暂无 HTTP 参数</p>
    </div>

    <!-- 参数表格 -->
    <div v-else class="overflow-x-auto">
      <table class="table table-zebra table-xs">
        <thead>
          <tr>
            <th class="w-12 text-center">排序</th>
            <th class="w-12 text-center">启用</th>
            <th>参数名</th>
            <th>参数值</th>
            <th class="w-24 text-center">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(param, index) in params" :key="param.id">
            <td class="text-center">
              <div class="flex items-center justify-center gap-0.5">
                <button
                  @click="moveUp(index)"
                  :disabled="index === 0"
                  class="btn btn-ghost btn-xs btn-square"
                  title="上移"
                >
                  <SvgIcon name="chevronUp" size="10" />
                </button>
                <span class="text-xs text-base-content/60 w-3 text-center">{{ param.sort ?? index + 1 }}</span>
                <button
                  @click="moveDown(index)"
                  :disabled="index === params.length - 1"
                  class="btn btn-ghost btn-xs btn-square"
                  title="下移"
                >
                  <SvgIcon name="chevronDown" size="10" />
                </button>
              </div>
            </td>
            <td class="text-center">
              <input
                type="checkbox"
                :checked="param.enabled !== false"
                @change="toggleEnabled(param)"
                class="checkbox checkbox-xs"
              />
            </td>
            <td>
              <input
                v-model="param.name"
                @change="onUpdateParam(param)"
                class="input input-bordered input-xs w-full"
                placeholder="参数名"
              />
            </td>
            <td>
              <input
                v-model="param.value"
                @change="onUpdateParam(param)"
                class="input input-bordered input-xs w-full"
                placeholder="参数值"
              />
            </td>
            <td class="text-center">
              <div class="flex items-center justify-center gap-0.5">
                <button
                  @click="onDeleteParam(param.id)"
                  class="btn btn-ghost btn-xs btn-square text-error"
                  title="删除"
                >
                  <SvgIcon name="trash" size="14" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
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
const params = ref<any[]>([])
const showGuide = ref(false)

interface CommonParam {
  key: string
  label: string
  defaultValue: string
  hasValue?: boolean
}

const commonParams: CommonParam[] = [
  { key: 'keepalive_timeout', label: 'keepalive_timeout', defaultValue: '65s' },
  { key: 'client_max_body_size', label: 'client_max_body_size', defaultValue: '100m' },
  { key: 'client_header_buffer_size', label: 'client_header_buffer_size', defaultValue: '32k' },
  { key: 'sendfile', label: 'sendfile', defaultValue: 'on', hasValue: false },
  { key: 'gzip', label: 'gzip', defaultValue: 'on', hasValue: false },
  { key: 'gzip_min_length', label: 'gzip_min_length', defaultValue: '1k' },
  { key: 'gzip_types', label: 'gzip_types', defaultValue: 'text/plain application/javascript application/x-javascript text/css application/xml text/javascript application/json' },
]

function findParam(key: string) {
  return params.value.find(p => p.name === key)
}

function isParamEnabled(key: string): boolean {
  const p = findParam(key)
  return p ? p.enabled !== false : false
}

function getParamValue(key: string): string {
  const p = findParam(key)
  return p ? p.value || '' : ''
}

async function toggleCommonParam(item: CommonParam) {
  const existing = findParam(item.key)
  if (existing) {
    if (existing.enabled !== false) {
      existing.enabled = false
    } else {
      existing.enabled = true
    }
    await onUpdateParam(existing)
  } else {
    // Add new param
    const newParam = {
      id: crypto.randomUUID(),
      presetId: props.presetId,
      name: item.key,
      value: item.hasValue !== false ? item.defaultValue : '',
      sort: params.value.length + 1,
      enabled: true,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    }
    try {
      const result = await api.addNginxHttpParam(newParam)
      const saved = result?.data ?? result
      params.value.push({ ...newParam, ...saved })
      toast.success(`已添加 ${item.key}`)
    } catch (err: any) {
      toast.error('添加参数失败: ' + (err?.message || err))
    }
  }
}

async function updateCommonParamValue(key: string, value: string) {
  const p = findParam(key)
  if (p) {
    p.value = value
    await onUpdateParam(p)
  }
}

const api = getTauriAPI()

// 加载数据
async function loadParams() {
  if (!props.presetId) {return}
  loading.value = true
  try {
    const result = await api.getHttpParamsByPreset(props.presetId)
    params.value = (result?.data ?? result ?? [])
      .sort((a: any, b: any) => (a.sort ?? 999) - (b.sort ?? 999))
  } catch (err: any) {
    toast.error('加载 HTTP 参数失败: ' + (err?.message || err))
  } finally {
    loading.value = false
  }
}

watch(() => props.presetId, () => { loadParams() }, { immediate: true })

// 新增参数 — 仅加入本地数组，失焦时统一保存
function onAddParam() {
  params.value.push({
    _key: crypto.randomUUID(),
    _isNew: true,
    presetId: props.presetId,
    name: 'new_param',
    value: '',
    sort: params.value.length + 1,
    enabled: true,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  })
}

// 更新参数（新增项首次保存走 add，已有项走 update）
async function onUpdateParam(param: any) {
  param.updatedAt = new Date().toISOString()
  try {
    if (param._isNew) {
      const { _key, _isNew, ...payload } = param
      void _key; void _isNew
      const result = await api.addNginxHttpParam(payload)
      const saved = result?.data ?? result
      if (saved?.id) { param.id = saved.id }
      param._isNew = false
    } else {
      await api.updateNginxHttpParam(param)
    }
  } catch (err: any) {
    toast.error('保存参数失败: ' + (err?.message || err))
  }
}

// 切换启用状态
async function toggleEnabled(param: any) {
  param.enabled = !param.enabled
  try {
    await api.updateNginxHttpParam(param)
  } catch (err: any) {
    // 失败时回滚 UI 状态
    param.enabled = !param.enabled
    toast.error('更新失败: ' + (err?.message || err))
  }
}

// 删除参数
async function onDeleteParam(id: string) {
  try {
    await api.deleteNginxHttpParam(id)
    params.value = params.value.filter(p => p.id !== id)
    toast.success('参数已删除')
  } catch (err: any) {
    toast.error('删除参数失败: ' + (err?.message || err))
  }
}

// 上移/下移
async function moveUp(index: number) {
  if (index <= 0) {return}
  swapParams(index, index - 1)
}

async function moveDown(index: number) {
  if (index >= params.value.length - 1) {return}
  swapParams(index, index + 1)
}

async function swapParams(i: number, j: number) {
  const arr = params.value
  const temp = arr[i].sort
  arr[i].sort = arr[j].sort
  arr[j].sort = temp
  ;[arr[i], arr[j]] = [arr[j], arr[i]]
  // 触发 reactivity
  params.value = [...arr]
  try {
    await Promise.all([
      api.updateNginxHttpParam(arr[i]),
      api.updateNginxHttpParam(arr[j]),
    ])
  } catch (err: any) {
    toast.error('排序更新失败: ' + (err?.message || err))
    await loadParams()
  }
}
</script>
