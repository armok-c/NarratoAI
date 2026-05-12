import { computed } from 'vue'
import { useTheme as useVuetifyTheme } from 'vuetify'
import { useAppStore } from '@/stores/app'

export function useTheme() {
  const appStore = useAppStore()
  const vuetifyTheme = useVuetifyTheme()

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
