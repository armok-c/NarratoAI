---
phase: 15-layout-shell-3-mode-ux
plan: 02
subsystem: ui
tags: [vue, vuetify, pinia, tauri, typescript, v-app-bar, window-controls, localStorage]

requires:
  - phase: 14-frontend-foundation
    provides: Pinia stores (app, pipeline), useTheme composable, Vuetify plugin, Vite config
  - phase: 15-01-PLAN.md
    provides: Context, decisions (D-01~D-35), UI spec, research on patterns and pitfalls

provides:
  - AppHeader.vue — 40px custom titlebar with logo, title, gear button, theme toggle, WindowControls slot, window drag with button exclusion, dblclick maximize
  - WindowControls.vue — pure component with min/maximize/restore/close buttons, close red hover
  - logo.svg — app logo (copied from SmartEdit, md5sum verified)
  - ModeStore extension — localStorage persistence (lastMode), setMode writes to localStorage, resetPipeline calls usePipelineStore().reset()

affects:
  - 15-03-PLAN.md (DefaultLayout — consumer of AppHeader/WindowControls; receives emit events for window controls)
  - 15-04-PLAN.md (SettingsDrawer — toggled via AppHeader gear button)
  - 15-05-PLAN.md (ModeSelector — consumes ModeStore)

tech-stack:
  added: []
  patterns:
    - "Window control event chain: WindowControls (pure component, emits events) → AppHeader (forwards events) → DefaultLayout (handles Tauri API calls via dynamic import)"
    - "AppHeader drag with button exclusion: startDrag uses e.target.closest('button') and e.target.closest('.window-controls') to prevent drag on interactive elements"
    - "Dynamic import fallback: @tauri-apps/api/window imported dynamically with try/catch for non-Tauri browser dev mode"

key-files:
  created:
    - src/assets/logo.svg
    - src/components/AppHeader.vue
    - src/components/WindowControls.vue
  modified:
    - src/stores/mode.ts

key-decisions:
  - "D-03: Logo reused from SmartEdit logo.svg"
  - "D-04: Window controls arranged as minimize | maximize | close on far right of AppHeader"
  - "D-05: AppHeader layout (left to right): Logo | NarratoAI title | v-spacer | gear | theme | WindowControls"
  - "D-06: Window drag on mousedown with button/window-controls exclusion via e.target.closest()"
  - "D-07: AppHeader dblclick emits toggleMaximize"
  - "D-08: AppHeader border-bottom: 1px solid rgba(var(--v-theme-on-surface), 0.12)"
  - "D-24: ModeStore setMode updates currentMode + resetPipeline (clears pipeline state, not currentMode)"
  - "D-25: currentMode initialized from localStorage('lastMode') with fallback to 'documentary'; setMode persists"

patterns-established:
  - "Window control pure component pattern: WindowControls is a stateless presentational component with no Tauri imports. It emits minimize/maximize/close events for parent consumption."
  - "AppHeader drag with button exclusion: mousedown handler checks e.target.closest() to avoid initiating window drag on interactive elements."
  - "Dynamic import for Tauri APIs: All @tauri-apps/api/window imports are dynamic (await import(...)) wrapped in try/catch to prevent crashes when running outside Tauri."

requirements-completed:
  - LOUT-01
  - MODE-01

duration: 3min
completed: 2026-05-13
---

# Phase 15 Plan 02: AppHeader + WindowControls + ModeStore Extension Summary

**Custom titlebar AppHeader (40px, window drag, dblclick maximize), WindowControls pure component (minimize/maximize/restore/close with close red hover), and ModeStore extension (localStorage persistence + resetPipeline)**

## Performance

- **Duration:** 3 min
- **Started:** 2026-05-13T08:13:00Z
- **Completed:** 2026-05-13T08:16:46Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments

- Copied logo.svg from SmartEdit (md5sum verified identical, 24 lines meets min_lines:20)
- Created AppHeader.vue with 40px v-app-bar, window drag with button exclusion via e.target.closest(), dblclick toggleMaximize, gear button (mdi-cog), theme toggle (mdi-weather-sunny/mdi-weather-night), and WindowControls integration
- Created WindowControls.vue as pure component with three buttons: minimize (mdi-window-minimize), maximize/restore (mdi-window-maximize/mdi-window-restore), close (mdi-close) with red hover (rgba(244, 67, 54, 0.8))
- Extended ModeStore with localStorage('lastMode') initialization with validation, setMode persistence, and resetPipeline calling usePipelineStore().reset()

## Task Commits

Each task was committed atomically:

1. **Task 1: Copy logo.svg and create AppHeader.vue** - `3a3cd61` (feat)
2. **Task 2: Create WindowControls.vue** - `f7daa84` (feat)
3. **Task 3: Extend ModeStore with localStorage persistence and resetPipeline** - `a81deca` (feat)

## Files Created/Modified

- `src/assets/logo.svg` - App logo (copied from SmartEdit, 24 lines SVG)
- `src/components/AppHeader.vue` - Custom 40px titlebar with window drag, dblclick maximize, gear/theme buttons, WindowControls slot (150 lines)
- `src/components/WindowControls.vue` - Pure component: min/maximize/restore/close buttons with close red hover (66 lines)
- `src/stores/mode.ts` - Extended with localStorage('lastMode') initialization, setMode persistence, and resetPipeline implementation

## Decisions Made

- Followed all locked decisions from CONTEXT.md (D-03 through D-10 for AppHeader/WindowControls, D-24/D-25 for ModeStore)
- Used Vuetify `v-app-bar` with `density="compact"` and explicit `height="40"` for the custom titlebar, matching SmartEdit pattern
- WindowControls close hover uses `rgba(244, 67, 54, 0.8)` with `!important` to override Vuetify default hover styles, per D-04
- Dynamic import with try/catch for all `@tauri-apps/api/window` calls to support browser dev mode outside Tauri

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - all tasks completed without issues.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- AppHeader and WindowControls ready for consumption by DefaultLayout (15-03-PLAN.md)
- ModeStore extended with localStorage persistence, ready for ModeSelector (15-05-PLAN.md)
- Logo asset in place for all layout components
- All components pass TypeScript compilation (npx vue-tsc --noEmit)

---
*Phase: 15-layout-shell-3-mode-ux*
*Completed: 2026-05-13*
