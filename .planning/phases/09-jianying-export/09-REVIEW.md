---
phase: 09-jianying-export
reviewed: 2026-04-29T18:00:00Z
depth: standard
files_reviewed: 13
files_reviewed_list:
  - Cargo.toml
  - src/ffmpeg/probe.rs
  - src/jianying/builder.rs
  - src/jianying/error.rs
  - src/jianying/material.rs
  - src/jianying/mod.rs
  - src/jianying/segment.rs
  - src/jianying/template.rs
  - src/jianying/time.rs
  - src/jianying/track.rs
  - src/jianying/types.rs
  - src/lib.rs
  - tests/jianying_export.rs
findings:
  critical: 1
  warning: 4
  info: 3
  total: 8
status: issues_found
---

# Phase 09: Code Review Report

**Reviewed:** 2026-04-29T18:00:00Z
**Depth:** standard
**Files Reviewed:** 13
**Status:** issues_found

## Summary

Re-reviewed all 13 source files after two prior review fix rounds. Previous critical issues (CR-01 silent segment loss, CR-02 timestamp parser) have been properly resolved. The codebase now has proper error propagation, validation for negative durations, and numeric ffprobe duration handling. However, one new critical issue and several warnings were identified in this iteration. The code is well-structured with comprehensive test coverage (unit + integration), clear builder patterns, and good error types.

## Critical Issues

### CR-01: `trange_from_secs` allows negative `start` without validation

**File:** `src/jianying/time.rs:32-39`
**Issue:** `trange_from_secs` validates that `duration_us >= 0` (added in a prior fix round), but does **not** validate that `start_us >= 0`. A negative start time in microseconds would produce an invalid JianYing draft where segments reference positions before the timeline origin. While the current call chain in `export_draft` accumulates `current_time_secs` from zero and always passes non-negative values, the function is `pub` and can be called with arbitrary `start_secs` from other modules.

More critically, `start_secs` is an `f64` and the cast `(start_secs * SEC as f64).round() as i64` can overflow silently. For `start_secs = f64::MAX`, the multiplication produces `+Inf` and `.round()` returns `+Inf`, then `as i64` converts to an implementation-defined value (saturating to `i64::MAX` or `i64::MIN` depending on platform/LLVM behavior). This is UB-adjacent and could produce corrupted timerange data.

**Fix:**
```rust
pub fn trange_from_secs(start_secs: f64, duration_secs: f64) -> Option<Timerange> {
    if !start_secs.is_finite() || start_secs < 0.0 || !duration_secs.is_finite() || duration_secs < 0.0 {
        return None;
    }
    let start_us = (start_secs * SEC as f64).round() as i64;
    let duration_us = (duration_secs * SEC as f64).round() as i64;
    if start_us < 0 || duration_us < 0 {
        return None;
    }
    Some(Timerange { start: start_us, duration: duration_us })
}
```

## Warnings

### WR-01: `calculate_total_duration` silently defaults to 60 seconds for empty timelines

**File:** `src/jianying/builder.rs:241`
**Issue:** When all tracks are empty (no segments added), `max_end` is `None` and the function returns `Ok(60_000_000)` (60 seconds). While `export_draft` validates that the script is non-empty before reaching this code, the silent default of 60 seconds is a magic number that could mask bugs if the validation is ever bypassed or if segments fail to appear in tracks for any reason. A duration of 0 would be more correct and less likely to hide issues.
**Fix:** Replace the magic number with a named constant, or return 0 for empty timelines:
```rust
const DEFAULT_EMPTY_TIMELINE_DURATION_US: i64 = 60_000_000;
// ...
Ok(max_end.unwrap_or(DEFAULT_EMPTY_TIMELINE_DURATION_US))
```
Or better:
```rust
Ok(max_end.unwrap_or(0))
```

### WR-02: `probe_audio` does not validate parsed duration is positive and finite

**File:** `src/ffmpeg/probe.rs:142-146`
**Issue:** The function parses ffprobe stdout as `f64` but does not check whether the result is positive, finite, or non-NaN. A corrupted media file or unusual ffprobe output could produce a negative duration, `NaN`, or `Infinity`. When `audio_duration` is `NaN`, `duration.min(f64::NAN)` returns `duration` in Rust (IEEE 754 `min` semantics), masking the error. When `audio_duration` is negative, `min` returns the negative value, which would then fail at `trange_from_secs` with a confusing error message rather than a clear diagnostic.
**Fix:**
```rust
let duration: f64 = stdout
    .trim()
    .parse()
    .map_err(|e| FFmpegError::OutputParseError(format!("音频时长解析失败: {}", e)))?;
if !duration.is_finite() || duration <= 0.0 {
    return Err(FFmpegError::OutputParseError(
        format!("Invalid audio duration from ffprobe: {}", duration)
    ));
}
Ok(duration)
```

