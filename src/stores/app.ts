import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useAppStore = defineStore('app', () => {
  const theme = ref<'light' | 'dark'>(
    (localStorage.getItem('theme') as 'light' | 'dark') || 'light'
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
