<template>
  <div class="connection-tree">
    <!-- Search box -->
    <div class="tree-search">
      <svg class="tree-search-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8" />
        <path d="M21 21l-4.35-4.35" />
      </svg>
      <input
        v-model="searchQuery"
        class="tree-search-input"
        placeholder="搜索数据库、表、键..."
        @focus="onSearchFocus"
      />
      <button v-if="searchQuery" class="tree-search-clear" @click="searchQuery = ''" title="清除">✕</button>
    </div>

    <div v-if="sortedConnections.length === 0" class="tree-empty">
      <svg viewBox="0 0 24 24" width="32" height="32" fill="none" stroke="currentColor" stroke-width="1.5">
        <ellipse cx="12" cy="5" rx="9" ry="3" />
        <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3" />
        <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5" />
      </svg>
      <p>暂无数据库连接</p>
      <button @click="$emit('add-connection')" class="btn btn-primary btn-sm">添加连接</button>
    </div>

    <div v-for="conn in sortedConnections" :key="conn.id" class="tree-connection">
      <!-- Connection node -->
      <div
        class="tree-item"
        :class="{ active: activeConnectionId === conn.id, expanded: isConnectionExpanded(conn.id) }"
        @click="$emit('toggle', conn.id); $emit('select', conn.id)"
        @contextmenu.prevent="onConnContext($event, conn)"
      >
        <span class="tree-toggle">{{ isConnectionExpanded(conn.id) ? '▼' : '▶' }}</span>
        <span class="tree-icon">{{ dbTypeIcon(conn.type) }}</span>
        <span class="tree-label">{{ conn.name }}</span>
        <span v-if="conn.requiresApproval" class="tree-security-badge" title="SQL 执行审核已开启">🔒</span>
        <span class="tree-type-badge">{{ conn.type }}</span>
        <button class="tree-action" @click.stop="$emit('edit', conn)" title="编辑">✏️</button>
        <button class="tree-action tree-action-danger" @click.stop="$emit('delete', conn.id)" title="删除">🗑️</button>
      </div>

      <Transition name="accordion">
        <div v-show="isConnectionExpanded(conn.id)" class="tree-children">
          <!-- Redis: database -> path-based folder tree -->
          <template v-if="conn.type === 'redis'">
            <div v-if="loadingRedisDatabases[conn.id]" class="tree-loading">加载中...</div>
            <template v-else>
              <div
                v-for="redisDb in getFilteredRedisDatabases(conn.id)"
                :key="redisDb.db"
                class="tree-database-group"
              >
                <!-- Redis DB node -->
                <div
                  class="tree-item tree-database"
                  :class="{ expanded: isRedisDatabaseExpanded(conn.id, redisDb.db) }"
                  @click.stop="onToggleRedisDatabase(conn.id, redisDb.db)"
                  @contextmenu.prevent="onRedisDatabaseContext($event, conn, redisDb.db)"
                >
                  <span class="tree-toggle">{{ isRedisDatabaseExpanded(conn.id, redisDb.db) ? '▼' : '▶' }}</span>
                  <span class="tree-icon">🗃️</span>
                  <span class="tree-label">db{{ redisDb.db }}</span>
                  <span class="tree-count">{{ redisDb.keys }} keys</span>
                </div>

                <!-- Redis DB children: path-based folder tree -->
                <Transition name="accordion">
                  <div v-show="isRedisDatabaseExpanded(conn.id, redisDb.db)" class="tree-children">
                    <div v-if="loadingRedisKeyTrees[redisDbKey(conn.id, redisDb.db)]" class="tree-loading">加载中...</div>
                    <template v-else-if="redisKeyTrees[redisDbKey(conn.id, redisDb.db)]">
                      <!-- Render root-level folder nodes -->
                      <RedisFolderNode
                        v-for="child in getRedisRootNodes(conn.id, redisDb.db)"
                        :key="child.segment"
                        :node="child"
                        :conn="conn"
                        :db-index="redisDb.db"
                        :parent-path="''"
                        @open-key="onOpenRedisKey(conn.id, redisDb.db, $event)"
                        @folder-context="onRedisFolderContext"
                        @key-context="onRedisKeyContext"
                        @toggle-folder="(path, expanded) => onToggleRedisFolder(conn.id, redisDb.db, path, expanded)"
                      />
                      <!-- Load More button -->
                      <div v-if="redisDbHasMore[redisDbKey(conn.id, redisDb.db)]" class="tree-load-more">
                        <button @click.stop="loadMoreRedisKeys(conn.id, redisDb.db)" class="btn btn-ghost btn-sm">
                          <span v-if="loadingRedisKeyTrees[redisDbKey(conn.id, redisDb.db)]">加载中...</span>
                          <span v-else>加载更多</span>
                        </button>
                      </div>
                    </template>
                    <div v-else class="tree-empty-sub">{{ searchQuery ? '无匹配键' : '无键' }}</div>
                  </div>
                </Transition>
              </div>
            </template>
          </template>

          <!-- MySQL / PostgreSQL: database list -->
          <template v-else-if="conn.type === 'mysql' || conn.type === 'postgresql'">
            <!-- Loading databases -->
            <div v-if="loadingDatabases[conn.id]" class="tree-loading">加载中...</div>

            <!-- Database list -->
            <template v-else>
              <div
                v-for="dbName in getFilteredDatabases(conn.id)"
                :key="dbName"
                class="tree-database-group"
              >
                <!-- Database node -->
                <div
                  class="tree-item tree-database"
                  :class="{ expanded: isDatabaseExpanded(conn.id, dbName) }"
                  @click.stop="onToggleDatabase(conn.id, dbName)"
                  @contextmenu.prevent="onDatabaseContext($event, conn, dbName)"
                >
                  <span class="tree-toggle">{{ isDatabaseExpanded(conn.id, dbName) ? '▼' : '▶' }}</span>
                  <span class="tree-icon">📦</span>
                  <span class="tree-label" :title="dbName">{{ dbName }}</span>
                </div>

                <!-- Database children: Tables + Views -->
                <Transition name="accordion">
                  <div v-show="isDatabaseExpanded(conn.id, dbName)" class="tree-children">
                    <!-- Tables folder -->
                    <div
                      class="tree-item tree-folder"
                      :class="{ expanded: areDbTablesExpanded(conn.id, dbName) }"
                      @click.stop="onToggleDbTables(conn.id, dbName)"
                      @contextmenu.prevent="onFolderContext($event, conn)"
                    >
                      <span class="tree-toggle">{{ areDbTablesExpanded(conn.id, dbName) ? '▼' : '▶' }}</span>
                      <span class="tree-icon">📋</span>
                      <span class="tree-label">Tables</span>
                      <span class="tree-count" v-if="dbTables[dbKey(conn.id, dbName)] !== undefined">
                        {{ dbTables[dbKey(conn.id, dbName)]?.length ?? 0 }}
                      </span>
                    </div>

                    <!-- Tables list -->
                    <Transition name="accordion">
                      <div v-show="areDbTablesExpanded(conn.id, dbName)" class="tree-tables">
                        <div v-if="loadingTables[dbKey(conn.id, dbName)]" class="tree-loading">加载中...</div>
                        <div v-else-if="(getFilteredTables(conn.id, dbName).length ?? 0) === 0" class="tree-empty-sub">{{ searchQuery ? '无匹配表' : '无表' }}</div>
                        <div
                          v-for="table in getFilteredTables(conn.id, dbName)"
                          :key="table"
                          class="tree-item tree-table"
                          :class="{ selected: selectedTable === table && activeConnectionId === conn.id }"
                          @dblclick.stop="onSelectTable(conn.id, table, dbName)"
                          @contextmenu.prevent="onTableContext($event, conn, table, dbName)"
                        >
                          <span class="tree-icon">📄</span>
                          <span class="tree-label" :title="getTableTooltip(conn.id, table)">{{ table }}</span>
                          <span v-if="getTableComment(conn.id, table)" class="tree-table-comment">{{ getTableComment(conn.id, table) }}</span>
                        </div>
                      </div>
                    </Transition>

                    <!-- Views folder -->
                    <div
                      class="tree-item tree-folder tree-views"
                      :class="{ expanded: areDbViewsExpanded(conn.id, dbName) }"
                      @click.stop="onToggleDbViews(conn.id, dbName)"
                      @contextmenu.prevent="onFolderContext($event, conn)"
                    >
                      <span class="tree-toggle">{{ areDbViewsExpanded(conn.id, dbName) ? '▼' : '▶' }}</span>
                      <span class="tree-icon">👁️</span>
                      <span class="tree-label">Views</span>
                      <span class="tree-count" v-if="dbViews[dbKey(conn.id, dbName)] !== undefined">
                        {{ dbViews[dbKey(conn.id, dbName)]?.length ?? 0 }}
                      </span>
                    </div>

                    <!-- Views list -->
                    <Transition name="accordion">
                      <div v-show="areDbViewsExpanded(conn.id, dbName)" class="tree-tables">
                        <div v-if="loadingViews[dbKey(conn.id, dbName)]" class="tree-loading">加载中...</div>
                        <div v-else-if="(getFilteredViews(conn.id, dbName).length ?? 0) === 0" class="tree-empty-sub">{{ searchQuery ? '无匹配视图' : '无视图' }}</div>
                        <div
                          v-for="view in getFilteredViews(conn.id, dbName)"
                          :key="view"
                          class="tree-item tree-view"
                          @click.stop="onSelectTable(conn.id, view, dbName)"
                          @contextmenu.prevent="onViewContext($event, conn, view, dbName)"
                        >
                          <span class="tree-icon">📄</span>
                          <span class="tree-label">{{ view }}</span>
                        </div>
                      </div>
                    </Transition>
                  </div>
                </Transition>
              </div>
            </template>
          </template>

          <!-- SQLite: single database node (file path) -->
          <template v-else-if="conn.type === 'sqlite'">
            <div class="tree-database-group">
              <div
                class="tree-item tree-database"
                :class="{ expanded: isDatabaseExpanded(conn.id, 'sqlite_main') }"
                @click.stop="onToggleDatabase(conn.id, 'sqlite_main')"
                @contextmenu.prevent="onDatabaseContext($event, conn, conn.path || 'main')"
              >
                <span class="tree-toggle">{{ isDatabaseExpanded(conn.id, 'sqlite_main') ? '▼' : '▶' }}</span>
                <span class="tree-icon">📄</span>
                <span class="tree-label" :title="conn.path || 'main'">{{ conn.path || 'main' }}</span>
              </div>

              <Transition name="accordion">
                <div v-show="isDatabaseExpanded(conn.id, 'sqlite_main')" class="tree-children">
                  <!-- Tables folder -->
                  <div
                    class="tree-item tree-folder"
                    :class="{ expanded: areDbTablesExpanded(conn.id, 'sqlite_main') }"
                    @click.stop="onToggleDbTables(conn.id, 'sqlite_main')"
                    @contextmenu.prevent="onFolderContext($event, conn)"
                  >
                    <span class="tree-toggle">{{ areDbTablesExpanded(conn.id, 'sqlite_main') ? '▼' : '▶' }}</span>
                    <span class="tree-icon">📋</span>
                    <span class="tree-label">Tables</span>
                    <span class="tree-count" v-if="dbTables[dbKey(conn.id, 'sqlite_main')] !== undefined">
                      {{ dbTables[dbKey(conn.id, 'sqlite_main')]?.length ?? 0 }}
                    </span>
                  </div>

                  <Transition name="accordion">
                    <div v-show="areDbTablesExpanded(conn.id, 'sqlite_main')" class="tree-tables">
                      <div v-if="loadingTables[dbKey(conn.id, 'sqlite_main')]" class="tree-loading">加载中...</div>
                      <div v-else-if="(getFilteredTables(conn.id, 'sqlite_main').length ?? 0) === 0" class="tree-empty-sub">{{ searchQuery ? '无匹配表' : '无表' }}</div>
                      <div
                        v-for="table in getFilteredTables(conn.id, 'sqlite_main')"
                        :key="table"
                        class="tree-item tree-table"
                        :class="{ selected: selectedTable === table && activeConnectionId === conn.id }"
                        @dblclick.stop="onSelectTable(conn.id, table, undefined)"
                        @contextmenu.prevent="onTableContext($event, conn, table, undefined)"
                      >
                        <span class="tree-icon">📄</span>
                        <span class="tree-label" :title="getTableTooltip(conn.id, table)">{{ table }}</span>
                        <span v-if="getTableComment(conn.id, table)" class="tree-table-comment">{{ getTableComment(conn.id, table) }}</span>
                      </div>
                    </div>
                  </Transition>

                  <!-- Views folder -->
                  <div
                    class="tree-item tree-folder tree-views"
                    :class="{ expanded: areDbViewsExpanded(conn.id, 'sqlite_main') }"
                    @click.stop="onToggleDbViews(conn.id, 'sqlite_main')"
                    @contextmenu.prevent="onFolderContext($event, conn)"
                  >
                    <span class="tree-toggle">{{ areDbViewsExpanded(conn.id, 'sqlite_main') ? '▼' : '▶' }}</span>
                    <span class="tree-icon">👁️</span>
                    <span class="tree-label">Views</span>
                    <span class="tree-count" v-if="dbViews[dbKey(conn.id, 'sqlite_main')] !== undefined">
                      {{ dbViews[dbKey(conn.id, 'sqlite_main')]?.length ?? 0 }}
                    </span>
                  </div>

                  <Transition name="accordion">
                    <div v-show="areDbViewsExpanded(conn.id, 'sqlite_main')" class="tree-tables">
                      <div v-if="loadingViews[dbKey(conn.id, 'sqlite_main')]" class="tree-loading">加载中...</div>
                      <div v-else-if="(getFilteredViews(conn.id, 'sqlite_main').length ?? 0) === 0" class="tree-empty-sub">{{ searchQuery ? '无匹配视图' : '无视图' }}</div>
                      <div
                        v-for="view in getFilteredViews(conn.id, 'sqlite_main')"
                        :key="view"
                        class="tree-item tree-view"
                        @click.stop="onSelectTable(conn.id, view, undefined)"
                        @contextmenu.prevent="onViewContext($event, conn, view, undefined)"
                      >
                        <span class="tree-icon">📄</span>
                        <span class="tree-label">{{ view }}</span>
                      </div>
                    </div>
                  </Transition>
                </div>
              </Transition>
            </div>
          </template>
        </div>
      </Transition>
    </div>

    <!-- Context Menu -->
    <Teleport to="body">
      <div
        v-if="contextMenu.visible"
        class="context-menu"
        :style="{ top: contextMenu.y + 'px', left: contextMenu.x + 'px' }"
        @click.stop
      >
        <template v-for="(item, idx) in contextMenu.items" :key="idx">
          <div
            v-if="!item.separator"
            class="context-menu-item"
            @click="item.action"
          >
            <span class="context-menu-icon">{{ item.icon }}</span>
            <span class="context-menu-label">{{ item.label }}</span>
          </div>
          <div v-else class="context-menu-separator"></div>
        </template>
      </div>
    </Teleport>
    <!-- Overlay to close context menu -->
    <div v-if="contextMenu.visible" class="context-menu-overlay" @click="closeContextMenu"></div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { useConnectionTree } from '@/composables/useConnectionTree'
