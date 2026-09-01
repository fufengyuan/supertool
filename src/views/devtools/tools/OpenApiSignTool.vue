<template>
  <ToolPage
    icon="zap"
    name="OpenAPI 签名"
    description="开放接口 MD5 签名/验签 + AES-ECB 字段加解密（与 Java 端 TreeMap+MD5、Hutool AES-ECB 对齐）"
    @back="$emit('back')"
  >
    <!-- 模式选择 -->
    <div class="flex items-end gap-3 bg-base-100 border border-base-content/10 rounded-xl p-4">
      <div>
        <span class="text-[11px] font-medium text-base-content/50 mb-1 block">功能</span>
        <div class="join">
          <button class="btn btn-sm join-item" :class="tab === 'sign' ? 'btn-primary' : 'btn-ghost'" @click="tab = 'sign'">请求签名</button>
          <button class="btn btn-sm join-item" :class="tab === 'verify' ? 'btn-primary' : 'btn-ghost'" @click="tab = 'verify'">响应验签</button>
          <button class="btn btn-sm join-item" :class="tab === 'aes' ? 'btn-primary' : 'btn-ghost'" @click="tab = 'aes'">AES-ECB 字段</button>
        </div>
      </div>
      <div class="flex-1 min-w-[240px]">
        <span class="text-[11px] font-medium text-base-content/50 mb-1 block">签名密钥 signKey</span>
        <input v-model="signKey" class="input input-bordered input-sm w-full font-mono text-xs bg-base-200/60" placeholder="如 pDwkFKUZv60KWs53" />
      </div>
    </div>

    <!-- 请求签名 -->
    <template v-if="tab === 'sign'">
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4">
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs font-semibold text-base-content/70">请求参数（JSON 对象）</span>
            <button class="btn btn-ghost btn-xs" @click="loadSignSample" title="填入示例">示例</button>
          </div>
          <textarea v-model="signInput" class="textarea textarea-bordered w-full font-mono text-xs bg-base-200/60 min-h-[140px] resize-none flex-1" placeholder='{"partnerId":"C1740965852","outTradeNo":"VQ123456789","faceValue":10,"buyCount":2}'></textarea>
        </div>
        <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4">
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs font-semibold text-base-content/70">签名结果</span>
            <button class="btn btn-primary btn-xs" @click="copySign" :disabled="!signOutput"><SvgIcon name="copy" size="11" /> 复制</button>
          </div>
          <div class="flex-1 p-3 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-xs whitespace-pre-wrap break-all overflow-y-auto min-h-[140px]">{{ signOutput || '结果将显示在这里...' }}</div>
        </div>
      </div>

      <!-- 拼串预览 -->
      <div v-if="signDataPreview" class="bg-base-100 border border-base-content/10 rounded-xl p-4">
        <div class="text-xs font-semibold text-base-content/70 mb-2">待签名串（key 字典序拼串）</div>
        <div class="p-3 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-xs break-all max-h-[100px] overflow-y-auto">{{ signDataPreview }}</div>
        <div class="text-[11px] text-base-content/50 mt-2">
          规则：TreeMap 按 key 字典序排列 → <code>k=v&amp;k=v</code> → <code>MD5(拼串 + "&amp;key=" + signKey)</code>，空字符串值不参与签名（与 Java 端一致）
        </div>
      </div>

      <div class="flex gap-2 bg-base-100 border border-base-content/10 rounded-xl px-4 py-3">
        <button class="btn btn-primary btn-sm flex-1 max-w-[160px]" @click="doSign">生成签名</button>
        <span class="text-xs text-base-content/50 self-center">生成后可直接把 sign 字段放进请求参数</span>
      </div>
    </template>

    <!-- 响应验签 -->
    <template v-if="tab === 'verify'">
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4">
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs font-semibold text-base-content/70">响应报文（完整 JSON，含 sign）</span>
            <button class="btn btn-ghost btn-xs" @click="loadVerifySample" title="填入示例">示例</button>
          </div>
          <textarea v-model="verifyInput" class="textarea textarea-bordered w-full font-mono text-xs bg-base-200/60 min-h-[180px] resize-none flex-1" placeholder='{"code":0,"data":{"orderNo":"..."},"msg":"","sign":"..."}'></textarea>
        </div>
        <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4">
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs font-semibold text-base-content/70">验签结果</span>
          </div>
          <div class="flex-1 flex flex-col gap-2 min-h-[180px]">
            <div v-if="verifyResult !== null" class="p-3 rounded-lg border text-sm font-semibold" :class="verifyResult.ok ? 'bg-success/10 text-success border-success/30' : 'bg-error/10 text-error border-error/30'">
              {{ verifyResult.ok ? '✓ 验签通过' : '✗ 验签失败' }}
              <div v-if="!verifyResult.ok" class="text-xs font-normal mt-2 space-y-1">
                <div>响应中的 sign：<code class="break-all">{{ verifyResult.remoteSign }}</code></div>
                <div>本地计算 sign：<code class="break-all">{{ verifyResult.localSign }}</code></div>
              </div>
            </div>
            <div v-if="verifyDataPreview" class="p-3 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-xs break-all max-h-[120px] overflow-y-auto">{{ verifyDataPreview }}</div>
          </div>
        </div>
      </div>

      <div class="bg-base-100 border border-base-content/10 rounded-xl px-4 py-3 text-[11px] text-base-content/50 leading-relaxed">
        验签规则：响应去掉 sign 后 → <code>data</code> 值重序列化为 <b>key 字典序 JSON 串</b>（内层 TreeMap），其余字段按字典序拼串 →
        <code>MD5(拼串 + "&amp;key=" + signKey)</code> 与响应 sign 比对
      </div>

      <div class="flex gap-2 bg-base-100 border border-base-content/10 rounded-xl px-4 py-3">
        <button class="btn btn-primary btn-sm flex-1 max-w-[160px]" @click="doVerify">验签</button>
      </div>
    </template>

    <!-- AES-ECB 字段加解密 -->
    <template v-if="tab === 'aes'">
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-4 flex flex-col gap-3">
        <div class="flex items-end gap-3">
          <div class="min-w-[260px] flex-1">
            <span class="text-[11px] font-medium text-base-content/50 mb-1 block">AES 密钥（32 位 hex = 16 字节）</span>
            <input v-model="aesKey" class="input input-bordered input-sm w-full font-mono text-xs bg-base-200/60" placeholder="如 238be0efdc748197317021efdbc9ecde" />
          </div>
          <div>
            <span class="text-[11px] font-medium text-base-content/50 mb-1 block">方向</span>
            <div class="join">
              <button class="btn btn-sm join-item" :class="aesMode === 'encrypt' ? 'btn-primary' : 'btn-ghost'" @click="aesMode = 'encrypt'">加密（明文→Base64）</button>
              <button class="btn btn-sm join-item" :class="aesMode === 'decrypt' ? 'btn-primary' : 'btn-ghost'" @click="aesMode = 'decrypt'">解密（Base64→明文）</button>
            </div>
          </div>
        </div>
        <div class="text-[11px] text-base-content/50">
          对应 Java 端 <code>new AES(Mode.ECB, Padding.PKCS5Padding, HexUtil.decodeHex(key))</code>：AES-128-ECB / PKCS7，密文 Base64。
          典型场景：<code>userPhone</code> 请求加密、<code>voucherList</code> 响应解密。
        </div>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4">
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs font-semibold text-base-content/70">{{ aesMode === 'encrypt' ? '明文（UTF-8）' : '密文（Base64）' }}</span>
            <button class="btn btn-ghost btn-xs" @click="loadAesSample">示例</button>
          </div>
          <textarea v-model="aesInput" class="textarea textarea-bordered w-full font-mono text-xs bg-base-200/60 min-h-[120px] resize-none flex-1" :placeholder="aesMode === 'encrypt' ? '13800000000' : '粘贴 Base64 密文...'"></textarea>
        </div>
        <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4">
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs font-semibold text-base-content/70">{{ aesMode === 'encrypt' ? '密文（Base64）' : '明文（UTF-8）' }}</span>
            <button class="btn btn-primary btn-xs" @click="copyAes" :disabled="!aesOutput"><SvgIcon name="copy" size="11" /> 复制</button>
          </div>
          <div class="flex-1 p-3 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-xs whitespace-pre-wrap break-all overflow-y-auto min-h-[120px]">{{ aesOutput || '结果将显示在这里...' }}</div>
        </div>
      </div>

      <div class="flex gap-2 bg-base-100 border border-base-content/10 rounded-xl px-4 py-3">
        <button class="btn btn-primary btn-sm flex-1 max-w-[160px]" @click="processAes">{{ aesMode === 'encrypt' ? '加密' : '解密' }}</button>
      </div>
    </template>
  </ToolPage>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import CryptoJS from 'crypto-js'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

