<template>
  <div :class="['h-full flex flex-col bg-base-200 text-base-content', isFullscreen ? 'fixed inset-0 z-[9999] p-2' : '']">
    <div class="flex gap-4 flex-1 min-h-0">
      <!-- 左侧：预设列表（按分组展示）- 全屏时隐藏 -->
      <div v-show="!isFullscreen" class="w-[220px] flex flex-col gap-3 overflow-y-auto">
        <div class="bg-base-100 rounded-box p-2">
          <h3 class="text-xs text-base-content/70 mb-2 font-medium">查询预设</h3>

          <!-- 首屏骨架：与真实分组行同构，避免加载完成瞬间跳动 -->
          <div v-if="presetsLoading && presets.length === 0" class="flex flex-col gap-2">
            <template v-for="i in 7" :key="i">
              <div v-if="i === 3" class="h-3 w-14 rounded bg-base-content/10 animate-pulse" :style="{ animationDelay: `${i * 60}ms` }"></div>
              <div class="flex items-center gap-2">
                <span class="w-1 h-3.5 rounded-full bg-base-content/10 animate-pulse" :style="{ animationDelay: `${i * 60}ms` }"></span>
                <span class="h-3 flex-1 rounded bg-base-content/10 animate-pulse" :style="{ maxWidth: `${42 + ((i * 19) % 44)}%`, animationDelay: `${i * 60}ms` }"></span>
              </div>
            </template>
          </div>

          <!-- 分组 -->
          <div
            v-for="groupEntry in groupedPresets"
            :key="groupEntry.presetGroup"
          >
            <div class="flex items-center gap-1.5 px-1.5 py-1 cursor-pointer select-none rounded-md transition-all duration-200"
              :class="groupStyle(groupEntry.presetGroup).bg"
              @click="togglePresetGroup(groupEntry.presetGroup)">
              <span :class="['w-1 h-3.5 rounded-full shrink-0', groupStyle(groupEntry.presetGroup).dot]"></span>
              <span class="text-[10px] text-base-content/50 min-w-[10px] inline-flex items-center">
                <SvgIcon v-if="collapsedPresetGroups.has(groupEntry.presetGroup)" name="chevronRight" size="10" />
                <SvgIcon v-else name="chevronDown" size="10" />
              </span>
              <span class="font-bold text-xs text-base-content flex-1 truncate">{{ groupEntry.presetGroup }}</span>
              <span class="text-[10px] font-semibold text-base-content/70 bg-base-content/10 rounded-full px-1 py-px tabular-nums">{{ groupEntry.presets.length }}</span>
            </div>
            <div v-show="!collapsedPresetGroups.has(groupEntry.presetGroup)" class="pl-1.5 flex flex-col">
              <div
                v-for="preset in groupEntry.presets"
                :key="preset.id"
                class="group flex items-center gap-1 px-1.5 py-1 rounded-md cursor-pointer transition-colors duration-150"
                :class="selectedPreset?.id === preset.id ? 'bg-primary text-primary-content' : 'hover:bg-base-200'"
                :title="`${preset.name} · ${preset.serverIds.length} 节点 · ${preset.logType}`"
                @click="selectAndQuery(preset)"
              >
                <span v-if="isStreaming && selectedPreset?.id === preset.id" class="w-1.5 h-1.5 shrink-0 rounded-full bg-green-400 animate-pulse"></span>
                <span class="min-w-0 flex-1 truncate text-xs font-medium">{{ preset.name }}</span>
                <span class="shrink-0 text-[10px] opacity-60 tabular-nums group-hover:hidden">{{ preset.serverIds.length }}节点</span>
                <span class="hidden shrink-0 items-center gap-0.5 group-hover:flex">
                  <button @click.stop="editPreset(preset)" class="p-0.5 rounded opacity-60 hover:opacity-100" title="编辑"><SvgIcon name="pencil" size="12" /></button>
                  <button @click.stop="deletePreset(preset.id)" class="p-0.5 rounded opacity-60 hover:opacity-100 hover:text-error" title="删除"><SvgIcon name="x" size="12" /></button>
                </span>
              </div>
            </div>
          </div>

          <div v-if="!presetsLoading && presets.length === 0" class="text-center text-base-content/60 text-xs p-3">
            <template v-if="allServers.length === 0">
              <div class="text-center">
                <p><SvgIcon name="monitor" size="14" class="inline" /> 尚未配置服务器</p>
                <p class="text-xs opacity-70 mt-1">日志聚合需要先添加 SSH 服务器：</p>
                <button @click="goToServers" class="btn btn-primary btn-sm mt-2">前往配置服务器 <SvgIcon name="arrowRight" size="14" class="inline" /></button>
              </div>
            </template>
            <template v-else>
              暂无预设，点击上方按钮添加
            </template>
          </div>
        </div>
      </div>

      <!-- 右侧：日志输出 -->
      <div class="flex-1 flex flex-col min-h-0 bg-base-100 rounded-box overflow-hidden relative">
        <!-- 查询模式切换栏 -->
        <div class="flex items-center gap-3 px-3 py-2 border-b border-base-content/10 bg-base-100">
          <div class="flex gap-0.5 bg-base-200 rounded-lg p-0.5">
            <button
              :class="['btn btn-sm rounded text-xs transition-all', queryMode === 'stream' ? 'btn-primary' : 'btn-ghost text-base-content/60']"
              @click="switchQueryMode('stream')"
            ><SvgIcon name="send" size="14" /> 流式查询</button>
            <button
              :class="['btn btn-sm rounded text-xs transition-all', queryMode === 'search' ? 'btn-primary' : 'btn-ghost text-base-content/60']"
              @click="switchQueryMode('search')"
            ><SvgIcon name="search" size="14" /> 日志搜索</button>
          </div>
          <button @click="openNewPresetForm" class="btn btn-primary btn-sm">+ 新增预设</button>

          <!-- 搜索模式：关键字输入 -->
          <div v-if="queryMode === 'search'" class="flex items-center gap-2 flex-1">
            <input
              v-model="searchKeyword"
              :placeholder="searchPlaceholder"
              class="input input-bordered flex-1 h-8 min-h-0 text-xs"
              @keyup.enter="doSearch"
            />
            <div class="flex items-center gap-1.5 text-xs text-base-content/60 whitespace-nowrap">
              <label>
                上下文行数
                <div class="inline-flex items-center gap-1">
                  <button
                    :class="['btn btn-xs rounded', searchContextLines === 0 ? 'btn-primary' : 'btn-ghost border border-base-content/10']"
                    @click="searchContextLines = 0"
                    title="精准搜索（仅匹配行）"
                  >0行</button>
                  <button
                    :class="['btn btn-xs rounded', searchContextLines === 10 ? 'btn-primary' : 'btn-ghost border border-base-content/10']"
                    @click="searchContextLines = 10"
                    title="模糊搜索（匹配行上下各10行）"
                  >±10行</button>
                  <input
                    v-model.number="searchContextLines"
                    type="number"
                    min="0"
                    max="500"
                    class="input input-bordered w-[60px] h-7 min-h-0 text-xs text-center px-1"
                    placeholder="自定义"
                  />
                </div>
              </label>
              <span class="text-[11px]">{{ searchContextLines === 0 ? '精确匹配' : `匹配行上下各 ${searchContextLines} 行` }}</span>
            </div>
            <button
              @click="doSearch"
              :disabled="!selectedPreset || !searchKeyword.trim() || isSearching"
              class="btn btn-primary btn-sm whitespace-nowrap"
            >
              <template v-if="isSearching"><SvgIcon name="refresh" size="14" :class="{ 'animate-spin': isSearching }" /> 搜索中...</template><template v-else><SvgIcon name="search" size="14" /> 搜索</template>
            </button>
          </div>
        </div>

        <div class="flex justify-between items-center px-3 py-2 border-b border-base-content/10 text-xs flex-wrap gap-2">
          <div class="text-base-content/60 flex gap-1 flex-wrap">
            <template v-if="queryMode === 'stream' && selectedPreset?.keywords?.length">
              <span>{{ displayLines.length }} 行</span>
              <span class="text-base-content/30">/</span>
              <span class="text-base-content/40">{{ logLines.length }} 行(全部)</span>
            </template>
            <template v-else-if="queryMode === 'search' && hasSearched">
              <span>{{ matchIndices.length }} 个匹配</span>
              <span class="text-base-content/30">/</span>
              <span class="text-base-content/40">{{ displayLines.length }} 行</span>
              <!-- 搜索导航 -->
              <span class="ml-2 flex items-center gap-1">
                <button @click="prevMatch" :disabled="matchIndices.length === 0 || currentMatchIndex <= 0"
                  class="btn btn-ghost btn-xs px-1" title="上一个匹配 (Shift+N)">
                  <SvgIcon name="chevronUp" size="12" />
                </button>
                <span class="text-xs font-mono min-w-[40px] text-center">
                  {{ matchIndices.length > 0 ? currentMatchIndex + 1 : 0 }}/{{ matchIndices.length }}
                </span>
                <button @click="nextMatch" :disabled="matchIndices.length === 0 || currentMatchIndex >= matchIndices.length - 1"
                  class="btn btn-ghost btn-xs px-1" title="下一个匹配 (N)">
                  <SvgIcon name="chevronDown" size="12" />
                </button>
              </span>
            </template>
            <template v-else>
              <span>{{ displayLines.length }} 行</span>
            </template>
            <span v-if="activeServers.size > 0">· {{ activeServers.size }} 个节点在线</span>
            <span v-if="selectedPreset" class="text-primary font-medium">· 当前：{{ selectedPreset.name }}</span>
          </div>
          <div class="flex items-center gap-2">
            <!-- 节点筛选：聚合全部 / 单节点切换 -->
            <select
              v-if="availableServers.length > 1"
              v-model="selectedServerFilter"
              class="select select-sm select-bordered h-8 min-h-0 text-xs w-[140px]"
              :title="selectedServerFilter ? '当前仅显示该节点日志' : '聚合显示所有节点日志'"
            >
              <option :value="null">全部节点 ({{ availableServers.length }})</option>
              <option v-for="s in availableServers" :key="s.id" :value="s.id">
                {{ s.name }}{{ s.online ? ' ●' : '' }}
              </option>
            </select>
            <button
              @click="stopQuery"
              v-if="isStreaming"
              class="btn btn-error btn-sm animate-pulse"
              title="终止当前日志查询"
            >
              <SvgIcon name="stopSquare" size="14" /> 终止
            </button>
            <button
              @click="resumeQuery"
              v-if="!isStreaming && selectedPreset && queryMode === 'stream' && logLines.length > 0"
              class="btn btn-primary btn-sm"
              title="继续查询同预设"
            >
              <SvgIcon name="refresh" size="14" /> 继续
            </button>
            <button @click="clearLogs" class="btn btn-ghost btn-sm border border-base-content/10">清除</button>
            <button @click="viewFullRemoteLog" v-if="selectedPreset && selectedPreset.logType === 'file'" class="btn btn-ghost btn-sm border border-base-content/10" title="下载远程日志文件到本地离线查看，支持多节点"><SvgIcon name="fileText" size="14" /> 离线查看</button>
            <!-- 全屏切换：隐藏侧栏 + 浮动覆盖整个窗口 -->
            <button @click="toggleFullscreen" class="btn btn-ghost btn-sm border border-base-content/10" :title="isFullscreen ? '退出全屏 (Esc)' : '全屏显示'">
              <SvgIcon :name="isFullscreen ? 'minimize' : 'maximize'" size="14" /> {{ isFullscreen ? '退出全屏' : '全屏' }}
            </button>
          </div>
        </div>

        <div class="flex-1 overflow-y-auto p-2 font-mono text-xs leading-relaxed allow-select" ref="logContainer" @scroll="onScroll" style="overflow-anchor: none">
          <div v-if="displayLines.length === 0 && !isStreaming && !hasSearched"
            class="log-empty flex h-full flex-col items-center justify-center gap-2 text-base-content/50">
            <SvgIcon :name="queryMode === 'stream' ? 'fileText' : 'search'" size="36" stroke-width="1.4" class="opacity-35" />
            <p class="m-0 text-sm">{{ queryMode === 'stream' ? '选择左侧预设开始查询日志' : '输入关键字后点击搜索' }}</p>
            <p class="m-0 text-[11px] opacity-70">{{ queryMode === 'stream' ? '流式模式实时追加输出，自动滚到最新' : '搜索模式按时间倒序返回匹配行' }}</p>
          </div>

          <div v-if="queryMode === 'search' && searchKeyword.trim() && !isSearching && displayLines.length === 0 && hasSearched" class="flex items-center justify-center h-full text-base-content/60">
            <p>未找到匹配结果</p>
          </div>

          <!-- 手动加载更多历史日志按钮（仅 stream 模式） -->
          <div v-if="queryMode === 'stream' && isStreaming && streamId && displayLines.length > 0" class="text-center py-1">
            <button
              @click="loadMoreHistory"
              :disabled="loadingMore"
              class="btn btn-ghost btn-xs text-[10px] text-base-content/50 hover:text-base-content"
            >
              <SvgIcon v-if="loadingMore" name="refresh" size="11" class="animate-spin" />
              {{ loadingMore ? '加载中...' : '加载更多历史日志' }}
            </button>
          </div>

          <!-- 虚拟滚动容器：流式模式用 paddingTop/Bottom 撑起全量滚动空间，只渲染视口附近的行；
               搜索模式全量渲染——日志行 whitespace-pre-wrap 长行会换行，固定行高估算的 spacer
               与真实内容高度不符，滚动时 scrollHeight 波动导致浏览器把 scrollTop 钳回（无法滑到底部）。
               流式模式改用「真实行高前缀和」撑高（streamPrefixAt），逐步贴合实际内容高度，消除反弹 -->
          <div
            v-if="displayLines.length > 0"
            :style="queryMode === 'stream'
              ? (streamUseRealHeight
                  ? { paddingTop: streamPrefixAt(visibleStart) + 'px', paddingBottom: (streamPrefixAt(totalItems) - streamPrefixAt(visibleEnd)) + 'px' }
                  : { paddingTop: (visibleStart * VIRTUAL_LINE_HEIGHT) + 'px', paddingBottom: ((totalItems - visibleEnd) * VIRTUAL_LINE_HEIGHT) + 'px' })
              : undefined"
          >
            <div
              v-for="(line, i) in renderedLines"
              :key="line.id"
              :data-log-idx="queryMode === 'search' ? i : undefined"
              :data-stream-idx="queryMode === 'stream' ? visibleStart + i : undefined"
              class="flex gap-2 py-0.5 hover:bg-white/5"
              :class="{
                'bg-warning/10 border-l-4 border-warning': line.isMatch,
                'bg-primary/20 border-l-4 border-primary': line.id === currentMatchId
              }"
            >
              <span v-if="queryMode === 'search'" class="text-base-content/60 min-w-[50px] text-[11px] opacity-60 text-right">{{ line.lineNum || '' }}</span>
              <span class="min-w-[80px] font-medium" :style="{ color: getServerColor(line.serverId) }">[{{ line.serverName }}]</span>
              <span
                class="log-line-text flex-1 whitespace-pre-wrap break-all"
                :class="{ 'text-error': line.level === 'error', 'text-warning': line.level === 'warn', 'text-base-content/40': line.level === 'debug' }"
                v-html="line.html ?? highlightSearchResult(line.content)"
              ></span>
            </div>
          </div>
        </div>

        <!-- 回到底部浮动按钮 -->
        <button
          v-if="queryMode === 'stream' && showScrollBottom"
          @click="scrollToBottom"
          class="btn btn-primary btn-sm rounded-full absolute bottom-4 right-4 z-10 shadow-lg hover:scale-105 hover:shadow-xl transition-all"
          title="回到底部"
        >
          <SvgIcon name="arrowDown" size="14" /> 回到底部
        </button>
      </div>
    </div>

    <!-- 预设表单弹窗 -->
    <div v-if="showPresetForm" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[1000]">
      <div class="bg-base-100 p-5 rounded-2xl w-[680px] max-h-[80vh] overflow-y-auto relative">
        <button @click="showPresetForm = false" class="absolute top-3 right-3 btn btn-ghost btn-sm btn-square rounded-full" title="关闭">
          <SvgIcon name="x" size="16" />
        </button>
        <h3 class="mt-0 mb-4 text-lg font-semibold">{{ editingPreset ? '编辑预设' : '新增预设' }}</h3>
        <div class="mb-3">
          <label class="block text-xs mb-1 text-base-content/60">预设名称</label>
          <input v-model="presetForm.name" placeholder="例如：API服务日志" class="input input-bordered w-full" />
        </div>
        <div class="mb-3">
          <label class="block text-xs mb-1 text-base-content/60">分组</label>
          <input
            v-model="presetForm.presetGroup"
            list="group-suggestions"
            placeholder="例如：生产 / 测试"
            class="input input-bordered w-full"
          />
          <datalist id="group-suggestions">
            <option value="生产" />
            <option value="测试" />
            <option value="开发" />
            <option value="预发" />
          </datalist>
        </div>
        <div class="mb-3">
          <label class="block text-xs mb-1 text-base-content/60">服务器</label>
          <GroupedServerSelector
            :servers="allServers"
            :groups="allGroups"
            v-model="presetForm.serverIds"
            mode="multi"
          />
        </div>
        <div class="mb-3">
          <label class="block text-xs mb-1 text-base-content/60">日志类型</label>
          <select v-model="presetForm.logType" class="select select-bordered w-full">
            <option value="file">文件 (tail)</option>
            <option value="journalctl">Journalctl</option>
            <option value="docker">Docker</option>
            <option value="custom">自定义</option>
          </select>
        </div>
        <div class="mb-3">
          <label class="block text-xs mb-1 text-base-content/60">日志路径 / 容器名</label>
          <textarea v-model="presetForm.logPath" placeholder="/var/log/app/api.log&#10;/var/log/app/error.log&#10;(每行一个路径)" class="textarea textarea-bordered w-full font-mono resize-y leading-relaxed min-h-[60px]" rows="3" />
        </div>
        <div class="mb-3">
          <label class="block text-xs mb-1 text-base-content/60">关键字（逗号分隔）</label>
          <input v-model="presetForm.keywordsInput" placeholder="ERROR, Exception" class="input input-bordered w-full" />
        </div>
        <div class="mb-3">
          <label class="block text-xs mb-1 text-base-content/60">初始行数</label>
          <input v-model.number="presetForm.maxLines" type="number" min="50" max="50000" class="input input-bordered w-20" />
        </div>
        <div class="flex justify-end gap-2 mt-5">
          <button @click="showPresetForm = false" class="btn btn-ghost">取消</button>
          <button @click="savePreset" class="btn btn-primary">保存</button>
        </div>
      </div>
    </div>
  </div>

    <!-- 确认删除对话框 -->
    <dialog ref="deleteConfirmDialog" class="modal">
      <div class="modal-box max-w-sm relative">
        <button @click="cancelDeletePreset" class="absolute top-3 right-3 btn btn-ghost btn-sm btn-square rounded-full" title="关闭">
          <SvgIcon name="x" size="16" />
        </button>
        <h3 class="text-lg font-bold flex items-center gap-2">
          <SvgIcon name="alertTriangle" size="18" class="text-warning" />
          <span>确认删除</span>
        </h3>
        <p class="py-3 text-sm">{{ deleteConfirmMessage }}</p>
        <div class="modal-action">
          <button class="btn btn-error btn-sm" @click="executeDeletePreset"><SvgIcon name="trash" size="14" /> 确认删除</button>
          <button class="btn btn-sm" @click="cancelDeletePreset">取消</button>
        </div>
      </div>
      <form method="dialog" class="modal-backdrop"><button @click="cancelDeletePreset">close</button></form>
    </dialog>

    <!-- 离线日志查看弹窗 -->
    <dialog ref="fullLogDialog" class="modal">
      <div class="modal-box max-w-[96vw] w-[1500px] h-[92vh] p-0 bg-[#1e1e1e] text-[#d4d4d4] flex flex-col">
        <div class="flex items-center gap-3 px-4 py-2.5 border-b border-white/10 bg-[#252526]">
          <SvgIcon name="fileText" size="16" class="text-primary" />
          <h3 class="text-sm font-semibold flex-1 truncate" :title="fullLogActiveSession ? `${fullLogActiveSession.serverName} · ${fullLogActiveSession.fileName}` : '离线日志'">
            {{ fullLogActiveSession ? `${fullLogActiveSession.serverName} · ${fullLogActiveSession.fileName}` : '离线日志' }}
          </h3>
          <div class="flex items-center gap-2">
            <input
              v-model="fullLogSearchKeyword"
              type="text"
              placeholder="搜索关键字（高亮匹配，可上下跳转）..."
              class="input input-bordered input-sm h-8 w-[300px] bg-[#3c3c3c] border-white/10 text-[#d4d4d4] text-xs placeholder:text-white/30"
              @keyup.enter="fullLogSearch"
            />
            <button @click="fullLogSearch" class="btn btn-primary btn-xs" title="搜索">搜索</button>
            <button @click="fullLogPrevMatch" :disabled="fullLogMatchLineNos.length === 0" class="btn btn-ghost btn-xs text-white/70 hover:text-white border border-white/10 disabled:opacity-30" title="上一个匹配"><SvgIcon name="chevronUp" size="12" /></button>
            <span class="text-[11px] text-white/60 min-w-[60px] text-center font-mono">
              {{ fullLogMatchLineNos.length === 0 ? '0/0' : `${fullLogCurrentMatchIndex + 1}/${fullLogMatchLineNos.length}` }}
            </span>
            <button @click="fullLogNextMatch" :disabled="fullLogMatchLineNos.length === 0" class="btn btn-ghost btn-xs text-white/70 hover:text-white border border-white/10 disabled:opacity-30" title="下一个匹配"><SvgIcon name="chevronDown" size="12" /></button>
            <button
              @click="copyFullLog"
              :disabled="fullLogCopying || !fullLogActiveSession?.localPath"
              class="btn btn-ghost btn-xs text-white/70 hover:text-white border border-white/10 disabled:opacity-30"
              title="复制完整日志（不受虚拟滚动限制，与选中复制不同）"
            >
              <SvgIcon v-if="!fullLogCopying" name="copy" size="12" />
              <span v-else class="loading loading-spinner loading-xs"></span>
              {{ fullLogCopying ? '复制中...' : '复制全部' }}
            </button>
            <button @click="closeFullLogDialog" class="btn btn-ghost btn-xs text-white/70 hover:text-white border border-white/10" title="关闭"><SvgIcon name="x" size="12" /> 关闭</button>
          </div>
        </div>
        <!-- 服务器节点 + 文件名 Tab 切换 -->
        <div v-if="fullLogSessions.length > 1" class="flex items-center gap-1 px-4 py-1 border-b border-white/10 bg-[#2d2d2d] overflow-x-auto">
          <button
            v-for="(s, idx) in fullLogSessions"
            :key="s.downloadId"
            @click="switchFullLogSession(idx)"
            :class="['btn btn-xs rounded whitespace-nowrap', idx === fullLogActiveIndex ? 'btn-primary' : 'btn-ghost text-white/60 hover:text-white']"
            :title="s.loadError ? s.loadError : `${s.serverName} · ${s.fileName}`"
          >
            <span class="opacity-90">{{ s.serverName }}</span>
            <span class="text-white/40 mx-0.5">·</span>
            <span class="text-[10px] opacity-80">{{ s.fileName }}</span>
            <span v-if="s.totalLines > 0" class="text-[10px] opacity-60 ml-1">({{ s.totalLines }})</span>
            <span v-if="s.loadError" class="text-error ml-1" :title="s.loadError">⚠</span>
            <span v-else-if="!s.downloaded && !fullLogLoading" class="text-warning ml-1">⏳</span>
          </button>
        </div>
        <div ref="fullLogContainer" class="flex-1 overflow-auto px-4 py-2 text-xs leading-relaxed font-mono">
          <!-- loading 遮罩：仅在任何节点展示前全屏显示 -->
          <div v-if="fullLogLoading" class="flex flex-col items-center justify-center h-full text-white/60 gap-4">
            <div class="flex items-center">
              <SvgIcon name="refresh" size="14" class="animate-spin inline-block mr-2" />
              <span>{{ fullLogLoadingText }}</span>
            </div>
            <!-- 下载进度面板 -->
            <div v-if="fullLogSessions.length > 0" class="w-[700px] max-w-[90%] bg-[#2d2d2d] rounded-lg p-4 border border-white/10">
              <!-- 总进度 -->
              <div class="mb-3">
                <div class="flex justify-between items-center mb-1.5">
                  <span class="text-[11px] text-white/70 font-medium">总进度</span>
                  <span class="text-[11px] text-white/60 font-mono">
                    {{ formatBytes(fullLogDownloadProgress.downloaded) }} / {{ formatBytes(fullLogDownloadProgress.total) }}
                    · {{ fullLogDownloadProgress.percent }}%
                    · {{ fullLogDownloadProgress.done }}/{{ fullLogDownloadProgress.count }} 完成
                  </span>
                </div>
                <div class="h-2 bg-[#1e1e1e] rounded-full overflow-hidden">
                  <div
                    class="h-full bg-primary transition-all duration-200 rounded-full"
                    :style="{ width: fullLogDownloadProgress.percent + '%' }"
                  ></div>
                </div>
              </div>
              <!-- 每个节点的进度 -->
              <div class="space-y-2 max-h-[300px] overflow-y-auto">
                <div v-for="s in fullLogSessions" :key="s.downloadId" class="flex items-center gap-2">
                  <span class="text-[11px] text-white/70 w-[200px] truncate" :title="`${s.serverName} · ${s.fileName}`">{{ s.serverName }} · {{ s.fileName }}</span>
                  <div class="flex-1 h-1.5 bg-[#1e1e1e] rounded-full overflow-hidden">
                    <div
                      class="h-full transition-all duration-200 rounded-full"
                      :class="{
                        'bg-green-500': s.downloadStatus === 'done',
                        'bg-red-500': s.downloadStatus === 'failed',
                        'bg-primary': s.downloadStatus === 'downloading' || s.downloadStatus === 'pending',
                      }"
                      :style="{ width: (s.downloadTotal > 0 ? Math.min(100, (s.downloadDownloaded / s.downloadTotal) * 100) : (s.downloadStatus === 'done' ? 100 : 0)) + '%' }"
                    ></div>
                  </div>
                  <span class="text-[10px] text-white/50 font-mono w-[140px] text-right">
                    <template v-if="s.downloadStatus === 'done'">完成 ({{ formatBytes(s.downloadTotal) }})</template>
                    <template v-else-if="s.downloadStatus === 'failed'" class="text-red-400">失败</template>
                    <template v-else-if="s.downloadTotal > 0">{{ formatBytes(s.downloadDownloaded) }} / {{ formatBytes(s.downloadTotal) }}</template>
                    <template v-else>等待中...</template>
                  </span>
                </div>
              </div>
            </div>
          </div>
          <div v-else-if="fullLogError" class="flex items-center justify-center h-full text-error text-sm">
            <SvgIcon name="alertTriangle" size="16" class="mr-2" /> {{ fullLogError }}
          </div>
          <div v-else-if="fullLogTotalLines === 0" class="flex items-center justify-center h-full text-white/40">
            日志为空
          </div>
          <div v-else :style="{ overflowAnchor: 'none' }">
            <!-- 首节点已显示、其余节点仍在后台下载：日志顶部紧凑进度条（不遮挡日志） -->
            <div v-if="fullLogHasPending" class="sticky top-0 z-10 -mx-4 px-4 py-1.5 bg-[#252526]/95 border-b border-white/10 flex items-center gap-2">
              <SvgIcon name="refresh" size="12" class="animate-spin text-primary inline-block" />
              <span class="text-[11px] text-white/70">正在下载剩余节点...</span>
              <span class="text-[11px] text-white/50 font-mono">{{ fullLogDownloadProgress.done }}/{{ fullLogDownloadProgress.count }} 完成 · {{ fullLogDownloadProgress.percent }}%</span>
            </div>
            <div :style="{ height: fullLogPrefixAt(fullLogVisibleStart) + 'px' }"></div>
            <div
              v-for="item in fullLogVisibleLines"
              :key="item.lineNo"
              :data-line-no="item.lineNo"
              class="flex gap-2 py-0.5 hover:bg-white/5 rounded"
              :class="{ 'bg-yellow-500/20 border-l-2 border-yellow-400': fullLogCurrentMatchLineNo === item.lineNo }"
            >
              <span class="text-white/30 select-none w-[60px] text-right flex-shrink-0">{{ item.lineNo + 1 }}</span>
              <span
                class="flex-1 whitespace-pre-wrap break-all"
                v-html="item.html"
              ></span>
            </div>
            <div :style="{ height: (fullLogPrefixAt(fullLogTotalLines) - fullLogPrefixAt(fullLogVisibleEnd)) + 'px' }"></div>
          </div>
        </div>
        <div class="flex items-center justify-between px-4 py-1.5 border-t border-white/10 bg-[#252526] text-[11px] text-white/50">
          <span class="truncate" :title="fullLogActiveSession?.localPath">{{ fullLogActiveSession ? `${fullLogActiveSession.serverName} · ${fullLogActiveSession.fileName}` : '' }} <span class="text-white/30">· {{ fullLogActiveSession?.localPath || '' }}</span></span>
          <span v-if="fullLogTotalLines > 0">{{ fullLogTotalLines }} 行{{ fullLogMatchLineNos.length > 0 ? ` · ${fullLogMatchLineNos.length} 个匹配` : '' }}</span>
        </div>
      </div>
      <form method="dialog" class="modal-backdrop"><button @click="closeFullLogDialog">close</button></form>
    </dialog>

    <!-- 离线查看入口选择：实时日志 / 历史日志 -->
    <dialog ref="fullLogEntryDialog" class="modal">
      <div class="modal-box max-w-md relative">
        <button @click="fullLogEntryDialog?.close()" class="absolute top-3 right-3 btn btn-ghost btn-sm btn-square rounded-full" title="关闭">
          <SvgIcon name="x" size="16" />
        </button>
        <h3 class="mt-0 mb-2 text-lg font-semibold flex items-center gap-2">
          <SvgIcon name="fileText" size="18" class="text-primary" />
          <span>离线查看</span>
        </h3>
        <p class="text-sm text-base-content/70 mb-4">请选择日志类型：</p>
        <div class="grid grid-cols-2 gap-3">
          <button @click="startRealtimeFullLog" class="btn btn-primary btn-outline h-[100px] flex flex-col gap-2">
            <SvgIcon name="refresh" size="22" />
            <span class="text-sm font-medium">实时日志</span>
            <span class="text-[11px] opacity-70">按预设路径下载当前文件</span>
          </button>
          <button @click="startHistoricalFullLog" class="btn btn-primary btn-outline h-[100px] flex flex-col gap-2">
            <SvgIcon name="fileText" size="22" />
            <span class="text-sm font-medium">历史日志</span>
            <span class="text-[11px] opacity-70">浏览服务器选择历史文件</span>
          </button>
        </div>
        <div v-if="selectedPreset" class="mt-4 text-[11px] text-base-content/60 bg-base-200 rounded p-2">
          <div>预设：{{ selectedPreset.name }}</div>
          <div>节点数：{{ selectedPreset.serverIds?.length || 0 }}</div>
        </div>
      </div>
      <form method="dialog" class="modal-backdrop"><button>close</button></form>
    </dialog>

    <!-- 历史日志文件选择器（SFTP 浏览 + 多选 + 子目录导航） -->
    <dialog ref="fullLogFilePickerDialog" class="modal">
      <div class="modal-box max-w-[96vw] w-[1100px] h-[80vh] p-0 flex flex-col">
        <!-- 顶部：标题 + 路径导航 -->
        <div class="flex items-center gap-2 px-4 py-2.5 border-b border-base-content/10">
          <SvgIcon name="folder" size="16" class="text-primary" />
          <h3 class="text-sm font-semibold flex-shrink-0">选择历史日志文件</h3>
          <button
            @click="filePickerGoUp"
            :disabled="filePickerPathStack.length === 0 || filePickerLoading"
            class="btn btn-ghost btn-xs border border-base-content/10 disabled:opacity-30"
            title="返回上一级"
          >
            <SvgIcon name="chevronUp" size="12" /> 上一级
          </button>
          <div class="flex-1 flex items-center gap-1">
            <input
              v-model="filePickerPathInput"
              @keyup.enter="filePickerLoadFromInput"
              :disabled="filePickerLoading"
              placeholder="输入路径后回车"
              class="input input-bordered input-sm h-8 flex-1 font-mono text-xs"
            />
            <button @click="filePickerLoadFromInput" :disabled="filePickerLoading" class="btn btn-primary btn-xs">前往</button>
          </div>
          <button @click="cancelFilePicker" class="btn btn-ghost btn-xs btn-square" title="关闭">
            <SvgIcon name="x" size="14" />
          </button>
        </div>
        <!-- 当前路径 + 已选文件数 -->
        <div class="flex items-center justify-between px-4 py-1.5 border-b border-base-content/10 text-[11px] text-base-content/60 bg-base-200">
          <span class="truncate font-mono" :title="filePickerCurrentPath">当前目录：{{ filePickerCurrentPath }}</span>
          <span v-if="filePickerSelected.length > 0" class="text-primary font-medium whitespace-nowrap">已选 {{ filePickerSelected.length }} 个文件</span>
        </div>
        <!-- 文件列表 -->
        <div class="flex-1 overflow-y-auto">
          <div v-if="filePickerLoading" class="flex items-center justify-center h-full text-base-content/60 text-sm">
            <SvgIcon name="refresh" size="16" class="animate-spin mr-2" /> 加载中...
          </div>
          <div v-else-if="filePickerError" class="flex items-center justify-center h-full text-error text-sm">
            <SvgIcon name="alertTriangle" size="16" class="mr-2" /> {{ filePickerError }}
          </div>
          <div v-else-if="filePickerFiles.length === 0" class="flex items-center justify-center h-full text-base-content/40 text-sm">
            目录为空
          </div>
          <table v-else class="table table-xs table-zebra w-full">
            <thead class="sticky top-0 bg-base-200 text-[11px] text-base-content/60">
              <tr>
                <th class="w-[40px]"></th>
                <th>名称</th>
                <th class="w-[100px] text-right">大小</th>
                <th class="w-[160px]">修改时间</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="entry in filePickerFiles"
                :key="entry.path"
                @click="entry.isDir ? filePickerEnterDir(entry) : filePickerToggleSelect(entry)"
                :class="['cursor-pointer hover:bg-primary/10', !entry.isDir && isFilePickerSelected(entry) ? 'bg-primary/20' : '']"
              >
                <td class="text-center">
                  <input
                    v-if="!entry.isDir"
                    type="checkbox"
                    :checked="isFilePickerSelected(entry)"
                    @click.stop="filePickerToggleSelect(entry)"
                    class="checkbox checkbox-xs checkbox-primary"
                  />
                  <SvgIcon v-else name="folder" size="14" class="inline text-warning" />
                </td>
                <td class="font-mono text-xs">
                  <span v-if="entry.isDir" class="text-warning font-medium">{{ entry.name }}/</span>
                  <span v-else>
                    {{ entry.name }}
                    <span v-if="entry.isGz" class="badge badge-xs badge-primary ml-1">gz</span>
                  </span>
                </td>
                <td class="text-right font-mono text-[11px] text-base-content/60">
                  {{ entry.isDir ? '-' : formatBytes(entry.size) }}
                </td>
                <td class="font-mono text-[11px] text-base-content/60">
                  {{ entry.modifyTime ? new Date(entry.modifyTime).toLocaleString() : '-' }}
                </td>
              </tr>
            </tbody>
          </table>
        </div>
        <!-- 底部：已选文件列表 + 确认按钮 -->
        <div class="border-t border-base-content/10 px-4 py-2 bg-base-200">
          <div v-if="filePickerSelected.length > 0" class="mb-2 flex flex-wrap gap-1 max-h-[80px] overflow-y-auto">
            <span
              v-for="f in filePickerSelected"
              :key="f.path"
              class="badge badge-sm badge-primary gap-1 cursor-pointer"
              @click="filePickerToggleSelect(f)"
              :title="`移除 ${f.name}`"
            >
              {{ f.name }}
              <SvgIcon name="x" size="10" />
            </span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-[11px] text-base-content/60">
              <SvgIcon name="info" size="11" class="inline" />
              所选文件将从所有服务器节点下载（.gz 自动解压）
            </span>
            <div class="flex gap-2">
              <button @click="cancelFilePicker" class="btn btn-ghost btn-sm">取消</button>
              <button
                @click="confirmFilePickerSelection"
                :disabled="filePickerSelected.length === 0"
                class="btn btn-primary btn-sm"
              >
                <SvgIcon name="download" size="14" /> 下载并查看 ({{ filePickerSelected.length }})
              </button>
            </div>
          </div>
        </div>
      </div>
      <form method="dialog" class="modal-backdrop"><button @click="cancelFilePicker">close</button></form>
    </dialog>
