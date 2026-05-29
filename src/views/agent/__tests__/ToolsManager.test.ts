// @vitest-environment happy-dom
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import ToolsManager from '../ToolsManager.vue'
import type { ToolsetInfo, MCPServerInfo } from '@/types'

// --- Mock tauri-api ---
const mockListToolsets = vi.fn<() => Promise<ToolsetInfo[]>>()
const mockListMcpServers = vi.fn<() => Promise<MCPServerInfo[]>>()
const mockSetToolsetEnabled = vi.fn<(key: string, enabled: boolean) => Promise<void>>()

vi.mock('@/utils/tauri-api', () => ({
  getTauriAPI: () => ({
    listToolsets: mockListToolsets,
    listMcpServers: mockListMcpServers,
    setToolsetEnabled: mockSetToolsetEnabled,
  }),
}))

// Helper to flush pending promises (safe for browser env, avoids process.nextTick)
function flushPromises() {
  return new Promise(resolve => setTimeout(resolve, 0))
}

function sampleToolsets(): ToolsetInfo[] {
  return [
    { key: 'web',           label: 'Web',          description: 'Web search',          enabled: true },
    { key: 'browser',       label: 'Browser',       description: 'Web browsing',       enabled: false },
    { key: 'terminal',      label: 'Terminal',       description: 'Shell commands',      enabled: true },
    { key: 'file',          label: 'File',           description: 'Read/write files',    enabled: false },
    { key: 'code_execution',label: 'Code Execution', description: 'Execute Python code', enabled: true },
    { key: 'vision',        label: 'Vision',         description: 'Image analysis',      enabled: true },
    { key: 'image_gen',     label: 'Image Gen',      description: 'Generate images',     enabled: false },
    { key: 'tts',           label: 'TTS',            description: 'Text to speech',      enabled: true },
    { key: 'skills',        label: 'Skills',         description: 'Load skills',         enabled: true },
    { key: 'memory',        label: 'Memory',          description: 'Persistent memory',   enabled: false },
    { key: 'session_search',label: 'Session Search',  description: 'Search history',       enabled: true },
    { key: 'clarify',       label: 'Clarify',        description: 'Ask questions',       enabled: true },
    { key: 'delegation',    label: 'Delegation',      description: 'Spawn sub-agents',    enabled: false },
    { key: 'cronjob',       label: 'Cron Job',        description: 'Schedule tasks',      enabled: true },
    { key: 'moa',           label: 'MOA',             description: 'Mixture of Agents',   enabled: false },
    { key: 'todo',          label: 'Todo',            description: 'Task list',           enabled: true },
  ]
}

function sampleMcpServers(): MCPServerInfo[] {
  return [
    { name: 'my-api',    type: 'http',  detail: 'http://localhost:8080/api' },
    { name: 'file-tools', type: 'stdio', detail: 'python -m mcp_server' },
  ]
}

// Stub components
const STUBS = {
  SvgIcon: true,
  IconWifi: true,
  IconBrowser: true,
  IconTerminal2: true,
  IconFile: true,
  IconCode: true,
  IconEye: true,
  IconPhoto: true,
  IconMicrophone: true,
  IconTool: true,
  IconBrain: true,
  IconSearch: true,
  IconMessage: true,
  IconUsers: true,
  IconClock: true,
  IconLayoutBottombar: true,
  IconCheckbox: true,
  IconServer: true,
}

