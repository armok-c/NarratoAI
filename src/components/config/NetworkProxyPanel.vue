<template>
  <SettingSection
    icon="mdi-earth"
    title="网络代理"
    collapsible
    :default-expanded="false"
  >
    <template #default>
      <v-switch
        v-model="proxyEnabled"
        label="启用代理"
        color="primary"
        density="compact"
        hide-details
        class="mb-3"
      />

      <v-text-field
        v-model="proxyHttp"
        label="HTTP 代理地址"
        placeholder="留空使用环境变量"
        variant="outlined"
        density="compact"
        class="mb-3"
        :disabled="!proxyEnabled"
      />
      <v-text-field
        v-model="proxyHttps"
        label="HTTPS 代理地址"
        placeholder="留空使用环境变量"
        variant="outlined"
        density="compact"
        class="mb-3"
        :disabled="!proxyEnabled"
      />
    </template>
  </SettingSection>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import SettingSection from '@/components/SettingSection.vue'

const proxyEnabled = ref(false)
const proxyHttp = ref('')
const proxyHttps = ref('')
</script>
