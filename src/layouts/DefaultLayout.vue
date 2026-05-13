<template>
  <v-layout class="default-layout" style="min-height: 100vh">
    <AppHeader
      v-model:show-settings="showSettings"
      :is-maximized="isMaximized"
      @minimize="handleMinimize"
      @maximize="handleToggleMaximize"
      @close="handleClose"
      @toggle-maximize="handleToggleMaximize"
    />
    <v-main>
      <router-view />
    </v-main>
    <v-footer height="28" class="pa-0 d-flex align-center justify-end">
      <SystemMonitorBar />
    </v-footer>
    <SettingsDrawer v-model="showSettings" />
  </v-layout>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useTheme } from 'vuetify'
import AppHeader from '@/components/AppHeader.vue'
import SystemMonitorBar from '@/components/SystemMonitorBar.vue'
import SettingsDrawer from '@/components/SettingsDrawer.vue'
import { useAppStore } from '@/stores/app'

const showSettings = ref(false)
const isMaximized = ref(false)
let unlistenResize: (() => void) | null = null

const appStore = useAppStore()
const vuetifyTheme = useTheme()

onMounted(async () => {
  vuetifyTheme.global.name.value = appStore.theme

  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    const win = getCurrentWindow()
    isMaximized.value = await win.isMaximized()
    unlistenResize = await win.onResized(async () => {
      isMaximized.value = await win.isMaximized()
    })
  } catch {
    // Non-Tauri environment — keep default false
  }
})

onUnmounted(() => {
  unlistenResize?.()
})

async function handleMinimize() {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    await getCurrentWindow().minimize()
  } catch {
    // Non-Tauri environment fallback
  }
}

async function handleToggleMaximize() {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    const win = getCurrentWindow()
    await win.toggleMaximize()
    isMaximized.value = await win.isMaximized()
  } catch {
    // Non-Tauri environment fallback
  }
}

async function handleClose() {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    await getCurrentWindow().close()
  } catch {
    // Non-Tauri environment fallback
  }
}
</script>

<style scoped>
.default-layout {
  /* Default layout container — three zones managed by Vuetify v-layout */
}
</style>
