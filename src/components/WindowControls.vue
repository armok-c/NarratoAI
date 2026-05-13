<script setup lang="ts">
withDefaults(defineProps<{
  isMaximized?: boolean
}>(), {
  isMaximized: false,
})

defineEmits<{
  minimize: []
  maximize: []
  close: []
}>()
</script>

<template>
  <div class="window-controls d-flex align-center">
    <v-btn
      icon
      variant="text"
      size="small"
      class="win-btn"
      @click="$emit('minimize')"
      aria-label="最小化"
    >
      <v-icon size="16">mdi-window-minimize</v-icon>
    </v-btn>
    <v-btn
      icon
      variant="text"
      size="small"
      class="win-btn"
      @click="$emit('maximize')"
      aria-label="最大化/还原"
    >
      <v-icon size="16">{{ isMaximized ? 'mdi-window-restore' : 'mdi-window-maximize' }}</v-icon>
    </v-btn>
    <v-btn
      icon
      variant="text"
      size="small"
      class="win-btn close-btn"
      @click="$emit('close')"
      aria-label="关闭"
    >
      <v-icon size="16">mdi-close</v-icon>
    </v-btn>
  </div>
</template>

<style scoped>
.window-controls {
  display: flex;
  align-items: center;
}

.win-btn {
  width: 36px;
  height: 36px;
  min-width: 36px;
  border-radius: 0;
}

.close-btn:hover {
  background-color: rgba(244, 67, 54, 0.8) !important;
}
</style>
