---
phase: 08-sdp-pipeline
reviewed: 2026-05-08T12:00:00Z
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
  critical: 0
  warning: 3
  info: 2
  total: 5
status: issues_found
---

# Phase 08: SDP Pipeline Code Review Report (Re-review 7, Iteration 8)

**Reviewed:** 2026-05-08T12:00:00Z
**Depth:** standard
**Files Reviewed:** 20
**Status:** issues_found

## Summary

This is the eighth review pass (iteration 8) of Phase 08. All 20 source files were read and analyzed at standard depth. All previously fixed issues (CR-01 through CR-03, WR-01 through WR-17) were verified to remain in place. Three new warnings and two informational findings were identified. No critical issues were found. The codebase is in solid shape after 7 prior fix iterations -- the new findings are robustness improvements and latent inconsistencies rather than correctness bugs affecting normal operation.

## Previously Fixed Issues -- Verification

All six fixes from the most recent iterations were verified present in the current code:

| Fix | File | Lines | Status |
|-----|------|-------|--------|
| CR-01 | sdp/pipeline.rs | 59-69 | VERIFIED -- subtitle_segments populated via parse_subtitle_file |
| CR-02 | sdp/script_gen.rs | 250-275 | VERIFIED -- extraction_prompt rendered and sent to LLM via provider.generate_text |
| CR-03 | sde/pipeline.rs | 245-272 | VERIFIED -- source_time_range parsed directly from clip.timestamp |
| WR-15 | sdp/script_gen.rs | 230-239 | VERIFIED -- empty plot_titles and summary validated after step 1 |
| WR-16 | sdp/types.rs | 90-95 | VERIFIED -- temperature range [0, 2] validated in SdpRequest::validate() |
| WR-17 | sdp/clip.rs | 3 | VERIFIED -- uses crate::documentary::timestamp::secs_to_srt_time, no local duplicate |

Earlier fixes (WR-01 through WR-14) were also confirmed intact based on the code reviewed.

## Warnings

### WR-18: parse_srt_timestamp rejects valid SRT timestamps with hours > 23

**File:** `narratoai-core/src/subtitle/timestamp.rs:49-53`
**Issue:** The hour validation `if h > 23` rejects valid SRT timestamps for long-form content (videos exceeding 24 hours). SRT format imposes no upper limit on hours. The total seconds validation at line 79 (`0.0..=86400.0`) also constrains timestamps to exactly one day. This affects both the subtitle parser and the `find_precise_range` function used for OST=1 precise timestamp correction -- if any subtitle segment has hours > 23, `parse_srt_timestamp` fails on it, causing `find_precise_range` to return None and fall back to uncorrected LLM timestamps. The `parse_time_to_secs` function in `documentary/timestamp.rs` has the same `h > 23` constraint at line 51.
**Fix:** Remove the hour range check and increase the total seconds upper bound to accommodate realistic long-form content:

```rust
// Remove lines 49-53 (h > 23 check entirely)
// Change line 79 total validation to allow longer videos:
if !(0.0..=604800.0).contains(&total) {  // 7 days = 604800 seconds
```

### WR-19: SRT path containing single quote breaks FFmpeg subtitles filter

**File:** `narratoai-core/src/sde/pipeline.rs:525-537`
**Issue:** The SRT path escaping for the FFmpeg `subtitles` filter wraps the path in single quotes within the filter string: `subtitles='{}':force_style='{}'` (line 534-535). The code at line 529 attempts to escape single quotes with `replace("'", "\\'")`, but FFmpeg's filter parser does not support backslash-escaping of single quotes within single-quoted strings. If the SRT file path contains a single quote character, the FFmpeg command will fail with a parse error. The SDP composite step does not burn subtitles, so it is not affected. While paths with single quotes are rare, this is a latent failure that would produce an opaque FFmpeg error.
**Fix:** Remove the single-quote wrapping and use FFmpeg's native escaping for the entire path value:

```rust
let escaped_srt = srt_str
    .replace('\\', "/")
    .replace(':', "\\:")
    .replace("'", "\\'")
    .replace('[', "\\[")
    .replace(']', "\\]")
    .replace(';', "\\;")
    .replace(',', "\\,")
    .replace('\n', "")
    .replace('\r', "");
filter_complex_parts.push(format!(
    "[0:v]subtitles={}:force_style='{}'[vout]",
    escaped_srt, subtitle_force_style
));
```

### WR-20: Millisecond parsing inconsistency between two timestamp parsers

**File:** `narratoai-core/src/subtitle/timestamp.rs:74` vs `narratoai-core/src/documentary/timestamp.rs:67`
**Issue:** Two functions parse millisecond timestamps differently:
- `parse_srt_timestamp` (subtitle module, line 74): divides by `10^min(len,3)`, so 2-digit input "50" becomes 0.5 (500ms)
- `parse_time_to_secs` (documentary module, line 67): divides by 1000.0 unconditionally, so "50" becomes 0.05 (50ms)

For standard 3-digit SRT millis ("500"), both produce 0.5 correctly. But for non-standard 1 or 2 digit millis, the results diverge. The SDP clip step uses `parse_srt_timestamp` while the SDE pipeline uses `parse_time_to_secs`. In practice, the normalization layer always produces 3-digit millis, so this inconsistency does not trigger in normal operation. However, if raw un-normalized timestamps ever reach these functions, they would compute different durations in the SDP vs SDE pipelines.
**Fix:** Standardize both functions. Prefer the simpler approach -- always left-pad to 3 digits then divide by 1000:

```rust
// In parse_srt_timestamp, replace lines 65-75:
let ms: f64 = if millis.is_empty() {
    0.0
} else {
    let padded = format!("{:0<3}", millis);
    let ms_val: u32 = padded[..3].parse().map_err(|_| SubtitleError::ParseSubtitle {
        details: format!("毫秒解析失败: {}", millis),
    })?;
    ms_val as f64 / 1000.0
};
```

## Info

### IN-03: Empty script not explicitly validated before SDP clip step

**File:** `narratoai-core/src/sdp/pipeline.rs:53-57`
**Issue:** The G4 OST validation at line 53 uses `script.iter().any(|c| c.ost != OstType::OriginalSound)`. For an empty script, `iter().any()` returns false, so validation passes. The downstream concat step would fail because FFmpeg errors on an empty concat list file, but the error message would be an opaque FFmpeg error rather than a clear "script is empty" message. An explicit empty-script check would provide better error reporting.
**Fix:** Add an explicit check before the OST validation:
```rust
if state.script.is_empty() {
    return Err(SdpError::Validation {
        details: "脚本为空，无片段可处理".into(),
    });
}
```

### IN-04: sde/timestamp.rs is a pure re-export shim

**File:** `narratoai-core/src/sde/timestamp.rs:1-2`
**Issue:** This file only re-exports `parse_srt_timestamp` and `find_precise_range` from `crate::subtitle::timestamp`. It exists as a backward-compatibility shim after the timestamp logic was migrated to the shared subtitle module. All callers could import directly from `crate::subtitle::timestamp` instead. This is a maintainability note, not a defect.
**Fix:** No action needed now. During future refactoring, update all `use crate::sde::timestamp::{...}` imports to `use crate::subtitle::timestamp::{...}` and remove the shim file.

---

_Reviewed: 2026-05-08T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 8 (re-review)_