</template>

<script setup lang="ts">
defineOptions({ name: 'LogAggregator' })
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref, computed, onMounted, onUnmounted, onActivated, onDeactivated, nextTick, watch } from 'vue'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { getTauriAPI } from '../../utils/tauri-api'
import { useToast } from '../../composables/useToast'
import type { Server, ServerGroup } from '../../types'
import GroupedServerSelector from '../server/GroupedServerSelector.vue'

const toast = useToast()

// 状态
const presets = ref<any[]>([])
// 首屏预设加载中：左栏显示骨架行而不是空白
const presetsLoading = ref(true)
const allServers = ref<Server[]>([])
const allGroups = ref<ServerGroup[]>([])
const selectedPreset = ref<any | null>(null)
const logLines = ref<Array<{ id: string; serverId: string; serverName: string; timestamp: number; content: string; level: string; isMatch?: boolean; matched?: boolean; lineNum?: string; html?: string; sortKey?: number }>>([])
const isStreaming = ref(false)
const followMode = ref(true)
const userScrolledUp = ref(false)
const activeServers = ref(new Set<string>())
// 节点筛选：null=聚合全部，否则只看某个 serverId（流式 + 搜索通用）
const selectedServerFilter = ref<string | null>(null)
// 全屏模式：fixed 覆盖整个窗口，隐藏左侧预设栏
const isFullscreen = ref(false)

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value
  // 全屏切换后容器尺寸变化，需等 DOM 更新后重新测量高度并刷新虚拟滚动
  nextTick(() => {
    if (logContainer.value) {
      containerHeight.value = logContainer.value.clientHeight
      // 流式跟随模式自动吸底，否则保持当前位置
      if (followMode.value) {scrollToBottomSilent()}
    }
  })
}

