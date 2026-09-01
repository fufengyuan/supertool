<template>
  <ToolPage
    icon="key"
    name="加密/解密"
    description="AES / DES / TripleDES / RC4 / Rabbit / SM4 / SM2 / RSA 加解密，支持密钥生成"
    @back="$emit('back')"
  >
    <!-- 算法与模式 -->
    <div class="flex flex-wrap items-end gap-3 bg-base-100 border border-base-content/10 rounded-xl p-4">
      <div class="min-w-[220px] flex-1">
        <span class="text-[11px] font-medium text-base-content/50 mb-1 block">算法</span>
        <select v-model="algorithm" class="select select-bordered select-sm w-full bg-base-200/60" @change="onAlgorithmChange">
          <option value="AES">AES (对称, 128/192/256 bit)</option>
          <option value="DES">DES (对称, 64 bit)</option>
          <option value="TripleDES">TripleDES (对称, 192 bit)</option>
          <option value="RC4">RC4 (对称, 40-2048 bit)</option>
          <option value="Rabbit">Rabbit (对称, 128 bit)</option>
          <option value="SM4">SM4 (对称, 128 bit)</option>
          <option value="SM2">SM2 (非对称, 256 bit ECC)</option>
          <option value="RSA">RSA (非对称, 2048 bit)</option>
        </select>
      </div>
      <div>
        <span class="text-[11px] font-medium text-base-content/50 mb-1 block">模式</span>
        <div class="join">
          <button class="btn btn-sm join-item" :class="mode === 'encrypt' ? 'btn-primary' : 'btn-ghost'" @click="mode = 'encrypt'">加密</button>
          <button class="btn btn-sm join-item" :class="mode === 'decrypt' ? 'btn-primary' : 'btn-ghost'" @click="mode = 'decrypt'">解密</button>
        </div>
      </div>
      <!-- SM2 密文排列：业务端（sm-crypto 默认）普遍 C1C3C4 -->
      <div v-if="algorithm === 'SM2'">
        <span class="text-[11px] font-medium text-base-content/50 mb-1 block">密文排列</span>
        <select v-model="sm2CipherMode" class="select select-bordered select-sm w-[150px] bg-base-200/60" title="C1C3C4 为 sm-crypto / 国密标准默认排列">
          <option value="1">C1C3C4（标准/默认）</option>
          <option value="0">C1C2C3（旧标准）</option>
        </select>
      </div>
      <!-- SM2 04 前缀：业务端常带 04 未压缩点前缀 -->
      <div v-if="algorithm === 'SM2'">
        <span class="text-[11px] font-medium text-base-content/50 mb-1 block">04 前缀</span>
        <div class="join">
          <button class="btn btn-sm join-item" :class="sm2Prefix04 ? 'btn-primary' : 'btn-ghost'" @click="sm2Prefix04 = true" title="密文自动加/剥 04 前缀（buyer-mobile smUtil 风格）">带 04</button>
          <button class="btn btn-sm join-item" :class="!sm2Prefix04 ? 'btn-primary' : 'btn-ghost'" @click="sm2Prefix04 = false" title="纯 C1C3C4/C1C2C3 hex，无前缀">不带</button>
        </div>
      </div>
      <!-- SM4 密钥格式：业务端（sm-crypto sm4）直接吃 32 位 hex；文本密钥需逐字符转 hex -->
      <div v-if="algorithm === 'SM4'">
        <span class="text-[11px] font-medium text-base-content/50 mb-1 block">密钥格式</span>
        <select v-model="sm4KeyStyle" class="select select-bordered select-sm w-[170px] bg-base-200/60" title="16位文本密钥：逐字符 charCodeAt → 2位hex 拼接（buyer-mobile SM4Util.convertKey 语义）">
          <option value="hex">标准（hex/base64/文本字节）</option>
          <option value="text16">16位文本密钥 → hex</option>
        </select>
      </div>
    </div>

    <!-- 对称密钥 -->
    <template v-if="!isAsymmetric">
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-4 flex flex-col gap-3">
        <div>
          <div class="flex items-center justify-between mb-1">
            <span class="text-[11px] font-medium text-base-content/50">密钥 (Key) <span class="text-base-content/30">{{ keyHint }}</span></span>
            <label class="flex items-center gap-1 text-[11px] text-base-content/50">
              格式
              <select v-model="keyFormat" class="select select-xs select-bordered bg-base-200/60 w-[90px]" title="密钥解析格式">
                <option value="hex">Hex</option>
                <option value="base64">Base64</option>
                <option value="utf8">UTF-8 文本</option>
              </select>
            </label>
          </div>
          <div class="flex gap-2">
            <input
              v-model="key"
              class="input input-bordered input-sm w-full font-mono text-xs bg-base-200/60"
              type="text"
              :placeholder="keyPlaceholder"
            />
            <button class="btn btn-outline btn-sm" @click="generateKey" title="生成随机密钥">🎲 生成</button>
          </div>
        </div>
        <div v-if="showIV">
          <div class="flex items-center justify-between mb-1">
            <span class="text-[11px] font-medium text-base-content/50">初始向量 (IV) <span class="text-base-content/30">16 字节</span></span>
            <label class="flex items-center gap-1 text-[11px] text-base-content/50">
              格式
              <select v-model="ivFormat" class="select select-xs select-bordered bg-base-200/60 w-[90px]" title="IV 解析格式">
                <option value="hex">Hex</option>
                <option value="base64">Base64</option>
                <option value="utf8">UTF-8 文本</option>
              </select>
            </label>
          </div>
          <div class="flex gap-2">
            <input
              v-model="iv"
              class="input input-bordered input-sm w-full font-mono text-xs bg-base-200/60"
              type="text"
              :placeholder="ivPlaceholder"
            />
            <button class="btn btn-outline btn-sm" @click="generateIV" title="生成随机 IV">🎲 生成</button>
          </div>
        </div>
      </div>
    </template>

    <!-- 非对称密钥 -->
    <template v-if="isAsymmetric">
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
          <span class="text-[11px] font-medium text-base-content/50 mb-1 block">公钥 (Public Key) <span class="text-base-content/30">用于加密</span></span>
          <textarea
            v-model="asymmetricPublicKey"
            class="textarea textarea-bordered w-full font-mono text-xs bg-base-200/60 min-h-[100px] resize-none"
            :placeholder="mode === 'encrypt' ? '输入或生成公钥...' : '加密后的数据'"
          ></textarea>
          <button class="btn btn-outline btn-sm mt-2" @click="generateAsymmetricKeys">🎲 生成密钥对</button>
        </div>
        <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
          <span class="text-[11px] font-medium text-base-content/50 mb-1 block">私钥 (Private Key) <span class="text-base-content/30">用于解密</span></span>
          <textarea
            v-model="asymmetricPrivateKey"
            class="textarea textarea-bordered w-full font-mono text-xs bg-base-200/60 min-h-[100px] resize-none"
            :placeholder="mode === 'decrypt' ? '输入或生成私钥...' : '留空（加密不需要私钥）'"
          ></textarea>
          <button class="btn btn-outline btn-sm mt-2" @click="copyAsymmetricKeys" v-if="asymmetricPublicKey && asymmetricPrivateKey"><SvgIcon name="copy" size="12" /> 复制密钥对</button>
        </div>
      </div>
    </template>

    <!-- 输入输出 -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
      <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4 min-h-[200px]">
        <div class="flex items-center justify-between mb-2.5">
          <h4 class="text-xs font-semibold text-base-content/70 flex items-center gap-1.5"><SvgIcon name="arrowDown" size="12" /> 输入</h4>
          <label class="flex items-center gap-1 text-[11px] text-base-content/50">
            明文格式
            <select v-model="plainFormat" class="select select-xs select-bordered bg-base-200/60 w-[110px]" :disabled="isAsymmetric" :title="isAsymmetric ? '非对称加密仅支持 UTF-8 文本' : '输入内容的解析/输出内容的编码格式'">
              <option value="utf8">UTF-8</option>
              <option value="hex">Hex</option>
              <option value="base64">Base64</option>
            </select>
          </label>
        </div>
        <textarea
          v-model="inputText"
          class="textarea textarea-bordered w-full min-h-[110px] font-mono text-xs flex-1 resize-none bg-base-200/60"
          :placeholder="inputPlaceholder"
        ></textarea>
      </div>
      <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4 min-h-[200px]">
        <div class="flex items-center justify-between mb-2.5">
          <h4 class="text-xs font-semibold text-base-content/70 flex items-center gap-1.5"><SvgIcon name="arrowUp" size="12" /> 输出</h4>
          <div class="flex items-center gap-1.5">
            <label class="flex items-center gap-1 text-[11px] text-base-content/50" v-if="mode === 'encrypt' || !isAsymmetric">
              密文格式
              <select v-model="cipherFormat" class="select select-xs select-bordered bg-base-200/60 w-[90px]" title="密文输出/输入格式（base64 兼容常见工具）">
                <option value="base64">Base64</option>
                <option value="hex">Hex</option>
              </select>
            </label>
            <button class="btn btn-primary btn-xs" @click="copyResult" :disabled="!outputText"><SvgIcon name="copy" size="11" /> 复制</button>
            <button class="btn btn-ghost btn-xs" @click="clearAll" :disabled="!inputText && !outputText">清空</button>
          </div>
        </div>
        <div class="flex-1 p-3 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-xs whitespace-pre-wrap break-all max-h-[220px] overflow-y-auto min-h-[80px]">{{ outputText || '结果将显示在这里...' }}</div>
      </div>
    </div>

    <!-- 操作 -->
    <div class="flex gap-2 bg-base-100 border border-base-content/10 rounded-xl px-4 py-3">
      <button class="btn btn-primary btn-sm flex-1 max-w-[160px]" @click="process">
        {{ mode === 'encrypt' ? '加密' : '解密' }}
      </button>
    </div>
  </ToolPage>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { ref, computed } from 'vue'
