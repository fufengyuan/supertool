<template>
  <div class="todo-description bg-base-100 rounded-lg border border-base-content/10 overflow-hidden">
    <!-- 标签栏 + 工具栏 -->
    <div class="flex items-center justify-between gap-2 px-2 py-1 border-b border-base-content/10 bg-base-200/50">
      <div class="flex items-center gap-0.5">
        <button
          @click="tab = 'edit'"
          class="px-2.5 py-1 text-xs font-medium rounded-md transition-colors duration-100"
          :class="tab === 'edit' ? 'bg-primary/10 text-primary' : 'text-base-content/50 hover:text-base-content/70'"
        >编辑</button>
        <button
          @click="tab = 'preview'"
          class="px-2.5 py-1 text-xs font-medium rounded-md transition-colors duration-100"
          :class="tab === 'preview' ? 'bg-primary/10 text-primary' : 'text-base-content/50 hover:text-base-content/70'"
        >预览</button>
      </div>
      <!-- 格式工具栏（仅在编辑模式显示） -->
      <div v-if="tab === 'edit'" class="flex items-center gap-0.5 flex-wrap">
        <button @click="wrap('**', '**')" title="粗体" class="toolbar-btn" ref="toolbarBtns"><strong style="font-size:13px">B</strong></button>
        <button @click="wrap('*', '*')" title="斜体" class="toolbar-btn"><em style="font-size:13px;font-style:italic">I</em></button>
        <button @click="wrap('~~', '~~')" title="删除线" class="toolbar-btn"><span style="font-size:13px;text-decoration:line-through">S</span></button>
        <span class="w-px h-3.5 bg-base-content/15 mx-0.5"></span>
        <button @click="insertHeading" title="标题" class="toolbar-btn"><span style="font-size:11px;font-weight:700">H</span></button>
        <button @click="insertBeforeLines('- ')" title="无序列表" class="toolbar-btn"><SvgIcon name="list" size="13" /></button>
        <button @click="insertOrderedList" title="有序列表" class="toolbar-btn"><span style="font-size:10px;font-weight:600">1.</span></button>
        <button @click="insertBeforeLines('- [ ] ')" title="任务列表" class="toolbar-btn"><span style="font-size:11px">☐</span></button>
        <span class="w-px h-3.5 bg-base-content/15 mx-0.5"></span>
        <button @click="insertLink" title="链接" class="toolbar-btn"><SvgIcon name="link" size="13" /></button>
        <button @click="insertImage" title="图片" class="toolbar-btn"><SvgIcon name="image" size="13" /></button>
        <button @click="wrap('`', '`')" title="行内代码" class="toolbar-btn"><SvgIcon name="code" size="13" /></button>
        <button @click="insertCodeBlock" title="代码块" class="toolbar-btn"><span style="font-size:10px;font-weight:700;font-family:monospace">&lt;/&gt;</span></button>
        <span class="w-px h-3.5 bg-base-content/15 mx-0.5"></span>
        <button @click="insertBeforeLines('> ')" title="引用" class="toolbar-btn"><span style="font-size:12px;font-weight:700">"</span></button>
        <button @click="insertHr" title="分隔线" class="toolbar-btn"><span style="font-size:12px;font-weight:600">—</span></button>
      </div>
    </div>

    <!-- 编辑模式 -->
    <div v-if="tab === 'edit'" class="relative">
      <textarea
        ref="textareaRef"
        :value="localContent"
        @input="onInput"
        @keydown="handleTabKey"
        class="w-full min-h-[180px] p-3 text-sm font-mono leading-relaxed bg-transparent border-none outline-none resize-y focus:ring-0"
        placeholder="输入 Markdown 格式的详细描述…

