<template>
  <div class="h-full flex flex-col">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-2 border-b border-base-content/10">
      <h1 class="text-sm font-medium">定时任务</h1>
      <div class="flex items-center gap-2">
        <button class="btn btn-sm btn-ghost" @click="refresh" :disabled="loading">
          <SvgIcon name="refresh" size="14" />
        </button>
        <button class="btn btn-sm btn-primary" @click="openCreateModal">
          <IconPlus size="14" class="mr-0.5" />
          新建任务
        </button>
      </div>
    </div>

    <!-- Error message -->
    <div v-if="errorMsg" class="px-4 py-2 text-xs text-error bg-error/5 border-b border-error/10 flex items-center gap-2">
      <IconAlertCircle size="14" class="shrink-0" />
      <span class="flex-1">{{ errorMsg }}</span>
      <button class="btn btn-ghost btn-xs px-1" @click="errorMsg = ''"><SvgIcon name="x" size="12" /></button>
    </div>

    <!-- Success message -->
    <div v-if="successMsg" class="px-4 py-2 text-xs text-success bg-success/5 border-b border-success/10 flex items-center gap-2">
      <IconCircleCheck size="14" class="shrink-0" />
      <span class="flex-1">{{ successMsg }}</span>
      <button class="btn btn-ghost btn-xs px-1" @click="successMsg = ''"><SvgIcon name="x" size="12" /></button>
    </div>

    <!-- Loading state -->
    <div v-if="loading && jobs.length === 0" class="flex-1 flex items-center justify-center">
      <span class="text-xs text-base-content/40">加载中...</span>
    </div>

    <!-- Empty state -->
    <div v-else-if="jobs.length === 0" class="flex-1 flex flex-col items-center justify-center gap-2">
      <IconClock size="32" class="text-base-content/20" stroke-width="1.5" />
      <p class="text-sm text-base-content/40">暂无定时任务</p>
      <button class="btn btn-sm btn-primary mt-2" @click="openCreateModal">新建定时任务</button>
    </div>

    <!-- Job cards -->
    <div v-else class="flex-1 overflow-y-auto p-4">
      <div class="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-3">
        <div
          v-for="job in jobs"
          :key="job.id"
          class="bg-base-100 rounded-lg border border-base-content/10 p-3 hover:border-primary/30 transition-colors flex flex-col gap-2"
          :class="{ 'opacity-60': job.state === 'paused' || job.state === 'completed' }"
        >
          <!-- Row 1: name + status badge -->
          <div class="flex items-start justify-between gap-2">
            <span class="text-sm font-medium truncate" :title="job.name || job.id">{{ job.name || job.id }}</span>
            <span
              class="text-[10px] font-semibold uppercase px-1.5 py-0.5 rounded shrink-0"
              :class="statusBadgeClass(job)"
            >
              {{ job.state === 'paused' ? '已暂停' : job.state === 'active' ? '运行中' : job.state }}
            </span>
          </div>

          <!-- Row 2: schedule -->
          <div class="flex items-center gap-1.5 text-xs text-base-content/60">
            <IconCalendarEvent size="12" class="shrink-0" />
            <span class="font-mono">{{ job.schedule }}</span>
          </div>

          <!-- Row 3: prompt preview -->
          <p v-if="job.prompt" class="text-xs text-base-content/50 leading-relaxed line-clamp-2">{{ job.prompt }}</p>

          <!-- Row 4: timestamps -->
          <div class="flex items-center justify-between text-[11px] text-base-content/40">
            <span v-if="job.last_run_at" title="上次运行">
              <IconPlayerPlay size="10" class="inline mr-0.5" />{{ formatTime(job.last_run_at) }}
            </span>
            <span v-else-if="job.last_error" title="上次错误" class="text-error/60">
              <IconAlertCircle size="10" class="inline mr-0.5" />{{ formatTime(job.last_error) }}
            </span>
            <span v-else>-</span>
            <span v-if="job.next_run_at" title="下次运行">
              <IconClock size="10" class="inline mr-0.5" />{{ formatTime(job.next_run_at) }}
            </span>
            <span v-else class="text-base-content/30">-</span>
          </div>

          <!-- Last error -->
          <div v-if="job.last_error" class="text-[11px] text-error/70 truncate" :title="job.last_error">
            <IconAlertCircle size="10" class="inline mr-0.5 shrink-0" />{{ job.last_error }}
          </div>

          <!-- Row 5: actions -->
          <div class="flex items-center gap-1 pt-1 border-t border-base-content/5 mt-auto">
            <button
              class="btn btn-ghost btn-xs px-1.5 text-base-content/50 hover:text-primary"
              title="立即执行"
              :disabled="actionLoading === job.id"
              @click="triggerJob(job)"
            >
              <IconPlayerPlay size="13" />
            </button>
            <button
              v-if="job.state === 'paused' || job.state === 'completed'"
              class="btn btn-ghost btn-xs px-1.5 text-base-content/50 hover:text-success"
              title="恢复"
              :disabled="actionLoading === job.id"
              @click="resumeJob(job)"
            >
              <IconPlayerPlay size="13" />
            </button>
            <button
              v-else-if="job.state === 'active'"
              class="btn btn-ghost btn-xs px-1.5 text-base-content/50 hover:text-warning"
              title="暂停"
              :disabled="actionLoading === job.id"
              @click="pauseJob(job)"
            >
              <IconPlayerPause size="13" />
            </button>
            <button
              class="btn btn-ghost btn-xs px-1.5 text-base-content/50 hover:text-error"
              title="删除"
              :disabled="actionLoading === job.id"
              @click="removeJob(job)"
            >
              <IconTrash size="13" />
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Create Modal -->
    <Modal v-model="showCreateModal" title="新建定时任务" width="540px">
      <div class="space-y-4">
        <!-- Name -->
        <div>
          <label class="text-xs font-medium text-base-content/70 block mb-1">任务名称</label>
          <input
            v-model="form.name"
            type="text"
            class="input input-bordered input-sm w-full text-sm"
            placeholder="可选，留空自动生成"
          />
        </div>

        <!-- Schedule -->
        <div>
          <label class="text-xs font-medium text-base-content/70 block mb-1">
            调度规则 <span class="text-error">*</span>
          </label>
          <input
            v-model="form.schedule"
            type="text"
            class="input input-bordered input-sm w-full text-sm"
            placeholder="e.g. every 2h, 0 9 * * *, once in 30m"
          />
          <p class="text-[11px] text-base-content/40 mt-1">支持自然语言描述、cron 表达式或 "once in 30m" 格式</p>
        </div>

        <!-- Prompt -->
        <div>
          <label class="text-xs font-medium text-base-content/70 block mb-1">任务内容 (Prompt)</label>
          <textarea
            v-model="form.prompt"
            class="textarea textarea-bordered textarea-sm w-full text-sm"
            rows="4"
            placeholder="可选，描述任务的执行内容"
          ></textarea>
        </div>

        <!-- Deliver -->
        <div>
          <label class="text-xs font-medium text-base-content/70 block mb-1">送达方式</label>
          <input
            v-model="form.deliver"
            type="text"
            class="input input-bordered input-sm w-full text-sm"
            placeholder="可选，如 telegram, discord:#channel"
          />
          <p class="text-[11px] text-base-content/40 mt-1">留空使用默认通知方式</p>
        </div>
      </div>

      <template #footer>
        <button class="btn btn-sm btn-ghost" @click="showCreateModal = false">取消</button>
        <button
          class="btn btn-sm btn-primary"
          :disabled="!form.schedule.trim() || creating"
          @click="createJob"
        >
          {{ creating ? '创建中...' : '创建' }}
        </button>
      </template>
    </Modal>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import Modal from '@/components/ui/Modal.vue'
