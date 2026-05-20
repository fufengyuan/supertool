<template>
  <div class="flex h-full">
    <!-- 左侧会话列表 -->
    <div class="w-64 border-r border-base-content/10 flex flex-col bg-base-100">
      <!-- 会话列表头部 -->
      <div class="flex items-center justify-between px-3 py-2 border-b border-base-content/10">
        <span class="text-sm font-semibold text-base-content">会话</span>
        <button class="btn btn-ghost btn-xs" @click="refreshSessions" :disabled="loadingSessions">
          <SvgIcon name="refresh" size="12" :class="{ 'animate-spin': loadingSessions }" />
        </button>
      </div>

      <!-- 新会话按钮 -->
      <div class="px-2 py-2">
        <button class="btn btn-primary btn-sm w-full gap-1.5" @click="startNewChat" title="快捷键: Cmd+K">
          <SvgIcon name="plus" size="14" />
          新对话
        </button>
      </div>

      <!-- 会话搜索框 -->
      <div class="px-2 py-1">
        <div class="relative">
          <input
            v-model="sessionSearchQuery"
            type="text"
            class="input input-sm input-bordered w-full pl-7 text-xs"
            placeholder="搜索会话..."
          />
          <SvgIcon name="search" size="12" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-base-content/40" />
          <button
            v-if="sessionSearchQuery"
            class="absolute right-2 top-1/2 -translate-y-1/2 text-base-content/40 hover:text-base-content"
            @click="sessionSearchQuery = ''"
          >
            <SvgIcon name="close" size="12" />
          </button>
        </div>
      </div>

      <!-- 会话列表/搜索结果 -->
      <div class="flex-1 overflow-y-auto">
        <!-- 搜索结果 -->
        <template v-if="isSearchMode">
          <div v-if="isSearching" class="flex items-center justify-center py-8">
            <SvgIcon name="refresh" size="16" class="animate-spin text-base-content/40" />
          </div>
          <div v-else-if="searchResults.length === 0" class="flex flex-col items-center justify-center py-8 text-center">
            <SvgIcon name="search" size="24" class="text-base-content/30" />
            <p class="mt-2 text-xs text-base-content/50">未找到匹配的内容</p>
          </div>
          <div v-else class="flex flex-col gap-1 px-2 py-1">
            <div
              v-for="result in searchResults"
              :key="result.messageId"
              class="group flex flex-col gap-1 px-2 py-1.5 rounded-lg cursor-pointer transition-colors hover:bg-base-200"
              @click="jumpToSearchResult(result)"
            >
              <div class="flex items-center gap-2">
                <SvgIcon :name="sourceIcon(result.source)" size="12" class="shrink-0 text-base-content/50" />
                <span class="text-xs text-base-content/60">{{ result.sessionTitle || '新会话' }}</span>
                <span class="text-xs text-base-content/40">•</span>
                <span class="text-xs text-base-content/50">{{ result.role }}</span>
              </div>
              <div class="text-xs text-base-content line-clamp-2" v-html="highlightSnippet(result.snippet, sessionSearchQuery)"></div>
            </div>
          </div>
        </template>
        
        <!-- 正常会话列表 -->
        <template v-else>
          <div v-if="loadingSessions" class="flex items-center justify-center py-8">
            <SvgIcon name="refresh" size="16" class="animate-spin text-base-content/40" />
          </div>
          <div v-else-if="sessions.length === 0" class="flex flex-col items-center justify-center py-8 text-center">
            <SvgIcon name="chat" size="24" class="text-base-content/30" />
            <p class="mt-2 text-xs text-base-content/50">暂无会话</p>
          </div>
          <div v-else class="flex flex-col gap-1 px-2 py-1">
            <div
              v-for="session in sessions"
              :key="session.id"
              class="group flex items-center gap-2 px-2 py-1.5 rounded-lg cursor-pointer transition-colors"
              :class="currentSessionId === session.id ? 'bg-primary/10 text-primary' : 'hover:bg-base-200'"
              @click="selectSession(session)"
            >
              <SvgIcon :name="sourceIcon(session.source)" size="14" class="shrink-0" />
              <div class="flex flex-col min-w-0 flex-1">
                <span class="truncate text-xs font-medium">{{ session.title || session.preview || '新会话' }}</span>
                <span class="truncate text-xs text-base-content/50">{{ formatTime(session.lastActive || session.startedAt) }}</span>
              </div>
              <span class="text-xs text-base-content/40 shrink-0">{{ session.messageCount }}</span>
              <!-- hover 显示删除按钮 -->
              <button 
                class="btn btn-ghost btn-xs btn-square opacity-0 group-hover:opacity-100 transition-opacity shrink-0"
                @click.stop="deleteSession(session.id)"
                title="删除会话"
              >
                <SvgIcon name="trash" size="10" class="text-error" />
              </button>
            </div>
          </div>
        </template>
      </div>
    </div>

    <!-- 右侧聊天区域 -->
    <div class="flex-1 flex flex-col">
      <!-- 聊天头部 -->
      <div class="flex items-center justify-between px-4 py-2 border-b border-base-content/10 bg-base-100">
        <div class="flex items-center gap-2 flex-1">
          <button class="btn btn-ghost btn-xs btn-circle" @click="router.back()" title="返回">
            <SvgIcon name="arrowLeft" size="14" />
          </button>
          <SvgIcon name="bot" size="16" class="text-primary" />
          <!-- 标题显示/编辑 -->
          <template v-if="isEditingTitle">
            <input
              ref="titleInputRef"
              v-model="editingTitle"
              class="input input-sm input-bordered w-48 text-sm"
              placeholder="输入标题..."
              @keydown.enter.exact="saveTitle"
              @keydown.escape="cancelEditTitle"
              @blur="saveTitle"
            />
          </template>
          <template v-else>
            <span class="text-sm font-semibold text-base-content cursor-pointer hover:opacity-80" @click="startEditTitle">
              {{ currentSession?.title || '新对话' }}
            </span>
            <button v-if="currentSession" class="btn btn-ghost btn-xs btn-square" @click="startEditTitle">
              <SvgIcon name="edit" size="12" />
            </button>
          </template>
          <span v-if="currentSession" class="badge badge-ghost badge-xs">
            {{ currentSession.model }}
          </span>
          <!-- 会话统计 -->
          <span v-if="messages.length > 0" class="text-xs text-base-content/40">
            {{ sessionStats.totalMessages }} 条消息 · {{ sessionStats.totalTokens > 0 ? `${sessionStats.totalTokens} tokens` : '' }}
          </span>
        </div>
        <div class="flex items-center gap-2">
          <!-- 停止按钮 - 流式输出时显示，更醒目 -->
          <button 
            v-if="isStreaming && !isAborting" 
            class="btn btn-error btn-xs gap-1 animate-pulse shadow-error/30 shadow-md" 
            @click="abortChat"
          >
            <SvgIcon name="stop" size="12" />
            停止
          </button>
          <!-- 停止中状态 -->
          <button 
            v-if="isAborting" 
            class="btn btn-xs gap-1 btn-disabled"
            disabled
          >
            <span class="loading loading-spinner loading-xs"></span>
            停止中...
          </button>
          <button v-if="currentSession && messages.length > 0" class="btn btn-ghost btn-xs" @click="exportSession" title="导出 (Cmd+S)">
            <SvgIcon name="download" size="12" />
          </button>
          <button v-if="messages.length > 0" class="btn btn-ghost btn-xs" @click="clearMessages" title="清空消息">
            <SvgIcon name="clear" size="12" />
          </button>
          <!-- 任务面板按钮 -->
          <button 
            v-if="currentTasks.length > 0" 
            class="btn btn-xs"
            :class="showTaskPanel ? 'btn-primary' : 'btn-ghost'"
            @click="showTaskPanel = !showTaskPanel"
            title="显示任务列表"
          >
            <SvgIcon name="checklist" size="12" />
            <span class="text-xs">{{ completedTasksCount }}/{{ currentTasks.length }}</span>
          </button>
          <button v-if="currentSession" class="btn btn-ghost btn-xs" @click="deleteCurrentSession" title="删除">
            <SvgIcon name="trash" size="12" />
          </button>
          <!-- 搜索按钮 -->
          <div v-if="messages.length > 0" class="relative">
            <input
              v-model="searchQuery"
              type="text"
              class="input input-xs input-bordered w-20 focus:w-40 transition-all"
              placeholder="搜索..."
            />
            <button
              v-if="searchQuery"
              class="btn btn-ghost btn-xs btn-square absolute right-0"
              @click="clearSearch"
            >
              <SvgIcon name="close" size="10" />
            </button>
          </div>
        </div>
      </div>

      <!-- 消息列表 -->
      <div ref="messagesContainer" class="flex-1 min-w-0 overflow-y-auto overflow-x-hidden px-4 py-2 space-y-1">
        <!-- 加载消息状态 - 骨架屏 -->
        <div v-if="loadingMessages" class="space-y-1">
          <div class="flex gap-2">
            <div class="h-8 w-8 rounded-full bg-base-200 shrink-0 animate-pulse"></div>
            <div class="flex-1 space-y-1">
              <div class="h-4 bg-base-200 rounded w-3/4 animate-pulse"></div>
              <div class="h-4 bg-base-200 rounded w-1/2 animate-pulse"></div>
            </div>
          </div>
          <div class="flex gap-2">
            <div class="h-8 w-8 rounded-full bg-primary/20 shrink-0 animate-pulse"></div>
            <div class="flex-1 space-y-1">
              <div class="h-4 bg-primary/10 rounded w-full animate-pulse"></div>
              <div class="h-4 bg-primary/10 rounded w-2/3 animate-pulse"></div>
            </div>
          </div>
        </div>

        <!-- 消息列表 -->
        <template v-else-if="messages.length > 0">
          <div v-for="(msg, idx) in (searchQuery ? filteredMessages : displayMessages)" :key="idx" class="flex gap-2 w-full">
            <!-- 用户消息 -->
            <div v-if="msg.role === 'user'" class="flex gap-2 w-full group">
              <div class="flex h-8 w-8 items-center justify-center rounded-full bg-base-200 shrink-0">
                <SvgIcon name="user" size="14" class="text-base-content/60" />
              </div>
              <div class="max-w-[800px]">
                <div class="bg-primary/10 border border-primary/20 rounded-xl px-3 py-2">
                  <!-- 文件/文件夹路径徽章 -->
                  <div v-if="msg.filePaths && msg.filePaths.length > 0" class="flex flex-wrap gap-1.5 mb-1.5">
                    <div
                      v-for="(pathItem, pi) in msg.filePaths" :key="pi"
                      class="flex items-center gap-1 px-2 py-0.5 rounded-md text-xs border cursor-default"
                      :class="pathItem.type === 'folder'
                        ? 'bg-warning/10 border-warning/25 text-warning'
                        : 'bg-info/10 border-info/25 text-info/90'"
                      :title="pathItem.path"
                    >
                      <SvgIcon :name="pathItem.type === 'folder' ? 'folder' : 'file'" size="11" />
                      <span class="max-w-[200px] truncate">{{ pathItem.name }}</span>
                    </div>
                  </div>
                  <!-- 搜索时高亮显示 -->
                  <p v-if="searchQuery" class="text-sm text-base-content whitespace-pre-wrap" v-html="highlightText(userDisplayContent(msg), searchQuery)"></p>
                  <p v-else class="text-sm text-base-content whitespace-pre-wrap">{{ userDisplayContent(msg) }}</p>
                </div>
                <!-- 时间戳 -->
                <div v-if="msg.timestamp" class="mt-1 text-xs text-base-content/40">
                  {{ formatMessageTime(msg.timestamp) }}
                </div>
              </div>
            </div>

            <!-- Assistant 消息 -->
            <div v-else class="flex gap-2 w-full group">
              <div class="flex h-8 w-8 items-center justify-center rounded-full bg-primary/20 shrink-0">
                <SvgIcon name="bot" size="14" class="text-primary" />
              </div>
              <div class="max-w-[800px]">
                <!-- 思考过程（如果有）- 可折叠 -->
                <div 
                  v-if="msg.thinking" 
                  class="mb-2 bg-base-200/30 rounded-lg px-3 py-2 text-xs text-base-content/50 italic border border-base-content/10 cursor-pointer"
                  @click="toggleThinkingExpand(idx)"
                >
                  <div class="flex items-center justify-between">
                    <span>💭 思考过程</span>
                    <SvgIcon :name="isThinkingExpanded(idx) ? 'chevronDown' : 'chevronRight'" size="10" />
                  </div>
                  <div v-if="isThinkingExpanded(idx)" class="mt-2 whitespace-pre-wrap">
                    {{ msg.thinking }}
                  </div>
                </div>
                
                <div class="bg-base-100 border border-base-300 rounded-xl px-3 py-2">
                  <!-- 已停止徽章 -->
                  <div v-if="msg.isStopped" class="mb-2 flex items-center gap-1 text-xs text-warning">
                    <span class="inline-flex items-center gap-1 bg-warning/10 border border-warning/20 rounded px-1.5 py-0.5">
                      <SvgIcon name="stop" size="10" class="text-warning" />
                      已停止
                    </span>
                  </div>
                  <!-- Markdown 渲染的消息内容 - 主要样式 -->
                  <div v-if="msg.content" class="markdown-content text-sm text-base-content" v-html="renderMarkdown(msg.content)"></div>
                  
                  <!-- 工具调用卡片 - 次要样式 -->
                  <div v-if="msg.toolCalls && msg.toolCalls.length > 0" class="space-y-1.5">
                    <div v-for="(tool, tIdx) in msg.toolCalls" :key="`${tool.name}-${tIdx}`">
                      <!-- 子 Agent 卡片（次要样式） -->
                      <div v-if="tool.isSubAgent" class="bg-base-200/40 rounded-lg border border-base-content/10">
                        <!-- 外层：工具名 + 参数摘要 -->
                        <div 
                          class="px-2.5 py-1.5 cursor-pointer hover:bg-base-200/60 transition-colors"
                          @click="toggleToolCallExpand(`${idx}-${tIdx}`)"
                        >
                          <div class="flex items-center gap-2">
                            <SvgIcon name="bot" size="12" class="text-base-content/50" />
                            <span class="text-base-content/60 text-xs font-medium">子 Agent</span>
                            <span class="text-base-content/50 text-xs truncate flex-1">
                              {{ tool.args?.goal || tool.args?.task || tool.args?.prompt ? String(tool.args.goal || tool.args.task || tool.args.prompt).slice(0, 100) + '...' : '执行任务' }}
                            </span>
                            <span v-if="tool.status === 'completed'" class="text-base-content/40 text-xs">✓</span>
                            <span v-else-if="tool.status === 'running'" class="text-base-content/40 text-xs animate-pulse">○</span>
                            <span v-else-if="tool.status === 'error'" class="text-error/60 text-xs">✕</span>
                            <SvgIcon 
                              :name="isToolCallExpanded(`${idx}-${tIdx}`) ? 'chevronDown' : 'chevronRight'" 
                              size="10" 
                              class="text-base-content/30"
                            />
                          </div>
                        </div>
                        <!-- 折叠内容：详细结果 -->
                        <div v-if="isToolCallExpanded(`${idx}-${tIdx}`)" class="px-2.5 py-1.5 bg-base-200/20 text-xs">
                          <!-- 任务参数 -->
                          <div v-if="tool.args" class="mb-1">
                            <span class="text-base-content/40">参数：</span>
                            <pre class="bg-base-200/50 rounded p-1.5 mt-1 overflow-auto text-xs max-h-24">{{ JSON.stringify(tool.args, null, 2) }}</pre>
                          </div>
                          <!-- 执行结果 -->
                          <div v-if="tool.result" class="mt-1">
                            <span class="text-base-content/40">结果：</span>
                            <div class="bg-base-200/50 rounded p-1.5 mt-1 overflow-auto max-h-32 text-xs" v-html="formatToolResult(tool.name, tool.result)"></div>
                          </div>
                        </div>
                      </div>
                      
                      <!-- 普通工具卡片 - 次要样式 -->
                      <div v-else class="bg-base-200/40 rounded-lg border border-base-content/10">
                        <!-- 外层：工具名 + 参数摘要 -->
                        <div 
                          class="px-2.5 py-1.5 cursor-pointer hover:bg-base-200/60 transition-colors"
                          @click="toggleToolCallExpand(`${idx}-${tIdx}`)"
                        >
                          <div class="flex items-center gap-2">
                            <SvgIcon :name="getToolIcon(tool.name).icon" size="12" class="text-base-content/50" />
                            <span class="text-base-content/60 text-xs font-medium">{{ tool.name }}</span>
                            <!-- 参数摘要：显示关键参数的一行摘要 -->
                            <span v-if="tool.name === 'todo'" class="text-xs text-success truncate flex-1">
                              待办任务
                            </span>
                            <span v-else-if="tool.args && Object.keys(tool.args).length > 0" class="text-base-content/40 text-xs truncate flex-1">
                              {{ formatArgsSummary(tool.args) }}
                            </span>
                            <span v-if="tool.status === 'completed'" class="text-base-content/40 text-xs">✓</span>
                            <span v-else-if="tool.status === 'running'" class="text-base-content/40 text-xs animate-pulse">○</span>
                            <span v-else-if="tool.status === 'error'" class="text-error/60 text-xs">✕</span>
                            <SvgIcon 
                              :name="isToolCallExpanded(`${idx}-${tIdx}`) ? 'chevronDown' : 'chevronRight'" 
                              size="10" 
                              class="text-base-content/30"
                            />
                          </div>
                        </div>
                        <!-- todo 预览：折叠时展示格式化结果 -->
                        <div v-if="tool.name === 'todo' && tool.result && !isToolCallExpanded(`${idx}-${tIdx}`)" class="px-2.5 pb-1.5 text-xs" v-html="formatTodoResult(tool.result)"></div>
                        <!-- 折叠内容：详细结果 -->
                        <div v-if="isToolCallExpanded(`${idx}-${tIdx}`)" class="px-2.5 py-1.5 bg-base-200/20 text-xs">
                          <!-- 参数 -->
                          <div v-if="tool.args && Object.keys(tool.args).length > 0" class="mb-1">
                            <span class="text-base-content/40">参数：</span>
                            <pre class="bg-base-200/50 rounded p-1.5 mt-1 overflow-auto text-xs max-h-24">{{ JSON.stringify(tool.args, null, 2) }}</pre>
                          </div>
                          <!-- 结果 -->
                          <div v-if="tool.result" class="mt-1">
                            <span class="text-base-content/40">{{ tool.name === 'todo' ? '原始结果' : '结果' }}：</span>
                            <div v-if="tool.name === 'todo'" class="bg-base-200/50 rounded p-1.5 mt-1 overflow-auto max-h-32 text-xs whitespace-pre-wrap font-mono">{{ tool.result }}</div>
                            <div v-else class="bg-base-200/50 rounded p-1.5 mt-1 overflow-auto max-h-32 text-xs" v-html="formatToolResult(tool.name, tool.result)"></div>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
                <!-- 时间戳和重试按钮 -->
                <div class="mt-1 flex items-center justify-between">
                  <span v-if="msg.timestamp" class="text-xs text-base-content/40">
                    {{ formatMessageTime(msg.timestamp) }}
                  </span>
                  <!-- 错误消息重试按钮 -->
                  <button 
                    v-if="msg.isError && msg.retryContent"
                    class="btn btn-ghost btn-xs text-xs text-error hover:bg-error/10"
                    @click="retryMessage(msg.retryContent!)"
                  >
                    <SvgIcon name="refresh" size="10" />
                    重试
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- 流式响应状态（思考中/工具调用）+ 停止按钮 -->
          <div v-if="isStreaming" class="flex gap-2 w-full">
            <div class="flex h-8 w-8 items-center justify-center rounded-full bg-primary/20 shrink-0">
              <SvgIcon name="bot" size="14" class="text-primary animate-pulse" />
            </div>
            <div class="max-w-[1200px] bg-base-100 border border-base-300 rounded-xl px-3 py-2">
              <!-- 思考文本 -->
              <p v-if="thinkingText" class="text-sm text-base-content/60 animate-pulse">{{ thinkingText }}</p>
              <!-- 工具调用（只显示 running 状态的） -->
              <div v-else-if="currentStreamingMsg?.toolCalls && currentStreamingMsg.toolCalls.length > 0" class="mt-0 space-y-1">
                <div v-for="(tool, idx) in currentStreamingMsg.toolCalls.filter(t => t.status === 'running')" :key="idx" class="flex items-center gap-2 text-xs bg-base-200/50 rounded px-2 py-1">
                  <SvgIcon :name="getToolIcon(tool.name).icon" size="12" :class="getToolIcon(tool.name).color + ' animate-pulse'" />
                  <span :class="getToolIcon(tool.name).color" class="font-medium">{{ tool.name }}</span>
                  <span v-if="tool.args" class="text-base-content/70 truncate max-w-[600px]">{{ formatArgsSummary(tool.args) }}</span>
                  <span class="text-base-content/60 ml-auto animate-pulse">执行中...</span>
                </div>
              </div>
              <!-- 思考中（无工具调用时） -->
              <p v-else class="text-sm text-base-content/50 animate-pulse">思考中...</p>
            </div>
            <!-- 取消按钮 - 更醒目 -->
            <button 
              v-if="!isAborting"
              class="btn btn-error btn-sm btn-square self-center animate-pulse shadow-error/20 shadow-sm"
              @click="abortChat"
              title="取消处理 (Esc)"
            >
              <SvgIcon name="stop" size="16" />
            </button>
            <!-- 停止中状态 -->
            <button 
              v-if="isAborting"
              class="btn btn-sm btn-square self-center btn-disabled"
              disabled
              title="停止中..."
            >
              <span class="loading loading-spinner loading-sm"></span>
            </button>
          </div>
        </template>

        <!-- 空状态 -->
        <div v-else class="flex flex-col items-center justify-center py-16 text-center">
          <SvgIcon name="chat" size="32" class="text-base-content/30" />
          <p class="mt-2 text-sm text-base-content/50">开始对话</p>
          <p class="text-xs text-base-content/40">输入消息与 Hermes Agent 交流</p>
          <!-- 快捷建议 -->
          <div class="mt-4 flex flex-wrap gap-2 justify-center">
            <button 
              class="btn btn-ghost btn-xs text-xs"
              @click="inputText = '帮我分析一下当前项目的结构'"
            >
              分析项目
            </button>
            <button 
              class="btn btn-ghost btn-xs text-xs"
              @click="inputText = '帮我写一个测试用例'"
            >
              写测试
            </button>
            <button 
              class="btn btn-ghost btn-xs text-xs"
              @click="inputText = '帮我重构这段代码'"
            >
              重构代码
            </button>
          </div>
        </div>
        
        <!-- 回到底部按钮 - 用户向上滚动时显示 -->
        <button 
          v-if="showScrollToBottom && messages.length > 0"
          class="fixed bottom-[140px] right-[30px] btn btn-circle btn-sm btn-primary shadow-lg opacity-90 hover:opacity-100 transition-opacity z-10"
          @click="scrollToBottom"
          title="回到底部"
        >
          <SvgIcon name="chevronDown" size="16" />
        </button>
      </div>

      <!-- 输入区域 -->
      <div class="border-t border-base-content/10 px-4 py-3 bg-base-100">
        <!-- Hermes 未安装提示 -->
        <div v-if="!hermesAvailable" class="flex items-center justify-center gap-2 py-2">
          <SvgIcon name="warning" size="14" class="text-warning" />
          <span class="text-xs text-base-content/60">Hermes 未安装或不可用</span>
          <button class="btn btn-ghost btn-xs" @click="checkHermes">检测</button>
        </div>

        <!-- 正常输入 -->
        <div v-else class="space-y-2">
          <!-- 模型选择、工具集和引用消息显示 -->
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <!-- 附件按钮 -->
              <div class="relative">
                <button
                  class="btn btn-ghost btn-xs btn-square"
                  @click="showAttachMenu = !showAttachMenu"
                  title="添加文件/文件夹/Git仓库路径"
                  :disabled="isStreaming"
                >
                  <SvgIcon name="paperclip" size="14" />
                </button>
                <!-- 下拉菜单 -->
                <div 
                  v-if="showAttachMenu" 
                  class="absolute left-0 bottom-full mb-1 bg-base-100 border border-base-content/20 rounded-lg shadow-lg z-50 min-w-[200px]"
                >
                  <!-- 常用文件夹 -->
                  <div v-if="favoriteFolders.length > 0" class="border-b border-base-content/10">
                    <div class="px-3 py-1.5 text-xs text-base-content/50 font-medium flex items-center justify-between">
                      <span>常用文件夹</span>
                    </div>
                    <div 
                      v-for="folder in favoriteFolders" 
                      :key="folder"
                      class="flex items-center gap-1 px-2 py-1 text-xs hover:bg-base-200 group"
                    >
                      <span class="truncate flex-1 text-base-content/70" :title="folder">{{ folder.split('/').pop() || folder }}</span>
                      <button 
                        class="btn btn-ghost btn-xs btn-square opacity-0 group-hover:opacity-100" 
                        @click.stop="selectFromFavorite(folder, 'file')"
                        title="从此文件夹选择文件"
                      >
                        <SvgIcon name="file" size="10" class="text-base-content/60" />
                      </button>
                      <button 
                        class="btn btn-ghost btn-xs btn-square opacity-0 group-hover:opacity-100" 
                        @click.stop="selectFromFavorite(folder, 'folder')"
                        title="从此文件夹选择子文件夹"
                      >
                        <SvgIcon name="folder" size="10" class="text-base-content/60" />
                      </button>
                      <button 
                        class="btn btn-ghost btn-xs btn-square opacity-0 group-hover:opacity-100 hover:text-error" 
                        @click.stop="removeFavoriteFolder(folder)"
                        title="移除"
                      >
                        <SvgIcon name="close" size="10" />
                      </button>
                    </div>
                  </div>
                  <!-- 文件/文件夹选择 -->
                  <button class="flex items-center gap-2 w-full px-3 py-2 text-xs hover:bg-base-200" @click="selectFile()">
                    <SvgIcon name="file" size="14" class="text-base-content/60" />
                    <span>选择文件</span>
                  </button>
                  <button class="flex items-center gap-2 w-full px-3 py-2 text-xs hover:bg-base-200" @click="selectFolder()">
                    <SvgIcon name="folder" size="14" class="text-base-content/60" />
                    <span>选择文件夹</span>
                  </button>
                  <!-- Git 仓库列表 -->
                  <div v-if="gitRepos.length > 0" class="border-t border-base-content/10">
                    <div class="px-3 py-1.5 text-xs text-base-content/50 font-medium">Git 仓库</div>
                    <button 
                      v-for="repo in gitRepos" 
                      :key="repo.id" 
                      class="flex items-center gap-2 w-full px-3 py-1.5 text-xs hover:bg-base-200 rounded-b-lg"
                      @click="selectGitRepo(repo)"
                    >
                      <SvgIcon name="github" size="12" class="text-base-content/60" />
                      <span class="truncate">{{ repo.name }}</span>
                    </button>
                  </div>
                </div>
              </div>
              <!-- 模型选择 -->
              <select
                v-model="selectedModel"
                class="select select-bordered select-xs w-auto"
                :disabled="isStreaming"
              >
                <option value="">{{ defaultModel || '默认模型' }}</option>
                <option v-for="model in availableModels" :key="model" :value="model">{{ model }}</option>
              </select>
              <!-- 添加模型按钮 -->
              <button
                class="btn btn-ghost btn-xs btn-square"
                @click="showAddModelDialog = true"
                :disabled="isStreaming"
                title="添加模型"
              >
                <SvgIcon name="plus" size="14" />
              </button>
            </div>
          </div>
          <!-- 已选择路径徽章 -->
          <div v-if="attachedPaths.length > 0" class="flex flex-wrap gap-1.5 px-1">
            <div
              v-for="(item, idx) in attachedPaths"
              :key="idx"
              class="group flex items-center gap-1 px-2 py-1 rounded-md text-xs border cursor-default transition-all"
              :class="item.type === 'folder'
                ? 'bg-warning/5 border-warning/20 text-warning/80'
                : 'bg-info/5 border-info/20 text-info/80'"
              :title="item.path"
            >
              <img v-if="item.previewUrl" :src="item.previewUrl" class="w-6 h-6 rounded object-cover shrink-0" />
              <SvgIcon v-else :name="item.type === 'folder' ? 'folder' : 'file'" size="12" />
              <span class="max-w-[160px] truncate">{{ item.name }}</span>
              <button
                class="ml-0.5 opacity-40 group-hover:opacity-100 hover:!opacity-100 transition-opacity rounded-full hover:bg-base-content/10 p-0.5"
                :class="item.type === 'folder' ? 'hover:text-warning' : 'hover:text-info'"
                @click.stop="removeAttachedPath(idx)"
                title="移除"
              >
                <SvgIcon name="close" size="10" />
              </button>
            </div>
          </div>
          <!-- 输入框 -->
          <div class="flex gap-2">
            <textarea
              ref="inputRef"
              v-model="inputText"
              class="textarea w-full resize-none text-sm transition-colors"
              :class="isStreaming ? 'textarea-warning border-warning/30 bg-warning/5' : 'textarea-bordered'"
              style="min-height: 52px; max-height: 200px;"
              :placeholder="isStreaming ? '正在处理中，输入新消息将打断当前任务...' : '输入消息...'"
              @keydown.enter.exact.prevent="sendMessage"
              @paste="onPaste"
            ></textarea>
            <!-- 发送按钮 -->
            <button
              class="btn self-end transition-colors"
              :class="isStreaming ? 'btn-warning' : 'btn-primary'"
              :disabled="!inputText.trim() && !isStreaming"
              @click="sendMessage"
              :title="isStreaming ? '发送新消息将打断当前处理' : '发送'"
            >
              <SvgIcon v-if="isStreaming" name="send" size="14" class="animate-pulse" />
              <SvgIcon v-else name="send" size="14" />
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 右侧任务专栏 -->
    <div v-if="showTaskPanel && currentTasks.length > 0" class="w-72 border-l border-base-content/10 flex flex-col bg-base-100">
      <!-- 任务面板头部 -->
      <div class="flex items-center justify-between px-3 py-2 border-b border-base-content/10">
        <span class="text-sm font-semibold text-base-content">任务列表</span>
        <button class="btn btn-ghost btn-xs btn-square" @click="showTaskPanel = false" title="关闭">
          <SvgIcon name="close" size="12" />
        </button>
      </div>
      
      <!-- 任务列表 -->
      <div class="flex-1 overflow-y-auto px-3 py-2">
        <div v-for="task in currentTasks" :key="task.id" class="flex items-center gap-2 py-1.5 border-b border-base-content/5 last:border-b-0">
          <!-- 状态图标 -->
          <span :class="task.status === 'completed' ? 'text-success' : task.status === 'cancelled' ? 'text-base-content/30' : 'text-base-content/40'" class="text-xs">
            {{ task.status === 'completed' ? '✓' : task.status === 'cancelled' ? '✕' : '○' }}
          </span>
          <!-- 任务内容 -->
          <div class="flex-1 min-w-0">
            <span class="text-xs text-base-content truncate">{{ task.content }}</span>
          </div>
        </div>
      </div>
      
      <!-- 任务统计 -->
      <div class="px-3 py-2 border-t border-base-content/10 text-xs text-base-content/50">
        {{ completedTasksCount }}/{{ currentTasks.length }} 已完成
      </div>
    </div>
  </div>

  <!-- 添加模型对话框 -->
  <div v-if="showAddModelDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div class="bg-base-100 rounded-lg p-4 w-80 shadow-xl">
      <h3 class="text-sm font-medium mb-3">添加模型</h3>
      <input
        v-model="newModelName"
        type="text"
        class="input input-bordered input-sm w-full"
        placeholder="输入模型名称"
        @keyup.enter="addModel"
      />
      <div class="flex justify-end gap-2 mt-3">
        <button class="btn btn-ghost btn-sm" @click="showAddModelDialog = false; newModelName = ''">取消</button>
        <button class="btn btn-primary btn-sm" @click="addModel" :disabled="!newModelName.trim()">添加</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'HermesChat' })
