<template>
  <div class="mfa-manager">
    <!-- 页面头部 -->
    <div class="mfa-header">
      <div class="mfa-header-left">
        <div class="header-icon">🔐</div>
        <div class="header-info">
          <h2 class="mfa-title">MFA 验证码</h2>
          <p class="mfa-subtitle">双因素身份验证令牌管理器</p>
        </div>
      </div>
      <button class="btn-add" @click="showAddDialog = true">
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        添加账户
      </button>
    </div>

    <!-- 统计信息栏 -->
    <div v-if="secrets.length > 0" class="mfa-stats">
      <div class="stat-item">
        <span class="stat-value">{{ secrets.length }}</span>
        <span class="stat-label">账户总数</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value">{{ activeCodes }}</span>
        <span class="stat-label">活跃令牌</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value">{{ nextRefresh }}s</span>
        <span class="stat-label">下次刷新</span>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-if="secrets.length === 0" class="mfa-empty">
      <div class="empty-icon-wrapper">
        <svg xmlns="http://www.w3.org/2000/svg" width="80" height="80" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/>
        </svg>
      </div>
      <p class="empty-text">暂无 MFA 账户</p>
      <p class="empty-hint">点击「添加账户」录入你的第一个 MFA 密钥</p>
      <button class="btn-add-empty" @click="showAddDialog = true">
        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        添加第一个账户
      </button>
    </div>

    <!-- MFA 列表 -->
    <div v-else class="mfa-list">
      <div
        v-for="(entry, idx) in secrets"
        :key="entry.id"
        class="mfa-card"
        :class="{ 'card-expiring': remainingFor(entry) <= 5 }"
        :style="{ '--card-color': cardColor(idx) }"
        @click="copyCode(entry)"
      >
        <div class="card-color-bar"></div>
        <div class="mfa-card-top">
          <div class="mfa-card-info">
            <div class="mfa-issuer">{{ displayIssuer(entry) }}</div>
            <div v-if="entry.account" class="mfa-account">{{ entry.account }}</div>
          </div>
          <div class="mfa-card-actions">
            <button class="mfa-action-btn" @click.stop="editEntry(entry)" title="编辑">
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
            </button>
            <button class="mfa-action-btn mfa-action-delete" @click.stop="confirmDelete(entry)" title="删除">
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
            </button>
          </div>
        </div>

        <div class="mfa-card-bottom">
          <div class="mfa-code">
            <span v-if="codes[entry.id]" class="code-text">{{ codes[entry.id] }}</span>
            <span v-else class="code-text code-loading">------</span>
          </div>
          <div class="mfa-timer-ring">
            <svg viewBox="0 0 36 36" class="timer-svg">
              <circle class="timer-bg" cx="18" cy="18" r="15.5" />
              <circle
                class="timer-progress"
                cx="18" cy="18" r="15.5"
                :style="{
                  strokeDasharray: circumference,
                  strokeDashoffset: dashOffsetFor(entry),
                  stroke: remainingFor(entry) <= 5 ? 'text-error' : 'var(--card-color)'
                }"
              />
            </svg>
            <span class="timer-text">{{ remainingFor(entry) }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 复制成功提示 -->
    <Transition name="copy-toast">
      <div v-if="showCopyToast" class="copy-toast">
        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
        已复制
      </div>
    </Transition>

    <!-- 添加/编辑对话框 -->
    <Teleport to="body">
      <div v-if="showAddDialog || showEditDialog" class="mfa-overlay" @click.self="closeDialogs">
        <div class="mfa-dialog" @click.stop>
          <div class="mfa-dialog-header">
            <h3>{{ editingTarget ? '✏️ 编辑账户' : '🔑 添加 MFA 账户' }}</h3>
            <button class="mfa-dialog-close" @click="closeDialogs">×</button>
          </div>
          <div class="mfa-dialog-body">
            <div class="form-field">
              <label>otpauth:// 链接 或 Base32 密钥</label>
              <textarea
                v-model="uriInput"
                class="form-textarea uri-input"
                placeholder="otpauth://totp/... 或 Base32 密钥"
                rows="3"
                spellcheck="false"
                @input="onUriInput"
              ></textarea>
            </div>

            <div class="form-row">
              <div class="form-field">
                <label>名称 <span class="required">*</span></label>
                <input v-model="form.name" class="form-input" placeholder="GitHub、AWS..." />
              </div>
              <div class="form-field">
                <label>账户</label>
                <input v-model="form.account" class="form-input" placeholder="user@example.com" />
              </div>
            </div>

            <details class="mfa-advanced">
              <summary>高级选项</summary>
              <div class="form-row">
                <div class="form-field">
                  <label>位数</label>
                  <select v-model.number="form.digits" class="form-select">
                    <option :value="6">6 位</option>
                    <option :value="8">8 位</option>
                  </select>
                </div>
                <div class="form-field">
                  <label>周期</label>
                  <select v-model.number="form.period" class="form-select">
                    <option :value="30">30 秒</option>
                    <option :value="60">60 秒</option>
                  </select>
                </div>
                <div class="form-field">
                  <label>算法</label>
                  <select v-model="form.algorithm" class="form-select">
                    <option value="sha1">SHA1</option>
                    <option value="sha256">SHA256</option>
                    <option value="sha512">SHA512</option>
                  </select>
                </div>
              </div>
            </details>

            <div v-if="previewCode" class="mfa-preview">
              <span class="preview-label">预览验证码：</span>
              <span class="preview-code">{{ previewCode }}</span>
            </div>

            <p v-if="formError" class="mfa-error">{{ formError }}</p>
          </div>
          <div class="mfa-dialog-footer">
            <button class="btn btn-ghost" @click="closeDialogs">取消</button>
            <button class="btn btn-primary" @click="submitForm" :disabled="submitting">
              {{ submitting ? '处理中...' : (editingTarget ? '保存' : '添加') }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- 删除确认对话框 -->
    <Teleport to="body">
      <div v-if="deleteTarget" class="mfa-overlay" @click.self="deleteTarget = null">
        <div class="mfa-dialog mfa-dialog-small" @click.stop>
          <div class="mfa-dialog-header">
            <h3>⚠️ 确认删除</h3>
            <button class="mfa-dialog-close" @click="deleteTarget = null">×</button>
          </div>
          <div class="mfa-dialog-body">
            <p>确定要删除 <strong>{{ deleteTarget.name }}</strong> 吗？此操作不可撤销。</p>
          </div>
          <div class="mfa-dialog-footer">
            <button class="btn btn-ghost" @click="deleteTarget = null">取消</button>
            <button class="btn btn-danger" @click="executeDelete">删除</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { getTauriAPI } from '../utils/tauri-api'
import { useToast } from '../composables/useToast';

interface MfaEntry {
  id: string;
  name: string;
  secret: string;
  digits: number;
  period: number;
  algorithm: string;
  account?: string;
  issuer?: string;
  createdAt: string;
  updatedAt: string;
}

const toast = useToast();
const secrets = ref<MfaEntry[]>([]);
const codes = ref<Record<string, string>>({});
const now = ref(Date.now());
const showAddDialog = ref(false);
const showEditDialog = ref(false);
const uriInput = ref('');
const formError = ref('');
const submitting = ref(false);
const deleteTarget = ref<MfaEntry | null>(null);
const editingTarget = ref<MfaEntry | null>(null);
const showCopyToast = ref(false);
const previewCode = ref('');

// 统计信息
const activeCodes = computed(() => secrets.value.length);
const nextRefresh = computed(() => {
  if (secrets.value.length === 0) return 0;
  const minRemaining = Math.min(...secrets.value.map(e => remainingFor(e)));
  return Math.max(0, minRemaining);
});

const form = ref({
  name: '',
  secret: '',
  account: '',
  issuer: '',
  digits: 6,
  period: 30,
  algorithm: 'sha1',
});

let timer: ReturnType<typeof setInterval> | null = null;
let copyTimer: ReturnType<typeof setTimeout> | null = null;

// Google Authenticator 风格的卡片颜色
const CARD_COLORS = [
  '#4285f4', '#ea4335', '#fbbc04', '#34a853',
  '#ff6d01', '#46bdc6', '#7b1fa2', '#c2185b',
  '#0097a7', '#689f38', '#f57c00', '#5c6bc0',
];

function cardColor(idx: number): string {
  return CARD_COLORS[idx % CARD_COLORS.length];
}

function displayIssuer(entry: MfaEntry): string {
  return entry.issuer || entry.name;
}

// 圆形进度条
const circumference = 2 * Math.PI * 15.5;

function dashOffsetFor(entry: MfaEntry): number {
  const progress = remainingFor(entry) / entry.period;
  return circumference * (1 - progress);
}

function remainingFor(entry: MfaEntry): number {
  const epoch = Math.floor(now.value / 1000);
  return entry.period - (epoch % entry.period);
}

// 加载列表
async function loadSecrets() {
  try {
    secrets.value = await getTauriAPI().getMfaSecrets();
  } catch {
    toast.error('加载 MFA 列表失败');
  }
}

// 刷新所有验证码
async function refreshCodes() {
  for (const entry of secrets.value) {
    try {
      const result = await getTauriAPI().generateTotp(
        entry.secret, entry.digits, entry.period, entry.algorithm
      );
      if (result && result.code) {
        codes.value[entry.id] = result.code;
      }
    } catch {
      // ignore
    }
  }
}

// 复制到剪贴板
async function copyCode(entry: MfaEntry) {
  const code = codes.value[entry.id];
  if (!code) return;
  const raw = code.replace(/\s/g, '');
  try {
    await navigator.clipboard.writeText(raw);
    showCopyToast.value = true;
    if (copyTimer) clearTimeout(copyTimer);
    copyTimer = setTimeout(() => { showCopyToast.value = false }, 1500);
  } catch {
    toast.error('复制失败');
  }
}

// URI 输入时自动解析
async function onUriInput() {
  formError.value = '';
  previewCode.value = '';
  const val = uriInput.value.trim();
  if (!val) return;

  // 尝试解析 otpauth:// URI
  if (val.startsWith('otpauth://')) {
    try {
      const result: any = await getTauriAPI().parseOtpAuthUri(val);
      if (result.success) {
        form.value.name = result.issuer || result.account || '未命名';
        form.value.secret = result.secret;
        form.value.account = result.account || '';
        form.value.issuer = result.issuer || '';
        form.value.digits = result.digits || 6;
        form.value.period = result.period || 30;
        form.value.algorithm = result.algorithm || 'sha1';
        // 预览验证码
        const codeResult = await getTauriAPI().generateTotp(
          form.value.secret, form.value.digits, form.value.period, form.value.algorithm
        );
        previewCode.value = codeResult?.code || '';
      } else {
        // 解析失败，显示真实错误信息
        formError.value = result.error || 'OTP URI 解析失败';
      }
    } catch (e: any) {
      formError.value = 'OTP URI 格式无效: ' + (e?.message || '解析出错');
    }
  }
  // 否则当作 Base32 密钥处理
}

// 关闭对话框
function closeDialogs() {
  showAddDialog.value = false;
  showEditDialog.value = false;
  editingTarget.value = null;
  uriInput.value = '';
  formError.value = '';
  previewCode.value = '';
  form.value = { name: '', secret: '', account: '', issuer: '', digits: 6, period: 30, algorithm: 'sha1' };
}

// 编辑条目
function editEntry(entry: MfaEntry) {
  editingTarget.value = entry;
  showEditDialog.value = true;
  showAddDialog.value = false;
  form.value = {
    name: entry.name,
    secret: entry.secret,
    account: entry.account || '',
    issuer: entry.issuer || '',
    digits: entry.digits,
    period: entry.period,
    algorithm: entry.algorithm,
  };
  uriInput.value = entry.secret;
}

// 提交表单
async function submitForm() {
  formError.value = '';
  if (!form.value.name.trim()) {
    formError.value = '名称不能为空';
    return;
  }
  if (!form.value.secret.trim()) {
    formError.value = '密钥不能为空';
    return;
  }

  submitting.value = true;
  try {
    if (editingTarget.value) {
      // 更新
      const result: any = await getTauriAPI().updateMfaSecret(editingTarget.value.id, {
        name: form.value.name.trim(),
        account: form.value.account.trim(),
        issuer: form.value.issuer.trim(),
      });
      if (result && result.data) {
        const idx = secrets.value.findIndex(s => s.id === editingTarget.value!.id);
        if (idx !== -1) {
          secrets.value[idx] = { ...secrets.value[idx], ...result.data };
        }
        toast.success('已更新');
      }
    } else {
      // 新增
      const secretClean = form.value.secret.trim().toUpperCase().replace(/[=\s]/g, '');
      const result: any = await getTauriAPI().addMfaSecret({
        name: form.value.name.trim(),
        secret: secretClean,
        account: form.value.account.trim(),
        issuer: form.value.issuer.trim(),
        digits: form.value.digits,
        period: form.value.period,
        algorithm: form.value.algorithm,
      });
      if (result && result.success && result.data) {
        secrets.value.push(result.data);
        toast.success('MFA 密钥已添加');
        await refreshCodes();
      } else {
        formError.value = result?.error || '添加失败';
        submitting.value = false;
        return;
      }
    }
    closeDialogs();
  } catch (e: any) {
    formError.value = e.message || '操作失败';
  } finally {
    submitting.value = false;
  }
}

// 删除
function confirmDelete(entry: MfaEntry) {
  deleteTarget.value = entry;
}

async function executeDelete() {
  if (!deleteTarget.value) return;
  try {
    await getTauriAPI().deleteMfaSecret(deleteTarget.value.id);
    secrets.value = secrets.value.filter(s => s.id !== deleteTarget.value!.id);
    delete codes.value[deleteTarget.value.id];
    toast.success('已删除');
  } catch {
    toast.error('删除失败');
  }
  deleteTarget.value = null;
}

onMounted(async () => {
  await loadSecrets();
  await refreshCodes();
  timer = setInterval(() => {
    now.value = Date.now();
    refreshCodes();
  }, 1000);
});

onUnmounted(() => {
  if (timer) clearInterval(timer);
  if (copyTimer) clearTimeout(copyTimer);
});
</script>

<style scoped>
/* ======================== 容器 ======================== */
.mfa-manager {
  max-width: 1400px;
  width: 100%;
  margin: 0 auto;
  padding: 0 32px;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

/* ======================== 头部 ======================== */
.mfa-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 28px 32px;
  border-radius: 16px;
  background: linear-gradient(135deg, var(--color-primary), color-mix(in oklab, var(--color-primary) 80%, transparent), #7c3aed);
  box-shadow: 0 4px 16px rgba(136, 57, 239, 0.2);
  position: relative;
  overflow: hidden;
}

.mfa-header::before {
  content: '';
  position: absolute;
  top: -50%;
  right: -10%;
  width: 300px;
  height: 300px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.08);
  pointer-events: none;
}

.mfa-header-left {
  display: flex;
  align-items: center;
  gap: 16px;
  position: relative;
  z-index: 1;
}

.header-icon {
  font-size: 36px;
  line-height: 1;
  filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.15));
}