import {
  IconClock,
  IconPlus,
  IconTrash,
  IconPlayerPlay,
  IconPlayerPause,
  IconCalendarEvent,
  IconAlertCircle,
  IconCircleCheck,
} from '@tabler/icons-vue'
import { getTauriAPI } from '@/utils/tauri-api'
import type { CronJob } from '@/types'

const loading = ref(false)
const jobs = ref<CronJob[]>([])
const errorMsg = ref('')
const successMsg = ref('')
const actionLoading = ref<string | null>(null)
const creating = ref(false)
const showCreateModal = ref(false)
const form = ref({
  name: '',
  schedule: '',
  prompt: '',
  deliver: '',
})

function statusBadgeClass(job: CronJob): string {
  switch (job.state) {
    case 'active':
      return 'bg-success/15 text-success'
    case 'paused':
      return 'bg-warning/15 text-warning'
    case 'completed':
      return 'bg-info/15 text-info'
    default:
      return 'bg-base-300 text-base-content/60'
  }
}

function formatTime(t: string | null): string {
  if (!t) return ''
  try {
    const d = new Date(t)
    return d.toLocaleString('zh-CN', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    })
  } catch {
    return t
  }
}

function showError(msg: string) {
  errorMsg.value = msg
  successMsg.value = ''
  setTimeout(() => { errorMsg.value = '' }, 6000)
}

