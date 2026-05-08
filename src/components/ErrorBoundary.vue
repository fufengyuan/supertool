<template>
  <div class="error-boundary">
    <template v-if="hasError">
      <div class="error-fallback">
        <div class="error-icon">
          <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10"/>
            <line x1="15" y1="9" x2="9" y2="15"/>
            <line x1="9" y1="9" x2="15" y2="15"/>
          </svg>
        </div>
        <h2 class="error-title">出错了</h2>
        <p class="error-message">{{ errorMessage }}</p>
        <button class="retry-btn" @click="retry">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="23 4 23 10 17 10"/>
            <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
          </svg>
          重试
        </button>

        <details class="error-details">
          <summary class="error-details-summary">
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="6 9 12 15 18 9"/>
            </svg>
            错误详情
          </summary>
          <pre class="error-stack">{{ errorStack }}</pre>
        </details>
      </div>
    </template>
    <slot v-else />
  </div>
</template>

<script setup lang="ts">
import { ref, onErrorCaptured, provide } from 'vue'
import { useErrorHandler } from '../composables/useErrorHandler'

const props = defineProps({
  // 自定义重试函数
  onRetry: {
    type: Function,
    default: null,
  },
})

const emit = defineEmits(['error'])

const hasError = ref(false)
const errorMessage = ref('')
const errorStack = ref('')

const { handleError } = useErrorHandler()

onErrorCaptured((err, instance, info) => {
  hasError.value = true
  errorMessage.value = err.message || '组件渲染时发生未知错误'
  errorStack.value = err.stack || String(err)

  handleError(err, {
    context: `ErrorBoundary:${info}`,
    showToast: false, // 由 ErrorBoundary 自己控制 toast 显示
  })

  emit('error', { error: err, instance, info })
  return false // 阻止错误继续传播
})

function retry() {
  hasError.value = false
  errorMessage.value = ''
  errorStack.value = ''

  if (props.onRetry) {
    props.onRetry()
  } else {
    // 默认重试: 刷新当前视图
    window.location.reload()
  }
}
</script>

<style scoped>
.error-boundary {
  width: 100%;
}

.error-fallback {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 24px;
  text-align: center;
  min-height: 300px;
}

.error-icon {
  color: #ef4444;
  margin-bottom: 16px;
  opacity: 0.8;
}

.error-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--color-base-content);
  margin: 0 0 8px;
}

.error-message {
  font-size: 14px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  margin: 0 0 24px;
  max-width: 400px;
}

.retry-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 24px;
  border: none;
  border-radius: 8px;
  background: var(--color-primary);
  color: #fff;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s ease;
}

.retry-btn:hover {
  background: color-mix(in oklab, var(--color-primary) 80%, transparent);
}

.error-details {
  margin-top: 24px;
  width: 100%;
  max-width: 600px;
  text-align: left;
}

.error-details-summary {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  cursor: pointer;
  padding: 8px 12px;
  border-radius: 6px;
  background: rgba(0, 0, 0, 0.03);
  transition: background 0.15s ease;
}

.error-details-summary:hover {
  background: rgba(0, 0, 0, 0.06);
}

.error-stack {
  margin: 8px 0 0;
  padding: 16px;
  background: rgba(0, 0, 0, 0.04);
  border-radius: 6px;
  font-size: 12px;
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  color: #ef4444;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 200px;
  overflow-y: auto;
}

/* 暗色模式适配 */
:deep(.dark) .error-title {
  color: var(--color-base-content);
}

:deep(.dark) .error-message {
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

:deep(.dark) .error-details-summary {
  background: rgba(255, 255, 255, 0.05);
}

:deep(.dark) .error-details-summary:hover {
  background: rgba(255, 255, 255, 0.08);
}

:deep(.dark) .error-stack {
  background: rgba(255, 255, 255, 0.05);
}
</style>
