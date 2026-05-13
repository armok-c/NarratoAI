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
      <SettingSection icon="mdi-brightness-6" title="外观" :collapsible="false">
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
    </div>

    <!-- Footer (fixed bottom) -->
    <div class="settings-drawer__footer d-flex align-center justify-center pa-4">
      <span class="text-caption" style="opacity: 0.6">{{ versionText }}</span>
    </div>
  </v-navigation-drawer>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import SettingSection from '@/components/SettingSection.vue'
import { useTheme } from '@/composables/useTheme'

const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const { theme, toggleTheme } = useTheme()
const isDark = computed(() => theme.value === 'dark')
const versionText = ref('v0.1.0')

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
