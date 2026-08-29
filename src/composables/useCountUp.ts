import { ref, watch, type Ref } from 'vue'

/**
 * 数字滚动：目标值变化时用 rAF 平滑过渡（首屏从 0 递增）。
 * 系统「减弱动态效果」下直接跳到目标值，不做动画。
 *
 * @param source 目标数值
 * @param duration 动画时长 ms，默认 420
 */
export function useCountUp(source: Ref<number>, duration = 420) {
  const display = ref(0);
  let raf = 0;
  const reduced = typeof window !== 'undefined'
    && window.matchMedia?.('(prefers-reduced-motion: reduce)').matches;

  watch(source, (to, from) => {
    cancelAnimationFrame(raf);
    const target = Number.isFinite(to) ? to : 0;
    if (reduced) {
      display.value = target;
      return;
    }
    const start = Number.isFinite(from as number) ? (from as number) : 0;
    const delta = target - start;
    if (delta === 0) {
      display.value = target;
      return;
    }
    const t0 = performance.now();
    const step = (now: number) => {
      const p = Math.min(1, (now - t0) / duration);
      const eased = 1 - Math.pow(1 - p, 3); // easeOutCubic
      display.value = Math.round(start + delta * eased);
      if (p < 1) {raf = requestAnimationFrame(step);}
    };
    raf = requestAnimationFrame(step);
  }, { immediate: true });

  return display;
}