const toast = useToast()
const tab = ref<'sign' | 'verify' | 'aes'>('sign')
const signKey = ref('')

// ===== 请求签名 =====
const signInput = ref('')
const signOutput = ref('')
const signDataPreview = ref('')

/** TreeMap 语义：key 字典序；空字符串值不参与（Java 端约定"空字符不参与加/验签"） */
function buildSignData(params: Record<string, unknown>): string {
  const keys = Object.keys(params)
    .filter(k => params[k] !== null && params[k] !== undefined && params[k] !== '')
    .sort()
  return keys.map(k => `${k}=${params[k]}`).join('&')
}

function md5(s: string): string {
  return CryptoJS.MD5(s).toString()
}

function doSign() {
  signOutput.value = ''
  signDataPreview.value = ''
  if (!signKey.value.trim()) {
    toast.warning('请填写签名密钥 signKey')
    return
  }
  if (!signInput.value.trim()) {
    toast.warning('请输入请求参数 JSON')
    return
  }
  try {
    const obj = JSON.parse(signInput.value)
    if (obj === null || typeof obj !== 'object' || Array.isArray(obj)) {
      toast.error('参数必须是 JSON 对象（键值对）')
      return
    }
    if ('sign' in obj) {delete (obj as Record<string, unknown>).sign}
    const signData = buildSignData(obj)
    signDataPreview.value = signData
    signOutput.value = md5(`${signData}&key=${signKey.value.trim()}`)
    toast.success('签名已生成')
  } catch (e: any) {
    toast.error(`参数不是合法 JSON：${e.message}`)
  }
}

