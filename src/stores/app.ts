import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useAppStore = defineStore('app', () => {
  const stored = localStorage.getItem('theme')
  const theme = ref<'light' | 'dark'>(
    stored === 'light' || stored === 'dark' ? stored : 'light'
  )

  function toggleTheme() {
    theme.value = theme.value === 'light' ? 'dark' : 'light'
    localStorage.setItem('theme', theme.value)
  }

  function setTheme(val: 'light' | 'dark') {
    theme.value = val
    localStorage.setItem('theme', val)
  }

  return {
    theme,
    toggleTheme,
    setTheme,
  }
})
