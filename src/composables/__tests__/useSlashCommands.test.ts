import { describe, it, expect, vi, beforeEach } from 'vitest'
import { useSlashCommands, SLASH_COMMANDS, CATEGORY_ICONS, CATEGORY_LABELS } from '../useSlashCommands'

// ── Helpers ─────────────────────────────────────────────────────────────────

/** Create default options for useSlashCommands with mock callbacks */
function makeOptions(overrides: Record<string, unknown> = {}): Record<string, any> {
  return {
    onNewChat: vi.fn(),
    onClear: vi.fn(),
    addAgentMessage: vi.fn(),
    usageStats: null as any,
    ...overrides,
  }
}

/** Helper to call filteredSlashCommands.value after updating input text */
function filterWith(inputText: string, slash: ReturnType<typeof useSlashCommands>) {
  slash.updateInputText(inputText)
  return slash.filteredSlashCommands.value
}

// ── SLASH_COMMANDS data integrity ───────────────────────────────────────────

describe('SLASH_COMMANDS list', () => {
  it('should have at least 30 commands', () => {
    expect(SLASH_COMMANDS.length).toBeGreaterThanOrEqual(30)
  })

  it('should have all 4 categories represented', () => {
    const cats = new Set(SLASH_COMMANDS.map(c => c.category))
    expect(cats).toEqual(new Set(['chat', 'agent', 'tools', 'info']))
  })

  it('each command should have name starting with /', () => {
    for (const cmd of SLASH_COMMANDS) {
      expect(cmd.name).toMatch(/^\//)
    }
  })

  it('each command should have a non-empty description', () => {
    for (const cmd of SLASH_COMMANDS) {
      expect(cmd.description).toBeTruthy()
    }
  })

  it('each command should have a valid category', () => {
    const valid = ['chat', 'agent', 'tools', 'info']
    for (const cmd of SLASH_COMMANDS) {
      expect(valid).toContain(cmd.category)
    }
  })

  it('each command should have a boolean local flag', () => {
    for (const cmd of SLASH_COMMANDS) {
      expect(typeof cmd.local).toBe('boolean')
    }
  })

  it('should contain expected commands', () => {
    const names = SLASH_COMMANDS.map(c => c.name)
    const expected = ['/new', '/clear', '/help', '/usage', '/web', '/code', '/shell', '/fast', '/model', '/version']
    for (const n of expected) {
      expect(names).toContain(n)
    }
  })

  it('should have no duplicate command names', () => {
    const names = SLASH_COMMANDS.map(c => c.name)
    expect(new Set(names).size).toBe(names.length)
  })

  it('should contain exactly /new and /clear in chat category', () => {
    const chatCmds = SLASH_COMMANDS.filter(c => c.category === 'chat')
    expect(chatCmds.map(c => c.name).sort()).toEqual(['/clear', '/new'])
  })

  it('should have at least 10 agent commands', () => {
    const agentCmds = SLASH_COMMANDS.filter(c => c.category === 'agent')
    expect(agentCmds.length).toBeGreaterThanOrEqual(10)
  })

  it('should have at least 5 tool commands', () => {
    const toolCmds = SLASH_COMMANDS.filter(c => c.category === 'tools')
    expect(toolCmds.length).toBeGreaterThanOrEqual(5)
  })

  it('should have at least 8 info commands', () => {
    const infoCmds = SLASH_COMMANDS.filter(c => c.category === 'info')
    expect(infoCmds.length).toBeGreaterThanOrEqual(5)
  })
})

describe('CATEGORY_ICONS and CATEGORY_LABELS', () => {
  it('should have icon for each category', () => {
    const cats = new Set(SLASH_COMMANDS.map(c => c.category))
    for (const cat of cats) {
      expect(CATEGORY_ICONS).toHaveProperty(cat)
    }
  })

  it('should have label for each category', () => {
    const cats = new Set(SLASH_COMMANDS.map(c => c.category))
    for (const cat of cats) {
      expect(CATEGORY_LABELS).toHaveProperty(cat)
    }
  })
})

// ── useSlashCommands() ─────────────────────────────────────────────────────

describe('useSlashCommands()', () => {
  let options: Record<string, any>
  let slash: ReturnType<typeof useSlashCommands>

  beforeEach(() => {
    vi.clearAllMocks()
    options = makeOptions()
    slash = useSlashCommands(options as any)
  })

  // ── Initial state ─────────────────────────────────────────────────────

  describe('initial state', () => {
    it('should start with menu hidden', () => {
      expect(slash.isSlashMenuVisible.value).toBe(false)
    })

    it('should start with index 0', () => {
      expect(slash.slashMenuIndex.value).toBe(0)
    })

    it('should start with empty filtered commands', () => {
      expect(slash.filteredSlashCommands.value).toEqual([])
    })
  })

  // ── getSlashPrefix ─────────────────────────────────────────────────────

  describe('getSlashPrefix', () => {
    it('should return empty string for empty input', () => {
      expect(slash.getSlashPrefix('')).toBe('')
    })

    it('should return empty string for text without slash', () => {
      expect(slash.getSlashPrefix('hello world')).toBe('')
    })

    it('should return the prefix for a slash command', () => {
      expect(slash.getSlashPrefix('/help')).toBe('/help')
    })

    it('should return the prefix for partial slash command', () => {
      expect(slash.getSlashPrefix('/he')).toBe('/he')
    })

    it('should return only the first token', () => {
      expect(slash.getSlashPrefix('/new some args')).toBe('/new')
    })

    it('should be case-insensitive (lowercase)', () => {
      expect(slash.getSlashPrefix('/HELP')).toBe('/help')
      expect(slash.getSlashPrefix('/New')).toBe('/new')
    })

    it('should return the slash for just a slash', () => {
      expect(slash.getSlashPrefix('/')).toBe('/')
    })

    it('should handle leading spaces', () => {
      expect(slash.getSlashPrefix('  /help')).toBe('/help')
    })
  })

  // ── updateInputText + filteredSlashCommands ─────────────────────────────

  describe('updateInputText + filteredSlashCommands', () => {
    it('should hide menu for empty input', () => {
      slash.isSlashMenuVisible.value = true
      slash.updateInputText('')
      expect(slash.isSlashMenuVisible.value).toBe(false)
    })

    it('should hide menu for non-slash input', () => {
      slash.isSlashMenuVisible.value = true
      slash.updateInputText('hello')
      expect(slash.isSlashMenuVisible.value).toBe(false)
    })

    it('should show menu and return all commands when typing just /', () => {
      const matches = filterWith('/', slash)
      expect(slash.isSlashMenuVisible.value).toBe(true)
      expect(matches.length).toBeGreaterThanOrEqual(30)
    })

    it('should filter commands by prefix /h', () => {
      const matches = filterWith('/h', slash)
      expect(matches.every(c => c.name.startsWith('/h'))).toBe(true)
      expect(matches.map(c => c.name)).toContain('/help')
    })

    it('should find exact match for /new', () => {
      const matches = filterWith('/new', slash)
      expect(matches).toHaveLength(1)
      expect(matches[0].name).toBe('/new')
    })

    it('should keep menu visible when exact match has trailing whitespace only', () => {
      // Code uses text.trim() to check afterCmd — trailing whitespace is removed,
      // so the menu stays visible until user actually types argument characters.
      slash.updateInputText('/new ')
      expect(slash.isSlashMenuVisible.value).toBe(true)
    })

    it('should hide menu when exact match has trailing args', () => {
      slash.updateInputText('/new my-chat')
      expect(slash.isSlashMenuVisible.value).toBe(false)
    })

    it('should hide menu for unknown slash command', () => {
      slash.updateInputText('/xyz')
      expect(slash.isSlashMenuVisible.value).toBe(false)
    })

    it('should reset index to 0 when updating text', () => {
      slash.slashMenuIndex.value = 5
      filterWith('/h', slash)
      expect(slash.slashMenuIndex.value).toBe(0)
    })
  })

  // ── handleSlashKeydown ─────────────────────────────────────────────────

  describe('handleSlashKeydown', () => {
    it('should return false when menu not visible', () => {
      const result = slash.handleSlashKeydown(new KeyboardEvent('keydown', { key: 'Enter' }))
      expect(result).toBe(false)
    })

    it('should return false and hide menu when no commands match', () => {
      slash.isSlashMenuVisible.value = true
      slash.updateInputText('')
      const result = slash.handleSlashKeydown(new KeyboardEvent('keydown', { key: 'Enter' }))
      expect(result).toBe(false)
    })

    describe('ArrowDown', () => {
      it('should move index down by 1', () => {
        filterWith('/h', slash)
        const prevIdx = slash.slashMenuIndex.value
        const e = new KeyboardEvent('keydown', { key: 'ArrowDown' })
        e.preventDefault = vi.fn()
        e.stopPropagation = vi.fn()
        const consumed = slash.handleSlashKeydown(e)
        expect(consumed).toBe(true)
        const expected = (prevIdx + 1) % slash.filteredSlashCommands.value.length
        expect(slash.slashMenuIndex.value).toBe(expected)
      })

      it('should wrap to first when at last', () => {
        filterWith('/h', slash)
        const count = slash.filteredSlashCommands.value.length
        slash.slashMenuIndex.value = count - 1
        const e = new KeyboardEvent('keydown', { key: 'ArrowDown' })
        e.preventDefault = vi.fn()
        e.stopPropagation = vi.fn()
        slash.handleSlashKeydown(e)
        expect(slash.slashMenuIndex.value).toBe(0)
      })
    })

    describe('ArrowUp', () => {
      it('should move index up by 1', () => {
        filterWith('/h', slash)
        slash.slashMenuIndex.value = 1
        const e = new KeyboardEvent('keydown', { key: 'ArrowUp' })
        e.preventDefault = vi.fn()
        e.stopPropagation = vi.fn()
        const consumed = slash.handleSlashKeydown(e)
        expect(consumed).toBe(true)
        expect(slash.slashMenuIndex.value).toBe(0)
      })

      it('should wrap to last when at first', () => {
        filterWith('/h', slash)
        const count = slash.filteredSlashCommands.value.length
        slash.slashMenuIndex.value = 0
        const e = new KeyboardEvent('keydown', { key: 'ArrowUp' })
        e.preventDefault = vi.fn()
        e.stopPropagation = vi.fn()
        slash.handleSlashKeydown(e)
        expect(slash.slashMenuIndex.value).toBe(count - 1)
      })
    })

    describe('Enter on local command', () => {
      it('should execute local command and hide menu', () => {
        filterWith('/new', slash)
        const matches = slash.filteredSlashCommands.value
        expect(matches).toHaveLength(1)
        expect(matches[0].local).toBe(true)

        const e = new KeyboardEvent('keydown', { key: 'Enter' })
        e.preventDefault = vi.fn()
        e.stopPropagation = vi.fn()
        const consumed = slash.handleSlashKeydown(e)
        expect(consumed).toBe(true)
        expect(options.onNewChat).toHaveBeenCalledTimes(1)
        expect(slash.isSlashMenuVisible.value).toBe(false)
      })
    })

    describe('Enter on non-local command', () => {
      it('should call emitSend and hide menu', () => {
        filterWith('/web', slash)
        const matches = slash.filteredSlashCommands.value
        expect(matches).toHaveLength(1)
        expect(matches[0].local).toBe(false)

        const emitSend = vi.fn()
        const e = new KeyboardEvent('keydown', { key: 'Enter' })
        e.preventDefault = vi.fn()
        e.stopPropagation = vi.fn()
        const consumed = slash.handleSlashKeydown(e, emitSend)
        expect(consumed).toBe(true)
        expect(emitSend).toHaveBeenCalledWith('/web ')
        expect(slash.isSlashMenuVisible.value).toBe(false)
      })
    })

    describe('Tab on non-local command', () => {
      it('should behave like Enter', () => {
        filterWith('/web', slash)
        const emitSend = vi.fn()
        const e = new KeyboardEvent('keydown', { key: 'Tab' })
        e.preventDefault = vi.fn()
        e.stopPropagation = vi.fn()
        const consumed = slash.handleSlashKeydown(e, emitSend)
        expect(consumed).toBe(true)
        expect(emitSend).toHaveBeenCalledWith('/web ')
      })
    })

    describe('Escape', () => {
      it('should hide menu and return true', () => {
        filterWith('/h', slash)
        expect(slash.isSlashMenuVisible.value).toBe(true)
        const e = new KeyboardEvent('keydown', { key: 'Escape' })
        e.preventDefault = vi.fn()
        e.stopPropagation = vi.fn()
        const consumed = slash.handleSlashKeydown(e)
        expect(consumed).toBe(true)
        expect(slash.isSlashMenuVisible.value).toBe(false)
        expect(slash.slashMenuIndex.value).toBe(0)
      })
    })

    it('should fall back to first command when index out of bounds', () => {
      filterWith('/h', slash)
      slash.slashMenuIndex.value = 999 // out of bounds — commands[999] is undefined
      const emitSend = vi.fn()
      const e = new KeyboardEvent('keydown', { key: 'Enter' })
      e.preventDefault = vi.fn()
      e.stopPropagation = vi.fn()
      const consumed = slash.handleSlashKeydown(e, emitSend)
      expect(consumed).toBe(true)
      // First command matching /h is /help (local=true), so addAgentMessage was called
      expect(options.addAgentMessage).toHaveBeenCalled()
      expect(slash.isSlashMenuVisible.value).toBe(false)
    })

    it('should not consume non-menu keys', () => {
      filterWith('/h', slash)
      const e = new KeyboardEvent('keydown', { key: 'a' })
      const consumed = slash.handleSlashKeydown(e)
      expect(consumed).toBe(false)
    })
  })

  // ── executeLocal ───────────────────────────────────────────────────────

  describe('executeLocal', () => {
    it('/new should call onNewChat', async () => {
      const result = await slash.executeLocal('/new')
      expect(result).toBe(true)
      expect(options.onNewChat).toHaveBeenCalledTimes(1)
    })

    it('/clear should call onClear', async () => {
      const result = await slash.executeLocal('/clear')
      expect(result).toBe(true)
      expect(options.onClear).toHaveBeenCalledTimes(1)
    })

    it('/help should generate help text and call addAgentMessage', async () => {
      const result = await slash.executeLocal('/help')
      expect(result).toBe(true)
      expect(options.addAgentMessage).toHaveBeenCalledTimes(1)
      const msg = options.addAgentMessage.mock.calls[0][0] as string
      expect(msg).toContain('Available Commands')
      expect(msg).toContain('/new')
      expect(msg).toContain('/help')
      expect(msg).toContain('local')
    })

    describe('/usage', () => {
      it('should show token stats when available', async () => {
        const optionsWithStats = makeOptions({
          usageStats: { inputTokens: 100, outputTokens: 50, totalTokens: 150 },
        })
        const s = useSlashCommands(optionsWithStats as any)
        const result = await s.executeLocal('/usage')
        expect(result).toBe(true)
        expect(optionsWithStats.addAgentMessage).toHaveBeenCalledTimes(1)
        const msg = optionsWithStats.addAgentMessage.mock.calls[0][0] as string
        expect(msg).toContain('100')
        expect(msg).toContain('50')
        expect(msg).toContain('Token Usage')
      })

      it('should show no-data message when stats are null', async () => {
        const result = await slash.executeLocal('/usage')
        expect(result).toBe(true)
        expect(options.addAgentMessage).toHaveBeenCalledWith(
          'No token usage data available for this session.'
        )
      })

      it('should show no-data message when stats are undefined (omitted)', async () => {
        const opts = makeOptions()
        delete opts.usageStats
        opts.addAgentMessage = vi.fn()
        const s = useSlashCommands(opts as any)
        const result = await s.executeLocal('/usage')
        expect(result).toBe(true)
        expect(opts.addAgentMessage).toHaveBeenCalledWith(
          'No token usage data available for this session.'
        )
      })
    })

    it('/model should show help message', async () => {
      const result = await slash.executeLocal('/model')
      expect(result).toBe(true)
      expect(options.addAgentMessage).toHaveBeenCalledWith(
        'Use `/model <name>` to switch the active model.'
      )
    })

    it('/version should show version info message', async () => {
      const result = await slash.executeLocal('/version')
      expect(result).toBe(true)
      expect(options.addAgentMessage).toHaveBeenCalledWith(
        'Version info: use the settings page to check version details.'
      )
    })

    it('should return false for unknown command', async () => {
      const result = await slash.executeLocal('/unknown')
      expect(result).toBe(false)
    })

    it('should return false for empty string', async () => {
      const result = await slash.executeLocal('')
      expect(result).toBe(false)
    })

    it('should handle command with extra arguments', async () => {
      const result = await slash.executeLocal('/new my-chat')
      expect(result).toBe(true)
      expect(options.onNewChat).toHaveBeenCalledTimes(1)
    })

    it('should be case-insensitive', async () => {
      await slash.executeLocal('/NEW')
      expect(options.onNewChat).toHaveBeenCalledTimes(1)
    })
  })

  // ── hideSlashMenu ──────────────────────────────────────────────────────

  describe('hideSlashMenu', () => {
    it('should hide menu and reset index', () => {
      slash.isSlashMenuVisible.value = true
      slash.slashMenuIndex.value = 5
      slash.hideSlashMenu()
      expect(slash.isSlashMenuVisible.value).toBe(false)
      expect(slash.slashMenuIndex.value).toBe(0)
    })

    it('should be idempotent', () => {
      slash.hideSlashMenu()
      slash.hideSlashMenu()
      expect(slash.isSlashMenuVisible.value).toBe(false)
      expect(slash.slashMenuIndex.value).toBe(0)
    })
  })

  // ── Edge cases ─────────────────────────────────────────────────────────

  describe('edge cases', () => {
    it('should filter correctly with mixed case', () => {
      const matches = filterWith('/HEL', slash)
      expect(matches.every(c => c.name.startsWith('/hel'))).toBe(true)
      expect(matches.map(c => c.name)).toContain('/help')
    })

    it('should handle text with multiple slashes (only first is command)', () => {
      filterWith('/new /help', slash)
      // first token is /new, trailing content exists, so menu hides
      expect(slash.isSlashMenuVisible.value).toBe(false)
    })

    it('should handle just a slash followed by space', () => {
      const matches = filterWith('/ ', slash)
      expect(slash.isSlashMenuVisible.value).toBe(true)
      expect(matches.length).toBeGreaterThanOrEqual(30)
    })

    it('should show menu with all commands when typing /, then narrow by adding letters', () => {
      const all = filterWith('/', slash)
      expect(all.length).toBeGreaterThanOrEqual(30)

      const narrowed = filterWith('/h', slash)
      expect(narrowed.length).toBeLessThan(all.length)
      expect(narrowed.every(c => c.name.startsWith('/h'))).toBe(true)
    })
  })
})
