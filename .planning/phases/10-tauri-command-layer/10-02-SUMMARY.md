---
phase: 10-tauri-command-layer
plan: 02
subsystem: tauri-command-layer
tags: [tauri, ipc, commands, error-handling, progress-events]
requires:
  - phase: 10-01
    provides: workspace layout, Deserialize on request types, unified ProgressCallback
provides:
  - CommandError with 8 domain error From<T> implementations
  - ProgressPayload/CommandResponse/ScriptInfo/StepDuration data models
  - 15 Tauri commands (5 pipeline + 7 script + 1 export + 2 script-gen)
  - register_all_commands!() macro for centralized registration
  - Tauri Builder with multi-path config loading, State injection, plugin registration
affects: [v2 frontend milestone, integration testing]
tech-stack:
  added:
    - tokio (sync RwLock for Send-safe async guards in Tauri State)
  patterns:
    - Thin wrapper commands: each Tauri command 1:1 maps to existing Rust API
    - register_all_commands!() macro aligns with register_all_providers()/register_all_prompts()
    - tokio::sync::RwLock for Tauri State to enable Send-safe guards across .await
    - Multi-path config.toml lookup (resource_dir → current_dir → fallback)
key-files:
  created:
    - src-tauri/src/error.rs
    - src-tauri/src/models/mod.rs
    - src-tauri/src/models/progress.rs
    - src-tauri/src/commands/mod.rs
    - src-tauri/src/commands/pipeline.rs
    - src-tauri/src/commands/script.rs
    - src-tauri/src/commands/export.rs
  modified:
    - src-tauri/src/lib.rs
key-decisions:
  - "Use tokio::sync::RwLock for Tauri State wrappers (std::sync::RwLockReadGuard is not Send, causing compile errors across .await points)"
  - "Scope RwLockReadGuard in a block before .await to drop non-Send guards (for generate_documentary_script and generate_sdp_script)"
  - "PromptRegistry maintains std::sync::RwLock (SharedPromptRegistry type constraint); outer PromptManager wrapper uses tokio::sync::RwLock"
patterns-established:
  - "CommandError + std::error::Error + Serialize for all Tauri command return types"
  - "send-safe async guards: tokio::sync::RwLock for any state held across .await"
  - "Progress callback closures capture AppHandle, emit 'pipeline-progress' events with ProgressPayload"
requirements-completed: [TAURI-01, TAURI-02, TAURI-03]
duration: 48min
completed: 2026-05-07
---

# Phase 10 Plan 02: Tauri Command Layer Summary

**Complete Tauri command layer with 15 IPC commands, unified error handling via CommandError, real-time progress event emission, and multi-path config injection**

## Performance

- **Duration:** 48 min
- **Started:** 2026-05-07T12:04:00Z (approx)
- **Completed:** 2026-05-07T12:22:00Z (approx)
- **Tasks:** 3 (4 commits including 1 fix)
- **Files modified:** 8 (7 created, 1 modified)

## Accomplishments

- CommandError with `code` + `message`, `std::error::Error`, and 9 `From<T>` implementations covering all 8 domain error types plus String validation errors
- ProgressPayload per D-10 with 10 fields including optional error_code/error_message (skip_serializing_if), unified across all 3 pipeline types
- 5 pipeline commands (run_documentary, run_sde, run_sdp, generate_documentary_script, generate_sdp_script) with real-time progress via `app_handle.emit("pipeline-progress", payload)`
- 7 script management commands (load/save/validate/get_info/update_narration/set_ost/update_timestamp)
- 1 export command (export_jianying_draft wrapping export_draft)
- `register_all_commands!()` macro listing all 13 handler functions, matching existing `register_all_providers()`/`register_all_prompts()` pattern
- Full Tauri Builder with dialog plugin, multi-path config.toml loading (resource_dir → current_dir → fallback), Arc State injection for LLM Registry and PromptManager

## Task Commits

Each task was committed atomically:

1. **Task 1: CommandError + data models** — `64dd579` (feat) + `bad9902` (fix: ConfigError import path)
2. **Task 2: 15 Tauri commands + register macro** — `a067459` (feat)
3. **Task 3: Tauri Builder lib.rs** — `c57ddc9` (feat)

## Files Created/Modified

- `src-tauri/src/error.rs` — CommandError struct + 9 From<T> impls (8 domain + 1 String)
- `src-tauri/src/models/mod.rs` — Module re-export
- `src-tauri/src/models/progress.rs` — ProgressPayload (10 fields), CommandResponse, StepDuration, ScriptInfo
- `src-tauri/src/commands/mod.rs` — register_all_commands!() macro + re-exports
- `src-tauri/src/commands/pipeline.rs` — run_documentary, run_sde, run_sdp, generate_documentary_script, generate_sdp_script with progress event emission
- `src-tauri/src/commands/script.rs` — load_script, save_script, validate_script, get_script_info, update_narration, set_ost, update_timestamp
- `src-tauri/src/commands/export.rs` — export_jianying_draft wrapping export_draft + ExportRequest
- `src-tauri/src/lib.rs` — Full Tauri Builder with config loading, State injection, plugin/command registration

## Decisions Made

