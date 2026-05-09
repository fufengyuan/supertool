<template>
  <div class="flex flex-col gap-5 p-5">
    <!-- 页面头部 -->
    <div class="flex items-center justify-between px-5 py-4 bg-base-100 border border-base-content/10 rounded-xl">
      <div class="flex items-center gap-3">
        <span class="text-2xl">🔐</span>
        <div class="flex flex-col gap-0.5">
          <h2 class="m-0 text-lg font-bold text-base-content">MFA 验证码</h2>
          <p class="m-0 text-xs text-base-content/60">双因素身份验证令牌管理器</p>
        </div>
      </div>
      <button class="btn btn-primary btn-sm gap-1.5" @click="showAddDialog = true">
        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        添加账户
      </button>
    </div>

    <!-- 统计信息栏 -->
    <div v-if="secrets.length > 0" class="flex items-center rounded-xl border border-base-content/10 bg-base-100 px-6 py-4">
      <div class="flex flex-1 flex-col items-center gap-0.5">
        <span class="text-xl font-bold text-primary">{{ secrets.length }}</span>
        <span class="text-xs text-base-content/60">账户总数</span>
      </div>
      <div class="h-8 w-px bg-base-content/10"></div>
      <div class="flex flex-1 flex-col items-center gap-0.5">
        <span class="text-xl font-bold text-primary">{{ activeCodes }}</span>
        <span class="text-xs text-base-content/60">活跃令牌</span>
      </div>
      <div class="h-8 w-px bg-base-content/10"></div>
      <div class="flex flex-1 flex-col items-center gap-0.5">
        <span class="text-xl font-bold text-primary">{{ nextRefresh }}s</span>
        <span class="text-xs text-base-content/60">下次刷新</span>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-if="secrets.length === 0" class="flex flex-col items-center justify-center gap-4 py-20 text-center text-base-content/60">
      <div class="flex h-20 w-20 items-center justify-center rounded-full bg-base-200">
        <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="text-base-content/30">
          <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/>
        </svg>
      </div>
      <p class="m-0 text-base font-semibold text-base-content">暂无 MFA 账户</p>
      <p class="m-0 text-sm opacity-70">点击「添加账户」录入你的第一个 MFA 密钥</p>
      <button class="btn btn-primary btn-sm" @click="showAddDialog = true">
        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        添加第一个账户
      </button>
    </div>

    <!-- MFA 列表 -->
    <div v-else class="grid w-full grid-cols-[repeat(auto-fill,minmax(300px,1fr))] gap-4">
      <div
        v-for="(entry, idx) in secrets"
        :key="entry.id"
        class="group relative flex cursor-pointer select-none flex-col gap-0 overflow-hidden rounded-xl border border-base-content/10 bg-base-100 transition-all duration-200 hover:border-primary"
        :class="{ 'animate-[cardPulse_0.5s_ease_infinite_alternate]': remainingFor(entry) <= 5 }"
        :style="{ '--card-color': cardColor(idx) }"
        @click="copyCode(entry)"
      >
        <div class="h-1 flex-shrink-0" :style="{ background: `linear-gradient(90deg, var(--card-color), color-mix(in oklab, var(--color-primary) 80%, transparent))` }"></div>
        <div class="flex items-start justify-between px-4 pt-3.5">
          <div class="flex min-w-0 flex-col gap-0.5">
            <div class="truncate text-sm font-semibold text-base-content">{{ displayIssuer(entry) }}</div>
            <div v-if="entry.account" class="truncate text-xs text-base-content/60">{{ entry.account }}</div>
          </div>
          <div class="flex gap-1 opacity-0 transition-opacity duration-150 group-hover:opacity-100">
            <button class="btn btn-ghost btn-square h-6 w-6 text-base-content/60 hover:bg-primary/10 hover:text-primary min-h-0" @click.stop="editEntry(entry)" title="编辑">
              <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
            </button>
            <button class="btn btn-ghost btn-square h-6 w-6 text-base-content/60 hover:bg-error/10 hover:text-error min-h-0" @click.stop="confirmDelete(entry)" title="删除">
              <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
            </button>
          </div>
        </div>

        <div class="flex items-center justify-between gap-4 px-4 pb-4 pt-3">
          <div class="min-w-0 flex-1">
            <span v-if="codes[entry.id]" class="block text-center font-mono text-2xl font-bold tracking-widest text-base-content">{{ codes[entry.id] }}</span>
            <span v-else class="block text-center font-mono text-2xl font-bold tracking-wider text-base-content opacity-30">------</span>
          </div>
          <div class="relative h-12 w-12 flex-shrink-0">
            <svg viewBox="0 0 36 36" class="h-full w-full [transform:rotate(-90deg)]">
              <circle class="[fill:none] [stroke:color-mix(in_oklab,var(--color-base-content)_10%,transparent)] [stroke-width:3]" cx="18" cy="18" r="15.5" />
              <circle
                class="[fill:none] [stroke-width:3] [stroke-linecap:round] [transition:stroke-dashoffset_0.3s_ease,stroke_0.3s_ease]"
                cx="18" cy="18" r="15.5"
                :style="{
                  strokeDasharray: circumference,
                  strokeDashoffset: dashOffsetFor(entry),
                  stroke: remainingFor(entry) <= 5 ? 'var(--color-error)' : 'var(--card-color)'
                }"
              />
            </svg>
            <span class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 text-xs font-bold text-base-content/60">{{ remainingFor(entry) }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 复制成功提示 -->
    <Transition name="copy-toast">
      <div v-if="showCopyToast" class="fixed bottom-8 left-1/2 z-[10001] flex -translate-x-1/2 items-center gap-1.5 rounded-full bg-base-content px-5 py-2.5 text-sm font-medium text-base-100">
        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
        已复制
      </div>
    </Transition>

    <!-- 添加/编辑对话框 -->
    <Teleport to="body">
      <div v-if="showAddDialog || showEditDialog" class="fixed inset-0 z-[10000] flex items-center justify-center bg-black/45" @click.self="closeDialogs">
        <div class="w-[90%] max-w-[520px] rounded-xl border border-base-content/10 bg-base-100 shadow-lg" @click.stop>
          <div class="flex items-center justify-between border-b border-base-content/10 px-5 py-4">
            <h3 class="m-0 text-base font-semibold text-base-content">{{ editingTarget ? '✏️ 编辑账户' : '🔑 添加 MFA 账户' }}</h3>
            <button class="btn btn-ghost btn-square btn-sm text-lg text-base-content/60" @click="closeDialogs">×</button>
          </div>
          <div class="px-5 py-5">
            <div class="mb-4">
              <label class="mb-1.5 block text-sm font-medium text-base-content/70">otpauth:// 链接 或 Base32 密钥</label>
              <textarea
                v-model="uriInput"
                class="textarea textarea-bordered w-full resize-y rounded-lg font-mono text-xs"
                placeholder="otpauth://totp/... 或 Base32 密钥"
                rows="3"
                spellcheck="false"
                @input="onUriInput"
              ></textarea>
            </div>

            <div class="mb-4 grid grid-cols-2 gap-3">
              <div>
                <label class="mb-1.5 block text-sm font-medium text-base-content/70">名称 <span class="text-error">*</span></label>
                <input v-model="form.name" class="input input-bordered w-full rounded-lg" placeholder="GitHub、AWS..." />
              </div>
              <div>
                <label class="mb-1.5 block text-sm font-medium text-base-content/70">账户</label>
                <input v-model="form.account" class="input input-bordered w-full rounded-lg" placeholder="user@example.com" />
              </div>
            </div>

            <details class="mb-4 overflow-hidden rounded-lg border border-base-content/10">
              <summary class="cursor-pointer select-none px-4 py-2.5 text-xs font-medium text-base-content/60 transition-colors hover:bg-base-200">高级选项</summary>
              <div class="grid grid-cols-3 gap-3 border-t border-base-content/10 px-4 py-3">
                <div>
                  <label class="mb-1.5 block text-xs font-medium text-base-content/70">位数</label>
                  <select v-model.number="form.digits" class="select select-bordered w-full rounded-lg select-sm">
                    <option :value="6">6 位</option>
                    <option :value="8">8 位</option>
                  </select>
                </div>
                <div>
                  <label class="mb-1.5 block text-xs font-medium text-base-content/70">周期</label>
                  <select v-model.number="form.period" class="select select-bordered w-full rounded-lg select-sm">
                    <option :value="30">30 秒</option>
                    <option :value="60">60 秒</option>
                  </select>
                </div>
                <div>
                  <label class="mb-1.5 block text-xs font-medium text-base-content/70">算法</label>
                  <select v-model="form.algorithm" class="select select-bordered w-full rounded-lg select-sm">
                    <option value="sha1">SHA1</option>
                    <option value="sha256">SHA256</option>
                    <option value="sha512">SHA512</option>
                  </select>
                </div>
              </div>
            </details>

            <div v-if="previewCode" class="mb-4 flex items-center gap-2.5 rounded-lg border border-base-content/10 bg-base-200 px-4 py-3">
              <span class="text-xs text-base-content/60">预览验证码：</span>
              <span class="font-mono text-xl font-bold tracking-widest text-primary">{{ previewCode }}</span>
            </div>

            <p v-if="formError" class="m-0 mb-4 rounded-lg border border-error/15 bg-error/8 px-3 py-2 text-xs text-error">{{ formError }}</p>
          </div>
          <div class="flex justify-end gap-3 border-t border-base-content/10 px-5 py-4">
            <button class="btn btn-ghost btn-sm" @click="closeDialogs">取消</button>
            <button class="btn btn-primary btn-sm" @click="submitForm" :disabled="submitting">
              {{ submitting ? '处理中...' : (editingTarget ? '保存' : '添加') }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- 删除确认对话框 -->
    <Teleport to="body">
      <div v-if="deleteTarget" class="fixed inset-0 z-[10000] flex items-center justify-center bg-black/45" @click.self="deleteTarget = null">
        <div class="w-[90%] max-w-[400px] rounded-xl border border-base-content/10 bg-base-100 shadow-lg" @click.stop>
          <div class="flex items-center justify-between border-b border-base-content/10 px-5 py-4">
            <h3 class="m-0 text-base font-semibold text-base-content">⚠️ 确认删除</h3>
            <button class="btn btn-ghost btn-square btn-sm text-lg text-base-content/60" @click="deleteTarget = null">×</button>
          </div>
          <div class="px-5 py-5">
            <p class="text-sm m-0">确定要删除 <strong>{{ deleteTarget.name }}</strong> 吗？此操作不可撤销。</p>
          </div>
          <div class="flex justify-end gap-3 border-t border-base-content/10 px-5 py-4">
            <button class="btn btn-ghost btn-sm" @click="deleteTarget = null">取消</button>
            <button class="btn btn-error btn-sm" @click="executeDelete">删除</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { getTauriAPI } from '../../utils/tauri-api'
import { useToast } from '../../composables/useToast';

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

<style>
/* cardPulse animation for cards with ≤5s remaining */
@keyframes cardPulse {
  from { opacity: 1; }
  to { opacity: 0.65; }
}

/* copy-toast transition */
.copy-toast-enter-active {
  transition: all 0.25s cubic-bezier(0.34, 1.56, 0.64, 1);
}
.copy-toast-leave-active {
  transition: all 0.2s ease-in;
}
.copy-toast-enter-from {
  opacity: 0;
  transform: translate(-50%, 100%);
}
.copy-toast-leave-to {
  opacity: 0;
  transform: translate(-50%, 50%);
}
</style>
