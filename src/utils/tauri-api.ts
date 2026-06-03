// @ts-nocheck
/**
 * Tauri API — 统一 IPC/原生接口层
 * 覆盖 81+ 个 Tauri commands
 */

import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import type {
  Project, Server, ServerGroup, DbConnectionConfig, ApiResponse, ProjectStats,
  Todo, Tag, Subtask, Note, NoteGroup, WeeklyReport, MfaSecret,
  AccountingRecord, AccountingCategory, Budget, LogPreset, NotificationSettings,
  WireGuardConfig, WireGuardStatus,
  ToolsetInfo, MCPServerInfo,
  HermesConfigInfo, ConfigExportResult, ConfigImportResult,
  MemoryInfo, MemoryWriteResult, MemoryProviderResult,
  SkillInfo, SkillCliResult,
  CronJob,
  ProviderInfo, ProviderListResult, ProviderSaveResult,
  OAuthFlowResult, OAuthPollResult,
} from '../types'

// ============ 日志脱敏 ============

/** 敏感字段名（大小写不敏感匹配，包含子串即命中） */
const SENSITIVE_KEYS = [
  'password', 'passwd', 'pwd', 'secret', 'token',
  'access_token', 'refresh_token', 'api_key', 'apikey', 'api_secret',
  'auth', 'authorization', 'credential',
  'private_key', 'ssh_key', 'privatekey',
  'sshkeypath', 'ssh_key_path',
  'dbpassword', 'db_password', 'sshpassword', 'ssh_password',
]

/** 递归脱敏，原地修改并返回 */
function sanitizeLogValue(val: unknown): unknown {
  if (val === null || val === undefined) {return val}
  if (Array.isArray(val)) {
    return val.map(v => sanitizeLogValue(v))
  }
  if (typeof val === 'object') {
    const obj = val as Record<string, unknown>
    for (const key of Object.keys(obj)) {
      const lower = key.toLowerCase()
      if (SENSITIVE_KEYS.some(sk => lower.includes(sk))) {
        const v = obj[key]
        obj[key] = typeof v === 'string' && v.length > 0 ? '**' : v
      } else {
        obj[key] = sanitizeLogValue(obj[key])
      }
    }
    return obj
  }
  return val
}

/** 将任意 JSON 值序列化为安全日志字符串（不超过 maxLen 字符） */
function safeJsonLog(val: unknown, maxLen = 300): string {
  try {
    const clone = JSON.parse(JSON.stringify(val))
    sanitizeLogValue(clone)
    const s = JSON.stringify(clone)
    return s.length <= maxLen ? s : s.slice(0, maxLen) + '...'
  } catch {
    return '[日志序列化失败]'
  }
}

// ============ 核心调用 ============

/** 检测后端返回是否已是标准响应格式 { success, data/error }（避免双层嵌套） */
function isStandardResponse(obj: unknown): boolean {
  if (obj === null || typeof obj !== 'object') {return false}
  const o = obj as Record<string, unknown>
  return typeof o['success'] === 'boolean' && ('data' in o || 'error' in o)
}

async function tauriInvoke<T>(command: string, args: Record<string, unknown> = {}, silent = !import.meta.env.DEV): Promise<ApiResponse<T>> {
  if (!silent) {console.log(`[Tauri IPC] → ${command}  ${safeJsonLog(args, 200)}`)}
  const t0 = performance.now()
  try {
    const raw = await invoke(command, args)
    const elapsed = (performance.now() - t0).toFixed(0)
    if (!silent) {
      console.log(`[Tauri IPC] ← ${command} ✅ ${elapsed}ms  ${safeJsonLog(raw)}`)
    }
    // 后端已返回标准格式 { success, data/error } → 直接透传，避免再包一层造成双层嵌套
    if (isStandardResponse(raw)) {
      return raw as ApiResponse<T>
    }
    // 非标准格式（裸值/扁平对象）→ 包装为统一响应
    return { success: true, data: raw as unknown as T }
  } catch (err: unknown) {
    const elapsed = (performance.now() - t0).toFixed(0)
    const message = err instanceof Error ? err.message : String(err)
    // Always log errors even in silent mode
    console.error(`[Tauri IPC] ← ${command} ❌ ${elapsed}ms  ${message}`)
    return { success: false, error: message }
  }
}

/** tauriCall: like tauriInvoke but auto-unwraps .data */
async function tauriCall<T>(command: string, args: Record<string, unknown> = {}, silent = !import.meta.env.DEV): Promise<T> {
  const res = await tauriInvoke<T>(command, args, silent)
  if (!res.success) {throw new Error(res.error || `IPC call failed: ${command}`)}
  return res.data as T
}

export { tauriInvoke, tauriCall }

// ============ 项目 ============

