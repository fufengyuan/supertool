<template>
  <!-- ===== Stash Save Dialog ===== -->
  <div
    v-if="showStashSaveDialog"
    class="fixed inset-0 bg-black/40 z-[900] flex items-center justify-center"
    @click="$emit('update:show-stash-save-dialog', false)"
  >
    <div class="max-w-lg w-full bg-base-100 rounded-xl shadow-2xl p-6" @click.stop>
      <h3 class="text-lg font-semibold m-0 mb-4">保存暂存</h3>
      <div class="flex flex-col gap-2.5 mb-4">
        <label class="text-xs font-semibold text-base-content/60">描述</label>
        <input
          :value="stashSaveMessage"
          class="p-2 px-2.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[13px] outline-none focus:border-primary"
          placeholder="输入 stash 描述..."
          @input="$emit('update:stash-save-message', ($event.target as HTMLInputElement).value)"
          @keydown.enter="$emit('stash-save')"
          spellcheck="false"
          ref="stashSaveInput"
        />
        <label class="flex items-center gap-1.5 text-xs text-base-content cursor-pointer [&_input[type=checkbox]]:accent-primary">
          <input
            type="checkbox"
            :checked="stashIncludeUntracked"
            @change="$emit('update:stash-include-untracked', ($event.target as HTMLInputElement).checked)"
          />
          包含未跟踪文件
        </label>
      </div>
      <div class="flex justify-end gap-2">
        <button class="btn btn-ghost btn-sm" @click="$emit('update:show-stash-save-dialog', false)">取消</button>
        <button class="btn btn-primary btn-sm" @click="$emit('stash-save')" :disabled="!stashSaveMessage.trim()">保存</button>
      </div>
    </div>
  </div>

  <!-- ===== Tag Create Dialog ===== -->
  <div
    v-if="showCreateTagDialog"
    class="fixed inset-0 bg-black/40 z-[900] flex items-center justify-center"
    @click="$emit('update:show-create-tag-dialog', false)"
  >
    <div class="max-w-lg w-full bg-base-100 rounded-xl shadow-2xl p-6" @click.stop>
      <h3 class="text-lg font-semibold m-0 mb-4">创建标签</h3>
      <div class="flex flex-col gap-2.5 mb-4">
        <label class="text-xs font-semibold text-base-content/60">Tag 名称</label>
        <input
          :value="newTagName"
          class="p-2 px-2.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[13px] outline-none focus:border-primary"
          placeholder="v1.0.0"
          @input="$emit('update:new-tag-name', ($event.target as HTMLInputElement).value)"
          @keydown.enter="$emit('create-tag')"
          spellcheck="false"
        />
        <label class="text-xs font-semibold text-base-content/60">指向提交</label>
        <input
          :value="newTagCommit"
          class="p-2 px-2.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[13px] outline-none focus:border-primary"
          placeholder="HEAD"
          @input="$emit('update:new-tag-commit', ($event.target as HTMLInputElement).value)"
          spellcheck="false"
        />
        <label class="text-xs font-semibold text-base-content/60">消息 (可选)</label>
        <input
          :value="newTagMessage"
          class="p-2 px-2.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[13px] outline-none focus:border-primary"
          placeholder="Tag 描述..."
          @input="$emit('update:new-tag-message', ($event.target as HTMLInputElement).value)"
          spellcheck="false"
        />
      </div>
      <div class="flex justify-end gap-2">
        <button class="btn btn-ghost btn-sm" @click="$emit('update:show-create-tag-dialog', false)">取消</button>
        <button class="btn btn-primary btn-sm" @click="$emit('create-tag')" :disabled="!newTagName.trim()">创建</button>
      </div>
    </div>
  </div>

  <!-- ===== Tag Manage Dialog ===== -->
  <div
    v-if="showTagsDialog"
    class="fixed inset-0 bg-black/40 z-[900] flex items-center justify-center"
    @click="$emit('update:show-tags-dialog', false)"
  >
    <div class="max-w-md w-full bg-base-100 rounded-xl shadow-2xl flex flex-col overflow-hidden max-h-[70vh]" @click.stop>
      <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
        <span class="font-semibold text-sm">标签管理</span>
        <button class="btn btn-ghost btn-xs" @click="$emit('update:show-tags-dialog', false)"><SvgIcon name="x" size="14" class="inline-block" /></button>
      </div>
      <div class="px-4 py-2 border-b border-base-content/10 flex gap-2">
        <button class="btn btn-primary btn-sm" @click="$emit('create-tag-open')">
          <SvgIcon name="plus" size="14" />
          新建标签
        </button>
        <button class="btn btn-ghost btn-sm" @click="$emit('create-branch-from-tag')" :disabled="!selectedTagForBranch">
          <SvgIcon name="gitBranch" size="14" />
          从标签创建分支
        </button>
      </div>
      <div class="flex-1 overflow-y-auto p-2">
        <div
          v-for="tag in tagsList"
          :key="tag.name"
          class="flex items-center gap-2 px-2 py-1.5 rounded text-[13px] hover:bg-base-200/50 cursor-pointer"
          :class="{ 'bg-primary/10': selectedTagForBranch === tag.name }"
          @click="$emit('update:selected-tag-for-branch', tag.name)"
        >
          <SvgIcon name="tag" size="14" class="text-base-content/60" />
          <span class="flex-1 font-mono">{{ tag.name }}</span>
          <span class="text-[11px] text-base-content/60 font-mono" :title="tag.commit">{{ tag.commit?.substring(0, 7) || '' }}</span>
          <button class="btn btn-ghost btn-xs text-red-500 hover:text-red-600" @click.stop="$emit('delete-tag', tag.name)" title="删除"><SvgIcon name="x" size="14" class="inline-block" /></button>
        </div>
        <div v-if="tagsList.length === 0 && !loading" class="p-5 text-center text-base-content/60">没有标签</div>
      </div>
    </div>
  </div>

  <!-- ===== Compare Branches Dialog ===== -->
  <div
    v-if="showCompareBranchesDialog"
    class="fixed inset-0 bg-black/40 z-[900] flex items-center justify-center"
    @click="$emit('update:show-compare-branches-dialog', false)"
  >
    <div class="max-w-sm w-full bg-base-100 rounded-xl shadow-2xl flex flex-col overflow-hidden" @click.stop>
      <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
        <span class="font-semibold text-sm">比较分支</span>
        <button class="btn btn-ghost btn-xs" @click="$emit('update:show-compare-branches-dialog', false)"><SvgIcon name="x" size="14" class="inline-block" /></button>
      </div>
      <div class="flex flex-col gap-2.5 mb-4 px-4 pt-3">
        <div class="flex items-center gap-2 mb-2">
          <label class="text-xs font-semibold text-base-content/60 min-w-[80px]">当前分支</label>
          <span class="text-primary font-medium font-mono">{{ currentBranch }}</span>
        </div>
        <div class="flex items-center gap-2 mb-2">
          <label class="text-xs font-semibold text-base-content/60 min-w-[80px]">对比分支</label>
          <select
            :value="compareBranchTarget"
            class="p-2 px-2.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[13px] outline-none focus:border-primary flex-1"
            @change="(e) => { $emit('update:compare-branch-target', (e.target as HTMLSelectElement).value); $emit('compare-branches') }"
          >
            <option value="">选择分支...</option>
            <option v-for="b in localBranches" :key="b.name" :value="b.name" :disabled="b.name === currentBranch">{{ b.name }}</option>
          </select>
        </div>
      </div>
      <div v-if="compareResult" class="px-4 py-3 border-t border-b border-base-content/10">
        <div class="flex items-center gap-2 py-1 text-[13px]">
          <span class="text-base-content/60 min-w-[80px]">变更文件:</span>
          <span class="font-semibold">{{ compareResult.changedFiles }}</span>
        </div>
        <div class="flex items-center gap-2 py-1 text-[13px]">
          <span class="text-base-content/60 min-w-[80px]">新增行数:</span>
          <span class="font-semibold text-green-500">{{ compareResult.insertions }}</span>
        </div>
        <div class="flex items-center gap-2 py-1 text-[13px]">
          <span class="text-base-content/60 min-w-[80px]">删除行数:</span>
          <span class="font-semibold text-red-500">{{ compareResult.deletions }}</span>
        </div>
      </div>
      <div class="flex justify-end gap-2 px-4 py-3">
        <button class="btn btn-ghost btn-sm" @click="$emit('update:show-compare-branches-dialog', false)">关闭</button>
      </div>
    </div>
  </div>

  <!-- ===== Push Dialog ===== -->
  <div
    v-if="showPushDialog"
    class="fixed inset-0 bg-black/40 z-[900] flex items-center justify-center"
    @click="$emit('update:show-push-dialog', false)"
  >
    <div class="max-w-md w-full bg-base-100 rounded-xl shadow-2xl flex flex-col overflow-hidden max-h-[70vh]" @click.stop>
      <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
        <span class="font-semibold text-sm">推送到远程</span>
        <button class="btn btn-ghost btn-xs" @click="$emit('update:show-push-dialog', false)"><SvgIcon name="x" size="14" class="inline-block" /></button>
      </div>
      <div class="flex flex-col gap-2.5 mb-4 px-4 pt-3">
        <label class="text-xs font-semibold text-base-content/60">远程仓库</label>
        <select
          :value="pushRemote"
          class="p-2 px-2.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[13px] outline-none focus:border-primary"
          @change="$emit('update:push-remote', ($event.target as HTMLSelectElement).value)"
        >
          <option v-for="r in remotesList" :key="r" :value="r">{{ r }}</option>
        </select>
        <label class="text-xs font-semibold text-base-content/60">目标分支</label>
        <input
          :value="pushBranch"
          class="p-2 px-2.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[13px] outline-none focus:border-primary"
          placeholder="默认当前分支"
          @input="$emit('update:push-branch', ($event.target as HTMLInputElement).value)"
        />
        <label class="flex items-center gap-1.5 text-xs text-base-content cursor-pointer [&_input[type=checkbox]]:accent-primary">
          <input
            type="checkbox"
            :checked="pushForce"
            @change="$emit('update:push-force', ($event.target as HTMLInputElement).checked)"
          />
          强制推送 (--force)
        </label>
        <label class="flex items-center gap-1.5 text-xs text-base-content cursor-pointer [&_input[type=checkbox]]:accent-primary">
          <input
            type="checkbox"
            :checked="pushSetUpstream"
            @change="$emit('update:push-set-upstream', ($event.target as HTMLInputElement).checked)"
          />
          设置上游分支 (--set-upstream)
        </label>
      </div>
      <div v-if="pushUnpushedCommits.length > 0" class="max-h-[200px] overflow-y-auto border-t border-base-content/10">
        <div class="px-4 py-2 text-xs font-semibold text-base-content/60 bg-base-200 sticky top-0">待推送提交 ({{ pushUnpushedCommits.length }})</div>
        <div v-for="c in pushUnpushedCommits" :key="c.hash" class="flex items-center gap-2 px-4 py-1 text-xs hover:bg-base-200/50">
          <code class="font-mono text-[11px] text-primary bg-primary/10 px-1 py-[1px] rounded-sm">{{ c?.hash?.substring(0, 7) || '-' }}</code>
          <span class="text-base-content/60 truncate block max-w-[560px]">{{ c.message }}</span>
        </div>
      </div>
      <div class="flex justify-end gap-2 px-4 py-3">
        <button class="btn btn-ghost btn-sm" @click="$emit('update:show-push-dialog', false)">取消</button>
        <button class="btn btn-primary btn-sm" @click="$emit('push-with-options')" :disabled="pushing">
          <SvgIcon v-if="pushing" name="refresh" :size="14" class="animate-spin h-3.5 w-3.5" />
          {{ pushForce ? '强制推送' : '推送' }}
        </button>
      </div>
    </div>
  </div>

  <!-- ===== Pull Dialog ===== -->
  <div
    v-if="showPullDialog"
    class="fixed inset-0 bg-black/40 z-[900] flex items-center justify-center"
    @click="$emit('update:show-pull-dialog', false)"
  >
    <div class="max-w-sm w-full bg-base-100 rounded-xl shadow-2xl flex flex-col overflow-hidden" @click.stop>
      <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
        <span class="font-semibold text-sm">从远程拉取</span>
        <button class="btn btn-ghost btn-xs" @click="$emit('update:show-pull-dialog', false)"><SvgIcon name="x" size="14" class="inline-block" /></button>
      </div>
      <div class="flex flex-col gap-2.5 mb-4 px-4 pt-3">
        <label class="text-xs font-semibold text-base-content/60">远程仓库</label>
        <select
          :value="pullRemote"
          class="p-2 px-2.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[13px] outline-none focus:border-primary"
          @change="(e) => $emit('update:pull-remote', (e.target as HTMLSelectElement).value)"
        >
          <option v-for="r in remotesList" :key="r" :value="r">{{ r }}</option>
        </select>
        <label class="text-xs font-semibold text-base-content/60">来源分支</label>
        <input
          :value="pullBranch"
          class="p-2 px-2.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[13px] outline-none focus:border-primary"
          placeholder="默认当前分支"
          @input="$emit('update:pull-branch', ($event.target as HTMLInputElement).value)"
        />
        <label class="text-xs font-semibold text-base-content/60">拉取方式</label>
        <select
          :value="pullRebase"
          class="p-2 px-2.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[13px] outline-none focus:border-primary"
          @change="(e) => $emit('update:pull-rebase', (e.target as HTMLSelectElement).value === 'true')"
        >
          <option :value="false">合并</option>
          <option :value="true">变基</option>
        </select>
        <label class="flex items-center gap-1.5 text-xs text-base-content cursor-pointer [&_input[type=checkbox]]:accent-primary">
          <input
            type="checkbox"
            :checked="pullAutoStash"
            @change="$emit('update:pull-auto-stash', ($event.target as HTMLInputElement).checked)"
          />
          自动暂存未提交的变更
        </label>
      </div>
      <div class="flex justify-end gap-2 px-4 py-3">
        <button class="btn btn-ghost btn-sm" @click="$emit('update:show-pull-dialog', false)">取消</button>
        <button class="btn btn-primary btn-sm" @click="$emit('pull-with-options')" :disabled="pulling">
          <SvgIcon v-if="pulling" name="refresh" :size="14" class="animate-spin h-3.5 w-3.5" />
