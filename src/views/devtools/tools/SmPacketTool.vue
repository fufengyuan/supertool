<template>
  <ToolPage
    icon="key"
    name="国密报文"
    description="国密接口报文加解密：SM4 对称加密（ECB 补零 / ECB PKCS7）+ SM3 排序签名，适配常见的 Hutool / BouncyCastle 报文协议"
    @back="$emit('back')"
  >
    <!-- 模式切换 -->
    <div class="flex items-end gap-3 bg-base-100 border border-base-content/10 rounded-xl p-4">
      <div>
        <span class="text-[11px] font-medium text-base-content/50 mb-1 block">报文类型</span>
        <div class="join">
          <button class="btn btn-sm join-item" :class="tab === 'packet' ? 'btn-primary' : 'btn-ghost'" @click="tab = 'packet'">接口报文（ECB 补零）</button>
          <button class="btn btn-sm join-item" :class="tab === 'web' ? 'btn-primary' : 'btn-ghost'" @click="tab = 'web'">前后端报文（H5↔后端）</button>
        </div>
      </div>
      <div v-if="tab === 'packet'" class="flex-1 min-w-[200px]">
        <span class="text-[11px] font-medium text-base-content/50 mb-1 block">org_code（机构号）</span>
        <input v-model="orgCode" class="input input-bordered input-sm w-full font-mono text-xs bg-base-200/60" placeholder="机构号 / 商户号，如 1000000001" />
      </div>
      <label class="flex items-center gap-1.5 text-[11px] text-base-content/50 cursor-pointer pb-1.5" title="勾选后密钥明文保存在本机 localStorage（移除勾选即清除），生产密钥请谨慎">
        <input v-model="remember" type="checkbox" class="checkbox checkbox-xs" @change="onRememberChange" />
        记住密钥（仅存本机）
      </label>
    </div>

    <!-- ============ 接口报文（ECB 补零 + SM3 签名）============ -->
    <template v-if="tab === 'packet'">
      <!-- 密钥区 -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 bg-base-100 border border-base-content/10 rounded-xl p-4">
        <div>
          <span class="text-[11px] font-medium text-base-content/50 mb-1 block">SM4 密钥 secret（hex，>32 位自动截断前 32 位）</span>
          <input v-model="secret" class="input input-bordered input-sm w-full font-mono text-xs bg-base-200/60" placeholder="32 位 hex" />
        </div>
        <div>
          <span class="text-[11px] font-medium text-base-content/50 mb-1 block">SM3 签名密钥 key</span>
          <input v-model="signKey" class="input input-bordered input-sm w-full font-mono text-xs bg-base-200/60" placeholder="渠道配置的 key" />
        </div>
      </div>

      <!-- 动作切换 -->
      <div class="flex items-center gap-2">
        <div class="join">
          <button class="btn btn-sm join-item" :class="action === 'decrypt' ? 'btn-primary' : 'btn-ghost'" @click="action = 'decrypt'">解报文</button>
          <button class="btn btn-sm join-item" :class="action === 'encrypt' ? 'btn-primary' : 'btn-ghost'" @click="action = 'encrypt'">加密文</button>
          <button class="btn btn-sm join-item" :class="action === 'sign' ? 'btn-primary' : 'btn-ghost'" @click="action = 'sign'">算签/验签</button>
        </div>
        <button class="btn btn-ghost btn-xs" @click="loadSample">填入示例</button>
      </div>

      <!-- 解报文 -->
      <template v-if="action === 'decrypt'">
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
          <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4">
            <div class="flex items-center justify-between mb-2">
              <span class="text-xs font-semibold text-base-content/70">报文（完整 JSON 或仅 biz_content 密文）</span>
            </div>
            <textarea v-model="decInput" class="textarea textarea-bordered w-full font-mono text-xs bg-base-200/60 min-h-[160px] resize-none flex-1" placeholder='{"biz_content":"35474249...","charset":"UTF-8","method":"api.xxx",...}'></textarea>
          </div>
          <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4">
            <div class="flex items-center justify-between mb-2">
              <span class="text-xs font-semibold text-base-content/70">biz_content 明文</span>
              <button class="btn btn-primary btn-xs" @click="copyDec" :disabled="!decOutput"><SvgIcon name="copy" size="11" /> 复制</button>
            </div>
            <div class="flex-1 p-3 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-xs whitespace-pre-wrap break-all overflow-y-auto min-h-[160px]">{{ decOutput || '结果将显示在这里...' }}</div>
          </div>
        </div>
        <div v-if="decVerify" class="bg-base-100 border border-base-content/10 rounded-xl p-4">
          <div class="text-xs font-semibold text-base-content/70 mb-2">验签</div>
          <div class="p-3 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-xs break-all">{{ decVerify }}</div>
        </div>
        <div class="flex gap-2 bg-base-100 border border-base-content/10 rounded-xl px-4 py-3">
          <button class="btn btn-primary btn-sm flex-1 max-w-[160px]" @click="doDecrypt">解密</button>
          <span class="text-xs text-base-content/50 self-center">SM4/ECB/NoPadding + 明文侧补 0x00，密文 hex 大写，解密后 trim</span>
        </div>
      </template>

      <!-- 加密文 -->
      <template v-if="action === 'encrypt'">
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
          <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4">
            <div class="flex items-center justify-between mb-2">
              <span class="text-xs font-semibold text-base-content/70">biz_content 明文（JSON）</span>
            </div>
            <textarea v-model="encInput" class="textarea textarea-bordered w-full font-mono text-xs bg-base-200/60 min-h-[160px] resize-none flex-1" placeholder='{"orderNo":"202609070001","amt":1}'></textarea>
            <div class="grid grid-cols-2 gap-2 mt-2">
              <div>
                <span class="text-[11px] font-medium text-base-content/50 mb-1 block">method</span>
                <input v-model="method" class="input input-bordered input-sm w-full font-mono text-xs bg-base-200/60" placeholder="api.xxx" />
              </div>
              <div>
                <span class="text-[11px] font-medium text-base-content/50 mb-1 block">time_stamp（留空=当前毫秒）</span>
                <input v-model="timeStamp" class="input input-bordered input-sm w-full font-mono text-xs bg-base-200/60" placeholder="1788714000819" />
              </div>
            </div>
          </div>
          <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4">
            <div class="flex items-center justify-between mb-2">
              <span class="text-xs font-semibold text-base-content/70">完整请求报文（含 sign）</span>
              <button class="btn btn-primary btn-xs" @click="copyEnc" :disabled="!encOutput"><SvgIcon name="copy" size="11" /> 复制</button>
            </div>
            <div class="flex-1 p-3 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-xs whitespace-pre-wrap break-all overflow-y-auto min-h-[160px]">{{ encOutput || '结果将显示在这里...' }}</div>
          </div>
        </div>
        <div class="flex gap-2 bg-base-100 border border-base-content/10 rounded-xl px-4 py-3">
          <button class="btn btn-primary btn-sm flex-1 max-w-[160px]" @click="doEncrypt">加密并签名</button>
          <span class="text-xs text-base-content/50 self-center">request_id 自动生成 UUID；sign 对除自身外全部字段排序后 SM3</span>
        </div>
      </template>

      <!-- 算签 / 验签 -->
      <template v-if="action === 'sign'">
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
          <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4">
            <div class="flex items-center justify-between mb-2">
              <span class="text-xs font-semibold text-base-content/70">报文 JSON（含 sign 则验签，不含则算签）</span>
            </div>
            <textarea v-model="signInput" class="textarea textarea-bordered w-full font-mono text-xs bg-base-200/60 min-h-[160px] resize-none flex-1" placeholder='{"biz_content":"...","charset":"UTF-8","sign":"..."}'></textarea>
          </div>
          <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4">
            <div class="flex items-center justify-between mb-2">
              <span class="text-xs font-semibold text-base-content/70">结果</span>
              <button class="btn btn-primary btn-xs" @click="copySign" :disabled="!signOutput"><SvgIcon name="copy" size="11" /> 复制</button>
            </div>
            <div class="flex-1 p-3 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-xs whitespace-pre-wrap break-all overflow-y-auto min-h-[160px]">{{ signOutput || '结果将显示在这里...' }}</div>
          </div>
        </div>
        <div v-if="signDataPreview" class="bg-base-100 border border-base-content/10 rounded-xl p-4">
          <div class="text-xs font-semibold text-base-content/70 mb-2">待签名串</div>
          <div class="p-3 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-xs break-all max-h-[120px] overflow-y-auto">{{ signDataPreview }}</div>
          <div class="text-[11px] text-base-content/50 mt-2">
            规则：字段按 key 字典序 → <code>k=v&amp;k=v&amp;</code> → 末尾 <code>key=签名密钥</code> → SM3 → hex 大写（sign 字段不参与）
          </div>
        </div>
        <div class="flex gap-2 bg-base-100 border border-base-content/10 rounded-xl px-4 py-3">
          <button class="btn btn-primary btn-sm flex-1 max-w-[160px]" @click="doSign">算签 / 验签</button>
        </div>
      </template>
    </template>

    <!-- ============ 前后端报文（H5 ↔ 后端）============ -->
    <template v-else>
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 bg-base-100 border border-base-content/10 rounded-xl p-4">
        <div>
          <span class="text-[11px] font-medium text-base-content/50 mb-1 block">SM4 密钥（16 位字符，多为登录时动态下发的会话密钥）</span>
          <input v-model="webKey" class="input input-bordered input-sm w-full font-mono text-xs bg-base-200/60" placeholder="16 位" />
        </div>
        <div class="flex items-end gap-2">
          <div class="join">
            <button class="btn btn-sm join-item" :class="webMode === 'decrypt' ? 'btn-primary' : 'btn-ghost'" @click="webMode = 'decrypt'">解密</button>
            <button class="btn btn-sm join-item" :class="webMode === 'encrypt' ? 'btn-primary' : 'btn-ghost'" @click="webMode = 'encrypt'">加密</button>
          </div>
        </div>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4">
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs font-semibold text-base-content/70">{{ webMode === 'encrypt' ? '明文（请求/响应 JSON）' : '密文（hex）' }}</span>
          </div>
          <textarea v-model="webInput" class="textarea textarea-bordered w-full font-mono text-xs bg-base-200/60 min-h-[180px] resize-none flex-1" :placeholder="webMode === 'encrypt' ? '明文 JSON，如 {orderNo:123}' : '密文 hex，如 35474249A1C65E9C...'"></textarea>
        </div>
        <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4">
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs font-semibold text-base-content/70">{{ webMode === 'encrypt' ? '密文（hex）' : '明文' }}</span>
            <button class="btn btn-primary btn-xs" @click="copyWeb" :disabled="!webOutput"><SvgIcon name="copy" size="11" /> 复制</button>
          </div>
          <div class="flex-1 p-3 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-xs whitespace-pre-wrap break-all overflow-y-auto min-h-[180px]">{{ webOutput || '结果将显示在这里...' }}</div>
        </div>
      </div>

      <div class="flex gap-2 bg-base-100 border border-base-content/10 rounded-xl px-4 py-3">
        <button class="btn btn-primary btn-sm flex-1 max-w-[160px]" @click="processWeb">{{ webMode === 'encrypt' ? '加密' : '解密' }}</button>
        <span class="text-xs text-base-content/50 self-center">Hutool SmUtil.sm4(key.getBytes())：SM4/ECB/PKCS5Padding，密钥取 16 位字符的 UTF-8 字节，密文 hex</span>
      </div>
    </template>
  </ToolPage>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { sm3, sm4 } from 'sm-crypto'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

