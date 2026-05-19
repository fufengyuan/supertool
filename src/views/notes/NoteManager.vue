<template>
  <div class="w-full flex flex-col gap-3 h-full p-3">
    <div class="flex items-center justify-between px-4 py-3 bg-base-100 border border-base-content/10 rounded-xl shrink-0">
      <div class="flex items-center gap-3">
        <div class="w-8 h-8 flex items-center justify-center text-base bg-base-200 rounded-lg"><SvgIcon name="file" size="14" class="inline-block align-text-bottom" /></div>
        <div class="flex flex-col gap-0.5">
          <h2 class="text-base font-bold text-base-content m-0">笔记</h2>
          <p class="text-xs text-base-content/60 m-0">记录工作与生活的灵感</p>
        </div>
      </div>
      <div class="flex items-center gap-2">
        <div class="relative flex items-center text-base-content/60">
          <SvgIcon name="search" size="14" class="absolute left-2.5 pointer-events-none" />
          <input v-model="searchQuery" class="input input-bordered rounded-full px-3 py-1.5 pl-8 text-xs w-[180px] transition-all duration-200 focus:w-[220px]" placeholder="搜索笔记..." @input="onSearch"/>
        </div>
        <button class="btn btn-primary btn-sm gap-1.5" @click="createNewNote">
          <SvgIcon name="plus" size="14" />
          新建笔记
        </button>
      </div>
    </div>
    <div class="flex gap-3 flex-1 min-h-0">
      <aside class="w-64 min-w-[220px] bg-base-100 rounded-xl border border-base-content/10 flex flex-col shrink-0">
        <!-- 搜索 + 新建 -->
        <div class="p-2 border-b border-base-content/10 flex gap-1.5">
          <div class="relative flex-1">
            <SvgIcon name="search" size="12" class="absolute left-2 top-1/2 -translate-y-1/2 pointer-events-none text-base-content/40" />
            <input v-model="searchQuery" class="input input-xs w-full pl-6 rounded-lg" placeholder="搜索笔记..." @input="onSearch"/>
          </div>
          <button class="btn btn-primary btn-xs btn-square" @click="createNewNote" title="新建笔记"><SvgIcon name="plus" size="12" /></button>
        </div>
        <!-- 树状笔记列表 -->
        <div class="flex-1 overflow-y-auto p-1.5 space-y-0.5">
          <!-- 未分组 -->
          <div>
            <div :class="['flex items-center gap-1 px-2 py-1 rounded-md cursor-pointer text-xs font-medium text-base-content/60 hover:bg-base-200 transition-colors', { 'bg-primary/10 text-primary': selectedGroupId === '__ungrouped__' }]" @click="selectGroup('__ungrouped__')">
              <SvgIcon name="file" size="12" class="shrink-0" />
              <span class="flex-1 truncate">未分组</span>
              <span class="text-[10px] text-base-content/40">{{ ungroupedNotes.length }}</span>
            </div>
            <div v-if="selectedGroupId === '__ungrouped__' || expandedGroups.has('__ungrouped__')" class="ml-1">
              <div v-for="note in ungroupedNotes" :key="note.id" :class="['flex items-center gap-1.5 pl-5 pr-2 py-1 rounded-md cursor-pointer text-xs transition-colors hover:bg-base-200 group/tree', { 'bg-primary/10 text-primary font-medium': selectedNote?.id === note.id }]" @click="selectNote(note)">
                <span class="text-[10px] text-base-content/30 shrink-0">{{ note.pinned ? '★' : '' }}</span>
                <span class="flex-1 truncate" :title="note.title || '无标题'">{{ note.title || '无标题' }}</span>
                <button class="shrink-0 opacity-0 group-hover/tree:opacity-100 text-[10px] text-base-content/30 hover:text-error transition-all" @click.stop="confirmDelete(note)" title="删除">✕</button>
              </div>
            </div>
          </div>
          <!-- 分组树 -->
          <div v-for="group in noteGroups" :key="group.id">
            <div :class="['flex items-center gap-1 px-2 py-1 rounded-md cursor-pointer text-xs font-medium hover:bg-base-200 transition-colors', { 'bg-primary/10 text-primary': selectedGroupId === group.id }]" @click="toggleGroupExpand(group.id)">
              <SvgIcon :name="expandedGroups.has(group.id) ? 'chevronDown' : 'chevronRight'" size="10" class="shrink-0 text-base-content/30" />
              <span class="text-sm shrink-0">{{ group.icon || '📁' }}</span>
              <span class="flex-1 truncate">{{ group.name }}</span>
              <span class="text-[10px] text-base-content/40">{{ getGroupNoteCount(group.id) }}</span>
            </div>
            <div v-if="expandedGroups.has(group.id)" class="ml-1">
              <div v-for="note in getGroupNotes(group.id)" :key="note.id" :class="['flex items-center gap-1.5 pl-5 pr-2 py-1 rounded-md cursor-pointer text-xs transition-colors hover:bg-base-200 group/tree', { 'bg-primary/10 text-primary font-medium': selectedNote?.id === note.id }]" @click="selectNote(note)">
                <span class="text-[10px] text-base-content/30 shrink-0">{{ note.pinned ? '★' : '' }}</span>
                <span class="flex-1 truncate" :title="note.title || '无标题'">{{ note.title || '无标题' }}</span>
                <button class="shrink-0 opacity-0 group-hover/tree:opacity-100 text-[10px] text-base-content/30 hover:text-error transition-all" @click.stop="confirmDelete(note)" title="删除">✕</button>
              </div>
            </div>
          </div>
          <!-- 空状态 -->
          <div v-if="notes.length === 0" class="text-center py-8 text-xs text-base-content/30">
            <p>暂无笔记</p>
            <p class="mt-1">点击 + 新建</p>
          </div>
        </div>
        <!-- 底部操作 -->
        <div class="border-t border-base-content/10 p-1.5 flex gap-1.5">
          <button class="btn btn-ghost btn-xs flex-1 gap-1" @click="showCreateGroup = true"><SvgIcon name="folder" size="11" /> 新建分组</button>
          <button class="btn btn-ghost btn-xs btn-square" @click="showGroupManager = true" title="管理分组"><SvgIcon name="settings" size="11" /></button>
        </div>
      </aside>
      <main class="flex-1 min-w-0 bg-base-100 rounded-xl border border-base-content/10 flex flex-col overflow-hidden">
        <template v-if="selectedNote">
          <div class="flex items-center justify-between px-4 py-2 border-b border-base-content/10 shrink-0">
            <div class="flex items-center gap-2">
              <button :class="['px-2.5 py-1.5 text-xs font-medium border border-base-content/10 rounded-lg bg-transparent text-base-content/60 cursor-pointer transition-all duration-150 whitespace-nowrap hover:bg-base-200 hover:text-base-content', { '!bg-primary !text-white !border-primary hover:!bg-primary hover:!text-white': showRawMd }]" @click="showRawMd = !showRawMd" :title="showRawMd ? '预览' : '编辑 Markdown'"><template v-if="showRawMd"><SvgIcon name="eye" size="14" class="inline-block align-text-bottom" /> 预览</template><template v-else><SvgIcon name="file" size="14" class="inline-block" /> Markdown</template></button>
              <button :class="['px-2.5 py-1.5 text-xs font-medium border border-base-content/10 rounded-lg bg-transparent text-base-content/60 cursor-pointer transition-all duration-150 whitespace-nowrap hover:bg-base-200 hover:text-base-content', { '!bg-primary !text-white !border-primary hover:!bg-primary hover:!text-white': selectedNote.pinned }]" @click="togglePin" title="置顶"><SvgIcon name="star" size="14" class="inline-block align-text-bottom" /> 置顶</button>
              <div class="relative note-group-selector">
                <button class="px-2.5 py-1.5 text-xs font-medium border border-base-content/10 rounded-lg bg-transparent text-base-content/60 cursor-pointer transition-all duration-150 whitespace-nowrap hover:bg-base-200 hover:text-base-content flex items-center gap-1" @click="toggleGroupSelector" title="修改分组"><SvgIcon name="folder" size="14" class="inline-block align-text-bottom" /> {{ getGroupName(selectedNote.groupId) || '分组' }}</button>
                <div v-if="showGroupSelector" class="absolute top-full left-0 mt-1 bg-base-100 border border-base-content/10 rounded-lg shadow-lg z-[100] min-w-[160px] max-h-60 overflow-y-auto p-1" @click.stop>
                  <div :class="['flex items-center gap-2 px-3 py-1.5 text-sm cursor-pointer rounded-lg transition-colors duration-100 hover:bg-base-200', { 'bg-primary/10 text-primary font-medium': !selectedNote.groupId }]" @click="assignGroup(null)"><SvgIcon name="file" size="14" class="inline-block align-text-bottom" /> 未分组</div>
                  <div v-for="group in noteGroups" :key="group.id" :class="['flex items-center gap-2 px-3 py-1.5 text-sm cursor-pointer rounded-lg transition-colors duration-100 hover:bg-base-200', { 'bg-primary/10 text-primary font-medium': selectedNote.groupId === group.id }]" @click="assignGroup(group.id)"><SvgIcon name="folder" size="14" class="inline-block align-text-bottom" /> {{ group.name }}</div>
                </div>
              </div>
            </div>
            <div class="flex items-center gap-2"><span class="text-xs text-base-content/60">{{ saveStatus }}</span></div>
          </div>
          <div class="px-4 pt-3 pb-1 shrink-0">
            <input v-model="editorTitle" class="input w-full text-xl font-bold text-base-content border-none bg-transparent px-0 focus:outline-none placeholder:text-base-content/60 placeholder:opacity-40" placeholder="输入标题..." @input="onTitleChange"/>
          </div>
          <div :class="['flex-1 min-h-0 px-4 pb-4 pt-2', { 'hidden': !showRawMd }]">
            <textarea v-model="editorContent" class="w-full h-full min-h-[300px] text-sm leading-relaxed text-base-content bg-transparent border-none resize-none font-mono focus:outline-none placeholder:text-base-content/60 placeholder:opacity-40" placeholder="开始用 Markdown 编写笔记..." @input="onContentChange" spellcheck="false"></textarea>
          </div>
          <div
            :class="['editor-preview flex-1 min-h-0 px-4 pb-4 pt-2 overflow-y-auto leading-relaxed text-base-content', { 'hidden': showRawMd }]"
            ref="previewRef"
            contenteditable="true"
            @input="onPreviewEdit"
          ></div>
        </template>
        <div v-else class="flex-1 flex flex-col items-center justify-center text-base-content/60 opacity-50">
          <SvgIcon name="file" size="48" :strokeWidth="1.5" class="mb-3" />
          <p class="text-sm">选择或创建一个笔记</p>
        </div>
      </main>
    </div>
    <Teleport to="body">
      <div v-if="deleteTarget" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[10000]" @click.self="deleteTarget = null">
        <div class="bg-base-100 rounded-2xl p-5 max-w-sm w-[90%] shadow-[0_16px_48px_rgba(0,0,0,0.3)]" @click.stop>
          <h3 class="text-lg font-bold m-0 mb-2 text-base-content"><SvgIcon name="alertTriangle" size="14" class="inline-block align-text-bottom" /> 确认删除</h3>
          <p class="text-sm text-base-content/60 m-0 mb-4">确定要删除「{{ deleteTarget.title || '无标题' }}」吗？此操作不可撤销。</p>
          <div class="flex justify-end gap-2">
            <button class="btn btn-ghost" @click="deleteTarget = null">取消</button>
            <button class="btn btn-error" @click="executeDelete">删除</button>
          </div>
        </div>
      </div>
    </Teleport>
    <Teleport to="body">
      <div v-if="showCreateGroup || editingGroup" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[10000]" @click.self="cancelGroupEdit">
        <div class="bg-base-100 rounded-2xl p-5 max-w-sm w-[90%] shadow-[0_16px_48px_rgba(0,0,0,0.3)]" @click.stop>
          <h3 class="text-lg font-bold m-0 mb-3 text-base-content"><template v-if="editingGroup"><SvgIcon name="pencil" size="14" class="inline-block" /> 重命名分组</template><template v-else><SvgIcon name="folder" size="14" class="inline-block align-text-bottom" /> 新建分组</template></h3>
          <div class="mb-4">
            <div class="mb-3"><label class="block text-xs font-semibold text-base-content/60 mb-1">名称</label><input v-model="groupForm.name" class="input input-bordered w-full px-3 py-2 text-sm rounded-lg bg-base-200 text-base-content focus:outline-none focus:border-primary focus:shadow-[0_0_0_3px_rgba(66,133,244,0.15)]" placeholder="分组名称" @keyup.enter="saveGroup" ref="groupInputRef"/></div>
            <div><label class="block text-xs font-semibold text-base-content/60 mb-1">图标</label><div class="flex flex-wrap gap-1"><button v-for="icon in iconOptions" :key="icon" :class="['w-7 h-7 flex items-center justify-center text-sm border-2 border-transparent rounded-lg bg-transparent cursor-pointer transition-all duration-150 hover:bg-base-200', { 'border-primary bg-[rgba(66,133,244,0.1)]': groupForm.icon === icon }]" @click="groupForm.icon = icon">{{ icon }}</button></div></div>
          </div>
          <div class="flex justify-end gap-2">
            <button class="btn btn-ghost" @click="cancelGroupEdit">取消</button>
            <button class="btn btn-primary" @click="saveGroup" :disabled="!groupForm.name.trim()">保存</button>
          </div>
        </div>
      </div>
    </Teleport>
    <Teleport to="body">
      <div v-if="deleteGroupTarget" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[10000]" @click.self="deleteGroupTarget = null">
        <div class="bg-base-100 rounded-2xl p-5 max-w-sm w-[90%] shadow-[0_16px_48px_rgba(0,0,0,0.3)]" @click.stop>
          <h3 class="text-lg font-bold m-0 mb-2 text-base-content"><SvgIcon name="alertTriangle" size="14" class="inline-block align-text-bottom" /> 删除分组</h3>
          <p class="text-sm text-base-content/60 m-0 mb-4">确定要删除分组「{{ deleteGroupTarget.name }}」吗？分组内的笔记不会被删除，将变为未分组。</p>
          <div class="flex justify-end gap-2">
            <button class="btn btn-ghost" @click="deleteGroupTarget = null">取消</button>
            <button class="btn btn-error" @click="executeDeleteGroup">删除</button>
          </div>
        </div>
      </div>
    </Teleport>
    <Teleport to="body">
      <div v-if="showGroupManager" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[10000]" @click.self="showGroupManager = false">
        <div class="bg-base-100 rounded-2xl p-5 w-[420px] max-w-[90vw] max-h-[70vh] shadow-[0_16px_48px_rgba(0,0,0,0.3)] flex flex-col" @click.stop>
          <div class="flex items-center justify-between mb-4">
            <h3 class="text-lg font-bold m-0 text-base-content"><SvgIcon name="folder" size="14" class="inline-block align-text-bottom" /> 管理分组</h3>
            <button class="btn btn-ghost btn-xs btn-square" @click="showGroupManager = false"><SvgIcon name="close" size="14" /></button>
          </div>
          <div class="flex-1 overflow-y-auto space-y-1 min-h-0">
            <div v-for="group in noteGroups" :key="group.id" class="flex items-center gap-2 px-3 py-2 rounded-lg hover:bg-base-200 group/row">
              <div class="flex items-center gap-2 flex-1 min-w-0">
                <template v-if="editingGroupId === group.id">
                  <input v-model="editingGroupName" class="input input-bordered input-xs flex-1" @keyup.enter="saveGroupRename(group)" @keyup.escape="editingGroupId = null" ref="renameInputRef" />
                  <button class="btn btn-ghost btn-xs" @click="saveGroupRename(group)">保存</button>
                  <button class="btn btn-ghost btn-xs" @click="editingGroupId = null">取消</button>
                </template>
                <template v-else>
                  <span class="text-sm shrink-0">{{ group.icon || '📁' }}</span>
                  <span class="text-sm text-base-content flex-1 truncate">{{ group.name }}</span>
                  <span class="text-xs text-base-content/40 shrink-0">{{ getGroupNoteCount(group.id) }} 篇</span>
                </template>
              </div>
              <div v-if="editingGroupId !== group.id" class="flex gap-1 opacity-0 group-hover/row:opacity-100 transition-opacity">
                <button class="w-6 h-6 flex items-center justify-center border-none rounded bg-transparent cursor-pointer text-xs p-0 hover:bg-base-300" @click="startRenameGroupInline(group)" title="重命名"><SvgIcon name="pencil" size="12" /></button>
                <button class="w-6 h-6 flex items-center justify-center border-none rounded bg-transparent cursor-pointer text-xs p-0 hover:bg-error/20 hover:text-error" @click="confirmDeleteGroup(group)" title="删除"><SvgIcon name="trash" size="12" /></button>
              </div>
            </div>
            <div v-if="noteGroups.length === 0" class="text-center py-6 text-sm text-base-content/40">暂无分组</div>
          </div>
          <div class="border-t border-base-content/10 pt-3 mt-3">
            <button class="btn btn-primary btn-sm w-full gap-1.5" @click="showCreateGroup = true; showGroupManager = false">
              <SvgIcon name="plus" size="14" /> 新建分组
            </button>
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
const selectedGroupId = ref<string | null>(null)
const showRawMd = ref(false)
const editorTitle = ref('')
const editorContent = ref('')
const saveStatus = ref('')
const deleteTarget = ref<Note | null>(null)
const showGroupSelector = ref(false)
const previewRef = ref<HTMLDivElement | null>(null)
let saveTimer: ReturnType<typeof setTimeout> | null = null
let isUpdatingPreview = false

