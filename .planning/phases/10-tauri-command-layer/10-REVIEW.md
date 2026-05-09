---
phase: 10-tauri-command-layer
reviewed: 2026-05-09T19:30:00Z
depth: standard
files_reviewed: 22
files_reviewed_list:
  - Cargo.toml
  - narratoai-core/Cargo.toml
  - src-tauri/Cargo.toml
  - src-tauri/build.rs
  - src-tauri/tauri.conf.json
  - src-tauri/capabilities/default.json
  - src-tauri/src/main.rs
  - src-tauri/src/lib.rs
  - src-tauri/src/error.rs
  - src-tauri/src/models/mod.rs
  - src-tauri/src/models/progress.rs
  - src-tauri/src/commands/mod.rs
  - src-tauri/src/commands/pipeline.rs
  - src-tauri/src/commands/script.rs
  - src-tauri/src/commands/export.rs
  - narratoai-core/src/documentary/types.rs
  - narratoai-core/src/sde/types.rs
  - narratoai-core/src/sdp/types.rs
  - narratoai-core/src/documentary/script_gen.rs
  - narratoai-core/src/jianying/builder.rs
  - narratoai-core/src/documentary/pipeline.rs
  - narratoai-core/src/sde/pipeline.rs
  - narratoai-core/src/sdp/pipeline.rs
findings:
  critical: 0
  warning: 0
  info: 2
  total: 2
status: issues_found
---

# Phase 10: Code Review Report (Iteration 8)

**Reviewed:** 2026-05-09
**Depth:** standard
**Files Reviewed:** 22
**Status:** issues_found

## Summary

Iteration 8 re-reviewed all 22 files at standard depth, verifying fixes from iteration 7 (CR-04, WR-07, WR-08, WR-09, WR-10) and searching for new issues. All 5 previously identified fixable issues have been verified as correctly implemented:

- **CR-04 (VERIFIED):** `validate_path` at `error.rs:29-46` now uses `std::path::Component` traversal instead of string matching. Correctly rejects `ParentDir` components while accepting Windows absolute paths (`C:\Users\...`) and UNC paths.
- **WR-07 (VERIFIED):** `load_script`, `save_script`, and `export_jianying_draft` all use `tokio::task::spawn_blocking` (`script.rs:22,45`, `export.rs:37`).
- **WR-08 (VERIFIED):** No `progress_task_id` variable remains. All three pipeline commands (`run_documentary:57`, `run_sde:116`, `run_sdp:181`) use `task_id` directly in the closure.
- **WR-09 (VERIFIED):** `generate_sdp_script` at `pipeline.rs:308-318` validates `temperature` in [0,2] and `custom_clips > 0`.
- **WR-10 (VERIFIED):** `SdeRequest::validate()` at `sde/types.rs:91-93` validates `temperature` range [0,2].

Two previously carried info items remain unfixed (IN-04, IN-05). Three previously skipped architectural issues remain unchanged (CR-03, WR-05, WR-06). No new critical or warning issues were discovered in this iteration.

## Verification of Previous Fixes (Iterations 1-7)

All fixes from iterations 1-5 remain intact:

- **CR-01/CR-02 (path traversal):** `validate_path` at `error.rs:29-46` uses `Component` traversal. `draft_name` validation at `export.rs:21-28` checks for `/`, `\`, `..`, null bytes.
- **WR-01 (splitn fix):** `get_script_info` at `script.rs:74` uses `splitn(2, '-')`.
- **WR-02 (serde skip):** `ScriptGenRequest.progress` at `script_gen.rs:24` has `#[serde(skip)]`.
- **WR-03 (trailing comma repair):** `remove_trailing_commas` at `script_gen.rs:381-423` correctly handles string context.
- **WR-04 (blocking_write):** `lib.rs:40-47` constructs `Registry` before wrapping in `Arc<RwLock>`.

Iteration 6 fix (CR-04) verified correct: `validate_path` at `error.rs:29-46` iterates over `path.components()` and rejects only `ParentDir` components. Windows paths (`C:\Users\...`) are accepted because `Prefix` and `Normal` components pass through.

Iteration 7 fixes (WR-07, WR-08, WR-09, WR-10) all verified correct as detailed in the Summary.

## Previously Skipped Issues (Unchanged)

These require large-scale refactoring outside the scope of Phase 10:

- **CR-03** (skipped -- requires refactoring outside phase scope): RwLock read guards held across SDE pipeline. Still present at `pipeline.rs:151-152`.
- **WR-05** (skipped -- requires refactoring outside phase scope): Duplicated FFmpeg command execution across documentary, SDE, SDP pipelines.
- **WR-06** (skipped -- requires refactoring outside phase scope): Inconsistent task_dir cleanup (SDE pipeline does not clean up).

## Info

### IN-04 (carried): ProgressPayload.pipeline_type comment missing "script_gen"

**File:** `src-tauri/src/models/progress.rs:10`
**Issue:** Comment says `"documentary" | "sde" | "sdp"` but omits `"script_gen"` which is used at `pipeline.rs:257`.
**Fix:** Update comment to: `// "documentary" | "sde" | "sdp" | "script_gen"`

### IN-05 (carried): Case-sensitive file extension check in load_script/save_script

**File:** `src-tauri/src/commands/script.rs:15,38`
**Issue:** The extension check `ext != "json"` performs a case-sensitive comparison using `OsStr` equality. On case-sensitive filesystems (Linux, macOS with case-sensitive volumes), a file named `script.JSON` would be rejected.
**Fix:** Use case-insensitive comparison:
```rust
if path.extension().map_or(true, |ext| ext.to_ascii_lowercase() != "json") {
```

---

_Reviewed: 2026-05-09_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 8_