// Esc 退出全屏
function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && isFullscreen.value) {
    isFullscreen.value = false
    nextTick(() => {
      if (logContainer.value) {
        containerHeight.value = logContainer.value.clientHeight
        if (followMode.value) {scrollToBottomSilent()}
      }
    })
  }
}
const streamId = ref('')
const logContainer = ref<HTMLElement | null>(null)
// 计数器替代布尔标志：多个程序化滚动操作并发时（如 scrollToBottom + loadMoreHistory 同时触发）
// 不会互相覆盖重置，只要还有未完成的程序化滚动，onScroll 就忽略
let scrollingFromRAFCount = 0
let pendingScroll = false

// 查询模式
const queryMode = ref<'stream' | 'search'>('stream')

// 搜索模式状态
const searchKeyword = ref('')
const searchContextLines = ref(10)
const isSearching = ref(false)
const hasSearched = ref(false)
// 搜索导航
const matchIndices = ref<number[]>([])
const currentMatchIndex = ref(-1)
const currentMatchId = ref<string | null>(null)
// 加载更多历史日志
const loadingMore = ref(false)

// 下载状态
// 下载状态（已移除导出/下载日志功能，isDownloadingLog 保留声明避免其他引用报错）
const isDownloadingLog = ref(false)

// ── 离线日志查看（多服务器节点，后端按需返回 HTML，前端只渲染） ──
const fullLogDialog = ref<HTMLDialogElement | null>(null)
const fullLogLoading = ref(false)
const fullLogLoadingText = ref('')
const fullLogError = ref('')
const fullLogSearchKeyword = ref('')
const fullLogCopying = ref(false)

// 复制完整日志：直接从本地文件读取写入剪贴板。
// 虚拟滚动 DOM 只渲染可视区，浏览器选中复制只能拿到可视区内容——必须绕过 DOM。
async function copyFullLog() {
  const s = fullLogActiveSession.value
  if (!s?.localPath || fullLogCopying.value) {return}
  fullLogCopying.value = true
  try {
    const content = await getTauriAPI().readLogCacheFile(s.localPath)
    if (!content) {toast.warning('日志内容为空');return}
    await navigator.clipboard.writeText(content)
    toast.success(`已复制 ${content.length.toLocaleString()} 字符（${(content.length / 1024 / 1024).toFixed(1)}MB）`)
  } catch (e: any) {
    toast.error('复制失败: ' + (e?.message || String(e)))
  } finally {
    fullLogCopying.value = false
  }
}
const fullLogContainer = ref<HTMLElement | null>(null)
const FULL_LOG_LINE_HEIGHT = 18
const FULL_LOG_OVERSCAN = 30
const FULL_LOG_BATCH = 300   // 每次后端请求拉 300 行
const FULL_LOG_LOAD_STEP = 150  // loadStart 对齐步长，避免微小滚动触发重复加载
const FULL_LOG_CACHE_LIMIT = 5000  // LRU 缓存上限
let _fullLogResizeHandler: (() => void) | null = null
let _fullLogScrollHandler: (() => void) | null = null
// 跳转进行中标志：阻止 scroll handler 在用户未操作时清除跳转锚点
let _fullLogJumping = false
let _fullLogRafId = 0
// 离线查看调用代数：新一次 downloadAndShowLogs / 关闭弹窗时自增，
// 使旧调用的异步回调（下载完成、进度、激活）不再污染新视图
let _fullLogGen = 0
// 本次离线查看是否已展示首个成功节点（只展示一次，其余节点后台预读）
let _fullLogFirstShown = false
// 首个节点激活进行中标志：多个节点同时完成下载时，防止并发重复激活
let _fullLogActivating = false

// 单个服务器节点的离线日志会话状态
interface FullLogSession {
  serverId: string
  serverName: string
  fileName: string
  remotePath: string
  localPath: string
  totalLines: number
  cache: Map<number, { lineNo: number; html: string }>
  // 逐行真实行高（未测量行用默认 FULL_LOG_LINE_HEIGHT 估算，滚动经过时采样填充）
  rowHeights: number[]
  scrollTop: number
  lastLoadRange: { start: number; end: number }
  loadingPromise: Promise<void> | null
  // vim 搜索状态（每个 session 独立）
  matchLineNos: number[]
  currentMatchIndex: number
  currentMatchLineNo: number
  loadError: string
  downloaded: boolean  // 是否已下载完成
  // 下载进度状态
  downloadId: string
  downloadTotal: number  // 文件总字节数
  downloadDownloaded: number  // 已下载字节数
  downloadStatus: 'pending' | 'downloading' | 'done' | 'failed'
}

// 所有会话（按服务器顺序）
const fullLogSessions = ref<FullLogSession[]>([])
// 当前激活的会话索引
const fullLogActiveIndex = ref(0)
// 当前激活的会话（computed）
const fullLogActiveSession = computed<FullLogSession | null>(() => fullLogSessions.value[fullLogActiveIndex.value] ?? null)

// 是否还有节点在下载（首节点激活后进度面板仍需保持可见）
const fullLogHasPending = computed(() =>
  fullLogSessions.value.some(s => s.downloadStatus === 'downloading' || s.downloadStatus === 'pending')
)

// 整体下载进度汇总（所有 session 合计）
const fullLogDownloadProgress = computed(() => {
  const sessions = fullLogSessions.value
  if (sessions.length === 0) {return { total: 0, downloaded: 0, percent: 0, active: 0, done: 0, failed: 0 }}
  let total = 0, downloaded = 0, done = 0, failed = 0, active = 0
  for (const s of sessions) {
    if (s.downloadTotal > 0) {
      total += s.downloadTotal
      downloaded += s.downloadDownloaded
    }
    if (s.downloadStatus === 'done') {done++}
    else if (s.downloadStatus === 'failed') {failed++}
    else if (s.downloadStatus === 'downloading') {active++}
  }
  const percent = total > 0 ? Math.min(100, Math.round((downloaded / total) * 100)) : 0
  return { total, downloaded, percent, active, done, failed, count: sessions.length }
})

// 下载进度事件监听器（unlisten 函数）
let _downloadProgressUnlisten: UnlistenFn | null = null

// ===== 历史日志文件选择对话框 =====
// viewFullRemoteLog 入口选择对话框：实时日志 / 历史日志
const fullLogEntryDialog = ref<HTMLDialogElement | null>(null)
// 历史日志文件选择对话框
const fullLogFilePickerDialog = ref<HTMLDialogElement | null>(null)
// 文件选择器当前服务器 ID
const filePickerServerId = ref<string>('')
// 文件选择器当前路径
const filePickerCurrentPath = ref<string>('')
// 文件选择器路径输入框值（支持手动输入）
const filePickerPathInput = ref<string>('')
// 文件选择器加载状态
const filePickerLoading = ref(false)
// 文件选择器错误信息
const filePickerError = ref('')
// 文件选择器当前目录的文件列表
interface RemoteFileEntry {
  name: string
  path: string  // 完整路径
  isDir: boolean
  size: number
  modifyTime: string
  isGz: boolean
}
const filePickerFiles = ref<RemoteFileEntry[]>([])
// 已选中的文件（按服务器分组，因为不同服务器都要下载同一相对路径）
const filePickerSelected = ref<RemoteFileEntry[]>([])
// 文件选择器路径历史栈（用于"上一级"导航）
const filePickerPathStack = ref<string[]>([])

// 以下 ref 都是"当前激活 session"的视图代理，便于模板绑定
const fullLogScrollTop = ref(0)
const fullLogContainerHeight = ref(0)
const fullLogTotalLines = ref(0)
const fullLogVisibleLines = ref<Array<{ lineNo: number; html: string }>>([])
const fullLogMatchLineNos = ref<number[]>([])
const fullLogCurrentMatchIndex = ref(-1)
const fullLogCurrentMatchLineNo = ref(-1)
let _fullLogFindingMatches = false
let _fullLogCurrentKeyword = ''    // 当前生效的关键字（全局共享，搜索时对所有 session 生效）

// 跳转锚点：当设置时，visibleStart/End 直接以该行为中心计算（而非 scrollTop 估算行高反推），
// 确保跳转目标行一定落在渲染窗口内。这是修复"跳转前几次正常、之后失效"的关键——
// 真实行高因换行可变，scrollTop/18 反推行号会随累积偏差越来越大而错位。
const fullLogAnchorLine = ref(-1)
// 跳转后用户手动滚动时清除锚点，恢复 scrollTop 驱动的虚拟滚动
function clearFullLogAnchor() { fullLogAnchorLine.value = -1 }

// 当前激活 session 的行高前缀和：prefix[i] = Σ rowHeights[0..i)，未测量行按默认行高估算。
// 与 scrollTop→行号二分反推共用，spacer 撑高逐步贴合真实内容高度，消除固定行高虚拟滚动回弹。
// 性能：computed 惰性求值，仅在测量批次写入后重建一次（O(totalLines)，10 万行约 1ms），滚动本身只做二分 O(log n)。
const fullLogHeightPrefix = computed(() => {
  const session = fullLogActiveSession.value
  const total = session ? session.totalLines : 0
  const prefix: number[] = Array.from({ length: total + 1 }, () => 0)
  prefix[0] = 0
  if (total > 0) {
    const h = session?.rowHeights ?? []
    for (let i = 0; i < total; i++) {
      prefix[i + 1] = prefix[i] + (h[i] ?? FULL_LOG_LINE_HEIGHT)
    }
  }
  return prefix
})

// scrollTop 落在哪一行（二分前缀和），供 visibleStart/End 反推渲染窗口
function fullLogRowAtScrollTop(): number {
  const prefix = fullLogHeightPrefix.value
  const st = fullLogScrollTop.value
  if (prefix.length === 0) {return 0}
  if (st <= 0) {return 0}
  let lo = 0, hi = prefix.length - 1
  while (lo < hi) {
    const mid = (lo + hi) >> 1
    if (prefix[mid] <= st) { lo = mid + 1 } else { hi = mid }
  }
  // lo 是第一个 prefix[lo] > st 的位置；scrollTop 落在行 lo-1
  if (prefix[lo] <= st) {return Math.max(0, prefix.length - 2)}
  return Math.max(0, lo - 1)
}

// 行号 → 前缀高度（spacer 撑高用，越界钳制到总高）
function fullLogPrefixAt(row: number): number {
  if (row <= 0) {return 0}
  const prefix = fullLogHeightPrefix.value
  return prefix[Math.min(row, prefix.length - 1)] ?? 0
}

const fullLogVisibleStart = computed(() => {
  if (fullLogAnchorLine.value >= 0) {
    return Math.max(0, fullLogAnchorLine.value - FULL_LOG_OVERSCAN - 5)
  }
  return Math.max(0, fullLogRowAtScrollTop() - FULL_LOG_OVERSCAN)
})
const fullLogVisibleEnd = computed(() => {
  if (fullLogAnchorLine.value >= 0) {
    // 锚点模式：用前缀和精确计算"目标行 + 容器高度"覆盖到的行号（真实行高可变的固定 18px 估算
    // 会低估窗口导致目标行下方渲染不足），再保底至少 anchor + overscan*2 行
    const anchor = fullLogAnchorLine.value
    const prefix = fullLogHeightPrefix.value
    const targetBottom = fullLogPrefixAt(anchor) + fullLogContainerHeight.value
    let lo = anchor, hi = prefix.length - 1
    while (lo < hi) {
      const mid = (lo + hi) >> 1
      if (prefix[mid] <= targetBottom) { lo = mid + 1 } else { hi = mid }
    }
    const endRow = prefix[lo] <= targetBottom ? prefix.length - 2 : lo - 1
    const minEnd = Math.min(fullLogTotalLines.value, anchor + FULL_LOG_OVERSCAN * 2 + 10)
    return Math.max(endRow, minEnd)
  }
  // 视口底边所在行（二分前缀和），再加 overscan
  const prefix = fullLogHeightPrefix.value
  const bottom = fullLogScrollTop.value + fullLogContainerHeight.value
  let lo = 0, hi = prefix.length - 1
  while (lo < hi) {
    const mid = (lo + hi) >> 1
    if (prefix[mid] <= bottom) { lo = mid + 1 } else { hi = mid }
  }
  const endRow = prefix[lo] <= bottom ? prefix.length - 2 : lo - 1
  return Math.min(fullLogTotalLines.value, endRow + FULL_LOG_OVERSCAN + 1)
})