import { ref, computed, onMounted, onUnmounted, nextTick, watch, type Ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import { marked } from 'marked';
import DOMPurify from 'dompurify';
import hljs from 'highlight.js/lib/core';
import { markedHighlight } from 'marked-highlight';
import javascript from 'highlight.js/lib/languages/javascript';
import { getTauriAPI } from '../../utils/tauri-api';
import type { GitRepo } from '../../types';
import python from 'highlight.js/lib/languages/python';
import json from 'highlight.js/lib/languages/json';
import sql from 'highlight.js/lib/languages/sql';
import typescript from 'highlight.js/lib/languages/typescript';
import bash from 'highlight.js/lib/languages/bash';
import xml from 'highlight.js/lib/languages/xml';
import yaml from 'highlight.js/lib/languages/yaml';
import SvgIcon from '@/components/ui/SvgIcon.vue';

hljs.registerLanguage('bash', bash);
hljs.registerLanguage('json', json);
hljs.registerLanguage('sql', sql);
hljs.registerLanguage('typescript', typescript);
hljs.registerLanguage('xml', xml);
hljs.registerLanguage('yaml', yaml);

// 配置 marked 使用 highlight.js
marked.use(markedHighlight({
  highlight(code: string, lang: string | undefined) {
    // 检测内容是否已经被 hljs 高亮过（包含 hljs-xxx 类名）
    if (code.includes('class="hljs-') || code.includes('class=\'hljs-')) {
      // 已经高亮过，直接返回，避免双重编码
      return code;
    }
    if (lang && hljs.getLanguage(lang)) {
      try {
        return hljs.highlight(code, { language: lang }).value;
      } catch {}
    }
    return hljs.highlightAuto(code).value;
  },
}));
marked.setOptions({
  breaks: true, // 支持 GFM 换行
  gfm: true, // GitHub Flavored Markdown
});

const route = useRoute();
const router = useRouter();

interface Session {
  id: string;
  title: string | null;
  model: string;
  source: string;
  startedAt?: number; // 可选，因为 Python bridge 可能不返回
  endedAt?: number | null; // 可选
  messageCount: number;
  preview: string;
  lastActive?: number; // 可选
}

// 搜索结果（来自 Hermes FTS5 搜索）
interface SearchResult {
  sessionId: string;
  sessionTitle: string | null;
  messageId: string;
  role: string;
  snippet: string;
  content: string | null;
  timestamp: number | null;
  source: string;
  model: string;
}

// 工具调用详情
interface ToolCall {
  id?: string; // 工具调用唯一 ID
  name: string;
  args?: Record<string, unknown>; // 工具参数
  result?: string; // 工具返回结果
  durationMs: number;
  isSubAgent?: boolean; // 是否是子 agent
  status?: 'running' | 'completed' | 'error'; // 状态
}

interface Message {
  role: string;
  content: string | null;
  timestamp: number | null;
  toolName: string | null;
  toolCalls?: ToolCall[];
  thinking?: string; // 思考过程
  isError?: boolean; // 是否是错误消息
  isStopped?: boolean; // 是否被用户停止
  retryContent?: string; // 用于重试的原始消息内容
  tokens?: { input: number; output: number }; // token 使用量
  filePaths?: PathItem[]; // 附带的文件/文件夹路径（仅用户消息）
}

// Raw message from backend (matches HermesMessage in Rust)
interface RawMessage {
  id: number;
  sessionId: string;
  role: string;
  content: string | null;
  timestamp: number;
  toolName: string | null;
  toolCallId: string | null;  // 工具调用 ID（tool 消息才有）
  toolCalls?: string;  // JSON string, parse in frontend
  finishReason: string | null;
  reasoning: string | null;  // 思考内容
  reasoningContent: string | null;
}

// Raw tool call from backend
interface RawToolCall {
  id: string;
  function: {
    name: string;
    arguments: string;
  };
}

// Task item from todo tool
interface TaskItem {
  id: string;
  content: string;
  status: 'pending' | 'in_progress' | 'completed' | 'cancelled';
}

// State
const sessions = ref<Session[]>([]);
const sessionSearchQuery = ref(''); // 会话搜索关键词
const searchResults = ref<SearchResult[]>([]); // 搜索结果
const isSearching = ref(false); // 搜索中状态
const currentSessionId = ref<string | null>(null);
const currentSession = ref<Session | null>(null);
const messages = ref<Message[]>([]);
const currentTasks = ref<TaskItem[]>([]); // 当前任务列表
const showTaskPanel = ref(true); // 是否显示任务面板
const inputText = ref('');
interface PathItem {
  path: string;
  type: 'file' | 'folder';
  name: string;
  previewUrl?: string; // 图片预览（object URL，需在移除时 revoke）
}
const attachedPaths = ref<PathItem[]>([]);
const gitRepos = ref<GitRepo[]>([]); // Git 仓库列表
const showAttachMenu = ref(false); // 显示附件菜单
const favoriteFolders = ref<string[]>([]); // 常用文件夹列表
const FAVORITE_KEY = 'hermes-favorite-folders'; // localStorage key
const loadingSessions = ref(false);
const loadingMessages = ref(false);
const isStreaming = ref(false);
const isAborting = ref(false); // 正在停止中
const thinkingText = ref(''); // 思考动画文本
const showScrollToBottom = ref(false); // 是否显示回到底部按钮
const hermesAvailable = ref(false);

// 当前流式响应的 assistant 消息（数组最后一个 assistant 消息）
const currentStreamingMsg = computed(() => {
  const lastMsg = messages.value[messages.value.length - 1];
  return lastMsg?.role === 'assistant' ? lastMsg : null;
});

// 用于渲染的消息列表（流式输出时跳过最后一个 assistant 消息，避免与实时气泡重复）
const displayMessages = computed(() => {
  // 直接返回 messages，流式内容实时显示在消息列表
  // 实时气泡仅用于显示工具调用状态和进度文本
  return messages.value;
});

// 模型选择
const selectedModel = ref('');
const availableModels = ref<string[]>([]); // 从 Hermes 配置读取
const defaultModel = ref<string>(''); // 默认模型

// 加载模型列表
const loadModels = async () => {
  try {
    const result = await invoke<{ customModels: string[]; defaultModel: string | null }>('agent_get_models');
    availableModels.value = result.customModels || [];
    defaultModel.value = result.defaultModel || '';
  } catch (e) {
    console.error('Failed to load models:', e);
    availableModels.value = [];
  }
};

// 添加模型
const showAddModelDialog = ref(false);
const newModelName = ref('');

const addModel = async () => {
  if (!newModelName.value.trim()) return;
  try {
    const result = await invoke<{ success: boolean; customModels: string[] }>('agent_add_model', {
      model: newModelName.value.trim(),
    });
    if (result.success) {
      availableModels.value = result.customModels;
      newModelName.value = '';
      showAddModelDialog.value = false;
    }
  } catch (e) {
    console.error('Failed to add model:', e);
  }
};

// 删除模型
const removeModel = async (model: string) => {
  try {
    const result = await invoke<{ success: boolean; customModels: string[] }>('agent_remove_model', {
      model,
    });
    if (result.success) {
      availableModels.value = result.customModels;
      // 如果删除的是当前选中的模型，重置选择
      if (selectedModel.value === model) {
        selectedModel.value = '';
      }
    }
  } catch (e) {
    console.error('Failed to remove model:', e);
  }
};

// 搜索状态
const searchQuery = ref('');
const filteredMessages = ref<Message[]>([]);

// 工具调用展开状态 (key: `${msgIdx}-${toolIdx}`)
const expandedToolCalls = ref<Set<string>>(new Set());

// 思考过程展开状态 (key: msgIdx)
const expandedThinking = ref<Set<number>>(new Set());

// 切换工具调用展开
const toggleToolCallExpand = (key: string) => {
  if (expandedToolCalls.value.has(key)) {
    expandedToolCalls.value.delete(key);
  } else {
    expandedToolCalls.value.add(key);
  }
};

// 检查是否展开
const isToolCallExpanded = (key: string): boolean => {
  return expandedToolCalls.value.has(key);
};

// 切换思考过程展开
const toggleThinkingExpand = (msgIdx: number) => {
  if (expandedThinking.value.has(msgIdx)) {
    expandedThinking.value.delete(msgIdx);
  } else {
    expandedThinking.value.add(msgIdx);
  }
};

// 检查思考过程是否展开
const isThinkingExpanded = (msgIdx: number): boolean => {
  return expandedThinking.value.has(msgIdx);
};

// Refs
const messagesContainer = ref<HTMLElement | null>(null);
const inputRef = ref<HTMLTextAreaElement | null>(null);
const titleInputRef = ref<HTMLInputElement | null>(null);

// 标题编辑状态
const isEditingTitle = ref(false);
const editingTitle = ref('');

// 复制代码功能（全局函数）
const copyCode = (codeId: string) => {
  const codeElement = document.getElementById(codeId);
  if (codeElement) {
    const text = codeElement.textContent || '';
    navigator.clipboard.writeText(text).then(() => {
      // 显示复制成功提示
      const btn = codeElement.closest('.code-block-wrapper')?.querySelector('.copy-btn');
      if (btn) {
        btn.classList.add('copied');
        setTimeout(() => btn.classList.remove('copied'), 2000);
      }
    });
  }
};
// 挂载到 window 以便 onclick 调用
if (typeof window !== 'undefined') {
  (window as any).copyCode = copyCode;
}

// Event listeners
let unlistenDelta: UnlistenFn | null = null;
let unlistenToolStart: UnlistenFn | null = null;
let unlistenToolComplete: UnlistenFn | null = null;
let unlistenError: UnlistenFn | null = null;
let unlistenDone: UnlistenFn | null = null;
// 标志：上一轮是否结束（收到 tool_complete 后等待新一轮）
let lastAssistantRoundEnded = false;

// 调试日志函数（写入日志文件）
const agentLog = async (message: string) => {
  // 直接写入 DEBUG 日志，不再调用 console.log（会被 main.ts 拦截写入 INFO，导致双重记录）
  try {
    const api = getTauriAPI();
    await api.writeSystemLog('debug', 'agent-chat', message);
  } catch (e) {
    // 忽略日志写入失败
  }
};

// 自动调整输入框高度
const adjustTextareaHeight = () => {
  if (inputRef.value) {
    inputRef.value.style.height = 'auto';
    // 限制最大高度为 200px（约 8 行）
    const maxHeight = 200;
    const newHeight = Math.min(inputRef.value.scrollHeight, maxHeight);
    inputRef.value.style.height = `${newHeight}px`;
  }
};

// 选择文件并追加路径到输入框
const selectFile = async (defaultPath?: string) => {
  try {
    const selected = await open({
      multiple: false,
      title: '选择文件',
      defaultPath: defaultPath || undefined,
    });
    if (selected) {
      const path = Array.isArray(selected) ? selected[0] : selected;
      const name = path.split('/').pop() || path;
      attachedPaths.value.push({ path, type: 'file', name });
      // 记住选择的目录作为常用文件夹
      const dir = path.split('/').slice(0, -1).join('/') || path;
      addFavoriteFolder(dir);
      nextTick(() => adjustTextareaHeight());
    }
  } catch (e) {
    console.error('选择文件失败:', e);
  }
  showAttachMenu.value = false;
};

// 选择文件夹并追加路径到输入框
const selectFolder = async (defaultPath?: string) => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择文件夹',
      defaultPath: defaultPath || undefined,
    });
    if (selected) {
      const path = Array.isArray(selected) ? selected[0] : selected;
      const name = path.split('/').pop() || path;
      attachedPaths.value.push({ path, type: 'folder', name });
      // 记住选择的目录作为常用文件夹
      addFavoriteFolder(path);
      nextTick(() => adjustTextareaHeight());
    }
  } catch (e) {
    console.error('选择文件夹失败:', e);
  }
  showAttachMenu.value = false;
};

