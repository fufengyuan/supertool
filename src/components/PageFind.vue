<template>
  <div v-if="visible" class="page-find fixed top-4 right-4 z-50 bg-base-100 border border-base-content/20 rounded-lg shadow-lg p-2 flex items-center gap-2">
    <SvgIcon name="search" size="14" class="text-base-content/60" />
    <input
      ref="inputRef"
      v-model="query"
      type="text"
      placeholder="页面内查找..."
      class="input input-sm input-bordered w-48"
      @keydown.enter="findNext"
      @keydown.escape="close"
      @keydown.shift.enter="findPrev"
    />
    <span v-if="matches > 0" class="text-xs text-base-content/60">{{ currentMatch }}/{{ matches }}</span>
    <button @click="findPrev" class="btn btn-ghost btn-xs" :disabled="matches === 0">
      <SvgIcon name="chevronUp" size="12" />
    </button>
    <button @click="findNext" class="btn btn-ghost btn-xs" :disabled="matches === 0">
      <SvgIcon name="chevronDown" size="12" />
    </button>
    <button @click="close" class="btn btn-ghost btn-xs">
      <SvgIcon name="x" size="12" />
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'

const visible = ref(false)
const query = ref('')
const inputRef = ref<HTMLInputElement | null>(null)
const matches = ref(0)
const currentMatch = ref(0)
const highlightedElements: HTMLElement[] = []
let currentHighlightIndex = -1

// Open find panel
function open() {
  visible.value = true
  query.value = ''
  matches.value = 0
  currentMatch.value = 0
  setTimeout(() => inputRef.value?.focus(), 50)
}

// Close and clear highlights
function close() {
  visible.value = false
  clearHighlights()
}

// Clear all highlights
function clearHighlights() {
  for (const el of highlightedElements) {
    const parent = el.parentNode
    if (parent) {
      parent.replaceChild(document.createTextNode(el.textContent || ''), el)
    }
  }
  highlightedElements.length = 0
  currentHighlightIndex = -1
  matches.value = 0
  currentMatch.value = 0
}

// Highlight matches in the page
function highlightMatches() {
  if (!query.value.trim()) {
    clearHighlights()
    return
  }

  clearHighlights()

  // Search in visible content area (main content, excluding sidebar)
  const contentArea = document.querySelector('.main-content') || document.body
  const walker = document.createTreeWalker(
    contentArea,
    NodeFilter.SHOW_TEXT,
    null
  )

  const textNodes: Text[] = []
  while (walker.nextNode()) {
    const node = walker.currentNode as Text
    if (node.parentElement?.closest('.page-find, .modal, .sidebar, script, style')) continue
    if (node.textContent && node.textContent.toLowerCase().includes(query.value.toLowerCase())) {
      textNodes.push(node)
    }
  }

  for (const node of textNodes) {
    const text = node.textContent || ''
    const lowerText = text.toLowerCase()
    const lowerQuery = query.value.toLowerCase()
    let idx = lowerText.indexOf(lowerQuery)
    
    while (idx !== -1 && highlightedElements.length < 100) {
      const before = text.substring(0, idx)
      const match = text.substring(idx, idx + query.value.length)
      const after = text.substring(idx + query.value.length)
      
      const span = document.createElement('span')
      span.className = 'page-find-highlight bg-warning/30 rounded px-0.5'
      span.textContent = match
      
      const parent = node.parentNode
      if (parent) {
        const newNode = node.splitText(idx)
        newNode.splitText(query.value.length)
        parent.insertBefore(span, newNode)
        parent.insertBefore(document.createTextNode(before), span)
        highlightedElements.push(span)
        // Continue search in the remaining text
        idx = after.toLowerCase().indexOf(lowerQuery)
      }
    }
  }

  matches.value = highlightedElements.length
  if (matches.value > 0) {
    currentHighlightIndex = 0
    scrollToHighlight(0)
  }
}

// Scroll to a specific highlight
function scrollToHighlight(index: number) {
  if (index < 0 || index >= highlightedElements.length) return
  
  // Remove previous active highlight
  if (currentHighlightIndex >= 0 && highlightedElements[currentHighlightIndex]) {
    highlightedElements[currentHighlightIndex].classList.remove('bg-warning', 'text-warning-content')
    highlightedElements[currentHighlightIndex].classList.add('bg-warning/30')
  }
  
  currentHighlightIndex = index
  currentMatch.value = index + 1
  
  const el = highlightedElements[index]
  el.classList.remove('bg-warning/30')
  el.classList.add('bg-warning', 'text-warning-content')
  el.scrollIntoView({ behavior: 'smooth', block: 'center' })
}

// Find next match
function findNext() {
  if (matches.value === 0) return
  const next = (currentHighlightIndex + 1) % matches.value
  scrollToHighlight(next)
}

// Find previous match
function findPrev() {
  if (matches.value === 0) return
  const prev = (currentHighlightIndex - 1 + matches.value) % matches.value
  scrollToHighlight(prev)
}

// Watch query changes
watch(query, () => {
  highlightMatches()
})

// Global keyboard shortcut handler
function handleKeyDown(e: KeyboardEvent) {
  const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0
  const mod = isMac ? e.metaKey : e.ctrlKey
  
  // Cmd/Ctrl+F: open find
  if (mod && (e.key === 'f' || e.key === 'F')) {
    e.preventDefault()
    if (visible.value) {
      inputRef.value?.focus()
    } else {
      open()
    }
  }
  
  // Cmd/Ctrl+G: find next (when find panel is open)
  if (visible.value && mod && (e.key === 'g' || e.key === 'G')) {
    e.preventDefault()
    if (e.shiftKey) {
      findPrev()
    } else {
      findNext()
    }
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeyDown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeyDown)
  clearHighlights()
})

// Expose for external use
defineExpose({ open, close })
</script>

<style scoped>
.page-find {
  animation: fadeIn 0.15s ease-out;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(-4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>