import CryptoJS from 'crypto-js'
import { SM4 } from 'gm-crypto'
import { sm2 } from 'sm-crypto'
import JSEncrypt from 'jsencrypt'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

const toast = useToast()

const algorithm = ref('AES')
const mode = ref<'encrypt' | 'decrypt'>('encrypt')
const key = ref('')
const iv = ref('')
const inputText = ref('')
const outputText = ref('')
// 格式控制：密钥/IV/明文（输入内容）均支持 hex / base64 / UTF-8 文本；密文支持 base64 / hex
const keyFormat = ref<'hex' | 'base64' | 'utf8'>('hex')
const ivFormat = ref<'hex' | 'base64' | 'utf8'>('hex')
const plainFormat = ref<'utf8' | 'hex' | 'base64'>('utf8')
const cipherFormat = ref<'base64' | 'hex'>('base64')
// SM2：密文排列（1=C1C3C4 sm-crypto 默认，0=C1C2C3 旧标准）+ 04 未压缩点前缀（buyer-mobile smUtil 风格）
const sm2CipherMode = ref<'1' | '0'>('1')
const sm2Prefix04 = ref(true)
// SM4：密钥格式风格。text16 = 16 位文本密钥逐字符 charCodeAt→2位hex（业务端 SM4Util.convertKey 语义）
const sm4KeyStyle = ref<'hex' | 'text16'>('hex')