// 选择 Git 仓库并追加路径到输入框
const selectGitRepo = (repo: GitRepo) => {
  const name = repo.path.split('/').pop() || repo.name || repo.path;
  attachedPaths.value.push({ path: repo.path, type: 'folder', name });
  showAttachMenu.value = false;
  nextTick(() => adjustTextareaHeight());
};

// 从常用文件夹打开文件选择
const selectFromFavorite = (folder: string, type: 'file' | 'folder') => {
  if (type === 'file') {
    selectFile(folder);
  } else {
    selectFolder(folder);
  }
};

// 移除已选择的路径（同时释放图片预览的 object URL）
const removeAttachedPath = (idx: number) => {
  const item = attachedPaths.value[idx];
  if (item?.previewUrl) URL.revokeObjectURL(item.previewUrl);
  attachedPaths.value.splice(idx, 1);
  nextTick(() => adjustTextareaHeight());
};

// 粘贴处理（支持粘贴图片/文件到输入框，保存为临时文件后追加路径到已选路径列表）
async function onPaste(e: ClipboardEvent) {
  const dt = e.clipboardData;
  if (!dt) return;

  const files = dt.files;
  const items = dt.items;
  const savedPaths: SavedFile[] = [];

  // === 先做同步检测，再决定是否阻止默认粘贴 ===
  // WKWebView 在 await 期间可能已经执行了默认行为，
  // 所以 preventDefault() 必须在任意 await 之前同步调用

  const hasFiles = files.length > 0;
  const hasImageItems = items && Array.from(items).some(item => item.type.startsWith('image/'));
  const uriText = dt.getData('text/uri-list')?.trim();
  const hasFileUrls = !!uriText && uriText.includes('file://');
  const rawText = dt.getData('text/plain')?.trim();
  const hasPathText = !!rawText && rawText.split('\n').every(l => {
    const t = l.trim();
    return t.length > 0 && t.startsWith('/') && t.lastIndexOf('/') > 0 && t.indexOf(' ') === -1;
  });

  // 有文件/图片/路径 → 同步阻止默认粘贴
  if (hasFiles || hasImageItems || hasFileUrls || hasPathText) {
    e.preventDefault();
  } else {
    return; // 纯文本，让浏览器默认行为处理
  }

  // 辅助：将 File 保存为临时文件，返回路径和图片预览
  type SavedFile = { path: string; previewUrl?: string };
  const saveFile = async (file: File, isImage: boolean): Promise<SavedFile | null> => {
    try {
      const arrayBuffer = await file.arrayBuffer();
      const blob = new Blob([arrayBuffer]);
      // 图片文件创建 object URL 用于预览（非图片不创建）
      let previewUrl: string | undefined;
      if (isImage) {
        previewUrl = URL.createObjectURL(blob);
      }
      const dataUrl = await new Promise<string>((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => resolve(reader.result as string);
        reader.onerror = () => reject(reader.error);
        reader.readAsDataURL(blob);
      });
      const base64 = dataUrl.split(',')[1];
      const fileName = file.name ? `pasted_${Date.now()}_${file.name}` : `pasted_${Date.now()}`;
      const result = await getTauriAPI().saveTempFile(base64, fileName);
      if (result) {
        return { path: result, previewUrl };
      }
      // 保存失败时释放 object URL
      if (previewUrl) URL.revokeObjectURL(previewUrl);
      return null;
    } catch (err) {
      console.error('[HermesChat] paste save error:', err);
      return null;
    }
  };

  // 1. 从 clipboardData.files 检测粘贴的文件/图片（Finder 粘贴文件、截图等）
  if (files.length > 0) {
    for (let i = 0; i < files.length; i++) {
      const file = files[i];
      if (!file) continue;
      const isImage = !!file.type?.startsWith('image/');
      const saved = await saveFile(file, isImage);
      if (saved) savedPaths.push(saved);
    }
  }

  // 2. 备用：从 clipboardData.items 检测图片（部分场景 files 为空但 items 有）
  if (savedPaths.length === 0 && items) {
    for (let i = 0; i < items.length; i++) {
      const item = items[i];
      if (item.type.startsWith('image/')) {
        const file = item.getAsFile();
        if (file) {
          const saved = await saveFile(file, true);
          if (saved) savedPaths.push(saved);
        }
        break;
      }
    }
  }

  // 3. 从 text/uri-list 检测文件路径（macOS Finder 粘贴）
  if (savedPaths.length === 0) {
    const uriText = dt.getData('text/uri-list')?.trim();
    if (uriText) {
      const urls = uriText.split('\n').map(l => l.trim()).filter(l => l.startsWith('file://'));
      if (urls.length > 0) {
        for (const url of urls) {
          const path = decodeURIComponent(url.replace(/^file:\/\//, ''));
          savedPaths.push({ path });
        }
      }
    }
  }

  // 3b. 备选：从 text/plain 检测绝对路径
  if (savedPaths.length === 0) {
    const rawText = dt.getData('text/plain')?.trim();
    if (rawText) {
      const lines = rawText.split('\n').map(l => l.trim()).filter(l => l.length > 0);
      const allArePaths = lines.length > 0 && lines.every(l =>
        l.startsWith('/') && l.lastIndexOf('/') > 0 && l.indexOf(' ') === -1
      );
      if (allArePaths) {
        for (const path of lines) {
          savedPaths.push({ path });
        }
      }
    }
  }

  // 4. 追加到已选路径徽章
  if (savedPaths.length > 0) {
    for (const { path, previewUrl } of savedPaths) {
      const name = path.split('/').pop() || path;
      const type: 'file' | 'folder' = (!name.includes('.') || path.endsWith('/')) ? 'folder' : 'file';
      attachedPaths.value.push({ path, type, name, previewUrl });
    }
    nextTick(() => adjustTextareaHeight());
  }
}
// 追加路径到输入框（保留用于兼容外部调用）
const appendPathToInput = (path: string) => {
  if (inputText.value.trim()) {
    inputText.value += '\n' + path;
  } else {
    inputText.value = path;
  }
  // 调整输入框高度
  nextTick(() => adjustTextareaHeight());
};

// 添加常用文件夹（最多保留3个，按最近使用排序）
const addFavoriteFolder = (folder: string) => {
  if (!folder) {
    return;
  }
  // 如果已存在，移到第一位（最近使用）
  if (favoriteFolders.value.includes(folder)) {
    favoriteFolders.value = [folder, ...favoriteFolders.value.filter(f => f !== folder)];
    saveFavoriteFolders(); // 持久化顺序更新
    return;
  }
  // 添加新文件夹到第一位，最多保留3个
  favoriteFolders.value = [folder, ...favoriteFolders.value].slice(0, 3);
  // 持久化到 localStorage
  saveFavoriteFolders();
};

// 删除常用文件夹
const removeFavoriteFolder = (folder: string) => {
  favoriteFolders.value = favoriteFolders.value.filter(f => f !== folder);
  saveFavoriteFolders();
};

// 加载常用文件夹
const loadFavoriteFolders = () => {
  try {
    const saved = localStorage.getItem(FAVORITE_KEY);
    if (saved) {
      favoriteFolders.value = JSON.parse(saved);
    }
  } catch {
    favoriteFolders.value = [];
  }
};

// 保存常用文件夹
const saveFavoriteFolders = () => {
  try {
    localStorage.setItem(FAVORITE_KEY, JSON.stringify(favoriteFolders.value));
  } catch {
    // localStorage 不可用
  }
};

// 加载 Git 仓库列表
const loadGitRepos = async () => {
  try {
    const api = getTauriAPI();
    const res = await api.getGitRepos();
    gitRepos.value = res?.data || [];
  } catch (e) {
    console.error('加载 Git 仓库列表失败:', e);
    gitRepos.value = [];
  }
};

// Computed
// 是否处于搜索模式
const isSearchMode = computed(() => sessionSearchQuery.value.trim().length > 0);

// 搜索会话内容
const searchSessions = async () => {
  const query = sessionSearchQuery.value.trim();
  if (!query) {
    searchResults.value = [];
    return;
  }
  
  isSearching.value = true;
  try {
    const result = await invoke<{ results: SearchResult[]; total: number; query: string }>('agent_search_sessions', {
      query,
      limit: 20,
    });
    searchResults.value = result.results;
  } catch (e) {
    console.error('Search failed:', e);
    searchResults.value = [];
  } finally {
    isSearching.value = false;
  }
};

// 搜索防抖
let searchDebounceTimer: number | null = null;
const debouncedSearch = () => {
  if (searchDebounceTimer) {
    clearTimeout(searchDebounceTimer);
  }
  searchDebounceTimer = window.setTimeout(() => {
    searchSessions();
  }, 300);
};

// 监听搜索输入变化
watch(sessionSearchQuery, () => {
  if (sessionSearchQuery.value.trim()) {
    debouncedSearch();
  } else {
    searchResults.value = [];
  }
});

const sourceIcon = (source: string) => {
  const icons: Record<string, string> = {
    cli: 'terminal',
    feishu: 'message',
    telegram: 'message',
    discord: 'message',
    slack: 'message',
    cron: 'clock',
  };
  return icons[source] || 'chat';
};

// 工具图标映射
const toolIconMap: Record<string, { icon: string; color: string }> = {
  // 搜索类
  'search_files': { icon: 'search', color: 'text-info' },
  'web_search': { icon: 'search', color: 'text-info' },
  'browser_*': { icon: 'browser', color: 'text-info' },
  
  // 文件操作
  'read_file': { icon: 'file', color: 'text-success' },
  'write_file': { icon: 'fileEdit', color: 'text-warning' },
  'patch': { icon: 'tool', color: 'text-warning' },
  
  // 终端/代码
  'terminal': { icon: 'terminal', color: 'text-error' },
  'execute_code': { icon: 'code', color: 'text-primary' },
  
  // Agent/技能
  'delegate_task': { icon: 'bot', color: 'text-info' },
  'skill_view': { icon: 'skill', color: 'text-secondary' },
  'skill_manage': { icon: 'skill', color: 'text-secondary' },
  'skills_list': { icon: 'list', color: 'text-secondary' },
  
  // 会话/记忆
  'session_search': { icon: 'history', color: 'text-accent' },
  'memory': { icon: 'brain', color: 'text-accent' },
  
  // 浏览器操作
  'browser_navigate': { icon: 'browser', color: 'text-info' },
  'browser_click': { icon: 'mouse', color: 'text-info' },
  'browser_snapshot': { icon: 'camera', color: 'text-info' },
  'browser_vision': { icon: 'eye', color: 'text-info' },
  
  // Cron
  'cronjob': { icon: 'clock', color: 'text-warning' },
  
  // 其他
  'clarify': { icon: 'question', color: 'text-warning' },
  'todo': { icon: 'checklist', color: 'text-success' },
  'image_generate': { icon: 'image', color: 'text-secondary' },
  'text_to_speech': { icon: 'audio', color: 'text-secondary' },
  'vision_analyze': { icon: 'eye', color: 'text-info' },
  'send_message': { icon: 'send', color: 'text-success' },
};

// 获取工具图标信息
const getToolIcon = (toolName: string): { icon: string; color: string } => {
  // 精确匹配
  if (toolIconMap[toolName]) {
    return toolIconMap[toolName];
  }
  
  // 通配符匹配 (browser_*)
  for (const [pattern, info] of Object.entries(toolIconMap)) {
    if (pattern.endsWith('*') && toolName.startsWith(pattern.slice(0, -1))) {
      return info;
    }
  }
  
  // 默认
  return { icon: 'tool', color: 'text-warning' };
};

// 格式化工具参数摘要（显示关键参数的一行）
const formatArgsSummary = (args: Record<string, unknown>): string => {
  if (!args || typeof args !== 'object') return '';
  
  // 优先显示的关键参数名
  const priorityKeys = ['path', 'url', 'message', 'query', 'command', 'file', 'text', 'pattern', 'name', 'target'];
  
  for (const key of priorityKeys) {
    if (args[key]) {
      const value = String(args[key]);
      return `${key}: ${value.length > 200 ? value.slice(0, 200) + '...' : value}`;
    }
  }
  
  // 没有优先参数，显示第一个参数
  const firstKey = Object.keys(args)[0];
  if (firstKey) {
    const value = String(args[firstKey]);
    return `${firstKey}: ${value.length > 200 ? value.slice(0, 200) + '...' : value}`;
  }
  
  return '';
};

// 格式化 todo 工具返回的任务列表为友好的 HTML
const formatTodoResult = (result: string): string => {
  try {
    // 尝试解析 JSON
    const parsed = JSON.parse(result);
    
    // 提取任务数组：支持两种格式
    // 1. 直接数组 [{id, content, status}]
    // 2. 对象格式 {todos: [...], summary: {...}}
    let tasks: Array<{ id: string; content: string; status?: string }> = [];
    let summary = null;
    
    if (Array.isArray(parsed) && parsed.length > 0 && parsed[0].id && parsed[0].content) {
      // 直接数组格式
      tasks = parsed;
    } else if (parsed.todos && Array.isArray(parsed.todos)) {
      // 对象格式 {todos, summary}
      tasks = parsed.todos;
      summary = parsed.summary;
    }
    
    // 如果成功提取到任务列表
    if (tasks.length > 0) {
      const tasksHtml = tasks.map((task: { id: string; content: string; status?: string }) => {
        const status = task.status || 'pending';
        const isCompleted = status === 'completed';
        const isCancelled = status === 'cancelled';
        const symbol = isCompleted ? '✓' : isCancelled ? '✕' : '○';
        const colorClass = isCompleted ? 'text-success' : isCancelled ? 'text-base-content/30' : 'text-base-content/40';
        return `<div class="flex items-center gap-2 py-0.5">
          <span class="${colorClass} text-xs">${symbol}</span>
          <span class="text-xs flex-1">${task.content}</span>
        </div>`;
      }).join('');
      
      // 如果有汇总信息，显示在底部
      let summaryHtml = '';
      if (summary) {
        summaryHtml = `<div class="flex items-center gap-3 mt-2 pt-1 border-t border-base-content/10 text-xs text-base-content/50">
          <span>总计 ${summary.total}</span>
          <span class="text-warning">进行中 ${summary.in_progress}</span>
          <span class="text-base-content/40">待处理 ${summary.pending}</span>
          <span class="text-success">已完成 ${summary.completed}</span>
          ${summary.cancelled > 0 ? `<span class="text-base-content/30">已取消 ${summary.cancelled}</span>` : ''}
        </div>`;
      }
      
      return `<div class="space-y-0">${tasksHtml}${summaryHtml}</div>`;
    }
    
    // 其他 JSON 格式，美化显示
    return `<pre class="text-xs whitespace-pre-wrap">${JSON.stringify(parsed, null, 2)}</pre>`;
  } catch {
    // 非 JSON，直接显示
    return `<div class="text-xs whitespace-pre-wrap">${result}</div>`;
  }
};

// 格式化工具结果（根据工具类型选择渲染方式）
const formatToolResult = (toolName: string, result: string): string => {
  // todo 工具特殊渲染
  if (toolName === 'todo') {
    return formatTodoResult(result);
  }
  
  // 其他工具，默认显示
  // 尝试解析为 JSON 并美化
  try {
    const parsed = JSON.parse(result);
    return `<pre class="text-xs whitespace-pre-wrap overflow-auto max-h-48">${JSON.stringify(parsed, null, 2)}</pre>`;
  } catch {
    return `<div class="text-xs whitespace-pre-wrap overflow-auto max-h-48">${result}</div>`;
  }
};

// Markdown 渲染缓存 — key 为消息原文，value 为渲染后的 HTML
const markdownCache = new Map<string, string>();
const MAX_CACHE = 500;

// 自定义渲染器单例（不依赖输入，只需创建一次）
const markdownRenderer = new marked.Renderer();
markdownRenderer.code = function({ text: code, lang }: { text: string; lang?: string }): string {
  const language = lang || 'plaintext';
  
  // 检测内容是否已经被 hljs 高亮过（包含 hljs-xxx 类名）
  if (code.includes('class="hljs-') || code.includes('class=\'hljs-')) {
    // 已经高亮过，直接返回，避免双重编码
    const codeId = `code-${Math.random().toString(36).substr(2, 9)}`;
    return `<div class="code-block-wrapper">
      <div class="code-header">
        <span class="code-lang">${language}</span>
        <button class="copy-btn" onclick="copyCode('${codeId}')">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
          </svg>
        </button>
      </div>
      <pre><code id="${codeId}" class="hljs">${code}</code></pre>
    </div>`;
  }
  
  const highlighted = language && hljs.getLanguage(language) 
    ? hljs.highlight(code, { language }).value 
    : hljs.highlightAuto(code).value;
  
  const codeId = `code-${Math.random().toString(36).substr(2, 9)}`;
  
  return `<div class="code-block-wrapper">
    <div class="code-header">
      <span class="code-lang">${language}</span>
      <button class="copy-btn" onclick="copyCode('${codeId}')" title="复制代码">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
          <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
        </svg>
      </button>
    </div>
    <pre><code id="${codeId}" class="hljs">${highlighted}</code></pre>
  </div>`;
};

// Markdown 渲染函数 - 添加代码块复制按钮和特殊格式处理
const renderMarkdown = (text: string | null): string => {
  if (!text) return '';
  // 缓存命中 — 避免重复解析已渲染过的消息
  const cached = markdownCache.get(text);
  if (cached) return cached;
  try {
    // 预处理：处理特殊格式的警告框
    let processedText = text
      .replace(/^\[IMPORTANT:\s*([^\]]+)\]/gm, '<div class="alert-box alert-important">⚠️ <strong>重要:</strong> $1</div>')
      .replace(/^\[WARNING:\s*([^\]]+)\]/gm, '<div class="alert-box alert-warning">⚠️ <strong>警告:</strong> $1</div>')
      .replace(/^\[NOTE:\s*([^\]]+)\]/gm, '<div class="alert-box alert-note">📝 <strong>注意:</strong> $1</div>')
      .replace(/^\[SILENT\]/gm, '<div class="alert-box alert-silent">🔇 <strong>静默模式</strong></div>')
      .replace(/^\[CONTEXT:/gm, '<div class="alert-box alert-context">📋 <strong>上下文压缩摘要</strong><br>');

    // 直接传 renderer 给 marked.parse，不修改全局 marked 配置
    const html = marked.parse(processedText, {
      renderer: markdownRenderer,
      breaks: true,
      gfm: true,
      async: false,
    }) as string;
    const result = DOMPurify.sanitize(html, {
      ADD_ATTR: ['target', 'onclick', 'id', 'title'],
      ADD_TAGS: ['button', 'svg', 'rect', 'path', 'div'],
    });

    // LRU 缓存：淘汰最旧的条目
    if (markdownCache.size >= MAX_CACHE) {
      const firstKey = markdownCache.keys().next().value;
      if (firstKey) markdownCache.delete(firstKey);
    }
    markdownCache.set(text, result);
    return result;
  } catch (e) {
    console.error('[renderMarkdown] Error:', e);
    return text;
  }
};

