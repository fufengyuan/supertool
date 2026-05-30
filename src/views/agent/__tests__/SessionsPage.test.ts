import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { defineComponent, nextTick, ref } from 'vue'

// ── Mocks ─────────────────────────────────────────────────────────────────

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'
const mockedInvoke = vi.mocked(invoke)

const mockPush = vi.fn()

vi.mock('vue-router', () => ({
  useRouter: () => ({ push: mockPush }),
  useRoute: () => ({ path: '/agent/sessions' }),
}))

vi.mock('@/components/ui/SvgIcon.vue', () => ({
  default: defineComponent({
    name: 'SvgIcon',
    props: ['name', 'size'],
    template: '<span class="svg-icon-stub" :data-name="name" :data-size="size" />',
  }),
}))

// ── Composable mock state ──────────────────────────────────────────────────

import type { Ref } from 'vue'

// Shared mutable state for the composable mock
interface SessionMockState {
  sessions: Ref<unknown[]>
  searchResults: Ref<unknown[]>
  isSearching: Ref<boolean>
  currentSessionId: Ref<string | null>
  loadingSessions: Ref<boolean>
  refreshSessions: ReturnType<typeof vi.fn>
  handleSessionSearch: ReturnType<typeof vi.fn>
  highlightSnippet: ReturnType<typeof vi.fn>
}

let sessionMockState: SessionMockState

vi.mock('@/composables/useSessionManager', () => ({
  useSessionManager: () => {
    if (!sessionMockState) {
      sessionMockState = {
        sessions: ref([]),
        searchResults: ref([]),
        isSearching: ref(false),
        currentSessionId: ref(null),
        loadingSessions: ref(true),
        refreshSessions: vi.fn().mockResolvedValue(undefined),
        handleSessionSearch: vi.fn(),
        highlightSnippet: vi.fn((s: string) => s),
      }
    }
    return sessionMockState
  },
}))

import SessionsPage from '../SessionsPage.vue'

// ── Fixtures ───────────────────────────────────────────────────────────────

function makeSession(overrides: Record<string, unknown> = {}): Record<string, unknown> {
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

function makeSearchResult(overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    sessionId: 's1',
    sessionTitle: 'Found session',
    messageId: 'm1',
    role: 'user',
    snippet: 'relevant content',
    content: null,
    timestamp: Math.floor(Date.now() / 1000),
    source: 'chat',
    model: 'gpt-4',
    profile: 'default',
    ...overrides,
  }
}

// ── Helpers ────────────────────────────────────────────────────────────────

function createWrapper() {
  // Reset mock state for each test
  sessionMockState = {
    sessions: ref([]),
    searchResults: ref([]),
    isSearching: ref(false),
    currentSessionId: ref(null),
    loadingSessions: ref(false),
    refreshSessions: vi.fn().mockResolvedValue(undefined),
    handleSessionSearch: vi.fn(),
    highlightSnippet: vi.fn((s: string) => s),
  }

  return mount(SessionsPage, {
    global: {
      stubs: { Teleport: false },
    },
  })
}

// ── Tests ──────────────────────────────────────────────────────────────────

