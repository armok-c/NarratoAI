---
phase: 16-configuration-panels-ffmpeg-sidecar-core-tech-debt
plan: 04
subsystem: ui
tags: vue3, vuetify, pinia, settings, llm, tts, bgm, export, proxy
requires:
  - phase: 16-03
    provides: expanded Pinia stores with loadConfig/testConnection/resetPanel
provides:
  - SettingsDrawer with 7-panel accordion layout (D-03 order)
  - LlmVisionPanel and LlmTextPanel with Provider v-select + password fields + test connection
  - TtsPanel with 7-engine dropdown and dynamic per-engine config fields
  - BgmPanel with Tauri dialog folder picker + random/specified switch
  - ExportPanel with output directory picker + format selector
  - ModeParamsPanel with dynamic fields per currentMode (documentary/sde/sdp)
  - NetworkProxyPanel with enable switch + HTTP/HTTPS address fields
  - Mode-aware panel visibility (MODE-03 deferred from Phase 15)
affects:
  - 16-05: global save/reset integration, snackbar feedback, config persist

tech-stack:
  added: []
  patterns:
    - Each config panel is an independent component file using SettingSection wrapper
    - Store props destructured from Pinia stores for reactive binding
    - Dynamic Tauri plugin imports with try/catch for non-Tauri dev mode

key-files:
  created:
    - src/components/config/LlmVisionPanel.vue
    - src/components/config/LlmTextPanel.vue
    - src/components/config/TtsPanel.vue
    - src/components/config/BgmPanel.vue
    - src/components/config/ExportPanel.vue
    - src/components/config/ModeParamsPanel.vue
    - src/components/config/NetworkProxyPanel.vue
  modified:
    - src/components/SettingsDrawer.vue (rewritten with 7-panel accordion)
    - src/stores/llm.ts (added testing ref for connection test loading state)

key-decisions:
  - "Followed D-01 through D-19 from CONTEXT.md for all panel UX decisions"
  - "Kept appearance theme toggle section above the 7 config panels"
  - "Footer save button rendered disabled (wired in 16-05)"
  - "NetworkProxyPanel uses local refs (not store) — proxy config fetching deferred to 16-05"
  - "BGM/Export panel folder selection uses dynamic import of @tauri-apps/plugin-dialog with non-Tauri fallback"

patterns-established:
  - "Config panel pattern: SettingSection wrapper + store destructuring + per-component snackbar"
  - "Panel visibility via v-if on modeStore.currentMode (MODE-03)"

requirements-completed:
  - CONF-01
  - CONF-02
  - CONF-03
  - CONF-04
  - CONF-05
  - MODE-03

duration: 16min
completed: 2026-05-14
---

# Phase 16 Plan 04: SettingsDrawer Panels Summary

**7 config panel components with accordion layout in SettingsDrawer, mode-aware visibility, and expanded footer with disabled save button**

## Performance

- **Duration:** 16 min
- **Started:** 2026-05-14T06:45:00Z
- **Completed:** 2026-05-14T07:01:38Z
- **Tasks:** 3
- **Files created:** 7
- **Files modified:** 2

## Accomplishments

- Created 7 independent config panel components (LlmVision, LlmText, TTS, BGM, Export, ModeParams, NetworkProxy)
- Rewrote SettingsDrawer.vue with hand accordion layout in D-03 order with mode-aware visibility (MODE-03)
- All 7 panels pass `npx vue-tsc --noEmit` — zero TypeScript errors
- LLM panels include Provider v-select with 11 presets, model hint text, password-toggle API Key field, and test connection button with snackbar feedback
- TTS panel supports all 7 engines with dynamic per-engine field sets and Edge-TTS voice autocomplete
- BGM and Export panels use Tauri native folder dialog with non-Tauri fallback
- Footer contains disabled Save button, Reset All link, and version text

## Task Commits

Each task was committed atomically:

1. **Task 1: Create LLM Vision/Text panels + ProviderPresets** - `e206ee3` (feat)
2. **Task 2: Create TTS/BGM/Export panels** - `a7f1b09` (feat)
3. **Task 3: Create ModeParams/NetworkProxy panels + rewrite SettingsDrawer** - `2618514` (feat)

## Files Created/Modified

### Created
- `src/components/config/LlmVisionPanel.vue` - Vision LLM config with provider/model/apiKey/baseUrl fields + test connection
- `src/components/config/LlmTextPanel.vue` - Text LLM config, same structure as Vision
- `src/components/config/TtsPanel.vue` - 7-engine TTS config with dynamic field rendering (416 lines)
- `src/components/config/BgmPanel.vue` - BGM folder picker + random/specified switch
- `src/components/config/ExportPanel.vue` - Export directory picker + format selector
- `src/components/config/ModeParamsPanel.vue` - Mode-specific parameters (frameInterval/dramaName/clipCount)
- `src/components/config/NetworkProxyPanel.vue` - Proxy enable/disable + HTTP/HTTPS addresses

### Modified
- `src/components/SettingsDrawer.vue` - Rewritten with 7-panel accordion, mode visibility, expanded footer
- `src/stores/llm.ts` - Added `testing` ref for connection test loading state

## Decisions Made

- **Appearance panel preserved:** Kept the existing theme toggle section above the 7 config panels rather than removing it (not explicitly in plan but follows D-02's non-config panel UX)
- **Footer save button disabled:** Rendered with `disabled` attribute — will be wired to actual save logic in Plan 16-05
- **NetworkProxyPanel uses local refs:** Proxy config fetching from backend deferred to Plan 16-05 when the full save/load cycle is implemented
- **Tauri dialog dynamic import:** Used dynamic `import()` with try/catch for `@tauri-apps/plugin-dialog` — gracefully degrades in non-Tauri dev mode

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Missing `testing` ref on LLM store**
- **Found during:** Task 1 (LLM panel creation)
- **Issue:** Components referenced `store.testing` for the test connection button loading state, but the store didn't export a `testing` ref
- **Fix:** Added `testing = ref(false)` to the store, wired it in `testConnection()` with try/finally, and exported it
- **Files modified:** src/stores/llm.ts
- **Verification:** `npx vue-tsc --noEmit` passes (was failing with TS2339 before fix)
- **Committed in:** e206ee3 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Auto-fix essential for TypeScript compilation. No scope creep.

## Issues Encountered

- `@tauri-apps/plugin-dialog` is not installed as an npm dependency. Used dynamic import with `@ts-expect-error` inside try/catch for graceful non-Tauri fallback. This is the correct pattern per project conventions.
- `vue-tsc --noEmit` uncovered that destructured Pinia setup store properties are auto-unwrapped (not Refs) — used without `.value` in script blocks.

## Next Phase Readiness

- All 7 panel UI components complete and ready for Plan 16-05 integration (global save/reset, draft persistence, snackbar feedback)
- Panel visibility rules per MODE-03 are wired via `v-if` on `modeStore.currentMode`
- TypeScript compilation passes with zero errors

---
*Phase: 16-configuration-panels-ffmpeg-sidecar-core-tech-debt*
*Completed: 2026-05-14*
