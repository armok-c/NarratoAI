---
status: issues-found
phase: 06-documentary-pipeline
depth: standard
files_reviewed: 12
date: 2026-05-05
findings:
  critical: 2
  warning: 4
  info: 5
  total: 11
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
---

# Code Review: Phase 06 -- Documentary Pipeline

## Summary

Reviewed 12 Rust source files implementing the documentary video processing pipeline (6-step: load script, TTS, clip, merge audio/subtitle, concat, composite). The code is well-structured with clear module separation, proper error handling via `thiserror`, and good test coverage for timestamp parsing and subtitle generation. However, two critical issues were found: (1) the BGM `aloop` filter combined with `amix=duration=longest` will produce excessively long audio output for short videos, and (2) the FFmpeg `filter_complex` subtitle path escaping is incomplete, allowing `filter_complex` injection via file paths containing semicolons. Additional warnings include unused configuration fields, missing file path validation, silent error swallowing in directory reads, and unused `SubtitleSegment::offset_secs` semantics.

## Critical Findings

### CR-01: BGM aloop + amix=duration=longest causes excessively long output audio

**File:** `src/documentary/pipeline.rs:309-311`
**Category:** bug
**Description:** The composite step applies `aloop=loop=-1:size=2e+09` to the BGM input, then uses `amix=inputs=N:duration=longest` to mix audio streams. The `aloop` filter will loop the BGM for up to ~2e+09 samples (~12.6 hours at 44100 Hz). Because `amix` uses `duration=longest`, the output audio duration will be the longest of all inputs -- which is the looped BGM, not the video. For a 60-second video with BGM, the output will have ~12.6 hours of audio, making the final video unacceptably long.

**Impact:** Every video with BGM enabled will produce an output whose audio track is potentially hours longer than the video, resulting in massive file sizes and unplayable content.

**Fix:** Add an `atrim` filter to truncate the BGM to `total_duration` before the loop and fade, or change `amix` to use `duration=first` (but ensure the first input is TTS/silence base). The most robust fix:

```rust
// Instead of aloop + volume + afade, add atrim to cap BGM duration:
filter_complex_parts.push(format!(
    "[{}:a]atrim=0:{:.3},asetpts=PTS-STARTPTS,aloop=loop=-1:size=2e+09,volume={:.2},afade=t=out:st={:.1}:d=3[bgm]",
    bgm_idx, total_dur, bgm_vol, fade_start
));
```

Alternatively, switch to `amix=inputs=N:duration=first` and ensure the first input (anullsrc or TTS) defines the output length, or use `amix=inputs=N:duration=shortest` with a silent base of `total_duration`.

### CR-02: SRT path not escaped for FFmpeg filter_complex semicolon separator

**File:** `src/documentary/pipeline.rs:335-340`
**Category:** security
**Description:** The SRT file path is embedded into an FFmpeg `filter_complex` argument string. The path is escaped for `\`, `:`, `'`, `[`, `]` but NOT for `;` (semicolon), which is the `filter_complex` chain separator. If the SRT file path contains a semicolon (valid on Unix filesystems, e.g. `/tmp/my;file.srt`), the `filter_complex` string will be split at the semicolon, causing FFmpeg to interpret the remainder as a separate filter chain. This can lead to:
- FFmpeg parsing errors (denial of service on the pipeline)
- Potential filter injection if the path is attacker-controlled

**Impact:** Pipeline failure or FFmpeg filter injection when output directory or SRT file path contains semicolons. On Windows this is less likely (semicolons are invalid in filenames), but the task directory is user-configurable via `output_dir`.

**Fix:** Escape semicolons in the path (and optionally `\n` and `\r`):

```rust
let escaped_srt = srt_str
    .replace('\\', "/")
    .replace(':', "\\:")
    .replace("'", "\\'")
    .replace('[', "\\[")
    .replace(']', "\\]")
    .replace(';', "\\;")
    .replace('\n', "")
    .replace('\r', "");
```

## Warning Findings

### WR-01: DocumentaryRequest.validate() does not check video_path and script_path

**File:** `src/documentary/types.rs:54-74`
**Category:** logic error
**Description:** The `validate()` method checks numeric ranges for `voice_rate`, `voice_pitch`, `tts_volume`, `original_volume`, `bgm_volume`, and `threads`, but does not verify that `video_path` and `script_path` are non-empty `PathBuf`s. Both fields default to `PathBuf::new()` (empty). If a user constructs `DocumentaryRequest::default()` without setting these, `run_documentary` will proceed to `step_load_script` which will fail with a generic `std::io::Error` ("cannot find file") rather than a clear validation error.

**Impact:** Poor error messages for missing required paths; the pipeline progresses past validation before failing with an unhelpful IO error.

