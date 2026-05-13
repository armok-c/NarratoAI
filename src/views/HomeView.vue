<template>
  <div class="home-view d-flex flex-column pa-4" style="height: 100%">
    <!-- Upper section: 60% - VideoTable placeholder + ModeSelector card -->
    <div class="d-flex ga-4 upper-section" style="flex: 6">
      <!-- Left: VideoTable placeholder (flex: 6 = 60%) -->
      <div
        class="placeholder-card d-flex flex-column align-center justify-center"
        style="flex: 6; min-height: 200px"
      >
        <v-icon size="48" class="mb-2" style="opacity: 0.3">mdi-video-outline</v-icon>
        <span class="text-body-2 font-weight-medium" style="opacity: 0.6">视频列表</span>
        <span class="text-caption" style="opacity: 0.4">Phase 17 实现</span>
      </div>

      <!-- Right: ModeSelector card (flex: 4 = 40%) -->
      <div style="flex: 4">
        <v-card variant="outlined" class="pa-4">
          <v-card-title class="pa-0 mb-3 d-flex align-center text-subtitle-2 font-weight-medium">
            <v-icon class="mr-2">mdi-folder-multiple</v-icon>
            工作模式
          </v-card-title>
          <v-select
            v-model="selectedMode"
            :items="modeOptions"
            variant="outlined"
            density="compact"
            hide-details
            @update:model-value="handleModeChange"
          />
        </v-card>
      </div>
    </div>

    <!-- Lower section: 40% - 操作按钮 placeholder + LogPanel placeholder -->
    <div class="d-flex ga-4 lower-section" style="flex: 4">
      <!-- Left: 操作按钮 placeholder (flex: 6 = 60%) -->
      <div
        class="placeholder-card d-flex flex-column align-center justify-center"
        style="flex: 6; min-height: 120px"
      >
        <v-icon size="48" class="mb-2" style="opacity: 0.3">mdi-play-circle-outline</v-icon>
        <span class="text-body-2 font-weight-medium" style="opacity: 0.6">操作按钮</span>
        <span class="text-caption" style="opacity: 0.4">Phase 17 实现</span>
      </div>

      <!-- Right: LogPanel placeholder (flex: 4 = 40%) -->
      <div
        class="placeholder-card d-flex flex-column align-center justify-center"
        style="flex: 4; min-height: 120px"
      >
        <v-icon size="48" class="mb-2" style="opacity: 0.3">mdi-text-box-outline</v-icon>
        <span class="text-body-2 font-weight-medium" style="opacity: 0.6">日志面板</span>
        <span class="text-caption" style="opacity: 0.4">Phase 17 实现</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useModeStore, type WorkMode } from '@/stores/mode'
import { usePipelineStore } from '@/stores/pipeline'

const modeStore = useModeStore()
const pipelineStore = usePipelineStore()

const selectedMode = ref<WorkMode>(modeStore.currentMode)

const modeOptions = ref([
  { title: '纪录片解说', value: 'documentary' },
  { title: '短剧解说', value: 'sde' },
  { title: '短剧混剪', value: 'sdp' },
])

function handleModeChange(val: WorkMode) {
  modeStore.setMode(val)
  pipelineStore.reset()
}
</script>

<style>
.home-view {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.placeholder-card {
  border: 2px dashed rgba(var(--v-theme-on-surface), 0.2);
  border-radius: 8px;
  background: rgba(var(--v-theme-on-surface), 0.02);
}
</style>