// SM2/RSA keys
const asymmetricPublicKey = ref('')
const asymmetricPrivateKey = ref('')

const isAsymmetric = computed(() => algorithm.value === 'SM2' || algorithm.value === 'RSA')
const showIV = computed(() => algorithm.value === 'AES' || algorithm.value === 'SM4')

interface KeyConfig {
  name: string
  keyBits: number
  keyBytes: number
  description: string
  defaultBits?: number
}

const keyConfigs: Record<string, KeyConfig> = {
  AES: { name: 'AES', keyBits: 256, keyBytes: 32, description: '256-bit (32 字节, 64 hex)', defaultBits: 256 },
  DES: { name: 'DES', keyBits: 64, keyBytes: 8, description: '64-bit (8 字节, 16 hex)' },
  TripleDES: { name: 'TripleDES', keyBits: 192, keyBytes: 24, description: '192-bit (24 字节, 48 hex)' },
  RC4: { name: 'RC4', keyBits: 128, keyBytes: 16, description: '128-bit (16 字节, 32 hex), 支持 40-2048 bit' },
  Rabbit: { name: 'Rabbit', keyBits: 128, keyBytes: 32, description: '128-bit (32 hex 作为 passphrase)' },
  SM4: { name: 'SM4', keyBits: 128, keyBytes: 16, description: '128-bit (16 字节, 32 hex)' },
}