// 树状展开状态
const expandedGroups = ref<Set<string>>(new Set())
function toggleGroupExpand(groupId: string) {
  if (expandedGroups.value.has(groupId)) expandedGroups.value.delete(groupId)
  else expandedGroups.value.add(groupId)
}

// 未分组笔记（支持搜索过滤）
const ungroupedNotes = computed(() => {
  let result = notes.value.filter(n => !n.groupId)
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase()
    result = result.filter(n => n.title.toLowerCase().includes(q) || n.content.toLowerCase().includes(q))
  }
  return result
})
// 某分组下的笔记（支持搜索过滤）
function getGroupNotes(groupId: string): Note[] {
  let result = notes.value.filter(n => n.groupId === groupId)
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase()
    result = result.filter(n => n.title.toLowerCase().includes(q) || n.content.toLowerCase().includes(q))
  }
  return result
}

const showCreateGroup = ref(false)
const editingGroup = ref<NoteGroup | null>(null)
const deleteGroupTarget = ref<NoteGroup | null>(null)
const groupForm = ref({ name: '', icon: '📁' })
const groupInputRef = ref<HTMLInputElement | null>(null)
const iconOptions = ['📁', '📂', '📋', '📌', '🏷️', '💼', '🎯', '📚', '🔧', '💡', '🌟', '📝', '🗂️', '🏠', '🎨', '⚙️']

