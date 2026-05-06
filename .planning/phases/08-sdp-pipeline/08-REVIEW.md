---
phase: 08-sdp-pipeline
reviewed: 2026-05-06T12:00:00Z
depth: standard
files_reviewed: 21
files_reviewed_list:
  - src/lib.rs
  - src/prompt/register.rs
  - src/prompt/templates/short_drama_editing/plot_extraction_v2.0.md
  - src/prompt/templates/short_drama_editing/subtitle_analysis_v2.0.md
  - src/sde/error.rs
  - src/sde/mod.rs
  - src/sde/pipeline.rs
  - src/sde/timestamp.rs
  - src/sde/types.rs
  - src/sdp/clip.rs
  - src/sdp/error.rs
  - src/sdp/mod.rs
  - src/sdp/pipeline.rs
  - src/sdp/script_gen.rs
  - src/sdp/types.rs
  - src/subtitle/error.rs
  - src/subtitle/mod.rs
  - src/subtitle/parser.rs
  - src/subtitle/timestamp.rs
  - src/subtitle/types.rs
findings:
  critical: 0
  warning: 9
  info: 4
  total: 13
status: issues_found
---

# Phase 08: Code Review Report

**Reviewed:** 2026-05-06T12:00:00Z
**Depth:** standard
**Files Reviewed:** 21
**Status:** issues_found

## Summary

Phase 08 extracted `src/subtitle/` as a public module, created SDP pipeline (`src/sdp/`) with LLM-driven script generation and video composition, and added prompt templates for short drama editing (subtitle_analysis/plot_extraction v2.0). The codebase is structurally sound with good error type hygiene, but has several quality issues: silent error swallowing in timestamp parsing (SDP clip), cross-module coupling (SDP importing from SDE instead of subtitle directly), unhandled edge cases (video without audio stream), and duplicated utility functions. No critical blockers were found -- the pipeline will function correctly for normal inputs but may fail silently or confusingly for edge cases.

## Warnings

### WR-01: Timestamp parse errors silently swallowed in sdp_step_clip

**File:** `src/sdp/clip.rs:62-63`
**Issue:** `parse_srt_timestamp` returns `Result<f64, SubtitleError>`, but the results are consumed with `.unwrap_or(0.0)`, silently defaulting to 0.0 on parse failure. If both start and end timestamps fail to parse, `clip_duration` becomes `(0.0 - 0.0).max(0.0) = 0.0`, the `if clip_duration > 0.0` guard skips setting duration/time_range, and the clip is effectively dropped from the output without any error or warning.

```rust
// Current (line 62-64):
let start_secs = parse_srt_timestamp(ts_parts[0]).unwrap_or(0.0);
let end_secs = parse_srt_timestamp(ts_parts[1]).unwrap_or(0.0);
let clip_duration = (end_secs - start_secs).max(0.0);
```

**Fix:** Propagate the error instead of silently defaulting:

```rust
let start_secs = parse_srt_timestamp(ts_parts[0])
    .map_err(|e| SdpError::ParseSubtitle {
        details: format!("解析片段 {} 开始时间戳失败: {}", clip._id, e),
    })?;
let end_secs = parse_srt_timestamp(ts_parts[1])
    .map_err(|e| SdpError::ParseSubtitle {
        details: format!("解析片段 {} 结束时间戳失败: {}", clip._id, e),
    })?;
```

### WR-02: SDP clip imports from SDE module instead of direct subtitle module

**File:** `src/sdp/clip.rs:4-5`
**Issue:** Imports `find_precise_range` and `parse_srt_timestamp` via `crate::sde::timestamp`, and `SubtitleSegment` via `crate::sde::types`. This creates an unnecessary coupling between SDP and SDE -- the SDE module is only a transitive re-export of the subtitle module. If the SDE module is refactored or removed, SDP compilation breaks.

```rust
// Current:
use crate::sde::timestamp::{find_precise_range, parse_srt_timestamp};
use crate::sde::types::SubtitleSegment;
```

**Fix:** Import directly from the canonical subtitle module:

```rust
use crate::subtitle::timestamp::{find_precise_range, parse_srt_timestamp};
use crate::subtitle::types::SubtitleSegment;
```

