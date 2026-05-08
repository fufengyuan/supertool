<template>
  <div class="nginx-manager">
    <div class="nginx-layout">
      <!-- 左侧：预设列表 -->
      <div class="nginx-sidebar">
        <div class="preset-section">
          <div class="preset-header">
            <h3>🔧 Nginx 配置</h3>
            <button @click="openNewPresetForm" class="btn-add-preset">+ 新增预设</button>
          </div>

          <!-- 分组 -->
          <div
            v-for="groupEntry in groupedPresets"
            :key="groupEntry.groupName"
            class="preset-group"
          >
            <div class="preset-group-header" @click="toggleGroup(groupEntry.groupName)">
              <span class="group-toggle">{{ collapsedGroups.has(groupEntry.groupName) ? '▶' : '▼' }}</span>
              <span class="group-label">{{ groupEntry.groupName }}</span>
              <span class="group-count">{{ groupEntry.presets.length }}</span>
            </div>
            <div v-show="!collapsedGroups.has(groupEntry.groupName)" class="preset-group-body">
              <div
                v-for="preset in groupEntry.presets"
                :key="preset.id"
                class="preset-item"
                :class="{ active: currentPreset?.id === preset.id }"
                @click="onSelectPreset(preset)"
              >
                <div class="preset-info">
                  <span class="preset-name">{{ preset.name }}</span>
                  <span class="preset-meta">{{ preset.configPath || '未设置路径' }}</span>
                </div>
                <button @click.stop="openEditPresetForm(preset)" class="btn-icon" title="编辑">✏️</button>
                <button @click.stop="onDeletePreset(preset.id)" class="btn-icon btn-icon-danger" title="删除">×</button>
              </div>
            </div>
          </div>

          <div v-if="presets.length === 0" class="empty-presets">
            <template v-if="servers.length === 0">
              <p>🔌 尚未配置服务器</p>
              <p class="guide-text">Nginx 管理需要先添加 SSH 服务器</p>
            </template>
            <template v-else>
              暂无预设，点击上方按钮添加
            </template>
          </div>
        </div>
      </div>

      <!-- 右侧：配置编辑 -->
      <div class="nginx-main">
        <!-- 工具栏 -->
        <div class="nginx-toolbar">
          <div class="toolbar-left">
            <span v-if="currentPreset" class="current-preset-name">{{ currentPreset.name }}</span>
            <span v-else class="no-preset-hint">请先选择预设</span>
          </div>
          <div class="toolbar-actions">
            <button
              @click="onFetchConfig"
              :disabled="!currentPreset || loading"
              class="btn-action"
            >
              {{ loading ? '⏳ 加载中...' : '📥 获取配置' }}
            </button>
            <button
              @click="onTestConfig"
              :disabled="!currentPreset || loading"
              class="btn-action btn-secondary"
            >
              🧪 预检测试
            </button>
            <button
              @click="showDeployDialog = true"
              :disabled="!currentPreset || !configContent || loading"
              class="btn-action btn-primary"
            >
              🚀 发布
            </button>
          </div>
        </div>

        <!-- 视图模式切换 -->
        <div class="view-mode-bar">
          <button
            :class="['mode-tab', { active: viewMode === 'raw' }]"
            @click="viewMode = 'raw'"
          >📝 原始编辑</button>
        </div>

        <!-- 测试结果提示 -->
        <div v-if="testResult" class="test-result" :class="testResult.passed ? 'test-success' : 'test-error'">
          <span>{{ testResult.passed ? '✅ 配置检测通过' : '❌ 配置检测失败' }}</span>
          <span v-if="testResult.message" class="test-message">{{ testResult.message }}</span>
          <button @click="testResult = null" class="btn-icon btn-close-test">×</button>
        </div>

        <!-- 配置编辑器 -->
        <div class="config-editor">
          <textarea
            v-if="viewMode === 'raw'"
            v-model="configContent"
            :disabled="!currentPreset"
            placeholder="选择预设后点击「获取配置」加载远程 Nginx 配置..."
            class="config-textarea"
            spellcheck="false"
          ></textarea>
          <div v-else class="config-preview">
            <pre v-if="configContent">{{ configContent }}</pre>
            <div v-else class="empty-config">暂无配置内容</div>
          </div>
        </div>

        <!-- 版本历史 -->
        <div v-if="versions.length > 0" class="version-section">
          <h4>📜 版本历史</h4>
          <div class="version-list">
            <div
              v-for="version in versions"
              :key="version.id"
              class="version-item"
            >
              <div class="version-info">
                <span class="version-comment">{{ version.comment || '无备注' }}</span>
                <span v-if="version.isCurrent" class="version-current">当前生效</span>
                <span class="version-time">{{ formatDate(version.createdAt) }}</span>
                <span v-if="version.checksum" class="version-checksum">{{ version.checksum }}</span>
              </div>
              <button
                @click="onRollback(version.id)"
                :disabled="loading"
                class="btn-rollback"
                title="回滚到此版本"
              >🔄 回滚</button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 新增/编辑预设弹窗 -->
    <div v-if="showPresetForm" class="modal-overlay" @click.self="showPresetForm = false">
      <div class="modal-content">
        <h3>{{ editingPreset ? '编辑预设' : '新增预设' }}</h3>
        <div class="form-group">
          <label>预设名称</label>
          <input v-model="presetForm.name" placeholder="例如：生产环境API配置" class="form-input" />
        </div>
        <div class="form-group">
          <label>分组</label>
          <input
            v-model="presetForm.groupName"
            list="group-suggestions"
            placeholder="例如：生产 / 测试"
            class="form-input"
          />
          <datalist id="group-suggestions">
            <option value="生产" />
            <option value="测试" />
            <option value="开发" />
            <option value="预发" />
          </datalist>
        </div>
        <div class="form-group">
          <label>服务器</label>
          <div class="server-selector-wrapper">
            <GroupedServerSelector
              :servers="servers"
              :groups="serverGroups"
              v-model="presetForm.serverId"
              mode="single"
            />
          </div>
        </div>
        <div class="form-group">
          <label>配置文件路径</label>
          <input v-model="presetForm.configPath" placeholder="例如：/etc/nginx/nginx.conf" class="form-input" />
        </div>
        <div class="form-group">
          <label>描述</label>
          <textarea v-model="presetForm.description" placeholder="可选描述信息" class="form-textarea" rows="2"></textarea>
        </div>
        <div class="modal-actions">
          <button @click="showPresetForm = false" class="btn-cancel">取消</button>
          <button @click="onSavePreset" class="btn-save" :disabled="!presetForm.name || !presetForm.serverId || !presetForm.configPath">
            {{ editingPreset ? '保存' : '创建' }}
          </button>
        </div>
      </div>
    </div>

    <!-- 发布弹窗 -->
    <div v-if="showDeployDialog" class="modal-overlay" @click.self="showDeployDialog = false">
      <div class="modal-content modal-sm">
        <h3>🚀 发布配置</h3>
        <div class="form-group">
          <label>备注</label>
          <input
            v-model="deployComment"
            placeholder="请输入发布说明"
            class="form-input"
            @keyup.enter="onDeploy"
          />
        </div>
        <div class="modal-actions">
          <button @click="showDeployDialog = false" class="btn-cancel">取消</button>
          <button @click="onDeploy" class="btn-save" :disabled="!deployComment.trim()">
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

