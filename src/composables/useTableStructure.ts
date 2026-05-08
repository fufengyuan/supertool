// @ts-nocheck
import { ref, computed, onMounted } from 'vue'
import { getErrorMessage } from '../utils/helpers'
import { getTauriAPI } from '../utils/tauri-api'

export function useTableStructure(
  props: {
    connId: string; tableName: string; dbName?: string; dbType?: string; connectionId?: string },
  _emit: unknown
) {
  console.log("[useTableStructure.ts] useTableStructure() init")


// ============ Props ============

// ============ Types ============
function uid(): number {
  return Date.now() + Math.floor(Math.random() * 10000)
}

interface ColumnDef {
  _uid: number
  _isNew: boolean
  _deleted: boolean
  _originalName?: string
  _orderChanged?: boolean
  name: string
  type: string
  length: number | null
  decimals: number | null
  nullable: boolean
  defaultValue: string | null
  primaryKey: boolean
  autoIncrement: boolean
  comment: string
  _originalData?: RawColumnData
}

interface IndexDef {
  _uid: number
  _isNew: boolean
  _deleted: boolean
  _originalName?: string
  name: string
  type: 'PRIMARY' | 'UNIQUE' | 'INDEX' | 'FULLTEXT'
  columns: string[]
  _originalData?: RawIndexRow[]
}

/** Raw column data from backend (SHOW FULL COLUMNS result) */
interface RawColumnData {
  COLUMN_NAME?: string
  name?: string
  COLUMN_TYPE?: string
  type?: string
  length?: number
  decimals?: number
  IS_NULLABLE?: string | boolean
  nullable?: boolean
  COLUMN_DEFAULT?: string | null
  default?: string | null
  COLUMN_KEY?: string
  key?: string
  EXTRA?: string
  autoIncrement?: boolean
  primaryKey?: boolean
  COLUMN_COMMENT?: string
  comment?: string
}

/** Raw index row from backend (SHOW INDEX result) */
interface RawIndexRow {
  Key_name?: string
  name?: string
  Non_unique?: number
  non_unique?: number
  Column_name?: string
  column_name?: string
}

// ============ State ============
const columns = ref<ColumnDef[]>([])
const indexes = ref<IndexDef[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const activeTab = ref<'columns' | 'indexes'>('columns')
const selectedColumnIndex = ref<number | null>(null)
const selectedIndexes = ref<number[]>([])
const showPreview = ref(false)
const previewSqls = ref<string[]>([])
const showCreateSqlModal = ref(false)
const createSql = ref('')
const loadingCreateSql = ref(false)
const previewError = ref<string | null>(null)
const executing = ref(false)
const dragIndex = ref<number | null>(null)
const dragOverIndex = ref<number | null>(null)
const dragOverPosition = ref<'before' | 'after'>('before')

// ============ Column Types ============
const columnTypes = [
  'TINYINT', 'SMALLINT', 'MEDIUMINT', 'INT', 'BIGINT',
  'FLOAT', 'DOUBLE', 'DECIMAL',
  'BOOLEAN',
  'CHAR', 'VARCHAR', 'TINYTEXT', 'TEXT', 'MEDIUMTEXT', 'LONGTEXT',
  'DATE', 'TIME', 'DATETIME', 'TIMESTAMP', 'YEAR',
  'BLOB', 'TINYBLOB', 'MEDIUMBLOB', 'LONGBLOB', 'BINARY', 'VARBINARY',
  'JSON',
  'ENUM', 'SET',
  'BIT',
]

// ============ Computed ============
const dbTypeLabel = computed(() => {
  const map: Record<string, string> = { mysql: 'MySQL', postgresql: 'PostgreSQL', sqlite: 'SQLite' }
  return props.dbType ? (map[props.dbType] || props.dbType) : ''
})

const availableColumnNames = computed(() =>
  columns.value.filter(c => !c._deleted && c.name).map(c => c.name)
)

const groupedIndexes = computed(() => indexes.value.filter(i => !i._deleted))

const columnChangeCount = computed(() => {
  let c = 0
  columns.value.forEach(col => {
    if (col._isNew || col._deleted) c++
    else if (isColumnModified(col)) c++
  })
  return c
})

const indexChangeCount = computed(() => {
  let c = 0
  indexes.value.forEach(idx => {
    if (idx._isNew || idx._deleted) c++
    else if (isIndexModified(idx)) c++
  })
  return c
})

const hasChanges = computed(() => columnChangeCount.value > 0 || indexChangeCount.value > 0)
const changeCount = computed(() => columnChangeCount.value + indexChangeCount.value)

// ============ Data Loading ============

function parseColumnType(raw: string): { baseType: string; length: number | null; decimals: number | null } {
  if (!raw) return { baseType: 'VARCHAR', length: null, decimals: null }
  // Handle PostgreSQL multi-word types like "character varying(255)", "double precision"
  const pgMultiWord: Record<string, string> = {
    'CHARACTER VARYING': 'VARCHAR',
    'CHARACTER': 'CHAR',
    'DOUBLE PRECISION': 'DOUBLE',
    'TIMESTAMP WITHOUT TIME ZONE': 'TIMESTAMP',
    'TIMESTAMP WITH TIME ZONE': 'TIMESTAMPTZ',
    'TIME WITHOUT TIME ZONE': 'TIME',
    'TIME WITH TIME ZONE': 'TIMETZ',
  }
  const rawUpper = raw.toUpperCase()
  for (const [pgType, normalized] of Object.entries(pgMultiWord)) {
    if (rawUpper.startsWith(pgType)) {
      const rest = rawUpper.slice(pgType.length).trim()
      const m = rest.match(/^\((\d+)(?:,(\d+))?\)/)
      return {
        baseType: normalized,
        length: m ? parseInt(m[1], 10) : null,
        decimals: m?.[2] ? parseInt(m[2], 10) : null,
      }
    }
  }
  // Fallback: simple type(name) or type(name,decimals)
  const m = raw.match(/^([a-zA-Z]+)(?:\((\d+)(?:,(\d+))?\))?$/)
  if (m) {
    return {
      baseType: m[1].toUpperCase(),
      length: m[2] ? parseInt(m[2], 10) : null,
      decimals: m[3] ? parseInt(m[3], 10) : null,
    }
  }
  return { baseType: raw.toUpperCase(), length: null, decimals: null }
}

// ============ Column Operations ============
function addColumn() {
  columns.value.push({
    _uid: uid(),
    _isNew: true,
    _deleted: false,
    name: '',
    type: 'VARCHAR',
    length: 255,
    decimals: null,
    nullable: true,
    defaultValue: null,
    primaryKey: false,
    autoIncrement: false,
    comment: '',
  })
  selectedColumnIndex.value = columns.value.length - 1
}

function deleteSelectedColumn() {
  if (selectedColumnIndex.value === null) return
  const col = columns.value[selectedColumnIndex.value]
  if (col._isNew) {
    columns.value.splice(selectedColumnIndex.value, 1)
  } else {
    col._deleted = true
  }
  selectedColumnIndex.value = null
}

function onPrimaryKeyChange(col: ColumnDef) {
  if (col.primaryKey) {
    // Unset other primary keys
    columns.value.forEach(c => {
      if (c !== col) c.primaryKey = false
    })
  }
}

function canAutoIncrement(col: ColumnDef): boolean {
  const numericTypes = ['TINYINT', 'SMALLINT', 'MEDIUMINT', 'INT', 'BIGINT']
  return numericTypes.includes(col.type.toUpperCase()) && col.primaryKey
}

function isColumnModified(col: ColumnDef): boolean {
  if (col._isNew || col._deleted) return false
  const orig = findOriginalColumn(col)
  if (!orig) return false
  const { baseType: origType } = parseColumnType(orig.COLUMN_TYPE || orig.type || '')
  const origPk = orig.primaryKey === true || (orig.COLUMN_KEY || orig.key) === 'PRI' || orig.primaryKey === true
  const origAi = orig.autoIncrement === true || (orig.EXTRA || '').includes('auto_increment') || orig.autoIncrement === true
  // Use originalData's length/decimals directly (from INFORMATION_SCHEMA) instead of re-parsing from type string
  const origLen = orig.length ?? null
  const origDec = orig.decimals ?? null
  const origDefault = (orig as any).COLUMN_DEFAULT ?? (orig as any).default ?? (orig as any).defaultValue ?? null
  const origComment = (orig as any).COLUMN_COMMENT ?? (orig as any).comment ?? ''
  return (
    col._orderChanged === true ||
    col.name !== (orig.COLUMN_NAME || orig.name) ||
    col.type !== origType ||
    col.length !== origLen ||
    col.decimals !== origDec ||
    (col.nullable ? 'YES' : 'NO') !== (orig.IS_NULLABLE || (orig.nullable ? 'YES' : 'NO')) ||
    normalizeDefault(col.defaultValue) !== normalizeDefault(origDefault) ||
    col.primaryKey !== origPk ||
    col.autoIncrement !== origAi ||
    col.comment !== origComment
  )
}

function findOriginalColumn(col: ColumnDef): RawColumnData | null {
  return col._originalData || null
}

// Store original data on load
const _originalColumns = ref<Map<number, RawColumnData>>(new Map())

// Override refresh to store originals
async function refreshWithOriginals() {
  loading.value = true
  error.value = null
  try {
    const api = getTauriAPI()
    if (!api) {
      error.value = 'API 不可用'
      return
    }
    const res = await api.dbGetTableStructure(
      props.connId,
      props.tableName,
      props.dbName || undefined
    )
    
    // 兼容处理：现在的 API 直接返回 rows 数组 (包含列信息)
    // 或者返回 { rows: [...columns], indexes: [...] }
    const columns = Array.isArray(res) ? res : (res?.rows ?? [])
    const indexes = res?.indexes ?? []
    
    if (!columns || columns.length === 0) {
      error.value = '表结构数据为空'
      return
    }

    const rawCols = columns as RawColumnData[]
    const rawIdxs = indexes as any[]
    _originalColumns.value.clear()

    columns.value = rawCols.map((c: RawColumnData) => {
      // Backend now returns separate length/decimals fields.
      // Fall back to parseColumnType for backward compat (old responses with type='varchar(255)').
      let baseType: string, length: number | null, decimals: number | null
      if (c.length != null || c.decimals != null) {
        // New format: separate fields
        baseType = (c.COLUMN_TYPE || c.type || '').toUpperCase() || 'VARCHAR'
        length = c.length != null ? c.length : null
        decimals = c.decimals != null ? c.decimals : null
      } else {
        // Old format: type contains length like 'varchar(255)'
        const parsed = parseColumnType(c.COLUMN_TYPE || c.type || '')
        baseType = parsed.baseType
        length = parsed.length
        decimals = parsed.decimals
      }
      // Backend returns { isPrimaryKey, isAutoIncrement } (new format)
      // or { COLUMN_KEY, EXTRA } (legacy format)
      const col: ColumnDef = {
        _uid: uid(),
        _isNew: false,
        _deleted: false,
        _originalName: c.COLUMN_NAME || c.name,
        name: c.COLUMN_NAME || c.name || '',
        type: baseType || 'VARCHAR',
        length,
        decimals,
        nullable: (c.IS_NULLABLE || c.nullable) === 'YES' || (c.IS_NULLABLE || c.nullable) === true,
        defaultValue: c.COLUMN_DEFAULT ?? c.default ?? (c as any).defaultValue ?? null,
        primaryKey: c.primaryKey === true || (c.COLUMN_KEY || c.key) === 'PRI' || c.primaryKey === true,
        autoIncrement: c.autoIncrement === true || (c.EXTRA || '').includes('auto_increment') || c.autoIncrement === true,
        comment: c.COLUMN_COMMENT || c.comment || (c as any).comment || '',
        _originalData: { ...c },
      }
      _originalColumns.value.set(col._uid, { ...c })
      return col
    })

    // rawIdxs already declared above, reuse it or reassign
    const idxData = (struct.indexes as any[]) ?? []
    indexes.value = idxData.map((idx: any) => {
      // Backend new format: { name, columns: ['col1', 'col2'], isUnique, isPrimary }
      // Backend legacy format: [{ Key_name, Column_name, Non_unique }, ...]
      const isPrimary = idx.isPrimary === true || idx.name === 'PRIMARY'
      const isUnique = idx.isUnique === true || (idx.Non_unique ?? idx.non_unique ?? 1) === 0

      // Get columns array from either format
      let cols: string[]
      if (Array.isArray(idx.columns) && idx.columns.length > 0) {
        cols = [...idx.columns]
      } else {
        const cn = idx.Column_name || idx.column_name || ''
        cols = cn ? [cn] : []
      }

      // Build _originalData in legacy format so isIndexModified works correctly
      const origData: RawIndexRow[] = cols.map(cn => ({
        Key_name: idx.name || '',
        Column_name: cn,
        Non_unique: isUnique ? 0 : 1,
      }))

      return {
        _uid: uid(),
        _isNew: false,
        _deleted: false,
        _originalName: idx.name || '',
        name: idx.name || '',
        type: isPrimary ? 'PRIMARY' : (isUnique ? 'UNIQUE' : 'INDEX'),
        columns: cols,
        _originalData: origData,
      }
    })

    selectedColumnIndex.value = null
    selectedIndexes.value = []
  } catch (e: unknown) {
    error.value = getErrorMessage(e) || '加载表结构失败'
  } finally {
    loading.value = false
  }
}

function discardChanges() {
  refreshWithOriginals()
}

// ============ Drag & Drop ============
function onDragStart(index: number, event: DragEvent) {
  dragIndex.value = index
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move'
    event.dataTransfer.setData('text/plain', String(index))
  }
}

function onDragOver(index: number, event: DragEvent) {
  event.preventDefault()
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move'
  }
  // Calculate whether to drop before or after the target row
  const rect = (event.target as HTMLElement)?.closest?.('tr')?.getBoundingClientRect()
  if (rect) {
    const mouseY = event.clientY
    const midY = rect.top + rect.height / 2
    dragOverIndex.value = index
    dragOverPosition.value = mouseY < midY ? 'before' : 'after'
  }
}

