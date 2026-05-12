// @ts-nocheck
import * as logger from '../../../services/logger'
import { ref, watch, computed } from 'vue'
import type { DBConnection } from '../../../composables/useDBManager'
import { getTauriAPI } from '../../../utils/tauri-api'

// ============ Emit Events Type ============
type ConnectionTreeEmit = (
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  event: any,
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  ...args: any[]
) => void

export function useConnectionTree(
  props: {
    sortedConnections: DBConnection[]
    activeConnectionId: string | null
    selectedTable: string | null
    isConnectionExpanded: (id: string) => boolean
    areTablesExpanded: (id: string) => boolean
    isDatabaseExpanded: (connId: string, dbName: string) => boolean
    areDbTablesExpanded: (connId: string, dbName: string) => boolean
    areDbViewsExpanded: (connId: string, dbName: string) => boolean
    isRedisDatabaseExpanded: (connId: string, dbIndex: number) => boolean
    isRedisFolderExpanded: (connId: string, dbIndex: number, folderPath: string) => boolean
  },
  emit: ConnectionTreeEmit
) {


// ============ Redis Tree Node Interface ============

interface RedisTreeNode {
  segment: string           // This segment of the path (e.g., "user", "1001", "name")
  children: Map<string, RedisTreeNode>  // Child segments
  isLeaf: boolean           // True if this represents an actual Redis key
  key: string | null        // Full Redis key (only for leaf nodes)
  type: string | null       // Redis key type (only for leaf nodes)
  totalCount: number        // Total leaf keys under this node
}

// ============ Search State ============
const searchQuery = ref('')



// ============ State ============

// Database list per connection
const databases = ref<Record<string, string[]>>({})
const loadingDatabases = ref<Record<string, boolean>>({})

// Tables per database (key: "connId:dbName")
const dbTables = ref<Record<string, string[]>>({})
const loadingTables = ref<Record<string, boolean>>({})

// Views per database (key: "connId:dbName")
const dbViews = ref<Record<string, string[]>>({})
const loadingViews = ref<Record<string, boolean>>({})

// Redis databases per connection
const redisDatabases = ref<Record<string, Array<{ db: number; keys: number }>>>({})
const loadingRedisDatabases = ref<Record<string, boolean>>({})

// Redis key tree by db (key: "connId:dbIndex") -> root node of parsed path tree
const redisKeyTrees = ref<Record<string, RedisTreeNode>>({})
const loadingRedisKeyTrees = ref<Record<string, boolean>>({})

// Table comments (key: "connId:dbName:tableName") -> comment
const tableComments = ref<Record<string, string>>({})
const loadingTableComments = ref<Record<string, boolean>>({})

// ============ Table Comment Helpers ============

function tableCommentKey(connId: string, table: string): string {
  return `${connId}:${table}`
}

function getTableComment(connId: string, table: string): string {
  return tableComments.value[tableCommentKey(connId, table)] || ''
}

function getTableTooltip(connId: string, table: string): string {
  const comment = getTableComment(connId, table)
  return comment ? `${table} — ${comment}` : table
}

async function loadTableComments(connId: string, dbName: string, tables: string[]) {
  if (!tables || tables.length === 0) return
  const conn = props.sortedConnections.find(c => c.id === connId)
  if (!conn || conn.type === 'redis') return

  // Skip if already loaded for this db
  const loadKey = `${connId}:${dbName}`
  if (loadingTableComments.value[loadKey]) return
  loadingTableComments.value[loadKey] = true

  try {
    const safeDb = dbName ? dbName.replace(/'/g, "''") : null
    let sql: string
    if (conn.type === 'mysql') {
      const dbCond = safeDb ? `TABLE_SCHEMA = '${safeDb}'` : 'TABLE_SCHEMA = DATABASE()'
      sql = `SELECT TABLE_NAME, TABLE_COMMENT FROM INFORMATION_SCHEMA.TABLES WHERE ${dbCond} AND TABLE_TYPE = 'BASE TABLE'`
    } else if (conn.type === 'postgresql') {
      // Use dbName as schema name, fall back to 'public'
      const safeSchema = dbName ? dbName.replace(/'/g, "''") : 'public'
      sql = `SELECT t.tablename AS table_name, pg_catalog.obj_description(c.oid, 'pg_class') AS table_comment FROM pg_tables t LEFT JOIN pg_class c ON c.relname = t.tablename AND c.relnamespace = (SELECT oid FROM pg_namespace WHERE nspname = t.schemaname) WHERE t.schemaname = '${safeSchema}'`
    } else {
      // SQLite: no native table comment support
      loadingTableComments.value[loadKey] = false
      return
    }

    const result = await getTauriAPI().dbQuery(connId, sql)
    const rows: Array<Record<string, unknown>> = Array.isArray(result)
      ? result as Array<Record<string, unknown>>
      : []

    for (const row of rows) {
      const name = (row.TABLE_NAME ?? row.table_name) as string | undefined
      const comment = (row.TABLE_COMMENT ?? row.table_comment) as string | undefined
      if (name && comment && comment.trim()) {
        const key = tableCommentKey(connId, name)
        tableComments.value[key] = comment.trim()
      }
    }
  } catch (_e) {
    // Silently fail — comments are optional
  } finally {
    loadingTableComments.value[loadKey] = false
  }
}

// ============ Search Filtering ============

function hasMatchingTables(connId: string, dbName: string): boolean {
  const key = dbKey(connId, dbName)
  const tables = dbTables.value[key] || []
  const views = dbViews.value[key] || []
  return tables.some(t => matchesSearch(t)) || views.some(v => matchesSearch(v))
}

function getFilteredDatabases(connId: string): string[] {
  const dbs = databases.value[connId] || []
  if (!searchQuery.value.trim()) return dbs
  return dbs.filter(db => matchesSearch(db) || hasMatchingTables(connId, db))
}

function getFilteredTables(connId: string, dbName: string): string[] {
  const key = dbKey(connId, dbName)
  const tables = dbTables.value[key] || []
  if (!searchQuery.value.trim()) return tables
  return tables.filter(t => matchesSearch(t))
}

function getFilteredViews(connId: string, dbName: string): string[] {
  const key = dbKey(connId, dbName)
  const views = dbViews.value[key] || []
  if (!searchQuery.value.trim()) return views
  return views.filter(v => matchesSearch(v))
}

function getFilteredRedisDatabases(connId: string): Array<{ db: number; keys: number }> {
  const dbs = redisDatabases.value[connId] || []
  if (!searchQuery.value.trim()) return dbs
  return dbs.filter(redisDb => {
    const rk = redisDbKey(connId, redisDb.db)
    const tree = redisKeyTrees.value[rk]
    if (!tree) return false
    return treeHasMatchingKey(tree)
  })
}

function treeHasMatchingKey(node: RedisTreeNode): boolean {
  if (node.isLeaf && node.key) return matchesSearch(node.key)
  for (const child of node.children.values()) {
    if (treeHasMatchingKey(child)) return true
  }
  return false
}

function filterTreeNode(node: RedisTreeNode): RedisTreeNode | null {
  if (node.isLeaf) {
    return node.key && matchesSearch(node.key) ? node : null
  }
  const filteredChildren = new Map<string, RedisTreeNode>()
  for (const [seg, child] of node.children.entries()) {
    const filtered = filterTreeNode(child)
    if (filtered) filteredChildren.set(seg, filtered)
  }
  if (filteredChildren.size === 0) return null
  return {
    ...node,
    children: filteredChildren,
    totalCount: countLeaves(filteredChildren)
  }
}

function countLeaves(children: Map<string, RedisTreeNode>): number {
  let count = 0
  for (const child of children.values()) {
    count += child.isLeaf ? 1 : countLeaves(child.children)
  }
  return count
}

// ============ Search Helpers ============

function matchesSearch(text: string): boolean {
  if (!searchQuery.value.trim()) return true
  return text.toLowerCase().includes(searchQuery.value.toLowerCase())
}

function onSearchFocus() {
  if (searchQuery.value.trim()) {
    expandAllForSearch()
  }
}

function expandAllForSearch() {
  for (const conn of props.sortedConnections) {
    if (conn.type === 'redis') {
      if (!props.isConnectionExpanded(conn.id)) {
        emit('toggle', conn.id)
      }
      for (const redisDb of redisDatabases.value[conn.id] || []) {
        if (!props.isRedisDatabaseExpanded(conn.id, redisDb.db)) {
          emit('toggle-redis-database', conn.id, redisDb.db)
        }
      }
    } else {
      if (!props.isConnectionExpanded(conn.id)) {
        emit('toggle', conn.id)
      }
      for (const dbName of databases.value[conn.id] || []) {
        if (!props.isDatabaseExpanded(conn.id, dbName)) {
          emit('toggle-database', conn.id, dbName)
        }
        if (hasMatchingTables(conn.id, dbName)) {
          if (!props.areDbTablesExpanded(conn.id, dbName)) {
            emit('toggle-db-tables', conn.id, dbName)
          }
          if (!props.areDbViewsExpanded(conn.id, dbName)) {
            emit('toggle-db-views', conn.id, dbName)
          }
        }
      }
    }
  }
}

// Auto-expand when search query changes
watch(searchQuery, (newVal) => {
  if (newVal.trim()) {
    expandAllForSearch()
  }
})

// ============ Event Handlers ============

// ============ Context Menu ============

interface ContextMenuItem {
  icon: string
  label: string
  action: () => void
  separator?: never
}
interface ContextMenuSeparator {
  separator: true
  icon?: never
  label?: never
  action?: never
}

const contextMenu = ref<{
  visible: boolean
  x: number
  y: number
  items: (ContextMenuItem | ContextMenuSeparator)[]
}>({
  visible: false,
  x: 0,
  y: 0,
  items: []
})

function closeContextMenu() {
  contextMenu.value.visible = false
}

function showContextMenu(x: number, y: number, items: (ContextMenuItem | ContextMenuSeparator)[]) {
  contextMenu.value = { visible: true, x, y, items }
}

// ============ Event Handlers ============

function dbKey(connId: string, dbName: string): string {
  return `${connId}:${dbName}`
}

function onToggleDatabase(connId: string, dbName: string) {
  emit('toggle-database', connId, dbName)
}

function onToggleDbTables(connId: string, dbName: string) {
  emit('toggle-db-tables', connId, dbName)
  // Auto-load table comments when expanding
  const key = dbKey(connId, dbName)
  const tables = dbTables.value[key]
  if (tables && tables.length > 0) {
    loadTableComments(connId, dbName, tables)
  }
}

function onToggleDbViews(connId: string, dbName: string) {
  emit('toggle-db-views', connId, dbName)
}

function onSelectTable(connId: string, table: string, dbName?: string) {
  emit('select-table', connId, table, dbName)
}

// ============ Redis Helpers ============

function redisDbKey(connId: string, dbIndex: number): string {
  return `${connId}:${dbIndex}`
}

function typeIcon(type: string): string {
  const icons: Record<string, string> = {
    string: '📝',
    hash: '🗂️',
    list: '📃',
    set: '🔵',
    zset: '📊'
  }
  return icons[type] || '🔑'
}

function onToggleRedisDatabase(connId: string, dbIndex: number) {
  emit('toggle-redis-database', connId, dbIndex)
}

async function onToggleRedisFolder(connId: string, dbIndex: number, folderPath: string, isExpanded: boolean) {
  if (!isExpanded) return // Only load on expand
  
  const key = redisDbKey(connId, dbIndex)
  const root = redisKeyTrees.value[key]
  if (!root) return
  
  // Find the target node
  const parts = folderPath.split(':')
  let targetNode = root
  for (const part of parts) {
    if (targetNode.children.has(part)) {
      targetNode = targetNode.children.get(part)!
    } else {
      return // Node not found
    }
  }
  
  // Check if already loaded (children exist or isLeaf)
  if (targetNode.children.size > 0) return
  
  // Load children
  const loadingKey = `loading:${key}:${folderPath}`
  loadingRedisKeyTrees.value[loadingKey] = true
  
  try {
    const conn = props.sortedConnections.find(c => c.id === connId)
    if (conn) await ensureConnected(conn)
    
    const prefix = folderPath + ':'
    const result = await getTauriAPI().dbRedisKeysTree(connId, dbIndex, prefix)
    if (result && result.success) {
      mergeKeysIntoTree(targetNode, result.folders || [], result.leaves || [], prefix)
      fixTreeCounts(targetNode)
      // Propagate counts up to root
      fixTreeCounts(root)
    }
  } catch (e) {
    console.error('[ConnectionTree] Failed to load folder children:', e)
  } finally {
    delete loadingRedisKeyTrees.value[loadingKey]
  }
}

function onOpenRedisKey(connId: string, dbIndex: number, key: string) {
  emit('open-redis-key', connId, dbIndex, key)
}

// ============ Build Redis Key Tree ============

function mergeKeysIntoTree(
  tree: RedisTreeNode,
  folders: Array<{ name: string; count: number }>,
  leaves: Array<{ name: string; type: string }>,
  prefix: string = ''
): void {
  // Merge new folders
  for (const folder of folders) {
    if (!tree.children.has(folder.name)) {
      tree.children.set(folder.name, {
        segment: folder.name,
        children: new Map(),
        isLeaf: false,
        key: null,
        type: null,
        totalCount: folder.count
      })
    } else {
      // Update count if it already exists
      const existing = tree.children.get(folder.name)!
      if (existing.totalCount < folder.count) {
        existing.totalCount = folder.count
      }
    }
  }

  // Merge new leaves
  for (const leaf of leaves) {
    if (!tree.children.has(leaf.name)) {
      tree.children.set(leaf.name, {
        segment: leaf.name,
        children: new Map(),
        isLeaf: true,
        key: prefix + leaf.name,
        type: leaf.type,
        totalCount: 1
      })
    }
  }

  // Re-sort children to maintain alphabetical order
  const sortedEntries = Array.from(tree.children.entries()).sort((a, b) => {
    // Folders first, then leaves; alphabetical within groups
    const aIsFolder = !a[1].isLeaf
    const bIsFolder = !b[1].isLeaf
    if (aIsFolder !== bIsFolder) return aIsFolder ? -1 : 1
    return a[0].localeCompare(b[0])
  })
  tree.children = new Map(sortedEntries)
}

function getRedisRootNodes(connId: string, dbIndex: number): RedisTreeNode[] {
  const rk = redisDbKey(connId, dbIndex)
  const tree = redisKeyTrees.value[rk]
  if (!tree) return []

  const node = searchQuery.value.trim() ? filterTreeNode(tree) : tree
  if (!node) return []

  // If search narrowed down to a single leaf, show it
  if (node.isLeaf) return [node]

  return Array.from(node.children.values())
}

// ============ Redis Context Menus ============

function onRedisDatabaseContext(event: MouseEvent, conn: DBConnection, dbIndex: number) {
  const items: (ContextMenuItem | ContextMenuSeparator)[] = [
    {
      icon: '💻',
      label: '打开控制台',
      action: () => { emit('open-sql', conn.id); closeContextMenu() }
    },
    {
      icon: '📦',
      label: '消息队列',
      action: () => { emit('open-redis-queue', conn.id, dbIndex); closeContextMenu() }
    },
    { separator: true },
    {
      icon: '🔄',
      label: '刷新',
      action: () => { emit('refresh-tables', conn.id); closeContextMenu() }
    }
  ]
  showContextMenu(event.clientX, event.clientY, items)
}

function onRedisFolderContext(event: MouseEvent, conn: DBConnection, dbIndex: number, folderPath: string) {
  const items: (ContextMenuItem | ContextMenuSeparator)[] = [
    {
      icon: '➕',
      label: '新建键',
      action: () => { emit('open-redis-key', conn.id, dbIndex, ''); closeContextMenu() }
    },
    { separator: true },
    {
      icon: '📋',
      label: '复制路径',
      action: () => {
        navigator.clipboard?.writeText(folderPath).catch(() => {})
        closeContextMenu()
      }
    },
    { separator: true },
    {
      icon: '🔄',
      label: '刷新',
      action: () => { emit('refresh-tables', conn.id); closeContextMenu() }
    }
  ]
  showContextMenu(event.clientX, event.clientY, items)
}

function onRedisKeyContext(event: MouseEvent, conn: DBConnection, dbIndex: number, key: string, _type: string) {
  const items: (ContextMenuItem | ContextMenuSeparator)[] = [
    {
      icon: '👁️',
      label: '查看值',
      action: () => { emit('open-redis-key', conn.id, dbIndex, key); closeContextMenu() }
    },
    { separator: true },
    {
      icon: '📋',
      label: '复制键名',
      action: () => {
        navigator.clipboard?.writeText(key).catch(() => {})
        closeContextMenu()
      }
    },
    {
      icon: '🗑️',
      label: '删除',
      action: async () => {
        try {
          const res = await getTauriAPI().dbRedisDeleteKey(conn.id, conn.dbIndex || 0, key)
          if (res?.success) {
            // Invalidate cached tree for this db to force reload
            const rk = redisDbKey(conn.id, dbIndex)
            delete redisKeyTrees.value[rk]
          }
        } catch (e) {
          console.error('Failed to delete Redis key:', e)
        }
        closeContextMenu()
      }
    }
  ]
  showContextMenu(event.clientX, event.clientY, items)
}

// ============ Context Menus ============

function onConnContext(event: MouseEvent, conn: DBConnection) {
  const items: (ContextMenuItem | ContextMenuSeparator)[] = [
    {
      icon: '📝',
      label: '新建查询',
      action: () => { emit('open-sql', conn.id); closeContextMenu() }
    },
    { separator: true },
    {
      icon: '🔄',
      label: '刷新',
      action: () => { emit('refresh-tables', conn.id); closeContextMenu() }
    },
    { separator: true },
    {
      icon: '✏️',
      label: '编辑连接',
      action: () => { emit('edit', conn); closeContextMenu() }
    },
    {
      icon: '🗑️',
      label: '删除连接',
      action: () => { emit('delete', conn.id); closeContextMenu() }
    }
  ]
  showContextMenu(event.clientX, event.clientY, items)
}

function onDatabaseContext(event: MouseEvent, conn: DBConnection, dbName: string) {
  const items: (ContextMenuItem | ContextMenuSeparator)[] = [
    {
      icon: '📝',
      label: '新建查询',
      action: () => { emit('open-sql', conn.id); closeContextMenu() }
    },
    { separator: true },
    {
      icon: '📋',
      label: '复制数据库名称',
      action: () => {
        navigator.clipboard?.writeText(dbName).catch(() => {})
        closeContextMenu()
      }
    },
    { separator: true },
    {
      icon: '🔄',
      label: '刷新',
      action: () => { emit('refresh-tables', conn.id); closeContextMenu() }
    }
  ]
  showContextMenu(event.clientX, event.clientY, items)
}

function onFolderContext(event: MouseEvent, conn: DBConnection) {
  const items: (ContextMenuItem | ContextMenuSeparator)[] = [
    {
      icon: '📝',
      label: '新建查询',
      action: () => { emit('open-sql', conn.id); closeContextMenu() }
    },
    { separator: true },
    {
      icon: '🔄',
      label: '刷新表列表',
      action: () => { emit('refresh-tables', conn.id); closeContextMenu() }
    }
  ]
  showContextMenu(event.clientX, event.clientY, items)
}

function onTableContext(event: MouseEvent, conn: DBConnection, table: string, dbName?: string) {
  const items: (ContextMenuItem | ContextMenuSeparator)[] = [
    {
      icon: '📊',
      label: '查看数据',
      action: () => { emit('open-table-data', conn.id, table, dbName); closeContextMenu() }
    },
    {
      icon: '🏗️',
      label: '查看结构',
      action: () => { emit('open-table-structure', conn.id, table, dbName); closeContextMenu() }
    },
    { separator: true },
    {
      icon: '📋',
      label: '复制表名',
      action: () => {
        navigator.clipboard?.writeText(table).catch(() => {})
        closeContextMenu()
      }
    },
    { separator: true },
    {
      icon: '📝',
      label: '生成 SELECT 查询',
      action: () => { emit('open-sql', conn.id, table, dbName); closeContextMenu() }
    }
  ]
  showContextMenu(event.clientX, event.clientY, items)
}

function onViewContext(event: MouseEvent, conn: DBConnection, view: string, dbName?: string) {
  const items: (ContextMenuItem | ContextMenuSeparator)[] = [
    {
      icon: '📊',
      label: '查看数据',
      action: () => { emit('open-table-data', conn.id, view, dbName); closeContextMenu() }
    },
    {
      icon: '🏗️',
      label: '查看结构',
      action: () => { emit('open-table-structure', conn.id, view, dbName); closeContextMenu() }
    },
    { separator: true },
    {
      icon: '📋',
      label: '复制视图名称',
      action: () => {
        navigator.clipboard?.writeText(view).catch(() => {})
        closeContextMenu()
      }
    }
  ]
  showContextMenu(event.clientX, event.clientY, items)
}

// ============ Lazy Loading ============

// Ensure connection is connected
async function ensureConnected(conn: DBConnection) {
  try {
    await getTauriAPI().dbConnect(JSON.parse(JSON.stringify(conn)))
  } catch (e) {
    console.warn('[ConnectionTree] Failed to connect:', e)
  }
}

// Load databases when connection expands
watch(
  () => props.sortedConnections.map(c => ({ id: c.id, expanded: props.isConnectionExpanded(c.id) })),
  async (items) => {
    for (const item of items) {
      if (item.expanded && !databases.value[item.id] && !loadingDatabases.value[item.id]) {
        const conn = props.sortedConnections.find(c => c.id === item.id)
        if (!conn) continue

        // Redis doesn't have databases
        if (conn.type === 'redis') {
          databases.value[item.id] = []
          continue
        }

        loadingDatabases.value[item.id] = true
        try {
          await ensureConnected(conn)

          const result = await getTauriAPI().dbGetDatabases(item.id)
          if (result && result.success && result.databases) {
            databases.value[item.id] = result.databases
          } else {
            databases.value[item.id] = []
          }
        } catch (e) {
          console.error('[ConnectionTree] Failed to load databases:', e)
          databases.value[item.id] = []
        } finally {
          loadingDatabases.value[item.id] = false
        }
      }
    }
  },
  { immediate: true }
)

// Load tables when database Tables folder expands
watch(
  () => props.sortedConnections.flatMap(conn => {
    if (conn.type === 'redis') return []
    const dbList = databases.value[conn.id] || []
    return dbList.map(dbName => ({
      connId: conn.id,
      dbName,
      conn,
      expanded: props.areDbTablesExpanded(conn.id, dbName)
    }))
  }),
  async (items) => {
    for (const item of items) {
      const key = dbKey(item.connId, item.dbName)
      if (item.expanded && dbTables.value[key] === undefined && !loadingTables.value[key]) {
        loadingTables.value[key] = true
        try {
          await ensureConnected(item.conn)
          const result = await getTauriAPI().dbGetTables(item.connId, item.dbName)
          if (result && typeof result === 'object' && 'tables' in result) {
            dbTables.value[key] = (result.success ? (result.tables || []) : []) as string[]
          } else if (Array.isArray(result)) {
            dbTables.value[key] = result
          } else {
            dbTables.value[key] = []
          }
        } catch (e) {
          console.error('[ConnectionTree] Failed to load tables:', e)
          dbTables.value[key] = []
        } finally {
          loadingTables.value[key] = false
        }
      }
    }
  },
  {}
)

// Load views when database Views folder expands
watch(
  () => props.sortedConnections.flatMap(conn => {
    if (conn.type === 'redis') return []
    const dbList = databases.value[conn.id] || []
    return dbList.map(dbName => ({
      connId: conn.id,
      dbName,
      conn,
      expanded: props.areDbViewsExpanded(conn.id, dbName)
    }))
  }),
  async (items) => {
    for (const item of items) {
      const key = dbKey(item.connId, item.dbName)
      if (item.expanded && dbViews.value[key] === undefined && !loadingViews.value[key]) {
        loadingViews.value[key] = true
        try {
          await ensureConnected(item.conn)
          const result = await getTauriAPI().dbGetViews(item.connId, item.dbName)
          if (result && result.success && result.views) {
            dbViews.value[key] = result.views
          } else {
            dbViews.value[key] = []
          }
        } catch (e) {
          console.error('[ConnectionTree] Failed to load views:', e)
          dbViews.value[key] = []
        } finally {
          loadingViews.value[key] = false
        }
      }
    }
  },
  {}
)

// Load Redis databases when connection expands
watch(
  () => props.sortedConnections.map(c => ({ id: c.id, expanded: props.isConnectionExpanded(c.id) })),
  async (items) => {
    for (const item of items) {
      const conn = props.sortedConnections.find(c => c.id === item.id)
      if (!conn || conn.type !== 'redis') continue

      if (item.expanded && !redisDatabases.value[item.id] && !loadingRedisDatabases.value[item.id]) {
        loadingRedisDatabases.value[item.id] = true
        try {
          await ensureConnected(conn)
          const result = await getTauriAPI().dbRedisDatabases(item.id)
          if (result && result.success && result.databases) {
            redisDatabases.value[item.id] = result.databases
          } else {
            redisDatabases.value[item.id] = []
          }
        } catch (e) {
          console.error('[ConnectionTree] Failed to load Redis databases:', e)
          redisDatabases.value[item.id] = []
        } finally {
          loadingRedisDatabases.value[item.id] = false
        }
      }
    }
  },
  { immediate: true }
)

// Track hasMore state for each Redis DB
const redisDbHasMore = ref<Record<string, boolean>>({})

// Load Redis keys when a Redis DB is expanded
// Use a computed to track both database list AND expansion state changes
const redisDbExpansionState = computed(() => {
  const result = props.sortedConnections
    .filter(c => c.type === 'redis')
    .flatMap(conn => {
      const dbList = redisDatabases.value[conn.id] || []
      return dbList.map(redisDb => ({
        connId: conn.id,
        dbIndex: redisDb.db,
        conn,
        expanded: props.isRedisDatabaseExpanded(conn.id, redisDb.db)
      }))
    })
  logger.info('[ConnectionTree] redisDbExpansionState computed:', result.map(r => `${r.connId}:db${r.dbIndex}=${r.expanded}`).join(', '))
  return result
})

watch(redisDbExpansionState, async (items) => {
  logger.info(`[ConnectionTree] redisDbExpansionState watch fired, items: ${items.length}`)
    for (const item of items) {
      const key = redisDbKey(item.connId, item.dbIndex)
      const treeExists = redisKeyTrees.value[key] !== undefined
      const isLoading = loadingRedisKeyTrees.value[key]
      logger.info(`[ConnectionTree] Checking db${item.dbIndex}: expanded=${item.expanded}, treeExists=${treeExists}, isLoading=${isLoading}`)
      if (item.expanded && redisKeyTrees.value[key] === undefined && !loadingRedisKeyTrees.value[key]) {
        logger.info(`[ConnectionTree] → Loading keys for ${item.connId}:db${item.dbIndex}...`)
        loadingRedisKeyTrees.value[key] = true
        try {
          await ensureConnected(item.conn)
          logger.info(`[ConnectionTree] → Calling dbRedisKeysTree(${item.connId}, ${item.dbIndex}, '', false)`)
          // Initial load: loadMore = false
          const result = await getTauriAPI().dbRedisKeysTree(item.connId, item.dbIndex, '')
          logger.info(`[ConnectionTree] → dbRedisKeysTree result:`, JSON.stringify(result).slice(0, 300))
          if (result && result.success) {
            const root: RedisTreeNode = {
              segment: '__root__',
              children: new Map(),
              isLeaf: false,
              key: null,
              type: null,
              totalCount: 0
            }
            // Merge initial batch
            logger.info(`[ConnectionTree] → Folders: ${(result.folders || []).length}, Leaves: ${(result.leaves || []).length}, hasMore: ${result.hasMore}`)
            mergeKeysIntoTree(root, result.folders || [], result.leaves || [], '')
            // Fix totalCounts
            fixTreeCounts(root)
            logger.info(`[ConnectionTree] → Root totalCount: ${root.totalCount}, children: ${root.children.size}`)
            
            redisKeyTrees.value[key] = root
            redisDbHasMore.value[key] = result.hasMore || false
          } else {
            // Fallback for error/empty
            console.warn('[ConnectionTree] → Result not success:', result?.error || 'unknown')
            redisKeyTrees.value[key] = {
              segment: '__root__',
              children: new Map(),
              isLeaf: false,
              key: null,
              type: null,
              totalCount: 0
            }
            redisDbHasMore.value[key] = false
          }
        } catch (e) {
          console.error('[ConnectionTree] → FAILED to load Redis keys:', e)
          redisKeyTrees.value[key] = {
            segment: '__root__',
            children: new Map(),
            isLeaf: false,
            key: null,
            type: null,
            totalCount: 0
          }
          redisDbHasMore.value[key] = false
        } finally {
          loadingRedisKeyTrees.value[key] = false
          logger.info(`[ConnectionTree] → Loading finished for db${item.dbIndex}`)
        }
      }
    }
  },
  { immediate: true }
)

async function loadMoreRedisKeys(connId: string, dbIndex: number) {
  const key = redisDbKey(connId, dbIndex)
  if (loadingRedisKeyTrees.value[key] || !redisDbHasMore.value[key]) return
  
  loadingRedisKeyTrees.value[key] = true
  try {
    const conn = props.sortedConnections.find(c => c.id === connId)
    if (conn) await ensureConnected(conn)
    
    // Incremental load: loadMore = true
    const result = await getTauriAPI().dbRedisKeysTree(connId, dbIndex, '')
    if (result && result.success) {
      const root = redisKeyTrees.value[key]
      if (root) {
        mergeKeysIntoTree(root, result.folders || [], result.leaves || [], '')
        fixTreeCounts(root)
      }
      redisDbHasMore.value[key] = result.hasMore || false
    }
  } catch (e) {
    console.error('[ConnectionTree] Failed to load more Redis keys:', e)
  } finally {
    loadingRedisKeyTrees.value[key] = false
  }
}

function fixTreeCounts(n: RedisTreeNode): number {
  let total = n.isLeaf ? 1 : 0
  for (const child of n.children.values()) {
    total += fixTreeCounts(child)
  }
  n.totalCount = total
  return total
}

// ============ Icons ============

function dbTypeIcon(type: string): string {
  const icons: Record<string, string> = {
    mysql: '🐬',
    postgresql: '🐘',
    redis: '🔴',
    sqlite: '📄'
  }
  return icons[type] || '🗄️'
}

// ============ Refresh ============

async function refreshTables(connId: string) {
    // Clear cached data for this connection
    delete databases.value[connId]
    loadingDatabases.value[connId] = true

    try {
      const conn = props.sortedConnections.find(c => c.id === connId)
      if (!conn) return

      if (conn.type === 'redis') {
        // Clear all Redis caches for this connection
        delete redisDatabases.value[connId]
        delete loadingRedisDatabases.value[connId]
        const dbList = Object.keys(redisKeyTrees.value).filter(k => k.startsWith(`${connId}:`))
        for (const key of dbList) delete redisKeyTrees.value[key]
        const loadingKeys = Object.keys(loadingRedisKeyTrees.value).filter(k => k.startsWith(`${connId}:`))
        for (const key of loadingKeys) delete loadingRedisKeyTrees.value[key]

        // Re-load Redis databases
        try {
          await ensureConnected(conn)
          const result = await getTauriAPI().dbRedisDatabases(connId)
          if (result && result.success && result.databases) {
            redisDatabases.value[connId] = result.databases
          } else {
            redisDatabases.value[connId] = []
          }
        } catch (e) {
          console.error('[ConnectionTree] Failed to refresh Redis databases:', e)
          redisDatabases.value[connId] = []
        }
        return
      }

      await ensureConnected(conn)

      const result = await getTauriAPI().dbGetDatabases(connId)
      if (result && result.success && result.databases) {
        databases.value[connId] = result.databases

        // Also refresh all loaded tables/views for each database
        for (const dbName of result.databases) {
          const key = dbKey(connId, dbName)
          delete dbTables.value[key]
          delete dbViews.value[key]
        }
      } else {
        databases.value[connId] = []
      }
    } catch (e) {
      console.error('[ConnectionTree] Failed to refresh:', e)
      databases.value[connId] = []
    } finally {
      loadingDatabases.value[connId] = false
    }
  }

  return {
    searchQuery, contextMenu,
    databases, dbTables, dbViews, loadingDatabases, loadingTables, loadingViews, loadingTableComments,
    redisDatabases, redisDbExpansionState, redisDbHasMore, loadingRedisDatabases,
    redisKeyTrees, loadingRedisKeyTrees, tableComments,
    dbTypeIcon, typeIcon, redisDbKey, tableCommentKey,
    closeContextMenu, showContextMenu,
    onSearchFocus, onSelectTable,
    onConnContext, onDatabaseContext, onTableContext, onViewContext,
    onToggleDatabase, onToggleDbTables, onToggleDbViews,
    onRedisDatabaseContext, onRedisFolderContext, onRedisKeyContext,
    onToggleRedisDatabase, onToggleRedisFolder, onOpenRedisKey, onFolderContext,
    ensureConnected, getTableTooltip, getTableComment,
    getFilteredDatabases, getFilteredTables, getFilteredViews,
    getFilteredRedisDatabases, getRedisRootNodes,
    loadTableComments, loadMoreRedisKeys,
    filterTreeNode, matchesSearch, treeHasMatchingKey,
    countLeaves, fixTreeCounts, mergeKeysIntoTree,
    expandAllForSearch, hasMatchingTables,
    dbKey, refreshTables,
  }
}