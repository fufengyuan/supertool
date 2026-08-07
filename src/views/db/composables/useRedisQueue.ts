// @ts-nocheck
import * as logger from '../../../services/logger'
import { ref, onMounted, onUnmounted, watch, computed } from 'vue'
import { useToast } from '../../../composables/useToast'
import { getErrorMessage } from '../../../utils/helpers'
import { getTauriAPI } from '../../../utils/tauri-api'

export interface DBConnection {
  id: string; name: string; type: string; host: string; port: number;
  user?: string; password?: string; database?: string; dbIndex?: number;
}

// ============ Data Interfaces ============

interface MessageEnvelope {
  messageId: string
  messageType: string
  createdAt: string
  traceId: string
  tenantId: string
  payload: string
}

interface StreamMessage {
  id: string
  fields?: Record<string, string>
  envelope: MessageEnvelope | null
  rawJson: string
  showRaw: boolean
}

interface ZSetEntry {
  value: string
  score: number
}

interface ConsumerInfo {
  name: string
  pending: number
  idle: number
  pendingCount?: number
  lastDeliveredId?: string
}

interface DelayMessage {
  value: string
  score: number
  messageJson: string
  remainingMs: number
}

// ============ IPC Response Interfaces ============

interface RedisExecResponse {
  success: boolean
  result?: unknown
  error?: string
}

interface RedisStreamsResponse {
  success: boolean
  streams?: Array<{ name: string; length: number; groups: number }>
  hasMore?: boolean
  error?: string
}

interface RedisStreamGroupInfo {
  name: string
  pending: number
  consumers: number
  lastDeliveredId?: string
}

interface RedisStreamInfoIPCResponse {
  success: boolean
  info?: Record<string, unknown>
  groups?: RedisStreamGroupInfo[]
  error?: string
}

interface RedisStreamMessagesResponse {
  success: boolean
  messages?: Array<{ id: string; fields?: Record<string, string> }>
  error?: string
}

interface RedisStreamAddResponse {
  success: boolean
  id?: string
  error?: string
}

interface RedisStreamConsumersResponse {
  success: boolean
  consumers?: ConsumerInfo[]
  error?: string
}

interface RedisStreamPendingResponse {
  success: boolean
  pending?: unknown
  error?: string
}

interface RedisStreamRetryResponse {
  success: boolean
  newMessageId?: string
  error?: string
}

interface RedisStreamTrimResponse {
  success: boolean
  trimmed?: number
  error?: string
}

interface RedisScanKeysResponse {
  success: boolean
  keys?: string[]
  error?: string
}

interface RedisZSetRangeResponse {
  success: boolean
  entries?: ZSetEntry[]
  error?: string
}

interface RedisZSetRemoveResponse {
  success: boolean
  removed?: boolean
  error?: string
}

