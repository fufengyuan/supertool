import { describe, it, expect, vi, beforeEach } from 'vitest'
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

// Top-level static import — no dynamic import
import SkillsBrowser from '@/views/agent/SkillsBrowser.vue'

// ── Skill Fixtures ─────────────────────────────────────────────────────────

function makeSkill(overrides: Partial<import('@/types').SkillInfo> = {}): import('@/types').SkillInfo {
  return {
    name: 'test-skill',
    category: 'devops',
    description: 'A test skill description',
    path: '/tmp/skills/devops/test-skill',
    source: 'bundled',
    ...overrides,
  }
}

// ── Mount helper ───────────────────────────────────────────────────────────

async function createSkillsBrowser() {
  const el = document.createElement('div')
  document.body.appendChild(el)
  const wrapper = mount(SkillsBrowser, {
    attachTo: el,
    global: {
      stubs: { Teleport: false },
    },
  })
  // Flush: onMounted → loadAll → API calls settle → Vue DOM update
  await vi.dynamicImportSettled()
  await nextTick()
  return { wrapper }
}

// ── Tests ──────────────────────────────────────────────────────────────────

describe('SkillsBrowser.vue', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    document.body.innerHTML = ''
  })

  // ── Loading State ─────────────────────────────────────────────────────────

  it('should show loading spinner on mount', async () => {
    mockedInvoke.mockReturnValue(new Promise(() => {}))
    const { wrapper } = await createSkillsBrowser()
    expect(wrapper.find('.loading-spinner').exists()).toBe(true)
  })

  // ── Empty State ───────────────────────────────────────────────────────────

  it('should show empty message when no skills installed', async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {return Promise.resolve([])}
      if (cmd === 'list_bundled_skills') {return Promise.resolve([])}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()
    expect(wrapper.text()).toContain('没有匹配的技能')
  })

  it('should show empty message in browse tab', async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {return Promise.resolve([])}
      if (cmd === 'list_bundled_skills') {return Promise.resolve([])}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()
    const browseBtn = wrapper.findAll('button').find(b => b.text().includes('浏览'))
    await browseBtn!.trigger('click')
    expect(wrapper.text()).toContain('没有匹配的技能')
  })

  // ── Skill Rendering ───────────────────────────────────────────────────────

  it('should render installed skills in the installed tab', async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {return Promise.resolve([
        makeSkill({ name: 'git-helper', source: 'installed' }),
        makeSkill({ name: 'deploy-tool', source: 'installed' }),
      ])}
      if (cmd === 'list_bundled_skills') {return Promise.resolve([])}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()
    expect(wrapper.text()).toContain('git-helper')
    expect(wrapper.text()).toContain('deploy-tool')
  })

  it('should render bundled skills in the browse tab', async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {return Promise.resolve([])}
      if (cmd === 'list_bundled_skills') {return Promise.resolve([
        makeSkill({ name: 'git-skills', source: 'bundled' }),
        makeSkill({ name: 'ml-training', source: 'bundled' }),
      ])}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()
    const browseBtn = wrapper.findAll('button').find(b => b.text().includes('浏览'))
    await browseBtn!.trigger('click')
    expect(wrapper.text()).toContain('git-skills')
    expect(wrapper.text()).toContain('ml-training')
  })

  // ── Tab Switching ─────────────────────────────────────────────────────────

  it('should switch between installed and browse tabs', async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {return Promise.resolve([
        makeSkill({ name: 'installed-only', source: 'installed' }),
      ])}
      if (cmd === 'list_bundled_skills') {return Promise.resolve([
        makeSkill({ name: 'bundled-only', source: 'bundled' }),
      ])}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()
    expect(wrapper.text()).toContain('installed-only')
    expect(wrapper.text()).not.toContain('bundled-only')

    const browseBtn = wrapper.findAll('button').find(b => b.text().includes('浏览'))
    await browseBtn!.trigger('click')
    expect(wrapper.text()).toContain('bundled-only')
    expect(wrapper.text()).not.toContain('installed-only')
  })

  // ── Search Filtering ──────────────────────────────────────────────────────

  it('should filter skills by name', async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {return Promise.resolve([
        makeSkill({ name: 'docker-deploy', source: 'installed' }),
        makeSkill({ name: 'git-helper', source: 'installed' }),
      ])}
      if (cmd === 'list_bundled_skills') {return Promise.resolve([])}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()
    const input = wrapper.find('input')
    await input.setValue('docker')
    expect(wrapper.text()).toContain('docker-deploy')
    expect(wrapper.text()).not.toContain('git-helper')
  })

  it('should filter skills by description', async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {return Promise.resolve([
        makeSkill({ name: 'tool-a', description: 'Docker management', source: 'installed' }),
        makeSkill({ name: 'tool-b', description: 'Git operations', source: 'installed' }),
      ])}
      if (cmd === 'list_bundled_skills') {return Promise.resolve([])}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()
    const input = wrapper.find('input')
    await input.setValue('management')
    expect(wrapper.text()).toContain('tool-a')
    expect(wrapper.text()).not.toContain('tool-b')
  })

  it('should filter skills by category', async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {return Promise.resolve([])}
      if (cmd === 'list_bundled_skills') {return Promise.resolve([
        makeSkill({ name: 'backend', category: 'devops', source: 'bundled' }),
        makeSkill({ name: 'trainer', category: 'mlops', source: 'bundled' }),
        makeSkill({ name: 'deployer', category: 'devops', source: 'bundled' }),
      ])}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()
    const browseBtn = wrapper.findAll('button').find(b => b.text().includes('浏览'))
    await browseBtn!.trigger('click')
    const catBtns = wrapper.findAll('button').filter(
      b => b.text().trim() === 'devops' && b.classes().includes('rounded-full'),
    )
    expect(catBtns.length).toBeGreaterThanOrEqual(1)
    await catBtns[0].trigger('click')
    expect(wrapper.text()).toContain('backend')
    expect(wrapper.text()).toContain('deployer')
    expect(wrapper.text()).not.toContain('trainer')
  })

  it('should show empty message when search finds nothing', async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {return Promise.resolve([
        makeSkill({ name: 'tool-a', source: 'installed' }),
      ])}
      if (cmd === 'list_bundled_skills') {return Promise.resolve([])}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()
    const input = wrapper.find('input')
    await input.setValue('zzz_nonexistent')
    expect(wrapper.text()).toContain('没有匹配的技能')
  })

  // ── Error States ──────────────────────────────────────────────────────────

  it('should display error when API calls fail', async () => {
    mockedInvoke.mockRejectedValue(new Error('Network error'))
    const { wrapper } = await createSkillsBrowser()
    expect(wrapper.text()).toContain('Network error')
  })

  it('should dismiss error when close button is clicked', async () => {
    mockedInvoke.mockRejectedValue(new Error('Network error'))
    const { wrapper } = await createSkillsBrowser()
    expect(wrapper.text()).toContain('Network error')
    const dismissBtn = wrapper.find('.text-error button, .bg-error\\/10 button')
    if (dismissBtn.exists()) {
      await dismissBtn.trigger('click')
      await nextTick()
      expect(wrapper.text()).not.toContain('Network error')
    }
  })

  // ── Detail Overlay ────────────────────────────────────────────────────────

  it('should open detail overlay on skill card click', async () => {
    const skill = makeSkill({ name: 'detail-skill', source: 'installed' })
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {return Promise.resolve([skill])}
      if (cmd === 'list_bundled_skills') {return Promise.resolve([])}
      if (cmd === 'get_skill_content') {return Promise.resolve('# Full body')}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()

    const card = wrapper.find('.cursor-pointer')
    expect(card.exists()).toBe(true)
    await card.trigger('click')

    await vi.dynamicImportSettled()
    await nextTick()

    const bodyText = document.body.textContent || ''
    expect(bodyText).toContain('detail-skill')
    expect(bodyText).toContain('Full body')
  })

  it('should close detail overlay', async () => {
    const skill = makeSkill({ name: 'close-skill', source: 'installed' })
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {return Promise.resolve([skill])}
      if (cmd === 'list_bundled_skills') {return Promise.resolve([])}
      if (cmd === 'get_skill_content') {return Promise.resolve('# Content')}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()

    const card = wrapper.find('.cursor-pointer')
    await card.trigger('click')
    await vi.dynamicImportSettled()
    await nextTick()

    expect(document.body.textContent).toContain('# Content')

    const overlay = document.querySelector('.fixed.inset-0')
    expect(overlay).not.toBeNull()
    ;(overlay as HTMLElement).click()
    await nextTick()

    expect(document.body.textContent).not.toContain('# Content')
  })

  it('should show error when getSkillContent fails', async () => {
    const skill = makeSkill({ name: 'err-skill', source: 'installed' })
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {return Promise.resolve([skill])}
      if (cmd === 'list_bundled_skills') {return Promise.resolve([])}
      if (cmd === 'get_skill_content') {return Promise.reject(new Error('Read error'))}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()

    const card = wrapper.find('.cursor-pointer')
    await card.trigger('click')
    await vi.dynamicImportSettled()
    await nextTick()

    expect(document.body.textContent).toContain('Read error')
  })

  // ── Install Flow ──────────────────────────────────────────────────────────

  it('should call installSkill and reload on success', async () => {
    const skill = makeSkill({ name: 'install-me', source: 'bundled' })
    let resolveInstall!: (v: unknown) => void
    const installPromise = new Promise(resolve => { resolveInstall = resolve })
    let callCount = 0

    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {
        callCount++
        return Promise.resolve(callCount === 1 ? [] : [makeSkill({ ...skill, source: 'installed' })])
      }
      if (cmd === 'list_bundled_skills') {return Promise.resolve([skill])}
      if (cmd === 'install_skill') {return installPromise}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()

    const browseBtn = wrapper.findAll('button').find(b => b.text().includes('浏览'))
    await browseBtn!.trigger('click')

    const installBtn = wrapper.findAll('button').find(b => b.text() === '安装')
    expect(installBtn).toBeTruthy()
    await installBtn!.trigger('click')

    expect(wrapper.text()).toContain('...')

    resolveInstall({ success: true })
    await vi.dynamicImportSettled()
    await nextTick()

    expect(mockedInvoke).toHaveBeenCalledWith('install_skill', { identifier: 'devops/install-me' })
    expect(callCount).toBe(2)
  })

  it('should show error when installSkill returns error', async () => {
    const skill = makeSkill({ name: 'fail-install', source: 'bundled' })
    let resolveInstall!: (v: unknown) => void
    const installPromise = new Promise(resolve => { resolveInstall = resolve })

    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {return Promise.resolve([])}
      if (cmd === 'list_bundled_skills') {return Promise.resolve([skill])}
      if (cmd === 'install_skill') {return installPromise}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()

    const browseBtn = wrapper.findAll('button').find(b => b.text().includes('浏览'))
    await browseBtn!.trigger('click')
    const installBtn = wrapper.findAll('button').find(b => b.text() === '安装')
    await installBtn!.trigger('click')

    resolveInstall({ success: false, error: 'Permission denied' })
    await vi.dynamicImportSettled()
    await nextTick()

    expect(wrapper.text()).toContain('Permission denied')
  })

  it('should show error when installSkill throws', async () => {
    const skill = makeSkill({ name: 'throw-install', source: 'bundled' })
    let rejectInstall!: (e: Error) => void
    const installPromise = new Promise((_, reject) => { rejectInstall = reject })

    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {return Promise.resolve([])}
      if (cmd === 'list_bundled_skills') {return Promise.resolve([skill])}
      if (cmd === 'install_skill') {return installPromise}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()

    const browseBtn = wrapper.findAll('button').find(b => b.text().includes('浏览'))
    await browseBtn!.trigger('click')
    const installBtn = wrapper.findAll('button').find(b => b.text() === '安装')
    await installBtn!.trigger('click')

    rejectInstall(new Error('CLI crashed'))
    await vi.dynamicImportSettled()
    await nextTick()

    expect(wrapper.text()).toContain('CLI crashed')
  })

  // ── Uninstall Flow ────────────────────────────────────────────────────────

  it('should call uninstallSkill and reload on success', async () => {
    const skill = makeSkill({ name: 'uninstall-me', source: 'installed' })
    let resolveUninstall!: (v: unknown) => void
    const uninstallPromise = new Promise(resolve => { resolveUninstall = resolve })
    let callCount = 0

    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {
        callCount++
        return Promise.resolve(callCount === 1 ? [skill] : [])
      }
      if (cmd === 'list_bundled_skills') {return Promise.resolve([])}
      if (cmd === 'uninstall_skill') {return uninstallPromise}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()

    const uninstallBtn = wrapper.findAll('button').find(b => b.text() === '卸载')
    expect(uninstallBtn).toBeTruthy()
    await uninstallBtn!.trigger('click')

    expect(wrapper.text()).toContain('...')

    resolveUninstall({ success: true })
    await vi.dynamicImportSettled()
    await nextTick()

    expect(mockedInvoke).toHaveBeenCalledWith('uninstall_skill', { name: 'devops/uninstall-me' })
    expect(callCount).toBe(2)
  })

  it('should show error when uninstall fails', async () => {
    const skill = makeSkill({ name: 'fail-uninstall', source: 'installed' })
    let resolveUninstall!: (v: unknown) => void
    const uninstallPromise = new Promise(resolve => { resolveUninstall = resolve })

    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {return Promise.resolve([skill])}
      if (cmd === 'list_bundled_skills') {return Promise.resolve([])}
      if (cmd === 'uninstall_skill') {return uninstallPromise}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()

    const uninstallBtn = wrapper.findAll('button').find(b => b.text() === '卸载')
    await uninstallBtn!.trigger('click')

    resolveUninstall({ success: false, error: 'Skill is protected' })
    await vi.dynamicImportSettled()
    await nextTick()

    expect(wrapper.text()).toContain('Skill is protected')
  })

  // ── Refresh Button ────────────────────────────────────────────────────────

  it('should reload skills when refresh button is clicked', async () => {
    let callCount = 0
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {
        callCount++
        return Promise.resolve(
          callCount === 1
            ? [makeSkill({ name: 'old-skill', source: 'installed' })]
            : [makeSkill({ name: 'new-skill', source: 'installed' })],
        )
      }
      if (cmd === 'list_bundled_skills') {return Promise.resolve([])}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()

    expect(wrapper.text()).toContain('old-skill')
    expect(wrapper.text()).not.toContain('new-skill')

    const refreshIcon = wrapper.find('.svg-icon-stub[data-name="refresh"]')
    const parentBtn = refreshIcon.element.closest('button')
    ;(parentBtn as HTMLElement).click()
    await vi.dynamicImportSettled()
    await nextTick()

    expect(wrapper.text()).toContain('new-skill')
    expect(callCount).toBe(2)
  })

  // ── Category Filter Toggle ────────────────────────────────────────────────

  it('should toggle category filter on/off', async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {return Promise.resolve([])}
      if (cmd === 'list_bundled_skills') {return Promise.resolve([
        makeSkill({ name: 'skill-a', category: 'devops', source: 'bundled' }),
        makeSkill({ name: 'skill-b', category: 'mlops', source: 'bundled' }),
      ])}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()
    const browseBtn = wrapper.findAll('button').find(b => b.text().includes('浏览'))
    await browseBtn!.trigger('click')

    const catBtns = wrapper.findAll('button').filter(
      b => b.text().trim() === 'devops' && b.classes().includes('rounded-full'),
    )
    expect(catBtns.length).toBeGreaterThanOrEqual(1)

    await catBtns[0].trigger('click')
    expect(wrapper.text()).toContain('skill-a')
    expect(wrapper.text()).not.toContain('skill-b')

    await catBtns[0].trigger('click')
    expect(wrapper.text()).toContain('skill-a')
    expect(wrapper.text()).toContain('skill-b')
  })

  // ── Empty Content Fallback ────────────────────────────────────────────────

  it('should show fallback when skill content is empty', async () => {
    const skill = makeSkill({ name: 'empty-content', source: 'installed' })
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {return Promise.resolve([skill])}
      if (cmd === 'list_bundled_skills') {return Promise.resolve([])}
      if (cmd === 'get_skill_content') {return Promise.resolve('')}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()

    const card = wrapper.find('.cursor-pointer')
    await card.trigger('click')
    await vi.dynamicImportSettled()
    await nextTick()

    expect(document.body.textContent).toContain('（空的技能文件）')
  })

  // ── Boot contract ─────────────────────────────────────────────────────────

  it('should call both list APIs on mount', async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {return Promise.resolve([])}
      if (cmd === 'list_bundled_skills') {return Promise.resolve([])}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    await createSkillsBrowser()
    expect(mockedInvoke).toHaveBeenCalledWith('list_installed_skills', {})
    expect(mockedInvoke).toHaveBeenCalledWith('list_bundled_skills', {})
  })

  // ── Edge Cases ────────────────────────────────────────────────────────────

  it('should handle empty category and description gracefully', async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {return Promise.resolve([])}
      if (cmd === 'list_bundled_skills') {return Promise.resolve([
        makeSkill({ name: 'minimal', description: '', category: '' }),
      ])}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()
    const browseBtn = wrapper.findAll('button').find(b => b.text().includes('浏览'))
    await browseBtn!.trigger('click')
    expect(wrapper.text()).toContain('minimal')
    expect(wrapper.text()).toContain('暂无描述')
  })

  it('should display correct source badge for installed vs bundled', async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {return Promise.resolve([
        makeSkill({ name: 'inst', source: 'installed' }),
      ])}
      if (cmd === 'list_bundled_skills') {return Promise.resolve([
        makeSkill({ name: 'bndl', source: 'bundled' }),
      ])}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()
    expect(wrapper.text()).toContain('inst')
    expect(wrapper.text()).toContain('已安装')

    const browseBtn = wrapper.findAll('button').find(b => b.text().includes('浏览'))
    await browseBtn!.trigger('click')
    expect(wrapper.text()).toContain('bndl')
    expect(wrapper.text()).toContain('可安装')
  })

  // ── Search Clear Restore ────────────────────────────────────────────────

  it('should restore full list when search is cleared', async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {return Promise.resolve([
        makeSkill({ name: 'skill-alpha', source: 'installed' }),
        makeSkill({ name: 'skill-beta', source: 'installed' }),
      ])}
      if (cmd === 'list_bundled_skills') {return Promise.resolve([])}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()
    const input = wrapper.find('input')
    await input.setValue('beta')
    expect(wrapper.text()).toContain('skill-beta')
    expect(wrapper.text()).not.toContain('skill-alpha')
    await input.setValue('')
    expect(wrapper.text()).toContain('skill-alpha')
    expect(wrapper.text()).toContain('skill-beta')
  })

  // ── Search by Category ──────────────────────────────────────────────────

  it('should filter by category name in search', async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {return Promise.resolve([
        makeSkill({ name: 'tool-a', category: 'devops', source: 'installed' }),
        makeSkill({ name: 'tool-b', category: 'mlops', source: 'installed' }),
      ])}
      if (cmd === 'list_bundled_skills') {return Promise.resolve([])}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()
    const input = wrapper.find('input')
    await input.setValue('mlops')
    expect(wrapper.text()).toContain('tool-b')
    expect(wrapper.text()).not.toContain('tool-a')
  })

  // ── Very Long Name ──────────────────────────────────────────────────────

  it('should handle very long skill name without breaking', async () => {
    const longName = 'a'.repeat(200)
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {return Promise.resolve([
        makeSkill({ name: longName, source: 'installed' }),
      ])}
      if (cmd === 'list_bundled_skills') {return Promise.resolve([])}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()
    expect(wrapper.text()).toContain(longName)
  })

  // ── Category Pills Hidden ───────────────────────────────────────────────

  it('should not show category pills when browse tab has no categories', async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {return Promise.resolve([])}
      if (cmd === 'list_bundled_skills') {return Promise.resolve([])}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()
    const browseBtn = wrapper.findAll('button').find(b => b.text().includes('浏览'))
    await browseBtn!.trigger('click')
    await nextTick()
    const catPills = wrapper.findAll('button').filter(
      b => b.classes().includes('rounded-full'),
    )
    expect(catPills.length).toBe(0)
  })

  // ── Detail Overlay Install Button ───────────────────────────────────────

  it('should show install button in detail overlay for bundled skill', async () => {
    const skill = makeSkill({ name: 'browse-detail', source: 'bundled' })
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {return Promise.resolve([])}
      if (cmd === 'list_bundled_skills') {return Promise.resolve([skill])}
      if (cmd === 'get_skill_content') {return Promise.resolve('# Details')}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()
    const browseBtn = wrapper.findAll('button').find(b => b.text().includes('浏览'))
    await browseBtn!.trigger('click')
    const card = wrapper.find('.cursor-pointer')
    await card.trigger('click')
    await vi.dynamicImportSettled()
    await nextTick()
    expect(document.body.textContent).toContain('# Details')
    expect(document.body.textContent).toContain('安装')
  })

  // ── Detail Overlay Uninstall Button ─────────────────────────────────────

  it('should show uninstall button in detail overlay for installed skill', async () => {
    const skill = makeSkill({ name: 'installed-detail', source: 'installed' })
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_installed_skills') {return Promise.resolve([skill])}
      if (cmd === 'list_bundled_skills') {return Promise.resolve([])}
      if (cmd === 'get_skill_content') {return Promise.resolve('# Installed body')}
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()
    const card = wrapper.find('.cursor-pointer')
    await card.trigger('click')
    await vi.dynamicImportSettled()
    await nextTick()
    expect(document.body.textContent).toContain('# Installed body')
    expect(document.body.textContent).toContain('卸载')
  })

  // ── Error Cleared on Refresh ────────────────────────────────────────────

  it('should clear error when refresh is clicked', async () => {
    let callCount = 0
    mockedInvoke.mockImplementation((cmd: string) => {
      callCount++
      if (cmd === 'list_installed_skills') {
        return callCount === 1
          ? Promise.reject(new Error('First load failed'))
          : Promise.resolve([])
      }
      if (cmd === 'list_bundled_skills') {
        return callCount === 2
          ? Promise.reject(new Error('First load failed'))
          : Promise.resolve([])
      }
      return Promise.reject(new Error(`unexpected cmd: ${cmd}`))
    })
    const { wrapper } = await createSkillsBrowser()
    expect(wrapper.text()).toContain('First load failed')

    const refreshIcon = wrapper.find('.svg-icon-stub[data-name="refresh"]')
    const parentBtn = refreshIcon.element.closest('button')
    ;(parentBtn as HTMLElement).click()
    await vi.dynamicImportSettled()
    await nextTick()
    expect(wrapper.text()).not.toContain('First load failed')
    expect(wrapper.text()).toContain('没有匹配的技能')
  })
})