function onDrop(targetIndex: number, _event: DragEvent) {
  if (dragIndex.value === null || dragIndex.value === targetIndex) return
  // Adjust target index based on position
  let insertIndex = targetIndex
  if (dragOverPosition.value === 'after' && dragIndex.value < targetIndex) {
    insertIndex = targetIndex + 1
  } else if (dragOverPosition.value === 'before' && dragIndex.value > targetIndex) {
    insertIndex = targetIndex
  }

  const item = columns.value.splice(dragIndex.value, 1)[0]
  columns.value.splice(insertIndex, 0, item)

  if (!item._isNew && !item._deleted) {
    item._orderChanged = true
  }
  dragIndex.value = null
  dragOverIndex.value = null
}

function onDragEnd() {
  dragIndex.value = null
  dragOverIndex.value = null
}

// ============ Index Operations ============
function addIndex() {
  indexes.value.push({
    _uid: uid(),
    _isNew: true,
    _deleted: false,
    name: 'idx_' + Date.now().toString(36),
    type: 'INDEX',
    columns: [''],
  })
}

function deleteIndexAt(i: number) {
  const idx = indexes.value[i]
  if (idx._isNew) {
    indexes.value.splice(i, 1)
  } else {
    idx._deleted = true
  }
  selectedIndexes.value = selectedIndexes.value.filter(x => x !== i)
}

