<template>
  <div class="note-manager">
    <div class="note-header">
      <div class="note-header-left">
        <div class="header-icon">📝</div>
        <div class="header-info">
          <h2 class="note-title">笔记</h2>
          <p class="note-subtitle">记录工作与生活的灵感</p>
        </div>
      </div>
      <div class="note-header-actions">
        <div class="search-wrapper">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
          <input v-model="searchQuery" class="search-input" placeholder="搜索笔记..." @input="onSearch"/>
        </div>
        <button class="btn-new" @click="createNewNote">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
          新建笔记
        </button>
      </div>
    </div>
    <div class="note-layout">
      <aside class="note-sidebar">
        <div class="group-section">
          <div class="group-header" @click="groupsCollapsed = !groupsCollapsed">
            <span class="group-header-title">📁 分组</span>
            <svg class="group-chevron" :class="{ collapsed: groupsCollapsed }" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"/></svg>
          </div>
          <div v-show="!groupsCollapsed" class="group-list">
            <div class="group-item" :class="{ active: selectedGroupId === '__all__' }" @click="selectGroup('__all__')">
              <span class="group-icon">📋</span><span class="group-name">全部</span>
            </div>
            <div class="group-item" :class="{ active: selectedGroupId === '__ungrouped__' }" @click="selectGroup('__ungrouped__')">
              <span class="group-icon">📄</span><span class="group-name">未分组</span>
            </div>
            <div v-for="group in noteGroups" :key="group.id" class="group-item" :class="{ active: selectedGroupId === group.id }" @click="selectGroup(group.id)">
              <span class="group-icon">{{ group.icon || '📁' }}</span>
              <span class="group-name">{{ group.name }}</span>
              <div class="group-actions" @click.stop>
                <button class="group-action-btn" @click="startRenameGroup(group)" title="重命名">✏️</button>
                <button class="group-action-btn group-action-delete" @click="confirmDeleteGroup(group)" title="删除">🗑️</button>
              </div>
            </div>
            <div class="group-add" @click="showCreateGroup = true">
              <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
              <span>新建分组</span>
            </div>
          </div>
        </div>
        <div v-if="filteredNotes.length === 0" class="sidebar-empty">
          <p>{{ searchQuery ? '没有匹配的笔记' : '暂无笔记' }}</p>
        </div>
        <div v-else class="note-list">
          <div v-for="note in filteredNotes" :key="note.id" class="note-list-item" :class="{ active: selectedNote?.id === note.id, pinned: note.pinned }" @click="selectNote(note)">
            <div class="note-item-top">
              <div class="note-item-title">
                <span v-if="note.pinned" class="pin-icon">📌</span>
                <span class="title-text" v-html="highlightText(note.title || '无标题')"></span>
              </div>
              <button class="note-item-delete" @click.stop="confirmDelete(note)" title="删除">
                <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
              </button>
            </div>
            <div class="note-item-preview" v-html="highlightText(getPreview(note.content))"></div>
            <div class="note-item-meta">
              <span class="note-item-date">{{ formatDate(note.updatedAt) }}</span>
              <span v-if="getGroupName(note.groupId)" class="note-item-group">{{ getGroupName(note.groupId) }}</span>
            </div>
          </div>
        </div>
      </aside>
      <main class="note-editor">
        <template v-if="selectedNote">
          <div class="editor-toolbar">
            <div class="toolbar-left">
              <button class="toolbar-btn" :class="{ active: editMode }" @click="editMode = !editMode" :title="editMode ? '预览' : '编辑'">{{ editMode ? '👁️ 预览' : '✏️ 编辑' }}</button>
              <button class="toolbar-btn" :class="{ active: selectedNote.pinned }" @click="togglePin" title="置顶">📌 置顶</button>
              <div class="group-selector-wrapper">
                <button class="toolbar-btn group-selector-btn" @click="showGroupSelector = !showGroupSelector" :title="getGroupName(selectedNote.groupId) || '选择分组'">📁 {{ getGroupName(selectedNote.groupId) || '分组' }}</button>
                <div v-if="showGroupSelector" class="group-selector-dropdown">
                  <div class="group-option" :class="{ selected: selectedNote.groupId === null }" @click="assignGroup(null)"><span>📄 未分组</span></div>
                  <div v-for="group in noteGroups" :key="group.id" class="group-option" :class="{ selected: selectedNote.groupId === group.id }" @click="assignGroup(group.id)"><span>{{ group.icon || '📁' }} {{ group.name }}</span></div>
                </div>
              </div>
            </div>
            <div class="toolbar-right"><span class="save-status">{{ saveStatus }}</span></div>
          </div>
          <div class="editor-title-row">
            <input v-model="editorTitle" class="editor-title-input" placeholder="输入标题..." @input="onTitleChange"/>
          </div>
          <div v-if="editMode" class="editor-content">
            <textarea v-model="editorContent" class="editor-textarea" placeholder="开始用 Markdown 编写笔记..." @input="onContentChange" spellcheck="false"></textarea>
          </div>
          <div v-else class="editor-preview" v-html="renderedContent"></div>
        </template>
        <div v-else class="editor-empty">
          <svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
          <p>选择或创建一个笔记</p>
        </div>
      </main>
    </div>
    <Teleport to="body">
      <div v-if="deleteTarget" class="note-overlay" @click.self="deleteTarget = null">
        <div class="note-dialog" @click.stop>
          <h3>⚠️ 确认删除</h3>
          <p>确定要删除「{{ deleteTarget.title || '无标题' }}」吗？此操作不可撤销。</p>
          <div class="dialog-actions">
            <button class="btn btn-ghost" @click="deleteTarget = null">取消</button>
            <button class="btn btn-danger" @click="executeDelete">删除</button>
          </div>
        </div>
      </div>
    </Teleport>
    <Teleport to="body">
      <div v-if="showCreateGroup || editingGroup" class="note-overlay" @click.self="cancelGroupEdit">
        <div class="note-dialog" @click.stop>
          <h3>{{ editingGroup ? '✏️ 重命名分组' : '📁 新建分组' }}</h3>
          <div class="group-form">
            <div class="form-group"><label>名称</label><input v-model="groupForm.name" class="form-input" placeholder="分组名称" @keyup.enter="saveGroup" ref="groupInputRef"/></div>
            <div class="form-group"><label>图标</label><div class="icon-picker"><button v-for="icon in iconOptions" :key="icon" class="icon-option" :class="{ active: groupForm.icon === icon }" @click="groupForm.icon = icon">{{ icon }}</button></div></div>
          </div>
          <div class="dialog-actions">
            <button class="btn btn-ghost" @click="cancelGroupEdit">取消</button>
            <button class="btn btn-primary" @click="saveGroup" :disabled="!groupForm.name.trim()">保存</button>
          </div>
        </div>
      </div>
    </Teleport>
    <Teleport to="body">
      <div v-if="deleteGroupTarget" class="note-overlay" @click.self="deleteGroupTarget = null">
        <div class="note-dialog" @click.stop>
          <h3>⚠️ 删除分组</h3>
          <p>确定要删除分组「{{ deleteGroupTarget.name }}」吗？分组内的笔记不会被删除，将变为未分组。</p>
          <div class="dialog-actions">
            <button class="btn btn-ghost" @click="deleteGroupTarget = null">取消</button>
            <button class="btn btn-danger" @click="executeDeleteGroup">删除</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, computed, onMounted, nextTick } from 'vue'
