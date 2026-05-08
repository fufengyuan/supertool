<template>
  <div class="form-modal-overlay" @click="$emit('close')">
    <div class="form-modal" @click.stop>
      <div class="form-modal-header">
        <h3>{{ isEditing ? '✏️ 编辑服务器' : '🖥️ 添加服务器' }}</h3>
        <button @click="$emit('close')" class="form-modal-close">×</button>
      </div>
      <div class="form-modal-body">
        <div class="form-row">
          <div class="form-field">
            <label>服务器名称 <span class="required">*</span></label>
            <input v-model="localForm.name" class="form-input" placeholder="生产服务器" />
          </div>
          <div class="form-field">
            <label>端口</label>
            <input v-model.number="localForm.port" type="number" class="form-input" placeholder="22" />
          </div>
        </div>

        <div class="form-field">
          <label>主机地址</label>
          <input v-model="localForm.host" class="form-input" placeholder="192.168.1.100" />
        </div>

        <div class="form-field">
          <label>用户名</label>
          <input v-model="localForm.username" class="form-input" placeholder="root" />
        </div>

        <div class="form-section-card">
          <div class="form-section-title">
            <span class="icon">🔑</span>
            <span>认证方式</span>
          </div>

          <div class="form-field">
            <label>SSH Key 路径</label>
            <input v-model="localForm.sshKeyPath" class="form-input" placeholder="~/.ssh/id_rsa" />
            <small>推荐使用 SSH Key 认证，更安全</small>
          </div>

          <div class="form-field">
            <label>密码</label>
            <input
              v-model="localForm.password"
              type="password"
              class="form-input"
              autocomplete="off"
              :placeholder="isEditing ? '留空则保留原密码' : '留空则使用 Key 认证'"
            />
          </div>
        </div>

        <div class="form-field">
          <label>分组</label>
          <div class="tree-select-wrapper">
            <div class="tree-select-trigger" @click="showTreeSelect = !showTreeSelect">
              <span v-if="localForm.groupId" class="selected-group">
                <span class="group-dot" :style="{ background: selectedGroupColor }"></span>
                {{ selectedGroupName }}
              </span>
              <span v-else class="placeholder">选择分组...</span>
              <svg class="select-chevron" :class="{ open: showTreeSelect }" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.5">
                <polyline points="6 9 12 15 18 9"/>
              </svg>
            </div>
            <Transition name="tree-dropdown">
              <div v-if="showTreeSelect" class="tree-select-dropdown">
                <div class="tree-option" :class="{ active: !localForm.groupId }" @click="selectGroup(null)">
                  无分组
                </div>
                <div v-for="g in sortedGroups" :key="g.id"
                  class="tree-option"
                  :class="{ active: localForm.groupId === g.id }"
                  :style="{ paddingLeft: `${12 + g.depth * 20}px` }"
                  @click="selectGroup(g.id)">
                  <span v-if="g.depth > 0" class="tree-indent-line">└</span>
                  <span class="tree-option-dot" :style="{ background: g.color || '#6c63ff' }"></span>
                  <span class="tree-option-name">{{ g.name }}</span>
                </div>
              </div>
            </Transition>
          </div>
        </div>

        <div class="form-field">
          <label>标签（逗号分隔）</label>
          <input v-model="localForm.tagsInput" class="form-input" placeholder="生产, Web, API" />
        </div>

        <div class="form-field">
          <label>描述</label>
          <textarea
            v-model="localForm.description"
            class="form-textarea"
            placeholder="服务器描述..."
            rows="2"
          ></textarea>
        </div>

        <div class="form-section-card approval-section">
          <div class="form-section-title">
            <span class="icon">🛡️</span>
            <span>安全管控</span>
          </div>
          <div class="approval-toggle-row">
            <div class="approval-info">
              <div class="approval-label">🔒 执行审核</div>
              <div class="approval-desc">开启后，CLI 无法在此服务器执行命令，GUI 执行需人工确认</div>
            </div>
            <label class="toggle-switch">
              <input type="checkbox" v-model="localForm.requiresApproval" />
              <span class="toggle-slider"></span>
            </label>
          </div>
        </div>
      </div>

      <div class="form-modal-footer">
        <button @click="$emit('test-connection')" class="btn btn-ghost">
          <svg
            viewBox="0 0 24 24"
            width="14"
            height="14"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
            <polyline points="22 4 12 14.01 9 11.01" />
          </svg>
          测试连接
        </button>
        <button @click="$emit('close')" class="btn btn-ghost">取消</button>
        <button @click="$emit('save')" class="btn btn-primary">保存</button>
      </div>

      <div
        v-if="testResult"
        class="test-result"
        :class="testResult.success ? 'success' : 'error'"
        style="margin: 0 24px 20px"
      >
        {{ testResult.success ? '✅ 连接成功！' : '❌ 连接失败: ' + testResult.error }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed, onMounted, onUnmounted } from 'vue';