### WR-03: `probe_video` does not validate `duration_secs` is positive and finite

**File:** `src/ffmpeg/probe.rs:58-64`
**Issue:** Same pattern as WR-02. The parsed `duration_secs` could be negative, NaN, or Infinity for malformed media files. The prior fix round added fallback to `.as_f64()` for numeric ffprobe output, but neither path validates the value is sensible. While `probe_video` is not currently called from the JianYing export path, it is a `pub` function intended for use by other modules.
**Fix:**
```rust
let duration_secs = json["format"]["duration"]
    .as_str()
    .and_then(|s| s.parse::<f64>().ok())
    .or_else(|| json["format"]["duration"].as_f64())
    .ok_or_else(|| FFmpegError::OutputParseError(
        "Missing or invalid duration in ffprobe output".into()
    ))?;
if !duration_secs.is_finite() || duration_secs <= 0.0 {
    return Err(FFmpegError::OutputParseError(
        format!("Invalid video duration: {}", duration_secs)
    ));
}
```

### WR-04: `export_draft` does not validate `video_origin_path` is non-empty

**File:** `src/jianying/builder.rs:329-346`
**Issue:** `validate_export_request` checks that `draft_path` is non-empty and resolution is non-zero, but does not check that `video_origin_path` is non-empty. When `clip.video` is `None`, the code falls back to `req.video_origin_path` for the video segment (line 291). If `video_origin_path` is empty, `VideoMaterial::new` receives an empty path, `canonicalize()` fails (empty path), falls back to the empty `PathBuf`, and produces a draft with an empty material path that JianYing cannot open. The error would only surface when a user tries to open the draft in JianYing, not during export.
**Fix:**
```rust
fn validate_export_request(req: &ExportRequest) -> Result<(), JianYingError> {
    if req.script.is_empty() {
        return Err(JianYingError::Validation {
            details: "脚本不能为空".to_string(),
        });
    }
    if req.draft_path.as_os_str().is_empty() {
        return Err(JianYingError::Validation {
            details: "草稿保存路径不能为空".to_string(),
        });
    }
    if req.video_origin_path.as_os_str().is_empty() {
        return Err(JianYingError::Validation {
            details: "原始视频路径不能为空".to_string(),
        });
    }
    if req.width == 0 || req.height == 0 {
        return Err(JianYingError::Validation {
            details: "分辨率宽高必须大于 0".to_string(),
        });
    }
    Ok(())
}
```

## Info

### IN-01: `DraftFolder::create_draft` takes `&PathBuf` instead of `&Path`

**File:** `src/jianying/builder.rs:69`
**Issue:** The idiomatic Rust convention is to accept `&Path` for function parameters rather than `&PathBuf`. `PathBuf` auto-derefs to `Path`, so callers with `PathBuf` can still pass it, but `&Path` also accepts other path-like types. This is a minor API style issue.
**Fix:** Change the parameter type:
```rust
pub fn create_draft(
    draft_path: &Path,
    draft_name: &str,
    width: u32,
    height: u32,
) -> Result<(Self, ScriptFile), JianYingError> {
```

### IN-02: `parse_timestamp_start` uses `f64` for integer-valued hour/minute/second fields

**File:** `src/jianying/builder.rs:375-377`
**Issue:** Hours, minutes, and seconds are parsed as `f64` even though they are always integer values in the `"HH:MM:SS"` format. Using `f64` means inputs like `"1.5"` for the minutes field would be silently accepted (e.g., `"00:01.5:00"`). Parsing as `u32` would be more precise and reject fractional inputs naturally.
**Fix:**
```rust
let h: u32 = time_parts[0].parse().ok()?;
let m: u32 = time_parts[1].parse().ok()?;
let s: u32 = time_parts[2].parse().ok()?;
// Then the range check simplifies since u32 is always non-negative
if m >= 60 || s >= 60 {
    return None;
}
```

### IN-03: `calculate_total_duration` re-serializes track data to JSON just to read timeranges

**File:** `src/jianying/builder.rs:224-242`
**Issue:** `calculate_total_duration` calls `track.to_json()` (which serializes segments to `serde_json::Value`), then walks the resulting JSON tree to extract `target_timerange` values. The `Timerange` data is already available in the Rust `SegmentOutput` structs before serialization. This unnecessary serialization round-trip adds CPU cost and makes the code harder to follow. The `Track` struct could expose a method to compute the max end time directly.
**Fix:** Add a method to `Track` that returns the max end time from its stored segments without JSON serialization. This would require `SegmentOutput` to expose its `target_timerange`, or `Track` to store timeranges separately.

---

_Reviewed: 2026-04-29T18:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
