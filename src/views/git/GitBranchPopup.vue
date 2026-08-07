<template>
  <!-- ===== 分支管理弹窗（IDEA Branch Popup 风格） ===== -->
  <div v-if="showBranchesPopup" class="modal-overlay" @click="$emit('update:showBranchesPopup', false)">
    <div class="branches-popup" @click.stop @keydown.esc="$emit('update:showBranchesPopup', false)">
      <div class="popup-header">
        <span class="popup-title">
          <SvgIcon name="gitBranch" size="15" class="text-primary align-text-bottom mr-1" />
          分支
          <span class="text-[11px] text-base-content/50 font-normal">{{ currentBranch }}</span>
        </span>
        <button class="btn btn-ghost btn-xs btn-circle" @click="$emit('update:showBranchesPopup', false)"><SvgIcon name="x" size="14" /></button>
      </div>

      <!-- 搜索 -->
      <div class="popup-search">
        <SvgIcon name="search" size="13" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-base-content/40" />
        <input
          :value="branchSearch"
          class="search-input"
          placeholder="搜索分支..."
          @input="$emit('update:branchSearch', ($event.target as HTMLInputElement).value)"
        />
      </div>

      <!-- 操作按钮 -->
      <div class="popup-actions">
        <button class="btn btn-primary btn-sm gap-1" @click="$emit('update:showCreateBranch', true)">
          <SvgIcon name="plus" size="13" /> 新建分支
        </button>
        <button class="btn btn-outline btn-sm gap-1" @click="$emit('open-merge-dialog')">
          <SvgIcon name="gitMerge" size="13" /> 合并分支...
        </button>
      </div>

      <div class="branches-content">
        <!-- 本地分支 -->
        <div class="branch-section">
          <h4 class="section-label">本地分支 ({{ filteredLocal.length }})</h4>
          <div class="branch-list">
            <div
              v-for="b in filteredLocal"
              :key="b.name"
              class="branch-item"
              :class="{ 'branch-current': b.current }"
              :title="b.current ? '当前分支' : '双击切换'"
              @click="onClickBranch(b)"
              @dblclick="onDblClickBranch(b)"
              @contextmenu.prevent="openCtx($event, b.name, false)"
            >
              <SvgIcon name="gitBranch" size="14" class="shrink-0" :class="b.current ? 'text-primary' : 'text-base-content/40'" />
              <span class="branch-label">
                {{ b.name }}
                <SvgIcon v-if="b.current" name="star" size="10" class="text-yellow-500 align-text-bottom" />
              </span>
              <span v-if="b.upstream" class="branch-upstream-badge">
                <span v-if="b.ahead > 0" class="ahead-badge">↑{{ b.ahead }}</span>
                <span v-if="b.behind > 0" class="behind-badge">↓{{ b.behind }}</span>
              </span>
              <span v-if="b.current" class="current-badge">当前</span>
              <div class="branch-actions" v-if="!b.current">
                <button class="icon-btn" title="签出" @click.stop="$emit('checkout-branch', b.name)"><SvgIcon name="play" size="11" /></button>
                <button class="icon-btn" title="新建分支（基于此）" @click.stop="$emit('open-new-branch-from', b.name)"><SvgIcon name="gitBranch" size="11" /></button>
                <button class="icon-btn" title="合并到当前分支" @click.stop="$emit('open-merge-dialog', b.name)"><SvgIcon name="gitMerge" size="11" /></button>
                <button class="icon-btn" title="重命名" @click.stop="$emit('open-branch-rename', b.name)"><SvgIcon name="pencil" size="11" /></button>
                <button class="icon-btn danger" title="删除" @click.stop="$emit('delete-branch', b.name)"><SvgIcon name="trash" size="11" /></button>
              </div>
            </div>
            <div v-if="filteredLocal.length === 0" class="branch-empty">{{ branchSearch ? '没有匹配的分支' : '没有本地分支' }}</div>
          </div>
        </div>

        <!-- 远程分支 -->
        <div class="branch-section">
          <h4 class="section-label">远程分支 ({{ filteredRemote.length }})</h4>
          <div class="branch-list">
            <div
              v-for="b in filteredRemote"
              :key="b.name"
              class="branch-item"
              title="双击检出为本地分支"
              @dblclick="onDblClickRemote(b)"
              @contextmenu.prevent="openCtx($event, b.name, true)"
            >
              <SvgIcon name="globe" size="13" class="shrink-0 text-base-content/40" />
              <span class="branch-label">{{ b.name }}</span>
              <div class="branch-actions">
                <button class="icon-btn" title="检出为本地分支" @click.stop="$emit('checkout-remote-branch', b.name)"><SvgIcon name="play" size="11" /></button>
                <button class="icon-btn danger" title="删除远程分支" @click.stop="$emit('delete-remote-branch', b.name)"><SvgIcon name="trash" size="11" /></button>
              </div>
            </div>
            <div v-if="filteredRemote.length === 0" class="branch-empty">{{ branchSearch ? '没有匹配的分支' : '没有远程分支' }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- 分支右键菜单 -->
  <Teleport to="body">
    <div v-if="ctxMenu.show" class="fixed z-[950]" :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px' }" @click.stop @contextmenu.prevent @keydown.esc="closeCtx">
      <div class="branch-ctx-menu">
        <template v-if="!ctxMenu.isRemote">
          <button class="ctx-item" @click="$emit('checkout-branch', ctxMenu.branch); closeCtx()"><SvgIcon name="play" size="12" /> 签出</button>
          <button class="ctx-item" @click="$emit('open-new-branch-from', ctxMenu.branch); closeCtx()"><SvgIcon name="gitBranch" size="12" /> 新建分支（基于 {{ shortName(ctxMenu.branch) }}）</button>
          <div class="ctx-divider"></div>
          <button class="ctx-item" @click="$emit('open-merge-dialog', ctxMenu.branch); closeCtx()"><SvgIcon name="gitMerge" size="12" /> 合并到当前分支</button>
          <button class="ctx-item" @click="$emit('open-branch-rename', ctxMenu.branch); closeCtx()"><SvgIcon name="pencil" size="12" /> 重命名</button>
          <div class="ctx-divider"></div>
          <button class="ctx-item danger" @click="$emit('delete-branch', ctxMenu.branch); closeCtx()"><SvgIcon name="trash" size="12" /> 删除</button>
        </template>
        <template v-else>
          <button class="ctx-item" @click="$emit('checkout-remote-branch', ctxMenu.branch); closeCtx()"><SvgIcon name="play" size="12" /> 检出为本地分支</button>
          <div class="ctx-divider"></div>
          <button class="ctx-item danger" @click="$emit('delete-remote-branch', ctxMenu.branch); closeCtx()"><SvgIcon name="trash" size="12" /> 删除远程分支</button>
        </template>
      </div>
    </div>
  </Teleport>

  <!-- ===== 新建分支对话框 ===== -->
  <div v-if="showCreateBranch" class="modal-overlay" @click="$emit('update:showCreateBranch', false)">
    <div class="dialog-card" @click.stop @keydown.esc="$emit('update:showCreateBranch', false)">
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
          ref="createBranchInputRef"
        />
        <label class="form-label">基于</label>
        <select
          :value="newBranchFrom"
          class="form-input"
          @change="$emit('update:newBranchFrom', ($event.target as HTMLSelectElement).value)"
        >
          <option value="">当前分支 ({{ currentBranch }})</option>
          <option v-for="b in localBranches" :key="b.name" :value="b.name">{{ b.name }}</option>
          <option v-for="b in remoteBranches" :key="'r-' + b.name" :value="b.name">{{ b.name }}</option>
        </select>
      </div>
      <div class="dialog-actions">
        <button class="btn btn-ghost btn-sm" @click="$emit('update:showCreateBranch', false)">取消</button>
        <button class="btn btn-primary btn-sm" @click="$emit('create-branch')" :disabled="!newBranchName.trim()">创建</button>
      </div>
    </div>
  </div>

  <!-- ===== 合并对话框（IDEA Merge Branches） ===== -->
  <div v-if="showMergeDialog" class="modal-overlay" @click="$emit('update:showMergeDialog', false)">
    <div class="dialog-card" @click.stop @keydown.esc="$emit('update:showMergeDialog', false)">
      <h3 class="dialog-title">
        <SvgIcon name="gitMerge" size="14" class="text-primary align-text-bottom mr-1" />
        合并分支到 {{ currentBranch }}
      </h3>
      <p class="dialog-text">选择要合并到当前分支的分支：</p>
      <div class="merge-search">
        <SvgIcon name="search" size="13" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-base-content/40" />
        <input v-model="mergeSearch" class="search-input" placeholder="搜索分支..." />
      </div>
      <div class="merge-list">
        <div
          v-for="b in mergeCandidates"
          :key="b.name"
          class="merge-item"
          :class="{ 'merge-item-active': mergeTarget === b.name }"
          @click="$emit('update:mergeTarget', b.name)"
        >
          <SvgIcon name="gitBranch" size="13" class="shrink-0 text-base-content/40" />
          <span class="flex-1 truncate">{{ b.name }}</span>
          <span v-if="b.current" class="current-badge">当前</span>
          <span class="merge-radio" :class="{ 'merge-radio-on': mergeTarget === b.name }"></span>
        </div>
        <div v-if="mergeCandidates.length === 0" class="branch-empty">没有可合并的本地分支</div>
      </div>
      <div class="dialog-actions mt-3">
        <button class="btn btn-ghost btn-sm" @click="$emit('update:showMergeDialog', false)">取消</button>
        <button class="btn btn-primary btn-sm" @click="$emit('merge')" :disabled="!mergeTarget || merging">
          <span v-if="merging" class="loading loading-spinner loading-xs" />
          合并
        </button>
      </div>
    </div>
  </div>

  <!-- ===== 分支重命名对话框 ===== -->
  <div v-if="showBranchRenameDialog" class="modal-overlay" @click="$emit('update:showBranchRenameDialog', false)">
    <div class="dialog-card" @click.stop @keydown.esc="$emit('update:showBranchRenameDialog', false)">
      <h3 class="dialog-title">重命名分支</h3>
      <div class="dialog-form">
        <label class="form-label">当前名称</label>
        <input :value="branchRenameOld" class="form-input" disabled />
        <label class="form-label">新名称</label>
        <input
          :value="branchRenameNew"
          class="form-input"
          placeholder="输入新的分支名称..."
          @input="$emit('update:branchRenameNew', ($event.target as HTMLInputElement).value)"
          @keydown.enter="$emit('do-branch-rename')"
          spellcheck="false"
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
import { ref, computed, watch, nextTick } from 'vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'

const props = defineProps<{
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
  showMergeDialog: boolean
  branchSearch: string
}>()

const emit = defineEmits<{
  'update:showBranchesPopup': [value: boolean]
  'update:showCreateBranch': [value: boolean]
  'update:showBranchRenameDialog': [value: boolean]
  'update:newBranchName': [value: string]
  'update:newBranchFrom': [value: string]
  'update:branchRenameNew': [value: string]
  'update:mergeTarget': [value: string | null]
  'update:showMergeDialog': [value: boolean]
  'update:branchSearch': [value: string]
  'checkout-branch': [name: string]
  'create-branch': []
  'delete-branch': [name: string]
  'open-merge-dialog': [name?: string]
  'open-branch-rename': [name: string]
  'do-branch-rename': []
  'checkout-remote-branch': [name: string]
  'delete-remote-branch': [name: string]
  'open-new-branch-from': [name: string]
  'merge': []
}>()

const createBranchInputRef = ref<HTMLInputElement | null>(null)

// 右键菜单（local state）
const ctxMenu = ref<{ show: boolean; x: number; y: number; branch: string; isRemote: boolean }>({ show: false, x: 0, y: 0, branch: '', isRemote: false })
function openCtx(e: MouseEvent, branch: string, isRemote: boolean) {
  ctxMenu.value = { show: true, x: e.clientX, y: e.clientY, branch, isRemote }
  document.addEventListener('click', closeCtx, { once: true })
}
function closeCtx() {
  ctxMenu.value.show = false
}
function shortName(branch: string): string {
  const i = branch.lastIndexOf('/')
  return i >= 0 ? branch.slice(i + 1) : branch
}

// 点击本地分支：当前分支忽略；其他分支单击不切换（防误触），双击切换
function onClickBranch(b: any) {
  if (b.current) { return }
  emit('checkout-branch', b.name)
}
function onDblClickBranch(b: any) {
  if (b.current) { return }
  emit('checkout-branch', b.name)
}
function onDblClickRemote(b: any) {
  emit('checkout-remote-branch', b.name)
}

// 搜索过滤
const filteredLocal = computed(() => filterBranches(props.localBranches))
const filteredRemote = computed(() => filterBranches(props.remoteBranches))
function filterBranches(list: any[]) {
  const q = props.branchSearch.trim().toLowerCase()
  if (!q) { return list }
  return list.filter(b => b.name.toLowerCase().includes(q))
}

// 合并对话框
const mergeSearch = ref('')
const mergeCandidates = computed(() => {
  const q = mergeSearch.value.trim().toLowerCase()
  let list = props.localBranches.filter(b => !b.current)
  if (q) { list = list.filter(b => b.name.toLowerCase().includes(q)) }
  return list
})

// 打开弹窗时自动聚焦搜索框/新建输入
watch(() => props.showBranchesPopup, (v) => {
  if (v) { nextTick(() => { const el = document.querySelector('.branches-popup .search-input') as HTMLInputElement | null; el?.focus() }) }
})
watch(() => props.showCreateBranch, (v) => {
  if (v) { nextTick(() => createBranchInputRef.value?.focus()) }
})
</script>

<style>
/* ===================== 分支弹窗（IDEA 风格） ===================== */
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
  width: 480px;
  max-height: 72vh;
  background: var(--color-base-100);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 10px;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.35);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.popup-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 8%, transparent);
}

