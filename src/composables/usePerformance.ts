import { ref, reactive, onMounted, onBeforeUnmount } from 'vue';
import type { Ref } from 'vue';
import type { PerformanceMetrics } from '../types';

interface RenderWarning {
  type: string;
  component: string;
  duration: string;
  timestamp: number;
}

interface RenderTimeStats {
  avg: number;
  max: number;
  count: number;
}

/**
 * 性能监控 Composable
 * 提供组件渲染时间、内存使用、操作耗时等性能指标
 */
export function usePerformance() {
  const metrics = reactive<PerformanceMetrics>({
    componentRenders: new Map(),
    operationTimings: [],
    fps: 0,
    memoryUsage: null,
    virtualListEnabled: false,
  });

  const renderTimes: Ref<Record<string, RenderTimeStats>> = ref<any>({});
  const warnings: Ref<RenderWarning[]> = ref<any[]>([]);
  const fpsHistory: Ref<Array<{ time: number; fps: number }>> = ref<any[]>([]);

  // FPS 监控
  let frameCount = 0;
  let lastFpsTime = performance.now();
  let animationFrameId: number | null = null;

  function startFpsMonitoring(): void {
    function measureFrame(): void {
      frameCount++;
      const now = performance.now();
      if (now - lastFpsTime >= 1000) {
        metrics.fps = Math.round((frameCount * 1000) / (now - lastFpsTime));
        fpsHistory.value.push({
          time: Date.now(),
          fps: metrics.fps,
        });
        // 保留最近 60 秒的数据
        if (fpsHistory.value.length > 60) {
          fpsHistory.value = fpsHistory.value.slice(-60);
        }
        frameCount = 0;
        lastFpsTime = now;
      }
      animationFrameId = requestAnimationFrame(measureFrame);
    }
    animationFrameId = requestAnimationFrame(measureFrame);
  }

  function stopFpsMonitoring(): void {
    if (animationFrameId) {
      cancelAnimationFrame(animationFrameId);
      animationFrameId = null;
    }
  }

  // 组件渲染计时
  function measureComponentRender<T>(componentName: string, fn: () => T): T {
    const start = performance.now();
    const result = fn();
    const duration = performance.now() - start;

    const prev = metrics.componentRenders.get(componentName) || { count: 0, total: 0, max: 0 };
    prev.count++;
    prev.total += duration;
    prev.max = Math.max(prev.max, duration);
    metrics.componentRenders.set(componentName, prev);

    renderTimes.value[componentName] = {
      avg: prev.total / prev.count,
      max: prev.max,
      count: prev.count,
    };

    if (duration > 16) {
      warnings.value.push({
        type: 'slow-render',
        component: componentName,
        duration: duration.toFixed(2),
        timestamp: Date.now(),
      });
      if (warnings.value.length > 50) {
        warnings.value = warnings.value.slice(-50);
      }
    }

    return result;
  }

  // 操作计时
  function measureOperation<T>(name: string, fn: () => T | Promise<T>): T | Promise<T> {
    const start = performance.now();
    const result = fn();

    // 支持 async 函数
    if (result instanceof Promise) {
      return result.then((resolved: T) => {
        const duration = performance.now() - start;
        metrics.operationTimings.push({ name, duration, timestamp: Date.now() });
        if (metrics.operationTimings.length > 100) {
          metrics.operationTimings = metrics.operationTimings.slice(-100);
        }
        return resolved;
      });
    }

    const duration = performance.now() - start;
    metrics.operationTimings.push({ name, duration, timestamp: Date.now() });
    if (metrics.operationTimings.length > 100) {
      metrics.operationTimings = metrics.operationTimings.slice(-100);
    }
    return result;
  }

  // 获取内存使用
  function updateMemoryUsage(): void {
    if ((performance as Performance & { memory?: { usedJSHeapSize: number; totalJSHeapSize: number; jsHeapSizeLimit: number } }).memory) {
      const mem = (performance as Performance & { memory: { usedJSHeapSize: number; totalJSHeapSize: number; jsHeapSizeLimit: number } }).memory;
      metrics.memoryUsage = {
        usedJSHeapSize: mem.usedJSHeapSize,
        totalJSHeapSize: mem.totalJSHeapSize,
        jsHeapSizeLimit: mem.jsHeapSizeLimit,
      };
    }
  }

  // 虚拟滚动状态
  function setVirtualListEnabled(enabled: boolean): void {
    metrics.virtualListEnabled = enabled;
  }

  // 获取性能报告
  function getReport(): Record<string, unknown> {
    const report: Record<string, unknown> = {
      fps: metrics.fps,
      virtualListEnabled: metrics.virtualListEnabled,
      componentRenders: {} as Record<string, unknown>,
      recentOperations: metrics.operationTimings.slice(-10),
      warnings: warnings.value.slice(-10),
      memoryUsage: metrics.memoryUsage,
    };

    for (const [name, data] of metrics.componentRenders) {
      (report.componentRenders as Record<string, unknown>)[name] = {
        avgMs: (data.total / data.count).toFixed(2),
        maxMs: data.max.toFixed(2),
        renderCount: data.count,
      };
    }

    return report;
  }

  // 清除数据
  function reset(): void {
    metrics.componentRenders.clear();
    metrics.operationTimings = [];
    warnings.value = [];
    fpsHistory.value = [];
    renderTimes.value = {};
  }

  onMounted(() => {
    startFpsMonitoring();
    // 每 5 秒更新一次内存使用
    const memInterval = setInterval(updateMemoryUsage, 5000);
    onBeforeUnmount(() => {
      stopFpsMonitoring();
      clearInterval(memInterval);
    });
  });

  return {
    metrics,
    renderTimes,
    warnings,
    fpsHistory,
    measureComponentRender,
    measureOperation,
    updateMemoryUsage,
    setVirtualListEnabled,
    getReport,
    reset,
  };
}

export default usePerformance;