export function useProjectsAPI() {
  return {
    getServers: async (): Promise<Server[]> => {
      const res = await tauriInvoke<Server[]>('get_all_servers')
      return res.success ? (res.data ?? []) : []
    },
    getProjects: async (onlyActive = true): Promise<Project[]> => {
      const res = await tauriInvoke<Project[]>('get_projects', { onlyActive })
      return res.success ? (res.data ?? []) : []
    },
    addProject: async (project: Partial<Project>): Promise<Project> => {
      const fullProject = {
        id: project.id ?? crypto.randomUUID(), name: project.name ?? '',
        description: project.description ?? '', color: project.color ?? '#6366f1',
        repoPath: project.repoPath ?? null, branch: project.branch ?? null,
        repoPath2: project.repoPath2 ?? null, branch2: project.branch2 ?? null,
        gitUrl1: project.gitUrl1 ?? null, gitUrl2: project.gitUrl2 ?? null,
        category: project.category ?? null,
        createdAt: project.createdAt ?? new Date().toISOString(),
        updatedAt: project.updatedAt ?? new Date().toISOString(),
        archived: project.archived ?? false,
      }
      const res = await tauriInvoke<Project>('add_project', { project: fullProject })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    updateProject: async (project: Project): Promise<Project> => {
      const res = await tauriInvoke<Project>('update_project', { project })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    deleteProject: async (id: string): Promise<void> => {
      const res = await tauriInvoke<string>('delete_project', { id })
      if (!res.success) {throw new Error(res.error)}
    },
    getProjectStats: async (projectId: string): Promise<ProjectStats> => {
      const res = await tauriInvoke<ProjectStats>('get_project_stats', { projectId })
      return res.success ? (res.data ?? { total: 0, completed: 0, progress: 0 }) : { total: 0, completed: 0, progress: 0 }
    },
    getProjectTodos: async (projectId: string): Promise<any[]> => {
      const res = await tauriInvoke<any[]>('get_project_todos', { projectId })
      return res.success ? (res.data ?? []) : []
    },
  }
}

// ============ 服务器 ============

export function useServersAPI() {
  return {
    getServers: async (): Promise<Server[]> => {
      const res = await tauriInvoke<Server[]>('get_all_servers')
      return res.success ? (res.data ?? []) : []
    },
    getAllServers: async (): Promise<Server[]> => {
      const res = await tauriInvoke<Server[]>('get_all_servers')
      return res.success ? (res.data ?? []) : []
    },
    getServerById: async (serverId: string): Promise<Server | null> => {
      const res = await tauriInvoke<Server | null>('get_server_by_id', { serverId })
      return res.success ? (res.data ?? null) : null
    },
    addServer: async (server: Partial<Server>): Promise<Server> => {
      const fullServer = {
        id: server.id ?? crypto.randomUUID(), name: server.name ?? '',
        host: server.host ?? '', port: server.port ?? 22,
        username: server.username ?? '', sshKeyPath: server.sshKeyPath ?? null,
        password: server.password ?? null, description: server.description ?? '',
        tags: server.tags ?? [], groupId: server.groupId ?? null,
        requiresApproval: server.requiresApproval ?? false,
        createdAt: server.createdAt ?? new Date().toISOString(),
        updatedAt: server.updatedAt ?? new Date().toISOString(),
      }
      const res = await tauriInvoke<Server>('add_server', { server: fullServer })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    updateServer: async (server: Server): Promise<Server> => {
      const res = await tauriInvoke<Server>('update_server', { server })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    deleteServer: async (serverId: string): Promise<void> => {
      const res = await tauriInvoke<string>('delete_server', { serverId })
      if (!res.success) {throw new Error(res.error)}
    },
    getAllServerGroups: async (): Promise<ServerGroup[]> => {
      const res = await tauriInvoke<ServerGroup[]>('get_all_server_groups')
      return res.success ? (res.data ?? []) : []
    },
    addServerGroup: async (group: Partial<ServerGroup>): Promise<ServerGroup> => {
      const fullGroup = {
        id: group.id ?? crypto.randomUUID(), name: group.name ?? '',
        description: group.description ?? '', parentId: group.parentId ?? null,
        color: group.color ?? '#6c63ff',
        createdAt: group.createdAt ?? new Date().toISOString(),
        updatedAt: group.updatedAt ?? new Date().toISOString(),
      }
      const res = await tauriInvoke<ServerGroup>('add_server_group', { group: fullGroup })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    updateServerGroup: async (groupId: string, updates: { name?: string; description?: string; parentId?: string | null; color?: string }): Promise<ServerGroup> => {
      const res = await tauriInvoke<ServerGroup>('update_server_group', {
        groupId, name: updates.name ?? '', description: updates.description ?? '',
        parentId: updates.parentId ?? null, color: updates.color ?? '#6c63ff',
      })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    deleteServerGroup: async (groupId: string): Promise<void> => {
      const res = await tauriInvoke<string>('delete_server_group', { groupId })
      if (!res.success) {throw new Error(res.error)}
    },
    testConnection: async (server: Partial<Server>): Promise<{ success: boolean; error?: string }> => {
      const res = await tauriInvoke<boolean>('test_connection', {
        host: server.host ?? '', port: server.port ?? 22,
        username: server.username ?? '', password: server.password ?? null,
        sshKeyPath: server.sshKeyPath ?? null,
      })
      return { success: res.success && res.data === true, error: res.error }
    },
  }
}

// ============ 数据库 ============

export function useDatabaseAPI() {
  return {
    dbConnect: async (config: DbConnectionConfig): Promise<{ success: boolean; error?: string }> => {
      // 清理重复字段
      const { database, user, ...normalized } = config as any
      normalized.type = config.type === 'postgresql' ? 'postgres' : config.type
      if (!normalized.username && user) {normalized.username = user}
      // SQLite 没有 username/host/password，确保必填字段有默认值避免 Rust 反序列化失败
      if (config.type === 'sqlite') {
        normalized.username = normalized.username ?? ''
      }
      if (!normalized.dbName && database) {normalized.dbName = database}
      const res = await tauriInvoke<boolean>('db_connect', { config: normalized })
      return { success: res.success, error: res.error }
    },
    dbDisconnect: async (id: string): Promise<{ success: boolean; error?: string }> => {
      const res = await tauriInvoke<boolean>('db_disconnect', { id })
      return { success: res.success, error: res.error }
    },
    dbQuery: async (id: string, sql: string): Promise<{ success: boolean; rows?: any; error?: string }> => {
      const res = await tauriInvoke<any>('db_query', { id, sql })
      // Rust 返回 { success, rows }，tauriInvoke 包装为 { success, data: { success, rows } }
      if (res.success && res.data) {
        return { success: res.data.success ?? true, rows: res.data.rows, error: res.data.error }
      }
      return { success: res.success, error: res.error }
    },
    getTables: async (id: string, dbName: string): Promise<{ success: boolean; tables?: any; error?: string }> => {
      const res = await tauriInvoke<any>('db_get_tables', { id, dbName })
      if (res.success && res.data && res.data.rows) {
        // SHOW TABLES returns objects like { "Tables_in_dbname": "table1" }
        // Extract just the table names.
        const rows = res.data.rows;
        const tables = rows.map((r: any) => Object.values(r)[0] || r);
        return { success: true, tables: tables, error: res.data.error }
      }
      return { success: res.success, error: res.error }
    },
    getDatabases: async (id: string): Promise<{ success: boolean; databases?: any; error?: string }> => {
      const res = await tauriInvoke<any>('db_get_databases', { id })
      if (res.success && res.data && res.data.rows) {
        const rows = res.data.rows;
        // 健壮性提取：兼容 { "Database": "name" } 或 { "datname": "name" } 或直接是字符串
        const names = rows.map((r: any) => {
          if (typeof r === 'object' && r !== null) {
            return Object.values(r)[0] as string;
          }
          return r;
        });
        return { success: true, databases: names, error: null }
      }
      return { success: res.success, error: res.error }
    },
    // 表结构 & 视图
    dbGetTableStructure: async (id: string, table: string, dbName: string): Promise<any> => {
      const res = await tauriInvoke<any>('db_get_table_structure', { id, table, dbName })
      if (res.success) {
        // Return full object with rows + indexes so composable can access both
        return { rows: res.rows ?? res.data?.rows ?? [], indexes: res.indexes ?? res.data?.indexes ?? [] }
      }
      return { rows: [], indexes: [] }
    },
    dbGetTablePrimaryKeys: async (id: string, table: string, dbName: string): Promise<any> => {
      const res = await tauriInvoke<any>('db_get_table_primary_keys', { id, table, dbName })
      if (res.success && res.data?.rows) {
        const pks = res.data.rows.map((r: any) => r.COLUMN_NAME || r.column_name).filter(Boolean)
        return { success: true, primaryKeys: pks }
      }
      return { success: false, primaryKeys: [] }
    },
    dbGetViews: async (id: string, dbName: string): Promise<any> => {
      const res = await tauriInvoke<any>('db_get_views', { id, dbName })
      // SHOW FULL TABLES returns rows like { "Tables_in_db": "view_name", "Table_type": "VIEW" }
      if (res.success && res.data && res.data.rows) {
        const rows = res.data.rows;
        // Extract view names (usually the first value or keyed by Tables_in_...)
        const views = rows.map((r: any) => {
            // Try common keys for table/view names
            const keys = Object.keys(r);
            // Look for a key that looks like a table name (not Table_type)
            const nameKey = keys.find(k => !k.includes('Type') && !k.includes('type')) || keys[0];
            return r[nameKey];
        });
        return views;
      }
      return [];
    },
    dbGetCreateSql: async (id: string, table: string, dbName: string): Promise<string> => {
      const res = await tauriInvoke<any>('db_get_create_sql', { id, table, dbName })
      if (res.success && res.data?.rows && res.data.rows.length > 0) {
        return res.data.rows[0]['Create Table'] ?? res.data.rows[0]?.['Create View'] ?? ''
      }
      return ''
    },
    // 结构对比 & 同步
    dbCompareStructures: async (sourceId: string, sourceDb: string, targetId: string, targetDb: string): Promise<any> => {
      const res = await tauriInvoke<any>('db_compare_structures', { sourceId, sourceDb, targetId, targetDb })
      return res.success && res.data ? res.data : {}
    },
    dbExecuteStructureSync: async (id: string, sqls: string[], dbName: string): Promise<any> => {
      const res = await tauriInvoke<any>('db_execute_structure_sync', { id, sqls, targetDbName: dbName })
      return res.success && res.data ? res.data : null
    },
    // 数据对比 & 同步
    dbCompareData: async (params: Record<string, unknown>): Promise<any> => {
      // Rust expects: sourceId, targetId, tableName, primaryKeys, sourceDb, targetDb
      // Vue passes: sourceId, targetId, table, primaryKeys, columns, sourceDb, targetDb, tablePrimaryKeys
      const { table, ...rest } = params as any
      const res = await tauriInvoke<any>('db_compare_data', { ...rest, tableName: table })
      return res.success && res.data ? res.data : {}
    },
    dbExecuteDataSync: async (params: Record<string, unknown>): Promise<any> => {
      const res = await tauriInvoke<any>('db_execute_data_sync', { params })
      return res.success && res.data ? res.data : { synced: 0 }
    },
    // 数据库备份
    dbBackupCreate: async (id: string, dbName: string, objects: any[]): Promise<any> => {
      const res = await tauriInvoke<any>('db_backup_create', { id, dbName, objects })
      return res.success && res.data ? res.data : null
    },
    dbBackupList: async (id?: string): Promise<any[]> => {
      const res = await tauriInvoke<any>('db_backup_list', { id: id ?? null })
      return res.success && res.data ? (res.data.backups ?? []) : []
    },
    dbBackupRestore: async (id: string, file: string): Promise<any> => {
      const res = await tauriInvoke<any>('db_backup_restore', { id, file })
      return res.success && res.data ? res.data : null
    },
    dbBackupDelete: async (file: string): Promise<boolean> => {
      const res = await tauriInvoke<any>('db_backup_delete', { file })
      return res.success && res.data?.success === true
    },
    // Redis 操作
    dbRedisDatabases: async (id: string): Promise<number[]> => {
      const res = await tauriInvoke<any>('db_redis_databases', { id })
      return res.success && res.data ? (res.data.databases ?? [0]) : [0]
    },
    dbRedisKeys: async (id: string, dbIndex: number, pattern: string): Promise<string[]> => {
      const res = await tauriInvoke<any>('db_redis_keys', { id, dbIndex, pattern })
      return res.success && res.data ? (res.data.keys ?? []) : []
    },
    dbRedisKeysTree: async (id: string, dbIndex: number, pattern: string): Promise<any> => {
      const res = await tauriInvoke<any>('db_redis_keys_tree', { id, dbIndex, pattern })
      if (res.success && res.data) {
        const keys = res.data.keys || []
        // 前端期望格式: { success: true, folders: [{name, count}], leaves: [{name, type}], hasMore }
        // Rust 返回的是全量 keys 数组，这里做兼容转换
        const folders = new Map<string, number>()
        const leaves: Array<{ name: string; type: string }> = []
        for (const k of keys) {
          const idx = k.indexOf(':')
          if (idx > 0) {
            const folder = k.substring(0, idx)
            folders.set(folder, (folders.get(folder) || 0) + 1)
          } else {
            // 无冒号的 key 视为叶子节点（类型未知）
            leaves.push({ name: k, type: 'string' })
          }
        }
        return {
          success: true,
          folders: Array.from(folders.entries()).map(([name, count]) => ({ name, count })),
          leaves,
          hasMore: false
        }
      }
      return { success: false, folders: [], leaves: [], hasMore: false }
    },
    dbRedisKeysByType: async (id: string, dbIndex: number, type: string): Promise<any> => {
      const res = await tauriInvoke<any>('db_redis_keys_by_type', { id, dbIndex, type })
      if (res.success && res.data) {
        // 后端 SCAN 返回的是 [cursor, keys[]]
        // 前端期望 { keysByType: { type: [keys] } }
        // 这里做简单的兼容：把所有 key 归类为 type (如果指定) 或 'string'
        const raw = res.data.result
        let keys: string[] = []
        if (Array.isArray(raw) && raw.length >= 2 && Array.isArray(raw[1])) {
          keys = raw[1]
        } else if (Array.isArray(raw)) {
          keys = raw // Fallback if format is different
        }
        
        // 简单分组（因为 SCAN 不返回类型）
        const keysByType: Record<string, string[]> = {}
        const targetType = type !== '*' ? type : 'string' // 默认归类
        keysByType[targetType] = keys
        
        return { success: true, keysByType }
      }
      return { success: false, keysByType: {} }
    },
    dbRedisKeyInfo: async (id: string, dbIndex: number, key: string): Promise<any> => {
      const res = await tauriInvoke<any>('db_redis_key_info', { id, dbIndex, key })
      return res.success && res.data ? (res.data.info ?? null) : null
    },
    dbRedisKeyValue: async (id: string, dbIndex: number, key: string): Promise<any> => {
      const res = await tauriInvoke<any>('db_redis_key_value', { id, dbIndex, key })
      return res.success && res.data ? res.data : null
    },
    dbRedisSetKey: async (id: string, dbIndex: number, key: string, value: string, ttl?: number): Promise<boolean> => {
      const res = await tauriInvoke<any>('db_redis_set_key', { id, dbIndex, key, value, ttl: ttl ?? 0 })
      return res.success && res.data?.success === true
    },
    dbRedisAddKey: async (id: string, dbIndex: number, keyType: string, key: string, value: any): Promise<boolean> => {
      const res = await tauriInvoke<any>('db_redis_add_key', { id, dbIndex, keyType, key, value })
      return res.success && res.data?.success === true
    },
    dbRedisDeleteKey: async (id: string, dbIndex: number, key: string): Promise<boolean> => {
      const res = await tauriInvoke<any>('db_redis_delete_key', { id, dbIndex, key })
      return res.success && res.data?.success === true
    },
    dbRedisExec: async (id: string, dbIndex: number, command: string): Promise<any> => {
      const res = await tauriInvoke<any>('db_redis_exec', { id, dbIndex, command })
      return res.success && res.data ? (res.data.result ?? null) : null
    },
    // CICD 工具检测
    detectToolPaths: async (): Promise<Record<string, string>> => {
      const res = await tauriInvoke<Record<string, string>>('detect_tool_paths')
      return res.success ? (res.data ?? {}) : {}
    },
    detectBuildTools: async (): Promise<Record<string, any>> => {
      const res = await tauriInvoke<Record<string, any>>('detect_build_tools')
      return res.success ? (res.data ?? {}) : {}
    },
    detectSdkVersions: async (): Promise<Record<string, any>> => {
      const res = await tauriInvoke<Record<string, any>>('detect_sdk_versions')
      return res.success ? (res.data ?? {}) : {}
    },
    getCicdConfigs: async (): Promise<any[]> => {
      const res = await tauriInvoke<any[]>('get_cicd_configs')
      return res.success ? (res.data ?? []) : []
    },
    getCicdGroups: async (): Promise<any[]> => {
      const res = await tauriInvoke<any[]>('get_cicd_groups')
      return res.success ? (res.data ?? []) : []
    },
    // ── CICD Config CRUD ──
    getCicdConfigById: async (id: string): Promise<any> => {
      const res = await tauriInvoke<any>('get_cicd_config_by_id', { id })
      return res.success ? res.data : null
    },
    addCicdConfig: async (config: Record<string, unknown>, modules?: any[]): Promise<any> => {
      const res = await tauriInvoke<any>('save_cicd_config', { config, modules })
      return res.success ? res.data : { success: false, error: res.error }
    },
    updateCicdConfig: async (config: Record<string, unknown>): Promise<any> => {
      const res = await tauriInvoke<any>('save_cicd_config', { config })
      return res.success ? res.data : { success: false, error: res.error }
    },
    deleteCicdConfig: async (id: string): Promise<any> => {
      const res = await tauriInvoke<any>('delete_cicd_config', { id })
      return res.success ? res.data : { success: false, error: res.error }
    },
    // ── CICD Modules ──
    getDeployModules: async (configId: string): Promise<any[]> => {
      const res = await tauriInvoke<any[]>('get_deploy_modules', { configId })
      return res.success ? (res.data ?? []) : []
    },
    addDeployModule: async (module: Record<string, unknown>): Promise<any> => {
      const res = await tauriInvoke<any>('save_deploy_module', { module })
      return res.success ? res.data : { success: false, error: res.error }
    },
    updateDeployModule: async (module: Record<string, unknown>): Promise<any> => {
      const res = await tauriInvoke<any>('update_deploy_module', { module })
      return res.success ? res.data : { success: false, error: res.error }
    },
    deleteDeployModule: async (moduleId: string): Promise<any> => {
      const res = await tauriInvoke<any>('delete_deploy_module', { module_id: moduleId })
      return res.success ? res.data : { success: false, error: res.error }
    },
    // ── CICD Scan ──
    scanProject: async (localPath: string): Promise<any> => {
      const res = await tauriInvoke<any>('scan_project', { localPath })
      return res.success ? res.data : {}
    },
    scanProjectModules: async (projectPath: string): Promise<any> => {
      const res = await tauriInvoke<any>('scan_project_modules', { projectPath })
      return res.success ? res.data : { success: false, modules: [], error: '扫描失败' }
    },
    // ── SSH Test ──
    testSsh: async (config: Record<string, unknown>): Promise<any> => {
      const res = await tauriInvoke<any>('test_connection', config)
      return res.success ? res.data : { success: false, error: res.error }
    },
    checkNetworkPermission: async (host: string, port: number): Promise<{ success: boolean; error?: string }> => {
      const res = await tauriInvoke<boolean>('check_network_permission', { host, port })
      return { success: res.success && res.data === true, error: res.error }
    },
    // ── Schema modification ──
    // TODO: Re-implement with proper db_name support and matching frontend/Rust signatures.
    // Previously: dbAddColumn, dbDropColumn, dbModifyColumn, dbRenameColumn, dbAddIndex, dbDropIndex, dbRenameTable
    // All deleted from both frontend and Rust (L770-854 of database.rs) due to param mismatch and no consumers.
  }
}

// ============ LAN 协作 ============

export function useLanAPI() {
  return {
    lanSetStatus: async (status: string): Promise<void> => {
      await tauriInvoke('lan_set_status', { status })
    },
    lanRefreshDiscovery: async (): Promise<void> => {
      await tauriInvoke('lan_refresh_discovery')
    },
    lanGetUserInfo: async (): Promise<any> => {
      const res = await tauriInvoke<any>('lan_get_user_info')
      return res.success ? (res.data ?? {}) : {}
    },
    lanGetAllUnreadCounts: async (userId: string): Promise<any> => {
      const res = await tauriInvoke<any>('lan_get_all_unread_counts', { userId })
      return res.success ? (res.data ?? {}) : {}
    },
    lanGetStatus: async (): Promise<any> => {
      const res = await tauriInvoke<any>('lan_get_status')
      return res.success ? (res.data ?? {}) : {}
    },
    lanGetNetworkInfo: async (): Promise<any> => {
      const res = await tauriInvoke<any>('lan_get_network_info')
      return res.success ? (res.data ?? {}) : {}
    },
    lanGetReceivePath: async (): Promise<string> => {
      const res = await tauriInvoke<string>('lan_get_receive_path')
      return res.success ? (res.data ?? '') : ''
    },
    lanGetPeers: async (): Promise<any[]> => {
      const res = await tauriInvoke<any[]>('lan_get_peers')
      return res.success ? (res.data ?? []) : []
    },
    lanSetNickName: async (name: string): Promise<void> => {
      await tauriInvoke('lan_set_nick_name', { name })
    },
    lanSetAvatar: async (avatar: string): Promise<void> => {
      await tauriInvoke('lan_set_avatar', { avatar })
    },
    lanUploadAvatar: async (filePath: string): Promise<{ path: string; fullPath: string }> => {
      const res = await tauriInvoke<{ path: string; fullPath: string }>('lan_upload_avatar', { filePath })
      if (res.success && res.data) {return res.data}
      throw new Error(res.error || '上传失败')
    },
    lanGetAvatarPath: async (avatar: string): Promise<{ isEmoji: boolean; path: string }> => {
      const res = await tauriInvoke<{ isEmoji: boolean; path: string }>('lan_get_avatar_path', { avatar })
      if (res.success && res.data) {return res.data}
      return { isEmoji: true, path: avatar }
    },
    lanShowOpenDialogForDirs: async (): Promise<{ filePaths: string[]; canceled: boolean }> => {
      const selected = await open({
        directory: true,
        multiple: false,
        title: '选择文件保存目录',
      })
      if (!selected) {return { filePaths: [], canceled: true }}
      const paths = Array.isArray(selected) ? selected : [selected]
      return { filePaths: paths, canceled: false }
    },
    lanSetReceivePath: async (path: string): Promise<void> => {
      await tauriInvoke('lan_set_receive_path', { path })
    },
    lanGetFileTransferHistory: async (): Promise<any[]> => {
      const res = await tauriInvoke<any[]>('lan_get_file_transfer_history')
      return res.success ? (res.data ?? []) : []
    },
    lanGetLogs: async (): Promise<any[]> => {
      const res = await tauriInvoke<any[]>('lan_get_logs')
      return res.success ? (res.data ?? []) : []
    },
    lanGetUnreadCount: async (peerId: string): Promise<number> => {
      const res = await tauriInvoke<number>('lan_get_unread_count', { peerId })
      return res.success ? (res.data ?? 0) : 0
    },
    lanMarkMessagesRead: async (peerId: string): Promise<void> => {
      await tauriInvoke('lan_mark_messages_read', { peerId })
    },
    // ── 新增 LAN 方法 ──
    lanStop: async (): Promise<void> => {
      await tauriInvoke('lan_stop')
    },
    startLan: async (userId: string, userName: string): Promise<any> => {
      const res = await tauriInvoke<any>('lan_start', { userId, userName })
      return res.success ? res.data : { success: false, error: res.error }
    },
    lanGetMessagesBetween: async (userId1: string, userId2: string, limit: number, offset: number): Promise<any[]> => {
      const res = await tauriInvoke<any>('lan_get_messages_between', { userId1, userId2, limit, offset })
      if (res.success && res.data) {
        const inner = res.data as any
        return (inner.data ?? []) as any[]
      }
      return []
    },
    lanSendMessage: async (peerId: string, content: string): Promise<any> => {
      const res = await tauriInvoke<any>('lan_send_message', { peerId, content })
      return { success: res.success, sent: res.success ? res.data?.sent : false, error: res.error }
    },
    lanSendFile: async (peerId: string, filePath: string, fileName: string, resumeOffset = 0, fileId?: string): Promise<any> => {
      const res = await tauriInvoke<any>('lan_send_file', { peerId, filePath, fileName, resumeOffset, fileId })
      return res.success ? res.data : { success: false, error: res.error }
    },
    lanOnMessage: async (handler: (data: any) => void) => {
      return listen('lan-message-received', (event) => handler(event.payload))
    },
    lanOnFileTransferStarted: async (handler: (data: any) => void) => {
      return listen('lan-file-transfer-started', (event) => handler(event.payload))
    },
    lanOnFileTransferProgress: async (handler: (data: any) => void) => {
      return listen('lan-file-transfer-progress', (event) => handler(event.payload))
    },
    lanOnFileTransferCompleted: async (handler: (data: any) => void) => {
      return listen('lan-file-transfer-completed', (event) => handler(event.payload))
    },
    lanOnFileTransferError: async (handler: (data: any) => void) => {
      return listen('lan-file-transfer-error', (event) => handler(event.payload))
    },
    lanOnFileReceived: async (handler: (data: any) => void) => {
      return listen('lan-file-received', (event) => handler(event.payload))
    },
    lanOnTaskAssigned: async (handler: (data: any) => void) => {
      return listen('lan-task-assigned', (event) => handler(event.payload))
    },
    lanAssignTask: async (peerId: string, task: string): Promise<void> => {
      await tauriInvoke('lan_assign_task', { peerId, task })
    },
    lanSyncTaskStatus: async (taskJson: string): Promise<void> => {
      await tauriInvoke('lan_sync_task_status', { taskJson })
    },
    lanBroadcastMessage: async (message: string): Promise<void> => {
      await tauriInvoke('lan_broadcast_message', { message })
    },
    lanBroadcastTaskUpdate: async (task: string): Promise<void> => {
      await tauriInvoke('lan_broadcast_task_update', { task })
    },
    lanBroadcastTaskStatusChange: async (task: string): Promise<void> => {
      await tauriInvoke('lan_broadcast_task_status_change', { task })
    },
    lanBroadcastTaskComment: async (data: string): Promise<void> => {
      await tauriInvoke('lan_broadcast_task_comment', { data })
    },
    lanBroadcastCollaborationStarted: async (data: string): Promise<void> => {
      await tauriInvoke('lan_broadcast_collaboration_started', { data })
    },
    lanBroadcastCollaborationEnded: async (data: string): Promise<void> => {
      await tauriInvoke('lan_broadcast_collaboration_ended', { data })
    },
    lanScreenshot: async (): Promise<any> => {
      const res = await tauriInvoke<any>('lan_screenshot')
      return res.success ? (res.data ?? {}) : {}
    },
    lanSaveTempFile: async (base64Data: string, fileName: string): Promise<any> => {
      const res = await tauriInvoke<any>('lan_save_temp_file', { base64Data, fileName })
      return res.success ? (res.data ?? null) : null
    },
    lanLoadLocalFileAsBase64: async (filePath: string): Promise<string> => {
      const res = await tauriInvoke<any>('lan_load_local_file_as_base64', { filePath })
      return res.success ? (res.data ?? '') : ''
    },
    lanOpenFile: async (filePath: string): Promise<void> => {
      await tauriInvoke('lan_open_file', { filePath })
    },
    lanOpenFileFolder: async (filePath: string): Promise<void> => {
      await tauriInvoke('lan_open_file_folder', { filePath })
    },
    lanCheckNetworkPermission: async (): Promise<any> => {
      const res = await tauriInvoke<any>('lan_check_network_permission')
      return res.success ? (res.data ?? {}) : {}
    },
    lanGetPermissionStatus: async (): Promise<any> => {
      const res = await tauriInvoke<any>('lan_get_permission_status')
      return res.success ? (res.data ?? {}) : {}
    },
    // ── LAN 事件监听 ──
    onLanPeerDiscovered: async (handler: (payload: any) => void): Promise<UnlistenFn> => {
      return listen('lan-peer-discovered', (event) => handler(event.payload))
    },
    onLanPeerLost: async (handler: (payload: any) => void): Promise<UnlistenFn> => {
      return listen('lan-peer-lost', (event) => handler(event.payload))
    },
    onLanPeerAvatarUpdated: async (handler: (payload: any) => void): Promise<UnlistenFn> => {
      return listen('lan-peer-avatar-updated', (event) => handler(event.payload))
    },
    onLanMessage: async (handler: (payload: any) => void): Promise<UnlistenFn> => {
      return listen('lan-message-received', (event) => handler(event.payload))
    },
    onLanTaskAssigned: async (handler: (payload: any) => void): Promise<UnlistenFn> => {
      return listen('lan-task-assigned', (event) => handler(event.payload))
    },
    onLanTaskUpdated: async (handler: (payload: any) => void): Promise<UnlistenFn> => {
      return listen('lan-task-updated', (event) => handler(event.payload))
    },
    onLanTaskStatusChanged: async (handler: (payload: any) => void): Promise<UnlistenFn> => {
      return listen('lan-task-status-changed', (event) => handler(event.payload))
    },
    onLanTaskCommentAdded: async (handler: (payload: any) => void): Promise<UnlistenFn> => {
      return listen('lan-task-comment-added', (event) => handler(event.payload))
    },
    onLanCollaborationStarted: async (handler: (payload: any) => void): Promise<UnlistenFn> => {
      return listen('lan-collaboration-started', (event) => handler(event.payload))
    },
    onLanCollaborationEnded: async (handler: (payload: any) => void): Promise<UnlistenFn> => {
      return listen('lan-collaboration-ended', (event) => handler(event.payload))
    },
    onLanFileTransferStarted: async (handler: (payload: any) => void): Promise<UnlistenFn> => {
      return listen('lan-file-transfer-started', (event) => handler(event.payload))
    },
    onLanFileTransferProgress: async (handler: (payload: any) => void): Promise<UnlistenFn> => {
      return listen('lan-file-transfer-progress', (event) => handler(event.payload))
    },
    onLanFileTransferCompleted: async (handler: (payload: any) => void): Promise<UnlistenFn> => {
      return listen('lan-file-transfer-completed', (event) => handler(event.payload))
    },
    onLanFileTransferError: async (handler: (payload: any) => void): Promise<UnlistenFn> => {
      return listen('lan-file-transfer-error', (event) => handler(event.payload))
    },
    onLanFileReceived: async (handler: (payload: any) => void): Promise<UnlistenFn> => {
      return listen('lan-file-received', (event) => handler(event.payload))
    },
    // Redis Stream & ZSet 队列管理
    dbRedisStreams: async (id: string, dbIndex: number, pattern: string): Promise<any> => {
      const res = await tauriInvoke<any>('db_redis_streams', { id, dbIndex, pattern })
      return res.success && res.data ? (res.data.streams ?? []) : []
    },
    dbRedisStreamInfo: async (id: string, dbIndex: number, stream: string): Promise<any> => {
      const res = await tauriInvoke<any>('db_redis_stream_info', { id, dbIndex, stream })
      return res.success && res.data ? (res.data.info ?? null) : null
    },
    dbRedisStreamMessages: async (id: string, dbIndex: number, stream: string, count?: number, start?: string): Promise<any> => {
      const res = await tauriInvoke<any>('db_redis_stream_messages', { id, dbIndex, stream, count: count ?? 100, start: start ?? '-' })
      return res.success && res.data ? (res.data.messages ?? []) : []
    },
    dbRedisStreamAdd: async (id: string, dbIndex: number, stream: string, data: Record<string, string>): Promise<string> => {
      const res = await tauriInvoke<any>('db_redis_stream_add', { id, dbIndex, stream, data })
      return res.success && res.data ? (res.data.message_id ?? '') : ''
    },
    dbRedisStreamDel: async (id: string, dbIndex: number, stream: string): Promise<boolean> => {
      const res = await tauriInvoke<any>('db_redis_stream_del', { id, dbIndex, stream })
      return res.success && res.data?.success === true
    },
    dbRedisStreamDelete: async (id: string, dbIndex: number, stream: string): Promise<boolean> => {
      const res = await tauriInvoke<any>('db_redis_stream_delete', { id, dbIndex, stream })
      return res.success && res.data?.success === true
    },
    dbRedisStreamGroupCreate: async (id: string, dbIndex: number, stream: string, group: string): Promise<boolean> => {
      const res = await tauriInvoke<any>('db_redis_stream_group_create', { id, dbIndex, stream, group })
      return res.success && res.data?.success === true
    },
    dbRedisStreamGroupDestroy: async (id: string, dbIndex: number, stream: string, group: string): Promise<boolean> => {
      const res = await tauriInvoke<any>('db_redis_stream_group_destroy', { id, dbIndex, stream, group })
      return res.success && res.data?.success === true
    },
    dbRedisStreamConsumers: async (id: string, dbIndex: number, stream: string, group: string): Promise<any> => {
      const res = await tauriInvoke<any>('db_redis_stream_consumers', { id, dbIndex, stream, group })
      return res.success && res.data ? (res.data.consumers ?? []) : []
    },
    dbRedisStreamPending: async (id: string, dbIndex: number, stream: string, group: string): Promise<any> => {
      const res = await tauriInvoke<any>('db_redis_stream_pending', { id, dbIndex, stream, group })
      return res.success && res.data ? (res.data.pending ?? []) : []
    },
    dbRedisStreamClaim: async (id: string, dbIndex: number, stream: string, group: string, consumer: string, msgIds: string[]): Promise<any> => {
      const res = await tauriInvoke<any>('db_redis_stream_claim', { id, dbIndex, stream, group, consumer, msgIds })
      return res.success && res.data ? (res.data.result ?? []) : []
    },
    dbRedisStreamAck: async (id: string, dbIndex: number, stream: string, group: string, msgIds: string[]): Promise<boolean> => {
      const res = await tauriInvoke<any>('db_redis_stream_ack', { id, dbIndex, stream, group, msgIds })
      return res.success && res.data?.success === true
    },
    dbRedisStreamRetry: async (id: string, dbIndex: number, stream: string, group: string, consumer: string, msgIds: string[]): Promise<any> => {
      const res = await tauriInvoke<any>('db_redis_stream_retry', { id, dbIndex, stream, group, consumer, msgIds })
      return res.success && res.data ? (res.data.result ?? []) : []
    },
    dbRedisStreamTrim: async (id: string, dbIndex: number, stream: string, count: number): Promise<boolean> => {
      const res = await tauriInvoke<any>('db_redis_stream_trim', { id, dbIndex, stream, count })
      return res.success && res.data?.success === true
    },
    dbRedisScanKeys: async (id: string, dbIndex: number, pattern: string, type?: string): Promise<any> => {
      const res = await tauriInvoke<any>('db_redis_scan_keys', { id, dbIndex, pattern, type: type ?? '*' })
      return res.success && res.data ? (res.data.keys ?? []) : []
    },
    dbRedisZSetRange: async (id: string, dbIndex: number, key: string, start: number, stop: number): Promise<any> => {
      const res = await tauriInvoke<any>('db_redis_zset_range', { id, dbIndex, key, start, stop })
      return res.success && res.data ? (res.data.result ?? []) : []
    },
    dbRedisZSetRemove: async (id: string, dbIndex: number, key: string, members: string[]): Promise<boolean> => {
      const res = await tauriInvoke<any>('db_redis_zset_remove', { id, dbIndex, key, members })
      return res.success && res.data?.success === true
    },
    // 表结构导出
    dbGetTableStructure: async (id: string, table: string, dbName: string): Promise<any> => {
      const res = await tauriInvoke<any>('db_get_table_structure', { id, table, dbName })
      if (res.success) {
        return { rows: res.rows ?? res.data?.rows ?? [], indexes: res.indexes ?? res.data?.indexes ?? [] }
      }
      return { rows: [], indexes: [] }
    },
  }
}

// ============ Todo ============

export function useTodosAPI() {
  return {
    getAllTodos: async (): Promise<Todo[]> => {
      const res = await tauriInvoke<Todo[]>('get_all_todos')
      return res.success ? (res.data ?? []) : []
    },
    addTodo: async (todo: Partial<Todo>): Promise<Todo> => {
      const full = {
        id: todo.id ?? crypto.randomUUID(), text: todo.text ?? '',
        completed: todo.completed ?? false, priority: todo.priority ?? 'medium',
        dueDate: todo.dueDate ?? null, description: todo.description ?? '',
        tag: todo.tag ?? null, projectId: todo.projectId ?? null,
        orderNum: todo.orderNum ?? 0, assignedBy: todo.assignedTo ?? null,
        createdAt: todo.createdAt ?? new Date().toISOString(),
        updatedAt: todo.updatedAt ?? new Date().toISOString(),
      }
      const res = await tauriInvoke<Todo>('add_todo', { todo: full })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    updateTodo: async (todo: Todo): Promise<Todo> => {
      const res = await tauriInvoke<Todo>('update_todo', { params: todo })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    deleteTodo: async (id: string): Promise<void> => {
      const res = await tauriInvoke<string>('delete_todo', { id })
      if (!res.success) {throw new Error(res.error)}
    },
    getAllTags: async (): Promise<Tag[]> => {
      const res = await tauriInvoke<Tag[]>('get_all_tags')
      return res.success ? (res.data ?? []) : []
    },
    addTag: async (tag: { name: string; color?: string }): Promise<Tag> => {
      const res = await tauriInvoke<Tag>('add_tag', { tag })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    deleteTag: async (id: string): Promise<void> => {
      const res = await tauriInvoke<string>('delete_tag', { id })
      if (!res.success) {throw new Error(res.error)}
    },
    addSubtask: async (subtask: Partial<Subtask>): Promise<Subtask> => {
      const full = {
        id: subtask.id ?? crypto.randomUUID(), todoId: subtask.todoId ?? '',
        text: subtask.text ?? '', completed: subtask.completed ?? false,
        orderNum: subtask.orderNum ?? 0,
        createdAt: subtask.createdAt ?? new Date().toISOString(),
        updatedAt: subtask.updatedAt ?? new Date().toISOString(),
      }
      const res = await tauriInvoke<Subtask>('add_subtask', { subtask: full })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    updateSubtask: async (id: string, updates: Partial<Subtask>): Promise<Subtask> => {
      const res = await tauriInvoke<Subtask>('update_subtask', { id, updates })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    deleteSubtask: async (id: string): Promise<void> => {
      const res = await tauriInvoke<string>('delete_subtask', { id })
      if (!res.success) {throw new Error(res.error)}
    },
    getSubtasksForTodo: async (todoId: string): Promise<Subtask[]> => {
      const res = await tauriInvoke<Subtask[]>('get_subtasks_for_todo', { todoId })
      return res.success ? (res.data ?? []) : []
    },
  }
}

// ============ 笔记 ============

export function useNotesAPI() {
  return {
    getAllNotes: async (): Promise<Note[]> => {
      const res = await tauriInvoke<Note[]>('get_all_notes')
      return res.success ? (res.data ?? []) : []
    },
    addNote: async (note: Partial<Note>): Promise<Note> => {
      const full = {
        id: note.id ?? crypto.randomUUID(), title: note.title ?? '',
        content: note.content ?? '', groupId: note.groupId ?? null,
        createdAt: note.createdAt ?? new Date().toISOString(),
        updatedAt: note.updatedAt ?? new Date().toISOString(),
      }
      const res = await tauriInvoke<Note>('add_note', { note: full })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    updateNote: async (id: string, updates: Partial<Note>): Promise<Note> => {
      const res = await tauriInvoke<Note>('update_note', { id, updates })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    deleteNote: async (id: string): Promise<void> => {
      const res = await tauriInvoke<string>('delete_note', { id })
      if (!res.success) {throw new Error(res.error)}
    },
    getAllNoteGroups: async (): Promise<NoteGroup[]> => {
      const res = await tauriInvoke<NoteGroup[]>('get_all_note_groups')
      return res.success ? (res.data ?? []) : []
    },
    addNoteGroup: async (group: Partial<NoteGroup>): Promise<NoteGroup> => {
      const full = {
        id: group.id ?? crypto.randomUUID(), name: group.name ?? '',
        color: group.color ?? '#6c63ff',
        createdAt: group.createdAt ?? new Date().toISOString(),
        updatedAt: group.updatedAt ?? new Date().toISOString(),
      }
      const res = await tauriInvoke<NoteGroup>('add_note_group', { group: full })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    updateNoteGroup: async (id: string, updates: Partial<NoteGroup>): Promise<NoteGroup> => {
      const res = await tauriInvoke<NoteGroup>('update_note_group', { id, updates })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    deleteNoteGroup: async (id: string): Promise<void> => {
      const res = await tauriInvoke<string>('delete_note_group', { id })
      if (!res.success) {throw new Error(res.error)}
    },
  }
}

// ============ 周报 ============

export function useWeeklyAPI() {
  return {
    getWeeklyReports: async (params?: { weekStart?: string; weekEnd?: string }): Promise<WeeklyReport[]> => {
      const res = await tauriInvoke<WeeklyReport[]>('get_weekly_reports', { params: params ?? {} })
      return res.success ? (res.data ?? []) : []
    },
    getWeeklyReport: async (id: string): Promise<WeeklyReport | null> => {
      const res = await tauriInvoke<WeeklyReport | null>('get_weekly_report', { id })
      return res.success ? (res.data ?? null) : null
    },
    saveWeeklyReport: async (report: Partial<WeeklyReport>): Promise<WeeklyReport> => {
      const full = {
        id: report.id ?? crypto.randomUUID(), weekStart: report.weekStart ?? '',
        weekEnd: report.weekEnd ?? '', content: report.content ?? '',
        createdAt: report.createdAt ?? new Date().toISOString(),
        updatedAt: report.updatedAt ?? new Date().toISOString(),
      }
      const res = await tauriInvoke<WeeklyReport>('save_weekly_report', { report: full })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
  }
}

// ============ MFA ============

export function useMfaAPI() {
  return {
    getAllMfaSecrets: async (): Promise<MfaSecret[]> => {
      const res = await tauriInvoke<MfaSecret[]>('get_all_mfa_secrets')
      return res.success ? (res.data ?? []) : []
    },
    addMfaSecret: async (secret: Partial<MfaSecret>): Promise<MfaSecret> => {
      const full = {
        id: secret.id ?? crypto.randomUUID(), name: secret.name ?? '',
        secret: secret.secret ?? '', issuer: secret.issuer ?? null,
        account: secret.account ?? null,
        algorithm: secret.algorithm ?? 'SHA1', digits: secret.digits ?? 6,
        period: secret.period ?? 30,
        createdAt: secret.createdAt ?? new Date().toISOString(),
        updatedAt: secret.updatedAt ?? new Date().toISOString(),
      }
      const res = await tauriInvoke<MfaSecret>('add_mfa_secret', { secret: full })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    updateMfaSecret: async (id: string, updates: Partial<MfaSecret>): Promise<MfaSecret> => {
      const res = await tauriInvoke<MfaSecret>('update_mfa_secret', { id, updates })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    deleteMfaSecret: async (id: string): Promise<void> => {
      const res = await tauriInvoke<string>('delete_mfa_secret', { id })
      if (!res.success) {throw new Error(res.error)}
    },
    generateTotp: async (secret: string, digits?: number, period?: number, algorithm?: string): Promise<string> => {
      const res = await tauriInvoke<string>('generate_totp', { secret, digits: digits ?? 6, period: period ?? 30, algorithm: algorithm ?? 'SHA1' })
      return res.success ? (res.data ?? '') : ''
    },
  }
}

// ============ 记账 ============

export function useAccountingAPI() {
  return {
    getAccountingRecords: async (params?: Record<string, unknown>): Promise<{ records: AccountingRecord[], total: number }> => {
      const res = await tauriInvoke<{ records: AccountingRecord[], total: number }>('get_accounting_records', { params: params ?? {} })
      return res.success ? (res.data ?? { records: [], total: 0 }) : { records: [], total: 0 }
    },
    addAccountingRecord: async (record: Partial<AccountingRecord>): Promise<AccountingRecord> => {
      const full = {
        id: record.id ?? crypto.randomUUID(), categoryId: record.category ?? '',
        amount: record.amount ?? 0, type: record.type ?? 'expense',
        description: record.description ?? '', date: record.date ?? new Date().toISOString().slice(0, 10),
        receiptPath: record.receiptPath ?? null,
        createdAt: record.createdAt ?? new Date().toISOString(),
        updatedAt: record.updatedAt ?? new Date().toISOString(),
      }
      const res = await tauriInvoke<AccountingRecord>('add_accounting_record', { record: full })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    updateAccountingRecord: async (id: string, updates: Partial<AccountingRecord>): Promise<AccountingRecord> => {
      const res = await tauriInvoke<AccountingRecord>('update_accounting_record', { id, updates })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    deleteAccountingRecord: async (id: string): Promise<void> => {
      const res = await tauriInvoke<string>('delete_accounting_record', { id })
      if (!res.success) {throw new Error(res.error)}
    },
    getAccountingCategories: async (): Promise<AccountingCategory[]> => {
      const res = await tauriInvoke<AccountingCategory[]>('get_accounting_categories')
      return res.success ? (res.data ?? []) : []
    },
    addAccountingCategory: async (cat: Partial<AccountingCategory>): Promise<AccountingCategory> => {
      const full = {
        id: cat.id ?? crypto.randomUUID(), name: cat.name ?? '',
        type: cat.type ?? 'expense', color: cat.color ?? '#6c63ff',
        icon: cat.icon ?? '',
        createdAt: cat.createdAt ?? new Date().toISOString(),
        updatedAt: cat.updatedAt ?? new Date().toISOString(),
      }
      const res = await tauriInvoke<AccountingCategory>('add_accounting_category', { category: full })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    updateAccountingCategory: async (id: string, updates: Partial<AccountingCategory>): Promise<AccountingCategory> => {
      const res = await tauriInvoke<AccountingCategory>('update_accounting_category', { id, updates })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    deleteAccountingCategory: async (id: string): Promise<void> => {
      const res = await tauriInvoke<string>('delete_accounting_category', { id })
      if (!res.success) {throw new Error(res.error)}
    },
    getAccountingStats: async (params?: Record<string, unknown>): Promise<any> => {
      const res = await tauriInvoke<any>('get_accounting_stats', { params: params ?? {} })
      return res.success ? (res.data ?? {}) : {}
    },
    getAccountingTrend: async (months?: number): Promise<any> => {
      const res = await tauriInvoke<any>('get_accounting_trend', { months: months ?? 12 })
      return res.success ? (res.data ?? []) : []
    },
    getBudgets: async (): Promise<Budget[]> => {
      const res = await tauriInvoke<Budget[]>('get_budgets')
      return res.success ? (res.data ?? []) : []
    },
    addBudget: async (budget: Partial<Budget>): Promise<Budget> => {
      const full = {
        category: budget.category ?? '',
        amount: budget.amount ?? 0,
      }
      const res = await tauriInvoke<Budget>('add_budget', { budget: full })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    updateBudget: async (id: string, updates: Partial<Budget>): Promise<Budget> => {
      const res = await tauriInvoke<Budget>('update_budget', { id, updates })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    deleteBudget: async (id: string): Promise<void> => {
      const res = await tauriInvoke<string>('delete_budget', { id })
      if (!res.success) {throw new Error(res.error)}
    },
    checkBudgetAlerts: async (): Promise<any> => {
      const res = await tauriInvoke<any>('check_budget_alerts')
      return res.success ? (res.data ?? {}) : {}
    },
    getTemplates: async (): Promise<any> => {
      const res = await tauriInvoke<any>('get_templates')
      return res.success ? (res.data ?? []) : []
    },
    addTemplate: async (template: Record<string, unknown>): Promise<any> => {
      const res = await tauriInvoke<any>('add_template', { template })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    updateTemplate: async (id: string, updates: Record<string, unknown>): Promise<any> => {
      const res = await tauriInvoke<any>('update_template', { id, updates })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    deleteTemplate: async (id: string): Promise<void> => {
      const res = await tauriInvoke<string>('delete_template', { id })
      if (!res.success) {throw new Error(res.error)}
    },
    useTemplate: async (id: string): Promise<any> => {
      const res = await tauriInvoke<any>('use_template', { id })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    uploadAccountingReceipt: async (name: string, data: string): Promise<any> => {
      const res = await tauriInvoke<any>('upload_accounting_receipt', { fileName: name, base64Data: data })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    getAccountingReceiptFile: async (filePath: string): Promise<any> => {
      const res = await tauriInvoke<any>('get_accounting_receipt_file', { filePath })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    exportAccountingCSV: async (params?: Record<string, unknown>): Promise<any> => {
      const res = await tauriInvoke<any>('export_accounting_csv', { params: params ?? {} })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
  }
}

// ============ 日志 ============

export function useLogsAPI() {
  return {
    getLogPresets: async (): Promise<LogPreset[]> => {
      const res = await tauriInvoke<LogPreset[]>('get_log_presets')
      return res.success ? (res.data ?? []) : []
    },
    addLogPreset: async (preset: Partial<LogPreset>): Promise<LogPreset> => {
      const full = {
        id: preset.id ?? crypto.randomUUID(), name: preset.name ?? '',
        path: preset.path ?? '', description: preset.description ?? '',
        createdAt: preset.createdAt ?? new Date().toISOString(),
        updatedAt: preset.updatedAt ?? new Date().toISOString(),
      }
      const res = await tauriInvoke<LogPreset>('add_log_preset', { preset: full })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    updateLogPreset: async (id: string, updates: Partial<LogPreset>): Promise<LogPreset> => {
      const res = await tauriInvoke<LogPreset>('update_log_preset', { id, updates })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    deleteLogPreset: async (id: string): Promise<void> => {
      const res = await tauriInvoke<string>('delete_log_preset', { id })
      if (!res.success) {throw new Error(res.error)}
    },
    logSearch: async (params: { preset_id: string; keyword: string; lines?: number }): Promise<any> => {
      const res = await tauriCall<any>('log_search', {
        presetId: params.preset_id,
        keyword: params.keyword,
        lines: params.lines ?? 50
      })
      return res
    },
    logTail: async (params: { preset_id: string; lines?: number }): Promise<any> => {
      const res = await tauriCall<any>('log_tail', {
        presetId: params.preset_id,
        lines: params.lines ?? 100
      })
      return res
    },
  }
}

// ============ Nginx ============

export function useNginxAPI() {
  return {
    // Presets
    getNginxPresets: async (): Promise<any> => tauriCall('get_all_nginx_presets'),
    addNginxPreset: async (preset: any): Promise<any> => tauriCall('add_nginx_preset', { preset }),
    updateNginxPreset: async (preset: any): Promise<any> => tauriCall('update_nginx_preset', { preset }),
    deleteNginxPreset: async (id: string): Promise<any> => tauriCall('delete_nginx_preset', { id }),
    // Config
    fetchNginxConfig: async (serverId: string, configPath: string): Promise<any> => tauriCall('fetch_nginx_config', { serverId, configPath }),
    testNginxConfig: async (serverId: string, configPath: string): Promise<any> => tauriCall('test_nginx_config', { serverId, configPath }),
    testNginxConfigContent: async (serverId: string, configPath: string, content: string): Promise<any> => tauriCall('test_nginx_config_content', { serverId, configPath, content }),
    deployNginxConfig: async (serverId: string, configPath: string, content: string, comment: string): Promise<any> => tauriCall('deploy_nginx_config', { serverId, configPath, content, comment }),
    getNginxConfigVersions: async (presetId: string): Promise<any> => tauriCall('get_nginx_config_versions', { presetId }),
    saveNginxConfigVersion: async (version: any): Promise<any> => tauriCall('save_nginx_config_version', { version }),
    setActiveNginxVersion: async (presetId: string, versionId: string): Promise<any> => tauriCall('set_active_nginx_version', { presetId, versionId }),
    generateNginxConfig: async (presetId: string): Promise<any> => tauriCall('generate_nginx_config', { presetId }),
    generateNginxConfigDecomposed: async (presetId: string): Promise<any> => tauriCall('generate_nginx_config_decomposed', { presetId }),
    previewNginxServer: async (presetId: string, server: any, locations: any[]): Promise<any> => tauriCall('preview_nginx_server', { presetId, server, locations }),
    deployNginxConfigDecomposed: async (serverId: string, configPath: string, mainContent: string, subFiles: Array<{filename: string, content: string}>, comment: string): Promise<any> =>
      tauriCall('deploy_nginx_config_decomposed', { serverId, configPath, mainContent, subFiles, comment }),
    // Servers
    getServersByPreset: async (presetId: string): Promise<any> => tauriCall('get_servers_by_preset', { presetId }),
    addNginxServer: async (server: any): Promise<any> => tauriCall('add_nginx_server', { server }),
    updateNginxServer: async (server: any): Promise<any> => tauriCall('update_nginx_server', { server }),
    deleteNginxServer: async (id: string): Promise<any> => tauriCall('delete_nginx_server', { id }),
    // Locations
    getLocationsByServer: async (serverId: string): Promise<any> => tauriCall('get_locations_by_server', { serverId }),
    addNginxLocation: async (location: any): Promise<any> => tauriCall('add_nginx_location', { location }),
    updateNginxLocation: async (location: any): Promise<any> => tauriCall('update_nginx_location', { location }),
    deleteNginxLocation: async (id: string): Promise<any> => tauriCall('delete_nginx_location', { id }),
    // Upstreams
    getUpstreamsByPreset: async (presetId: string): Promise<any> => tauriCall('get_upstreams_by_preset', { presetId }),
    addNginxUpstream: async (upstream: any): Promise<any> => tauriCall('add_nginx_upstream', { upstream }),
    updateNginxUpstream: async (upstream: any): Promise<any> => tauriCall('update_nginx_upstream', { upstream }),
    deleteNginxUpstream: async (id: string): Promise<any> => tauriCall('delete_nginx_upstream', { id }),
    // Upstream Servers
    getUpstreamServers: async (upstreamId: string): Promise<any> => tauriCall('get_upstream_servers', { upstreamId }),
    addNginxUpstreamServer: async (upstreamServer: any): Promise<any> => tauriCall('add_nginx_upstream_server', { upstreamServer }),
    updateNginxUpstreamServer: async (upstreamServer: any): Promise<any> => tauriCall('update_nginx_upstream_server', { upstreamServer }),
    deleteNginxUpstreamServer: async (id: string): Promise<any> => tauriCall('delete_nginx_upstream_server', { id }),
    // HTTP Params
    getHttpParamsByPreset: async (presetId: string): Promise<any> => tauriCall('get_http_params_by_preset', { presetId }),
    addNginxHttpParam: async (param: any): Promise<any> => tauriCall('add_nginx_http_param', { param }),
    updateNginxHttpParam: async (param: any): Promise<any> => tauriCall('update_nginx_http_param', { param }),
    deleteNginxHttpParam: async (id: string): Promise<any> => tauriCall('delete_nginx_http_param', { id }),
    // Streams
    getStreamsByPreset: async (presetId: string): Promise<any> => tauriCall('get_streams_by_preset', { presetId }),
    addNginxStream: async (stream: any): Promise<any> => tauriCall('add_nginx_stream', { stream }),
    updateNginxStream: async (stream: any): Promise<any> => tauriCall('update_nginx_stream', { stream }),
    deleteNginxStream: async (id: string): Promise<any> => tauriCall('delete_nginx_stream', { id }),
    // Certs
    getCertsByPreset: async (presetId: string): Promise<any> => tauriCall('get_certs_by_preset', { presetId }),
    addNginxCert: async (cert: any): Promise<any> => tauriCall('add_nginx_cert', { cert }),
    updateNginxCert: async (cert: any): Promise<any> => tauriCall('update_nginx_cert', { cert }),
    deleteNginxCert: async (id: string): Promise<any> => tauriCall('delete_nginx_cert', { id }),
    // Templates
    getTemplatesByPreset: async (presetId: string): Promise<any> => tauriCall('get_templates_by_preset', { presetId }),
    addNginxTemplate: async (template: any): Promise<any> => tauriCall('add_nginx_template', { template }),
    updateNginxTemplate: async (template: any): Promise<any> => tauriCall('update_nginx_template', { template }),
    deleteNginxTemplate: async (id: string): Promise<any> => tauriCall('delete_nginx_template', { id }),
    // Basic Settings (key-value)
    getBasicSettings: async (presetId: string): Promise<any> => tauriCall('get_basic_settings', { presetId }),
    saveBasicSettings: async (presetId: string, settings: any[]): Promise<any> => tauriCall('save_basic_settings', { presetId, settings }),
    // Params
    getParamsByPreset: async (presetId: string): Promise<any> => tauriCall('get_params_by_preset', { presetId }),
    addNginxParam: async (param: any): Promise<any> => tauriCall('add_nginx_param', { param }),
    updateNginxParam: async (param: any): Promise<any> => tauriCall('update_nginx_param', { param }),
    deleteNginxParam: async (id: string): Promise<any> => tauriCall('delete_nginx_param', { id }),
    // Deny/Allow
    getDenyAllowsByPreset: async (presetId: string): Promise<any> => tauriCall('get_deny_allows_by_preset', { presetId }),
    addNginxDenyAllow: async (denyAllow: any): Promise<any> => tauriCall('add_nginx_deny_allow', { denyAllow }),
    updateNginxDenyAllow: async (denyAllow: any): Promise<any> => tauriCall('update_nginx_deny_allow', { denyAllow }),
    deleteNginxDenyAllow: async (id: string): Promise<any> => tauriCall('delete_nginx_deny_allow', { id }),
    // Passwords
    getPasswordsByPreset: async (presetId: string): Promise<any> => tauriCall('get_passwords_by_preset', { presetId }),
    addNginxPassword: async (password: any): Promise<any> => tauriCall('add_nginx_password', { password }),
    updateNginxPassword: async (password: any): Promise<any> => tauriCall('update_nginx_password', { password }),
    deleteNginxPassword: async (id: string): Promise<any> => tauriCall('delete_nginx_password', { id }),
    // Import config
    importNginxConfig: async (presetId: string, configText: string): Promise<any> => tauriCall('import_nginx_config', { presetId, configText }),
    getNginxPresetStats: async (presetId: string): Promise<any> => tauriCall('get_nginx_preset_stats', { presetId }),
  };
}

// ============ 告警 ============

export function useAlertAPI() {
  return {
    getEmailConfig: async (): Promise<any> => tauriCall('get_email_config'),
    saveEmailConfig: async (config: any): Promise<any> => tauriCall('save_email_config', { ...config }),
    testEmailConfig: async (config: any): Promise<any> => tauriCall('test_email_config', { ...config }),
    getAlertServices: async (): Promise<any> => tauriCall('get_alert_services'),
    addAlertService: async (service: any): Promise<any> => tauriCall('add_alert_service', { service }),
    updateAlertService: async (service: any): Promise<any> => tauriCall('update_alert_service', { service }),
    deleteAlertService: async (id: string): Promise<any> => tauriCall('delete_alert_service', { id }),
    getAlertResources: async (): Promise<any> => tauriCall('get_alert_resources'),
    addAlertResource: async (resource: any): Promise<any> => tauriCall('add_alert_resource', { resource }),
    updateAlertResource: async (resource: any): Promise<any> => tauriCall('update_alert_resource', { resource }),
    deleteAlertResource: async (id: string): Promise<any> => tauriCall('delete_alert_resource', { id }),
    getAlertHistory: async (): Promise<any> => tauriCall('get_alert_history'),
    triggerAlertCheck: async (): Promise<any> => tauriCall('trigger_alert_check'),
  };
}

// ============ 设置 ============

export function useSettingsAPI() {
  return {
    getMenuIcon: async (key: string): Promise<string | null> => {
      const res = await tauriInvoke<string | null>('get_menu_icon', { key })
      return res.success ? (res.data ?? null) : null
    },
    getSetting: async (key: string): Promise<any> => {
      const res = await tauriInvoke<any>('get_setting', { key })
      return res.success ? res.data : null
    },
    setSetting: async (key: string, value: any): Promise<void> => {
      const res = await tauriInvoke<string>('set_setting', { key, value })
      if (!res.success) {throw new Error(res.error)}
    },
    getNotificationSettings: async (): Promise<NotificationSettings | null> => {
      const res = await tauriInvoke<NotificationSettings | null>('get_notification_settings')
      return res.success ? (res.data ?? null) : null
    },
    setNotificationSettings: async (settings: NotificationSettings): Promise<void> => {
      const res = await tauriInvoke<string>('set_notification_settings', { settings })
      if (!res.success) {throw new Error(res.error)}
    },
  }
}

// ============ 数据备份 ============

export function useDataBackupAPI() {
  return {
    exportAllData: async (): Promise<any> => {
      const res = await tauriInvoke<any>('export_all_data')
      return res.success ? (res.data ?? res) : { success: false, error: res.error }
    },
    exportData: async (options: Record<string, unknown>): Promise<any> => {
      const res = await tauriInvoke<any>('export_data', options)
      return res.success ? (res.data ?? res) : { success: false, error: res.error }
    },
    importJson: async (options: Record<string, unknown>): Promise<any> => {
      const rustArgs: Record<string, unknown> = {}
      for (const [k, v] of Object.entries(options)) {
        const rustKey = k === 'importMode' ? 'import_mode' : k
        rustArgs[rustKey] = v
      }
      const res = await tauriInvoke<any>('import_json', rustArgs)
      return res.success ? (res.data ?? res) : { success: false, error: res.error }
    },
    exportCsv: async (options: Record<string, unknown>): Promise<any> => {
      return await tauriInvoke<any>('export_csv', options)
    },
    importAllData: async (data: any): Promise<void> => {
      const res = await tauriInvoke<string>('import_all_data', { data })
      if (!res.success) {throw new Error(res.error)}
    },
    getAppPath: async (): Promise<string> => {
      const res = await tauriInvoke<string>('get_app_path')
      return res.success ? (res.data ?? '') : ''
    },
    setAutoBackup: async (settings: Record<string, unknown>): Promise<void> => {
      const res = await tauriInvoke<string>('set_setting', { key: 'autoBackupSettings', value: settings })
      if (!res.success) {throw new Error(res.error)}
    },
    getDataDir: async (): Promise<any> => {
      return await tauriCall<any>('get_data_dir')
    },
    setDataDir: async (path: string): Promise<any> => {
      return await tauriCall<any>('set_data_dir', { path })
    },
  }
}

// ============ OpenVPN ============

export function useOpenVPNAPI() {
  return {
    openvpnGetAll: async (): Promise<any[]> => {
      const res = await tauriInvoke<any[]>('openvpn_get_all')
      return res.success ? (res.data ?? []) : []
    },
    openvpnAdd: async (data: { name: string; filePath: string; content: string }): Promise<any> => {
      const res = await tauriInvoke<any>('openvpn_add', { data })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    openvpnDelete: async (id: string): Promise<void> => {
      const res = await tauriInvoke<string>('openvpn_delete', { id })
      if (!res.success) {throw new Error(res.error)}
    },
    openvpnConnect: async (configId: string, configName: string, content: string, sudoPassword?: string): Promise<any> => {
      const res = await tauriInvoke<any>('openvpn_connect', { configId, configName, content, sudoPassword })
      return res.success ? res.data : null
    },
    openvpnRetryWithPassword: async (password: string): Promise<any> => {
      const res = await tauriInvoke<any>('openvpn_retry_with_password', { password })
      return res.success ? res.data : null
    },
    openvpnDisconnect: async (): Promise<void> => {
      const res = await tauriInvoke<string>('openvpn_disconnect')
      if (!res.success) {throw new Error(res.error)}
    },
    openvpnGetStatus: async (): Promise<any> => {
      const res = await tauriInvoke<any>('openvpn_get_status', {}, true)
      return res.success ? res.data : null
    },
    openvpnGetLogs: async (): Promise<string> => {
      const res = await tauriInvoke<string>('openvpn_get_logs')
      return res.success ? (res.data ?? '') : ''
    },
    openvpnCheckAvailable: async (): Promise<boolean> => {
      const res = await tauriInvoke<boolean>('openvpn_check_available')
      return res.success ? (res.data ?? false) : false
    },
    openvpnValidateConfig: async (content: string): Promise<{ valid: boolean; error?: string }> => {
      const res = await tauriInvoke<any>('openvpn_validate_config', { content })
      return res.success ? (res.data ?? { valid: false }) : { valid: false, error: res.error }
    },
    openvpnGetTrafficStats: async (): Promise<any> => {
      const res = await tauriInvoke<any>('openvpn_get_traffic_stats', {}, true)
      return res.success ? res.data : null
    },
  }
}

// ============ WireGuard ============

export function useWireGuardAPI() {
  return {
    wireguardGetAll: async (): Promise<any[]> => {
      const res = await tauriInvoke<any[]>('wireguard_get_all')
      return res.success ? (res.data ?? []) : []
    },
    wireguardGetById: async (id: string): Promise<any> => {
      const res = await tauriInvoke<any>('wireguard_get_by_id', { id })
      return res.success ? res.data : null
    },
    wireguardAdd: async (data: object): Promise<any> => {
      const res = await tauriInvoke<any>('wireguard_add', { data })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    wireguardUpdate: async (data: object): Promise<any> => {
      const res = await tauriInvoke<any>('wireguard_update', { data })
      if (!res.success) {throw new Error(res.error)}
      return res.data!
    },
    wireguardDelete: async (id: string): Promise<void> => {
      const res = await tauriInvoke<string>('wireguard_delete', { id })
      if (!res.success) {throw new Error(res.error)}
    },
    wireguardConnect: async (configId: string, configName: string, privateKey: string, peerPublicKey: string, peerEndpoint: string, presharedKey?: string, address?: string, mtu?: number): Promise<any> => {
      const res = await tauriInvoke<any>('wireguard_connect', { configId, configName, privateKey, peerPublicKey, peerEndpoint, presharedKey, address, mtu })
      return res.success ? res.data : null
    },
    wireguardDisconnect: async (): Promise<void> => {
      const res = await tauriInvoke<string>('wireguard_disconnect')
      if (!res.success) {throw new Error(res.error)}
    },
    wireguardGetStatus: async (): Promise<any> => {
      const res = await tauriInvoke<any>('wireguard_get_status', {}, true)
      return res.success ? res.data : null
    },
    wireguardGenerateKeypair: async (): Promise<{ privateKey: string; publicKey: string }> => {
      const res = await tauriInvoke<{ privateKey: string; publicKey: string }>('wireguard_generate_keypair')
      return res.success ? (res.data ?? { privateKey: '', publicKey: '' }) : { privateKey: '', publicKey: '' }
    },
    wireguardDerivePublicKey: async (privateKey: string): Promise<{ publicKey: string }> => {
      const res = await tauriInvoke<{ publicKey: string }>('wireguard_derive_public_key', { privateKey })
      return res.success ? (res.data ?? { publicKey: '' }) : { publicKey: '' }
    },
  }
}

// ============ 统一 API ============

export interface TauriAPI {
  // Projects
  getProjects: (onlyActive?: boolean) => Promise<Project[]>
  addProject: (project: Partial<Project>) => Promise<Project>
  updateProject: (project: Project) => Promise<Project>
  deleteProject: (id: string) => Promise<void>
  getProjectStats: (projectId: string) => Promise<ProjectStats>
  getProjectTodos: (projectId: string) => Promise<any[]>
  getAllServers: () => Promise<Server[]>
  getServerById: (serverId: string) => Promise<Server | null>
  addServer: (server: Partial<Server>) => Promise<Server>
  updateServer: (server: Server) => Promise<Server>
  deleteServer: (serverId: string) => Promise<void>
  getAllServerGroups: () => Promise<ServerGroup[]>
  getServerGroups: () => Promise<ServerGroup[]>
  addServerGroup: (group: Partial<ServerGroup>) => Promise<ServerGroup>
  updateServerGroup: (groupId: string, updates: any) => Promise<ServerGroup>
  deleteServerGroup: (groupId: string) => Promise<void>
  testServerConnection: (server: Partial<Server>) => Promise<{ success: boolean; error?: string }>
  // Database
  dbConnect: (config: DbConnectionConfig) => Promise<{ success: boolean; error?: string }>
  dbDisconnect: (id: string) => Promise<{ success: boolean; error?: string }>
  dbQuery: (id: string, sql: string) => Promise<{ success: boolean; rows?: any; error?: string }>
  dbGetTables: (id: string, dbName: string) => Promise<{ success: boolean; tables?: any; error?: string }>
  dbGetDatabases: (id: string) => Promise<{ success: boolean; databases?: any; error?: string }>
  dbGetTableStructure: (id: string, table: string, dbName: string) => Promise<any>
  dbGetTablePrimaryKeys: (id: string, table: string, dbName: string) => Promise<any>
  dbGetViews: (id: string, dbName: string) => Promise<any>
  dbGetCreateSql: (id: string, table: string, dbName: string) => Promise<string>
  dbCompareStructures: (sourceId: string, sourceDb: string, targetId: string, targetDb: string) => Promise<any>
  dbExecuteStructureSync: (id: string, sqls: string[], dbName: string) => Promise<any>
  dbCompareData: (params: Record<string, unknown>) => Promise<any>
  dbExecuteDataSync: (params: Record<string, unknown>) => Promise<any>
  dbBackupCreate: (id: string, dbName: string, objects: any[]) => Promise<any>
  dbBackupList: (id?: string) => Promise<any[]>
  dbBackupRestore: (id: string, file: string) => Promise<any>
  dbBackupDelete: (file: string) => Promise<boolean>
  dbRedisDatabases: (id: string) => Promise<number[]>
  dbRedisKeysTree: (id: string, dbIndex: number, pattern: string) => Promise<any>
  dbRedisKeysByType: (id: string, dbIndex: number, type: string) => Promise<any>
  dbRedisKeyInfo: (id: string, dbIndex: number, key: string) => Promise<any>
  dbRedisKeyValue: (id: string, dbIndex: number, key: string) => Promise<any>
  dbRedisSetKey: (id: string, dbIndex: number, key: string, value: string, ttl?: number) => Promise<boolean>
  dbRedisAddKey: (id: string, dbIndex: number, keyType: string, key: string, value: any) => Promise<boolean>
  dbRedisDeleteKey: (id: string, dbIndex: number, key: string) => Promise<boolean>
  dbRedisExec: (id: string, dbIndex: number, command: string) => Promise<any>
  dbTest: (config: Record<string, unknown>) => Promise<any>
  dbGetTableDataFiltered: (filter: Record<string, unknown>) => Promise<any>
  dbGetTablesFiltered: (filter: Record<string, unknown>) => Promise<any>
  dbUpdateTableRow: (connId: string, table: string, oldRow: Record<string, unknown>, newRow: Record<string, unknown>, dbName?: string) => Promise<any>
  dbInsertTableRow: (connId: string, table: string, row: Record<string, unknown>, dbName?: string) => Promise<any>
  dbDeleteTableRow: (connId: string, table: string, row: Record<string, unknown>, dbName?: string) => Promise<any>
  detectToolPaths: () => Promise<Record<string, string>>
  detectBuildTools: () => Promise<any[]>
  detectSdkVersions: () => Promise<Record<string, any>>
  getCicdConfigs: () => Promise<any[]>
  getCicdGroups: () => Promise<any[]>
  getCicdConfigById: (id: string) => Promise<any>
  addCicdConfig: (config: Record<string, unknown>, modules?: any[]) => Promise<any>
  updateCicdConfig: (config: Record<string, unknown>) => Promise<any>
  deleteCicdConfig: (id: string) => Promise<any>
  getDeployModules: (configId: string) => Promise<any[]>
  addDeployModule: (module: Record<string, unknown>) => Promise<any>
  updateDeployModule: (module: Record<string, unknown>) => Promise<any>
  deleteDeployModule: (moduleId: string) => Promise<any>
  scanProject: (localPath: string) => Promise<any>
  scanProjectModules: (projectPath: string) => Promise<any>
  testSsh: (config: Record<string, unknown>) => Promise<any>
  checkNetworkPermission: (host: string, port: number) => Promise<{ success: boolean; error?: string }>
  // Todos
  getAllTodos: () => Promise<Todo[]>
  addTodo: (todo: Partial<Todo>) => Promise<Todo>
  updateTodo: (todo: Todo) => Promise<Todo>
  deleteTodo: (id: string) => Promise<void>
  getAllTags: () => Promise<Tag[]>
  addTag: (tag: { name: string; color?: string }) => Promise<Tag>
  deleteTag: (id: string) => Promise<void>
  addSubtask: (subtask: Partial<Subtask>) => Promise<Subtask>
  updateSubtask: (id: string, updates: Partial<Subtask>) => Promise<Subtask>
  deleteSubtask: (id: string) => Promise<void>
  getSubtasksForTodo: (todoId: string) => Promise<Subtask[]>
  // Notes
  getAllNotes: () => Promise<Note[]>
  addNote: (note: Partial<Note>) => Promise<Note>
  updateNote: (id: string, updates: Partial<Note>) => Promise<Note>
  deleteNote: (id: string) => Promise<void>
  getAllNoteGroups: () => Promise<NoteGroup[]>
  getNoteGroups: () => Promise<NoteGroup[]>
  addNoteGroup: (group: Partial<NoteGroup>) => Promise<NoteGroup>
  updateNoteGroup: (id: string, updates: Partial<NoteGroup>) => Promise<NoteGroup>
  deleteNoteGroup: (id: string) => Promise<void>
  // Weekly
  getWeeklyReports: (params?: any) => Promise<WeeklyReport[]>
  getWeeklyReport: (id: string) => Promise<WeeklyReport | null>
  saveWeeklyReport: (report: Partial<WeeklyReport>) => Promise<WeeklyReport>
  // MFA
  getAllMfaSecrets: () => Promise<MfaSecret[]>
  getMfaSecrets: () => Promise<MfaSecret[]>
  addMfaSecret: (secret: Partial<MfaSecret>) => Promise<MfaSecret>
  updateMfaSecret: (id: string, updates: Partial<MfaSecret>) => Promise<MfaSecret>
  deleteMfaSecret: (id: string) => Promise<void>
  generateTotp: (secret: string) => Promise<string>
  // Accounting
  getAccountingRecords: (params?: any) => Promise<{ records: AccountingRecord[], total: number }>
  addAccountingRecord: (record: Partial<AccountingRecord>) => Promise<AccountingRecord>
  updateAccountingRecord: (id: string, updates: Partial<AccountingRecord>) => Promise<AccountingRecord>
  deleteAccountingRecord: (id: string) => Promise<void>
  getAccountingCategories: () => Promise<AccountingCategory[]>
  addAccountingCategory: (cat: Partial<AccountingCategory>) => Promise<AccountingCategory>
  updateAccountingCategory: (id: string, updates: Partial<AccountingCategory>) => Promise<AccountingCategory>
  deleteAccountingCategory: (id: string) => Promise<void>
  getAccountingStats: (params?: Record<string, unknown>) => Promise<any>
  getAccountingTrend: (months?: number) => Promise<any>
  getBudgets: () => Promise<Budget[]>
  addBudget: (budget: Partial<Budget>) => Promise<Budget>
  updateBudget: (id: string, updates: Partial<Budget>) => Promise<Budget>
  deleteBudget: (id: string) => Promise<void>
  checkBudgetAlerts: () => Promise<any>
  getTemplates: () => Promise<any>
  addTemplate: (template: Record<string, unknown>) => Promise<any>
  updateTemplate: (id: string, updates: Record<string, unknown>) => Promise<any>
  deleteTemplate: (id: string) => Promise<void>
  useTemplate: (id: string) => Promise<any>
  uploadAccountingReceipt: (name: string, data: string) => Promise<any>
  getAccountingReceiptFile: (filePath: string) => Promise<any>
  exportAccountingCSV: (params?: Record<string, unknown>) => Promise<any>
  // Logs
  getLogPresets: () => Promise<LogPreset[]>
  logPresetsGetAll: () => Promise<LogPreset[]>
  addLogPreset: (preset: Partial<LogPreset>) => Promise<LogPreset>
  updateLogPreset: (id: string, updates: Partial<LogPreset>) => Promise<LogPreset>
  deleteLogPreset: (id: string) => Promise<void>
  logSearch: (params: { query: string; presetId?: string; lines?: number }) => Promise<any>
  logTail: (params: { path: string; lines?: number }) => Promise<any>
  // Settings
  getMenuIcon: (key: string) => Promise<string | null>
  getSetting: (key: string) => Promise<any>
  setSetting: (key: string, value: any) => Promise<void>
  getNotificationSettings: () => Promise<NotificationSettings | null>
  setNotificationSettings: (settings: NotificationSettings) => Promise<void>
  // App
  getAppVersion: () => Promise<string>
  // Data Backup
  exportAllData: () => Promise<any>
  exportData: (options: Record<string, unknown>) => Promise<any>
  importJson: (options: Record<string, unknown>) => Promise<any>
  exportCsv: (options: Record<string, unknown>) => Promise<any>
  importAllData: (data: any) => Promise<void>
  setAutoBackup: (settings: Record<string, unknown>) => Promise<void>
  getAppPath: () => Promise<string>
  getDataDir: () => Promise<any>
  setDataDir: (path: string) => Promise<any>
  // OpenVPN
  openvpnGetAll: () => Promise<any[]>
  openvpnAdd: (data: { name: string; filePath: string; content: string }) => Promise<any>
  openvpnDelete: (id: string) => Promise<void>
  openvpnConnect: (configId: string, configName: string, content: string, sudoPassword?: string) => Promise<any>
  openvpnRetryWithPassword: (password: string) => Promise<any>
  openvpnDisconnect: () => Promise<void>
  openvpnGetStatus: () => Promise<any>
  openvpnGetLogs: () => Promise<string>
  openvpnCheckAvailable: () => Promise<boolean>
  openvpnValidateConfig: (content: string) => Promise<{ valid: boolean; error?: string }>
  openvpnGetTrafficStats: () => Promise<any>
  // WireGuard
  wireguardGetAll: () => Promise<any[]>
  wireguardGetById: (id: string) => Promise<any>
  wireguardAdd: (data: object) => Promise<any>
  wireguardUpdate: (data: object) => Promise<any>
  wireguardDelete: (id: string) => Promise<void>
  wireguardConnect: (configId: string, configName: string, privateKey: string, peerPublicKey: string, peerEndpoint: string, presharedKey?: string) => Promise<any>
  wireguardDisconnect: () => Promise<void>
  wireguardGetStatus: () => Promise<any>
  wireguardGenerateKeypair: () => Promise<{ privateKey: string; publicKey: string }>
  wireguardDerivePublicKey: (privateKey: string) => Promise<{ publicKey: string }>
  // Events
  onTaskUpdated: (callback: (data: any) => void) => () => void
  onTaskStatusChanged: (callback: (data: any) => void) => () => void
  gitSyncStatus: () => Promise<any>
  // Git Repo Management
  getGitRepos: () => Promise<any>
  addGitRepo: (repo: Record<string, unknown>) => Promise<any>
  updateGitRepo: (id: number | string, repo: Record<string, unknown>) => Promise<any>
  deleteGitRepo: (id: number | string) => Promise<any>
  validateGitRepoPath: (path: string) => Promise<any>
  showOpenDialogForDirs: () => Promise<any>
  showOpenDialog: (options?: Record<string, unknown>) => Promise<any>
  getGitCommits: (path: string, since?: string) => Promise<any>,
  scanLocalGitRepos: (directories: string[]) => Promise<any>
  getGitBranches: (path: string) => Promise<any>
  openInFileManager: (path: string) => Promise<any>
  getGitCommitDetail: (repoPath: string, commitHash: string) => Promise<any>
  fetchPageContent: (url: string) => Promise<string>
  convertHtmlToMd: (html: string) => Promise<string>
  // Calculator
  getCalculatorHistory: (limit?: number) => Promise<any>
  onMenuNewTask: (callback: () => void) => () => void
  onMenuExportMarkdown: (callback: () => void) => () => void
  onMenuExportWord: (callback: () => void) => () => void
  onMenuExportJson: (callback: () => void) => () => void
  onMenuImportJson: (callback: () => void) => () => void
  onMenuClearCompleted: (callback: () => void) => () => void
  onMenuSearchTasks: (callback: () => void) => () => void
  onMenuSelectAll: (callback: () => void) => () => void
  onMenuDeleteSelected: (callback: () => void) => () => void
  onMenuToggleComplete: (callback: () => void) => () => void
  onMenuSetPriority: (callback: (priority: string) => void) => () => void
  onMenuSetTag: (callback: () => void) => () => void
  onMenuAbout: (callback: () => void) => Promise<UnlistenFn>
  onMenuShortcutsHelp: (callback: () => void) => () => void | Promise<UnlistenFn>
  onMenuNav: (callback: (view: string) => void) => Promise<UnlistenFn>
  onMenuToggleLanPanel: (callback: () => void) => () => void | Promise<UnlistenFn>
  onMenuToggleLocale: (callback: () => void) => () => void | Promise<UnlistenFn>
  onMenuToggleTheme: (callback: () => void) => Promise<UnlistenFn>
  onMenuSearch: (callback: () => void) => Promise<UnlistenFn>
  onMenuSwitchView: (callback: (view: string) => void) => () => void | Promise<UnlistenFn>
  onMenuCheckUpdate: (callback: () => void) => () => void
  onTaskCommentAdded: (callback: (data: any) => void) => () => void
  onTaskAssigned: (callback: (data: any) => void) => () => void
  onDataChanged: (callback: (data: any) => void) => Promise<() => void>
  onDeployProgress: (callback: (data: any) => void) => () => void
  onDeployNotification: (callback: (data: any) => void) => () => void
  onDeployLogIdCreated: (callback: (data: any) => void) => Promise<UnlistenFn>
  onFileReceived: (callback: (data: any) => void) => () => void
  onFileTransferCompleted: (callback: (data: any) => void) => () => void
  onFileTransferError: (callback: (data: any) => void) => () => void
  onFileTransferProgress: (callback: (data: any) => void) => () => void
  onFileTransferStarted: (callback: (data: any) => void) => () => void
  onGitSyncStatusUpdated: (callback: (data: any) => void) => () => void
  onLanPeerDiscovered: (callback: (data: any) => void) => () => void
  onLanPeerLost: (callback: (data: any) => void) => () => void
  onLanPeerAvatarUpdated: (callback: (data: any) => void) => () => void
  onMessage: (callback: (data: any) => void) => () => void
  onServerConnected: (callback: (data: any) => void) => () => void
  onServerDisconnected: (callback: (data: any) => void) => () => void
  onServerHeartbeatFailed: (callback: (data: any) => void) => () => void
  onSftpDownloadProgress: (callback: (data: any) => void) => () => void
  onSftpUploadDone: (callback: (data: any) => void) => () => void
  onSftpUploadProgress: (callback: (data: any) => void) => () => void
  onTerminalClose: (callback: (data: any) => void) => () => void
  onTerminalData: (callback: (data: any) => void) => () => void
  onAutoBackupCompleted: (callback: (data: any) => void) => () => void
  onCollaborationStarted: (callback: (data: any) => void) => () => void
  onCollaborationEnded: (callback: (data: any) => void) => () => void
  importOvpnFile: () => Promise<any>
  readFileContent: (filePath: string) => Promise<string>
  getUserInfo: (userId: string) => Promise<any>
  lanGetUserInfo: () => Promise<any>
  setStatus: (status: string) => Promise<any>
  refreshDiscovery: () => Promise<any>
  getAllUnreadCounts: (userId: string) => Promise<any>
  getStatus: (userId: string) => Promise<any>
  getNetworkInfo: () => Promise<any>
  getReceivePath: () => Promise<any>
  getPeers: () => Promise<any>
  setNickName: (name: string) => Promise<any>
  setAvatar: (avatar: string) => Promise<any>
  setReceivePath: (path: string) => Promise<any>
  getMessagesBetween: (userId: string, limit?: number) => Promise<any>
  markMessagesRead: (userId: string) => Promise<any>
  // ── LAN Chat ──
  lanGetMessagesBetween: (userId1: string, userId2: string, limit: number, offset: number) => Promise<any[]>
  lanSendMessage: (peerId: string, content: string) => Promise<any>
  lanSendFile: (peerId: string, filePath: string, fileName: string, resumeOffset?: number, fileId?: string) => Promise<any>
  lanMarkMessagesRead: (peerId: string) => Promise<any>
  lanGetUnreadCount: (peerId: string) => Promise<number>
  lanGetAllUnreadCounts: (userId: string) => Promise<any>
  lanOnMessage: (handler: (data: any) => void) => Promise<() => void>
  lanOnFileTransferStarted: (handler: (data: any) => void) => Promise<() => void>
  lanOnFileTransferProgress: (handler: (data: any) => void) => Promise<() => void>
  lanOnFileTransferCompleted: (handler: (data: any) => void) => Promise<() => void>
  lanOnFileTransferError: (handler: (data: any) => void) => Promise<() => void>
  lanOnFileReceived: (handler: (data: any) => void) => Promise<() => void>
  lanOnTaskAssigned: (handler: (data: any) => void) => Promise<() => void>
  lanSaveTempFile: (base64Data: string, fileName: string) => Promise<any>
  lanScreenshot: () => Promise<any>
  startLan: (userId: string, userName: string) => Promise<any>
  sendFile: (userId: string, filePath: string) => Promise<any>
  openFile: (filePath: string) => Promise<any>
  openFileFolder: (filePath: string) => Promise<any>
  saveTempFile: (base64Data: string, fileName: string) => Promise<string | null>
  loadLocalFileAsBase64: (filePath: string) => Promise<string>
  connectServer: (serverId: string) => Promise<any>
  disconnectServer: (serverId: string) => Promise<any>
  isServerConnected: (serverId: string) => Promise<any>
    createTerminal: (serverId: string, terminalId: string, rows?: number, cols?: number) => Promise<any>
  writeTerminal: (terminalId: string, data: string) => Promise<any>
  resizeTerminal: (terminalId: string, cols: number, rows: number) => Promise<any>
  closeTerminal: (terminalId: string) => Promise<any>
  readTerminal: (terminalId: string) => Promise<any>
  isTerminalActive: (terminalId: string) => Promise<any>
  deploy: (configId: string, confirmed?: boolean) => Promise<any>
  cancelDeploy: (deployLogId: string) => Promise<any>
  rollback: (configId: string, logId: string) => Promise<any>
  getDeployLogs: (configId: string, limit?: number) => Promise<any>
  getDeployStepLogs: (deployLogId: string) => Promise<any>
  readLogFile: (logId: string) => Promise<any>
  writeLogFile: (logId: string, content: string) => Promise<any>
  writeSystemLog: (level: string, prefix: string, message: string) => Promise<void>
  getServerMonitor: (serverId: string, commands: string[]) => Promise<any>
  listSftpDir: (serverId: string, path: string) => Promise<any>
  openSftpFileEditor: (serverId: string, filePath: string) => Promise<any>
  deleteSftpFile: (serverId: string, filePath: string, isDir?: boolean) => Promise<any>
  getDownloadsDir: () => Promise<any>
  downloadFile: (serverId: string, remotePath: string, localPath: string) => Promise<any>
  uploadFile: (serverId: string, localPath: string, remotePath: string) => Promise<any>
  uploadFolder: (serverId: string, localPath: string, remotePath: string) => Promise<any>
  uploadSessionStart: (serverId: string, remotePath: string) => Promise<any>
  uploadSessionAdd: (sessionId: string, localPath: string, remotePath: string) => Promise<any>
  uploadSessionCheckConflicts: (sessionId: string) => Promise<any>
  uploadSessionCommit: (sessionId: string) => Promise<any>
  uploadSessionCancel: (sessionId: string) => Promise<any>
  dbRedisStreamInfo: (id: string, dbIndex: number, stream: string) => Promise<any>
  dbRedisStreams: (id: string, dbIndex: number) => Promise<any>
  dbRedisStreamAdd: (id: string, dbIndex: number, stream: string, fields: Record<string, unknown>) => Promise<any>
  dbRedisStreamMessages: (id: string, dbIndex: number, stream: string, start: string, end: string, count?: number) => Promise<any>
  dbRedisStreamDel: (id: string, dbIndex: number, stream: string, entryId: string) => Promise<any>
  dbRedisStreamDelete: (id: string, dbIndex: number, stream: string) => Promise<any>
  dbRedisStreamConsumers: (id: string, dbIndex: number, stream: string) => Promise<any>
  dbRedisStreamPending: (id: string, dbIndex: number, stream: string, group: string) => Promise<any>
  dbRedisStreamGroupCreate: (id: string, dbIndex: number, stream: string, group: string) => Promise<any>
  dbRedisStreamGroupDestroy: (id: string, dbIndex: number, stream: string, group: string) => Promise<any>
  dbRedisStreamClaim: (id: string, dbIndex: number, stream: string, group: string, consumer: string, entryId: string) => Promise<any>
  dbRedisStreamAck: (id: string, dbIndex: number, stream: string, group: string, entryId: string) => Promise<any>
  dbRedisStreamRetry: (id: string, dbIndex: number, stream: string, group: string, entryId: string) => Promise<any>
  dbRedisStreamTrim: (id: string, dbIndex: number, stream: string, maxLen: number) => Promise<any>
  dbRedisZSetRemove: (id: string, dbIndex: number, key: string, member: string) => Promise<any>
  dbRedisZSetRange: (id: string, dbIndex: number, key: string, start: number, stop: number) => Promise<any>
  logsStartStream: (params: Record<string, unknown>) => Promise<any>
  logsStopStream: (streamId: string) => Promise<any>
  logPresetsUpdate: (id: string, updates: Record<string, unknown>) => Promise<any>
  logPresetsAdd: (preset: Record<string, unknown>) => Promise<any>
  logPresetsDelete: (id: string) => Promise<any>
  generateTotp: (secret: string, digits?: number, period?: number, algorithm?: string) => Promise<any>
  parseOtpAuthUri: (uri: string) => Promise<any>
  apiRequestsUpdate: (id: string, updates: Record<string, unknown>) => Promise<any>
  apiRequestsAdd: (req: Record<string, unknown>) => Promise<any>
  apiRequestsDelete: (id: string) => Promise<any>
  apiRequestsGetAll: () => Promise<any>
  apiHttpRequest: (req: Record<string, unknown>) => Promise<any>
  assignTask: (todoId: string, userId: string) => Promise<any>
  broadcastTaskUpdate: (todo: Record<string, unknown>) => Promise<any>
  broadcastTaskComment: (todoId: string, comment: Record<string, unknown>) => Promise<any>
  getTodos: () => Promise<any>
  getTags: () => Promise<any>
  syncTaskStatus: (todoId: string, completed: boolean) => Promise<any>
  createRepeatInstance: (todoId: string) => Promise<any>
  gitSyncInit: () => Promise<any>
  gitSyncPull: () => Promise<any>
  gitSyncPush: () => Promise<any>
  gitSyncConfigure: (params: Record<string, unknown>) => Promise<any>
  checkBudgetAlerts: () => Promise<any>
  getTemplates: () => Promise<any>
  addTemplate: (template: Record<string, unknown>) => Promise<any>
  updateTemplate: (id: string, template: Record<string, unknown>) => Promise<any>
  deleteTemplate: (id: string) => Promise<any>
  useTemplate: (id: string) => Promise<any>
  uploadAccountingReceipt: (name: string, data: string) => Promise<any>
  getAccountingReceiptFile: (path: string) => Promise<any>
  exportAccountingCSV: (params?: Record<string, unknown>) => Promise<any>
  checkMavenAvailable: () => Promise<any>
  checkJavaAvailable: () => Promise<any>
  checkNodeAvailable: () => Promise<any>
  screenshot: () => Promise<any>
  exportWordReport: (params: Record<string, unknown>) => Promise<any>
  // Nginx
  getNginxPresets: () => Promise<any>
  addNginxPreset: (preset: any) => Promise<any>
  updateNginxPreset: (preset: any) => Promise<any>
  deleteNginxPreset: (id: string) => Promise<any>
  fetchNginxConfig: (serverId: string, configPath: string) => Promise<any>
  testNginxConfig: (serverId: string, configPath: string) => Promise<any>
  testNginxConfigContent: (serverId: string, configPath: string, content: string) => Promise<any>
  deployNginxConfig: (serverId: string, configPath: string, content: string, comment: string) => Promise<any>
  getNginxConfigVersions: (presetId: string) => Promise<any>
  saveNginxConfigVersion: (version: any) => Promise<any>
  setActiveNginxVersion: (presetId: string, versionId: string) => Promise<any>
  generateNginxConfig: (presetId: string) => Promise<any>
  generateNginxConfigDecomposed: (presetId: string) => Promise<any>
  previewNginxServer: (presetId: string, server: any, locations: any[]) => Promise<any>
  deployNginxConfigDecomposed: (serverId: string, configPath: string, mainContent: string, subFiles: Array<{filename: string, content: string}>, comment: string) => Promise<any>
  getServersByPreset: (presetId: string) => Promise<any>
  addNginxServer: (server: any) => Promise<any>
  updateNginxServer: (server: any) => Promise<any>
  deleteNginxServer: (id: string) => Promise<any>
  getLocationsByServer: (serverId: string) => Promise<any>
  addNginxLocation: (location: any) => Promise<any>
  updateNginxLocation: (location: any) => Promise<any>
  deleteNginxLocation: (id: string) => Promise<any>
  getUpstreamsByPreset: (presetId: string) => Promise<any>
  addNginxUpstream: (upstream: any) => Promise<any>
  updateNginxUpstream: (upstream: any) => Promise<any>
  deleteNginxUpstream: (id: string) => Promise<any>
  getUpstreamServers: (upstreamId: string) => Promise<any>
  addNginxUpstreamServer: (upstreamServer: any) => Promise<any>
  updateNginxUpstreamServer: (upstreamServer: any) => Promise<any>
  deleteNginxUpstreamServer: (id: string) => Promise<any>
  getHttpParamsByPreset: (presetId: string) => Promise<any>
  addNginxHttpParam: (param: any) => Promise<any>
  updateNginxHttpParam: (param: any) => Promise<any>
  deleteNginxHttpParam: (id: string) => Promise<any>
  getStreamsByPreset: (presetId: string) => Promise<any>
  addNginxStream: (stream: any) => Promise<any>
  updateNginxStream: (stream: any) => Promise<any>
  deleteNginxStream: (id: string) => Promise<any>
  getCertsByPreset: (presetId: string) => Promise<any>
  addNginxCert: (cert: any) => Promise<any>
  updateNginxCert: (cert: any) => Promise<any>
  deleteNginxCert: (id: string) => Promise<any>
  getTemplatesByPreset: (presetId: string) => Promise<any>
  addNginxTemplate: (template: any) => Promise<any>
  updateNginxTemplate: (template: any) => Promise<any>
  deleteNginxTemplate: (id: string) => Promise<any>
  getBasicSettings: (presetId: string) => Promise<any>
  saveBasicSettings: (presetId: string, settings: any[]) => Promise<any>
  getParamsByPreset: (presetId: string) => Promise<any>
  addNginxParam: (param: any) => Promise<any>
  updateNginxParam: (param: any) => Promise<any>
  deleteNginxParam: (id: string) => Promise<any>
  getDenyAllowsByPreset: (presetId: string) => Promise<any>
  addNginxDenyAllow: (denyAllow: any) => Promise<any>
  updateNginxDenyAllow: (denyAllow: any) => Promise<any>
  deleteNginxDenyAllow: (id: string) => Promise<any>
  getPasswordsByPreset: (presetId: string) => Promise<any>
  addNginxPassword: (password: any) => Promise<any>
  updateNginxPassword: (password: any) => Promise<any>
  deleteNginxPassword: (id: string) => Promise<any>
  importNginxConfig: (presetId: string, configText: string) => Promise<any>
  getNginxPresetStats: (presetId: string) => Promise<any>
  // Alert
  getEmailConfig: () => Promise<any>
  saveEmailConfig: (config: any) => Promise<any>
  testEmailConfig: (config: any) => Promise<any>
  getAlertServices: () => Promise<any>
  addAlertService: (service: any) => Promise<any>
  updateAlertService: (service: any) => Promise<any>
  deleteAlertService: (id: string) => Promise<any>
  getAlertResources: () => Promise<any>
  addAlertResource: (resource: any) => Promise<any>
  updateAlertResource: (resource: any) => Promise<any>
  deleteAlertResource: (id: string) => Promise<any>
  getAlertHistory: () => Promise<any>
  triggerAlertCheck: () => Promise<any>
  // Git Operations
  gitStatus: (repoPath: string) => Promise<any>
  gitCurrentBranch: (repoPath: string) => Promise<any>
  gitBranches: (repoPath: string) => Promise<any>
  gitLog: (repoPath: string, limit?: number) => Promise<any>
  gitDiff: (repoPath: string, file?: string) => Promise<any>
  gitAdd: (repoPath: string, files: string[]) => Promise<any>
  gitReset: (repoPath: string, file?: string) => Promise<any>
  gitCommit: (repoPath: string, message: string, files?: string[]) => Promise<any>
  gitCheckout: (repoPath: string, branch: string) => Promise<any>
  gitCreateBranch: (repoPath: string, branchName: string, from?: string) => Promise<any>
  gitDeleteBranch: (repoPath: string, branchName: string, force: boolean) => Promise<any>
  gitMerge: (repoPath: string, branch: string) => Promise<any>
  gitPull: (repoPath: string) => Promise<any>
  gitPush: (repoPath: string) => Promise<any>
  gitForcePush: (repoPath: string) => Promise<any>
  gitFetch: (repoPath: string, remote?: string) => Promise<any>
  gitRemotes: (repoPath: string) => Promise<any>
  gitDiscardChanges: (repoPath: string, file: string) => Promise<any>
  gitStashSave: (repoPath: string, message?: string, includeUntracked?: boolean, keepIndex?: boolean) => Promise<any>
  gitStashList: (repoPath: string) => Promise<any>
  gitStashApply: (repoPath: string, stashRef?: string) => Promise<any>
  gitStashPop: (repoPath: string, stashRef?: string) => Promise<any>
  gitStashDrop: (repoPath: string, stashRef?: string) => Promise<any>
  gitListTags: (repoPath: string) => Promise<any>
  gitCreateTag: (repoPath: string, tagName: string, message?: string, force?: boolean) => Promise<any>
  gitDeleteTag: (repoPath: string, tagName: string) => Promise<any>
  gitRebase: (repoPath: string, targetBranch: string, onto?: string) => Promise<any>
  gitRebaseAbort: (repoPath: string) => Promise<any>
  gitRebaseContinue: (repoPath: string) => Promise<any>
  gitRebaseInteractive: (repoPath: string, baseCommit: string, operations: any[]) => Promise<any>
  gitRebaseTodoList: (repoPath: string, baseCommit: string) => Promise<any>
  gitFileHistory: (repoPath: string, filePath: string, limit?: number) => Promise<any>
  gitUnpushedCommits: (repoPath: string) => Promise<any>
  gitCherryPick: (repoPath: string, commitHash: string, noCommit?: boolean) => Promise<any>
  gitRevert: (repoPath: string, commitHash: string, noCommit?: boolean) => Promise<any>
  gitAmendCommit: (repoPath: string, message: string) => Promise<any>
  gitResetToCommit: (repoPath: string, commitHash: string, mode: string) => Promise<any>
  gitFileBlame: (repoPath: string, filePath: string) => Promise<any>
  gitSubmoduleList: (repoPath: string) => Promise<any>
  gitSubmoduleInit: (repoPath: string, recursive: boolean) => Promise<any>
  gitAddRemote: (repoPath: string, name: string, url: string) => Promise<any>
  gitDeleteRemote: (repoPath: string, name: string) => Promise<any>
  gitRenameBranch: (repoPath: string, oldName: string, newName: string) => Promise<any>
  gitDiffBranches: (repoPath: string, target: string) => Promise<any>
  gitPushTags: (repoPath: string) => Promise<any>
  gitClean: (repoPath: string, dryRun: boolean, force: boolean, includeIgnored: boolean, directories: boolean) => Promise<any>
  gitDeleteRemoteBranch: (repoPath: string, branch: string) => Promise<any>
  gitCheckoutRemoteBranch: (repoPath: string, branch: string) => Promise<any>
  gitGetFileAtRevision: (repoPath: string, commit: string, path: string) => Promise<string>
  // Git 高级操作
  gitSubmoduleUpdate: (repoPath: string, submodulePath: string, recursive: boolean) => Promise<any>
  gitSubmoduleUpdateAll: (repoPath: string, recursive: boolean) => Promise<any>
  gitCompareCommits: (repoPath: string, commit1: string, commit2: string) => Promise<any>
  gitCreatePatch: (repoPath: string, commit1: string, commit2: string) => Promise<any>
  gitApplyPatch: (repoPath: string, patchContent: string) => Promise<any>
  gitRawCommand: (repoPath: string, args: string[]) => Promise<any>

  // Hermes Tools
  listToolsets: () => Promise<ToolsetInfo[]>
  setToolsetEnabled: (key: string, enabled: boolean) => Promise<void>
  listMcpServers: () => Promise<MCPServerInfo[]>

  // Hermes Config (Agent Settings)
  getHermesConfigInfo: () => Promise<HermesConfigInfo>
  exportHermesConfig: () => Promise<ConfigExportResult>
  importHermesConfig: (content: string) => Promise<ConfigImportResult>

  // Hermes Memory
  readMemory: () => Promise<MemoryInfo>
  addMemoryEntry: (content: string) => Promise<MemoryWriteResult>
  updateMemoryEntry: (index: number, content: string) => Promise<MemoryWriteResult>
  removeMemoryEntry: (index: number) => Promise<MemoryWriteResult>
  writeUserProfile: (content: string) => Promise<MemoryWriteResult>
  listMemoryProviders: () => Promise<MemoryProviderResult>
  setMemoryProvider: (provider: string) => Promise<MemoryWriteResult>
  readEnvVars: (keys: string[]) => Promise<Record<string, string>>
  saveEnvVar: (key: string, value: string) => Promise<MemoryWriteResult>

  // Hermes Provider
  listProviders: () => Promise<ProviderListResult>
  saveProviderCredential: (providerId: string, apiKey: string) => Promise<ProviderSaveResult>
  removeProviderCredential: (providerId: string) => Promise<ProviderSaveResult>
  startOauthFlow: (providerId: string) => Promise<ProviderSaveResult>
  pollOauthResult: (providerId: string) => Promise<ProviderSaveResult>

  // Hermes Skills
  listInstalledSkills: () => Promise<SkillInfo[]>
  listBundledSkills: () => Promise<SkillInfo[]>
  getSkillContent: (path: string) => Promise<string>
  installSkill: (identifier: string) => Promise<SkillCliResult>
  uninstallSkill: (name: string) => Promise<SkillCliResult>

  // Hermes Cron Jobs
  listCronJobs: () => Promise<CronJob[]>
  createCronJob: (schedule: string, prompt?: string, name?: string, deliver?: string) => Promise<void>
  removeCronJob: (jobId: string) => Promise<void>
  pauseCronJob: (jobId: string) => Promise<void>
  resumeCronJob: (jobId: string) => Promise<void>
  triggerCronJob: (jobId: string) => Promise<void>

  // OMP Chat (ACP protocol)
  ompChatInit: (cwd?: string) => Promise<void>
  ompChatSend: (message: string) => Promise<void>
  ompChatClose: () => Promise<void>
  ompChatListSessions: () => Promise<unknown>
  ompChatInfo: () => Promise<{ binary: string }>
  ompReadModelsConfig: () => Promise<unknown>
  ompReadStats: () => Promise<{ sessions: number; messages: number }>
}

let cachedAPI: TauriAPI | null = null

export function getTauriAPI(): TauriAPI {
  if (cachedAPI) {return cachedAPI}

  const projects = useProjectsAPI()
  const servers = useServersAPI()
  const database = useDatabaseAPI()
  const todos = useTodosAPI()
  const notes = useNotesAPI()
  const weekly = useWeeklyAPI()
  const mfa = useMfaAPI()
  const accounting = useAccountingAPI()
  const logs = useLogsAPI()
  const settings = useSettingsAPI()
  const dataBackup = useDataBackupAPI()
  const openvpn = useOpenVPNAPI()
  const wireguard = useWireGuardAPI()
  const nginx = useNginxAPI()
  const alert = useAlertAPI()
  const lan = useLanAPI()

  cachedAPI = {
    // Projects
    getServers: projects.getAllServers,
    getProjects: projects.getProjects,
    addProject: projects.addProject,
    updateProject: projects.updateProject,
    deleteProject: projects.deleteProject,
    getProjectStats: projects.getProjectStats,
    getProjectTodos: projects.getProjectTodos,
    // Servers
    getAllServers: servers.getAllServers,
    getServerById: servers.getServerById,
    addServer: servers.addServer,
    updateServer: servers.updateServer,
    deleteServer: servers.deleteServer,
    getAllServerGroups: servers.getAllServerGroups,
    getServerGroups: servers.getAllServerGroups,
    addServerGroup: servers.addServerGroup,
    updateServerGroup: servers.updateServerGroup,
    deleteServerGroup: servers.deleteServerGroup,
    testServerConnection: servers.testConnection,
    // Database
    dbConnect: database.dbConnect,
    dbDisconnect: database.dbDisconnect,
    dbQuery: database.dbQuery,
    dbGetTables: database.getTables,
    dbGetDatabases: database.getDatabases,
    dbGetTableStructure: database.dbGetTableStructure,
    dbGetTablePrimaryKeys: database.dbGetTablePrimaryKeys,
    dbGetViews: database.dbGetViews,
    dbGetCreateSql: database.dbGetCreateSql,
    dbCompareStructures: database.dbCompareStructures,
    dbExecuteStructureSync: database.dbExecuteStructureSync,
    dbCompareData: database.dbCompareData,
    dbExecuteDataSync: database.dbExecuteDataSync,
    dbBackupCreate: database.dbBackupCreate,
    dbBackupList: database.dbBackupList,
    dbBackupRestore: database.dbBackupRestore,
    dbBackupDelete: database.dbBackupDelete,
    dbRedisDatabases: database.dbRedisDatabases,
    dbRedisKeysTree: database.dbRedisKeysTree,
    dbRedisKeysByType: database.dbRedisKeysByType,
    dbRedisKeyInfo: database.dbRedisKeyInfo,
    dbRedisKeyValue: database.dbRedisKeyValue,
    dbRedisSetKey: database.dbRedisSetKey,
    dbRedisAddKey: database.dbRedisAddKey,
    dbRedisDeleteKey: database.dbRedisDeleteKey,
    dbRedisExec: database.dbRedisExec,
    detectToolPaths: database.detectToolPaths,
    detectBuildTools: database.detectBuildTools,
    detectSdkVersions: database.detectSdkVersions,
    getCicdConfigs: database.getCicdConfigs,
    getCicdGroups: database.getCicdGroups,
    getCicdConfigById: async (id: string) => {
      const res = await tauriInvoke<any>('get_cicd_config_by_id', { id })
      return res.success ? res.data : null
    },
    addCicdConfig: async (config: Record<string, unknown>) => {
      const res = await tauriInvoke<any>('save_cicd_config', { config })
      return res.success ? res.data : { success: false, error: res.error }
    },
    updateCicdConfig: async (config: Record<string, unknown>) => {
      const res = await tauriInvoke<any>('save_cicd_config', { config })
      return res.success ? res.data : { success: false, error: res.error }
    },
    deleteCicdConfig: async (id: string) => {
      const res = await tauriInvoke<any>('delete_cicd_config', { id })
      return res.success ? res.data : { success: false, error: res.error }
    },
    getDeployModules: async (configId: string) => {
      const res = await tauriInvoke<any[]>('get_deploy_modules', { configId })
      return res.success ? (res.data ?? []) : []
    },
    addDeployModule: async (module: Record<string, unknown>) => {
      const res = await tauriInvoke<any>('save_deploy_module', { module })
      return res.success ? res.data : { success: false, error: res.error }
    },
    updateDeployModule: async (module: Record<string, unknown>) => {
      const res = await tauriInvoke<any>('update_deploy_module', { module })
      return res.success ? res.data : { success: false, error: res.error }
    },
    deleteDeployModule: async (moduleId: string) => {
      const res = await tauriInvoke<any>('delete_deploy_module', { module_id: moduleId })
      return res.success ? res.data : { success: false, error: res.error }
    },
    scanProject: async (localPath: string) => {
      const res = await tauriInvoke<any>('scan_project', { localPath })
      return res.success ? res.data : {}
    },
    scanProjectModules: async (projectPath: string) => {
      const res = await tauriInvoke<any>('scan_project_modules', { projectPath })
      return res.success ? res.data : { success: false, modules: [], error: '扫描失败' }
    },
    testSsh: async (config: Record<string, unknown>) => {
      const res = await tauriInvoke<any>('test_connection', config)
      return res.success ? res.data : { success: false, error: res.error }
    },
    checkNetworkPermission: database.checkNetworkPermission,
    // Todos
    getAllTodos: todos.getAllTodos,
    addTodo: todos.addTodo,
    updateTodo: todos.updateTodo,
    deleteTodo: todos.deleteTodo,
    getAllTags: todos.getAllTags,
    addTag: todos.addTag,
    deleteTag: todos.deleteTag,
    addSubtask: todos.addSubtask,
    updateSubtask: todos.updateSubtask,
    deleteSubtask: todos.deleteSubtask,
    getSubtasksForTodo: todos.getSubtasksForTodo,
    // Notes
    getAllNotes: notes.getAllNotes,
    addNote: notes.addNote,
    updateNote: notes.updateNote,
    deleteNote: notes.deleteNote,
    getAllNoteGroups: notes.getAllNoteGroups,
    getNoteGroups: notes.getAllNoteGroups,
    addNoteGroup: notes.addNoteGroup,
    updateNoteGroup: notes.updateNoteGroup,
    deleteNoteGroup: notes.deleteNoteGroup,
    // Weekly
    getWeeklyReports: weekly.getWeeklyReports,
    getWeeklyReport: weekly.getWeeklyReport,
    saveWeeklyReport: weekly.saveWeeklyReport,
    // MFA
    getAllMfaSecrets: mfa.getAllMfaSecrets,
    getMfaSecrets: mfa.getAllMfaSecrets,
    addMfaSecret: mfa.addMfaSecret,
    updateMfaSecret: mfa.updateMfaSecret,
    deleteMfaSecret: mfa.deleteMfaSecret,
    generateTotp: mfa.generateTotp,
    // Accounting
    getAccountingRecords: accounting.getAccountingRecords,
    addAccountingRecord: accounting.addAccountingRecord,
    updateAccountingRecord: accounting.updateAccountingRecord,
    deleteAccountingRecord: accounting.deleteAccountingRecord,
    getAccountingCategories: accounting.getAccountingCategories,
    addAccountingCategory: accounting.addAccountingCategory,
    updateAccountingCategory: accounting.updateAccountingCategory,
    deleteAccountingCategory: accounting.deleteAccountingCategory,
    getAccountingStats: accounting.getAccountingStats,
    getAccountingTrend: accounting.getAccountingTrend,
    getBudgets: accounting.getBudgets,
    addBudget: accounting.addBudget,
    updateBudget: accounting.updateBudget,
    deleteBudget: accounting.deleteBudget,
    checkBudgetAlerts: accounting.checkBudgetAlerts,
    getTemplates: accounting.getTemplates,
    addTemplate: accounting.addTemplate,
    updateTemplate: accounting.updateTemplate,
    deleteTemplate: accounting.deleteTemplate,
    useTemplate: accounting.useTemplate,
    uploadAccountingReceipt: accounting.uploadAccountingReceipt,
    getAccountingReceiptFile: accounting.getAccountingReceiptFile,
    exportAccountingCSV: accounting.exportAccountingCSV,
    // Logs
    getLogPresets: logs.getLogPresets,
    logPresetsGetAll: logs.getLogPresets,
    addLogPreset: logs.addLogPreset,
    updateLogPreset: logs.updateLogPreset,
    deleteLogPreset: logs.deleteLogPreset,
    logSearch: async (params: Record<string, unknown>) => tauriCall("log_search", {
      presetId: (params as any).preset_id || (params as any).presetId,
      keyword: (params as any).keyword || (params as any).query,
      lines: (params as any).lines ?? 50
    }),
    logTail: logs.logTail,
    // Settings
    getMenuIcon: settings.getMenuIcon,
    getSetting: settings.getSetting,
    setSetting: settings.setSetting,
    getNotificationSettings: settings.getNotificationSettings,
    setNotificationSettings: settings.setNotificationSettings,
    // App
    getAppVersion: async (): Promise<string> => {
      try {
        const res = await tauriInvoke<string>('get_app_version')
        return res.success ? (res.data ?? __APP_VERSION__ ?? '1.0.0') : (__APP_VERSION__ ?? '1.0.0')
      } catch {
        return __APP_VERSION__ ?? '1.0.0'
      }
    },
    exportAllData: dataBackup.exportAllData,
    exportData: dataBackup.exportData,
    importJson: dataBackup.importJson,
    exportCsv: dataBackup.exportCsv,
    importAllData: dataBackup.importAllData,
    setAutoBackup: dataBackup.setAutoBackup,
    getAppPath: dataBackup.getAppPath,
    getDataDir: dataBackup.getDataDir,
    setDataDir: dataBackup.setDataDir,
    // OpenVPN
    openvpnGetAll: openvpn.openvpnGetAll,
    openvpnAdd: openvpn.openvpnAdd,
    openvpnDelete: openvpn.openvpnDelete,
    openvpnConnect: openvpn.openvpnConnect,
    openvpnRetryWithPassword: openvpn.openvpnRetryWithPassword,
    openvpnDisconnect: openvpn.openvpnDisconnect,
    openvpnGetStatus: openvpn.openvpnGetStatus,
    openvpnGetLogs: openvpn.openvpnGetLogs,
    openvpnCheckAvailable: openvpn.openvpnCheckAvailable,
    openvpnValidateConfig: openvpn.openvpnValidateConfig,
    openvpnGetTrafficStats: openvpn.openvpnGetTrafficStats,
    // WireGuard
    wireguardGetAll: wireguard.wireguardGetAll,
    wireguardGetById: wireguard.wireguardGetById,
    wireguardAdd: wireguard.wireguardAdd,
    wireguardUpdate: wireguard.wireguardUpdate,
    wireguardDelete: wireguard.wireguardDelete,
    wireguardConnect: wireguard.wireguardConnect,
    wireguardDisconnect: wireguard.wireguardDisconnect,
    wireguardGetStatus: wireguard.wireguardGetStatus,
    wireguardGenerateKeypair: wireguard.wireguardGenerateKeypair,
    wireguardDerivePublicKey: wireguard.wireguardDerivePublicKey,
    // Nginx
    getNginxPresets: nginx.getNginxPresets,
    addNginxPreset: nginx.addNginxPreset,
    updateNginxPreset: nginx.updateNginxPreset,
    deleteNginxPreset: nginx.deleteNginxPreset,
    fetchNginxConfig: nginx.fetchNginxConfig,
    testNginxConfig: nginx.testNginxConfig,
    testNginxConfigContent: nginx.testNginxConfigContent,
    deployNginxConfig: nginx.deployNginxConfig,
    getNginxConfigVersions: nginx.getNginxConfigVersions,
    saveNginxConfigVersion: nginx.saveNginxConfigVersion,
    setActiveNginxVersion: nginx.setActiveNginxVersion,
    generateNginxConfig: nginx.generateNginxConfig,
    generateNginxConfigDecomposed: nginx.generateNginxConfigDecomposed,
    previewNginxServer: nginx.previewNginxServer,
    deployNginxConfigDecomposed: nginx.deployNginxConfigDecomposed,
    getServersByPreset: nginx.getServersByPreset,
    addNginxServer: nginx.addNginxServer,
    updateNginxServer: nginx.updateNginxServer,
    deleteNginxServer: nginx.deleteNginxServer,
    getLocationsByServer: nginx.getLocationsByServer,
    addNginxLocation: nginx.addNginxLocation,
    updateNginxLocation: nginx.updateNginxLocation,
    deleteNginxLocation: nginx.deleteNginxLocation,
    getUpstreamsByPreset: nginx.getUpstreamsByPreset,
    addNginxUpstream: nginx.addNginxUpstream,
    updateNginxUpstream: nginx.updateNginxUpstream,
    deleteNginxUpstream: nginx.deleteNginxUpstream,
    getUpstreamServers: nginx.getUpstreamServers,
    addNginxUpstreamServer: nginx.addNginxUpstreamServer,
    updateNginxUpstreamServer: nginx.updateNginxUpstreamServer,
    deleteNginxUpstreamServer: nginx.deleteNginxUpstreamServer,
    getHttpParamsByPreset: nginx.getHttpParamsByPreset,
    addNginxHttpParam: nginx.addNginxHttpParam,
    updateNginxHttpParam: nginx.updateNginxHttpParam,
    deleteNginxHttpParam: nginx.deleteNginxHttpParam,
    getStreamsByPreset: nginx.getStreamsByPreset,
    addNginxStream: nginx.addNginxStream,
    updateNginxStream: nginx.updateNginxStream,
    deleteNginxStream: nginx.deleteNginxStream,
    getCertsByPreset: nginx.getCertsByPreset,
    addNginxCert: nginx.addNginxCert,
    updateNginxCert: nginx.updateNginxCert,
    deleteNginxCert: nginx.deleteNginxCert,
    getTemplatesByPreset: nginx.getTemplatesByPreset,
    addNginxTemplate: nginx.addNginxTemplate,
    updateNginxTemplate: nginx.updateNginxTemplate,
    deleteNginxTemplate: nginx.deleteNginxTemplate,
    getBasicSettings: nginx.getBasicSettings,
    saveBasicSettings: nginx.saveBasicSettings,
    getParamsByPreset: nginx.getParamsByPreset,
    addNginxParam: nginx.addNginxParam,
    updateNginxParam: nginx.updateNginxParam,
    deleteNginxParam: nginx.deleteNginxParam,
    getDenyAllowsByPreset: nginx.getDenyAllowsByPreset,
    addNginxDenyAllow: nginx.addNginxDenyAllow,
    updateNginxDenyAllow: nginx.updateNginxDenyAllow,
    deleteNginxDenyAllow: nginx.deleteNginxDenyAllow,
    getPasswordsByPreset: nginx.getPasswordsByPreset,
    addNginxPassword: nginx.addNginxPassword,
    updateNginxPassword: nginx.updateNginxPassword,
    deleteNginxPassword: nginx.deleteNginxPassword,
    importNginxConfig: nginx.importNginxConfig,
    getNginxPresetStats: nginx.getNginxPresetStats,
    // Alert
    getEmailConfig: alert.getEmailConfig,
    saveEmailConfig: alert.saveEmailConfig,
    testEmailConfig: alert.testEmailConfig,
    getAlertServices: alert.getAlertServices,
    addAlertService: alert.addAlertService,
    updateAlertService: alert.updateAlertService,
    deleteAlertService: alert.deleteAlertService,
    getAlertResources: alert.getAlertResources,
    addAlertResource: alert.addAlertResource,
    updateAlertResource: alert.updateAlertResource,
    deleteAlertResource: alert.deleteAlertResource,
    getAlertHistory: alert.getAlertHistory,
    triggerAlertCheck: alert.triggerAlertCheck,
    // Events
    onTaskUpdated: (callback: (data: any) => void) => { return () => {} },
    onTaskStatusChanged: (_callback: (data: any) => void) => { return () => {} },
    gitSyncStatus: async (): Promise<any> => {
      const res = await tauriInvoke<any>('git_sync_status')
      // Rust returns flat { enabled, remote_url, branch, ... }, not { success, data }
      return (res as any)?.remote_url !== undefined ? res : { enabled: false, status: 'error', error: res.error || 'unknown' }
    },
    gitSyncConfigure: async (params: Record<string, unknown>): Promise<any> => {
      // Convert frontend camelCase to Rust snake_case
      const rustParams: Record<string, unknown> = {}
      if (params.enabled !== undefined) {rustParams.enabled = params.enabled === 'true' || params.enabled === true}
      if (params.remote_url) {rustParams.remote_url = params.remote_url}
      else if (params.remoteUrl) {rustParams.remote_url = params.remoteUrl}
      if (params.branch) {rustParams.branch = params.branch}
      if (params.interval !== undefined) {rustParams.interval = Number(params.interval) || 30}
      if (params.ssh_key) {rustParams.ssh_key = params.ssh_key}
      else if (params.ssh_key_path) {rustParams.ssh_key = params.ssh_key_path}
      else if (params.sshKey) {rustParams.ssh_key = params.sshKey}
      const res = await tauriInvoke<any>('git_sync_configure', { params: rustParams })
      if (!res.success) {throw new Error(res.error)}
      return res.data || res
    },
    gitSyncInit: async (): Promise<any> => {
      const res = await tauriInvoke<any>('git_sync_init')
      if (!res.success) {throw new Error(res.error)}
      return res.data
    },
    gitSyncPull: async (): Promise<any> => {
      const res = await tauriInvoke<any>('git_sync_pull')
      if (!res.success) {throw new Error(res.error)}
      // Rust returns flat { success, output, last_sync }, not { success, data }
      return res
    },
    gitSyncPush: async (): Promise<any> => {
      const res = await tauriInvoke<any>('git_sync_push')
      if (!res.success) {throw new Error(res.error)}
      return res
    },
    // Git Repo Management
    getGitRepos: async (): Promise<any> => {
      const res = await tauriInvoke<any>('get_git_repos')
      return res.success ? (res.data ?? []) : []
    },
    addGitRepo: async (repo: Record<string, unknown>): Promise<any> => { return tauriInvoke('add_git_repo', { data: repo }); },
    updateGitRepo: async (id: number | string, repo: Record<string, unknown>): Promise<any> => { return tauriInvoke('update_git_repo', { id, data: repo }); },
    deleteGitRepo: async (id: number | string): Promise<any> => { return tauriInvoke('delete_git_repo', { id }); },
    validateGitRepoPath: async (path: string): Promise<any> => { return tauriCall('validate_repo_path', { path }); },
    showOpenDialogForDirs: async (): Promise<{ filePaths?: string[] }> => { 
      const selected = await open({ directory: true, multiple: false })
      return { filePaths: selected ? [selected] : [] }
    },
    showOpenDialog: async (options?: Record<string, unknown>): Promise<any> => {
      const filters = (options?.filters as any[])?.map((f: any) => ({
        name: f.name || 'Files',
        extensions: f.extensions || ['*']
      }));
      const filePaths = await open({
        multiple: options?.multiple as boolean || false,
        directory: options?.directory as boolean || false,
        filters: filters,
        title: options?.title as string || '选择文件',
      });
      return { success: true, filePaths: filePaths ? (Array.isArray(filePaths) ? filePaths : [filePaths]) : [] };
    },
    // Git
    getGitCommits: async (path: string, since?: string): Promise<any> => tauriCall('get_git_commits', { repoPath: path, since }),
    scanLocalGitRepos: async (directories: string[]): Promise<any> => tauriCall('scan_local_repos', { directories }),
    getGitBranches: async (path: string): Promise<any> => tauriCall('get_git_branches', { repoPath: path }),
    openInFileManager: async (path: string): Promise<any> => tauriCall('open_in_file_manager', { path }),
    getGitCommitDetail: async (repoPath: string, commitHash: string): Promise<any> => tauriCall('get_git_commit_detail', { repoPath, commitHash }),
    fetchPageContent: async (url: string): Promise<string> => tauriCall('fetch_page_content', { url }),
    convertHtmlToMd: async (html: string): Promise<string> => tauriCall('convert_html_to_md', { html }),
    // Subtask
    updateTodoCompletionBasedOnSubtasks: async (todoId: string): Promise<any> => { return tauriCall('update_todo_completion_based_on_subtasks', { todoId }); },
    // Project
    // Calculator
    getCalculatorHistory: async (_limit?: number): Promise<any> => { return []; },
    // DB
    dbTest: async (config: Record<string, unknown>): Promise<any> => {
      // 清理重复字段：前端 DBConfig 同时有 database/dbName 和 user/username
      const { database, dbName, user, username, ...rest } = config
      const normalized: Record<string, unknown> = {
        ...rest,
        type: config.type === 'postgresql' ? 'postgres' : config.type,
        username: username || user || '',
        dbName: dbName || database || undefined,
      }
      return tauriCall('db_test', { config: normalized })
    },
    dbGetTableDataFiltered: async (filter: Record<string, unknown>): Promise<any> => {
      // 后端期望 6 个独立参数: id, dbName, tableName, filtersJson, limit, offset
      return tauriCall('db_get_table_data_filtered', {
        id: (filter as any).connId,
        dbName: (filter as any).dbName || (filter as any).database || '',
        tableName: (filter as any).table,
        filtersJson: (filter as any).filters ?? {},
        limit: (filter as any).pageSize ?? (filter as any).limit ?? 100,
        offset: (filter as any).offset ?? 0,
        sortColumn: (filter as any).sortColumn ?? null,
        sortDir: (filter as any).sortDirection ?? null,
      })
    },
    dbGetTablesFiltered: async (filter: Record<string, unknown>): Promise<any> => {
      // 与 dbGetTableDataFiltered 共用同一后端接口
      return tauriCall('db_get_table_data_filtered', {
        id: (filter as any).connectionId,
        dbName: (filter as any).dbName || (filter as any).database || '',
        tableName: (filter as any).tableName,
        filtersJson: (filter as any).filters ?? {},
        limit: (filter as any).limit ?? 100,
        offset: (filter as any).offset ?? 0,
      })
    },
    dbUpdateTableRow: async (connId: string, table: string, oldRow: Record<string, unknown>, newRow: Record<string, unknown>, dbName?: string): Promise<any> => {
      return tauriCall('db_update_table_row', {
        id: connId,
        tableName: table,
        primaryKeyJson: oldRow,
        valuesJson: newRow,
        dbName: dbName || null,
      })
    },
    dbInsertTableRow: async (connId: string, table: string, row: Record<string, unknown>, dbName?: string): Promise<any> => {
      return tauriCall('db_insert_table_row', { id: connId, tableName: table, valuesJson: row, dbName: dbName || null })
    },
    dbDeleteTableRow: async (connId: string, table: string, row: Record<string, unknown>, dbName?: string): Promise<any> => {
      return tauriCall('db_delete_table_row', { id: connId, tableName: table, primaryKeyJson: row, dbName: dbName || null })
    },
    // Menu events — listen to Tauri native menu events
    onMenuNewTask: (callback: () => void) => { return listen('menu:new-task', () => callback()) as Promise<UnlistenFn> },
    onMenuExportMarkdown: (callback: () => void) => { return listen('menu:export-markdown', () => callback()) as Promise<UnlistenFn> },
    onMenuExportWord: (callback: () => void) => { return listen('menu:export-word', () => callback()) as Promise<UnlistenFn> },
    onMenuExportJson: (callback: () => void) => { return listen('menu:export-json', () => callback()) as Promise<UnlistenFn> },
    onMenuImportJson: (callback: () => void) => { return listen('menu:import-json', () => callback()) as Promise<UnlistenFn> },
    onMenuClearCompleted: (callback: () => void) => { return listen('menu:clear-completed', () => callback()) as Promise<UnlistenFn> },
    onMenuSearchTasks: (callback: () => void) => { return listen('menu:search-tasks', () => callback()) as Promise<UnlistenFn> },
    onMenuSelectAll: (callback: () => void) => { return listen('menu:select-all', () => callback()) as Promise<UnlistenFn> },
    onMenuDeleteSelected: (callback: () => void) => { return listen('menu:delete-selected', () => callback()) as Promise<UnlistenFn> },
    onMenuToggleComplete: (callback: () => void) => { return listen('menu:toggle-complete', () => callback()) as Promise<UnlistenFn> },
    onMenuSetPriority: (callback: (priority: string) => void) => { return listen('menu:set-priority', () => callback('')) as Promise<UnlistenFn> },
    onMenuSetTag: (callback: () => void) => { return listen('menu:set-tag', () => callback()) as Promise<UnlistenFn> },
    onMenuAbout: (callback: () => void) => { return listen('menu:about', () => callback()) as Promise<UnlistenFn> },
    onMenuShortcutsHelp: (callback: () => void) => { return listen('menu:shortcuts-help', () => callback()) as Promise<UnlistenFn> },
    onMenuNav: (callback: (view: string) => void) => { return listen('menu:nav', (e) => callback(e.payload as string)) as Promise<UnlistenFn> },
    onMenuToggleLanPanel: (callback: () => void) => { return listen('menu:toggle-lan-panel', () => callback()) as Promise<UnlistenFn> },
    onMenuToggleLocale: (callback: () => void) => { return listen('menu:toggle-locale', () => callback()) as Promise<UnlistenFn> },
    onMenuToggleTheme: (callback: () => void) => { return listen('menu:toggle-theme', () => callback()) as Promise<UnlistenFn> },
    onMenuSearch: (callback: () => void) => { return listen('menu:search', () => callback()) as Promise<UnlistenFn> },
    onMenuSwitchView: (callback: (view: string) => void) => { return listen('menu:switch-view', (e) => callback(e.payload as string)) as Promise<UnlistenFn> },
    onMenuCheckUpdate: (callback: () => void) => { return listen('menu:check-update', () => callback()) as Promise<UnlistenFn> },
    // Nav event
    onNav: (callback: (view: string) => void) => { return listen('menu:nav', (e) => callback(e.payload as string)) as Promise<UnlistenFn> },
    // Collaboration & other events
    onTaskCommentAdded: (callback: (data: any) => void) => { return listen('task-comment-added', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onTaskAssigned: (callback: (data: any) => void) => { return listen('task-assigned', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onDataChanged: (callback: (data: any) => void) => { return listen('data-changed', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onDeployProgress: (callback: (data: any) => void) => { return listen('deploy-progress', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onDeployNotification: (callback: (data: any) => void) => { return listen('deploy-notification', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onDeployLogIdCreated: (callback: (data: any) => void) => { return listen('deploy-log-id-created', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    // Log streaming events
    onLogsLine: (callback: (data: any) => void) => { return listen('logs:line', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onLogsServerEnd: (callback: (data: any) => void) => { return listen('logs:server-end', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onLogsError: (callback: (data: any) => void) => { return listen('logs:error', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onLogsStreamStopped: (callback: (data: any) => void) => { return listen('logs:stream-stopped', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onFileReceived: (callback: (data: any) => void) => { return listen('file-received', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onFileTransferCompleted: (callback: (data: any) => void) => { return listen('file-transfer-completed', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onFileTransferError: (callback: (data: any) => void) => { return listen('file-transfer-error', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onFileTransferProgress: (callback: (data: any) => void) => { return listen('file-transfer-progress', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onFileTransferStarted: (callback: (data: any) => void) => { return listen('file-transfer-started', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onGitSyncStatusUpdated: (callback: (data: any) => void) => { return listen('git-sync-status-updated', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onLanPeerDiscovered: (callback: (data: any) => void) => { return listen('lan-peer-discovered', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onLanPeerLost: (callback: (data: any) => void) => { return listen('lan-peer-lost', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onLanPeerAvatarUpdated: (callback: (data: any) => void) => { return listen('lan-peer-avatar-updated', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onMessage: (callback: (data: any) => void) => { return listen('message', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onServerConnected: (callback: (data: any) => void) => { return listen('server-connected', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onServerDisconnected: (callback: (data: any) => void) => { return listen('server-disconnected', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onServerHeartbeatFailed: (callback: (data: any) => void) => { return listen('server-heartbeat-failed', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onSftpDownloadProgress: (callback: (data: any) => void) => { return listen('sftp-download-progress', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onSftpUploadDone: (callback: (data: any) => void) => { return listen('sftp-upload-done', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onSftpUploadProgress: (callback: (data: any) => void) => { return listen('sftp-upload-progress', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onTerminalClose: (callback: (data: any) => void) => { return listen('terminal-close', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onTerminalData: (callback: (data: any) => void) => { return listen('terminal-data', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onAutoBackupCompleted: (callback: (data: any) => void) => { return listen('auto-backup-completed', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onCollaborationStarted: (callback: (data: any) => void) => { return listen('collaboration-started', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    onCollaborationEnded: (callback: (data: any) => void) => { return listen('collaboration-ended', (e) => callback(e.payload)) as Promise<UnlistenFn> },
    importOvpnFile: async (): Promise<any> => {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'OpenVPN', extensions: ['ovpn', 'conf'] }],
      })
      if (!selected) {return { canceled: true, filePaths: [] }}
      return { canceled: false, filePaths: [selected] }
    },
    // LAN Chat
    sendMessage: async (peerId: string, content: string): Promise<any> => { return tauriCall('lan_send_message', { peerId, content }); },
    sendFile: async (peerId: string, filePath: string, fileName: string): Promise<any> => { return tauriCall('lan_send_file', { peerId, filePath, fileName }); },
    getMessagesBetween: async (peerId: string): Promise<any> => { return tauriCall('lan_get_message_history', { peerId }); },
    markMessagesRead: async (peerId: string): Promise<any> => { return tauriCall('lan_mark_messages_read', { peerId }); },
    // CICD Tool Checks
    checkJavaAvailable: async (javaHome?: string): Promise<any> => { return tauriCall('check_java', { javaHome: javaHome || null }); },
    checkMavenAvailable: async (mavenHome?: string): Promise<any> => { return tauriCall('check_maven', { mavenHome: mavenHome || null }); },
    checkNodeAvailable: async (nodeHome?: string): Promise<any> => { return tauriCall('check_node', { nodeHome: nodeHome || null }); },
    // Todo Batch Operations
    deleteTodos: async (ids: string[]): Promise<any> => { return tauriCall('delete_many', { ids }); },
    updateTodoOrder: async (items: any[]): Promise<any> => { return tauriCall('update_order', { items }); },
    createRepeatInstance: async (todoId: string): Promise<any> => { return tauriCall('create_repeat_instance', { todoId }); },
    // SFTP Operations
    uploadFile: async (serverId: string, remotePath: string, localPath: string): Promise<any> => { return tauriCall('sftp_upload_file', { serverId, remotePath, localPath }); },
    downloadFile: async (serverId: string, remotePath: string, localPath: string): Promise<any> => { return tauriCall('sftp_download_file', { serverId, remotePath, localPath }); },
    uploadFolder: async (serverId: string, remotePath: string, localPath: string): Promise<any> => { return tauriCall('sftp_upload_folder', { serverId, remotePath, localPath }); },
    getDownloadsDir: async (): Promise<any> => { return tauriCall('sftp_get_downloads_dir'); },
    // MFA
    parseOtpAuthUri: async (uri: string): Promise<any> => { return tauriCall('mfa_parse_uri', { uri }); },
    // Notification
    notificationTest: async (): Promise<any> => { return tauriCall('notification_test'); },
    playSound: async (): Promise<any> => { return tauriCall('play_sound'); },
    // Log
    readLogFile: async (filePath: string): Promise<any> => { return tauriCall('read_log_file', { filePath }); },
    // LAN Chat
    startLan: async (userId: string, userName: string) => tauriCall("lan_start", { userId, userName }),
    getUserInfo: async (userId: string) => tauriCall("lan_get_user_info"),
    setStatus: async (status: string) => tauriCall("lan_set_status", { status }),
    refreshDiscovery: async () => tauriCall("lan_refresh_discovery"),
    getAllUnreadCounts: async (userId: string) => tauriCall("lan_get_all_unread_counts", { userId }),
    getStatus: async (userId: string) => tauriCall("lan_get_status"),
    getNetworkInfo: async () => tauriCall("lan_get_network_info"),
    getReceivePath: async () => {
      const res = await tauriCall("lan_get_receive_path")
      return (res as any)?.data as string || res as string || ''
    },
    getPeers: async () => tauriCall("lan_get_peers"),
    setNickName: async (name: string) => tauriCall("lan_set_nick_name", { name }),
    setAvatar: async (avatar: string) => tauriCall("lan_set_avatar", { avatar }),
    setReceivePath: async (path: string) => tauriCall("lan_set_receive_path", { path }),
    openFile: async (filePath: string) => tauriCall("lan_open_file", { filePath }),
    openFileFolder: async (filePath: string) => tauriCall("lan_open_file_folder", { filePath }),
    saveTempFile: async (base64Data: string, fileName: string): Promise<string | null> => {
      const res = await tauriCall<any>('save_temp_file', { base64Data, fileName });
      return res?.path ?? null;
    },
    loadLocalFileAsBase64: async (filePath: string) => tauriCall("lan_load_local_file_as_base64", { filePath }),
    // SSH Terminal
    connectServer: async (serverId: string) => tauriCall("connect_server", { serverId }),
    disconnectServer: async (serverId: string) => tauriCall("disconnect_server", { serverId }),
    isServerConnected: async (serverId: string) => tauriCall("is_server_connected", { serverId }),
    createTerminal: async (serverId: string, terminalId: string, rows?: number, cols?: number) => tauriCall("create_terminal", { serverId, terminalId, rows, cols }),
    writeTerminal: async (terminalId: string, data: string) => tauriCall("write_to_terminal", { terminalId, data }),
    resizeTerminal: async (terminalId: string, cols: number, rows: number) => tauriCall("resize_terminal", { terminalId, cols, rows }),
    closeTerminal: async (terminalId: string) => tauriCall("close_terminal", { terminalId }),
    readTerminal: async (terminalId: string) => tauriCall("read_terminal", { terminalId }, true),
    isTerminalActive: async (terminalId: string) => tauriCall("is_terminal_active", { terminalId }),
    // Deploy
    deploy: async (configId: string, confirmed?: boolean) => tauriCall("deploy", { configId, confirmed }),
    cancelDeploy: async (deployLogId: string) => tauriCall("cancel_deploy", { deployLogId }),
    rollback: async (configId: string, logId: string) => tauriCall("rollback", { configId, logId }),
    getDeployLogs: async (configId: string, limit?: number) => tauriCall("get_deploy_logs", { configId, limit }),
    getDeployStepLogs: async (deployLogId: string) => tauriCall("get_deploy_step_logs", { deployLogId }),
    getDeployHistory: async (projectId: string, limit?: number) => tauriCall("get_deploy_history", { projectId, limit }),
    getRollbackHistory: async (configId: string) => tauriCall("get_rollback_history", { configId }),
    writeLogFile: async (logId: string, content: string) => tauriCall("write_log_file", { logId, content }),
    writeSystemLog: async (level: string, prefix: string, message: string) => {
      await tauriInvoke("write_system_log", { level, prefix, message }, true)
    },
    // SFTP
    getServerMonitor: async (serverId: string, commands: string[]) => tauriCall("get_server_monitor", { serverId, commands }),
    listSftpDir: async (serverId: string, path: string) => tauriCall("list_sftp_dir", { serverId, path }),
    openSftpFileEditor: async (serverId: string, filePath: string) => tauriCall("open_sftp_file_editor", { serverId, filePath }),
    deleteSftpFile: async (serverId: string, filePath: string, isDir = false) => tauriCall("delete_sftp_file", { serverId, filePath, isDir }),
    sftpCreateDir: async (serverId: string, path: string) => tauriCall("sftp_create_dir", { serverId, path }),
    uploadSessionStart: async (serverId: string, remotePath: string) => tauriCall("upload_session_start", { serverId, remotePath }),
    uploadSessionAdd: async (sessionId: string, localPath: string, remotePath: string) => tauriCall("upload_session_add", { sessionId, localPath, remotePath }),
    uploadSessionCheckConflicts: async (sessionId: string) => tauriCall("upload_session_check_conflicts", { sessionId }),
    uploadSessionCommit: async (sessionId: string) => tauriCall("upload_session_commit", { sessionId }),
    uploadSessionCancel: async (sessionId: string) => tauriCall("upload_session_cancel", { sessionId }),
    // Redis Stream
    dbRedisStreamInfo: async (id: string, dbIndex: number, stream: string) => tauriCall("db_redis_stream_info", { connId: id, dbIndex, stream }),
    dbRedisStreams: async (id: string, dbIndex: number) => tauriCall("db_redis_streams", { connId: id, dbIndex }),
    dbRedisStreamAdd: async (id: string, dbIndex: number, stream: string, fields: Record<string, unknown>) => tauriCall("db_redis_stream_add", { connId: id, dbIndex, stream, fields }),
    dbRedisStreamMessages: async (id: string, dbIndex: number, stream: string, start: string, end: string, count?: number) => tauriCall("db_redis_stream_messages", { connId: id, dbIndex, stream, start, end, count }),
    dbRedisStreamDel: async (id: string, dbIndex: number, stream: string, entryId: string) => tauriCall("db_redis_stream_del", { connId: id, dbIndex, stream, entryId }),
    dbRedisStreamDelete: async (id: string, dbIndex: number, stream: string) => tauriCall("db_redis_stream_delete", { connId: id, dbIndex, stream }),
    dbRedisStreamConsumers: async (id: string, dbIndex: number, stream: string) => tauriCall("db_redis_stream_consumers", { connId: id, dbIndex, stream }),
    dbRedisStreamPending: async (id: string, dbIndex: number, stream: string, group: string) => tauriCall("db_redis_stream_pending", { connId: id, dbIndex, stream, group }),
    dbRedisStreamGroupCreate: async (id: string, dbIndex: number, stream: string, group: string) => tauriCall("db_redis_stream_group_create", { connId: id, dbIndex, stream, group }),
    dbRedisStreamGroupDestroy: async (id: string, dbIndex: number, stream: string, group: string) => tauriCall("db_redis_stream_group_destroy", { connId: id, dbIndex, stream, group }),
    dbRedisStreamClaim: async (id: string, dbIndex: number, stream: string, group: string, consumer: string, entryId: string) => tauriCall("db_redis_stream_claim", { connId: id, dbIndex, stream, group, consumer, entryId }),
    dbRedisStreamAck: async (id: string, dbIndex: number, stream: string, group: string, entryId: string) => tauriCall("db_redis_stream_ack", { connId: id, dbIndex, stream, group, entryId }),
    dbRedisStreamRetry: async (id: string, dbIndex: number, stream: string, group: string, entryId: string) => tauriCall("db_redis_stream_retry", { connId: id, dbIndex, stream, group, entryId }),
    dbRedisStreamTrim: async (id: string, dbIndex: number, stream: string, maxLen: number) => tauriCall("db_redis_stream_trim", { connId: id, dbIndex, stream, maxLen }),
    dbRedisZSetRemove: async (id: string, dbIndex: number, key: string, member: string) => tauriCall("db_redis_zset_remove", { connId: id, dbIndex, key, member }),
    dbRedisZSetRange: async (id: string, dbIndex: number, key: string, start: number, stop: number) => tauriCall("db_redis_zset_range", { connId: id, dbIndex, key, start, stop }),
    // Logs
    logsStartStream: async (params: Record<string, unknown>) => tauriCall("logs_start_stream", { params }),
    logsStopStream: async (streamId: string) => tauriCall("logs_stop_stream", { streamId }),
    logPresetsUpdate: async (id: string, updates: Record<string, unknown>) => tauriCall("update_log_preset", { id, updates }),
    logPresetsAdd: async (preset: Record<string, unknown>) => tauriCall("add_log_preset", { preset }),
    logPresetsDelete: async (id: string) => tauriCall("delete_log_preset", { id }),
    // MFA
    generateTotp: async (secret: string, digits?: number, period?: number, algorithm?: string) => 
      tauriCall("generate_totp", { secret, digits, period, algorithm }, true),
    // API Requests
    apiRequestsUpdate: async (id: string, updates: Record<string, unknown>) => tauriCall("api_requests_update", { id, updates }),
    apiRequestsAdd: async (req: Record<string, unknown>) => tauriCall("api_requests_add", { req }),
    apiRequestsDelete: async (id: string) => tauriCall("api_requests_delete", { id }),
    apiRequestsGetAll: async () => tauriCall("api_requests_get_all"),
    apiHttpRequest: async (req: Record<string, unknown>) => tauriCall("api_http_request", { req }),
    // Todo Collaboration
    assignTask: async (todoId: string, userId: string) => tauriCall("assign_task", { todoId, userId }),
    broadcastTaskUpdate: async (todo: Record<string, unknown>) => tauriCall("broadcast_task_update", { todo }),
    broadcastTaskComment: async (todoId: string, comment: Record<string, unknown>) => tauriCall("broadcast_task_comment", { todoId, comment }),
    getTodos: async () => tauriCall("get_todos"),
    getTags: async () => tauriCall("get_all_tags"),
    syncTaskStatus: async (todoId: string, completed: boolean) => tauriCall("sync_task_status", { todoId, completed }),
    // Misc
    screenshot: async () => tauriCall("screenshot"),
    exportWordReport: async (params: Record<string, unknown>) => tauriCall("export_word_report", { params }),
    // LAN
    ...lan,
    // Git Operations
    gitStatus: async (repoPath: string) => tauriCall('git_status', { repoPath }),
    gitCurrentBranch: async (repoPath: string) => tauriCall('git_current_branch', { repoPath }),
    gitBranches: async (repoPath: string) => tauriCall('git_branches', { repoPath }),
    gitLog: async (repoPath: string, limit?: number) => tauriCall('git_log', { repoPath, limit }),
    gitDiff: async (repoPath: string, file?: string) => tauriCall('git_diff', { repoPath, file }),
    gitAdd: async (repoPath: string, files: string[]) => tauriCall('git_add', { repoPath, files }),
    gitReset: async (repoPath: string, file?: string) => tauriCall('git_reset', { repoPath, file }),
    gitCommit: async (repoPath: string, message: string, files?: string[]) => tauriCall('git_commit', { repoPath, message, files }),
    gitCheckout: async (repoPath: string, branch: string) => tauriCall('git_checkout', { repoPath, branch }),
    gitCreateBranch: async (repoPath: string, branchName: string, from?: string) => tauriCall('git_create_branch', { repoPath, branchName, from }),
    gitDeleteBranch: async (repoPath: string, branchName: string, force: boolean) => tauriCall('git_delete_branch', { repoPath, branchName, force }),
    gitMerge: async (repoPath: string, branch: string) => tauriCall('git_merge', { repoPath, branch }),
    gitPull: async (repoPath: string) => tauriCall('git_pull', { repoPath }),
    gitPush: async (repoPath: string) => tauriCall('git_push', { repoPath }),
    gitForcePush: async (repoPath: string) => tauriCall('git_force_push', { repoPath }),
    gitFetch: async (repoPath: string, remote?: string) => tauriCall('git_fetch', { repoPath, remote }),
    gitRemotes: async (repoPath: string) => tauriCall('git_remotes', { repoPath }),
    gitDiscardChanges: async (repoPath: string, file: string) => tauriCall('git_discard_changes', { repoPath, file }),
    gitStashSave: async (repoPath: string, message?: string, includeUntracked?: boolean, keepIndex?: boolean) => 
      tauriCall('git_stash_save', { repoPath, message, includeUntracked, keepIndex }),
    gitStashList: async (repoPath: string) => tauriCall('git_stash_list', { repoPath }),
    gitStashApply: async (repoPath: string, stashRef?: string) => tauriCall('git_stash_apply', { repoPath, stashRef }),
    gitStashPop: async (repoPath: string, stashRef?: string) => tauriCall('git_stash_pop', { repoPath, stashRef }),
    gitStashDrop: async (repoPath: string, stashRef?: string) => tauriCall('git_stash_drop', { repoPath, stashRef }),
    gitListTags: async (repoPath: string) => tauriCall('git_list_tags', { repoPath }),
    gitCreateTag: async (repoPath: string, tagName: string, message?: string, force?: boolean) => 
      tauriCall('git_create_tag', { repoPath, tagName, message, force }),
    gitDeleteTag: async (repoPath: string, tagName: string) => tauriCall('git_delete_tag', { repoPath, tagName }),
    gitRebase: async (repoPath: string, targetBranch: string, onto?: string) => tauriCall('git_rebase', { repoPath, targetBranch, onto }),
    gitRebaseAbort: async (repoPath: string) => tauriCall('git_rebase_abort', { repoPath }),
    gitRebaseContinue: async (repoPath: string) => tauriCall('git_rebase_continue', { repoPath }),
    gitRebaseInteractive: async (repoPath: string, baseCommit: string, operations: any[]) => tauriCall('git_rebase_interactive', { repoPath, baseCommit, operations }),
    gitRebaseTodoList: async (repoPath: string, baseCommit: string) => tauriCall('git_rebase_todo_list', { repoPath, baseCommit }),
    gitFileHistory: async (repoPath: string, filePath: string, limit?: number) => tauriCall('git_file_history', { repoPath, filePath, limit }),
    gitUnpushedCommits: async (repoPath: string) => tauriCall('git_unpushed_commits', { repoPath }),
    gitCherryPick: async (repoPath: string, commitHash: string, noCommit?: boolean) => tauriCall('git_cherry_pick', { repoPath, commitHash, noCommit }),
    gitRevert: async (repoPath: string, commitHash: string, noCommit?: boolean) => tauriCall('git_revert', { repoPath, commitHash, noCommit }),
    gitAmendCommit: async (repoPath: string, message: string) => tauriCall('git_amend_commit', { repoPath, message }),
    gitResetToCommit: async (repoPath: string, commitHash: string, mode: string) => tauriCall('git_reset_to_commit', { repoPath, commitHash, mode }),
    gitFileBlame: async (repoPath: string, filePath: string) => tauriCall('git_file_blame', { repoPath, filePath }),
    gitSubmoduleList: async (repoPath: string) => tauriCall('git_submodule_list', { repoPath }),
    gitSubmoduleInit: async (repoPath: string, recursive: boolean) => tauriCall('git_submodule_init', { repoPath, recursive }),
    gitAddRemote: async (repoPath: string, name: string, url: string) => tauriCall('git_add_remote', { repoPath, name, url }),
    gitDeleteRemote: async (repoPath: string, name: string) => tauriCall('git_delete_remote', { repoPath, name }),
    gitRenameBranch: async (repoPath: string, oldName: string, newName: string) => tauriCall('git_rename_branch', { repoPath, oldName, newName }),
    gitDiffBranches: async (repoPath: string, target: string) => tauriCall('git_diff_branches', { repoPath, target }),
    gitPushTags: async (repoPath: string) => tauriCall('git_push_tags', { repoPath }),
    gitClean: async (repoPath: string, dryRun: boolean, force: boolean, includeIgnored: boolean, directories: boolean) => tauriCall('git_clean', { repoPath, dryRun, force, includeIgnored, directories }),
    gitDeleteRemoteBranch: async (repoPath: string, branch: string) => tauriCall('git_delete_remote_branch', { repoPath, branch }),
    gitCheckoutRemoteBranch: async (repoPath: string, branch: string) => tauriCall('git_checkout_remote_branch', { repoPath, branch }),
    gitGetFileAtRevision: async (repoPath: string, commit: string, path: string) => tauriCall<string>('git_get_file_at_revision', { repoPath, commit, path }),
    // Git 高级操作
    gitSubmoduleUpdate: async (repoPath: string, submodulePath: string, recursive: boolean) => tauriCall('git_submodule_update', { repoPath, submodulePath, recursive }),
    gitSubmoduleUpdateAll: async (repoPath: string, recursive: boolean) => tauriCall('git_submodule_update_all', { repoPath, recursive }),
    gitCompareCommits: async (repoPath: string, commit1: string, commit2: string) => tauriCall('git_compare_commits', { repoPath, commit1, commit2 }),
    gitCreatePatch: async (repoPath: string, commit1: string, commit2: string) => tauriCall('git_create_patch', { repoPath, commit1, commit2 }),
    gitApplyPatch: async (repoPath: string, patchContent: string) => tauriCall('git_apply_patch', { repoPath, patchContent }),
    gitRawCommand: async (repoPath: string, args: string[]) => tauriCall('git_raw_command', { repoPath, args }),
    getGitCommitDetail: async (repoPath: string, commitHash: string) => tauriCall('get_git_commit_detail', { repoPath, commitHash }),
    // 文件浏览
    getFileTree: async (repoPath: string, subdir?: string) => tauriCall<FileTreeEntry[]>('get_file_tree', { repoPath, subdir }),
    readFileContent: async (repoPath: string, filePath: string) => tauriCall<string>('read_file_content', { repoPath, filePath }),
    saveFileContent: async (repoPath: string, filePath: string, content: string) => tauriCall<void>('save_file_content', { repoPath, filePath, content }),

    // ============ Hermes Tools ============
    listToolsets: async () => tauriCall<ToolsetInfo[]>('list_toolsets'),
    setToolsetEnabled: async (key: string, enabled: boolean) => tauriCall<void>('set_toolset_enabled', { key, enabled }),
    listMcpServers: async () => tauriCall<MCPServerInfo[]>('list_mcp_servers'),

    // ============ Hermes Config Export/Import ============
    getHermesConfigInfo: async () => tauriCall<HermesConfigInfo>('get_hermes_config_info'),
    exportHermesConfig: async () => tauriCall<ConfigExportResult>('export_hermes_config'),
    importHermesConfig: async (content: string) => tauriCall<ConfigImportResult>('import_hermes_config', { content }),

    // ============ Hermes Memory ============
    readMemory: async () => tauriCall<MemoryInfo>('read_memory'),
    addMemoryEntry: async (content: string) => tauriCall<MemoryWriteResult>('add_memory_entry', { content }),
    updateMemoryEntry: async (index: number, content: string) => tauriCall<MemoryWriteResult>('update_memory_entry', { index, content }),
    removeMemoryEntry: async (index: number) => tauriCall<MemoryWriteResult>('remove_memory_entry', { index }),
    writeUserProfile: async (content: string) => tauriCall<MemoryWriteResult>('write_user_profile', { content }),
    listMemoryProviders: async () => tauriCall<MemoryProviderResult>('list_memory_providers'),
    setMemoryProvider: async (provider: string) => tauriCall<MemoryWriteResult>('set_memory_provider', { provider }),
    readEnvVars: async (keys: string[]) => tauriCall<Record<string, string>>('read_env_vars', { keys }),
    saveEnvVar: async (key: string, value: string) => tauriCall<MemoryWriteResult>('save_env_var', { key, value }),

    // ============ Hermes Skills ============
    listInstalledSkills: async () => tauriCall<SkillInfo[]>('list_installed_skills'),
    listBundledSkills: async () => tauriCall<SkillInfo[]>('list_bundled_skills'),
    getSkillContent: async (path: string) => tauriCall<string>('get_skill_content', { path }),
    installSkill: async (identifier: string) => tauriCall<SkillCliResult>('install_skill', { identifier }),
    uninstallSkill: async (name: string) => tauriCall<SkillCliResult>('uninstall_skill', { name }),

    // ============ Provider Credential Management ============
    listProviders: async () => tauriCall<ProviderListResult>('list_providers'),
    saveProviderCredential: async (providerId: string, apiKey: string) => tauriCall<ProviderSaveResult>('save_provider_credential', { providerId, apiKey }),
    removeProviderCredential: async (providerId: string) => tauriCall<ProviderSaveResult>('remove_provider_credential', { providerId }),
    startOAuthFlow: async (providerId: string) => tauriCall<OAuthFlowResult>('start_oauth_flow', { providerId }),
    // ============ Hermes Cron Jobs ============
    listCronJobs: async () => tauriCall<CronJob[]>('list_cron_jobs'),
    createCronJob: async (
      schedule: string, prompt: string, name?: string, deliver?: string,
    ) =>
      tauriCall<void>('create_cron_job', { schedule, prompt, name, deliver }),
    removeCronJob: async (jobId: string) => tauriCall<void>('remove_cron_job', { jobId }),
    pauseCronJob: async (jobId: string) => tauriCall<void>('pause_cron_job', { jobId }),
    resumeCronJob: async (jobId: string) => tauriCall<void>('resume_cron_job', { jobId }),
    triggerCronJob: async (jobId: string) => tauriCall<void>('trigger_cron_job', { jobId }),

    // ============ Hermes Gateway ============
    gatewayStatus: async () => tauriCall<GatewayStatus>('gateway_status'),
    gatewayStart: async () => tauriCall<GatewayResult>('gateway_start'),
    gatewayStop: async () => tauriCall<GatewayResult>('gateway_stop'),
    gatewayRestart: async () => tauriCall<GatewayResult>('gateway_restart'),

    // ============ Hermes Sessions ============
    sessionsExport: async (output: string, source?: string, sessionId?: string) =>
      tauriCall<GatewayResult>('sessions_export', { output, source, sessionId }),
    sessionsPrune: async (olderThan?: number, source?: string, yes?: boolean) =>
      tauriCall<GatewayResult>('sessions_prune', { olderThan, source, yes }),

    // ============ Hermes Insights ============
    getInsights: async (days?: number, source?: string) =>
      tauriCall<GatewayResult>('get_insights', { days, source }),

    // ============ Hermes Config (generic set) ============
    hermesSetConfig: async (key: string, value: unknown) => tauriCall('hermes_set_config', { key, value }),

    // ============ OMP Chat (ACP protocol) ============
    ompChatInit: async (cwd?: string) =>
      tauriCall<void>('omp_chat_init', { cwd }),
    ompChatSend: async (message: string) =>
      tauriCall<void>('omp_chat_send', { message }),
    ompChatClose: async () =>
      tauriCall<void>('omp_chat_close'),
    ompChatListSessions: async () =>
      tauriCall<unknown>('omp_chat_list_sessions'),
    ompChatInfo: async () =>
      tauriCall<{ binary: string }>('omp_chat_info'),
    ompReadModelsConfig: async () =>
      tauriCall<unknown>('omp_read_models_config'),
    ompReadStats: async () =>
      tauriCall<{ sessions: number; messages: number }>('omp_read_stats'),

  }

  return cachedAPI
}
