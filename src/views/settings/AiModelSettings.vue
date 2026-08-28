<template>
  <div class="space-y-4">
    <!-- 说明 -->
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-5">
      <h2 class="flex items-center gap-2 text-base font-semibold text-base-content m-0 mb-2">
        <SvgIcon name="brain" size="16" /> AI 模型
      </h2>
      <p class="text-sm text-base-content/60 m-0 leading-relaxed">
        给「AI 配置助手」接入模型。支持任意 OpenAI 兼容网关（<code>/chat/completions</code>）与
        Anthropic（<code>/v1/messages</code>）两种协议，模型 ID 与上下文窗口都可自定义；
        接口地址允许内网/本机（如 <code>http://127.0.0.1:11434/v1</code>）。
        常见模型可用下方的「快捷模板」一键填充，上下文/输出上限可点快捷档位选择（1M / 256K / 128K…）。
        apiKey 用 AES-256-GCM 加密后存在本地库，界面与助手永远只会看到掩码。
      </p>
    </div>

    <!-- 当前使用 -->
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-5">
      <div class="flex items-center justify-between mb-3">
        <h3 class="text-sm font-semibold text-base-content m-0">当前使用的模型</h3>
        <button class="btn btn-primary btn-sm gap-1" @click="openForm(null)">
          <SvgIcon name="plus" size="13" /> 新增提供商
        </button>
      </div>
      <div v-if="!providers.length" class="text-sm text-base-content/50 leading-relaxed">
        还没有配置模型。点「新增提供商」，填名称 + 协议 + 接口地址 + apiKey，再加至少一个模型 ID 就能用。
      </div>
      <select
        v-else
        class="select select-bordered select-sm w-full max-w-md font-mono text-xs"
        :value="activeKey"
        @change="onPickActive"
      >
        <option value="" disabled>请选择要使用的模型</option>
        <option v-for="opt in modelOptions" :key="opt.value" :value="opt.value">
          {{ opt.label }}
        </option>
      </select>
      <p v-if="activeModel" class="text-[11px] text-base-content/50 mt-2 m-0">
        上下文窗口 {{ formatTokens(activeModel.contextWindow) }} tokens（{{ activeModel.contextWindow }}）· 单次输出上限
        {{ formatTokens(activeModel.maxOutputTokens) }} tokens ·
        <span :class="activeModel.vision ? 'text-success' : 'text-base-content/45'">
          {{ activeModel.vision ? '支持识图，可粘贴截图' : '未开启识图' }}
        </span>
        · 决定助手能带多少历史，配太小会频繁截断上下文
      </p>
    </div>

    <!-- 提供商列表 -->
    <div
      v-for="p in providers"
      :key="p.id"
      class="bg-base-100 border border-base-content/10 rounded-xl p-5"
    >
      <div class="flex items-start gap-2 flex-wrap">
        <div class="min-w-0 flex-1">
          <div class="flex items-center gap-2 flex-wrap">
            <span class="text-sm font-semibold text-base-content">{{ p.name }}</span>
            <span class="text-[10px] px-1.5 py-0.5 rounded-full"
              :class="p.protocol === 'anthropic' ? 'bg-purple-500/10 text-purple-500' : 'bg-primary/10 text-primary'">
              {{ p.protocol === 'anthropic' ? 'Anthropic' : 'OpenAI 兼容' }}
            </span>
            <span v-if="!p.enabled" class="text-[10px] px-1.5 py-0.5 rounded-full bg-base-content/10 text-base-content/50">
              已停用
            </span>
          </div>
          <div class="text-[11px] text-base-content/50 mt-1 font-mono break-all">
            {{ p.baseUrl }} · key {{ p.apiKeyMasked || '（未设置）' }}
          </div>
        </div>
        <div class="flex items-center gap-1 shrink-0">
          <button class="btn btn-ghost btn-xs border border-base-content/10" @click="openForm(p)">编辑</button>
          <button class="btn btn-ghost btn-xs text-error hover:bg-error/10" @click="remove(p)">删除</button>
        </div>
      </div>

      <div class="mt-3 overflow-x-auto">
        <table class="w-full text-[11px]">
          <thead>
            <tr class="text-left text-base-content/50 border-b border-base-content/10">
              <th class="py-1.5 pr-3 font-medium">模型 ID</th>
              <th class="py-1.5 pr-3 font-medium">显示名</th>
              <th class="py-1.5 pr-3 font-medium">上下文窗口</th>
              <th class="py-1.5 pr-3 font-medium">输出上限</th>
              <th class="py-1.5 font-medium">状态</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="m in p.models || []" :key="m.id" class="border-b border-base-content/5 last:border-b-0">
              <td class="py-1.5 pr-3 font-mono">{{ m.id }}</td>
              <td class="py-1.5 pr-3">{{ m.label || m.id }}</td>
              <td class="py-1.5 pr-3 font-mono">{{ m.contextWindow }}</td>
              <td class="py-1.5 pr-3 font-mono">{{ m.maxOutputTokens }}</td>
              <td class="py-1.5">
                <span v-if="isActive(p, m)" class="text-success font-semibold">使用中</span>
                <div v-else class="flex items-center gap-1">
                  <button class="btn btn-ghost btn-xs px-1.5" @click="setActive(p, m)">设为当前</button>
                  <button class="btn btn-ghost btn-xs px-1.5" :disabled="testing === testKey(p, m)" @click="test(p, m)">
                    {{ testing === testKey(p, m) ? '测试中…' : '测试' }}
                  </button>
                </div>
              </td>
            </tr>
            <tr v-if="!(p.models || []).length">
              <td colspan="5" class="py-2 text-base-content/40">这个提供商还没有模型，点「编辑」添加模型 ID</td>
            </tr>
          </tbody>
        </table>
      </div>

      <div
        v-for="m in p.models || []"
        :key="'r-' + m.id"
        v-show="results[testKey(p, m)]"
        class="mt-2 text-[11px] px-2.5 py-2 rounded-lg border"
        :class="results[testKey(p, m)]?.ok ? 'border-success/30 bg-success/5 text-success' : 'border-error/30 bg-error/5 text-error'"
      >
        <template v-if="results[testKey(p, m)]?.ok">
          可用 · {{ results[testKey(p, m)].latencyMs }}ms · 回复「{{ results[testKey(p, m)].reply }}」
        </template>
        <template v-else>
          {{ results[testKey(p, m)]?.error || '测试失败' }}
          <div class="text-base-content/60 mt-1">
            常见原因：协议选错（网关只认 OpenAI 格式却选了 Anthropic）、接口地址多了/少了
            <code>/v1</code>、模型 ID 与网关不一致、key 无该模型权限。
          </div>
        </template>
      </div>
    </div>

    <!-- 编辑表单 -->
    <div v-if="form" class="bg-base-100 border border-primary/30 rounded-xl p-5">
      <h3 class="text-sm font-semibold text-base-content m-0 mb-3">
        {{ form.id ? '编辑提供商' : '新增提供商' }}
      </h3>
      <div class="flex flex-col gap-3">
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <label class="flex flex-col gap-1 text-xs text-base-content/60">
            名称
            <input v-model="form.name" class="input input-bordered input-sm" placeholder="如：公司内网网关" />
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
          <span class="text-[11px] text-base-content/45">
            填到 <code v-if="form.protocol === 'anthropic'">/v1</code>
            <code v-else>/v1</code> 这一层即可，请求端点由工具自动拼接；支持内网与本机地址
          </span>
        </label>

        <label class="flex flex-col gap-1 text-xs text-base-content/60">
          apiKey
          <input
            v-model="form.apiKey"
            type="password"
            autocomplete="new-password"
            class="input input-bordered input-sm font-mono"
            :placeholder="form.id ? `已存 ${existingMask}，留空表示不修改` : '填入密钥（本机服务可留空）'"
          />
          <span v-if="form.id" class="flex items-center gap-1.5 text-[11px]">
            <label class="flex items-center gap-1 cursor-pointer">
              <input v-model="clearKey" type="checkbox" class="checkbox checkbox-xs" /> 清除已存密钥
            </label>
            <span class="text-base-content/45">密钥只存本地（加密），不会出现在对话或日志里</span>
          </span>
        </label>

        <div>
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs text-base-content/60">模型（模型 ID 按网关实际命名自由填）</span>
            <button class="btn btn-ghost btn-xs" @click="addModel">
              <SvgIcon name="plus" size="12" /> 加一行
            </button>
          </div>

          <!-- 快捷模板：只在新增时展示，避免编辑时覆盖已有配置 -->
          <div v-if="!form.id" class="mb-3">
            <span class="text-[11px] text-base-content/45 block mb-1">
              快捷模板（一键填充协议/地址/模型，版本更新快，按实际网关调整）：
            </span>
            <div class="flex flex-wrap gap-1.5">
              <button
                v-for="t in MODEL_PRESETS"
                :key="t.name"
                class="btn btn-ghost btn-xs border border-base-content/15"
                :title="t.note"
                @click="applyTemplate(t)"
              >
                {{ t.name }}
              </button>
            </div>
          </div>

          <div class="flex flex-col gap-3">
            <div
              v-for="(m, i) in form.models"
              :key="i"
              class="border border-base-content/10 rounded-lg p-3 flex flex-col gap-2.5"
            >
              <div class="flex items-center justify-between">
                <span class="text-[11px] font-semibold text-base-content/70">模型 #{{ i + 1 }}</span>
                <button class="btn btn-ghost btn-xs text-error" title="删除该模型" @click="form.models.splice(i, 1)">
                  <SvgIcon name="x" size="12" /> 删除
                </button>
              </div>

              <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5">
                <label class="flex flex-col gap-1 text-xs text-base-content/60">
                  模型 ID（必填）
                  <input
                    v-model="m.id"
                    class="input input-bordered input-sm font-mono text-xs"
                    placeholder="如 deepseek-chat"
                  />
                </label>
                <label class="flex flex-col gap-1 text-xs text-base-content/60">
                  显示名（可空）
                  <input
                    v-model="m.label"
                    class="input input-bordered input-sm text-xs"
                    placeholder="如 DeepSeek Chat"
                  />
                </label>
              </div>

              <label class="flex flex-col gap-1 text-xs text-base-content/60">
                上下文窗口（tokens）
                <div class="flex items-center gap-2">
                  <input
                    v-model.number="m.contextWindow"
                    type="number"
                    min="512"
                    class="input input-bordered input-sm font-mono text-xs w-36"
                  />
                  <span class="text-[11px] text-base-content/45 font-mono shrink-0">
                    {{ formatTokens(m.contextWindow) }}
                  </span>
                </div>
                <div class="flex flex-wrap gap-1">
                  <button
                    v-for="p in CONTEXT_PRESETS"
                    :key="p.label"
                    class="btn btn-xs px-2 py-0.5 h-auto min-h-0"
                    :class="Number(m.contextWindow) === p.value ? 'btn-primary text-primary-content' : 'btn-ghost border border-base-content/15'"
                    @click="setContextPreset(m, p)"
                  >
                    {{ p.label }}
                  </button>
                </div>
              </label>

              <label class="flex flex-col gap-1 text-xs text-base-content/60">
                单次输出上限（tokens）
                <div class="flex items-center gap-2">
                  <input
                    v-model.number="m.maxOutputTokens"
                    type="number"
                    min="1"
                    class="input input-bordered input-sm font-mono text-xs w-36"
                  />
                  <span class="text-[11px] text-base-content/45 font-mono shrink-0">
                    {{ formatTokens(m.maxOutputTokens) }}
                  </span>
                </div>
                <div class="flex flex-wrap gap-1">
                  <button
                    v-for="p in OUTPUT_PRESETS"
                    :key="p.label"
                    class="btn btn-xs px-2 py-0.5 h-auto min-h-0"
                    :class="Number(m.maxOutputTokens) === p.value ? 'btn-primary text-primary-content' : 'btn-ghost border border-base-content/15'"
                    @click="setOutputPreset(m, p)"
                  >
                    {{ p.label }}
                  </button>
                </div>
              </label>

              <label class="flex items-center gap-1.5 text-xs text-base-content/60 cursor-pointer">
                <input v-model="m.vision" type="checkbox" class="toggle toggle-xs toggle-primary" />
                支持识图
                <span class="text-[10px] text-base-content/45">开启后可给助手粘贴截图/图片排查问题</span>
              </label>
            </div>

            <p class="text-[11px] text-base-content/45 m-0 leading-relaxed">
              上下文窗口决定助手能带多少历史（太小会频繁裁剪、太大浪费额度），主流模型普遍 128K~1M；
              输出上限会被自动收敛到窗口以内。窗口/输出可点上方快捷档位，也可以直接手动输入。
            </p>
          </div>
        </div>

        <label class="flex items-center gap-2 text-xs text-base-content/60">
          <input v-model="form.enabled" type="checkbox" class="toggle toggle-primary toggle-sm" /> 启用该提供商
        </label>

        <div class="flex items-center gap-2">
          <button class="btn btn-primary btn-sm" :disabled="saving || !form.name || !form.baseUrl" @click="save">
            <span v-if="saving" class="loading loading-spinner loading-xs" />
            保存
          </button>
          <button class="btn btn-ghost btn-sm border border-base-content/10" @click="form = null">取消</button>
          <span v-if="saveError" class="text-[11px] text-error">{{ saveError }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { getTauriAPI } from '@/utils/tauri-api'
import { useToast } from '@/composables/useToast'

interface AiModelRow { id: string; label?: string; contextWindow?: number; maxOutputTokens?: number; vision?: boolean }
interface AiProviderRow {
  id: string
  name: string
  protocol: 'openai' | 'anthropic'
  baseUrl: string
  apiKeyMasked?: string
  hasKey?: boolean
  enabled?: boolean
  models: AiModelRow[]
}

const toast = useToast()
const providers = ref<AiProviderRow[]>([])
const active = ref<{ providerId?: string; modelId?: string } | null>(null)
const form = ref<(AiProviderRow & { apiKey: string }) | null>(null)
const clearKey = ref(false)
const saving = ref(false)
const saveError = ref('')
const testing = ref('')
const results = ref<Record<string, any>>({})

const api = () => getTauriAPI() as any

/** 上下文窗口快捷档位（常见值，点击填入；仍可手动微调） */
const CONTEXT_PRESETS = [
  { label: '1M', value: 1048576 },
  { label: '512K', value: 524288 },
  { label: '256K', value: 262144 },
  { label: '128K', value: 131072 },
  { label: '64K', value: 65536 },
  { label: '32K', value: 32768 },
  { label: '8K', value: 8192 },
]
/** 输出上限快捷档位 */
const OUTPUT_PRESETS = [
  { label: '128K', value: 131072 },
  { label: '64K', value: 65536 },
  { label: '32K', value: 32768 },
  { label: '16K', value: 16384 },
  { label: '8K', value: 8192 },
  { label: '4K', value: 4096 },
]
/** 常见模型一键预设（协议/地址/模型ID/窗口/输出；模型版本更新快，仅作起点，按实际网关调整） */
const MODEL_PRESETS = [
  {
    name: 'DeepSeek',
    note: '官方 OpenAI 兼容端点（根路径）；deepseek-v4-flash 1M 上下文、输出最大 384K；识图需用 deepseek-v4-flash-vision-exp',
    protocol: 'openai',
    baseUrl: 'https://api.deepseek.com',
    models: [{ id: 'deepseek-v4-flash', label: 'DeepSeek V4 Flash', contextWindow: 1048576, maxOutputTokens: 32768, vision: false }],
  },
  {
    name: '智谱 GLM',
    note: 'glm-5.3-flash 为 1M 上下文、输出最大 128K、支持识图',
    protocol: 'openai',
    baseUrl: 'https://open.bigmodel.cn/api/paas/v4',
    models: [{ id: 'glm-5.3-flash', label: 'GLM 5.3 Flash', contextWindow: 1048576, maxOutputTokens: 32768, vision: true }],
  },
  {
    name: 'Kimi',
    note: 'kimi-k3 为 1M 上下文、默认输出 128K、原生支持识图',
    protocol: 'openai',
    baseUrl: 'https://api.moonshot.cn/v1',
    models: [{ id: 'kimi-k3', label: 'Kimi K3', contextWindow: 1048576, maxOutputTokens: 32768, vision: true }],
  },
  {
    name: '通义千问',
    note: 'qwen3.8-flash 为 1M 上下文（DashScope 国内兼容端点）；识图能力视网关而定，可手动开启',
    protocol: 'openai',
    baseUrl: 'https://dashscope.aliyuncs.com/compatible-mode/v1',
    models: [{ id: 'qwen3.8-flash', label: 'Qwen3.8 Flash', contextWindow: 1048576, maxOutputTokens: 32768, vision: false }],
  },
  {
    name: 'Anthropic Claude',
    note: '1M 上下文为 Sonnet 5 / Opus 5；Haiku 4.5 为 200K；均支持识图',
    protocol: 'anthropic',
    baseUrl: 'https://api.anthropic.com',
    models: [{ id: 'claude-sonnet-5', label: 'Claude Sonnet 5', contextWindow: 1000000, maxOutputTokens: 128000, vision: true }],
  },
  {
    name: 'OpenAI GPT',
    note: 'GPT-5.6 系（Sol/Terra/Luna）为 1M 上下文、支持识图',
    protocol: 'openai',
    baseUrl: 'https://api.openai.com/v1',
    models: [{ id: 'gpt-5.6-sol', label: 'GPT-5.6 Sol', contextWindow: 1050000, maxOutputTokens: 128000, vision: true }],
  },
  {
    name: 'Google Gemini',
    note: 'OpenAI 兼容端点；gemini-2.5-pro / 2.5-flash 均为 1M、支持识图',
    protocol: 'openai',
    baseUrl: 'https://generativelanguage.googleapis.com/v1beta/openai',
    models: [{ id: 'gemini-2.5-pro', label: 'Gemini 2.5 Pro', contextWindow: 1048576, maxOutputTokens: 65536, vision: true }],
  },
]

/** 友好显示 token 数：1048576 → 1M、131072 → 128K */
function formatTokens(n?: number) {
  const v = Number(n) || 0
  if (v >= 1048576) {return `${Math.round(v / 1048576 * 10) / 10}M`}
  if (v >= 1024) {return `${Math.round(v / 1024)}K`}
  return String(v)
}

/** 一键套用常见模型模板：填充提供商表单 */
function applyTemplate(t: (typeof MODEL_PRESETS)[number]) {
  if (!form.value) {return}
  form.value.name = t.name
  form.value.protocol = t.protocol as 'openai' | 'anthropic'
  form.value.baseUrl = t.baseUrl
  form.value.models = t.models.map(m => ({ ...m }))
  saveError.value = ''
}

/** 上下文快捷档位填入当前模型行 */
function setContextPreset(model: AiModelRow, preset: { label: string; value: number }) {
  model.contextWindow = preset.value
}
/** 输出上限快捷档位填入当前模型行 */
function setOutputPreset(model: AiModelRow, preset: { label: string; value: number }) {
  model.maxOutputTokens = preset.value
}

const activeKey = computed(() => (active.value ? `${active.value.providerId}::${active.value.modelId}` : ''))
const modelOptions = computed(() =>
  providers.value.flatMap(p =>
    (p.models || []).map(m => ({
      value: `${p.id}::${m.id}`,
      label: `${p.name} · ${m.label || m.id}（${formatTokens(m.contextWindow || 8192)} 窗口）`,
    })),
  ),
)
const activeModel = computed(() => {
  const p = providers.value.find(x => x.id === active.value?.providerId)
  return p?.models.find(m => m.id === active.value?.modelId) || null
})
const existingMask = computed(() => {
  const p = providers.value.find(x => x.id === form.value?.id)
  return p?.hasKey ? p.apiKeyMasked : '（未设置）'
})

function testKey(p: AiProviderRow, m: AiModelRow) {
  return `${p.id}::${m.id}`
}

function isActive(p: AiProviderRow, m: AiModelRow) {
  return active.value?.providerId === p.id && active.value?.modelId === m.id
}

async function load() {
  try {
    providers.value = (await api().listAiProviders()) || []
    active.value = (await api().getActiveAiModel()) || null
  } catch (e) {
    toast.error(`读取模型配置失败：${String((e as Error)?.message || e)}`)
  }
}

function openForm(p: AiProviderRow | null) {
  clearKey.value = false
  saveError.value = ''
  form.value = p
    ? { ...JSON.parse(JSON.stringify(p)), apiKey: '' }
    : {
        id: '',
        name: '',
        protocol: 'openai',
        baseUrl: 'https://',
        apiKey: '',
        enabled: true,
        models: [{ id: '', contextWindow: 131072, maxOutputTokens: 8192 }],
      }
}

function addModel() {
  form.value?.models.push({ id: '', contextWindow: 131072, maxOutputTokens: 8192 })
}

async function save() {
  if (!form.value) {return}
  saving.value = true
  saveError.value = ''
  const payload: any = {
    ...form.value,
    models: (form.value.models || []).filter(m => String(m.id || '').trim()),
    apiKey: clearKey.value ? '__clear__' : form.value.apiKey || '',
  }
  try {
    await api().saveAiProvider(payload)
    toast.success('已保存')
    form.value = null
    await load()
  } catch (e) {
    saveError.value = String((e as Error)?.message || e)
  } finally {
    saving.value = false
  }
}

async function remove(p: AiProviderRow) {
  try {
    await api().deleteAiProvider(p.id)
    toast.info('已删除')
    await load()
  } catch (e) {
    toast.error(`删除失败：${String((e as Error)?.message || e)}`)
  }
}

async function onPickActive(event: Event) {
  const value = (event.target as HTMLSelectElement).value
  const [providerId, modelId] = value.split('::')
  if (!providerId || !modelId) {return}
  await setActiveById(providerId, modelId)
}

async function setActive(p: AiProviderRow, m: AiModelRow) {
  await setActiveById(p.id, m.id)
}

async function setActiveById(providerId: string, modelId: string) {
  try {
    await api().setActiveAiModel(providerId, modelId)
    active.value = { providerId, modelId }
    toast.success('已切换当前模型')
  } catch (e) {
    toast.error(`切换失败：${String((e as Error)?.message || e)}`)
  }
}

async function test(p: AiProviderRow, m: AiModelRow) {
  const key = testKey(p, m)
  testing.value = key
  try {
    results.value[key] = await api().testAiModel(p.id, m.id)
  } catch (e) {
    results.value[key] = { ok: false, error: String((e as Error)?.message || e) }
  } finally {
    testing.value = ''
  }
}

onMounted(load)
defineExpose({ load })
</script>
