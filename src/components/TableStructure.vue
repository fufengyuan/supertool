<template>
  <div class="table-structure">
    <!-- 顶部工具栏 -->
    <div class="ts-toolbar">
      <div class="ts-toolbar-left">
        <span class="ts-table-icon">📋</span>
        <span class="ts-table-name">{{ tableName }}</span>
        <span v-if="dbType" class="ts-db-type">{{ dbTypeLabel }}</span>
      </div>
      <div class="ts-toolbar-right">
        <button
          class="ts-btn ts-btn-ghost"
          :disabled="loading"
          @click="showCreateSql"
          title="查看建表 SQL"
        >
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="16 18 22 12 16 6" /><polyline points="8 6 2 12 8 18" />
          </svg>
          建表 SQL
        </button>
        <button
          class="ts-btn ts-btn-ghost"
          :disabled="loading || !hasChanges"
          @click="discardChanges"
          title="放弃修改"
        >
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="1 4 1 10 7 10" /><path d="M3.51 15a9 9 0 1 0 2.12-9.36L1 10" />
          </svg>
          放弃修改
        </button>
        <button
          class="ts-btn ts-btn-ghost"
          :disabled="loading"
          @click="refreshWithOriginals"
          title="刷新"
        >
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="23 4 23 10 17 10" />
            <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
          </svg>
          刷新
        </button>
        <button
          class="ts-btn ts-btn-primary"
          :disabled="loading || !hasChanges"
          @click="showSqlPreview"
          title="保存结构"
        >
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
            <polyline points="17 21 17 13 7 13 7 21" />
            <polyline points="7 3 7 8 15 8" />
          </svg>
          保存 ({{ changeCount }})
        </button>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading && !columns.length" class="ts-loading">
      <svg class="ts-spinner" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 12a9 9 0 11-6.219-8.56" />
      </svg>
      <span>加载表结构中...</span>
    </div>

    <!-- 错误状态 -->
    <div v-else-if="error" class="ts-error">
      <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10" />
        <line x1="12" y1="8" x2="12" y2="12" />
        <line x1="12" y1="16" x2="12.01" y2="16" />
      </svg>
      <span>{{ error }}</span>
      <button class="ts-btn ts-btn-ghost ts-btn-sm" @click="refreshWithOriginals">重试</button>
    </div>

    <template v-else>
      <!-- Tab 切换 -->
      <div class="ts-tabs">
        <div
          class="ts-tab"
          :class="{ active: activeTab === 'columns' }"
          @click="activeTab = 'columns'"
        >
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
            <line x1="3" y1="9" x2="21" y2="9" />
            <line x1="9" y1="21" x2="9" y2="9" />
          </svg>
          字段 ({{ columns.length }})
          <span v-if="columnChangeCount > 0" class="ts-tab-badge">{{ columnChangeCount }}</span>
        </div>
        <div
          class="ts-tab"
          :class="{ active: activeTab === 'indexes' }"
          @click="activeTab = 'indexes'"
        >
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
          </svg>
          索引 ({{ groupedIndexes.length }})
          <span v-if="indexChangeCount > 0" class="ts-tab-badge">{{ indexChangeCount }}</span>
        </div>
      </div>

      <!-- 字段 Tab -->
      <div v-show="activeTab === 'columns'" class="ts-panel">
        <div class="ts-panel-toolbar">
          <button class="ts-btn ts-btn-ghost ts-btn-sm" @click="addColumn" title="添加列">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
            </svg>
            添加列
          </button>
          <button
            class="ts-btn ts-btn-ghost ts-btn-sm ts-btn-danger"
            :disabled="!selectedColumnIndex"
            @click="deleteSelectedColumn"
            title="删除选中列"
          >
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="3 6 5 6 21 6" /><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
            </svg>
            删除列
          </button>
          <span class="ts-hint">拖拽左侧手柄可调整列顺序</span>
        </div>

        <div class="ts-table-wrapper">
          <table class="ts-table">
            <thead>
              <tr>
                <th class="ts-th-drag" style="width:36px"></th>
                <th class="ts-th-check" style="width:32px"></th>
                <th style="width:150px">字段名</th>
                <th style="width:130px">类型</th>
                <th style="width:70px">长度</th>
                <th style="width:60px">小数</th>
                <th style="width:56px">NULL</th>
                <th style="width:100px">默认值</th>
                <th style="width:56px">主键</th>
                <th style="width:56px">自增</th>
                <th>注释</th>
              </tr>
            </thead>
            <tbody>
              <template v-for="(col, idx) in columns" :key="col._uid">
                <!-- Drop indicator before this row -->
                <tr v-if="dragIndex !== null && dragOverIndex === idx && dragOverPosition === 'before'" class="ts-drop-indicator">
                  <td :colspan="11"></td>
                </tr>
                <tr
                  :class="{
                    'ts-row-new': col._isNew,
                    'ts-row-deleted': col._deleted,
                    'ts-row-modified': !col._isNew && !col._deleted && isColumnModified(col),
                    'ts-row-selected': selectedColumnIndex === idx,
                    'ts-row-dragging': dragIndex === idx
                  }"
                  @click="selectedColumnIndex = idx"
                  draggable="true"
                  @dragstart="onDragStart(idx, $event)"
                  @dragover.prevent="onDragOver(idx, $event)"
                  @drop="onDrop(idx, $event)"
                  @dragend="onDragEnd"
                >
                  <!-- 拖拽手柄 -->
                  <td class="ts-td-drag">
                    <span class="ts-drag-handle" title="拖拽排序">⠿</span>
                  </td>
                  <!-- 选中复选 -->
                  <td class="ts-td-check">
                    <input
                      type="checkbox"
                      :checked="selectedColumnIndex === idx"
                      @click.stop="selectedColumnIndex = selectedColumnIndex === idx ? null : idx"
                    />
                  </td>
                  <!-- 字段名 -->
                  <td>
                    <input
                      class="ts-input ts-input-name"
                      v-model="col.name"
                      :class="{ 'ts-input-pri': col.primaryKey }"
                      placeholder="字段名"
                      @click.stop
                    />
                  </td>
                  <!-- 类型 -->
                  <td>
                    <select class="ts-select" v-model="col.type" @click.stop>
                      <option v-for="t in columnTypes" :key="t" :value="t">{{ t }}</option>
                    </select>
                  </td>
                  <!-- 长度 -->
                  <td>
                    <input
                      class="ts-input ts-input-sm"
                      type="number"
                      v-model.number="col.length"
                      placeholder="—"
                      min="0"
                      @click.stop
                    />
                  </td>
                  <!-- 小数位 -->
                  <td>
                    <input
                      class="ts-input ts-input-sm"
                      type="number"
                      v-model.number="col.decimals"
                      placeholder="—"
                      min="0"
                      @click.stop
                    />
                  </td>
                  <!-- NULL -->
                  <td class="ts-td-center">
                    <label class="ts-toggle" title="允许 NULL">
                      <input type="checkbox" v-model="col.nullable" @click.stop />
                      <span class="ts-toggle-track">
                        <span class="ts-toggle-thumb"></span>
                      </span>
                    </label>
                  </td>
                  <!-- 默认值 -->
                  <td>
                    <input
                      class="ts-input ts-input-sm"
                      v-model="col.defaultValue"
                      placeholder="NULL"
                      @click.stop
                    />
                  </td>
                  <!-- 主键 -->
                  <td class="ts-td-center">
                    <label class="ts-toggle" title="主键">
                      <input type="checkbox" v-model="col.primaryKey" @click.stop @change="onPrimaryKeyChange(col)" />
                      <span class="ts-toggle-track">
                        <span class="ts-toggle-thumb"></span>
                      </span>
                    </label>
                  </td>
                  <!-- 自增 -->
                  <td class="ts-td-center">
                    <label class="ts-toggle" title="自增">
                      <input type="checkbox" v-model="col.autoIncrement" :disabled="!canAutoIncrement(col)" @click.stop />
                      <span class="ts-toggle-track">
                        <span class="ts-toggle-thumb"></span>
                      </span>
                    </label>
                  </td>
                  <!-- 注释 -->
                  <td>
                    <input
                      class="ts-input"
                      v-model="col.comment"
                      placeholder="注释"
                      @click.stop
                    />
                  </td>
                </tr>
                <!-- Drop indicator after last row -->
                <tr v-if="dragIndex !== null && dragOverIndex === idx && dragOverPosition === 'after' && idx === columns.length - 1" class="ts-drop-indicator">
                  <td :colspan="11"></td>
                </tr>
              </template>
            </tbody>
          </table>
        </div>
      </div>

      <!-- 索引 Tab -->
      <div v-show="activeTab === 'indexes'" class="ts-panel">
        <div class="ts-panel-toolbar">
          <button class="ts-btn ts-btn-ghost ts-btn-sm" @click="addIndex" title="添加索引">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
            </svg>
            添加索引
          </button>
          <button
            class="ts-btn ts-btn-ghost ts-btn-sm ts-btn-danger"
            :disabled="selectedIndexes.length === 0"
            @click="deleteSelectedIndexes"
            title="删除选中索引"
          >
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="3 6 5 6 21 6" /><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
            </svg>
            删除索引
          </button>
        </div>

        <div class="ts-table-wrapper">
          <table class="ts-table">
            <thead>
              <tr>
                <th style="width:32px"></th>
                <th style="width:140px">索引名称</th>
                <th style="width:100px">类型</th>
                <th>包含列</th>
                <th style="width:56px">操作</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="(idx, i) in indexes"
                :key="idx._uid"
                :class="{
                  'ts-row-new': idx._isNew,
                  'ts-row-deleted': idx._deleted,
                  'ts-row-modified': !idx._isNew && !idx._deleted && isIndexModified(idx)
                }"
              >
                <td class="ts-td-check">
                  <input
                    type="checkbox"
                    :checked="selectedIndexes.includes(i)"
                    @change="toggleIndexSelection(i)"
                  />
                </td>
                <td>
                  <input
                    class="ts-input ts-input-name"
                    v-model="idx.name"
                    :disabled="idx.type === 'PRIMARY'"
                    placeholder="索引名称"
                  />
                </td>
                <td>
                  <select class="ts-select" v-model="idx.type">
                    <option value="PRIMARY">PRIMARY</option>
                    <option value="UNIQUE">UNIQUE</option>
                    <option value="INDEX">INDEX</option>
                    <option value="FULLTEXT">FULLTEXT</option>
                  </select>
                </td>
                <td>
                  <div class="ts-index-columns">
                    <div v-for="(col, ci) in idx.columns" :key="`idxcol-${idx._uid}-${ci}-${idx.columns[ci] || 'empty'}`" class="ts-index-col-row">
                      <select class="ts-select ts-select-sm" v-model="idx.columns[ci]">
                        <option value="">— 选择列 —</option>
                        <option v-for="c in availableColumnNames" :key="c" :value="c">{{ c }}</option>
                      </select>
                      <button
                        v-if="idx.columns.length > 1"
                        class="ts-btn-icon"
                        @click="removeIndexColumn(idx, ci)"
                        title="移除列"
                      >
                        <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
                          <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
                        </svg>
                      </button>
                    </div>
                    <button class="ts-btn ts-btn-ghost ts-btn-xs" @click="addIndexColumn(idx)">
                      + 添加列
                    </button>
                  </div>
                </td>
                <td class="ts-td-center">
                  <button
                    v-if="idx._isNew"
                    class="ts-btn-icon ts-btn-danger"
                    @click="deleteIndexAt(i)"
                    title="删除"
                  >
                    <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                      <polyline points="3 6 5 6 21 6" /><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                    </svg>
                  </button>
                  <span v-else-if="idx._deleted" class="ts-deleted-badge">已删除</span>
                </td>
              </tr>
              <tr v-if="indexes.length === 0">
                <td colspan="5" class="ts-empty">暂无索引，点击"添加索引"创建</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </template>

    <!-- SQL 预览对话框 -->
    <div v-if="showPreview" class="ts-modal-overlay" @click.self="showPreview = false">
      <div class="ts-modal">
        <div class="ts-modal-header">
          <h3 class="ts-modal-title">
            <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="16 18 22 12 16 6" /><polyline points="8 6 2 12 8 18" />
            </svg>
            SQL 预览
          </h3>
          <button class="ts-btn-close" @click="showPreview = false">
            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
        <div class="ts-modal-body">
          <p class="ts-modal-hint">即将执行以下 {{ previewSqls.length }} 条 DDL 语句：</p>
          <div class="ts-sql-preview">
            <div v-for="(sql, i) in previewSqls" :key="i" class="ts-sql-stmt">
              <span class="ts-sql-num">{{ i + 1 }}</span>
              <code>{{ sql }}</code>
            </div>
          </div>
          <div v-if="previewError" class="ts-modal-error">{{ previewError }}</div>
        </div>
        <div class="ts-modal-footer">
          <button class="ts-btn ts-btn-ghost" @click="showPreview = false">取消</button>
          <button
            class="ts-btn ts-btn-primary"
            :disabled="executing"
            @click="executeSqls"
          >
            <svg v-if="executing" class="ts-spinner-inline" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 12a9 9 0 11-6.219-8.56" />
            </svg>
            {{ executing ? '执行中...' : '确认执行' }}
          </button>
        </div>
      </div>
    </div>

    <!-- CREATE TABLE SQL Modal -->
    <div v-if="showCreateSqlModal" class="ts-modal-overlay" @click.self="showCreateSqlModal = false">
      <div class="ts-modal ts-modal-wide">
        <div class="ts-modal-header">
          <h3 class="ts-modal-title">
            <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="16 18 22 12 16 6" /><polyline points="8 6 2 12 8 18" />
            </svg>
            建表 SQL — {{ tableName }}
          </h3>
          <button class="ts-btn-close" @click="showCreateSqlModal = false">
            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
        <div class="ts-modal-body">
          <div v-if="loadingCreateSql" class="ts-loading-inline">加载中...</div>
          <pre v-else class="ts-create-sql">{{ createSql }}</pre>
        </div>
        <div class="ts-modal-footer">
          <button class="ts-btn ts-btn-ghost" @click="showCreateSqlModal = false">关闭</button>
          <button class="ts-btn ts-btn-ghost" @click="copyCreateSql" :disabled="!createSql">
            📋 复制 SQL
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useTableStructure } from '@/composables/useTableStructure'

