---
phase: 09-jianying-export
reviewed: 2026-04-30T01:51:30Z
depth: standard
files_reviewed: 13
files_reviewed_list:
  - Cargo.toml
  - src/lib.rs
  - src/jianying/mod.rs
  - src/jianying/error.rs
  - src/jianying/time.rs
  - src/jianying/types.rs
  - src/jianying/template.rs
  - src/jianying/material.rs
  - src/jianying/segment.rs
  - src/jianying/track.rs
  - src/jianying/builder.rs
  - src/ffmpeg/probe.rs
  - tests/jianying_export.rs
findings:
  critical: 0
  warning: 5
  info: 3
  total: 8
status: issues_found
---

# Phase 09: Code Review Report (Iteration 7)

**Reviewed:** 2026-04-30T01:51:30Z
**Depth:** standard
**Files Reviewed:** 13
**Status:** issues_found

## Summary

Full standard-depth review of the JianYing (CapCut) draft export module -- 13 files across `src/jianying/`, `src/ffmpeg/probe.rs`, `src/lib.rs`, `Cargo.toml`, and `tests/jianying_export.rs`. This is iteration 7.

### Previous Findings Verification

**WR-01 (iteration 6): Non-atomic file write for draft_content.json.** VERIFIED FIXED. `builder.rs` lines 244-249 now use the atomic write-rename pattern (`write to .json.tmp` then `rename`), matching the convention in `src/script/mod.rs`. The same pattern is applied to `draft_meta_info.json` at lines 103-108. Both include best-effort cleanup on rename failure. No regression.

**IN-01 (iterations 4-6, carried): Inconsistent error message language in probe.rs.** STILL PRESENT. Error messages in `probe_video` (lines 62-63, 67, 97-98) and `probe_audio` (lines 153-154) still mix Chinese and English. See IN-01 below.

### Fresh Review Assessment

The module is well-structured with comprehensive test coverage (69 unit tests passing, 10 integration tests passing, 2 ignored). No critical issues found in this iteration. Five warnings identified covering API safety gaps, path handling robustness, state cleanup, and code duplication. Three info-level items noted.

## Critical Issues

None.

## Warnings

### WR-01: Track does not enforce segment type matching track_type -- silent data corruption possible

**File:** `src/jianying/track.rs:52-58`
**Issue:** `Track::add_video_segment` and `Track::add_audio_segment` accept any segment regardless of `self.track_type`. Adding a `VideoSegment` to a `TrackType::Audio` track produces a `TrackJson` with `type: "audio"` but containing video segment JSON. JianYing would reject or misinterpret this draft. The current builder only calls these methods with matching types, so this is not a live bug, but the API is unsound -- any future caller can introduce silent data corruption.

**Fix:** Add a runtime check or use the type system:

```rust
pub fn add_video_segment(&mut self, seg: VideoSegment) -> Result<(), JianYingError> {
    if self.track_type != TrackType::Video {
        return Err(JianYingError::Validation {
            details: format!(
                "Cannot add video segment to {:?} track '{}'",
                self.track_type, self.name
            ),
        });
    }
    self.segments.push(SegmentOutput::Video(seg.to_json()));
    Ok(())
}
```

Alternatively, use separate `VideoTrack` and `AudioTrack` types to enforce this at compile time.

### WR-02: VideoMaterial::new and AudioMaterial::new silently accept invalid (non-existent, relative) paths

**File:** `src/jianying/material.rs:59` and `src/jianying/material.rs:118`
**Issue:** `path.canonicalize().unwrap_or_else(|_| path.to_path_buf())` silently falls back to the original path when the file does not exist. This means:
1. Relative paths (e.g., `"video.mp4"`) remain relative, producing an invalid absolute path in the exported JSON that JianYing cannot resolve.
2. Non-existent file paths are silently accepted without any error indication.

The function returns `Result<Self, JianYingError>` but never actually returns an error for bad paths. Callers cannot distinguish between valid and invalid paths.

**Fix:** At minimum, check that the canonicalized path is absolute:

```rust
let abs_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
if !abs_path.is_absolute() {
    return Err(JianYingError::Validation {
        details: format!("路径必须为绝对路径或已存在的文件: {}", abs_path.display()),
    });
}
```

### WR-03: export_draft leaks draft directory on partial failure

**File:** `src/jianying/builder.rs:265-345`
**Issue:** When `export_draft` fails after `DraftFolder::create_draft` succeeds (e.g., a clip has missing `source_time_range` at index 5 out of 10), the created directory and `draft_meta_info.json` remain on disk as orphaned state. Subsequent retry attempts with the same `draft_name` will hit the existing directory (which is fine on overwrite) but may confuse users who see a stale `draft_meta_info.json` referencing data that was never completed.

**Fix:** Clean up the draft directory on error:

