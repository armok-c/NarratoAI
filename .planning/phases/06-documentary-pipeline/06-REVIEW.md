---
phase: 06-documentary-pipeline
reviewed: 2026-05-07T14:45:00Z
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
  warning: 4
  info: 4
  total: 8
status: issues_found
previous_review:
  date: 2026-05-07
  iteration: 4
  findings: 6 (1 CR + 5 WR)
  status: issues_found
  fixes_applied: 5 fixed, 1 skipped (WR-05)
---

# Phase 06: Code Review Report (Iteration 5)

**Reviewed:** 2026-05-07T14:45:00Z
**Depth:** standard
**Files Reviewed:** 12
**Status:** issues_found

## Summary

Adversarial re-review after iteration 4 fixes (5 of 6 applied; WR-05 sequential processing intentionally skipped). All 5 fixes verified in source:

- CR-01 (audio delay truncation): `.round()` confirmed at `audio.rs:54`
- WR-01 (test data values): `5_000_000`, `12_000_000`, `20_000_000` confirmed at `subtitle.rs:154-164`
- WR-02 (font_size bounds): validation `[1, 200]` confirmed at `types.rs:77-82`
- WR-03 (missing TTS error): `ok_or_else` confirmed at `audio.rs:46`
- WR-04 (unused param doc): comment confirmed at `pipeline.rs:150-152`

This pass found **0 critical**, **4 warnings**, and **4 info items** -- all new findings from deeper analysis. The codebase has solid error propagation, proper FFmpeg argument construction (no shell injection), path escaping for subtitle filters, and font name sanitization. No security vulnerabilities were identified.

## Warnings

### WR-01: Temporary keyframe directory never cleaned up on failure

**File:** `src/documentary/script_gen.rs:38-42`
**Issue:** `analyze_video` creates a `keyframes_cache/{uuid}/` directory inside the video's parent directory. If any subsequent step fails (frame extraction, LLM analysis, JSON serialization), this directory and its contents are permanently orphaned on disk. There is no cleanup on any error path. Over time, repeated failures accumulate orphaned directories consuming disk space in the user's video directory.
**Fix:** Use a cleanup guard or `tempfile::TempDir` for automatic cleanup on failure.

```rust
// Option A: tempfile::TempDir (removed on drop unless persisted)
let output_dir = tempfile::tempdir_in(
    request.video_path.parent().unwrap_or(Path::new("."))
)?;
let output_path = output_dir.path();

// Option B: Scope guard with manual cleanup
struct CleanupOnDrop(PathBuf);
impl Drop for CleanupOnDrop {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
let _guard = CleanupOnDrop(output_dir.clone());
```

### WR-02: Negative timestamps silently clamped to zero in SRT generation

**File:** `src/documentary/subtitle.rs:22-24` and `src/documentary/timestamp.rs:83-85`
**Issue:** If `offset_secs` is negative or `WordBoundary` offsets produce a negative total, `secs_to_srt_time` silently clamps to `"00:00:00,000"` without any warning or error. This hides calculation bugs in the caller, potentially producing SRT files with overlapping or zero-length subtitle blocks that are difficult to diagnose.
**Fix:** Add validation in `generate_srt_from_word_boundaries` to detect and warn on negative timestamps.

```rust
pub fn generate_srt_from_word_boundaries(
    word_boundaries: &[WordBoundary],
    offset_secs: f64,
) -> String {
    let mut blocks = Vec::new();
    for (i, wb) in word_boundaries.iter().enumerate() {
        let start_secs = offset_secs + wb.start_offset as f64 / 10_000_000.0;
        let end_secs = offset_secs + wb.end_offset as f64 / 10_000_000.0;
        if start_secs < 0.0 || end_secs < 0.0 {
            tracing::warn!(
                "Word boundary #{} produces negative timestamp (start={:.3}, end={:.3}, offset={:.3})",
                i, start_secs, end_secs, offset_secs
            );
        }
        // ... rest unchanged
    }
}
```

### WR-03: SubtitleSegment.offset_secs always 0.0 -- misleading data flow

