---
phase: 08-sdp-pipeline
reviewed: 2026-05-07T12:00:00Z
depth: standard
files_reviewed: 20
files_reviewed_list:
  - narratoai-core/src/subtitle/mod.rs
  - narratoai-core/src/subtitle/error.rs
  - narratoai-core/src/subtitle/types.rs
  - narratoai-core/src/subtitle/parser.rs
  - narratoai-core/src/subtitle/timestamp.rs
  - narratoai-core/src/sde/types.rs
  - narratoai-core/src/sde/timestamp.rs
  - narratoai-core/src/sde/error.rs
  - narratoai-core/src/sde/pipeline.rs
  - narratoai-core/src/sde/mod.rs
  - narratoai-core/src/lib.rs
  - narratoai-core/src/prompt/templates/short_drama_editing/subtitle_analysis_v2.0.md
  - narratoai-core/src/prompt/templates/short_drama_editing/plot_extraction_v2.0.md
  - narratoai-core/src/prompt/register.rs
  - narratoai-core/src/sdp/mod.rs
  - narratoai-core/src/sdp/error.rs
  - narratoai-core/src/sdp/types.rs
  - narratoai-core/src/sdp/script_gen.rs
  - narratoai-core/src/sdp/clip.rs
  - narratoai-core/src/sdp/pipeline.rs
findings:
  critical: 0
  warning: 8
  info: 4
  total: 12
status: issues_found
---

# Phase 08: SDP Pipeline Code Review Report

**Reviewed:** 2026-05-07T12:00:00Z
**Depth:** standard
**Files Reviewed:** 20
**Status:** issues_found

## Summary

Reviewed 20 files across the subtitle extraction layer, SDE pipeline, SDP pipeline, and prompt registration. The subtitle module is clean -- error types, parsing, and timestamp handling are well-structured. The defects cluster in the SDP pipeline: silent timestamp parse failures, zero-duration clips dropped without error, missing file-existence checks, blocking I/O in async context, and a redundant public API parameter. The SDE pipeline has one code quality issue (redundant TTS skip condition). No critical security bugs were confirmed -- the `'\''` concat escaping is correct for FFmpeg because `av_get_token()` handles backslash-escape outside quotes. The most impactful pattern is that SDP consistently uses `unwrap_or(0.0)` instead of propagating errors, while SDE uses `map_err` -- this means SDP hides root causes that SDE would surface.

## Warnings

### WR-01: Silent timestamp parse failure in SDP clip step

**File:** `narratoai-core/src/sdp/clip.rs:61-62`
**Issue:** Uses `unwrap_or(0.0)` on `parse_srt_timestamp()`, silently swallowing parse errors. If timestamp format is invalid, both `start_secs` and `end_secs` become 0.0, yielding zero duration, and the clip is silently dropped. The SDE pipeline (line 249-252 of `pipeline.rs`) correctly propagates this as an error via `map_err`.

Additionally, if `end_secs < start_secs`, the expression `(end_secs - start_secs).max(0.0)` returns 0.0 instead of warning about an inverted timestamp range.

```rust
// Current (lines 61-62):
let start_secs = parse_srt_timestamp(ts_parts[0]).unwrap_or(0.0);
let end_secs = parse_srt_timestamp(ts_parts[1]).unwrap_or(0.0);
let clip_duration = (end_secs - start_secs).max(0.0);
```

**Fix:** Propagate error instead of defaulting, and check for inverted ranges:
```rust
let start_secs = parse_srt_timestamp(ts_parts[0])
    .map_err(|e| SdpError::ParseSubtitle {
        details: format!("片段 {} 开始时间戳解析失败: {}", clip._id, e),
    })?;
let end_secs = parse_srt_timestamp(ts_parts[1])
    .map_err(|e| SdpError::ParseSubtitle {
        details: format!("片段 {} 结束时间戳解析失败: {}", clip._id, e),
    })?;
if end_secs < start_secs {
    return Err(SdpError::Validation {
        details: format!("片段 {} 结束时间 ({}) 早于开始时间 ({})", clip._id, end_secs, start_secs),
    });
}
let clip_duration = end_secs - start_secs;
```

### WR-02: Zero-duration clips silently skipped without error