.header-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.mfa-title {
  font-size: 24px;
  font-weight: 700;
  color: #ffffff;
  margin: 0;
  letter-spacing: -0.5px;
}

.mfa-subtitle {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.8);
  margin: 0;
  font-weight: 400;
}

.mfa-header .btn-add {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 20px;
  font-size: 14px;
  font-weight: 500;
  border: 1.5px solid rgba(255, 255, 255, 0.35);
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.15);
  color: #ffffff;
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;
  z-index: 1;
}

.mfa-header .btn-add:hover {
  background: #ffffff;
  color: var(--color-primary);
  border-color: #ffffff;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

/* ======================== 统计栏 ======================== */
.mfa-stats {
  display: flex;
  align-items: center;
  gap: 0;
  padding: 18px 24px;
  background: var(--color-base-100);
  border-radius: 12px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.04);
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  flex: 1;
  padding: 0 16px;
}

.stat-value {
  font-size: 22px;
  font-weight: 700;
  color: var(--color-primary);
  line-height: 1.2;
}

.stat-label {
  font-size: 12px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.stat-divider {
  width: 1px;
  height: 32px;
  background: color-mix(in oklab, var(--color-base-content) 10%, transparent);
  flex-shrink: 0;
}

/* ======================== 空状态 ======================== */
.mfa-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 20px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  text-align: center;
  gap: 16px;
}

