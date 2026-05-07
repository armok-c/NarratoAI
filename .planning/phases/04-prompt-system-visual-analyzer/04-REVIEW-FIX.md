---
phase: 04-prompt-system-visual-analyzer
fixed_at: 2026-05-07T15:30:00Z
review_path: .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
iteration: 1
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 04: Code Review Fix Report

**Fixed at:** 2026-05-07T15:30:00Z
**Source review:** `.planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 2
- Fixed: 2
- Skipped: 0

## Fixed Issues

### WR-01: FrameObservation::validate() defined but never called in the analysis pipeline

**Files modified:** `narratoai-core/src/visual/analyzer.rs`
**Commit:** `31a9128`
**Applied fix:** In `analyze_video_frames()` Step 7, after calling `parse_and_retry()` for each batch, filter the resulting `observations` vector through `FrameObservation::validate()`. Observations with out-of-range `visual_salience` values (outside `[0.0, 1.0]`) are rejected, and each rejection is logged via `warn!()`. This ensures the public `validate()` method is actually exercised in the analysis pipeline as originally intended.

### WR-02: Fallback frame extraction uses system PATH ffmpeg/ffprobe inconsistent with fast path

**Files modified:** `narratoai-core/src/visual/frame_extractor.rs`
**Commit:** `a180a9d`
**Applied fix:** Replaced `std::process::Command::new("ffprobe")` with `ffmpeg_sidecar::ffprobe::ffprobe_path()` in `get_video_duration()`, and `std::process::Command::new("ffmpeg")` with `ffmpeg_sidecar::paths::ffmpeg_path()` in `run_ffmpeg_with_cancel()`. Added the necessary imports. Updated doc comments to reflect the consistent binary discovery strategy. Both fallback helpers now resolve FFmpeg/FFprobe through `ffmpeg_sidecar`, matching the fast path and eliminating the risk of inconsistent binaries.

---

_Fixed: 2026-05-07T15:30:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
