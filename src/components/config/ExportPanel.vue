<template>
  <SettingSection
    icon="mdi-file-export"
    title="导出设置"
    collapsible
    :default-expanded="false"
    :loading="loading"
    :badge-count="badgeCount"
  >
    <template #default>
      <div class="d-flex align-center ga-2 mb-3">
        <v-text-field
          :model-value="store.outputDir"
          label="输出目录"
          variant="outlined"
          density="compact"
          readonly
          hide-details
          class="flex-grow-1"
        />
        <v-btn variant="outlined" size="small" @click="selectExportDir">浏览</v-btn>
      </div>

      <v-select
        :model-value="store.format"
        :items="FORMATS"
        label="导出格式"
        variant="outlined"
        density="compact"
        hide-details
        @update:model-value="store.setFormat($event)"
      />
    </template>
  </SettingSection>
</template>

<script setup lang="ts">
import { useExportStore } from '@/stores/export'
import SettingSection from '@/components/SettingSection.vue'

const props = withDefaults(defineProps<{ badgeCount?: number }>(), { badgeCount: 0 })

const FORMATS = ['mp4', 'mkv', 'mov']

const store = useExportStore()
const { loading } = store

async function selectExportDir() {
  try {
    // @ts-expect-error - @tauri-apps/plugin-dialog may not be installed in dev mode
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({ directory: true, title: '选择输出目录' }) as string | null
    if (selected) {
      store.setOutputDir(selected)
    }
  } catch (err) {
    console.warn('[ExportPanel] 无法打开文件夹选择对话框:', err)
  }
}
</script>
