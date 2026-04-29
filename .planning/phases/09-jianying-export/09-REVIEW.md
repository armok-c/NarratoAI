---
phase: 09-jianying-export
reviewed: 2026-04-29T16:30:00Z
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
  critical: 0
  warning: 4
  info: 0
  total: 4
status: issues_found
---

# Phase 9: Code Review Report (Re-review after fixes)

**Reviewed:** 2026-04-29T16:30:00Z
**Depth:** standard
**Files Reviewed:** 13
**Status:** issues_found

## Summary

Re-reviewed all 13 source files after the code review fix round (commits `66d4015..dc1129f`). The previous 2 critical issues (CR-01: silent segment loss; CR-02: parse_timestamp_start mishandling) have been properly fixed and verified. Of the 7 warnings, 5 were fixed (WR-04, WR-05, WR-06, WR-07, and WR-02/WR-03/IN-01 combined). Two previously reported warnings remain partially unfixed (WR-01) or not addressed at the source level.

The remaining issues are all at the warning level -- no critical bugs or security vulnerabilities were found in the current codebase. The code is well-structured with comprehensive unit and integration tests.

### Previous Findings Status

| ID | Description | Status |
|----|-------------|--------|
| CR-01 | Silent segment loss when track name does not match | **FIXED** -- `add_video_segment`/`add_audio_segment` now return `Result` with rollback |
| CR-02 | `parse_timestamp_start` mishandles comma-delimited ms | **FIXED** -- uses `rsplit_once('-')` and validates ranges |
| WR-01 | No validation for negative duration in Timerange | **PARTIALLY FIXED** -- `trange()` rejects negative, but `trange_from_secs()` still accepts |
| WR-02 | `parse_seconds` silently returns 0 | **FIXED** -- returns `Option<i64>` |
| WR-03 | Unnecessary `clone()` on freshly constructed String | **FIXED** -- `id` moved directly |
| WR-04 | `#[serde(flatten)]` collision risk | **FIXED** -- design note added |
| WR-05 | `Track::to_json()` uses `expect()` | **FIXED** -- returns `Result<TrackJson, JianYingError>` |
| WR-06 | `ScriptFile::save()` uses `expect()` | **FIXED** -- uses `?` propagation |
| WR-07 | Integration tests use `unwrap()` | **FIXED** -- replaced with `expect()` |
| IN-01 | Unused `id` local variable | **FIXED** -- removed along with unnecessary clone |
| IN-02 | `DraftContentJson` unused | **FIXED** -- removed |
| IN-03 | `probe_video_async` unused | **FIXED** -- removed |

## Warnings

### WR-01: `trange_from_secs` accepts negative duration without validation

**File:** `src/jianying/time.rs:30-35`
**Issue:** The `trange()` function was fixed to reject negative duration (returns `None`), but `trange_from_secs()` -- the primary function used in the export pipeline (`builder.rs:273,282,299`) -- accepts any `f64` including negative values. If `clip.duration` is `Some(-5.0)`, `trange_from_secs` will produce a `Timerange { start: ..., duration: -5000000 }`, creating a corrupted JianYing draft with negative-duration segments. This is a data integrity risk for the exported draft.

**Fix:**
```rust
pub fn trange_from_secs(start_secs: f64, duration_secs: f64) -> Option<Timerange> {
    let start_us = (start_secs * SEC as f64).round() as i64;
    let duration_us = (duration_secs * SEC as f64).round() as i64;
    if duration_us < 0 {
        return None;
    }
    Some(Timerange { start: start_us, duration: duration_us })
}
```
Callers (e.g., `export_draft`) would need to handle the `Option` with `.ok_or_else(...)`.

### WR-02: `probe_video` only accepts string-typed duration from ffprobe JSON

**File:** `src/ffmpeg/probe.rs:58-63`
**Issue:** The duration extraction uses `.as_str()` which only works when ffprobe outputs the duration as a JSON string (e.g., `"duration": "120.5"`). If ffprobe outputs it as a JSON number (e.g., `"duration": 120.5`), `.as_str()` returns `None` and the function fails with a parse error. While most ffprobe versions output duration as a string, some versions or configurations may return a number.

**Fix:**
```rust
let duration_secs = json["format"]["duration"]
    .as_str()
    .and_then(|s| s.parse::<f64>().ok())
    .or_else(|| json["format"]["duration"].as_f64())
    .ok_or_else(|| FFmpegError::OutputParseError(
        "Missing or invalid duration in ffprobe output".into()
    ))?;
```

### WR-03: `export_draft` does not validate `duration` is positive

**File:** `src/jianying/builder.rs:268-273`
**Issue:** The `export_draft` function checks that `clip.duration` is `Some` but does not validate that the value is positive. A `duration` of `0.0` or a negative value would pass through and create invalid segments. Zero-duration segments in a JianYing draft could cause the editor to behave unexpectedly or crash. This is the caller-side complement to WR-01.

**Fix:**
```rust
let duration = clip.duration.ok_or_else(|| JianYingError::MissingField {
    field: "duration".to_string(),
    clip_index: i,
})?;
if duration <= 0.0 {
    return Err(JianYingError::Validation {
        details: format!("第 {} 段时长必须为正数，实际: {}", i + 1, duration),
    });
}
```

### WR-04: `calculate_total_duration` silently swallows serialization errors

**File:** `src/jianying/builder.rs:228-229`
**Issue:** The method uses `.ok()?` in a `filter_map` to convert `Result<TrackJson, JianYingError>` into `Option<TrackJson>`. If `track.to_json()` fails for any track, the error is silently discarded and that track's segments are excluded from the total duration calculation. This could produce an incorrect `duration` field in the exported draft. Since `save()` also calls `to_json()` for each track and would fail at that point, the practical impact is low, but the silent error swallowing makes debugging harder.

**Fix:**
```rust
fn calculate_total_duration(&self) -> Result<i64, JianYingError> {
    let max_end = self
        .tracks
        .iter()
        .map(|track| track.to_json())  // Propagate errors via ?
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .filter_map(|json| {
            json.segments.iter().filter_map(|seg| {
                let target = seg.get("target_timerange")?;
                let start = target.get("start")?.as_i64()?;
                let duration = target.get("duration")?.as_i64()?;
                Some(start + duration)
            }).max()
        })
        .max();
    Ok(max_end.unwrap_or(60_000_000))
}
```

---

_Reviewed: 2026-04-29T16:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