export function useRedisQueue(props: { connectionId: string; connectionName: string; connection?: DBConnection; redisDbIndex?: number }) {
  const toast = useToast()

// ==================== Connection Status ====================

const connectionStatus = ref<'connected' | 'disconnected' | 'connecting'>('disconnected')

async function checkConnection(): Promise<boolean> {
  connectionStatus.value = 'connecting'
  try {
    const result = await getTauriAPI().dbRedisExec(props.connectionId, 'PING')
    if (result?.success) {
      connectionStatus.value = 'connected'
      return true
    }
    connectionStatus.value = 'disconnected'
    return false
  } catch {
    connectionStatus.value = 'disconnected'
    return false
  }
}

async function withReconnect<T>(fn: () => Promise<T>): Promise<T> {
  try {
    return await fn()
  } catch (e: unknown) {
    const msg = getErrorMessage(e)
    if (msg.includes('Connection not found') || msg.includes('Connection is closed')) {
      console.warn('[RedisQueueManager] Connection lost, attempting reconnect...')
      await checkConnection()
      return await fn()
    }
    throw e
  }
}

// ==================== State ====================

const loading = ref(false)
const msgLoading = ref(false)
const groupLoading = ref(false)
const consumersLoading = ref(false)
const pendingLoading = ref(false)
const pushing = ref(false)

const streamPattern = ref('*')
const streams = ref<Array<{ name: string; length: number; groups: number; pendingCount: number }>>([])
const hasMoreStreams = ref(false)
const loadingMore = ref(false)
const selectedStream = ref('')

// Stream info
const streamInfo = ref<Record<string, unknown> | null>(null)
const groups = ref<RedisStreamGroupInfo[]>([])

// Messages
const messages = ref<StreamMessage[]>([])
const msgStart = ref('-')
const msgEnd = ref('+')

// Consumption status tracking
// Map<msgId, { status: 'consumed' | 'pending' | 'new', groups: { name: string, status: string }[] }>
const msgConsumptionStatus = ref<Map<string, { status: 'consumed' | 'pending' | 'new'; groupStatuses: Array<{ name: string; status: string }> }>>(new Map())
const pendingMsgIds = ref<Set<string>>(new Set())

// Message search
const messageSearchQuery = ref('')

// Group detail
const selectedGroup = ref('')
const groupDetailTab = ref<'consumers' | 'pending'>('consumers')
const consumers = ref<ConsumerInfo[]>([])
const pendingMessages = ref<Array<{ id: string; consumer: string; timesDelivered: number; idleTime: number }>>([])

const claimConsumerName = ref('admin')

// Right panel tab
const detailTab = ref<'messages' | 'stats' | 'groups'>('messages')

// Add message modal
const showAddModal = ref(false)
const addKey = ref('')
const addFieldsText = ref('')
const addMaxlen = ref(0)
const addAsMqMessage = ref(false)

// Create group modal
const showGroupModal = ref(false)
const newGroupName = ref('')
const newGroupStartId = ref('0')

// Trim modal
const showTrimModal = ref(false)
const trimKeepN = ref(100)

// Delay queues
const delayQueues = ref<Array<{ name: string; count: number }>>([])
const selectedDelayQueue = ref('')
const delayMessages = ref<DelayMessage[]>([])
const delayLoading = ref(false)
const delaySectionCollapsed = ref(false)

// Auto refresh
const autoRefreshEnabled = ref(false)
const autoRefreshInterval = ref('5000')
let autoRefreshTimer: ReturnType<typeof setInterval> | null = null

// Stats
const statsLoading = ref(false)
const groupStats = ref<Array<{
  name: string
  pendingCount: number
  consumers: ConsumerInfo[]
  healthyConsumers: number
  idleConsumers: number
  staleConsumers: number
}>>([])

// ==================== Computed ====================

const filteredStreams = computed(() => {
  return [...streams.value].sort((a, b) => {
    // DANGER first, then WARN, then HEALTHY
    const healthA = a.pendingCount > 10 ? 0 : a.pendingCount > 0 ? 1 : 2
    const healthB = b.pendingCount > 10 ? 0 : b.pendingCount > 0 ? 1 : 2
    if (healthA !== healthB) {return healthA - healthB}
    return b.pendingCount - a.pendingCount
  })
})

const totalPending = computed(() => {
  return streams.value.reduce((sum, s) => sum + s.pendingCount, 0)
})

const filteredMessages = computed(() => {
  if (!messageSearchQuery.value) {return messages.value}
  const q = messageSearchQuery.value.toLowerCase()
  return messages.value.filter((msg: StreamMessage) => {
    if (msg.envelope) {
      return (
        (msg.envelope.messageType || '').toLowerCase().includes(q) ||
        (msg.envelope.messageId || '').toLowerCase().includes(q) ||
        (msg.envelope.traceId || '').toLowerCase().includes(q) ||
        (msg.envelope.payload || '').toLowerCase().includes(q)
      )
    }
    return JSON.stringify(msg.fields || '').toLowerCase().includes(q)
  })
})

// ==================== Stats Computed ====================

const totalConsumers = computed(() => {
  return groupStats.value.reduce((sum, g) => sum + (g.consumers?.length || 0), 0)
})

// Classify consumers by idle time
function classifyConsumer(pending: number, idle: number): 'healthy' | 'idle' | 'stale' {
  if (idle > 86400000) {return 'stale'}    // > 24h = stale/lost
  if (idle > 3600000) {return 'idle'}       // > 1h = idle
  return 'healthy'                         // < 1h = healthy
}

const consumerStats = computed(() => {
  let healthy = 0, idle = 0, stale = 0
  for (const g of groupStats.value) {
    for (const c of (g.consumers || [])) {
      const cls = classifyConsumer((c.pending as number) || 0, (c.idle as number) || 0)
      if (cls === 'healthy') {healthy++}
      else if (cls === 'idle') {idle++}
      else {stale++}
    }
  }
  return { healthy, idle, stale }
})

const healthPercentages = computed(() => {
  const total = totalConsumers.value
  if (total === 0) {return { healthy: 0, idle: 0, stale: 0 }}
  return {
    healthy: Math.round((consumerStats.value.healthy / total) * 100),
    idle: Math.round((consumerStats.value.idle / total) * 100),
    stale: Math.round((consumerStats.value.stale / total) * 100),
  }
})

// ==================== Stream List ====================

async function refreshStreams() {
  loading.value = true
  streams.value = []
  hasMoreStreams.value = false
  
  logger.info(`[RedisQueueManager] refreshStreams called, connectionId: ${props.connectionId}, dbIndex: ${props.redisDbIndex}`)
  
  try {
    await checkConnection()
    logger.info('[RedisQueueManager] Connection check passed, calling dbRedisStreams...')
    const result = await withReconnect(() =>
      getTauriAPI().dbRedisStreams(props.connectionId, props.redisDbIndex ?? 0, streamPattern.value, false)
    ) as RedisStreamsResponse
    logger.info('[RedisQueueManager] dbRedisStreams result:', JSON.stringify(result))
    if (result.success) {
      const streamList = result.streams || []
      hasMoreStreams.value = result.hasMore || false
      logger.info(`[RedisQueueManager] Got ${streamList.length} streams, hasMore: ${hasMoreStreams.value}`)
      
      // Enrich with pending counts (only for this batch)
      const streamsEnriched = await Promise.all(streamList.map(async (s: { name: string; length: number; groups: number }) => {
        let pendingCount = 0
        try {
          const infoRes = await withReconnect(() =>
            getTauriAPI().dbRedisStreamInfo(props.connectionId, props.redisDbIndex ?? 0, s.name)
          ) as RedisStreamInfoIPCResponse
          if (infoRes.success && infoRes.groups) {
            for (const g of infoRes.groups) {
              pendingCount += g.pending || 0
            }
          }
        } catch {}
        return { ...s, pendingCount }
      }))
      streams.value = streamsEnriched.sort((a, b) => b.length - a.length)
      logger.info(`[RedisQueueManager] Final streams: ${streams.value.length}`)
    } else {
      console.error('[RedisQueueManager] refreshStreams failed:', result.error)
      toast.error('刷新 Streams 失败: ' + (result.error || '未知错误'))
    }
  } catch (e: unknown) {
    console.error('[RedisQueueManager] refreshStreams exception:', e)
    toast.error('刷新 Streams 异常: ' + getErrorMessage(e))
  } finally {
    loading.value = false
  }
}

async function loadMoreStreams() {
  loadingMore.value = true
  try {
    const result = await withReconnect(() =>
      getTauriAPI().dbRedisStreams(props.connectionId, props.redisDbIndex, streamPattern.value, true)
    ) as RedisStreamsResponse
    if (result.success) {
      const streamList = result.streams || []
      hasMoreStreams.value = result.hasMore || false
      
      // Enrich new batch
      const streamsEnriched = await Promise.all(streamList.map(async (s: { name: string; length: number; groups: number }) => {
        let pendingCount = 0
        try {
          const infoRes = await withReconnect(() =>
            getTauriAPI().dbRedisStreamInfo(props.connectionId, props.redisDbIndex ?? 0, s.name)
          ) as RedisStreamInfoIPCResponse
          if (infoRes.success && infoRes.groups) {
            for (const g of infoRes.groups) {
              pendingCount += g.pending || 0
            }
          }
        } catch {}
        return { ...s, pendingCount }
      }))
      
      // Sort only the new batch before appending — prevents scroll jumping
      // that would happen if the entire list was re-sorted
      streamsEnriched.sort((a, b) => b.pendingCount - a.pendingCount)
      streams.value.push(...streamsEnriched)
    }
  } catch (e: unknown) {
    toast.error('加载失败: ' + getErrorMessage(e))
  } finally {
    loadingMore.value = false
  }
}

async function selectStream(name: string) {
  logger.info('[RedisQueueManager] selectStream:', name)
  selectedStream.value = name
  selectedDelayQueue.value = ''
  detailTab.value = 'messages'
  selectedGroup.value = ''
  addKey.value = name
  // loadStreamInfo must finish first to populate groups before enrichment
  await loadStreamInfo()
  await loadMessages()
}

async function loadStreamInfo() {
  logger.info('[RedisQueueManager] loadStreamInfo for:', selectedStream.value)
  try {
    const result = await withReconnect(() =>
      getTauriAPI().dbRedisStreamInfo(props.connectionId, props.redisDbIndex ?? 0, selectedStream.value)
    ) as RedisStreamInfoIPCResponse
    logger.info('[RedisQueueManager] loadStreamInfo result:', JSON.stringify(result))
    if (result.success) {
      streamInfo.value = result.info ?? null
      groups.value = result.groups || []
    } else {
      console.error('[RedisQueueManager] loadStreamInfo failed:', result.error)
    }
  } catch (e: unknown) {
    console.error('[RedisQueueManager] loadStreamInfo exception:', e)
    toast.error('加载 stream 信息失败: ' + getErrorMessage(e))
  }
}

// ==================== Messages ====================

async function loadMessages() {
  if (!selectedStream.value) {return}
  msgLoading.value = true
  logger.info(`[RedisQueueManager] loadMessages for: ${selectedStream.value}, range: ${msgStart.value}-${msgEnd.value}`)
  try {
    const result = await withReconnect(() =>
      getTauriAPI().dbRedisStreamMessages(
        props.connectionId,
        props.redisDbIndex ?? 0,
        selectedStream.value,
        msgStart.value || '-',
        msgEnd.value || '+',
        200
      )
    ) as RedisStreamMessagesResponse
    logger.info('[RedisQueueManager] loadMessages result:', JSON.stringify(result))
    if (result.success) {
      messages.value = (result.messages || []).map((msg: { id: string; fields?: Record<string, string> }) => {
        const dataField = msg.fields?.data
        let envelope: MessageEnvelope | null = null
        if (dataField) {
          try {
            const parsed = JSON.parse(dataField)
            if (parsed.messageId || parsed.payload || parsed.createdAt) {
              envelope = {
                messageId: parsed.messageId,
                messageType: parsed.messageType,
                createdAt: parsed.createdAt,
                traceId: parsed.traceId,
                tenantId: parsed.tenantId,
                payload: parsed.payload,
              }
            }
          } catch {}
        }
        return {
          ...msg,
          envelope,
          rawJson: dataField || JSON.stringify(msg.fields),
          showRaw: false,
        }
      })
      logger.info(`[RedisQueueManager] Loaded ${messages.value.length} messages`)
      // Enrich messages with consumption status
      await enrichMessageConsumptionStatus()
    } else {
      console.error('[RedisQueueManager] loadMessages failed:', result.error)
    }
  } catch (e: unknown) {
    console.error('[RedisQueueManager] loadMessages exception:', e)
    toast.error('加载消息失败: ' + getErrorMessage(e))
  } finally {
    msgLoading.value = false
  }
}

async function addMessage() {
  if (!addKey.value || !addFieldsText.value) {return}
  pushing.value = true
  try {
    let fields: Record<string, string>
    if (addAsMqMessage.value) {
      const mqEnvelope = {
        messageId: generateUUID(),
        messageType: 'MANUAL_TEST',
        createdAt: new Date().toISOString(),
        traceId: '',
        tenantId: '',
        payload: addFieldsText.value,
      }
      fields = { data: JSON.stringify(mqEnvelope) }
    } else {
      fields = JSON.parse(addFieldsText.value)
    }
    const result = await withReconnect(() =>
      getTauriAPI().dbRedisStreamAdd(
        props.connectionId,
        props.redisDbIndex ?? 0,
        addKey.value,
        fields,
        addMaxlen.value > 0 ? addMaxlen.value : undefined
      )
    ) as RedisStreamAddResponse
    if (result.success) {
      showAddModal.value = false
      addFieldsText.value = ''
      addMaxlen.value = 0
      toast.success('消息投递成功')
      await selectStream(addKey.value)
    }
  } catch (e: unknown) {
    toast.error('投递消息失败: ' + getErrorMessage(e))
  } finally {
    pushing.value = false
  }
}

function openAddMessage() {
  if (selectedStream.value) {addKey.value = selectedStream.value}
  showAddModal.value = true
}

async function deleteMessage(messageId: string) {
  if (!selectedStream.value) {return}
  if (!confirm(`确定删除消息 ${messageId} 吗？`)) {return}
  try {
    await withReconnect(() =>
      getTauriAPI().dbRedisStreamDel(props.connectionId, props.redisDbIndex ?? 0, selectedStream.value, messageId)
    )
    await loadMessages()
    toast.success('消息已删除')
  } catch (e: unknown) {
    toast.error('删除消息失败: ' + getErrorMessage(e))
  }
}

async function deleteStream() {
  if (!selectedStream.value) {return}
  if (!confirm(`确定删除整个 Stream "${selectedStream.value}" 吗？此操作不可恢复！`)) {return}
  try {
    await withReconnect(() =>
      getTauriAPI().dbRedisStreamDelete(props.connectionId, props.redisDbIndex ?? 0, selectedStream.value)
    )
    selectedStream.value = ''
    streamInfo.value = null
    groups.value = []
    messages.value = []
    toast.success('Stream 已删除')
    await refreshStreams()
  } catch (e: unknown) {
    toast.error('删除 Stream 失败: ' + getErrorMessage(e))
  }
}

// ==================== Groups ====================

async function loadGroups() {
  if (!selectedStream.value) {return}
  groupLoading.value = true
  try {
    const result = await withReconnect(() =>
      getTauriAPI().dbRedisStreamInfo(props.connectionId, props.redisDbIndex ?? 0, selectedStream.value)
    ) as RedisStreamInfoIPCResponse
    if (result.success) {
      groups.value = result.groups || []
    }
  } catch (e: unknown) {
    toast.error('加载消费组失败: ' + getErrorMessage(e))
  } finally {
    groupLoading.value = false
  }
}

async function createGroup() {
  if (!selectedStream.value || !newGroupName.value) {return}
  try {
    const result = await withReconnect(() =>
      getTauriAPI().dbRedisStreamGroupCreate(
        props.connectionId,
        props.redisDbIndex ?? 0,
        selectedStream.value,
        newGroupName.value,
        newGroupStartId.value || '0'
      )
    ) as RedisStreamAddResponse
    if (result.success) {
      showGroupModal.value = false
      newGroupName.value = ''
      newGroupStartId.value = '0'
      toast.success('消费组创建成功')
      await loadGroups()
      // Refresh consumption status since new group changes the baseline
      if (messages.value.length > 0) {await enrichMessageConsumptionStatus()}
    }
  } catch (e: unknown) {
    toast.error('创建消费组失败: ' + getErrorMessage(e))
  }
}

async function destroyGroup(groupName: string) {
  if (!selectedStream.value) {return}
  if (!confirm(`确定删除消费组 "${groupName}" 吗？`)) {return}
  try {
    await withReconnect(() =>
      getTauriAPI().dbRedisStreamGroupDestroy(props.connectionId, props.redisDbIndex ?? 0, selectedStream.value, groupName)
    )
    if (selectedGroup.value === groupName) {selectedGroup.value = ''}
    toast.success('消费组已删除')
    await loadGroups()
  } catch (e: unknown) {
    toast.error('删除消费组失败: ' + getErrorMessage(e))
  }
}

// ==================== Group Detail ====================

async function selectGroup(groupName: string) {
  selectedGroup.value = groupName
  groupDetailTab.value = 'consumers'
  await Promise.all([loadConsumers(groupName), loadPending(groupName)])
}

async function loadConsumers(groupName: string) {
  if (!selectedStream.value) {return}
  consumersLoading.value = true
  try {
    const result = await withReconnect(() =>
      getTauriAPI().dbRedisStreamConsumers(
        props.connectionId,
        props.redisDbIndex ?? 0,
        selectedStream.value,
        groupName
      )
    ) as RedisStreamConsumersResponse
    if (result.success) {
      consumers.value = result.consumers || []
    }
  } catch (e: unknown) {
    toast.error('加载消费者失败: ' + getErrorMessage(e))
  } finally {
    consumersLoading.value = false
  }
}

async function loadPending(groupName: string) {
  if (!selectedStream.value) {return}
  pendingLoading.value = true
  try {
    const result = await withReconnect(() =>
      getTauriAPI().dbRedisStreamPending(
        props.connectionId,
        props.redisDbIndex ?? 0,
        selectedStream.value,
        groupName,
        '-',
        '+',
        100
      )
    ) as RedisStreamPendingResponse
    if (result.success) {
      const pending = result.pending
      if (Array.isArray(pending) && pending.length > 0 && typeof pending[0] === 'object' && pending[0] !== null) {
        pendingMessages.value = (pending as Array<Record<string, unknown>>).map((p) => ({
          id: (p.id as string) || '',
          consumer: (p.consumer as string) || 'unknown',
          timesDelivered: (p.deliveryCount as number) ?? (p.timesDelivered as number) ?? 0,
          idleTime: (p.idleTime as number) ?? 0,
        }))
      } else {
        pendingMessages.value = []
      }
    }
  } catch (e: unknown) {
    toast.error('加载 pending 失败: ' + getErrorMessage(e))
  } finally {
    pendingLoading.value = false
  }
}

async function claimPending(messageId: string) {
  if (!selectedStream.value || !selectedGroup.value) {return}
  try {
    await withReconnect(() =>
      getTauriAPI().dbRedisStreamClaim(
        props.connectionId,
        props.redisDbIndex ?? 0,
        selectedStream.value,
        selectedGroup.value,
        claimConsumerName.value,
        messageId
      )
    )
    toast.success('消息 Claim 成功')
    await loadPending(selectedGroup.value)
  } catch (e: unknown) {
    toast.error('Claim 失败: ' + getErrorMessage(e))
  }
}

async function ackPending(messageId: string) {
  if (!selectedStream.value || !selectedGroup.value) {return}
  try {
    await withReconnect(() =>
      getTauriAPI().dbRedisStreamAck(
        props.connectionId,
        props.redisDbIndex ?? 0,
        selectedStream.value,
        selectedGroup.value,
        messageId
      )
    )
    toast.success('消息 Ack 成功')
    await loadPending(selectedGroup.value)
  } catch (e: unknown) {
    toast.error('Ack 失败: ' + getErrorMessage(e))
  }
}

async function retryPending(messageId: string) {
  if (!selectedStream.value || !selectedGroup.value) {return}
  if (!confirm(`确定重试消息 ${formatStreamId(messageId)} 吗？将 ACK 旧消息并重新投递。`)) {return}
  try {
    const result = await withReconnect(() =>
      getTauriAPI().dbRedisStreamRetry(
        props.connectionId,
        props.redisDbIndex ?? 0,
        selectedStream.value,
        selectedGroup.value,
        messageId
      )
    ) as RedisStreamRetryResponse
    if (result.success) {
      toast.success(`重试成功，新消息ID: ${formatStreamId(result.newMessageId || '')}`)
      await loadPending(selectedGroup.value)
      await loadMessages()
    }
  } catch (e: unknown) {
    toast.error('重试失败: ' + getErrorMessage(e))
  }
}

// ==================== Stats ====================

async function loadStats() {
  if (!selectedStream.value) {return}
  statsLoading.value = true
  try {
    groupStats.value = []
    for (const g of groups.value) {
      const groupName = g.name
      const pendingCount = g.pending
      let consumerList: ConsumerInfo[] = []
      try {
        const res = await withReconnect(() =>
          getTauriAPI().dbRedisStreamConsumers(props.connectionId, props.redisDbIndex ?? 0, selectedStream.value, groupName)
        ) as RedisStreamConsumersResponse
        if (res.success) {consumerList = res.consumers || []}
      } catch {}
      // Per-group health breakdown
      let healthyConsumers = 0, idleConsumers = 0, staleConsumers = 0
      for (const c of consumerList) {
        const cls = classifyConsumer(c.pending, c.idle)
        if (cls === 'healthy') {healthyConsumers++}
        else if (cls === 'idle') {idleConsumers++}
        else {staleConsumers++}
      }
      groupStats.value.push({
        name: groupName, pendingCount, consumers: consumerList,
        healthyConsumers, idleConsumers, staleConsumers,
      })
    }
  } catch (e: unknown) {
    toast.error('加载统计失败: ' + getErrorMessage(e))
  } finally {
    statsLoading.value = false
  }
}

// ==================== Trim ====================

async function trimQueue() {
  if (!selectedStream.value || !trimKeepN.value || trimKeepN.value < 10) {return}
  if (!confirm(`确定清理队列，仅保留最近 ${trimKeepN.value} 条消息？`)) {return}
  try {
    const result = await withReconnect(() =>
      getTauriAPI().dbRedisStreamTrim(props.connectionId, props.redisDbIndex ?? 0, selectedStream.value, trimKeepN.value)
    ) as RedisStreamTrimResponse
    if (result.success) {
      toast.success(`清理完成，删除了 ${result.trimmed} 条过期消息`)
      showTrimModal.value = false
      await loadStreamInfo()
    }
  } catch (e: unknown) {
    toast.error('清理失败: ' + getErrorMessage(e))
  }
}

// ==================== Delay Queues ====================

async function refreshDelayQueues() {
  try {
    const result = await withReconnect(() =>
      getTauriAPI().dbRedisSetKey(props.connectionId, props.redisDbIndex, 'delay:*', 'zset')
    ) as RedisScanKeysResponse
    if (result.success) {
      const delayList = await Promise.all((result.keys || []).map(async (key: string) => {
        let count = 0
        try {
          const infoRes = await withReconnect(() =>
            getTauriAPI().dbRedisExec(props.connectionId, `ZCARD ${key}`)
          ) as RedisExecResponse
          count = Number(infoRes?.result) || 0
        } catch {}
        return { name: key, count }
      }))
      delayQueues.value = delayList.filter((d: { name: string; count: number }) => d.count > 0)
    }
  } catch (e: unknown) {
    console.error('刷新延迟队列失败:', e)
  }
}

async function selectDelayQueue(name: string) {
  selectedDelayQueue.value = name
  selectedStream.value = ''
  await refreshDelayQueue()
}

async function refreshDelayQueue() {
  if (!selectedDelayQueue.value) {return}
  delayLoading.value = true
  try {
    const now = Date.now()
    const result = await withReconnect(() =>
      getTauriAPI().dbRedisZSetRange(
        props.connectionId,
        props.redisDbIndex ?? 0,
        selectedDelayQueue.value,
        -Infinity,
        Infinity,
        200
      )
    ) as RedisZSetRangeResponse
    if (result.success) {
      delayMessages.value = (result.entries || []).map((e: ZSetEntry) => ({
        value: e.value,
        score: e.score,
        messageJson: e.value,
        remainingMs: Math.max(0, e.score - now),
      }))
    }
  } catch (e: unknown) {
    toast.error('加载延迟消息失败: ' + getErrorMessage(e))
  } finally {
    delayLoading.value = false
  }
}

async function fireDelayMessage(dm: DelayMessage) {
  if (!selectedDelayQueue.value) {return}
  // Remove from ZSet
  const removeResult = await withReconnect(() =>
    getTauriAPI().dbRedisZSetRemove(props.connectionId, props.redisDbIndex ?? 0, selectedDelayQueue.value, dm.value)
  ) as RedisZSetRemoveResponse
  if (removeResult.success && removeResult.removed) {
    // Parse to find target queue
    try {
      const _parsed = JSON.parse(dm.value)
      // The message might contain a target queue, or we can just XADD to a temp queue
      // For simplicity, we XADD to a derived queue name
      const targetQueue = selectedDelayQueue.value.replace('delay:', '')
      const addResult = await withReconnect(() =>
        getTauriAPI().dbRedisStreamAdd(props.connectionId, props.redisDbIndex ?? 0, targetQueue, { data: dm.value })
      ) as RedisStreamAddResponse
      if (addResult.success) {
        toast.success('延迟消息已投递到队列')
        await refreshDelayQueue()
        await refreshStreams()
      }
    } catch {
      toast.error('无法解析延迟消息格式')
    }
  }
}

async function deleteDelayMessage(dm: DelayMessage) {
  if (!selectedDelayQueue.value) {return}
  if (!confirm('确定删除此延迟消息？')) {return}
  try {
    const result = await withReconnect(() =>
      getTauriAPI().dbRedisZSetRemove(props.connectionId, props.redisDbIndex ?? 0, selectedDelayQueue.value, dm.value)
    ) as RedisZSetRemoveResponse
    if (result.success) {
      toast.success('延迟消息已删除')
      await refreshDelayQueue()
    }
  } catch (e: unknown) {
    toast.error('删除失败: ' + getErrorMessage(e))
  }
}

// ==================== Auto Refresh ====================

function toggleAutoRefresh() {
  autoRefreshEnabled.value = !autoRefreshEnabled.value
  if (autoRefreshEnabled.value) {
    startAutoRefresh()
  } else {
    stopAutoRefresh()
  }
}

function startAutoRefresh() {
  stopAutoRefresh()
  autoRefreshTimer = setInterval(() => {
    if (selectedStream.value) {
      // Lightweight refresh: only update stream info (length, groups, pending)
      // Don't pull full message content to save bandwidth
      loadStreamInfo()
    }
    if (selectedDelayQueue.value) {
      refreshDelayQueue()
    }
    refreshDelayQueues()
  }, parseInt(autoRefreshInterval.value))
}

function stopAutoRefresh() {
  if (autoRefreshTimer) {
    clearInterval(autoRefreshTimer)
    autoRefreshTimer = null
  }
}

function restartAutoRefresh() {
  if (autoRefreshEnabled.value) {startAutoRefresh()}
}

// ==================== Utilities ====================

function isJSON(str: string): boolean {
  try {
    JSON.parse(str)
    return true
  } catch {
    return false
  }
}

function formatValue(val: string): string {
  try {
    return JSON.stringify(JSON.parse(val), null, 2)
  } catch {
    return val
  }
}

function formatStreamId(id: string): string {
  if (id.length > 20) {return '...' + id.slice(-18)}
  return id
}

// Compare two Redis Stream IDs: returns -1, 0, or 1
function compareStreamIds(a: string, b: string): number {
  if (a === b) {return 0}
  const [aTs = '0', aSeq = '0'] = a.split('-').map(Number)
  const [bTs = '0', bSeq = '0'] = b.split('-').map(Number)
  if (aTs !== bTs) {return aTs < bTs ? -1 : 1}
  return aSeq < bSeq ? -1 : 1
}

// Load pending messages for all groups and build a Set of pending message IDs
async function loadAllPendingIds(): Promise<Set<string>> {
  const pendingSet = new Set<string>()
  if (!selectedStream.value || groups.value.length === 0) {return pendingSet}
  
  for (const g of groups.value) {
    try {
      const res = await withReconnect(() =>
        getTauriAPI().dbRedisStreamPending(
          props.connectionId,
          props.redisDbIndex ?? 0,
          selectedStream.value,
          g.name as string,
          '-', '+', 1000
        )
      ) as RedisStreamPendingResponse
      if (res.success && res.pending) {
        const pending = res.pending as unknown[]
        if (Array.isArray(pending)) {
          for (const p of pending) {
            if (typeof p === 'object' && p !== null && 'id' in p) {
              const id = (p as Record<string, unknown>).id
              if (typeof id === 'string') {pendingSet.add(id)}
            }
            else if (typeof p === 'string') {pendingSet.add(p)}
          }
        }
      }
    } catch {}
  }
  return pendingSet
}

// Enrich messages with consumption status
async function enrichMessageConsumptionStatus() {
  if (groups.value.length === 0) {
    // No consumer groups = all messages are "new"
    msgConsumptionStatus.value = new Map()
    return
  }
  
  const pendingSet = await loadAllPendingIds()
  pendingMsgIds.value = pendingSet
  
  const statusMap = new Map<string, { status: 'consumed' | 'pending' | 'new'; groupStatuses: Array<{ name: string; status: string }> }>()
  
  for (const msg of messages.value) {
    const msgId = msg.id
    const groupStatuses: Array<{ name: string; status: string }> = []
    let hasNew = false
    let hasPending = false
    let _hasConsumed = false
    
    for (const g of groups.value) {
      const lastDeliveredId = (g['lastDeliveredId'] as string) || '0-0'
      const cmp = compareStreamIds(msgId, lastDeliveredId)
      
      if (cmp <= 0) {
        // Message ID <= last-delivered-id: has been delivered to this group
        if (pendingSet.has(msgId)) {
          groupStatuses.push({ name: g.name as string, status: 'pending' })
          hasPending = true
        } else {
          groupStatuses.push({ name: g.name as string, status: 'consumed' })
          _hasConsumed = true
        }
      } else {
        // Message ID > last-delivered-id: not yet delivered
        groupStatuses.push({ name: g.name as string, status: 'new' })
        hasNew = true
      }
    }
    
    // Overall status
    let overall: 'consumed' | 'pending' | 'new'
    if (hasNew) {overall = 'new'}
    else if (hasPending) {overall = 'pending'}
    else {overall = 'consumed'}
    
    statusMap.set(msgId, { status: overall, groupStatuses })
  }
  
  msgConsumptionStatus.value = statusMap
}

function getConsumptionBadgeClass(status: string): string {
  if (status === 'consumed') {return 'badge-consumed'}
  if (status === 'pending') {return 'badge-pending'}
  return 'badge-new'
}

function getConsumptionLabel(status: string): string {
  if (status === 'consumed') {return '已消费'}
  if (status === 'pending') {return '处理中'}
  return '未消费'
}

function getConsumptionIcon(status: string): string {
  if (status === 'consumed') {return '✅'}
  if (status === 'pending') {return '⏳'}
  return '📭'
}

function shortId(id: string): string {
  if (!id) {return ''}
  return id.length > 12 ? id.slice(0, 6) + '...' + id.slice(-6) : id
}

function formatTime(isoStr: string): string {
  if (!isoStr) {return ''}
  try {
    const d = new Date(isoStr)
    return d.toLocaleString('zh-CN', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit', second: '2-digit' })
  } catch {
    return isoStr
  }
}

function formatTimestamp(ts: number): string {
  return new Date(ts).toLocaleString('zh-CN')
}

function formatDuration(ms: number): string {
  if (ms < 1000) {return `${ms}ms`}
  if (ms < 60000) {return `${Math.floor(ms / 1000)}s`}
  if (ms < 3600000) {return `${Math.floor(ms / 60000)}m ${Math.floor((ms % 60000) / 1000)}s`}
  return `${Math.floor(ms / 3600000)}h ${Math.floor((ms % 3600000) / 60000)}m`
}

function formatJsonPreview(jsonStr: string): string {
  if (!jsonStr) {return ''}
  try {
    const parsed = JSON.parse(jsonStr)
    const str = JSON.stringify(parsed, null, 2)
    return str.length > 500 ? str.slice(0, 500) + '\n...' : str
  } catch {
    return jsonStr
  }
}

function formatJsonDisplay(jsonStr: string): string {
  if (!jsonStr) {return ''}
  try {
    return JSON.stringify(JSON.parse(jsonStr), null, 2)
  } catch {
    return jsonStr
  }
}

function generateUUID(): string {
  return crypto.randomUUID?.() || Date.now().toString(36) + Math.random().toString(36).slice(2)
}

function getHealthClass(pending: number, idle: number): string {
  if (pending > 10 || idle > 600000) {return 'dot-danger'}
  if (pending > 0 || idle > 60000) {return 'dot-warn'}
  return 'dot-healthy'
}

// Extract short pod ID from consumer name (e.g. "ecm-9e92-0001-08c1c645" → "08c1c645")
function extractPodId(name: string): string {
  const parts = name.split('-')
  if (parts.length >= 4) {
    return parts.slice(3).join('-').replace('-retrier', '')
  }
  return name.length > 30 ? name.slice(0, 30) + '…' : name
}

// Check if consumer is a retrier
function isRetrier(name: string): boolean {
  return name.endsWith('-retrier')
}

// Card-level health class
function getConsumerHealthClass(pending: number, idle: number): string {
  if (idle > 86400000) {return 'card-stale'}
  if (idle > 3600000) {return 'card-idle'}
  return 'card-healthy'
}

// Health indicator dot
function getConsumerHealthDot(pending: number, idle: number): string {
  if (pending > 10) {return 'dot-danger'}
  if (pending > 0 || idle > 86400000) {return 'dot-stale'}
  if (idle > 3600000) {return 'dot-idle'}
  return 'dot-healthy'
}

async function copyText(text: string) {
  try {
    await navigator.clipboard.writeText(text)
    toast.success('已复制到剪贴板')
  } catch {
    toast.error('复制失败')
  }
}

async function refreshAll() {
  await refreshStreams()
  await refreshDelayQueues()
  if (selectedStream.value) {await selectStream(selectedStream.value)}
  if (selectedDelayQueue.value) {await refreshDelayQueue()}
}

watch(detailTab, () => {
  if (detailTab.value === 'groups') {loadGroups()}
})

watch(groupDetailTab, (newTab) => {
  if (newTab === 'consumers' && selectedGroup.value) {loadConsumers(selectedGroup.value)}
  else if (newTab === 'pending' && selectedGroup.value) {loadPending(selectedGroup.value)}
})

onMounted(async () => {
  await checkConnection()
  refreshStreams()
  refreshDelayQueues()
})

onUnmounted(() => {
  stopAutoRefresh()
})

  return {
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
  }
}