### WR-03: Composite step assumes video always has audio track

**File:** `src/sdp/pipeline.rs:196` (also affects `src/sde/pipeline.rs:439`)
**Issue:** The `sdp_step_composite` function unconditionally references `[0:a]` in filter_complex (line 196: `filter_complex_parts.push(format!("[0:a]volume={:.2}[orig]", original_volume))`). If the input video has no audio stream, FFmpeg will fail with a stream index error. The SDE pipeline has the same issue -- it checks `has_original_audio` before adding `[0:a]` but never probes whether the video file actually has an audio stream.

**Fix:** Before building the filter graph, probe the video file for audio stream presence (e.g., using `ffprobe` or checking the video container metadata), and conditionally include `[0:a]` based on actual availability, not just OST type.

### WR-04: Catch-all branch in ScriptError-to-SdpError conversion loses type information

**File:** `src/sdp/script_gen.rs:119-122`
**Issue:** The `merge_script` function maps `crate::script::error::ScriptError` to `SdpError` with a catch-all `_` branch:

```rust
_ => SdpError::Io {
    source: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
},
```

`ScriptError` has 5 variants (`JsonParse`, `Validation`, `Io`, `IndexOutOfBounds`, `InvalidTimestamp`). Only 3 are explicitly mapped; `IndexOutOfBounds` and `InvalidTimestamp` fall through to the catch-all, losing semantic information by wrapping them as generic IO errors. Additionally, `JsonParse` is mapped to `SdpError::JsonRepair`, which conflates "JSON parsing failure" with "JSON repair failure" -- these are semantically distinct concepts.

**Fix:** Add explicit arms for all variants:

```rust
crate::script::error::ScriptError::Validation(errors) => SdpError::Validation {
    details: format!("脚本校验失败: {:?}", errors),
},
crate::script::error::ScriptError::Io(io) => SdpError::Io { source: io },
crate::script::error::ScriptError::JsonParse(je) => SdpError::JsonRepair {
    details: format!("JSON 解析失败: {}", je),
},
crate::script::error::ScriptError::IndexOutOfBounds => SdpError::Validation {
    details: "脚本索引越界".into(),
},
crate::script::error::ScriptError::InvalidTimestamp(ts) => SdpError::ParseSubtitle {
    details: format!("脚本中无效的时间戳: {}", ts),
},
```

### WR-05: SDP depends on SDE's repair_json function

**File:** `src/sdp/script_gen.rs:218, 266`
**Issue:** `generate_sdp_script` calls `crate::sde::script_gen::repair_json()` twice (lines 218 and 266), creating a module dependency from `sdp` to `sde`. This is architecturally incorrect because:
1. The SDP module should not depend on SDE internals
2. If `repair_json` is refactored or moved, SDP silently breaks
3. It prevents independent testing of SDP without SDE

**Fix:** Move `repair_json` into a shared utility location (e.g., `src/prompt/` or a new `src/utils/json.rs`) and have both SDE and SDP import from there.

### WR-06: Fragile magic string for TTS skip detection

**File:** `src/sde/pipeline.rs:145`
**Issue:** The TTS loop skips narration entries starting with `"播放原片"`:

```rust
if clip.narration.starts_with("播放原片") {
    continue;
}
```

This is coupled to the exact string format generated by the LLM prompt (`script_generation_v2.0.md` defines narration format as `"播放原片+序号"`). If the prompt template is updated to use a different prefix, TTS generation will silently attempt to synthesize audio for playback-only clips, producing wasted API calls and corrupted output. This is a fragile convention that should be enforced at the type level.

**Fix:** Add a dedicated boolean field (e.g., `skip_tts: bool`) to `ScriptClip` in addition to the OST type, populated during script parsing rather than relying on string prefix matching.

### WR-07: Subtitle blocks silently dropped on parse failure

**File:** `src/subtitle/parser.rs:193, 215-222`
**Issue:** Multiple parse failures are silently swallowed with no logging:

- Line 193: `lines[0].trim().parse().unwrap_or(0)` -- if the block index line is non-numeric, it defaults to 0 without warning
- Lines 215-222: `continue` on `parse_srt_timestamp` error -- malformed timestamps cause the entire subtitle block to be silently skipped

