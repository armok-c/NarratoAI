---
phase: 15-layout-shell-3-mode-ux
plan: 03
type: execute
wave: 2
subsystem: settings-drawer
tags: [vue, vuetify, components, settings, theme-toggle]
requires: [15-01-Backend Infra]
provides: [SettingSection reusable card, SettingsDrawer drawer shell]
affects: [src/components]
tech-stack:
  added: []
  patterns: [v-expand-transition, v-navigation-drawer temporary, dynamic import for Tauri API]
key-files:
  created:
    - src/components/SettingSection.vue
    - src/components/SettingsDrawer.vue
  modified: []
metrics:
  duration: ~10 min
  completed: 2026-05-13
  tasks: 2
  files: 2 created
  commits: 2
---

# Phase 15 Plan 03: SettingsDrawer and SettingSection Components Summary

**One-liner:** Created SettingSection reusable collapsible settings card (3 variants, collapsible with v-expand-transition) and SettingsDrawer 420px right temporary drawer (3-section layout with theme toggle and version display).

## Tasks Executed

### Task 1: Create SettingSection.vue (reusable collapsible settings card)

- Created `src/components/SettingSection.vue` with `<script setup lang="ts">` and scoped styles
- **Props:** `title` (string, required), `icon` (optional), `collapsible` (default false), `defaultExpanded` (default true), `variant` ('outlined' | 'flat' | 'elevated', default 'outlined'), `loading` (default false)
- **Template structure:**
  - Outer div with variant-driven CSS class (`.setting-section--outlined` / `--flat` / `--elevated`)
  - Header row: icon + title on left, `header-actions` slot + chevron icon on right
  - Collapsible mode: header click toggles `isExpanded`, chevron rotates between `mdi-chevron-up`/`mdi-chevron-down`
  - Content wrapped in `<v-expand-transition>` with `v-show="isExpanded"` for smooth expand/collapse
  - Loading overlay: `<v-progress-linear indeterminate>` shown when `loading=true`
- **Scoped styles:** 48px min-height header, 0.87 title opacity, 0.6 chevron opacity with 0.2s transform transition, hover highlight on clickable header (4% surface overlay)
- **Slots:** default slot for configuration content (used by Phase 16 panels), `header-actions` slot for additional controls

### Task 2: Create SettingsDrawer.vue (420px right temporary drawer)

- Created `src/components/SettingsDrawer.vue` with flex column layout (height: 100%)
- **v-navigation-drawer:** `location="right" temporary width="420"`, v-model via `modelValue`/`update:modelValue` for DefaultLayout control
- **Three-section structure:**
  - **Header** (flex-shrink: 0): `mdi-cog` icon + "设置" title + `mdi-close` button with border-bottom separator
  - **Content** (flex: 1, overflow-y: auto): `SettingSection` with `mdi-brightness-6` icon and "外观" title containing theme toggle (`v-switch` bound to `isDark` via `useTheme().toggleTheme`)
  - **Footer** (flex-shrink: 0): Version text (`getVersion()` via dynamic import from `@tauri-apps/api/app`, fallback `v0.1.0`) with border-top separator
- **Custom scrollbar:** 8px width, `rgba(var(--v-theme-on-surface), 0.2)` thumb with 4px radius, 0.3 on hover, transparent track
- **Close methods:** Header close button (explicit emit), scrim click (Vuetify temporary default), gear toggle from parent (v-model bidirectional)

### Deviations from Plan

No deviations. Plan executed exactly as written.

### Key Decisions

| Decision | Rationale |
|----------|-----------|
| SettingSection `defaultExpanded` defaults to `true` | Most settings sections should be visible by default; collapsible is opt-in |
| `withDefaults` with `undefined` for optional icon | Allows conditional rendering via `v-if="icon"` — no icon shown when prop not provided |
| Dynamic import for `getVersion` | Consistent with SmartEdit pattern for Tauri API calls; avoids crash in browser dev mode; try/catch provides `v0.1.0` fallback |
| 8px scrollbar width (UI-SPEC override of D-30's 6px) | Aligns with 4px grid system as specified in UI-SPEC Section 3.2, which takes precedence over initial decision |

### Verification Results

| Check | Result |
|-------|--------|
| `npx vue-tsc --noEmit` | Passed (exit code 0) |
| SettingSection.vue props contract | 6 props defined with correct types and defaults |
| SettingsDrawer.vue v-model contract | modelValue prop + update:modelValue emit properly configured |
| SettingSection template structure | Header + v-expand-transition + loading overlay present |
| SettingsDrawer 3-section layout | Header (cog/title/close) + Content (scrollable theme switch) + Footer (version) |

### Known Stubs

None. Both components are complete with real implementations — no placeholder text, no empty data sources, no hardcoded empty values.

## Self-Check: PASSED

**Created files verified:**
- `src/components/SettingSection.vue` — FOUND (2462 bytes, 112 lines)
- `src/components/SettingsDrawer.vue` — FOUND (3451 bytes, 128 lines)

**Commits verified:**
- `47ca1a7` feat(15-layout-shell-3-mode-ux): create SettingSection reusable collapsible settings card component — FOUND
- `ab7fb0e` feat(15-layout-shell-3-mode-ux): create SettingsDrawer 420px right temporary drawer — FOUND

**TypeScript compilation:** Passed (exit code 0)