# 标题
**粗体** *斜体* ~~删除线~~
- 列表项
> 引用
`代码`"
        spellcheck="false"
      />
      <!-- 保存/取消（编辑模式下显示） -->
      <div v-if="isEditing" class="flex items-center gap-2 justify-end px-3 py-2 border-t border-base-content/10 bg-base-200/30">
        <button @click="$emit('cancel')" class="btn btn-ghost btn-xs">取消</button>
        <button @click="handleSave" class="btn btn-primary btn-xs gap-1">
          <SvgIcon name="check" size="12" /> 保存
        </button>
      </div>
    </div>

    <!-- 预览模式 -->
    <div
      v-else
      class="p-3 min-h-[120px] markdown-preview cursor-default"
      @dblclick="switchToEdit"
    >
      <div v-if="localContent" v-html="renderedHtml"></div>
      <div v-else class="flex flex-col items-center justify-center py-8 text-base-content/30">
        <SvgIcon name="fileText" size="32" class="mb-2 opacity-40" />
        <p class="text-xs">暂无详细内容，双击此处开始编辑</p>
      </div>
    </div>

    <!-- 空状态（未编辑且无内容时） -->
    <div
      v-if="!isEditing && !localContent && tab === 'edit'"
      class="absolute inset-0 flex items-center justify-center pointer-events-none"
    >
      <p class="text-xs text-base-content/30">开始输入 Markdown 内容…</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { marked } from 'marked'
import DOMPurify from 'dompurify'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { useErrorHandler } from '../../composables/useErrorHandler'

const { handleError } = useErrorHandler()

const props = defineProps({
  markdown: { type: String, default: '' },
  isEditing: { type: Boolean, default: false },
  content: { type: String, default: '' },
})

const emit = defineEmits<{
  'update:content': [value: string]
  save: []
  cancel: []
  'start-edit': []
}>()

const textareaRef = ref<HTMLTextAreaElement | null>(null)
const tab = ref<'edit' | 'preview'>(props.isEditing ? 'edit' : 'preview')
const localContent = ref(props.isEditing ? props.content : props.markdown)

// 同步外部 prop 变化
watch(() => props.markdown, (val) => {
  if (!props.isEditing) {
    localContent.value = val || ''
  }
})

watch(() => props.isEditing, (val) => {
  if (val) {
    localContent.value = props.content || ''
    tab.value = 'edit'
    nextTick(() => autoResize())
  }
})

watch(() => props.content, (val) => {
  if (props.isEditing) {
    localContent.value = val || ''
  }
})

const renderedHtml = computed(() => {
  if (!localContent.value) return ''
  try {
    const html = marked.parse(localContent.value, { async: false }) as string
    return DOMPurify.sanitize(html)
  } catch (error) {
    handleError(error, { context: 'Markdown渲染', showToast: false })
    return DOMPurify.sanitize(localContent.value)
  }
})

function onInput(e: Event) {
  const target = e.target as HTMLTextAreaElement
  localContent.value = target.value
  emit('update:content', localContent.value)
  autoResize()
}

function autoResize() {
  const ta = textareaRef.value
  if (!ta) return
  ta.style.height = 'auto'
  ta.style.height = Math.max(180, ta.scrollHeight) + 'px'
}

function handleSave() {
  emit('update:content', localContent.value)
  emit('save')
}

function switchToEdit() {
  if (!props.isEditing) {
    emit('start-edit')
    tab.value = 'edit'
  }
}

function handleTabKey(e: KeyboardEvent) {
  if (e.key === 'Tab') {
    e.preventDefault()
    const ta = textareaRef.value
    if (!ta) return
    const start = ta.selectionStart
    const end = ta.selectionEnd
    localContent.value = localContent.value.substring(0, start) + '  ' + localContent.value.substring(end)
    emit('update:content', localContent.value)
    nextTick(() => {
      ta.selectionStart = ta.selectionEnd = start + 2
      autoResize()
    })
  }
}

// ===== 工具栏辅助函数 =====

function getTextarea(): HTMLTextAreaElement | null {
  return textareaRef.value
}

function replaceRange(textarea: HTMLTextAreaElement, start: number, end: number, replacement: string) {
  localContent.value = localContent.value.substring(0, start) + replacement + localContent.value.substring(end)
  emit('update:content', localContent.value)
  return replacement.length
}

function wrap(before: string, after: string) {
  const ta = getTextarea()
  if (!ta) return
  const start = ta.selectionStart
  const end = ta.selectionEnd
  const selected = localContent.value.substring(start, end)

  replaceRange(ta, start, end, before + selected + after)
  nextTick(() => {
    ta.focus()
    if (selected) {
      ta.setSelectionRange(start + before.length, end + before.length)
    } else {
      const cursorPos = start + before.length
      ta.setSelectionRange(cursorPos, cursorPos)
    }
  })
}

