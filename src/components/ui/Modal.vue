<template>
  <Teleport to="body">
    <dialog ref="dialogRef" class="modal" :class="{ 'modal-open': modelValue }" @close="handleClose">
      <div class="modal-box w-full" :style="{ maxWidth: width, maxHeight: maxHeight }">
        <form method="dialog">
          <button v-if="showClose" class="btn btn-sm btn-circle btn-ghost absolute right-2 top-2 text-base-content/60 hover:text-base-content" @click="close"><SvgIcon name="x" size="14" /></button>
        </form>
        <h3 v-if="title" class="text-lg font-bold mb-2">{{ title }}</h3>
        <slot name="header" />
        <div class="py-2">
          <slot />
        </div>
        <div v-if="$slots.footer" class="modal-action">
          <slot name="footer" />
        </div>
      </div>
      <form method="dialog" class="modal-backdrop">
        <button @click="close">关闭</button>
      </form>
    </dialog>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'

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
})

const emit = defineEmits(['update:modelValue', 'close'])
const dialogRef = ref<HTMLDialogElement | null>(null)

watch(() => props.modelValue, (val) => {
  const el = dialogRef.value
  if (!el) return
  if (val) {
    el.showModal()
  } else {
    el.close()
  }
})

function handleClose() {
  emit('update:modelValue', false)
  emit('close')
}

function close() {
  const el = dialogRef.value
  if (el) el.close()
}
</script>