**Fix:**
```rust
pub fn validate(&self) -> Result<(), String> {
    if self.video_path.as_os_str().is_empty() {
        return Err("video_path 不能为空".to_string());
    }
    if self.script_path.as_os_str().is_empty() {
        return Err("script_path 不能为空".to_string());
    }
    // ... existing checks
}
```

### WR-02: original_volume field defined, validated, but never used in pipeline

**File:** `src/documentary/types.rs:15` and `src/documentary/pipeline.rs:263-418`
**Category:** dead code / logic error
**Description:** `DocumentaryRequest::original_volume` (default 0.7) is defined and validated in `validate()`, but is never referenced in `step_composite` or anywhere in the pipeline. When clips have `OstType::Mixed` (mixed mode), the original video audio is preserved during clipping, but the composite step does not apply any volume adjustment to it. Users who set `original_volume` expect the original audio volume to be adjusted in the final composite, but it is silently ignored.

**Impact:** User-configurable original audio volume setting has no effect on output.

**Fix:** In `step_composite`, add a volume filter for the original audio stream (stream 0:a) before mixing:

```rust
// Map original audio with volume adjustment
filter_complex_parts.push(format!("[0:a]volume={:.2}[orig]", original_volume));
```

Then include `[orig]` in the `amix` inputs.

### WR-03: subtitle_font, subtitle_color, subtitle_position, video_aspect fields defined but never used

**File:** `src/documentary/types.rs:19-23` and `src/documentary/pipeline.rs:341-343`
**Category:** dead code
**Description:** `DocumentaryRequest` defines `subtitle_font`, `subtitle_color`, `subtitle_position`, and `video_aspect` fields with defaults, but none are used in the pipeline. The composite step's `force_style` only includes `FontSize={}`. Users who configure subtitle color, position, or font will see no effect.

**Impact:** User-configurable subtitle appearance settings are silently ignored.

**Fix:** Include these fields in the `force_style` parameter:

```rust
let force_style = format!(
    "FontSize={},PrimaryColour={},BackColour={},Alignment={}",
    font_size,
    request.subtitle_color,
    "&H00000000",  // transparent background
    match request.subtitle_position.as_str() {
        "top" => "8",
        "center" => "5",
        _ => "2",  // bottom
    }
);
filter_complex_parts.push(format!(
    "[0:v]subtitles='{}':force_style='{}'[vout]",
    escaped_srt, force_style
));
```

### WR-04: collect_keyframe_paths silently swallows directory read errors

**File:** `src/documentary/script_gen.rs:386-387`
**Category:** error handling
**Description:** `collect_keyframe_paths` uses `filter_map(|entry| entry.ok())` which silently discards any `io::Error` from `read_dir`. If the keyframe directory has a permission issue, corrupted filesystem entry, or was deleted between creation and listing, the function returns a partial (or empty) list of keyframes without any indication that something went wrong.

**Impact:** Silent data loss -- frames may be missing from analysis without any warning or error, leading to incomplete LLM analysis results.

**Fix:**
```rust
fn collect_keyframe_paths(dir: &std::path::Path) -> Result<Vec<PathBuf>, PipelineError> {
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry.map_err(|e| PipelineError::FrameExtraction {
            details: format!("读取关键帧目录失败: {}", e),
        })?;
        if entry.path().extension()
            .is_some_and(|ext| ext == "jpg" || ext == "jpeg" || ext == "png")
            && entry.file_name().to_str()
                .is_some_and(|name| name.starts_with("keyframe_"))
        {
            paths.push(entry.path());
        }
    }
    paths.sort();
    Ok(paths)
}
```

## Info Findings

### IN-01: SubtitleSegment.offset_secs is always 0.0 -- confusing dual-offset pattern

**File:** `src/documentary/audio.rs:153-156`
**Category:** code clarity
**Description:** In `merge_subtitle_files`, `SubtitleSegment::offset_secs` is always set to `0.0`. The actual time offset is applied when generating SRT content via `generate_srt_from_word_boundaries(&tts.word_boundaries, cumulative_offset)`. This means `SubtitleSegment.offset_secs` is never used for its intended purpose (applying offsets during merge). The `merge_srt_files` function adds this offset via `apply_offset_to_block`, but since it is always 0.0, the offset logic in `merge_srt_files` is dead code for this call path.

**Fix:** Either apply the offset in `merge_srt_files` by setting `offset_secs` correctly (and passing `0.0` to `generate_srt_from_word_boundaries`), or remove the `offset_secs` field from `SubtitleSegment` if it is never needed. The current dual-offset approach is confusing and error-prone.

### IN-02: PipelineState.progress always None -- progress callbacks never fire

**File:** `src/documentary/pipeline.rs:447`
**Category:** dead code
**Description:** `PipelineState.progress` is hardcoded to `None` in `run_documentary`. The `emit_progress` method and all progress callback invocations throughout the 6 pipeline steps are effectively dead code. Users cannot receive progress updates because `DocumentaryRequest` has no `progress` field and `run_documentary` does not accept a progress callback parameter.