function deleteSelectedIndexes() {
  const toDelete = [...selectedIndexes.value].sort((a, b) => b - a)
  for (const i of toDelete) {
    deleteIndexAt(i)
  }
  selectedIndexes.value = []
}

function toggleIndexSelection(i: number) {
  const idx = selectedIndexes.value.indexOf(i)
  if (idx >= 0) selectedIndexes.value.splice(idx, 1)
  else selectedIndexes.value.push(i)
}

function addIndexColumn(idx: IndexDef) {
  idx.columns.push('')
}

function removeIndexColumn(idx: IndexDef, ci: number) {
  idx.columns.splice(ci, 1)
}

function isIndexModified(idx: IndexDef): boolean {
  if (idx._isNew || idx._deleted) return false
  const origData = idx._originalData
  if (!origData || !origData.length) return false
  const origCols = origData.map((r: RawIndexRow) => r.Column_name || r.column_name || '')
  const origName = idx._originalName || ''
  const _origType = idx.type
  const origNonUnique = origData[0]?.Non_unique ?? origData[0]?.non_unique ?? 1
  const origTypeComputed = origName === 'PRIMARY' ? 'PRIMARY' : (origNonUnique === 0 ? 'UNIQUE' : 'INDEX')

  return (
    idx.name !== origName ||
    idx.type !== origTypeComputed ||
    JSON.stringify(idx.columns.filter(Boolean)) !== JSON.stringify(origCols.filter(Boolean))
  )
}