// 可见行渲染完成后采样真实行高写入 session.rowHeights，前缀和随之精确，
// spacer 逐步贴合真实内容高度（每行只在首次进入视口时测量一次，滚动中渐进收敛）
watch(fullLogVisibleLines, () => {
  nextTick(() => {
    const container = fullLogContainer.value
    const session = fullLogActiveSession.value
    if (!container || !session?.rowHeights) {return}
    const rows = container.querySelectorAll('[data-line-no]')
    for (const r of rows) {
      const el = r as HTMLElement
      const lineNo = Number(el.dataset.lineNo)
      const h = el.offsetHeight
      if (h > 0 && session.rowHeights[lineNo] !== h) {
        session.rowHeights[lineNo] = h
      }
    }
  })
})

// 从激活 session 的 cache 中拉取当前可见行段（同步操作，无 IO）
function refreshVisibleLines() {
  const session = fullLogActiveSession.value
  if (!session) {
    fullLogVisibleLines.value = []
    return
  }
  const start = fullLogVisibleStart.value
  const end = fullLogVisibleEnd.value
  if (end <= start || session.totalLines === 0) {
    fullLogVisibleLines.value = []
    return
  }
  const arr: Array<{ lineNo: number; html: string }> = []
  for (let i = start; i < end; i++) {
    const row = session.cache.get(i)
    if (row !== null && row !== undefined) {
      arr.push(row)
    }
  }
  fullLogVisibleLines.value = arr
}

// 检查可见区间是否在 cache 中，若不足则触发后端加载
async function ensureVisibleRangeLoaded() {
  const session = fullLogActiveSession.value
  if (!session || !session.localPath) {return}
  const start = fullLogVisibleStart.value
  const end = fullLogVisibleEnd.value
  const knownTotal = session.totalLines

  // 强检查：如果可见区间全部在 cache 里，直接刷新显示，绝对不发请求
  if (knownTotal > 0 && end > start) {
    let allCached = true
    for (let i = start; i < end; i++) {
      if (!session.cache.has(i)) { allCached = false; break }
    }
    if (allCached) {
      refreshVisibleLines()
      return
    }
  }

  // 避免并发请求：若已有进行中的加载，等它完成后再决定是否需要补充加载
  if (session.loadingPromise) {
    await session.loadingPromise
    // 加载完成后再次检查：可能这次加载的区间已覆盖目标区间
    if (end > start) {
      let allCached = true
      for (let i = start; i < end; i++) {
        if (!session.cache.has(i)) { allCached = false; break }
      }
      if (allCached) {
        refreshVisibleLines()
        return
      }
    }
  }

  // 计算需要加载的区间，loadStart 对齐到 FULL_LOG_LOAD_STEP 步长
  const loadStart = Math.max(0, Math.floor((start - Math.floor(FULL_LOG_BATCH / 3)) / FULL_LOG_LOAD_STEP) * FULL_LOG_LOAD_STEP)
  const loadCount = Math.min(
    FULL_LOG_BATCH,
    knownTotal > 0 ? knownTotal - loadStart : FULL_LOG_BATCH
  )
  if (loadCount <= 0) {return}

  // 防止重复加载同一区间
  if (session.lastLoadRange.start === loadStart && session.lastLoadRange.end === loadStart + loadCount) {
    return
  }
  session.lastLoadRange = { start: loadStart, end: loadStart + loadCount }

  session.loadingPromise = (async () => {
    try {
      const result = await getTauriAPI().readLogFileLines(
        session.localPath,
        loadStart,
        loadCount,
        _fullLogCurrentKeyword || undefined
      )
      // 清理超出缓存上限的旧条目
      if (session.cache.size > FULL_LOG_CACHE_LIMIT) {
        const keys = Array.from(session.cache.keys()).sort((a, b) => a - b)
        for (let i = 0; i < 1000 && i < keys.length; i++) {
          session.cache.delete(keys[i])
        }
      }
      for (let i = 0; i < result.lines.length; i++) {
        session.cache.set(result.start + i, {
          lineNo: result.lines[i].lineNo,
          html: result.lines[i].html,
        })
      }
      if (result.totalLines !== session.totalLines) {
        session.totalLines = result.totalLines
        if (fullLogActiveSession.value === session) {
          fullLogTotalLines.value = result.totalLines
        }
      }
      if (fullLogActiveSession.value === session) {
        refreshVisibleLines()
      }
    } catch (e: any) {
      console.error('[LogAggregator] ensureVisibleRangeLoaded failed:', e)
      session.loadError = e.message || String(e)
      if (fullLogActiveSession.value === session) {
        fullLogError.value = session.loadError
      }
    } finally {
      session.loadingPromise = null
    }
  })()
  await session.loadingPromise
}

// 监听可见区间变化，触发按需加载
watch([fullLogVisibleStart, fullLogVisibleEnd], () => {
  if (!fullLogContainer.value) {return}
  ensureVisibleRangeLoaded()
})

// 将激活 session 的状态同步到视图代理 ref
function syncActiveSessionToView() {
  // 切换 session 时清除跳转锚点和标志，恢复正常虚拟滚动
  _fullLogJumping = false
  clearFullLogAnchor()
  const session = fullLogActiveSession.value
  if (!session) {
    fullLogScrollTop.value = 0
    fullLogTotalLines.value = 0
    fullLogVisibleLines.value = []
    fullLogMatchLineNos.value = []
    fullLogCurrentMatchIndex.value = -1
    fullLogCurrentMatchLineNo.value = -1
    fullLogError.value = ''
    return
  }
  fullLogTotalLines.value = session.totalLines
  fullLogMatchLineNos.value = session.matchLineNos
  fullLogCurrentMatchIndex.value = session.currentMatchIndex
  fullLogCurrentMatchLineNo.value = session.currentMatchLineNo
  fullLogError.value = session.loadError
  // 恢复滚动位置
  if (fullLogContainer.value) {
    fullLogContainer.value.scrollTop = session.scrollTop
  }
  fullLogScrollTop.value = session.scrollTop
  refreshVisibleLines()
  // 切换后需要确保新 session 的可见区间已加载
  nextTick(() => ensureVisibleRangeLoaded())
}

// 切换到指定 session
function switchFullLogSession(idx: number) {
  if (idx < 0 || idx >= fullLogSessions.value.length) {return}
  if (idx === fullLogActiveIndex.value) {return}
  // 保存当前 session 的滚动位置
  const cur = fullLogActiveSession.value
  if (cur && fullLogContainer.value) {
    cur.scrollTop = fullLogContainer.value.scrollTop
  }
  fullLogActiveIndex.value = idx
  syncActiveSessionToView()
}

// 关键字变化时：不再自动触发（改用 fullLogSearch 按钮手动触发，避免输入过程频繁扫描全文）
// 仅清空旧的匹配状态（所有 session 都清）
watch(fullLogSearchKeyword, (newKw) => {
  const trimmed = newKw.trim()
  if (trimmed === _fullLogCurrentKeyword) {return}
  fullLogMatchLineNos.value = []
  fullLogCurrentMatchIndex.value = -1
  fullLogCurrentMatchLineNo.value = -1
  for (const s of fullLogSessions.value) {
    s.matchLineNos = []
    s.currentMatchIndex = -1
    s.currentMatchLineNo = -1
  }
})

// vim 式搜索：对当前激活 session 调用后端扫描全文获取匹配行号，清 cache 重新加载（带高亮）
async function fullLogSearch() {
  const session = fullLogActiveSession.value
  if (!session || !session.localPath) {return}
  const trimmed = fullLogSearchKeyword.value.trim()
  if (!trimmed) {
    // 清空搜索：恢复无高亮状态
    if (_fullLogCurrentKeyword !== '') {
      _fullLogCurrentKeyword = ''
      for (const s of fullLogSessions.value) {
        s.cache.clear()
        s.lastLoadRange = { start: -1, end: -1 }
        s.matchLineNos = []
        s.currentMatchIndex = -1
        s.currentMatchLineNo = -1
      }
      fullLogMatchLineNos.value = []
      fullLogCurrentMatchIndex.value = -1
      fullLogCurrentMatchLineNo.value = -1
      fullLogVisibleLines.value = []
      ensureVisibleRangeLoaded()
    }
    return
  }
  if (trimmed === _fullLogCurrentKeyword && session.matchLineNos.length > 0) {
    // 同样的关键字，跳到第一个匹配
    fullLogJumpToMatch(0)
    return
  }
  if (_fullLogFindingMatches) {return}
  _fullLogFindingMatches = true
  fullLogLoadingText.value = '正在搜索匹配...'
  try {
    // 对所有已下载的 session 并行搜索
    const sessions = fullLogSessions.value.filter(s => s.downloaded && s.localPath)
    const results = await Promise.all(
      sessions.map(async s => {
        try {
          const matchLineNos = await getTauriAPI().findLogMatches(s.localPath, trimmed)
          return { session: s, matchLineNos }
        } catch (e) {
          return { session: s, matchLineNos: [] }
        }
      })
    )
    _fullLogCurrentKeyword = trimmed
    for (const { session: s, matchLineNos } of results) {
      s.matchLineNos = matchLineNos
      s.currentMatchIndex = matchLineNos.length > 0 ? 0 : -1
      s.currentMatchLineNo = matchLineNos.length > 0 ? matchLineNos[0] : -1
      // 清 cache，重新加载带高亮的 HTML
      s.cache.clear()
      s.lastLoadRange = { start: -1, end: -1 }
    }
    // 同步当前 session 到视图
    const cur = fullLogActiveSession.value
    if (cur) {
      fullLogMatchLineNos.value = cur.matchLineNos
      fullLogCurrentMatchIndex.value = cur.currentMatchIndex
      fullLogCurrentMatchLineNo.value = cur.currentMatchLineNo
    }
    if (cur && cur.matchLineNos.length > 0) {
      fullLogJumpToMatch(0)
    } else {
      toast.info('当前节点未找到匹配')
      ensureVisibleRangeLoaded()
    }
  } catch (e: any) {
    console.error('[LogAggregator] fullLogSearch failed:', e)
    toast.error('搜索失败: ' + (e.message || String(e)))
  } finally {
    _fullLogFindingMatches = false
    fullLogLoadingText.value = ''
  }
}

// 跳转到第 idx 个匹配（滚动到对应行，触发按需加载）
// 跳转到第 idx 个匹配（滚动到对应行，触发按需加载）
function fullLogJumpToMatch(idx: number) {
  const session = fullLogActiveSession.value
  if (!session) {return}
  if (idx < 0 || idx >= session.matchLineNos.length) {return}
  const targetLineNo = session.matchLineNos[idx]
  session.currentMatchIndex = idx
  session.currentMatchLineNo = targetLineNo
  fullLogCurrentMatchIndex.value = idx
  fullLogCurrentMatchLineNo.value = targetLineNo
  if (!fullLogContainer.value) {return}

  const container = fullLogContainer.value

  // 核心修复：进入锚点模式，让 visibleStart/End 直接以目标行为中心计算，
  // 而非 scrollTop/估算行高反推行号（真实行高可变会导致累积错位）。
  _fullLogJumping = true
  fullLogAnchorLine.value = targetLineNo

  // 用真实 DOM 位置精确居中目标行
  const centerElement = (el: HTMLElement): void => {
    const containerRect = container.getBoundingClientRect()
    const elRect = el.getBoundingClientRect()
    const elTopInContent = container.scrollTop + (elRect.top - containerRect.top)
    const newScrollTop = Math.max(0, elTopInContent - container.clientHeight / 2 + elRect.height / 2)
    container.scrollTop = newScrollTop
    fullLogScrollTop.value = newScrollTop
    if (session) {session.scrollTop = newScrollTop}
  }

  const doCenter = () => {
    // 锚点模式下 visibleStart/End 已确保目标行在渲染窗口内，
    // 直接刷新渲染并居中。重试等待 watch 触发的 ensureVisibleRangeLoaded 完成。
    const tryCenter = (attempts: number): void => {
      nextTick(() => {
        const el = container.querySelector(`[data-line-no="${targetLineNo}"]`) as HTMLElement | null
        if (el) {
          centerElement(el)
          // 居中完成，退出跳转模式。此后用户滚动会清除锚点，恢复正常虚拟滚动。
          _fullLogJumping = false
        } else if (attempts > 0) {
          setTimeout(() => tryCenter(attempts - 1), 30)
        } else {
          // 兜底：cache 可能尚未加载目标行，强制刷新可见区间后重试
          ensureVisibleRangeLoaded().then(() => {
            nextTick(() => {
              const el2 = container.querySelector(`[data-line-no="${targetLineNo}"]`) as HTMLElement | null
              if (el2) {centerElement(el2)}
              _fullLogJumping = false
            })
          }).catch(() => { _fullLogJumping = false })
        }
      })
    }
    tryCenter(8)
  }

  // 确保目标行所在区间已加载到 cache（锚点模式已让 visibleStart/End 覆盖目标行，
  // watch 会自动触发 ensureVisibleRangeLoaded，但首次跳转需主动加载目标行附近区间）
  const loadStart = Math.max(0, Math.floor((targetLineNo - FULL_LOG_OVERSCAN - Math.floor(FULL_LOG_BATCH / 3)) / FULL_LOG_LOAD_STEP) * FULL_LOG_LOAD_STEP)
  const loadCount = Math.min(FULL_LOG_BATCH, session.totalLines > 0 ? session.totalLines - loadStart : FULL_LOG_BATCH)

  if (session.cache.has(targetLineNo)) {
    // 已缓存：直接刷新渲染并居中
    refreshVisibleLines()
    doCenter()
  } else {
    // 未缓存：加载目标行所在区间后居中
    session.lastLoadRange = { start: loadStart, end: loadStart + loadCount }
    session.loadingPromise = (async () => {
      try {
        const result = await getTauriAPI().readLogFileLines(
          session.localPath,
          loadStart,
          loadCount,
          _fullLogCurrentKeyword || undefined
        )
        if (session.cache.size > FULL_LOG_CACHE_LIMIT) {
          const keys = Array.from(session.cache.keys()).sort((a, b) => a - b)
          for (let i = 0; i < 1000 && i < keys.length; i++) {
            session.cache.delete(keys[i])
          }
        }
        for (let i = 0; i < result.lines.length; i++) {
          session.cache.set(result.start + i, {
            lineNo: result.lines[i].lineNo,
            html: result.lines[i].html,
          })
        }
        if (result.totalLines !== session.totalLines) {
          session.totalLines = result.totalLines
          if (fullLogActiveSession.value === session) {
            fullLogTotalLines.value = result.totalLines
          }
        }
        if (fullLogActiveSession.value === session) {
          refreshVisibleLines()
        }
      } catch (e: any) {
        console.error('[LogAggregator] jump load failed:', e)
      } finally {
        session.loadingPromise = null
      }
    })()
    session.loadingPromise.then(() => doCenter()).catch(() => { _fullLogJumping = false })
  }
}

function fullLogNextMatch() {
  const session = fullLogActiveSession.value
  if (!session || session.matchLineNos.length === 0) {return}
  const nextIdx = (session.currentMatchIndex + 1) % session.matchLineNos.length
  fullLogJumpToMatch(nextIdx)
}

function fullLogPrevMatch() {
  const session = fullLogActiveSession.value
  if (!session || session.matchLineNos.length === 0) {return}
  const prevIdx = (session.currentMatchIndex - 1 + session.matchLineNos.length) % session.matchLineNos.length
  fullLogJumpToMatch(prevIdx)
}

function closeFullLogDialog() {
  // 使进行中的下载/激活回调失效，避免关闭后旧回调把 sessions 填回视图
  _fullLogGen++
  _fullLogFirstShown = false
  _fullLogActivating = false
  _fullLogJumping = false
  clearFullLogAnchor()
  if (fullLogDialog.value) {
    fullLogDialog.value.close()
  }
  // 清理状态，避免下次打开闪现旧内容
  _fullLogCurrentKeyword = ''
  fullLogSessions.value = []
  fullLogActiveIndex.value = 0
  fullLogVisibleLines.value = []
  fullLogTotalLines.value = 0
  fullLogSearchKeyword.value = ''
  fullLogMatchLineNos.value = []
  fullLogCurrentMatchIndex.value = -1
  fullLogCurrentMatchLineNo.value = -1
  fullLogError.value = ''
  if (_downloadProgressUnlisten) {
    try { _downloadProgressUnlisten() } catch {}
    _downloadProgressUnlisten = null
  }
  if (_fullLogResizeHandler) {
    window.removeEventListener('resize', _fullLogResizeHandler)
    _fullLogResizeHandler = null
  }
  if (_fullLogScrollHandler && fullLogContainer.value) {
    fullLogContainer.value.removeEventListener('scroll', _fullLogScrollHandler)
    _fullLogScrollHandler = null
  }
  if (_fullLogRafId) {
    cancelAnimationFrame(_fullLogRafId)
    _fullLogRafId = 0
  }
}

