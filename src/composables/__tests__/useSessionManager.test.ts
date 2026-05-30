import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'

// ── Mocks ─────────────────────────────────────────────────────────────────

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'
const mockedInvoke = vi.mocked(invoke)

// Mock the dynamic import used by loadGitRepos
const mockGetGitRepos = vi.fn()
vi.mock('@/utils/tauri-api', () => ({
  getTauriAPI: () => ({
    getGitRepos: mockGetGitRepos,
  }),
}))

import { useSessionManager, type Session, type SearchResult } from '../useSessionManager'

// ── Fixtures ────────────────────────────────────────────────────────────────

function makeSession(overrides: Partial<Session> = {}): Session {
  return {
    id: 'session_test_001',
    title: 'Test Session',
    model: 'anthropic/claude-sonnet-4',
    source: 'chat',
    startedAt: Math.floor(Date.now() / 1000) - 3600,
    endedAt: null,
    messageCount: 5,
    preview: 'This is a preview',
    lastActive: Math.floor(Date.now() / 1000) - 600,
    parentSessionId: null,
    profile: 'default',
    ...overrides,
  }
}

function makeSearchResult(overrides: Partial<SearchResult> = {}): SearchResult {
  return {
    sessionId: 's1',
    sessionTitle: 'Found session',
    messageId: 'm1',
    role: 'user',
    snippet: 'relevant >>>content<<< here',
    content: null,
    timestamp: Math.floor(Date.now() / 1000),
    source: 'chat',
    model: 'gpt-4',
    profile: 'default',
    ...overrides,
  }
}

// ── Tests ────────────────────────────────────────────────────────────────────