// ============ SQL Generation ============
function generateDdl(): string[] {
  const sqls: string[] = []
  const db = props.dbType || 'mysql'
  const safeTable = quoteIdent(props.tableName, db)
  const safeDb = props.dbName ? quoteIdent(props.dbName, db) + '.' : ''

  // Build the list of existing (non-new, non-deleted) columns in their CURRENT order
  // This is used to determine the AFTER clause for reordering
  const activeColumns = columns.value.filter(c => !c._deleted)

  // --- Column changes ---
  for (let i = 0; i < activeColumns.length; i++) {
    const col = activeColumns[i]
    if (col._isNew) {
      if (!col.name) continue
      const colDef = buildColumnDef(col, db)
      // New column: determine placement
      let afterClause = ''
      if (i === 0) {
        afterClause = ' FIRST'
      } else {
        const prevCol = activeColumns[i - 1]
        if (prevCol && prevCol.name) {
          afterClause = ` AFTER ${quoteIdent(prevCol.name, db)}`
        }
      }
      sqls.push(`ALTER TABLE ${safeDb}${safeTable} ADD COLUMN ${quoteIdent(col.name, db)} ${colDef}${afterClause};`)
      continue
    }

    // Existing column — check if anything changed
    if (!col.name || !col._originalName) continue
    const orig = col._originalData
    if (!orig && !col._orderChanged) continue

    const isRename = col.name !== (col._originalName || '')
    const attrsChanged = isTypeChangeAttrs(col, orig)

    if (!isRename && !attrsChanged && !col._orderChanged) continue

    // Generate precise ALTER SQL
    if (db === 'mysql') {
      const colDef = buildColumnDef(col, db)
      // Determine AFTER clause for reordering
      let afterClause = ''
      if (i === 0) {
        afterClause = ' FIRST'
      } else {
        const prevCol = activeColumns[i - 1]
        if (prevCol && prevCol.name) {
          afterClause = ` AFTER ${quoteIdent(prevCol.name, db)}`
        }
      }

      if (isRename) {
        // CHANGE COLUMN handles both rename and reorder in one statement
        sqls.push(`ALTER TABLE ${safeDb}${safeTable} CHANGE COLUMN ${quoteIdent(col._originalName, db)} ${quoteIdent(col.name, db)} ${colDef}${afterClause};`)
      } else {
        // MODIFY COLUMN for type changes and/or reordering
        sqls.push(`ALTER TABLE ${safeDb}${safeTable} MODIFY COLUMN ${quoteIdent(col.name, db)} ${colDef}${afterClause};`)
      }
    } else if (db === 'postgresql') {
      // PG: handle rename, type, nullable, default, comment separately
      if (isRename) {
        sqls.push(`ALTER TABLE ${safeDb}${safeTable} RENAME COLUMN ${quoteIdent(col._originalName, db)} TO ${quoteIdent(col.name, db)};`)
      }
      if (attrsChanged && orig) {
        const { baseType: origType } = parseColumnType(orig.COLUMN_TYPE || orig.type || '')
        const newType = mapTypeToPg(col.type.toUpperCase())
        if (origType.toUpperCase() !== newType || col.length !== null || col.decimals !== null) {
          sqls.push(`ALTER TABLE ${safeDb}${safeTable} ALTER COLUMN ${quoteIdent(col.name, db)} TYPE ${buildTypeFull(col, db)};`)
        }
      }
      if (orig) {
        const origNullable = (orig.IS_NULLABLE || (orig.nullable ? 'YES' : 'NO')) === 'YES'
        if (col.nullable !== origNullable) {
          if (col.nullable) {
            sqls.push(`ALTER TABLE ${safeDb}${safeTable} ALTER COLUMN ${quoteIdent(col.name, db)} DROP NOT NULL;`)
          } else {
            sqls.push(`ALTER TABLE ${safeDb}${safeTable} ALTER COLUMN ${quoteIdent(col.name, db)} SET NOT NULL;`)
          }
        }
        const origDefault = orig.COLUMN_DEFAULT ?? orig.default ?? null
        if (col.defaultValue !== origDefault) {
          if (col.defaultValue != null && col.defaultValue !== '') {
            sqls.push(`ALTER TABLE ${safeDb}${safeTable} ALTER COLUMN ${quoteIdent(col.name, db)} SET DEFAULT ${buildDefaultValue(col.defaultValue, col.type, db)};`)
          } else {
            sqls.push(`ALTER TABLE ${safeDb}${safeTable} ALTER COLUMN ${quoteIdent(col.name, db)} DROP DEFAULT;`)
          }
        }
        const origComment = orig.COLUMN_COMMENT || orig.comment || ''
        if (col.comment !== origComment) {
          if (col.comment) {
            sqls.push(`COMMENT ON COLUMN ${safeDb}${safeTable}.${quoteIdent(col.name, db)} IS '${col.comment.replace(/'/g, "''")}';`)
          } else {
            sqls.push(`COMMENT ON COLUMN ${safeDb}${safeTable}.${quoteIdent(col.name, db)} IS NULL;`)
          }
        }
      }
      // PG cannot reorder columns natively — skip silently
    } else {
      // SQLite: limited ALTER TABLE support
      if (isRename) {
        sqls.push(`ALTER TABLE ${safeDb}${safeTable} RENAME COLUMN ${quoteIdent(col._originalName, db)} TO ${quoteIdent(col.name, db)};`)
      }
      // SQLite cannot reorder, change type, or add NOT NULL easily — skip
    }
  }

  // --- Primary Key changes ---
  if (db === 'mysql' || db === 'postgresql') {
    const origPkCols: string[] = []
    for (const col of columns.value) {
      if (!col._isNew && !col._deleted && col._originalData) {
        const origPk = col._originalData.primaryKey === true
          || (col._originalData.COLUMN_KEY || col._originalData.key) === 'PRI'
          || col._originalData.primaryKey === true
        if (origPk) {
          origPkCols.push(col._originalName || col.name)
        }
      }
    }
    const newPkCols = columns.value.filter(c => !c._deleted && c.primaryKey && c.name).map(c => c.name)

    const pkChanged = JSON.stringify(origPkCols.sort()) !== JSON.stringify(newPkCols.sort())
    if (pkChanged) {
      if (origPkCols.length > 0) {
        sqls.push(`ALTER TABLE ${safeDb}${safeTable} DROP PRIMARY KEY;`)
      }
      if (newPkCols.length > 0) {
        const cols = newPkCols.map(c => quoteIdent(c, db)).join(', ')
        sqls.push(`ALTER TABLE ${safeDb}${safeTable} ADD PRIMARY KEY (${cols});`)
      }
    }
  }

  // --- AutoIncrement changes (MySQL only — requires MODIFY COLUMN) ---
  if (db === 'mysql') {
    for (const col of activeColumns) {
      if (col._isNew || col._deleted || !col._originalData) continue
      const origAi = col._originalData.autoIncrement === true
        || (col._originalData.EXTRA || '').includes('auto_increment')
        || col._originalData.autoIncrement === true
      if (col.autoIncrement !== origAi) {
        const colDef = buildColumnDef(col, db)
        sqls.push(`ALTER TABLE ${safeDb}${safeTable} MODIFY COLUMN ${quoteIdent(col.name, db)} ${colDef};`)
      }
    }
  }

  // --- Index changes (skip PRIMARY — handled above) ---
  for (const idx of indexes.value) {
    if (idx.type === 'PRIMARY') continue
    if (idx._deleted && idx._originalName) {
      if (idx.type === 'PRIMARY') {
        sqls.push(`ALTER TABLE ${safeDb}${safeTable} DROP PRIMARY KEY;`)
      } else {
        const dropSyntax = db === 'postgresql'
          ? `DROP INDEX ${quoteIdent(idx._originalName, db)};`
          : `DROP INDEX ${quoteIdent(idx._originalName, db)} ON ${safeDb}${safeTable};`
        sqls.push(dropSyntax)
      }
      continue
    }
    if (idx._isNew) {
      const validCols = idx.columns.filter(c => c && c.trim())
      if (validCols.length === 0) continue
      const createIdx = buildCreateIndex(idx, safeDb, safeTable, db)
      if (createIdx) sqls.push(createIdx)
      continue
    }
    // Modified index - drop and recreate
    if (isIndexModified(idx)) {
      if (idx._originalName) {
        if (idx.type === 'PRIMARY') {
          sqls.push(`ALTER TABLE ${safeDb}${safeTable} DROP PRIMARY KEY;`)
        } else {
          const dropSyntax = db === 'postgresql'
            ? `DROP INDEX ${quoteIdent(idx._originalName, db)};`
            : `DROP INDEX ${quoteIdent(idx._originalName, db)} ON ${safeDb}${safeTable};`
          sqls.push(dropSyntax)
        }
      }
      const validCols = idx.columns.filter(c => c && c.trim())
      if (validCols.length > 0) {
        const createIdx = buildCreateIndex(idx, safeDb, safeTable, db)
        if (createIdx) sqls.push(createIdx)
      }
    }
  }

  return sqls
}

