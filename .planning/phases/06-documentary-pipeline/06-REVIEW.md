---
status: issues-found
phase: 06-documentary-pipeline
depth: standard
files_reviewed: 12
date: 2026-05-05
iteration: 2
findings:
  critical: 0
  warning: 2
  info: 4
  total: 6
files_reviewed_list:
  - src/documentary/mod.rs
  - src/documentary/error.rs
  - src/documentary/types.rs
  - src/documentary/timestamp.rs
  - src/documentary/subtitle.rs
  - src/documentary/pipeline.rs
  - src/documentary/clip.rs
  - src/documentary/audio.rs
  - src/documentary/script_gen.rs
  - src/lib.rs
  - tests/documentary_integration_test.rs
  - tests/common/mod.rs
previous_review:
  date: 2026-05-05
  findings: 11 (2 CR + 4 WR + 5 IN)
  fixed: 7
  remaining: 4
---

# Code Review: Phase 06 -- Documentary Pipeline (Re-Review #2)

## Summary

Second review of 12 Rust source files. Previous review found 11 issues (2 CR + 4 WR + 5 IN); 7 were fixed and 4 deferred as design decisions. This re-review confirms all 7 fixes remain intact, the 4 deferred issues persist, and identifies 1 new info-level finding (`threads` field unused). Code quality is good overall: well-structured modules, proper `thiserror` error handling, comprehensive unit tests for timestamp and subtitle logic. No critical or security issues remain.

## Previous Fix Verification

| ID | Issue | Status | Verified |
|----|-------|--------|----------|
| CR-01 | BGM aloop + amix 超长输出 | Fixed | ✅ pipeline.rs:326-329 `atrim=0:{total_dur}` caps BGM |
| CR-02 | SRT path semicolon injection | Fixed | ✅ pipeline.rs:359 `.replace(';', "\\;")` |
| WR-01 | video_path/script_path validation | Fixed | ✅ types.rs:54-59 non-empty checks |
| WR-03 | subtitle_font/color/position unused | Fixed | ✅ pipeline.rs:288-305 `force_style` uses all 3 fields |
| WR-04 | collect_keyframe_paths silent errors | Fixed | ✅ script_gen.rs:385-401 explicit `map_err` |
| IN-04 | Extreme f64 time values | Fixed | ✅ timestamp.rs:86,102 `.min(86399.999)` |
| IN-05 | FFmpeg Progress events ignored | Fixed | ✅ clip.rs:182-183 `tracing::trace!` |

## Warning Findings

### WR-01: `original_volume` validated but never applied in composite step

**File:** `src/documentary/types.rs:15` and `src/documentary/pipeline.rs:263-438`
**Category:** logic error (carried over from previous review)
**Previous ID:** WR-02

`DocumentaryRequest::original_volume` (default 0.7) is validated in `validate()` (types.rs:70-72) but never referenced in `step_composite`. When clips have `OstType::Mixed`, the original video audio is preserved during clipping but no volume adjustment is applied in the final composite. Users who set `original_volume` expect the original audio level to change in the output, but it is silently ignored.

**Impact:** User-configurable original audio volume has no effect on output.

### WR-02: `video_aspect` field defined but never used

**File:** `src/documentary/types.rs:23` and `src/documentary/pipeline.rs:263-438`
**Category:** dead code (carried over from previous WR-03 partial fix)

`DocumentaryRequest::video_aspect` (default `"9:16"`) is never referenced anywhere in the pipeline. No FFmpeg `-aspect` or scale filter is applied based on this value. The previous fix addressed `subtitle_font`, `subtitle_color`, and `subtitle_position` from the original WR-03, but `video_aspect` was not included.

**Impact:** User-configurable video aspect ratio setting has no effect.

## Info Findings

### IN-01: `SubtitleSegment.offset_secs` always 0.0 -- confusing dual-offset pattern

**File:** `src/documentary/audio.rs:153-156`
**Category:** code clarity (carried over from previous review)

In `merge_subtitle_files`, `SubtitleSegment::offset_secs` is always set to `0.0`. The actual time offset is applied in `generate_srt_from_word_boundaries` via the `cumulative_offset` parameter. The `offset_secs` field and `apply_offset_to_block` logic in `merge_srt_files` are effectively dead code for this call path.

### IN-02: `PipelineState.progress` always None -- progress callbacks never fire

**File:** `src/documentary/pipeline.rs:468`
**Category:** dead code (carried over from previous review)

`PipelineState.progress` is hardcoded to `None` in `run_documentary`. The `emit_progress` method and all progress callback invocations are dead code. `DocumentaryRequest` has no `progress` field and `run_documentary` accepts no callback parameter.

### IN-03: Integration tests ignored with empty bodies

**File:** `tests/documentary_integration_test.rs:166-203`
**Category:** test quality (carried over from previous review)

Four integration tests are marked `#[ignore = "需要 FFmpeg"]` and contain empty bodies. There is zero end-to-end pipeline test coverage. Even the `ffmpeg_available()` guard inside these tests is pointless since they're already `#[ignore]`.

### IN-04: `threads` field validated but never passed to FFmpeg

**File:** `src/documentary/types.rs:25` and `src/documentary/pipeline.rs`, `src/documentary/clip.rs`
**Category:** dead code (new finding)

`DocumentaryRequest::threads` (default 4) is validated as non-zero in `validate()` (types.rs:76-78) but is never passed to any FFmpeg command. Neither `step_composite` nor `run_clip_ffmpeg` uses the `-threads` flag. Users who set `threads` expect parallelism control, but FFmpeg defaults are used instead.

**Fix:** Pass `-threads` to FFmpeg commands:
```rust
cmd.arg("-threads").arg(request.threads.to_string());
```

## Files Reviewed

| File | Lines | Findings |
|------|-------|----------|
| src/documentary/mod.rs | 14 | 0 |
| src/documentary/error.rs | 205 | 0 |
| src/documentary/types.rs | 106 | 0 WR, 0 IN (fields defined here, issues in pipeline.rs) |
| src/documentary/timestamp.rs | 249 | 0 |
| src/documentary/subtitle.rs | 243 | 0 |
| src/documentary/pipeline.rs | 563 | 2 WR (WR-01, WR-02) |
| src/documentary/clip.rs | 243 | 0 |
| src/documentary/audio.rs | 273 | 0 (IN-01 affects this file but is structural) |
| src/documentary/script_gen.rs | 522 | 0 |
| src/lib.rs | 30 | 0 |
| tests/documentary_integration_test.rs | 242 | 0 (IN-03 is test quality, not code defect) |
| tests/common/mod.rs | 13 | 0 |

## Recommendations

1. **WR-01 (original_volume)** -- Either implement volume adjustment in `step_composite` for the original audio stream, or remove the field from `DocumentaryRequest` to avoid misleading users.
2. **WR-02 (video_aspect)** -- Either apply aspect ratio scaling via FFmpeg `scale` filter, or remove the field.
3. **IN-04 (threads)** -- Pass `-threads` to FFmpeg commands or remove the field.
4. **IN-01/IN-02/IN-03** -- Deferred items from previous review; track as tech debt.

---

_Reviewed: 2026-05-05_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 2_