const keyHint = computed(() => {
  const cfg = keyConfigs[algorithm.value]
  if (!cfg) {return ''}
  if (keyFormat.value === 'utf8') {return `${cfg.name} ${cfg.keyBits}bit，可输入任意文本密钥`}
  const bytes = cfg.keyBytes
  return keyFormat.value === 'hex' ? `${bytes * 2} 位 hex（${bytes} 字节）` : `Base64（${bytes} 字节 ≈ ${Math.ceil(bytes / 3) * 4} 字符）`
})

const keyPlaceholder = computed(() => {
  const cfg = keyConfigs[algorithm.value]
  if (!cfg) {return '输入密钥'}
  if (keyFormat.value === 'utf8') {return `${cfg.keyBits}bit 文本密钥...`}
  return keyFormat.value === 'hex' ? `${cfg.keyBytes * 2} 位 hex 密钥...` : `${cfg.keyBytes} 字节 base64 密钥...`
})

const ivPlaceholder = computed(() => {
  return ivFormat.value === 'hex' ? 'hex 格式，留空自动生成' : ivFormat.value === 'base64' ? 'base64 格式，留空自动生成' : 'UTF-8 文本，留空自动生成'
})

const inputPlaceholder = computed(() => {
  if (isAsymmetric.value) {
    return mode.value === 'encrypt' ? '输入要加密的文本（仅支持 UTF-8）...' : '输入要解密的文本（base64）...'
  }
  const fmt = mode.value === 'encrypt' ? plainFormat.value : cipherFormat.value
  const fmtName = fmt === 'hex' ? 'hex' : fmt === 'base64' ? 'base64' : 'UTF-8 文本'
  return mode.value === 'encrypt' ? `输入要加密的内容（${fmtName}）...` : `输入要解密的密文（${fmtName}）...`
})

function randomBytes(length: number): Uint8Array {
  const bytes = new Uint8Array(length)
  crypto.getRandomValues(bytes)
  return bytes
}

function bytesToHex(bytes: Uint8Array | ArrayBuffer): string {
  const arr = bytes instanceof ArrayBuffer ? new Uint8Array(bytes) : bytes
  return Array.from(arr, b => b.toString(16).padStart(2, '0')).join('')
}

function bytesToBase64(bytes: Uint8Array | ArrayBuffer): string {
  const arr = bytes instanceof ArrayBuffer ? new Uint8Array(bytes) : bytes
  let bin = ''
  for (let i = 0; i < arr.length; i++) {bin += String.fromCharCode(arr[i])}
  return btoa(bin)
}

function hexToBytes(hex: string): Uint8Array {
  const clean = hex.replace(/\s+/g, '')
  if (clean.length % 2 !== 0) {throw new Error('Hex 长度必须为偶数')}
  if (!/^[0-9a-fA-F]+$/.test(clean)) {throw new Error('无效的 Hex 字符串')}
  const bytes = new Uint8Array(clean.length / 2)
  for (let i = 0; i < clean.length; i += 2) {
    bytes[i / 2] = parseInt(clean.substring(i, i + 2), 16)
  }
  return bytes
}

function base64ToBytes(b64: string): Uint8Array {
  const bin = atob(b64.replace(/\s+/g, ''))
  const bytes = new Uint8Array(bin.length)
  for (let i = 0; i < bin.length; i++) {bytes[i] = bin.charCodeAt(i)}
  return bytes
}

function utf8ToBytes(text: string): Uint8Array {
  return new TextEncoder().encode(text)
}

function bytesToUtf8(bytes: Uint8Array): string {
  try {
    return new TextDecoder('utf-8', { fatal: true }).decode(bytes)
  } catch {
    throw new Error('解密结果无法按 UTF-8 解码，请改用 Hex/Base64 明文格式查看')
  }
}

// 按格式解析输入为字节（明文 / 密文通用）
function parseBytes(str: string, format: 'utf8' | 'hex' | 'base64'): Uint8Array {
  if (format === 'hex') {return hexToBytes(str)}
  if (format === 'base64') {return base64ToBytes(str)}
  return utf8ToBytes(str)
}