describe('ToolsManager.vue', () => {
  beforeEach(() => {
    mockListToolsets.mockReset()
    mockListMcpServers.mockReset()
    mockSetToolsetEnabled.mockReset()
  })

  describe('renders toolsets', () => {
    it('should render all 16 toolset cards when data loads', async () => {
      mockListToolsets.mockResolvedValue(sampleToolsets())
      mockListMcpServers.mockResolvedValue([])

      const wrapper = mount(ToolsManager, {
        global: { stubs: STUBS },
      })

      // Wait for onMounted refresh
      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(mockListToolsets).toHaveBeenCalledTimes(1)
      expect(mockListMcpServers).toHaveBeenCalledTimes(1)
    })
  })

  describe('with empty MCP servers', () => {
    it('should show empty MCP message when no servers configured', async () => {
      mockListToolsets.mockResolvedValue(sampleToolsets())
      mockListMcpServers.mockResolvedValue([])

      const wrapper = mount(ToolsManager, {
        global: { stubs: STUBS },
      })

      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('未配置 MCP 服务器')
    })
  })

  describe('with MCP servers', () => {
    it('should render MCP server cards', async () => {
      mockListToolsets.mockResolvedValue(sampleToolsets())
      mockListMcpServers.mockResolvedValue(sampleMcpServers())

      const wrapper = mount(ToolsManager, {
        global: { stubs: STUBS },
      })

      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('my-api')
      expect(wrapper.text()).toContain('file-tools')
      expect(wrapper.text()).toContain('http://localhost:8080/api')
      expect(wrapper.text()).toContain('python -m mcp_server')
      expect(wrapper.text()).toContain('MCP 服务器 (2)')
    })

    it('should show type badges for http and stdio', async () => {
      mockListToolsets.mockResolvedValue(sampleToolsets())
      mockListMcpServers.mockResolvedValue(sampleMcpServers())

      const wrapper = mount(ToolsManager, {
        global: { stubs: STUBS },
      })

      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('http')
      expect(wrapper.text()).toContain('stdio')
    })
  })

  describe('tool toggle behavior', () => {
    it('should call setToolsetEnabled when toggling a tool on', async () => {
      mockListToolsets.mockResolvedValue(sampleToolsets())
      mockListMcpServers.mockResolvedValue([])
      mockSetToolsetEnabled.mockResolvedValue(undefined)

      const wrapper = mount(ToolsManager, {
        global: { stubs: STUBS },
      })

      await flushPromises()
      await wrapper.vm.$nextTick()

      const checkboxes = wrapper.findAll('input[type="checkbox"]')
      expect(checkboxes.length).toBeGreaterThan(0)

      // First tool: web (enabled=true), clicking disables it
      await checkboxes[0].trigger('change')

      expect(mockSetToolsetEnabled).toHaveBeenCalledWith('web', false)
    })

    it('should call setToolsetEnabled when toggling a tool off', async () => {
      mockListToolsets.mockResolvedValue(sampleToolsets())
      mockListMcpServers.mockResolvedValue([])
      mockSetToolsetEnabled.mockResolvedValue(undefined)

      const wrapper = mount(ToolsManager, {
        global: { stubs: STUBS },
      })

      await flushPromises()
      await wrapper.vm.$nextTick()

      const checkboxes = wrapper.findAll('input[type="checkbox"]')
      // Index 1 is browser (enabled=false), clicking enables it
      await checkboxes[1].trigger('change')

      expect(mockSetToolsetEnabled).toHaveBeenCalledWith('browser', true)
    })

    it('should show error message when toggle fails', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      mockListToolsets.mockResolvedValue(sampleToolsets())
      mockListMcpServers.mockResolvedValue([])
      mockSetToolsetEnabled.mockRejectedValue(new Error('API error'))

      const wrapper = mount(ToolsManager, {
        global: { stubs: STUBS },
      })

      await flushPromises()
      await wrapper.vm.$nextTick()

      const checkboxes = wrapper.findAll('input[type="checkbox"]')
      await checkboxes[0].trigger('change')

      // Wait for the async toggle to fail
      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(consoleSpy).toHaveBeenCalled()
      expect(mockSetToolsetEnabled).toHaveBeenCalledWith('web', false)
      consoleSpy.mockRestore()
    })
  })

  describe('refresh button', () => {
    it('should reload data when refresh button is clicked', async () => {
      mockListToolsets.mockResolvedValue(sampleToolsets())
      mockListMcpServers.mockResolvedValue([])

      const wrapper = mount(ToolsManager, {
        global: { stubs: STUBS },
      })

      await flushPromises()
      await wrapper.vm.$nextTick()

      // Reset call counts and set new data
      mockListToolsets.mockReset()
      mockListMcpServers.mockReset()
      mockListToolsets.mockResolvedValue(sampleToolsets())
      mockListMcpServers.mockResolvedValue([])

      // Find and click the refresh button
      const refreshBtn = wrapper.find('button')
      expect(refreshBtn.exists()).toBe(true)
      await refreshBtn.trigger('click')

      // Wait for the async refresh
      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(mockListToolsets).toHaveBeenCalledTimes(1)
      expect(mockListMcpServers).toHaveBeenCalledTimes(1)
    })
  })

  describe('error handling on load', () => {
    it('should handle listToolsets failure gracefully', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      mockListToolsets.mockRejectedValue(new Error('Network error'))
      mockListMcpServers.mockResolvedValue([])

      const wrapper = mount(ToolsManager, {
        global: { stubs: STUBS },
      })

      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(consoleSpy).toHaveBeenCalled()
      expect(wrapper.find('h1').text()).toBe('工具集管理')
      consoleSpy.mockRestore()
    })

    it('should handle listMcpServers failure gracefully', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      mockListToolsets.mockResolvedValue(sampleToolsets())
      mockListMcpServers.mockRejectedValue(new Error('Network error'))

      const wrapper = mount(ToolsManager, {
        global: { stubs: STUBS },
      })

      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(consoleSpy).toHaveBeenCalled()
      expect(wrapper.text()).toContain('工具集管理')
      consoleSpy.mockRestore()
    })
  })

  describe('renders toolset product texts', () => {
    it('should show toolset count header and labels', async () => {
      mockListToolsets.mockResolvedValue(sampleToolsets())
      mockListMcpServers.mockResolvedValue([])

      const wrapper = mount(ToolsManager, {
        global: { stubs: STUBS },
      })

      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('平台工具集 (16)')
      expect(wrapper.text()).toContain('Web')
      expect(wrapper.text()).toContain('Browser')
      expect(wrapper.text()).toContain('Terminal')
      expect(wrapper.text()).toContain('File')
      expect(wrapper.text()).toContain('Web search')
      expect(wrapper.text()).toContain('Shell commands')
    })
  })

  describe('loads data on mount', () => {
    it('should call listToolsets and listMcpServers on mount', async () => {
      mockListToolsets.mockResolvedValue(sampleToolsets())
      mockListMcpServers.mockResolvedValue([])

      mount(ToolsManager, {
        global: { stubs: STUBS },
      })

      expect(mockListToolsets).toHaveBeenCalled()
      expect(mockListMcpServers).toHaveBeenCalled()
    })
  })

  describe('disabled tool visual state', () => {
    it('should apply opacity-50 class to disabled tools', async () => {
      mockListToolsets.mockResolvedValue(sampleToolsets())
      mockListMcpServers.mockResolvedValue([])

      const wrapper = mount(ToolsManager, {
        global: { stubs: STUBS },
      })

      await flushPromises()
      await wrapper.vm.$nextTick()

      const cards = wrapper.findAll('.grid.grid-cols-1 > div')
      // Find the disabled tool cards
      const disabledCards = cards.filter(c => c.classes().includes('opacity-50'))
      expect(disabledCards.length).toBeGreaterThan(0)

      // browser is disabled in sample data
      const browserCard = cards.find(c => c.text().includes('Browser'))
      expect(browserCard?.classes()).toContain('opacity-50')
    })

    it('should show Disabled badge on disabled tools', async () => {
      mockListToolsets.mockResolvedValue(sampleToolsets())
      mockListMcpServers.mockResolvedValue([])

      const wrapper = mount(ToolsManager, {
        global: { stubs: STUBS },
      })

      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('Disabled')
      // browser is disabled, should show Disabled badge near it
      expect(wrapper.html()).toContain('Disabled')
    })
  })

  describe('tool icon fallback', () => {
    it('should fall back to IconTool for unknown tool keys', async () => {
      const toolsWithUnknown: ToolsetInfo[] = [
        { key: 'unknown_tool', label: 'Unknown', description: 'Some tool', enabled: true },
        { key: 'web', label: 'Web', description: 'Web search', enabled: true },
      ]
      mockListToolsets.mockResolvedValue(toolsWithUnknown)
      mockListMcpServers.mockResolvedValue([])

      const wrapper = mount(ToolsManager, {
        global: { stubs: STUBS },
      })

      await flushPromises()
      await wrapper.vm.$nextTick()

      // Component should render without error for unknown key
      expect(wrapper.text()).toContain('Unknown')
      expect(wrapper.text()).toContain('Web')
    })
  })

  describe('empty toolsets', () => {
    it('should show header with zero count when no toolsets returned', async () => {
      mockListToolsets.mockResolvedValue([])
      mockListMcpServers.mockResolvedValue([])

      const wrapper = mount(ToolsManager, {
        global: { stubs: STUBS },
      })

      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('平台工具集 (0)')
    })
  })

  describe('MCP server unknown type styling', () => {
    it('should render MCP server with unknown type using fallback style', async () => {
      mockListToolsets.mockResolvedValue(sampleToolsets())
      const serversWithUnknown: MCPServerInfo[] = [
        { name: 'custom-server', type: 'other', detail: 'ws://localhost:9090' },
      ]
      mockListMcpServers.mockResolvedValue(serversWithUnknown)

      const wrapper = mount(ToolsManager, {
        global: { stubs: STUBS },
      })

      await flushPromises()
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('other')
      expect(wrapper.text()).toContain('custom-server')
      expect(wrapper.text()).toContain('ws://localhost:9090')
    })
  })

  describe('optimistic toggle update', () => {
    it('should update tool.enabled after successful toggle', async () => {
      mockListToolsets.mockResolvedValue(sampleToolsets())
      mockListMcpServers.mockResolvedValue([])
      mockSetToolsetEnabled.mockResolvedValue(undefined)

      const wrapper = mount(ToolsManager, {
        global: { stubs: STUBS },
      })

      await flushPromises()
      await wrapper.vm.$nextTick()

      // browser is disabled (enabled=false), toggle should enable it
      const checkboxes = wrapper.findAll('input[type="checkbox"]')
      const browserCheckbox = checkboxes[1] // index 1 is browser in sample data
      await browserCheckbox.trigger('change')

      await flushPromises()
      await wrapper.vm.$nextTick()

      // The browser card should no longer have opacity-50 (now enabled)
      const cards = wrapper.findAll('.grid.grid-cols-1 > div')
      const browserCard = cards.find(c => c.text().includes('Browser'))
      expect(browserCard?.classes()).not.toContain('opacity-50')
    })
  })
})