<style scoped>
.nginx-manager {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.nginx-layout {
  display: flex;
  gap: 16px;
  flex: 1;
  min-height: 0;
}

/* ── 左侧边栏 ── */
.nginx-sidebar {
  width: 260px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  overflow-y: auto;
}

.preset-section {
  background: var(--color-base-100);
  border-radius: 8px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.preset-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.preset-header h3 {
  font-size: 14px;
  font-weight: 600;
  margin: 0;
}

.btn-add-preset {
  background: var(--color-primary);
  color: white;
  border: none;
  padding: 6px 12px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 12px;
  white-space: nowrap;
}

.btn-add-preset:hover {
  opacity: 0.9;
}

/* 分组 */
.preset-group {
  margin-bottom: 4px;
}

.preset-group-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 8px;
  border-radius: 6px;
  cursor: pointer;
  user-select: none;
  transition: background 0.15s;
}

.preset-group-header:hover {
  background: color-mix(in oklab, var(--color-base-content) 5%, transparent);
}

.group-toggle {
  font-size: 10px;
  color: var(--text-secondary);
  width: 14px;
  flex-shrink: 0;
}

.group-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
  flex: 1;
}

.group-count {
  font-size: 10px;
  font-weight: 600;
  padding: 1px 6px;
  border-radius: 10px;
  background: #6c63ff22;
  color: #6c63ff;
}

.preset-group-body {
  padding: 2px 0 2px 8px;
}

.preset-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.preset-item:hover {
  background: color-mix(in oklab, var(--color-base-content) 5%, transparent);
}

.preset-item.active {
  background: var(--color-primary);
  color: white;
}

.preset-item.active .preset-meta {
  color: rgba(255, 255, 255, 0.7);
}

.preset-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.preset-name {
  font-size: 13px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.preset-meta {
  font-size: 11px;
  color: var(--text-tertiary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.btn-icon {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 14px;
  padding: 2px 4px;
  border-radius: 4px;
  opacity: 0;
  transition: opacity 0.15s;
  color: inherit;
}

.preset-item:hover .btn-icon {
  opacity: 0.7;
}

.btn-icon:hover {
  opacity: 1 !important;
}

.btn-icon-danger:hover {
  color: #ef4444;
}

.empty-presets {
  text-align: center;
  padding: 24px 16px;
  color: var(--text-secondary);
  font-size: 13px;
}

.guide-text {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 4px;
}

/* ── 右侧主内容 ── */
.nginx-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--color-base-100);
  border-radius: 8px;
  overflow: hidden;
}

.nginx-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  gap: 12px;
}

.toolbar-left {
  min-width: 0;
}

.current-preset-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.no-preset-hint {
  font-size: 13px;
  color: var(--text-tertiary);
}