/** Check if only type-related attributes changed (excluding rename, order, PK, and autoIncrement) */
function isTypeChangeAttrs(col: ColumnDef, orig: RawColumnData | undefined): boolean {
  if (!orig) return false
  const { baseType: origType } = parseColumnType(orig.COLUMN_TYPE || orig.type || '')
  // Use originalData's length/decimals directly instead of re-parsing from type string
  const origLen = orig.length ?? null
  const origDec = orig.decimals ?? null
  const origDefault = (orig as any).COLUMN_DEFAULT ?? (orig as any).default ?? (orig as any).defaultValue ?? null
  const origComment = (orig as any).COLUMN_COMMENT ?? (orig as any).comment ?? ''
  return (
    col.type !== origType ||
    col.length !== origLen ||
    col.decimals !== origDec ||
    (col.nullable ? 'YES' : 'NO') !== (orig.IS_NULLABLE || (orig.nullable ? 'YES' : 'NO')) ||
    normalizeDefault(col.defaultValue) !== normalizeDefault(origDefault) ||
    col.comment !== origComment
  )
}

/** Normalize default values for comparison — handles null/undefined/empty string equivalence */
function normalizeDefault(v: string | null | undefined): string | null {
  if (v == null || v === '') return null
  return String(v)
}

/** Build type string with length/decimals for DDL */
function buildTypeString(col: ColumnDef): string {
  let typeStr = col.type.toUpperCase()
  if (col.length != null && col.length > 0) {
    if (col.decimals != null && col.decimals >= 0 && ['DECIMAL', 'FLOAT', 'DOUBLE'].includes(typeStr)) {
      typeStr += `(${col.length},${col.decimals})`
    } else if (!['TEXT', 'BLOB', 'JSON', 'DATE', 'TIME', 'DATETIME', 'TIMESTAMP', 'YEAR', 'BOOLEAN'].includes(typeStr)) {
      typeStr += `(${col.length})`
    }
  }
  return typeStr
}

