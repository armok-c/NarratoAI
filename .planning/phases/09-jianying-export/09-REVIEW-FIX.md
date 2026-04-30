---
phase: 09-jianying-export
fixed_at: 2026-04-30T10:20:00Z
review_path: .planning/phases/09-jianying-export/09-REVIEW.md
iteration: 7
findings_in_scope: 5
fixed: 5
skipped: 0
status: all_fixed
---

# Phase 09: Code Review Fix Report (Iteration 7)

**Fixed at:** 2026-04-30T10:20:00Z
**Source review:** `.planning/phases/09-jianying-export/09-REVIEW.md`
**Iteration:** 7

**Summary:**
- Findings in scope: 5 (WR-01 through WR-05)
- Fixed: 5
- Skipped: 0

## Fixed Issues

### WR-01: Track does not enforce segment type matching track_type

**Files modified:** `src/jianying/track.rs`, `src/jianying/builder.rs`
**Commit:** 0af21ec
**Applied fix:** Added runtime validation in `add_video_segment` and `add_audio_segment` to return `JianYingError::Validation` when segment type does not match `track_type`. Updated callers in `builder.rs` to propagate `Result`. Added two new tests for type mismatch rejection.

### WR-02: VideoMaterial::new and AudioMaterial::new silently accept invalid paths

**Files modified:** `src/jianying/material.rs`, `src/jianying/segment.rs`, `src/jianying/track.rs`, `tests/jianying_export.rs`
**Commit:** e27442f
**Applied fix:** Added `is_absolute()` check after `canonicalize()` in both `VideoMaterial::new` and `AudioMaterial::new`. Returns `JianYingError::Validation` for non-absolute paths. Updated all unit tests (material, segment, track) to use temp files with absolute paths. Updated integration tests to create real temp files and pass absolute paths.

### WR-03: export_draft leaks draft directory on partial failure

**Files modified:** `src/jianying/builder.rs`
**Commit:** 6ae7b38
**Applied fix:** Extracted `build_timeline` helper function. In `export_draft`, wrapped `build_timeline` call in a match -- on error, calls `std::fs::remove_dir_all` on the draft directory to clean up orphaned state.

### WR-04: trange_from_secs accepts zero duration

**Files modified:** `src/jianying/time.rs`
**Commit:** 7681095
**Applied fix:** Changed duration check from `< 0.0` to `<= 0.0` in `trange_from_secs` and from `< 0` to `<= 0` in `trange`. Updated `test_trange_zero_seconds` to expect `None`. Added `test_trange_from_secs_zero_duration_returns_none` test.

### WR-05: build_base_segment duplicated between VideoSegment and AudioSegment

**Files modified:** `src/jianying/segment.rs`, `tests/jianying_export.rs`
**Commit:** 4121f7f
**Applied fix:** Extracted shared `build_base_segment` free function taking `segment_id`, `material_id`, `target_timerange`, `source_timerange`, and `speed_id` as parameters. Both `VideoSegment::to_json` and `AudioSegment::to_json` now call the shared helper, eliminating 50 lines of duplication.

## Verification Results

- **cargo test --lib:** 144 unit tests passed, 0 failed
- **cargo test --test jianying_export:** 10 integration tests passed, 2 ignored (require ffmpeg), 0 failed

---

_Fixed: 2026-04-30T10:20:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 7_