const props = defineProps<{
  connectionId: string
  tableName: string
  dbName?: string
  dbType?: string
}>()

const emit = defineEmits<{
  close: []
}>()

const ts = useTableStructure({ connId: props.connectionId, tableName: props.tableName, dbName: props.dbName }, emit)

const {
  loading, error, columns, indexes,
  activeTab, selectedColumnIndex, selectedIndexes,
  showCreateSqlModal, showPreview, previewSqls, previewError,
  executing, createSql, loadingCreateSql, dbTypeLabel,
  hasChanges, changeCount, columnChangeCount, indexChangeCount,
  availableColumnNames, groupedIndexes, dragIndex, dragOverIndex, dragOverPosition,
  canAutoIncrement,
  addColumn, deleteSelectedColumn, addIndex, addIndexColumn,
  removeIndexColumn, deleteIndexAt, deleteSelectedIndexes,
  onDragStart, onDragOver, onDragEnd, onDrop,
  onPrimaryKeyChange, toggleIndexSelection,
  refreshWithOriginals, discardChanges,
  executeSqls, generateDdl, showCreateSql, copyCreateSql,
  showSqlPreview,
  columnTypes, isColumnModified, isIndexModified,
} = ts
</script>

<style scoped>
/* ============ Root ============ */
.table-structure {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  background: var(--card-bg);
  color: var(--main-text);
}

