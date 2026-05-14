<template>
  <SettingSection
    icon="mdi-music-note"
    title="背景音乐"
    collapsible
    :default-expanded="false"
    :loading="loading"
  >
    <template #default>
      <div class="d-flex align-center ga-2 mb-3">
        <v-text-field
          :model-value="store.folder"
          label="BGM 文件夹"
          variant="outlined"
          density="compact"
          readonly
          hide-details
          class="flex-grow-1"
        />
        <v-btn variant="outlined" size="small" @click="selectBgmFolder">浏览</v-btn>
      </div>

      <v-switch
        :model-value="store.mode === 'specified'"
        label="指定"
        false-label="随机"
        color="primary"
        density="compact"
        hide-details
        class="mb-3"
        @update:model-value="store.setMode($event ? 'specified' : 'random')"
      />

      <v-select
        v-if="store.mode === 'specified'"
        v-model="selectedFile"
        :items="store.audioFiles"
        label="选择音频文件"
        variant="outlined"
        density="compact"
        :disabled="!store.folder"
        hide-details
      />
    </template>
  </SettingSection>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useBgmStore } from '@/stores/bgm'
import SettingSection from '@/components/SettingSection.vue'

const store = useBgmStore()
const { loading } = store
const selectedFile = ref('')

async function selectBgmFolder() {
  try {
    // @ts-expect-error - @tauri-apps/plugin-dialog may not be installed in dev mode
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({ directory: true, title: '选择 BGM 文件夹' }) as string | null
    if (selected) {
      store.setFolder(selected)
    }
  } catch {
    // 非 Tauri 环境降级
  }
}
</script>
