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
    </div>

    <!-- 对称密钥 -->
    <template v-if="!isAsymmetric">
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-4 flex flex-col gap-3">
        <div>
          <span class="text-[11px] font-medium text-base-content/50 mb-1 block">密钥 (Key) <span class="text-base-content/30">{{ keyHint }}</span></span>
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
          <span class="text-[11px] font-medium text-base-content/50 mb-1 block">初始向量 (IV) <span class="text-base-content/30">16 字节 (32 hex)</span></span>
          <div class="flex gap-2">
            <input
              v-model="iv"
              class="input input-bordered input-sm w-full font-mono text-xs bg-base-200/60"
              type="text"
              placeholder="hex 格式，留空自动生成"
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
        <h4 class="text-xs font-semibold text-base-content/70 mb-2.5 flex items-center gap-1.5"><SvgIcon name="arrowDown" size="12" /> 输入</h4>
        <textarea
          v-model="inputText"
          class="textarea textarea-bordered w-full min-h-[110px] font-mono text-xs flex-1 resize-none bg-base-200/60"
          :placeholder="mode === 'encrypt' ? '输入要加密的文本...' : '输入要解密的文本（hex/C1C3C2 格式）...'"
        ></textarea>
      </div>
      <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4 min-h-[200px]">
        <div class="flex items-center justify-between mb-2.5">
          <h4 class="text-xs font-semibold text-base-content/70 flex items-center gap-1.5"><SvgIcon name="arrowUp" size="12" /> 输出</h4>
          <div class="flex gap-1.5">
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
import SvgIcon from '@/components/ui/SvgIcon.vue'// @ts-nocheck
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
  return cfg ? cfg.description : ''
})

const keyPlaceholder = computed(() => {
  const cfg = keyConfigs[algorithm.value]
  return cfg ? `${cfg.keyBytes * 2} 位 hex 密钥...` : '输入密钥'
})

function generateRandomHex(length: number): string {
  const bytes = new Uint8Array(length)
  crypto.getRandomValues(bytes)
  return Array.from(bytes, b => b.toString(16).padStart(2, '0')).join('')
}

function generateKey() {
  const cfg = keyConfigs[algorithm.value]
  if (!cfg) {return}
  key.value = generateRandomHex(cfg.keyBytes)
  toast.success(`已生成 ${cfg.name} ${cfg.keyBits}bit 随机密钥 (${cfg.keyBytes} 字节)`)
}