const formatTime = (ts: number | null | undefined) => {
  if (!ts) return '';
  const date = new Date(ts * 1000);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const days = Math.floor(diff / (1000 * 60 * 60 * 24));
  if (days === 0) {
    const hours = Math.floor(diff / (1000 * 60 * 60));
    if (hours === 0) {
      const mins = Math.floor(diff / (1000 * 60));
      return mins <= 1 ? '刚刚' : `${mins}分钟前`;
    }
    return `${hours}小时前`;
  } else if (days === 1) {
    return '昨天';
  } else if (days < 7) {
    return `${days}天前`;
  }
  return date.toLocaleDateString();
};

// 格式化消息时间戳（显示具体时间，不是相对时间）
const formatMessageTime = (ts: number | null | undefined) => {
  if (!ts) return '';
  const date = new Date(ts * 1000);
  const now = new Date();
  const isToday = date.toDateString() === now.toDateString();
  
  if (isToday) {
    return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' });
  } else {
    return date.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
  }
};

// Methods
const refreshSessions = async () => {
  loadingSessions.value = true;
  try {
    const result = await invoke<{ sessions: Session[]; total: number }>('agent_list_sessions', { limit: 50 });
    // 按 lastActive 降序排序（最近活跃的在前）
    sessions.value = result.sessions.sort((a, b) => {
      const aTime = a.lastActive || a.startedAt || 0;
      const bTime = b.lastActive || b.startedAt || 0;
      return bTime - aTime;
    });
  } catch (e) {
    console.error('Failed to list sessions:', e);
  }
  loadingSessions.value = false;
};