const toast = useToast()
const STORAGE_KEY = 'devtools.sm-packet-tool'
/** 示例密钥（公开演示值，非任何真实渠道密钥） */
const SAMPLE_SECRET = '00112233445566778899aabbccddeeff'

const tab = ref<'packet' | 'web'>('packet')
const action = ref<'decrypt' | 'encrypt' | 'sign'>('decrypt')

// ===== 渠道配置（手填，可选记住到本机）=====
const secret = ref('')
const signKey = ref('')
const orgCode = ref('')
const remember = ref(false)

const decInput = ref('')
const decOutput = ref('')
const decVerify = ref('')
const encInput = ref('')
const encOutput = ref('')
const method = ref('')
const timeStamp = ref('')
const signInput = ref('')
const signOutput = ref('')
const signDataPreview = ref('')

// ===== 前后端报文 =====
const webKey = ref('')
const webMode = ref<'encrypt' | 'decrypt'>('decrypt')
const webInput = ref('')
const webOutput = ref('')

// ---------- 基础工具 ----------
const encoder = new TextEncoder()
const decoder = new TextDecoder()

/** 密钥规范化：与 Java Sm4BcUtil 一致，hex 串超过 32 位取前 32 位 */
function normalizeSecret(hex: string): string {
  const s = hex.trim()
  return s.length > 32 ? s.slice(0, 32) : s
}

