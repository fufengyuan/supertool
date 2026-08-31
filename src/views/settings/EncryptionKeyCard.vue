<template>
  <div class="bg-base-100 border border-base-content/10 rounded-xl p-5 flex flex-col gap-3">
    <div class="flex items-center gap-2 font-semibold text-sm text-base-content">
      <SvgIcon name="key" size="15" />
      <span>加密密钥</span>
      <span v-if="isCustom" class="badge badge-success badge-sm">自定义</span>
      <span v-else class="badge badge-ghost badge-sm">内置默认</span>
    </div>

    <div class="text-xs text-base-content/60 leading-relaxed">
      服务器、数据库、AI 等敏感凭据用它加密后入库；<b>备份文件里保存的是密文</b>。
      把同一份密钥配到另一台机器，导入备份后密码可直接用；不配则密码为空需重新录入。
    </div>

    <div class="flex flex-col gap-1">
      <label class="text-xs text-base-content/60">当前密钥（32 字节 base64）</label>
      <div class="flex gap-2">
        <input
          v-model="keyInput"
          readonly
          class="input input-bordered input-sm flex-1 min-w-[150px] font-mono text-xs"
          placeholder="内置默认密钥（未设置自定义密钥）"
        />
        <button @click="copyKey" class="btn btn-ghost btn-sm" :disabled="!keyInput">复制</button>
        <button @click="revealKey = true" v-if="!revealKey && keyInput" class="btn btn-ghost btn-sm">查看</button>
      </div>
    </div>

    <div class="flex flex-col gap-1">
      <label class="text-xs text-base-content/60">新密钥（留空则随机生成）</label>
      <div class="flex gap-2">
        <input
          v-model="newKeyInput"
          class="input input-bordered input-sm flex-1 min-w-[150px] font-mono text-xs"
          placeholder="留空自动生成 32 字节随机密钥"
        />
        <button @click="rotateKey" class="btn btn-primary btn-sm" :disabled="rotating">
          {{ rotating ? '轮换中...' : '生成并轮换' }}
        </button>
      </div>
    </div>

    <div
      v-if="message"
      class="px-3 py-1.5 rounded-md text-xs border"
      :class="{
        'bg-success/10 text-success border-success/30': messageType === 'success',
        'bg-error/10 text-error border-error/30': messageType === 'error',
        'bg-warning/10 text-warning border-warning/30': messageType === 'warning',
      }"
    >
      {{ message }}
    </div>

    <div v-if="failedList.length" class="px-3 py-2 rounded-md text-xs border bg-error/10 text-error border-error/30 flex flex-col gap-1">
      <span>以下密文无法用当前旧密钥解密（未做任何修改，需先确认来源或重新录入）：</span>
      <span v-for="(f, i) in failedList.slice(0, 5)" :key="i" class="font-mono opacity-80">· {{ f }}</span>
      <span v-if="failedList.length > 5" class="opacity-70">... 共 {{ failedList.length }} 条</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { getTauriAPI } from '../../utils/tauri-api'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref, onMounted } from 'vue'

const keyInput = ref('')
const newKeyInput = ref('')
const isCustom = ref(false)
const rotating = ref(false)
const revealKey = ref(false)
const message = ref('')
const messageType = ref<'success' | 'error' | 'warning'>('success')
const failedList = ref<string[]>([])

async function loadKey() {
  try {
    const r = await getTauriAPI().getEncryptionKey()
    isCustom.value = !!r?.isCustom
    keyInput.value = r?.key || ''
  } catch (e: any) {
    message.value = `读取密钥失败: ${e?.message ?? e}`
    messageType.value = 'error'
  }
}

async function copyKey() {
  if (!keyInput.value) return
  try {
    await navigator.clipboard.writeText(keyInput.value)
    message.value = '密钥已复制到剪贴板'
    messageType.value = 'success'
  } catch {
    message.value = '复制失败，请手动选中复制'
    messageType.value = 'error'
  }
}

async function rotateKey() {
  rotating.value = true
  message.value = ''
  failedList.value = []
  try {
    const r = await getTauriAPI().rotateEncryptionKey(newKeyInput.value.trim() || undefined)
    if (r?.success) {
      message.value = `密钥已轮换，重加密 ${r.reencrypted ?? 0} 条密文（共 ${r.total ?? 0} 条）`
      messageType.value = 'success'
      newKeyInput.value = ''
      await loadKey()
    } else {
      failedList.value = r?.failed ?? []
      message.value = r?.error || '密钥轮换失败'
      messageType.value = 'error'
    }
  } catch (e: any) {
    message.value = `密钥轮换失败: ${e?.message ?? e}`
    messageType.value = 'error'
  } finally {
    rotating.value = false
  }
}

onMounted(loadKey)
</script>
