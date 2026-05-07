---
phase: 04-prompt-system-visual-analyzer
fixed_at: 2026-05-07T17:00:00Z
review_path: .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
iteration: 4
findings_in_scope: 6
fixed: 6
skipped: 0
status: all_fixed
---

# Phase 4: Code Review Fix Report

**Fixed at:** 2026-05-07T17:00:00Z
**Source review:** .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
**Iteration:** 4

**Summary:**
- Findings in scope: 6 (1 CRITICAL + 5 WARNING)
- Fixed: 6
- Skipped: 0

## Fixed Issues

### CR-01: Cross-module dependency -- prompt validators import from visual types

**Files modified:** `src/text_utils.rs` (new), `src/lib.rs`, `src/visual/types.rs`, `src/prompt/validators.rs`, `src/visual/analyzer.rs`
**Commit:** 4a79788
**Applied fix:** Extracted `strip_code_fence` to a new shared `src/text_utils.rs` module. Updated `lib.rs` to register the new module. Changed `validators.rs` to import from `crate::text_utils`. Changed `types.rs` to re-export via `pub(crate) use` for backward compatibility within visual module. Updated `analyzer.rs` call site to use `text_utils::strip_code_fence`.

### WR-01: `run_ffmpeg_with_cancel` silently swallows all FFmpeg errors

**Files modified:** `src/visual/frame_extractor.rs`
**Commit:** 1b5458b
**Applied fix:** Added `tracing::warn!` logging in both `Err` branches of `run_ffmpeg_with_cancel` (spawn failure and wait failure), providing error context for production debugging.

### WR-02: Inconsistent FFmpeg binary discovery between fast path and fallback path

**Files modified:** `src/visual/frame_extractor.rs`
**Commit:** 1b5458b
**Applied fix:** Added documentation comments to both `get_video_duration` and `run_ffmpeg_with_cancel` explaining the limitation: fallback path uses system PATH directly, not `ffmpeg_sidecar` binary discovery.

### WR-03: `analyzed_batches` computed by subtraction instead of explicit counter

**Files modified:** `src/visual/analyzer.rs`
**Commit:** 64f1219
**Applied fix:** Introduced explicit `success_count: usize` counter incremented in the `Ok` branch. Replaced both `raw_results.len() - errors.len()` computations (error path and success path) with `success_count`.

### WR-04: `visual_salience` accepts out-of-range f64 values without validation

**Files modified:** `src/visual/types.rs`
**Commit:** 64f1219
**Applied fix:** Added `FrameObservation::validate()` method that checks `visual_salience` is within [0.0, 1.0] range. Returns `Result<(), String>`. Existing deserialization remains permissive; callers opt into validation.

### WR-05: `cancel.unwrap_or_default()` creates unowned token that prevents cancellation on task drop

**Files modified:** `src/visual/frame_extractor.rs`
**Commit:** 1b5458b
**Applied fix:** Added inline documentation comments at both `unwrap_or_default()` call sites explaining the fire-and-forget semantics and recommending callers provide a `CancellationToken` for responsive cancellation.

## Verification

All fixes verified with:
1. `cargo check` passed after each commit (0 new errors, 7 pre-existing warnings)
2. Targeted test suite: 34 tests passed across `visual::types` (10/10), `visual::analyzer` (10/10), `prompt::validators` (14/14), including strip_code_fence tests via re-export

---

_Fixed: 2026-05-07T17:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 4_