.empty-icon-wrapper {
  width: 120px;
  height: 120px;
  border-radius: 50%;
  background: linear-gradient(135deg, color-mix(in oklab, var(--color-primary) 10%, transparent), var(--color-base-200));
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 4px;
  box-shadow: 0 4px 16px rgba(136, 57, 239, 0.1);
}

.empty-icon-wrapper svg {
  opacity: 0.4;
  color: var(--color-primary);
}

.empty-text {
  font-size: 18px;
  font-weight: 600;
  margin: 0;
  color: var(--color-base-content);
}

.empty-hint {
  font-size: 14px;
  opacity: 0.7;
  margin: 0;
}

.btn-add-empty {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 12px 28px;
  font-size: 15px;
  font-weight: 500;
  border: none;
  border-radius: 10px;
  background: var(--color-primary);
  color: #ffffff;
  cursor: pointer;
  transition: all 0.2s ease;
  margin-top: 4px;
}

.btn-add-empty:hover {
  background: color-mix(in oklab, var(--color-primary) 80%, transparent);
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(136, 57, 239, 0.3);
}

/* ======================== 卡片网格 ======================== */
.mfa-list {
  display: grid !important;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)) !important;
  gap: 20px !important;
  padding: 4px !important;
  flex-direction: unset !important;
  width: 100%;
}

