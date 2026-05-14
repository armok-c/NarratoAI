<template>
  <SettingSection
    icon="mdi-text-to-speech"
    title="语音合成"
    collapsible
    :default-expanded="false"
    :loading="loading"
    :badge-count="badgeCount"
  >
    <template #header-actions>
      <v-btn variant="text" size="small" @click="handleReset">重置</v-btn>
    </template>
    <template #default>
      <v-select
        v-model="store.engine"
        :items="ENGINE_OPTIONS"
        label="TTS 引擎"
        variant="outlined"
        density="compact"
        class="mb-3"
      />

      <!-- Edge-TTS -->
      <template v-if="store.engine === 'edge_tts'">
        <v-autocomplete
          v-model="edgeCfg.voiceName"
          :items="store.voiceList"
          label="音色"
          variant="outlined"
          density="compact"
          class="mb-3"
          :loading="store.voicesLoading"
          clearable
        />
        <v-text-field
          v-model.number="edgeCfg.volume"
          label="音量"
          type="number"
          min="0"
          max="100"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
        <v-text-field
          v-model.number="edgeCfg.rate"
          label="语速"
          type="number"
          step="0.1"
          min="0.5"
          max="2"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
        <v-text-field
          v-model.number="edgeCfg.pitch"
          label="音调"
          type="number"
          step="1"
          min="-20"
          max="20"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
      </template>

      <!-- Azure -->
      <template v-else-if="store.engine === 'azure_speech'">
        <v-text-field
          v-model="azureCfg.speechKey"
          :type="showAzureKey ? 'text' : 'password'"
          label="订阅密钥"
          variant="outlined"
          density="compact"
          class="mb-3"
          :append-inner-icon="showAzureKey ? 'mdi-eye-off' : 'mdi-eye'"
          @click:append-inner="showAzureKey = !showAzureKey"
        />
        <v-text-field
          v-model="azureCfg.speechRegion"
          label="区域"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
        <v-text-field
          v-model="azureCfg.voiceName"
          label="音色"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
        <v-text-field
          v-model.number="azureCfg.volume"
          label="音量"
          type="number"
          min="0"
          max="100"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
        <v-text-field
          v-model.number="azureCfg.rate"
          label="语速"
          type="number"
          step="0.1"
          min="0.5"
          max="2"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
        <v-text-field
          v-model.number="azureCfg.pitch"
          label="音调"
          type="number"
          step="1"
          min="-20"
          max="20"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
      </template>

      <!-- Tencent -->
      <template v-else-if="store.engine === 'tencent_tts'">
        <v-text-field
          v-model="tencentCfg.secretId"
          :type="showTencentSecretId ? 'text' : 'password'"
          label="SecretId"
          variant="outlined"
          density="compact"
          class="mb-3"
          :append-inner-icon="showTencentSecretId ? 'mdi-eye-off' : 'mdi-eye'"
          @click:append-inner="showTencentSecretId = !showTencentSecretId"
        />
        <v-text-field
          v-model="tencentCfg.secretKey"
          :type="showTencentSecretKey ? 'text' : 'password'"
          label="SecretKey"
          variant="outlined"
          density="compact"
          class="mb-3"
          :append-inner-icon="showTencentSecretKey ? 'mdi-eye-off' : 'mdi-eye'"
          @click:append-inner="showTencentSecretKey = !showTencentSecretKey"
        />
        <v-text-field
          v-model="tencentCfg.region"
          label="区域"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
      </template>

      <!-- SoulVoice -->
      <template v-else-if="store.engine === 'soulvoice'">
        <v-text-field
          v-model="soulCfg.apiKey"
          :type="showSoulKey ? 'text' : 'password'"
          label="API Key"
          variant="outlined"
          density="compact"
          class="mb-3"
          :append-inner-icon="showSoulKey ? 'mdi-eye-off' : 'mdi-eye'"
          @click:append-inner="showSoulKey = !showSoulKey"
        />
        <v-text-field
          v-model="soulCfg.voiceUri"
          label="音色 URI"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
        <v-text-field
          v-model="soulCfg.apiUrl"
          label="接口地址"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
        <v-text-field
          v-model="soulCfg.model"
          label="模型"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
      </template>

      <!-- 阿里通义 Qwen -->
      <template v-else-if="store.engine === 'tts_qwen'">
        <v-text-field
          v-model="qwenCfg.apiKey"
          :type="showQwenKey ? 'text' : 'password'"
          label="API Key"
          variant="outlined"
          density="compact"
          class="mb-3"
          :append-inner-icon="showQwenKey ? 'mdi-eye-off' : 'mdi-eye'"
          @click:append-inner="showQwenKey = !showQwenKey"
        />
        <v-text-field
          v-model="qwenCfg.modelName"
          label="模型名称"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
        <v-text-field
          v-model="qwenCfg.apiUrl"
          label="接口地址"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
      </template>

      <!-- IndexTTS2 -->
      <template v-else-if="store.engine === 'indextts2'">
        <v-text-field
          v-model="indexCfg.apiUrl"
          label="接口地址"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
        <v-text-field
          v-model="indexCfg.referenceAudio"
          label="参考音频"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
        <v-select
          v-model="indexCfg.inferMode"
          :items="INFER_MODES"
          label="推理模式"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
        <v-text-field
          v-model.number="indexCfg.temperature"
          label="温度"
          type="number"
          step="0.1"
          min="0"
          max="2"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
        <v-text-field
          v-model.number="indexCfg.topP"
          label="Top P"
          type="number"
          step="0.05"
          min="0"
          max="1"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
        <v-text-field
          v-model.number="indexCfg.topK"
          label="Top K"
          type="number"
          min="0"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
        <v-switch
          v-model="indexCfg.doSample"
          label="采样"
          color="primary"
          density="compact"
          hide-details
          class="mb-3"
        />
        <v-text-field
          v-model.number="indexCfg.numBeams"
          label="Beam 数量"
          type="number"
          min="1"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
      </template>

      <!-- 豆包 Doubao -->
      <template v-else-if="store.engine === 'doubaotts'">
        <v-text-field
          v-model="doubaoCfg.ak"
          :type="showDoubaoAk ? 'text' : 'password'"
          label="Access Key"
          variant="outlined"
          density="compact"
          class="mb-3"
          :append-inner-icon="showDoubaoAk ? 'mdi-eye-off' : 'mdi-eye'"
          @click:append-inner="showDoubaoAk = !showDoubaoAk"
        />
        <v-text-field
          v-model="doubaoCfg.sk"
          :type="showDoubaoSk ? 'text' : 'password'"
          label="Secret Key"
          variant="outlined"
          density="compact"
          class="mb-3"
          :append-inner-icon="showDoubaoSk ? 'mdi-eye-off' : 'mdi-eye'"
          @click:append-inner="showDoubaoSk = !showDoubaoSk"
        />
        <v-text-field
          v-model="doubaoCfg.appid"
          label="App ID"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
        <v-text-field
          v-model="doubaoCfg.token"
          :type="showDoubaoTokens ? 'text' : 'password'"
          label="Token"
          variant="outlined"
          density="compact"
          class="mb-3"
          :append-inner-icon="showDoubaoTokens ? 'mdi-eye-off' : 'mdi-eye'"
          @click:append-inner="showDoubaoTokens = !showDoubaoTokens"
        />
        <v-text-field
          v-model="doubaoCfg.cluster"
          label="集群"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
        <v-text-field
          v-model="doubaoCfg.apiUrl"
          label="接口地址"
          variant="outlined"
          density="compact"
          class="mb-3"
        />
      </template>
    </template>
  </SettingSection>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useTtsStore } from '@/stores/tts'
