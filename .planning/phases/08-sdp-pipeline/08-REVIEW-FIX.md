---
phase: 08-sdp-pipeline
fixed_at: 2026-05-08
review_path: .planning/phases/08-sdp-pipeline/08-REVIEW.md
iteration: 5
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 08: Code Review Fix Report

**Fixed at:** 2026-05-08
**Source review:** .planning/phases/08-sdp-pipeline/08-REVIEW.md
**Iteration:** 5

**Summary:**
- Findings in scope: 3 (1 Critical, 2 Warning)
- Fixed: 3
- Skipped: 0

## Fixed Issues

### CR-03: SDE pipeline computes `source_time_range` incorrectly for OST=0/OST=2 clips using TTS duration instead of source timestamp end

**Files modified:** `narratoai-core/src/sde/pipeline.rs`
**Commit:** ea8b7e6
**Applied fix:** Replaced `end_secs = start_secs + clip_duration` (line 249) with direct timestamp parsing using `splitn(2, '-')`. Now parses both `start_secs` and `source_end_secs` from the timestamp field directly, ensuring `source_time_range` always reflects the source video time range. Added validation for missing `-` separator. `clip_duration` still determines `edited_time_range` and `cumulative_time` as before.

### WR-12: `SdpRequest::validate` accepts non-existent file paths in tests

**Files modified:** `narratoai-core/src/sdp/types.rs`
**Commit:** ecb3fef
**Applied fix:** Replaced hardcoded relative paths `"sub.srt"` / `"video.mp4"` in `test_sdp_request_validate_valid` with `tempfile::TempDir` and actual file creation, matching the SDE test pattern at `sde/types.rs:182-194`.

### WR-13: `parse_srt_timestamp` normalizes ALL dots to commas including within SS field

**Files modified:** `narratoai-core/src/subtitle/timestamp.rs`
**Commit:** ad8e44e
**Applied fix:** Replaced `input.replace('.', ",")` (global replacement) with targeted normalization that only replaces dots in the suffix after the last colon. This ensures only millis separator dots (e.g., `HH:MM:SS.mmm`) are normalized while dots elsewhere are left untouched.

## Skipped Issues

None -- all 3 findings were successfully fixed.

---
_Fixed: 2026-05-08_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 5_
