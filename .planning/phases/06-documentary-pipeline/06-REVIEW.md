---
phase: 06-documentary-pipeline
reviewed: 2026-05-07T12:00:00Z
depth: standard
files_reviewed: 12
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
findings:
  critical: 1
  warning: 5
  info: 3
  total: 9
status: issues_found
previous_review:
  date: 2026-05-05
  iteration: 3
  findings: 4 (0 CR + 0 WR + 4 IN)
  status: clean
---

# Phase 06: Code Review Report (Iteration 4)

**Reviewed:** 2026-05-07T12:00:00Z
**Depth:** standard
**Files Reviewed:** 12
**Status:** issues_found

## Summary

Adversarial re-review of all 12 documentary pipeline source files. Previous iterations (1-3) found and fixed 2 critical, 4 warnings, and 4 info items. The code was declared "clean" in iteration 3.

This adversarial pass surfaced 1 new critical bug (integer truncation in audio delay calculation causing potential audio-video desync), 5 warnings (misleading test data, missing validation bounds, silent skip of missing TTS results, unused parameter, sequential clip processing), and 3 info items. The code has solid error propagation and Chinese Display messages throughout. Security is adequate: FFmpeg arguments are built programmatically (no shell injection), subtitle paths are escaped, font names are sanitized, and color values are validated as hex.

## Critical Issues

### CR-01: Integer truncation in audio delay causes cumulative audio-video desync

**File:** `src/documentary/audio.rs:47`
**Issue:** The delay calculation `(cumulative_offset * 1000.0) as u64` performs a truncating cast from `f64` to `u64`, silently discarding fractional milliseconds. For a `cumulative_offset` of `5.9994` seconds, the result is `5999` ms instead of the correct `5999.4` ms. Over multiple clips, this truncation accumulates and causes progressive audio-video desynchronization. For example, with 10 clips each having a 0.5ms truncation error, the audio drifts by 5ms per clip, reaching 50ms total drift -- perceptible to users.

The `adelay` FFmpeg filter accepts integer milliseconds, so some rounding is unavoidable, but `round()` would minimize error (truncation always rounds down; rounding halves the average error).

**Fix:**
```rust
// Before (line 47):
let delay_ms = (cumulative_offset * 1000.0) as u64;

// After:
let delay_ms = (cumulative_offset * 1000.0).round() as u64;
```

## Warnings

### WR-01: Misleading underscore separators and incorrect comments in subtitle test data

**File:** `src/documentary/subtitle.rs:153-165`
**Issue:** Test helper `make_word_boundaries()` uses numeric literals `500_000_0`, `1_200_000_0`, and `2_000_000_0` with comments claiming "0.5s", "1.2s", and "2.0s" respectively. In Rust, underscores in numeric literals are visual separators only, so these equal `50000000`, `120000000`, and `200000000` in 100ns units. These represent 5.0s, 12.0s, and 20.0s -- not the commented durations. The tests pass only because they verify offset application (SRT contains `"00:00:10,000"` which is the offset, not the boundary duration), not absolute boundary timing. A developer relying on these comments would write incorrect code.

**Fix:** Either fix the values to match comments or fix the comments to match the values:
```rust
// Option A: Fix values to match comments (0.5s, 1.2s, 2.0s)
end_offset: 5_000_000,    // 0.5s in 100ns units (5,000,000 * 100ns = 0.5s)
start_offset: 5_000_000,
end_offset: 12_000_000,   // 1.2s in 100ns units
start_offset: 12_000_000,
end_offset: 20_000_000,   // 2.0s in 100ns units
```

### WR-02: `subtitle_font_size` has no upper or lower bound validation

**File:** `src/documentary/types.rs:74-75`
**Issue:** `validate()` checks numeric ranges for `voice_rate`, `voice_pitch`, `tts_volume`, `original_volume`, `bgm_volume`, and `threads`, but not `subtitle_font_size` (a `u32`). A value of `u32::MAX` passes validation and gets embedded in the FFmpeg `force_style` string as `FontSize=4294967295`, which would cause FFmpeg to produce garbled output or crash. A value of `0` produces invisible subtitles.

**Fix:**
```rust
// Add to validate():
if self.subtitle_font_size == 0 || self.subtitle_font_size > 200 {
    return Err(format!(
        "subtitle_font_size 超出有效范围 [1, 200]: {}",
        self.subtitle_font_size
    ));
}
```

### WR-03: `merge_audio_files` silently skips OST=0/2 clips with missing TTS results