import type {
  EdgeEngineConfig,
  AzureEngineConfig,
  TencentEngineConfig,
  SoulvoiceEngineConfig,
  QwenEngineConfig,
  IndexTTS2EngineConfig,
  DoubaoEngineConfig,
} from '@/stores/tts'
import SettingSection from '@/components/SettingSection.vue'

const props = withDefaults(defineProps<{ badgeCount?: number }>(), { badgeCount: 0 })

const ENGINE_OPTIONS = [
  { title: 'Edge-TTS', value: 'edge_tts' },
  { title: 'Azure', value: 'azure_speech' },
  { title: '腾讯云', value: 'tencent_tts' },
  { title: 'SoulVoice', value: 'soulvoice' },
  { title: '阿里通义', value: 'tts_qwen' },
  { title: 'IndexTTS2', value: 'indextts2' },
  { title: '豆包', value: 'doubaotts' },
]

const INFER_MODES = [
  { title: '普通推理', value: 'normal' },
  { title: '快速推理', value: 'fast' },
]

const store = useTtsStore()
const { loading } = store

const edgeCfg = computed(() => store.engineConfigs.edge_tts as unknown as EdgeEngineConfig)
const azureCfg = computed(() => store.engineConfigs.azure_speech as unknown as AzureEngineConfig)
const tencentCfg = computed(() => store.engineConfigs.tencent_tts as unknown as TencentEngineConfig)
const soulCfg = computed(() => store.engineConfigs.soulvoice as unknown as SoulvoiceEngineConfig)
const qwenCfg = computed(() => store.engineConfigs.tts_qwen as unknown as QwenEngineConfig)
const indexCfg = computed(() => store.engineConfigs.indextts2 as unknown as IndexTTS2EngineConfig)
const doubaoCfg = computed(() => store.engineConfigs.doubaotts as unknown as DoubaoEngineConfig)

const showAzureKey = ref(false)
const showTencentSecretId = ref(false)
const showTencentSecretKey = ref(false)
const showSoulKey = ref(false)
const showQwenKey = ref(false)
const showDoubaoAk = ref(false)
const showDoubaoSk = ref(false)
const showDoubaoTokens = ref(false)

watch(() => store.engine, (newEngine) => {
  if (newEngine === 'edge_tts') {
    store.loadVoices()
  }
})

function handleReset() {
  store.resetPanel()
}

function handleEngineChange(_val: string) {
  // Engine switch handled by v-model on store.engine
}
</script>
