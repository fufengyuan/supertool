<template>
  <div class="flex flex-col h-full overflow-hidden bg-base-200">
    <!-- 顶部栏 -->
    <header class="shrink-0 flex items-center gap-3 px-4 py-2.5 border-b border-base-content/10 bg-base-100 z-10">
      <div class="flex items-center gap-2 shrink-0">
        <span class="w-8 h-8 rounded-lg bg-primary/15 text-primary flex items-center justify-center">
          <SvgIcon name="file" size="15" />
        </span>
        <h1 class="m-0 text-base font-bold text-base-content">笔记</h1>
        <span class="text-[11px] text-base-content/40 hidden sm:inline">{{ filteredNotes.length }} 篇</span>
      </div>

      <div class="flex-1"></div>

      <!-- 搜索 -->
      <div class="relative shrink-0">
        <SvgIcon name="search" size="13" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-base-content/40 pointer-events-none" />
        <input
          v-model="searchQuery"
          class="input input-sm w-44 lg:w-56 pl-8 rounded-lg bg-base-200/60 border-base-content/10"
          placeholder="搜索标题或内容..."
        />
        <button v-if="searchQuery" class="absolute right-2 top-1/2 -translate-y-1/2 btn btn-ghost btn-xs btn-circle" @click="searchQuery = ''">
          <SvgIcon name="x" size="11" />
        </button>
      </div>

      <!-- 分组筛选 -->
      <div class="relative shrink-0" ref="groupFilterRef">
        <button class="btn btn-sm border border-base-content/10 bg-base-200/60 gap-1.5" @click="showGroupFilter = !showGroupFilter">
          <SvgIcon name="folder" size="13" class="text-base-content/60" />
          <span class="max-w-[90px] truncate">{{ groupFilterLabel }}</span>
          <SvgIcon name="chevronDown" size="11" class="text-base-content/40" />
        </button>
        <div v-if="showGroupFilter" class="absolute right-0 top-full mt-1.5 w-64 bg-base-100 border border-base-content/10 rounded-xl shadow-xl z-50 p-1.5">
          <div
            class="flex items-center gap-2 px-2.5 py-2 rounded-lg cursor-pointer text-sm hover:bg-base-200 transition-colors"
            :class="{ 'bg-primary/10 text-primary': groupFilter === 'all' }"
            @click="groupFilter = 'all'; showGroupFilter = false"
          >
            <SvgIcon name="file" size="13" /> 全部笔记
            <span class="ml-auto text-[11px] text-base-content/40">{{ notes.length }}</span>
          </div>
          <div
            class="flex items-center gap-2 px-2.5 py-2 rounded-lg cursor-pointer text-sm hover:bg-base-200 transition-colors"
            :class="{ 'bg-primary/10 text-primary': groupFilter === '__ungrouped__' }"
            @click="groupFilter = '__ungrouped__'; showGroupFilter = false"
          >
            <SvgIcon name="inbox" size="13" /> 未分组
            <span class="ml-auto text-[11px] text-base-content/40">{{ ungroupedCount }}</span>
          </div>
          <div class="border-t border-base-content/10 my-1"></div>
          <div v-for="g in noteGroups" :key="g.id" class="group/row">
            <template v-if="inlineRenameId === g.id">
              <div class="flex items-center gap-1.5 px-1.5 py-1">
                <input v-model="inlineRenameName" class="input input-xs flex-1" @keyup.enter="saveRenameGroup(g)" @keyup.esc="inlineRenameId = null" />
                <button class="btn btn-primary btn-xs" @click="saveRenameGroup(g)">保存</button>
                <button class="btn btn-ghost btn-xs" @click="inlineRenameId = null">取消</button>
              </div>
            </template>
            <template v-else>
              <div
                class="flex items-center gap-2 px-2.5 py-2 rounded-lg cursor-pointer text-sm hover:bg-base-200 transition-colors"
                :class="{ 'bg-primary/10 text-primary': groupFilter === g.id }"
                @click="groupFilter = g.id; showGroupFilter = false"
              >
                <span class="text-sm">{{ g.icon || '📁' }}</span>
                <span class="flex-1 truncate">{{ g.name }}</span>
                <span class="text-[11px] text-base-content/40">{{ getGroupNoteCount(g.id) }}</span>
                <button class="opacity-0 group-hover/row:opacity-100 text-base-content/40 hover:text-base-content transition-all p-0.5" title="重命名" @click.stop="startRenameGroup(g)"><SvgIcon name="pencil" size="11" /></button>
                <button class="opacity-0 group-hover/row:opacity-100 text-base-content/40 hover:text-error transition-all p-0.5" title="删除" @click.stop="confirmDeleteGroup(g)"><SvgIcon name="trash" size="11" /></button>
              </div>
            </template>
          </div>
          <div class="border-t border-base-content/10 my-1"></div>
          <template v-if="creatingGroup">
            <div class="flex items-center gap-1.5 px-1.5 py-1">
              <input v-model="createGroupName" ref="createGroupInputRef" class="input input-xs flex-1" placeholder="分组名称" @keyup.enter="saveCreateGroup" @keyup.esc="creatingGroup = false" />
              <button class="btn btn-primary btn-xs" @click="saveCreateGroup" :disabled="!createGroupName.trim()">创建</button>
              <button class="btn btn-ghost btn-xs" @click="creatingGroup = false">取消</button>
            </div>
          </template>
          <button v-else class="w-full flex items-center gap-2 px-2.5 py-2 rounded-lg text-sm text-base-content/70 hover:bg-base-200 transition-colors" @click="creatingGroup = true; nextTick(() => createGroupInputRef.value?.focus())">
            <SvgIcon name="folderPlus" size="13" /> 新建分组
          </button>
        </div>
      </div>

      <!-- 新建笔记 -->
      <button class="btn btn-primary btn-sm gap-1.5 shrink-0" @click="createNewNote">
        <SvgIcon name="plus" size="13" /> 新建笔记
      </button>
    </header>

    <div class="flex flex-1 min-h-0">
      <!-- 左侧：笔记卡片列表 -->
      <aside class="w-72 min-w-[220px] max-w-[320px] shrink-0 border-r border-base-content/10 bg-base-100 flex flex-col">
        <div class="flex-1 overflow-y-auto p-2 space-y-1.5">
          <div
            v-for="note in filteredNotes"
            :key="note.id"
            class="group/card px-3 py-2.5 rounded-xl border border-base-content/10 cursor-pointer transition-all duration-150 hover:border-primary/40 hover:shadow-sm"
            :class="{ '!border-primary/60 bg-primary/5': selectedNote?.id === note.id }"
            @click="selectNote(note)"
          >
            <div class="flex items-center gap-1.5 mb-1">
              <SvgIcon v-if="note.pinned" name="star" size="11" class="text-yellow-500 shrink-0" />
              <span class="flex-1 font-medium text-sm text-base-content truncate">{{ note.title || '无标题' }}</span>
              <span class="text-[10px] text-base-content/40 shrink-0 whitespace-nowrap">{{ formatDate(note.updatedAt) }}</span>
              <button
                class="shrink-0 opacity-0 group-hover/card:opacity-100 text-base-content/30 hover:text-error transition-all p-0.5"
                title="删除"
                @click.stop="confirmDelete(note)"
              >
                <SvgIcon name="trash" size="11" />
              </button>
            </div>
            <p class="m-0 text-[11px] text-base-content/50 leading-snug line-clamp-2 break-all">{{ getPreview(note.content) }}</p>
          </div>

          <div v-if="filteredNotes.length === 0" class="flex flex-col items-center justify-center py-16 text-base-content/30 gap-2">
            <SvgIcon name="file" size="36" :strokeWidth="1.5" class="opacity-40" />
            <p class="text-xs m-0">{{ searchQuery ? '没有匹配的笔记' : notes.length === 0 ? '点击「新建笔记」开始记录' : '该分组下暂无笔记' }}</p>
          </div>
        </div>
      </aside>

      <!-- 右侧：编辑器 -->
      <main class="flex-1 min-w-0 bg-base-100 flex flex-col">
        <template v-if="selectedNote">
          <!-- 工具栏 -->
          <div class="shrink-0 flex items-center gap-0.5 px-3 py-1.5 border-b border-base-content/10 flex-wrap">
            <template v-if="!showRawMd">
              <button class="toolbar-btn" title="加粗 (Ctrl+B)" @mousedown.prevent @click="execCmd('bold')"><b>B</b></button>
              <button class="toolbar-btn" title="斜体 (Ctrl+I)" @mousedown.prevent @click="execCmd('italic')"><i>I</i></button>
              <button class="toolbar-btn" title="下划线 (Ctrl+U)" @mousedown.prevent @click="execCmd('underline')"><u>U</u></button>
              <button class="toolbar-btn" title="删除线" @mousedown.prevent @click="execCmd('strikeThrough')"><s>S</s></button>
              <span class="toolbar-divider"></span>
              <button class="toolbar-btn" title="标题 1" @mousedown.prevent @click="formatBlock('h1')">H1</button>
              <button class="toolbar-btn" title="标题 2" @mousedown.prevent @click="formatBlock('h2')">H2</button>
              <button class="toolbar-btn" title="标题 3" @mousedown.prevent @click="formatBlock('h3')">H3</button>
              <span class="toolbar-divider"></span>
              <button class="toolbar-btn" title="无序列表" @mousedown.prevent @click="execCmd('insertUnorderedList')"><SvgIcon name="list" size="13" /></button>
              <button class="toolbar-btn" title="有序列表" @mousedown.prevent @click="execCmd('insertOrderedList')"><SvgIcon name="checklist" size="13" /></button>
              <button class="toolbar-btn" title="引用" @mousedown.prevent @click="formatBlock('blockquote')">❝</button>
              <button class="toolbar-btn" title="代码块" @mousedown.prevent @click="insertCodeBlock"><SvgIcon name="code" size="13" /></button>
              <button class="toolbar-btn" title="插入链接" @mousedown.prevent @click="insertLink"><SvgIcon name="link" size="13" /></button>
              <span class="toolbar-divider"></span>
              <button class="toolbar-btn" title="撤销" @mousedown.prevent @click="execCmd('undo')"><SvgIcon name="undo" size="13" /></button>
              <button class="toolbar-btn" title="重做" @mousedown.prevent @click="execCmd('redo')">↪</button>
            </template>

            <div class="flex-1"></div>

            <button class="toolbar-btn" :class="{ '!text-primary': showRawMd }" :title="showRawMd ? '切回富文本' : '查看 Markdown 源码'" @click="toggleRawMd">
              <SvgIcon :name="showRawMd ? 'eye' : 'code'" size="13" /> {{ showRawMd ? '富文本' : '源码' }}
            </button>
            <span class="toolbar-divider"></span>
            <button class="toolbar-btn" :class="{ '!text-yellow-500': selectedNote.pinned }" title="置顶" @click="togglePin">
              <SvgIcon name="star" size="13" />
            </button>
            <!-- 移动分组 -->
            <div class="relative" ref="moveGroupRef">
              <button class="toolbar-btn gap-1" title="移动分组" @click="showGroupSelector = !showGroupSelector">
                <SvgIcon name="folder" size="13" /><span class="max-w-[70px] truncate">{{ getGroupName(selectedNote.groupId) || '未分组' }}</span>
              </button>
              <div v-if="showGroupSelector" class="absolute right-0 top-full mt-1 w-52 bg-base-100 border border-base-content/10 rounded-xl shadow-xl z-50 p-1.5">
                <div class="flex items-center gap-2 px-2.5 py-1.5 rounded-lg cursor-pointer text-xs hover:bg-base-200" :class="{ 'bg-primary/10 text-primary': !selectedNote.groupId }" @click="assignGroup(null)">
                  <SvgIcon name="inbox" size="12" /> 未分组
                </div>
                <div v-for="g in noteGroups" :key="g.id" class="flex items-center gap-2 px-2.5 py-1.5 rounded-lg cursor-pointer text-xs hover:bg-base-200" :class="{ 'bg-primary/10 text-primary': selectedNote.groupId === g.id }" @click="assignGroup(g.id)">
                  <span>{{ g.icon || '📁' }}</span> {{ g.name }}
                </div>
              </div>
            </div>
            <button class="toolbar-btn hover:!text-error" title="删除笔记" @click="confirmDelete(selectedNote)">
              <SvgIcon name="trash" size="13" />
            </button>
          </div>

          <!-- 链接输入条 -->
          <div v-if="linkInputVisible && !showRawMd" class="shrink-0 flex items-center gap-2 px-5 py-1.5 border-b border-base-content/10 bg-base-200/40">
            <SvgIcon name="link" size="12" class="text-base-content/50 shrink-0" />
            <input
              ref="linkInputRef"
              v-model="linkInputUrl"
              class="input input-xs flex-1 font-mono bg-base-100"
              placeholder="输入链接地址，回车应用，Esc 取消"
              @keyup.enter="applyLink"
              @keyup.esc="linkInputVisible = false"
            />
            <button class="btn btn-primary btn-xs" @click="applyLink" :disabled="!linkInputUrl.trim()">插入</button>
          </div>

          <!-- 标题 -->
          <input
            v-model="editorTitle"
            class="shrink-0 input text-xl font-bold text-base-content border-none bg-transparent px-5 pt-3 pb-1 focus:outline-none placeholder:text-base-content/40"
            placeholder="输入标题..."
            @input="onTitleChange"
          />

          <!-- 富文本编辑区 -->
          <div
            v-show="!showRawMd"
            ref="editorRef"
            class="markdown-body flex-1 min-h-0 overflow-y-auto px-5 pb-8 pt-2 focus:outline-none"
            contenteditable="true"
            data-placeholder="开始记录..."
            @input="onRichEdit"
            @keydown.ctrl.b.prevent="execCmd('bold')"
            @keydown.ctrl.i.prevent="execCmd('italic')"
            @keydown.ctrl.u.prevent="execCmd('underline')"
          ></div>

          <!-- 源码编辑区 -->
          <textarea
            v-show="showRawMd"
            v-model="editorContent"
            class="flex-1 min-h-0 w-full font-mono text-sm leading-relaxed text-base-content bg-transparent resize-none px-5 py-3 focus:outline-none"
            placeholder="Markdown 源码..."
            spellcheck="false"
            @input="onContentChange"
          ></textarea>
        </template>

        <div v-else class="flex-1 flex flex-col items-center justify-center text-base-content/40 gap-3">
          <span class="w-16 h-16 rounded-2xl bg-base-200 flex items-center justify-center">
            <SvgIcon name="file" size="28" :strokeWidth="1.5" class="opacity-60" />
          </span>
          <p class="text-sm m-0">选择左侧笔记开始编辑，或点击「新建笔记」</p>
        </div>
      </main>
    </div>

    <!-- 删除确认弹窗 -->
    <Teleport to="body">
      <div v-if="deleteTarget" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[10000]" @click.self="deleteTarget = null">
        <div class="bg-base-100 rounded-2xl p-5 max-w-sm w-[90%] shadow-[0_16px_48px_rgba(0,0,0,0.3)]" @click.stop>
          <h3 class="text-lg font-bold m-0 mb-2 text-base-content flex items-center gap-2">
            <SvgIcon name="alertTriangle" size="15" class="text-error" />
            <template v-if="deleteTarget.title !== undefined">删除笔记</template>
            <template v-else>删除分组</template>
          </h3>
          <p class="text-sm text-base-content/60 m-0 mb-4">
            <template v-if="deleteTarget.title !== undefined">确定要删除「{{ deleteTarget.title || '无标题' }}」吗？此操作不可撤销。</template>
            <template v-else>确定要删除分组「{{ deleteTarget.name }}」吗？分组内的笔记不会被删除，将变为未分组。</template>
          </p>
          <div class="flex justify-end gap-2">
            <button class="btn btn-ghost" @click="deleteTarget = null">取消</button>
            <button class="btn btn-error" @click="executeDelete">删除</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
