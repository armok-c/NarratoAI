---
phase: 07-sde-pipeline
fixed_at: 2026-05-08T12:30:00Z
review_path: .planning/phases/07-sde-pipeline/07-REVIEW.md
iteration: 12
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 07: Code Review Fix Report

**Fixed at:** 2026-05-08T12:30:00Z
**Source review:** .planning/phases/07-sde-pipeline/07-REVIEW.md
**Iteration:** 12

**Summary:**
- Findings in scope: 2 (WR-01, WR-02)
- Fixed: 2
- Skipped: 0

## Fixed Issues

### WR-01: SDE composite step missing amix volume compensation (regression from prior WR-01 fix)

**Files modified:** `narratoai-core/src/sde/pipeline.rs`
**Commit:** `1a1300d`
**Applied fix:** Added `volume=N` compensation after `amix=inputs=N:duration=longest` in the SDE pipeline's composite step (lines 506-514), matching the pattern already present in `documentary/pipeline.rs:375-383`. The compensation multiplies volume by the number of amix inputs to counteract FFmpeg's built-in 1/N normalization.

### WR-02: SdeRequest::validate() missing subtitle_font_size range check

**Files modified:** `narratoai-core/src/sde/types.rs`
**Commit:** `cfed052`
**Applied fix:** Added `subtitle_font_size` range validation `[1, 200]` in `SdeRequest::validate()` after the `threads == 0` check (lines 91-96), matching the identical check in `DocumentaryRequest::validate()` at `documentary/types.rs:85-90`.

## Skipped Issues

None -- all in-scope findings were fixed.

---

_Fixed: 2026-05-08T12:30:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 12_
