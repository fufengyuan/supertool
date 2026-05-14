<template>
  <div>
    <!-- 工具栏 -->
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-base font-semibold m-0">Upstream 配置</h3>
      <button @click="openAddDialog" class="btn btn-primary btn-sm">
        <SvgIcon name="plus" size="14" /> 新增 Upstream
      </button>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="flex items-center justify-center py-8 text-base-content/50">
      <SvgIcon name="clock" size="14" /> 加载中...
    </div>

    <!-- 空状态 -->
    <div v-else-if="upstreams.length === 0" class="flex flex-col items-center justify-center py-8 text-base-content/50">
      <SvgIcon name="layers" size="24" class="mb-2 opacity-50" />
      <p class="text-sm">暂无 Upstream 配置</p>
    </div>

    <!-- 表格 -->
    <div v-else class="overflow-x-auto">
      <table class="table table-zebra table-sm">
        <thead>
          <tr>
            <th>代理类型</th>
            <th>名称</th>
            <th>策略</th>
            <th>描述</th>
            <th class="w-36 text-center">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="upstream in upstreams" :key="upstream.id">
            <td>
              <span class="badge badge-sm" :class="upstream.proxyType === 1 ? 'badge-info' : upstream.proxyType === 2 ? 'badge-warning' : 'badge-ghost'">
                {{ upstream.proxyType === 1 ? 'TCP' : upstream.proxyType === 2 ? 'UDP' : 'HTTP' }}
              </span>
            </td>
            <td class="font-medium">{{ upstream.name }}</td>
            <td>
              <span class="badge badge-sm badge-ghost">{{ strategyLabel(upstream.strategy) }}</span>
            </td>
            <td class="text-base-content/60 text-sm">{{ upstream.descr || '-' }}</td>
            <td class="text-center">
              <div class="flex items-center justify-center gap-1">
                <button @click="openEditDialog(upstream)" class="btn btn-ghost btn-xs" title="编辑">
                  <SvgIcon name="pencil" size="14" />
                </button>
                <button @click="onCloneUpstream(upstream)" class="btn btn-ghost btn-xs" title="克隆">
                  <SvgIcon name="copy" size="14" />
                </button>
                <button @click="onDeleteUpstream(upstream.id)" class="btn btn-ghost btn-xs btn-square text-error" title="删除">
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
      <div class="modal-box max-w-7xl max-h-[85vh] overflow-y-auto">
        <h3 class="font-bold text-lg">{{ editingUpstream ? '编辑 Upstream' : '新增 Upstream' }}</h3>

        <div class="grid grid-cols-2 gap-x-8 gap-y-4 mt-4">
          <!-- 名称 -->
          <div class="flex flex-col gap-1">
            <label class="text-sm font-medium">名称</label>
            <input v-model="form.name" placeholder="例如：backend-api" class="input input-bordered w-full" />
          </div>

          <!-- 代理类型 -->
          <div class="flex flex-col gap-1">
            <label class="text-sm font-medium">代理类型</label>
            <select v-model="form.proxyType" class="select select-bordered w-full">
              <option :value="0">HTTP</option>
              <option :value="1">TCP</option>
              <option :value="2">UDP</option>
            </select>
          </div>

          <!-- 策略 -->
          <div class="flex flex-col gap-1">
            <label class="text-sm font-medium">负载均衡策略</label>
            <select v-model="form.strategy" class="select select-bordered w-full">
              <option value="polling">轮询 (polling)</option>
              <option value="ip_hash">IP Hash</option>
              <option value="least_conn">最小连接 (least_conn)</option>
              <option value="random">随机 (random)</option>
            </select>
          </div>

          <!-- 描述 -->
          <div class="flex flex-col gap-1">
            <label class="text-sm font-medium">描述</label>
            <input v-model="form.descr" placeholder="可选描述" class="input input-bordered w-full" />
          </div>
        </div>

        <!-- 隐藏 paramJson -->
        <textarea v-model="form.paramJson" class="hidden"></textarea>

        <!-- 上游服务器列表 -->
        <div class="mt-6">
          <div class="flex items-center justify-between mb-3">
            <span class="text-sm font-medium">上游服务器</span>
            <button @click="onAddUpstreamServer" class="btn btn-primary btn-xs">
              <SvgIcon name="plus" size="12" /> 新增服务器
            </button>
          </div>

          <div v-if="upstreamServers.length === 0" class="text-center py-4 text-base-content/50 text-sm">
            暂无上游服务器，请点击上方按钮添加
          </div>

          <div v-else class="overflow-x-auto">
            <table class="table table-zebra table-xs">
              <thead>
                <tr>
                  <th>地址</th>
                  <th>端口</th>
                  <th>权重</th>
                  <th>最大失败</th>
                  <th>超时</th>
                  <th>最大连接</th>
                  <th class="text-center">备用</th>
                  <th class="text-center">下线</th>
                  <th class="w-16 text-center">操作</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(svr, idx) in upstreamServers" :key="svr._key || idx">
                  <td>
                    <input v-model="svr.address" placeholder="127.0.0.1" class="input input-bordered input-xs w-28" />
                  </td>
                  <td>
                    <input v-model.number="svr.port" type="number" placeholder="80" class="input input-bordered input-xs w-16" />
                  </td>
                  <td>
                    <input v-model.number="svr.weight" type="number" placeholder="1" class="input input-bordered input-xs w-14" />
                  </td>
                  <td>
                    <input v-model.number="svr.maxFails" type="number" placeholder="3" class="input input-bordered input-xs w-14" />
                  </td>
                  <td>
                    <input v-model="svr.failTimeout" placeholder="10s" class="input input-bordered input-xs w-16" />
                  </td>
                  <td>
                    <input v-model.number="svr.maxConns" type="number" placeholder="0" class="input input-bordered input-xs w-14" />
                  </td>
                  <td class="text-center">
                    <input type="checkbox" v-model="svr.backup" class="checkbox checkbox-xs" />
                  </td>
                  <td class="text-center">
                    <input type="checkbox" v-model="svr.down" class="checkbox checkbox-xs" />
                  </td>
                  <td class="text-center">
                    <button @click="upstreamServers.splice(idx, 1)" class="btn btn-ghost btn-xs btn-square text-error" title="删除">
                      <SvgIcon name="x" size="12" />
                    </button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <div class="modal-action">
          <button @click="closeDialog" class="btn btn-ghost">取消</button>
          <button @click="onSave" class="btn btn-primary" :disabled="!form.name">保存</button>
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
const editingUpstream = ref<any>(null)
const api = getTauriAPI()

