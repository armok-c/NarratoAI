---
phase: 14-foundation-frontend-scaffold
plan: 14-02
subsystem: api
tags: [pinia, tauri, typescript, composables, stores, vue]

requires:
  - phase: 14-foundation-frontend-scaffold
    plan: 14-01
    provides: project scaffold (Vite + Vue 3.5 + TypeScript + Vuetify + vue-router + main.ts + App.vue + layouts + views)
provides:
  - tauriInvoke IPC wrapper with BigInt serialization, sensitive field filtering, debug logging (direct port from SmartEdit)
  - 9 Pinia 3 store skeletons with Composition API (app/mode/video/pipeline/script/llm/tts/bgm/export)
  - 4 API command modules using tauriInvoke (pipelineApi/scriptApi/configApi/exportApi)
  - logger.ts with loglevel wrapper, useTheme composable, supplementary type definitions
affects:
  - Phase 15 (layout components consume app/mode stores + useTheme)
  - Phase 16 (settings panels consume llm/tts/bgm stores + API modules)
  - Phase 17 (pipeline UI consumes pipeline/video/script stores + API modules)

tech-stack:
  added: [pinia, loglevel]
  patterns: [Pinia 3 Composition API stores, tauriInvoke as sole IPC gateway, domain-split API command modules]

key-files:
  created:
    - src/composables/useTauri.ts
    - src/composables/useTheme.ts
    - src/utils/logger.ts
    - src/types/index.ts
    - src/stores/app.ts
    - src/stores/mode.ts
    - src/stores/video.ts
    - src/stores/pipeline.ts
    - src/stores/script.ts
    - src/stores/llm.ts
    - src/stores/tts.ts
    - src/stores/bgm.ts
    - src/stores/export.ts
    - src/stores/index.ts
    - src/api/pipelineApi.ts
    - src/api/scriptApi.ts
    - src/api/configApi.ts
    - src/api/exportApi.ts

key-decisions:
  - "D-07: tauriInvoke direct port from SmartEdit (BigInt serialization + sensitive field filtering + debug logging)"
  - "D-05/D-06: 9 Pinia 3 store skeletons using Composition API (type + initial state + basic actions only)"
  - "D-08: Domain-split API command modules using tauriInvoke instead of raw invoke"
  - "app store uses manual localStorage for theme persistence (not pinia-plugin-persistedstate)"

patterns-established:
  - "tauriInvoke<T>(command, args) is the sole IPC gateway — no direct invoke() calls outside composables layer"
  - "defineStore('name', () => { ... }) Composition API pattern for all Pinia stores"
  - "API modules are pure IPC forwarding layers with no business logic"

requirements-completed: [FNDT-04, FNDT-05]

duration: 18min
completed: 2026-05-12
---

# Phase 14 Plan 14-02: tauriInvoke IPC Wrapper, 9 Pinia Store Skeletons, and 4 API Command Modules

**tauriInvoke wrapper (direct SmartEdit port) with BigInt serialization, sensitive field filtering, and debug logging; 9 Pinia 3 store skeletons with Composition API; 4 domain-split API command modules using tauriInvoke as sole IPC gateway.**

## Performance

- **Duration:** 18 min
- **Started:** 2026-05-12T21:05:00Z
- **Completed:** 2026-05-12T21:23:00Z
- **Tasks:** 3
- **Files created:** 18

## Accomplishments
- Direct-ported SmartEdit useTauri.ts with all functions: isDebugMode, filterSensitiveArgs, debugLog, debugError, serializeBigIntArg, normalizeTauriArgs, tauriInvoke
- Created useTheme composable syncing app store theme with Vuetify useTheme API
- Created logger.ts with loglevel wrapper auto-attaching [NarratoAI] prefix
- Defined supplementary TypeScript interfaces (VideoMeta, ProgressPayload, LLMConfig) in types/index.ts
- Created 9 Pinia 3 store skeletons (app/mode/video/pipeline/script/llm/tts/bgm/export) using Composition API pattern
- Created stores/index.ts barrel export for all 9 use*Store hooks
- Created 4 API command modules (pipelineApi, scriptApi, configApi, exportApi) using tauriInvoke
- All command name strings match actual Tauri backend registrations in src-tauri/src/commands/
- Full TypeScript typecheck passes with zero errors

## Task Commits

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create composables (useTauri.ts direct port + useTheme.ts + logger.ts + types/index.ts) | `647dbc4` | src/composables/useTauri.ts, src/composables/useTheme.ts, src/utils/logger.ts, src/types/index.ts, src/stores/app.ts |
| 2 | Create 9 Pinia 3 Store skeletons + index.ts | `ca20fdb` | src/stores/mode.ts, src/stores/video.ts, src/stores/pipeline.ts, src/stores/script.ts, src/stores/llm.ts, src/stores/tts.ts, src/stores/bgm.ts, src/stores/export.ts, src/stores/index.ts |
| 3 | Create 4 API command modules | `7e210cc` | src/api/pipelineApi.ts, src/api/scriptApi.ts, src/api/configApi.ts, src/api/exportApi.ts |

