<script setup lang="ts">
import { ref, computed } from 'vue'
import { useTheme } from '@/composables/useTheme'
import WindowControls from '@/components/WindowControls.vue'

const props = withDefaults(defineProps<{
  isMaximized?: boolean
  showSettings?: boolean
}>(), {
  isMaximized: false,
  showSettings: false,
})

const emit = defineEmits<{
  minimize: []
  maximize: []
  close: []
  toggleMaximize: []
  'update:showSettings': [value: boolean]
}>()

const { theme, toggleTheme } = useTheme()
const isDark = computed(() => theme.value === 'dark')

function toggleSettings() {
  emit('update:showSettings', !props.showSettings)
}

async function startDrag(e: MouseEvent) {
  if (e.button !== 0) return
  const target = e.target as HTMLElement
  if (target.closest('button') || target.closest('.window-controls')) return
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    await getCurrentWindow().startDragging()
  } catch {
    // Non-Tauri environment — no-op
  }
}

function handleDoubleClick() {
  emit('toggleMaximize')
}
</script>

<template>
  <v-app-bar
    density="compact"
    height="40"
    class="app-header"
    @mousedown="startDrag"
    @dblclick="handleDoubleClick"
  >
    <div class="logo-container d-flex align-center ml-2">
      <img src="@/assets/logo.svg" width="28" height="28" class="logo-img" />
      <div class="logo-glow" />
    </div>
    <v-app-bar-title class="text-body-2 font-weight-bold ml-2">
      NarratoAI
    </v-app-bar-title>
    <v-spacer />
    <v-btn
      icon
      variant="text"
      size="small"
      class="app-header-btn"
      @click="toggleSettings"
      aria-label="设置"
    >
      <v-icon>mdi-cog</v-icon>
    </v-btn>
    <v-btn
      icon
      variant="text"
      size="small"
      class="app-header-btn"
      @click="toggleTheme"
      aria-label="切换主题"
    >
      <v-icon>{{ isDark ? 'mdi-weather-sunny' : 'mdi-weather-night' }}</v-icon>
    </v-btn>
    <WindowControls
      :is-maximized="isMaximized"
      @minimize="$emit('minimize')"
      @maximize="$emit('maximize')"
      @close="$emit('close')"
    />
  </v-app-bar>
</template>

<style scoped>
.app-header {
  border-bottom: 1px solid rgba(var(--v-theme-on-surface), 0.12);
  user-select: none;
  cursor: default;
}

.app-header-btn {
  width: 36px;
  height: 36px;
  min-width: 36px;
}

.logo-container {
  position: relative;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.logo-glow {
  position: absolute;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  pointer-events: none;
  background: radial-gradient(circle, rgba(var(--v-theme-primary), 0.25) 0%, transparent 70%);
}

:root[class*="dark"] .logo-glow,
.v-theme--dark .logo-glow {
  background: radial-gradient(circle, rgba(var(--v-theme-primary), 0.4) 0%, transparent 70%);
}
</style>