// 离线查看入口：弹窗让用户选择实时日志 或 历史日志
async function viewFullRemoteLog() {
  if (!selectedPreset.value) {
    toast.warning('请先选择预设')
    return
  }
  if (!selectedPreset.value.serverIds?.length) {
    toast.warning('预设未配置服务器')
    return
  }
  const paths = selectedPreset.value.logPath.split('\n').map((p: string) => p.trim()).filter(Boolean)
  if (paths.length === 0) {
    toast.warning('日志路径为空')
    return
  }
  if (fullLogEntryDialog.value && !fullLogEntryDialog.value.open) {
    fullLogEntryDialog.value.showModal()
  }
}

// 用户选择：实时日志
async function startRealtimeFullLog() {
  if (fullLogEntryDialog.value) {fullLogEntryDialog.value.close()}
  if (!selectedPreset.value) {return}
  const paths = selectedPreset.value.logPath.split('\n').map((p: string) => p.trim()).filter(Boolean)
  if (paths.length === 0) {
    toast.warning('日志路径为空')
    return
  }
  const logPath = paths[0].trim()
  const fileName = logPath.split('/').pop() || 'log.txt'
  await downloadAndShowLogs([{ path: logPath, name: fileName, isDir: false, size: 0, modifyTime: '0', isGz: false }], false)
}

// 用户选择：历史日志 -> 打开文件选择器
async function startHistoricalFullLog() {
  if (fullLogEntryDialog.value) {fullLogEntryDialog.value.close()}
  await openFilePickerForFirstServer()
}

// 取得预设 logPath 的父目录，用作文件选择器初始路径
function getParentDir(path: string): string {
  if (!path) {return '/'}
  const idx = path.lastIndexOf('/')
  if (idx <= 0) {return '/'}
  return path.substring(0, idx)
}

// 打开文件选择器，浏览预设中第一个服务器的目录
async function openFilePickerForFirstServer() {
  if (!selectedPreset.value?.serverIds?.length) {
    toast.warning('预设未配置服务器')
    return
  }
  const firstServerId = selectedPreset.value.serverIds[0]
  const paths = selectedPreset.value.logPath.split('\n').map((p: string) => p.trim()).filter(Boolean)
  const initialPath = paths.length > 0 ? getParentDir(paths[0]) : '/'

  filePickerServerId.value = firstServerId
  filePickerCurrentPath.value = initialPath
  filePickerPathInput.value = initialPath
  filePickerPathStack.value = []
  filePickerSelected.value = []
  filePickerError.value = ''
  filePickerFiles.value = []

  if (fullLogFilePickerDialog.value && !fullLogFilePickerDialog.value.open) {
    fullLogFilePickerDialog.value.showModal()
  }
  await filePickerLoadDir(initialPath)
}

// 加载目录列表
async function filePickerLoadDir(path: string) {
  if (!filePickerServerId.value) {return}
  filePickerLoading.value = true
  filePickerError.value = ''
  try {
    const result: any = await getTauriAPI().listSftpDir(filePickerServerId.value, path)
    if (result && result.success) {
      const raw: any[] = result.files || []
      const entries: RemoteFileEntry[] = raw
        .filter((f: any) => f.name !== '.' && f.name !== '..')
        .map((f: any) => ({
          name: f.name,
          path: path === '/' ? `/${f.name}` : `${path.replace(/\/$/, '')}/${f.name}`,
          isDir: f.type === 'directory',
          size: Number(f.size) || 0,
          modifyTime: f.modifyTime || '',
          isGz: f.name.toLowerCase().endsWith('.gz'),
        }))
        .sort((a: RemoteFileEntry, b: RemoteFileEntry) => {
          if (a.isDir && !b.isDir) {return -1}
          if (!a.isDir && b.isDir) {return 1}
          return a.name.localeCompare(b.name)
        })
      filePickerFiles.value = entries
      filePickerCurrentPath.value = path
      filePickerPathInput.value = path
    } else {
      filePickerError.value = (result && result.error) || '加载失败'
      filePickerFiles.value = []
    }
  } catch (e: any) {
    filePickerError.value = e.message || String(e)
    filePickerFiles.value = []
  } finally {
    filePickerLoading.value = false
  }
}

// 进入子目录
function filePickerEnterDir(entry: RemoteFileEntry) {
  if (!entry.isDir) {return}
  filePickerPathStack.value.push(filePickerCurrentPath.value)
  filePickerLoadDir(entry.path)
}

// 返回上一级
function filePickerGoUp() {
  if (filePickerPathStack.value.length === 0) {return}
  const prev = filePickerPathStack.value.pop()!
  filePickerLoadDir(prev)
}

// 按输入框路径加载
function filePickerLoadFromInput() {
  const p = filePickerPathInput.value.trim()
  if (!p || p === filePickerCurrentPath.value) {return}
  filePickerPathStack.value.push(filePickerCurrentPath.value)
  filePickerLoadDir(p)
}

// 切换文件选中状态
function filePickerToggleSelect(entry: RemoteFileEntry) {
  if (entry.isDir) {return}
  const idx = filePickerSelected.value.findIndex(f => f.path === entry.path)
  if (idx >= 0) {
    filePickerSelected.value.splice(idx, 1)
  } else {
    filePickerSelected.value.push(entry)
  }
}

function isFilePickerSelected(entry: RemoteFileEntry): boolean {
  return filePickerSelected.value.some(f => f.path === entry.path)
}

// 取消文件选择
function cancelFilePicker() {
  if (fullLogFilePickerDialog.value) {fullLogFilePickerDialog.value.close()}
  filePickerFiles.value = []
  filePickerSelected.value = []
  filePickerError.value = ''
}

// 确认选择 -> 下载并展示
async function confirmFilePickerSelection() {
  if (filePickerSelected.value.length === 0) {
    toast.warning('请至少选择一个文件')
    return
  }
  const selectedFiles = [...filePickerSelected.value]
  if (fullLogFilePickerDialog.value) {fullLogFilePickerDialog.value.close()}
  await downloadAndShowLogs(selectedFiles, true)
}

// 激活并展示指定 session：读取首段、绑定滚动/缩放监听、渲染可见行
async function activateFullLogSession(idx: number, gen: number) {
  const session = fullLogSessions.value[idx]
  if (!session) { return }
  fullLogActiveIndex.value = idx
  fullLogLoadingText.value = '正在读取日志...'
  const firstBatch = await getTauriAPI().readLogFileLines(session.localPath, 0, FULL_LOG_BATCH)
  // await 期间弹窗可能已被关闭/重新打开，旧调用不得污染新视图
  if (gen !== _fullLogGen) { return }
  session.totalLines = firstBatch.totalLines
  session.cache.clear()
  for (let i = 0; i < firstBatch.lines.length; i++) {
    session.cache.set(i, {
      lineNo: firstBatch.lines[i].lineNo,
      html: firstBatch.lines[i].html,
    })
  }
  fullLogTotalLines.value = session.totalLines
  fullLogLoading.value = false
  fullLogLoadingText.value = ''
  await nextTick()
  if (gen !== _fullLogGen) { return }
  if (fullLogContainer.value) {
    fullLogContainerHeight.value = fullLogContainer.value.clientHeight
    fullLogContainer.value.scrollTop = 0
    fullLogScrollTop.value = 0
    // 用局部引用绑定，避免交错激活时互相 remove 掉对方的 handler
    const scrollHandler = () => {
      if (_fullLogRafId) { return }
      _fullLogRafId = requestAnimationFrame(() => {
        _fullLogRafId = 0
        if (fullLogContainer.value) {
          const cur = fullLogActiveSession.value
          const st = fullLogContainer.value!.scrollTop
          // 程序化跳转期间（_fullLogJumping=true）不清除锚点，避免跳转后 scrollTop 估算
          // 行号又接管渲染导致目标行移出窗口。用户手动滚动时才清除锚点恢复正常虚拟滚动。
          if (!_fullLogJumping) {
            clearFullLogAnchor()
          }
          fullLogScrollTop.value = st
          if (cur) { cur.scrollTop = st }
        }
      })
    }
    if (_fullLogScrollHandler && fullLogContainer.value) {
      fullLogContainer.value.removeEventListener('scroll', _fullLogScrollHandler)
    }
    _fullLogScrollHandler = scrollHandler
    fullLogContainer.value.addEventListener('scroll', scrollHandler, { passive: true })
    const resizeHandler = () => {
      if (fullLogContainer.value) {
        const newH = fullLogContainer.value.clientHeight
        if (newH > 0 && newH !== fullLogContainerHeight.value) {
          fullLogContainerHeight.value = newH
        }
      }
    }
    if (_fullLogResizeHandler) {
      window.removeEventListener('resize', _fullLogResizeHandler)
    }
    _fullLogResizeHandler = resizeHandler
    window.addEventListener('resize', resizeHandler, { passive: true })
    refreshVisibleLines()
  }
}

// 后台预读 session 首段（切换 Tab 时立即可见）
async function preloadFullLogSession(s: FullLogSession) {
  if (!s.downloaded || s.totalLines > 0) { return }
  try {
    const batch = await getTauriAPI().readLogFileLines(s.localPath, 0, FULL_LOG_BATCH)
    s.totalLines = batch.totalLines
    for (let j = 0; j < batch.lines.length; j++) {
      s.cache.set(j, {
        lineNo: batch.lines[j].lineNo,
        html: batch.lines[j].html,
      })
    }
    // 若该 session 恰为当前激活 Tab，同步行数到视图，避免停留在"日志为空"
    if (fullLogActiveSession.value === s) {
      syncActiveSessionToView()
    }
  } catch (e: any) {
    s.loadError = `读取失败: ${e.message || String(e)}`
  }
}

// 通用：根据所选文件 + 所有服务器节点创建 session，并行下载并展示
// isHistorical=true 时按"每 (server, file) 一个 session"展开；isHistorical=false 时 fileName 来自预设单个 logPath
async function downloadAndShowLogs(files: RemoteFileEntry[], isHistorical: boolean) {
  if (!selectedPreset.value?.serverIds?.length) {return}
  const serverIds = selectedPreset.value.serverIds
  const downloadsDir = await getTauriAPI().getDownloadsDir()
  const timestamp = new Date().toISOString().slice(0, 19).replace(/:/g, '-')
  const sep = downloadsDir.endsWith('/') || downloadsDir.endsWith('\\') ? '' : '/'

  // 创建 sessions：历史日志每个 (server, file) 一个 session；实时日志每个 server 一个 session（取 files[0]）
  const sessions: FullLogSession[] = []
  const downloadIdPrefix = `logview_${isHistorical ? 'hist' : 'rt'}_${Date.now()}`
  let idCounter = 0
  const expandFiles = isHistorical ? files : [files[0]]
  for (const file of expandFiles) {
    for (const serverId of serverIds) {
      const server = allServers.value.find(s => s.id === serverId)
      if (!server) {continue}
      const localPath = `${downloadsDir}${sep}${server.name}_${timestamp}_${file.name}`
      sessions.push({
        serverId,
        serverName: server.name,
        fileName: file.name,
        remotePath: file.path,
        localPath,
        totalLines: 0,
        cache: new Map<number, { lineNo: number; html: string }>(),
        rowHeights: [],
        scrollTop: 0,
        lastLoadRange: { start: -1, end: -1 },
        loadingPromise: null,
        matchLineNos: [],
        currentMatchIndex: -1,
        currentMatchLineNo: -1,
        loadError: '',
        downloaded: false,
        downloadId: `${downloadIdPrefix}_${idCounter++}`,
        downloadTotal: 0,
        downloadDownloaded: 0,
        downloadStatus: 'pending',
      })
    }
  }

  if (sessions.length === 0) {
    toast.warning('服务器不存在')
    return
  }

  // 重置视图状态；自增代数使旧调用的异步回调不再污染本次视图
  const gen = ++_fullLogGen
  _fullLogFirstShown = false
  _fullLogActivating = false
  fullLogError.value = ''
  _fullLogCurrentKeyword = ''
  fullLogSearchKeyword.value = ''
  fullLogSessions.value = sessions
  fullLogActiveIndex.value = 0
  fullLogVisibleLines.value = []
  fullLogTotalLines.value = 0
  fullLogLoading.value = true
  fullLogLoadingText.value = `正在并行下载 ${sessions.length} 个日志文件...`
  if (fullLogDialog.value && !fullLogDialog.value.open) {
    fullLogDialog.value.showModal()
  }

  // 注册下载进度监听
  if (_downloadProgressUnlisten) {
    try { _downloadProgressUnlisten() } catch {}
    _downloadProgressUnlisten = null
  }
  try {
    _downloadProgressUnlisten = await getTauriAPI().onSftpDownloadProgress((payload) => {
      if (gen !== _fullLogGen) {return}
      const s = sessions.find(s => s.downloadId === payload.downloadId)
      if (!s) {return}
      s.downloadDownloaded = payload.downloaded
      s.downloadTotal = payload.total
      if (s.downloadStatus === 'pending') {s.downloadStatus = 'downloading'}
      fullLogSessions.value = [...sessions]
    })
  } catch (e) {
    console.warn('[LogAggregator] 注册下载进度监听失败:', e)
  }

  // 并行下载：每个节点下载完成后立即尝试展示（首个成功节点即出），其余后台预读
  const downloadPromises = sessions.map(async (s, idx) => {
    s.downloadStatus = 'downloading'
    if (gen === _fullLogGen) { fullLogSessions.value = [...sessions] }
    try {
      await getTauriAPI().downloadFileWithProgress(
        s.downloadId,
        s.serverId,
        s.serverName,
        s.remotePath,
        s.localPath,
        s.fileName
      )
      s.downloaded = true
      s.downloadStatus = 'done'
      // 历史日志：.gz 自动解压，localPath 切换为解压后路径
      if (isHistorical && s.fileName.toLowerCase().endsWith('.gz')) {
        try {
          const decompressed = await getTauriAPI().gunzipLocalFile(s.localPath)
          s.localPath = decompressed.decompressedPath
        } catch (e: any) {
          s.loadError = `解压失败: ${e.message || String(e)}`
          s.downloaded = false
          s.downloadStatus = 'failed'
        }
      }
    } catch (e: any) {
      s.loadError = `下载失败: ${e.message || String(e)}`
      s.downloaded = false
      s.downloadStatus = 'failed'
    }
    // 本次调用已被更新的调用取代，丢弃旧回调结果
    if (gen !== _fullLogGen) { return }
    fullLogSessions.value = [...sessions]
    // 首个成功节点立即激活展示，不必等所有节点下载完；
    // _fullLogActivating 防止多个节点同时完成时并发重复激活
    if (!_fullLogFirstShown && !_fullLogActivating && s.downloaded && !s.loadError) {
      _fullLogActivating = true
      try {
        await activateFullLogSession(idx, gen)
        if (gen !== _fullLogGen) { return }
        _fullLogFirstShown = true
      } catch (e: any) {
        // gen 可能已在读取期间变化（关闭后重开），旧 catch 不得覆盖新视图
        if (gen !== _fullLogGen) { return }
        s.loadError = `读取失败: ${e.message || String(e)}`
        fullLogSessions.value = [...sessions]
      } finally {
        // 仅当仍是本代调用时才复位标志，避免误清新调用的激活中状态
        if (gen === _fullLogGen) { _fullLogActivating = false }
      }
    } else if (s.downloaded && !s.loadError) {
      // 其余成功节点后台预读首段，切换 Tab 时立即可见
      preloadFullLogSession(s)
    }
  })
  await Promise.all(downloadPromises)
  if (gen !== _fullLogGen) { return }

  const okCount = sessions.filter(s => s.downloaded).length
  const failCount = sessions.length - okCount
  if (okCount === 0) {
    fullLogLoading.value = false
    fullLogLoadingText.value = ''
    fullLogError.value = '所有节点下载失败'
    return
  }
  // 兜底：节点下载成功但激活读取全部失败时，不能一直转圈
  if (fullLogLoading.value) {
    fullLogLoading.value = false
    fullLogLoadingText.value = ''
    fullLogError.value = '节点已下载但读取失败，请查看 Tab 上的错误标记'
  }
  const shown = fullLogSessions.value[fullLogActiveIndex.value]
  let msg = `已加载 ${okCount} 个文件`
  if (shown && shown.totalLines > 0) {msg += `，当前：${shown.serverName} · ${shown.fileName}（${shown.totalLines} 行）`}
  if (failCount > 0) {msg += `，${failCount} 个文件下载失败`}
  toast.success(msg)
}

// 滚动状态
const showScrollBottom = ref(false)
// 虚拟滚动：只渲染可视区内的行
const VIRTUAL_LINE_HEIGHT = 24    // px，匹配 contain-intrinsic-size: 1.5rem
const OVERSCAN = 10                // 视口上下额外渲染的行数（5-15 足够流畅，原值 100 浪费 DOM）
// 搜索模式全量渲染的行数上限：超出截断并提示，避免超大结果集卡死渲染
const SEARCH_MAX_LINES = 20000
const scrollTop = ref(0)
const containerHeight = ref(0)

// 流式日志保留上限：优先用预设的 maxLines，默认 3000
const MAX_LINES = computed(() => {
  const v = selectedPreset.value?.maxLines
  return (typeof v === 'number' && v >= 500 && v <= 50000) ? v : 3000
})