## Files Created

- `src/composables/useTauri.ts` - tauriInvoke wrapper (BigInt serialization, sensitive field filtering, debug logging) — direct SmartEdit port
- `src/composables/useTheme.ts` - Theme composable syncing app store with Vuetify useTheme
- `src/utils/logger.ts` - loglevel wrapper with [NarratoAI] namespace prefix
- `src/types/index.ts` - VideoMeta, ProgressPayload, LLMConfig interfaces
- `src/stores/app.ts` - Theme state (light/dark) with localStorage persistence
- `src/stores/mode.ts` - WorkMode state (documentary/sde/sdp) with setMode/resetPipeline
- `src/stores/video.ts` - VideoMeta list with addVideo/removeVideo/clear
- `src/stores/pipeline.ts` - PipelineStatus + ProgressPayload with start/stop/reset/updateProgress
- `src/stores/script.ts` - Script content with skeleton load/save/validate/update actions
- `src/stores/llm.ts` - Dual LLMConfig (vision + text) with partial update actions
- `src/stores/tts.ts` - TTS engine selector with per-engine config map
- `src/stores/bgm.ts` - BGM folder + random/specified mode toggle
- `src/stores/export.ts` - Export outputDir and format settings
- `src/stores/index.ts` - Barrel re-export of all 9 use*Store hooks
- `src/api/pipelineApi.ts` - runDocumentary, runSde, runSdp, stopPipeline
- `src/api/scriptApi.ts` - loadScript, saveScript, validateScript, updateNarration, setOst
- `src/api/configApi.ts` - getConfig, setConfig
- `src/api/exportApi.ts` - exportJianyingDraft

## Decisions Made

- Used direct port of SmartEdit useTauri.ts with all functions preserved (D-07)
- Created app store in Task 1 to unblock useTheme.ts cross-task dependency (Rule 3 deviation)
- All 9 stores use Composition API (defineStore('name', () => { ... })) per D-05
- API modules use `any` types with TODO annotations for Phase 16 ts-rs replacement per plan spec
- stopPipeline function included despite not yet existing in backend (future command planned)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Created app store in Task 1 to unblock useTheme.ts typecheck**
- **Found during:** Task 1 (useTheme.ts creation)
- **Issue:** useTheme.ts imports `@/stores/app` but stores were scheduled for Task 2, causing typecheck failure
- **Fix:** Created `src/stores/app.ts` with full theme toggle logic in Task 1 alongside composables
- **Files modified:** src/stores/app.ts (new file)
- **Verification:** typecheck passes with zero errors after Task 1
- **Committed in:** 647dbc4 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Minimal. The app store file was needed only 1 task early. Its content matches the Task 2 spec exactly — no behavior change, no scope creep.

## Known Stubs

| Location | Type | Reason | Resolution Phase |
|----------|------|--------|------------------|
| `src/stores/script.ts` — load/save/validate/update | Empty actions with TODO | Script editing logic deferred to Phase 17 | Phase 17 |
| `src/stores/mode.ts` — resetPipeline | Empty action with TODO | Pipeline lifecycle logic deferred to Phase 17 | Phase 17 |
| `src/api/*.ts` — all function signatures | `any` parameter types | ts-rs type generation deferred to Phase 16 | Phase 16 |

All stubs are intentional per plan decisions D-06 (skeleton stores) and Phase 16 scope (ts-rs type replacement).

## Threat Surface Scan

No new threat surface introduced beyond the plan's existing threat model:
- T-14-03 (debugLog information disclosure) — mitigated via filterSensitiveArgs and DEV-only guard
- T-14-04 (BigInt tampering) — mitigated via safe integer range check and throw
- T-14-05 (localStorage theme) — accepted risk, only stores light/dark preference

## Issues Encountered

- **Cross-task dependency:** useTheme.ts imports from @/stores/app which is a Task 2 file. Resolved by creating app store in Task 1 (Rule 3 deviation documented above).
- **`stop_pipeline`** function included in pipelineApi.ts but corresponding Tauri command does not exist in the backend yet (no stop_pipeline registered in src-tauri/src/commands/mod.rs). This is a planned future command — the API function will work once the backend command is added in a later phase.

## Next Phase Readiness

- Full IPC layer ready: any component can call tauriInvoke to communicate with Tauri backend
- All 9 stores are importable and return correct initial states
- API modules mapped to all Tauri commands registered in the backend
- Phase 15 can immediately consume app/mode stores in layout components (AppHeader, SettingsDrawer)
- Phase 16 can consume llm/tts/bgm stores and API modules for settings panels
- Phase 17 can consume pipeline/video/script stores and pipeline/script API modules for pipeline UI

---
*Phase: 14-foundation-frontend-scaffold*
*Completed: 2026-05-12*
