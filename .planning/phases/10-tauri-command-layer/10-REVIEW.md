---
phase: 10-tauri-command-layer
reviewed: 2026-05-09T18:00:00Z
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
  critical: 1
  warning: 4
  info: 2
  total: 7
status: issues_found
---

# Phase 10: Code Review Report (Iteration 7)

**Reviewed:** 2026-05-09
**Depth:** standard
**Files Reviewed:** 22
**Status:** issues_found

## Summary

Iteration 7 re-reviewed all 22 files at standard depth, verifying fixes from iterations 1-6 and searching for new issues. The one BLOCKER from iteration 6 (CR-04: `validate_path` rejects all Windows absolute paths) remains unfixed -- this is a functional showstopper on the primary target platform. Three warnings from iteration 6 (WR-07, WR-08, WR-09) also remain unfixed. One new warning was discovered: `SdeRequest::validate()` lacks temperature range validation (WR-10), and one new info item was found: case-sensitive file extension check (IN-05).

## Verification of Previous Fixes (Iterations 1-5)

All fixes from iterations 1-5 remain intact and correct:

- **CR-01/CR-02 (path traversal):** `validate_path` at `error.rs:24-50` checks for `..`, backslashes, and Windows drive letters. `draft_name` validation at `export.rs:21-28` checks for `/`, `\`, `..`, null bytes.
- **WR-01 (splitn fix):** `get_script_info` at `script.rs:62` uses `splitn(2, '-')`.
- **WR-02 (serde skip):** `ScriptGenRequest.progress` at `script_gen.rs:23-25` has design-intent comment and `#[serde(skip)]`.
- **WR-03 (trailing comma repair):** `remove_trailing_commas` at `script_gen.rs:381-423` correctly handles string context.
- **WR-04 (blocking_write):** `lib.rs:40-47` constructs `Registry` before wrapping in `Arc<RwLock>`.

## Status of Iteration 6 Findings

### UNFIXED -- CR-04: validate_path rejects ALL Windows absolute paths

**File:** `src-tauri/src/error.rs:24-50`
**Status:** Unfixed. Still a BLOCKER.

The `validate_path` function at lines 35-39 rejects any path containing backslash (`\`), and lines 42-48 reject any path with a drive letter at byte index 1. On Windows, the native file picker dialog always returns absolute paths like `C:\Users\video.mp4` -- these contain both `\` and a drive letter and are unconditionally rejected.

Every Tauri command that accepts a file path (`load_script`, `save_script`, all pipeline commands, `export_jianying_draft`, `generate_sdp_script`) calls `validate_path` first, meaning **the application cannot open or save any file on Windows**.

On Linux/macOS, absolute paths like `/home/user/video.mp4` pass all three checks (no `..`, no `\`, no drive letter) and are accepted. The bug is Windows-specific but affects the primary target platform.

**Fix:** Replace the current approach with a proper path component check:
```rust
pub fn validate_path(path: &std::path::PathBuf) -> Result<(), CommandError> {
    let path_str = path.to_string_lossy();
    if path_str.contains('\0') {
        return Err(CommandError {
            code: "INVALID_PATH".into(),
            message: "path contains null bytes".into(),
        });
    }
    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return Err(CommandError {
                code: "INVALID_PATH".into(),
                message: "path traversal sequences not allowed".into(),
            });
        }
    }
    Ok(())
}
```

### UNFIXED -- WR-07: Blocking filesystem I/O in async Tauri commands

**File:** `src-tauri/src/commands/script.rs:21,38` and `src-tauri/src/commands/export.rs:37`
**Status:** Unfixed.

`load_script` calls `narratoai_core::script::load_script(&path)` (blocking `std::fs` I/O) directly in an `async fn`. Same for `save_script` and `export_jianying_draft`. These block tokio runtime worker threads.

**Fix:** Wrap in `tokio::task::spawn_blocking` (see iteration 6 for code example).

### UNFIXED -- WR-08: Unused task_id variable

**File:** `src-tauri/src/commands/pipeline.rs:57-58,117-118,183-184`
**Status:** Unfixed.

In `run_documentary`, `run_sde`, and `run_sdp`, a `task_id` is created, cloned to `progress_task_id`, then the original is never used again. The closure only captures `progress_task_id`.

**Fix:** Use `task_id` directly in the closure (as done in `generate_documentary_script` at line 252).

### UNFIXED -- WR-09: generate_sdp_script missing parameter validation

**File:** `src-tauri/src/commands/pipeline.rs:289-290`
**Status:** Unfixed.

`generate_sdp_script` accepts `temperature: f64` and `custom_clips: u32` but does not validate ranges before passing to core library. `custom_clips == 0` is never validated anywhere.

**Fix:** Add validation:
```rust
if temperature < 0.0 || temperature > 2.0 {
    return Err(CommandError { code: "INVALID_PARAM".into(), message: "temperature out of range [0, 2]".into() });
}
if custom_clips == 0 {
    return Err(CommandError { code: "INVALID_PARAM".into(), message: "custom_clips must be > 0".into() });
}
```

### UNFIXED -- IN-04: ProgressPayload.pipeline_type comment incomplete

**File:** `src-tauri/src/models/progress.rs:10`
**Status:** Unfixed.

Comment says `"documentary" | "sde" | "sdp"` but omits `"script_gen"` which is used at `pipeline.rs:257`.

**Fix:** Update comment to: `// "documentary" | "sde" | "sdp" | "script_gen"`