// 字节按格式编码为字符串（明文输出用）
function bytesToString(bytes: Uint8Array, format: 'utf8' | 'hex' | 'base64'): string {
  if (format === 'hex') {return bytesToHex(bytes)}
  if (format === 'base64') {return bytesToBase64(bytes)}
  return bytesToUtf8(bytes)
}

// Uint8Array ↔ CryptoJS WordArray（crypto-js 类型为任意声明，这里用轻量接口）
// 注意：WordArray.create 直接持有同一 ArrayBuffer，勿再修改源数组
interface WordArrayLike {
  words: number[]
  sigBytes: number
}

function bytesToWordArray(bytes: Uint8Array): WordArrayLike {
  return (CryptoJS as any).lib.WordArray.create(bytes) as WordArrayLike
}

function wordArrayToBytes(wa: WordArrayLike): Uint8Array {
  const words = wa.words
  const sigBytes = wa.sigBytes
  const bytes = new Uint8Array(sigBytes)
  for (let i = 0; i < sigBytes; i++) {
    bytes[i] = (words[i >>> 2] >>> (24 - (i % 4) * 8)) & 0xff
  }
  return bytes
}

function generateKey() {
  const cfg = keyConfigs[algorithm.value]
  if (!cfg) {return}
  if (algorithm.value === 'SM4' && sm4KeyStyle.value === 'text16') {
    // 16 位文本密钥：生成 16 字符可读串（业务端 generateKey 同款字符集）
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789'
    key.value = Array.from(randomBytes(16), b => chars[b % chars.length]).join('')
    toast.success('已生成 16 位随机文本密钥（转 hex 后为 32 位）')
    return
  }
  if (keyFormat.value === 'utf8') {
    // 文本密钥：生成可读随机串
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*'
    key.value = Array.from(randomBytes(cfg.keyBytes), b => chars[b % chars.length]).join('')
  } else {
    const bytes = randomBytes(cfg.keyBytes)
    key.value = keyFormat.value === 'hex' ? bytesToHex(bytes) : bytesToBase64(bytes)
  }
  toast.success(`已生成 ${cfg.name} ${cfg.keyBits}bit 随机密钥 (${keyFormat.value === 'utf8' ? '文本' : cfg.keyBytes + ' 字节'})`)
}

function generateIV() {
  makeRandomIv()
  toast.success(`已生成随机 IV (128 bit, ${ivFormat.value === 'hex' ? 'hex' : ivFormat.value === 'base64' ? 'base64' : 'UTF-8'})`)
}

// 生成随机 IV 并回填 iv 输入框：UTF-8 格式下随机字节几乎必然不是合法 UTF-8，
// 改为生成 16 字符可读串（ASCII，UTF-8 编码后恰为 16 字节），其余格式用随机字节
function makeRandomIv(): Uint8Array {
  if (ivFormat.value === 'utf8') {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*'
    iv.value = Array.from(randomBytes(16), b => chars[b % chars.length]).join('')
  } else {
    const bytes = randomBytes(16)
    iv.value = ivFormat.value === 'hex' ? bytesToHex(bytes) : bytesToBase64(bytes)
  }
  return parseBytes(iv.value, ivFormat.value)
}

function generateAsymmetricKeys() {
  if (algorithm.value === 'SM2') {
    const keyPair = sm2.generateKeyPairHex()
    asymmetricPublicKey.value = keyPair.publicKey
    asymmetricPrivateKey.value = keyPair.privateKey
    toast.success('已生成 SM2 密钥对 (256 bit ECC)')
  } else if (algorithm.value === 'RSA') {
    const key = new JSEncrypt({ default_key_size: '2048' })
    key.getKey()
    asymmetricPublicKey.value = key.getPublicKey()
    asymmetricPrivateKey.value = key.getPrivateKey()
    toast.success('已生成 RSA 2048 bit 密钥对')
  }
}

function copyAsymmetricKeys() {
  const text = `公钥:\n${asymmetricPublicKey.value}\n\n私钥:\n${asymmetricPrivateKey.value}`
  copyText(text, toast, `${algorithm.value} 密钥对已复制`)
}

function onAlgorithmChange() {
  key.value = ''
  iv.value = ''
  asymmetricPublicKey.value = ''
  asymmetricPrivateKey.value = ''
  outputText.value = ''
}

