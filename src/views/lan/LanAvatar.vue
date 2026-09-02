<template>
  <!-- 上传的自定义图片 -->
  <img
    v-if="imageSrc"
    :src="imageSrc"
    class="w-full h-full object-cover"
    :alt="altText"
    draggable="false"
  />

  <!-- 内置 SVG 头像 -->
  <svg
    v-else-if="preset"
    class="w-full h-full block"
    viewBox="0 0 64 64"
    xmlns="http://www.w3.org/2000/svg"
    v-html="presetSvg"
  />

  <!-- 旧数据回退：emoji 直接渲染（未知的 av: 值已在 preset 中兜底，不会走到这里） -->
  <span
    v-else
    class="w-full h-full flex items-center justify-center leading-none select-none"
  >{{ avatar }}</span>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { resolveLanAvatar, LAN_AVATARS } from './avatarPresets'

const props = withDefaults(defineProps<{
  /** 头像字段值：`av:cat` 内置 / `avatar:xxx` 上传图 / 其他按 emoji 回退 */
  avatar?: string | null
  /** 上传图的本地绝对路径（需已通过 convertFileSrc 转换） */
  src?: string
}>(), {
  avatar: '',
  src: '',
})

/** 全局递增序号，保证同页面多个头像的渐变 id 不冲突 */
let uidSeq = 0

const preset = computed(() => {
  const found = resolveLanAvatar(props.avatar)
  if (found) {return found}
  // `av:` 前缀但 key 未知：可能来自更高版本新增的头像（本端旧），
  // 此时渲染字面量毫无意义，兜底为默认头像。
  if (props.avatar?.startsWith('av:')) {return LAN_AVATARS[0]}
  return null
})

/** 把渐变 id 占位符替换为唯一值 */
const presetSvg = computed(() => {
  if (!preset.value) {return ''}
  uidSeq += 1
  const uid = `a${uidSeq}_${Math.random().toString(36).slice(2, 8)}`
  return preset.value.svg.replaceAll('{{uid}}', uid)
})

const imageSrc = computed(() => {
  // 上传图通过 src 传入（父组件负责 convertFileSrc）
  if (props.src) {return props.src}
  return ''
})

const altText = computed(() => preset.value?.label || '头像')
</script>
