<template>
  <div class="monitor-container d-flex align-center ga-2 px-4">
    <div class="d-flex align-center" style="gap: 6px">
      <v-icon size="16" class="text-medium-emphasis">mdi-chip</v-icon>
      <span class="text-caption text-medium-emphasis">
        {{ error ? 'CPU --%' : `CPU ${cpuPercent}%` }}
      </span>
    </div>
    <div class="d-flex align-center" style="gap: 6px">
      <v-icon size="16" class="text-medium-emphasis">mdi-memory</v-icon>
      <span class="text-caption text-medium-emphasis">
        {{ error ? 'RAM --%' : `RAM ${ramPercent}%` }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { tauriInvoke } from '@/composables/useTauri'

const cpuPercent = ref(0)
const ramPercent = ref(0)
const error = ref(false)
let intervalId: ReturnType<typeof setInterval> | null = null

async function fetchStats() {
  try {
    const stats = await tauriInvoke<{ cpu_percent: number; ram_percent: number }>('get_system_stats')
    cpuPercent.value = Math.round(stats.cpu_percent)
    ramPercent.value = Math.round(stats.ram_percent)
    error.value = false
  } catch {
    error.value = true
  }
}

onMounted(() => {
  fetchStats()
  intervalId = setInterval(fetchStats, 2000)
})

onUnmounted(() => {
  if (intervalId) clearInterval(intervalId)
})
</script>

<style scoped>
.monitor-container {
  background: rgba(128, 128, 128, 0.15);
  border-radius: 4px;
  height: 100%;
  display: flex;
  align-items: center;
}
</style>
