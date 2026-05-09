<template>
  <div class="flex flex-col h-full bg-base-200 font-sans">
    <!-- Top Bar -->
    <div class="flex items-center justify-between px-4 h-12 bg-base-100 border-b border-base-content/10 shrink-0">
      <div class="flex items-center gap-2 font-semibold text-sm text-base-content">
        <span class="text-lg">🌊</span>
        <span>Stream 管理</span>
        <span v-if="streams.length > 0" class="badge badge-primary badge-sm">{{ streams.length }}</span>
      </div>
      <div class="flex items-center gap-2">
        <span :class="[
          'inline-flex items-center gap-1 text-xs px-2.5 py-1 rounded-full mr-1',
          connectionStatus === 'connected' ? 'bg-success/10' : connectionStatus === 'connecting' ? 'bg-warning/10' : 'bg-error/10'
        ]">
          <span class="text-[10px] leading-none"><template v-if="connectionStatus === 'connected'"><SvgIcon name="dot" size="14" class="inline-block align-text-bottom" /></template><template v-else-if="connectionStatus === 'connecting'"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="6" fill="currentColor"/></svg></template><template v-else><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="6" fill="currentColor"/></svg></template></span>
          <span :class="[
            'whitespace-nowrap',
            connectionStatus === 'connected' ? 'text-success' : connectionStatus === 'connecting' ? 'text-warning' : 'text-error'
          ]">{{ connectionStatus === 'connected' ? '已连接' : connectionStatus === 'connecting' ? '连接中' : '未连接' }}</span>
        </span>
        <!-- Auto Refresh -->
        <div class="flex items-center gap-1">
          <button @click="toggleAutoRefresh" class="btn btn-ghost btn-square btn-sm" :class="{ 'bg-primary/10 text-primary': autoRefreshEnabled }" :title="autoRefreshEnabled ? '停止自动刷新' : '开启自动刷新'">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/></svg> 
          </button>
          <select v-if="autoRefreshEnabled" v-model="autoRefreshInterval" class="select select-xs" @change="restartAutoRefresh">
            <option value="3000">3s</option>
            <option value="5000">5s</option>
            <option value="10000">10s</option>
          </select>
        </div>
        <button @click="refreshAll" class="btn btn-ghost btn-square btn-sm" :disabled="loading" title="刷新">
          <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" :class="{ 'animate-spin': loading }">
            <polyline points="23 4 23 10 17 10" />
            <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
          </svg>
        </button>
      </div>
    </div>

    <div class="flex flex-1 overflow-hidden">
      <!-- Left: Stream List -->
      <div class="w-72 border-r border-base-content/10 flex flex-col bg-base-100 shrink-0">
        <div class="px-4 py-3 border-b border-base-content/10">
          <span class="font-semibold text-xs text-base-content">Streams</span>
        </div>
        <div class="px-4 py-3 border-b border-base-content/10">
          <input
            v-model="streamPattern"
            @keydown.enter="refreshStreams"
            class="input input-sm w-full bg-base-200"
            placeholder="搜索 stream，如 * 或 my-stream-*"
          />
        </div>
        <div class="flex-1 overflow-y-auto p-2 space-y-1">
          <div v-if="loading && !selectedStream" class="py-5 text-center text-base-content/60">加载中...</div>
          <div v-else-if="streams.length === 0" class="flex flex-col items-center justify-center h-full text-base-content/60 text-center px-5 py-10">
            <div class="text-5xl mb-3 opacity-50">🌊</div>
            <div class="text-sm font-medium mb-1">未找到 Stream</div>
            <div class="text-xs opacity-70">使用上方搜索框查找 stream 类型的 key</div>
          </div>
          <div
            v-for="s in filteredStreams"
            :key="s.name"
            :class="[
              'flex items-center justify-between px-3 py-2.5 rounded-lg cursor-pointer transition-all hover:bg-base-200',
              selectedStream === s.name ? 'bg-primary/10 ring-1 ring-primary' : ''
            ]"
            @click="selectStream(s.name)"
          >
            <div class="flex items-center gap-2 min-w-0">
              <span class="text-base shrink-0">🌊</span>
              <span class="text-xs text-base-content truncate" :title="s.name">{{ s.name }}</span>
            </div>
            <div class="flex gap-1 shrink-0">
              <span :class="[
                'badge badge-xs',
                s.pendingCount > 10 ? 'badge-error font-semibold' :
                s.pendingCount > 0 ? 'badge-warning' :
                'badge-ghost'
              ]" :title="`${s.pendingCount} 条 pending`">{{ s.pendingCount }}</span>
              <span class="badge badge-ghost badge-xs" :title="`${s.length} 条消息`">{{ s.length }}</span>
              <span v-if="s.groups > 0" class="badge badge-warning badge-xs" :title="`${s.groups} 个消费组`">{{ s.groups }}g</span>
            </div>
          </div>

          <!-- Load More Button -->
          <div v-if="hasMoreStreams" class="text-center py-2">
            <button @click="loadMoreStreams" :disabled="loadingMore" class="btn btn-outline btn-primary btn-xs">
              {{ loadingMore ? '加载中...' : '加载更多 Streams' }}
            </button>
          </div>

          <!-- Delay Queues Section -->
          <div v-if="delayQueues.length > 0" class="mt-2 border-t border-base-content/10 pt-2">
            <div class="flex items-center gap-1.5 px-3 py-2 cursor-pointer rounded-md hover:bg-base-200 transition-colors" @click="delaySectionCollapsed = !delaySectionCollapsed">
              <span class="text-[10px] text-base-content/60">{{ delaySectionCollapsed ? '▶' : '▼' }}</span>
              <span class="text-xs font-semibold text-base-content"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>  延迟队列</span>
              <span class="badge badge-primary badge-xs">{{ delayQueues.length }}</span>
            </div>
            <div v-show="!delaySectionCollapsed" class="p-1 space-y-0.5">
              <div
                v-for="dq in delayQueues"
                :key="dq.name"
                :class="[
                  'flex items-center gap-1.5 px-3 py-2 rounded-md cursor-pointer transition-colors hover:bg-base-200 mb-0.5',
                  selectedDelayQueue === dq.name ? 'bg-primary/10 ring-1 ring-primary' : ''
                ]"
                @click="selectDelayQueue(dq.name)"
              >
                <span class="text-sm"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg> </span>
                <span class="text-xs text-base-content flex-1 truncate" :title="dq.name">{{ dq.name.replace('delay:', '') }}</span>
                <span class="badge badge-success badge-xs">{{ dq.count }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Right: Stream Detail -->
      <div class="flex-1 flex flex-col overflow-hidden">
        <template v-if="!selectedStream && !selectedDelayQueue">
          <div class="flex flex-col items-center justify-center h-full text-base-content/60 text-center px-5 py-10">
            <div class="text-5xl mb-3 opacity-50">👈</div>
            <div class="text-sm font-medium mb-1">从左侧选择一个 Stream 或延迟队列</div>
          </div>
        </template>

        <!-- ===== Delay Queue View ===== -->
        <template v-if="selectedDelayQueue">
          <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10 bg-base-100 gap-3 shrink-0">
            <div class="min-w-0">
              <h3 class="m-0 text-sm font-semibold text-base-content truncate" :title="selectedDelayQueue"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>  {{ selectedDelayQueue }}</h3>
              <div class="text-xs text-base-content/60">
                <span>{{ delayMessages.length }} 条待到期消息</span>
              </div>
            </div>
            <div class="flex gap-1.5 shrink-0">
              <button @click="refreshDelayQueue" class="btn btn-ghost btn-sm"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/></svg>  刷新</button>
              <button @click="selectedDelayQueue = ''" class="btn btn-ghost btn-sm"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg> 关闭</button>
            </div>
          </div>
          <div class="flex-1 overflow-y-auto px-4 py-3 space-y-2.5 bg-base-200">
            <div v-if="delayLoading" class="py-5 text-center text-base-content/60">加载中...</div>
            <div v-else-if="delayMessages.length === 0" class="flex flex-col items-center justify-center h-full text-base-content/60 text-center px-5 py-10">
              <div class="text-5xl mb-3 opacity-50"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg> </div>
              <div class="text-sm font-medium mb-1">暂无延迟消息</div>
            </div>
            <div v-for="dm in delayMessages" :key="dm.value + dm.score" class="bg-base-100 rounded-xl p-3 mb-2.5 shadow-sm border border-base-content/10 transition-all hover:shadow-md border-l-4 border-l-warning">
              <div class="flex items-center justify-between mb-2">
                <span class="text-[11px] text-base-content/60 font-medium font-mono">Score: {{ formatTimestamp(dm.score) }}</span>
                <span :class="['text-xs font-semibold', dm.remainingMs <= 0 ? 'text-error' : 'text-success']">
                  {{ dm.remainingMs <= 0 ? '已到期' : `剩余 ${formatDuration(dm.remainingMs)}` }}
                </span>
              </div>
              <div class="flex flex-col gap-1">
                <pre class="m-0 text-xs font-mono text-success whitespace-pre-wrap break-all max-h-[120px] overflow-y-auto leading-relaxed">{{ formatJsonPreview(dm.messageJson) }}</pre>
              </div>
              <div class="flex gap-1.5 mt-2 pt-2 border-t border-base-content/10">
                <button v-if="dm.remainingMs <= 0" @click="fireDelayMessage(dm)" class="btn btn-ghost btn-xs"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg> 立即投递</button>
                <button @click="deleteDelayMessage(dm)" class="btn btn-ghost btn-xs text-error"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>  删除</button>
              </div>
            </div>
          </div>
        </template>

        <!-- ===== Stream Detail View ===== -->
        <template v-if="selectedStream && !selectedDelayQueue">
          <!-- Stream Info Header -->
          <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10 bg-base-100 gap-3 shrink-0">
            <div class="min-w-0">
              <h3 class="m-0 text-sm font-semibold text-base-content truncate" :title="selectedStream">{{ selectedStream }}</h3>
              <div class="text-xs text-base-content/60">
                <span v-if="streamInfo">{{ streamInfo.length }} 条消息</span>
                <span v-if="streamInfo" class="mx-1">·</span>
                <span v-if="streamInfo">{{ streamInfo.groups }} 个消费组</span>
                <span v-if="totalPending > 0" class="mx-1">·</span>
                <span v-if="totalPending > 0" class="text-error font-semibold">{{ totalPending }} pending</span>
              </div>
            </div>
            <div class="flex gap-1.5 shrink-0">
              <button @click="openAddMessage" class="btn btn-primary btn-sm"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>  投递消息</button>
              <button @click="showGroupModal = true" class="btn btn-ghost btn-sm"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>  创建消费组</button>
              <button @click="showTrimModal = true" class="btn btn-ghost btn-sm"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="6" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><line x1="20" y1="4" x2="8.12" y2="15.88"/><line x1="14.47" y1="14.48" x2="20" y2="20"/><line x1="8.12" y1="8.12" x2="12" y2="12"/></svg> 清理</button>
              <button @click="deleteStream" class="btn btn-error btn-sm"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>  删除</button>
            </div>
          </div>

          <!-- Tabs -->
          <div class="flex border-b border-base-content/10 bg-base-100 shrink-0">
            <button :class="[
              'px-4 py-2.5 border-none bg-transparent cursor-pointer text-xs text-base-content/60 transition-colors relative hover:text-base-content',
              detailTab === 'messages' ? 'text-primary font-medium after:absolute after:bottom-0 after:inset-x-0 after:h-0.5 after:bg-primary after:rounded-t-sm' : ''
            ]" @click="detailTab = 'messages'">
              消息列表 <span v-if="messages.length" class="badge badge-ghost badge-xs ml-1">{{ messages.length }}</span>
            </button>
            <button :class="[
              'px-4 py-2.5 border-none bg-transparent cursor-pointer text-xs text-base-content/60 transition-colors relative hover:text-base-content',
              detailTab === 'stats' ? 'text-primary font-medium after:absolute after:bottom-0 after:inset-x-0 after:h-0.5 after:bg-primary after:rounded-t-sm' : ''
            ]" @click="detailTab = 'stats'; loadStats()">
              <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="20" x2="18" y2="10"/><line x1="12" y1="20" x2="12" y2="4"/><line x1="6" y1="20" x2="6" y2="14"/></svg>  统计
            </button>
            <button :class="[
              'px-4 py-2.5 border-none bg-transparent cursor-pointer text-xs text-base-content/60 transition-colors relative hover:text-base-content',
              detailTab === 'groups' ? 'text-primary font-medium after:absolute after:bottom-0 after:inset-x-0 after:h-0.5 after:bg-primary after:rounded-t-sm' : ''
            ]" @click="detailTab = 'groups'">
              消费组 <span v-if="groups.length" class="badge badge-ghost badge-xs ml-1">{{ groups.length }}</span>
            </button>
          </div>

          <!-- Messages Tab -->
          <template v-if="detailTab === 'messages'">
            <div class="flex items-center gap-2 px-4 py-2 border-b border-base-content/10 bg-base-200 flex-wrap">
              <div class="flex items-center gap-1.5 flex-1 min-w-[200px]">
                <input v-model="msgStart" class="input input-sm flex-1 font-mono bg-base-100" placeholder="起始 ID (默认 -)" />
                <span class="text-base-content/60">→</span>
                <input v-model="msgEnd" class="input input-sm flex-1 font-mono bg-base-100" placeholder="结束 ID (默认 +)" />
              </div>
              <!-- Search -->
              <div class="min-w-[150px]">
                <input v-model="messageSearchQuery" class="input input-sm bg-base-100 w-full" placeholder="搜索消息内容..." />
              </div>
              <button @click="loadMessages" class="btn btn-ghost btn-xs">加载</button>
            </div>
            <div class="flex-1 overflow-y-auto px-4 py-3 space-y-2.5 bg-base-200">
              <div v-if="msgLoading" class="py-5 text-center text-base-content/60">加载中...</div>
              <div v-else-if="filteredMessages.length === 0" class="flex flex-col items-center justify-center h-full text-base-content/60 text-center px-5 py-10">
                <div class="text-5xl mb-3 opacity-50">📭</div>
                <div class="text-sm font-medium mb-1">{{ messageSearchQuery ? '未找到匹配的消息' : '暂无消息' }}</div>
                <div v-if="messageSearchQuery" class="text-xs opacity-70">尝试修改搜索关键词</div>
              </div>
              <div v-for="msg in filteredMessages" :key="msg.id" :class="[
                'bg-base-100 rounded-xl shadow-sm border border-base-content/10 transition-all hover:shadow-md',
                msg.envelope ? 'p-0 overflow-hidden' : 'p-3 mb-2.5 border-l-4',
                !msg.envelope ? ((msgConsumptionStatus.get(msg.id)?.status || 'new') === 'consumed' ? 'border-l-success' :
                (msgConsumptionStatus.get(msg.id)?.status || 'new') === 'pending' ? 'border-l-warning' :
                'border-l-base-content/30') : ''
              ]">
                <!-- Envelope-style message card -->
                <template v-if="msg.envelope">
                  <div :class="[
                    'p-3 transition-colors border-l-4',
                    (msgConsumptionStatus.get(msg.id)?.status || 'new') === 'consumed' ? 'border-l-success' :
                    (msgConsumptionStatus.get(msg.id)?.status || 'new') === 'pending' ? 'border-l-warning' :
                    'border-l-base-content/30'
                  ]">
                    <div class="flex items-center justify-between mb-1.5">
                      <div class="flex items-center gap-1.5">
                        <span class="text-base">{{ getConsumptionIcon(msgConsumptionStatus.get(msg.id)?.status || 'new') }}</span>
                        <span class="text-xs font-semibold text-primary font-mono">{{ msg.envelope.messageType || 'UNKNOWN' }}</span>
                        <span v-if="msgConsumptionStatus.size > 0" :class="[
                          'badge badge-xs font-medium',
                          (msgConsumptionStatus.get(msg.id)?.status || 'new') === 'consumed' ? 'badge-success' :
                          (msgConsumptionStatus.get(msg.id)?.status || 'new') === 'pending' ? 'badge-warning' :
                          'badge-ghost'
                        ]">
                          {{ getConsumptionLabel(msgConsumptionStatus.get(msg.id)?.status || 'new') }}
                        </span>
                      </div>
                      <div class="flex gap-1">
                        <button @click="copyText(msg.envelope.messageId || '')" class="btn btn-ghost btn-xs px-1" title="复制 Message ID"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg> </button>
                        <button @click="deleteMessage(msg.id)" class="btn btn-ghost btn-xs px-1" title="删除消息"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg> </button>
                      </div>
                    </div>
                    <div class="flex items-center gap-1 text-xs text-base-content/60 mb-2 flex-wrap">
                      <span v-if="msg.envelope.messageId">ID: <code class="font-mono bg-base-200 px-1 rounded text-[10px]">{{ shortId(msg.envelope.messageId) }}</code></span>
                      <span class="mx-1">|</span>
                      <span v-if="msg.envelope.createdAt"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg> {{ formatTime(msg.envelope.createdAt) }}</span>
                      <span class="mx-1" v-if="msg.envelope.traceId">|</span>
                      <span v-if="msg.envelope.traceId">Trace: <code class="font-mono bg-base-200 px-1 rounded text-[10px]">{{ shortId(msg.envelope.traceId) }}</code></span>
                      <span class="mx-1" v-if="msg.envelope.tenantId">|</span>
                      <span v-if="msg.envelope.tenantId">Tenant: {{ msg.envelope.tenantId }}</span>
                    </div>
                    <div class="bg-base-200 rounded-lg p-2.5 mb-2">
                      <div class="text-xs font-semibold text-base-content/60 mb-1">Payload:</div>
                      <pre class="m-0 text-xs font-mono text-success whitespace-pre-wrap break-all max-h-[200px] overflow-y-auto leading-relaxed">{{ formatJsonDisplay(msg.envelope.payload) }}</pre>
                    </div>
                    <!-- Raw data toggle -->
                    <div class="text-center">
                      <button @click="msg.showRaw = !msg.showRaw" class="btn btn-ghost btn-xs">
                        {{ msg.showRaw ? '收起原始数据' : '查看原始数据' }}
                      </button>
                    </div>
                    <div v-if="msg.showRaw" class="bg-base-200 rounded-lg p-2.5 mt-2">
                      <pre class="m-0 text-[11px] font-mono text-base-content/60 whitespace-pre-wrap break-all max-h-[150px] overflow-y-auto leading-snug">{{ formatJsonDisplay(msg.rawJson) }}</pre>
                    </div>
                  </div>
                </template>
                <!-- Legacy flat fields display -->
                <template v-else>
                  <div class="flex items-center justify-between mb-2">
                    <div class="flex items-center gap-2">
                      <span class="text-[11px] text-base-content/60 font-medium font-mono" :title="msg.id">{{ formatStreamId(msg.id) }}</span>
                      <span v-if="msgConsumptionStatus.size > 0" :class="[
                        'badge badge-xs font-medium',
                        (msgConsumptionStatus.get(msg.id)?.status || 'new') === 'consumed' ? 'badge-success' :
                        (msgConsumptionStatus.get(msg.id)?.status || 'new') === 'pending' ? 'badge-warning' :
                        'badge-ghost'
                      ]">
                        {{ getConsumptionIcon(msgConsumptionStatus.get(msg.id)?.status || 'new') }} {{ getConsumptionLabel(msgConsumptionStatus.get(msg.id)?.status || 'new') }}
                      </span>
                    </div>
                    <div class="flex gap-1">
                      <button @click="copyText(msg.id)" class="btn btn-ghost btn-xs px-1" title="复制 ID"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg> </button>
                      <button @click="deleteMessage(msg.id)" class="btn btn-ghost btn-xs px-1" title="删除消息"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg> </button>
                    </div>
                  </div>
                  <div class="flex flex-col gap-1">
                    <div v-for="(value, fieldKey) in msg.fields" :key="fieldKey" class="flex items-baseline gap-1 text-xs leading-relaxed">
                      <span class="font-semibold text-purple-600 dark:text-purple-400 font-mono shrink-0">{{ fieldKey }}</span>
                      <span class="text-base-content/60">=</span>
                      <pre class="m-0 text-xs font-mono text-base-content whitespace-pre-wrap break-all max-h-[120px] overflow-y-auto leading-relaxed" :class="{ 'text-success dark:text-emerald-400': isJSON(value) }">{{ formatValue(value) }}</pre>
                    </div>
                  </div>
                </template>
              </div>
            </div>
          </template>

          <!-- Stats Tab -->
          <template v-if="detailTab === 'stats'">
            <div class="flex-1 overflow-y-auto p-4 bg-base-200">
              <div v-if="statsLoading" class="py-5 text-center text-base-content/60">加载中...</div>
              <template v-else>
                <!-- Section Title -->
                <div class="text-xs font-semibold text-base-content mt-0 mb-2.5 pb-1.5 border-b border-base-content/10 first:mt-0"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="20" x2="18" y2="10"/><line x1="12" y1="20" x2="12" y2="4"/><line x1="6" y1="20" x2="6" y2="14"/></svg>  Stream 概览</div>
                <div class="grid grid-cols-4 gap-3 mb-5">
                  <div class="bg-base-100 border border-base-content/10 rounded-xl p-4 text-center">
                    <div class="text-xs text-base-content/60 mb-1">消息总数</div>
                    <div class="text-2xl font-bold text-base-content">{{ streamInfo?.length || 0 }}</div>
                  </div>
                  <div class="bg-base-100 border border-base-content/10 rounded-xl p-4 text-center">
                    <div class="text-xs text-base-content/60 mb-1">消费组数</div>
                    <div class="text-2xl font-bold text-base-content">{{ groups.length }}</div>
                  </div>
                  <div class="bg-base-100 border border-base-content/10 rounded-xl p-4 text-center">
                    <div class="text-xs text-base-content/60 mb-1">总 Pending</div>
                    <div :class="[
                      'text-2xl font-bold',
                      totalPending > 10 ? 'text-error' : totalPending > 0 ? 'text-warning' : 'text-base-content'
                    ]">{{ totalPending }}</div>
                  </div>
                  <div class="bg-base-100 border border-base-content/10 rounded-xl p-4 text-center">
                    <div class="text-xs text-base-content/60 mb-1">消费者总数</div>
                    <div class="text-2xl font-bold text-base-content">{{ totalConsumers }}</div>
                  </div>
                </div>

                <!-- Health Distribution -->
                <div class="text-xs font-semibold text-base-content mt-4 mb-2.5 pb-1.5 border-b border-base-content/10">💚 消费者健康分布</div>
                <div class="flex h-9 rounded-xl overflow-hidden mb-2 border border-base-content/10">
                  <div class="flex items-center justify-center text-xs font-semibold text-white transition-all duration-300 min-w-0 overflow-hidden bg-success" :style="{ width: healthPercentages.healthy + '%' }">
                    <span v-if="healthPercentages.healthy > 15">{{ consumerStats.healthy }} 活跃</span>
                  </div>
                  <div class="flex items-center justify-center text-xs font-semibold text-white transition-all duration-300 min-w-0 overflow-hidden bg-warning" :style="{ width: healthPercentages.idle + '%' }">
                    <span v-if="healthPercentages.idle > 15">{{ consumerStats.idle }} 空闲</span>
                  </div>
                  <div class="flex items-center justify-center text-xs font-semibold text-white transition-all duration-300 min-w-0 overflow-hidden bg-base-content/30" :style="{ width: healthPercentages.stale + '%' }">
                    <span v-if="healthPercentages.stale > 15">{{ consumerStats.stale }} 失联</span>
                  </div>
                </div>
                <div class="flex gap-4 mb-4 px-3 py-2 bg-base-100 rounded-lg border border-base-content/10">
                  <span class="flex items-center gap-1.5 text-xs text-base-content/60"><span class="w-2.5 h-2.5 rounded-full bg-success"></span> 活跃 (idle &lt; 1h)</span>
                  <span class="flex items-center gap-1.5 text-xs text-base-content/60"><span class="w-2.5 h-2.5 rounded-full bg-warning"></span> 空闲 (1h ~ 24h)</span>
                  <span class="flex items-center gap-1.5 text-xs text-base-content/60"><span class="w-2.5 h-2.5 rounded-full bg-base-content/30"></span> 失联 (idle &gt; 24h)</span>
                </div>

                <!-- Per-group breakdown -->
                <div v-for="g in groupStats" :key="g.name" class="bg-base-100 border border-base-content/10 rounded-xl mb-3 overflow-hidden">
                  <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10 bg-base-200">
                    <div class="flex flex-col gap-1">
                      <span class="text-xs font-semibold text-base-content"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>  {{ g.name }}</span>
                      <div class="flex gap-1.5">
                        <span class="badge badge-xs font-medium badge-success">{{ g.healthyConsumers }} 活跃</span>
                        <span class="badge badge-xs font-medium badge-warning">{{ g.idleConsumers }} 空闲</span>
                        <span class="badge badge-xs font-medium badge-ghost">{{ g.staleConsumers }} 失联</span>
                      </div>
                    </div>
                    <span :class="[
                      'text-xs font-semibold',
                      g.pendingCount > 10 ? 'text-error' : g.pendingCount > 0 ? 'text-warning' : ''
                    ]">
                      pending: {{ g.pendingCount }}
                    </span>
                  </div>
                  <!-- Consumer grid -->
                  <div class="grid grid-cols-[repeat(auto-fill,minmax(160px,1fr))] gap-2 p-3">
                    <div v-for="c in g.consumers" :key="c.name" :class="[
                      'bg-base-200 border border-base-content/10 rounded-lg p-2.5 relative transition-all hover:shadow-md hover:-translate-y-0.5 border-t-3',
                      getConsumerHealthClass(c.pending || 0, c.idle || 0) === 'card-healthy' ? 'border-t-success' :
                      getConsumerHealthClass(c.pending || 0, c.idle || 0) === 'card-idle' ? 'border-t-warning' :
                      'border-t-base-content/30'
                    ]">
                      <div class="flex items-center justify-between gap-1.5 mb-1.5">
                        <span class="text-xs font-mono text-base-content font-semibold truncate" :title="c.name">{{ extractPodId(c.name) }}</span>
                        <span v-if="isRetrier(c.name)" class="text-[9px] px-1 py-0.5 rounded bg-purple-100 dark:bg-purple-900 text-purple-700 dark:text-purple-300 font-semibold uppercase shrink-0">retrier</span>
                      </div>
                      <div class="flex items-center gap-2 text-[11px] text-base-content/60">
                        <span class="whitespace-nowrap">⏱ {{ formatDuration(c.idle || 0) }}</span>
                        <span class="whitespace-nowrap" v-if="c.pending !== undefined">📬 {{ c.pending }}</span>
                      </div>
                      <!-- Health indicator dot -->
                      <div :class="[
                        'absolute top-1.5 right-1.5 w-2 h-2 rounded-full',
                        getConsumerHealthDot(c.pending || 0, c.idle || 0) === 'dot-healthy' ? 'bg-success' :
                        getConsumerHealthDot(c.pending || 0, c.idle || 0) === 'dot-idle' ? 'bg-warning' :
                        getConsumerHealthDot(c.pending || 0, c.idle || 0) === 'dot-stale' ? 'bg-base-content/30' :
                        'bg-error'
                      ]"></div>
                    </div>
                  </div>
                </div>
              </template>
            </div>
          </template>

          <!-- Groups Tab -->
          <template v-if="detailTab === 'groups'">
            <div class="flex-1 overflow-y-auto px-4 py-3 bg-base-200 space-y-2.5">
              <div v-if="groupLoading" class="py-5 text-center text-base-content/60">加载中...</div>
              <div v-else-if="groups.length === 0" class="flex flex-col items-center justify-center h-full text-base-content/60 text-center px-5 py-10">
                <div class="text-5xl mb-3 opacity-50"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg> </div>
                <div class="text-sm font-medium mb-1">暂无消费组</div>
                <div class="text-xs opacity-70">点击 "创建消费组" 添加</div>
              </div>
              <div v-for="g in groups" :key="g.name" class="bg-base-100 rounded-xl p-3 border border-base-content/10">
                <div class="flex items-center justify-between">
                  <div class="min-w-0">
                    <span class="font-semibold text-xs text-base-content"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>  {{ g.name }}</span>
                    <div class="text-[11px] text-base-content/60 flex gap-2 mt-1">
                      <span v-if="g.pending !== undefined">pending: {{ g.pending }}</span>
                      <span v-if="g.consumers !== undefined">consumers: {{ g.consumers }}</span>
                      <span v-if="g['lastDeliveredId']">last-id: {{ g['lastDeliveredId'] }}</span>
                    </div>
                  </div>
                  <div class="flex gap-1">
                    <button @click="selectGroup(g.name)" class="btn btn-ghost btn-xs">详情</button>
                    <button @click="destroyGroup(g.name)" class="btn btn-ghost btn-xs px-1" title="删除消费组"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg> </button>
                  </div>
                </div>
              </div>
            </div>
          </template>
        </template>
      </div>
    </div>

    <!-- Group Detail Drawer -->
    <div v-if="selectedGroup" class="border-t-2 border-primary bg-base-100 max-h-72 flex flex-col shrink-0">
      <div class="flex items-center justify-between px-4 py-2 border-b border-base-content/10">
        <div class="font-semibold text-xs text-base-content flex items-center gap-2">
          <span><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg> </span>
          <span>{{ selectedGroup }}</span>
          <span class="font-normal text-base-content/60 text-xs">@ {{ selectedStream }}</span>
        </div>
        <div class="flex items-center gap-2">
          <div class="flex items-center gap-1">
            <label class="text-[11px] text-base-content/60 whitespace-nowrap">Claim 消费者:</label>
            <input v-model="claimConsumerName" class="input input-xs w-20 font-mono bg-base-200" placeholder="admin" title="Claim 目标消费者名称" />
          </div>
          <button @click="selectedGroup = ''" class="border-none bg-transparent cursor-pointer text-lg text-base-content/60 p-1 rounded hover:bg-base-200 transition-colors"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg></button>
        </div>
      </div>
      <div class="flex border-b border-base-content/10 px-4">
        <button :class="[
          'px-4 py-2 border-none bg-transparent cursor-pointer text-xs text-base-content/60 relative transition-colors hover:text-base-content',
          groupDetailTab === 'consumers' ? 'text-primary font-medium after:absolute after:bottom-0 after:inset-x-0 after:h-0.5 after:bg-primary' : ''
        ]" @click="groupDetailTab = 'consumers'">消费者</button>
        <button :class="[
          'px-4 py-2 border-none bg-transparent cursor-pointer text-xs text-base-content/60 relative transition-colors hover:text-base-content',
          groupDetailTab === 'pending' ? 'text-primary font-medium after:absolute after:bottom-0 after:inset-x-0 after:h-0.5 after:bg-primary' : ''
        ]" @click="groupDetailTab = 'pending'">Pending 消息</button>
      </div>
      <!-- Consumers -->
      <template v-if="groupDetailTab === 'consumers'">
        <div class="flex-1 overflow-y-auto px-4 py-2">
          <div v-if="consumersLoading" class="py-5 text-center text-base-content/60">加载中...</div>
          <div v-else-if="consumers.length === 0" class="flex flex-col items-center justify-center h-full text-base-content/60 text-center px-5 py-10"><div class="text-sm font-medium mb-1">暂无消费者</div></div>
          <div v-for="c in consumers" :key="c.name" class="flex items-center justify-between px-3 py-2 rounded-md mb-1 bg-base-200 text-xs">
            <span class="font-medium text-base-content">🟢 {{ c.name }}</span>
            <span class="text-base-content/60">pending: {{ c.pending }} | idle: {{ c.idle }}</span>
          </div>
        </div>
      </template>
      <!-- Pending -->
      <template v-if="groupDetailTab === 'pending'">
        <div class="flex-1 overflow-y-auto px-4 py-2">
          <div v-if="pendingLoading" class="py-5 text-center text-base-content/60">加载中...</div>
          <div v-else-if="pendingMessages.length === 0" class="flex flex-col items-center justify-center h-full text-base-content/60 text-center px-5 py-10"><div class="text-sm font-medium mb-1">暂无 pending 消息</div></div>
          <div v-for="p in pendingMessages" :key="p.id" :class="[
            'flex items-center justify-between px-3 py-2 rounded-md mb-1 text-xs',
            p.idleTime > 300000 ? 'bg-error/10 border-l-3 border-l-error' : 'bg-base-200'
          ]">
            <div class="flex items-center gap-1 min-w-0">
              <span class="font-mono text-base-content font-medium" :title="p.id">{{ formatStreamId(p.id) }}</span>
              <span class="text-base-content/60 ml-3">
                consumer: {{ p.consumer }} | delivery: {{ p.timesDelivered }} | idle: {{ formatDuration(p.idleTime) }}
              </span>
            </div>
            <div class="flex gap-1 shrink-0">
              <button @click="retryPending(p.id)" class="btn btn-ghost btn-xs text-warning" title="重试（重新投递）">🔁 重试</button>
              <button @click="claimPending(p.id)" class="btn btn-ghost btn-xs" title="Claim"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="17" x2="12" y2="22"/><path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V6h1a2 2 0 0 0 0-4H8a2 2 0 0 0 0 4h1v4.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24Z"/></svg>  Claim</button>
              <button @click="ackPending(p.id)" class="btn btn-ghost btn-xs" title="Ack"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>  Ack</button>
            </div>
          </div>
        </div>
      </template>
    </div>

    <!-- Add Message Modal -->
    <div v-if="showAddModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showAddModal = false">
      <div class="bg-base-100 rounded-xl w-[560px] max-w-[90vw] shadow-2xl border border-base-content/10">
        <div class="flex items-center justify-between px-5 py-4 border-b border-base-content/10">
          <h3 class="m-0 text-sm font-semibold text-base-content"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>  投递消息 (XADD)</h3>
          <button @click="showAddModal = false" class="border-none bg-transparent cursor-pointer text-lg text-base-content/60 p-1 rounded hover:bg-base-200 transition-colors"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg></button>
        </div>
        <div class="p-5">
          <div class="flex gap-4">
            <div class="flex-1 mb-4">
              <label class="block text-xs font-semibold text-base-content/60 mb-1.5">Stream Key</label>
              <input v-model="addKey" class="input input-sm w-full bg-base-200" placeholder="stream key" />
            </div>
            <div class="flex-1 mb-4">
              <label class="block text-xs font-semibold text-base-content/60 mb-1.5">MAXLEN (可选, 0 表示不限制)</label>
              <input v-model.number="addMaxlen" type="number" class="input input-sm w-full bg-base-200" placeholder="0" min="0" />
            </div>
          </div>
          <div class="mb-4">
            <label class="block text-xs font-semibold text-base-content/60 mb-1.5">字段值 (JSON 格式)</label>
            <textarea v-model="addFieldsText" class="textarea textarea-sm w-full font-mono bg-base-200 resize-y" rows="6" placeholder='{"field1": "value1", "field2": "value2"}'></textarea>
          </div>
          <div class="mb-4">
            <label class="flex items-center gap-1.5 cursor-pointer">
              <input type="checkbox" v-model="addAsMqMessage" class="checkbox checkbox-sm" /> 包装为 MqMessage 信封格式
            </label>
          </div>
        </div>
        <div class="flex justify-end gap-2 px-5 py-4 border-t border-base-content/10">
          <button @click="showAddModal = false" class="btn btn-ghost">取消</button>
          <button @click="addMessage" class="btn btn-primary" :disabled="!addKey || !addFieldsText">添加</button>
        </div>
      </div>
    </div>

    <!-- Create Group Modal -->
    <div v-if="showGroupModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showGroupModal = false">
      <div class="bg-base-100 rounded-xl w-[560px] max-w-[90vw] shadow-2xl border border-base-content/10">
        <div class="flex items-center justify-between px-5 py-4 border-b border-base-content/10">
          <h3 class="m-0 text-sm font-semibold text-base-content">创建消费组 (XGROUP CREATE)</h3>
          <button @click="showGroupModal = false" class="border-none bg-transparent cursor-pointer text-lg text-base-content/60 p-1 rounded hover:bg-base-200 transition-colors"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg></button>
        </div>
        <div class="p-5">
          <div class="flex gap-4">
            <div class="flex-1 mb-4">
              <label class="block text-xs font-semibold text-base-content/60 mb-1.5">消费组名称</label>
              <input v-model="newGroupName" class="input input-sm w-full bg-base-200" placeholder="group name" />
            </div>
            <div class="flex-1 mb-4">
              <label class="block text-xs font-semibold text-base-content/60 mb-1.5">起始 ID (默认 0 从头开始)</label>
              <input v-model="newGroupStartId" class="input input-sm w-full bg-base-200" placeholder="0" />
            </div>
          </div>
        </div>
        <div class="flex justify-end gap-2 px-5 py-4 border-t border-base-content/10">
          <button @click="showGroupModal = false" class="btn btn-ghost">取消</button>
          <button @click="createGroup" class="btn btn-primary" :disabled="!newGroupName">创建</button>
        </div>
      </div>
    </div>

    <!-- Trim Modal -->
    <div v-if="showTrimModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showTrimModal = false">
      <div class="bg-base-100 rounded-xl w-[560px] max-w-[90vw] shadow-2xl border border-base-content/10">
        <div class="flex items-center justify-between px-5 py-4 border-b border-base-content/10">
          <h3 class="m-0 text-sm font-semibold text-base-content"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="6" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><line x1="20" y1="4" x2="8.12" y2="15.88"/><line x1="14.47" y1="14.48" x2="20" y2="20"/><line x1="8.12" y1="8.12" x2="12" y2="12"/></svg> 清理队列 (XTRIM)</h3>
          <button @click="showTrimModal = false" class="border-none bg-transparent cursor-pointer text-lg text-base-content/60 p-1 rounded hover:bg-base-200 transition-colors"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg></button>
        </div>
        <div class="p-5">
          <div class="p-3 bg-warning/10 border border-warning/30 rounded-lg text-xs text-warning mb-4"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg> 此操作将保留最近 N 条消息，其余全部删除。此操作不可恢复！</div>
          <div class="mb-4">
            <label class="block text-xs font-semibold text-base-content/60 mb-1.5">当前消息数: {{ streamInfo?.length || 0 }}</label>
          </div>
          <div class="mb-4">
            <label class="block text-xs font-semibold text-base-content/60 mb-1.5">保留最近 N 条</label>
            <input v-model.number="trimKeepN" type="number" class="input input-sm w-full bg-base-200" placeholder="100" min="10" />
          </div>
        </div>
        <div class="flex justify-end gap-2 px-5 py-4 border-t border-base-content/10">
          <button @click="showTrimModal = false" class="btn btn-ghost">取消</button>
          <button @click="trimQueue" class="btn btn-primary" :disabled="!trimKeepN || trimKeepN < 10">确认清理</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { useRedisQueue, type DBConnection } from '@/composables/useRedisQueue'

