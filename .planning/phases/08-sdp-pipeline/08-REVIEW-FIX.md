---
phase: 08-sdp-pipeline
fixed_at: 2026-05-08T15:40:00Z
review_path: .planning/phases/08-sdp-pipeline/08-REVIEW.md
iteration: 8
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 08: Code Review Fix Report

**Fixed at:** 2026-05-08T15:40:00Z
**Source review:** .planning/phases/08-sdp-pipeline/08-REVIEW.md
**Iteration:** 8

**Summary:**
- Findings in scope: 3
- Fixed: 3
- Skipped: 0

## Fixed Issues

### WR-18: parse_srt_timestamp rejects valid SRT timestamps with hours > 23

**Files modified:** `narratoai-core/src/subtitle/timestamp.rs`, `narratoai-core/src/documentary/timestamp.rs`
**Commit:** 21c5f1b
**Applied fix:** Removed the `h > 23` check from `parse_srt_timestamp` and the `!(0.0..=23.0).contains(&h)` check from `parse_time_to_secs`. Updated total seconds validation from 86400 (1 day) to 604800 (7 days) in both parsers. Updated `secs_to_srt_time` and `secs_to_ffmpeg_time` caps from 86399.999 to 604799.999 in the documentary module.

### WR-19: SRT path containing single quote breaks FFmpeg subtitles filter

**Files modified:** `narratoai-core/src/sde/pipeline.rs`
**Commit:** d926892
**Applied fix:** Changed the FFmpeg subtitles filter format from `subtitles='{}'` (single-quoted) to `subtitles={}` (unquoted). The existing escaping chain at lines 525-533 already handles special characters via FFmpeg's native escaping, so removing the single-quote wrapping avoids the issue where paths containing single quotes break FFmpeg's filter parser.

### WR-20: Millisecond parsing inconsistency between two timestamp parsers

**Files modified:** `narratoai-core/src/subtitle/timestamp.rs`, `narratoai-core/src/documentary/timestamp.rs`
**Commit:** 38db894
**Applied fix:** Standardized both `parse_srt_timestamp` and `parse_time_to_secs` to use left-pad-to-3-digits approach: `format!("{:0<3}", millis)` then take first 3 characters, parse, and divide by 1000. This ensures consistent behavior for non-standard 1 or 2 digit millisecond inputs across both the subtitle and documentary parsers.

---

_Fixed: 2026-05-08T15:40:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 8_