function isHex32(s: string): boolean {
  return /^[0-9a-fA-F]{32}$/.test(s)
}

/** 明文 → UTF-8 字节 → 末尾补 0x00 到 16 倍数（正好 16 倍数时不补，与 Java 分支一致） */
function zeroPad(bytes: number[]): number[] {
  const mod = bytes.length % 16
  if (mod === 0) {return bytes}
  return [...bytes, ...new Array(16 - mod).fill(0)]
}

function stripTrailingZero(bytes: number[]): number[] {
  const out = [...bytes]
  while (out.length > 0 && out[out.length - 1] === 0) {out.pop()}
  return out
}

/** 渠道报文加密：SM4/ECB/NoPadding + 补零，输出 hex 大写 */
function sm4EncryptYfk(plain: string, keyHex: string): string {
  const padded = zeroPad([...encoder.encode(plain)])
  const out = sm4.encrypt(padded, normalizeSecret(keyHex), { padding: 'none', mode: 'ecb' })
  return String(out).toUpperCase()
}

/** 渠道报文解密：hex → SM4/ECB/NoPadding → 去掉尾部补零 → trim */
function sm4DecryptYfk(cipherHex: string, keyHex: string): string {
  const raw = sm4.decrypt(cipherHex.trim(), normalizeSecret(keyHex), {
    padding: 'none',
    mode: 'ecb',
    output: 'array',
  }) as number[]
  return decoder.decode(new Uint8Array(stripTrailingZero(raw))).trim()
}

