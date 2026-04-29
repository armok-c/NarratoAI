---
phase: 09-jianying-export
fixed_at: 2026-04-29T19:00:00Z
review_path: .planning/phases/09-jianying-export/09-REVIEW.md
iteration: 3
findings_in_scope: 8
fixed: 8
skipped: 0
status: all_fixed
---

# Phase 09: Code Review Fix Report (Iteration 3)

**Fixed at:** 2026-04-29T19:00:00Z
**Source review:** .planning/phases/09-jianying-export/09-REVIEW.md
**Iteration:** 3

**Summary:**
- Findings in scope: 8 (1 Critical + 4 Warning + 3 Info)
- Fixed: 8
- Skipped: 0

## Fixed Issues

### CR-01: `trange_from_secs` allows negative `start` without validation

**Files modified:** `src/jianying/time.rs`
**Commit:** ca358ae
**Applied fix:** Added `is_finite()` and `>= 0` checks for both `start_secs` and `duration_secs` before conversion. Added post-cast `i64` negative check for both values. Added 3 new tests: `test_trange_from_secs_negative_start_returns_none`, `test_trange_from_secs_nan_returns_none`, `test_trange_from_secs_inf_returns_none`.

### WR-01: `calculate_total_duration` silently defaults to 60 seconds for empty timelines

**Files modified:** `src/jianying/builder.rs`
**Commit:** ca358ae
**Applied fix:** Changed `unwrap_or(60_000_000)` to `unwrap_or(0)` — empty timelines now correctly report 0 duration instead of a silent 60-second default.

### WR-02: `probe_audio` does not validate parsed duration is positive and finite

**Files modified:** `src/ffmpeg/probe.rs`
**Commit:** ca358ae
**Applied fix:** Added `!duration.is_finite() || duration <= 0.0` validation after parsing, returning `FFmpegError::OutputParseError` with a descriptive message.

### WR-03: `probe_video` does not validate `duration_secs` is positive and finite

**Files modified:** `src/ffmpeg/probe.rs`
**Commit:** ca358ae
**Applied fix:** Added `!duration_secs.is_finite() || duration_secs <= 0.0` validation after parsing, returning `FFmpegError::OutputParseError` with a descriptive message.

### WR-04: `export_draft` does not validate `video_origin_path` is non-empty

**Files modified:** `src/jianying/builder.rs`
**Commit:** ca358ae
**Applied fix:** Added `video_origin_path.as_os_str().is_empty()` check in `validate_export_request`, returning `JianYingError::Validation` with "原始视频路径不能为空". Added test `test_validate_empty_video_origin_path`.

### IN-01: `DraftFolder::create_draft` takes `&PathBuf` instead of `&Path`

**Files modified:** `src/jianying/builder.rs`
**Commit:** ca358ae
**Applied fix:** Changed parameter type from `&PathBuf` to `&Path`, updated internal clone to `to_path_buf()`. Call sites auto-deref via `Deref<Target=Path>`.

### IN-02: `parse_timestamp_start` uses `f64` for integer-valued hour/minute/second fields

**Files modified:** `src/jianying/builder.rs`
**Commit:** ca358ae
**Applied fix:** Changed `h`, `m`, `s` from `f64` to `u32`. Removed now-unnecessary negative checks (`u32` is always non-negative). Cast to `f64` only for the final arithmetic. Fractional inputs like `"01.5:30:00"` now correctly fail.

### IN-03: `calculate_total_duration` re-serializes track data to JSON just to read timeranges

**Files modified:** `src/jianying/builder.rs`, `src/jianying/track.rs`
**Commit:** ca358ae
**Applied fix:** Added `Track::max_end_time()` method that computes max end time directly from `SegmentOutput`'s `base.target_timerange` without JSON serialization. Simplified `calculate_total_duration` to a one-liner that delegates to `max_end_time()`, changing return type from `Result<i64, JianYingError>` to `i64`.

## Verification Results

- **cargo check**: All fixes compile cleanly.
- **cargo test --lib**: 136 unit tests passed, 0 failed (4 new tests added).
- **cargo test --test jianying_export**: 10 integration tests passed, 2 ignored (require actual audio files and ffmpeg).

---

_Fixed: 2026-04-29T19:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 3_