/* ============ Toolbar ============ */
.ts-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  border-bottom: 1px solid var(--border-color);
  background: var(--main-bg);
  flex-shrink: 0;
}

.ts-toolbar-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.ts-table-icon {
  font-size: 18px;
}

.ts-table-name {
  font-size: 15px;
  font-weight: 600;
}

.ts-db-type {
  font-size: 11px;
  padding: 2px 6px;
  border-radius: 4px;
  background: var(--primary-light);
  color: var(--primary-color);
  font-weight: 500;
}

.ts-toolbar-right {
  display: flex;
  align-items: center;
  gap: 6px;
}

/* ============ Buttons ============ */
.ts-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--card-bg);
  color: var(--main-text);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
  white-space: nowrap;
}

.ts-btn:hover:not(:disabled) {
  background: var(--primary-light);
  border-color: var(--primary-color);
  color: var(--primary-color);
}

.ts-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.ts-btn-primary {
  background: var(--primary-color);
  border-color: var(--primary-color);
  color: #fff;
}

.ts-btn-primary:hover:not(:disabled) {
  background: var(--primary-hover);
  color: #fff;
}

.ts-btn-ghost {
  background: transparent;
  border-color: transparent;
}

.ts-btn-ghost:hover:not(:disabled) {
  background: var(--primary-light);
  border-color: transparent;
}