## Previously Skipped Issues (Unchanged)

These require large-scale refactoring outside the scope of Phase 10:

- **CR-03** (RwLock read guards held across SDE pipeline): Still present at `pipeline.rs:153-154`.
- **WR-05** (Duplicated FFmpeg command execution): Still present across documentary, SDE, SDP pipelines.
- **WR-06** (Inconsistent task_dir cleanup): SDE pipeline does not clean up `task_dir`.

## Critical Issues

### CR-04 (carried from iteration 6): validate_path rejects ALL Windows absolute paths

*(Full description and fix in "Status of Iteration 6 Findings" section above. This is the only critical issue.)*

## Warnings

### WR-07 (carried): Blocking filesystem I/O in async Tauri commands without spawn_blocking

*(See iteration 6 findings section.)*

### WR-08 (carried): Unused task_id variable in run_documentary, run_sde, run_sdp

*(See iteration 6 findings section.)*

### WR-09 (carried): generate_sdp_script does not validate temperature or custom_clips range

*(See iteration 6 findings section.)*

### WR-10: SdeRequest.validate() does not validate temperature range

**File:** `narratoai-core/src/sde/types.rs:58-102`
**Issue:** `SdeRequest::validate()` validates `voice_rate`, `voice_pitch`, `tts_volume`, `original_volume`, `bgm_volume`, `threads`, `subtitle_font_size`, and `subtitle_color` -- but does not validate the `temperature` field. A negative temperature or an extremely large value would be forwarded directly to the LLM API call, causing either a confusing downstream error or unexpected behavior. In contrast, `SdpRequest::validate()` at `sdp/types.rs:90-95` correctly validates temperature range `[0, 2]`.
**Fix:** Add temperature validation to `SdeRequest::validate()`:
```rust
if self.temperature < 0.0 || self.temperature > 2.0 {
    return Err(format!("temperature out of range [0, 2]: {}", self.temperature));
}
```

## Info

### IN-04 (carried): ProgressPayload.pipeline_type comment missing "script_gen"

*(See iteration 6 findings section.)*

### IN-05: Case-sensitive file extension check in load_script and save_script

**File:** `src-tauri/src/commands/script.rs:15,32`
**Issue:** The extension check `ext != "json"` performs a case-sensitive comparison using `OsStr` equality. On case-sensitive filesystems (Linux, macOS with case-sensitive volumes), a file named `script.JSON` would be rejected. While this is unlikely to cause real-world problems (most scripts will have `.json` extension), it is an inconsistency with how operating systems typically handle file extensions.
**Fix:** Use case-insensitive comparison:
```rust
if path.extension().map_or(true, |ext| ext.to_ascii_lowercase() != "json") {
```

---

_Reviewed: 2026-05-09_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 7_
