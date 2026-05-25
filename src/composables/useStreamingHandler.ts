import { ref, reactive, computed, type Ref } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

// 工具调用详情
export interface ToolCall {
  id?: string; // 工具调用唯一 ID
  name: string;
  args?: Record<string, unknown>; // 工具参数
  result?: string; // 工具返回结果
  durationMs: number;
  isSubAgent?: boolean; // 是否是子 agent
  status?: 'running' | 'completed' | 'error'; // 状态
  // Hermes API 返回的友好显示字段
  label?: string; // 工具调用友好标签（如 "读取文件"）
  emoji?: string; // 工具调用图标 emoji
}

// 消息类型
export interface Message {
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
  isChild?: boolean; // 是否来自子会话（subagent）
  sessionId?: string; // 子会话的 session_id
}

// 路径项
export interface PathItem {
  path: string;
  type: 'file' | 'folder';
  name: string;
  previewUrl?: string;
}

// Task item from todo tool
export interface TaskItem {
  id: string;
  content: string;
  status: 'pending' | 'in_progress' | 'completed' | 'cancelled';
}

// useStreamingHandler 参数
export interface UseStreamingHandlerOptions {
  currentSessionId: Ref<string | null>;
  messages: Ref<Message[]>;
  currentTasks?: Ref<TaskItem[]>; // 可选，用于 todo 工具更新任务列表
  agentLog?: (message: string) => Promise<void>; // 可选日志函数
  scrollToBottom?: () => void; // 可选滚动函数
}

// useStreamingHandler 返回值
export interface UseStreamingHandlerReturn {
  // 流式响应状态
  streamingSessions: Record<string, boolean>;
  thinkingTexts: Record<string, string>;
  sessionRoundEnded: Record<string, boolean>;
  sessionMessagesCache: Record<string, Message[]>;
  
  // 计算属性
  isStreaming: Readonly<Ref<boolean>>;
  thinkingText: Ref<string>; // 可写的 computed
  
  // 方法
  setupStreamingListeners: () => Promise<void>;
  cleanupStreamingListeners: () => void;
  
  // 内部状态访问（用于高级用例）
  setStreaming: (sessionId: string, value: boolean) => void;
  setThinkingText: (sessionId: string, value: string) => void;
  resetRoundEnded: (sessionId: string) => void;
}

/**
 * 流式响应处理 composable
 * 处理 Tauri event 监听和状态更新
 */
