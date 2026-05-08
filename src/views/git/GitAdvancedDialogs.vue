<template>
  <!-- ===== 1. File History Dialog ===== -->
  <div v-if="showFileHistoryDialog" class="fixed inset-0 bg-black/40 z-[900] flex items-center justify-center" @click="$emit('update:show-file-history-dialog', false)">
    <div class="max-w-xl w-full bg-base-100 rounded-xl shadow-2xl flex flex-col overflow-hidden max-h-[70vh]" @click.stop>
      <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
        <span class="font-semibold text-sm">文件历史: {{ fileHistoryFile }}</span>
        <button class="btn btn-ghost btn-xs" @click="$emit('update:show-file-history-dialog', false)">✕</button>
      </div>
      <div class="flex-1 overflow-y-auto p-2">
        <div v-for="commit in fileHistoryData" :key="commit.hash" class="flex items-center gap-2 px-2 py-1.5 rounded text-xs hover:bg-base-content/5 transition-colors duration-100">
          <code class="font-mono text-[11px] text-primary bg-primary/10 px-1 py-[1px] rounded-sm shrink-0">{{ commit.hash.substring(0, 7) }}</code>
          <span class="text-base-content/60 text-[11px] shrink-0" :title="formatFullDate(commit.date)">{{ formatRelativeDate(commit.date) }}</span>
          <span class="text-base-content/60 truncate block flex-1 min-w-0">{{ commit.message }}</span>
          <span class="text-base-content shrink-0">{{ getAuthorName(commit.author) }}</span>
        </div>
        <div v-if="fileHistoryData.length === 0 && !loading" class="p-5 text-center text-base-content/60 text-xs">没有历史记录</div>
      </div>
    </div>
  </div>

  <!-- ===== 2. Blame Dialog ===== -->
  <div v-if="showBlameDialog" class="fixed inset-0 bg-black/40 z-[900] flex items-center justify-center" @click="$emit('update:show-blame-dialog', false)">
    <div class="max-w-2xl w-full bg-base-100 rounded-xl shadow-2xl flex flex-col overflow-hidden max-h-[80vh]" @click.stop>
      <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
        <span class="font-semibold text-sm">Blame: {{ blameFile }}</span>
        <button class="btn btn-ghost btn-xs" @click="$emit('update:show-blame-dialog', false)">✕</button>
      </div>
      <div class="flex-1 overflow-auto p-2.5">
        <pre class="font-mono text-[11px] leading-relaxed whitespace-pre text-base-content">{{ blameData }}</pre>
      </div>
    </div>
  </div>

  <!-- ===== 3. Interactive Rebase Dialog ===== -->
  <div v-if="showInteractiveRebaseDialog" class="fixed inset-0 bg-black/40 z-[900] flex items-center justify-center" @click="$emit('update:show-interactive-rebase-dialog', false)">
    <div class="max-w-xl w-full bg-base-100 rounded-xl shadow-2xl flex flex-col overflow-hidden max-h-[80vh]" @click.stop>
      <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
        <span class="font-semibold text-sm">交互式 Rebase</span>
        <button class="btn btn-ghost btn-xs" @click="$emit('update:show-interactive-rebase-dialog', false)">✕</button>
      </div>
      <div class="flex flex-col gap-2.5 p-3">
        <label class="text-xs font-semibold text-base-content/60">Rebase 起点</label>
        <div class="flex gap-2">
          <input
            :value="interactiveRebaseBase"
            @input="$emit('update:interactive-rebase-base', ($event.target as HTMLInputElement).value)"
            class="flex-1 p-2 px-2.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[13px] outline-none focus:border-primary"
            placeholder="HEAD~6, 提交hash, 分支名..."
            spellcheck="false"
          />
          <button class="btn btn-ghost btn-sm" @click="$emit('load-ir-commits')" :disabled="!interactiveRebaseBase.trim() || irLoading">加载</button>
        </div>
      </div>
      <div v-if="irCommits.length > 0" class="flex-1 overflow-y-auto max-h-[400px] p-2 space-y-0.5">
        <div
          v-for="(c, idx) in irCommits"
          :key="c.hash"
          class="flex items-center gap-1.5 px-2 py-1 rounded text-xs cursor-pointer transition-colors duration-100"
          :class="irSelectedIndex === idx ? 'bg-primary/10' : 'hover:bg-base-content/5'"
          @click="irSelectedIndex = idx"
        >
          <select
            :value="c.action"
            @change="c.action = ($event.target as HTMLSelectElement).value"
            @click.stop
            class="bg-base-200 border border-base-content/10 rounded text-[11px] font-mono px-1 py-0.5 cursor-pointer shrink-0 text-base-content"
          >
            <option value="pick" class="text-green-500">pick</option>
            <option value="reword" class="text-blue-500">reword</option>
            <option value="edit" class="text-amber-500">edit</option>
            <option value="squash" class="text-purple-500">squash</option>
            <option value="fixup" class="text-violet-500">fixup</option>
            <option value="drop" class="text-red-500">drop</option>
          </select>
          <code class="font-mono text-[11px] text-primary bg-primary/10 px-1 py-[1px] rounded-sm shrink-0">{{ c.hash.substring(0, 7) }}</code>
          <span class="text-base-content/60 truncate flex-1 min-w-0">{{ c.message }}</span>
          <button class="btn btn-ghost btn-xs px-1 text-sm leading-none" @click.stop="$emit('ir-move-up', idx)" :disabled="idx === 0" title="上移">↑</button>
          <button class="btn btn-ghost btn-xs px-1 text-sm leading-none" @click.stop="$emit('ir-move-down', idx)" :disabled="idx === irCommits.length - 1" title="下移">↓</button>
        </div>
      </div>
      <div class="flex justify-end gap-2 p-3 border-t border-base-content/10">
        <button class="btn btn-ghost btn-sm" @click="$emit('update:show-interactive-rebase-dialog', false)">取消</button>
        <button class="btn btn-primary btn-sm" @click="$emit('start-interactive-rebase')" :disabled="irCommits.length === 0 || irLoading">开始 Rebase</button>
      </div>
    </div>
  </div>

  <!-- ===== 4. Remotes Dialog ===== -->
  <div v-if="showRemotesDialog" class="fixed inset-0 bg-black/40 z-[900] flex items-center justify-center" @click="$emit('update:show-remotes-dialog', false)">
    <div class="max-w-lg w-full bg-base-100 rounded-xl shadow-2xl flex flex-col overflow-hidden max-h-[70vh]" @click.stop>
      <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
        <span class="font-semibold text-sm">远程仓库管理</span>
        <button class="btn btn-ghost btn-xs" @click="$emit('update:show-remotes-dialog', false)">✕</button>
      </div>
      <div class="px-4 py-2 border-b border-base-content/10">
        <button class="btn btn-primary btn-sm" @click="$emit('update:show-add-remote-form', true)">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
          </svg>
          添加远程
        </button>
      </div>
      <div v-if="showAddRemoteForm" class="flex flex-col gap-2.5 p-3 border-b border-base-content/10">
        <label class="text-xs font-semibold text-base-content/60">名称</label>
        <input :value="newRemoteName" @input="$emit('update:new-remote-name', ($event.target as HTMLInputElement).value)" class="p-2 px-2.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[13px] outline-none focus:border-primary" placeholder="origin" spellcheck="false" />
        <label class="text-xs font-semibold text-base-content/60">URL</label>
        <input :value="newRemoteUrl" @input="$emit('update:new-remote-url', ($event.target as HTMLInputElement).value)" class="p-2 px-2.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[13px] outline-none focus:border-primary" placeholder="git@github.com:user/repo.git" spellcheck="false" />
        <div class="flex justify-end gap-2">
          <button class="btn btn-ghost btn-sm" @click="$emit('update:show-add-remote-form', false)">取消</button>
          <button class="btn btn-primary btn-sm" @click="$emit('add-remote')" :disabled="!newRemoteName.trim() || !newRemoteUrl.trim()">添加</button>
        </div>
      </div>
      <div class="flex-1 overflow-y-auto p-2">
        <div v-for="r in remotesList" :key="r" class="flex items-center gap-2 px-2 py-1.5 rounded text-xs hover:bg-base-content/5 transition-colors duration-100">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" class="shrink-0 text-base-content/60">
            <circle cx="12" cy="12" r="10" /><line x1="2" y1="12" x2="22" y2="12" />
            <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
          </svg>
          <span class="font-mono font-semibold text-primary min-w-[60px]">{{ r }}</span>
          <span class="flex-1 text-[11px] text-base-content/60 truncate font-mono" :title="remoteUrls[r]">{{ remoteUrls[r] }}</span>
          <button class="btn btn-ghost btn-xs" @click.stop="$emit('fetch-remote', r)" title="Fetch">Fetch</button>
          <button class="btn btn-ghost btn-xs text-red-500 hover:bg-red-500/10" @click.stop="$emit('delete-remote', r)" title="删除">✕</button>
        </div>
        <div v-if="remotesList.length === 0 && !loading" class="p-5 text-center text-base-content/60 text-xs">没有远程仓库</div>
      </div>
    </div>
  </div>

  <!-- ===== 5. Submodules Dialog ===== -->
  <div v-if="showSubmodulesDialog" class="fixed inset-0 bg-black/40 z-[900] flex items-center justify-center" @click="$emit('update:show-submodules-dialog', false)">
    <div class="max-w-lg w-full bg-base-100 rounded-xl shadow-2xl flex flex-col overflow-hidden max-h-[70vh]" @click.stop>
      <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
        <span class="font-semibold text-sm">Submodules</span>
        <button class="btn btn-ghost btn-xs" @click="$emit('update:show-submodules-dialog', false)">✕</button>
      </div>
      <div class="px-4 py-2 border-b border-base-content/10 flex gap-2">
        <button class="btn btn-primary btn-sm" @click="$emit('submodule-init-all')" :disabled="smLoading">Init All</button>
        <button class="btn btn-ghost btn-sm" @click="$emit('submodule-update-all')" :disabled="smLoading">Update All</button>
        <button class="btn btn-ghost btn-sm" @click="$emit('refresh-submodules')" :disabled="smLoading">Refresh</button>
      </div>
      <div class="flex-1 overflow-y-auto p-2">
        <div
          v-for="sm in submodulesList"
          :key="sm.name"
          class="flex items-center gap-2 px-2 py-1.5 rounded text-xs hover:bg-base-content/5 transition-colors duration-100"
        >
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" class="shrink-0 text-base-content/60">
            <rect x="2" y="2" width="9" height="9" rx="1" /><rect x="13" y="2" width="9" height="9" rx="1" />
            <rect x="2" y="13" width="9" height="9" rx="1" /><rect x="13" y="13" width="9" height="9" rx="1" />
          </svg>
          <span class="font-medium truncate max-w-[120px]" :title="sm.name">{{ sm.name }}</span>
          <span class="text-[11px] text-base-content/60 truncate max-w-[140px] cursor-pointer hover:text-primary" @click="$emit('open-submodule-path', sm.path)" :title="sm.path">{{ sm.path }}</span>
          <span class="font-mono text-[11px] text-base-content/60 shrink-0" :title="sm.hash">{{ sm.hash ? sm.hash.substring(0, 7) : '-' }}</span>
          <span class="shrink-0 text-[11px]" :class="sm.initialized ? 'text-green-500' : 'text-amber-500'">
            {{ sm.initialized ? 'Init' : 'Not init' }}
          </span>
          <div class="ml-auto shrink-0">
            <button v-if="!sm.initialized" class="btn btn-ghost btn-xs" @click.stop="$emit('submodule-init', sm.name)">Init</button>
            <button v-if="sm.initialized" class="btn btn-ghost btn-xs" @click.stop="$emit('submodule-update', sm.name)">Update</button>
          </div>
        </div>
        <div v-if="submodulesList.length === 0 && !smLoading" class="p-5 text-center text-base-content/60 text-xs">No submodules defined in .gitmodules</div>
        <div v-if="smLoading" class="p-5 text-center text-base-content/60 text-xs">Loading...</div>
      </div>
      <div class="flex justify-end gap-2 p-3 border-t border-base-content/10">
        <button class="btn btn-ghost btn-sm" @click="$emit('update:show-submodules-dialog', false)">关闭</button>
      </div>
    </div>
  </div>

  <!-- ===== 6. Compare Two Commits Dialog ===== -->
  <div v-if="showCompareCommitsDialog" class="fixed inset-0 bg-black/40 z-[900] flex items-center justify-center" @click="$emit('update:show-compare-commits-dialog', false)">
    <div class="max-w-xl w-full bg-base-100 rounded-xl shadow-2xl flex flex-col overflow-hidden max-h-[80vh]" @click.stop>
      <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
        <span class="font-semibold text-sm">比较两个提交</span>
        <button class="btn btn-ghost btn-xs" @click="$emit('update:show-compare-commits-dialog', false)">✕</button>
      </div>
      <div class="flex flex-col gap-2.5 p-3">
        <label class="text-xs font-semibold text-base-content/60">From Commit</label>
        <input :value="compareCommitFrom" @input="$emit('update:compare-commit-from', ($event.target as HTMLInputElement).value)" class="p-2 px-2.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[13px] outline-none focus:border-primary" placeholder="Commit hash or ref (e.g., HEAD~5)" spellcheck="false" />
        <label class="text-xs font-semibold text-base-content/60">To Commit</label>
        <input :value="compareCommitTo" @input="$emit('update:compare-commit-to', ($event.target as HTMLInputElement).value)" class="p-2 px-2.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[13px] outline-none focus:border-primary" placeholder="Commit hash or ref (e.g., HEAD)" spellcheck="false" />
        <button class="btn btn-ghost btn-sm self-start" @click="$emit('compare-commits')" :disabled="!compareCommitFrom || !compareCommitTo || ccLoading">比较</button>
      </div>
      <div v-if="compareCommitsDiff" class="border-t border-base-content/10">
        <pre class="bg-base-200 p-2.5 rounded font-mono text-[11px] leading-relaxed overflow-x-auto max-h-[300px] whitespace-pre-wrap text-base-content m-0">{{ compareCommitsDiff }}</pre>
      </div>
      <div class="flex justify-end gap-2 p-3 border-t border-base-content/10">
        <button class="btn btn-ghost btn-sm" @click="$emit('update:show-compare-commits-dialog', false)">关闭</button>
      </div>
    </div>
  </div>

  <!-- ===== 7. Get File at Revision Dialog ===== -->
  <div v-if="showGetFileRevisionDialog" class="fixed inset-0 bg-black/40 z-[900] flex items-center justify-center" @click="$emit('update:show-get-file-revision-dialog', false)">
    <div class="max-w-lg w-full bg-base-100 rounded-xl shadow-2xl flex flex-col overflow-hidden" @click.stop>
      <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
        <span class="font-semibold text-sm">获取指定版本的文件</span>
        <button class="btn btn-ghost btn-xs" @click="$emit('update:show-get-file-revision-dialog', false)">✕</button>
      </div>
      <div class="flex flex-col gap-2.5 p-3">
        <label class="text-xs font-semibold text-base-content/60">Commit</label>
        <input :value="getFileCommit" @input="$emit('update:get-file-commit', ($event.target as HTMLInputElement).value)" class="p-2 px-2.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[13px] outline-none focus:border-primary" placeholder="Commit hash or ref" spellcheck="false" />
        <label class="text-xs font-semibold text-base-content/60">File Path</label>
        <input :value="getFilePath" @input="$emit('update:get-file-path', ($event.target as HTMLInputElement).value)" class="p-2 px-2.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[13px] outline-none focus:border-primary" placeholder="path/to/file.txt" spellcheck="false" />
      </div>
      <div class="flex justify-end gap-2 p-3 border-t border-base-content/10">
        <button class="btn btn-ghost btn-sm" @click="$emit('update:show-get-file-revision-dialog', false)">取消</button>
        <button class="btn btn-primary btn-sm" @click="$emit('get-file-at-revision')" :disabled="!getFileCommit || !getFilePath">预览文件</button>
      </div>
    </div>
  </div>

  <!-- ===== 8. File Preview Dialog ===== -->
  <div v-if="showGetFilePreviewDialog" class="fixed inset-0 bg-black/40 z-[900] flex items-center justify-center" @click="$emit('update:show-get-file-preview-dialog', false)">
    <div class="max-w-3xl w-full bg-base-100 rounded-xl shadow-2xl flex flex-col overflow-hidden max-h-[80vh]" @click.stop>
      <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
        <span class="font-semibold text-sm">文件预览: {{ getFilePath }}</span>
        <div class="flex gap-1">
          <button class="btn btn-ghost btn-xs" @click="$emit('copy-file-content')" title="复制">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2" /><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
            </svg>
            复制
          </button>
          <button class="btn btn-ghost btn-xs" @click="$emit('update:show-get-file-preview-dialog', false)">✕</button>
        </div>
      </div>
      <pre class="flex-1 overflow-auto p-3 font-mono text-xs leading-relaxed whitespace-pre-wrap break-all text-base-content bg-base-200 m-0">{{ getFileContent }}</pre>
    </div>
  </div>

  <!-- ===== 9. Create Patch Dialog ===== -->
  <div v-if="showCreatePatchDialog" class="fixed inset-0 bg-black/40 z-[900] flex items-center justify-center" @click="$emit('update:show-create-patch-dialog', false)">
    <div class="max-w-lg w-full bg-base-100 rounded-xl shadow-2xl flex flex-col overflow-hidden" @click.stop>
      <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
        <span class="font-semibold text-sm">创建 Patch</span>
        <button class="btn btn-ghost btn-xs" @click="$emit('update:show-create-patch-dialog', false)">✕</button>
      </div>
      <div class="flex flex-col gap-2.5 p-3">
        <label class="text-xs font-semibold text-base-content/60">From (exclusive)</label>
        <input :value="patchFrom" @input="$emit('update:patch-from', ($event.target as HTMLInputElement).value)" class="p-2 px-2.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[13px] outline-none focus:border-primary" placeholder="HEAD~5 or commit hash" spellcheck="false" />
        <label class="text-xs font-semibold text-base-content/60">To (inclusive)</label>
        <input :value="patchTo" @input="$emit('update:patch-to', ($event.target as HTMLInputElement).value)" class="p-2 px-2.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[13px] outline-none focus:border-primary" placeholder="HEAD or commit hash" spellcheck="false" />
        <label class="text-xs font-semibold text-base-content/60">Output Directory</label>
        <input :value="patchOutputDir" @input="$emit('update:patch-output-dir', ($event.target as HTMLInputElement).value)" class="p-2 px-2.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[13px] outline-none focus:border-primary" placeholder="Leave empty for repo root" spellcheck="false" />
      </div>
      <div class="flex justify-end gap-2 p-3 border-t border-base-content/10">
        <button class="btn btn-ghost btn-sm" @click="$emit('update:show-create-patch-dialog', false)">取消</button>
        <button class="btn btn-primary btn-sm" @click="$emit('create-patch')" :disabled="!patchFrom || !patchTo">创建 Patch</button>
      </div>
    </div>
  </div>

  <!-- ===== 10. Apply Patch Dialog ===== -->
  <div v-if="showApplyPatchDialog" class="fixed inset-0 bg-black/40 z-[900] flex items-center justify-center" @click="$emit('update:show-apply-patch-dialog', false)">
    <div class="max-w-lg w-full bg-base-100 rounded-xl shadow-2xl flex flex-col overflow-hidden max-h-[80vh]" @click.stop>
      <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
        <span class="font-semibold text-sm">应用 Patch</span>
        <button class="btn btn-ghost btn-xs" @click="$emit('update:show-apply-patch-dialog', false)">✕</button>
      </div>
      <div class="flex flex-col gap-2.5 p-3">
        <label class="text-xs font-semibold text-base-content/60">Patch File</label>
        <div class="flex gap-2">
          <input :value="applyPatchFile" @input="$emit('update:apply-patch-file', ($event.target as HTMLInputElement).value)" class="flex-1 p-2 px-2.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[13px] outline-none focus:border-primary" placeholder="选择或输入 patch 文件路径" spellcheck="false" />
          <button class="btn btn-ghost btn-xs" @click="$emit('select-patch-file')" title="选择文件">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" /><polyline points="14 2 14 8 20 8" />
            </svg>
          </button>
        </div>
        <label class="flex items-center gap-1.5 text-xs text-base-content cursor-pointer [&_input[type=checkbox]]:accent-primary">
          <input type="checkbox" :checked="applyPatchCheck" @change="$emit('update:apply-patch-check', ($event.target as HTMLInputElement).checked)" />
          先检查 (--check, 不实际应用)
        </label>
        <label class="flex items-center gap-1.5 text-xs text-base-content cursor-pointer [&_input[type=checkbox]]:accent-primary">
          <input type="checkbox" :checked="applyPatchSign" @change="$emit('update:apply-patch-sign', ($event.target as HTMLInputElement).checked)" />
          添加 Signed-off-by (--signoff)
        </label>
        <label class="flex items-center gap-1.5 text-xs text-base-content cursor-pointer [&_input[type=checkbox]]:accent-primary">
          <input type="checkbox" :checked="applyPatch3way" @change="$emit('update:apply-patch-3way', ($event.target as HTMLInputElement).checked)" />
          三路合并 (--3way)
        </label>
      </div>
      <div v-if="applyPatchResult" class="border-t border-base-content/10">
        <pre class="p-3 font-mono text-[11px] leading-relaxed whitespace-pre-wrap overflow-auto max-h-[200px] m-0" :class="applyPatchError ? 'text-red-500 bg-red-500/5' : 'text-base-content bg-base-200'">{{ applyPatchResult }}</pre>
      </div>
      <div class="flex justify-end gap-2 p-3 border-t border-base-content/10">
        <button class="btn btn-ghost btn-sm" @click="$emit('update:show-apply-patch-dialog', false)">关闭</button>
        <button class="btn btn-primary btn-sm" @click="$emit('apply-patch')" :disabled="!applyPatchFile">应用</button>
      </div>
    </div>
  </div>

  <!-- ===== 11. Cherry-pick Multiple Dialog ===== -->
  <div v-if="showCherryPickMultiDialog" class="fixed inset-0 bg-black/40 z-[900] flex items-center justify-center" @click="$emit('update:show-cherry-pick-multi-dialog', false)">
    <div class="max-w-lg w-full bg-base-100 rounded-xl shadow-2xl flex flex-col overflow-hidden max-h-[70vh]" @click.stop>
      <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
        <span class="font-semibold text-sm">Cherry-pick 多个提交 ({{ selectedLogCommits.size }})</span>
        <button class="btn btn-ghost btn-xs" @click="$emit('update:show-cherry-pick-multi-dialog', false)">✕</button>
      </div>
      <div class="flex-1 overflow-y-auto p-2">
        <div v-for="hash in Array.from(selectedLogCommits)" :key="hash" class="flex items-center gap-2 px-2 py-1.5 rounded text-xs">
          <code class="font-mono text-[11px] text-primary bg-primary/10 px-1 py-[1px] rounded-sm shrink-0">{{ hash.substring(0, 7) }}</code>
          <span class="text-base-content/60 truncate flex-1 min-w-0">{{ getCommitMessage(hash) }}</span>
        </div>
      </div>
      <label class="flex items-center gap-1.5 text-xs text-base-content cursor-pointer [&_input[type=checkbox]]:accent-primary px-4 py-2">
        <input type="checkbox" :checked="cherryPickMultiNoCommit" @change="$emit('update:cherry-pick-multi-no-commit', ($event.target as HTMLInputElement).checked)" />
        不自动提交 (--no-commit)
      </label>
      <div class="flex justify-end gap-2 p-3 border-t border-base-content/10">
        <button class="btn btn-ghost btn-sm" @click="$emit('update:show-cherry-pick-multi-dialog', false)">取消</button>
        <button class="btn btn-primary btn-sm" @click="$emit('cherry-pick-multi')" :disabled="selectedLogCommits.size === 0 || cherryPicking">
          Cherry-pick All
        </button>
      </div>
    </div>
  </div>

  <!-- ===== 12. Git Clean Dialog ===== -->
  <div v-if="showGitCleanDialog" class="fixed inset-0 bg-black/40 z-[900] flex items-center justify-center" @click="$emit('update:show-git-clean-dialog', false)">
    <div class="max-w-lg w-full bg-base-100 rounded-xl shadow-2xl flex flex-col overflow-hidden max-h-[70vh]" @click.stop>
      <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
        <span class="font-semibold text-sm">Clean Working Tree</span>
        <button class="btn btn-ghost btn-xs" @click="$emit('update:show-git-clean-dialog', false)">✕</button>
      </div>
      <div class="flex flex-col gap-2.5 p-3">
        <label class="flex items-center gap-1.5 text-xs text-base-content cursor-pointer [&_input[type=checkbox]]:accent-primary">
          <input type="checkbox" :checked="gitCleanIncludeIgnored" @change="$emit('update:git-clean-include-ignored', ($event.target as HTMLInputElement).checked)" />
          包含忽略的文件 (-x)
        </label>
        <label class="flex items-center gap-1.5 text-xs text-base-content cursor-pointer [&_input[type=checkbox]]:accent-primary">
          <input type="checkbox" :checked="gitCleanForceDirectories" @change="$emit('update:git-clean-force-directories', ($event.target as HTMLInputElement).checked)" />
          删除目录 (-d)
        </label>
        <button class="btn btn-ghost btn-sm self-start" @click="$emit('git-clean-dry-run')" :disabled="gcLoading">Dry Run (预览)</button>
      </div>
      <div v-if="gitCleanFiles.length > 0" class="border-t border-base-content/10">
        <div class="px-4 py-2 text-xs font-semibold text-base-content/60 bg-base-200/50">将删除以下 {{ gitCleanFiles.length }} 个文件/目录:</div>
        <div v-for="f in gitCleanFiles" :key="f" class="flex items-center gap-2 px-4 py-1.5 text-xs hover:bg-base-content/5 transition-colors duration-100">
          <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" class="shrink-0 text-base-content/60">
            <polyline points="3 6 5 6 21 6" /><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
          </svg>
          <span class="truncate">{{ f }}</span>
        </div>
      </div>
      <div v-if="gitCleanFiles.length === 0 && !gcLoading" class="px-4 py-3 text-center text-base-content/60 text-xs">没有未跟踪的文件需要清理</div>
      <div class="flex justify-end gap-2 p-3 border-t border-base-content/10">
        <button class="btn btn-ghost btn-sm" @click="$emit('update:show-git-clean-dialog', false)">取消</button>
        <button class="btn btn-error btn-sm" @click="$emit('git-clean')" :disabled="gitCleanFiles.length === 0 || gcLoading">
          确认清理
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  // 1. File History
  showFileHistoryDialog: boolean
  fileHistoryFile: string
  fileHistoryData: Array<{ hash: string; date: string; message: string; author: string }>
  loading: boolean
  getAuthorName: (author: string) => string
  formatRelativeDate: (date: string) => string
  formatFullDate: (date: string) => string

  // 2. Blame
  showBlameDialog: boolean
  blameFile: string
  blameData: string

  // 3. Interactive Rebase
  showInteractiveRebaseDialog: boolean
  interactiveRebaseBase: string
  irCommits: Array<{ hash: string; message: string; action: string }>
  irSelectedIndex: number
  irLoading: boolean

  // 4. Remotes
  showRemotesDialog: boolean
  remotesList: string[]
  remoteUrls: Record<string, string>
  showAddRemoteForm: boolean
  newRemoteName: string
  newRemoteUrl: string

  // 5. Submodules
  showSubmodulesDialog: boolean
  submodulesList: Array<{ name: string; path: string; hash: string; initialized: boolean }>
  smLoading: boolean

  // 6. Compare Two Commits
  showCompareCommitsDialog: boolean
  compareCommitFrom: string
  compareCommitTo: string
  compareCommitsDiff: string
  ccLoading: boolean

  // 7. Get File at Revision
  showGetFileRevisionDialog: boolean
  getFileCommit: string
  getFilePath: string

  // 8. File Preview
  showGetFilePreviewDialog: boolean
  getFileContent: string

  // 9. Create Patch
  showCreatePatchDialog: boolean
  patchFrom: string
  patchTo: string
  patchOutputDir: string

  // 10. Apply Patch
  showApplyPatchDialog: boolean
  applyPatchFile: string
  applyPatchCheck: boolean
  applyPatchSign: boolean
  applyPatch3way: boolean
  applyPatchResult: string
  applyPatchError: boolean

  // 11. Cherry-pick Multiple
  showCherryPickMultiDialog: boolean
  selectedLogCommits: Set<string>
  cherryPickMultiNoCommit: boolean
  cherryPicking: boolean
  getCommitMessage: (hash: string) => string

  // 12. Git Clean
  showGitCleanDialog: boolean
  gitCleanIncludeIgnored: boolean
  gitCleanForceDirectories: boolean
  gitCleanFiles: string[]
  gcLoading: boolean
}>()

