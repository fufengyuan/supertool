// @vitest-environment happy-dom
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { h, defineComponent } from 'vue'
import CronManager from '../CronManager.vue'
import type { CronJob } from '@/types'

// --- Mock tauri-api ---
const mockListCronJobs = vi.fn<() => Promise<CronJob[]>>()
const mockCreateCronJob = vi.fn<
  (schedule: string, prompt?: string, name?: string, deliver?: string) => Promise<void>
>()
const mockRemoveCronJob = vi.fn<(jobId: string) => Promise<void>>()
const mockPauseCronJob = vi.fn<(jobId: string) => Promise<void>>()
const mockResumeCronJob = vi.fn<(jobId: string) => Promise<void>>()
const mockTriggerCronJob = vi.fn<(jobId: string) => Promise<void>>()

vi.mock('@/utils/tauri-api', () => ({
  getTauriAPI: () => ({
    listCronJobs: mockListCronJobs,
    createCronJob: mockCreateCronJob,
    removeCronJob: mockRemoveCronJob,
    pauseCronJob: mockPauseCronJob,
    resumeCronJob: mockResumeCronJob,
    triggerCronJob: mockTriggerCronJob,
  }),
}))

// --- Helper ---
function flushPromises() {
  return new Promise(resolve => setTimeout(resolve, 0))
}

// --- Modal stub ---
const ModalStub = defineComponent({
  props: ['modelValue', 'title', 'width', 'maxHeight', 'showClose'],
  emits: ['update:modelValue'],
  setup(props, { slots }) {
    return () =>
      props.modelValue
        ? h('div', { class: 'modal-stub' }, [
            props.title ? h('h3', { class: 'modal-title' }, props.title) : null,
            slots.default?.(),
            slots.footer?.(),
          ])
        : null
  },
})

// --- Stubs ---
const STUBS = {
  SvgIcon: true,
  Modal: ModalStub,
  IconClock: true,
  IconPlus: true,
  IconTrash: true,
  IconPlayerPlay: true,
  IconPlayerPause: true,
  IconCalendarEvent: true,
  IconAlertCircle: true,
  IconCircleCheck: true,
}

// --- Sample data ---
function sampleJobs(): CronJob[] {
  return [
    {
      id: 'job-1',
      name: 'Daily Report',
      prompt: 'Send daily summary report',
      schedule: '0 9 * * *',
      state: 'active',
      enabled: true,
      next_run_at: '2026-05-30T09:00:00Z',
      last_run_at: '2026-05-29T09:00:00Z',
      last_status: 'success',
      last_error: null,
      deliver: 'telegram',
      skills: [],
      script: null,
    },
    {
      id: 'job-2',
      name: 'Weekly Backup',
      prompt: 'Backup data to S3',
      schedule: '0 0 * * 0',
      state: 'paused',
      enabled: false,
      next_run_at: null,
      last_run_at: null,
      last_status: null,
      last_error: null,
      deliver: 'local',
      skills: [],
      script: null,
    },
    {
      id: 'job-3',
      name: '',
      prompt: '',
      schedule: 'once in 30m',
      state: 'completed',
      enabled: false,
      next_run_at: null,
      last_run_at: '2026-05-29T10:00:00Z',
      last_status: 'completed',
      last_error: 'Disk full',
      deliver: 'local',
      skills: [],
      script: null,
    },
  ]
}