function showSuccess(msg: string) {
  successMsg.value = msg
  errorMsg.value = ''
  setTimeout(() => { successMsg.value = '' }, 3000)
}

async function refresh() {
  loading.value = true
  errorMsg.value = ''
  try {
    const api = getTauriAPI()
    jobs.value = await api.listCronJobs()
  } catch (e: any) {
    showError(e?.message || '加载失败')
  } finally {
    loading.value = false
  }
}

function openCreateModal() {
  form.value = { name: '', schedule: '', prompt: '', deliver: '' }
  showCreateModal.value = true
}

async function createJob() {
  if (!form.value.schedule.trim()) return
  creating.value = true
  try {
    const api = getTauriAPI()
    await api.createCronJob(
      form.value.schedule.trim(),
      form.value.prompt.trim() || undefined,
      form.value.name.trim() || undefined,
      form.value.deliver.trim() || undefined,
    )
    showCreateModal.value = false
    showSuccess('任务创建成功')
    await refresh()
  } catch (e: any) {
    showError(e?.message || '创建失败')
  } finally {
    creating.value = false
  }
}

async function removeJob(job: CronJob) {
  if (!confirm(`确定要删除定时任务「${job.name || job.id}」吗？`)) return
  actionLoading.value = job.id
  try {
    const api = getTauriAPI()
    await api.removeCronJob(job.id)
    showSuccess('任务已删除')
    await refresh()
  } catch (e: any) {
    showError(e?.message || '删除失败')
  } finally {
    actionLoading.value = null
  }
}

async function pauseJob(job: CronJob) {
  actionLoading.value = job.id
  try {
    const api = getTauriAPI()
    await api.pauseCronJob(job.id)
    showSuccess('任务已暂停')
    await refresh()
  } catch (e: any) {
    showError(e?.message || '暂停失败')
  } finally {
    actionLoading.value = null
  }
}

async function resumeJob(job: CronJob) {
  actionLoading.value = job.id
  try {
    const api = getTauriAPI()
    await api.resumeCronJob(job.id)
    showSuccess('任务已恢复')
    await refresh()
  } catch (e: any) {
    showError(e?.message || '恢复失败')
  } finally {
    actionLoading.value = null
  }
}

async function triggerJob(job: CronJob) {
  actionLoading.value = job.id
  try {
    const api = getTauriAPI()
    await api.triggerCronJob(job.id)
    showSuccess('任务已触发执行')
    await refresh()
  } catch (e: any) {
    showError(e?.message || '触发失败')
  } finally {
    actionLoading.value = null
  }
}

onMounted(refresh)
</script>