.ts-btn-danger:hover:not(:disabled) {
  background: var(--primary-light);
  border-color: var(--danger-color);
  color: var(--danger-color);
}

.ts-btn-sm {
  padding: 4px 8px;
  font-size: 11px;
}

.ts-btn-xs {
  padding: 2px 6px;
  font-size: 10px;
}

.ts-btn-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  background: transparent;
  color: var(--main-text-secondary);
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.15s;
}

.ts-btn-icon:hover {
  background: var(--primary-light);
  color: var(--primary-color);
}

.ts-btn-icon.ts-btn-danger:hover {
  background: var(--primary-light);
  color: var(--danger-color);
}

.ts-btn-close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--main-text-secondary);
  border-radius: 4px;
  cursor: pointer;
}

.ts-btn-close:hover {
  background: var(--primary-light);
}

/* ============ Loading / Error ============ */
.ts-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 48px;
  color: var(--main-text-secondary);
  font-size: 13px;
}

.ts-spinner {
  animation: ts-spin 1s linear infinite;
}

.ts-spinner-inline {
  animation: ts-spin 1s linear infinite;
}

@keyframes ts-spin {
  from { transform: rotate(0); }
  to { transform: rotate(360deg); }
}

.ts-error {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 16px;
  margin: 16px;
  border-radius: 8px;
  background: rgba(210, 15, 57, 0.08);
  color: var(--danger-color);
  font-size: 13px;
}