// ── 流式虚拟滚动「真实行高」支撑（消除固定行高导致的触底反弹） ──
// 日志长行 whitespace-pre-wrap 会换行，真实行高往往 > VIRTUAL_LINE_HEIGHT(24px)。
// 固定行高估算的 paddingTop/Bottom spacer 与真实 scrollHeight 不符，浏览器会 clamp
// scrollTop，导致"拉到底又反弹"。这里用每行实测高度做前缀和，spacer 逐步贴合真实高度
// （与 fullLog 离线查看的 fullLogHeightPrefix 同款方案）。
// 流式真实行高优化仅在「无过滤」时启用（此时 displayLines === logLines，下标一致）。
// 一旦启用关键字过滤或节点筛选，displayLines 是 logLines 子集，高度数组下标会错位，
// 此时回退固定行高估算（过滤场景是次要路径，准确性优先于流畅度）。
const streamUseRealHeight = computed(() =>
  queryMode.value === 'stream'
  && !(selectedPreset.value?.keywords?.length)
  && !selectedServerFilter.value
)

const streamLineHeights = ref<number[]>([])
const streamHeightPrefix = computed(() => {
  const total = totalItems.value
  const heights = streamLineHeights.value
  const prefix: number[] = new Array(total + 1)
  prefix[0] = 0
  for (let i = 0; i < total; i++) {
    prefix[i + 1] = prefix[i] + (heights[i] ?? VIRTUAL_LINE_HEIGHT)
  }
  return prefix
})
// 滚到顶部的行号（二分前缀和）
function streamRowAtScrollTop(): number {
  const prefix = streamHeightPrefix.value
  const st = scrollTop.value
  if (prefix.length === 0) {return 0}
  if (st <= 0) {return 0}
  let lo = 0, hi = prefix.length - 1
  while (lo < hi) {
    const mid = (lo + hi) >> 1
    if (prefix[mid] <= st) { lo = mid + 1 } else { hi = mid }
  }
  if (prefix[lo] <= st) {return Math.max(0, prefix.length - 2)}
  return Math.max(0, lo - 1)
}
function streamPrefixAt(row: number): number {
  const prefix = streamHeightPrefix.value
  if (row <= 0) {return 0}
  return prefix[Math.min(row, prefix.length - 1)] ?? 0
}

// 显示的行（过滤）
// 流式模式：使用 flush 时预计算的 matched 标记，避免每次重扫
// 优化：当无关键字时直接返回原数组（零拷贝），避免 computed 无谓重建
// 必须在下方所有引用它的 computed/watch（totalItems/visibleLines/renderedLines 及 watch(renderedLines)）之前定义，
// 否则 Vue 的 watch 在创建时会立即求值 source，触发 "Cannot access 'displayLines' before initialization"（TDZ）。
const displayLines = computed(() => {
  let lines: typeof logLines.value
  if (queryMode.value === 'search') {
    lines = logLines.value
  } else if (!selectedPreset.value?.keywords?.length) {
    lines = logLines.value
  } else {
    lines = logLines.value.filter(line => line.matched !== false)
  }
  // 节点筛选：选中某节点时只显示该节点的日志行
  if (selectedServerFilter.value) {
    lines = lines.filter(line => line.serverId === selectedServerFilter.value)
  }
  return lines
})

const totalItems = computed(() => displayLines.value.length)

const visibleStart = computed(() => {
  // 用真实行高前缀和反推行号（替代固定行高估算），长行换行时高度也能准确定位，
  // 避免 slice 空窗白屏；仍钳制到 [0, total-1] 保底。
  const total = totalItems.value
  const raw = (streamUseRealHeight.value ? streamRowAtScrollTop() : Math.floor(scrollTop.value / VIRTUAL_LINE_HEIGHT)) - OVERSCAN
  return Math.max(0, Math.min(raw, Math.max(0, total - 1)))
})

const visibleEnd = computed(() => {
  // 视口底边所在行（二分前缀和），再补 overscan
  const prefix = streamHeightPrefix.value
  const total = totalItems.value
  if (prefix.length === 0) {return 0}
  let endRow
  if (streamUseRealHeight.value) {
    const bottom = scrollTop.value + containerHeight.value
    let lo = 0, hi = prefix.length - 1
    while (lo < hi) {
      const mid = (lo + hi) >> 1
      if (prefix[mid] <= bottom) { lo = mid + 1 } else { hi = mid }
    }
    endRow = prefix[lo] <= bottom ? prefix.length - 2 : lo - 1
  } else {
    endRow = Math.ceil((scrollTop.value + containerHeight.value) / VIRTUAL_LINE_HEIGHT)
  }
  return Math.min(total, endRow + OVERSCAN + 1)
})

// 模板只渲染这部分行，其余行用 paddingTop/paddingBottom 撑开滚动空间
const visibleLines = computed(() => {
  return displayLines.value.slice(visibleStart.value, visibleEnd.value)
})

// 渲染数据源：流式模式虚拟滚动（只渲染视口附近行）；搜索模式全量渲染
// （搜索结果行数有限，且固定行高虚拟滚动在长行换行时会滚动回弹，全量渲染交给浏览器原生滚动）
const renderedLines = computed(() => {
  return queryMode.value === 'stream' ? visibleLines.value : displayLines.value
})

// 流式可见行渲染完成后采样真实行高 → streamLineHeights，前缀和随之精确，
// spacer 逐步贴合真实内容高度（每行首次进入视口测量一次，滚动中渐进收敛，消除反弹）。
// 过滤激活时（streamUseRealHeight=false）跳过采样，避免对子集下标写错高度。
watch(renderedLines, () => {
  if (queryMode.value !== 'stream' || !streamUseRealHeight.value) {return}
  nextTick(() => {
    const container = logContainer.value
    if (!container) {return}
    const heights = streamLineHeights.value
    let changed = false
    const rows = container.querySelectorAll('[data-stream-idx]')
    for (const r of rows) {
      const el = r as HTMLElement
      const idx = Number(el.dataset.streamIdx)
      const h = el.offsetHeight
      if (Number.isFinite(idx) && h > 0 && heights[idx] !== h) {
        if (!heights[idx]) {
          // 首次测量直接填
          heights[idx] = h
          changed = true
        } else if (Math.abs(heights[idx] - h) > 1) {
          heights[idx] = h
          changed = true
        }
      }
    }
    if (changed) {
      streamLineHeights.value = [...heights]
    }
  })
})

// 预设分组折叠状态
const collapsedPresetGroups = ref(new Set<string>())

// 预设表单
const showPresetForm = ref(false)
const editingPreset = ref<string | null>(null)

// 确认删除
const deleteConfirmDialog = ref<HTMLDialogElement | null>(null)
const deleteConfirmMessage = ref('')
let pendingDeletePresetId: string | null = null

function deletePreset(id: string) {
  const preset = presets.value.find(p => p.id === id)
  if (!preset) {return}
  pendingDeletePresetId = id
  deleteConfirmMessage.value = `确定删除预设"${preset.name}"？`
  deleteConfirmDialog.value?.showModal()
}

function executeDeletePreset() {
  if (!pendingDeletePresetId) {return}
  const id = pendingDeletePresetId
  pendingDeletePresetId = null
  deleteConfirmDialog.value?.close()
  doDeletePreset(id)
}

function cancelDeletePreset() {
  pendingDeletePresetId = null
  deleteConfirmDialog.value?.close()
}

const presetForm = ref({
  name: '',
  presetGroup: '未分组',
  serverIds: [] as string[],
  logType: 'file' as 'file' | 'journalctl' | 'docker' | 'custom',
  logPath: '',
  keywordsInput: '',
  maxLines: 2000
})

// 颜色映射
const serverColors = new Map<string, string>()
const colorPalette = ['#4ade80', '#60a5fa', '#f472b6', '#fbbf24', '#a78bfa', '#34d399', '#f87171', '#38bdf8']
let colorIndex = 0

// 格式化字节数为人类可读字符串
function formatBytes(bytes: number): string {
  if (!bytes || bytes <= 0) {return '0 B'}
  if (bytes < 1024) {return `${bytes} B`}
  if (bytes < 1024 * 1024) {return `${(bytes / 1024).toFixed(1)} KB`}
  if (bytes < 1024 * 1024 * 1024) {return `${(bytes / (1024 * 1024)).toFixed(2)} MB`}
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`
}

function getServerColor(serverId: string): string {
  if (!serverColors.has(serverId)) {
    serverColors.set(serverId, colorPalette[colorIndex % colorPalette.length])
    colorIndex++
  }
  return serverColors.get(serverId)!
}

// 预设按分组
const groupedPresets = computed(() => {
  const groups = new Map<string, any[]>()
  for (const preset of presets.value) {
    const g = preset.presetGroup || '未分组'
    if (!groups.has(g)) {groups.set(g, [])}
    groups.get(g)!.push(preset)
  }
  // 排序：生产 → 测试 → 预发 → 开发 → 其他 → 未分组
  const groupOrder = ['生产', '测试', '预发', '开发']
  const sorted = [...groups.entries()].sort(([a], [b]) => {
    const ai = groupOrder.indexOf(a)
    const bi = groupOrder.indexOf(b)
    if (ai !== -1 && bi !== -1) {return ai - bi}
    if (ai !== -1) {return -1}
    if (bi !== -1) {return 1}
    if (a === '未分组') {return 1}
    if (b === '未分组') {return -1}
    return a.localeCompare(b, 'zh')
  })
  return sorted.map(([presetGroup, items]) => ({ presetGroup, presets: items }))
})

// 分组显眼样式：按环境语义给色标（显式完整类名，避免 Tailwind 动态拼接不生成）
const GROUP_STYLES: Record<string, { bg: string; dot: string }> = {
  '生产': { bg: 'bg-error/10 hover:bg-error/15', dot: 'bg-error' },
  '测试': { bg: 'bg-success/10 hover:bg-success/15', dot: 'bg-success' },
  '预发': { bg: 'bg-warning/10 hover:bg-warning/15', dot: 'bg-warning' },
  '开发': { bg: 'bg-info/10 hover:bg-info/15', dot: 'bg-info' },
}
const DEFAULT_GROUP_STYLE = { bg: 'bg-base-200/70 hover:bg-base-200', dot: 'bg-base-content/30' }
function groupStyle(g: string): { bg: string; dot: string } {
  const key = ['生产', '测试', '预发', '开发'].find(k => g.includes(k))
  return key ? (GROUP_STYLES[key] || DEFAULT_GROUP_STYLE) : DEFAULT_GROUP_STYLE
}

// 构建命令
function buildCommand(preset: any): string {
  const paths = preset.logPath.split('\n').map((p: string) => p.trim()).filter((p: string) => p)
  const quotePath = (p: string) => {
    if (p.startsWith('~')) {
      // ~ 展开必须在引号外：$HOME'/rest'（bash 中 $HOME 展开 + 单引号保护其余部分）
      const rest = p.slice(1).replace(/'/g, "'\\''")
      return rest ? `$HOME'${rest}'` : '$HOME'
    }
    return `'${p.replace(/'/g, "'\\''")}'`
  }

  switch (preset.logType) {
    case 'file':
      // tail supports multiple files natively
      return `tail -n ${preset.maxLines} -f ${paths.map(quotePath).join(' ')}`
    case 'journalctl':
      // journalctl supports -u multiple times
      return `journalctl ${paths.map((u: string) => `-u ${quotePath(u)}`).join(' ')} -n ${preset.maxLines} -f --no-pager`
    case 'docker':
      // docker logs doesn't support multiple containers, chain them
      return paths.map((c: string) => `(echo "=== ${quotePath(c)} ===" && docker logs --tail ${preset.maxLines} -f ${quotePath(c)} 2>&1)`).join(' & ')
    case 'custom':
      return preset.logPath
    default:
      return `tail -n ${preset.maxLines} -f ${paths.map(quotePath).join(' ')}`
  }
}

// 检测日志级别（用单词边界避免 user_error_count 之类的误判）
const _LEVEL_PATTERNS: Array<{ level: string; re: RegExp }> = [
  { level: 'error', re: /\b(ERROR|FATAL|CRITICAL|EXCEPTION)\b/ },
  { level: 'warn',  re: /\b(WARN|WARNING)\b/ },
  { level: 'debug', re: /\bDEBUG\b/ },
]
function detectLevel(content: string): string {
  if (!content) {return 'info'}
  // 仅扫前 200 字符够用，避免长行全文扫描
  const head = content.length > 200 ? content.slice(0, 200).toUpperCase() : content.toUpperCase()
  for (const p of _LEVEL_PATTERNS) {
    if (p.re.test(head)) {return p.level}
  }
  return 'info'
}

// 从日志行中解析时间戳，返回毫秒时间戳或 null
// 支持常见格式：2026-05-29 10:30:15, 2026-05-29T10:30:15, May 29 10:30:15 等
// 性能：流式高吞吐热点，使用预编译正则 + 一次匹配（替代多次 head.match 调用）
const _TS_REGEX = /^(?:(\d{4})[-/](\d{2})[-/](\d{2})[T ](\d{2}):(\d{2}):(\d{2})(?:\.(\d{1,3}))?|([A-Z][a-z]{2})\s+(\d{1,2})\s+(\d{2}):(\d{2}):(\d{2})|(\d{2}):(\d{2}):(\d{2})(?:\.(\d{1,3}))?)/
function parseLogTimestamp(content: string): number | null {
  // 只检查行首 40 个字符（时间戳通常在行首）
  const m = _TS_REGEX.exec(content)
  if (!m) {return null}

  // 分支1: 完整日期时间
  if (m[1]) {
    const d = new Date(`${m[1]}-${m[2]}-${m[3]}T${m[4]}:${m[5]}:${m[6]}${m[7] ? '.' + m[7].padEnd(3, '0') : ''}`)
    return isNaN(d.getTime()) ? null : d.getTime()
  }
  // 分支2: syslog 风格
  if (m[8]) {
    const d = new Date(`${m[8]} ${m[9]} ${new Date().getFullYear()} ${m[10]}:${m[11]}:${m[12]}`)
    return isNaN(d.getTime()) ? null : d.getTime()
  }
  // 分支3: 仅时间
  const now = new Date()
  const d = new Date(now.getFullYear(), now.getMonth(), now.getDate(),
    parseInt(m[13]), parseInt(m[14]), parseInt(m[15]),
    m[16] ? parseInt(m[16].padEnd(3, '0')) : 0)
  return isNaN(d.getTime()) ? null : d.getTime()
}

// 获取预设关键字
function getKeywordsFromPreset(): string {
  if (selectedPreset.value?.keywords?.length) {
    return selectedPreset.value.keywords.join(', ')
  }
  return ''
}

// 搜索结果高亮：缓存正则，仅在关键词变化时重建
const _highlightRegex = computed<RegExp | null>(() => {
  const kw = queryMode.value === 'search' ? searchKeyword.value : getKeywordsFromPreset()
  if (!kw?.trim()) {return null}
  const escapedKw = kw.trim().replace(/[.*+?^${}()|[\]\\]/g, String.fromCharCode(92) + '&')
  return new RegExp(`(${escapedKw})`, 'gi')
})

function highlightSearchResult(content: string): string {
  if (typeof content !== 'string') {return ''}
  let result = content.replace(/</g, '&lt;').replace(/>/g, '&gt;')
  const regex = _highlightRegex.value
  if (!regex) {return result}
  result = result.replace(regex, '<mark>$1</mark>')
  return result
}

// 搜索模式输入框占位提示
const searchPlaceholder = computed(() => {
  const preset = selectedPreset.value
  const kw = preset?.keywords?.length ? preset.keywords.join(', ') : ''
  if (kw) {return `搜索日志... 预设关键字：${kw}`}
  return '搜索关键字'
})

// 当前可选的节点列表（从预设配置 + 实际收到日志的节点合并）
const availableServers = computed(() => {
  const preset = selectedPreset.value
  if (!preset?.serverIds?.length) {return []}
  const presetServers = preset.serverIds.map((sid: string) => {
    const s = allServers.value.find(srv => srv.id === sid)
    return { id: sid, name: s?.name || sid, online: activeServers.value.has(sid) }
  })
  return presetServers
})

// 节点筛选切换时：重算搜索匹配索引（displayLines 变了）、重置导航、流式模式自动吸底
watch(selectedServerFilter, () => {
  // 搜索模式：displayLines 随节点筛选变化，匹配索引必须重算，否则 N/n 跳转指向失效行号
  if (queryMode.value === 'search' && hasSearched.value) {
    matchIndices.value = []
    currentMatchIndex.value = -1
    currentMatchId.value = null
    nextTick(() => updateMatchIndices())
  }
  // 流式模式：切换节点后内容量可能骤变，若在跟随模式则自动吸底，否则保持当前位置
  if (queryMode.value === 'stream' && followMode.value && logContainer.value) {
    nextTick(() => {
      if (followMode.value) {scrollToBottomSilent()}
    })
  }
})

