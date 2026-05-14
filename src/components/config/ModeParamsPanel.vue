<template>
  <SettingSection
    icon="mdi-tune"
    :title="panelTitle"
    collapsible
    :default-expanded="false"
  >
    <template #header-actions>
      <v-btn variant="text" size="small" @click="handleReset">重置</v-btn>
    </template>
    <template #default>
      <!-- Documentary: 帧参数 -->
      <template v-if="currentMode === 'documentary'">
        <v-text-field
          v-model.number="params.frameInterval"
          label="帧间隔"
          type="number"
          min="1"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
        <v-text-field
          v-model.number="params.visionBatchSize"
          label="视觉批处理大小"
          type="number"
          min="1"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
      </template>

      <!-- SDE: 短剧参数 -->
      <template v-else-if="currentMode === 'sde'">
        <v-text-field
          v-model="params.dramaName"
          label="剧名"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
        <v-label class="mb-1">温度</v-label>
        <v-slider
          v-model="params.temperature"
          :min="0"
          :max="2"
          :step="0.1"
          color="primary"
          class="mb-3"
          thumb-label
        />
      </template>

      <!-- SDP: 混剪参数 -->
      <template v-else-if="currentMode === 'sdp'">
        <v-text-field
          v-model.number="params.clipCount"
          label="自定义片段数"
          type="number"
          min="1"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
        <v-text-field
          v-model.number="params.minDuration"
          label="最短时长（秒）"
          type="number"
          min="1"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
        <v-text-field
          v-model.number="params.maxDuration"
          label="最长时长（秒）"
          type="number"
          min="1"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
      </template>
    </template>
  </SettingSection>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useModeStore } from '@/stores/mode'
import SettingSection from '@/components/SettingSection.vue'

const store = useModeStore()
const { currentMode, params } = store

const panelTitle = computed(() => {
  switch (currentMode) {
    case 'documentary': return '帧参数'
    case 'sde': return '短剧参数'
    case 'sdp': return '混剪参数'
    default: return '模式参数'
  }
})

function handleReset() {
  store.resetParams()
}
</script>
