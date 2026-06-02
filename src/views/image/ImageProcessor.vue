<template>
  <div class="flex h-full overflow-hidden bg-base-200" style="font-size: 11px;">
    <!-- Left Panel: Controls -->
    <aside class="w-[320px] min-w-[280px] max-w-[320px] border-r border-base-300 bg-base-100 flex flex-col shrink-0 overflow-y-auto">
      <!-- Header -->
      <div class="flex items-center gap-1.5 px-3 pt-3 pb-2 border-b border-base-300">
        <SvgIcon name="image" size="14" class="text-base-content/70" />
        <span class="text-xs font-bold text-base-content">图像处理</span>
      </div>

      <!-- File Selection -->
      <div class="p-3 border-b border-base-300">
        <div
          class="border-2 border-dashed rounded-lg p-3 text-center cursor-pointer transition-all duration-150"
          :class="dragOver ? 'border-primary bg-primary/8 scale-[1.01]' : 'border-base-300 hover:border-primary/40 hover:bg-base-200/50'"
          @dragover.prevent="dragOver = true"
          @dragleave="dragOver = false"
          @drop.prevent="onDrop"
          @click="selectFile"
        >
          <SvgIcon name="upload" size="18" class="text-base-content/30 mb-1" />
          <p class="text-xs text-base-content/50">拖拽图片到此处</p>
          <p class="text-[10px] text-base-content/30 mt-0.5">或点击选择文件</p>
        </div>
        <div v-if="originalPath" class="mt-2 flex items-center gap-1.5 text-[10px] text-base-content/60">
          <SvgIcon name="file" size="10" class="shrink-0" />
          <span class="truncate flex-1">{{ fileName }}</span>
          <button class="btn btn-ghost btn-xs px-1 text-base-content/40 hover:text-error" @click="clearFile" title="清除">
            <SvgIcon name="x" size="10" />
          </button>
        </div>
      </div>

      <!-- Function Selection -->
      <div class="px-3 pt-2.5 pb-1.5">
        <div class="text-[10px] font-semibold text-base-content/50 uppercase tracking-wider mb-1.5">功能选择</div>
        <div class="grid grid-cols-2 gap-1">
          <button
            v-for="fn in functions"
            :key="fn.id"
            class="flex items-center gap-1.5 px-2 py-1.5 rounded-md text-[11px] font-medium transition-colors"
            :class="activeFunction === fn.id ? 'bg-primary text-primary-content shadow-sm' : 'bg-base-200 text-base-content/60 hover:bg-base-300 hover:text-base-content'"
            @click="activeFunction = fn.id"
          >
            <SvgIcon :name="fn.icon" :size="12" class="shrink-0" />
            <span class="truncate">{{ fn.label }}</span>
          </button>
        </div>
      </div>

      <!-- Parameters -->
      <div class="flex-1 px-3 pb-3 space-y-3">
        <div v-if="activeFunction === 'compress'" class="space-y-2.5">
          <div>
            <div class="flex items-center justify-between mb-1">
              <span class="text-[10px] font-medium text-base-content/70">质量</span>
              <span class="text-[10px] font-mono text-base-content/50">{{ quality }}%</span>
            </div>
            <input type="range" min="1" max="100" v-model.number="quality" class="range range-primary range-xs w-full" />
          </div>
          <div>
            <label class="text-[10px] font-medium text-base-content/70 block mb-1">输出格式</label>
            <select v-model="format" class="select select-bordered select-xs w-full text-[11px]">
              <option value="jpeg">JPEG</option>
              <option value="png">PNG</option>
              <option value="webp">WebP</option>
            </select>
          </div>
        </div>

        <div v-if="activeFunction === 'resize'" class="space-y-2.5">
          <div v-if="originalWidth && originalHeight" class="text-[10px] text-base-content/40 flex items-center gap-1">
            <SvgIcon name="maximize" size="9" />
            原始尺寸: {{ originalWidth }} × {{ originalHeight }}
          </div>
          <div class="grid grid-cols-2 gap-2">
            <div>
              <label class="text-[10px] font-medium block mb-1" :class="resizeSource === 'percent' ? 'text-base-content/40' : 'text-base-content/70'">宽度 (px)</label>
              <input type="number" :value="resizeWidth" @input="onResizeWidthInput($event)" class="input input-bordered input-xs w-full text-[11px] bg-transparent" min="1" placeholder="自动" />
            </div>
            <div>
              <label class="text-[10px] font-medium block mb-1" :class="resizeSource === 'percent' ? 'text-base-content/40' : 'text-base-content/70'">高度 (px)</label>
              <input type="number" :value="resizeHeight" @input="onResizeHeightInput($event)" class="input input-bordered input-xs w-full text-[11px] bg-transparent" min="1" placeholder="自动" />
            </div>
          </div>
          <div>
            <div class="flex items-center justify-between mb-1">
              <span class="text-[10px] font-medium" :class="resizeSource === 'dimensions' ? 'text-base-content/40' : 'text-base-content/70'">缩放比例</span>
              <span class="text-[10px] font-mono text-base-content/50">{{ computedPercent }}%</span>
            </div>
            <input type="range" min="1" max="200" :value="computedPercent" @input="onPercentInput($event)" class="range range-primary range-xs w-full" :class="{ 'opacity-50': resizeSource === 'dimensions' }" />
          </div>
          <label class="flex items-center gap-2 cursor-pointer py-0.5">
            <input type="checkbox" v-model="keepAspect" class="checkbox checkbox-xs checkbox-primary" />
            <span class="text-[10px] text-base-content/70">保持宽高比</span>
          </label>
        </div>

        <div v-if="activeFunction === 'convert'" class="space-y-2.5">
          <div>
            <label class="text-[10px] font-medium text-base-content/70 block mb-1">目标格式</label>
            <select v-model="targetFormat" class="select select-bordered select-xs w-full text-[11px]">
              <option value="jpeg">JPEG</option>
              <option value="png">PNG</option>
              <option value="webp">WebP</option>
              <option value="gif">GIF</option>
              <option value="bmp">BMP</option>
            </select>
          </div>
        </div>

        <div v-if="activeFunction === 'crop'" class="space-y-2.5">
          <div class="grid grid-cols-2 gap-2">
            <div>
              <label class="text-[10px] font-medium text-base-content/70 block mb-1">X</label>
              <input type="number" v-model.number="cropX" class="input input-bordered input-xs w-full text-[11px]" min="0" placeholder="0" />
            </div>
            <div>
              <label class="text-[10px] font-medium text-base-content/70 block mb-1">Y</label>
              <input type="number" v-model.number="cropY" class="input input-bordered input-xs w-full text-[11px]" min="0" placeholder="0" />
            </div>
            <div>
              <label class="text-[10px] font-medium text-base-content/70 block mb-1">宽度</label>
              <input type="number" v-model.number="cropW" class="input input-bordered input-xs w-full text-[11px]" min="1" placeholder="宽度" />
            </div>
            <div>
              <label class="text-[10px] font-medium text-base-content/70 block mb-1">高度</label>
              <input type="number" v-model.number="cropH" class="input input-bordered input-xs w-full text-[11px]" min="1" placeholder="高度" />
            </div>
          </div>
        </div>

        <div v-if="activeFunction === 'removeBg'" class="space-y-2.5">
          <div class="bg-base-200 rounded-lg p-3 text-center">
            <SvgIcon name="sparkles" size="18" class="text-base-content/30 mb-1" />
            <p class="text-[10px] text-base-content/60">使用 AI 自动识别并移除图片背景</p>
            <p class="text-[10px] text-base-content/40 mt-0.5">支持人物、物品等常见主体</p>
          </div>
        </div>
      </div>

      <!-- Execute Button -->
      <div class="px-3 pb-3 pt-1 border-t border-base-300">
        <button
          class="btn btn-primary btn-sm w-full gap-1.5 text-[11px]"
          :disabled="!originalPath || processing"
          @click="processImage"
        >
          <span v-if="processing" class="loading loading-spinner loading-xs"></span>
          <SvgIcon v-else name="zap" size="12" />
          {{ processing ? '处理中...' : (processedPath ? '重新处理' : '执行') }}
        </button>
        <p v-if="errorMsg" class="text-[10px] text-error mt-1 flex items-center gap-1">
          <SvgIcon name="alertTriangle" size="10" />
          {{ errorMsg }}
        </p>
      </div>
    </aside>

    <!-- Right Panel: Preview -->
    <main class="flex-1 flex flex-col overflow-hidden bg-base-200">
      <!-- Preview Header -->
      <div class="flex items-center justify-between px-4 py-2 bg-base-100 border-b border-base-300 shrink-0">
        <div class="flex items-center gap-2">
          <SvgIcon name="eye" size="12" class="text-base-content/50" />
          <span class="text-[11px] font-medium text-base-content/70">预览</span>
          <span v-if="originalPath" class="text-[10px] text-base-content/40">| {{ fileName }}</span>
        </div>
        <div class="flex items-center gap-1.5">
          <button
            v-if="originalPath && processedPath"
            class="btn btn-ghost btn-xs text-[10px] gap-1"
            :class="{ 'text-primary': viewMode === 'split' }"
            @click="viewMode = 'split'"
            title="对比模式"
          >
            <SvgIcon name="layers" size="12" />
            对比
          </button>
          <button
            v-if="originalPath && processedPath"
            class="btn btn-ghost btn-xs text-[10px] gap-1"
            :class="{ 'text-primary': viewMode === 'result' }"
            @click="viewMode = 'result'"
            title="仅结果"
          >
            <SvgIcon name="checkCircle" size="12" />
            结果
          </button>
          <button
            v-if="processedPath"
            class="btn btn-primary btn-xs text-[10px] gap-1"
            @click="downloadImage"
            title="下载"
          >
            <SvgIcon name="download" size="12" />
            下载
          </button>
        </div>
      </div>

      <!-- Preview Content -->
      <div class="flex-1 flex overflow-hidden">
        <!-- No file selected -->
        <div v-if="!originalPath" class="flex flex-col items-center justify-center w-full h-full text-base-content/30 gap-2">
          <SvgIcon name="image" size="32" class="opacity-20" />
          <span class="text-xs">选择图片开始处理</span>
        </div>

        <!-- Split view (original + processed) -->
        <div v-else-if="viewMode === 'split' && processedPath" class="flex w-full h-full">
          <div class="flex-1 flex flex-col border-r border-base-300">
            <div class="text-[10px] text-base-content/40 px-3 py-1 bg-base-100 border-b border-base-300 shrink-0">原始图片</div>
            <div ref="imgContainerRef2" class="flex-1 flex items-center justify-center p-3 overflow-hidden relative">
              <img
                ref="imgElementRef"
                :src="originalUrl"
                class="max-w-full max-h-full object-contain rounded shadow-sm"
                alt="原始图片"
              />
              <CropOverlay
                v-if="activeFunction === 'crop' && imgDisplayWidth > 0 && imgDisplayHeight > 0"
                :img-natural-width="originalWidth"
                :img-natural-height="originalHeight"
                :img-display-width="imgDisplayWidth"
                :img-display-height="imgDisplayHeight"
                :img-offset-x="imgOffsetX"
                :img-offset-y="imgOffsetY"
                :crop-x="cropX"
                :crop-y="cropY"
                :crop-w="cropW"
                :crop-h="cropH"
                @update:crop-x="(v: number) => cropX = v"
                @update:crop-y="(v: number) => cropY = v"
                @update:crop-w="(v: number) => cropW = v"
                @update:crop-h="(v: number) => cropH = v"
              />
            </div>
          </div>
          <div class="flex-1 flex flex-col">
            <div class="text-[10px] text-base-content/40 px-3 py-1 bg-base-100 border-b border-base-300 shrink-0">处理后图片</div>
            <div class="flex-1 flex items-center justify-center p-3 overflow-hidden">
              <img :src="processedUrl" class="max-w-full max-h-full object-contain rounded shadow-sm" alt="处理后图片" />
            </div>
          </div>
        </div>

        <!-- Single view (original or result) -->
        <div v-else class="flex w-full h-full">
          <div class="flex-1 flex flex-col relative">
            <div v-if="viewMode === 'result' && processedPath" class="text-[10px] text-base-content/40 px-3 py-1 bg-base-100 border-b border-base-300 shrink-0">处理后图片</div>
            <div v-else class="text-[10px] text-base-content/40 px-3 py-1 bg-base-100 border-b border-base-300 shrink-0">原始图片</div>
            <div ref="imgContainerRef" class="flex-1 flex items-center justify-center p-4 overflow-hidden relative">
              <img
                ref="imgElementRef"
                :src="viewMode === 'result' && processedPath ? processedUrl : originalUrl"
                class="max-w-full max-h-full object-contain rounded shadow-sm"
                :alt="viewMode === 'result' && processedPath ? '处理后图片' : '原始图片'"
              />
              <CropOverlay
                v-if="activeFunction === 'crop' && imgDisplayWidth > 0 && imgDisplayHeight > 0"
                :img-natural-width="originalWidth"
                :img-natural-height="originalHeight"
                :img-display-width="imgDisplayWidth"
                :img-display-height="imgDisplayHeight"
                :img-offset-x="imgOffsetX"
                :img-offset-y="imgOffsetY"
                :crop-x="cropX"
                :crop-y="cropY"
                :crop-w="cropW"
                :crop-h="cropH"
                @update:crop-x="(v: number) => cropX = v"
                @update:crop-y="(v: number) => cropY = v"
                @update:crop-w="(v: number) => cropW = v"
                @update:crop-h="(v: number) => cropH = v"
              />
            </div>
          </div>
        </div>
      </div>

      <!-- Status Bar -->
      <div v-if="originalPath || processedPath" class="flex items-center gap-3 px-4 py-1.5 bg-base-100 border-t border-base-300 shrink-0 text-[10px] text-base-content/50">
        <span v-if="originalWidth && originalHeight" class="flex items-center gap-1">
          <SvgIcon name="maximize" size="9" />
          {{ originalWidth }}×{{ originalHeight }}
        </span>
        <span v-if="originalPath && originalSize" class="flex items-center gap-1">
          <SvgIcon name="file" size="9" />
          原始: {{ formatSize(originalSize) }}
        </span>
        <span v-if="processedPath && processedSize" class="flex items-center gap-1">
          <SvgIcon name="file" size="9" />
          处理后: {{ formatSize(processedSize) }}
        </span>
        <span v-if="processedPath && originalSize && processedSize && originalSize > 0" class="text-primary">
          压缩率: {{ ((1 - processedSize / originalSize) * 100).toFixed(1) }}%
        </span>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'ImageProcessor' })

