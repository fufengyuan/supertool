<template>
  <div class="flex flex-col h-full min-h-0">
    <!-- Tabs -->
    <div class="tabs tabs-bordered px-5 border-b border-base-content/10">
      <a
        v-for="tab in tabs"
        :key="tab.key"
        class="tab tab-bordered text-sm font-medium pb-2"
        :class="{ 'tab-active': activeTab === tab.key }"
        @click="activeTab = tab.key"
      >
        <SvgIcon :name="tab.icon" size="14" class="inline-block align-text-bottom mr-1" />
        {{ tab.label }}
      </a>
    </div>

    <div class="flex-1 overflow-y-auto p-4 lg:p-5">
      <!-- ============ Tab 1: 服务探测 ============ -->
      <div v-show="activeTab === 'services'">
        <div class="flex items-center justify-between mb-4 gap-3">
          <div class="flex items-center gap-2">
            <button class="btn btn-primary btn-sm" @click="openServiceModal()">
              <SvgIcon name="plus" size="14" />
              新增服务
            </button>
          </div>
          <div class="flex items-center gap-2">
            <input
              v-model="serviceSearch"
              type="text"
              placeholder="搜索名称…"
              class="input input-bordered input-sm w-48"
              @input="filterServices"
            />
          </div>
        </div>

        <div class="overflow-x-auto">
          <table class="table table-sm w-full">
            <thead>
              <tr>
                <th>名称</th>
                <th>目标</th>
                <th>间隔</th>
                <th>超时</th>
                <th>重试阈值</th>
                <th>状态</th>
                <th>下次检查</th>
                <th class="w-24">操作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="svc in filteredServices" :key="svc.id">
                <td class="font-medium">{{ svc.name }}</td>
                <td class="font-mono text-xs">{{ svc.host }}:{{ svc.port }}</td>
                <td>{{ svc.check_interval || svc.checkInterval || 60 }}s</td>
                <td>{{ svc.timeout || 5 }}s</td>
                <td>{{ svc.retry_threshold || svc.retryThreshold || 3 }}次</td>
                <td>
                  <span
                    class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium"
                    :class="serviceStatusClass(svc)"
                  >
                    <span
                      class="w-2 h-2 rounded-full inline-block"
                      :class="serviceStatusDot(svc)"
                    ></span>
                    {{ serviceStatusLabel(svc) }}
                  </span>
                </td>
                <td class="text-xs text-base-content/60">{{ svc.next_check_at || svc.nextCheckAt || '—' }}</td>
                <td>
                  <div class="flex items-center gap-1">
                    <button class="btn btn-ghost btn-xs btn-square" title="编辑" @click="openServiceModal(svc)">
                      <SvgIcon name="pencil" size="14" />
                    </button>
                    <button class="btn btn-ghost btn-xs btn-square text-error" title="删除" @click="confirmDeleteService(svc)">
                      <SvgIcon name="trash" size="14" />
                    </button>
                  </div>
                </td>
              </tr>
              <tr v-if="filteredServices.length === 0">
                <td colspan="8" class="text-center text-base-content/40 py-8">暂无服务探测项</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- ============ Tab 2: 资源到期 ============ -->
      <div v-show="activeTab === 'resources'">
        <div class="flex items-center justify-between mb-4">
          <button class="btn btn-primary btn-sm" @click="openResourceModal()">
            <SvgIcon name="plus" size="14" />
            新增资源
          </button>
        </div>

        <div class="overflow-x-auto">
          <table class="table table-sm w-full">
            <thead>
              <tr>
                <th>名称</th>
                <th>分类</th>
                <th>到期时间</th>
                <th>提前告警</th>
                <th>剩余天数</th>
                <th>状态</th>
                <th class="w-24">操作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="res in resources" :key="res.id">
                <td class="font-medium">{{ res.name }}</td>
                <td>{{ res.category }}</td>
                <td>{{ formatDate(res.expire_at || res.expireAt) }}</td>
                <td>{{ res.alert_advance_days || res.alertAdvanceDays || 30 }}天前</td>
                <td>{{ remainingDays(res) }}</td>
                <td>
                  <span
                    class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium"
                    :class="resourceStatusClass(res)"
                  >
                    <span
                      class="w-2 h-2 rounded-full inline-block"
                      :class="resourceStatusDot(res)"
                    ></span>
                    {{ resourceStatusLabel(res) }}
                  </span>
                </td>
                <td>
                  <div class="flex items-center gap-1">
                    <button class="btn btn-ghost btn-xs btn-square" title="编辑" @click="openResourceModal(res)">
                      <SvgIcon name="pencil" size="14" />
                    </button>
                    <button class="btn btn-ghost btn-xs btn-square text-error" title="删除" @click="confirmDeleteResource(res)">
                      <SvgIcon name="trash" size="14" />
                    </button>
                  </div>
                </td>
              </tr>
              <tr v-if="resources.length === 0">
                <td colspan="7" class="text-center text-base-content/40 py-8">暂无到期资源</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- ============ Tab 3: 邮件配置 ============ -->
      <div v-show="activeTab === 'email'">
        <div class="max-w-xl">
          <div class="bg-base-100 border border-base-content/10 rounded-xl p-5 space-y-4">
            <h4 class="text-sm font-semibold flex items-center gap-2">
              <SvgIcon name="mail" size="16" />
              邮件发送配置
            </h4>

            <div class="form-control">
              <label class="label py-1">
                <span class="label-text text-xs">SMTP 服务器</span>
              </label>
              <input v-model="emailConfig.smtp_host" type="text" placeholder="smtp.example.com" class="input input-bordered input-sm" />
            </div>

            <div class="form-control">
              <label class="label py-1">
                <span class="label-text text-xs">端口</span>
              </label>
              <input v-model.number="emailConfig.smtp_port" type="number" placeholder="465" class="input input-bordered input-sm" />
            </div>

            <div class="form-control">
              <label class="label py-1">
                <span class="label-text text-xs">用户名</span>
              </label>
              <input v-model="emailConfig.username" type="text" class="input input-bordered input-sm" />
            </div>

            <div class="form-control">
              <label class="label py-1">
                <span class="label-text text-xs">密码</span>
              </label>
              <input v-model="emailConfig.password" type="password" class="input input-bordered input-sm" />
            </div>

            <div class="form-control">
              <label class="label py-1">
                <span class="label-text text-xs">发送邮箱</span>
              </label>
              <input v-model="emailConfig.from_email" type="text" placeholder="alert@example.com" class="input input-bordered input-sm" />
            </div>

            <div class="form-control">
              <label class="label py-1">
                <span class="label-text text-xs">接收邮箱</span>
              </label>
              <input v-model="emailConfig.to_email" type="text" placeholder="admin@example.com" class="input input-bordered input-sm" />
            </div>

            <div class="form-control">
              <label class="label cursor-pointer justify-start gap-2 py-1">
                <input v-model="emailConfig.use_tls" type="checkbox" class="checkbox checkbox-sm checkbox-primary" />
                <span class="label-text text-xs">使用TLS</span>
              </label>
            </div>

            <div class="flex gap-2 pt-2">
              <button class="btn btn-ghost btn-sm" @click="testEmailConfig" :disabled="emailTesting">
                {{ emailTesting ? '测试中…' : '测试连接' }}
              </button>
              <button class="btn btn-primary btn-sm" @click="saveEmailConfig" :disabled="emailSaving">
                {{ emailSaving ? '保存中…' : '保存配置' }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- ============ 模态框：服务探测表单 ============ -->
    <Teleport to="body">
      <div v-if="showServiceModal" class="fixed inset-0 z-50 flex items-center justify-center" @click.self="closeServiceModal">
        <div class="fixed inset-0 bg-black/50"></div>
        <div class="bg-base-100 rounded-xl shadow-xl w-full max-w-lg mx-4 z-10 max-h-[85vh] overflow-y-auto">
          <div class="flex items-center justify-between px-5 py-3 border-b border-base-content/10">
            <h3 class="font-semibold text-base">{{ editingService ? '编辑服务' : '新增服务' }}</h3>
            <button class="btn btn-ghost btn-sm btn-circle" @click="closeServiceModal">
              <SvgIcon name="x" size="14" />
            </button>
          </div>
          <div class="p-5 space-y-4">
            <div class="form-control">
              <label class="label py-1"><span class="label-text text-xs">名称</span></label>
              <input v-model="serviceForm.name" type="text" class="input input-bordered input-sm" />
            </div>
            <div class="form-control">
              <label class="label py-1"><span class="label-text text-xs">主机</span></label>
              <input v-model="serviceForm.host" type="text" placeholder="IP 或域名" class="input input-bordered input-sm" />
            </div>
            <div class="form-control">
              <label class="label py-1"><span class="label-text text-xs">端口</span></label>
              <input v-model.number="serviceForm.port" type="number" class="input input-bordered input-sm" />
            </div>
            <div class="grid grid-cols-3 gap-3">
              <div class="form-control">
                <label class="label py-1"><span class="label-text text-xs">检查间隔(秒)</span></label>
                <input v-model.number="serviceForm.check_interval" type="number" class="input input-bordered input-sm" />
              </div>
              <div class="form-control">
                <label class="label py-1"><span class="label-text text-xs">超时(秒)</span></label>
                <input v-model.number="serviceForm.timeout" type="number" class="input input-bordered input-sm" />
              </div>
              <div class="form-control">
                <label class="label py-1"><span class="label-text text-xs">重试次数</span></label>
                <input v-model.number="serviceForm.retry_threshold" type="number" class="input input-bordered input-sm" />
              </div>
            </div>
            <div class="form-control">
              <label class="label cursor-pointer justify-start gap-2 py-1">
                <input v-model="serviceForm.enabled" type="checkbox" class="toggle toggle-primary toggle-sm" />
                <span class="label-text text-xs">启用</span>
              </label>
            </div>
          </div>
          <div class="flex justify-end gap-2 px-5 py-3 border-t border-base-content/10">
            <button class="btn btn-ghost btn-sm" @click="closeServiceModal">取消</button>
            <button class="btn btn-primary btn-sm" @click="saveService" :disabled="serviceSaving">
              {{ serviceSaving ? '保存中…' : '保存' }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- ============ 模态框：资源到期表单 ============ -->
    <Teleport to="body">
      <div v-if="showResourceModal" class="fixed inset-0 z-50 flex items-center justify-center" @click.self="closeResourceModal">
        <div class="fixed inset-0 bg-black/50"></div>
        <div class="bg-base-100 rounded-xl shadow-xl w-full max-w-lg mx-4 z-10 max-h-[85vh] overflow-y-auto">
          <div class="flex items-center justify-between px-5 py-3 border-b border-base-content/10">
            <h3 class="font-semibold text-base">{{ editingResource ? '编辑资源' : '新增资源' }}</h3>
            <button class="btn btn-ghost btn-sm btn-circle" @click="closeResourceModal">
              <SvgIcon name="x" size="14" />
            </button>
          </div>
          <div class="p-5 space-y-4">
            <div class="form-control">
              <label class="label py-1"><span class="label-text text-xs">名称</span></label>
              <input v-model="resourceForm.name" type="text" class="input input-bordered input-sm" />
            </div>
            <div class="form-control">
              <label class="label py-1"><span class="label-text text-xs">分类</span></label>
              <div class="flex gap-2">
                <select v-model="resourceForm.category" class="select select-bordered select-sm flex-1">
                  <option value="">选择分类</option>
                  <option v-for="cat in resourceCategories" :key="cat" :value="cat">{{ cat }}</option>
                  <option value="__custom__">自定义</option>
                </select>
                <input
                  v-if="resourceForm.category === '__custom__'"
                  v-model="resourceForm.categoryCustom"
                  type="text"
                  placeholder="输入分类"
                  class="input input-bordered input-sm flex-1"
                />
              </div>
            </div>
            <div class="form-control">
              <label class="label py-1"><span class="label-text text-xs">到期时间</span></label>
              <input v-model="resourceForm.expire_at" type="date" class="input input-bordered input-sm" />
            </div>
            <div class="form-control">
              <label class="label py-1"><span class="label-text text-xs">提前告警天数</span></label>
              <input v-model.number="resourceForm.alert_advance_days" type="number" class="input input-bordered input-sm" />
            </div>
          </div>
          <div class="flex justify-end gap-2 px-5 py-3 border-t border-base-content/10">
            <button class="btn btn-ghost btn-sm" @click="closeResourceModal">取消</button>
            <button class="btn btn-primary btn-sm" @click="saveResource" :disabled="resourceSaving">
              {{ resourceSaving ? '保存中…' : '保存' }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- ============ 确认删除对话框 ============ -->
    <Teleport to="body">
      <div v-if="deleteConfirm.show" class="fixed inset-0 z-50 flex items-center justify-center">
        <div class="fixed inset-0 bg-black/50"></div>
        <div class="bg-base-100 rounded-xl shadow-xl w-full max-w-sm mx-4 z-10 p-5">
          <h3 class="font-semibold text-base mb-2">确认删除</h3>
          <p class="text-sm text-base-content/70 mb-4">
            确定要删除「{{ deleteConfirm.name }}」吗？此操作不可撤销。
          </p>
          <div class="flex justify-end gap-2">
            <button class="btn btn-ghost btn-sm" @click="deleteConfirm.show = false">取消</button>
            <button class="btn btn-error btn-sm" @click="executeDelete" :disabled="deleteConfirm.deleting">
              {{ deleteConfirm.deleting ? '删除中…' : '确认删除' }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, reactive } from 'vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { useToast } from '@/composables/useToast'
import { getTauriAPI } from '@/utils/tauri-api'

const api = getTauriAPI() as any
const toast = useToast()

// ============ Tabs ============
const tabs = [
  { key: 'services', label: '服务探测', icon: 'search' },
  { key: 'resources', label: '资源到期', icon: 'clock' },
  { key: 'email', label: '邮件配置', icon: 'mail' },
]
const activeTab = ref('services')

// ============ 服务探测 ============
const services = ref<any[]>([])
const serviceSearch = ref('')
const filteredServices = ref<any[]>([])
const showServiceModal = ref(false)
const editingService = ref<any>(null)
const serviceSaving = ref(false)
const serviceForm = reactive({
  name: '',
  host: '',
  port: 80,
  check_interval: 60,
  timeout: 5,
  retry_threshold: 3,
  enabled: true,
})

function filterServices() {
  const q = serviceSearch.value.toLowerCase()
  if (!q) {
    filteredServices.value = [...services.value]
  } else {
    filteredServices.value = services.value.filter((s) =>
      s.name.toLowerCase().includes(q)
    )
  }
}

function serviceStatusClass(svc: any) {
  const status = svc.last_status ?? svc.lastStatus
  if (status === null || status === undefined) return 'bg-base-200 text-base-content/50'
  if (status === 1) return 'bg-success/10 text-success'
  return 'bg-error/10 text-error'
}

function serviceStatusDot(svc: any) {
  const status = svc.last_status ?? svc.lastStatus
  if (status === null || status === undefined) return 'bg-base-content/30'
  if (status === 1) return 'bg-success'
  return 'bg-error'
}

function serviceStatusLabel(svc: any) {
  const status = svc.last_status ?? svc.lastStatus
  if (status === null || status === undefined) return '未检测'
  if (status === 1) return '正常'
  return '异常'
}

function openServiceModal(svc?: any) {
  if (svc) {
    editingService.value = svc
    serviceForm.name = svc.name
    serviceForm.host = svc.host
    serviceForm.port = svc.port
    serviceForm.check_interval = svc.check_interval ?? svc.checkInterval ?? 60
    serviceForm.timeout = svc.timeout ?? 5
    serviceForm.retry_threshold = svc.retry_threshold ?? svc.retryThreshold ?? 3
    serviceForm.enabled = svc.enabled ?? true
  } else {
    editingService.value = null
    serviceForm.name = ''
    serviceForm.host = ''
    serviceForm.port = 80
    serviceForm.check_interval = 60
    serviceForm.timeout = 5
    serviceForm.retry_threshold = 3
    serviceForm.enabled = true
  }
  showServiceModal.value = true
}

function closeServiceModal() {
  showServiceModal.value = false
  editingService.value = null
}

async function saveService() {
  serviceSaving.value = true
  try {
    const data = {
      name: serviceForm.name,
      host: serviceForm.host,
      port: serviceForm.port,
      checkInterval: serviceForm.check_interval,
      timeoutSeconds: serviceForm.timeout,
      maxRetries: serviceForm.retry_threshold,
      enabled: serviceForm.enabled,
    }
    if (editingService.value) {
      await api.updateAlertService({ id: editingService.value.id, ...data })
      toast.success('服务已更新')
    } else {
      await api.addAlertService(data)
      toast.success('服务已添加')
    }
    closeServiceModal()
    await loadServices()
  } catch (e: any) {
    toast.error(e.message || '操作失败')
  } finally {
    serviceSaving.value = false
  }
}

// ============ 资源到期 ============
const resources = ref<any[]>([])
const showResourceModal = ref(false)
const editingResource = ref<any>(null)
const resourceSaving = ref(false)
const resourceCategories = ['域名', '服务器', '证书', '许可证', 'SSL', '其他']
const resourceForm = reactive({
  name: '',
  category: '',
  categoryCustom: '',
  expire_at: '',
  alert_advance_days: 30,
})

function getCategory(form: typeof resourceForm) {
  if (form.category === '__custom__') return form.categoryCustom || '其他'
  return form.category || '其他'
}

function remainingDays(res: any) {
  const expire = res.expire_at ?? res.expireAt
  if (!expire) return '—'
  const now = new Date()
  const exp = new Date(expire)
  const diff = Math.ceil((exp.getTime() - now.getTime()) / (1000 * 60 * 60 * 24))
  return diff >= 0 ? `${diff}天` : '已过期'
}

function resourceStatusClass(res: any) {
  const expire = res.expire_at ?? res.expireAt
  if (!expire) return 'bg-base-200 text-base-content/50'
  const now = new Date()
  const exp = new Date(expire)
  const diff = exp.getTime() - now.getTime()
  const advanceDays = res.alert_advance_days ?? res.alertAdvanceDays ?? 30
  const advanceMs = advanceDays * 24 * 60 * 60 * 1000
  if (diff < 0) return 'bg-error/10 text-error'
  if (diff <= advanceMs) return 'bg-warning/10 text-warning'
  return 'bg-success/10 text-success'
}

function resourceStatusDot(res: any) {
  const expire = res.expire_at ?? res.expireAt
  if (!expire) return 'bg-base-content/30'
  const now = new Date()
  const exp = new Date(expire)
  const diff = exp.getTime() - now.getTime()
  const advanceDays = res.alert_advance_days ?? res.alertAdvanceDays ?? 30
  const advanceMs = advanceDays * 24 * 60 * 60 * 1000
  if (diff < 0) return 'bg-error'
  if (diff <= advanceMs) return 'bg-warning'
  return 'bg-success'
}

function resourceStatusLabel(res: any) {
  const expire = res.expire_at ?? res.expireAt
  if (!expire) return '未知'
  const now = new Date()
  const exp = new Date(expire)
  const diff = exp.getTime() - now.getTime()
  const advanceDays = res.alert_advance_days ?? res.alertAdvanceDays ?? 30
  const advanceMs = advanceDays * 24 * 60 * 60 * 1000
  if (diff < 0) return '已过期'
  if (diff <= advanceMs) return '即将到期'
  return '正常'
}

function openResourceModal(res?: any) {
  if (res) {
    editingResource.value = res
    const cat = res.category || ''
    resourceForm.category = resourceCategories.includes(cat) ? cat : '__custom__'
    resourceForm.categoryCustom = resourceCategories.includes(cat) ? '' : cat
    resourceForm.name = res.name
    resourceForm.expire_at = res.expire_at ?? res.expireAt ?? ''
    resourceForm.alert_advance_days = res.alert_advance_days ?? res.alertAdvanceDays ?? 30
  } else {
    editingResource.value = null
    resourceForm.name = ''
    resourceForm.category = ''
    resourceForm.categoryCustom = ''
    resourceForm.expire_at = ''
    resourceForm.alert_advance_days = 30
  }
  showResourceModal.value = true
}

function closeResourceModal() {
  showResourceModal.value = false
  editingResource.value = null
}

async function saveResource() {
  resourceSaving.value = true
  try {
    const data = {
      name: resourceForm.name,
      category: getCategory(resourceForm),
      expireAt: resourceForm.expire_at,
      alertAdvanceDays: resourceForm.alert_advance_days,
    }
    if (editingResource.value) {
      await api.updateAlertResource({ id: editingResource.value.id, ...data })
      toast.success('资源已更新')
    } else {
      await api.addAlertResource(data)
      toast.success('资源已添加')
    }
    closeResourceModal()
    await loadResources()
  } catch (e: any) {
    toast.error(e.message || '操作失败')
  } finally {
    resourceSaving.value = false
  }
}

// ============ 邮件配置 ============
const emailConfig = reactive({
  smtp_host: '',
  smtp_port: 465,
  username: '',
  password: '',
  from_email: '',
  to_email: '',
  use_tls: true,
})
const emailTesting = ref(false)
const emailSaving = ref(false)

async function testEmailConfig() {
  emailTesting.value = true
  try {
    await api.testEmailConfig({
      smtpHost: emailConfig.smtp_host,
      smtpPort: emailConfig.smtp_port,
      smtpUsername: emailConfig.username,
      smtpPassword: emailConfig.password,
      fromEmail: emailConfig.from_email,
      toEmail: emailConfig.to_email,
      smtpUseTls: emailConfig.use_tls,
    })
    toast.success('测试邮件发送成功')
  } catch (e: any) {
    toast.error(e.message || '测试失败')
  } finally {
    emailTesting.value = false
  }
}

async function saveEmailConfig() {
  emailSaving.value = true
  try {
    await api.saveEmailConfig({
      smtpHost: emailConfig.smtp_host,
      smtpPort: emailConfig.smtp_port,
      smtpUsername: emailConfig.username,
      smtpPassword: emailConfig.password,
      fromEmail: emailConfig.from_email,
      toEmail: emailConfig.to_email,
      smtpUseTls: emailConfig.use_tls,
    })
    toast.success('邮件配置已保存')
  } catch (e: any) {
    toast.error(e.message || '保存失败')
  } finally {
    emailSaving.value = false
  }
}

// ============ 删除确认 ============
const deleteConfirm = reactive<{
  show: boolean
  type: 'service' | 'resource'
  id: string
  name: string
  deleting: boolean
}>({
  show: false,
  type: 'service',
  id: '',
  name: '',
  deleting: false,
})

function confirmDeleteService(svc: any) {
  deleteConfirm.show = true
  deleteConfirm.type = 'service'
  deleteConfirm.id = svc.id
  deleteConfirm.name = svc.name
  deleteConfirm.deleting = false
}

function confirmDeleteResource(res: any) {
  deleteConfirm.show = true
  deleteConfirm.type = 'resource'
  deleteConfirm.id = res.id
  deleteConfirm.name = res.name
  deleteConfirm.deleting = false
}

async function executeDelete() {
  deleteConfirm.deleting = true
  try {
    if (deleteConfirm.type === 'service') {
      await api.deleteAlertService(deleteConfirm.id)
      toast.success('服务已删除')
      await loadServices()
    } else {
      await api.deleteAlertResource(deleteConfirm.id)
      toast.success('资源已删除')
      await loadResources()
    }
    deleteConfirm.show = false
  } catch (e: any) {
    toast.error(e.message || '删除失败')
  } finally {
    deleteConfirm.deleting = false
  }
}

// ============ 工具函数 ============
function formatDate(dateStr: string) {
  if (!dateStr) return '—'
  const d = new Date(dateStr)
  return d.toISOString().split('T')[0]
}

// ============ 数据加载 ============
async function loadServices() {
  try {
    const data = await api.getAlertServices()
    services.value = data ?? []
    filterServices()
  } catch (e: any) {
    console.error('加载服务失败:', e)
  }
}

async function loadResources() {
  try {
    const data = await api.getAlertResources()
    resources.value = data ?? []
  } catch (e: any) {
    console.error('加载资源失败:', e)
  }
}

async function loadEmailConfig() {
  try {
    const data = await api.getEmailConfig()
    if (data) {
      emailConfig.smtp_host = data.smtp_host ?? data.smtpHost ?? ''
      emailConfig.smtp_port = data.smtp_port ?? data.smtpPort ?? 465
      emailConfig.username = data.username ?? ''
      emailConfig.password = data.password ?? ''
      emailConfig.from_email = data.from_email ?? data.fromEmail ?? ''
      emailConfig.to_email = data.to_email ?? data.toEmail ?? ''
      emailConfig.use_tls = data.use_tls ?? data.useTls ?? true
    }
  } catch (e: any) {
    console.error('加载邮件配置失败:', e)
  }
}

onMounted(() => {
  loadServices()
  loadResources()
  loadEmailConfig()
})
</script>
