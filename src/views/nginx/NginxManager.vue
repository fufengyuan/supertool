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

      <!-- 右侧：选项卡布局 -->
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
                @click="onGenerateConfig"
                :disabled="!currentPreset || loading"
                class="btn btn-ghost btn-sm"
              >
                <template v-if="loading"><SvgIcon name="clock" size="14" /> 生成中...</template>
                <template v-else><SvgIcon name="eye" size="14" /> 预览</template>
              </button>
              <button
                @click="onTestConfig"
                :disabled="!currentPreset || loading"
                class="btn btn-outline btn-sm"
              >
                <template v-if="loading"><SvgIcon name="clock" size="14" /> 检测中...</template>
                <template v-else><SvgIcon name="lightbulb" size="14" />  预检测试</template>
              </button>
              <button
                @click="openDeployDialog"
                :disabled="!currentPreset || loading"
                class="btn btn-primary btn-sm">
                <template v-if="loading"><SvgIcon name="clock" size="14" /> 发布中...</template>
                <template v-else><SvgIcon name="rocket" size="14" /> 发布</template>
              </button>
            </div>
          </div>
        </div>

        <!-- 选项卡导航 -->
        <div class="tabs tabs-boxed bg-base-100 border border-base-content/10">
          <button
            v-for="tab in tabs"
            :key="tab.key"
            class="tab"
            :class="{ 'tab-active': currentTab === tab.key }"
            :disabled="!currentPreset"
            @click="switchTab(tab.key)"
          >
            <SvgIcon :name="tab.icon" size="14" class="mr-1" />
            {{ tab.label }}
          </button>
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

        <!-- 选项卡内容 -->
        <div class="bg-base-100 border border-base-content/10 rounded-xl flex-1 min-h-[400px]">
          <template v-for="tab in tabs" :key="tab.key">
            <div v-if="currentTab === tab.key" class="p-4">
            <!-- 未选择预设时显示提示 -->
            <div v-if="!currentPreset" class="flex items-center justify-center h-32 text-base-content/50">
              <p>请先选择一个预设</p>
            </div>
            <!-- 组件加载中 -->
            <div v-else-if="!loadedComponents[tab.key]" class="flex items-center justify-center h-32 text-base-content/50">
              <p>加载中...</p>
            </div>
            <!-- 渲染子组件 -->
            <component
              v-else-if="loadedComponents[tab.key]"
              :is="loadedComponents[tab.key]"
              :preset-id="currentPreset.id"
            />
            <!-- 兜底占位 -->
            <div v-else class="flex flex-col items-center justify-center h-32 text-base-content/50">
              <p>子页面 <strong>{{ tabComponentMap[tab.key] }}</strong> 尚未创建</p>
              <p class="text-xs mt-2">请在 <code>src/views/nginx/</code> 下创建此组件</p>
            </div>
          </div>
          </template>
        </div>

        <!-- 配置预览（可编辑） -->
        <div v-if="currentPreset" class="bg-base-100 border border-base-content/10 rounded-xl">
          <div
            class="flex items-center justify-between p-3 cursor-pointer select-none"
            @click="showConfigPreview = !showConfigPreview"
          >
            <span class="text-sm font-semibold"><SvgIcon name="eye" size="14" /> 配置预览</span>
            <span class="text-xs text-base-content/50">{{ showConfigPreview ? '收起' : '展开' }}</span>
          </div>
          <div v-if="showConfigPreview" class="px-3 pb-3">
            <textarea
              v-model="configContent"
              class="textarea textarea-bordered w-full font-mono text-xs leading-relaxed"
              style="height: 400px; resize: vertical;"
              spellcheck="false"
              placeholder="点击「预览」按钮生成配置，或在此直接编辑..."
            ></textarea>
          </div>
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
      <div class="modal-box max-w-4xl max-h-[90vh] overflow-y-auto">
        <h3 class="font-bold text-lg"><SvgIcon name="rocket" size="14" /> 发布配置</h3>

        <!-- 配置差异 -->
        <div class="mt-4">
          <label class="text-sm font-medium mb-1 block">配置差异对比</label>
          <div class="border border-base-content/10 rounded-lg overflow-hidden">
            <!-- 加载中 -->
            <div v-if="diffLoading" class="flex items-center justify-center h-20 text-sm text-base-content/50">
              <SvgIcon name="clock" size="14" class="mr-2" /> 正在生成配置并计算差异…
            </div>
            <!-- 无差异 -->
            <div v-else-if="diffSame" class="flex items-center justify-center h-12 text-sm text-success gap-2">
              <SvgIcon name="check" size="14" /> 生成配置与当前配置一致，无变更
            </div>
            <!-- 失败 -->
            <div v-else-if="diffError" class="flex items-center justify-center h-12 text-sm text-error gap-2">
              <SvgIcon name="alertTriangle" size="14" /> {{ diffError }}
            </div>
            <!-- 差异内容 -->
            <pre v-else-if="diffContent" class="text-xs leading-relaxed overflow-auto max-h-80 m-0 p-3 bg-base-200/50 font-mono whitespace-pre-wrap">{{ diffContent }}</pre>
            <div v-else class="flex items-center justify-center h-12 text-sm text-base-content/50">点击"预览"生成配置后再发布可查看差异</div>
          </div>
        </div>

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
          <button @click="closeDeployDialog" class="btn btn-ghost">取消</button>
          <button @click="onDeploy" class="btn btn-primary" :disabled="!deployComment.trim() || diffLoading">
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
import { ref, computed, reactive, onMounted, markRaw } from 'vue'
import { useNginxConfig } from '../../composables/useNginxConfig'
import { getTauriAPI } from '../../utils/tauri-api'
import GroupedServerSelector from '@/views/server/GroupedServerSelector.vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'

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
const collapsedGroups = ref(new Set<string>())
const deployComment = ref('')
const showDeleteConfirm = ref(false)
const showRollbackConfirm = ref(false)
const confirmDeleteId = ref('')
const confirmRollbackId = ref('')
const deleting = ref(false)
const currentTab = ref('server')
const showConfigPreview = ref(false)