**File:** `src/documentary/audio.rs:155-162`
**Issue:** In `merge_subtitle_files`, the cumulative offset is passed directly into `generate_srt_from_word_boundaries`, which bakes the offset into the SRT content. Then `SubtitleSegment { offset_secs: 0.0 }` is created, and `merge_srt_files` applies `0.0` offset (a no-op). The `SubtitleSegment.offset_secs` field suggests it carries the offset, but it is always zero in practice. This design is confusing for maintainers who might change one offset application path without realizing the other exists, introducing double-offset or zero-offset bugs.
**Fix:** Apply the offset in exactly one place for clarity.

```rust
// Move offset to SubtitleSegment, generate SRT without offset:
let srt_content = crate::documentary::subtitle::generate_srt_from_word_boundaries(
    &tts.word_boundaries,
    0.0,  // no offset baked in here
);
segments.push(SubtitleSegment {
    srt_content,
    offset_secs: cumulative_offset,  // offset applied during merge
});
```

### WR-04: No file-existence check for video_path and script_path

**File:** `src/documentary/types.rs:52-58`
**Issue:** `DocumentaryRequest::validate()` checks that `video_path` and `script_path` are non-empty strings but does not verify the files actually exist on disk. This defers "file not found" errors to deep pipeline steps (step 1 for script at `pipeline.rs:37`, step 3 for video via FFmpeg) where the error message is less specific and harder to trace back to user input.
**Fix:** Add existence checks in validation for immediate, clear error reporting.

```rust
pub fn validate(&self) -> Result<(), String> {
    if self.video_path.as_os_str().is_empty() {
        return Err("video_path 不能为空".to_string());
    }
    if !self.video_path.exists() {
        return Err(format!("video_path 文件不存在: {}", self.video_path.display()));
    }
    if self.script_path.as_os_str().is_empty() {
        return Err("script_path 不能为空".to_string());
    }
    if !self.script_path.exists() {
        return Err(format!("script_path 文件不存在: {}", self.script_path.display()));
    }
    // ... rest unchanged
}
```

## Info

### IN-01: FFmpeg-dependent integration tests are empty stubs

**File:** `tests/documentary_integration_test.rs:166-203`
**Issue:** All four FFmpeg-dependent integration tests (`test_pipeline_mixed_ost_types`, `test_pipeline_ost0_narration_only`, `test_pipeline_ost1_original_sound`, `test_pipeline_ost2_mixed`) are marked `#[ignore]` and contain only a guard check with no actual test logic or assertions. These provide no coverage for the core 6-step pipeline flow.
**Fix:** Implement these tests or remove them to avoid false confidence in test counts.

### IN-02: Test data uses non-standard timestamp format

**File:** `src/documentary/script_gen.rs:488`
**Issue:** Test data uses timestamps like `"00:00-00:05"` (MM:SS format) which would fail `parse_time_to_secs()` if processed through the pipeline. The tests only validate JSON parsing, not timestamp validity, so this is not a bug -- but it could mislead future test authors into thinking this timestamp format is supported in production.
**Fix:** Use the expected `HH:MM:SS-HH:MM:SS` format in test data for consistency.

### IN-03: Dead-code fallback on infallible `split().next()`

**File:** `src/documentary/clip.rs:22` and `src/documentary/clip.rs:77`
**Issue:** `clip.timestamp.split('-').next().unwrap_or(&clip.timestamp)` -- `split().next()` always returns `Some(...)` because `split` yields at least one element. The `unwrap_or` fallback is dead code. Not a bug, but adds noise.
**Fix:** Simplify to `.unwrap()` or use `let start_part = clip.timestamp.split('-').next().unwrap();`.

### IN-04: PipelineState.progress always None -- progress infrastructure unused

**File:** `src/documentary/pipeline.rs:500`
**Issue:** `PipelineState.progress` is hardcoded to `None` in `run_documentary`. The `emit_progress` method and all `ProgressStep`-based callback invocations are dead code until the pipeline is integrated with a caller that injects a progress callback. The `ProgressCallback` type and `ProgressStep` enum are well-designed and tested, but unused in the current codebase.
**Fix:** Accept as designed-for-future-use, or expose a builder method to inject the callback.

---

_Reviewed: 2026-05-07T14:45:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 5_
