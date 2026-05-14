<template>
  <v-navigation-drawer
    :model-value="modelValue"
    location="right"
    temporary
    width="420"
    @update:model-value="emit('update:modelValue', $event)"
    class="settings-drawer"
  >
    <!-- Header (fixed top) -->
    <div class="settings-drawer__header d-flex align-center pa-4">
      <v-icon class="mr-2">mdi-cog</v-icon>
      <span class="text-h6 font-weight-semibold">设置</span>
      <v-spacer />
      <v-btn
        icon
        variant="text"
        size="small"
        @click="handleClose"
        aria-label="关闭设置"
      >
        <v-icon>mdi-close</v-icon>
      </v-btn>
    </div>

    <!-- Content (scrollable) -->
    <div class="settings-drawer__content pa-4 pt-2">
      <!-- 外观（非折叠，始终可见） -->
      <SettingSection icon="mdi-brightness-6" title="外观" :collapsible="false" class="mb-3">
        <div class="d-flex align-center justify-space-between">
          <div>
            <span class="text-body-2">深色模式</span>
            <br />
            <span class="text-caption text-medium-emphasis">
              {{ isDark ? '深色主题已启用' : '浅色主题已启用' }}
            </span>
          </div>
          <v-switch
            :model-value="isDark"
            color="primary"
            hide-details
            density="compact"
            @update:model-value="toggleTheme"
          />
        </div>
      </SettingSection>

      <!-- Panel 1: LLM Vision（仅 Documentary 模式可见，D-02） -->
      <LlmVisionPanel v-if="modeStore.currentMode === 'documentary'" class="mb-3" />

      <!-- Panel 2: LLM Text（始终可见） -->
      <LlmTextPanel class="mb-3" />

      <!-- Panel 3: TTS（SDP 模式隐藏，D-02） -->
      <TtsPanel v-if="modeStore.currentMode !== 'sdp'" class="mb-3" />

      <!-- Panel 4: BGM（始终可见） -->
      <BgmPanel class="mb-3" />

      <!-- Panel 5: Export（始终可见） -->
      <ExportPanel class="mb-3" />

      <!-- Panel 6: 网络代理（始终可见） -->
      <NetworkProxyPanel class="mb-3" />

      <!-- Panel 7: 模式专用参数（始终可见） -->
      <ModeParamsPanel class="mb-3" />
    </div>

    <!-- Footer (fixed bottom, D-04) -->
    <div class="settings-drawer__footer d-flex flex-column align-center pa-4 ga-2">
      <v-btn
        color="primary"
        variant="elevated"
        block
        disabled
        class="text-none"
      >
        保存设置
      </v-btn>
      <div class="d-flex align-center ga-1">
        <span class="text-caption text-decoration-underline" style="cursor: pointer; opacity: 0.7">
          重置全部
        </span>
        <span class="text-caption" style="opacity: 0.3">|</span>
        <span class="text-caption" style="opacity: 0.6">{{ versionText }}</span>
      </div>
    </div>
  </v-navigation-drawer>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import SettingSection from '@/components/SettingSection.vue'
import { useTheme } from '@/composables/useTheme'
import { useModeStore } from '@/stores/mode'
import LlmVisionPanel from '@/components/config/LlmVisionPanel.vue'
import LlmTextPanel from '@/components/config/LlmTextPanel.vue'
import TtsPanel from '@/components/config/TtsPanel.vue'
import BgmPanel from '@/components/config/BgmPanel.vue'
import ExportPanel from '@/components/config/ExportPanel.vue'
import NetworkProxyPanel from '@/components/config/NetworkProxyPanel.vue'
import ModeParamsPanel from '@/components/config/ModeParamsPanel.vue'

const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const { theme, toggleTheme } = useTheme()
const isDark = computed(() => theme.value === 'dark')
const versionText = ref('v0.1.0')
const modeStore = useModeStore()

onMounted(async () => {
  try {
    const { getVersion } = await import('@tauri-apps/api/app')
    versionText.value = 'v' + (await getVersion())
  } catch {
    versionText.value = 'v0.1.0'
  }
})

function handleClose() {
  emit('update:modelValue', false)
}
</script>

<style scoped>
.settings-drawer {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.settings-drawer__header {
  flex-shrink: 0;
  border-bottom: 1px solid rgba(var(--v-theme-on-surface), 0.12);
  padding: 16px 20px;
}

.settings-drawer__content {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px;
}

.settings-drawer__footer {
  flex-shrink: 0;
  border-top: 1px solid rgba(var(--v-theme-on-surface), 0.08);
  padding: 16px 20px;
}

/* Custom scrollbar - 8px width per UI-SPEC Section 3.2 */
.settings-drawer__content::-webkit-scrollbar {
  width: 8px;
}

.settings-drawer__content::-webkit-scrollbar-thumb {
  background: rgba(var(--v-theme-on-surface), 0.2);
  border-radius: 4px;
}

.settings-drawer__content::-webkit-scrollbar-thumb:hover {
  background: rgba(var(--v-theme-on-surface), 0.3);
}

.settings-drawer__content::-webkit-scrollbar-track {
  background: transparent;
}
</style>
