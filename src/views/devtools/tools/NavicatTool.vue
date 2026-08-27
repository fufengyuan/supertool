<template>
  <ToolPage
    icon="key"
    name="Navicat 密码加解密"
    description="Navicat 12+ 保存的数据库密码加解密（AES-128-CBC），多行批量处理"
    @back="$emit('back')"
  >
    <!-- 操作栏 -->
    <div class="flex flex-wrap items-center gap-3 bg-base-100 border border-base-content/10 rounded-xl px-4 py-3">
      <button class="btn btn-sm btn-primary gap-1" @click="doEncrypt" :disabled="!plainText">
        <SvgIcon name="arrowDown" size="13" /> 加密
      </button>
      <button class="btn btn-sm gap-1" @click="doDecrypt" :disabled="!encryptedText">
        <SvgIcon name="arrowUp" size="13" /> 解密
      </button>
      <span class="text-xs text-base-content/50 ml-1">
        支持一次粘贴多行（每行独立处理）；密文为 Navicat 存储的大写 HEX
      </span>
      <button class="btn btn-ghost btn-xs ml-auto" @click="swapContent" :disabled="!encryptedText && !plainText" title="双向互换">
        <SvgIcon name="refresh" size="12" /> 互换
      </button>
    </div>

    <!-- 输入/输出 双栏 -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
      <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4 min-h-[300px]">
        <div class="flex items-center justify-between mb-2.5">
          <h4 class="text-xs font-semibold text-base-content/70 flex items-center gap-1.5">
            <SvgIcon name="lockOpen" size="12" /> 明文密码
          </h4>
          <button class="btn btn-ghost btn-xs" @click="plainText = ''" :disabled="!plainText">清空</button>
        </div>
        <textarea
          v-model="plainText"
          class="textarea textarea-bordered w-full font-mono text-sm flex-1 resize-none bg-base-200/60 focus:bg-base-200"
          placeholder="输入要加密的密码（每行一个）..."
          spellcheck="false"
        ></textarea>
      </div>

      <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4 min-h-[300px]">
        <div class="flex items-center justify-between mb-2.5">
          <h4 class="text-xs font-semibold text-base-content/70 flex items-center gap-1.5">
            <SvgIcon name="lock" size="12" /> 密文（Navicat 存储格式）
          </h4>
          <div class="flex gap-1.5">
            <button class="btn btn-ghost btn-xs" @click="encryptedText = ''" :disabled="!encryptedText">清空</button>
            <button class="btn btn-primary btn-xs gap-1" @click="copyResult" :disabled="!encryptedText">
              <SvgIcon name="copy" size="11" /> 复制
            </button>
          </div>
        </div>
        <textarea
          v-model="encryptedText"
          class="textarea textarea-bordered w-full font-mono text-sm flex-1 resize-none bg-base-200/60 focus:bg-base-200"
          placeholder="输入要解密的 HEX 密文（每行一个）..."
          spellcheck="false"
        ></textarea>
      </div>
    </div>

    <!-- 提示 -->
    <div class="flex items-start gap-2 text-xs text-base-content/50 bg-base-100 border border-base-content/10 rounded-xl px-4 py-3">
      <SvgIcon name="helpCircle" size="14" class="mt-0.5 flex-shrink-0 text-primary/70" />
      <div>
        适用于 Navicat 12+（及 11 部分版本）<code class="bg-base-200 px-1 rounded text-primary font-mono">ncx</code> 文件中
        <code class="bg-base-200 px-1 rounded text-primary font-mono">Password</code> 字段的加解密。
        打开 <code class="bg-base-200 px-1 rounded text-primary font-mono">.ncx</code> 连接文件，找到
        <code class="bg-base-200 px-1 rounded text-primary font-mono">&lt;Password&gt;xxxx&lt;/Password&gt;</code>
        内的 HEX 串，粘贴到右侧即可还原数据库密码。全部在本地计算，不会上传任何数据。
      </div>
    </div>
  </ToolPage>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import CryptoJS from 'crypto-js'
import ToolPage from '../components/ToolPage.vue'

defineEmits<{ (e: 'back'): void }>()

const plainText = ref('')
const encryptedText = ref('')

// Navicat 12+：AES-128-CBC，key/iv 固定为 libcc（Latin1 编码）
const NAVICAT_KEY = CryptoJS.enc.Latin1.parse('libcckeylibcckey')
const NAVICAT_IV = CryptoJS.enc.Latin1.parse('libcciv libcciv ')

function hex2bin(hex: string): string {
  const clean = hex.replace(/\s+/g, '')
  if (clean.length % 2 !== 0 || !/^[0-9a-fA-F]*$/.test(clean)) {
    throw new Error('密文不是合法的 HEX 串')
  }
  return (clean.match(/.{1,2}/g) ?? []).map(h => String.fromCharCode(parseInt(h, 16))).join('')
}

function bin2hex(str: string): string {
  return Array.from(str).map(ch => ch.charCodeAt(0).toString(16).padStart(2, '0')).join('')
}

function encryptLine(line: string): string {
  const ciphertext = CryptoJS.AES.encrypt(line, NAVICAT_KEY, { mode: CryptoJS.mode.CBC, iv: NAVICAT_IV }).ciphertext
  return bin2hex(ciphertext.toString(CryptoJS.enc.Latin1)).toUpperCase()
}

function decryptLine(line: string): string {
  const bin = hex2bin(line.trim())
  const ciphertext = CryptoJS.enc.Base64.stringify(CryptoJS.enc.Latin1.parse(bin))
  const dec = CryptoJS.AES.decrypt(ciphertext, NAVICAT_KEY, { mode: CryptoJS.mode.CBC, iv: NAVICAT_IV })
  return dec.toString(CryptoJS.enc.Latin1)
}

function processLines(input: string, fn: (line: string) => string): string {
  return input
    .split('\n')
    .map(l => l.trim())
    .map(fn)
    .join('\n')
}

function doEncrypt() {
  if (!plainText.value) { return }
  try {
    encryptedText.value = processLines(plainText.value, encryptLine)
  } catch (e) {
    encryptedText.value = `加密失败: ${e instanceof Error ? e.message : e}`
  }
}

function doDecrypt() {
  if (!encryptedText.value) { return }
  try {
    plainText.value = processLines(encryptedText.value, decryptLine)
  } catch (e) {
    plainText.value = `解密失败: ${e instanceof Error ? e.message : e}`
  }
}

function swapContent() {
  const tmp = plainText.value
  plainText.value = encryptedText.value
  encryptedText.value = tmp
}

async function copyResult() {
  try {
    await navigator.clipboard.writeText(encryptedText.value)
  } catch {} // eslint-disable-line no-empty
}
</script>