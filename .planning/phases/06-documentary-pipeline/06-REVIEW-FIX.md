---
phase: 06-documentary-pipeline
fixed_date: 2026-05-07
iteration: 5
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 06: Code Review Fix Report

**Fixed at:** 2026-05-07
**Source review:** .planning/phases/06-documentary-pipeline/06-REVIEW.md
**Iteration:** 5

**Summary:**
- Findings in scope: 4
- Fixed: 4
- Skipped: 0

## Fixed Issues

### WR-01: Temporary keyframe directory never cleaned up on failure

**Files modified:** `src/documentary/script_gen.rs`
**Commit:** 2779437
**Applied fix:** Added RAII `CleanupOnDrop` guard struct that removes the keyframe cache directory when `analyze_video` fails. On success the guard is cancelled via `cancel()` so files are retained for user inspection. The guard activates on any early return (error path) and is explicitly disabled on the happy path.

### WR-02: Negative timestamps silently clamped to zero in SRT generation

**Files modified:** `src/documentary/subtitle.rs`
**Commit:** f15625b
**Applied fix:** Added `tracing::warn!` in `generate_srt_from_word_boundaries` when `start_secs` or `end_secs` are negative, before the silent clamping in `secs_to_srt_time`. This surfaces calculation bugs to callers without changing the clamping behavior.

### WR-03: SubtitleSegment.offset_secs always 0.0 -- misleading data flow

**Files modified:** `src/documentary/audio.rs`
**Commit:** e790735 (applied by parallel phase-04 fixer; fix verified in HEAD)
**Applied fix:** Changed `merge_subtitle_files` to pass `0.0` offset to `generate_srt_from_word_boundaries` and store `cumulative_offset` in `SubtitleSegment.offset_secs` instead. This ensures offset is applied in exactly one place (`merge_srt_files`) and `offset_secs` carries the actual value.

### WR-04: No file-existence check for video_path and script_path

**Files modified:** `src/documentary/types.rs`
**Commit:** a66839e
**Applied fix:** Added `self.video_path.exists()` and `self.script_path.exists()` checks in `DocumentaryRequest::validate()`, immediately after the empty-path checks. Returns clear error messages with the file path displayed.

## Verification

| Check | Result |
|-------|--------|
| `cargo check` compiles clean | Zero errors (7 pre-existing warnings unrelated to fixes) |
| `cargo test --lib documentary` | 57 passed, 0 failed |
| All fixes re-read verified | Fix text present, surrounding code intact |

---

_Fixed: 2026-05-07_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 5_
