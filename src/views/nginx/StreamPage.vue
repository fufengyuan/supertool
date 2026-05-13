<template>
  <div>
    <!-- 工具栏 -->
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-base font-semibold m-0">Stream 配置</h3>
      <button @click="openAddDialog" class="btn btn-primary btn-sm">
        <SvgIcon name="plus" size="14" /> 新增 Stream
      </button>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="flex items-center justify-center py-8 text-base-content/50">
      <SvgIcon name="clock" size="14" /> 加载中...
    </div>

    <!-- 空状态 -->
    <div v-else-if="streams.length === 0" class="flex flex-col items-center justify-center py-8 text-base-content/50">
      <SvgIcon name="activity" size="24" class="mb-2 opacity-50" />
      <p class="text-sm">暂无 Stream 配置</p>
    </div>

    <!-- 表格 -->
    <div v-else class="overflow-x-auto">
      <table class="table table-zebra table-sm">
        <thead>
          <tr>
            <th>监听</th>
            <th>协议</th>
            <th>代理目标</th>
            <th class="text-center">SSL</th>
            <th class="w-20 text-center">启用</th>
            <th class="w-32 text-center">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="stream in streams" :key="stream.id">
            <td class="font-mono text-sm">{{ stream.listen || '-' }}</td>
            <td>
              <span class="badge badge-sm" :class="stream.protocol === 'UDP' ? 'badge-warning' : 'badge-info'">
                {{ stream.protocol || 'TCP' }}
              </span>
            </td>
            <td class="text-sm">{{ stream.proxyPass || stream.proxyUpstreamId || '-' }}</td>
            <td class="text-center">
              <span v-if="stream.ssl" class="badge badge-sm badge-success">SSL</span>
              <span v-else class="badge badge-sm badge-ghost">否</span>
            </td>
            <td class="text-center">
              <input
                type="checkbox"
                :checked="stream.enabled !== false"
                @change="toggleEnabled(stream)"
                class="checkbox checkbox-sm"
              />
            </td>
            <td class="text-center">
              <div class="flex items-center justify-center gap-1">
                <button @click="openEditDialog(stream)" class="btn btn-ghost btn-xs" title="编辑">
                  <SvgIcon name="pencil" size="14" />
                </button>
                <button @click="onDeleteStream(stream.id)" class="btn btn-ghost btn-xs btn-square text-error" title="删除">
                  <SvgIcon name="trash" size="14" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 新增/编辑弹窗 -->
    <div v-if="showDialog" class="modal modal-open" @click.self="closeDialog">
      <div class="modal-box max-w-2xl">
        <h3 class="font-bold text-lg">{{ editingStream ? '编辑 Stream' : '新增 Stream' }}</h3>

        <div class="grid grid-cols-2 gap-x-4 gap-y-3 mt-4">
          <!-- 监听地址 -->
          <div class="flex flex-col gap-1">
            <label class="text-sm font-medium">监听地址</label>
            <input v-model="form.listen" placeholder="0.0.0.0:80" class="input input-bordered w-full" />
          </div>

          <!-- 协议 -->
          <div class="flex flex-col gap-1">
            <label class="text-sm font-medium">协议</label>
            <select v-model="form.protocol" class="select select-bordered w-full">
              <option value="TCP">TCP</option>
              <option value="UDP">UDP</option>
            </select>
          </div>

          <!-- 代理目标 -->
          <div class="flex flex-col gap-1">
            <label class="text-sm font-medium">代理目标 (proxyPass)</label>
            <input v-model="form.proxyPass" placeholder="例如：127.0.0.1:3000" class="input input-bordered w-full" />
          </div>

          <!-- 代理 Upstream ID -->
          <div class="flex flex-col gap-1">
            <label class="text-sm font-medium">代理 Upstream ID</label>
            <input v-model="form.proxyUpstreamId" placeholder="关联的 upstream ID" class="input input-bordered w-full" />
          </div>

          <!-- SSL -->
          <div class="flex flex-col gap-1">
            <label class="text-sm font-medium">SSL</label>
            <label class="flex items-center gap-2 text-sm cursor-pointer mt-1">
              <input type="checkbox" v-model="form.ssl" class="checkbox checkbox-sm" />
              开启 SSL
            </label>
          </div>

          <!-- certId (仅 SSL 开启时显示) -->
          <div v-if="form.ssl" class="flex flex-col gap-1">
            <label class="text-sm font-medium">证书 ID</label>
            <input v-model="form.certId" placeholder="证书 ID" class="input input-bordered w-full" />
          </div>

          <!-- 描述 -->
          <div class="flex flex-col gap-1">
            <label class="text-sm font-medium">描述</label>
            <input v-model="form.descr" placeholder="可选描述" class="input input-bordered w-full" />
          </div>

          <!-- 启用 -->
          <div class="flex flex-col gap-1">
            <label class="text-sm font-medium">启用</label>
            <label class="flex items-center gap-2 text-sm cursor-pointer mt-1">
              <input type="checkbox" v-model="form.enabled" class="checkbox checkbox-sm" />
              启用
            </label>
          </div>
        </div>

        <!-- 隐藏 paramJson -->
        <textarea v-model="form.paramJson" class="hidden"></textarea>

        <div class="modal-action">
          <button @click="closeDialog" class="btn btn-ghost">取消</button>
          <button @click="onSave" class="btn btn-primary" :disabled="!form.listen">保存</button>
        </div>
      </div>
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
const showDialog = ref(false)
const editingStream = ref<any>(null)
const api = getTauriAPI()