import { getTauriAPI } from '../utils/tauri-api'
import { marked } from 'marked'
import { useToast } from '../composables/useToast'
const toast = useToast()

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
const selectedGroupId = ref<string>('__all__')
const editMode = ref(true)
const editorTitle = ref('')
const editorContent = ref('')
const saveStatus = ref('')
const deleteTarget = ref<Note | null>(null)
const groupsCollapsed = ref(false)
const showGroupSelector = ref(false)
let saveTimer: ReturnType<typeof setTimeout> | null = null

const showCreateGroup = ref(false)
const editingGroup = ref<NoteGroup | null>(null)
const deleteGroupTarget = ref<NoteGroup | null>(null)
const groupForm = ref({ name: '', icon: '📁' })
const groupInputRef = ref<HTMLInputElement | null>(null)
const iconOptions = ['📁', '📂', '📋', '📌', '🏷️', '💼', '🎯', '📚', '🔧', '💡', '🌟', '📝', '🗂️', '🏠', '🎨', '⚙️']

const filteredNotes = computed(() => {
  let result = notes.value
  if (selectedGroupId.value === '__ungrouped__') result = result.filter(n => !n.groupId)
  else if (selectedGroupId.value !== '__all__') result = result.filter(n => n.groupId === selectedGroupId.value)
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase()
    result = result.filter(n => n.title.toLowerCase().includes(q) || n.content.toLowerCase().includes(q))
  }
  return result
})

