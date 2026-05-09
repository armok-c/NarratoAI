---
phase: 10-tauri-command-layer
fixed_at: 2026-05-09T12:00:00Z
review_path: .planning/phases/10-tauri-command-layer/10-REVIEW.md
fix_scope: critical_warning
iteration: 2
findings_in_scope: 5
fixed: 4
skipped: 1
status: partial
---

# Phase 10: Code Review Fix Report

**Fixed at:** 2026-05-09T12:00:00Z
**Source review:** .planning/phases/10-tauri-command-layer/10-REVIEW.md
**Iteration:** 2

**Summary:**
- Findings in scope: 5 (1 Critical, 4 Warning)
- Fixed: 4
- Skipped: 1

## Fixed Issues

### CR-01: IPC file path commands lack path traversal protection (BLOCKER)

**Files modified:** `src-tauri/src/error.rs`, `src-tauri/src/commands/script.rs`, `src-tauri/src/commands/export.rs`
**Commit:** 413b04d
**Applied fix:** Added `validate_path()` function in `error.rs` that rejects paths containing `..` traversal sequences. Applied the validation to `load_script`, `save_script`, and `export_jianying_draft` commands at the IPC boundary.

### WR-02: generate_sdp_script doesn't validate subtitle_path parameter

**Files modified:** `src-tauri/src/commands/pipeline.rs`
**Commit:** b7b71ad
**Applied fix:** Added existence check and extension validation for `subtitle_path` at the start of `generate_sdp_script`. Rejects missing files with "FILE_NOT_FOUND" and unsupported formats with "INVALID_PATH".

### WR-03: run_sdp comment says "4 steps" but total_steps = 3

**Files modified:** `src-tauri/src/commands/pipeline.rs`
**Commit:** ab11171
**Applied fix:** Corrected the comment from "执行完整 4 步 SDP 流水线" to "执行完整 3 步 SDP 流水线" to match `total_steps = 3` and the 3 step_index mappings.

### WR-04: get_script_info uses fold(max) for total duration

**Files modified:** `src-tauri/src/commands/script.rs`
**Commit:** 720421c
**Applied fix:** Replaced `.fold(0.0, f64::max)` (which returned the maximum end timestamp) with actual clip duration computation: parse both start and end timestamps, compute `end - start` for each clip, then sum all durations. This correctly handles non-contiguous clip segments.

## Skipped Issues

### WR-01: run_sde and generate_sdp_script hold locks across long-running async pipelines

**File:** `src-tauri/src/commands/pipeline.rs:117-129,258-265`
**Reason:** Requires core API refactoring in `narratoai-core`. The core `run_sde` and `generate_sdp_script` functions accept `&Registry` and `&PromptManager` references, making it impossible to release locks before the pipeline call without changing those public API signatures. This cross-module refactoring affects all internal step functions and risks breaking other consumers. Should be addressed in a dedicated refactoring pass.
**Original issue:** `run_sde` holds `registry.read().await` and `prompt_manager.read().await` across the entire pipeline call (potentially minutes). Same for `generate_sdp_script` holding `prompt_manager.read().await`. Write locks (e.g., for runtime config reload) would be blocked.

---

_Fixed: 2026-05-09T12:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 2_