>拉取<
        </button>
      </div>
    </div>
  </div>

  <!-- ===== Rebase Dialog ===== -->
  <div
    v-if="showRebaseDialog"
    class="fixed inset-0 bg-black/40 z-[900] flex items-center justify-center"
    @click="$emit('update:show-rebase-dialog', false)"
  >
    <div class="max-w-md w-full bg-base-100 rounded-xl shadow-2xl p-6" @click.stop>
      <h3 class="text-lg font-semibold m-0 mb-4">变基</h3>
      <div v-if="rebaseInProgress" class="text-center">
        <p class="m-0 mb-4 text-[13px] leading-relaxed">Rebase 进行中，存在冲突。请解决冲突后选择：</p>
        <div class="flex justify-center gap-2">
          <button class="btn btn-ghost btn-sm" @click="$emit('rebase-abort')">中止变基</button>
          <button class="btn btn-primary btn-sm" @click="$emit('rebase-continue')">继续</button>
        </div>
      </div>
      <div v-else>
        <div class="flex flex-col gap-2.5 mb-4">
          <label class="text-xs font-semibold text-base-content/60">变基到</label>
          <select
            :value="rebaseTarget"
            class="p-2 px-2.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[13px] outline-none focus:border-primary"
            @change="$emit('update:rebase-target', ($event.target as HTMLSelectElement).value)"
          >
            <option value="">选择分支...</option>
            <option v-for="b in localBranches" :key="b.name" :value="b.name" :disabled="b.name === currentBranch">{{ b.name }}</option>
          </select>
        </div>
        <div class="flex justify-end gap-2">
          <button class="btn btn-ghost btn-sm" @click="$emit('update:show-rebase-dialog', false)">取消</button>
          <button class="btn btn-primary btn-sm" @click="$emit('rebase')" :disabled="!rebaseTarget || rebasing">开始变基</button>
        </div>
      </div>
    </div>
  </div>

  <!-- ===== Reset Dialog ===== -->
  <div
    v-if="showResetDialog"
    class="fixed inset-0 bg-black/40 z-[900] flex items-center justify-center"
    @click="$emit('update:show-reset-dialog', false)"
  >
    <div class="max-w-sm w-full bg-base-100 rounded-xl shadow-2xl p-6" @click.stop>
      <h3 class="text-lg font-semibold m-0 mb-4">重置到提交</h3>
      <div class="flex flex-col gap-2.5 mb-4">
        <label class="text-xs font-semibold text-base-content/60">提交 Hash / 引用</label>
        <input
          :value="resetTarget"
          class="p-2 px-2.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[13px] outline-none focus:border-primary"
          placeholder="HEAD~1, abc1234, etc."
          @input="$emit('update:reset-target', ($event.target as HTMLInputElement).value)"
          @keydown.enter="$emit('reset')"
          spellcheck="false"
        />
        <label class="text-xs font-semibold text-base-content/60">重置模式</label>
        <select
          :value="resetMode"
          class="p-2 px-2.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[13px] outline-none focus:border-primary"
          @change="$emit('update:reset-mode', ($event.target as HTMLSelectElement).value)"
        >
          <option value="soft">Soft — 保留变更到暂存区</option>
          <option value="mixed">Mixed — 保留变更到工作区 (默认)</option>
          <option value="hard">Hard — 丢弃所有变更</option>
        </select>
        <div v-if="resetMode === 'hard'" class="mt-2 p-2.5 bg-red-500/10 border border-red-500/30 rounded text-red-500 text-xs">
          <SvgIcon name="alertTriangle" size="14" class="inline-block align-text-bottom" /> Hard Reset 将丢弃所有未提交的变更，此操作不可撤销！
        </div>
      </div>
      <div class="flex justify-end gap-2">
        <button class="btn btn-ghost btn-sm" @click="$emit('update:show-reset-dialog', false)">取消</button>
        <button class="btn btn-primary btn-sm" @click="$emit('reset')" :disabled="!resetTarget.trim() || resetting">Reset</button>
      </div>
    </div>
  </div>

  <!-- ===== Amend Dialog ===== -->
  <div
    v-if="showAmendDialog"
    class="fixed inset-0 bg-black/40 z-[900] flex items-center justify-center"
    @click="$emit('update:show-amend-dialog', false)"
  >
    <div class="max-w-sm w-full bg-base-100 rounded-xl shadow-2xl p-6" @click.stop>
      <h3 class="text-lg font-semibold m-0 mb-4">修改最后一次提交</h3>
      <div class="flex flex-col gap-2.5 mb-4">
        <label class="text-xs font-semibold text-base-content/60">新的提交信息</label>
        <textarea
          :value="amendMessage"
          class="w-full p-2.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[13px] outline-none focus:border-primary resize-y"
          rows="4"
          placeholder="输入新的提交信息 (留空则使用原信息)"
          @input="$emit('update:amend-message', ($event.target as HTMLTextAreaElement).value)"
          spellcheck="false"
        />
        <label class="flex items-center gap-1.5 text-xs text-base-content cursor-pointer [&_input[type=checkbox]]:accent-primary">
          <input
            type="checkbox"
            :checked="amendNoEdit"
            @change="$emit('update:amend-no-edit', ($event.target as HTMLInputElement).checked)"
          />
          不修改提交信息 (仅添加变更)
        </label>
      </div>
      <div class="flex justify-end gap-2">
        <button class="btn btn-ghost btn-sm" @click="$emit('update:show-amend-dialog', false)">取消</button>
        <button class="btn btn-primary btn-sm" @click="$emit('amend')" :disabled="amending">修改</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'

