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
                <span v-if="collapsedGroups.has(groupEntry.groupName)" class="text-xs text-base-content/60"><SvgIcon name="chevronRight" size="10" /></span>
                <span v-else class="text-xs text-base-content/60"><SvgIcon name="chevronDown" size="10" /></span>
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
      <div class="flex-1 flex flex-col gap-3 min-w-0">
        <!-- 工具栏 -->
        <div class="bg-base-100 border border-base-content/10 rounded-xl p-3">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
              <span v-if="currentPreset" class="font-semibold text-sm text-base-content">{{ currentPreset.name }}</span>
              <span v-else class="text-xs text-base-content/50">请先选择预设</span>
              <!-- 状态信息 -->
              <div v-if="currentPreset" class="flex items-center gap-2 text-xs text-base-content/50 border-l border-base-content/10 pl-3">
                <span class="flex items-center gap-1">
                  <SvgIcon name="clock" size="10" /> 上次发布：
                  <span v-if="lastDeployTime">{{ lastDeployTime }}</span>
                  <span v-else class="text-base-content/30">暂无</span>
                </span>
              </div>
            </div>
            <div class="flex gap-1.5">
              <label
                class="flex items-center gap-1.5 text-xs text-base-content/60 cursor-pointer select-none mr-1"
                title="开启后 CLI/AI 无法部署 nginx 配置，需在 GUI 操作"
              >
                <input
                  type="checkbox"
                  class="checkbox checkbox-sm"
                  v-model="deployApproval"
                  @change="onToggleDeployApproval"
                />
                部署需审批
              </label>
              <button
                @click="onGenerateConfig"
                :disabled="!currentPreset || loading"
                class="btn btn-ghost btn-sm btn-square"
                title="预览"
              >
                <template v-if="loading"><SvgIcon name="clock" size="14" /></template>
                <template v-else><SvgIcon name="eye" size="14" /></template>
              </button>
              <button
                @click="onTestConfig"
                :disabled="!currentPreset || loading"
                class="btn btn-outline btn-sm"
              >
                <template v-if="loading"><SvgIcon name="clock" size="14" /></template>
                <template v-else><SvgIcon name="lightbulb" size="14" /> 预检测试</template>
              </button>
              <button
                @click="onImportConfig"
                :disabled="!currentPreset || loading || !configContent"
                class="btn btn-outline btn-sm"
              >
                <template v-if="loading"><SvgIcon name="clock" size="14" /> 导入中...</template>
                <template v-else><SvgIcon name="upload" size="14" /> 导入配置</template>
              </button>
              <button
                @click="openDeployDialog"
                :disabled="!currentPreset || loading"
                class="btn btn-primary btn-sm">
                <template v-if="loading"><SvgIcon name="clock" size="14" /></template>
                <template v-else><SvgIcon name="rocket" size="14" /> 发布</template>
              </button>
            </div>
          </div>
        </div>

        <!-- 选项卡导航 -->
        <div class="tabs tabs-boxed bg-base-100 border border-base-content/10 p-0.5 gap-0">
          <button
            v-for="tab in tabs"
            :key="tab.key"
            class="tab tab-xs"
            :class="{ 'tab-active': currentTab === tab.key }"
            :disabled="!currentPreset"
            @click="switchTab(tab.key)"
          >
            <SvgIcon :name="tab.icon" size="12" />
            {{ tab.label }}
          </button>
        </div>

        <!-- 测试结果提示 -->
        <div
          v-if="testResult"
          class="flex items-center gap-2 px-3 py-2 rounded-lg text-xs"
          :class="testResult.passed ? 'bg-success/10 text-success' : 'bg-error/10 text-error'"
        >
          <span><template v-if="testResult.passed"><SvgIcon name="check" size="12" /> 配置检测通过</template><template v-else><SvgIcon name="x" size="12" /> 配置检测失败</template></span>
          <span v-if="testResult.message" class="opacity-70">{{ testResult.message }}</span>
          <button @click="testResult = null" class="btn btn-ghost btn-xs ml-auto"><SvgIcon name="x" size="12" /></button>
        </div>

        <!-- 选项卡内容 -->
        <div class="bg-base-100 border border-base-content/10 rounded-xl flex-1 min-h-[400px]">
          <template v-for="tab in tabs" :key="tab.key">
            <div v-if="currentTab === tab.key" class="p-3">
            <!-- 未选择预设时显示提示 -->
            <div v-if="!currentPreset" class="flex items-center justify-center h-32 text-base-content/50">
              <p>请先选择一个预设</p>
            </div>
            <!-- 组件加载中 -->
            <div v-else-if="componentStates[tab.key] === 'loading'" class="flex items-center justify-center h-32 text-base-content/50">
              <p>加载中...</p>
            </div>
            <!-- 组件加载失败 -->
            <div v-else-if="componentStates[tab.key] === 'error'" class="flex flex-col items-center justify-center h-32 text-base-content/50">
              <p class="text-error">子页面 <strong>{{ tabComponentMap[tab.key] }}</strong> 加载失败</p>
              <button @click="loadTabComponent(tab.key as NginxTabKey)" class="btn btn-ghost btn-xs mt-2">重试</button>
            </div>
            <!-- 渲染子组件（dataVersion 变化强制重建，导入后自动刷新数据） -->
            <component
              v-else-if="loadedComponents[tab.key]"
              :is="loadedComponents[tab.key]"
              :preset-id="currentPreset?.id"
              :key="dataVersion"
            />
            <!-- 兜底占位 -->
            <div v-else class="flex flex-col items-center justify-center h-32 text-base-content/50">
              <p>子页面 <strong>{{ tabComponentMap[tab.key] }}</strong> 尚未创建</p>
              <p class="text-xs mt-2">请在 <code>src/views/nginx/</code> 下创建此组件</p>
            </div>
          </div>
          </template>
        </div>

        <!-- 配置预览（语法高亮） -->
        <div v-if="currentPreset" class="bg-base-100 border border-base-content/10 rounded-xl">
          <div
            class="flex items-center justify-between p-3 cursor-pointer select-none"
            @click="showConfigPreview = !showConfigPreview"
          >
            <span class="text-sm font-semibold"><SvgIcon name="eye" size="14" /> 配置预览</span>
            <span class="text-xs text-base-content/50">{{ showConfigPreview ? '收起' : '展开' }}</span>
          </div>
          <div v-if="showConfigPreview" class="px-3 pb-3">
            <div class="nginx-preview text-xs font-mono leading-relaxed bg-base-200 rounded-lg overflow-auto" style="height: 600px; resize: vertical; padding: 12px;">
              <pre class="m-0 h-full whitespace-pre-wrap"><code v-html="highlightedConfig"></code></pre>
            </div>
          </div>
        </div>

        <!-- 版本历史 -->
        <div v-if="versions.length > 0" class="bg-base-100 border border-base-content/10 rounded-xl p-3">
          <h4 class="text-xs font-semibold text-base-content m-0 mb-2"><SvgIcon name="file" size="12" /> 版本历史</h4>
          <div class="flex flex-col">
            <div
              v-for="version in versions"
              :key="version.id"
              class="flex items-center justify-between py-2 border-b border-base-content/5 last:border-b-0"
            >
              <div class="flex items-center gap-2 flex-wrap">
                <span class="text-xs text-base-content">{{ version.comment || '无备注' }}</span>
                <span v-if="version.isCurrent" class="badge badge-success badge-xs">当前生效</span>
                <span class="text-xs text-base-content/50">{{ formatDate(version.createdAt) }}</span>
                <span v-if="version.checksum" class="text-xs text-base-content/30 font-mono">{{ version.checksum }}</span>
              </div>
              <button
                @click="onRollback(version.id)"
                :disabled="loading"
                class="btn btn-ghost btn-xs"
                title="回滚到此版本"
              ><SvgIcon name="refresh" size="12" /> 回滚</button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 新增/编辑预设弹窗 -->
    <div v-if="showPresetForm" class="modal modal-open">
      <div class="modal-box relative">
        <button @click="showPresetForm = false" class="absolute top-3 right-3 btn btn-ghost btn-sm btn-square rounded-full" title="关闭"><SvgIcon name="x" size="16" /></button>
        <h3 class="font-bold text-lg">{{ editingPreset ? '编辑预设' : '新增预设' }}</h3>
        <div class="flex flex-col gap-1 mt-3">
          <label class="text-xs font-medium text-base-content/80">预设名称</label>
          <input v-model="presetForm.name" placeholder="例如：生产环境API配置" class="input input-sm input-bordered w-full" />
        </div>
        <div class="flex flex-col gap-1 mt-2">
          <label class="text-xs font-medium text-base-content/80">分组</label>
          <input
            v-model="presetForm.groupName"
            list="group-suggestions"
            placeholder="例如：生产 / 测试"
            class="input input-sm input-bordered w-full"
          />
          <datalist id="group-suggestions">
            <option value="生产" />
            <option value="测试" />
            <option value="开发" />
            <option value="预发" />
          </datalist>
        </div>
        <div class="flex flex-col gap-1 mt-2">
          <label class="text-xs font-medium text-base-content/80">服务器</label>
          <div>
            <GroupedServerSelector
              :servers="servers"
              :groups="serverGroups"
              v-model="presetForm.serverId"
              mode="single"
            />
          </div>
        </div>
        <div class="flex flex-col gap-1 mt-2">
          <label class="text-xs font-medium text-base-content/80">配置文件路径</label>
          <input v-model="presetForm.configPath" placeholder="例如：/etc/nginx/nginx.conf" class="input input-sm input-bordered w-full" />
        </div>
        <div class="flex flex-col gap-1 mt-2">
          <label class="text-xs font-medium text-base-content/80">描述</label>
          <textarea v-model="presetForm.description" placeholder="可选描述信息" class="textarea textarea-bordered w-full text-xs" rows="2"></textarea>
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
    <div v-if="showDeployDialog" class="modal modal-open">
      <div class="modal-box relative max-w-5xl w-full max-h-[90vh] overflow-y-auto">
        <button @click="showDeployDialog = false" class="absolute top-3 right-3 btn btn-ghost btn-sm btn-square rounded-full" title="关闭"><SvgIcon name="x" size="16" /></button>
        <h3 class="font-bold text-lg"><SvgIcon name="rocket" size="14" /> 发布配置</h3>

        <!-- 分解模式开关 -->
        <div class="mt-3 flex items-center gap-2">
          <label class="text-xs font-medium cursor-pointer" for="decompose-switch">分解模式（conf.d/）</label>
          <input id="decompose-switch" type="checkbox" v-model="decomposeMode" class="toggle toggle-xs toggle-primary" @change="onDecomposeChange" />
          <span class="text-xs text-base-content/50">将 Server / Upstream 拆分为独立子文件</span>
        </div>

        <!-- 配置差异 -->
        <div class="mt-3">
          <label class="text-xs font-medium mb-1 block">配置差异对比</label>
          <div class="border border-base-content/10 rounded-lg overflow-hidden min-h-[300px]">
            <!-- 加载中 -->
            <div v-if="diffLoading" class="flex items-center justify-center h-16 text-xs text-base-content/50">
              <SvgIcon name="clock" size="12" class="mr-1 animate-spin" /> 正在生成配置并计算差异…
            </div>
            <!-- 无差异 -->
            <div v-else-if="diffSame" class="flex items-center justify-center h-10 text-xs text-success gap-1">
              <SvgIcon name="check" size="12" /> 生成配置与当前配置一致，无变更
            </div>
            <!-- 失败 -->
            <div v-else-if="diffError" class="flex items-center justify-center h-10 text-xs text-error gap-1">
              <SvgIcon name="alertTriangle" size="12" /> {{ diffError }}
            </div>
            <!-- 差异内容 - 使用 SplitDiffViewer -->
            <SplitDiffViewer
              v-else-if="diffContent"
              :files="diffFiles"
              :diff="diffContent"
              :loading="false"
            />
            <div v-else class="flex items-center justify-center h-10 text-xs text-base-content/50">点击"预览"生成配置后再发布可查看差异</div>
          </div>
        </div>

        <!-- 分解后的子文件列表 -->
        <div v-if="decomposeMode && decomposedSubFiles.length > 0" class="mt-3">
          <label class="text-xs font-medium mb-1 block">分解子文件（{{ decomposedSubFiles.length }} 个）</label>
          <div class="border border-base-content/10 rounded-lg overflow-hidden max-h-40 overflow-y-auto">
            <div
              v-for="(sf, idx) in decomposedSubFiles"
              :key="idx"
              class="px-2 py-1.5 border-b border-base-content/5 last:border-b-0 text-xs font-mono text-base-content/70 hover:bg-base-200/50 flex items-center gap-1"
            >
              <SvgIcon name="file" size="10" />
              <span class="font-semibold text-base-content">{{ sf.filename }}</span>
              <span class="text-base-content/40">—</span>
              <span class="truncate">{{ sf.content.split('\n')[0].substring(0, 60) }}{{ sf.content.includes('\n') ? '…' : '' }}</span>
            </div>
          </div>
        </div>

        <div class="flex flex-col gap-1 mt-3">
          <label class="text-xs font-medium">备注</label>
          <input
            v-model="deployComment"
            placeholder="请输入发布说明"
            class="input input-sm input-bordered w-full"
            @keyup.enter="onDeploy"
          />
        </div>
        <div class="modal-action">
          <button @click="closeDeployDialog" class="btn btn-ghost btn-sm">取消</button>
          <button @click="onDeploy" class="btn btn-primary btn-sm" :disabled="!deployComment.trim() || diffLoading">
            确认发布
          </button>
        </div>
      </div>
    </div>

    <!-- 部署审批确认弹窗（对齐 cicd 模式） -->
    <div v-if="showApprovalDialog" class="modal modal-open">
      <div class="modal-box relative">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-full bg-warning/20 flex items-center justify-center text-warning">
            <SvgIcon name="lock" size="20" />
          </div>
          <div>
            <h4 class="m-0 text-lg font-bold text-base-content">审核确认</h4>
            <p class="m-0 text-sm text-base-content/60 mt-0.5">Nginx 已开启部署审批</p>
          </div>
        </div>
        <p class="text-sm text-base-content/80 mb-5 leading-relaxed">
          当前预设「<strong>{{ currentPreset?.name }}</strong>」的部署需要审核确认。
          <br />请确认你已准备好部署，是否继续？
        </p>
        <div class="flex justify-end gap-2">
          <button @click="cancelApproval" class="btn btn-ghost">取消</button>
          <button @click="confirmApproval" class="btn bg-gradient-to-br from-warning to-amber-600 border-warning text-white hover:from-warning/90 hover:to-amber-600/90">
            <SvgIcon name="rocket" size="14" class="inline-block align-text-bottom" /> 确认部署
          </button>
        </div>
      </div>
    </div>

    <!-- 删除确认弹窗 -->
    <div v-if="showDeleteConfirm" class="modal modal-open">
      <div class="modal-box relative">
        <button @click="showDeleteConfirm = false" class="absolute top-3 right-3 btn btn-ghost btn-sm btn-square rounded-full" title="关闭"><SvgIcon name="x" size="16" /></button>
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
    <div v-if="showRollbackConfirm" class="modal modal-open">
      <div class="modal-box relative">
        <button @click="showRollbackConfirm = false" class="absolute top-3 right-3 btn btn-ghost btn-sm btn-square rounded-full" title="关闭"><SvgIcon name="x" size="16" /></button>
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

