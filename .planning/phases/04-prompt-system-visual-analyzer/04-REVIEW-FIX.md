---
phase: 04-prompt-system-visual-analyzer
fixed_at: 2026-05-07T10:50:00Z
review_path: .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
iteration: 18
findings_in_scope: 7
fixed: 5
skipped: 2
status: partial
---

# Phase 04: Code Review Fix Report (Iteration 18)

**Fixed at:** 2026-05-07T10:50:00Z
**Source review:** .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
**Iteration:** 18

**Summary:**
- Fix scope: critical + warning
- Findings in scope: 7 (2 CRITICAL, 5 WARNING)
- Fixed: 5
- Skipped: 2 (1 false positive, 1 already covered by another fix)

## Fixed Issues

### CR-02: `$variable` regex false-matches dollar signs in template content

**Files modified:** `src/prompt/template.rs`
**Commit:** 5f8b598 (worktree), 95fc3ec (main)
**Applied fix:** Removed the bare `\$(\w+)` branch from the variable regex, keeping only `\$\{(\w+)\}`. Updated doc comments to document single-syntax support and added safety notice about caller responsibility for variable value validation. Updated 2 unit tests that used bare `$variable` syntax to use `${variable}`.

### WR-01: No validation of `batch_size`/`max_concurrency` inputs

**Files modified:** `src/visual/analyzer.rs`
**Commit:** 45f810e (worktree), fc56734 (main)
**Applied fix:** Added input validation at the top of `analyze_video_frames` body (after Step 1 progress callback, before Step 2 frame extraction). Returns `VisualError::Analysis` with Chinese error message for zero values.

### WR-02: `analyzed_batches` misleading count

**Files modified:** `src/visual/analyzer.rs`
**Commit:** cac8935 (worktree), 641c7b1 (main)
**Applied fix:** Changed `analyzed_batches` value from `raw_results.len()` to `raw_results.len() - errors.len()` so the field only counts successfully analyzed batches.

### WR-04: RwLock reentrancy documentation

**Files modified:** `src/prompt/manager.rs`
**Commit:** 9cd8f37 (worktree), 43f297f (main)
**Applied fix:** Added `# 线程安全` section to `PromptManager` struct doc comment warning that methods must not be called reentrantly from the same thread due to `std::sync::RwLock` deadlock risk.

### WR-05: Dead code `parse_frame_number_from_name`

**Files modified:** `src/visual/frame_extractor.rs`
**Commit:** 8718b0b (worktree), 50767a7 (main)
**Applied fix:** Refactored `rename_fast_path_frames` to call `parse_frame_number_from_name` instead of duplicating inline parsing logic. Removed `#[allow(dead_code)]` annotation since the function is now used in production code.

## Skipped Issues

### CR-01: `format!` macro will panic if prompt_template contains `{` or `}` characters

**File:** `src/visual/analyzer.rs:109-116`
**Reason:** False positive. `format!("{}", prompt_template)` is safe -- `prompt_template` is the value parameter (second argument), not the format string. Rust's `format!` macro parses the format string literal (first argument) at compile time. The `Display` trait for `&str` outputs the string as-is. Content like `{`, `}`, `{:?}` in `prompt_template` will NOT cause panics. This was confirmed in previous fix iteration 17.
**Original issue:** The `format!` call was reported as potentially panicking if `prompt_template` contains literal braces.

### WR-03: Template injection -- user-supplied values not sanitized

**File:** `src/prompt/template.rs:75-108`
**Reason:** Already covered by CR-02 fix. The safety documentation (`# 安全性` section) was added as part of the CR-02 commit (5f8b598/95fc3ec), which included the doc comment: "调用方负责验证变量值的合法性；本函数不执行任何清理或转义。"
**Original issue:** The template renderer performs pure string substitution without sanitization of variable values.

## Verification

All fixes verified with:
1. Tier 1: Re-read modified file sections to confirm changes present and surrounding code intact
2. Tier 2: `cargo check --lib` passed with 0 new warnings (7 pre-existing warnings unrelated to changes)
3. Template tests: All 15 `prompt::template` tests passed after CR-02 fix

---
_Fixed: 2026-05-07T10:50:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 18_
