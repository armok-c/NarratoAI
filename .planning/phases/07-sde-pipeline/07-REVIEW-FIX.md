---
phase: 07-sde-pipeline
source_review: 07-REVIEW.md (sixth pass)
fix_date: 2026-05-06
findings_fixed: 5
findings_total: 5
fix_mode: critical_and_warning
status: all_fixed
---

# Phase 07: Code Review Fix Report (Sixth Pass)

**Source:** 07-REVIEW.md (sixth pass, 0 critical, 5 warnings, 1 info)
**Fixed:** 2026-05-06
**Scope:** Critical + Warning only (no `--all` flag)
**Status:** all_fixed

## Summary

All 5 warnings from the sixth-pass review have been fixed. No critical issues were found. The single info finding (IN-01: redundant "播放原片" check) was not in scope.

## Fixes Applied

### WR-01: Sanitize subtitle_font in documentary composite step

**File:** `src/documentary/pipeline.rs`
**Commit:** 7999544
**Fix:** Added character filtering (alphanumeric + space + dash + underscore) to `subtitle_font` before inserting into FFmpeg `force_style` string, matching the SDE pipeline's existing hardening.

### WR-02: Add newline sanitization in documentary concat file paths

**File:** `src/documentary/pipeline.rs`
**Commit:** 7999544
**Fix:** Added newline character check (`\n`, `\r`) for video paths before writing to FFmpeg concat list file. Returns `PipelineError::Concat` if illegal characters found, matching the SDE pipeline's validation.

### WR-03: Replace sync std::fs::write with async tokio::fs::write in documentary concat

**File:** `src/documentary/pipeline.rs`
**Commit:** 7999544
**Fix:** Replaced `std::fs::write(&concat_list_path, &concat_content)?` with `tokio::fs::write(...).await.map_err(PipelineError::from)?` in `step_concat`, consistent with async patterns elsewhere.

### WR-04: Replace sync write_srt_file with async tokio::fs::write in SDE TTS loop

**File:** `src/sde/pipeline.rs`
**Commit:** 7999544
**Fix:** Replaced `write_srt_file(&srt_content, &srt_path)?` with `tokio::fs::write(&srt_path, &srt_content).await.map_err(|e| SdeError::Io { source: e })?`. Removed unused `write_srt_file` import.

### WR-05: Reject tts_volume=0 in DocumentaryRequest::validate

**File:** `src/documentary/types.rs`
**Commit:** 7999544
**Fix:** Changed `self.tts_volume < 0.0` to `self.tts_volume <= 0.0` and updated error message range from `[0, 10]` to `(0, 10]`, matching SDE validation.

## Verification

- `cargo check`: passes with 0 new warnings (4 pre-existing warnings from earlier passes)
- `cargo test`: 513/514 tests pass; 1 pre-existing failure (`test_clip_video_async` — FFmpeg environment issue, fails identically on clean HEAD)
- 3 files changed, 24 insertions, 8 deletions

## Not in Scope

| ID | Severity | Description | Reason |
|----|----------|-------------|--------|
| IN-01 | Info | Redundant "播放原片" check in SDE TTS loop | Info-level, not included in default fix scope |

---
_Fixed: 2026-05-06_
_Fixer: Claude (gsd-code-review-fix)_
