<template>
  <div class="w-full flex flex-col gap-3 h-full p-3">
    <div class="flex items-center justify-between px-4 py-3 bg-base-100 border border-base-content/10 rounded-xl shrink-0">
      <div class="flex items-center gap-3">
        <div class="w-8 h-8 flex items-center justify-center text-base bg-base-200 rounded-lg"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg></div>
        <div class="flex flex-col gap-0.5">
          <h2 class="text-base font-bold text-base-content m-0">笔记</h2>
          <p class="text-xs text-base-content/60 m-0">记录工作与生活的灵感</p>
        </div>
      </div>
      <div class="flex items-center gap-2">
        <div class="relative flex items-center text-base-content/60">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="absolute left-2.5 pointer-events-none"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
          <input v-model="searchQuery" class="input input-bordered rounded-full px-3 py-1.5 pl-8 text-xs w-[180px] transition-all duration-200 focus:w-[220px]" placeholder="搜索笔记..." @input="onSearch"/>
        </div>
        <button class="btn btn-primary btn-sm gap-1.5" @click="createNewNote">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
          新建笔记
        </button>
      </div>
    </div>
    <div class="flex gap-3 flex-1 min-h-0">
      <aside class="w-64 min-w-[220px] bg-base-100 rounded-xl border border-base-content/10 overflow-y-auto shrink-0 flex flex-col">
        <div class="border-b border-base-content/10 shrink-0">
          <div class="flex items-center justify-between px-3 py-2 cursor-pointer select-none transition-colors duration-150 hover:bg-base-200" @click="groupsCollapsed = !groupsCollapsed">
            <span class="text-xs font-semibold text-base-content/60 uppercase tracking-wider"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg> 分组</span>
            <svg :class="['transition-transform duration-200 text-base-content/60', { '-rotate-90': groupsCollapsed }]" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"/></svg>
          </div>
          <div v-show="!groupsCollapsed" class="px-2 pb-2 pt-1">
            <div :class="['flex items-center gap-2 px-2.5 py-[6px] rounded-lg cursor-pointer transition-all duration-150 relative hover:bg-base-200 group/item', { 'bg-primary/10': selectedGroupId === '__all__' }]" @click="selectGroup('__all__')">
              <span class="text-sm shrink-0"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg></span><span class="text-sm text-base-content flex-1 overflow-hidden text-ellipsis whitespace-nowrap">全部</span>
            </div>
            <div :class="['flex items-center gap-2 px-2.5 py-[6px] rounded-lg cursor-pointer transition-all duration-150 relative hover:bg-base-200 group/item', { 'bg-primary/10': selectedGroupId === '__ungrouped__' }]" @click="selectGroup('__ungrouped__')">
              <span class="text-sm shrink-0"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg></span><span class="text-sm text-base-content flex-1 overflow-hidden text-ellipsis whitespace-nowrap">未分组</span>
            </div>
            <div v-for="group in noteGroups" :key="group.id" :class="['flex items-center gap-2 px-2.5 py-[6px] rounded-lg cursor-pointer transition-all duration-150 relative hover:bg-base-200 group/item', { 'bg-primary/10': selectedGroupId === group.id }]" @click="selectGroup(group.id)">
              <span class="text-sm shrink-0"><template v-if="group.icon">{{ group.icon }}</template><template v-else><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg></template></span>
              <span class="text-sm text-base-content flex-1 overflow-hidden text-ellipsis whitespace-nowrap">{{ group.name }}</span>
              <div class="flex gap-0.5 opacity-0 group-hover/item:opacity-100 transition-opacity duration-150" @click.stop>
                <button class="w-5 h-5 flex items-center justify-center border-none rounded bg-transparent cursor-pointer text-[10px] p-0 hover:bg-black/8" @click="startRenameGroup(group)" title="重命名"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg></button>
                <button class="w-5 h-5 flex items-center justify-center border-none rounded bg-transparent cursor-pointer text-[10px] p-0 hover:bg-[rgba(210,15,57,0.15)]" @click="confirmDeleteGroup(group)" title="删除"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg></button>
              </div>
            </div>
            <div class="flex items-center gap-1.5 px-2.5 py-[6px] rounded-lg cursor-pointer text-base-content/60 text-xs transition-all duration-150 mt-0.5 hover:bg-base-200 hover:text-primary" @click="showCreateGroup = true">
              <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
              <span>新建分组</span>
            </div>
          </div>
        </div>
        <div v-if="filteredNotes.length === 0" class="px-4 py-8 text-center text-base-content/60 text-sm">
          <p>{{ searchQuery ? '没有匹配的笔记' : '暂无笔记' }}</p>
        </div>
        <div v-else class="flex flex-col overflow-y-auto flex-1">
          <div v-for="note in filteredNotes" :key="note.id" :class="['px-3 py-2.5 cursor-pointer border-b border-base-content/10 transition-all duration-150 hover:bg-base-200 group/item', { 'bg-primary/10 border-l-[3px] border-primary': selectedNote?.id === note.id, 'bg-warning/5': note.pinned }]" @click="selectNote(note)">
            <div class="flex items-center justify-between mb-0.5">
              <div class="flex items-center gap-1 min-w-0 flex-1">
                <span v-if="note.pinned" class="text-xs shrink-0"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M12 2L15.09 8.26L22 9.27L17 14.14L18.18 21.02L12 17.77L5.82 21.02L7 14.14L2 9.27L8.91 8.26L12 2z"/></svg></span>
                <span class="text-sm font-semibold text-base-content overflow-hidden text-ellipsis whitespace-nowrap" v-html="highlightText(note.title || '无标题')"></span>
              </div>
              <button class="w-5 h-5 flex items-center justify-center border-none rounded-lg bg-transparent text-base-content/60 cursor-pointer opacity-0 transition-all duration-150 shrink-0 group-hover/item:opacity-100 hover:bg-[rgba(210,15,57,0.1)] hover:text-error" @click.stop="confirmDelete(note)" title="删除">
                <svg xmlns="http://www.w3.org/2000/svg" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
              </button>
            </div>
            <div class="text-xs text-base-content/60 opacity-80 overflow-hidden text-ellipsis whitespace-nowrap mb-0.5" v-html="highlightText(getPreview(note.content))"></div>
            <div class="flex items-center justify-between text-[11px] text-base-content/60 opacity-60">
              <span>{{ formatDate(note.updatedAt) }}</span>
              <span v-if="getGroupName(note.groupId)" class="badge badge-sm badge-ghost text-primary bg-[rgba(66,133,244,0.1)] border-none text-[10px]">{{ getGroupName(note.groupId) }}</span>
            </div>
          </div>
        </div>
      </aside>
      <main class="flex-1 min-w-0 bg-base-100 rounded-xl border border-base-content/10 flex flex-col overflow-hidden">
        <template v-if="selectedNote">
          <div class="flex items-center justify-between px-4 py-2 border-b border-base-content/10 shrink-0">
            <div class="flex items-center gap-2">
              <button :class="['px-2.5 py-1.5 text-xs font-medium border border-base-content/10 rounded-lg bg-transparent text-base-content/60 cursor-pointer transition-all duration-150 whitespace-nowrap hover:bg-base-200 hover:text-base-content', { '!bg-primary !text-white !border-primary hover:!bg-primary hover:!text-white': editMode }]" @click="editMode = !editMode" :title="editMode ? '预览' : '编辑'"><template v-if="editMode"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg> 预览</template><template v-else><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg> 编辑</template></button>
              <button :class="['px-2.5 py-1.5 text-xs font-medium border border-base-content/10 rounded-lg bg-transparent text-base-content/60 cursor-pointer transition-all duration-150 whitespace-nowrap hover:bg-base-200 hover:text-base-content', { '!bg-primary !text-white !border-primary hover:!bg-primary hover:!text-white': selectedNote.pinned }]" @click="togglePin" title="置顶"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M12 2L15.09 8.26L22 9.27L17 14.14L18.18 21.02L12 17.77L5.82 21.02L7 14.14L2 9.27L8.91 8.26L12 2z"/></svg> 置顶</button>
              <div class="relative">
                <button class="px-2.5 py-1.5 text-xs font-medium border border-base-content/10 rounded-lg bg-transparent text-base-content/60 cursor-pointer transition-all duration-150 whitespace-nowrap hover:bg-base-200 hover:text-base-content flex items-center gap-1" @click="showGroupSelector = !showGroupSelector" :title="getGroupName(selectedNote.groupId) || '选择分组'"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg> {{ getGroupName(selectedNote.groupId) || '分组' }}</button>
                <div v-if="showGroupSelector" class="absolute top-full left-0 mt-1 bg-base-100 border border-base-content/10 rounded-lg shadow-lg z-[100] min-w-[160px] max-h-60 overflow-y-auto p-1">
                  <div :class="['px-3 py-1.5 text-sm cursor-pointer rounded-lg transition-colors duration-100 hover:bg-base-200', { 'bg-primary/10 text-primary font-medium': selectedNote.groupId === null }]" @click="assignGroup(null)"><span><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg> 未分组</span></div>
                  <div v-for="group in noteGroups" :key="group.id" :class="['px-3 py-1.5 text-sm cursor-pointer rounded-lg transition-colors duration-100 hover:bg-base-200', { 'bg-primary/10 text-primary font-medium': selectedNote.groupId === group.id }]"><span><template v-if="group.icon">{{ group.icon }}</template><template v-else><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg></template> {{ group.name }}</span></div>
                </div>
              </div>
            </div>
            <div class="flex items-center gap-2"><span class="text-xs text-base-content/60">{{ saveStatus }}</span></div>
          </div>
          <div class="px-4 pt-3 pb-1 shrink-0">
            <input v-model="editorTitle" class="input w-full text-xl font-bold text-base-content border-none bg-transparent px-0 focus:outline-none placeholder:text-base-content/60 placeholder:opacity-40" placeholder="输入标题..." @input="onTitleChange"/>
          </div>
          <div v-if="editMode" class="flex-1 min-h-0 px-4 pb-4 pt-2">
            <textarea v-model="editorContent" class="w-full h-full min-h-[300px] text-sm leading-relaxed text-base-content bg-transparent border-none resize-none font-mono focus:outline-none placeholder:text-base-content/60 placeholder:opacity-40" placeholder="开始用 Markdown 编写笔记..." @input="onContentChange" spellcheck="false"></textarea>
          </div>
          <div v-else class="editor-preview flex-1 min-h-0 px-4 pb-4 pt-2 overflow-y-auto leading-relaxed text-base-content" v-html="renderedContent"></div>
        </template>
        <div v-else class="flex-1 flex flex-col items-center justify-center text-base-content/60 opacity-50">
          <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="mb-3"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
          <p class="text-sm">选择或创建一个笔记</p>
        </div>
      </main>
    </div>
    <Teleport to="body">
      <div v-if="deleteTarget" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[10000]" @click.self="deleteTarget = null">
        <div class="bg-base-100 rounded-2xl p-5 max-w-sm w-[90%] shadow-[0_16px_48px_rgba(0,0,0,0.3)]" @click.stop>
          <h3 class="text-lg font-bold m-0 mb-2 text-base-content"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg> 确认删除</h3>
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
          <h3 class="text-lg font-bold m-0 mb-3 text-base-content"><template v-if="editingGroup"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg> 重命名分组</template><template v-else><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg> 新建分组</template></h3>
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
          <h3 class="text-lg font-bold m-0 mb-2 text-base-content"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg> 删除分组</h3>
          <p class="text-sm text-base-content/60 m-0 mb-4">确定要删除分组「{{ deleteGroupTarget.name }}」吗？分组内的笔记不会被删除，将变为未分组。</p>
          <div class="flex justify-end gap-2">
            <button class="btn btn-ghost" @click="deleteGroupTarget = null">取消</button>
            <button class="btn btn-error" @click="executeDeleteGroup">删除</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, computed, onMounted, nextTick } from 'vue'
import { getTauriAPI } from '../../utils/tauri-api'
import { marked } from 'marked'
import { useToast } from '../../composables/useToast'
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

async function loadNotes() {
  try {
    const gid = selectedGroupId.value === '__all__' ? undefined : selectedGroupId.value
    notes.value = await getTauriAPI().getAllNotes(searchQuery.value || undefined, gid)
  } catch { toast.error('加载笔记失败') }
}

async function loadGroups() {
  try {
    noteGroups.value = await getTauriAPI().getNoteGroups() } catch { toast.error('加载分组失败') }
}

function onSearch() { loadNotes() }

function selectGroup(groupId: string) {
    selectedGroupId.value = groupId; loadNotes() }

function selectNote(note: Note) {
    selectedNote.value = note; editorTitle.value = note.title; editorContent.value = note.content
  editMode.value = false; saveStatus.value = ''; showGroupSelector.value = false
}

async function createNewNote() {
    try {
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
    if (!selectedNote.value) return
  saveStatus.value = '保存中...'; if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(() => saveNote(), 500)
}

async function saveNote() {
    if (!selectedNote.value) return
  try {
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
    loadGroups(); loadNotes() })
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
