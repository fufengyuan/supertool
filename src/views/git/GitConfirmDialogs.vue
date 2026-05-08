<template>
  <!-- ===== Cherry-pick 确认对话框 ===== -->
  <div
    v-if="cherryPickTarget"
    class="fixed inset-0 bg-black/40 z-[900] flex items-center justify-center"
    @click="$emit('update:cherry-pick-target', null)"
  >
    <div class="max-w-md w-full bg-base-100 rounded-xl shadow-2xl p-6" @click.stop>
      <h3 class="text-lg font-semibold m-0 mb-4">Cherry-pick</h3>
      <p class="m-0 mb-4 text-[13px] leading-relaxed">
        将提交
        <code class="bg-primary/10 text-primary px-1 py-[1px] rounded-sm font-mono text-xs">{{ cherryPickTarget.substring(0, 7) }}</code>
        应用到当前分支？
      </p>
      <div class="flex justify-end gap-2">
        <button class="btn btn-ghost btn-sm" @click="$emit('update:cherry-pick-target', null)">取消</button>
        <button class="btn btn-primary btn-sm" @click="$emit('cherry-pick')" :disabled="cherryPicking">Cherry-pick</button>
      </div>
    </div>
  </div>

  <!-- ===== Revert 确认对话框 ===== -->
  <div
    v-if="revertTarget"
    class="fixed inset-0 bg-black/40 z-[900] flex items-center justify-center"
    @click="$emit('update:revert-target', null)"
  >
    <div class="max-w-md w-full bg-base-100 rounded-xl shadow-2xl p-6" @click.stop>
      <h3 class="text-lg font-semibold m-0 mb-4">Revert Commit</h3>
      <p class="m-0 mb-4 text-[13px] leading-relaxed">
        创建一个新的提交来撤销
        <code class="bg-primary/10 text-primary px-1 py-[1px] rounded-sm font-mono text-xs">{{ revertTarget.substring(0, 7) }}</code>
        的变更？
      </p>
      <div class="flex justify-end gap-2">
        <button class="btn btn-ghost btn-sm" @click="$emit('update:revert-target', null)">取消</button>
        <button class="btn btn-primary btn-sm" @click="$emit('revert')" :disabled="reverting">Revert</button>
      </div>
    </div>
  </div>

  <!-- ===== Delete Remote Branch 确认对话框 ===== -->
  <div
    v-if="deleteRemoteBranchTarget"
    class="fixed inset-0 bg-black/40 z-[900] flex items-center justify-center"
    @click="$emit('update:delete-remote-branch-target', null)"
  >
    <div class="max-w-md w-full bg-base-100 rounded-xl shadow-2xl p-6" @click.stop>
      <h3 class="text-lg font-semibold m-0 mb-4">删除远程分支</h3>
      <p class="m-0 mb-4 text-[13px] leading-relaxed">
        确定要删除远程分支
        <code class="bg-primary/10 text-primary px-1 py-[1px] rounded-sm font-mono text-xs">{{ deleteRemoteBranchTarget }}</code>？
        此操作不可撤销！
      </p>
      <div class="flex justify-end gap-2">
        <button class="btn btn-ghost btn-sm" @click="$emit('update:delete-remote-branch-target', null)">取消</button>
        <button class="btn btn-error btn-sm" @click="$emit('delete-remote-branch')" :disabled="deletingBranch">删除</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  cherryPickTarget: string | null
  revertTarget: string | null
  deleteRemoteBranchTarget: string | null
  cherryPicking: boolean
  reverting: boolean
  deletingBranch: boolean
}>()

defineEmits<{
  'update:cherry-pick-target': [value: string | null]
  'update:revert-target': [value: string | null]
  'update:delete-remote-branch-target': [value: string | null]
  'cherry-pick': []
  'revert': []
  'delete-remote-branch': []
}>()
</script>
