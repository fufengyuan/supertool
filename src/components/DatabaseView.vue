<template>
  <div class="database-view">
    <div class="view-header">
      <h2>数据库管理</h2>
      <button class="btn-primary" @click="showAddDialog = true">+ 添加连接</button>
    </div>

    <div v-if="connections.length === 0 && !loading" class="empty-state">
      <p>暂无数据库连接</p>
      <p class="hint">点击「添加连接」配置 MySQL / PostgreSQL / Redis</p>
    </div>

    <div v-else class="connection-list">
      <div v-for="conn in connections" :key="conn.id" class="connection-card">
        <div class="conn-header">
          <span class="conn-type" :class="conn.type">{{ typeLabel(conn.type) }}</span>
          <h3>{{ conn.name }}</h3>
          <span class="conn-host">{{ conn.host }}:{{ conn.port }}</span>
        </div>
        <div class="conn-actions">
          <button class="btn-small" @click="testConn(conn)">测试</button>
          <button class="btn-small" @click="openQuery(conn.id)">查询</button>
          <button class="btn-small danger" @click="removeConnection(conn.id)">删除</button>
        </div>
      </div>
    </div>

    <!-- Add Connection Dialog -->
    <div v-if="showAddDialog" class="dialog-overlay" @click.self="showAddDialog = false">
      <div class="dialog">
        <h3>添加数据库连接</h3>
        <form @submit.prevent="handleAdd">
          <div class="form-group">
            <label>类型</label>
            <select v-model="form.type">
              <option value="mysql">MySQL</option>
              <option value="postgres">PostgreSQL</option>
              <option value="redis">Redis</option>
            </select>
          </div>
          <div class="form-group">
            <label>名称</label>
            <input v-model="form.name" required placeholder="我的数据库" />
          </div>
          <div class="form-row">
            <div class="form-group">
              <label>主机</label>
              <input v-model="form.host" required />
            </div>
            <div class="form-group">
              <label>端口</label>
              <input v-model.number="form.port" type="number" />
            </div>
          </div>
          <div class="form-group">
            <label>用户名</label>
            <input v-model="form.username" />
          </div>
          <div class="form-group">
            <label>密码</label>
            <input v-model="form.password" type="password" autocomplete="off" />
          </div>
          <div class="form-group" v-if="form.type !== 'redis'">
            <label>数据库名</label>
            <input v-model="form.dbName" placeholder="可选" />
          </div>
          <div class="form-group" v-if="form.type === 'redis'">
            <label>DB Index</label>
            <input v-model.number="form.dbIndex" type="number" placeholder="默认 0" />
          </div>
          <div class="dialog-actions">
            <button type="button" class="btn-secondary" @click="showAddDialog = false">取消</button>
            <button type="submit" class="btn-primary">保存</button>
          </div>
        </form>
      </div>
    </div>

    <!-- Query Dialog -->
    <div v-if="queryDialog" class="dialog-overlay" @click.self="queryDialog = false">
      <div class="dialog query-dialog">
        <h3>SQL 查询</h3>
        <textarea v-model="sqlInput" placeholder="SELECT * FROM ..." class="sql-input"></textarea>
        <div class="dialog-actions">
          <button class="btn-secondary" @click="queryDialog = false">关闭</button>
          <button class="btn-primary" @click="executeQuery" :disabled="queryLoading">
            {{ queryLoading ? '执行中...' : '执行' }}
          </button>
        </div>
        <div v-if="queryResult" class="query-result">
          <pre>{{ queryResult }}</pre>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
console.log("[components/DatabaseView.vue] component loaded")
import { ref, onMounted } from 'vue'
import { getTauriAPI } from '../utils/tauri-api'
import type { DbConnectionConfig } from '../types'

const api = getTauriAPI()

const connections = ref<DbConnectionConfig[]>([])
const loading = ref(false)
const showAddDialog = ref(false)
const queryDialog = ref(false)
const queryLoading = ref(false)
const sqlInput = ref('')
const queryResult = ref('')
const currentConnId = ref('')

const form = ref({
  name: '',
  type: 'mysql' as string,
  host: 'localhost',
  port: 3306,
  username: 'root',
  password: '',
  dbName: '',
  dbIndex: 0,
})

onMounted(async () => {
  await loadConnections()
})

async function loadConnections() {
  loading.value = true
  // Read from SQLite db_connections table via a direct query
  // For simplicity, use localStorage as connection registry in this migration
  const stored = localStorage.getItem('tauri_db_connections')
  connections.value = stored ? JSON.parse(stored) : []
  loading.value = false
}

function saveConnections() {
  localStorage.setItem('tauri_db_connections', JSON.stringify(connections.value))
}

function typeLabel(type: string): string {
  return { mysql: 'MySQL', postgres: 'PostgreSQL', redis: 'Redis' }[type] || type
}