<script setup lang="ts">
defineOptions({ name: 'NginxManager' })
import { ref, computed, reactive, onMounted, markRaw } from 'vue'
import { useNginxConfig } from '../../composables/useNginxConfig'
import { useToast } from '../../composables/useToast'
import { getTauriAPI } from '../../utils/tauri-api'
import { confirm } from '@tauri-apps/plugin-dialog'
import GroupedServerSelector from '@/views/server/GroupedServerSelector.vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import SplitDiffViewer from '@/components/ui/SplitDiffViewer.vue'
import hljs from 'highlight.js/lib/core'
import nginxLang from 'highlight.js/lib/languages/nginx'
hljs.registerLanguage('nginx', nginxLang)

const {
  loading, presets, currentPreset, configContent, versions, testResult,
  servers, serverGroups,
  loadPresets, loadServers, savePreset, deletePreset,
  fetchConfig, testConfig, testConfigContent, deployConfig, rollbackToVersion,
  loadCachedConfig,
} = useNginxConfig()

const toast = useToast()

// UI state
const showPresetForm = ref(false)
const showDeployDialog = ref(false)
const editingPreset = ref<any>(null)

// nginx 模块级审批开关（settings: nginx_requires_approval）
// 开启后 CLI/AI 无法部署 nginx 配置（fetch/test 只读不受限）
const deployApproval = ref(false)
const onToggleDeployApproval = async () => {
  try {
    await getTauriAPI().setSetting('nginx_requires_approval', deployApproval.value ? 'true' : 'false')
    toast.success(deployApproval.value ? '已开启部署审批（CLI 将拦截 nginx deploy）' : '已关闭部署审批')
  } catch (e) {
    toast.error('保存失败: ' + String(e))
    deployApproval.value = !deployApproval.value
  }
}
const collapsedGroups = ref(new Set<string>())
const deployComment = ref('')
// 部署审批确认弹窗（对齐 cicd DeployPanel 模式）
const showApprovalDialog = ref(false)
let approvalResolve: ((v: boolean) => void) | null = null
function showApprovalConfirm(): Promise<boolean> {
  showApprovalDialog.value = true
  return new Promise(resolve => { approvalResolve = resolve })
}
function confirmApproval() {
  showApprovalDialog.value = false
  approvalResolve?.(true)
  approvalResolve = null
}
function cancelApproval() {
  showApprovalDialog.value = false
  approvalResolve?.(false)
  approvalResolve = null
}
const showDeleteConfirm = ref(false)
const showRollbackConfirm = ref(false)
const confirmDeleteId = ref('')
const confirmRollbackId = ref('')
const deleting = ref(false)
const currentTab = ref('server')
// 子页面数据版本：导入/切换预设后 +1 强制重建子组件（子组件 watch presetId + onMounted 重新加载）
const dataVersion = ref(0)
const showConfigPreview = ref(false)
const lastDeployTime = ref('')

