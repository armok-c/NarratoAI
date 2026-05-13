---
phase: 15-layout-shell-3-mode-ux
plan: 01
type: execute
wave: 1
subsystem: system-monitor
tags: [rust, tauri, sysinfo, vitest, test-stubs]
requires: []
provides: [get_system_stats command, Tauri window config, test stubs]
affects: [src-tauri commands, Tauri config, vitest config]
tech-stack:
  added: [sysinfo 0.33]
  patterns: [Tauri State with std::sync::Mutex for sync data]
key-files:
  created:
    - src-tauri/src/commands/system.rs
    - src/components/__tests__/AppHeader.test.ts
    - src/components/__tests__/WindowControls.test.ts
    - src/components/__tests__/SettingSection.test.ts
    - src/components/__tests__/SettingsDrawer.test.ts
    - src/components/__tests__/SystemMonitorBar.test.ts
    - src/views/__tests__/HomeView.test.ts
    - src/layouts/__tests__/DefaultLayout.test.ts
    - src/stores/__tests__/mode.test.ts
  modified:
    - src-tauri/Cargo.toml
    - src-tauri/src/commands/mod.rs
    - src-tauri/src/lib.rs
    - src-tauri/tauri.conf.json
    - src-tauri/capabilities/default.json
    - vitest.config.ts
metrics:
  duration: ~15 min
  completed: 2026-05-13
  tasks: 3
  files: 9 created, 6 modified
  commits: 4
---

# Phase 15 Plan 01: Backend Infrastructure and Test Stubs Summary

**One-liner:** Created Tauri get_system_stats command using sysinfo crate, configured custom window decorations and permissions, and added 8 vitest test stubs for Phase 15 frontend components.

## Tasks Executed

### Task 1: Create get_system_stats Rust backend with sysinfo crate

- Added `sysinfo = "0.33"` dependency to `src-tauri/Cargo.toml`
- Created `src-tauri/src/commands/system.rs` with:
  - `SystemStats` struct (Serialize, cpu_percent: f64, ram_percent: f64)
  - `SystemState` wrapper (`pub struct SystemState(pub Mutex<System>)`)
  - `get_system_stats` Tauri command — refreshes system info via sysinfo, returns CPU and RAM percentages
  - `#[cfg(test)]` module with 3 unit tests (stats construction, state construction, zero-memory edge case)
- Registered `pub mod system` and `pub use system::*` in `commands/mod.rs`
- Added `crate::commands::system::get_system_stats` to `register_all_commands!()` macro
- Injected `SystemState(Mutex::new(System::new_all()))` via `app.manage()` in `lib.rs` setup closure
- Used `std::sync::Mutex` (not tokio Mutex) since sysinfo operations are synchronous
- **Deviation (Rule 1):** The locked sysinfo 0.33.1 uses `global_cpu_usage()` returning `f32` (not `global_cpu_info().cpu_usage()` as originally written). Fixed during compilation.

### Task 2: Configure Tauri window (decorations, minWidth, permissions)

- Set `decorations: false`, `minWidth: 900`, `minHeight: 600`, `center: true` on the window config in `tauri.conf.json`
- Appended 5 `core:window:allow-*` permissions to `capabilities/default.json`: `allow-minimize`, `allow-toggle-maximize`, `allow-close`, `allow-start-dragging`, `allow-is-maximized`
- Auto-generated `src-tauri/gen/schemas/capabilities.json` reflected the new permissions

### Task 3: Create Phase 15 test stub files and update vitest config

- Extended `vitest.config.ts` include pattern to `['tests/**/*.test.ts', 'src/**/__tests__/*.test.ts']`
- Created 8 test stub files covering all planned Phase 15 targets:
  - `src/components/__tests__/AppHeader.test.ts`
  - `src/components/__tests__/WindowControls.test.ts`
  - `src/components/__tests__/SettingSection.test.ts`
  - `src/components/__tests__/SettingsDrawer.test.ts`
  - `src/components/__tests__/SystemMonitorBar.test.ts`
  - `src/views/__tests__/HomeView.test.ts`
  - `src/layouts/__tests__/DefaultLayout.test.ts`
  - `src/stores/__tests__/mode.test.ts`
- All stubs provide a single `renders without crashing` test with `expect(true).toBe(true)`
- `npx vitest run` passes for all 8 new stubs (1 pre-existing failure in `useTauri.test.ts` unrelated to Phase 15)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] sysinfo 0.33 API mismatch**

- **Found during:** Task 1, cargo check
- **Issue:** sysinfo 0.33.1 uses `global_cpu_usage() -> f32` instead of the newer `global_cpu_info().cpu_usage()` API
- **Fix:** Replaced `system.global_cpu_info().cpu_usage()` with `system.global_cpu_usage() as f64`
- **Files modified:** `src-tauri/src/commands/system.rs`
- **Commit:** `5b87df7`

## Key Decisions

| Decision | Rationale |
|----------|-----------|
| Used std::sync::Mutex for SystemState | sysinfo operations are synchronous (no .await points), lock held for milliseconds |
| Chose sysinfo 0.33 (latest compatible) | Needed stable Rust edition 2021 compatibility with existing dependency tree |
| Added `#[cfg(test)]` unit tests inline in system.rs | Keeps test and implementation colocated, no test file overhead |

## Verification Results

| Check | Result |
|-------|--------|
| `cargo check` | Passed (only pre-existing pipeline.rs warnings) |
| `cargo test system` | 3/3 passed |
| `npx vitest run` | 8 new stubs pass (1 pre-existing failure in useTauri.test.ts) |
| JSON syntax (tauri.conf.json) | Valid (cargo check confirms) |
| JSON syntax (capabilities/default.json) | Valid (cargo check confirms) |

## Self-Check: PASSED

**Created files verified:**
- `src-tauri/src/commands/system.rs` — FOUND
- `src/components/__tests__/AppHeader.test.ts` — FOUND
- `src/components/__tests__/WindowControls.test.ts` — FOUND
- `src/components/__tests__/SettingSection.test.ts` — FOUND
- `src/components/__tests__/SettingsDrawer.test.ts` — FOUND
- `src/components/__tests__/SystemMonitorBar.test.ts` — FOUND
- `src/views/__tests__/HomeView.test.ts` — FOUND
- `src/layouts/__tests__/DefaultLayout.test.ts` — FOUND
- `src/stores/__tests__/mode.test.ts` — FOUND

**Commits verified:**
- `5b87df7` — FOUND
- `e8dab94` — FOUND
- `7ce5281` — FOUND
- `6bff75f` — FOUND