.popup-title {
  font-weight: 600;
  font-size: 14px;
  display: flex;
  align-items: center;
  gap: 4px;
}

.popup-search {
  position: relative;
  padding: 8px 14px 4px;
}

.search-input {
  width: 100%;
  padding: 6px 10px 6px 30px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 15%, transparent);
  border-radius: 6px;
  background: var(--color-base-200);
  color: var(--color-base-content);
  font-size: 13px;
  outline: none;
}
.search-input:focus {
  border-color: var(--color-primary);
}

.popup-actions {
  display: flex;
  gap: 8px;
  padding: 8px 14px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 8%, transparent);
}

.branches-content {
  flex: 1;
  overflow-y: auto;
  padding: 6px 0;
}

.branch-section {
  padding: 2px 0;
}

.section-label {
  font-size: 10px;
  font-weight: 700;
  color: color-mix(in oklab, var(--color-base-content) 45%, transparent);
  padding: 6px 16px 4px;
  text-transform: uppercase;
  letter-spacing: 0.6px;
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
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.1s;
}
.branch-item:hover {
  background: var(--color-base-200);
}
.branch-item.branch-current {
  background: color-mix(in oklab, var(--color-primary) 8%, transparent);
}

.branch-label {
  font-size: 13px;
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.current-badge {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 3px;
  background: var(--color-primary);
  color: white;
  font-weight: 600;
  flex-shrink: 0;
}

.branch-upstream-badge {
  display: inline-flex;
  gap: 2px;
  flex-shrink: 0;
}
.ahead-badge {
  font-size: 10px;
  padding: 1px 4px;
  border-radius: 3px;
  background: color-mix(in oklab, var(--color-green) 15%, transparent);
  color: var(--color-green);
  font-weight: 600;
}
.behind-badge {
  font-size: 10px;
  padding: 1px 4px;
  border-radius: 3px;
  background: color-mix(in oklab, var(--color-amber) 15%, transparent);
  color: var(--color-amber);
  font-weight: 600;
}

.branch-actions {
  display: flex;
  gap: 2px;
  opacity: 0;
  transition: opacity 0.15s;
  flex-shrink: 0;
}
.branch-item:hover .branch-actions {
  opacity: 1;
}

.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: color-mix(in oklab, var(--color-base-content) 55%, transparent);
  cursor: pointer;
  transition: all 0.12s;
}
.icon-btn:hover {
  background: color-mix(in oklab, var(--color-base-content) 10%, transparent);
  color: var(--color-base-content);
}
.icon-btn.danger:hover {
  background: color-mix(in oklab, #ef4444 15%, transparent);
  color: #ef4444;
}

.branch-empty {
  padding: 8px 16px;
  color: color-mix(in oklab, var(--color-base-content) 45%, transparent);
  font-size: 12px;
}

/* 右键菜单 */
.branch-ctx-menu {
  min-width: 180px;
  background: var(--color-base-100);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 8px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
  padding: 4px;
  display: flex;
  flex-direction: column;
}
.ctx-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border: none;
  border-radius: 5px;
  background: transparent;
  color: var(--color-base-content);
  font-size: 12.5px;
  text-align: left;
  cursor: pointer;
  white-space: nowrap;
}
.ctx-item:hover {
  background: var(--color-primary);
  color: var(--color-primary-content, #fff);
}
.ctx-item.danger:hover {
  background: #ef4444;
  color: #fff;
}
.ctx-divider {
  height: 1px;
  margin: 4px 6px;
  background: color-mix(in oklab, var(--color-base-content) 8%, transparent);
}

/* 对话框 */
.dialog-card {
  width: 460px;
  background: var(--color-base-100);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 10px;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.35);
  padding: 20px;
}
.dialog-title {
  font-size: 15px;
  font-weight: 600;
  margin: 0 0 12px;
  display: flex;
  align-items: center;
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
  border-radius: 6px;
  background: var(--color-base-200);
  color: var(--color-base-content);
  font-size: 13px;
  outline: none;
}
.form-input:focus {
  border-color: var(--color-primary);
}
.dialog-text {
  margin: 0 0 10px;
  font-size: 13px;
  line-height: 1.5;
  color: color-mix(in oklab, var(--color-base-content) 70%, transparent);
}
.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

/* 合并对话框 */
.merge-search {
  position: relative;
  margin-bottom: 8px;
}
.merge-list {
  max-height: 240px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 2px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 8px;
  padding: 6px;
}
.merge-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  transition: background 0.1s;
}
.merge-item:hover {
  background: var(--color-base-200);
}
.merge-item-active {
  background: color-mix(in oklab, var(--color-primary) 8%, transparent);
}
.merge-radio {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  border: 2px solid color-mix(in oklab, var(--color-base-content) 30%, transparent);
  flex-shrink: 0;
}
.merge-radio-on {
  border-color: var(--color-primary);
  border-width: 4px;
}
</style>