**File:** `narratoai-core/src/sdp/clip.rs:63-65`
**Issue:** When `clip_duration <= 0.0` after the `max(0.0)` guard, the `if clip_duration > 0.0` block is skipped entirely -- no error, no log, no cumulative time advancement. This creates two downstream defects:
1. The output timeline has silent gaps where zero-duration clips were dropped.
2. `state.total_duration` is shorter than expected, causing BGM fade-out timing in `sdp_step_composite` to trigger too early.

The SDE pipeline (lines 243-247) correctly returns an error:
```rust
if clip_duration <= 0.0 {
    return Err(SdeError::VideoProcess(PipelineError::VideoClip {
        details: format!("片段 {} 时长计算为零", clip._id),
    }));
}
```

**Fix:** Replace the silent skip with an authoritative error check matching SDE's pattern.

### WR-03: Missing file existence checks in SdpRequest::validate

**File:** `narratoai-core/src/sdp/types.rs:52-76`
**Issue:** `SdpRequest::validate()` checks that paths are non-empty but does NOT verify that `subtitle_path` or `video_path` point to existing files. `SdeRequest::validate()` (sde/types.rs lines 67-72) performs these checks. This means `run_sdp` or `generate_sdp_script` proceeds with non-existent files and fails later with a less informative system error.

**Fix:** Add existence checks matching SDE's pattern:
```rust
if !self.subtitle_path.exists() {
    return Err(format!("字幕文件不存在: {}", self.subtitle_path.display()));
}
if !self.video_path.exists() {
    return Err(format!("视频文件不存在: {}", self.video_path.display()));
}
```

### WR-04: Missing newline/control-character guard in SDP concat step

**File:** `narratoai-core/src/sdp/pipeline.rs:88-100`
**Issue:** `sdp_step_concat` writes video paths into the FFmpeg concat demuxer list file without checking for newline characters (`\n`, `\r`). The SDE pipeline (lines 308-312) validates and rejects such paths. If a path contains a newline, the concat file format is corrupted -- FFmpeg reads partial paths, producing wrong output or cryptic errors.

**Fix:** Add the same guard as SDE:
```rust
let path_str = video_path.to_string_lossy().replace('\\', "/");
if path_str.contains('\n') || path_str.contains('\r') {
    return Err(SdpError::Validation {
        details: format!("视频路径包含非法字符: {}", clip._id),
    });
}
let escaped = path_str.replace("'", "'\\''");
concat_content.push_str(&format!("file '{}'\n", escaped));
```

### WR-05: Unused `_config` parameter on public API `run_sdp`

**File:** `narratoai-core/src/sdp/pipeline.rs:20`
**Issue:** `run_sdp` accepts `_config: &crate::config::types::AppConfig` but never uses it. As a `pub async fn`, every caller must pass a config reference that is silently ignored. This creates coupling without benefit -- if the function later needs config, changing the signature is a breaking change. The underscore prefix signals "unused" but does not justify keeping the parameter in the public API.

**Fix:** Remove the parameter. If future needs arise, add it back:
```rust
pub async fn run_sdp(
    request: SdpRequest,
    progress: Option<SdpProgressCallback>,
) -> Result<PathBuf, SdpError> {
```
Update all callers accordingly.

### WR-06: Synchronous blocking I/O in async context

**File:** `narratoai-core/src/sdp/pipeline.rs:34` (`std::fs::create_dir_all`), `narratoai-core/src/sdp/pipeline.rs:102` (`std::fs::write`)
**Issue:** Synchronous file operations are called directly in `async fn run_sdp` and `async fn sdp_step_concat` without `spawn_blocking`. This blocks the tokio worker thread. In a single-threaded runtime, this could cause deadlock (blocked thread cannot poll other futures). The SDE pipeline consistently uses `tokio::fs::create_dir_all` and `tokio::fs::write` for these operations.

Also, `sdp_step_concat` (line 102) returns `Result<(), SdpError>` via `std::fs::write?` -- the `?` works because `From<std::io::Error> for SdpError` is implemented. But the function is `async` while doing only synchronous work, which is misleading.

**Fix:** Use tokio async variants:
```rust
// Line 34:
tokio::fs::create_dir_all(&task_dir).await?;
// Line 102:
tokio::fs::write(&concat_list_path, &concat_content).await?;
```

### WR-07: Redundant TTS skip condition in SDE pipeline

