<template>
  <v-navigation-drawer
    :model-value="modelValue"
    location="right"
    temporary
    width="420"
    @update:model-value="onDrawerClose"
    class="settings-drawer"
    aria-label="设置面板"
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

      <!-- Panel 1: LLM Vision（仅 Documentary 模式可见） -->
      <LlmVisionPanel
        v-if="modeStore.currentMode === 'documentary'"
        :badge-count="panelErrors.llmVision?.length || 0"
        class="mb-3"
      />

      <!-- Panel 2: LLM Text（始终可见） -->
      <LlmTextPanel
        :badge-count="panelErrors.llmText?.length || 0"
        class="mb-3"
      />

      <!-- Panel 3: TTS（SDP 模式隐藏） -->
      <TtsPanel
        v-if="modeStore.currentMode !== 'sdp'"
        :badge-count="panelErrors.tts?.length || 0"
        class="mb-3"
      />

      <!-- Panel 4: BGM（始终可见） -->
      <BgmPanel
        :badge-count="panelErrors.bgm?.length || 0"
        class="mb-3"
      />

      <!-- Panel 5: Export（始终可见） -->
      <ExportPanel
        :badge-count="panelErrors.export?.length || 0"
        class="mb-3"
      />

      <!-- Panel 6: 网络代理（始终可见） -->
      <NetworkProxyPanel
        :badge-count="panelErrors.networkProxy?.length || 0"
        class="mb-3"
      />

      <!-- Panel 7: 模式专用参数（始终可见） -->
      <ModeParamsPanel
        :badge-count="panelErrors.modeParams?.length || 0"
        class="mb-3"
      />
    </div>

    <!-- Footer (fixed bottom, D-04) -->
    <div class="settings-drawer__footer d-flex flex-column align-center pa-4 ga-2">
      <v-btn
        color="primary"
        variant="elevated"
        block
        class="text-none"
        :disabled="!hasUnsavedChanges"
        :loading="isSaving"
        @click="handleSave"
        aria-label="保存设置"
      >
        保存设置
      </v-btn>
      <div class="d-flex align-center ga-2" style="width: 100%">
        <span
          v-if="hasUnsavedChanges"
          class="text-body-2 text-warning"
        >
          未保存的更改
        </span>
        <v-spacer />
        <span
          class="text-caption text-decoration-underline"
          style="cursor: pointer; opacity: 0.7"
          @click="showResetDialog = true"
        >
          重置全部
        </span>
        <span class="text-caption" style="opacity: 0.3">|</span>
        <span class="text-caption" style="opacity: 0.6">{{ versionText }}</span>
      </div>
    </div>

    <!-- Snackbar -->
    <v-snackbar
      v-model="snackbar.show"
      :color="snackbar.color"
      :timeout="snackbar.timeout"
      location="top"
    >
      {{ snackbar.text }}
    </v-snackbar>

    <!-- 重置确认对话框 -->
    <v-dialog v-model="showResetDialog" max-width="400">
      <v-card>
        <v-card-text class="text-body-1 pa-6">
          确认重置全部设置？这将清除所有未保存的更改。
        </v-card-text>
        <v-card-actions class="pa-4 pt-0">
          <v-spacer />
          <v-btn variant="text" @click="showResetDialog = false">取消</v-btn>
          <v-btn color="error" variant="elevated" @click="handleResetAll">确认重置</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- 模式切换确认对话框 -->
    <v-dialog v-model="showModeSwitchDialog" max-width="400">
      <v-card>
        <v-card-text class="text-body-1 pa-6">
          未保存的更改将丢失，是否继续？
        </v-card-text>
        <v-card-actions class="pa-4 pt-0">
          <v-spacer />
          <v-btn variant="text" @click="cancelModeSwitch">取消</v-btn>
          <v-btn color="primary" variant="elevated" @click="confirmModeSwitch">继续</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- 覆盖冲突确认对话框 -->
    <v-dialog v-model="showConflictDialog" max-width="400">
      <v-card>
        <v-card-text class="text-body-1 pa-6">
          配置文件已被外部修改，是否覆盖？
        </v-card-text>
        <v-card-actions class="pa-4 pt-0">
          <v-spacer />
          <v-btn variant="text" @click="showConflictDialog = false">取消</v-btn>
          <v-btn color="warning" variant="elevated" @click="confirmOverwrite">覆盖</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-navigation-drawer>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import SettingSection from '@/components/SettingSection.vue'