function process() {
  if (!inputText.value.trim()) {
    toast.warning('请输入内容')
    return
  }

  if (algorithm.value === 'SM2' || algorithm.value === 'RSA') {
    processAsymmetric()
    return
  }

  const cfg = keyConfigs[algorithm.value]
  if (!cfg) {
    toast.error('不支持的算法')
    return
  }

  if (!key.value.trim()) {
    toast.warning('请输入密钥')
    return
  }

  // Rabbit 使用 passphrase 语义（EVP_BytesToKey），仅支持文本密钥
  if (algorithm.value === 'Rabbit' && keyFormat.value !== 'utf8') {
    toast.warning('Rabbit 使用口令派生密钥，密钥格式请选择 UTF-8 文本')
    return
  }

  try {
    if (algorithm.value === 'SM4') {
      processSM4()
      return
    }

    // Rabbit: passphrase 直接传入（CryptoJS 内部做 EVP_BytesToKey）
    if (algorithm.value === 'Rabbit') {
      processRabbit()
      return
    }

    // AES, DES, TripleDES, RC4: 密钥按所选格式解析为字节 → WordArray
    const keyWord = bytesToWordArray(parseBytes(key.value.trim(), keyFormat.value))
    let cfg2: any = { mode: CryptoJS.mode.ECB, padding: CryptoJS.pad.Pkcs7 }

    if (showIV.value) {
      cfg2.mode = CryptoJS.mode.CBC
      if (iv.value.trim()) {
        cfg2.iv = bytesToWordArray(parseBytes(iv.value.trim(), ivFormat.value))
      } else {
        if (mode.value === 'encrypt') {
          const ivBytes = makeRandomIv()
          cfg2.iv = bytesToWordArray(ivBytes)
          toast.info(`加密使用随机 IV: ${iv.value}`)
        } else {
          toast.warning('CBC 模式解密需要提供 IV')
          return
        }
      }
    }

    const cryptoModule = (CryptoJS as any)[algorithm.value]
    if (!cryptoModule) {
      toast.error(`算法 ${algorithm.value} 不可用`)
      return
    }

    if (mode.value === 'encrypt') {
      // 明文按所选格式解析为字节后加密，输出密文按密文格式编码
      const plainWA = bytesToWordArray(parseBytes(inputText.value, plainFormat.value))
      const result = cryptoModule.encrypt(plainWA, keyWord, cfg2)
      const ctBytes = wordArrayToBytes(result.ciphertext)
      outputText.value = cipherFormat.value === 'hex' ? bytesToHex(ctBytes) : bytesToBase64(ctBytes)
    } else {
      // 密文按密文格式解析（base64 / hex），与加密输出完全对称
      const ctBytes = parseBytes(inputText.value, cipherFormat.value)
      const result = cryptoModule.decrypt({ ciphertext: bytesToWordArray(ctBytes) }, keyWord, cfg2)
      const plainBytes = wordArrayToBytes(result)
      if (plainBytes.length === 0) {
        toast.error('解密失败：密钥不正确或数据已损坏')
        outputText.value = ''
        return
      }
      outputText.value = bytesToString(plainBytes, plainFormat.value)
    }
  } catch (e: any) {
    toast.error(`${mode.value === 'encrypt' ? '加密' : '解密'}失败: ${e.message}`)
    outputText.value = ''
  }
}