defineOptions({ name: 'NoteManager' })
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref, computed, onMounted, nextTick, onUnmounted, watch } from 'vue'
import { getTauriAPI } from '../../utils/tauri-api'
import { marked } from 'marked'
import { useToast } from '../../composables/useToast'
import TurndownService from 'turndown'
import DOMPurify from 'dompurify'
const toast = useToast()

const turndownService = new TurndownService({
  headingStyle: 'atx',
  codeBlockStyle: 'fenced',
  emDelimiter: '*',
  bulletListMarker: '-',
})

interface Note {
  id: string; title: string; content: string; tags: string;
  pinned: number; groupId: string | null; createdAt: string; updatedAt: string;
}
interface NoteGroup {
  id: string; name: string; icon: string; sortOrder: number; createdAt: string;
}

const notes = ref<Note[]>([])
const noteGroups = ref<NoteGroup[]>([])
const searchQuery = ref('')
const selectedNote = ref<Note | null>(null)
const showRawMd = ref(false)
const editorTitle = ref('')
const editorContent = ref('')
const saveStatus = ref('')
const deleteTarget = ref<Note | NoteGroup | null>(null)
const showGroupSelector = ref(false)

// 分组筛选（顶部下拉）
const groupFilter = ref<'all' | '__ungrouped__' | string>('all')
const showGroupFilter = ref(false)
const creatingGroup = ref(false)
const createGroupName = ref('')
const inlineRenameId = ref<string | null>(null)
const inlineRenameName = ref('')
const groupFilterRef = ref<HTMLElement | null>(null)
const moveGroupRef = ref<HTMLElement | null>(null)
const createGroupInputRef = ref<HTMLInputElement | null>(null)
const linkInputRef = ref<HTMLInputElement | null>(null)

