<template>
  <SettingSection
    icon="mdi-robot"
    title="视觉模型"
    collapsible
    :default-expanded="true"
    :loading="loading"
  >
    <template #header-actions>
      <v-btn variant="text" size="small" @click="handleReset">重置</v-btn>
    </template>
    <template #default>
      <v-select
        v-model="visionConfig.provider"
        :items="PROVIDER_OPTIONS"
        label="提供商"
        variant="outlined"
        density="compact"
        class="mb-3"
      />
      <div v-if="modelHint" class="text-caption text-medium-emphasis mb-3">
        推荐: {{ modelHint }}
      </div>
      <v-text-field
        v-model="visionConfig.model"
        label="模型"
        variant="outlined"
        density="compact"
        class="mb-3"
      />
      <v-text-field
        v-model="visionConfig.apiKey"
        :type="showKey ? 'text' : 'password'"
        label="API 密钥"
        variant="outlined"
        density="compact"
        class="mb-3"
        :append-inner-icon="showKey ? 'mdi-eye-off' : 'mdi-eye'"
        @click:append-inner="showKey = !showKey"
      />
      <v-text-field
        v-model="visionConfig.baseUrl"
        label="接口地址"
        placeholder="留空使用默认"
        variant="outlined"
        density="compact"
        class="mb-3"
      />
      <v-btn
        variant="outlined"
        size="small"
        :loading="testing"
        @click="handleTestConnection"
      >
        测试连接
      </v-btn>
      <v-snackbar v-model="snackbar.show" :color="snackbar.color" :timeout="snackbar.timeout" location="top">
        {{ snackbar.text }}
      </v-snackbar>
    </template>
  </SettingSection>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useLlmStore } from '@/stores/llm'
import { PROVIDER_OPTIONS, getProviderPreset } from '@/components/config/ProviderPresets'
import SettingSection from '@/components/SettingSection.vue'

const store = useLlmStore()
const { visionConfig, loading, testing } = store
const showKey = ref(false)

const snackbar = ref({
  show: false,
  text: '',
  color: 'success' as 'success' | 'error',
  timeout: 3000,
})

const modelHint = computed(() => {
  const preset = getProviderPreset(visionConfig.provider)
  return preset?.recommendedModel ?? null
})

async function handleTestConnection() {
  try {
    await store.testConnection('vision')
    snackbar.value = { show: true, text: '连接成功', color: 'success', timeout: 3000 }
  } catch (e: any) {
    snackbar.value = { show: true, text: e?.message || '连接失败', color: 'error', timeout: 5000 }
  }
}

function handleReset() {
  store.resetPanel('vision')
}
</script>