const renderedContent = computed(() => {
  if (!editorContent.value) return '<p class="preview-empty">没有内容</p>'
  let html = marked.parse(editorContent.value) as string
  if (searchQuery.value) {
    const regex = new RegExp(`(${escapeRegex(searchQuery.value)})`, 'gi')
    html = html.replace(regex, '<mark class="search-highlight">$1</mark>')
  }
  return html
})

function escapeRegex(str: string): string { return str.replace(/[.*+?${}()|[\]\\]/g, '\\$&') }

function getPreview(content: string): string {
  if (!content) return '空笔记'
  const plain = content.replace(/[#*`>-\[\]()!]/g, '').trim()
  return plain.slice(0, 80) + (plain.length > 80 ? '...' : '')
}

function highlightText(text: string): string {
  if (!searchQuery.value || !text) return text
  const regex = new RegExp(`(${escapeRegex(searchQuery.value)})`, 'gi')
  return text.replace(regex, '<mark>$1</mark>')
}

function getTagCount(tagsJson: string): number {
  try { const tags = JSON.parse(tagsJson); return Array.isArray(tags) ? tags.length : 0 } catch { return 0 }
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

async function loadNotes() {
  try {
    console.log("[loadNotes] called");
    const gid = selectedGroupId.value === '__all__' ? undefined : selectedGroupId.value
    notes.value = await getTauriAPI().getAllNotes(searchQuery.value || undefined, gid)
  } catch { toast.error('加载笔记失败') }
}

async function loadGroups() {
  try {
    console.log("[loadGroups] called");
    noteGroups.value = await getTauriAPI().getNoteGroups() } catch { toast.error('加载分组失败') }
}

function onSearch() { loadNotes() }

function selectGroup(groupId: string) {
    console.log("[onSearch] called");
    selectedGroupId.value = groupId; loadNotes() }

function selectNote(note: Note) {
    console.log("[selectGroup] called");
    selectedNote.value = note; editorTitle.value = note.title; editorContent.value = note.content
  editMode.value = false; saveStatus.value = ''; showGroupSelector.value = false
}

async function createNewNote() {
    console.log("[selectNote] called");
    try {
    console.log("[createNewNote] called");
    const gid = selectedGroupId.value !== '__all__' && selectedGroupId.value !== '__ungrouped__' ? selectedGroupId.value : null
    const note = await getTauriAPI().addNote({ title: '', content: '', pinned: false, groupId: gid })
    notes.value.unshift(note); selectNote(note); toast.success('已创建新笔记')
  } catch { toast.error('创建失败') }
}

function onTitleChange() {
  if (!selectedNote.value) return
  saveStatus.value = '保存中...'; if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(() => saveNote(), 500)
}

function onContentChange() {
    console.log("[onTitleChange] called");
    if (!selectedNote.value) return
  saveStatus.value = '保存中...'; if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(() => saveNote(), 500)
}

async function saveNote() {
    console.log("[onContentChange] called");
    if (!selectedNote.value) return
  try {
    console.log("[saveNote] called");
    const updated = await getTauriAPI().updateNote(selectedNote.value.id, { title: editorTitle.value, content: editorContent.value })
    if (updated) {
      selectedNote.value = updated
      const idx = notes.value.findIndex(n => n.id === updated.id)
      if (idx !== -1) notes.value[idx] = updated
      saveStatus.value = '已保存'
    }
  } catch { saveStatus.value = '保存失败'; toast.error('保存失败') }
}

async function togglePin() {
  if (!selectedNote.value) return
  try {
    console.log("[togglePin] called");
    const updated = await getTauriAPI().updateNote(selectedNote.value.id, { pinned: !selectedNote.value.pinned })
    if (updated) {
      selectedNote.value = updated
      const idx = notes.value.findIndex(n => n.id === updated.id)
      if (idx !== -1) notes.value[idx] = updated
    }
  } catch { toast.error('操作失败') }
}

async function assignGroup(groupId: string | null) {
  if (!selectedNote.value) return
  try {
    console.log("[assignGroup] called");
    const updated = await getTauriAPI().updateNote(selectedNote.value.id, { groupId })
    if (updated) {
      selectedNote.value = updated
      const idx = notes.value.findIndex(n => n.id === updated.id)
      if (idx !== -1) notes.value[idx] = updated
    }
    showGroupSelector.value = false
  } catch { toast.error('设置分组失败') }
}

function confirmDelete(note: Note) { deleteTarget.value = note }

async function executeDelete() {
    console.log("[confirmDelete] called");
    if (!deleteTarget.value) return
  try {
    console.log("[executeDelete] called");
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

function confirmDeleteGroup(group: NoteGroup) { deleteGroupTarget.value = group }

async function executeDeleteGroup() {
    console.log("[confirmDeleteGroup] called");
    if (!deleteGroupTarget.value) return
  try {
    console.log("[executeDeleteGroup] called");
    await getTauriAPI().deleteNoteGroup(deleteGroupTarget.value.id)
    const notesToUpdate = notes.value.filter(n => n.groupId === deleteGroupTarget.value!.id)
    for (const note of notesToUpdate) {
      const updated = await getTauriAPI().updateNote(note.id, { groupId: null })
      if (updated) { const idx = notes.value.findIndex(n => n.id === updated.id); if (idx !== -1) notes.value[idx] = updated }
    }
    if (selectedGroupId.value === deleteGroupTarget.value.id) selectedGroupId.value = '__all__'
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
    console.log("[saveGroup] called");
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
    console.log("[components/NoteManager.vue] mounted");
    loadGroups(); loadNotes() })
</script>

<style scoped>
.note-manager { width: 100%; max-width: 1400px; margin: 0 auto; display: flex; flex-direction: column; gap: 20px; height: calc(100vh - 40px); padding: 8px; }
.note-header { display: flex; align-items: center; justify-content: space-between; padding: 14px 20px; background: linear-gradient(135deg, oklch(var(--b1)) 0%, rgba(66, 133, 244, 0.05) 100%); border-radius: 14px; border: 1px solid oklch(var(--bc) / 0.1); flex-shrink: 0; }
.note-header-left { display: flex; align-items: center; gap: 14px; }
.header-icon { width: 48px; height: 48px; display: flex; align-items: center; justify-content: center; font-size: 24px; background: linear-gradient(135deg, #fbbc04, #ff6d01); border-radius: 12px; box-shadow: 0 3px 10px rgba(251, 188, 4, 0.25); }
.header-info { display: flex; flex-direction: column; gap: 2px; }
.note-title { font-size: 20px; font-weight: 700; color: oklch(var(--bc)); margin: 0; }
.note-subtitle { font-size: 12px; color: oklch(var(--bc) / 0.6); margin: 0; opacity: 0.8; }
.note-header-actions { display: flex; align-items: center; gap: 12px; }
.search-wrapper { position: relative; display: flex; align-items: center; color: oklch(var(--bc) / 0.6); }
.search-wrapper svg { position: absolute; left: 10px; pointer-events: none; }
.search-input { padding: 8px 12px 8px 34px; font-size: 13px; border: 1px solid oklch(var(--bc) / 0.1); border-radius: 20px; background: oklch(var(--b2)); color: oklch(var(--bc)); width: 200px; transition: all 0.2s ease; }
.search-input:focus { outline: none; border-color: oklch(var(--p)); box-shadow: 0 0 0 3px rgba(66, 133, 244, 0.15); width: 260px; }
.btn-new { display: flex; align-items: center; gap: 6px; padding: 8px 16px; font-size: 13px; font-weight: 600; border: none; border-radius: 20px; background: linear-gradient(135deg, oklch(var(--p)), #34a853); color: white; cursor: pointer; transition: all 0.2s ease; box-shadow: 0 2px 6px rgba(66, 133, 244, 0.25); }
.btn-new:hover { transform: translateY(-1px); box-shadow: 0 4px 12px rgba(66, 133, 244, 0.35); }
.note-layout { display: flex; gap: 16px; flex: 1; min-height: 0; }
.note-sidebar { width: 300px; min-width: 280px; background: oklch(var(--b1)); border-radius: 12px; border: 1px solid oklch(var(--bc) / 0.1); overflow-y: auto; flex-shrink: 0; display: flex; flex-direction: column; }
.group-section { border-bottom: 1px solid oklch(var(--bc) / 0.1); flex-shrink: 0; }
.group-header { display: flex; align-items: center; justify-content: space-between; padding: 10px 14px; cursor: pointer; user-select: none; transition: background 0.15s ease; }
.group-header:hover { background: oklch(var(--b2)); }
.group-header-title { font-size: 12px; font-weight: 600; color: oklch(var(--bc) / 0.6); text-transform: uppercase; letter-spacing: 0.5px; }
.group-chevron { transition: transform 0.2s ease; color: oklch(var(--bc) / 0.6); }
.group-chevron.collapsed { transform: rotate(-90deg); }
.group-list { padding: 4px 8px 8px; }
.group-item { display: flex; align-items: center; gap: 8px; padding: 7px 10px; border-radius: 8px; cursor: pointer; transition: all 0.15s ease; position: relative; }
.group-item:hover { background: oklch(var(--b2)); }
.group-item:hover .group-actions { opacity: 1; }
.group-item.active { background: rgba(66, 133, 244, 0.12); }
.group-icon { font-size: 14px; flex-shrink: 0; }
.group-name { font-size: 13px; color: oklch(var(--bc)); flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.group-actions { display: flex; gap: 2px; opacity: 0; transition: opacity 0.15s ease; }
.group-action-btn { width: 20px; height: 20px; display: flex; align-items: center; justify-content: center; border: none; border-radius: 4px; background: transparent; cursor: pointer; font-size: 10px; padding: 0; }
.group-action-btn:hover { background: rgba(0, 0, 0, 0.08); }
.group-action-delete:hover { background: rgba(210, 15, 57, 0.15); }
.group-add { display: flex; align-items: center; gap: 6px; padding: 7px 10px; border-radius: 8px; cursor: pointer; color: oklch(var(--bc) / 0.6); font-size: 12px; transition: all 0.15s ease; margin-top: 2px; }
.group-add:hover { background: oklch(var(--b2)); color: oklch(var(--p)); }
.sidebar-empty { padding: 40px 20px; text-align: center; color: oklch(var(--bc) / 0.6); font-size: 14px; }
.note-list { display: flex; flex-direction: column; overflow-y: auto; flex: 1; }
.note-list-item { padding: 12px 14px; cursor: pointer; border-bottom: 1px solid oklch(var(--bc) / 0.1); transition: all 0.15s ease; }
.note-list-item:hover { background: oklch(var(--b2)); }
.note-list-item.active { background: rgba(66, 133, 244, 0.08); border-left: 3px solid oklch(var(--p)); }
.note-list-item.pinned { background: rgba(251, 188, 4, 0.05); }
.note-item-top { display: flex; align-items: center; justify-content: space-between; margin-bottom: 4px; }
.note-item-title { display: flex; align-items: center; gap: 4px; min-width: 0; flex: 1; }
.pin-icon { font-size: 12px; flex-shrink: 0; }
.title-text { font-size: 14px; font-weight: 600; color: oklch(var(--bc)); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.note-item-delete { width: 24px; height: 24px; display: flex; align-items: center; justify-content: center; border: none; border-radius: 6px; background: transparent; color: oklch(var(--bc) / 0.6); cursor: pointer; opacity: 0; transition: all 0.15s ease; flex-shrink: 0; }
.note-list-item:hover .note-item-delete { opacity: 1; }
.note-item-delete:hover { background: rgba(210, 15, 57, 0.1); color: oklch(var(--er)); }
.note-item-preview { font-size: 12px; color: oklch(var(--bc) / 0.6); opacity: 0.8; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; margin-bottom: 4px; }
.note-item-meta { display: flex; align-items: center; justify-content: space-between; font-size: 11px; color: oklch(var(--bc) / 0.6); opacity: 0.6; }
.note-item-group { background: rgba(66, 133, 244, 0.1); color: oklch(var(--p)); padding: 1px 6px; border-radius: 4px; font-size: 10px; }
.note-editor { flex: 1; min-width: 0; background: oklch(var(--b1)); border-radius: 12px; border: 1px solid oklch(var(--bc) / 0.1); display: flex; flex-direction: column; overflow: hidden; }
.editor-toolbar { display: flex; align-items: center; justify-content: space-between; padding: 10px 16px; border-bottom: 1px solid oklch(var(--bc) / 0.1); flex-shrink: 0; }
.toolbar-left, .toolbar-right { display: flex; align-items: center; gap: 8px; }
.toolbar-btn { padding: 6px 12px; font-size: 12px; font-weight: 500; border: 1px solid oklch(var(--bc) / 0.1); border-radius: 8px; background: transparent; color: oklch(var(--bc) / 0.6); cursor: pointer; transition: all 0.15s ease; white-space: nowrap; }
.toolbar-btn:hover { background: oklch(var(--b2)); color: oklch(var(--bc)); }
.toolbar-btn.active { background: oklch(var(--p)); color: white; border-color: oklch(var(--p)); }
.group-selector-wrapper { position: relative; }
.group-selector-btn { display: flex; align-items: center; gap: 4px; }
.group-selector-dropdown { position: absolute; top: 100%; left: 0; margin-top: 4px; background: oklch(var(--b1)); border: 1px solid oklch(var(--bc) / 0.1); border-radius: 10px; box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15); z-index: 100; min-width: 180px; max-height: 240px; overflow-y: auto; padding: 4px; }
.group-option { padding: 8px 12px; font-size: 13px; cursor: pointer; border-radius: 6px; transition: background 0.1s ease; }
.group-option:hover { background: oklch(var(--b2)); }
.group-option.selected { background: rgba(66, 133, 244, 0.12); color: oklch(var(--p)); font-weight: 500; }
.save-status { font-size: 12px; color: oklch(var(--bc) / 0.6); }
.editor-title-row { padding: 16px 20px 8px; flex-shrink: 0; }
.editor-title-input { width: 100%; font-size: 22px; font-weight: 700; color: oklch(var(--bc)); border: none; background: transparent; padding: 0; }
.editor-title-input:focus { outline: none; }
.editor-title-input::placeholder { color: oklch(var(--bc) / 0.6); opacity: 0.4; }
.editor-content { flex: 1; min-height: 0; padding: 8px 20px 20px; }
.editor-textarea { width: 100%; height: 100%; min-height: 300px; font-size: 14px; line-height: 1.7; color: oklch(var(--bc)); background: transparent; border: none; resize: none; font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; }
.editor-textarea:focus { outline: none; }
.editor-textarea::placeholder { color: oklch(var(--bc) / 0.6); opacity: 0.4; }
.editor-preview { flex: 1; min-height: 0; padding: 8px 20px 20px; overflow-y: auto; line-height: 1.7; color: oklch(var(--bc)); }
.editor-preview :deep(h1) { font-size: 28px; font-weight: 700; margin: 24px 0 12px; border-bottom: 1px solid oklch(var(--bc) / 0.1); padding-bottom: 8px; }
.editor-preview :deep(h2) { font-size: 22px; font-weight: 600; margin: 20px 0 10px; }
.editor-preview :deep(h3) { font-size: 18px; font-weight: 600; margin: 16px 0 8px; }
.editor-preview :deep(p) { margin: 8px 0; }
.editor-preview :deep(code) { background: oklch(var(--b2)); padding: 2px 6px; border-radius: 4px; font-family: 'SF Mono', monospace; font-size: 13px; }
.editor-preview :deep(pre) { background: oklch(var(--b2)); padding: 16px; border-radius: 8px; overflow-x: auto; margin: 12px 0; }
.editor-preview :deep(pre code) { background: none; padding: 0; }
.editor-preview :deep(blockquote) { border-left: 3px solid oklch(var(--p)); padding-left: 16px; margin: 12px 0; color: oklch(var(--bc) / 0.6); }
.editor-preview :deep(ul), .editor-preview :deep(ol) { padding-left: 24px; margin: 8px 0; }
.editor-preview :deep(li) { margin: 4px 0; }
.editor-preview :deep(a) { color: oklch(var(--p)); text-decoration: underline; }
.editor-preview :deep(img) { max-width: 100%; border-radius: 8px; margin: 12px 0; }
.editor-preview :deep(table) { border-collapse: collapse; width: 100%; margin: 12px 0; }
.editor-preview :deep(th), .editor-preview :deep(td) { border: 1px solid oklch(var(--bc) / 0.1); padding: 8px 12px; text-align: left; }
.editor-preview :deep(th) { background: oklch(var(--b2)); font-weight: 600; }
.editor-preview :deep(.search-highlight) { background: rgba(251, 191, 36, 0.35); color: oklch(var(--bc)); border-radius: 2px; padding: 0 2px; }
.editor-preview .preview-empty { color: oklch(var(--bc) / 0.6); opacity: 0.5; }
.editor-empty { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; color: oklch(var(--bc) / 0.6); opacity: 0.5; }
.editor-empty svg { margin-bottom: 16px; }
.editor-empty p { font-size: 14px; }
.note-overlay { position: fixed; inset: 0; background: rgba(0, 0, 0, 0.5); display: flex; align-items: center; justify-content: center; z-index: 10000; }
.note-dialog { background: oklch(var(--b1)); border-radius: 16px; padding: 24px; max-width: 400px; width: 90%; box-shadow: 0 16px 48px rgba(0, 0, 0, 0.3); }
.note-dialog h3 { font-size: 18px; font-weight: 700; margin: 0 0 12px; color: oklch(var(--bc)); }
.note-dialog p { font-size: 14px; color: oklch(var(--bc) / 0.6); margin: 0 0 20px; }
.dialog-actions { display: flex; justify-content: flex-end; gap: 10px; }
.btn { padding: 8px 16px; font-size: 13px; font-weight: 600; border-radius: 8px; cursor: pointer; transition: all 0.15s ease; border: none; }
.btn-ghost { background: transparent; color: oklch(var(--bc) / 0.6); border: 1px solid oklch(var(--bc) / 0.1); }
.btn-ghost:hover { background: oklch(var(--b2)); }
.btn-danger { background: oklch(var(--er)); color: white; }
.btn-danger:hover { opacity: 0.9; }
.btn-primary { background: oklch(var(--p)); color: white; }
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
.group-form { margin-bottom: 20px; }
.form-group { margin-bottom: 14px; }
.form-group label { display: block; font-size: 12px; font-weight: 600; color: oklch(var(--bc) / 0.6); margin-bottom: 6px; }
.form-input { width: 100%; padding: 8px 12px; font-size: 14px; border: 1px solid oklch(var(--bc) / 0.1); border-radius: 8px; background: oklch(var(--b2)); color: oklch(var(--bc)); }
.form-input:focus { outline: none; border-color: oklch(var(--p)); box-shadow: 0 0 0 3px rgba(66, 133, 244, 0.15); }
.icon-picker { display: flex; flex-wrap: wrap; gap: 4px; }
.icon-option { width: 32px; height: 32px; display: flex; align-items: center; justify-content: center; font-size: 16px; border: 2px solid transparent; border-radius: 8px; background: transparent; cursor: pointer; transition: all 0.15s ease; }
.icon-option:hover { background: oklch(var(--b2)); }
.icon-option.active { border-color: oklch(var(--p)); background: rgba(66, 133, 244, 0.1); }
.note-list-item :deep(mark) { background: rgba(251, 191, 36, 0.35); color: oklch(var(--bc)); border-radius: 2px; padding: 0 2px; }
</style>
