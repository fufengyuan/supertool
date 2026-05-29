// @vitest-environment happy-dom
import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import EmptyState from '../EmptyState.vue'

describe('EmptyState.vue', () => {
  it('should render default text when no props provided', () => {
    const wrapper = mount(EmptyState)
    expect(wrapper.text()).toContain('暂无内容')
  })

  it('should render custom text prop', () => {
    const wrapper = mount(EmptyState, {
      props: { text: 'No data available' },
    })
    expect(wrapper.text()).toContain('No data available')
  })

  it('should render subtext when provided', () => {
    const wrapper = mount(EmptyState, {
      props: {
        text: 'Empty',
        subtext: 'Try adding some items',
      },
    })
    expect(wrapper.text()).toContain('Empty')
    expect(wrapper.text()).toContain('Try adding some items')
  })

  it('should not render subtext when not provided', () => {
    const wrapper = mount(EmptyState, {
      props: { text: 'Nothing here' },
    })
    expect(wrapper.text()).toContain('Nothing here')
    // Default subtext is empty string, so no subtext element
    const subtextEl = wrapper.find('p + p')
    expect(subtextEl.exists()).toBe(false)
  })

  it('should render icon slot content', () => {
    const wrapper = mount(EmptyState, {
      props: { text: 'With custom icon' },
      slots: {
        icon: '<span class="custom-icon">🔍</span>',
      },
    })
    expect(wrapper.find('.custom-icon').exists()).toBe(true)
    expect(wrapper.find('.custom-icon').text()).toBe('🔍')
  })

  it('should render action slot content', () => {
    const wrapper = mount(EmptyState, {
      props: { text: 'With action' },
      slots: {
        action: '<button class="action-btn">Add Item</button>',
      },
    })
    expect(wrapper.find('.action-btn').exists()).toBe(true)
    expect(wrapper.find('.action-btn').text()).toBe('Add Item')
  })

  it('should render default folder icon when no icon slot', () => {
    const wrapper = mount(EmptyState, {
      props: { text: 'Default icon' },
    })
    // The default SVG icon has a viewBox attribute
    const svg = wrapper.find('svg')
    expect(svg.exists()).toBe(true)
    expect(svg.attributes('viewBox')).toBe('0 0 24 24')
  })

  it('should hide default icon when icon slot is provided', () => {
    const wrapper = mount(EmptyState, {
      props: { text: 'Custom icon replaces default' },
      slots: {
        icon: '<span class="my-icon">*</span>',
      },
    })
    // Default SVG icon should NOT render when icon slot is used
    expect(wrapper.find('svg').exists()).toBe(false)
    expect(wrapper.find('.my-icon').exists()).toBe(true)
  })

  it('should render empty subtext as empty string and not show subtext element', () => {
    const wrapper = mount(EmptyState, {
      props: { text: 'Test', subtext: '' },
    })
    // Only one <p> tag (the text), no subtext <p>
    const paragraphs = wrapper.findAll('p')
    expect(paragraphs.length).toBe(1)
    expect(paragraphs[0].text()).toBe('Test')
  })
})
