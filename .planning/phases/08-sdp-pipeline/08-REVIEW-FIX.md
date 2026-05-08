---
phase: 08-sdp-pipeline
fixed_at: 2026-05-08T21:00:00Z
review_path: .planning/phases/08-sdp-pipeline/08-REVIEW.md
iteration: 9
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 08: Code Review Fix Report

**Fixed at:** 2026-05-08T21:00:00Z
**Source review:** .planning/phases/08-sdp-pipeline/08-REVIEW.md
**Iteration:** 9

**Summary:**
- Findings in scope: 3
- Fixed: 3
- Skipped: 0

## Fixed Issues

### CR-01: Documentary pipeline subtitles filter still wraps path in single quotes

**Files modified:** `narratoai-core/src/documentary/pipeline.rs`
**Commit:** dea19eb
**Applied fix:** Changed the FFmpeg subtitles filter format from `subtitles='{}'` (single-quoted path) to `subtitles={}` (unquoted path) at line 400. This aligns with the WR-19 fix already applied to `sde/pipeline.rs` and avoids the bug where paths containing single quotes break FFmpeg's filter parser. The existing escaping chain already handles special characters via FFmpeg's native escaping conventions.

### WR-21: validate_timestamp in script/mod.rs still rejects hours > 23

**Files modified:** `narratoai-core/src/script/mod.rs`
**Commit:** 03d592c
**Applied fix:** Removed the `h > 23` check from `timestamp_to_millis` on line 28, changing `if h > 23 || m > 59 || s > 59` to `if m > 59 || s > 59`. This makes `validate_timestamp` consistent with the relaxed `parse_srt_timestamp` and `parse_time_to_secs` parsers (fixed in WR-18). Updated the corresponding test to assert that hours > 23 now passes validation.

### WR-22: SDP pipeline has no timestamp overlap detection

**Files modified:** `narratoai-core/src/sdp/pipeline.rs`
**Commit:** 01db339
**Applied fix:** Added timestamp overlap detection after OST validation (G2 guard). Uses `parse_srt_timestamp` from `crate::subtitle::timestamp` to parse adjacent clip boundaries and returns `SdpError::Validation` with a descriptive Chinese error message when overlapping timestamps are detected. Inline implementation (24 lines) avoids cross-module coupling with SDE's private `detect_and_abort_overlaps` function.

---
_Fixed: 2026-05-08T21:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 9_
