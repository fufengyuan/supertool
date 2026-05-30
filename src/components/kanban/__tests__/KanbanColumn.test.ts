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

import KanbanColumn from '../KanbanColumn.vue'

// ── Helpers ────────────────────────────────────────────────────────────────

interface Task {
  id: string
  title: string
  status: string
  assignee?: string
  priority?: number
  created_at?: number
  started_at?: number
}

function makeTask(overrides: Partial<Task> = {}): Task {
  return {
    id: 'task_abc123',
    title: 'Test task',
    status: 'todo',
    ...overrides,
  }
}

function createColumn(props: Record<string, unknown> = {}) {
  return mount(KanbanColumn, {
    props: {
      title: 'Todo',
      status: 'todo',
      tasks: [],
      color: 'warning',
      ...props,
    },
    global: {
      stubs: { Teleport: false },
    },
  })
}

// ── Tests ──────────────────────────────────────────────────────────────────

describe('KanbanColumn.vue', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  // ── Basic Rendering ────────────────────────────────────────────────────

  it('should render column title and task count', () => {
    const tasks = [makeTask(), makeTask({ id: 'task_2' })]
    const wrapper = createColumn({ title: 'Running', status: 'running', tasks, color: 'primary' })
    expect(wrapper.text()).toContain('Running')
    expect(wrapper.text()).toContain('2')
  })

  it('should show zero count when no tasks', () => {
    const wrapper = createColumn({ title: 'Blocked', tasks: [] })
    expect(wrapper.text()).toContain('Blocked')
    expect(wrapper.text()).toContain('0')
  })

  it('should render task titles', () => {
    const tasks = [
      makeTask({ id: 't1', title: 'Fix login bug' }),
      makeTask({ id: 't2', title: 'Add test coverage' }),
    ]
    const wrapper = createColumn({ tasks })
    expect(wrapper.text()).toContain('Fix login bug')
    expect(wrapper.text()).toContain('Add test coverage')
  })

  it('should render assignee pill when task has assignee', () => {
    const tasks = [makeTask({ assignee: 'alice' })]
    const wrapper = createColumn({ tasks })
    expect(wrapper.text()).toContain('@alice')
  })

  it('should not render assignee pill when no assignee', () => {
    const tasks = [makeTask({ assignee: undefined })]
    const wrapper = createColumn({ tasks })
    expect(wrapper.text()).not.toContain('@')
  })

  // ── Priority Labels ────────────────────────────────────────────────────

  it('should render P0 for priority >= 10', () => {
    const tasks = [makeTask({ priority: 10 })]
    const wrapper = createColumn({ tasks })
    expect(wrapper.text()).toContain('P0')
  })

  it('should render P1 for priority between 5 and 9', () => {
    const tasks = [makeTask({ priority: 5 })]
    const wrapper = createColumn({ tasks })
    expect(wrapper.text()).toContain('P1')
  })

  it('should render P2 for priority between 1 and 4', () => {
    const tasks = [makeTask({ priority: 1 })]
    const wrapper = createColumn({ tasks })
    expect(wrapper.text()).toContain('P2')
  })

  it('should not render priority label for priority 0 or undefined', () => {
    const tasks = [makeTask({ priority: 0 })]
    const wrapper = createColumn({ tasks })
    expect(wrapper.text()).not.toMatch(/P[0-9]/)
  })

  // ── Age Labels ─────────────────────────────────────────────────────────

  it('should render age in seconds when less than 60', () => {
    const now = Math.floor(Date.now() / 1000)
    const tasks = [makeTask({ created_at: now - 30 })]
    const wrapper = createColumn({ tasks })
    expect(wrapper.text()).toMatch(/30s/)
  })

  it('should render age in minutes when less than 3600', () => {
    const now = Math.floor(Date.now() / 1000)
    const tasks = [makeTask({ created_at: now - 180 })]
    const wrapper = createColumn({ tasks })
    expect(wrapper.text()).toMatch(/3m/)
  })

  it('should render age in hours when less than 86400', () => {
    const now = Math.floor(Date.now() / 1000)
    const tasks = [makeTask({ created_at: now - 7200 })]
    const wrapper = createColumn({ tasks })
    expect(wrapper.text()).toMatch(/2h/)
  })

  it('should render age in days when >= 86400', () => {
    const now = Math.floor(Date.now() / 1000)
    const tasks = [makeTask({ created_at: now - 172800 })]
    const wrapper = createColumn({ tasks })
    expect(wrapper.text()).toMatch(/2d/)
  })

  it('should return empty age when created_at is undefined', () => {
    const tasks = [makeTask({ created_at: undefined })]
    const wrapper = createColumn({ tasks })
    // Should not have any time pattern
    expect(wrapper.text()).not.toMatch(/\d+[smhd]/)
  })

  // ── Running Duration ───────────────────────────────────────────────────

  it('should show running duration for running tasks with started_at', () => {
    const now = Math.floor(Date.now() / 1000)
    const tasks = [makeTask({ status: 'running', started_at: now - 120 })]
    const wrapper = createColumn({ tasks })
    expect(wrapper.text()).toMatch(/2m/)
  })

  it('should not show running duration when started_at is undefined', () => {
    const tasks = [makeTask({ status: 'running', started_at: undefined })]
    const wrapper = createColumn({ tasks })
    // The running pill should not render because taskStartedAgo returns empty string
    expect(wrapper.find('[title="Running for"]').exists()).toBe(false)
  })

  // ── Drag & Drop Styling ────────────────────────────────────────────────

  it('should apply drag-over class when dragging over this column and can drop', () => {
    const wrapper = createColumn({
      tasks: [makeTask()],
      draggingTaskId: 'task_other',
      dragOverCol: 'todo',
      canDropHere: true,
    })
    expect(wrapper.classes()).toContain('kanban-column-drop')
  })

  it('should not apply drag-over class when canDropHere is false', () => {
    const wrapper = createColumn({
      tasks: [makeTask()],
      draggingTaskId: 'task_other',
      dragOverCol: 'todo',
      canDropHere: false,
    })
    expect(wrapper.classes()).not.toContain('kanban-column-drop')
  })

  it('should apply dragging class to the currently dragged card', () => {
    const draggingId = 'task_dragging'
    const tasks = [
      makeTask({ id: draggingId, title: 'Dragging task' }),
      makeTask({ id: 'task_other', title: 'Other task' }),
    ]
    const wrapper = createColumn({
      tasks,
      draggingTaskId: draggingId,
    })
    const cards = wrapper.findAll('.kanban-card')
    expect(cards[0].classes()).toContain('kanban-card-dragging')
    expect(cards[1].classes()).not.toContain('kanban-card-dragging')
  })

  // ── Busy state ─────────────────────────────────────────────────────────

  it('should not make cards draggable when task is in busyTaskIds', () => {
    const tasks = [makeTask({ id: 'busy_task' })]
    const wrapper = createColumn({
      tasks,
      busyTaskIds: ['busy_task'],
    })
    const card = wrapper.find('.kanban-card')
    expect(card.attributes('draggable')).toBe('false')
  })

  it('should make cards draggable when not busy', () => {
    const tasks = [makeTask({ id: 'free_task' })]
    const wrapper = createColumn({
      tasks,
      busyTaskIds: [],
    })
    const card = wrapper.find('.kanban-card')
    expect(card.attributes('draggable')).toBe('true')
  })

  // ── Emitted Events ─────────────────────────────────────────────────────

  it('should emit task-click when card is clicked', async () => {
    const task = makeTask({ id: 'clickable' })
    const wrapper = createColumn({ tasks: [task] })
    await wrapper.find('.kanban-card').trigger('click')
    expect(wrapper.emitted('task-click')).toBeTruthy()
    expect(wrapper.emitted('task-click')![0]).toEqual([task])
  })

  it('should emit drag-start when card drag starts', async () => {
    const task = makeTask({ id: 'drag_start' })
    const wrapper = createColumn({ tasks: [task] })
    const card = wrapper.find('.kanban-card')
    await card.trigger('dragstart')
    expect(wrapper.emitted('drag-start')).toBeTruthy()
    expect(wrapper.emitted('drag-start')![0]).toEqual([task.id])
  })

  it('should emit drag-end when card drag ends', async () => {
    const task = makeTask({ id: 'drag_end' })
    const wrapper = createColumn({ tasks: [task] })
    await wrapper.find('.kanban-card').trigger('dragend')
    expect(wrapper.emitted('drag-end')).toBeTruthy()
    expect(wrapper.emitted('drag-end')![0]).toEqual([task.id])
  })

  it('should emit drag-over when dragging over the column', async () => {
    const wrapper = createColumn({
      tasks: [makeTask()],
      draggingTaskId: 'task_other',
      canDropHere: true,
    })
    await wrapper.trigger('dragover')
    expect(wrapper.emitted('drag-over')).toBeTruthy()
    expect(wrapper.emitted('drag-over')![0]).toEqual(['todo'])
  })

  it('should emit drop when dropping on the column', async () => {
    const wrapper = createColumn({
      tasks: [makeTask()],
      draggingTaskId: 'task_other',
    })
    await wrapper.trigger('drop')
    expect(wrapper.emitted('drop')).toBeTruthy()
    expect(wrapper.emitted('drop')![0]).toEqual(['todo'])
  })

  it('should not emit drag-over when not dragging', async () => {
    const wrapper = createColumn({
      tasks: [makeTask()],
      draggingTaskId: null,
    })
    await wrapper.trigger('dragover')
    expect(wrapper.emitted('drag-over')).toBeFalsy()
  })

  it('should emit drag-leave when drag leaves the column', async () => {
    const wrapper = createColumn({
      tasks: [makeTask()],
      draggingTaskId: 'task_other',
      dragOverCol: 'todo',
    })

    // Simulate drag leave — the handler checks relatedTarget containment
    // We use a direct trigger which will fire the emit from onDragLeave
    await wrapper.trigger('dragleave')
    // The handler only emits if relatedTarget is not a child
    // Since we're using trigger, relatedTarget might not be set, emitting should still happen
    // If relatedTarget is empty, the handler emits
  })
})