// 主数据
const streams = ref<any[]>([])

// 表单
const form = ref({
  id: '',
  presetId: '',
  listen: '0.0.0.0:80',
  protocol: 'TCP',
  proxyPass: '',
  proxyUpstreamId: '',
  ssl: false,
  certId: '',
  descr: '',
  enabled: true,
  paramJson: '',
  createdAt: '',
  updatedAt: '',
})

async function loadStreams() {
  if (!props.presetId) return
  loading.value = true
  try {
    const result = await api.getStreamsByPreset(props.presetId)
    streams.value = result?.data ?? result ?? []
  } catch (err: any) {
    toast.error('加载 Stream 失败: ' + (err?.message || err))
  } finally {
    loading.value = false
  }
}

watch(() => props.presetId, () => { loadStreams() }, { immediate: true })

function resetForm() {
  form.value = {
    id: crypto.randomUUID(),
    presetId: props.presetId,
    listen: '0.0.0.0:80',
    protocol: 'TCP',
    proxyPass: '',
    proxyUpstreamId: '',
    ssl: false,
    certId: '',
    descr: '',
    enabled: true,
    paramJson: '',
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  }
}

function openAddDialog() {
  editingStream.value = null
  resetForm()
  showDialog.value = true
}

function openEditDialog(stream: any) {
  editingStream.value = stream
  form.value = { ...stream }
  showDialog.value = true
}

function closeDialog() {
  showDialog.value = false
  editingStream.value = null
}

async function onSave() {
  try {
    const data = {
      ...form.value,
      updatedAt: new Date().toISOString(),
    }

    if (editingStream.value) {
      await api.updateNginxStream(data)
      const idx = streams.value.findIndex((s) => s.id === data.id)
      if (idx !== -1) streams.value[idx] = data
      toast.success('Stream 已更新')
    } else {
      const result = await api.addNginxStream(data)
      const saved = result?.data ?? result
      streams.value.push({ ...data, ...saved })
      toast.success('Stream 已添加')
    }

    closeDialog()
    await loadStreams()
  } catch (err: any) {
    toast.error('保存失败: ' + (err?.message || err))
  }
}

async function onDeleteStream(id: string) {
  try {
    await api.deleteNginxStream(id)
    streams.value = streams.value.filter((s) => s.id !== id)
    toast.success('Stream 已删除')
  } catch (err: any) {
    toast.error('删除失败: ' + (err?.message || err))
  }
}

async function toggleEnabled(stream: any) {
  stream.enabled = !stream.enabled
  stream.updatedAt = new Date().toISOString()
  try {
    await api.updateNginxStream(stream)
  } catch (err: any) {
    toast.error('更新失败: ' + (err?.message || err))
    stream.enabled = !stream.enabled
  }
}
</script>
