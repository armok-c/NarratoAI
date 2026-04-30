---
phase: 04-prompt-system-visual-analyzer
reviewed: 2026-04-30T12:00:00Z
depth: standard
files_reviewed: 19
files_reviewed_list:
  - Cargo.toml
  - src/lib.rs
  - src/prompt/mod.rs
  - src/prompt/types.rs
  - src/prompt/error.rs
  - src/prompt/registry.rs
  - src/prompt/template.rs
  - src/prompt/manager.rs
  - src/prompt/validators.rs
  - src/prompt/register.rs
  - src/prompt/templates/documentary/frame_analysis_v1.0.md
  - src/prompt/templates/documentary/narration_generation_v2.0.md
  - src/prompt/templates/short_drama_editing/plot_extraction_v2.0.md
  - src/prompt/templates/short_drama_narration/script_generation_v1.0.md
  - src/visual/mod.rs
  - src/visual/error.rs
  - src/visual/types.rs
  - src/visual/frame_extractor.rs
  - src/visual/analyzer.rs
findings:
  critical: 0
  warning: 3
  info: 5
  total: 8
status: issues_found
---

# Phase 4: Code Review Report (Re-review #8, Iteration 9)

**Reviewed:** 2026-04-30T12:00:00Z
**Depth:** standard
**Files Reviewed:** 19
**Status:** issues_found

## Summary

Re-review #8 of Phase 04 covering the prompt system (types, error, registry, template, manager, validators, register) and visual analyzer (error, types, frame_extractor, analyzer) subsystems. All 273 library tests pass.

**Previous fix verification (iteration 8 -> 9):**
- IN-01 (PromptError::Version dead code): Verified -- `grep` for `PromptError::Version` returns zero matches across `src/`. Fix confirmed.
- IN-02 (total_frames used extract count): Verified -- line 185 of `analyzer.rs` now reads `total_frames: frame_paths.len()`. Fix confirmed.
- IN-03 (missing LockFailure test): Verified -- `error.rs` lines 48-53 contain `test_lock_failure_error_message_chinese` test. Fix confirmed.

All three previous fixes are correctly applied.

**New findings:** 0 critical, 3 warnings, 5 info items. The warnings involve unvalidated `quality` parameter bounds in frame extraction, an `extract_frames_fast_path` function that is `pub` but only used internally, and an incorrect `analyzed_count` value (always 0) in the `BatchPartial` error path.

## Warnings

### WR-01: Unvalidated `quality` parameter passed directly to FFmpeg `-q:v`

**File:** `src/visual/frame_extractor.rs:41`
**Issue:** The `quality` parameter (`Option<u32>`) is accepted with `unwrap_or(5)` but never validated against the FFmpeg `-q:v` valid range (2-31). A caller passing `quality: Some(0)` or `quality: Some(999)` will produce a malformed FFmpeg command. While `extract_single_frame` calls `convert_image_to_jpeg` which does `ffmpeg_quality.clamp(2, 31)` for the image crate conversion, the direct FFmpeg `-q:v` argument at line 128 (`&quality.to_string()`) and lines 279, 298 is unclamped.

Note: `convert_image_to_jpeg` at line 528 correctly clamps the quality, but the primary FFmpeg JPEG extraction paths (levels 1 and 2) do not.

**Fix:** Add validation at the entry point of `extract_frames`:
```rust
let quality_val = quality.unwrap_or(5);
if quality_val < 2 || quality_val > 31 {
    return Err(VisualError::FrameExtraction(
        format!("JPEG quality 值 ({}) 超出有效范围 2-31", quality_val)
    ));
}
```

### WR-02: `extract_frames_fast_path` is `pub` but only called internally

**File:** `src/visual/frame_extractor.rs:97`
**Issue:** The function `extract_frames_fast_path` is declared `pub async fn` but is only called from `extract_frames` (line 54) within the same module. No external crate or module references it. Making it `pub` exposes an internal implementation detail as part of the module's public API, increasing the maintenance surface and making it harder to change the fast-path/fallback strategy later.

Similarly, `parse_frame_number_from_name` (line 541) is `pub(crate)` with `#[allow(dead_code)]` and is never used outside its own test block.

**Fix:** Change visibility to `pub(crate)` or remove `pub` entirely:
```rust
pub(crate) async fn extract_frames_fast_path(
```
And remove `#[allow(dead_code)]` from `parse_frame_number_from_name` if it remains unused, or remove the function entirely.

### WR-03: `analyzed_count` always 0 in `VisualError::BatchPartial`