// Config diff state
const diffLoading = ref(false)
const diffContent = ref('')
const diffSame = ref(false)
const diffError = ref('')
const generatedNewConfig = ref('')
const decomposedSubFiles = ref<Array<{filename: string, content: string}>>([])
const decomposeMode = ref(false)

// Files list for SplitDiffViewer
const diffFiles = computed(() => {
  if (!diffContent.value) {return null}
  return [{ path: 'nginx.conf', status: 'modified', changes: '' }]
})

const highlightedConfig = computed(() => {
  if (!configContent.value) {return ''}
  try {
    return hljs.highlight(configContent.value, { language: 'nginx' }).value
  } catch {
    return configContent.value.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
  }
})

// Tab definitions
const tabs = [
  { key: 'server', label: 'Server', icon: 'server' },
  { key: 'upstream', label: 'Upstream', icon: 'layers' },
  { key: 'http', label: 'HTTP', icon: 'globe' },
  { key: 'stream', label: 'Stream', icon: 'zap' },
  { key: 'cert', label: 'Cert', icon: 'shield' },
  { key: 'template', label: '模板', icon: 'file' },
  { key: 'basic', label: '基本设置', icon: 'settings' },
  { key: 'param', label: '额外参数', icon: 'menu' },
  { key: 'deny', label: '黑白名单', icon: 'shield' },
  { key: 'password', label: '密码文件', icon: 'lock' },
] as const

