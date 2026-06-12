import { describe, it, expect } from 'vitest'
import { SLASH_COMMANDS } from '../slashCommands'
import type { SlashCommand } from '../slashCommands'

describe('SLASH_COMMANDS data contract', () => {
  it('should have at least 10 commands defined', () => {
    expect(SLASH_COMMANDS.length).toBeGreaterThanOrEqual(10)
  })

  it('every command should have name, description, and category', () => {
    for (const cmd of SLASH_COMMANDS) {
      expect(cmd.name).toBeTruthy()
      expect(cmd.description).toBeTruthy()
      expect(['chat', 'agent', 'tools', 'info']).toContain(cmd.category)
    }
  })

  it('all names should start with /', () => {
    for (const cmd of SLASH_COMMANDS) {
      expect(cmd.name.startsWith('/')).toBe(true)
    }
  })

  it('should have unique names', () => {
    const names = SLASH_COMMANDS.map((c) => c.name)
    expect(new Set(names).size).toBe(names.length)
  })

  it('should have essential commands', () => {
    const names = SLASH_COMMANDS.map((c) => c.name)
    expect(names).toContain('/new')
    expect(names).toContain('/clear')
    expect(names).toContain('/help')
    expect(names).toContain('/model')
    expect(names).toContain('/memory')
  })

  it('should mark local commands appropriately', () => {
    const localCommands = SLASH_COMMANDS.filter((c) => c.local)
    const localNames = localCommands.map((c) => c.name)
    expect(localNames).toContain('/new')
    expect(localNames).toContain('/clear')
  })

  it('should have chat category commands', () => {
    const chatCmds = SLASH_COMMANDS.filter((c) => c.category === 'chat')
    expect(chatCmds.length).toBeGreaterThanOrEqual(2)
    expect(chatCmds.map((c) => c.name)).toContain('/new')
    expect(chatCmds.map((c) => c.name)).toContain('/clear')
  })

  it('should have agent category commands', () => {
    const agentCmds = SLASH_COMMANDS.filter((c) => c.category === 'agent')
    expect(agentCmds.length).toBeGreaterThanOrEqual(5)
  })

  it('should have tools category commands', () => {
    const toolsCmds = SLASH_COMMANDS.filter((c) => c.category === 'tools')
    expect(toolsCmds.length).toBeGreaterThanOrEqual(3)
  })

  it('should have info category commands', () => {
    const infoCmds = SLASH_COMMANDS.filter((c) => c.category === 'info')
    expect(infoCmds.length).toBeGreaterThanOrEqual(3)
    expect(infoCmds.map((c) => c.name)).toContain('/help')
    expect(infoCmds.map((c) => c.name)).toContain('/model')
    expect(infoCmds.map((c) => c.name)).toContain('/memory')
  })
})
