import { ref, type Ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { GitRepo } from '@/types';

/**
 * 会话接口定义
 */
export interface Session {
  id: string;
  title: string | null;
  model: string;
  source: string;
  startedAt?: number;
  endedAt?: number | null;
  messageCount: number;
  preview: string;
  lastActive?: number;
  parentSessionId?: string | null;
}

/**
 * 搜索结果接口（来自 Hermes FTS5 搜索）
 */
export interface SearchResult {
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

/**
 * 消息加载回调参数
 */
export interface MessageLoadParams {
  sessionId: string;
  messages: any[];
}

/**
 * 会话管理 Composable
 * 提供会话列表管理、会话选择、搜索、删除等功能
 */
export function useSessionManager() {
  // ===== 状态 =====
  const sessions = ref<Session[]>([]);
  const searchResults = ref<SearchResult[]>([]);
  const isSearching = ref(false);
  const currentSessionId = ref<string | null>(null);
  const currentSession = ref<Session | null>(null);
  const loadingSessions = ref(false);
  const gitRepos = ref<GitRepo[]>([]);

  // 搜索防抖定时器
  let searchDebounceTimer: number | null = null;

  // ===== 会话列表管理 =====

  /**
   * 刷新会话列表
   */
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

  /**
   * 选择会话（只设置状态，消息加载由回调处理）
   * @param session 要选择的会话
* @param onLoadMessages 消息加载回调（可选）
  */
  const selectSession = async (
    session: Session,
    onLoadMessages?: (params: MessageLoadParams) => Promise<void> | void
  ) => {
    // CRITICAL: Resolve compression tip first
    // If the session has been compressed, use the latest continuation session_id
    let effectiveSessionId = session.id;
    try {
      const tipResult = await invoke<{ success: boolean; tipSessionId: string; originalSessionId: string }>(
        'agent_get_compression_tip',
        { sessionId: session.id }
      );
      if (tipResult.success && tipResult.tipSessionId !== session.id) {
        console.log(`[SessionManager] Compression tip resolved: ${session.id} -> ${tipResult.tipSessionId}`);
        effectiveSessionId = tipResult.tipSessionId;
        // Update session id to effective id (compression tip)
        session = { ...session, id: effectiveSessionId };
      }
    } catch (e) {
      console.warn('[SessionManager] Failed to resolve compression tip:', e);
    }

    currentSessionId.value = effectiveSessionId;
    currentSession.value = session;

    // 如果提供了消息加载回调，调用它
    if (onLoadMessages) {
      try {
        const result = await invoke<{ success: boolean; messages: any[]; sessionId: string }>(
          'agent_list_messages',
          { sessionId: effectiveSessionId }
        );
        if (result.success && result.messages) {
          await onLoadMessages({ sessionId: effectiveSessionId, messages: result.messages });
        }
      } catch (e) {
        console.error('Failed to load messages:', e);
      }
    }
  };

  /**
   * 开始新对话
   * @param onClear 清理回调（可选，用于清理消息、输入框等）
   */
  const startNewChat = (onClear?: () => void) => {
    currentSessionId.value = null;
    currentSession.value = null;
    if (onClear) {
      onClear();
    }
  };

  /**
   * 删除指定会话
   * @param sessionId 会话ID
   * @param onSessionDeleted 删除后回调（可选，用于清理当前会话状态）
   */
  const deleteSession = async (
    sessionId: string,
    onSessionDeleted?: (deletedSessionId: string) => void
  ) => {
    if (!sessionId) {return;}

    // 简单确认对话框
    if (!confirm('确定要删除该会话吗？')) {return;}

    try {
      await invoke('agent_delete_session', { sessionId });
      sessions.value = sessions.value.filter(s => s.id !== sessionId);

      // 如果删除的是当前会话，触发回调
      if (currentSessionId.value === sessionId) {
        if (onSessionDeleted) {
          onSessionDeleted(sessionId);
        } else {
          // 默认行为：清空当前会话状态
          startNewChat();
        }
      }
    } catch (e) {
      console.error('Delete error:', e);
    }
  };

  /**
   * 删除当前会话
   * @param onSessionDeleted 删除后回调
   */
  const deleteCurrentSession = async (onSessionDeleted?: () => void) => {
    if (!currentSessionId.value) {return;}

    if (!confirm('确定要删除当前会话吗？此操作不可撤销。')) {return;}

    try {
      await invoke('agent_delete_session', { sessionId: currentSessionId.value });
      sessions.value = sessions.value.filter(s => s.id !== currentSessionId.value);
      startNewChat(onSessionDeleted);
    } catch (e) {
      console.error('Delete error:', e);
    }
  };

  // ===== 搜索功能 =====

  /**
   * 搜索会话（内部方法）
   * @param query 搜索关键词
   */
  const searchSessions = async (query: string) => {
    const trimmedQuery = query.trim();
    if (!trimmedQuery) {
      searchResults.value = [];
      return;
    }

    isSearching.value = true;
    try {
      const result = await invoke<{ results: SearchResult[]; total: number; query: string }>(
        'agent_search_sessions',
        { query: trimmedQuery, limit: 20 }
      );
      searchResults.value = result.results;
    } catch (e) {
      console.error('Search failed:', e);
      searchResults.value = [];
    } finally {
      isSearching.value = false;
    }
  };

  /**
   * 处理搜索输入（带防抖）
   * @param query 搜索关键词
   */
  const handleSessionSearch = (query: string) => {
    if (searchDebounceTimer) {
      clearTimeout(searchDebounceTimer);
    }
    if (query.trim()) {
      searchDebounceTimer = window.setTimeout(() => {
        searchSessions(query);
      }, 300);
    } else {
      searchResults.value = [];
    }
  };

  /**
   * 清空搜索结果
   */
  const clearSessionSearch = () => {
    searchResults.value = [];
    if (searchDebounceTimer) {
      clearTimeout(searchDebounceTimer);
      searchDebounceTimer = null;
    }
  };

  /**
   * 跳转到搜索结果对应的会话
   * @param result 搜索结果
   * @param onLoadMessages 消息加载回调
   */
  const jumpToSearchResult = async (
    result: SearchResult,
    onLoadMessages?: (params: MessageLoadParams) => Promise<void> | void
  ) => {
    // 清空搜索，回到正常模式
    clearSessionSearch();

    // 查找会话是否在列表中
    const session = sessions.value.find(s => s.id === result.sessionId);
    if (session) {
      await selectSession(session, onLoadMessages);
    } else {
      // 会话不在列表中，需要加载
      try {
        const sessionResult = await invoke<{ sessionId: string; messages: any[] }>('agent_get_session', {
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
        await selectSession(tempSession, onLoadMessages);
      } catch (e) {
        console.error('Failed to load session:', e);
      }
    }
  };

  // ===== Git 仓库管理 =====

  /**
   * 加载 Git 仓库列表
   */
  const loadGitRepos = async () => {
    try {
      // 动态导入 tauri-api 以避免循环依赖
      const { getTauriAPI } = await import('@/utils/tauri-api');
      const api = getTauriAPI();
      const res = await api.getGitRepos();
      gitRepos.value = res?.data || [];
    } catch (e) {
      console.error('加载 Git 仓库列表失败:', e);
      gitRepos.value = [];
    }
  };

  // ===== 辅助函数 =====

  /**
   * 获取来源图标
   * @param source 来源类型
   */
  const sourceIcon = (source: string): string => {
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

  /**
   * 高亮搜索关键词
   * FTS5 已经用 >>>...<<< 标记匹配位置，转换为 <mark> 标签
   * @param snippet 原始片段
   * @param query 搜索关键词（可选，主要用于转义处理）
   */
  const highlightSnippet = (snippet: string, query?: string): string => {
    // FTS5 already marks matches with >>>...<<<
    // Convert to <mark> tags
    return snippet
      .replace(/>>>/g, '<mark class="bg-warning/30 text-warning px-0.5 rounded">')
      .replace(/<<</g, '</mark>');
  };

  /**
   * 生成会话标题（基于第一条消息）
   * @param firstMessage 第一条消息内容
   */
  const generateSessionTitle = (firstMessage: string): string => {
    let title = firstMessage.trim().slice(0, 30);
    if (firstMessage.trim().length > 30) {
      title += '...';
    }
    return title;
  };

  /**
   * 重命名会话
   * @param sessionId 会话ID
   * @param newTitle 新标题
   */
  const renameSession = async (sessionId: string, newTitle: string) => {
    if (!sessionId || !newTitle.trim()) {return;}

    try {
      await invoke('agent_rename_session', {
        sessionId,
        title: newTitle.trim(),
      });

      // 更新本地状态
      if (currentSession.value && currentSession.value.id === sessionId) {
        currentSession.value.title = newTitle.trim();
      }

      // 更新会话列表中的标题
      const session = sessions.value.find(s => s.id === sessionId);
      if (session) {
        session.title = newTitle.trim();
      }
    } catch (e) {
      console.error('Rename error:', e);
      throw e;
    }
  };

  /**
   * 检查 Hermes 是否可用
   */
  const checkHermesAvailable = async (): Promise<boolean> => {
    try {
      const result = await invoke<{ available: boolean; error: string | null }>('agent_check_available');
      return result.available;
    } catch (e) {
      return false;
    }
  };

  // ===== 返回状态和方法 =====
  return {
    // 状态
    sessions,
    searchResults,
    isSearching,
    currentSessionId,
    currentSession,
    loadingSessions,
    gitRepos,

    // 会话管理方法
    refreshSessions,
    selectSession,
    startNewChat,
    deleteSession,
    deleteCurrentSession,
    renameSession,
    generateSessionTitle,

    // 搜索方法
    handleSessionSearch,
    clearSessionSearch,
    jumpToSearchResult,

    // Git 仓库
    loadGitRepos,

    // 辅助函数
    sourceIcon,
    highlightSnippet,
    checkHermesAvailable,
  };
}

// 导出类型
export type SessionManagerReturn = ReturnType<typeof useSessionManager>;