import SvgIcon from '@/components/ui/SvgIcon.vue'
import CropOverlay from '@/components/CropOverlay.vue'
import { ref, computed, onUnmounted, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { convertFileSrc } from '@tauri-apps/api/core'
import { copyFile } from '@tauri-apps/plugin-fs'

// ============ Cleanup ============

onUnmounted(() => {
  invoke('cleanTempDir', { maxAgeHours: 24 }).catch(() => {})
})

// ============ State ============

const originalPath = ref('')
const processedPath = ref('')
const originalSize = ref(0)
const processedSize = ref(0)
const originalWidth = ref(0)
const originalHeight = ref(0)
const processing = ref(false)
const errorMsg = ref('')
const dragOver = ref(false)
const viewMode = ref<'split' | 'result'>('split')

// Image container for crop overlay measurement
const imgContainerRef = ref<HTMLDivElement>()
const imgContainerRef2 = ref<HTMLDivElement>()
const imgElementRef = ref<HTMLImageElement>()
const imgDisplayWidth = ref(0)
const imgDisplayHeight = ref(0)
const imgOffsetX = ref(0)
const imgOffsetY = ref(0)

const activeFunction = ref('compress')

// Compress params
const quality = ref(80)
const format = ref('jpeg')

// Resize params
const resizeWidth = ref(0)
const resizeHeight = ref(0)
const percent = ref(100)
const keepAspect = ref(true)

// Resize linkage
const resizeSource = ref<'dimensions' | 'percent'>('dimensions')
const updatingResize = ref(false)

function onResizeWidthInput(e: Event) {
  if (updatingResize.value) return
  const val = Number((e.target as HTMLInputElement).value)
  if (isNaN(val) || val < 0) return
  updatingResize.value = true
  resizeSource.value = 'dimensions'
  resizeWidth.value = val
  if (val > 0 && originalWidth.value > 0) {
    percent.value = Math.round((val / originalWidth.value) * 100)
    if (keepAspect.value && originalHeight.value > 0) {
      resizeHeight.value = Math.round(originalHeight.value * val / originalWidth.value)
    }
  }
  updatingResize.value = false
}

function onResizeHeightInput(e: Event) {
  if (updatingResize.value) return
  const val = Number((e.target as HTMLInputElement).value)
  if (isNaN(val) || val < 0) return
  updatingResize.value = true
  resizeSource.value = 'dimensions'
  resizeHeight.value = val
  if (val > 0 && originalHeight.value > 0) {
    percent.value = Math.round((val / originalHeight.value) * 100)
    if (keepAspect.value && originalWidth.value > 0) {
      resizeWidth.value = Math.round(originalWidth.value * val / originalHeight.value)
    }
  }
  updatingResize.value = false
}

function onPercentInput(e: Event) {
  if (updatingResize.value) return
  const val = Number((e.target as HTMLInputElement).value)
  if (isNaN(val) || val < 0) return
  updatingResize.value = true
  resizeSource.value = 'percent'
  percent.value = val
  if (originalWidth.value > 0) {
    resizeWidth.value = Math.round(originalWidth.value * val / 100)
  }
  if (originalHeight.value > 0) {
    resizeHeight.value = Math.round(originalHeight.value * val / 100)
  }
  updatingResize.value = false
}

const computedPercent = computed(() => {
  if (resizeWidth.value > 0 && originalWidth.value > 0) {
    return Math.round((resizeWidth.value / originalWidth.value) * 100)
  }
  return percent.value
})

// Convert params
const targetFormat = ref('png')

// Crop params
const cropX = ref(0)
const cropY = ref(0)
const cropW = ref(0)
const cropH = ref(0)

// ============ Computed ============

const fileName = computed(() => {
  if (!originalPath.value) {return ''}
  const parts = originalPath.value.replace(/\\/g, '/').split('/')
  return parts[parts.length - 1] || ''
})

const originalUrl = computed(() => {
  if (!originalPath.value) {return ''}
  return convertFileSrc(originalPath.value)
})

const processedUrl = computed(() => {
  if (!processedPath.value) {return ''}
  return convertFileSrc(processedPath.value)
})

// ============ Functions ============

const functions = [
  { id: 'compress', label: '压缩', icon: 'compress' },
  { id: 'resize', label: '尺寸调整', icon: 'maximize' },
  { id: 'convert', label: '格式转换', icon: 'refresh' },
  { id: 'crop', label: '裁剪', icon: 'crop' },
  { id: 'removeBg', label: '智能抠图', icon: 'sparkles' },
]

// ============ Methods ============

async function selectFile() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: '图片', extensions: ['jpg', 'jpeg', 'png', 'webp', 'gif', 'bmp'] }],
    })
    if (selected) {
      loadFile(selected as string)
    }
  } catch (e: any) {
    console.error('Failed to open file dialog:', e)
  }
}

