<template>
  <div>
    <!-- 工具栏 -->
    <div class="flex items-center justify-between mb-3">
      <h3 class="text-base font-semibold m-0">SSL 证书管理</h3>
      <button @click="openAddDialog" class="btn btn-primary btn-sm">
        <SvgIcon name="plus" size="14" /> 新增证书
      </button>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="flex items-center justify-center py-8 text-base-content/50">
      <SvgIcon name="clock" size="14" /> 加载中...
    </div>

    <!-- 空状态 -->
    <div v-else-if="certs.length === 0" class="flex flex-col items-center justify-center py-8 text-base-content/50">
      <SvgIcon name="shield" size="24" class="mb-2 opacity-50" />
      <p class="text-sm">暂无证书配置</p>
    </div>

    <!-- 表格 -->
    <div v-else class="overflow-x-auto">
      <table class="table table-zebra table-xs">
        <thead>
          <tr>
            <th>名称</th>
            <th>域名</th>
            <th>PEM 路径</th>
            <th>Key 路径</th>
            <th>创建时间</th>
            <th class="w-20 text-center">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="cert in certs" :key="cert.id">
            <td class="font-medium">{{ cert.name }}</td>
            <td class="text-xs font-mono">{{ cert.domain || '-' }}</td>
            <td class="text-xs font-mono text-base-content/70 truncate max-w-[160px]" :title="cert.pem">{{ cert.pem || '-' }}</td>
            <td class="text-xs font-mono text-base-content/70 truncate max-w-[160px]" :title="cert.key">{{ cert.key || '-' }}</td>
            <td class="text-xs text-base-content/50">{{ cert.createdAt ? formatDate(cert.createdAt) : '-' }}</td>
            <td class="text-center">
              <div class="flex items-center justify-center gap-0.5">
                <button @click="openEditDialog(cert)" class="btn btn-ghost btn-xs btn-square" title="编辑">
                  <SvgIcon name="pencil" size="12" />
                </button>
                <button @click="onDeleteCert(cert.id)" class="btn btn-ghost btn-xs btn-square text-error" title="删除">
                  <SvgIcon name="trash" size="12" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 新增/编辑弹窗 -->
    <div v-if="showDialog" class="modal modal-open">
      <div class="modal-box relative max-w-lg">
        <button @click="closeDialog" class="absolute top-3 right-3 btn btn-ghost btn-sm btn-square rounded-full" title="关闭"><SvgIcon name="x" size="16" /></button>
        <h3 class="font-bold text-lg">{{ editingCert ? '编辑证书' : '新增证书' }}</h3>

        <div class="flex flex-col gap-2 mt-3">
          <!-- 名称 -->
          <div class="flex flex-col gap-1">
            <label class="text-xs font-medium text-base-content/80">名称</label>
            <input v-model="form.name" placeholder="例如：example-com-cert" class="input input-sm input-bordered w-full" />
          </div>

          <!-- 域名 -->
          <div class="flex flex-col gap-1">
            <label class="text-xs font-medium text-base-content/80">域名</label>
            <input v-model="form.domain" placeholder="example.com" class="input input-sm input-bordered w-full" />
          </div>

          <!-- PEM 路径 -->
          <div class="flex flex-col gap-1">
            <label class="text-xs font-medium text-base-content/80">PEM 路径</label>
            <input v-model="form.pem" placeholder="/etc/nginx/ssl/cert.pem" class="input input-sm input-bordered w-full" />
          </div>

          <!-- Key 路径 -->
          <div class="flex flex-col gap-1">
            <label class="text-xs font-medium text-base-content/80">Key 路径</label>
            <input v-model="form.key" placeholder="/etc/nginx/ssl/cert.key" class="input input-sm input-bordered w-full" />
          </div>
        </div>

        <div class="modal-action">
          <button @click="closeDialog" class="btn btn-ghost btn-sm">取消</button>
          <button @click="onSave" class="btn btn-primary btn-sm" :disabled="!form.name">保存</button>
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
const editingCert = ref<any>(null)
const api = getTauriAPI()

// 主数据
const certs = ref<any[]>([])

// 表单
const form = ref({
  id: '',
  presetId: '',
  name: '',
  domain: '',
  pem: '',
  key: '',
  createdAt: '',
  updatedAt: '',
})

function formatDate(dateStr: string) {
  try {
    const d = new Date(dateStr)
    return d.toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
  } catch {
    return dateStr
  }
}

async function loadCerts() {
  if (!props.presetId) return
  loading.value = true
  try {
    const result = await api.getCertsByPreset(props.presetId)
    certs.value = result?.data ?? result ?? []
  } catch (err: any) {
    toast.error('加载证书失败: ' + (err?.message || err))
  } finally {
    loading.value = false
  }
}

watch(() => props.presetId, () => { loadCerts() }, { immediate: true })

function resetForm() {
  form.value = {
    id: crypto.randomUUID(),
    presetId: props.presetId,
    name: '',
    domain: '',
    pem: '',
    key: '',
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  }
}

function openAddDialog() {
  editingCert.value = null
  resetForm()
  showDialog.value = true
}

function openEditDialog(cert: any) {
  editingCert.value = cert
  form.value = { ...cert }
  showDialog.value = true
}

function closeDialog() {
  showDialog.value = false
  editingCert.value = null
}

async function onSave() {
  try {
    const data = {
      ...form.value,
      updatedAt: new Date().toISOString(),
    }

    if (editingCert.value) {
      await api.updateNginxCert(data)
      const idx = certs.value.findIndex((c) => c.id === data.id)
      if (idx !== -1) certs.value[idx] = data
      toast.success('证书已更新')
    } else {
      const result = await api.addNginxCert(data)
      const saved = result?.data ?? result
      certs.value.push({ ...data, ...saved })
      toast.success('证书已添加')
    }

    closeDialog()
    await loadCerts()
  } catch (err: any) {
    toast.error('保存失败: ' + (err?.message || err))
  }
}

async function onDeleteCert(id: string) {
  try {
    await api.deleteNginxCert(id)
    certs.value = certs.value.filter((c) => c.id !== id)
    toast.success('证书已删除')
  } catch (err: any) {
    toast.error('删除失败: ' + (err?.message || err))
  }
}
</script>