// Config diff state
const diffLoading = ref(false)
const diffContent = ref('')
const diffSame = ref(false)
const diffError = ref('')
const generatedNewConfig = ref('')

// Tab definitions
const tabs = [
  { key: 'server', label: 'Server', icon: 'server' },
  { key: 'upstream', label: 'Upstream', icon: 'layers' },
  { key: 'http', label: 'HTTP', icon: 'globe' },
  { key: 'stream', label: 'Stream', icon: 'activity' },
  { key: 'cert', label: 'Cert', icon: 'shield' },
  { key: 'template', label: '模板', icon: 'file' },
  { key: 'basic', label: '基本设置', icon: 'settings' },
  { key: 'param', label: '额外参数', icon: 'list' },
  { key: 'deny', label: '黑白名单', icon: 'shield' },
  { key: 'password', label: '密码文件', icon: 'lock' },
]

// Map tab keys to component file names
const tabComponentMap = {
  server: 'ServerPage.vue',
  upstream: 'UpstreamPage.vue',
  http: 'HttpPage.vue',
  stream: 'StreamPage.vue',
  cert: 'CertPage.vue',
  template: 'TemplatePage.vue',
  basic: 'BasicSettingPage.vue',
  param: 'ParamPage.vue',
  deny: 'DenyAllowPage.vue',
  password: 'PasswordPage.vue',
}

// Dynamically loaded components cache
const loadedComponents = reactive<Record<string, any>>({
  server: null,
  upstream: null,
  http: null,
  stream: null,
  cert: null,
  template: null,
  basic: null,
  param: null,
  deny: null,
  password: null,
})

function switchTab(tabKey: string) {
  currentTab.value = tabKey
  // Lazy-load the component if not already loaded
  if (!loadedComponents[tabKey]) {
    loadTabComponent(tabKey)
  }
}

async function loadTabComponent(tabKey: string) {
  const fileName = tabComponentMap[tabKey]
  try {
    const mod = await import(`./${fileName}`)
    loadedComponents[tabKey] = markRaw(mod.default || mod)
  } catch (err: any) {
    console.warn(`[NginxManager] 未能加载 ${fileName}:`, err?.message || err)
    // Mark as null so the template shows the fallback placeholder
    loadedComponents[tabKey] = null
  }
}

onMounted(async () => {
  await Promise.all([loadPresets(), loadServers()])
  // Pre-load the default tab
  loadTabComponent('server')
})

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
  currentTab.value = 'server'
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
}

async function onTestConfig() {
  if (!currentPreset.value) return
  await testConfig(currentPreset.value.serverId, currentPreset.value.configPath)
}

async function onGenerateConfig() {
  if (!currentPreset.value) return
  try {
    loading.value = true
    const result = await getTauriAPI().generateNginxConfig(currentPreset.value.id)
    configContent.value = result?.data || result || ''
    showConfigPreview.value = true
    // Also load version history
    const verResult = await getTauriAPI().getNginxConfigVersions(currentPreset.value.id)
    versions.value = verResult?.data || verResult || []
  } catch (err) {
    console.error('生成配置失败:', err)
  } finally {
    loading.value = false
  }
}