**File:** `narratoai-core/src/sde/pipeline.rs:158-163`
**Issue:** Two adjacent `if` blocks check TTS skip conditions:
```rust
if clip.narration.starts_with("播放原片") && clip.ost == OstType::OriginalSound {
    continue;
}
// OST=1 片段使用原始音轨，无需 TTS
if clip.ost == OstType::OriginalSound {
    continue;
}
```
The second `if` already handles ALL `OstType::OriginalSound` clips regardless of narration content. The first `if` block's `"播放原片"` prefix check adds no unique behavior: every clip that enters the first block also enters the second. The first block is dead code.

The comment says `"播放原片" 前缀仅对 OST=1 片段跳过 TTS；OST=0/2 仍需 TTS 结果` -- but the code doesn't implement this intent for OST=0/2 since the `&&` condition requires OST=OriginalSound anyway.

**Fix:** Remove the first block entirely. If the intent was to skip TTS for "播放原片" prefix on any OST type, remove the `&& clip.ost == OstType::OriginalSound` condition:
```rust
// Option A: Remove redundant block
if clip.ost == OstType::OriginalSound {
    continue;
}

// Option B: If "播放原片" should skip TTS regardless of OST:
if clip.narration.starts_with("播放原片") {
    continue;
}
```

### WR-08: Missing intermediate progress checkpoint in SDP composite step

**File:** `narratoai-core/src/sdp/pipeline.rs:71-73`
**Issue:** The function doc comment documents composite as `60% to 90% to 100%`, but the code only emits `60.0` (line 71) and `100.0` (line 73). The `90%` checkpoint is never emitted. SDE correctly emits both `95%` (start) and `100%` (end). Progress callback consumers relying on the documented step granularity miss a progress update.

**Fix:** Insert an intermediate progress emission:
```rust
state.emit_progress("composite", 60.0, "正在最终合成...");
sdp_step_composite(&mut state, &request).await?;
state.emit_progress("composite", 90.0, "合成完成");  // Missing
state.emit_progress("composite", 100.0, "最终合成完成");
```

## Info

### IN-01: Duplicate `secs_to_timestamp_str` function

**File:** `narratoai-core/src/sdp/clip.rs:87-100`
**Issue:** `sdp/clip.rs` defines `secs_to_timestamp_str` which is functionally identical to `secs_to_srt_time` from `crate::documentary::timestamp` (documentary/timestamp.rs:82-95). The SDE pipeline imports and uses `secs_to_srt_time` for the same purpose. Duplication creates maintenance risk if the format needs to change.

**Fix:** Import `secs_to_srt_time` from documentary and remove the local duplicate:
```rust
use crate::documentary::timestamp::secs_to_srt_time;
```

### IN-02: Unused `_encoding` variable in `generate_sdp_script`

**File:** `narratoai-core/src/sdp/script_gen.rs:156`
**Issue:** The `_encoding` field from `parse_subtitle_file` is deliberately ignored (prefix `_`). The `segments` variable is only used to build `subtitle_plain` text via iteration -- structured segment data (with timestamps) is discarded. This means `generate_sdp_script` re-parses timestamps from plain text rather than using the already-parsed structured data, which is wasteful since `detect_encoding` (a CPU-intensive operation) already ran.

### IN-03: Inconsistent timestamp parser usage across SDE and SDP

**File:** `narratoai-core/src/sde/pipeline.rs:249`, `narratoai-core/src/sdp/clip.rs:61`
**Issue:** SDE uses `parse_time_to_secs` from `crate::documentary::timestamp` for clip duration calculation. SDP uses `parse_srt_timestamp` from `crate::subtitle::timestamp`. These parsers differ: `parse_srt_timestamp` normalizes `'.'` to `','` first (supporting both separators), while `parse_time_to_secs` only accepts `','`. This inconsistency means SDP handles non-standard timestamp formats more gracefully than SDE.

### IN-04: `SdeError` has no `From<ScriptError>` impl

**File:** `narratoai-core/src/sde/error.rs` (all lines)
**Issue:** `SdeError` does not implement `From<crate::script::error::ScriptError>`, but `sde/pipeline.rs:128` calls `parse_script(&state.narration_raw)?;` which (based on import path `crate::sde::script_gen::parse_script`) apparently returns a type compatible with `SdeError`. If `parse_script` is ever refactored to return `ScriptError` instead, this call site silently breaks. `SdpError` (sdp/error.rs:45-60) correctly implements `From<ScriptError>` -- SDE should follow the same pattern for consistency and robustness.

---

_Reviewed: 2026-05-07T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
