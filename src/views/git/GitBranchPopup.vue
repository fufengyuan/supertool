<template>
  <!-- ===== 分支管理弹窗 ===== -->
  <div v-if="showBranchesPopup" class="modal-overlay" @click="$emit('update:showBranchesPopup', false)">
    <div class="branches-popup" @click.stop>
      <div class="popup-header">
        <span class="popup-title">分支管理</span>
        <button class="btn btn-ghost btn-xs" @click="$emit('update:showBranchesPopup', false)">✕</button>
      </div>

      <div class="popup-actions">
        <button class="btn btn-primary btn-sm" @click="$emit('update:showCreateBranch', true)">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
          </svg>
          新建分支
        </button>
      </div>

      <div class="branches-content">
        <div class="branch-section">
          <h4 class="section-label">本地分支</h4>
          <div class="branch-list">
            <div
              v-for="b in localBranches"
              :key="b.name"
              class="branch-item"
              :class="{ current: b.current }"
              @click="$emit('checkout-branch', b.name)"
            >
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="6" y1="3" x2="6" y2="15" />
                <circle cx="18" cy="6" r="3" />
                <circle cx="6" cy="18" r="3" />
                <path d="M18 9a9 9 0 0 1-9 9" />
              </svg>
              <span class="branch-label">{{ b.name }}</span>
              <span v-if="b.current" class="current-badge">当前</span>
              <div class="branch-actions" v-if="!b.current">
                <button class="btn btn-ghost btn-xs" @click.stop="$emit('open-branch-rename', b.name)" title="重命名分支">Rename</button>
                <button class="btn btn-ghost btn-xs" @click.stop="$emit('show-merge-dialog', b.name)" title="合并到此分支">Merge</button>
                <button class="btn btn-ghost btn-xs btn-error" @click.stop="$emit('delete-branch', b.name)" title="删除分支">✕</button>
              </div>
            </div>
          </div>
        </div>

        <div class="branch-section">
          <h4 class="section-label">远程分支</h4>
          <div class="branch-list">
            <div
              v-for="b in remoteBranches"
              :key="b.name"
              class="branch-item remote"
            >
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="18" r="3" /><circle cx="12" cy="6" r="3" />
                <line x1="12" y1="9" x2="12" y2="15" />
              </svg>
              <span class="branch-label">{{ b.name }}</span>
              <div class="branch-actions">
                <button class="btn btn-ghost btn-xs" @click.stop="$emit('checkout-remote-branch', b.name)" title="Checkout as new local branch">Checkout</button>
                <button class="btn btn-ghost btn-xs btn-error" @click.stop="$emit('delete-remote-branch', b.name)" title="删除远程分支">✕</button>
              </div>
            </div>
            <div v-if="remoteBranches.length === 0" class="branch-empty">没有远程分支</div>
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- ===== 创建分支对话框 ===== -->
  <div v-if="showCreateBranch" class="modal-overlay" @click="$emit('update:showCreateBranch', false)">
    <div class="create-branch-dialog" @click.stop>
      <h3 class="dialog-title">新建分支</h3>
      <div class="dialog-form">
        <label class="form-label">分支名称</label>
        <input
          :value="newBranchName"
          class="form-input"
          placeholder="feature/xxx"
          @input="$emit('update:newBranchName', ($event.target as HTMLInputElement).value)"
          @keydown.enter="$emit('create-branch')"
          spellcheck="false"
        />
        <label class="form-label">基于</label>
        <select
          :value="newBranchFrom"
          class="form-input"
          @change="$emit('update:newBranchFrom', ($event.target as HTMLSelectElement).value)"
        >
          <option value="">当前分支</option>
          <option v-for="b in localBranches" :key="b.name" :value="b.name">{{ b.name }}</option>
        </select>
      </div>
      <div class="dialog-actions">
        <button class="btn btn-ghost btn-sm" @click="$emit('update:showCreateBranch', false)">取消</button>
        <button class="btn btn-primary btn-sm" @click="$emit('create-branch')" :disabled="!newBranchName.trim()">创建</button>
      </div>
    </div>
  </div>

  <!-- ===== 合并确认对话框 ===== -->
  <div v-if="mergeTarget" class="modal-overlay" @click="$emit('update:mergeTarget', null)">
    <div class="merge-dialog" @click.stop>
      <h3 class="dialog-title">合并分支</h3>
      <p class="dialog-text">
        将分支 <code class="code-highlight">{{ mergeTarget }}</code> 合并到当前分支
        <code class="code-highlight">{{ currentBranch }}</code>？
      </p>
      <div class="dialog-actions">
        <button class="btn btn-ghost btn-sm" @click="$emit('update:mergeTarget', null)">取消</button>
        <button class="btn btn-primary btn-sm" @click="$emit('merge')" :disabled="merging">合并</button>
      </div>
    </div>
  </div>

  <!-- ===== 分支重命名对话框 ===== -->
  <div v-if="showBranchRenameDialog" class="modal-overlay" @click="$emit('update:showBranchRenameDialog', false)">
    <div class="branch-rename-dialog" @click.stop>
      <h3 class="dialog-title">重命名分支</h3>
      <div class="dialog-form">
        <label class="form-label">当前名称</label>
        <input :value="branchRenameOld" class="form-input" disabled />
        <label class="form-label">新名称</label>
        <input
          :value="branchRenameNew"
          class="form-input"
          placeholder="新分支名称"
          @input="$emit('update:branchRenameNew', ($event.target as HTMLInputElement).value)"
          @keydown.enter="$emit('do-branch-rename')"
          spellcheck="false"
          ref="branchRenameInputRef"
        />
      </div>
      <div class="dialog-actions">
        <button class="btn btn-ghost btn-sm" @click="$emit('update:showBranchRenameDialog', false)">取消</button>
        <button class="btn btn-primary btn-sm" @click="$emit('do-branch-rename')" :disabled="!branchRenameNew.trim()">重命名</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