// 分组管理弹窗
const showGroupManager = ref(false)
const editingGroupId = ref<string | null>(null)
const editingGroupName = ref('')
const renameInputRef = ref<HTMLInputElement | null>(null)

const renderedContent = computed(() => {
  if (!editorContent.value) return '<p class="preview-empty">没有内容</p>'
  let html = marked.parse(editorContent.value, { async: false }) as string
  if (searchQuery.value) {
    const regex = new RegExp(`(${escapeRegex(searchQuery.value)})`, 'gi')
    html = html.replace(regex, '<mark class="search-highlight">$1</mark>')
  }
  return html
})

function escapeRegex(str: string): string { return str.replace(/[.*+?${}()|[\\]\\\\]/g, '\\\\$&') }

function getPreview(content: string): string {
  if (!content) return '空笔记'
  const plain = content.replace(/[#*`>-\\[\\]()!]/g, '').trim()
  return plain.slice(0, 80) + (plain.length > 80 ? '...' : '')
}

function highlightText(text: string): string {
  if (!searchQuery.value || !text) return text
  const regex = new RegExp(`(${escapeRegex(searchQuery.value)})`, 'gi')
  return text.replace(regex, '<mark>$1</mark>')
}

function formatDate(iso: string): string {
  if (!iso) return ''
  const d = new Date(iso), now = new Date(), diff = now.getTime() - d.getTime(), mins = Math.floor(diff / 60000)
  if (mins < 1) return '刚刚'
  if (mins < 60) return `${mins} 分钟前`
  const hours = Math.floor(mins / 60)
  if (hours < 24) return `${hours} 小时前`
  return d.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' })
}

function getGroupName(groupId: string | null | undefined): string {
  if (!groupId) return ''
  const group = noteGroups.value.find(g => g.id === groupId)
  return group ? group.name : ''
}

function renderPreview() {
  if (!previewRef.value) return
  const html = renderedContent.value
  previewRef.value.innerHTML = html
}

// 选中笔记时渲染预览
watch(selectedNote, () => {
  if (!showRawMd.value) {
    nextTick(() => renderPreview())
  }
})

// 切回预览模式时同步最新内容
watch(showRawMd, (val) => {
  if (!val) {
    renderPreview()
  }
})

async function loadNotes() {
  try {
    notes.value = await getTauriAPI().getAllNotes(searchQuery.value || undefined)
  } catch { toast.error('加载笔记失败') }
}

async function loadGroups() {
  try {
    noteGroups.value = await getTauriAPI().getNoteGroups() } catch { toast.error('加载分组失败') }
}

function onSearch() { loadNotes() }

function selectGroup(groupId: string) {
    selectedGroupId.value = groupId
  // 自动展开该分组
  if (groupId !== '__ungrouped__') expandedGroups.value.add(groupId)
}

function selectNote(note: Note) {
    selectedNote.value = note; editorTitle.value = note.title; editorContent.value = note.content
  showRawMd.value = false; saveStatus.value = ''; showGroupSelector.value = false
  // 若笔记有分组且已折叠，自动展开
  if (note.groupId) expandedGroups.value.add(note.groupId)
  nextTick(() => renderPreview())
}

async function createNewNote() {
    try {
    const gid = selectedGroupId.value && selectedGroupId.value !== '__ungrouped__' ? selectedGroupId.value : null
    const note = await getTauriAPI().addNote({ title: '', content: '', pinned: false, groupId: gid })
    notes.value.unshift(note); selectNote(note); toast.success('已创建新笔记')
    // 自动展开对应分组
    if (gid) expandedGroups.value.add(gid)
  } catch { toast.error('创建失败') }
}

function onTitleChange() {
  if (!selectedNote.value) return
  saveStatus.value = '保存中...'; if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(() => saveNote(), 500)
}

// 预览区始终在 DOM 中（CSS hidden 切换），ref 稳定可用
// 在 onContentChange（源码模式编辑）时同步更新预览
function onContentChange() {
    if (!selectedNote.value) return
  // 同步更新预览区
  if (previewRef.value && !showRawMd.value) {
    renderPreview()
  }
  saveStatus.value = '保存中...'; if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(() => saveNote(), 500)
}

async function saveNote() {
    if (!selectedNote.value) return
  try {
    const updated = await getTauriAPI().updateNote(selectedNote.value.id, { title: editorTitle.value, content: editorContent.value })
    if (updated) {
      selectedNote.value = { ...selectedNote.value, ...updated }
      const idx = notes.value.findIndex(n => n.id === updated.id)
      if (idx !== -1) notes.value[idx] = { ...notes.value[idx], ...updated }
      saveStatus.value = '已保存'
    }
  } catch { saveStatus.value = '保存失败'; toast.error('保存失败') }
}

async function togglePin() {
  if (!selectedNote.value) return
  try {
    const updated = await getTauriAPI().updateNote(selectedNote.value.id, { pinned: !selectedNote.value.pinned })
    if (updated) {
      selectedNote.value = { ...selectedNote.value, ...updated }
      const idx = notes.value.findIndex(n => n.id === updated.id)
      if (idx !== -1) notes.value[idx] = { ...notes.value[idx], ...updated }
    }
  } catch { toast.error('操作失败') }
}

function toggleGroupSelector() {
  showGroupSelector.value = !showGroupSelector.value
}

// 点击外部关闭分组选择器
function onDocumentClick(e: MouseEvent) {
  if (showGroupSelector.value) {
    const target = e.target as HTMLElement
    if (!target.closest('.note-group-selector')) {
      showGroupSelector.value = false
    }
  }
}

// 内容可编辑预览 → 通过 turndown 转回 Markdown
function onPreviewEdit() {
  if (!selectedNote.value || !previewRef.value || isUpdatingPreview) return
  isUpdatingPreview = true
  try {
    const html = previewRef.value.innerHTML
    // 用 turndown 转回 Markdown
    const md = turndownService.turndown(html)
    editorContent.value = md
    saveStatus.value = '保存中...'
    if (saveTimer) clearTimeout(saveTimer)
    saveTimer = setTimeout(() => saveNote(), 500)
  } finally {
    isUpdatingPreview = false
  }
}

async function assignGroup(groupId: string | null) {
  if (!selectedNote.value) return
  try {
    const updated = await getTauriAPI().updateNote(selectedNote.value.id, { groupId })
    if (updated) {
      selectedNote.value = { ...selectedNote.value, ...updated }
      const idx = notes.value.findIndex(n => n.id === updated.id)
      if (idx !== -1) notes.value[idx] = { ...notes.value[idx], ...updated }
      // 切换到新分组并展开
      if (groupId) {
        selectedGroupId.value = groupId
        expandedGroups.value.add(groupId)
      } else {
        selectedGroupId.value = '__ungrouped__'
      }
    }
    showGroupSelector.value = false
  } catch { toast.error('设置分组失败') }
}

function confirmDelete(note: Note) { deleteTarget.value = note }

async function executeDelete() {
    if (!deleteTarget.value) return
  try {
    await getTauriAPI().deleteNote(deleteTarget.value.id)
    notes.value = notes.value.filter(n => n.id !== deleteTarget.value!.id)
    if (selectedNote.value?.id === deleteTarget.value.id) selectedNote.value = null
    toast.success('已删除')
  } catch { toast.error('删除失败') }
  deleteTarget.value = null
}

function startRenameGroup(group: NoteGroup) {
  editingGroup.value = group
  groupForm.value = { name: group.name, icon: group.icon || '📁' }
  nextTick(() => { groupInputRef.value?.focus() })
}

// 分组管理弹窗 - 内联重命名
function startRenameGroupInline(group: NoteGroup) {
  editingGroupId.value = group.id
  editingGroupName.value = group.name
  nextTick(() => { renameInputRef.value?.focus() })
}

async function saveGroupRename(group: NoteGroup) {
  if (!editingGroupName.value.trim()) return
  try {
    const updated = await getTauriAPI().updateNoteGroup(group.id, { name: editingGroupName.value.trim() })
    if (updated) {
      const idx = noteGroups.value.findIndex(g => g.id === updated.id)
      if (idx !== -1) noteGroups.value[idx] = { ...noteGroups.value[idx], ...updated }
      toast.success('分组已重命名')
    }
    editingGroupId.value = null
  } catch { toast.error('重命名失败') }
}

function getGroupNoteCount(groupId: string): number {
  return notes.value.filter(n => n.groupId === groupId).length
}

function confirmDeleteGroup(group: NoteGroup) { deleteGroupTarget.value = group }

async function executeDeleteGroup() {
    if (!deleteGroupTarget.value) return
  try {
    await getTauriAPI().deleteNoteGroup(deleteGroupTarget.value.id)
    const notesToUpdate = notes.value.filter(n => n.groupId === deleteGroupTarget.value!.id)
    for (const note of notesToUpdate) {
      const updated = await getTauriAPI().updateNote(note.id, { groupId: null })
      if (updated) { const idx = notes.value.findIndex(n => n.id === updated.id); if (idx !== -1) notes.value[idx] = updated }
    }
    if (selectedGroupId.value === deleteGroupTarget.value.id) selectedGroupId.value = null
    if (selectedNote.value?.groupId === deleteGroupTarget.value.id) selectedNote.value = { ...selectedNote.value, groupId: null }
    noteGroups.value = noteGroups.value.filter(g => g.id !== deleteGroupTarget.value!.id)
    toast.success('分组已删除')
  } catch { toast.error('删除分组失败') }
  deleteGroupTarget.value = null
}

function cancelGroupEdit() { showCreateGroup.value = false; editingGroup.value = null; groupForm.value = { name: '', icon: '📁' } }

async function saveGroup() {
  if (!groupForm.value.name.trim()) return
  try {
    if (editingGroup.value) {
      const updated = await getTauriAPI().updateNoteGroup(editingGroup.value.id, { name: groupForm.value.name.trim(), icon: groupForm.value.icon })
      if (updated) { const idx = noteGroups.value.findIndex(g => g.id === updated.id); if (idx !== -1) noteGroups.value[idx] = updated }
      toast.success('分组已更新')
    } else {
      const newGroup = await getTauriAPI().addNoteGroup({ name: groupForm.value.name.trim(), icon: groupForm.value.icon })
      noteGroups.value.push(newGroup)
      toast.success('分组已创建')
    }
    cancelGroupEdit()
  } catch { toast.error(editingGroup.value ? '更新分组失败' : '创建分组失败') }
}

onMounted(() => {
    loadGroups(); loadNotes(); document.addEventListener('click', onDocumentClick) })
onUnmounted(() => { document.removeEventListener('click', onDocumentClick) })
</script>

<style>
.editor-preview h1 { font-size: 28px; font-weight: 700; margin: 24px 0 12px; border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); padding-bottom: 8px; }
.editor-preview h2 { font-size: 22px; font-weight: 600; margin: 20px 0 10px; }
.editor-preview h3 { font-size: 18px; font-weight: 600; margin: 16px 0 8px; }
.editor-preview p { margin: 8px 0; }
.editor-preview code { background: var(--color-base-200); padding: 2px 6px; border-radius: 4px; font-family: 'SF Mono', monospace; font-size: 13px; }
.editor-preview pre { background: var(--color-base-200); padding: 16px; border-radius: 8px; overflow-x: auto; margin: 12px 0; }
.editor-preview pre code { background: none; padding: 0; }
.editor-preview blockquote { border-left: 3px solid var(--color-primary); padding-left: 16px; margin: 12px 0; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); }
.editor-preview ul, .editor-preview ol { padding-left: 24px; margin: 8px 0; }
.editor-preview li { margin: 4px 0; }
.editor-preview a { color: var(--color-primary); text-decoration: underline; }
.editor-preview img { max-width: 100%; border-radius: 8px; margin: 12px 0; }
.editor-preview table { border-collapse: collapse; width: 100%; margin: 12px 0; }
.editor-preview th, .editor-preview td { border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); padding: 8px 12px; text-align: left; }
.editor-preview th { background: var(--color-base-200); font-weight: 600; }
.editor-preview .search-highlight { background: rgba(251, 191, 36, 0.35); color: var(--color-base-content); border-radius: 2px; padding: 0 2px; }
.editor-preview .preview-empty { color: color-mix(in oklab, var(--color-base-content) 60%, transparent); opacity: 0.5; }
.note-list-item mark { background: rgba(251, 191, 36, 0.35); color: var(--color-base-content); border-radius: 2px; padding: 0 2px; }
</style>
