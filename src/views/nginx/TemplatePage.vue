<template>
  <div>
    <!-- 工具栏 -->
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-base font-semibold m-0">配置模板</h3>
      <button @click="openAddDialog" class="btn btn-primary btn-sm">
        <SvgIcon name="plus" size="14" /> 新增模板
      </button>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="flex items-center justify-center py-8 text-base-content/50">
      <SvgIcon name="clock" size="14" /> 加载中...
    </div>

    <!-- 空状态 -->
    <div v-else-if="templates.length === 0" class="flex flex-col items-center justify-center py-8 text-base-content/50">
      <SvgIcon name="file" size="24" class="mb-2 opacity-50" />
      <p class="text-sm">暂无模板配置</p>
    </div>

    <!-- 表格 -->
    <div v-else class="overflow-x-auto">
      <table class="table table-zebra table-sm">
        <thead>
          <tr>
            <th>名称</th>
            <th>创建时间</th>
            <th class="w-28 text-center">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="tpl in templates" :key="tpl.id">
            <td class="font-medium">{{ tpl.name }}</td>
            <td class="text-xs text-base-content/50">{{ tpl.createdAt ? formatDate(tpl.createdAt) : '-' }}</td>
            <td class="text-center">
              <div class="flex items-center justify-center gap-1">
                <button @click="openEditDialog(tpl)" class="btn btn-ghost btn-xs" title="编辑">
                  <SvgIcon name="pencil" size="14" />
                </button>
                <button @click="onDeleteTemplate(tpl.id)" class="btn btn-ghost btn-xs btn-square text-error" title="删除">
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
      <div class="modal-box max-w-3xl max-h-[80vh] overflow-y-auto">
        <h3 class="font-bold text-lg">{{ editingTemplate ? '编辑模板' : '新增模板' }}</h3>

        <div class="flex flex-col gap-3 mt-4">
          <!-- 名称 -->
          <div class="flex flex-col gap-1">
            <label class="text-sm font-medium">名称</label>
            <input v-model="form.name" placeholder="例如：location-template" class="input input-bordered w-full" />
          </div>

          <!-- 内容 -->
          <div class="flex flex-col gap-1">
            <label class="text-sm font-medium">模板内容</label>
            <textarea
              v-model="form.content"
              placeholder="粘贴 Nginx 配置片段..."
              class="textarea textarea-bordered w-full font-mono text-sm leading-relaxed"
              rows="16"
            ></textarea>
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
const editingTemplate = ref<any>(null)
const api = getTauriAPI()

// 主数据
const templates = ref<any[]>([])

// 表单
const form = ref({
  id: '',
  presetId: '',
  name: '',
  content: '',
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

async function loadTemplates() {
  if (!props.presetId) return
  loading.value = true
  try {
    const result = await api.getTemplatesByPreset(props.presetId)
    templates.value = result?.data ?? result ?? []
  } catch (err: any) {
    toast.error('加载模板失败: ' + (err?.message || err))
  } finally {
    loading.value = false
  }
}

watch(() => props.presetId, () => { loadTemplates() }, { immediate: true })

function resetForm() {
  form.value = {
    id: crypto.randomUUID(),
    presetId: props.presetId,
    name: '',
    content: '',
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  }
}

function openAddDialog() {
  editingTemplate.value = null
  resetForm()
  showDialog.value = true
}

function openEditDialog(tpl: any) {
  editingTemplate.value = tpl
  form.value = { ...tpl }
  showDialog.value = true
}

function closeDialog() {
  showDialog.value = false
  editingTemplate.value = null
}

async function onSave() {
  try {
    const data = {
      ...form.value,
      updatedAt: new Date().toISOString(),
    }

    if (editingTemplate.value) {
      await api.updateNginxTemplate(data)
      const idx = templates.value.findIndex((t) => t.id === data.id)
      if (idx !== -1) templates.value[idx] = data
      toast.success('模板已更新')
    } else {
      const result = await api.addNginxTemplate(data)
      const saved = result?.data ?? result
      templates.value.push({ ...data, ...saved })
      toast.success('模板已添加')
    }

    closeDialog()
    await loadTemplates()
  } catch (err: any) {
    toast.error('保存失败: ' + (err?.message || err))
  }
}

async function onDeleteTemplate(id: string) {
  try {
    await api.deleteNginxTemplate(id)
    templates.value = templates.value.filter((t) => t.id !== id)
    toast.success('模板已删除')
  } catch (err: any) {
    toast.error('删除失败: ' + (err?.message || err))
  }
}
</script>