// 高亮搜索关键词
const highlightSnippet = (snippet: string, query: string) => {
  if (!query) return snippet;
  // FTS5 already marks matches with >>>...<<<
  // Convert to <mark> tags
  return snippet
    .replace(/>>>/g, '<mark class="bg-warning/30 text-warning px-0.5 rounded">')
    .replace(/<<</g, '</mark>');
};

// 点击搜索结果，跳转到对应会话和消息
const jumpToSearchResult = async (result: SearchResult) => {
  // 清空搜索，回到正常模式
  sessionSearchQuery.value = '';
  
  // 查找会话是否在列表中
  const session = sessions.value.find(s => s.id === result.sessionId);
  if (session) {
    await selectSession(session);
  } else {
    // 会话不在列表中，需要加载
    try {
      const sessionResult = await invoke<{ sessionId: string; messages: RawMessage[] }>('agent_get_session', {
        sessionId: result.sessionId,
      });
      // 创建临时 Session 对象
      const tempSession: Session = {
        id: result.sessionId,
        title: result.sessionTitle,
        model: result.model,
        source: result.source,
        messageCount: sessionResult.messages.length,
        preview: '',
        lastActive: result.timestamp || Date.now() / 1000,
      };
      sessions.value.unshift(tempSession);
      await selectSession(tempSession);
    } catch (e) {
      console.error('Failed to load session:', e);
    }
  }
};