// Config diff functions
async function openDeployDialog() {
  showDeployDialog.value = true
  deployComment.value = ''
  // Reset diff state
  diffLoading.value = true
  diffContent.value = ''
  diffSame.value = false
  diffError.value = ''
  generatedNewConfig.value = ''
  // Auto-generate config for diff comparison
  await computeConfigDiff()
}

function closeDeployDialog() {
  showDeployDialog.value = false
  diffContent.value = ''
  diffSame.value = false
  diffError.value = ''
  generatedNewConfig.value = ''
}

async function computeConfigDiff() {
  if (!currentPreset.value) {
    diffError.value = '未选择预设'
    diffLoading.value = false
    return
  }
  try {
    const result = await getTauriAPI().generateNginxConfig(currentPreset.value.id)
    const newConfig = result?.data || result || ''
    generatedNewConfig.value = newConfig
    const oldConfig = configContent.value || ''

    if (oldConfig === newConfig) {
      diffSame.value = true
      diffContent.value = ''
      return
    }

    diffContent.value = computeUnifiedDiff(oldConfig, newConfig)
  } catch (err: any) {
    console.error('计算配置差异失败:', err)
    diffError.value = err?.message || '计算差异失败'
  } finally {
    diffLoading.value = false
  }
}

function computeUnifiedDiff(oldText: string, newText: string): string {
  const oldLines = oldText.split('\n')
  const newLines = newText.split('\n')

  // Build LCS table
  const m = oldLines.length
  const n = newLines.length
  const dp: number[][] = Array.from({ length: m + 1 }, () => new Array(n + 1).fill(0))
  for (let i = 1; i <= m; i++) {
    const oi = oldLines[i - 1]
    for (let j = 1; j <= n; j++) {
      if (oi === newLines[j - 1]) {
        dp[i][j] = dp[i - 1][j - 1] + 1
      } else {
        dp[i][j] = Math.max(dp[i - 1][j], dp[i][j - 1])
      }
    }
  }

  // Backtrack to get diff ops
  const ops: Array<{ prefix: string; text: string }> = []
  let i = m, j = n
  while (i > 0 || j > 0) {
    if (i > 0 && j > 0 && oldLines[i - 1] === newLines[j - 1]) {
      ops.push({ prefix: ' ', text: oldLines[i - 1] })
      i--; j--
    } else if (j > 0 && (i === 0 || dp[i][j - 1] >= dp[i - 1][j])) {
      ops.push({ prefix: '+', text: newLines[j - 1] })
      j--
    } else {
      ops.push({ prefix: '-', text: oldLines[i - 1] })
      i--
    }
  }
  ops.reverse()

  // Group into hunks with context
  const result: string[] = []
  let hunkStart = -1
  const hunkLines: Array<{ prefix: string; text: string }> = []
  const ctxBefore = 2

  for (let idx = 0; idx < ops.length; idx++) {
    const op = ops[idx]
    if (op.prefix !== ' ') {
      // Start or extend a hunk
      if (hunkStart === -1) {
        hunkStart = Math.max(0, idx - ctxBefore)
      }
      hunkLines.push(op)
    } else {
      if (hunkStart !== -1) {
        // Add trailing context
        const end = Math.min(ops.length, idx + ctxBefore + 1)
        for (let k = idx; k < end; k++) {
          hunkLines.push(ops[k])
        }
        // Emit hunk
        if (hunkLines.length > 0) {
          result.push('@@ -' + (hunkStart + 1) + ' +' + (hunkStart + 1) + ' @@')
          for (const hl of hunkLines) {
            result.push(hl.prefix + ' ' + hl.text)
          }
        }
        hunkStart = -1
        hunkLines.length = 0
        // Skip ahead past the context we just emitted
        idx += ctxBefore
      }
    }
  }

  // Emit remaining hunk
  if (hunkStart !== -1 && hunkLines.length > 0) {
    result.push('@@ -' + (hunkStart + 1) + ' +' + (hunkStart + 1) + ' @@')
    for (const hl of hunkLines) {
      result.push(hl.prefix + ' ' + hl.text)
    }
  }

  return result.join('\n')
}

async function onDeploy() {
  if (!deployComment.value.trim()) return
  // If we generated a new config for diffing and it differs, deploy the new config
  if (generatedNewConfig.value && !diffSame.value) {
    configContent.value = generatedNewConfig.value
  }
  const result = await deployConfig(deployComment.value)
  if (result?.success) {
    showDeployDialog.value = false
    deployComment.value = ''
    diffContent.value = ''
    diffSame.value = false
    diffError.value = ''
    generatedNewConfig.value = ''
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
</script>