/** request_id：优先 crypto.randomUUID，老 webview 无此 API 时退回随机 hex */
function genRequestId(): string {
  const c = globalThis.crypto
  if (c && typeof c.randomUUID === 'function') {return c.randomUUID()}
  const hex = (n: number) => Array.from({ length: n }, () => Math.floor(Math.random() * 16).toString(16)).join('')
  return `${hex(8)}-${hex(4)}-${hex(4)}-${hex(4)}-${hex(12)}`
}

/** 排序拼串 + 末尾 key=签名密钥（sign 字段不参与） */
function buildSignData(data: Record<string, unknown>, key: string): string {
  let sb = ''
  for (const k of Object.keys(data).sort()) {
    if (k === 'sign') {continue}
    // 与 Java data.get(key) 拼接语义一致：值按字符串参与（渠道报文字段均为字符串/数字）
    sb += `${k}=${String(data[k])}&`
  }
  return `${sb}key=${key}`
}

function sm3Upper(s: string): string {
  return sm3(s).toUpperCase()
}

/** 从粘贴内容里取出待处理的报文对象：整体是 JSON 对象则直接用，否则视为裸密文 */
function parseEnvelope(input: string): { data: Record<string, unknown> | null; raw: string } {
  const text = input.trim()
  if (!text) {return { data: null, raw: '' }}
  try {
    const obj = JSON.parse(text)
    if (obj && typeof obj === 'object' && !Array.isArray(obj)) {
      return { data: obj as Record<string, unknown>, raw: text }
    }
  } catch {/* 非 JSON：按裸密文处理 */}
  return { data: null, raw: text }
}

