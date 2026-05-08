---
phase: 08-sdp-pipeline
reviewed: 2026-05-08T21:00:00Z
depth: standard
files_reviewed: 20
files_reviewed_list:
  - narratoai-core/src/lib.rs
  - narratoai-core/src/prompt/register.rs
  - narratoai-core/src/prompt/templates/short_drama_editing/plot_extraction_v2.0.md
  - narratoai-core/src/prompt/templates/short_drama_editing/subtitle_analysis_v2.0.md
  - narratoai-core/src/sde/error.rs
  - narratoai-core/src/sde/mod.rs
  - narratoai-core/src/sde/pipeline.rs
  - narratoai-core/src/sde/timestamp.rs
  - narratoai-core/src/sde/types.rs
  - narratoai-core/src/sdp/clip.rs
  - narratoai-core/src/sdp/error.rs
  - narratoai-core/src/sdp/mod.rs
  - narratoai-core/src/sdp/pipeline.rs
  - narratoai-core/src/sdp/script_gen.rs
  - narratoai-core/src/sdp/types.rs
  - narratoai-core/src/subtitle/error.rs
  - narratoai-core/src/subtitle/mod.rs
  - narratoai-core/src/subtitle/parser.rs
  - narratoai-core/src/subtitle/timestamp.rs
  - narratoai-core/src/subtitle/types.rs
findings:
  critical: 1
  warning: 2
  info: 0
  total: 3
status: issues_found
---

# Phase 08: SDP Pipeline Code Review Report (Re-review 4, Iteration 5)

**Reviewed:** 2026-05-08T21:00:00Z
**Depth:** standard
**Files Reviewed:** 20
**Status:** issues_found

## Summary

This is the FIFTH review pass (iteration 5) of Phase 08. Previous reviews found and fixed CR-01, CR-02, WR-01 through WR-09, and the last re-review (iteration 4) found and fixed WR-10 (script_path existence check) and WR-11 (load_script in spawn_blocking). Both are confirmed fixed in the current code.

This pass found 1 new critical issue and 2 new warnings. The critical issue is a logic bug in SDE pipeline where `source_time_range` is computed incorrectly for OST=0/OST=2 clips by using TTS duration instead of the actual source video timestamp range.

## Previously Confirmed Fixed Issues

All previously reported issues remain correctly fixed:

- **CR-01** (subtitle_segments never populated): Fixed in `sdp/pipeline.rs:52-62`
- **CR-02** (step 2 sends plain text to LLM): Fixed in `sdp/script_gen.rs:234`
- **WR-01 through WR-09**: All remain correctly fixed
- **WR-10** (script_path existence check): Fixed in `sdp/types.rs:65-69`
- **WR-11** (load_script blocking call): Fixed in `sdp/pipeline.rs:44-50`

## Critical Issues

### CR-03: SDE pipeline computes `source_time_range` incorrectly for OST=0/OST=2 clips using TTS duration instead of source timestamp end

**File:** `narratoai-core/src/sde/pipeline.rs:245-255`
**Issue:** In the SDE clip step (step 5), after calculating `clip_duration` via `calculate_clip_duration()`, the code computes `end_secs = start_secs + clip_duration` (line 249) and uses this to set `source_time_range`. However, `clip_duration` for OST=0 (NarrationOnly) and OST=2 (Mixed) clips comes from TTS duration, not from the source video timestamp. This means `source_time_range` records the *narration* duration starting from the source start time, not the actual source video segment end time.

For OST=0 clips: the video clip is cut to TTS duration (not the original timestamp range), so `source_time_range` is misleading -- it shows `timestamp_start to (timestamp_start + tts_duration)` rather than the original `timestamp_start to timestamp_end`.

For OST=1 (OriginalSound) clips: `clip_duration` happens to be computed from the timestamp (since no TTS is generated), so this is accidentally correct. But the same code path is shared for all OST types.

The `source_time_range` field semantically should represent the source video time range (from the script timestamp), while `edited_time_range` represents the output timeline. Mixing TTS duration into `source_time_range` conflates these two concepts.

Compare with `sdp/clip.rs:60-90` which correctly parses both `start_secs` and `end_secs` from the timestamp and computes duration from those values.

**Fix:** Parse `end_secs` from the timestamp directly for `source_time_range`:

