---
phase: 08-sdp-pipeline
fixed_at: 2026-05-08T00:00:00Z
review_path: .planning/phases/08-sdp-pipeline/08-REVIEW.md
iteration: 2
findings_in_scope: 9
fixed: 9
skipped: 0
status: all_fixed
---

# Phase 08: Code Review Fix Report

**Fixed at:** 2026-05-08T00:00:00Z
**Source review:** .planning/phases/08-sdp-pipeline/08-REVIEW.md
**Iteration:** 2

**Summary:**
- Findings in scope: 9 (1 Critical + 8 Warnings)
- Fixed: 9
- Skipped: 0

## Fixed Issues

### CR-01: subtitle_segments never populated in SDP pipeline

**Files modified:** `narratoai-core/src/sdp/pipeline.rs`
**Commit:** f12e83e
**Applied fix:** Added `parse_subtitle_file` call via `tokio::task::spawn_blocking` before `sdp_step_clip` to populate `state.subtitle_segments`. This enables the OST=1 precise timestamp correction in `find_precise_range` to work correctly instead of always receiving an empty slice.
**Note:** This is a logic fix -- requires human verification to confirm the subtitle parsing and segment population works end-to-end.

### WR-01: Silent timestamp parse failure in SDP clip step

**Files modified:** `narratoai-core/src/sdp/clip.rs`
**Commit:** af2eca0
**Applied fix:** Replaced `unwrap_or(0.0)` with `map_err` to propagate `parse_srt_timestamp` errors as `SdpError::ParseSubtitle`. Added inverted range check (`end_secs < start_secs`) returning `SdpError::Validation`.

### WR-02: Zero-duration clips silently skipped without error

**Files modified:** `narratoai-core/src/sdp/clip.rs`
**Commit:** af2eca0
**Applied fix:** Replaced silent `if clip_duration > 0.0` skip with explicit error return `SdpError::Validation` when `clip_duration <= 0.0`. This prevents silent timeline gaps and incorrect BGM fade-out timing.

### WR-03: Missing file existence checks in SdpRequest validate

**Files modified:** `narratoai-core/src/sdp/types.rs`
**Commit:** 181dad3
**Applied fix:** Added `.exists()` checks for `subtitle_path` and `video_path` after the empty-path checks, matching SDE pipeline's validation pattern.

### WR-04: Missing newline/control-character guard in SDP concat step

**Files modified:** `narratoai-core/src/sdp/pipeline.rs`
**Commit:** f12e83e
**Applied fix:** Added `\n` and `\r` validation on path strings in `sdp_step_concat`, matching SDE pipeline's pattern. Split the replace chain to check for control characters before escaping quotes.

### WR-05: Unused _config parameter on run_sdp

**Files modified:** `narratoai-core/src/sdp/pipeline.rs`, `src-tauri/src/commands/pipeline.rs`
**Commit:** f12e83e
**Applied fix:** Removed `_config: &crate::config::types::AppConfig` parameter from `run_sdp` public API. Updated Tauri command layer to prefix the now-unused state parameter with `_` to suppress warnings.

### WR-06: Synchronous blocking I/O in async context

**Files modified:** `narratoai-core/src/sdp/pipeline.rs`
**Commit:** f12e83e
**Applied fix:** Replaced `std::fs::create_dir_all` with `tokio::fs::create_dir_all().await` and `std::fs::write` with `tokio::fs::write().await` in `sdp_step_concat`.

### WR-07: Redundant TTS skip condition in SDE pipeline

**Files modified:** `narratoai-core/src/sde/pipeline.rs`
**Commit:** 9a612c2
**Applied fix:** Removed the first `if` block checking `clip.narration.starts_with("播放原片") && clip.ost == OstType::OriginalSound` since the second block already handles all `OstType::OriginalSound` clips.

### WR-08: Missing 90% progress checkpoint in SDP composite step

**Files modified:** `narratoai-core/src/sdp/pipeline.rs`
**Commit:** f12e83e
**Applied fix:** Added `state.emit_progress("composite", 90.0, "合成完成")` between the `sdp_step_composite` call and the final 100% emission, matching the documented 60% -> 90% -> 100% step granularity.

## Skipped Issues

None -- all 9 findings (1 Critical + 8 Warnings) were successfully fixed.

---

_Fixed: 2026-05-08T00:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 2_
