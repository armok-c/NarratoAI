---
phase: 10-tauri-command-layer
fixed_at: 2026-05-09T13:30:00Z
review_path: .planning/phases/10-tauri-command-layer/10-REVIEW.md
fix_scope: critical_warning
iteration: 5
findings_in_scope: 8
fixed: 5
skipped: 3
status: partial
---

# Phase 10: Code Review Fix Report (Iteration 5)

**Fixed at:** 2026-05-09
**Source review:** .planning/phases/10-tauri-command-layer/10-REVIEW.md
**Iteration:** 5

**Summary:**
- Findings in scope: 8 (2 Critical, 6 Warning)
- Fixed: 5
- Skipped: 3

## Fixed Issues

### CR-02: validate_path bypassed on Windows via absolute paths and embedded dot-dot in canonicalized form

**Files modified:** `src-tauri/src/error.rs`, `src-tauri/src/commands/export.rs`
**Commit:** e980f27
**Applied fix:**
- `error.rs`: Enhanced `validate_path` with three additional checks: (1) normalize backslashes to forward slashes before `..` check to prevent `\..\` bypass, (2) reject paths containing backslashes entirely, (3) reject Windows absolute paths (drive letter pattern `X:`).
- `export.rs`: Added `draft_name` validation at the IPC boundary, rejecting names containing `/`, `\`, `..`, or null bytes. Previously, validation only existed inside the core library's `validate_draft_name`.

### WR-01: get_script_info uses fragile timestamp parsing that splits on all `-` characters

**Files modified:** `src-tauri/src/commands/script.rs`
**Commit:** 12f3c31
**Applied fix:** Changed `ts.split('-')` to `ts.splitn(2, '-')` at line 62, limiting the split to exactly 2 parts. This matches the pattern already used in the core library's `detect_and_abort_overlaps` function and prevents malformed timestamps with extra `-` characters from being silently counted as unparseable.

### WR-02: ScriptGenRequest has #[serde(skip)] on progress but is received from IPC

**Files modified:** `narratoai-core/src/documentary/script_gen.rs`
**Commit:** c3911a9
**Applied fix:** Added design intent comment above the `#[serde(skip)]` attribute explaining that the progress callback is intentionally excluded from IPC serialization and that the Tauri command layer manually injects it post-deserialization. This documents the latent design concern so future developers understand the trade-off.

### WR-03: strip_and_repair_json trailing comma replacement is overly aggressive

**Files modified:** `narratoai-core/src/documentary/script_gen.rs`
**Commit:** c3911a9
**Applied fix:** Replaced the naive `text.replace(",}", "}").replace(",]", "]")` with a new `remove_trailing_commas()` function that performs character-level scanning with string boundary and escape sequence tracking. This ensures trailing commas are only removed at structural positions (outside of JSON string values), preventing corruption of string content containing `,}` or `,]` sequences. Added a regression test `test_json_repair_trailing_comma_in_string` that verifies a string value `"a,}b"` is preserved while a structural trailing comma in `[1, 2, 3,]` is correctly removed.

### WR-04: blocking_write called during Tauri setup could deadlock if setup is called from async context

**Files modified:** `src-tauri/src/lib.rs`
**Commit:** e2de6a0
**Applied fix:** Restructured the Registry initialization to construct and populate `Registry::new()` on a plain mutable variable first, then wrap it in `Arc<tokio::sync::RwLock>` only after registration is complete. This eliminates the `blocking_write()` call on the tokio RwLock, removing the risk of panic or deadlock when the tokio runtime thread pool is exhausted during initialization.

## Skipped Issues

### CR-03: RwLock read guards held across entire SDE pipeline execution block all concurrent users

**File:** `src-tauri/src/commands/pipeline.rs:153-154`
**Reason:** Requires core API refactoring to accept pre-extracted providers and pre-rendered prompts instead of `&Registry`/`&PromptManager` references. The `generate_documentary` command already correctly scopes its read guard; replicating this for SDE/SDP requires breaking changes to the core pipeline API. Existing TODO comments acknowledge this issue. Continued from iteration 4 skip.

### WR-05: Duplicated FFmpeg command execution code across 5 pipeline modules

**Files:** `narratoai-core/src/documentary/pipeline.rs`, `narratoai-core/src/sde/pipeline.rs`, `narratoai-core/src/sdp/pipeline.rs`
**Reason:** Large-scale refactoring that extracts a shared FFmpeg helper across 3+ modules. Risk of introducing regressions in error handling paths. Each copy has slight variations in error type wrapping. Should be addressed as a dedicated refactoring task, not as part of a code review fix cycle.

### WR-06: generate_sdp_script cleans up task_dir but run_sde and run_documentary do not

**File:** `src-tauri/src/commands/pipeline.rs:341`
**Reason:** Design issue — for `run_documentary` and `run_sde`, the output video file is written inside the task_dir (`combined.mp4`, `combined_sde.mp4`). Cleaning up the task_dir immediately would delete the output file before the caller can use it. Proper fix requires either: (1) moving/copying output to a final destination before cleanup, or (2) restructuring to separate temp intermediates from output location. This is a design change beyond the scope of a code review fix.

---

_Fixed: 2026-05-09_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 5_