function requireKeys(): boolean {
  if (!isHex32(normalizeSecret(secret.value))) {
    toast.warning('SM4 密钥 secret 需为 32 位 hex')
    return false
  }
  if (!signKey.value.trim()) {
    toast.warning('请填写 SM3 签名密钥 key')
    return false
  }
  return true
}

// ---------- 动作 ----------
function doDecrypt() {
  decOutput.value = ''
  decVerify.value = ''
  if (!isHex32(normalizeSecret(secret.value))) {
    toast.warning('SM4 密钥 secret 需为 32 位 hex')
    return
  }
  const { data, raw } = parseEnvelope(decInput.value)
  const cipher = data ? String(data.biz_content ?? '') : raw
  if (!cipher) {
    toast.warning('未找到 biz_content，请粘贴完整报文或密文')
    return
  }
  try {
    const plain = sm4DecryptYfk(cipher, secret.value)
    decOutput.value = plain
    // 顺带验签：能拿到 sign 且有签名密钥时才做
    if (data && data.sign && signKey.value.trim()) {
      const expect = sm3Upper(buildSignData(data, signKey.value.trim()))
      decVerify.value = expect === String(data.sign).toUpperCase()
        ? `✅ 验签通过（sign = ${expect}）`
        : `❌ 验签不通过\n报文 sign：${String(data.sign)}\n重算 sign：${expect}`
    }
    toast.success('解密完成')
  } catch (e: any) {
    toast.error(`解密失败：${e.message || e}`)
  }
}

function doEncrypt() {
  encOutput.value = ''
  if (!requireKeys()) {return}
  if (!encInput.value.trim()) {
    toast.warning('请输入 biz_content 明文')
    return
  }
  try {
    const bizContent = sm4EncryptYfk(encInput.value.trim(), secret.value)
    const ts = timeStamp.value.trim() || String(Date.now())
    const payload: Record<string, unknown> = {
      biz_content: bizContent,
      charset: 'UTF-8',
      encrypt_type: 'SM4',
      method: method.value.trim() || 'api.xxx',
      org_code: orgCode.value.trim(),
      request_id: genRequestId(),
      sign_type: 'SM3',
      time_stamp: ts,
    }
    payload.sign = sm3Upper(buildSignData(payload, signKey.value.trim()))
    encOutput.value = JSON.stringify(payload, null, 2)
    toast.success('加密并签名完成')
  } catch (e: any) {
    toast.error(`加密失败：${e.message || e}`)
  }
}

function doSign() {
  signOutput.value = ''
  signDataPreview.value = ''
  if (!signKey.value.trim()) {
    toast.warning('请填写 SM3 签名密钥 key')
    return
  }
  const { data } = parseEnvelope(signInput.value)
  if (!data) {
    toast.warning('请输入报文 JSON 对象')
    return
  }
  try {
    const signData = buildSignData(data, signKey.value.trim())
    const sign = sm3Upper(signData)
    signDataPreview.value = signData
    const given = data.sign ? String(data.sign).toUpperCase() : ''
    signOutput.value = given
      ? (given === sign ? `✅ 验签通过\nsign = ${sign}` : `❌ 验签不通过\n报文 sign：${given}\n重算 sign：${sign}`)
      : sign
    toast.success(given ? '验签完成' : '签名已生成')
  } catch (e: any) {
    toast.error(`签名失败：${e.message || e}`)
  }
}

