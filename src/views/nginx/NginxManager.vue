<template>
  <div class="min-h-screen">
    <div class="flex gap-4">
      <!-- 左侧：预设列表 -->
      <div class="w-80 shrink-0">
        <div class="card bg-base-200">
          <div class="card-body p-4">
            <div class="card-title flex justify-between items-center">
              <h3 class="text-lg">🔧 Nginx 配置</h3>
              <button @click="openNewPresetForm" class="btn btn-primary btn-sm">+ 新增预设</button>
            </div>

            <!-- 分组 -->
            <div
              v-for="groupEntry in groupedPresets"
              :key="groupEntry.groupName"
            >
              <div
                class="flex items-center gap-2 cursor-pointer px-3 py-2 hover:bg-base-300 rounded-box"
                @click="toggleGroup(groupEntry.groupName)"
              >
                <span>{{ collapsedGroups.has(groupEntry.groupName) ? '▶' : '▼' }}</span>
                <span class="font-medium">{{ groupEntry.groupName }}</span>
                <span class="badge badge-sm">{{ groupEntry.presets.length }}</span>
              </div>
              <div v-show="!collapsedGroups.has(groupEntry.groupName)">
                <div
                  v-for="preset in groupEntry.presets"
                  :key="preset.id"
                  class="flex items-center justify-between px-3 py-2 hover:bg-base-300 cursor-pointer rounded-box"
                  :class="{ 'bg-primary/10': currentPreset?.id === preset.id }"
                  @click="onSelectPreset(preset)"
                >
                  <div class="flex flex-col min-w-0">
                    <span class="font-medium truncate">{{ preset.name }}</span>
                    <span class="text-xs opacity-60 truncate">{{ preset.configPath || '未设置路径' }}</span>
                  </div>
                  <div class="flex gap-1 shrink-0">
                    <button @click.stop="openEditPresetForm(preset)" class="btn btn-ghost btn-xs btn-square" title="编辑">✏️</button>
                    <button @click.stop="onDeletePreset(preset.id)" class="btn btn-ghost btn-xs btn-square text-error" title="删除">×</button>
                  </div>
                </div>
              </div>
            </div>

            <div v-if="presets.length === 0" class="p-4 text-center opacity-60">
              <template v-if="servers.length === 0">
                <p>🔌 尚未配置服务器</p>
                <p class="text-sm">Nginx 管理需要先添加 SSH 服务器</p>
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
        <div class="card bg-base-200">
          <div class="card-body p-4">
            <div class="flex items-center justify-between">
              <div>
                <span v-if="currentPreset" class="font-bold text-lg">{{ currentPreset.name }}</span>
                <span v-else class="opacity-50 italic">请先选择预设</span>
              </div>
              <div class="flex gap-2">
                <button
                  @click="onFetchConfig"
                  :disabled="!currentPreset || loading"
                  class="btn"
                >
                  {{ loading ? '⏳ 加载中...' : '📥 获取配置' }}
                </button>
                <button
                  @click="onTestConfig"
                  :disabled="!currentPreset || loading"
                  class="btn btn-outline"
                >
                  🧪 预检测试
                </button>
                <button
                  @click="showDeployDialog = true"
                  :disabled="!currentPreset || !configContent || loading"
                  class="btn btn-primary"
                >
                  🚀 发布
                </button>
              </div>
            </div>
          </div>
        </div>

        <!-- 视图模式切换 -->
        <div class="flex gap-1">
          <button
            :class="['btn btn-ghost btn-sm', { 'btn-active': viewMode === 'raw' }]"
            @click="viewMode = 'raw'"
          >📝 原始编辑</button>
        </div>

        <!-- 测试结果提示 -->
        <div
          v-if="testResult"
          class="alert"
          :class="testResult.passed ? 'alert-success' : 'alert-error'"
        >
          <span>{{ testResult.passed ? '✅ 配置检测通过' : '❌ 配置检测失败' }}</span>
          <span v-if="testResult.message">{{ testResult.message }}</span>
          <button @click="testResult = null" class="btn btn-ghost btn-xs">×</button>
        </div>

        <!-- 配置编辑器 -->
        <div class="card bg-base-200 flex-1">
          <div class="card-body p-0">
            <textarea
              v-if="viewMode === 'raw'"
              v-model="configContent"
              :disabled="!currentPreset"
              placeholder="选择预设后点击「获取配置」加载远程 Nginx 配置..."
              class="textarea textarea-bordered font-mono min-h-[400px] rounded-box p-4"
              spellcheck="false"
            ></textarea>
            <div v-else class="p-4">
              <pre v-if="configContent" class="overflow-x-auto whitespace-pre-wrap">{{ configContent }}</pre>
              <div v-else class="opacity-50">暂无配置内容</div>
            </div>
          </div>
        </div>

        <!-- 版本历史 -->
        <div v-if="versions.length > 0" class="card bg-base-200">
          <div class="card-body p-4">
            <h4 class="card-title">📜 版本历史</h4>
            <div class="divide-y">
              <div
                v-for="version in versions"
                :key="version.id"
                class="flex items-center justify-between p-3"
              >
                <div class="flex items-center gap-2 flex-wrap">
                  <span class="font-medium">{{ version.comment || '无备注' }}</span>
                  <span v-if="version.isCurrent" class="badge badge-success badge-sm">当前生效</span>
                  <span class="text-xs opacity-60">{{ formatDate(version.createdAt) }}</span>
                  <span v-if="version.checksum" class="text-xs opacity-40 font-mono">{{ version.checksum }}</span>
                </div>
                <button
                  @click="onRollback(version.id)"
                  :disabled="loading"
                  class="btn btn-ghost btn-xs"
                  title="回滚到此版本"
                >🔄 回滚</button>
              </div>
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
        <h3 class="font-bold text-lg">🚀 发布配置</h3>
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
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck
import { ref, computed, onMounted } from 'vue'
import { useNginxConfig } from '../../composables/useNginxConfig'
import GroupedServerSelector from '@/views/server/GroupedServerSelector.vue'

const {
  loading, presets, currentPreset, configContent, versions, testResult,
  servers, serverGroups,
  loadPresets, loadServers, savePreset, deletePreset,
  fetchConfig, testConfig, deployConfig, rollbackToVersion,
} = useNginxConfig()

// UI state
const showPresetForm = ref(false)
const showDeployDialog = ref(false)
const editingPreset = ref<any>(null)
const viewMode = ref<'raw' | 'parsed'>('raw')
const collapsedGroups = ref(new Set<string>())
const deployComment = ref('')

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
  if (!confirm('确定删除此预设？关联的版本历史也会一并删除。')) return
  await deletePreset(id)
}

function onSelectPreset(preset: any) {
  currentPreset.value = preset
  configContent.value = ''
  versions.value = []
  testResult.value = null
}

async function onFetchConfig() {
  if (!currentPreset.value) return
  await fetchConfig(currentPreset.value)
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
  if (!confirm('确定回滚到此版本？当前配置将被替换。')) return
  await rollbackToVersion(versionId)
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