import { useTheme } from '@/composables/useTheme'
import { useModeStore } from '@/stores/mode'
import { useLlmStore } from '@/stores/llm'
import { useTtsStore } from '@/stores/tts'
import { useBgmStore } from '@/stores/bgm'
import { useExportStore } from '@/stores/export'
import { saveDraft, clearDraft, saveToBackend, collectAllConfig } from '@/composables/useConfig'
import LlmVisionPanel from '@/components/config/LlmVisionPanel.vue'
import LlmTextPanel from '@/components/config/LlmTextPanel.vue'
import TtsPanel from '@/components/config/TtsPanel.vue'
import BgmPanel from '@/components/config/BgmPanel.vue'
import ExportPanel from '@/components/config/ExportPanel.vue'
import NetworkProxyPanel from '@/components/config/NetworkProxyPanel.vue'
import ModeParamsPanel from '@/components/config/ModeParamsPanel.vue'
import type { WorkMode } from '@/stores/mode'

const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

// ============================================================
// Theme & version
// ============================================================
const { theme, toggleTheme } = useTheme()
const isDark = computed(() => theme.value === 'dark')
const versionText = ref('v0.1.0')

// ============================================================
// Stores
// ============================================================
const modeStore = useModeStore()
const llmStore = useLlmStore()
const ttsStore = useTtsStore()
const bgmStore = useBgmStore()
const exportStore = useExportStore()

// ============================================================
// Save state
// ============================================================
const isSaving = ref(false)
const panelErrors = ref<Record<string, string[]>>({})

const hasUnsavedChanges = computed(() =>
  llmStore.dirty || ttsStore.dirty || bgmStore.dirty || exportStore.dirty
)

// ============================================================
// Snackbar
// ============================================================
const snackbar = ref({
  show: false,
  text: '',
  color: 'success' as 'success' | 'error' | 'warning' | 'info',
  timeout: 3000,
})

function showSnackbar(text: string, color: 'success' | 'error' | 'warning' | 'info', timeout: number) {
  snackbar.value = { show: true, text, color, timeout }
}

// ============================================================
// Dialog visibility
// ============================================================
const showResetDialog = ref(false)
const showModeSwitchDialog = ref(false)
const showConflictDialog = ref(false)

// ============================================================
// Mode switch guard
// ============================================================
let _suppressNextModeWatch = false
let _pendingOldMode: WorkMode = modeStore.currentMode

watch(() => modeStore.currentMode, (newMode, oldMode) => {
  if (_suppressNextModeWatch) {
    _suppressNextModeWatch = false
    return
  }
  if (!hasUnsavedChanges.value) return
  if (!oldMode) return

  _pendingOldMode = oldMode
  showModeSwitchDialog.value = true
})

function confirmModeSwitch() {
  showModeSwitchDialog.value = false
}

function cancelModeSwitch() {
  _suppressNextModeWatch = true
  modeStore.setMode(_pendingOldMode)
  showModeSwitchDialog.value = false
}

// ============================================================
// Draft auto-save on drawer close (D-20)
// ============================================================
function onDrawerClose(val: boolean) {
  if (!val && hasUnsavedChanges.value) {
    saveDraft()
  }
  emit('update:modelValue', val)
}

// ============================================================
// Field-to-panel mapping for validation errors
// ============================================================
const FIELD_TO_PANEL: Record<string, string> = {
  vision_llm_provider: 'llmVision',
  vision_openai_model_name: 'llmVision',
  vision_openai_api_key: 'llmVision',
  vision_openai_base_url: 'llmVision',
  text_llm_provider: 'llmText',
  text_openai_model_name: 'llmText',
  text_openai_api_key: 'llmText',
  text_openai_base_url: 'llmText',
  tts_engine: 'tts',
  edge_voice_name: 'tts',
  edge_volume: 'tts',
  edge_rate: 'tts',
  edge_pitch: 'tts',
  azure_voice_name: 'tts',
  azure_volume: 'tts',
  azure_rate: 'tts',
  azure_pitch: 'tts',
  azure_speech_key: 'tts',
  azure_speech_region: 'tts',
  tencent_secret_id: 'tts',
  tencent_secret_key: 'tts',
  tencent_region: 'tts',
  soulvoice_api_key: 'tts',
  soulvoice_voice_uri: 'tts',
  soulvoice_api_url: 'tts',
  soulvoice_model: 'tts',
  tts_qwen_api_key: 'tts',
  tts_qwen_api_url: 'tts',
  tts_qwen_model_name: 'tts',
  indextts2_api_url: 'tts',
  indextts2_reference_audio: 'tts',
  indextts2_infer_mode: 'tts',
  indextts2_temperature: 'tts',
  indextts2_top_p: 'tts',
  indextts2_top_k: 'tts',
  indextts2_do_sample: 'tts',
  indextts2_num_beams: 'tts',
  doubaotts_ak: 'tts',
  doubaotts_sk: 'tts',
  doubaotts_appid: 'tts',
  doubaotts_token: 'tts',
  doubaotts_cluster: 'tts',
  doubaotts_api_url: 'tts',
  doubaotts_volume: 'tts',
  doubaotts_pitch: 'tts',
  bgm_folder: 'bgm',
  bgm_mode: 'bgm',
  output_dir: 'export',
  output_format: 'export',
  proxy_http: 'networkProxy',
  proxy_https: 'networkProxy',
  proxy_enabled: 'networkProxy',
  mode_frame_interval: 'modeParams',
  mode_vision_batch_size: 'modeParams',
  mode_drama_name: 'modeParams',
  mode_temperature: 'modeParams',
  mode_clip_count: 'modeParams',
  mode_min_duration: 'modeParams',
  mode_max_duration: 'modeParams',
}

