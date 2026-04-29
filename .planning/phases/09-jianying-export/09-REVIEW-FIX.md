---
phase: 09-jianying-export
fixed_at: 2026-04-29T14:30:00Z
review_path: .planning/phases/09-jianying-export/09-REVIEW.md
iteration: 1
findings_in_scope: 12
fixed: 12
skipped: 0
status: all_fixed
---

# Phase 9: Code Review Fix Report

**Fixed at:** 2026-04-29T14:30:00Z
**Source review:** .planning/phases/09-jianying-export/09-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 12
- Fixed: 12
- Skipped: 0

## Fixed Issues

### CR-01: Silent segment loss when track name does not match

**Files modified:** `src/jianying/builder.rs`
**Commit:** e0ce892
**Applied fix:** Changed `add_video_segment` and `add_audio_segment` return types from `()` to `Result<(), JianYingError>`. When the target track name is not found, the already-pushed materials are rolled back (popped) and a `Validation` error is returned. Updated all callers in `export_draft` to propagate errors via `?`.

### CR-02: parse_timestamp_start mishandles comma-delimited milliseconds

**Files modified:** `src/jianying/builder.rs`
**Commit:** 381931c
**Applied fix:** Replaced `split('-').next()` with `rsplit_once('-')` to correctly split on the last hyphen, avoiding collision with negative signs and comma separators. Added validation that parsed hours, minutes, seconds are non-negative and that minutes and seconds are less than 60. Milliseconds are also validated as non-negative.

### WR-01: No validation for negative duration in Timerange

**Files modified:** `src/jianying/time.rs`, `src/jianying/segment.rs`, `src/jianying/track.rs`
**Commit:** 4517ae6
**Applied fix:** Added negative duration validation in `trange()`. If duration_us < 0, `trange` returns `None`. Added test `test_trange_negative_duration_returns_none`.

### WR-02: parse_seconds silently returns 0 on invalid input

**Files modified:** `src/jianying/time.rs`
**Commit:** 4517ae6
**Applied fix:** Changed `parse_seconds` return type from `i64` to `Option<i64>`. Instead of `unwrap_or(0.0)`, uses `?` to propagate parse failures as `None`. `trange` now returns `Option<Timerange>`. Added test `test_parse_seconds_invalid_returns_none`. Updated all test callers in segment.rs and track.rs to handle the Option return.

### WR-03: Unnecessary clone() on freshly constructed String in material.rs

**Files modified:** `src/jianying/material.rs`
**Commit:** 40a0160
**Applied fix:** Removed `id.clone()` in both `VideoMaterial::new` and `AudioMaterial::new`. The freshly constructed UUID string is now moved directly into `material_id`.

### WR-04: serde(flatten) collision risk in VideoSegmentJson / AudioSegmentJson

**Files modified:** `src/jianying/types.rs`
**Commit:** 66d4015
**Applied fix:** Added documentation comments to both structs explaining the `#[serde(flatten)]` constraint. Notes that the structs are serialization-only (safe) and documents the risk if deserialization is ever attempted.

### WR-05: Track::to_json() uses expect() for serialization in production code

**Files modified:** `src/jianying/track.rs`
**Commit:** b7f3150
**Applied fix:** Changed `to_json` return type from `TrackJson` to `Result<TrackJson, JianYingError>`. Replaced `expect()` with `map_err(JianYingError::JsonSerialize)` and `collect::<Result<Vec<_>, _>>()`. Updated all test callers.

### WR-06: ScriptFile::save() also uses expect() for track serialization

**Files modified:** `src/jianying/builder.rs`
**Commit:** b7f3150
**Applied fix:** Replaced `expect("Track serialization should succeed")` with `?` operator. The `t.to_json()?` now returns `Result`, and `serde_json::to_value` error is mapped via `map_err(JianYingError::JsonSerialize)`. Also updated `calculate_total_duration` to return `Result<i64, JianYingError>`.

### WR-07: Integration tests use parent().unwrap() without descriptive message

**Files modified:** `tests/jianying_export.rs`
**Commit:** c24a400
**Applied fix:** Replaced all `.parent().unwrap()` calls with `.parent().expect("descriptive message")` providing context-specific panic messages.

### IN-01: Unused id local variable in VideoMaterial::new and AudioMaterial::new

**Files modified:** `src/jianying/material.rs`
**Commit:** 40a0160
**Applied fix:** Fixed together with WR-03. The `id` local variable is now moved directly into `material_id` without clone, so there is no unused variable.

### IN-02: DraftContentJson and related structs defined but unused

**Files modified:** `src/jianying/types.rs`
**Commit:** 8a2cf12
**Applied fix:** Removed `MaterialsJson`, `CanvasConfig`, `ProjectConfig`, and `DraftContentJson` structs. The builder uses template string replacement and dynamic `serde_json::Value` manipulation instead.

### IN-03: probe_video_async exists but is never called

**Files modified:** `src/ffmpeg/probe.rs`
**Commit:** e671e66
**Applied fix:** Removed the unused `probe_video_async` function. If async support is needed in the future, it can be re-added with proper feature gating.

## Skipped Issues

None -- all findings were fixed.

## Verification Results

- **cargo check**: All fixes compile cleanly.
- **cargo test --lib**: 131 unit tests passed, 0 failed.
- **cargo test --test jianying_export**: 10 integration tests passed, 2 ignored (require actual audio files and ffmpeg).
- **Pre-existing failure**: `test_clip_video_async` in `tests/ffmpeg_test.rs` fails independently of these fixes (FFmpeg execution error unrelated to jianying module changes).

---

_Fixed: 2026-04-29T14:30:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
