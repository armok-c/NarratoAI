---
phase: 16-configuration-panels-ffmpeg-sidecar-core-tech-debt
plan: 01
subsystem: backend-config
tags: [tauri, ffmpeg, sidecar, config, shell-plugin, git-lfs]
requires:
  - phase: 15-layout-shell-3-mode-ux
    provides: SettingsDrawer shell and Phase 16 entry point
provides:
  - FFmpeg and FFprobe Tauri sidecar wiring through bundle.externalBin
  - Tauri shell plugin dependency, initialization, and capabilities
  - Five configuration IPC commands for frontend settings panels
  - ConfigManager path access for safe config.toml writes
  - Git LFS tracking for large sidecar binaries
affects: [phase-16, configuration-panels, ffmpeg-packaging, frontend-settings]
tech-stack:
  added: [tauri-plugin-shell, "@tauri-apps/plugin-shell", reqwest, toml, git-lfs]
  patterns:
    - Config commands use State<ConfigManager> instead of a detached AppConfig snapshot.
    - get_config returns full JSON with server-side API key masking.
    - save_config merges sparse JSON updates and checks expected_mtime before writing.
key-files:
  created:
    - src-tauri/src/commands/config.rs
    - src-tauri/binaries/.gitkeep
    - src-tauri/binaries/ffmpeg-x86_64-pc-windows-msvc.exe
    - src-tauri/binaries/ffprobe-x86_64-pc-windows-msvc.exe
    - .gitattributes
    - .planning/phases/16-configuration-panels-ffmpeg-sidecar-core-tech-debt/16-USER-SETUP.md
  modified:
    - src-tauri/Cargo.toml
    - package.json
    - package-lock.json
    - Cargo.lock
    - src-tauri/tauri.conf.json
    - src-tauri/capabilities/default.json
    - src-tauri/src/lib.rs
    - src-tauri/src/commands/mod.rs
    - narratoai-core/src/config/mod.rs
    - narratoai-core/src/tts/mod.rs
    - src-tauri/gen/schemas/acl-manifests.json
    - src-tauri/gen/schemas/capabilities.json
    - src-tauri/gen/schemas/desktop-schema.json
    - src-tauri/gen/schemas/windows-schema.json
key-decisions:
  - "ConfigManager is the single Tauri State for config commands; commands do not use a stale AppConfig State snapshot."
  - "save_config ignores masked secret placeholders so a frontend save cannot write sk-...****...xyz back into config.toml."
  - "Windows FFmpeg/FFprobe sidecars are stored through Git LFS because Tauri requires real target-named binaries for cargo check with externalBin."
patterns-established:
  - "Tauri IPC config commands return Result<T, CommandError> and convert external errors into stable code/message pairs."
  - "Sparse JSON config updates are merged into the current AppConfig before TOML serialization."
  - "Sidecar binaries follow binaries/{name}-{target-triple}{.exe} and are tracked by LFS."
requirements-completed: [TDET-07, CONF-01, CONF-02, CONF-03, CONF-04, CONF-05]
duration: 1h
completed: 2026-05-14
---

# Phase 16 Plan 01: FFmpeg Sidecar + Config Commands Summary

**Tauri FFmpeg sidecar packaging and five config IPC commands for Phase 16 settings panels**

## Performance

- **Duration:** ~1h
- **Started:** 2026-05-13T23:xx:xx+08:00
- **Completed:** 2026-05-14T01:05:23+08:00
- **Tasks:** 3/3 complete
- **Files modified:** 19 in production commit, plus this summary and user setup document

## Accomplishments

- Added `tauri-plugin-shell` / `@tauri-apps/plugin-shell`, shell capabilities, and `bundle.externalBin` entries for `ffmpeg` and `ffprobe`.
- Created `src-tauri/src/commands/config.rs` with `get_config`, `save_config`, `test_llm_connection`, `get_edge_tts_voices`, and `get_system_proxy`.
- Registered all five commands through `register_all_commands!()` and injected `ConfigManager` into Tauri State.
- Added ConfigManager `config_path()` so `save_config` can do `expected_mtime` conflict checks against the real `config.toml`.
- Added Windows x64 FFmpeg/FFprobe sidecar binaries as Git LFS tracked files.

## Task Commits

1. **Tasks 1-3: FFmpeg sidecar, config commands, command registration** - `e453902` (`feat(16-01)`)

**Plan metadata:** pending in docs commit.

## Files Created/Modified

