<template>
  <div
    ref="containerRef"
    class="absolute inset-0 cursor-crosshair"
    @pointerdown="onPointerDown"
    @pointermove="onPointerMove"
    @pointerup="onPointerUp"
  >
    <canvas ref="canvasRef" class="absolute inset-0 w-full h-full" />
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, nextTick } from 'vue'

interface Props {
  imgNaturalWidth: number
  imgNaturalHeight: number
  imgDisplayWidth: number
  imgDisplayHeight: number
  imgOffsetX: number
  imgOffsetY: number
  cropX: number
  cropY: number
  cropW: number
  cropH: number
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'update:cropX', value: number): void
  (e: 'update:cropY', value: number): void
  (e: 'update:cropW', value: number): void
  (e: 'update:cropH', value: number): void
}>()

const containerRef = ref<HTMLDivElement>()
const canvasRef = ref<HTMLCanvasElement>()

// Interaction state
type HandleType = 'none' | 'create' | 'move' | 'nw' | 'n' | 'ne' | 'e' | 'se' | 's' | 'sw' | 'w'
const activeHandle = ref<HandleType>('none')
const dragStart = ref({ x: 0, y: 0 })
const dragStartRect = ref({ x: 0, y: 0, w: 0, h: 0 })
const isDragging = ref(false)

// Current crop rect in display coordinates (pixels on canvas)
const displayRect = ref({ x: 0, y: 0, w: 0, h: 0 })

const HANDLE_SIZE = 8
const MIN_SIZE = 4

// Convert between image natural coordinates and display coordinates
function toDisplayX(naturalX: number): number {
  return props.imgOffsetX + (naturalX / props.imgNaturalWidth) * props.imgDisplayWidth
}

function toDisplayY(naturalY: number): number {
  return props.imgOffsetY + (naturalY / props.imgNaturalHeight) * props.imgDisplayHeight
}

function toNaturalX(displayX: number): number {
  return Math.round(((displayX - props.imgOffsetX) / props.imgDisplayWidth) * props.imgNaturalWidth)
}

function toNaturalY(displayY: number): number {
  return Math.round(((displayY - props.imgOffsetY) / props.imgDisplayHeight) * props.imgNaturalHeight)
}

// Sync display rect from props (natural coords → display coords)
function syncFromProps() {
  if (props.cropW > 0 && props.cropH > 0) {
    displayRect.value = {
      x: toDisplayX(props.cropX),
      y: toDisplayY(props.cropY),
      w: (props.cropW / props.imgNaturalWidth) * props.imgDisplayWidth,
      h: (props.cropH / props.imgNaturalHeight) * props.imgDisplayHeight,
    }
  } else {
    displayRect.value = { x: 0, y: 0, w: 0, h: 0 }
  }
}

// Emit natural coords from display rect
function emitFromDisplay() {
  const r = displayRect.value
  if (r.w <= 0 || r.h <= 0) { return }
  const nx = toNaturalX(r.x)
  const ny = toNaturalY(r.y)
  const nw = Math.max(1, Math.round((r.w / props.imgDisplayWidth) * props.imgNaturalWidth))
  const nh = Math.max(1, Math.round((r.h / props.imgDisplayHeight) * props.imgNaturalHeight))
  emit('update:cropX', Math.max(0, nx))
  emit('update:cropY', Math.max(0, ny))
  emit('update:cropW', nw)
  emit('update:cropH', nh)
}

// Get handles positions in display coordinates
function getHandles() {
  const r = displayRect.value
  if (r.w <= 0 || r.h <= 0) { return [] }
  return [
    { id: 'nw', x: r.x, y: r.y },
    { id: 'n', x: r.x + r.w / 2, y: r.y },
    { id: 'ne', x: r.x + r.w, y: r.y },
    { id: 'e', x: r.x + r.w, y: r.y + r.h / 2 },
    { id: 'se', x: r.x + r.w, y: r.y + r.h },
    { id: 's', x: r.x + r.w / 2, y: r.y + r.h },
    { id: 'sw', x: r.x, y: r.y + r.h },
    { id: 'w', x: r.x, y: r.y + r.h / 2 },
  ]
}

// Hit test: which handle is under the pointer?
function hitTestHandle(px: number, py: number): HandleType {
  const handles = getHandles()
  for (const h of handles) {
    const dx = px - h.x
    const dy = py - h.y
    if (Math.abs(dx) <= HANDLE_SIZE && Math.abs(dy) <= HANDLE_SIZE) {
      return h.id as HandleType
    }
  }
  // Check if inside crop rect for move
  const r = displayRect.value
  if (r.w > 0 && r.h > 0 && px >= r.x && px <= r.x + r.w && py >= r.y && py <= r.y + r.h) {
    return 'move'
  }
  return 'none'
}