// 预设分组折叠
function togglePresetGroup(group: string) {
  if (collapsedPresetGroups.value.has(group)) {
    collapsedPresetGroups.value.delete(group)
  } else {
    collapsedPresetGroups.value.add(group)
  }
}

// 预设管理
function openNewPresetForm() {
  editingPreset.value = null
  presetForm.value = { name: '', presetGroup: '未分组', serverIds: [], logType: 'file', logPath: '', keywordsInput: '', maxLines: 2000 }
  showPresetForm.value = true
}

function editPreset(preset: any) {
  editingPreset.value = preset.id
  presetForm.value = {
    name: preset.name,
    presetGroup: preset.presetGroup || '未分组',
    serverIds: [...preset.serverIds],
    logType: preset.logType,
    logPath: preset.logPath,
    keywordsInput: preset.keywords.join(', '),
    maxLines: preset.maxLines
  }
  showPresetForm.value = true
}

// 当预设切换时，重新计算存量行的 matched 标记
function recalculateMatched() {
  const keywords = queryMode.value === 'stream' && selectedPreset.value?.keywords?.length
    ? selectedPreset.value.keywords.map((k: string) => k.toLowerCase())
    : []
  for (const line of logLines.value) {
    line.matched = keywords.length === 0 || keywords.some((kw: string) => line.content.toLowerCase().includes(kw))
  }
  // 触发响应式更新：直接 mutate 对象属性不会让 ref 重新求值，
  // 必须替换数组引用才能让 displayLines computed 重算
  logLines.value = [...logLines.value]
}

// 选择预设并查询
async function selectAndQuery(preset: any) {
  // 搜索模式下只选中预设
  if (queryMode.value === 'search') {
    selectedPreset.value = preset
    selectedServerFilter.value = null
    return
  }

  // 如果已经是当前预设且正在流，不做任何事（只有停止按钮能中断）
  if (selectedPreset.value?.id === preset.id && isStreaming.value) {
    scrollToBottom()
    return
  }

  selectedPreset.value = preset
  selectedServerFilter.value = null  // 切换预设时重置节点筛选
  recalculateMatched()
  if (isStreaming.value) {
    await stopQuery()
    // 小幅延迟，确保后端旧流完全清理后再启动新流
    await new Promise(r => setTimeout(r, 100))
  }
  await startQueryFromPreset(preset)
}

// 停止查询
async function stopQuery() {
  // 清空缓冲区和定时器
  if (logFlushTimer) {
    clearTimeout(logFlushTimer)
    logFlushTimer = null
  }
  logBuffer.length = 0
  pendingScroll = false

  // 先保存当前 streamId，避免被后续调用覆盖
  const id = streamId.value
  streamId.value = ''
  isStreaming.value = false

  try {
    if (id) {await getTauriAPI().logsStopStream(id)}
  } catch (e) {
    console.error('Failed to stop stream:', e)
  }
  followMode.value = false
  userScrolledUp.value = false
}

// 从预设开始查询
async function startQueryFromPreset(preset: any) {
  // 清理旧缓冲区
  if (logFlushTimer) { clearTimeout(logFlushTimer); logFlushTimer = null }
  logBuffer.length = 0
  pendingScroll = false

  streamId.value = `stream_${Date.now()}`
  logLines.value = []
  streamLineHeights.value = []
  activeServers.value = new Set<string>()
  followMode.value = true
  userScrolledUp.value = false
  showScrollBottom.value = false

  const command = buildCommand(preset)

  try {
    const result = await getTauriAPI().logsStartStream({
      streamId: streamId.value,
      serverIds: JSON.parse(JSON.stringify(preset.serverIds)),
      command
    })
    if (result?.success) {
      isStreaming.value = true
    } else {
      streamId.value = ''
      toast.error(result?.error || '启动日志流失败')
    }
  } catch (e: any) {
    console.error('Failed to start log stream:', e)
    streamId.value = ''
    toast.error('启动日志流失败: ' + e.message)
  }
}

// 执行搜索
async function doSearch() {
  if (!selectedPreset.value) {
    toast.warning('请先选择一个预设')
    return
  }
  if (!searchKeyword.value.trim()) {
    toast.warning('请输入搜索关键字')
    return
  }

  isSearching.value = true
  hasSearched.value = true
  logLines.value = []
  streamLineHeights.value = []
  // 同步清空导航状态：搜索失败/重新搜索时避免残留旧匹配索引（跳转会指向失效行）
  matchIndices.value = []
  currentMatchIndex.value = -1
  currentMatchId.value = null
  toast.info('正在搜索...')

  try {
    const result = await getTauriAPI().logSearch({
      query: searchKeyword.value.trim(),
      presetId: selectedPreset.value.id,
      lines: searchContextLines.value
    })

    if (result?.matches) {
      // 高亮预计算：全量渲染下逐行 v-html 在每次跳转（currentMatchId 变化）时都会重跑正则替换，
      // 预先生成 html 字段后渲染与跳转都只读缓存，避免大结果集卡顿
      for (const match of (result.matches || [])) {
        for (const m of match.lines) {
          const parsedTime = parseLogTimestamp(m.content)
          logLines.value.push({
            id: `${match.serverId}-${m.lineNum}-${Date.now()}`,
            serverId: match.serverId,
            serverName: match.serverName,
            timestamp: parsedTime ?? Date.now(),
            content: m.content,
            html: highlightSearchResult(m.content),
            level: detectLevel(m.content),
            isMatch: m.isMatch,
            lineNum: String(m.lineNum),
            sortKey: parsedTime ?? Date.now()
          } as any)
        }
      }

      // 行数上限保护：后端 grep 输出无限制，超大结果集全量渲染会卡死，截断并提示
      let truncated = false
      if (logLines.value.length > SEARCH_MAX_LINES) {
        logLines.value.splice(SEARCH_MAX_LINES)
        truncated = true
      }

      const totalMatches = result.matches?.reduce((s: number, m: any) => s + (m.matchCount || 0), 0) || 0
      toast.success(`搜索完成：${totalMatches} 个匹配，${logLines.value.length} 行结果${truncated ? '（结果过多，已截断显示前 ' + SEARCH_MAX_LINES + ' 行）' : ''}`)
      // 搜索完成后更新匹配索引
      nextTick(() => updateMatchIndices())
    } else {
      toast.error(result?.error || '搜索失败')
    }
  } catch (e: any) {
    console.error('Search failed:', e)
    toast.error('搜索失败: ' + e.message)
  } finally {
    isSearching.value = false
  }
}

// 日志行缓冲 — 批量添加减少 Vue 重渲染
const logBuffer: Array<{ serverId: string; serverName: string; line: string }> = []
let logFlushTimer: ReturnType<typeof setTimeout> | null = null

function scheduleFlush() {
  if (logFlushTimer) {return}
  logFlushTimer = setTimeout(() => {
    logFlushTimer = null
    if (logBuffer.length === 0) {return}
    const batch = logBuffer.splice(0, logBuffer.length)
    const len = logLines.value.length
    // 预分配数组空间，避免频繁扩容（unicorn: 用 Array.from 替代 new Array(n)）
    const newLines: Array<{ id: string; serverId: string; serverName: string; timestamp: number; content: string; level: string; matched?: boolean; sortKey: number }> = Array.from({ length: batch.length })
    const now = Date.now()
    // 预计算当前预设关键字（流式模式下只需计算一次）
    const presetKeywords = queryMode.value === 'stream' && selectedPreset.value?.keywords?.length
      ? selectedPreset.value.keywords.map((k: string) => k.toLowerCase())
      : []
    // 收集本批新增的 serverId，循环外批量 add（避免每行都触发响应式更新）
    const seenServerIds = new Set<string>()
    let validCount = 0
    for (let i = 0; i < batch.length; i++) {
      const data = batch[i]
      if (!data?.line || typeof data.line !== 'string' || !data?.serverId) {continue}
      const content = data.line
      const parsedTime = parseLogTimestamp(content)
      newLines[validCount++] = {
        id: `${data.serverId}-${now}-${Math.random()}`,
        serverId: data.serverId,
        serverName: data.serverName,
        timestamp: now,
        content,
        level: detectLevel(content),
        matched: presetKeywords.length === 0 || presetKeywords.some((kw: string) => content.toLowerCase().includes(kw)),
        sortKey: parsedTime ?? (now + validCount * 0.001)
      }
      seenServerIds.add(data.serverId)
    }
	    newLines.length = validCount

	    // 批量追加（不排序，按服务器返回的原始顺序显示）
	    logLines.value.push(...newLines)
	    // 同步高度数组（新实时行未测量用默认高度兜底）
	    streamLineHeights.value.push(...new Array(newLines.length).fill(undefined))

    // 批量更新 activeServers，避免循环内多次响应式触发
    if (seenServerIds.size > 0) {
      const merged = new Set(activeServers.value)
      for (const id of seenServerIds) {merged.add(id)}
      activeServers.value = merged
    }

    // 智能裁剪：仅在超出上限时裁剪，避免每次 flush 都排序
    // 历史日志行（isHistory，用户主动加载并插入头部）不参与裁剪——
    // 否则 tail 持续追加触发裁剪时，splice(0, overflow) 会从头部把刚加载的历史日志
    // 当"最早的实时行"删掉（表现为加载几秒后消失）
    const maxLines = MAX_LINES.value
    if (logLines.value.length > maxLines) {
      let historyCount = 0
      while (historyCount < logLines.value.length && (logLines.value[historyCount] as any).isHistory) {
        historyCount++
      }
      const nonHistoryCount = logLines.value.length - historyCount
      if (nonHistoryCount > maxLines) {
        const overflow = nonHistoryCount - maxLines
        // 从历史行之后开始删（最早的实时行），保留历史日志与最新日志
        logLines.value.splice(historyCount, overflow)
        // 同步高度数组
        streamLineHeights.value.splice(historyCount, overflow)
        // 同步调整虚拟滚动偏移量，避免裁剪后 visibleStart 索引错位（paddingTop 跳跃）
        // 删除点位于历史区（cutPx）之后：仅当视口在删除点下方时才需要前移 scrollTop
        const cutPx = streamUseRealHeight.value ? streamPrefixAt(historyCount) : historyCount * VIRTUAL_LINE_HEIGHT
        if (!followMode.value && logContainer.value && scrollTop.value > cutPx) {
          // 用被删行的真实高度累加（未测量行按默认高度兜底），替代固定行高估算
          let removedPx = 0
          if (streamUseRealHeight.value) {
            const heights = streamLineHeights.value
            for (let i = historyCount; i < historyCount + overflow && i < heights.length; i++) {
              removedPx += heights[i] ?? VIRTUAL_LINE_HEIGHT
            }
          } else {
            removedPx = overflow * VIRTUAL_LINE_HEIGHT
          }
          const adjustedScroll = Math.max(cutPx, scrollTop.value - removedPx)
          // 标记程序化滚动，避免 onScroll 把 followMode 翻转为 false
          scrollingFromRAFCount++
          logContainer.value.scrollTop = adjustedScroll
          scrollTop.value = adjustedScroll
          // 用 rAF 重置，等浏览器派发完 scroll 事件后再放行 onScroll
          requestAnimationFrame(() => { requestAnimationFrame(() => {
            scrollingFromRAFCount--
            if (scrollingFromRAFCount < 0) { scrollingFromRAFCount = 0 }
          }) })
        }
      }
      // 历史行同样设上限：避免用户反复点击加载更多导致历史区无限增长撑爆内存
      const HISTORY_MAX_LINES = Math.max(maxLines, 5000)
      if (historyCount > HISTORY_MAX_LINES) {
        const overflow = historyCount - HISTORY_MAX_LINES
        logLines.value.splice(0, overflow)
        // 同步高度数组
        streamLineHeights.value.splice(0, overflow)
        if (!followMode.value && logContainer.value && scrollTop.value > 0) {
          // 用被删历史行的真实高度累加
          let removedPx = 0
          if (streamUseRealHeight.value) {
            const heights = streamLineHeights.value
            for (let i = 0; i < overflow && i < heights.length; i++) {
              removedPx += heights[i] ?? VIRTUAL_LINE_HEIGHT
            }
          } else {
            removedPx = overflow * VIRTUAL_LINE_HEIGHT
          }
          const adjustedScroll = Math.max(0, scrollTop.value - removedPx)
          scrollingFromRAFCount++
          logContainer.value.scrollTop = adjustedScroll
          scrollTop.value = adjustedScroll
          requestAnimationFrame(() => { requestAnimationFrame(() => {
            scrollingFromRAFCount--
            if (scrollingFromRAFCount < 0) { scrollingFromRAFCount = 0 }
          }) })
        }
      }
    }
    if (followMode.value) {
      nextTick(() => {
        // 二次校验 followMode：nextTick 跨越了一个事件循环周期，
        // 期间用户可能已经向上滚动，需在执行前再次确认
        if (followMode.value) {
          scrollToBottomSilent()
        }
      })
    }
  }, 30) // 30ms flush，约 33fps，更流畅
}

// 添加日志行 — 推入缓冲区，由批量 flush 处理
function addLogLine(data: { serverId: string; serverName: string; line: string }) {
  logBuffer.push(data)
  scheduleFlush()
}

// 静默滚动到底部（不触发 followMode 判断）
function scrollToBottomSilent() {
  if (!logContainer.value) {
    pendingScroll = false
    return
  }
  // 二次校验：调用方可能在 nextTick / setTimeout 上下文中延迟调用本函数，
  // 到此刻用户可能已经手动向上滚动，需立即放弃，避免把用户拉回底部
  if (!followMode.value) {
    pendingScroll = false
    return
  }
  scrollingFromRAFCount++
  logContainer.value.scrollTop = logContainer.value.scrollHeight
  // ⚡ 同步更新虚拟滚动 ref，与真实 DOM 位置保持一致
  scrollTop.value = logContainer.value.scrollTop
  // 用双层 rAF 重置标志：浏览器在当前帧派发 scroll 事件，
  // 第一帧 rAF 在事件派发后执行，第二帧 rAF 确保下一轮 scroll 也被吞掉，
  // 避免微任务过早重置导致 onScroll 误判 followMode
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      scrollingFromRAFCount--
      if (scrollingFromRAFCount < 0) {scrollingFromRAFCount = 0}
    })
  })
}

function scrollToBottom() {
  if (logContainer.value) {
    userScrolledUp.value = false
    followMode.value = true
    scrollToBottomSilent()
    showScrollBottom.value = false
  }
}

// ── 搜索导航 ──
function updateMatchIndices() {
  matchIndices.value = []
  for (let i = 0; i < displayLines.value.length; i++) {
    if (displayLines.value[i].isMatch) {
      matchIndices.value.push(i)
    }
  }
}

function scrollToLineIndex(idx: number) {
  if (!logContainer.value) {return}
  // 搜索模式全量渲染：用真实 DOM 位置精确居中目标行（行高真实可变，idx*估算行高会累积错位）
  if (queryMode.value === 'search') {
    const container = logContainer.value
    const el = container.querySelector(`[data-log-idx="${idx}"]`) as HTMLElement | null
    if (el) {
      const containerRect = container.getBoundingClientRect()
      const elRect = el.getBoundingClientRect()
      const elTopInContent = container.scrollTop + (elRect.top - containerRect.top)
      const newScrollTop = Math.max(0, elTopInContent - container.clientHeight / 2 + elRect.height / 2)
      scrollingFromRAFCount++
      container.scrollTop = newScrollTop
      // ⚡ 同步更新虚拟滚动 ref，与真实 DOM 位置保持一致
      scrollTop.value = container.scrollTop
      requestAnimationFrame(() => { requestAnimationFrame(() => {
        scrollingFromRAFCount--
        if (scrollingFromRAFCount < 0) {scrollingFromRAFCount = 0}
      }) })
    }
    return
  }
  // 流式模式：虚拟滚动，按真实行高前缀和滚动到目标行顶部
  const targetTop = streamUseRealHeight.value ? streamPrefixAt(idx) : idx * VIRTUAL_LINE_HEIGHT
  const halfVisible = containerHeight.value / 2
  if (logContainer.value) {
    scrollingFromRAFCount++
    logContainer.value.scrollTop = Math.max(0, targetTop - halfVisible)
    // ⚡ 同步更新虚拟滚动 ref
    scrollTop.value = logContainer.value.scrollTop
    // 用双层 rAF 等浏览器派发完 scroll 事件再放行 onScroll
    requestAnimationFrame(() => { requestAnimationFrame(() => {
      scrollingFromRAFCount--
      if (scrollingFromRAFCount < 0) {scrollingFromRAFCount = 0}
    }) })
  }
}

function nextMatch() {
  if (matchIndices.value.length === 0) {return}
  const next = (currentMatchIndex.value + 1) % matchIndices.value.length
  currentMatchIndex.value = next
  const idx = matchIndices.value[next]
  currentMatchId.value = displayLines.value[idx]?.id ?? null
  scrollToLineIndex(idx)
}

function prevMatch() {
  if (matchIndices.value.length === 0) {return}
  const prev = (currentMatchIndex.value - 1 + matchIndices.value.length) % matchIndices.value.length
  currentMatchIndex.value = prev
  const idx = matchIndices.value[prev]
  currentMatchId.value = displayLines.value[idx]?.id ?? null
  scrollToLineIndex(idx)
}