// =====================================
// Tests
// =====================================
describe('CronManager.vue', () => {
  beforeEach(() => {
    mockListCronJobs.mockReset()
    mockCreateCronJob.mockReset()
    mockRemoveCronJob.mockReset()
    mockPauseCronJob.mockReset()
    mockResumeCronJob.mockReset()
    mockTriggerCronJob.mockReset()
    // Default: return empty array so mount doesn't crash
    mockListCronJobs.mockResolvedValue([])
  })

  // ============ Load & Initial State ============
  describe('initial load', () => {
    it('should call listCronJobs on mount', async () => {
      mockListCronJobs.mockResolvedValue([])
      mount(CronManager, { global: { stubs: STUBS } })
      expect(mockListCronJobs).toHaveBeenCalledTimes(1)
    })

    it('should show loading spinner when loading and no jobs', async () => {
      // Keep the promise pending so loading stays true
      const delayed = new Promise<CronJob[]>(() => {})
      mockListCronJobs.mockReturnValue(delayed)

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      // Wait for onMounted to set loading=true
      await wrapper.vm.$nextTick()
      expect(wrapper.text()).toContain('加载中...')
    })
  })

  // ============ Empty State ============
  describe('empty state', () => {
    it('should show empty state when jobs list is empty', async () => {
      mockListCronJobs.mockResolvedValue([])

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('暂无定时任务')
    })

    it('should show create button in empty state', async () => {
      mockListCronJobs.mockResolvedValue([])

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      const emptyCreateBtn = wrapper.findAll('button').find(b => b.text().includes('新建定时任务'))
      expect(emptyCreateBtn).toBeTruthy()
    })
  })

  // ============ Job List Rendering ============
  describe('job list rendering', () => {
    it('should render all jobs with state badges', async () => {
      mockListCronJobs.mockResolvedValue(sampleJobs())

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('Daily Report')
      expect(wrapper.text()).toContain('Weekly Backup')
      // job-3 has empty name, so id should show
      expect(wrapper.text()).toContain('job-3')
    })

    it('should show state labels correctly', async () => {
      mockListCronJobs.mockResolvedValue(sampleJobs())

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('运行中')
      expect(wrapper.text()).toContain('已暂停')
      expect(wrapper.text()).toContain('completed') // raw state for unknown
    })

    it('should show schedule display text', async () => {
      mockListCronJobs.mockResolvedValue(sampleJobs())

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('0 9 * * *')
      expect(wrapper.text()).toContain('0 0 * * 0')
      expect(wrapper.text()).toContain('once in 30m')
    })

    it('should show prompt preview text', async () => {
      mockListCronJobs.mockResolvedValue(sampleJobs())

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('Send daily summary report')
      expect(wrapper.text()).toContain('Backup data to S3')
    })

    it('should show timestamps when available', async () => {
      mockListCronJobs.mockResolvedValue(sampleJobs())

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      // last_run_at dates should be formatted (month/day hour:minute)
      // 2026-05-29T09:00:00Z → local time (CST UTC+8 = 17:00)
      expect(wrapper.text()).toContain('05/29')
      expect(wrapper.text()).toMatch(/\d{2}:\d{2}/) // any hour:minute
    })

    it('should show last error text', async () => {
      mockListCronJobs.mockResolvedValue(sampleJobs())

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('Disk full')
    })

    it('should apply opacity-60 to paused/completed jobs', async () => {
      mockListCronJobs.mockResolvedValue(sampleJobs())

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      const cards = wrapper.findAll('.bg-base-100.rounded-lg')
      // job-3 (completed) should have opacity-60
      const completedCard = cards.find(c => c.text().includes('job-3'))
      expect(completedCard?.classes()).toContain('opacity-60')

      // job-1 (active) should NOT have opacity-60
      const activeCard = cards.find(c => c.text().includes('Daily Report'))
      expect(activeCard?.classes()).not.toContain('opacity-60')
    })
  })

  // ============ Action Buttons ============
  describe('action buttons', () => {
    it('should show trigger button for all jobs', async () => {
      mockListCronJobs.mockResolvedValue(sampleJobs())

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      // Each job card should have an "立即执行" button
      const playButtons = wrapper.findAll('button[title="立即执行"]')
      expect(playButtons.length).toBeGreaterThanOrEqual(3)
    })

    it('should show pause button for active jobs', async () => {
      mockListCronJobs.mockResolvedValue(sampleJobs())

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      const pauseButtons = wrapper.findAll('button[title="暂停"]')
      expect(pauseButtons.length).toBe(1) // Only job-1 (active)
    })

    it('should show resume button for paused/completed jobs', async () => {
      mockListCronJobs.mockResolvedValue(sampleJobs())

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      const resumeButtons = wrapper.findAll('button[title="恢复"]')
      expect(resumeButtons.length).toBe(2) // job-2 (paused) and job-3 (completed)
    })

    it('should show delete button for all jobs', async () => {
      mockListCronJobs.mockResolvedValue(sampleJobs())

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      const deleteButtons = wrapper.findAll('button[title="删除"]')
      expect(deleteButtons.length).toBe(3)
    })
  })

  // ============ CRUD Operations ============
  describe('create flow', () => {
    it('should open create modal when clicking 新建任务', async () => {
      mockListCronJobs.mockResolvedValue([])

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      // Initially no modal
      expect(wrapper.find('.modal-stub').exists()).toBe(false)

      // Click "新建任务" button in header
      const headerBtn = wrapper.findAll('button').find(b => b.text().includes('新建任务'))
      await headerBtn!.trigger('click')

      await wrapper.vm.$nextTick()
      expect(wrapper.find('.modal-stub').exists()).toBe(true)
      expect(wrapper.text()).toContain('新建定时任务')
    })

    it('should have create button disabled when schedule is empty', async () => {
      mockListCronJobs.mockResolvedValue([])

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      // Open modal
      await (wrapper.vm as any).openCreateModal()
      await wrapper.vm.$nextTick()

      expect(wrapper.find('.modal-stub').exists()).toBe(true)

      // Find create button in modal footer — should be disabled
      const createBtn = wrapper.findAll('button').find(b => b.text() === '创建')
      expect(createBtn?.attributes('disabled')).toBeDefined()
    })

    it('should enable create button when schedule is filled', async () => {
      mockListCronJobs.mockResolvedValue([])

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      // Open modal and set schedule via input
      await (wrapper.vm as any).openCreateModal()
      await wrapper.vm.$nextTick()

      // Find schedule input and type
      const scheduleInput = wrapper.find('input[placeholder*="e.g."]')
      await scheduleInput.setValue('every 2h')
      await wrapper.vm.$nextTick()

      const createBtn = wrapper.findAll('button').find(b => b.text() === '创建')
      expect(createBtn?.attributes('disabled')).toBeUndefined()
    })

    it('should call createCronJob with form values and refresh list', async () => {
      mockListCronJobs.mockResolvedValue([])
      mockCreateCronJob.mockResolvedValue(undefined)

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      // Reset list count after mount
      mockListCronJobs.mockReset()
      mockListCronJobs.mockResolvedValue([])

      // Open modal and fill form
      await (wrapper.vm as any).openCreateModal()
      await wrapper.vm.$nextTick()

      await wrapper.find('input[placeholder*="e.g."]').setValue('0 9 * * *')
      await wrapper.find('input[placeholder*="可选，留空自动生成"]').setValue('My Job')
      await wrapper.find('textarea').setValue('Do something')
      await wrapper.find('input[placeholder*="可选，如 telegram"]').setValue('discord:#general')

      // Click create
      const createBtn = wrapper.findAll('button').find(b => b.text() === '创建')
      await createBtn!.trigger('click')

      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(mockCreateCronJob).toHaveBeenCalledWith(
        '0 9 * * *',
        'Do something',
        'My Job',
        'discord:#general',
      )
      // Should refresh the list after creating
      expect(mockListCronJobs).toHaveBeenCalled()
      // Modal should close
      expect(wrapper.find('.modal-stub').exists()).toBe(false)
    })

    it('should call createCronJob with only schedule when other fields are empty', async () => {
      mockListCronJobs.mockResolvedValue([])
      mockCreateCronJob.mockResolvedValue(undefined)

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      mockListCronJobs.mockReset()
      mockListCronJobs.mockResolvedValue([])

      await (wrapper.vm as any).openCreateModal()
      await wrapper.vm.$nextTick()

      await wrapper.find('input[placeholder*="e.g."]').setValue('once in 30m')
      await wrapper.vm.$nextTick()

      const createBtn = wrapper.findAll('button').find(b => b.text() === '创建')
      await createBtn!.trigger('click')

      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(mockCreateCronJob).toHaveBeenCalledWith('once in 30m', undefined, undefined, undefined)
    })

    it('should show error when create fails', async () => {
      mockListCronJobs.mockResolvedValue([])
      mockCreateCronJob.mockRejectedValue(new Error('Invalid cron expression'))

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      mockListCronJobs.mockReset()
      mockListCronJobs.mockResolvedValue([])

      await (wrapper.vm as any).openCreateModal()
      await wrapper.vm.$nextTick()

      await wrapper.find('input[placeholder*="e.g."]').setValue('invalid cron')
      await wrapper.vm.$nextTick()

      const createBtn = wrapper.findAll('button').find(b => b.text() === '创建')
      await createBtn!.trigger('click')

      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('Invalid cron expression')
    })
  })

  describe('trigger job', () => {
    it('should call triggerCronJob and show success', async () => {
      mockListCronJobs.mockResolvedValue(sampleJobs())
      mockTriggerCronJob.mockResolvedValue(undefined)

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      mockListCronJobs.mockReset()
      mockListCronJobs.mockResolvedValue(sampleJobs())

      const triggerBtn = wrapper.findAll('button[title="立即执行"]')[0]
      await triggerBtn.trigger('click')

      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(mockTriggerCronJob).toHaveBeenCalledWith('job-1')
      expect(wrapper.text()).toContain('任务已触发执行')
    })

    it('should show error when trigger fails', async () => {
      mockListCronJobs.mockResolvedValue(sampleJobs())
      mockTriggerCronJob.mockRejectedValue(new Error('Job not found'))

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      const triggerBtn = wrapper.findAll('button[title="立即执行"]')[0]
      await triggerBtn.trigger('click')

      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('Job not found')
    })
  })

  describe('pause job', () => {
    it('should call pauseCronJob and show success', async () => {
      mockListCronJobs.mockResolvedValue(sampleJobs())
      mockPauseCronJob.mockResolvedValue(undefined)

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      mockListCronJobs.mockReset()
      mockListCronJobs.mockResolvedValue(sampleJobs())

      const pauseBtn = wrapper.find('button[title="暂停"]')
      await pauseBtn.trigger('click')

      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(mockPauseCronJob).toHaveBeenCalledWith('job-1')
      expect(wrapper.text()).toContain('任务已暂停')
    })

    it('should show error when pause fails', async () => {
      mockListCronJobs.mockResolvedValue(sampleJobs())
      mockPauseCronJob.mockRejectedValue(new Error('Pause failed'))

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      const pauseBtn = wrapper.find('button[title="暂停"]')
      await pauseBtn.trigger('click')

      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('Pause failed')
    })
  })

  describe('resume job', () => {
    it('should call resumeCronJob and show success', async () => {
      mockListCronJobs.mockResolvedValue(sampleJobs())
      mockResumeCronJob.mockResolvedValue(undefined)

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      mockListCronJobs.mockReset()
      mockListCronJobs.mockResolvedValue(sampleJobs())

      const resumeBtn = wrapper.find('button[title="恢复"]')
      await resumeBtn.trigger('click')

      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(mockResumeCronJob).toHaveBeenCalledWith('job-2')
      expect(wrapper.text()).toContain('任务已恢复')
    })
  })

  describe('delete job', () => {
    it('should call removeCronJob after confirm and show success', async () => {
      window.confirm = vi.fn(() => true)

      mockListCronJobs.mockResolvedValue(sampleJobs())
      mockRemoveCronJob.mockResolvedValue(undefined)

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      mockListCronJobs.mockReset()
      mockListCronJobs.mockResolvedValue(sampleJobs())

      const deleteBtn = wrapper.find('button[title="删除"]')
      await deleteBtn.trigger('click')

      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(window.confirm).toHaveBeenCalled()
      expect(mockRemoveCronJob).toHaveBeenCalledWith('job-1')
      expect(wrapper.text()).toContain('任务已删除')
    })

    it('should NOT call removeCronJob if confirm is cancelled', async () => {
      window.confirm = vi.fn(() => false)

      mockListCronJobs.mockResolvedValue(sampleJobs())

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      const deleteBtn = wrapper.find('button[title="删除"]')
      await deleteBtn.trigger('click')

      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(window.confirm).toHaveBeenCalled()
      expect(mockRemoveCronJob).not.toHaveBeenCalled()
    })
  })

  // ============ Refresh ============
  describe('refresh', () => {
    it('should reload jobs when refresh method is called', async () => {
      mockListCronJobs.mockResolvedValue(sampleJobs())

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      mockListCronJobs.mockReset()
      mockListCronJobs.mockResolvedValue(sampleJobs())

      await (wrapper.vm as any).refresh()
      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(mockListCronJobs).toHaveBeenCalledTimes(1)
    })
  })

  // ============ statusBadgeClass ============
  describe('statusBadgeClass', () => {
    it('should return success class for active jobs', () => {
      const vm = mount(CronManager, {
        global: { stubs: STUBS },
      }).vm as any

      expect(vm.statusBadgeClass({ state: 'active' })).toBe('bg-success/15 text-success')
    })

    it('should return warning class for paused jobs', () => {
      const vm = mount(CronManager, {
        global: { stubs: STUBS },
      }).vm as any

      expect(vm.statusBadgeClass({ state: 'paused' })).toBe('bg-warning/15 text-warning')
    })

    it('should return info class for completed jobs', () => {
      const vm = mount(CronManager, {
        global: { stubs: STUBS },
      }).vm as any

      expect(vm.statusBadgeClass({ state: 'completed' })).toBe('bg-info/15 text-info')
    })

    it('should return default class for unknown states', () => {
      const vm = mount(CronManager, {
        global: { stubs: STUBS },
      }).vm as any

      expect(vm.statusBadgeClass({ state: 'scheduled' })).toBe('bg-base-300 text-base-content/60')
    })
  })

  // ============ Success Messages ============
  describe('success message display', () => {
    it('should show success message after trigger', async () => {
      mockListCronJobs.mockResolvedValue(sampleJobs())
      mockTriggerCronJob.mockResolvedValue(undefined)

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      mockListCronJobs.mockReset()
      mockListCronJobs.mockResolvedValue(sampleJobs())

      const triggerBtn = wrapper.findAll('button[title="立即执行"]')[0]
      await triggerBtn.trigger('click')
      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('任务已触发执行')
    })

    it('should show success message after create', async () => {
      mockListCronJobs.mockResolvedValue([])
      mockCreateCronJob.mockResolvedValue(undefined)

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      mockListCronJobs.mockReset()
      mockListCronJobs.mockResolvedValue([])

      await (wrapper.vm as any).openCreateModal()
      await wrapper.vm.$nextTick()

      await wrapper.find('input[placeholder*="e.g."]').setValue('every 1h')
      await wrapper.vm.$nextTick()

      const createBtn = wrapper.findAll('button').find(b => b.text() === '创建')
      await createBtn!.trigger('click')
      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('任务创建成功')
    })
  })

  // ============ Error Message Dismiss ============
  describe('error message handling', () => {
    it('should show error when listCronJobs fails', async () => {
      mockListCronJobs.mockRejectedValue(new Error('Failed to connect'))

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('Failed to connect')
    })

    it('should change message when new error overwrites success', async () => {
      mockListCronJobs.mockResolvedValue(sampleJobs())
      mockPauseCronJob.mockResolvedValue(undefined)

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      mockListCronJobs.mockReset()
      mockListCronJobs.mockResolvedValue(sampleJobs())
      mockPauseCronJob.mockRejectedValue(new Error('Pause denied'))

      const pauseBtn = wrapper.find('button[title="暂停"]')
      await pauseBtn.trigger('click')
      await flushPromises()
      await wrapper.vm.$nextTick()

      // Error should be shown
      expect(wrapper.text()).toContain('Pause denied')
      // No success message
      expect(wrapper.text()).not.toContain('任务已暂停')
    })
  })

  // ============ Boundary: Invalid Cron Expressions ============
  describe('boundary: invalid cron expressions', () => {
    it('should show error when submitting invalid schedule string', async () => {
      mockListCronJobs.mockResolvedValue([])
      mockCreateCronJob.mockRejectedValue(new Error('Invalid cron expression'))

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      mockListCronJobs.mockReset()
      mockListCronJobs.mockResolvedValue([])

      await (wrapper.vm as any).openCreateModal()
      await wrapper.vm.$nextTick()

      await wrapper.find('input[placeholder*="e.g."]').setValue('not-a-valid-cron')
      await wrapper.vm.$nextTick()

      const createBtn = wrapper.findAll('button').find(b => b.text() === '创建')
      await createBtn!.trigger('click')

      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('Invalid cron expression')
    })

    it('should not submit when schedule is only whitespace', async () => {
      mockListCronJobs.mockResolvedValue([])

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      await (wrapper.vm as any).openCreateModal()
      await wrapper.vm.$nextTick()

      await wrapper.find('input[placeholder*="e.g."]').setValue('   ')
      await wrapper.vm.$nextTick()

      const createBtn = wrapper.findAll('button').find(b => b.text() === '创建')
      // Button should be disabled because schedule.trim() is empty
      expect(createBtn?.attributes('disabled')).toBeDefined()
      expect(mockCreateCronJob).not.toHaveBeenCalled()
    })

    it('should handle very long schedule input (boundary)', async () => {
      mockListCronJobs.mockResolvedValue([])

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      await (wrapper.vm as any).openCreateModal()
      await wrapper.vm.$nextTick()

      // 1000-character schedule string
      const longSchedule = '*/5 * * * *' + 'x'.repeat(990)
      await wrapper.find('input[placeholder*="e.g."]').setValue(longSchedule)
      await wrapper.vm.$nextTick()

      const createBtn = wrapper.findAll('button').find(b => b.text() === '创建')
      expect(createBtn?.attributes('disabled')).toBeUndefined()
    })

    it('should handle schedule with special characters (unicode, HTML)', async () => {
      mockListCronJobs.mockResolvedValue([])
      mockCreateCronJob.mockResolvedValue(undefined)

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      mockListCronJobs.mockReset()
      mockListCronJobs.mockResolvedValue([])

      await (wrapper.vm as any).openCreateModal()
      await wrapper.vm.$nextTick()

      await wrapper.find('input[placeholder*="e.g."]').setValue('every 你好 世界')
      await wrapper.vm.$nextTick()

      const createBtn = wrapper.findAll('button').find(b => b.text() === '创建')
      await createBtn!.trigger('click')

      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(mockCreateCronJob).toHaveBeenCalledWith('every 你好 世界', undefined, undefined, undefined)
    })

    it('should handle schedule with HTML-like input', async () => {
      mockListCronJobs.mockResolvedValue([])
      mockCreateCronJob.mockResolvedValue(undefined)

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      mockListCronJobs.mockReset()
      mockListCronJobs.mockResolvedValue([])

      await (wrapper.vm as any).openCreateModal()
      await wrapper.vm.$nextTick()

      await wrapper.find('input[placeholder*="e.g."]').setValue('<script>alert(1)</script>')
      await wrapper.vm.$nextTick()

      const createBtn = wrapper.findAll('button').find(b => b.text() === '创建')
      await createBtn!.trigger('click')

      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(mockCreateCronJob).toHaveBeenCalledWith('<script>alert(1)</script>', undefined, undefined, undefined)
    })
  })

  // ============ Boundary: Very Long Inputs ============
  describe('boundary: very long inputs', () => {
    it('should handle very long name input', async () => {
      mockListCronJobs.mockResolvedValue([])
      mockCreateCronJob.mockResolvedValue(undefined)

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      mockListCronJobs.mockReset()
      mockListCronJobs.mockResolvedValue([])

      await (wrapper.vm as any).openCreateModal()
      await wrapper.vm.$nextTick()

      const longName = 'A'.repeat(2000)
      await wrapper.find('input[placeholder*="e.g."]').setValue('0 9 * * *')
      await wrapper.find('input[placeholder*="可选，留空自动生成"]').setValue(longName)
      await wrapper.vm.$nextTick()

      const createBtn = wrapper.findAll('button').find(b => b.text() === '创建')
      await createBtn!.trigger('click')

      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(mockCreateCronJob).toHaveBeenCalledWith('0 9 * * *', undefined, longName, undefined)
    })

    it('should handle very long prompt input', async () => {
      mockListCronJobs.mockResolvedValue([])
      mockCreateCronJob.mockResolvedValue(undefined)

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      mockListCronJobs.mockReset()
      mockListCronJobs.mockResolvedValue([])

      await (wrapper.vm as any).openCreateModal()
      await wrapper.vm.$nextTick()

      const longPrompt = 'x'.repeat(5000)
      await wrapper.find('input[placeholder*="e.g."]').setValue('0 9 * * *')
      await wrapper.find('textarea').setValue(longPrompt)
      await wrapper.vm.$nextTick()

      const createBtn = wrapper.findAll('button').find(b => b.text() === '创建')
      await createBtn!.trigger('click')

      await flushPromises()
      await wrapper.vm.$nextTick()

      // Verify the long prompt was passed through without errors
      expect(mockCreateCronJob).toHaveBeenCalled()
      expect(mockCreateCronJob.mock.calls[0][0]).toBe('0 9 * * *')
      expect(mockCreateCronJob.mock.calls[0][1]).toBe(longPrompt)
    })
  })

  // ============ Form Reset Behavior ============
  describe('form reset on modal open', () => {
    it('should reset form fields when opening create modal', async () => {
      mockListCronJobs.mockResolvedValue([])

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      // Open modal and fill fields
      await (wrapper.vm as any).openCreateModal()
      await wrapper.vm.$nextTick()

      await wrapper.find('input[placeholder*="e.g."]').setValue('0 9 * * *')
      await wrapper.find('input[placeholder*="可选，留空自动生成"]').setValue('Test Name')
      await wrapper.find('textarea').setValue('Test prompt')

      // Close and reopen modal
      await (wrapper.vm as any).openCreateModal()
      await wrapper.vm.$nextTick()

      // Fields should be reset
      const scheduleInput = wrapper.find('input[placeholder*="e.g."]')
      expect((scheduleInput.element as HTMLInputElement).value).toBe('')
    })
  })

  // ============ formatTime Edge Cases ============
  describe('formatTime edge cases', () => {
    it('should return empty string for null input', () => {
      const vm = mount(CronManager, {
        global: { stubs: STUBS },
      }).vm as any

      expect(vm.formatTime(null)).toBe('')
    })

    it('should return empty string for undefined input', () => {
      const vm = mount(CronManager, {
        global: { stubs: STUBS },
      }).vm as any

      expect(vm.formatTime(undefined)).toBe('')
    })

    it('should return Invalid Date string for invalid date', () => {
      const vm = mount(CronManager, {
        global: { stubs: STUBS },
      }).vm as any

      // new Date('not-a-date') creates Invalid Date, toLocaleString() returns 'Invalid Date'
      expect(vm.formatTime('not-a-date')).toBe('Invalid Date')
    })

    it('should format valid ISO date string correctly', () => {
      const vm = mount(CronManager, {
        global: { stubs: STUBS },
      }).vm as any

      const result = vm.formatTime('2026-05-29T09:00:00Z')
      // Should return localized month/day and time (e.g. "05/29 17:00" in CST)
      expect(result).toContain('05/29')
    })
  })

  // ============ Error Message Dismiss ============
  describe('error message dismiss', () => {
    it('should dismiss error message when close button is clicked', async () => {
      mockListCronJobs.mockRejectedValue(new Error('Some error'))

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      // Error should be shown
      expect(wrapper.text()).toContain('Some error')

      // Find and click the dismiss X button
      const dismissBtn = wrapper.findAll('button').find(b => b.find('SvgIcon[name="x"]'))
        || wrapper.findAll('button').filter(b => b.attributes('class')?.includes('btn-ghost')).find(b =>
          b.text() === ''
        )
      // Fallback: find any button inside the error div
      const errorDiv = wrapper.find('.text-error')
      if (errorDiv.exists()) {
        const closeBtns = errorDiv.findAll('button')
        for (const btn of closeBtns) {
          await btn.trigger('click')
        }
        await wrapper.vm.$nextTick()
        expect(wrapper.text()).not.toContain('Some error')
      }
    })
  })

  // ============ Loading State on Action Buttons ============
  describe('action button loading state', () => {
    it('should disable action buttons while an action is in progress', async () => {
      mockListCronJobs.mockResolvedValue(sampleJobs())
      // Keep trigger promise pending so loading stays true
      mockTriggerCronJob.mockReturnValue(new Promise(() => {}))

      const wrapper = mount(CronManager, { global: { stubs: STUBS } })
      await flushPromises()
      await wrapper.vm.$nextTick()

      // Click trigger on first job
      const triggerBtn = wrapper.findAll('button[title="立即执行"]')[0]
      await triggerBtn.trigger('click')
      await wrapper.vm.$nextTick()

      // All buttons for this job should be disabled
      expect(triggerBtn.attributes('disabled')).toBeDefined()
    })
  })
})
