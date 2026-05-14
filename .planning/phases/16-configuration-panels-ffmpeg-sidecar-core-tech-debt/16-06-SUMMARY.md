# 16-06 Summary: Pinia Store Dirty Tracking System

**Status:** Complete
**Date:** 2026-05-14
**Type:** Gap closure (Wave 1)
**Requirements:** CONF-06

## Changes

### Task 1: Watch-based dirty tracking (6 stores)

Added `watch` listeners to all 6 Pinia config stores using `flush: 'sync'` for immediate reactivity:

| Store | Watch Targets |
|-------|--------------|
| `src/stores/llm.ts` | `watch(visionConfig)` + `watch(textConfig)` — `{ deep: true }` |
| `src/stores/tts.ts` | `watch(engine)` + `watch(engineConfigs)` — engineConfigs with `{ deep: true }` |
| `src/stores/bgm.ts` | `watch([folder, mode, selectedFile])` — multi-source |
| `src/stores/export.ts` | `watch([outputDir, format])` — multi-source |
| `src/stores/proxy.ts` | `watch([enabled, http, https])` — multi-source |
| `src/stores/mode.ts` | `watch(params)` — `{ deep: true }` + new `dirty` ref + `markClean()` |

### Task 2: SettingsDrawer + useConfig integration + vitest tests

- **SettingsDrawer.vue**: `hasUnsavedChanges` includes `modeStore.dirty`; `handleSave`/`confirmOverwrite` call `modeStore.markClean()`; `handleResetAll` calls `markClean()` on all 6 stores
- **useConfig.ts**: `clearDraft()` resets `mode.dirty = false`
- **tests/stores/dirty.test.ts**: 7 new test cases covering all 6 stores

## Verification

| Check | Result |
|-------|--------|
| `npx vue-tsc --noEmit` | PASS (exit 0) |
| `npx vitest run tests/stores/dirty.test.ts` | 7/7 PASS |
| `npx vitest run` (full suite) | 57/58 PASS (1 pre-existing failure unrelated) |

## Pre-existing Issue

`tests/useTauri.test.ts > filterSensitiveArgs > returns arrays-as-is` fails due to `Object.is` vs deep equality — pre-existing, unrelated to this change.