import type { DBConnection } from '@/composables/useDBManager'
import RedisFolderNode from './RedisFolderNode.vue'

const props = defineProps<{
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
}>()

const emit = defineEmits<{
  select: [id: string]
  'select-table': [connId: string, table: string, dbName?: string]
  'open-table-data': [connId: string, table: string, dbName?: string]
  'open-table-structure': [connId: string, table: string, dbName?: string]
  'open-sql': [connId: string, table?: string, dbName?: string]
  'refresh-tables': [connId: string]
  toggle: [id: string]
  'toggle-tables': [id: string]
  'toggle-database': [connId: string, dbName: string]
  'toggle-db-tables': [connId: string, dbName: string]
  'toggle-db-views': [connId: string, dbName: string]
  'toggle-redis-database': [connId: string, dbIndex: number]
  'toggle-redis-folder': [connId: string, dbIndex: number, folderPath: string]
  'open-redis-key': [connId: string, dbIndex: number, key: string]
  'open-redis-manager': [connId: string, dbIndex: number]
  'add-connection': []
  edit: [conn: any]
  delete: [id: string]
}>()

const ct = useConnectionTree(props, emit)

const {
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
} = ct

defineExpose({ refreshTables })
</script>

<style scoped>
/* ============================================================
   Redis / Database Tree — IDE Navicat-style
   ============================================================ */