function getLineStart(textarea: HTMLTextAreaElement): number {
  const value = localContent.value
  let pos = textarea.selectionStart
  while (pos > 0 && value[pos - 1] !== '\n') pos--
  return pos
}

function getLineEnd(textarea: HTMLTextAreaElement): number {
  const value = localContent.value
  let pos = textarea.selectionEnd
  const len = value.length
  while (pos < len && value[pos] !== '\n') pos++
  return pos
}

function insertBeforeLines(prefix: string) {
  const ta = getTextarea()
  if (!ta) return
  const start = ta.selectionStart
  const end = ta.selectionEnd
  let selected = localContent.value.substring(start, end)

  if (!selected) {
    // 如果没有选中，对当前行操作
    const lineStart = getLineStart(ta)
    const lineEnd = getLineEnd(ta)
    selected = localContent.value.substring(lineStart, lineEnd)
    const newLine = prefix + selected
    replaceRange(ta, lineStart, lineEnd, newLine)
    nextTick(() => {
      ta.focus()
      const cursorPos = lineStart + newLine.length
      ta.setSelectionRange(cursorPos, cursorPos)
      autoResize()
    })
    return
  }

  // 对选中的多行操作
  const lines = selected.split('\n')
  const newText = lines.map(l => prefix + l).join('\n')
  replaceRange(ta, start, end, newText)
  nextTick(() => {
    ta.focus()
    ta.setSelectionRange(start, start + newText.length)
    autoResize()
  })
}