- `src-tauri/src/commands/config.rs` - Five Tauri commands for config panel backend access.
- `src-tauri/tauri.conf.json` - Adds `bundle.externalBin` and shell plugin config.
- `src-tauri/capabilities/default.json` - Adds shell plugin permissions.
- `src-tauri/src/lib.rs` - Initializes shell plugin and manages `ConfigManager`.
- `src-tauri/src/commands/mod.rs` - Registers config module and five commands.
- `narratoai-core/src/config/mod.rs` - Exposes `config_path()` getter.
- `narratoai-core/src/tts/mod.rs` - Exposes `tts::common` so Tauri commands can reuse `ProxyConfig`.
- `src-tauri/binaries/*` - Adds Windows target FFmpeg/FFprobe sidecars through Git LFS.
- `.gitattributes` - Tracks `src-tauri/binaries/**` through Git LFS.

## Decisions Made

- Used `State<'_, ConfigManager>` for config IPC instead of a separate `AppConfig` State snapshot. This keeps reads and writes on the same authoritative config object.
- Added `reqwest` and `toml` directly to `src-tauri/Cargo.toml` because Rust crates cannot rely on transitive dependencies from `narratoai-core`.
- Protected `save_config` from writing masked secret placeholders back to disk; masked values are treated as unchanged, while empty strings still clear keys.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Real sidecar binaries are required for `cargo check` once `externalBin` is configured**
- **Found during:** Task 1 (`cargo check`)
- **Issue:** Tauri build script failed because `binaries\ffmpeg-x86_64-pc-windows-msvc.exe` did not exist.
- **Fix:** Copied real FFmpeg/FFprobe binaries from the local Chocolatey install into `src-tauri/binaries/` and added Git LFS tracking.
- **Files modified:** `.gitattributes`, `src-tauri/binaries/*`
- **Verification:** `cargo check` passes after binaries are present.
- **Committed in:** `e453902`

**2. [Rule 2 - Missing Critical] ConfigManager did not expose config file path**
- **Found during:** Task 2 (`save_config expected_mtime`)
- **Issue:** The plan required mtime conflict detection but `ConfigManager` kept `config_path` private with no getter.
- **Fix:** Added `ConfigManager::config_path()` and used it in `save_config`.
- **Files modified:** `narratoai-core/src/config/mod.rs`, `src-tauri/src/commands/config.rs`
- **Verification:** `cargo check` passes.
- **Committed in:** `e453902`

**3. [Rule 2 - Missing Critical] Masked secrets could be persisted by sparse save**
- **Found during:** Task 2 (`get_config` + `save_config` interaction)
- **Issue:** `get_config` returns masked values; saving an unchanged frontend draft could write masked placeholders to `config.toml`.
- **Fix:** `save_config` strips values equal to `****` or containing `...****...` before merge.
- **Files modified:** `src-tauri/src/commands/config.rs`
- **Verification:** `cargo check` passes and static search confirms `strip_masked_secrets`.
- **Committed in:** `e453902`

---

**Total deviations:** 3 auto-fixed (2 missing critical, 1 blocking)
**Impact on plan:** All fixes were needed to make the planned behavior buildable and safe. No scope expansion beyond the config/sidecar backend surface.

## Issues Encountered

- `npm test` still has one pre-existing failure in `tests/useTauri.test.ts` (`filterSensitiveArgs(arr)).toBe(arr)`). This is the same Phase 15 verification note and is unrelated to Plan 16-01 backend changes.
- `cargo check` passes with existing warnings: ts-rs serde attribute parse warning and two pre-existing unnecessary parentheses warnings in `src-tauri/src/commands/pipeline.rs`.

## User Setup Required

External binary setup is partially complete. See `16-USER-SETUP.md` for:
- Non-Windows FFmpeg/FFprobe target binaries
- Git LFS setup verification
- Packaging checks

## Self-Check: PASSED

- `cd src-tauri; cargo check` passed.
- `npm.cmd ls @tauri-apps/plugin-shell` returned `@tauri-apps/plugin-shell@2.3.5`.
- `config.rs` contains 5 `#[tauri::command]` functions.
- `config.rs` has 279+ lines and implements masking, sparse save, conflict return, LLM connection test, Edge-TTS voice list, and system proxy detection.
- Sidecar binaries exist and are staged as Git LFS pointer files in the production commit.

## Next Phase Readiness

Plan 16-02 can start next. Wave 0 is not complete yet because Edge-TTS proxy tunnel and VisualAnalyzer facade review are still pending.

---
*Phase: 16-configuration-panels-ffmpeg-sidecar-core-tech-debt*
*Completed: 2026-05-14*