function onDrop(event: DragEvent) {
  dragOver.value = false
  const files = event.dataTransfer?.files
  if (files && files.length > 0) {
    const file = files[0]
    // For drag-and-drop in Tauri, we get a File object with a path property
    const path = (file as any).path
    if (path) {
      loadFile(path)
    } else {
      errorMsg.value = '请使用文件选择按钮选择图片（拖拽在 Tauri 中可能不支持路径读取）'
    }
  }
}

function loadFile(path: string) {
  originalPath.value = path
  processedPath.value = ''
  originalSize.value = 0
  processedSize.value = 0
  originalWidth.value = 0
  originalHeight.value = 0
  errorMsg.value = ''
  viewMode.value = 'split'
  // Try to get file size
  getFileSize(path)
  // Get original dimensions via Image
  getOriginalDimensions(path)
}

function getOriginalDimensions(path: string) {
  const url = convertFileSrc(path)
  const img = new Image()
  img.onload = () => {
    originalWidth.value = img.naturalWidth
    originalHeight.value = img.naturalHeight
    URL.revokeObjectURL(url)
  }
  img.onerror = () => URL.revokeObjectURL(url)
  img.src = url
}

async function getFileSize(path: string) {
  try {
    const result = await invoke<{ size: number }>('file_size', { path })
    originalSize.value = result.size
  } catch {
    // ignore, size fetching is optional
  }
}