/** Build full type declaration for PG ALTER COLUMN TYPE */
function buildTypeFull(col: ColumnDef, db: string): string {
  let typeStr = col.type.toUpperCase()
  if (col.length != null && col.length > 0) {
    if (col.decimals != null && col.decimals >= 0 && ['DECIMAL', 'FLOAT', 'DOUBLE'].includes(typeStr)) {
      typeStr += `(${col.length},${col.decimals})`
    } else if (!['TEXT', 'BLOB', 'JSON', 'DATE', 'TIME', 'DATETIME', 'TIMESTAMP', 'YEAR', 'BOOLEAN'].includes(typeStr)) {
      typeStr += `(${col.length})`
    }
  }
  if (db === 'postgresql') typeStr = mapTypeToPg(typeStr)
  return typeStr
}

function buildColumnDef(col: ColumnDef, db: string): string {
  let typeStr = col.type.toUpperCase()

  // Length / decimals
  if (col.length != null && col.length > 0) {
    if (col.decimals != null && col.decimals >= 0 && ['DECIMAL', 'FLOAT', 'DOUBLE'].includes(typeStr)) {
      typeStr += `(${col.length},${col.decimals})`
    } else if (!['TEXT', 'BLOB', 'JSON', 'DATE', 'TIME', 'DATETIME', 'TIMESTAMP', 'YEAR', 'BOOLEAN'].includes(typeStr)) {
      typeStr += `(${col.length})`
    }
  }

  // PostgreSQL type mapping
  if (db === 'postgresql') {
    typeStr = mapTypeToPg(typeStr)
  }
  if (db === 'sqlite') {
    typeStr = mapTypeToSqlite(typeStr)
  }

  let def = typeStr

  // NOT NULL
  if (!col.nullable && !col.primaryKey) {
    def += ' NOT NULL'
  } else if (col.nullable && db === 'postgresql') {
    def += ' NULL'
  }

  // Default value
  if (col.defaultValue != null && col.defaultValue !== '') {
    def += ` DEFAULT ${buildDefaultValue(col.defaultValue, col.type, db)}`
  }

  // Auto increment
  if (col.autoIncrement && db === 'mysql') {
    def += ' AUTO_INCREMENT'
  }

  // Primary key (inline for new columns)
  if (col.primaryKey && col._isNew && db === 'mysql') {
    def += ' PRIMARY KEY'
  }

  // Comment (MySQL only)
  if (col.comment && db === 'mysql') {
    def += ` COMMENT '${col.comment.replace(/'/g, "''")}'`
  }

  return def
}