const selectSession = async (session: Session) => {
  currentSessionId.value = session.id;
  currentSession.value = session;
  loadingMessages.value = true;
  messages.value = [];

  try {
    // 直接从 SQLite 读取（毫秒级），不再通过 Python bridge
    const result = await invoke<{ success: boolean; messages: RawMessage[]; sessionId: string }>('agent_list_messages', {
      sessionId: session.id,
    });
    
    if (!result.success || !result.messages) {
      console.error('Failed to load messages');
      messages.value = [];
      return;
    }
    
    // 处理消息：关联 tool_calls 和 tool 结果
    const processedMessages: Message[] = [];
    const toolResultsMap = new Map<string, string>();
    
    // 先收集所有 tool 消息的结果
    for (const m of result.messages) {
      if (m.role === 'tool' && m.toolCallId) {
        toolResultsMap.set(m.toolCallId, m.content || '');
      }
    }
    
    // 再处理 user 和 assistant 消息
    for (const m of result.messages) {
      if (m.role === 'tool') continue; // tool 消息不单独显示，合并到 assistant
      
      // 历史消息不需要显示思考内容（干扰视线）
      const msg: Message = {
        role: m.role,
        content: m.content,
        timestamp: m.timestamp,
        toolName: m.toolName,
        toolCalls: [],
      };
      
      // 如果是 assistant 消息且有 tool_calls（JSON 字符串），解析并关联结果
      if (m.role === 'assistant' && m.toolCalls) {
        try {
          const toolCallsParsed = JSON.parse(m.toolCalls) as RawToolCall[];
          for (const tc of toolCallsParsed) {
            const toolName = tc.function?.name || 'unknown';
            const toolArgs = tc.function?.arguments ? JSON.parse(tc.function.arguments) : {};
            const toolResult = toolResultsMap.get(tc.id) || '';
            
            if (!msg.toolCalls) msg.toolCalls = [];
            msg.toolCalls.push({
              name: toolName,
              args: toolArgs,
              result: toolResult,
              durationMs: 0, // 历史消息没有时长信息
              isSubAgent: toolName === 'delegate_task' || toolName === 'subagent',
              status: 'completed',
            });
          }
        } catch (e) {
          console.warn('Failed to parse tool_calls:', e);
        }
      }
      
      processedMessages.push(msg);
    }
    
    messages.value = processedMessages;
  } catch (e) {
    console.error('Failed to get session:', e);
  }

  loadingMessages.value = false;
  scrollToBottom();
};

const startNewChat = () => {
  currentSessionId.value = null;
  currentSession.value = null;
  messages.value = [];
  inputText.value = '';
  thinkingText.value = '';
  isStreaming.value = false;
};

// 自动生成会话标题（基于第一条用户消息）
const generateSessionTitle = (firstMessage: string): string => {
  // 截取前30个字符作为标题
  let title = firstMessage.trim().slice(0, 30);
  // 如果截断，添加省略号
  if (firstMessage.trim().length > 30) {
    title += '...';
  }
  return title;
};

const sendMessage = async () => {
  if (!inputText.value.trim()) return;

  // 如果正在处理，先打断当前处理
  if (isStreaming.value) {
    await abortChat();
    // 等待足够时间让 abort 完成（Python 进程被 kill）
    await new Promise(resolve => setTimeout(resolve, 200));
    // 确认状态已恢复
    if (isStreaming.value) {
      void agentLog('[sendMessage] abort 后状态仍为 streaming，强制重置');
      isStreaming.value = false;
    }
  }

  // 构建消息
  let text = inputText.value.trim();
  inputText.value = '';
  // 将已选择路径拼入消息头部（Agent 接收文本路径），同时保存结构化数据用于 UI 显示
  let pathPrefix = '';
  let msgFilePaths: PathItem[] | undefined;
  if (attachedPaths.value.length > 0) {
    pathPrefix = attachedPaths.value.map(p => p.path).join('\n') + '\n';
    msgFilePaths = [...attachedPaths.value];
    // 释放所有图片预览的 object URL
    for (const item of attachedPaths.value) {
      if (item.previewUrl) URL.revokeObjectURL(item.previewUrl);
    }
    attachedPaths.value = [];
  }
  text = pathPrefix + text;

  // 添加用户消息
  messages.value.push({
    role: 'user',
    content: text,
    timestamp: Date.now() / 1000,
    toolName: null,
    filePaths: msgFilePaths,
  });
  scrollToBottom();

  // 开始流式输出
  isStreaming.value = true;
  thinkingText.value = '';
  lastAssistantRoundEnded = false;

  try {
    // 使用选择的模型（如果有）
    const modelToUse = selectedModel.value || null;
    
    const result = await invoke<{ response: string; session_id: string; message_count: number }>('agent_chat', {
      message: text,
      sessionId: currentSessionId.value,
      model: modelToUse,
    });

    // 更新 session ID
    if (result.session_id && !currentSessionId.value) {
      currentSessionId.value = result.session_id;
      // 自动生成标题（如果是第一条消息）
      const autoTitle = generateSessionTitle(text);
      // 尝试重命名会话
      try {
        await invoke('agent_rename_session', {
          sessionId: result.session_id,
          newTitle: autoTitle,
        });
        // 更新本地 session 信息
        currentSession.value = {
          id: result.session_id,
          title: autoTitle,
          model: modelToUse || 'unknown',
          source: 'unknown',
          startedAt: Date.now() / 1000,
          endedAt: null,
          messageCount: 1,
          preview: text.slice(0, 50),
          lastActive: Date.now() / 1000,
        };
      } catch (e) {
        console.warn('Auto-title failed:', e);
      }
      // 刷新会话列表
      refreshSessions();
    }

    // invoke 返回后，消息已通过事件处理添加到 messages 数组
    // 清空流式状态
    thinkingText.value = '';
    lastAssistantRoundEnded = false;
  } catch (e) {
    console.error('Chat error:', e);
    // 添加错误消息，保存原始内容以便重试
    messages.value.push({
      role: 'assistant',
      content: `错误: ${e}`,
      timestamp: Date.now() / 1000,
      toolName: null,
      isError: true,
      retryContent: text, // 保存原始消息用于重试
    });
  }

  isStreaming.value = false;
  scrollToBottom();
  // 自动聚焦输入框，方便继续输入
  inputRef.value?.focus();
};

// 重试发送消息
const retryMessage = async (retryContent: string) => {
  if (!retryContent.trim()) return;

  // 如果正在处理，先打断当前处理
  if (isStreaming.value) {
    await abortChat();
    await new Promise(resolve => setTimeout(resolve, 100));
  }

  // 移除最后一条错误消息
  if (messages.value.length > 0 && messages.value[messages.value.length - 1].isError) {
    messages.value.pop();
  }

  // 设置输入文本并重新发送
  inputText.value = retryContent;
  await sendMessage();
};

// 取消当前处理
const abortChat = async () => {
  if (!isStreaming.value || isAborting.value) return;
  
  isAborting.value = true;
  
  // 标记当前正在输出的消息为"已停止"
  const lastMsg = messages.value[messages.value.length - 1];
  if (lastMsg && lastMsg.role === 'assistant') {
    lastMsg.isStopped = true;
  }
  
  // 不等待后端完成，立即响应用户
  invoke('agent_abort_chat').catch(e => {
    console.error('Abort error:', e);
  });
  
  // 立即重置状态，让用户感知响应
  setTimeout(() => {
    isStreaming.value = false;
    isAborting.value = false;
    lastAssistantRoundEnded = false;
    thinkingText.value = '';
  }, 100);
};

// 复制消息内容
const copyMessageContent = async (content: string | null) => {
  if (!content) return;
  try {
    await navigator.clipboard.writeText(content);
    // 可选：显示复制成功提示（用 toast 或临时状态）
  } catch (e) {
    console.error('Copy failed:', e);
  }
};

// 获取用户消息的显示内容（去除已单独展示的文件路径前缀）
const userDisplayContent = (msg: Message): string => {
  if (!msg.content) return '';
  if (!msg.filePaths || msg.filePaths.length === 0) return msg.content;
  // 跳过文件路径行（用户不可见的原始内容）
  const lines = msg.content.split('\n');
  return lines.slice(msg.filePaths.length).join('\n').trimStart();
};

// 高亮搜索匹配文本
const highlightText = (text: string | null, query: string): string => {
  if (!text || !query.trim()) return text || '';
  const escapedQuery = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const regex = new RegExp(escapedQuery, 'gi');
  return text.replace(regex, '<mark class="bg-warning/30 text-inherit px-0.5 rounded">$&</mark>');
};

// 计算已完成任务数量
const completedTasksCount = computed(() => {
  return currentTasks.value.filter(t => t.status === 'completed').length;
});

// 计算会话统计
const sessionStats = computed(() => {
  const userMessages = messages.value.filter(m => m.role === 'user');
  const assistantMessages = messages.value.filter(m => m.role === 'assistant');
  
  const totalInputTokens = messages.value.reduce((sum, m) => sum + (m.tokens?.input || 0), 0);
  const totalOutputTokens = messages.value.reduce((sum, m) => sum + (m.tokens?.output || 0), 0);
  
  return {
    userCount: userMessages.length,
    assistantCount: assistantMessages.length,
    totalMessages: messages.value.length,
    totalTokens: totalInputTokens + totalOutputTokens,
    inputTokens: totalInputTokens,
    outputTokens: totalOutputTokens,
  };
});

// 搜索消息
const searchMessages = () => {
  if (!searchQuery.value.trim()) {
    filteredMessages.value = messages.value;
    return;
  }
  const query = searchQuery.value.toLowerCase();
  filteredMessages.value = messages.value.filter(msg => 
    msg.content?.toLowerCase().includes(query)
  );
};

// 清除搜索
const clearSearch = () => {
  searchQuery.value = '';
  filteredMessages.value = messages.value;
};

