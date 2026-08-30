import { getTauriAPI } from '../utils/tauri-api'
import { ref, computed, watch } from 'vue'

export interface DBConnection {
  id: string
  name: string
  type: 'mysql' | 'postgresql' | 'redis' | 'sqlite' | 'elasticsearch'
  host: string
  port: number
  user?: string
  password?: string
  database?: string
  path?: string
  dbIndex?: number  // For Redis
  requiresApproval?: boolean  // 安全审核开关
}

export interface QueryRecord {
  id: string
  sql: string
  timestamp: string
  connectionId: string
  success: boolean
  error?: string
  rowCount?: number
  executionTime?: number
}

export interface TableNode {
  name: string
  selected: boolean
}

export interface ExpandedState {
  connections: Set<string>
  tables: Set<string>
  databases: Set<string>  // expanded database nodes: "connId:dbName"
  dbTables: Set<string>   // expanded Tables folders: "connId:dbName"
  dbViews: Set<string>    // expanded Views folders: "connId:dbName"
  redisDatabases: Set<string> // expanded Redis db nodes: "connId:dbIndex"
  redisFolders: Set<string>   // expanded Redis key folders: "connId:dbIndex:folderPath"
}

export type TabType = 'sql' | 'tableData' | 'tableStructure' | 'redisConsole' | 'redisManager' | 'redisQueue' | 'esManager' | 'structureSync' | 'dataSync' | 'backup'

export interface WorkspaceTab {
  id: string
  type: TabType
  title: string
  connectionId: string
  connectionName: string
  // SQL tab
  sql?: string
  // Table tab
  tableName?: string
  dbName?: string
  // Redis tab
  redisDbIndex?: number
  initialKey?: string
  /** 键名原始字节的 base64：非 UTF-8 键只能用它的值回传给命令 */
  initialKeyB64?: string
}

const connections = ref<DBConnection[]>([])
const activeConnectionId = ref<string | null>(null)
const queryHistory = ref<QueryRecord[]>([])
const selectedTable = ref<string | null>(null)
const showConnectionForm = ref(false)
const editingConnection = ref<DBConnection | null>(null)
const expandedState = ref<ExpandedState>({
  connections: new Set(),
  tables: new Set(),
  databases: new Set(),
  dbTables: new Set(),
  dbViews: new Set(),
  redisDatabases: new Set(),
  redisFolders: new Set()
})
const isLoaded = ref(false)

// Tab state
const tabs = ref<WorkspaceTab[]>([])
const activeTabIndex = ref(-1)