// 主数据
const upstreams = ref<any[]>([])

// 表单
const form = ref({
  id: '',
  presetId: '',
  name: '',
  proxyType: 0,
  strategy: 'polling',
  descr: '',
  paramJson: '',
  createdAt: '',
  updatedAt: '',
})

// 子表 - 上游服务器
const upstreamServers = ref<any[]>([])

function strategyLabel(strategy: string) {
  const map: Record<string, string> = {
    polling: '轮询',
    ip_hash: 'IP Hash',
    least_conn: '最小连接',
    random: '随机',
  }
  return map[strategy] || strategy || '轮询'
}

async function loadUpstreams() {
  if (!props.presetId) return
  loading.value = true
  try {
    const result = await api.getUpstreamsByPreset(props.presetId)
    upstreams.value = result?.data ?? result ?? []
  } catch (err: any) {
    toast.error('加载 Upstream 失败: ' + (err?.message || err))
  } finally {
    loading.value = false
  }
}

watch(() => props.presetId, () => { loadUpstreams() }, { immediate: true })

function resetForm() {
  form.value = {
    id: crypto.randomUUID(),
    presetId: props.presetId,
    name: '',
    proxyType: 0,
    strategy: 'polling',
    descr: '',
    paramJson: '',
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  }
  upstreamServers.value = []
}

function openAddDialog() {
  editingUpstream.value = null
  resetForm()
  showDialog.value = true
}

async function openEditDialog(upstream: any) {
  editingUpstream.value = upstream
  form.value = { ...upstream }
  showDialog.value = true
  // 加载关联的上游服务器
  try {
    const result = await api.getUpstreamServers(upstream.id)
    upstreamServers.value = (result?.data ?? result ?? []).map((s: any) => ({ ...s, _key: crypto.randomUUID() }))
  } catch (err: any) {
    toast.error('加载上游服务器失败: ' + (err?.message || err))
    upstreamServers.value = []
  }
}

function closeDialog() {
  showDialog.value = false
  editingUpstream.value = null
}

function onAddUpstreamServer() {
  upstreamServers.value.push({
    id: crypto.randomUUID(),
    upstreamId: form.value.id,
    address: '',
    port: 80,
    weight: 1,
    maxFails: 3,
    failTimeout: '10s',
    maxConns: 0,
    backup: false,
    down: false,
    _key: crypto.randomUUID(),
  })
}

async function onSave() {
  try {
    if (editingUpstream.value) {
      // 更新
      const updated = { ...form.value, updatedAt: new Date().toISOString() }
      await api.updateNginxUpstream(updated)
      // 替换列表中的记录
      const idx = upstreams.value.findIndex(u => u.id === updated.id)
      if (idx !== -1) upstreams.value[idx] = updated
      toast.success('Upstream 已更新')
    } else {
      // 新增
      const result = await api.addNginxUpstream(form.value)
      const saved = result?.data ?? result
      upstreams.value.push({ ...form.value, ...saved })
      toast.success('Upstream 已添加')
    }

    // 保存上游服务器
    for (const svr of upstreamServers.value) {
      svr.upstreamId = form.value.id
      if (svr._key && !svr.id) {
        // 新增
        delete svr._key
        try {
          await api.addNginxUpstreamServer(svr)
        } catch (_) { /* server may already exist */ }
      } else if (svr.id && !svr._deleted) {
        try {
          await api.updateNginxUpstreamServer(svr)
        } catch (_) { /* ignore */ }
      }
    }

    closeDialog()
    await loadUpstreams()
  } catch (err: any) {
    toast.error('保存失败: ' + (err?.message || err))
  }
}

async function onCloneUpstream(upstream: any) {
  const clone = {
    ...upstream,
    id: crypto.randomUUID(),
    name: upstream.name + ' (副本)',
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  }
  try {
    await api.addNginxUpstream(clone)
    toast.success('Upstream 已克隆')
    await loadUpstreams()
  } catch (err: any) {
    toast.error('克隆失败: ' + (err?.message || err))
  }
}

async function onDeleteUpstream(id: string) {
  try {
    await api.deleteNginxUpstream(id)
    upstreams.value = upstreams.value.filter(u => u.id !== id)
    toast.success('Upstream 已删除')
  } catch (err: any) {
    toast.error('删除失败: ' + (err?.message || err))
  }
}
</script>
