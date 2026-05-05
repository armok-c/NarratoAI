---
status: clean
phase: 06-documentary-pipeline
depth: standard
files_reviewed: 12
date: 2026-05-05
iteration: 3
findings:
  critical: 0
  warning: 0
  info: 4
  total: 4
files_reviewed_list:
  - src/documentary/mod.rs
  - src/documentary/error.rs
  - src/documentary/types.rs
  - src/documentary/timestamp.rs
  - src/documentary/subtitle.rs
  - src/documentary/pipeline.rs
  - src/documentary/clip.rs
  - src/documentary/audio.rs
  - src/documentary/script_gen.rs
  - src/lib.rs
  - tests/documentary_integration_test.rs
  - tests/common/mod.rs
previous_review:
  date: 2026-05-05
  iteration: 2
  findings: 6 (0 CR + 2 WR + 4 IN)
  fixed: 2
  remaining: 4
---

# Code Review: Phase 06 -- Documentary Pipeline (Iteration 3)

## Summary

Third iteration review of 12 Rust source files in the documentary pipeline module. This iteration focused on verifying the fixes from commit 8177521 (WR-01: apply `original_volume` in composite audio mix; WR-02: remove unused `video_aspect` field) and checking for any regressions or new issues introduced by the changes.

Both warnings from iteration 2 have been correctly fixed. All previously fixed items from iteration 1 remain intact. No new issues were introduced by the fix commit. The 4 deferred info-level items persist and none warrant elevation to Warning or Critical.

## Fix Verification (Iteration 2 -> 3)

### WR-01 FIX VERIFIED: `original_volume` now applied in composite audio mix

**Commit:** 8177521

The fix correctly applies `original_volume` in `step_composite`:

1. `orig_vol` is extracted from `request.original_volume` at `pipeline.rs:286`
2. `has_original_audio` is computed from script clips at `pipeline.rs:287` using `state.script.iter().any(|c| c.ost != OstType::NarrationOnly)` -- correctly identifies when the concatenated video contains audio from OST=1/2 clips
3. Volume filter `[0:a]volume={orig_vol}[orig]` is applied at `pipeline.rs:319` when `has_original_audio` is true
4. The `[orig]` label is correctly included in the `amix` input chain at `pipeline.rs:346-347`
5. `amix_input_count` is incremented and matches the actual label count passed to `amix`
6. The `external_audio_count` variable correctly tracks external input indices for BGM

All combinations verified correct:
- `has_original_audio=true` + TTS + BGM: 3-input `amix` with `[orig][tts][bgm]`
- `has_original_audio=true` + TTS, no BGM: 2-input `amix` with `[orig][tts]`
- `has_original_audio=false` + TTS + BGM: 2-input `amix` with `[tts][bgm]`
- `has_original_audio=false` + TTS, no BGM: 1-input `amix` with `[tts]`

The FFmpeg filter graph correctly references `[0:a]` (video file audio) and `[0:v]` (video file video) as separate streams without conflict.

### WR-02 FIX VERIFIED: `video_aspect` field removed

**Commit:** 8177521

- Field removed from `DocumentaryRequest` struct in `types.rs`
- Default value `"9:16"` removed from `impl Default`
- No validation reference remains in `validate()`
- Zero references to `video_aspect` exist anywhere in the Rust codebase (confirmed via grep)

### Previously Fixed Items (Iterations 1-2) -- All Intact

| ID | Issue | Status |
|----|-------|--------|
| CR-01 | BGM aloop + amix producing excessively long output | Fixed, intact |
| CR-02 | SRT path semicolon injection in FFmpeg filter | Fixed, intact |
| WR-01 (iter1) | video_path/script_path empty-string validation | Fixed, intact |
| WR-03 | subtitle_font/color/position unused | Fixed, intact |
| WR-04 | collect_keyframe_paths silently discards errors | Fixed, intact |
| IN-04 (iter1) | Extreme f64 time values | Fixed, intact |
| IN-05 (iter1) | FFmpeg Progress events ignored | Fixed, intact |

## Info Findings (Deferred, Reassessed)

### IN-01: `SubtitleSegment.offset_secs` always 0.0 -- confusing dual-offset pattern

**File:** `src/documentary/audio.rs:153-156`
**Category:** code clarity (deferred from iteration 2)

In `merge_subtitle_files`, `SubtitleSegment::offset_secs` is always set to `0.0`. The actual time offset is applied upstream in `generate_srt_from_word_boundaries` via the `cumulative_offset` parameter. The `offset_secs` field and `apply_offset_to_block` in `merge_srt_files` are effectively dead code for this call path.

**Reassessment:** No change. The logic is correct (no double-offset). The field is misleading but not a bug. Stays Info.

### IN-02: `PipelineState.progress` always None -- progress callbacks never fire

**File:** `src/documentary/pipeline.rs:481`
**Category:** dead code (deferred from iteration 2)

`PipelineState.progress` is hardcoded to `None` in `run_documentary`. The `emit_progress` method and all progress callback invocations are dead code until the pipeline is integrated with a caller that passes a progress callback.

**Reassessment:** No change. Intentional design for future integration. Stays Info.

### IN-03: Integration tests ignored with empty bodies

**File:** `tests/documentary_integration_test.rs:167-203`
**Category:** test quality (deferred from iteration 2)

Four FFmpeg-dependent integration tests are marked `#[ignore]` and contain no test logic beyond the availability check.

**Reassessment:** No change. Placeholder tests for future implementation. Stays Info.

### IN-04: `threads` field validated but never passed to FFmpeg

**File:** `src/documentary/types.rs:24,74-75`
**Category:** dead code (deferred from iteration 2)

`DocumentaryRequest::threads` (default 4) is validated as non-zero in `validate()` but is never passed to any FFmpeg command. FFmpeg uses its default threading behavior instead.

**Reassessment:** No change. Not a correctness bug -- FFmpeg defaults work fine. Stays Info.

## Files Reviewed

| File | Lines | Findings |
|------|-------|----------|
| src/documentary/mod.rs | 14 | 0 |
| src/documentary/error.rs | 205 | 0 |
| src/documentary/types.rs | 106 | 0 |
| src/documentary/timestamp.rs | 249 | 0 |
| src/documentary/subtitle.rs | 243 | 0 |
| src/documentary/pipeline.rs | 563 | 0 (WR-01/WR-02 fixed) |
| src/documentary/clip.rs | 243 | 0 |
| src/documentary/audio.rs | 273 | 0 (IN-01 deferred) |
| src/documentary/script_gen.rs | 522 | 0 |
| src/lib.rs | 30 | 0 |
| tests/documentary_integration_test.rs | 242 | 0 (IN-03 deferred) |
| tests/common/mod.rs | 13 | 0 |

## Conclusion

All actionable findings from iterations 1 and 2 have been fixed. The remaining 4 info-level items are design decisions that do not affect correctness, security, or data integrity. No new issues were introduced by the fix commit 8177521. The codebase is ready for the next development phase.

---

_Reviewed: 2026-05-05_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 3_
