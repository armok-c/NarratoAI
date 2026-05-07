---
phase: 04-prompt-system-visual-analyzer
fixed_at: '2026-05-07T00:00:00Z'
review_path: .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
iteration: 1
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 04: Code Review Fix Report

**Fixed at:** 2026-05-07
**Source review:** .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 3
- Fixed: 3
- Skipped: 0

## Fixed Issues

### WR-01: `validate_prompt_parameters` regex misses filter syntax

**Files modified:** `narratoai-core/src/prompt/registry.rs`
**Commit:** 7f102ec
**Applied fix:** Updated the regex in `validate_prompt_parameters` from `r"\$\{(\w+)\}"` to `r"\$\{(\w+)(?:\|(\w+))?\}"` to match `${variable|filter}` syntax. The capture group extraction loop already used `caps.get(1)` for the variable name, so no loop changes were needed. Templates using undeclared variables with filters (e.g., `${undeclared_var|upper}`) will now be caught at registration time with a proper `Validation` error instead of surfacing at render time as a `TemplateRender` error.

### WR-02: `map_err` in `analyze_video_frames` double-wraps `FrameExtraction` error

**Files modified:** `narratoai-core/src/visual/analyzer.rs`
**Commit:** 315057e
**Applied fix:** Changed the `map_err` handler in the `extract_frames` call to return the original error `e` after logging, instead of wrapping it in another `VisualError::FrameExtraction`. This eliminates the tripled Chinese prefix in error messages (`帧提取失败: 帧提取失败: 帧提取失败: ...` becomes `帧提取失败: ...`).

### WR-03: Cancel token consumed by `extract_frames`, unavailable for LLM analysis

**Files modified:** `narratoai-core/src/visual/analyzer.rs`
**Commit:** 315057e
**Applied fix:** Added `let cancel_after_extract = cancel.clone()` before the `extract_frames` call so the cancellation token is preserved for later use. Passed the original `cancel` into `extract_frames`. Added a pre-flight cancellation check after `extract_frames` returns and before the LLM analysis phase begins. If a cancel signal was received during frame extraction, the function now short-circuits with `VisualError::Analysis("分析被取消".into())` instead of proceeding into the potentially long-running LLM API calls.

## Skipped Issues

(No skipped issues.)

---

_Fixed: 2026-05-07_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
