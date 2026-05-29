// @vitest-environment happy-dom
import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import Input from '../Input.vue'

describe('Input.vue', () => {
  it('should render input element by default', () => {
    const wrapper = mount(Input)
    const input = wrapper.find('input')
    expect(input.exists()).toBe(true)
  })

  it('should render textarea when type is textarea', () => {
    const wrapper = mount(Input, {
      props: { type: 'textarea' },
    })
    expect(wrapper.find('textarea').exists()).toBe(true)
    expect(wrapper.find('input').exists()).toBe(false)
  })

  it('should render select when type is select', () => {
    const wrapper = mount(Input, {
      props: { type: 'select' },
      slots: { default: '<option value="1">Option 1</option>' },
    })
    expect(wrapper.find('select').exists()).toBe(true)
    expect(wrapper.find('input').exists()).toBe(false)
  })

  it('should display label when provided', () => {
    const wrapper = mount(Input, {
      props: { label: 'Username' },
    })
    expect(wrapper.text()).toContain('Username')
  })

  it('should show required asterisk when required is true', () => {
    const wrapper = mount(Input, {
      props: { label: 'Email', required: true },
    })
    const legend = wrapper.find('legend')
    expect(legend.exists()).toBe(true)
    expect(legend.find('.text-error').exists()).toBe(true)
  })

  it('should not show label when not provided', () => {
    const wrapper = mount(Input)
    expect(wrapper.find('legend').exists()).toBe(false)
  })

  it('should display placeholder on input', () => {
    const wrapper = mount(Input, {
      props: { placeholder: 'Enter text...' },
    })
    expect(wrapper.find('input').attributes('placeholder')).toBe('Enter text...')
  })

  it('should emit update:modelValue on input change', async () => {
    const wrapper = mount(Input, {
      props: { modelValue: '' },
    })
    const input = wrapper.find('input')
    await input.setValue('hello')
    expect(wrapper.emitted('update:modelValue')).toBeTruthy()
    expect(wrapper.emitted('update:modelValue')![0]).toEqual(['hello'])
  })

  it('should emit update:modelValue on textarea change', async () => {
    const wrapper = mount(Input, {
      props: { type: 'textarea', modelValue: '' },
    })
    const textarea = wrapper.find('textarea')
    await textarea.setValue('multiline text')
    expect(wrapper.emitted('update:modelValue')).toBeTruthy()
    expect(wrapper.emitted('update:modelValue')![0]).toEqual(['multiline text'])
  })

  it('should emit update:modelValue on select change', async () => {
    const wrapper = mount(Input, {
      props: { type: 'select', modelValue: '' },
      slots: { default: '<option value="a">A</option><option value="b">B</option>' },
    })
    const select = wrapper.find('select')
    await select.setValue('b')
    expect(wrapper.emitted('update:modelValue')).toBeTruthy()
    expect(wrapper.emitted('update:modelValue')![0]).toEqual(['b'])
  })

  it('should emit focus and blur events', async () => {
    const wrapper = mount(Input, {
      props: { modelValue: '' },
    })
    const input = wrapper.find('input')

    await input.trigger('focus')
    expect(wrapper.emitted('focus')).toBeTruthy()
    expect(wrapper.emitted('focus')!.length).toBe(1)

    await input.trigger('blur')
    expect(wrapper.emitted('blur')).toBeTruthy()
    expect(wrapper.emitted('blur')!.length).toBe(1)
  })

  it('should apply disabled attribute', () => {
    const wrapper = mount(Input, {
      props: { disabled: true },
    })
    expect(wrapper.find('input').attributes('disabled')).toBeDefined()
  })

  it('should disable textarea when disabled', () => {
    const wrapper = mount(Input, {
      props: { type: 'textarea', disabled: true },
    })
    expect(wrapper.find('textarea').attributes('disabled')).toBeDefined()
  })

  it('should disable select when disabled', () => {
    const wrapper = mount(Input, {
      props: { type: 'select', disabled: true },
    })
    expect(wrapper.find('select').attributes('disabled')).toBeDefined()
  })

  it('should display hint text when provided', () => {
    const wrapper = mount(Input, {
      props: { hint: 'This is a hint' },
    })
    expect(wrapper.text()).toContain('This is a hint')
  })

  it('should not display hint when not provided', () => {
    const wrapper = mount(Input)
    expect(wrapper.text()).not.toContain('fieldset-label')
  })

  it('should render with password type', () => {
    const wrapper = mount(Input, {
      props: { type: 'password' },
    })
    expect(wrapper.find('input').attributes('type')).toBe('password')
  })

  it('should render with number type', () => {
    const wrapper = mount(Input, {
      props: { type: 'number' },
    })
    expect(wrapper.find('input').attributes('type')).toBe('number')
  })

  it('should render with email type', () => {
    const wrapper = mount(Input, {
      props: { type: 'email' },
    })
    expect(wrapper.find('input').attributes('type')).toBe('email')
  })

  it('should render with color type', () => {
    const wrapper = mount(Input, {
      props: { type: 'color' },
    })
    expect(wrapper.find('input').attributes('type')).toBe('color')
  })

  it('should render textarea with specified rows', () => {
    const wrapper = mount(Input, {
      props: { type: 'textarea', rows: 5 },
    })
    expect(wrapper.find('textarea').attributes('rows')).toBe('5')
  })

  it('should bind modelValue to input value', () => {
    const wrapper = mount(Input, {
      props: { modelValue: 'preset value' },
    })
    const input = wrapper.find('input').element as HTMLInputElement
    expect(input.value).toBe('preset value')
  })

  it('should bind modelValue to textarea value', () => {
    const wrapper = mount(Input, {
      props: { type: 'textarea', modelValue: 'preset text' },
    })
    const textarea = wrapper.find('textarea').element as HTMLTextAreaElement
    expect(textarea.value).toBe('preset text')
  })

  it('should handle invalid type by falling back to text input', () => {
    const wrapper = mount(Input, {
      props: { type: 'invalid-type' as any },
    })
    expect(wrapper.find('input').exists()).toBe(true)
    expect(wrapper.find('input').attributes('type')).toBe('invalid-type')
  })
})