interface CompareResult {
  changedFiles: number
  insertions: number
  deletions: number
}

interface Tag {
  name: string
  commit: string
}

interface Branch {
  name: string
}

interface UnpushedCommit {
  hash: string
  message: string
}

defineProps<{
  showStashSaveDialog: boolean
  stashSaveMessage: string
  stashIncludeUntracked: boolean

  showCreateTagDialog: boolean
  newTagName: string
  newTagCommit: string
  newTagMessage: string

  showTagsDialog: boolean
  tagsList: Tag[]
  selectedTagForBranch: string | null
  loading: boolean

  showCompareBranchesDialog: boolean
  localBranches: Branch[]
  currentBranch: string
  compareBranchTarget: string
  compareResult: CompareResult | null

  showPushDialog: boolean
  remotesList: string[]
  pushRemote: string
  pushBranch: string
  pushForce: boolean
  pushSetUpstream: boolean
  pushUnpushedCommits: UnpushedCommit[]
  pushing: boolean

  showPullDialog: boolean
  pullRemote: string
  pullBranch: string
  pullRebase: boolean
  pullAutoStash: boolean
  pulling: boolean

  showRebaseDialog: boolean
  rebaseTarget: string
  rebasing: boolean
  rebaseInProgress: boolean

  showResetDialog: boolean
  resetTarget: string
  resetMode: string
  resetting: boolean

  showAmendDialog: boolean
  amendMessage: string
  amendNoEdit: boolean
  amending: boolean
}>()

