<template>
  <div class="flex flex-col overflow-y-auto flex-1 select-none text-sm">
    <!-- Search box -->
    <div class="relative px-2 pt-2 pb-1">
      <SvgIcon name="search" size="2" class="absolute left-4 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-base-content/40" />
      <input
        v-model="searchQuery"
        class="input input-sm input-bordered w-full pl-8 pr-8 h-8 text-sm"
        placeholder="搜索数据库、表、键..."
        @focus="onSearchFocus"
      />
      <button
        v-if="searchQuery"
        class="absolute right-4 top-1/2 -translate-y-1/2 btn btn-ghost btn-xs btn-square text-base-content/50 hover:text-base-content min-h-0 h-5 w-5"
        @click="searchQuery = ''"
        title="清除"
      ><SvgIcon name="x" size="14" /></button>
    </div>

    <div v-if="sortedConnections.length === 0" class="flex flex-col items-center justify-center gap-2 py-8 px-4 text-base-content/50">
      <SvgIcon name="database" size="16" class="w-8 h-8 text-base-content/30" />
      <p class="text-sm">暂无数据库连接</p>
      <button @click="$emit('add-connection')" class="btn btn-primary btn-sm mt-1">添加连接</button>
    </div>

    <div v-for="conn in sortedConnections" :key="conn.id">
      <!-- Connection node -->
      <div
        class="flex items-center gap-1 px-2 py-[3px] rounded cursor-pointer hover:bg-base-200 transition-colors group whitespace-nowrap min-h-[28px]"
        :class="{ 'bg-primary/10 text-primary font-medium': activeConnectionId === conn.id }"
        @click="$emit('toggle', conn.id); $emit('select', conn.id)"
        @contextmenu.prevent="onConnContext($event, conn)"
      >
        <span class="w-4 text-center text-[10px] text-base-content/40 flex-shrink-0 leading-none">{{ isConnectionExpanded(conn.id) ? '▼' : '▶' }}</span>
        <span class="flex-shrink-0 text-sm leading-none w-[18px] text-center">{{ dbTypeIcon(conn.type) }}</span>
        <span class="flex-1 truncate text-sm leading-tight min-w-0" :title="conn.name">{{ conn.name }}</span>
        <span v-if="conn.requiresApproval" class="flex-shrink-0 text-xs" title="SQL 执行审核已开启"><SvgIcon name="lock" size="14" class="align-text-bottom" /></span>
        <span class="badge badge-xs badge-ghost uppercase text-[10px] flex-shrink-0 leading-none">{{ conn.type }}</span>
        <button
          class="btn btn-ghost btn-xs px-1 min-h-0 h-5 w-5 opacity-0 group-hover:opacity-60 hover:!opacity-100 hover:bg-base-300 transition-all"
          @click.stop="$emit('edit', conn)"
          title="编辑"
        ><SvgIcon name="pencil" size="14" /></button>
        <button
          class="btn btn-ghost btn-xs px-1 min-h-0 h-5 w-5 opacity-0 group-hover:opacity-60 hover:!opacity-100 hover:!bg-red-50 dark:hover:!bg-red-900/20 hover:!text-red-500 transition-all"
          @click.stop="$emit('delete', conn.id)"
          title="删除"
        ><SvgIcon name="trash" size="14" class="align-text-bottom" /></button>
      </div>

      <Transition
        enter-from-class="opacity-0 max-h-0"
        enter-to-class="opacity-100 max-h-[1000px]"
        enter-active-class="transition-all duration-[180ms] ease overflow-hidden"
        leave-from-class="opacity-100 max-h-[1000px]"
        leave-to-class="opacity-0 max-h-0"
        leave-active-class="transition-all duration-[120ms] ease overflow-hidden"
      >
        <div v-show="isConnectionExpanded(conn.id)" class="ml-4 pl-2 border-l border-base-200/60">
          <!-- Redis: database -> path-based folder tree -->
          <template v-if="conn.type === 'redis'">
            <div v-if="loadingRedisDatabases[conn.id]" class="px-3 py-2 text-xs text-base-content/50 italic">加载中...</div>
            <template v-else>
              <div
                v-for="redisDb in getFilteredRedisDatabases(conn.id)"
                :key="redisDb.db"
              >
                <!-- Redis DB node -->
                <div
                  class="flex items-center gap-1 px-2 py-[3px] rounded cursor-pointer hover:bg-base-200 transition-colors group whitespace-nowrap min-h-[26px]"
                  @click.stop="onToggleRedisDatabase(conn.id, redisDb.db)"
                  @contextmenu.prevent="onRedisDatabaseContext($event, conn, redisDb.db)"
                >
                  <span class="w-4 text-center text-[10px] text-base-content/40 flex-shrink-0 leading-none">{{ isRedisDatabaseExpanded(conn.id, redisDb.db) ? '▼' : '▶' }}</span>
                  <span class="flex-shrink-0 text-sm leading-none w-[18px] text-center"><SvgIcon name="archive" size="14" class="align-text-bottom" /></span>
                  <span class="flex-1 truncate text-sm leading-tight min-w-0">db{{ redisDb.db }}</span>
                  <span class="text-xs text-base-content/40 tabular-nums flex-shrink-0">{{ redisDb.keys }} keys</span>
                </div>

                <!-- Redis DB children: path-based folder tree -->
                <Transition
                  enter-from-class="opacity-0 max-h-0"
                  enter-to-class="opacity-100 max-h-[1000px]"
                  enter-active-class="transition-all duration-[180ms] ease overflow-hidden"
                  leave-from-class="opacity-100 max-h-[1000px]"
                  leave-to-class="opacity-0 max-h-0"
                  leave-active-class="transition-all duration-[120ms] ease overflow-hidden"
                >
                  <div v-show="isRedisDatabaseExpanded(conn.id, redisDb.db)" class="ml-4 pl-2 border-l border-base-200/40">
                    <div v-if="loadingRedisKeyTrees[redisDbKey(conn.id, redisDb.db)]" class="px-3 py-2 text-xs text-base-content/50 italic">加载中...</div>
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
                      <div v-if="redisDbHasMore[redisDbKey(conn.id, redisDb.db)]" class="flex justify-center py-1">
                        <button @click.stop="loadMoreRedisKeys(conn.id, redisDb.db)" class="btn btn-ghost btn-sm btn-xs">
                          <span v-if="loadingRedisKeyTrees[redisDbKey(conn.id, redisDb.db)]">加载中...</span>
                          <span v-else>加载更多</span>
                        </button>
                      </div>
                    </template>
                    <div v-else class="px-3 py-2 text-xs text-base-content/40 italic">{{ searchQuery ? '无匹配键' : '无键' }}</div>
                  </div>
                </Transition>
              </div>
            </template>
          </template>

          <!-- MySQL / PostgreSQL: database list -->
          <template v-else-if="conn.type === 'mysql' || conn.type === 'postgresql'">
            <!-- Loading databases -->
            <div v-if="loadingDatabases[conn.id]" class="px-3 py-2 text-xs text-base-content/50 italic">加载中...</div>

            <!-- Database list -->
            <template v-else>
              <div
                v-for="dbName in getFilteredDatabases(conn.id)"
                :key="dbName"
              >
                <!-- Database node -->
                <div
                  class="flex items-center gap-1 px-2 py-[3px] rounded cursor-pointer hover:bg-base-200 transition-colors group whitespace-nowrap min-h-[26px]"
                  @click.stop="onToggleDatabase(conn.id, dbName)"
                  @contextmenu.prevent="onDatabaseContext($event, conn, dbName)"
                >
                  <span class="w-4 text-center text-[10px] text-base-content/40 flex-shrink-0 leading-none">{{ isDatabaseExpanded(conn.id, dbName) ? '▼' : '▶' }}</span>
                  <span class="flex-shrink-0 text-sm leading-none w-[18px] text-center">📦</span>
                  <span class="flex-1 truncate text-sm leading-tight min-w-0" :title="dbName">{{ dbName }}</span>
                </div>

                <!-- Database children: Tables + Views -->
                <Transition
                  enter-from-class="opacity-0 max-h-0"
                  enter-to-class="opacity-100 max-h-[1000px]"
                  enter-active-class="transition-all duration-[180ms] ease overflow-hidden"
                  leave-from-class="opacity-100 max-h-[1000px]"
                  leave-to-class="opacity-0 max-h-0"
                  leave-active-class="transition-all duration-[120ms] ease overflow-hidden"
                >
                  <div v-show="isDatabaseExpanded(conn.id, dbName)" class="ml-4 pl-2 border-l border-base-200/40">
                    <!-- Tables folder -->
                    <div
                      class="flex items-center gap-1 px-2 py-[3px] rounded cursor-pointer hover:bg-base-200 transition-colors group whitespace-nowrap min-h-[26px]"
                      @click.stop="onToggleDbTables(conn.id, dbName)"
                      @contextmenu.prevent="onFolderContext($event, conn)"
                    >
                      <span class="w-4 text-center text-[10px] text-base-content/40 flex-shrink-0 leading-none">{{ areDbTablesExpanded(conn.id, dbName) ? '▼' : '▶' }}</span>
                      <span class="flex-shrink-0 text-sm leading-none w-[18px] text-center">📋</span>
                      <span class="flex-1 truncate text-sm leading-tight min-w-0 font-medium">Tables</span>
                      <span v-if="dbTables[dbKey(conn.id, dbName)] !== undefined" class="text-xs text-base-content/40 tabular-nums flex-shrink-0 ml-auto">
                        {{ dbTables[dbKey(conn.id, dbName)]?.length ?? 0 }}
                      </span>
                    </div>

                    <!-- Tables list -->
                    <Transition
                      enter-from-class="opacity-0 max-h-0"
                      enter-to-class="opacity-100 max-h-[1000px]"
                      enter-active-class="transition-all duration-[180ms] ease overflow-hidden"
                      leave-from-class="opacity-100 max-h-[1000px]"
                      leave-to-class="opacity-0 max-h-0"
                      leave-active-class="transition-all duration-[120ms] ease overflow-hidden"
                    >
                      <div v-show="areDbTablesExpanded(conn.id, dbName)" class="ml-3 pl-1">
                        <div v-if="loadingTables[dbKey(conn.id, dbName)]" class="px-3 py-2 text-xs text-base-content/50 italic">加载中...</div>
                        <div v-else-if="(getFilteredTables(conn.id, dbName).length ?? 0) === 0" class="px-3 py-2 text-xs text-base-content/40 italic">{{ searchQuery ? '无匹配表' : '无表' }}</div>
                        <div
                          v-for="table in getFilteredTables(conn.id, dbName)"
                          :key="table"
                          class="flex items-center gap-1 px-2 py-[2px] rounded cursor-pointer hover:bg-base-200 transition-colors group whitespace-nowrap min-h-[24px] text-sm"
                          :class="{ 'bg-primary/10': selectedTable === table && activeConnectionId === conn.id }"
                          @dblclick.stop="onSelectTable(conn.id, table, dbName)"
                          @contextmenu.prevent="onTableContext($event, conn, table, dbName)"
                        >
                          <span class="flex-shrink-0 text-xs leading-none w-[16px] text-center">📄</span>
                          <span class="flex-1 truncate text-sm leading-tight min-w-0" :title="getTableTooltip(conn.id, table)">{{ table }}</span>
                          <span v-if="getTableComment(conn.id, table)" class="text-[11px] text-base-content/40 ml-1 truncate hidden sm:inline max-w-[120px]">{{ getTableComment(conn.id, table) }}</span>
                        </div>
                      </div>
                    </Transition>

                    <!-- Views folder -->
                    <div
                      class="flex items-center gap-1 px-2 py-[3px] rounded cursor-pointer hover:bg-base-200 transition-colors group whitespace-nowrap min-h-[26px]"
                      @click.stop="onToggleDbViews(conn.id, dbName)"
                      @contextmenu.prevent="onFolderContext($event, conn)"
                    >
                      <span class="w-4 text-center text-[10px] text-base-content/40 flex-shrink-0 leading-none">{{ areDbViewsExpanded(conn.id, dbName) ? '▼' : '▶' }}</span>
                      <span class="flex-shrink-0 text-sm leading-none w-[18px] text-center">👁️</span>
                      <span class="flex-1 truncate text-sm leading-tight min-w-0 font-medium">Views</span>
                      <span v-if="dbViews[dbKey(conn.id, dbName)] !== undefined" class="text-xs text-base-content/40 tabular-nums flex-shrink-0 ml-auto">
                        {{ dbViews[dbKey(conn.id, dbName)]?.length ?? 0 }}
                      </span>
                    </div>

                    <!-- Views list -->
                    <Transition
                      enter-from-class="opacity-0 max-h-0"
                      enter-to-class="opacity-100 max-h-[1000px]"
                      enter-active-class="transition-all duration-[180ms] ease overflow-hidden"
                      leave-from-class="opacity-100 max-h-[1000px]"
                      leave-to-class="opacity-0 max-h-0"
                      leave-active-class="transition-all duration-[120ms] ease overflow-hidden"
                    >
                      <div v-show="areDbViewsExpanded(conn.id, dbName)" class="ml-3 pl-1">
                        <div v-if="loadingViews[dbKey(conn.id, dbName)]" class="px-3 py-2 text-xs text-base-content/50 italic">加载中...</div>
                        <div v-else-if="(getFilteredViews(conn.id, dbName).length ?? 0) === 0" class="px-3 py-2 text-xs text-base-content/40 italic">{{ searchQuery ? '无匹配视图' : '无视图' }}</div>
                        <div
                          v-for="view in getFilteredViews(conn.id, dbName)"
                          :key="view"
                          class="flex items-center gap-1 px-2 py-[2px] rounded cursor-pointer hover:bg-base-200 transition-colors group whitespace-nowrap min-h-[24px] text-sm"
                          @click.stop="onSelectTable(conn.id, view, dbName)"
                          @contextmenu.prevent="onViewContext($event, conn, view, dbName)"
                        >
                          <span class="flex-shrink-0 text-xs leading-none w-[16px] text-center">📄</span>
                          <span class="flex-1 truncate text-sm leading-tight min-w-0">{{ view }}</span>
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
            <div>
              <div
                class="flex items-center gap-1 px-2 py-[3px] rounded cursor-pointer hover:bg-base-200 transition-colors group whitespace-nowrap min-h-[26px]"
                @click.stop="onToggleDatabase(conn.id, 'sqlite_main')"
                @contextmenu.prevent="onDatabaseContext($event, conn, conn.path || 'main')"
              >
                <span class="w-4 text-center text-[10px] text-base-content/40 flex-shrink-0 leading-none">{{ isDatabaseExpanded(conn.id, 'sqlite_main') ? '▼' : '▶' }}</span>
                <span class="flex-shrink-0 text-sm leading-none w-[18px] text-center">📄</span>
                <span class="flex-1 truncate text-sm leading-tight min-w-0" :title="conn.path || 'main'">{{ conn.path || 'main' }}</span>
              </div>

              <Transition
                enter-from-class="opacity-0 max-h-0"
                enter-to-class="opacity-100 max-h-[1000px]"
                enter-active-class="transition-all duration-[180ms] ease overflow-hidden"
                leave-from-class="opacity-100 max-h-[1000px]"
                leave-to-class="opacity-0 max-h-0"
                leave-active-class="transition-all duration-[120ms] ease overflow-hidden"
              >
                <div v-show="isDatabaseExpanded(conn.id, 'sqlite_main')" class="ml-4 pl-2 border-l border-base-200/40">
                  <!-- Tables folder -->
                  <div
                    class="flex items-center gap-1 px-2 py-[3px] rounded cursor-pointer hover:bg-base-200 transition-colors group whitespace-nowrap min-h-[26px]"
                    @click.stop="onToggleDbTables(conn.id, 'sqlite_main')"
                    @contextmenu.prevent="onFolderContext($event, conn)"
                  >
                    <span class="w-4 text-center text-[10px] text-base-content/40 flex-shrink-0 leading-none">{{ areDbTablesExpanded(conn.id, 'sqlite_main') ? '▼' : '▶' }}</span>
                    <span class="flex-shrink-0 text-sm leading-none w-[18px] text-center">📋</span>
                    <span class="flex-1 truncate text-sm leading-tight min-w-0 font-medium">Tables</span>
                    <span v-if="dbTables[dbKey(conn.id, 'sqlite_main')] !== undefined" class="text-xs text-base-content/40 tabular-nums flex-shrink-0 ml-auto">
                      {{ dbTables[dbKey(conn.id, 'sqlite_main')]?.length ?? 0 }}
                    </span>
                  </div>

                  <Transition
                    enter-from-class="opacity-0 max-h-0"
                    enter-to-class="opacity-100 max-h-[1000px]"
                    enter-active-class="transition-all duration-[180ms] ease overflow-hidden"
                    leave-from-class="opacity-100 max-h-[1000px]"
                    leave-to-class="opacity-0 max-h-0"
                    leave-active-class="transition-all duration-[120ms] ease overflow-hidden"
                  >
                    <div v-show="areDbTablesExpanded(conn.id, 'sqlite_main')" class="ml-3 pl-1">
                      <div v-if="loadingTables[dbKey(conn.id, 'sqlite_main')]" class="px-3 py-2 text-xs text-base-content/50 italic">加载中...</div>
                      <div v-else-if="(getFilteredTables(conn.id, 'sqlite_main').length ?? 0) === 0" class="px-3 py-2 text-xs text-base-content/40 italic">{{ searchQuery ? '无匹配表' : '无表' }}</div>
                      <div
                        v-for="table in getFilteredTables(conn.id, 'sqlite_main')"
                        :key="table"
                        class="flex items-center gap-1 px-2 py-[2px] rounded cursor-pointer hover:bg-base-200 transition-colors group whitespace-nowrap min-h-[24px] text-sm"
                        :class="{ 'bg-primary/10': selectedTable === table && activeConnectionId === conn.id }"
                        @dblclick.stop="onSelectTable(conn.id, table, undefined)"
                        @contextmenu.prevent="onTableContext($event, conn, table, undefined)"
                      >
                        <span class="flex-shrink-0 text-xs leading-none w-[16px] text-center">📄</span>
                        <span class="flex-1 truncate text-sm leading-tight min-w-0" :title="getTableTooltip(conn.id, table)">{{ table }}</span>
                        <span v-if="getTableComment(conn.id, table)" class="text-[11px] text-base-content/40 ml-1 truncate hidden sm:inline max-w-[120px]">{{ getTableComment(conn.id, table) }}</span>
                      </div>
                    </div>
                  </Transition>

                  <!-- Views folder -->
                  <div
                    class="flex items-center gap-1 px-2 py-[3px] rounded cursor-pointer hover:bg-base-200 transition-colors group whitespace-nowrap min-h-[26px]"
                    @click.stop="onToggleDbViews(conn.id, 'sqlite_main')"
                    @contextmenu.prevent="onFolderContext($event, conn)"
                  >
                    <span class="w-4 text-center text-[10px] text-base-content/40 flex-shrink-0 leading-none">{{ areDbViewsExpanded(conn.id, 'sqlite_main') ? '▼' : '▶' }}</span>
                    <span class="flex-shrink-0 text-sm leading-none w-[18px] text-center">👁️</span>
                    <span class="flex-1 truncate text-sm leading-tight min-w-0 font-medium">Views</span>
                    <span v-if="dbViews[dbKey(conn.id, 'sqlite_main')] !== undefined" class="text-xs text-base-content/40 tabular-nums flex-shrink-0 ml-auto">
                      {{ dbViews[dbKey(conn.id, 'sqlite_main')]?.length ?? 0 }}
                    </span>
                  </div>

                  <Transition
                    enter-from-class="opacity-0 max-h-0"
                    enter-to-class="opacity-100 max-h-[1000px]"
                    enter-active-class="transition-all duration-[180ms] ease overflow-hidden"
                    leave-from-class="opacity-100 max-h-[1000px]"
                    leave-to-class="opacity-0 max-h-0"
                    leave-active-class="transition-all duration-[120ms] ease overflow-hidden"
                  >
                    <div v-show="areDbViewsExpanded(conn.id, 'sqlite_main')" class="ml-3 pl-1">
                      <div v-if="loadingViews[dbKey(conn.id, 'sqlite_main')]" class="px-3 py-2 text-xs text-base-content/50 italic">加载中...</div>
                      <div v-else-if="(getFilteredViews(conn.id, 'sqlite_main').length ?? 0) === 0" class="px-3 py-2 text-xs text-base-content/40 italic">{{ searchQuery ? '无匹配视图' : '无视图' }}</div>
                      <div
                        v-for="view in getFilteredViews(conn.id, 'sqlite_main')"
                        :key="view"
                        class="flex items-center gap-1 px-2 py-[2px] rounded cursor-pointer hover:bg-base-200 transition-colors group whitespace-nowrap min-h-[24px] text-sm"
                        @click.stop="onSelectTable(conn.id, view, undefined)"
                        @contextmenu.prevent="onViewContext($event, conn, view, undefined)"
                      >
                        <span class="flex-shrink-0 text-xs leading-none w-[16px] text-center">📄</span>
                        <span class="flex-1 truncate text-sm leading-tight min-w-0">{{ view }}</span>
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
        class="fixed z-50 min-w-44 rounded-lg bg-base-100 shadow-xl border border-base-300 py-1 text-sm"
        :style="{ top: contextMenu.y + 'px', left: contextMenu.x + 'px' }"
        @click.stop
      >
        <template v-for="(item, idx) in contextMenu.items" :key="idx">
          <div
            v-if="!item.separator"
            class="flex items-center gap-2 px-3 py-2 cursor-pointer hover:bg-base-200 transition-colors text-sm whitespace-nowrap"
            @click="item.action"
          >
            <span class="w-5 text-center flex-shrink-0 text-base">{{ item.icon }}</span>
            <span class="flex-1">{{ item.label }}</span>
          </div>
          <div v-else class="border-t border-base-200 my-1"></div>
        </template>
      </div>
    </Teleport>
    <!-- Overlay to close context menu -->
    <div v-if="contextMenu.visible" class="fixed inset-0 z-40 bg-transparent" @click="closeContextMenu"></div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'// @ts-nocheck
import { useConnectionTree } from './composables/useConnectionTree'
import type { DBConnection } from '../../composables/useDBManager'
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
