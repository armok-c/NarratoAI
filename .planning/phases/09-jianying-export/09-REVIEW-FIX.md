---
phase: 09-jianying-export
fixed_at: 2026-04-29T17:00:00Z
review_path: .planning/phases/09-jianying-export/09-REVIEW.md
iteration: 2
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 9: Code Review Fix Report (Iteration 2)

**Fixed at:** 2026-04-29T17:00:00Z
**Source review:** .planning/phases/09-jianying-export/09-REVIEW.md
**Iteration:** 2

**Summary:**
- Findings in scope: 4
- Fixed: 4
- Skipped: 0

## Fixed Issues

### WR-01: `trange_from_secs` accepts negative duration without validation

**Files modified:** `src/jianying/time.rs`, `src/jianying/builder.rs`
**Commit:** b44d21f
**Applied fix:** Changed `trange_from_secs` return type from `Timerange` to `Option<Timerange>`, adding negative duration validation (`duration_us < 0` returns `None`). Updated all 3 callers in `builder.rs` (lines 275, 287, 307) to handle `Option` with `.ok_or_else(|| JianYingError::Validation { ... })`. Added new test `test_trange_from_secs_negative_duration_returns_none` and updated existing `test_trange_from_secs` to use `.expect()`.

### WR-02: `probe_video` only accepts string-typed duration from ffprobe JSON

**Files modified:** `src/ffmpeg/probe.rs`
**Commit:** b44d21f
**Applied fix:** Added `.or_else(|| json["format"]["duration"].as_f64())` fallback after the existing `.as_str().and_then(...)` chain, so the function handles both string (`"120.5"`) and number (`120.5`) JSON representations of duration.

### WR-03: `export_draft` does not validate duration is positive

**Files modified:** `src/jianying/builder.rs`
**Commit:** b44d21f
**Applied fix:** Added `if duration <= 0.0` check immediately after extracting `duration` from `clip.duration`, returning `JianYingError::Validation` with a descriptive message including clip index and actual value (e.g., "第 1 段时长必须为正数，实际: -5").

### WR-04: `calculate_total_duration` silently swallows serialization errors

**Files modified:** `src/jianying/builder.rs`
**Commit:** b44d21f
**Applied fix:** Replaced `filter_map` with `.map(|track| track.to_json()).collect::<Result<Vec<_>, _>>()?` to properly propagate `JianYingError` from `to_json()` calls, followed by `.iter().filter_map(...)` on the collected `Vec<TrackJson>` for the max-end calculation.

## Verification Results

- **cargo check**: All fixes compile cleanly.
- **cargo test --lib**: 132 unit tests passed, 0 failed.
- **cargo test --test jianying_export**: 10 integration tests passed, 2 ignored (require actual audio files and ffmpeg).

---

_Fixed: 2026-04-29T17:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 2_