function clearFile() {
  originalPath.value = ''
  processedPath.value = ''
  originalSize.value = 0
  processedSize.value = 0
  originalWidth.value = 0
  originalHeight.value = 0
  errorMsg.value = ''
}

async function processImage() {
  if (!originalPath.value || processing.value) {return}
  processing.value = true
  errorMsg.value = ''
  processedPath.value = ''

  try {
    let result: string = ''

    switch (activeFunction.value) {
      case 'compress':
        result = await invoke<string>('image_compress', {
          path: originalPath.value,
          quality: quality.value,
          format: format.value,
        })
        break

      case 'resize':
        result = await invoke<string>('image_resize', {
          path: originalPath.value,
          width: resizeWidth.value > 0 ? resizeWidth.value : null,
          height: resizeHeight.value > 0 ? resizeHeight.value : null,
          // 只在用户明确使用了百分比（且没有填宽高）时才传 percent
          percent: (resizeWidth.value <= 0 && resizeHeight.value <= 0) ? percent.value : null,
          keepAspect: keepAspect.value,
        })
        break

      case 'convert':
        result = await invoke<string>('image_convert', {
          path: originalPath.value,
          targetFormat: targetFormat.value,
        })
        break

      case 'crop':
        result = await invoke<string>('image_crop', {
          path: originalPath.value,
          x: cropX.value,
          y: cropY.value,
          width: cropW.value,
          height: cropH.value,
        })
        break

      case 'removeBg':
        result = await invoke<string>('image_remove_bg', {
          path: originalPath.value,
        })
        break
    }

    processedPath.value = result
    // Try to get processed file size
    await getProcessedFileSize(result)
  } catch (e: any) {
    errorMsg.value = e?.message || String(e) || '处理失败'
    console.error('Image processing error:', e)
  } finally {
    processing.value = false
  }
}