.mfa-card {
  background: var(--color-base-100);
  border-radius: 12px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  padding: 0;
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  user-select: none;
  display: flex;
  flex-direction: column;
  gap: 0;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
  overflow: hidden;
  position: relative;
}

.card-color-bar {
  height: 4px;
  background: linear-gradient(90deg, var(--card-color), color-mix(in oklab, var(--color-primary) 80%, transparent));
  flex-shrink: 0;
}

.mfa-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
  border-color: transparent;
}

.mfa-card.card-expiring {
  animation: cardPulse 0.5s ease infinite alternate;
}

@keyframes cardPulse {
  from { opacity: 1; }
  to { opacity: 0.65; }
}

/* 卡片内容区 */
.mfa-card > :not(.card-color-bar) {
  padding-left: 18px;
  padding-right: 18px;
}

.mfa-card-top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  padding-top: 16px;
}

.mfa-card-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.mfa-issuer {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-base-content);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.mfa-account {
  font-size: 12px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.mfa-card-actions {
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.15s ease;
}

.mfa-card:hover .mfa-card-actions {
  opacity: 1;
}

.mfa-action-btn {
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}

.mfa-action-btn:hover {
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
  color: var(--color-primary);
}

.mfa-action-delete:hover {
  background: rgba(210, 15, 57, 0.1);
  color: var(--color-error);
}

/* 底部：验证码 + 倒计时 */
.mfa-card-bottom {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 14px 18px 18px 18px;
}

.mfa-code {
  flex: 1;
  min-width: 0;
}

.code-text {
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 28px;
  font-weight: 700;
  letter-spacing: 4px;
  color: var(--color-base-content);
  text-align: center;
  display: block;
}

.code-loading {
  opacity: 0.3;
  letter-spacing: 2px;
}

/* ======================== 倒计时环 ======================== */
.mfa-timer-ring {
  position: relative;
  width: 52px;
  height: 52px;
  flex-shrink: 0;
}

.timer-svg {
  width: 100%;
  height: 100%;
  transform: rotate(-90deg);
}

.timer-bg {
  fill: none;
  stroke: color-mix(in oklab, var(--color-base-content) 10%, transparent);
  stroke-width: 3;
}

.timer-progress {
  fill: none;
  stroke-width: 3;
  stroke-linecap: round;
  transition: stroke-dashoffset 0.3s ease, stroke 0.3s ease;
}

.timer-text {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-size: 14px;
  font-weight: 700;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

/* ======================== 复制提示 ======================== */
.copy-toast {
  position: fixed;
  bottom: 32px;
  left: 50%;
  transform: translateX(-50%);
  background: var(--color-base-content);
  color: var(--color-base-100);
  padding: 10px 20px;
  border-radius: 24px;
  font-size: 14px;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 6px;
  z-index: 10001;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
}

.copy-toast-enter-active {
  animation: toastIn 0.2s ease;
}

.copy-toast-leave-active {
  animation: toastOut 0.2s ease;
}

@keyframes toastIn {
  from { opacity: 0; transform: translateX(-50%) translateY(10px); }
  to { opacity: 1; transform: translateX(-50%) translateY(0); }
}

@keyframes toastOut {
  from { opacity: 1; transform: translateX(-50%) translateY(0); }
  to { opacity: 0; transform: translateX(-50%) translateY(10px); }
}

/* ======================== 对话框 ======================== */
.mfa-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  animation: overlayIn 0.25s ease;
}

@keyframes overlayIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

.mfa-dialog {
  background: var(--color-base-100);
  border-radius: 20px;
  width: 90%;
  max-width: 520px;
  max-height: 85vh;
  overflow-y: auto;
  box-shadow: 0 25px 60px -12px rgba(0, 0, 0, 0.3);
  animation: dialogIn 0.35s cubic-bezier(0.34, 1.56, 0.64, 1);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

@keyframes dialogIn {
  from { opacity: 0; transform: scale(0.92) translateY(20px); }
  to { opacity: 1; transform: scale(1) translateY(0); }
}

.mfa-dialog-small {
  max-width: 400px;
}

.mfa-dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.mfa-dialog-header h3 {
  margin: 0;
  font-size: 17px;
  font-weight: 600;
  color: var(--color-base-content);
}

.mfa-dialog-close {
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
  transition: all 0.15s ease;
}

.mfa-dialog-close:hover {
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
  color: var(--color-primary);
}

.mfa-dialog-body {
  padding: 24px;
}

.mfa-dialog-body .form-field {
  margin-bottom: 16px;
}

.mfa-dialog-body .form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  margin-bottom: 16px;
}

.mfa-dialog-body .form-row .form-field {
  margin-bottom: 0;
}

/* URI 输入框 */
.uri-input {
  font-family: 'SF Mono', 'Fira Code', monospace;
  font-size: 12px;
  resize: vertical;
}

/* ======================== 高级选项 ======================== */
.mfa-advanced {
  margin-bottom: 16px;
  border: 1.5px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 10px;
  overflow: hidden;
  transition: border-color 0.15s ease;
}

.mfa-advanced:hover {
  border-color: var(--color-primary);
}

.mfa-advanced summary {
  padding: 12px 16px;
  font-size: 13px;
  font-weight: 500;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  cursor: pointer;
  user-select: none;
  transition: background 0.15s ease;
}

.mfa-advanced summary:hover {
  background: var(--color-base-200));
}

