import { describe, it, expect } from 'vitest'
import { shallowMount } from '@vue/test-utils'
import SettingSection from '../../src/components/SettingSection.vue'

function mountSection(props: Record<string, unknown>, slots?: Record<string, string>) {
  return shallowMount(SettingSection, {
    props,
    slots,
    global: {
      stubs: {
        'v-icon': { template: '<i class="v-icon">{{ icon }}</i>', props: ['icon'] },
        'v-badge': { template: '<span class="v-badge"><slot /></span>' },
        'v-progress-linear': { template: '<div class="v-progress-linear" />' },
        'v-expand-transition': { template: '<div><slot /></div>' },
      },
    },
  })
}

describe('SettingSection', () => {
  it('renders title text', () => {
    const wrapper = mountSection({ title: '测试面板' })
    expect(wrapper.text()).toContain('测试面板')
  })

  it('renders icon when provided', () => {
    const wrapper = mountSection({ title: '面板', icon: 'mdi-cog' })
    expect(wrapper.find('i.v-icon').exists()).toBe(true)
  })

  it('is expanded by default showing content', () => {
    const wrapper = mountSection({ title: '面板', collapsible: true }, { default: '内容' })
    expect(wrapper.text()).toContain('内容')
  })

  it('collapsible mode toggles content on header click', async () => {
    const wrapper = mountSection({ title: '面板', collapsible: true }, { default: '可折叠内容' })
    const header = wrapper.find('.setting-section-header')

    // Default: expanded
    expect(wrapper.text()).toContain('可折叠内容')

    // Click to collapse
    await header.trigger('click')
    // isExpanded becomes false → v-show toggles on the content div
    const content = wrapper.find('.setting-section-content')
    expect(content.attributes('style')).toContain('display: none')
  })

  it('shows progress linear bar when loading prop is true', () => {
    const wrapper = mountSection({ title: '面板', loading: true })
    expect(wrapper.find('.v-progress-linear').exists()).toBe(true)
  })

  it('renders header-actions slot content', () => {
    const wrapper = mountSection(
      { title: '面板' },
      { 'header-actions': '<button class="reset-btn">重置</button>' }
    )
    expect(wrapper.find('.reset-btn').exists()).toBe(true)
    expect(wrapper.text()).toContain('重置')
  })
})
