---
phase: 04-prompt-system-visual-analyzer
fixed_at: 2026-04-30T15:00:00Z
review_path: .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
iteration: 2
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 4: Code Review Fix Report

**Fixed at:** 2026-04-30T15:00:00Z
**Source review:** .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
**Iteration:** 2

**Summary:**
- Findings in scope: 3 (re-review warnings)
- Fixed: 3
- Skipped: 0

## Fixed Issues

### WR-01: `analyze_video_frames` discards `overall_activity_summary` from LLM responses

**Files modified:** `src/visual/analyzer.rs`, `src/prompt/validators.rs`
**Commit:** 974c0da
**Applied fix:** Introduced `ParsedBatch` struct as return type of `parse_and_retry`, carrying both `observations` and `overall_activity_summary`. Updated `analyze_video_frames` to accumulate the last non-empty summary across batches and pass it to `BatchAnalysisResult`. Removed `#[allow(dead_code)]` from `BatchResponse`. Updated all unit tests to access `.observations` on the new return type. Also fixed `test_narration_too_few_paragraphs` test string which was under 50 characters when counted correctly (related to WR-02 fix).

### WR-02: `validators.rs` uses byte length while error messages claim "character" count

**Files modified:** `src/prompt/validators.rs`
**Commit:** 3bf4d35
**Applied fix:** Replaced `trimmed.len()` with `trimmed.chars().count()` in both `validate_narration_script` (threshold 50) and `validate_plot_analysis` (threshold 100) so error messages accurately report character counts. For Chinese text (CJK = 3 UTF-8 bytes per char), the previous byte-based thresholds were ~3x weaker than documented.

### WR-03: `convert_image_to_jpeg` uses hardcoded quality 85, ignoring caller-provided quality parameter

**Files modified:** `src/visual/frame_extractor.rs`
**Commit:** fa3ee3e
**Applied fix:** Added `ffmpeg_quality: u32` parameter to `convert_image_to_jpeg` and implemented scale mapping from FFmpeg `-q:v` (2-31, lower=better) to image crate quality (1-100, higher=better). Updated both caller sites (Level 3 PNG and Level 4 BMP fallback) to pass the quality parameter through.

## Verification

- All 273 library unit tests pass (`cargo test --lib`)
- Zero compiler warnings after fixes
- Integration test `test_clip_video_async` fails due to FFmpeg not processing test video (pre-existing, unrelated)

## Skipped Issues

None -- all in-scope findings were fixed.

---

_Fixed: 2026-04-30T15:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 2_
