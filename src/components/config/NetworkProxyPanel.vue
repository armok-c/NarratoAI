<template>
  <SettingSection
    icon="mdi-earth"
    title="网络代理"
    collapsible
    :default-expanded="false"
    :badge-count="badgeCount"
  >
    <template #default>
      <v-switch
        v-model="store.enabled"
        label="启用代理"
        color="primary"
        density="compact"
        hide-details
        class="mb-3"
      />

      <v-text-field
        v-model="store.http"
        label="HTTP 代理地址"
        placeholder="留空使用环境变量"
        variant="outlined"
        density="compact"
        class="mb-3"
        :disabled="!store.enabled"
      />
      <v-text-field
        v-model="store.https"
        label="HTTPS 代理地址"
        placeholder="留空使用环境变量"
        variant="outlined"
        density="compact"
        class="mb-3"
        :disabled="!store.enabled"
      />
    </template>
  </SettingSection>
</template>

<script setup lang="ts">
import { useProxyStore } from '@/stores/proxy'
import SettingSection from '@/components/SettingSection.vue'

const props = withDefaults(defineProps<{ badgeCount?: number }>(), { badgeCount: 0 })

const store = useProxyStore()
</script>