**File:** `src/documentary/audio.rs:46-58`
**Issue:** For OST=0 (NarrationOnly) or OST=2 (Mixed) clips, if `tts_results.get(&clip._id)` returns `None`, the clip's audio is silently skipped -- no TTS audio is added, but `cumulative_offset` is still incremented by `clip_duration`. This creates a silent gap where narration was expected. While `clip_all_videos` in `clip.rs` would fail first for missing TTS (it uses `ok_or_else`), `merge_audio_files` does not enforce this invariant itself. If called from a different code path, or if TTS results are somehow removed between steps, the silent gap would go undetected.

**Fix:**
```rust
// Replace lines 46-56 with:
if clip.ost != OstType::OriginalSound {
    let tts = tts_results.get(&clip._id).ok_or_else(|| {
        PipelineError::AudioMerge {
            details: format!(
                "OST={:?} 片段 {} 缺少 TTS 结果，无法合并音频",
                clip.ost, clip._id
            ),
        }
    })?;
    let delay_ms = (cumulative_offset * 1000.0).round() as u64;
    let input_idx = mix_inputs.len();
    input_args.push("-i".to_string());
    input_args.push(tts.audio_path.to_string_lossy().to_string());
    let label = format!("[{}:a]", input_idx);
    filter_parts.push(format!("{}adelay={}|{}[tts{}]", label, delay_ms, delay_ms, clip._id));
    mix_inputs.push(format!("[tts{}]", clip._id));
}
```

### WR-04: `step_merge_audio_subtitle` receives unused `_request` parameter

**File:** `src/documentary/pipeline.rs:152`
**Issue:** The `_request` parameter in `step_merge_audio_subtitle` is prefixed with underscore to suppress unused-variable warnings, but this function is `pub(crate)` and its signature is part of the module's API surface. An unused parameter in a public function is a code smell that indicates either incomplete implementation or API design drift.

**Fix:** Either remove the parameter (if truly not needed) or document why it is intentionally unused (if reserved for future use):
```rust
// Option A: Remove the parameter and update the call site at line 510.
// Option B: Add a doc comment:
/// 步骤 4: 合并音频和字幕
///
/// Note: `_request` is currently unused but retained for future
/// per-request audio volume control during merge.
```

### WR-05: `clip_all_videos` processes clips sequentially, missing parallelism opportunity

**File:** `src/documentary/clip.rs:89-124`
**Issue:** All clips are processed in a sequential `for` loop, each invoking FFmpeg via `await`. For a documentary with many clips (20-50 is common), this serializes all FFmpeg invocations. Since each clip is independent and FFmpeg primarily uses CPU/GPU, running 2-4 clip extractions concurrently could significantly reduce pipeline wall time. The `DocumentaryRequest::threads` field (validated as non-zero but currently unused per IN-04) could control this concurrency.

**Fix:** This is a quality improvement, not a correctness bug. No immediate action required, but worth tracking.

## Info

### IN-01: `SubtitleSegment.offset_secs` always 0.0 -- redundant field in current usage

**File:** `src/documentary/audio.rs:153-156`
**Issue:** In `merge_subtitle_files`, `SubtitleSegment::offset_secs` is always `0.0`. The actual time offset is applied upstream in `generate_srt_from_word_boundaries` via the `cumulative_offset` parameter. The `offset_secs` field and `apply_offset_to_block` in `merge_srt_files` are effectively dead code for this call path. The `SubtitleSegment` struct and `merge_srt_files` function appear to have been designed for a different usage pattern where multiple SRT files with different offsets are merged, but the current pipeline applies offsets before creating segments.

### IN-02: `parse_time_to_secs` accepts fractional hour/minute components

**File:** `src/documentary/timestamp.rs:41-59`
**Issue:** The function parses hour/minute/second components as `f64` and validates ranges `[0, 23]`, `[0, 59]`, `[0, 59]`. This means `"00.5:00:00"` would parse as `0.5 * 3600 = 1800.0` seconds (30 minutes), which is a valid but unusual interpretation. Standard timestamp formats always use integer hour/minute/second components. This looseness does not cause bugs in practice since timestamps come from scripts with standard formatting, but it could lead to surprising behavior with malformed input.

### IN-03: `PipelineState.progress` is always `None` -- progress callbacks never fire

**File:** `src/documentary/pipeline.rs:481`
**Issue:** `PipelineState.progress` is hardcoded to `None` in `run_documentary`. The `emit_progress` method and all `ProgressStep`-based callback invocations are dead code until the pipeline is integrated with a caller that injects a progress callback. The `ProgressCallback` type and `ProgressStep` enum are well-designed and tested (see `test_progress_callback_receives_all_steps`), but unused in the current codebase.

---

_Reviewed: 2026-05-07T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 4_