function parseValidationErrors(errorMessage: string): Record<string, string[]> {
  const groups: Record<string, string[]> = {}
  // Try to parse field names from the error message
  const fieldPattern = /(\w+)(?: is required| cannot be empty| 为空| 不完整)/g
  let match
  while ((match = fieldPattern.exec(errorMessage)) !== null) {
    const field = match[1]
    const panel = FIELD_TO_PANEL[field] || 'general'
    if (!groups[panel]) groups[panel] = []
    groups[panel].push(field)
  }
  // Also handle structured JSON in error message
  try {
    const jsonStart = errorMessage.indexOf('{')
    if (jsonStart >= 0) {
      const jsonStr = errorMessage.slice(jsonStart)
      const parsed = JSON.parse(jsonStr)
      if (typeof parsed === 'object') {
        for (const [key, msg] of Object.entries(parsed)) {
          const panel = FIELD_TO_PANEL[key] || 'general'
          if (!groups[panel]) groups[panel] = []
          groups[panel].push(`${key}: ${msg}`)
        }
      }
    }
  } catch {
    // Not JSON, ignore
  }
  return groups
}

// ============================================================
// Save handler (D-04, D-05, D-18)
// ============================================================
async function handleSave() {
  isSaving.value = true
  panelErrors.value = {}

  try {
    const changes = collectAllConfig()
    await saveToBackend(changes)

    // 保存成功
    clearDraft()
    llmStore.markClean()
    ttsStore.markClean()
    bgmStore.markClean()
    exportStore.markClean()

    showSnackbar('设置已保存', 'success', 3000)
  } catch (err: any) {
    const msg = err?.message || err?.toString() || '未知错误'

    if (msg.toLowerCase().includes('conflict') || msg.toLowerCase().includes('已修改')) {
      // 外部修改冲突 (D-22)
      showConflictDialog.value = true
    } else if (
      msg.toLowerCase().includes('validation') ||
      msg.toLowerCase().includes('不完整') ||
      msg.toLowerCase().includes('required')
    ) {
      // 验证错误 (D-18)
      panelErrors.value = parseValidationErrors(msg)
      const errorCount = Object.values(panelErrors.value).reduce((sum, arr) => sum + arr.length, 0)
      showSnackbar(`${errorCount} 项配置不完整`, 'warning', 4000)
    } else {
      // 网络/服务器错误
      showSnackbar(`配置保存失败: ${msg}`, 'error', 5000)
    }
  } finally {
    isSaving.value = false
  }
}

// ============================================================
// Conflict overwrite
// ============================================================
async function confirmOverwrite() {
  showConflictDialog.value = false
  isSaving.value = true

  try {
    const changes = collectAllConfig()
    await saveToBackend(changes)

    clearDraft()
    llmStore.markClean()
    ttsStore.markClean()
    bgmStore.markClean()
    exportStore.markClean()
    showSnackbar('设置已保存', 'success', 3000)
  } catch (err: any) {
    showSnackbar(`配置保存失败: ${err?.message || err}`, 'error', 5000)
  } finally {
    isSaving.value = false
  }
}

// ============================================================
// Reset all (D-06, D-23)
// ============================================================
function handleResetAll() {
  showResetDialog.value = false
  clearDraft()

  llmStore.resetPanel('vision')
  llmStore.resetPanel('text')
  ttsStore.resetPanel()
  bgmStore.resetPanel()
  exportStore.resetPanel()
  modeStore.resetParams()
}

// ============================================================
// Close
// ============================================================
function handleClose() {
  emit('update:modelValue', false)
}

// ============================================================
// Lifecycle
// ============================================================
onMounted(async () => {
  try {
    const { getVersion } = await import('@tauri-apps/api/app')
    versionText.value = 'v' + (await getVersion())
  } catch {
    versionText.value = 'v0.1.0'
  }
})
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
