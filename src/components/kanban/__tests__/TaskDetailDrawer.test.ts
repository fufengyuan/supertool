import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { defineComponent } from 'vue'

// ── Mocks ─────────────────────────────────────────────────────────────────

vi.mock('@/components/ui/SvgIcon.vue', () => ({
  default: defineComponent({
    name: 'SvgIcon',
    props: ['name', 'size'],
    template: '<span class="svg-icon-stub" :data-name="name" :data-size="size" />',
  }),
}))

vi.mock('../LogContent.vue', () => ({
  default: defineComponent({
    name: 'LogContent',
    props: ['log'],
    template: '<div class="log-content-stub">{{ log }}</div>',
  }),
}))

import TaskDetailDrawer from '../TaskDetailDrawer.vue'

// ── Fixtures ───────────────────────────────────────────────────────────────

function makeDetail(overrides: Record<string, unknown> = {}) {
  return {
    task: {
      id: 'task_test123',
      title: 'Test task detail',
      status: 'todo',
      assignee: 'test-user',
      priority: 5,
      body: 'A detailed description',
      created_at: Math.floor(Date.now() / 1000) - 3600,
      started_at: undefined,
      completed_at: undefined,
      tenant: 'default',
    },
    latest_summary: 'Recent progress made',
    parents: ['t_parent_1'],
    children: ['t_child_1'],
    comments: [
      { id: 1, author: 'alice', body: 'First comment', created_at: Math.floor(Date.now() / 1000) - 1800 },
    ],
    events: [
      { kind: 'created', payload: {}, created_at: Math.floor(Date.now() / 1000) - 3600, run_id: undefined },
    ],
    runs: [
      { id: 1, profile: 'worker', outcome: 'completed', summary: 'Done', started_at: Math.floor(Date.now() / 1000) - 3000, ended_at: Math.floor(Date.now() / 1000) - 500 },
    ],
    ...overrides,
  }
}

function createDrawer(props: Record<string, unknown> = {}) {
  return mount(TaskDetailDrawer, {
    props: {
      task: makeDetail(),
      assignees: [{ name: 'test-user', on_disk: true, counts: { todo: 3 } }],
      ...props,
    },
    global: {
      stubs: { Teleport: false },
    },
  })
}

// ── Tests ──────────────────────────────────────────────────────────────────