.toolbar-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.btn-action {
  padding: 6px 14px;
  border-radius: 6px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 15%, transparent);
  background: var(--color-base-200);
  color: var(--text-primary);
  cursor: pointer;
  font-size: 12px;
  white-space: nowrap;
  transition: all 0.15s;
}

.btn-action:hover:not(:disabled) {
  border-color: var(--color-primary);
}

.btn-action:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-action.btn-primary {
  background: var(--color-primary);
  color: white;
  border-color: var(--color-primary);
}

.btn-action.btn-primary:hover:not(:disabled) {
  opacity: 0.9;
}

.btn-action.btn-secondary {
  background: transparent;
}

/* 视图模式切换 */
.view-mode-bar {
  display: flex;
  gap: 2px;
  padding: 6px 16px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 8%, transparent);
}

.mode-tab {
  padding: 4px 12px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 12px;
  border-radius: 4px;
  transition: all 0.15s;
}

.mode-tab:hover {
  background: color-mix(in oklab, var(--color-base-content) 5%, transparent);
}

.mode-tab.active {
  background: var(--color-primary);
  color: white;
}

/* 测试结果 */
.test-result {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  font-size: 12px;
}

.test-success {
  background: color-mix(in oklab, #22c55e 10%, transparent);
  color: #16a34a;
}

.test-error {
  background: color-mix(in oklab, #ef4444 10%, transparent);
  color: #dc2626;
}

.test-message {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.btn-close-test {
  margin-left: auto;
}

/* 配置编辑器 */
.config-editor {
  flex: 1;
  min-height: 0;
  display: flex;
  overflow: hidden;
}

.config-textarea {
  flex: 1;
  width: 100%;
  padding: 16px;
  border: none;
  outline: none;
  resize: none;
  font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace;
  font-size: 13px;
  line-height: 1.6;
  background: var(--color-base-200);
  color: var(--text-primary);
  tab-size: 4;
}

.config-textarea::placeholder {
  color: var(--text-tertiary);
}

.config-preview {
  flex: 1;
  overflow: auto;
  padding: 16px;
  background: var(--color-base-200);
}

.config-preview pre {
  font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace;
  font-size: 13px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
  margin: 0;
  color: var(--text-primary);
}

.empty-config {
  text-align: center;
  padding: 48px 16px;
  color: var(--text-tertiary);
  font-size: 14px;
}

/* 版本历史 */
.version-section {
  border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  padding: 12px 16px;
  max-height: 200px;
  overflow-y: auto;
}

.version-section h4 {
  font-size: 13px;
  font-weight: 600;
  margin: 0 0 8px;
}

.version-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.version-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 10px;
  border-radius: 6px;
  background: var(--color-base-200);
  gap: 8px;
}

.version-info {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  flex: 1;
}

.version-comment {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.version-time {
  font-size: 11px;
  color: var(--text-tertiary);
  white-space: nowrap;
}

.version-checksum {
  font-size: 10px;
  color: var(--text-tertiary);
  font-family: 'JetBrains Mono', monospace;
  background: color-mix(in oklab, var(--color-base-content) 5%, transparent);
  padding: 1px 4px;
  border-radius: 3px;
  white-space: nowrap;
}

.version-current {
  font-size: 10px;
  color: #10b981;
  background: rgba(16, 185, 129, 0.1);
  padding: 1px 6px;
  border-radius: 3px;
  font-weight: 600;
  white-space: nowrap;
}

.btn-rollback {
  padding: 3px 10px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 15%, transparent);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 11px;
  border-radius: 4px;
  white-space: nowrap;
  transition: all 0.15s;
}

.btn-rollback:hover:not(:disabled) {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.btn-rollback:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* ── 弹窗 ── */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  backdrop-filter: blur(2px);
}

.modal-content {
  background: var(--color-base-100);
  border-radius: 12px;
  padding: 24px;
  width: 480px;
  max-height: 80vh;
  overflow-y: auto;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

.modal-content.modal-sm {
  width: 380px;
}

.modal-content h3 {
  font-size: 16px;
  font-weight: 600;
  margin: 0 0 16px;
}

.form-group {
  margin-bottom: 14px;
}

.form-group label {
  display: block;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  margin-bottom: 4px;
}

.form-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 15%, transparent);
  border-radius: 6px;
  font-size: 13px;
  background: var(--color-base-200);
  color: var(--text-primary);
  outline: none;
  transition: border-color 0.15s;
  box-sizing: border-box;
}

.form-input:focus {
  border-color: var(--color-primary);
}

.form-textarea {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 15%, transparent);
  border-radius: 6px;
  font-size: 13px;
  background: var(--color-base-200);
  color: var(--text-primary);
  outline: none;
  resize: vertical;
  box-sizing: border-box;
}

.form-textarea:focus {
  border-color: var(--color-primary);
}

.server-selector-wrapper {
  max-height: 200px;
  overflow-y: auto;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 6px;
  padding: 6px;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 20px;
}

.btn-cancel {
  padding: 8px 16px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 15%, transparent);
  background: transparent;
  color: var(--text-secondary);
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
}

.btn-save {
  padding: 8px 20px;
  background: var(--color-primary);
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
}

.btn-save:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-save:hover:not(:disabled) {
  opacity: 0.9;
}
</style>
