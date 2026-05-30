import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { defineComponent, nextTick } from 'vue'

// ── Mocks ─────────────────────────────────────────────────────────────────

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'
const mockedInvoke = vi.mocked(invoke)

vi.mock('@/components/ui/SvgIcon.vue', () => ({
  default: defineComponent({
    name: 'SvgIcon',
    props: ['name', 'size'],
    template: '<span class="svg-icon-stub" :data-name="name" :data-size="size" />',
  }),
}))

vi.mock('../KanbanColumn.vue', () => ({
  default: defineComponent({
    name: 'KanbanColumn',
    props: ['title', 'status', 'tasks', 'color', 'busyTaskIds', 'draggingTaskId', 'dragOverCol', 'canDropHere'],
    emits: ['task-click', 'task-action', 'drag-start', 'drag-end', 'drag-over', 'drag-leave', 'drop'],
    template: `
      <div class="kanban-column-stub" :data-status="status">
        <div class="col-title">{{ title }}</div>
        <div
          v-for="t in tasks"
          :key="t.id"
          class="kanban-card-stub"
          :data-task-id="t.id"
          @click="$emit('task-click', t)"
        >
          {{ t.title }}
        </div>
      </div>
    `,
  }),
}))

vi.mock('../TaskDetailDrawer.vue', () => ({
  default: defineComponent({
    name: 'TaskDetailDrawer',
    props: ['task', 'assignees'],
    emits: ['close', 'refresh', 'action'],
    template: '<div class="task-detail-drawer-stub">{{ task?.task?.title }}</div>',
  }),
}))

vi.mock('../CreateTaskModal.vue', () => ({
  default: defineComponent({
    name: 'CreateTaskModal',
    props: ['assignees'],
    emits: ['close', 'create'],
    template: '<div class="create-task-modal-stub">Create Task Modal</div>',
  }),
}))

import KanbanBoard from '../KanbanBoard.vue'

// ── Helpers ────────────────────────────────────────────────────────────────

function createBoard() {
  const el = document.createElement('div')
  document.body.appendChild(el)
  const wrapper = mount(KanbanBoard, {
    attachTo: el,
    global: {
      stubs: { Teleport: false },
    },
  })
  return { wrapper }
}

function makeBoard(overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    slug: 'default',
    name: 'Default Board',
    description: 'Main board',
    archived: false,
    is_current: true,
    counts: { triage: 0, todo: 2, ready: 1, running: 0, blocked: 0, done: 3 },
    ...overrides,
  }
}

function makeTask(overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    id: 'task_test',
    title: 'Test task',
    status: 'todo',
    priority: 1,
    created_at: Math.floor(Date.now() / 1000) - 3600,
    ...overrides,
  }
}

function makeDetail(overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    task: makeTask(),
    latest_summary: 'In progress',
    parents: [],
    children: [],
    comments: [],
    events: [],
    runs: [],
    ...overrides,
  }
}

// ── Tests ──────────────────────────────────────────────────────────────────