// 富文本
const editorRef = ref<HTMLDivElement | null>(null)
// 链接输入条（Tauri WebView 中 window.prompt 通常不弹窗，用自绘输入）
const linkInputVisible = ref(false)
const linkInputUrl = ref('')
// 打开输入条前的编辑器选区，应用链接时恢复（焦点移到 input 后选区会被清空）
const savedLinkRange = ref<Range | null>(null)

let saveTimer: ReturnType<typeof setTimeout> | null = null

/* ─── 笔记列表（筛选 + 排序） ─── */
const ungroupedCount = computed(() => notes.value.filter(n => !n.groupId).length)

const filteredNotes = computed(() => {
  let list = notes.value
  if (groupFilter.value === '__ungrouped__') {
    list = list.filter(n => !n.groupId)
  } else if (groupFilter.value !== 'all') {
    list = list.filter(n => n.groupId === groupFilter.value)
  }
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase()
    list = list.filter(n => n.title.toLowerCase().includes(q) || n.content.toLowerCase().includes(q))
  }
  // 置顶优先，其次按更新时间倒序
  return [...list].sort((a, b) => {
    if ((a.pinned ? 1 : 0) !== (b.pinned ? 1 : 0)) { return (b.pinned ? 1 : 0) - (a.pinned ? 1 : 0) }
    return (b.updatedAt || '').localeCompare(a.updatedAt || '')
  })
})

