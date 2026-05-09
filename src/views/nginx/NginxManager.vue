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
                    <button @click.stop="openEditPresetForm(preset)" class="btn btn-ghost btn-xs btn-square" title="编辑"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg> </button>
                    <button @click.stop="onDeletePreset(preset.id)" class="btn btn-ghost btn-xs btn-square text-error" title="删除">×</button>
                  </div>
                </div>
              </div>
            </div>

            <div v-if="presets.length === 0" class="p-4 text-center text-base-content/60 text-sm">
              <template v-if="servers.length === 0">
                <p class="m-0"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M12 2v8"/><path d="M4.93 10.93a8 8 0 1 1 14.14 0"/><path d="M16 16l-1.5-1.5"/></svg> 尚未配置服务器</p>
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
                <template v-if="loading"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg> 加载中...</template><template v-else><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><polyline points="22 12 16 12 14 15 10 15 8 12 2 12"/><path d="M5.45 5.11L2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"/></svg> 获取配置</template>
              </button>
              <button
                @click="onTestConfig"
                :disabled="!currentPreset || loading"
                class="btn btn-outline btn-sm"
              >
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 3h6v5a5 5 0 0 0 4.38 4.97l.62.03A2 2 0 0 1 20 17.86V19a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2v-1.14a2 2 0 0 1 1-1.86l.62-.03A5 5 0 0 0 9 8V3z"/><line x1="6" y1="9" x2="18" y2="9"/></svg>  预检测试
              </button>
              <button
                @click="showDeployDialog = true"
                :disabled="!currentPreset || !configContent || loading"
                class="btn btn-primary btn-sm"
              >
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09z"/><path d="M12 15l-3-3a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 0 1-4 2z"/><path d="M9 12H4s.55-3.03 2-4c1.62-1.08 5 0 5 0"/><path d="M12 15v5s3.03-.55 4-2c1.08-1.62 0-5 0-5"/></svg> 发布
              </button>
            </div>
          </div>
        </div>

        <!-- 视图模式切换 -->
        <div class="flex gap-1">
          <button
            :class="['btn btn-ghost btn-sm', { 'btn-active': viewMode === 'raw' }]"
            @click="viewMode = 'raw'"
          ><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>  原始编辑</button>
        </div>

        <!-- 测试结果提示 -->
        <div
          v-if="testResult"
          class="flex items-center gap-2 px-4 py-2.5 rounded-lg text-sm"
          :class="testResult.passed ? 'bg-success/10 text-success' : 'bg-error/10 text-error'"
        >
          <span><template v-if="testResult.passed"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" class="inline-block align-text-bottom"><polyline points="20 6 9 17 4 12"/></svg> 配置检测通过</template><template v-else><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" class="inline-block align-text-bottom"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg> 配置检测失败</template></span>
          <span v-if="testResult.message" class="text-xs opacity-70">{{ testResult.message }}</span>
          <button @click="testResult = null" class="btn btn-ghost btn-xs ml-auto"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" class="inline-block"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg></button>
        </div>

        <!-- 配置编辑器 -->
        <div class="bg-base-100 border border-base-content/10 rounded-xl flex-1">
          <textarea
            v-if="viewMode === 'raw'"
            v-model="configContent"
            :disabled="!currentPreset"
            placeholder="选择预设后点击「获取配置」加载远程 Nginx 配置..."
            class="textarea textarea-bordered font-mono min-h-[400px] w-full border-0 focus:outline-none rounded-xl p-4 resize-y"
            spellcheck="false"
          ></textarea>
          <div v-else class="p-4">
            <pre v-if="configContent" class="overflow-x-auto whitespace-pre-wrap text-sm">{{ configContent }}</pre>
            <div v-else class="text-sm text-base-content/50">暂无配置内容</div>
          </div>
        </div>

        <!-- 版本历史 -->
        <div v-if="versions.length > 0" class="bg-base-100 border border-base-content/10 rounded-xl p-4">
          <h4 class="text-sm font-semibold text-base-content m-0 mb-3"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg> 版本历史</h4>
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
              ><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/></svg>  回滚</button>
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
        <h3 class="font-bold text-lg"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09z"/><path d="M12 15l-3-3a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 0 1-4 2z"/><path d="M9 12H4s.55-3.03 2-4c1.62-1.08 5 0 5 0"/><path d="M12 15v5s3.03-.55 4-2c1.08-1.62 0-5 0-5"/></svg> 发布配置</h3>
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

<script setup lang="ts">// @ts-nocheck
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
