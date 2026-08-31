<template>
  <!-- 首次启动引导：未配置 AI 模型时全屏展示，分步引导用户快速配置助手 -->
  <div class="fixed inset-0 z-[100] bg-base-100 text-base-content overflow-y-auto">
    <!-- 顶部步骤指示 -->
    <div class="sticky top-0 z-10 bg-base-100/90 backdrop-blur border-b border-base-content/10">
      <div class="max-w-2xl mx-auto px-6 py-4">
        <div class="flex items-center gap-2">
          <div
            v-for="(s, i) in steps"
            :key="s.key"
            class="flex items-center gap-2"
            :class="i > 0 ? 'flex-1' : ''"
          >
            <template v-if="i > 0"><div class="h-px flex-1 bg-base-content/15" /></template>
            <div
              class="flex items-center gap-1.5 text-xs font-medium whitespace-nowrap"
              :class="i <= step ? 'text-primary' : 'text-base-content/35'"
            >
              <span
                class="w-5 h-5 rounded-full grid place-items-center text-[11px] font-bold"
                :class="i < step ? 'bg-primary text-primary-content' : i === step ? 'bg-primary/15 text-primary border border-primary/40' : 'bg-base-200 text-base-content/40'"
              >
                {{ i < step ? '✓' : i + 1 }}
              </span>
              <span>{{ s.label }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="max-w-2xl mx-auto px-6 py-10">
      <!-- Step 1：欢迎 -->
      <section v-if="step === 0" class="text-center py-10">
        <div class="w-16 h-16 mx-auto mb-6 rounded-2xl bg-primary/10 grid place-items-center text-3xl">
          ⚡
        </div>
        <h1 class="text-2xl font-bold mb-3">欢迎使用 SuperTool</h1>
        <p class="text-sm text-base-content/60 leading-relaxed max-w-md mx-auto mb-8">
          一个开箱即用的桌面运维工具箱：服务器、CI/CD 部署、数据库、日志、Git、Nginx 一站式管理，
          并内置 AI 配置助手替你读配置、查原因、给建议。
        </p>

        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 mb-10 text-left">
          <div
            v-for="c in highlights"
            :key="c.title"
            class="bg-base-200/60 rounded-xl p-4"
          >
            <div class="text-2xl mb-2">{{ c.icon }}</div>
            <div class="text-sm font-semibold mb-1">{{ c.title }}</div>
            <div class="text-xs text-base-content/55 leading-relaxed">{{ c.desc }}</div>
          </div>
        </div>

        <button class="btn btn-primary btn-lg" @click="next">开始配置</button>
      </section>

      <!-- Step 2：配置 AI 模型 -->
      <section v-if="step === 1" class="py-4">
        <div class="mb-6">
          <h2 class="text-xl font-bold mb-1">配置 AI 助手模型</h2>
          <p class="text-sm text-base-content/55">
            接入一个模型，助手就能帮你读配置、查原因、给建议。支持任意 OpenAI 兼容网关与 Anthropic，
            选个常见模型可一键填充。
          </p>
        </div>

        <div class="flex flex-col gap-4">
          <!-- 快捷模板 -->
          <div>
            <span class="text-xs text-base-content/45 block mb-1.5">快捷模板（一键填充，可再微调）：</span>
            <div class="flex flex-wrap gap-1.5">
              <button
                v-for="t in MODEL_PRESETS"
                :key="t.name"
                class="btn btn-ghost btn-sm border border-base-content/15"
                :class="form.name === t.name ? 'border-primary text-primary bg-primary/5' : ''"
                :title="t.note"
                @click="applyTemplate(t)"
              >
                {{ t.name }}
              </button>
            </div>
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <label class="flex flex-col gap-1 text-xs text-base-content/60">
              提供商名称
              <input v-model="form.name" class="input input-bordered input-sm" placeholder="如：DeepSeek" />
            </label>
            <label class="flex flex-col gap-1 text-xs text-base-content/60">
              接口协议
              <select v-model="form.protocol" class="select select-bordered select-sm">
                <option value="openai">OpenAI 兼容（/chat/completions）</option>
                <option value="anthropic">Anthropic（/v1/messages）</option>
              </select>
            </label>
          </div>

          <label class="flex flex-col gap-1 text-xs text-base-content/60">
            接口地址
            <input
              v-model="form.baseUrl"
              class="input input-bordered input-sm font-mono"
              :placeholder="form.protocol === 'anthropic' ? 'https://api.anthropic.com' : 'https://your-gateway.com/v1'"
            />
            <span class="text-[11px] text-base-content/45">支持内网与本机地址（如 Ollama http://127.0.0.1:11434/v1）</span>
          </label>

          <label class="flex flex-col gap-1 text-xs text-base-content/60">
            apiKey
            <input
              v-model="form.apiKey"
              type="password"
              autocomplete="new-password"
              class="input input-bordered input-sm font-mono"
              placeholder="填入密钥（本机服务可留空）"
            />
            <span class="text-[11px] text-base-content/45">密钥只存本地（加密），不会出现在对话或日志里</span>
          </label>

          <label class="flex flex-col gap-1 text-xs text-base-content/60">
            模型 ID
            <input
              v-model="form.modelId"
              class="input input-bordered input-sm font-mono"
              placeholder="如 deepseek-v4-flash"
            />
            <span class="text-[11px] text-base-content/45">
              按网关实际命名填；已按模板填好一个，可修改。上下文窗口 {{ formatTokens(form.contextWindow) }} · 输出上限
              {{ formatTokens(form.maxOutputTokens) }}
            </span>
          </label>

          <div class="flex items-center gap-2 text-sm">
            <button class="btn btn-primary" :disabled="saving || !canSave" @click="save">
              <span v-if="saving" class="loading loading-spinner loading-sm" />
              保存并测试
            </button>
            <button
              class="btn btn-ghost btn-sm border border-base-content/15"
              :disabled="testing || !form.modelId"
              @click="test"
            >
              <span v-if="testing" class="loading loading-spinner loading-xs" />
              仅测试连接
            </button>
            <span v-if="saveError" class="text-xs text-error">{{ saveError }}</span>
          </div>

          <div
            v-if="testResult !== null"
            class="text-xs rounded-lg border px-3 py-2"
            :class="testResult.ok ? 'border-success/30 bg-success/5 text-success' : 'border-error/30 bg-error/5 text-error'"
          >
            <template v-if="testResult.ok">✓ 连接可用 · {{ testResult.latencyMs }}ms · 回复「{{ testResult.reply }}」</template>
            <template v-else>✗ {{ testResult.error }}<div class="text-base-content/55 mt-1">常见原因：协议选错、地址多了/少了 /v1、模型 ID 与网关不一致、key 无权限</div></template>
          </div>
        </div>

        <div class="flex items-center justify-between mt-8">
          <button class="btn btn-ghost btn-sm" @click="step = 0">上一步</button>
          <button class="btn btn-ghost btn-sm text-base-content/50" @click="skip">暂时跳过</button>
        </div>
      </section>

      <!-- Step 3：完成 -->
      <section v-if="step === 2" class="text-center py-10">
        <div class="w-16 h-16 mx-auto mb-6 rounded-2xl bg-success/10 grid place-items-center text-3xl">
          🎉
        </div>
        <h2 class="text-xl font-bold mb-2">
          {{ saved ? '配置完成，开始使用！' : '准备好了，随时可再配置' }}
        </h2>
        <p class="text-sm text-base-content/55 leading-relaxed max-w-md mx-auto mb-8">
          {{ saved
            ? 'AI 助手已就绪。接下来可以：添加服务器、登记 Git 仓库，或直接让助手帮你梳理配置。'
            : '模型可以稍后在「设置 → AI 模型」中补充，其余功能不受影响。' }}
        </p>

        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 mb-10 text-left">
          <button
            v-for="n in nextSteps"
            :key="n.title"
            class="bg-base-200/60 rounded-xl p-4 text-left hover:bg-base-200 transition-colors"
            @click="go(n.link)"
          >
            <div class="text-2xl mb-2">{{ n.icon }}</div>
            <div class="text-sm font-semibold mb-1">{{ n.title }}</div>
            <div class="text-xs text-base-content/55 leading-relaxed">{{ n.desc }}</div>
          </button>
        </div>

        <button class="btn btn-primary btn-lg" @click="finish">进入 SuperTool</button>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { getTauriAPI } from '@/utils/tauri-api'
import { useToast } from '@/composables/useToast'
import { MODEL_PRESETS, formatTokens } from '@/features/aiModelPresets'

const emit = defineEmits<{ (e: 'done'): void }>()

const router = useRouter()
const toast = useToast()
const api = () => getTauriAPI() as any

const step = ref(0)
const saving = ref(false)
const testing = ref(false)
const saveError = ref('')
const testResult = ref<{ ok: boolean; latencyMs?: number; reply?: string; error?: string } | null>(null)
const saved = ref(false)

const steps = [
  { key: 'welcome', label: '欢迎' },
  { key: 'model', label: '配置 AI 助手' },
  { key: 'done', label: '完成' },
]

const highlights = [
  { icon: '🖥️', title: '多端运维', desc: '服务器、SSH、健康检查、命令执行、文件传输一站式管理' },
  { icon: '🚀', title: '持续部署', desc: 'CICD 自动构建部署、回滚、多环境，前后端通用' },
  { icon: '🤖', title: 'AI 助手', desc: '读配置、查原因、给建议，改动经你确认后落地' },
]

const nextSteps = [
  { icon: '🖥️', title: '添加服务器', desc: '登记 SSH 连接，解锁日志、Nginx、备份等远程能力', link: '/servers' },
  { icon: '📦', title: '登记 Git 仓库', desc: '作为 CI/CD 部署的前置，关联本机或远程仓库', link: '/git' },
  { icon: '💬', title: '和助手聊聊', desc: '直接说目标，让它帮你梳理配置、排查问题', link: '/assistant' },
]

const form = ref({
  name: '',
  protocol: 'openai' as 'openai' | 'anthropic',
  baseUrl: 'https://',
  apiKey: '',
  modelId: '',
  contextWindow: 131072,
  maxOutputTokens: 8192,
})

const canSave = computed(() =>
  !!(form.value.name.trim() && form.value.baseUrl.trim() && form.value.modelId.trim()),
)

function applyTemplate(t: (typeof MODEL_PRESETS)[number]) {
  form.value.name = t.name
  form.value.protocol = t.protocol
  form.value.baseUrl = t.baseUrl
  const m = t.models[0]
  form.value.modelId = m?.id || ''
  form.value.contextWindow = m?.contextWindow || 131072
  form.value.maxOutputTokens = m?.maxOutputTokens || 8192
  saveError.value = ''
  testResult.value = null
}

async function save() {
  if (!canSave.value) {return}
  saving.value = true
  saveError.value = ''
  testResult.value = null
  try {
    const provider = {
      id: '',
      name: form.value.name,
      protocol: form.value.protocol,
      baseUrl: form.value.baseUrl,
      apiKey: form.value.apiKey,
      enabled: true,
      models: [{ id: form.value.modelId, contextWindow: form.value.contextWindow, maxOutputTokens: form.value.maxOutputTokens }],
    }
    await api().saveAiProvider(provider)
    saved.value = true
    toast.success('模型已配置')
    step.value = 2
  } catch (e) {
    saveError.value = String((e as Error)?.message || e)
  } finally {
    saving.value = false
  }
}

async function test() {
  if (!form.value.modelId) {return}
  testing.value = true
  testResult.value = null
  try {
    const res = await api().testAiModelRaw({ name: form.value.name, protocol: form.value.protocol, baseUrl: form.value.baseUrl, apiKey: form.value.apiKey, modelId: form.value.modelId })
    testResult.value = res.ok ? { ok: true, latencyMs: res.latencyMs, reply: res.reply } : { ok: false, error: res.error }
  } catch (e) {
    testResult.value = { ok: false, error: String((e as Error)?.message || e) }
  } finally {
    testing.value = false
  }
}

function next() { step.value = 1 }

function skip() {
  saved.value = false
  step.value = 2
}

function go(link: string) {
  finish()
  router.push(link)
}

function finish() { emit('done') }
</script>