:root.dark .ts-error {
  background: rgba(243, 139, 168, 0.08);
}

/* ============ Tabs ============ */
.ts-tabs {
  display: flex;
  align-items: center;
  gap: 0;
  border-bottom: 1px solid var(--border-color);
  padding: 0 16px;
  flex-shrink: 0;
  background: var(--main-bg);
}

.ts-tab {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 10px 16px;
  font-size: 13px;
  font-weight: 500;
  color: var(--main-text-secondary);
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: all 0.15s;
  user-select: none;
}

.ts-tab:hover {
  color: var(--main-text);
  background: var(--primary-light);
}

.ts-tab.active {
  color: var(--primary-color);
  border-bottom-color: var(--primary-color);
}

.ts-tab-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 16px;
  height: 16px;
  padding: 0 4px;
  border-radius: 8px;
  background: var(--primary-color);
  color: #fff;
  font-size: 10px;
  font-weight: 600;
}

/* ============ Panel ============ */
.ts-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.ts-panel-toolbar {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.ts-hint {
  font-size: 11px;
  color: var(--main-text-secondary);
  margin-left: auto;
}

/* ============ Table ============ */
.ts-table-wrapper {
  flex: 1;
  overflow: auto;
}

.ts-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
}

.ts-table thead {
  position: sticky;
  top: 0;
  z-index: 2;
}

.ts-table th {
  padding: 6px 8px;
  text-align: left;
  font-weight: 600;
  font-size: 11px;
  color: var(--main-text-secondary);
  background: var(--input-bg);
  border-bottom: 1px solid var(--border-color);
  white-space: nowrap;
  user-select: none;
}