function generateIV() {
  iv.value = generateRandomHex(16)
  toast.success('已生成随机 IV (128 bit)')
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
    toast.warning('请输入文本')
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

  try {
    if (algorithm.value === 'SM4') {
      processSM4()
      return
    }

    // Rabbit: pass hex string as passphrase (CryptoJS uses EVP_BytesToKey)
    if (algorithm.value === 'Rabbit') {
      processRabbit()
      return
    }

    // AES, DES, TripleDES, RC4: parse hex key as raw WordArray
    const keyHex = key.value.trim()
    const keyWord = CryptoJS.enc.Hex.parse(keyHex)
    let cfg2: any = { mode: CryptoJS.mode.ECB, padding: CryptoJS.pad.Pkcs7 }

    if (showIV.value) {
      cfg2.mode = CryptoJS.mode.CBC
      if (iv.value.trim()) {
        cfg2.iv = CryptoJS.enc.Hex.parse(iv.value.trim())
      } else {
        if (mode.value === 'encrypt') {
          cfg2.iv = CryptoJS.enc.Hex.parse(generateRandomHex(16))
          toast.info(`加密使用随机 IV: ${cfg2.iv.toString()}`)
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
      const result = cryptoModule.encrypt(inputText.value, keyWord, cfg2)
      outputText.value = result.toString()
    } else {
      const result = cryptoModule.decrypt(inputText.value, keyWord, cfg2)
      const decrypted = result.toString(CryptoJS.enc.Utf8)
      if (!decrypted) {
        toast.error('解密失败：密钥不正确或数据已损坏')
        outputText.value = ''
        return
      }
      outputText.value = decrypted
    }
  } catch (e: any) {
    toast.error(`${mode.value === 'encrypt' ? '加密' : '解密'}失败: ${e.message}`)
    outputText.value = ''
  }
}

function processRabbit() {
  // Rabbit uses passphrase-based key derivation (EVP_BytesToKey)
  // Pass the hex key string directly as the passphrase
  const keyStr = key.value.trim()
  if (mode.value === 'encrypt') {
    const result = CryptoJS.Rabbit.encrypt(inputText.value, keyStr)
    outputText.value = result.toString()
  } else {
    const result = CryptoJS.Rabbit.decrypt(inputText.value, keyStr)
    const decrypted = result.toString(CryptoJS.enc.Utf8)
    if (!decrypted) {
      toast.error('解密失败：密钥不正确或数据已损坏')
      outputText.value = ''
      return
    }
    outputText.value = decrypted
  }
}

function processSM4() {
  try {
    const keyHex = key.value.trim()
    const ivHex = iv.value.trim() ? padHex(iv.value.trim(), 32) : padHex(generateRandomHex(16), 32)

    if (mode.value === 'encrypt') {
      const encrypted = SM4.encrypt(inputText.value, keyHex, {
        mode: SM4.constants.CBC,
        iv: ivHex,
        outputEncoding: 'base64',
      })
      outputText.value = typeof encrypted === 'string' ? encrypted : btoa(String.fromCharCode(...new Uint8Array(encrypted)))
    } else {
      const decrypted = SM4.decrypt(inputText.value, keyHex, {
        mode: SM4.constants.CBC,
        iv: ivHex,
        outputEncoding: 'utf8',
      })
      outputText.value = typeof decrypted === 'string' ? decrypted : new TextDecoder().decode(decrypted)
    }
  } catch (e: any) {
    toast.error(`SM4 ${mode.value === 'encrypt' ? '加密' : '解密'}失败: ${e.message}`)
    outputText.value = ''
  }
}

function padHex(hex: string, length: number): string {
  return hex.replace(/\s/g, '').padEnd(length, '0').slice(0, length)
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

function processSM2Internal() {
  if (mode.value === 'encrypt') {
    if (!asymmetricPublicKey.value.trim()) {
      toast.warning('加密需要公钥')
      return
    }
    const encrypted = sm2.doEncrypt(inputText.value, asymmetricPublicKey.value.trim(), 0)
    outputText.value = encrypted
  } else {
    if (!asymmetricPrivateKey.value.trim()) {
      toast.warning('解密需要私钥')
      return
    }
    const decrypted = sm2.doDecrypt(inputText.value.trim(), asymmetricPrivateKey.value.trim(), 0)
    if (!decrypted) {
      toast.error('解密失败：私钥不匹配或数据已损坏')
      outputText.value = ''
      return
    }
    outputText.value = decrypted
  }
}

function processRSA() {
  if (mode.value === 'encrypt') {
    if (!asymmetricPublicKey.value.trim()) {
      toast.warning('加密需要公钥')
      return
    }
    const key = new JSEncrypt()
    key.setPublicKey(asymmetricPublicKey.value.trim())
    const encrypted = key.encrypt(inputText.value)
    if (!encrypted) {
      toast.error('加密失败：文本过长或公钥无效')
      outputText.value = ''
      return
    }
    outputText.value = encrypted
  } else {
    if (!asymmetricPrivateKey.value.trim()) {
      toast.warning('解密需要私钥')
      return
    }
    const key = new JSEncrypt()
    key.setPrivateKey(asymmetricPrivateKey.value.trim())
    const decrypted = key.decrypt(inputText.value.trim())
    if (!decrypted) {
      toast.error('解密失败：私钥不匹配或数据已损坏')
      outputText.value = ''
      return
    }
    outputText.value = decrypted
  }
}

function hexToBytes(hex: string): Uint8Array {
  const clean = hex.replace(/\s/g, '')
  const bytes = new Uint8Array(clean.length / 2)
  for (let i = 0; i < clean.length; i += 2) {
    bytes[i / 2] = parseInt(clean.substring(i, i + 2), 16)
  }
  return bytes
}

function bytesToHex(bytes: Uint8Array | ArrayBuffer): string {
  const arr = bytes instanceof ArrayBuffer ? new Uint8Array(bytes) : bytes
  return Array.from(arr, b => b.toString(16).padStart(2, '0')).join('')
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