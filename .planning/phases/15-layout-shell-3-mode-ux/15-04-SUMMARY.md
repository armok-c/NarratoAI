---
phase: 15-layout-shell-3-mode-ux
plan: 04
name: SystemMonitorBar + DefaultLayout
subsystem: Layout Shell
tags:
  - layout
  - system-monitor
  - window-controls
  - theme-persistence
requires:
  - 15-01-PLAN.md (AppHeader + WindowControls)
  - 15-02-PLAN.md (SettingsDrawer + SettingSection)
affects:
  - src/router/index.ts (consumes DefaultLayout as layout wrapper)
provides:
  - SystemMonitorBar component
  - DefaultLayout rewrite
tech-stack:
  added: []
  patterns:
    - Dynamic import of @tauri-apps/api/window for browser dev fallback
    - setInterval polling with onUnmounted cleanup
    - Vuetify v-layout three-zone component ordering
commit_docs: true
key-files:
  created:
    - src/components/SystemMonitorBar.vue
  modified:
    - src/layouts/DefaultLayout.vue
decisions:
  - D-11: DefaultLayout three-zone structure (AppHeader -> v-main -> SystemMonitorBar v-footer)
  - D-12: DefaultLayout manages showSettings ref for SettingsDrawer toggle
  - D-13: DefaultLayout manages isMaximized ref, initialized from getCurrentWindow().isMaximized()
  - D-14: DefaultLayout handles window control events via dynamic import of @tauri-apps/api/window
  - D-32: SystemMonitorBar positioned in v-footer, shared across all pages
  - D-33: Data sourced from Rust backend get_system_stats Tauri command
  - D-34: Display format CPU XX% / RAM XX% with mdi-chip/mdi-memory icons
  - D-35: 2-second setInterval polling, cleanup on onUnmounted
metrics:
  duration: ~5 min
  completed_date: 2026-05-13
  tasks: 2/2
  files_created: 1
  files_modified: 1
  commits: 2
---

# Phase 15 Plan 04: SystemMonitorBar + DefaultLayout Summary

SystemMonitorBar (CPU/RAM real-time monitoring with 2-second polling) and DefaultLayout (three-zone layout shell: AppHeader + v-main + SystemMonitorBar + SettingsDrawer drawer).

## One-liner

SystemMonitorBar polls `get_system_stats` every 2 seconds via `tauriInvoke`, displays `CPU XX%`/`RAM XX%` with icons, falls back to `--%` on error; DefaultLayout rewrites from trivial placeholder to full three-zone layout with window control event handlers (minimize/maximize/close/toggleMaximize) via dynamic import of `@tauri-apps/api/window`, manages `showSettings` and `isMaximized` refs, and syncs Vuetify theme from AppStore on mount.

## Tasks

| # | Type | Name | Status | Commit |
|---|------|------|--------|--------|
| 1 | auto | Create SystemMonitorBar.vue (CPU/RAM monitoring footer bar) | Done | `a2a2e1f` |
| 2 | auto | Rewrite DefaultLayout.vue (three-zone layout + SettingsDrawer) | Done | `f3a6a14` |

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None — both components have full implementations with proper error handling and lifecycle management.

## Threat Flags

None — all threat model mitigations are implemented:
- T-15-06 (Spoofing): `--%` fallback on error, no auth needed for local system data
- T-15-07 (DoS): `onUnmounted` cleanup of interval, `try/catch` preventing interval breakage
- T-15-08 (Tampering): All dynamic imports wrapped in `try/catch`, non-Tauri environments silently degrade

## Verification Results

- `npx vue-tsc --noEmit`: Clean compilation (no errors)

## Self-Check: PASSED

- [x] `src/components/SystemMonitorBar.vue` exists
- [x] `src/layouts/DefaultLayout.vue` rewritten
- [x] Commit `a2a2e1f` exists
- [x] Commit `f3a6a14` exists
