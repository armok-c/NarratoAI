---
phase: 04-prompt-system-visual-analyzer
reviewed: 2026-04-30T14:30:00Z
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
  warning: 1
  info: 3
  total: 4
status: issues_found
---

# Phase 4: Code Review Report (Re-review #6, Iteration 7)

**Reviewed:** 2026-04-30T14:30:00Z
**Depth:** standard
**Files Reviewed:** 19
**Status:** issues_found

## Summary

Re-review iteration 7 (re-review #6). All three previously reported fix issues from the last round are verified as correctly applied with no regressions:

- **WR-01 (NaN check):** `frame_extractor.rs:43` now has `interval_seconds.is_nan() || interval_seconds <= 0.0` guard.
- **WR-02 (LockFailure):** `error.rs:17` defines `LockFailure(String)` variant; all five `manager.rs` sites (lines 38, 83, 91, 110, 118) use it instead of `TemplateRender`.
- **WR-03 (deny_unknown_fields):** `analyzer.rs:29` `BatchResponse` now has `#[serde(deny_unknown_fields)]`.

No new critical or security issues found. One new warning identified. Three info-level items carried forward or newly noted.

## Previous Fixes Verification

| Previous Issue | Status | Verification |
|---|---|---|
| WR-01: NaN interval_seconds bypasses validation | Verified fixed | `frame_extractor.rs:43` checks `is_nan()` before `<= 0.0` |
| WR-02: Misleading error variant for lock poisoning | Verified fixed | `error.rs:17` has `LockFailure`; `manager.rs:38,83,91,110,118` all use it |
| WR-03: BatchResponse missing deny_unknown_fields | Verified fixed | `analyzer.rs:29` has `#[serde(deny_unknown_fields)]` |
| IN-01: `PromptError::Version` dead code | Not fixed (info-level) | Still only used in test at `error.rs:67` |
| IN-02: `total_frames` from extract count | Not fixed (info-level) | Still uses `frame_count` at `analyzer.rs:185` |

## Warnings

### WR-01: json filter fallback produces malformed JSON for strings containing quotes/backslashes

**File:** `src/prompt/template.rs:53-55`
**Issue:** The `json` filter's fallback path `format!("\"{}\"", s)` wraps the raw string in double quotes without escaping internal quotes, backslashes, or control characters. If `serde_json::to_string(s)` were to fail and the fallback were triggered, a string like `hello "world"` would produce `"hello "world""` -- invalid JSON. In practice, `serde_json::to_string(&str)` never fails for any valid UTF-8 string (it always produces a valid JSON string), so the fallback is unreachable dead code. However, the fallback encodes an incorrect assumption that could mislead future maintainers who may assume it is reachable and correct.
**Fix:** Remove the unreachable fallback and use `expect()` with a clear message, or keep the fallback but make it correct:
```rust
// Option A: Remove unreachable fallback
m.insert("json", |s: &str| {
    serde_json::to_string(s).expect("serde_json cannot fail serializing &str")
});

// Option B: Correct the fallback (if defensive coding is preferred)
m.insert("json", |s: &str| {
    serde_json::to_string(s).unwrap_or_else(|_| {
        // Manual JSON escaping as last resort
        let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
        format!("\"{}\"", escaped)
    })
});
```

## Info

### IN-01: PromptError::Version variant is dead code (carried from iteration 5)

**File:** `src/prompt/error.rs:22-23`
**Issue:** The `Version(String)` variant is defined and tested but never constructed in non-test code. It adds unused API surface.
**Fix:** Remove the variant or add a version validation function that uses it.

### IN-02: total_frames uses extract_frames count, not actual file count (carried from iteration 5)

**File:** `src/visual/analyzer.rs:185`
**Issue:** `BatchAnalysisResult.total_frames` is set to `frame_count as usize` (returned by `extract_frames`), not the actual number of files collected by `collect_frame_paths`. If some frames were corrupted or deleted between extraction and collection, `total_frames` would overcount.
**Fix:**
```rust
Ok(BatchAnalysisResult {
    observations,
    overall_activity_summary: last_summary,
    total_frames: frame_paths.len(),  // Use actual collected count
    analyzed_batches: raw_results.len(),
    errors,
})
```

### IN-03: Missing test coverage for PromptError::LockFailure variant

**File:** `src/prompt/error.rs:17`
**Issue:** Every other variant in `PromptError` has a corresponding unit test in `error.rs` verifying its Chinese error message, but the newly added `LockFailure(String)` variant (added in WR-02 fix) has no test. This is an inconsistency in test coverage.
**Fix:** Add a test analogous to the existing ones:
```rust
#[test]
fn test_lock_failure_error_message_chinese() {
    let err = PromptError::LockFailure("read lock poisoned".into());
    let msg = err.to_string();
    assert!(msg.contains("注册中心锁失败"), "消息应包含中文: {}", msg);
}
```

---

_Reviewed: 2026-04-30T14:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 7 (re-review #6)_
