---
phase: 16-configuration-panels-ffmpeg-sidecar-core-tech-debt
plan: 05
subsystem: frontend-config-panels
tags:
  - conf-06
  - draft
  - save-flow
  - settings-panels
  - tests
  - vitest
dependency_graph:
  requires:
    - "16-04 (SettingsDrawer + 7 panels)"
  provides:
    - "Config save/validation/snackbar integration"
    - "Local Draft lifecycle (save/restore/clear)"
    - "Mode switch unsaved changes guard"
    - "Vitest tests for stores/composables/component"
  affects:
    - "17-pipeline-controls (save_config integration)"
tech-stack:
  added:
    - "@vue/test-utils (devDependency for vitest component tests)"
  patterns:
    - "Pinia store markClean() pattern for dirty flag management"
    - "provide/inject avoided — badgeCount prop chain from SettingsDrawer→panels→SettingSection"
    - "ERROR_WARNING for external conflict detection (conflict dialog)"
key-files:
  created:
    - tests/stores/llm.test.ts (46 lines)
    - tests/stores/tts.test.ts (37 lines)
    - tests/composables/useConfig.test.ts (89 lines)
    - tests/components/SettingSection.test.ts (70 lines)
  modified:
    - src/composables/useConfig.ts (draft management + collectAllConfig)
    - src/App.vue (onMounted config preload + draft restore)
    - src/components/SettingsDrawer.vue (save flow + dialogs + badge pass)
    - src/components/SettingSection.vue (badgeCount prop)
    - src/components/config/LlmVisionPanel.vue (badgeCount prop)
    - src/components/config/LlmTextPanel.vue (badgeCount prop)
    - src/components/config/TtsPanel.vue (badgeCount prop)
    - src/components/config/BgmPanel.vue (badgeCount prop)
    - src/components/config/ExportPanel.vue (badgeCount prop)
    - src/components/config/NetworkProxyPanel.vue (badgeCount prop)
    - src/components/config/ModeParamsPanel.vue (badgeCount prop)
    - src/stores/tts.ts (markClean added)
    - src/stores/bgm.ts (markClean added)
    - src/stores/export.ts (markClean added)
    - vitest.config.ts (vue plugin + @ alias)
decisions:
  - "Badge indicator implemented via prop chain (SettingSection.badgeCount) rather than provide/inject for explicit data flow"
  - "Stores use direct dirty flag access in clearDraft() + exposed markClean() for component use"
  - "Mode switch guard uses manual revert on cancel with suppressNextWatch flag to prevent infinite watcher loop"
metrics:
  duration: 26m
  completed_date: 2026-05-14
  commits: 3
  files_changed: 22
  tests_added: 21
  test_files: 4
---

# Phase 16 Plan 05: Local Draft + Global Save Flow + Vitest Tests

**One-liner:** SettingsDrawer complete integration with localStorage draft lifecycle, backend save with validation/snackbar feedback, mode-switch protection, 21 new vitest tests across stores/composables/component.

## Objective

Complete CONF-06 (Local Draft), save/validation/snackbar UX, mode switch panel visibility guard, configuration preload in App.vue, and vitest tests for all core config logic.

## Tasks

### Task 1: Local Draft Management + App.vue Configuration Preload

**Commit:** `bd79b6c`

**Changes:**
- `src/composables/useConfig.ts`: Added `saveDraft()`, `restoreDraft()`, `clearDraft()`, `hasDraft()`, `collectAllConfig()`, `draftStatus` reactive ref, and `_lastConfig` caching for proxy section in draft
- `src/App.vue`: Added `onMounted` with `loadFromBackend()` + `restoreDraft()` waterfall, `configLoaded` ref, try/catch for graceful degradation

### Task 2: Global Save Flow + Mode Switch Guard + Panel Badge

**Commit:** `2eab6bc`

**Changes:**
- `src/components/SettingsDrawer.vue`: Complete save flow (`handleSave`, `clearDraft` on success, validation error parsing with `FIELD_TO_PANEL` mapping, conflict detection dialog). Unsaved changes indicator (`hasUnsavedChanges` computed). Draft auto-save on drawer close via watcher. Reset all dialog with `clearDraft` + all store resets. Mode switch watcher with confirm/cancel dialog (suppressNextWatch flag for revert safety). Snackbar for success/error/warning flows.
- `src/components/SettingSection.vue`: Added `badgeCount` prop (0-255), wraps icon in `v-badge dot color="error"` when count > 0
- All 7 config panels: Added `badgeCount` prop propagation
- `src/stores/{tts,bgm,export}.ts`: Added `markClean()` for consistency

### Task 3: Vitest Tests

**Commit:** `93b13ed`

**Test files:**
- `tests/stores/llm.test.ts`: 5 tests — initial state, updateVisionConfig, updateTextConfig, resetPanel, markClean/markDirty
- `tests/stores/tts.test.ts`: 4 tests — default engine, setEngine, updateConfig per-engine, resetPanel
- `tests/composables/useConfig.test.ts`: 6 tests — saveDraft localStorage write, restoreDraft with store population, no-draft returns false, clearDraft removes + resets dirty, hasDraft boolean, corrupt JSON silent skip
- `tests/components/SettingSection.test.ts`: 6 tests — title render, icon render, default expand, collapse toggle, loading progress bar, header-actions slot

**Infrastructure:**
- Added `@vue/test-utils` as devDependency
- Updated `vitest.config.ts` with vue plugin and `@` alias resolution
- All 21 new tests pass; SettingSection uses `shallowMount` with Vuetify stubs

## Verification

- `npx vue-tsc --noEmit` — PASS (exit 0)
- `npx vitest run tests/stores/llm.test.ts tests/stores/tts.test.ts tests/composables/useConfig.test.ts tests/components/SettingSection.test.ts` — PASS (21/21)
- Pre-existing test `tests/useTauri.test.ts:81` uses `toBe(arr)` which fails with array reference comparison — pre-existing, out of scope

## Deviations

### Rule 2 additions

1. **`badgeCount` prop on SettingSection.vue** — The plan requires validation error badges on panel headers but SettingSection lacked badge support. Added `badgeCount` prop with `v-badge dot color="error"` wrapping the icon element. This is a critical missing feature for the validation error UX.

2. **`markClean()` on tts/bgm/export stores** — The plan's `clearDraft()` sets `store.dirty = false` directly, but stores should expose `markClean()` for component use during per-panel operations. Added to all three stores.

3. **`badgeCount` prop on all 7 panel components** — Each panel needs to accept and forward `badgeCount` to its `SettingSection` child. Added props definition + template binding to all 7 config panels.

## Scope Boundary Compliance

- Pre-existing `tests/useTauri.test.ts` failure (array `toBe` vs `toStrictEqual`) — not modified, out of scope
- Phase 15 test stubs in `src/{components,stores,layouts,views}/__tests__/` — not modified, preserved as-is

## Key Metrics

| Metric | Value |
|--------|-------|
| New test cases | 21 |
| New test files | 4 |
| Files modified | 15 |
| Files created | 4 |
| Commits | 3 |
| Duration | ~26 min |

## Self-Check: PASSED

- [x] src/composables/useConfig.ts exists (319 lines, >120 min)
- [x] src/App.vue contains "loadFromBackend" (2 occurrences)
- [x] tests/stores/llm.test.ts exists (46 lines, >30 min)
- [x] tests/composables/useConfig.test.ts exists (89 lines, >30 min)
- [x] All 3 commit hashes exist in git log
- [x] npx vue-tsc --noEmit passes
- [x] 21 test cases across 4 test files pass