interface ServerGroup {
  id: string;
  name: string;
  color: string;
  parentId?: string | null;
}

interface TestResult {
  success: boolean;
  error?: string;
}

const props = defineProps<{
  form: Record<string, unknown> & { name?: string; host?: string; port?: number; username?: string; password?: string; sshKeyPath?: string; tagsInput?: string; description?: string; groupId?: string | null; tags?: string[]; requiresApproval?: boolean };
  isEditing?: boolean;
  testResult: TestResult | null;
  groups: ServerGroup[];
}>();

const emit = defineEmits(['close', 'test-connection', 'save', 'update:form']);

// Local reactive wrapper to avoid mutating props directly
const localForm = ref({ ...props.form });

// Sync localForm changes back to parent — batch rapid input events
let emitTimer: ReturnType<typeof setTimeout> | null = null;
watch(localForm, (newVal) => {
  if (emitTimer) clearTimeout(emitTimer);
  emitTimer = setTimeout(() => {
    emit('update:form', { ...newVal });
  }, 16);
}, { deep: true });

// Sync prop changes back to localForm (e.g., when parent resets form)
watch(() => props.form, (newVal) => {
  localForm.value = { ...newVal };
});

// Tree select state
const showTreeSelect = ref(false);

// Close dropdown when clicking outside
function handleClickOutside(e: MouseEvent) {
  const target = e.target as HTMLElement;
  if (!target.closest('.tree-select-wrapper')) {
    showTreeSelect.value = false;
  }
}
onMounted(() => document.addEventListener('click', handleClickOutside));
onUnmounted(() => document.removeEventListener('click', handleClickOutside));

// Build sorted flat list with depth for tree display
const sortedGroups = computed(() => {
  const result: Array<ServerGroup & { depth: number }> = [];
  function walk(parentId: string | null, depth: number) {
    const children = props.groups.filter(g => (g.parentId || null) === parentId);
    for (const child of children) {
      result.push({ ...child, depth });
      walk(child.id, depth + 1);
    }
  }
  walk(null, 0);
  return result;
});

const selectedGroupName = computed(() => {
  const g = props.groups.find(g => g.id === localForm.value.groupId);
  return g?.name || '';
});

const selectedGroupColor = computed(() => {
  const g = props.groups.find(g => g.id === localForm.value.groupId);
  return g?.color || '#6c63ff';
});

function selectGroup(groupId: string | null) {
  localForm.value.groupId = groupId;
  showTreeSelect.value = false;
}
</script>

<style scoped>
.form-modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.form-modal {
  background: var(--color-base-100);
  border-radius: 16px;
  width: 90%;
  max-width: 640px;
  max-height: 85vh;
  overflow-y: auto;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
}

.form-modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.form-modal-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--color-base-content);
}

