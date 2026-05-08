<template>
  <div class="redis-stream-manager">
    <!-- Top Bar -->
    <div class="stream-topbar">
      <div class="topbar-title">
        <span class="topbar-icon">🌊</span>
        <span>Stream 管理</span>
        <span v-if="streams.length > 0" class="topbar-badge">{{ streams.length }}</span>
      </div>
      <div class="topbar-actions">
        <span class="connection-status" :class="connectionStatus">
          <span class="status-dot">{{ connectionStatus === 'connected' ? '🟢' : connectionStatus === 'connecting' ? '🟡' : '🔴' }}</span>
          <span class="status-text">{{ connectionStatus === 'connected' ? '已连接' : connectionStatus === 'connecting' ? '连接中' : '未连接' }}</span>
        </span>
        <!-- Auto Refresh -->
        <div class="auto-refresh-control">
          <button @click="toggleAutoRefresh" class="btn-icon" :class="{ active: autoRefreshEnabled }" :title="autoRefreshEnabled ? '停止自动刷新' : '开启自动刷新'">
            🔄
          </button>
          <select v-if="autoRefreshEnabled" v-model="autoRefreshInterval" class="refresh-interval-select" @change="restartAutoRefresh">
            <option value="3000">3s</option>
            <option value="5000">5s</option>
            <option value="10000">10s</option>
          </select>
        </div>
        <button @click="refreshAll" class="btn-icon" :disabled="loading" title="刷新">
          <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" :class="{ spinning: loading }">
            <polyline points="23 4 23 10 17 10" />
            <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
          </svg>
        </button>
      </div>
    </div>

    <div class="stream-split">
      <!-- Left: Stream List -->
      <div class="stream-list-panel">
        <div class="panel-header">
          <span class="panel-title">Streams</span>
        </div>
        <div class="panel-search">
          <input
            v-model="streamPattern"
            @keydown.enter="refreshStreams"
            class="search-input"
            placeholder="搜索 stream，如 * 或 my-stream-*"
          />
        </div>
        <div class="stream-list">
          <div v-if="loading && !selectedStream" class="loading-state">加载中...</div>
          <div v-else-if="streams.length === 0" class="empty-state">
            <div class="empty-icon">🌊</div>
            <div class="empty-text">未找到 Stream</div>
            <div class="empty-hint">使用上方搜索框查找 stream 类型的 key</div>
          </div>
          <div
            v-for="s in filteredStreams"
            :key="s.name"
            class="stream-item"
            :class="{ active: selectedStream === s.name }"
            @click="selectStream(s.name)"
          >
            <div class="stream-item-main">
              <span class="stream-icon">🌊</span>
              <span class="stream-name" :title="s.name">{{ s.name }}</span>
            </div>
            <div class="stream-badges">
              <span class="stream-badge" :class="{ danger: s.pendingCount > 10, warn: s.pendingCount > 0 && s.pendingCount <= 10 }" :title="`${s.pendingCount} 条 pending`">{{ s.pendingCount }}</span>
              <span class="stream-badge" :title="`${s.length} 条消息`">{{ s.length }}</span>
              <span v-if="s.groups > 0" class="stream-badge group-badge" :title="`${s.groups} 个消费组`">{{ s.groups }}g</span>
            </div>
          </div>

          <!-- Load More Button -->
          <div v-if="hasMoreStreams" class="load-more-container">
            <button @click="loadMoreStreams" :disabled="loadingMore" class="btn-load-more">
              {{ loadingMore ? '加载中...' : '加载更多 Streams' }}
            </button>
          </div>

          <!-- Delay Queues Section -->
          <div v-if="delayQueues.length > 0" class="delay-section">
            <div class="delay-section-header" @click="delaySectionCollapsed = !delaySectionCollapsed">
              <span class="delay-arrow">{{ delaySectionCollapsed ? '▶' : '▼' }}</span>
              <span class="delay-title">⏰ 延迟队列</span>
              <span class="delay-badge">{{ delayQueues.length }}</span>
            </div>
            <div v-show="!delaySectionCollapsed" class="delay-list">
              <div
                v-for="dq in delayQueues"
                :key="dq.name"
                class="delay-item"
                :class="{ active: selectedDelayQueue === dq.name }"
                @click="selectDelayQueue(dq.name)"
              >
                <span class="delay-icon">⏰</span>
                <span class="delay-name" :title="dq.name">{{ dq.name.replace('delay:', '') }}</span>
                <span class="delay-count">{{ dq.count }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Right: Stream Detail -->
      <div class="stream-detail-panel">
        <template v-if="!selectedStream && !selectedDelayQueue">
          <div class="placeholder-state">
            <div class="placeholder-icon">👈</div>
            <div class="placeholder-text">从左侧选择一个 Stream 或延迟队列</div>
          </div>
        </template>

        <!-- ===== Delay Queue View ===== -->
        <template v-if="selectedDelayQueue">
          <div class="stream-header">
            <div class="stream-info">
              <h3 class="stream-name-large" :title="selectedDelayQueue">⏰ {{ selectedDelayQueue }}</h3>
              <div class="stream-meta">
                <span>{{ delayMessages.length }} 条待到期消息</span>
              </div>
            </div>
            <div class="stream-actions">
              <button @click="refreshDelayQueue" class="btn btn-ghost btn-sm">🔄 刷新</button>
              <button @click="selectedDelayQueue = ''" class="btn btn-ghost btn-sm">✕ 关闭</button>
            </div>
          </div>
          <div class="messages-list delay-messages-list">
            <div v-if="delayLoading" class="loading-state">加载中...</div>
            <div v-else-if="delayMessages.length === 0" class="empty-messages">
              <div class="empty-icon">⏰</div>
              <div class="empty-text">暂无延迟消息</div>
            </div>
            <div v-for="dm in delayMessages" :key="dm.value + dm.score" class="message-item delay-message-item">
              <div class="message-header">
                <span class="message-id">Score: {{ formatTimestamp(dm.score) }}</span>
                <span class="delay-countdown" :class="{ expired: dm.remainingMs <= 0 }">
                  {{ dm.remainingMs <= 0 ? '已到期' : `剩余 ${formatDuration(dm.remainingMs)}` }}
                </span>
              </div>
              <div class="message-fields">
                <pre class="field-value" :class="{ 'is-json': true }">{{ formatJsonPreview(dm.messageJson) }}</pre>
              </div>
              <div class="delay-actions">
                <button v-if="dm.remainingMs <= 0" @click="fireDelayMessage(dm)" class="btn btn-ghost btn-xs">🔥 立即投递</button>
                <button @click="deleteDelayMessage(dm)" class="btn btn-ghost btn-xs btn-danger-text">🗑️ 删除</button>
              </div>
            </div>
          </div>
        </template>

        <!-- ===== Stream Detail View ===== -->
        <template v-if="selectedStream && !selectedDelayQueue">
          <!-- Stream Info Header -->
          <div class="stream-header">
            <div class="stream-info">
              <h3 class="stream-name-large" :title="selectedStream">{{ selectedStream }}</h3>
              <div class="stream-meta">
                <span v-if="streamInfo">{{ streamInfo.length }} 条消息</span>
                <span v-if="streamInfo" class="meta-sep">·</span>
                <span v-if="streamInfo">{{ streamInfo.groups }} 个消费组</span>
                <span v-if="totalPending > 0" class="meta-sep">·</span>
                <span v-if="totalPending > 0" class="pending-warn">{{ totalPending }} pending</span>
              </div>
            </div>
            <div class="stream-actions">
              <button @click="openAddMessage" class="btn btn-primary btn-sm">📤 投递消息</button>
              <button @click="showGroupModal = true" class="btn btn-ghost btn-sm">👥 创建消费组</button>
              <button @click="showTrimModal = true" class="btn btn-ghost btn-sm">✂️ 清理</button>
              <button @click="deleteStream" class="btn btn-danger btn-sm">🗑️ 删除</button>
            </div>
          </div>

          <!-- Tabs -->
          <div class="detail-tabs">
            <button :class="['detail-tab', { active: detailTab === 'messages' }]" @click="detailTab = 'messages'">
              消息列表 <span v-if="messages.length" class="tab-badge">{{ messages.length }}</span>
            </button>
            <button :class="['detail-tab', { active: detailTab === 'stats' }]" @click="detailTab = 'stats'; loadStats()">
              📊 统计
            </button>
            <button :class="['detail-tab', { active: detailTab === 'groups' }]" @click="detailTab = 'groups'">
              消费组 <span v-if="groups.length" class="tab-badge">{{ groups.length }}</span>
            </button>
          </div>

          <!-- Messages Tab -->
          <template v-if="detailTab === 'messages'">
            <div class="messages-toolbar">
              <div class="range-inputs">
                <input v-model="msgStart" class="range-input" placeholder="起始 ID (默认 -)" />
                <span class="range-sep">→</span>
                <input v-model="msgEnd" class="range-input" placeholder="结束 ID (默认 +)" />
              </div>
              <!-- Search -->
              <div class="message-search">
                <input v-model="messageSearchQuery" class="search-input-sm" placeholder="🔍 搜索消息内容..." />
              </div>
              <button @click="loadMessages" class="btn btn-ghost btn-xs">加载</button>
            </div>
            <div class="messages-list">
              <div v-if="msgLoading" class="loading-state">加载中...</div>
              <div v-else-if="filteredMessages.length === 0" class="empty-messages">
                <div class="empty-icon">📭</div>
                <div class="empty-text">{{ messageSearchQuery ? '未找到匹配的消息' : '暂无消息' }}</div>
                <div v-if="messageSearchQuery" class="empty-hint">尝试修改搜索关键词</div>
              </div>
              <div v-for="msg in filteredMessages" :key="msg.id" class="message-item" :class="[
                { 'is-envelope': msg.envelope },
                'consumption-' + (msgConsumptionStatus.get(msg.id)?.status || 'new')
              ]">
                <!-- Envelope-style message card -->
                <template v-if="msg.envelope">
                  <div class="envelope-card" :class="'consumption-' + (msgConsumptionStatus.get(msg.id)?.status || 'new')">
                    <div class="envelope-header">
                      <div class="envelope-type">
                        <span class="envelope-icon">{{ getConsumptionIcon(msgConsumptionStatus.get(msg.id)?.status || 'new') }}</span>
                        <span class="type-label">{{ msg.envelope.messageType || 'UNKNOWN' }}</span>
                        <span v-if="msgConsumptionStatus.size > 0" class="consumption-badge" :class="getConsumptionBadgeClass(msgConsumptionStatus.get(msg.id)?.status || 'new')">
                          {{ getConsumptionLabel(msgConsumptionStatus.get(msg.id)?.status || 'new') }}
                        </span>
                      </div>
                      <div class="envelope-actions">
                        <button @click="copyText(msg.envelope.messageId || '')" class="btn-icon-sm" title="复制 Message ID">📋</button>
                        <button @click="deleteMessage(msg.id)" class="btn-icon-sm" title="删除消息">🗑️</button>
                      </div>
                    </div>
                    <div class="envelope-meta">
                      <span class="meta-item" v-if="msg.envelope.messageId">ID: <code>{{ shortId(msg.envelope.messageId) }}</code></span>
                      <span class="meta-sep">|</span>
                      <span class="meta-item" v-if="msg.envelope.createdAt">🕐 {{ formatTime(msg.envelope.createdAt) }}</span>
                      <span class="meta-sep" v-if="msg.envelope.traceId">|</span>
                      <span class="meta-item" v-if="msg.envelope.traceId">Trace: <code>{{ shortId(msg.envelope.traceId) }}</code></span>
                      <span class="meta-sep" v-if="msg.envelope.tenantId">|</span>
                      <span class="meta-item" v-if="msg.envelope.tenantId">Tenant: {{ msg.envelope.tenantId }}</span>
                    </div>
                    <div class="envelope-payload">
                      <div class="payload-label">Payload:</div>
                      <pre class="payload-json">{{ formatJsonDisplay(msg.envelope.payload) }}</pre>
                    </div>
                    <!-- Raw data toggle -->
                    <div class="envelope-raw-toggle">
                      <button @click="msg.showRaw = !msg.showRaw" class="btn btn-ghost btn-xs">
                        {{ msg.showRaw ? '收起原始数据' : '查看原始数据' }}
                      </button>
                    </div>
                    <div v-if="msg.showRaw" class="envelope-raw">
                      <pre class="raw-json">{{ formatJsonDisplay(msg.rawJson) }}</pre>
                    </div>
                  </div>
                </template>
                <!-- Legacy flat fields display -->
                <template v-else>
                  <div class="message-header">
                    <div class="message-id-group">
                      <span class="message-id" :title="msg.id">{{ formatStreamId(msg.id) }}</span>
                      <span v-if="msgConsumptionStatus.size > 0" class="consumption-badge" :class="getConsumptionBadgeClass(msgConsumptionStatus.get(msg.id)?.status || 'new')">
                        {{ getConsumptionIcon(msgConsumptionStatus.get(msg.id)?.status || 'new') }} {{ getConsumptionLabel(msgConsumptionStatus.get(msg.id)?.status || 'new') }}
                      </span>
                    </div>
                    <div class="message-actions">
                      <button @click="copyText(msg.id)" class="btn-icon-sm" title="复制 ID">📋</button>
                      <button @click="deleteMessage(msg.id)" class="btn-icon-sm" title="删除消息">🗑️</button>
                    </div>
                  </div>
                  <div class="message-fields">
                    <div v-for="(value, fieldKey) in msg.fields" :key="fieldKey" class="field-row">
                      <span class="field-key">{{ fieldKey }}</span>
                      <span class="field-sep">=</span>
                      <pre class="field-value" :class="{ 'is-json': isJSON(value) }">{{ formatValue(value) }}</pre>
                    </div>
                  </div>
                </template>
              </div>
            </div>
          </template>

          <!-- Stats Tab -->
          <template v-if="detailTab === 'stats'">
            <div class="stats-panel">
              <div v-if="statsLoading" class="loading-state">加载中...</div>
              <template v-else>
                <!-- Section Title -->
                <div class="stats-section-title">📊 Stream 概览</div>
                <div class="stats-overview">
                  <div class="stat-card">
                    <div class="stat-label">消息总数</div>
                    <div class="stat-value">{{ streamInfo?.length || 0 }}</div>
                  </div>
                  <div class="stat-card">
                    <div class="stat-label">消费组数</div>
                    <div class="stat-value">{{ groups.length }}</div>
                  </div>
                  <div class="stat-card">
                    <div class="stat-label">总 Pending</div>
                    <div class="stat-value" :class="{ 'stat-warn': totalPending > 0, 'stat-danger': totalPending > 10 }">{{ totalPending }}</div>
                  </div>
                  <div class="stat-card">
                    <div class="stat-label">消费者总数</div>
                    <div class="stat-value">{{ totalConsumers }}</div>
                  </div>
                </div>

                <!-- Health Distribution -->
                <div class="stats-section-title">💚 消费者健康分布</div>
                <div class="health-distribution-bar">
                  <div class="health-bar-segment healthy" :style="{ width: healthPercentages.healthy + '%' }">
                    <span v-if="healthPercentages.healthy > 15">{{ consumerStats.healthy }} 活跃</span>
                  </div>
                  <div class="health-bar-segment idle" :style="{ width: healthPercentages.idle + '%' }">
                    <span v-if="healthPercentages.idle > 15">{{ consumerStats.idle }} 空闲</span>
                  </div>
                  <div class="health-bar-segment stale" :style="{ width: healthPercentages.stale + '%' }">
                    <span v-if="healthPercentages.stale > 15">{{ consumerStats.stale }} 失联</span>
                  </div>
                </div>
                <div class="health-legend">
                  <span class="legend-item"><span class="legend-dot dot-healthy"></span> 活跃 (idle &lt; 1h)</span>
                  <span class="legend-item"><span class="legend-dot dot-idle"></span> 空闲 (1h ~ 24h)</span>
                  <span class="legend-item"><span class="legend-dot dot-stale"></span> 失联 (idle &gt; 24h)</span>
                </div>

                <!-- Per-group breakdown -->
                <div v-for="g in groupStats" :key="g.name" class="stat-group-card">
                  <div class="stat-group-header">
                    <div class="stat-group-title">
                      <span class="stat-group-name">👥 {{ g.name }}</span>
                      <div class="stat-group-consumer-summary">
                        <span class="consumer-summary-badge healthy">{{ g.healthyConsumers }} 活跃</span>
                        <span class="consumer-summary-badge idle">{{ g.idleConsumers }} 空闲</span>
                        <span class="consumer-summary-badge stale">{{ g.staleConsumers }} 失联</span>
                      </div>
                    </div>
                    <span class="stat-group-pending" :class="{ 'stat-warn': g.pendingCount > 0, 'stat-danger': g.pendingCount > 10 }">
                      pending: {{ g.pendingCount }}
                    </span>
                  </div>
                  <!-- Consumer grid -->
                  <div class="consumer-grid">
                    <div v-for="c in g.consumers" :key="c.name" class="consumer-card" :class="getConsumerHealthClass(c.pending || 0, c.idle || 0)">
                      <div class="consumer-card-top">
                        <span class="consumer-short-name" :title="c.name">{{ extractPodId(c.name) }}</span>
                        <span class="consumer-type-badge" v-if="isRetrier(c.name)">retrier</span>
                      </div>
                      <div class="consumer-card-meta">
                        <span class="consumer-meta-item">⏱ {{ formatDuration(c.idle || 0) }}</span>
                        <span class="consumer-meta-item" v-if="c.pending !== undefined">📬 {{ c.pending }}</span>
                      </div>
                      <div class="consumer-health-indicator" :class="getConsumerHealthDot(c.pending || 0, c.idle || 0)"></div>
                    </div>
                  </div>
                </div>
              </template>
            </div>
          </template>

          <!-- Groups Tab -->
          <template v-if="detailTab === 'groups'">
            <div class="groups-list">
              <div v-if="groupLoading" class="loading-state">加载中...</div>
              <div v-else-if="groups.length === 0" class="empty-messages">
                <div class="empty-icon">👥</div>
                <div class="empty-text">暂无消费组</div>
                <div class="empty-hint">点击 "创建消费组" 添加</div>
              </div>
              <div v-for="g in groups" :key="g.name" class="group-card">
                <div class="group-header">
                  <div class="group-info">
                    <span class="group-name">👥 {{ g.name }}</span>
                    <span class="group-meta">
                      <span v-if="g.pending !== undefined">pending: {{ g.pending }}</span>
                      <span v-if="g.consumers !== undefined">consumers: {{ g.consumers }}</span>
                      <span v-if="g['lastDeliveredId']">last-id: {{ g['lastDeliveredId'] }}</span>
                    </span>
                  </div>
                  <div class="group-actions">
                    <button @click="selectGroup(g.name)" class="btn btn-ghost btn-xs">详情</button>
                    <button @click="destroyGroup(g.name)" class="btn-icon-sm" title="删除消费组">🗑️</button>
                  </div>
                </div>
              </div>
            </div>
          </template>
        </template>
      </div>
    </div>

    <!-- Group Detail Drawer -->
    <div v-if="selectedGroup" class="group-detail-panel">
      <div class="group-detail-header">
        <div class="group-detail-title">
          <span>👥</span>
          <span>{{ selectedGroup }}</span>
          <span class="group-detail-key">@ {{ selectedStream }}</span>
        </div>
        <div class="group-detail-header-actions">
          <div class="claim-consumer-input">
            <label>Claim 消费者:</label>
            <input v-model="claimConsumerName" class="claim-input" placeholder="admin" title="Claim 目标消费者名称" />
          </div>
          <button @click="selectedGroup = ''" class="modal-close">✕</button>
        </div>
      </div>
      <div class="group-detail-tabs">
        <button :class="['group-tab', { active: groupDetailTab === 'consumers' }]" @click="groupDetailTab = 'consumers'">消费者</button>
        <button :class="['group-tab', { active: groupDetailTab === 'pending' }]" @click="groupDetailTab = 'pending'">Pending 消息</button>
      </div>
      <!-- Consumers -->
      <template v-if="groupDetailTab === 'consumers'">
        <div class="group-detail-content">
          <div v-if="consumersLoading" class="loading-state">加载中...</div>
          <div v-else-if="consumers.length === 0" class="empty-messages"><div class="empty-text">暂无消费者</div></div>
          <div v-for="c in consumers" :key="c.name" class="consumer-row">
            <span class="consumer-name">🟢 {{ c.name }}</span>
            <span class="consumer-meta">pending: {{ c.pending }} | idle: {{ c.idle }}</span>
          </div>
        </div>
      </template>
      <!-- Pending -->
      <template v-if="groupDetailTab === 'pending'">
        <div class="group-detail-content">
          <div v-if="pendingLoading" class="loading-state">加载中...</div>
          <div v-else-if="pendingMessages.length === 0" class="empty-messages"><div class="empty-text">暂无 pending 消息</div></div>
          <div v-for="p in pendingMessages" :key="p.id" class="pending-row" :class="{ 'pending-stale': p.idleTime > 300000 }">
            <div class="pending-info">
              <span class="pending-id" :title="p.id">{{ formatStreamId(p.id) }}</span>
              <span class="pending-meta">
                consumer: {{ p.consumer }} | delivery: {{ p.timesDelivered }} | idle: {{ formatDuration(p.idleTime) }}
              </span>
            </div>
            <div class="pending-actions">
              <button @click="retryPending(p.id)" class="btn btn-ghost btn-xs btn-retry" title="重试（重新投递）">🔁 重试</button>
              <button @click="claimPending(p.id)" class="btn btn-ghost btn-xs" title="Claim">📌 Claim</button>
              <button @click="ackPending(p.id)" class="btn btn-ghost btn-xs" title="Ack">✅ Ack</button>
            </div>
          </div>
        </div>
      </template>
    </div>

    <!-- Add Message Modal -->
    <div v-if="showAddModal" class="modal-overlay" @click.self="showAddModal = false">
      <div class="modal">
        <div class="modal-header">
          <h3>📤 投递消息 (XADD)</h3>
          <button @click="showAddModal = false" class="modal-close">✕</button>
        </div>
        <div class="modal-body">
          <div class="form-row">
            <div class="form-group">
              <label>Stream Key</label>
              <input v-model="addKey" class="form-input" placeholder="stream key" />
            </div>
            <div class="form-group">
              <label>MAXLEN (可选, 0 表示不限制)</label>
              <input v-model.number="addMaxlen" type="number" class="form-input" placeholder="0" min="0" />
            </div>
          </div>
          <div class="form-group">
            <label>字段值 (JSON 格式)</label>
            <textarea v-model="addFieldsText" class="form-textarea" rows="6" placeholder='{"field1": "value1", "field2": "value2"}'></textarea>
          </div>
          <div class="form-group">
            <label>
              <input type="checkbox" v-model="addAsMqMessage" /> 包装为 MqMessage 信封格式
            </label>
          </div>
        </div>
        <div class="modal-footer">
          <button @click="showAddModal = false" class="btn btn-ghost">取消</button>
          <button @click="addMessage" class="btn btn-primary" :disabled="!addKey || !addFieldsText">添加</button>
        </div>
      </div>
    </div>

    <!-- Create Group Modal -->
    <div v-if="showGroupModal" class="modal-overlay" @click.self="showGroupModal = false">
      <div class="modal">
        <div class="modal-header">
          <h3>创建消费组 (XGROUP CREATE)</h3>
          <button @click="showGroupModal = false" class="modal-close">✕</button>
        </div>
        <div class="modal-body">
          <div class="form-row">
            <div class="form-group">
              <label>消费组名称</label>
              <input v-model="newGroupName" class="form-input" placeholder="group name" />
            </div>
            <div class="form-group">
              <label>起始 ID (默认 0 从头开始)</label>
              <input v-model="newGroupStartId" class="form-input" placeholder="0" />
            </div>
          </div>
        </div>
        <div class="modal-footer">
          <button @click="showGroupModal = false" class="btn btn-ghost">取消</button>
          <button @click="createGroup" class="btn btn-primary" :disabled="!newGroupName">创建</button>
        </div>
      </div>
    </div>

    <!-- Trim Modal -->
    <div v-if="showTrimModal" class="modal-overlay" @click.self="showTrimModal = false">
      <div class="modal">
        <div class="modal-header">
          <h3>✂️ 清理队列 (XTRIM)</h3>
          <button @click="showTrimModal = false" class="modal-close">✕</button>
        </div>
        <div class="modal-body">
          <p class="trim-warning">⚠️ 此操作将保留最近 N 条消息，其余全部删除。此操作不可恢复！</p>
          <div class="form-group">
            <label>当前消息数: {{ streamInfo?.length || 0 }}</label>
          </div>
          <div class="form-group">
            <label>保留最近 N 条</label>
            <input v-model.number="trimKeepN" type="number" class="form-input" placeholder="100" min="10" />
          </div>
        </div>
        <div class="modal-footer">
          <button @click="showTrimModal = false" class="btn btn-ghost">取消</button>
          <button @click="trimQueue" class="btn btn-primary" :disabled="!trimKeepN || trimKeepN < 10">确认清理</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
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