defineEmits<{
  // 1. File History
  'update:show-file-history-dialog': [value: boolean]

  // 2. Blame
  'update:show-blame-dialog': [value: boolean]

  // 3. Interactive Rebase
  'update:show-interactive-rebase-dialog': [value: boolean]
  'update:interactive-rebase-base': [value: string]
  'load-ir-commits': []
  'ir-move-up': [idx: number]
  'ir-move-down': [idx: number]
  'start-interactive-rebase': []

  // 4. Remotes
  'update:show-remotes-dialog': [value: boolean]
  'update:show-add-remote-form': [value: boolean]
  'update:new-remote-name': [value: string]
  'update:new-remote-url': [value: string]
  'add-remote': []
  'fetch-remote': [name: string]
  'delete-remote': [name: string]

  // 5. Submodules
  'update:show-submodules-dialog': [value: boolean]
  'submodule-init-all': []
  'submodule-update-all': []
  'refresh-submodules': []
  'submodule-init': [name: string]
  'submodule-update': [name: string]
  'open-submodule-path': [path: string]

  // 6. Compare Two Commits
  'update:show-compare-commits-dialog': [value: boolean]
  'update:compare-commit-from': [value: string]
  'update:compare-commit-to': [value: string]
  'compare-commits': []

  // 7. Get File at Revision
  'update:show-get-file-revision-dialog': [value: boolean]
  'update:get-file-commit': [value: string]
  'update:get-file-path': [value: string]
  'get-file-at-revision': []

  // 8. File Preview
  'update:show-get-file-preview-dialog': [value: boolean]
  'copy-file-content': []

  // 9. Create Patch
  'update:show-create-patch-dialog': [value: boolean]
  'update:patch-from': [value: string]
  'update:patch-to': [value: string]
  'update:patch-output-dir': [value: string]
  'create-patch': []

  // 10. Apply Patch
  'update:show-apply-patch-dialog': [value: boolean]
  'update:apply-patch-file': [value: string]
  'update:apply-patch-check': [value: boolean]
  'update:apply-patch-sign': [value: boolean]
  'update:apply-patch-3way': [value: boolean]
  'select-patch-file': []
  'apply-patch': []

  // 11. Cherry-pick Multiple
  'update:show-cherry-pick-multi-dialog': [value: boolean]
  'update:cherry-pick-multi-no-commit': [value: boolean]
  'cherry-pick-multi': []

  // 12. Git Clean
  'update:show-git-clean-dialog': [value: boolean]
  'update:git-clean-include-ignored': [value: boolean]
  'update:git-clean-force-directories': [value: boolean]
  'git-clean-dry-run': []
  'git-clean': []
}>()
</script>