describe('TaskDetailDrawer.vue', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  // ── Basic Rendering ────────────────────────────────────────────────────

  it('should render task title', () => {
    const wrapper = createDrawer()
    expect(wrapper.text()).toContain('Test task detail')
  })

  it('should render status badge', () => {
    const wrapper = createDrawer()
    expect(wrapper.text()).toContain('Todo')
  })

  it('should render assignee', () => {
    const wrapper = createDrawer()
    expect(wrapper.text()).toContain('@test-user')
  })

  it('should render tenant when present', () => {
    const wrapper = createDrawer()
    expect(wrapper.text()).toContain('default')
  })

  it('should render short task ID (first 8 chars)', () => {
    const wrapper = createDrawer()
    // task.id = 'task_test123', slice(0,8) → 'task_tes'
    expect(wrapper.text()).toContain('task_tes')
  })

  it('should render description when body is present', () => {
    const wrapper = createDrawer()
    expect(wrapper.text()).toContain('A detailed description')
  })

  it('should render latest summary when present', () => {
    const wrapper = createDrawer()
    expect(wrapper.text()).toContain('Recent progress made')
  })

  it('should render priority label', () => {
    const wrapper = createDrawer()
    expect(wrapper.text()).toContain('Priority')
    expect(wrapper.text()).toContain('P1')
  })

  it('should not render priority section when priority is undefined', () => {
    const detail = makeDetail()
    const task = detail.task as Record<string, unknown>
    task.priority = undefined
    const wrapper = createDrawer({ task: detail })
    expect(wrapper.text()).not.toContain('Priority')
  })

  // ── Schedule Display ──────────────────────────────────────────────────

  it('should show "Created X ago" for created_at', () => {
    const wrapper = createDrawer()
    expect(wrapper.text()).toMatch(/Created.*ago/)
  })

  it('should show running duration for running tasks', () => {
    const now = Math.floor(Date.now() / 1000)
    const detail = makeDetail()
    const task = detail.task as Record<string, unknown>
    task.status = 'running'
    task.started_at = now - 120
    const wrapper = createDrawer({ task: detail })
    expect(wrapper.text()).toMatch(/Running/)
  })

  it('should show completed time for completed tasks', () => {
    const now = Math.floor(Date.now() / 1000)
    const detail = makeDetail()
    const task = detail.task as Record<string, unknown>
    task.completed_at = now - 600
    const wrapper = createDrawer({ task: detail })
    expect(wrapper.text()).toMatch(/Completed/)
  })

  // ── Status Color Classes ──────────────────────────────────────────────

  it('should apply correct status color for triage', () => {
    const detail = makeDetail()
    detail.task.status = 'triage'
    const wrapper = createDrawer({ task: detail })
    const dot = wrapper.find('.w-2\\.5')
    expect(dot.classes().join(' ')).toContain('bg-secondary')
  })

  it('should apply correct status color for done', () => {
    const detail = makeDetail()
    detail.task.status = 'done'
    const wrapper = createDrawer({ task: detail })
    const dot = wrapper.find('.w-2\\.5')
    expect(dot.classes().join(' ')).toContain('bg-success')
  })

  it('should apply correct status color for blocked', () => {
    const detail = makeDetail()
    detail.task.status = 'blocked'
    const wrapper = createDrawer({ task: detail })
    const dot = wrapper.find('.w-2\\.5')
    expect(dot.classes().join(' ')).toContain('bg-error')
  })

  // ── Assign Selector ───────────────────────────────────────────────────

  it('should render assignee select with options', () => {
    const wrapper = createDrawer()
    expect(wrapper.find('select').exists()).toBe(true)
    expect(wrapper.text()).toContain('Unassigned')
  })

  it('should have current assignee selected', () => {
    const wrapper = createDrawer()
    const select = wrapper.find('select')
    expect((select.element as HTMLSelectElement).value).toBe('test-user')
  })

  // ── Run History ───────────────────────────────────────────────────────

  it('should render run history entries', () => {
    const wrapper = createDrawer()
    expect(wrapper.text()).toContain('worker')
    expect(wrapper.text()).toContain('completed')
  })

  // ── Comments ──────────────────────────────────────────────────────────

  it('should render comments', () => {
    const wrapper = createDrawer()
    expect(wrapper.text()).toContain('alice')
    expect(wrapper.text()).toContain('First comment')
  })

  it('should have comment input field', () => {
    const wrapper = createDrawer()
    // Look for a textarea or input where user types comments
    const textareas = wrapper.findAll('textarea')
    const inputs = wrapper.findAll('input[type="text"]')
    expect(textareas.length + inputs.length).toBeGreaterThan(0)
  })

  // ── Emits ─────────────────────────────────────────────────────────────

  it('should emit close when close button is clicked', async () => {
    const wrapper = createDrawer()
    // Find the close button (X icon)
    const closeBtn = wrapper.find('.btn-circle')
    if (!closeBtn.exists()) {
      // Try finding any visible button with just SVG icon (no text)
      const allBtns = wrapper.findAll('button')
      for (const btn of allBtns) {
        if (btn.text().trim() === '') {
          await btn.trigger('click')
          break
        }
      }
    } else {
      await closeBtn.trigger('click')
    }
    expect(wrapper.emitted('close')).toBeTruthy()
  })

  // ── Edge Cases ─────────────────────────────────────────────────────────

  it('should handle empty comments list', () => {
    const detail = makeDetail()
    detail.comments = []
    const wrapper = createDrawer({ task: detail })
    expect(wrapper.text()).not.toContain('First comment')
  })

  it('should handle empty runs list', () => {
    const detail = makeDetail()
    detail.runs = []
    const wrapper = createDrawer({ task: detail })
    expect(wrapper.text()).not.toContain('completed')
  })

  it('should handle missing description', () => {
    const detail = makeDetail() as Record<string, unknown>
    const task = detail.task as Record<string, unknown>
    task.body = undefined
    const wrapper = createDrawer({ task: detail })
    expect(wrapper.text()).not.toContain('Description')
  })

  it('should handle missing latest_summary', () => {
    const detail = makeDetail() as Record<string, unknown>
    detail.latest_summary = undefined
    const wrapper = createDrawer({ task: detail })
    expect(wrapper.text()).not.toContain('Latest Summary')
  })

  it('should handle missing child tasks', () => {
    const detail = makeDetail()
    detail.children = []
    const wrapper = createDrawer({ task: detail })
    expect(wrapper.text()).not.toContain('Children')
  })
})
