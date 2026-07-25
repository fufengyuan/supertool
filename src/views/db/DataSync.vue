<template>
    <div class="flex flex-col h-full p-4 overflow-auto">
        <div class="mb-4">
            <h3 class="text-base font-semibold m-0 mb-1"><SvgIcon name="database" size="14" />  数据同步</h3>
            <p class="text-xs opacity-60 m-0">
                对比并同步两个数据库之间的表数据（Navicat 风格多表对比）
            </p>
        </div>

        <!-- Step 1: Connection & Database Selection -->
        <div v-if="step === 1" class="bg-base-100 rounded-lg p-4">
            <div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-3 mb-4">
                <div class="flex flex-col gap-1">
                    <label class="text-xs font-medium opacity-60"><SvgIcon name="dot" size="14" class="inline-block align-text-bottom" /> 源连接</label>
                    <select
                        v-model="sourceId"
                        @change="onSourceChange"
                        class="select select-bordered w-full"
                    >
                        <option value="">选择源连接</option>
                        <option v-for="conn in connections" :key="conn.id" :value="conn.id">
                            {{ conn.name }} ({{ conn.type }})
                        </option>
                    </select>
                </div>
                <div class="flex flex-col gap-1">
                    <label class="text-xs font-medium opacity-60"><SvgIcon name="dot" size="14" class="inline-block align-text-bottom" /> 目标连接</label>
                    <select
                        v-model="targetId"
                        @change="onTargetChange"
                        class="select select-bordered w-full"
                    >
                        <option value="">选择目标连接</option>
                        <option v-for="conn in connections" :key="conn.id" :value="conn.id">
                            {{ conn.name }} ({{ conn.type }})
                        </option>
                    </select>
                </div>
                <div class="flex flex-col gap-1">
                    <label class="text-xs font-medium opacity-60"><SvgIcon name="folder" size="14" />  源数据库</label>
                    <select
                        v-model="sourceDb"
                        @change="loadSourceTables"
                        :disabled="!sourceId || loadingSourceDb"
                        class="select select-bordered w-full"
                    >
                        <option value="">{{ loadingSourceDb ? "加载中..." : "选择数据库" }}</option>
                        <option v-for="db in sourceDatabases" :key="db" :value="db">
                            {{ db }}
                        </option>
                    </select>
                </div>
                <div class="flex flex-col gap-1">
                    <label class="text-xs font-medium opacity-60"><SvgIcon name="folder" size="14" />  目标数据库</label>
                    <select
                        v-model="targetDb"
                        @change="loadTargetTables"
                        :disabled="!targetId || loadingTargetDb"
                        class="select select-bordered w-full"
                    >
                        <option value="">{{ loadingTargetDb ? "加载中..." : "选择数据库" }}</option>
                        <option v-for="db in targetDatabases" :key="db" :value="db">
                            {{ db }}
                        </option>
                    </select>
                </div>
                <div class="flex flex-col gap-1">
                    <label class="text-xs font-medium opacity-60">同步模式</label>
                    <select v-model="syncMode" class="select select-bordered w-full">
                        <option value="full">完整同步（INSERT + UPDATE + DELETE）</option>
                        <option value="insert_only">仅插入（INSERT only）</option>
                        <option value="update_only">仅更新（UPDATE only）</option>
                    </select>
                </div>
                <div class="flex items-center pt-5">
                    <label class="flex items-center gap-1.5 text-sm cursor-pointer">
                        <input
                            type="checkbox"
                            v-model="useTransaction"
                            class="checkbox checkbox-sm"
                        />
                        使用事务
                    </label>
                </div>
            </div>

            <div class="flex gap-2 justify-end">
                <button
                    @click="goToStep2"
                    :disabled="!sourceId || !targetId || !sourceDb || !targetDb"
                    class="btn btn-primary"
                >
                    下一步：选择表 →
                </button>
            </div>
        </div>

        <!-- Step 2: Multi-Table Selection -->
        <div v-if="step === 2" class="bg-base-100 rounded-lg p-4">
            <div class="flex items-center justify-between mb-3">
                <h4 class="text-sm font-semibold m-0"><SvgIcon name="file" size="14" />  选择要对比数据的表</h4>
                <div class="flex gap-1.5">
                    <button @click="selectAllTables" class="btn btn-ghost btn-xs">全选</button>
                    <button @click="selectCommonTables" class="btn btn-ghost btn-xs">
                        仅共有表
                    </button>
                    <button @click="selectNone" class="btn btn-ghost btn-xs">清空</button>
                </div>
            </div>

            <div
                class="max-h-[300px] overflow-y-auto border border-base-300 rounded-lg p-2 bg-base-200 mb-3"
            >
                <!-- Common tables only (data sync requires same table in both) -->
                <template v-if="commonTablesList.length > 0">
                    <div
                        class="text-[11px] font-semibold uppercase tracking-wider opacity-60 px-2 pb-1 border-b border-base-300 mb-1 first:mt-0 mt-2"
                    >
                        共有表（{{ commonTablesList.length }}）
                    </div>
                    <div
                        v-for="table in commonTablesList"
                        :key="'common-' + table"
                        class="flex items-center gap-2 px-2 py-1.5 rounded cursor-pointer text-sm hover:bg-primary/10"
                    >
                        <label class="flex items-center gap-2 flex-1 cursor-pointer">
                            <input
                                type="checkbox"
                                v-model="selectedTables"
                                :value="table"
                                class="checkbox checkbox-sm"
                            />
                            <span class="flex-1 font-mono text-xs">{{ table }}</span>
                            <span class="badge badge-success badge-sm">共有</span>
                        </label>
                    </div>
                </template>

                <!-- Source-only tables -->
                <template v-if="sourceOnlyTables.length > 0">
                    <div
                        class="text-[11px] font-semibold uppercase tracking-wider opacity-60 px-2 pb-1 border-b border-base-300 mb-1 first:mt-0 mt-2"
                    >
                        仅源端有（无法同步数据）
                    </div>
                    <div
                        v-for="table in sourceOnlyTables"
                        :key="'src-' + table"
                        class="flex items-center gap-2 px-2 py-1.5 rounded opacity-60"
                    >
                        <label class="flex items-center gap-2 flex-1">
                            <span class="flex-1 font-mono text-xs">{{ table }}</span>
                            <span class="badge badge-info badge-sm">仅源</span>
                        </label>
                    </div>
                </template>

                <!-- Target-only tables -->
                <template v-if="targetOnlyTables.length > 0">
                    <div
                        class="text-[11px] font-semibold uppercase tracking-wider opacity-60 px-2 pb-1 border-b border-base-300 mb-1 first:mt-0 mt-2"
                    >
                        仅目标端有（无法同步数据）
                    </div>
                    <div
                        v-for="table in targetOnlyTables"
                        :key="'tgt-' + table"
                        class="flex items-center gap-2 px-2 py-1.5 rounded opacity-60"
                    >
                        <label class="flex items-center gap-2 flex-1">
                            <span class="flex-1 font-mono text-xs">{{ table }}</span>
                            <span class="badge badge-secondary badge-sm">仅目标</span>
                        </label>
                    </div>
                </template>

                <div v-if="commonTablesList.length === 0" class="text-center p-6 opacity-60 italic">
                    两个数据库没有共有的表，无法进行数据同步
                </div>
            </div>

            <!-- Compare key configuration per table (auto-detected PKs, editable with column dropdown) -->
            <div
                v-if="selectedTables.length > 0"
                class="bg-base-200 border border-base-300 rounded-lg p-3 mb-3"
            >
                <h4 class="text-sm font-semibold m-0 mb-2">
                    <SvgIcon name="key" size="14" />  对比字段
                    <span class="font-normal text-xs opacity-60"
                        >（默认主键，可切换为其他业务字段）</span
                    >
                </h4>
                <div
                    v-for="table in selectedTables"
                    :key="'pk-' + table"
                    class="flex items-center gap-2 py-1"
                >
                    <span class="font-mono text-xs min-w-[120px] font-medium">{{ table }}</span>
                    <!-- Multi-select dropdown for columns -->
                    <div class="pk-select-wrapper flex-1 relative" v-if="tableColumns[table]">
                        <div
                            class="flex items-center flex-wrap gap-1 p-1 min-h-[28px] border rounded bg-base-100 cursor-pointer transition-colors"
                            :class="{
                                'border-primary shadow-[0_0_0_2px_rgba(100,108,255,0.15)]':
                                    openDropdown === table,
                                'border-base-300 hover:border-primary': openDropdown !== table,
                            }"
                            @click="toggleDropdown(table)"
                        >
                            <span
                                v-if="compareKeys[table]?.length"
                                class="flex flex-wrap gap-1 flex-1"
                            >
                                <span
                                    v-for="key in compareKeys[table]"
                                    :key="key"
                                    class="inline-flex items-center gap-0.5 px-1.5 py-0.5 text-[11px] font-mono rounded border"
                                    :class="
                                        tablePrimaryKeys[table]?.includes(key)
                                            ? 'bg-success/10 text-success border-success/25'
                                            : 'bg-primary/10 text-primary border-primary/25'
                                    "
                                >
                                    {{ key }}
                                    <span
                                        class="cursor-pointer opacity-60 hover:opacity-100 text-sm leading-none"
                                        @click.stop="removeCompareKey(table, key)"
                                        >×</span
                                    >
                                </span>
                            </span>
                            <span v-else class="text-xs opacity-60">选择对比字段</span>
                            <span
                                class="text-[10px] ml-auto opacity-60 transition-transform"
                                :class="{ 'rotate-180': openDropdown === table }"
                                >▾</span
                            >
                        </div>
                        <!-- Dropdown menu -->
                        <div
                            v-if="openDropdown === table"
                            class="absolute top-full left-0 right-0 max-h-60 overflow-y-auto bg-base-100 border border-base-300 rounded-lg shadow-lg z-[1000] py-1"
                        >
                            <div
                                v-for="col in tableColumns[table]"
                                :key="col"
                                class="flex items-center gap-2 px-3 py-1.5 cursor-pointer hover:bg-primary/10"
                                :class="{ 'bg-primary/5': compareKeys[table]?.includes(col) }"
                                @click="toggleCompareKey(table, col)"
                            >
                                <span
                                    class="flex items-center justify-center w-4 h-4 border border-base-300 rounded text-[10px] transition-all shrink-0"
                                    :class="
                                        compareKeys[table]?.includes(col)
                                            ? 'bg-primary border-primary text-white'
                                            : 'text-transparent'
                                    "
                                >
                                    {{ compareKeys[table]?.includes(col) ? "✓" : "" }}
                                </span>
                                <span class="font-mono text-xs flex-1">{{ col }}</span>
                                <span
                                    v-if="tablePrimaryKeys[table]?.includes(col)"
                                    class="text-[9px] font-bold px-1 py-0.5 rounded bg-success/15 text-success tracking-wider"
                                    >PK</span
                                >
                            </div>
                        </div>
                    </div>
                    <!-- Loading state -->
                    <span v-else class="text-[11px] opacity-60"><SvgIcon name="clock" size="14" />  加载中...</span>
                </div>
            </div>

            <div class="flex items-center justify-between">
                <span class="text-sm font-medium text-primary"
                    >已选 {{ selectedTables.length }} 张表</span
                >
                <div class="flex gap-2">
                    <button @click="step = 1" class="btn btn-ghost">← 返回</button>
                    <button
                        @click="startCompare"
                        :disabled="!canCompare || comparing"
                        class="btn btn-primary"
                    >
                        {{ comparing ? "对比中..." : `🔍 对比 ${selectedTables.length} 张表` }}
                    </button>
                </div>
            </div>
        </div>

        <!-- Comparing State -->
        <div v-if="comparing" class="flex flex-col items-center justify-center py-12 gap-3">
            <span class="loading loading-spinner loading-lg text-primary"></span>
            <p>正在对比表数据... ({{ compareProgress }}/{{ selectedTables.length }})</p>
        </div>

        <!-- Results -->
        <div v-if="result && !comparing" class="flex flex-col gap-4">
            <div class="bg-base-100 rounded-lg p-3 px-4">
                <h4 class="m-0 mb-2 text-sm">数据对比结果</h4>
                <div class="flex gap-3 flex-wrap">
                    <span class="px-2.5 py-1 rounded text-xs font-medium bg-success/10 text-success"
                        >新增: {{ totalInserts }}</span
                    >
                    <span class="px-2.5 py-1 rounded text-xs font-medium bg-warning/10 text-warning"
                        >更新: {{ totalUpdates }}</span
                    >
                    <span class="px-2.5 py-1 rounded text-xs font-medium bg-error/10 text-error"
                        >删除: {{ totalDeletes }}</span
                    >
                    <span class="px-2.5 py-1 rounded text-xs font-medium bg-info/10 text-info"
                        >总计: {{ result.diffs.length }}</span
                    >
                </div>
            </div>

            <!-- Diff Filter -->
            <div class="flex gap-4 py-2">
                <label class="flex items-center gap-1.5 cursor-pointer text-sm">
                    <input
                        type="checkbox"
                        :checked="filterTypes.has('insert')"
                        @change="toggleFilter('insert')"
                        class="checkbox checkbox-sm"
                    />
                    <span class="px-2 py-0.5 rounded text-xs font-medium bg-success/10 text-success"
                        >新增 ({{
                            result.diffs.filter((d) => d.diffType === "insert").length
                        }})</span
                    >
                </label>
                <label class="flex items-center gap-1.5 cursor-pointer text-sm">
                    <input
                        type="checkbox"
                        :checked="filterTypes.has('update')"
                        @change="toggleFilter('update')"
                        class="checkbox checkbox-sm"
                    />
                    <span class="px-2 py-0.5 rounded text-xs font-medium bg-warning/10 text-warning"
                        >更新 ({{
                            result.diffs.filter((d) => d.diffType === "update").length
                        }})</span
                    >
                </label>
                <label class="flex items-center gap-1.5 cursor-pointer text-sm">
                    <input
                        type="checkbox"
                        :checked="filterTypes.has('delete')"
                        @change="toggleFilter('delete')"
                        class="checkbox checkbox-sm"
                    />
                    <span class="px-2 py-0.5 rounded text-xs font-medium bg-error/10 text-error"
                        >删除 ({{
                            result.diffs.filter((d) => d.diffType === "delete").length
                        }})</span
                    >
                </label>
            </div>

            <!-- Filter by table -->
            <div
                v-if="affectedTableList.length > 1"
                class="flex items-center gap-1.5 flex-wrap py-2"
            >
                <span class="text-xs opacity-60 font-medium">按表筛选:</span>
                <label
                    v-for="table in affectedTableList"
                    :key="table"
                    class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full border border-base-300 text-[11px] cursor-pointer font-mono hover:bg-primary/10"
                >
                    <input
                        type="checkbox"
                        v-model="filterTables"
                        :value="table"
                        class="checkbox checkbox-xs"
                    />
                    {{ table }}
                </label>
            </div>

            <!-- Navicat-style Grouped Data Diff View -->
            <div class="flex flex-col gap-1 max-h-[500px] overflow-y-auto">
                <div
                    v-for="(group, tIdx) in groupedDiffs"
                    :key="tIdx"
                    class="border border-base-300 rounded-lg overflow-hidden"
                >
                    <!-- Table group header -->
                    <div
                        class="flex items-center gap-2.5 px-3.5 py-2.5 bg-base-100 cursor-pointer select-none hover:bg-primary/10"
                        @click="toggleTableExpand(group.tableName)"
                    >
                        <span class="text-[10px] opacity-60 w-3 text-center">{{
                            isTableExpanded(group.tableName) ? "▼" : "▶"
                        }}</span>
                        <span class="font-mono text-sm font-semibold">{{ group.tableName }}</span>
                        <span class="text-[11px] opacity-60">{{ group.diffs.length }} 行差异</span>
                        <span class="flex gap-1.5 flex-wrap ml-auto">
                            <span
                                v-for="(count, type) in group.typeCounts"
                                :key="type"
                                class="text-[10px] px-1.5 py-0.5 rounded font-medium"
                                :class="getDiffTypeBadgeClass(type)"
                            >
                                {{ getDiffTypeLabel(type) }} ×{{ count }}
                            </span>
                        </span>
                    </div>

                    <!-- Expanded diff rows -->
                    <div v-if="isTableExpanded(group.tableName)" class="border-t border-base-300">
                        <table class="w-full border-collapse text-xs">
                            <thead>
                                <tr>
                                    <th
                                        class="text-left px-2.5 py-1.5 text-[11px] font-semibold opacity-60 bg-base-200 border-b border-base-300 sticky top-0 w-1/4"
                                    >
                                        主键值
                                    </th>
                                    <th
                                        class="text-left px-2.5 py-1.5 text-[11px] font-semibold opacity-60 bg-base-200 border-b border-base-300 sticky top-0 w-[30%]"
                                    >
                                        源数据
                                    </th>
                                    <th
                                        class="text-left px-2.5 py-1.5 text-[11px] font-semibold opacity-60 bg-base-200 border-b border-base-300 sticky top-0 w-[30%]"
                                    >
                                        目标数据
                                    </th>
                                    <th
                                        class="text-left px-2.5 py-1.5 text-[11px] font-semibold opacity-60 bg-base-200 border-b border-base-300 sticky top-0 w-[70px] text-center"
                                    >
                                        操作
                                    </th>
                                </tr>
                            </thead>
                            <tbody>
                                <template v-for="(diff, dIdx) in group.diffs" :key="dIdx">
                                    <tr
                                        class="transition-colors hover:bg-primary/10"
                                        :class="{
                                            'border-l-[3px] border-l-success':
                                                diff.diffType === 'insert',
                                            'border-l-[3px] border-l-warning':
                                                diff.diffType === 'update',
                                            'border-l-[3px] border-l-error':
                                                diff.diffType === 'delete',
                                        }"
                                    >
                                        <td
                                            class="w-1/4 px-2.5 py-1.5 border-b border-base-300 align-middle"
                                        >
                                            <span class="font-mono text-[11px] font-medium">{{
                                                formatPrimaryKey(diff.primaryKey)
                                            }}</span>
                                        </td>
                                        <td
                                            class="w-[30%] px-2.5 py-1.5 border-b border-base-300 align-middle"
                                        >
                                            <template v-if="diff.sourceRow">
                                                <span class="font-mono text-[11px] break-all">{{
                                                    getRowPreview(diff.sourceRow)
                                                }}</span>
                                            </template>
                                            <span v-else class="opacity-60 italic text-[11px]"
                                                >—</span
                                            >
                                        </td>
                                        <td
                                            class="w-[30%] px-2.5 py-1.5 border-b border-base-300 align-middle"
                                        >
                                            <template v-if="diff.targetRow">
                                                <span class="font-mono text-[11px] break-all">{{
                                                    getRowPreview(diff.targetRow)
                                                }}</span>
                                            </template>
                                            <span v-else class="opacity-60 italic text-[11px]"
                                                >—</span
                                            >
                                        </td>
                                        <td
                                            class="w-[70px] px-2.5 py-1.5 border-b border-base-300 align-middle text-center"
                                        >
                                            <button
                                                class="text-[11px] px-2 py-0.5 border border-base-300 rounded bg-base-200 text-primary cursor-pointer whitespace-nowrap hover:bg-primary/10"
                                                @click="toggleSqlRow(tIdx + '-' + dIdx)"
                                            >
                                                {{
                                                    isSqlRowExpanded(tIdx + "-" + dIdx)
                                                        ? "收起"
                                                        : "详情"
                                                }}
                                            </button>
                                        </td>
                                    </tr>
                                    <!-- Expandable detail row showing changed columns -->
                                    <tr v-if="isSqlRowExpanded(tIdx + '-' + dIdx)">
                                        <td colspan="4" class="p-0 bg-black/[3%]">
                                            <div class="p-3 px-3.5">
                                                <!-- For updates: show column-by-column comparison -->
                                                <template
                                                    v-if="
                                                        diff.diffType === 'update' &&
                                                        diff.sourceRow &&
                                                        diff.targetRow
                                                    "
                                                >
                                                    <div
                                                        class="text-xs font-semibold opacity-60 mb-2"
                                                    >
                                                        列值对比（仅显示变化的列）
                                                    </div>
                                                    <table
                                                        class="w-full border-collapse text-[11px]"
                                                    >
                                                        <thead>
                                                            <tr>
                                                                <th
                                                                    class="text-left px-2 py-1 text-[10px] font-semibold opacity-60 bg-base-200 border-b border-base-300"
                                                                >
                                                                    列名
                                                                </th>
                                                                <th
                                                                    class="text-left px-2 py-1 text-[10px] font-semibold opacity-60 bg-base-200 border-b border-base-300"
                                                                >
                                                                    源值
                                                                </th>
                                                                <th
                                                                    class="text-left px-2 py-1 text-[10px] font-semibold opacity-60 bg-base-200 border-b border-base-300"
                                                                >
                                                                    目标值
                                                                </th>
                                                            </tr>
                                                        </thead>
                                                        <tbody>
                                                            <tr
                                                                v-for="col in getChangedColumns(
                                                                    diff.sourceRow,
                                                                    diff.targetRow,
                                                                )"
                                                                :key="col.name"
                                                                class="bg-warning/5 hover:bg-warning/10"
                                                            >
                                                                <td
                                                                    class="px-2 py-1 border-b border-base-300 font-mono font-semibold"
                                                                >
                                                                    {{ col.name }}
                                                                </td>
                                                                <td
                                                                    class="px-2 py-1 border-b border-base-300 font-mono text-error bg-error/5"
                                                                >
                                                                    {{
                                                                        formatCellValue(
                                                                            col.sourceVal,
                                                                        )
                                                                    }}
                                                                </td>
                                                                <td
                                                                    class="px-2 py-1 border-b border-base-300 font-mono text-success bg-success/5"
                                                                >
                                                                    {{
                                                                        formatCellValue(
                                                                            col.targetVal,
                                                                        )
                                                                    }}
                                                                </td>
                                                            </tr>
                                                            <tr
                                                                v-if="
                                                                    getChangedColumns(
                                                                        diff.sourceRow,
                                                                        diff.targetRow,
                                                                    ).length === 0
                                                                "
                                                            >
                                                                <td
                                                                    colspan="3"
                                                                    class="text-center opacity-60 italic p-2 border-b border-base-300 font-mono"
                                                                >
                                                                    所有列值相同
                                                                </td>
                                                            </tr>
                                                        </tbody>
                                                    </table>
                                                </template>
                                                <!-- For inserts: show source row -->
                                                <template
                                                    v-else-if="
                                                        diff.diffType === 'insert' && diff.sourceRow
                                                    "
                                                >
                                                    <div
                                                        class="text-xs font-semibold opacity-60 mb-2"
                                                    >
                                                        新增行数据
                                                    </div>
                                                    <pre
                                                        class="m-0 p-2 px-2.5 font-mono text-[11px] leading-relaxed overflow-x-auto whitespace-pre-wrap break-all bg-base-200 rounded"
                                                        >{{ formatRow(diff.sourceRow) }}</pre
                                                    >
                                                </template>
                                                <!-- For deletes: show target row -->
                                                <template
                                                    v-else-if="
                                                        diff.diffType === 'delete' && diff.targetRow
                                                    "
                                                >
                                                    <div
                                                        class="text-xs font-semibold opacity-60 mb-2"
                                                    >
                                                        待删除行数据
                                                    </div>
                                                    <pre
                                                        class="m-0 p-2 px-2.5 font-mono text-[11px] leading-relaxed overflow-x-auto whitespace-pre-wrap break-all bg-base-200 rounded"
                                                        >{{ formatRow(diff.targetRow) }}</pre
                                                    >
                                                </template>
                                            </div>
                                        </td>
                                    </tr>
                                </template>
                            </tbody>
                        </table>
                    </div>
                </div>

                <div v-if="filteredDiffs.length === 0" class="text-center p-8 opacity-60 text-sm">
                    所选表数据完全一致 <SvgIcon name="check" size="14" /> 
                </div>
            </div>

            <!-- Execute Actions -->
            <div class="flex items-center justify-between py-3 border-t border-base-300">
                <span class="text-sm font-medium text-primary"
                    >将同步 {{ filteredDiffs.length }} 项更改</span
                >
                <div class="flex gap-2">
                    <button @click="reset" class="btn btn-ghost">重新对比</button>
                    <button
                        @click="showSqlDialog = true"
                        :disabled="filteredDiffs.length === 0"
                        class="btn btn-ghost"
                    >
                        <SvgIcon name="file" size="14" />  查看SQL
                    </button>
                    <button
                        @click="executeSync"
                        :disabled="filteredDiffs.length === 0 || executing"
                        class="btn btn-primary"
                    >
                        {{ executing ? "执行中..." : "🚀 执行同步" }}
                    </button>
                </div>
            </div>
        </div>

        <!-- SQL Preview Dialog -->
        <Teleport to="body">
            <div
                v-if="showSqlDialog"
                class="fixed inset-0 bg-black/50 flex items-center justify-center z-[2000]"
                @click="showSqlDialog = false"
            >
                <div
                    class="bg-base-100 rounded-xl w-[720px] max-w-[90vw] max-h-[80vh] flex flex-col shadow-2xl"
                    @click.stop
                >
                    <div
                        class="flex items-center justify-between px-5 py-4 border-b border-base-300"
                    >
                        <h3 class="m-0 text-base font-semibold">
                            <SvgIcon name="file" size="14" />  待执行 SQL ({{ generatedSqlList.length }} 条)
                        </h3>
                        <div class="flex items-center gap-2">
                            <button @click="copyAllSql" class="btn btn-ghost btn-sm">
                                <SvgIcon name="file" size="14" />  复制全部
                            </button>
                            <button
                                @click="showSqlDialog = false"
                                class="w-7 h-7 border-none rounded-lg bg-transparent opacity-60 text-lg cursor-pointer flex items-center justify-center hover:bg-base-200 hover:opacity-100"
                            >
                                <SvgIcon name="x" size="16" />
                            </button>
                        </div>
                    </div>
                    <div class="px-5 py-4 overflow-y-auto flex-1">
                        <div
                            v-for="(sql, idx) in generatedSqlList"
                            :key="idx"
                            class="flex items-start gap-2.5 p-2.5 px-3 bg-base-200 rounded-lg mb-2 text-xs"
                        >
                            <span
                                class="shrink-0 w-[22px] h-[22px] rounded-full bg-primary text-white flex items-center justify-center text-[11px] font-semibold mt-0.5"
                                >{{ idx + 1 }}</span
                            >
                            <pre
                                class="flex-1 m-0 p-0 whitespace-pre-wrap break-all font-mono text-xs text-base-content bg-transparent"
                                >{{ sql }}</pre
                            >
                            <button
                                @click="copySingleSql(idx)"
                                class="shrink-0 px-2 py-1 border-none rounded bg-transparent cursor-pointer text-xs hover:bg-base-content/10"
                                title="复制"
                            >
                                <SvgIcon name="file" size="14" /> 
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </Teleport>

        <!-- Execution Result -->
        <div
            v-if="execResult"
            class="bg-base-100 rounded-lg p-4 text-center"
            :class="{
                'border border-success': execResult.success,
                'border border-error': !execResult.success,
            }"
        >
            <h4 class="m-0 mb-2 flex items-center justify-center gap-1.5">
                <SvgIcon v-if="execResult.success" name="checkCircle" size="16" class="inline-block text-success" />
                <SvgIcon v-else name="alertCircle" size="16" class="inline-block text-error" />
                {{ execResult.success ? '同步成功' : '同步失败' }}
            </h4>
            <div class="flex gap-4 justify-center text-sm mb-3">
                <span>新增: {{ execResult.inserted }}</span>
                <span>更新: {{ execResult.updated }}</span>
                <span>删除: {{ execResult.deleted }}</span>
                <span>耗时: {{ (execResult.duration / 1000).toFixed(2) }}s</span>
            </div>
            <div v-if="execResult.errors.length > 0" class="text-left my-3 p-3 bg-error/10 rounded">
                <p>错误信息:</p>
                <ul class="m-2 mt-2 pl-5">
                    <li
                        v-for="(err, idx) in execResult.errors"
                        :key="idx"
                        class="text-xs text-error mb-1"
                    >
                        {{ err }}
                    </li>
                </ul>
            </div>
            <button @click="reset" class="btn btn-primary">完成</button>
        </div>
    </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
    import { ref, computed, watch } from "vue";
    import { getTauriAPI } from "../../utils/tauri-api";
    import { useDBManager } from "../../composables/useDBManager";
    import { useToast } from "../../composables/useToast";

    const db = useDBManager();
    const toast = useToast();
    const connections = computed(() => db.connections.value);

    const step = ref(1);
    const sourceId = ref("");
    const targetId = ref("");
    const sourceDb = ref("");
    const targetDb = ref("");
    const syncMode = ref("full");
    const useTransaction = ref(true);
    const comparing = ref(false);
    const compareProgress = ref(0);
    const executing = ref(false);
    const loadingSourceDb = ref(false);
    const loadingTargetDb = ref(false);

    // Database lists
    const sourceDatabases = ref<string[]>([]);
    const targetDatabases = ref<string[]>([]);

    // Table lists
    const sourceTables = ref<string[]>([]);
    const targetTables = ref<string[]>([]);

    // Selected tables
    const selectedTables = ref<string[]>([]);

    // Per-table data: columns and comparison keys
    const tableColumns = ref<Record<string, string[]>>({});
    const tablePrimaryKeys = ref<Record<string, string[]>>({});
    const compareKeys = ref<Record<string, string[]>>({});

    // Dropdown state for column selection
    const openDropdown = ref<string | null>(null);

    interface DataDiffItem {
        diffType: "insert" | "update" | "delete";
        primaryKey: Record<string, any>;
        sourceRow?: Record<string, any>;
        targetRow?: Record<string, any>;
        tableName: string;
    }

    interface DataSyncResult {
        diffs: DataDiffItem[];
        totalInserts: number;
        totalUpdates: number;
        totalDeletes: number;
    }

    const result = ref<DataSyncResult | null>(null);
    const filterTypes = ref<Set<string>>(new Set(["insert", "update", "delete"]));

    // Sync mode → pre-filter diff types
    watch(syncMode, (mode) => {
        if (mode === "insert_only") {
            filterTypes.value = new Set(["insert"]);
        } else if (mode === "update_only") {
            filterTypes.value = new Set(["update"]);
        } else {
            filterTypes.value = new Set(["insert", "update", "delete"]);
        }
    });

    // Auto-fetch table metadata when tables are selected/deselected
    watch(selectedTables, (newTables, oldTables) => {
        const added = newTables.filter((t) => !oldTables.includes(t));
        if (added.length > 0) {
            autoFetchTableMeta(added);
        }
    });
    const filterTables = ref<string[]>([]);
    const execResult = ref<{
        success: boolean;
        inserted: number;
        updated: number;
        deleted: number;
        errors: string[];
        duration: number;
    } | null>(null);
    const showSqlDialog = ref(false);

    // Navicat-style: track expanded table groups and detail rows
    const expandedTableGroups = ref<Set<string>>(new Set());
    const expandedSqlRows = ref<Set<string>>(new Set());

    // Computed — case-insensitive matching
    const commonTablesList = computed(() => {
        const targetLower = new Set(targetTables.value.map((t) => t.toLowerCase()));
        return sourceTables.value.filter((t) => targetLower.has(t.toLowerCase()));
    });

    const sourceOnlyTables = computed(() => {
        const targetLower = new Set(targetTables.value.map((t) => t.toLowerCase()));
        return sourceTables.value.filter((t) => !targetLower.has(t.toLowerCase()));
    });

    const targetOnlyTables = computed(() => {
        const sourceLower = new Set(sourceTables.value.map((t) => t.toLowerCase()));
        return targetTables.value.filter((t) => !sourceLower.has(t.toLowerCase()));
    });

    const affectedTableList = computed(() => {
        if (!result.value) {return [];}
        return [...new Set(result.value.diffs.map((d) => d.tableName))];
    });

    const filteredDiffs = computed(() => {
        if (!result.value) {return [];}
        let diffs = result.value.diffs.filter((d) => filterTypes.value.has(d.diffType));
        if (filterTables.value.length > 0) {
            diffs = diffs.filter((d) => filterTables.value.includes(d.tableName));
        }
        return diffs;
    });

    const totalInserts = computed(() => {
        if (!result.value) {return 0;}
        return result.value.diffs.filter((d) => d.diffType === "insert").length;
    });

    const totalUpdates = computed(() => {
        if (!result.value) {return 0;}
        return result.value.diffs.filter((d) => d.diffType === "update").length;
    });

    const totalDeletes = computed(() => {
        if (!result.value) {return 0;}
        return result.value.diffs.filter((d) => d.diffType === "delete").length;
    });

    const canCompare = computed(() => {
        return (
            selectedTables.value.length > 0 &&
            selectedTables.value.every(
                (t) => compareKeys.value[t] && compareKeys.value[t].length > 0,
            )
        );
    });

    // Navicat-style: group diffs by table
    const groupedDiffs = computed(() => {
        const groups: {
            tableName: string;
            diffs: DataDiffItem[];
            typeCounts: Record<string, number>;
        }[] = [];
        const tableMap = new Map<string, DataDiffItem[]>();

        for (const diff of filteredDiffs.value) {
            if (!tableMap.has(diff.tableName)) {
                tableMap.set(diff.tableName, []);
            }
            tableMap.get(diff.tableName)!.push(diff);
        }

        for (const [tableName, diffs] of tableMap) {
            const typeCounts: Record<string, number> = {};
            for (const diff of diffs) {
                typeCounts[diff.diffType] = (typeCounts[diff.diffType] || 0) + 1;
            }
            groups.push({ tableName, diffs, typeCounts });
        }

        return groups;
    });

    // Watch connection changes
    watch(sourceId, () => {
        sourceDatabases.value = [];
        sourceDb.value = "";
        sourceTables.value = [];
    });

    watch(targetId, () => {
        targetDatabases.value = [];
        targetDb.value = "";
        targetTables.value = [];
    });

    async function onSourceChange() {
        if (!sourceId.value) {return;}
        loadingSourceDb.value = true;
        try {
            const conn = connections.value.find((c) => c.id === sourceId.value);
            if (conn) {
                await getTauriAPI().dbConnect(JSON.parse(JSON.stringify(conn)));
            }
            const res = await getTauriAPI().dbGetDatabases(sourceId.value);
            if (res?.success) {
                sourceDatabases.value = (res as any).databases || [];
            }
        } catch {
            // silently fail
        } finally {
            loadingSourceDb.value = false;
        }
    }

    async function onTargetChange() {
        if (!targetId.value) {return;}
        loadingTargetDb.value = true;
        try {
            const conn = connections.value.find((c) => c.id === targetId.value);
            if (conn) {
                await getTauriAPI().dbConnect(JSON.parse(JSON.stringify(conn)));
            }
            const res = await getTauriAPI().dbGetDatabases(targetId.value);
            if (res?.success) {
                targetDatabases.value = (res as any).databases || [];
            }
        } catch {
            // silently fail
        } finally {
            loadingTargetDb.value = false;
        }
    }

    async function loadSourceTables() {
        if (!sourceId.value || !sourceDb.value) {return;}
        try {
            const res = await getTauriAPI().dbGetTables(sourceId.value, sourceDb.value);
            if (res?.success) {
                sourceTables.value = (res as any).tables || [];
            }
        } catch {
            // silently fail
        }
    }

    async function loadTargetTables() {
        if (!targetId.value || !targetDb.value) {return;}
        try {
            const res = await getTauriAPI().dbGetTables(targetId.value, targetDb.value);
            if (res?.success) {
                targetTables.value = (res as any).tables || [];
            }
        } catch {
            // silently fail
        }
    }

    async function goToStep2() {
        step.value = 2;
        tableColumns.value = {};
        tablePrimaryKeys.value = {};
        compareKeys.value = {};
        if (commonTablesList.value.length > 0) {
            selectedTables.value = [...commonTablesList.value];
            // Auto-fetch columns + primary keys for all common tables
            await autoFetchTableMeta(commonTablesList.value);
        } else {
            selectedTables.value = [];
        }
    }

    async function autoFetchTableMeta(tables: string[]) {
        for (const table of tables) {
            if (tableColumns.value[table]) {continue;} // already fetched
            try {
                // Fetch table structure for columns
                const structRes = await getTauriAPI().dbGetTableStructure(
                    targetId.value,
                    table,
                    targetDb.value,
                );
                if (Array.isArray(structRes) && structRes.length > 0) {
                    tableColumns.value[table] = structRes.map((c: any) => c.COLUMN_NAME || c.name);
                }
                // Fetch primary keys
                const pkRes = await getTauriAPI().dbGetTablePrimaryKeys(
                    targetId.value,
                    table,
                    targetDb.value,
                );
                if (pkRes?.success && pkRes.primaryKeys && pkRes.primaryKeys.length > 0) {
                    tablePrimaryKeys.value[table] = pkRes.primaryKeys;
                    compareKeys.value[table] = [...pkRes.primaryKeys]; // default to PKs
                }
            } catch {
                // fallback: user will need to manually configure
            }
        }
    }

    function selectAllTables() {
        selectedTables.value = [...commonTablesList.value];
        autoFetchTableMeta(commonTablesList.value);
    }

    function selectCommonTables() {
        selectedTables.value = [...commonTablesList.value];
        autoFetchTableMeta(commonTablesList.value);
    }

    function selectNone() {
        selectedTables.value = [];
        tableColumns.value = {};
        tablePrimaryKeys.value = {};
        compareKeys.value = {};
        openDropdown.value = null;
    }

    // Dropdown helpers for compare key selection
    function toggleDropdown(table: string) {
        openDropdown.value = openDropdown.value === table ? null : table;
    }

    function toggleCompareKey(table: string, col: string) {
        if (!compareKeys.value[table]) {compareKeys.value[table] = [];}
        const idx = compareKeys.value[table].indexOf(col);
        if (idx >= 0) {
            compareKeys.value[table].splice(idx, 1);
        } else {
            compareKeys.value[table].push(col);
        }
    }

    function removeCompareKey(table: string, key: string) {
        if (!compareKeys.value[table]) {return;}
        compareKeys.value[table] = compareKeys.value[table].filter((k) => k !== key);
    }

    // Close dropdown when clicking outside
    function closeDropdown(event: MouseEvent) {
        const target = event.target as HTMLElement;
        if (!target.closest(".pk-select-wrapper")) {
            openDropdown.value = null;
        }
    }
    if (typeof document !== "undefined") {
        document.addEventListener("click", closeDropdown);
    }

    async function startCompare() {
        // Check for missing compare keys before starting
        const missingKeyTables = selectedTables.value.filter(
            (t) => !compareKeys.value[t] || compareKeys.value[t].length === 0,
        );
        if (missingKeyTables.length > 0) {
            toast.error(`以下表未配置对比字段，无法对比数据：${missingKeyTables.join(", ")}`);
            return;
        }

        comparing.value = true;
        result.value = null;
        execResult.value = null;
        filterTypes.value = new Set(["insert", "update", "delete"]);
        filterTables.value = [];
        compareProgress.value = 0;
        expandedTableGroups.value = new Set();
        expandedSqlRows.value = new Set();

        const allDiffs: DataDiffItem[] = [];
        let totalInserts = 0;
        let totalUpdates = 0;
        let totalDeletes = 0;

        try {
            for (const table of selectedTables.value) {
                const pks = compareKeys.value[table];
                if (!pks || pks.length === 0) {continue;}

                // Get table structure to determine columns
                const structRes = await getTauriAPI().dbGetTableStructure(
                    sourceId.value,
                    table,
                    sourceDb.value,
                );
                let columns: string[] = [];
                if (Array.isArray(structRes) && structRes.length > 0) {
                    columns = structRes.map((c: any) => c.COLUMN_NAME || c.name);
                }

                const res = await getTauriAPI().dbCompareData({
                    sourceId: sourceId.value,
                    targetId: targetId.value,
                    table,
                    primaryKeys: JSON.parse(JSON.stringify(pks)),
                    columns: JSON.parse(JSON.stringify(columns)),
                    sourceDb: sourceDb.value,
                    targetDb: targetDb.value,
                    tablePrimaryKeys: JSON.parse(
                        JSON.stringify(tablePrimaryKeys.value[table] || []),
                    ),
                });
                compareProgress.value++;

                if (res?.success && res.result?.diffs) {
                    const tableDiffs = res.result.diffs as any[];
                    // Add tableName to each diff
                    for (const diff of tableDiffs) {
                        diff.tableName = table;
                    }
                    allDiffs.push(...tableDiffs);
                    totalInserts += res.result.totalInserts || 0;
                    totalUpdates += res.result.totalUpdates || 0;
                    totalDeletes += res.result.totalDeletes || 0;
                }
            }

            result.value = {
                diffs: allDiffs,
                totalInserts,
                totalUpdates,
                totalDeletes,
            };
            // Auto-expand all table groups on first load
            const tables = [...new Set(allDiffs.map((d) => d.tableName))];
            expandedTableGroups.value = new Set(tables);
        } catch (e: any) {
            toast.error("对比失败: " + (e?.message || "未知错误"));
        } finally {
            comparing.value = false;
        }
    }

    function toggleFilter(type: string) {
        if (filterTypes.value.has(type)) {
            filterTypes.value.delete(type);
        } else {
            filterTypes.value.add(type);
        }
    }

    async function executeSync() {
        executing.value = true;
        try {
            // Execute sync per table (since the backend expects a single table)
            const tablesToSync = [...new Set(filteredDiffs.value.map((d) => d.tableName))];
            let totalInserted = 0;
            let totalUpdated = 0;
            let totalDeleted = 0;
            const allErrors: string[] = [];

            for (const table of tablesToSync) {
                const pks = compareKeys.value[table];
                if (!pks || pks.length === 0) {
                    allErrors.push(`No compare keys configured for table '${table}'`);
                    continue;
                }

                const structRes = await getTauriAPI().dbGetTableStructure(
                    sourceId.value,
                    table,
                    sourceDb.value,
                );
                let columns: string[] = [];
                if (Array.isArray(structRes) && structRes.length > 0) {
                    columns = structRes.map((c: any) => c.COLUMN_NAME || c.name);
                }

                const tableDiffs = filteredDiffs.value.filter((d) => d.tableName === table);

                const syncPayload = JSON.parse(
                    JSON.stringify({
                        sourceConnectionId: sourceId.value,
                        targetConnectionId: targetId.value,
                        tableName: table,
                        primaryKeys: pks,
                        tablePrimaryKeys: tablePrimaryKeys.value[table] || [],
                        columns,
                        diffs: tableDiffs,
                        useTransaction: useTransaction.value,
                        batchSize: 100,
                        sourceDbName: sourceDb.value,
                        targetDbName: targetDb.value,
                    }),
                );
                const res = await getTauriAPI().dbExecuteDataSync(syncPayload);
                if (res?.success) {
                    totalInserted += res.inserted || 0;
                    totalUpdated += res.updated || 0;
                    totalDeleted += res.deleted || 0;
                } else {
                    const r = res as any;
                    if (r?.errors) {allErrors.push(...r.errors);}
                    if (r?.error) {allErrors.push(r.error);}
                    if (allErrors.length === 0) {allErrors.push("同步失败");}
                }
            }

            execResult.value = {
                success: allErrors.length === 0,
                inserted: totalInserted,
                updated: totalUpdated,
                deleted: totalDeleted,
                errors: allErrors,
                duration: 0,
            };
        } catch (e: any) {
            execResult.value = {
                success: false,
                inserted: 0,
                updated: 0,
                deleted: 0,
                errors: [e?.message || "执行失败"],
                duration: 0,
            };
        } finally {
            executing.value = false;
        }
    }

    function reset() {
        result.value = null;
        execResult.value = null;
        filterTypes.value = new Set(["insert", "update", "delete"]);
        filterTables.value = [];
        expandedTableGroups.value = new Set();
        expandedSqlRows.value = new Set();
        comparing.value = false;
        step.value = 2;
    }

    // Generate SQL from diffs for preview
    const generatedSqlList = computed(() => {
        const sqls: string[] = [];
        for (const diff of filteredDiffs.value) {
            const table = escapeIdentifier(diff.tableName);
            if (diff.diffType === "insert" && diff.sourceRow) {
                const cols = Object.keys(diff.sourceRow).map(escapeIdentifier).join(", ");
                const vals = Object.values(diff.sourceRow).map(formatSqlValue).join(", ");
                sqls.push(`INSERT INTO ${table} (${cols}) VALUES (${vals});`);
            } else if (diff.diffType === "update" && diff.sourceRow && diff.primaryKey) {
                const sets = Object.entries(diff.sourceRow)
                    .map(([k, v]) => `${escapeIdentifier(k)} = ${formatSqlValue(v)}`)
                    .join(", ");
                const where = Object.entries(diff.primaryKey)
                    .map(([k, v]) => `${escapeIdentifier(k)} = ${formatSqlValue(v)}`)
                    .join(" AND ");
                sqls.push(`UPDATE ${table} SET ${sets} WHERE ${where};`);
            } else if (diff.diffType === "delete" && diff.primaryKey) {
                const where = Object.entries(diff.primaryKey)
                    .map(([k, v]) => `${escapeIdentifier(k)} = ${formatSqlValue(v)}`)
                    .join(" AND ");
                sqls.push(`DELETE FROM ${table} WHERE ${where};`);
            }
        }
        return sqls;
    });

    function escapeIdentifier(name: string): string {
        return name.includes("-") || name.includes(" ") || /^[0-9]/.test(name) ? `"${name}"` : name;
    }

    function formatSqlValue(val: unknown): string {
        if (val === null || val === undefined) {return "NULL";}
        if (typeof val === "number") {return String(val);}
        if (typeof val === "boolean") {return val ? "1" : "0";}
        return "'" + String(val).replace(/'/g, "''") + "'";
    }

    async function copyAllSql() {
        const text = generatedSqlList.value.join("\n\n");
        await navigator.clipboard.writeText(text);
        toast.success("已复制全部 SQL");
    }

    async function copySingleSql(idx: number) {
        await navigator.clipboard.writeText(generatedSqlList.value[idx]);
        toast.success("已复制");
    }

    function getDiffTypeLabel(type: string): string {
        const labels: Record<string, string> = {
            insert: "新增",
            update: "更新",
            delete: "删除",
        };
        return labels[type] || type;
    }

    function getDiffTypeBadgeClass(type: string): string {
        const classes: Record<string, string> = {
            insert: "bg-success/10 text-success",
            update: "bg-warning/10 text-warning",
            delete: "bg-error/10 text-error",
        };
        return classes[type] || "";
    }

    function formatPrimaryKey(pk: Record<string, any>): string {
        return Object.entries(pk)
            .map(([k, v]) => `${k}=${v}`)
            .join(", ");
    }

    function formatRow(row: Record<string, any>): string {
        return JSON.stringify(row, null, 2);
    }

    // Navicat-style: compact row preview showing only non-PK columns
    function getRowPreview(row: Record<string, any>): string {
        const entries = Object.entries(row);
        if (entries.length === 0) {return "—";}
        // Show first 3 key-value pairs as preview
        const shown = entries.slice(0, 3).map(([k, v]) => `${k}=${formatCellValue(v)}`);
        const preview = shown.join(", ");
        if (entries.length > 3) {return preview + ` (+${entries.length - 3} more)`;}
        return preview;
    }

    function formatCellValue(val: any): string {
        if (val == null) {return "NULL";}
        if (typeof val === "string") {return val.length > 50 ? val.slice(0, 50) + "…" : val;}
        return String(val);
    }

    // Get columns that differ between source and target rows
    function getChangedColumns(
        sourceRow: Record<string, any>,
        targetRow: Record<string, any>,
    ): { name: string; sourceVal: any; targetVal: any }[] {
        const changed: { name: string; sourceVal: any; targetVal: any }[] = [];
        const allKeys = new Set([...Object.keys(sourceRow), ...Object.keys(targetRow)]);
        for (const key of allKeys) {
            const sVal = sourceRow[key];
            const tVal = targetRow[key];
            if (JSON.stringify(sVal) !== JSON.stringify(tVal)) {
                changed.push({ name: key, sourceVal: sVal, targetVal: tVal });
            }
        }
        return changed;
    }

    // Expand/collapse table groups
    function isTableExpanded(tableName: string): boolean {
        return expandedTableGroups.value.has(tableName);
    }

    function toggleTableExpand(tableName: string) {
        const next = new Set(expandedTableGroups.value);
        if (next.has(tableName)) {
            next.delete(tableName);
        } else {
            next.add(tableName);
        }
        expandedTableGroups.value = next;
    }

    // Expand/collapse detail rows
    function isSqlRowExpanded(key: string): boolean {
        return expandedSqlRows.value.has(key);
    }

    function toggleSqlRow(key: string) {
        const next = new Set(expandedSqlRows.value);
        if (next.has(key)) {
            next.delete(key);
        } else {
            next.add(key);
        }
        expandedSqlRows.value = next;
    }
</script>