function insertHeading() {
  const ta = getTextarea()
  if (!ta) return
  const start = ta.selectionStart
  const lineStart = getLineStart(ta)
  const lineText = localContent.value.substring(lineStart, getLineEnd(ta))

  // 判断当前标题级别并循环：无→#→##→###→无
  const match = lineText.match(/^(#{1,6})\s/)
  if (match) {
    const level = match[1].length
    if (level < 3) {
      // 升级标题
      replaceRange(ta, lineStart, lineStart + match[0].length, '#'.repeat(level + 1) + ' ')
    } else {
      // 移除标题
      replaceRange(ta, lineStart, lineStart + match[0].length, '')
    }
  } else {
    // 插入 #
    replaceRange(ta, lineStart, lineStart, '# ')
  }
  nextTick(() => { ta.focus(); autoResize() })
}

function insertOrderedList() {
  const ta = getTextarea()
  if (!ta) return
  const start = ta.selectionStart
  const end = ta.selectionEnd
  let selected = localContent.value.substring(start, end)

  if (!selected) {
    insertBeforeLines('1. ')
    return
  }

  const lines = selected.split('\n')
  const newText = lines.map((l, i) => `${i + 1}. ${l}`).join('\n')
  replaceRange(ta, start, end, newText)
  nextTick(() => {
    ta.focus()
    ta.setSelectionRange(start, start + newText.length)
    autoResize()
  })
}

function insertLink() {
  const ta = getTextarea()
  if (!ta) return
  const start = ta.selectionStart
  const end = ta.selectionEnd
  const selected = localContent.value.substring(start, end)

  if (selected) {
    replaceRange(ta, start, end, `[${selected}](url)`)
    nextTick(() => {
      ta.focus()
      const urlStart = start + selected.length + 3
      ta.setSelectionRange(urlStart, urlStart + 3)
    })
  } else {
    replaceRange(ta, start, end, '[链接文字](url)')
    nextTick(() => {
      ta.focus()
      ta.setSelectionRange(start + 1, start + 5)
    })
  }
}

function insertImage() {
  const ta = getTextarea()
  if (!ta) return
  const start = ta.selectionStart
  const end = ta.selectionEnd
  const selected = localContent.value.substring(start, end)

  if (selected) {
    replaceRange(ta, start, end, `![${selected}](url)`)
  } else {
    replaceRange(ta, start, end, '![图片描述](url)')
  }
  nextTick(() => { ta.focus(); autoResize() })
}

function insertCodeBlock() {
  const ta = getTextarea()
  if (!ta) return
  const start = ta.selectionStart
  const end = ta.selectionEnd
  const selected = localContent.value.substring(start, end)

  if (selected) {
    replaceRange(ta, start, end, '```\n' + selected + '\n```')
    nextTick(() => {
      ta.focus()
      ta.setSelectionRange(start + 4, end + 4)
    })
  } else {
    replaceRange(ta, start, end, '```\n代码\n```')
    nextTick(() => {
      ta.focus()
      const cursorPos = start + 4
      ta.setSelectionRange(cursorPos, cursorPos + 2)
    })
  }
}

function insertHr() {
  const ta = getTextarea()
  if (!ta) return
  const start = ta.selectionStart
  const end = ta.selectionEnd

  // 如果光标不在行首，先换行
  const lineStart = getLineStart(ta)
  const prefix = start > lineStart ? '\n' : ''
  replaceRange(ta, end, end, prefix + '\n---\n')
  nextTick(() => {
    ta.focus()
    const cursorPos = end + prefix.length + 5
    ta.setSelectionRange(cursorPos, cursorPos)
    autoResize()
  })
}
</script>

<style scoped>
.toolbar-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 24px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--color-base-content);
  opacity: 0.55;
  cursor: default;
  transition: all 0.1s ease;
}
.toolbar-btn:hover {
  opacity: 1;
  background: color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.markdown-preview {
  font-size: 13px;
  line-height: 1.7;
  color: var(--color-base-content);
}
.markdown-preview :deep(h1) { font-size: 1.4em; font-weight: 700; margin: 0.6em 0 0.3em; padding-bottom: 0.2em; border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); }
.markdown-preview :deep(h2) { font-size: 1.2em; font-weight: 700; margin: 0.5em 0 0.25em; padding-bottom: 0.15em; border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 8%, transparent); }
.markdown-preview :deep(h3) { font-size: 1.1em; font-weight: 600; margin: 0.4em 0 0.2em; }
.markdown-preview :deep(h4) { font-size: 1em; font-weight: 600; margin: 0.3em 0 0.15em; }
.markdown-preview :deep(p) { margin: 0.4em 0; }
.markdown-preview :deep(ul), .markdown-preview :deep(ol) { padding-left: 1.5em; margin: 0.3em 0; }
.markdown-preview :deep(li) { margin: 0.15em 0; }
.markdown-preview :deep(blockquote) {
  margin: 0.4em 0;
  padding: 0.3em 0.8em;
  border-left: 3px solid var(--color-primary);
  background: color-mix(in oklab, var(--color-primary) 5%, transparent);
  border-radius: 0 4px 4px 0;
}
.markdown-preview :deep(code) {
  font-family: 'SF Mono', 'Fira Code', 'JetBrains Mono', monospace;
  font-size: 0.9em;
  padding: 0.15em 0.4em;
  background: color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 3px;
}
.markdown-preview :deep(pre) {
  margin: 0.5em 0;
  padding: 0.8em;
  background: color-mix(in oklab, var(--color-base-content) 8%, transparent);
  border-radius: 6px;
  overflow-x: auto;
}
.markdown-preview :deep(pre code) {
  background: none;
  padding: 0;
  font-size: 0.85em;
}
.markdown-preview :deep(a) { color: var(--color-primary); text-decoration: underline; }
.markdown-preview :deep(img) { max-width: 100%; border-radius: 6px; margin: 0.5em 0; }
.markdown-preview :deep(hr) { margin: 1em 0; border: none; border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); }
.markdown-preview :deep(table) { width: 100%; border-collapse: collapse; margin: 0.5em 0; }
.markdown-preview :deep(th), .markdown-preview :deep(td) { padding: 0.4em 0.6em; border: 1px solid color-mix(in oklab, var(--color-base-content) 15%, transparent); text-align: left; }
.markdown-preview :deep(th) { font-weight: 600; background: color-mix(in oklab, var(--color-base-content) 5%, transparent); }
.markdown-preview :deep(del) { text-decoration: line-through; opacity: 0.7; }
.markdown-preview :deep(input[type="checkbox"]) { margin-right: 0.4em; }
</style>