// ── 滚动到顶部自动加载更多历史日志 ──
// 冷却时间：加载完成后短期内不再触发，避免 scrollTop 仍 < 50 时连续触发导致行数暴增
let _loadMoreCooldownUntil = 0
// TODO: Requires backend logs_load_more Tauri command. For now, gracefully degrades.
async function loadMoreHistory() {
  if (queryMode.value !== 'stream' || !selectedPreset.value || !streamId.value || loadingMore.value) {return}
  if (Date.now() < _loadMoreCooldownUntil) {return}
  // 用户主动查看历史：退出自动吸底，避免后续实时追加把视图拉回底部
  followMode.value = false
  loadingMore.value = true
  try {
    const result = await getTauriAPI().logsLoadMore({
      streamId: streamId.value,
      presetId: selectedPreset.value.id,
      currentCount: logLines.value.length,
      batchSize: 500,
    })
    if (result?.results && result.results.length > 0) {
      // 预计算当前预设关键字，避免每行都重新计算
      const presetKeywords = selectedPreset.value?.keywords?.length
        ? selectedPreset.value.keywords.map((k: string) => k.toLowerCase())
        : []
      const kwMatch = (content: string) =>
        presetKeywords.length === 0 || presetKeywords.some((kw: string) => content.toLowerCase().includes(kw))

      // 历史日志按时间正序回填：服务器返回的是"更早的行"，需要插入到现有头部之前
      // 为保证 sortKey 单调，给每条历史行分配一个递减的小数偏移

      // 去重：后端用 current_count=tail 偏移估算历史位置，但 logLines.length 含
      // 已加载历史行 + stream 实时追加行，导致窗口偏移不准、返回重复历史日志。
      // 用 Set 过滤掉已存在的行，避免"越滚越多"。
      const existingKeys = new Set(
        logLines.value.map(l => `${l.serverId}|${l.content}`))

      const baseSortKey = (logLines.value[0]?.sortKey ?? Date.now())
      const newLines: any[] = []
      const now = Date.now()
      let addedCount = 0
      let dupCount = 0
      for (const serverResult of result.results) {
        if (!serverResult.lines || serverResult.lines.length === 0) {continue}
        // 后端 `tail -n N | head -n M` 返回正序（旧→新，更早的行在前），
        // 正序遍历保证 newLines 里最早的历史行在最前，splice 到头部后保持正序。
        for (let i = 0; i < serverResult.lines.length; i++) {
          const content = serverResult.lines[i]
          if (!content) {continue}
          const dedupKey = `${serverResult.serverId}|${content}`
          if (existingKeys.has(dedupKey)) { dupCount++; continue }
          existingKeys.add(dedupKey)
          const parsedTime = parseLogTimestamp(content)
          newLines.push({
            id: `${serverResult.serverId}-more-${now}-${Math.random()}-${i}`,
            serverId: serverResult.serverId,
            serverName: serverResult.serverName || '',
            timestamp: parsedTime ?? now,
            content,
            level: detectLevel(content),
            matched: kwMatch(content),
            sortKey: parsedTime ?? (baseSortKey - (addedCount + 1)),
            // 标记为历史行：流式智能裁剪时跳过，避免被当作"最早的实时行"从头删掉
            isHistory: true,
          })
          addedCount++
        }
      }
      if (addedCount > 0) {
        // 一次性插入，避免多次响应式触发
        logLines.value.splice(0, 0, ...newLines)
        // 同步高度数组：历史行插入到头部，原有行高度整体后移 addedCount 位，
        // 新历史行未测量用默认高度兜底（渲染后由采样 watch 补测）。
        streamLineHeights.value = [
          ...new Array(addedCount).fill(undefined),
          ...streamLineHeights.value,
        ]
        // 新插入的历史行高度：优先用已测量的真实行高，未测量按默认行高兜底
        // （避免固定 VIRTUAL_LINE_HEIGHT 估算与真实长行高度不符导致 scrollTop 补偿失准）
        let addedHeight = streamUseRealHeight.value ? 0 : addedCount * VIRTUAL_LINE_HEIGHT
        if (streamUseRealHeight.value) {
          for (const nl of newLines) {
            const idx = displayLines.value.indexOf(nl)
            addedHeight += idx >= 0 ? (streamLineHeights.value[idx] ?? VIRTUAL_LINE_HEIGHT) : VIRTUAL_LINE_HEIGHT
          }
        }
        if (!followMode.value && logContainer.value && scrollTop.value >= 0) {
          scrollingFromRAFCount++
          // 确保加载后 scrollTop 离开触发区（>100），避免 onScroll 立刻再次触发 loadMoreHistory
          const newScrollTop = Math.max(100, scrollTop.value + addedHeight)
          logContainer.value.scrollTop = newScrollTop
          // ⚡ 同步虚拟滚动 ref
          scrollTop.value = logContainer.value.scrollTop
          // 双层 rAF 重置标志，等浏览器派发完 scroll 事件
          requestAnimationFrame(() => { requestAnimationFrame(() => {
            scrollingFromRAFCount--
            if (scrollingFromRAFCount < 0) {scrollingFromRAFCount = 0}
          }) })
        }
        // 设置 2 秒冷却，避免连续触发
        _loadMoreCooldownUntil = Date.now() + 2000
        toast.info(`已加载 ${addedCount} 条历史日志`)
      } else if (dupCount > 0) {
        // 返回的全是重复行 → 已到历史尽头，设长冷却避免空转
        _loadMoreCooldownUntil = Date.now() + 30000
        toast.info('没有更多历史日志了')
      }
    }
  } catch (e) {
    console.warn('[LogAggregator] loadMoreHistory failed:', e)
  } finally {
    loadingMore.value = false
  }
}

// 滚动事件
function onScroll() {
  if (!logContainer.value) {return}
  // 程序化滚动期间，忽略 onScroll（计数器 > 0 表示有未完成的程序化滚动）
  if (scrollingFromRAFCount > 0) {return}

  const el = logContainer.value
  scrollTop.value = el.scrollTop
  // 仅在大小变化时更新，避免每帧赋值
  if (containerHeight.value !== el.clientHeight) {
    containerHeight.value = el.clientHeight
  }

  const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 50

  // 用户向上滚动 → 停止自动追踪
  userScrolledUp.value = !atBottom && isStreaming.value
  // 只有用户手动滚动时才改变 followMode（程序化滚动被 scrollingFromRAFCount 拦截）
  if (!atBottom && isStreaming.value) {
    followMode.value = false
  } else if (atBottom) {
    followMode.value = true
  }

  showScrollBottom.value = userScrolledUp.value && isStreaming.value
}
// 继续查询（终止后重新启动同一预设，不清除日志）
async function resumeQuery() {
  if (!selectedPreset.value) {return}
  // 清理缓冲但不清除已有日志
  if (logFlushTimer) { clearTimeout(logFlushTimer); logFlushTimer = null }
  logBuffer.length = 0
  pendingScroll = false

  streamId.value = `stream_${Date.now()}`
  activeServers.value = new Set<string>()
  followMode.value = true
  userScrolledUp.value = false
  showScrollBottom.value = false

  const command = buildCommand(selectedPreset.value)

  try {
    const result = await getTauriAPI().logsStartStream({
      streamId: streamId.value,
      serverIds: JSON.parse(JSON.stringify(selectedPreset.value.serverIds)),
      command
    })
    if (result?.success) {
      isStreaming.value = true
    } else {
      streamId.value = ''
      toast.error(result?.error || '启动日志流失败')
    }
  } catch (e: any) {
    console.error('Failed to resume log stream:', e)
    streamId.value = ''
    toast.error('启动日志流失败: ' + e.message)
  }
}

// 切换查询模式
async function switchQueryMode(mode: 'stream' | 'search') {
  queryMode.value = mode

  // 清理缓冲和定时器，避免旧流残留
  if (logFlushTimer) { clearTimeout(logFlushTimer); logFlushTimer = null }
  logBuffer.length = 0
  pendingScroll = false
  logLines.value = []
  streamLineHeights.value = []
  hasSearched.value = false
  // 切换模式时重置节点筛选（搜索/流式共用同一筛选器，切换时清空避免困惑）
  selectedServerFilter.value = null
  // 重置搜索导航状态，避免切回搜索模式时显示陈旧的匹配索引
  matchIndices.value = []
  currentMatchIndex.value = -1
  currentMatchId.value = null

  if (mode === 'stream') {
    // 切换到流式模式：先停止旧查询，如果已有选中的预设则自动启动流式查询
    if (isStreaming.value) {
      await stopQuery()
    }
    followMode.value = true
    recalculateMatched()
    if (selectedPreset.value) {
      await startQueryFromPreset(selectedPreset.value)
    }
  } else {
    // 切换到搜索模式：停止正在的流式查询
    if (isStreaming.value) {
      await stopQuery()
    }
    followMode.value = false
  }
  showScrollBottom.value = false
}

// 清除日志
function clearLogs() {
  logLines.value = []
  streamLineHeights.value = []
  hasSearched.value = false
  selectedServerFilter.value = null
}

// 导出日志与下载远程日志功能已移除（离线查看已覆盖该需求）

// 预设管理
async function loadPresets() {
  try {
    presets.value = await getTauriAPI().logPresetsGetAll()
  } catch (e) {
    console.error('Failed to load presets:', e)
  } finally {
    presetsLoading.value = false
  }
}

async function savePreset() {
  if (!presetForm.value.name.trim()) {
    toast.warning('请输入预设名称')
    return
  }
  try {
    const data = {
      name: presetForm.value.name,
      presetGroup: presetForm.value.presetGroup || '未分组',
      serverIds: JSON.parse(JSON.stringify(presetForm.value.serverIds)),
      logPath: presetForm.value.logPath,
      logType: presetForm.value.logType,
      keywords: presetForm.value.keywordsInput.split(',').map((k: string) => k.trim()).filter((k: string) => k),
      maxLines: presetForm.value.maxLines
    }

    if (editingPreset.value) {
      await getTauriAPI().logPresetsUpdate(editingPreset.value, data)
    } else {
      await getTauriAPI().logPresetsAdd(data)
    }

    showPresetForm.value = false
    editingPreset.value = null
    presetForm.value = { name: '', presetGroup: '未分组', serverIds: [], logType: 'file', logPath: '', keywordsInput: '', maxLines: 2000 }
    await loadPresets()
    toast.success('预设已保存')
  } catch (e: any) {
    console.error('Failed to save preset:', e)
    toast.error('保存预设失败: ' + (e.message || '未知错误'))
  }
}

async function doDeletePreset(id: string) {
  try {
    await getTauriAPI().logPresetsDelete(id)
    if (selectedPreset.value?.id === id) {
      if (isStreaming.value) {await stopQuery()}
      selectedPreset.value = null
    }
    await loadPresets()
    toast.success('预设已删除')
  } catch (e: any) {
    console.error('Failed to delete preset:', e)
    toast.error('删除预设失败: ' + (e.message || '未知错误'))
  }
}

async function goToServers() {
  const { useAppStore } = await import("../../stores/appStore");
  const appStore = useAppStore()
  appStore.setViewMode('servers')
}

async function loadServers() {
  try {
    allServers.value = (await getTauriAPI().getAllServers()) || []
    allGroups.value = (await getTauriAPI().getServerGroups?.()) || []
  } catch (e) {
    console.error('Failed to load servers:', e)
    allServers.value = []
    allGroups.value = []
  }
}

// 事件监听
const onLineHandler = (data: any) => { if (data?.streamId === streamId.value) {addLogLine(data)} }
const onEndHandler = (data: any) => {
  if (!data?.serverId) {return}
  activeServers.value.delete(data.serverId)
  if (activeServers.value.size === 0) {
    isStreaming.value = false
    streamId.value = ''
  }
}
const onErrorHandler = (data: any) => {
  console.error(`[Log Error] ${data?.serverId}:`, data?.error)
  // 流式查询中某节点失败（如无权限）必须有可见提示，否则表现为「节点在线但一直没日志」
  toast.error(`日志流错误: ${data?.error || '未知错误'}`)
}
const onStreamStoppedHandler = (data: any) => {
  if (data?.streamId === streamId.value) {
    streamId.value = ''
    isStreaming.value = false
  }
}

let cleanupLogsLine: (() => void) | null = null
let cleanupLogsServerEnd: (() => void) | null = null
let cleanupLogsError: (() => void) | null = null
let cleanupStreamStopped: (() => void) | null = null
let _cleanupDataChanged: (() => void) | undefined
let _cleanupKeyDown: (() => void) | undefined
let _resizeObserver: ResizeObserver | null = null

onMounted(async () => {
  await Promise.all([loadPresets(), loadServers()])

  // 初始化虚拟滚动容器高度 + 监听后续尺寸变化（窗口 resize 等）
  if (logContainer.value) {
    containerHeight.value = logContainer.value.clientHeight
    _resizeObserver = new ResizeObserver((entries) => {
      const h = entries[0]?.contentRect.height
      if (typeof h === 'number' && h > 0 && h !== containerHeight.value) {
        containerHeight.value = h
      }
    })
    _resizeObserver.observe(logContainer.value)
  }

  // Esc 退出全屏
  window.addEventListener('keydown', onKeydown)

  /* Event listeners for log streaming from Tauri backend */
  cleanupLogsLine = await getTauriAPI().onLogsLine(onLineHandler);
  cleanupLogsServerEnd = await getTauriAPI().onLogsServerEnd(onEndHandler);
  cleanupLogsError = await getTauriAPI().onLogsError(onErrorHandler);
  cleanupStreamStopped = await getTauriAPI().onLogsStreamStopped(onStreamStoppedHandler);

  _cleanupDataChanged = await getTauriAPI().onDataChanged?.(({ type }: { type: string }) => {
    if (type === 'servers') {loadServers()}
  })

  // N/N 快捷键搜索导航
  function onKeyDown(e: KeyboardEvent) {
    if (e.key === 'n' || e.key === 'N') {
      // 在输入框/文本域/contenteditable 中敲 n 是输入，不触发跳转
      const t = e.target as HTMLElement | null
      if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.isContentEditable)) {return}
      if (queryMode.value !== 'search' || matchIndices.value.length === 0) {return}
      e.preventDefault()
      if (e.shiftKey) {
        prevMatch()
      } else {
        nextMatch()
      }
    }
  }
  window.addEventListener('keydown', onKeyDown)
  _cleanupKeyDown = () => window.removeEventListener('keydown', onKeyDown)
})

onUnmounted(async () => {
  if (logFlushTimer) { clearTimeout(logFlushTimer); logFlushTimer = null }
  logBuffer.length = 0
  if (isStreaming.value && streamId.value) {
    try { await getTauriAPI().logsStopStream(streamId.value) } catch {}
  }
  cleanupLogsLine?.()
  cleanupLogsServerEnd?.()
  cleanupLogsError?.()
  cleanupStreamStopped?.()
  _cleanupDataChanged?.()
  _resizeObserver?.disconnect()
  _resizeObserver = null
  window.removeEventListener('keydown', onKeydown)
  serverColors.clear()
  colorIndex = 0
  // 清理下载进度监听
  if (_downloadProgressUnlisten) {
    try { _downloadProgressUnlisten() } catch {}
    _downloadProgressUnlisten = null
  }
  // 清理离线日志滚动/缩放监听，避免组件卸载后回调残留
  if (_fullLogScrollHandler && fullLogContainer.value) {
    fullLogContainer.value.removeEventListener('scroll', _fullLogScrollHandler)
    _fullLogScrollHandler = null
  }
  if (_fullLogResizeHandler) {
    window.removeEventListener('resize', _fullLogResizeHandler)
    _fullLogResizeHandler = null
  }
})

// 用于记录切出前是否有活跃查询，等用户切回后自动恢复
let wasQueryBeforeDeactivate = false

// 切到其他 tab 时暂停后台日志流，避免持续占用主线程（导致部署时页面卡死）
onDeactivated(() => {
  wasQueryBeforeDeactivate = isStreaming.value
  if (logFlushTimer) { clearTimeout(logFlushTimer); logFlushTimer = null }
  logBuffer.length = 0
  if (isStreaming.value && streamId.value) {
    getTauriAPI().logsStopStream(streamId.value).catch(() => {})
    isStreaming.value = false
    streamId.value = ''
  }
})

// 切回时恢复日志流
onActivated(async () => {
  if (selectedPreset.value && wasQueryBeforeDeactivate) {
    await resumeQuery()
  }
})

</script>

<!-- 用于 v-html 渲染的 <mark> 标签样式（必须全局/非 scoped） -->
<style>
.log-line-text mark {
  background: #fbbf24;
  color: #000;
  padding: 0 2px;
  border-radius: 2px;
}

/* 未开始查询时的空态淡入 */
.log-empty { animation: logEmptyIn .3s ease both; }
@keyframes logEmptyIn { from { opacity: 0; transform: translateY(6px); } to { opacity: 1; transform: none; } }
@media (prefers-reduced-motion: reduce) { .log-empty { animation: none; } }
</style>
