<template>
  <Modal
    :model-value="visible"
    :title="intro?.title || ''"
    width="620px"
    :show-close="false"
    class="feature-intro-modal"
    @close="closeOnNativeClose"
  >
    <!-- 功能介绍 -->
    <div v-if="intro" class="flex flex-col gap-4 py-1">
      <div class="flex gap-2.5 items-start">
        <SvgIcon name="lightbulb" :size="16" class="shrink-0 mt-0.5 text-primary" />
        <div class="min-w-0">
          <div class="text-xs font-semibold text-base-content/60 uppercase tracking-wider mb-1">功能介绍</div>
          <p class="m-0 text-sm leading-relaxed text-base-content/85">{{ intro.intro }}</p>
        </div>
      </div>

      <div class="flex gap-2.5 items-start">
        <SvgIcon name="list" :size="16" class="shrink-0 mt-0.5 text-primary" />
        <div class="min-w-0">
          <div class="text-xs font-semibold text-base-content/60 uppercase tracking-wider mb-1">使用方法</div>
          <p class="m-0 text-sm leading-relaxed text-base-content/85 whitespace-pre-line">{{ intro.howto }}</p>
        </div>
      </div>

      <div v-if="intro.prereqs.length" class="flex gap-2.5 items-start">
        <SvgIcon name="alertTriangle" :size="16" class="shrink-0 mt-0.5 text-amber-500" />
        <div class="min-w-0">
          <div class="text-xs font-semibold text-base-content/60 uppercase tracking-wider mb-1.5">前置条件</div>
          <div class="flex flex-col gap-1.5">
            <div
              v-for="(p, i) in intro.prereqs"
              :key="i"
              class="flex items-center gap-2 text-sm text-base-content/85"
            >
              <span class="w-1.5 h-1.5 rounded-full bg-amber-500 shrink-0" />
              <span class="flex-1">{{ p.label }}</span>
              <button
                v-if="p.link"
                class="btn btn-xs btn-primary btn-outline gap-1 whitespace-nowrap"
                @click="go(p)"
              >
                {{ p.linkLabel || '去添加' }}
                <SvgIcon name="arrowRight" :size="12" />
              </button>
            </div>
          </div>
        </div>
      </div>

      <div v-else class="flex gap-2.5 items-start">
        <SvgIcon name="check" :size="16" class="shrink-0 mt-0.5 text-success" />
        <div class="text-xs font-semibold text-base-content/60 uppercase tracking-wider">无前置依赖，可直接使用</div>
      </div>
    </div>

    <template #footer>
      <button class="btn btn-ghost btn-sm" @click="close">跳过</button>
      <button class="btn btn-primary btn-sm" @click="close">知道了，开始使用</button>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import Modal from '@/components/ui/Modal.vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { useFeatureIntro } from '@/composables/useFeatureIntro'
import type { FeaturePrereq } from '@/features/featureIntro'

defineProps<{
  /** 当前要展示的引导；null 表示无可展示内容 */
  intro: import('@/features/featureIntro').FeatureIntro | null
  visible: boolean
}>()

const router = useRouter()
const { closeIntro } = useFeatureIntro()

function close() {
  closeIntro()
}

/** Esc / 点击遮罩等原生关闭时，同步复位单例状态，避免下次弹不出来 */
function closeOnNativeClose() {
  close()
}

function go(p: FeaturePrereq) {
  if (p.link) {
    close()
    router.push(p.link)
  }
}
</script>