type NginxTabKey = (typeof tabs)[number]['key']

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
// 组件加载状态：undefined=未加载, 'loading'=加载中, 'loaded'=已加载, 'error'=加载失败
const componentStates = reactive<Record<string, 'loading' | 'loaded' | 'error'>>({})

function switchTab(tabKey: string) {
  currentTab.value = tabKey
  // Lazy-load the component if not already loaded
  if (!loadedComponents[tabKey] && componentStates[tabKey] !== 'loading') {
    loadTabComponent(tabKey as NginxTabKey)
  }
}

async function loadTabComponent(tabKey: NginxTabKey) {
  const fileName = tabComponentMap[tabKey]
  try {
    // Vite needs static import paths for code-splitting in production builds
    const MODULE_MAP: Record<string, () => Promise<any>> = {
      'ServerPage.vue': () => import('./ServerPage.vue'),
      'UpstreamPage.vue': () => import('./UpstreamPage.vue'),
      'HttpPage.vue': () => import('./HttpPage.vue'),
      'StreamPage.vue': () => import('./StreamPage.vue'),
      'CertPage.vue': () => import('./CertPage.vue'),
      'TemplatePage.vue': () => import('./TemplatePage.vue'),
      'BasicSettingPage.vue': () => import('./BasicSettingPage.vue'),
      'ParamPage.vue': () => import('./ParamPage.vue'),
      'DenyAllowPage.vue': () => import('./DenyAllowPage.vue'),
      'PasswordPage.vue': () => import('./PasswordPage.vue'),
    }
    const loader = MODULE_MAP[fileName]
    if (!loader) {throw new Error(`Unknown component: ${fileName}`)}
    componentStates[tabKey] = 'loading'
    const mod = await loader()
    loadedComponents[tabKey] = markRaw(mod.default || mod)
    componentStates[tabKey] = 'loaded'
  } catch (err: any) {
    console.warn(`[NginxManager] 未能加载 ${fileName}:`, err?.message || err)
    loadedComponents[tabKey] = null
    componentStates[tabKey] = 'error'
  }
}