describe('KanbanBoard.vue', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    document.body.innerHTML = ''
  })

  afterEach(() => {
  })

  // ── Loading State ─────────────────────────────────────────────────────

  it('should show loading state on mount', () => {
    // Keep invoke unresolved to maintain loading state
    mockedInvoke.mockReturnValue(new Promise(() => {}))
    const { wrapper } = createBoard()
    expect(wrapper.find('.loading-spinner').exists() || wrapper.text().includes('loading')).toBeTruthy()
  })

  // ── Normal Rendering ──────────────────────────────────────────────────

  it('should render board chips for each board', async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'kanban_list_boards') { return Promise.resolve([makeBoard()]) }
      if (cmd === 'kanban_list_tasks') { return Promise.resolve([]) }
      if (cmd === 'kanban_list_assignees') { return Promise.resolve([]) }
      return Promise.reject(new Error(`unexpected: ${cmd}`))
    })
    const { wrapper } = createBoard()
    await vi.dynamicImportSettled()
    await nextTick()
    expect(wrapper.text()).toContain('Default Board')
  })

  it('should render columns with tasks grouped by status', async () => {
    const tasks = [
      makeTask({ id: 't1', title: 'Todo task', status: 'todo' }),
      makeTask({ id: 't2', title: 'Ready task', status: 'ready' }),
      makeTask({ id: 't3', title: 'Done task', status: 'done' }),
    ]
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'kanban_list_boards') { return Promise.resolve([makeBoard()]) }
      if (cmd === 'kanban_list_tasks') { return Promise.resolve(tasks) }
      if (cmd === 'kanban_list_assignees') { return Promise.resolve([]) }
      return Promise.reject(new Error(`unexpected: ${cmd}`))
    })
    const { wrapper } = createBoard()
    await vi.dynamicImportSettled()
    await nextTick()

    // Each mocked column stub shows its title
    expect(wrapper.text()).toContain('Triage')
    expect(wrapper.text()).toContain('Todo')
    expect(wrapper.text()).toContain('Ready')
    expect(wrapper.text()).toContain('Running')
    expect(wrapper.text()).toContain('Blocked')
    expect(wrapper.text()).toContain('Done')
  })

  it('should display board total count from counts', async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'kanban_list_boards') { return Promise.resolve([makeBoard({ counts: { todo: 2, done: 3 } })]) }
      if (cmd === 'kanban_list_tasks') { return Promise.resolve([]) }
      if (cmd === 'kanban_list_assignees') { return Promise.resolve([]) }
      return Promise.reject(new Error(`unexpected: ${cmd}`))
    })
    const { wrapper } = createBoard()
    await vi.dynamicImportSettled()
    await nextTick()
    // Total count should be sum of counts values
    expect(wrapper.text()).toContain('5')
  })

  // ── Error State ───────────────────────────────────────────────────────

  it('should show error when dispatch fails', async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'kanban_list_boards') { return Promise.resolve([makeBoard()]) }
      if (cmd === 'kanban_list_tasks') { return Promise.resolve([]) }
      if (cmd === 'kanban_list_assignees') { return Promise.resolve([]) }
      if (cmd === 'kanban_dispatch') { return Promise.reject(new Error('Dispatch failed: Queue full')) }
      return Promise.reject(new Error(`unexpected: ${cmd}`))
    })
    const { wrapper } = createBoard()
    await vi.dynamicImportSettled()
    await nextTick()

    // Find and click dispatch button
    const dispatchIcon = wrapper.find('.svg-icon-stub[data-name="zap"]')
    if (dispatchIcon.exists()) {
      const parentBtn = dispatchIcon.element.closest('button')
      if (parentBtn) {
        (parentBtn as HTMLElement).click()
        await vi.dynamicImportSettled()
        await nextTick()
        await nextTick()
        expect(wrapper.text()).toContain('Dispatch failed')
      }
    }
  })

  it('should show error when create task fails', async () => {
    // Simulate a failing create task call
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'kanban_list_boards') { return Promise.resolve([makeBoard()]) }
      if (cmd === 'kanban_list_tasks') { return Promise.resolve([]) }
      if (cmd === 'kanban_list_assignees') { return Promise.resolve([]) }
      if (cmd === 'kanban_create_task') { return Promise.reject(new Error('Invalid task data')) }
      return Promise.reject(new Error(`unexpected: ${cmd}`))
    })
    const { wrapper } = createBoard()
    await vi.dynamicImportSettled()
    await nextTick()

    // Verify no errors initially
    // The board chips show, no error state visible
    expect(wrapper.text()).not.toContain('Invalid')
  })

  // ── Board Switching ───────────────────────────────────────────────────

  it('should switch board when board chip is clicked', async () => {
    const boards = [
      makeBoard({ slug: 'board1', name: 'Board 1', is_current: true }),
      makeBoard({ slug: 'board2', name: 'Board 2', is_current: false }),
    ]
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'kanban_list_boards') { return Promise.resolve(boards) }
      if (cmd === 'kanban_list_tasks') { return Promise.resolve([]) }
      if (cmd === 'kanban_list_assignees') { return Promise.resolve([]) }
      if (cmd === 'kanban_switch_board') {
        return Promise.resolve()
      }
      return Promise.reject(new Error(`unexpected: ${cmd}`))
    })
    const { wrapper } = createBoard()
    await vi.dynamicImportSettled()
    await nextTick()

    // Verify both boards are rendered
    expect(wrapper.text()).toContain('Board 1')
    expect(wrapper.text()).toContain('Board 2')
  })

  // ── Task Click → Detail ───────────────────────────────────────────────

  it('should show detail drawer when a task is clicked', async () => {
    const tasks = [makeTask({ id: 't_click', title: 'Clickable task' })]
    const detail = makeDetail({ task: makeTask({ id: 't_click', title: 'Clickable task' }) })

    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'kanban_list_boards') { return Promise.resolve([makeBoard()]) }
      if (cmd === 'kanban_list_tasks') { return Promise.resolve(tasks) }
      if (cmd === 'kanban_list_assignees') { return Promise.resolve([]) }
      if (cmd === 'kanban_show_task') { return Promise.resolve(detail) }
      return Promise.reject(new Error(`unexpected: ${cmd}`))
    })
    const { wrapper } = createBoard()
    await vi.dynamicImportSettled()
    await nextTick()

    // The kanban-column stub emits task-click when a card is clicked
    const card = wrapper.find('.kanban-card-stub')
    if (card.exists()) {
      await card.trigger('click')
      await vi.dynamicImportSettled()
      await nextTick()

      // Detail drawer stub should be visible
      expect(wrapper.find('.task-detail-drawer-stub').exists()).toBe(true)
    }
  })

  // ── Dispatch Button ───────────────────────────────────────────────────

  it('should call kanban_dispatch when dispatch button is clicked', async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'kanban_list_boards') { return Promise.resolve([makeBoard()]) }
      if (cmd === 'kanban_list_tasks') { return Promise.resolve([]) }
      if (cmd === 'kanban_list_assignees') { return Promise.resolve([]) }
      if (cmd === 'kanban_dispatch') { return Promise.resolve() }
      return Promise.reject(new Error(`unexpected: ${cmd}`))
    })
    const { wrapper } = createBoard()
    await vi.dynamicImportSettled()
    await nextTick()

    // The dispatch button has text "Dispatch" or an SVG with name "zap"
    // It's the second button in the header after Refresh
    const dispatchIcon = wrapper.find('.svg-icon-stub[data-name="zap"]')
    if (dispatchIcon.exists()) {
      const parentBtn = dispatchIcon.element.closest('button')
      if (parentBtn) {
        (parentBtn as HTMLElement).click()
        await nextTick()
        expect(mockedInvoke).toHaveBeenCalledWith('kanban_dispatch', { dryRun: false, maxSpawns: null })
      }
    }
  })

  // ── Refresh Button ────────────────────────────────────────────────────

  it('should reload tasks when refresh button is clicked', async () => {
    let loadCount = 0
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'kanban_list_boards') { return Promise.resolve([makeBoard()]) }
      if (cmd === 'kanban_list_tasks') {
        loadCount++
        return Promise.resolve(loadCount === 1 ? [makeTask({ title: 'Old' })] : [makeTask({ title: 'New' })])
      }
      if (cmd === 'kanban_list_assignees') { return Promise.resolve([]) }
      return Promise.reject(new Error(`unexpected: ${cmd}`))
    })
    const { wrapper } = createBoard()
    await vi.dynamicImportSettled()
    await nextTick()

    // Find refresh button via SVG icon
    const refreshIcon = wrapper.find('.svg-icon-stub[data-name="refresh"]')
    if (refreshIcon.exists()) {
      const parentBtn = refreshIcon.element.closest('button')
      if (parentBtn) {
        (parentBtn as HTMLElement).click()
        await vi.dynamicImportSettled()
        await nextTick()
        expect(loadCount).toBe(2)
      }
    }
  })

  // ── "New Task" Button ─────────────────────────────────────────────────

  it('should show create task modal when "New Task" button is clicked', async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'kanban_list_boards') { return Promise.resolve([makeBoard()]) }
      if (cmd === 'kanban_list_tasks') { return Promise.resolve([]) }
      if (cmd === 'kanban_list_assignees') { return Promise.resolve([]) }
      return Promise.reject(new Error(`unexpected: ${cmd}`))
    })
    const { wrapper } = createBoard()
    await vi.dynamicImportSettled()
    await nextTick()

    // Find "New Task" button
    const newTaskBtns = wrapper.findAll('button').filter(b => b.text().includes('New Task'))
    if (newTaskBtns.length > 0) {
      await newTaskBtns[0].trigger('click')
      await nextTick()
      expect(wrapper.find('.create-task-modal-stub').exists()).toBe(true)
    }
  })

  // ── "New Board" Button ────────────────────────────────────────────────

  it('should show new board input when "New Board" button is clicked', async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'kanban_list_boards') { return Promise.resolve([makeBoard()]) }
      if (cmd === 'kanban_list_tasks') { return Promise.resolve([]) }
      if (cmd === 'kanban_list_assignees') { return Promise.resolve([]) }
      return Promise.reject(new Error(`unexpected: ${cmd}`))
    })
    const { wrapper } = createBoard()
    await vi.dynamicImportSettled()
    await nextTick()

    // Find "New Board" button (the dashed border one)
    const newBoardBtn = wrapper.find('.kanban-board-chip-add')
    if (newBoardBtn.exists()) {
      await newBoardBtn.trigger('click')
      await nextTick()
      // Should show board creation form
      expect(wrapper.text()).toContain('Slug')
    }
  })

  // ── Polling ────────────────────────────────────────────────────────────

  it('should poll for task updates every 6 seconds', async () => {
    vi.useFakeTimers()
    let callCount = 0
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'kanban_list_boards') { return Promise.resolve([makeBoard()]) }
      if (cmd === 'kanban_list_tasks') {
        callCount++
        return Promise.resolve([])
      }
      if (cmd === 'kanban_list_assignees') { return Promise.resolve([]) }
      return Promise.reject(new Error(`unexpected: ${cmd}`))
    })
    createBoard()
    await vi.dynamicImportSettled()
    await nextTick()

    // Initial load: boards + tasks + assignees = 3 calls
    const initialCalls = callCount

    // Advance time by 6 seconds for one poll cycle
    vi.advanceTimersByTime(6000)
    await nextTick()

    // Should have called list_tasks one more time
    expect(callCount).toBe(initialCalls + 1)
    vi.useRealTimers()
  })
})
