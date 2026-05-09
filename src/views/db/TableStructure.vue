<template>
  <div class="table-structure flex flex-col h-full overflow-hidden bg-base-100 text-base-content">
    <!-- 顶部工具栏 -->
    <div class="flex items-center justify-between px-4 py-[10px] border-b border-base-content/10 bg-base-200 shrink-0">
      <div class="flex items-center gap-2">
        <span class="text-lg"><SvgIcon name="file" size="14" class="align-text-bottom" /></span>
        <span class="text-[15px] font-semibold">{{ tableName }}</span>
        <span v-if="dbType" class="text-[11px] px-[6px] py-[2px] rounded bg-primary/10 text-primary font-medium">{{ dbTypeLabel }}</span>
      </div>
      <div class="flex items-center gap-1.5">
        <button
          class="btn btn-ghost btn-xs"
          :disabled="loading"
          @click="showCreateSql"
          title="查看建表 SQL"
        >
          <SvgIcon name="code" size="14" />
          建表 SQL
        </button>
        <button
          class="btn btn-ghost btn-xs"
          :disabled="loading || !hasChanges"
          @click="discardChanges"
          title="放弃修改"
        >
          <SvgIcon name="undo" size="14" />
          放弃修改
        </button>
        <button
          class="btn btn-ghost btn-xs"
          :disabled="loading"
          @click="refreshWithOriginals"
          title="刷新"
        >
          <SvgIcon name="refresh" size="14" />
          刷新
        </button>
        <button
          class="btn btn-primary btn-xs"
          :disabled="loading || !hasChanges"
          @click="showSqlPreview"
          title="保存结构"
        >
          <SvgIcon name="save" size="14" />
          保存 ({{ changeCount }})
        </button>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading && !columns.length" class="flex items-center justify-center gap-2 p-12 text-base-content/60 text-sm">
      <SvgIcon name="refresh" size="24" class="animate-spin" />
      <span>加载表结构中...</span>
    </div>

    <!-- 错误状态 -->
    <div v-else-if="error" class="flex items-center gap-2 p-4 m-4 rounded-lg bg-error/10 text-error text-sm">
      <SvgIcon name="alertCircle" size="16" />
      <span>{{ error }}</span>
      <button class="btn btn-ghost btn-xs hover:border-error hover:text-error" @click="refreshWithOriginals">重试</button>
    </div>

    <template v-else>
      <!-- Tab 切换 -->
      <div class="tabs tabs-bordered px-4 shrink-0 bg-base-200" role="tablist">
        <div
          role="tab"
          class="tab tab-sm"
          :class="{ 'tab-active': activeTab === 'columns' }"
          @click="activeTab = 'columns'"
        >
          <SvgIcon name="barChart" size="14" />
          字段 ({{ columns.length }})
          <span v-if="columnChangeCount > 0" class="badge badge-primary badge-sm">{{ columnChangeCount }}</span>
        </div>
        <div
          role="tab"
          class="tab tab-sm"
          :class="{ 'tab-active': activeTab === 'indexes' }"
          @click="activeTab = 'indexes'"
        >
          <SvgIcon name="file" size="14" />
          索引 ({{ groupedIndexes.length }})
          <span v-if="indexChangeCount > 0" class="badge badge-primary badge-sm">{{ indexChangeCount }}</span>
        </div>
      </div>

      <!-- 字段 Tab -->
      <div v-show="activeTab === 'columns'" class="flex-1 flex flex-col overflow-hidden">
        <div class="flex items-center gap-1.5 px-3 py-2 border-b border-base-content/10 shrink-0">
          <button class="btn btn-ghost btn-xs" @click="addColumn" title="添加列">
            <SvgIcon name="plus" size="14" />
            添加列
          </button>
          <button
            class="btn btn-ghost btn-xs hover:border-error hover:text-error"
            :disabled="!selectedColumnIndex"
            @click="deleteSelectedColumn"
            title="删除选中列"
          >
            <SvgIcon name="trash" size="14" />
            删除列
          </button>
          <span class="text-[11px] text-base-content/60 ml-auto">拖拽左侧手柄可调整列顺序</span>
        </div>

        <div class="flex-1 overflow-auto">
          <table class="w-full border-collapse text-xs">
            <thead>
              <tr>
                <th class="sticky top-0 z-[2] px-2 py-1.5 text-left font-semibold text-[11px] text-base-content/60 bg-base-200 border-b border-base-content/10 whitespace-nowrap select-none" style="width:36px"></th>
                <th class="sticky top-0 z-[2] px-2 py-1.5 text-left font-semibold text-[11px] text-base-content/60 bg-base-200 border-b border-base-content/10 whitespace-nowrap select-none" style="width:32px"></th>
                <th class="sticky top-0 z-[2] px-2 py-1.5 text-left font-semibold text-[11px] text-base-content/60 bg-base-200 border-b border-base-content/10 whitespace-nowrap select-none" style="width:150px">字段名</th>
                <th class="sticky top-0 z-[2] px-2 py-1.5 text-left font-semibold text-[11px] text-base-content/60 bg-base-200 border-b border-base-content/10 whitespace-nowrap select-none" style="width:130px">类型</th>
                <th class="sticky top-0 z-[2] px-2 py-1.5 text-left font-semibold text-[11px] text-base-content/60 bg-base-200 border-b border-base-content/10 whitespace-nowrap select-none" style="width:70px">长度</th>
                <th class="sticky top-0 z-[2] px-2 py-1.5 text-left font-semibold text-[11px] text-base-content/60 bg-base-200 border-b border-base-content/10 whitespace-nowrap select-none" style="width:60px">小数</th>
                <th class="sticky top-0 z-[2] px-2 py-1.5 text-left font-semibold text-[11px] text-base-content/60 bg-base-200 border-b border-base-content/10 whitespace-nowrap select-none" style="width:56px">NULL</th>
                <th class="sticky top-0 z-[2] px-2 py-1.5 text-left font-semibold text-[11px] text-base-content/60 bg-base-200 border-b border-base-content/10 whitespace-nowrap select-none" style="width:100px">默认值</th>
                <th class="sticky top-0 z-[2] px-2 py-1.5 text-left font-semibold text-[11px] text-base-content/60 bg-base-200 border-b border-base-content/10 whitespace-nowrap select-none" style="width:56px">主键</th>
                <th class="sticky top-0 z-[2] px-2 py-1.5 text-left font-semibold text-[11px] text-base-content/60 bg-base-200 border-b border-base-content/10 whitespace-nowrap select-none" style="width:56px">自增</th>
                <th class="sticky top-0 z-[2] px-2 py-1.5 text-left font-semibold text-[11px] text-base-content/60 bg-base-200 border-b border-base-content/10 whitespace-nowrap select-none">注释</th>
              </tr>
            </thead>
            <tbody ref="tbodyRef">
              <template v-for="(col, idx) in columns" :key="col._uid">
                <tr
                  :class="[
                    col._isNew ? 'bg-success/10' : '',
                    col._deleted ? 'bg-error/10 line-through opacity-50' : '',
                    !col._isNew && !col._deleted && isColumnModified(col) ? 'bg-warning/10' : '',
                    selectedColumnIndex === idx ? '!bg-primary/10' : '',
                    dragRowIndex === idx ? 'opacity-40' : '',
                    dropTargetIndex === idx && dropPosition === 'before' ? 'drop-before' : '',
                    dropTargetIndex === idx && dropPosition === 'after' ? 'drop-after' : '',
                  ]"
                  :data-row-idx="idx"
                  @click="selectedColumnIndex = idx"
                  @mousedown="onRowMouseDown(idx, $event)"
                >
                  <!-- 拖拽手柄 -->
                  <td class="text-center cursor-grab select-none px-1 py-0.5 border-b border-base-content/10 whitespace-nowrap align-middle active:cursor-grabbing">
                    <span class="inline-block text-base-content/60 text-sm leading-none cursor-grab select-none p-0.5 active:cursor-grabbing" title="拖拽排序">⠿</span>
                  </td>
                  <!-- 选中复选 -->
                  <td class="text-center px-1 py-0.5 border-b border-base-content/10 whitespace-nowrap align-middle">
                    <input
                      type="checkbox"
                      class="checkbox checkbox-xs"
                      :checked="selectedColumnIndex === idx"
                      @click.stop="selectedColumnIndex = selectedColumnIndex === idx ? null : idx"
                    />
                  </td>
                  <!-- 字段名 -->
                  <td class="px-1 py-0.5 border-b border-base-content/10 whitespace-nowrap align-middle">
                    <input
                      class="input input-ghost input-xs w-full font-mono"
                      v-model="col.name"
                      :class="{ 'text-primary': col.primaryKey }"
                      placeholder="字段名"
                      @click.stop
                    />
                  </td>
                  <!-- 类型 -->
                  <td class="px-1 py-0.5 border-b border-base-content/10 whitespace-nowrap align-middle">
                    <select class="select select-ghost select-xs w-full font-mono" v-model="col.type" @click.stop>
                      <option v-for="t in columnTypes" :key="t" :value="t">{{ t }}</option>
                    </select>
                  </td>
                  <!-- 长度 -->
                  <td class="px-1 py-0.5 border-b border-base-content/10 whitespace-nowrap align-middle">
                    <input
                      class="input input-ghost input-xs w-full"
                      type="number"
                      v-model.number="col.length"
                      placeholder="—"
                      min="0"
                      @click.stop
                    />
                  </td>
                  <!-- 小数位 -->
                  <td class="px-1 py-0.5 border-b border-base-content/10 whitespace-nowrap align-middle">
                    <input
                      class="input input-ghost input-xs w-full"
                      type="number"
                      v-model.number="col.decimals"
                      placeholder="—"
                      min="0"
                      @click.stop
                    />
                  </td>
                  <!-- NULL -->
                  <td class="text-center px-1 py-0.5 border-b border-base-content/10 whitespace-nowrap align-middle">
                    <input type="checkbox" class="toggle toggle-sm" v-model="col.nullable" @click.stop title="允许 NULL" />
                  </td>
                  <!-- 默认值 -->
                  <td class="px-1 py-0.5 border-b border-base-content/10 whitespace-nowrap align-middle">
                    <input
                      class="input input-ghost input-xs w-full"
                      v-model="col.defaultValue"
                      placeholder="NULL"
                      @click.stop
                    />
                  </td>
                  <!-- 主键 -->
                  <td class="text-center px-1 py-0.5 border-b border-base-content/10 whitespace-nowrap align-middle">
                    <input type="checkbox" class="toggle toggle-sm" v-model="col.primaryKey" @click.stop @change="onPrimaryKeyChange(col)" title="主键" />
                  </td>
                  <!-- 自增 -->
                  <td class="text-center px-1 py-0.5 border-b border-base-content/10 whitespace-nowrap align-middle">
                    <input type="checkbox" class="toggle toggle-sm" v-model="col.autoIncrement" :disabled="!canAutoIncrement(col)" @click.stop title="自增" />
                  </td>
                  <!-- 注释 -->
                  <td class="px-1 py-0.5 border-b border-base-content/10 whitespace-nowrap align-middle">
                    <input
                      class="input input-ghost input-xs w-full"
                      v-model="col.comment"
                      placeholder="注释"
                      @click.stop
                    />
                  </td>
                </tr>
              </template>
            </tbody>
          </table>
        </div>
      </div>

      <!-- 索引 Tab -->
      <div v-show="activeTab === 'indexes'" class="flex-1 flex flex-col overflow-hidden">
        <div class="flex items-center gap-1.5 px-3 py-2 border-b border-base-content/10 shrink-0">
          <button class="btn btn-ghost btn-xs" @click="addIndex" title="添加索引">
            <SvgIcon name="plus" size="14" />
            添加索引
          </button>
          <button
            class="btn btn-ghost btn-xs hover:border-error hover:text-error"
            :disabled="selectedIndexes.length === 0"
            @click="deleteSelectedIndexes"
            title="删除选中索引"
          >
            <SvgIcon name="trash" size="14" />
            删除索引
          </button>
        </div>

        <div class="flex-1 overflow-auto">
          <table class="w-full border-collapse text-xs">
            <thead>
              <tr>
                <th class="sticky top-0 z-[2] px-2 py-1.5 text-left font-semibold text-[11px] text-base-content/60 bg-base-200 border-b border-base-content/10 whitespace-nowrap select-none" style="width:32px"></th>
                <th class="sticky top-0 z-[2] px-2 py-1.5 text-left font-semibold text-[11px] text-base-content/60 bg-base-200 border-b border-base-content/10 whitespace-nowrap select-none" style="width:140px">索引名称</th>
                <th class="sticky top-0 z-[2] px-2 py-1.5 text-left font-semibold text-[11px] text-base-content/60 bg-base-200 border-b border-base-content/10 whitespace-nowrap select-none" style="width:100px">类型</th>
                <th class="sticky top-0 z-[2] px-2 py-1.5 text-left font-semibold text-[11px] text-base-content/60 bg-base-200 border-b border-base-content/10 whitespace-nowrap select-none">包含列</th>
                <th class="sticky top-0 z-[2] px-2 py-1.5 text-left font-semibold text-[11px] text-base-content/60 bg-base-200 border-b border-base-content/10 whitespace-nowrap select-none" style="width:56px">操作</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="(idx, i) in indexes"
                :key="idx._uid"
                :class="{
                  'bg-success/10': idx._isNew,
                  'bg-error/10 line-through opacity-50': idx._deleted,
                  'bg-warning/10': !idx._isNew && !idx._deleted && isIndexModified(idx)
                }"
              >
                <td class="text-center px-1 py-0.5 border-b border-base-content/10 whitespace-nowrap align-middle">
                  <input
                    type="checkbox"
                    class="checkbox checkbox-xs"
                    :checked="selectedIndexes.includes(i)"
                    @change="toggleIndexSelection(i)"
                  />
                </td>
                <td class="px-1 py-0.5 border-b border-base-content/10 whitespace-nowrap align-middle">
                  <input
                    class="input input-ghost input-xs w-full font-mono"
                    v-model="idx.name"
                    :disabled="idx.type === 'PRIMARY'"
                    placeholder="索引名称"
                  />
                </td>
                <td class="px-1 py-0.5 border-b border-base-content/10 whitespace-nowrap align-middle">
                  <select class="select select-ghost select-xs w-full font-mono" v-model="idx.type">
                    <option value="PRIMARY">PRIMARY</option>
                    <option value="UNIQUE">UNIQUE</option>
                    <option value="INDEX">INDEX</option>
                    <option value="FULLTEXT">FULLTEXT</option>
                  </select>
                </td>
                <td class="px-1 py-0.5 border-b border-base-content/10 whitespace-nowrap align-middle">
                  <div class="flex flex-col gap-0.5">
                    <div v-for="(col, ci) in idx.columns" :key="`idxcol-${idx._uid}-${ci}-${idx.columns[ci] || 'empty'}`" class="flex items-center gap-0.5">
                      <select class="select select-ghost select-xs w-full font-mono" v-model="idx.columns[ci]">
                        <option value="">— 选择列 —</option>
                        <option v-for="c in availableColumnNames" :key="c" :value="c">{{ c }}</option>
                      </select>
                      <button
                        v-if="idx.columns.length > 1"
                        class="btn btn-ghost btn-xs btn-square"
                        @click="removeIndexColumn(idx, ci)"
                        title="移除列"
                      >
                        <SvgIcon name="x" size="12" />
                      </button>
                    </div>
                    <button class="btn btn-ghost btn-xs" @click="addIndexColumn(idx)">
                      + 添加列
                    </button>
                  </div>
                </td>
                <td class="text-center px-1 py-0.5 border-b border-base-content/10 whitespace-nowrap align-middle">
                  <button
                    v-if="idx._isNew"
                    class="btn btn-ghost btn-xs btn-square hover:text-error"
                    @click="deleteIndexAt(i)"
                    title="删除"
                  >
                    <SvgIcon name="trash" size="14" />
                  </button>
                  <span v-else-if="idx._deleted" class="badge badge-error badge-sm">已删除</span>
                </td>
              </tr>
              <tr v-if="indexes.length === 0">
                <td colspan="5" class="text-center p-8 text-base-content/60 text-sm">暂无索引，点击"添加索引"创建</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </template>

    <!-- SQL 预览对话框 -->
    <div v-if="showPreview" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[1000] fade-in" @click.self="showPreview = false">
      <div class="w-[720px] max-w-[90vw] max-h-[80vh] bg-base-100 rounded-xl shadow-2xl flex flex-col overflow-hidden animate-[slideUp_0.2s_ease-out]">
        <div class="flex items-center justify-between px-5 py-4 border-b border-base-content/10">
          <h3 class="flex items-center gap-2 text-lg font-semibold m-0">
            <SvgIcon name="code" size="18" />
            SQL 预览
          </h3>
          <button class="btn btn-ghost btn-xs btn-square" @click="showPreview = false">
            <SvgIcon name="x" size="16" />
          </button>
        </div>
        <div class="px-5 py-4 overflow-y-auto flex-1">
          <p class="text-sm text-base-content/60 m-0 mb-3">即将执行以下 {{ previewSqls.length }} 条 DDL 语句：</p>
          <div class="bg-base-100 border border-base-content/10 rounded-lg p-3 font-mono text-xs leading-relaxed max-h-[400px] overflow-y-auto">
            <div v-for="(sql, i) in previewSqls" :key="i" class="flex gap-2 py-1 border-b border-base-content/10 last:border-b-0">
              <span class="text-base-content/60 select-none min-w-[20px] text-right shrink-0">{{ i + 1 }}</span>
              <code class="text-success break-all">{{ sql }}</code>
            </div>
          </div>
          <div v-if="previewError" class="mt-3 px-3 py-2 rounded-lg bg-error/10 text-error text-xs">{{ previewError }}</div>
        </div>
        <div class="flex items-center justify-end gap-2 px-5 py-3 border-t border-base-content/10">
          <button class="btn btn-ghost btn-xs" @click="showPreview = false">取消</button>
          <button
            class="btn btn-primary btn-xs"
            :disabled="executing"
            @click="executeSqls"
          >
            <SvgIcon v-if="executing" name="refresh" size="14" class="animate-spin" />
            {{ executing ? '执行中...' : '确认执行' }}
          </button>
        </div>
      </div>
    </div>

    <!-- CREATE TABLE SQL Modal -->
    <div v-if="showCreateSqlModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[1000] fade-in" @click.self="showCreateSqlModal = false">
      <div class="w-[900px] max-w-[95vw] max-h-[80vh] bg-base-100 rounded-xl shadow-2xl flex flex-col overflow-hidden animate-[slideUp_0.2s_ease-out]">
        <div class="flex items-center justify-between px-5 py-4 border-b border-base-content/10">
          <h3 class="flex items-center gap-2 text-lg font-semibold m-0">
            <SvgIcon name="code" size="18" />
            建表 SQL — {{ tableName }}
          </h3>
          <button class="btn btn-ghost btn-xs btn-square" @click="showCreateSqlModal = false">
            <SvgIcon name="x" size="16" />
          </button>
        </div>
        <div class="px-5 py-4 overflow-y-auto flex-1">
          <div v-if="loadingCreateSql" class="text-center p-6 text-base-content/60">加载中...</div>
          <pre v-else class="bg-base-200 text-base-content p-4 rounded-lg font-mono text-sm leading-relaxed max-h-[60vh] overflow-auto whitespace-pre-wrap break-all m-0">{{ createSql }}</pre>
        </div>
        <div class="flex items-center justify-end gap-2 px-5 py-3 border-t border-base-content/10">
          <button class="btn btn-ghost btn-xs" @click="showCreateSqlModal = false">关闭</button>
          <button class="btn btn-ghost btn-xs" @click="copyCreateSql" :disabled="!createSql">
            📋 复制 SQL
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { useTableStructure } from '@/composables/useTableStructure'
import { ref, watch, nextTick } from 'vue'

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
  availableColumnNames, groupedIndexes, dragRowIndex, dropTargetIndex, dropPosition,
  canAutoIncrement,
  addColumn, deleteSelectedColumn, addIndex, addIndexColumn,
  removeIndexColumn, deleteIndexAt, deleteSelectedIndexes,
  initDragTable, onRowMouseDown,
  onPrimaryKeyChange, toggleIndexSelection,
  refreshWithOriginals, discardChanges,
  executeSqls, generateDdl, showCreateSql, copyCreateSql,
  showSqlPreview,
  columnTypes, isColumnModified, isIndexModified,
} = ts

const tbodyRef = ref<HTMLElement | null>(null)
// initDragTable when tbody becomes available (it's inside v-else, not rendered at mount time)
watch(() => columns.value.length, async (len) => {
  if (len > 0) {
    await nextTick()
    const el = document.querySelector('.table-structure tbody') as HTMLElement
    if (el) initDragTable(el)
  }
}, { immediate: true })
</script>

<style>
/* Drop indicators for row drag & drop */
.drop-before td:first-child {
  position: relative;
}
.drop-before td:first-child::before {
  content: '';
  position: absolute;
  left: 0;
  right: 0;
  top: -1px;
  height: 2px;
  background: var(--color-primary);
  border-radius: 1px;
  z-index: 3;
}
.drop-after td:first-child {
  position: relative;
}
.drop-after td:first-child::after {
  content: '';
  position: absolute;
  left: 0;
  right: 0;
  bottom: -1px;
  height: 2px;
  background: var(--color-primary);
  border-radius: 1px;
  z-index: 3;
}
</style>