export function useStreamingHandler(options: UseStreamingHandlerOptions): UseStreamingHandlerReturn {
  const { currentSessionId, messages, currentTasks, agentLog, scrollToBottom } = options;
  
  // 每个会话独立的状态（支持同时处理多个会话）
  const streamingSessions = reactive<Record<string, boolean>>({});  // 各会话是否流式响应中
  const thinkingTexts = reactive<Record<string, string>>({});       // 各会话的思考/工具文字
  const sessionRoundEnded = reactive<Record<string, boolean>>({});  // 各会话的轮次结束标记
  const sessionMessagesCache = reactive<Record<string, Message[]>>({});  // 各会话的消息缓存
  
  // 计算属性：当前会话是否正在流式响应
  const isStreaming = computed(() => !!currentSessionId.value && !!streamingSessions[currentSessionId.value]);
  
  // 计算属性：当前会话的思考文本
  const thinkingText = computed({
    get: () => currentSessionId.value ? (thinkingTexts[currentSessionId.value] || '') : '',
    set: (val: string) => { if (currentSessionId.value) {thinkingTexts[currentSessionId.value] = val;} },
  });
  
  // Event listeners 清理函数
  let unlistenDelta: UnlistenFn | null = null;
  let unlistenThinking: UnlistenFn | null = null;
  let unlistenToolStart: UnlistenFn | null = null;
  let unlistenToolComplete: UnlistenFn | null = null;
  let unlistenError: UnlistenFn | null = null;
  let unlistenDone: UnlistenFn | null = null;
  
  // 日志函数（默认空实现）
  const log = agentLog || (async (_message: string) => {});
  
  // 滚动函数（默认空实现）
  const scroll = scrollToBottom || (() => {});
  
  /**
   * 处理 agent-delta 事件（文本增量）
   */
  const handleDelta = async (event: { payload: { text: string | null; session_id: string | null } }) => {
    const eventSid = event.payload?.session_id;
    void log('[agent-delta] 收到事件: ' + JSON.stringify(event.payload?.text?.slice(0, 50)) + ' session_id: ' + eventSid);
    
    // 必须有 session_id 才处理
    if (!eventSid) {return;}
    
    // 更新该会话的状态
    if (event.payload?.text) {thinkingTexts[eventSid] = '';}
    
    // 获取该会话的消息缓存（优先用缓存，支持后台会话）
    let sessionMsgs = sessionMessagesCache[eventSid];
    if (!sessionMsgs) {
      // 如果当前会话没有缓存，从 messages.value 复制（当前视图）
      // 处理首次对话：currentSessionId.value 是 null，eventSid 是新创建的 session_id
      if (currentSessionId.value === null || eventSid === currentSessionId.value) {
        sessionMessagesCache[eventSid] = [...messages.value];
        sessionMsgs = sessionMessagesCache[eventSid];
      } else {
        // 非当前会话（子会话），初始化空数组
        sessionMessagesCache[eventSid] = [];
        sessionMsgs = sessionMessagesCache[eventSid];
      }
    }
    
    if (event.payload?.text) {
      // 查找最后一个 assistant 消息（在缓存中）
      const messagesCopy = [...sessionMsgs].reverse();
      let currentMsg: Message | undefined = messagesCopy.find((m: Message) => m.role === 'assistant');
      
      // 检查最后一条消息是否是 user（刚发送的），如果是，需要创建新 assistant 消息
      const lastMsg = sessionMsgs[sessionMsgs.length - 1];
      const needsNewMsg = lastMsg?.role === 'user';
      
      // 检查是否已有空内容的 assistant 消息（由 tool_start 创建），避免重复创建
      const hasEmptyAssistant = currentMsg && !currentMsg.content && currentMsg.toolCalls && currentMsg.toolCalls.length > 0;
      
      void log('[agent-delta] session: ' + eventSid + 
        ' 当前 assistant 消息: ' + (currentMsg ? '存在' : '不存在') +
        ' lastAssistantRoundEnded: ' + !!sessionRoundEnded[eventSid] +
        ' 最后一条: ' + (lastMsg?.role || 'none') +
        ' needsNewMsg: ' + needsNewMsg +
        ' hasEmptyAssistant: ' + hasEmptyAssistant);
      
      // 如果没有 assistant 消息，或上一轮已结束，或最后一条是 user（需要新消息），创建新消息
      // 但如果已有空内容的 assistant 消息（由 tool_start 创建），则复用
      const roundEnded = !!sessionRoundEnded[eventSid];
      if (!currentMsg || (roundEnded && !hasEmptyAssistant) || needsNewMsg) {
        const newMsg: Message = {
          role: 'assistant',
          content: '',
          timestamp: Date.now() / 1000,
          toolName: null,
          toolCalls: [],
        };
        sessionMsgs.push(newMsg);
        // 从缓存获取 Vue 的 reactive proxy
        currentMsg = sessionMsgs[sessionMsgs.length - 1];
        sessionRoundEnded[eventSid] = false;
        void log('[agent-delta] 创建新 assistant 消息, 缓存 length: ' + sessionMsgs.length);
      } else if (hasEmptyAssistant) {
        // 复用已有的空内容 assistant 消息
        sessionRoundEnded[eventSid] = false;
        void log('[agent-delta] 复用已有空 assistant 消息');
      }
      
      // 添加 delta 内容
      if (currentMsg) {
        currentMsg.content = (currentMsg.content || '') + event.payload.text;
      }
      
      // 同步到 messages.value
      // 处理首次对话的情况：currentSessionId.value 是 null，但 eventSid 是新创建的 session_id
      // 此时应该立即更新 currentSessionId，并按主会话处理
      if (currentSessionId.value === null) {
        // 首次对话，立即更新 currentSessionId
        currentSessionId.value = eventSid;
        messages.value = [...sessionMsgs];
        scroll();
      } else if (eventSid === currentSessionId.value) {
        messages.value = [...sessionMsgs];
        scroll();
      } else {
        // 子会话消息需要标记 isChild 和 sessionId
        const syncedMsgs = sessionMsgs.map(m => ({
          ...m,
          isChild: true,
          sessionId: eventSid,
        }));
        // 合并到 messages.value：保留主会话消息 + 更新该子会话的消息
        const mainMsgs = messages.value.filter(m => !m.isChild || m.sessionId !== eventSid);
        messages.value = [...mainMsgs, ...syncedMsgs];
        scroll();
      }
    }
  };
  
  /**
   * 处理 agent-tool-start 事件（工具调用开始）
   */
  const handleToolStart = async (event: { payload: { id?: string; name: string; args: unknown; session_id: string | null; label?: string; emoji?: string } }) => {
    const eventSid = event.payload?.session_id;
    void log('[agent-tool-start] 收到事件: ' + JSON.stringify(event.payload) + ' session_id: ' + eventSid);
    
    // 必须有 session_id 才处理
    if (!eventSid) {return;}
    
    // 更新该会话的状态
    if (event.payload.name) {
      const isSubAgent = event.payload.name === 'delegate_task';
      thinkingTexts[eventSid] = isSubAgent ? '🤖 启动子 Agent 处理任务...' : `🔧 调用工具: ${event.payload.name}...`;
    }
    sessionRoundEnded[eventSid] = false;
    
    // 获取该会话的消息缓存
    let sessionMsgs = sessionMessagesCache[eventSid];
    if (!sessionMsgs) {
      // 处理首次对话：currentSessionId.value 是 null，eventSid 是新创建的 session_id
      if (currentSessionId.value === null || eventSid === currentSessionId.value) {
        sessionMessagesCache[eventSid] = [...messages.value];
        sessionMsgs = sessionMessagesCache[eventSid];
      } else {
        sessionMessagesCache[eventSid] = [];
        sessionMsgs = sessionMessagesCache[eventSid];
      }
    }
    
    // 工具开始
    const toolId = event.payload.id;
    const toolName = event.payload.name;
    const isSubAgent = toolName === 'delegate_task';
    
    // 使用 Hermes 返回的友好标签（如果有）
    const displayLabel = event.payload.label || toolName;
    const displayEmoji = event.payload.emoji || (isSubAgent ? '🤖' : '🔧');
    
    // 更新思考状态文本（使用友好标签）
    thinkingTexts[eventSid] = `${displayEmoji} ${displayLabel}...`;
    
    // 获取当前消息（如果没有 assistant 消息，创建一个）
    const messagesCopy = [...sessionMsgs].reverse();
    let currentMsg: Message | undefined = messagesCopy.find((m: Message) => m.role === 'assistant');
    
    // 检查最后一条消息是否是 user（刚发送的），如果是，需要创建新 assistant 消息
    const lastMsg = sessionMsgs[sessionMsgs.length - 1];
    const needsNewMsg = lastMsg?.role === 'user';
    
    void log('[agent-tool-start] session: ' + eventSid +
      ' 当前 assistant 消息: ' + (currentMsg ? '存在' : '不存在') +
      ' 最后一条: ' + (lastMsg?.role || 'none') +
      ' needsNewMsg: ' + needsNewMsg + ' toolId: ' + (toolId || 'none') +
      ' label: ' + displayLabel + ' emoji: ' + displayEmoji);
    
    // 重置轮结束标志（新的工具调用开始）
    sessionRoundEnded[eventSid] = false;
    
    if (!currentMsg || needsNewMsg) {
      const newMsg: Message = {
        role: 'assistant',
        content: '',
        timestamp: Date.now() / 1000,
        toolName: null,
        toolCalls: [],
      };
      sessionMsgs.push(newMsg);
      currentMsg = sessionMsgs[sessionMsgs.length - 1];
      void log('[agent-tool-start] 创建新 assistant 消息, 缓存 length: ' + sessionMsgs.length);
    }
    
    // 确保 toolCalls 数组存在
    if (!currentMsg.toolCalls) {
      currentMsg.toolCalls = [];
    }
    
    // 添加工具调用
    currentMsg.toolCalls.push({
      id: toolId,
      name: toolName,
      args: event.payload.args as Record<string, unknown> || {},
      durationMs: 0,
      isSubAgent,
      status: 'running',
      // 新增：友好显示标签
      label: displayLabel,
      emoji: displayEmoji,
    });
    void log('[agent-tool-start] 添加工具调用: ' + toolName + ' id: ' + (toolId || 'none') + ' toolCalls.length: ' + currentMsg.toolCalls.length);
    
    // 如果是当前会话，同步更新 messages.value
    // 处理首次对话：currentSessionId.value 是 null，立即更新并按主会话处理
    if (currentSessionId.value === null) {
      currentSessionId.value = eventSid;
      messages.value = [...sessionMsgs];
      thinkingText.value = isSubAgent ? '🤖 启动子 Agent 处理任务...' : `🔧 调用工具: ${toolName}...`;
      scroll();
    } else if (eventSid === currentSessionId.value) {
      messages.value = [...sessionMsgs];
      // 显示提示
      if (isSubAgent) {
        thinkingText.value = '🤖 启动子 Agent 处理任务...';
      } else {
        thinkingText.value = `🔧 调用工具: ${toolName}...`;
      }
      scroll();
    } else {
      // 子会话消息需要标记 isChild 和 sessionId
      const syncedMsgs = sessionMsgs.map(m => ({
        ...m,
        isChild: true,
        sessionId: eventSid,
      }));
      const mainMsgs = messages.value.filter(m => !m.isChild || m.sessionId !== eventSid);
      messages.value = [...mainMsgs, ...syncedMsgs];
      scroll();
    }
  };
  
  /**
   * 处理 agent-tool-complete 事件（工具调用完成）
   */
  const handleToolComplete = async (event: { payload: { id?: string; name: string; result: string | null; duration_ms: number; session_id: string | null } }) => {
    const eventSid = event.payload?.session_id;
    void log('[agent-tool-complete] 收到事件: ' + JSON.stringify({id: event.payload.id, name: event.payload.name, duration_ms: event.payload.duration_ms, session_id: eventSid}));
    
    // 必须有 session_id 才处理
    if (!eventSid) {return;}
    
    // 更新该会话的状态
    sessionRoundEnded[eventSid] = true;
    
    // 获取该会话的消息缓存
    let sessionMsgs = sessionMessagesCache[eventSid];
    if (!sessionMsgs) {
      if (eventSid === currentSessionId.value) {
        sessionMessagesCache[eventSid] = [...messages.value];
        sessionMsgs = sessionMessagesCache[eventSid];
      } else {
        // 非当前会话，没有缓存就跳过（这种情况不应该发生）
        return;
      }
    }
    
    // 获取当前 assistant 消息
    const messagesCopy = [...sessionMsgs].reverse();
    const currentMsg = messagesCopy.find((m: Message) => m.role === 'assistant');
    void log('[agent-tool-complete] session: ' + eventSid + 
      ' 当前 assistant 消息: ' + (currentMsg ? '存在' : '不存在') + 
      ' toolCalls: ' + (currentMsg?.toolCalls?.length || 0));
    
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
        void log('[agent-tool-complete] 更新工具调用: ' + event.payload.name + ' id: ' + (toolId || 'none') + ' status: completed');
      } else {
        void log('[agent-tool-complete] 未找到匹配的 running 工具调用, id: ' + (toolId || 'none'));
      }
    }
    
    // 标记当前轮次结束
    sessionRoundEnded[eventSid] = true;
    void log('[agent-tool-complete] 设置 sessionRoundEnded = true');
    
    // 同步到 messages.value
    // 处理首次对话：currentSessionId.value 是 null，立即更新并按主会话处理
    if (currentSessionId.value === null) {
      currentSessionId.value = eventSid;
      messages.value = [...sessionMsgs];
    } else if (eventSid === currentSessionId.value) {
      messages.value = [...sessionMsgs];
    } else {
      // 子会话消息需要标记 isChild 和 sessionId
      const syncedMsgs = sessionMsgs.map(m => ({
        ...m,
        isChild: true,
        sessionId: eventSid,
      }));
      const mainMsgs = messages.value.filter(m => !m.isChild || m.sessionId !== eventSid);
      messages.value = [...mainMsgs, ...syncedMsgs];
    }
    
    // 如果是 todo 工具，更新任务列表（仅当前会话）
    if (currentTasks && (currentSessionId.value === null || eventSid === currentSessionId.value) && event.payload.name === 'todo' && event.payload.result) {
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
    
    scroll();
  };
  
  /**
   * 处理 agent-thinking 事件（思考动画）
   */
  const handleThinking = async (event: { payload: { text: string | null; session_id: string | null } }) => {
    const eventSid = event.payload?.session_id;
    // 更新该会话的思考文字（不论是否当前会话）
    if (eventSid) {
      thinkingTexts[eventSid] = event.payload?.text || '';
    }
    // 当前会话时同步到视图
    if (eventSid && currentSessionId.value && eventSid === currentSessionId.value) {
      if (event.payload?.text) {
        thinkingText.value = event.payload.text;
      } else {
        thinkingText.value = '';
      }
    }
  };
  
  /**
   * 处理 agent-error 事件
   */
  const handleError = async (event: { payload: { message: string; session_id: string | null } }) => {
    const eventSid = event.payload?.session_id;
    void log('[agent-error] 收到事件: ' + event.payload?.message + ' session_id: ' + eventSid);
    
    // 必须有 session_id 才处理
    if (!eventSid) {return;}
    
    // 更新该会话的状态
    streamingSessions[eventSid] = false;
    thinkingTexts[eventSid] = '';
    
    // 获取该会话的消息缓存
    let sessionMsgs = sessionMessagesCache[eventSid];
    if (!sessionMsgs) {
      // 处理首次对话：currentSessionId.value 是 null，eventSid 是新创建的 session_id
      if (currentSessionId.value === null || eventSid === currentSessionId.value) {
        sessionMessagesCache[eventSid] = [...messages.value];
        sessionMsgs = sessionMessagesCache[eventSid];
      } else {
        sessionMessagesCache[eventSid] = [];
        sessionMsgs = sessionMessagesCache[eventSid];
      }
    }
    
    // 在缓存中添加错误信息
    const messagesCopy = [...sessionMsgs].reverse();
    const currentMsg = messagesCopy.find((m: Message) => m.role === 'assistant');
    if (currentMsg) {
      currentMsg.content = (currentMsg.content || '') + `\n[错误: ${event.payload?.message}]`;
    }
    
    // 如果是当前会话，同步更新 messages.value
    // 处理首次对话：currentSessionId.value 是 null，立即更新并按主会话处理
    if (currentSessionId.value === null) {
      currentSessionId.value = eventSid;
      messages.value = [...sessionMsgs];
      thinkingText.value = '';
    } else if (eventSid === currentSessionId.value) {
      messages.value = [...sessionMsgs];
      thinkingText.value = '';
    } else {
      // 子会话消息需要标记 isChild 和 sessionId
      const syncedMsgs = sessionMsgs.map(m => ({
        ...m,
        isChild: true,
        sessionId: eventSid,
      }));
      const mainMsgs = messages.value.filter(m => !m.isChild || m.sessionId !== eventSid);
      messages.value = [...mainMsgs, ...syncedMsgs];
    }
  };
  
  /**
   * 处理 agent-done 事件（流式结束）
   */
  const handleDone = async (event: { payload: { response: string | null; session_id: string; message_count: number } }) => {
    const eventSid = event.payload?.session_id;
    void log('[agent-done] 收到事件: ' + JSON.stringify(event.payload));
    // 更新该会话的状态（不论是否当前会话）
    if (eventSid) {
      streamingSessions[eventSid] = false;
      thinkingTexts[eventSid] = '';
      sessionRoundEnded[eventSid] = true; // 标记这一轮已结束，下一轮新消息需要创建新 assistant 消息
      // 流式结束后清除缓存（下次切换会话会从数据库加载）
      delete sessionMessagesCache[eventSid];
      
      // 播放提醒音（对话完毕）
      playNotificationSound();
    }
    // 当前会话时同步到视图
    // 处理首次对话：currentSessionId.value 可能是 null，需要更新
    if (eventSid && (currentSessionId.value === null || eventSid === currentSessionId.value)) {
      if (currentSessionId.value === null) {
        currentSessionId.value = eventSid;
      }
      thinkingText.value = '';
      if (currentSessionId.value) {sessionRoundEnded[currentSessionId.value] = true;}
      void log('[agent-done] messages.length: ' + messages.value.length + ' 最后一条: ' + (messages.value[messages.value.length - 1]?.role || 'none'));
      if (currentSessionId.value) {streamingSessions[currentSessionId.value] = false;} // trigger computed update via streamingSessions
    }
    // 恢复 UI 状态（仅当该会话是当前会话时）
    if (!eventSid || (currentSessionId.value && eventSid === currentSessionId.value)) {
      scroll();
    }
  };
  
  /**
   * 播放提醒音（对话完毕时）
   */
  const playNotificationSound = () => {
    try {
      // 使用 Web Audio API 播放短促提示音
      const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
      const oscillator = audioContext.createOscillator();
      const gainNode = audioContext.createGain();
      
      oscillator.connect(gainNode);
      gainNode.connect(audioContext.destination);
      
      // 设置音调（800Hz，柔和的提示音）
      oscillator.frequency.setValueAtTime(800, audioContext.currentTime);
      oscillator.type = 'sine';
      
      // 设置音量（渐弱效果）
      gainNode.gain.setValueAtTime(0.3, audioContext.currentTime);
      gainNode.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 0.3);
      
      // 播放 0.3 秒
      oscillator.start(audioContext.currentTime);
      oscillator.stop(audioContext.currentTime + 0.3);
    } catch (e) {
      // 静默失败（某些浏览器可能不支持 Web Audio API）
      console.log('[notification] 无法播放提醒音:', e);
    }
  };
  
  /**
   * 设置流式事件监听
   */
  const setupStreamingListeners = async () => {
    unlistenDelta = await listen<{ text: string | null; session_id: string | null }>('agent-delta', handleDelta);
    unlistenToolStart = await listen<{ id?: string; name: string; args: unknown; session_id: string | null }>('agent-tool-start', handleToolStart);
    unlistenToolComplete = await listen<{ id?: string; name: string; result: string | null; duration_ms: number; session_id: string | null }>('agent-tool-complete', handleToolComplete);
    unlistenThinking = await listen<{ text: string | null; session_id: string | null }>('agent-thinking', handleThinking);
    unlistenError = await listen<{ message: string; session_id: string | null }>('agent-error', handleError);
    unlistenDone = await listen<{ response: string | null; session_id: string; message_count: number }>('agent-done', handleDone);
  };
  
  /**
   * 清理流式事件监听
   */
  const cleanupStreamingListeners = () => {
    unlistenDelta?.();
    unlistenThinking?.();
    unlistenToolStart?.();
    unlistenToolComplete?.();
    unlistenError?.();
    unlistenDone?.();
    unlistenDelta = null;
    unlistenThinking = null;
    unlistenToolStart = null;
    unlistenToolComplete = null;
    unlistenError = null;
    unlistenDone = null;
  };
  
  // 手动设置流式状态的方法
  const setStreaming = (sessionId: string, value: boolean) => {
    streamingSessions[sessionId] = value;
  };
  
  const setThinkingText = (sessionId: string, value: string) => {
    thinkingTexts[sessionId] = value;
  };
  
  const resetRoundEnded = (sessionId: string) => {
    sessionRoundEnded[sessionId] = false;
  };
  
  return {
    streamingSessions,
    thinkingTexts,
    sessionRoundEnded,
    sessionMessagesCache,
    isStreaming,
    thinkingText,
    setupStreamingListeners,
    cleanupStreamingListeners,
    setStreaming,
    setThinkingText,
    resetRoundEnded,
  };
}