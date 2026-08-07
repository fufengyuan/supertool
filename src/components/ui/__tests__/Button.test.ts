// @vitest-environment happy-dom
import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import Button from '../Button.vue'

describe('Button.vue', () => {
  it('should render with default props', () => {
    const wrapper = mount(Button, {
      slots: { default: 'Click me' },
    })
    expect(wrapper.text()).toContain('Click me')
    expect(wrapper.attributes('type')).toBe('button')
  })

  it('should emit click event when clicked', async () => {
    const wrapper = mount(Button, {
      slots: { default: 'Click' },
    })
    await wrapper.trigger('click')
    expect(wrapper.emitted('click')).toBeTruthy()
    expect(wrapper.emitted('click')!.length).toBe(1)
  })

  it('should not emit click when disabled', async () => {
    const wrapper = mount(Button, {
      props: { disabled: true },
      slots: { default: 'Disabled' },
    })
    await wrapper.trigger('click')
    expect(wrapper.emitted('click')).toBeFalsy()
  })

  it('should apply disabled attribute when disabled prop is true', () => {
    const wrapper = mount(Button, {
      props: { disabled: true },
      slots: { default: 'Disabled' },
    })
    expect(wrapper.attributes('disabled')).toBeDefined()
  })

  it('should show loading spinner and disable interaction when loading', async () => {
    const wrapper = mount(Button, {
      props: { loading: true },
      slots: { default: 'Saving...' },
    })
    expect(wrapper.find('.loading-spinner').exists()).toBe(true)
    expect(wrapper.attributes('disabled')).toBeDefined()

    await wrapper.trigger('click')
    expect(wrapper.emitted('click')).toBeFalsy()
  })

  it('should render with sm size class', () => {
    const wrapper = mount(Button, {
      props: { size: 'sm' },
      slots: { default: 'Small' },
    })
    expect(wrapper.classes()).toContain('btn-sm')
  })

  it('should not have sm class for md size', () => {
    const wrapper = mount(Button, {
      props: { size: 'md' },
      slots: { default: 'Medium' },
    })
    expect(wrapper.classes()).not.toContain('btn-sm')
  })

  it('should render as anchor tag when tag prop is "a"', () => {
    const wrapper = mount(Button, {
      props: { tag: 'a' },
      slots: { default: 'Link' },
    })
    expect(wrapper.element.tagName).toBe('A')
  })

  it('should render as button by default', () => {
    const wrapper = mount(Button, {
      slots: { default: 'Btn' },
    })
    expect(wrapper.element.tagName).toBe('BUTTON')
  })

  it('should apply correct variant class for primary variant', () => {
    const wrapper = mount(Button, {
      props: { variant: 'primary' },
      slots: { default: 'Primary' },
    })
    expect(wrapper.classes()).toContain('btn-primary')
  })

  it('should apply correct variant class for danger variant', () => {
    const wrapper = mount(Button, {
      props: { variant: 'danger' },
      slots: { default: 'Danger' },
    })
    expect(wrapper.classes()).toContain('btn-error')
  })

  it('should apply correct variant class for success variant', () => {
    const wrapper = mount(Button, {
      props: { variant: 'success' },
      slots: { default: 'Success' },
    })
    expect(wrapper.classes()).toContain('btn-success')
  })

  it('should apply correct variant class for warning variant', () => {
    const wrapper = mount(Button, {
      props: { variant: 'warning' },
      slots: { default: 'Warning' },
    })
    expect(wrapper.classes()).toContain('btn-warning')
  })

  it('should apply ghost class and outline for ghost variant', () => {
    const wrapper = mount(Button, {
      props: { variant: 'ghost' },
      slots: { default: 'Ghost' },
    })
    expect(wrapper.classes()).toContain('btn-ghost')
    expect(wrapper.classes()).toContain('btn-outline')
  })

  it('should fallback to primary for unknown variant', () => {
    const wrapper = mount(Button, {
      props: { variant: 'unknown' as any },
      slots: { default: 'Unknown' },
    })
    expect(wrapper.classes()).toContain('btn-primary')
  })

  it('should pass through buttonType prop', () => {
    const wrapper = mount(Button, {
      props: { buttonType: 'submit' },
      slots: { default: 'Submit' },
    })
    expect(wrapper.attributes('type')).toBe('submit')
  })

  it('should render slot content with loading spinner', () => {
    const wrapper = mount(Button, {
      props: { loading: true },
      slots: { default: 'Loading' },
    })
    // Both the spinner and the text should be rendered
    expect(wrapper.find('.loading-spinner').exists()).toBe(true)
    expect(wrapper.text()).toContain('Loading')
  })
})