.connection-tree {
  display: flex;
  flex-direction: column;
  gap: 1px;
  overflow-y: auto;
  overflow-x: hidden;
  flex: 1;
  min-height: 0;
  padding: 4px 2px;
  font-size: 12.5px;
  line-height: 1.4;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}

/* ── Search Box ─────────────────────────────────────────── */
.tree-search {
  display: flex;
  align-items: center;
  padding: 5px 8px;
  margin: 2px 2px 4px 2px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 5px;
  background: oklch(var(--b2));
  gap: 6px;
  flex-shrink: 0;
  transition: border-color 0.15s ease, box-shadow 0.15s ease;
}

.tree-search:focus-within {
  border-color: oklch(var(--p));
  box-shadow: 0 0 0 2px oklch(var(--p) / 0.1);
}

.tree-search-icon {
  color: oklch(var(--bc) / 0.6);
  flex-shrink: 0;
}

.tree-search-input {
  flex: 1;
  border: none;
  background: transparent;
  outline: none;
  font-size: 12px;
  color: oklch(var(--bc));
  min-width: 0;
}

.tree-search-input::placeholder {
  color: oklch(var(--bc) / 0.6);
}

.tree-search-clear {
  width: 16px;
  height: 16px;
  border: none;
  background: transparent;
  cursor: pointer;
  border-radius: 50%;
  font-size: 10px;
  color: oklch(var(--bc) / 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all 0.12s ease;
}

.tree-search-clear:hover {
  background: oklch(var(--p) / 0.1);
  color: oklch(var(--p));
}

/* ── Empty State ────────────────────────────────────────── */
.tree-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 32px 16px;
  color: oklch(var(--bc) / 0.6);
  text-align: center;
}