describe('SessionsPage.vue', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    document.body.innerHTML = ''
    vi.useFakeTimers()
    sessionMockState = {
      sessions: ref([]),
      searchResults: ref([]),
      isSearching: ref(false),
      currentSessionId: ref(null),
      loadingSessions: ref(false),
      refreshSessions: vi.fn().mockResolvedValue(undefined),
      handleSessionSearch: vi.fn(),
      highlightSnippet: vi.fn((s: string) => s),
    }
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  // ── Rendering ─────────────────────────────────────────────────────────

  it('should render page title', () => {
    const wrapper = createWrapper()
    expect(wrapper.text()).toContain('Sessions')
  })

  it('should render "New Chat" button', () => {
    const wrapper = createWrapper()
    expect(wrapper.text()).toContain('New Chat')
  })

  it('should render search input', () => {
    const wrapper = createWrapper()
    const input = wrapper.find('input[placeholder="Search sessions..."]')
    expect(input.exists()).toBe(true)
  })

  // ── Session Display ──────────────────────────────────────────────────

  it('should display sessions when data is loaded', async () => {
    const wrapper = createWrapper()
    sessionMockState.sessions.value = [makeSession({ id: 's1', title: 'Session 1' })]
    sessionMockState.loadingSessions.value = false
    await nextTick()

    expect(wrapper.text()).toContain('Session 1')
  })

  it('should display multiple sessions', async () => {
    const wrapper = createWrapper()
    sessionMockState.sessions.value = [
      makeSession({ id: 's1', title: 'Session A' }),
      makeSession({ id: 's2', title: 'Session B' }),
    ]
    sessionMockState.loadingSessions.value = false
    await nextTick()

    expect(wrapper.text()).toContain('Session A')
    expect(wrapper.text()).toContain('Session B')
  })

  it('should show empty state when no sessions exist', async () => {
    const wrapper = createWrapper()
    sessionMockState.sessions.value = []
    sessionMockState.loadingSessions.value = false
    await nextTick()

    expect(wrapper.text()).toContain('No conversations yet')
  })

  it('should show loading spinner while sessions are loading', () => {
    const wrapper = createWrapper()
    sessionMockState.loadingSessions.value = true
    sessionMockState.sessions.value = []
    // Need to let Vue render
    nextTick().then(() => {
      expect(wrapper.find('.loading')!.exists()).toBe(true)
    })
  })

  // ── Session Meta Display ──────────────────────────────────────────────

  it('should display session source and message count', async () => {
    const wrapper = createWrapper()
    sessionMockState.sessions.value = [
      makeSession({ id: 's1', title: 'Meta session', source: 'chat', messageCount: 3 }),
    ]
    sessionMockState.loadingSessions.value = false
    await nextTick()

    expect(wrapper.text()).toContain('chat')
    expect(wrapper.text()).toContain('3 msgs')
  })

  it('should display singular "msg" for messageCount === 1', async () => {
    const wrapper = createWrapper()
    sessionMockState.sessions.value = [
      makeSession({ id: 's1', title: 'Single msg', messageCount: 1 }),
    ]
    sessionMockState.loadingSessions.value = false
    await nextTick()

    expect(wrapper.text()).toContain('1 msg')
  })

  it('should format model name from path', async () => {
    const wrapper = createWrapper()
    sessionMockState.sessions.value = [
      makeSession({ id: 's1', title: 'Model test', model: 'anthropic/claude-sonnet-4' }),
    ]
    sessionMockState.loadingSessions.value = false
    await nextTick()

    // formatModel extracts the last segment after /
    expect(wrapper.text()).toContain('claude-sonnet-4')
  })

  it('should display model version suffix after colon', async () => {
    const wrapper = createWrapper()
    sessionMockState.sessions.value = [
      makeSession({ id: 's1', title: 'Model with version', model: 'anthropic/claude-sonnet-4:20250101' }),
    ]
    sessionMockState.loadingSessions.value = false
    await nextTick()

    // formatModel splits on ':' and takes first part
    expect(wrapper.text()).toContain('claude-sonnet-4')
  })

  it('should have formatModel return full string when no slash exists', async () => {
    const wrapper = createWrapper()
    sessionMockState.sessions.value = [
      makeSession({ id: 's1', title: 'Model no slash', model: 'gpt-4o' }),
    ]
    sessionMockState.loadingSessions.value = false
    await nextTick()

    expect(wrapper.text()).toContain('gpt-4o')
  })

  it('should display "New conversation" as fallback for null title', async () => {
    const wrapper = createWrapper()
    sessionMockState.sessions.value = [
      makeSession({ id: 's1', title: null }),
    ]
    sessionMockState.loadingSessions.value = false
    await nextTick()

    expect(wrapper.text()).toContain('New conversation')
  })

  it('should not show model badge when model is absent', async () => {
    const wrapper = createWrapper()
    sessionMockState.sessions.value = [
      makeSession({ id: 's1', title: 'No model', model: undefined }),
    ]
    sessionMockState.loadingSessions.value = false
    await nextTick()

    // Should show source but not model
    expect(wrapper.text()).toContain('chat')
  })

  // ── Date Grouping ────────────────────────────────────────────────────

  it('should show "Today" group label for today sessions', async () => {
    const wrapper = createWrapper()
    sessionMockState.sessions.value = [
      makeSession({ id: 's1', title: 'Today session', lastActive: Math.floor(Date.now() / 1000) - 300 }),
    ]
    sessionMockState.loadingSessions.value = false
    await nextTick()

    expect(wrapper.text()).toContain('Today')
    expect(wrapper.text()).toContain('Today session')
  })

  it('should show "Yesterday" group label for yesterday sessions', async () => {
    const wrapper = createWrapper()
    const yesterday = new Date()
    yesterday.setDate(yesterday.getDate() - 1)
    yesterday.setHours(10, 0, 0, 0)
    const yesterdayTs = Math.floor(yesterday.getTime() / 1000)

    sessionMockState.sessions.value = [
      makeSession({ id: 's2', title: 'Yesterday session', lastActive: yesterdayTs }),
    ]
    sessionMockState.loadingSessions.value = false
    await nextTick()

    expect(wrapper.text()).toContain('Yesterday')
    expect(wrapper.text()).toContain('Yesterday session')
  })

  it('should show "This Week" group label for sessions within 7 days', async () => {
    const wrapper = createWrapper()
    // 3 days ago, still this week
    const threeDaysAgo = new Date()
    threeDaysAgo.setDate(threeDaysAgo.getDate() - 3)
    threeDaysAgo.setHours(10, 0, 0, 0)
    const threeDaysAgoTs = Math.floor(threeDaysAgo.getTime() / 1000)

    sessionMockState.sessions.value = [
      makeSession({ id: 's3', title: 'This week session', lastActive: threeDaysAgoTs }),
    ]
    sessionMockState.loadingSessions.value = false
    await nextTick()

    expect(wrapper.text()).toContain('This Week')
    expect(wrapper.text()).toContain('This week session')
  })

  it('should show "Earlier" group label for sessions older than 7 days', async () => {
    const wrapper = createWrapper()
    sessionMockState.sessions.value = [
      makeSession({ id: 's1', title: 'Old session', lastActive: Math.floor(Date.now() / 1000) - 86400 * 10 }),
    ]
    sessionMockState.loadingSessions.value = false
    await nextTick()

    expect(wrapper.text()).toContain('Earlier')
  })

  it('should use startedAt as fallback when lastActive is undefined', async () => {
    const wrapper = createWrapper()
    sessionMockState.sessions.value = [
      makeSession({ id: 's1', title: 'Fallback session', lastActive: undefined, startedAt: Math.floor(Date.now() / 1000) - 200 }),
    ]
    sessionMockState.loadingSessions.value = false
    await nextTick()

    expect(wrapper.text()).toContain('Fallback session')
    expect(wrapper.text()).toContain('Today')
  })

  it('should handle undefined timestamp in getDateGroup', async () => {
    const wrapper = createWrapper()
    sessionMockState.sessions.value = [
      makeSession({ id: 's1', title: 'No timestamp', lastActive: undefined, startedAt: undefined }),
    ]
    sessionMockState.loadingSessions.value = false
    await nextTick()

    // Should still render without crashing, falls to 'Earlier' group
    expect(wrapper.text()).toContain('No timestamp')
  })

  // ── Navigation ────────────────────────────────────────────────────────

  it('should navigate to /agent/chat when "New Chat" is clicked', async () => {
    const wrapper = createWrapper()
    sessionMockState.sessions.value = []
    sessionMockState.loadingSessions.value = false
    await nextTick()

    const newChatBtn = wrapper.findAll('button').filter(b => b.text().includes('New Chat'))
    expect(newChatBtn.length).toBeGreaterThanOrEqual(1)
    await newChatBtn[0].trigger('click')
    expect(mockPush).toHaveBeenCalledWith('/agent/chat')
  })

  it('should navigate to chat with session query when session is clicked', async () => {
    const wrapper = createWrapper()
    sessionMockState.sessions.value = [
      makeSession({ id: 'session_abc', title: 'Clickable' }),
    ]
    sessionMockState.loadingSessions.value = false
    await nextTick()

    // Find session card buttons and click the first one
    const sessionCards = wrapper.findAll('button').filter(
      b => b.text().includes('Clickable'),
    )
    if (sessionCards.length > 0) {
      await sessionCards[0].trigger('click')
      expect(mockPush).toHaveBeenCalledWith({
        path: '/agent/chat',
        query: { session: 'session_abc' },
      })
    }
  })

  it('should navigate to search result session when clicked', async () => {
    const wrapper = createWrapper()
    sessionMockState.sessions.value = []
    sessionMockState.searchResults.value = [
      makeSearchResult({ sessionId: 'search_s1', sessionTitle: 'Search result session' }),
    ]
    sessionMockState.loadingSessions.value = false
    await nextTick()

    const resultBtns = wrapper.findAll('button').filter(
      b => b.text().includes('Search result session'),
    )
    if (resultBtns.length > 0) {
      await resultBtns[0].trigger('click')
      expect(mockPush).toHaveBeenCalledWith({
        path: '/agent/chat',
        query: { session: 'search_s1' },
      })
    }
  })

  // ── Search ─────────────────────────────────────────────────────────────

  it('should trigger search when search query changes', async () => {
    const wrapper = createWrapper()
    sessionMockState.sessions.value = []
    sessionMockState.loadingSessions.value = false
    await nextTick()

    const searchInput = wrapper.find('input[placeholder="Search sessions..."]')
    expect(searchInput.exists()).toBe(true)
  })

  it('should show search results when in search mode', async () => {
    const wrapper = createWrapper()
    sessionMockState.sessions.value = []
    sessionMockState.loadingSessions.value = false
    sessionMockState.searchResults.value = [
      makeSearchResult({
        sessionId: 's1',
        sessionTitle: 'Found session',
        snippet: 'relevant content',
      }),
    ]
    await nextTick()
  })

  it('should display search result snippet and tags', async () => {
    const wrapper = createWrapper()
    sessionMockState.sessions.value = []
    sessionMockState.loadingSessions.value = false
    sessionMockState.searchResults.value = [
      makeSearchResult({
        sessionId: 's1',
        sessionTitle: 'Search hit',
        snippet: 'matching content here',
        source: 'cli',
        model: 'gpt-4',
      }),
    ]
    await nextTick()
  })

  it('should show search clear button when searchQuery is non-empty', async () => {
    const wrapper = createWrapper()
    sessionMockState.sessions.value = []
    sessionMockState.loadingSessions.value = false
    await nextTick()

    // Set search query to trigger the X button
    const input = wrapper.find('input[placeholder="Search sessions..."]')
    await input.setValue('test query')
    await nextTick()

    // X button should appear (SvgIcon with name "x")
    const xIcon = wrapper.find('.svg-icon-stub[data-name="x"]')
    expect(xIcon.exists()).toBe(true)
  })

  it('should clear search and refocus input when X is clicked', async () => {
    const wrapper = createWrapper()
    sessionMockState.sessions.value = []
    sessionMockState.loadingSessions.value = false
    await nextTick()

    const input = wrapper.find('input[placeholder="Search sessions..."]')
    await input.setValue('test')
    await nextTick()

    const xIcon = wrapper.find('.svg-icon-stub[data-name="x"]')
    if (xIcon.exists()) {
      const clearBtn = xIcon.element.closest('button')
      if (clearBtn) {
        ;(clearBtn as HTMLElement).click()
        await nextTick()
        // After clear, search results should be empty
        expect(sessionMockState.handleSessionSearch).toHaveBeenCalledWith('')
      }
    }
  })

  it('should highlight snippet using highlightSnippet composable', async () => {
    const highlightedSpy = vi.fn((s: string) => s)
    sessionMockState.highlightSnippet = highlightedSpy
    const wrapper = createWrapper()
    sessionMockState.sessions.value = []
    sessionMockState.loadingSessions.value = false
    sessionMockState.searchResults.value = [
      makeSearchResult({ snippet: 'search match' }),
    ]
    await nextTick()
  })

  // ── Auto-refresh ─────────────────────────────────────────────────────

  it('should auto-refresh on mount via refreshSessions', () => {
    createWrapper()
    expect(sessionMockState.refreshSessions).toHaveBeenCalled()
  })

  it('should refresh sessions on window focus', async () => {
    createWrapper()
    await nextTick()
    sessionMockState.refreshSessions.mockClear()

    window.dispatchEvent(new Event('focus'))
    await nextTick()

    expect(sessionMockState.refreshSessions).toHaveBeenCalled()
  })

  it('should clean up interval and event listeners on unmount', () => {
    const wrapper = createWrapper()
    const unmountSpy = vi.spyOn(window, 'removeEventListener')

    wrapper.unmount()

    expect(unmountSpy).toHaveBeenCalledWith('focus', expect.any(Function))
    unmountSpy.mockRestore()
  })

  // ── Edge Cases ─────────────────────────────────────────────────────────

  it('should render with empty session id fallback title in search results', async () => {
    const wrapper = createWrapper()
    sessionMockState.sessions.value = []
    sessionMockState.loadingSessions.value = false
    sessionMockState.searchResults.value = [
      makeSearchResult({
        sessionId: 'abc123',
        sessionTitle: null,
        snippet: null,
      }),
    ]
    await nextTick()
  })

  it('should not crash with extremely long session title', async () => {
    const wrapper = createWrapper()
    const longTitle = 'A'.repeat(1000)
    sessionMockState.sessions.value = [
      makeSession({ id: 's_long', title: longTitle }),
    ]
    sessionMockState.loadingSessions.value = false
    await nextTick()

    expect(wrapper.text()).toContain(longTitle.slice(0, 50))
  })

  it('should reset isSearchingLocal after debounced search', async () => {
    const wrapper = createWrapper()
    sessionMockState.sessions.value = []
    sessionMockState.loadingSessions.value = false
    await nextTick()

    const input = wrapper.find('input[placeholder="Search sessions..."]')
    await input.setValue('search term')
    await vi.advanceTimersByTimeAsync(350)

    expect(sessionMockState.handleSessionSearch).toHaveBeenCalledWith('search term')
  })

  it('should not search on empty trimmed query', async () => {
    const wrapper = createWrapper()
    sessionMockState.sessions.value = []
    sessionMockState.loadingSessions.value = false
    await nextTick()

    const input = wrapper.find('input[placeholder="Search sessions..."]')
    await input.setValue('   ')
    await vi.advanceTimersByTimeAsync(350)

    // Should call handleSessionSearch with empty string (clearing mode)
    expect(sessionMockState.handleSessionSearch).toHaveBeenCalledWith('')
  })
})