defineProps<{
  showBranchesPopup: boolean
  showCreateBranch: boolean
  showBranchRenameDialog: boolean
  localBranches: any[]
  remoteBranches: any[]
  currentBranch: string
  newBranchName: string
  newBranchFrom: string
  branchRenameOld: string
  branchRenameNew: string
  mergeTarget: string | null
  merging: boolean
}>()

defineEmits<{
  'update:showBranchesPopup': [value: boolean]
  'update:showCreateBranch': [value: boolean]
  'update:showBranchRenameDialog': [value: boolean]
  'update:newBranchName': [value: string]
  'update:newBranchFrom': [value: string]
  'update:branchRenameNew': [value: string]
  'update:mergeTarget': [value: string | null]
  'checkout-branch': [name: string]
  'create-branch': []
  'delete-branch': [name: string]
  'show-merge-dialog': [name: string]
  'open-branch-rename': [name: string]
  'do-branch-rename': []
  'checkout-remote-branch': [name: string]
  'delete-remote-branch': [name: string]
  'merge': []
}>()

const branchRenameInputRef = ref<HTMLInputElement | null>(null)
</script>

<style>
/* ===================== 分支管理弹窗 — 从 GitManager.vue 提取 ===================== */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  z-index: 900;
  display: flex;
  align-items: center;
  justify-content: center;
}

.branches-popup {
  width: 420px;
  max-height: 70vh;
  background: var(--color-base-100);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.popup-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.popup-title {
  font-weight: 600;
  font-size: 14px;
}

.popup-actions {
  padding: 8px 16px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.branches-content {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.branch-section {
  padding: 4px 0;
}

.section-label {
  font-size: 11px;
  font-weight: 600;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  padding: 4px 16px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin: 0;
}

.branch-list {
  padding: 0 8px;
}

.branch-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 8px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.1s;
}

.branch-item:hover {
  background: var(--hover-bg);
}

.branch-item.current {
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
}

.branch-item.remote {
  opacity: 0.8;
  cursor: default;
}

.branch-label {
  font-size: 13px;
  flex: 1;
}

.current-badge {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 3px;
  background: var(--color-primary);
  color: white;
  font-weight: 600;
}

.branch-actions {
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.15s;
}

.branch-item:hover .branch-actions {
  opacity: 1;
}

.branch-empty {
  padding: 8px 16px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-size: 12px;
}

/* ===================== 对话框 ===================== */
.create-branch-dialog,
.merge-dialog {
  width: 480px;
  background: var(--color-base-100);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  padding: 20px;
}

.dialog-title {
  font-size: 16px;
  font-weight: 600;
  margin: 0 0 16px;
}

.dialog-form {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 16px;
}

.form-label {
  font-size: 12px;
  font-weight: 600;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.form-input {
  padding: 8px 10px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 20%, transparent);
  border-radius: 4px;
  background: var(--color-base-200);
  color: var(--color-base-content);
  font-size: 13px;
  outline: none;
}

.form-input:focus {
  border-color: var(--color-primary);
}

.dialog-text {
  margin: 0 0 16px;
  font-size: 13px;
  line-height: 1.5;
}

.code-highlight {
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
  color: var(--color-primary);
  padding: 1px 4px;
  border-radius: 3px;
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  font-size: 12px;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

/* ===================== 分支重命名对话框 ===================== */
.branch-rename-dialog {
  width: 380px;
  background: var(--color-base-100);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  padding: 20px;
}

/* ===================== 错误按钮 ===================== */
.btn-error {
  color: #ef4444;
}

.btn-error:hover {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

/* ===================== 滚动条 ===================== */
.branches-content::-webkit-scrollbar-track {
  background: transparent;
}

.branches-content::-webkit-scrollbar-thumb {
  background: color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 3px;
}

.branches-content::-webkit-scrollbar-thumb:hover {
  background: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}
</style>
