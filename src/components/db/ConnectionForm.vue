<template>
  <div class="form-modal-overlay" @click="$emit('close')">
    <div class="form-modal" @click.stop>
      <div class="form-modal-header">
        <h3>{{ isEditing ? '✏️ 编辑连接' : '🗄️ 添加数据库连接' }}</h3>
        <button @click="$emit('close')" class="form-modal-close">×</button>
      </div>
      <div class="form-modal-body">
        <div class="form-row">
          <div class="form-field">
            <label>连接名称 <span class="required">*</span></label>
            <input v-model="localForm.name" class="form-input" placeholder="我的数据库" />
          </div>
          <div class="form-field">
            <label>数据库类型 <span class="required">*</span></label>
            <select v-model="localForm.type" class="form-input form-select">
              <option value="mysql">MySQL</option>
              <option value="postgresql">PostgreSQL</option>
              <option value="redis">Redis</option>
              <option value="sqlite">SQLite</option>
            </select>
          </div>
        </div>

        <!-- SQLite uses path instead of host/port -->
        <template v-if="localForm.type === 'sqlite'">
          <div class="form-field">
            <label>数据库文件路径</label>
            <input v-model="localForm.path" class="form-input" placeholder="/path/to/database.db" />
          </div>
        </template>

        <template v-else>
          <div class="form-row">
            <div class="form-field">
              <label>主机地址</label>
              <input v-model="localForm.host" class="form-input" placeholder="127.0.0.1" />
            </div>
            <div class="form-field">
              <label>端口</label>
              <input v-model.number="localForm.port" type="number" class="form-input" :placeholder="defaultPort" />
            </div>
          </div>

          <div class="form-row">
            <div class="form-field" v-if="localForm.type !== 'redis'">
              <label>用户名</label>
              <input v-model="localForm.user" class="form-input" placeholder="root" />
            </div>
            <div class="form-field" :class="{ 'full-width': localForm.type === 'redis' }">
              <label>密码</label>
              <input v-model="localForm.password" type="password" class="form-input" autocomplete="off" :placeholder="localForm.type === 'redis' ? '无密码留空' : '密码'" />
            </div>
          </div>

          <div class="form-field" v-if="localForm.type !== 'redis'">
            <label>数据库名</label>
            <input v-model="localForm.database" class="form-input" placeholder="database_name" />
          </div>

          <div class="form-field" v-if="localForm.type === 'redis'">
            <label>数据库索引</label>
            <input v-model.number="localForm.dbIndex" type="number" class="form-input" placeholder="0" />
          </div>

          <div class="form-divider"></div>

          <label class="form-checkbox form-checkbox-security">
            <input v-model="localForm.requiresApproval" type="checkbox" />
            <span class="security-label">
              <span>🔒 SQL 执行审核</span>
              <span class="security-desc">开启后 CLI 无法执行 SQL，GUI 执行前需要确认</span>
            </span>
          </label>
        </template>
      </div>

      <div class="form-modal-footer">
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

      <div v-if="testResult" class="test-result" :class="testResult.success ? 'success' : 'error'"
        style="margin: 0 24px 20px">
        {{ testResult.success ? '✅ 连接成功！' : '❌ 连接失败: ' + testResult.error }}
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

<style scoped>
.form-field.full-width {
  flex: 1 1 100%;
  max-width: 100%;
}

.form-divider {
  margin: 8px 0;
  border-top: 1px solid oklch(var(--bc) / 0.1);
}

.form-checkbox-security {
  padding: 8px 10px;
  background: rgba(245, 158, 11, 0.06);
  border-radius: 6px;
  border: 1px solid rgba(245, 158, 11, 0.15);
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

.security-label {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.security-desc {
  font-size: 11px;
  color: var(--text-secondary);
}
</style>