.tree-empty svg {
  opacity: 0.35;
}

.tree-empty p {
  font-size: 12.5px;
  margin: 0;
}

/* ── Connection wrapper ─────────────────────────────────── */
.tree-connection {
  display: flex;
  flex-direction: column;
}

/* ============================================================
   Tree Items — core
   ============================================================ */

.tree-item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 3px 6px;
  margin: 0 2px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12.5px;
  line-height: 1.5;
  color: oklch(var(--bc));
  transition: background 0.12s ease, color 0.12s ease;
  user-select: none;
  white-space: nowrap;
  min-height: 22px;
}

/* Subtle indentation guide line — left border on children containers */
.tree-children,
.tree-tables {
  position: relative;
  padding-left: 16px;
}

.tree-children::before,
.tree-tables::before {
  content: '';
  position: absolute;
  left: 8px;
  top: 0;
  bottom: 0;
  width: 1px;
  background: oklch(var(--bc) / 0.1);
  opacity: 0.6;
  pointer-events: none;
}

.tree-database-group {
  display: flex;
  flex-direction: column;
}

/* ── Hover ──────────────────────────────────────────────── */
.tree-item:hover {
  background: oklch(var(--p) / 0.1);
}

/* ── Active / Selected ──────────────────────────────────── */
.tree-item.active,
.tree-item.selected {
  background: oklch(var(--p)) !important;
  color: #fff !important;
  font-weight: 500;
}