function loadSample() {
  orgCode.value = orgCode.value || '1000000001'
  method.value = method.value || 'api.xxx'
  encInput.value = '{"orderNo":"202609070001","amt":1}'
  decInput.value = JSON.stringify({
    biz_content: '35474249A1C65E9C53B46F9236D43636DF441FD94E44AB0BEA8E0CAC1DCCA30E9D3A12EE2806F42F960495CEC6444019',
    charset: 'UTF-8',
    encrypt_type: 'SM4',
    method: 'api.xxx',
    org_code: '1000000001',
    request_id: '7484de8b-f4bf-472e-afa8-4e433906a3d1',
    sign_type: 'SM3',
    time_stamp: '1788714000819',
  }, null, 2)
  signInput.value = decInput.value
  // 示例密钥为公开演示值：填入后可直接点「解密」看到自洽结果
  if (!secret.value.trim()) {secret.value = SAMPLE_SECRET}
  toast.info('已填入示例（含示例密钥，可点解密验证）')
}

// ---------- 前后端报文（Hutool SM4/ECB/PKCS5Padding）----------
function webKeyHex(): string {
  // Hutool: password.getBytes(UTF-8) 直接当 16 字节密钥
  const bytes = [...encoder.encode(webKey.value)]
  return bytes.map(b => b.toString(16).padStart(2, '0')).join('')
}

function processWeb() {
  webOutput.value = ''
  if (!webKey.value) {
    toast.warning('请填写 16 位 SM4 密钥')
    return
  }
  const keyBytes = [...encoder.encode(webKey.value)]
  if (keyBytes.length !== 16) {
    toast.warning(`SM4 密钥需为 16 位字符（当前 ${keyBytes.length} 字节）`)
    return
  }
  if (!webInput.value.trim()) {
    toast.warning('请输入内容')
    return
  }
  try {
    const keyHex = webKeyHex()
    if (webMode.value === 'encrypt') {
      webOutput.value = String(sm4.encrypt(webInput.value, keyHex, { mode: 'ecb' }))
    } else {
      const arr = sm4.decrypt(webInput.value.trim(), keyHex, { mode: 'ecb', output: 'array' }) as number[]
      webOutput.value = decoder.decode(new Uint8Array(arr))
    }
    toast.success(webMode.value === 'encrypt' ? '加密完成' : '解密完成')
  } catch (e: any) {
    toast.error(`${webMode.value === 'encrypt' ? '加密' : '解密'}失败：${e.message || e}`)
  }
}

// ---------- 复制 ----------
const copyDec = () => copyText(decOutput.value, toast)
const copyEnc = () => copyText(encOutput.value, toast)
const copySign = () => copyText(signOutput.value, toast)
const copyWeb = () => copyText(webOutput.value, toast)

// ---------- 密钥记忆（仅本机 localStorage，取消勾选即清除）----------
function saveConfig() {
  localStorage.setItem(STORAGE_KEY, JSON.stringify({
    secret: secret.value,
    signKey: signKey.value,
    orgCode: orgCode.value,
    webKey: webKey.value,
  }))
}

function onRememberChange() {
  if (!remember.value) {
    localStorage.removeItem(STORAGE_KEY)
    return
  }
  saveConfig()
}

// 勾选记住后，密钥改动随同步（取消勾选即清除，不留副本）
watch([secret, signKey, orgCode, webKey], () => {
  if (remember.value) {saveConfig()}
})

onMounted(() => {
  try {
    const saved = localStorage.getItem(STORAGE_KEY)
    if (!saved) {return}
    const cfg = JSON.parse(saved)
    secret.value = cfg.secret || ''
    signKey.value = cfg.signKey || ''
    orgCode.value = cfg.orgCode || ''
    webKey.value = cfg.webKey || ''
    remember.value = true
  } catch {/* 忽略损坏的本地配置 */}
})
</script>