// Clamp display rect to image bounds
function clampRect(rect: { x: number; y: number; w: number; h: number }) {
  const minX = props.imgOffsetX
  const minY = props.imgOffsetY
  const maxX = props.imgOffsetX + props.imgDisplayWidth
  const maxY = props.imgOffsetY + props.imgDisplayHeight

  let { x, y, w, h } = rect

  // Clamp size
  w = Math.max(MIN_SIZE, Math.min(w, maxX - minX))
  h = Math.max(MIN_SIZE, Math.min(h, maxY - minY))

  // Clamp position
  x = Math.max(minX, Math.min(x, maxX - w))
  y = Math.max(minY, Math.min(y, maxY - h))

  return { x, y, w, h }
}

function getPointerPos(e: PointerEvent): { x: number; y: number } {
  if (!containerRef.value) { return { x: 0, y: 0 } }
  const rect = containerRef.value.getBoundingClientRect()
  return { x: e.clientX - rect.left, y: e.clientY - rect.top }
}

function onPointerDown(e: PointerEvent) {
  const pos = getPointerPos(e)
  const hit = hitTestHandle(pos.x, pos.y)

  activeHandle.value = hit
  dragStart.value = pos
  dragStartRect.value = { ...displayRect.value }

  if (hit === 'none') {
    // Start creating a new selection
    activeHandle.value = 'create'
    displayRect.value = { x: pos.x, y: pos.y, w: 0, h: 0 }
  }

  isDragging.value = true
  containerRef.value?.setPointerCapture(e.pointerId)
}

function onPointerMove(e: PointerEvent) {
  const pos = getPointerPos(e)

  // Update cursor based on hover
  if (!isDragging.value) {
    const hit = hitTestHandle(pos.x, pos.y)
    if (hit === 'none') {
      containerRef.value && (containerRef.value.style.cursor = 'crosshair')
    } else if (hit === 'move') {
      containerRef.value && (containerRef.value.style.cursor = 'move')
    } else {
      containerRef.value && (containerRef.value.style.cursor = getCursorForHandle(hit))
    }
    return
  }

  const dx = pos.x - dragStart.value.x
  const dy = pos.y - dragStart.value.y
  const sr = dragStartRect.value
  const handle = activeHandle.value

  if (handle === 'create') {
    let x = Math.min(dragStart.value.x, pos.x)
    let y = Math.min(dragStart.value.y, pos.y)
    let w = Math.abs(pos.x - dragStart.value.x)
    let h = Math.abs(pos.y - dragStart.value.y)
    displayRect.value = clampRect({ x, y, w, h })
  } else if (handle === 'move') {
    displayRect.value = clampRect({ x: sr.x + dx, y: sr.y + dy, w: sr.w, h: sr.h })
  } else {
    // Resize handles - clean implementation
    let newX = sr.x
    let newY = sr.y
    let newW = sr.w
    let newH = sr.h

    // Determine which edges to resize based on handle
    const resizeLeft = handle === 'nw' || handle === 'w' || handle === 'sw'
    const resizeRight = handle === 'ne' || handle === 'e' || handle === 'se'
    const resizeTop = handle === 'nw' || handle === 'n' || handle === 'ne'
    const resizeBottom = handle === 'sw' || handle === 's' || handle === 'se'

    if (resizeLeft) {
      newX = sr.x + dx
      newW = sr.w - dx
    } else if (resizeRight) {
      newW = sr.w + dx
    }

    if (resizeTop) {
      newY = sr.y + dy
      newH = sr.h - dy
    } else if (resizeBottom) {
      newH = sr.h + dy
    }

    // Normalize negative dimensions (when dragging past opposite edge)
    if (newW < 0) {
      newX = newX + newW
      newW = -newW
    }
    if (newH < 0) {
      newY = newY + newH
      newH = -newH
    }

    displayRect.value = clampRect({ x: newX, y: newY, w: newW, h: newH })
  }

  emitFromDisplay()
}

function onPointerUp(e: PointerEvent) {
  if (isDragging.value) {
    isDragging.value = false
    activeHandle.value = 'none'
    containerRef.value?.releasePointerCapture(e.pointerId)

    // If create resulted in too small rect, clear it
    if (displayRect.value.w < MIN_SIZE || displayRect.value.h < MIN_SIZE) {
      displayRect.value = { x: 0, y: 0, w: 0, h: 0 }
      emit('update:cropW', 0)
      emit('update:cropH', 0)
    }
  }
}

function getCursorForHandle(handle: HandleType): string {
  const map: Record<string, string> = {
    nw: 'nw-resize',
    n: 'n-resize',
    ne: 'ne-resize',
    e: 'e-resize',
    se: 'se-resize',
    s: 's-resize',
    sw: 'sw-resize',
    w: 'w-resize',
  }
  return map[handle] || 'crosshair'
}

// ============ Drawing ============

let animFrameId: number | null = null

