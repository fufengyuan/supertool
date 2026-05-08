<template>
  <div class="tool-panel">
    <h3>🔒 加密/解密</h3>

    <!-- Algorithm & Mode Selection -->
    <div class="tool-row">
      <div>
        <label class="tool-label">算法</label>
        <select v-model="algorithm" class="tool-select" @change="onAlgorithmChange">
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
        <label class="tool-label">模式</label>
        <div class="tool-btn-group">
          <button
            class="tool-btn"
            :class="{ active: mode === 'encrypt' }"
            @click="mode = 'encrypt'"
          >加密</button>
          <button
            class="tool-btn"
            :class="{ active: mode === 'decrypt' }"
            @click="mode = 'decrypt'"
          >解密</button>
        </div>
      </div>
    </div>

    <!-- Symmetric Key & IV -->
    <template v-if="!isAsymmetric">
      <div class="tool-row">
        <div style="flex: 1;">
          <label class="tool-label">密钥 (Key) <span class="key-hint">{{ keyHint }}</span></label>
          <div style="display: flex; gap: 6px;">
            <input
              v-model="key"
              class="tool-input"
              type="text"
              :placeholder="keyPlaceholder"
              style="flex: 1;"
            />
            <button class="tool-btn" @click="generateKey" title="生成随机密钥">🎲 生成</button>
          </div>
        </div>
        <div v-if="showIV" style="flex: 1;">
          <label class="tool-label">初始向量 (IV) <span class="key-hint">16 字节 (32 hex)</span></label>
          <div style="display: flex; gap: 6px;">
            <input
              v-model="iv"
              class="tool-input"
              type="text"
              placeholder="hex 格式，留空自动生成"
              style="flex: 1;"
            />
            <button class="tool-btn" @click="generateIV" title="生成随机 IV">🎲 生成</button>
          </div>
        </div>
      </div>
    </template>

    <!-- Asymmetric (SM2/RSA) Keys -->
    <template v-if="isAsymmetric">
      <div class="tool-row sm2-keys">
        <div style="flex: 1;">
          <label class="tool-label">公钥 (Public Key) <span class="key-hint">用于加密</span></label>
          <textarea
            v-model="asymmetricPublicKey"
            class="tool-textarea mono"
            :placeholder="mode === 'encrypt' ? '输入或生成公钥...' : '加密后的数据'"
            rows="4"
          ></textarea>
          <button class="tool-btn" @click="generateAsymmetricKeys" style="margin-top: 6px">🎲 生成密钥对</button>
        </div>
        <div style="flex: 1;">
          <label class="tool-label">私钥 (Private Key) <span class="key-hint">用于解密</span></label>
          <textarea
            v-model="asymmetricPrivateKey"
            class="tool-textarea mono"
            :placeholder="mode === 'decrypt' ? '输入或生成私钥...' : '留空（加密不需要私钥）'"
            rows="4"
          ></textarea>
          <button class="tool-btn" @click="copyAsymmetricKeys" style="margin-top: 6px" v-if="asymmetricPublicKey && asymmetricPrivateKey">📋 复制密钥对</button>
        </div>
      </div>
    </template>

    <!-- Input -->
    <div class="tool-section">
      <label class="tool-label">输入</label>
      <textarea
        v-model="inputText"
        class="tool-textarea"
        :placeholder="mode === 'encrypt' ? '输入要加密的文本...' : '输入要解密的文本（hex/C1C3C2 格式）...'"
      ></textarea>
    </div>

    <!-- Action Buttons -->
    <div class="tool-row">
      <button class="tool-btn primary" @click="process">
        {{ mode === 'encrypt' ? '加密' : '解密' }}
      </button>
      <button class="tool-btn" @click="copyResult">复制结果</button>
      <button class="tool-btn" @click="clearAll">清空</button>
    </div>

    <!-- Output -->
    <div class="tool-section">
      <label class="tool-label">输出</label>
      <div class="tool-result">{{ outputText || '结果将显示在这里...' }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, computed } from 'vue'
import CryptoJS from 'crypto-js'
import { SM4 } from 'gm-crypto'
import { sm2 } from 'sm-crypto'
import JSEncrypt from 'jsencrypt'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

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
  if (!cfg) return
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

<style scoped>
.tool-panel h3 {
  font-size: 18px;
  font-weight: 700;
  color: var(--color-base-content);
  margin: 0 0 20px 0;
}

.sm2-keys {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.sm2-keys > div {
  flex: 1;
  min-width: 250px;
}

.key-hint {
  font-weight: 400;
  font-size: 11px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.mono {
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 11px;
}

.tool-row > div {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.tool-section { margin-bottom: 20px; }
.tool-section h4 { font-size: 14px; font-weight: 600; color: var(--color-base-content); margin: 0 0 10px 0; display: flex; align-items: center; gap: 6px; }
.tool-textarea { width: 100%; min-height: 120px; padding: 10px 12px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 8px; font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; font-size: 13px; background: var(--color-base-200); color: var(--color-base-content); resize: vertical; outline: none; }
.tool-textarea:focus { border-color: var(--color-primary); }
.tool-input { width: 100%; padding: 8px 12px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 6px; font-size: 13px; background: var(--color-base-200); color: var(--color-base-content); outline: none; }
.tool-input:focus { border-color: var(--color-primary); }
.tool-row { display: flex; gap: 10px; margin-bottom: 12px; flex-wrap: wrap; }
.tool-btn { padding: 7px 16px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 6px; font-size: 13px; font-weight: 500; cursor: pointer; background: var(--color-base-100); color: var(--color-base-content); transition: all 0.15s; display: inline-flex; align-items: center; gap: 4px; }
.tool-btn:hover { border-color: var(--color-primary); color: var(--color-primary); }
.tool-btn.primary { background: var(--color-primary); color: white; border-color: var(--color-primary); }
.tool-btn.primary:hover { opacity: 0.9; }
.tool-btn-group { display: flex; gap: 4px; }
.tool-btn-group .tool-btn { border-radius: 0; }
.tool-btn-group .tool-btn:first-child { border-radius: 6px 0 0 6px; }
.tool-btn-group .tool-btn:last-child { border-radius: 0 6px 6px 0; }
.tool-btn-group .tool-btn.active { background: var(--color-primary); color: white; border-color: var(--color-primary); position: relative; z-index: 1; }
.tool-result { margin-top: 10px; padding: 10px 12px; background: var(--color-base-200); border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 8px; font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; font-size: 13px; color: var(--color-base-content); white-space: pre-wrap; word-break: break-all; max-height: 300px; overflow-y: auto; }
.tool-label { font-size: 12px; font-weight: 500; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); margin-bottom: 4px; display: block; }
.tool-select { padding: 7px 10px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 6px; font-size: 13px; background: var(--color-base-200); color: var(--color-base-content); outline: none; }
.tool-select:focus { border-color: var(--color-primary); }
.tool-checkbox { display: flex; align-items: center; gap: 6px; font-size: 13px; color: var(--color-base-content); cursor: pointer; }
.tool-divider { border: none; border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); margin: 20px 0; }
</style>