function mapTypeToPg(type: string): string {
  const map: Record<string, string> = {
    'TINYINT': 'SMALLINT',
    'MEDIUMINT': 'INTEGER',
    'BIGINT': 'BIGINT',
    'INT': 'INTEGER',
    'FLOAT': 'REAL',
    'DOUBLE': 'DOUBLE PRECISION',
    'DECIMAL': 'NUMERIC',
    'BOOLEAN': 'BOOLEAN',
    'DATETIME': 'TIMESTAMP',
    'TIMESTAMP': 'TIMESTAMP',
    'TINYTEXT': 'TEXT',
    'MEDIUMTEXT': 'TEXT',
    'LONGTEXT': 'TEXT',
    'TINYBLOB': 'BYTEA',
    'MEDIUMBLOB': 'BYTEA',
    'LONGBLOB': 'BYTEA',
    'BLOB': 'BYTEA',
    'JSON': 'JSONB',
    'BINARY': 'BYTEA',
    'VARBINARY': 'BYTEA',
  }
  return map[type] || type
}

function mapTypeToSqlite(type: string): string {
  const map: Record<string, string> = {
    'TINYINT': 'INTEGER', 'SMALLINT': 'INTEGER', 'MEDIUMINT': 'INTEGER',
    'INT': 'INTEGER', 'BIGINT': 'INTEGER',
    'FLOAT': 'REAL', 'DOUBLE': 'REAL', 'DECIMAL': 'REAL',
    'BOOLEAN': 'INTEGER',
    'VARCHAR': 'TEXT', 'CHAR': 'TEXT',
    'TINYTEXT': 'TEXT', 'TEXT': 'TEXT', 'MEDIUMTEXT': 'TEXT', 'LONGTEXT': 'TEXT',
    'BLOB': 'BLOB', 'TINYBLOB': 'BLOB', 'MEDIUMBLOB': 'BLOB', 'LONGBLOB': 'BLOB',
    'JSON': 'TEXT',
  }
  return map[type] || type
}