describe('useSessionManager()', () => {
  let sm: ReturnType<typeof useSessionManager>

  beforeEach(() => {
    vi.clearAllMocks()
    // happy-dom does not implement window.confirm — stub via globalThis
    globalThis.confirm = vi.fn(() => true)
    vi.useFakeTimers()
    sm = useSessionManager()
  })

  afterEach(() => {
    vi.useRealTimers()
    vi.restoreAllMocks()
  })

  // ── Initial State ───────────────────────────────────────────────────────
  describe('initial state', () => {
    it('should start with empty sessions', () => {
      expect(sm.sessions.value).toEqual([])
    })

    it('should start with empty search results', () => {
      expect(sm.searchResults.value).toEqual([])
    })

    it('should start not searching', () => {
      expect(sm.isSearching.value).toBe(false)
    })

    it('should start with null currentSessionId', () => {
      expect(sm.currentSessionId.value).toBeNull()
    })

    it('should start with null currentSession', () => {
      expect(sm.currentSession.value).toBeNull()
    })

    it('should start not loading', () => {
      expect(sm.loadingSessions.value).toBe(false)
    })

    it('should start with empty gitRepos', () => {
      expect(sm.gitRepos.value).toEqual([])
    })
  })

  // ── refreshSessions ────────────────────────────────────────────────────
  describe('refreshSessions()', () => {
    it('should load sessions sorted by lastActive descending', async () => {
      const old = makeSession({ id: 's_old', lastActive: 100 })
      const recent = makeSession({ id: 's_recent', lastActive: 300 })
      mockedInvoke.mockResolvedValue({ sessions: [old, recent], total: 2 })

      await sm.refreshSessions()

      expect(sm.sessions.value).toHaveLength(2)
      expect(sm.sessions.value[0].id).toBe('s_recent')
      expect(sm.sessions.value[1].id).toBe('s_old')
    })

    it('should set loadingSessions to false after load', async () => {
      mockedInvoke.mockResolvedValue({ sessions: [], total: 0 })

      const promise = sm.refreshSessions()
      expect(sm.loadingSessions.value).toBe(true)
      await promise
      expect(sm.loadingSessions.value).toBe(false)
    })

    it('should handle empty sessions gracefully', async () => {
      mockedInvoke.mockResolvedValue({ sessions: [], total: 0 })

      await sm.refreshSessions()

      expect(sm.sessions.value).toEqual([])
    })

    it('should handle invoke error gracefully', async () => {
      mockedInvoke.mockRejectedValue(new Error('Network error'))

      await sm.refreshSessions()

      expect(sm.sessions.value).toEqual([])
      expect(sm.loadingSessions.value).toBe(false)
    })

    it('should use startedAt as fallback sort when lastActive is undefined', async () => {
      const old = makeSession({ id: 's_old', lastActive: undefined, startedAt: 100 })
      const recent = makeSession({ id: 's_recent', lastActive: undefined, startedAt: 300 })
      mockedInvoke.mockResolvedValue({ sessions: [old, recent], total: 2 })

      await sm.refreshSessions()

      expect(sm.sessions.value[0].id).toBe('s_recent')
      expect(sm.sessions.value[1].id).toBe('s_old')
    })

    it('should sort sessions with 0 timestamps to the end', async () => {
      const withTime = makeSession({ id: 's_time', lastActive: 200, startedAt: 200 })
      const noTime = makeSession({ id: 's_notime', lastActive: undefined, startedAt: undefined })
      mockedInvoke.mockResolvedValue({ sessions: [noTime, withTime], total: 2 })

      await sm.refreshSessions()

      expect(sm.sessions.value[0].id).toBe('s_time')
      expect(sm.sessions.value[1].id).toBe('s_notime')
    })
  })

  // ── selectSession ──────────────────────────────────────────────────────
  describe('selectSession()', () => {
    it('should set currentSessionId and currentSession', async () => {
      const session = makeSession()
      mockedInvoke.mockImplementation((cmd: string) => {
        if (cmd === 'agent_get_compression_tip') {
          return Promise.resolve({ success: false, tipSessionId: session.id, originalSessionId: session.id })
        }
        return Promise.reject(new Error(`unexpected: ${cmd}`))
      })

      await sm.selectSession(session)

      expect(sm.currentSessionId.value).toBe(session.id)
      expect(sm.currentSession.value).toEqual(session)
    })

    it('should follow compression tip when tipSessionId differs', async () => {
      const session = makeSession({ id: 'original_id' })
      mockedInvoke.mockImplementation((cmd: string) => {
        if (cmd === 'agent_get_compression_tip') {
          return Promise.resolve({ success: true, tipSessionId: 'compressed_id', originalSessionId: 'original_id' })
        }
        if (cmd === 'agent_list_messages') {
          return Promise.resolve({ success: true, messages: [], sessionId: 'compressed_id' })
        }
        return Promise.reject(new Error(`unexpected: ${cmd}`))
      })

      await sm.selectSession(session)

      expect(sm.currentSessionId.value).toBe('compressed_id')
    })

    it('should call onLoadMessages when provided', async () => {
      const session = makeSession()
      const onLoad = vi.fn()
      mockedInvoke.mockImplementation((cmd: string) => {
        if (cmd === 'agent_get_compression_tip') {
          return Promise.resolve({ success: false, tipSessionId: session.id, originalSessionId: session.id })
        }
        if (cmd === 'agent_list_messages') {
          return Promise.resolve({ success: true, messages: [{ id: 'm1', content: 'hello' }], sessionId: session.id })
        }
        return Promise.reject(new Error(`unexpected: ${cmd}`))
      })

      await sm.selectSession(session, onLoad)

      expect(onLoad).toHaveBeenCalledWith({
        sessionId: session.id,
        messages: [{ id: 'm1', content: 'hello' }],
      })
    })

    it('should handle compression tip errors silently', async () => {
      const session = makeSession()
      mockedInvoke.mockImplementation((cmd: string) => {
        if (cmd === 'agent_get_compression_tip') {
          return Promise.reject(new Error('No compression'))
        }
        return Promise.reject(new Error(`unexpected: ${cmd}`))
      })

      await sm.selectSession(session)

      // Falls back to original session id
      expect(sm.currentSessionId.value).toBe(session.id)
    })

    it('should handle message loading errors gracefully', async () => {
      const session = makeSession()
      const onLoad = vi.fn()
      mockedInvoke.mockImplementation((cmd: string) => {
        if (cmd === 'agent_get_compression_tip') {
          return Promise.resolve({ success: false, tipSessionId: session.id, originalSessionId: session.id })
        }
        if (cmd === 'agent_list_messages') {
          return Promise.reject(new Error('Failed to load'))
        }
        return Promise.reject(new Error(`unexpected: ${cmd}`))
      })

      await sm.selectSession(session, onLoad)

      // current state still set
      expect(sm.currentSessionId.value).toBe(session.id)
      // onLoad not called on error
      expect(onLoad).not.toHaveBeenCalled()
    })
  })

  // ── startNewChat ───────────────────────────────────────────────────────
  describe('startNewChat()', () => {
    it('should clear current session state', () => {
      // Set some state first
      const session = makeSession()
      sm.currentSessionId.value = session.id
      sm.currentSession.value = session

      sm.startNewChat()

      expect(sm.currentSessionId.value).toBeNull()
      expect(sm.currentSession.value).toBeNull()
    })

    it('should call onClear callback when provided', () => {
      const onClear = vi.fn()

      sm.startNewChat(onClear)

      expect(onClear).toHaveBeenCalledOnce()
    })
  })

  // ── deleteSession ──────────────────────────────────────────────────────
  describe('deleteSession()', () => {
    it('should return early if sessionId is empty', async () => {
      await sm.deleteSession('')
      expect(mockedInvoke).not.toHaveBeenCalled()
    })

    it('should return early if confirm is cancelled', async () => {
      // confirmFn returns false → delete should be aborted
      await sm.deleteSession('session_test_001', undefined, async () => false)

      expect(mockedInvoke).not.toHaveBeenCalled()
    })

    it('should delete session and remove from list', async () => {
      const session = makeSession({ id: 's_del' })
      mockedInvoke.mockResolvedValue(undefined)
      sm.sessions.value = [session]

      await sm.deleteSession('s_del')

      expect(mockedInvoke).toHaveBeenCalledWith('agent_delete_session', { sessionId: 's_del' })
      expect(sm.sessions.value).toEqual([])
    })

    it('should call onSessionDeleted when deleting current session', async () => {
      const session = makeSession({ id: 's_current' })
      mockedInvoke.mockResolvedValue(undefined)
      sm.sessions.value = [session]
      sm.currentSessionId.value = 's_current'
      const onDeleted = vi.fn()

      await sm.deleteSession('s_current', onDeleted)

      expect(onDeleted).toHaveBeenCalledWith('s_current')
    })

    it('should call startNewChat as default when no callback and deleting current', async () => {
      const session = makeSession({ id: 's_current' })
      mockedInvoke.mockResolvedValue(undefined)
      sm.sessions.value = [session]
      sm.currentSessionId.value = 's_current'
      sm.currentSession.value = session

      await sm.deleteSession('s_current')

      expect(sm.currentSessionId.value).toBeNull()
    })

    it('should handle delete error gracefully', async () => {
      mockedInvoke.mockRejectedValue(new Error('Delete failed'))
      sm.sessions.value = [makeSession({ id: 's_err' })]

      await sm.deleteSession('s_err')

      // Session should still be in the list
      expect(sm.sessions.value).toHaveLength(1)
    })
  })

  // ── deleteCurrentSession ───────────────────────────────────────────────
  describe('deleteCurrentSession()', () => {
    it('should return early if no current session', async () => {
      sm.currentSessionId.value = null

      await sm.deleteCurrentSession()

      expect(mockedInvoke).not.toHaveBeenCalled()
    })

    it('should return early if confirm is cancelled', async () => {
      // confirmFn returns false → delete should be aborted
      sm.currentSessionId.value = 's_id'

      await sm.deleteCurrentSession(undefined, async () => false)

      expect(mockedInvoke).not.toHaveBeenCalled()
    })

    it('should delete current session and start new chat', async () => {
      sm.currentSessionId.value = 's_current'
      sm.sessions.value = [makeSession({ id: 's_current' })]
      mockedInvoke.mockResolvedValue(undefined)

      await sm.deleteCurrentSession()

      expect(mockedInvoke).toHaveBeenCalledWith('agent_delete_session', { sessionId: 's_current' })
      expect(sm.currentSessionId.value).toBeNull()
    })

    it('should call onSessionDeleted callback when provided', async () => {
      sm.currentSessionId.value = 's_cb'
      mockedInvoke.mockResolvedValue(undefined)
      const onDeleted = vi.fn()

      await sm.deleteCurrentSession(onDeleted)

      expect(onDeleted).toHaveBeenCalledOnce()
    })
  })

  // ── searchSessions (internal) ───────────────────────────────────────────
  describe('searchSessions()', () => {
    it('should clear results for empty query via searchSessions', async () => {
      sm.searchResults.value = [makeSearchResult()]

      // Call handleSessionSearch with whitespace — clears immediately without debounce
      sm.handleSessionSearch('   ')

      expect(sm.searchResults.value).toEqual([])
    })

    it('should set search results on success', async () => {
      const results = [makeSearchResult()]
      mockedInvoke.mockResolvedValue({ results, total: 1, query: 'test' })

      // Trigger through handleSessionSearch with debounce
      sm.handleSessionSearch('test')
      // Advance time to trigger the debounced searchSessions call
      vi.advanceTimersByTime(300)
      // Wait for microtasks (the async searchSessions)
      await vi.waitFor(() => {
        expect(mockedInvoke).toHaveBeenCalled()
      })
    })

    it('should set isSearching during search', async () => {
      mockedInvoke.mockImplementation(() => new Promise(() => {})) // never resolves

      sm.handleSessionSearch('test')
      vi.advanceTimersByTime(300)
      // Wait for the macro task to execute the debounce callback
      await Promise.resolve()

      // isSearching should be true while loading
      expect(sm.isSearching.value).toBe(true)
    })

    it('should clear isSearching after search completes', async () => {
      mockedInvoke.mockResolvedValue({ results: [], total: 0, query: 'test' })

      sm.handleSessionSearch('test')
      vi.advanceTimersByTime(300)
      // Wait for the async searchSessions + finally to complete
      await vi.waitFor(() => {
        expect(sm.isSearching.value).toBe(false)
      })
    })

    it('should set empty results on search error', async () => {
      mockedInvoke.mockRejectedValue(new Error('Search failed'))

      sm.handleSessionSearch('test')
      vi.advanceTimersByTime(300)
      // Wait for the async searchSessions to catch and clear
      await vi.waitFor(() => {
        expect(sm.searchResults.value).toEqual([])
      })
    })
  })

  // ── handleSessionSearch ────────────────────────────────────────────────
  describe('handleSessionSearch()', () => {
    it('should clear results immediately for empty input', () => {
      sm.searchResults.value = [makeSearchResult()]

      sm.handleSessionSearch('')

      expect(sm.searchResults.value).toEqual([])
    })

    it('should debounce search calls', async () => {
      mockedInvoke.mockResolvedValue({ results: [], total: 0, query: 'test' })

      sm.handleSessionSearch('test')
      expect(mockedInvoke).not.toHaveBeenCalled() // debounced

      vi.advanceTimersByTime(200)
      expect(mockedInvoke).not.toHaveBeenCalled() // still within 300ms

      vi.advanceTimersByTime(200)
      // Should have fired now — wait for microtask
      await vi.waitFor(() => {
        expect(mockedInvoke).toHaveBeenCalledWith('agent_search_sessions', {
          query: 'test',
          limit: 20,
        })
      })
    })

    it('should cancel previous debounce on new call', async () => {
      mockedInvoke.mockResolvedValue({ results: [], total: 0, query: 'test' })

      sm.handleSessionSearch('first')
      sm.handleSessionSearch('second')
      vi.advanceTimersByTime(300)
      // Wait for microtask
      await vi.waitFor(() => {
        expect(mockedInvoke).toHaveBeenCalledTimes(1)
        expect(mockedInvoke).toHaveBeenCalledWith('agent_search_sessions', {
          query: 'second',
          limit: 20,
        })
      })
    })
  })

  // ── clearSessionSearch ─────────────────────────────────────────────────
  describe('clearSessionSearch()', () => {
    it('should clear search results', () => {
      sm.searchResults.value = [makeSearchResult()]

      sm.clearSessionSearch()

      expect(sm.searchResults.value).toEqual([])
    })
  })

  // ── jumpToSearchResult ─────────────────────────────────────────────────
  describe('jumpToSearchResult()', () => {
    it('should clear search results', async () => {
      sm.searchResults.value = [makeSearchResult()]
      mockedInvoke.mockResolvedValue({ success: false, tipSessionId: 's1', originalSessionId: 's1' })

      await sm.jumpToSearchResult(makeSearchResult())

      expect(sm.searchResults.value).toEqual([])
    })

    it('should select existing session from list', async () => {
      const session = makeSession({ id: 's1', title: 'Found in list' })
      sm.sessions.value = [session]
      mockedInvoke.mockImplementation((cmd: string) => {
        if (cmd === 'agent_get_compression_tip') {
          return Promise.resolve({ success: false, tipSessionId: 's1', originalSessionId: 's1' })
        }
        return Promise.reject(new Error(`unexpected: ${cmd}`))
      })

      await sm.jumpToSearchResult(makeSearchResult())

      expect(sm.currentSessionId.value).toBe('s1')
    })

    it('should load session when not in list', async () => {
      mockedInvoke.mockImplementation((cmd: string) => {
        if (cmd === 'agent_get_session') {
          return Promise.resolve({ sessionId: 's1', messages: [{ id: 'm1' }] })
        }
        if (cmd === 'agent_get_compression_tip') {
          return Promise.resolve({ success: false, tipSessionId: 's1', originalSessionId: 's1' })
        }
        return Promise.reject(new Error(`unexpected: ${cmd}`))
      })

      await sm.jumpToSearchResult(makeSearchResult())

      expect(sm.sessions.value).toHaveLength(1)
      expect(sm.sessions.value[0].id).toBe('s1')
    })

    it('should handle load-session error gracefully', async () => {
      mockedInvoke.mockImplementation((cmd: string) => {
        if (cmd === 'agent_get_session') {
          return Promise.reject(new Error('Session not found'))
        }
        return Promise.reject(new Error(`unexpected: ${cmd}`))
      })

      // Should not throw
      await expect(sm.jumpToSearchResult(makeSearchResult())).resolves.toBeUndefined()
    })
  })

  // ── renameSession ──────────────────────────────────────────────────────
  describe('renameSession()', () => {
    it('should return early if sessionId is empty', async () => {
      await sm.renameSession('', 'New Title')
      expect(mockedInvoke).not.toHaveBeenCalled()
    })

    it('should return early if newTitle is empty', async () => {
      await sm.renameSession('s_id', '')
      expect(mockedInvoke).not.toHaveBeenCalled()
    })

    it('should return early if newTitle is only whitespace', async () => {
      await sm.renameSession('s_id', '   ')
      expect(mockedInvoke).not.toHaveBeenCalled()
    })

    it('should invoke rename and update local state', async () => {
      const session = makeSession({ id: 's_rename', title: 'Old Title' })
      sm.currentSession.value = session
      sm.sessions.value = [session]
      mockedInvoke.mockResolvedValue(undefined)

      await sm.renameSession('s_rename', 'New Title')

      expect(mockedInvoke).toHaveBeenCalledWith('agent_rename_session', {
        sessionId: 's_rename',
        title: 'New Title',
      })
      expect(sm.currentSession.value!.title).toBe('New Title')
      expect(sm.sessions.value[0].title).toBe('New Title')
    })

    it('should throw on invoke error', async () => {
      mockedInvoke.mockRejectedValue(new Error('Rename failed'))

      await expect(sm.renameSession('s_id', 'New Title')).rejects.toThrow('Rename failed')
    })
  })

  // ── checkHermesAvailable ───────────────────────────────────────────────
  describe('checkHermesAvailable()', () => {
    it('should return true when available', async () => {
      mockedInvoke.mockResolvedValue({ available: true, error: null })

      const result = await sm.checkHermesAvailable()

      expect(result).toBe(true)
    })

    it('should return false when unavailable', async () => {
      mockedInvoke.mockResolvedValue({ available: false, error: 'Not running' })

      const result = await sm.checkHermesAvailable()

      expect(result).toBe(false)
    })

    it('should return false on error', async () => {
      mockedInvoke.mockRejectedValue(new Error('Connection refused'))

      const result = await sm.checkHermesAvailable()

      expect(result).toBe(false)
    })
  })

  // ── sourceIcon ─────────────────────────────────────────────────────────┬
  describe('sourceIcon()', () => {
    it('should return terminal for cli', () => {
      expect(sm.sourceIcon('cli')).toBe('terminal')
    })

    it('should return message for feishu', () => {
      expect(sm.sourceIcon('feishu')).toBe('message')
    })

    it('should return message for telegram', () => {
      expect(sm.sourceIcon('telegram')).toBe('message')
    })

    it('should return message for discord', () => {
      expect(sm.sourceIcon('discord')).toBe('message')
    })

    it('should return message for slack', () => {
      expect(sm.sourceIcon('slack')).toBe('message')
    })

    it('should return clock for cron', () => {
      expect(sm.sourceIcon('cron')).toBe('clock')
    })

    it('should return chat as default for unknown sources', () => {
      expect(sm.sourceIcon('unknown')).toBe('chat')
    })

    it('should return chat for empty string', () => {
      expect(sm.sourceIcon('')).toBe('chat')
    })
  })

  // ── highlightSnippet ───────────────────────────────────────────────────
  describe('highlightSnippet()', () => {
    it('should convert >>> to opening mark tag', () => {
      const result = sm.highlightSnippet('>>>content<<<')
      expect(result).toContain('<mark class="bg-warning/30 text-warning px-0.5 rounded">')
      expect(result).toContain('content')
    })

    it('should convert <<< to closing mark tag', () => {
      const result = sm.highlightSnippet('>>>content<<<')
      expect(result).toContain('</mark>')
    })

    it('should return unchanged string when no markers', () => {
      const result = sm.highlightSnippet('plain text content')
      expect(result).toBe('plain text content')
    })

    it('should handle empty string', () => {
      expect(sm.highlightSnippet('')).toBe('')
    })
  })

  // ── generateSessionTitle ───────────────────────────────────────────────
  describe('generateSessionTitle()', () => {
    it('should return trimmed title for short messages', () => {
      const title = sm.generateSessionTitle('  Hello World  ')
      expect(title).toBe('Hello World')
    })

    it('should truncate and add ellipsis for long messages', () => {
      const longMsg = 'This is a very long message that exceeds thirty characters limit'
      const title = sm.generateSessionTitle(longMsg)
      expect(title).toHaveLength(33) // 30 + '...'
      expect(title.endsWith('...')).toBe(true)
    })

    it('should handle empty string input', () => {
      expect(sm.generateSessionTitle('')).toBe('')
    })

    it('should handle whitespace-only input', () => {
      expect(sm.generateSessionTitle('   ')).toBe('')
    })
  })

  // ── loadGitRepos ───────────────────────────────────────────────────────
  describe('loadGitRepos()', () => {
    it('should load git repos from tauri-api', async () => {
      mockGetGitRepos.mockResolvedValue({ data: [{ name: 'test-repo', path: '/tmp/repo' }] })

      await sm.loadGitRepos()

      expect(sm.gitRepos.value).toEqual([{ name: 'test-repo', path: '/tmp/repo' }])
    })

    it('should handle empty git repo list', async () => {
      mockGetGitRepos.mockResolvedValue({ data: [] })

      await sm.loadGitRepos()

      expect(sm.gitRepos.value).toEqual([])
    })

    it('should handle missing data field', async () => {
      mockGetGitRepos.mockResolvedValue({})

      await sm.loadGitRepos()

      expect(sm.gitRepos.value).toEqual([])
    })

    it('should handle API error gracefully', async () => {
      mockGetGitRepos.mockRejectedValue(new Error('API error'))

      await sm.loadGitRepos()

      expect(sm.gitRepos.value).toEqual([])
    })
  })
})