```rust
let ts_parts: Vec<&str> = clip.timestamp.splitn(2, '-').collect();
if ts_parts.len() == 2 {
    let start_secs = parse_time_to_secs(ts_parts[0])
        .map_err(|e| SdeError::VideoProcess(PipelineError::Timestamp(e.to_string())))?;
    let source_end_secs = parse_time_to_secs(ts_parts[1])
        .map_err(|e| SdeError::VideoProcess(PipelineError::Timestamp(e.to_string())))?;
    
    clip.source_time_range = Some(format!(
        "{}-{}",
        secs_to_ffmpeg_time(start_secs),
        secs_to_ffmpeg_time(source_end_secs)
    ));
    clip.duration = Some(clip_duration);
    clip.edited_time_range = Some(format!(
        "{}-{}",
        secs_to_ffmpeg_time(cumulative_time),
        secs_to_ffmpeg_time(cumulative_time + clip_duration)
    ));
    cumulative_time += clip_duration;
} else {
    return Err(SdeError::Validation {
        details: format!("片段 {} 时间戳格式无效（缺少 '-' 分隔符）: {}", clip._id, clip.timestamp),
    });
}
```

## Warnings

### WR-12: `SdpRequest::validate` accepts non-existent file paths in tests because it checks `PathBuf::from("sub.srt")` without requiring file existence

**File:** `narratoai-core/src/sdp/types.rs:151-158`
**Issue:** The test `test_sdp_request_validate_valid` creates an `SdpRequest` with `subtitle_path: PathBuf::from("sub.srt")` and `video_path: PathBuf::from("video.mp4")`. The `validate()` function at lines 59-63 checks `self.subtitle_path.exists()` and `self.video_path.exists()`, which should fail for these relative paths (since the files don't actually exist in the working directory). Yet the test asserts `req.validate().is_ok()`.

This test will pass only if there happen to be files named `sub.srt` and `video.mp4` in the current working directory at test time, which makes the test brittle and environment-dependent. This is a test reliability bug.

Compare with the SDE equivalent test at `sde/types.rs:183-194` which correctly uses `tempfile::TempDir` to create real files.

**Fix:** Use `tempfile::TempDir` and create actual files, matching the SDE test pattern:

```rust
#[test]
fn test_sdp_request_validate_valid() {
    let dir = tempfile::TempDir::new().unwrap();
    let sub = dir.path().join("sub.srt");
    let vid = dir.path().join("video.mp4");
    std::fs::write(&sub, "").unwrap();
    std::fs::write(&vid, "").unwrap();
    let req = SdpRequest {
        subtitle_path: sub,
        video_path: vid,
        ..Default::default()
    };
    assert!(req.validate().is_ok());
}
```

### WR-13: `parse_srt_timestamp` normalizes ALL dots to commas including within SS field, causing ambiguous parsing

**File:** `narratoai-core/src/subtitle/timestamp.rs:11-15`
**Issue:** The function replaces ALL dots with commas at line 12 (`input.replace('.', ",")`). This means a timestamp like `00.01.02,500` (dots instead of colons, which is invalid but conceivable as malformed input) would become `00,01,02,500` and potentially parse incorrectly. More practically, the timestamp `1.5` (a bare float seconds value) would become `1,5` and be parsed as `HH:MM:SS` with `1` as hours and `5` as minutes. While SRT timestamps use dots as millisecond separators, the blanket replacement could mask malformed input instead of rejecting it.

The current SRT/ASS formats always use `HH:MM:SS.mmm` where the dot is only in the millis separator position. The normalization should be more targeted -- only replacing dots in the expected millis position (after the SS field) rather than globally.

**Fix:** Use a more targeted normalization that only replaces the dot between seconds and milliseconds:

```rust
// Only normalize dot in the expected millis position (after the last colon-separated group)
let normalized = if let Some(pos) = input.rfind('.') {
    // Check if the dot is after the last colon (i.e., it's a millis separator)
    if input.rfind(':').map_or(false, |cp| cp < pos) {
        format!("{}{}", &input[..pos], &input[pos..].replace('.', ","))
    } else {
        input.to_string()
    }
} else {
    input.to_string()
};
```

---

_Reviewed: 2026-05-08T21:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