onMounted(async () => {
  await Promise.all([loadPresets(), loadServers()])
  // 读取 nginx 审批开关状态
  try {
    const setting = await getTauriAPI().getSetting('nginx_requires_approval')
    deployApproval.value = setting === 'true'
  } catch (e) {
    // 忽略：开关保持默认关闭
  }
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
    if (!groups.has(g)) {groups.set(g, [])}
    groups.get(g)?.push(preset)
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
  if (!id) {return}
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
  generatedNewConfig.value = ''
  currentTab.value = 'server'
  // Auto-load cached config from local DB
  const hasCache = await loadCachedConfig(preset.id)
  if (!hasCache) {
    // No cached version — try fetching remote config automatically
    await onFetchConfig()
  }
}

async function onFetchConfig() {
  if (!currentPreset.value) {return}
  await fetchConfig(currentPreset.value)
}

async function onTestConfig() {
  if (!currentPreset.value) {return}
  try {
    loading.value = true
    // Auto generate config if not already generated
    if (!generatedNewConfig.value) {
      const result = await getTauriAPI().generateNginxConfig(currentPreset.value.id)
      const newConfig = result?.data || result || ''
      if (newConfig) {
        generatedNewConfig.value = newConfig
        configContent.value = newConfig
      }
    }
    const contentToTest = generatedNewConfig.value || configContent.value
    if (!contentToTest) {
      toast.warning('生成配置失败，无法测试')
      return
    }
    await testConfigContent(currentPreset.value.serverId, currentPreset.value.configPath, contentToTest)
  } finally {
    loading.value = false
  }
}

async function onGenerateConfig() {
  if (!currentPreset.value) {return}
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

async function onImportConfig() {
  if (!currentPreset.value || !configContent.value) {return}

  // Check for existing data first
  try {
    const stats = await getTauriAPI().getNginxPresetStats(currentPreset.value.id)
    const data = stats?.data || stats
    if (data?.hasData) {
      const parts: string[] = []
      if (data.servers > 0) {parts.push(`Server ${data.servers} 个`)}
      if (data.upstreams > 0) {parts.push(`Upstream ${data.upstreams} 个`)}
      if (data.streams > 0) {parts.push(`Stream ${data.streams} 个`)}
      if (!await confirm(`该预设已有 ${parts.join('、')}，导入将覆盖现有数据。确定继续？`, { title: '确认导入', kind: 'warning' })) {return}
    }
  } catch { /* if stats fails, proceed anyway */ }

  loading.value = true
  try {
    const result = await getTauriAPI().importNginxConfig(currentPreset.value.id, configContent.value)
    const summary = result?.data || result
    toast.success(`导入完成: 基本设置 ${summary.basic_settings}，HTTP参数 ${summary.http_params}，Upstreams ${summary.upstreams}，Servers ${summary.servers}，Streams ${summary.streams}`)
    // Refresh all data + 强制重建子页面组件（它们各自重新加载数据）
    await loadPresets()
    dataVersion.value++
  } catch (err: any) {
    toast.error('导入失败: ' + (err?.message || err))
  } finally {
    loading.value = false
  }
}

// Config diff functions
async function openDeployDialog() {
  showDeployDialog.value = true
  deployComment.value = ''
  decomposeMode.value = false
  decomposedSubFiles.value = []
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
  decomposedSubFiles.value = []
  decomposeMode.value = false
}

async function computeConfigDiff() {
  if (!currentPreset.value) {
    diffError.value = '未选择预设'
    diffLoading.value = false
    return
  }
  try {
    if (decomposeMode.value) {
      const result = await getTauriAPI().generateNginxConfigDecomposed(currentPreset.value.id)
      const data = result?.data || result
      const newConfig = data?.main_config || ''
      decomposedSubFiles.value = data?.sub_files || []
      generatedNewConfig.value = newConfig
    } else {
      const result = await getTauriAPI().generateNginxConfig(currentPreset.value.id)
      const newConfig = result?.data || result || ''
      generatedNewConfig.value = newConfig
      decomposedSubFiles.value = []
    }
    const oldConfig = configContent.value || ''

    if (oldConfig === generatedNewConfig.value) {
      diffSame.value = true
      diffContent.value = ''
      return
    }

    diffContent.value = computeUnifiedDiff(oldConfig, generatedNewConfig.value)
  } catch (err: any) {
    console.error('计算配置差异失败:', err)
    diffError.value = err?.message || '计算差异失败'
  } finally {
    diffLoading.value = false
  }
}

// LCS 全量对比的 O(m×n) 表在超大配置（万行）时会卡死 UI——超限降级为 O(n) 逐行对比
const MAX_DIFF_LINES = 5000

function computeUnifiedDiff(oldText: string, newText: string): string {
  const oldLines = oldText.split('\n')
  const newLines = newText.split('\n')

  if (oldLines.length > MAX_DIFF_LINES || newLines.length > MAX_DIFF_LINES) {
    return computeLineDiff(oldLines, newLines)
  }

  // Build LCS table
  const m = oldLines.length
  const n = newLines.length
  const dp: number[][] = Array.from({ length: m + 1 }, () => Array.from({ length: n + 1 }).fill(0) as number[])
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

  // Backtrack to get diff ops - output ALL lines, not just hunks
  const result: string[] = []
  result.push('diff --git a/nginx.conf b/nginx.conf')
  result.push('--- a/nginx.conf')
  result.push('+++ b/nginx.conf')
  result.push(`@@ -1,${m} +1,${n} @@`)  // Single hunk header covering entire file

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

  // Output all lines
  for (const op of ops) {
    result.push(op.prefix + op.text)
  }

  return result.join('\n')
}

// 超限降级：前缀/后缀匹配 + 中间段整体标记（O(n)），避免 LCS 在万行配置卡死
function computeLineDiff(oldLines: string[], newLines: string[]): string {
  let start = 0
  const minLen = Math.min(oldLines.length, newLines.length)
  while (start < minLen && oldLines[start] === newLines[start]) {start++}
  let endOld = oldLines.length, endNew = newLines.length
  while (endOld > start && endNew > start && oldLines[endOld - 1] === newLines[endNew - 1]) {endOld--; endNew--}

  const result: string[] = []
  result.push('diff --git a/nginx.conf b/nginx.conf')
  result.push('--- a/nginx.conf')
  result.push('+++ b/nginx.conf')
  result.push(`@@ -1,${oldLines.length} +1,${newLines.length} @@`)
  for (let k = 0; k < start; k++) {result.push(' ' + oldLines[k])}
  for (let k = start; k < endOld; k++) {result.push('-' + oldLines[k])}
  for (let k = start; k < endNew; k++) {result.push('+' + newLines[k])}
  for (let k = endOld; k < oldLines.length; k++) {result.push(' ' + oldLines[k])}
  return result.join('\n')
}

async function onDeploy() {
  if (!deployComment.value.trim()) {return}
  // 审批开关开启时二次确认（对齐 cicd：后端需 confirmed=true 才执行）
  if (deployApproval.value) {
    const proceed = await showApprovalConfirm()
    if (!proceed) {return}
  }
  // If we generated a new config for diffing and it differs, deploy the new config
  if (generatedNewConfig.value && !diffSame.value) {
    configContent.value = generatedNewConfig.value
  }

  let result: any
  if (decomposeMode.value && decomposedSubFiles.value.length > 0) {
    // Decomposed deploy: write main config + sub-files to conf.d/
    const p = currentPreset.value
    result = await getTauriAPI().deployNginxConfigDecomposed(
      p.serverId, p.configPath, generatedNewConfig.value, decomposedSubFiles.value, deployComment.value, deployApproval.value || undefined
    )
  } else {
    result = await deployConfig(deployComment.value, deployApproval.value || undefined)
  }

  if (result?.success || result?.data?.success) {
    showDeployDialog.value = false
    deployComment.value = ''
    diffContent.value = ''
    diffSame.value = false
    diffError.value = ''
    generatedNewConfig.value = ''
    decomposedSubFiles.value = []
    decomposeMode.value = false
  } else if (result?.requiresApproval) {
    toast.warning(result.message || 'Nginx 已开启部署审批，请确认后再次部署', 5000)
  } else {
    toast.error(result?.error || result?.data?.error || '部署失败，请检查服务器连接和配置')
  }
}

async function onDecomposeChange() {
  if (!currentPreset.value) {return}
  diffLoading.value = true
  diffContent.value = ''
  diffSame.value = false
  diffError.value = ''
  generatedNewConfig.value = ''
  decomposedSubFiles.value = []
  await computeConfigDiff()
}

async function onRollback(versionId: string) {
  confirmRollbackId.value = versionId
  showRollbackConfirm.value = true
}

async function executeRollback() {
  const versionId = confirmRollbackId.value
  if (!versionId) {return}
  // 回滚是部署类操作：审批开关开启时同样需要二次确认（对齐部署流程）
  if (deployApproval.value) {
    const proceed = await showApprovalConfirm()
    if (!proceed) {return}
  }
  // 确认已由弹窗按钮触发，无需再次 confirm
  deleting.value = true
  await rollbackToVersion(versionId, deployApproval.value || undefined)
  showRollbackConfirm.value = false
  confirmRollbackId.value = ''
  deleting.value = false
}

function formatDate(dateStr: string) {
  if (!dateStr) {return ''}
  try {
    const d = new Date(dateStr)
    return d.toLocaleString('zh-CN', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
  } catch {
    return dateStr
  }
}
</script>

<style scoped>
/* highlight.js token colors — Catppuccin Mocha */
.nginx-preview :deep(.hljs-keyword) { color: #cba6f7; font-weight: 500; }
.nginx-preview :deep(.hljs-attr) { color: #89b4fa; }
.nginx-preview :deep(.hljs-string) { color: #a6e3a1; }
.nginx-preview :deep(.hljs-number) { color: #fab387; }
.nginx-preview :deep(.hljs-comment) { color: #6c7086; font-style: italic; }
.nginx-preview :deep(.hljs-variable) { color: #f38ba8; }
.nginx-preview :deep(.hljs-title) { color: #f9e2af; }
.nginx-preview :deep(.hljs-literal) { color: #fab387; }
.nginx-preview :deep(.hljs-built_in) { color: #a6e3a1; }
.nginx-preview :deep(.hljs-section) { color: #89b4fa; }
.nginx-preview code { font-family: inherit; background: transparent; padding: 0; }
</style>