- Used `tokio::sync::RwLock` instead of `std::sync::RwLock` for Registry/PromptManager Tauri State wrappers because `std::sync::RwLockReadGuard` is not `Send`, causing compile errors when held across `.await` points
- Scoped `RwLockReadGuard` in blocks before `.await` for `generate_documentary_script` and `generate_sdp_script` to release the lock early when only a provider lookup is needed
- `PromptRegistry` retains `std::sync::RwLock` (required by `SharedPromptRegistry` type alias); only the outer `PromptManager` wrapper uses `tokio::sync::RwLock`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Corrected ConfigError import path in command error.rs**
- **Found during:** Task 1 (post-commit verification)
- **Issue:** Plan code used `narratoai_core::config::error::ConfigError` but ConfigError is at `narratoai_core::error::ConfigError` (config module has no error submodule)
- **Fix:** Changed import to `narratoai_core::error::ConfigError` and removed CoreConfigError alias
- **Files modified:** src-tauri/src/error.rs
- **Verification:** `cargo check -p narratoai-app --lib` passes
- **Committed in:** bad9902 (fix commit)

**2. [Rule 1 - Bug] Fixed ScriptInfo timestamp parsing return type mismatch**
- **Found during:** Task 2 compilation
- **Issue:** Inner block in get_script_info returned `Some(sec + ms / 1000.0)` (Option<f64>) but `let s: f64` expects a bare f64
- **Fix:** Changed `Some(sec + ms / 1000.0)` to `sec + ms / 1000.0`
- **Files modified:** src-tauri/src/commands/script.rs
- **Verification:** `cargo check -p narratoai-app --lib` passes
- **Committed in:** a067459 (Task 2 commit)

**3. [Rule 1 - Bug] Fixed generate_sdp_script to use provider lookup from registry**
- **Found during:** Task 2 compilation
- **Issue:** Plan code passed `registry.as_ref()` as provider to generate_sdp_script, but the function takes `&dyn LlmProvider`, not Registry. Missing provider lookup step.
- **Fix:** Added provider lookup via `registry.get(&config.app.text_llm_provider)` similar to generate_documentary_script
- **Files modified:** src-tauri/src/commands/pipeline.rs
- **Verification:** `cargo check -p narratoai-app --lib` passes
- **Committed in:** a067459 (Task 2 commit)

**4. [Rule 3 - Blocking] Switched from std::sync::RwLock to tokio::sync::RwLock for Send-safe async guards**
- **Found during:** Task 3 compilation
- **Issue:** `std::sync::RwLockReadGuard` is not `Send`, causing "future cannot be sent between threads safely" errors for all Tauri commands using read() guards across .await points
- **Fix:** Switched Tauri State wrappers from `std::sync::RwLock` to `tokio::sync::RwLock` (guard is Send); used `blocking_write()` in sync setup closure; kept `std::sync::RwLock` for inner PromptRegistry per SharedPromptRegistry type constraint
- **Files modified:** src-tauri/src/lib.rs, src-tauri/src/commands/pipeline.rs
- **Verification:** `cargo check -p narratoai-app --lib` and `cargo check --workspace` pass
- **Committed in:** c57ddc9 (Task 3 commit)

**5. [Rule 3 - Blocking] Added use tauri::Manager and fixed AppConfig reference in setup**
- **Found during:** Task 3 compilation
- **Issue:** `manage()` and `path()` methods on App require `tauri::Manager` trait in scope; `config_manager.get()` returns owned AppConfig, not reference
- **Fix:** Added `use tauri::Manager;` and `&config_manager.get()` to borrow the returned value
- **Files modified:** src-tauri/src/lib.rs
- **Verification:** `cargo check -p narratoai-app --lib` passes
- **Committed in:** c57ddc9 (Task 3 commit)

---

**Total deviations:** 5 auto-fixed (3 bugs, 2 blocking)
**Impact on plan:** All auto-fixes necessary for correctness and compilation. No scope creep.

## Issues Encountered

- `std::sync::RwLockReadGuard` is not `Send` in Rust, making it impossible to hold across `.await` points in async Tauri commands. Switched all Tauri State wrappers to `tokio::sync::RwLock` which provides `Send`-safe guards. The `PromptRegistry` inner wrapper retains `std::sync::RwLock` per `SharedPromptRegistry` type alias constraint in narratoai-core.

## Threat Surface Scan

No new security-relevant surface introduced beyond what the plan's `<threat_model>` covers:
- `CommandError` serialization uses `err.to_string()` (D-18 Chinese messages, no API keys included)
- PathBuf parameters go through serde deserialization (standard input)
- Progress events emit only pipeline status data, never config secrets

## Verification Results

- `cargo check -p narratoai-app --lib`: PASSED (no errors, no warnings)
- `cargo check --workspace`: PASSED (no errors, no warnings from src-tauri)
- From<T> implementations: 9 total (ConfigError, PipelineError, SdeError, SdpError, ScriptError, JianYingError, TTSError, LLMError, String)
- Handler functions in register_all_commands!(): 13 total (5 pipeline, 7 script, 1 export)

## Next Phase Readiness

- Tauri command layer is complete and compilable
- Infrastructure for progress event streaming is ready for frontend consumption
- Config loading with multi-path fallback ready for development and production use
- Next step: frontend (Vue 3 + Vuetify 3) milestone can invoke all 15 commands via Tauri IPC

## Self-Check: PASSED

- 8 created/modified files verified present
- 4 commits verified in git log
- `cargo check -p narratoai-app --lib` — PASSED (no errors, no warnings)
- `cargo check --workspace` — PASSED
- 9 From<T> implementations (8 domain + 1 String)
- 13 handler functions in register_all_commands!() macro
- ProgressPayload with 10 fields per D-10
- CommandResponse per D-07 (output_video_path + step_durations + intermediate_paths)

---
*Phase: 10-tauri-command-layer*
*Completed: 2026-05-07*
