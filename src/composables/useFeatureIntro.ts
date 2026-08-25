import { ref } from 'vue'
import { getIntroForPath, isIntroSeen, markIntroSeen } from '../features/featureIntro'
import type { FeatureIntro } from '../features/featureIntro'

// 模块级单例状态：全局只维护一份「当前展示的功能引导」
const currentIntro = ref<FeatureIntro | null>(null)
const visible = ref(false)

export function useFeatureIntro() {
  /** 打开引导弹窗（同时标记已看，保证首次进入只弹一次） */
  function showIntro(intro: FeatureIntro) {
    currentIntro.value = intro
    visible.value = true
    markIntroSeen(intro.path)
  }

  /** 路由进入时调用：匹配到引导配置且未看过 → 弹窗 */
  function maybeShowForPath(path: string) {
    const intro = getIntroForPath(path)
    if (intro && !isIntroSeen(intro.path)) {
      showIntro(intro)
    }
  }

  function closeIntro() {
    visible.value = false
  }

  return { currentIntro, visible, showIntro, maybeShowForPath, closeIntro }
}