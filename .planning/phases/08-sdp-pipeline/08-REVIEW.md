---
phase: 08-sdp-pipeline
reviewed: 2026-05-08T23:45:00Z
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

# Phase 08: SDP Pipeline Code Review Report (Re-review 6, Iteration 7)

**Reviewed:** 2026-05-08T23:45:00Z
**Depth:** standard
**Files Reviewed:** 20
**Status:** issues_found

## Summary

This is the SEVENTH review pass (iteration 7) of Phase 08. All previously reported issues (CR-01 through CR-03, WR-01 through WR-14) remain correctly fixed. This pass found 3 new warnings and 2 info items.

The code is in good shape overall. The SDP pipeline correctly reuses infrastructure from SDE, documentary, and subtitle modules. Error handling is thorough, input validation covers most edge cases, and FFmpeg operations are properly wrapped in `spawn_blocking`. The new findings are quality/robustness improvements rather than correctness bugs.

## Previously Confirmed Fixed Issues

All previously reported issues remain correctly fixed:

- **CR-01** (subtitle_segments never populated in SDP pipeline): Fixed in `sdp/pipeline.rs:59-69`
- **CR-02** (step 2 sends plain text to LLM instead of rendered prompt): Fixed in `sdp/script_gen.rs:238-244`
- **CR-03** (SDE pipeline computes source_time_range incorrectly for OST=0/OST=2): Fixed in `sde/pipeline.rs:245-271`
- **WR-01 through WR-14**: All remain correctly fixed

## Warnings

### WR-15: `generate_sdp_script` does not validate empty `plot_titles` from step 1 LLM analysis

**File:** `narratoai-core/src/sdp/script_gen.rs:229-236`
**Issue:** After step 1 (subtitle_analysis), the `analysis.plot_titles` vector is used directly without checking if it is empty. If the LLM returns an empty `plot_titles` array, `format_plot_titles` returns `""`, and step 2 (plot_extraction) receives an empty `plot_titles` variable in the prompt. This will likely cause the LLM to produce garbage results or fail to generate meaningful plot points, but the error message will be confusing (a JSON parse error or "plot_points 为空" from `merge_script`) rather than a clear "第一步分析未产生情节点" message.

The `summary` field has the same problem -- an empty summary is passed directly to step 2.

**Fix:** Add validation after parsing the step 1 result:

```rust
// ---- 8. 校验分析结果 ----
if analysis.plot_titles.is_empty() {
    return Err(SdpError::LlmError {
        details: "第一步字幕分析未产生任何情节点（plot_titles 为空）".into(),
    });
}
if analysis.summary.trim().is_empty() {
    return Err(SdpError::LlmError {
        details: "第一步字幕分析未产生剧情梗概（summary 为空）".into(),
    });
}
```

### WR-16: `SdpRequest::validate()` does not validate `temperature` range

**File:** `narratoai-core/src/sdp/types.rs:52-91`
**Issue:** The `validate()` function validates volume ranges, thread count, and file existence, but does not validate the `temperature` field. Extreme values (negative, > 2.0) would be passed directly to the LLM API call in `generate_sdp_script` (`sdp/script_gen.rs:209`), potentially causing API errors or unpredictable LLM behavior. Compare with `SdeRequest` which also does not validate `temperature` (same pattern, same risk).

**Fix:** Add range validation in `validate()`:

```rust
if self.temperature < 0.0 || self.temperature > 2.0 {
    return Err(format!(
        "temperature 超出有效范围 [0, 2]: {}",
        self.temperature
    ));
}
```

### WR-17: `sdp/clip.rs` duplicates `secs_to_srt_time` logic from `documentary/timestamp.rs`

**File:** `narratoai-core/src/sdp/clip.rs:112-125`
**Issue:** The local `secs_to_timestamp_str` function is functionally identical to `crate::documentary::timestamp::secs_to_srt_time` (same logic, same output format). This duplication creates a maintenance risk: if one version is fixed (e.g., for an edge case), the other may be missed. The `sdp/clip.rs` file already imports `crate::sde::timestamp::parse_srt_timestamp`, so it could also import or use `secs_to_srt_time` from the documentary module.

**Fix:** Replace the local `secs_to_timestamp_str` with the existing shared function:

```rust
use crate::documentary::timestamp::secs_to_srt_time;
// Replace all calls to secs_to_timestamp_str(x) with secs_to_srt_time(x)
```

## Info

### IN-01: `SdpRequest` fields `temperature` and `custom_clips` are unused by `run_sdp()`

**File:** `narratoai-core/src/sdp/pipeline.rs:17-97`
**Issue:** The `SdpRequest` struct contains `temperature` and `custom_clips` fields that are used by `generate_sdp_script` but never referenced in `run_sdp()`. This is by design (the same request type serves both the script generation and video processing entry points), but callers might be confused about which fields matter for which function. No code change needed -- this is a documentation/ergonomics observation.

### IN-02: Test `test_sdp_progress_step_names` is a no-op tautology

**File:** `narratoai-core/src/sdp/pipeline.rs:366-377`
**Issue:** The test `test_sdp_progress_step_names` compares string literals to themselves (e.g., `assert_eq!("clip", "clip")`). This test can never fail and does not test any actual code. It provides no value as a regression test.

**Fix:** Either remove the test or make it test actual runtime behavior (e.g., verifying that progress step name strings match the `SdpProgressStep` enum discriminant names).

---

_Reviewed: 2026-05-08T23:45:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