function processRabbit() {
  // Rabbit 使用 passphrase 派生密钥（EVP_BytesToKey + 随机 salt，OpenSSL 格式）。
  // 密文 = Salted__(8B) + salt(8B) + ciphertext，与 openssl enc / 旧版 base64 输出兼容。
  const keyStr = key.value.trim()
  const SALTED = [0x53, 0x61, 0x6c, 0x74, 0x65, 0x64, 0x5f, 0x5f] // "Salted__"
  if (mode.value === 'encrypt') {
    const plainWA = bytesToWordArray(parseBytes(inputText.value, plainFormat.value))
    const result = CryptoJS.Rabbit.encrypt(plainWA, keyStr)
    const saltBytes = result.salt ? wordArrayToBytes(result.salt) : new Uint8Array(0)
    const ctBytes = wordArrayToBytes(result.ciphertext)
    const combined = new Uint8Array(8 + saltBytes.length + ctBytes.length)
    combined.set(SALTED, 0)
    combined.set(saltBytes, 8)
    combined.set(ctBytes, 8 + saltBytes.length)
    outputText.value = cipherFormat.value === 'hex' ? bytesToHex(combined) : bytesToBase64(combined)
  } else {
    const rawBytes = parseBytes(inputText.value, cipherFormat.value)
    // CryptoJS OpenSSL 解析要求 base64 字符串（含 Salted__ 前缀），先统一转回
    const opensslStr = bytesToBase64(rawBytes)
    const result = CryptoJS.Rabbit.decrypt(opensslStr, keyStr)
    const plainBytes = wordArrayToBytes(result)
    if (plainBytes.length === 0) {
      toast.error('解密失败：密钥不正确或数据已损坏')
      outputText.value = ''
      return
    }
    outputText.value = bytesToString(plainBytes, plainFormat.value)
  }
}

function toArrayBuffer(bytes: Uint8Array): ArrayBuffer {
  const copy = new Uint8Array(bytes)
  return copy.buffer
}

function processSM4() {
  try {
    let keyHex: string
    if (sm4KeyStyle.value === 'text16') {
      // 业务端 SM4Util.convertKey 语义：16 位文本密钥逐字符 charCodeAt → 2 位 hex 拼接成 32 位 hex
      const k = key.value.trim()
      if (k.length !== 16) {
        toast.warning(`16位文本密钥模式要求密钥恰好 16 个字符，当前 ${k.length} 个`)
        return
      }
      keyHex = Array.from(k, ch => ch.charCodeAt(0).toString(16).padStart(2, '0')).join('')
    } else {
      // 标准模式：密钥按所选格式解析为字节，不足 16 字节右侧补 0 / 超出截断
      const keyBytes = parseBytes(key.value.trim(), keyFormat.value)
      const key16 = new Uint8Array(16)
      key16.set(keyBytes.slice(0, 16))
      keyHex = bytesToHex(key16)
    }
    let ivBytes: Uint8Array
    if (iv.value.trim()) {
      ivBytes = parseBytes(iv.value.trim(), ivFormat.value)
    } else if (mode.value === 'encrypt') {
      ivBytes = makeRandomIv()
      toast.info(`加密使用随机 IV: ${iv.value}`)
    } else {
      toast.warning('CBC 模式解密需要提供 IV')
      return
    }
    const iv16 = new Uint8Array(16)
    iv16.set(ivBytes.slice(0, 16))
    const ivHex = bytesToHex(iv16)

    if (mode.value === 'encrypt') {
      const inputAB = toArrayBuffer(parseBytes(inputText.value, plainFormat.value))
      const encrypted = SM4.encrypt(inputAB, keyHex, {
        mode: SM4.constants.CBC,
        iv: ivHex,
        outputEncoding: cipherFormat.value === 'hex' ? 'hex' : 'base64',
      })
      outputText.value = typeof encrypted === 'string' ? encrypted : bytesToBase64(new Uint8Array(encrypted))
    } else {
      const ctBytes = parseBytes(inputText.value, cipherFormat.value)
      // 解密输出统一走 hex（无损），再按明文格式转换
      const decrypted = SM4.decrypt(toArrayBuffer(ctBytes), keyHex, {
        mode: SM4.constants.CBC,
        iv: ivHex,
        outputEncoding: 'hex',
      })
      const plainBytes = hexToBytes(typeof decrypted === 'string' ? decrypted : bytesToHex(new Uint8Array(decrypted)))
      outputText.value = bytesToString(plainBytes, plainFormat.value)
    }
  } catch (e: any) {
    toast.error(`SM4 ${mode.value === 'encrypt' ? '加密' : '解密'}失败: ${e.message}`)
    outputText.value = ''
  }
}

function processAsymmetric() {
  try {
    if (algorithm.value === 'SM2') {
      processSM2Internal()
    } else if (algorithm.value === 'RSA') {
      processRSA()
    }
  } catch (e: any) {
    toast.error(`${algorithm.value} ${mode.value === 'encrypt' ? '加密' : '解密'}失败: ${e.message}`)
    outputText.value = ''
  }
}

