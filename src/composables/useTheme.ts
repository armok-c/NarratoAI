import { computed } from 'vue'
import { useTheme as useVuetifyTheme } from 'vuetify'
import { useAppStore } from '@/stores/app'

export function useTheme() {
  const appStore = useAppStore()
  const vuetifyTheme = useVuetifyTheme()

  // 初始化时同步 Vuetify 主题到 app store 保存的值
  vuetifyTheme.global.name.value = appStore.theme

  function toggleTheme() {
    appStore.toggleTheme()
    vuetifyTheme.global.name.value = appStore.theme
  }

  function setTheme(val: 'light' | 'dark') {
    appStore.setTheme(val)
    vuetifyTheme.global.name.value = val
  }

  return {
    theme: computed(() => appStore.theme),
    toggleTheme,
    setTheme,
  }
}