async function handleAdd() {
  const conn: DbConnectionConfig = {
    id: crypto.randomUUID(),
    name: form.value.name,
    type: form.value.type,
    host: form.value.host,
    port: form.value.port,
    username: form.value.username,
    password: form.value.password || undefined,
    dbName: form.value.dbName || undefined,
    dbIndex: form.value.dbIndex,
  }
  connections.value.push(conn)
  saveConnections()
  showAddDialog.value = false
  form.value = { name: '', type: 'mysql', host: 'localhost', port: 3306, username: 'root', password: '', dbName: '', dbIndex: 0 }
}

async function testConn(conn: DbConnectionConfig) {
  const result = await api.dbConnect(conn)
  alert(result.success ? '连接成功！' : `连接失败: ${result.error}`)
}

function openQuery(connId: string) {
  currentConnId.value = connId
  sqlInput.value = ''
  queryResult.value = ''
  queryDialog.value = true
}

async function executeQuery() {
  queryLoading.value = true
  queryResult.value = ''
  try {
    const result = await api.dbQuery(currentConnId.value, sqlInput.value)
    queryResult.value = result.success
      ? JSON.stringify(result.rows, null, 2)
      : `错误: ${result.error}`
  } catch (err) {
    queryResult.value = `异常: ${err}`
  } finally {
    queryLoading.value = false
  }
}

async function removeConnection(id: string) {
  if (confirm('确定删除此连接？')) {
    connections.value = connections.value.filter(c => c.id !== id)
    saveConnections()
  }
}
</script>

<style scoped>
.database-view { max-width: 1200px; }
.view-header {
  display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px;
}
.empty-state { text-align: center; padding: 60px 20px; color: #64748b; }
.empty-state .hint { font-size: 13px; margin-top: 8px; }

.connection-list { display: flex; flex-direction: column; gap: 8px; }

.connection-card {
  display: flex; justify-content: space-between; align-items: center;
  background: #1e293b; border: 1px solid #334155; border-radius: 8px; padding: 16px;
}

.conn-header { display: flex; align-items: center; gap: 10px; }
.conn-type {
  padding: 2px 8px; border-radius: 4px; font-size: 11px; font-weight: 600; text-transform: uppercase;
}
.conn-type.mysql { background: #1e3a5f; color: #60a5fa; }
.conn-type.postgres { background: #312e81; color: #a78bfa; }
.conn-type.redis { background: #7f1d1d; color: #fca5a5; }

.conn-header h3 { font-size: 15px; }
.conn-host { font-size: 13px; color: #64748b; }
.conn-actions { display: flex; gap: 6px; }

.btn-primary, .btn-secondary, .btn-small, .btn-small.danger { /* same */ }
.btn-primary { padding: 8px 16px; background: #4f46e5; color: #fff; border: none; border-radius: 6px; cursor: pointer; font-size: 14px; }
.btn-secondary { padding: 8px 16px; background: #334155; color: #e2e8f0; border: none; border-radius: 6px; cursor: pointer; font-size: 14px; }
.btn-small { padding: 4px 12px; background: #334155; color: #e2e8f0; border: none; border-radius: 4px; cursor: pointer; font-size: 12px; }
.btn-small.danger { background: #7f1d1d; }
.btn-small:hover { opacity: 0.85; }

.dialog-overlay {
  position: fixed; inset: 0; background: rgba(0, 0, 0, 0.5);
  display: flex; align-items: center; justify-content: center; z-index: 100;
}
.dialog {
  background: #1e293b; border: 1px solid #334155; border-radius: 12px; padding: 24px; min-width: 400px;
}
.dialog h3 { margin-bottom: 16px; }
.form-group { margin-bottom: 12px; }
.form-group label { display: block; font-size: 13px; color: #94a3b8; margin-bottom: 4px; }
.form-group input, .form-group select {
  width: 100%; padding: 8px 12px; background: #0f172a;
  border: 1px solid #334155; border-radius: 6px; color: #e2e8f0; font-size: 14px;
}
.form-row { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
.dialog-actions { display: flex; gap: 8px; justify-content: flex-end; margin-top: 16px; }

.query-dialog { min-width: 600px; }
.sql-input {
  width: 100%; min-height: 100px; padding: 10px; background: #0f172a;
  border: 1px solid #334155; border-radius: 6px; color: #e2e8f0;
  font-family: 'SF Mono', Monaco, monospace; font-size: 13px; resize: vertical;
  margin-bottom: 12px;
}
.query-result {
  margin-top: 12px; background: #0f172a; border: 1px solid #334155;
  border-radius: 6px; padding: 12px; max-height: 300px; overflow: auto;
}
.query-result pre {
  font-family: 'SF Mono', Monaco, monospace; font-size: 12px;
  color: #e2e8f0; white-space: pre-wrap;
}
</style>