const groupFilterLabel = computed(() => {
  if (groupFilter.value === 'all') { return '全部笔记' }
  if (groupFilter.value === '__ungrouped__') { return '未分组' }
  const g = noteGroups.value.find(x => x.id === groupFilter.value)
  return g ? g.name : '全部笔记'
})

function getGroupNoteCount(groupId: string): number {
  return notes.value.filter(n => n.groupId === groupId).length
}

function getGroupName(groupId: string | null | undefined): string {
  if (!groupId) { return '' }
  const group = noteGroups.value.find(g => g.id === groupId)
  return group ? group.name : ''
}

function getPreview(content: string): string {
  if (!content) { return '空笔记' }
  const plain = content.replace(/[#*`>[\](),!_-]/g, ' ').replace(/\s+/g, ' ').trim()
  return plain.slice(0, 80) + (plain.length > 80 ? '...' : '')
}

function formatDate(iso: string): string {
  if (!iso) { return '' }
  const d = new Date(iso)
  if (isNaN(d.getTime())) { return '' }
  const now = new Date()
  const mins = Math.floor((now.getTime() - d.getTime()) / 60000)
  if (mins < 1) { return '刚刚' }
  if (mins < 60) { return `${mins} 分钟前` }
  const hours = Math.floor(mins / 60)
  if (hours < 24) { return `${hours} 小时前` }
  if (hours < 24 * 7) { return `${Math.floor(hours / 24)} 天前` }
  return d.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' })
}

/* ─── 加载 ─── */
async function loadNotes() {
  try {
    notes.value = await getTauriAPI().getAllNotes()
  } catch { toast.error('加载笔记失败') }
}

async function loadGroups() {
  try {
    noteGroups.value = await getTauriAPI().getNoteGroups()
  } catch { toast.error('加载分组失败') }
}

/* ─── 选中与新建 ─── */
// 切换笔记前把未落盘的修改保存（仅当有防抖 timer 挂着时，即内容真的脏），
// 避免切换后旧 timer 触发、把旧修改写到新笔记上
async function flushSave() {
  if (!saveTimer) { return }
  clearTimeout(saveTimer)
  saveTimer = null
  if (selectedNote.value) { await saveNote() }
}

function selectNote(note: Note) {
  flushSave()
  selectedNote.value = note
  editorTitle.value = note.title
  editorContent.value = note.content
  saveStatus.value = ''
  showRawMd.value = false
  showGroupSelector.value = false
  linkInputVisible.value = false
  nextTick(() => renderRich())
}

async function createNewNote() {
  await flushSave()
  try {
    const gid = groupFilter.value !== 'all' && groupFilter.value !== '__ungrouped__' ? groupFilter.value : null
    const note = await getTauriAPI().addNote({ title: '', content: '', pinned: false, groupId: gid })
    notes.value.unshift(note)
    selectNote(note)
    toast.success('已创建新笔记')
  } catch { toast.error('创建失败') }
}

/* ─── 富文本渲染 / 回写 ─── */
function renderRich() {
  if (!editorRef.value) { return }
  // marked 输出经 DOMPurify 消毒后再注入，避免存储型 XSS（源码模式可写入任意 HTML）
  const html = editorContent.value ? DOMPurify.sanitize(marked.parse(editorContent.value, { async: false }) as string) : ''
  editorRef.value.innerHTML = html
}

function syncRichToMarkdown() {
  if (!editorRef.value || !selectedNote.value) { return }
  editorContent.value = turndownService.turndown(editorRef.value.innerHTML || '')
}

function toggleRawMd() {
  showRawMd.value = !showRawMd.value
  linkInputVisible.value = false
  if (!showRawMd.value) {
    // 切回富文本：重新渲染
    nextTick(() => renderRich())
  }
}

/* ─── 富文本工具栏 ─── */
function execCmd(cmd: string, value?: string) {
  if (!editorRef.value) { return }
  // 仅当选区锚点在编辑器内时才执行，避免失焦后 formatBlock 作用到整个文档
  const sel = window.getSelection()
  if (!sel || !sel.anchorNode || !editorRef.value.contains(sel.anchorNode)) {
    // 编辑器内没有有效选区时，把光标移入再执行
    editorRef.value.focus()
    const range = document.createRange()
    range.selectNodeContents(editorRef.value)
    range.collapse(false)
    sel?.removeAllRanges()
    sel?.addRange(range)
  }
  document.execCommand(cmd, false, value)
  syncRichToMarkdown()
  onContentChange()
}

function formatBlock(tag: string) {
  execCmd('formatBlock', tag)
}

function insertCodeBlock() {
  if (!editorRef.value) { return }
  // 与 execCmd 相同的选区校验：失焦时先移入光标，避免作用到编辑器外
  const sel = window.getSelection()
  if (!sel || !sel.anchorNode || !editorRef.value.contains(sel.anchorNode)) {
    editorRef.value.focus()
    const range = document.createRange()
    range.selectNodeContents(editorRef.value)
    range.collapse(false)
    sel?.removeAllRanges()
    sel?.addRange(range)
  }
  // 选区文本先转义，避免 < & 被当作 HTML 解析破坏 DOM
  const currentSel = window.getSelection()
  const selected = (currentSel ? currentSel.toString() : '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
  document.execCommand('insertHTML', false, `<pre><code>${selected || 'code'}</code></pre>`)
  syncRichToMarkdown()
  onContentChange()
}

function insertLink() {
  const sel = window.getSelection()
  savedLinkRange.value = sel && sel.rangeCount > 0 ? sel.getRangeAt(0).cloneRange() : null
  linkInputVisible.value = true
  linkInputUrl.value = ''
  nextTick(() => linkInputRef.value?.focus())
}

function applyLink() {
  const url = linkInputUrl.value.trim()
  if (!url) { linkInputVisible.value = false; return }
  // 恢复打开输入条前的选区，使 createLink 作用到原选中文本
  if (savedLinkRange.value && editorRef.value) {
    const sel = window.getSelection()
    if (sel) {
      sel.removeAllRanges()
      sel.addRange(savedLinkRange.value)
    }
  }
  execCmd('createLink', url)
  linkInputVisible.value = false
  savedLinkRange.value = null
}

/* ─── 编辑事件 ─── */
function onRichEdit() {
  syncRichToMarkdown()
  onContentChange()
}

function onTitleChange() {
  onContentChange()
}

function onContentChange() {
  if (!selectedNote.value) { return }
  saveStatus.value = '保存中...'
  if (saveTimer) { clearTimeout(saveTimer) }
  saveTimer = setTimeout(() => saveNote(), 500)
}

async function saveNote() {
  if (!selectedNote.value) { return }
  const noteId = selectedNote.value.id
  const savedTitle = editorTitle.value
  const savedContent = editorContent.value
  try {
    const updated = await getTauriAPI().updateNote(noteId, { title: savedTitle, content: savedContent })
    if (updated) {
      // 仅当当前选中仍是该笔记时才更新选中对象，避免异步回调污染已切换的新笔记
      if (selectedNote.value?.id === updated.id) {
        selectedNote.value = { ...selectedNote.value, ...updated }
      }
      const idx = notes.value.findIndex(n => n.id === updated.id)
      if (idx !== -1) { notes.value[idx] = { ...notes.value[idx], ...updated } }
      if (selectedNote.value?.id === updated.id) { saveStatus.value = '已保存' }
    }
  } catch { saveStatus.value = '保存失败'; toast.error('保存失败') }
}

/* ─── 置顶 ─── */
async function togglePin() {
  if (!selectedNote.value) { return }
  try {
    const updated = await getTauriAPI().updateNote(selectedNote.value.id, { pinned: !selectedNote.value.pinned })
    if (updated) {
      selectedNote.value = { ...selectedNote.value, ...updated }
      const idx = notes.value.findIndex(n => n.id === updated.id)
      if (idx !== -1) { notes.value[idx] = { ...notes.value[idx], ...updated } }
    }
  } catch { toast.error('操作失败') }
}

/* ─── 移动分组 ─── */
async function assignGroup(groupId: string | null) {
  if (!selectedNote.value) { return }
  try {
    const updated = await getTauriAPI().updateNote(selectedNote.value.id, { groupId })
    if (updated) {
      selectedNote.value = { ...selectedNote.value, ...updated }
      const idx = notes.value.findIndex(n => n.id === updated.id)
      if (idx !== -1) { notes.value[idx] = { ...notes.value[idx], ...updated } }
    }
    showGroupSelector.value = false
  } catch { toast.error('设置分组失败') }
}

/* ─── 分组管理（下拉内行内操作） ─── */
async function saveCreateGroup() {
  const name = createGroupName.value.trim()
  if (!name) { return }
  try {
    const g = await getTauriAPI().addNoteGroup({ name, icon: '📁' })
    noteGroups.value.push(g)
    groupFilter.value = g.id
    toast.success('分组已创建')
  } catch { toast.error('创建分组失败') }
  creatingGroup.value = false
  createGroupName.value = ''
}

function startRenameGroup(group: NoteGroup) {
  inlineRenameId.value = group.id
  inlineRenameName.value = group.name
}

async function saveRenameGroup(group: NoteGroup) {
  const name = inlineRenameName.value.trim()
  if (!name) { return }
  try {
    const updated = await getTauriAPI().updateNoteGroup(group.id, { name })
    if (updated) {
      const idx = noteGroups.value.findIndex(g => g.id === updated.id)
      if (idx !== -1) { noteGroups.value[idx] = updated }
      toast.success('分组已重命名')
    }
  } catch { toast.error('重命名失败') }
  inlineRenameId.value = null
}

function confirmDeleteGroup(group: NoteGroup) {
  deleteTarget.value = group
}

async function executeDelete() {
  const target = deleteTarget.value
  if (!target) { return }
  if ('title' in target) {
    // 删除笔记
    try {
      await getTauriAPI().deleteNote(target.id)
      notes.value = notes.value.filter(n => n.id !== target.id)
      if (selectedNote.value?.id === target.id) { selectedNote.value = null }
      toast.success('已删除')
    } catch { toast.error('删除失败') }
  } else {
    // 删除分组（组内笔记转未分组）
    try {
      await getTauriAPI().deleteNoteGroup(target.id)
      const notesToUpdate = notes.value.filter(n => n.groupId === target.id)
      for (const note of notesToUpdate) {
        const updated = await getTauriAPI().updateNote(note.id, { groupId: null })
        if (updated) {
          const idx = notes.value.findIndex(n => n.id === updated.id)
          if (idx !== -1) { notes.value[idx] = updated }
        }
      }
      if (groupFilter.value === target.id) { groupFilter.value = 'all' }
      if (selectedNote.value?.groupId === target.id) { selectedNote.value = { ...selectedNote.value, groupId: null } }
      noteGroups.value = noteGroups.value.filter(g => g.id !== target.id)
      toast.success('分组已删除')
    } catch {
      // 分组已删但部分笔记未迁移时回滚：重新拉取最新数据恢复一致状态
      toast.error('分组删除不完整，已刷新数据')
      await loadGroups()
      await loadNotes()
    }
  }
  deleteTarget.value = null
}

function confirmDelete(note: Note) {
  deleteTarget.value = note
}

/* ─── 外部点击关闭下拉 ─── */
function onDocumentClick(e: MouseEvent) {
  const target = e.target as HTMLElement
  if (showGroupFilter.value && groupFilterRef.value && !groupFilterRef.value.contains(target)) {
    showGroupFilter.value = false
  }
  if (showGroupSelector.value && moveGroupRef.value && !moveGroupRef.value.contains(target)) {
    showGroupSelector.value = false
  }
}

onMounted(() => {
  loadGroups()
  loadNotes()
  document.addEventListener('click', onDocumentClick)
})
onUnmounted(() => {
  document.removeEventListener('click', onDocumentClick)
  if (saveTimer) { clearTimeout(saveTimer); saveTimer = null }
  // 组件卸载前同步保存一次未落盘的修改（fire-and-forget）
  if (selectedNote.value) { saveNote() }
})
</script>

<style>
.toolbar-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  min-width: 28px;
  height: 26px;
  padding: 0 6px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--color-base-content, currentColor);
  opacity: 0.75;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
  white-space: nowrap;
}
.toolbar-btn:hover {
  opacity: 1;
  background: var(--color-base-200, rgba(0,0,0,0.06));
}
.toolbar-divider {
  width: 1px;
  height: 16px;
  background: color-mix(in oklab, var(--color-base-content) 12%, transparent);
  margin: 0 6px;
  flex-shrink: 0;
}

/* 富文本正文样式 */
.markdown-body { line-height: 1.7; font-size: 14px; color: var(--color-base-content); }
.markdown-body:empty::before {
  content: attr(data-placeholder);
  color: color-mix(in oklab, var(--color-base-content) 35%, transparent);
  pointer-events: none;
}
.markdown-body h1 { font-size: 24px; font-weight: 700; margin: 18px 0 10px; border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); padding-bottom: 6px; }
.markdown-body h2 { font-size: 20px; font-weight: 600; margin: 16px 0 8px; }
.markdown-body h3 { font-size: 17px; font-weight: 600; margin: 14px 0 6px; }
.markdown-body p { margin: 8px 0; }
.markdown-body code { background: var(--color-base-200); padding: 2px 6px; border-radius: 4px; font-family: 'SF Mono', ui-monospace, monospace; font-size: 13px; }
.markdown-body pre { background: var(--color-base-200); padding: 14px; border-radius: 8px; overflow-x: auto; margin: 12px 0; }
.markdown-body pre code { background: none; padding: 0; }
.markdown-body blockquote { border-left: 3px solid var(--color-primary); padding-left: 14px; margin: 12px 0; color: color-mix(in oklab, var(--color-base-content) 65%, transparent); }
.markdown-body ul, .markdown-body ol { padding-left: 24px; margin: 8px 0; }
.markdown-body li { margin: 4px 0; }
.markdown-body a { color: var(--color-primary); text-decoration: underline; }
.markdown-body img { max-width: 100%; border-radius: 8px; margin: 12px 0; }
.markdown-body table { border-collapse: collapse; width: 100%; margin: 12px 0; }
.markdown-body th, .markdown-body td { border: 1px solid color-mix(in oklab, var(--color-base-content) 12%, transparent); padding: 8px 12px; text-align: left; }
.markdown-body th { background: var(--color-base-200); font-weight: 600; }
</style>