**Fix:** Add a `progress: Option<ProgressCallback>` field to `DocumentaryRequest` and propagate it to `PipelineState`.

### IN-03: Integration tests are all ignored -- no pipeline-level test coverage

**File:** `tests/documentary_integration_test.rs:168-203`
**Category:** test quality
**Description:** Four integration tests (`test_pipeline_mixed_ost_types`, `test_pipeline_ost0_narration_only`, `test_pipeline_ost1_original_sound`, `test_pipeline_ost2_mixed`) are all marked with `#[ignore = "需要 FFmpeg"]` and contain empty bodies. The only non-ignored tests are unit tests for timestamp parsing, SRT generation, progress callbacks, and request defaults. There is zero end-to-end pipeline test coverage.

**Fix:** At minimum, create a synthetic video using FFmpeg (or embed a small test video as bytes) and test the pipeline with a simple 2-clip script. Even if this test runs only in CI with FFmpeg available, it would catch issues like CR-01.

### IN-04: secs_to_srt_time/secs_to_ffmpeg_time produce invalid output for extreme f64 values

**File:** `src/documentary/timestamp.rs:86,101`
**Category:** edge case
**Description:** If `secs` is `f64::INFINITY`, `(secs * 1000.0).round() as u64` saturates to `u64::MAX`, producing a time string like `"5133986312586145:20:15,615"`. While unlikely in practice (video durations are finite), the functions should clamp to a reasonable maximum (e.g., 86400 seconds = 24 hours) for robustness.

**Fix:** Add a clamp before conversion:
```rust
let secs = secs.min(86399.999);
let total_ms = (secs * 1000.0).round() as u64;
```

### IN-05: FFmpeg FFmpegEvent::Progress variant silently ignored in clip.rs

**File:** `src/documentary/clip.rs:176-184`
**Category:** minor
**Description:** The `run_clip_ffmpeg` event loop matches only `FfmpegEvent::Error` and uses `_ => {}` for all other events. `FfmpegEvent::Progress` events (which contain `time`, `speed`, `frame` information) are discarded. The pipeline.rs equivalent also discards progress events. While not a bug, logging progress events at debug/trace level would aid debugging of slow clips.

**Fix:**
```rust
match event {
    ffmpeg_sidecar::event::FfmpegEvent::Error(e) => {
        tracing::error!("FFmpeg clip error: {}", e);
        had_errors = true;
    }
    ffmpeg_sidecar::event::FfmpegEvent::Progress(p) => {
        tracing::trace!("FFmpeg clip progress: {:?}", p);
    }
    _ => {}
}
```

## Files Reviewed

| File | Lines | Findings |
|------|-------|----------|
| src/documentary/mod.rs | 14 | 0 CR, 0 WR, 0 IN |
| src/documentary/error.rs | 205 | 0 CR, 0 WR, 0 IN |
| src/documentary/types.rs | 100 | 0 CR, 1 WR (WR-01), 1 IN (IN-02) |
| src/documentary/timestamp.rs | 247 | 0 CR, 0 WR, 1 IN (IN-04) |
| src/documentary/subtitle.rs | 243 | 0 CR, 0 WR, 1 IN (IN-01) |
| src/documentary/pipeline.rs | 542 | 2 CR (CR-01, CR-02), 2 WR (WR-02, WR-03), 0 IN |
| src/documentary/clip.rs | 240 | 0 CR, 0 WR, 1 IN (IN-05) |
| src/documentary/audio.rs | 273 | 0 CR, 0 WR, 0 IN |
| src/documentary/script_gen.rs | 522 | 0 CR, 1 WR (WR-04), 0 IN |
| src/lib.rs | 30 | 0 CR, 0 WR, 0 IN |
| tests/documentary_integration_test.rs | 242 | 0 CR, 0 WR, 1 IN (IN-03) |
| tests/common/mod.rs | 13 | 0 CR, 0 WR, 0 IN |

## Recommendations

1. **Fix CR-01 immediately** -- Add `atrim=0:{total_dur}` to the BGM filter chain or switch `amix` to `duration=first` to prevent output audio from being orders of magnitude longer than the video.
2. **Fix CR-02 immediately** -- Escape `;` (and `\n`, `\r`) in the SRT path before embedding it into `filter_complex`.
3. **Address WR-01** -- Add `video_path` and `script_path` non-empty checks to `validate()`.
4. **Address WR-02/WR-03** -- Either implement the missing volume/font/color/position logic in `step_composite`, or remove these fields from `DocumentaryRequest` to avoid misleading users.
5. **Address WR-04** -- Stop silently swallowing `read_dir` errors in `collect_keyframe_paths`.
6. **Consider IN-03** -- Add at least one non-ignored integration test that exercises the pipeline with a synthetic video to catch regressions like CR-01 in the future.

---

_Reviewed: 2026-05-05_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
