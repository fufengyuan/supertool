<template>
  <div class="fixed inset-0 flex items-center justify-center z-50 bg-black/40" @click="$emit('close')">
    <div class="bg-base-100 rounded-2xl w-[520px] max-h-[85vh] overflow-y-auto shadow-2xl" @click.stop>
      <div class="flex items-center justify-between p-5 border-b border-base-content/10 sticky top-0 bg-base-100 z-10 rounded-t-2xl">
        <h3 class="m-0 text-lg font-semibold"><template v-if="isEditing"><SvgIcon name="pencil" size="14" class="inline-block" /> 编辑连接</template><template v-else><SvgIcon name="archive" size="14" class="inline-block align-text-bottom" /> 添加数据库连接</template></h3>
        <button @click="$emit('close')" class="btn btn-ghost btn-sm btn-square"><SvgIcon name="x" size="16" /></button>
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
            <div class="flex gap-2">
              <input v-model="localForm.path" class="input input-bordered flex-1" placeholder="/path/to/database.db" />
              <button @click="pickSqliteFile" class="btn btn-ghost border border-base-content/10 shrink-0" title="选择文件">
                <SvgIcon name="folder" size="14" /> 选择
              </button>
            </div>
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
              <div class="relative">
                <input v-model="localForm.password" :type="showPassword ? 'text' : 'password'" class="input input-bordered w-full pr-10" autocomplete="off" :placeholder="localForm.type === 'redis' ? '无密码留空' : '密码'" />
                <button
                  type="button"
                  class="absolute right-2.5 top-1/2 -translate-y-1/2 text-base-content/50 hover:text-base-content"
                  :title="showPassword ? '隐藏密码' : '显示密码'"
                  @click="showPassword = !showPassword"
                >
                  <SvgIcon :name="showPassword ? 'eyeOff' : 'eye'" :size="16" />
                </button>
              </div>
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

          <label class="flex items-center gap-2 p-2 bg-warning/5 rounded-lg border border-warning/20 cursor-pointer">
            <input v-model="localForm.requiresApproval" type="checkbox" class="checkbox checkbox-sm" />
            <span class="flex flex-col gap-0.5">
              <span class="flex items-center gap-1.5"><SvgIcon name="lock" size="14" /> SQL 执行审核</span>
              <span class="text-xs text-base-content/60">开启后 CLI 无法执行 SQL，GUI 执行前需要确认</span>
            </span>
          </label>
        </template>
      </div>

      <div class="flex justify-end gap-2 p-4 border-t border-base-content/10">
        <button @click="$emit('test', localForm)" class="btn btn-ghost btn-sm gap-1.5" :disabled="testing">
          <SvgIcon v-if="testing" name="refresh" size="14" class="animate-spin" />
          <SvgIcon v-else name="checkCircle" size="14" />
          {{ testing ? '测试中...' : '测试连接' }}
        </button>
        <button @click="$emit('close')" class="btn btn-ghost btn-sm">取消</button>
        <button @click="$emit('save', localForm)" class="btn btn-primary btn-sm gap-1.5"
                :disabled="!localForm.name || (localForm.type !== 'sqlite' && !localForm.host) || (localForm.type === 'sqlite' && !localForm.path)">
          <SvgIcon name="save" size="14" /> 保存
        </button>
      </div>

      <div v-if="testResult" :class="testResult.success ? 'alert alert-success' : 'alert alert-error'"
        class="mx-6 mb-5 rounded-lg text-sm">
        <SvgIcon :name="testResult.success ? 'checkCircle' : 'alertCircle'" size="16" />
        <span>{{ testResult.success ? '连接成功！' : '连接失败: ' + testResult.error }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref, computed, watch, onUnmounted, nextTick, onMounted } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'

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

// 密码框明文/掩码切换
const showPassword = ref(false)

// Sync localForm when parent updates the form prop (e.g. switching edit target)
watch(() => props.form, (newVal) => {
  localForm.value = { ...newVal }
})

// Emit changes to parent — 用 nextTick 防抖，避免每个字符触发一次 emit
let emitTimer: ReturnType<typeof setTimeout> | null = null
watch(localForm, (newVal) => {
  if (emitTimer) {clearTimeout(emitTimer)}
  emitTimer = setTimeout(() => {
    emit('update:form', { ...newVal })
  }, 16) // ~1 frame, batches rapid input events
}, { deep: true })

onUnmounted(() => {
  if (emitTimer) {clearTimeout(emitTimer)}
})

async function pickSqliteFile() {
  try {
    const selected = await open({
      multiple: false,
      title: '选择 SQLite 数据库文件',
      filters: [{
        name: 'SQLite 数据库',
        extensions: ['db', 'sqlite', 'sqlite3', 'db3']
      }]
    })
    if (selected) {
      const path = Array.isArray(selected) ? selected[0] : selected
      localForm.value.path = path
    }
  } catch (e) {
    console.error('文件选择失败:', e)
  }
}

const defaultPort = computed(() => {
  const map: Record<string, string> = {
    mysql: '3306',
    postgresql: '5432',
    redis: '6379'
  }
  return map[localForm.value.type] || ''
})
</script>
