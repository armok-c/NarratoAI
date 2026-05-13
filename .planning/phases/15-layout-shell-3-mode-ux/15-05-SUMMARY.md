---
phase: 15-layout-shell-3-mode-ux
plan: 05
subsystem: ui
tags: [vue, vuetify, pinia, typescript, HomeView, ModeSelector]

requires:
  - phase: 14-frontend-foundation
    provides: Pinia stores (app, pipeline, mode), vue-router, Vuetify plugin, Vite config
  - phase: 15-01-PLAN.md
    provides: Context, decisions (D-15~D-26), UI spec, copywriting, layout tokens
  - phase: 15-02-PLAN.md
    provides: ModeStore with localStorage persistence, setMode, resetPipeline

provides:
  - HomeView.vue — rewritten with 60/40 upper/lower split layout, three labeled placeholder zones (VideoTable / 操作按钮 / LogPanel), and ModeSelector v-card with v-select mode switching

affects:
  - Phase 16 (replaces placeholders with real panels)
  - Phase 17 (VideoTable real component replaces placeholder)

tech-stack:
  added: []
  patterns:
    - "Placeholder-card pattern: dashed border, icon + title + Phase annotation for forward scheduling"
    - "ModeSelector integration: v-select update:model-value calls ModeStore.setMode() + resetPipeline(), watch sync non-local changes"

key-files:
  modified:
    - src/views/HomeView.vue

key-decisions:
  - "D-15: Upper/lower split 60/40 fixed ratio (non-draggable)"
  - "D-16: Upper row columns: VideoTable 60% / settings card 40%, 16px gap"
  - "D-17: Lower row columns: buttons 60% / LogPanel 40%, 16px gap, aligned with upper columns"
  - "D-18: Full-width layout (no max-width), content fills v-main"
  - "D-19: 16px gaps between sections and columns, no v-divider"
  - "D-20: Placeholders use gray dashed border + title + Phase 17 annotation"
  - "D-21: Upper right column contains ModeSelector only; no LLM/TTS/BGM/Export panels in Phase 15"
  - "D-22: v-select with Chinese labels: documentary/SDE/SDP"
  - "D-23: ModeSelector wrapped in v-card outlined variant with title + mdi-folder-multiple"
  - "D-24: Mode change calls setMode + resetPipeline, no confirmation dialog"
  - "D-25: Default mode 'documentary', persisted via localStorage(lastMode)"
  - "D-26: Panel dynamic show/hide (MODE-03) not active in Phase 15"

requirements-completed:
  - LOUT-03
  - MODE-02
  - MODE-03

duration: 2min
completed: 2026-05-13
---

# Phase 15 Plan 05: HomeView 60/40 Split Layout, Placeholder Zones, and ModeSelector Summary

**HomeView.vue rewritten with 60/40 upper/lower split, three placeholder cards (VideoTable / 操作按钮 / LogPanel) with gray dashed borders and Phase 17 labels, and ModeSelector v-select with documentary/SDE/SDP mode switching via ModeStore**

## Performance

- **Duration:** 2 min
- **Started:** 2026-05-13T08:17:00Z
- **Completed:** 2026-05-13T08:19:00Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments

- Rewrote HomeView.vue from welcome placeholder to full 60/40 upper/lower split layout
- Created three labeled placeholder zones: VideoTable (mdi-video-outline), 操作按钮 (mdi-play-circle-outline), LogPanel (mdi-text-box-outline) with "Phase 17 实现" annotation
- Implemented ModeSelector v-card with "工作模式" title, mdi-folder-multiple icon, outlined variant, and v-select dropdown
- v-select offers three Chinese-labeled options: documentary / SDE / SDP
- Mode change handler calls ModeStore.setMode() + ModeStore.resetPipeline()
- watch syncs selectedMode with external ModeStore.currentMode changes

## Task Commits

Each task was committed atomically:

1. **Task 1: Rewrite HomeView.vue with 60/40 split layout, labeled placeholder zones, and ModeSelector card** - `f0048b3` (feat)

## Files Created/Modified

- `src/views/HomeView.vue` - Rewritten from welcome placeholder to full HomeView with 60/40 split, three placeholder zones, and ModeSelector card (91 lines, +85/-6)

## Decisions Made

- Followed all locked decisions from CONTEXT.md (D-15 through D-26) for HomeView layout ratios, placeholder styling, and ModeSelector behavior
- Used CSS flex: ratios combined with Vuetify d-flex ga-4 classes for layout; avoided v-row/v-col to keep markup minimal
- Three placeholders each have a distinct mdi icon for developer clarity: mdi-video-outline, mdi-play-circle-outline, mdi-text-box-outline
- Placeholder style uses 2px dashed rgba(var(--v-theme-on-surface), 0.2) border with subtle background rgba(var(--v-theme-on-surface), 0.02), following Vuetify theme token pattern

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - single task completed without issues.

## Threat Surface Scan

No new security-relevant surface introduced. v-select uses hardcoded options array (T-15-09 mitigated by static items). Placeholder cards display no sensitive information (T-15-10 accepted).

## Known Stubs

| Stub | File | Line | Reason |
|------|------|------|--------|
| VideoTable placeholder | src/views/HomeView.vue | 14-17 | Phase 17 will replace with real VideoTable component |
| 操作按钮 placeholder | src/views/HomeView.vue | 47-51 | Phase 17 will replace with real action buttons |
| LogPanel placeholder | src/views/HomeView.vue | 56-60 | Phase 17 will replace with real LogPanel component |

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- HomeView layout shell complete and ready for Phase 16 (SettingsDrawer content panels)
- ModeSelector integration with ModeStore ready for mode-driven panel visibility in Phase 16
- Three placeholder zones clearly marked for Phase 17 replacement
- TypeScript compilation passes cleanly (npx vue-tsc --noEmit)

---
*Phase: 15-layout-shell-3-mode-ux*
*Completed: 2026-05-13*