.form-modal-close {
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-size: 20px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.form-modal-body {
  padding: 24px;
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.form-field {
  margin-bottom: 16px;
}

.form-field label {
  display: block;
  margin-bottom: 6px;
  font-size: 13px;
  font-weight: 500;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.form-field .required {
  color: var(--color-error);
}

.form-field small {
  display: block;
  margin-top: 4px;
  font-size: 12px;
  color: color-mix(in oklab, var(--color-base-content) 40%, transparent);
}

.form-input,
.form-textarea {
  width: 100%;
  padding: 10px 12px;
  border: 1.5px solid color-mix(in oklab, var(--color-base-content) 20%, transparent);
  border-radius: 8px;
  background: var(--color-base-200);
  color: var(--color-base-content);
  font-size: 14px;
  transition: border-color 0.15s ease;
}

.form-input:focus,
.form-textarea:focus {
  outline: none;
  border-color: var(--color-primary);
}

.form-select {
  cursor: pointer;
}

/* 树状分组选择器 */
.tree-select-wrapper {
  position: relative;
}

.tree-select-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border: 1.5px solid color-mix(in oklab, var(--color-base-content) 20%, transparent);
  border-radius: 8px;
  background: var(--color-base-200);
  cursor: pointer;
  transition: border-color 0.15s ease;
  min-height: 42px;
}

.tree-select-trigger:hover {
  border-color: var(--color-primary);
}

.selected-group {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  color: var(--color-base-content);
}

.group-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}

.placeholder {
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-size: 14px;
  opacity: 0.6;
}

.select-chevron {
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  transition: transform 0.2s ease;
  flex-shrink: 0;
}

.select-chevron.open {
  transform: rotate(180deg);
}

.tree-select-dropdown {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  background: var(--color-base-100);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 10px;
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.15);
  z-index: 100;
  max-height: 280px;
  overflow-y: auto;
  padding: 6px;
}

.tree-option {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  color: var(--color-base-content);
  transition: background 0.12s ease;
  user-select: none;
}

.tree-option:hover {
  background: var(--color-base-200);
}

.tree-option.active {
  background: rgba(108, 99, 255, 0.1);
  color: var(--color-primary);
  font-weight: 500;
}

.tree-indent-line {
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  opacity: 0.4;
  font-size: 12px;
  flex-shrink: 0;
  width: 14px;
  text-align: center;
}

.tree-option-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.tree-option-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 下拉动画 */
.tree-dropdown-enter-active,
.tree-dropdown-leave-active {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.tree-dropdown-enter-from,
.tree-dropdown-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}

.form-textarea {
  resize: vertical;
  font-family: inherit;
}

.form-section-card {
  background: var(--color-base-200);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 12px;
  padding: 16px;
  margin-bottom: 16px;
}

.form-section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 600;
  color: var(--color-base-content);
  margin-bottom: 16px;
}

.form-section-title .icon {
  font-size: 16px;
}

.form-modal-footer {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  padding: 16px 24px;
  border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.test-result {
  padding: 12px;
  border-radius: 8px;
  font-size: 14px;
  margin: 0 24px 20px;
}

.test-result.success {
  background: rgba(var(--success-rgb, 166, 227, 161), 0.15);
  color: var(--color-success);
}

.test-result.error {
  background: rgba(var(--danger-rgb, 243, 139, 168), 0.15);
  color: var(--color-error);
}

/* 安全管控区域 */
.approval-section {
  border-color: rgba(250, 179, 135, 0.3);
  background: rgba(250, 179, 135, 0.05);
}

.approval-toggle-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.approval-info {
  flex: 1;
}

.approval-label {
  font-size: 14px;
  font-weight: 600;
  color: #fab387;
  margin-bottom: 4px;
}

.approval-desc {
  font-size: 12px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  line-height: 1.4;
}

/* Toggle switch */
.toggle-switch {
  position: relative;
  display: inline-block;
  width: 44px;
  height: 24px;
  flex-shrink: 0;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: color-mix(in oklab, var(--color-base-content) 20%, transparent);
  transition: 0.3s;
  border-radius: 24px;
}

.toggle-slider:before {
  position: absolute;
  content: "";
  height: 18px;
  width: 18px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  transition: 0.3s;
  border-radius: 50%;
}

.toggle-switch input:checked + .toggle-slider {
  background-color: #fab387;
}

.toggle-switch input:checked + .toggle-slider:before {
  transform: translateX(20px);
}
</style>