.ts-table td {
  padding: 2px 4px;
  border-bottom: 1px solid var(--border-color);
  white-space: nowrap;
  vertical-align: middle;
}

.ts-td-center {
  text-align: center;
}

.ts-td-drag {
  text-align: center;
  cursor: grab;
}

.ts-td-drag:active {
  cursor: grabbing;
}

.ts-td-check {
  text-align: center;
}

.ts-td-check input[type="checkbox"] {
  cursor: pointer;
}

/* ============ Row States ============ */
.ts-row-new td {
  background: rgba(64, 160, 43, 0.08);
}

:root.dark .ts-row-new td {
  background: rgba(166, 227, 161, 0.08);
}

.ts-row-deleted td {
  background: rgba(210, 15, 57, 0.08);
  text-decoration: line-through;
  opacity: 0.5;
}

:root.dark .ts-row-deleted td {
  background: rgba(243, 139, 168, 0.08);
}

.ts-row-modified td {
  background: rgba(223, 142, 29, 0.08);
}

:root.dark .ts-row-modified td {
  background: rgba(249, 226, 175, 0.08);
}

.ts-row-selected td {
  background: var(--primary-light) !important;
}

.ts-row-dragging {
  opacity: 0.4;
}

.ts-row-dragging td {
  border-top: 2px solid var(--primary-color) !important;
  border-bottom: 2px solid var(--primary-color) !important;
}

/* Drop indicator */
.ts-drop-indicator {
  height: 3px;
  background: var(--primary-color);
  position: relative;
}

.ts-drop-indicator::before,
.ts-drop-indicator::after {
  content: '';
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  width: 8px;
  height: 8px;
  background: var(--primary-color);
  border-radius: 50%;
}

.ts-drop-indicator::before { left: 4px; }
.ts-drop-indicator::after { right: 4px; }

/* ============ Inputs ============ */
.ts-input {
  width: 100%;
  padding: 4px 6px;
  border: 1px solid transparent;
  border-radius: 4px;
  background: transparent;
  color: var(--main-text);
  font-size: 12px;
  font-family: inherit;
  outline: none;
  transition: all 0.15s;
}

.ts-input:hover {
  border-color: var(--border-color);
  background: var(--card-bg);
}

.ts-input:focus {
  border-color: var(--primary-color);
  background: var(--card-bg);
  box-shadow: 0 0 0 2px rgba(26, 115, 232, 0.15);
}

.ts-input-sm {
  padding: 3px 4px;
  font-size: 11px;
}

.ts-input-name {
  font-weight: 500;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
}

.ts-input-pri {
  color: var(--primary-color);
}

.ts-select {
  width: 100%;
  padding: 4px 6px;
  border: 1px solid transparent;
  border-radius: 4px;
  background: transparent;
  color: var(--main-text);
  font-size: 12px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  outline: none;
  cursor: pointer;
  transition: all 0.15s;
}

.ts-select:hover {
  border-color: var(--border-color);
  background: var(--card-bg);
}

.ts-select:focus {
  border-color: var(--primary-color);
  background: var(--card-bg);
}

.ts-select-sm {
  font-size: 11px;
  padding: 2px 4px;
}

/* ============ Toggle ============ */
.ts-toggle {
  display: inline-flex;
  align-items: center;
  cursor: pointer;
}

.ts-toggle input {
  position: absolute;
  opacity: 0;
  width: 0;
  height: 0;
}

.ts-toggle-track {
  display: block;
  width: 32px;
  height: 18px;
  border-radius: 9px;
  background: var(--border-color);
  position: relative;
  transition: all 0.2s;
}

.ts-toggle-thumb {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--main-text);
  transition: all 0.2s;
  box-shadow: 0 1px 2px rgba(0,0,0,0.2);
}

.ts-toggle input:checked + .ts-toggle-track {
  background: var(--primary-color);
}

.ts-toggle input:checked + .ts-toggle-track .ts-toggle-thumb {
  left: 16px;
}