function loadSignSample() {
  signInput.value = JSON.stringify({
    partnerId: 'C1740965852',
    outTradeNo: 'VQ' + Date.now(),
    faceValue: 10,
    buyCount: 2,
  })
  if (!signKey.value) {signKey.value = 'pDwkFKUZv60KWs53'}
}

function copySign() {
  copyText(signOutput.value, toast)
}

// ===== 响应验签 =====
const verifyInput = ref('')
const verifyResult = ref<{ ok: boolean; remoteSign: string; localSign: string } | null>(null)
const verifyDataPreview = ref('')

/** 模拟 fastjson TreeMap 序列化：key 字典序（JSON.parseObject(new TreeMap) 语义） */
function sortedJsonString(obj: Record<string, unknown>): string {
  const keys = Object.keys(obj).sort()
  return JSON.stringify(Object.fromEntries(keys.map(k => [k, obj[k]])))
}

function doVerify() {
  verifyResult.value = null
  verifyDataPreview.value = ''
  if (!signKey.value.trim()) {
    toast.warning('请填写签名密钥 signKey')
    return
  }
  if (!verifyInput.value.trim()) {
    toast.warning('请输入响应 JSON')
    return
  }
  try {
    const map = JSON.parse(verifyInput.value)
    if (map === null || typeof map !== 'object' || Array.isArray(map)) {
      toast.error('响应必须是 JSON 对象')
      return
    }
    const remoteSign = String(map.sign ?? '')
    if (!remoteSign) {
      toast.error('响应中没有 sign 字段')
      return
    }
    // 与 Java 端一致：data 重序列化为 TreeMap JSON 串；其余字段（含 code/msg）字典序拼串
    const cloned: Record<string, unknown> = { ...map }
    delete cloned.sign
    if (cloned.data !== null && typeof cloned.data === 'object' && !Array.isArray(cloned.data)) {
      cloned.data = sortedJsonString(cloned.data as Record<string, unknown>)
    }
    const signData = buildSignData(cloned)
    verifyDataPreview.value = signData
    const localSign = md5(`${signData}&key=${signKey.value.trim()}`)
    verifyResult.value = { ok: localSign === remoteSign, remoteSign, localSign }
    if (verifyResult.value.ok) {toast.success('验签通过')} else {toast.error('验签失败')}
  } catch (e: any) {
    toast.error(`响应不是合法 JSON：${e.message}`)
  }
}

function loadVerifySample() {
  verifyInput.value = JSON.stringify({
    code: 0,
    data: { orderNo: 'VQ20260901001', status: 'SUCCESS' },
    msg: '',
    sign: '（用「请求签名」页算出后填入）',
  }, null, 2)
  if (!signKey.value) {signKey.value = 'pDwkFKUZv60KWs53'}
}

// ===== AES-ECB 字段加解密 =====
const aesKey = ref('')
const aesMode = ref<'encrypt' | 'decrypt'>('encrypt')
const aesInput = ref('')
const aesOutput = ref('')

function processAes() {
  aesOutput.value = ''
  const keyHex = aesKey.value.trim().replace(/\s+/g, '')
  if (!/^[0-9a-fA-F]{32}$/.test(keyHex)) {
    toast.warning('AES 密钥需要 32 位 hex（16 字节），如 238be0efdc748197317021efdbc9ecde')
    return
  }
  if (!aesInput.value.trim()) {
    toast.warning('请输入内容')
    return
  }
  try {
    // HexUtil.decodeHex(key) → WordArray；ECB + PKCS7（= Java PKCS5Padding）
    const keyWA = CryptoJS.enc.Hex.parse(keyHex)
    if (aesMode.value === 'encrypt') {
      const encrypted = CryptoJS.AES.encrypt(aesInput.value, keyWA, {
        mode: CryptoJS.mode.ECB,
        padding: CryptoJS.pad.Pkcs7,
      })
      aesOutput.value = encrypted.toString() // base64
    } else {
      const decrypted = CryptoJS.AES.decrypt(aesInput.value.trim(), keyWA, {
        mode: CryptoJS.mode.ECB,
        padding: CryptoJS.pad.Pkcs7,
      })
      const text = decrypted.toString(CryptoJS.enc.Utf8)
      if (!text) {
        toast.error('解密失败：密钥不正确或密文已损坏')
        return
      }
      aesOutput.value = text
    }
    toast.success(aesMode.value === 'encrypt' ? '加密完成' : '解密完成')
  } catch (e: any) {
    toast.error(`AES ${aesMode.value === 'encrypt' ? '加密' : '解密'}失败: ${e.message}`)
  }
}

function loadAesSample() {
  if (!aesKey.value) {aesKey.value = '238be0efdc748197317021efdbc9ecde'}
  aesInput.value = aesMode.value === 'encrypt' ? '13800000000' : ''
}

function copyAes() {
  copyText(aesOutput.value, toast)
}
</script>