const props = defineProps<{
  connectionId: string
  connectionName: string
  connection?: DBConnection
  redisDbIndex?: number
}>()

const rq = useRedisQueue(props)

const {
  connectionStatus, loading, msgLoading, groupLoading, consumersLoading,
  pendingLoading, pushing, streamPattern, streams, hasMoreStreams,
  loadingMore, selectedStream, streamInfo, groups, messages, msgStart,
  msgEnd, msgConsumptionStatus, pendingMsgIds, messageSearchQuery,
  selectedGroup, groupDetailTab, consumers, pendingMessages,
  claimConsumerName, detailTab, showAddModal, addKey, addFieldsText,
  addMaxlen, addAsMqMessage, showGroupModal, newGroupName,
  newGroupStartId, showTrimModal, trimKeepN, delayQueues,
  selectedDelayQueue, delayMessages, delayLoading, delaySectionCollapsed,
  autoRefreshEnabled, autoRefreshInterval, statsLoading, groupStats,
  filteredStreams, totalPending, filteredMessages, totalConsumers,
  consumerStats, healthPercentages,
  checkConnection, withReconnect, refreshStreams, loadMoreStreams,
  selectStream, loadStreamInfo, loadMessages, addMessage,
  openAddMessage, deleteMessage, deleteStream, loadGroups, createGroup,
  destroyGroup, selectGroup, loadConsumers, loadPending, claimPending,
  ackPending, retryPending, loadStats, trimQueue, refreshDelayQueues,
  selectDelayQueue, refreshDelayQueue, fireDelayMessage, deleteDelayMessage,
  toggleAutoRefresh, startAutoRefresh, stopAutoRefresh, restartAutoRefresh,
  isJSON, formatValue, formatStreamId, compareStreamIds, loadAllPendingIds,
  enrichMessageConsumptionStatus, getConsumptionBadgeClass, getConsumptionLabel,
  getConsumptionIcon, shortId, formatTime, formatTimestamp, formatDuration,
  formatJsonPreview, formatJsonDisplay, generateUUID, getHealthClass,
  extractPodId, isRetrier, getConsumerHealthClass, getConsumerHealthDot,
  copyText, refreshAll,
} = rq
</script>
