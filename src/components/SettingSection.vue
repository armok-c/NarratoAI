<template>
  <div class="setting-section" :class="[`setting-section--${variant}`]">
    <!-- Header -->
    <div
      class="setting-section-header d-flex align-center justify-space-between"
      :class="{ 'setting-section-header--clickable': collapsible }"
      @click="toggleExpand"
    >
      <div class="d-flex align-center">
        <v-badge
          dot
          color="error"
          :model-value="badgeCount > 0"
          offset-x="3"
          offset-y="3"
        >
          <v-icon v-if="icon" :icon="icon" size="20" class="mr-2" />
        </v-badge>
        <span class="setting-section-title text-subtitle-2 font-weight-medium">{{ title }}</span>
      </div>
      <div class="d-flex align-center">
        <slot name="header-actions" />
        <v-icon
          v-if="collapsible"
          :icon="isExpanded ? 'mdi-chevron-up' : 'mdi-chevron-down'"
          size="20"
          class="ml-2 setting-section-chevron"
        />
      </div>
    </div>

    <!-- Content with expand transition -->
    <v-expand-transition>
      <div v-show="isExpanded" class="setting-section-content">
        <slot />
        <v-progress-linear
          v-if="loading"
          indeterminate
          color="primary"
          class="mt-2"
        />
      </div>
    </v-expand-transition>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

interface SettingSectionProps {
  title: string
  icon?: string
  collapsible?: boolean
  defaultExpanded?: boolean
  variant?: 'outlined' | 'flat' | 'elevated'
  loading?: boolean
  badgeCount?: number
}

const props = withDefaults(defineProps<SettingSectionProps>(), {
  icon: undefined,
  collapsible: false,
  defaultExpanded: true,
  variant: 'outlined',
  loading: false,
  badgeCount: 0,
})

const isExpanded = ref(props.defaultExpanded)

function toggleExpand() {
  if (props.collapsible) {
    isExpanded.value = !isExpanded.value
  }
}
</script>

<style scoped>
.setting-section {
  width: 100%;
}

.setting-section--outlined {
  border: 1px solid rgba(var(--v-theme-on-surface), 0.12);
  border-radius: 8px;
}

.setting-section--flat {
  border: none;
  background: transparent;
}

.setting-section--elevated {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
}

.setting-section-header {
  min-height: 48px;
  padding: 12px 16px;
}

.setting-section-header--clickable {
  cursor: pointer;
}

.setting-section-header--clickable:hover {
  background: rgba(var(--v-theme-on-surface), 0.04);
}

.setting-section-content {
  padding: 0 16px 16px 16px;
}

.setting-section-title {
  opacity: 0.87;
}

.setting-section-chevron {
  opacity: 0.6;
  transition: transform 0.2s ease;
}
</style>