// 非对称库（sm-crypto / jsencrypt）内部按 UTF-8 处理文本，加密输入仅支持 UTF-8
function checkAsymmetricPlainFormat() {
  if (mode.value === 'encrypt' && plainFormat.value !== 'utf8') {
    toast.warning('非对称加密（SM2/RSA）明文仅支持 UTF-8 文本，如需加密字节请先用 Base64/Hex 转换工具')
    return false
  }
  return true
}

function processSM2Internal() {
  if (!checkAsymmetricPlainFormat()) {return}
  const cipherMode = Number(sm2CipherMode.value) // 1=C1C3C4（sm-crypto/业务端默认），0=C1C2C3
  if (mode.value === 'encrypt') {
    if (!asymmetricPublicKey.value.trim()) {
      toast.warning('加密需要公钥')
      return
    }
    // 公钥自动补 04 未压缩点前缀（业务端 getSm2DataHexByString 同款容错）
    let pubKey = asymmetricPublicKey.value.trim().replace(/\s+/g, '')
    if (!pubKey.startsWith('04')) {pubKey = '04' + pubKey}
    // doEncrypt 输出不含 04 前缀；带 04 风格时补上
    const encryptedHex = sm2.doEncrypt(inputText.value, pubKey, cipherMode)
    const withPrefix = '04' + encryptedHex
    const finalHex = sm2Prefix04.value ? withPrefix : encryptedHex
    outputText.value = cipherFormat.value === 'hex' ? finalHex : bytesToBase64(hexToBytes(finalHex))
  } else {
    if (!asymmetricPrivateKey.value.trim()) {
      toast.warning('解密需要私钥')
      return
    }
    // sm-crypto 接收无 04 前缀的 hex 密文；输入可能是 base64 / hex，且可能带 04 前缀
    let inputHex = cipherFormat.value === 'hex'
      ? inputText.value.trim().replace(/\s+/g, '')
      : bytesToHex(base64ToBytes(inputText.value.trim()))
    if (sm2Prefix04.value && inputHex.toLowerCase().startsWith('04')) {
      inputHex = inputHex.slice(2)
    }
    const decrypted = sm2.doDecrypt(inputHex, asymmetricPrivateKey.value.trim(), cipherMode)
    if (!decrypted) {
      toast.error('解密失败：私钥不匹配、密文排列/04前缀选项不对或数据已损坏')
      outputText.value = ''
      return
    }
    outputText.value = bytesToString(utf8ToBytes(decrypted), plainFormat.value)
  }
}

function processRSA() {
  if (mode.value === 'encrypt') {
    if (!asymmetricPublicKey.value.trim()) {
      toast.warning('加密需要公钥')
      return
    }
    if (!checkAsymmetricPlainFormat()) {return}
    const key = new JSEncrypt()
    key.setPublicKey(asymmetricPublicKey.value.trim())
    const encrypted = key.encrypt(inputText.value)
    if (!encrypted) {
      toast.error('加密失败：文本过长或公钥无效')
      outputText.value = ''
      return
    }
    // jsencrypt 输出 base64；选择 hex 时互转
    outputText.value = cipherFormat.value === 'base64' ? encrypted : bytesToHex(base64ToBytes(encrypted))
  } else {
    if (!asymmetricPrivateKey.value.trim()) {
      toast.warning('解密需要私钥')
      return
    }
    const key = new JSEncrypt()
    key.setPrivateKey(asymmetricPrivateKey.value.trim())
    // jsencrypt 接收 base64 密文；选择 hex 时先互转
    const inputB64 = cipherFormat.value === 'base64' ? inputText.value.trim() : bytesToBase64(hexToBytes(inputText.value.trim()))
    const decrypted = key.decrypt(inputB64)
    if (!decrypted) {
      toast.error('解密失败：私钥不匹配或数据已损坏')
      outputText.value = ''
      return
    }
    outputText.value = bytesToString(utf8ToBytes(decrypted), plainFormat.value)
  }
}

function copyResult() {
  if (!outputText.value) {
    toast.warning('没有可复制的结果')
    return
  }
  copyText(outputText.value, toast)
}

function clearAll() {
  inputText.value = ''
  outputText.value = ''
  key.value = ''
  iv.value = ''
  asymmetricPublicKey.value = ''
  asymmetricPrivateKey.value = ''
}
</script>