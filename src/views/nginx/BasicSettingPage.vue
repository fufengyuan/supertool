<template>
  <div>
    <!-- 标题 -->
    <h3 class="text-base font-semibold mb-4">基本设置</h3>

    <!-- 加载中 -->
    <div v-if="loading" class="flex items-center justify-center py-8 text-base-content/50">
      <SvgIcon name="clock" size="14" /> 加载中...
    </div>

    <!-- 表单 -->
    <div v-else class="max-w-2xl">
      <div class="grid grid-cols-2 gap-x-6 gap-y-4">
        <!-- workerProcesses -->
        <div class="flex flex-col gap-1">
          <label class="text-sm font-medium">workerProcesses</label>
          <input v-model="form.workerProcesses" placeholder="auto" class="input input-bordered w-full" />
        </div>

        <!-- workerConnections -->
        <div class="flex flex-col gap-1">
          <label class="text-sm font-medium">workerConnections</label>
          <input v-model.number="form.workerConnections" type="number" placeholder="1024" class="input input-bordered w-full" />
        </div>

        <!-- errorLog -->
        <div class="flex flex-col gap-1">
          <label class="text-sm font-medium">错误日志路径</label>
          <input v-model="form.errorLog" placeholder="/var/log/nginx/error.log" class="input input-bordered w-full" />
        </div>

        <!-- errorLogLevel -->
        <div class="flex flex-col gap-1">
          <label class="text-sm font-medium">错误日志级别</label>
          <select v-model="form.errorLogLevel" class="select select-bordered w-full">
            <option value="debug">debug</option>
            <option value="info">info</option>
            <option value="notice">notice</option>
            <option value="warn">warn</option>
            <option value="error">error</option>
            <option value="crit">crit</option>
            <option value="alert">alert</option>
            <option value="emerg">emerg</option>
          </select>
        </div>

        <!-- pid -->
        <div class="flex flex-col gap-1">
          <label class="text-sm font-medium">PID 文件路径</label>
          <input v-model="form.pid" placeholder="/var/run/nginx.pid" class="input input-bordered w-full" />
        </div>
      </div>

      <!-- events -->
      <div class="flex flex-col gap-1 mt-4">
        <label class="text-sm font-medium">Events 自定义配置</label>
        <textarea
          v-model="form.events"
          placeholder="可在此填写自定义 events 块内容，例如：&#10;use epoll;&#10;multi_accept on;"
          class="textarea textarea-bordered w-full font-mono text-sm leading-relaxed"
          rows="6"
        ></textarea>
      </div>

      <!-- 操作按钮 -->
      <div class="flex items-center gap-3 mt-6">
        <button @click="onSave" class="btn btn-primary" :disabled="loading || !loaded">
          <SvgIcon name="check" size="14" /> 保存设置
        </button>
        <button v-if="loaded && !initialLoading" @click="resetToDefaults" class="btn btn-ghost">
          恢复默认
        </button>
        <span v-if="saved" class="text-sm text-success">已保存</span>
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
const initialLoading = ref(false)
const loaded = ref(false)
const saved = ref(false)
const api = getTauriAPI()

// 表单
const form = ref({
  id: '',
  presetId: '',
  workerProcesses: 'auto',
  workerConnections: 1024,
  errorLog: '/var/log/nginx/error.log',
  errorLogLevel: 'warn',
  pid: '/var/run/nginx.pid',
  events: '',
  createdAt: '',
  updatedAt: '',
})

function getDefaultForm() {
  return {
    id: crypto.randomUUID(),
    presetId: props.presetId,
    workerProcesses: 'auto',
    workerConnections: 1024,
    errorLog: '/var/log/nginx/error.log',
    errorLogLevel: 'warn',
    pid: '/var/run/nginx.pid',
    events: '',
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  }
}

function resetToDefaults() {
  form.value = getDefaultForm()
  saved.value = false
  toast.info('已恢复默认值')
}

async function loadSetting() {
  if (!props.presetId) return
  initialLoading.value = true
  loading.value = true
  try {
    const result = await api.getBasicSetting(props.presetId)
    const data = result?.data ?? result
    if (data) {
      form.value = { ...data }
    } else {
      // 首次使用，初始化为默认值
      form.value = getDefaultForm()
    }
    loaded.value = true
  } catch (err: any) {
    toast.error('加载基本设置失败: ' + (err?.message || err))
    form.value = getDefaultForm()
    loaded.value = true
  } finally {
    loading.value = false
    initialLoading.value = false
  }
}

watch(() => props.presetId, () => {
  loaded.value = false
  loadSetting()
}, { immediate: true })

async function onSave() {
  saved.value = false
  loading.value = true
  try {
    const data = {
      ...form.value,
      presetId: props.presetId,
      workerConnections: Number(form.value.workerConnections) || 1024,
      updatedAt: new Date().toISOString(),
    }
    await api.upsertBasicSetting(data)
    form.value = { ...data }
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