// 全局快捷键处理
const handleGlobalKeydown = (e: KeyboardEvent) => {
  // ESC: 关闭附件菜单
  if (e.key === 'Escape' && showAttachMenu.value) {
    showAttachMenu.value = false;
    return;
  }
  
  // Cmd/Ctrl + K: 新对话
  if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
    e.preventDefault();
    startNewChat();
    return;
  }
  
  // Cmd/Ctrl + S: 保存/导出当前会话
  if ((e.metaKey || e.ctrlKey) && e.key === 's') {
    e.preventDefault();
    exportSession();
    return;
  }
  
  // Home: 滚动到顶部
  if (e.key === 'Home' && messages.value.length > 0) {
    scrollToTop();
    return;
  }
  
  // End: 滚动到底部
  if (e.key === 'End' && messages.value.length > 0) {
    scrollToBottom();
    return;
  }
  
  // Escape: 如果正在编辑标题，取消编辑
  if (e.key === 'Escape' && isEditingTitle.value) {
    cancelEditTitle();
    return;
  }
  
  // Escape: 如果正在流式输出，停止
  if (e.key === 'Escape' && isStreaming.value) {
    abortChat();
    return;
  }
};

// 导出会话为 Markdown
const exportSession = () => {
  if (messages.value.length === 0) return;
  
  const title = currentSession.value?.title || '新对话';
  const timestamp = new Date().toISOString().split('T')[0];
  
  let markdown = `# ${title}\n\n`;
  markdown += `> 导出时间: ${timestamp}\n> 模型: ${currentSession.value?.model || 'unknown'}\n\n---\n\n`;
  
  for (const msg of messages.value) {
    if (msg.role === 'user') {
      markdown += `## 用户\n\n${msg.content || ''}\n\n`;
    } else if (msg.role === 'assistant') {
      markdown += `## Hermes\n\n${msg.content || ''}\n\n`;
      if (msg.toolCalls && msg.toolCalls.length > 0) {
        markdown += `**工具调用:**\n`;
        for (const tool of msg.toolCalls) {
          markdown += `- ${tool.name} (${tool.durationMs}ms)\n`;
        }
        markdown += '\n';
      }
    }
  }
  
  // 下载文件
  const blob = new Blob([markdown], { type: 'text/markdown' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `${title.replace(/[^a-zA-Z0-9\u4e00-\u9fa5]/g, '_')}_${timestamp}.md`;
  a.click();
  URL.revokeObjectURL(url);
};

// 清空当前会话消息
const clearMessages = () => {
  if (messages.value.length === 0) return;
  if (!confirm('确定清空所有消息？此操作不可撤销。')) return;
  messages.value = [];
  currentTasks.value = []; // 清空任务列表
};

// 滚动到顶部
const scrollToTop = () => {
  if (messagesContainer.value) {
    messagesContainer.value.scrollTop = 0;
  }
};

// 滚动到底部（已有 scrollToBottom 函数）

const deleteCurrentSession = async () => {
  if (!currentSessionId.value) return;

  // 简单确认对话框
  if (!confirm('确定要删除当前会话吗？此操作不可撤销。')) return;

  try {
    await invoke('agent_delete_session', { sessionId: currentSessionId.value });
    sessions.value = sessions.value.filter(s => s.id !== currentSessionId.value);
    startNewChat();
  } catch (e) {
    console.error('Delete error:', e);
  }
};

// 删除指定会话（从列表直接删除）
const deleteSession = async (sessionId: string) => {
  if (!sessionId) return;

  // 简单确认对话框
  if (!confirm('确定要删除该会话吗？')) return;

  try {
    await invoke('agent_delete_session', { sessionId });
    sessions.value = sessions.value.filter(s => s.id !== sessionId);
    // 如果删除的是当前会话，清空消息
    if (currentSessionId.value === sessionId) {
      startNewChat();
    }
  } catch (e) {
    console.error('Delete error:', e);
  }
};

// 标题编辑功能
const startEditTitle = () => {
  if (!currentSession.value) return;
  isEditingTitle.value = true;
  editingTitle.value = currentSession.value.title || '';
  nextTick(() => {
    titleInputRef.value?.focus();
  });
};

const cancelEditTitle = () => {
  isEditingTitle.value = false;
  editingTitle.value = '';
};

const saveTitle = async () => {
  if (!isEditingTitle.value || !currentSessionId.value) return;
  
  const newTitle = editingTitle.value.trim();
  if (!newTitle) {
    cancelEditTitle();
    return;
  }
  
  // 如果标题没有变化，直接取消编辑
  if (newTitle === currentSession.value?.title) {
    cancelEditTitle();
    return;
  }
  
  isEditingTitle.value = false;
  
  try {
    await invoke('agent_rename_session', {
      sessionId: currentSessionId.value,
      title: newTitle,
    });
    
    // 更新本地状态
    if (currentSession.value) {
      currentSession.value.title = newTitle;
    }
    // 更新会话列表中的标题
    const session = sessions.value.find(s => s.id === currentSessionId.value);
    if (session) {
      session.title = newTitle;
    }
  } catch (e) {
    console.error('Rename error:', e);
    // 恢复原标题
    editingTitle.value = currentSession.value?.title || '';
  }
};

const checkHermes = async () => {
  try {
    const result = await invoke<{ available: boolean; error: string | null }>('agent_check_available');
    hermesAvailable.value = result.available;
  } catch (e) {
    hermesAvailable.value = false;
  }
};

const scrollToBottom = () => {
  nextTick(() => {
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight;
      showScrollToBottom.value = false;
    }
  });
};

// 检测是否需要显示"回到底部"按钮
const checkScrollPosition = () => {
  if (messagesContainer.value) {
    const { scrollTop, scrollHeight, clientHeight } = messagesContainer.value;
    // 当用户向上滚动超过 100px 时显示按钮
    const isNearBottom = scrollHeight - scrollTop - clientHeight < 100;
    showScrollToBottom.value = !isNearBottom && messages.value.length > 0;
  }
};

// Lifecycle
onMounted(async () => {
  // 全局快捷键
  document.addEventListener('keydown', handleGlobalKeydown);
  
  // 滚动监听 - 检测是否需要显示"回到底部"按钮
  if (messagesContainer.value) {
    messagesContainer.value.addEventListener('scroll', checkScrollPosition);
  }
  
// 监听流式事件
  unlistenDelta = await listen<{ text: string | null; session_id: string | null }>('agent-delta', (event) => {
    void agentLog('[agent-delta] 收到事件: ' + JSON.stringify(event.payload?.text?.slice(0, 50)) + ' session_id: ' + event.payload?.session_id);
    
    // 会话 ID 过滤：只处理当前会话的事件
    const eventSessionId = event.payload?.session_id;
    if (eventSessionId && currentSessionId.value && eventSessionId !== currentSessionId.value) {
      void agentLog('[agent-delta] 忽略非当前会话事件: event=' + eventSessionId + ' current=' + currentSessionId.value);
      return;
    }
    
    // 收到实际内容时清空思考动画
    thinkingText.value = '';
    
    if (event.payload?.text) {
      // 查找最后一个 assistant 消息
      const messagesCopy = [...messages.value].reverse();
      let currentMsg: Message | undefined = messagesCopy.find((m: Message) => m.role === 'assistant');
      
      // 检查最后一条消息是否是 user（刚发送的），如果是，需要创建新 assistant 消息
      const lastMsg = messages.value[messages.value.length - 1];
      const needsNewMsg = lastMsg?.role === 'user';
      
      // 检查是否已有空内容的 assistant 消息（由 tool_start 创建），避免重复创建
      const hasEmptyAssistant = currentMsg && !currentMsg.content && currentMsg.toolCalls && currentMsg.toolCalls.length > 0;
      
      void agentLog('[agent-delta] 当前 assistant 消息: ' + (currentMsg ? '存在' : '不存在') +
        ' lastAssistantRoundEnded: ' + lastAssistantRoundEnded +
        ' 最后一条: ' + (lastMsg?.role || 'none') +
        ' needsNewMsg: ' + needsNewMsg +
        ' hasEmptyAssistant: ' + hasEmptyAssistant);
      
      // 如果没有 assistant 消息，或上一轮已结束，或最后一条是 user（需要新消息），创建新消息
      // 但如果已有空内容的 assistant 消息（由 tool_start 创建），则复用
      if (!currentMsg || (lastAssistantRoundEnded && !hasEmptyAssistant) || needsNewMsg) {
        const newMsg: Message = {
          role: 'assistant',
          content: '',
          timestamp: Date.now() / 1000,
          toolName: null,
          toolCalls: [],
        };
        messages.value.push(newMsg);
        // 从 messages.value 获取 Vue 的 reactive proxy，确保响应式触发
        currentMsg = messages.value[messages.value.length - 1];
        lastAssistantRoundEnded = false;
        void agentLog('[agent-delta] 创建新 assistant 消息, messages.length: ' + messages.value.length);
      } else if (hasEmptyAssistant) {
        // 复用已有的空内容 assistant 消息
        lastAssistantRoundEnded = false;
        void agentLog('[agent-delta] 复用已有空 assistant 消息');
      }
      
      // 添加 delta 内容（currentMsg 是 Vue reactive proxy，修改会触发响应式）
      if (currentMsg) {
        currentMsg.content = (currentMsg.content || '') + event.payload.text;
      }
      scrollToBottom();
    }
  });

  unlistenToolStart = await listen<{ id?: string; name: string; args: unknown; session_id: string | null }>('agent-tool-start', (event) => {
    void agentLog('[agent-tool-start] 收到事件: ' + JSON.stringify(event.payload) + ' session_id: ' + event.payload?.session_id);
    
    // 会话 ID 过滤：只处理当前会话的事件
    const eventSessionId = event.payload?.session_id;
    if (eventSessionId && currentSessionId.value && eventSessionId !== currentSessionId.value) {
      void agentLog('[agent-tool-start] 忽略非当前会话事件: event=' + eventSessionId + ' current=' + currentSessionId.value);
      return;
    }
    
    // 工具开始
    const toolId = event.payload.id;
    const toolName = event.payload.name;
    const isSubAgent = toolName === 'delegate_task';
    
    // 获取当前消息（如果没有 assistant 消息，创建一个）
    const messagesCopy = [...messages.value].reverse();
    let currentMsg: Message | undefined = messagesCopy.find((m: Message) => m.role === 'assistant');
    
    // 检查最后一条消息是否是 user（刚发送的），如果是，需要创建新 assistant 消息
    const lastMsg = messages.value[messages.value.length - 1];
    const needsNewMsg = lastMsg?.role === 'user';
    
    void agentLog('[agent-tool-start] 当前 assistant 消息: ' + (currentMsg ? '存在' : '不存在') +
      ' 最后一条: ' + (lastMsg?.role || 'none') +
      ' needsNewMsg: ' + needsNewMsg + ' toolId: ' + (toolId || 'none'));
    
    // 重置轮结束标志（新的工具调用开始）
    lastAssistantRoundEnded = false;
    
    if (!currentMsg || needsNewMsg) {
      const newMsg: Message = {
        role: 'assistant',
        content: '',
        timestamp: Date.now() / 1000,
        toolName: null,
        toolCalls: [],
      };
      messages.value.push(newMsg);
      // 从 messages.value 获取 Vue 的 reactive proxy
      currentMsg = messages.value[messages.value.length - 1];
      void agentLog('[agent-tool-start] 创建新 assistant 消息, messages.length: ' + messages.value.length);
    }
    
    // 确保 toolCalls 数组存在（currentMsg 是 Vue reactive proxy）
    if (!currentMsg.toolCalls) {
      currentMsg.toolCalls = [];
    }
    
    // 添加工具调用（包含 id 用于精确匹配）
    currentMsg.toolCalls.push({
      id: toolId,
      name: toolName,
      args: event.payload.args as Record<string, unknown> || {},
      durationMs: 0,
      isSubAgent,
      status: 'running',
    });
    void agentLog('[agent-tool-start] 添加工具调用: ' + toolName + ' id: ' + (toolId || 'none') + ' toolCalls.length: ' + currentMsg.toolCalls.length);
    
    // 显示提示
    if (isSubAgent) {
      thinkingText.value = '🤖 启动子 Agent 处理任务...';
    } else {
      thinkingText.value = `🔧 调用工具: ${toolName}...`;
    }
    scrollToBottom();
  });

  unlistenToolComplete = await listen<{ id?: string; name: string; result: string | null; duration_ms: number; session_id: string | null }>('agent-tool-complete', (event) => {
    void agentLog('[agent-tool-complete] 收到事件: ' + JSON.stringify({id: event.payload.id, name: event.payload.name, duration_ms: event.payload.duration_ms, session_id: event.payload.session_id}));
    
    // 会话 ID 过滤：只处理当前会话的事件
    const eventSessionId = event.payload?.session_id;
    if (eventSessionId && currentSessionId.value && eventSessionId !== currentSessionId.value) {
      void agentLog('[agent-tool-complete] 忽略非当前会话事件: event=' + eventSessionId + ' current=' + currentSessionId.value);
      return;
    }
    
    thinkingText.value = '';
    
    // 获取当前 assistant 消息
    const messagesCopy = [...messages.value].reverse();
    const currentMsg = messagesCopy.find((m: Message) => m.role === 'assistant');
    void agentLog('[agent-tool-complete] 当前 assistant 消息: ' + (currentMsg ? '存在' : '不存在') + ' toolCalls: ' + (currentMsg?.toolCalls?.length || 0));
    
    if (currentMsg && currentMsg.toolCalls) {
      const toolId = event.payload.id;
      // 优先用 id 精确匹配，如果没有 id 则用 name 匹配（向后兼容）
      const toolCall = toolId
        ? currentMsg.toolCalls.find((t: ToolCall) => t.id === toolId)
        : currentMsg.toolCalls.find((t: ToolCall) => t.name === event.payload.name && t.status === 'running');
      if (toolCall) {
        toolCall.result = event.payload.result ?? '';
        toolCall.durationMs = event.payload.duration_ms || 0;
        toolCall.status = 'completed';
        void agentLog('[agent-tool-complete] 更新工具调用: ' + event.payload.name + ' id: ' + (toolId || 'none') + ' status: completed');
      } else {
        void agentLog('[agent-tool-complete] 未找到匹配的 running 工具调用, id: ' + (toolId || 'none'));
      }
    }
    
    // 标记当前轮次结束（下一次 delta 将创建新消息）
    lastAssistantRoundEnded = true;
    void agentLog('[agent-tool-complete] 设置 lastAssistantRoundEnded = true');
    
    // 如果是 todo 工具，更新任务列表
    if (event.payload.name === 'todo' && event.payload.result) {
      try {
        const parsed = JSON.parse(event.payload.result);
        // 支持两种格式：直接数组 或 {todos: [...], summary: {...}}
        let tasks: Array<{ id: string; content: string; status?: string }> = [];
        
        if (Array.isArray(parsed) && parsed.length > 0 && parsed[0].id && parsed[0].content) {
          // 直接数组格式
          tasks = parsed;
        } else if (parsed.todos && Array.isArray(parsed.todos) && parsed.todos.length > 0) {
          // 对象格式 {todos, summary}
          tasks = parsed.todos;
        }
        
        if (tasks.length > 0) {
          currentTasks.value = tasks.map((t: { id: string; content: string; status?: string }) => ({
            id: t.id,
            content: t.content,
            status: (['pending', 'in_progress', 'completed', 'cancelled'].includes(t.status || '') 
              ? t.status 
              : 'pending') as TaskItem['status'],
          }));
        }
      } catch {
        // 解析失败，忽略
      }
    }
    
    scrollToBottom();
  });

  // 思考动画事件已移除（服务端不再发送 agent-thinking 事件）
  
  unlistenError = await listen<{ message: string; session_id: string | null }>('agent-error', (event) => {
    void agentLog('[agent-error] 收到事件: ' + event.payload?.message + ' session_id: ' + event.payload?.session_id);
    
    // 会话 ID 过滤：只处理当前会话的事件
    const eventSessionId = event.payload?.session_id;
    if (eventSessionId && currentSessionId.value && eventSessionId !== currentSessionId.value) {
      void agentLog('[agent-error] 忽略非当前会话事件: event=' + eventSessionId + ' current=' + currentSessionId.value);
      return;
    }
    
    thinkingText.value = '';
    const messagesCopy = [...messages.value].reverse();
    const currentMsg = messagesCopy.find((m: Message) => m.role === 'assistant');
    if (currentMsg) {
      currentMsg.content = (currentMsg.content || '') + `\n[错误: ${event.payload?.message}]`;
    }
  });

  // 流式结束事件
  unlistenDone = await listen<{ response: string | null; session_id: string; message_count: number }>('agent-done', (event) => {
    void agentLog('[agent-done] 收到事件: ' + JSON.stringify(event.payload));
    // 清空思考动画
    thinkingText.value = '';
    
    // 清空流式状态
    lastAssistantRoundEnded = false;
    void agentLog('[agent-done] messages.length: ' + messages.value.length + ' 最后一条: ' + (messages.value[messages.value.length - 1]?.role || 'none'));
    
    // 恢复 UI 状态
    isStreaming.value = false;
    scrollToBottom();
  });

  // 初始化
  await loadModels(); // 加载模型列表
  await checkHermes();
  await refreshSessions();
  await loadGitRepos(); // 加载 Git 仓库列表
  loadFavoriteFolders(); // 加载常用文件夹

  // 如果URL有sessionId参数，自动选择该会话
  const sessionIdFromQuery = route.query.sessionId as string;
  if (sessionIdFromQuery) {
    const session = sessions.value.find(s => s.id === sessionIdFromQuery);
    if (session) {
      selectSession(session);
    } else {
      // 会话不存在，尝试直接加载
      try {
        const result = await invoke<{ session_id: string; messages: Message[] }>('agent_get_session', {
          sessionId: sessionIdFromQuery,
        });
        currentSessionId.value = sessionIdFromQuery;
        currentSession.value = {
          id: sessionIdFromQuery,
          title: null,
          model: 'unknown',
          source: 'unknown',
          startedAt: Date.now() / 1000,
          endedAt: null,
          messageCount: result.messages.length,
          preview: '',
        };
        messages.value = result.messages;
        scrollToBottom();
      } catch (e) {
        console.error('Failed to load session from query:', e);
      }
    }
  }

  // 自动聚焦输入框，方便立即开始对话
  inputRef.value?.focus();
});

onUnmounted(() => {
  // Properly clean up event listeners
  unlistenDelta?.();
  unlistenToolStart?.();
  unlistenToolComplete?.();
  unlistenError?.();
  unlistenDone?.();
  // 移除快捷键监听
  document.removeEventListener('keydown', handleGlobalKeydown);
  // 移除滚动监听
  if (messagesContainer.value) {
    messagesContainer.value.removeEventListener('scroll', checkScrollPosition);
  }
  // 释放残存的图片预览 object URL
  for (const item of attachedPaths.value) {
    if (item.previewUrl) URL.revokeObjectURL(item.previewUrl);
  }
});

// Watch inputText to auto-adjust textarea height
watch(inputText, () => {
  adjustTextareaHeight();
});

// Watch messages to update filteredMessages
watch(messages, () => {
  filteredMessages.value = messages.value;
}, { immediate: true });

// Watch searchQuery to filter messages
watch(searchQuery, () => {
  searchMessages();
});
</script>

<style scoped>
/* Markdown 内容样式 */
.markdown-content {
  line-height: 1.6;
  word-break: break-word;
}

.markdown-content :deep(p) {
  margin: 0.5em 0;
}

/* 行内代码样式 */
.markdown-content :deep(code:not(.hljs)) {
  background: rgba(0, 0, 0, 0.1);
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 0.9em;
  font-family: 'Fira Code', 'Monaco', 'Consolas', monospace;
}

/* 代码块包装器 */
.markdown-content :deep(.code-block-wrapper) {
  position: relative;
  margin: 0.8em 0;
  border-radius: 8px;
  overflow-x: auto;
}

/* 代码块头部 */
.markdown-content :deep(.code-header) {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 12px;
  background: rgba(0, 0, 0, 0.08);
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
}

/* 语言标签 */
.markdown-content :deep(.code-lang) {
  font-size: 0.75em;
  color: var(--color-base-content, #666);
  opacity: 0.7;
  text-transform: uppercase;
  font-weight: 500;
}

/* 复制按钮 */
.markdown-content :deep(.copy-btn) {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border-radius: 4px;
  background: transparent;
  border: 1px solid rgba(0, 0, 0, 0.1);
  cursor: pointer;
  transition: all 0.2s;
  color: var(--color-base-content, #666);
}

.markdown-content :deep(.copy-btn:hover) {
  background: rgba(0, 0, 0, 0.05);
}

.markdown-content :deep(.copy-btn.copied) {
  background: rgba(76, 175, 80, 0.2);
  border-color: #4caf50;
  color: #4caf50;
}

/* 代码块内容 */
.markdown-content :deep(.code-block-wrapper pre) {
  margin: 0;
  padding: 12px;
  background: rgba(0, 0, 0, 0.03);
  border-radius: 0;
  overflow-x: auto;
}

.markdown-content :deep(.code-block-wrapper pre code) {
  background: transparent;
  padding: 0;
  font-size: 0.85em;
  font-family: 'Fira Code', 'Monaco', 'Consolas', monospace;
}

/* 旧版 pre 样式（兼容无包装器的代码块） */
.markdown-content :deep(pre:not(.code-block-wrapper pre)) {
  background: rgba(0, 0, 0, 0.05);
  padding: 12px;
  border-radius: 8px;
  overflow-x: auto;
  margin: 0.8em 0;
}

.markdown-content :deep(pre:not(.code-block-wrapper pre) code) {
  background: transparent;
  padding: 0;
  font-size: 0.85em;
}

/* 代码高亮主题 - 适配 daisyUI 主题 */
.markdown-content :deep(.hljs-keyword),
.markdown-content :deep(.hljs-selector-tag) {
  color: #e91e63;
}

.markdown-content :deep(.hljs-string),
.markdown-content :deep(.hljs-attr) {
  color: #4caf50;
}

.markdown-content :deep(.hljs-number),
.markdown-content :deep(.hljs-literal) {
  color: #2196f3;
}

.markdown-content :deep(.hljs-comment) {
  color: #9e9e9e;
}

.markdown-content :deep(.hljs-function),
.markdown-content :deep(.hljs-title) {
  color: #ff9800;
}

.markdown-content :deep(.hljs-variable),
.markdown-content :deep(.hljs-params) {
  color: #673ab7;
}

/* 链接样式 */
.markdown-content :deep(a) {
  color: var(--color-primary);
  text-decoration: underline;
}

.markdown-content :deep(a:hover) {
  opacity: 0.8;
}

/* 列表样式 */
.markdown-content :deep(ul),
.markdown-content :deep(ol) {
  margin: 0.5em 0;
  padding-left: 1.5em;
}

.markdown-content :deep(li) {
  margin: 0.3em 0;
}

/* 表格样式 */
.markdown-content :deep(table) {
  border-collapse: collapse;
  margin: 0.8em 0;
  width: 100%;
}

.markdown-content :deep(th),
.markdown-content :deep(td) {
  border: 1px solid var(--color-base-content, #ccc);
  padding: 6px 12px;
  text-align: left;
}

.markdown-content :deep(th) {
  background: rgba(0, 0, 0, 0.05);
  font-weight: 600;
}

/* 引用块样式 */
.markdown-content :deep(blockquote) {
  border-left: 3px solid var(--color-primary);
  padding-left: 1em;
  margin: 0.8em 0;
  color: var(--color-base-content);
  opacity: 0.8;
}

/* 标题样式 */
.markdown-content :deep(h1),
.markdown-content :deep(h2),
.markdown-content :deep(h3),
.markdown-content :deep(h4) {
  margin: 1em 0 0.5em;
  font-weight: 600;
  line-height: 1.3;
}

.markdown-content :deep(h1) { font-size: 1.4em; border-bottom: 1px solid rgba(0,0,0,0.1); padding-bottom: 0.3em; }
.markdown-content :deep(h2) { font-size: 1.2em; }
.markdown-content :deep(h3) { font-size: 1.1em; }
.markdown-content :deep(h4) { font-size: 1em; }

/* 加粗和斜体 */
.markdown-content :deep(strong) {
  font-weight: 600;
}

.markdown-content :deep(em) {
  font-style: italic;
}

/* 分隔线 */
.markdown-content :deep(hr) {
  border: none;
  border-top: 1px solid rgba(0,0,0,0.1);
  margin: 1em 0;
}

/* 特殊警告框样式 */
.markdown-content :deep(.alert-box) {
  padding: 8px 12px;
  border-radius: 6px;
  margin: 0.8em 0;
  font-size: 0.9em;
}

.markdown-content :deep(.alert-important) {
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  color: #dc2626;
}

.markdown-content :deep(.alert-warning) {
  background: rgba(245, 158, 11, 0.1);
  border: 1px solid rgba(245, 158, 11, 0.3);
  color: #d97706;
}

.markdown-content :deep(.alert-note) {
  background: rgba(59, 130, 246, 0.1);
  border: 1px solid rgba(59, 130, 246, 0.3);
  color: #2563eb;
}

.markdown-content :deep(.alert-silent) {
  background: rgba(107, 114, 128, 0.1);
  border: 1px solid rgba(107, 114, 128, 0.3);
  color: #4b5563;
}

.markdown-content :deep(.alert-context) {
  background: rgba(168, 85, 247, 0.1);
  border: 1px solid rgba(168, 85, 247, 0.3);
  color: #7c3aed;
}
</style>