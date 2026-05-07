---
phase: 06-documentary-pipeline
reviewed: 2026-05-07T16:00:00Z
depth: standard
files_reviewed: 12
files_reviewed_list:
  - src/documentary/audio.rs
  - src/documentary/clip.rs
  - src/documentary/error.rs
  - src/documentary/mod.rs
  - src/documentary/pipeline.rs
  - src/documentary/script_gen.rs
  - src/documentary/subtitle.rs
  - src/documentary/timestamp.rs
  - src/documentary/types.rs
  - src/lib.rs
  - tests/common/mod.rs
  - tests/documentary_integration_test.rs
findings:
  critical: 0
  warning: 2
  info: 2
  total: 4
status: issues_found
previous_review:
  date: 2026-05-07
  iteration: 5
  findings: 8 (0 CR + 4 WR + 4 IN)
  fixes_applied: 4 fixed, 0 skipped (all warnings resolved)
---

# Phase 06: Code Review Report (Iteration 6)

**Reviewed:** 2026-05-07T16:00:00Z
**Depth:** standard
**Files Reviewed:** 12
**Status:** issues_found

## Summary

Iteration 6 review of the documentary pipeline module. The previous iteration (5) found 0 critical, 4 warnings, and 4 info items. Fixes applied in commits `a66839e` and `3b03da1`:

- WR-01 (temp directory cleanup): `CleanupOnDrop` RAII guard confirmed at `script_gen.rs:31-51`, with `cancel()` on success at line 148.
- WR-02 (negative timestamps): Warning log added at `subtitle.rs:25-30`, confirmed working.
- WR-03 (SubtitleSegment offset): Offset now passed at `0.0` to `generate_srt_from_word_boundaries` and `cumulative_offset` into `SubtitleSegment.offset_secs` at `audio.rs:157-162`.
- WR-04 (file existence checks): Both `video_path.exists()` and `script_path.exists()` confirmed at `types.rs:56-64`.

All 4 prior warnings verified as fixed. This iteration found **0 critical**, **2 warnings**, and **2 info items** -- all new findings from re-analysis. The codebase continues to demonstrate solid architecture: clean error hierarchy, proper RAII patterns, comprehensive validation, and well-structured FFmpeg command construction.

## Warnings

### WR-01: Negative timestamp bypass in `generate_srt_from_word_boundaries` -- warning logged but block still emitted

**File:** `src/documentary/subtitle.rs:25-39`
**Issue:** The iteration-5 fix added a `tracing::warn!` for negative timestamps, which partially addresses the prior WR-02. However, after logging the warning, the function still emits the SRT block unconditionally (lines 31-39). Downstream `secs_to_srt_time` clamps negative values to `"00:00:00,000"`, so output is not visually broken, but the SRT block will have incorrect timing -- subtitles appear at 00:00:00 instead of their intended position. This is inconsistent with `apply_offset_to_block` (lines 137-141) which returns an `Err` for the same condition. Two code paths handling the same logical error differently is a maintenance hazard: future changes to one path will likely miss the other.

**Fix:** Make handling consistent. Either skip the block or return an error, matching `apply_offset_to_block` behavior:

```rust
if start_secs < 0.0 || end_secs < 0.0 {
    tracing::warn!(
        "Word boundary #{} produces negative timestamp (start={:.3}, end={:.3}, offset={:.3}), skipping",
        i, start_secs, end_secs, offset_secs
    );
    continue; // skip -- consistent with apply_offset_to_block's error
}
```

### WR-02: FFmpeg-dependent integration tests remain empty stubs with no coverage

**File:** `tests/documentary_integration_test.rs:166-203`
**Issue:** This was reported as IN-01 in iteration 5 and remains unfixed. Four test functions (`test_pipeline_mixed_ost_types`, `test_pipeline_ost0_narration_only`, `test_pipeline_ost1_original_sound`, `test_pipeline_ost2_mixed`) are marked `#[ignore]` and contain only a `ffmpeg_available()` guard and a comment -- no assertions or pipeline logic. These stubs provide zero coverage for the OST-type-specific clipping paths (`clip_ost0_narration_only`, `clip_ost1_original_sound`, `clip_ost2_mixed`) and the main pipeline steps (`step_clip`, `step_concat`, `step_composite`). The unit tests in `clip.rs` and `pipeline.rs` cover timestamp parsing and duration calculation but not the FFmpeg command construction or the 6-step orchestration. Upgrading to WARNING because this was flagged previously and remains unaddressed.

**Fix:** Implement at least one test with a synthetic video fixture that exercises the pipeline. A minimal approach: use FFmpeg to create a 2-second test video in a temp directory, construct a simple script JSON, and run `clip_all_videos` for each OST type. Alternatively, remove the stubs entirely and document the coverage gap in the module-level doc comment.

## Info

### IN-01: `strip_and_repair_json` single-quote replacement can corrupt natural language content

**File:** `src/documentary/script_gen.rs:358-364`
**Issue:** The heuristic at line 359 ("replace all single quotes with double quotes when no double quotes are present") can corrupt legitimate text content in LLM responses. For example, narration text containing contractions like `It's a cat` would become `It"s a cat`, breaking JSON parsing. In practice this branch is only reached after all other repair attempts fail, and the result is validated before return, so the risk is low. However, for content-rich JSON (which LLM script output is), this is a fragile heuristic.

**Fix:** Acceptable as a last-resort heuristic. Consider adding a `tracing::debug!` when this branch is reached to monitor frequency. If it fires often, a smarter approach would be to only replace single quotes that appear in key/value delimiter positions (after `:` or around keys), not inside string values.

### IN-02: `collect_keyframe_paths` does not recursively search subdirectories

**File:** `src/documentary/script_gen.rs:413-429`
**Issue:** `collect_keyframe_paths` uses `std::fs::read_dir` which is non-recursive. If `extract_frames` ever changes to organize frames into subdirectories (e.g., by batch), this function would silently miss them. Currently `extract_frames` writes all frames flat into the output directory, so this is not a bug. The function also sorts by `PathBuf` default `Ord` which is lexicographic -- correct for zero-padded frame numbers like `keyframe_0001.jpg` but would give wrong order for unpadded names like `keyframe_1.jpg`, `keyframe_10.jpg`, `keyframe_2.jpg`.

**Fix:** Not a current bug. If frame naming convention changes, the sort order may need natural sorting. Document the assumption that keyframe filenames are zero-padded.

---

_Reviewed: 2026-05-07T16:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 6_
