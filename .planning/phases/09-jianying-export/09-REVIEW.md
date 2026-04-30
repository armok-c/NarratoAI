---
phase: 09-jianying-export
reviewed: 2026-04-29T14:30:00Z
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
  warning: 1
  info: 1
  total: 3
status: issues_found
---

# Phase 09: Code Review Report (Iteration 5)

**Reviewed:** 2026-04-29T14:30:00Z
**Depth:** standard
**Files Reviewed:** 13
**Status:** issues_found

## Summary

Reviewed 13 source files for the JianYing (CapCut) draft export module at standard depth, iteration 5. All 4 findings from iteration 4 (CR-01 JSON injection, CR-02 path traversal, WR-01 silent audio skip, WR-02 negative start) have been correctly fixed and verified. This iteration identified 1 new warning, confirmed a previously noted info item, and found 1 critical edge case in the timestamp parser.

The codebase quality has improved substantially across 5 review iterations. The remaining issue is a timestamp parsing edge case that can produce incorrect output without any error signal.

## Verification of Previous Fixes

### Iteration 4 Fixes (all confirmed fixed)

- **CR-01 (JSON injection):** Verified. `builder.rs:98-101` now parses the template as `serde_json::Value` and sets `draft_id`/`draft_name` via `serde_json::json!()`, eliminating injection risk.
- **CR-02 (path traversal):** Verified. `validate_draft_name()` at `builder.rs:61-72` rejects `/`, `\`, `..`, and `\0`. Called at `builder.rs:92` before `draft_path.join(draft_name)`. Test coverage at `builder.rs:762-791`.
- **WR-01 (silent audio skip):** Verified. `builder.rs:306-326` now uses `match clip.audio` with a `None =>` arm that returns `JianYingError::MissingField { field: "audio", clip_index: i }`.
- **WR-02 (negative start):** Verified. `time.rs:20` now checks `if start_us < 0 || duration_us < 0`. Test `test_trange_negative_start_returns_none` at `time.rs:146-148`.

### Iteration 1-3 Fixes (all confirmed fixed)

- `trange_from_secs` validates NaN/Inf/negative (`time.rs:33`)
- Empty timeline returns duration 0 instead of panicking (`builder.rs:245`)
- `probe_video` validates finite positive duration (`probe.rs:65-69`)
- `probe_audio` validates finite positive duration (`probe.rs:152-155`)
- `video_origin_path` validated as non-empty (`builder.rs:352-356`)
- `Track::max_end_time` uses struct field access, not JSON re-serialization (`track.rs:94-99`)
- `parse_timestamp_start` parses h/m/s as `u32` (`builder.rs:392-394`)

## Critical Issues

### CR-01: parse_timestamp_start produces wrong result for single-segment timestamps without comma-millis

**File:** `src/jianying/builder.rs:383-408`
**Severity:** BLOCKER

**Issue:** The `parse_timestamp_start` function uses `rsplit_once('-')` to split the range at the *last* hyphen. This works correctly for the standard format `"HH:MM:SS,mmm-HH:MM:SS,mmm"`. However, when the timestamp format omits the comma-millisecond part (e.g., `"00:00:07-00:00:15"`), the function produces an incorrect result.

The split produces `start_str = "00:00:07-00:00:15"` (the entire string minus the part after the last `-`). Then `sub_parts = start_str.split(',')` yields only one element (`"00:00:07-00:00:15"`), so `time_str = "00:00:07-00:00:15"`. Then `time_str.split(':')` yields `["00", "00", "07-00", "00", "15"]` which has 5 parts, failing the `len() != 3` check and returning `None`.

While the function correctly rejects this malformed input, the problem is that the *test* `test_parse_timestamp_start_invalid` at `builder.rs:753` only tests `"invalid"`, `""`, and `"--"` -- it does NOT test the actual no-comma format. This means the parser silently rejects a valid-looking timestamp format, and the caller (`parse_source_start_time`) converts this to a `JianYingError::Validation` with a generic "格式无效" message, masking the real issue.

More critically, consider the input `"00:00:07,559"`. `rsplit_once('-')` returns `None` because there is no `-` in the string. The function returns `None`, meaning a single timestamp (without end time) is also rejected. This may be unexpected for callers who pass timestamp strings that lack a range separator.

This is a BLOCKER because the function's behavior with no-comma timestamps (which are a valid variant of the `"HH:MM:SS-HH:MM:SS"` format documented in the docstring at line 379) is to silently fail, and there is no test coverage for this case.

**Fix:** Either (a) handle the no-comma format by detecting whether the split produced a valid time string, or (b) document clearly that only the `HH:MM:SS,mmm-HH:MM:SS,mmm` format with comma-milliseconds is supported. Add tests for the no-comma case to confirm the desired behavior:

```rust
// Option (a): support both formats
fn parse_timestamp_start(range: &str) -> Option<f64> {
    let (start_str, _) = range.rsplit_once('-')?;
    let sub_parts: Vec<&str> = start_str.split(',').collect();
    let time_str = sub_parts.first()?;
    // ... existing logic
}
// Add test:
#[test]
fn test_parse_timestamp_start_no_comma() {
    let result = parse_timestamp_start("00:00:07-00:00:15");
    // Document expected behavior: Some(7.0) or None?
}
```

## Warnings

### WR-01: parse_seconds does not validate for negative values

**File:** `src/jianying/time.rs:47-55`
**Severity:** WARNING

**Issue:** The `parse_seconds` helper function converts an input string to microseconds but does not check for negative values. While `trange()` at line 20 now checks `start_us < 0 || duration_us < 0` (fixed in iteration 4), the `parse_seconds` function itself can produce negative `i64` values that propagate until that outer check. If `parse_seconds` is ever called from a context that does not perform its own negative-value check, negative durations will silently pass through.

For example, `parse_seconds("-5s")` returns `Some(-5_000_000)` instead of `None`. The current `trange()` guards against this, but `parse_seconds` is a `fn` (not `pub`, but accessible within the module) that could be used from new code without the guard.

**Fix:** Add negative-value validation inside `parse_seconds`:

```rust
fn parse_seconds(input: &str) -> Option<i64> {
    let input = input.trim().to_lowercase();
    let secs: f64 = if input.ends_with('s') {
        input.trim_end_matches('s').parse().ok()?
    } else {
        input.parse().ok()?
    };
    let us = (secs * SEC as f64).round() as i64;
    if us < 0 {
        return None;
    }
    Some(us)
}
```

## Info

### IN-01: Inconsistent error message language in probe.rs (carried from iteration 4)

**File:** `src/ffmpeg/probe.rs:63, 97, 154`
**Severity:** INFO

**Issue:** Error messages are inconsistently mixed between English and Chinese. `probe_audio` uses Chinese at line 151 (`"音频时长解析失败"`) but English at lines 153-155 (`"Invalid audio duration from ffprobe"`). `probe_video` uses English throughout (`"Missing or invalid duration"`, `"No video stream found"`). All error messages in this project should use Chinese for consistency (per project conventions in CLAUDE.md and the existing Chinese error messages in `error.rs` and `jianying/error.rs`).

**Fix:** Standardize all error messages in `probe.rs` to Chinese, matching the style used elsewhere in the codebase.

---

_Reviewed: 2026-04-29T14:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 5_