```rust
pub fn export_draft(req: &ExportRequest) -> Result<PathBuf, JianYingError> {
    validate_export_request(req)?;
    let draft_name = req.draft_name_or_default();
    let (folder, mut script_file) =
        DraftFolder::create_draft(&req.draft_path, &draft_name, req.width, req.height)?;

    // ... add tracks, process clips ...
    // If an error occurs during clip processing, clean up:
    match build_timeline(req, &mut script_file) {
        Ok(()) => script_file.save(&folder),
        Err(e) => {
            let _ = std::fs::remove_dir_all(folder.draft_dir());
            Err(e)
        }
    }
}
```

### WR-04: trange_from_secs accepts zero duration, producing degenerate Timerange

**File:** `src/jianying/time.rs:32-41`
**Issue:** `trange_from_secs(0.0, 0.0)` returns `Some(Timerange { start: 0, duration: 0 })`. A zero-duration timerange in a JianYing draft represents a segment with no playback, which is semantically invalid. The caller `export_draft` validates `duration > 0.0` before calling this, but `trange_from_secs` is a public function -- other callers may not have this guard. Similarly, `trange("0s", "0s")` also returns `Some` with zero duration.

**Fix:** Reject zero duration explicitly:

```rust
pub fn trange_from_secs(start_secs: f64, duration_secs: f64) -> Option<Timerange> {
    if !start_secs.is_finite() || start_secs < 0.0
        || !duration_secs.is_finite() || duration_secs <= 0.0
    //                                    ^^^ changed from < to <=
    {
        return None;
    }
    // ...
}
```

Note: This would change the existing test `test_trange_zero_seconds` which expects `trange("0s", "0s")` to succeed. That test should be updated to expect `None`.

### WR-05: build_base_segment duplicated between VideoSegment and AudioSegment

**File:** `src/jianying/segment.rs:68-93` and `src/jianying/segment.rs:156-181`
**Issue:** `VideoSegment::build_base_segment` and `AudioSegment::build_base_segment` are nearly identical (25 lines each, differing only in `source_timerange` handling). Any future change to base segment fields must be made in both places, risking divergence.

**Fix:** Extract a shared helper:

```rust
fn build_base_segment(
    segment_id: &str,
    material_id: &str,
    target_timerange: Timerange,
    source_timerange: Option<Timerange>,
    speed_id: &str,
) -> SegmentJson {
    SegmentJson {
        enable_adjust: true,
        enable_color_correct_adjust: false,
        enable_color_curves: true,
        enable_color_match_adjust: false,
        enable_color_wheels: true,
        enable_lut: true,
        enable_smart_color_adjust: false,
        last_nonzero_volume: 1.0,
        reverse: false,
        track_attribute: 0,
        track_render_index: 0,
        visible: true,
        id: segment_id.to_string(),
        material_id: material_id.to_string(),
        target_timerange,
        common_keyframes: vec![],
        keyframe_refs: vec![],
        source_timerange,
        speed: 1.0,
        volume: 1.0,
        extra_material_refs: vec![speed_id.to_string()],
        is_tone_modify: false,
    }
}
```

## Info

### IN-01: Inconsistent error message language in probe.rs (carried from iteration 4)

**File:** `src/ffmpeg/probe.rs:62-63, 67, 97-98, 153-154`
**Issue:** Within `probe_video` and `probe_audio`, error messages mix Chinese (line 151: `"音频时长解析失败"`) and English (lines 62-63: `"Missing or invalid duration in ffprobe output"`, line 67: `"Invalid video duration"`, line 97: `"No video stream found in ffprobe output"`, lines 153-154: `"Invalid audio duration from ffprobe"`). The project convention (established in `src/error.rs` and `src/jianying/error.rs`) is to use Chinese for all error messages.

**Fix:** Standardize all error messages in `probe.rs` to Chinese:
```rust
// Line 62-63:
"ffprobe 输出中缺少 duration 字段或格式无效".into()
// Line 67:
format!("无效的视频时长: {}", duration_secs)
// Line 97-98:
"ffprobe 输出中未找到视频流".into()
// Line 153-154:
format!("ffprobe 返回无效的音频时长: {}", duration)
```

### IN-02: Integration test timestamp format overflow risk

**File:** `tests/jianying_export.rs:32-36`
**Issue:** The `make_clip` helper uses `format!("00:00:{:02},000-00:00:{:02},000", id * 5, id * 5 + 5)`. When `id >= 12`, `id * 5 >= 60`, producing invalid timestamps like `"00:00:60,000"`. Current test values (1-3) are safe, but this is fragile -- anyone adding a test with a larger ID will get silently wrong timestamps.
**Fix:** Use a proper timestamp formatter or clamp/wrap values.

### IN-03: serde_json::to_string used for draft_meta_info.json instead of to_string_pretty

**File:** `src/jianying/builder.rs:101`
**Issue:** `draft_meta_info.json` is written with `serde_json::to_string(&meta)` (compact, single-line), while `draft_content.json` is written with `serde_json::to_string_pretty(&content)` (formatted, multi-line). The inconsistency makes `draft_meta_info.json` harder to debug by visual inspection. JianYing likely does not care about formatting, but it is inconsistent within the codebase.
**Fix:** Use `serde_json::to_string_pretty` for both files, or document why the difference is intentional.

---

_Reviewed: 2026-04-30T01:51:30Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 7_
