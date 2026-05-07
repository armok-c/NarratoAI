---
phase: 04-prompt-system-visual-analyzer
fixed_at: 2026-05-07T16:30:00Z
review_path: .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
iteration: 3
findings_in_scope: 5
fixed: 4
skipped: 1
status: partial
---

# Phase 4: Code Review Fix Report

**Fixed at:** 2026-05-07T16:30:00Z
**Source review:** .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
**Iteration:** 3

**Summary:**
- Findings in scope: 5 (1 CRITICAL + 4 WARNING)
- Fixed: 4
- Skipped: 1

## Fixed Issues

### CR-01: `strip_code_fence` fails to strip closing fence when JSON abuts ` ``` ` without trailing whitespace

**Files modified:** `src/visual/types.rs`
**Commit:** 4c7c8b3
**Applied fix:** Removed the flawed conditional in the `strip_suffix("```")` branch that returned the unstripped `content` when the remainder had no trailing whitespace. Simplified to always `trim_end()` the stripped result and return it. Verified with all 10 existing `visual::types` tests passing.

### WR-01: O(n^2) duplicate detection in template variable validation

**Files modified:** `src/prompt/template.rs`
**Commit:** 40df4cb
**Applied fix:** Replaced `Vec<String>` with `HashSet<String>` for both `missing` and `missing_filter_vars` collections. Removed the O(n) `.contains()` dedup scan. Added sorting before error message formatting for deterministic output. All 15 template tests pass.

### WR-02: Empty-result blind spot -- zero observations with zero errors succeeds silently

**Files modified:** `src/visual/analyzer.rs`
**Commit:** e790735
**Applied fix:** Added Step 8b check after the existing empty-result barrier: when `observations.is_empty()` (regardless of errors), return `VisualError::Analysis("所有批次返回空观察结果")`. This catches the case where all LLM batches succeed but return empty `frame_observations` arrays.

### WR-03: `validate_json` does not strip code fences before parsing

**Files modified:** `src/prompt/validators.rs`
**Commit:** f0db2ed
**Applied fix:** Added `crate::visual::types::strip_code_fence(output)` call at the start of `validate_json()`, before trimming and parsing. This ensures LLM responses wrapped in ```json...``` code fences pass JSON validation. All 14 validator tests pass.

## Skipped Issues

### WR-04: Fast-path FFmpeg event loop blocks indefinitely on stalled FFmpeg

**File:** `src/visual/frame_extractor.rs:157-166`
**Reason:** No code change required -- limitation is already documented in code comments (lines 153-156). REVIEW.md explicitly states "No code change required (limitation is documented)."
**Original issue:** The fast path iterates `child.iter()` inside a for loop with cancellation only checked when a new event arrives. If FFmpeg stalls, the thread blocks without checking the cancellation token.

## Verification

All fixes verified with:
1. Tier 1: Re-read modified file sections to confirm changes present and surrounding code intact
2. Tier 2: `cargo check` passed after each fix (0 new errors)
3. Targeted `cargo test --lib` for each modified module: all tests pass (visual::types: 10/10, prompt::template: 15/15, prompt::validators: 14/14)

---

_Fixed: 2026-05-07T16:30:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 3_