function buildDefaultValue(val: string, type: string, _db: string): string {
  const upper = val.toUpperCase().trim()
  if (upper === 'NULL' || upper === 'CURRENT_TIMESTAMP' || upper === 'NOW()' || upper === 'CURRENT_DATE') {
    return val
  }
  // Sequence / function defaults (PG: nextval, gen_random_uuid; MySQL: uuid, etc.)
  if (/^(nextval|currval|setval|gen_random_uuid|uuid_generate|uuid)\s*\(/i.test(val)) {
    return val
  }
  if (type.toUpperCase().includes('INT') || type.toUpperCase() === 'BIGINT' || type.toUpperCase() === 'DECIMAL' || type.toUpperCase() === 'FLOAT' || type.toUpperCase() === 'DOUBLE') {
    if (!isNaN(Number(val))) return val
  }
  return `'${val.replace(/'/g, "''")}'`
}

function buildCreateIndex(idx: IndexDef, safeDb: string, safeTable: string, db: string): string | null {
  const cols = idx.columns.filter(c => c && c.trim())
  if (cols.length === 0) return null

  const idxName = idx.name || 'idx_' + cols.join('_')

  if (idx.type === 'PRIMARY') {
    const colList = cols.map(c => quoteIdent(c, db)).join(', ')
    if (db === 'postgresql') {
      const pgCols = cols.map(c => quoteIdent(c, db)).join(', ')
      return `ALTER TABLE ${safeDb}${safeTable} ADD CONSTRAINT ${quoteIdent(idxName, db)} PRIMARY KEY (${pgCols});`
    }
    return `ALTER TABLE ${safeDb}${safeTable} ADD PRIMARY KEY (${colList});`
  }

  const unique = idx.type === 'UNIQUE' ? 'UNIQUE ' : ''
  const fulltext = idx.type === 'FULLTEXT' ? 'FULLTEXT ' : ''

  if (db === 'postgresql') {
    const pgCols = cols.map(c => quoteIdent(c, db)).join(', ')
    return `CREATE ${unique}INDEX ${quoteIdent(idxName, db)} ON ${safeDb}${safeTable} (${pgCols});`
  }
  if (db === 'sqlite') {
    const sqCols = cols.map(c => quoteIdent(c, db)).join(', ')
    return `CREATE ${unique}INDEX ${quoteIdent(idxName, db)} ON ${safeDb}${safeTable} (${sqCols});`
  }
  const colList = cols.map(c => quoteIdent(c, db)).join(', ')
  return `CREATE ${unique}${fulltext}INDEX ${quoteIdent(idxName, db)} ON ${safeDb}${safeTable} (${colList});`
}

function quoteIdent(name: string, db: string): string {
  if (db === 'mysql') return '`' + name.replace(/`/g, '``') + '`'
  return '"' + name.replace(/"/g, '""') + '"'
}

function _safeIdent(name: string, dbType?: string): string {
  // kept for legacy compatibility but quoteIdent should be used for DDL
  if (dbType === 'mysql') {
    return '`' + name.replace(/`/g, '``') + '`'
  }
  return '"' + name.replace(/"/g, '""') + '"'
}

// ============ SQL Preview & Execute ============
function showSqlPreview() {
  const sqls = generateDdl()
  if (sqls.length === 0) {
    return
  }
  previewSqls.value = sqls
  previewError.value = null
  showPreview.value = true
}

async function showCreateSql() {
  loadingCreateSql.value = true
  createSql.value = ''
  showCreateSqlModal.value = true
  try {
    const res = await getTauriAPI().dbGetCreateSql(props.connId, props.tableName, props.dbName || undefined)
    if (res?.success && res.sql) {
      createSql.value = res.sql
    } else {
      createSql.value = `-- 获取失败: ${res?.error || '未知错误'}`
    }
  } catch (e: unknown) {
    createSql.value = `-- 获取失败: ${getErrorMessage(e) || '未知错误'}`
  } finally {
    loadingCreateSql.value = false
  }
}

function copyCreateSql() {
  navigator.clipboard?.writeText(createSql.value)
}

/** IPC response for DDL execution */
interface DdlExecutionResponse {
  success: boolean
  error?: string
}

async function executeSqls() {
  executing.value = true
  previewError.value = null
  try {
    // Try db:execute-structure-sync first (if available)
    const api = getTauriAPI()
    let result: DdlExecutionResponse | undefined

    if (typeof api?.dbExecuteStructureSync === 'function') {
      result = await api.dbExecuteStructureSync(props.connId, JSON.parse(JSON.stringify(previewSqls.value)), props.dbName || undefined) as DdlExecutionResponse
    } else {
      previewError.value = 'DDL 执行接口不可用。请确保 dbExecuteStructureSync 已注册。'
      return
    }

    if (result?.success) {
      showPreview.value = false
      previewSqls.value = []
      // Reload structure
      await refreshWithOriginals()
    } else {
      previewError.value = result?.error || '执行失败'
    }
  } catch (e: unknown) {
    previewError.value = getErrorMessage(e) || '执行出错'
  } finally {
    executing.value = false
  }
}

// ============ Init ============
onMounted(() => {
  refreshWithOriginals()
})

  return {
    loading, error, columns, indexes,
    activeTab, selectedColumnIndex, selectedIndexes,
    showCreateSqlModal, showPreview, previewSqls, previewError,
    executing, createSql, loadingCreateSql, dbTypeLabel,
    hasChanges, changeCount, columnChangeCount, indexChangeCount,
    availableColumnNames, groupedIndexes, dragIndex, dragOverIndex, dragOverPosition,
    // Alias names for TableStructure.vue compatibility
    dragRowIndex: dragIndex, dropTargetIndex: dragOverIndex, dropPosition: dragOverPosition,
    canAutoIncrement, columnTypes, isColumnModified, isIndexModified,
    addColumn, deleteSelectedColumn, addIndex, addIndexColumn,
    removeIndexColumn, deleteIndexAt, deleteSelectedIndexes,
    onDragStart, onDragOver, onDragEnd, onDrop,
    // Mouse-based drag stubs (component expects these but composable uses HTML5 drag)
    initDragTable: (_el: HTMLElement) => {},
    onRowMouseDown: (_idx: number, _e: MouseEvent) => {},
    onPrimaryKeyChange, toggleIndexSelection,
    refreshWithOriginals, discardChanges,
    executeSqls, generateDdl, showCreateSql, copyCreateSql,
    showSqlPreview,

  }
}
