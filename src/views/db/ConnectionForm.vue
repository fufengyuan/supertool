<template>
  <div class="fixed inset-0 flex items-center justify-center z-50 bg-black/40" @click="$emit('close')">
    <div class="bg-base-100 rounded-2xl w-[520px] max-h-[85vh] overflow-y-auto shadow-2xl" @click.stop>
      <div class="flex items-center justify-between p-5 border-b border-base-content/10 sticky top-0 bg-base-100 z-10 rounded-t-2xl">
        <h3 class="m-0 text-lg font-semibold">{{ isEditing ? '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg> 编辑连接' : '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><rect x="2" y="3" width="20" height="18" rx="2" ry="2"/><line x1="2" y1="9" x2="22" y2="9"/></svg> 添加数据库连接' }}</h3>
        <button @click="$emit('close')" class="btn btn-ghost btn-sm btn-square">×</button>
      </div>
      <div class="p-6">
        <div class="flex gap-4">
          <div class="flex-1 min-w-0">
            <label class="label"><span class="label-text">连接名称 <span class="text-error">*</span></span></label>
            <input v-model="localForm.name" class="input input-bordered w-full" placeholder="我的数据库" />
          </div>
          <div class="flex-1 min-w-0">
            <label class="label"><span class="label-text">数据库类型 <span class="text-error">*</span></span></label>
            <select v-model="localForm.type" class="select select-bordered w-full">
              <option value="mysql">MySQL</option>
              <option value="postgresql">PostgreSQL</option>
              <option value="redis">Redis</option>
              <option value="sqlite">SQLite</option>
            </select>
          </div>
        </div>

        <!-- SQLite uses path instead of host/port -->
        <template v-if="localForm.type === 'sqlite'">
          <div class="mt-4">
            <label class="label"><span class="label-text">数据库文件路径</span></label>
            <input v-model="localForm.path" class="input input-bordered w-full" placeholder="/path/to/database.db" />
          </div>
        </template>

        <template v-else>
          <div class="flex gap-4 mt-4">
            <div class="flex-1 min-w-0">
              <label class="label"><span class="label-text">主机地址</span></label>
              <input v-model="localForm.host" class="input input-bordered w-full" placeholder="127.0.0.1" />
            </div>
            <div class="flex-1 min-w-0">
              <label class="label"><span class="label-text">端口</span></label>
              <input v-model.number="localForm.port" type="number" class="input input-bordered w-full" :placeholder="defaultPort" />
            </div>
          </div>

          <div class="flex gap-4">
            <div class="flex-1 min-w-0" v-if="localForm.type !== 'redis'">
              <label class="label"><span class="label-text">用户名</span></label>
              <input v-model="localForm.user" class="input input-bordered w-full" placeholder="root" />
            </div>
            <div class="flex-1 min-w-0" :class="{ 'basis-full': localForm.type === 'redis' }">
              <label class="label"><span class="label-text">密码</span></label>
              <input v-model="localForm.password" type="password" class="input input-bordered w-full" autocomplete="off" :placeholder="localForm.type === 'redis' ? '无密码留空' : '密码'" />
            </div>
          </div>

          <div class="mt-4" v-if="localForm.type !== 'redis'">
            <label class="label"><span class="label-text">数据库名</span></label>
            <input v-model="localForm.database" class="input input-bordered w-full" placeholder="database_name" />
          </div>

          <div class="mt-4" v-if="localForm.type === 'redis'">
            <label class="label"><span class="label-text">数据库索引</span></label>
            <input v-model.number="localForm.dbIndex" type="number" class="input input-bordered w-full" placeholder="0" />
          </div>

          <div class="my-2 border-t border-base-content/10"></div>

          <label class="flex items-center gap-2 p-2 bg-amber-50/60 dark:bg-amber-900/10 rounded-lg border border-amber-200/30 dark:border-amber-500/15 cursor-pointer">
            <input v-model="localForm.requiresApproval" type="checkbox" class="checkbox checkbox-sm" />
            <span class="flex flex-col gap-0.5">
              <span>🔒 SQL 执行审核</span>
              <span class="text-xs text-base-content/60">开启后 CLI 无法执行 SQL，GUI 执行前需要确认</span>
            </span>
          </label>
        </template>
      </div>

      <div class="flex justify-end gap-3 p-4 border-t border-base-content/10">
        <button @click="$emit('test', localForm)" class="btn btn-ghost" :disabled="testing">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
            <polyline points="22 4 12 14.01 9 11.01" />
          </svg>
          {{ testing ? '测试中...' : '测试连接' }}
        </button>
        <button @click="$emit('close')" class="btn btn-ghost">取消</button>
        <button @click="$emit('save', localForm)" class="btn btn-primary">保存</button>
      </div>

      <div v-if="testResult" :class="testResult.success ? 'alert alert-success' : 'alert alert-error'"
        class="mx-6 mb-5 rounded-lg">
        <span>{{ testResult.success ? '✅ 连接成功！' : '❌ 连接失败: ' + testResult.error }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, computed, watch, onUnmounted, nextTick, onMounted } from 'vue'

interface TestResult {
  success: boolean
  error?: string
}

const props = defineProps<{
  form: DBConfig
  isEditing: boolean
  testResult: TestResult | null
  testing: boolean
}>()

const emit = defineEmits<{
  close: []
  save: [form: DBConfig]
  test: [form: DBConfig]
  'update:form': [form: DBConfig]
}>()

// Local reactive wrapper to avoid mutating props directly
const localForm = ref<DBConfig>({ ...props.form })

// Sync localForm when parent updates the form prop (e.g. switching edit target)
watch(() => props.form, (newVal) => {
  localForm.value = { ...newVal }
})

// Emit changes to parent — 用 nextTick 防抖，避免每个字符触发一次 emit
let emitTimer: ReturnType<typeof setTimeout> | null = null
watch(localForm, (newVal) => {
  if (emitTimer) clearTimeout(emitTimer)
  emitTimer = setTimeout(() => {
    emit('update:form', { ...newVal })
  }, 16) // ~1 frame, batches rapid input events
}, { deep: true })

onUnmounted(() => {
  if (emitTimer) clearTimeout(emitTimer)
})

const defaultPort = computed(() => {
  const map: Record<string, string> = {
    mysql: '3306',
    postgresql: '5432',
    redis: '6379'
  }
  return map[localForm.value.type] || ''
})
</script>