.tree-item.active .tree-count,
.tree-item.selected .tree-count {
  background: rgba(255, 255, 255, 0.25);
  color: #fff;
}

.tree-item.active .tree-toggle,
.tree-item.selected .tree-toggle {
  color: rgba(255, 255, 255, 0.8);
}

.tree-item.active .tree-type-badge,
.tree-item.selected .tree-type-badge {
  background: rgba(255, 255, 255, 0.2);
  color: #fff;
}

.tree-item.active .tree-action,
.tree-item.selected .tree-action {
  opacity: 0.85;
}

.tree-item.active .tree-action:hover,
.tree-item.selected .tree-action:hover {
  background: rgba(255, 255, 255, 0.2);
}

/* ============================================================
   Node-type variants
   ============================================================ */

/* Connection node */
.tree-item:not(.tree-folder):not(.tree-database):not(.tree-table):not(.tree-view):not(.tree-redis-key) {
  font-weight: 600;
  font-size: 12.5px;
}

/* Database node */
.tree-item.tree-database {
  font-weight: 600;
  font-size: 12px;
}

.tree-item.tree-database:hover {
  background: oklch(var(--p) / 0.1);
}

/* Folder nodes (Tables, Views, Redis folders) */
.tree-item.tree-folder {
  color: oklch(var(--bc));
  font-weight: 500;
  font-size: 12px;
}