<style scoped>
.redis-stream-manager {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: oklch(var(--b2));
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

/* ==================== Top Bar ==================== */
.stream-topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  height: 48px;
  background: oklch(var(--b1));
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
}

.topbar-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  font-size: 14px;
  color: oklch(var(--bc));
}

.topbar-icon { font-size: 18px; }

.topbar-badge {
  font-size: 11px;
  padding: 1px 8px;
  border-radius: 10px;
  background: oklch(var(--p));
  color: white;
  font-weight: 500;
}

.topbar-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.auto-refresh-control {
  display: flex;
  align-items: center;
  gap: 4px;
}

.btn-icon.active {
  background: oklch(var(--p) / 0.1);
  color: oklch(var(--p));
}

.refresh-interval-select {
  font-size: 11px;
  padding: 2px 4px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 4px;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
}

.connection-status {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  padding: 4px 10px;
  border-radius: 12px;
  background: oklch(var(--b2));
  margin-right: 4px;
}

.status-dot { font-size: 10px; line-height: 1; }
.status-text { color: oklch(var(--bc) / 0.6); white-space: nowrap; }

.connection-status.connected { background: rgba(16, 185, 129, 0.1); }
.connection-status.connected .status-text { color: #059669; }
.connection-status.disconnected { background: rgba(239, 68, 68, 0.1); }
.connection-status.disconnected .status-text { color: #dc2626; }
.connection-status.connecting { background: rgba(245, 158, 11, 0.1); }
.connection-status.connecting .status-text { color: #d97706; }

.btn-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  border-radius: 6px;
  cursor: pointer;
  color: oklch(var(--bc) / 0.6);
  transition: all 0.15s;
}

.btn-icon:hover:not(:disabled) { background: oklch(var(--b2)); color: oklch(var(--bc)); }
.btn-icon:disabled { opacity: 0.5; cursor: not-allowed; }

.spinning { animation: spin 1s linear infinite; }

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* ==================== Split Layout ==================== */
.stream-split {
  display: flex;
  flex: 1;
  overflow: hidden;
}

/* ==================== Left Panel ==================== */
.stream-list-panel {
  width: 280px;
  border-right: 1px solid oklch(var(--bc) / 0.1);
  display: flex;
  flex-direction: column;
  background: oklch(var(--b1));
}

.panel-header {
  padding: 12px 16px;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
}

.panel-title { font-weight: 600; font-size: 13px; color: oklch(var(--bc)); }

.panel-search {
  padding: 12px 16px;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
}

.search-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 6px;
  font-size: 13px;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  transition: border-color 0.15s;
  box-sizing: border-box;
}

.search-input:focus { outline: none; border-color: oklch(var(--p)); background: oklch(var(--b1)); }

.stream-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.stream-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border-radius: 8px;
  cursor: pointer;
  margin-bottom: 4px;
  transition: all 0.15s;
}

