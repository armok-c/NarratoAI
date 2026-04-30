---
phase: 04-prompt-system-visual-analyzer
reviewed: 2026-04-30T16:00:00Z
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
  info: 2
  total: 3
status: issues_found
---

# Phase 4: Code Review Report (Re-Review #3)

**Reviewed:** 2026-04-30T16:00:00Z
**Depth:** standard
**Files Reviewed:** 19
**Status:** issues_found

## Summary

Re-review iteration 4 (after fixes from re-review #2). All 3 previously actionable findings have been correctly applied:

| Previous Finding | Status | Verification |
|---|---|---|
| CR-01: Byte-index slice on UTF-8 string | **Fixed** | `truncate_str` helper at line 271 uses `char_indices().nth()` for safe char-based truncation. Used at line 218 in `parse_and_retry` error logging. |
| WR-01: `ParsedBatch` and `parse_and_retry` were `pub` | **Fixed** | Both are now private (no `pub` qualifier). No external references exist. |
| WR-02: Unused `_context` parameter | **Fixed** | Parameter removed from `analyze_video_frames` signature. `HashMap` import also removed. |

The two info-level findings from the previous review (IN-01: `visual_salience` range; IN-02: `BatchResponse` leniency) remain as documented design decisions and are not re-flagged.

After thorough re-examination of all 19 files at standard depth (full file reads, import/usage cross-referencing, edge case analysis), one new warning and two info items were identified. No critical issues found.

## Critical Issues

None.

## Warnings

### WR-01: `frame_data` declared as required parameter but never referenced in template

**File:** `src/prompt/register.rs:43` + `src/prompt/templates/documentary/frame_analysis_v1.0.md`
**Issue:** The `frame_analysis` prompt declares `frame_data` as `required: true` in its `ParameterDef` metadata (register.rs line 43), but the template content (`frame_analysis_v1.0.md`) only contains `${video_description}` and `${language}` placeholders. There is no `${frame_data}` in the template text.

This creates a metadata/content inconsistency: any system that validates input against the metadata schema will require callers to supply `frame_data`, but the template engine (`template::render`) only checks variables that actually appear in the template text. Conversely, if `frame_data` is supplied, it is silently ignored by the renderer. The parameter appears to be a leftover from when frame data was intended to be injected via template rather than passed as image paths to `analyze_images`.

**Fix:** Remove the unused parameter from the metadata since frame data is passed via the image path mechanism in `analyze_video_frames`:
```rust
parameters: vec![
    ParameterDef { name: "video_description".into(), required: true, default: None, description: "视频内容摘要".into() },
    ParameterDef { name: "language".into(), required: false, default: Some("zh-CN".into()), description: "输出语言".into() },
    // frame_data removed -- frame images are passed via LlmProvider::analyze_images, not template vars
],
```

## Info

### IN-01: `extract_frames_fast_path` is unnecessarily `pub`

**File:** `src/visual/frame_extractor.rs:97`
**Issue:** `extract_frames_fast_path` is declared `pub async fn` but is only called internally by `extract_frames()` at line 54 of the same file. No other module in the crate references it. Exposing it as `pub` leaks an internal implementation detail of the fast-path/fallback strategy.
**Fix:** Change visibility to `async fn` (private) or `pub(crate) async fn` if cross-module access is anticipated.

### IN-02: `seconds_to_hhmmssmmm` and `parse_frame_number_from_name` are `pub(crate)` but unused externally

**File:** `src/visual/frame_extractor.rs:449,541`
**Issue:** Both helper functions are marked `pub(crate)` but are never referenced outside `frame_extractor.rs`. The `#[allow(dead_code)]` on `parse_frame_number_from_name` further indicates it has no current consumers. These can remain as-is if a Phase 5+ consumer is planned, but currently the visibility exceeds usage.
**Fix:** Downgrade to private `fn` if no external consumer is planned, or leave as-is with a comment noting the anticipated future usage.

---

_Reviewed: 2026-04-30T16:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 4 (re-review #3)_