**File:** `src/visual/analyzer.rs:162-166`
**Issue:** When the "empty result barrier" triggers (all batches failed, zero observations), the code returns:
```rust
VisualError::BatchPartial {
    analyzed_count: 0,
    total_count: raw_results.len(),
    errors,
}
```
Since all batches failed, `analyzed_count` is indeed 0. However, the error message format `"已分析 {analyzed_count}/{total_count} 批次"` (from `error.rs` line 15) will always read `"已分析 0/N 批次"`. This is technically correct but misleading -- the `BatchPartial` variant is described as "部分批次失败" (partial batch failure), yet this code path represents a total failure, not a partial one. A consumer distinguishing between "some succeeded" vs "all failed" cannot do so from this variant alone.

**Fix:** Consider tracking the actual number of successful batches and using it:
```rust
let success_count = raw_results.len() - errors.len();
if observations.is_empty() && !errors.is_empty() {
    return Err(VisualError::BatchPartial {
        analyzed_count: success_count, // or use a separate TotalFailure variant
        total_count: raw_results.len(),
        errors,
    });
}
```

## Info

### IN-01: `strip_code_fence` does not handle whitespace between ``` and json

**File:** `src/visual/types.rs:47`
**Issue:** The function checks `strip_prefix("```json")` but does not handle the case where the LLM returns `` ``` json `` (with a space). This is a less common but possible LLM output variation. The current code would fall through to the plain `` ``` `` strip, which would then leave `json ...` as part of the content, causing a JSON parse failure.

**Fix:** Consider normalizing whitespace:
```rust
pub(crate) fn strip_code_fence(text: &str) -> &str {
    let trimmed = text.trim();
    trimmed
        .strip_prefix("```")
        .map(|rest| {
            // Skip optional language tag (json, etc.)
            let rest = rest.strip_prefix("json").unwrap_or(rest);
            rest.trim().trim_end_matches("```").trim()
        })
        .unwrap_or(trimmed)
}
```
Alternatively, use a regex or match the language tag more loosely.

### IN-02: Missing `interval_seconds.is_infinite()` check

**File:** `src/visual/frame_extractor.rs:43`
**Issue:** The validation checks `is_nan() || <= 0.0` but does not check `is_infinite()`. If `interval_seconds` is `f64::INFINITY`, it passes validation. The fast path would generate `fps=1/inf` which FFmpeg would reject, causing a silent fallback to the per-frame path. In the fallback path, `(duration / f64::INFINITY).ceil()` would produce 0.0, and the function would return `Ok(0)` silently. Not a crash, but a confusing silent no-op.

**Fix:**
```rust
if interval_seconds.is_nan() || interval_seconds.is_infinite() || interval_seconds <= 0.0 {
    return Err(VisualError::FrameExtraction("帧提取间隔必须为有限正数".into()));
}
```

### IN-03: Stale intermediate files left on fallback level transition

**File:** `src/visual/frame_extractor.rs:268-369`
**Issue:** In `extract_single_frame`, when Level 1 (HW JPEG) writes to `output_path` but produces an invalid file, and Level 2 also fails, the stale/empty `output_path` file from Level 1/2 is left on disk. Levels 3 and 4 write to separate `.png`/`.bmp` paths and only convert to `output_path` on success. But if Level 3's `run_ffmpeg` succeeds but `convert_image_to_jpeg` fails, the stale `output_path` from Level 1/2 remains. Similarly if Level 4 fails after Level 3's conversion failed. The stale JPEG at `output_path` (0 bytes or corrupted from Level 1/2) is never cleaned up before the final `Err` return.

This is unlikely to cause real-world issues since `file_is_valid` checks would catch it, and the caller (`extract_frames_fallback`) does not re-use failed frame paths, but it leaves orphan files in the output directory.

**Fix:** Add cleanup before returning the final error:
```rust
// Before the final error return
let _ = std::fs::remove_file(output_path);
Err(VisualError::FrameExtraction(...))
```

### IN-04: `#[allow(dead_code)]` on `seconds_to_hhmmssmmm` is unnecessary

**File:** `src/visual/frame_extractor.rs:448`
**Issue:** The function `seconds_to_hhmmssmmm` has `#[allow(dead_code)]` but is used at lines 212, 430, 576, and 602. The `#[allow(dead_code)]` annotation is stale and should be removed.

**Fix:** Remove `#[allow(dead_code)]` from line 448.

### IN-05: `truncate` filter test uses byte indexing on potentially multi-byte strings

**File:** `src/prompt/template.rs:253`
**Issue:** The assertion `&result[..97] == &long_text[..97]` works only because `long_text` is `"A".repeat(150)` (pure ASCII). However, this test pattern could mislead future developers into using byte slicing on multi-byte strings. The `truncate` filter itself correctly uses `.chars().take(97)`, but the test assertion couples to byte boundaries.

**Fix:** Use character-based comparison instead:
```rust
let result_chars: String = result.chars().take(97).collect();
let expected_chars: String = long_text.chars().take(97).collect();
assert_eq!(result_chars, expected_chars);
```

---

_Reviewed: 2026-04-30T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Re-review: #8, iteration 9_
