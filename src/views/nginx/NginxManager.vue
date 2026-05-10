<template>
  <div>
    <div class="flex gap-4">
      <!-- 左侧：预设列表 -->
      <div class="w-80 shrink-0">
        <div class="bg-base-100 border border-base-content/10 rounded-xl">
          <div class="p-4">
            <div class="flex justify-between items-center mb-3">
              <h3 class="text-base font-semibold m-0"><SvgIcon name="tool" size="14" />  Nginx 配置</h3>
              <button @click="openNewPresetForm" class="btn btn-primary btn-sm">+ 新增预设</button>
            </div>

            <!-- 分组 -->
            <div
              v-for="groupEntry in groupedPresets"
              :key="groupEntry.groupName"
            >
              <div
                class="flex items-center gap-2 cursor-pointer px-3 py-2 rounded-lg hover:bg-base-200"
                @click="toggleGroup(groupEntry.groupName)"
              >
                <span class="text-xs text-base-content/60">{{ collapsedGroups.has(groupEntry.groupName) ? '▶' : '▼' }}</span>
                <span class="text-sm font-medium text-base-content/80">{{ groupEntry.groupName }}</span>
                <span class="badge badge-sm badge-ghost">{{ groupEntry.presets.length }}</span>
              </div>
              <div v-show="!collapsedGroups.has(groupEntry.groupName)" class="ml-1 flex flex-col">
                <div
                  v-for="preset in groupEntry.presets"
                  :key="preset.id"
                  class="flex items-center justify-between px-3 py-2 rounded-lg cursor-pointer transition-colors duration-100 hover:bg-base-200"
                  :class="{ 'bg-primary/10': currentPreset?.id === preset.id }"
                  @click="onSelectPreset(preset)"
                >
                  <div class="flex flex-col min-w-0">
                    <span class="text-sm font-medium text-base-content truncate">{{ preset.name }}</span>
                    <span class="text-xs text-base-content/50 truncate">{{ preset.configPath || '未设置路径' }}</span>
                  </div>
                  <div class="flex gap-1 shrink-0">
                    <button @click.stop="openEditPresetForm(preset)" class="btn btn-ghost btn-xs btn-square" title="编辑"><SvgIcon name="pencil" size="14" /> </button>
                    <button @click.stop="onDeletePreset(preset.id)" class="btn btn-ghost btn-xs btn-square text-error" title="删除"><SvgIcon name="trash" size="14" /></button>
                  </div>
                </div>
              </div>
            </div>

            <div v-if="presets.length === 0" class="p-4 text-center text-base-content/60 text-sm">
              <template v-if="servers.length === 0">
                <p class="m-0"><SvgIcon name="plug" size="14" /> 尚未配置服务器</p>
                <p class="text-xs mt-1">Nginx 管理需要先添加 SSH 服务器</p>
              </template>
              <template v-else>
                暂无预设，点击上方按钮添加
              </template>
            </div>
          </div>
        </div>
      </div>

      <!-- 右侧：配置编辑 -->
      <div class="flex-1 flex flex-col gap-4 min-w-0">
        <!-- 工具栏 -->
        <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
          <div class="flex items-center justify-between">
            <div>
              <span v-if="currentPreset" class="font-semibold text-base text-base-content">{{ currentPreset.name }}</span>
              <span v-else class="text-sm text-base-content/50">请先选择预设</span>
            </div>
            <div class="flex gap-2">
              <button
                @click="onFetchConfig"
                :disabled="!currentPreset || loading"
                class="btn btn-ghost btn-sm"
              >
                <template v-if="loading"><SvgIcon name="clock" size="14" /> 加载中...</template><template v-else><SvgIcon name="inbox" size="14" /> 获取配置</template>
              </button>
              <button
                @click="onTestConfig"
                :disabled="!currentPreset || loading"
                class="btn btn-outline btn-sm"
              >
                <template v-if="loading"><SvgIcon name="clock" size="14" /> 检测中...</template><template v-else><SvgIcon name="lightbulb" size="14" />  预检测试</template>
              </button>
              <button
                @click="showDeployDialog = true"
                :disabled="!currentPreset || !configContent || loading"
                class="btn btn-primary btn-sm">
                <template v-if="loading"><SvgIcon name="clock" size="14" /> 发布中...</template><template v-else><SvgIcon name="rocket" size="14" /> 发布</template>
              </button>
            </div>
          </div>
        </div>

        <!-- 视图模式切换 -->
        <div class="flex gap-1">
          <button
            :class="['btn btn-ghost btn-sm', { 'btn-active': viewMode === 'visual' }]"
            @click="viewMode = 'visual'"
          ><SvgIcon name="grid" size="14" />  可视化编辑</button>
          <button
            :class="['btn btn-ghost btn-sm', { 'btn-active': viewMode === 'raw' }]"
            @click="viewMode = 'raw'"
          ><SvgIcon name="file" size="14" />  查看原生配置</button>
        </div>

        <!-- 测试结果提示 -->
        <div
          v-if="testResult"
          class="flex items-center gap-2 px-4 py-2.5 rounded-lg text-sm"
          :class="testResult.passed ? 'bg-success/10 text-success' : 'bg-error/10 text-error'"
        >
          <span><template v-if="testResult.passed"><SvgIcon name="check" size="14" /> 配置检测通过</template><template v-else><SvgIcon name="x" size="14" /> 配置检测失败</template></span>
          <span v-if="testResult.message" class="text-xs opacity-70">{{ testResult.message }}</span>
          <button @click="testResult = null" class="btn btn-ghost btn-xs ml-auto"><SvgIcon name="x" size="14" /></button>
        </div>

        <!-- 配置编辑器 -->
        <div class="bg-base-100 border border-base-content/10 rounded-xl flex-1 min-h-[300px]">
          <textarea
            v-if="viewMode === 'raw'"
            v-model="configContent"
            :disabled="!currentPreset"
            placeholder="Nginx 原生配置文本..."
            class="textarea textarea-bordered font-mono min-h-[400px] w-full border-0 focus:outline-none rounded-xl p-4 resize-y"
            spellcheck="false"
          ></textarea>
          <NginxStructuredEditor
            v-else
            v-model="configContent"
            :key="currentPreset?.id"
          />
        </div>

        <!-- 版本历史 -->
        <div v-if="versions.length > 0" class="bg-base-100 border border-base-content/10 rounded-xl p-4">
          <h4 class="text-sm font-semibold text-base-content m-0 mb-3"><SvgIcon name="file" size="14" /> 版本历史</h4>
          <div class="flex flex-col">
            <div
              v-for="version in versions"
              :key="version.id"
              class="flex items-center justify-between py-2.5 border-b border-base-content/5 last:border-b-0"
            >
              <div class="flex items-center gap-2 flex-wrap">
                <span class="text-sm text-base-content">{{ version.comment || '无备注' }}</span>
                <span v-if="version.isCurrent" class="badge badge-success badge-sm">当前生效</span>
                <span class="text-xs text-base-content/50">{{ formatDate(version.createdAt) }}</span>
                <span v-if="version.checksum" class="text-xs text-base-content/30 font-mono">{{ version.checksum }}</span>
              </div>
              <button
                @click="onRollback(version.id)"
                :disabled="loading"
                class="btn btn-ghost btn-xs"
                title="回滚到此版本"
              ><SvgIcon name="refresh" size="14" />  回滚</button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 新增/编辑预设弹窗 -->
    <div v-if="showPresetForm" class="modal modal-open" @click.self="showPresetForm = false">
      <div class="modal-box">
        <h3 class="font-bold text-lg">{{ editingPreset ? '编辑预设' : '新增预设' }}</h3>
        <div class="flex flex-col gap-1 mt-4">
          <label class="text-sm font-medium">预设名称</label>
          <input v-model="presetForm.name" placeholder="例如：生产环境API配置" class="input input-bordered w-full" />
        </div>
        <div class="flex flex-col gap-1 mt-3">
          <label class="text-sm font-medium">分组</label>
          <input
            v-model="presetForm.groupName"
            list="group-suggestions"
            placeholder="例如：生产 / 测试"
            class="input input-bordered w-full"
          />
          <datalist id="group-suggestions">
            <option value="生产" />
            <option value="测试" />
            <option value="开发" />
            <option value="预发" />
          </datalist>
        </div>
        <div class="flex flex-col gap-1 mt-3">
          <label class="text-sm font-medium">服务器</label>
          <div>
            <GroupedServerSelector
              :servers="servers"
              :groups="serverGroups"
              v-model="presetForm.serverId"
              mode="single"
            />
          </div>
        </div>
        <div class="flex flex-col gap-1 mt-3">
          <label class="text-sm font-medium">配置文件路径</label>
          <input v-model="presetForm.configPath" placeholder="例如：/etc/nginx/nginx.conf" class="input input-bordered w-full" />
        </div>
        <div class="flex flex-col gap-1 mt-3">
          <label class="text-sm font-medium">描述</label>
          <textarea v-model="presetForm.description" placeholder="可选描述信息" class="textarea textarea-bordered w-full" rows="2"></textarea>
        </div>
        <div class="modal-action">
          <button @click="showPresetForm = false" class="btn btn-ghost">取消</button>
          <button @click="onSavePreset" class="btn btn-primary" :disabled="!presetForm.name || !presetForm.serverId || !presetForm.configPath">
            {{ editingPreset ? '保存' : '创建' }}
          </button>
        </div>
      </div>
    </div>

    <!-- 发布弹窗 -->
    <div v-if="showDeployDialog" class="modal modal-open" @click.self="showDeployDialog = false">
      <div class="modal-box">
        <h3 class="font-bold text-lg"><SvgIcon name="rocket" size="14" /> 发布配置</h3>
        <div class="flex flex-col gap-1 mt-4">
          <label class="text-sm font-medium">备注</label>
          <input
            v-model="deployComment"
            placeholder="请输入发布说明"
            class="input input-bordered w-full"
            @keyup.enter="onDeploy"
          />
        </div>
        <div class="modal-action">
          <button @click="showDeployDialog = false" class="btn btn-ghost">取消</button>
          <button @click="onDeploy" class="btn btn-primary" :disabled="!deployComment.trim()">
            确认发布
          </button>
        </div>
      </div>
    </div>

    <!-- 删除确认弹窗 -->
    <div v-if="showDeleteConfirm" class="modal modal-open" @click.self="showDeleteConfirm = false">
      <div class="modal-box">
        <h3 class="font-bold text-lg"><SvgIcon name="alertTriangle" size="14" /> 确认删除</h3>
        <p class="text-sm text-base-content/70 mt-2">确定删除此预设？关联的版本历史也会一并删除。</p>
        <div class="modal-action">
          <button @click="showDeleteConfirm = false" class="btn btn-ghost btn-sm">取消</button>
          <button @click="executeDeletePreset" class="btn btn-error btn-sm" :disabled="deleting">
            {{ deleting ? '删除中…' : '确认删除' }}
          </button>
        </div>
      </div>
    </div>

    <!-- 回滚确认弹窗 -->
    <div v-if="showRollbackConfirm" class="modal modal-open" @click.self="showRollbackConfirm = false">
      <div class="modal-box">
        <h3 class="font-bold text-lg"><SvgIcon name="refresh" size="14" /> 确认回滚</h3>
        <p class="text-sm text-base-content/70 mt-2">确定回滚到此版本？当前配置将被替换。</p>
        <div class="modal-action">
          <button @click="showRollbackConfirm = false" class="btn btn-ghost btn-sm">取消</button>
          <button @click="executeRollback" class="btn btn-warning btn-sm" :disabled="deleting">
            {{ deleting ? '回滚中…' : '确认回滚' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, computed, onMounted } from 'vue'
import { useNginxConfig } from '../../composables/useNginxConfig'
import GroupedServerSelector from '@/views/server/GroupedServerSelector.vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import NginxStructuredEditor from './components/NginxStructuredEditor.vue'

const {
  loading, presets, currentPreset, configContent, versions, testResult,
  servers, serverGroups,
  loadPresets, loadServers, savePreset, deletePreset,
  fetchConfig, testConfig, deployConfig, rollbackToVersion,
  loadCachedConfig,
} = useNginxConfig()

// UI state
const showPresetForm = ref(false)
const showDeployDialog = ref(false)
const editingPreset = ref<any>(null)
const viewMode = ref<'raw' | 'visual'>('visual')
const collapsedGroups = ref(new Set<string>())
const deployComment = ref('')
const showDeleteConfirm = ref(false)
const showRollbackConfirm = ref(false)
const confirmDeleteId = ref('')
const confirmRollbackId = ref('')
const deleting = ref(false)

const presetForm = ref({
  id: '',
  name: '',
  serverId: '',
  configPath: '',
  description: '',
  groupName: '默认',
})

// Group presets by groupName
const groupedPresets = computed(() => {
  const groups = new Map<string, any[]>()
  for (const preset of presets.value) {
    const g = preset.groupName || '默认'
    if (!groups.has(g)) groups.set(g, [])
    groups.get(g).push(preset)
  }
  return Array.from(groups.entries()).map(([groupName, presets]) => ({
    groupName,
    presets,
  }))
})

function toggleGroup(groupName: string) {
  if (collapsedGroups.value.has(groupName)) {
    collapsedGroups.value.delete(groupName)
  } else {
    collapsedGroups.value.add(groupName)
  }
  collapsedGroups.value = new Set(collapsedGroups.value)
}

function openNewPresetForm() {
  editingPreset.value = null
  presetForm.value = {
    id: '',
    name: '',
    serverId: '',
    configPath: '',
    description: '',
    groupName: '默认',
  }
  showPresetForm.value = true
}

function openEditPresetForm(preset: any) {
  editingPreset.value = preset
  presetForm.value = { ...preset }
  showPresetForm.value = true
}

async function onSavePreset() {
  const result = await savePreset({ ...presetForm.value })
  if (result) {
    showPresetForm.value = false
  }
}

async function onDeletePreset(id: string) {
  confirmDeleteId.value = id
  showDeleteConfirm.value = true
}

async function executeDeletePreset() {
  const id = confirmDeleteId.value
  if (!id) return
  deleting.value = true
  await deletePreset(id)
  showDeleteConfirm.value = false
  confirmDeleteId.value = ''
  deleting.value = false
}

async function onSelectPreset(preset: any) {
  currentPreset.value = preset
  configContent.value = ''
  versions.value = []
  testResult.value = null
  viewMode.value = 'visual'
  // Auto-load cached config from local DB
  const hasCache = await loadCachedConfig(preset.id)
  if (!hasCache) {
    // No cached version — try fetching remote config automatically
    await onFetchConfig()
  }
}

async function onFetchConfig() {
  if (!currentPreset.value) return
  await fetchConfig(currentPreset.value)
  // Auto-switch to visual mode
  if (configContent.value) {
    viewMode.value = 'visual'
  }
}

async function onTestConfig() {
  if (!currentPreset.value) return
  await testConfig(currentPreset.value.serverId, currentPreset.value.configPath)
}

async function onDeploy() {
  if (!deployComment.value.trim()) return
  const result = await deployConfig(deployComment.value)
  if (result?.success) {
    showDeployDialog.value = false
    deployComment.value = ''
  }
}

async function onRollback(versionId: string) {
  confirmRollbackId.value = versionId
  showRollbackConfirm.value = true
}

async function executeRollback() {
  const versionId = confirmRollbackId.value
  if (!versionId) return
  if (!confirm('确定回滚到此版本？当前配置将被替换。')) return
  deleting.value = true
  await rollbackToVersion(versionId)
  showRollbackConfirm.value = false
  confirmRollbackId.value = ''
  deleting.value = false
  if (configContent.value) {
    viewMode.value = 'visual'
  }
}

function formatDate(dateStr: string) {
  if (!dateStr) return ''
  try {
    const d = new Date(dateStr)
    return d.toLocaleString('zh-CN', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
  } catch {
    return dateStr
  }
}

onMounted(async () => {
  await Promise.all([loadPresets(), loadServers()])
})
</script>