export function useDBManager() {
  const activeConnection = computed(() =>
    connections.value.find(c => c.id === activeConnectionId.value) || null
  )

  const sortedConnections = computed(() =>
    [...connections.value].sort((a, b) => a.name.localeCompare(b.name))
  )

  const activeTab = computed(() =>
    activeTabIndex.value >= 0 && activeTabIndex.value < tabs.value.length
      ? tabs.value[activeTabIndex.value]
      : null
  )

  // Load connections from Tauri settings
  const loadConnections = async () => {
    if (isLoaded.value) {return}
    try {
      // 走专用接口：后端对 db_connections 里每项密码解密后返回（旧 Electron 密文也能解）
      const raw = await getTauriAPI().getDbConnections()
      if (Array.isArray(raw) && raw.length > 0) {
        connections.value = raw
      } else if (Array.isArray(raw) && raw.length === 0) {
        connections.value = []
      }
      const lastId = await getTauriAPI().getSetting('db_active_connection')
      if (lastId) {
        activeConnectionId.value = lastId
      }
    } catch (e) {
      console.error('Failed to load DB connections:', e)
    }
    isLoaded.value = true
  }

  // Save connections to Tauri settings
  const saveConnections = async () => {
    try {
      // 走专用接口：后端对每项明文密码加密后落盘
      await getTauriAPI().setDbConnections(connections.value)
      if (activeConnectionId.value) {
        await getTauriAPI().setSetting('db_active_connection', activeConnectionId.value)
      }
    } catch (e) {
      console.error('Failed to save DB connections:', e)
    }
  }

  // Auto-save on changes — debounce 避免频繁 IPC 写磁盘
  let saveTimer: ReturnType<typeof setTimeout> | null = null
  watch(connections, () => {
    if (saveTimer) {clearTimeout(saveTimer)}
    saveTimer = setTimeout(saveConnections, 300)
  }, { deep: true })

  watch(activeConnectionId, async (val) => {
    try {
      if (val) {
        await getTauriAPI().setSetting('db_active_connection', val)
      }
    } catch (e) {
      console.error('Failed to save active connection:', e)
    }
  })

  const setActiveConnection = (id: string) => {
    activeConnectionId.value = id
    selectedTable.value = null
  }

  const addConnection = (config: Omit<DBConnection, 'id'>) => {
    const conn: DBConnection = {
      ...config,
      id: 'db_' + Date.now().toString(36) + Math.random().toString(36).slice(2, 6)
    }
    connections.value.push(conn)
    if (!activeConnectionId.value) {
      activeConnectionId.value = conn.id
    }
    return conn
  }

  const updateConnection = (id: string, updates: Partial<DBConnection>) => {
    const idx = connections.value.findIndex(c => c.id === id)
    if (idx >= 0) {
      connections.value[idx] = { ...connections.value[idx], ...updates }
    }
  }

  const deleteConnection = (id: string) => {
    connections.value = connections.value.filter(c => c.id !== id)
    // Close tabs for this connection
    tabs.value = tabs.value.filter(t => t.connectionId !== id)
    if (activeTabIndex.value >= tabs.value.length) {
      activeTabIndex.value = tabs.value.length - 1
    }
    if (activeConnectionId.value === id) {
      activeConnectionId.value = connections.value.length > 0 ? connections.value[0].id : null
    }
  }

  const openAddForm = () => {
    const t0 = performance.now()
    editingConnection.value = null
    showConnectionForm.value = true
    console.log(`[DBManager] openAddForm: ${(performance.now() - t0).toFixed(1)}ms`)
  }

  const openEditForm = (conn: DBConnection) => {
    // Normalize old field names (user→username, database→dbName)
    const raw = conn as any
    editingConnection.value = {
      id: conn.id, name: conn.name, type: conn.type,
      host: conn.host, port: conn.port,
      user: raw.user || '',
      password: conn.password || '',
      database: raw.database || '',
      path: (conn as any).path || '',
      dbIndex: (conn as any).dbIndex,
      requiresApproval: (conn as any).requiresApproval,
    }
    showConnectionForm.value = true
  }

  const closeForm = () => {
    showConnectionForm.value = false
    editingConnection.value = null
  }

  const toggleConnection = (id: string) => {
    const s = expandedState.value.connections
    if (s.has(id)) {s.delete(id)}
    else {s.add(id)}
  }

  const toggleTables = (connId: string) => {
    const s = expandedState.value.tables
    if (s.has(connId)) {s.delete(connId)}
    else {s.add(connId)}
  }

  const isConnectionExpanded = (id: string) => expandedState.value.connections.has(id)
  const areTablesExpanded = (id: string) => expandedState.value.tables.has(id)

  // Database-level expansion
  const dbKey = (connId: string, dbName: string) => `${connId}:${dbName}`
  const toggleDatabase = (connId: string, dbName: string) => {
    const key = dbKey(connId, dbName)
    const s = expandedState.value.databases
    if (s.has(key)) {s.delete(key)}
    else {s.add(key)}
  }
  const isDatabaseExpanded = (connId: string, dbName: string) =>
    expandedState.value.databases.has(dbKey(connId, dbName))

  const toggleDbTables = (connId: string, dbName: string) => {
    const key = dbKey(connId, dbName)
    const s = expandedState.value.dbTables
    if (s.has(key)) {s.delete(key)}
    else {s.add(key)}
  }
  const areDbTablesExpanded = (connId: string, dbName: string) =>
    expandedState.value.dbTables.has(dbKey(connId, dbName))

  const toggleDbViews = (connId: string, dbName: string) => {
    const key = dbKey(connId, dbName)
    const s = expandedState.value.dbViews
    if (s.has(key)) {s.delete(key)}
    else {s.add(key)}
  }
  const areDbViewsExpanded = (connId: string, dbName: string) =>
    expandedState.value.dbViews.has(dbKey(connId, dbName))

  // Redis expansion
  const redisDbKey = (connId: string, dbIndex: number) => `${connId}:${dbIndex}`
  const toggleRedisDatabase = (connId: string, dbIndex: number) => {
    const key = redisDbKey(connId, dbIndex)
    const s = expandedState.value.redisDatabases
    if (s.has(key)) {s.delete(key)}
    else {s.add(key)}
  }
  const isRedisDatabaseExpanded = (connId: string, dbIndex: number) =>
    expandedState.value.redisDatabases.has(redisDbKey(connId, dbIndex))

  const redisFolderKey = (connId: string, dbIndex: number, folderPath: string) => `${connId}:${dbIndex}:${folderPath}`
  const toggleRedisFolder = (connId: string, dbIndex: number, folderPath: string) => {
    const key = redisFolderKey(connId, dbIndex, folderPath)
    const s = expandedState.value.redisFolders
    if (s.has(key)) {s.delete(key)}
    else {s.add(key)}
  }
  const isRedisFolderExpanded = (connId: string, dbIndex: number, folderPath: string) =>
    expandedState.value.redisFolders.has(redisFolderKey(connId, dbIndex, folderPath))

  const selectTable = (tableName: string) => {
    selectedTable.value = selectedTable.value === tableName ? null : tableName
  }

  const addQueryRecord = (record: Omit<QueryRecord, 'id' | 'timestamp'>) => {
    queryHistory.value.unshift({
      ...record,
      id: 'q_' + Date.now().toString(36),
      timestamp: new Date().toISOString()
    })
    // Keep only last 50
    if (queryHistory.value.length > 50) {
      queryHistory.value = queryHistory.value.slice(0, 50)
    }
  }

  // ========== Tab Management ==========

  const addTab = (tab: Omit<WorkspaceTab, 'id'>) => {
    // Check if a similar tab already exists (reuse instead of duplicate)
    const existing = tabs.value.find(t =>
      t.type === tab.type &&
      t.connectionId === tab.connectionId &&
      t.tableName === tab.tableName &&
      t.dbName === tab.dbName
    )
    if (existing) {
      const idx = tabs.value.indexOf(existing)
      activeTabIndex.value = idx
      // Update initialKey if provided (for redisManager tabs)
      // 复用空白查询页时把新生成的 SQL 填进去，避免「切了个页但什么都没变」
      if (tab.type === 'sql' && tab.sql && !existing.sql) {
        existing.sql = tab.sql
        if (tab.title) {existing.title = tab.title}
      }
      if (tab.type === 'redisManager' && 'initialKey' in tab && tab.initialKey) {
        existing.initialKey = tab.initialKey
        existing.initialKeyB64 = tab.initialKeyB64
        existing.redisDbIndex = tab.redisDbIndex
        // Update the tab title to reflect the selected key
        existing.title = `${tab.connectionName} - ${tab.initialKey}`
      }
      return existing
    }

    const newTab: WorkspaceTab = {
      ...tab,
      id: 'tab_' + Date.now().toString(36) + Math.random().toString(36).slice(2, 6)
    }
    tabs.value.push(newTab)
    activeTabIndex.value = tabs.value.length - 1
    return newTab
  }

  const closeTab = (tabId: string) => {
    const idx = tabs.value.findIndex(t => t.id === tabId)
    if (idx === -1) {return}

    tabs.value.splice(idx, 1)

    // Adjust active tab index
    if (tabs.value.length === 0) {
      activeTabIndex.value = -1
    } else if (idx <= activeTabIndex.value) {
      activeTabIndex.value = Math.max(0, activeTabIndex.value - 1)
    } else if (activeTabIndex.value >= tabs.value.length) {
      activeTabIndex.value = tabs.value.length - 1
    }
  }

  const setActiveTab = (index: number) => {
    activeTabIndex.value = index
  }

  // Convenience: open a new SQL query tab
  /**
   * 打开查询标签页。传 tableName/dbName 时按「库+表」独立成页 —— 否则右键
   * 「生成 SELECT 查询」会和已存在的空白查询页命中同一条去重规则（type+connectionId），
   * 只是把旧页切到前台、SQL 从不写入，表现为「点了没反应」。
   */
  const openSqlTab = (
    connectionId: string,
    connectionName: string,
    sql?: string,
    opts?: { tableName?: string; dbName?: string }
  ) => {
    const tableName = opts?.tableName
    return addTab({
      type: 'sql',
      title: tableName ? `查询 - ${tableName}` : '查询',
      connectionId,
      connectionName,
      tableName,
      dbName: opts?.dbName,
      sql: sql || ''
    })
  }

  // Convenience: open a table data tab
  const openTableDataTab = (connectionId: string, connectionName: string, tableName: string, dbName?: string) => {
    return addTab({
      type: 'tableData',
      title: `${tableName} - 数据`,
      connectionId,
      connectionName,
      tableName,
      dbName
    })
  }

  // Convenience: open a table structure tab
  const openTableStructureTab = (connectionId: string, connectionName: string, tableName: string, dbName?: string) => {
    return addTab({
      type: 'tableStructure',
      title: `${tableName} - 结构`,
      connectionId,
      connectionName,
      tableName,
      dbName
    })
  }

  // Convenience: open Redis console tab
  const openRedisConsoleTab = (connectionId: string, connectionName: string, dbIndex?: number) => {
    return addTab({
      type: 'redisConsole',
      title: 'Redis 控制台',
      connectionId,
      connectionName,
      redisDbIndex: dbIndex
    })
  }

  // Convenience: open Redis Manager tab (Navicat-like)
  const openRedisManagerTab = (connectionId: string, connectionName: string, initialKey?: string, redisDbIndex?: number, initialKeyB64?: string) => {
    const tab: Omit<WorkspaceTab, 'id'> = {
      type: 'redisManager',
      title: initialKey ? `${connectionName} - ${initialKey}` : `${connectionName} - Redis`,
      connectionId,
      connectionName,
    }
    if (redisDbIndex !== undefined) {
      tab.redisDbIndex = redisDbIndex
    }
    if (initialKey) {
      tab.initialKey = initialKey
    }
    if (initialKeyB64) {
      tab.initialKeyB64 = initialKeyB64
    }
    return addTab(tab)
  }

  // Convenience: open Redis Queue tab
  const openRedisQueueTab = (connectionId: string, connectionName: string, dbIndex?: number) => {
    const tab: Omit<WorkspaceTab, 'id'> = {
      type: 'redisQueue',
      title: `${connectionName} - 消息队列`,
      connectionId,
      connectionName,
    }
    if (dbIndex !== undefined) {
      tab.redisDbIndex = dbIndex
    }
    return addTab(tab)
  }

  // Convenience: open Elasticsearch Manager tab
  const openEsManagerTab = (connectionId: string, connectionName: string) => {
    return addTab({
      type: 'esManager',
      title: `${connectionName} - Elasticsearch`,
      connectionId,
      connectionName,
    })
  }

  // Convenience: open Structure Sync tab
  const openStructureSyncTab = () => {
    return addTab({
      type: 'structureSync',
      title: '🔧 结构同步',
      connectionId: '',
      connectionName: ''
    })
  }

  // Convenience: open Data Sync tab
  const openDataSyncTab = () => {
    return addTab({
      type: 'dataSync',
      title: '📦 数据同步',
      connectionId: '',
      connectionName: ''
    })
  }

  // Convenience: open Backup tab
  const openBackupTab = (connectionId: string, connectionName: string) => {
    return addTab({
      type: 'backup',
      title: '🗂️ 数据库备份',
      connectionId,
      connectionName
    })
  }

  return {
    connections,
    sortedConnections,
    activeConnectionId,
    activeConnection,
    queryHistory,
    selectedTable,
    showConnectionForm,
    editingConnection,
    expandedState,
    // Tab state
    tabs,
    activeTabIndex,
    activeTab,

    loadConnections,
    setActiveConnection,
    addConnection,
    updateConnection,
    deleteConnection,
    openAddForm,
    openEditForm,
    closeForm,
    toggleConnection,
    toggleTables,
    isConnectionExpanded,
    areTablesExpanded,
    // Database-level expansion
    toggleDatabase,
    isDatabaseExpanded,
    toggleDbTables,
    areDbTablesExpanded,
    toggleDbViews,
    areDbViewsExpanded,
    // Redis expansion
    toggleRedisDatabase,
    isRedisDatabaseExpanded,
    toggleRedisFolder,
    isRedisFolderExpanded,
    selectTable,
    addQueryRecord,
    // Tab management
    addTab,
    closeTab,
    setActiveTab,
    openSqlTab,
    openTableDataTab,
    openTableStructureTab,
    openRedisConsoleTab,
    openRedisManagerTab,
    openRedisQueueTab,
    openEsManagerTab,
    openStructureSyncTab,
    openDataSyncTab,
    openBackupTab
  }
}
