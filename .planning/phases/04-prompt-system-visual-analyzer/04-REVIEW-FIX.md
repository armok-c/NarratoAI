---
phase: 04-prompt-system-visual-analyzer
fix_date: 2026-04-30
review_path: .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
findings_in_scope: 9
fixed: 9
skipped: 0
iteration: 1
status: all_fixed
---

# Phase 4: Code Review Fix Report

**Date:** 2026-04-30
**Scope:** critical_warning
**Source:** [04-REVIEW.md](./04-REVIEW.md)
**Status:** all_fixed

## Summary

All 9 findings in scope (2 Critical + 7 Warning) have been fixed. 58 related tests pass.

## Critical Fixes

### CR-01: LLM prompt schema vs parse_and_retry deserialization type mismatch — FIXED

**Files:** `src/visual/analyzer.rs`, `src/visual/types.rs`

**Changes:**
- Introduced `BatchResponse` wrapper struct matching the LLM prompt JSON schema (`frame_observations` array + `overall_activity_summary`)
- Rewrote `parse_and_retry` to return `Vec<FrameObservation>` instead of single `FrameObservation`
- Added fallback: tries `BatchResponse` first, then single `FrameObservation` for backward compatibility
- `analyze_video_frames` now uses `.extend()` to flatten batch results into observations vector
- Removed duplicated `strip_code_fence` logic, now calls `crate::visual::types::strip_code_fence`
- Removed `#[allow(dead_code)]` from `strip_code_fence` in `types.rs` (now used)
- Updated all tests to match new API

### CR-02: Truncate filter panics on multi-byte UTF-8 characters — FIXED

**File:** `src/prompt/template.rs`

**Changes:**
- Replaced byte-based `s.len()` / `&s[..97]` with char-based `s.chars().count()` / `s.chars().take(97).collect()`
- Added `test_truncate_filter_chinese` test with 120 Chinese characters

## Warning Fixes

### WR-01: Missing filter variable silently ignored — FIXED

**File:** `src/prompt/template.rs`

**Changes:**
- Added validation pass before filter application that checks all filter-referenced variables exist in `vars`
- Returns `PromptError::TemplateRender` with missing variable names, consistent with pass 1 behavior
- Changed `filter_re` from `unwrap()` to `.map_err()` for consistency

### WR-02: Dead retry loop in parse_and_retry — FIXED

**File:** `src/visual/analyzer.rs`

**Changes:**
- Removed the meaningless retry loop (identical input each iteration)
- Simplified to single-pass: try `BatchResponse`, fallback to `FrameObservation`, else error

### WR-03: RwLock poisoning silently recovered via into_inner() — FIXED

**File:** `src/prompt/manager.rs`

**Changes:**
- Changed `search_prompts`, `list_categories`, `list_prompts` return types to `Result<Vec<...>, PromptError>`
- All three now use `.map_err()` for lock poisoning, consistent with `get_prompt` and `register_prompt`
- Removed unused `use std::sync::RwLock` import
- Updated all test callers to use `.unwrap()`

### WR-04: interval_seconds=0 causes near-infinite loop — FIXED

**File:** `src/visual/frame_extractor.rs`

**Changes:**
- Added early validation at `extract_frames` entry: `if interval_seconds <= 0.0` returns `VisualError::FrameExtraction`

### WR-05: has_chinese detection uses > instead of >= — FIXED

**File:** `src/prompt/validators.rs`

**Changes:**
- Changed `c > '\u{4E00}'` to `c >= '\u{4E00}' && c <= '\u{9FFF}'`
- Added CJK Extension A range (`\u{3400}`..`\u{4DBF}`) and Compatibility Ideographs range (`\u{F900}`..`\u{FAFF}`)

### WR-06: strip_code_fence dead code duplication — FIXED

**Files:** `src/visual/types.rs`, `src/visual/analyzer.rs`

**Changes:**
- `parse_and_retry` now calls `crate::visual::types::strip_code_fence` instead of inlining
- Removed `#[allow(dead_code)]` from `strip_code_fence` in `types.rs`

### WR-07: Documented $ escape not implemented — FIXED

**File:** `src/prompt/template.rs`

**Changes:**
- Removed the false doc claim "supports `\$` escape to literal `$`"

## Files Modified

| File | Findings Fixed |
|------|---------------|
| `src/visual/analyzer.rs` | CR-01, WR-02, WR-06 |
| `src/visual/types.rs` | WR-06 |
| `src/visual/frame_extractor.rs` | WR-04 |
| `src/prompt/template.rs` | CR-02, WR-01, WR-07 |
| `src/prompt/manager.rs` | WR-03 |
| `src/prompt/validators.rs` | WR-05 |

## Test Results

All 58 related tests pass:
- `prompt::template`: 15 passed
- `prompt::manager`: 11 passed
- `prompt::validators`: 14 passed
- `visual::analyzer`: 9 passed
- `visual::frame_extractor`: 9 passed

## Skipped (Info-level, out of scope)

| ID | Description | Reason |
|----|------------|--------|
| IN-01 | Byte length vs char count in validators | Info level, not in fix scope |
| IN-02 | visual_salience range validation | Info level, not in fix scope |
| IN-03 | Hardcoded quality 85 in convert_image_to_jpeg | Info level, not in fix scope |
| IN-04 | unwrap() on static regex consistency | Info level, not in fix scope |
