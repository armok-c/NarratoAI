<template>
  <v-app>
    <router-view />
  </v-app>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { loadFromBackend, restoreDraft } from '@/composables/useConfig'

const configLoaded = ref(false)

onMounted(async () => {
  try {
    await loadFromBackend()
    const restored = restoreDraft()
    if (restored) {
      console.info('[App] 从草稿恢复配置')
    }
    configLoaded.value = true
  } catch (err) {
    console.warn('[App] 配置预加载失败:', err)
    // 不阻止应用启动
  }
})
</script>
