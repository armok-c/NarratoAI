---
phase: 08-sdp-pipeline
fixed_at: 2026-05-07T06:23:59Z
review_path: .planning/phases/08-sdp-pipeline/08-REVIEW.md
iteration: 1
findings_in_scope: 8
fixed: 8
skipped: 0
status: all_fixed
---

# Phase 08: Code Review Fix Report

**Fixed at:** 2026-05-07T06:23:59Z
**Source review:** .planning/phases/08-sdp-pipeline/08-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 8 (all Warnings)
- Fixed: 8
- Skipped: 0

## Fixed Issues

### WR-01: Silent timestamp parse failure in SDP clip step

**Files modified:** `narratoai-core/src/sdp/clip.rs`
**Commit:** 3fb2e51
**Applied fix:** Replaced `unwrap_or(0.0)` on `parse_srt_timestamp()` with `map_err` + `?` to propagate parse errors as `SdpError::ParseSubtitle`. Added inverted timestamp range check (`end_secs < start_secs`) returning `SdpError::Validation`.

### WR-02: Zero-duration clips silently skipped without error

**Files modified:** `narratoai-core/src/sdp/clip.rs`
**Commit:** 3fb2e51
**Applied fix:** Replaced `if clip_duration > 0.0` silent skip guard with `if clip_duration <= 0.0` error return (`SdpError::Validation`), matching the SDE pipeline pattern. Applied together with WR-01 as they modify overlapping code in the same block.

### WR-03: Missing file existence checks in SdpRequest::validate

**Files modified:** `narratoai-core/src/sdp/types.rs`
**Commit:** 4b15f3a
**Applied fix:** Added `self.subtitle_path.exists()` and `self.video_path.exists()` checks in `SdpRequest::validate()`, matching SDE's pattern. Updated `test_sdp_request_validate_valid` to create temp files before testing.

### WR-04: Missing newline/control-character guard in SDP concat step

**Files modified:** `narratoai-core/src/sdp/pipeline.rs`
**Commit:** 83d55c1
**Applied fix:** Added newline (`\n`, `\r`) detection before path escaping in `sdp_step_concat`, matching the SDE concat guard pattern. Returns `SdpError::Validation` if path contains illegal characters.

### WR-05: Unused `_config` parameter on public API `run_sdp`

**Files modified:** `narratoai-core/src/sdp/pipeline.rs`, `src-tauri/src/commands/pipeline.rs`
**Commit:** 566513f
**Applied fix:** Removed `_config: &crate::config::types::AppConfig` parameter from `run_sdp` function signature and updated the caller in the Tauri command layer to not pass the argument. Prefixed `config` with `_` in the Tauri command signature.

### WR-06: Synchronous blocking I/O in async context

**Files modified:** `narratoai-core/src/sdp/pipeline.rs`
**Commit:** b27d44a
**Applied fix:** Changed `std::fs::create_dir_all(&task_dir)?` to `tokio::fs::create_dir_all(&task_dir).await?` (line 34) and `std::fs::write(&concat_list_path, &concat_content)?` to `tokio::fs::write(&concat_list_path, &concat_content).await?` (line 107), matching SDE's async I/O patterns.

### WR-07: Redundant TTS skip condition in SDE pipeline

**Files modified:** `narratoai-core/src/sde/pipeline.rs`
**Commit:** 9c9c254
**Applied fix:** Removed the redundant `if clip.narration.starts_with("播放原片") && clip.ost == OstType::OriginalSound { continue; }` block, which was dead code since the subsequent `if clip.ost == OstType::OriginalSound { continue; }` already handles all OST=1 clips. Updated the section comment to remove the stale D-23 reference.

### WR-08: Missing intermediate progress checkpoint in SDP composite step

**Files modified:** `narratoai-core/src/sdp/pipeline.rs`
**Commit:** 681fe7a
**Applied fix:** Added `state.emit_progress("composite", 90.0, "合成完成");` between `sdp_step_composite` and the final 100% progress emission, matching the documented step granularity (`60% to 90% to 100%`).

## Skipped Issues

None -- all 8 warnings were successfully fixed.

---

_Fixed: 2026-05-07T06:23:59Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