function draw() {
  const canvas = canvasRef.value
  if (!canvas) { return }

  const ctx = canvas.getContext('2d')
  if (!ctx) { return }

  // Set canvas size to match container
  const container = containerRef.value
  if (!container) { return }

  const dpr = window.devicePixelRatio || 1
  const rect = container.getBoundingClientRect()

  if (canvas.width !== rect.width * dpr || canvas.height !== rect.height * dpr) {
    canvas.width = rect.width * dpr
    canvas.height = rect.height * dpr
    canvas.style.width = rect.width + 'px'
    canvas.style.height = rect.height + 'px'
  }

  ctx.clearRect(0, 0, canvas.width, canvas.height)
  ctx.save()
  ctx.scale(dpr, dpr)

  const r = displayRect.value

  if (r.w > 0 && r.h > 0) {
    // Draw dark overlay outside crop area
    ctx.fillStyle = 'rgba(0, 0, 0, 0.55)'
    // Top
    ctx.fillRect(0, 0, rect.width, r.y)
    // Bottom
    ctx.fillRect(0, r.y + r.h, rect.width, rect.height - r.y - r.h)
    // Left
    ctx.fillRect(0, r.y, r.x, r.h)
    // Right
    ctx.fillRect(r.x + r.w, r.y, rect.width - r.x - r.w, r.h)

    // Draw dashed border
    ctx.strokeStyle = '#ffffff'
    ctx.lineWidth = 1.5
    ctx.setLineDash([6, 4])
    ctx.strokeRect(r.x, r.y, r.w, r.h)
    ctx.setLineDash([])

    // Draw rule of thirds lines
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.25)'
    ctx.lineWidth = 0.5
    for (let i = 1; i <= 2; i++) {
      ctx.beginPath()
      ctx.moveTo(r.x + (r.w * i) / 3, r.y)
      ctx.lineTo(r.x + (r.w * i) / 3, r.y + r.h)
      ctx.stroke()
      ctx.beginPath()
      ctx.moveTo(r.x, r.y + (r.h * i) / 3)
      ctx.lineTo(r.x + r.w, r.y + (r.h * i) / 3)
      ctx.stroke()
    }

    // Draw dimension label
    const naturalW = Math.round((r.w / props.imgDisplayWidth) * props.imgNaturalWidth)
    const naturalH = Math.round((r.h / props.imgDisplayHeight) * props.imgNaturalHeight)
    const label = `${naturalW} × ${naturalH}`
    ctx.font = '11px sans-serif'
    ctx.textAlign = 'center'
    ctx.textBaseline = 'bottom'
    const textWidth = ctx.measureText(label).width
    const labelX = r.x + r.w / 2
    const labelY = r.y - 6

    // Label background
    ctx.fillStyle = 'rgba(0, 0, 0, 0.6)'
    const pad = 4
    const lx = labelX - textWidth / 2 - pad
    const ly = labelY - 14 - pad
    const lw = textWidth + pad * 2
    const lh = 14 + pad * 2
    // Use regular rect instead of roundRect for compatibility
    ctx.beginPath()
    ctx.moveTo(lx + 3, ly)
    ctx.lineTo(lx + lw - 3, ly)
    ctx.quadraticCurveTo(lx + lw, ly, lx + lw, ly + 3)
    ctx.lineTo(lx + lw, ly + lh - 3)
    ctx.quadraticCurveTo(lx + lw, ly + lh, lx + lw - 3, ly + lh)
    ctx.lineTo(lx + 3, ly + lh)
    ctx.quadraticCurveTo(lx, ly + lh, lx, ly + lh - 3)
    ctx.lineTo(lx, ly + 3)
    ctx.quadraticCurveTo(lx, ly, lx + 3, ly)
    ctx.closePath()
    ctx.fill()

    ctx.fillStyle = '#ffffff'
    ctx.fillText(label, labelX, labelY - 2)

    // Draw 8 control points
    const handles = getHandles()
    for (const h of handles) {
      ctx.fillStyle = '#ffffff'
      ctx.strokeStyle = '#333333'
      ctx.lineWidth = 1.5
      ctx.beginPath()
      ctx.arc(h.x, h.y, HANDLE_SIZE / 2 + 1, 0, Math.PI * 2)
      ctx.fill()
      ctx.stroke()
    }
  }

  ctx.restore()
  animFrameId = requestAnimationFrame(draw)
}

// Watch for prop changes to sync display rect
watch(
  () => [props.cropX, props.cropY, props.cropW, props.cropH, props.imgDisplayWidth, props.imgDisplayHeight],
  () => {
    if (!isDragging.value) {
      syncFromProps()
    }
  },
  { deep: true }
)

onMounted(() => {
  nextTick(() => {
    syncFromProps()
    animFrameId = requestAnimationFrame(draw)
  })
})
</script>
