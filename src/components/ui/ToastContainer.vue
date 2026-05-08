<template>
  <Teleport to="body">
    <div class="toast toast-top toast-end z-[9999] pointer-events-auto" v-if="toasts.length > 0">
      <TransitionGroup name="toast-list">
        <div
          v-for="toast in toasts"
          :key="toast.id"
          class="relative animate-fadeIn"
        >
          <Toast
            :message="toast.message"
            :type="toast.type"
            :duration="toast.duration"
            :progress="toast.progress"
            @close="removeToast(toast.id)"
          />
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { useToast } from '../../composables/useToast'
import Toast from './Toast.vue'

const { toasts, removeToast } = useToast()
</script>

<style scoped>
.toast-list-enter-active { animation: toastSlideIn 0.3s ease; }
.toast-list-leave-active { animation: toastSlideOut 0.3s ease; position: absolute; }
.toast-list-move { transition: transform 0.3s ease; }
@keyframes toastSlideIn { from { opacity: 0; transform: translateX(100%); } to { opacity: 1; transform: translateX(0); } }
@keyframes toastSlideOut { from { opacity: 1; transform: translateX(0); } to { opacity: 0; transform: translateX(100%); } }
</style>