/* ============ Drag Handle ============ */
.ts-drag-handle {
  display: inline-block;
  color: var(--main-text-secondary);
  font-size: 14px;
  line-height: 1;
  cursor: grab;
  user-select: none;
  padding: 2px;
}

.ts-drag-handle:active {
  cursor: grabbing;
}

/* ============ Index Columns ============ */
.ts-index-columns {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.ts-index-col-row {
  display: flex;
  align-items: center;
  gap: 2px;
}

/* ============ Deletion Badge ============ */
.ts-deleted-badge {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 3px;
  background: rgba(210, 15, 57, 0.1);
  color: var(--danger-color);
}

:root.dark .ts-deleted-badge {
  background: rgba(243, 139, 168, 0.1);
}

/* ============ Empty ============ */
.ts-empty {
  text-align: center;
  padding: 32px;
  color: var(--main-text-secondary);
  font-size: 13px;
}

/* ============ Modal ============ */
.ts-modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: ts-fade-in 0.15s ease-out;
}

@keyframes ts-fade-in {
  from { opacity: 0; }
  to { opacity: 1; }
}

.ts-modal {
  width: 720px;
  max-width: 90vw;
  max-height: 80vh;
  background: var(--card-bg);
  border-radius: 12px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  animation: ts-slide-up 0.2s ease-out;
}

@keyframes ts-slide-up {
  from { transform: translateY(20px); opacity: 0; }
  to { transform: translateY(0); opacity: 1; }
}

.ts-modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
}

.ts-modal-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
  margin: 0;
}

.ts-modal-body {
  padding: 16px 20px;
  overflow-y: auto;
  flex: 1;
}

.ts-modal-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 20px;
  border-top: 1px solid var(--border-color);
}

.ts-modal-hint {
  font-size: 13px;
  color: var(--main-text-secondary);
  margin: 0 0 12px;
}

.ts-modal-error {
  margin-top: 12px;
  padding: 8px 12px;
  border-radius: 6px;
  background: rgba(210, 15, 57, 0.08);
  color: var(--danger-color);
  font-size: 12px;
}

:root.dark .ts-modal-error {
  background: rgba(243, 139, 168, 0.08);
}

/* ============ SQL Preview ============ */
.ts-sql-stmt code {
  color: var(--success-color);
  word-break: break-all;
}

.ts-sql-preview {
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 12px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
  line-height: 1.6;
  max-height: 400px;
  overflow-y: auto;
}

.ts-sql-stmt {
  display: flex;
  gap: 8px;
  padding: 4px 0;
  border-bottom: 1px solid var(--border-color);
}

.ts-sql-stmt:last-child {
  border-bottom: none;
}

.ts-sql-num {
  color: var(--main-text-secondary);
  user-select: none;
  min-width: 20px;
  text-align: right;
  flex-shrink: 0;
}

/* ============ Scrollbar ============ */
.ts-table-wrapper::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

.ts-table-wrapper::-webkit-scrollbar-track {
  background: transparent;
}

.ts-table-wrapper::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 4px;
}

.ts-table-wrapper::-webkit-scrollbar-thumb:hover {
  background: var(--main-text-secondary);
}

.ts-sql-preview::-webkit-scrollbar {
  width: 6px;
}

.ts-sql-preview::-webkit-scrollbar-track {
  background: transparent;
}

.ts-sql-preview::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 3px;
}

.ts-modal-wide {
  width: 900px;
  max-width: 95vw;
}

.ts-loading-inline {
  text-align: center;
  padding: 24px;
  color: var(--main-text-secondary);
}

.ts-create-sql {
  background: var(--input-bg);
  color: var(--main-text);
  padding: 16px;
  border-radius: 8px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', 'Menlo', monospace;
  font-size: 13px;
  line-height: 1.6;
  max-height: 60vh;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-all;
  margin: 0;
}

.ts-create-sql::-webkit-scrollbar {
  width: 6px;
}

.ts-create-sql::-webkit-scrollbar-track {
  background: transparent;
}

.ts-create-sql::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 3px;
}
</style>