.stream-item:hover { background: oklch(var(--b2)); }

.stream-item.active {
  background: oklch(var(--p) / 0.1);
  box-shadow: 0 0 0 1px oklch(var(--p));
}

.stream-item-main { display: flex; align-items: center; gap: 8px; min-width: 0; }
.stream-icon { font-size: 16px; flex-shrink: 0; }
.stream-name { font-size: 13px; color: oklch(var(--bc)); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

.stream-badges { display: flex; gap: 4px; flex-shrink: 0; }

.stream-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 10px;
  background: oklch(var(--b2));
  color: oklch(var(--bc) / 0.6);
}

.stream-badge.danger { background: #fef2f2; color: #dc2626; font-weight: 600; }
.stream-badge.warn { background: #fefce8; color: #ca8a04; }
.group-badge { background: #fef3c7; color: #d97706; font-weight: 500; }

/* Delay Section */
.delay-section { margin-top: 8px; border-top: 1px solid oklch(var(--bc) / 0.1); padding-top: 8px; }

.delay-section-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  cursor: pointer;
  border-radius: 6px;
  transition: background 0.15s;
}

.delay-section-header:hover { background: oklch(var(--b2)); }

.delay-arrow { font-size: 10px; color: oklch(var(--bc) / 0.6); }
.delay-title { font-size: 12px; font-weight: 600; color: oklch(var(--bc)); }
.delay-badge { font-size: 11px; padding: 1px 6px; border-radius: 10px; background: oklch(var(--p)); color: white; }

.delay-list { padding: 4px 8px; }

.delay-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s;
  margin-bottom: 2px;
}

.delay-item:hover { background: oklch(var(--b2)); }
.delay-item.active { background: oklch(var(--p) / 0.1); box-shadow: 0 0 0 1px oklch(var(--p)); }
.delay-icon { font-size: 14px; }
.delay-name { font-size: 12px; color: oklch(var(--bc)); flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.delay-count { font-size: 11px; padding: 1px 6px; border-radius: 10px; background: #f0fdf4; color: #059669; }

/* ==================== Right Panel ==================== */
.stream-detail-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.stream-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
  background: oklch(var(--b1));
  gap: 12px;
}

.stream-info { min-width: 0; }
.stream-name-large { margin: 0; font-size: 15px; font-weight: 600; color: oklch(var(--bc)); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.stream-meta { font-size: 12px; color: oklch(var(--bc) / 0.6); }
.meta-sep { margin: 0 4px; }
.pending-warn { color: #dc2626; font-weight: 600; }

.stream-actions { display: flex; gap: 6px; flex-shrink: 0; }

/* ==================== Detail Tabs ==================== */
.detail-tabs {
  display: flex;
  gap: 0;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
  background: oklch(var(--b1));
  padding: 0 16px;
}

.detail-tab {
  padding: 10px 16px;
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 13px;
  color: oklch(var(--bc) / 0.6);
  transition: all 0.15s;
  position: relative;
}

.detail-tab:hover { color: oklch(var(--bc)); }
.detail-tab.active { color: oklch(var(--p)); font-weight: 500; }
.detail-tab.active::after {
  content: '';
  position: absolute;
  bottom: 0; left: 0; right: 0;
  height: 2px;
  background: oklch(var(--p));
  border-radius: 2px 2px 0 0;
}

.tab-badge {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 10px;
  background: oklch(var(--b2));
  color: oklch(var(--bc) / 0.6);
  margin-left: 4px;
}

/* ==================== Messages Toolbar ==================== */
.messages-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
  background: oklch(var(--b2));
  flex-wrap: wrap;
}

.range-inputs { display: flex; align-items: center; gap: 6px; flex: 1; min-width: 200px; }
.range-input {
  flex: 1;
  padding: 6px 10px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 6px;
  font-size: 12px;
  font-family: monospace;
  background: oklch(var(--b1));
  color: oklch(var(--bc));
}
.range-input:focus { outline: none; border-color: oklch(var(--p)); }
.range-sep { color: oklch(var(--bc) / 0.6); }

.message-search { min-width: 150px; }
.search-input-sm {
  width: 100%;
  padding: 6px 10px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 6px;
  font-size: 12px;
  background: oklch(var(--b1));
  color: oklch(var(--bc));
  box-sizing: border-box;
}
.search-input-sm:focus { outline: none; border-color: oklch(var(--p)); }

/* ==================== Messages List ==================== */
.messages-list {
  flex: 1;
  overflow-y: auto;
  padding: 12px 16px;
  background: oklch(var(--b2));
}

.delay-messages-list { background: #f8fafc; }

.message-item {
  background: oklch(var(--b1));
  border-radius: 10px;
  padding: 12px 16px;
  margin-bottom: 10px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.05);
  border: 1px solid oklch(var(--bc) / 0.1);
  transition: all 0.2s;
}

.message-item:hover { box-shadow: 0 2px 8px rgba(0,0,0,0.08); }

.message-item.is-envelope { padding: 0; overflow: hidden; }

/* ==================== Envelope Card ==================== */
.envelope-card { padding: 12px 16px; border-left: 4px solid transparent; transition: border-color 0.2s; }

/* Consumption status borders */
.envelope-card.consumption-consumed { border-left-color: #10b981; }
.envelope-card.consumption-pending { border-left-color: #f59e0b; }
.envelope-card.consumption-new { border-left-color: #94a3b8; }

/* Legacy message item consumption borders */
.message-item:not(.is-envelope) { border-left: 4px solid transparent; transition: border-color 0.2s; }
.message-item.consumption-consumed { border-left-color: #10b981; }
.message-item.consumption-pending { border-left-color: #f59e0b; }
.message-item.consumption-new { border-left-color: #94a3b8; }

.envelope-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.envelope-type { display: flex; align-items: center; gap: 6px; }
.envelope-icon { font-size: 16px; }
.type-label {
  font-size: 13px;
  font-weight: 600;
  color: oklch(var(--p));
  font-family: monospace;
}

/* Consumption status badge */
.consumption-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 10px;
  font-weight: 500;
  white-space: nowrap;
}
.consumption-badge.badge-consumed { background: #d1fae5; color: #059669; }
.consumption-badge.badge-pending { background: #fef3c7; color: #d97706; }
.consumption-badge.badge-new { background: #f1f5f9; color: #64748b; }

/* Message ID group (id + badge in legacy view) */
.message-id-group { display: flex; align-items: center; gap: 8px; }

.envelope-actions { display: flex; gap: 4px; }

.envelope-meta {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: oklch(var(--bc) / 0.6);
  margin-bottom: 8px;
  flex-wrap: wrap;
}

.meta-item code {
  font-family: monospace;
  background: oklch(var(--b2));
  padding: 1px 4px;
  border-radius: 3px;
  font-size: 10px;
}

.envelope-payload {
  background: oklch(var(--b2));
  border-radius: 8px;
  padding: 10px 12px;
  margin-bottom: 8px;
}

.payload-label {
  font-size: 11px;
  font-weight: 600;
  color: oklch(var(--bc) / 0.6);
  margin-bottom: 4px;
}

.payload-json {
  margin: 0;
  font-size: 12px;
  font-family: 'Menlo', 'Monaco', 'Courier New', monospace;
  color: #059669;
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 200px;
  overflow-y: auto;
  line-height: 1.5;
}

.envelope-raw-toggle { text-align: center; }

.envelope-raw {
  background: oklch(var(--b2));
  border-radius: 8px;
  padding: 10px 12px;
  margin-top: 8px;
}

.raw-json {
  margin: 0;
  font-size: 11px;
  font-family: 'Menlo', 'Monaco', 'Courier New', monospace;
  color: oklch(var(--bc) / 0.6);
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 150px;
  overflow-y: auto;
  line-height: 1.4;
}

/* ==================== Message Header (legacy) ==================== */
.message-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.message-id { font-size: 11px; color: oklch(var(--bc) / 0.6); font-weight: 500; font-family: monospace; }
.message-actions { display: flex; gap: 4px; }

.btn-icon-sm {
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 14px;
  padding: 2px 4px;
  border-radius: 4px;
  opacity: 0.6;
  transition: all 0.15s;
}

.btn-icon-sm:hover { opacity: 1; background: oklch(var(--b2)); }

.message-fields { display: flex; flex-direction: column; gap: 4px; }
.field-row { display: flex; align-items: baseline; gap: 4px; font-size: 12px; line-height: 1.4; }
.field-key { font-weight: 600; color: #7c3aed; font-family: monospace; flex-shrink: 0; }
.field-sep { color: oklch(var(--bc) / 0.6); }

.field-value {
  margin: 0;
  font-size: 12px;
  font-family: 'Menlo', 'Monaco', 'Courier New', monospace;
  color: oklch(var(--bc));
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 120px;
  overflow-y: auto;
  line-height: 1.5;
}

.field-value.is-json { color: #059669; }

/* Delay message item */
.delay-message-item { border-left: 3px solid #f59e0b; }
.delay-message-item .delay-countdown {
  font-size: 11px;
  font-weight: 600;
  color: #059669;
}
.delay-message-item .delay-countdown.expired { color: #dc2626; }

.delay-actions {
  display: flex;
  gap: 6px;
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid oklch(var(--bc) / 0.1);
}

/* ==================== Stats Panel ==================== */
.stats-panel {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  background: oklch(var(--b2));
}

.stats-section-title {
  font-size: 13px;
  font-weight: 600;
  color: oklch(var(--bc));
  margin: 16px 0 10px;
  padding-bottom: 6px;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
}
.stats-section-title:first-child { margin-top: 0; }

.stats-overview {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
  margin-bottom: 20px;
}

.stat-card {
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 10px;
  padding: 16px;
  text-align: center;
}

.stat-label { font-size: 12px; color: oklch(var(--bc) / 0.6); margin-bottom: 4px; }
.stat-value { font-size: 24px; font-weight: 700; color: oklch(var(--bc)); }
.stat-value.stat-warn { color: #f59e0b; }
.stat-value.stat-danger { color: #dc2626; }

/* Health Distribution Bar */
.health-distribution-bar {
  display: flex;
  height: 36px;
  border-radius: 10px;
  overflow: hidden;
  margin-bottom: 8px;
  border: 1px solid oklch(var(--bc) / 0.1);
}

.health-bar-segment {
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 600;
  color: white;
  transition: width 0.3s;
  min-width: 0;
  overflow: hidden;
}
.health-bar-segment.healthy { background: #10b981; }
.health-bar-segment.idle { background: #f59e0b; }
.health-bar-segment.stale { background: #94a3b8; }

.health-legend {
  display: flex;
  gap: 16px;
  margin-bottom: 16px;
  padding: 8px 12px;
  background: oklch(var(--b1));
  border-radius: 8px;
  border: 1px solid oklch(var(--bc) / 0.1);
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: oklch(var(--bc) / 0.6);
}

.legend-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
}
.legend-dot.dot-healthy { background: #10b981; }
.legend-dot.dot-idle { background: #f59e0b; }
.legend-dot.dot-stale { background: #94a3b8; }

/* Per-group card */
.stat-group-card {
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 10px;
  margin-bottom: 12px;
  overflow: hidden;
}

.stat-group-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
  background: oklch(var(--b2));
}

.stat-group-title { display: flex; flex-direction: column; gap: 4px; }
.stat-group-name { font-size: 13px; font-weight: 600; color: oklch(var(--bc)); }

.stat-group-consumer-summary { display: flex; gap: 6px; }
.consumer-summary-badge {
  font-size: 11px;
  padding: 1px 8px;
  border-radius: 10px;
  font-weight: 500;
}
.consumer-summary-badge.healthy { background: #d1fae5; color: #059669; }
.consumer-summary-badge.idle { background: #fef3c7; color: #d97706; }
.consumer-summary-badge.stale { background: #f1f5f9; color: #64748b; }

.stat-group-pending { font-size: 12px; font-weight: 600; }
.stat-group-pending.stat-warn { color: #f59e0b; }
.stat-group-pending.stat-danger { color: #dc2626; }

/* Consumer Grid */
.consumer-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  gap: 8px;
  padding: 12px;
}

.consumer-card {
  background: oklch(var(--b2));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  padding: 10px 12px;
  position: relative;
  transition: all 0.15s;
  border-top: 3px solid transparent;
}
.consumer-card:hover { box-shadow: 0 2px 8px rgba(0,0,0,0.08); transform: translateY(-1px); }

.consumer-card.card-healthy { border-top-color: #10b981; }
.consumer-card.card-idle { border-top-color: #f59e0b; }
.consumer-card.card-stale { border-top-color: #94a3b8; }

.consumer-card-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
  margin-bottom: 6px;
}

.consumer-short-name {
  font-size: 12px;
  font-family: monospace;
  color: oklch(var(--bc));
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.consumer-type-badge {
  font-size: 9px;
  padding: 1px 5px;
  border-radius: 4px;
  background: #ede9fe;
  color: #7c3aed;
  font-weight: 600;
  text-transform: uppercase;
  flex-shrink: 0;
}

.consumer-card-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
  color: oklch(var(--bc) / 0.6);
}

.consumer-meta-item { white-space: nowrap; }

/* Health indicator dot (top-right corner of card) */
.consumer-health-indicator {
  position: absolute;
  top: 6px;
  right: 6px;
  width: 8px;
  height: 8px;
  border-radius: 50%;
}
.consumer-health-indicator.dot-healthy { background: #10b981; }
.consumer-health-indicator.dot-idle { background: #f59e0b; }
.consumer-health-indicator.dot-stale { background: #94a3b8; }
.consumer-health-indicator.dot-danger { background: #dc2626; }

.stat-consumer-list { padding: 8px 16px; }

.stat-consumer-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-radius: 6px;
  margin-bottom: 4px;
  background: oklch(var(--b2));
  font-size: 12px;
}

.stat-consumer-name { font-weight: 500; color: oklch(var(--bc)); }
.stat-consumer-meta { color: oklch(var(--bc) / 0.6); }

.health-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  margin-left: 8px;
}

.dot-healthy { background: #22c55e; }
.dot-idle { background: #f59e0b; }
.dot-stale { background: #94a3b8; }
.dot-warn { background: #f59e0b; }
.dot-danger { background: #dc2626; }

/* ==================== Groups ==================== */
.groups-list {
  flex: 1;
  overflow-y: auto;
  padding: 12px 16px;
  background: oklch(var(--b2));
}

.group-card {
  background: oklch(var(--b1));
  border-radius: 10px;
  padding: 12px 16px;
  margin-bottom: 10px;
  border: 1px solid oklch(var(--bc) / 0.1);
}

.group-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.group-name { font-weight: 600; font-size: 13px; color: oklch(var(--bc)); }
.group-meta { font-size: 11px; color: oklch(var(--bc) / 0.6); display: flex; gap: 8px; margin-top: 4px; }
.group-actions { display: flex; gap: 4px; }
.group-info { min-width: 0; }

/* ==================== Group Detail Panel ==================== */
.group-detail-panel {
  border-top: 2px solid oklch(var(--p));
  background: oklch(var(--b1));
  max-height: 300px;
  display: flex;
  flex-direction: column;
}

.group-detail-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
}

.group-detail-title {
  font-weight: 600;
  font-size: 13px;
  color: oklch(var(--bc));
  display: flex;
  align-items: center;
  gap: 8px;
}

.group-detail-key { font-weight: 400; color: oklch(var(--bc) / 0.6); font-size: 12px; }
.group-detail-header-actions { display: flex; align-items: center; gap: 8px; }

.claim-consumer-input { display: flex; align-items: center; gap: 4px; }
.claim-consumer-input label { font-size: 11px; color: oklch(var(--bc) / 0.6); white-space: nowrap; }
.claim-input {
  width: 80px;
  padding: 3px 6px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 4px;
  font-size: 11px;
  font-family: monospace;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
}
.claim-input:focus { outline: none; border-color: oklch(var(--p)); }

.group-detail-tabs { display: flex; gap: 0; border-bottom: 1px solid oklch(var(--bc) / 0.1); padding: 0 16px; }
.group-tab {
  padding: 8px 16px;
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 12px;
  color: oklch(var(--bc) / 0.6);
  position: relative;
  transition: all 0.15s;
}
.group-tab:hover { color: oklch(var(--bc)); }
.group-tab.active { color: oklch(var(--p)); font-weight: 500; }
.group-tab.active::after {
  content: '';
  position: absolute;
  bottom: 0; left: 0; right: 0;
  height: 2px;
  background: oklch(var(--p));
}

.group-detail-content { flex: 1; overflow-y: auto; padding: 8px 16px; }

.consumer-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-radius: 6px;
  margin-bottom: 4px;
  background: oklch(var(--b2));
  font-size: 12px;
}

.consumer-name { font-weight: 500; color: oklch(var(--bc)); }
.consumer-meta { color: oklch(var(--bc) / 0.6); }

.pending-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-radius: 6px;
  margin-bottom: 4px;
  background: oklch(var(--b2));
  font-size: 12px;
}

.pending-row.pending-stale { background: #fef2f2; border-left: 3px solid #dc2626; }

.pending-id { font-family: monospace; color: oklch(var(--bc)); font-weight: 500; }
.pending-meta { color: oklch(var(--bc) / 0.6); margin-left: 12px; }
.pending-info { display: flex; align-items: center; gap: 4px; min-width: 0; }
.pending-actions { display: flex; gap: 4px; flex-shrink: 0; }

.btn-retry { color: #f59e0b; }

/* ==================== Modal ==================== */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal {
  background: oklch(var(--b1));
  border-radius: 12px;
  width: 560px;
  max-width: 90vw;
  box-shadow: 0 20px 60px rgba(0,0,0,0.3);
  border: 1px solid oklch(var(--bc) / 0.1);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
}

.modal-header h3 { margin: 0; font-size: 15px; font-weight: 600; color: oklch(var(--bc)); }

.modal-close {
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 18px;
  color: oklch(var(--bc) / 0.6);
  padding: 4px;
  border-radius: 4px;
}
.modal-close:hover { background: oklch(var(--b2)); }

.modal-body { padding: 20px; }
.modal-footer { display: flex; justify-content: flex-end; gap: 8px; padding: 16px 20px; border-top: 1px solid oklch(var(--bc) / 0.1); }

.form-group { margin-bottom: 16px; }
.form-group label { display: block; font-size: 12px; font-weight: 600; color: oklch(var(--bc) / 0.6); margin-bottom: 6px; }
.form-group label:has(input[type="checkbox"]) { display: flex; align-items: center; gap: 6px; cursor: pointer; }

.form-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 6px;
  font-size: 13px;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  box-sizing: border-box;
}
.form-input:focus { outline: none; border-color: oklch(var(--p)); }

.form-textarea {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 6px;
  font-size: 13px;
  font-family: monospace;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  box-sizing: border-box;
  resize: vertical;
}
.form-textarea:focus { outline: none; border-color: oklch(var(--p)); }

.trim-warning {
  padding: 12px;
  background: #fefce8;
  border: 1px solid #fde047;
  border-radius: 8px;
  font-size: 13px;
  color: #a16207;
  margin-bottom: 16px;
}

/* ==================== Buttons ==================== */
.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s;
  font-weight: 500;
}

.btn-primary { background: oklch(var(--p)); color: white; }
.btn-primary:hover:not(:disabled) { background: oklch(var(--p) / 0.8); }
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-ghost { background: transparent; color: oklch(var(--bc) / 0.6); }
.btn-ghost:hover:not(:disabled) { background: oklch(var(--b2)); }
.btn-danger { background: #ef4444; color: white; }
.btn-danger:hover:not(:disabled) { background: #dc2626; }
.btn-sm { padding: 6px 12px; font-size: 12px; }
.btn-xs { padding: 4px 8px; font-size: 11px; }
.btn-danger-text { color: #ef4444; }

/* ==================== States ==================== */
.placeholder-state, .empty-state, .empty-messages {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: oklch(var(--bc) / 0.6);
  text-align: center;
  padding: 40px 20px;
}

.placeholder-icon, .empty-icon { font-size: 48px; margin-bottom: 12px; opacity: 0.5; }
.placeholder-text, .empty-text { font-size: 14px; font-weight: 500; margin-bottom: 4px; }
.empty-hint { font-size: 12px; opacity: 0.7; }
.loading-state { padding: 20px; text-align: center; color: oklch(var(--bc) / 0.6); }

.btn-load-more {
  padding: 6px 16px;
  background: transparent;
  border: 1px solid oklch(var(--p));
  color: oklch(var(--p));
  border-radius: 6px;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.15s;
}

.btn-load-more:hover:not(:disabled) {
  background: oklch(var(--p) / 0.1);
}

.btn-load-more:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* ==================== Dark Theme ==================== */
.dark .stream-topbar,
.dark .stream-list-panel { background: #1f2937; border-color: #374151; }
.dark .search-input { background: oklch(var(--b2)); border-color: #374151; color: oklch(var(--bc)); }
.dark .search-input:focus { background: #1f2937; border-color: #818cf8; }
.dark .stream-item:hover { background: #374151; }
.dark .stream-item.active { background: rgba(129, 140, 248, 0.2); box-shadow: 0 0 0 1px #818cf8; }
.dark .stream-header { background: #1f2937; border-color: #374151; }
.dark .detail-tabs,
.dark .group-detail-tabs { background: #1f2937; border-color: #374151; }
.dark .messages-toolbar { background: oklch(var(--b2)); border-color: #374151; }
.dark .messages-list,
.dark .groups-list { background: oklch(var(--b2)); }
.dark .message-item { background: #1f2937; border-color: #374151; }
.dark .field-value.is-json { color: #34d399; }
.dark .group-card { background: #1f2937; border-color: #374151; }
.dark .group-detail-panel { background: #1f2937; border-color: #818cf8; }
.dark .consumer-row,
.dark .pending-row { background: #374151; }
.dark .pending-row.pending-stale { background: #451a1a; }
.dark .modal { background: #1f2937; border-color: #374151; }
.dark .modal-header { border-color: #374151; }
.dark .modal-footer { border-color: #374151; }
.dark .form-input,
.dark .form-textarea { background: oklch(var(--b2)); border-color: #374151; color: oklch(var(--bc)); }
.dark .form-input:focus,
.dark .form-textarea:focus { background: #1f2937; border-color: #818cf8; }
.dark .group-badge { background: #422006; color: #fbbf24; }
.dark .field-key { color: #a78bfa; }
.dark .claim-input { background: #374151; border-color: #4b5563; color: oklch(var(--bc)); }
.dark .claim-input:focus { border-color: #818cf8; }
.dark .connection-status { background: #374151; }
.dark .stream-badge.danger { background: #451a1a; color: #fca5a5; }
.dark .stream-badge.warn { background: #422006; color: #fbbf24; }
.dark .delay-count { background: #14532d; color: #4ade80; }
.dark .delay-message-item { border-left-color: #d97706; }
.dark .delay-messages-list { background: #1a1a2e; }
.dark .payload-json { color: #34d399; }
.dark .envelope-card { }
.dark .envelope-payload,
.dark .envelope-raw { background: #374151; }
.dark .stat-card { background: #1f2937; border-color: #374151; }
.dark .stat-group-card { background: #1f2937; border-color: #374151; }
.dark .stat-consumer-row { background: #374151; }
.dark .trim-warning { background: #422006; border-color: #854d0e; color: #fbbf24; }
.dark .btn-icon.active { background: rgba(129, 140, 248, 0.2); color: #818cf8; }
</style>