This means a subtitle file with a single malformed entry will lose that entry without any indication to the user or caller.

**Fix:** Log a warning via `tracing::warn!` when a subtitle block is skipped due to parse failure, including the raw block text for debugging:

```rust
if let Err(e) = parse_srt_timestamp(ts_parts[0].trim()) {
    tracing::warn!("跳过字幕块第 {} 行: 开始时间戳解析失败: {}", ...);
    continue;
}
```

### WR-08: Fragile implicit ordering between generate_sdp_script and run_sdp

**File:** `src/sdp/pipeline.rs:39-42`
**Issue:** When `script_path` is `None`, `run_sdp` defaults to `state.task_dir.join("merged_subtitle.json")`. This file is created by `generate_sdp_script()`, meaning the caller must call `generate_sdp_script` first and then `run_sdp` with `script_path: None`. However, this ordering constraint is not enforced at the type or documentation level. If `script_path` is `None` but `merged_subtitle.json` doesn't exist, the error will be a cryptic `No such file or directory` from `load_script`.

**Fix:** Either make `script_path` required (not `Option`), or document the ordering constraint clearly in the doc comment for `run_sdp`. A stronger fix: accept the output of `generate_sdp_script` directly (a `Script`) rather than requiring it to be serialized and re-read.

### WR-09: Duplicate `secs_to_timestamp_str` across three modules

**File:** `src/sdp/clip.rs:88-101` (also `src/sde/clip.rs:48-61`, `src/documentary/timestamp.rs:82-94`)
**Issue:** The same `secs → SRT timestamp` conversion logic is implemented in three places with near-identical code:
- `src/sdp/clip.rs:secs_to_timestamp_str` (used by SDP clip)
- `src/sde/clip.rs:secs_to_timestamp_str` (used by SDE clip)
- `src/documentary/timestamp.rs:secs_to_srt_time` (exported and used by SDE pipeline)

Any bug fix or format change must be applied across all three, creating a maintenance risk.

**Fix:** Move a single `secs_to_srt_time` to `src/subtitle/timestamp.rs` (since it's a subtitle-format utility) and have all callers import from there.

## Info

### IN-01: Redundant range check in SdeRequest validation

**File:** `src/sde/types.rs:65-66`
**Issue:** The `voice_rate` validation uses two overlapping checks:

```rust
if !(0.0..=5.0).contains(&self.voice_rate) || self.voice_rate <= 0.0 {
```

The range `(0.0..=5.0)` already excludes values < 0.0 and > 5.0. The second check `self.voice_rate <= 0.0` only additionally catches `0.0` itself. The same effective range could be expressed as `self.voice_rate <= 0.0 || self.voice_rate > 5.0` without the redundant `contains` call. Not harmful, just messy.

### IN-02: Dead code: extract_text_from_srt and extract_text_from_ass

**File:** `src/subtitle/parser.rs:342-403`
**Issue:** `extract_text_from_srt` (line 342) and `extract_text_from_ass` (line 370) are defined but never called anywhere in the reviewed files. They are also not re-exported from `src/subtitle/mod.rs`, making them inaccessible to external callers. Either remove them or export them if they are intended for future use.

### IN-03: CSS shorthand color silently defaults instead of expanding

**File:** `src/sde/pipeline.rs:407-412`
**Issue:** The hex-to-ASS color conversion checks `hex.len() == 6` to distinguish between `#RRGGBB` and shorthand. If a user passes `#FFF` (valid CSS 3-digit shorthand), it falls through to the default white (`&H00FFFFFF`) instead of expanding to `#FFFFFF` (`&H000000FF`). Not a runtime crash but produces unexpected color output.

### IN-04: `unwrap_or` fallback may pass invalid timestamps to parser

**File:** `src/sde/pipeline.rs:235`
**Issue:** `clip.timestamp.split('-').next().unwrap_or(&clip.timestamp)` -- if the timestamp string lacks a `-` separator, the full string (including the end time) is passed to `parse_time_to_secs`, which will likely fail with a confusing error message. Using `unwrap_or` masks the actual precondition violation.

---

_Reviewed: 2026-05-06T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