.mfa-advanced .form-row {
  padding: 14px 16px;
  border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.mfa-advanced .form-row .form-field {
  margin-bottom: 0;
}

/* ======================== 预览验证码 ======================== */
.mfa-preview {
  background: var(--color-base-200));
  border-radius: 10px;
  border: 1.5px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  padding: 14px 16px;
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 16px;
}

.preview-label {
  font-size: 12px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-weight: 500;
}

.preview-code {
  font-family: 'SF Mono', 'Fira Code', monospace;
  font-size: 22px;
  font-weight: 700;
  letter-spacing: 4px;
  color: var(--color-primary);
}

.mfa-error {
  color: var(--color-error);
  font-size: 13px;
  margin: 0 0 16px 0;
  padding: 10px 14px;
  background: rgba(210, 15, 57, 0.08);
  border-radius: 8px;
  border: 1px solid rgba(210, 15, 57, 0.15);
}

.mfa-dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 24px;
  border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

/* ======================== 表单覆盖 ======================== */
/* 对话框内 textarea 和 input/select 的圆角统一 */
.mfa-dialog-body .form-textarea,
.mfa-dialog-body .form-input,
.mfa-dialog-body .form-select {
  border-radius: 10px;
  border: 1.5px solid color-mix(in oklab, var(--color-base-content) 20%, transparent);
  padding: 10px 14px;
  transition: all 0.15s ease;
  outline: none;
}

.mfa-dialog-body .form-textarea:focus,
.mfa-dialog-body .form-input:focus,
.mfa-dialog-body .form-select:focus {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px color-mix(in oklab, var(--color-primary) 10%, transparent);
}

/* 对话框内按钮圆角 */
.mfa-dialog-footer .btn {
  border-radius: 10px;
  padding: 10px 22px;
}

.mfa-dialog-footer .btn-danger {
  border-radius: 10px;
  padding: 10px 22px;
}

@keyframes slideUp {
  from { opacity: 0; transform: translateY(20px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