/* Table nodes */
.tree-item.tree-table {
  font-size: 12px;
}

.tree-item.tree-table:hover {
  background: oklch(var(--p) / 0.1);
}

/* View nodes */
.tree-item.tree-view {
  font-size: 12px;
  font-style: italic;
  opacity: 0.85;
}

.tree-item.tree-view:hover {
  background: oklch(var(--p) / 0.1);
  opacity: 1;
}

/* Redis key leaf nodes */
.tree-item.tree-redis-key {
  font-family: 'SF Mono', 'Fira Code', 'Cascadia Code', 'JetBrains Mono', Consolas, monospace;
  font-size: 11.5px;
  color: oklch(var(--bc));
  padding-left: 20px;  /* no chevron, compensate */
}

.tree-item.tree-redis-key:hover {
  background: oklch(var(--p) / 0.1);
}

/* Redis key type dot indicator (small colored dot before label) */
.tree-redis-key .tree-key-type-dot {
  display: inline-block;
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
  margin-right: 2px;
}

.tree-redis-key .tree-key-type-dot.type-string  { background: #52c41a; }
.tree-redis-key .tree-key-type-dot.type-hash    { background: #1677ff; }
.tree-redis-key .tree-key-type-dot.type-list    { background: #fa8c16; }
.tree-redis-key .tree-key-type-dot.type-set     { background: #722ed1; }
.tree-redis-key .tree-key-type-dot.type-zset    { background: #eb2f96; }
.tree-redis-key .tree-key-type-dot.type-default { background: #8c8c8c; }

/* ============================================================
   Sub-elements within tree items
   ============================================================ */

/* Chevron toggle */
.tree-toggle {
  width: 14px;
  height: 14px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 8px;
  flex-shrink: 0;
  color: oklch(var(--bc) / 0.6);
  transition: transform 0.15s ease, color 0.12s ease;
}

.tree-toggle:hover {
  color: oklch(var(--bc));
}

.tree-item.active .tree-toggle,
.tree-item.selected .tree-toggle {
  color: rgba(255, 255, 255, 0.75);
}

/* Icon / emoji */
.tree-icon {
  width: 16px;
  height: 16px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  font-size: 13px;
  line-height: 1;
}

/* Label */
.tree-label {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Key count badge — subtle gray pill */
.tree-count {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 10px;
  background: oklch(var(--b2));
  color: oklch(var(--bc) / 0.6);
  flex-shrink: 0;
  line-height: 1.4;
  transition: background 0.12s ease, color 0.12s ease;
  max-width: 50px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: center;
}

.tree-item:hover .tree-count {
  background: oklch(var(--bc) / 0.1);
}

/* Type badge (connection level) */
.tree-type-badge {
  font-size: 9.5px;
  padding: 1px 6px;
  border-radius: 4px;
  background: oklch(var(--b2));
  color: oklch(var(--bc) / 0.6);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  font-weight: 600;
  flex-shrink: 0;
  line-height: 1.5;
  transition: background 0.12s ease, color 0.12s ease;
}

.tree-security-badge {
  font-size: 11px;
  flex-shrink: 0;
}

/* Action buttons */
.tree-action {
  width: 20px;
  height: 20px;
  border: none;
  background: transparent;
  cursor: pointer;
  border-radius: 3px;
  font-size: 11px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.15s ease, background 0.12s ease;
}

.tree-item:hover .tree-action {
  opacity: 1;
}

.tree-action:hover {
  background: oklch(var(--bc) / 0.1);
}

.tree-action-danger:hover {
  background: rgba(210, 15, 57, 0.12);
}

/* Table comment */
.tree-table-comment {
  font-size: 11px;
  color: oklch(var(--bc) / 0.6);
  opacity: 0.65;
  max-width: 100px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-left: 4px;
  flex-shrink: 1;
}

/* ============================================================
   Loading / empty sub-items
   ============================================================ */

.tree-loading {
  padding: 4px 8px 4px 24px;
  font-size: 11.5px;
  color: oklch(var(--bc) / 0.6);
  font-style: italic;
}

.tree-empty-sub {
  text-align: center;
  color: oklch(var(--bc) / 0.6);
  padding: 8px;
  font-size: 12px;
}

.tree-load-more {
  text-align: center;
  padding: 8px 0;
  border-top: 1px dashed oklch(var(--bc) / 0.1);
  margin-top: 4px;
}

/* ============================================================
   Accordion animation
   ============================================================ */

.accordion-enter-active {
  transition: opacity 0.18s ease, max-height 0.25s ease;
}

.accordion-leave-active {
  transition: opacity 0.12s ease, max-height 0.2s ease;
}

.accordion-enter-from,
.accordion-leave-to {
  opacity: 0;
}

/* ============================================================
   Context Menu
   ============================================================ */

.context-menu {
  position: fixed;
  z-index: 10000;
  min-width: 180px;
  padding: 4px;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.18);
  animation: fadeIn 0.1s ease;
}

.context-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border-radius: 5px;
  cursor: pointer;
  font-size: 12px;
  color: oklch(var(--bc));
  transition: background 0.1s ease;
}

.context-menu-item:hover {
  background: oklch(var(--p) / 0.1);
  color: oklch(var(--p));
}

.context-menu-icon {
  font-size: 14px;
  width: 18px;
  text-align: center;
}

.context-menu-label {
  flex: 1;
}

.context-menu-separator {
  height: 1px;
  margin: 4px 8px;
  background: oklch(var(--bc) / 0.1);
}

.context-menu-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 9999;
}

.btn-sm {
  padding: 6px 12px;
  font-size: 12px;
}
</style>