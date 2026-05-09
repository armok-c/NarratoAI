---
phase: 10-tauri-command-layer
fixed_at: 2026-05-09T17:00:00Z
review_path: .planning/phases/10-tauri-command-layer/10-REVIEW.md
fix_scope: critical_warning
iteration: 3
findings_in_scope: 4
fixed: 2
skipped: 2
status: partial
---

# Phase 10: Code Review Fix Report

**Fixed at:** 2026-05-09T17:00:00Z
**Source review:** .planning/phases/10-tauri-command-layer/10-REVIEW.md
**Iteration:** 3

**Summary:**
- Findings in scope: 4 (1 Critical, 3 Warning)
- Fixed: 2
- Skipped: 2

## Fixed Issues

### CR-01: generate_sdp_script in pipeline.rs missing validate_path() call

**Files modified:** `src-tauri/src/commands/pipeline.rs`
**Commit:** 3b78519
**Applied fix:** Added `validate_path` to imports (`use crate::error::{validate_path, CommandError}`) and inserted `validate_path(&subtitle_path)?;` as the first check in `generate_sdp_script`, before the existing existence and extension checks. This closes the path traversal vulnerability that was missed in the previous iteration's fix.

### WR-03: get_script_info silently returns total_duration_secs=0.0 for clips with unparseable timestamps

**Files modified:** `src-tauri/src/commands/script.rs`, `src-tauri/src/models/progress.rs`
**Commit:** 4cde25f
**Applied fix:** Added `unparseable_clips: usize` field to `ScriptInfo` struct. Replaced the `filter_map` chain in `get_script_info` with an explicit `for` loop that counts clips with unparseable timestamps instead of silently discarding them. The caller can now detect when clips were excluded from the duration total.

## Skipped Issues

### WR-01: run_sde and generate_sdp_script hold tokio RwLock read guards across long async pipelines

**File:** `src-tauri/src/commands/pipeline.rs:117-129`
**Commit:** 84d2907 (TODO comment added)
**Reason:** Requires core API refactoring in `narratoai-core`. The core `run_sde` function accepts `&Registry` and `&PromptManager` references, making it impossible to release locks before the pipeline call without changing those public API signatures. Added TODO comment documenting the limitation.

### WR-02: generate_sdp_script holds prompt_manager read lock across the entire async LLM call chain

**File:** `src-tauri/src/commands/pipeline.rs:284-287`
**Commit:** 84d2907 (TODO comment added)
**Reason:** The core `generate_sdp_script` calls `prompt_manager.render_prompt()` at two points separated by an `.await` (LLM call). The second prompt's variables depend on the first LLM output (analysis.summary, analysis.plot_titles), so pre-rendering both upfront is not possible without splitting the core API. Added TODO comment documenting the limitation.

---

_Fixed: 2026-05-09T17:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 3_