async function getProcessedFileSize(path: string) {
  try {
    const result = await invoke<{ size: number }>('file_size', { path })
    processedSize.value = result.size
  } catch {
    // ignore
  }
}

async function downloadImage() {
  if (!processedPath.value) {return}
  try {
    const { save } = await import('@tauri-apps/plugin-dialog')
    // 正确处理文件名：在扩展名前插入 _processed
    const baseName = fileName.value.replace(/\.\w+$/, '')
    const ext = fileName.value.match(/\.(\w+)$/)?.[1] || 'png'
    const destPath = await save({
      defaultPath: `${baseName}_processed.${ext}`,
      filters: [{ name: '图片', extensions: ['jpg', 'jpeg', 'png', 'webp', 'gif', 'bmp'] }],
    })
    if (destPath) {
      await copyFile(processedPath.value, destPath as string)
      errorMsg.value = ''
    }
  } catch (e: any) {
    console.error('Download failed:', e)
    errorMsg.value = '下载失败: ' + (e?.message || String(e))
  }
}

function formatSize(bytes: number): string {
  if (bytes === 0) {return '0 B'}
  const units = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return (bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0) + ' ' + units[i]
}

// ============ Image display measurement for crop overlay ============

function measureImage() {
  const img = imgElementRef.value
  if (!img || !img.complete || img.naturalWidth === 0) { return }
  imgDisplayWidth.value = img.clientWidth
  imgDisplayHeight.value = img.clientHeight
  const container = viewMode.value === 'split' ? imgContainerRef2.value : imgContainerRef.value
  if (container) {
    const containerRect = container.getBoundingClientRect()
    const imgRect = img.getBoundingClientRect()
    imgOffsetX.value = imgRect.left - containerRect.left
    imgOffsetY.value = imgRect.top - containerRect.top
  }
}

// Watch for image load to measure
watch(originalUrl, (url) => {
  if (url) {
    nextTick(() => {
      const img = imgElementRef.value
      if (img) {
        img.onload = () => measureImage()
        if (img.complete) { measureImage() }
      }
    })
  } else {
    imgDisplayWidth.value = 0
    imgDisplayHeight.value = 0
    cropX.value = 0
    cropY.value = 0
    cropW.value = 0
    cropH.value = 0
  }
})

// Resize observer to re-measure when container resizes
let resizeObserver: ResizeObserver | null = null

watch([imgContainerRef, imgContainerRef2, viewMode], () => {
  resizeObserver?.disconnect()
  const el = viewMode.value === 'split' ? imgContainerRef2.value : imgContainerRef.value
  if (el) {
    resizeObserver = new ResizeObserver(() => {
      if (originalUrl.value) { measureImage() }
    })
    resizeObserver.observe(el)
  }
})

onUnmounted(() => {
  resizeObserver?.disconnect()
})
</script>
