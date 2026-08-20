<template>
  <div class="flex flex-col gap-3">
    <div>
      <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">部署模式</label>
      <div class="grid grid-cols-2 gap-2">
        <label
          v-for="mode in modes" :key="mode.key"
          class="flex flex-col border-2 rounded-xl px-3 py-2.5 cursor-pointer transition-all duration-150 relative hover:border-primary"
          :class="isActive(mode.key) ? 'border-primary bg-primary/10' : 'border-base-content/10'"
        >
          <div class="flex items-center gap-2">
            <input type="radio" name="deploy-mode" :value="mode.key" :checked="isActive(mode.key)" @change="pick(mode.key)" class="radio radio-primary radio-sm" />
            <span class="text-sm font-semibold text-base-content">{{ mode.name }}</span>
          </div>
          <span class="mt-1 text-[11px] leading-snug text-base-content/50 pl-6">{{ mode.desc }}</span>
        </label>
      </div>
    </div>

    <!-- Jar 与 Lib 分离：两种部署模式下的公共能力开关 -->
    <div class="flex items-center gap-2 px-3 py-2.5 rounded-lg bg-primary/5 border border-primary/15 text-xs text-base-content/70">
      <input :checked="libSeparate" @change="emit('update:libSeparate', ($event.target as HTMLInputElement).checked)" type="checkbox" class="checkbox checkbox-primary checkbox-sm" />
      <span>启用 Jar 与 Lib 分离：业务 jar 与依赖 lib 分别上传到 {{ deployPath || '部署目录' }} 及其 <code class="bg-base-200 px-1 rounded">lib/</code> 子目录</span>
    </div>
  </div>
</template>

<script setup lang="ts">
// 部署模式共享组件（向导与编辑页共用）：
// monolith=单体部署（parentBuildMode=true，整体打包单 jar）
// multi=多模块部署（parentBuildMode=false，逐模块独立构建）
// Jar/Lib 分离是两种模式下的公共能力开关，非独立模式
const props = withDefaults(defineProps<{
  // 单体部署标志：true=单体（单jar），false=多模块
  modelValue: boolean
  libSeparate: boolean
  deployPath?: string
}>(), {
  deployPath: '',
})

const emit = defineEmits<{
  (e: 'update:modelValue', v: boolean): void
  (e: 'update:libSeparate', v: boolean): void
}>()

const modes = [
  { key: 'monolith', name: '单体部署', desc: '整体构建产出单个 jar 部署' },
  { key: 'multi', name: '多模块部署', desc: '每个模块独立构建并部署到独立远程目录' },
] as const

function isActive(key: 'monolith' | 'multi') {
  return key === 'monolith' ? props.modelValue === true : props.modelValue === false
}

function pick(key: 'monolith' | 'multi') {
  emit('update:modelValue', key === 'monolith')
}
</script>