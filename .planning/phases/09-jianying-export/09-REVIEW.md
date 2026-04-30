---
phase: 09-jianying-export
reviewed: 2026-04-30T12:00:00Z
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
  warning: 1
  info: 1
  total: 2
status: issues_found
---

# Phase 09: Code Review Report (Iteration 6)

**Reviewed:** 2026-04-30T12:00:00Z
**Depth:** standard
**Files Reviewed:** 13
**Status:** issues_found

## Summary

Full standard-depth review of the JianYing (CapCut) draft export module -- 13 files across `src/jianying/`, `src/ffmpeg/probe.rs`, `src/lib.rs`, `Cargo.toml`, and `tests/jianying_export.rs`. This is iteration 6.

### Previous Findings Verification

**CR-01 (iteration 5): parse_timestamp_start wrong result for single-segment timestamps.** VERIFIED FIXED. Commit 98a5f53 correctly handles the edge case: `rsplit_once('-')?` returns `None` for single timestamps like `"00:00:07,559"` (no `-` present), and the function correctly returns `None`. The docstring now explicitly states the input must contain `-`. New test `test_parse_timestamp_start_single_timestamp_rejected` confirms. No regression introduced.

**WR-01 (iteration 5): parse_seconds does not validate negative values.** VERIFIED FIXED. Commit 98a5f53 adds `if us < 0 { return None; }` after the `f64 -> i64` conversion at `time.rs:54-56`. Combined with the pre-existing `trange()` negative checks at line 20-22, this provides defense-in-depth. New test `test_parse_seconds_negative_returns_none` covers `"-5s"` and `"-3.0"`. No regression introduced.

**IN-01 (iterations 4-5, carried): Inconsistent error message language in probe.rs.** STILL PRESENT. See IN-01 below.

### Fresh Review Assessment

The module is well-structured with comprehensive test coverage (39+ unit tests across 7 files, 12 integration tests). No critical issues found in this iteration. The code demonstrates careful error handling, proper input validation, and clean separation of concerns. One warning identified (non-atomic file write pattern) and one carried info item.

## Critical Issues

None.

## Warnings

### WR-01: Non-atomic file write for draft_content.json risks data corruption

**File:** `src/jianying/builder.rs:237-238`
**Issue:** `ScriptFile::save` writes `draft_content.json` using `std::fs::write`, which is not atomic. If the process crashes or the disk fills mid-write, the file will be left in a partial/corrupt state. The same codebase's `save_script` function (in `src/script/mod.rs:147-153`) already uses an atomic write-rename pattern (`write to .tmp` then `rename`), establishing a project convention that should be followed here for consistency and robustness.

The same non-atomic pattern also applies to `draft_meta_info.json` at `builder.rs:102`, though that file is smaller and less likely to be interrupted.

**Fix:**
```rust
// In ScriptFile::save, replace lines 237-238:
let json_str = serde_json::to_string_pretty(&content)?;
let temp_path = content_path.with_extension("json.tmp");
std::fs::write(&temp_path, &json_str)?;
if let Err(e) = std::fs::rename(&temp_path, &content_path) {
    let _ = std::fs::remove_file(&temp_path); // best-effort cleanup
    return Err(JianYingError::Io(e));
}
```

## Info

### IN-01: Inconsistent error message language in probe_audio (carried from iteration 4)

**File:** `src/ffmpeg/probe.rs:151,153-154`
**Issue:** Within the same function `probe_audio`, error messages mix Chinese (line 151: `"音频时长解析失败"`) and English (lines 153-154: `"Invalid audio duration from ffprobe"`). The project convention (established in `src/error.rs` and `src/jianying/error.rs`) is to use Chinese for all error messages. The same file has additional English messages at lines 63 (`"Missing or invalid duration in ffprobe output"`), 67 (`"Invalid video duration"`), and 97 (`"No video stream found in ffprobe output"`).

**Fix:** Standardize all error messages in `probe.rs` to Chinese:
```rust
// Line 63:
"ffprobe 输出中缺少 duration 字段或格式无效".into()
// Line 67:
format!("无效的视频时长: {}", duration_secs)
// Line 97:
"ffprobe 输出中未找到视频流".into()
// Line 153-154:
format!("ffprobe 返回无效的音频时长: {}", duration)
```

---

_Reviewed: 2026-04-30T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 6_
