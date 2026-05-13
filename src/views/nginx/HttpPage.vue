<template>
  <div>
    <!-- 工具栏 -->
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-base font-semibold m-0">HTTP 全局参数</h3>
      <button @click="onAddParam" class="btn btn-primary btn-sm">
        <SvgIcon name="plus" size="14" /> 新增参数
      </button>
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
      <table class="table table-zebra table-sm">
        <thead>
          <tr>
            <th class="w-16 text-center">排序</th>
            <th class="w-16 text-center">启用</th>
            <th>参数名</th>
            <th>参数值</th>
            <th class="w-32 text-center">操作</th>
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
                  <SvgIcon name="chevronUp" size="12" />
                </button>
                <span class="text-xs text-base-content/60 w-4 text-center">{{ param.sort ?? index + 1 }}</span>
                <button
                  @click="moveDown(index)"
                  :disabled="index === params.length - 1"
                  class="btn btn-ghost btn-xs btn-square"
                  title="下移"
                >
                  <SvgIcon name="chevronDown" size="12" />
                </button>
              </div>
            </td>
            <td class="text-center">
              <input
                type="checkbox"
                :checked="param.enabled !== false"
                @change="toggleEnabled(param)"
                class="checkbox checkbox-sm"
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
              <div class="flex items-center justify-center gap-1">
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

const api = getTauriAPI()

// 加载数据
async function loadParams() {
  if (!props.presetId) return
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

// 新增参数
async function onAddParam() {
  const newParam = {
    id: crypto.randomUUID(),
    presetId: props.presetId,
    name: '',
    value: '',
    sort: params.value.length + 1,
    enabled: true,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  }
  try {
    const result = await api.addNginxHttpParam(newParam)
    const saved = result?.data ?? result
    params.value.push({ ...newParam, ...saved })
    toast.success('参数已添加')
  } catch (err: any) {
    toast.error('添加参数失败: ' + (err?.message || err))
  }
}

// 更新参数
async function onUpdateParam(param: any) {
  param.updatedAt = new Date().toISOString()
  try {
    await api.updateNginxHttpParam(param)
    toast.success('参数已更新')
  } catch (err: any) {
    toast.error('更新参数失败: ' + (err?.message || err))
  }
}

// 切换启用状态
async function toggleEnabled(param: any) {
  param.enabled = !param.enabled
  await onUpdateParam(param)
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
  if (index <= 0) return
  swapParams(index, index - 1)
}

async function moveDown(index: number) {
  if (index >= params.value.length - 1) return
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