defineEmits<{
  'update:show-stash-save-dialog': [value: boolean]
  'update:stash-save-message': [value: string]
  'update:stash-include-untracked': [value: boolean]
  'stash-save': []

  'update:show-create-tag-dialog': [value: boolean]
  'update:new-tag-name': [value: string]
  'update:new-tag-commit': [value: string]
  'update:new-tag-message': [value: string]
  'create-tag': []

  'update:show-tags-dialog': [value: boolean]
  'update:selected-tag-for-branch': [value: string | null]
  'create-tag-open': []
  'create-branch-from-tag': []
  'delete-tag': [name: string]

  'update:show-compare-branches-dialog': [value: boolean]
  'update:compare-branch-target': [value: string]
  'compare-branches': []

  'update:show-push-dialog': [value: boolean]
  'update:push-remote': [value: string]
  'update:push-branch': [value: string]
  'update:push-force': [value: boolean]
  'update:push-set-upstream': [value: boolean]
  'push-with-options': []

  'update:show-pull-dialog': [value: boolean]
  'update:pull-remote': [value: string]
  'update:pull-branch': [value: string]
  'update:pull-rebase': [value: boolean]
  'update:pull-auto-stash': [value: boolean]
  'pull-with-options': []

  'update:show-rebase-dialog': [value: boolean]
  'update:rebase-target': [value: string]
  'rebase': []
  'rebase-abort': []
  'rebase-continue': []

  'update:show-reset-dialog': [value: boolean]
  'update:reset-target': [value: string]
  'update:reset-mode': [value: string]
  'reset': []

  'update:show-amend-dialog': [value: boolean]
  'update:amend-message': [value: string]
  'update:amend-no-edit': [value: boolean]
  'amend': []
}>()
</script>
