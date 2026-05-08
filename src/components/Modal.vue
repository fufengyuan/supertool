<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="modelValue" class="modal-overlay" @click="handleOverlayClick">
        <div class="modal-content" :class="contentClass" @click.stop :style="contentStyle">
          <div class="modal-header" v-if="$slots.header || title || showClose">
            <slot name="header">
              <h3>{{ title }}</h3>
            </slot>
            <button v-if="showClose" class="modal-close-btn" @click="close" title="关闭">×</button>
          </div>
          <div class="modal-body">
            <slot />
          </div>
          <div class="modal-footer" v-if="$slots.footer">
            <slot name="footer" />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
const props = defineProps({
  modelValue: {
    type: Boolean,
    default: false,
  },
  title: {
    type: String,
    default: '',
  },
  showClose: {
    type: Boolean,
    default: true,
  },
  width: {
    type: String,
    default: '560px',
  },
  maxHeight: {
    type: String,
    default: '85vh',
  },
});

const emit = defineEmits(['update:modelValue', 'close']);

const handleOverlayClick = () => {
  close();
};

const close = () => {
  emit('update:modelValue', false);
  emit('close');
};

const contentClass = '';
const contentStyle = {
  maxWidth: props.width,
  maxHeight: props.maxHeight,
};
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background: oklch(var(--b1));
  border-radius: 16px;
  width: 90%;
  max-height: v-bind(maxHeight);
  overflow-y: auto;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
}

.modal-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: oklch(var(--bc));
}

.modal-close-btn {
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: oklch(var(--bc) / 0.6);
  font-size: 20px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}

.modal-close-btn:hover {
  background: oklch(var(--b2));
  color: oklch(var(--bc));
}

.modal-body {
  padding: 24px;
}

.modal-footer {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  padding: 16px 24px;
  border-top: 1px solid oklch(var(--bc) / 0.1);
}

/* Transitions */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}

.modal-enter-active .modal-content,
.modal-leave-active .modal-content {
  transition:
    transform 0.3s ease,
    opacity 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .modal-content {
  opacity: 0;
  transform: translateY(20px);
}

.modal-leave-to .modal-content {
  opacity: 0;
  transform: translateY(10px);
}
</style>
